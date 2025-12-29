//! # Subscription Management
//!
//! This module handles subscription status tracking, renewal handling,
//! upgrade/downgrade paths, and billing integration.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

use super::license::LicenseType;

/// Errors that can occur with subscriptions
#[derive(Debug, Error)]
pub enum SubscriptionError {
    #[error("Subscription not found")]
    NotFound,

    #[error("Subscription has been cancelled")]
    Cancelled,

    #[error("Subscription has expired")]
    Expired,

    #[error("Invalid subscription state transition")]
    InvalidStateTransition,

    #[error("Payment failed: {0}")]
    PaymentFailed(String),

    #[error("Invalid upgrade path")]
    InvalidUpgradePath,

    #[error("Billing integration error: {0}")]
    BillingError(String),

    #[error("Proration calculation failed")]
    ProrationFailed,
}

/// Subscription status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubscriptionStatus {
    /// Active and in good standing
    Active,

    /// Trial period
    Trial,

    /// Past due (payment failed but grace period active)
    PastDue,

    /// Suspended (payment failed, grace period expired)
    Suspended,

    /// Cancelled by user
    Cancelled,

    /// Expired (end of term)
    Expired,

    /// Pending activation
    Pending,
}

/// Billing interval
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BillingInterval {
    Monthly,
    Quarterly,
    Yearly,
    Biennial,
    Perpetual,
}

impl BillingInterval {
    /// Get the duration of the billing interval
    pub fn duration(&self) -> Option<Duration> {
        match self {
            BillingInterval::Monthly => Some(Duration::days(30)),
            BillingInterval::Quarterly => Some(Duration::days(90)),
            BillingInterval::Yearly => Some(Duration::days(365)),
            BillingInterval::Biennial => Some(Duration::days(730)),
            BillingInterval::Perpetual => None,
        }
    }

    /// Get a human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            BillingInterval::Monthly => "Monthly",
            BillingInterval::Quarterly => "Quarterly",
            BillingInterval::Yearly => "Yearly",
            BillingInterval::Biennial => "Biennial",
            BillingInterval::Perpetual => "Perpetual",
        }
    }
}

/// Payment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentInfo {
    /// Payment method ID (from billing provider)
    pub payment_method_id: String,

    /// Last 4 digits of card/account
    pub last_four: Option<String>,

    /// Payment method type
    pub method_type: String,

    /// Billing address
    pub billing_address: Option<BillingAddress>,
}

/// Billing address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingAddress {
    pub line1: String,
    pub line2: Option<String>,
    pub city: String,
    pub state: Option<String>,
    pub postal_code: String,
    pub country: String,
}

/// Subscription plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionPlan {
    /// Plan ID
    pub id: String,

    /// Plan name
    pub name: String,

    /// License type
    pub license_type: LicenseType,

    /// Billing interval
    pub interval: BillingInterval,

    /// Price in cents
    pub price_cents: u64,

    /// Currency code (ISO 4217)
    pub currency: String,

    /// Maximum users/seats
    pub max_users: Option<u32>,

    /// Features included
    pub features: Vec<String>,

    /// Whether this plan is active/available
    pub active: bool,
}

impl SubscriptionPlan {
    /// Check if upgrade is allowed to another plan
    pub fn can_upgrade_to(&self, target: &SubscriptionPlan) -> bool {
        // Can upgrade to higher tier or same tier with longer interval
        self.license_type as u8 <= target.license_type as u8
    }

    /// Check if downgrade is allowed to another plan
    pub fn can_downgrade_to(&self, target: &SubscriptionPlan) -> bool {
        // Can downgrade to lower tier
        self.license_type as u8 >= target.license_type as u8
    }
}

/// Subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    /// Unique subscription ID
    pub id: Uuid,

    /// Associated license ID
    pub license_id: Uuid,

    /// Current plan
    pub plan: SubscriptionPlan,

    /// Status
    pub status: SubscriptionStatus,

    /// Current period start
    pub current_period_start: DateTime<Utc>,

    /// Current period end
    pub current_period_end: DateTime<Utc>,

    /// Trial end date (if applicable)
    pub trial_end: Option<DateTime<Utc>>,

    /// Cancellation date (if cancelled)
    pub cancelled_at: Option<DateTime<Utc>>,

    /// Whether to cancel at period end
    pub cancel_at_period_end: bool,

    /// Payment information
    pub payment_info: Option<PaymentInfo>,

    /// External billing provider ID (e.g., Stripe subscription ID)
    pub billing_provider_id: Option<String>,

    /// Auto-renew enabled
    pub auto_renew: bool,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Metadata
    pub metadata: HashMap<String, String>,
}

