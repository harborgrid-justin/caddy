//! Collaboration Communication Protocol
//!
//! This module defines the binary protocol for efficient communication between
//! collaboration clients, including message types, serialization, and versioning.

use super::{
    CRDTOperation, Operation, PresenceUpdate, SessionId, SessionState,
};
use crate::enterprise::collaboration::operations::OperationWithMetadata;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use thiserror::Error;
use uuid::Uuid;

/// Protocol version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl ProtocolVersion {
    /// Current protocol version
    pub const CURRENT: Self = Self {
        major: 1,
        minor: 0,
        patch: 0,
    };

    /// Check if this version is compatible with another
    pub fn is_compatible(&self, other: &ProtocolVersion) -> bool {
        self.major == other.major
    }

    /// Convert to u32 for comparison
    pub fn as_u32(&self) -> u32 {
        ((self.major as u32) << 16) | ((self.minor as u32) << 8) | (self.patch as u32)
    }
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        Self::CURRENT
    }
}

/// Protocol error types
#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Incompatible protocol version: {0:?}")]
    IncompatibleVersion(ProtocolVersion),

    #[error("Invalid message type: {0}")]
    InvalidMessageType(u8),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Bincode error: {0}")]
    Bincode(#[from] bincode::Error),

    #[error("Message too large: {0} bytes (max: {1})")]
    MessageTooLarge(usize, usize),

    #[error("Invalid checksum")]
    InvalidChecksum,
}

/// Message type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    // Session management
    JoinSession = 0x01,
    LeaveSession = 0x02,
    SessionState = 0x03,
    SessionInfo = 0x04,

    // Operations
    Operation = 0x10,
    OperationAck = 0x11,
    OperationBatch = 0x12,

    // Presence
    PresenceUpdate = 0x20,
    PresenceRequest = 0x21,
    PresenceSnapshot = 0x22,

    // Permissions
    PermissionRequest = 0x30,
    PermissionGrant = 0x31,
    PermissionRevoke = 0x32,

    // Control
    Heartbeat = 0x40,
    HeartbeatAck = 0x41,
    Error = 0x42,
    Disconnect = 0x43,

    // Sync
    SyncRequest = 0x50,
    SyncResponse = 0x51,
    StateSnapshot = 0x52,

    // CRDT
    CRDTOperation = 0x60,
    CRDTMerge = 0x61,
}

impl MessageType {
    /// Convert from u8
    pub fn from_u8(value: u8) -> Result<Self, ProtocolError> {
        match value {
            0x01 => Ok(Self::JoinSession),
            0x02 => Ok(Self::LeaveSession),
            0x03 => Ok(Self::SessionState),
            0x04 => Ok(Self::SessionInfo),
            0x10 => Ok(Self::Operation),
            0x11 => Ok(Self::OperationAck),
            0x12 => Ok(Self::OperationBatch),
            0x20 => Ok(Self::PresenceUpdate),
            0x21 => Ok(Self::PresenceRequest),
            0x22 => Ok(Self::PresenceSnapshot),
            0x30 => Ok(Self::PermissionRequest),
            0x31 => Ok(Self::PermissionGrant),
            0x32 => Ok(Self::PermissionRevoke),
            0x40 => Ok(Self::Heartbeat),
            0x41 => Ok(Self::HeartbeatAck),
            0x42 => Ok(Self::Error),
            0x43 => Ok(Self::Disconnect),
            0x50 => Ok(Self::SyncRequest),
            0x51 => Ok(Self::SyncResponse),
            0x52 => Ok(Self::StateSnapshot),
            0x60 => Ok(Self::CRDTOperation),
            0x61 => Ok(Self::CRDTMerge),
            _ => Err(ProtocolError::InvalidMessageType(value)),
        }
    }

    /// Convert to u8
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}

