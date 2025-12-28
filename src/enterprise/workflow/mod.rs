//! Enterprise Workflow Automation System
//!
//! A comprehensive workflow automation system for CADDY enterprise edition.
//! Provides workflow definitions, execution engine, scheduling, approvals,
//! notifications, and more.
//!
//! # Features
//!
//! - **Workflow Management**: Create, version, and manage workflows
//! - **Flexible Steps**: Extensible step system with built-in steps
//! - **Triggers**: Manual, scheduled, event-based, and condition-based triggers
//! - **Execution Engine**: Async execution with parallelism and error handling
//! - **Approvals**: Multi-level approval workflows with delegation and escalation
//! - **Notifications**: Multi-channel notifications with templates
//! - **Scheduling**: Cron-like scheduling with recurring workflows
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Workflow Engine                          │
//! │  - Execution orchestration                                  │
//! │  - State management                                         │
//! │  - Error handling & retries                                 │
//! └─────────────────────────────────────────────────────────────┘
//!          │                │                │
//!          ▼                ▼                ▼
//! ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
//! │   Steps     │  │  Triggers   │  │  Scheduler  │
//! │  - Approval │  │  - Manual   │  │  - Cron     │
//! │  - Notify   │  │  - Event    │  │  - Interval │
//! │  - Transform│  │  - Schedule │  │  - One-time │
//! │  - Export   │  │  - Webhook  │  │             │
//! │  - Validate │  │             │  │             │
//! └─────────────┘  └─────────────┘  └─────────────┘
//!          │                │                │
//!          ▼                ▼                ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                Support Services                              │
//! │  - Approvals (multi-level, delegation, escalation)          │
//! │  - Notifications (email, in-app, webhook, SMS)              │
//! │  - State persistence                                         │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use caddy::enterprise::workflow::*;
//! use uuid::Uuid;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a workflow
//!     let user_id = Uuid::new_v4();
//!     let mut workflow = Workflow::new(
//!         "Drawing Approval".to_string(),
//!         "Approve engineering drawings".to_string(),
//!         user_id,
//!     );
//!
//!     // Add approval step
//!     let mut approval_step = StepConfig::new(
//!         "manager_approval".to_string(),
//!         StepType::Approval,
//!     );
//!     approval_step.set_config(
//!         "approvers".to_string(),
//!         serde_json::json!([manager_id]),
//!     );
//!     workflow.add_step(approval_step)?;
//!
//!     // Add notification step
//!     let mut notify_step = StepConfig::new(
//!         "notify_team".to_string(),
//!         StepType::Notification,
//!     );
//!     notify_step.set_config(
//!         "recipients".to_string(),
//!         serde_json::json!(["team@company.com"]),
//!     );
//!     notify_step.set_config(
//!         "message".to_string(),
//!         serde_json::json!("Drawing approved"),
//!     );
//!     workflow.add_step(notify_step)?;
//!
//!     // Create engine and execute
//!     let engine = WorkflowEngine::new();
//!     engine.create_workflow(workflow.clone()).await?;
//!     engine.start_workflow(workflow.id).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Workflow Lifecycle
//!
//! 1. **Draft**: Workflow is being designed
//! 2. **Active**: Workflow is executing
//! 3. **Paused**: Workflow is temporarily paused (e.g., waiting for approval)
//! 4. **Completed**: Workflow finished successfully
//! 5. **Failed**: Workflow encountered an error
//! 6. **Cancelled**: Workflow was cancelled by user
//!
//! # Step Types
//!
//! - **Approval**: Request approval from users
//! - **Notification**: Send notifications via various channels
//! - **Transform**: Transform data between steps
//! - **Export**: Export data or documents
//! - **Validate**: Validate data against rules
//! - **Script**: Execute custom scripts
//! - **HttpRequest**: Make HTTP API calls
//! - **Database**: Perform database operations
//!
//! # Trigger Types
//!
//! - **Manual**: User-initiated
//! - **Scheduled**: Time-based (cron-like)
//! - **EventBased**: System events
//! - **ConditionBased**: When conditions are met
//! - **Webhook**: HTTP webhooks
//! - **FileWatch**: File system changes

pub mod workflow;
pub mod step;
pub mod trigger;
pub mod engine;
pub mod approval;
pub mod notification;
pub mod scheduler;

// Re-export main types for convenience
pub use workflow::{
    Workflow, WorkflowStatus, WorkflowTemplate, WorkflowContext,
    WorkflowVersion, WorkflowError, WorkflowExecutionEntry,
};

pub use step::{
    Step, StepConfig, StepType, StepStatus, StepError,
    StepExecutionContext, StepExecutionResult, StepCondition,
    RetryConfig, ApprovalStep, NotificationStep, TransformStep,
    ExportStep, ValidateStep,
};

pub use trigger::{
    TriggerConfig, TriggerType, TriggerError, TriggerExecutionContext,
    TriggerCondition, TriggerRegistration, WorkflowEvent, TriggerRegistry,
    ConditionOperator,
};

pub use engine::{
    WorkflowEngine, EngineError, WorkflowPersistence, InMemoryPersistence,
};

pub use approval::{
    ApprovalRequest, ApprovalStatus, ApprovalLevel, ApprovalAction,
    ApprovalActionType, ApprovalPolicy, ApprovalError, PolicyCondition,
    ApprovalLevelTemplate, AutoEscalation,
};

pub use notification::{
    Notification, NotificationTemplate, NotificationChannel,
    NotificationPriority, NotificationStatus, NotificationRecipient,
    NotificationPreferences, DeliveryTracking, NotificationError,
    RenderedTemplate, QuietHours, DigestSettings,
};

pub use scheduler::{
    WorkflowSchedule, ScheduleType, Scheduler, SchedulerError,
    ScheduleExecution, CronExpression,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_imports() {
        // Verify all modules are accessible
        let _workflow: Option<Workflow> = None;
        let _step: Option<StepConfig> = None;
        let _trigger: Option<TriggerConfig> = None;
        let _approval: Option<ApprovalRequest> = None;
        let _notification: Option<Notification> = None;
        let _schedule: Option<WorkflowSchedule> = None;
    }
}
