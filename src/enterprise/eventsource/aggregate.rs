//! Aggregate Root Implementation
//!
//! Provides traits and utilities for implementing aggregates in event sourcing,
//! including event application, state reconstruction, and optimistic concurrency.

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use uuid::Uuid;

use super::store::{EventData, EventStore, StoredEvent};
use crate::enterprise::error::{EnterpriseError, EnterpriseResult};

/// Trait for domain events that can be applied to aggregates
pub trait DomainEvent: Clone + Serialize + DeserializeOwned + Send + Sync + Debug {
    /// Get the event type identifier
    fn event_type(&self) -> &str;

    /// Serialize the event to bytes
    fn to_bytes(&self) -> EnterpriseResult<Vec<u8>> {
        serde_json::to_vec(self)
            .map_err(|e| EnterpriseError::Other(format!("Failed to serialize event: {}", e)))
    }

    /// Deserialize the event from bytes
    fn from_bytes(data: &[u8]) -> EnterpriseResult<Self> {
        serde_json::from_slice(data)
            .map_err(|e| EnterpriseError::Other(format!("Failed to deserialize event: {}", e)))
    }
}

/// Trait for aggregate roots in event sourcing
#[async_trait]
pub trait AggregateRoot: Default + Clone + Send + Sync + Debug {
    /// The type of events this aggregate produces
    type Event: DomainEvent;

    /// Get the aggregate ID
    fn aggregate_id(&self) -> String;

    /// Get the current version of the aggregate
    fn version(&self) -> u64;

    /// Apply an event to the aggregate state
    ///
    /// This method should update the aggregate's state based on the event.
    /// It should be deterministic and have no side effects.
    fn apply_event(&mut self, event: &Self::Event);

    /// Get the aggregate type name
    fn aggregate_type() -> &'static str
    where
        Self: Sized;
}

/// Repository for loading and saving aggregates from/to the event store
pub struct AggregateRepository<A: AggregateRoot> {
    pub event_store: Box<dyn EventStore>,
    _phantom: std::marker::PhantomData<A>,
}