/// Collaboration message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollaborationMessage {
    // Session management
    JoinSession {
        session_id: SessionId,
        user_id: Uuid,
        username: String,
        auth_token: Option<String>,
    },
    LeaveSession {
        session_id: SessionId,
        user_id: Uuid,
    },
    SessionState {
        session_id: SessionId,
        state: SessionState,
    },
    SessionInfo {
        session_id: SessionId,
        participant_count: usize,
        operation_count: u64,
    },

    // Operations
    Operation {
        operation: OperationWithMetadata,
    },
    OperationAck {
        operation_id: Uuid,
        success: bool,
        error: Option<String>,
    },
    OperationBatch {
        operations: Vec<OperationWithMetadata>,
    },

    // Presence
    PresenceUpdate {
        update: PresenceUpdate,
    },
    PresenceRequest {
        session_id: SessionId,
    },
    PresenceSnapshot {
        session_id: SessionId,
        presences: Vec<super::PresenceInfo>,
    },

    // Permissions
    PermissionRequest {
        user_id: Uuid,
        resource: String,
        permission_type: String,
    },
    PermissionGrant {
        user_id: Uuid,
        resource: String,
        permission: super::Permission,
    },
    PermissionRevoke {
        user_id: Uuid,
        resource: String,
    },

    // Control
    Heartbeat {
        timestamp: i64,
    },
    HeartbeatAck {
        timestamp: i64,
        server_time: i64,
    },
    Error {
        code: u32,
        message: String,
    },
    Disconnect {
        reason: String,
    },

    // Sync
    SyncRequest {
        session_id: SessionId,
        since_operation: Option<Uuid>,
    },
    SyncResponse {
        session_id: SessionId,
        operations: Vec<OperationWithMetadata>,
        complete: bool,
    },
    StateSnapshot {
        session_id: SessionId,
        snapshot_data: Vec<u8>,
    },

    // CRDT
    CRDTOperation {
        operation: CRDTOperation,
    },
    CRDTMerge {
        session_id: SessionId,
        state: super::CRDTState,
    },
}

impl CollaborationMessage {
    /// Get the message type
    pub fn message_type(&self) -> MessageType {
        match self {
            Self::JoinSession { .. } => MessageType::JoinSession,
            Self::LeaveSession { .. } => MessageType::LeaveSession,
            Self::SessionState { .. } => MessageType::SessionState,
            Self::SessionInfo { .. } => MessageType::SessionInfo,
            Self::Operation { .. } => MessageType::Operation,
            Self::OperationAck { .. } => MessageType::OperationAck,
            Self::OperationBatch { .. } => MessageType::OperationBatch,
            Self::PresenceUpdate { .. } => MessageType::PresenceUpdate,
            Self::PresenceRequest { .. } => MessageType::PresenceRequest,
            Self::PresenceSnapshot { .. } => MessageType::PresenceSnapshot,
            Self::PermissionRequest { .. } => MessageType::PermissionRequest,
            Self::PermissionGrant { .. } => MessageType::PermissionGrant,
            Self::PermissionRevoke { .. } => MessageType::PermissionRevoke,
            Self::Heartbeat { .. } => MessageType::Heartbeat,
            Self::HeartbeatAck { .. } => MessageType::HeartbeatAck,
            Self::Error { .. } => MessageType::Error,
            Self::Disconnect { .. } => MessageType::Disconnect,
            Self::SyncRequest { .. } => MessageType::SyncRequest,
            Self::SyncResponse { .. } => MessageType::SyncResponse,
            Self::StateSnapshot { .. } => MessageType::StateSnapshot,
            Self::CRDTOperation { .. } => MessageType::CRDTOperation,
            Self::CRDTMerge { .. } => MessageType::CRDTMerge,
        }
    }
}

/// Serialized message with header
#[derive(Debug, Clone)]
pub struct SerializedMessage {
    /// Protocol version
    pub version: ProtocolVersion,
    /// Message type
    pub message_type: MessageType,
    /// Message payload
    pub payload: Vec<u8>,
    /// Checksum (CRC32)
    pub checksum: u32,
}

/// Message codec for serialization/deserialization
pub struct MessageCodec;

