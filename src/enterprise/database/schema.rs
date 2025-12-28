//! Database Schema Management
//!
//! Provides schema definitions, versioning, and migration support for CAD entities.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Schema management errors
#[derive(Debug, Error)]
pub enum SchemaError {
    #[error("Schema version {0} not found")]
    VersionNotFound(u32),

    #[error("Invalid schema: {0}")]
    InvalidSchema(String),

    #[error("Schema conflict: {0}")]
    Conflict(String),

    #[error("Migration required from version {0} to {1}")]
    MigrationRequired(u32, u32),

    #[error("Rollback not supported for version {0}")]
    RollbackNotSupported(u32),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Schema version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaVersion {
    /// Version number
    pub version: u32,
    /// Version name/description
    pub name: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Schema hash for verification
    pub schema_hash: String,
    /// Applied migrations
    pub applied_migrations: Vec<String>,
}

impl SchemaVersion {
    /// Create a new schema version
    pub fn new(version: u32, name: String) -> Self {
        Self {
            version,
            name,
            created_at: Utc::now(),
            schema_hash: String::new(),
            applied_migrations: Vec::new(),
        }
    }

    /// Calculate schema hash
    pub fn calculate_hash(&mut self, schema: &Schema) -> Result<(), SchemaError> {
        let schema_json = serde_json::to_string(schema)?;
        self.schema_hash = format!("{:x}", md5::compute(schema_json));
        Ok(())
    }
}

/// Column data type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ColumnType {
    /// Integer (32-bit)
    Integer,
    /// Big integer (64-bit)
    BigInt,
    /// Floating point
    Float,
    /// Double precision float
    Double,
    /// Boolean
    Boolean,
    /// Variable-length string
    String { max_length: Option<usize> },
    /// Fixed-length string
    Char { length: usize },
    /// Text (unlimited length)
    Text,
    /// Binary data
    Binary { max_length: Option<usize> },
    /// UUID
    Uuid,
    /// Timestamp with timezone
    Timestamp,
    /// Date only
    Date,
    /// Time only
    Time,
    /// JSON data
    Json,
    /// Array of another type
    Array { element_type: Box<ColumnType> },
    /// Custom type
    Custom { type_name: String },
}

impl ColumnType {
    /// Convert to SQL type string for PostgreSQL
    pub fn to_postgres_type(&self) -> String {
        match self {
            ColumnType::Integer => "INTEGER".to_string(),
            ColumnType::BigInt => "BIGINT".to_string(),
            ColumnType::Float => "REAL".to_string(),
            ColumnType::Double => "DOUBLE PRECISION".to_string(),
            ColumnType::Boolean => "BOOLEAN".to_string(),
            ColumnType::String { max_length } => {
                if let Some(len) = max_length {
                    format!("VARCHAR({})", len)
                } else {
                    "VARCHAR".to_string()
                }
            }
            ColumnType::Char { length } => format!("CHAR({})", length),
            ColumnType::Text => "TEXT".to_string(),
            ColumnType::Binary { max_length } => {
                if let Some(len) = max_length {
                    format!("BYTEA({})", len)
                } else {
                    "BYTEA".to_string()
                }
            }
            ColumnType::Uuid => "UUID".to_string(),
            ColumnType::Timestamp => "TIMESTAMP WITH TIME ZONE".to_string(),
            ColumnType::Date => "DATE".to_string(),
            ColumnType::Time => "TIME".to_string(),
            ColumnType::Json => "JSONB".to_string(),
            ColumnType::Array { element_type } => {
                format!("{}[]", element_type.to_postgres_type())
            }
            ColumnType::Custom { type_name } => type_name.clone(),
        }
    }

    /// Convert to SQL type string for MySQL
    pub fn to_mysql_type(&self) -> String {
        match self {
            ColumnType::Integer => "INT".to_string(),
            ColumnType::BigInt => "BIGINT".to_string(),
            ColumnType::Float => "FLOAT".to_string(),
            ColumnType::Double => "DOUBLE".to_string(),
            ColumnType::Boolean => "BOOLEAN".to_string(),
            ColumnType::String { max_length } => {
                if let Some(len) = max_length {
                    format!("VARCHAR({})", len)
                } else {
                    "VARCHAR(255)".to_string()
                }
            }
            ColumnType::Char { length } => format!("CHAR({})", length),
            ColumnType::Text => "TEXT".to_string(),
            ColumnType::Binary { max_length } => {
                if let Some(len) = max_length {
                    format!("VARBINARY({})", len)
                } else {
                    "BLOB".to_string()
                }
            }
            ColumnType::Uuid => "CHAR(36)".to_string(),
            ColumnType::Timestamp => "TIMESTAMP".to_string(),
            ColumnType::Date => "DATE".to_string(),
            ColumnType::Time => "TIME".to_string(),
            ColumnType::Json => "JSON".to_string(),
            ColumnType::Array { .. } => "JSON".to_string(), // MySQL doesn't have array type
            ColumnType::Custom { type_name } => type_name.clone(),
        }
    }

