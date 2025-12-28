//! Compliance alert rules and anomaly detection
//!
//! This module implements compliance violation detection, anomaly alerting,
//! escalation policies, and notification channels for compliance monitoring.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Alert severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Informational alert
    Info,
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

/// Alert category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertCategory {
    /// Security-related alert
    Security,
    /// Compliance violation
    Compliance,
    /// Access control issue
    AccessControl,
    /// Data integrity issue
    DataIntegrity,
    /// Anomalous behavior detected
    Anomaly,
    /// Policy violation
    PolicyViolation,
    /// Retention policy issue
    Retention,
    /// Audit trail issue
    AuditTrail,
}

/// Alert status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertStatus {
    /// New alert
    New,
    /// Alert acknowledged
    Acknowledged,
    /// Under investigation
    Investigating,
    /// Resolved
    Resolved,
    /// False positive
    FalsePositive,
    /// Suppressed
    Suppressed,
}

/// Compliance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAlert {
    /// Unique alert ID
    pub id: Uuid,

    /// Alert title
    pub title: String,

    /// Alert description
    pub description: String,

    /// Alert category
    pub category: AlertCategory,

    /// Alert severity
    pub severity: AlertSeverity,

    /// Triggered timestamp
    pub triggered_at: DateTime<Utc>,

    /// Rule that triggered this alert
    pub rule_id: Option<Uuid>,

    /// Alert status
    pub status: AlertStatus,

    /// Acknowledged by
    pub acknowledged_by: Option<String>,

    /// Acknowledged timestamp
    pub acknowledged_at: Option<DateTime<Utc>>,

    /// Resolved by
    pub resolved_by: Option<String>,

    /// Resolved timestamp
    pub resolved_at: Option<DateTime<Utc>>,

    /// Resolution notes
    pub resolution_notes: Option<String>,

    /// Related entity ID
    pub entity_id: Option<String>,

    /// Related entity type
    pub entity_type: Option<String>,

    /// Alert metadata
    pub metadata: HashMap<String, String>,

    /// Notification channels used
    pub notified_channels: Vec<String>,

    /// Escalation level
    pub escalation_level: u32,

    /// Last escalation time
    pub last_escalated: Option<DateTime<Utc>>,
}

impl ComplianceAlert {
    /// Create a new alert
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
        category: AlertCategory,
        severity: AlertSeverity,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            description: description.into(),
            category,
            severity,
            triggered_at: Utc::now(),
            rule_id: None,
            status: AlertStatus::New,
            acknowledged_by: None,
            acknowledged_at: None,
            resolved_by: None,
            resolved_at: None,
            resolution_notes: None,
            entity_id: None,
            entity_type: None,
            metadata: HashMap::new(),
            notified_channels: Vec::new(),
            escalation_level: 0,
            last_escalated: None,
        }
    }

    /// Acknowledge the alert
    pub fn acknowledge(&mut self, user: impl Into<String>) {
        self.status = AlertStatus::Acknowledged;
        self.acknowledged_by = Some(user.into());
        self.acknowledged_at = Some(Utc::now());
    }

    /// Resolve the alert
    pub fn resolve(&mut self, user: impl Into<String>, notes: Option<String>) {
        self.status = AlertStatus::Resolved;
        self.resolved_by = Some(user.into());
        self.resolved_at = Some(Utc::now());
        self.resolution_notes = notes;
    }

    /// Mark as false positive
    pub fn mark_false_positive(&mut self, user: impl Into<String>) {
        self.status = AlertStatus::FalsePositive;
        self.resolved_by = Some(user.into());
        self.resolved_at = Some(Utc::now());
    }

    /// Escalate the alert
    pub fn escalate(&mut self) {
        self.escalation_level += 1;
        self.last_escalated = Some(Utc::now());
    }
}

/// Alert rule condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    /// Threshold exceeded
    Threshold {
        metric: String,
        operator: ComparisonOperator,
        value: f64,
    },
    /// Pattern match
    Pattern {
        field: String,
        pattern: String,
    },
    /// Time-based
    TimeWindow {
        count: u32,
        window: Duration,
    },
    /// Anomaly detection
    Anomaly {
        metric: String,
        sensitivity: f64,
    },
    /// Custom condition
    Custom(String),
}

