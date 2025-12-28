use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum StepError {
    #[error("Step execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Step validation failed: {0}")]
    ValidationFailed(String),
    #[error("Step configuration invalid: {0}")]
    InvalidConfiguration(String),
    #[error("Step timeout after {0} seconds")]
    Timeout(u64),
    #[error("Required field missing: {0}")]
    MissingField(String),
    #[error("Step precondition not met: {0}")]
    PreconditionFailed(String),
}

/// Result type for step execution
pub type StepResult<T> = Result<T, StepError>;

/// Step execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepStatus {
    /// Not yet started
    Pending,
    /// Currently executing
    Running,
    /// Completed successfully
    Completed,
    /// Failed with error
    Failed,
    /// Skipped due to conditions
    Skipped,
    /// Waiting for external input (e.g., approval)
    Waiting,
}

/// Execution context passed to steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepExecutionContext {
    /// Workflow ID
    pub workflow_id: Uuid,
    /// Step ID
    pub step_id: Uuid,
    /// User who initiated the workflow
    pub initiator_id: Uuid,
    /// Current workflow variables
    pub variables: HashMap<String, serde_json::Value>,
    /// Output from previous steps
    pub previous_outputs: HashMap<String, serde_json::Value>,
    /// Entity being operated on
    pub entity_id: Option<Uuid>,
    /// Entity type
    pub entity_type: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Execution start time
    pub started_at: DateTime<Utc>,
}

impl StepExecutionContext {
    /// Create a new execution context
    pub fn new(workflow_id: Uuid, step_id: Uuid, initiator_id: Uuid) -> Self {
        Self {
            workflow_id,
            step_id,
            initiator_id,
            variables: HashMap::new(),
            previous_outputs: HashMap::new(),
            entity_id: None,
            entity_type: None,
            metadata: HashMap::new(),
            started_at: Utc::now(),
        }
    }

    /// Get a variable value
    pub fn get_variable(&self, key: &str) -> Option<&serde_json::Value> {
        self.variables.get(key)
    }

    /// Set a variable value
    pub fn set_variable(&mut self, key: String, value: serde_json::Value) {
        self.variables.insert(key, value);
    }

    /// Get output from a previous step
    pub fn get_previous_output(&self, step_name: &str) -> Option<&serde_json::Value> {
        self.previous_outputs.get(step_name)
    }
}

/// Step execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepExecutionResult {
    /// Execution status
    pub status: StepStatus,
    /// Output data from the step
    pub output: serde_json::Value,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Execution duration
    pub duration_ms: u64,
    /// Log messages
    pub logs: Vec<String>,
}

impl StepExecutionResult {
    /// Create a successful result
    pub fn success(output: serde_json::Value, duration_ms: u64) -> Self {
        Self {
            status: StepStatus::Completed,
            output,
            error_message: None,
            duration_ms,
            logs: Vec::new(),
        }
    }

    /// Create a failed result
    pub fn failure(error: String, duration_ms: u64) -> Self {
        Self {
            status: StepStatus::Failed,
            output: serde_json::Value::Null,
            error_message: Some(error),
            duration_ms,
            logs: Vec::new(),
        }
    }

    /// Create a skipped result
    pub fn skipped(reason: String) -> Self {
        Self {
            status: StepStatus::Skipped,
            output: serde_json::Value::Null,
            error_message: Some(reason),
            duration_ms: 0,
            logs: Vec::new(),
        }
    }
}

/// Trait for workflow steps
#[async_trait]
pub trait Step: Send + Sync {
    /// Get the step type identifier
    fn step_type(&self) -> &str;

    /// Validate the step configuration
    fn validate(&self) -> StepResult<()>;

    /// Execute the step
    async fn execute(&self, context: &mut StepExecutionContext) -> StepResult<StepExecutionResult>;

    /// Check if step should be executed based on conditions
    async fn should_execute(&self, context: &StepExecutionContext) -> bool {
        true
    }

    /// Get estimated execution time in seconds
    fn estimated_duration(&self) -> u64 {
        60 // Default 1 minute
    }

    /// Whether this step can be retried on failure
    fn is_retryable(&self) -> bool {
        true
    }

    /// Maximum number of retry attempts
    fn max_retries(&self) -> u32 {
        3
    }
}

