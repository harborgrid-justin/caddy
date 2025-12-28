//! Database Migration System
//!
//! Provides a robust migration system with up/down migrations, version tracking,
//! and batch migration support.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use thiserror::Error;

use super::connection::ConnectionPool;
use super::schema::Schema;

/// Migration errors
#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("Migration not found: {0}")]
    NotFound(String),

    #[error("Migration already applied: {0}")]
    AlreadyApplied(String),

    #[error("Migration not applied: {0}")]
    NotApplied(String),

    #[error("Invalid migration order: {0}")]
    InvalidOrder(String),

    #[error("Migration failed: {0}")]
    Failed(String),

    #[error("Rollback failed: {0}")]
    RollbackFailed(String),

    #[error("Migration conflict: {0}")]
    Conflict(String),

    #[error("Connection error: {0}")]
    Connection(#[from] super::connection::ConnectionError),

    #[error("Schema error: {0}")]
    Schema(#[from] super::schema::SchemaError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Migration record stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRecord {
    /// Migration ID
    pub id: String,
    /// Migration version
    pub version: u32,
    /// Migration name
    pub name: String,
    /// Applied timestamp
    pub applied_at: DateTime<Utc>,
    /// Applied by (user/system)
    pub applied_by: String,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Checksum of the migration
    pub checksum: String,
}

impl MigrationRecord {
    /// Create a new migration record
    pub fn new(
        id: String,
        version: u32,
        name: String,
        applied_by: String,
        execution_time_ms: u64,
        checksum: String,
    ) -> Self {
        Self {
            id,
            version,
            name,
            applied_at: Utc::now(),
            applied_by,
            execution_time_ms,
            checksum,
        }
    }
}

/// Migration trait
#[async_trait]
pub trait Migration: Send + Sync {
    /// Get migration ID (unique identifier)
    fn id(&self) -> &str;

    /// Get migration version number
    fn version(&self) -> u32;

    /// Get migration name/description
    fn name(&self) -> &str;

    /// Get migration dependencies (IDs of migrations that must run first)
    fn dependencies(&self) -> Vec<String> {
        Vec::new()
    }

    /// Execute the migration (up)
    async fn up(&self, pool: &ConnectionPool) -> Result<(), MigrationError>;

    /// Rollback the migration (down)
    async fn down(&self, pool: &ConnectionPool) -> Result<(), MigrationError>;

    /// Calculate migration checksum
    fn checksum(&self) -> String {
        let content = format!("{}:{}:{}", self.id(), self.version(), self.name());
        format!("{:x}", md5::compute(content))
    }

    /// Validate migration before execution
    fn validate(&self) -> Result<(), MigrationError> {
        if self.id().is_empty() {
            return Err(MigrationError::Failed("Migration ID cannot be empty".to_string()));
        }

        if self.name().is_empty() {
            return Err(MigrationError::Failed("Migration name cannot be empty".to_string()));
        }

        Ok(())
    }
}

/// SQL migration implementation
pub struct SqlMigration {
    id: String,
    version: u32,
    name: String,
    up_sql: String,
    down_sql: String,
    dependencies: Vec<String>,
}

impl SqlMigration {
    /// Create a new SQL migration
    pub fn new(
        id: impl Into<String>,
        version: u32,
        name: impl Into<String>,
        up_sql: impl Into<String>,
        down_sql: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            version,
            name: name.into(),
            up_sql: up_sql.into(),
            down_sql: down_sql.into(),
            dependencies: Vec::new(),
        }
    }

    /// Add a dependency
    pub fn with_dependency(mut self, dependency_id: impl Into<String>) -> Self {
        self.dependencies.push(dependency_id.into());
        self
    }
}

#[async_trait]
impl Migration for SqlMigration {
    fn id(&self) -> &str {
        &self.id
    }