/// Comparison operator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOperator {
    /// Greater than
    GreaterThan,
    /// Greater than or equal
    GreaterThanOrEqual,
    /// Less than
    LessThan,
    /// Less than or equal
    LessThanOrEqual,
    /// Equal
    Equal,
    /// Not equal
    NotEqual,
}

/// Alert rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// Unique rule ID
    pub id: Uuid,

    /// Rule name
    pub name: String,

    /// Rule description
    pub description: String,

    /// Alert category
    pub category: AlertCategory,

    /// Alert severity
    pub severity: AlertSeverity,

    /// Rule conditions
    pub conditions: Vec<RuleCondition>,

    /// Whether rule is enabled
    pub enabled: bool,

    /// Notification channels
    pub notification_channels: Vec<String>,

    /// Cooldown period (prevent alert spam)
    pub cooldown: Option<Duration>,

    /// Last triggered time
    pub last_triggered: Option<DateTime<Utc>>,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Created by
    pub created_by: String,
}

impl AlertRule {
    /// Check if rule is in cooldown period
    pub fn is_in_cooldown(&self) -> bool {
        if let (Some(cooldown), Some(last_triggered)) = (self.cooldown, self.last_triggered) {
            Utc::now() - last_triggered < cooldown
        } else {
            false
        }
    }
}

/// Notification channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    /// Channel ID
    pub id: String,

    /// Channel name
    pub name: String,

    /// Channel type
    pub channel_type: ChannelType,

    /// Channel configuration
    pub config: HashMap<String, String>,

    /// Whether channel is enabled
    pub enabled: bool,

    /// Minimum severity to notify
    pub min_severity: AlertSeverity,
}

/// Channel type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelType {
    /// Email notification
    Email,
    /// SMS notification
    Sms,
    /// Slack notification
    Slack,
    /// Webhook
    Webhook,
    /// PagerDuty
    PagerDuty,
    /// Microsoft Teams
    Teams,
    /// Custom channel
    Custom,
}

/// Escalation policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationPolicy {
    /// Policy ID
    pub id: Uuid,

    /// Policy name
    pub name: String,

    /// Escalation levels
    pub levels: Vec<EscalationLevel>,

    /// Whether policy is enabled
    pub enabled: bool,
}

/// Escalation level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationLevel {
    /// Level number
    pub level: u32,

    /// Time to wait before escalation
    pub delay: Duration,

    /// Notification channels for this level
    pub channels: Vec<String>,

    /// Recipients
    pub recipients: Vec<String>,
}

/// Anomaly detector for baseline-based alerting
pub struct AnomalyDetector {
    /// Metric baselines
    baselines: HashMap<String, MetricBaseline>,

    /// Recent data points
    recent_data: HashMap<String, VecDeque<DataPoint>>,

    /// Lookback window size
    window_size: usize,
}

/// Metric baseline statistics
#[derive(Debug, Clone)]
struct MetricBaseline {
    mean: f64,
    std_dev: f64,
    min: f64,
    max: f64,
    sample_count: usize,
}

/// Data point for anomaly detection
#[derive(Debug, Clone)]
struct DataPoint {
    timestamp: DateTime<Utc>,
    value: f64,
}

impl AnomalyDetector {
    /// Create new anomaly detector
    pub fn new(window_size: usize) -> Self {
        Self {
            baselines: HashMap::new(),
            recent_data: HashMap::new(),
            window_size,
        }
    }

    /// Record a data point
    pub fn record(&mut self, metric: impl Into<String>, value: f64) {
        let metric = metric.into();
        let data_point = DataPoint {
            timestamp: Utc::now(),
            value,
        };

        let recent = self.recent_data.entry(metric.clone()).or_insert_with(VecDeque::new);
        recent.push_back(data_point);

        // Keep only recent window
        while recent.len() > self.window_size {
            recent.pop_front();
        }

        // Update baseline
        self.update_baseline(&metric);
    }

