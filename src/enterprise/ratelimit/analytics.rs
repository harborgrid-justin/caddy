//! Rate Limit Analytics
//!
//! This module provides comprehensive analytics for rate limiting:
//! - Rate limit hit tracking
//! - Quota usage dashboards
//! - Abuse detection
//! - Anomaly alerting

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::algorithm::Decision;
use super::quota::QuotaIdentifier;

// ============================================================================
// Event Types
// ============================================================================

/// Rate limit event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    /// Request allowed
    Allowed,
    /// Request denied (rate limited)
    Denied,
    /// Request queued
    Queued,
    /// Request delayed
    Delayed,
    /// Service degraded
    Degraded,
}

/// Rate limit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitEvent {
    /// Event ID
    pub id: String,
    /// Event type
    pub event_type: EventType,
    /// Timestamp
    pub timestamp: SystemTime,
    /// Identifier (user, API key, etc.)
    pub identifier: QuotaIdentifier,
    /// Operation/endpoint
    pub operation: String,
    /// Limit that was checked
    pub limit: u64,
    /// Remaining quota at time of event
    pub remaining: u64,
    /// Retry after duration (if denied)
    pub retry_after: Option<u64>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl RateLimitEvent {
    /// Create a new event
    pub fn new(
        event_type: EventType,
        identifier: QuotaIdentifier,
        operation: String,
        decision: &Decision,
    ) -> Self {
        let (limit, remaining, retry_after) = match decision {
            Decision::Allowed { remaining, .. } => (remaining + 1, *remaining, None),
            Decision::Denied { retry_after, limit } => (*limit, 0, Some(*retry_after)),
        };

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            timestamp: SystemTime::now(),
            identifier,
            operation,
            limit,
            remaining,
            retry_after,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Get age of event
    pub fn age(&self) -> Duration {
        SystemTime::now()
            .duration_since(self.timestamp)
            .unwrap_or(Duration::ZERO)
    }
}

// ============================================================================
// Analytics Collector
// ============================================================================

/// Analytics collector for rate limit events
pub struct RateLimitAnalytics {
    /// Event storage
    events: Arc<RwLock<VecDeque<RateLimitEvent>>>,
    /// Maximum events to store
    max_events: usize,
    /// Aggregated statistics
    stats: Arc<DashMap<String, Statistics>>,
    /// Event listeners
    listeners: Arc<RwLock<Vec<Box<dyn EventListener>>>>,
    /// Retention period
    retention: Duration,
}

impl RateLimitAnalytics {
    /// Create a new analytics collector
    pub fn new(max_events: usize, retention: Duration) -> Self {
        Self {
            events: Arc::new(RwLock::new(VecDeque::with_capacity(max_events))),
            max_events,
            stats: Arc::new(DashMap::new()),
            listeners: Arc::new(RwLock::new(Vec::new())),
            retention,
        }
    }

    /// Record an event
    pub async fn record(&self, event: RateLimitEvent) {
        // Update statistics
        self.update_stats(&event).await;

        // Notify listeners
        self.notify_listeners(&event).await;

        // Store event
        let mut events = self.events.write().await;

        // Remove old events
        while events.len() >= self.max_events {
            events.pop_front();
        }

        events.push_back(event);
    }

    /// Update statistics
    async fn update_stats(&self, event: &RateLimitEvent) {
        let key = format!("{}:{}", event.identifier.to_key(), event.operation);

        self.stats
            .entry(key)
            .and_modify(|stats| stats.update(event))
            .or_insert_with(|| Statistics::from_event(event));
    }

    /// Notify event listeners
    async fn notify_listeners(&self, event: &RateLimitEvent) {
        let listeners = self.listeners.read().await;
        for listener in listeners.iter() {
            listener.on_event(event).await;
        }
    }

    /// Add an event listener
    pub async fn add_listener(&self, listener: Box<dyn EventListener>) {
        self.listeners.write().await.push(listener);
    }

    /// Get events for an identifier
    pub async fn get_events(
        &self,
        identifier: &QuotaIdentifier,
        operation: Option<&str>,
        limit: Option<usize>,
    ) -> Vec<RateLimitEvent> {
        let events = self.events.read().await;
        let key = identifier.to_key();

        events
            .iter()
            .filter(|e| {
                e.identifier.to_key() == key
                    && operation.map_or(true, |op| e.operation == op)
                    && e.age() <= self.retention
            })
            .rev()
            .take(limit.unwrap_or(usize::MAX))
            .cloned()
            .collect()
    }

    /// Get statistics for an identifier
    pub async fn get_statistics(
        &self,
        identifier: &QuotaIdentifier,
        operation: &str,
    ) -> Option<Statistics> {
        let key = format!("{}:{}", identifier.to_key(), operation);
        self.stats.get(&key).map(|s| s.clone())
    }

    /// Get all statistics
    pub async fn get_all_statistics(&self) -> HashMap<String, Statistics> {
        self.stats
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    /// Clear old events
    pub async fn cleanup(&self) {
        let cutoff = SystemTime::now() - self.retention;

        let mut events = self.events.write().await;
        events.retain(|e| e.timestamp >= cutoff);
    }

    /// Get event count
    pub async fn event_count(&self) -> usize {
        self.events.read().await.len()
    }
}

// ============================================================================
// Statistics
// ============================================================================

/// Aggregated statistics for rate limiting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    /// Total requests
    pub total_requests: u64,
    /// Allowed requests
    pub allowed: u64,
    /// Denied requests
    pub denied: u64,
    /// Queued requests
    pub queued: u64,
    /// Delayed requests
    pub delayed: u64,
    /// Degraded requests
    pub degraded: u64,
    /// First event timestamp
    pub first_seen: SystemTime,
    /// Last event timestamp
    pub last_seen: SystemTime,
    /// Average remaining quota
    pub avg_remaining: f64,
    /// Minimum remaining quota seen
    pub min_remaining: u64,
    /// Maximum remaining quota seen
    pub max_remaining: u64,
}

impl Statistics {
    /// Create from an event
    fn from_event(event: &RateLimitEvent) -> Self {
        let mut stats = Self {
            total_requests: 1,
            allowed: 0,
            denied: 0,
            queued: 0,
            delayed: 0,
            degraded: 0,
            first_seen: event.timestamp,
            last_seen: event.timestamp,
            avg_remaining: event.remaining as f64,
            min_remaining: event.remaining,
            max_remaining: event.remaining,
        };

        match event.event_type {
            EventType::Allowed => stats.allowed = 1,
            EventType::Denied => stats.denied = 1,
            EventType::Queued => stats.queued = 1,
            EventType::Delayed => stats.delayed = 1,
            EventType::Degraded => stats.degraded = 1,
        }

        stats
    }

    /// Update with new event
    fn update(&mut self, event: &RateLimitEvent) {
        self.total_requests += 1;

        match event.event_type {
            EventType::Allowed => self.allowed += 1,
            EventType::Denied => self.denied += 1,
            EventType::Queued => self.queued += 1,
            EventType::Delayed => self.delayed += 1,
            EventType::Degraded => self.degraded += 1,
        }

        self.last_seen = event.timestamp;

        // Update remaining quota stats
        self.avg_remaining = (self.avg_remaining * (self.total_requests - 1) as f64
            + event.remaining as f64)
            / self.total_requests as f64;

        self.min_remaining = self.min_remaining.min(event.remaining);
        self.max_remaining = self.max_remaining.max(event.remaining);
    }

    /// Calculate denial rate
    pub fn denial_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.denied as f64 / self.total_requests as f64
        }
    }

    /// Check if showing signs of abuse
    pub fn is_abusive(&self, threshold: f64) -> bool {
        self.denial_rate() >= threshold
    }
}

