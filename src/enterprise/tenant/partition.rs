//! Data Partitioning and Isolation
//!
//! Implements schema-based isolation, row-level security, encryption key separation,
//! and cross-tenant query prevention.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use sha2::{Sha256, Digest};

use super::context::{TenantId, get_tenant_id};

/// Data partitioning errors
#[derive(Error, Debug)]
pub enum PartitionError {
    #[error("Cross-tenant access violation: attempting to access {target} from {current}")]
    CrossTenantViolation { current: String, target: String },

    #[error("Schema not found for tenant: {0}")]
    SchemaNotFound(String),

    #[error("Encryption key not found for tenant: {0}")]
    KeyNotFound(String),

    #[error("Data decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Invalid partition configuration: {0}")]
    InvalidConfig(String),

    #[error("Row-level security policy violation: {0}")]
    RlsPolicyViolation(String),
}

pub type PartitionResult<T> = Result<T, PartitionError>;

/// Partitioning strategy for multi-tenant data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PartitionStrategy {
    /// Separate database per tenant
    SeparateDatabase,
    /// Separate schema per tenant in shared database
    SeparateSchema,
    /// Shared schema with tenant_id column (discriminator)
    SharedSchema,
    /// Hybrid: large tenants get separate schema, small ones share
    Hybrid { threshold_users: u32 },
}

/// Schema naming strategy
#[derive(Debug, Clone)]
pub struct SchemaConfig {
    /// Base prefix for tenant schemas
    pub prefix: String,
    /// Strategy for partitioning
    pub strategy: PartitionStrategy,
}

impl Default for SchemaConfig {
    fn default() -> Self {
        Self {
            prefix: "tenant".to_string(),
            strategy: PartitionStrategy::SeparateSchema,
        }
    }
}

impl SchemaConfig {
    /// Get schema name for a tenant
    pub fn schema_name(&self, tenant_id: &TenantId) -> String {
        match &self.strategy {
            PartitionStrategy::SeparateDatabase => {
                format!("{}_{}", self.prefix, tenant_id.org_id)
            }
            PartitionStrategy::SeparateSchema => {
                let hash = self.hash_tenant_id(tenant_id);
                format!("{}_{}", self.prefix, hash)
            }
            PartitionStrategy::SharedSchema => "public".to_string(),
            PartitionStrategy::Hybrid { .. } => {
                // In real implementation, would check tenant size
                format!("{}_{}", self.prefix, tenant_id.org_id)
            }
        }
    }

    /// Hash tenant ID for schema naming
    fn hash_tenant_id(&self, tenant_id: &TenantId) -> String {
        let mut hasher = Sha256::new();
        hasher.update(tenant_id.to_string().as_bytes());
        let result = hasher.finalize();
        hex::encode(&result[..8]) // Use first 8 bytes
    }
}

/// Row-level security policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RlsPolicy {
    /// Policy name
    pub name: String,
    /// Table this policy applies to
    pub table: String,
    /// SQL condition for filtering rows
    pub condition: String,
    /// Operations this policy applies to (SELECT, INSERT, UPDATE, DELETE)
    pub operations: Vec<String>,
}

/// Data partitioning manager
pub struct DataPartitionManager {
    config: SchemaConfig,
    /// Tenant-specific encryption keys
    encryption_keys: Arc<RwLock<HashMap<TenantId, Vec<u8>>>>,
    /// RLS policies per tenant
    rls_policies: Arc<RwLock<HashMap<TenantId, Vec<RlsPolicy>>>>,
    /// Schema assignments
    schema_map: Arc<RwLock<HashMap<TenantId, String>>>,
}