impl Subscription {
    /// Create a new subscription
    pub fn new(license_id: Uuid, plan: SubscriptionPlan, trial_days: Option<i64>) -> Self {
        let now = Utc::now();
        let current_period_end = if let Some(duration) = plan.interval.duration() {
            now + duration
        } else {
            // Perpetual - set to far future
            now + Duration::days(36500) // ~100 years
        };

        let (status, trial_end) = if let Some(days) = trial_days {
            (SubscriptionStatus::Trial, Some(now + Duration::days(days)))
        } else {
            (SubscriptionStatus::Active, None)
        };

        Self {
            id: Uuid::new_v4(),
            license_id,
            plan,
            status,
            current_period_start: now,
            current_period_end,
            trial_end,
            cancelled_at: None,
            cancel_at_period_end: false,
            payment_info: None,
            billing_provider_id: None,
            auto_renew: true,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    /// Check if subscription is active
    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            SubscriptionStatus::Active | SubscriptionStatus::Trial
        )
    }

    /// Check if subscription is in trial
    pub fn is_trial(&self) -> bool {
        self.status == SubscriptionStatus::Trial
    }

    /// Check if subscription has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.current_period_end
            || self.status == SubscriptionStatus::Expired
    }

    /// Get days until renewal
    pub fn days_until_renewal(&self) -> i64 {
        self.current_period_end
            .signed_duration_since(Utc::now())
            .num_days()
    }

    /// Get days remaining in trial
    pub fn trial_days_remaining(&self) -> Option<i64> {
        self.trial_end.map(|end| {
            end.signed_duration_since(Utc::now())
                .num_days()
                .max(0)
        })
    }

    /// Renew the subscription
    pub fn renew(&mut self) -> Result<(), SubscriptionError> {
        if !self.auto_renew {
            return Err(SubscriptionError::InvalidStateTransition);
        }

        if self.status == SubscriptionStatus::Cancelled {
            return Err(SubscriptionError::Cancelled);
        }

        let duration = self.plan.interval.duration()
            .ok_or(SubscriptionError::InvalidStateTransition)?;

        self.current_period_start = self.current_period_end;
        self.current_period_end = self.current_period_start + duration;
        self.status = SubscriptionStatus::Active;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Cancel the subscription
    pub fn cancel(&mut self, immediate: bool) -> Result<(), SubscriptionError> {
        if immediate {
            self.status = SubscriptionStatus::Cancelled;
            self.cancelled_at = Some(Utc::now());
            self.current_period_end = Utc::now();
        } else {
            self.cancel_at_period_end = true;
        }

        self.auto_renew = false;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Mark subscription as past due
    pub fn mark_past_due(&mut self) {
        self.status = SubscriptionStatus::PastDue;
        self.updated_at = Utc::now();
    }

    /// Suspend subscription
    pub fn suspend(&mut self) {
        self.status = SubscriptionStatus::Suspended;
        self.updated_at = Utc::now();
    }

    /// Reactivate suspended subscription
    pub fn reactivate(&mut self) -> Result<(), SubscriptionError> {
        if self.status != SubscriptionStatus::Suspended
            && self.status != SubscriptionStatus::PastDue
        {
            return Err(SubscriptionError::InvalidStateTransition);
        }

        self.status = SubscriptionStatus::Active;
        self.updated_at = Utc::now();

        Ok(())
    }
}

/// Invoice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    /// Invoice ID
    pub id: Uuid,

    /// Subscription ID
    pub subscription_id: Uuid,

    /// Invoice number
    pub number: String,

    /// Amount in cents
    pub amount_cents: u64,

    /// Currency
    pub currency: String,

    /// Status
    pub status: InvoiceStatus,

    /// Due date
    pub due_date: DateTime<Utc>,

    /// Paid date
    pub paid_at: Option<DateTime<Utc>>,

    /// Line items
    pub line_items: Vec<InvoiceLineItem>,

    /// Created timestamp
    pub created_at: DateTime<Utc>,
}

/// Invoice status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvoiceStatus {
    Draft,
    Open,
    Paid,
    Void,
    Uncollectible,
}

/// Invoice line item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLineItem {
    /// Description
    pub description: String,

    /// Quantity
    pub quantity: u32,

    /// Unit price in cents
    pub unit_price_cents: u64,

    /// Total amount in cents
    pub amount_cents: u64,
}

/// Subscription manager
pub struct SubscriptionManager {
    /// Active subscriptions
    subscriptions: HashMap<Uuid, Subscription>,

    /// Available plans
    plans: HashMap<String, SubscriptionPlan>,

    /// Invoices
    invoices: HashMap<Uuid, Invoice>,
}

