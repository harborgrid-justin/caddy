//! # Rate Limiting & Throttling Module
//!
//! This module provides comprehensive rate limiting and throttling capabilities for CADDY,
//! including multiple algorithms, distributed rate limiting, quota management, and analytics.
//!
//! ## Features
//!
//! - **Multiple Algorithms**: Token bucket, leaky bucket, sliding window, and GCRA
//! - **Distributed Rate Limiting**: Redis-backed coordination across multiple instances
//! - **Quota Management**: Per-user, per-API-key, and per-tenant quotas with inheritance
//! - **Throttling Policies**: Reject, delay, degrade, and priority queue policies
//! - **HTTP Headers**: Standardized rate limit headers (X-RateLimit-*, IETF, GitHub, Twitter)
//! - **Analytics**: Event tracking, abuse detection, and anomaly alerting
//!
//! ## Quick Start
//!
//! ```rust
//! use caddy::enterprise::ratelimit::{
//!     algorithm::{TokenBucket, Decision},
//!     quota::{QuotaManager, QuotaIdentifier, QuotaLimits, QuotaPeriod},
//!     policy::{RejectPolicy, ThrottlingPolicy},
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a token bucket rate limiter
//! let bucket = TokenBucket::new(100, 10.0); // 100 capacity, 10 tokens/sec
//!
//! // Check if request is allowed
//! let decision = bucket.check(1);
//! if decision.is_allowed() {
//!     println!("Request allowed!");
//! }
//!
//! // Use quota manager
//! let quota_manager = QuotaManager::new();
//! quota_manager.set_default_limits(
//!     "api_call".to_string(),
//!     QuotaLimits::new(1000, QuotaPeriod::Hour),
//! ).await?;
//!
//! let user_id = QuotaIdentifier::User("user123".to_string());
//! let decision = quota_manager.check(&user_id, "api_call", 1).await?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Architecture
//!
//! The rate limiting system is organized into several layers:
//!
//! 1. **Algorithm Layer** (`algorithm`): Core rate limiting algorithms
//! 2. **Distribution Layer** (`distributed`): Multi-instance coordination
//! 3. **Management Layer** (`quota`): Quota configuration and tracking
//! 4. **Policy Layer** (`policy`): Request handling strategies
//! 5. **Presentation Layer** (`headers`): HTTP header generation
//! 6. **Analytics Layer** (`analytics`): Monitoring and abuse detection

use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};

// ============================================================================
// Module Declarations
// ============================================================================

/// Rate limiting algorithms
pub mod algorithm;

/// Distributed rate limiting
pub mod distributed;

/// Quota management
pub mod quota;

/// Throttling policies
pub mod policy;

/// HTTP headers
pub mod headers;

/// Analytics and tracking
pub mod analytics;

// ============================================================================
// Re-exports
// ============================================================================

// Algorithm re-exports
pub use algorithm::{
    AlgorithmConfig, AlgorithmType, Decision, GCRA, LeakyBucket, RateLimitError,
    RateLimitResult, SlidingWindowCounter, SlidingWindowLog, TokenBucket,
};

// Distributed re-exports
pub use distributed::{
    ConsistentHashRing, DistributedLock, DistributedRateLimiter, OptimisticLock,
    RedisConfig, RedisRateLimiter,
};

// Quota re-exports
pub use quota::{
    HierarchicalQuotaBuilder, QuotaConfig, QuotaIdentifier, QuotaLimits, QuotaManager,
    QuotaPeriod, QuotaStatistics, QuotaUsage,
};

// Policy re-exports
pub use policy::{
    AdaptiveConfig, AdaptivePolicy, DegradationConfig, DegradedServicePolicy, DelayPolicy,
    PolicyType, PriorityQueueConfig, PriorityQueuePolicy, RejectPolicy, ThrottleAction,
    ThrottlingPolicy,
};

// Headers re-exports
pub use headers::{
    HeaderStandard, RateLimitHeaderMiddleware, RateLimitHeaders, RateLimitInfo,
    RateLimitResponse, RetryAfterFormat,
};