// ============================================================================
// Event Listener Trait
// ============================================================================

/// Event listener trait for receiving rate limit events
#[async_trait]
pub trait EventListener: Send + Sync {
    /// Called when a rate limit event occurs
    async fn on_event(&self, event: &RateLimitEvent);
}

// ============================================================================
// Abuse Detection
// ============================================================================

/// Abuse detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbuseDetectionConfig {
    /// Denial rate threshold (0.0 to 1.0)
    pub denial_threshold: f64,
    /// Minimum requests before flagging
    pub min_requests: u64,
    /// Time window for analysis
    pub window: Duration,
    /// Enable automatic blocking
    pub auto_block: bool,
    /// Block duration
    pub block_duration: Duration,
}

impl Default for AbuseDetectionConfig {
    fn default() -> Self {
        Self {
            denial_threshold: 0.8,
            min_requests: 100,
            window: Duration::from_secs(3600),
            auto_block: false,
            block_duration: Duration::from_secs(3600),
        }
    }
}

/// Abuse detector
pub struct AbuseDetector {
    /// Configuration
    config: AbuseDetectionConfig,
    /// Flagged identifiers
    flagged: Arc<DashMap<String, AbuseReport>>,
    /// Blocked identifiers
    blocked: Arc<DashMap<String, SystemTime>>,
}

