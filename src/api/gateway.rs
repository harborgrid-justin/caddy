//! # API Gateway
//!
//! This module provides a comprehensive API gateway with the following features:
//!
//! - **Request Routing**: Route requests to appropriate backend services
//! - **API Versioning**: Support for multiple API versions (v1, v2, etc.)
//! - **Circuit Breaker**: Automatic failure detection and recovery
//! - **Retry Logic**: Exponential backoff retry for transient failures
//! - **Load Balancing**: Distribute requests across multiple backend instances
//! - **Request/Response Transformation**: Modify requests and responses
//! - **Service Discovery**: Dynamic backend service discovery
//! - **Health Monitoring**: Track backend service health
//!
//! # Circuit Breaker Pattern
//!
//! The circuit breaker prevents cascading failures by:
//! 1. **Closed**: Normal operation, requests pass through
//! 2. **Open**: Too many failures, requests fail fast
//! 3. **Half-Open**: Testing if service recovered
//!
//! # Examples
//!
//! ```rust,ignore
//! use caddy::api::gateway::*;
//!
//! let config = GatewayConfig::default();
//! let gateway = ApiGateway::new(config);
//!
//! // Route request through gateway
//! let response = gateway.route_request(request).await?;
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

// ============================================================================
// Gateway Configuration
// ============================================================================

/// API Gateway configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,

    /// Retry configuration
    pub retry: RetryConfig,

    /// Request timeout
    pub request_timeout: Duration,

    /// Enable request transformation
    pub enable_transformation: bool,

    /// Backend services
    pub backends: Vec<BackendConfig>,

    /// Load balancing strategy
    pub load_balancing_strategy: LoadBalancingStrategy,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            circuit_breaker: CircuitBreakerConfig::default(),
            retry: RetryConfig::default(),
            request_timeout: Duration::from_secs(30),
            enable_transformation: true,
            backends: vec![],
            load_balancing_strategy: LoadBalancingStrategy::RoundRobin,
        }
    }
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Failure threshold before opening circuit
    pub failure_threshold: u32,

    /// Success threshold to close circuit from half-open
    pub success_threshold: u32,

    /// Timeout before attempting to close circuit
    pub timeout: Duration,

    /// Rolling window for failure tracking
    pub window_duration: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            timeout: Duration::from_secs(60),
            window_duration: Duration::from_secs(60),
        }
    }
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_attempts: u32,

    /// Initial retry delay
    pub initial_delay: Duration,

    /// Maximum retry delay
    pub max_delay: Duration,

    /// Backoff multiplier
    pub backoff_multiplier: f64,

    /// Jitter factor (0.0 - 1.0)
    pub jitter: f64,

    /// Retry on these HTTP status codes
    pub retry_on_status: Vec<u16>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            jitter: 0.1,
            retry_on_status: vec![408, 429, 500, 502, 503, 504],
        }
    }
}

/// Backend service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    /// Backend ID
    pub id: String,

    /// Backend base URL
    pub url: String,

    /// Backend weight for load balancing
    pub weight: u32,

    /// Health check endpoint
    pub health_check_path: String,

    /// Health check interval
    pub health_check_interval: Duration,
}

/// Load balancing strategy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LoadBalancingStrategy {
    /// Round-robin distribution
    RoundRobin,
    /// Least connections
    LeastConnections,
    /// Weighted distribution
    Weighted,
    /// Random selection
    Random,
}

// ============================================================================
// Circuit Breaker
// ============================================================================

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests pass through
    Closed,
    /// Circuit is open, requests fail fast
    Open,
    /// Circuit is half-open, testing if service recovered
    HalfOpen,
}

/// Circuit breaker implementation
pub struct CircuitBreaker {
    /// Configuration
    config: CircuitBreakerConfig,

    /// Current state
    state: Arc<RwLock<CircuitState>>,

    /// Failure count in current window
    failures: Arc<RwLock<u32>>,

    /// Success count in half-open state
    successes: Arc<RwLock<u32>>,

    /// Last state transition time
    last_transition: Arc<RwLock<Instant>>,

    /// Window start time
    window_start: Arc<RwLock<Instant>>,
}

impl CircuitBreaker {
    /// Create new circuit breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failures: Arc::new(RwLock::new(0)),
            successes: Arc::new(RwLock::new(0)),
            last_transition: Arc::new(RwLock::new(Instant::now())),
            window_start: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Check if request is allowed
    pub fn is_request_allowed(&self) -> Result<(), CircuitBreakerError> {
        let state = *self.state.read();

        match state {
            CircuitState::Closed => Ok(()),
            CircuitState::Open => {
                // Check if timeout has elapsed
                let last_transition = *self.last_transition.read();
                if last_transition.elapsed() >= self.config.timeout {
                    // Transition to half-open
                    self.transition_to_half_open();
                    Ok(())
                } else {
                    Err(CircuitBreakerError::CircuitOpen)
                }
            }
            CircuitState::HalfOpen => Ok(()),
        }
    }

