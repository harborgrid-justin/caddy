//! Saga and Process Manager Implementation
//!
//! Provides infrastructure for long-running business processes that coordinate
//! multiple aggregates with compensation actions and timeout handling.

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::store::{EventStore, StoredEvent};
use crate::enterprise::error::{EnterpriseError, EnterpriseResult};

/// Saga state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SagaStatus {
    /// Saga is running
    Running,
    /// Saga completed successfully
    Completed,
    /// Saga failed and is compensating
    Compensating,
    /// Saga failed and compensation completed
    Compensated,
    /// Saga failed and compensation also failed
    Failed,
}

/// Saga step definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaStep {
    /// Step name/identifier
    pub name: String,
    /// Whether this step has been executed
    pub executed: bool,
    /// Whether this step has been compensated
    pub compensated: bool,
    /// Error if step failed
    pub error: Option<String>,
}

impl SagaStep {
    /// Create a new saga step
    pub fn new(name: String) -> Self {
        Self {
            name,
            executed: false,
            compensated: false,
            error: None,
        }
    }

    /// Mark step as executed
    pub fn mark_executed(&mut self) {
        self.executed = true;
    }

    /// Mark step as failed
    pub fn mark_failed(&mut self, error: String) {
        self.error = Some(error);
    }

    /// Mark step as compensated
    pub fn mark_compensated(&mut self) {
        self.compensated = true;
    }
}

/// Saga instance tracking state and progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaInstance {
    /// Unique saga ID
    pub saga_id: Uuid,
    /// Saga type/name
    pub saga_type: String,
    /// Current status
    pub status: SagaStatus,
    /// Steps in the saga
    pub steps: Vec<SagaStep>,
    /// Current step index
    pub current_step: usize,
    /// Saga data (serialized)
    pub data: Vec<u8>,
    /// When the saga was started
    pub started_at: DateTime<Utc>,
    /// When the saga was last updated
    pub updated_at: DateTime<Utc>,
    /// When the saga completed (if completed)
    pub completed_at: Option<DateTime<Utc>>,
    /// Timeout duration
    pub timeout: Option<Duration>,
    /// Correlation ID for tracking
    pub correlation_id: Option<Uuid>,
}

impl SagaInstance {
    /// Create a new saga instance
    pub fn new(saga_type: String, data: Vec<u8>, steps: Vec<String>) -> Self {
        Self {
            saga_id: Uuid::new_v4(),
            saga_type,
            status: SagaStatus::Running,
            steps: steps.into_iter().map(SagaStep::new).collect(),
            current_step: 0,
            data,
            started_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
            timeout: None,
            correlation_id: None,
        }
    }

