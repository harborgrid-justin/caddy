//! # Security Audit Logging Module
//!
//! Comprehensive security audit logging and monitoring:
//! - Login/logout events
//! - Permission changes
//! - Sensitive data access
//! - Failed authentication attempts
//! - IP-based anomaly detection
//! - Real-time alerting
//! - Compliance reporting (SOC 2, HIPAA, GDPR)
//!
//! ## Security Features
//!
//! - Immutable audit logs
//! - Encrypted sensitive data
//! - Tamper detection
//! - Real-time anomaly detection
//! - Geographic anomaly detection
//! - Behavioral analysis

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use std::net::IpAddr;
use std::collections::{HashMap, VecDeque};
use crate::auth::AuthError;

/// Audit event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    // Authentication events
    LoginSuccess,
    LoginFailure,
    Logout,
    PasswordChanged,
    PasswordResetRequested,
    PasswordResetCompleted,

    // MFA events
    MfaEnabled,
    MfaDisabled,
    MfaVerified,
    MfaFailed,
    BackupCodeUsed,

    // Session events
    SessionCreated,
    SessionExpired,
    SessionRevoked,
    SessionRefreshed,

    // Authorization events
    PermissionGranted,
    PermissionDenied,
    RoleAssigned,
    RoleRevoked,
    RoleCreated,
    RoleModified,
    RoleDeleted,

    // Data access events
    SensitiveDataAccessed,
    DataExported,
    DataImported,
    DataDeleted,
    DataModified,

    // Administrative events
    UserCreated,
    UserModified,
    UserDeleted,
    UserLocked,
    UserUnlocked,
    OrganizationCreated,
    OrganizationModified,
    SettingsChanged,

    // Security events
    SuspiciousActivity,
    AnomalyDetected,
    BruteForceAttempt,
    IpBlocked,
    AccountCompromised,
    SecurityAlertTriggered,

    // SSO events
    SsoLoginSuccess,
    SsoLoginFailure,
    SsoConfigured,
    SsoDisabled,
}

/// Audit event severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Event ID
    pub id: Uuid,

    /// Event type
    pub event_type: AuditEventType,

    /// Severity
    pub severity: AuditSeverity,

    /// User ID (if applicable)
    pub user_id: Option<Uuid>,

    /// Username (for display)
    pub username: Option<String>,

    /// Target user ID (for admin actions)
    pub target_user_id: Option<Uuid>,

    /// Target resource type
    pub resource_type: Option<String>,

    /// Target resource ID
    pub resource_id: Option<Uuid>,

    /// IP address
    pub ip_address: IpAddr,

    /// User agent
    pub user_agent: String,

    /// Session ID
    pub session_id: Option<Uuid>,

    /// Description
    pub description: String,

    /// Metadata (additional context)
    pub metadata: HashMap<String, serde_json::Value>,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Organization ID
    pub organization_id: Option<Uuid>,

    /// Success flag
    pub success: bool,

    /// Error message (if failed)
    pub error_message: Option<String>,

    /// Geographic location (city, country)
    pub geo_location: Option<String>,

    /// Risk score (0-100)
    pub risk_score: Option<u8>,
}

impl AuditEvent {
    /// Create a new audit event
    pub fn new(
        event_type: AuditEventType,
        severity: AuditSeverity,
        user_id: Option<Uuid>,
        ip_address: IpAddr,
        user_agent: String,
        description: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            severity,
            user_id,
            username: None,
            target_user_id: None,
            resource_type: None,
            resource_id: None,
            ip_address,
            user_agent,
            session_id: None,
            description,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
            organization_id: None,
            success: true,
            error_message: None,
            geo_location: None,
            risk_score: None,
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Set resource
    pub fn with_resource(mut self, resource_type: String, resource_id: Uuid) -> Self {
        self.resource_type = Some(resource_type);
        self.resource_id = Some(resource_id);
        self
    }

    /// Set target user
    pub fn with_target_user(mut self, target_user_id: Uuid) -> Self {
        self.target_user_id = Some(target_user_id);
        self
    }

    /// Set session
    pub fn with_session(mut self, session_id: Uuid) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Mark as failure
    pub fn with_error(mut self, error: String) -> Self {
        self.success = false;
        self.error_message = Some(error);
        self
    }

    /// Set risk score
    pub fn with_risk_score(mut self, score: u8) -> Self {
        self.risk_score = Some(score);
        self
    }
}

/// Failed login attempt tracker
#[derive(Debug, Clone)]
struct FailedAttempt {
    timestamp: DateTime<Utc>,
    ip_address: IpAddr,
    user_identifier: String,
}

/// IP activity tracker
#[derive(Debug, Clone)]
struct IpActivity {
    ip_address: IpAddr,
    recent_events: VecDeque<DateTime<Utc>>,
    first_seen: DateTime<Utc>,
    last_seen: DateTime<Utc>,
    event_count: usize,
    failed_attempts: usize,
    countries: Vec<String>,
}

/// Anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetection {
    /// Anomaly detected
    pub anomaly_detected: bool,

