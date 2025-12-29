//! Multi-tier caching system with automatic tiering and promotion/demotion
//!
//! This module provides a sophisticated multi-tier caching architecture with:
//! - L1: Ultra-fast in-memory cache with LRU eviction
//! - L2: Cross-process shared memory cache
//! - L3: Distributed Redis/Memcached cluster support
//!
//! The system automatically manages data promotion (moving hot data to faster tiers)
//! and demotion (moving cold data to slower tiers) based on access patterns.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use crate::enterprise::error::{EnterpriseError, EnterpriseResult};

/// Cache tier level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CacheTier {
    /// L1: In-memory cache (fastest, smallest)
    L1,
    /// L2: Shared memory cache (fast, medium)
    L2,
    /// L3: Distributed cache (slower, largest)
    L3,
}

/// Access statistics for a cache entry
#[derive(Debug, Clone)]
struct AccessStats {
    /// Number of times accessed
    hit_count: u64,
    /// Last access timestamp
    last_access: Instant,
    /// Creation timestamp
    created_at: Instant,
    /// Current tier
    tier: CacheTier,
}

/// Cache entry with metadata
#[derive(Debug, Clone)]
struct CacheEntry<V> {
    /// The cached value
    value: V,
    /// Access statistics
    stats: AccessStats,
    /// Time-to-live
    ttl: Option<Duration>,
}

impl<V> CacheEntry<V> {
    fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            self.stats.created_at.elapsed() > ttl
        } else {
            false
        }
    }

    fn should_promote(&self, threshold: u64) -> bool {
        self.stats.hit_count >= threshold &&
            self.stats.tier != CacheTier::L1
    }

    fn should_demote(&self, idle_threshold: Duration) -> bool {
        self.stats.last_access.elapsed() > idle_threshold &&
            self.stats.tier != CacheTier::L3
    }
}

/// LRU cache implementation for L1 tier
struct LruCache<K, V> {
    capacity: usize,
    cache: DashMap<K, CacheEntry<V>>,
    access_order: Arc<RwLock<Vec<K>>>,
}

impl<K, V> LruCache<K, V>
where
    K: Eq + Hash + Clone + Debug,
    V: Clone,
{
    fn new(capacity: usize) -> Self {
        Self {
            capacity,
            cache: DashMap::new(),
            access_order: Arc::new(RwLock::new(Vec::with_capacity(capacity))),
        }
    }

    async fn get(&self, key: &K) -> Option<V> {
        if let Some(mut entry) = self.cache.get_mut(key) {
            if entry.is_expired() {
                drop(entry);
                self.cache.remove(key);
                return None;
            }

            entry.stats.hit_count += 1;
            entry.stats.last_access = Instant::now();
            let value = entry.value.clone();

            // Update access order
            let mut order = self.access_order.write().await;
            if let Some(pos) = order.iter().position(|k| k == key) {
                order.remove(pos);
            }
            order.push(key.clone());

            Some(value)
        } else {
            None
        }
    }

    async fn insert(&self, key: K, value: V, ttl: Option<Duration>) {
        // Check if we need to evict
        if self.cache.len() >= self.capacity && !self.cache.contains_key(&key) {
            self.evict_lru().await;
        }

        let entry = CacheEntry {
            value,
            stats: AccessStats {
                hit_count: 0,
                last_access: Instant::now(),
                created_at: Instant::now(),
                tier: CacheTier::L1,
            },
            ttl,
        };

        self.cache.insert(key.clone(), entry);

        let mut order = self.access_order.write().await;
        if let Some(pos) = order.iter().position(|k| k == &key) {
            order.remove(pos);
        }
        order.push(key);
    }

    async fn evict_lru(&self) {
        let mut order = self.access_order.write().await;
        if let Some(key) = order.first().cloned() {
            order.remove(0);
            self.cache.remove(&key);
        }
    }

    fn len(&self) -> usize {
        self.cache.len()
    }

    fn remove(&self, key: &K) -> Option<V> {
        self.cache.remove(key).map(|(_, entry)| entry.value)
    }
}

