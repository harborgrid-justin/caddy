//! Event Replay and Versioning
//!
//! Provides infrastructure for replaying events for migrations, rebuilding projections,
//! event upcasting (transforming old event formats to new ones), and progress tracking.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::store::{EventStore, StoredEvent};
use crate::enterprise::error::{EnterpriseError, EnterpriseResult};

/// Event upcaster for transforming old event versions to new ones
#[async_trait]
pub trait EventUpcaster: Send + Sync {
    /// Check if this upcaster can handle the given event type and version
    fn can_upcast(&self, event_type: &str, version: &str) -> bool;

    /// Transform an event to a newer version
    ///
    /// Returns the new event data and the new version identifier
    async fn upcast(&self, event: &StoredEvent) -> EnterpriseResult<(Vec<u8>, String)>;
}

/// Chain of upcasters for multi-version transformations
pub struct UpcasterChain {
    upcasters: Vec<Arc<dyn EventUpcaster>>,
}

impl UpcasterChain {
    /// Create a new upcaster chain
    pub fn new() -> Self {
        Self {
            upcasters: Vec::new(),
        }
    }

    /// Add an upcaster to the chain
    pub fn add(&mut self, upcaster: Arc<dyn EventUpcaster>) {
        self.upcasters.push(upcaster);
    }

    /// Apply all applicable upcasters to an event
    pub async fn upcast(&self, mut event: StoredEvent) -> EnterpriseResult<StoredEvent> {
        let mut current_version = event
            .metadata
            .metadata
            .get("event_version")
            .cloned()
            .unwrap_or_else(|| "1".to_string());

        for upcaster in &self.upcasters {
            if upcaster.can_upcast(&event.metadata.event_type, &current_version) {
                let (new_data, new_version) = upcaster.upcast(&event).await?;
                event.data = new_data;
                current_version = new_version.clone();
                event
                    .metadata
                    .metadata
                    .insert("event_version".to_string(), new_version);
            }
        }

        Ok(event)
    }
}

impl Default for UpcasterChain {
    fn default() -> Self {
        Self::new()
    }
}

/// Progress tracking for replay operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayProgress {
    /// Replay operation ID
    pub replay_id: String,
    /// Total number of events to replay
    pub total_events: u64,
    /// Number of events processed so far
    pub processed_events: u64,
    /// Number of events that failed to process
    pub failed_events: u64,
    /// Start time
    pub start_time: DateTime<Utc>,
    /// Last update time
    pub last_update: DateTime<Utc>,
    /// Optional completion time
    pub completed_at: Option<DateTime<Utc>>,
    /// Current status
    pub status: ReplayStatus,
    /// Optional error message
    pub error: Option<String>,
}

impl ReplayProgress {
    /// Create a new replay progress tracker
    pub fn new(replay_id: String, total_events: u64) -> Self {
        Self {
            replay_id,
            total_events,
            processed_events: 0,
            failed_events: 0,
            start_time: Utc::now(),
            last_update: Utc::now(),
            completed_at: None,
            status: ReplayStatus::Running,
            error: None,
        }
    }

    /// Update progress
    pub fn update(&mut self, processed: u64, failed: u64) {
        self.processed_events = processed;
        self.failed_events = failed;
        self.last_update = Utc::now();
    }

    /// Mark as completed
    pub fn complete(&mut self) {
        self.status = ReplayStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.last_update = Utc::now();
    }

    /// Mark as failed
    pub fn fail(&mut self, error: String) {
        self.status = ReplayStatus::Failed;
        self.error = Some(error);
        self.completed_at = Some(Utc::now());
        self.last_update = Utc::now();
    }

    /// Get progress percentage
    pub fn percentage(&self) -> f64 {
        if self.total_events == 0 {
            return 100.0;
        }
        (self.processed_events as f64 / self.total_events as f64) * 100.0
    }

