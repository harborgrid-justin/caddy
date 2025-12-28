//! Event classification for compliance and security monitoring
//!
//! This module provides event categorization, severity levels, and
//! classification rules for compliance frameworks.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Security event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecurityEventType {
    /// Successful authentication
    AuthSuccess,
    /// Failed authentication attempt
    AuthFailure,
    /// Account locked due to too many failures
    AccountLocked,
    /// Password changed
    PasswordChanged,
    /// Multi-factor authentication enabled/disabled
    MfaChanged,
    /// Privilege escalation attempt
    PrivilegeEscalation,
    /// Unauthorized access attempt
    UnauthorizedAccess,
    /// Suspicious activity detected
    SuspiciousActivity,
    /// Security policy violation
    PolicyViolation,
    /// Encryption key accessed
    KeyAccess,
    /// Security configuration changed
    SecurityConfigChanged,
}

/// Data access event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataAccessEventType {
    /// Data read/viewed
    Read,
    /// Data created
    Create,
    /// Data updated
    Update,
    /// Data deleted
    Delete,
    /// Data exported
    Export,
    /// Data imported
    Import,
    /// Data shared
    Share,
    /// Data downloaded
    Download,
    /// Data copied
    Copy,
    /// Data printed
    Print,
    /// Bulk data access
    BulkAccess,
}

/// Configuration change event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfigEventType {
    /// System configuration changed
    SystemConfig,
    /// User settings changed
    UserSettings,
    /// Permission/role changed
    PermissionChange,
    /// Feature enabled/disabled
    FeatureToggle,
    /// Integration configured
    IntegrationConfig,
    /// Backup settings changed
    BackupConfig,
    /// Retention policy changed
    RetentionConfig,
    /// Audit settings changed
    AuditConfig,
}

/// Authentication event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuthEventType {
    /// User login
    Login,
    /// User logout
    Logout,
    /// Session created
    SessionCreated,
    /// Session expired
    SessionExpired,
    /// Session terminated
    SessionTerminated,
    /// Token issued
    TokenIssued,
    /// Token revoked
    TokenRevoked,
    /// Password reset requested
    PasswordResetRequested,
    /// Password reset completed
    PasswordResetCompleted,
}

/// Event severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EventSeverity {
    /// Debug/trace level
    Debug,
    /// Informational event
    Info,
    /// Notice - normal but significant
    Notice,
    /// Warning - may require attention
    Warning,
    /// Error - error condition
    Error,
    /// Critical - critical condition
    Critical,
    /// Alert - action must be taken immediately
    Alert,
    /// Emergency - system unusable
    Emergency,
}

impl EventSeverity {
    /// Check if this severity level requires immediate attention
    pub fn requires_immediate_attention(&self) -> bool {
        matches!(self, EventSeverity::Critical | EventSeverity::Alert | EventSeverity::Emergency)
    }

    /// Check if this severity should trigger notifications
    pub fn should_notify(&self) -> bool {
        *self >= EventSeverity::Warning
    }
}

/// Classified event with full categorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifiedEvent {
    /// Unique event ID
    pub id: Uuid,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Event category
    pub category: EventCategory,

    /// Event severity
    pub severity: EventSeverity,

    /// Actor who performed the action
    pub actor: String,

    /// Resource affected
    pub resource: String,

    /// Action description
    pub action: String,

    /// Additional context
    pub context: HashMap<String, String>,

    /// Source IP address
    pub source_ip: Option<String>,

    /// User agent
    pub user_agent: Option<String>,

    /// Session ID
    pub session_id: Option<String>,

    /// Whether action succeeded
    pub success: bool,

    /// Error message if failed
    pub error_message: Option<String>,

    /// Tags for filtering/searching
    pub tags: Vec<String>,

    /// Compliance frameworks this event relates to
    pub compliance_frameworks: Vec<ComplianceFramework>,
}

/// Event category (high-level classification)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventCategory {
    /// Security-related event
    Security(SecurityEventType),
    /// Data access event
    DataAccess(DataAccessEventType),
    /// Configuration change
    Configuration(ConfigEventType),
    /// Authentication event
    Authentication(AuthEventType),
    /// System event
    System,
    /// Application event
    Application,
    /// Custom event
    Custom(String),
}

/// Compliance frameworks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComplianceFramework {
    /// SOC 2
    SOC2,
    /// GDPR
    GDPR,
    /// HIPAA
    HIPAA,
    /// PCI DSS
    PCIDSS,
    /// ISO 27001
    ISO27001,
    /// NIST
    NIST,
}

/// Event classifier for automatic categorization
pub struct EventClassifier {
    /// Classification rules
    rules: Vec<ClassificationRule>,
}

