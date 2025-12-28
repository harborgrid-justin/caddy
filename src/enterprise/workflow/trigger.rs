use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum TriggerError {
    #[error("Trigger validation failed: {0}")]
    ValidationFailed(String),
    #[error("Trigger not found: {0}")]
    NotFound(Uuid),
    #[error("Invalid trigger configuration: {0}")]
    InvalidConfiguration(String),
    #[error("Trigger condition not met: {0}")]
    ConditionNotMet(String),
    #[error("Schedule parsing failed: {0}")]
    InvalidSchedule(String),
}

pub type TriggerResult<T> = Result<T, TriggerError>;

/// Trigger configuration for workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerConfig {
    /// Unique identifier
    pub id: Uuid,
    /// Trigger name
    pub name: String,
    /// Trigger type
    pub trigger_type: TriggerType,
    /// Whether this trigger is enabled
    pub enabled: bool,
    /// Trigger-specific configuration
    pub config: HashMap<String, serde_json::Value>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl TriggerConfig {
    /// Create a new trigger configuration
    pub fn new(name: String, trigger_type: TriggerType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            trigger_type,
            enabled: true,
            config: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Set a configuration value
    pub fn set_config(&mut self, key: String, value: serde_json::Value) {
        self.config.insert(key, value);
        self.updated_at = Utc::now();
    }

    /// Get a configuration value
    pub fn get_config(&self, key: &str) -> Option<&serde_json::Value> {
        self.config.get(key)
    }

    /// Validate the trigger configuration
    pub fn validate(&self) -> TriggerResult<()> {
        match &self.trigger_type {
            TriggerType::Manual => Ok(()),
            TriggerType::Scheduled { schedule } => {
                if schedule.is_empty() {
                    return Err(TriggerError::ValidationFailed(
                        "Schedule cannot be empty".to_string(),
                    ));
                }
                // In production, would validate cron expression
                Ok(())
            }
            TriggerType::EventBased { event_type } => {
                if event_type.is_empty() {
                    return Err(TriggerError::ValidationFailed(
                        "Event type cannot be empty".to_string(),
                    ));
                }
                Ok(())
            }
            TriggerType::ConditionBased { condition } => {
                if condition.is_empty() {
                    return Err(TriggerError::ValidationFailed(
                        "Condition cannot be empty".to_string(),
                    ));
                }
                Ok(())
            }
            TriggerType::Webhook { endpoint } => {
                if endpoint.is_empty() {
                    return Err(TriggerError::ValidationFailed(
                        "Webhook endpoint cannot be empty".to_string(),
                    ));
                }
                Ok(())
            }
            TriggerType::FileWatch { path, pattern } => {
                if path.is_empty() {
                    return Err(TriggerError::ValidationFailed(
                        "File watch path cannot be empty".to_string(),
                    ));
                }
                Ok(())
            }
        }
    }
}

/// Types of workflow triggers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TriggerType {
    /// Manually triggered by user
    Manual,

    /// Scheduled execution (cron-like)
    Scheduled {
        /// Cron expression or schedule definition
        schedule: String,
    },

    /// Triggered by system events
    EventBased {
        /// Event type to listen for
        event_type: String,
    },

    /// Triggered when condition is met
    ConditionBased {
        /// Condition expression
        condition: String,
    },

    /// Webhook trigger
    Webhook {
        /// Webhook endpoint URL
        endpoint: String,
    },

    /// File system watch trigger
    FileWatch {
        /// Directory path to watch
        path: String,
        /// File pattern to match
        pattern: String,
    },
}

/// Trigger execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerExecutionContext {
    /// Trigger ID
    pub trigger_id: Uuid,
    /// Workflow ID
    pub workflow_id: Uuid,
    /// Time when trigger fired
    pub fired_at: DateTime<Utc>,
    /// Event data that caused the trigger
    pub event_data: serde_json::Value,
    /// User who initiated (if applicable)
    pub initiator_id: Option<Uuid>,
    /// Additional context
    pub metadata: HashMap<String, String>,
}

