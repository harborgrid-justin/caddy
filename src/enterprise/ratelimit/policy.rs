//! Throttling Policies
//!
//! This module provides various throttling policies for handling rate-limited requests:
//! - Delay-based throttling
//! - Rejection with retry-after
//! - Degraded service mode
//! - Priority queuing

use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, Semaphore};
use tokio::time::sleep;

use super::algorithm::{Decision, RateLimitError, RateLimitResult};

// ============================================================================
// Policy Types
// ============================================================================

/// Throttling policy type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyType {
    /// Reject immediately with retry-after
    Reject,
    /// Delay request until capacity available
    Delay,
    /// Enter degraded service mode
    Degrade,
    /// Queue with priority
    PriorityQueue,
    /// Adaptive based on load
    Adaptive,
}

/// Throttling action to take when rate limit exceeded
#[derive(Debug, Clone)]
pub enum ThrottleAction {
    /// Allow the request
    Allow,
    /// Reject the request
    Reject {
        /// Reason for rejection
        reason: String,
        /// Retry after duration
        retry_after: Duration,
    },
    /// Delay the request
    Delay {
        /// Delay duration
        duration: Duration,
    },
    /// Degrade service quality
    Degrade {
        /// Degradation level (0.0 = full service, 1.0 = minimal service)
        level: f64,
    },
    /// Queue the request
    Queue {
        /// Position in queue
        position: usize,
        /// Estimated wait time
        wait_time: Duration,
    },
}

// ============================================================================
// Throttling Policy Trait
// ============================================================================

/// Throttling policy trait
#[async_trait]
pub trait ThrottlingPolicy: Send + Sync {
    /// Handle a rate-limited request
    ///
    /// Returns the action to take for this request
    async fn handle(&self, decision: Decision, priority: u32) -> RateLimitResult<ThrottleAction>;

    /// Get policy type
    fn policy_type(&self) -> PolicyType;

    /// Reset policy state
    async fn reset(&self) -> RateLimitResult<()>;
}

// ============================================================================
// Reject Policy
// ============================================================================

/// Simple rejection policy
///
/// Immediately rejects requests that exceed the rate limit
pub struct RejectPolicy {
    /// Custom rejection message
    message: String,
}

impl RejectPolicy {
    /// Create a new reject policy
    pub fn new(message: Option<String>) -> Self {
        Self {
            message: message.unwrap_or_else(|| "Rate limit exceeded".to_string()),
        }
    }
}

#[async_trait]
impl ThrottlingPolicy for RejectPolicy {
    async fn handle(&self, decision: Decision, _priority: u32) -> RateLimitResult<ThrottleAction> {
        match decision {
            Decision::Allowed { .. } => Ok(ThrottleAction::Allow),
            Decision::Denied { retry_after, .. } => Ok(ThrottleAction::Reject {
                reason: self.message.clone(),
                retry_after: Duration::from_secs(retry_after),
            }),
        }
    }

    fn policy_type(&self) -> PolicyType {
        PolicyType::Reject
    }

    async fn reset(&self) -> RateLimitResult<()> {
        Ok(())
    }
}

// ============================================================================
// Delay Policy
// ============================================================================

/// Delay-based throttling policy
///
/// Delays requests when rate limit is exceeded, up to a maximum delay
pub struct DelayPolicy {
    /// Maximum delay duration
    max_delay: Duration,
    /// Active delays
    delays: Arc<DashMap<String, SystemTime>>,
}

impl DelayPolicy {
    /// Create a new delay policy
    pub fn new(max_delay: Duration) -> Self {
        Self {
            max_delay,
            delays: Arc::new(DashMap::new()),
        }
    }

    /// Apply delay for a request
    async fn apply_delay(&self, duration: Duration) -> RateLimitResult<()> {
        if duration > Duration::ZERO && duration <= self.max_delay {
            sleep(duration).await;
            Ok(())
        } else if duration > self.max_delay {
            Err(RateLimitError::Exceeded(self.max_delay.as_secs()))
        } else {
            Ok(())
        }
    }
}

