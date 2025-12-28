//! Projection and Read Model Management
//!
//! Provides infrastructure for building and maintaining read models (projections)
//! from event streams with support for catch-up subscriptions and checkpointing.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

use super::store::{EventStore, StoredEvent};
use crate::enterprise::error::{EnterpriseError, EnterpriseResult};

/// Checkpoint for tracking projection progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Projection name
    pub projection_name: String,
    /// Last processed global sequence number
    pub last_sequence: u64,
    /// Timestamp of last update
    pub timestamp: DateTime<Utc>,
    /// Optional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl Checkpoint {
    /// Create a new checkpoint
    pub fn new(projection_name: String, last_sequence: u64) -> Self {
        Self {
            projection_name,
            last_sequence,
            timestamp: Utc::now(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Update the checkpoint sequence
    pub fn update(&mut self, sequence: u64) {
        self.last_sequence = sequence;
        self.timestamp = Utc::now();
    }
}

/// Trait for checkpoint storage
#[async_trait]
pub trait CheckpointStore: Send + Sync {
    /// Save a checkpoint
    async fn save(&self, checkpoint: &Checkpoint) -> EnterpriseResult<()>;

    /// Load a checkpoint
    async fn load(&self, projection_name: &str) -> EnterpriseResult<Option<Checkpoint>>;

    /// Delete a checkpoint (for projection reset)
    async fn delete(&self, projection_name: &str) -> EnterpriseResult<()>;
}

/// In-memory checkpoint store for testing
pub struct InMemoryCheckpointStore {
    checkpoints: Arc<DashMap<String, Checkpoint>>,
}

impl InMemoryCheckpointStore {
    /// Create a new in-memory checkpoint store
    pub fn new() -> Self {
        Self {
            checkpoints: Arc::new(DashMap::new()),
        }
    }
}

impl Default for InMemoryCheckpointStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CheckpointStore for InMemoryCheckpointStore {
    async fn save(&self, checkpoint: &Checkpoint) -> EnterpriseResult<()> {
        self.checkpoints
            .insert(checkpoint.projection_name.clone(), checkpoint.clone());
        Ok(())
    }

    async fn load(&self, projection_name: &str) -> EnterpriseResult<Option<Checkpoint>> {
        Ok(self.checkpoints.get(projection_name).map(|c| c.clone()))
    }

    async fn delete(&self, projection_name: &str) -> EnterpriseResult<()> {
        self.checkpoints.remove(projection_name);
        Ok(())
    }
}

/// Trait for projections that process events
#[async_trait]
pub trait Projection: Send + Sync {
    /// Get the projection name
    fn name(&self) -> &str;

    /// Handle an event
    async fn handle(&self, event: &StoredEvent) -> EnterpriseResult<()>;

    /// Reset the projection (clear all state)
    async fn reset(&self) -> EnterpriseResult<()>;

    /// Get projection statistics
    async fn stats(&self) -> ProjectionStats {
        ProjectionStats::default()
    }
}

/// Statistics about a projection
#[derive(Debug, Clone, Default)]
pub struct ProjectionStats {
    /// Number of events processed
    pub events_processed: u64,
    /// Number of errors encountered
    pub errors: u64,
    /// Last processing time in milliseconds
    pub last_processing_time_ms: u64,
    /// Average processing time in milliseconds
    pub avg_processing_time_ms: u64,
}

/// Projection manager for running and managing projections
pub struct ProjectionManager {
    event_store: Arc<dyn EventStore>,
    checkpoint_store: Arc<dyn CheckpointStore>,
    projections: Arc<DashMap<String, Arc<dyn Projection + 'static>>>,
    running: Arc<RwLock<bool>>,
}

impl ProjectionManager {
    /// Create a new projection manager
    pub fn new(
        event_store: Arc<dyn EventStore>,
        checkpoint_store: Arc<dyn CheckpointStore>,
    ) -> Self {
        Self {
            event_store,
            checkpoint_store,
            projections: Arc::new(DashMap::new()),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Register a projection
    pub fn register(&self, projection: Arc<dyn Projection + 'static>) {
        self.projections
            .insert(projection.name().to_string(), projection);
    }

    /// Start all projections (catch-up and live updates)
    pub async fn start(&self) -> EnterpriseResult<()> {
        let mut running = self.running.write().await;
        if *running {
            return Err(EnterpriseError::Other(
                "Projection manager already running".to_string(),
            ));
        }
        *running = true;
        drop(running);

        // Start catch-up for all projections
        for entry in self.projections.iter() {
            let projection = entry.value().clone();
            self.catch_up_projection(projection).await?;
        }

        // Start live subscription
        self.start_live_subscription();

        Ok(())
    }

    /// Stop all projections
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
    }

    /// Run catch-up for a specific projection
    async fn catch_up_projection(&self, projection: Arc<dyn Projection + 'static>) -> EnterpriseResult<()> {
        let projection_name = projection.name();

        // Load checkpoint
        let checkpoint = self
            .checkpoint_store
            .load(projection_name)
            .await?
            .unwrap_or_else(|| Checkpoint::new(projection_name.to_string(), 0));

        let mut last_sequence = checkpoint.last_sequence;

        // Process events in batches
        const BATCH_SIZE: usize = 100;
        loop {
            let events = self
                .event_store
                .read_all(last_sequence + 1, BATCH_SIZE)
                .await?;

            if events.is_empty() {
                break;
            }

            for event in events {
                projection.handle(&event).await?;
                last_sequence = event.metadata.sequence;
            }

            // Save checkpoint after each batch
            let mut updated_checkpoint = checkpoint.clone();
            updated_checkpoint.update(last_sequence);
            self.checkpoint_store.save(&updated_checkpoint).await?;
        }

        Ok(())
    }

    /// Start live subscription to process new events as they arrive
    fn start_live_subscription(&self) {
        let event_store = self.event_store.clone();
        let checkpoint_store = self.checkpoint_store.clone();
        let projections = self.projections.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(100));

            loop {
                ticker.tick().await;

                let is_running = *running.read().await;
                if !is_running {
                    break;
                }

                // Collect all projections into a Vec to avoid lifetime issues with DashMap
                let projection_list: Vec<Arc<dyn Projection + 'static>> = projections
                    .iter()
                    .map(|entry| entry.value().clone())
                    .collect();

                // Process new events for each projection
                for projection in projection_list {
                    let projection_name = projection.name();

                    // Load checkpoint
                    let checkpoint = match checkpoint_store.load(projection_name).await {
                        Ok(Some(cp)) => cp,
                        Ok(None) => Checkpoint::new(projection_name.to_string(), 0),
                        Err(_) => continue,
                    };

                    // Read new events
                    let events = match event_store.read_all(checkpoint.last_sequence + 1, 10).await
                    {
                        Ok(e) => e,
                        Err(_) => continue,
                    };

                    if events.is_empty() {
                        continue;
                    }

                    // Process events
                    let mut last_sequence = checkpoint.last_sequence;
                    for event in events {
                        if let Err(_e) = projection.handle(&event).await {
                            // Log error but continue processing
                            continue;
                        }
                        last_sequence = event.metadata.sequence;
                    }

                    // Update checkpoint
                    let mut updated = checkpoint;
                    updated.update(last_sequence);
                    let _ = checkpoint_store.save(&updated).await;
                }
            }
        });
    }

    /// Reset a projection (clear state and checkpoint)
    pub async fn reset_projection(&self, projection_name: &str) -> EnterpriseResult<()> {
        if let Some(entry) = self.projections.get(projection_name) {
            let projection = entry.value();
            projection.reset().await?;
            self.checkpoint_store.delete(projection_name).await?;
            Ok(())
        } else {
            Err(EnterpriseError::Other(format!(
                "Projection not found: {}",
                projection_name
            )))
        }
    }

    /// Rebuild a projection from the beginning
    pub async fn rebuild_projection(&self, projection_name: &str) -> EnterpriseResult<()> {
        if let Some(entry) = self.projections.get(projection_name) {
            let projection = entry.value().clone();

            // Reset the projection
            projection.reset().await?;
            self.checkpoint_store.delete(projection_name).await?;

            // Run catch-up
            self.catch_up_projection(projection).await?;

            Ok(())
        } else {
            Err(EnterpriseError::Other(format!(
                "Projection not found: {}",
                projection_name
            )))
        }
    }

    /// Get statistics for a projection
    pub async fn get_stats(&self, projection_name: &str) -> EnterpriseResult<ProjectionStats> {
        if let Some(entry) = self.projections.get(projection_name) {
            Ok(entry.value().stats().await)
        } else {
            Err(EnterpriseError::Other(format!(
                "Projection not found: {}",
                projection_name
            )))
        }
    }
}