impl DataPartitionManager {
    /// Create a new data partition manager
    pub fn new(config: SchemaConfig) -> Self {
        Self {
            config,
            encryption_keys: Arc::new(RwLock::new(HashMap::new())),
            rls_policies: Arc::new(RwLock::new(HashMap::new())),
            schema_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize partition for a new tenant
    pub fn initialize_tenant(&self, tenant_id: TenantId) -> PartitionResult<TenantPartitionInfo> {
        let schema_name = self.config.schema_name(&tenant_id);

        // Generate encryption key
        let encryption_key = self.generate_encryption_key();

        // Store schema mapping
        self.schema_map.write().insert(tenant_id.clone(), schema_name.clone());

        // Store encryption key
        self.encryption_keys.write().insert(tenant_id.clone(), encryption_key.clone());

        Ok(TenantPartitionInfo {
            tenant_id: tenant_id.clone(),
            schema_name,
            has_encryption_key: true,
            partition_strategy: self.config.strategy.clone(),
        })
    }

    /// Get schema name for a tenant
    pub fn get_schema(&self, tenant_id: &TenantId) -> PartitionResult<String> {
        self.schema_map
            .read()
            .get(tenant_id)
            .cloned()
            .ok_or_else(|| PartitionError::SchemaNotFound(tenant_id.to_string()))
    }

    /// Get current tenant's schema (from context)
    pub fn current_schema() -> PartitionResult<String> {
        let tenant_id = get_tenant_id()
            .map_err(|e| PartitionError::InvalidConfig(e.to_string()))?;

        // In a real implementation, would look up from manager instance
        // For now, generate based on tenant ID
        Ok(format!("tenant_{}", tenant_id.org_id))
    }

    /// Validate that data access is within same tenant
    pub fn validate_access(&self, target_tenant: &TenantId) -> PartitionResult<()> {
        let current_tenant = get_tenant_id()
            .map_err(|e| PartitionError::InvalidConfig(e.to_string()))?;

        // Allow access if current tenant is ancestor (org can access workspace data)
        if current_tenant.is_ancestor_of(target_tenant) {
            return Ok(());
        }

        // Allow access if same tenant or descendant
        if target_tenant.is_ancestor_of(&current_tenant) || current_tenant == *target_tenant {
            return Ok(());
        }

        Err(PartitionError::CrossTenantViolation {
            current: current_tenant.to_string(),
            target: target_tenant.to_string(),
        })
    }

    /// Get encryption key for a tenant
    pub fn get_encryption_key(&self, tenant_id: &TenantId) -> PartitionResult<Vec<u8>> {
        self.encryption_keys
            .read()
            .get(tenant_id)
            .cloned()
            .ok_or_else(|| PartitionError::KeyNotFound(tenant_id.to_string()))
    }

    /// Encrypt data for a tenant
    pub fn encrypt_data(&self, tenant_id: &TenantId, data: &[u8]) -> PartitionResult<Vec<u8>> {
        let key = self.get_encryption_key(tenant_id)?;

        // In production, use proper AEAD encryption (AES-GCM)
        // This is a simplified example
        let encrypted = data.iter()
            .zip(key.iter().cycle())
            .map(|(d, k)| d ^ k)
            .collect();

        Ok(encrypted)
    }

    /// Decrypt data for a tenant
    pub fn decrypt_data(&self, tenant_id: &TenantId, encrypted: &[u8]) -> PartitionResult<Vec<u8>> {
        let key = self.get_encryption_key(tenant_id)?;

        // XOR is symmetric, so decryption is same as encryption
        let decrypted = encrypted.iter()
            .zip(key.iter().cycle())
            .map(|(e, k)| e ^ k)
            .collect();

        Ok(decrypted)
    }

    /// Add RLS policy for a tenant
    pub fn add_rls_policy(&self, tenant_id: TenantId, policy: RlsPolicy) {
        self.rls_policies
            .write()
            .entry(tenant_id)
            .or_insert_with(Vec::new)
            .push(policy);
    }

    /// Get RLS policies for a tenant
    pub fn get_rls_policies(&self, tenant_id: &TenantId) -> Vec<RlsPolicy> {
        self.rls_policies
            .read()
            .get(tenant_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Generate SQL query with tenant isolation
    pub fn isolate_query(&self, base_query: &str, tenant_id: &TenantId) -> PartitionResult<String> {
        let schema = self.get_schema(tenant_id)?;

        match self.config.strategy {
            PartitionStrategy::SeparateSchema => {
                // Prefix table names with schema
                Ok(format!("SET search_path TO {}; {}", schema, base_query))
            }
            PartitionStrategy::SharedSchema => {
                // Add tenant_id filter to WHERE clause
                self.add_tenant_filter(base_query, tenant_id)
            }
            _ => Ok(base_query.to_string()),
        }
    }

    /// Add tenant_id filter to SQL query
    fn add_tenant_filter(&self, query: &str, tenant_id: &TenantId) -> PartitionResult<String> {
        // Simple injection of tenant filter
        // In production, use a proper SQL parser
        let tenant_filter = format!("tenant_id = '{}'", tenant_id.org_id);

        if query.to_lowercase().contains("where") {
            Ok(query.replace("WHERE", &format!("WHERE {} AND", tenant_filter)))
        } else if query.to_lowercase().contains("from") {
            Ok(query.replace("FROM", &format!("FROM")))
                .map(|q| format!("{} WHERE {}", q, tenant_filter))
        } else {
            Ok(query.to_string())
        }
    }

    /// Generate a new encryption key
    fn generate_encryption_key(&self) -> Vec<u8> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..32).map(|_| rng.gen()).collect()
    }

    /// Remove tenant partition
    pub fn remove_tenant(&self, tenant_id: &TenantId) {
        self.schema_map.write().remove(tenant_id);
        self.encryption_keys.write().remove(tenant_id);
        self.rls_policies.write().remove(tenant_id);
    }
}

/// Information about a tenant's data partition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantPartitionInfo {
    pub tenant_id: TenantId,
    pub schema_name: String,
    pub has_encryption_key: bool,
    pub partition_strategy: PartitionStrategy,
}

/// Query builder with automatic tenant isolation
pub struct TenantQuery {
    base_query: String,
    tenant_id: TenantId,
    parameters: HashMap<String, String>,
}

impl TenantQuery {
    /// Create a new tenant-isolated query
    pub fn new(base_query: impl Into<String>, tenant_id: TenantId) -> Self {
        Self {
            base_query: base_query.into(),
            tenant_id,
            parameters: HashMap::new(),
        }
    }