impl TriggerExecutionContext {
    /// Create a new trigger execution context
    pub fn new(trigger_id: Uuid, workflow_id: Uuid) -> Self {
        Self {
            trigger_id,
            workflow_id,
            fired_at: Utc::now(),
            event_data: serde_json::Value::Null,
            initiator_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Create context for manual trigger
    pub fn manual(trigger_id: Uuid, workflow_id: Uuid, user_id: Uuid) -> Self {
        let mut context = Self::new(trigger_id, workflow_id);
        context.initiator_id = Some(user_id);
        context
    }

    /// Create context for event trigger
    pub fn event(trigger_id: Uuid, workflow_id: Uuid, event_data: serde_json::Value) -> Self {
        let mut context = Self::new(trigger_id, workflow_id);
        context.event_data = event_data;
        context
    }
}

/// Trigger condition for conditional execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerCondition {
    /// Condition expression
    pub expression: String,
    /// Variable references in the condition
    pub variables: Vec<String>,
    /// Comparison operator
    pub operator: ConditionOperator,
    /// Expected value
    pub expected_value: serde_json::Value,
}

impl TriggerCondition {
    /// Create a new trigger condition
    pub fn new(expression: String, operator: ConditionOperator, expected_value: serde_json::Value) -> Self {
        Self {
            expression,
            variables: Vec::new(),
            operator,
            expected_value,
        }
    }

    /// Evaluate the condition
    pub fn evaluate(&self, context: &TriggerExecutionContext) -> bool {
        // Simplified evaluation - in production would use expression parser
        // For now, always return true
        true
    }
}

/// Condition operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    Matches, // Regex match
}

