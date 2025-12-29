//! Quota Management System
//!
//! This module provides comprehensive quota management including:
//! - Per-user quotas
//! - Per-API-key quotas
//! - Per-tenant quotas
//! - Hierarchical quota inheritance

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::algorithm::{Decision, RateLimitError, RateLimitResult};

// ============================================================================
// Quota Types and Identifiers
// ============================================================================

/// Quota identifier type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuotaIdentifier {
    /// User-based quota
    User(String),
    /// API key-based quota
    ApiKey(String),
    /// Tenant/organization-based quota
    Tenant(String),
    /// IP address-based quota
    IpAddress(String),
    /// Custom identifier
    Custom(String),
}

impl QuotaIdentifier {
    /// Convert to string key
    pub fn to_key(&self) -> String {
        match self {
            Self::User(id) => format!("user:{}", id),
            Self::ApiKey(key) => format!("apikey:{}", key),
            Self::Tenant(id) => format!("tenant:{}", id),
            Self::IpAddress(ip) => format!("ip:{}", ip),
            Self::Custom(key) => format!("custom:{}", key),
        }
    }
}

/// Quota period
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuotaPeriod {
    /// Per second
    Second,
    /// Per minute
    Minute,
    /// Per hour
    Hour,
    /// Per day
    Day,
    /// Per week
    Week,
    /// Per month (30 days)
    Month,
    /// Per year (365 days)
    Year,
    /// Custom duration
    Custom(u64), // seconds
}

impl QuotaPeriod {
    /// Get duration in seconds
    pub fn as_secs(&self) -> u64 {
        match self {
            Self::Second => 1,
            Self::Minute => 60,
            Self::Hour => 3600,
            Self::Day => 86400,
            Self::Week => 604800,
            Self::Month => 2592000,  // 30 days
            Self::Year => 31536000,  // 365 days
            Self::Custom(secs) => *secs,
        }
    }

    /// Get duration
    pub fn as_duration(&self) -> Duration {
        Duration::from_secs(self.as_secs())
    }
}

// ============================================================================
// Quota Configuration
// ============================================================================

/// Quota limits for a specific resource or operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaLimits {
    /// Maximum requests allowed
    pub max_requests: u64,
    /// Time period
    pub period: QuotaPeriod,
    /// Burst allowance (optional)
    pub burst: Option<u64>,
    /// Soft limit (warning threshold, optional)
    pub soft_limit: Option<u64>,
}

impl QuotaLimits {
    /// Create new quota limits
    pub fn new(max_requests: u64, period: QuotaPeriod) -> Self {
        Self {
            max_requests,
            period,
            burst: None,
            soft_limit: None,
        }
    }

    /// Set burst allowance
    pub fn with_burst(mut self, burst: u64) -> Self {
        self.burst = Some(burst);
        self
    }

    /// Set soft limit
    pub fn with_soft_limit(mut self, soft_limit: u64) -> Self {
        self.soft_limit = Some(soft_limit);
        self
    }

    /// Check if soft limit exceeded
    pub fn is_soft_limit_exceeded(&self, usage: u64) -> bool {
        if let Some(limit) = self.soft_limit {
            usage >= limit
        } else {
            false
        }
    }
}

/// Quota configuration for an identifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaConfig {
    /// Identifier this quota applies to
    pub identifier: QuotaIdentifier,
    /// Quota limits per operation
    pub limits: HashMap<String, QuotaLimits>,
    /// Priority (higher priority quotas override lower)
    pub priority: u32,
    /// Enabled status
    pub enabled: bool,
    /// Parent quota (for inheritance)
    pub parent: Option<QuotaIdentifier>,
}

impl QuotaConfig {
    /// Create a new quota configuration
    pub fn new(identifier: QuotaIdentifier) -> Self {
        Self {
            identifier,
            limits: HashMap::new(),
            priority: 0,
            enabled: true,
            parent: None,
        }
    }

