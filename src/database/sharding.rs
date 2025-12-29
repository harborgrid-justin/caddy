//! # Database Sharding
//!
//! Provides horizontal sharding for scalability across multiple database instances.
//! Supports hash-based, range-based, and consistent hashing strategies.

use crate::database::{connection_pool::ConnectionPool, DatabaseError, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Shard key for routing queries
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShardKey {
    /// Integer-based key
    Int(i64),

    /// String-based key
    String(String),

    /// UUID-based key
    Uuid(String),

    /// Composite key
    Composite(Vec<String>),
}

impl ShardKey {
    /// Get the hash value of this key
    pub fn hash(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        match self {
            ShardKey::Int(v) => v.hash(&mut hasher),
            ShardKey::String(v) => v.hash(&mut hasher),
            ShardKey::Uuid(v) => v.hash(&mut hasher),
            ShardKey::Composite(parts) => {
                for part in parts {
                    part.hash(&mut hasher);
                }
            }
        }
        hasher.finish()
    }

    /// Get a numeric representation for range-based sharding
    pub fn as_number(&self) -> Option<i64> {
        match self {
            ShardKey::Int(v) => Some(*v),
            _ => None,
        }
    }
}

/// Sharding strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShardingStrategy {
    /// Hash-based sharding (default)
    Hash,

    /// Range-based sharding
    Range,

    /// Consistent hashing
    ConsistentHash,

    /// Directory-based (lookup table)
    Directory,
}

/// Shard configuration
#[derive(Debug, Clone)]
pub struct ShardConfig {
    /// Sharding strategy
    pub strategy: ShardingStrategy,

    /// Shard connection URLs
    pub shard_urls: Vec<String>,

    /// Virtual nodes for consistent hashing
    pub virtual_nodes: usize,

    /// Enable cross-shard transactions
    pub enable_cross_shard_tx: bool,

    /// Directory service URL (for directory-based sharding)
    pub directory_url: Option<String>,
}

impl Default for ShardConfig {
    fn default() -> Self {
        Self {
            strategy: ShardingStrategy::Hash,
            shard_urls: Vec::new(),
            virtual_nodes: 150,
            enable_cross_shard_tx: false,
            directory_url: None,
        }
    }
}

/// Shard information
#[derive(Debug, Clone)]
pub struct ShardInfo {
    /// Shard ID
    pub id: usize,

    /// Shard URL
    pub url: String,

    /// Connection pool
    pub pool: Arc<ConnectionPool>,

    /// Range start (for range-based sharding)
    pub range_start: Option<i64>,

    /// Range end (for range-based sharding)
    pub range_end: Option<i64>,

    /// Whether this shard is available
    pub is_available: bool,

    /// Number of keys in this shard
    pub key_count: u64,
}

/// Shard manager
pub struct ShardManager {
    /// Configuration
    config: ShardConfig,

    /// Shard information
    shards: Arc<RwLock<Vec<ShardInfo>>>,

    /// Consistent hash ring (for consistent hashing)
    hash_ring: Arc<RwLock<HashMap<u64, usize>>>,

    /// Directory (for directory-based sharding)
    directory: Arc<RwLock<HashMap<String, usize>>>,

    /// Statistics
    stats: Arc<RwLock<ShardingStats>>,
}

impl ShardManager {
    /// Create a new shard manager
    pub async fn new(config: ShardConfig) -> Result<Self> {
        let mut shards = Vec::new();

        for (id, url) in config.shard_urls.iter().enumerate() {
            let pool_config = crate::database::connection_pool::DatabaseConfig {
                url: url.clone(),
                ..Default::default()
            };

            let pool = Arc::new(ConnectionPool::new(pool_config).await?);

            shards.push(ShardInfo {
                id,
                url: url.clone(),
                pool,
                range_start: None,
                range_end: None,
                is_available: true,
                key_count: 0,
            });
        }

        let mut manager = Self {
            config: config.clone(),
            shards: Arc::new(RwLock::new(shards)),
            hash_ring: Arc::new(RwLock::new(HashMap::new())),
            directory: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(ShardingStats::default())),
        };

        // Initialize hash ring for consistent hashing
        if config.strategy == ShardingStrategy::ConsistentHash {
            manager.build_hash_ring();
        }

