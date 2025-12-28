//! GraphQL Persisted Queries
//!
//! Provides query registration, hash-based lookup, and Automatic Persisted
//! Queries (APQ) for improved performance and security.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;

// ============================================================================
// Error Types
// ============================================================================

/// Persisted query errors
#[derive(Error, Debug, Clone)]
pub enum PersistedQueryError {
    /// Query not found
    #[error("Persisted query not found: {0}")]
    NotFound(String),

    /// Invalid hash
    #[error("Invalid query hash: {0}")]
    InvalidHash(String),

    /// Hash mismatch
    #[error("Query hash mismatch: expected {0}, got {1}")]
    HashMismatch(String, String),

    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(String),

    /// Query too large
    #[error("Query exceeds maximum size: {0} bytes")]
    QueryTooLarge(usize),

    /// Registration disabled
    #[error("Query registration is disabled")]
    RegistrationDisabled,

    /// APQ not supported
    #[error("Automatic persisted queries not supported")]
    APQNotSupported,
}

pub type PersistedQueryResult<T> = Result<T, PersistedQueryError>;

// ============================================================================
// Configuration
// ============================================================================

/// Persisted query configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedQueryConfig {
    /// Enable persisted queries
    pub enabled: bool,
    /// Enable automatic persisted queries (APQ)
    pub enable_apq: bool,
    /// Enable query registration
    pub allow_registration: bool,
    /// Maximum query size in bytes
    pub max_query_size: usize,
    /// Cache TTL in seconds (None = permanent)
    pub cache_ttl: Option<u64>,
    /// Maximum number of cached queries
    pub max_cached_queries: usize,
    /// Hash algorithm (currently only SHA256)
    pub hash_algorithm: String,
}

impl Default for PersistedQueryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            enable_apq: true,
            allow_registration: true,
            max_query_size: 100_000, // 100KB
            cache_ttl: None,
            max_cached_queries: 10_000,
            hash_algorithm: "sha256".to_string(),
        }
    }
}

impl PersistedQueryConfig {
    /// Production configuration (strict)
    pub fn production() -> Self {
        Self {
            enabled: true,
            enable_apq: false,
            allow_registration: false,
            max_query_size: 50_000,
            cache_ttl: None,
            max_cached_queries: 5_000,
            hash_algorithm: "sha256".to_string(),
        }
    }

    /// Development configuration (permissive)
    pub fn development() -> Self {
        Self {
            enabled: true,
            enable_apq: true,
            allow_registration: true,
            max_query_size: 200_000,
            cache_ttl: Some(3600),
            max_cached_queries: 1_000,
            hash_algorithm: "sha256".to_string(),
        }
    }
}

// ============================================================================
// Query Hash
// ============================================================================

/// Query hash type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QueryHash(String);

impl QueryHash {
    /// Create a new query hash
    pub fn new(hash: impl Into<String>) -> Self {
        Self(hash.into())
    }

    /// Compute hash from query string
    pub fn from_query(query: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(query.as_bytes());
        let result = hasher.finalize();
        Self(hex::encode(result))
    }

    /// Get hash as string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Verify hash matches query
    pub fn verify(&self, query: &str) -> bool {
        let computed = Self::from_query(query);
        self == &computed
    }
}

impl std::fmt::Display for QueryHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for QueryHash {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for QueryHash {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

// ============================================================================
// Persisted Query Entry
// ============================================================================

/// Persisted query entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedQuery {
    /// Query hash
    pub hash: QueryHash,
    /// Query string
    pub query: String,
    /// Query name/identifier
    pub name: Option<String>,
    /// Query version
    pub version: Option<String>,
    /// Created timestamp
    #[serde(skip)]
    pub created_at: Option<Instant>,
    /// Last accessed timestamp
    #[serde(skip)]
    pub last_accessed: Option<Instant>,
    /// Access count
    pub access_count: u64,
}

impl PersistedQuery {
    /// Create a new persisted query
    pub fn new(query: impl Into<String>) -> Self {
        let query = query.into();
        let hash = QueryHash::from_query(&query);

        Self {
            hash,
            query,
            name: None,
            version: None,
            created_at: Some(Instant::now()),
            last_accessed: None,
            access_count: 0,
        }
    }

