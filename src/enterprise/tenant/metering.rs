//! Billing and Metering
//!
//! Resource usage tracking, API call counting, storage metering, and billing event generation.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use parking_lot::RwLock;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use chrono::{DateTime, Utc};

use super::context::TenantId;

/// Metering errors
#[derive(Error, Debug)]
pub enum MeteringError {
    #[error("Metering not enabled for tenant: {0}")]
    NotEnabled(String),

    #[error("Invalid metric type: {0}")]
    InvalidMetric(String),

    #[error("Failed to record metric: {0}")]
    RecordingFailed(String),

    #[error("Billing period not found")]
    PeriodNotFound,
}

pub type MeteringResult<T> = Result<T, MeteringError>;

/// Metered resource type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricType {
    /// API calls/requests
    ApiCalls,
    /// Storage in bytes
    StorageBytes,
    /// Compute time in milliseconds
    ComputeTime,
    /// Network egress in bytes
    NetworkEgress,
    /// Network ingress in bytes
    NetworkIngress,
    /// Active users
    ActiveUsers,
    /// Collaboration sessions
    CollaborationSessions,
    /// AI operations
    AiOperations,
    /// Rendering operations
    RenderOperations,
    /// File exports
    FileExports,
    /// Custom metric
    Custom(u8),
}

impl std::fmt::Display for MetricType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricType::ApiCalls => write!(f, "api_calls"),
            MetricType::StorageBytes => write!(f, "storage_bytes"),
            MetricType::ComputeTime => write!(f, "compute_time"),
            MetricType::NetworkEgress => write!(f, "network_egress"),
            MetricType::NetworkIngress => write!(f, "network_ingress"),
            MetricType::ActiveUsers => write!(f, "active_users"),
            MetricType::CollaborationSessions => write!(f, "collaboration_sessions"),
            MetricType::AiOperations => write!(f, "ai_operations"),
            MetricType::RenderOperations => write!(f, "render_operations"),
            MetricType::FileExports => write!(f, "file_exports"),
            MetricType::Custom(id) => write!(f, "custom_{}", id),
        }
    }
}

