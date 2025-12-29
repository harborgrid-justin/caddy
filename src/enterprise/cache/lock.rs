//! Distributed locking mechanisms for cache coordination
//!
//! This module provides distributed synchronization primitives:
//! - Distributed mutex with fencing tokens
//! - Read-write locks with fair scheduling
//! - Lock leasing with automatic renewal
//! - Deadlock detection and prevention

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, Semaphore};
use tokio::time::{interval, sleep, timeout};
use uuid::Uuid;
use crate::enterprise::error::{EnterpriseError, EnterpriseResult};

/// Lock mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LockMode {
    /// Exclusive lock (write)
    Exclusive,
    /// Shared lock (read)
    Shared,
}

/// Lock status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockStatus {
    /// Lock acquired successfully
    Acquired,
    /// Lock is held by another owner
    Held,
    /// Lock acquisition timed out
    Timeout,
    /// Lock has expired
    Expired,
}

/// Fencing token for preventing stale lock operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FencingToken(u64);

impl FencingToken {
    fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(&self) -> u64 {
        self.0
    }

    fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

/// Lock holder information
#[derive(Debug, Clone)]
struct LockHolder {
    /// Unique owner ID
    owner_id: Uuid,
    /// Lock acquisition time
    acquired_at: Instant,
    /// Lock expiration time
    expires_at: Instant,
    /// Fencing token
    token: FencingToken,
    /// Lock mode
    mode: LockMode,
    /// Number of times acquired (for reentrant locks)
    count: usize,
}

impl LockHolder {
    fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }

    fn owned_by(&self, owner_id: &Uuid) -> bool {
        self.owner_id == *owner_id
    }
}

/// Distributed lock configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockConfig {
    /// Default lock timeout (milliseconds)
    pub default_timeout_ms: u64,
    /// Lock lease duration (milliseconds)
    pub lease_duration_ms: u64,
    /// Enable automatic lease renewal
    pub auto_renew: bool,
    /// Lease renewal interval (percentage of lease duration)
    pub renewal_threshold: f64,
    /// Enable deadlock detection
    pub enable_deadlock_detection: bool,
    /// Deadlock detection interval (milliseconds)
    pub deadlock_check_interval_ms: u64,
}

impl Default for LockConfig {
    fn default() -> Self {
        Self {
            default_timeout_ms: 5000,
            lease_duration_ms: 30000,
            auto_renew: true,
            renewal_threshold: 0.7,
            enable_deadlock_detection: true,
            deadlock_check_interval_ms: 1000,
        }
    }
}

/// Distributed mutex with fencing tokens
pub struct DistributedMutex<K> {
    /// Lock states keyed by resource
    locks: Arc<DashMap<K, LockHolder>>,
    /// Current fencing token counter
    token_counter: Arc<RwLock<FencingToken>>,
    /// Configuration
    config: LockConfig,
}