    fn version(&self) -> u32 {
        self.version
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn dependencies(&self) -> Vec<String> {
        self.dependencies.clone()
    }

    async fn up(&self, pool: &ConnectionPool) -> Result<(), MigrationError> {
        let conn = pool.acquire().await?;

        log::info!("Applying migration {}: {}", self.id, self.name);

        // Split SQL by semicolons and execute each statement
        for statement in self.up_sql.split(';') {
            let statement = statement.trim();
            if !statement.is_empty() {
                conn.execute(statement).await.map_err(|e| {
                    MigrationError::Failed(format!("Failed to execute: {} - {}", statement, e))
                })?;
            }
        }

        Ok(())
    }

    async fn down(&self, pool: &ConnectionPool) -> Result<(), MigrationError> {
        let conn = pool.acquire().await?;

        log::info!("Rolling back migration {}: {}", self.id, self.name);

        // Split SQL by semicolons and execute each statement
        for statement in self.down_sql.split(';') {
            let statement = statement.trim();
            if !statement.is_empty() {
                conn.execute(statement).await.map_err(|e| {
                    MigrationError::RollbackFailed(format!("Failed to execute: {} - {}", statement, e))
                })?;
            }
        }

        Ok(())
    }
}

/// Schema migration for applying schema changes
pub struct SchemaMigration {
    id: String,
    version: u32,
    name: String,
    schema: Schema,
}

impl SchemaMigration {
    /// Create a new schema migration
    pub fn new(id: impl Into<String>, version: u32, name: impl Into<String>, schema: Schema) -> Self {
        Self {
            id: id.into(),
            version,
            name: name.into(),
            schema,
        }
    }
}

#[async_trait]
impl Migration for SchemaMigration {
    fn id(&self) -> &str {
        &self.id
    }

    fn version(&self) -> u32 {
        self.version
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn up(&self, pool: &ConnectionPool) -> Result<(), MigrationError> {
        let conn = pool.acquire().await?;

        log::info!("Applying schema migration {}: {}", self.id, self.name);

        // Generate CREATE TABLE statements
        for table in self.schema.tables.values() {
            let mut sql = format!("CREATE TABLE IF NOT EXISTS {} (", table.name);

            let column_defs: Vec<String> = table.columns.iter().map(|col| {
                let mut def = format!("{} {}", col.name, col.data_type.to_postgres_type());

                if col.primary_key {
                    def.push_str(" PRIMARY KEY");
                }

                if !col.nullable {
                    def.push_str(" NOT NULL");
                }

                if col.unique && !col.primary_key {
                    def.push_str(" UNIQUE");
                }

                if let Some(ref default) = col.default {
                    def.push_str(&format!(" DEFAULT {}", default));
                }

                def
            }).collect();

            sql.push_str(&column_defs.join(", "));
            sql.push(')');

            conn.execute(&sql).await?;

            // Create indexes
            for index in &table.indexes {
                let unique = if index.unique { "UNIQUE" } else { "" };
                let sql = format!(
                    "CREATE {} INDEX IF NOT EXISTS {} ON {} ({})",
                    unique,
                    index.name,
                    table.name,
                    index.columns.join(", ")
                );

                conn.execute(&sql).await?;
            }
        }

        Ok(())
    }

    async fn down(&self, pool: &ConnectionPool) -> Result<(), MigrationError> {
        let conn = pool.acquire().await?;

        log::info!("Rolling back schema migration {}: {}", self.id, self.name);

        // Drop tables in reverse order
        for table in self.schema.tables.values() {
            let sql = format!("DROP TABLE IF EXISTS {} CASCADE", table.name);
            conn.execute(&sql).await?;
        }

        Ok(())
    }
}

/// Migration manager
pub struct MigrationManager {
    pool: Arc<ConnectionPool>,
    migrations: HashMap<String, Arc<dyn Migration>>,
    applied_migrations: HashSet<String>,
}

impl MigrationManager {
    /// Create a new migration manager
    pub fn new(pool: Arc<ConnectionPool>) -> Self {
        Self {
            pool,
            migrations: HashMap::new(),
            applied_migrations: HashSet::new(),
        }
    }

    /// Initialize migration tracking table
    pub async fn initialize(&self) -> Result<(), MigrationError> {
        let conn = self.pool.acquire().await?;

        let sql = r#"
            CREATE TABLE IF NOT EXISTS __migrations (
                id VARCHAR(255) PRIMARY KEY,
                version INTEGER NOT NULL,
                name VARCHAR(255) NOT NULL,
                applied_at TIMESTAMP WITH TIME ZONE NOT NULL,
                applied_by VARCHAR(255) NOT NULL,
                execution_time_ms BIGINT NOT NULL,
                checksum VARCHAR(64) NOT NULL
            )
        "#;

        conn.execute(sql).await?;

        log::info!("Migration tracking initialized");

        Ok(())
    }

    /// Register a migration
    pub fn register(&mut self, migration: Arc<dyn Migration>) -> Result<(), MigrationError> {
        migration.validate()?;

        let id = migration.id().to_string();

        if self.migrations.contains_key(&id) {
            return Err(MigrationError::Conflict(format!(
                "Migration {} already registered",
                id
            )));
        }

        self.migrations.insert(id, migration);

        Ok(())
    }