impl AbuseDetector {
    /// Create a new abuse detector
    pub fn new(config: AbuseDetectionConfig) -> Self {
        Self {
            config,
            flagged: Arc::new(DashMap::new()),
            blocked: Arc::new(DashMap::new()),
        }
    }

    /// Analyze statistics for abuse
    pub async fn analyze(&self, identifier: &QuotaIdentifier, stats: &Statistics) -> Option<AbuseReport> {
        if stats.total_requests < self.config.min_requests {
            return None;
        }

        if stats.is_abusive(self.config.denial_threshold) {
            let report = AbuseReport {
                identifier: identifier.clone(),
                denial_rate: stats.denial_rate(),
                total_requests: stats.total_requests,
                denied_requests: stats.denied,
                first_seen: stats.first_seen,
                last_seen: stats.last_seen,
                flagged_at: SystemTime::now(),
                severity: self.calculate_severity(stats),
            };

            // Store report
            self.flagged.insert(identifier.to_key(), report.clone());

            // Auto-block if enabled
            if self.config.auto_block {
                self.block(identifier).await;
            }

            Some(report)
        } else {
            None
        }
    }

    /// Calculate abuse severity
    fn calculate_severity(&self, stats: &Statistics) -> AbuseSeverity {
        let rate = stats.denial_rate();

        if rate >= 0.95 {
            AbuseSeverity::Critical
        } else if rate >= 0.9 {
            AbuseSeverity::High
        } else if rate >= 0.85 {
            AbuseSeverity::Medium
        } else {
            AbuseSeverity::Low
        }
    }

    /// Block an identifier
    pub async fn block(&self, identifier: &QuotaIdentifier) {
        let until = SystemTime::now() + self.config.block_duration;
        self.blocked.insert(identifier.to_key(), until);
    }

    /// Unblock an identifier
    pub async fn unblock(&self, identifier: &QuotaIdentifier) {
        self.blocked.remove(&identifier.to_key());
    }

    /// Check if identifier is blocked
    pub async fn is_blocked(&self, identifier: &QuotaIdentifier) -> bool {
        if let Some(entry) = self.blocked.get(&identifier.to_key()) {
            if *entry.value() > SystemTime::now() {
                return true;
            }
            // Block expired, remove it
            drop(entry);
            self.blocked.remove(&identifier.to_key());
        }
        false
    }

    /// Get all flagged identifiers
    pub async fn get_flagged(&self) -> Vec<AbuseReport> {
        self.flagged.iter().map(|e| e.value().clone()).collect()
    }

    /// Get all blocked identifiers
    pub async fn get_blocked(&self) -> Vec<(String, SystemTime)> {
        self.blocked
            .iter()
            .map(|e| (e.key().clone(), *e.value()))
            .collect()
    }

    /// Cleanup expired blocks
    pub async fn cleanup(&self) {
        let now = SystemTime::now();
        self.blocked.retain(|_, &mut until| until > now);
    }
}

/// Abuse report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbuseReport {
    /// Identifier
    pub identifier: QuotaIdentifier,
    /// Denial rate
    pub denial_rate: f64,
    /// Total requests
    pub total_requests: u64,
    /// Denied requests
    pub denied_requests: u64,
    /// First seen timestamp
    pub first_seen: SystemTime,
    /// Last seen timestamp
    pub last_seen: SystemTime,
    /// When flagged
    pub flagged_at: SystemTime,
    /// Severity level
    pub severity: AbuseSeverity,
}