/// Trigger registration for managing active triggers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerRegistration {
    /// Registration ID
    pub id: Uuid,
    /// Trigger configuration
    pub trigger: TriggerConfig,
    /// Workflow ID this trigger is for
    pub workflow_id: Uuid,
    /// Whether this registration is active
    pub is_active: bool,
    /// Last execution time
    pub last_executed_at: Option<DateTime<Utc>>,
    /// Next scheduled execution (for scheduled triggers)
    pub next_execution_at: Option<DateTime<Utc>>,
    /// Number of times triggered
    pub execution_count: u64,
    /// Number of successful executions
    pub success_count: u64,
    /// Number of failed executions
    pub failure_count: u64,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl TriggerRegistration {
    /// Create a new trigger registration
    pub fn new(trigger: TriggerConfig, workflow_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            trigger,
            workflow_id,
            is_active: true,
            last_executed_at: None,
            next_execution_at: None,
            execution_count: 0,
            success_count: 0,
            failure_count: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Record a successful execution
    pub fn record_success(&mut self) {
        self.execution_count += 1;
        self.success_count += 1;
        self.last_executed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Record a failed execution
    pub fn record_failure(&mut self) {
        self.execution_count += 1;
        self.failure_count += 1;
        self.last_executed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Activate the trigger
    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }

    /// Deactivate the trigger
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.execution_count == 0 {
            return 0.0;
        }
        (self.success_count as f64) / (self.execution_count as f64)
    }
}

/// Event that can trigger workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEvent {
    /// Event ID
    pub id: Uuid,
    /// Event type
    pub event_type: String,
    /// Event source
    pub source: String,
    /// Event data
    pub data: serde_json::Value,
    /// Entity ID related to event
    pub entity_id: Option<Uuid>,
    /// Entity type
    pub entity_type: Option<String>,
    /// User who caused the event
    pub user_id: Option<Uuid>,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl WorkflowEvent {
    /// Create a new workflow event
    pub fn new(event_type: String, source: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            source,
            data: serde_json::Value::Null,
            entity_id: None,
            entity_type: None,
            user_id: None,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Create event for entity change
    pub fn entity_changed(
        event_type: String,
        entity_id: Uuid,
        entity_type: String,
        user_id: Option<Uuid>,
    ) -> Self {
        let mut event = Self::new(event_type, "system".to_string());
        event.entity_id = Some(entity_id);
        event.entity_type = Some(entity_type);
        event.user_id = user_id;
        event
    }

    /// Set event data
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Manager for trigger registrations
pub struct TriggerRegistry {
    registrations: HashMap<Uuid, TriggerRegistration>,
    workflow_triggers: HashMap<Uuid, Vec<Uuid>>, // workflow_id -> trigger_ids
    event_triggers: HashMap<String, Vec<Uuid>>,  // event_type -> trigger_ids
}

impl TriggerRegistry {
    /// Create a new trigger registry
    pub fn new() -> Self {
        Self {
            registrations: HashMap::new(),
            workflow_triggers: HashMap::new(),
            event_triggers: HashMap::new(),
        }
    }

    /// Register a trigger
    pub fn register(&mut self, registration: TriggerRegistration) -> TriggerResult<()> {
        registration.trigger.validate()?;

        let workflow_id = registration.workflow_id;
        let registration_id = registration.id;

        // Index by workflow
        self.workflow_triggers
            .entry(workflow_id)
            .or_insert_with(Vec::new)
            .push(registration_id);

        // Index by event type for event-based triggers
        if let TriggerType::EventBased { event_type } = &registration.trigger.trigger_type {
            self.event_triggers
                .entry(event_type.clone())
                .or_insert_with(Vec::new)
                .push(registration_id);
        }

        self.registrations.insert(registration_id, registration);
        Ok(())
    }

    /// Unregister a trigger
    pub fn unregister(&mut self, registration_id: Uuid) -> TriggerResult<TriggerRegistration> {
        let registration = self.registrations
            .remove(&registration_id)
            .ok_or(TriggerError::NotFound(registration_id))?;

        // Remove from workflow index
        if let Some(triggers) = self.workflow_triggers.get_mut(&registration.workflow_id) {
            triggers.retain(|id| *id != registration_id);
        }

        // Remove from event index
        if let TriggerType::EventBased { event_type } = &registration.trigger.trigger_type {
            if let Some(triggers) = self.event_triggers.get_mut(event_type) {
                triggers.retain(|id| *id != registration_id);
            }
        }

        Ok(registration)
    }

    /// Get triggers for a workflow
    pub fn get_workflow_triggers(&self, workflow_id: Uuid) -> Vec<&TriggerRegistration> {
        self.workflow_triggers
            .get(&workflow_id)
            .map(|trigger_ids| {
                trigger_ids
                    .iter()
                    .filter_map(|id| self.registrations.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get triggers for an event type
    pub fn get_event_triggers(&self, event_type: &str) -> Vec<&TriggerRegistration> {
        self.event_triggers
            .get(event_type)
            .map(|trigger_ids| {
                trigger_ids
                    .iter()
                    .filter_map(|id| self.registrations.get(id))
                    .filter(|reg| reg.is_active && reg.trigger.enabled)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get a trigger registration
    pub fn get(&self, registration_id: Uuid) -> Option<&TriggerRegistration> {
        self.registrations.get(&registration_id)
    }

    /// Get a mutable trigger registration
    pub fn get_mut(&mut self, registration_id: Uuid) -> Option<&mut TriggerRegistration> {
        self.registrations.get_mut(&registration_id)
    }

    /// Get all active triggers
    pub fn get_active_triggers(&self) -> Vec<&TriggerRegistration> {
        self.registrations
            .values()
            .filter(|reg| reg.is_active && reg.trigger.enabled)
            .collect()
    }
}

impl Default for TriggerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_config_creation() {
        let trigger = TriggerConfig::new(
            "Test Trigger".to_string(),
            TriggerType::Manual,
        );
        assert_eq!(trigger.name, "Test Trigger");
        assert!(trigger.enabled);
    }

    #[test]
    fn test_scheduled_trigger_validation() {
        let trigger = TriggerConfig::new(
            "Scheduled".to_string(),
            TriggerType::Scheduled {
                schedule: "0 9 * * *".to_string(),
            },
        );
        assert!(trigger.validate().is_ok());
    }

    #[test]
    fn test_trigger_registry() {
        let mut registry = TriggerRegistry::new();
        let workflow_id = Uuid::new_v4();

        let trigger = TriggerConfig::new(
            "Test".to_string(),
            TriggerType::Manual,
        );
        let registration = TriggerRegistration::new(trigger, workflow_id);
        let registration_id = registration.id;

        assert!(registry.register(registration).is_ok());
        assert!(registry.get(registration_id).is_some());

        let workflow_triggers = registry.get_workflow_triggers(workflow_id);
        assert_eq!(workflow_triggers.len(), 1);
    }

    #[test]
    fn test_event_trigger_indexing() {
        let mut registry = TriggerRegistry::new();
        let workflow_id = Uuid::new_v4();

        let trigger = TriggerConfig::new(
            "Event Trigger".to_string(),
            TriggerType::EventBased {
                event_type: "drawing.updated".to_string(),
            },
        );
        let registration = TriggerRegistration::new(trigger, workflow_id);

        registry.register(registration).unwrap();

        let event_triggers = registry.get_event_triggers("drawing.updated");
        assert_eq!(event_triggers.len(), 1);
    }
}