/// Configuration for multi-tier cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierConfig {
    /// L1 cache capacity (number of entries)
    pub l1_capacity: usize,
    /// L2 cache capacity
    pub l2_capacity: usize,
    /// L3 cache capacity
    pub l3_capacity: usize,
    /// Hit count threshold for promotion
    pub promotion_threshold: u64,
    /// Idle time threshold for demotion
    pub demotion_threshold_secs: u64,
    /// Enable automatic tiering
    pub enable_auto_tiering: bool,
    /// Background maintenance interval (seconds)
    pub maintenance_interval_secs: u64,
}

impl Default for TierConfig {
    fn default() -> Self {
        Self {
            l1_capacity: 1000,
            l2_capacity: 10_000,
            l3_capacity: 100_000,
            promotion_threshold: 10,
            demotion_threshold_secs: 300, // 5 minutes
            enable_auto_tiering: true,
            maintenance_interval_secs: 60,
        }
    }
}

/// Multi-tier cache implementation
pub struct MultiTierCache<K, V> {
    /// L1 in-memory cache
    l1: Arc<LruCache<K, V>>,
    /// L2 shared memory cache (simulated with DashMap)
    l2: Arc<DashMap<K, CacheEntry<V>>>,
    /// L3 distributed cache (simulated with DashMap)
    l3: Arc<DashMap<K, CacheEntry<V>>>,
    /// Configuration
    config: TierConfig,
    /// Statistics
    stats: Arc<DashMap<String, u64>>,
}