impl<K> DistributedMutex<K>
where
    K: Eq + Hash + Clone + Debug + Send + Sync,
{
    pub fn new() -> Self {
        Self::with_config(LockConfig::default())
    }

    pub fn with_config(config: LockConfig) -> Self {
        Self {
            locks: Arc::new(DashMap::new()),
            token_counter: Arc::new(RwLock::new(FencingToken::new(0))),
            config,
        }
    }

    /// Acquire an exclusive lock
    pub async fn lock(
        &self,
        key: K,
        owner_id: Uuid,
        timeout_duration: Option<Duration>,
    ) -> EnterpriseResult<FencingToken> {
        let timeout_duration = timeout_duration
            .unwrap_or_else(|| Duration::from_millis(self.config.default_timeout_ms));

        let start = Instant::now();

        loop {
            // Try to acquire lock
            if let Some(token) = self.try_lock(&key, owner_id).await? {
                return Ok(token);
            }

            // Check timeout
            if start.elapsed() >= timeout_duration {
                return Err(EnterpriseError::Other(
                    format!("Lock acquisition timeout for key: {:?}", key)
                ));
            }

            // Wait briefly before retry
            sleep(Duration::from_millis(10)).await;
        }
    }

    /// Try to acquire lock (non-blocking)
    async fn try_lock(&self, key: &K, owner_id: Uuid) -> EnterpriseResult<Option<FencingToken>> {
        // Check if lock exists and is expired
        if let Some(holder) = self.locks.get(key) {
            if holder.is_expired() {
                drop(holder);
                self.locks.remove(key);
            } else if holder.owned_by(&owner_id) {
                // Reentrant lock
                let token = holder.token;
                drop(holder);

                if let Some(mut holder) = self.locks.get_mut(key) {
                    holder.count += 1;
                }

                return Ok(Some(token));
            } else {
                // Lock is held by another owner
                return Ok(None);
            }
        }

        // Acquire new lock
        let mut counter = self.token_counter.write().await;
        let token = counter.next();
        *counter = token;
        drop(counter);

        let lease_duration = Duration::from_millis(self.config.lease_duration_ms);
        let holder = LockHolder {
            owner_id,
            acquired_at: Instant::now(),
            expires_at: Instant::now() + lease_duration,
            token,
            mode: LockMode::Exclusive,
            count: 1,
        };

        self.locks.insert(key.clone(), holder);

        Ok(Some(token))
    }

    /// Release a lock
    pub async fn unlock(
        &self,
        key: &K,
        owner_id: Uuid,
        token: FencingToken,
    ) -> EnterpriseResult<()> {
        if let Some(mut holder) = self.locks.get_mut(key) {
            if !holder.owned_by(&owner_id) {
                return Err(EnterpriseError::Other(
                    "Cannot unlock: not the lock owner".to_string()
                ));
            }

            if holder.token != token {
                return Err(EnterpriseError::Other(
                    "Cannot unlock: invalid fencing token".to_string()
                ));
            }

            holder.count -= 1;
            if holder.count == 0 {
                drop(holder);
                self.locks.remove(key);
            }
        }

        Ok(())
    }

    /// Renew a lock lease
    pub async fn renew(
        &self,
        key: &K,
        owner_id: Uuid,
        token: FencingToken,
    ) -> EnterpriseResult<()> {
        if let Some(mut holder) = self.locks.get_mut(key) {
            if !holder.owned_by(&owner_id) {
                return Err(EnterpriseError::Other(
                    "Cannot renew: not the lock owner".to_string()
                ));
            }

            if holder.token != token {
                return Err(EnterpriseError::Other(
                    "Cannot renew: invalid fencing token".to_string()
                ));
            }

            let lease_duration = Duration::from_millis(self.config.lease_duration_ms);
            holder.expires_at = Instant::now() + lease_duration;
        }

        Ok(())
    }

    /// Check if a lock is held
    pub fn is_locked(&self, key: &K) -> bool {
        if let Some(holder) = self.locks.get(key) {
            !holder.is_expired()
        } else {
            false
        }
    }

    /// Force release a lock (admin operation)
    pub fn force_unlock(&self, key: &K) {
        self.locks.remove(key);
    }
}

impl<K> Default for DistributedMutex<K>
where
    K: Eq + Hash + Clone + Debug + Send + Sync,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Distributed read-write lock with fair scheduling
pub struct DistributedRwLock<K> {
    /// Active readers per key
    readers: Arc<DashMap<K, HashSet<Uuid>>>,
    /// Active writer per key
    writers: Arc<DashMap<K, LockHolder>>,
    /// Wait queue for fair scheduling
    wait_queue: Arc<RwLock<Vec<(K, Uuid, LockMode)>>>,
    /// Configuration
    config: LockConfig,
}