    /// Convert to SQL type string for SQLite
    pub fn to_sqlite_type(&self) -> String {
        match self {
            ColumnType::Integer | ColumnType::BigInt => "INTEGER".to_string(),
            ColumnType::Float | ColumnType::Double => "REAL".to_string(),
            ColumnType::Boolean => "INTEGER".to_string(),
            ColumnType::String { .. } | ColumnType::Char { .. } | ColumnType::Text => {
                "TEXT".to_string()
            }
            ColumnType::Binary { .. } => "BLOB".to_string(),
            ColumnType::Uuid => "TEXT".to_string(),
            ColumnType::Timestamp | ColumnType::Date | ColumnType::Time => "TEXT".to_string(),
            ColumnType::Json | ColumnType::Array { .. } => "TEXT".to_string(),
            ColumnType::Custom { .. } => "TEXT".to_string(),
        }
    }
}

/// Column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    /// Column name
    pub name: String,
    /// Data type
    pub data_type: ColumnType,
    /// Is nullable
    pub nullable: bool,
    /// Is primary key
    pub primary_key: bool,
    /// Is unique
    pub unique: bool,
    /// Default value
    pub default: Option<String>,
    /// Foreign key reference
    pub foreign_key: Option<ForeignKey>,
    /// Column comment
    pub comment: Option<String>,
}

impl Column {
    /// Create a new column
    pub fn new(name: impl Into<String>, data_type: ColumnType) -> Self {
        Self {
            name: name.into(),
            data_type,
            nullable: true,
            primary_key: false,
            unique: false,
            default: None,
            foreign_key: None,
            comment: None,
        }
    }

    /// Mark as primary key
    pub fn primary_key(mut self) -> Self {
        self.primary_key = true;
        self.nullable = false;
        self
    }

    /// Mark as not null
    pub fn not_null(mut self) -> Self {
        self.nullable = false;
        self
    }

    /// Mark as unique
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    /// Set default value
    pub fn default(mut self, value: impl Into<String>) -> Self {
        self.default = Some(value.into());
        self
    }

    /// Add foreign key constraint
    pub fn foreign_key(mut self, fk: ForeignKey) -> Self {
        self.foreign_key = Some(fk);
        self
    }
}

/// Foreign key constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKey {
    /// Referenced table
    pub table: String,
    /// Referenced column
    pub column: String,
    /// On delete action
    pub on_delete: ForeignKeyAction,
    /// On update action
    pub on_update: ForeignKeyAction,
}

/// Foreign key actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ForeignKeyAction {
    Cascade,
    SetNull,
    SetDefault,
    Restrict,
    NoAction,
}

impl ForeignKeyAction {
    pub fn to_sql(&self) -> &'static str {
        match self {
            ForeignKeyAction::Cascade => "CASCADE",
            ForeignKeyAction::SetNull => "SET NULL",
            ForeignKeyAction::SetDefault => "SET DEFAULT",
            ForeignKeyAction::Restrict => "RESTRICT",
            ForeignKeyAction::NoAction => "NO ACTION",
        }
    }
}

/// Index type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexType {
    BTree,
    Hash,
    GiST,
    GIN,
    BRIN,
}

/// Index definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    /// Index name
    pub name: String,
    /// Columns in the index
    pub columns: Vec<String>,
    /// Is unique index
    pub unique: bool,
    /// Index type
    pub index_type: IndexType,
    /// Where clause for partial index
    pub where_clause: Option<String>,
}

/// Table definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    /// Table name
    pub name: String,
    /// Columns
    pub columns: Vec<Column>,
    /// Indexes
    pub indexes: Vec<Index>,
    /// Table comment
    pub comment: Option<String>,
}

impl Table {
    /// Create a new table
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            columns: Vec::new(),
            indexes: Vec::new(),
            comment: None,
        }
    }

    /// Add a column
    pub fn add_column(mut self, column: Column) -> Self {
        self.columns.push(column);
        self
    }

    /// Add an index
    pub fn add_index(mut self, index: Index) -> Self {
        self.indexes.push(index);
        self
    }

    /// Get primary key columns
    pub fn primary_keys(&self) -> Vec<&Column> {
        self.columns.iter().filter(|c| c.primary_key).collect()
    }
}

/// Database schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    /// Schema name
    pub name: String,
    /// Schema version
    pub version: u32,
    /// Tables
    pub tables: HashMap<String, Table>,
    /// Custom types
    pub custom_types: HashMap<String, String>,
}

impl Schema {
    /// Create a new schema
    pub fn new(name: impl Into<String>, version: u32) -> Self {
        Self {
            name: name.into(),
            version,
            tables: HashMap::new(),
            custom_types: HashMap::new(),
        }
    }

    /// Add a table
    pub fn add_table(&mut self, table: Table) -> Result<(), SchemaError> {
        if self.tables.contains_key(&table.name) {
            return Err(SchemaError::Conflict(format!(
                "Table {} already exists",
                table.name
            )));
        }
        self.tables.insert(table.name.clone(), table);
        Ok(())
    }

    /// Get a table
    pub fn get_table(&self, name: &str) -> Option<&Table> {
        self.tables.get(name)
    }