    /// Add a limit for an operation
    pub fn add_limit(mut self, operation: String, limits: QuotaLimits) -> Self {
        self.limits.insert(operation, limits);
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Set parent for inheritance
    pub fn with_parent(mut self, parent: QuotaIdentifier) -> Self {
        self.parent = Some(parent);
        self
    }
}

// ============================================================================
// Quota Usage Tracking
// ============================================================================

/// Current quota usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaUsage {
    /// Identifier
    pub identifier: QuotaIdentifier,
    /// Operation
    pub operation: String,
    /// Current usage count
    pub current: u64,
    /// Maximum allowed
    pub limit: u64,
    /// Remaining quota
    pub remaining: u64,
    /// Period start time
    pub period_start: SystemTime,
    /// Period end time
    pub period_end: SystemTime,
    /// Percentage used
    pub percentage: f64,
}

impl QuotaUsage {
    /// Check if quota is exceeded
    pub fn is_exceeded(&self) -> bool {
        self.current >= self.limit
    }

    /// Check if near limit (>80%)
    pub fn is_near_limit(&self) -> bool {
        self.percentage >= 0.8
    }

    /// Time until reset
    pub fn reset_in(&self) -> Duration {
        self.period_end
            .duration_since(SystemTime::now())
            .unwrap_or(Duration::ZERO)
    }
}

// ============================================================================
// Quota Manager
// ============================================================================

/// Quota manager for tracking and enforcing quotas
pub struct QuotaManager {
    /// Quota configurations
    configs: Arc<DashMap<String, QuotaConfig>>,
    /// Usage tracking
    usage: Arc<DashMap<String, QuotaUsageTracker>>,
    /// Default limits
    default_limits: Arc<RwLock<HashMap<String, QuotaLimits>>>,
}