// Analytics re-exports
pub use analytics::{
    AbuseDetectionConfig, AbuseDetector, AbuseReport, AbuseSeverity, Alert, AlertManager,
    AlertSeverity, AnomalyDetector, AnomalyStatistics, ConsoleAlertManager, EventListener,
    EventType, RateLimitAnalytics, RateLimitEvent, Statistics,
};

// ============================================================================
// Integrated Rate Limiter
// ============================================================================

/// Comprehensive rate limiter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimiterConfig {
    /// Algorithm configuration
    pub algorithm: AlgorithmConfig,
    /// Enable distributed mode
    pub distributed: bool,
    /// Redis configuration (for distributed mode)
    pub redis: Option<RedisConfig>,
    /// Throttling policy
    pub policy_type: PolicyType,
    /// Enable analytics
    pub enable_analytics: bool,
    /// Enable abuse detection
    pub enable_abuse_detection: bool,
    /// Header standard to use
    pub header_standard: HeaderStandard,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            algorithm: AlgorithmConfig::default(),
            distributed: false,
            redis: None,
            policy_type: PolicyType::Reject,
            enable_analytics: true,
            enable_abuse_detection: true,
            header_standard: HeaderStandard::Traditional,
        }
    }
}

/// Integrated rate limiter combining all features
pub struct RateLimiter {
    /// Configuration
    config: RateLimiterConfig,
    /// Quota manager
    quota_manager: Arc<QuotaManager>,
    /// Throttling policy
    policy: Arc<dyn ThrottlingPolicy>,
    /// Analytics (optional)
    analytics: Option<Arc<RateLimitAnalytics>>,
    /// Abuse detector (optional)
    abuse_detector: Option<Arc<AbuseDetector>>,
    /// Distributed limiter (optional)
    distributed: Option<Arc<RedisRateLimiter>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(config: RateLimiterConfig) -> Self {
        let quota_manager = Arc::new(QuotaManager::new());

        // Create policy based on type
        let policy: Arc<dyn ThrottlingPolicy> = match config.policy_type {
            PolicyType::Reject => Arc::new(RejectPolicy::new(None)),
            PolicyType::Delay => Arc::new(DelayPolicy::new(Duration::from_secs(5))),
            PolicyType::Degrade => {
                Arc::new(DegradedServicePolicy::new(DegradationConfig::default()))
            }
            PolicyType::PriorityQueue => {
                Arc::new(PriorityQueuePolicy::new(PriorityQueueConfig::default()))
            }
            PolicyType::Adaptive => Arc::new(AdaptivePolicy::new(AdaptiveConfig::default())),
        };

        // Create analytics if enabled
        let analytics = if config.enable_analytics {
            Some(Arc::new(RateLimitAnalytics::new(
                10000,
                Duration::from_secs(86400),
            )))
        } else {
            None
        };

        // Create abuse detector if enabled
        let abuse_detector = if config.enable_abuse_detection {
            Some(Arc::new(AbuseDetector::new(
                AbuseDetectionConfig::default(),
            )))
        } else {
            None
        };

        // Create distributed limiter if enabled
        let distributed = if config.distributed {
            if let Some(redis_config) = &config.redis {
                Some(Arc::new(RedisRateLimiter::new(
                    redis_config.clone(),
                    config.algorithm.burst,
                    Duration::from_secs(config.algorithm.window_secs),
                )))
            } else {
                None
            }
        } else {
            None
        };

        Self {
            config,
            quota_manager,
            policy,
            analytics,
            abuse_detector,
            distributed,
        }
    }