#[async_trait]
impl ThrottlingPolicy for DelayPolicy {
    async fn handle(&self, decision: Decision, _priority: u32) -> RateLimitResult<ThrottleAction> {
        match decision {
            Decision::Allowed { .. } => Ok(ThrottleAction::Allow),
            Decision::Denied { retry_after, .. } => {
                let delay = Duration::from_secs(retry_after);

                if delay <= self.max_delay {
                    // Apply the delay
                    self.apply_delay(delay).await?;
                    Ok(ThrottleAction::Allow)
                } else {
                    Ok(ThrottleAction::Delay { duration: delay })
                }
            }
        }
    }

    fn policy_type(&self) -> PolicyType {
        PolicyType::Delay
    }

    async fn reset(&self) -> RateLimitResult<()> {
        self.delays.clear();
        Ok(())
    }
}

// ============================================================================
// Degraded Service Policy
// ============================================================================

/// Service degradation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradationConfig {
    /// Threshold for entering degraded mode (% of limit)
    pub threshold: f64,
    /// Degradation level at threshold
    pub initial_level: f64,
    /// Maximum degradation level
    pub max_level: f64,
    /// Features to disable at each level
    pub disabled_features: Vec<Vec<String>>,
}

impl Default for DegradationConfig {
    fn default() -> Self {
        Self {
            threshold: 0.8,
            initial_level: 0.3,
            max_level: 0.8,
            disabled_features: vec![
                vec!["analytics".to_string(), "caching".to_string()],
                vec!["search".to_string(), "recommendations".to_string()],
                vec!["notifications".to_string(), "background_tasks".to_string()],
            ],
        }
    }
}

/// Degraded service policy
///
/// Gradually degrades service quality as rate limits are approached
pub struct DegradedServicePolicy {
    /// Configuration
    config: DegradationConfig,
    /// Current degradation level
    current_level: Arc<RwLock<f64>>,
}

impl DegradedServicePolicy {
    /// Create a new degraded service policy
    pub fn new(config: DegradationConfig) -> Self {
        Self {
            config,
            current_level: Arc::new(RwLock::new(0.0)),
        }
    }

    /// Calculate degradation level based on usage
    fn calculate_degradation(&self, remaining: u64, limit: u64) -> f64 {
        if limit == 0 {
            return self.config.max_level;
        }

        let usage_percent = 1.0 - (remaining as f64 / limit as f64);

        if usage_percent < self.config.threshold {
            0.0
        } else {
            let excess = usage_percent - self.config.threshold;
            let max_excess = 1.0 - self.config.threshold;
            let degradation_percent = excess / max_excess;

            self.config.initial_level
                + (self.config.max_level - self.config.initial_level) * degradation_percent
        }
    }

    /// Get disabled features for current level
    pub async fn get_disabled_features(&self) -> Vec<String> {
        let level = *self.current_level.read().await;
        let tier = (level / (1.0 / self.config.disabled_features.len() as f64)).floor() as usize;

        self.config
            .disabled_features
            .iter()
            .take(tier.min(self.config.disabled_features.len()))
            .flat_map(|features| features.clone())
            .collect()
    }
}

#[async_trait]
impl ThrottlingPolicy for DegradedServicePolicy {
    async fn handle(&self, decision: Decision, _priority: u32) -> RateLimitResult<ThrottleAction> {
        match decision {
            Decision::Allowed { remaining, .. } => {
                // Calculate degradation based on remaining capacity
                let limit = remaining + 1; // Approximate
                let level = self.calculate_degradation(remaining, limit);

                *self.current_level.write().await = level;

                if level > 0.0 {
                    Ok(ThrottleAction::Degrade { level })
                } else {
                    Ok(ThrottleAction::Allow)
                }
            }
            Decision::Denied { limit, .. } => {
                // Maximum degradation
                *self.current_level.write().await = self.config.max_level;

                Ok(ThrottleAction::Degrade {
                    level: self.config.max_level,
                })
            }
        }
    }

    fn policy_type(&self) -> PolicyType {
        PolicyType::Degrade
    }

    async fn reset(&self) -> RateLimitResult<()> {
        *self.current_level.write().await = 0.0;
        Ok(())
    }
}

// ============================================================================
// Priority Queue Policy
// ============================================================================

