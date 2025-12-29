//! # Connection Pool Management
//!
//! Provides async connection pooling with automatic health checks,
//! connection lifecycle management, and performance monitoring.

use crate::database::{DatabaseError, Result};
use async_trait::async_trait;
use parking_lot::RwLock;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::time::interval;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Database URL (e.g., "sqlite://caddy.db" or "postgres://...")
    pub url: String,

    /// Minimum number of connections in the pool
    pub min_connections: u32,

    /// Maximum number of connections in the pool
    pub max_connections: u32,

    /// Connection timeout in seconds
    pub connect_timeout: u64,

    /// Idle timeout in seconds (how long a connection can be idle before being closed)
    pub idle_timeout: u64,

    /// Maximum lifetime of a connection in seconds
    pub max_lifetime: u64,

    /// Health check interval in seconds
    pub health_check_interval: u64,

    /// Enable statement caching
    pub statement_cache_capacity: usize,

    /// Enable WAL mode for SQLite
    pub enable_wal: bool,

    /// Busy timeout for SQLite in milliseconds
    pub busy_timeout: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite://caddy.db".to_string(),
            min_connections: 5,
            max_connections: 100,
            connect_timeout: 30,
            idle_timeout: 600,
            max_lifetime: 3600,
            health_check_interval: 60,
            statement_cache_capacity: 128,
            enable_wal: true,
            busy_timeout: 5000,
        }
    }
}

/// Connection pool statistics
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    /// Total number of connections created
    pub total_connections: u64,

    /// Current number of active connections
    pub active_connections: u32,

    /// Current number of idle connections
    pub idle_connections: u32,

    /// Total queries executed
    pub total_queries: u64,

    /// Total query errors
    pub total_errors: u64,

    /// Average query execution time in microseconds
    pub avg_query_time_us: u64,

    /// Last health check time
    pub last_health_check: Option<Instant>,

    /// Health check status
    pub is_healthy: bool,
}

/// Connection pool with health monitoring
#[derive(Clone)]
pub struct ConnectionPool {
    /// SQLite connection pool
    pool: SqlitePool,

    /// Configuration
    config: DatabaseConfig,

    /// Pool statistics
    stats: Arc<RwLock<PoolStats>>,

    /// Health check semaphore to prevent concurrent health checks
    health_check_semaphore: Arc<Semaphore>,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        let connect_options = SqliteConnectOptions::new()
            .filename(&config.url.replace("sqlite://", ""))
            .create_if_missing(true)
            .statement_cache_capacity(config.statement_cache_capacity)
            .busy_timeout(Duration::from_millis(config.busy_timeout));

        let pool = SqlitePoolOptions::new()
            .min_connections(config.min_connections)
            .max_connections(config.max_connections)
            .acquire_timeout(Duration::from_secs(config.connect_timeout))
            .idle_timeout(Some(Duration::from_secs(config.idle_timeout)))
            .max_lifetime(Some(Duration::from_secs(config.max_lifetime)))
            .connect_with(connect_options)
            .await
            .map_err(|e| DatabaseError::ConnectionPool(e.to_string()))?;

        // Enable WAL mode if configured
        if config.enable_wal {
            sqlx::query("PRAGMA journal_mode=WAL")
                .execute(&pool)
                .await
                .map_err(|e| DatabaseError::ConnectionPool(e.to_string()))?;
        }

        // Set other performance pragmas
        sqlx::query("PRAGMA synchronous=NORMAL")
            .execute(&pool)
            .await
            .map_err(|e| DatabaseError::ConnectionPool(e.to_string()))?;

        sqlx::query("PRAGMA cache_size=-64000") // 64MB cache
            .execute(&pool)
            .await
            .map_err(|e| DatabaseError::ConnectionPool(e.to_string()))?;

        sqlx::query("PRAGMA temp_store=MEMORY")
            .execute(&pool)
            .await
            .map_err(|e| DatabaseError::ConnectionPool(e.to_string()))?;

        let mut stats = PoolStats::default();
        stats.total_connections = config.min_connections as u64;
        stats.is_healthy = true;

        let pool_instance = Self {
            pool,
            config: config.clone(),
            stats: Arc::new(RwLock::new(stats)),
            health_check_semaphore: Arc::new(Semaphore::new(1)),
        };

        // Start background health check task
        pool_instance.start_health_check_task();

