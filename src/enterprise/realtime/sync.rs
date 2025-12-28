//! # Synchronization Protocol
//!
//! Implements WebSocket-based synchronization protocol with delta sync and compression.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

use super::ot::Operation;
use super::presence::{CursorPosition, Selection, UserStatus};

/// Errors related to synchronization
#[derive(Debug, Error)]
pub enum SyncError {
    #[error("Invalid message format")]
    InvalidMessage,
    #[error("Version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: u64, actual: u64 },
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Compression error: {0}")]
    CompressionError(String),
    #[error("Authentication required")]
    AuthenticationRequired,
    #[error("Room not found: {0}")]
    RoomNotFound(String),
}

/// Message types for the sync protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SyncMessage {
    /// Join a collaboration room
    Join {
        room_id: String,
        user_id: Uuid,
        user_name: String,
        token: Option<String>,
    },

    /// Leave a room
    Leave {
        room_id: String,
        user_id: Uuid,
    },

    /// Acknowledge join
    JoinAck {
        room_id: String,
        session_id: Uuid,
        current_version: u64,
        users: Vec<UserSnapshot>,
    },

    /// Operation from client to server
    ClientOp {
        room_id: String,
        user_id: Uuid,
        operation: Operation,
        base_version: u64,
    },

    /// Operation from server to clients
    ServerOp {
        room_id: String,
        operation: Operation,
        version: u64,
    },

    /// Batch of operations
    OpBatch {
        room_id: String,
        operations: Vec<Operation>,
        start_version: u64,
    },

    /// Cursor update
    CursorUpdate {
        room_id: String,
        user_id: Uuid,
        position: CursorPosition,
    },

    /// Selection update
    SelectionUpdate {
        room_id: String,
        user_id: Uuid,
        selection: Option<Selection>,
    },

    /// Status update
    StatusUpdate {
        room_id: String,
        user_id: Uuid,
        status: UserStatus,
    },

    /// Heartbeat from client
    Heartbeat {
        user_id: Uuid,
        timestamp: i64,
    },

    /// Heartbeat acknowledgment
    HeartbeatAck {
        timestamp: i64,
    },

    /// Request full sync
    RequestSync {
        room_id: String,
        user_id: Uuid,
    },

    /// Full sync response
    FullSync {
        room_id: String,
        content: String,
        version: u64,
        compressed: bool,
    },

    /// Request delta sync (operations since version)
    RequestDelta {
        room_id: String,
        user_id: Uuid,
        since_version: u64,
    },

    /// Error message
    Error {
        code: String,
        message: String,
    },

    /// User joined notification
    UserJoined {
        room_id: String,
        user: UserSnapshot,
    },

    /// User left notification
    UserLeft {
        room_id: String,
        user_id: Uuid,
    },

    /// Acknowledgment
    Ack {
        message_id: Uuid,
    },
}

/// Snapshot of user state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSnapshot {
    pub user_id: Uuid,
    pub user_name: String,
    pub color: String,
    pub cursor: CursorPosition,
    pub selection: Option<Selection>,
    pub status: UserStatus,
}

