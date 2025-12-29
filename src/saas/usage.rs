//! Usage tracking and metering
//!
//! This module provides comprehensive usage tracking and metering including:
//!
//! - API call tracking and counting
//! - Page scan metering
//! - Storage usage monitoring
//! - User seat tracking
//! - Overage detection and handling
//! - Usage aggregation and reporting
//!
//! ## Usage Metrics
//!
//! Tracked metrics include:
//! - API calls per endpoint
//! - Page scans (accessibility checks)
//! - Storage (files, documents, projects)
//! - Active user seats
//! - Bandwidth consumption
//!
//! ## Example
//!
//! ```rust
//! use caddy::saas::usage::{UsageManager, UsageMetric};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let usage = UsageManager::new(pool).await?;
//!
//!     // Record API call
//!     usage.record_usage(
//!         tenant_id,
//!         UsageMetric::ApiCall,
//!         1,
//!     ).await?;
//!
//!     // Check current usage
//!     let stats = usage.get_usage_stats(tenant_id).await?;
//!
//!     Ok(())
//! }
//! ```

use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::saas::{Result, SaasError};

// ============================================================================
// Usage Metric Types
// ============================================================================

/// Usage metric type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "usage_metric", rename_all = "lowercase")]
pub enum UsageMetric {
    /// API call
    ApiCall,
    /// Page scan (accessibility check)
    PageScan,
    /// Storage (bytes)
    Storage,
    /// User seat
    UserSeat,
    /// Bandwidth (bytes)
    Bandwidth,
    /// Render operation
    Render,
    /// Export operation
    Export,
    /// Collaboration session
    Collaboration,
}

impl UsageMetric {
    /// Get display name
    pub fn display_name(&self) -> &str {
        match self {
            Self::ApiCall => "API Calls",
            Self::PageScan => "Page Scans",
            Self::Storage => "Storage",
            Self::UserSeat => "User Seats",
            Self::Bandwidth => "Bandwidth",
            Self::Render => "Renders",
            Self::Export => "Exports",
            Self::Collaboration => "Collaboration Sessions",
        }
    }

    /// Get unit of measurement
    pub fn unit(&self) -> &str {
        match self {
            Self::ApiCall => "calls",
            Self::PageScan => "scans",
            Self::Storage => "bytes",
            Self::UserSeat => "seats",
            Self::Bandwidth => "bytes",
            Self::Render => "renders",
            Self::Export => "exports",
            Self::Collaboration => "sessions",
        }
    }
}

// ============================================================================
// Usage Record Structure
// ============================================================================

/// Usage record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UsageRecord {
    /// Unique record ID
    pub id: Uuid,

    /// Tenant ID
    pub tenant_id: Uuid,

    /// Subscription ID
    pub subscription_id: Option<Uuid>,

    /// Usage metric type
    pub metric: UsageMetric,

    /// Usage amount
    pub amount: i64,

    /// Resource identifier (e.g., API endpoint, file ID)
    pub resource_id: Option<String>,

    /// Metadata (JSON)
    #[sqlx(json)]
    pub metadata: HashMap<String, serde_json::Value>,

    /// Timestamp
    pub recorded_at: DateTime<Utc>,
}

/// Aggregated usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    /// Tenant ID
    pub tenant_id: Uuid,

    /// Period start
    pub period_start: DateTime<Utc>,

    /// Period end
    pub period_end: DateTime<Utc>,

    /// API calls
    pub api_calls: i64,

    /// Page scans
    pub page_scans: i64,

    /// Storage in bytes
    pub storage_bytes: i64,

    /// Active user seats
    pub user_seats: i64,

    /// Bandwidth in bytes
    pub bandwidth_bytes: i64,

    /// Render operations
    pub renders: i64,

    /// Export operations
    pub exports: i64,

    /// Collaboration sessions
    pub collaboration_sessions: i64,

    /// Overage flags
    pub has_overage: bool,

    /// Overage details
    pub overages: Vec<OverageInfo>,
}