/// Step configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepConfig {
    /// Unique identifier
    pub id: Uuid,
    /// Step name (unique within workflow)
    pub name: String,
    /// Step type (approval, notification, transform, etc.)
    pub step_type: StepType,
    /// Step description
    pub description: String,
    /// Step-specific configuration
    pub config: HashMap<String, serde_json::Value>,
    /// Condition for executing this step (optional)
    pub condition: Option<StepCondition>,
    /// Whether to continue on failure
    pub continue_on_failure: bool,
    /// Timeout in seconds
    pub timeout_seconds: u64,
    /// Retry configuration
    pub retry_config: RetryConfig,
    /// Dependencies (steps that must complete before this)
    pub dependencies: Vec<String>,
}

impl StepConfig {
    /// Create a new step configuration
    pub fn new(name: String, step_type: StepType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            step_type,
            description: String::new(),
            config: HashMap::new(),
            condition: None,
            continue_on_failure: false,
            timeout_seconds: 300, // 5 minutes default
            retry_config: RetryConfig::default(),
            dependencies: Vec::new(),
        }
    }

    /// Set a configuration value
    pub fn set_config(&mut self, key: String, value: serde_json::Value) {
        self.config.insert(key, value);
    }

    /// Get a configuration value
    pub fn get_config(&self, key: &str) -> Option<&serde_json::Value> {
        self.config.get(key)
    }
}

/// Step type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StepType {
    /// Approval step
    Approval,
    /// Send notification
    Notification,
    /// Data transformation
    Transform,
    /// Export data/document
    Export,
    /// Validate data
    Validate,
    /// Execute custom script
    Script,
    /// HTTP API call
    HttpRequest,
    /// Database operation
    Database,
    /// Conditional branching
    Conditional,
    /// Parallel execution
    Parallel,
}

/// Condition for step execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepCondition {
    /// Condition expression (e.g., "variables.amount > 1000")
    pub expression: String,
    /// Variables to use in evaluation
    pub variables: Vec<String>,
}

impl StepCondition {
    /// Create a new condition
    pub fn new(expression: String) -> Self {
        Self {
            expression,
            variables: Vec::new(),
        }
    }

    /// Evaluate the condition (simplified - would use expression parser in production)
    pub fn evaluate(&self, context: &StepExecutionContext) -> bool {
        // Simple evaluation - in production, use a proper expression parser
        // For now, always return true
        true
    }
}

/// Retry configuration for steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Initial delay between retries in seconds
    pub initial_delay_seconds: u64,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
    /// Maximum delay between retries in seconds
    pub max_delay_seconds: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_seconds: 5,
            backoff_multiplier: 2.0,
            max_delay_seconds: 300,
        }
    }
}

impl RetryConfig {
    /// Calculate delay for a given attempt
    pub fn delay_for_attempt(&self, attempt: u32) -> u64 {
        let delay = (self.initial_delay_seconds as f64)
            * self.backoff_multiplier.powi(attempt as i32);
        delay.min(self.max_delay_seconds as f64) as u64
    }
}

// Built-in Step Implementations

/// Approval step implementation
#[derive(Debug, Clone)]
pub struct ApprovalStep {
    pub config: StepConfig,
}

#[async_trait]
impl Step for ApprovalStep {
    fn step_type(&self) -> &str {
        "approval"
    }

    fn validate(&self) -> StepResult<()> {
        if self.config.get_config("approvers").is_none() {
            return Err(StepError::ValidationFailed(
                "Approvers not configured".to_string(),
            ));
        }
        Ok(())
    }

    async fn execute(&self, context: &mut StepExecutionContext) -> StepResult<StepExecutionResult> {
        let start = std::time::Instant::now();

        // In production, this would create an approval request
        // For now, we'll simulate by checking if approval is already granted
        let approved = context
            .get_variable("approval_granted")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !approved {
            // Return waiting status - approval needed
            return Ok(StepExecutionResult {
                status: StepStatus::Waiting,
                output: serde_json::json!({
                    "status": "waiting_for_approval",
                    "approvers": self.config.get_config("approvers"),
                }),
                error_message: None,
                duration_ms: start.elapsed().as_millis() as u64,
                logs: vec!["Waiting for approval".to_string()],
            });
        }

        Ok(StepExecutionResult::success(
            serde_json::json!({"approved": true}),
            start.elapsed().as_millis() as u64,
        ))
    }

    fn is_retryable(&self) -> bool {
        false // Approval steps shouldn't auto-retry
    }
}

/// Notification step implementation
#[derive(Debug, Clone)]
pub struct NotificationStep {
    pub config: StepConfig,
}

#[async_trait]
impl Step for NotificationStep {
    fn step_type(&self) -> &str {
        "notification"
    }

