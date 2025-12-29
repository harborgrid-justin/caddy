//! # Database Migration System
//!
//! Provides schema migration management with version control,
//! rollback support, and automatic migration discovery.

use crate::database::{connection_pool::ConnectionPool, DatabaseError, Result};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Migration version (timestamp-based)
pub type MigrationVersion = i64;

/// Migration trait
#[async_trait::async_trait]
pub trait Migration: Send + Sync {
    /// Get the migration version
    fn version(&self) -> MigrationVersion;

    /// Get the migration name
    fn name(&self) -> &str;

    /// Get the migration description
    fn description(&self) -> &str;

    /// Execute the migration (up)
    async fn up(&self, pool: &ConnectionPool) -> Result<()>;

    /// Rollback the migration (down)
    async fn down(&self, pool: &ConnectionPool) -> Result<()>;

    /// Check if this migration can be safely rolled back
    fn is_reversible(&self) -> bool {
        true
    }
}

/// SQL-based migration
pub struct SqlMigration {
    version: MigrationVersion,
    name: String,
    description: String,
    up_sql: String,
    down_sql: Option<String>,
}

impl SqlMigration {
    /// Create a new SQL migration
    pub fn new(
        version: MigrationVersion,
        name: impl Into<String>,
        description: impl Into<String>,
        up_sql: impl Into<String>,
        down_sql: Option<String>,
    ) -> Self {
        Self {
            version,
            name: name.into(),
            description: description.into(),
            up_sql: up_sql.into(),
            down_sql,
        }
    }
}

#[async_trait::async_trait]
impl Migration for SqlMigration {
    fn version(&self) -> MigrationVersion {
        self.version
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn up(&self, pool: &ConnectionPool) -> Result<()> {
        pool.execute(sqlx::query(&self.up_sql))
            .await
            .map_err(|e| DatabaseError::Migration(format!("Migration up failed: {}", e)))?;
        Ok(())
    }

    async fn down(&self, pool: &ConnectionPool) -> Result<()> {
        if let Some(down_sql) = &self.down_sql {
            pool.execute(sqlx::query(down_sql))
                .await
                .map_err(|e| DatabaseError::Migration(format!("Migration down failed: {}", e)))?;
            Ok(())
        } else {
            Err(DatabaseError::Migration(
                "Migration is not reversible".to_string(),
            ))
        }
    }

    fn is_reversible(&self) -> bool {
        self.down_sql.is_some()
    }
}

/// Migration record in the database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
struct MigrationRecord {
    version: i64,
    name: String,
    description: String,
    applied_at: String,
    checksum: String,
}

/// Migration manager
pub struct MigrationManager {
    /// Connection pool
    pool: ConnectionPool,

    /// Registered migrations
    migrations: Arc<RwLock<HashMap<MigrationVersion, Box<dyn Migration>>>>,

    /// Migration history
    history: Arc<RwLock<Vec<MigrationRecord>>>,
}

impl MigrationManager {
    /// Create a new migration manager
    pub fn new(pool: ConnectionPool) -> Self {
        Self {
            pool,
            migrations: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Initialize the migrations table
    pub async fn init(&self) -> Result<()> {
        let create_table_sql = r#"
            CREATE TABLE IF NOT EXISTS _migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                applied_at TEXT NOT NULL,
                checksum TEXT NOT NULL
            )
        "#;

        self.pool
            .execute(sqlx::query(create_table_sql))
            .await
            .map_err(|e| DatabaseError::Migration(format!("Failed to create migrations table: {}", e)))?;

        // Load migration history
        self.load_history().await?;

        Ok(())
    }

    /// Register a migration
    pub fn register<M: Migration + 'static>(&self, migration: M) {
        let version = migration.version();
        self.migrations.write().insert(version, Box::new(migration));
    }

    /// Load migration history from the database
    async fn load_history(&self) -> Result<()> {
        let records: Vec<MigrationRecord> = self
            .pool
            .fetch_all(sqlx::query_as("SELECT * FROM _migrations ORDER BY version"))
            .await
            .map_err(|e| DatabaseError::Migration(format!("Failed to load migration history: {}", e)))?;

        *self.history.write() = records;

        Ok(())
    }

    /// Get pending migrations
    pub fn pending_migrations(&self) -> Vec<MigrationVersion> {
        let applied_versions: Vec<i64> = self
            .history
            .read()
            .iter()
            .map(|r| r.version)
            .collect();

        let mut pending: Vec<MigrationVersion> = self
            .migrations
            .read()
            .keys()
            .filter(|v| !applied_versions.contains(v))
            .copied()
            .collect();

        pending.sort();
        pending
    }