impl SyncMessage {
    /// Get the room ID from the message
    pub fn room_id(&self) -> Option<&str> {
        match self {
            SyncMessage::Join { room_id, .. }
            | SyncMessage::Leave { room_id, .. }
            | SyncMessage::JoinAck { room_id, .. }
            | SyncMessage::ClientOp { room_id, .. }
            | SyncMessage::ServerOp { room_id, .. }
            | SyncMessage::OpBatch { room_id, .. }
            | SyncMessage::CursorUpdate { room_id, .. }
            | SyncMessage::SelectionUpdate { room_id, .. }
            | SyncMessage::StatusUpdate { room_id, .. }
            | SyncMessage::RequestSync { room_id, .. }
            | SyncMessage::FullSync { room_id, .. }
            | SyncMessage::RequestDelta { room_id, .. }
            | SyncMessage::UserJoined { room_id, .. }
            | SyncMessage::UserLeft { room_id, .. } => Some(room_id),
            _ => None,
        }
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, SyncError> {
        serde_json::to_string(self)
            .map_err(|e| SyncError::SerializationError(e.to_string()))
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, SyncError> {
        serde_json::from_str(json)
            .map_err(|e| SyncError::SerializationError(e.to_string()))
    }

    /// Serialize to binary (using bincode)
    pub fn to_binary(&self) -> Result<Vec<u8>, SyncError> {
        bincode::serialize(self)
            .map_err(|e| SyncError::SerializationError(e.to_string()))
    }

    /// Deserialize from binary
    pub fn from_binary(data: &[u8]) -> Result<Self, SyncError> {
        bincode::deserialize(data)
            .map_err(|e| SyncError::SerializationError(e.to_string()))
    }
}

/// Delta synchronization tracker
#[derive(Debug, Clone)]
pub struct DeltaSync {
    /// Room ID
    room_id: String,
    /// Current version
    current_version: u64,
    /// Pending operations (not yet acknowledged)
    pending_ops: Vec<Operation>,
    /// Operations waiting for server acknowledgment
    inflight_ops: HashMap<Uuid, Operation>,
}

impl DeltaSync {
    /// Create a new delta sync tracker
    pub fn new(room_id: String, initial_version: u64) -> Self {
        Self {
            room_id,
            current_version: initial_version,
            pending_ops: Vec::new(),
            inflight_ops: HashMap::new(),
        }
    }

    /// Add a pending operation
    pub fn add_pending(&mut self, operation: Operation) {
        self.pending_ops.push(operation);
    }

    /// Move pending operations to inflight
    pub fn flush_pending(&mut self) -> Vec<Operation> {
        let ops = self.pending_ops.drain(..).collect::<Vec<_>>();
        for op in &ops {
            self.inflight_ops.insert(op.id, op.clone());
        }
        ops
    }

    /// Acknowledge an operation
    pub fn acknowledge(&mut self, op_id: Uuid, version: u64) -> Option<Operation> {
        self.current_version = self.current_version.max(version);
        self.inflight_ops.remove(&op_id)
    }

    /// Get current version
    pub fn version(&self) -> u64 {
        self.current_version
    }

    /// Update version
    pub fn set_version(&mut self, version: u64) {
        self.current_version = version;
    }

    /// Get pending operation count
    pub fn pending_count(&self) -> usize {
        self.pending_ops.len()
    }

    /// Get inflight operation count
    pub fn inflight_count(&self) -> usize {
        self.inflight_ops.len()
    }

    /// Check if there are pending operations
    pub fn has_pending(&self) -> bool {
        !self.pending_ops.is_empty()
    }
}

/// Compression utilities
pub mod compression {
    use super::SyncError;

    /// Compress data using a simple run-length encoding
    pub fn compress(data: &str) -> Result<Vec<u8>, SyncError> {
        // For a real implementation, use a proper compression library like flate2
        // This is a placeholder that just converts to bytes
        Ok(data.as_bytes().to_vec())
    }

    /// Decompress data
    pub fn decompress(data: &[u8]) -> Result<String, SyncError> {
        String::from_utf8(data.to_vec())
            .map_err(|e| SyncError::CompressionError(e.to_string()))
    }

    /// Check if compression would be beneficial
    pub fn should_compress(data: &str) -> bool {
        data.len() > 1024 // Compress if larger than 1KB
    }
}

/// Message queue for reliable delivery
#[derive(Debug)]
pub struct MessageQueue {
    /// Queued messages
    queue: Vec<QueuedMessage>,
    /// Maximum queue size
    max_size: usize,
}

#[derive(Debug, Clone)]
struct QueuedMessage {
    id: Uuid,
    message: SyncMessage,
    timestamp: std::time::SystemTime,
    retry_count: u32,
}

impl MessageQueue {
    /// Create a new message queue
    pub fn new(max_size: usize) -> Self {
        Self {
            queue: Vec::new(),
            max_size,
        }
    }

    /// Enqueue a message
    pub fn enqueue(&mut self, message: SyncMessage) -> Result<Uuid, SyncError> {
        if self.queue.len() >= self.max_size {
            return Err(SyncError::SerializationError(
                "Queue full".to_string(),
            ));
        }

        let id = Uuid::new_v4();
        let queued = QueuedMessage {
            id,
            message,
            timestamp: std::time::SystemTime::now(),
            retry_count: 0,
        };

        self.queue.push(queued);
        Ok(id)
    }

    /// Dequeue next message
    pub fn dequeue(&mut self) -> Option<(Uuid, SyncMessage)> {
        if let Some(queued) = self.queue.first().cloned() {
            self.queue.remove(0);
            Some((queued.id, queued.message))
        } else {
            None
        }
    }

    /// Peek next message without removing
    pub fn peek(&self) -> Option<&SyncMessage> {
        self.queue.first().map(|q| &q.message)
    }

    /// Remove acknowledged message
    pub fn acknowledge(&mut self, id: Uuid) -> bool {
        if let Some(pos) = self.queue.iter().position(|q| q.id == id) {
            self.queue.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get queue size
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Clear the queue
    pub fn clear(&mut self) {
        self.queue.clear();
    }
}

/// Protocol version
pub const PROTOCOL_VERSION: u32 = 1;

/// Sync protocol handler
#[derive(Debug)]
pub struct SyncProtocol {
    /// Protocol version
    version: u32,
    /// Message queue
    message_queue: MessageQueue,
    /// Delta sync tracker
    delta_sync: Option<DeltaSync>,
}

impl SyncProtocol {
    /// Create a new sync protocol handler
    pub fn new() -> Self {
        Self {
            version: PROTOCOL_VERSION,
            message_queue: MessageQueue::new(1000),
            delta_sync: None,
        }
    }

    /// Initialize delta sync for a room
    pub fn init_delta_sync(&mut self, room_id: String, version: u64) {
        self.delta_sync = Some(DeltaSync::new(room_id, version));
    }

    /// Send a message
    pub fn send(&mut self, message: SyncMessage) -> Result<Uuid, SyncError> {
        self.message_queue.enqueue(message)
    }

    /// Receive next message
    pub fn receive(&mut self) -> Option<(Uuid, SyncMessage)> {
        self.message_queue.dequeue()
    }

    /// Acknowledge a message
    pub fn acknowledge(&mut self, message_id: Uuid) {
        self.message_queue.acknowledge(message_id);
    }

    /// Get protocol version
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Get delta sync tracker
    pub fn delta_sync(&self) -> Option<&DeltaSync> {
        self.delta_sync.as_ref()
    }

    /// Get mutable delta sync tracker
    pub fn delta_sync_mut(&mut self) -> Option<&mut DeltaSync> {
        self.delta_sync.as_mut()
    }
}

impl Default for SyncProtocol {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_message_serialization() {
        let msg = SyncMessage::Heartbeat {
            user_id: Uuid::new_v4(),
            timestamp: 12345,
        };

        let json = msg.to_json().unwrap();
        let deserialized = SyncMessage::from_json(&json).unwrap();

        match deserialized {
            SyncMessage::Heartbeat { timestamp, .. } => {
                assert_eq!(timestamp, 12345);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_delta_sync() {
        let mut delta = DeltaSync::new("room1".to_string(), 0);

        let op = Operation::insert(0, "test".to_string(), 1, Uuid::new_v4());
        delta.add_pending(op.clone());

        assert_eq!(delta.pending_count(), 1);

        let pending = delta.flush_pending();
        assert_eq!(pending.len(), 1);
        assert_eq!(delta.inflight_count(), 1);
        assert_eq!(delta.pending_count(), 0);
    }

    #[test]
    fn test_message_queue() {
        let mut queue = MessageQueue::new(10);

        let msg = SyncMessage::Heartbeat {
            user_id: Uuid::new_v4(),
            timestamp: 123,
        };

        let id = queue.enqueue(msg).unwrap();
        assert_eq!(queue.len(), 1);

        queue.acknowledge(id);
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_compression() {
        let data = "Hello, World!";
        let compressed = compression::compress(data).unwrap();
        let decompressed = compression::decompress(&compressed).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_sync_protocol() {
        let mut protocol = SyncProtocol::new();

        assert_eq!(protocol.version(), PROTOCOL_VERSION);

        let msg = SyncMessage::Heartbeat {
            user_id: Uuid::new_v4(),
            timestamp: 999,
        };

        let msg_id = protocol.send(msg).unwrap();

        let (received_id, received_msg) = protocol.receive().unwrap();
        assert_eq!(msg_id, received_id);

        match received_msg {
            SyncMessage::Heartbeat { timestamp, .. } => {
                assert_eq!(timestamp, 999);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_room_id_extraction() {
        let msg = SyncMessage::Join {
            room_id: "room123".to_string(),
            user_id: Uuid::new_v4(),
            user_name: "Alice".to_string(),
            token: None,
        };

        assert_eq!(msg.room_id(), Some("room123"));
    }
}
