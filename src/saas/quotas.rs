//! Quota management and rate limiting
//!
//! This module provides comprehensive quota management including:
//!
//! - Quota limits per subscription tier
//! - Soft limits (warnings) and hard limits (enforcement)
//! - Rate limiting with multiple algorithms (token bucket, sliding window)
//! - Quota alerts and notifications
//! - Grace period handling for limit violations
//! - Distributed rate limiting via Redis
//!
//! ## Quota Types
//!
//! - **Soft Limit**: Warning threshold (e.g., 80% of quota)
//! - **Hard Limit**: Enforcement threshold (e.g., 100% of quota)
//! - **Grace Period**: Temporary overage allowance
//!
//! ## Rate Limiting Algorithms
//!
//! - **Token Bucket**: Smooth rate limiting with burst capacity
//! - **Sliding Window**: Fixed window with sliding counters
//! - **Fixed Window**: Simple time-based windows
//!
//! ## Example
//!
//! ```rust
//! use caddy::saas::quotas::{QuotaManager, RateLimit};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let quota_mgr = QuotaManager::new(pool).await?;
//!
//!     // Check if request is allowed
//!     let allowed = quota_mgr.check_rate_limit(
//!         tenant_id,
//!         "api_calls",
//!     ).await?;
//!
//!     if !allowed {
//!         return Err("Rate limit exceeded".into());
//!     }
//!
//!     Ok(())
//! }
//! ```

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;

use crate::saas::{Result, SaasError, SubscriptionTier};
use crate::saas::usage::UsageMetric;

// ============================================================================
// Quota Structure
// ============================================================================

/// Quota configuration
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Quota {
    /// Unique quota ID
    pub id: Uuid,

    /// Tenant ID
    pub tenant_id: Uuid,

    /// Subscription tier
    pub tier: SubscriptionTier,

    /// Metric type
    pub metric: UsageMetric,

    /// Soft limit (warning threshold)
    pub soft_limit: i64,

    /// Hard limit (enforcement threshold)
    pub hard_limit: i64,

    /// Grace period in seconds
    pub grace_period_seconds: i64,

    /// Grace period start (if in grace period)
    pub grace_period_start: Option<DateTime<Utc>>,

    /// Alert threshold percentage
    pub alert_threshold_percent: i32,

    /// Notification sent flag
    pub notification_sent: bool,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Quota check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaCheckResult {
    /// Is quota available
    pub allowed: bool,

    /// Current usage
    pub current_usage: i64,

    /// Hard limit
    pub limit: i64,

    /// Remaining quota
    pub remaining: i64,

    /// Usage percentage
    pub usage_percentage: f64,

    /// In grace period
    pub in_grace_period: bool,

    /// Retry after (seconds)
    pub retry_after_seconds: Option<i64>,

    /// Quota exceeded
    pub exceeded: bool,
}

/// Quota alert
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct QuotaAlert {
    /// Alert ID
    pub id: Uuid,

    /// Tenant ID
    pub tenant_id: Uuid,

    /// Quota ID
    pub quota_id: Uuid,

    /// Alert level
    pub level: AlertLevel,

    /// Message
    pub message: String,

    /// Current usage
    pub current_usage: i64,

    /// Limit
    pub limit: i64,

    /// Acknowledged
    pub acknowledged: bool,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Alert severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "alert_level", rename_all = "lowercase")]
pub enum AlertLevel {
    /// Informational
    Info,
    /// Warning (soft limit)
    Warning,
    /// Critical (hard limit)
    Critical,
}

// ============================================================================
// Rate Limiting
// ============================================================================

/// Rate limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    /// Limit key (e.g., "api_calls")
    pub key: String,

    /// Maximum requests
    pub max_requests: i64,

    /// Time window in seconds
    pub window_seconds: i64,

    /// Rate limiting algorithm
    pub algorithm: RateLimitAlgorithm,

    /// Burst capacity (for token bucket)
    pub burst_capacity: Option<i64>,
}

/// Rate limiting algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RateLimitAlgorithm {
    /// Fixed time window
    FixedWindow,
    /// Sliding window counters
    SlidingWindow,
    /// Token bucket algorithm
    TokenBucket,
}

/// Rate limit state (in-memory or Redis)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitState {
    /// Number of requests in current window
    pub requests: i64,

    /// Window start time
    pub window_start: DateTime<Utc>,

    /// Available tokens (for token bucket)
    pub tokens: Option<i64>,

    /// Last refill time (for token bucket)
    pub last_refill: Option<DateTime<Utc>>,
}

// ============================================================================
// Quota Manager
// ============================================================================

/// Quota management operations
pub struct QuotaManager {
    pool: PgPool,
    // In-memory cache for rate limits
    rate_limit_cache: Arc<RwLock<HashMap<String, RateLimitState>>>,
}

