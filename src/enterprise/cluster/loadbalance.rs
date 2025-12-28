//! Load Balancing
//!
//! Implements multiple load balancing strategies: round-robin, least connections,
//! weighted distribution, and health-aware routing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use thiserror::Error;
use tokio::sync::RwLock;

use super::config::NodeId;

/// Load balancing errors
#[derive(Error, Debug)]
pub enum LoadBalanceError {
    #[error("No healthy backends available")]
    NoHealthyBackends,
    #[error("Backend not found: {0}")]
    BackendNotFound(NodeId),
    #[error("Invalid weight: {0}")]
    InvalidWeight(u32),
}

pub type LoadBalanceResult<T> = Result<T, LoadBalanceError>;

/// Load balancing strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Strategy {
    /// Simple round-robin
    RoundRobin,
    /// Least connections
    LeastConnections,
    /// Weighted round-robin
    Weighted,
    /// Random selection
    Random,
    /// IP hash (sticky sessions)
    IpHash,
}

/// Backend node information
#[derive(Debug, Clone)]
pub struct Backend {
    pub id: NodeId,
    pub weight: u32,
    pub max_connections: usize,
    pub current_connections: Arc<AtomicUsize>,
    pub is_healthy: bool,
    pub total_requests: Arc<AtomicU64>,
    pub failed_requests: Arc<AtomicU64>,
}

