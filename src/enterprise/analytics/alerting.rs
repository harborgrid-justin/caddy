//! Alert system for metrics monitoring
//!
//! This module provides alert rules, conditions, routing,
//! and alert history management.

use super::{MetricRegistry, Result, AnalyticsError};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    /// Informational alert
    Info,
    /// Warning - requires attention
    Warning,
    /// Error - requires immediate attention
    Error,
    /// Critical - system failure imminent
    Critical,
}

impl AlertSeverity {
    /// Convert to string
    pub fn as_str(&self) -> &str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Critical => "critical",
        }
    }
}

/// Alert state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertState {
    /// Alert is inactive
    Inactive,
    /// Alert is pending (condition met but not yet triggered)
    Pending,
    /// Alert is active
    Active,
    /// Alert has been resolved
    Resolved,
}

/// Comparison operators for conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comparator {
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

impl Comparator {
    /// Evaluate comparison
    pub fn evaluate(&self, left: f64, right: f64) -> bool {
        match self {
            Self::GreaterThan => left > right,
            Self::GreaterThanOrEqual => left >= right,
            Self::LessThan => left < right,
            Self::LessThanOrEqual => left <= right,
            Self::Equal => (left - right).abs() < f64::EPSILON,
            Self::NotEqual => (left - right).abs() >= f64::EPSILON,
        }
    }
}

/// Alert condition types
#[derive(Debug, Clone)]
pub enum AlertCondition {
    /// Simple threshold condition
    Threshold {
        metric: String,
        comparator: Comparator,
        value: f64,
    },
    /// Rate of change condition
    RateOfChange {
        metric: String,
        comparator: Comparator,
        rate: f64,
        window: Duration,
    },
    /// Anomaly detection
    Anomaly {
        metric: String,
        std_dev_threshold: f64,
        window: Duration,
    },
    /// Multiple metrics comparison
    Comparison {
        metric_a: String,
        metric_b: String,
        comparator: Comparator,
    },
    /// Composite condition (AND/OR)
    Composite {
        conditions: Vec<AlertCondition>,
        operator: LogicOperator,
    },
}

/// Logic operators for composite conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicOperator {
    And,
    Or,
}

impl AlertCondition {
    /// Create a simple threshold condition
    pub fn threshold(metric: impl Into<String>, comparator: Comparator, value: f64) -> Self {
        Self::Threshold {
            metric: metric.into(),
            comparator,
            value,
        }
    }

    /// Create a rate of change condition
    pub fn rate_of_change(
        metric: impl Into<String>,
        comparator: Comparator,
        rate: f64,
        window: Duration,
    ) -> Self {
        Self::RateOfChange {
            metric: metric.into(),
            comparator,
            rate,
            window,
        }
    }

    /// Create an anomaly detection condition
    pub fn anomaly(metric: impl Into<String>, std_dev_threshold: f64, window: Duration) -> Self {
        Self::Anomaly {
            metric: metric.into(),
            std_dev_threshold,
            window,
        }
    }

    /// Create a metric comparison condition
    pub fn comparison(
        metric_a: impl Into<String>,
        metric_b: impl Into<String>,
        comparator: Comparator,
    ) -> Self {
        Self::Comparison {
            metric_a: metric_a.into(),
            metric_b: metric_b.into(),
            comparator,
        }
    }

    /// Create a composite AND condition
    pub fn and(conditions: Vec<AlertCondition>) -> Self {
        Self::Composite {
            conditions,
            operator: LogicOperator::And,
        }
    }

    /// Create a composite OR condition
    pub fn or(conditions: Vec<AlertCondition>) -> Self {
        Self::Composite {
            conditions,
            operator: LogicOperator::Or,
        }
    }

