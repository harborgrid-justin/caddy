use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

use super::step::StepConfig;
use super::trigger::TriggerConfig;

#[derive(Debug, Error)]
pub enum WorkflowError {
    #[error("Workflow not found: {0}")]
    NotFound(Uuid),
    #[error("Invalid workflow state transition from {from:?} to {to:?}")]
    InvalidStateTransition {
        from: WorkflowStatus,
        to: WorkflowStatus,
    },
    #[error("Workflow validation failed: {0}")]
    ValidationFailed(String),
    #[error("Version conflict: expected {expected}, found {found}")]
    VersionConflict { expected: u32, found: u32 },
    #[error("Workflow is not editable in status {0:?}")]
    NotEditable(WorkflowStatus),
}

/// Status of a workflow instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    /// Draft state, can be edited
    Draft,
    /// Active and running
    Active,
    /// Temporarily paused
    Paused,
    /// Successfully completed
    Completed,
    /// Failed with errors
    Failed,
    /// Cancelled by user
    Cancelled,
}

impl WorkflowStatus {
    /// Check if transition to another status is valid
    pub fn can_transition_to(&self, target: &WorkflowStatus) -> bool {
        use WorkflowStatus::*;
        match (self, target) {
            // From Draft
            (Draft, Active) => true,
            (Draft, Cancelled) => true,
            // From Active
            (Active, Paused) => true,
            (Active, Completed) => true,
            (Active, Failed) => true,
            (Active, Cancelled) => true,
            // From Paused
            (Paused, Active) => true,
            (Paused, Cancelled) => true,
            (Paused, Failed) => true,
            // Terminal states cannot transition
            (Completed, _) => false,
            (Failed, _) => false,
            (Cancelled, _) => false,
            // Same state is always valid
            _ if self == target => true,
            // All other transitions are invalid
            _ => false,
        }
    }

    /// Check if workflow is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            WorkflowStatus::Completed | WorkflowStatus::Failed | WorkflowStatus::Cancelled
        )
    }

    /// Check if workflow can be edited
    pub fn is_editable(&self) -> bool {
        matches!(self, WorkflowStatus::Draft)
    }
}

/// Workflow execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContext {
    /// User who initiated the workflow
    pub initiator_id: Uuid,
    /// Variables available to the workflow
    pub variables: HashMap<String, serde_json::Value>,
    /// Entity this workflow is operating on (e.g., drawing ID)
    pub entity_id: Option<Uuid>,
    /// Entity type (e.g., "drawing", "document", "user")
    pub entity_type: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Default for WorkflowContext {
    fn default() -> Self {
        Self {
            initiator_id: Uuid::nil(),
            variables: HashMap::new(),
            entity_id: None,
            entity_type: None,
            metadata: HashMap::new(),
        }
    }
}

/// A workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// Unique identifier
    pub id: Uuid,
    /// Workflow name
    pub name: String,
    /// Description
    pub description: String,
    /// Current status
    pub status: WorkflowStatus,
    /// Ordered list of steps
    pub steps: Vec<StepConfig>,
    /// Trigger configuration
    pub trigger: Option<TriggerConfig>,
    /// Workflow template ID if created from template
    pub template_id: Option<Uuid>,
    /// Version number for optimistic locking
    pub version: u32,
    /// Execution context
    pub context: WorkflowContext,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// User who created this workflow
    pub created_by: Uuid,
    /// Start time (when workflow became active)
    pub started_at: Option<DateTime<Utc>>,
    /// Completion time
    pub completed_at: Option<DateTime<Utc>>,
    /// Current step index being executed
    pub current_step_index: Option<usize>,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Execution history
    pub execution_log: Vec<WorkflowExecutionEntry>,
}

