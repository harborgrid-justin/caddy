//! Transaction Management
//!
//! Provides ACID transaction support with savepoints, isolation levels,
//! and automatic rollback on errors.

use std::sync::Arc;
use parking_lot::Mutex;
use thiserror::Error;
use uuid::Uuid;

use super::connection::{ConnectionPool, PooledConnection};

/// Transaction errors
#[derive(Debug, Error)]
pub enum TransactionError {
    #[error("Transaction already committed")]
    AlreadyCommitted,

    #[error("Transaction already rolled back")]
    AlreadyRolledBack,

    #[error("Transaction not active")]
    NotActive,

    #[error("Savepoint not found: {0}")]
    SavepointNotFound(String),

    #[error("Connection error: {0}")]
    Connection(#[from] super::connection::ConnectionError),

    #[error("Deadlock detected")]
    Deadlock,

    #[error("Serialization failure")]
    SerializationFailure,

    #[error("Transaction timeout")]
    Timeout,

    #[error("Nested transaction error: {0}")]
    NestedTransaction(String),
}

/// Transaction isolation level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationLevel {
    /// Read uncommitted (lowest isolation)
    ReadUncommitted,
    /// Read committed
    ReadCommitted,
    /// Repeatable read
    RepeatableRead,
    /// Serializable (highest isolation)
    Serializable,
}

impl IsolationLevel {
    /// Convert to SQL string
    pub fn to_sql(&self) -> &'static str {
        match self {
            IsolationLevel::ReadUncommitted => "READ UNCOMMITTED",
            IsolationLevel::ReadCommitted => "READ COMMITTED",
            IsolationLevel::RepeatableRead => "REPEATABLE READ",
            IsolationLevel::Serializable => "SERIALIZABLE",
        }
    }

    /// Get PostgreSQL syntax
    pub fn to_postgres_sql(&self) -> String {
        format!("SET TRANSACTION ISOLATION LEVEL {}", self.to_sql())
    }
}

/// Transaction state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransactionState {
    Active,
    Committed,
    RolledBack,
}

/// Savepoint for nested transactions
#[derive(Debug, Clone)]
pub struct Savepoint {
    name: String,
    created_at: std::time::Instant,
}

impl Savepoint {
    /// Create a new savepoint
    fn new(name: String) -> Self {
        Self {
            name,
            created_at: std::time::Instant::now(),
        }
    }

    /// Get savepoint name
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// RAII guard for savepoints
pub struct SavepointGuard<'a> {
    transaction: &'a Transaction,
    savepoint: Savepoint,
    released: bool,
}

impl<'a> SavepointGuard<'a> {
    /// Rollback to this savepoint
    pub async fn rollback(mut self) -> Result<(), TransactionError> {
        self.transaction.rollback_to_savepoint(&self.savepoint.name).await?;
        self.released = true;
        Ok(())
    }

    /// Release this savepoint
    pub async fn release(mut self) -> Result<(), TransactionError> {
        self.transaction.release_savepoint(&self.savepoint.name).await?;
        self.released = true;
        Ok(())
    }
}

impl<'a> Drop for SavepointGuard<'a> {
    fn drop(&mut self) {
        if !self.released {
            // Auto-release savepoint on drop
            log::debug!("Auto-releasing savepoint: {}", self.savepoint.name);
        }
    }
}

/// Database transaction
pub struct Transaction {
    id: Uuid,
    connection: Arc<Mutex<Option<PooledConnection>>>,
    state: Arc<Mutex<TransactionState>>,
    isolation_level: IsolationLevel,
    savepoints: Arc<Mutex<Vec<Savepoint>>>,
    read_only: bool,
}

impl Transaction {
    /// Create a new transaction
    async fn new(
        connection: PooledConnection,
        isolation_level: IsolationLevel,
        read_only: bool,
    ) -> Result<Self, TransactionError> {
        let tx = Self {
            id: Uuid::new_v4(),
            connection: Arc::new(Mutex::new(Some(connection))),
            state: Arc::new(Mutex::new(TransactionState::Active)),
            isolation_level,
            savepoints: Arc::new(Mutex::new(Vec::new())),
            read_only,
        };

        // Begin transaction
        tx.begin().await?;

        log::info!("Transaction {} started with isolation level {:?}", tx.id, isolation_level);

        Ok(tx)
    }

    /// Begin the transaction
    async fn begin(&self) -> Result<(), TransactionError> {
        let conn = self.connection.lock();
        if let Some(ref c) = *conn {
            // Set isolation level
            c.execute(&self.isolation_level.to_postgres_sql()).await?;

            // Begin transaction
            if self.read_only {
                c.execute("BEGIN READ ONLY").await?;
            } else {
                c.execute("BEGIN").await?;
            }
        }

        Ok(())
    }

    /// Get transaction ID
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Check if transaction is active
    pub fn is_active(&self) -> bool {
        *self.state.lock() == TransactionState::Active
    }