    /// Create with explicit hash
    pub fn with_hash(hash: QueryHash, query: impl Into<String>) -> Self {
        Self {
            hash,
            query: query.into(),
            name: None,
            version: None,
            created_at: Some(Instant::now()),
            last_accessed: None,
            access_count: 0,
        }
    }

    /// Set query name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set query version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Record access
    pub fn record_access(&mut self) {
        self.last_accessed = Some(Instant::now());
        self.access_count += 1;
    }

    /// Check if entry is expired
    pub fn is_expired(&self, ttl: Duration) -> bool {
        if let Some(created) = self.created_at {
            created.elapsed() > ttl
        } else {
            false
        }
    }
}

// ============================================================================
// Query Storage
// ============================================================================

/// Trait for persisted query storage
#[async_trait]
pub trait QueryStorage: Send + Sync {
    /// Get a query by hash
    async fn get(&self, hash: &QueryHash) -> PersistedQueryResult<Option<PersistedQuery>>;

    /// Store a query
    async fn store(&self, query: PersistedQuery) -> PersistedQueryResult<()>;

    /// Delete a query
    async fn delete(&self, hash: &QueryHash) -> PersistedQueryResult<bool>;

    /// Check if query exists
    async fn exists(&self, hash: &QueryHash) -> PersistedQueryResult<bool>;

    /// Get all queries
    async fn list(&self) -> PersistedQueryResult<Vec<PersistedQuery>>;

    /// Clear all queries
    async fn clear(&self) -> PersistedQueryResult<()>;

    /// Get storage statistics
    async fn stats(&self) -> PersistedQueryResult<StorageStats>;
}

/// In-memory query storage
pub struct InMemoryStorage {
    /// Queries by hash
    queries: Arc<RwLock<HashMap<QueryHash, PersistedQuery>>>,
    /// Configuration
    config: PersistedQueryConfig,
}

impl InMemoryStorage {
    /// Create a new in-memory storage
    pub fn new(config: PersistedQueryConfig) -> Self {
        Self {
            queries: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Evict old entries if cache is full
    async fn evict_if_needed(&self) {
        let mut queries = self.queries.write().await;

        if queries.len() >= self.config.max_cached_queries {
            // Simple LRU eviction: remove oldest accessed queries
            let mut entries: Vec<_> = queries.iter().map(|(k, v)| (k.clone(), v.last_accessed)).collect();
            entries.sort_by(|a, b| {
                let a_time = a.1.unwrap_or(Instant::now());
                let b_time = b.1.unwrap_or(Instant::now());
                a_time.cmp(&b_time)
            });

            // Remove oldest 10%
            let to_remove = self.config.max_cached_queries / 10;
            for (hash, _) in entries.iter().take(to_remove) {
                queries.remove(hash);
            }
        }
    }
}

#[async_trait]
impl QueryStorage for InMemoryStorage {
    async fn get(&self, hash: &QueryHash) -> PersistedQueryResult<Option<PersistedQuery>> {
        let mut queries = self.queries.write().await;

        if let Some(query) = queries.get_mut(hash) {
            // Record access
            query.record_access();

            // Check TTL
            if let Some(ttl) = self.config.cache_ttl {
                if query.is_expired(Duration::from_secs(ttl)) {
                    queries.remove(hash);
                    return Ok(None);
                }
            }

            Ok(Some(query.clone()))
        } else {
            Ok(None)
        }
    }

    async fn store(&self, query: PersistedQuery) -> PersistedQueryResult<()> {
        // Check query size
        if query.query.len() > self.config.max_query_size {
            return Err(PersistedQueryError::QueryTooLarge(query.query.len()));
        }

        // Evict if needed
        self.evict_if_needed().await;

        let mut queries = self.queries.write().await;
        queries.insert(query.hash.clone(), query);

        Ok(())
    }

    async fn delete(&self, hash: &QueryHash) -> PersistedQueryResult<bool> {
        let mut queries = self.queries.write().await;
        Ok(queries.remove(hash).is_some())
    }

    async fn exists(&self, hash: &QueryHash) -> PersistedQueryResult<bool> {
        let queries = self.queries.read().await;
        Ok(queries.contains_key(hash))
    }

    async fn list(&self) -> PersistedQueryResult<Vec<PersistedQuery>> {
        let queries = self.queries.read().await;
        Ok(queries.values().cloned().collect())
    }

    async fn clear(&self) -> PersistedQueryResult<()> {
        let mut queries = self.queries.write().await;
        queries.clear();
        Ok(())
    }

    async fn stats(&self) -> PersistedQueryResult<StorageStats> {
        let queries = self.queries.read().await;
        let total_size: usize = queries.values().map(|q| q.query.len()).sum();
        let total_accesses: u64 = queries.values().map(|q| q.access_count).sum();

        Ok(StorageStats {
            total_queries: queries.len(),
            total_size_bytes: total_size,
            total_accesses,
            average_query_size: if queries.is_empty() {
                0
            } else {
                total_size / queries.len()
            },
        })
    }
}

// ============================================================================
// Storage Statistics
// ============================================================================

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    /// Total number of queries
    pub total_queries: usize,
    /// Total size in bytes
    pub total_size_bytes: usize,
    /// Total access count
    pub total_accesses: u64,
    /// Average query size
    pub average_query_size: usize,
}

// ============================================================================
// Persisted Query Manager
// ============================================================================

/// Persisted query manager
pub struct PersistedQueryManager {
    /// Configuration
    config: PersistedQueryConfig,
    /// Storage backend
    storage: Arc<dyn QueryStorage>,
}

impl PersistedQueryManager {
    /// Create a new persisted query manager
    pub fn new(config: PersistedQueryConfig, storage: Arc<dyn QueryStorage>) -> Self {
        Self { config, storage }
    }