impl Workflow {
    /// Create a new workflow
    pub fn new(name: String, description: String, created_by: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            status: WorkflowStatus::Draft,
            steps: Vec::new(),
            trigger: None,
            template_id: None,
            version: 1,
            context: WorkflowContext::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by,
            started_at: None,
            completed_at: None,
            current_step_index: None,
            error_message: None,
            execution_log: Vec::new(),
        }
    }

    /// Create workflow from template
    pub fn from_template(template: &WorkflowTemplate, created_by: Uuid) -> Self {
        let mut workflow = Self::new(template.name.clone(), template.description.clone(), created_by);
        workflow.template_id = Some(template.id);
        workflow.steps = template.steps.clone();
        workflow.trigger = template.default_trigger.clone();
        workflow
    }

    /// Add a step to the workflow
    pub fn add_step(&mut self, step: StepConfig) -> Result<(), WorkflowError> {
        if !self.status.is_editable() {
            return Err(WorkflowError::NotEditable(self.status));
        }
        self.steps.push(step);
        self.updated_at = Utc::now();
        self.version += 1;
        Ok(())
    }

    /// Remove a step by index
    pub fn remove_step(&mut self, index: usize) -> Result<StepConfig, WorkflowError> {
        if !self.status.is_editable() {
            return Err(WorkflowError::NotEditable(self.status));
        }
        if index >= self.steps.len() {
            return Err(WorkflowError::ValidationFailed(
                "Step index out of bounds".to_string(),
            ));
        }
        let step = self.steps.remove(index);
        self.updated_at = Utc::now();
        self.version += 1;
        Ok(step)
    }

    /// Update workflow status
    pub fn set_status(&mut self, new_status: WorkflowStatus) -> Result<(), WorkflowError> {
        if !self.status.can_transition_to(&new_status) {
            return Err(WorkflowError::InvalidStateTransition {
                from: self.status,
                to: new_status,
            });
        }

        let old_status = self.status;
        self.status = new_status;
        self.updated_at = Utc::now();

        // Update timestamps based on status
        match new_status {
            WorkflowStatus::Active if old_status == WorkflowStatus::Draft => {
                self.started_at = Some(Utc::now());
            }
            WorkflowStatus::Completed | WorkflowStatus::Failed | WorkflowStatus::Cancelled => {
                self.completed_at = Some(Utc::now());
            }
            _ => {}
        }

        self.log_execution(format!("Status changed from {:?} to {:?}", old_status, new_status));
        Ok(())
    }

    /// Validate workflow is ready to execute
    pub fn validate(&self) -> Result<(), WorkflowError> {
        if self.steps.is_empty() {
            return Err(WorkflowError::ValidationFailed(
                "Workflow must have at least one step".to_string(),
            ));
        }

        // Validate step names are unique
        let mut step_names = std::collections::HashSet::new();
        for step in &self.steps {
            if !step_names.insert(&step.name) {
                return Err(WorkflowError::ValidationFailed(format!(
                    "Duplicate step name: {}",
                    step.name
                )));
            }
        }

        Ok(())
    }

    /// Log an execution event
    pub fn log_execution(&mut self, message: String) {
        self.execution_log.push(WorkflowExecutionEntry {
            timestamp: Utc::now(),
            message,
            step_index: self.current_step_index,
        });
    }

    /// Get execution duration
    pub fn execution_duration(&self) -> Option<chrono::Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end - start),
            (Some(start), None) => Some(Utc::now() - start),
            _ => None,
        }
    }
}

/// Workflow execution log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecutionEntry {
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub step_index: Option<usize>,
}

/// Reusable workflow template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    /// Unique identifier
    pub id: Uuid,
    /// Template name
    pub name: String,
    /// Description
    pub description: String,
    /// Category for organization
    pub category: String,
    /// Template steps
    pub steps: Vec<StepConfig>,
    /// Default trigger (can be overridden)
    pub default_trigger: Option<TriggerConfig>,
    /// Tags for searchability
    pub tags: Vec<String>,
    /// Is this template active/available
    pub is_active: bool,
    /// Template version
    pub version: u32,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// User who created this template
    pub created_by: Uuid,
    /// Number of times this template has been used
    pub usage_count: u64,
}

impl WorkflowTemplate {
    /// Create a new template
    pub fn new(name: String, description: String, category: String, created_by: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            category,
            steps: Vec::new(),
            default_trigger: None,
            tags: Vec::new(),
            is_active: true,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by,
            usage_count: 0,
        }
    }

    /// Add a step to the template
    pub fn add_step(&mut self, step: StepConfig) {
        self.steps.push(step);
        self.updated_at = Utc::now();
        self.version += 1;
    }

    /// Increment usage counter
    pub fn increment_usage(&mut self) {
        self.usage_count += 1;
        self.updated_at = Utc::now();
    }
}

/// Workflow versioning for history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowVersion {
    /// Version number
    pub version: u32,
    /// Workflow ID
    pub workflow_id: Uuid,
    /// Snapshot of workflow at this version
    pub snapshot: Workflow,
    /// Change description
    pub change_description: String,
    /// User who made the change
    pub changed_by: Uuid,
    /// Timestamp of this version
    pub created_at: DateTime<Utc>,
}

impl WorkflowVersion {
    /// Create a new version from a workflow
    pub fn from_workflow(workflow: &Workflow, change_description: String, changed_by: Uuid) -> Self {
        Self {
            version: workflow.version,
            workflow_id: workflow.id,
            snapshot: workflow.clone(),
            change_description,
            changed_by,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_creation() {
        let user_id = Uuid::new_v4();
        let workflow = Workflow::new(
            "Test Workflow".to_string(),
            "A test workflow".to_string(),
            user_id,
        );

        assert_eq!(workflow.status, WorkflowStatus::Draft);
        assert_eq!(workflow.version, 1);
        assert_eq!(workflow.created_by, user_id);
    }

    #[test]
    fn test_status_transitions() {
        assert!(WorkflowStatus::Draft.can_transition_to(&WorkflowStatus::Active));
        assert!(WorkflowStatus::Active.can_transition_to(&WorkflowStatus::Paused));
        assert!(WorkflowStatus::Paused.can_transition_to(&WorkflowStatus::Active));
        assert!(!WorkflowStatus::Completed.can_transition_to(&WorkflowStatus::Active));
    }

    #[test]
    fn test_workflow_validation() {
        let user_id = Uuid::new_v4();
        let workflow = Workflow::new(
            "Test".to_string(),
            "Test".to_string(),
            user_id,
        );

        // Empty workflow should fail validation
        assert!(workflow.validate().is_err());
    }

    #[test]
    fn test_template_creation() {
        let user_id = Uuid::new_v4();
        let template = WorkflowTemplate::new(
            "Test Template".to_string(),
            "A test template".to_string(),
            "Testing".to_string(),
            user_id,
        );

        assert_eq!(template.version, 1);
        assert_eq!(template.usage_count, 0);
        assert!(template.is_active);
    }
}