/// Abuse severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AbuseSeverity {
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

// ============================================================================
// Anomaly Detection
// ============================================================================

/// Anomaly detection using statistical methods
pub struct AnomalyDetector {
    /// Historical data window
    window: Duration,
    /// Historical request rates
    history: Arc<RwLock<VecDeque<(SystemTime, f64)>>>,
    /// Standard deviation threshold
    std_dev_threshold: f64,
}

impl AnomalyDetector {
    /// Create a new anomaly detector
    pub fn new(window: Duration, std_dev_threshold: f64) -> Self {
        Self {
            window,
            history: Arc::new(RwLock::new(VecDeque::new())),
            std_dev_threshold,
        }
    }

    /// Record a data point
    pub async fn record(&self, rate: f64) {
        let now = SystemTime::now();
        let mut history = self.history.write().await;

        history.push_back((now, rate));

        // Remove old data
        let cutoff = now - self.window;
        history.retain(|(ts, _)| *ts >= cutoff);
    }

    /// Detect if current rate is anomalous
    pub async fn detect(&self, current_rate: f64) -> bool {
        let history = self.history.read().await;

        if history.len() < 10 {
            // Not enough data
            return false;
        }

        let rates: Vec<f64> = history.iter().map(|(_, r)| *r).collect();
        let mean = rates.iter().sum::<f64>() / rates.len() as f64;

        let variance = rates.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / rates.len() as f64;

        let std_dev = variance.sqrt();

        // Anomaly if more than N standard deviations from mean
        (current_rate - mean).abs() > (std_dev * self.std_dev_threshold)
    }

    /// Get statistics
    pub async fn get_statistics(&self) -> Option<AnomalyStatistics> {
        let history = self.history.read().await;

        if history.is_empty() {
            return None;
        }

        let rates: Vec<f64> = history.iter().map(|(_, r)| *r).collect();
        let mean = rates.iter().sum::<f64>() / rates.len() as f64;

        let variance = rates.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / rates.len() as f64;

        let std_dev = variance.sqrt();
        let min = rates.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = rates.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        Some(AnomalyStatistics {
            mean,
            std_dev,
            min,
            max,
            samples: rates.len(),
        })
    }
}

/// Anomaly statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyStatistics {
    /// Mean rate
    pub mean: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// Minimum rate
    pub min: f64,
    /// Maximum rate
    pub max: f64,
    /// Number of samples
    pub samples: usize,
}

// ============================================================================
// Alert Manager
// ============================================================================

/// Alert severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Informational
    Info,
    /// Warning
    Warning,
    /// Error
    Error,
    /// Critical
    Critical,
}

/// Alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Alert ID
    pub id: String,
    /// Severity
    pub severity: AlertSeverity,
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// Timestamp
    pub timestamp: SystemTime,
    /// Related identifier
    pub identifier: Option<QuotaIdentifier>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