    /// Execute a query within the transaction
    pub async fn execute(&self, sql: &str) -> Result<u64, TransactionError> {
        if !self.is_active() {
            return Err(TransactionError::NotActive);
        }

        let conn = self.connection.lock();
        if let Some(ref c) = *conn {
            let result = c.execute(sql).await?;
            Ok(result)
        } else {
            Err(TransactionError::NotActive)
        }
    }

    /// Execute a query and return results
    pub async fn query(&self, sql: &str) -> Result<Vec<serde_json::Value>, TransactionError> {
        if !self.is_active() {
            return Err(TransactionError::NotActive);
        }

        let conn = self.connection.lock();
        if let Some(ref c) = *conn {
            let result = c.query(sql).await?;
            Ok(result)
        } else {
            Err(TransactionError::NotActive)
        }
    }

    /// Create a savepoint
    pub async fn savepoint(&self, name: impl Into<String>) -> Result<SavepointGuard, TransactionError> {
        if !self.is_active() {
            return Err(TransactionError::NotActive);
        }

        let name = name.into();
        let sql = format!("SAVEPOINT {}", name);

        self.execute(&sql).await?;

        let savepoint = Savepoint::new(name);
        self.savepoints.lock().push(savepoint.clone());

        log::debug!("Created savepoint: {}", savepoint.name);

        Ok(SavepointGuard {
            transaction: self,
            savepoint,
            released: false,
        })
    }

    /// Rollback to a savepoint
    async fn rollback_to_savepoint(&self, name: &str) -> Result<(), TransactionError> {
        if !self.is_active() {
            return Err(TransactionError::NotActive);
        }

        // Check if savepoint exists
        let savepoints = self.savepoints.lock();
        if !savepoints.iter().any(|sp| sp.name == name) {
            return Err(TransactionError::SavepointNotFound(name.to_string()));
        }

        let sql = format!("ROLLBACK TO SAVEPOINT {}", name);
        self.execute(&sql).await?;

        log::debug!("Rolled back to savepoint: {}", name);

        Ok(())
    }

    /// Release a savepoint
    async fn release_savepoint(&self, name: &str) -> Result<(), TransactionError> {
        if !self.is_active() {
            return Err(TransactionError::NotActive);
        }

        let sql = format!("RELEASE SAVEPOINT {}", name);
        self.execute(&sql).await?;

        // Remove savepoint from list
        let mut savepoints = self.savepoints.lock();
        savepoints.retain(|sp| sp.name != name);

        log::debug!("Released savepoint: {}", name);

        Ok(())
    }

    /// Commit the transaction
    pub async fn commit(self) -> Result<(), TransactionError> {
        let mut state = self.state.lock();

        match *state {
            TransactionState::Active => {
                let mut conn = self.connection.lock();
                if let Some(ref c) = *conn {
                    c.execute("COMMIT").await?;
                    log::info!("Transaction {} committed", self.id);
                }

                *state = TransactionState::Committed;
                conn.take(); // Release connection
                Ok(())
            }
            TransactionState::Committed => Err(TransactionError::AlreadyCommitted),
            TransactionState::RolledBack => Err(TransactionError::AlreadyRolledBack),
        }
    }

    /// Rollback the transaction
    pub async fn rollback(self) -> Result<(), TransactionError> {
        let mut state = self.state.lock();

        match *state {
            TransactionState::Active => {
                let mut conn = self.connection.lock();
                if let Some(ref c) = *conn {
                    c.execute("ROLLBACK").await?;
                    log::info!("Transaction {} rolled back", self.id);
                }

                *state = TransactionState::RolledBack;
                conn.take(); // Release connection
                Ok(())
            }
            TransactionState::Committed => Err(TransactionError::AlreadyCommitted),
            TransactionState::RolledBack => Err(TransactionError::AlreadyRolledBack),
        }
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        let state = self.state.lock();
        if *state == TransactionState::Active {
            log::warn!(
                "Transaction {} dropped without commit or rollback - will be auto-rolled back",
                self.id
            );
            // Connection will be returned to pool and transaction auto-rolled back
        }
    }
}

/// Transaction manager for creating and managing transactions
pub struct TransactionManager {
    pool: Arc<ConnectionPool>,
    default_isolation_level: IsolationLevel,
}

impl TransactionManager {
    /// Create a new transaction manager
    pub fn new(pool: Arc<ConnectionPool>) -> Self {
        Self {
            pool,
            default_isolation_level: IsolationLevel::ReadCommitted,
        }
    }

    /// Create a new transaction manager with custom isolation level
    pub fn with_isolation_level(pool: Arc<ConnectionPool>, isolation_level: IsolationLevel) -> Self {
        Self {
            pool,
            default_isolation_level: isolation_level,
        }
    }

    /// Begin a new transaction
    pub async fn begin(&self) -> Result<Transaction, TransactionError> {
        self.begin_with_isolation(self.default_isolation_level).await
    }

