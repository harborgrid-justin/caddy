//! Command Handling and CQRS
//!
//! Provides command handling infrastructure with validation, routing,
//! and idempotency support for CQRS architecture.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;
use uuid::Uuid;

use super::aggregate::{AggregateRepository, AggregateRoot};
use crate::enterprise::error::{EnterpriseError, EnterpriseResult};

/// Trait for commands that can be executed
pub trait Command: Clone + Serialize + DeserializeOwned + Send + Sync + Debug {
    /// Get the command type identifier
    fn command_type(&self) -> &str;

    /// Get the aggregate ID this command targets
    fn aggregate_id(&self) -> String;

    /// Validate the command (business rule validation)
    fn validate(&self) -> EnterpriseResult<()> {
        Ok(())
    }

    /// Get idempotency key if this command should be idempotent
    fn idempotency_key(&self) -> Option<String> {
        None
    }
}

/// Result of command execution
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// Command ID
    pub command_id: Uuid,
    /// Aggregate ID affected
    pub aggregate_id: String,
    /// Events produced by the command
    pub events: Vec<String>,
    /// Final aggregate version
    pub version: u64,
    /// Timestamp of execution
    pub timestamp: DateTime<Utc>,
}

/// Command handler trait for processing commands
#[async_trait]
pub trait CommandHandler<C: Command, A: AggregateRoot>: Send + Sync {
    /// Handle a command and produce events
    ///
    /// # Arguments
    ///
    /// * `command` - The command to handle
    /// * `aggregate` - The current state of the aggregate (None if new)
    ///
    /// # Returns
    ///
    /// Vector of events to be applied to the aggregate
    async fn handle(
        &self,
        command: &C,
        aggregate: Option<&A>,
    ) -> EnterpriseResult<Vec<A::Event>>;
}

/// Command bus for routing and executing commands
pub struct CommandBus<A: AggregateRoot> {
    repository: Arc<AggregateRepository<A>>,
    idempotency_store: Arc<DashMap<String, CommandResult>>,
}

impl<A: AggregateRoot> CommandBus<A> {
    /// Create a new command bus
    pub fn new(repository: Arc<AggregateRepository<A>>) -> Self {
        Self {
            repository,
            idempotency_store: Arc::new(DashMap::new()),
        }
    }

    /// Execute a command with a specific handler
    ///
    /// # Arguments
    ///
    /// * `command` - The command to execute
    /// * `handler` - The handler that will process the command
    /// * `correlation_id` - Optional correlation ID for event tracking
    ///
    /// # Returns
    ///
    /// Command result with produced events and final state
    pub async fn execute<C, H>(
        &self,
        command: &C,
        handler: &H,
        correlation_id: Option<Uuid>,
    ) -> EnterpriseResult<CommandResult>
    where
        C: Command,
        H: CommandHandler<C, A>,
    {
        // Check for idempotency
        if let Some(idempotency_key) = command.idempotency_key() {
            if let Some(cached_result) = self.idempotency_store.get(&idempotency_key) {
                return Ok(cached_result.clone());
            }
        }

        // Validate command
        command.validate()?;

        let aggregate_id = command.aggregate_id();
        let command_id = Uuid::new_v4();

        // Load current aggregate state
        let aggregate = self.repository.load(&aggregate_id).await?;
        let expected_version = aggregate.as_ref().map(|a| a.version()).unwrap_or(0);

        // Handle command to produce events
        let events = handler.handle(command, aggregate.as_ref()).await?;

        if events.is_empty() {
            return Err(EnterpriseError::Other(
                "Command handler produced no events".to_string(),
            ));
        }

        // Apply events to aggregate for validation
        let mut new_aggregate = aggregate.unwrap_or_default();
        for event in &events {
            new_aggregate.apply_event(event);
        }

        // Save events to event store
        let stored_events = self
            .repository
            .save(
                &new_aggregate,
                events.clone(),
                expected_version,
                correlation_id,
                Some(command_id),
            )
            .await?;

        let result = CommandResult {
            command_id,
            aggregate_id: aggregate_id.clone(),
            events: stored_events
                .iter()
                .map(|e| e.metadata.event_type.clone())
                .collect(),
            version: new_aggregate.version(),
            timestamp: Utc::now(),
        };

        // Store for idempotency if key provided
        if let Some(idempotency_key) = command.idempotency_key() {
            self.idempotency_store
                .insert(idempotency_key, result.clone());
        }

        Ok(result)
    }