impl SubscriptionManager {
    /// Create a new subscription manager
    pub fn new() -> Self {
        Self {
            subscriptions: HashMap::new(),
            plans: HashMap::new(),
            invoices: HashMap::new(),
        }
    }

    /// Add a subscription plan
    pub fn add_plan(&mut self, plan: SubscriptionPlan) {
        self.plans.insert(plan.id.clone(), plan);
    }

    /// Get a plan by ID
    pub fn get_plan(&self, plan_id: &str) -> Option<&SubscriptionPlan> {
        self.plans.get(plan_id)
    }

    /// Create a new subscription
    pub fn create_subscription(
        &mut self,
        license_id: Uuid,
        plan_id: &str,
        trial_days: Option<i64>,
    ) -> Result<Subscription, SubscriptionError> {
        let plan = self
            .get_plan(plan_id)
            .ok_or(SubscriptionError::NotFound)?
            .clone();

        let subscription = Subscription::new(license_id, plan, trial_days);
        let sub_id = subscription.id;

        self.subscriptions.insert(sub_id, subscription.clone());

        Ok(subscription)
    }

    /// Get subscription
    pub fn get_subscription(&self, subscription_id: Uuid) -> Option<&Subscription> {
        self.subscriptions.get(&subscription_id)
    }

    /// Get mutable subscription
    pub fn get_subscription_mut(&mut self, subscription_id: Uuid) -> Option<&mut Subscription> {
        self.subscriptions.get_mut(&subscription_id)
    }

    /// Upgrade subscription
    pub fn upgrade_subscription(
        &mut self,
        subscription_id: Uuid,
        new_plan_id: &str,
    ) -> Result<(), SubscriptionError> {
        let new_plan = self
            .get_plan(new_plan_id)
            .ok_or(SubscriptionError::NotFound)?
            .clone();

        // First, check if upgrade is valid and calculate proration
        let proration = {
            let subscription = self
                .get_subscription(subscription_id)
                .ok_or(SubscriptionError::NotFound)?;

            if !subscription.plan.can_upgrade_to(&new_plan) {
                return Err(SubscriptionError::InvalidUpgradePath);
            }

            self.calculate_proration(subscription, &new_plan)?
        };

        // Now update the subscription
        let subscription = self
            .get_subscription_mut(subscription_id)
            .ok_or(SubscriptionError::NotFound)?;

        let _proration = proration;
        subscription.plan = new_plan;
        subscription.updated_at = Utc::now();

        Ok(())
    }

    /// Downgrade subscription
    pub fn downgrade_subscription(
        &mut self,
        subscription_id: Uuid,
        new_plan_id: &str,
    ) -> Result<(), SubscriptionError> {
        let new_plan = self
            .get_plan(new_plan_id)
            .ok_or(SubscriptionError::NotFound)?
            .clone();

        let subscription = self
            .get_subscription_mut(subscription_id)
            .ok_or(SubscriptionError::NotFound)?;

        if !subscription.plan.can_downgrade_to(&new_plan) {
            return Err(SubscriptionError::InvalidUpgradePath);
        }

        // Schedule downgrade for end of period
        subscription.plan = new_plan;
        subscription.updated_at = Utc::now();

        Ok(())
    }

    /// Calculate proration amount
    fn calculate_proration(
        &self,
        subscription: &Subscription,
        new_plan: &SubscriptionPlan,
    ) -> Result<i64, SubscriptionError> {
        let remaining_days = subscription.days_until_renewal();
        if remaining_days <= 0 {
            return Ok(0);
        }

        let old_daily_rate = subscription.plan.price_cents as f64
            / subscription.plan.interval.duration()
                .ok_or(SubscriptionError::ProrationFailed)?
                .num_days() as f64;

        let new_daily_rate = new_plan.price_cents as f64
            / new_plan.interval.duration()
                .ok_or(SubscriptionError::ProrationFailed)?
                .num_days() as f64;

        let proration = (new_daily_rate - old_daily_rate) * remaining_days as f64;

        Ok(proration as i64)
    }

    /// Process renewal for subscriptions
    pub fn process_renewals(&mut self) -> Vec<Result<Uuid, SubscriptionError>> {
        let mut results = Vec::new();

        let now = Utc::now();
        let subscription_ids: Vec<Uuid> = self.subscriptions.keys().copied().collect();

        for sub_id in subscription_ids {
            if let Some(subscription) = self.subscriptions.get_mut(&sub_id) {
                // Check if renewal is due
                if subscription.current_period_end <= now && subscription.auto_renew {
                    match subscription.renew() {
                        Ok(()) => results.push(Ok(sub_id)),
                        Err(e) => results.push(Err(e)),
                    }
                }

                // Check if should be cancelled at period end
                if subscription.cancel_at_period_end && subscription.current_period_end <= now {
                    subscription.status = SubscriptionStatus::Cancelled;
                    subscription.cancelled_at = Some(now);
                }
            }
        }

        results
    }