    /// Create with in-memory storage
    pub fn with_memory_storage(config: PersistedQueryConfig) -> Self {
        let storage = Arc::new(InMemoryStorage::new(config.clone()));
        Self { config, storage }
    }

    /// Register a query
    pub async fn register(
        &self,
        query: impl Into<String>,
        name: Option<String>,
        version: Option<String>,
    ) -> PersistedQueryResult<QueryHash> {
        if !self.config.allow_registration {
            return Err(PersistedQueryError::RegistrationDisabled);
        }

        let mut pq = PersistedQuery::new(query);
        if let Some(n) = name {
            pq = pq.with_name(n);
        }
        if let Some(v) = version {
            pq = pq.with_version(v);
        }

        let hash = pq.hash.clone();
        self.storage.store(pq).await?;

        Ok(hash)
    }

    /// Get a query by hash
    pub async fn get(&self, hash: &QueryHash) -> PersistedQueryResult<Option<String>> {
        if !self.config.enabled {
            return Ok(None);
        }

        Ok(self.storage.get(hash).await?.map(|pq| pq.query))
    }

    /// Handle APQ request
    pub async fn handle_apq(
        &self,
        hash: &QueryHash,
        query: Option<String>,
    ) -> PersistedQueryResult<String> {
        if !self.config.enabled {
            return Err(PersistedQueryError::APQNotSupported);
        }

        // Try to get from storage
        if let Some(stored_query) = self.storage.get(hash).await? {
            // If query provided, verify hash
            if let Some(q) = query {
                if !hash.verify(&q) {
                    return Err(PersistedQueryError::HashMismatch(
                        hash.to_string(),
                        QueryHash::from_query(&q).to_string(),
                    ));
                }
            }
            return Ok(stored_query.query);
        }

        // Query not found
        if let Some(q) = query {
            // APQ: register the query
            if self.config.enable_apq {
                // Verify hash
                if !hash.verify(&q) {
                    return Err(PersistedQueryError::HashMismatch(
                        hash.to_string(),
                        QueryHash::from_query(&q).to_string(),
                    ));
                }

                // Store for future use
                let pq = PersistedQuery::with_hash(hash.clone(), q.clone());
                self.storage.store(pq).await?;

                Ok(q)
            } else {
                Err(PersistedQueryError::NotFound(hash.to_string()))
            }
        } else {
            Err(PersistedQueryError::NotFound(hash.to_string()))
        }
    }

    /// Delete a query
    pub async fn delete(&self, hash: &QueryHash) -> PersistedQueryResult<bool> {
        self.storage.delete(hash).await
    }

    /// List all queries
    pub async fn list(&self) -> PersistedQueryResult<Vec<PersistedQuery>> {
        self.storage.list().await
    }

    /// Clear all queries
    pub async fn clear(&self) -> PersistedQueryResult<()> {
        self.storage.clear().await
    }

