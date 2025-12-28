//! Event Store Implementation
//!
//! Provides an append-only event log with versioning, stream management,
//! and global ordering guarantees for event sourcing.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::enterprise::error::{EnterpriseError, EnterpriseResult};

/// Event metadata stored with each event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Unique event ID
    pub event_id: Uuid,
    /// Stream ID (aggregate ID)
    pub stream_id: String,
    /// Event type identifier
    pub event_type: String,
    /// Event version in the stream
    pub version: u64,
    /// Global sequence number
    pub sequence: u64,
    /// Timestamp when event was stored
    pub timestamp: DateTime<Utc>,
    /// Optional correlation ID for tracking related events
    pub correlation_id: Option<Uuid>,
    /// Optional causation ID (ID of the command that caused this event)
    pub causation_id: Option<Uuid>,
    /// Custom metadata key-value pairs
    pub metadata: HashMap<String, String>,
}

/// Stored event with metadata and payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent {
    /// Event metadata
    pub metadata: EventMetadata,
    /// Serialized event data
    pub data: Vec<u8>,
}

/// Event to be appended to the store
#[derive(Debug, Clone)]
pub struct EventData {
    /// Stream ID (typically aggregate ID)
    pub stream_id: String,
    /// Event type identifier
    pub event_type: String,
    /// Serialized event payload
    pub data: Vec<u8>,
    /// Expected version for optimistic concurrency (-1 for any, 0 for new stream)
    pub expected_version: i64,
    /// Optional correlation ID
    pub correlation_id: Option<Uuid>,
    /// Optional causation ID
    pub causation_id: Option<Uuid>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Stream slice with events
#[derive(Debug, Clone)]
pub struct StreamSlice {
    /// Stream ID
    pub stream_id: String,
    /// Current version of the stream
    pub current_version: u64,
    /// Events in this slice
    pub events: Vec<StoredEvent>,
}

/// Event store trait for different backend implementations
#[async_trait]
pub trait EventStore: Send + Sync {
    /// Append events to a stream with optimistic concurrency control
    ///
    /// # Arguments
    ///
    /// * `events` - Events to append
    ///
    /// # Returns
    ///
    /// Vector of stored events with their assigned metadata
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Expected version doesn't match current version (concurrency conflict)
    /// - Stream is locked or unavailable
    async fn append_events(&self, events: Vec<EventData>) -> EnterpriseResult<Vec<StoredEvent>>;

    /// Read events from a stream
    ///
    /// # Arguments
    ///
    /// * `stream_id` - ID of the stream to read
    /// * `from_version` - Starting version (inclusive)
    /// * `max_count` - Maximum number of events to read
    async fn read_stream(
        &self,
        stream_id: &str,
        from_version: u64,
        max_count: usize,
    ) -> EnterpriseResult<StreamSlice>;

    /// Read all events from a stream
    async fn read_stream_all(&self, stream_id: &str) -> EnterpriseResult<StreamSlice> {
        self.read_stream(stream_id, 0, usize::MAX).await
    }

    /// Read events from all streams in global order
    ///
    /// # Arguments
    ///
    /// * `from_sequence` - Starting global sequence number
    /// * `max_count` - Maximum number of events to read
    async fn read_all(
        &self,
        from_sequence: u64,
        max_count: usize,
    ) -> EnterpriseResult<Vec<StoredEvent>>;

    /// Get the current version of a stream
    async fn get_stream_version(&self, stream_id: &str) -> EnterpriseResult<u64>;

    /// Check if a stream exists
    async fn stream_exists(&self, stream_id: &str) -> EnterpriseResult<bool>;

    /// Delete a stream (soft delete by marking as deleted)
    async fn delete_stream(&self, stream_id: &str) -> EnterpriseResult<()>;

    /// Get the global sequence number
    async fn get_global_sequence(&self) -> EnterpriseResult<u64>;
}

/// In-memory event store implementation for testing and development
pub struct InMemoryEventStore {
    /// All events by stream ID
    streams: Arc<RwLock<HashMap<String, Vec<StoredEvent>>>>,
    /// Global event log (all events in order)
    global_log: Arc<RwLock<Vec<StoredEvent>>>,
    /// Current global sequence number
    global_sequence: Arc<RwLock<u64>>,
}

impl InMemoryEventStore {
    /// Create a new in-memory event store
    pub fn new() -> Self {
        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
            global_log: Arc::new(RwLock::new(Vec::new())),
            global_sequence: Arc::new(RwLock::new(0)),
        }
    }