    /// Create invoice for subscription
    pub fn create_invoice(
        &mut self,
        subscription_id: Uuid,
    ) -> Result<Invoice, SubscriptionError> {
        let subscription = self
            .get_subscription(subscription_id)
            .ok_or(SubscriptionError::NotFound)?;

        let line_item = InvoiceLineItem {
            description: format!("{} - {}", subscription.plan.name, subscription.plan.interval.name()),
            quantity: 1,
            unit_price_cents: subscription.plan.price_cents,
            amount_cents: subscription.plan.price_cents,
        };

        let invoice = Invoice {
            id: Uuid::new_v4(),
            subscription_id,
            number: format!("INV-{}", Uuid::new_v4().simple()),
            amount_cents: subscription.plan.price_cents,
            currency: subscription.plan.currency.clone(),
            status: InvoiceStatus::Open,
            due_date: Utc::now() + Duration::days(30),
            paid_at: None,
            line_items: vec![line_item],
            created_at: Utc::now(),
        };

        let invoice_id = invoice.id;
        self.invoices.insert(invoice_id, invoice.clone());

        Ok(invoice)
    }

    /// Mark invoice as paid
    pub fn mark_invoice_paid(&mut self, invoice_id: Uuid) -> Result<(), SubscriptionError> {
        let invoice = self
            .invoices
            .get_mut(&invoice_id)
            .ok_or(SubscriptionError::NotFound)?;

        invoice.status = InvoiceStatus::Paid;
        invoice.paid_at = Some(Utc::now());

        Ok(())
    }
}

impl Default for SubscriptionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_creation() {
        let plan = SubscriptionPlan {
            id: "pro-monthly".to_string(),
            name: "Professional Monthly".to_string(),
            license_type: LicenseType::Professional,
            interval: BillingInterval::Monthly,
            price_cents: 4900,
            currency: "USD".to_string(),
            max_users: Some(1),
            features: vec!["feature1".to_string(), "feature2".to_string()],
            active: true,
        };

        let license_id = Uuid::new_v4();
        let subscription = Subscription::new(license_id, plan, None);

        assert_eq!(subscription.status, SubscriptionStatus::Active);
        assert!(subscription.is_active());
        assert!(!subscription.is_trial());
    }

    #[test]
    fn test_subscription_renewal() {
        let plan = SubscriptionPlan {
            id: "pro-monthly".to_string(),
            name: "Professional Monthly".to_string(),
            license_type: LicenseType::Professional,
            interval: BillingInterval::Monthly,
            price_cents: 4900,
            currency: "USD".to_string(),
            max_users: Some(1),
            features: vec![],
            active: true,
        };

        let license_id = Uuid::new_v4();
        let mut subscription = Subscription::new(license_id, plan, None);

        let original_end = subscription.current_period_end;
        subscription.renew().unwrap();

        assert!(subscription.current_period_end > original_end);
        assert_eq!(subscription.status, SubscriptionStatus::Active);
    }

    #[test]
    fn test_subscription_cancellation() {
        let plan = SubscriptionPlan {
            id: "pro-monthly".to_string(),
            name: "Professional Monthly".to_string(),
            license_type: LicenseType::Professional,
            interval: BillingInterval::Monthly,
            price_cents: 4900,
            currency: "USD".to_string(),
            max_users: Some(1),
            features: vec![],
            active: true,
        };

        let license_id = Uuid::new_v4();
        let mut subscription = Subscription::new(license_id, plan, None);

        // Cancel at period end
        subscription.cancel(false).unwrap();
        assert!(subscription.cancel_at_period_end);
        assert!(!subscription.auto_renew);

        // Immediate cancel
        let mut subscription2 = subscription.clone();
        subscription2.cancel(true).unwrap();
        assert_eq!(subscription2.status, SubscriptionStatus::Cancelled);
    }

    #[test]
    fn test_subscription_manager() {
        let mut manager = SubscriptionManager::new();

        let plan = SubscriptionPlan {
            id: "pro-monthly".to_string(),
            name: "Professional Monthly".to_string(),
            license_type: LicenseType::Professional,
            interval: BillingInterval::Monthly,
            price_cents: 4900,
            currency: "USD".to_string(),
            max_users: Some(1),
            features: vec![],
            active: true,
        };

        manager.add_plan(plan);

        let license_id = Uuid::new_v4();
        let subscription = manager
            .create_subscription(license_id, "pro-monthly", None)
            .unwrap();

        assert!(manager.get_subscription(subscription.id).is_some());
    }
}