impl MessageCodec {
    /// Maximum message size (16 MB)
    pub const MAX_MESSAGE_SIZE: usize = 16 * 1024 * 1024;

    /// Message header size
    const HEADER_SIZE: usize = 12; // version(3) + type(1) + length(4) + checksum(4)

    /// CRC32 lookup table
    const CRC32_TABLE: [u32; 256] = Self::generate_crc32_table_const();

    /// Serialize a message to bytes
    pub fn serialize(message: &CollaborationMessage) -> Result<Vec<u8>, ProtocolError> {
        // Serialize payload using bincode
        let payload = bincode::serialize(message)
            .map_err(|e| ProtocolError::Serialization(e.to_string()))?;

        if payload.len() > Self::MAX_MESSAGE_SIZE {
            return Err(ProtocolError::MessageTooLarge(
                payload.len(),
                Self::MAX_MESSAGE_SIZE,
            ));
        }

        let version = ProtocolVersion::CURRENT;
        let message_type = message.message_type();
        let checksum = Self::calculate_checksum(&payload);

        let serialized = SerializedMessage {
            version,
            message_type,
            payload,
            checksum,
        };

        Self::write_message(&serialized)
    }

    /// Deserialize a message from bytes
    pub fn deserialize(data: &[u8]) -> Result<CollaborationMessage, ProtocolError> {
        let serialized = Self::read_message(data)?;

        // Verify checksum
        let computed_checksum = Self::calculate_checksum(&serialized.payload);
        if computed_checksum != serialized.checksum {
            return Err(ProtocolError::InvalidChecksum);
        }

        // Check version compatibility
        if !serialized.version.is_compatible(&ProtocolVersion::CURRENT) {
            return Err(ProtocolError::IncompatibleVersion(serialized.version));
        }

        // Deserialize payload
        bincode::deserialize(&serialized.payload)
            .map_err(|e| ProtocolError::Deserialization(e.to_string()))
    }

    /// Write a serialized message to bytes
    fn write_message(msg: &SerializedMessage) -> Result<Vec<u8>, ProtocolError> {
        let mut buffer = Vec::with_capacity(Self::HEADER_SIZE + msg.payload.len());

        // Write version (3 bytes)
        buffer.push(msg.version.major);
        buffer.push(msg.version.minor);
        buffer.push(msg.version.patch);

        // Write message type (1 byte)
        buffer.push(msg.message_type.as_u8());

        // Write payload length (4 bytes, big-endian)
        let length = msg.payload.len() as u32;
        buffer.extend_from_slice(&length.to_be_bytes());

        // Write checksum (4 bytes, big-endian)
        buffer.extend_from_slice(&msg.checksum.to_be_bytes());

        // Write payload
        buffer.extend_from_slice(&msg.payload);

        Ok(buffer)
    }

    /// Read a serialized message from bytes
    fn read_message(data: &[u8]) -> Result<SerializedMessage, ProtocolError> {
        if data.len() < Self::HEADER_SIZE {
            return Err(ProtocolError::Io(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Insufficient data for header",
            )));
        }

        // Read version
        let version = ProtocolVersion {
            major: data[0],
            minor: data[1],
            patch: data[2],
        };

        // Read message type
        let message_type = MessageType::from_u8(data[3])?;

        // Read payload length
        let length = u32::from_be_bytes([data[4], data[5], data[6], data[7]]) as usize;

        if length > Self::MAX_MESSAGE_SIZE {
            return Err(ProtocolError::MessageTooLarge(length, Self::MAX_MESSAGE_SIZE));
        }