    /// Check if a request is allowed
    ///
    /// # Arguments
    /// * `identifier` - User/API key/tenant identifier
    /// * `operation` - Operation being performed
    /// * `amount` - Number of tokens/requests to consume
    /// * `priority` - Request priority (for queue policies)
    pub async fn check(
        &self,
        identifier: &QuotaIdentifier,
        operation: &str,
        amount: u64,
        priority: u32,
    ) -> RateLimitResult<CheckResult> {
        // Check if blocked
        if let Some(detector) = &self.abuse_detector {
            if detector.is_blocked(identifier).await {
                return Ok(CheckResult {
                    decision: Decision::Denied {
                        retry_after: 3600,
                        limit: 0,
                    },
                    action: ThrottleAction::Reject {
                        reason: "Blocked due to abuse".to_string(),
                        retry_after: Duration::from_secs(3600),
                    },
                    headers: None,
                });
            }
        }

        // Check quota
        let decision = if let Some(distributed) = &self.distributed {
            // Use distributed limiter
            let key = format!("{}:{}", identifier.to_key(), operation);
            distributed.check(&key, amount).await?
        } else {
            // Use local quota manager
            self.quota_manager.check(identifier, operation, amount).await?
        };

        // Apply throttling policy
        let action = self.policy.handle(decision.clone(), priority).await?;

        // Record analytics
        if let Some(analytics) = &self.analytics {
            let event_type = match &decision {
                Decision::Allowed { .. } => EventType::Allowed,
                Decision::Denied { .. } => EventType::Denied,
            };

            let event = RateLimitEvent::new(
                event_type,
                identifier.clone(),
                operation.to_string(),
                &decision,
            );

            analytics.record(event).await;
        }

        // Check for abuse
        if let Some(detector) = &self.abuse_detector {
            if let Some(analytics) = &self.analytics {
                if let Some(stats) = analytics.get_statistics(identifier, operation).await {
                    detector.analyze(identifier, &stats).await;
                }
            }
        }

        // Generate headers
        let headers = if let Decision::Allowed { remaining, reset_after } = &decision {
            let limit = *remaining + amount;
            let info = RateLimitInfo {
                limit,
                remaining: *remaining,
                reset: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    + *reset_after,
                window: self.config.algorithm.window_secs,
                retry_after: None,
                policy: Some(operation.to_string()),
            };

            Some(RateLimitHeaders::new(self.config.header_standard, info).build())
        } else if let Decision::Denied { retry_after, limit } = &decision {
            let info = RateLimitInfo {
                limit: *limit,
                remaining: 0,
                reset: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    + *retry_after,
                window: self.config.algorithm.window_secs,
                retry_after: Some(*retry_after),
                policy: Some(operation.to_string()),
            };

            Some(RateLimitHeaders::new(self.config.header_standard, info).build())
        } else {
            None
        };

        Ok(CheckResult {
            decision,
            action,
            headers,
        })
    }

    /// Get quota manager
    pub fn quota_manager(&self) -> Arc<QuotaManager> {
        self.quota_manager.clone()
    }

    /// Get analytics
    pub fn analytics(&self) -> Option<Arc<RateLimitAnalytics>> {
        self.analytics.clone()
    }

    /// Get abuse detector
    pub fn abuse_detector(&self) -> Option<Arc<AbuseDetector>> {
        self.abuse_detector.clone()
    }

    /// Reset rate limit for an identifier
    pub async fn reset(&self, identifier: &QuotaIdentifier, operation: Option<&str>) -> RateLimitResult<()> {
        if let Some(distributed) = &self.distributed {
            if let Some(op) = operation {
                let key = format!("{}:{}", identifier.to_key(), op);
                distributed.reset(&key).await?;
            }
        }

        self.quota_manager.reset(identifier, operation).await?;
        Ok(())
    }

    /// Get current usage for an identifier
    pub async fn get_usage(
        &self,
        identifier: &QuotaIdentifier,
        operation: &str,
    ) -> RateLimitResult<QuotaUsage> {
        self.quota_manager.get_usage(identifier, operation).await
    }

    /// Get statistics
    pub async fn get_statistics(&self) -> Option<QuotaStatistics> {
        Some(self.quota_manager.get_statistics().await)
    }
}

