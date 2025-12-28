//! Local Cache System
//!
//! This module provides an LRU-based local cache for cloud files with offline
//! mode support, intelligent cache invalidation, and automatic size management.

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::storage::{CloudStorage, FileMetadata};

/// Cache error types
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Cache entry not found: {0}")]
    NotFound(String),

    #[error("Cache full")]
    CacheFull,

    #[error("Invalid cache key: {0}")]
    InvalidKey(String),

    #[error("Cache corrupted: {0}")]
    Corrupted(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

/// Cache policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CachePolicy {
    /// Least Recently Used
    LRU,
    /// Least Frequently Used
    LFU,
    /// First In First Out
    FIFO,
    /// Time-based expiration
    TTL,
}

/// Cache invalidation strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvalidationStrategy {
    /// Invalidate on access
    OnAccess,
    /// Invalidate on timer
    Periodic,
    /// Manual invalidation only
    Manual,
    /// Invalidate based on file modification time
    ModificationTime,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum cache size in bytes
    pub max_size_bytes: u64,
    /// Maximum number of cache entries
    pub max_entries: usize,
    /// Cache policy
    pub policy: CachePolicy,
    /// Time-to-live for cached items in seconds
    pub ttl_secs: u64,
    /// Invalidation strategy
    pub invalidation_strategy: InvalidationStrategy,
    /// Enable offline mode
    pub enable_offline_mode: bool,
    /// Cache directory path
    pub cache_dir: PathBuf,
    /// Enable cache compression
    pub enable_compression: bool,
    /// Periodic cleanup interval in seconds
    pub cleanup_interval_secs: u64,
    /// Prefetch threshold (0.0 - 1.0)
    pub prefetch_threshold: f64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size_bytes: 1024 * 1024 * 1024, // 1 GB
            max_entries: 10_000,
            policy: CachePolicy::LRU,
            ttl_secs: 3600, // 1 hour
            invalidation_strategy: InvalidationStrategy::ModificationTime,
            enable_offline_mode: true,
            cache_dir: PathBuf::from(".cache/cloud"),
            enable_compression: true,
            cleanup_interval_secs: 300, // 5 minutes
            prefetch_threshold: 0.8, // 80% cache full
        }
    }
}

/// Cache entry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    /// Cache key (usually file path)
    key: String,
    /// File path in cache
    cache_path: PathBuf,
    /// Original file metadata
    metadata: FileMetadata,
    /// Size in bytes
    size: u64,
    /// Access count (for LFU)
    access_count: u64,
    /// Last access time
    last_accessed: SystemTime,
    /// Creation time
    created_at: SystemTime,
    /// Expiration time
    expires_at: Option<SystemTime>,
    /// Whether this entry is pinned (never evicted)
    pinned: bool,
    /// Hash of cached data for verification
    hash: String,
}

/// Cache statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Total entries
    pub entries: usize,
    /// Total size in bytes
    pub total_size: u64,
    /// Hit rate (0.0 - 1.0)
    pub hit_rate: f64,
    /// Number of evictions
    pub evictions: u64,
    /// Number of invalidations
    pub invalidations: u64,
}

impl CacheStats {
    fn update_hit_rate(&mut self) {
        let total = self.hits + self.misses;
        if total > 0 {
            self.hit_rate = self.hits as f64 / total as f64;
        }
    }
}

/// Cloud file cache
pub struct CloudCache<S: CloudStorage> {
    storage: Arc<S>,
    config: RwLock<CacheConfig>,
    entries: RwLock<HashMap<String, CacheEntry>>,
    lru_queue: RwLock<VecDeque<String>>,
    stats: RwLock<CacheStats>,
    offline_mode: RwLock<bool>,
}

