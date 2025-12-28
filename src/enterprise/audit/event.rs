//! Audit event types and structures
//!
//! This module defines the core audit event types used throughout the system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Type of audit event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    /// Resource creation
    Create,
    /// Resource read/access
    Read,
    /// Resource update/modification
    Update,
    /// Resource deletion
    Delete,
    /// User login
    Login,
    /// User logout
    Logout,
    /// Data export
    Export,
    /// Data import
    Import,
    /// Resource sharing
    Share,
    /// Permission change
    PermissionChange,
    /// Configuration change
    ConfigChange,
    /// System event
    System,
    /// Custom event type
    Custom(u32),
}

/// Severity level of audit event
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EventSeverity {
    /// Informational event
    Info,
    /// Warning level event
    Warning,
    /// Critical event requiring attention
    Critical,
    /// Security-related event
    Security,
}

/// Complete audit event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event identifier
    pub event_id: Uuid,

    /// Timestamp when event occurred
    pub timestamp: DateTime<Utc>,

    /// User ID who triggered the event
    pub user_id: String,

    /// Type of action performed
    pub action: EventType,

    /// Resource affected by the action
    pub resource: String,

    /// Resource type (e.g., "drawing", "layer", "user")
    pub resource_type: Option<String>,

    /// Additional event details
    pub details: HashMap<String, String>,

    /// IP address of the user
    pub ip_address: Option<String>,

    /// Session identifier
    pub session_id: Option<String>,

    /// Event severity
    pub severity: EventSeverity,

    /// Whether the action was successful
    pub success: bool,

    /// Error message if action failed
    pub error_message: Option<String>,

    /// Previous state (for update operations)
    pub previous_state: Option<String>,

    /// New state (for update operations)
    pub new_state: Option<String>,

    /// Correlation ID for related events
    pub correlation_id: Option<Uuid>,

    /// Hash of the event for tamper detection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,

    /// Previous event hash for chain verification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_hash: Option<String>,
}

impl AuditEvent {
    /// Create a new audit event builder
    pub fn builder() -> AuditEventBuilder {
        AuditEventBuilder::new()
    }

    /// Calculate hash of this event for tamper detection
    pub fn calculate_hash(&self, previous_hash: Option<&str>) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Hash core event data
        self.event_id.hash(&mut hasher);
        self.timestamp.to_rfc3339().hash(&mut hasher);
        self.user_id.hash(&mut hasher);
        self.resource.hash(&mut hasher);

        if let Some(prev) = previous_hash {
            prev.hash(&mut hasher);
        }

        format!("{:x}", hasher.finish())
    }

    /// Verify the hash of this event
    pub fn verify_hash(&self) -> bool {
        if let Some(ref stored_hash) = self.hash {
            let calculated = self.calculate_hash(self.previous_hash.as_deref());
            &calculated == stored_hash
        } else {
            false
        }
    }
}

/// Builder for creating audit events
pub struct AuditEventBuilder {
    event: AuditEvent,
}

impl AuditEventBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            event: AuditEvent {
                event_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                user_id: String::new(),
                action: EventType::Read,
                resource: String::new(),
                resource_type: None,
                details: HashMap::new(),
                ip_address: None,
                session_id: None,
                severity: EventSeverity::Info,
                success: true,
                error_message: None,
                previous_state: None,
                new_state: None,
                correlation_id: None,
                hash: None,
                previous_hash: None,
            },
        }
    }

    /// Set the user ID
    pub fn user_id(mut self, user_id: impl Into<String>) -> Self {
        self.event.user_id = user_id.into();
        self
    }

    /// Set the action type
    pub fn action(mut self, action: EventType) -> Self {
        self.event.action = action;
        self
    }

    /// Set the resource
    pub fn resource(mut self, resource: impl Into<String>) -> Self {
        self.event.resource = resource.into();
        self
    }

    /// Set the resource type
    pub fn resource_type(mut self, resource_type: impl Into<String>) -> Self {
        self.event.resource_type = Some(resource_type.into());
        self
    }

    /// Add a detail key-value pair
    pub fn detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.event.details.insert(key.into(), value.into());
        self
    }

    /// Set IP address
    pub fn ip_address(mut self, ip: impl Into<String>) -> Self {
        self.event.ip_address = Some(ip.into());
        self
    }

    /// Set session ID
    pub fn session_id(mut self, session_id: impl Into<String>) -> Self {
        self.event.session_id = Some(session_id.into());
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

    /// Set previous state
    pub fn previous_state(mut self, state: impl Into<String>) -> Self {
        self.event.previous_state = Some(state.into());
        self
    }

    /// Set new state
    pub fn new_state(mut self, state: impl Into<String>) -> Self {
        self.event.new_state = Some(state.into());
        self
    }

    /// Set correlation ID
    pub fn correlation_id(mut self, id: Uuid) -> Self {
        self.event.correlation_id = Some(id);
        self
    }

    /// Build the audit event
    pub fn build(self) -> AuditEvent {
        self.event
    }

    /// Build with hash calculation
    pub fn build_with_hash(mut self, previous_hash: Option<&str>) -> AuditEvent {
        let hash = self.event.calculate_hash(previous_hash);
        self.event.hash = Some(hash);
        if let Some(prev) = previous_hash {
            self.event.previous_hash = Some(prev.to_string());
        }
        self.event
    }
}

impl Default for AuditEventBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_builder() {
        let event = AuditEvent::builder()
            .user_id("user123")
            .action(EventType::Create)
            .resource("drawing/123")
            .resource_type("drawing")
            .detail("name", "Floor Plan")
            .severity(EventSeverity::Info)
            .build();

        assert_eq!(event.user_id, "user123");
        assert_eq!(event.action, EventType::Create);
        assert_eq!(event.resource, "drawing/123");
        assert_eq!(event.details.get("name"), Some(&"Floor Plan".to_string()));
    }

    #[test]
    fn test_event_hash() {
        let event = AuditEvent::builder()
            .user_id("user123")
            .action(EventType::Update)
            .resource("layer/456")
            .build_with_hash(None);

        assert!(event.hash.is_some());
        assert!(event.verify_hash());
    }

    #[test]
    fn test_event_chain() {
        let event1 = AuditEvent::builder()
            .user_id("user123")
            .action(EventType::Create)
            .resource("drawing/1")
            .build_with_hash(None);

        let hash1 = event1.hash.clone().unwrap();

        let event2 = AuditEvent::builder()
            .user_id("user123")
            .action(EventType::Update)
            .resource("drawing/1")
            .build_with_hash(Some(&hash1));

        assert_eq!(event2.previous_hash, Some(hash1));
        assert!(event2.verify_hash());
    }
}
