//! # Multi-Tier Caching System
//!
//! Provides a comprehensive multi-tier caching system:
//! - L1: In-memory LRU cache (fastest, smallest capacity)
//! - L2: Disk-based cache using embedded KV store (medium speed, large capacity)
//! - L3: Distributed Redis cache (shared across instances)

use crate::database::{DatabaseError, Result};
use moka::future::Cache as MokaCache;
use parking_lot::RwLock;
use redis::aio::ConnectionManager;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sled::Db as SledDb;
use std::hash::Hasher;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock as TokioRwLock;

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Enable L1 memory cache
    pub enable_l1: bool,

    /// L1 cache capacity (number of entries)
    pub l1_capacity: usize,

    /// L1 cache TTL
    pub l1_ttl: Duration,

    /// Enable L2 disk cache
    pub enable_l2: bool,

    /// L2 cache directory
    pub l2_directory: PathBuf,

    /// L2 cache max size in bytes
    pub l2_max_size: u64,

    /// L2 cache TTL
    pub l2_ttl: Duration,

    /// Enable L3 distributed cache
    pub enable_l3: bool,

    /// L3 Redis connection string
    pub l3_redis_url: Option<String>,

    /// L3 cache TTL
    pub l3_ttl: Duration,

    /// Compression threshold (compress entries larger than this in bytes)
    pub compression_threshold: usize,

    /// Enable statistics collection
    pub enable_stats: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enable_l1: true,
            l1_capacity: 10000,
            l1_ttl: Duration::from_secs(300),
            enable_l2: true,
            l2_directory: PathBuf::from("./cache"),
            l2_max_size: 1024 * 1024 * 1024, // 1GB
            l2_ttl: Duration::from_secs(3600),
            enable_l3: false,
            l3_redis_url: None,
            l3_ttl: Duration::from_secs(1800),
            compression_threshold: 1024, // 1KB
            enable_stats: true,
        }
    }
}

/// Cache layer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheLayer {
    /// L1 memory cache
    L1,
    /// L2 disk cache
    L2,
    /// L3 distributed cache
    L3,
}

/// Cache entry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry<T> {
    /// The cached value
    value: T,

    /// When this entry was created
    created_at: u64, // Unix timestamp in seconds

    /// TTL in seconds
    ttl: u64,

    /// Size in bytes
    size: usize,

    /// Whether this entry is compressed
    compressed: bool,
}

impl<T> CacheEntry<T> {
    fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now > self.created_at + self.ttl
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// L1 statistics
    pub l1_hits: u64,
    pub l1_misses: u64,
    pub l1_evictions: u64,
    pub l1_size: usize,

    /// L2 statistics
    pub l2_hits: u64,
    pub l2_misses: u64,
    pub l2_evictions: u64,
    pub l2_size: u64,

    /// L3 statistics
    pub l3_hits: u64,
    pub l3_misses: u64,
    pub l3_size: u64,

    /// Overall statistics
    pub total_hits: u64,
    pub total_misses: u64,
    pub hit_rate: f64,

    /// Performance metrics
    pub avg_get_time_us: u64,
    pub avg_set_time_us: u64,
}

impl CacheStats {
    fn update_hit_rate(&mut self) {
        let total = self.total_hits + self.total_misses;
        if total > 0 {
            self.hit_rate = self.total_hits as f64 / total as f64;
        }
    }
}

/// Multi-tier cache manager
pub struct CacheManager {
    /// L1 in-memory cache (Moka for async support)
    l1_cache: Option<MokaCache<String, Vec<u8>>>,

    /// L2 disk cache (Sled)
    l2_cache: Option<Arc<SledDb>>,

    /// L3 distributed cache (Redis)
    l3_cache: Option<Arc<TokioRwLock<ConnectionManager>>>,

    /// Configuration
    config: CacheConfig,

    /// Statistics
    stats: Arc<RwLock<CacheStats>>,
}

impl CacheManager {
    /// Create a new cache manager
    pub async fn new(config: CacheConfig) -> Result<Self> {
        // Initialize L1 cache
        let l1_cache = if config.enable_l1 {
            Some(
                MokaCache::builder()
                    .max_capacity(config.l1_capacity as u64)
                    .time_to_live(config.l1_ttl)
                    .build(),
            )
        } else {
            None
        };

        // Initialize L2 cache
        let l2_cache = if config.enable_l2 {
            std::fs::create_dir_all(&config.l2_directory)
                .map_err(|e| DatabaseError::Cache(format!("Failed to create L2 cache directory: {}", e)))?;

            let db = sled::open(&config.l2_directory)
                .map_err(|e| DatabaseError::Cache(format!("Failed to open L2 cache: {}", e)))?;

            Some(Arc::new(db))
        } else {
            None
        };

        // Initialize L3 cache
        let l3_cache = if config.enable_l3 {
            if let Some(redis_url) = &config.l3_redis_url {
                let client = redis::Client::open(redis_url.as_str())
                    .map_err(|e| DatabaseError::Cache(format!("Failed to connect to Redis: {}", e)))?;

                let conn_manager = ConnectionManager::new(client).await
                    .map_err(|e| DatabaseError::Cache(format!("Failed to create Redis connection: {}", e)))?;

                Some(Arc::new(TokioRwLock::new(conn_manager)))
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            l1_cache,
            l2_cache,
            l3_cache,
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        })
    }