impl<A: AggregateRoot> AggregateRepository<A> {
    /// Create a new aggregate repository
    pub fn new(event_store: Box<dyn EventStore>) -> Self {
        Self {
            event_store,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Load an aggregate from the event store by replaying all events
    ///
    /// # Arguments
    ///
    /// * `aggregate_id` - The ID of the aggregate to load
    ///
    /// # Returns
    ///
    /// The reconstructed aggregate with all events applied
    pub async fn load(&self, aggregate_id: &str) -> EnterpriseResult<Option<A>> {
        let slice = self.event_store.read_stream_all(aggregate_id).await?;

        if slice.events.is_empty() {
            return Ok(None);
        }

        let mut aggregate = A::default();

        for stored_event in &slice.events {
            let event = A::Event::from_bytes(&stored_event.data)?;
            aggregate.apply_event(&event);
        }

        Ok(Some(aggregate))
    }

    /// Load an aggregate up to a specific version
    ///
    /// # Arguments
    ///
    /// * `aggregate_id` - The ID of the aggregate to load
    /// * `version` - The version to load up to (inclusive)
    pub async fn load_at_version(
        &self,
        aggregate_id: &str,
        version: u64,
    ) -> EnterpriseResult<Option<A>> {
        let slice = self
            .event_store
            .read_stream(aggregate_id, 0, version as usize)
            .await?;

        if slice.events.is_empty() {
            return Ok(None);
        }

        let mut aggregate = A::default();

        for stored_event in &slice.events {
            if stored_event.metadata.version > version {
                break;
            }
            let event = A::Event::from_bytes(&stored_event.data)?;
            aggregate.apply_event(&event);
        }

        Ok(Some(aggregate))
    }

    /// Save new events for an aggregate with optimistic concurrency control
    ///
    /// # Arguments
    ///
    /// * `aggregate` - The aggregate to save
    /// * `events` - New events to append
    /// * `expected_version` - Expected current version for concurrency control
    /// * `correlation_id` - Optional correlation ID
    /// * `causation_id` - Optional causation ID (e.g., command ID)
    pub async fn save(
        &self,
        aggregate: &A,
        events: Vec<A::Event>,
        expected_version: u64,
        correlation_id: Option<Uuid>,
        causation_id: Option<Uuid>,
    ) -> EnterpriseResult<Vec<StoredEvent>> {
        if events.is_empty() {
            return Ok(Vec::new());
        }

        let aggregate_id = aggregate.aggregate_id();
        let mut event_data_list = Vec::new();

        for event in events {
            let event_data = EventData {
                stream_id: aggregate_id.clone(),
                event_type: event.event_type().to_string(),
                data: event.to_bytes()?,
                expected_version: expected_version as i64,
                correlation_id,
                causation_id,
                metadata: std::collections::HashMap::new(),
            };
            event_data_list.push(event_data);
        }

        self.event_store.append_events(event_data_list).await
    }

    /// Check if an aggregate exists
    pub async fn exists(&self, aggregate_id: &str) -> EnterpriseResult<bool> {
        self.event_store.stream_exists(aggregate_id).await
    }

    /// Get the current version of an aggregate without loading it
    pub async fn get_version(&self, aggregate_id: &str) -> EnterpriseResult<u64> {
        self.event_store.get_stream_version(aggregate_id).await
    }
}

/// Helper for building aggregates with a fluent API
pub struct AggregateBuilder<A: AggregateRoot> {
    aggregate: A,
    uncommitted_events: Vec<A::Event>,
}

impl<A: AggregateRoot> AggregateBuilder<A> {
    /// Create a new builder with a default aggregate
    pub fn new() -> Self {
        Self {
            aggregate: A::default(),
            uncommitted_events: Vec::new(),
        }
    }

    /// Create a builder from an existing aggregate
    pub fn from_aggregate(aggregate: A) -> Self {
        Self {
            aggregate,
            uncommitted_events: Vec::new(),
        }
    }

    /// Apply and record an event
    pub fn apply(mut self, event: A::Event) -> Self {
        self.aggregate.apply_event(&event);
        self.uncommitted_events.push(event);
        self
    }

    /// Apply multiple events
    pub fn apply_many(mut self, events: Vec<A::Event>) -> Self {
        for event in events {
            self.aggregate.apply_event(&event);
            self.uncommitted_events.push(event);
        }
        self
    }

    /// Get the current aggregate state
    pub fn aggregate(&self) -> &A {
        &self.aggregate
    }

    /// Get uncommitted events
    pub fn uncommitted_events(&self) -> &[A::Event] {
        &self.uncommitted_events
    }

    /// Build and return the aggregate and uncommitted events
    pub fn build(self) -> (A, Vec<A::Event>) {
        (self.aggregate, self.uncommitted_events)
    }

    /// Save the aggregate to the repository
    pub async fn save(
        self,
        repository: &AggregateRepository<A>,
        correlation_id: Option<Uuid>,
        causation_id: Option<Uuid>,
    ) -> EnterpriseResult<Vec<StoredEvent>> {
        let expected_version = self.aggregate.version();
        repository
            .save(
                &self.aggregate,
                self.uncommitted_events,
                expected_version,
                correlation_id,
                causation_id,
            )
            .await
    }
}

impl<A: AggregateRoot> Default for AggregateBuilder<A> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enterprise::eventsource::store::InMemoryEventStore;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    enum TestEvent {
        Created { id: String, name: String },
        NameChanged { name: String },
        Deleted,
    }

    impl DomainEvent for TestEvent {
        fn event_type(&self) -> &str {
            match self {
                TestEvent::Created { .. } => "TestAggregate.Created",
                TestEvent::NameChanged { .. } => "TestAggregate.NameChanged",
                TestEvent::Deleted => "TestAggregate.Deleted",
            }
        }
    }

    #[derive(Debug, Clone, Default)]
    struct TestAggregate {
        id: String,
        name: String,
        version: u64,
        deleted: bool,
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
                TestEvent::Created { id, name } => {
                    self.id = id.clone();
                    self.name = name.clone();
                }
                TestEvent::NameChanged { name } => {
                    self.name = name.clone();
                }
                TestEvent::Deleted => {
                    self.deleted = true;
                }
            }
        }

