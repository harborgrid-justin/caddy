//! Cache strategies for different access patterns and consistency requirements
//!
//! This module provides various caching strategies:
//! - Write-through: Synchronous writes to cache and backing store
//! - Write-behind: Asynchronous writes with eventual consistency
//! - Write-around: Bypass cache on writes
//! - Read-through: Automatic cache population on misses
//! - Cache-aside: Manual cache management
//! - Refresh-ahead: Proactive refresh of hot keys before expiration

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio::time::interval;
use crate::enterprise::error::EnterpriseResult;

/// Trait for backing store operations
#[async_trait]
pub trait BackingStore<K, V>: Send + Sync
where
    K: Send,
    V: Send,
{
    /// Load a value from the backing store
    async fn load(&self, key: &K) -> EnterpriseResult<Option<V>>;

    /// Save a value to the backing store
    async fn save(&self, key: &K, value: &V) -> EnterpriseResult<()>;

    /// Delete a value from the backing store
    async fn delete(&self, key: &K) -> EnterpriseResult<()>;

    /// Batch load multiple values
    async fn batch_load(&self, keys: &[K]) -> EnterpriseResult<HashMap<K, V>>
    where
        K: Clone + Eq + Hash + Sync,
    {
        let mut result = HashMap::new();
        for key in keys {
            let key = key.clone();
            if let Some(value) = self.load(&key).await? {
                result.insert(key, value);
            }
        }
        Ok(result)
    }
}

/// In-memory backing store implementation (for testing)
pub struct InMemoryStore<K, V> {
    data: Arc<DashMap<K, V>>,
}

impl<K, V> InMemoryStore<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new() -> Self {
        Self {
            data: Arc::new(DashMap::new()),
        }
    }
}

impl<K, V> Default for InMemoryStore<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<K, V> BackingStore<K, V> for InMemoryStore<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + Debug,
    V: Clone + Send + Sync,
{
    async fn load(&self, key: &K) -> EnterpriseResult<Option<V>> {
        Ok(self.data.get(key).map(|v| v.clone()))
    }

    async fn save(&self, key: &K, value: &V) -> EnterpriseResult<()> {
        self.data.insert(key.clone(), value.clone());
        Ok(())
    }

    async fn delete(&self, key: &K) -> EnterpriseResult<()> {
        self.data.remove(key);
        Ok(())
    }
}

/// Cache strategy type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyType {
    /// Synchronous write to cache and store
    WriteThrough,
    /// Asynchronous write with batching
    WriteBehind,
    /// Write only to store, invalidate cache
    WriteAround,
    /// Read from cache, load from store on miss
    ReadThrough,
    /// Manual cache management
    CacheAside,
    /// Proactive refresh of hot keys
    RefreshAhead,
}

/// Configuration for cache strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    /// Strategy type
    pub strategy: StrategyType,
    /// Write-behind batch size
    pub write_behind_batch_size: usize,
    /// Write-behind flush interval (milliseconds)
    pub write_behind_flush_interval_ms: u64,
    /// Refresh-ahead threshold (percentage of TTL)
    pub refresh_ahead_threshold: f64,
    /// Enable statistics collection
    pub enable_stats: bool,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            strategy: StrategyType::ReadThrough,
            write_behind_batch_size: 100,
            write_behind_flush_interval_ms: 1000,
            refresh_ahead_threshold: 0.8,
            enable_stats: true,
        }
    }
}

/// Cache entry with metadata
#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    created_at: Instant,
    ttl: Option<Duration>,
    access_count: u64,
}

impl<V> CacheEntry<V> {
    fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            self.created_at.elapsed() > ttl
        } else {
            false
        }
    }

    fn should_refresh(&self, threshold: f64) -> bool {
        if let Some(ttl) = self.ttl {
            let elapsed = self.created_at.elapsed().as_secs_f64();
            let ttl_secs = ttl.as_secs_f64();
            elapsed >= ttl_secs * threshold
        } else {
            false
        }
    }
}