    fn validate(&self) -> StepResult<()> {
        if self.config.get_config("recipients").is_none() {
            return Err(StepError::ValidationFailed(
                "Recipients not configured".to_string(),
            ));
        }
        if self.config.get_config("message").is_none() {
            return Err(StepError::ValidationFailed(
                "Message not configured".to_string(),
            ));
        }
        Ok(())
    }

    async fn execute(&self, context: &mut StepExecutionContext) -> StepResult<StepExecutionResult> {
        let start = std::time::Instant::now();

        // In production, would send actual notifications
        let recipients = self.config.get_config("recipients");
        let message = self.config.get_config("message");

        Ok(StepExecutionResult::success(
            serde_json::json!({
                "sent": true,
                "recipients": recipients,
                "message": message,
            }),
            start.elapsed().as_millis() as u64,
        ))
    }
}

/// Transform step for data transformation
#[derive(Debug, Clone)]
pub struct TransformStep {
    pub config: StepConfig,
}

#[async_trait]
impl Step for TransformStep {
    fn step_type(&self) -> &str {
        "transform"
    }

    fn validate(&self) -> StepResult<()> {
        if self.config.get_config("input").is_none() {
            return Err(StepError::ValidationFailed(
                "Input not configured".to_string(),
            ));
        }
        Ok(())
    }

    async fn execute(&self, context: &mut StepExecutionContext) -> StepResult<StepExecutionResult> {
        let start = std::time::Instant::now();

        // Get input data
        let input = self.config.get_config("input").cloned()
            .unwrap_or(serde_json::Value::Null);

        // Apply transformation (simplified)
        let output = input; // In production, would apply actual transformations

        Ok(StepExecutionResult::success(
            output,
            start.elapsed().as_millis() as u64,
        ))
    }
}

/// Export step for exporting data
#[derive(Debug, Clone)]
pub struct ExportStep {
    pub config: StepConfig,
}

#[async_trait]
impl Step for ExportStep {
    fn step_type(&self) -> &str {
        "export"
    }

    fn validate(&self) -> StepResult<()> {
        if self.config.get_config("format").is_none() {
            return Err(StepError::ValidationFailed(
                "Export format not configured".to_string(),
            ));
        }
        Ok(())
    }

    async fn execute(&self, context: &mut StepExecutionContext) -> StepResult<StepExecutionResult> {
        let start = std::time::Instant::now();

        let format = self.config.get_config("format");
        let destination = self.config.get_config("destination");

        Ok(StepExecutionResult::success(
            serde_json::json!({
                "exported": true,
                "format": format,
                "destination": destination,
            }),
            start.elapsed().as_millis() as u64,
        ))
    }
}

/// Validate step for data validation
#[derive(Debug, Clone)]
pub struct ValidateStep {
    pub config: StepConfig,
}

#[async_trait]
impl Step for ValidateStep {
    fn step_type(&self) -> &str {
        "validate"
    }

    fn validate(&self) -> StepResult<()> {
        if self.config.get_config("rules").is_none() {
            return Err(StepError::ValidationFailed(
                "Validation rules not configured".to_string(),
            ));
        }
        Ok(())
    }

    async fn execute(&self, context: &mut StepExecutionContext) -> StepResult<StepExecutionResult> {
        let start = std::time::Instant::now();

        // Get validation rules
        let rules = self.config.get_config("rules");

        // In production, would perform actual validation
        let is_valid = true;

        if !is_valid {
            return Ok(StepExecutionResult::failure(
                "Validation failed".to_string(),
                start.elapsed().as_millis() as u64,
            ));
        }

        Ok(StepExecutionResult::success(
            serde_json::json!({"valid": true, "rules": rules}),
            start.elapsed().as_millis() as u64,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_config_creation() {
        let config = StepConfig::new("Test Step".to_string(), StepType::Approval);
        assert_eq!(config.name, "Test Step");
        assert_eq!(config.step_type, StepType::Approval);
    }

    #[test]
    fn test_retry_config_delay() {
        let config = RetryConfig::default();
        assert_eq!(config.delay_for_attempt(0), 5);
        assert_eq!(config.delay_for_attempt(1), 10);
        assert_eq!(config.delay_for_attempt(2), 20);
    }

    #[tokio::test]
    async fn test_notification_step() {
        let mut config = StepConfig::new("Notify".to_string(), StepType::Notification);
        config.set_config("recipients".to_string(), serde_json::json!(["user1", "user2"]));
        config.set_config("message".to_string(), serde_json::json!("Test message"));

        let step = NotificationStep { config };
        assert!(step.validate().is_ok());

        let mut context = StepExecutionContext::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        );

        let result = step.execute(&mut context).await.unwrap();
        assert_eq!(result.status, StepStatus::Completed);
    }
}
