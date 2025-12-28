//! Marketplace analytics and tracking
//!
//! This module provides comprehensive analytics including download tracking,
//! usage analytics, revenue tracking, and developer dashboard metrics.

use super::{MarketplaceError, Result};
use chrono::{DateTime, Duration, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use uuid::Uuid;

/// Analytics event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnalyticsEvent {
    /// Plugin page view
    PageView {
        plugin_id: Uuid,
        user_id: Option<Uuid>,
        referrer: Option<String>,
    },

    /// Plugin download
    Download {
        plugin_id: Uuid,
        user_id: Uuid,
        version: String,
    },

    /// Plugin installation
    Install {
        plugin_id: Uuid,
        user_id: Uuid,
        version: String,
        platform: String,
    },

    /// Plugin uninstallation
    Uninstall {
        plugin_id: Uuid,
        user_id: Uuid,
        version: String,
        reason: Option<String>,
    },

    /// Plugin activation
    Activation {
        plugin_id: Uuid,
        user_id: Uuid,
    },

    /// Plugin deactivation
    Deactivation {
        plugin_id: Uuid,
        user_id: Uuid,
    },

    /// Purchase
    Purchase {
        plugin_id: Uuid,
        user_id: Uuid,
        amount_cents: u32,
        currency: String,
    },

    /// Refund
    Refund {
        plugin_id: Uuid,
        user_id: Uuid,
        amount_cents: u32,
        reason: String,
    },

    /// Review submitted
    ReviewSubmitted {
        plugin_id: Uuid,
        user_id: Uuid,
        rating: u8,
    },

    /// Support request
    SupportRequest {
        plugin_id: Uuid,
        user_id: Uuid,
        category: String,
    },

    /// Error/crash report
    Error {
        plugin_id: Uuid,
        user_id: Option<Uuid>,
        error_type: String,
        version: String,
    },
}

impl AnalyticsEvent {
    /// Get plugin ID for the event
    pub fn plugin_id(&self) -> Uuid {
        match self {
            Self::PageView { plugin_id, .. }
            | Self::Download { plugin_id, .. }
            | Self::Install { plugin_id, .. }
            | Self::Uninstall { plugin_id, .. }
            | Self::Activation { plugin_id, .. }
            | Self::Deactivation { plugin_id, .. }
            | Self::Purchase { plugin_id, .. }
            | Self::Refund { plugin_id, .. }
            | Self::ReviewSubmitted { plugin_id, .. }
            | Self::SupportRequest { plugin_id, .. }
            | Self::Error { plugin_id, .. } => *plugin_id,
        }
    }

    /// Get event type name
    pub fn event_type(&self) -> &str {
        match self {
            Self::PageView { .. } => "page_view",
            Self::Download { .. } => "download",
            Self::Install { .. } => "install",
            Self::Uninstall { .. } => "uninstall",
            Self::Activation { .. } => "activation",
            Self::Deactivation { .. } => "deactivation",
            Self::Purchase { .. } => "purchase",
            Self::Refund { .. } => "refund",
            Self::ReviewSubmitted { .. } => "review_submitted",
            Self::SupportRequest { .. } => "support_request",
            Self::Error { .. } => "error",
        }
    }
}

/// Analytics event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord {
    /// Event ID
    pub id: Uuid,

    /// Event data
    pub event: AnalyticsEvent,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// User agent
    pub user_agent: Option<String>,

    /// IP address (anonymized)
    pub ip_hash: Option<String>,

    /// Session ID
    pub session_id: Option<Uuid>,
}

impl EventRecord {
    /// Create a new event record
    pub fn new(event: AnalyticsEvent) -> Self {
        Self {
            id: Uuid::new_v4(),
            event,
            timestamp: Utc::now(),
            user_agent: None,
            ip_hash: None,
            session_id: None,
        }
    }

    /// Create with metadata
    pub fn with_metadata(
        event: AnalyticsEvent,
        user_agent: Option<String>,
        ip_hash: Option<String>,
        session_id: Option<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            event,
            timestamp: Utc::now(),
            user_agent,
            ip_hash,
            session_id,
        }
    }
}

/// Analytics tracker
#[derive(Debug)]
pub struct AnalyticsTracker {
    /// Event storage (plugin_id -> events)
    events: Arc<RwLock<HashMap<Uuid, VecDeque<EventRecord>>>>,

    /// Global event log
    global_events: Arc<RwLock<VecDeque<EventRecord>>>,

    /// Configuration
    config: Arc<RwLock<TrackerConfig>>,
}