/// Usage record for a specific metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    /// Tenant identifier
    pub tenant_id: TenantId,
    /// Metric type
    pub metric_type: MetricType,
    /// Metric value
    pub value: u64,
    /// Timestamp of record
    pub timestamp: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl UsageRecord {
    /// Create a new usage record
    pub fn new(tenant_id: TenantId, metric_type: MetricType, value: u64) -> Self {
        Self {
            tenant_id,
            metric_type,
            value,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the record
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Aggregated usage for a billing period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingPeriodUsage {
    /// Tenant identifier
    pub tenant_id: TenantId,
    /// Period start
    pub period_start: DateTime<Utc>,
    /// Period end
    pub period_end: DateTime<Utc>,
    /// Usage by metric type
    pub usage: HashMap<MetricType, u64>,
    /// Peak values (max observed)
    pub peaks: HashMap<MetricType, u64>,
    /// Total cost in cents (if pricing configured)
    pub total_cost_cents: Option<u64>,
}

impl BillingPeriodUsage {
    /// Create a new billing period
    pub fn new(tenant_id: TenantId, period_start: DateTime<Utc>, period_end: DateTime<Utc>) -> Self {
        Self {
            tenant_id,
            period_start,
            period_end,
            usage: HashMap::new(),
            peaks: HashMap::new(),
            total_cost_cents: None,
        }
    }

    /// Add usage for a metric
    pub fn add_usage(&mut self, metric_type: MetricType, value: u64) {
        *self.usage.entry(metric_type).or_insert(0) += value;

        // Track peak
        let peak = self.peaks.entry(metric_type).or_insert(0);
        if value > *peak {
            *peak = value;
        }
    }

    /// Get usage for a specific metric
    pub fn get_usage(&self, metric_type: MetricType) -> u64 {
        self.usage.get(&metric_type).copied().unwrap_or(0)
    }

    /// Calculate cost based on pricing
    pub fn calculate_cost(&mut self, pricing: &PricingModel) -> u64 {
        let mut total_cents = 0u64;

        for (metric_type, usage) in &self.usage {
            if let Some(price) = pricing.get_price(metric_type) {
                let cost = (usage * price.price_per_unit) / price.unit_size;
                total_cents += cost;
            }
        }

        self.total_cost_cents = Some(total_cents);
        total_cents
    }
}

/// Pricing for a metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPricing {
    /// Metric type
    pub metric_type: MetricType,
    /// Price in cents per unit
    pub price_per_unit: u64,
    /// Unit size (e.g., per 1000 API calls)
    pub unit_size: u64,
    /// Free tier allowance
    pub free_tier: u64,
}

/// Pricing model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingModel {
    /// Name of pricing model
    pub name: String,
    /// Prices by metric
    pub prices: HashMap<MetricType, MetricPricing>,
}

impl PricingModel {
    /// Create a new pricing model
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            prices: HashMap::new(),
        }
    }

    /// Add pricing for a metric
    pub fn add_price(&mut self, pricing: MetricPricing) {
        self.prices.insert(pricing.metric_type, pricing);
    }

    /// Get pricing for a metric
    pub fn get_price(&self, metric_type: &MetricType) -> Option<&MetricPricing> {
        self.prices.get(metric_type)
    }

    /// Standard PAYG pricing
    pub fn pay_as_you_go() -> Self {
        let mut model = Self::new("Pay As You Go");

        model.add_price(MetricPricing {
            metric_type: MetricType::ApiCalls,
            price_per_unit: 100, // $1 per 1000 calls
            unit_size: 1000,
            free_tier: 10_000,
        });

        model.add_price(MetricPricing {
            metric_type: MetricType::StorageBytes,
            price_per_unit: 20, // $0.20 per GB
            unit_size: 1024 * 1024 * 1024,
            free_tier: 5 * 1024 * 1024 * 1024, // 5 GB free
        });

        model.add_price(MetricPricing {
            metric_type: MetricType::ComputeTime,
            price_per_unit: 500, // $5 per hour
            unit_size: 3_600_000, // milliseconds in hour
            free_tier: 600_000, // 10 minutes free
        });

        model.add_price(MetricPricing {
            metric_type: MetricType::NetworkEgress,
            price_per_unit: 10, // $0.10 per GB
            unit_size: 1024 * 1024 * 1024,
            free_tier: 10 * 1024 * 1024 * 1024, // 10 GB free
        });

        model
    }
}

/// Billing event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingEvent {
    /// Event ID
    pub id: uuid::Uuid,
    /// Tenant identifier
    pub tenant_id: TenantId,
    /// Event type
    pub event_type: BillingEventType,
    /// Amount in cents
    pub amount_cents: u64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Description
    pub description: String,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Billing event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BillingEventType {
    /// Usage charge
    Usage,
    /// Subscription charge
    Subscription,
    /// Credit applied
    Credit,
    /// Refund issued
    Refund,
    /// Payment received
    Payment,
}

/// Metering manager
pub struct MeteringManager {
    /// Usage records buffer
    records: Arc<RwLock<Vec<UsageRecord>>>,
    /// Current period usage by tenant
    current_usage: DashMap<TenantId, Arc<RwLock<BillingPeriodUsage>>>,
    /// Historical billing periods
    history: DashMap<TenantId, Vec<BillingPeriodUsage>>,
    /// Billing events
    events: Arc<RwLock<Vec<BillingEvent>>>,
    /// Pricing model
    pricing: Arc<RwLock<PricingModel>>,
    /// Period duration
    period_duration: Duration,
}

impl MeteringManager {
    /// Create a new metering manager
    pub fn new(pricing: PricingModel) -> Self {
        Self {
            records: Arc::new(RwLock::new(Vec::new())),
            current_usage: DashMap::new(),
            history: DashMap::new(),
            events: Arc::new(RwLock::new(Vec::new())),
            pricing: Arc::new(RwLock::new(pricing)),
            period_duration: Duration::from_secs(30 * 24 * 60 * 60), // 30 days
        }
    }

    /// Initialize metering for a tenant
    pub fn initialize_tenant(&self, tenant_id: TenantId) {
        let period_start = Utc::now();
        let period_end = period_start + chrono::Duration::seconds(self.period_duration.as_secs() as i64);

        let usage = BillingPeriodUsage::new(tenant_id.clone(), period_start, period_end);
        self.current_usage.insert(tenant_id, Arc::new(RwLock::new(usage)));
    }