    /// Get a value from the cache
    pub async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: DeserializeOwned + Send + Sync,
    {
        let start = Instant::now();

        // Try L1 first
        if let Some(l1) = &self.l1_cache {
            if let Some(data) = l1.get(key).await {
                let entry: CacheEntry<T> = self.deserialize(&data)?;
                if !entry.is_expired() {
                    self.record_hit(CacheLayer::L1, start.elapsed());
                    return Ok(Some(entry.value));
                } else {
                    // Remove expired entry
                    l1.invalidate(key).await;
                }
            }
            self.stats.write().l1_misses += 1;
        }

        // Try L2
        if let Some(l2) = &self.l2_cache {
            if let Ok(Some(data)) = l2.get(key.as_bytes()) {
                let entry: CacheEntry<T> = self.deserialize(&data)?;
                if !entry.is_expired() {
                    // Promote to L1
                    if let Some(l1) = &self.l1_cache {
                        if let Ok(serialized) = self.serialize(&entry) {
                            l1.insert(key.to_string(), serialized).await;
                        }
                    }

                    self.record_hit(CacheLayer::L2, start.elapsed());
                    return Ok(Some(entry.value));
                } else {
                    // Remove expired entry
                    let _ = l2.remove(key.as_bytes());
                }
            }
            self.stats.write().l2_misses += 1;
        }

        // Try L3
        if let Some(l3) = &self.l3_cache {
            let mut conn = l3.write().await;
            let data: Option<Vec<u8>> = redis::cmd("GET")
                .arg(key)
                .query_async(&mut *conn)
                .await
                .map_err(|e| DatabaseError::Cache(format!("Redis GET failed: {}", e)))?;

            if let Some(data) = data {
                let entry: CacheEntry<T> = self.deserialize(&data)?;
                if !entry.is_expired() {
                    // Promote to L2 and L1
                    if let Ok(serialized) = self.serialize(&entry) {
                        if let Some(l2) = &self.l2_cache {
                            let _ = l2.insert(key.as_bytes(), serialized.as_slice());
                        }
                        if let Some(l1) = &self.l1_cache {
                            l1.insert(key.to_string(), serialized).await;
                        }
                    }

                    self.record_hit(CacheLayer::L3, start.elapsed());
                    return Ok(Some(entry.value));
                }
            }
            self.stats.write().l3_misses += 1;
        }

        self.stats.write().total_misses += 1;
        self.stats.write().update_hit_rate();

        Ok(None)
    }

    /// Set a value in the cache
    pub async fn set<T>(&self, key: &str, value: T, ttl: Option<Duration>) -> Result<()>
    where
        T: Serialize + Send + Sync,
    {
        let start = Instant::now();

        let ttl_secs = ttl.unwrap_or(self.config.l1_ttl).as_secs();

        let _entry = CacheEntry {
            value,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ttl: ttl_secs,
            size: 0, // Will be updated after serialization
            compressed: false,
        };

        let mut serialized = self.serialize(&entry)?;
        let size = serialized.len();

        // Compress if needed
        let compressed = if size > self.config.compression_threshold {
            match self.compress(&serialized) {
                Ok(compressed_data) => {
                    serialized = compressed_data;
                    true
                }
                Err(_) => false,
            }
        } else {
            false
        };

        // Set in L1
        if let Some(l1) = &self.l1_cache {
            l1.insert(key.to_string(), serialized.clone()).await;
        }

        // Set in L2
        if let Some(l2) = &self.l2_cache {
            l2.insert(key.as_bytes(), serialized.as_slice())
                .map_err(|e| DatabaseError::Cache(format!("L2 cache set failed: {}", e)))?;
        }

        // Set in L3
        if let Some(l3) = &self.l3_cache {
            let mut conn = l3.write().await;
            redis::cmd("SETEX")
                .arg(key)
                .arg(ttl_secs)
                .arg(&serialized)
                .query_async(&mut *conn)
                .await
                .map_err(|e| DatabaseError::Cache(format!("Redis SETEX failed: {}", e)))?;
        }

        let elapsed = start.elapsed();
        let mut stats = self.stats.write();
        stats.avg_set_time_us = (stats.avg_set_time_us + elapsed.as_micros() as u64) / 2;

        Ok(())
    }

