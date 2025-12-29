//! Database Caching Layer
//!
//! Provides multi-level caching with query result caching, entity caching,
//! and cache invalidation strategies. Includes Redis integration stub.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;
use parking_lot::RwLock;
use thiserror::Error;

/// Cache errors
#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Cache miss for key: {0}")]
    CacheMiss(String),

    #[error("Cache serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Cache backend error: {0}")]
    Backend(String),

    #[error("Invalid cache key: {0}")]
    InvalidKey(String),

    #[error("Cache expired for key: {0}")]
    Expired(String),

    #[error("Redis connection error: {0}")]
    Redis(String),
}

/// Cache invalidation strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheStrategy {
    /// Time-based expiration
    TimeToLive { seconds: u64 },
    /// Least Recently Used
    LRU { max_entries: usize },
    /// Least Frequently Used
    LFU { max_entries: usize },
    /// Write-through caching
    WriteThrough,
    /// Write-behind caching
    WriteBehind { flush_interval_seconds: u64 },
}

/// Cache entry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntryMetadata {
    /// When the entry was created
    created_at: DateTime<Utc>,
    /// When the entry was last accessed
    last_accessed: DateTime<Utc>,
    /// Number of times accessed
    access_count: u64,
    /// Time-to-live in seconds
    ttl_seconds: Option<u64>,
}

impl CacheEntryMetadata {
    fn new(ttl_seconds: Option<u64>) -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            last_accessed: now,
            access_count: 0,
            ttl_seconds,
        }
    }

    fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl_seconds {
            let age = Utc::now()
                .signed_duration_since(self.created_at)
                .num_seconds() as u64;
            age > ttl
        } else {
            false
        }
    }

    fn touch(&mut self) {
        self.last_accessed = Utc::now();
        self.access_count += 1;
    }
}

/// Cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    /// The cached value
    value: serde_json::Value,
    /// Entry metadata
    metadata: CacheEntryMetadata,
}