    /// Record usage for a tenant
    pub fn record_usage(
        &self,
        tenant_id: &TenantId,
        metric_type: MetricType,
        value: u64,
    ) -> MeteringResult<()> {
        // Create usage record
        let record = UsageRecord::new(tenant_id.clone(), metric_type, value);

        // Add to buffer
        self.records.write().push(record.clone());

        // Update current period
        if let Some(usage_ref) = self.current_usage.get(tenant_id) {
            let mut usage = usage_ref.write();
            usage.add_usage(metric_type, value);
        } else {
            self.initialize_tenant(tenant_id.clone());
            if let Some(usage_ref) = self.current_usage.get(tenant_id) {
                usage_ref.write().add_usage(metric_type, value);
            }
        }

        Ok(())
    }

    /// Get current period usage for a tenant
    pub fn get_current_usage(&self, tenant_id: &TenantId) -> Option<BillingPeriodUsage> {
        self.current_usage
            .get(tenant_id)
            .map(|usage| usage.read().clone())
    }

    /// Finalize current billing period and start new one
    pub fn finalize_period(&self, tenant_id: &TenantId) -> MeteringResult<BillingPeriodUsage> {
        if let Some(usage_ref) = self.current_usage.get(tenant_id) {
            let mut usage = usage_ref.write().clone();

            // Calculate cost
            let pricing = self.pricing.read();
            usage.calculate_cost(&pricing);

            // Generate billing event
            if let Some(cost) = usage.total_cost_cents {
                if cost > 0 {
                    let event = BillingEvent {
                        id: uuid::Uuid::new_v4(),
                        tenant_id: tenant_id.clone(),
                        event_type: BillingEventType::Usage,
                        amount_cents: cost,
                        timestamp: Utc::now(),
                        description: format!("Usage charges for period {} to {}",
                            usage.period_start.format("%Y-%m-%d"),
                            usage.period_end.format("%Y-%m-%d")),
                        metadata: HashMap::new(),
                    };
                    self.events.write().push(event);
                }
            }

            // Save to history
            self.history
                .entry(tenant_id.clone())
                .or_insert_with(Vec::new)
                .push(usage.clone());

            // Start new period
            let period_start = Utc::now();
            let period_end = period_start + chrono::Duration::seconds(self.period_duration.as_secs() as i64);
            let new_usage = BillingPeriodUsage::new(tenant_id.clone(), period_start, period_end);
            *usage_ref.write() = new_usage;

            Ok(usage)
        } else {
            Err(MeteringError::PeriodNotFound)
        }
    }

    /// Get historical usage
    pub fn get_history(&self, tenant_id: &TenantId) -> Vec<BillingPeriodUsage> {
        self.history
            .get(tenant_id)
            .map(|h| h.clone())
            .unwrap_or_default()
    }

    /// Get billing events for a tenant
    pub fn get_billing_events(&self, tenant_id: &TenantId) -> Vec<BillingEvent> {
        self.events
            .read()
            .iter()
            .filter(|e| &e.tenant_id == tenant_id)
            .cloned()
            .collect()
    }

    /// Add a billing event
    pub fn add_billing_event(&self, event: BillingEvent) {
        self.events.write().push(event);
    }

    /// Flush buffered records (for persistence)
    pub fn flush_records(&self) -> Vec<UsageRecord> {
        let mut records = self.records.write();
        let flushed = records.clone();
        records.clear();
        flushed
    }

    /// Get total cost for a tenant (current period)
    pub fn get_current_cost(&self, tenant_id: &TenantId) -> Option<u64> {
        self.current_usage.get(tenant_id).and_then(|usage_ref| {
            let mut usage = usage_ref.write();
            let pricing = self.pricing.read();
            Some(usage.calculate_cost(&pricing))
        })
    }

    /// Update pricing model
    pub fn update_pricing(&self, pricing: PricingModel) {
        *self.pricing.write() = pricing;
    }
}

impl Default for MeteringManager {
    fn default() -> Self {
        Self::new(PricingModel::pay_as_you_go())
    }
}

/// Helper trait for automatic usage tracking
pub trait Metered {
    /// Record metric automatically
    fn record(&self, manager: &MeteringManager, tenant_id: &TenantId, value: u64) -> MeteringResult<()>;
}

/// API call tracker
pub struct ApiCallTracker;

