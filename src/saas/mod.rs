//! # CADDY SaaS Infrastructure - v0.3.0
//!
//! Production-ready multi-tenant SaaS infrastructure for CADDY Enterprise Edition.
//!
//! ## Overview
//!
//! This module provides comprehensive SaaS functionality including:
//!
//! - **Tenant Management** (`tenant`): Multi-tenant isolation, provisioning, custom domains, white-labeling
//! - **Subscription Management** (`subscription`): Tiered subscriptions, feature flags, lifecycle management
//! - **Billing Integration** (`billing`): Stripe integration, invoicing, payment methods, dunning
//! - **Usage Tracking** (`usage`): API metering, resource monitoring, overage handling
//! - **Quota Management** (`quotas`): Rate limiting, soft/hard limits, grace periods
//!
//! ## Architecture
//!
//! The SaaS infrastructure is designed with:
//!
//! - **Row-level Security**: Database-enforced tenant isolation
//! - **Usage-based Billing**: Real-time metering and billing calculations
//! - **Flexible Subscriptions**: Support for trials, upgrades, downgrades, and proration
//! - **Payment Processing**: Stripe integration with webhook handling
//! - **Resource Management**: Comprehensive quota and rate limiting
//!
//! ## Example Usage
//!
//! ```rust
//! use caddy::saas::{
//!     tenant::{TenantManager, TenantConfig},
//!     subscription::{SubscriptionManager, SubscriptionTier},
//!     billing::BillingManager,
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize SaaS infrastructure
//!     let tenant_mgr = TenantManager::new(db_pool).await?;
//!     let subscription_mgr = SubscriptionManager::new(db_pool).await?;
//!     let billing_mgr = BillingManager::new("sk_test_...").await?;
//!
//!     // Create a new tenant with Pro subscription
//!     let tenant = tenant_mgr.create_tenant("acme-corp", "Acme Corporation").await?;
//!     let subscription = subscription_mgr.create_subscription(
//!         tenant.id,
//!         SubscriptionTier::Pro,
//!     ).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Database Schema
//!
//! The SaaS infrastructure requires the following database tables:
//!
//! - `tenants`: Tenant information and settings
//! - `subscriptions`: Subscription records and billing cycles
//! - `usage_records`: Metered usage data
//! - `invoices`: Generated invoices and payment records
//! - `payment_methods`: Customer payment methods
//! - `quotas`: Tenant quota configurations
//!
//! Run migrations with:
//!
//! ```bash
//! sqlx migrate run --source migrations/saas
//! ```
//!
//! ## Security Considerations
//!
//! - All sensitive data (API keys, payment tokens) are encrypted at rest
//! - Row-level security enforces tenant data isolation
//! - Webhook signatures are verified for payment notifications
//! - PCI compliance guidelines followed for payment data
//!
//! ## License
//!
//! CADDY Enterprise Edition - Commercial License Required

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use thiserror::Error;
use serde::{Deserialize, Serialize};

// ============================================================================
// Module Declarations
// ============================================================================

/// Tenant management and multi-tenancy infrastructure
///
/// Provides tenant provisioning, isolation, custom domains, and white-label branding.
pub mod tenant;

/// Subscription management and lifecycle
///
/// Handles subscription tiers, feature flags, trials, upgrades/downgrades, and proration.
pub mod subscription;

/// Billing and payment processing
///
/// Stripe integration, invoice generation, payment methods, and dunning management.
pub mod billing;

/// Usage tracking and metering
///
/// Tracks API calls, page scans, storage, user seats, and handles overage.
pub mod usage;

/// Quota management and rate limiting
///
/// Enforces quotas per tier with soft/hard limits, alerts, and grace periods.
pub mod quotas;

// ============================================================================
// Error Types
// ============================================================================

/// SaaS infrastructure error types
#[derive(Error, Debug)]
pub enum SaasError {
    /// Tenant-related errors
    #[error("Tenant error: {0}")]
    Tenant(String),

    /// Tenant not found
    #[error("Tenant not found: {0}")]
    TenantNotFound(String),