    /// Get estimated time remaining
    pub fn estimated_time_remaining(&self) -> Option<chrono::Duration> {
        if self.processed_events == 0 || self.status != ReplayStatus::Running {
            return None;
        }

        let elapsed = Utc::now() - self.start_time;
        let rate = self.processed_events as f64 / elapsed.num_seconds() as f64;
        let remaining = self.total_events - self.processed_events;
        let seconds_remaining = (remaining as f64 / rate) as i64;

        Some(chrono::Duration::seconds(seconds_remaining))
    }
}

/// Replay status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplayStatus {
    /// Replay is running
    Running,
    /// Replay is paused
    Paused,
    /// Replay completed successfully
    Completed,
    /// Replay failed
    Failed,
}

/// Handler for processing events during replay
#[async_trait]
pub trait ReplayHandler: Send + Sync {
    /// Handle an event during replay
    async fn handle(&self, event: &StoredEvent) -> EnterpriseResult<()>;

    /// Called when replay starts
    async fn on_start(&self) -> EnterpriseResult<()> {
        Ok(())
    }

    /// Called when replay completes
    async fn on_complete(&self) -> EnterpriseResult<()> {
        Ok(())
    }

    /// Called when replay fails
    async fn on_error(&self, error: &EnterpriseError) -> EnterpriseResult<()> {
        let _ = error;
        Ok(())
    }
}

/// Event replay engine
pub struct ReplayEngine {
    event_store: Arc<dyn EventStore>,
    upcaster_chain: Arc<UpcasterChain>,
    progress: Arc<RwLock<HashMap<String, ReplayProgress>>>,
}