impl Backend {
    /// Create a new backend
    pub fn new(id: NodeId, weight: u32, max_connections: usize) -> Self {
        Self {
            id,
            weight,
            max_connections,
            current_connections: Arc::new(AtomicUsize::new(0)),
            is_healthy: true,
            total_requests: Arc::new(AtomicU64::new(0)),
            failed_requests: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Get current connection count
    pub fn connection_count(&self) -> usize {
        self.current_connections.load(Ordering::Relaxed)
    }

    /// Check if backend can accept more connections
    pub fn can_accept(&self) -> bool {
        self.is_healthy && self.connection_count() < self.max_connections
    }

    /// Increment connection count
    pub fn acquire_connection(&self) {
        self.current_connections.fetch_add(1, Ordering::Relaxed);
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement connection count
    pub fn release_connection(&self) {
        self.current_connections.fetch_sub(1, Ordering::Relaxed);
    }

    /// Record failed request
    pub fn record_failure(&self) {
        self.failed_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.total_requests.load(Ordering::Relaxed);
        if total == 0 {
            return 1.0;
        }
        let failed = self.failed_requests.load(Ordering::Relaxed);
        1.0 - (failed as f64 / total as f64)
    }

    /// Get load factor (0.0 - 1.0)
    pub fn load_factor(&self) -> f64 {
        if self.max_connections == 0 {
            return 0.0;
        }
        self.connection_count() as f64 / self.max_connections as f64
    }
}

/// Load balancer
pub struct LoadBalancer {
    /// Load balancing strategy
    strategy: Strategy,

    /// All backends
    backends: Arc<RwLock<HashMap<NodeId, Backend>>>,

    /// Round-robin counter
    round_robin_counter: Arc<AtomicUsize>,

    /// Weighted round-robin state
    weighted_state: Arc<RwLock<WeightedState>>,
}

/// Weighted round-robin state
struct WeightedState {
    current_index: usize,
    current_weight: i32,
    max_weight: i32,
    gcd: i32,
}

impl WeightedState {
    fn new() -> Self {
        Self {
            current_index: 0,
            current_weight: 0,
            max_weight: 0,
            gcd: 1,
        }
    }
}

impl LoadBalancer {
    /// Create a new load balancer
    pub fn new(strategy: Strategy) -> Self {
        Self {
            strategy,
            backends: Arc::new(RwLock::new(HashMap::new())),
            round_robin_counter: Arc::new(AtomicUsize::new(0)),
            weighted_state: Arc::new(RwLock::new(WeightedState::new())),
        }
    }

    /// Add a backend
    pub async fn add_backend(&self, backend: Backend) -> LoadBalanceResult<()> {
        let mut backends = self.backends.write().await;
        backends.insert(backend.id.clone(), backend);

        // Update weighted state if using weighted strategy
        if self.strategy == Strategy::Weighted {
            self.update_weighted_state(&backends).await;
        }

        Ok(())
    }

    /// Remove a backend
    pub async fn remove_backend(&self, node_id: &NodeId) -> LoadBalanceResult<Backend> {
        let mut backends = self.backends.write().await;
        let backend = backends
            .remove(node_id)
            .ok_or_else(|| LoadBalanceError::BackendNotFound(node_id.clone()))?;

        // Update weighted state if using weighted strategy
        if self.strategy == Strategy::Weighted {
            self.update_weighted_state(&backends).await;
        }

        Ok(backend)
    }

    /// Update backend health status
    pub async fn set_backend_health(&self, node_id: &NodeId, is_healthy: bool) {
        let mut backends = self.backends.write().await;
        if let Some(backend) = backends.get_mut(node_id) {
            backend.is_healthy = is_healthy;
        }
    }

    /// Select a backend using the configured strategy
    pub async fn select(&self) -> LoadBalanceResult<NodeId> {
        match self.strategy {
            Strategy::RoundRobin => self.select_round_robin().await,
            Strategy::LeastConnections => self.select_least_connections().await,
            Strategy::Weighted => self.select_weighted().await,
            Strategy::Random => self.select_random().await,
            Strategy::IpHash => self.select_round_robin().await, // Fallback
        }
    }

    /// Select backend with IP hash for sticky sessions
    pub async fn select_with_ip(&self, client_ip: &str) -> LoadBalanceResult<NodeId> {
        if self.strategy == Strategy::IpHash {
            self.select_ip_hash(client_ip).await
        } else {
            self.select().await
        }
    }

    /// Round-robin selection
    async fn select_round_robin(&self) -> LoadBalanceResult<NodeId> {
        let backends = self.backends.read().await;
        let healthy: Vec<&Backend> = backends
            .values()
            .filter(|b| b.is_healthy && b.can_accept())
            .collect();

        if healthy.is_empty() {
            return Err(LoadBalanceError::NoHealthyBackends);
        }

        let index = self.round_robin_counter.fetch_add(1, Ordering::Relaxed) % healthy.len();
        Ok(healthy[index].id.clone())
    }

    /// Least connections selection
    async fn select_least_connections(&self) -> LoadBalanceResult<NodeId> {
        let backends = self.backends.read().await;
        let healthy: Vec<&Backend> = backends
            .values()
            .filter(|b| b.is_healthy && b.can_accept())
            .collect();

        if healthy.is_empty() {
            return Err(LoadBalanceError::NoHealthyBackends);
        }

        // Find backend with least connections
        let selected = healthy
            .into_iter()
            .min_by_key(|b| b.connection_count())
            .unwrap();

        Ok(selected.id.clone())
    }

    /// Weighted round-robin selection
    async fn select_weighted(&self) -> LoadBalanceResult<NodeId> {
        let backends = self.backends.read().await;
        let healthy: Vec<&Backend> = backends
            .values()
            .filter(|b| b.is_healthy && b.can_accept())
            .collect();

        if healthy.is_empty() {
            return Err(LoadBalanceError::NoHealthyBackends);
        }

        let mut state = self.weighted_state.write().await;

        loop {
            state.current_index = (state.current_index + 1) % healthy.len();

            if state.current_index == 0 {
                state.current_weight -= state.gcd;
                if state.current_weight <= 0 {
                    state.current_weight = state.max_weight;
                }
            }

            let backend = healthy[state.current_index];
            if backend.weight as i32 >= state.current_weight {
                return Ok(backend.id.clone());
            }
        }
    }

    /// Random selection
    async fn select_random(&self) -> LoadBalanceResult<NodeId> {
        use rand::Rng;

        let backends = self.backends.read().await;
        let healthy: Vec<&Backend> = backends
            .values()
            .filter(|b| b.is_healthy && b.can_accept())
            .collect();

        if healthy.is_empty() {
            return Err(LoadBalanceError::NoHealthyBackends);
        }

        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..healthy.len());

        Ok(healthy[index].id.clone())
    }

    /// IP hash selection (sticky sessions)
    async fn select_ip_hash(&self, client_ip: &str) -> LoadBalanceResult<NodeId> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let backends = self.backends.read().await;
        let healthy: Vec<&Backend> = backends
            .values()
            .filter(|b| b.is_healthy && b.can_accept())
            .collect();

        if healthy.is_empty() {
            return Err(LoadBalanceError::NoHealthyBackends);
        }

        // Hash the IP address
        let mut hasher = DefaultHasher::new();
        client_ip.hash(&mut hasher);
        let hash = hasher.finish();

        let index = (hash as usize) % healthy.len();
        Ok(healthy[index].id.clone())
    }

    /// Update weighted state
    async fn update_weighted_state(&self, backends: &HashMap<NodeId, Backend>) {
        let healthy: Vec<&Backend> = backends
            .values()
            .filter(|b| b.is_healthy)
            .collect();

        if healthy.is_empty() {
            return;
        }

        let max_weight = healthy.iter().map(|b| b.weight).max().unwrap_or(1) as i32;
        let gcd = Self::gcd_list(&healthy.iter().map(|b| b.weight as i32).collect::<Vec<_>>());

        let mut state = self.weighted_state.write().await;
        state.max_weight = max_weight;
        state.gcd = gcd;
        state.current_weight = 0;
        state.current_index = 0;
    }

    /// Calculate GCD of a list of numbers
    fn gcd_list(numbers: &[i32]) -> i32 {
        if numbers.is_empty() {
            return 1;
        }

        let mut result = numbers[0];
        for &num in &numbers[1..] {
            result = Self::gcd(result, num);
        }
        result
    }

    /// Calculate GCD of two numbers
    fn gcd(mut a: i32, mut b: i32) -> i32 {
        while b != 0 {
            let temp = b;
            b = a % b;
            a = temp;
        }
        a.abs()
    }

    /// Get backend by ID
    pub async fn get_backend(&self, node_id: &NodeId) -> Option<Backend> {
        self.backends.read().await.get(node_id).cloned()
    }

    /// Get all backends
    pub async fn get_backends(&self) -> Vec<Backend> {
        self.backends.read().await.values().cloned().collect()
    }

    /// Get statistics
    pub async fn stats(&self) -> LoadBalancerStats {
        let backends = self.backends.read().await;

        let total_backends = backends.len();
        let healthy_backends = backends.values().filter(|b| b.is_healthy).count();
        let total_connections: usize = backends
            .values()
            .map(|b| b.connection_count())
            .sum();
        let total_requests: u64 = backends
            .values()
            .map(|b| b.total_requests.load(Ordering::Relaxed))
            .sum();
        let failed_requests: u64 = backends
            .values()
            .map(|b| b.failed_requests.load(Ordering::Relaxed))
            .sum();

        LoadBalancerStats {
            strategy: self.strategy,
            total_backends,
            healthy_backends,
            total_connections,
            total_requests,
            failed_requests,
        }
    }
}

/// Load balancer statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerStats {
    pub strategy: Strategy,
    pub total_backends: usize,
    pub healthy_backends: usize,
    pub total_connections: usize,
    pub total_requests: u64,
    pub failed_requests: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_backend_creation() {
        let backend = Backend::new("node1".to_string(), 100, 1000);

        assert_eq!(backend.id, "node1");
        assert_eq!(backend.weight, 100);
        assert_eq!(backend.connection_count(), 0);
        assert!(backend.can_accept());
    }