/// Request with priority
#[derive(Debug, Clone)]
struct PriorityRequest {
    /// Request ID
    id: String,
    /// Priority (higher = more important)
    priority: u32,
    /// Timestamp
    timestamp: SystemTime,
}

impl PartialEq for PriorityRequest {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.timestamp == other.timestamp
    }
}

impl Eq for PriorityRequest {}

impl PartialOrd for PriorityRequest {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityRequest {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first, then earlier timestamp
        match other.priority.cmp(&self.priority) {
            Ordering::Equal => self.timestamp.cmp(&other.timestamp),
            other_order => other_order,
        }
    }
}

/// Priority queue configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityQueueConfig {
    /// Maximum queue size
    pub max_queue_size: usize,
    /// Processing rate (requests per second)
    pub processing_rate: f64,
    /// Timeout for queued requests
    pub queue_timeout: Duration,
}

impl Default for PriorityQueueConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 1000,
            processing_rate: 10.0,
            queue_timeout: Duration::from_secs(30),
        }
    }
}

/// Priority queue policy
///
/// Queues requests with priority ordering when rate limit exceeded
pub struct PriorityQueuePolicy {
    /// Configuration
    config: PriorityQueueConfig,
    /// Priority queue
    queue: Arc<RwLock<BinaryHeap<PriorityRequest>>>,
    /// Processing semaphore
    semaphore: Arc<Semaphore>,
}

impl PriorityQueuePolicy {
    /// Create a new priority queue policy
    pub fn new(config: PriorityQueueConfig) -> Self {
        let permits = (config.processing_rate as usize).max(1);

        Self {
            config,
            queue: Arc::new(RwLock::new(BinaryHeap::new())),
            semaphore: Arc::new(Semaphore::new(permits)),
        }
    }

    /// Add request to queue
    async fn enqueue(&self, priority: u32) -> RateLimitResult<ThrottleAction> {
        let mut queue = self.queue.write().await;

        if queue.len() >= self.config.max_queue_size {
            return Err(RateLimitError::Exceeded(
                self.config.queue_timeout.as_secs(),
            ));
        }

        let request = PriorityRequest {
            id: uuid::Uuid::new_v4().to_string(),
            priority,
            timestamp: SystemTime::now(),
        };

        queue.push(request);

        let position = queue.len();
        let wait_time =
            Duration::from_secs_f64(position as f64 / self.config.processing_rate);

        Ok(ThrottleAction::Queue {
            position,
            wait_time,
        })
    }

    /// Process next request from queue
    pub async fn process_next(&self) -> RateLimitResult<Option<String>> {
        let _permit = self.semaphore.acquire().await.map_err(|e| {
            RateLimitError::Internal(format!("Failed to acquire semaphore: {}", e))
        })?;

        let mut queue = self.queue.write().await;

        if let Some(request) = queue.pop() {
            // Check if request has timed out
            let elapsed = SystemTime::now()
                .duration_since(request.timestamp)
                .unwrap_or(Duration::ZERO);

            if elapsed > self.config.queue_timeout {
                // Request timed out, skip it
                return Ok(None);
            }

            Ok(Some(request.id))
        } else {
            Ok(None)
        }
    }

    /// Get current queue size
    pub async fn queue_size(&self) -> usize {
        self.queue.read().await.len()
    }
}

#[async_trait]
impl ThrottlingPolicy for PriorityQueuePolicy {
    async fn handle(&self, decision: Decision, priority: u32) -> RateLimitResult<ThrottleAction> {
        match decision {
            Decision::Allowed { .. } => {
                // Try to acquire semaphore
                if let Ok(_permit) = self.semaphore.try_acquire() {
                    Ok(ThrottleAction::Allow)
                } else {
                    // Queue is full, enqueue request
                    self.enqueue(priority).await
                }
            }
            Decision::Denied { .. } => {
                // Always enqueue denied requests
                self.enqueue(priority).await
            }
        }
    }

    fn policy_type(&self) -> PolicyType {
        PolicyType::PriorityQueue
    }

    async fn reset(&self) -> RateLimitResult<()> {
        self.queue.write().await.clear();
        Ok(())
    }
}

