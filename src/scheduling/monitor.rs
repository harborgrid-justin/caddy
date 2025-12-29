//! Site monitoring system with continuous monitoring and change detection
//!
//! This module provides:
//! - Continuous monitoring mode
//! - Change detection and diffing
//! - Regression alerts
//! - Uptime tracking
//! - Performance monitoring

use chrono::{DateTime, Duration, Utc};
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Monitor errors
#[derive(Error, Debug)]
pub enum MonitorError {
    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Monitor not found: {0}")]
    MonitorNotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Check failed: {0}")]
    CheckFailed(String),
}

/// Result type for monitor operations
pub type MonitorResult<T> = Result<T, MonitorError>;

/// Monitor status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonitorStatus {
    Up,
    Down,
    Degraded,
    Unknown,
}

/// Monitor check type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckType {
    /// HTTP endpoint check
    Http {
        url: String,
        method: String,
        expected_status: u16,
        timeout_ms: u64,
    },

    /// Accessibility scan check
    AccessibilityScan {
        url: String,
        standards: Vec<String>,
    },

    /// Content change detection
    ContentChange {
        url: String,
        selector: Option<String>,
        hash_algorithm: String,
    },

    /// Performance check
    Performance {
        url: String,
        max_load_time_ms: u64,
        max_first_byte_ms: u64,
    },

    /// Custom check (extensible)
    Custom {
        check_type: String,
        config: serde_json::Value,
    },
}