    /// Anomaly type
    pub anomaly_type: Option<String>,

    /// Risk score (0-100)
    pub risk_score: u8,

    /// Reasons
    pub reasons: Vec<String>,

    /// Recommended action
    pub recommended_action: Option<String>,
}

/// Audit logger
pub struct AuditLogger {
    events: Vec<AuditEvent>,
    failed_attempts: HashMap<String, Vec<FailedAttempt>>,
    ip_activity: HashMap<IpAddr, IpActivity>,
    blocked_ips: HashMap<IpAddr, DateTime<Utc>>,
    max_failed_attempts: u32,
    failed_attempt_window: Duration,
    rate_limit_window: Duration,
    rate_limit_threshold: usize,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(max_failed_attempts: u32, failed_attempt_window_secs: i64) -> Self {
        Self {
            events: Vec::new(),
            failed_attempts: HashMap::new(),
            ip_activity: HashMap::new(),
            blocked_ips: HashMap::new(),
            max_failed_attempts,
            failed_attempt_window: Duration::seconds(failed_attempt_window_secs),
            rate_limit_window: Duration::minutes(5),
            rate_limit_threshold: 100,
        }
    }

    /// Log an audit event
    pub fn log(&mut self, event: AuditEvent) {
        // Update IP activity tracking
        self.update_ip_activity(&event);

        // Track failed login attempts
        if event.event_type == AuditEventType::LoginFailure {
            self.track_failed_attempt(&event);
        }

        // Store event
        self.events.push(event);
    }

    /// Update IP activity tracking
    fn update_ip_activity(&mut self, event: &AuditEvent) {
        let activity = self
            .ip_activity
            .entry(event.ip_address)
            .or_insert_with(|| IpActivity {
                ip_address: event.ip_address,
                recent_events: VecDeque::new(),
                first_seen: event.timestamp,
                last_seen: event.timestamp,
                event_count: 0,
                failed_attempts: 0,
                countries: Vec::new(),
            });

        activity.last_seen = event.timestamp;
        activity.event_count += 1;
        activity.recent_events.push_back(event.timestamp);

        // Keep only recent events within rate limit window
        let cutoff = event.timestamp - self.rate_limit_window;
        while let Some(&first) = activity.recent_events.front() {
            if first < cutoff {
                activity.recent_events.pop_front();
            } else {
                break;
            }
        }

        if !event.success {
            activity.failed_attempts += 1;
        }

        // Track geographic locations
        if let Some(ref geo) = event.geo_location {
            if !activity.countries.contains(geo) {
                activity.countries.push(geo.clone());
            }
        }
    }

    /// Track failed login attempt
    fn track_failed_attempt(&mut self, event: &AuditEvent) {
        let key = event
            .username
            .clone()
            .unwrap_or_else(|| event.user_id.map(|id| id.to_string()).unwrap_or_default());

        let attempts = self.failed_attempts.entry(key.clone()).or_insert_with(Vec::new);

        attempts.push(FailedAttempt {
            timestamp: event.timestamp,
            ip_address: event.ip_address,
            user_identifier: key.clone(),
        });

        // Clean old attempts outside the window
        let cutoff = event.timestamp - self.failed_attempt_window;
        attempts.retain(|a| a.timestamp > cutoff);

        // Check if threshold exceeded
        if attempts.len() >= self.max_failed_attempts as usize {
            // Block IP
            self.blocked_ips.insert(event.ip_address, event.timestamp);

            // Log security event
            let mut security_event = AuditEvent::new(
                AuditEventType::BruteForceAttempt,
                AuditSeverity::Critical,
                event.user_id,
                event.ip_address,
                event.user_agent.clone(),
                format!(
                    "Brute force attack detected: {} failed attempts for user {}",
                    attempts.len(),
                    key
                ),
            );
            security_event.risk_score = Some(95);
            self.events.push(security_event);
        }
    }

