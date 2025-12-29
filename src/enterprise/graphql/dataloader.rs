//! GraphQL DataLoader
//!
//! Provides batching and caching to solve the N+1 query problem.
//! DataLoaders batch multiple individual loads into a single batch request
//! and cache results within a single request context.

use async_trait::async_trait;
use dashmap::DashMap;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::{Mutex, RwLock};
use tokio::time::sleep;

// ============================================================================
// Error Types
// ============================================================================

/// DataLoader errors
#[derive(Error, Debug, Clone)]
pub enum DataLoaderError {
    /// Batch load failed
    #[error("Batch load failed: {0}")]
    BatchLoadFailed(String),

    /// Key not found
    #[error("Key not found: {0}")]
    KeyNotFound(String),

    /// Cache error
    #[error("Cache error: {0}")]
    CacheError(String),

    /// Timeout waiting for batch
    #[error("Timeout waiting for batch")]
    Timeout,
}

pub type DataLoaderResult<T> = Result<T, DataLoaderError>;

// ============================================================================
// Batch Loader Trait
// ============================================================================

/// Trait for implementing batch loading logic
#[async_trait]
pub trait BatchLoadFn<K, V>: Send + Sync
where
    K: Eq + Hash + Clone + Send + Sync + Debug,
    V: Clone + Send + Sync,
{
    /// Load multiple values by their keys
    ///
    /// The returned HashMap should contain an entry for each input key.
    /// Missing keys will be treated as null/not found.
    async fn load(&self, keys: &[K]) -> DataLoaderResult<HashMap<K, V>>;
}

// ============================================================================
// DataLoader Configuration
// ============================================================================

/// DataLoader configuration
#[derive(Debug, Clone)]
pub struct DataLoaderConfig {
    /// Maximum batch size
    pub max_batch_size: usize,
    /// Batch delay in milliseconds (time to wait for batching)
    pub batch_delay_ms: u64,
    /// Enable caching
    pub cache_enabled: bool,
    /// Cache TTL in seconds (None = cache for request lifetime)
    pub cache_ttl: Option<u64>,
    /// Maximum number of cached items
    pub max_cache_size: Option<usize>,
}

impl Default for DataLoaderConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 100,
            batch_delay_ms: 10,
            cache_enabled: true,
            cache_ttl: None,
            max_cache_size: Some(10000),
        }
    }
}

impl DataLoaderConfig {
    /// Create config with batching disabled
    pub fn no_batching() -> Self {
        Self {
            max_batch_size: 1,
            batch_delay_ms: 0,
            ..Default::default()
        }
    }

    /// Create config with caching disabled
    pub fn no_caching() -> Self {
        Self {
            cache_enabled: false,
            ..Default::default()
        }
    }
}

// ============================================================================
// Cache Entry
// ============================================================================

/// Cached value with metadata
#[derive(Debug, Clone)]
struct CacheEntry<V> {
    /// Cached value
    value: V,
    /// Time when entry was created
    created_at: Instant,
}

impl<V> CacheEntry<V> {
    /// Create a new cache entry
    fn new(value: V) -> Self {
        Self {
            value,
            created_at: Instant::now(),
        }
    }

    /// Check if entry is expired
    fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }
}

// ============================================================================
// DataLoader Cache
// ============================================================================

/// Thread-safe cache for DataLoader
struct DataLoaderCache<K, V>
where
    K: Eq + Hash + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    /// Cache storage
    cache: DashMap<K, CacheEntry<V>>,
    /// Configuration
    config: DataLoaderConfig,
}

