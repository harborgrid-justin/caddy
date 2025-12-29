//! Distributed Rate Limiting
//!
//! This module provides distributed rate limiting using Redis as a backing store,
//! enabling rate limiting across multiple application instances.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::algorithm::{Decision, RateLimitResult};

/// Distributed rate limiter trait
#[async_trait]
pub trait DistributedRateLimiter: Send + Sync {
    /// Check if a request is allowed for the given key
    async fn check(&self, key: &str, tokens: u64) -> RateLimitResult<Decision>;

    /// Reset rate limit for a key
    async fn reset(&self, key: &str) -> RateLimitResult<()>;

    /// Get current count for a key
    async fn get_count(&self, key: &str) -> RateLimitResult<u64>;

    /// Set rate limit for a key
    async fn set_limit(&self, key: &str, limit: u64, window: Duration) -> RateLimitResult<()>;
}

// ============================================================================
// Redis-backed Rate Limiter
// ============================================================================

/// Redis backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    /// Redis server URLs
    pub servers: Vec<String>,
    /// Connection pool size
    pub pool_size: u32,
    /// Connection timeout
    pub timeout_ms: u64,
    /// Enable TLS
    pub use_tls: bool,
    /// Password (optional)
    pub password: Option<String>,
    /// Database number
    pub database: u8,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            servers: vec!["redis://localhost:6379".to_string()],
            pool_size: 10,
            timeout_ms: 5000,
            use_tls: false,
            password: None,
            database: 0,
        }
    }
}

/// Redis-backed distributed rate limiter
///
/// Uses Redis atomic operations and Lua scripts for consistent
/// distributed rate limiting across multiple instances.
pub struct RedisRateLimiter {
    /// Configuration
    config: RedisConfig,
    /// Consistent hash ring for sharding
    hash_ring: ConsistentHashRing,
    /// Connection pool (simulated - would use actual Redis client in production)
    connections: Arc<DashMap<String, RedisConnection>>,
    /// Default rate limit configuration
    default_limit: u64,
    /// Default window duration
    default_window: Duration,
}

impl RedisRateLimiter {
    /// Create a new Redis rate limiter
    pub fn new(config: RedisConfig, default_limit: u64, default_window: Duration) -> Self {
        let hash_ring = ConsistentHashRing::new(config.servers.clone(), 150);

        Self {
            config,
            hash_ring,
            connections: Arc::new(DashMap::new()),
            default_limit,
            default_window,
        }
    }

    /// Get or create connection to appropriate Redis server
    async fn get_connection(&self, key: &str) -> RateLimitResult<Arc<RedisConnection>> {
        let server = self.hash_ring.get_node(key);

        if let Some(conn) = self.connections.get(server) {
            return Ok(Arc::new(conn.clone()));
        }

        // Create new connection
        let conn = RedisConnection::new(server, &self.config).await?;
        self.connections.insert(server.to_string(), conn.clone());

        Ok(Arc::new(conn))
    }

    /// Execute token bucket algorithm in Redis using Lua script
    async fn redis_token_bucket(
        &self,
        key: &str,
        tokens: u64,
        limit: u64,
        window: Duration,
    ) -> RateLimitResult<Decision> {
        let conn = self.get_connection(key).await?;

        // Lua script for atomic token bucket operation
        let script = r#"
            local key = KEYS[1]
            local tokens_key = key .. ":tokens"
            local timestamp_key = key .. ":timestamp"

            local tokens_requested = tonumber(ARGV[1])
            local capacity = tonumber(ARGV[2])
            local refill_rate = tonumber(ARGV[3])
            local now = tonumber(ARGV[4])

            -- Get current state
            local tokens = tonumber(redis.call('GET', tokens_key) or capacity)
            local last_refill = tonumber(redis.call('GET', timestamp_key) or now)

            -- Refill tokens based on elapsed time
            local elapsed = math.max(0, now - last_refill)
            local tokens_to_add = math.floor(elapsed * refill_rate / 1000000000)
            tokens = math.min(capacity, tokens + tokens_to_add)

            -- Try to consume tokens
            if tokens >= tokens_requested then
                tokens = tokens - tokens_requested
                redis.call('SET', tokens_key, tokens)
                redis.call('SET', timestamp_key, now)
                redis.call('EXPIRE', tokens_key, 3600)
                redis.call('EXPIRE', timestamp_key, 3600)
                return {1, tokens}  -- allowed, remaining
            else
                return {0, tokens}  -- denied, current
            end
        "#;

        let now = Self::now_nanos();
        let refill_rate = limit as f64 / window.as_secs() as f64;

        let result = conn
            .eval_script(
                script,
                vec![key],
                vec![
                    tokens.to_string(),
                    limit.to_string(),
                    refill_rate.to_string(),
                    now.to_string(),
                ],
            )
            .await?;

        if result[0] == 1 {
            Ok(Decision::Allowed {
                remaining: result[1],
                reset_after: window.as_secs(),
            })
        } else {
            let retry_after = ((limit - result[1]) as f64 / refill_rate) as u64;
            Ok(Decision::Denied {
                retry_after: retry_after.max(1),
                limit,
            })
        }
    }