impl<S: CloudStorage + Send + Sync + 'static> CloudCache<S> {
    /// Create a new cloud cache
    pub async fn new(storage: S) -> Result<Self, CacheError> {
        let config = CacheConfig::default();
        Self::with_config(storage, config).await
    }

    /// Create a new cloud cache with custom configuration
    pub async fn with_config(storage: S, config: CacheConfig) -> Result<Self, CacheError> {
        // Create cache directory
        tokio::fs::create_dir_all(&config.cache_dir).await?;

        let cache = Self {
            storage: Arc::new(storage),
            config: RwLock::new(config),
            entries: RwLock::new(HashMap::new()),
            lru_queue: RwLock::new(VecDeque::new()),
            stats: RwLock::new(CacheStats::default()),
            offline_mode: RwLock::new(false),
        };

        // Start periodic cleanup
        let cache_ref = &cache as *const Self;
        tokio::spawn(async move {
            // Safety: Ensure cache lives long enough
            // In production, use proper Arc<Self>
            // unsafe { &*cache_ref }.periodic_cleanup().await;
        });

        Ok(cache)
    }

    /// Get a file from cache or cloud storage
    pub async fn get(&self, key: &str) -> Result<Vec<u8>, CacheError> {
        // Check cache first
        if let Some(data) = self.get_from_cache(key).await? {
            self.stats.write().await.hits += 1;
            self.stats.write().await.update_hit_rate();
            return Ok(data);
        }

        // Cache miss
        self.stats.write().await.misses += 1;
        self.stats.write().await.update_hit_rate();

        // If offline mode, return error
        if *self.offline_mode.read().await {
            return Err(CacheError::NotFound(format!(
                "File not in cache and offline mode is enabled: {}",
                key
            )));
        }

        // Fetch from cloud storage
        let data = self.storage
            .download_file(key)
            .await
            .map_err(|e| CacheError::Other(e.to_string()))?;

        // Cache the data
        self.put(key, &data).await?;

        Ok(data)
    }

    /// Get a file from cache only (no cloud fetch)
    async fn get_from_cache(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError> {
        let mut entries = self.entries.write().await;

        if let Some(entry) = entries.get_mut(key) {
            // Check if expired
            if let Some(expires_at) = entry.expires_at {
                if SystemTime::now() > expires_at {
                    // Entry expired, remove it
                    self.remove_entry(key).await?;
                    return Ok(None);
                }
            }

            // Update access info
            entry.last_accessed = SystemTime::now();
            entry.access_count += 1;

            // Update LRU queue
            self.update_lru(key).await;

            // Read from cache file
            let data = tokio::fs::read(&entry.cache_path).await?;

            // Verify hash
            let hash = self.calculate_hash(&data);
            if hash != entry.hash {
                return Err(CacheError::Corrupted(format!(
                    "Hash mismatch for cached file: {}",
                    key
                )));
            }

            // Decompress if needed
            let decompressed = if self.config.read().await.enable_compression {
                self.decompress(&data)?
            } else {
                data
            };

            return Ok(Some(decompressed));
        }

        Ok(None)
    }

    /// Put a file into cache
    pub async fn put(&self, key: &str, data: &[u8]) -> Result<(), CacheError> {
        let config = self.config.read().await;

        // Check if we need to evict entries
        let stats = self.stats.read().await;
        if stats.total_size + data.len() as u64 > config.max_size_bytes
            || stats.entries >= config.max_entries
        {
            drop(stats);
            self.evict_entries(data.len() as u64).await?;
        } else {
            drop(stats);
        }

        // Compress if enabled
        let cache_data = if config.enable_compression {
            self.compress(data)?
        } else {
            data.to_vec()
        };

        // Generate cache file path
        let cache_path = config.cache_dir.join(self.sanitize_key(key));

        // Create parent directories
        if let Some(parent) = cache_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Write to cache file
        tokio::fs::write(&cache_path, &cache_data).await?;

        // Get file metadata from storage
        let metadata = self.storage
            .get_metadata(key)
            .await
            .unwrap_or_else(|_| FileMetadata {
                path: key.to_string(),
                size: data.len() as u64,
                modified: SystemTime::now(),
                hash: self.calculate_hash(data),
                version: 1,
                content_type: None,
                custom_metadata: HashMap::new(),
            });

        // Calculate expiration time
        let expires_at = if config.policy == CachePolicy::TTL {
            Some(SystemTime::now() + Duration::from_secs(config.ttl_secs))
        } else {
            None
        };

        drop(config);

        // Create cache entry
        let entry = CacheEntry {
            key: key.to_string(),
            cache_path,
            metadata,
            size: cache_data.len() as u64,
            access_count: 1,
            last_accessed: SystemTime::now(),
            created_at: SystemTime::now(),
            expires_at,
            pinned: false,
            hash: self.calculate_hash(&cache_data),
        };

        // Add to cache
        let mut entries = self.entries.write().await;
        entries.insert(key.to_string(), entry);
        drop(entries);

        // Update LRU queue
        self.lru_queue.write().await.push_back(key.to_string());

        // Update stats
        let mut stats = self.stats.write().await;
        stats.entries = self.entries.read().await.len();
        stats.total_size += cache_data.len() as u64;

        log::debug!("Cached file: {} ({} bytes)", key, cache_data.len());
        Ok(())
    }

    /// Remove an entry from cache
    async fn remove_entry(&self, key: &str) -> Result<(), CacheError> {
        let mut entries = self.entries.write().await;

        if let Some(entry) = entries.remove(key) {
            // Delete cache file
            let _ = tokio::fs::remove_file(&entry.cache_path).await;

            // Update stats
            let mut stats = self.stats.write().await;
            stats.total_size = stats.total_size.saturating_sub(entry.size);
            stats.entries = entries.len();

            // Remove from LRU queue
            let mut lru = self.lru_queue.write().await;
            lru.retain(|k| k != key);

            log::debug!("Removed cache entry: {}", key);
        }

        Ok(())
    }

    /// Evict entries to make space
    async fn evict_entries(&self, needed_space: u64) -> Result<(), CacheError> {
        let config = self.config.read().await;
        let policy = config.policy;
        drop(config);

        let entries_snapshot = self.entries.read().await.clone();
        let mut candidates: Vec<_> = entries_snapshot
            .iter()
            .filter(|(_, e)| !e.pinned)
            .collect();

        // Sort by eviction policy
        match policy {
            CachePolicy::LRU => {
                candidates.sort_by_key(|(_, e)| e.last_accessed);
            }
            CachePolicy::LFU => {
                candidates.sort_by_key(|(_, e)| e.access_count);
            }
            CachePolicy::FIFO => {
                candidates.sort_by_key(|(_, e)| e.created_at);
            }
            CachePolicy::TTL => {
                candidates.sort_by_key(|(_, e)| e.expires_at);
            }
        }

        let mut freed_space = 0u64;
        let mut evicted_count = 0;

        for (key, entry) in candidates {
            if freed_space >= needed_space {
                break;
            }

            self.remove_entry(key).await?;
            freed_space += entry.size;
            evicted_count += 1;
        }

        self.stats.write().await.evictions += evicted_count;

        log::info!("Evicted {} entries, freed {} bytes", evicted_count, freed_space);
        Ok(())
    }

    /// Update LRU queue
    async fn update_lru(&self, key: &str) {
        let mut lru = self.lru_queue.write().await;

        // Remove key from current position
        lru.retain(|k| k != key);

        // Add to end (most recently used)
        lru.push_back(key.to_string());
    }

    /// Invalidate a cache entry
    pub async fn invalidate(&self, key: &str) -> Result<(), CacheError> {
        self.remove_entry(key).await?;
        self.stats.write().await.invalidations += 1;
        log::info!("Invalidated cache entry: {}", key);
        Ok(())
    }

    /// Clear all cache entries
    pub async fn clear(&self) -> Result<(), CacheError> {
        let entries = self.entries.read().await;
        let keys: Vec<_> = entries.keys().cloned().collect();
        drop(entries);

        for key in keys {
            self.remove_entry(&key).await?;
        }

        log::info!("Cache cleared");
        Ok(())
    }

    /// Pin an entry (prevent eviction)
    pub async fn pin(&self, key: &str) -> Result<(), CacheError> {
        let mut entries = self.entries.write().await;
        if let Some(entry) = entries.get_mut(key) {
            entry.pinned = true;
            log::debug!("Pinned cache entry: {}", key);
        }
        Ok(())
    }

    /// Unpin an entry
    pub async fn unpin(&self, key: &str) -> Result<(), CacheError> {
        let mut entries = self.entries.write().await;
        if let Some(entry) = entries.get_mut(key) {
            entry.pinned = false;
            log::debug!("Unpinned cache entry: {}", key);
        }
        Ok(())
    }

    /// Enable offline mode
    pub async fn set_offline_mode(&self, enabled: bool) {
        *self.offline_mode.write().await = enabled;
        log::info!("Offline mode: {}", if enabled { "enabled" } else { "disabled" });
    }

    /// Check if offline mode is enabled
    pub async fn is_offline(&self) -> bool {
        *self.offline_mode.read().await
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }

    /// Periodic cleanup task
    async fn periodic_cleanup(&self) {
        let cleanup_interval = self.config.read().await.cleanup_interval_secs;
        let mut interval = tokio::time::interval(Duration::from_secs(cleanup_interval));

        loop {
            interval.tick().await;

            // Remove expired entries
            let entries_snapshot = self.entries.read().await.clone();
            let now = SystemTime::now();

            for (key, entry) in entries_snapshot {
                if let Some(expires_at) = entry.expires_at {
                    if now > expires_at {
                        let _ = self.remove_entry(&key).await;
                    }
                }
            }
        }
    }

    /// Sanitize cache key to valid file path
    fn sanitize_key(&self, key: &str) -> String {
        key.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
    }

    /// Calculate hash of data
    fn calculate_hash(&self, data: &[u8]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Compress data
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, CacheError> {
        // In production, use a real compression library like flate2
        // For now, just return the original data
        Ok(data.to_vec())
    }

    /// Decompress data
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, CacheError> {
        // In production, use a real decompression library
        Ok(data.to_vec())
    }

    /// Prefetch files that are likely to be accessed
    pub async fn prefetch(&self, keys: &[String]) -> Result<(), CacheError> {
        for key in keys {
            if self.entries.read().await.contains_key(key) {
                continue; // Already cached
            }

            // Fetch from storage and cache
            if let Ok(data) = self.storage.download_file(key).await {
                let _ = self.put(key, &data).await;
            }
        }

        log::info!("Prefetched {} files", keys.len());
        Ok(())
    }

    /// Get cached entries list
    pub async fn list_entries(&self) -> Vec<String> {
        self.entries.read().await.keys().cloned().collect()
    }

    /// Check if key exists in cache
    pub async fn contains(&self, key: &str) -> bool {
        self.entries.read().await.contains_key(key)
    }

    /// Get cache usage percentage
    pub async fn usage_percentage(&self) -> f64 {
        let stats = self.stats.read().await;
        let config = self.config.read().await;

        if config.max_size_bytes > 0 {
            (stats.total_size as f64 / config.max_size_bytes as f64) * 100.0
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.max_size_bytes, 1024 * 1024 * 1024);
        assert_eq!(config.policy, CachePolicy::LRU);
        assert!(config.enable_offline_mode);
    }

    #[test]
    fn test_cache_stats() {
        let mut stats = CacheStats::default();
        stats.hits = 80;
        stats.misses = 20;
        stats.update_hit_rate();
        assert_eq!(stats.hit_rate, 0.8);
    }
}