impl ReplayEngine {
    /// Create a new replay engine
    pub fn new(event_store: Arc<dyn EventStore>) -> Self {
        Self {
            event_store,
            upcaster_chain: Arc::new(UpcasterChain::new()),
            progress: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a replay engine with custom upcaster chain
    pub fn with_upcasters(event_store: Arc<dyn EventStore>, upcaster_chain: UpcasterChain) -> Self {
        Self {
            event_store,
            upcaster_chain: Arc::new(upcaster_chain),
            progress: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Replay all events through a handler
    ///
    /// # Arguments
    ///
    /// * `handler` - Handler to process each event
    /// * `from_sequence` - Starting sequence number
    /// * `to_sequence` - Ending sequence number (None for all)
    /// * `batch_size` - Number of events to process per batch
    pub async fn replay_all<H: ReplayHandler>(
        &self,
        handler: Arc<H>,
        from_sequence: u64,
        to_sequence: Option<u64>,
        batch_size: usize,
    ) -> EnterpriseResult<String> {
        let replay_id = uuid::Uuid::new_v4().to_string();

        // Estimate total events
        let global_sequence = self.event_store.get_global_sequence().await?;
        let end_seq = to_sequence.unwrap_or(global_sequence);
        let total_events = end_seq.saturating_sub(from_sequence);

        // Initialize progress
        let mut progress_map = self.progress.write().await;
        progress_map.insert(
            replay_id.clone(),
            ReplayProgress::new(replay_id.clone(), total_events),
        );
        drop(progress_map);

        // Notify handler of start
        if let Err(e) = handler.on_start().await {
            self.mark_failed(&replay_id, e.to_string()).await;
            return Err(e);
        }

        // Process events in batches
        let mut current_sequence = from_sequence;
        let mut processed = 0u64;
        let mut failed = 0u64;

        loop {
            let events = self
                .event_store
                .read_all(current_sequence, batch_size)
                .await?;

            if events.is_empty() {
                break;
            }

            for event in events {
                if let Some(end) = to_sequence {
                    if event.metadata.sequence > end {
                        break;
                    }
                }

                // Apply upcasters
                let upcasted = self.upcaster_chain.upcast(event).await?;

                // Process event
                match handler.handle(&upcasted).await {
                    Ok(()) => processed += 1,
                    Err(e) => {
                        failed += 1;
                        // Log error but continue processing
                        eprintln!("Error processing event: {}", e);
                    }
                }

                current_sequence = upcasted.metadata.sequence + 1;

                // Update progress periodically
                if processed % 100 == 0 {
                    self.update_progress(&replay_id, processed, failed).await;
                }
            }

            if let Some(end) = to_sequence {
                if current_sequence > end {
                    break;
                }
            }
        }

        // Final progress update
        self.update_progress(&replay_id, processed, failed).await;

        // Notify handler of completion
        if let Err(e) = handler.on_complete().await {
            self.mark_failed(&replay_id, e.to_string()).await;
            handler.on_error(&e).await?;
            return Err(e);
        }

        // Mark as completed
        self.mark_completed(&replay_id).await;

        Ok(replay_id)
    }

    /// Replay events for a specific stream
    pub async fn replay_stream<H: ReplayHandler>(
        &self,
        stream_id: &str,
        handler: Arc<H>,
        from_version: u64,
        to_version: Option<u64>,
    ) -> EnterpriseResult<String> {
        let replay_id = uuid::Uuid::new_v4().to_string();

        // Get stream slice
        let slice = self.event_store.read_stream_all(stream_id).await?;
        let total_events = slice.events.len() as u64;

        // Initialize progress
        let mut progress_map = self.progress.write().await;
        progress_map.insert(
            replay_id.clone(),
            ReplayProgress::new(replay_id.clone(), total_events),
        );
        drop(progress_map);

        handler.on_start().await?;

        let mut processed = 0u64;
        let mut failed = 0u64;

        for event in slice.events {
            if event.metadata.version < from_version {
                continue;
            }
            if let Some(end) = to_version {
                if event.metadata.version > end {
                    break;
                }
            }

            let upcasted = self.upcaster_chain.upcast(event).await?;

            match handler.handle(&upcasted).await {
                Ok(()) => processed += 1,
                Err(_) => failed += 1,
            }

            self.update_progress(&replay_id, processed, failed).await;
        }

        handler.on_complete().await?;
        self.mark_completed(&replay_id).await;

        Ok(replay_id)
    }

    /// Get progress for a replay operation
    pub async fn get_progress(&self, replay_id: &str) -> Option<ReplayProgress> {
        let progress = self.progress.read().await;
        progress.get(replay_id).cloned()
    }

    /// Update progress
    async fn update_progress(&self, replay_id: &str, processed: u64, failed: u64) {
        let mut progress = self.progress.write().await;
        if let Some(p) = progress.get_mut(replay_id) {
            p.update(processed, failed);
        }
    }

    /// Mark replay as completed
    async fn mark_completed(&self, replay_id: &str) {
        let mut progress = self.progress.write().await;
        if let Some(p) = progress.get_mut(replay_id) {
            p.complete();
        }
    }

    /// Mark replay as failed
    async fn mark_failed(&self, replay_id: &str, error: String) {
        let mut progress = self.progress.write().await;
        if let Some(p) = progress.get_mut(replay_id) {
            p.fail(error);
        }
    }
}

/// Simple event counter handler for testing
pub struct CountingHandler {
    count: Arc<RwLock<u64>>,
}

impl CountingHandler {
    /// Create a new counting handler
    pub fn new() -> Self {
        Self {
            count: Arc::new(RwLock::new(0)),
        }
    }

    /// Get the current count
    pub async fn count(&self) -> u64 {
        *self.count.read().await
    }
}

impl Default for CountingHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ReplayHandler for CountingHandler {
    async fn handle(&self, _event: &StoredEvent) -> EnterpriseResult<()> {
        let mut count = self.count.write().await;
        *count += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enterprise::eventsource::store::{EventData, InMemoryEventStore};

    #[tokio::test]
    async fn test_replay_progress() {
        let mut progress = ReplayProgress::new("test-replay".to_string(), 100);

        assert_eq!(progress.percentage(), 0.0);
        assert_eq!(progress.status, ReplayStatus::Running);

        progress.update(50, 0);
        assert_eq!(progress.percentage(), 50.0);

        progress.complete();
        assert_eq!(progress.status, ReplayStatus::Completed);
        assert!(progress.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_replay_all_events() {
        let store = Arc::new(InMemoryEventStore::new()) as Arc<dyn EventStore>;

        // Create test events
        for i in 1..=10 {
            let event = EventData {
                stream_id: format!("stream-{}", i),
                event_type: "TestEvent".to_string(),
                data: vec![i],
                expected_version: -1,
                correlation_id: None,
                causation_id: None,
                metadata: HashMap::new(),
            };
            store.append_events(vec![event]).await.unwrap();
        }

        let engine = ReplayEngine::new(store);
        let handler = Arc::new(CountingHandler::new());

        let replay_id = engine
            .replay_all(handler.clone(), 0, None, 5)
            .await
            .unwrap();

        // Verify all events were processed
        assert_eq!(handler.count().await, 10);

        // Check progress
        let progress = engine.get_progress(&replay_id).await.unwrap();
        assert_eq!(progress.status, ReplayStatus::Completed);
        assert_eq!(progress.processed_events, 10);
    }

    #[tokio::test]
    async fn test_replay_stream() {
        let store = Arc::new(InMemoryEventStore::new()) as Arc<dyn EventStore>;

        // Create events for a specific stream
        for i in 1..=5 {
            let event = EventData {
                stream_id: "stream-1".to_string(),
                event_type: "TestEvent".to_string(),
                data: vec![i],
                expected_version: -1,
                correlation_id: None,
                causation_id: None,
                metadata: HashMap::new(),
            };
            store.append_events(vec![event]).await.unwrap();
        }

        let engine = ReplayEngine::new(store);
        let handler = Arc::new(CountingHandler::new());

        let replay_id = engine
            .replay_stream("stream-1", handler.clone(), 0, None)
            .await
            .unwrap();

        assert_eq!(handler.count().await, 5);

        let progress = engine.get_progress(&replay_id).await.unwrap();
        assert_eq!(progress.status, ReplayStatus::Completed);
    }

    #[tokio::test]
    async fn test_partial_replay() {
        let store = Arc::new(InMemoryEventStore::new()) as Arc<dyn EventStore>;

        for i in 1..=10 {
            let event = EventData {
                stream_id: "stream-1".to_string(),
                event_type: "TestEvent".to_string(),
                data: vec![i],
                expected_version: -1,
                correlation_id: None,
                causation_id: None,
                metadata: HashMap::new(),
            };
            store.append_events(vec![event]).await.unwrap();
        }

        let engine = ReplayEngine::new(store);
        let handler = Arc::new(CountingHandler::new());

        // Replay only sequences 3-7
        engine
            .replay_all(handler.clone(), 3, Some(7), 10)
            .await
            .unwrap();

        assert_eq!(handler.count().await, 5);
    }

    #[tokio::test]
    async fn test_upcaster_chain() {
        struct TestUpcaster;

        #[async_trait]
        impl EventUpcaster for TestUpcaster {
            fn can_upcast(&self, event_type: &str, version: &str) -> bool {
                event_type == "TestEvent" && version == "1"
            }

            async fn upcast(&self, event: &StoredEvent) -> EnterpriseResult<(Vec<u8>, String)> {
                // Transform event data
                let mut new_data = event.data.clone();
                new_data.push(99); // Add marker
                Ok((new_data, "2".to_string()))
            }
        }

        let mut chain = UpcasterChain::new();
        chain.add(Arc::new(TestUpcaster));

        let mut event = StoredEvent {
            metadata: super::super::store::EventMetadata {
                event_id: uuid::Uuid::new_v4(),
                stream_id: "test".to_string(),
                event_type: "TestEvent".to_string(),
                version: 1,
                sequence: 1,
                timestamp: Utc::now(),
                correlation_id: None,
                causation_id: None,
                metadata: HashMap::from([("event_version".to_string(), "1".to_string())]),
            },
            data: vec![1, 2, 3],
        };

        let upcasted = chain.upcast(event).await.unwrap();
        assert_eq!(upcasted.data, vec![1, 2, 3, 99]);
        assert_eq!(
            upcasted.metadata.metadata.get("event_version"),
            Some(&"2".to_string())
        );
    }
}