impl<K, V> DataLoaderCache<K, V>
where
    K: Eq + Hash + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    /// Create a new cache
    fn new(config: DataLoaderConfig) -> Self {
        Self {
            cache: DashMap::new(),
            config,
        }
    }

    /// Get value from cache
    fn get(&self, key: &K) -> Option<V> {
        if !self.config.cache_enabled {
            return None;
        }

        self.cache.get(key).and_then(|entry| {
            // Check if expired
            if let Some(ttl) = self.config.cache_ttl {
                if entry.is_expired(Duration::from_secs(ttl)) {
                    return None;
                }
            }
            Some(entry.value.clone())
        })
    }

    /// Insert value into cache
    fn insert(&self, key: K, value: V) {
        if !self.config.cache_enabled {
            return;
        }

        // Check cache size limit
        if let Some(max_size) = self.config.max_cache_size {
            if self.cache.len() >= max_size {
                // Simple eviction: clear 10% of cache
                let to_remove = max_size / 10;
                let keys: Vec<K> = self.cache.iter().take(to_remove).map(|e| e.key().clone()).collect();
                for key in keys {
                    self.cache.remove(&key);
                }
            }
        }

        self.cache.insert(key, CacheEntry::new(value));
    }

    /// Clear all cached values
    fn clear(&self) {
        self.cache.clear();
    }

    /// Get cache size
    fn len(&self) -> usize {
        self.cache.len()
    }
}

// ============================================================================
// Batch Queue
// ============================================================================

/// Pending batch request
struct BatchRequest<K, V>
where
    K: Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    /// Keys to load
    keys: Vec<K>,
    /// Sender for results
    sender: tokio::sync::oneshot::Sender<DataLoaderResult<HashMap<K, V>>>,
}

/// Batch queue manager
struct BatchQueue<K, V>
where
    K: Eq + Hash + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    /// Pending requests
    pending: Mutex<Vec<BatchRequest<K, V>>>,
    /// Configuration
    config: DataLoaderConfig,
}

impl<K, V> BatchQueue<K, V>
where
    K: Eq + Hash + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    /// Create a new batch queue
    fn new(config: DataLoaderConfig) -> Self {
        Self {
            pending: Mutex::new(Vec::new()),
            config,
        }
    }

    /// Add a request to the queue
    async fn enqueue(
        &self,
        keys: Vec<K>,
    ) -> tokio::sync::oneshot::Receiver<DataLoaderResult<HashMap<K, V>>> {
        let (tx, rx) = tokio::sync::oneshot::channel();

        let mut pending = self.pending.lock().await;
        pending.push(BatchRequest {
            keys,
            sender: tx,
        });

        rx
    }

    /// Drain all pending requests
    async fn drain(&self) -> Vec<BatchRequest<K, V>> {
        let mut pending = self.pending.lock().await;
        std::mem::take(&mut *pending)
    }
}

// ============================================================================
// DataLoader
// ============================================================================

