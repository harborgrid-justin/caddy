//! Enterprise Real-Time Collaboration Module
//!
//! This module provides a comprehensive real-time collaboration system for CADDY,
//! enabling multiple users to work simultaneously on CAD projects with operational
//! transformation, presence awareness, and fine-grained permissions.
//!
//! # Features
//!
//! - **Collaboration Sessions**: Multi-user session management with persistence
//! - **User Presence**: Real-time cursor, selection, and activity tracking
//! - **Operational Transformation**: Conflict-free collaborative editing with CRDT support
//! - **Binary Protocol**: Efficient message serialization for low latency
//! - **WebSocket Transport**: Reliable transport with reconnection and state recovery
//! - **Fine-Grained Permissions**: Edit, view-only, and region-locking capabilities
//!
//! # Architecture
//!
//! The collaboration system is built on several key components:
//!
//! - **Session Management**: Handles the lifecycle of collaboration sessions
//! - **Presence Manager**: Tracks user activity and shares cursor/selection state
//! - **Operation Engine**: Applies operational transformation for conflict resolution
//! - **Protocol Layer**: Defines message types and serialization
//! - **Transport Layer**: Manages WebSocket connections with reliability features
//! - **Permission System**: Enforces access control and editing rights
//!
//! # Example
//!
//! ```no_run
//! use caddy::enterprise::collaboration::{
//!     CollaborationSession, SessionConfig, PresenceManager, Transport
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a collaboration session
//! let config = SessionConfig {
//!     session_name: "Project Alpha".to_string(),
//!     max_participants: 50,
//!     persistence_enabled: true,
//!     ..Default::default()
//! };
//!
//! let session = CollaborationSession::create(config).await?;
//!
//! // Join the session
//! let user_id = uuid::Uuid::new_v4();
//! session.join(user_id, "John Doe".to_string()).await?;
//! # Ok(())
//! # }
//! ```

pub mod session;
pub mod presence;
pub mod operations;
pub mod protocol;
pub mod transport;
pub mod permissions;
pub mod crdt;
pub mod sync_engine;
pub mod versioning;
pub mod conflict_resolver;

// Re-export commonly used types
pub use session::{
    CollaborationSession, Participant, ParticipantInfo, SessionConfig, SessionId,
    SessionMetrics, SessionState, SessionStorage,
};
pub use presence::{
    ActivityStatus, CursorPosition, PresenceInfo, PresenceManager, PresenceUpdate,
    SelectionRange, UserPresence,
};
pub use operations::{
    CRDTOperation, CRDTState, Operation, OperationComposer, OperationId, OperationInverter,
    OperationTransform, TransformResult, VectorClock,
};
pub use protocol::{
    CollaborationMessage, MessageCodec, MessageType, ProtocolError, ProtocolVersion,
    SerializedMessage,
};
pub use transport::{
    ConnectionState, ReconnectStrategy, Transport, TransportConfig, TransportEvent,
    WebSocketTransport,
};
pub use permissions::{
    EditPermission, Permission, PermissionDelegation, PermissionManager, RegionLock, ViewMode,
};
pub use crdt::{
    CADEntityCRDT, CRDTId, DocumentCRDT, DocumentSnapshot, GSet, LWWRegister,
    LamportTimestamp, ORSet, TwoPhaseSet,
};
pub use sync_engine::{
    SyncEngine, SyncEngineConfig, SyncEvent, SyncMessage, SyncState, SyncStatistics,
};
pub use versioning::{
    AuthorInfo, Branch, BranchName, MergeResult, MergeStrategy, Version, VersionControl,
    VersionDiff, VersionId,
};
pub use conflict_resolver::{
    Conflict, ConflictResolution, ConflictResolver, ConflictResolverConfig, ConflictSeverity,
    ConflictStatistics, ConflictType, ResolutionStrategy,
};

use thiserror::Error;
use uuid::Uuid;

/// Collaboration module error types
#[derive(Debug, Error)]
pub enum CollaborationError {
    #[error("Session error: {0}")]
    Session(String),

    #[error("Presence error: {0}")]
    Presence(String),

    #[error("Operation error: {0}")]
    Operation(String),

    #[error("Protocol error: {0}")]
    Protocol(#[from] protocol::ProtocolError),

    #[error("Transport error: {0}")]
    Transport(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Participant not found: {0}")]
    ParticipantNotFound(Uuid),

    #[error("Session not found: {0}")]
    SessionNotFound(Uuid),

    #[error("Session full (max participants: {0})")]
    SessionFull(usize),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Result type for collaboration operations
pub type Result<T> = std::result::Result<T, CollaborationError>;

/// Collaboration event types for notifications
#[derive(Debug, Clone)]
pub enum CollaborationEvent {
    /// A user joined the session
    UserJoined {
        session_id: Uuid,
        user_id: Uuid,
        username: String,
    },

    /// A user left the session
    UserLeft {
        session_id: Uuid,
        user_id: Uuid,
    },

    /// User presence updated (cursor, selection, etc.)
    PresenceUpdated {
        session_id: Uuid,
        user_id: Uuid,
        presence: PresenceInfo,
    },

    /// Operation applied
    OperationApplied {
        session_id: Uuid,
        user_id: Uuid,
        operation_id: Uuid,
    },

    /// Session state changed
    SessionStateChanged {
        session_id: Uuid,
        old_state: SessionState,
        new_state: SessionState,
    },

    /// Permission changed
    PermissionChanged {
        session_id: Uuid,
        user_id: Uuid,
        permission: Permission,
    },

    /// Connection state changed
    ConnectionStateChanged {
        user_id: Uuid,
        old_state: ConnectionState,
        new_state: ConnectionState,
    },

    /// Error occurred
    Error {
        session_id: Uuid,
        error: String,
    },
}

/// Callback type for collaboration events
pub type EventCallback = Box<dyn Fn(CollaborationEvent) + Send + Sync>;