        Ok(manager)
    }

    /// Get the shard for a given key
    pub fn get_shard(&self, key: &ShardKey) -> Result<Arc<ConnectionPool>> {
        let shard_id = self.route_key(key)?;

        let shards = self.shards.read();
        let shard = shards
            .get(shard_id)
            .ok_or_else(|| DatabaseError::Sharding(format!("Shard {} not found", shard_id)))?;

        if !shard.is_available {
            return Err(DatabaseError::Sharding(format!(
                "Shard {} is not available",
                shard_id
            )));
        }

        self.stats.write().total_lookups += 1;

        Ok(shard.pool.clone())
    }

    /// Route a key to a shard ID
    fn route_key(&self, key: &ShardKey) -> Result<usize> {
        match self.config.strategy {
            ShardingStrategy::Hash => self.route_hash(key),
            ShardingStrategy::Range => self.route_range(key),
            ShardingStrategy::ConsistentHash => self.route_consistent_hash(key),
            ShardingStrategy::Directory => self.route_directory(key),
        }
    }

    /// Route using hash-based sharding
    fn route_hash(&self, key: &ShardKey) -> Result<usize> {
        let shard_count = self.shards.read().len();
        if shard_count == 0 {
            return Err(DatabaseError::Sharding("No shards available".to_string()));
        }

        let hash = key.hash();
        Ok((hash % shard_count as u64) as usize)
    }

    /// Route using range-based sharding
    fn route_range(&self, key: &ShardKey) -> Result<usize> {
        let value = key
            .as_number()
            .ok_or_else(|| DatabaseError::Sharding("Key must be numeric for range sharding".to_string()))?;

        let shards = self.shards.read();

        for shard in shards.iter() {
            if let (Some(start), Some(end)) = (shard.range_start, shard.range_end) {
                if value >= start && value < end {
                    return Ok(shard.id);
                }
            }
        }

        Err(DatabaseError::Sharding(format!(
            "No shard found for value {}",
            value
        )))
    }

    /// Route using consistent hashing
    fn route_consistent_hash(&self, key: &ShardKey) -> Result<usize> {
        let hash = key.hash();
        let ring = self.hash_ring.read();

        if ring.is_empty() {
            return Err(DatabaseError::Sharding("Hash ring is empty".to_string()));
        }

        // Find the first node >= hash
        let shard_id = ring
            .iter()
            .filter(|(k, _)| **k >= hash)
            .min_by_key(|(k, _)| *k)
            .map(|(_, v)| *v)
            .or_else(|| {
                // Wrap around to the first node
                ring.iter().min_by_key(|(k, _)| *k).map(|(_, v)| *v)
            })
            .ok_or_else(|| DatabaseError::Sharding("No node found in hash ring".to_string()))?;

        Ok(shard_id)
    }

    /// Route using directory-based sharding
    fn route_directory(&self, key: &ShardKey) -> Result<usize> {
        let key_str = match key {
            ShardKey::String(s) => s.clone(),
            ShardKey::Uuid(s) => s.clone(),
            _ => return Err(DatabaseError::Sharding("Directory sharding requires string keys".to_string())),
        };

        let directory = self.directory.read();
        directory
            .get(&key_str)
            .copied()
            .ok_or_else(|| DatabaseError::Sharding(format!("Key {} not found in directory", key_str)))
    }

    /// Build consistent hash ring
    fn build_hash_ring(&mut self) {
        let mut ring = self.hash_ring.write();
        ring.clear();

        let shards = self.shards.read();
        let virtual_nodes = self.config.virtual_nodes;

        for shard in shards.iter() {
            for i in 0..virtual_nodes {
                let virtual_key = format!("{}:{}", shard.id, i);
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                virtual_key.hash(&mut hasher);
                let hash = hasher.finish();

                ring.insert(hash, shard.id);
            }
        }

        log::info!(
            "Built consistent hash ring with {} virtual nodes across {} shards",
            ring.len(),
            shards.len()
        );
    }

    /// Add a key to directory
    pub fn add_to_directory(&self, key: String, shard_id: usize) -> Result<()> {
        if self.config.strategy != ShardingStrategy::Directory {
            return Err(DatabaseError::Sharding(
                "Directory operations only available for directory-based sharding".to_string(),
            ));
        }

        self.directory.write().insert(key, shard_id);
        Ok(())
    }

    /// Get all shards for cross-shard operations
    pub fn get_all_shards(&self) -> Vec<Arc<ConnectionPool>> {
        self.shards
            .read()
            .iter()
            .filter(|s| s.is_available)
            .map(|s| s.pool.clone())
            .collect()
    }

    /// Execute a query on a specific shard
    pub async fn execute_on_shard(
        &self,
        shard_id: usize,
        query: &str,
    ) -> Result<sqlx::sqlite::SqliteQueryResult> {
        let shards = self.shards.read();
        let shard = shards
            .get(shard_id)
            .ok_or_else(|| DatabaseError::Sharding(format!("Shard {} not found", shard_id)))?;

        shard.pool.execute(sqlx::query(query)).await
    }

    /// Execute a query on all shards (scatter-gather)
    pub async fn execute_on_all_shards(&self, query: &str) -> Result<Vec<sqlx::sqlite::SqliteQueryResult>> {
        let shards = self.get_all_shards();
        let mut results = Vec::new();

        for shard in shards {
            let result = shard.execute(sqlx::query(query)).await?;
            results.push(result);
        }

        self.stats.write().cross_shard_queries += 1;

        Ok(results)
    }

    /// Get shard statistics
    pub fn stats(&self) -> ShardingStats {
        let mut stats = self.stats.read().clone();
        stats.total_shards = self.shards.read().len();
        stats.available_shards = self.shards.read().iter().filter(|s| s.is_available).count();
        stats
    }

    /// Rebalance shards (for range-based sharding)
    pub async fn rebalance(&self) -> Result<()> {
        log::info!("Starting shard rebalancing");

        // This would involve:
        // 1. Analyzing current distribution
        // 2. Computing new ranges
        // 3. Moving data between shards
        // 4. Updating range information

        Ok(())
    }

    /// Mark a shard as unavailable
    pub fn mark_shard_unavailable(&self, shard_id: usize) {
        if let Some(shard) = self.shards.write().get_mut(shard_id) {
            shard.is_available = false;
            log::warn!("Shard {} marked as unavailable", shard_id);
        }
    }

    /// Mark a shard as available
    pub fn mark_shard_available(&self, shard_id: usize) {
        if let Some(shard) = self.shards.write().get_mut(shard_id) {
            shard.is_available = true;
            log::info!("Shard {} marked as available", shard_id);
        }
    }
}

