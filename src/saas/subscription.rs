//! Subscription management and lifecycle
//!
//! This module provides comprehensive subscription management including:
//!
//! - Multi-tier subscriptions (Free, Pro, Enterprise)
//! - Feature flags per subscription tier
//! - Trial period management
//! - Subscription lifecycle (trial → active → cancelled → expired)
//! - Upgrades and downgrades with proration
//! - Billing cycle management
//!
//! ## Subscription Tiers
//!
//! - **Free**: Basic features with limited usage
//! - **Pro**: Advanced features with higher quotas ($49/month or $490/year)
//! - **Enterprise**: Full features with custom pricing
//!
//! ## Example
//!
//! ```rust
//! use caddy::saas::subscription::{SubscriptionManager, SubscriptionTier};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let manager = SubscriptionManager::new(pool).await?;
//!
//!     // Create subscription with trial
//!     let subscription = manager.create_subscription(
//!         tenant_id,
//!         SubscriptionTier::Pro,
//!     ).await?;
//!
//!     // Upgrade subscription
//!     manager.upgrade_subscription(subscription.id, SubscriptionTier::Enterprise).await?;
//!
//!     Ok(())
//! }
//! ```

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::saas::{
    BillingInterval, Result, SaasError, SubscriptionStatus, SubscriptionTier,
};

// ============================================================================
// Subscription Structure
// ============================================================================

/// Subscription record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Subscription {
    /// Unique subscription ID
    pub id: Uuid,

    /// Tenant ID
    pub tenant_id: Uuid,

    /// Subscription tier
    pub tier: SubscriptionTier,

    /// Subscription status
    pub status: SubscriptionStatus,

    /// Billing interval
    pub interval: BillingInterval,

    /// Trial end date (if in trial)
    pub trial_end_at: Option<DateTime<Utc>>,

    /// Current billing period start
    pub current_period_start: DateTime<Utc>,

    /// Current billing period end
    pub current_period_end: DateTime<Utc>,

    /// Cancellation scheduled for end of period
    pub cancel_at_period_end: bool,

    /// Cancellation timestamp
    pub cancelled_at: Option<DateTime<Utc>>,

    /// Stripe subscription ID
    pub stripe_subscription_id: Option<String>,

    /// Stripe customer ID
    pub stripe_customer_id: Option<String>,

    /// Subscription metadata
    #[sqlx(json)]
    pub metadata: HashMap<String, serde_json::Value>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Feature flag definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    /// Feature name/key
    pub key: String,

    /// Display name
    pub name: String,

    /// Description
    pub description: String,

    /// Enabled in Free tier
    pub enabled_free: bool,

    /// Enabled in Pro tier
    pub enabled_pro: bool,

    /// Enabled in Enterprise tier
    pub enabled_enterprise: bool,
}

/// Proration calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProrationCalculation {
    /// Current tier
    pub from_tier: SubscriptionTier,

    /// New tier
    pub to_tier: SubscriptionTier,

    /// Days remaining in current period
    pub days_remaining: i64,

    /// Total days in period
    pub total_days: i64,

    /// Credit amount in cents (negative if downgrade)
    pub credit_cents: i64,

    /// Amount due immediately in cents
    pub amount_due_cents: i64,

    /// Next billing amount in cents
    pub next_billing_cents: i64,
}

// ============================================================================
// Subscription Manager
// ============================================================================

/// Subscription management operations
pub struct SubscriptionManager {
    pool: PgPool,
    trial_period_days: i32,
}

impl SubscriptionManager {
    /// Create a new subscription manager
    pub async fn new(pool: PgPool) -> Result<Self> {
        Ok(Self {
            pool,
            trial_period_days: 14, // Default 14-day trial
        })
    }

    /// Create a new subscription manager with custom trial period
    pub async fn with_trial_period(pool: PgPool, trial_days: i32) -> Result<Self> {
        Ok(Self {
            pool,
            trial_period_days: trial_days,
        })
    }

    /// Create a new subscription with trial
    pub async fn create_subscription(
        &self,
        tenant_id: Uuid,
        tier: SubscriptionTier,
    ) -> Result<Subscription> {
        self.create_subscription_with_interval(tenant_id, tier, BillingInterval::Monthly)
            .await
    }