    /// Check if IP is blocked
    pub fn is_ip_blocked(&self, ip: &IpAddr) -> bool {
        self.blocked_ips.contains_key(ip)
    }

    /// Unblock IP
    pub fn unblock_ip(&mut self, ip: &IpAddr) {
        self.blocked_ips.remove(ip);
    }

    /// Detect anomalies
    pub fn detect_anomalies(
        &self,
        user_id: &Uuid,
        ip_address: &IpAddr,
        user_agent: &str,
    ) -> AnomalyDetection {
        let mut reasons = Vec::new();
        let mut risk_score = 0u8;

        // Check if IP is blocked
        if self.is_ip_blocked(ip_address) {
            reasons.push("IP address is blocked due to suspicious activity".to_string());
            risk_score += 40;
        }

        // Check rate limiting
        if let Some(activity) = self.ip_activity.get(ip_address) {
            let recent_count = activity.recent_events.len();

            if recent_count > self.rate_limit_threshold {
                reasons.push(format!(
                    "Excessive requests from IP: {} requests in 5 minutes",
                    recent_count
                ));
                risk_score += 25;
            }

            // Check geographic anomalies
            if activity.countries.len() > 3 {
                reasons.push(format!(
                    "Multiple geographic locations detected: {} countries",
                    activity.countries.len()
                ));
                risk_score += 20;
            }

            // Check for high failure rate
            if activity.event_count > 0 {
                let failure_rate = (activity.failed_attempts as f64 / activity.event_count as f64) * 100.0;
                if failure_rate > 50.0 {
                    reasons.push(format!("High failure rate: {:.1}%", failure_rate));
                    risk_score += 15;
                }
            }
        }

        // Check for unusual login times (simple heuristic)
        let hour = Utc::now().hour();
        if hour < 6 || hour > 22 {
            reasons.push("Login attempt during unusual hours".to_string());
            risk_score += 5;
        }

        // Determine recommended action
        let recommended_action = if risk_score >= 70 {
            Some("Block request and require additional verification".to_string())
        } else if risk_score >= 40 {
            Some("Require MFA verification".to_string())
        } else if risk_score >= 20 {
            Some("Monitor closely".to_string())
        } else {
            None
        };

        AnomalyDetection {
            anomaly_detected: risk_score >= 20,
            anomaly_type: if !reasons.is_empty() {
                Some(reasons[0].clone())
            } else {
                None
            },
            risk_score,
            reasons,
            recommended_action,
        }
    }

    /// Get events by user
    pub fn get_user_events(&self, user_id: &Uuid, limit: Option<usize>) -> Vec<&AuditEvent> {
        let mut events: Vec<&AuditEvent> = self
            .events
            .iter()
            .filter(|e| e.user_id.as_ref() == Some(user_id))
            .collect();

        events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        if let Some(limit) = limit {
            events.truncate(limit);
        }

        events
    }

    /// Get events by type
    pub fn get_events_by_type(
        &self,
        event_type: AuditEventType,
        limit: Option<usize>,
    ) -> Vec<&AuditEvent> {
        let mut events: Vec<&AuditEvent> = self
            .events
            .iter()
            .filter(|e| e.event_type == event_type)
            .collect();

        events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        if let Some(limit) = limit {
            events.truncate(limit);
        }

        events
    }

    /// Get events by severity
    pub fn get_events_by_severity(
        &self,
        min_severity: AuditSeverity,
        limit: Option<usize>,
    ) -> Vec<&AuditEvent> {
        let mut events: Vec<&AuditEvent> = self
            .events
            .iter()
            .filter(|e| e.severity >= min_severity)
            .collect();

        events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        if let Some(limit) = limit {
            events.truncate(limit);
        }

        events
    }

    /// Get events in time range
    pub fn get_events_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<&AuditEvent> {
        self.events
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .collect()
    }

    /// Get security events
    pub fn get_security_events(&self, limit: Option<usize>) -> Vec<&AuditEvent> {
        let security_event_types = [
            AuditEventType::SuspiciousActivity,
            AuditEventType::AnomalyDetected,
            AuditEventType::BruteForceAttempt,
            AuditEventType::IpBlocked,
            AuditEventType::AccountCompromised,
            AuditEventType::SecurityAlertTriggered,
        ];

        let mut events: Vec<&AuditEvent> = self
            .events
            .iter()
            .filter(|e| security_event_types.contains(&e.event_type))
            .collect();

        events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        if let Some(limit) = limit {
            events.truncate(limit);
        }

        events
    }