    /// Get statistics
    pub async fn stats(&self) -> PersistedQueryResult<StorageStats> {
        self.storage.stats().await
    }
}

// ============================================================================
// APQ Request/Response
// ============================================================================

/// APQ extension in GraphQL request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APQExtension {
    /// Protocol version
    pub version: u32,
    /// Query hash (SHA256)
    #[serde(rename = "sha256Hash")]
    pub sha256_hash: String,
}

impl APQExtension {
    /// Create a new APQ extension
    pub fn new(hash: impl Into<String>) -> Self {
        Self {
            version: 1,
            sha256_hash: hash.into(),
        }
    }

    /// Get query hash
    pub fn hash(&self) -> QueryHash {
        QueryHash::new(&self.sha256_hash)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_hash() {
        let query = "{ hello }";
        let hash1 = QueryHash::from_query(query);
        let hash2 = QueryHash::from_query(query);

        assert_eq!(hash1, hash2);
        assert!(hash1.verify(query));
        assert!(!hash1.verify("{ world }"));
    }

    #[test]
    fn test_persisted_query() {
        let query = "{ user(id: 1) { name } }";
        let pq = PersistedQuery::new(query)
            .with_name("GetUser")
            .with_version("1.0");

        assert_eq!(pq.query, query);
        assert_eq!(pq.name, Some("GetUser".to_string()));
        assert_eq!(pq.version, Some("1.0".to_string()));
        assert_eq!(pq.access_count, 0);
    }

    #[tokio::test]
    async fn test_in_memory_storage() {
        let config = PersistedQueryConfig::default();
        let storage = InMemoryStorage::new(config);

        let query = PersistedQuery::new("{ hello }");
        let hash = query.hash.clone();

        // Store
        storage.store(query).await.unwrap();

        // Get
        let retrieved = storage.get(&hash).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().query, "{ hello }");

        // Exists
        assert!(storage.exists(&hash).await.unwrap());

        // Delete
        assert!(storage.delete(&hash).await.unwrap());
        assert!(!storage.exists(&hash).await.unwrap());
    }

    #[tokio::test]
    async fn test_persisted_query_manager() {
        let config = PersistedQueryConfig::development();
        let manager = PersistedQueryManager::with_memory_storage(config);

        // Register
        let hash = manager
            .register("{ hello }", Some("HelloQuery".to_string()), None)
            .await
            .unwrap();

        // Get
        let query = manager.get(&hash).await.unwrap();
        assert_eq!(query, Some("{ hello }".to_string()));

        // Stats
        let stats = manager.stats().await.unwrap();
        assert_eq!(stats.total_queries, 1);
    }

    #[tokio::test]
    async fn test_apq_workflow() {
        let config = PersistedQueryConfig::development();
        let manager = PersistedQueryManager::with_memory_storage(config);

        let query = "{ user { id } }";
        let hash = QueryHash::from_query(query);

        // First request: query not found, provide query
        let result = manager.handle_apq(&hash, Some(query.to_string())).await;
        assert!(result.is_ok());

        // Second request: query found, no need to provide query
        let result = manager.handle_apq(&hash, None).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), query);
    }

    #[tokio::test]
    async fn test_hash_verification() {
        let config = PersistedQueryConfig::development();
        let manager = PersistedQueryManager::with_memory_storage(config);

        let query = "{ hello }";
        let correct_hash = QueryHash::from_query(query);
        let wrong_hash = QueryHash::from_query("{ world }");

        // Try to register with wrong hash
        let result = manager.handle_apq(&wrong_hash, Some(query.to_string())).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PersistedQueryError::HashMismatch(_, _)));

        // Register with correct hash
        let result = manager.handle_apq(&correct_hash, Some(query.to_string())).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_presets() {
        let prod = PersistedQueryConfig::production();
        assert!(!prod.enable_apq);
        assert!(!prod.allow_registration);

        let dev = PersistedQueryConfig::development();
        assert!(dev.enable_apq);
        assert!(dev.allow_registration);
    }

    #[test]
    fn test_apq_extension() {
        let ext = APQExtension::new("abc123");
        assert_eq!(ext.version, 1);
        assert_eq!(ext.sha256_hash, "abc123");
        assert_eq!(ext.hash().as_str(), "abc123");
    }
}