impl EventClassifier {
    /// Create new classifier with default rules
    pub fn new() -> Self {
        let mut classifier = Self { rules: Vec::new() };
        classifier.add_default_rules();
        classifier
    }

    /// Add a classification rule
    pub fn add_rule(&mut self, rule: ClassificationRule) {
        self.rules.push(rule);
    }

    /// Classify an event
    pub fn classify(&self, event: &mut ClassifiedEvent) {
        for rule in &self.rules {
            if rule.matches(event) {
                rule.apply(event);
            }
        }
    }

    /// Add default classification rules
    fn add_default_rules(&mut self) {
        // Authentication failures are warnings
        self.add_rule(ClassificationRule {
            condition: Box::new(|e| {
                matches!(e.category, EventCategory::Authentication(_)) && !e.success
            }),
            action: Box::new(|e| {
                e.severity = EventSeverity::Warning;
                e.tags.push("auth_failure".to_string());
            }),
        });

        // Multiple auth failures are critical
        self.add_rule(ClassificationRule {
            condition: Box::new(|e| {
                e.action.contains("locked") || e.action.contains("blocked")
            }),
            action: Box::new(|e| {
                e.severity = EventSeverity::Critical;
                e.tags.push("account_locked".to_string());
            }),
        });

        // Privilege escalation attempts are critical
        self.add_rule(ClassificationRule {
            condition: Box::new(|e| {
                matches!(
                    e.category,
                    EventCategory::Security(SecurityEventType::PrivilegeEscalation)
                )
            }),
            action: Box::new(|e| {
                e.severity = EventSeverity::Critical;
                e.tags.push("privilege_escalation".to_string());
                e.compliance_frameworks.push(ComplianceFramework::SOC2);
            }),
        });

        // Unauthorized access is critical
        self.add_rule(ClassificationRule {
            condition: Box::new(|e| {
                matches!(
                    e.category,
                    EventCategory::Security(SecurityEventType::UnauthorizedAccess)
                )
            }),
            action: Box::new(|e| {
                e.severity = EventSeverity::Critical;
                e.tags.push("unauthorized_access".to_string());
            }),
        });

        // Data exports require GDPR tracking
        self.add_rule(ClassificationRule {
            condition: Box::new(|e| {
                matches!(
                    e.category,
                    EventCategory::DataAccess(DataAccessEventType::Export)
                )
            }),
            action: Box::new(|e| {
                e.compliance_frameworks.push(ComplianceFramework::GDPR);
                e.tags.push("data_export".to_string());
            }),
        });

        // Bulk access requires heightened scrutiny
        self.add_rule(ClassificationRule {
            condition: Box::new(|e| {
                matches!(
                    e.category,
                    EventCategory::DataAccess(DataAccessEventType::BulkAccess)
                )
            }),
            action: Box::new(|e| {
                e.severity = EventSeverity::Notice;
                e.tags.push("bulk_access".to_string());
                e.compliance_frameworks.push(ComplianceFramework::SOC2);
            }),
        });

        // Configuration changes are notices
        self.add_rule(ClassificationRule {
            condition: Box::new(|e| matches!(e.category, EventCategory::Configuration(_))),
            action: Box::new(|e| {
                e.severity = EventSeverity::Notice;
                e.tags.push("config_change".to_string());
                e.compliance_frameworks.push(ComplianceFramework::SOC2);
            }),
        });

        // Security config changes are warnings
        self.add_rule(ClassificationRule {
            condition: Box::new(|e| {
                matches!(
                    e.category,
                    EventCategory::Security(SecurityEventType::SecurityConfigChanged)
                )
            }),
            action: Box::new(|e| {
                e.severity = EventSeverity::Warning;
                e.tags.push("security_config_change".to_string());
                e.compliance_frameworks.push(ComplianceFramework::SOC2);
                e.compliance_frameworks.push(ComplianceFramework::ISO27001);
            }),
        });
    }
}

impl Default for EventClassifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Classification rule
pub struct ClassificationRule {
    /// Condition to check
    condition: Box<dyn Fn(&ClassifiedEvent) -> bool + Send + Sync>,
    /// Action to apply if condition matches
    action: Box<dyn Fn(&mut ClassifiedEvent) + Send + Sync>,
}

impl ClassificationRule {
    /// Check if rule matches event
    pub fn matches(&self, event: &ClassifiedEvent) -> bool {
        (self.condition)(event)
    }

    /// Apply rule to event
    pub fn apply(&self, event: &mut ClassifiedEvent) {
        (self.action)(event);
    }
}

/// Builder for classified events
pub struct ClassifiedEventBuilder {
    event: ClassifiedEvent,
}