        // Read checksum
        let checksum = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);

        // Read payload
        let payload_start = Self::HEADER_SIZE;
        let payload_end = payload_start + length;

        if data.len() < payload_end {
            return Err(ProtocolError::Io(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Insufficient data for payload",
            )));
        }

        let payload = data[payload_start..payload_end].to_vec();

        Ok(SerializedMessage {
            version,
            message_type,
            payload,
            checksum,
        })
    }

    /// Calculate CRC32 checksum
    fn calculate_checksum(data: &[u8]) -> u32 {
        let mut crc = 0xFFFFFFFF_u32;
        for &byte in data {
            let index = ((crc ^ byte as u32) & 0xFF) as usize;
            crc = (crc >> 8) ^ Self::CRC32_TABLE[index];
        }
        !crc
    }

    /// Generate CRC32 lookup table
    const fn generate_crc32_table_const() -> [u32; 256] {
        let mut table = [0u32; 256];
        let mut i = 0;
        while i < 256 {
            let mut crc = i as u32;
            let mut j = 0;
            while j < 8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ 0xEDB88320;
                } else {
                    crc >>= 1;
                }
                j += 1;
            }
            table[i] = crc;
            i += 1;
        }
        table
    }

    /// Stream writer for messages
    pub fn write_to<W: Write>(
        writer: &mut W,
        message: &CollaborationMessage,
    ) -> Result<(), ProtocolError> {
        let data = Self::serialize(message)?;
        writer.write_all(&data)?;
        writer.flush()?;
        Ok(())
    }

    /// Stream reader for messages
    pub fn read_from<R: Read>(reader: &mut R) -> Result<CollaborationMessage, ProtocolError> {
        // Read header first
        let mut header = [0u8; Self::HEADER_SIZE];
        reader.read_exact(&mut header)?;

        // Parse length from header
        let length = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;

        if length > Self::MAX_MESSAGE_SIZE {
            return Err(ProtocolError::MessageTooLarge(length, Self::MAX_MESSAGE_SIZE));
        }

        // Read full message
        let mut full_data = Vec::with_capacity(Self::HEADER_SIZE + length);
        full_data.extend_from_slice(&header);

        let mut payload = vec![0u8; length];
        reader.read_exact(&mut payload)?;
        full_data.extend_from_slice(&payload);

        Self::deserialize(&full_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_protocol_version() {
        let v1 = ProtocolVersion { major: 1, minor: 0, patch: 0 };
        let v2 = ProtocolVersion { major: 1, minor: 1, patch: 0 };
        let v3 = ProtocolVersion { major: 2, minor: 0, patch: 0 };

        assert!(v1.is_compatible(&v2));
        assert!(!v1.is_compatible(&v3));
    }

    #[test]
    fn test_message_serialization() {
        let message = CollaborationMessage::Heartbeat {
            timestamp: Utc::now().timestamp(),
        };

        let serialized = MessageCodec::serialize(&message).unwrap();
        assert!(serialized.len() > MessageCodec::HEADER_SIZE);

        let deserialized = MessageCodec::deserialize(&serialized).unwrap();

        match deserialized {
            CollaborationMessage::Heartbeat { .. } => {}
            _ => panic!("Expected Heartbeat message"),
        }
    }

    #[test]
    fn test_checksum() {
        let data = b"Hello, World!";
        let checksum1 = MessageCodec::calculate_checksum(data);
        let checksum2 = MessageCodec::calculate_checksum(data);

        assert_eq!(checksum1, checksum2);

        let different_data = b"Hello, World?";
        let checksum3 = MessageCodec::calculate_checksum(different_data);

        assert_ne!(checksum1, checksum3);
    }

    #[test]
    fn test_message_roundtrip() {
        let session_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let message = CollaborationMessage::JoinSession {
            session_id,
            user_id,
            username: "Alice".to_string(),
            auth_token: Some("token123".to_string()),
        };

        let serialized = MessageCodec::serialize(&message).unwrap();
        let deserialized = MessageCodec::deserialize(&serialized).unwrap();

        match deserialized {
            CollaborationMessage::JoinSession {
                session_id: sid,
                user_id: uid,
                username,
                ..
            } => {
                assert_eq!(sid, session_id);
                assert_eq!(uid, user_id);
                assert_eq!(username, "Alice");
            }
            _ => panic!("Expected JoinSession message"),
        }
    }
}