/// Check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub check_id: String,
    pub monitor_id: String,
    pub status: MonitorStatus,
    pub response_time_ms: u64,
    pub timestamp: DateTime<Utc>,
    pub error: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl CheckResult {
    /// Create a successful check result
    pub fn success(check_id: String, monitor_id: String, response_time_ms: u64) -> Self {
        Self {
            check_id,
            monitor_id,
            status: MonitorStatus::Up,
            response_time_ms,
            timestamp: Utc::now(),
            error: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a failed check result
    pub fn failure(check_id: String, monitor_id: String, error: String) -> Self {
        Self {
            check_id,
            monitor_id,
            status: MonitorStatus::Down,
            response_time_ms: 0,
            timestamp: Utc::now(),
            error: Some(error),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Change detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeDetection {
    pub monitor_id: String,
    pub previous_hash: String,
    pub current_hash: String,
    pub changed: bool,
    pub change_percentage: f64,
    pub detected_at: DateTime<Utc>,
    pub details: Option<String>,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub monitor_id: String,
    pub timestamp: DateTime<Utc>,
    pub dns_time_ms: u64,
    pub connect_time_ms: u64,
    pub first_byte_time_ms: u64,
    pub download_time_ms: u64,
    pub total_time_ms: u64,
    pub content_size_bytes: u64,
    pub http_status: u16,
}

/// Uptime statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UptimeStats {
    pub monitor_id: String,
    pub total_checks: u64,
    pub successful_checks: u64,
    pub failed_checks: u64,
    pub uptime_percentage: f64,
    pub average_response_time_ms: f64,
    pub last_downtime: Option<DateTime<Utc>>,
    pub last_check: Option<DateTime<Utc>>,
}

impl UptimeStats {
    /// Create new uptime stats
    pub fn new(monitor_id: String) -> Self {
        Self {
            monitor_id,
            total_checks: 0,
            successful_checks: 0,
            failed_checks: 0,
            uptime_percentage: 100.0,
            average_response_time_ms: 0.0,
            last_downtime: None,
            last_check: None,
        }
    }

    /// Update stats with check result
    pub fn update(&mut self, result: &CheckResult) {
        self.total_checks += 1;
        self.last_check = Some(result.timestamp);

        match result.status {
            MonitorStatus::Up => {
                self.successful_checks += 1;
            }
            MonitorStatus::Down => {
                self.failed_checks += 1;
                self.last_downtime = Some(result.timestamp);
            }
            _ => {}
        }

        // Calculate uptime percentage
        if self.total_checks > 0 {
            self.uptime_percentage =
                (self.successful_checks as f64 / self.total_checks as f64) * 100.0;
        }

        // Update average response time (running average)
        let old_avg = self.average_response_time_ms;
        let new_value = result.response_time_ms as f64;
        self.average_response_time_ms =
            old_avg + (new_value - old_avg) / self.total_checks as f64;
    }
}

/// Monitor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monitor {
    pub id: String,
    pub name: String,
    pub check_type: CheckType,
    pub interval_seconds: u64,
    pub enabled: bool,
    pub alert_on_failure: bool,
    pub alert_threshold: u32, // Number of consecutive failures before alerting
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: HashMap<String, String>,
}

impl Monitor {
    /// Create a new monitor
    pub fn new(name: String, check_type: CheckType, interval_seconds: u64) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            check_type,
            interval_seconds,
            enabled: true,
            alert_on_failure: true,
            alert_threshold: 3,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: HashMap::new(),
        }
    }

    /// Update monitor
    pub fn update(&mut self) {
        self.updated_at = Utc::now();
    }
}

/// Alert for monitor issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorAlert {
    pub id: String,
    pub monitor_id: String,
    pub monitor_name: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub consecutive_failures: u32,
    pub created_at: DateTime<Utc>,
    pub acknowledged: bool,
    pub resolved: bool,
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Monitoring system
pub struct MonitoringSystem {
    monitors: Arc<RwLock<HashMap<String, Monitor>>>,
    uptime_stats: Arc<RwLock<HashMap<String, UptimeStats>>>,
    recent_results: Arc<RwLock<HashMap<String, VecDeque<CheckResult>>>>,
    alerts: Arc<RwLock<HashMap<String, MonitorAlert>>>,
    change_history: Arc<RwLock<HashMap<String, Vec<ChangeDetection>>>>,
    performance_history: Arc<RwLock<HashMap<String, VecDeque<PerformanceMetrics>>>>,
    redis: ConnectionManager,
    max_history_size: usize,
}

impl MonitoringSystem {
    /// Create a new monitoring system
    pub async fn new(redis_url: &str) -> MonitorResult<Self> {
        let client = redis::Client::open(redis_url)?;
        let redis = ConnectionManager::new(client).await?;

        Ok(Self {
            monitors: Arc::new(RwLock::new(HashMap::new())),
            uptime_stats: Arc::new(RwLock::new(HashMap::new())),
            recent_results: Arc::new(RwLock::new(HashMap::new())),
            alerts: Arc::new(RwLock::new(HashMap::new())),
            change_history: Arc::new(RwLock::new(HashMap::new())),
            performance_history: Arc::new(RwLock::new(HashMap::new())),
            redis,
            max_history_size: 1000,
        })
    }

    /// Add a monitor
    pub async fn add_monitor(&self, monitor: Monitor) -> MonitorResult<String> {
        let monitor_id = monitor.id.clone();

        // Persist to Redis
        self.persist_monitor(&monitor).await?;

        // Store in memory
        let mut monitors = self.monitors.write().await;
        monitors.insert(monitor_id.clone(), monitor);

        // Initialize uptime stats
        let mut stats = self.uptime_stats.write().await;
        stats.insert(monitor_id.clone(), UptimeStats::new(monitor_id.clone()));

        Ok(monitor_id)
    }

    /// Remove a monitor
    pub async fn remove_monitor(&self, monitor_id: &str) -> MonitorResult<()> {
        let mut monitors = self.monitors.write().await;
        monitors.remove(monitor_id);

        // Clean up Redis
        let monitor_key = format!("monitor:{}", monitor_id);
        redis::cmd("DEL")
            .arg(&monitor_key)
            .query_async(&mut self.redis.clone())
            .await?;

        Ok(())
    }

    /// Get monitor
    pub async fn get_monitor(&self, monitor_id: &str) -> Option<Monitor> {
        let monitors = self.monitors.read().await;
        monitors.get(monitor_id).cloned()
    }

    /// List all monitors
    pub async fn list_monitors(&self) -> Vec<Monitor> {
        let monitors = self.monitors.read().await;
        monitors.values().cloned().collect()
    }

    /// Record check result
    pub async fn record_check(&self, result: CheckResult) -> MonitorResult<()> {
        let monitor_id = result.monitor_id.clone();

        // Update uptime stats
        {
            let mut stats = self.uptime_stats.write().await;
            if let Some(stat) = stats.get_mut(&monitor_id) {
                stat.update(&result);
            }
        }

        // Store result in recent history
        {
            let mut recent = self.recent_results.write().await;
            let results = recent.entry(monitor_id.clone()).or_insert_with(VecDeque::new);
            results.push_back(result.clone());

            // Keep only last N results
            while results.len() > self.max_history_size {
                results.pop_front();
            }
        }

        // Check for consecutive failures and create alerts
        if result.status == MonitorStatus::Down {
            self.check_and_create_alert(&monitor_id).await?;
        } else {
            // Resolve alerts if monitor is back up
            self.resolve_alerts(&monitor_id).await?;
        }

        // Persist to Redis
        self.persist_check_result(&result).await?;

        Ok(())
    }

    /// Record change detection
    pub async fn record_change(&self, change: ChangeDetection) -> MonitorResult<()> {
        let monitor_id = change.monitor_id.clone();

        let mut history = self.change_history.write().await;
        let changes = history.entry(monitor_id.clone()).or_insert_with(Vec::new);
        changes.push(change.clone());

        // Keep only last 100 changes
        if changes.len() > 100 {
            changes.remove(0);
        }

        // Persist to Redis
        let change_key = format!("change:{}:{}", monitor_id, change.detected_at.timestamp());
        let change_data = serde_json::to_string(&change)?;

        redis::cmd("SET")
            .arg(&change_key)
            .arg(&change_data)
            .arg("EX")
            .arg(86400 * 30) // Keep for 30 days
            .query_async(&mut self.redis.clone())
            .await?;

        Ok(())
    }

    /// Record performance metrics
    pub async fn record_performance(&self, metrics: PerformanceMetrics) -> MonitorResult<()> {
        let monitor_id = metrics.monitor_id.clone();

        let mut history = self.performance_history.write().await;
        let perf_data = history.entry(monitor_id.clone()).or_insert_with(VecDeque::new);
        perf_data.push_back(metrics.clone());

        // Keep only last 1000 metrics
        while perf_data.len() > 1000 {
            perf_data.pop_front();
        }

        // Persist to Redis
        let perf_key = format!("perf:{}:{}", monitor_id, metrics.timestamp.timestamp());
        let perf_json = serde_json::to_string(&metrics)?;

        redis::cmd("SET")
            .arg(&perf_key)
            .arg(&perf_json)
            .arg("EX")
            .arg(86400 * 7) // Keep for 7 days
            .query_async(&mut self.redis.clone())
            .await?;

        Ok(())
    }

    /// Get uptime statistics
    pub async fn get_uptime_stats(&self, monitor_id: &str) -> Option<UptimeStats> {
        let stats = self.uptime_stats.read().await;
        stats.get(monitor_id).cloned()
    }

    /// Get recent check results
    pub async fn get_recent_results(&self, monitor_id: &str, limit: usize) -> Vec<CheckResult> {
        let recent = self.recent_results.read().await;
        if let Some(results) = recent.get(monitor_id) {
            results.iter().rev().take(limit).cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Get change history
    pub async fn get_change_history(&self, monitor_id: &str) -> Vec<ChangeDetection> {
        let history = self.change_history.read().await;
        history.get(monitor_id).cloned().unwrap_or_default()
    }

    /// Get performance history
    pub async fn get_performance_history(
        &self,
        monitor_id: &str,
        limit: usize,
    ) -> Vec<PerformanceMetrics> {
        let history = self.performance_history.read().await;
        if let Some(metrics) = history.get(monitor_id) {
            metrics.iter().rev().take(limit).cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<MonitorAlert> {
        let alerts = self.alerts.read().await;
        alerts
            .values()
            .filter(|a| !a.resolved)
            .cloned()
            .collect()
    }

    /// Acknowledge alert
    pub async fn acknowledge_alert(&self, alert_id: &str) -> MonitorResult<()> {
        let mut alerts = self.alerts.write().await;
        if let Some(alert) = alerts.get_mut(alert_id) {
            alert.acknowledged = true;
        }
        Ok(())
    }

    /// Check for consecutive failures and create alerts
    async fn check_and_create_alert(&self, monitor_id: &str) -> MonitorResult<()> {
        let monitor = {
            let monitors = self.monitors.read().await;
            monitors.get(monitor_id).cloned()
        };

        let monitor = match monitor {
            Some(m) => m,
            None => return Ok(()),
        };

        if !monitor.alert_on_failure {
            return Ok(());
        }

        // Count consecutive failures
        let consecutive_failures = self.count_consecutive_failures(monitor_id).await;

        if consecutive_failures >= monitor.alert_threshold {
            let alert = MonitorAlert {
                id: Uuid::new_v4().to_string(),
                monitor_id: monitor_id.to_string(),
                monitor_name: monitor.name.clone(),
                severity: AlertSeverity::Error,
                message: format!(
                    "Monitor '{}' has failed {} times consecutively",
                    monitor.name, consecutive_failures
                ),
                consecutive_failures,
                created_at: Utc::now(),
                acknowledged: false,
                resolved: false,
            };

            let mut alerts = self.alerts.write().await;
            alerts.insert(alert.id.clone(), alert);
        }

        Ok(())
    }

    /// Resolve alerts for a monitor
    async fn resolve_alerts(&self, monitor_id: &str) -> MonitorResult<()> {
        let mut alerts = self.alerts.write().await;

        for alert in alerts.values_mut() {
            if alert.monitor_id == monitor_id && !alert.resolved {
                alert.resolved = true;
            }
        }

        Ok(())
    }

    /// Count consecutive failures
    async fn count_consecutive_failures(&self, monitor_id: &str) -> u32 {
        let recent = self.recent_results.read().await;

        if let Some(results) = recent.get(monitor_id) {
            let mut count = 0;
            for result in results.iter().rev() {
                if result.status == MonitorStatus::Down {
                    count += 1;
                } else {
                    break;
                }
            }
            count
        } else {
            0
        }
    }

    /// Persist monitor to Redis
    async fn persist_monitor(&self, monitor: &Monitor) -> MonitorResult<()> {
        let monitor_key = format!("monitor:{}", monitor.id);
        let monitor_data = serde_json::to_string(monitor)?;

        redis::cmd("SET")
            .arg(&monitor_key)
            .arg(&monitor_data)
            .query_async(&mut self.redis.clone())
            .await?;

        Ok(())
    }

    /// Persist check result to Redis
    async fn persist_check_result(&self, result: &CheckResult) -> MonitorResult<()> {
        let result_key = format!(
            "check:{}:{}",
            result.monitor_id,
            result.timestamp.timestamp()
        );
        let result_data = serde_json::to_string(result)?;

        redis::cmd("SET")
            .arg(&result_key)
            .arg(&result_data)
            .arg("EX")
            .arg(86400 * 7) // Keep for 7 days
            .query_async(&mut self.redis.clone())
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_creation() {
        let check_type = CheckType::Http {
            url: "https://example.com".to_string(),
            method: "GET".to_string(),
            expected_status: 200,
            timeout_ms: 5000,
        };

        let monitor = Monitor::new("Test Monitor".to_string(), check_type, 60);

        assert_eq!(monitor.name, "Test Monitor");
        assert_eq!(monitor.interval_seconds, 60);
        assert!(monitor.enabled);
    }

    #[test]
    fn test_uptime_stats() {
        let mut stats = UptimeStats::new("monitor-1".to_string());

        let success = CheckResult::success("check-1".to_string(), "monitor-1".to_string(), 100);
        stats.update(&success);

        assert_eq!(stats.total_checks, 1);
        assert_eq!(stats.successful_checks, 1);
        assert_eq!(stats.uptime_percentage, 100.0);

        let failure = CheckResult::failure(
            "check-2".to_string(),
            "monitor-1".to_string(),
            "Error".to_string(),
        );
        stats.update(&failure);

        assert_eq!(stats.total_checks, 2);
        assert_eq!(stats.failed_checks, 1);
        assert_eq!(stats.uptime_percentage, 50.0);
    }

    #[test]
    fn test_check_result() {
        let result = CheckResult::success("check-1".to_string(), "monitor-1".to_string(), 150);

        assert_eq!(result.status, MonitorStatus::Up);
        assert_eq!(result.response_time_ms, 150);
        assert!(result.error.is_none());
    }
}