impl QuotaManager {
    /// Create a new quota manager
    pub fn new() -> Self {
        Self {
            configs: Arc::new(DashMap::new()),
            usage: Arc::new(DashMap::new()),
            default_limits: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a quota configuration
    pub async fn add_config(&self, config: QuotaConfig) -> RateLimitResult<()> {
        let key = config.identifier.to_key();
        self.configs.insert(key, config);
        Ok(())
    }

    /// Remove a quota configuration
    pub async fn remove_config(&self, identifier: &QuotaIdentifier) -> RateLimitResult<()> {
        let key = identifier.to_key();
        self.configs.remove(&key);
        self.usage.remove(&key);
        Ok(())
    }

    /// Set default limits for an operation
    pub async fn set_default_limits(
        &self,
        operation: String,
        limits: QuotaLimits,
    ) -> RateLimitResult<()> {
        self.default_limits.write().await.insert(operation, limits);
        Ok(())
    }

    /// Check quota for an operation
    pub async fn check(
        &self,
        identifier: &QuotaIdentifier,
        operation: &str,
        amount: u64,
    ) -> RateLimitResult<Decision> {
        // Get effective limits (check hierarchy)
        let limits = self.get_effective_limits(identifier, operation).await?;

        // Get or create usage tracker
        let key = format!("{}:{}", identifier.to_key(), operation);
        let tracker = self.usage.entry(key.clone()).or_insert_with(|| {
            QuotaUsageTracker::new(
                identifier.clone(),
                operation.to_string(),
                limits.max_requests,
                limits.period,
            )
        });

        // Check and update
        tracker.check(amount, &limits)
    }

    /// Get current usage
    pub async fn get_usage(
        &self,
        identifier: &QuotaIdentifier,
        operation: &str,
    ) -> RateLimitResult<QuotaUsage> {
        let key = format!("{}:{}", identifier.to_key(), operation);

        if let Some(tracker) = self.usage.get(&key) {
            Ok(tracker.get_usage())
        } else {
            // No usage yet
            let limits = self.get_effective_limits(identifier, operation).await?;
            let now = SystemTime::now();

            Ok(QuotaUsage {
                identifier: identifier.clone(),
                operation: operation.to_string(),
                current: 0,
                limit: limits.max_requests,
                remaining: limits.max_requests,
                period_start: now,
                period_end: now + limits.period.as_duration(),
                percentage: 0.0,
            })
        }
    }

    /// Reset quota for an identifier
    pub async fn reset(
        &self,
        identifier: &QuotaIdentifier,
        operation: Option<&str>,
    ) -> RateLimitResult<()> {
        if let Some(op) = operation {
            // Reset specific operation
            let key = format!("{}:{}", identifier.to_key(), op);
            if let Some(mut tracker) = self.usage.get_mut(&key) {
                tracker.reset();
            }
        } else {
            // Reset all operations for this identifier
            let prefix = identifier.to_key();
            self.usage.retain(|k, _| !k.starts_with(&prefix));
        }

        Ok(())
    }

    /// Get effective limits (with inheritance)
    async fn get_effective_limits(
        &self,
        identifier: &QuotaIdentifier,
        operation: &str,
    ) -> RateLimitResult<QuotaLimits> {
        let key = identifier.to_key();

        // Check direct configuration
        if let Some(config) = self.configs.get(&key) {
            if config.enabled {
                if let Some(limits) = config.limits.get(operation) {
                    return Ok(limits.clone());
                }

                // Check parent
                if let Some(parent) = &config.parent {
                    return Box::pin(self.get_effective_limits(parent, operation)).await;
                }
            }
        }

        // Fall back to default
        let defaults = self.default_limits.read().await;
        if let Some(limits) = defaults.get(operation) {
            Ok(limits.clone())
        } else {
            Err(RateLimitError::InvalidConfig(format!(
                "No quota limits found for operation: {}",
                operation
            )))
        }
    }

    /// List all quotas for an identifier
    pub async fn list_quotas(&self, identifier: &QuotaIdentifier) -> Vec<QuotaUsage> {
        let prefix = identifier.to_key();
        self.usage
            .iter()
            .filter(|entry| entry.key().starts_with(&prefix))
            .map(|entry| entry.get_usage())
            .collect()
    }

    /// Get quota statistics
    pub async fn get_statistics(&self) -> QuotaStatistics {
        let total_quotas = self.configs.len();
        let total_trackers = self.usage.len();

        let exceeded = self
            .usage
            .iter()
            .filter(|entry| entry.get_usage().is_exceeded())
            .count();

        let near_limit = self
            .usage
            .iter()
            .filter(|entry| entry.get_usage().is_near_limit())
            .count();

        QuotaStatistics {
            total_quotas,
            total_trackers,
            exceeded,
            near_limit,
        }
    }
}

impl Default for QuotaManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Quota Usage Tracker
// ============================================================================

/// Tracks quota usage for a specific identifier and operation
#[derive(Debug)]
struct QuotaUsageTracker {
    /// Identifier
    identifier: QuotaIdentifier,
    /// Operation
    operation: String,
    /// Current count
    count: Arc<parking_lot::RwLock<u64>>,
    /// Limit
    limit: u64,
    /// Period
    period: QuotaPeriod,
    /// Period start
    period_start: Arc<parking_lot::RwLock<SystemTime>>,
}

impl QuotaUsageTracker {
    /// Create a new usage tracker
    fn new(
        identifier: QuotaIdentifier,
        operation: String,
        limit: u64,
        period: QuotaPeriod,
    ) -> Self {
        Self {
            identifier,
            operation,
            count: Arc::new(parking_lot::RwLock::new(0)),
            limit,
            period,
            period_start: Arc::new(parking_lot::RwLock::new(SystemTime::now())),
        }
    }

    /// Check if request is allowed
    fn check(&self, amount: u64, limits: &QuotaLimits) -> RateLimitResult<Decision> {
        // Check if period has expired
        self.reset_if_expired();

        let mut count = self.count.write();
        let current = *count;

        let effective_limit = limits.burst.unwrap_or(limits.max_requests);

        if current + amount <= effective_limit {
            *count += amount;

            Ok(Decision::Allowed {
                remaining: effective_limit - *count,
                reset_after: self.time_until_reset(),
            })
        } else {
            Ok(Decision::Denied {
                retry_after: self.time_until_reset(),
                limit: effective_limit,
            })
        }
    }

    /// Reset if period expired
    fn reset_if_expired(&self) {
        let period_start = *self.period_start.read();
        let elapsed = SystemTime::now()
            .duration_since(period_start)
            .unwrap_or(Duration::ZERO);

        if elapsed >= self.period.as_duration() {
            self.reset();
        }
    }

    /// Reset the tracker
    fn reset(&self) {
        *self.count.write() = 0;
        *self.period_start.write() = SystemTime::now();
    }

    /// Get time until reset in seconds
    fn time_until_reset(&self) -> u64 {
        let period_start = *self.period_start.read();
        let period_end = period_start + self.period.as_duration();

        period_end
            .duration_since(SystemTime::now())
            .unwrap_or(Duration::ZERO)
            .as_secs()
            .max(1)
    }

    /// Get current usage
    fn get_usage(&self) -> QuotaUsage {
        self.reset_if_expired();

        let current = *self.count.read();
        let period_start = *self.period_start.read();
        let period_end = period_start + self.period.as_duration();

        QuotaUsage {
            identifier: self.identifier.clone(),
            operation: self.operation.clone(),
            current,
            limit: self.limit,
            remaining: self.limit.saturating_sub(current),
            period_start,
            period_end,
            percentage: current as f64 / self.limit as f64,
        }
    }
}

// ============================================================================
// Quota Statistics
// ============================================================================

/// Quota system statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaStatistics {
    /// Total quota configurations
    pub total_quotas: usize,
    /// Total active trackers
    pub total_trackers: usize,
    /// Quotas currently exceeded
    pub exceeded: usize,
    /// Quotas near limit (>80%)
    pub near_limit: usize,
}

// ============================================================================
// Hierarchical Quotas
// ============================================================================

/// Hierarchical quota builder for complex quota structures
pub struct HierarchicalQuotaBuilder {
    manager: Arc<QuotaManager>,
}

impl HierarchicalQuotaBuilder {
    /// Create a new builder
    pub fn new(manager: Arc<QuotaManager>) -> Self {
        Self { manager }
    }

