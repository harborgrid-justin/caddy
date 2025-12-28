//! Database Connection Management
//!
//! Provides connection pooling, health checking, and automatic reconnection
//! for multiple database backends.

use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use thiserror::Error;
use tokio::sync::Semaphore;

/// Database connection errors
#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("Failed to establish connection: {0}")]
    ConnectionFailed(String),

    #[error("Connection pool exhausted")]
    PoolExhausted,

    #[error("Connection timeout after {0:?}")]
    Timeout(Duration),

    #[error("Connection unhealthy: {0}")]
    Unhealthy(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Database type not supported: {0}")]
    UnsupportedDatabase(String),

    #[error("Connection closed")]
    ConnectionClosed,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Supported database types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    /// PostgreSQL database
    PostgreSQL,
    /// MySQL/MariaDB database
    MySQL,
    /// SQLite database
    SQLite,
}

impl DatabaseType {
    /// Get the default port for this database type
    pub fn default_port(&self) -> u16 {
        match self {
            DatabaseType::PostgreSQL => 5432,
            DatabaseType::MySQL => 3306,
            DatabaseType::SQLite => 0, // SQLite doesn't use network ports
        }
    }

    /// Get the connection string prefix
    pub fn connection_prefix(&self) -> &'static str {
        match self {
            DatabaseType::PostgreSQL => "postgresql://",
            DatabaseType::MySQL => "mysql://",
            DatabaseType::SQLite => "sqlite://",
        }
    }
}

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Database type
    pub db_type: DatabaseType,
    /// Host address
    pub host: String,
    /// Port number
    pub port: u16,
    /// Database name
    pub database: String,
    /// Username
    pub username: String,
    /// Password
    pub password: String,
    /// Maximum number of connections
    pub max_connections: u32,
    /// Minimum number of connections to maintain
    pub min_connections: u32,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Idle timeout before connection is closed
    pub idle_timeout: Duration,
    /// Maximum lifetime of a connection
    pub max_lifetime: Duration,
    /// Health check interval
    pub health_check_interval: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            db_type: DatabaseType::PostgreSQL,
            host: "localhost".to_string(),
            port: 5432,
            database: "caddy".to_string(),
            username: "caddy".to_string(),
            password: String::new(),
            max_connections: 10,
            min_connections: 2,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(3600),
            health_check_interval: Duration::from_secs(60),
        }
    }
}