    /// Get current nanoseconds since UNIX epoch
    fn now_nanos() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }
}

#[async_trait]
impl DistributedRateLimiter for RedisRateLimiter {
    async fn check(&self, key: &str, tokens: u64) -> RateLimitResult<Decision> {
        self.redis_token_bucket(key, tokens, self.default_limit, self.default_window)
            .await
    }

    async fn reset(&self, key: &str) -> RateLimitResult<()> {
        let conn = self.get_connection(key).await?;
        conn.delete(&[
            &format!("{}:tokens", key),
            &format!("{}:timestamp", key),
        ])
        .await?;
        Ok(())
    }

    async fn get_count(&self, key: &str) -> RateLimitResult<u64> {
        let conn = self.get_connection(key).await?;
        let tokens = conn.get(&format!("{}:tokens", key)).await?;
        Ok(self.default_limit.saturating_sub(tokens))
    }

    async fn set_limit(&self, key: &str, limit: u64, window: Duration) -> RateLimitResult<()> {
        let conn = self.get_connection(key).await?;
        conn.set(&format!("{}:limit", key), limit, Some(window))
            .await?;
        Ok(())
    }
}

// ============================================================================
// Consistent Hashing
// ============================================================================

/// Consistent hash ring for distributing keys across Redis servers
#[derive(Debug, Clone)]
pub struct ConsistentHashRing {
    /// Virtual nodes on the ring
    ring: Vec<(u64, String)>,
    /// Number of virtual nodes per server
    replicas: usize,
}

impl ConsistentHashRing {
    /// Create a new consistent hash ring
    ///
    /// # Arguments
    /// * `nodes` - List of server addresses
    /// * `replicas` - Number of virtual nodes per server (higher = better distribution)
    pub fn new(nodes: Vec<String>, replicas: usize) -> Self {
        let mut ring = Vec::new();

        for node in &nodes {
            for i in 0..replicas {
                let virtual_node = format!("{}:{}", node, i);
                let hash = Self::hash_string(&virtual_node);
                ring.push((hash, node.clone()));
            }
        }

        ring.sort_by_key(|(hash, _)| *hash);

        Self { ring, replicas }
    }

    /// Get the node responsible for a key
    pub fn get_node(&self, key: &str) -> &str {
        if self.ring.is_empty() {
            return "";
        }

        let hash = Self::hash_string(key);

        // Binary search for the first node with hash >= key hash
        match self.ring.binary_search_by_key(&hash, |(h, _)| *h) {
            Ok(idx) => &self.ring[idx].1,
            Err(idx) => {
                if idx >= self.ring.len() {
                    // Wrap around to first node
                    &self.ring[0].1
                } else {
                    &self.ring[idx].1
                }
            }
        }
    }

    /// Add a node to the ring
    pub fn add_node(&mut self, node: String) {
        for i in 0..self.replicas {
            let virtual_node = format!("{}:{}", node, i);
            let hash = Self::hash_string(&virtual_node);
            self.ring.push((hash, node.clone()));
        }
        self.ring.sort_by_key(|(hash, _)| *hash);
    }

    /// Remove a node from the ring
    pub fn remove_node(&mut self, node: &str) {
        self.ring.retain(|(_, n)| n != node);
    }

    /// Hash a string to u64
    fn hash_string(s: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }

    /// Get all nodes
    pub fn nodes(&self) -> Vec<String> {
        let mut nodes: Vec<String> = self
            .ring
            .iter()
            .map(|(_, node)| node.clone())
            .collect();
        nodes.sort();
        nodes.dedup();
        nodes
    }
}

// ============================================================================
// Redis Connection (Simulated)
// ============================================================================

/// Simulated Redis connection
///
/// In production, this would use an actual Redis client library like redis-rs
/// or fred. This is a mock implementation for demonstration.
#[derive(Debug, Clone)]
pub struct RedisConnection {
    /// Server address
    server: String,
    /// Local cache for simulation
    cache: Arc<DashMap<String, (u64, SystemTime)>>,
}