// ============================================================================
// Adaptive Policy
// ============================================================================

/// Adaptive policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveConfig {
    /// Load thresholds for switching policies
    pub thresholds: Vec<f64>,
    /// Policies to use at each threshold
    pub policies: Vec<PolicyType>,
    /// Sampling window for load calculation
    pub sample_window: Duration,
}

impl Default for AdaptiveConfig {
    fn default() -> Self {
        Self {
            thresholds: vec![0.5, 0.8, 0.95],
            policies: vec![
                PolicyType::Reject,
                PolicyType::Delay,
                PolicyType::Degrade,
                PolicyType::PriorityQueue,
            ],
            sample_window: Duration::from_secs(60),
        }
    }
}

/// Adaptive throttling policy
///
/// Switches between different policies based on current load
pub struct AdaptivePolicy {
    /// Configuration
    config: AdaptiveConfig,
    /// Current load (0.0 to 1.0)
    current_load: Arc<RwLock<f64>>,
    /// Load history for smoothing
    load_history: Arc<RwLock<VecDeque<(SystemTime, f64)>>>,
    /// Underlying policies
    reject_policy: Arc<RejectPolicy>,
    delay_policy: Arc<DelayPolicy>,
    degrade_policy: Arc<DegradedServicePolicy>,
    queue_policy: Arc<PriorityQueuePolicy>,
}

impl AdaptivePolicy {
    /// Create a new adaptive policy
    pub fn new(config: AdaptiveConfig) -> Self {
        Self {
            config,
            current_load: Arc::new(RwLock::new(0.0)),
            load_history: Arc::new(RwLock::new(VecDeque::new())),
            reject_policy: Arc::new(RejectPolicy::new(None)),
            delay_policy: Arc::new(DelayPolicy::new(Duration::from_secs(5))),
            degrade_policy: Arc::new(DegradedServicePolicy::new(DegradationConfig::default())),
            queue_policy: Arc::new(PriorityQueuePolicy::new(PriorityQueueConfig::default())),
        }
    }

    /// Update load measurement
    async fn update_load(&self, decision: &Decision) {
        let load = match decision {
            Decision::Allowed { remaining, .. } => {
                if *remaining > 0 {
                    1.0 - (*remaining as f64 / (*remaining as f64 + 1.0))
                } else {
                    1.0
                }
            }
            Decision::Denied { .. } => 1.0,
        };

        // Add to history
        let now = SystemTime::now();
        let mut history = self.load_history.write().await;
        history.push_back((now, load));

        // Remove old samples
        let cutoff = now - self.config.sample_window;
        while let Some((timestamp, _)) = history.front() {
            if *timestamp < cutoff {
                history.pop_front();
            } else {
                break;
            }
        }

        // Calculate average load
        if !history.is_empty() {
            let avg_load: f64 = history.iter().map(|(_, l)| l).sum::<f64>() / history.len() as f64;
            *self.current_load.write().await = avg_load;
        }
    }

    /// Select policy based on current load
    async fn select_policy(&self) -> PolicyType {
        let load = *self.current_load.read().await;

        for (i, &threshold) in self.config.thresholds.iter().enumerate() {
            if load < threshold {
                return self.config.policies.get(i).copied().unwrap_or(PolicyType::Reject);
            }
        }

        // Highest load - use last policy
        self.config.policies.last().copied().unwrap_or(PolicyType::PriorityQueue)
    }
}

#[async_trait]
impl ThrottlingPolicy for AdaptivePolicy {
    async fn handle(&self, decision: Decision, priority: u32) -> RateLimitResult<ThrottleAction> {
        // Update load measurement
        self.update_load(&decision).await;

        // Select appropriate policy
        let policy_type = self.select_policy().await;

        // Delegate to selected policy
        match policy_type {
            PolicyType::Reject => self.reject_policy.handle(decision, priority).await,
            PolicyType::Delay => self.delay_policy.handle(decision, priority).await,
            PolicyType::Degrade => self.degrade_policy.handle(decision, priority).await,
            PolicyType::PriorityQueue => self.queue_policy.handle(decision, priority).await,
            PolicyType::Adaptive => Ok(ThrottleAction::Allow), // Shouldn't happen
        }
    }