/// DataLoader for batching and caching
pub struct DataLoader<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + Debug + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Batch loading function
    batch_fn: Arc<dyn BatchLoadFn<K, V>>,
    /// Cache
    cache: Arc<DataLoaderCache<K, V>>,
    /// Batch queue
    queue: Arc<BatchQueue<K, V>>,
    /// Configuration
    config: DataLoaderConfig,
    /// Background task handle
    _task_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl<K, V> DataLoader<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + Debug + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Create a new DataLoader
    pub fn new(batch_fn: Arc<dyn BatchLoadFn<K, V>>) -> Self {
        Self::with_config(batch_fn, DataLoaderConfig::default())
    }

    /// Create a DataLoader with custom configuration
    pub fn with_config(
        batch_fn: Arc<dyn BatchLoadFn<K, V>>,
        config: DataLoaderConfig,
    ) -> Self {
        let cache = Arc::new(DataLoaderCache::new(config.clone()));
        let queue = Arc::new(BatchQueue::new(config.clone()));

        let loader = Self {
            batch_fn: Arc::clone(&batch_fn),
            cache,
            queue: Arc::clone(&queue),
            config: config.clone(),
            _task_handle: Arc::new(RwLock::new(None)),
        };

        // Start background batch processor
        if config.batch_delay_ms > 0 {
            loader.start_batch_processor();
        }

        loader
    }

    /// Load a single value by key
    pub async fn load(&self, key: K) -> DataLoaderResult<Option<V>> {
        // Check cache first
        if let Some(value) = self.cache.get(&key) {
            return Ok(Some(value));
        }

        // Enqueue for batch loading
        let rx = self.queue.enqueue(vec![key.clone()]).await;

        // Wait for batch result (with timeout)
        let timeout = Duration::from_millis(self.config.batch_delay_ms * 10 + 1000);
        let result = tokio::time::timeout(timeout, rx)
            .await
            .map_err(|_| DataLoaderError::Timeout)?
            .map_err(|_| DataLoaderError::BatchLoadFailed("Channel closed".to_string()))?;

        match result {
            Ok(map) => {
                // Cache all results
                for (k, v) in &map {
                    self.cache.insert(k.clone(), v.clone());
                }
                Ok(map.get(&key).cloned())
            }
            Err(e) => Err(e),
        }
    }

    /// Load multiple values by keys
    pub async fn load_many(&self, keys: Vec<K>) -> DataLoaderResult<HashMap<K, V>> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let mut result = HashMap::new();
        let mut uncached_keys = Vec::new();

        // Check cache for each key
        for key in keys {
            if let Some(value) = self.cache.get(&key) {
                result.insert(key, value);
            } else {
                uncached_keys.push(key);
            }
        }

        // Load uncached keys
        if !uncached_keys.is_empty() {
            let rx = self.queue.enqueue(uncached_keys).await;
            let timeout = Duration::from_millis(self.config.batch_delay_ms * 10 + 1000);
            let batch_result = tokio::time::timeout(timeout, rx)
                .await
                .map_err(|_| DataLoaderError::Timeout)?
                .map_err(|_| DataLoaderError::BatchLoadFailed("Channel closed".to_string()))?;

            match batch_result {
                Ok(map) => {
                    // Cache and add to result
                    for (k, v) in map {
                        self.cache.insert(k.clone(), v.clone());
                        result.insert(k, v);
                    }
                }
                Err(e) => return Err(e),
            }
        }

        Ok(result)
    }

    /// Prime the cache with a value
    pub fn prime(&self, key: K, value: V) {
        self.cache.insert(key, value);
    }

    /// Clear the cache
    pub fn clear(&self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            size: self.cache.len(),
            max_size: self.config.max_cache_size,
            enabled: self.config.cache_enabled,
        }
    }

    /// Start background batch processor
    fn start_batch_processor(&self) {
        let batch_fn = Arc::clone(&self.batch_fn);
        let queue = Arc::clone(&self.queue);
        let delay_ms = self.config.batch_delay_ms;
        let max_batch_size = self.config.max_batch_size;

        tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(delay_ms)).await;

                let requests = queue.drain().await;
                if requests.is_empty() {
                    continue;
                }

                // Collect all keys
                let mut all_keys = Vec::new();
                for req in &requests {
                    all_keys.extend(req.keys.clone());
                }

                // Remove duplicates
                all_keys.sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));
                all_keys.dedup();

                // Split into batches
                let batches: Vec<_> = all_keys
                    .chunks(max_batch_size)
                    .map(|chunk| chunk.to_vec())
                    .collect();

                // Load all batches
                let mut results = HashMap::new();
                let mut batch_error = None;
                for batch in batches {
                    match batch_fn.load(&batch).await {
                        Ok(batch_result) => {
                            results.extend(batch_result);
                        }
                        Err(e) => {
                            batch_error = Some(e);
                            break;
                        }
                    }
                }

                // Send results or errors back to requesters
                if let Some(error) = batch_error {
                    // Send error to all waiting requests
                    for req in requests {
                        let _ = req.sender.send(Err(error.clone()));
                    }
                } else {
                    // Send results back to requesters
                    for req in requests {
                        let req_results: HashMap<K, V> = req
                            .keys
                            .iter()
                            .filter_map(|k| results.get(k).map(|v| (k.clone(), v.clone())))
                            .collect();
                        let _ = req.sender.send(Ok(req_results));
                    }
                }
            }
        });
    }
}

// ============================================================================
// Cache Statistics
// ============================================================================

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Current cache size
    pub size: usize,
    /// Maximum cache size
    pub max_size: Option<usize>,
    /// Whether caching is enabled
    pub enabled: bool,
}