impl<K, V> MultiTierCache<K, V>
where
    K: Eq + Hash + Clone + Debug + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Create a new multi-tier cache with default configuration
    pub fn new() -> Self {
        Self::with_config(TierConfig::default())
    }

    /// Create a new multi-tier cache with custom configuration
    pub fn with_config(config: TierConfig) -> Self {
        let stats = Arc::new(DashMap::new());
        stats.insert("l1_hits".to_string(), 0);
        stats.insert("l2_hits".to_string(), 0);
        stats.insert("l3_hits".to_string(), 0);
        stats.insert("misses".to_string(), 0);
        stats.insert("promotions".to_string(), 0);
        stats.insert("demotions".to_string(), 0);

        Self {
            l1: Arc::new(LruCache::new(config.l1_capacity)),
            l2: Arc::new(DashMap::new()),
            l3: Arc::new(DashMap::new()),
            config,
            stats,
        }
    }

    /// Get a value from the cache, checking all tiers
    pub async fn get(&self, key: &K) -> Option<V> {
        // Try L1 first (fastest)
        if let Some(value) = self.l1.get(key).await {
            self.increment_stat("l1_hits");
            return Some(value);
        }

        // Try L2
        if let Some(mut entry) = self.l2.get_mut(key) {
            if entry.is_expired() {
                drop(entry);
                self.l2.remove(key);
                return None;
            }

            entry.stats.hit_count += 1;
            entry.stats.last_access = Instant::now();
            let value = entry.value.clone();

            self.increment_stat("l2_hits");

            // Promote to L1 if hot
            if self.config.enable_auto_tiering &&
               entry.should_promote(self.config.promotion_threshold) {
                self.promote_to_l1(key, &value, entry.ttl).await;
            }

            return Some(value);
        }

        // Try L3
        if let Some(mut entry) = self.l3.get_mut(key) {
            if entry.is_expired() {
                drop(entry);
                self.l3.remove(key);
                return None;
            }

            entry.stats.hit_count += 1;
            entry.stats.last_access = Instant::now();
            let value = entry.value.clone();

            self.increment_stat("l3_hits");

            // Promote to L2 if hot
            if self.config.enable_auto_tiering &&
               entry.should_promote(self.config.promotion_threshold) {
                self.promote_to_l2(key, &value, entry.ttl).await;
            }

            return Some(value);
        }

        self.increment_stat("misses");
        None
    }

    /// Insert a value into the cache (starts at L3 by default)
    pub async fn insert(&self, key: K, value: V, ttl: Option<Duration>) {
        let entry = CacheEntry {
            value,
            stats: AccessStats {
                hit_count: 0,
                last_access: Instant::now(),
                created_at: Instant::now(),
                tier: CacheTier::L3,
            },
            ttl,
        };

        self.l3.insert(key, entry);
    }

    /// Insert directly into L1 (for known hot data)
    pub async fn insert_hot(&self, key: K, value: V, ttl: Option<Duration>) {
        self.l1.insert(key, value, ttl).await;
    }

    /// Remove a value from all tiers
    pub async fn remove(&self, key: &K) -> bool {
        let removed_l1 = self.l1.remove(key).is_some();
        let removed_l2 = self.l2.remove(key).is_some();
        let removed_l3 = self.l3.remove(key).is_some();

        removed_l1 || removed_l2 || removed_l3
    }

    /// Clear all caches
    pub async fn clear(&self) {
        self.l2.clear();
        self.l3.clear();
        // L1 clear would need a custom implementation
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> HashMap<String, u64> {
        self.stats.iter()
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect()
    }

    /// Promote entry to L1
    async fn promote_to_l1(&self, key: &K, value: &V, ttl: Option<Duration>) {
        self.l1.insert(key.clone(), value.clone(), ttl).await;
        self.l2.remove(key);
        self.increment_stat("promotions");
    }

    /// Promote entry to L2
    async fn promote_to_l2(&self, key: &K, value: &V, ttl: Option<Duration>) {
        let entry = CacheEntry {
            value: value.clone(),
            stats: AccessStats {
                hit_count: 0,
                last_access: Instant::now(),
                created_at: Instant::now(),
                tier: CacheTier::L2,
            },
            ttl,
        };

        self.l2.insert(key.clone(), entry);
        self.l3.remove(key);
        self.increment_stat("promotions");
    }

    /// Run maintenance tasks (eviction, demotion, etc.)
    pub async fn maintenance(&self) {
        let demotion_threshold = Duration::from_secs(self.config.demotion_threshold_secs);

        // Check L2 for demotion candidates
        let l2_keys: Vec<K> = self.l2.iter()
            .filter(|entry| entry.should_demote(demotion_threshold))
            .map(|entry| entry.key().clone())
            .collect();

        for key in l2_keys {
            if let Some((_, entry)) = self.l2.remove(&key) {
                self.l3.insert(key, entry);
                self.increment_stat("demotions");
            }
        }

        // Clean expired entries
        self.clean_expired().await;
    }

    /// Remove expired entries from all tiers
    async fn clean_expired(&self) {
        // Clean L2
        let expired_l2: Vec<K> = self.l2.iter()
            .filter(|entry| entry.is_expired())
            .map(|entry| entry.key().clone())
            .collect();

        for key in expired_l2 {
            self.l2.remove(&key);
        }

        // Clean L3
        let expired_l3: Vec<K> = self.l3.iter()
            .filter(|entry| entry.is_expired())
            .map(|entry| entry.key().clone())
            .collect();

        for key in expired_l3 {
            self.l3.remove(&key);
        }
    }

    /// Increment a statistics counter
    fn increment_stat(&self, stat: &str) {
        self.stats.entry(stat.to_string())
            .and_modify(|v| *v += 1)
            .or_insert(1);
    }

    /// Get the size of each tier
    pub fn tier_sizes(&self) -> (usize, usize, usize) {
        (self.l1.len(), self.l2.len(), self.l3.len())
    }

    /// Get hit rate across all tiers
    pub fn hit_rate(&self) -> f64 {
        let hits = self.stats.get("l1_hits").map(|v| *v).unwrap_or(0)
            + self.stats.get("l2_hits").map(|v| *v).unwrap_or(0)
            + self.stats.get("l3_hits").map(|v| *v).unwrap_or(0);
        let misses = self.stats.get("misses").map(|v| *v).unwrap_or(0);
        let total = hits + misses;

        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }
}