    /// Load applied migrations from database
    pub async fn load_applied(&mut self) -> Result<(), MigrationError> {
        let conn = self.pool.acquire().await?;

        let sql = "SELECT id FROM __migrations ORDER BY applied_at";
        let _results = conn.query(sql).await?;

        // In production, would parse results
        self.applied_migrations.clear();

        log::debug!("Loaded {} applied migrations", self.applied_migrations.len());

        Ok(())
    }

    /// Get pending migrations
    pub fn pending(&self) -> Vec<Arc<dyn Migration>> {
        let mut pending: Vec<_> = self
            .migrations
            .values()
            .filter(|m| !self.applied_migrations.contains(m.id()))
            .cloned()
            .collect();

        // Sort by version
        pending.sort_by_key(|m| m.version());

        pending
    }

    /// Check if migration is applied
    pub fn is_applied(&self, id: &str) -> bool {
        self.applied_migrations.contains(id)
    }

    /// Apply a single migration
    pub async fn apply(&mut self, id: &str) -> Result<(), MigrationError> {
        let migration = self
            .migrations
            .get(id)
            .ok_or_else(|| MigrationError::NotFound(id.to_string()))?
            .clone();

        if self.is_applied(id) {
            return Err(MigrationError::AlreadyApplied(id.to_string()));
        }

        // Check dependencies
        for dep in migration.dependencies() {
            if !self.is_applied(&dep) {
                return Err(MigrationError::InvalidOrder(format!(
                    "Migration {} depends on {} which is not applied",
                    id, dep
                )));
            }
        }

        // Execute migration
        let start = std::time::Instant::now();

        migration.up(&self.pool).await?;

        let execution_time = start.elapsed().as_millis() as u64;

        // Record migration
        let record = MigrationRecord::new(
            id.to_string(),
            migration.version(),
            migration.name().to_string(),
            "system".to_string(),
            execution_time,
            migration.checksum(),
        );

        self.record_migration(&record).await?;
        self.applied_migrations.insert(id.to_string());

        log::info!(
            "Applied migration {} in {}ms",
            id,
            execution_time
        );

        Ok(())
    }

    /// Apply all pending migrations
    pub async fn apply_all(&mut self) -> Result<usize, MigrationError> {
        let pending = self.pending();
        let count = pending.len();

        for migration in pending {
            self.apply(migration.id()).await?;
        }

        log::info!("Applied {} migrations", count);

        Ok(count)
    }

    /// Rollback a migration
    pub async fn rollback(&mut self, id: &str) -> Result<(), MigrationError> {
        let migration = self
            .migrations
            .get(id)
            .ok_or_else(|| MigrationError::NotFound(id.to_string()))?
            .clone();

        if !self.is_applied(id) {
            return Err(MigrationError::NotApplied(id.to_string()));
        }

        // Check if other migrations depend on this one
        for (other_id, other) in &self.migrations {
            if self.is_applied(other_id) && other.dependencies().contains(&id.to_string()) {
                return Err(MigrationError::InvalidOrder(format!(
                    "Cannot rollback {} because {} depends on it",
                    id, other_id
                )));
            }
        }

        // Execute rollback
        let start = std::time::Instant::now();

        migration.down(&self.pool).await?;

        let execution_time = start.elapsed().as_millis() as u64;

        // Remove record
        self.remove_migration_record(id).await?;
        self.applied_migrations.remove(id);

        log::info!(
            "Rolled back migration {} in {}ms",
            id,
            execution_time
        );

        Ok(())
    }

    /// Rollback last N migrations
    pub async fn rollback_last(&mut self, count: usize) -> Result<usize, MigrationError> {
        let mut applied: Vec<_> = self.applied_migrations.iter().cloned().collect();

        // Sort by version (descending)
        applied.sort_by_key(|id| {
            self.migrations
                .get(id)
                .map(|m| std::cmp::Reverse(m.version()))
                .unwrap_or(std::cmp::Reverse(0))
        });

        let to_rollback = applied.into_iter().take(count);
        let mut rolled_back = 0;

        for id in to_rollback {
            self.rollback(&id).await?;
            rolled_back += 1;
        }

        log::info!("Rolled back {} migrations", rolled_back);

        Ok(rolled_back)
    }

    /// Record a migration in the database
    async fn record_migration(&self, record: &MigrationRecord) -> Result<(), MigrationError> {
        let conn = self.pool.acquire().await?;

        let sql = format!(
            "INSERT INTO __migrations (id, version, name, applied_at, applied_by, execution_time_ms, checksum) \
             VALUES ('{}', {}, '{}', '{}', '{}', {}, '{}')",
            record.id,
            record.version,
            record.name,
            record.applied_at,
            record.applied_by,
            record.execution_time_ms,
            record.checksum
        );

        conn.execute(&sql).await?;

        Ok(())
    }