    /// Run all pending migrations
    pub async fn run_pending(&self) -> Result<()> {
        let pending = self.pending_migrations();

        if pending.is_empty() {
            log::info!("No pending migrations");
            return Ok(());
        }

        log::info!("Running {} pending migrations", pending.len());

        for version in pending {
            self.migrate_up(version).await?;
        }

        log::info!("All migrations completed successfully");

        Ok(())
    }

    /// Run a specific migration up
    pub async fn migrate_up(&self, version: MigrationVersion) -> Result<()> {
        let migration = {
            let migrations = self.migrations.read();
            migrations
                .get(&version)
                .ok_or_else(|| {
                    DatabaseError::Migration(format!("Migration {} not found", version))
                })?;

            // We need to clone the reference here since we can't hold the lock across await
            // For now, we'll access it again inside the transaction
            true
        };

        let migrations = self.migrations.read();
        let migration = migrations.get(&version).unwrap();

        log::info!(
            "Running migration {}: {}",
            version,
            migration.name()
        );

        // Begin transaction
        let mut tx = self.pool.begin().await?;

        // Execute migration
        migration.up(&self.pool).await?;

        // Record migration
        let checksum = self.calculate_checksum(version);
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO _migrations (version, name, description, applied_at, checksum) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(version)
        .bind(migration.name())
        .bind(migration.description())
        .bind(&now)
        .bind(&checksum)
        .execute(&mut *tx)
        .await
        .map_err(|e| DatabaseError::Migration(format!("Failed to record migration: {}", e)))?;

        // Commit transaction
        tx.commit().await
            .map_err(|e| DatabaseError::Migration(format!("Failed to commit migration: {}", e)))?;

        // Reload history
        self.load_history().await?;

        log::info!("Migration {} completed", version);

        Ok(())
    }

    /// Rollback a migration
    pub async fn migrate_down(&self, version: MigrationVersion) -> Result<()> {
        // Check if migration is applied
        let is_applied = self
            .history
            .read()
            .iter()
            .any(|r| r.version == version);

        if !is_applied {
            return Err(DatabaseError::Migration(format!(
                "Migration {} is not applied",
                version
            )));
        }

        let migrations = self.migrations.read();
        let migration = migrations
            .get(&version)
            .ok_or_else(|| DatabaseError::Migration(format!("Migration {} not found", version)))?;

        if !migration.is_reversible() {
            return Err(DatabaseError::Migration(format!(
                "Migration {} is not reversible",
                version
            )));
        }

        log::info!(
            "Rolling back migration {}: {}",
            version,
            migration.name()
        );

        // Begin transaction
        let mut tx = self.pool.begin().await?;

        // Execute rollback
        migration.down(&self.pool).await?;

        // Remove migration record
        sqlx::query("DELETE FROM _migrations WHERE version = ?")
            .bind(version)
            .execute(&mut *tx)
            .await
            .map_err(|e| DatabaseError::Migration(format!("Failed to remove migration record: {}", e)))?;

        // Commit transaction
        tx.commit().await
            .map_err(|e| DatabaseError::Migration(format!("Failed to commit rollback: {}", e)))?;

        // Reload history
        self.load_history().await?;

        log::info!("Migration {} rolled back", version);

        Ok(())
    }

    /// Rollback the last N migrations
    pub async fn rollback(&self, count: usize) -> Result<()> {
        let mut applied_versions: Vec<i64> = self
            .history
            .read()
            .iter()
            .map(|r| r.version)
            .collect();

        applied_versions.sort();
        applied_versions.reverse();

        let to_rollback = applied_versions.into_iter().take(count);

        for version in to_rollback {
            self.migrate_down(version).await?;
        }

        Ok(())
    }

    /// Get migration status
    pub fn status(&self) -> MigrationStatus {
        let total = self.migrations.read().len();
        let applied = self.history.read().len();
        let pending = self.pending_migrations().len();

        let history: Vec<MigrationInfo> = self
            .history
            .read()
            .iter()
            .map(|r| MigrationInfo {
                version: r.version,
                name: r.name.clone(),
                description: r.description.clone(),
                applied_at: Some(r.applied_at.clone()),
                is_applied: true,
            })
            .collect();

        let pending_info: Vec<MigrationInfo> = {
            let migrations = self.migrations.read();
            self.pending_migrations()
                .into_iter()
                .filter_map(|v| {
                    migrations.get(&v).map(|m| MigrationInfo {
                        version: v,
                        name: m.name().to_string(),
                        description: m.description().to_string(),
                        applied_at: None,
                        is_applied: false,
                    })
                })
                .collect()
        };

        MigrationStatus {
            total,
            applied,
            pending,
            history,
            pending_migrations: pending_info,
        }
    }

    /// Calculate checksum for a migration
    fn calculate_checksum(&self, version: MigrationVersion) -> String {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(version.to_string().as_bytes());

        format!("{:x}", hasher.finalize())
    }