/// Result of a rate limit check
#[derive(Debug, Clone)]
pub struct CheckResult {
    /// Rate limit decision
    pub decision: Decision,
    /// Throttle action to take
    pub action: ThrottleAction,
    /// HTTP headers (if applicable)
    pub headers: Option<std::collections::HashMap<String, String>>,
}

impl CheckResult {
    /// Check if request is allowed
    pub fn is_allowed(&self) -> bool {
        matches!(self.action, ThrottleAction::Allow)
    }

    /// Get retry after duration
    pub fn retry_after(&self) -> Option<Duration> {
        match &self.action {
            ThrottleAction::Reject { retry_after, .. } => Some(*retry_after),
            ThrottleAction::Delay { duration } => Some(*duration),
            ThrottleAction::Queue { wait_time, .. } => Some(*wait_time),
            _ => None,
        }
    }
}

// ============================================================================
// Builder Pattern
// ============================================================================

/// Builder for creating a rate limiter
pub struct RateLimiterBuilder {
    config: RateLimiterConfig,
}

impl RateLimiterBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: RateLimiterConfig::default(),
        }
    }

    /// Set algorithm
    pub fn algorithm(mut self, algorithm: AlgorithmConfig) -> Self {
        self.config.algorithm = algorithm;
        self
    }

    /// Enable distributed mode
    pub fn distributed(mut self, redis_config: RedisConfig) -> Self {
        self.config.distributed = true;
        self.config.redis = Some(redis_config);
        self
    }

    /// Set policy type
    pub fn policy(mut self, policy_type: PolicyType) -> Self {
        self.config.policy_type = policy_type;
        self
    }

    /// Set header standard
    pub fn header_standard(mut self, standard: HeaderStandard) -> Self {
        self.config.header_standard = standard;
        self
    }

    /// Enable/disable analytics
    pub fn analytics(mut self, enable: bool) -> Self {
        self.config.enable_analytics = enable;
        self
    }

    /// Enable/disable abuse detection
    pub fn abuse_detection(mut self, enable: bool) -> Self {
        self.config.enable_abuse_detection = enable;
        self
    }

    /// Build the rate limiter
    pub fn build(self) -> RateLimiter {
        RateLimiter::new(self.config)
    }
}

impl Default for RateLimiterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Presets
// ============================================================================

/// Common rate limiter presets
pub struct RateLimiterPresets;

impl RateLimiterPresets {
    /// Create a basic API rate limiter (100 req/min)
    pub fn api() -> RateLimiter {
        RateLimiterBuilder::new()
            .algorithm(AlgorithmConfig {
                algorithm: AlgorithmType::TokenBucket,
                rate: 100.0,
                burst: 100,
                window_secs: 60,
            })
            .policy(PolicyType::Reject)
            .build()
    }

    /// Create a strict rate limiter with delay (10 req/sec, max 5 sec delay)
    pub fn strict() -> RateLimiter {
        RateLimiterBuilder::new()
            .algorithm(AlgorithmConfig {
                algorithm: AlgorithmType::LeakyBucket,
                rate: 10.0,
                burst: 10,
                window_secs: 1,
            })
            .policy(PolicyType::Delay)
            .build()
    }

    /// Create a high-volume rate limiter (10000 req/hour with degradation)
    pub fn high_volume() -> RateLimiter {
        RateLimiterBuilder::new()
            .algorithm(AlgorithmConfig {
                algorithm: AlgorithmType::SlidingWindowCounter,
                rate: 10000.0,
                burst: 12000,
                window_secs: 3600,
            })
            .policy(PolicyType::Degrade)
            .build()
    }

    /// Create a distributed rate limiter with Redis
    pub fn distributed(redis_servers: Vec<String>) -> RateLimiter {
        RateLimiterBuilder::new()
            .algorithm(AlgorithmConfig {
                algorithm: AlgorithmType::TokenBucket,
                rate: 1000.0,
                burst: 1000,
                window_secs: 60,
            })
            .distributed(RedisConfig {
                servers: redis_servers,
                ..Default::default()
            })
            .policy(PolicyType::PriorityQueue)
            .build()
    }

