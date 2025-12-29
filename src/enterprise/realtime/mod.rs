//! # Real-Time Collaboration Engine
//!
//! A comprehensive real-time collaboration system for CADDY v0.2.0.
//!
//! ## Overview
//!
//! This module provides a complete infrastructure for real-time collaborative editing
//! of CAD documents with support for:
//!
//! - **CRDTs**: Conflict-free replicated data types for automatic conflict resolution
//! - **Operational Transformation**: Transform operations for consistent concurrent editing
//! - **Document Management**: Version control, branching, and merging
//! - **Presence Tracking**: Real-time user cursor, selection, and status tracking
//! - **Synchronization**: WebSocket-based protocol with delta sync and compression
//! - **Conflict Resolution**: Multiple strategies for handling edit conflicts
//! - **Room Management**: Collaboration rooms with permissions and access control
//!
//! ## Architecture
//!
//! The real-time collaboration system is built on several key components:
//!
//! ### CRDTs (Conflict-free Replicated Data Types)
//!
//! CRDTs provide automatic conflict resolution without coordination:
//! - `GCounter`: Grow-only counter (increment only)
//! - `PNCounter`: Positive-negative counter (increment and decrement)
//! - `LWWRegister`: Last-write-wins register
//! - `ORSet`: Observed-remove set
//! - `RGA`: Replicated growable array (for sequences)
//! - `VectorClock`: Causality tracking
//!
//! ### Operational Transformation
//!
//! OT provides operational transformation for text editing:
//! - Insert, Delete, and Retain operations
//! - Transform function for concurrent operations
//! - Operation composition
//! - History buffer for tracking changes
//!
//! ### Document State
//!
//! Document management with full version control:
//! - Revision history with integrity checking
//! - Branch creation and management
//! - Merge support with conflict detection
//! - Revert to previous versions
//!
//! ### Presence System
//!
//! Real-time user awareness:
//! - Cursor position tracking
//! - Selection range awareness
//! - User status (editing, viewing, idle, away)
//! - Heartbeat mechanism for connection health
//!
//! ### Synchronization Protocol
//!
//! WebSocket-based sync with optimizations:
//! - Delta synchronization (send only changes)
//! - Full sync fallback
//! - Message compression
//! - Reliable message delivery
//!
//! ### Conflict Resolution
//!
//! Multiple strategies for handling conflicts:
//! - Last-writer-wins
//! - First-writer-wins
//! - Manual resolution
//! - Three-way merge
//!
//! ### Room Management
//!
//! Collaboration rooms with access control:
//! - Create/join/leave rooms
//! - Permission levels (Viewer, Editor, Moderator, Owner)
//! - Password protection
//! - Room state management
//!
//! ## Example Usage
//!
//! ```rust
//! use caddy::enterprise::realtime::{
//!     room::{Room, RoomManager, AccessLevel},
//!     document::DocumentState,
//!     presence::UserInfo,
//!     ot::Operation,
//! };
//! use uuid::Uuid;
//!
//! // Create a room manager
//! let mut manager = RoomManager::new();
//!
//! // Create a room
//! let owner = Uuid::new_v4();
//! let room = manager.create_room(
//!     "room1".to_string(),
//!     "Design Review".to_string(),
//!     owner,
//! ).unwrap();
//!
//! // Add a user
//! let user = UserInfo::new(Uuid::new_v4(), "Alice".to_string());
//! let session = Uuid::new_v4();
//!
//! let room = manager.get_room_mut("room1").unwrap();
//! room.add_user(user.clone(), session, AccessLevel::Editor, owner).unwrap();
//!
//! // Apply an operation
//! let op = Operation::insert(0, "Hello".to_string(), 1, user.id);
//! room.apply_operation(op, user.id).unwrap();
//! ```
//!
//! ## Performance Considerations
//!
//! - CRDTs provide O(1) merge operations
//! - OT transform is O(n) where n is operation size
//! - Delta sync reduces network bandwidth
//! - Compression applied for large payloads
//! - In-memory operation history with bounded size
//!
//! ## Thread Safety
//!
//! Most types in this module are not thread-safe by default. For concurrent access,
//! wrap them in appropriate synchronization primitives (e.g., `Arc<Mutex<Room>>`).
//!
//! ## Future Enhancements
//!
//! - Persistent storage backend
//! - WebRTC for peer-to-peer sync
//! - Operational transform for geometric operations
//! - Optimistic locking for critical sections
//! - Real-time voice/video integration

// Public modules
pub mod crdt;
pub mod ot;
pub mod document;
pub mod presence;
pub mod sync;
pub mod conflict;
pub mod room;

// Re-export commonly used types
pub use crdt::{
    GCounter, PNCounter, LWWRegister, ORSet, RGA,
    VectorClock, LamportTime, ReplicaId,
};

pub use ot::{
    Operation, OpType, HistoryBuffer,
    transform, compose, OTError,
};