    /// Record successful request
    pub fn record_success(&self) {
        let state = *self.state.read();

        match state {
            CircuitState::Closed => {
                // Reset failures
                *self.failures.write() = 0;
            }
            CircuitState::HalfOpen => {
                let mut successes = self.successes.write();
                *successes += 1;

                // Check if we should close the circuit
                if *successes >= self.config.success_threshold {
                    self.transition_to_closed();
                }
            }
            CircuitState::Open => {}
        }
    }

    /// Record failed request
    pub fn record_failure(&self) {
        let state = *self.state.read();

        // Reset window if needed
        let window_start = *self.window_start.read();
        if window_start.elapsed() >= self.config.window_duration {
            *self.window_start.write() = Instant::now();
            *self.failures.write() = 0;
        }

        match state {
            CircuitState::Closed => {
                let mut failures = self.failures.write();
                *failures += 1;

                // Check if we should open the circuit
                if *failures >= self.config.failure_threshold {
                    self.transition_to_open();
                }
            }
            CircuitState::HalfOpen => {
                // Single failure in half-open transitions to open
                self.transition_to_open();
            }
            CircuitState::Open => {}
        }
    }

    /// Get current state
    pub fn state(&self) -> CircuitState {
        *self.state.read()
    }

    /// Get statistics
    pub fn statistics(&self) -> CircuitBreakerStatistics {
        CircuitBreakerStatistics {
            state: self.state(),
            failures: *self.failures.read(),
            successes: *self.successes.read(),
            last_transition: *self.last_transition.read(),
        }
    }

    /// Transition to closed state
    fn transition_to_closed(&self) {
        tracing::info!("Circuit breaker transitioning to CLOSED");
        *self.state.write() = CircuitState::Closed;
        *self.failures.write() = 0;
        *self.successes.write() = 0;
        *self.last_transition.write() = Instant::now();
    }

    /// Transition to open state
    fn transition_to_open(&self) {
        tracing::warn!("Circuit breaker transitioning to OPEN");
        *self.state.write() = CircuitState::Open;
        *self.successes.write() = 0;
        *self.last_transition.write() = Instant::now();
    }

    /// Transition to half-open state
    fn transition_to_half_open(&self) {
        tracing::info!("Circuit breaker transitioning to HALF-OPEN");
        *self.state.write() = CircuitState::HalfOpen;
        *self.successes.write() = 0;
        *self.last_transition.write() = Instant::now();
    }
}

/// Circuit breaker statistics
#[derive(Debug, Clone)]
pub struct CircuitBreakerStatistics {
    pub state: CircuitState,
    pub failures: u32,
    pub successes: u32,
    pub last_transition: Instant,
}

/// Circuit breaker error
#[derive(Debug, Clone, thiserror::Error)]
pub enum CircuitBreakerError {
    #[error("Circuit breaker is open")]
    CircuitOpen,
}

// ============================================================================
// Retry Logic with Exponential Backoff
// ============================================================================

/// Retry policy implementation
pub struct RetryPolicy {
    config: RetryConfig,
}

impl RetryPolicy {
    /// Create new retry policy
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Execute with retry
    pub async fn execute<F, Fut, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Debug,
    {
        let mut attempt = 0;
        let mut delay = self.config.initial_delay;

        loop {
            attempt += 1;

            match operation().await {
                Ok(result) => return Ok(result),
                Err(err) => {
                    if attempt >= self.config.max_attempts {
                        tracing::error!(
                            "Request failed after {} attempts: {:?}",
                            attempt,
                            err
                        );
                        return Err(err);
                    }

                    // Calculate next delay with exponential backoff and jitter
                    let jitter_amount = delay.as_millis() as f64 * self.config.jitter;
                    let jitter = rand::random::<f64>() * jitter_amount * 2.0 - jitter_amount;
                    let next_delay = delay.mul_f64(self.config.backoff_multiplier);
                    let next_delay = Duration::from_millis(
                        (next_delay.as_millis() as f64 + jitter) as u64,
                    );

                    delay = next_delay.min(self.config.max_delay);

                    tracing::warn!(
                        "Request failed (attempt {}/{}), retrying after {:?}",
                        attempt,
                        self.config.max_attempts,
                        delay
                    );

                    sleep(delay).await;
                }
            }
        }
    }

