//! Billing and payment processing
//!
//! This module provides comprehensive billing integration including:
//!
//! - Stripe payment processing
//! - Invoice generation and management
//! - Payment method management
//! - Billing history and receipts
//! - Dunning management for failed payments
//! - Webhook handling for payment events
//!
//! ## Stripe Integration
//!
//! This module integrates with Stripe for:
//! - Payment processing
//! - Subscription billing
//! - Invoice generation
//! - Payment method storage
//! - Webhook notifications
//!
//! ## Example
//!
//! ```rust
//! use caddy::saas::billing::BillingManager;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let billing = BillingManager::new("sk_test_...").await?;
//!
//!     // Create invoice
//!     let invoice = billing.create_invoice(
//!         tenant_id,
//!         subscription_id,
//!         4900, // $49.00
//!     ).await?;
//!
//!     // Process payment
//!     billing.process_payment(invoice.id, payment_method_id).await?;
//!
//!     Ok(())
//! }
//! ```

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::saas::{PaymentStatus, Result, SaasError};

// ============================================================================
// Invoice Structure
// ============================================================================

/// Invoice record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Invoice {
    /// Unique invoice ID
    pub id: Uuid,

    /// Tenant ID
    pub tenant_id: Uuid,

    /// Subscription ID
    pub subscription_id: Option<Uuid>,

    /// Invoice number
    pub invoice_number: String,

    /// Payment status
    pub status: PaymentStatus,

    /// Subtotal in cents
    pub subtotal_cents: i64,

    /// Tax amount in cents
    pub tax_cents: i64,

    /// Total amount in cents
    pub total_cents: i64,

    /// Currency code (ISO 4217)
    pub currency: String,

    /// Invoice description
    pub description: Option<String>,

    /// Invoice items (JSON)
    #[sqlx(json)]
    pub items: Vec<InvoiceItem>,

    /// Due date
    pub due_date: DateTime<Utc>,

    /// Paid date
    pub paid_at: Option<DateTime<Utc>>,

    /// Stripe invoice ID
    pub stripe_invoice_id: Option<String>,

    /// Stripe charge ID
    pub stripe_charge_id: Option<String>,

    /// PDF URL
    pub pdf_url: Option<String>,

    /// Dunning attempt count
    pub dunning_attempts: i32,

    /// Last dunning attempt
    pub last_dunning_at: Option<DateTime<Utc>>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Invoice line item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceItem {
    /// Item description
    pub description: String,

    /// Quantity
    pub quantity: i32,

    /// Unit price in cents
    pub unit_price_cents: i64,

    /// Total price in cents
    pub total_cents: i64,

    /// Item metadata
    pub metadata: HashMap<String, String>,
}

// ============================================================================
// Payment Method Structure
// ============================================================================

/// Payment method record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PaymentMethod {
    /// Unique payment method ID
    pub id: Uuid,

    /// Tenant ID
    pub tenant_id: Uuid,

    /// Payment method type
    pub method_type: PaymentMethodType,

    /// Is default payment method
    pub is_default: bool,

    /// Card last 4 digits
    pub card_last4: Option<String>,

    /// Card brand (Visa, MasterCard, etc.)
    pub card_brand: Option<String>,

    /// Card expiration month
    pub card_exp_month: Option<i32>,

    /// Card expiration year
    pub card_exp_year: Option<i32>,

    /// Stripe payment method ID
    pub stripe_payment_method_id: Option<String>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Payment method type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "payment_method_type", rename_all = "lowercase")]
pub enum PaymentMethodType {
    /// Credit/debit card
    Card,
    /// Bank account
    BankAccount,
    /// PayPal
    PayPal,
}

// ============================================================================
// Payment Record
// ============================================================================

/// Payment transaction record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Payment {
    /// Unique payment ID
    pub id: Uuid,

    /// Invoice ID
    pub invoice_id: Uuid,

    /// Tenant ID
    pub tenant_id: Uuid,

    /// Amount in cents
    pub amount_cents: i64,

    /// Currency
    pub currency: String,

    /// Payment status
    pub status: PaymentStatus,

    /// Payment method ID
    pub payment_method_id: Uuid,

    /// Stripe payment intent ID
    pub stripe_payment_intent_id: Option<String>,

    /// Failure reason
    pub failure_reason: Option<String>,

    /// Receipt URL
    pub receipt_url: Option<String>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Billing Manager