    /// Evaluate the condition against a metric registry
    pub fn evaluate(&self, registry: &MetricRegistry) -> bool {
        match self {
            Self::Threshold {
                metric,
                comparator,
                value,
            } => {
                if let Some(m) = registry.get(metric) {
                    let current = match m {
                        super::metrics::Metric::Counter(c) => c.get(),
                        super::metrics::Metric::Gauge(g) => g.get(),
                        super::metrics::Metric::Histogram(h) => h.mean(),
                        super::metrics::Metric::Summary(s) => s.mean(),
                    };
                    comparator.evaluate(current, *value)
                } else {
                    false
                }
            }
            Self::RateOfChange { metric, .. } => {
                // Simplified - in production, calculate actual rate
                registry.get(metric).is_some()
            }
            Self::Anomaly { metric, .. } => {
                // Simplified - in production, use actual anomaly detection
                registry.get(metric).is_some()
            }
            Self::Comparison {
                metric_a,
                metric_b,
                comparator,
            } => {
                let val_a = registry.get(metric_a).and_then(|m| match m {
                    super::metrics::Metric::Counter(c) => Some(c.get()),
                    super::metrics::Metric::Gauge(g) => Some(g.get()),
                    super::metrics::Metric::Histogram(h) => Some(h.mean()),
                    super::metrics::Metric::Summary(s) => Some(s.mean()),
                });

                let val_b = registry.get(metric_b).and_then(|m| match m {
                    super::metrics::Metric::Counter(c) => Some(c.get()),
                    super::metrics::Metric::Gauge(g) => Some(g.get()),
                    super::metrics::Metric::Histogram(h) => Some(h.mean()),
                    super::metrics::Metric::Summary(s) => Some(s.mean()),
                });

                if let (Some(a), Some(b)) = (val_a, val_b) {
                    comparator.evaluate(a, b)
                } else {
                    false
                }
            }
            Self::Composite {
                conditions,
                operator,
            } => match operator {
                LogicOperator::And => conditions.iter().all(|c| c.evaluate(registry)),
                LogicOperator::Or => conditions.iter().any(|c| c.evaluate(registry)),
            },
        }
    }
}

/// Alert rule configuration
#[derive(Debug, Clone)]
pub struct AlertRule {
    /// Rule ID
    pub id: String,
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Alert condition
    pub condition: AlertCondition,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Evaluation interval
    pub interval: Duration,
    /// Number of consecutive evaluations before triggering
    pub for_duration: Option<Duration>,
    /// Alert labels
    pub labels: HashMap<String, String>,
    /// Notification channels
    pub channels: Vec<String>,
    /// Rule enabled
    pub enabled: bool,
}

impl AlertRule {
    /// Create a new alert rule
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        condition: AlertCondition,
        severity: AlertSeverity,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            condition,
            severity,
            interval: Duration::from_secs(60),
            for_duration: None,
            labels: HashMap::new(),
            channels: Vec::new(),
            enabled: true,
        }
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set evaluation interval
    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    /// Set for duration
    pub fn for_duration(mut self, duration: Duration) -> Self {
        self.for_duration = Some(duration);
        self
    }

    /// Add a label
    pub fn label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Add a notification channel
    pub fn channel(mut self, channel: impl Into<String>) -> Self {
        self.channels.push(channel.into());
        self
    }

    /// Set enabled state
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Alert instance
#[derive(Debug, Clone)]
pub struct Alert {
    /// Alert ID
    pub id: String,
    /// Rule ID that triggered this alert
    pub rule_id: String,
    /// Alert name
    pub name: String,
    /// Alert message
    pub message: String,
    /// Severity
    pub severity: AlertSeverity,
    /// Current state
    pub state: AlertState,
    /// Time when alert was first triggered
    pub triggered_at: u64,
    /// Time when alert was resolved
    pub resolved_at: Option<u64>,
    /// Labels
    pub labels: HashMap<String, String>,
    /// Current value that triggered the alert
    pub value: Option<f64>,
}

impl Alert {
    /// Create a new alert
    pub fn new(rule: &AlertRule, value: Option<f64>) -> Self {
        Self {
            id: format!("{}_{}", rule.id, current_timestamp()),
            rule_id: rule.id.clone(),
            name: rule.name.clone(),
            message: format!("{}: {}", rule.name, rule.description),
            severity: rule.severity,
            state: AlertState::Active,
            triggered_at: current_timestamp(),
            resolved_at: None,
            labels: rule.labels.clone(),
            value,
        }
    }