    /// Update baseline statistics
    fn update_baseline(&mut self, metric: &str) {
        if let Some(recent) = self.recent_data.get(metric) {
            if recent.is_empty() {
                return;
            }

            let values: Vec<f64> = recent.iter().map(|dp| dp.value).collect();
            let count = values.len();
            let sum: f64 = values.iter().sum();
            let mean = sum / count as f64;

            let variance: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / count as f64;
            let std_dev = variance.sqrt();

            let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

            self.baselines.insert(
                metric.to_string(),
                MetricBaseline {
                    mean,
                    std_dev,
                    min,
                    max,
                    sample_count: count,
                },
            );
        }
    }

    /// Check if value is anomalous
    pub fn is_anomaly(&self, metric: &str, value: f64, sensitivity: f64) -> bool {
        if let Some(baseline) = self.baselines.get(metric) {
            // Use standard deviation method: value is anomalous if it's more than
            // sensitivity * std_dev away from mean
            let threshold = sensitivity * baseline.std_dev;
            (value - baseline.mean).abs() > threshold
        } else {
            // No baseline yet, not anomalous
            false
        }
    }

    /// Get baseline for metric
    pub fn get_baseline(&self, metric: &str) -> Option<(f64, f64)> {
        self.baselines.get(metric).map(|b| (b.mean, b.std_dev))
    }
}

/// Alert manager
pub struct AlertManager {
    /// Active alerts
    alerts: Arc<RwLock<HashMap<Uuid, ComplianceAlert>>>,

    /// Alert rules
    rules: Arc<RwLock<HashMap<Uuid, AlertRule>>>,

    /// Notification channels
    channels: Arc<RwLock<HashMap<String, NotificationChannel>>>,

    /// Escalation policies
    escalation_policies: Arc<RwLock<HashMap<Uuid, EscalationPolicy>>>,

    /// Anomaly detector
    anomaly_detector: Arc<RwLock<AnomalyDetector>>,
}