    /// Tenant already exists
    #[error("Tenant already exists: {0}")]
    TenantExists(String),

    /// Subscription-related errors
    #[error("Subscription error: {0}")]
    Subscription(String),

    /// Subscription not found
    #[error("Subscription not found: {0}")]
    SubscriptionNotFound(String),

    /// Invalid subscription state transition
    #[error("Invalid subscription state: {0}")]
    InvalidSubscriptionState(String),

    /// Billing-related errors
    #[error("Billing error: {0}")]
    Billing(String),

    /// Payment processing error
    #[error("Payment error: {0}")]
    PaymentError(String),

    /// Invalid payment method
    #[error("Invalid payment method: {0}")]
    InvalidPaymentMethod(String),

    /// Usage tracking errors
    #[error("Usage tracking error: {0}")]
    Usage(String),

    /// Quota exceeded
    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// Feature not available in current tier
    #[error("Feature not available: {0}")]
    FeatureNotAvailable(String),

    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// HTTP/API error
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Invalid configuration
    #[error("Configuration error: {0}")]
    Config(String),

    /// Stripe API error
    #[error("Stripe error: {0}")]
    Stripe(String),

    /// Webhook verification failed
    #[error("Webhook verification failed: {0}")]
    WebhookVerification(String),

    /// Generic SaaS error
    #[error("SaaS error: {0}")]
    Other(String),
}

/// Result type for SaaS operations
pub type Result<T> = std::result::Result<T, SaasError>;

// ============================================================================
// Common Types
// ============================================================================

/// Subscription tier levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "subscription_tier", rename_all = "lowercase")]
pub enum SubscriptionTier {
    /// Free tier with basic features
    Free,
    /// Professional tier with advanced features
    Pro,
    /// Enterprise tier with all features
    Enterprise,
}

impl SubscriptionTier {
    /// Get the display name for the tier
    pub fn display_name(&self) -> &str {
        match self {
            Self::Free => "Free",
            Self::Pro => "Professional",
            Self::Enterprise => "Enterprise",
        }
    }

    /// Get the monthly price in cents
    pub fn monthly_price_cents(&self) -> i64 {
        match self {
            Self::Free => 0,
            Self::Pro => 4900,      // $49/month
            Self::Enterprise => 0,   // Custom pricing
        }
    }

    /// Get the yearly price in cents (with discount)
    pub fn yearly_price_cents(&self) -> i64 {
        match self {
            Self::Free => 0,
            Self::Pro => 49000,      // $490/year (2 months free)
            Self::Enterprise => 0,   // Custom pricing
        }
    }
}

/// Billing interval
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "billing_interval", rename_all = "lowercase")]
pub enum BillingInterval {
    /// Monthly billing
    Monthly,
    /// Yearly billing
    Yearly,
}

/// Subscription status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "subscription_status", rename_all = "lowercase")]
pub enum SubscriptionStatus {
    /// Trial period
    Trial,
    /// Active and paid
    Active,
    /// Past due payment
    PastDue,
    /// Cancelled but still active until period end
    Cancelled,
    /// Expired/inactive
    Expired,
}

/// Payment status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "payment_status", rename_all = "lowercase")]
pub enum PaymentStatus {
    /// Payment pending
    Pending,
    /// Payment processing
    Processing,
    /// Payment succeeded
    Succeeded,
    /// Payment failed
    Failed,
    /// Payment refunded
    Refunded,
}

// ============================================================================
// Configuration
// ============================================================================

/// SaaS infrastructure configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaasConfig {
    /// Enable tenant isolation
    pub enable_tenant_isolation: bool,

    /// Enable usage tracking
    pub enable_usage_tracking: bool,

    /// Enable quota enforcement
    pub enable_quota_enforcement: bool,

    /// Stripe API secret key
    pub stripe_secret_key: Option<String>,

    /// Stripe webhook secret
    pub stripe_webhook_secret: Option<String>,

    /// Trial period in days
    pub trial_period_days: i32,

    /// Grace period for overages in days
    pub grace_period_days: i32,

    /// Enable automatic dunning
    pub enable_dunning: bool,

    /// Maximum dunning retries
    pub max_dunning_retries: i32,
}