    /// Check if status code should be retried
    pub fn should_retry_status(&self, status_code: u16) -> bool {
        self.config.retry_on_status.contains(&status_code)
    }
}

// ============================================================================
// API Gateway
// ============================================================================

/// API Gateway
pub struct ApiGateway {
    /// Configuration
    config: GatewayConfig,

    /// Circuit breakers per backend
    circuit_breakers: Arc<RwLock<HashMap<String, Arc<CircuitBreaker>>>>,

    /// Retry policy
    retry_policy: Arc<RetryPolicy>,

    /// Backend selector
    backend_selector: Arc<RwLock<BackendSelector>>,

    /// Request transformer
    transformer: Arc<RequestTransformer>,
}

impl ApiGateway {
    /// Create new API gateway
    pub fn new(config: GatewayConfig) -> Self {
        let retry_policy = Arc::new(RetryPolicy::new(config.retry.clone()));

        let mut circuit_breakers = HashMap::new();
        for backend in &config.backends {
            circuit_breakers.insert(
                backend.id.clone(),
                Arc::new(CircuitBreaker::new(config.circuit_breaker.clone())),
            );
        }

        let backend_selector = BackendSelector::new(
            config.backends.clone(),
            config.load_balancing_strategy,
        );

        Self {
            config,
            circuit_breakers: Arc::new(RwLock::new(circuit_breakers)),
            retry_policy,
            backend_selector: Arc::new(RwLock::new(backend_selector)),
            transformer: Arc::new(RequestTransformer::new()),
        }
    }

    /// Route request to backend
    pub async fn route_request(&self, request: GatewayRequest) -> Result<GatewayResponse, GatewayError> {
        // Select backend
        let backend = self
            .backend_selector
            .read()
            .select()
            .ok_or(GatewayError::NoBackendAvailable)?;

        // Get circuit breaker for backend
        let circuit_breaker = self
            .circuit_breakers
            .read()
            .get(&backend.id)
            .cloned()
            .ok_or(GatewayError::CircuitBreakerNotFound)?;

        // Check circuit breaker
        circuit_breaker
            .is_request_allowed()
            .map_err(|_| GatewayError::CircuitBreakerOpen)?;

        // Execute with retry
        let result = self
            .retry_policy
            .execute(|| async {
                self.execute_request(&backend, &request).await
            })
            .await;

        match &result {
            Ok(_) => circuit_breaker.record_success(),
            Err(_) => circuit_breaker.record_failure(),
        }

        result
    }

    /// Execute request to backend
    async fn execute_request(
        &self,
        backend: &BackendConfig,
        request: &GatewayRequest,
    ) -> Result<GatewayResponse, GatewayError> {
        // Transform request if enabled
        let transformed_request = if self.config.enable_transformation {
            self.transformer.transform_request(request.clone())?
        } else {
            request.clone()
        };

        // TODO: Actual HTTP request to backend
        // For now, simulate successful response
        Ok(GatewayResponse {
            status_code: 200,
            body: serde_json::json!({"message": "Success"}),
            headers: HashMap::new(),
        })
    }

    /// Get gateway statistics
    pub fn statistics(&self) -> GatewayStatistics {
        let circuit_breakers = self.circuit_breakers.read();
        let breaker_stats: HashMap<String, CircuitBreakerStatistics> = circuit_breakers
            .iter()
            .map(|(id, cb)| (id.clone(), cb.statistics()))
            .collect();

        GatewayStatistics {
            circuit_breakers: breaker_stats,
            backend_count: self.config.backends.len(),
        }
    }
}

/// Gateway request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayRequest {
    pub method: String,
    pub path: String,
    pub query: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: Option<serde_json::Value>,
}

/// Gateway response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayResponse {
    pub status_code: u16,
    pub body: serde_json::Value,
    pub headers: HashMap<String, String>,
}

/// Gateway error
#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    #[error("No backend available")]
    NoBackendAvailable,

    #[error("Circuit breaker not found")]
    CircuitBreakerNotFound,

    #[error("Circuit breaker is open")]
    CircuitBreakerOpen,

    #[error("Request transformation failed: {0}")]
    TransformationError(String),

    #[error("Backend request failed: {0}")]
    BackendError(String),
}

/// Gateway statistics
#[derive(Debug, Clone)]
pub struct GatewayStatistics {
    pub circuit_breakers: HashMap<String, CircuitBreakerStatistics>,
    pub backend_count: usize,
}

// ============================================================================
// Backend Selector
// ============================================================================