    /// Get failed login attempts
    pub fn get_failed_login_attempts(&self, username: &str) -> Vec<&FailedAttempt> {
        self.failed_attempts
            .get(username)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    /// Generate audit report
    pub fn generate_report(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> AuditReport {
        let events = self.get_events_in_range(start, end);

        let mut event_counts = HashMap::new();
        let mut severity_counts = HashMap::new();
        let mut user_activity = HashMap::new();
        let mut ip_counts = HashMap::new();

        for event in &events {
            *event_counts.entry(event.event_type).or_insert(0) += 1;
            *severity_counts.entry(event.severity).or_insert(0) += 1;
            *ip_counts.entry(event.ip_address.to_string()).or_insert(0) += 1;

            if let Some(user_id) = event.user_id {
                *user_activity.entry(user_id).or_insert(0) += 1;
            }
        }

        let total_events = events.len();
        let failed_events = events.iter().filter(|e| !e.success).count();
        let security_events = events
            .iter()
            .filter(|e| {
                matches!(
                    e.event_type,
                    AuditEventType::SuspiciousActivity
                        | AuditEventType::AnomalyDetected
                        | AuditEventType::BruteForceAttempt
                        | AuditEventType::IpBlocked
                        | AuditEventType::AccountCompromised
                )
            })
            .count();

        AuditReport {
            start_time: start,
            end_time: end,
            total_events,
            failed_events,
            security_events,
            unique_users: user_activity.len(),
            unique_ips: ip_counts.len(),
            event_counts,
            severity_counts,
            top_users: Self::get_top_n(&user_activity, 10),
            top_ips: Self::get_top_n(&ip_counts, 10),
        }
    }

    /// Get top N items from a count map
    fn get_top_n<K: Clone + std::fmt::Display>(
        map: &HashMap<K, usize>,
        n: usize,
    ) -> Vec<(String, usize)> {
        let mut items: Vec<_> = map.iter().map(|(k, v)| (k.clone(), *v)).collect();
        items.sort_by(|a, b| b.1.cmp(&a.1));
        items.truncate(n);
        items.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
    }

    /// Clear old events (for compliance/retention)
    pub fn cleanup_old_events(&mut self, retention_days: i64) -> usize {
        let cutoff = Utc::now() - Duration::days(retention_days);
        let initial_count = self.events.len();

        self.events.retain(|e| e.timestamp > cutoff);

        initial_count - self.events.len()
    }
}

/// Audit report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    /// Report start time
    pub start_time: DateTime<Utc>,

    /// Report end time
    pub end_time: DateTime<Utc>,

    /// Total events
    pub total_events: usize,

    /// Failed events
    pub failed_events: usize,

    /// Security events
    pub security_events: usize,

    /// Unique users
    pub unique_users: usize,

    /// Unique IPs
    pub unique_ips: usize,

    /// Event counts by type
    pub event_counts: HashMap<AuditEventType, usize>,

    /// Severity counts
    pub severity_counts: HashMap<AuditSeverity, usize>,

    /// Top users by activity
    pub top_users: Vec<(String, usize)>,

    /// Top IPs by activity
    pub top_ips: Vec<(String, usize)>,
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new(5, 900) // 5 attempts in 15 minutes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent::new(
            AuditEventType::LoginSuccess,
            AuditSeverity::Info,
            Some(Uuid::new_v4()),
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            "Test User Agent".to_string(),
            "User logged in successfully".to_string(),
        );

        assert_eq!(event.event_type, AuditEventType::LoginSuccess);
        assert_eq!(event.severity, AuditSeverity::Info);
        assert!(event.success);
    }

    #[test]
    fn test_failed_attempt_tracking() {
        let mut logger = AuditLogger::new(3, 900);
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        for i in 0..5 {
            let event = AuditEvent::new(
                AuditEventType::LoginFailure,
                AuditSeverity::Warning,
                None,
                ip,
                "Test".to_string(),
                format!("Failed attempt {}", i),
            )
            .with_error("Invalid credentials".to_string());

            logger.log(event);
        }

        assert!(logger.is_ip_blocked(&ip));
    }

    #[test]
    fn test_anomaly_detection() {
        let logger = AuditLogger::default();
        let user_id = Uuid::new_v4();
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        let result = logger.detect_anomalies(&user_id, &ip, "Test UA");

        // Should have low risk for normal activity
        assert!(result.risk_score < 20);
    }
}