/// Write-through cache strategy
pub struct WriteThroughCache<K, V, S>
where
    K: Send,
    V: Send,
    S: BackingStore<K, V>,
{
    cache: Arc<DashMap<K, CacheEntry<V>>>,
    store: Arc<S>,
    config: StrategyConfig,
}

impl<K, V, S> WriteThroughCache<K, V, S>
where
    K: Eq + Hash + Clone + Send + Sync + Debug,
    V: Clone + Send + Sync,
    S: BackingStore<K, V>,
{
    pub fn new(store: S) -> Self {
        Self::with_config(store, StrategyConfig::default())
    }

    pub fn with_config(store: S, config: StrategyConfig) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            store: Arc::new(store),
            config,
        }
    }

    /// Get a value from cache or store
    pub async fn get(&self, key: &K) -> EnterpriseResult<Option<V>> {
        // Check cache first
        if let Some(mut entry) = self.cache.get_mut(key) {
            if !entry.is_expired() {
                entry.access_count += 1;
                return Ok(Some(entry.value.clone()));
            }
            // Expired, remove from cache
            drop(entry);
            self.cache.remove(key);
        }

        // Load from store
        if let Some(value) = self.store.load(key).await? {
            let entry = CacheEntry {
                value: value.clone(),
                created_at: Instant::now(),
                ttl: None,
                access_count: 1,
            };
            self.cache.insert(key.clone(), entry);
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Write to both cache and store synchronously
    pub async fn put(&self, key: K, value: V, ttl: Option<Duration>) -> EnterpriseResult<()> {
        // Write to store first
        self.store.save(&key, &value).await?;

        // Then update cache
        let entry = CacheEntry {
            value,
            created_at: Instant::now(),
            ttl,
            access_count: 0,
        };
        self.cache.insert(key, entry);

        Ok(())
    }

    /// Remove from both cache and store
    pub async fn remove(&self, key: &K) -> EnterpriseResult<()> {
        self.store.delete(key).await?;
        self.cache.remove(key);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

/// Write-behind (write-back) cache strategy
pub struct WriteBehindCache<K, V, S>
where
    K: Send,
    V: Send,
    S: BackingStore<K, V>,
{
    cache: Arc<DashMap<K, CacheEntry<V>>>,
    store: Arc<S>,
    write_queue: Arc<RwLock<Vec<(K, V)>>>,
    config: StrategyConfig,
}

impl<K, V, S> WriteBehindCache<K, V, S>
where
    K: Eq + Hash + Clone + Send + Sync + Debug + 'static,
    V: Clone + Send + Sync + 'static,
    S: BackingStore<K, V> + 'static,
{
    pub fn new(store: S) -> Self {
        Self::with_config(store, StrategyConfig::default())
    }

    pub fn with_config(store: S, mut config: StrategyConfig) -> Self {
        config.strategy = StrategyType::WriteBehind;

        let cache = Self {
            cache: Arc::new(DashMap::new()),
            store: Arc::new(store),
            write_queue: Arc::new(RwLock::new(Vec::new())),
            config,
        };

        // Start background flush task
        cache.start_flush_task();

        cache
    }

    /// Get a value from cache or store
    pub async fn get(&self, key: &K) -> EnterpriseResult<Option<V>> {
        // Check cache first
        if let Some(mut entry) = self.cache.get_mut(key) {
            if !entry.is_expired() {
                entry.access_count += 1;
                return Ok(Some(entry.value.clone()));
            }
            drop(entry);
            self.cache.remove(key);
        }

        // Load from store
        if let Some(value) = self.store.load(key).await? {
            let entry = CacheEntry {
                value: value.clone(),
                created_at: Instant::now(),
                ttl: None,
                access_count: 1,
            };
            self.cache.insert(key.clone(), entry);
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Write to cache immediately, queue for async write to store
    pub async fn put(&self, key: K, value: V, ttl: Option<Duration>) -> EnterpriseResult<()> {
        // Update cache immediately
        let entry = CacheEntry {
            value: value.clone(),
            created_at: Instant::now(),
            ttl,
            access_count: 0,
        };
        self.cache.insert(key.clone(), entry);

        // Queue for async write
        let mut queue = self.write_queue.write().await;
        queue.push((key, value));

        // Flush if batch size reached
        if queue.len() >= self.config.write_behind_batch_size {
            drop(queue);
            self.flush().await?;
        }

        Ok(())
    }

    /// Flush pending writes to store
    pub async fn flush(&self) -> EnterpriseResult<()> {
        let mut queue = self.write_queue.write().await;
        if queue.is_empty() {
            return Ok(());
        }

        let items = std::mem::take(&mut *queue);
        drop(queue);

        for (key, value) in items {
            self.store.save(&key, &value).await?;
        }

        Ok(())
    }

    /// Start background flush task
    fn start_flush_task(&self) {
        let store = Arc::clone(&self.store);
        let write_queue = Arc::clone(&self.write_queue);
        let flush_interval = Duration::from_millis(self.config.write_behind_flush_interval_ms);

        tokio::spawn(async move {
            let mut ticker = interval(flush_interval);
            loop {
                ticker.tick().await;

                let mut queue = write_queue.write().await;
                if queue.is_empty() {
                    continue;
                }

                let items = std::mem::take(&mut *queue);
                drop(queue);

                for (key, value) in items {
                    let _ = store.save(&key, &value).await;
                }
            }
        });
    }

    pub async fn remove(&self, key: &K) -> EnterpriseResult<()> {
        self.store.delete(key).await?;
        self.cache.remove(key);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

/// Read-through cache strategy
pub struct ReadThroughCache<K, V, S>
where
    K: Send,
    V: Send,
    S: BackingStore<K, V>,
{
    cache: Arc<DashMap<K, CacheEntry<V>>>,
    store: Arc<S>,
    config: StrategyConfig,
}

impl<K, V, S> ReadThroughCache<K, V, S>
where
    K: Eq + Hash + Clone + Send + Sync + Debug,
    V: Clone + Send + Sync,
    S: BackingStore<K, V>,
{
    pub fn new(store: S) -> Self {
        Self::with_config(store, StrategyConfig::default())
    }

    pub fn with_config(store: S, mut config: StrategyConfig) -> Self {
        config.strategy = StrategyType::ReadThrough;

        Self {
            cache: Arc::new(DashMap::new()),
            store: Arc::new(store),
            config,
        }
    }

    /// Get value with automatic cache population on miss
    pub async fn get(&self, key: &K, ttl: Option<Duration>) -> EnterpriseResult<Option<V>> {
        // Check cache first
        if let Some(mut entry) = self.cache.get_mut(key) {
            if !entry.is_expired() {
                entry.access_count += 1;
                return Ok(Some(entry.value.clone()));
            }
            drop(entry);
            self.cache.remove(key);
        }

        // Cache miss - load from store and populate cache
        if let Some(value) = self.store.load(key).await? {
            let entry = CacheEntry {
                value: value.clone(),
                created_at: Instant::now(),
                ttl,
                access_count: 1,
            };
            self.cache.insert(key.clone(), entry);
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Invalidate cache entry (forces reload on next access)
    pub fn invalidate(&self, key: &K) {
        self.cache.remove(key);
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

/// Refresh-ahead cache strategy
pub struct RefreshAheadCache<K, V, S>
where
    K: Send,
    V: Send,
    S: BackingStore<K, V>,
{
    cache: Arc<DashMap<K, CacheEntry<V>>>,
    store: Arc<S>,
    config: StrategyConfig,
}

impl<K, V, S> RefreshAheadCache<K, V, S>
where
    K: Eq + Hash + Clone + Send + Sync + Debug + 'static,
    V: Clone + Send + Sync + 'static,
    S: BackingStore<K, V> + 'static,
{
    pub fn new(store: S, _default_ttl: Duration) -> Self {
        let mut config = StrategyConfig::default();
        config.strategy = StrategyType::RefreshAhead;

        Self {
            cache: Arc::new(DashMap::new()),
            store: Arc::new(store),
            config,
        }
    }

    /// Get value with proactive refresh before expiration
    pub async fn get(&self, key: &K, ttl: Duration) -> EnterpriseResult<Option<V>> {
        // Check cache
        if let Some(mut entry) = self.cache.get_mut(key) {
            if !entry.is_expired() {
                entry.access_count += 1;

                // Check if we should refresh proactively
                if entry.should_refresh(self.config.refresh_ahead_threshold) {
                    let key = key.clone();
                    let store = Arc::clone(&self.store);
                    let cache = Arc::clone(&self.cache);

                    // Trigger async refresh
                    tokio::spawn(async move {
                        if let Ok(Some(value)) = store.load(&key).await {
                            let entry = CacheEntry {
                                value,
                                created_at: Instant::now(),
                                ttl: Some(ttl),
                                access_count: 0,
                            };
                            cache.insert(key, entry);
                        }
                    });
                }

                return Ok(Some(entry.value.clone()));
            }
            drop(entry);
            self.cache.remove(key);
        }

        // Cache miss - load from store
        if let Some(value) = self.store.load(key).await? {
            let entry = CacheEntry {
                value: value.clone(),
                created_at: Instant::now(),
                ttl: Some(ttl),
                access_count: 1,
            };
            self.cache.insert(key.clone(), entry);
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    pub fn invalidate(&self, key: &K) {
        self.cache.remove(key);
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_store() {
        let store = InMemoryStore::new();

        store.save(&1, &"value1".to_string()).await.unwrap();
        let value = store.load(&1).await.unwrap();
        assert_eq!(value, Some("value1".to_string()));

        store.delete(&1).await.unwrap();
        let value = store.load(&1).await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_write_through_cache() {
        let store = InMemoryStore::new();
        let cache = WriteThroughCache::new(store);

        cache.put(1, "value1".to_string(), None).await.unwrap();
        let value = cache.get(&1).await.unwrap();
        assert_eq!(value, Some("value1".to_string()));

        cache.remove(&1).await.unwrap();
        let value = cache.get(&1).await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_write_behind_cache() {
        let store = InMemoryStore::new();
        let cache = WriteBehindCache::new(store);

        cache.put(1, "value1".to_string(), None).await.unwrap();

        // Value should be in cache immediately
        let value = cache.get(&1).await.unwrap();
        assert_eq!(value, Some("value1".to_string()));

        // Flush to store
        cache.flush().await.unwrap();
    }

    #[tokio::test]
    async fn test_read_through_cache() {
        let store = InMemoryStore::new();
        store.save(&1, &"value1".to_string()).await.unwrap();

        let cache = ReadThroughCache::new(store);

        // First access loads from store
        let value = cache.get(&1, None).await.unwrap();
        assert_eq!(value, Some("value1".to_string()));

        // Second access hits cache
        let value = cache.get(&1, None).await.unwrap();
        assert_eq!(value, Some("value1".to_string()));
    }

    #[tokio::test]
    async fn test_refresh_ahead_cache() {
        let store = InMemoryStore::new();
        store.save(&1, &"value1".to_string()).await.unwrap();

        let cache = RefreshAheadCache::new(store, Duration::from_secs(10));

        let value = cache.get(&1, Duration::from_secs(10)).await.unwrap();
        assert_eq!(value, Some("value1".to_string()));
    }

    #[tokio::test]
    async fn test_cache_ttl_expiration() {
        let store = InMemoryStore::new();
        let cache = WriteThroughCache::new(store);

        cache.put(1, "value1".to_string(), Some(Duration::from_millis(100))).await.unwrap();

        // Should exist initially
        assert_eq!(cache.get(&1).await.unwrap(), Some("value1".to_string()));

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should be expired
        assert_eq!(cache.get(&1).await.unwrap(), None);
    }
}