    /// Reset all migrations (dangerous!)
    pub async fn reset(&self) -> Result<()> {
        log::warn!("Resetting all migrations - this will drop and recreate the database schema");

        // Rollback all migrations
        let count = self.history.read().len();
        self.rollback(count).await?;

        Ok(())
    }
}

/// Migration status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStatus {
    /// Total number of migrations
    pub total: usize,

    /// Number of applied migrations
    pub applied: usize,

    /// Number of pending migrations
    pub pending: usize,

    /// Migration history
    pub history: Vec<MigrationInfo>,

    /// Pending migrations
    pub pending_migrations: Vec<MigrationInfo>,
}

/// Migration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationInfo {
    /// Migration version
    pub version: MigrationVersion,

    /// Migration name
    pub name: String,

    /// Migration description
    pub description: String,

    /// When this migration was applied
    pub applied_at: Option<String>,

    /// Whether this migration is applied
    pub is_applied: bool,
}

/// Initialize default migrations for CADDY
pub fn init_default_migrations(manager: &MigrationManager) {
    // Migration 1: Create entities table
    manager.register(SqlMigration::new(
        20250101000001,
        "create_entities_table",
        "Create the main entities table for CAD objects",
        r#"
            CREATE TABLE entities (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                uuid TEXT NOT NULL UNIQUE,
                entity_type TEXT NOT NULL,
                layer_id INTEGER,
                geometry_data BLOB,
                metadata TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX idx_entities_layer ON entities(layer_id);
            CREATE INDEX idx_entities_type ON entities(entity_type);
            CREATE INDEX idx_entities_uuid ON entities(uuid);
        "#,
        Some(r#"
            DROP INDEX IF EXISTS idx_entities_uuid;
            DROP INDEX IF EXISTS idx_entities_type;
            DROP INDEX IF EXISTS idx_entities_layer;
            DROP TABLE entities;
        "#.to_string()),
    ));

    // Migration 2: Create layers table
    manager.register(SqlMigration::new(
        20250101000002,
        "create_layers_table",
        "Create the layers table",
        r#"
            CREATE TABLE layers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                color TEXT,
                visible INTEGER NOT NULL DEFAULT 1,
                locked INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX idx_layers_name ON layers(name);
        "#,
        Some(r#"
            DROP INDEX IF EXISTS idx_layers_name;
            DROP TABLE layers;
        "#.to_string()),
    ));

    // Migration 3: Create spatial index table
    manager.register(SqlMigration::new(
        20250101000003,
        "create_spatial_index",
        "Create spatial index for geometric queries",
        r#"
            CREATE TABLE spatial_index (
                entity_id INTEGER PRIMARY KEY,
                min_x REAL NOT NULL,
                min_y REAL NOT NULL,
                min_z REAL NOT NULL,
                max_x REAL NOT NULL,
                max_y REAL NOT NULL,
                max_z REAL NOT NULL,
                FOREIGN KEY (entity_id) REFERENCES entities(id) ON DELETE CASCADE
            );
            CREATE INDEX idx_spatial_bbox ON spatial_index(min_x, min_y, max_x, max_y);
        "#,
        Some(r#"
            DROP INDEX IF EXISTS idx_spatial_bbox;
            DROP TABLE spatial_index;
        "#.to_string()),
    ));

    // Migration 4: Create documents table
    manager.register(SqlMigration::new(
        20250101000004,
        "create_documents_table",
        "Create the documents table for CAD files",
        r#"
            CREATE TABLE documents (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                uuid TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                description TEXT,
                file_path TEXT,
                metadata TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX idx_documents_uuid ON documents(uuid);
        "#,
        Some(r#"
            DROP INDEX IF EXISTS idx_documents_uuid;
            DROP TABLE documents;
        "#.to_string()),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_migration_manager() {
        use crate::database::connection_pool::DatabaseConfig;

        let config = DatabaseConfig {
            url: "sqlite::memory:".to_string(),
            ..Default::default()
        };

        let pool = ConnectionPool::new(config).await.unwrap();
        let manager = MigrationManager::new(pool);

        // Initialize
        assert!(manager.init().await.is_ok());

        // Register a test migration
        manager.register(SqlMigration::new(
            1,
            "test",
            "Test migration",
            "CREATE TABLE test (id INTEGER PRIMARY KEY)",
            Some("DROP TABLE test".to_string()),
        ));

        // Check pending
        let pending = manager.pending_migrations();
        assert_eq!(pending.len(), 1);

        // Run migration
        assert!(manager.run_pending().await.is_ok());

        // Check no pending
        let pending = manager.pending_migrations();
        assert_eq!(pending.len(), 0);

        // Check status
        let status = manager.status();
        assert_eq!(status.applied, 1);
        assert_eq!(status.pending, 0);
    }
}