        Ok(pool_instance)
    }

    /// Get a reference to the underlying SQLx pool
    pub fn inner(&self) -> &SqlitePool {
        &self.pool
    }

    /// Execute a query and record statistics
    pub async fn execute<'q, Q>(&self, query: Q) -> Result<sqlx::sqlite::SqliteQueryResult>
    where
        Q: sqlx::Execute<'q, sqlx::Sqlite>,
    {
        let start = Instant::now();

        let result = query
            .execute(&self.pool)
            .await
            .map_err(|e| {
                self.record_error();
                DatabaseError::QueryExecution(e.to_string())
            })?;

        self.record_query(start.elapsed());

        Ok(result)
    }

    /// Fetch all rows from a query
    pub async fn fetch_all<'q, Q, O>(&self, query: Q) -> Result<Vec<O>>
    where
        Q: sqlx::Execute<'q, sqlx::Sqlite>,
        O: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    {
        let start = Instant::now();

        let result = query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                self.record_error();
                DatabaseError::QueryExecution(e.to_string())
            })?;

        self.record_query(start.elapsed());

        Ok(result)
    }

    /// Fetch one row from a query
    pub async fn fetch_one<'q, Q, O>(&self, query: Q) -> Result<O>
    where
        Q: sqlx::Execute<'q, sqlx::Sqlite>,
        O: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    {
        let start = Instant::now();

        let result = query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                self.record_error();
                DatabaseError::QueryExecution(e.to_string())
            })?;

        self.record_query(start.elapsed());

        Ok(result)
    }

    /// Begin a transaction
    pub async fn begin(&self) -> Result<sqlx::Transaction<'_, sqlx::Sqlite>> {
        self.pool
            .begin()
            .await
            .map_err(|e| DatabaseError::ConnectionPool(e.to_string()))
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        self.stats.read().clone()
    }

    /// Perform a health check
    pub async fn health_check(&self) -> Result<HealthCheckResult> {
        // Only one health check at a time
        let _permit = self.health_check_semaphore.try_acquire()
            .map_err(|_| DatabaseError::ConnectionPool("Health check already in progress".to_string()))?;

        let start = Instant::now();

        // Test connection with a simple query
        let result = sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await;

        let latency = start.elapsed();
        let is_healthy = result.is_ok();

        // Update stats
        {
            let mut stats = self.stats.write();
            stats.last_health_check = Some(Instant::now());
            stats.is_healthy = is_healthy;
        }

        Ok(HealthCheckResult {
            is_healthy,
            latency,
            pool_size: self.pool.size(),
            idle_connections: self.pool.num_idle(),
            error: result.err().map(|e| e.to_string()),
        })
    }

    /// Record a successful query
    fn record_query(&self, duration: Duration) {
        let mut stats = self.stats.write();
        stats.total_queries += 1;

        // Update moving average
        let new_query_time = duration.as_micros() as u64;
        if stats.total_queries == 1 {
            stats.avg_query_time_us = new_query_time;
        } else {
            // Exponential moving average with alpha = 0.1
            stats.avg_query_time_us = (stats.avg_query_time_us * 9 + new_query_time) / 10;
        }
    }

    /// Record a query error
    fn record_error(&self) {
        let mut stats = self.stats.write();
        stats.total_errors += 1;
    }

    /// Start background health check task
    fn start_health_check_task(&self) {
        let pool = self.clone();
        let check_interval = Duration::from_secs(self.config.health_check_interval);

        tokio::spawn(async move {
            let mut ticker = interval(check_interval);

            loop {
                ticker.tick().await;

                if let Err(e) = pool.health_check().await {
                    log::error!("Health check failed: {}", e);
                }
            }
        });
    }

    /// Close the connection pool
    pub async fn close(&self) {
        self.pool.close().await;
    }

    /// Get the current pool size
    pub fn size(&self) -> u32 {
        self.pool.size()
    }

    /// Get the number of idle connections
    pub fn num_idle(&self) -> usize {
        self.pool.num_idle()
    }
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Whether the pool is healthy
    pub is_healthy: bool,

    /// Response latency
    pub latency: Duration,

    /// Current pool size
    pub pool_size: u32,

    /// Number of idle connections
    pub idle_connections: usize,

    /// Error message if unhealthy
    pub error: Option<String>,
}

/// Health check trait for custom health checks
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Perform a health check
    async fn check(&self) -> Result<bool>;

    /// Get the name of this health check
    fn name(&self) -> &str;
}

/// Database health check
pub struct DatabaseHealthCheck {
    pool: ConnectionPool,
}

impl DatabaseHealthCheck {
    /// Create a new database health check
    pub fn new(pool: ConnectionPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HealthCheck for DatabaseHealthCheck {
    async fn check(&self) -> Result<bool> {
        let result = self.pool.health_check().await?;
        Ok(result.is_healthy)
    }

    fn name(&self) -> &str {
        "database"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_pool_creation() {
        let config = DatabaseConfig {
            url: "sqlite::memory:".to_string(),
            ..Default::default()
        };

        let pool = ConnectionPool::new(config).await;
        assert!(pool.is_ok());
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = DatabaseConfig {
            url: "sqlite::memory:".to_string(),
            ..Default::default()
        };

        let pool = ConnectionPool::new(config).await.unwrap();
        let result = pool.health_check().await.unwrap();

        assert!(result.is_healthy);
        assert!(result.latency.as_millis() < 1000);
    }

    #[tokio::test]
    async fn test_query_execution() {
        let config = DatabaseConfig {
            url: "sqlite::memory:".to_string(),
            ..Default::default()
        };

        let pool = ConnectionPool::new(config).await.unwrap();

        // Create a test table
        let result = pool.execute(sqlx::query("CREATE TABLE test (id INTEGER PRIMARY KEY)")).await;
        assert!(result.is_ok());

        // Verify stats were recorded
        let stats = pool.stats();
        assert_eq!(stats.total_queries, 1);
        assert_eq!(stats.total_errors, 0);
    }
}