pub use document::{
    DocumentState, Revision, Branch, ConflictMarker,
    DocumentError, DocumentStats,
};

pub use presence::{
    PresenceManager, UserPresence, UserInfo, UserStatus,
    CursorPosition, Selection, HeartbeatManager,
    PresenceError,
};

pub use sync::{
    SyncMessage, SyncProtocol, DeltaSync, UserSnapshot,
    MessageQueue, SyncError, PROTOCOL_VERSION,
};

pub use conflict::{
    ConflictResolver, Conflict, ResolutionStrategy,
    ConflictType, ConflictStatus, ThreeWayMerge,
    ConflictError, ConflictStats,
};

pub use room::{
    Room, RoomManager, RoomSettings, RoomState, RoomStats,
    AccessLevel, UserPermission, RoomError,
};

/// Version of the real-time collaboration engine
pub const REALTIME_VERSION: &str = "0.2.0";

/// Maximum number of operations to keep in history
pub const MAX_HISTORY_SIZE: usize = 10000;

/// Default heartbeat interval (seconds)
pub const DEFAULT_HEARTBEAT_INTERVAL: u64 = 30;

/// Default activity timeout (seconds)
pub const DEFAULT_ACTIVITY_TIMEOUT: u64 = 300;

/// Default idle timeout (seconds)
pub const DEFAULT_IDLE_TIMEOUT: u64 = 60;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_integration() {
        // Test that all modules work together
        let owner = uuid::Uuid::new_v4();
        let mut manager = RoomManager::new();

        // Create room
        manager
            .create_room("room1".to_string(), "Test".to_string(), owner)
            .unwrap();

        // Add document
        let _doc = DocumentState::new(
            "test.cad".to_string(),
            "Initial content".to_string(),
            owner,
        );

        let room = manager.get_room_mut("room1").unwrap();
        *room = room.clone().with_document(doc);

        // Add user
        let user = UserInfo::new(uuid::Uuid::new_v4(), "Alice".to_string());
        let session = uuid::Uuid::new_v4();

        room.add_user(user.clone(), session, AccessLevel::Editor, owner)
            .unwrap();

        // Apply operation
        let op = Operation::insert(15, " - edited".to_string(), 1, user.id);
        let version = room.apply_operation(op, user.id).unwrap();

        assert_eq!(version, 1);

        let content = room.get_content().unwrap();
        assert!(content.contains("edited"));
    }

    #[test]
    fn test_crdt_integration() {
        use uuid::Uuid;

        let replica1 = Uuid::new_v4();
        let replica2 = Uuid::new_v4();

        let mut counter1 = GCounter::new(replica1);
        let mut counter2 = GCounter::new(replica2);

        counter1.increment();
        counter1.increment();
        counter2.increment();

        counter1.merge(&counter2);

        assert_eq!(counter1.value(), 3);
    }

    #[test]
    fn test_ot_integration() {
        let client = uuid::Uuid::new_v4();

        let op1 = Operation::insert(0, "Hello".to_string(), 1, client);
        let op2 = Operation::insert(5, " World".to_string(), 2, client);

        let mut text = String::new();
        text = op1.apply(&text).unwrap();
        text = op2.apply(&text).unwrap();

        assert_eq!(text, "Hello World");
    }

    #[test]
    fn test_presence_integration() {
        let mut manager = PresenceManager::new();

        let user = UserInfo::new(uuid::Uuid::new_v4(), "Bob".to_string());
        let session = uuid::Uuid::new_v4();

        let user_id = manager.add_user(user, session);

        let cursor = CursorPosition::new(1, 2, 10);
        manager.update_cursor(user_id, cursor).unwrap();

        let presence = manager.get_user(user_id).unwrap();
        assert_eq!(presence.cursor, cursor);
    }

    #[test]
    fn test_conflict_resolution_integration() {
        let mut resolver = ConflictResolver::new(ResolutionStrategy::LastWriterWins);

        let client1 = uuid::Uuid::new_v4();
        let client2 = uuid::Uuid::new_v4();

        let op1 = Operation::insert(0, "A".to_string(), 1, client1);
        let op2 = Operation::insert(0, "B".to_string(), 1, client2);

        let conflict_id = resolver.detect_conflict(
            op1,
            op2,
            LamportTime(1),
            LamportTime(2),
        );

        let resolution = resolver.resolve_conflict(conflict_id).unwrap();

        assert!(resolution.id != uuid::Uuid::nil());
    }

    #[test]
    fn test_sync_protocol_integration() {
        let mut protocol = SyncProtocol::new();

        protocol.init_delta_sync("room1".to_string(), 0);

        let msg = SyncMessage::Heartbeat {
            user_id: uuid::Uuid::new_v4(),
            timestamp: 123,
        };

        let msg_id = protocol.send(msg).unwrap();
        let (recv_id, _) = protocol.receive().unwrap();

        assert_eq!(msg_id, recv_id);
    }
}