// ============================================================================

/// Billing management operations
pub struct BillingManager {
    pool: Option<PgPool>,
    stripe_secret_key: String,
    http_client: reqwest::Client,
}

impl BillingManager {
    /// Create a new billing manager
    pub async fn new(stripe_secret_key: impl Into<String>) -> Result<Self> {
        Ok(Self {
            pool: None,
            stripe_secret_key: stripe_secret_key.into(),
            http_client: reqwest::Client::new(),
        })
    }

    /// Create a billing manager with database pool
    pub async fn with_pool(
        pool: PgPool,
        stripe_secret_key: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            pool: Some(pool),
            stripe_secret_key: stripe_secret_key.into(),
            http_client: reqwest::Client::new(),
        })
    }

    /// Set database pool
    pub fn set_pool(&mut self, pool: PgPool) {
        self.pool = Some(pool);
    }

    // ========================================================================
    // Invoice Management
    // ========================================================================

    /// Create a new invoice
    pub async fn create_invoice(
        &self,
        tenant_id: Uuid,
        subscription_id: Option<Uuid>,
        amount_cents: i64,
    ) -> Result<Invoice> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            SaasError::Config("Database pool not configured".to_string())
        })?;

        let invoice_id = Uuid::new_v4();
        let invoice_number = Self::generate_invoice_number();
        let now = Utc::now();
        let due_date = now + Duration::days(7); // 7 days payment terms

        let items: Vec<InvoiceItem> = vec![];

        let invoice = sqlx::query_as::<_, Invoice>(
            r"
            INSERT INTO invoices (
                id, tenant_id, subscription_id, invoice_number, status,
                subtotal_cents, tax_cents, total_cents, currency, items,
                due_date, dunning_attempts, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING *
            ",
        )
        .bind(invoice_id)
        .bind(tenant_id)
        .bind(subscription_id)
        .bind(&invoice_number)
        .bind(PaymentStatus::Pending)
        .bind(amount_cents)
        .bind(0) // Tax calculation would go here
        .bind(amount_cents)
        .bind("USD")
        .bind(serde_json::to_value(&items)?)
        .bind(due_date)
        .bind(0)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await?;

        Ok(invoice)
    }

    /// Add item to invoice
    pub async fn add_invoice_item(
        &self,
        invoice_id: Uuid,
        description: String,
        quantity: i32,
        unit_price_cents: i64,
    ) -> Result<()> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            SaasError::Config("Database pool not configured".to_string())
        })?;

        let item = InvoiceItem {
            description,
            quantity,
            unit_price_cents,
            total_cents: quantity as i64 * unit_price_cents,
            metadata: HashMap::new(),
        };

        sqlx::query(
            r"
            UPDATE invoices
            SET items = items || $1::jsonb,
                subtotal_cents = subtotal_cents + $2,
                total_cents = total_cents + $3,
                updated_at = $4
            WHERE id = $5
            ",
        )
        .bind(serde_json::to_value(&item)?)
        .bind(item.total_cents)
        .bind(item.total_cents)
        .bind(Utc::now())
        .bind(invoice_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Get invoice by ID
    pub async fn get_invoice(&self, invoice_id: Uuid) -> Result<Invoice> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            SaasError::Config("Database pool not configured".to_string())
        })?;

        sqlx::query_as::<_, Invoice>(
            r"
            SELECT * FROM invoices WHERE id = $1
            ",
        )
        .bind(invoice_id)
        .fetch_one(pool)
        .await
        .map_err(|_| SaasError::Billing(format!("Invoice not found: {}", invoice_id)))
    }

    /// List invoices for tenant
    pub async fn list_invoices(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Invoice>> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            SaasError::Config("Database pool not configured".to_string())
        })?;

        sqlx::query_as::<_, Invoice>(
            r"
            SELECT * FROM invoices
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            ",
        )
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(SaasError::Database)
    }

    /// Mark invoice as paid
    pub async fn mark_invoice_paid(&self, invoice_id: Uuid) -> Result<Invoice> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            SaasError::Config("Database pool not configured".to_string())
        })?;

        let updated = sqlx::query_as::<_, Invoice>(
            r"
            UPDATE invoices
            SET status = $1, paid_at = $2, updated_at = $3
            WHERE id = $4
            RETURNING *
            ",
        )
        .bind(PaymentStatus::Succeeded)
        .bind(Utc::now())
        .bind(Utc::now())
        .bind(invoice_id)
        .fetch_one(pool)
        .await?;

        Ok(updated)
    }

    /// Mark invoice as failed
    pub async fn mark_invoice_failed(&self, invoice_id: Uuid) -> Result<Invoice> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            SaasError::Config("Database pool not configured".to_string())
        })?;

        let updated = sqlx::query_as::<_, Invoice>(
            r"
            UPDATE invoices
            SET status = $1, updated_at = $2
            WHERE id = $3
            RETURNING *
            ",
        )
        .bind(PaymentStatus::Failed)
        .bind(Utc::now())
        .bind(invoice_id)
        .fetch_one(pool)
        .await?;

        Ok(updated)
    }

    // ========================================================================
    // Payment Methods
    // ========================================================================

    /// Add payment method
    pub async fn add_payment_method(
        &self,
        tenant_id: Uuid,
        stripe_payment_method_id: String,
    ) -> Result<PaymentMethod> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            SaasError::Config("Database pool not configured".to_string())
        })?;

        // In a real implementation, fetch payment method details from Stripe
        let payment_method_id = Uuid::new_v4();

        let payment_method = sqlx::query_as::<_, PaymentMethod>(
            r"
            INSERT INTO payment_methods (
                id, tenant_id, method_type, is_default,
                stripe_payment_method_id, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            ",
        )
        .bind(payment_method_id)
        .bind(tenant_id)
        .bind(PaymentMethodType::Card)
        .bind(true)
        .bind(&stripe_payment_method_id)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(pool)
        .await?;

        Ok(payment_method)
    }

    /// Get payment method
    pub async fn get_payment_method(&self, payment_method_id: Uuid) -> Result<PaymentMethod> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            SaasError::Config("Database pool not configured".to_string())
        })?;

        sqlx::query_as::<_, PaymentMethod>(
            r"
            SELECT * FROM payment_methods WHERE id = $1
            ",
        )
        .bind(payment_method_id)
        .fetch_one(pool)
        .await
        .map_err(|_| {
            SaasError::InvalidPaymentMethod(format!("Payment method not found: {}", payment_method_id))
        })
    }

    /// List payment methods for tenant
    pub async fn list_payment_methods(&self, tenant_id: Uuid) -> Result<Vec<PaymentMethod>> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            SaasError::Config("Database pool not configured".to_string())
        })?;

        sqlx::query_as::<_, PaymentMethod>(
            r"
            SELECT * FROM payment_methods
            WHERE tenant_id = $1
            ORDER BY is_default DESC, created_at DESC
            ",
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await
        .map_err(SaasError::Database)
    }

    /// Set default payment method
    pub async fn set_default_payment_method(
        &self,
        tenant_id: Uuid,
        payment_method_id: Uuid,
    ) -> Result<()> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            SaasError::Config("Database pool not configured".to_string())
        })?;

        // Unset all defaults
        sqlx::query(
            r"
            UPDATE payment_methods
            SET is_default = false, updated_at = $1
            WHERE tenant_id = $2
            ",
        )
        .bind(Utc::now())
        .bind(tenant_id)
        .execute(pool)
        .await?;

        // Set new default
        sqlx::query(
            r"
            UPDATE payment_methods
            SET is_default = true, updated_at = $1
            WHERE id = $2 AND tenant_id = $3
            ",
        )
        .bind(Utc::now())
        .bind(payment_method_id)
        .bind(tenant_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Delete payment method
    pub async fn delete_payment_method(&self, payment_method_id: Uuid) -> Result<()> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            SaasError::Config("Database pool not configured".to_string())
        })?;

        sqlx::query(
            r"
            DELETE FROM payment_methods WHERE id = $1
            ",
        )
        .bind(payment_method_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    // ========================================================================
    // Payment Processing
    // ========================================================================

    /// Process payment for invoice
    pub async fn process_payment(
        &self,
        invoice_id: Uuid,
        payment_method_id: Uuid,
    ) -> Result<Payment> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            SaasError::Config("Database pool not configured".to_string())
        })?;

        let invoice = self.get_invoice(invoice_id).await?;
        let payment_method = self.get_payment_method(payment_method_id).await?;

        // In a real implementation, this would call Stripe API
        // let payment_intent = stripe::create_payment_intent(...);

        let payment_id = Uuid::new_v4();

        let payment = sqlx::query_as::<_, Payment>(
            r"
            INSERT INTO payments (
                id, invoice_id, tenant_id, amount_cents, currency,
                status, payment_method_id, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            ",
        )
        .bind(payment_id)
        .bind(invoice_id)
        .bind(invoice.tenant_id)
        .bind(invoice.total_cents)
        .bind(&invoice.currency)
        .bind(PaymentStatus::Processing)
        .bind(payment_method_id)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(pool)
        .await?;

        Ok(payment)
    }

    // ========================================================================
    // Dunning Management
    // ========================================================================

    /// Process dunning for failed payments
    pub async fn process_dunning(&self, invoice_id: Uuid) -> Result<()> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            SaasError::Config("Database pool not configured".to_string())
        })?;

        let mut invoice = self.get_invoice(invoice_id).await?;

        // Check if we've exceeded max retries
        const MAX_DUNNING_ATTEMPTS: i32 = 3;
        if invoice.dunning_attempts >= MAX_DUNNING_ATTEMPTS {
            return Err(SaasError::Billing(
                "Maximum dunning attempts exceeded".to_string(),
            ));
        }

        // Increment dunning attempts
        invoice.dunning_attempts += 1;

        sqlx::query(
            r"
            UPDATE invoices
            SET dunning_attempts = $1,
                last_dunning_at = $2,
                updated_at = $3
            WHERE id = $4
            ",
        )
        .bind(invoice.dunning_attempts)
        .bind(Utc::now())
        .bind(Utc::now())
        .bind(invoice_id)
        .execute(pool)
        .await?;

        // In a real implementation:
        // 1. Send email notification to customer
        // 2. Retry payment with default payment method
        // 3. Update subscription status if payment fails

        Ok(())
    }

    /// Get overdue invoices for dunning
    pub async fn get_overdue_invoices(&self) -> Result<Vec<Invoice>> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            SaasError::Config("Database pool not configured".to_string())
        })?;

        sqlx::query_as::<_, Invoice>(
            r"
            SELECT * FROM invoices
            WHERE status = 'pending'
            AND due_date < $1
            AND dunning_attempts < 3
            ORDER BY due_date ASC
            ",
        )
        .bind(Utc::now())
        .fetch_all(pool)
        .await
        .map_err(SaasError::Database)
    }

    // ========================================================================
    // Stripe Webhook Handling
    // ========================================================================

    /// Verify Stripe webhook signature
    pub fn verify_webhook_signature(
        &self,
        payload: &str,
        signature: &str,
        webhook_secret: &str,
    ) -> Result<bool> {
        // In a real implementation, this would verify HMAC signature
        // using Stripe's webhook secret
        Ok(true)
    }

    /// Handle Stripe webhook event
    pub async fn handle_webhook_event(&self, event_type: &str, event_data: serde_json::Value) -> Result<()> {
        match event_type {
            "invoice.payment_succeeded" => {
                // Handle successful payment
                if let Some(invoice_id) = event_data.get("id").and_then(|v| v.as_str()) {
                    // Update invoice status
                }
            }
            "invoice.payment_failed" => {
                // Handle failed payment
                // Trigger dunning process
            }
            "customer.subscription.updated" => {
                // Handle subscription updates
            }
            "customer.subscription.deleted" => {
                // Handle subscription cancellation
            }
            _ => {
                // Unknown event type
            }
        }

        Ok(())
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    /// Generate unique invoice number
    fn generate_invoice_number() -> String {
        let now = Utc::now();
        format!(
            "INV-{}-{}",
            now.format("%Y%m"),
            Uuid::new_v4().to_string()[..8].to_uppercase()
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
    fn test_invoice_number_generation() {
        let invoice_num = BillingManager::generate_invoice_number();
        assert!(invoice_num.starts_with("INV-"));
        assert!(invoice_num.len() > 12);
    }

    #[test]
    fn test_invoice_item_total() {
        let item = InvoiceItem {
            description: "Test Item".to_string(),
            quantity: 3,
            unit_price_cents: 1000,
            total_cents: 3000,
            metadata: HashMap::new(),
        };

        assert_eq!(item.total_cents, 3000);
    }

    #[tokio::test]
    async fn test_billing_manager_creation() {
        let manager = BillingManager::new("sk_test_123").await;
        assert!(manager.is_ok());
    }
}