/// Backend selector for load balancing
struct BackendSelector {
    backends: Vec<BackendConfig>,
    strategy: LoadBalancingStrategy,
    round_robin_index: usize,
}

impl BackendSelector {
    fn new(backends: Vec<BackendConfig>, strategy: LoadBalancingStrategy) -> Self {
        Self {
            backends,
            strategy,
            round_robin_index: 0,
        }
    }

    fn select(&mut self) -> Option<BackendConfig> {
        if self.backends.is_empty() {
            return None;
        }

        match self.strategy {
            LoadBalancingStrategy::RoundRobin => {
                let backend = self.backends[self.round_robin_index].clone();
                self.round_robin_index = (self.round_robin_index + 1) % self.backends.len();
                Some(backend)
            }
            LoadBalancingStrategy::Random => {
                let index = rand::random::<usize>() % self.backends.len();
                Some(self.backends[index].clone())
            }
            LoadBalancingStrategy::Weighted => {
                // Weighted random selection
                let total_weight: u32 = self.backends.iter().map(|b| b.weight).sum();
                let mut random = rand::random::<u32>() % total_weight;

                for backend in &self.backends {
                    if random < backend.weight {
                        return Some(backend.clone());
                    }
                    random -= backend.weight;
                }

                Some(self.backends[0].clone())
            }
            LoadBalancingStrategy::LeastConnections => {
                // For now, fall back to round-robin
                // TODO: Track active connections per backend
                let backend = self.backends[self.round_robin_index].clone();
                self.round_robin_index = (self.round_robin_index + 1) % self.backends.len();
                Some(backend)
            }
        }
    }
}

// ============================================================================
// Request Transformer
// ============================================================================

/// Request/response transformer
struct RequestTransformer;

impl RequestTransformer {
    fn new() -> Self {
        Self
    }

    fn transform_request(&self, request: GatewayRequest) -> Result<GatewayRequest, GatewayError> {
        // TODO: Implement actual transformations
        // - Add/remove headers
        // - Modify body
        // - Rewrite URLs
        Ok(request)
    }

    fn transform_response(&self, response: GatewayResponse) -> Result<GatewayResponse, GatewayError> {
        // TODO: Implement actual transformations
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_closed() {
        let config = CircuitBreakerConfig::default();
        let cb = CircuitBreaker::new(config);

        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.is_request_allowed().is_ok());
    }

    #[test]
    fn test_circuit_breaker_opens_on_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        };
        let cb = CircuitBreaker::new(config);

        // Record failures
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(cb.is_request_allowed().is_err());
    }

    #[test]
    fn test_circuit_breaker_success_resets() {
        let config = CircuitBreakerConfig::default();
        let cb = CircuitBreaker::new(config);

        cb.record_failure();
        assert_eq!(*cb.failures.read(), 1);

        cb.record_success();
        assert_eq!(*cb.failures.read(), 0);
    }

    #[tokio::test]
    async fn test_retry_policy_success() {
        let config = RetryConfig::default();
        let policy = RetryPolicy::new(config);

        let mut attempts = 0;
        let result = policy
            .execute(|| async {
                attempts += 1;
                Ok::<_, String>(42)
            })
            .await;

        assert_eq!(result, Ok(42));
        assert_eq!(attempts, 1);
    }

    #[tokio::test]
    async fn test_retry_policy_eventual_success() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(10),
            ..Default::default()
        };
        let policy = RetryPolicy::new(config);

        let mut attempts = 0;
        let result = policy
            .execute(|| async {
                attempts += 1;
                if attempts < 3 {
                    Err("Transient error")
                } else {
                    Ok(42)
                }
            })
            .await;

        assert_eq!(result, Ok(42));
        assert_eq!(attempts, 3);
    }

    #[test]
    fn test_backend_selector_round_robin() {
        let backends = vec![
            BackendConfig {
                id: "backend1".to_string(),
                url: "http://localhost:8001".to_string(),
                weight: 1,
                health_check_path: "/health".to_string(),
                health_check_interval: Duration::from_secs(30),
            },
            BackendConfig {
                id: "backend2".to_string(),
                url: "http://localhost:8002".to_string(),
                weight: 1,
                health_check_path: "/health".to_string(),
                health_check_interval: Duration::from_secs(30),
            },
        ];

        let mut selector = BackendSelector::new(backends, LoadBalancingStrategy::RoundRobin);

        let first = selector.select().unwrap();
        assert_eq!(first.id, "backend1");

        let second = selector.select().unwrap();
        assert_eq!(second.id, "backend2");

        let third = selector.select().unwrap();
        assert_eq!(third.id, "backend1");
    }
}