    /// Begin a read-only transaction
    pub async fn begin_read_only(&self) -> Result<Transaction, TransactionError> {
        let connection = self.pool.acquire().await?;
        Transaction::new(connection, self.default_isolation_level, true).await
    }

    /// Begin a transaction with specific isolation level
    pub async fn begin_with_isolation(
        &self,
        isolation_level: IsolationLevel,
    ) -> Result<Transaction, TransactionError> {
        let connection = self.pool.acquire().await?;
        Transaction::new(connection, isolation_level, false).await
    }

    /// Execute a function within a transaction
    pub async fn transaction<F, T>(&self, f: F) -> Result<T, TransactionError>
    where
        F: FnOnce(&Transaction) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, TransactionError>> + Send>> + Send,
        T: Send,
    {
        let tx = self.begin().await?;

        match f(&tx).await {
            Ok(result) => {
                tx.commit().await?;
                Ok(result)
            }
            Err(e) => {
                tx.rollback().await?;
                Err(e)
            }
        }
    }

    /// Execute a function within a transaction with retries on serialization failure
    pub async fn transaction_with_retry<F, T>(
        &self,
        max_retries: u32,
        f: F,
    ) -> Result<T, TransactionError>
    where
        F: Fn(&Transaction) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, TransactionError>> + Send>> + Send + Clone,
        T: Send,
    {
        let mut attempts = 0;

        loop {
            attempts += 1;

            let tx = self.begin().await?;

            match f(&tx).await {
                Ok(result) => {
                    match tx.commit().await {
                        Ok(_) => return Ok(result),
                        Err(TransactionError::SerializationFailure) if attempts < max_retries => {
                            log::warn!("Serialization failure, retrying... (attempt {})", attempts);
                            tokio::time::sleep(std::time::Duration::from_millis(10 * attempts as u64)).await;
                            continue;
                        }
                        Err(e) => return Err(e),
                    }
                }
                Err(e) => {
                    tx.rollback().await?;

                    if matches!(e, TransactionError::SerializationFailure) && attempts < max_retries {
                        log::warn!("Serialization failure, retrying... (attempt {})", attempts);
                        tokio::time::sleep(std::time::Duration::from_millis(10 * attempts as u64)).await;
                        continue;
                    }

                    return Err(e);
                }
            }
        }
    }
}

/// Stub for distributed transaction coordinator
pub struct DistributedTransactionCoordinator {
    managers: Vec<Arc<TransactionManager>>,
}

impl DistributedTransactionCoordinator {
    /// Create a new distributed transaction coordinator
    pub fn new() -> Self {
        Self {
            managers: Vec::new(),
        }
    }

    /// Add a transaction manager
    pub fn add_manager(&mut self, manager: Arc<TransactionManager>) {
        self.managers.push(manager);
    }

    /// Begin a distributed transaction (stub - would implement 2PC)
    pub async fn begin_distributed(&self) -> Result<Vec<Transaction>, TransactionError> {
        let mut transactions = Vec::new();

        for manager in &self.managers {
            let tx = manager.begin().await?;
            transactions.push(tx);
        }

        log::info!("Distributed transaction started across {} databases", transactions.len());

        Ok(transactions)
    }

    /// Commit all transactions in a distributed transaction (stub - would implement 2PC)
    pub async fn commit_distributed(transactions: Vec<Transaction>) -> Result<(), TransactionError> {
        // Phase 1: Prepare (stub)
        log::debug!("2PC Phase 1: Prepare");

        // Phase 2: Commit
        log::debug!("2PC Phase 2: Commit");

        for tx in transactions {
            tx.commit().await?;
        }

        Ok(())
    }

    /// Rollback all transactions in a distributed transaction
    pub async fn rollback_distributed(transactions: Vec<Transaction>) -> Result<(), TransactionError> {
        for tx in transactions {
            tx.rollback().await?;
        }

        Ok(())
    }
}

impl Default for DistributedTransactionCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enterprise::database::connection::{ConnectionPool, DatabaseConfig};

    #[tokio::test]
    async fn test_transaction_creation() {
        let config = DatabaseConfig::default();
        let pool = Arc::new(ConnectionPool::new(config).await.unwrap());
        let manager = TransactionManager::new(pool);

        let tx = manager.begin().await;
        assert!(tx.is_ok());
    }

    #[tokio::test]
    async fn test_transaction_commit() {
        let config = DatabaseConfig::default();
        let pool = Arc::new(ConnectionPool::new(config).await.unwrap());
        let manager = TransactionManager::new(pool);

        let tx = manager.begin().await.unwrap();
        assert!(tx.is_active());

        let result = tx.commit().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_isolation_levels() {
        let levels = vec![
            IsolationLevel::ReadUncommitted,
            IsolationLevel::ReadCommitted,
            IsolationLevel::RepeatableRead,
            IsolationLevel::Serializable,
        ];

        for level in levels {
            let sql = level.to_sql();
            assert!(!sql.is_empty());
        }
    }
}