impl RedisConnection {
    /// Create a new Redis connection
    async fn new(server: &str, _config: &RedisConfig) -> RateLimitResult<Self> {
        // In production: actually connect to Redis
        Ok(Self {
            server: server.to_string(),
            cache: Arc::new(DashMap::new()),
        })
    }

    /// Execute a Lua script
    async fn eval_script(
        &self,
        _script: &str,
        keys: Vec<&str>,
        args: Vec<String>,
    ) -> RateLimitResult<Vec<u64>> {
        // Simulated token bucket logic
        let key = keys[0];
        let tokens_requested: u64 = args[0].parse().unwrap();
        let capacity: u64 = args[1].parse().unwrap();
        let refill_rate: f64 = args[2].parse().unwrap();
        let now: u64 = args[3].parse().unwrap();

        let tokens_key = format!("{}:tokens", key);
        let timestamp_key = format!("{}:timestamp", key);

        // Get or initialize tokens
        let mut tokens = capacity;
        let mut last_refill = now;

        if let Some(entry) = self.cache.get(&tokens_key) {
            tokens = entry.0;
        }
        if let Some(entry) = self.cache.get(&timestamp_key) {
            last_refill = entry.0;
        }

        // Refill
        let elapsed = now.saturating_sub(last_refill);
        let tokens_to_add = ((elapsed as f64 * refill_rate) / 1_000_000_000.0) as u64;
        tokens = std::cmp::min(capacity, tokens + tokens_to_add);

        // Try to consume
        if tokens >= tokens_requested {
            tokens -= tokens_requested;
            self.cache.insert(
                tokens_key,
                (tokens, SystemTime::now() + Duration::from_secs(3600)),
            );
            self.cache.insert(
                timestamp_key,
                (now, SystemTime::now() + Duration::from_secs(3600)),
            );
            Ok(vec![1, tokens])
        } else {
            Ok(vec![0, tokens])
        }
    }

    /// Get a value
    async fn get(&self, key: &str) -> RateLimitResult<u64> {
        Ok(self.cache.get(key).map(|e| e.0).unwrap_or(0))
    }

    /// Set a value
    async fn set(&self, key: &str, value: u64, ttl: Option<Duration>) -> RateLimitResult<()> {
        let expiry = ttl
            .map(|d| SystemTime::now() + d)
            .unwrap_or(SystemTime::now() + Duration::from_secs(3600));
        self.cache.insert(key.to_string(), (value, expiry));
        Ok(())
    }

    /// Delete keys
    async fn delete(&self, keys: &[&str]) -> RateLimitResult<()> {
        for key in keys {
            self.cache.remove(*key);
        }
        Ok(())
    }
}

// ============================================================================
// Synchronization Protocols
// ============================================================================

/// Distributed lock for race condition handling
pub struct DistributedLock {
    /// Redis connection
    conn: Arc<RedisConnection>,
    /// Lock key
    key: String,
    /// Lock timeout
    timeout: Duration,
    /// Lock token (for ownership verification)
    token: String,
}

impl DistributedLock {
    /// Acquire a distributed lock
    pub async fn acquire(
        conn: Arc<RedisConnection>,
        key: String,
        timeout: Duration,
    ) -> RateLimitResult<Option<Self>> {
        let token = uuid::Uuid::new_v4().to_string();

        // Try to set lock with NX (only if not exists)
        // In production: use Redis SET key value NX EX timeout
        let lock_key = format!("lock:{}", key);

        if conn.cache.get(&lock_key).is_none() {
            conn.set(&lock_key, 1, Some(timeout)).await?;

            Ok(Some(Self {
                conn,
                key: lock_key,
                timeout,
                token,
            }))
        } else {
            Ok(None)
        }
    }

    /// Release the lock
    pub async fn release(self) -> RateLimitResult<()> {
        // In production: use Lua script to verify token before deleting
        self.conn.delete(&[&self.key]).await?;
        Ok(())
    }

    /// Extend lock timeout
    pub async fn extend(&self, additional_time: Duration) -> RateLimitResult<()> {
        let new_timeout = self.timeout + additional_time;
        self.conn.set(&self.key, 1, Some(new_timeout)).await?;
        Ok(())
    }
}

// ============================================================================
// Race Condition Handling
// ============================================================================