        fn aggregate_type() -> &'static str {
            "TestAggregate"
        }
    }

    #[tokio::test]
    async fn test_aggregate_save_and_load() {
        let store = Box::new(InMemoryEventStore::new());
        let repo = AggregateRepository::<TestAggregate>::new(store);

        let events = vec![
            TestEvent::Created {
                id: "test-1".to_string(),
                name: "Test".to_string(),
            },
            TestEvent::NameChanged {
                name: "Updated".to_string(),
            },
        ];

        let mut aggregate = TestAggregate::default();
        for event in &events {
            aggregate.apply_event(event);
        }

        repo.save(&aggregate, events, 0, None, None).await.unwrap();

        let loaded = repo.load("test-1").await.unwrap().unwrap();
        assert_eq!(loaded.name, "Updated");
        assert_eq!(loaded.version, 2);
    }

    #[tokio::test]
    async fn test_load_at_version() {
        let store = Box::new(InMemoryEventStore::new());
        let repo = AggregateRepository::<TestAggregate>::new(store);

        let events = vec![
            TestEvent::Created {
                id: "test-1".to_string(),
                name: "Test".to_string(),
            },
            TestEvent::NameChanged {
                name: "Updated".to_string(),
            },
            TestEvent::NameChanged {
                name: "Final".to_string(),
            },
        ];

        let mut aggregate = TestAggregate::default();
        for event in &events {
            aggregate.apply_event(event);
        }

        repo.save(&aggregate, events, 0, None, None).await.unwrap();

        let loaded_v1 = repo.load_at_version("test-1", 1).await.unwrap().unwrap();
        assert_eq!(loaded_v1.name, "Test");
        assert_eq!(loaded_v1.version, 1);

        let loaded_v2 = repo.load_at_version("test-1", 2).await.unwrap().unwrap();
        assert_eq!(loaded_v2.name, "Updated");
        assert_eq!(loaded_v2.version, 2);
    }

    #[tokio::test]
    async fn test_aggregate_builder() {
        let (aggregate, events) = AggregateBuilder::<TestAggregate>::new()
            .apply(TestEvent::Created {
                id: "test-1".to_string(),
                name: "Test".to_string(),
            })
            .apply(TestEvent::NameChanged {
                name: "Updated".to_string(),
            })
            .build();

        assert_eq!(aggregate.name, "Updated");
        assert_eq!(aggregate.version, 2);
        assert_eq!(events.len(), 2);
    }

    #[tokio::test]
    async fn test_optimistic_concurrency() {
        let store = Box::new(InMemoryEventStore::new());
        let repo = AggregateRepository::<TestAggregate>::new(store);

        let event1 = vec![TestEvent::Created {
            id: "test-1".to_string(),
            name: "Test".to_string(),
        }];

        let mut agg1 = TestAggregate::default();
        agg1.apply_event(&event1[0]);

        repo.save(&agg1, event1, 0, None, None).await.unwrap();

        // Try to save with wrong version
        let event2 = vec![TestEvent::NameChanged {
            name: "Wrong".to_string(),
        }];
        let result = repo.save(&agg1, event2, 0, None, None).await;
        assert!(result.is_err());

        // Save with correct version
        let event3 = vec![TestEvent::NameChanged {
            name: "Correct".to_string(),
        }];
        let mut agg2 = TestAggregate::default();
        agg2.id = "test-1".to_string();
        agg2.version = 1;

        repo.save(&agg2, event3, 1, None, None).await.unwrap();
    }
}