    /// Remove a table
    pub fn remove_table(&mut self, name: &str) -> Option<Table> {
        self.tables.remove(name)
    }
}

/// Schema manager for managing database schemas
pub struct SchemaManager {
    current_schema: Schema,
    versions: HashMap<u32, SchemaVersion>,
}

impl SchemaManager {
    /// Create a new schema manager
    pub fn new(schema: Schema) -> Self {
        Self {
            current_schema: schema,
            versions: HashMap::new(),
        }
    }

    /// Get current schema
    pub fn current_schema(&self) -> &Schema {
        &self.current_schema
    }

    /// Update to a new schema version
    pub fn update_schema(&mut self, schema: Schema) -> Result<(), SchemaError> {
        if schema.version <= self.current_schema.version {
            return Err(SchemaError::InvalidSchema(
                "New schema version must be greater than current version".to_string(),
            ));
        }

        let mut version = SchemaVersion::new(schema.version, schema.name.clone());
        version.calculate_hash(&schema)?;

        self.versions.insert(schema.version, version);
        self.current_schema = schema;

        Ok(())
    }

    /// Get schema version
    pub fn get_version(&self, version: u32) -> Option<&SchemaVersion> {
        self.versions.get(&version)
    }

    /// List all versions
    pub fn list_versions(&self) -> Vec<&SchemaVersion> {
        let mut versions: Vec<_> = self.versions.values().collect();
        versions.sort_by_key(|v| v.version);
        versions
    }

    /// Create standard CADDY schema
    pub fn create_caddy_schema() -> Schema {
        let mut schema = Schema::new("caddy", 1);

        // Users table
        let users_table = Table::new("users")
            .add_column(
                Column::new("id", ColumnType::Uuid)
                    .primary_key()
            )
            .add_column(
                Column::new("username", ColumnType::String { max_length: Some(100) })
                    .not_null()
                    .unique()
            )
            .add_column(
                Column::new("email", ColumnType::String { max_length: Some(255) })
                    .not_null()
                    .unique()
            )
            .add_column(
                Column::new("password_hash", ColumnType::String { max_length: Some(255) })
                    .not_null()
            )
            .add_column(
                Column::new("created_at", ColumnType::Timestamp)
                    .not_null()
            )
            .add_column(
                Column::new("updated_at", ColumnType::Timestamp)
                    .not_null()
            );

        // Projects table
        let projects_table = Table::new("projects")
            .add_column(
                Column::new("id", ColumnType::Uuid)
                    .primary_key()
            )
            .add_column(
                Column::new("name", ColumnType::String { max_length: Some(255) })
                    .not_null()
            )
            .add_column(
                Column::new("description", ColumnType::Text)
            )
            .add_column(
                Column::new("owner_id", ColumnType::Uuid)
                    .not_null()
                    .foreign_key(ForeignKey {
                        table: "users".to_string(),
                        column: "id".to_string(),
                        on_delete: ForeignKeyAction::Cascade,
                        on_update: ForeignKeyAction::Cascade,
                    })
            )
            .add_column(
                Column::new("created_at", ColumnType::Timestamp)
                    .not_null()
            );

        // Documents table
        let documents_table = Table::new("documents")
            .add_column(
                Column::new("id", ColumnType::Uuid)
                    .primary_key()
            )
            .add_column(
                Column::new("project_id", ColumnType::Uuid)
                    .not_null()
                    .foreign_key(ForeignKey {
                        table: "projects".to_string(),
                        column: "id".to_string(),
                        on_delete: ForeignKeyAction::Cascade,
                        on_update: ForeignKeyAction::Cascade,
                    })
            )
            .add_column(
                Column::new("name", ColumnType::String { max_length: Some(255) })
                    .not_null()
            )
            .add_column(
                Column::new("content", ColumnType::Json)
                    .not_null()
            )
            .add_column(
                Column::new("version", ColumnType::Integer)
                    .not_null()
                    .default("1")
            )
            .add_column(
                Column::new("created_at", ColumnType::Timestamp)
                    .not_null()
            )
            .add_column(
                Column::new("updated_at", ColumnType::Timestamp)
                    .not_null()
            );

        schema.add_table(users_table).ok();
        schema.add_table(projects_table).ok();
        schema.add_table(documents_table).ok();

        schema
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_creation() {
        let col = Column::new("test", ColumnType::Integer)
            .primary_key()
            .not_null();

        assert_eq!(col.name, "test");
        assert!(col.primary_key);
        assert!(!col.nullable);
    }

    #[test]
    fn test_schema_creation() {
        let mut schema = Schema::new("test", 1);
        let table = Table::new("users")
            .add_column(Column::new("id", ColumnType::Uuid).primary_key());

        assert!(schema.add_table(table).is_ok());
        assert!(schema.get_table("users").is_some());
    }

    #[test]
    fn test_caddy_schema() {
        let schema = SchemaManager::create_caddy_schema();
        assert_eq!(schema.version, 1);
        assert!(schema.get_table("users").is_some());
        assert!(schema.get_table("projects").is_some());
        assert!(schema.get_table("documents").is_some());
    }
}