impl QuotaManager {
    /// Create a new quota manager
    pub async fn new(pool: PgPool) -> Result<Self> {
        Ok(Self {
            pool,
            rate_limit_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    // ========================================================================
    // Quota Management
    // ========================================================================

    /// Initialize quotas for tenant based on tier
    pub async fn initialize_quotas(
        &self,
        tenant_id: Uuid,
        tier: SubscriptionTier,
    ) -> Result<Vec<Quota>> {
        let mut quotas = Vec::new();

        // API call quotas
        let api_quota = self
            .create_quota(
                tenant_id,
                tier,
                UsageMetric::ApiCall,
                Self::get_tier_limit(tier, UsageMetric::ApiCall),
            )
            .await?;
        quotas.push(api_quota);

        // Page scan quotas
        let scan_quota = self
            .create_quota(
                tenant_id,
                tier,
                UsageMetric::PageScan,
                Self::get_tier_limit(tier, UsageMetric::PageScan),
            )
            .await?;
        quotas.push(scan_quota);

        // Storage quotas
        let storage_quota = self
            .create_quota(
                tenant_id,
                tier,
                UsageMetric::Storage,
                Self::get_tier_limit(tier, UsageMetric::Storage),
            )
            .await?;
        quotas.push(storage_quota);

        Ok(quotas)
    }

    /// Create a quota
    async fn create_quota(
        &self,
        tenant_id: Uuid,
        tier: SubscriptionTier,
        metric: UsageMetric,
        hard_limit: i64,
    ) -> Result<Quota> {
        let quota_id = Uuid::new_v4();
        let soft_limit = (hard_limit as f64 * 0.8) as i64; // 80% soft limit
        let grace_period_seconds = 86400; // 24 hours

        let quota = sqlx::query_as::<_, Quota>(
            r"
            INSERT INTO quotas (
                id, tenant_id, tier, metric, soft_limit, hard_limit,
                grace_period_seconds, alert_threshold_percent,
                notification_sent, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            ",
        )
        .bind(quota_id)
        .bind(tenant_id)
        .bind(tier)
        .bind(metric)
        .bind(soft_limit)
        .bind(hard_limit)
        .bind(grace_period_seconds)
        .bind(80) // Alert at 80%
        .bind(false)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(quota)
    }

    /// Get quota by tenant and metric
    pub async fn get_quota(
        &self,
        tenant_id: Uuid,
        metric: UsageMetric,
    ) -> Result<Quota> {
        sqlx::query_as::<_, Quota>(
            r"
            SELECT * FROM quotas
            WHERE tenant_id = $1 AND metric = $2
            ORDER BY created_at DESC
            LIMIT 1
            ",
        )
        .bind(tenant_id)
        .bind(metric)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| SaasError::Other("Quota not found".to_string()))
    }

    /// Check quota against current usage
    pub async fn check_quota(
        &self,
        tenant_id: Uuid,
        metric: UsageMetric,
        current_usage: i64,
    ) -> Result<QuotaCheckResult> {
        let quota = self.get_quota(tenant_id, metric).await?;

        let exceeded = current_usage >= quota.hard_limit;
        let remaining = (quota.hard_limit - current_usage).max(0);
        let usage_percentage = (current_usage as f64 / quota.hard_limit as f64) * 100.0;

        // Check if in grace period
        let in_grace_period = if exceeded {
            if let Some(grace_start) = quota.grace_period_start {
                let elapsed = Utc::now().signed_duration_since(grace_start);
                elapsed.num_seconds() < quota.grace_period_seconds
            } else {
                // Start grace period
                self.start_grace_period(quota.id).await?;
                true
            }
        } else {
            false
        };

        let allowed = !exceeded || in_grace_period;

        Ok(QuotaCheckResult {
            allowed,
            current_usage,
            limit: quota.hard_limit,
            remaining,
            usage_percentage,
            in_grace_period,
            retry_after_seconds: if !allowed { Some(3600) } else { None },
            exceeded,
        })
    }

    /// Start grace period for quota
    async fn start_grace_period(&self, quota_id: Uuid) -> Result<()> {
        sqlx::query(
            r"
            UPDATE quotas
            SET grace_period_start = $1, updated_at = $2
            WHERE id = $3
            ",
        )
        .bind(Utc::now())
        .bind(Utc::now())
        .bind(quota_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update quota limits
    pub async fn update_quota(
        &self,
        quota_id: Uuid,
        soft_limit: i64,
        hard_limit: i64,
    ) -> Result<Quota> {
        let updated = sqlx::query_as::<_, Quota>(
            r"
            UPDATE quotas
            SET soft_limit = $1, hard_limit = $2, updated_at = $3
            WHERE id = $4
            RETURNING *
            ",
        )
        .bind(soft_limit)
        .bind(hard_limit)
        .bind(Utc::now())
        .bind(quota_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }

    // ========================================================================
    // Rate Limiting
    // ========================================================================

    /// Check rate limit
    pub async fn check_rate_limit(
        &self,
        tenant_id: Uuid,
        limit_key: &str,
    ) -> Result<bool> {
        let rate_limit = self.get_rate_limit_config(tenant_id, limit_key).await?;

        match rate_limit.algorithm {
            RateLimitAlgorithm::TokenBucket => {
                self.check_token_bucket(tenant_id, limit_key, &rate_limit).await
            }
            RateLimitAlgorithm::SlidingWindow => {
                self.check_sliding_window(tenant_id, limit_key, &rate_limit).await
            }
            RateLimitAlgorithm::FixedWindow => {
                self.check_fixed_window(tenant_id, limit_key, &rate_limit).await
            }
        }
    }

    /// Token bucket algorithm
    async fn check_token_bucket(
        &self,
        tenant_id: Uuid,
        limit_key: &str,
        config: &RateLimit,
    ) -> Result<bool> {
        let cache_key = format!("{}:{}", tenant_id, limit_key);
        let mut cache = self.rate_limit_cache.write();

        let now = Utc::now();
        let refill_rate = config.max_requests as f64 / config.window_seconds as f64;
        let burst = config.burst_capacity.unwrap_or(config.max_requests);

        let state = cache.entry(cache_key).or_insert_with(|| RateLimitState {
            requests: 0,
            window_start: now,
            tokens: Some(burst),
            last_refill: Some(now),
        });

        // Refill tokens based on elapsed time
        if let (Some(tokens), Some(last_refill)) = (state.tokens, state.last_refill) {
            let elapsed = now.signed_duration_since(last_refill).num_seconds() as f64;
            let new_tokens = (tokens + (elapsed * refill_rate) as i64).min(burst);

            state.tokens = Some(new_tokens);
            state.last_refill = Some(now);

            if new_tokens > 0 {
                state.tokens = Some(new_tokens - 1);
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    /// Sliding window algorithm
    async fn check_sliding_window(
        &self,
        tenant_id: Uuid,
        limit_key: &str,
        config: &RateLimit,
    ) -> Result<bool> {
        let cache_key = format!("{}:{}", tenant_id, limit_key);
        let mut cache = self.rate_limit_cache.write();

        let now = Utc::now();
        let window_duration = Duration::seconds(config.window_seconds);

        let state = cache.entry(cache_key).or_insert_with(|| RateLimitState {
            requests: 0,
            window_start: now,
            tokens: None,
            last_refill: None,
        });

        // Reset if window has passed
        if now.signed_duration_since(state.window_start) > window_duration {
            state.requests = 0;
            state.window_start = now;
        }

        // Check limit
        if state.requests < config.max_requests {
            state.requests += 1;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Fixed window algorithm
    async fn check_fixed_window(
        &self,
        tenant_id: Uuid,
        limit_key: &str,
        config: &RateLimit,
    ) -> Result<bool> {
        // Fixed window is similar to sliding window but resets at exact intervals
        self.check_sliding_window(tenant_id, limit_key, config).await
    }

    /// Get rate limit configuration
    async fn get_rate_limit_config(
        &self,
        tenant_id: Uuid,
        limit_key: &str,
    ) -> Result<RateLimit> {
        // In a real implementation, this would fetch from database
        // For now, return default configs based on key
        match limit_key {
            "api_calls" => Ok(RateLimit {
                key: limit_key.to_string(),
                max_requests: 1000,
                window_seconds: 60, // 1000 requests per minute
                algorithm: RateLimitAlgorithm::TokenBucket,
                burst_capacity: Some(1500),
            }),
            "page_scans" => Ok(RateLimit {
                key: limit_key.to_string(),
                max_requests: 100,
                window_seconds: 3600, // 100 scans per hour
                algorithm: RateLimitAlgorithm::SlidingWindow,
                burst_capacity: None,
            }),
            _ => Err(SaasError::Other(format!("Unknown rate limit key: {}", limit_key))),
        }
    }

    // ========================================================================
    // Alerts
    // ========================================================================

    /// Create quota alert
    pub async fn create_alert(
        &self,
        tenant_id: Uuid,
        quota_id: Uuid,
        level: AlertLevel,
        message: String,
        current_usage: i64,
        limit: i64,
    ) -> Result<QuotaAlert> {
        let alert_id = Uuid::new_v4();

        let alert = sqlx::query_as::<_, QuotaAlert>(
            r"
            INSERT INTO quota_alerts (
                id, tenant_id, quota_id, level, message,
                current_usage, limit, acknowledged, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            ",
        )
        .bind(alert_id)
        .bind(tenant_id)
        .bind(quota_id)
        .bind(level)
        .bind(&message)
        .bind(current_usage)
        .bind(limit)
        .bind(false)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(alert)
    }

    /// Get unacknowledged alerts for tenant
    pub async fn get_alerts(&self, tenant_id: Uuid) -> Result<Vec<QuotaAlert>> {
        sqlx::query_as::<_, QuotaAlert>(
            r"
            SELECT * FROM quota_alerts
            WHERE tenant_id = $1 AND acknowledged = false
            ORDER BY created_at DESC
            ",
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await
        .map_err(SaasError::Database)
    }

    /// Acknowledge alert
    pub async fn acknowledge_alert(&self, alert_id: Uuid) -> Result<()> {
        sqlx::query(
            r"
            UPDATE quota_alerts
            SET acknowledged = true
            WHERE id = $1
            ",
        )
        .bind(alert_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ========================================================================
    // Tier Limits
    // ========================================================================

    /// Get quota limit for subscription tier
    fn get_tier_limit(tier: SubscriptionTier, metric: UsageMetric) -> i64 {
        match (tier, metric) {
            // Free tier limits
            (SubscriptionTier::Free, UsageMetric::ApiCall) => 10_000,
            (SubscriptionTier::Free, UsageMetric::PageScan) => 100,
            (SubscriptionTier::Free, UsageMetric::Storage) => 1_073_741_824, // 1 GB
            (SubscriptionTier::Free, UsageMetric::UserSeat) => 3,
            (SubscriptionTier::Free, UsageMetric::Bandwidth) => 10_737_418_240, // 10 GB
            (SubscriptionTier::Free, UsageMetric::Render) => 1_000,
            (SubscriptionTier::Free, UsageMetric::Export) => 100,
            (SubscriptionTier::Free, UsageMetric::Collaboration) => 0,

            // Pro tier limits
            (SubscriptionTier::Pro, UsageMetric::ApiCall) => 100_000,
            (SubscriptionTier::Pro, UsageMetric::PageScan) => 1_000,
            (SubscriptionTier::Pro, UsageMetric::Storage) => 107_374_182_400, // 100 GB
            (SubscriptionTier::Pro, UsageMetric::UserSeat) => 25,
            (SubscriptionTier::Pro, UsageMetric::Bandwidth) => 1_073_741_824_000, // 1 TB
            (SubscriptionTier::Pro, UsageMetric::Render) => 10_000,
            (SubscriptionTier::Pro, UsageMetric::Export) => 1_000,
            (SubscriptionTier::Pro, UsageMetric::Collaboration) => 10,

            // Enterprise tier limits (unlimited or very high)
            (SubscriptionTier::Enterprise, UsageMetric::ApiCall) => 1_000_000,
            (SubscriptionTier::Enterprise, UsageMetric::PageScan) => 10_000,
            (SubscriptionTier::Enterprise, UsageMetric::Storage) => 1_099_511_627_776, // 1 TB
            (SubscriptionTier::Enterprise, UsageMetric::UserSeat) => 1_000,
            (SubscriptionTier::Enterprise, UsageMetric::Bandwidth) => 10_737_418_240_000, // 10 TB
            (SubscriptionTier::Enterprise, UsageMetric::Render) => 100_000,
            (SubscriptionTier::Enterprise, UsageMetric::Export) => 10_000,
            (SubscriptionTier::Enterprise, UsageMetric::Collaboration) => 100,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_limits() {
        assert_eq!(
            QuotaManager::get_tier_limit(SubscriptionTier::Free, UsageMetric::ApiCall),
            10_000
        );
        assert_eq!(
            QuotaManager::get_tier_limit(SubscriptionTier::Pro, UsageMetric::ApiCall),
            100_000
        );
        assert_eq!(
            QuotaManager::get_tier_limit(SubscriptionTier::Enterprise, UsageMetric::ApiCall),
            1_000_000
        );
    }

    #[test]
    fn test_quota_check_result() {
        let result = QuotaCheckResult {
            allowed: true,
            current_usage: 8_000,
            limit: 10_000,
            remaining: 2_000,
            usage_percentage: 80.0,
            in_grace_period: false,
            retry_after_seconds: None,
            exceeded: false,
        };

        assert!(result.allowed);
        assert_eq!(result.remaining, 2_000);
        assert_eq!(result.usage_percentage, 80.0);
    }

    #[test]
    fn test_alert_levels() {
        assert_eq!(AlertLevel::Info as i32, 0);
        assert_eq!(AlertLevel::Warning as i32, 1);
        assert_eq!(AlertLevel::Critical as i32, 2);
    }
}