    /// Clear idempotency cache (useful for testing or cleanup)
    pub fn clear_idempotency_cache(&self) {
        self.idempotency_store.clear();
    }

    /// Get the number of cached idempotent command results
    pub fn idempotency_cache_size(&self) -> usize {
        self.idempotency_store.len()
    }
}

/// Command dispatcher for routing commands to appropriate handlers
pub struct CommandDispatcher {
    handlers: DashMap<String, Arc<dyn std::any::Any + Send + Sync>>,
}

impl CommandDispatcher {
    /// Create a new command dispatcher
    pub fn new() -> Self {
        Self {
            handlers: DashMap::new(),
        }
    }

    /// Register a command handler
    pub fn register_handler<C, A, H>(&self, handler: Arc<H>)
    where
        C: Command + 'static,
        A: AggregateRoot + 'static,
        H: CommandHandler<C, A> + 'static,
    {
        let command_type = std::any::type_name::<C>().to_string();
        self.handlers.insert(command_type, handler as Arc<_>);
    }

    /// Get a registered handler
    pub fn get_handler<C, A, H>(&self) -> Option<Arc<H>>
    where
        C: Command + 'static,
        A: AggregateRoot + 'static,
        H: CommandHandler<C, A> + 'static,
    {
        let command_type = std::any::type_name::<C>();
        self.handlers.get(command_type).and_then(|entry| {
            let handler = entry.value();
            handler.clone().downcast::<H>().ok()
        })
    }

    /// Check if a handler is registered for a command type
    pub fn has_handler<C: Command + 'static>(&self) -> bool {
        let command_type = std::any::type_name::<C>();
        self.handlers.contains_key(command_type)
    }
}