    /// Clear all data (useful for testing)
    pub async fn clear(&self) {
        let mut streams = self.streams.write().await;
        let mut global_log = self.global_log.write().await;
        let mut sequence = self.global_sequence.write().await;

        streams.clear();
        global_log.clear();
        *sequence = 0;
    }
}

impl Default for InMemoryEventStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventStore for InMemoryEventStore {
    async fn append_events(&self, events: Vec<EventData>) -> EnterpriseResult<Vec<StoredEvent>> {
        if events.is_empty() {
            return Ok(Vec::new());
        }

        let mut streams = self.streams.write().await;
        let mut global_log = self.global_log.write().await;
        let mut global_sequence = self.global_sequence.write().await;

        let mut stored_events = Vec::new();

        // Group events by stream for batch processing
        let mut events_by_stream: HashMap<String, Vec<EventData>> = HashMap::new();
        for event in events {
            events_by_stream
                .entry(event.stream_id.clone())
                .or_default()
                .push(event);
        }

        // Process each stream
        for (stream_id, stream_events) in events_by_stream {
            let stream = streams.entry(stream_id.clone()).or_insert_with(Vec::new);
            let current_version = stream.len() as u64;

            // Check optimistic concurrency for first event
            if let Some(first_event) = stream_events.first() {
                let expected = first_event.expected_version;
                if expected >= 0 && expected as u64 != current_version {
                    return Err(EnterpriseError::Other(format!(
                        "Concurrency conflict: expected version {} but stream is at {}",
                        expected, current_version
                    )));
                }
                if expected == 0 && !stream.is_empty() {
                    return Err(EnterpriseError::Other(
                        "Stream already exists but expected new stream".to_string(),
                    ));
                }
            }

            // Append events to stream
            for (idx, event_data) in stream_events.into_iter().enumerate() {
                *global_sequence += 1;
                let version = current_version + idx as u64 + 1;

                let stored_event = StoredEvent {
                    metadata: EventMetadata {
                        event_id: Uuid::new_v4(),
                        stream_id: stream_id.clone(),
                        event_type: event_data.event_type,
                        version,
                        sequence: *global_sequence,
                        timestamp: Utc::now(),
                        correlation_id: event_data.correlation_id,
                        causation_id: event_data.causation_id,
                        metadata: event_data.metadata,
                    },
                    data: event_data.data,
                };

                stream.push(stored_event.clone());
                global_log.push(stored_event.clone());
                stored_events.push(stored_event);
            }
        }

        Ok(stored_events)
    }