    /// Mark alert as resolved
    pub fn resolve(&mut self) {
        self.state = AlertState::Resolved;
        self.resolved_at = Some(current_timestamp());
    }

    /// Get alert duration in seconds
    pub fn duration(&self) -> u64 {
        let end = self.resolved_at.unwrap_or_else(current_timestamp);
        end.saturating_sub(self.triggered_at)
    }
}

/// Alert manager
pub struct AlertManager {
    registry: MetricRegistry,
    rules: Arc<RwLock<HashMap<String, AlertRule>>>,
    active_alerts: Arc<RwLock<HashMap<String, Alert>>>,
    alert_history: Arc<RwLock<VecDeque<Alert>>>,
    max_history: usize,
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new(registry: MetricRegistry) -> Self {
        Self {
            registry,
            rules: Arc::new(RwLock::new(HashMap::new())),
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(RwLock::new(VecDeque::new())),
            max_history: 1000,
        }
    }

    /// Add an alert rule
    pub fn add_rule(&self, rule: AlertRule) -> Result<()> {
        let mut rules = self.rules.write().unwrap();

        if rules.contains_key(&rule.id) {
            return Err(AnalyticsError::AlertError(format!(
                "Rule '{}' already exists",
                rule.id
            )));
        }

        rules.insert(rule.id.clone(), rule);
        Ok(())
    }

    /// Remove an alert rule
    pub fn remove_rule(&self, rule_id: &str) -> Result<()> {
        let mut rules = self.rules.write().unwrap();
        rules.remove(rule_id).ok_or_else(|| {
            AnalyticsError::AlertError(format!("Rule '{}' not found", rule_id))
        })?;
        Ok(())
    }

    /// Get a rule by ID
    pub fn get_rule(&self, rule_id: &str) -> Option<AlertRule> {
        self.rules.read().unwrap().get(rule_id).cloned()
    }

    /// List all rules
    pub fn list_rules(&self) -> Vec<AlertRule> {
        self.rules.read().unwrap().values().cloned().collect()
    }

    /// Enable a rule
    pub fn enable_rule(&self, rule_id: &str) -> Result<()> {
        let mut rules = self.rules.write().unwrap();
        let rule = rules.get_mut(rule_id).ok_or_else(|| {
            AnalyticsError::AlertError(format!("Rule '{}' not found", rule_id))
        })?;
        rule.enabled = true;
        Ok(())
    }

    /// Disable a rule
    pub fn disable_rule(&self, rule_id: &str) -> Result<()> {
        let mut rules = self.rules.write().unwrap();
        let rule = rules.get_mut(rule_id).ok_or_else(|| {
            AnalyticsError::AlertError(format!("Rule '{}' not found", rule_id))
        })?;
        rule.enabled = false;
        Ok(())
    }

    /// Evaluate all rules and update alerts
    pub fn evaluate(&self) -> Vec<Alert> {
        let rules = self.rules.read().unwrap();
        let mut new_alerts = Vec::new();

        for rule in rules.values() {
            if !rule.enabled {
                continue;
            }

            let triggered = rule.condition.evaluate(&self.registry);

            if triggered {
                // Check if alert already exists
                let active_alerts = self.active_alerts.read().unwrap();
                if !active_alerts.contains_key(&rule.id) {
                    drop(active_alerts);

                    let alert = Alert::new(rule, None);
                    new_alerts.push(alert.clone());

                    // Add to active alerts
                    let mut active_alerts = self.active_alerts.write().unwrap();
                    active_alerts.insert(rule.id.clone(), alert);
                }
            } else {
                // Resolve alert if it exists
                let mut active_alerts = self.active_alerts.write().unwrap();
                if let Some(mut alert) = active_alerts.remove(&rule.id) {
                    alert.resolve();
                    self.add_to_history(alert);
                }
            }
        }

        new_alerts
    }

    /// Get active alerts
    pub fn active_alerts(&self) -> Vec<Alert> {
        self.active_alerts
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }

    /// Get alert history
    pub fn alert_history(&self, limit: Option<usize>) -> Vec<Alert> {
        let history = self.alert_history.read().unwrap();
        let limit = limit.unwrap_or(history.len());
        history.iter().take(limit).cloned().collect()
    }