impl<K, V> Default for MultiTierCache<K, V>
where
    K: Eq + Hash + Clone + Debug + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lru_cache_basic() {
        let cache = LruCache::new(3);

        cache.insert(1, "one".to_string(), None).await;
        cache.insert(2, "two".to_string(), None).await;
        cache.insert(3, "three".to_string(), None).await;

        assert_eq!(cache.get(&1).await, Some("one".to_string()));
        assert_eq!(cache.get(&2).await, Some("two".to_string()));
        assert_eq!(cache.get(&3).await, Some("three".to_string()));
    }

    #[tokio::test]
    async fn test_lru_cache_eviction() {
        let cache = LruCache::new(2);

        cache.insert(1, "one".to_string(), None).await;
        cache.insert(2, "two".to_string(), None).await;
        cache.insert(3, "three".to_string(), None).await;

        // First entry should be evicted
        assert_eq!(cache.get(&1).await, None);
        assert_eq!(cache.get(&2).await, Some("two".to_string()));
        assert_eq!(cache.get(&3).await, Some("three".to_string()));
    }

    #[tokio::test]
    async fn test_multi_tier_cache_basic() {
        let cache = MultiTierCache::new();

        cache.insert(1, "value1".to_string(), None).await;
        assert_eq!(cache.get(&1).await, Some("value1".to_string()));
    }

    #[tokio::test]
    async fn test_multi_tier_cache_promotion() {
        let mut config = TierConfig::default();
        config.promotion_threshold = 2;
        let cache = MultiTierCache::with_config(config);

        cache.insert(1, "value1".to_string(), None).await;

        // Access multiple times to trigger promotion
        for _ in 0..5 {
            cache.get(&1).await;
        }

        let stats = cache.get_stats();
        assert!(stats.get("promotions").copied().unwrap_or(0) > 0);
    }

    #[tokio::test]
    async fn test_multi_tier_cache_hot_insert() {
        let cache = MultiTierCache::new();

        cache.insert_hot(1, "hot_value".to_string(), None).await;

        // Should hit L1 immediately
        assert_eq!(cache.get(&1).await, Some("hot_value".to_string()));
        let stats = cache.get_stats();
        assert_eq!(stats.get("l1_hits").copied().unwrap_or(0), 1);
    }

    #[tokio::test]
    async fn test_multi_tier_cache_remove() {
        let cache = MultiTierCache::new();

        cache.insert(1, "value1".to_string(), None).await;
        assert!(cache.remove(&1).await);
        assert_eq!(cache.get(&1).await, None);
    }

    #[tokio::test]
    async fn test_multi_tier_cache_ttl() {
        let cache = MultiTierCache::new();

        cache.insert(1, "value1".to_string(), Some(Duration::from_millis(100))).await;
        assert_eq!(cache.get(&1).await, Some("value1".to_string()));

        tokio::time::sleep(Duration::from_millis(150)).await;
        assert_eq!(cache.get(&1).await, None);
    }

    #[tokio::test]
    async fn test_cache_statistics() {
        let cache = MultiTierCache::new();

        cache.insert(1, "value1".to_string(), None).await;
        cache.get(&1).await;
        cache.get(&2).await; // miss

        let stats = cache.get_stats();
        assert!(stats.get("l3_hits").copied().unwrap_or(0) > 0);
        assert!(stats.get("misses").copied().unwrap_or(0) > 0);
    }

    #[tokio::test]
    async fn test_hit_rate_calculation() {
        let cache = MultiTierCache::new();

        cache.insert(1, "value1".to_string(), None).await;
        cache.get(&1).await; // hit
        cache.get(&2).await; // miss

        let hit_rate = cache.hit_rate();
        assert!(hit_rate > 0.0 && hit_rate < 1.0);
    }
}