    /// Delete a value from all cache layers
    pub async fn delete(&self, key: &str) -> Result<()> {
        // Delete from L1
        if let Some(l1) = &self.l1_cache {
            l1.invalidate(key).await;
        }

        // Delete from L2
        if let Some(l2) = &self.l2_cache {
            l2.remove(key.as_bytes())
                .map_err(|e| DatabaseError::Cache(format!("L2 cache delete failed: {}", e)))?;
        }

        // Delete from L3
        if let Some(l3) = &self.l3_cache {
            let mut conn = l3.write().await;
            redis::cmd("DEL")
                .arg(key)
                .query_async(&mut *conn)
                .await
                .map_err(|e| DatabaseError::Cache(format!("Redis DEL failed: {}", e)))?;
        }

        Ok(())
    }

    /// Clear all cache layers
    pub async fn clear(&self) -> Result<()> {
        // Clear L1
        if let Some(l1) = &self.l1_cache {
            l1.invalidate_all();
        }

        // Clear L2
        if let Some(l2) = &self.l2_cache {
            l2.clear()
                .map_err(|e| DatabaseError::Cache(format!("L2 cache clear failed: {}", e)))?;
        }

        // Clear L3 (this is dangerous - only clear keys with a specific prefix)
        // For now, we skip this to avoid accidentally clearing shared cache

        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let mut stats = self.stats.read().clone();

        // Update L1 size
        if let Some(l1) = &self.l1_cache {
            stats.l1_size = l1.entry_count() as usize;
        }

        // Update L2 size
        if let Some(l2) = &self.l2_cache {
            stats.l2_size = l2.size_on_disk().unwrap_or(0);
        }

        stats
    }

    /// Record a cache hit
    fn record_hit(&self, layer: CacheLayer, elapsed: Duration) {
        let mut stats = self.stats.write();

        match layer {
            CacheLayer::L1 => stats.l1_hits += 1,
            CacheLayer::L2 => stats.l2_hits += 1,
            CacheLayer::L3 => stats.l3_hits += 1,
        }

        stats.total_hits += 1;
        stats.avg_get_time_us = (stats.avg_get_time_us + elapsed.as_micros() as u64) / 2;
        stats.update_hit_rate();
    }

    /// Serialize a value
    fn serialize<T: Serialize>(&self, value: &T) -> Result<Vec<u8>> {
        bincode::serialize(value)
            .map_err(|e| DatabaseError::Serialization(format!("Serialization failed: {}", e)))
    }

    /// Deserialize a value
    fn deserialize<T: DeserializeOwned>(&self, data: &[u8]) -> Result<T> {
        bincode::deserialize(data)
            .map_err(|e| DatabaseError::Serialization(format!("Deserialization failed: {}", e)))
    }

    /// Compress data using LZ4
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        lz4::block::compress(data, None, false)
            .map_err(|e| DatabaseError::Cache(format!("Compression failed: {}", e)))
    }

    /// Decompress data using LZ4
    fn decompress(&self, data: &[u8], original_size: i32) -> Result<Vec<u8>> {
        lz4::block::decompress(data, Some(original_size))
            .map_err(|e| DatabaseError::Cache(format!("Decompression failed: {}", e)))
    }
}

/// Cache key builder for consistent key generation
pub struct CacheKeyBuilder {
    parts: Vec<String>,
}

impl CacheKeyBuilder {
    /// Create a new cache key builder
    pub fn new() -> Self {
        Self { parts: Vec::new() }
    }

    /// Add a part to the key
    pub fn part<T: ToString>(mut self, part: T) -> Self {
        self.parts.push(part.to_string());
        self
    }

    /// Build the final key
    pub fn build(self) -> String {
        self.parts.join(":")
    }
}

impl Default for CacheKeyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_creation() {
        let config = CacheConfig {
            enable_l1: true,
            enable_l2: false,
            enable_l3: false,
            ..Default::default()
        };

        let cache = CacheManager::new(config).await;
        assert!(cache.is_ok());
    }

    #[tokio::test]
    async fn test_l1_cache() {
        let config = CacheConfig {
            enable_l1: true,
            enable_l2: false,
            enable_l3: false,
            ..Default::default()
        };

        let cache = CacheManager::new(config).await.unwrap();

        // Set a value
        cache.set("test_key", "test_value".to_string(), None).await.unwrap();

        // Get the value
        let result: Option<String> = cache.get("test_key").await.unwrap();
        assert_eq!(result, Some("test_value".to_string()));

        // Delete the value
        cache.delete("test_key").await.unwrap();

        // Verify it's gone
        let result: Option<String> = cache.get("test_key").await.unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_cache_key_builder() {
        let key = CacheKeyBuilder::new()
            .part("user")
            .part(123)
            .part("profile")
            .build();

        assert_eq!(key, "user:123:profile");
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let config = CacheConfig {
            enable_l1: true,
            enable_l2: false,
            enable_l3: false,
            enable_stats: true,
            ..Default::default()
        };

        let cache = CacheManager::new(config).await.unwrap();

        cache.set("key1", "value1".to_string(), None).await.unwrap();
        let _: Option<String> = cache.get("key1").await.unwrap();

        let stats = cache.stats();
        assert_eq!(stats.total_hits, 1);
        assert!(stats.hit_rate > 0.0);
    }
}