    #[tokio::test]
    async fn test_backend_connections() {
        let backend = Backend::new("node1".to_string(), 100, 10);

        backend.acquire_connection();
        assert_eq!(backend.connection_count(), 1);

        backend.release_connection();
        assert_eq!(backend.connection_count(), 0);
    }

    #[tokio::test]
    async fn test_load_balancer_creation() {
        let lb = LoadBalancer::new(Strategy::RoundRobin);

        let stats = lb.stats().await;
        assert_eq!(stats.total_backends, 0);
    }

    #[tokio::test]
    async fn test_add_backend() {
        let lb = LoadBalancer::new(Strategy::RoundRobin);
        let backend = Backend::new("node1".to_string(), 100, 1000);

        lb.add_backend(backend).await.unwrap();

        let stats = lb.stats().await;
        assert_eq!(stats.total_backends, 1);
    }

    #[tokio::test]
    async fn test_round_robin_selection() {
        let lb = LoadBalancer::new(Strategy::RoundRobin);

        let backend1 = Backend::new("node1".to_string(), 100, 1000);
        let backend2 = Backend::new("node2".to_string(), 100, 1000);

        lb.add_backend(backend1).await.unwrap();
        lb.add_backend(backend2).await.unwrap();

        let selected1 = lb.select().await.unwrap();
        let selected2 = lb.select().await.unwrap();

        // Should alternate (though order is not guaranteed in HashMap)
        assert_ne!(selected1, selected2);
    }

    #[tokio::test]
    async fn test_least_connections_selection() {
        let lb = LoadBalancer::new(Strategy::LeastConnections);

        let backend1 = Backend::new("node1".to_string(), 100, 1000);
        let backend2 = Backend::new("node2".to_string(), 100, 1000);

        // Add connection to backend1
        backend1.acquire_connection();

        lb.add_backend(backend1).await.unwrap();
        lb.add_backend(backend2).await.unwrap();

        let selected = lb.select().await.unwrap();
        // Should select node2 (least connections)
        assert_eq!(selected, "node2");
    }

    #[tokio::test]
    async fn test_no_healthy_backends() {
        let lb = LoadBalancer::new(Strategy::RoundRobin);

        let result = lb.select().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_backend_health() {
        let lb = LoadBalancer::new(Strategy::RoundRobin);
        let backend = Backend::new("node1".to_string(), 100, 1000);

        lb.add_backend(backend).await.unwrap();
        lb.set_backend_health(&"node1".to_string(), false).await;

        let result = lb.select().await;
        assert!(result.is_err()); // No healthy backends
    }

    #[test]
    fn test_gcd() {
        assert_eq!(LoadBalancer::gcd(12, 8), 4);
        assert_eq!(LoadBalancer::gcd(100, 50), 50);
        assert_eq!(LoadBalancer::gcd(7, 3), 1);
    }

    #[test]
    fn test_gcd_list() {
        assert_eq!(LoadBalancer::gcd_list(&[12, 8, 4]), 4);
        assert_eq!(LoadBalancer::gcd_list(&[100, 50, 25]), 25);
    }
}