impl Default for SaasConfig {
    fn default() -> Self {
        Self {
            enable_tenant_isolation: true,
            enable_usage_tracking: true,
            enable_quota_enforcement: true,
            stripe_secret_key: None,
            stripe_webhook_secret: None,
            trial_period_days: 14,
            grace_period_days: 3,
            enable_dunning: true,
            max_dunning_retries: 3,
        }
    }
}

impl SaasConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            enable_tenant_isolation: std::env::var("SAAS_TENANT_ISOLATION")
                .map(|v| v.parse().unwrap_or(true))
                .unwrap_or(true),
            enable_usage_tracking: std::env::var("SAAS_USAGE_TRACKING")
                .map(|v| v.parse().unwrap_or(true))
                .unwrap_or(true),
            enable_quota_enforcement: std::env::var("SAAS_QUOTA_ENFORCEMENT")
                .map(|v| v.parse().unwrap_or(true))
                .unwrap_or(true),
            stripe_secret_key: std::env::var("STRIPE_SECRET_KEY").ok(),
            stripe_webhook_secret: std::env::var("STRIPE_WEBHOOK_SECRET").ok(),
            trial_period_days: std::env::var("SAAS_TRIAL_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(14),
            grace_period_days: std::env::var("SAAS_GRACE_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3),
            enable_dunning: std::env::var("SAAS_DUNNING")
                .map(|v| v.parse().unwrap_or(true))
                .unwrap_or(true),
            max_dunning_retries: std::env::var("SAAS_MAX_DUNNING_RETRIES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3),
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.enable_quota_enforcement && self.stripe_secret_key.is_none() {
            return Err(SaasError::Config(
                "Stripe secret key required for billing".to_string()
            ));
        }

        if self.trial_period_days < 0 {
            return Err(SaasError::Config(
                "Trial period must be non-negative".to_string()
            ));
        }

        if self.grace_period_days < 0 {
            return Err(SaasError::Config(
                "Grace period must be non-negative".to_string()
            ));
        }

        Ok(())
    }
}

// ============================================================================
// Re-exports
// ============================================================================

pub use tenant::{Tenant, TenantManager, TenantConfig, TenantSettings};
pub use subscription::{Subscription, SubscriptionManager, FeatureFlag};
pub use billing::{BillingManager, Invoice, PaymentMethod};
pub use usage::{UsageManager, UsageRecord, UsageMetric};
pub use quotas::{QuotaManager, Quota, RateLimit};

// ============================================================================
// Version Information
// ============================================================================

/// SaaS module version
pub const SAAS_VERSION: &str = "0.3.0";

/// Module build date
pub const BUILD_DATE: &str = "2025-12-29";

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_tier_pricing() {
        assert_eq!(SubscriptionTier::Free.monthly_price_cents(), 0);
        assert_eq!(SubscriptionTier::Pro.monthly_price_cents(), 4900);
        assert_eq!(SubscriptionTier::Pro.yearly_price_cents(), 49000);
    }

    #[test]
    fn test_subscription_tier_display_name() {
        assert_eq!(SubscriptionTier::Free.display_name(), "Free");
        assert_eq!(SubscriptionTier::Pro.display_name(), "Professional");
        assert_eq!(SubscriptionTier::Enterprise.display_name(), "Enterprise");
    }

    #[test]
    fn test_saas_config_default() {
        let config = SaasConfig::default();
        assert!(config.enable_tenant_isolation);
        assert!(config.enable_usage_tracking);
        assert_eq!(config.trial_period_days, 14);
        assert_eq!(config.grace_period_days, 3);
    }

    #[test]
    fn test_saas_config_validation() {
        let mut config = SaasConfig::default();
        assert!(config.validate().is_err()); // Missing Stripe key

        config.stripe_secret_key = Some("sk_test_123".to_string());
        assert!(config.validate().is_ok());

        config.trial_period_days = -1;
        assert!(config.validate().is_err());
    }
}