    /// Set timeout for the saga
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set correlation ID
    pub fn with_correlation(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    /// Check if saga has timed out
    pub fn is_timed_out(&self) -> bool {
        if let Some(timeout) = self.timeout {
            Utc::now() - self.started_at > timeout
        } else {
            false
        }
    }

    /// Move to next step
    pub fn advance_step(&mut self) {
        if self.current_step < self.steps.len() {
            self.steps[self.current_step].mark_executed();
            self.current_step += 1;
            self.updated_at = Utc::now();
        }
    }

    /// Mark current step as failed
    pub fn fail_current_step(&mut self, error: String) {
        if self.current_step < self.steps.len() {
            self.steps[self.current_step].mark_failed(error);
            self.status = SagaStatus::Compensating;
            self.updated_at = Utc::now();
        }
    }

    /// Mark saga as completed
    pub fn complete(&mut self) {
        self.status = SagaStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Mark saga as failed
    pub fn fail(&mut self) {
        self.status = SagaStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Mark saga as compensated
    pub fn compensate(&mut self) {
        self.status = SagaStatus::Compensated;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
}

/// Trait for saga definitions
#[async_trait]
pub trait Saga: Send + Sync + Debug {
    /// Saga type identifier
    fn saga_type(&self) -> &str;

    /// Get step names in order
    fn steps(&self) -> Vec<String>;

    /// Execute a step
    async fn execute_step(&self, step_name: &str, data: &[u8]) -> EnterpriseResult<Vec<u8>>;

    /// Compensate a step (undo it)
    async fn compensate_step(&self, step_name: &str, data: &[u8]) -> EnterpriseResult<()>;

    /// Handle timeout
    async fn on_timeout(&self, _data: &[u8]) -> EnterpriseResult<()> {
        Ok(())
    }
}

/// Trait for saga persistence
#[async_trait]
pub trait SagaStore: Send + Sync {
    /// Save a saga instance
    async fn save(&self, instance: &SagaInstance) -> EnterpriseResult<()>;

    /// Load a saga instance
    async fn load(&self, saga_id: &Uuid) -> EnterpriseResult<Option<SagaInstance>>;

    /// Load all active sagas
    async fn load_active(&self) -> EnterpriseResult<Vec<SagaInstance>>;

    /// Delete a saga instance
    async fn delete(&self, saga_id: &Uuid) -> EnterpriseResult<()>;
}

/// In-memory saga store for testing
pub struct InMemorySagaStore {
    sagas: Arc<DashMap<Uuid, SagaInstance>>,
}

impl InMemorySagaStore {
    /// Create a new in-memory saga store
    pub fn new() -> Self {
        Self {
            sagas: Arc::new(DashMap::new()),
        }
    }
}

impl Default for InMemorySagaStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SagaStore for InMemorySagaStore {
    async fn save(&self, instance: &SagaInstance) -> EnterpriseResult<()> {
        self.sagas.insert(instance.saga_id, instance.clone());
        Ok(())
    }

    async fn load(&self, saga_id: &Uuid) -> EnterpriseResult<Option<SagaInstance>> {
        Ok(self.sagas.get(saga_id).map(|entry| entry.clone()))
    }

    async fn load_active(&self) -> EnterpriseResult<Vec<SagaInstance>> {
        Ok(self
            .sagas
            .iter()
            .filter(|entry| {
                matches!(
                    entry.status,
                    SagaStatus::Running | SagaStatus::Compensating
                )
            })
            .map(|entry| entry.clone())
            .collect())
    }

    async fn delete(&self, saga_id: &Uuid) -> EnterpriseResult<()> {
        self.sagas.remove(saga_id);
        Ok(())
    }
}

/// Saga coordinator/orchestrator
pub struct SagaCoordinator {
    store: Arc<dyn SagaStore>,
    sagas: Arc<DashMap<String, Arc<dyn Saga>>>,
    running: Arc<RwLock<bool>>,
}

impl SagaCoordinator {
    /// Create a new saga coordinator
    pub fn new(store: Arc<dyn SagaStore>) -> Self {
        Self {
            store,
            sagas: Arc::new(DashMap::new()),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Register a saga definition
    pub fn register(&self, saga: Arc<dyn Saga>) {
        self.sagas.insert(saga.saga_type().to_string(), saga);
    }

    /// Start a new saga
    pub async fn start_saga<T: Serialize>(
        &self,
        saga_type: &str,
        data: &T,
    ) -> EnterpriseResult<Uuid> {
        let saga = self
            .sagas
            .get(saga_type)
            .ok_or_else(|| {
                EnterpriseError::Other(format!("Saga type not found: {}", saga_type))
            })?
            .clone();

        let serialized_data = serde_json::to_vec(data)
            .map_err(|e| EnterpriseError::Other(format!("Failed to serialize data: {}", e)))?;

        let instance = SagaInstance::new(saga_type.to_string(), serialized_data, saga.steps());

        let saga_id = instance.saga_id;
        self.store.save(&instance).await?;

        // Execute the saga asynchronously
        self.execute_saga_async(saga_id).await;

        Ok(saga_id)
    }

    /// Execute a saga
    async fn execute_saga_async(&self, saga_id: Uuid) {
        let store = self.store.clone();
        let sagas = self.sagas.clone();

        tokio::spawn(async move {
            let _ = Self::execute_saga_internal(saga_id, store, sagas).await;
        });
    }

    async fn execute_saga_internal(
        saga_id: Uuid,
        store: Arc<dyn SagaStore>,
        sagas: Arc<DashMap<String, Arc<dyn Saga>>>,
    ) -> EnterpriseResult<()> {
        let mut instance = store
            .load(&saga_id)
            .await?
            .ok_or_else(|| EnterpriseError::Other("Saga not found".to_string()))?;

        let saga = sagas
            .get(&instance.saga_type)
            .ok_or_else(|| {
                EnterpriseError::Other(format!("Saga type not found: {}", instance.saga_type))
            })?
            .clone();

        // Check for timeout
        if instance.is_timed_out() {
            saga.on_timeout(&instance.data).await?;
            instance.fail();
            store.save(&instance).await?;
            return Ok(());
        }

        // Execute steps
        let mut current_data = instance.data.clone();

        while instance.current_step < instance.steps.len() {
            let step = &instance.steps[instance.current_step];
            let step_name = step.name.clone();

            match saga.execute_step(&step_name, &current_data).await {
                Ok(new_data) => {
                    current_data = new_data;
                    instance.data = current_data.clone();
                    instance.advance_step();
                    store.save(&instance).await?;
                }
                Err(e) => {
                    // Step failed, start compensation
                    instance.fail_current_step(e.to_string());
                    store.save(&instance).await?;

                    // Compensate all executed steps in reverse order
                    Self::compensate_saga(&mut instance, saga.as_ref(), &store).await?;
                    return Ok(());
                }
            }
        }

        // All steps completed successfully
        instance.complete();
        store.save(&instance).await?;

        Ok(())
    }

    async fn compensate_saga(
        instance: &mut SagaInstance,
        saga: &dyn Saga,
        store: &Arc<dyn SagaStore>,
    ) -> EnterpriseResult<()> {
        // Compensate in reverse order
        for i in (0..instance.current_step).rev() {
            let step = &instance.steps[i];
            if step.executed && !step.compensated {
                match saga.compensate_step(&step.name, &instance.data).await {
                    Ok(()) => {
                        instance.steps[i].mark_compensated();
                        store.save(instance).await?;
                    }
                    Err(e) => {
                        // Compensation failed
                        instance.fail();
                        store.save(instance).await?;
                        return Err(e);
                    }
                }
            }
        }

        instance.compensate();
        store.save(instance).await?;
        Ok(())
    }

    /// Get saga status
    pub async fn get_status(&self, saga_id: &Uuid) -> EnterpriseResult<Option<SagaInstance>> {
        self.store.load(saga_id).await
    }

    /// Start monitoring for timeouts (call this once at startup)
    pub async fn start_timeout_monitor(&self) {
        let mut running = self.running.write().await;
        if *running {
            return;
        }
        *running = true;
        drop(running);

        let store = self.store.clone();
        let running_flag = self.running.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));

            loop {
                interval.tick().await;

                let is_running = *running_flag.read().await;
                if !is_running {
                    break;
                }

                // Check for timed out sagas
                if let Ok(active_sagas) = store.load_active().await {
                    for saga in active_sagas {
                        if saga.is_timed_out() && saga.status == SagaStatus::Running {
                            // Handle timeout
                            let mut updated = saga;
                            updated.fail();
                            let _ = store.save(&updated).await;
                        }
                    }
                }
            }
        });
    }

    /// Stop the timeout monitor
    pub async fn stop_timeout_monitor(&self) {
        let mut running = self.running.write().await;
        *running = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestSagaData {
        value: i32,
        steps_completed: Vec<String>,
    }

    #[derive(Debug)]
    struct TestSaga;

    #[async_trait]
    impl Saga for TestSaga {
        fn saga_type(&self) -> &str {
            "TestSaga"
        }

        fn steps(&self) -> Vec<String> {
            vec![
                "Step1".to_string(),
                "Step2".to_string(),
                "Step3".to_string(),
            ]
        }

        async fn execute_step(&self, step_name: &str, data: &[u8]) -> EnterpriseResult<Vec<u8>> {
            let mut saga_data: TestSagaData = serde_json::from_slice(data)
                .map_err(|e| EnterpriseError::Other(e.to_string()))?;

            saga_data.value += 1;
            saga_data.steps_completed.push(step_name.to_string());

            serde_json::to_vec(&saga_data).map_err(|e| EnterpriseError::Other(e.to_string()))
        }

        async fn compensate_step(&self, step_name: &str, data: &[u8]) -> EnterpriseResult<()> {
            let mut saga_data: TestSagaData = serde_json::from_slice(data)
                .map_err(|e| EnterpriseError::Other(e.to_string()))?;

            saga_data.value -= 1;
            saga_data
                .steps_completed
                .retain(|s| s != step_name);

            Ok(())
        }
    }

    #[tokio::test]
    async fn test_saga_instance_creation() {
        let steps = vec!["Step1".to_string(), "Step2".to_string()];
        let instance = SagaInstance::new("TestSaga".to_string(), vec![1, 2, 3], steps);

        assert_eq!(instance.saga_type, "TestSaga");
        assert_eq!(instance.status, SagaStatus::Running);
        assert_eq!(instance.steps.len(), 2);
        assert_eq!(instance.current_step, 0);
    }

    #[tokio::test]
    async fn test_saga_step_progression() {
        let steps = vec!["Step1".to_string(), "Step2".to_string()];
        let mut instance = SagaInstance::new("TestSaga".to_string(), vec![], steps);

        assert_eq!(instance.current_step, 0);

        instance.advance_step();
        assert_eq!(instance.current_step, 1);
        assert!(instance.steps[0].executed);

        instance.advance_step();
        assert_eq!(instance.current_step, 2);
        assert!(instance.steps[1].executed);
    }

    #[tokio::test]
    async fn test_saga_timeout() {
        let steps = vec!["Step1".to_string()];
        let mut instance = SagaInstance::new("TestSaga".to_string(), vec![], steps)
            .with_timeout(Duration::milliseconds(10));

        assert!(!instance.is_timed_out());

        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;

        assert!(instance.is_timed_out());
    }

    #[tokio::test]
    async fn test_in_memory_saga_store() {
        let store = InMemorySagaStore::new();
        let steps = vec!["Step1".to_string()];
        let instance = SagaInstance::new("TestSaga".to_string(), vec![1, 2, 3], steps);
        let saga_id = instance.saga_id;

        store.save(&instance).await.unwrap();

        let loaded = store.load(&saga_id).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().saga_id, saga_id);

        store.delete(&saga_id).await.unwrap();
        let deleted = store.load(&saga_id).await.unwrap();
        assert!(deleted.is_none());
    }

    #[tokio::test]
    async fn test_saga_coordinator() {
        let store = Arc::new(InMemorySagaStore::new()) as Arc<dyn SagaStore>;
        let coordinator = SagaCoordinator::new(store.clone());

        let saga = Arc::new(TestSaga) as Arc<dyn Saga>;
        coordinator.register(saga);

        let data = TestSagaData {
            value: 0,
            steps_completed: vec![],
        };

        let saga_id = coordinator.start_saga("TestSaga", &data).await.unwrap();

        // Wait for saga to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let status = coordinator.get_status(&saga_id).await.unwrap();
        assert!(status.is_some());

        let instance = status.unwrap();
        assert_eq!(instance.status, SagaStatus::Completed);

        let final_data: TestSagaData = serde_json::from_slice(&instance.data).unwrap();
        assert_eq!(final_data.value, 3);
        assert_eq!(final_data.steps_completed.len(), 3);
    }

    #[tokio::test]
    async fn test_load_active_sagas() {
        let store = InMemorySagaStore::new();

        let mut running = SagaInstance::new(
            "TestSaga".to_string(),
            vec![],
            vec!["Step1".to_string()],
        );
        running.status = SagaStatus::Running;

        let mut completed = SagaInstance::new(
            "TestSaga".to_string(),
            vec![],
            vec!["Step1".to_string()],
        );
        completed.status = SagaStatus::Completed;

        store.save(&running).await.unwrap();
        store.save(&completed).await.unwrap();

        let active = store.load_active().await.unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].saga_id, running.saga_id);
    }
}