    fn policy_type(&self) -> PolicyType {
        PolicyType::Adaptive
    }

    async fn reset(&self) -> RateLimitResult<()> {
        *self.current_load.write().await = 0.0;
        self.load_history.write().await.clear();
        self.reject_policy.reset().await?;
        self.delay_policy.reset().await?;
        self.degrade_policy.reset().await?;
        self.queue_policy.reset().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_reject_policy() {
        let policy = RejectPolicy::new(None);

        let allowed = Decision::Allowed {
            remaining: 10,
            reset_after: 60,
        };
        let action = policy.handle(allowed, 0).await.unwrap();
        assert!(matches!(action, ThrottleAction::Allow));

        let denied = Decision::Denied {
            retry_after: 30,
            limit: 100,
        };
        let action = policy.handle(denied, 0).await.unwrap();
        assert!(matches!(action, ThrottleAction::Reject { .. }));
    }

    #[tokio::test]
    async fn test_delay_policy() {
        let policy = DelayPolicy::new(Duration::from_secs(10));

        let denied = Decision::Denied {
            retry_after: 1, // 1 second delay
            limit: 100,
        };

        let start = SystemTime::now();
        let action = policy.handle(denied, 0).await.unwrap();
        let elapsed = SystemTime::now().duration_since(start).unwrap();

        // Should have delayed and then allowed
        assert!(matches!(action, ThrottleAction::Allow));
        assert!(elapsed >= Duration::from_millis(900)); // Allow some timing variance
    }

    #[tokio::test]
    async fn test_degraded_service_policy() {
        let config = DegradationConfig::default();
        let policy = DegradedServicePolicy::new(config);

        // High remaining capacity - no degradation
        let allowed = Decision::Allowed {
            remaining: 90,
            reset_after: 60,
        };
        let action = policy.handle(allowed, 0).await.unwrap();
        assert!(matches!(action, ThrottleAction::Allow));

        // Low remaining capacity - degradation
        let allowed = Decision::Allowed {
            remaining: 5,
            reset_after: 60,
        };
        let action = policy.handle(allowed, 0).await.unwrap();
        assert!(matches!(action, ThrottleAction::Degrade { .. }));
    }

    #[tokio::test]
    async fn test_priority_queue_policy() {
        let config = PriorityQueueConfig {
            max_queue_size: 10,
            processing_rate: 10.0,
            queue_timeout: Duration::from_secs(30),
        };
        let policy = PriorityQueuePolicy::new(config);

        // Deny some requests - should queue them
        for priority in 0..5 {
            let denied = Decision::Denied {
                retry_after: 10,
                limit: 100,
            };
            let action = policy.handle(denied, priority).await.unwrap();
            assert!(matches!(action, ThrottleAction::Queue { .. }));
        }

        assert_eq!(policy.queue_size().await, 5);

        // Process requests - higher priority first
        let processed = policy.process_next().await.unwrap();
        assert!(processed.is_some());
    }

    #[tokio::test]
    async fn test_priority_request_ordering() {
        let mut heap = BinaryHeap::new();

        let low_priority = PriorityRequest {
            id: "low".to_string(),
            priority: 1,
            timestamp: SystemTime::now(),
        };

        let high_priority = PriorityRequest {
            id: "high".to_string(),
            priority: 10,
            timestamp: SystemTime::now(),
        };

        heap.push(low_priority);
        heap.push(high_priority);

        // Should pop high priority first
        let first = heap.pop().unwrap();
        assert_eq!(first.id, "high");
    }

    #[tokio::test]
    async fn test_adaptive_policy() {
        let config = AdaptiveConfig::default();
        let policy = AdaptivePolicy::new(config);

        // Low load - should use reject policy
        let allowed = Decision::Allowed {
            remaining: 80,
            reset_after: 60,
        };
        let _action = policy.handle(allowed, 0).await.unwrap();

        let selected = policy.select_policy().await;
        // With low load, should select early policy
        assert!(matches!(selected, PolicyType::Reject | PolicyType::Delay));
    }
}