    /// Add a parameter
    pub fn param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }

    /// Build the final query with tenant isolation
    pub fn build(self, manager: &DataPartitionManager) -> PartitionResult<String> {
        manager.isolate_query(&self.base_query, &self.tenant_id)
    }
}

/// Tenant-aware data access object
pub struct TenantDao<T> {
    tenant_id: TenantId,
    table_name: String,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> TenantDao<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    /// Create a new tenant DAO
    pub fn new(tenant_id: TenantId, table_name: impl Into<String>) -> Self {
        Self {
            tenant_id,
            table_name: table_name.into(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get the table name with schema prefix
    pub fn qualified_table_name(&self, manager: &DataPartitionManager) -> PartitionResult<String> {
        let schema = manager.get_schema(&self.tenant_id)?;
        Ok(format!("{}.{}", schema, self.table_name))
    }

    /// Build a SELECT query
    pub fn select(&self) -> TenantQuery {
        let query = format!("SELECT * FROM {}", self.table_name);
        TenantQuery::new(query, self.tenant_id.clone())
    }

    /// Build an INSERT query
    pub fn insert(&self, _data: &T) -> TenantQuery {
        let query = format!("INSERT INTO {} VALUES (?)", self.table_name);
        TenantQuery::new(query, self.tenant_id.clone())
    }

    /// Build an UPDATE query
    pub fn update(&self, _data: &T) -> TenantQuery {
        let query = format!("UPDATE {} SET ? WHERE id = ?", self.table_name);
        TenantQuery::new(query, self.tenant_id.clone())
    }

    /// Build a DELETE query
    pub fn delete(&self, id: &str) -> TenantQuery {
        let query = format!("DELETE FROM {} WHERE id = '{}'", self.table_name, id);
        TenantQuery::new(query, self.tenant_id.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_schema_naming() {
        let config = SchemaConfig::default();
        let tenant_id = TenantId::new_org(Uuid::new_v4());

        let schema_name = config.schema_name(&tenant_id);
        assert!(schema_name.starts_with("tenant_"));
    }

    #[test]
    fn test_tenant_initialization() {
        let config = SchemaConfig::default();
        let manager = DataPartitionManager::new(config);
        let tenant_id = TenantId::new_org(Uuid::new_v4());

        let info = manager.initialize_tenant(tenant_id.clone()).unwrap();
        assert_eq!(info.tenant_id, tenant_id);
        assert!(info.has_encryption_key);

        // Should be able to retrieve schema
        let schema = manager.get_schema(&tenant_id).unwrap();
        assert_eq!(schema, info.schema_name);
    }

    #[test]
    fn test_encryption_decryption() {
        let config = SchemaConfig::default();
        let manager = DataPartitionManager::new(config);
        let tenant_id = TenantId::new_org(Uuid::new_v4());

        manager.initialize_tenant(tenant_id.clone()).unwrap();

        let data = b"sensitive data";
        let encrypted = manager.encrypt_data(&tenant_id, data).unwrap();
        assert_ne!(encrypted, data);

        let decrypted = manager.decrypt_data(&tenant_id, &encrypted).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_rls_policies() {
        let config = SchemaConfig::default();
        let manager = DataPartitionManager::new(config);
        let tenant_id = TenantId::new_org(Uuid::new_v4());

        let policy = RlsPolicy {
            name: "user_access".to_string(),
            table: "documents".to_string(),
            condition: "user_id = current_user_id()".to_string(),
            operations: vec!["SELECT".to_string(), "UPDATE".to_string()],
        };

        manager.add_rls_policy(tenant_id.clone(), policy.clone());

        let policies = manager.get_rls_policies(&tenant_id);
        assert_eq!(policies.len(), 1);
        assert_eq!(policies[0].name, "user_access");
    }

    #[test]
    fn test_query_isolation() {
        let config = SchemaConfig {
            strategy: PartitionStrategy::SeparateSchema,
            ..Default::default()
        };
        let manager = DataPartitionManager::new(config);
        let tenant_id = TenantId::new_org(Uuid::new_v4());

        manager.initialize_tenant(tenant_id.clone()).unwrap();

        let query = "SELECT * FROM users";
        let isolated = manager.isolate_query(query, &tenant_id).unwrap();
        assert!(isolated.contains("SET search_path"));
    }

    #[test]
    fn test_tenant_dao() {
        #[derive(Serialize, Deserialize)]
        struct User {
            id: String,
            name: String,
        }

        let tenant_id = TenantId::new_org(Uuid::new_v4());
        let dao = TenantDao::<User>::new(tenant_id.clone(), "users");

        let query = dao.select();
        assert_eq!(query.tenant_id, tenant_id);
    }

    #[test]
    fn test_separate_encryption_keys() {
        let config = SchemaConfig::default();
        let manager = DataPartitionManager::new(config);

        let tenant1 = TenantId::new_org(Uuid::new_v4());
        let tenant2 = TenantId::new_org(Uuid::new_v4());

        manager.initialize_tenant(tenant1.clone()).unwrap();
        manager.initialize_tenant(tenant2.clone()).unwrap();

        let key1 = manager.get_encryption_key(&tenant1).unwrap();
        let key2 = manager.get_encryption_key(&tenant2).unwrap();

        // Keys should be different
        assert_ne!(key1, key2);
    }
}