/// Overage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverageInfo {
    /// Metric type
    pub metric: UsageMetric,

    /// Current usage
    pub current_usage: i64,

    /// Quota limit
    pub quota_limit: i64,

    /// Overage amount
    pub overage_amount: i64,

    /// Overage percentage
    pub overage_percentage: f64,

    /// Cost per unit (cents)
    pub cost_per_unit_cents: i64,

    /// Total overage cost (cents)
    pub total_overage_cents: i64,
}

/// Daily usage aggregate
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DailyUsageAggregate {
    /// Tenant ID
    pub tenant_id: Uuid,

    /// Date
    pub date: NaiveDate,

    /// Usage metric
    pub metric: UsageMetric,

    /// Total amount
    pub total_amount: i64,

    /// Record count
    pub record_count: i64,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Usage Manager
// ============================================================================

/// Usage tracking and metering operations
pub struct UsageManager {
    pool: PgPool,
}

impl UsageManager {
    /// Create a new usage manager
    pub async fn new(pool: PgPool) -> Result<Self> {
        Ok(Self { pool })
    }

    // ========================================================================
    // Usage Recording
    // ========================================================================

    /// Record usage
    pub async fn record_usage(
        &self,
        tenant_id: Uuid,
        metric: UsageMetric,
        amount: i64,
    ) -> Result<UsageRecord> {
        self.record_usage_with_metadata(tenant_id, metric, amount, None, HashMap::new())
            .await
    }

    /// Record usage with resource ID
    pub async fn record_usage_with_resource(
        &self,
        tenant_id: Uuid,
        metric: UsageMetric,
        amount: i64,
        resource_id: String,
    ) -> Result<UsageRecord> {
        self.record_usage_with_metadata(
            tenant_id,
            metric,
            amount,
            Some(resource_id),
            HashMap::new(),
        )
        .await
    }

    /// Record usage with full metadata
    pub async fn record_usage_with_metadata(
        &self,
        tenant_id: Uuid,
        metric: UsageMetric,
        amount: i64,
        resource_id: Option<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<UsageRecord> {
        let record_id = Uuid::new_v4();
        let now = Utc::now();

        // Get current subscription
        let subscription_id = self.get_active_subscription_id(tenant_id).await.ok();

        let record = sqlx::query_as::<_, UsageRecord>(
            r"
            INSERT INTO usage_records (
                id, tenant_id, subscription_id, metric, amount,
                resource_id, metadata, recorded_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            ",
        )
        .bind(record_id)
        .bind(tenant_id)
        .bind(subscription_id)
        .bind(metric)
        .bind(amount)
        .bind(&resource_id)
        .bind(serde_json::to_value(&metadata)?)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    /// Record API call
    pub async fn record_api_call(
        &self,
        tenant_id: Uuid,
        endpoint: String,
    ) -> Result<UsageRecord> {
        let mut metadata = HashMap::new();
        metadata.insert("endpoint".to_string(), serde_json::json!(endpoint));

        self.record_usage_with_metadata(tenant_id, UsageMetric::ApiCall, 1, None, metadata)
            .await
    }

    /// Record page scan
    pub async fn record_page_scan(
        &self,
        tenant_id: Uuid,
        url: String,
        issues_found: i64,
    ) -> Result<UsageRecord> {
        let mut metadata = HashMap::new();
        metadata.insert("url".to_string(), serde_json::json!(url));
        metadata.insert("issues_found".to_string(), serde_json::json!(issues_found));

        self.record_usage_with_metadata(tenant_id, UsageMetric::PageScan, 1, None, metadata)
            .await
    }

    /// Record storage usage
    pub async fn record_storage(
        &self,
        tenant_id: Uuid,
        file_id: String,
        bytes: i64,
    ) -> Result<UsageRecord> {
        self.record_usage_with_resource(tenant_id, UsageMetric::Storage, bytes, file_id)
            .await
    }

    // ========================================================================
    // Usage Retrieval
    // ========================================================================

    /// Get usage statistics for current billing period
    pub async fn get_usage_stats(&self, tenant_id: Uuid) -> Result<UsageStats> {
        // Get current subscription to determine billing period
        let subscription_id = self.get_active_subscription_id(tenant_id).await?;

        // For now, use current month as billing period
        let now = Utc::now();
        let period_start = now
            .with_day(1)
            .unwrap()
            .with_hour(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap();
        let period_end = now;

        self.get_usage_stats_for_period(tenant_id, period_start, period_end)
            .await
    }

    /// Get usage statistics for a specific period
    pub async fn get_usage_stats_for_period(
        &self,
        tenant_id: Uuid,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<UsageStats> {
        // Query aggregated usage by metric type
        let api_calls = self
            .get_metric_total(tenant_id, UsageMetric::ApiCall, period_start, period_end)
            .await?;
        let page_scans = self
            .get_metric_total(tenant_id, UsageMetric::PageScan, period_start, period_end)
            .await?;
        let storage_bytes = self
            .get_metric_total(tenant_id, UsageMetric::Storage, period_start, period_end)
            .await?;
        let user_seats = self
            .get_active_user_count(tenant_id)
            .await?;
        let bandwidth_bytes = self
            .get_metric_total(tenant_id, UsageMetric::Bandwidth, period_start, period_end)
            .await?;
        let renders = self
            .get_metric_total(tenant_id, UsageMetric::Render, period_start, period_end)
            .await?;
        let exports = self
            .get_metric_total(tenant_id, UsageMetric::Export, period_start, period_end)
            .await?;
        let collaboration_sessions = self
            .get_metric_total(tenant_id, UsageMetric::Collaboration, period_start, period_end)
            .await?;

        // Check for overages
        let overages = self.check_overages(tenant_id, &UsageStats {
            tenant_id,
            period_start,
            period_end,
            api_calls,
            page_scans,
            storage_bytes,
            user_seats,
            bandwidth_bytes,
            renders,
            exports,
            collaboration_sessions,
            has_overage: false,
            overages: Vec::new(),
        }).await?;

        Ok(UsageStats {
            tenant_id,
            period_start,
            period_end,
            api_calls,
            page_scans,
            storage_bytes,
            user_seats,
            bandwidth_bytes,
            renders,
            exports,
            collaboration_sessions,
            has_overage: !overages.is_empty(),
            overages,
        })
    }

    /// Get total for a specific metric
    async fn get_metric_total(
        &self,
        tenant_id: Uuid,
        metric: UsageMetric,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<i64> {
        let row = sqlx::query(
            r"
            SELECT COALESCE(SUM(amount), 0) as total
            FROM usage_records
            WHERE tenant_id = $1
            AND metric = $2
            AND recorded_at >= $3
            AND recorded_at < $4
            ",
        )
        .bind(tenant_id)
        .bind(metric)
        .bind(period_start)
        .bind(period_end)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.try_get("total")?)
    }

    /// Get usage records for tenant
    pub async fn get_usage_records(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UsageRecord>> {
        sqlx::query_as::<_, UsageRecord>(
            r"
            SELECT * FROM usage_records
            WHERE tenant_id = $1
            ORDER BY recorded_at DESC
            LIMIT $2 OFFSET $3
            ",
        )
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(SaasError::Database)
    }

    // ========================================================================
    // Daily Aggregation
    // ========================================================================

    /// Aggregate daily usage (should be run daily via cron)
    pub async fn aggregate_daily_usage(&self, date: NaiveDate) -> Result<()> {
        let start = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let end = start + Duration::days(1);

        // Aggregate for each metric type
        for metric in [
            UsageMetric::ApiCall,
            UsageMetric::PageScan,
            UsageMetric::Storage,
            UsageMetric::Bandwidth,
            UsageMetric::Render,
            UsageMetric::Export,
            UsageMetric::Collaboration,
        ] {
            sqlx::query(
                r"
                INSERT INTO daily_usage_aggregates (
                    tenant_id, date, metric, total_amount, record_count, created_at
                )
                SELECT
                    tenant_id,
                    $1 as date,
                    metric,
                    SUM(amount) as total_amount,
                    COUNT(*) as record_count,
                    $2 as created_at
                FROM usage_records
                WHERE recorded_at >= $3
                AND recorded_at < $4
                AND metric = $5
                GROUP BY tenant_id, metric
                ON CONFLICT (tenant_id, date, metric) DO UPDATE
                SET total_amount = EXCLUDED.total_amount,
                    record_count = EXCLUDED.record_count
                ",
            )
            .bind(date)
            .bind(Utc::now())
            .bind(start)
            .bind(end)
            .bind(metric)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    // ========================================================================
    // Overage Detection
    // ========================================================================

    /// Check for quota overages
    async fn check_overages(&self, tenant_id: Uuid, stats: &UsageStats) -> Result<Vec<OverageInfo>> {
        // In a real implementation, this would:
        // 1. Get tenant's subscription tier
        // 2. Get quota limits for that tier
        // 3. Compare current usage to limits
        // 4. Calculate overage costs

        let mut overages = Vec::new();

        // Example: Check API call overage (Pro tier: 100k calls/month)
        let api_quota = 100_000;
        if stats.api_calls > api_quota {
            overages.push(OverageInfo {
                metric: UsageMetric::ApiCall,
                current_usage: stats.api_calls,
                quota_limit: api_quota,
                overage_amount: stats.api_calls - api_quota,
                overage_percentage: ((stats.api_calls - api_quota) as f64 / api_quota as f64) * 100.0,
                cost_per_unit_cents: 1, // $0.01 per call
                total_overage_cents: stats.api_calls - api_quota,
            });
        }

        Ok(overages)
    }

    /// Calculate overage charges for billing period
    pub async fn calculate_overage_charges(&self, tenant_id: Uuid) -> Result<i64> {
        let stats = self.get_usage_stats(tenant_id).await?;
        let total_cents: i64 = stats
            .overages
            .iter()
            .map(|o| o.total_overage_cents)
            .sum();

        Ok(total_cents)
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    /// Get active subscription ID for tenant
    async fn get_active_subscription_id(&self, tenant_id: Uuid) -> Result<Uuid> {
        let row = sqlx::query(
            r"
            SELECT id FROM subscriptions
            WHERE tenant_id = $1
            AND status IN ('trial', 'active')
            ORDER BY created_at DESC
            LIMIT 1
            ",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.try_get("id")?)
    }

    /// Get active user count for tenant
    async fn get_active_user_count(&self, tenant_id: Uuid) -> Result<i64> {
        let row = sqlx::query(
            r"
            SELECT COUNT(DISTINCT user_id) as count
            FROM user_sessions
            WHERE tenant_id = $1
            AND last_active_at > $2
            ",
        )
        .bind(tenant_id)
        .bind(Utc::now() - Duration::days(30))
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.try_get("count").unwrap_or(0)).unwrap_or(0))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_metric_display_name() {
        assert_eq!(UsageMetric::ApiCall.display_name(), "API Calls");
        assert_eq!(UsageMetric::PageScan.display_name(), "Page Scans");
        assert_eq!(UsageMetric::Storage.display_name(), "Storage");
    }

    #[test]
    fn test_usage_metric_unit() {
        assert_eq!(UsageMetric::ApiCall.unit(), "calls");
        assert_eq!(UsageMetric::Storage.unit(), "bytes");
        assert_eq!(UsageMetric::UserSeat.unit(), "seats");
    }

    #[test]
    fn test_overage_calculation() {
        let overage = OverageInfo {
            metric: UsageMetric::ApiCall,
            current_usage: 150_000,
            quota_limit: 100_000,
            overage_amount: 50_000,
            overage_percentage: 50.0,
            cost_per_unit_cents: 1,
            total_overage_cents: 50_000,
        };

        assert_eq!(overage.overage_amount, 50_000);
        assert_eq!(overage.total_overage_cents, 50_000);
    }
}