    /// Acknowledge an alert
    pub fn acknowledge_alert(&self, alert_id: &str) -> Result<()> {
        let mut active_alerts = self.active_alerts.write().unwrap();
        active_alerts
            .get_mut(alert_id)
            .ok_or_else(|| {
                AnalyticsError::AlertError(format!("Alert '{}' not found", alert_id))
            })?;
        Ok(())
    }

    /// Clear alert history
    pub fn clear_history(&self) {
        let mut history = self.alert_history.write().unwrap();
        history.clear();
    }

    /// Get alerts by severity
    pub fn alerts_by_severity(&self, severity: AlertSeverity) -> Vec<Alert> {
        self.active_alerts
            .read()
            .unwrap()
            .values()
            .filter(|a| a.severity == severity)
            .cloned()
            .collect()
    }

    fn add_to_history(&self, alert: Alert) {
        let mut history = self.alert_history.write().unwrap();
        history.push_front(alert);

        // Limit history size
        while history.len() > self.max_history {
            history.pop_back();
        }
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enterprise::analytics::metrics::Labels;

    #[test]
    fn test_comparator() {
        assert!(Comparator::GreaterThan.evaluate(10.0, 5.0));
        assert!(!Comparator::GreaterThan.evaluate(5.0, 10.0));
        assert!(Comparator::LessThan.evaluate(5.0, 10.0));
        assert!(Comparator::Equal.evaluate(5.0, 5.0));
    }

    #[test]
    fn test_alert_condition() {
        let registry = MetricRegistry::new();
        let gauge = registry.gauge("cpu", Labels::new());
        gauge.set(80.0);

        let condition = AlertCondition::threshold("caddy_cpu", Comparator::GreaterThan, 70.0);
        assert!(condition.evaluate(&registry));

        let condition = AlertCondition::threshold("caddy_cpu", Comparator::GreaterThan, 90.0);
        assert!(!condition.evaluate(&registry));
    }

    #[test]
    fn test_composite_condition() {
        let registry = MetricRegistry::new();
        let cpu = registry.gauge("cpu", Labels::new());
        cpu.set(80.0);
        let mem = registry.gauge("memory", Labels::new());
        mem.set(90.0);

        let condition = AlertCondition::and(vec![
            AlertCondition::threshold("caddy_cpu", Comparator::GreaterThan, 70.0),
            AlertCondition::threshold("caddy_memory", Comparator::GreaterThan, 80.0),
        ]);

        assert!(condition.evaluate(&registry));
    }

    #[test]
    fn test_alert_rule() {
        let condition = AlertCondition::threshold("cpu", Comparator::GreaterThan, 80.0);
        let rule = AlertRule::new("cpu_high", "High CPU", condition, AlertSeverity::Warning)
            .description("CPU usage is too high")
            .channel("email");

        assert_eq!(rule.id, "cpu_high");
        assert_eq!(rule.severity, AlertSeverity::Warning);
        assert_eq!(rule.channels.len(), 1);
    }

    #[test]
    fn test_alert_manager() {
        let registry = MetricRegistry::new();
        let manager = AlertManager::new(registry.clone());

        let condition = AlertCondition::threshold("caddy_cpu", Comparator::GreaterThan, 80.0);
        let rule = AlertRule::new("cpu_high", "High CPU", condition, AlertSeverity::Warning);

        manager.add_rule(rule).unwrap();
        assert_eq!(manager.list_rules().len(), 1);

        // Set metric to trigger alert
        let cpu = registry.gauge("cpu", Labels::new());
        cpu.set(90.0);

        let new_alerts = manager.evaluate();
        assert_eq!(new_alerts.len(), 1);
        assert_eq!(manager.active_alerts().len(), 1);
    }

    #[test]
    fn test_alert_severity() {
        assert!(AlertSeverity::Critical > AlertSeverity::Error);
        assert!(AlertSeverity::Error > AlertSeverity::Warning);
        assert!(AlertSeverity::Warning > AlertSeverity::Info);
    }
}
