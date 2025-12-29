//! Snapshot Management
//!
//! Provides snapshot storage and management for optimizing aggregate loading
//! by storing periodic state snapshots instead of replaying all events.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::sync::Arc;

use super::aggregate::{AggregateRepository, AggregateRoot, DomainEvent};
use crate::enterprise::error::{EnterpriseError, EnterpriseResult};

/// Snapshot of an aggregate at a specific version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Aggregate ID
    pub aggregate_id: String,
    /// Aggregate type
    pub aggregate_type: String,
    /// Version at which this snapshot was taken
    pub version: u64,
    /// Serialized aggregate state
    pub data: Vec<u8>,
    /// Timestamp when snapshot was created
    pub timestamp: DateTime<Utc>,
    /// Optional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl Snapshot {
    /// Create a new snapshot
    pub fn new(aggregate_id: String, aggregate_type: String, version: u64, data: Vec<u8>) -> Self {
        Self {
            aggregate_id,
            aggregate_type,
            version,
            data,
            timestamp: Utc::now(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Add metadata to the snapshot
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Trait for snapshot storage backends
#[async_trait]
pub trait SnapshotStore: Send + Sync {
    /// Save a snapshot
    async fn save(&self, snapshot: &Snapshot) -> EnterpriseResult<()>;

    /// Load the latest snapshot for an aggregate
    async fn load(&self, aggregate_id: &str) -> EnterpriseResult<Option<Snapshot>>;

    /// Load a snapshot at or before a specific version
    async fn load_at_version(
        &self,
        aggregate_id: &str,
        version: u64,
    ) -> EnterpriseResult<Option<Snapshot>>;

    /// Delete all snapshots for an aggregate
    async fn delete(&self, aggregate_id: &str) -> EnterpriseResult<()>;

    /// Delete old snapshots (keep only the most recent N)
    async fn cleanup(&self, aggregate_id: &str, keep_count: usize) -> EnterpriseResult<()>;

    /// Get all snapshot versions for an aggregate
    async fn get_versions(&self, aggregate_id: &str) -> EnterpriseResult<Vec<u64>>;
}

/// In-memory snapshot store for testing
pub struct InMemorySnapshotStore {
    snapshots: Arc<DashMap<String, Vec<Snapshot>>>,
}

impl InMemorySnapshotStore {
    /// Create a new in-memory snapshot store
    pub fn new() -> Self {
        Self {
            snapshots: Arc::new(DashMap::new()),
        }
    }

    /// Clear all snapshots (useful for testing)
    pub fn clear(&self) {
        self.snapshots.clear();
    }
}

impl Default for InMemorySnapshotStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SnapshotStore for InMemorySnapshotStore {
    async fn save(&self, snapshot: &Snapshot) -> EnterpriseResult<()> {
        let mut snapshots = self
            .snapshots
            .entry(snapshot.aggregate_id.clone())
            .or_insert_with(Vec::new);

        // Remove existing snapshot at this version if it exists
        snapshots.retain(|s| s.version != snapshot.version);

        // Insert new snapshot and sort by version
        snapshots.push(snapshot.clone());
        snapshots.sort_by_key(|s| s.version);

        Ok(())
    }

    async fn load(&self, aggregate_id: &str) -> EnterpriseResult<Option<Snapshot>> {
        Ok(self
            .snapshots
            .get(aggregate_id)
            .and_then(|snapshots| snapshots.last().cloned()))
    }

    async fn load_at_version(
        &self,
        aggregate_id: &str,
        version: u64,
    ) -> EnterpriseResult<Option<Snapshot>> {
        Ok(self.snapshots.get(aggregate_id).and_then(|snapshots| {
            snapshots
                .iter()
                .rev()
                .find(|s| s.version <= version)
                .cloned()
        }))
    }

    async fn delete(&self, aggregate_id: &str) -> EnterpriseResult<()> {
        self.snapshots.remove(aggregate_id);
        Ok(())
    }

    async fn cleanup(&self, aggregate_id: &str, keep_count: usize) -> EnterpriseResult<()> {
        if let Some(mut entry) = self.snapshots.get_mut(aggregate_id) {
            if entry.len() > keep_count {
                let skip = entry.len() - keep_count;
                *entry = entry.iter().skip(skip).cloned().collect();
            }
        }
        Ok(())
    }

    async fn get_versions(&self, aggregate_id: &str) -> EnterpriseResult<Vec<u64>> {
        Ok(self
            .snapshots
            .get(aggregate_id)
            .map(|snapshots| snapshots.iter().map(|s| s.version).collect())
            .unwrap_or_default())
    }
}

/// Policy for determining when to take snapshots
pub trait SnapshotPolicy: Send + Sync {
    /// Check if a snapshot should be taken
    fn should_snapshot(&self, current_version: u64, last_snapshot_version: Option<u64>) -> bool;
}

/// Snapshot every N events
pub struct EveryNEventsPolicy {
    interval: u64,
}

impl EveryNEventsPolicy {
    /// Create a new policy that snapshots every N events
    pub fn new(interval: u64) -> Self {
        Self { interval }
    }
}

impl SnapshotPolicy for EveryNEventsPolicy {
    fn should_snapshot(&self, current_version: u64, last_snapshot_version: Option<u64>) -> bool {
        if current_version == 0 {
            return false;
        }

        match last_snapshot_version {
            None => current_version >= self.interval,
            Some(last) => current_version - last >= self.interval,
        }
    }
}

/// Never take snapshots
pub struct NoSnapshotPolicy;

impl SnapshotPolicy for NoSnapshotPolicy {
    fn should_snapshot(&self, _current_version: u64, _last_snapshot_version: Option<u64>) -> bool {
        false
    }
}

/// Always take snapshots (useful for testing)
pub struct AlwaysSnapshotPolicy;

impl SnapshotPolicy for AlwaysSnapshotPolicy {
    fn should_snapshot(&self, current_version: u64, _last_snapshot_version: Option<u64>) -> bool {
        current_version > 0
    }
}

/// Repository that uses snapshots for optimized loading
pub struct SnapshotRepository<A: AggregateRoot + Serialize + DeserializeOwned> {
    aggregate_repo: Arc<AggregateRepository<A>>,
    snapshot_store: Arc<dyn SnapshotStore>,
    snapshot_policy: Arc<dyn SnapshotPolicy>,
}

impl<A: AggregateRoot + Serialize + DeserializeOwned> SnapshotRepository<A> {
    /// Create a new snapshot repository
    pub fn new(
        aggregate_repo: Arc<AggregateRepository<A>>,
        snapshot_store: Arc<dyn SnapshotStore>,
        snapshot_policy: Arc<dyn SnapshotPolicy>,
    ) -> Self {
        Self {
            aggregate_repo,
            snapshot_store,
            snapshot_policy,
        }
    }

    /// Load an aggregate using snapshots
    ///
    /// This will load the latest snapshot (if available) and replay
    /// only the events since the snapshot.
    pub async fn load(&self, aggregate_id: &str) -> EnterpriseResult<Option<A>> {
        // Try to load the latest snapshot
        if let Some(snapshot) = self.snapshot_store.load(aggregate_id).await? {
            // Deserialize aggregate from snapshot
            let mut aggregate: A = serde_json::from_slice(&snapshot.data)
                .map_err(|e| EnterpriseError::Other(format!("Failed to deserialize: {}", e)))?;

            // Load events since the snapshot
            let current_version = self.aggregate_repo.get_version(aggregate_id).await?;

            if current_version > snapshot.version {
                // Replay events from snapshot version to current
                let slice = self
                    .aggregate_repo
                    .event_store
                    .read_stream(
                        aggregate_id,
                        snapshot.version + 1,
                        (current_version - snapshot.version) as usize,
                    )
                    .await?;

                for stored_event in &slice.events {
                    let event = A::Event::from_bytes(&stored_event.data)?;
                    aggregate.apply_event(&event);
                }
            }

            Ok(Some(aggregate))
        } else {
            // No snapshot, fall back to loading all events
            self.aggregate_repo.load(aggregate_id).await
        }
    }

    /// Save an aggregate and potentially create a snapshot
    pub async fn save(
        &self,
        aggregate: &A,
        events: Vec<A::Event>,
        expected_version: u64,
        correlation_id: Option<uuid::Uuid>,
        causation_id: Option<uuid::Uuid>,
    ) -> EnterpriseResult<()> {
        // Save events
        self.aggregate_repo
            .save(aggregate, events, expected_version, correlation_id, causation_id)
            .await?;

        // Check if we should snapshot
        let current_version = aggregate.version();
        let last_snapshot = self.snapshot_store.load(&aggregate.aggregate_id()).await?;
        let last_snapshot_version = last_snapshot.map(|s| s.version);

        if self
            .snapshot_policy
            .should_snapshot(current_version, last_snapshot_version)
        {
            self.create_snapshot(aggregate).await?;
        }

        Ok(())
    }

    /// Manually create a snapshot of an aggregate
    pub async fn create_snapshot(&self, aggregate: &A) -> EnterpriseResult<()> {
        let data = serde_json::to_vec(aggregate)
            .map_err(|e| EnterpriseError::Other(format!("Failed to serialize: {}", e)))?;

        let snapshot = Snapshot::new(
            aggregate.aggregate_id(),
            A::aggregate_type().to_string(),
            aggregate.version(),
            data,
        );

        self.snapshot_store.save(&snapshot).await
    }

    /// Delete all snapshots for an aggregate
    pub async fn delete_snapshots(&self, aggregate_id: &str) -> EnterpriseResult<()> {
        self.snapshot_store.delete(aggregate_id).await
    }

    /// Clean up old snapshots (keep only the most recent N)
    pub async fn cleanup_snapshots(
        &self,
        aggregate_id: &str,
        keep_count: usize,
    ) -> EnterpriseResult<()> {
        self.snapshot_store.cleanup(aggregate_id, keep_count).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enterprise::eventsource::{
        aggregate::AggregateRepository, store::InMemoryEventStore,
    };
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    enum TestEvent {
        Created { id: String, value: i32 },
        Incremented,
    }

    impl super::super::aggregate::DomainEvent for TestEvent {
        fn event_type(&self) -> &str {
            match self {
                TestEvent::Created { .. } => "Test.Created",
                TestEvent::Incremented => "Test.Incremented",
            }
        }
    }

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    struct TestAggregate {
        id: String,
        value: i32,
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
                TestEvent::Created { id, value } => {
                    self.id = id.clone();
                    self.value = *value;
                }
                TestEvent::Incremented => {
                    self.value += 1;
                }
            }
        }

        fn aggregate_type() -> &'static str {
            "TestAggregate"
        }
    }

    #[tokio::test]
    async fn test_snapshot_save_and_load() {
        let store = InMemorySnapshotStore::new();
        let snapshot = Snapshot::new("test-1".to_string(), "Test".to_string(), 5, vec![1, 2, 3]);

        store.save(&snapshot).await.unwrap();

        let loaded = store.load("test-1").await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().version, 5);
    }

    #[tokio::test]
    async fn test_snapshot_multiple_versions() {
        let store = InMemorySnapshotStore::new();

        store
            .save(&Snapshot::new(
                "test-1".to_string(),
                "Test".to_string(),
                5,
                vec![1],
            ))
            .await
            .unwrap();
        store
            .save(&Snapshot::new(
                "test-1".to_string(),
                "Test".to_string(),
                10,
                vec![2],
            ))
            .await
            .unwrap();
        store
            .save(&Snapshot::new(
                "test-1".to_string(),
                "Test".to_string(),
                15,
                vec![3],
            ))
            .await
            .unwrap();

        let latest = store.load("test-1").await.unwrap().unwrap();
        assert_eq!(latest.version, 15);

        let at_version = store.load_at_version("test-1", 12).await.unwrap().unwrap();
        assert_eq!(at_version.version, 10);
    }

    #[tokio::test]
    async fn test_snapshot_cleanup() {
        let store = InMemorySnapshotStore::new();

        for i in 1..=10 {
            store
                .save(&Snapshot::new(
                    "test-1".to_string(),
                    "Test".to_string(),
                    i,
                    vec![i as u8],
                ))
                .await
                .unwrap();
        }

        let versions = store.get_versions("test-1").await.unwrap();
        assert_eq!(versions.len(), 10);

        store.cleanup("test-1", 3).await.unwrap();

        let versions_after = store.get_versions("test-1").await.unwrap();
        assert_eq!(versions_after.len(), 3);
        assert_eq!(versions_after, vec![8, 9, 10]);
    }

    #[tokio::test]
    async fn test_every_n_events_policy() {
        let policy = EveryNEventsPolicy::new(5);

        assert!(!policy.should_snapshot(0, None));
        assert!(!policy.should_snapshot(4, None));
        assert!(policy.should_snapshot(5, None));
        assert!(policy.should_snapshot(10, Some(5)));
        assert!(!policy.should_snapshot(8, Some(5)));
    }

    #[tokio::test]
    async fn test_snapshot_repository() {
        let event_store = Box::new(InMemoryEventStore::new());
        let aggregate_repo = Arc::new(AggregateRepository::<TestAggregate>::new(event_store));
        let snapshot_store = Arc::new(InMemorySnapshotStore::new()) as Arc<dyn SnapshotStore>;
        let policy = Arc::new(EveryNEventsPolicy::new(3)) as Arc<dyn SnapshotPolicy>;

        let repo = SnapshotRepository::new(aggregate_repo.clone(), snapshot_store.clone(), policy);

        // Create aggregate with 5 events (should create snapshot at version 3)
        let events = vec![
            TestEvent::Created {
                id: "test-1".to_string(),
                value: 0,
            },
            TestEvent::Incremented,
            TestEvent::Incremented,
            TestEvent::Incremented,
            TestEvent::Incremented,
        ];

        let mut aggregate = TestAggregate::default();
        for event in &events {
            aggregate.apply_event(event);
        }

        repo.save(&aggregate, events, 0, None, None).await.unwrap();

        // Should have a snapshot at version 3
        let snapshot = snapshot_store.load("test-1").await.unwrap().unwrap();
        assert_eq!(snapshot.version, 3);

        // Load using snapshot
        let loaded = repo.load("test-1").await.unwrap().unwrap();
        assert_eq!(loaded.value, 4);
        assert_eq!(loaded.version, 5);
    }

    #[tokio::test]
    async fn test_no_snapshot_policy() {
        let policy = NoSnapshotPolicy;
        assert!(!policy.should_snapshot(0, None));
        assert!(!policy.should_snapshot(100, None));
        assert!(!policy.should_snapshot(1000, Some(900)));
    }

    #[tokio::test]
    async fn test_always_snapshot_policy() {
        let policy = AlwaysSnapshotPolicy;
        assert!(!policy.should_snapshot(0, None));
        assert!(policy.should_snapshot(1, None));
        assert!(policy.should_snapshot(100, Some(99)));
    }
}