impl CacheEntry {
    fn new(value: serde_json::Value, ttl_seconds: Option<u64>) -> Self {
        Self {
            value,
            metadata: CacheEntryMetadata::new(ttl_seconds),
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Default TTL in seconds
    pub default_ttl: Option<u64>,
    /// Maximum cache size
    pub max_entries: usize,
    /// Invalidation strategy
    pub strategy: CacheStrategy,
    /// Enable cache statistics
    pub enable_stats: bool,
    /// Redis connection string (if using Redis)
    pub redis_url: Option<String>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            default_ttl: Some(300), // 5 minutes
            max_entries: 10000,
            strategy: CacheStrategy::TimeToLive { seconds: 300 },
            enable_stats: true,
            redis_url: None,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Total entries
    pub entries: usize,
    /// Total evictions
    pub evictions: u64,
    /// Total sets
    pub sets: u64,
    /// Total deletes
    pub deletes: u64,
}

impl CacheStats {
    /// Calculate hit ratio
    pub fn hit_ratio(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

/// Generic cache trait
pub trait Cache: Send + Sync {
    /// Get a value from cache
    fn get(&self, key: &str) -> Result<Option<serde_json::Value>, CacheError>;

    /// Set a value in cache
    fn set(&self, key: &str, value: serde_json::Value, ttl: Option<u64>) -> Result<(), CacheError>;

    /// Delete a value from cache
    fn delete(&self, key: &str) -> Result<bool, CacheError>;

    /// Check if key exists
    fn exists(&self, key: &str) -> Result<bool, CacheError>;

    /// Clear all cache entries
    fn clear(&self) -> Result<(), CacheError>;

    /// Get cache statistics
    fn stats(&self) -> CacheStats;

    /// Invalidate cache entries matching a pattern
    fn invalidate_pattern(&self, pattern: &str) -> Result<usize, CacheError>;
}

/// In-memory cache implementation
pub struct MemoryCache {
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
    config: CacheConfig,
    stats: Arc<RwLock<CacheStats>>,
}

impl MemoryCache {
    /// Create a new memory cache
    pub fn new(config: CacheConfig) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// Evict entries based on strategy
    fn evict_if_needed(&self) {
        let mut entries = self.entries.write();
        let current_size = entries.len();

        if current_size < self.config.max_entries {
            return;
        }

        let to_remove = current_size - self.config.max_entries + 1;

        match self.config.strategy {
            CacheStrategy::LRU { .. } => {
                // Remove least recently used entries
                let mut items: Vec<_> = entries
                    .iter()
                    .map(|(k, v)| (k.clone(), v.metadata.last_accessed))
                    .collect();

                items.sort_by_key(|(_, last_accessed)| *last_accessed);

                for (key, _) in items.iter().take(to_remove) {
                    entries.remove(key);
                    self.stats.write().evictions += 1;
                }
            }
            CacheStrategy::LFU { .. } => {
                // Remove least frequently used entries
                let mut items: Vec<_> = entries
                    .iter()
                    .map(|(k, v)| (k.clone(), v.metadata.access_count))
                    .collect();

                items.sort_by_key(|(_, count)| *count);

                for (key, _) in items.iter().take(to_remove) {
                    entries.remove(key);
                    self.stats.write().evictions += 1;
                }
            }
            _ => {
                // Remove oldest entries by default
                let mut items: Vec<_> = entries
                    .iter()
                    .map(|(k, v)| (k.clone(), v.metadata.created_at))
                    .collect();

                items.sort_by_key(|(_, created)| *created);

                for (key, _) in items.iter().take(to_remove) {
                    entries.remove(key);
                    self.stats.write().evictions += 1;
                }
            }
        }

        log::debug!("Evicted {} cache entries", to_remove);
    }

    /// Clean up expired entries
    pub fn cleanup_expired(&self) {
        let mut entries = self.entries.write();
        let before = entries.len();

        entries.retain(|_, entry| !entry.metadata.is_expired());

        let removed = before - entries.len();
        if removed > 0 {
            self.stats.write().evictions += removed as u64;
            log::debug!("Cleaned up {} expired cache entries", removed);
        }
    }
}

impl Cache for MemoryCache {
    fn get(&self, key: &str) -> Result<Option<serde_json::Value>, CacheError> {
        let mut entries = self.entries.write();

        if let Some(entry) = entries.get_mut(key) {
            if entry.metadata.is_expired() {
                entries.remove(key);
                self.stats.write().misses += 1;
                return Ok(None);
            }

            entry.metadata.touch();
            self.stats.write().hits += 1;
            Ok(Some(entry.value.clone()))
        } else {
            self.stats.write().misses += 1;
            Ok(None)
        }
    }

    fn set(&self, key: &str, value: serde_json::Value, ttl: Option<u64>) -> Result<(), CacheError> {
        self.evict_if_needed();

        let ttl = ttl.or(self.config.default_ttl);
        let _entry = CacheEntry::new(value, ttl);

        self.entries.write().insert(key.to_string(), entry);
        self.stats.write().sets += 1;

        Ok(())
    }

    fn delete(&self, key: &str) -> Result<bool, CacheError> {
        let removed = self.entries.write().remove(key).is_some();

        if removed {
            self.stats.write().deletes += 1;
        }

        Ok(removed)
    }

    fn exists(&self, key: &str) -> Result<bool, CacheError> {
        let entries = self.entries.read();

        if let Some(entry) = entries.get(key) {
            Ok(!entry.metadata.is_expired())
        } else {
            Ok(false)
        }
    }

    fn clear(&self) -> Result<(), CacheError> {
        let count = self.entries.read().len();
        self.entries.write().clear();
        self.stats.write().evictions += count as u64;

        log::info!("Cleared {} cache entries", count);

        Ok(())
    }

    fn stats(&self) -> CacheStats {
        let mut stats = self.stats.read().clone();
        stats.entries = self.entries.read().len();
        stats
    }

    fn invalidate_pattern(&self, pattern: &str) -> Result<usize, CacheError> {
        let mut entries = self.entries.write();
        let before = entries.len();

        // Simple pattern matching - in production would use regex
        entries.retain(|key, _| !key.contains(pattern));

        let removed = before - entries.len();
        if removed > 0 {
            self.stats.write().deletes += removed as u64;
            log::debug!("Invalidated {} cache entries matching pattern '{}'", removed, pattern);
        }

        Ok(removed)
    }
}

/// Redis cache implementation (stub)
pub struct RedisCache {
    config: CacheConfig,
    stats: Arc<RwLock<CacheStats>>,
    fallback: Arc<MemoryCache>,
}

impl RedisCache {
    /// Create a new Redis cache
    pub fn new(config: CacheConfig) -> Result<Self, CacheError> {
        // Validate Redis URL
        if config.redis_url.is_none() {
            return Err(CacheError::Backend(
                "Redis URL not configured".to_string(),
            ));
        }

        log::info!("Initializing Redis cache (stub implementation)");

        // Create fallback memory cache
        let fallback = Arc::new(MemoryCache::new(config.clone()));

        Ok(Self {
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
            fallback,
        })
    }

    /// Connect to Redis (stub)
    async fn connect(&self) -> Result<(), CacheError> {
        // In production, this would establish actual Redis connection
        log::debug!("Connecting to Redis at: {:?}", self.config.redis_url);
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(())
    }

    /// Execute Redis command (stub)
    async fn execute_command(&self, _command: &str) -> Result<serde_json::Value, CacheError> {
        // Stub implementation - would use actual Redis client
        Ok(serde_json::Value::Null)
    }
}

impl Cache for RedisCache {
    fn get(&self, key: &str) -> Result<Option<serde_json::Value>, CacheError> {
        // Stub - would query Redis
        // Fall back to memory cache for now
        self.fallback.get(key)
    }

    fn set(&self, key: &str, value: serde_json::Value, ttl: Option<u64>) -> Result<(), CacheError> {
        // Stub - would set in Redis
        // Fall back to memory cache for now
        self.fallback.set(key, value, ttl)
    }

    fn delete(&self, key: &str) -> Result<bool, CacheError> {
        // Stub - would delete from Redis
        self.fallback.delete(key)
    }

    fn exists(&self, key: &str) -> Result<bool, CacheError> {
        // Stub - would check Redis
        self.fallback.exists(key)
    }

    fn clear(&self) -> Result<(), CacheError> {
        // Stub - would flush Redis database
        self.fallback.clear()
    }

    fn stats(&self) -> CacheStats {
        self.stats.read().clone()
    }

    fn invalidate_pattern(&self, pattern: &str) -> Result<usize, CacheError> {
        // Stub - would use Redis KEYS pattern matching
        self.fallback.invalidate_pattern(pattern)
    }
}

/// Query result cache
pub struct QueryCache {
    cache: Arc<dyn Cache>,
}

impl QueryCache {
    /// Create a new query cache
    pub fn new(cache: Arc<dyn Cache>) -> Self {
        Self { cache }
    }

    /// Generate cache key from SQL query
    fn generate_key(&self, sql: &str, params: &[serde_json::Value]) -> String {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        sql.hash(&mut hasher);

        for param in params {
            param.to_string().hash(&mut hasher);
        }

        format!("query:{:x}", hasher.finish())
    }

    /// Cache a query result
    pub fn cache_query(
        &self,
        sql: &str,
        params: &[serde_json::Value],
        result: &serde_json::Value,
        ttl: Option<u64>,
    ) -> Result<(), CacheError> {
        let key = self.generate_key(sql, params);
        self.cache.set(&key, result.clone(), ttl)
    }

    /// Get cached query result
    pub fn get_cached_query(
        &self,
        sql: &str,
        params: &[serde_json::Value],
    ) -> Result<Option<serde_json::Value>, CacheError> {
        let key = self.generate_key(sql, params);
        self.cache.get(&key)
    }

    /// Invalidate queries for a specific table
    pub fn invalidate_table(&self, table: &str) -> Result<usize, CacheError> {
        self.cache.invalidate_pattern(table)
    }
}

/// Entity cache
pub struct EntityCache<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    cache: Arc<dyn Cache>,
    entity_type: std::marker::PhantomData<T>,
}

impl<T> EntityCache<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    /// Create a new entity cache
    pub fn new(cache: Arc<dyn Cache>) -> Self {
        Self {
            cache,
            entity_type: std::marker::PhantomData,
        }
    }

    /// Generate entity cache key
    fn generate_key(&self, id: &str) -> String {
        format!("entity:{}:{}", std::any::type_name::<T>(), id)
    }

    /// Cache an entity
    pub fn cache_entity(&self, id: &str, entity: &T, ttl: Option<u64>) -> Result<(), CacheError> {
        let key = self.generate_key(id);
        let value = serde_json::to_value(entity)?;
        self.cache.set(&key, value, ttl)
    }

    /// Get cached entity
    pub fn get_entity(&self, id: &str) -> Result<Option<T>, CacheError> {
        let key = self.generate_key(id);

        if let Some(value) = self.cache.get(&key)? {
            let entity: T = serde_json::from_value(value)?;
            Ok(Some(entity))
        } else {
            Ok(None)
        }
    }

    /// Invalidate entity
    pub fn invalidate_entity(&self, id: &str) -> Result<bool, CacheError> {
        let key = self.generate_key(id);
        self.cache.delete(&key)
    }

    /// Invalidate all entities of this type
    pub fn invalidate_all(&self) -> Result<usize, CacheError> {
        let pattern = format!("entity:{}:", std::any::type_name::<T>());
        self.cache.invalidate_pattern(&pattern)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_cache_basic() {
        let cache = MemoryCache::new(CacheConfig::default());

        // Set and get
        let value = serde_json::json!({"name": "test"});
        cache.set("key1", value.clone(), None).unwrap();

        let retrieved = cache.get("key1").unwrap();
        assert_eq!(retrieved, Some(value));

        // Delete
        let deleted = cache.delete("key1").unwrap();
        assert!(deleted);

        let retrieved = cache.get("key1").unwrap();
        assert_eq!(retrieved, None);
    }

    #[test]
    fn test_cache_expiration() {
        let cache = MemoryCache::new(CacheConfig {
            default_ttl: Some(1), // 1 second
            ..Default::default()
        });

        let value = serde_json::json!({"test": "data"});
        cache.set("key1", value, Some(1)).unwrap();

        // Should exist immediately
        assert!(cache.exists("key1").unwrap());

        // Wait for expiration
        std::thread::sleep(Duration::from_secs(2));

        // Should be expired
        let retrieved = cache.get("key1").unwrap();
        assert_eq!(retrieved, None);
    }

    #[test]
    fn test_cache_stats() {
        let cache = MemoryCache::new(CacheConfig::default());

        // Generate some hits and misses
        cache.set("key1", serde_json::json!(1), None).unwrap();
        cache.get("key1").unwrap(); // hit
        cache.get("key2").unwrap(); // miss

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.sets, 1);
    }

    #[test]
    fn test_pattern_invalidation() {
        let cache = MemoryCache::new(CacheConfig::default());

        cache.set("user:1", serde_json::json!(1), None).unwrap();
        cache.set("user:2", serde_json::json!(2), None).unwrap();
        cache.set("project:1", serde_json::json!(1), None).unwrap();

        // Invalidate all user entries
        let removed = cache.invalidate_pattern("user:").unwrap();
        assert_eq!(removed, 2);

        // project:1 should still exist
        assert!(cache.exists("project:1").unwrap());
    }
}
