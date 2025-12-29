use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{sleep, Duration, timeout};
use thiserror::Error;
use uuid::Uuid;
use chrono::Utc;

use super::workflow::{Workflow, WorkflowStatus, WorkflowError};
use super::step::{
    Step, StepConfig, StepExecutionContext, StepExecutionResult, StepStatus, StepError,
    ApprovalStep, NotificationStep, TransformStep, ExportStep, ValidateStep,
};

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Workflow error: {0}")]
    WorkflowError(#[from] WorkflowError),
    #[error("Step error: {0}")]
    StepError(#[from] StepError),
    #[error("Execution timeout")]
    Timeout,
    #[error("Workflow not found: {0}")]
    NotFound(Uuid),
    #[error("Workflow already running: {0}")]
    AlreadyRunning(Uuid),
    #[error("Invalid workflow state: {0}")]
    InvalidState(String),
    #[error("Dependency not met: {0}")]
    DependencyNotMet(String),
    #[error("Persistence error: {0}")]
    PersistenceError(String),
}

pub type EngineResult<T> = Result<T, EngineError>;

/// Workflow execution engine
pub struct WorkflowEngine {
    /// Active workflows being executed
    active_workflows: Arc<RwLock<HashMap<Uuid, Arc<Mutex<WorkflowExecution>>>>>,
    /// Workflow persistence (simplified - would be database in production)
    workflow_store: Arc<RwLock<HashMap<Uuid, Workflow>>>,
    /// Maximum concurrent workflows
    max_concurrent_workflows: usize,
    /// Step factory for creating step instances
    step_factory: Arc<StepFactory>,
}

impl WorkflowEngine {
    /// Create a new workflow engine
    pub fn new() -> Self {
        Self {
            active_workflows: Arc::new(RwLock::new(HashMap::new())),
            workflow_store: Arc::new(RwLock::new(HashMap::new())),
            max_concurrent_workflows: 100,
            step_factory: Arc::new(StepFactory::new()),
        }
    }

    /// Start executing a workflow
    pub async fn start_workflow(&self, workflow_id: Uuid) -> EngineResult<()> {
        // Check if already running
        {
            let active = self.active_workflows.read().await;
            if active.contains_key(&workflow_id) {
                return Err(EngineError::AlreadyRunning(workflow_id));
            }
        }

        // Check concurrent workflow limit
        {
            let active = self.active_workflows.read().await;
            if active.len() >= self.max_concurrent_workflows {
                return Err(EngineError::InvalidState(
                    "Maximum concurrent workflows reached".to_string(),
                ));
            }
        }

        // Load workflow from store
        let mut workflow = {
            let store = self.workflow_store.read().await;
            store
                .get(&workflow_id)
                .cloned()
                .ok_or(EngineError::NotFound(workflow_id))?
        };

        // Validate workflow
        workflow.validate()?;

        // Update status to active
        workflow.set_status(WorkflowStatus::Active)?;

        // Save updated workflow
        self.save_workflow(&workflow).await?;

        // Create execution context
        let execution = Arc::new(Mutex::new(WorkflowExecution::new(workflow)));

        // Add to active workflows
        {
            let mut active = self.active_workflows.write().await;
            active.insert(workflow_id, execution.clone());
        }

        // Spawn execution task
        let engine = self.clone_for_task();
        tokio::spawn(async move {
            if let Err(e) = engine.execute_workflow(execution).await {
                eprintln!("Workflow execution failed: {}", e);
            }
        });

        Ok(())
    }

    /// Execute a workflow
    async fn execute_workflow(&self, execution: Arc<Mutex<WorkflowExecution>>) -> EngineResult<()> {
        let workflow_id = {
            let exec = execution.lock().await;
            exec.workflow.id
        };

        loop {
            let should_continue = {
                let mut exec = execution.lock().await;

                // Check if workflow is paused
                if exec.workflow.status == WorkflowStatus::Paused {
                    return Ok(());
                }

                // Check if workflow is cancelled
                if exec.workflow.status == WorkflowStatus::Cancelled {
                    return Ok(());
                }

                // Get next step to execute
                match exec.get_next_step() {
                    Some(step_index) => {
                        exec.workflow.current_step_index = Some(step_index);
                        true
                    }
                    None => {
                        // No more steps - workflow complete
                        exec.workflow.set_status(WorkflowStatus::Completed)?;
                        self.save_workflow(&exec.workflow).await?;
                        false
                    }
                }
            };

            if !should_continue {
                break;
            }

            // Execute the step
            let result = self.execute_step(execution.clone()).await;

            // Handle step result
            {
                let mut exec = execution.lock().await;
                let step_index = exec.workflow.current_step_index.unwrap();

                match result {
                    Ok(step_result) => {
                        // Record step result
                        exec.record_step_result(step_index, step_result.clone());

                        match step_result.status {
                            StepStatus::Completed => {
                                exec.workflow.log_execution(format!(
                                    "Step {} completed successfully",
                                    step_index
                                ));
                            }
                            StepStatus::Waiting => {
                                // Step is waiting (e.g., for approval)
                                exec.workflow.log_execution(format!(
                                    "Step {} is waiting for external input",
                                    step_index
                                ));
                                exec.workflow.set_status(WorkflowStatus::Paused)?;
                                self.save_workflow(&exec.workflow).await?;
                                break;
                            }
                            StepStatus::Skipped => {
                                exec.workflow.log_execution(format!(
                                    "Step {} was skipped",
                                    step_index
                                ));
                            }
                            StepStatus::Failed => {
                                let step_config = &exec.workflow.steps[step_index];
                                if !step_config.continue_on_failure {
                                    exec.workflow.set_status(WorkflowStatus::Failed)?;
                                    exec.workflow.error_message = step_result.error_message.clone();
                                    self.save_workflow(&exec.workflow).await?;
                                    break;
                                } else {
                                    exec.workflow.log_execution(format!(
                                        "Step {} failed but continuing: {}",
                                        step_index,
                                        step_result.error_message.unwrap_or_default()
                                    ));
                                }
                            }
                            _ => {}
                        }

                        // Save workflow state
                        self.save_workflow(&exec.workflow).await?;
                    }
                    Err(e) => {
                        // Step execution error
                        exec.workflow.log_execution(format!(
                            "Step {} execution error: {}",
                            step_index, e
                        ));

                        let step_config = &exec.workflow.steps[step_index];
                        if !step_config.continue_on_failure {
                            exec.workflow.set_status(WorkflowStatus::Failed)?;
                            exec.workflow.error_message = Some(e.to_string());
                            self.save_workflow(&exec.workflow).await?;
                            break;
                        }
                    }
                }
            }
        }

        // Remove from active workflows
        {
            let mut active = self.active_workflows.write().await;
            active.remove(&workflow_id);
        }

        Ok(())
    }

    /// Execute a single step
    async fn execute_step(
        &self,
        execution: Arc<Mutex<WorkflowExecution>>,
    ) -> EngineResult<StepExecutionResult> {
        let (step_config, mut context, timeout_duration) = {
            let exec = execution.lock().await;
            let step_index = exec.workflow.current_step_index.unwrap();
            let step_config = exec.workflow.steps[step_index].clone();

            // Create execution context
            let context = StepExecutionContext {
                workflow_id: exec.workflow.id,
                step_id: step_config.id,
                initiator_id: exec.workflow.context.initiator_id,
                variables: exec.workflow.context.variables.clone(),
                previous_outputs: exec.step_outputs.clone(),
                entity_id: exec.workflow.context.entity_id,
                entity_type: exec.workflow.context.entity_type.clone(),
                metadata: exec.workflow.context.metadata.clone(),
                started_at: Utc::now(),
            };

            let timeout_duration = Duration::from_secs(step_config.timeout_seconds);

            (step_config, context, timeout_duration)
        };

        // Create step instance
        let step = self.step_factory.create_step(&step_config)?;

        // Check if step should execute
        if !step.should_execute(&context).await {
            return Ok(StepExecutionResult::skipped(
                "Step condition not met".to_string(),
            ));
        }

        // Execute step with retry logic
        let mut attempt = 0;
        let max_retries = step_config.retry_config.max_retries;

        loop {
            // Execute with timeout
            let execute_future = step.execute(&mut context);
            let result = timeout(timeout_duration, execute_future).await;

            match result {
                Ok(Ok(step_result)) => {
                    // Store step output
                    {
                        let mut exec = execution.lock().await;
                        exec.step_outputs.insert(
                            step_config.name.clone(),
                            step_result.output.clone(),
                        );
                    }
                    return Ok(step_result);
                }
                Ok(Err(e)) => {
                    // Step execution failed
                    if step.is_retryable() && attempt < max_retries {
                        attempt += 1;
                        let delay = step_config.retry_config.delay_for_attempt(attempt);

                        {
                            let mut exec = execution.lock().await;
                            exec.workflow.log_execution(format!(
                                "Step {} failed (attempt {}), retrying in {} seconds: {}",
                                step_config.name, attempt, delay, e
                            ));
                        }

                        sleep(Duration::from_secs(delay)).await;
                        continue;
                    } else {
                        return Err(EngineError::StepError(e));
                    }
                }
                Err(_) => {
                    // Timeout
                    if step.is_retryable() && attempt < max_retries {
                        attempt += 1;
                        let delay = step_config.retry_config.delay_for_attempt(attempt);

                        {
                            let mut exec = execution.lock().await;
                            exec.workflow.log_execution(format!(
                                "Step {} timed out (attempt {}), retrying in {} seconds",
                                step_config.name, attempt, delay
                            ));
                        }

                        sleep(Duration::from_secs(delay)).await;
                        continue;
                    } else {
                        return Err(EngineError::Timeout);
                    }
                }
            }
        }
    }

    /// Pause a workflow
    pub async fn pause_workflow(&self, workflow_id: Uuid) -> EngineResult<()> {
        let mut store = self.workflow_store.write().await;
        let workflow = store
            .get_mut(&workflow_id)
            .ok_or(EngineError::NotFound(workflow_id))?;

        workflow.set_status(WorkflowStatus::Paused)?;
        Ok(())
    }

    /// Resume a paused workflow
    pub async fn resume_workflow(&self, workflow_id: Uuid) -> EngineResult<()> {
        let mut store = self.workflow_store.write().await;
        let workflow = store
            .get_mut(&workflow_id)
            .ok_or(EngineError::NotFound(workflow_id))?;

        if workflow.status != WorkflowStatus::Paused {
            return Err(EngineError::InvalidState(
                "Workflow is not paused".to_string(),
            ));
        }

        workflow.set_status(WorkflowStatus::Active)?;
        drop(store);

        // Restart execution
        self.start_workflow(workflow_id).await
    }

    /// Cancel a workflow
    pub async fn cancel_workflow(&self, workflow_id: Uuid) -> EngineResult<()> {
        let mut store = self.workflow_store.write().await;
        let workflow = store
            .get_mut(&workflow_id)
            .ok_or(EngineError::NotFound(workflow_id))?;

        workflow.set_status(WorkflowStatus::Cancelled)?;
        Ok(())
    }

    /// Save workflow to persistent storage
    async fn save_workflow(&self, workflow: &Workflow) -> EngineResult<()> {
        let mut store = self.workflow_store.write().await;
        store.insert(workflow.id, workflow.clone());
        Ok(())
    }

    /// Load workflow from persistent storage
    pub async fn load_workflow(&self, workflow_id: Uuid) -> EngineResult<Workflow> {
        let store = self.workflow_store.read().await;
        store
            .get(&workflow_id)
            .cloned()
            .ok_or(EngineError::NotFound(workflow_id))
    }

    /// Create a new workflow in the store
    pub async fn create_workflow(&self, workflow: Workflow) -> EngineResult<()> {
        self.save_workflow(&workflow).await
    }

    /// Clone for async tasks
    fn clone_for_task(&self) -> Self {
        Self {
            active_workflows: Arc::clone(&self.active_workflows),
            workflow_store: Arc::clone(&self.workflow_store),
            max_concurrent_workflows: self.max_concurrent_workflows,
            step_factory: Arc::clone(&self.step_factory),
        }
    }
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Active workflow execution state
struct WorkflowExecution {
    /// The workflow being executed
    workflow: Workflow,
    /// Outputs from completed steps
    step_outputs: HashMap<String, serde_json::Value>,
    /// Steps that have been completed
    completed_steps: Vec<usize>,
    /// Steps that are waiting
    waiting_steps: Vec<usize>,
}

impl WorkflowExecution {
    /// Create a new workflow execution
    fn new(workflow: Workflow) -> Self {
        Self {
            workflow,
            step_outputs: HashMap::new(),
            completed_steps: Vec::new(),
            waiting_steps: Vec::new(),
        }
    }

    /// Get the next step to execute
    fn get_next_step(&self) -> Option<usize> {
        for (index, step) in self.workflow.steps.iter().enumerate() {
            // Skip completed and waiting steps
            if self.completed_steps.contains(&index) || self.waiting_steps.contains(&index) {
                continue;
            }

            // Check dependencies
            if self.are_dependencies_met(step) {
                return Some(index);
            }
        }
        None
    }

    /// Check if step dependencies are met
    fn are_dependencies_met(&self, step: &StepConfig) -> bool {
        if step.dependencies.is_empty() {
            return true;
        }

        // All dependencies must be completed
        for dep_name in &step.dependencies {
            let dep_completed = self
                .workflow
                .steps
                .iter()
                .enumerate()
                .any(|(idx, s)| s.name == *dep_name && self.completed_steps.contains(&idx));

            if !dep_completed {
                return false;
            }
        }

        true
    }

    /// Record step execution result
    fn record_step_result(&mut self, step_index: usize, result: StepExecutionResult) {
        match result.status {
            StepStatus::Completed => {
                self.completed_steps.push(step_index);
                self.waiting_steps.retain(|&i| i != step_index);
            }
            StepStatus::Waiting => {
                if !self.waiting_steps.contains(&step_index) {
                    self.waiting_steps.push(step_index);
                }
            }
            StepStatus::Skipped => {
                self.completed_steps.push(step_index);
            }
            _ => {}
        }
    }
}

/// Factory for creating step instances
struct StepFactory;

impl StepFactory {
    fn new() -> Self {
        Self
    }

    /// Create a step instance from configuration
    fn create_step(&self, config: &StepConfig) -> EngineResult<Box<dyn Step>> {
        let step: Box<dyn Step> = match config.step_type {
            super::step::StepType::Approval => {
                Box::new(ApprovalStep {
                    config: config.clone(),
                })
            }
            super::step::StepType::Notification => {
                Box::new(NotificationStep {
                    config: config.clone(),
                })
            }
            super::step::StepType::Transform => {
                Box::new(TransformStep {
                    config: config.clone(),
                })
            }
            super::step::StepType::Export => {
                Box::new(ExportStep {
                    config: config.clone(),
                })
            }
            super::step::StepType::Validate => {
                Box::new(ValidateStep {
                    config: config.clone(),
                })
            }
            _ => {
                return Err(EngineError::InvalidState(format!(
                    "Unsupported step type: {:?}",
                    config.step_type
                )));
            }
        };

        Ok(step)
    }
}

/// Workflow state persistence (simplified interface)
#[async_trait::async_trait]
pub trait WorkflowPersistence: Send + Sync {
    /// Save workflow state
    async fn save(&self, workflow: &Workflow) -> Result<(), String>;

    /// Load workflow state
    async fn load(&self, workflow_id: Uuid) -> Result<Option<Workflow>, String>;

    /// Delete workflow
    async fn delete(&self, workflow_id: Uuid) -> Result<(), String>;

    /// List workflows by status
    async fn list_by_status(&self, status: WorkflowStatus) -> Result<Vec<Workflow>, String>;
}

/// In-memory workflow persistence (for testing/development)
pub struct InMemoryPersistence {
    workflows: Arc<RwLock<HashMap<Uuid, Workflow>>>,
}

impl InMemoryPersistence {
    pub fn new() -> Self {
        Self {
            workflows: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryPersistence {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl WorkflowPersistence for InMemoryPersistence {
    async fn save(&self, workflow: &Workflow) -> Result<(), String> {
        let mut workflows = self.workflows.write().await;
        workflows.insert(workflow.id, workflow.clone());
        Ok(())
    }

    async fn load(&self, workflow_id: Uuid) -> Result<Option<Workflow>, String> {
        let workflows = self.workflows.read().await;
        Ok(workflows.get(&workflow_id).cloned())
    }

    async fn delete(&self, workflow_id: Uuid) -> Result<(), String> {
        let mut workflows = self.workflows.write().await;
        workflows.remove(&workflow_id);
        Ok(())
    }

    async fn list_by_status(&self, status: WorkflowStatus) -> Result<Vec<Workflow>, String> {
        let workflows = self.workflows.read().await;
        Ok(workflows
            .values()
            .filter(|w| w.status == status)
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::step::StepType;

    #[tokio::test]
    async fn test_workflow_engine_creation() {
        let engine = WorkflowEngine::new();
        assert_eq!(engine.max_concurrent_workflows, 100);
    }

    #[tokio::test]
    async fn test_create_and_load_workflow() {
        let engine = WorkflowEngine::new();
        let user_id = Uuid::new_v4();

        let mut workflow = Workflow::new(
            "Test Workflow".to_string(),
            "Test Description".to_string(),
            user_id,
        );

        // Add a simple step
        let step = StepConfig::new("Step1".to_string(), StepType::Notification);
        workflow.add_step(step).unwrap();

        let workflow_id = workflow.id;
        engine.create_workflow(workflow).await.unwrap();

        let loaded = engine.load_workflow(workflow_id).await.unwrap();
        assert_eq!(loaded.id, workflow_id);
        assert_eq!(loaded.steps.len(), 1);
    }

    #[tokio::test]
    async fn test_in_memory_persistence() {
        let persistence = InMemoryPersistence::new();
        let user_id = Uuid::new_v4();

        let workflow = Workflow::new(
            "Test".to_string(),
            "Test".to_string(),
            user_id,
        );
        let workflow_id = workflow.id;

        persistence.save(&workflow).await.unwrap();

        let loaded = persistence.load(workflow_id).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().id, workflow_id);
    }

    #[tokio::test]
    async fn test_pause_resume_workflow() {
        let engine = WorkflowEngine::new();
        let user_id = Uuid::new_v4();

        let mut workflow = Workflow::new(
            "Test".to_string(),
            "Test".to_string(),
            user_id,
        );
        workflow.set_status(WorkflowStatus::Active).unwrap();

        let workflow_id = workflow.id;
        engine.create_workflow(workflow).await.unwrap();

        engine.pause_workflow(workflow_id).await.unwrap();

        let paused = engine.load_workflow(workflow_id).await.unwrap();
        assert_eq!(paused.status, WorkflowStatus::Paused);
    }
}
