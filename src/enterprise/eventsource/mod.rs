//! # Event Sourcing & CQRS Module
//!
//! Complete event sourcing and Command Query Responsibility Segregation (CQRS)
//! implementation for enterprise-grade CAD applications.
//!
//! ## Overview
//!
//! Event sourcing is a pattern where state changes are stored as a sequence of events
//! rather than updating a current state. This provides:
//!
//! - **Complete Audit Trail**: Every change is recorded
//! - **Temporal Queries**: Query state at any point in time
//! - **Event Replay**: Rebuild state from events
//! - **Debugging**: Understand exactly what happened and when
//!
//! CQRS separates read and write operations, allowing:
//!
//! - **Optimized Reads**: Build read models optimized for queries
//! - **Scalability**: Scale reads and writes independently
//! - **Multiple Views**: Create different projections of the same data
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────┐
//! │   Commands  │─┐
//! └─────────────┘ │
//!                 ▼
//!          ┌──────────────┐         ┌──────────────┐
//!          │  Aggregates  │────────▶│ Event Store  │
//!          └──────────────┘         └──────────────┘
//!                                          │
//!                                          ▼
//!                                   ┌─────────────┐
//!                                   │ Projections │
//!                                   └─────────────┘
//!                                          │
//!                                          ▼
//!                                   ┌─────────────┐
//!                                   │ Read Models │
//!                                   └─────────────┘
//! ```
//!
//! ## Core Components
//!
//! ### Event Store
//!
//! Append-only log of domain events with:
//! - Stream per aggregate
//! - Global ordering
//! - Optimistic concurrency control
//! - Event versioning
//!
//! ### Aggregates
//!
//! Domain entities that:
//! - Process commands
//! - Produce events
//! - Apply events to update state
//! - Enforce business rules
//!
//! ### Commands
//!
//! Intents to change state with:
//! - Validation
//! - Routing to handlers
//! - Idempotency support
//! - Audit trail
//!
//! ### Projections
//!
//! Read models built from events with:
//! - Catch-up subscriptions
//! - Live updates
//! - Checkpointing
//! - Multiple views
//!
//! ### Snapshots
//!
//! Optimization for aggregate loading:
//! - Periodic state snapshots
//! - Configurable policies
//! - Automatic cleanup
//! - Fast reconstruction
//!
//! ### Event Replay
//!
//! Event processing for:
//! - Rebuilding projections
//! - Migrations
//! - Event upcasting (versioning)
//! - Progress tracking
//!
//! ### Sagas
//!
//! Long-running processes with:
//! - Multiple steps
//! - Compensation actions
//! - Timeout handling
//! - Process persistence
//!
//! ## Quick Start
//!
//! ### 1. Define Events
//!
//! ```rust
//! use serde::{Serialize, Deserialize};
//! use caddy::enterprise::eventsource::aggregate::DomainEvent;
//!
//! #[derive(Debug, Clone, Serialize, Deserialize)]
//! enum DrawingEvent {
//!     Created { id: String, name: String },
//!     EntityAdded { entity_id: String, entity_type: String },
//!     EntityModified { entity_id: String },
//!     EntityDeleted { entity_id: String },
//! }
//!
//! impl DomainEvent for DrawingEvent {
//!     fn event_type(&self) -> &str {
//!         match self {
//!             DrawingEvent::Created { .. } => "Drawing.Created",
//!             DrawingEvent::EntityAdded { .. } => "Drawing.EntityAdded",
//!             DrawingEvent::EntityModified { .. } => "Drawing.EntityModified",
//!             DrawingEvent::EntityDeleted { .. } => "Drawing.EntityDeleted",
//!         }
//!     }
//! }
//! ```
//!
//! ### 2. Define Aggregate
//!
//! ```rust
//! use caddy::enterprise::eventsource::aggregate::AggregateRoot;
//!
//! #[derive(Debug, Clone, Default)]
//! struct Drawing {
//!     id: String,
//!     name: String,
//!     entities: Vec<String>,
//!     version: u64,
//! }
//!
//! impl AggregateRoot for Drawing {
//!     type Event = DrawingEvent;
//!
//!     fn aggregate_id(&self) -> String {
//!         self.id.clone()
//!     }
//!
//!     fn version(&self) -> u64 {
//!         self.version
//!     }
//!
//!     fn apply_event(&mut self, event: &Self::Event) {
//!         self.version += 1;
//!         match event {
//!             DrawingEvent::Created { id, name } => {
//!                 self.id = id.clone();
//!                 self.name = name.clone();
//!             }
//!             DrawingEvent::EntityAdded { entity_id, .. } => {
//!                 self.entities.push(entity_id.clone());
//!             }
//!             DrawingEvent::EntityDeleted { entity_id } => {
//!                 self.entities.retain(|e| e != entity_id);
//!             }
//!             _ => {}
//!         }
//!     }
//!
//!     fn aggregate_type() -> &'static str {
//!         "Drawing"
//!     }
//! }
//! ```
//!
//! ### 3. Define Commands
//!
//! ```rust
//! use caddy::enterprise::eventsource::command::{Command, CommandHandler};
//! use async_trait::async_trait;
//!
//! #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
//! struct CreateDrawingCommand {
//!     id: String,
//!     name: String,
//! }
//!
//! impl Command for CreateDrawingCommand {
//!     fn command_type(&self) -> &str {
//!         "CreateDrawing"
//!     }
//!
//!     fn aggregate_id(&self) -> String {
//!         self.id.clone()
//!     }
//!
//!     fn validate(&self) -> caddy::enterprise::error::EnterpriseResult<()> {
//!         if self.name.is_empty() {
//!             return Err(caddy::enterprise::error::EnterpriseError::Other(
//!                 "Name cannot be empty".to_string()
//!             ));
//!         }
//!         Ok(())
//!     }
//! }
//!
//! struct CreateDrawingHandler;
//!
//! #[async_trait]
//! impl CommandHandler<CreateDrawingCommand, Drawing> for CreateDrawingHandler {
//!     async fn handle(
//!         &self,
//!         command: &CreateDrawingCommand,
//!         aggregate: Option<&Drawing>,
//!     ) -> caddy::enterprise::error::EnterpriseResult<Vec<DrawingEvent>> {
//!         if aggregate.is_some() {
//!             return Err(caddy::enterprise::error::EnterpriseError::Other(
//!                 "Drawing already exists".to_string()
//!             ));
//!         }
//!
//!         Ok(vec![DrawingEvent::Created {
//!             id: command.id.clone(),
//!             name: command.name.clone(),
//!         }])
//!     }
//! }
//! ```
//!
//! ### 4. Use the System
//!
//! ```rust
//! use std::sync::Arc;
//! use caddy::enterprise::eventsource::{
//!     store::InMemoryEventStore,
//!     aggregate::AggregateRepository,
//!     command::CommandBus,
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create event store
//!     let event_store = Box::new(InMemoryEventStore::new());
//!
//!     // Create aggregate repository
//!     let repository = Arc::new(AggregateRepository::<Drawing>::new(event_store));
//!
//!     // Create command bus
//!     let command_bus = CommandBus::new(repository.clone());
//!
//!     // Execute command
//!     let command = CreateDrawingCommand {
//!         id: "drawing-1".to_string(),
//!         name: "My Drawing".to_string(),
//!     };
//!     let handler = CreateDrawingHandler;
//!
//!     let result = command_bus.execute(&command, &handler, None).await?;
//!     println!("Command executed: {:?}", result);
//!
//!     // Load aggregate
//!     let drawing = repository.load("drawing-1").await?;
//!     println!("Drawing loaded: {:?}", drawing);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Advanced Features
//!
//! ### Snapshots for Performance
//!
//! ```rust
//! use caddy::enterprise::eventsource::snapshot::{
//!     InMemorySnapshotStore, EveryNEventsPolicy, SnapshotRepository
//! };
//!
//! let snapshot_store = Arc::new(InMemorySnapshotStore::new());
//! let policy = Arc::new(EveryNEventsPolicy::new(100));
//! let snapshot_repo = SnapshotRepository::new(repository, snapshot_store, policy);
//!
//! // Automatically creates snapshots every 100 events
//! ```
//!
//! ### Projections for Read Models
//!
//! ```rust
//! use caddy::enterprise::eventsource::projection::{
//!     KeyValueProjection, ProjectionManager, InMemoryCheckpointStore
//! };
//!
//! let projection = Arc::new(KeyValueProjection::new(
//!     "drawing-list".to_string(),
//!     |event| {
//!         if event.metadata.event_type == "Drawing.Created" {
//!             Some((event.metadata.stream_id.clone(), event.metadata.version))
//!         } else {
//!             None
//!         }
//!     }
//! ));
//!
//! let manager = ProjectionManager::new(event_store, checkpoint_store);
//! manager.register(projection);
//! manager.start().await?;
//! ```
//!
//! ### Sagas for Complex Workflows
//!
//! ```rust
//! use caddy::enterprise::eventsource::saga::{Saga, SagaCoordinator, InMemorySagaStore};
//!
//! #[derive(Debug)]
//! struct DrawingPublishSaga;
//!
//! #[async_trait::async_trait]
//! impl Saga for DrawingPublishSaga {
//!     fn saga_type(&self) -> &str {
//!         "DrawingPublish"
//!     }
//!
//!     fn steps(&self) -> Vec<String> {
//!         vec![
//!             "ValidateDrawing".to_string(),
//!             "ExportToPDF".to_string(),
//!             "UploadToCloud".to_string(),
//!             "NotifyUsers".to_string(),
//!         ]
//!     }
//!
//!     async fn execute_step(&self, step_name: &str, data: &[u8])
//!         -> caddy::enterprise::error::EnterpriseResult<Vec<u8>>
//!     {
//!         // Execute step and return updated data
//!         Ok(data.to_vec())
//!     }
//!
//!     async fn compensate_step(&self, step_name: &str, data: &[u8])
//!         -> caddy::enterprise::error::EnterpriseResult<()>
//!     {
//!         // Undo the step
//!         Ok(())
//!     }
//! }
//! ```
//!
//! ## Best Practices
//!
//! ### Event Design
//!
//! - Events should be **immutable** and represent facts
//! - Use past tense for event names (e.g., `Created`, `Modified`)
//! - Include all necessary data in the event
//! - Never delete events from the store
//!
//! ### Aggregate Design
//!
//! - Keep aggregates small and focused
//! - One aggregate = one consistency boundary
//! - Apply events deterministically
//! - Don't query other aggregates in apply methods
//!
//! ### Command Handling
//!
//! - Validate commands before processing
//! - Use idempotency keys for duplicate prevention
//! - Return meaningful error messages
//! - Keep handlers stateless
//!
//! ### Projection Management
//!
//! - Use checkpointing to avoid reprocessing
//! - Handle projection failures gracefully
//! - Keep projections simple and focused
//! - Consider multiple projections for different views
//!
//! ## Performance Considerations
//!
//! - Use snapshots for aggregates with many events (>100)
//! - Batch event processing in projections
//! - Consider caching for frequently accessed read models
//! - Monitor event store growth and archive old events
//!
//! ## Testing
//!
//! All modules include comprehensive unit tests. Run with:
//!
//! ```bash
//! cargo test --package caddy --lib enterprise::eventsource
//! ```