impl Metered for ApiCallTracker {
    fn record(&self, manager: &MeteringManager, tenant_id: &TenantId, value: u64) -> MeteringResult<()> {
        manager.record_usage(tenant_id, MetricType::ApiCalls, value)
    }
}

/// Storage tracker
pub struct StorageTracker;

impl Metered for StorageTracker {
    fn record(&self, manager: &MeteringManager, tenant_id: &TenantId, value: u64) -> MeteringResult<()> {
        manager.record_usage(tenant_id, MetricType::StorageBytes, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_usage_recording() {
        let manager = MeteringManager::default();
        let tenant_id = TenantId::new_org(Uuid::new_v4());

        manager.initialize_tenant(tenant_id.clone());
        manager.record_usage(&tenant_id, MetricType::ApiCalls, 100).unwrap();
        manager.record_usage(&tenant_id, MetricType::StorageBytes, 1024).unwrap();

        let usage = manager.get_current_usage(&tenant_id).unwrap();
        assert_eq!(usage.get_usage(MetricType::ApiCalls), 100);
        assert_eq!(usage.get_usage(MetricType::StorageBytes), 1024);
    }

    #[test]
    fn test_cost_calculation() {
        let pricing = PricingModel::pay_as_you_go();
        let tenant_id = TenantId::new_org(Uuid::new_v4());

        let mut usage = BillingPeriodUsage::new(
            tenant_id.clone(),
            Utc::now(),
            Utc::now() + chrono::Duration::days(30),
        );

        // 20,000 API calls (10,000 free, 10,000 charged at $1/1000)
        usage.add_usage(MetricType::ApiCalls, 20_000);

        let cost = usage.calculate_cost(&pricing);
        assert!(cost > 0); // Should have some cost after free tier
    }

    #[test]
    fn test_period_finalization() {
        let manager = MeteringManager::default();
        let tenant_id = TenantId::new_org(Uuid::new_v4());

        manager.initialize_tenant(tenant_id.clone());
        manager.record_usage(&tenant_id, MetricType::ApiCalls, 15_000).unwrap();

        let finalized = manager.finalize_period(&tenant_id).unwrap();
        assert_eq!(finalized.get_usage(MetricType::ApiCalls), 15_000);
        assert!(finalized.total_cost_cents.is_some());

        // New period should be empty
        let current = manager.get_current_usage(&tenant_id).unwrap();
        assert_eq!(current.get_usage(MetricType::ApiCalls), 0);
    }

    #[test]
    fn test_billing_events() {
        let manager = MeteringManager::default();
        let tenant_id = TenantId::new_org(Uuid::new_v4());

        let event = BillingEvent {
            id: Uuid::new_v4(),
            tenant_id: tenant_id.clone(),
            event_type: BillingEventType::Payment,
            amount_cents: 10_000,
            timestamp: Utc::now(),
            description: "Payment received".to_string(),
            metadata: HashMap::new(),
        };

        manager.add_billing_event(event);

        let events = manager.get_billing_events(&tenant_id);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].amount_cents, 10_000);
    }

    #[test]
    fn test_record_flushing() {
        let manager = MeteringManager::default();
        let tenant_id = TenantId::new_org(Uuid::new_v4());

        manager.initialize_tenant(tenant_id.clone());
        manager.record_usage(&tenant_id, MetricType::ApiCalls, 100).unwrap();
        manager.record_usage(&tenant_id, MetricType::ApiCalls, 200).unwrap();

        let records = manager.flush_records();
        assert_eq!(records.len(), 2);

        // Buffer should be empty after flush
        let records2 = manager.flush_records();
        assert_eq!(records2.len(), 0);
    }

    #[test]
    fn test_peak_tracking() {
        let tenant_id = TenantId::new_org(Uuid::new_v4());
        let mut usage = BillingPeriodUsage::new(
            tenant_id.clone(),
            Utc::now(),
            Utc::now() + chrono::Duration::days(30),
        );

        usage.add_usage(MetricType::ActiveUsers, 10);
        usage.add_usage(MetricType::ActiveUsers, 50);
        usage.add_usage(MetricType::ActiveUsers, 30);

        assert_eq!(usage.get_usage(MetricType::ActiveUsers), 90); // Total
        assert_eq!(*usage.peaks.get(&MetricType::ActiveUsers).unwrap(), 50); // Peak
    }
}