    async fn read_stream(
        &self,
        stream_id: &str,
        from_version: u64,
        max_count: usize,
    ) -> EnterpriseResult<StreamSlice> {
        let streams = self.streams.read().await;

        let events = if let Some(stream) = streams.get(stream_id) {
            let start_idx = from_version.saturating_sub(1) as usize;
            stream
                .iter()
                .skip(start_idx)
                .take(max_count)
                .cloned()
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        let current_version = streams
            .get(stream_id)
            .map(|s| s.len() as u64)
            .unwrap_or(0);

        Ok(StreamSlice {
            stream_id: stream_id.to_string(),
            current_version,
            events,
        })
    }

    async fn read_all(
        &self,
        from_sequence: u64,
        max_count: usize,
    ) -> EnterpriseResult<Vec<StoredEvent>> {
        let global_log = self.global_log.read().await;

        let start_idx = from_sequence.saturating_sub(1) as usize;
        let events = global_log
            .iter()
            .skip(start_idx)
            .take(max_count)
            .cloned()
            .collect();

        Ok(events)
    }

    async fn get_stream_version(&self, stream_id: &str) -> EnterpriseResult<u64> {
        let streams = self.streams.read().await;
        Ok(streams.get(stream_id).map(|s| s.len() as u64).unwrap_or(0))
    }

    async fn stream_exists(&self, stream_id: &str) -> EnterpriseResult<bool> {
        let streams = self.streams.read().await;
        Ok(streams.contains_key(stream_id))
    }

    async fn delete_stream(&self, stream_id: &str) -> EnterpriseResult<()> {
        let mut streams = self.streams.write().await;
        streams.remove(stream_id);
        Ok(())
    }

    async fn get_global_sequence(&self) -> EnterpriseResult<u64> {
        let sequence = self.global_sequence.read().await;
        Ok(*sequence)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_event(stream_id: &str, event_type: &str, version: i64) -> EventData {
        EventData {
            stream_id: stream_id.to_string(),
            event_type: event_type.to_string(),
            data: b"test data".to_vec(),
            expected_version: version,
            correlation_id: None,
            causation_id: None,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_append_and_read_events() {
        let store = InMemoryEventStore::new();

        let events = vec![
            create_test_event("stream-1", "TestEvent1", 0),
            create_test_event("stream-1", "TestEvent2", 1),
        ];

        let stored = store.append_events(events).await.unwrap();
        assert_eq!(stored.len(), 2);
        assert_eq!(stored[0].metadata.version, 1);
        assert_eq!(stored[1].metadata.version, 2);

        let slice = store.read_stream_all("stream-1").await.unwrap();
        assert_eq!(slice.events.len(), 2);
        assert_eq!(slice.current_version, 2);
    }

    #[tokio::test]
    async fn test_optimistic_concurrency() {
        let store = InMemoryEventStore::new();

        // First event succeeds
        let event1 = vec![create_test_event("stream-1", "Event1", 0)];
        store.append_events(event1).await.unwrap();

        // Second event with wrong version fails
        let event2 = vec![create_test_event("stream-1", "Event2", 0)];
        let result = store.append_events(event2).await;
        assert!(result.is_err());

        // Correct version succeeds
        let event3 = vec![create_test_event("stream-1", "Event3", 1)];
        store.append_events(event3).await.unwrap();
    }

    #[tokio::test]
    async fn test_global_sequence() {
        let store = InMemoryEventStore::new();

        let events1 = vec![create_test_event("stream-1", "Event1", -1)];
        let events2 = vec![create_test_event("stream-2", "Event2", -1)];
        let events3 = vec![create_test_event("stream-1", "Event3", -1)];

        store.append_events(events1).await.unwrap();
        store.append_events(events2).await.unwrap();
        store.append_events(events3).await.unwrap();

        let all_events = store.read_all(0, 100).await.unwrap();
        assert_eq!(all_events.len(), 3);
        assert_eq!(all_events[0].metadata.sequence, 1);
        assert_eq!(all_events[1].metadata.sequence, 2);
        assert_eq!(all_events[2].metadata.sequence, 3);
    }

    #[tokio::test]
    async fn test_stream_version() {
        let store = InMemoryEventStore::new();

        assert_eq!(store.get_stream_version("stream-1").await.unwrap(), 0);
        assert!(!store.stream_exists("stream-1").await.unwrap());

        let events = vec![create_test_event("stream-1", "Event1", 0)];
        store.append_events(events).await.unwrap();

        assert_eq!(store.get_stream_version("stream-1").await.unwrap(), 1);
        assert!(store.stream_exists("stream-1").await.unwrap());
    }

    #[tokio::test]
    async fn test_read_stream_range() {
        let store = InMemoryEventStore::new();

        let events = vec![
            create_test_event("stream-1", "Event1", -1),
            create_test_event("stream-1", "Event2", -1),
            create_test_event("stream-1", "Event3", -1),
            create_test_event("stream-1", "Event4", -1),
        ];
        store.append_events(events).await.unwrap();

        let slice = store.read_stream("stream-1", 2, 2).await.unwrap();
        assert_eq!(slice.events.len(), 2);
        assert_eq!(slice.events[0].metadata.version, 2);
        assert_eq!(slice.events[1].metadata.version, 3);
    }

    #[tokio::test]
    async fn test_correlation_and_causation() {
        let store = InMemoryEventStore::new();

        let correlation_id = Uuid::new_v4();
        let causation_id = Uuid::new_v4();

        let mut event = create_test_event("stream-1", "Event1", 0);
        event.correlation_id = Some(correlation_id);
        event.causation_id = Some(causation_id);

        let stored = store.append_events(vec![event]).await.unwrap();
        assert_eq!(stored[0].metadata.correlation_id, Some(correlation_id));
        assert_eq!(stored[0].metadata.causation_id, Some(causation_id));
    }
}