    /// Create an adaptive rate limiter
    pub fn adaptive() -> RateLimiter {
        RateLimiterBuilder::new()
            .algorithm(AlgorithmConfig {
                algorithm: AlgorithmType::GCRA,
                rate: 100.0,
                burst: 150,
                window_secs: 60,
            })
            .policy(PolicyType::Adaptive)
            .analytics(true)
            .abuse_detection(true)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = RateLimiterPresets::api();

        // Set quota
        limiter
            .quota_manager()
            .set_default_limits(
                "test_op".to_string(),
                QuotaLimits::new(10, QuotaPeriod::Minute),
            )
            .await
            .unwrap();

        let user_id = QuotaIdentifier::User("test_user".to_string());

        // Should allow first 10 requests
        for _ in 0..10 {
            let result = limiter.check(&user_id, "test_op", 1, 0).await.unwrap();
            assert!(result.is_allowed());
        }

        // 11th should be denied
        let result = limiter.check(&user_id, "test_op", 1, 0).await.unwrap();
        assert!(!result.is_allowed());
    }

    #[tokio::test]
    async fn test_rate_limiter_builder() {
        let limiter = RateLimiterBuilder::new()
            .algorithm(AlgorithmConfig {
                algorithm: AlgorithmType::TokenBucket,
                rate: 50.0,
                burst: 50,
                window_secs: 60,
            })
            .policy(PolicyType::Reject)
            .header_standard(HeaderStandard::GitHub)
            .analytics(true)
            .build();

        limiter
            .quota_manager()
            .set_default_limits(
                "test".to_string(),
                QuotaLimits::new(5, QuotaPeriod::Minute),
            )
            .await
            .unwrap();

        let user_id = QuotaIdentifier::User("builder_test".to_string());
        let result = limiter.check(&user_id, "test", 1, 0).await.unwrap();

        assert!(result.is_allowed());
        assert!(result.headers.is_some());
    }

    #[tokio::test]
    async fn test_rate_limiter_usage() {
        let limiter = RateLimiterPresets::api();

        limiter
            .quota_manager()
            .set_default_limits(
                "usage_test".to_string(),
                QuotaLimits::new(100, QuotaPeriod::Hour),
            )
            .await
            .unwrap();

        let user_id = QuotaIdentifier::User("usage_user".to_string());

        // Make some requests
        for _ in 0..25 {
            limiter.check(&user_id, "usage_test", 1, 0).await.unwrap();
        }

        // Check usage
        let usage = limiter.get_usage(&user_id, "usage_test").await.unwrap();
        assert_eq!(usage.current, 25);
        assert_eq!(usage.remaining, 75);
    }

    #[tokio::test]
    async fn test_rate_limiter_reset() {
        let limiter = RateLimiterPresets::api();

        limiter
            .quota_manager()
            .set_default_limits(
                "reset_test".to_string(),
                QuotaLimits::new(5, QuotaPeriod::Minute),
            )
            .await
            .unwrap();

        let user_id = QuotaIdentifier::User("reset_user".to_string());

        // Use up quota
        for _ in 0..5 {
            limiter.check(&user_id, "reset_test", 1, 0).await.unwrap();
        }

        // Should be denied
        let result = limiter.check(&user_id, "reset_test", 1, 0).await.unwrap();
        assert!(!result.is_allowed());

        // Reset
        limiter.reset(&user_id, Some("reset_test")).await.unwrap();

        // Should work again
        let result = limiter.check(&user_id, "reset_test", 1, 0).await.unwrap();
        assert!(result.is_allowed());
    }

    #[tokio::test]
    async fn test_presets() {
        let _api = RateLimiterPresets::api();
        let _strict = RateLimiterPresets::strict();
        let _high_volume = RateLimiterPresets::high_volume();
        let _adaptive = RateLimiterPresets::adaptive();

        // All should create successfully
    }
}