    /// Create tenant-level quota
    pub async fn create_tenant_quota(
        &self,
        tenant_id: String,
        operation: String,
        limits: QuotaLimits,
    ) -> RateLimitResult<()> {
        let config = QuotaConfig::new(QuotaIdentifier::Tenant(tenant_id))
            .add_limit(operation, limits)
            .with_priority(100);

        self.manager.add_config(config).await
    }

    /// Create user-level quota (inherits from tenant)
    pub async fn create_user_quota(
        &self,
        user_id: String,
        tenant_id: String,
        operation: String,
        limits: QuotaLimits,
    ) -> RateLimitResult<()> {
        let config = QuotaConfig::new(QuotaIdentifier::User(user_id))
            .add_limit(operation, limits)
            .with_parent(QuotaIdentifier::Tenant(tenant_id))
            .with_priority(50);

        self.manager.add_config(config).await
    }

    /// Create API key quota (inherits from user)
    pub async fn create_api_key_quota(
        &self,
        api_key: String,
        user_id: String,
        operation: String,
        limits: QuotaLimits,
    ) -> RateLimitResult<()> {
        let config = QuotaConfig::new(QuotaIdentifier::ApiKey(api_key))
            .add_limit(operation, limits)
            .with_parent(QuotaIdentifier::User(user_id))
            .with_priority(10);

        self.manager.add_config(config).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quota_identifier() {
        let user_id = QuotaIdentifier::User("user123".to_string());
        assert_eq!(user_id.to_key(), "user:user123");

        let api_key = QuotaIdentifier::ApiKey("key456".to_string());
        assert_eq!(api_key.to_key(), "apikey:key456");
    }

    #[test]
    fn test_quota_period() {
        assert_eq!(QuotaPeriod::Second.as_secs(), 1);
        assert_eq!(QuotaPeriod::Minute.as_secs(), 60);
        assert_eq!(QuotaPeriod::Hour.as_secs(), 3600);
        assert_eq!(QuotaPeriod::Day.as_secs(), 86400);
    }

    #[test]
    fn test_quota_limits() {
        let limits = QuotaLimits::new(100, QuotaPeriod::Hour)
            .with_burst(150)
            .with_soft_limit(80);

        assert_eq!(limits.max_requests, 100);
        assert_eq!(limits.burst, Some(150));
        assert!(limits.is_soft_limit_exceeded(85));
        assert!(!limits.is_soft_limit_exceeded(75));
    }

    #[tokio::test]
    async fn test_quota_manager_basic() {
        let manager = QuotaManager::new();

        // Set default limits
        manager
            .set_default_limits(
                "api_call".to_string(),
                QuotaLimits::new(100, QuotaPeriod::Hour),
            )
            .await
            .unwrap();

        let user_id = QuotaIdentifier::User("user123".to_string());

        // Check quota (should use defaults)
        let decision = manager.check(&user_id, "api_call", 1).await.unwrap();
        assert!(decision.is_allowed());
        assert_eq!(decision.remaining(), Some(99));
    }

    #[tokio::test]
    async fn test_quota_manager_custom_config() {
        let manager = QuotaManager::new();

        let user_id = QuotaIdentifier::User("user456".to_string());
        let config = QuotaConfig::new(user_id.clone())
            .add_limit("api_call".to_string(), QuotaLimits::new(10, QuotaPeriod::Minute));

        manager.add_config(config).await.unwrap();

        // Should use custom limit (10)
        for _ in 0..10 {
            let decision = manager.check(&user_id, "api_call", 1).await.unwrap();
            assert!(decision.is_allowed());
        }

        // 11th should fail
        let decision = manager.check(&user_id, "api_call", 1).await.unwrap();
        assert!(!decision.is_allowed());
    }

    #[tokio::test]
    async fn test_quota_usage() {
        let manager = QuotaManager::new();

        manager
            .set_default_limits(
                "test_op".to_string(),
                QuotaLimits::new(50, QuotaPeriod::Hour),
            )
            .await
            .unwrap();

        let user_id = QuotaIdentifier::User("test_user".to_string());

        // Make some requests
        for _ in 0..20 {
            manager.check(&user_id, "test_op", 1).await.unwrap();
        }

        // Get usage
        let usage = manager.get_usage(&user_id, "test_op").await.unwrap();
        assert_eq!(usage.current, 20);
        assert_eq!(usage.remaining, 30);
        assert_eq!(usage.limit, 50);
        assert!(!usage.is_exceeded());
    }

    #[tokio::test]
    async fn test_quota_reset() {
        let manager = QuotaManager::new();

        manager
            .set_default_limits(
                "reset_test".to_string(),
                QuotaLimits::new(5, QuotaPeriod::Hour),
            )
            .await
            .unwrap();

        let user_id = QuotaIdentifier::User("reset_user".to_string());

        // Use up quota
        for _ in 0..5 {
            manager.check(&user_id, "reset_test", 1).await.unwrap();
        }

        // Should be denied
        let decision = manager.check(&user_id, "reset_test", 1).await.unwrap();
        assert!(!decision.is_allowed());

        // Reset
        manager.reset(&user_id, Some("reset_test")).await.unwrap();

        // Should work again
        let decision = manager.check(&user_id, "reset_test", 1).await.unwrap();
        assert!(decision.is_allowed());
    }

    #[tokio::test]
    async fn test_hierarchical_quotas() {
        let manager = Arc::new(QuotaManager::new());
        let builder = HierarchicalQuotaBuilder::new(manager.clone());

        // Create tenant quota
        builder
            .create_tenant_quota(
                "tenant1".to_string(),
                "api_call".to_string(),
                QuotaLimits::new(1000, QuotaPeriod::Hour),
            )
            .await
            .unwrap();

        // Create user quota (inherits from tenant)
        builder
            .create_user_quota(
                "user1".to_string(),
                "tenant1".to_string(),
                "api_call".to_string(),
                QuotaLimits::new(100, QuotaPeriod::Hour),
            )
            .await
            .unwrap();

        let user_id = QuotaIdentifier::User("user1".to_string());

        // Should use user limit (100), not tenant (1000)
        for _ in 0..100 {
            let decision = manager.check(&user_id, "api_call", 1).await.unwrap();
            assert!(decision.is_allowed());
        }

        let decision = manager.check(&user_id, "api_call", 1).await.unwrap();
        assert!(!decision.is_allowed());
    }
}