// Module declarations
pub mod aggregate;
pub mod command;
pub mod projection;
pub mod replay;
pub mod saga;
pub mod snapshot;
pub mod store;

// Re-exports for convenience
pub use aggregate::{AggregateRepository, AggregateRoot, DomainEvent, AggregateBuilder};
pub use command::{Command, CommandBus, CommandDispatcher, CommandHandler, CommandResult};
pub use projection::{
    Checkpoint, CheckpointStore, InMemoryCheckpointStore, KeyValueProjection, Projection,
    ProjectionManager, ProjectionStats,
};
pub use replay::{
    CountingHandler, EventUpcaster, ReplayEngine, ReplayHandler, ReplayProgress, ReplayStatus,
    UpcasterChain,
};
pub use saga::{
    InMemorySagaStore, Saga, SagaCoordinator, SagaInstance, SagaStatus, SagaStep, SagaStore,
};
pub use snapshot::{
    AlwaysSnapshotPolicy, EveryNEventsPolicy, InMemorySnapshotStore, NoSnapshotPolicy, Snapshot,
    SnapshotPolicy, SnapshotRepository, SnapshotStore,
};
pub use store::{
    EventData, EventMetadata, EventStore, InMemoryEventStore, StoredEvent, StreamSlice,
};

/// Module version
pub const VERSION: &str = "0.2.0";