/// Optimistic locking with versioning
pub struct OptimisticLock {
    /// Redis connection
    conn: Arc<RedisConnection>,
    /// Lock key
    key: String,
    /// Current version
    version: Arc<RwLock<u64>>,
}

impl OptimisticLock {
    /// Create a new optimistic lock
    pub fn new(conn: Arc<RedisConnection>, key: String) -> Self {
        Self {
            conn,
            key,
            version: Arc::new(RwLock::new(0)),
        }
    }

    /// Read current value and version
    pub async fn read(&self) -> RateLimitResult<(u64, u64)> {
        let version_key = format!("{}:version", self.key);
        let value = self.conn.get(&self.key).await?;
        let version = self.conn.get(&version_key).await?;

        *self.version.write().await = version;

        Ok((value, version))
    }

    /// Try to write with version check
    pub async fn write(&self, value: u64) -> RateLimitResult<bool> {
        let version_key = format!("{}:version", self.key);
        let current_version = *self.version.read().await;
        let stored_version = self.conn.get(&version_key).await?;

        if stored_version != current_version {
            // Version mismatch - someone else modified it
            return Ok(false);
        }

        // Write new value and increment version
        self.conn.set(&self.key, value, None).await?;
        self.conn
            .set(&version_key, current_version + 1, None)
            .await?;

        *self.version.write().await = current_version + 1;

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistent_hash_ring() {
        let nodes = vec![
            "server1:6379".to_string(),
            "server2:6379".to_string(),
            "server3:6379".to_string(),
        ];

        let ring = ConsistentHashRing::new(nodes.clone(), 150);

        // Same key should always map to same node
        let node1 = ring.get_node("test_key");
        let node2 = ring.get_node("test_key");
        assert_eq!(node1, node2);

        // Different keys may map to different nodes
        let _node3 = ring.get_node("another_key");
    }

    #[test]
    fn test_consistent_hash_ring_add_remove() {
        let nodes = vec!["server1:6379".to_string()];
        let mut ring = ConsistentHashRing::new(nodes, 150);

        // Add node
        ring.add_node("server2:6379".to_string());
        assert_eq!(ring.nodes().len(), 2);

        // Remove node
        ring.remove_node("server1:6379");
        assert_eq!(ring.nodes().len(), 1);
    }

    #[tokio::test]
    async fn test_redis_connection_basic() {
        let config = RedisConfig::default();
        let conn = RedisConnection::new("redis://localhost:6379", &config)
            .await
            .unwrap();

        conn.set("test_key", 42, Some(Duration::from_secs(60)))
            .await
            .unwrap();

        let value = conn.get("test_key").await.unwrap();
        assert_eq!(value, 42);
    }

    #[tokio::test]
    async fn test_redis_rate_limiter() {
        let config = RedisConfig::default();
        let limiter = RedisRateLimiter::new(config, 10, Duration::from_secs(60));

        // Should allow first requests
        for _ in 0..10 {
            let decision = limiter.check("test_user", 1).await.unwrap();
            assert!(decision.is_allowed());
        }

        // Should deny after limit
        let decision = limiter.check("test_user", 1).await.unwrap();
        assert!(!decision.is_allowed());
    }

    #[tokio::test]
    async fn test_distributed_lock() {
        let config = RedisConfig::default();
        let conn = Arc::new(
            RedisConnection::new("redis://localhost:6379", &config)
                .await
                .unwrap(),
        );

        let lock1 = DistributedLock::acquire(
            conn.clone(),
            "test_lock".to_string(),
            Duration::from_secs(10),
        )
        .await
        .unwrap();

        assert!(lock1.is_some());

        // Second acquire should fail
        let lock2 = DistributedLock::acquire(
            conn.clone(),
            "test_lock".to_string(),
            Duration::from_secs(10),
        )
        .await
        .unwrap();

        assert!(lock2.is_none());

        // Release and try again
        lock1.unwrap().release().await.unwrap();

        let lock3 = DistributedLock::acquire(conn, "test_lock".to_string(), Duration::from_secs(10))
            .await
            .unwrap();

        assert!(lock3.is_some());
    }

    #[tokio::test]
    async fn test_optimistic_lock() {
        let config = RedisConfig::default();
        let conn = Arc::new(
            RedisConnection::new("redis://localhost:6379", &config)
                .await
                .unwrap(),
        );

        let lock = OptimisticLock::new(conn, "test_counter".to_string());

        // First write should succeed
        assert!(lock.write(100).await.unwrap());

        // Read and write again
        let (_value, _version) = lock.read().await.unwrap();
        assert!(lock.write(200).await.unwrap());
    }
}