impl Alert {
    /// Create a new alert
    pub fn new(severity: AlertSeverity, title: String, description: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            severity,
            title,
            description,
            timestamp: SystemTime::now(),
            identifier: None,
            metadata: HashMap::new(),
        }
    }

    /// Set identifier
    pub fn with_identifier(mut self, identifier: QuotaIdentifier) -> Self {
        self.identifier = Some(identifier);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Alert manager
#[async_trait]
pub trait AlertManager: Send + Sync {
    /// Send an alert
    async fn send_alert(&self, alert: Alert);
}

/// Console alert manager (logs to stdout)
pub struct ConsoleAlertManager;

#[async_trait]
impl AlertManager for ConsoleAlertManager {
    async fn send_alert(&self, alert: Alert) {
        let severity_str = match alert.severity {
            AlertSeverity::Info => "INFO",
            AlertSeverity::Warning => "WARN",
            AlertSeverity::Error => "ERROR",
            AlertSeverity::Critical => "CRITICAL",
        };

        println!(
            "[{}] {} - {} - {}",
            severity_str,
            alert.id,
            alert.title,
            alert.description
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_event() {
        let decision = Decision::Allowed {
            remaining: 50,
            reset_after: 60,
        };

        let event = RateLimitEvent::new(
            EventType::Allowed,
            QuotaIdentifier::User("user123".to_string()),
            "api_call".to_string(),
            &decision,
        );

        assert_eq!(event.event_type, EventType::Allowed);
        assert_eq!(event.remaining, 50);
        assert!(event.retry_after.is_none());
    }

    #[tokio::test]
    async fn test_analytics_recording() {
        let analytics = RateLimitAnalytics::new(100, Duration::from_secs(3600));

        let decision = Decision::Allowed {
            remaining: 50,
            reset_after: 60,
        };

        let event = RateLimitEvent::new(
            EventType::Allowed,
            QuotaIdentifier::User("user123".to_string()),
            "api_call".to_string(),
            &decision,
        );

        analytics.record(event).await;

        assert_eq!(analytics.event_count().await, 1);
    }

    #[tokio::test]
    async fn test_statistics() {
        let mut stats = Statistics::from_event(&RateLimitEvent::new(
            EventType::Allowed,
            QuotaIdentifier::User("user1".to_string()),
            "test".to_string(),
            &Decision::Allowed {
                remaining: 100,
                reset_after: 60,
            },
        ));

        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.allowed, 1);
        assert_eq!(stats.denied, 0);

        let denied_event = RateLimitEvent::new(
            EventType::Denied,
            QuotaIdentifier::User("user1".to_string()),
            "test".to_string(),
            &Decision::Denied {
                retry_after: 30,
                limit: 100,
            },
        );

        stats.update(&denied_event);

        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.denied, 1);
        assert_eq!(stats.denial_rate(), 0.5);
    }

    #[tokio::test]
    async fn test_abuse_detection() {
        let config = AbuseDetectionConfig {
            denial_threshold: 0.8,
            min_requests: 10,
            ..Default::default()
        };

        let detector = AbuseDetector::new(config);

        let stats = Statistics {
            total_requests: 100,
            allowed: 10,
            denied: 90,
            queued: 0,
            delayed: 0,
            degraded: 0,
            first_seen: SystemTime::now(),
            last_seen: SystemTime::now(),
            avg_remaining: 5.0,
            min_remaining: 0,
            max_remaining: 10,
        };

        let user_id = QuotaIdentifier::User("abuser".to_string());
        let report = detector.analyze(&user_id, &stats).await;

        assert!(report.is_some());
        let report = report.unwrap();
        assert_eq!(report.denial_rate, 0.9);
        assert!(matches!(report.severity, AbuseSeverity::High | AbuseSeverity::Critical));
    }

    #[tokio::test]
    async fn test_anomaly_detection() {
        let detector = AnomalyDetector::new(Duration::from_secs(3600), 3.0);

        // Record normal data
        for _ in 0..20 {
            detector.record(100.0).await;
        }

        // Normal rate should not be anomalous
        assert!(!detector.detect(100.0).await);

        // Very high rate should be anomalous
        assert!(detector.detect(500.0).await);
    }

    #[tokio::test]
    async fn test_blocking() {
        let config = AbuseDetectionConfig {
            block_duration: Duration::from_secs(1),
            ..Default::default()
        };

        let detector = AbuseDetector::new(config);
        let user_id = QuotaIdentifier::User("test_user".to_string());

        detector.block(&user_id).await;
        assert!(detector.is_blocked(&user_id).await);

        // Wait for block to expire
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert!(!detector.is_blocked(&user_id).await);
    }

    #[test]
    fn test_alert_creation() {
        let alert = Alert::new(
            AlertSeverity::Warning,
            "High rate limit usage".to_string(),
            "User approaching rate limit".to_string(),
        )
        .with_identifier(QuotaIdentifier::User("user123".to_string()))
        .with_metadata("usage".to_string(), "85%".to_string());

        assert_eq!(alert.severity, AlertSeverity::Warning);
        assert!(alert.identifier.is_some());
        assert_eq!(alert.metadata.get("usage"), Some(&"85%".to_string()));
    }
}