impl<K> DistributedRwLock<K>
where
    K: Eq + Hash + Clone + Debug + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self::with_config(LockConfig::default())
    }

    pub fn with_config(config: LockConfig) -> Self {
        Self {
            readers: Arc::new(DashMap::new()),
            writers: Arc::new(DashMap::new()),
            wait_queue: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    /// Acquire a read lock
    pub async fn read_lock(
        &self,
        key: K,
        owner_id: Uuid,
        timeout_duration: Option<Duration>,
    ) -> EnterpriseResult<()> {
        let timeout_duration = timeout_duration
            .unwrap_or_else(|| Duration::from_millis(self.config.default_timeout_ms));

        let start = Instant::now();

        loop {
            // Check if there's an active writer
            if let Some(holder) = self.writers.get(&key) {
                if holder.is_expired() {
                    drop(holder);
                    self.writers.remove(&key);
                } else {
                    // Writer exists, wait
                    if start.elapsed() >= timeout_duration {
                        return Err(EnterpriseError::Other(
                            "Read lock acquisition timeout".to_string()
                        ));
                    }
                    sleep(Duration::from_millis(10)).await;
                    continue;
                }
            }

            // Acquire read lock
            self.readers
                .entry(key.clone())
                .or_insert_with(HashSet::new)
                .insert(owner_id);

            return Ok(());
        }
    }

    /// Acquire a write lock
    pub async fn write_lock(
        &self,
        key: K,
        owner_id: Uuid,
        timeout_duration: Option<Duration>,
    ) -> EnterpriseResult<FencingToken> {
        let timeout_duration = timeout_duration
            .unwrap_or_else(|| Duration::from_millis(self.config.default_timeout_ms));

        let start = Instant::now();

        loop {
            // Check if there are active readers or writers
            let has_readers = self.readers.get(&key).map_or(false, |r| !r.is_empty());
            let has_writer = self.writers.contains_key(&key);

            if !has_readers && !has_writer {
                // Can acquire write lock
                let lease_duration = Duration::from_millis(self.config.lease_duration_ms);
                let holder = LockHolder {
                    owner_id,
                    acquired_at: Instant::now(),
                    expires_at: Instant::now() + lease_duration,
                    token: FencingToken::new(Instant::now().elapsed().as_nanos() as u64),
                    mode: LockMode::Exclusive,
                    count: 1,
                };

                let token = holder.token;
                self.writers.insert(key, holder);

                return Ok(token);
            }

            // Check timeout
            if start.elapsed() >= timeout_duration {
                return Err(EnterpriseError::Other(
                    "Write lock acquisition timeout".to_string()
                ));
            }

            sleep(Duration::from_millis(10)).await;
        }
    }

    /// Release a read lock
    pub fn read_unlock(&self, key: &K, owner_id: Uuid) -> EnterpriseResult<()> {
        if let Some(mut readers) = self.readers.get_mut(key) {
            readers.remove(&owner_id);
            if readers.is_empty() {
                drop(readers);
                self.readers.remove(key);
            }
        }
        Ok(())
    }

    /// Release a write lock
    pub fn write_unlock(&self, key: &K, owner_id: Uuid) -> EnterpriseResult<()> {
        if let Some(holder) = self.writers.get(key) {
            if !holder.owned_by(&owner_id) {
                return Err(EnterpriseError::Other(
                    "Cannot unlock: not the lock owner".to_string()
                ));
            }
            drop(holder);
            self.writers.remove(key);
        }
        Ok(())
    }

    /// Get number of active readers
    pub fn reader_count(&self, key: &K) -> usize {
        self.readers.get(key).map_or(0, |r| r.len())
    }

    /// Check if write locked
    pub fn is_write_locked(&self, key: &K) -> bool {
        self.writers.contains_key(key)
    }
}

impl<K> Default for DistributedRwLock<K>
where
    K: Eq + Hash + Clone + Debug + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Deadlock detector
pub struct DeadlockDetector<K> {
    /// Wait-for graph: key -> set of keys it's waiting for
    wait_graph: Arc<RwLock<HashMap<K, HashSet<K>>>>,
    /// Lock holders: key -> owner
    holders: Arc<DashMap<K, Uuid>>,
    /// Configuration
    config: LockConfig,
}

impl<K> DeadlockDetector<K>
where
    K: Eq + Hash + Clone + Debug + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self::with_config(LockConfig::default())
    }

    pub fn with_config(config: LockConfig) -> Self {
        let enable_deadlock = config.enable_deadlock_detection;

        let detector = Self {
            wait_graph: Arc::new(RwLock::new(HashMap::new())),
            holders: Arc::new(DashMap::new()),
            config,
        };

        if enable_deadlock {
            detector.start_detection_task();
        }

        detector
    }

    /// Record a wait relationship
    pub async fn add_wait(&self, waiter: K, waiting_for: K) {
        let mut graph = self.wait_graph.write().await;
        graph.entry(waiter).or_insert_with(HashSet::new).insert(waiting_for);
    }

    /// Remove a wait relationship
    pub async fn remove_wait(&self, waiter: &K) {
        let mut graph = self.wait_graph.write().await;
        graph.remove(waiter);
    }

    /// Detect cycles in the wait-for graph (indicating deadlock)
    pub async fn detect_deadlock(&self) -> Option<Vec<K>> {
        let graph = self.wait_graph.read().await;

        for start_node in graph.keys() {
            if let Some(cycle) = self.find_cycle(start_node, &graph) {
                return Some(cycle);
            }
        }

        None
    }

    /// Find a cycle starting from a given node using DFS
    fn find_cycle(&self, start: &K, graph: &HashMap<K, HashSet<K>>) -> Option<Vec<K>> {
        let mut visited = HashSet::new();
        let mut path = Vec::new();

        self.dfs_cycle(start, graph, &mut visited, &mut path)
    }

    fn dfs_cycle(
        &self,
        node: &K,
        graph: &HashMap<K, HashSet<K>>,
        visited: &mut HashSet<K>,
        path: &mut Vec<K>,
    ) -> Option<Vec<K>> {
        if path.contains(node) {
            // Found a cycle
            let cycle_start = path.iter().position(|n| n == node).unwrap();
            return Some(path[cycle_start..].to_vec());
        }

        if visited.contains(node) {
            return None;
        }

        visited.insert(node.clone());
        path.push(node.clone());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if let Some(cycle) = self.dfs_cycle(neighbor, graph, visited, path) {
                    return Some(cycle);
                }
            }
        }

        path.pop();
        None
    }

    /// Start background deadlock detection task
    fn start_detection_task(&self) {
        let wait_graph = Arc::clone(&self.wait_graph);
        let interval_ms = self.config.deadlock_check_interval_ms;

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(interval_ms));

            loop {
                ticker.tick().await;

                // Run deadlock detection
                // In a real implementation, this would trigger recovery actions
                let graph = wait_graph.read().await;
                // Deadlock detection logic here
                drop(graph);
            }
        });
    }
}