impl ClassifiedEventBuilder {
    /// Create new builder
    pub fn new(category: EventCategory) -> Self {
        Self {
            event: ClassifiedEvent {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                category,
                severity: EventSeverity::Info,
                actor: String::new(),
                resource: String::new(),
                action: String::new(),
                context: HashMap::new(),
                source_ip: None,
                user_agent: None,
                session_id: None,
                success: true,
                error_message: None,
                tags: Vec::new(),
                compliance_frameworks: Vec::new(),
            },
        }
    }

    /// Set actor
    pub fn actor(mut self, actor: impl Into<String>) -> Self {
        self.event.actor = actor.into();
        self
    }

    /// Set resource
    pub fn resource(mut self, resource: impl Into<String>) -> Self {
        self.event.resource = resource.into();
        self
    }

    /// Set action
    pub fn action(mut self, action: impl Into<String>) -> Self {
        self.event.action = action.into();
        self
    }

    /// Set severity
    pub fn severity(mut self, severity: EventSeverity) -> Self {
        self.event.severity = severity;
        self
    }

    /// Set success status
    pub fn success(mut self, success: bool) -> Self {
        self.event.success = success;
        self
    }

    /// Set error message
    pub fn error(mut self, error: impl Into<String>) -> Self {
        self.event.error_message = Some(error.into());
        self.event.success = false;
        self
    }

    /// Add context
    pub fn context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.event.context.insert(key.into(), value.into());
        self
    }

    /// Set source IP
    pub fn source_ip(mut self, ip: impl Into<String>) -> Self {
        self.event.source_ip = Some(ip.into());
        self
    }

    /// Set session ID
    pub fn session_id(mut self, session_id: impl Into<String>) -> Self {
        self.event.session_id = Some(session_id.into());
        self
    }

    /// Add tag
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.event.tags.push(tag.into());
        self
    }

    /// Add compliance framework
    pub fn compliance(mut self, framework: ComplianceFramework) -> Self {
        self.event.compliance_frameworks.push(framework);
        self
    }

    /// Build the event
    pub fn build(self) -> ClassifiedEvent {
        self.event
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_severity_ordering() {
        assert!(EventSeverity::Critical > EventSeverity::Warning);
        assert!(EventSeverity::Emergency > EventSeverity::Critical);
        assert!(EventSeverity::Info < EventSeverity::Warning);
    }

    #[test]
    fn test_severity_requires_attention() {
        assert!(EventSeverity::Critical.requires_immediate_attention());
        assert!(EventSeverity::Emergency.requires_immediate_attention());
        assert!(!EventSeverity::Warning.requires_immediate_attention());
    }

    #[test]
    fn test_event_builder() {
        let event = ClassifiedEventBuilder::new(EventCategory::Security(
            SecurityEventType::AuthFailure,
        ))
        .actor("user1")
        .resource("system")
        .action("login_failed")
        .severity(EventSeverity::Warning)
        .success(false)
        .tag("security")
        .compliance(ComplianceFramework::SOC2)
        .build();

        assert_eq!(event.actor, "user1");
        assert_eq!(event.severity, EventSeverity::Warning);
        assert!(!event.success);
        assert!(event.tags.contains(&"security".to_string()));
    }

    #[test]
    fn test_classifier_auth_failure() {
        let classifier = EventClassifier::new();

        let mut event = ClassifiedEventBuilder::new(EventCategory::Authentication(
            AuthEventType::Login,
        ))
        .actor("user1")
        .action("login")
        .success(false)
        .build();

        classifier.classify(&mut event);

        assert_eq!(event.severity, EventSeverity::Warning);
        assert!(event.tags.contains(&"auth_failure".to_string()));
    }

    #[test]
    fn test_classifier_privilege_escalation() {
        let classifier = EventClassifier::new();

        let mut event = ClassifiedEventBuilder::new(EventCategory::Security(
            SecurityEventType::PrivilegeEscalation,
        ))
        .actor("user1")
        .action("escalate")
        .build();

        classifier.classify(&mut event);

        assert_eq!(event.severity, EventSeverity::Critical);
        assert!(event
            .compliance_frameworks
            .contains(&ComplianceFramework::SOC2));
    }

    #[test]
    fn test_classifier_data_export() {
        let classifier = EventClassifier::new();

        let mut event = ClassifiedEventBuilder::new(EventCategory::DataAccess(
            DataAccessEventType::Export,
        ))
        .actor("user1")
        .action("export_data")
        .build();

        classifier.classify(&mut event);

        assert!(event
            .compliance_frameworks
            .contains(&ComplianceFramework::GDPR));
    }
}
