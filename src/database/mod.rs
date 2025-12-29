//! # Database Layer for CADDY Enterprise
//!
//! This module provides a comprehensive database layer with:
//! - Async connection pooling with health checks
//! - Query optimization for CAD data patterns
//! - Spatial indexing (R-tree and octree)
//! - Multi-tier caching (L1 memory, L2 disk, L3 distributed)
//! - Schema migration system
//! - Master-slave replication support
//! - Horizontal sharding for large datasets
//! - Incremental backup and point-in-time recovery
//!
//! ## Architecture
//!
//! The database layer is designed for high-performance CAD workloads with:
//! - ACID guarantees for all transactions
//! - Optimized spatial queries for geometric data
//! - Intelligent caching to minimize disk I/O
//! - Horizontal scalability through sharding
//! - High availability through replication
//!
//! ## Example Usage
//!
//! ```rust
//! use caddy::database::{DatabaseConfig, ConnectionPool};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = DatabaseConfig::default();
//!     let pool = ConnectionPool::new(config).await?;
//!
//!     // Execute a query
//!     let result = pool.query("SELECT * FROM entities WHERE layer_id = ?")
//!         .bind(123)
//!         .fetch_all()
//!         .await?;
//!
//!     Ok(())
//! }
//! ```

use thiserror::Error;

/// Database error types
#[derive(Error, Debug)]
pub enum DatabaseError {
    /// Connection pool error
    #[error("Connection pool error: {0}")]
    ConnectionPool(String),

    /// Query execution error
    #[error("Query execution error: {0}")]
    QueryExecution(String),

    /// Migration error
    #[error("Migration error: {0}")]
    Migration(String),

    /// Replication error
    #[error("Replication error: {0}")]
    Replication(String),

    /// Sharding error
    #[error("Sharding error: {0}")]
    Sharding(String),

    /// Backup error
    #[error("Backup error: {0}")]
    Backup(String),

    /// Cache error
    #[error("Cache error: {0}")]
    Cache(String),

    /// Spatial index error
    #[error("Spatial index error: {0}")]
    SpatialIndex(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// SQLx error
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

/// Result type for database operations
pub type Result<T> = std::result::Result<T, DatabaseError>;

// Module declarations
pub mod connection_pool;
pub mod query_optimizer;
pub mod spatial_index;
pub mod cache;
pub mod migrations;
pub mod replication;
pub mod sharding;
pub mod backup;

// Re-exports for convenience
pub use connection_pool::{ConnectionPool, DatabaseConfig, HealthCheck};
pub use query_optimizer::{QueryOptimizer, QueryPlan, OptimizationHint};
pub use spatial_index::{SpatialIndex, RTreeIndex, OctreeIndex, BoundingVolume};
pub use cache::{CacheManager, CacheConfig, CacheLayer, CacheStats};
pub use migrations::{MigrationManager, Migration, MigrationVersion};
pub use replication::{ReplicationManager, ReplicationConfig, ReplicaRole};
pub use sharding::{ShardManager, ShardConfig, ShardKey};
pub use backup::{BackupManager, BackupConfig, BackupType, RestorePoint};

/// Database configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Primary database URL
    pub primary_url: String,

    /// Read replica URLs
    pub replica_urls: Vec<String>,

    /// Connection pool configuration
    pub pool_config: connection_pool::DatabaseConfig,

    /// Cache configuration
    pub cache_config: cache::CacheConfig,

    /// Replication configuration
    pub replication_config: Option<replication::ReplicationConfig>,

    /// Sharding configuration
    pub sharding_config: Option<sharding::ShardConfig>,

    /// Backup configuration
    pub backup_config: backup::BackupConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            primary_url: "sqlite://caddy.db".to_string(),
            replica_urls: Vec::new(),
            pool_config: connection_pool::DatabaseConfig::default(),
            cache_config: cache::CacheConfig::default(),
            replication_config: None,
            sharding_config: None,
            backup_config: backup::BackupConfig::default(),
        }
    }
}

/// Main database interface
pub struct Database {
    /// Connection pool
    pool: ConnectionPool,

    /// Query optimizer
    optimizer: QueryOptimizer,

    /// Spatial index
    spatial_index: SpatialIndex,

    /// Cache manager
    cache: CacheManager,

    /// Migration manager
    migrations: MigrationManager,

    /// Replication manager (optional)
    replication: Option<ReplicationManager>,

    /// Shard manager (optional)
    sharding: Option<ShardManager>,

    /// Backup manager
    backup: BackupManager,
}

impl Database {
    /// Create a new database instance
    pub async fn new(config: Config) -> Result<Self> {
        let pool = ConnectionPool::new(config.pool_config.clone()).await?;
        let optimizer = QueryOptimizer::new();
        let spatial_index = SpatialIndex::new();
        let cache = CacheManager::new(config.cache_config.clone()).await?;
        let migrations = MigrationManager::new(pool.clone());

        let replication = if let Some(repl_config) = config.replication_config {
            Some(ReplicationManager::new(repl_config).await?)
        } else {
            None
        };

        let sharding = if let Some(shard_config) = config.sharding_config {
            Some(ShardManager::new(shard_config).await?)
        } else {
            None
        };

        let backup = BackupManager::new(config.backup_config)?;

        Ok(Self {
            pool,
            optimizer,
            spatial_index,
            cache,
            migrations,
            replication,
            sharding,
            backup,
        })
    }

    /// Get the connection pool
    pub fn pool(&self) -> &ConnectionPool {
        &self.pool
    }

    /// Get the query optimizer
    pub fn optimizer(&self) -> &QueryOptimizer {
        &self.optimizer
    }

    /// Get the spatial index
    pub fn spatial_index(&self) -> &SpatialIndex {
        &self.spatial_index
    }

    /// Get the cache manager
    pub fn cache(&self) -> &CacheManager {
        &self.cache
    }

    /// Get the migration manager
    pub fn migrations(&self) -> &MigrationManager {
        &self.migrations
    }

    /// Get the replication manager
    pub fn replication(&self) -> Option<&ReplicationManager> {
        self.replication.as_ref()
    }

    /// Get the shard manager
    pub fn sharding(&self) -> Option<&ShardManager> {
        self.sharding.as_ref()
    }

    /// Get the backup manager
    pub fn backup(&self) -> &BackupManager {
        &self.backup
    }

    /// Run all pending migrations
    pub async fn migrate(&self) -> Result<()> {
        self.migrations.run_pending().await
    }

    /// Create a backup
    pub async fn create_backup(&self) -> Result<String> {
        self.backup.create_backup(&self.pool).await
    }

    /// Restore from a backup
    pub async fn restore_backup(&self, backup_id: &str) -> Result<()> {
        self.backup.restore_backup(&self.pool, backup_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_creation() {
        let config = Config::default();
        let db = Database::new(config).await;
        assert!(db.is_ok());
    }
}