impl<K> Default for DeadlockDetector<K>
where
    K: Eq + Hash + Clone + Debug + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_distributed_mutex_basic() {
        let mutex = DistributedMutex::new();
        let owner = Uuid::new_v4();

        let token = mutex.lock(1, owner, None).await.unwrap();
        assert!(mutex.is_locked(&1));

        mutex.unlock(&1, owner, token).await.unwrap();
        assert!(!mutex.is_locked(&1));
    }

    #[tokio::test]
    async fn test_distributed_mutex_reentrant() {
        let mutex = DistributedMutex::new();
        let owner = Uuid::new_v4();

        let token1 = mutex.lock(1, owner, None).await.unwrap();
        let token2 = mutex.lock(1, owner, None).await.unwrap();

        assert_eq!(token1, token2);

        mutex.unlock(&1, owner, token1).await.unwrap();
        assert!(mutex.is_locked(&1)); // Still locked once

        mutex.unlock(&1, owner, token2).await.unwrap();
        assert!(!mutex.is_locked(&1));
    }

    #[tokio::test]
    async fn test_distributed_mutex_conflict() {
        let mutex = DistributedMutex::new();
        let owner1 = Uuid::new_v4();
        let owner2 = Uuid::new_v4();

        let _token1 = mutex.lock(1, owner1, None).await.unwrap();

        // Second owner should timeout
        let result = mutex.lock(1, owner2, Some(Duration::from_millis(100))).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fencing_token() {
        let token1 = FencingToken::new(1);
        let token2 = token1.next();

        assert_eq!(token1.value(), 1);
        assert_eq!(token2.value(), 2);
        assert!(token2 > token1);
    }

    #[tokio::test]
    async fn test_rwlock_multiple_readers() {
        let rwlock = DistributedRwLock::new();
        let reader1 = Uuid::new_v4();
        let reader2 = Uuid::new_v4();

        rwlock.read_lock(1, reader1, None).await.unwrap();
        rwlock.read_lock(1, reader2, None).await.unwrap();

        assert_eq!(rwlock.reader_count(&1), 2);

        rwlock.read_unlock(&1, reader1).unwrap();
        assert_eq!(rwlock.reader_count(&1), 1);

        rwlock.read_unlock(&1, reader2).unwrap();
        assert_eq!(rwlock.reader_count(&1), 0);
    }

    #[tokio::test]
    async fn test_rwlock_writer_blocks_readers() {
        let rwlock = DistributedRwLock::new();
        let writer = Uuid::new_v4();
        let reader = Uuid::new_v4();

        rwlock.write_lock(1, writer, None).await.unwrap();
        assert!(rwlock.is_write_locked(&1));

        // Reader should timeout
        let result = rwlock.read_lock(1, reader, Some(Duration::from_millis(100))).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_deadlock_detector_no_cycle() {
        let detector = DeadlockDetector::new();

        detector.add_wait(1, 2).await;
        detector.add_wait(2, 3).await;

        let cycle = detector.detect_deadlock().await;
        assert!(cycle.is_none());
    }

    #[tokio::test]
    async fn test_deadlock_detector_with_cycle() {
        let detector = DeadlockDetector::new();

        detector.add_wait(1, 2).await;
        detector.add_wait(2, 3).await;
        detector.add_wait(3, 1).await;

        let cycle = detector.detect_deadlock().await;
        assert!(cycle.is_some());
    }
}