/// Sharding statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShardingStats {
    /// Total number of shards
    pub total_shards: usize,

    /// Number of available shards
    pub available_shards: usize,

    /// Total shard lookups
    pub total_lookups: u64,

    /// Cross-shard queries
    pub cross_shard_queries: u64,

    /// Rebalancing operations
    pub rebalance_count: u64,
}

/// Shard range builder for range-based sharding
pub struct ShardRangeBuilder {
    ranges: Vec<(i64, i64)>,
}

impl ShardRangeBuilder {
    /// Create a new range builder
    pub fn new() -> Self {
        Self { ranges: Vec::new() }
    }

    /// Add a range
    pub fn add_range(mut self, start: i64, end: i64) -> Self {
        self.ranges.push((start, end));
        self
    }

    /// Build ranges evenly across a value range
    pub fn build_even(shard_count: usize, min: i64, max: i64) -> Self {
        let mut builder = Self::new();
        let range_size = (max - min) / shard_count as i64;

        for i in 0..shard_count {
            let start = min + (i as i64 * range_size);
            let end = if i == shard_count - 1 {
                max
            } else {
                start + range_size
            };
            builder = builder.add_range(start, end);
        }

        builder
    }

    /// Apply ranges to shards
    pub fn apply(self, manager: &ShardManager) -> Result<()> {
        let mut shards = manager.shards.write();

        if shards.len() != self.ranges.len() {
            return Err(DatabaseError::Sharding(format!(
                "Range count ({}) does not match shard count ({})",
                self.ranges.len(),
                shards.len()
            )));
        }

        for (i, (start, end)) in self.ranges.into_iter().enumerate() {
            shards[i].range_start = Some(start);
            shards[i].range_end = Some(end);
        }

        Ok(())
    }
}

impl Default for ShardRangeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shard_key_hash() {
        let key1 = ShardKey::String("test".to_string());
        let key2 = ShardKey::String("test".to_string());
        let key3 = ShardKey::String("other".to_string());

        assert_eq!(key1.hash(), key2.hash());
        assert_ne!(key1.hash(), key3.hash());
    }

    #[test]
    fn test_range_builder() {
        let builder = ShardRangeBuilder::build_even(4, 0, 1000);
        assert_eq!(builder.ranges.len(), 4);
        assert_eq!(builder.ranges[0], (0, 250));
        assert_eq!(builder.ranges[3], (750, 1000));
    }

    #[tokio::test]
    async fn test_shard_manager_creation() {
        let config = ShardConfig {
            shard_urls: vec!["sqlite::memory:".to_string()],
            ..Default::default()
        };

        let manager = ShardManager::new(config).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_hash_routing() {
        let config = ShardConfig {
            strategy: ShardingStrategy::Hash,
            shard_urls: vec![
                "sqlite::memory:".to_string(),
                "sqlite::memory:".to_string(),
            ],
            ..Default::default()
        };

        let manager = ShardManager::new(config).await.unwrap();

        let key = ShardKey::String("test".to_string());
        let shard = manager.get_shard(&key);
        assert!(shard.is_ok());
    }
}