impl AnalyticsTracker {
    /// Create a new analytics tracker
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(HashMap::new())),
            global_events: Arc::new(RwLock::new(VecDeque::new())),
            config: Arc::new(RwLock::new(TrackerConfig::default())),
        }
    }

    /// Track an event
    pub fn track(&self, event: AnalyticsEvent) -> Result<Uuid> {
        let record = EventRecord::new(event.clone());
        let event_id = record.id;
        let plugin_id = event.plugin_id();

        // Add to plugin-specific events
        let mut events = self.events.write();
        let plugin_events = events.entry(plugin_id).or_insert_with(VecDeque::new);

        plugin_events.push_back(record.clone());

        // Enforce retention limit
        let max_events = self.config.read().max_events_per_plugin;
        while plugin_events.len() > max_events {
            plugin_events.pop_front();
        }

        drop(events);

        // Add to global log
        let mut global = self.global_events.write();
        global.push_back(record);

        let max_global = self.config.read().max_global_events;
        while global.len() > max_global {
            global.pop_front();
        }

        Ok(event_id)
    }

    /// Track event with metadata
    pub fn track_with_metadata(
        &self,
        event: AnalyticsEvent,
        user_agent: Option<String>,
        ip: Option<String>,
        session_id: Option<Uuid>,
    ) -> Result<Uuid> {
        // Anonymize IP
        let ip_hash = ip.map(|ip| self.hash_ip(&ip));

        let record = EventRecord::with_metadata(event.clone(), user_agent, ip_hash, session_id);
        let event_id = record.id;
        let plugin_id = event.plugin_id();

        // Add to plugin-specific events
        let mut events = self.events.write();
        let plugin_events = events.entry(plugin_id).or_insert_with(VecDeque::new);

        plugin_events.push_back(record.clone());

        let max_events = self.config.read().max_events_per_plugin;
        while plugin_events.len() > max_events {
            plugin_events.pop_front();
        }

        drop(events);

        // Add to global log
        let mut global = self.global_events.write();
        global.push_back(record);

        let max_global = self.config.read().max_global_events;
        while global.len() > max_global {
            global.pop_front();
        }

        Ok(event_id)
    }

    /// Get events for plugin
    pub fn get_plugin_events(&self, plugin_id: Uuid) -> Vec<EventRecord> {
        let events = self.events.read();
        if let Some(plugin_events) = events.get(&plugin_id) {
            plugin_events.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Get events for plugin within time range
    pub fn get_plugin_events_range(
        &self,
        plugin_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<EventRecord> {
        let events = self.events.read();
        if let Some(plugin_events) = events.get(&plugin_id) {
            plugin_events
                .iter()
                .filter(|e| e.timestamp >= start && e.timestamp <= end)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get event counts by type
    pub fn get_event_counts(&self, plugin_id: Uuid) -> HashMap<String, u64> {
        let events = self.get_plugin_events(plugin_id);
        let mut counts = HashMap::new();

        for event in events {
            let event_type = event.event.event_type().to_string();
            *counts.entry(event_type).or_insert(0) += 1;
        }

        counts
    }

    /// Anonymize IP address
    fn hash_ip(&self, ip: &str) -> String {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(ip.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Clear old events
    pub fn cleanup_old_events(&self, older_than: Duration) -> Result<usize> {
        let cutoff = Utc::now() - older_than;
        let mut removed = 0;

        // Clean plugin events
        let mut events = self.events.write();
        for plugin_events in events.values_mut() {
            let before = plugin_events.len();
            plugin_events.retain(|e| e.timestamp >= cutoff);
            removed += before - plugin_events.len();
        }

        drop(events);

        // Clean global events
        let mut global = self.global_events.write();
        let before = global.len();
        global.retain(|e| e.timestamp >= cutoff);
        removed += before - global.len();

        Ok(removed)
    }
}

impl Default for AnalyticsTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Tracker configuration
#[derive(Debug, Clone)]
struct TrackerConfig {
    /// Maximum events per plugin
    max_events_per_plugin: usize,

    /// Maximum global events
    max_global_events: usize,

    /// Retention period (days)
    retention_days: i64,
}

impl Default for TrackerConfig {
    fn default() -> Self {
        Self {
            max_events_per_plugin: 10000,
            max_global_events: 100000,
            retention_days: 90,
        }
    }
}

/// Revenue tracker
#[derive(Debug)]
pub struct RevenueTracker {
    /// Revenue records (plugin_id -> records)
    records: Arc<RwLock<HashMap<Uuid, Vec<RevenueRecord>>>>,
}

impl RevenueTracker {
    /// Create a new revenue tracker
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a purchase
    pub fn record_purchase(
        &self,
        plugin_id: Uuid,
        user_id: Uuid,
        amount_cents: u32,
        currency: String,
    ) -> Result<Uuid> {
        let record = RevenueRecord {
            id: Uuid::new_v4(),
            plugin_id,
            user_id,
            transaction_type: TransactionType::Purchase,
            amount_cents: amount_cents as i64,
            currency,
            timestamp: Utc::now(),
            notes: None,
        };

        let record_id = record.id;

        self.records.write()
            .entry(plugin_id)
            .or_insert_with(Vec::new)
            .push(record);

        Ok(record_id)
    }

    /// Record a refund
    pub fn record_refund(
        &self,
        plugin_id: Uuid,
        user_id: Uuid,
        amount_cents: u32,
        currency: String,
        reason: String,
    ) -> Result<Uuid> {
        let record = RevenueRecord {
            id: Uuid::new_v4(),
            plugin_id,
            user_id,
            transaction_type: TransactionType::Refund,
            amount_cents: -(amount_cents as i64),
            currency,
            timestamp: Utc::now(),
            notes: Some(reason),
        };

        let record_id = record.id;

        self.records.write()
            .entry(plugin_id)
            .or_insert_with(Vec::new)
            .push(record);

        Ok(record_id)
    }

    /// Get total revenue for plugin
    pub fn get_total_revenue(&self, plugin_id: Uuid) -> i64 {
        let records = self.records.read();

        if let Some(plugin_records) = records.get(&plugin_id) {
            plugin_records.iter()
                .map(|r| r.amount_cents)
                .sum()
        } else {
            0
        }
    }

    /// Get revenue for date range
    pub fn get_revenue_range(
        &self,
        plugin_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> i64 {
        let records = self.records.read();

        if let Some(plugin_records) = records.get(&plugin_id) {
            plugin_records.iter()
                .filter(|r| r.timestamp >= start && r.timestamp <= end)
                .map(|r| r.amount_cents)
                .sum()
        } else {
            0
        }
    }

    /// Get revenue statistics
    pub fn get_statistics(&self, plugin_id: Uuid) -> RevenueStatistics {
        let records = self.records.read();

        let mut stats = RevenueStatistics::default();

        if let Some(plugin_records) = records.get(&plugin_id) {
            for record in plugin_records {
                match record.transaction_type {
                    TransactionType::Purchase => {
                        stats.total_purchases += 1;
                        stats.gross_revenue_cents += record.amount_cents;
                    }
                    TransactionType::Refund => {
                        stats.total_refunds += 1;
                        stats.refund_amount_cents += record.amount_cents.abs();
                    }
                }
            }

            stats.net_revenue_cents = stats.gross_revenue_cents - stats.refund_amount_cents;
        }

        stats
    }
}

impl Default for RevenueTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Revenue record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueRecord {
    /// Record ID
    pub id: Uuid,

    /// Plugin ID
    pub plugin_id: Uuid,

    /// User ID
    pub user_id: Uuid,

    /// Transaction type
    pub transaction_type: TransactionType,

    /// Amount in cents (negative for refunds)
    pub amount_cents: i64,

    /// Currency code
    pub currency: String,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Optional notes
    pub notes: Option<String>,
}

/// Transaction type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionType {
    /// Purchase
    Purchase,

    /// Refund
    Refund,
}

/// Revenue statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RevenueStatistics {
    /// Total purchases
    pub total_purchases: u64,

    /// Total refunds
    pub total_refunds: u64,

    /// Gross revenue (USD cents)
    pub gross_revenue_cents: i64,

    /// Refund amount (USD cents)
    pub refund_amount_cents: i64,

    /// Net revenue (USD cents)
    pub net_revenue_cents: i64,
}

impl RevenueStatistics {
    /// Get refund rate
    pub fn refund_rate(&self) -> f32 {
        if self.total_purchases == 0 {
            0.0
        } else {
            self.total_refunds as f32 / self.total_purchases as f32
        }
    }

    /// Get average purchase amount
    pub fn avg_purchase_cents(&self) -> f32 {
        if self.total_purchases == 0 {
            0.0
        } else {
            self.gross_revenue_cents as f32 / self.total_purchases as f32
        }
    }
}

/// Developer dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperDashboard {
    /// Plugin ID
    pub plugin_id: Uuid,

    /// Total downloads
    pub total_downloads: u64,

    /// Active installations
    pub active_installs: u64,

    /// Daily active users (last 24h)
    pub daily_active_users: u64,

    /// Monthly active users (last 30d)
    pub monthly_active_users: u64,

    /// Average rating
    pub average_rating: f32,

    /// Total reviews
    pub total_reviews: u32,

    /// Revenue statistics
    pub revenue_stats: RevenueStatistics,

    /// Download trend (last 30 days)
    pub download_trend: Vec<DailyMetric>,

    /// Active install trend (last 30 days)
    pub install_trend: Vec<DailyMetric>,

    /// Revenue trend (last 30 days)
    pub revenue_trend: Vec<DailyMetric>,

    /// Top countries by downloads
    pub top_countries: Vec<(String, u64)>,

    /// Platform distribution
    pub platform_distribution: HashMap<String, u64>,

    /// Version distribution
    pub version_distribution: HashMap<String, u64>,

    /// Support metrics
    pub support_metrics: SupportMetrics,

    /// Last updated
    pub last_updated: DateTime<Utc>,
}

impl DeveloperDashboard {
    /// Create a new dashboard
    pub fn new(plugin_id: Uuid) -> Self {
        Self {
            plugin_id,
            total_downloads: 0,
            active_installs: 0,
            daily_active_users: 0,
            monthly_active_users: 0,
            average_rating: 0.0,
            total_reviews: 0,
            revenue_stats: RevenueStatistics::default(),
            download_trend: Vec::new(),
            install_trend: Vec::new(),
            revenue_trend: Vec::new(),
            top_countries: Vec::new(),
            platform_distribution: HashMap::new(),
            version_distribution: HashMap::new(),
            support_metrics: SupportMetrics::default(),
            last_updated: Utc::now(),
        }
    }

    /// Generate dashboard from analytics and revenue data
    pub fn generate(
        plugin_id: Uuid,
        analytics: &AnalyticsTracker,
        revenue: &RevenueTracker,
    ) -> Self {
        let mut dashboard = Self::new(plugin_id);

        // Get events for the last 30 days
        let now = Utc::now();
        let thirty_days_ago = now - Duration::days(30);
        let events = analytics.get_plugin_events_range(plugin_id, thirty_days_ago, now);

        // Count downloads
        dashboard.total_downloads = events.iter()
            .filter(|e| matches!(e.event, AnalyticsEvent::Download { .. }))
            .count() as u64;

        // Calculate active installs
        let installs = events.iter()
            .filter(|e| matches!(e.event, AnalyticsEvent::Install { .. }))
            .count() as u64;

        let uninstalls = events.iter()
            .filter(|e| matches!(e.event, AnalyticsEvent::Uninstall { .. }))
            .count() as u64;

        dashboard.active_installs = installs.saturating_sub(uninstalls);

        // Get revenue statistics
        dashboard.revenue_stats = revenue.get_statistics(plugin_id);

        dashboard.last_updated = Utc::now();

        dashboard
    }
}

/// Daily metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyMetric {
    /// Date
    pub date: DateTime<Utc>,

    /// Value
    pub value: u64,
}

/// Support metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SupportMetrics {
    /// Total support requests
    pub total_requests: u64,

    /// Open requests
    pub open_requests: u64,

    /// Average response time (hours)
    pub avg_response_time_hours: f32,

    /// Average resolution time (hours)
    pub avg_resolution_time_hours: f32,

    /// Customer satisfaction score (0-100)
    pub satisfaction_score: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_tracking() -> Result<()> {
        let tracker = AnalyticsTracker::new();

        let plugin_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let event = AnalyticsEvent::Download {
            plugin_id,
            user_id,
            version: "1.0.0".to_string(),
        };

        tracker.track(event)?;

        let events = tracker.get_plugin_events(plugin_id);
        assert_eq!(events.len(), 1);

        let counts = tracker.get_event_counts(plugin_id);
        assert_eq!(counts.get("download"), Some(&1));

        Ok(())
    }

    #[test]
    fn test_revenue_tracking() -> Result<()> {
        let tracker = RevenueTracker::new();

        let plugin_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        tracker.record_purchase(plugin_id, user_id, 1999, "USD".to_string())?;
        tracker.record_purchase(plugin_id, user_id, 2999, "USD".to_string())?;
        tracker.record_refund(plugin_id, user_id, 1999, "USD".to_string(), "Changed mind".to_string())?;

        let total = tracker.get_total_revenue(plugin_id);
        assert_eq!(total, 3000); // 1999 + 2999 - 1999 = 3000

        let stats = tracker.get_statistics(plugin_id);
        assert_eq!(stats.total_purchases, 2);
        assert_eq!(stats.total_refunds, 1);
        assert_eq!(stats.net_revenue_cents, 3000);

        Ok(())
    }

    #[test]
    fn test_revenue_statistics() {
        let mut stats = RevenueStatistics::default();
        stats.total_purchases = 100;
        stats.total_refunds = 5;
        stats.gross_revenue_cents = 10000;
        stats.refund_amount_cents = 500;

        assert_eq!(stats.refund_rate(), 0.05);
        assert_eq!(stats.avg_purchase_cents(), 100.0);
    }

    #[test]
    fn test_developer_dashboard() {
        let plugin_id = Uuid::new_v4();
        let analytics = AnalyticsTracker::new();
        let revenue = RevenueTracker::new();

        let dashboard = DeveloperDashboard::generate(plugin_id, &analytics, &revenue);

        assert_eq!(dashboard.plugin_id, plugin_id);
        assert_eq!(dashboard.total_downloads, 0);
    }
}