impl AlertManager {
    /// Create new alert manager
    pub fn new() -> Self {
        Self {
            alerts: Arc::new(RwLock::new(HashMap::new())),
            rules: Arc::new(RwLock::new(HashMap::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
            escalation_policies: Arc::new(RwLock::new(HashMap::new())),
            anomaly_detector: Arc::new(RwLock::new(AnomalyDetector::new(100))),
        }
    }

    // ========================================================================
    // Alert Management
    // ========================================================================

    /// Trigger an alert
    pub async fn trigger_alert(&self, mut alert: ComplianceAlert) -> Result<Uuid, String> {
        let alert_id = alert.id;

        // Notify configured channels
        if let Some(rule_id) = alert.rule_id {
            let rules = self.rules.read().await;
            if let Some(rule) = rules.get(&rule_id) {
                alert.notified_channels = rule.notification_channels.clone();
                // In production, actually send notifications here
            }
        }

        let mut alerts = self.alerts.write().await;
        alerts.insert(alert_id, alert);

        Ok(alert_id)
    }

    /// Get alert by ID
    pub async fn get_alert(&self, id: Uuid) -> Option<ComplianceAlert> {
        let alerts = self.alerts.read().await;
        alerts.get(&id).cloned()
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<ComplianceAlert> {
        let alerts = self.alerts.read().await;
        alerts
            .values()
            .filter(|a| !matches!(a.status, AlertStatus::Resolved | AlertStatus::FalsePositive))
            .cloned()
            .collect()
    }

    /// Get alerts by severity
    pub async fn get_alerts_by_severity(&self, severity: AlertSeverity) -> Vec<ComplianceAlert> {
        let alerts = self.alerts.read().await;
        alerts
            .values()
            .filter(|a| a.severity == severity)
            .cloned()
            .collect()
    }

    /// Acknowledge alert
    pub async fn acknowledge_alert(&self, alert_id: Uuid, user: impl Into<String>) -> Result<(), String> {
        let mut alerts = self.alerts.write().await;
        if let Some(alert) = alerts.get_mut(&alert_id) {
            alert.acknowledge(user);
            Ok(())
        } else {
            Err("Alert not found".to_string())
        }
    }

    /// Resolve alert
    pub async fn resolve_alert(
        &self,
        alert_id: Uuid,
        user: impl Into<String>,
        notes: Option<String>,
    ) -> Result<(), String> {
        let mut alerts = self.alerts.write().await;
        if let Some(alert) = alerts.get_mut(&alert_id) {
            alert.resolve(user, notes);
            Ok(())
        } else {
            Err("Alert not found".to_string())
        }
    }

    // ========================================================================
    // Rule Management
    // ========================================================================

    /// Add alert rule
    pub async fn add_rule(&self, rule: AlertRule) -> Result<Uuid, String> {
        let rule_id = rule.id;
        let mut rules = self.rules.write().await;
        rules.insert(rule_id, rule);
        Ok(rule_id)
    }

    /// Update rule
    pub async fn update_rule(&self, id: Uuid, mut rule: AlertRule) -> Result<(), String> {
        rule.updated_at = Utc::now();
        let mut rules = self.rules.write().await;
        rules.insert(id, rule);
        Ok(())
    }

    /// Enable/disable rule
    pub async fn set_rule_enabled(&self, rule_id: Uuid, enabled: bool) -> Result<(), String> {
        let mut rules = self.rules.write().await;
        if let Some(rule) = rules.get_mut(&rule_id) {
            rule.enabled = enabled;
            Ok(())
        } else {
            Err("Rule not found".to_string())
        }
    }

    /// Get enabled rules
    pub async fn get_enabled_rules(&self) -> Vec<AlertRule> {
        let rules = self.rules.read().await;
        rules.values().filter(|r| r.enabled).cloned().collect()
    }

    // ========================================================================
    // Notification Channels
    // ========================================================================

    /// Add notification channel
    pub async fn add_channel(&self, channel: NotificationChannel) -> Result<String, String> {
        let channel_id = channel.id.clone();
        let mut channels = self.channels.write().await;
        channels.insert(channel_id.clone(), channel);
        Ok(channel_id)
    }

    /// Get channel
    pub async fn get_channel(&self, id: &str) -> Option<NotificationChannel> {
        let channels = self.channels.read().await;
        channels.get(id).cloned()
    }

    /// Send notification (simplified)
    pub async fn send_notification(&self, channel_id: &str, alert: &ComplianceAlert) -> Result<(), String> {
        let channels = self.channels.read().await;
        if let Some(channel) = channels.get(channel_id) {
            if !channel.enabled {
                return Err("Channel is disabled".to_string());
            }

            if alert.severity < channel.min_severity {
                return Ok(()); // Below minimum severity
            }

            // In production, actually send the notification here based on channel type
            // For now, just log
            println!("Sending notification via {:?}: {}", channel.channel_type, alert.title);

            Ok(())
        } else {
            Err("Channel not found".to_string())
        }
    }

    // ========================================================================
    // Anomaly Detection
    // ========================================================================

    /// Record metric for anomaly detection
    pub async fn record_metric(&self, metric: impl Into<String>, value: f64) {
        let mut detector = self.anomaly_detector.write().await;
        detector.record(metric, value);
    }

    /// Check for anomaly
    pub async fn check_anomaly(&self, metric: &str, value: f64, sensitivity: f64) -> bool {
        let detector = self.anomaly_detector.read().await;
        detector.is_anomaly(metric, value, sensitivity)
    }

    // ========================================================================
    // Escalation
    // ========================================================================

    /// Add escalation policy
    pub async fn add_escalation_policy(&self, policy: EscalationPolicy) -> Result<Uuid, String> {
        let policy_id = policy.id;
        let mut policies = self.escalation_policies.write().await;
        policies.insert(policy_id, policy);
        Ok(policy_id)
    }

    /// Check and escalate alerts
    pub async fn check_escalations(&self) -> Result<Vec<Uuid>, String> {
        let mut escalated = Vec::new();
        let mut alerts = self.alerts.write().await;

        for alert in alerts.values_mut() {
            if alert.status == AlertStatus::New || alert.status == AlertStatus::Acknowledged {
                // Check if alert should be escalated based on age
                let age = Utc::now() - alert.triggered_at;
                let escalation_threshold = Duration::minutes(30 * (alert.escalation_level as i64 + 1));

                if age > escalation_threshold {
                    alert.escalate();
                    escalated.push(alert.id);
                }
            }
        }

        Ok(escalated)
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_alert_creation() {
        let manager = AlertManager::new();

        let alert = ComplianceAlert::new(
            "Test Alert",
            "Test description",
            AlertCategory::Compliance,
            AlertSeverity::High,
        );

        let alert_id = manager.trigger_alert(alert).await.unwrap();
        let retrieved = manager.get_alert(alert_id).await.unwrap();

        assert_eq!(retrieved.title, "Test Alert");
        assert_eq!(retrieved.severity, AlertSeverity::High);
    }

    #[tokio::test]
    async fn test_alert_acknowledgment() {
        let manager = AlertManager::new();

        let alert = ComplianceAlert::new(
            "Test Alert",
            "Description",
            AlertCategory::Security,
            AlertSeverity::Medium,
        );

        let alert_id = manager.trigger_alert(alert).await.unwrap();
        manager.acknowledge_alert(alert_id, "user1").await.unwrap();

        let alert = manager.get_alert(alert_id).await.unwrap();
        assert_eq!(alert.status, AlertStatus::Acknowledged);
        assert_eq!(alert.acknowledged_by, Some("user1".to_string()));
    }

    #[tokio::test]
    async fn test_alert_resolution() {
        let manager = AlertManager::new();

        let alert = ComplianceAlert::new(
            "Test Alert",
            "Description",
            AlertCategory::Compliance,
            AlertSeverity::Low,
        );

        let alert_id = manager.trigger_alert(alert).await.unwrap();
        manager
            .resolve_alert(alert_id, "user1", Some("Fixed".to_string()))
            .await
            .unwrap();

        let alert = manager.get_alert(alert_id).await.unwrap();
        assert_eq!(alert.status, AlertStatus::Resolved);
        assert_eq!(alert.resolution_notes, Some("Fixed".to_string()));
    }

    #[tokio::test]
    async fn test_anomaly_detection() {
        let manager = AlertManager::new();

        // Build baseline
        for i in 0..100 {
            manager.record_metric("test_metric", 100.0 + (i as f64)).await;
        }

        // Normal value - not anomalous
        let is_anomaly = manager.check_anomaly("test_metric", 150.0, 3.0).await;
        assert!(!is_anomaly);

        // Anomalous value
        let is_anomaly = manager.check_anomaly("test_metric", 500.0, 3.0).await;
        assert!(is_anomaly);
    }

    #[tokio::test]
    async fn test_alert_rules() {
        let manager = AlertManager::new();

        let rule = AlertRule {
            id: Uuid::new_v4(),
            name: "Failed Login Attempts".to_string(),
            description: "Alert on excessive failed logins".to_string(),
            category: AlertCategory::Security,
            severity: AlertSeverity::High,
            conditions: vec![RuleCondition::Threshold {
                metric: "failed_logins".to_string(),
                operator: ComparisonOperator::GreaterThan,
                value: 5.0,
            }],
            enabled: true,
            notification_channels: vec!["email".to_string()],
            cooldown: Some(Duration::minutes(15)),
            last_triggered: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: "system".to_string(),
        };

        let rule_id = manager.add_rule(rule).await.unwrap();
        let enabled_rules = manager.get_enabled_rules().await;

        assert_eq!(enabled_rules.len(), 1);
        assert_eq!(enabled_rules[0].id, rule_id);
    }

    #[tokio::test]
    async fn test_notification_channels() {
        let manager = AlertManager::new();

        let channel = NotificationChannel {
            id: "email-primary".to_string(),
            name: "Primary Email".to_string(),
            channel_type: ChannelType::Email,
            config: {
                let mut config = HashMap::new();
                config.insert("recipients".to_string(), "admin@example.com".to_string());
                config
            },
            enabled: true,
            min_severity: AlertSeverity::Medium,
        };

        manager.add_channel(channel).await.unwrap();

        let alert = ComplianceAlert::new(
            "Test",
            "Description",
            AlertCategory::Security,
            AlertSeverity::High,
        );

        let result = manager.send_notification("email-primary", &alert).await;
        assert!(result.is_ok());
    }
}