/// Simple key-value projection for maintaining read models
pub struct KeyValueProjection<K, V>
where
    K: std::hash::Hash + Eq + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    name: String,
    data: Arc<DashMap<K, V>>,
    handler: Arc<dyn Fn(&StoredEvent) -> Option<(K, V)> + Send + Sync>,
}

impl<K, V> KeyValueProjection<K, V>
where
    K: std::hash::Hash + Eq + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    /// Create a new key-value projection
    pub fn new<F>(name: String, handler: F) -> Self
    where
        F: Fn(&StoredEvent) -> Option<(K, V)> + Send + Sync + 'static,
    {
        Self {
            name,
            data: Arc::new(DashMap::new()),
            handler: Arc::new(handler),
        }
    }

    /// Get a value by key
    pub fn get(&self, key: &K) -> Option<V> {
        self.data.get(key).map(|entry| entry.value().clone())
    }

    /// Get all keys
    pub fn keys(&self) -> Vec<K> {
        self.data.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Get all values
    pub fn values(&self) -> Vec<V> {
        self.data
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get the size of the projection
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the projection is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[async_trait]
impl<K, V> Projection for KeyValueProjection<K, V>
where
    K: std::hash::Hash + Eq + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    fn name(&self) -> &str {
        &self.name
    }

    async fn handle(&self, event: &StoredEvent) -> EnterpriseResult<()> {
        if let Some((key, value)) = (self.handler)(event) {
            self.data.insert(key, value);
        }
        Ok(())
    }

    async fn reset(&self) -> EnterpriseResult<()> {
        self.data.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enterprise::eventsource::store::{EventData, InMemoryEventStore};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_checkpoint_store() {
        let store = InMemoryCheckpointStore::new();
        let checkpoint = Checkpoint::new("test-projection".to_string(), 100);

        store.save(&checkpoint).await.unwrap();

        let loaded = store.load("test-projection").await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().last_sequence, 100);

        store.delete("test-projection").await.unwrap();
        let deleted = store.load("test-projection").await.unwrap();
        assert!(deleted.is_none());
    }

    #[tokio::test]
    async fn test_checkpoint_update() {
        let mut checkpoint = Checkpoint::new("test".to_string(), 10);
        let old_timestamp = checkpoint.timestamp;

        tokio::time::sleep(Duration::from_millis(10)).await;

        checkpoint.update(20);
        assert_eq!(checkpoint.last_sequence, 20);
        assert!(checkpoint.timestamp > old_timestamp);
    }

    #[tokio::test]
    async fn test_key_value_projection() {
        let projection = KeyValueProjection::new("test-kv".to_string(), |event| {
            if event.metadata.event_type == "TestEvent" {
                Some((event.metadata.stream_id.clone(), event.metadata.sequence))
            } else {
                None
            }
        });

        let event = StoredEvent {
            metadata: super::super::store::EventMetadata {
                event_id: uuid::Uuid::new_v4(),
                stream_id: "stream-1".to_string(),
                event_type: "TestEvent".to_string(),
                version: 1,
                sequence: 100,
                timestamp: Utc::now(),
                correlation_id: None,
                causation_id: None,
                metadata: HashMap::new(),
            },
            data: vec![],
        };

        projection.handle(&event).await.unwrap();

        assert_eq!(projection.get(&"stream-1".to_string()), Some(100));
        assert_eq!(projection.len(), 1);
    }

    #[tokio::test]
    async fn test_projection_manager() {
        let event_store = Arc::new(InMemoryEventStore::new()) as Arc<dyn EventStore>;
        let checkpoint_store =
            Arc::new(InMemoryCheckpointStore::new()) as Arc<dyn CheckpointStore>;

        // Create some test events
        let events = vec![
            EventData {
                stream_id: "stream-1".to_string(),
                event_type: "Test".to_string(),
                data: vec![1, 2, 3],
                expected_version: -1,
                correlation_id: None,
                causation_id: None,
                metadata: HashMap::new(),
            },
            EventData {
                stream_id: "stream-2".to_string(),
                event_type: "Test".to_string(),
                data: vec![4, 5, 6],
                expected_version: -1,
                correlation_id: None,
                causation_id: None,
                metadata: HashMap::new(),
            },
        ];

        event_store.append_events(events).await.unwrap();

        // Create projection
        let projection = Arc::new(KeyValueProjection::new(
            "test-projection".to_string(),
            |event| Some((event.metadata.stream_id.clone(), event.metadata.sequence)),
        ));

        // Create manager and register projection
        let manager = ProjectionManager::new(event_store, checkpoint_store.clone());
        manager.register(projection.clone());

        // Start manager (catch-up)
        manager.start().await.unwrap();

        // Wait a bit for processing
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Verify projection was built
        assert_eq!(projection.len(), 2);

        // Verify checkpoint was saved
        let checkpoint = checkpoint_store
            .load("test-projection")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(checkpoint.last_sequence, 2);

        // Stop manager
        manager.stop().await;
    }

    #[tokio::test]
    async fn test_projection_reset() {
        let event_store = Arc::new(InMemoryEventStore::new()) as Arc<dyn EventStore>;
        let checkpoint_store =
            Arc::new(InMemoryCheckpointStore::new()) as Arc<dyn CheckpointStore>;

        let projection = Arc::new(KeyValueProjection::new(
            "test-projection".to_string(),
            |event| Some((event.metadata.stream_id.clone(), 1)),
        ));

        let manager = ProjectionManager::new(event_store, checkpoint_store.clone());
        manager.register(projection.clone());

        // Add some data
        let event = StoredEvent {
            metadata: super::super::store::EventMetadata {
                event_id: uuid::Uuid::new_v4(),
                stream_id: "stream-1".to_string(),
                event_type: "Test".to_string(),
                version: 1,
                sequence: 1,
                timestamp: Utc::now(),
                correlation_id: None,
                causation_id: None,
                metadata: HashMap::new(),
            },
            data: vec![],
        };
        projection.handle(&event).await.unwrap();

        assert_eq!(projection.len(), 1);

        // Reset projection
        manager.reset_projection("test-projection").await.unwrap();

        assert_eq!(projection.len(), 0);
    }
}