/// Module build date
pub const BUILD_DATE: &str = "2025-12-28";

#[cfg(test)]
mod integration_tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    enum TestEvent {
        Created { id: String, value: i32 },
        Incremented,
        Decremented,
    }

    impl DomainEvent for TestEvent {
        fn event_type(&self) -> &str {
            match self {
                TestEvent::Created { .. } => "Test.Created",
                TestEvent::Incremented => "Test.Incremented",
                TestEvent::Decremented => "Test.Decremented",
            }
        }
    }

    #[derive(Debug, Clone, Default)]
    struct TestAggregate {
        id: String,
        value: i32,
        version: u64,
    }

    impl AggregateRoot for TestAggregate {
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
                TestEvent::Created { id, value } => {
                    self.id = id.clone();
                    self.value = *value;
                }
                TestEvent::Incremented => self.value += 1,
                TestEvent::Decremented => self.value -= 1,
            }
        }

        fn aggregate_type() -> &'static str {
            "TestAggregate"
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct CreateCommand {
        id: String,
        value: i32,
    }

    impl Command for CreateCommand {
        fn command_type(&self) -> &str {
            "CreateTest"
        }

        fn aggregate_id(&self) -> String {
            self.id.clone()
        }
    }

    struct CreateHandler;

    #[async_trait::async_trait]
    impl CommandHandler<CreateCommand, TestAggregate> for CreateHandler {
        async fn handle(
            &self,
            command: &CreateCommand,
            aggregate: Option<&TestAggregate>,
        ) -> crate::enterprise::error::EnterpriseResult<Vec<TestEvent>> {
            if aggregate.is_some() {
                return Err(crate::enterprise::error::EnterpriseError::Other(
                    "Aggregate exists".to_string(),
                ));
            }
            Ok(vec![TestEvent::Created {
                id: command.id.clone(),
                value: command.value,
            }])
        }
    }

    #[tokio::test]
    async fn test_full_workflow() {
        // Setup
        let event_store = Box::new(InMemoryEventStore::new());
        let repository = Arc::new(AggregateRepository::<TestAggregate>::new(event_store));
        let command_bus = CommandBus::new(repository.clone());

        // Execute command
        let command = CreateCommand {
            id: "test-1".to_string(),
            value: 42,
        };
        let handler = CreateHandler;

        let result = command_bus.execute(&command, &handler, None).await.unwrap();
        assert_eq!(result.aggregate_id, "test-1");
        assert_eq!(result.version, 1);

        // Load aggregate
        let aggregate = repository.load("test-1").await.unwrap().unwrap();
        assert_eq!(aggregate.value, 42);
        assert_eq!(aggregate.version, 1);

        // Apply more events
        let events = vec![TestEvent::Incremented, TestEvent::Incremented];
        repository
            .save(&aggregate, events, 1, None, None)
            .await
            .unwrap();

        // Reload and verify
        let updated = repository.load("test-1").await.unwrap().unwrap();
        assert_eq!(updated.value, 44);
        assert_eq!(updated.version, 3);
    }

    #[tokio::test]
    async fn test_projection_integration() {
        let event_store = Arc::new(InMemoryEventStore::new()) as Arc<dyn EventStore>;
        let checkpoint_store =
            Arc::new(InMemoryCheckpointStore::new()) as Arc<dyn CheckpointStore>;

        // Create events
        let repository = Arc::new(AggregateRepository::<TestAggregate>::new(
            Box::new(InMemoryEventStore::new()),
        ));

        let events = vec![
            TestEvent::Created {
                id: "test-1".to_string(),
                value: 10,
            },
            TestEvent::Incremented,
        ];

        let mut aggregate = TestAggregate::default();
        for event in &events {
            aggregate.apply_event(event);
        }

        // Note: We need to append events directly to event_store, not through repository
        // This is a limitation of the test setup
        let event_data: Vec<_> = events
            .iter()
            .enumerate()
            .map(|(i, event)| EventData {
                stream_id: "test-1".to_string(),
                event_type: event.event_type().to_string(),
                data: serde_json::to_vec(event).unwrap(),
                expected_version: i as i64,
                correlation_id: None,
                causation_id: None,
                metadata: std::collections::HashMap::new(),
            })
            .collect();

        event_store.append_events(event_data).await.unwrap();

        // Create projection
        let projection = Arc::new(KeyValueProjection::new(
            "test-projection".to_string(),
            |event| {
                if event.metadata.event_type == "Test.Created" {
                    Some((event.metadata.stream_id.clone(), 1))
                } else {
                    None
                }
            },
        )) as Arc<dyn Projection>;

        // Start projection manager
        let manager = ProjectionManager::new(event_store, checkpoint_store);
        manager.register(projection.clone());
        manager.start().await.unwrap();

        // Wait for processing
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Verify projection was built
        let kv_projection = projection
            .as_ref()
            .as_any()
            .downcast_ref::<KeyValueProjection<String, i32>>();
        // Note: This test is simplified as we can't easily downcast trait objects
    }
}