    /// Create a new subscription with specified billing interval
    pub async fn create_subscription_with_interval(
        &self,
        tenant_id: Uuid,
        tier: SubscriptionTier,
        interval: BillingInterval,
    ) -> Result<Subscription> {
        let subscription_id = Uuid::new_v4();
        let now = Utc::now();
        let trial_end = now + Duration::days(self.trial_period_days.into());
        let period_start = now;
        let period_end = match interval {
            BillingInterval::Monthly => now + Duration::days(30),
            BillingInterval::Yearly => now + Duration::days(365),
        };

        let metadata: HashMap<String, serde_json::Value> = HashMap::new();

        let subscription = sqlx::query_as::<_, Subscription>(
            r"
            INSERT INTO subscriptions (
                id, tenant_id, tier, status, interval,
                trial_end_at, current_period_start, current_period_end,
                cancel_at_period_end, metadata, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            ",
        )
        .bind(subscription_id)
        .bind(tenant_id)
        .bind(tier)
        .bind(SubscriptionStatus::Trial)
        .bind(interval)
        .bind(trial_end)
        .bind(period_start)
        .bind(period_end)
        .bind(false)
        .bind(serde_json::to_value(&metadata)?)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(subscription)
    }

    /// Get subscription by ID
    pub async fn get_subscription(&self, subscription_id: Uuid) -> Result<Subscription> {
        sqlx::query_as::<_, Subscription>(
            r"
            SELECT * FROM subscriptions WHERE id = $1
            ",
        )
        .bind(subscription_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| SaasError::SubscriptionNotFound(subscription_id.to_string()))
    }

    /// Get active subscription for tenant
    pub async fn get_tenant_subscription(&self, tenant_id: Uuid) -> Result<Subscription> {
        sqlx::query_as::<_, Subscription>(
            r"
            SELECT * FROM subscriptions
            WHERE tenant_id = $1
            AND status IN ('trial', 'active', 'past_due')
            ORDER BY created_at DESC
            LIMIT 1
            ",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| SaasError::SubscriptionNotFound(tenant_id.to_string()))
    }

    /// Check if a feature is enabled for a subscription
    pub async fn is_feature_enabled(
        &self,
        subscription_id: Uuid,
        feature_key: &str,
    ) -> Result<bool> {
        let subscription = self.get_subscription(subscription_id).await?;
        let feature = Self::get_feature_flag(feature_key)?;

        let enabled = match subscription.tier {
            SubscriptionTier::Free => feature.enabled_free,
            SubscriptionTier::Pro => feature.enabled_pro,
            SubscriptionTier::Enterprise => feature.enabled_enterprise,
        };

        // Feature is only available if subscription is active or in trial
        let is_active = matches!(
            subscription.status,
            SubscriptionStatus::Trial | SubscriptionStatus::Active
        );

        Ok(enabled && is_active)
    }

    /// Upgrade subscription to a higher tier
    pub async fn upgrade_subscription(
        &self,
        subscription_id: Uuid,
        new_tier: SubscriptionTier,
    ) -> Result<Subscription> {
        let subscription = self.get_subscription(subscription_id).await?;

        // Validate upgrade path
        if !Self::is_upgrade(subscription.tier, new_tier) {
            return Err(SaasError::InvalidSubscriptionState(
                "Not an upgrade".to_string(),
            ));
        }

        // Calculate proration
        let proration = self.calculate_proration(&subscription, new_tier).await?;

        // Update subscription
        let updated = sqlx::query_as::<_, Subscription>(
            r"
            UPDATE subscriptions
            SET tier = $1, status = $2, updated_at = $3
            WHERE id = $4
            RETURNING *
            ",
        )
        .bind(new_tier)
        .bind(SubscriptionStatus::Active)
        .bind(Utc::now())
        .bind(subscription_id)
        .fetch_one(&self.pool)
        .await?;

        // In a real implementation, this would:
        // 1. Create a proration invoice
        // 2. Update Stripe subscription
        // 3. Charge the customer immediately

        Ok(updated)
    }

    /// Downgrade subscription to a lower tier
    pub async fn downgrade_subscription(
        &self,
        subscription_id: Uuid,
        new_tier: SubscriptionTier,
    ) -> Result<Subscription> {
        let subscription = self.get_subscription(subscription_id).await?;

        // Validate downgrade path
        if !Self::is_downgrade(subscription.tier, new_tier) {
            return Err(SaasError::InvalidSubscriptionState(
                "Not a downgrade".to_string(),
            ));
        }

        // For downgrades, schedule the change at period end
        let updated = sqlx::query_as::<_, Subscription>(
            r"
            UPDATE subscriptions
            SET metadata = jsonb_set(
                metadata,
                '{scheduled_tier}',
                to_jsonb($1::text)
            ),
            updated_at = $2
            WHERE id = $3
            RETURNING *
            ",
        )
        .bind(format!("{:?}", new_tier).to_lowercase())
        .bind(Utc::now())
        .bind(subscription_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }

    /// Convert trial to paid subscription
    pub async fn convert_trial_to_paid(&self, subscription_id: Uuid) -> Result<Subscription> {
        let subscription = self.get_subscription(subscription_id).await?;

        if subscription.status != SubscriptionStatus::Trial {
            return Err(SaasError::InvalidSubscriptionState(
                "Subscription is not in trial".to_string(),
            ));
        }

        let updated = sqlx::query_as::<_, Subscription>(
            r"
            UPDATE subscriptions
            SET status = $1, trial_end_at = NULL, updated_at = $2
            WHERE id = $3
            RETURNING *
            ",
        )
        .bind(SubscriptionStatus::Active)
        .bind(Utc::now())
        .bind(subscription_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }

    /// Cancel subscription at period end
    pub async fn cancel_subscription(&self, subscription_id: Uuid) -> Result<Subscription> {
        let updated = sqlx::query_as::<_, Subscription>(
            r"
            UPDATE subscriptions
            SET cancel_at_period_end = true,
                cancelled_at = $1,
                updated_at = $2
            WHERE id = $3
            RETURNING *
            ",
        )
        .bind(Utc::now())
        .bind(Utc::now())
        .bind(subscription_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }

    /// Cancel subscription immediately
    pub async fn cancel_subscription_immediately(
        &self,
        subscription_id: Uuid,
    ) -> Result<Subscription> {
        let updated = sqlx::query_as::<_, Subscription>(
            r"
            UPDATE subscriptions
            SET status = $1,
                cancel_at_period_end = true,
                cancelled_at = $2,
                current_period_end = $3,
                updated_at = $4
            WHERE id = $5
            RETURNING *
            ",
        )
        .bind(SubscriptionStatus::Cancelled)
        .bind(Utc::now())
        .bind(Utc::now())
        .bind(Utc::now())
        .bind(subscription_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }

    /// Reactivate a cancelled subscription
    pub async fn reactivate_subscription(&self, subscription_id: Uuid) -> Result<Subscription> {
        let subscription = self.get_subscription(subscription_id).await?;

        if !subscription.cancel_at_period_end {
            return Err(SaasError::InvalidSubscriptionState(
                "Subscription is not scheduled for cancellation".to_string(),
            ));
        }

        let updated = sqlx::query_as::<_, Subscription>(
            r"
            UPDATE subscriptions
            SET cancel_at_period_end = false,
                cancelled_at = NULL,
                status = $1,
                updated_at = $2
            WHERE id = $3
            RETURNING *
            ",
        )
        .bind(SubscriptionStatus::Active)
        .bind(Utc::now())
        .bind(subscription_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }

    /// Renew subscription for next billing period
    pub async fn renew_subscription(&self, subscription_id: Uuid) -> Result<Subscription> {
        let subscription = self.get_subscription(subscription_id).await?;

        let new_period_start = subscription.current_period_end;
        let new_period_end = match subscription.interval {
            BillingInterval::Monthly => new_period_start + Duration::days(30),
            BillingInterval::Yearly => new_period_start + Duration::days(365),
        };

        // Check if there's a scheduled tier change
        let new_tier = subscription
            .metadata
            .get("scheduled_tier")
            .and_then(|v| v.as_str())
            .and_then(|s| match s {
                "free" => Some(SubscriptionTier::Free),
                "pro" => Some(SubscriptionTier::Pro),
                "enterprise" => Some(SubscriptionTier::Enterprise),
                _ => None,
            })
            .unwrap_or(subscription.tier);

        let updated = sqlx::query_as::<_, Subscription>(
            r"
            UPDATE subscriptions
            SET tier = $1,
                current_period_start = $2,
                current_period_end = $3,
                updated_at = $4
            WHERE id = $5
            RETURNING *
            ",
        )
        .bind(new_tier)
        .bind(new_period_start)
        .bind(new_period_end)
        .bind(Utc::now())
        .bind(subscription_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }

    /// Mark subscription as past due
    pub async fn mark_past_due(&self, subscription_id: Uuid) -> Result<Subscription> {
        let updated = sqlx::query_as::<_, Subscription>(
            r"
            UPDATE subscriptions
            SET status = $1, updated_at = $2
            WHERE id = $3
            RETURNING *
            ",
        )
        .bind(SubscriptionStatus::PastDue)
        .bind(Utc::now())
        .bind(subscription_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }

    /// Expire subscription
    pub async fn expire_subscription(&self, subscription_id: Uuid) -> Result<Subscription> {
        let updated = sqlx::query_as::<_, Subscription>(
            r"
            UPDATE subscriptions
            SET status = $1, updated_at = $2
            WHERE id = $3
            RETURNING *
            ",
        )
        .bind(SubscriptionStatus::Expired)
        .bind(Utc::now())
        .bind(subscription_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }

    /// Calculate proration for subscription change
    pub async fn calculate_proration(
        &self,
        subscription: &Subscription,
        new_tier: SubscriptionTier,
    ) -> Result<ProrationCalculation> {
        let now = Utc::now();
        let days_remaining = (subscription.current_period_end - now).num_days();
        let total_days = (subscription.current_period_end - subscription.current_period_start)
            .num_days();

        let current_price = match subscription.interval {
            BillingInterval::Monthly => subscription.tier.monthly_price_cents(),
            BillingInterval::Yearly => subscription.tier.yearly_price_cents(),
        };

        let new_price = match subscription.interval {
            BillingInterval::Monthly => new_tier.monthly_price_cents(),
            BillingInterval::Yearly => new_tier.yearly_price_cents(),
        };

        // Calculate unused amount from current subscription
        let unused_amount = (current_price * days_remaining) / total_days;

        // Calculate prorated amount for new tier
        let prorated_amount = (new_price * days_remaining) / total_days;

        // Amount to charge immediately
        let amount_due = prorated_amount - unused_amount;

        Ok(ProrationCalculation {
            from_tier: subscription.tier,
            to_tier: new_tier,
            days_remaining,
            total_days,
            credit_cents: unused_amount,
            amount_due_cents: amount_due,
            next_billing_cents: new_price,
        })
    }

    // ========================================================================
    // Feature Flags
    // ========================================================================

    /// Get feature flag definition
    pub fn get_feature_flag(key: &str) -> Result<FeatureFlag> {
        let features = Self::get_all_feature_flags();
        features
            .into_iter()
            .find(|f| f.key == key)
            .ok_or_else(|| SaasError::FeatureNotAvailable(key.to_string()))
    }

    /// Get all feature flags
    pub fn get_all_feature_flags() -> Vec<FeatureFlag> {
        vec![
            FeatureFlag {
                key: "basic_cad".to_string(),
                name: "Basic CAD".to_string(),
                description: "Basic 2D drawing and editing".to_string(),
                enabled_free: true,
                enabled_pro: true,
                enabled_enterprise: true,
            },
            FeatureFlag {
                key: "3d_modeling".to_string(),
                name: "3D Modeling".to_string(),
                description: "Advanced 3D modeling capabilities".to_string(),
                enabled_free: false,
                enabled_pro: true,
                enabled_enterprise: true,
            },
            FeatureFlag {
                key: "api_access".to_string(),
                name: "API Access".to_string(),
                description: "REST API access for integrations".to_string(),
                enabled_free: false,
                enabled_pro: true,
                enabled_enterprise: true,
            },
            FeatureFlag {
                key: "collaboration".to_string(),
                name: "Real-time Collaboration".to_string(),
                description: "Multi-user real-time editing".to_string(),
                enabled_free: false,
                enabled_pro: false,
                enabled_enterprise: true,
            },
            FeatureFlag {
                key: "custom_branding".to_string(),
                name: "Custom Branding".to_string(),
                description: "White-label branding customization".to_string(),
                enabled_free: false,
                enabled_pro: false,
                enabled_enterprise: true,
            },
            FeatureFlag {
                key: "sso".to_string(),
                name: "Single Sign-On".to_string(),
                description: "SSO/SAML authentication".to_string(),
                enabled_free: false,
                enabled_pro: false,
                enabled_enterprise: true,
            },
            FeatureFlag {
                key: "audit_logs".to_string(),
                name: "Audit Logs".to_string(),
                description: "Comprehensive audit logging".to_string(),
                enabled_free: false,
                enabled_pro: true,
                enabled_enterprise: true,
            },
            FeatureFlag {
                key: "priority_support".to_string(),
                name: "Priority Support".to_string(),
                description: "Priority customer support".to_string(),
                enabled_free: false,
                enabled_pro: true,
                enabled_enterprise: true,
            },
        ]
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    /// Check if tier change is an upgrade
    fn is_upgrade(from: SubscriptionTier, to: SubscriptionTier) -> bool {
        matches!(
            (from, to),
            (SubscriptionTier::Free, SubscriptionTier::Pro)
                | (SubscriptionTier::Free, SubscriptionTier::Enterprise)
                | (SubscriptionTier::Pro, SubscriptionTier::Enterprise)
        )
    }

    /// Check if tier change is a downgrade
    fn is_downgrade(from: SubscriptionTier, to: SubscriptionTier) -> bool {
        matches!(
            (from, to),
            (SubscriptionTier::Pro, SubscriptionTier::Free)
                | (SubscriptionTier::Enterprise, SubscriptionTier::Free)
                | (SubscriptionTier::Enterprise, SubscriptionTier::Pro)
        )
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_upgrade() {
        assert!(SubscriptionManager::is_upgrade(
            SubscriptionTier::Free,
            SubscriptionTier::Pro
        ));
        assert!(SubscriptionManager::is_upgrade(
            SubscriptionTier::Pro,
            SubscriptionTier::Enterprise
        ));
        assert!(!SubscriptionManager::is_upgrade(
            SubscriptionTier::Pro,
            SubscriptionTier::Free
        ));
    }

    #[test]
    fn test_is_downgrade() {
        assert!(SubscriptionManager::is_downgrade(
            SubscriptionTier::Pro,
            SubscriptionTier::Free
        ));
        assert!(SubscriptionManager::is_downgrade(
            SubscriptionTier::Enterprise,
            SubscriptionTier::Pro
        ));
        assert!(!SubscriptionManager::is_downgrade(
            SubscriptionTier::Free,
            SubscriptionTier::Pro
        ));
    }

    #[test]
    fn test_feature_flags() {
        let flags = SubscriptionManager::get_all_feature_flags();
        assert!(!flags.is_empty());

        let basic_cad = SubscriptionManager::get_feature_flag("basic_cad").unwrap();
        assert!(basic_cad.enabled_free);
        assert!(basic_cad.enabled_pro);
        assert!(basic_cad.enabled_enterprise);

        let collab = SubscriptionManager::get_feature_flag("collaboration").unwrap();
        assert!(!collab.enabled_free);
        assert!(!collab.enabled_pro);
        assert!(collab.enabled_enterprise);
    }

    #[test]
    fn test_tier_pricing() {
        assert_eq!(SubscriptionTier::Free.monthly_price_cents(), 0);
        assert_eq!(SubscriptionTier::Pro.monthly_price_cents(), 4900);
        assert_eq!(SubscriptionTier::Pro.yearly_price_cents(), 49000);
    }
}