impl Default for CommandDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Command metadata for auditing and tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMetadata {
    /// Command ID
    pub command_id: Uuid,
    /// Command type
    pub command_type: String,
    /// User who issued the command
    pub user_id: Option<String>,
    /// Timestamp when command was issued
    pub timestamp: DateTime<Utc>,
    /// Correlation ID for tracking related commands
    pub correlation_id: Option<Uuid>,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl CommandMetadata {
    /// Create new command metadata
    pub fn new(command_type: String) -> Self {
        Self {
            command_id: Uuid::new_v4(),
            command_type,
            user_id: None,
            timestamp: Utc::now(),
            correlation_id: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Set the user ID
    pub fn with_user(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Set the correlation ID
    pub fn with_correlation(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    /// Add custom metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enterprise::eventsource::{
        aggregate::{AggregateBuilder, AggregateRepository},
        store::InMemoryEventStore,
    };
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    enum TestEvent {
        Created { id: String, name: String },
        NameChanged { name: String },
    }

    impl super::super::aggregate::DomainEvent for TestEvent {
        fn event_type(&self) -> &str {
            match self {
                TestEvent::Created { .. } => "Test.Created",
                TestEvent::NameChanged { .. } => "Test.NameChanged",
            }
        }
    }

    #[derive(Debug, Clone, Default)]
    struct TestAggregate {
        id: String,
        name: String,
        version: u64,
    }

    impl super::super::aggregate::AggregateRoot for TestAggregate {
        type Event = TestEvent;

        fn aggregate_id(&self) -> String {
            self.id.clone()
        }

        fn version(&self) -> u64 {
            self.version
        }

        fn apply_event(&mut self, event: &Self::Event) {
            self.version += 1;
            match event {
                TestEvent::Created { id, name } => {
                    self.id = id.clone();
                    self.name = name.clone();
                }
                TestEvent::NameChanged { name } => {
                    self.name = name.clone();
                }
            }
        }

        fn aggregate_type() -> &'static str {
            "TestAggregate"
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct CreateCommand {
        id: String,
        name: String,
    }

    impl Command for CreateCommand {
        fn command_type(&self) -> &str {
            "CreateTest"
        }

        fn aggregate_id(&self) -> String {
            self.id.clone()
        }

        fn validate(&self) -> EnterpriseResult<()> {
            if self.name.is_empty() {
                return Err(EnterpriseError::Other("Name cannot be empty".to_string()));
            }
            Ok(())
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct ChangeNameCommand {
        id: String,
        name: String,
        idempotency_key: Option<String>,
    }

    impl Command for ChangeNameCommand {
        fn command_type(&self) -> &str {
            "ChangeName"
        }

        fn aggregate_id(&self) -> String {
            self.id.clone()
        }

        fn idempotency_key(&self) -> Option<String> {
            self.idempotency_key.clone()
        }
    }

    struct CreateCommandHandler;

    #[async_trait]
    impl CommandHandler<CreateCommand, TestAggregate> for CreateCommandHandler {
        async fn handle(
            &self,
            command: &CreateCommand,
            aggregate: Option<&TestAggregate>,
        ) -> EnterpriseResult<Vec<TestEvent>> {
            if aggregate.is_some() {
                return Err(EnterpriseError::Other("Aggregate already exists".to_string()));
            }

            Ok(vec![TestEvent::Created {
                id: command.id.clone(),
                name: command.name.clone(),
            }])
        }
    }

    struct ChangeNameCommandHandler;

    #[async_trait]
    impl CommandHandler<ChangeNameCommand, TestAggregate> for ChangeNameCommandHandler {
        async fn handle(
            &self,
            command: &ChangeNameCommand,
            aggregate: Option<&TestAggregate>,
        ) -> EnterpriseResult<Vec<TestEvent>> {
            if aggregate.is_none() {
                return Err(EnterpriseError::Other("Aggregate does not exist".to_string()));
            }

            Ok(vec![TestEvent::NameChanged {
                name: command.name.clone(),
            }])
        }
    }

    #[tokio::test]
    async fn test_command_execution() {
        let store = Box::new(InMemoryEventStore::new());
        let repo = Arc::new(AggregateRepository::<TestAggregate>::new(store));
        let bus = CommandBus::new(repo.clone());
        let handler = CreateCommandHandler;

        let command = CreateCommand {
            id: "test-1".to_string(),
            name: "Test".to_string(),
        };

        let result = bus.execute(&command, &handler, None).await.unwrap();
        assert_eq!(result.aggregate_id, "test-1");
        assert_eq!(result.version, 1);
        assert_eq!(result.events.len(), 1);
    }

    #[tokio::test]
    async fn test_command_validation() {
        let store = Box::new(InMemoryEventStore::new());
        let repo = Arc::new(AggregateRepository::<TestAggregate>::new(store));
        let bus = CommandBus::new(repo);
        let handler = CreateCommandHandler;

        let command = CreateCommand {
            id: "test-1".to_string(),
            name: "".to_string(), // Invalid: empty name
        };

        let result = bus.execute(&command, &handler, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_idempotency() {
        let store = Box::new(InMemoryEventStore::new());
        let repo = Arc::new(AggregateRepository::<TestAggregate>::new(store));
        let bus = CommandBus::new(repo.clone());

        // First create the aggregate
        let create_cmd = CreateCommand {
            id: "test-1".to_string(),
            name: "Test".to_string(),
        };
        bus.execute(&create_cmd, &CreateCommandHandler, None)
            .await
            .unwrap();

        // Execute command with idempotency key
        let handler = ChangeNameCommandHandler;
        let command = ChangeNameCommand {
            id: "test-1".to_string(),
            name: "Updated".to_string(),
            idempotency_key: Some("idem-key-1".to_string()),
        };

        let result1 = bus.execute(&command, &handler, None).await.unwrap();
        let result2 = bus.execute(&command, &handler, None).await.unwrap();

        assert_eq!(result1.command_id, result2.command_id);
        assert_eq!(result1.version, result2.version);

        // Verify only one event was created
        let aggregate = repo.load("test-1").await.unwrap().unwrap();
        assert_eq!(aggregate.version, 2); // Create + one name change
    }

    #[tokio::test]
    async fn test_command_dispatcher() {
        let dispatcher = CommandDispatcher::new();
        let handler = Arc::new(CreateCommandHandler);

        dispatcher.register_handler::<CreateCommand, TestAggregate, _>(handler.clone());

        assert!(dispatcher.has_handler::<CreateCommand>());

        let retrieved = dispatcher.get_handler::<CreateCommand, TestAggregate, CreateCommandHandler>();
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_command_metadata() {
        let metadata = CommandMetadata::new("CreateTest".to_string())
            .with_user("user-123".to_string())
            .with_correlation(Uuid::new_v4())
            .with_metadata("ip".to_string(), "127.0.0.1".to_string());

        assert_eq!(metadata.command_type, "CreateTest");
        assert_eq!(metadata.user_id, Some("user-123".to_string()));
        assert!(metadata.correlation_id.is_some());
        assert_eq!(
            metadata.metadata.get("ip"),
            Some(&"127.0.0.1".to_string())
        );
    }
}
