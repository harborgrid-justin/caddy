//! Enterprise Database Integration Module
//!
//! This module provides a comprehensive database integration system for CADDY,
//! supporting multiple database backends with connection pooling, schema management,
//! migrations, caching, and the repository pattern.
//!
//! # Features
//!
//! - **Connection Management**: Robust connection pooling with health checks
//! - **Multi-Database Support**: PostgreSQL, MySQL, and SQLite
//! - **Schema Management**: Versioned schema with migration support
//! - **Repository Pattern**: Type-safe data access layer
//! - **Query Builder**: Fluent, type-safe query construction
//! - **Transaction Support**: ACID transactions with savepoint support
//! - **Caching**: Multi-level caching with Redis integration
//! - **Migration System**: Version-controlled database migrations
//!
//! # Example
//!
//! ```no_run
//! use caddy::enterprise::database::{ConnectionPool, DatabaseConfig, DatabaseType};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = DatabaseConfig {
//!     db_type: DatabaseType::PostgreSQL,
//!     host: "localhost".to_string(),
//!     port: 5432,
//!     database: "caddy".to_string(),
//!     username: "user".to_string(),
//!     password: "password".to_string(),
//!     max_connections: 10,
//!     min_connections: 2,
//!     connection_timeout: std::time::Duration::from_secs(30),
//! };
//!
//! let pool = ConnectionPool::new(config).await?;
//! # Ok(())
//! # }
//! ```

pub mod cache;
pub mod connection;
pub mod migration;
pub mod query;
pub mod repository;
pub mod schema;
pub mod transaction;

// Re-export commonly used types
pub use cache::{Cache, CacheConfig, CacheStrategy, RedisCache};
pub use connection::{
    Connection, ConnectionConfig, ConnectionPool, DatabaseConfig, DatabaseType, PoolMetrics,
};
pub use migration::{Migration, MigrationError, MigrationManager, MigrationRecord};
pub use query::{OrderDirection, Query, QueryBuilder, QueryError, WhereClause};
pub use repository::{
    DocumentRepository, ProjectRepository, Repository, RepositoryError, UserRepository,
};
pub use schema::{Schema, SchemaError, SchemaManager, SchemaVersion};
pub use transaction::{
    IsolationLevel, SavepointGuard, Transaction, TransactionError, TransactionManager,
};

use thiserror::Error;

/// Database module error types
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Connection error: {0}")]
    Connection(#[from] connection::ConnectionError),

    #[error("Query error: {0}")]
    Query(#[from] query::QueryError),

    #[error("Repository error: {0}")]
    Repository(#[from] repository::RepositoryError),

    #[error("Schema error: {0}")]
    Schema(#[from] schema::SchemaError),

    #[error("Transaction error: {0}")]
    Transaction(#[from] transaction::TransactionError),

    #[error("Migration error: {0}")]
    Migration(#[from] migration::MigrationError),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Result type for database operations
pub type Result<T> = std::result::Result<T, DatabaseError>;