// ============================================================================
// Helper Types
// ============================================================================

/// Simple batch loader implementation using a closure
pub struct SimpleBatchLoader<K, V, F>
where
    K: Eq + Hash + Clone + Send + Sync + Debug,
    V: Clone + Send + Sync,
    F: Fn(&[K]) -> HashMap<K, V> + Send + Sync,
{
    func: F,
    _phantom: std::marker::PhantomData<(K, V)>,
}

impl<K, V, F> SimpleBatchLoader<K, V, F>
where
    K: Eq + Hash + Clone + Send + Sync + Debug,
    V: Clone + Send + Sync,
    F: Fn(&[K]) -> HashMap<K, V> + Send + Sync,
{
    /// Create a new simple batch loader
    pub fn new(func: F) -> Self {
        Self {
            func,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<K, V, F> BatchLoadFn<K, V> for SimpleBatchLoader<K, V, F>
where
    K: Eq + Hash + Clone + Send + Sync + Debug,
    V: Clone + Send + Sync,
    F: Fn(&[K]) -> HashMap<K, V> + Send + Sync,
{
    async fn load(&self, keys: &[K]) -> DataLoaderResult<HashMap<K, V>> {
        Ok((self.func)(keys))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    struct TestBatchLoader;

    #[async_trait]
    impl BatchLoadFn<i32, String> for TestBatchLoader {
        async fn load(&self, keys: &[i32]) -> DataLoaderResult<HashMap<i32, String>> {
            let mut map = HashMap::new();
            for key in keys {
                map.insert(*key, format!("value_{}", key));
            }
            Ok(map)
        }
    }

    #[tokio::test]
    async fn test_dataloader_single_load() {
        let loader = DataLoader::new(Arc::new(TestBatchLoader));
        let result = loader.load(1).await.unwrap();
        assert_eq!(result, Some("value_1".to_string()));
    }

    #[tokio::test]
    async fn test_dataloader_batch_load() {
        let loader = DataLoader::new(Arc::new(TestBatchLoader));
        let result = loader.load_many(vec![1, 2, 3]).await.unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result.get(&1), Some(&"value_1".to_string()));
        assert_eq!(result.get(&2), Some(&"value_2".to_string()));
        assert_eq!(result.get(&3), Some(&"value_3".to_string()));
    }

    #[tokio::test]
    async fn test_dataloader_caching() {
        let loader = DataLoader::new(Arc::new(TestBatchLoader));

        // First load
        let result1 = loader.load(1).await.unwrap();
        assert_eq!(result1, Some("value_1".to_string()));

        // Second load should hit cache
        let stats_before = loader.cache_stats();
        let result2 = loader.load(1).await.unwrap();
        assert_eq!(result2, Some("value_1".to_string()));

        assert!(stats_before.enabled);
        assert!(stats_before.size > 0);
    }

    #[tokio::test]
    async fn test_dataloader_prime() {
        let loader = DataLoader::new(Arc::new(TestBatchLoader));
        loader.prime(42, "primed_value".to_string());

        let result = loader.load(42).await.unwrap();
        assert_eq!(result, Some("primed_value".to_string()));
    }

    #[tokio::test]
    async fn test_dataloader_clear() {
        let loader = DataLoader::new(Arc::new(TestBatchLoader));
        loader.load(1).await.unwrap();

        let stats_before = loader.cache_stats();
        assert!(stats_before.size > 0);

        loader.clear();

        let stats_after = loader.cache_stats();
        assert_eq!(stats_after.size, 0);
    }

    #[test]
    fn test_cache_config() {
        let config = DataLoaderConfig::default();
        assert_eq!(config.max_batch_size, 100);
        assert!(config.cache_enabled);

        let no_batch = DataLoaderConfig::no_batching();
        assert_eq!(no_batch.max_batch_size, 1);

        let no_cache = DataLoaderConfig::no_caching();
        assert!(!no_cache.cache_enabled);
    }
}