impl DatabaseConfig {
    /// Build connection string
    pub fn connection_string(&self) -> String {
        match self.db_type {
            DatabaseType::SQLite => {
                format!("sqlite://{}", self.database)
            }
            _ => {
                format!(
                    "{}{}:{}@{}:{}/{}",
                    self.db_type.connection_prefix(),
                    self.username,
                    self.password,
                    self.host,
                    self.port,
                    self.database
                )
            }
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), ConnectionError> {
        if self.max_connections == 0 {
            return Err(ConnectionError::InvalidConfig(
                "max_connections must be greater than 0".to_string(),
            ));
        }

        if self.min_connections > self.max_connections {
            return Err(ConnectionError::InvalidConfig(
                "min_connections cannot exceed max_connections".to_string(),
            ));
        }

        if self.db_type != DatabaseType::SQLite {
            if self.host.is_empty() {
                return Err(ConnectionError::InvalidConfig(
                    "host cannot be empty".to_string(),
                ));
            }

            if self.database.is_empty() {
                return Err(ConnectionError::InvalidConfig(
                    "database name cannot be empty".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// Connection configuration for individual connections
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// Connection string
    pub connection_string: String,
    /// Statement timeout
    pub statement_timeout: Duration,
    /// Enable prepared statement caching
    pub enable_prepared_cache: bool,
    /// Maximum prepared statements to cache
    pub max_prepared_statements: usize,
}

/// Represents a database connection
pub struct Connection {
    id: uuid::Uuid,
    config: ConnectionConfig,
    created_at: Instant,
    last_used: Arc<RwLock<Instant>>,
    is_healthy: Arc<RwLock<bool>>,
    db_type: DatabaseType,
}

impl Connection {
    /// Create a new connection
    pub async fn new(
        config: ConnectionConfig,
        db_type: DatabaseType,
    ) -> Result<Self, ConnectionError> {
        // Simulate connection establishment
        // In production, this would use actual database drivers
        tokio::time::sleep(Duration::from_millis(10)).await;

        Ok(Self {
            id: uuid::Uuid::new_v4(),
            config,
            created_at: Instant::now(),
            last_used: Arc::new(RwLock::new(Instant::now())),
            is_healthy: Arc::new(RwLock::new(true)),
            db_type,
        })
    }

    /// Get connection ID
    pub fn id(&self) -> uuid::Uuid {
        self.id
    }

    /// Check if connection is healthy
    pub async fn health_check(&self) -> Result<bool, ConnectionError> {
        // Simulate health check query
        tokio::time::sleep(Duration::from_millis(5)).await;

        let age = self.created_at.elapsed();
        let idle_time = self.last_used.read().elapsed();

        // Mark unhealthy if too old or idle too long
        let healthy = age < Duration::from_secs(3600) && idle_time < Duration::from_secs(600);

        *self.is_healthy.write() = healthy;
        Ok(healthy)
    }

    /// Execute a raw SQL query
    pub async fn execute(&self, sql: &str) -> Result<u64, ConnectionError> {
        self.update_last_used();

        // Simulate query execution
        log::debug!("Executing SQL on connection {}: {}", self.id, sql);
        tokio::time::sleep(Duration::from_millis(5)).await;

        Ok(1)
    }

    /// Execute a query and return results
    pub async fn query(&self, sql: &str) -> Result<Vec<serde_json::Value>, ConnectionError> {
        self.update_last_used();

        // Simulate query execution
        log::debug!("Querying on connection {}: {}", self.id, sql);
        tokio::time::sleep(Duration::from_millis(5)).await;

        Ok(vec![])
    }

    /// Begin a transaction
    pub async fn begin_transaction(&self) -> Result<(), ConnectionError> {
        self.execute("BEGIN").await?;
        Ok(())
    }

    /// Commit a transaction
    pub async fn commit(&self) -> Result<(), ConnectionError> {
        self.execute("COMMIT").await?;
        Ok(())
    }

    /// Rollback a transaction
    pub async fn rollback(&self) -> Result<(), ConnectionError> {
        self.execute("ROLLBACK").await?;
        Ok(())
    }

    /// Update last used timestamp
    fn update_last_used(&self) {
        *self.last_used.write() = Instant::now();
    }

    /// Get database type
    pub fn db_type(&self) -> DatabaseType {
        self.db_type
    }
}

/// Connection pool metrics
#[derive(Debug, Clone, Default)]
pub struct PoolMetrics {
    /// Total connections created
    pub total_created: u64,
    /// Total connections closed
    pub total_closed: u64,
    /// Active connections
    pub active: u32,
    /// Idle connections
    pub idle: u32,
    /// Failed connection attempts
    pub failed_attempts: u64,
    /// Total queries executed
    pub total_queries: u64,
    /// Average query time (microseconds)
    pub avg_query_time_us: u64,
}

/// Connection pool
pub struct ConnectionPool {
    config: DatabaseConfig,
    connections: Arc<RwLock<Vec<Arc<Connection>>>>,
    semaphore: Arc<Semaphore>,
    metrics: Arc<RwLock<PoolMetrics>>,
    health_check_handle: Option<tokio::task::JoinHandle<()>>,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub async fn new(config: DatabaseConfig) -> Result<Self, ConnectionError> {
        config.validate()?;

        let semaphore = Arc::new(Semaphore::new(config.max_connections as usize));
        let connections = Arc::new(RwLock::new(Vec::new()));
        let metrics = Arc::new(RwLock::new(PoolMetrics::default()));

        let mut pool = Self {
            config: config.clone(),
            connections: connections.clone(),
            semaphore,
            metrics: metrics.clone(),
            health_check_handle: None,
        };

        // Create minimum connections
        pool.ensure_min_connections().await?;

        // Start health check task
        let health_check_handle = tokio::spawn(Self::health_check_loop(
            connections,
            config.health_check_interval,
            metrics,
        ));

        pool.health_check_handle = Some(health_check_handle);

        Ok(pool)
    }

    /// Ensure minimum connections are maintained
    async fn ensure_min_connections(&self) -> Result<(), ConnectionError> {
        let current_count = self.connections.read().len();
        let needed = self.config.min_connections.saturating_sub(current_count as u32);

        for _ in 0..needed {
            match self.create_connection().await {
                Ok(conn) => {
                    self.connections.write().push(Arc::new(conn));
                    self.metrics.write().total_created += 1;
                }
                Err(e) => {
                    log::warn!("Failed to create minimum connection: {}", e);
                    self.metrics.write().failed_attempts += 1;
                }
            }
        }

        Ok(())
    }

    /// Create a new connection
    async fn create_connection(&self) -> Result<Connection, ConnectionError> {
        let conn_config = ConnectionConfig {
            connection_string: self.config.connection_string(),
            statement_timeout: Duration::from_secs(30),
            enable_prepared_cache: true,
            max_prepared_statements: 100,
        };

        Connection::new(conn_config, self.config.db_type).await
    }

    /// Acquire a connection from the pool
    pub async fn acquire(&self) -> Result<PooledConnection, ConnectionError> {
        let permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| ConnectionError::PoolExhausted)?;

        // Try to get an existing healthy connection
        let conn = {
            let mut conns = self.connections.write();
            conns.iter().find(|c| *c.is_healthy.read()).cloned()
        };

        let connection = match conn {
            Some(c) => c,
            None => {
                // Create new connection
                let new_conn = self.create_connection().await?;
                let arc_conn = Arc::new(new_conn);
                self.connections.write().push(arc_conn.clone());
                self.metrics.write().total_created += 1;
                arc_conn
            }
        };

        self.metrics.write().active += 1;

        Ok(PooledConnection {
            connection,
            _permit: permit,
            pool: self.metrics.clone(),
        })
    }

    /// Health check loop
    async fn health_check_loop(
        connections: Arc<RwLock<Vec<Arc<Connection>>>>,
        interval: Duration,
        metrics: Arc<RwLock<PoolMetrics>>,
    ) {
        let mut check_interval = tokio::time::interval(interval);

        loop {
            check_interval.tick().await;

            let conns = connections.read().clone();
            for conn in conns.iter() {
                if let Ok(healthy) = conn.health_check().await {
                    if !healthy {
                        log::warn!("Connection {} is unhealthy", conn.id());
                    }
                }
            }

            // Update metrics
            let mut m = metrics.write();
            m.active = conns.iter().filter(|c| *c.is_healthy.read()).count() as u32;
            m.idle = conns.len() as u32 - m.active;
        }
    }

    /// Get pool metrics
    pub fn metrics(&self) -> PoolMetrics {
        self.metrics.read().clone()
    }

    /// Close all connections
    pub async fn close(&mut self) -> Result<(), ConnectionError> {
        if let Some(handle) = self.health_check_handle.take() {
            handle.abort();
        }

        let count = self.connections.read().len();
        self.connections.write().clear();
        self.metrics.write().total_closed += count as u64;

        Ok(())
    }
}

impl Drop for ConnectionPool {
    fn drop(&mut self) {
        if let Some(handle) = self.health_check_handle.take() {
            handle.abort();
        }
    }
}

/// RAII guard for a pooled connection
pub struct PooledConnection {
    connection: Arc<Connection>,
    _permit: tokio::sync::OwnedSemaphorePermit,
    pool: Arc<RwLock<PoolMetrics>>,
}

impl PooledConnection {
    /// Get reference to the underlying connection
    pub fn as_ref(&self) -> &Connection {
        &self.connection
    }

    /// Execute a query
    pub async fn execute(&self, sql: &str) -> Result<u64, ConnectionError> {
        let start = Instant::now();
        let result = self.connection.execute(sql).await;
        let elapsed = start.elapsed().as_micros() as u64;

        let mut metrics = self.pool.write();
        metrics.total_queries += 1;
        metrics.avg_query_time_us =
            (metrics.avg_query_time_us + elapsed) / 2;

        result
    }

    /// Query and return results
    pub async fn query(&self, sql: &str) -> Result<Vec<serde_json::Value>, ConnectionError> {
        let start = Instant::now();
        let result = self.connection.query(sql).await;
        let elapsed = start.elapsed().as_micros() as u64;

        let mut metrics = self.pool.write();
        metrics.total_queries += 1;
        metrics.avg_query_time_us =
            (metrics.avg_query_time_us + elapsed) / 2;

        result
    }
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        self.pool.write().active = self.pool.read().active.saturating_sub(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config_validation() {
        let mut config = DatabaseConfig::default();
        assert!(config.validate().is_ok());

        config.max_connections = 0;
        assert!(config.validate().is_err());

        config.max_connections = 10;
        config.min_connections = 20;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_connection_string() {
        let config = DatabaseConfig {
            db_type: DatabaseType::PostgreSQL,
            host: "localhost".to_string(),
            port: 5432,
            database: "test".to_string(),
            username: "user".to_string(),
            password: "pass".to_string(),
            ..Default::default()
        };

        let conn_str = config.connection_string();
        assert!(conn_str.contains("postgresql://"));
        assert!(conn_str.contains("localhost:5432"));
    }

    #[tokio::test]
    async fn test_connection_pool_creation() {
        let config = DatabaseConfig::default();
        let pool = ConnectionPool::new(config).await;
        assert!(pool.is_ok());
    }
}