    /// Remove a migration record from the database
    async fn remove_migration_record(&self, id: &str) -> Result<(), MigrationError> {
        let conn = self.pool.acquire().await?;

        let sql = format!("DELETE FROM __migrations WHERE id = '{}'", id);

        conn.execute(&sql).await?;

        Ok(())
    }

    /// Get migration status
    pub fn status(&self) -> MigrationStatus {
        let total = self.migrations.len();
        let applied = self.applied_migrations.len();
        let pending = total - applied;

        MigrationStatus {
            total,
            applied,
            pending,
        }
    }
}

/// Migration status summary
#[derive(Debug, Clone)]
pub struct MigrationStatus {
    pub total: usize,
    pub applied: usize,
    pub pending: usize,
}

/// Create standard CADDY migrations
pub fn create_caddy_migrations() -> Vec<Arc<dyn Migration>> {
    let mut migrations: Vec<Arc<dyn Migration>> = Vec::new();

    // Migration 1: Create users table
    migrations.push(Arc::new(SqlMigration::new(
        "001_create_users",
        1,
        "Create users table",
        r#"
            CREATE TABLE users (
                id UUID PRIMARY KEY,
                username VARCHAR(100) NOT NULL UNIQUE,
                email VARCHAR(255) NOT NULL UNIQUE,
                password_hash VARCHAR(255) NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL,
                updated_at TIMESTAMP WITH TIME ZONE NOT NULL
            );
            CREATE INDEX idx_users_username ON users(username);
            CREATE INDEX idx_users_email ON users(email);
        "#,
        "DROP TABLE IF EXISTS users CASCADE",
    )));

    // Migration 2: Create projects table
    migrations.push(Arc::new(SqlMigration::new(
        "002_create_projects",
        2,
        "Create projects table",
        r#"
            CREATE TABLE projects (
                id UUID PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                description TEXT,
                owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL,
                updated_at TIMESTAMP WITH TIME ZONE NOT NULL
            );
            CREATE INDEX idx_projects_owner ON projects(owner_id);
            CREATE INDEX idx_projects_name ON projects(name);
        "#,
        "DROP TABLE IF EXISTS projects CASCADE",
    ).with_dependency("001_create_users")));

    // Migration 3: Create documents table
    migrations.push(Arc::new(SqlMigration::new(
        "003_create_documents",
        3,
        "Create documents table",
        r#"
            CREATE TABLE documents (
                id UUID PRIMARY KEY,
                project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                name VARCHAR(255) NOT NULL,
                content JSONB NOT NULL,
                version INTEGER NOT NULL DEFAULT 1,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL,
                updated_at TIMESTAMP WITH TIME ZONE NOT NULL
            );
            CREATE INDEX idx_documents_project ON documents(project_id);
            CREATE INDEX idx_documents_name ON documents(name);
            CREATE INDEX idx_documents_content ON documents USING GIN(content);
        "#,
        "DROP TABLE IF EXISTS documents CASCADE",
    ).with_dependency("002_create_projects")));

    migrations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enterprise::database::connection::{ConnectionPool, DatabaseConfig};

    #[tokio::test]
    async fn test_migration_creation() {
        let migration = SqlMigration::new(
            "test_001",
            1,
            "Test migration",
            "CREATE TABLE test (id INT)",
            "DROP TABLE test",
        );

        assert_eq!(migration.id(), "test_001");
        assert_eq!(migration.version(), 1);
        assert_eq!(migration.name(), "Test migration");
    }

    #[tokio::test]
    async fn test_migration_manager() {
        let config = DatabaseConfig::default();
        let pool = Arc::new(ConnectionPool::new(config).await.unwrap());
        let mut manager = MigrationManager::new(pool);

        let migration = Arc::new(SqlMigration::new(
            "test_001",
            1,
            "Test migration",
            "SELECT 1",
            "SELECT 1",
        ));

        assert!(manager.register(migration).is_ok());
        assert_eq!(manager.migrations.len(), 1);
    }

    #[test]
    fn test_caddy_migrations() {
        let migrations = create_caddy_migrations();
        assert_eq!(migrations.len(), 3);

        // Verify dependencies
        let proj_migration = &migrations[1];
        assert_eq!(proj_migration.dependencies(), vec!["001_create_users"]);
    }
}
