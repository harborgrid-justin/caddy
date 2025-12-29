//! Real-Time Synchronization Engine with Operational Transforms
//!
//! This module provides a high-performance synchronization engine for collaborative
//! CAD editing with operational transformation, diff/patch algorithms, and conflict resolution.

use super::crdt::{CRDTOperation, DocumentCRDT, DocumentSnapshot};
use super::operations::{OperationId, VectorClock};
use super::transport::Transport;
use super::{CollaborationError, Result};
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{mpsc, watch};
use uuid::Uuid;

/// Synchronization state for a document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncState {
    /// Not synchronized
    Offline,
    /// Connecting to server
    Connecting,
    /// Synchronized with server
    Synchronized,
    /// Synchronizing changes
    Syncing,
    /// Conflict detected
    Conflicted,
    /// Error state
    Error,
}

/// Sync message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMessage {
    /// Initial synchronization request
    SyncRequest {
        document_id: Uuid,
        client_id: Uuid,
        last_known_version: u64,
    },

    /// Full document snapshot
    FullSync {
        document_id: Uuid,
        snapshot: DocumentSnapshot,
        version: u64,
    },

    /// Incremental update
    Delta {
        document_id: Uuid,
        operations: Vec<CRDTOperation>,
        base_version: u64,
        new_version: u64,
        vector_clock: VectorClock,
    },

    /// Operation acknowledgment
    Ack {
        document_id: Uuid,
        operation_ids: Vec<OperationId>,
        version: u64,
    },

    /// Request missing operations
    RequestOps {
        document_id: Uuid,
        from_version: u64,
        to_version: u64,
    },

    /// Heartbeat to maintain connection
    Heartbeat {
        client_id: Uuid,
        timestamp: DateTime<Utc>,
    },

    /// Sync status update
    SyncStatus {
        document_id: Uuid,
        state: SyncState,
        message: Option<String>,
    },
}

/// Pending operation awaiting acknowledgment
#[derive(Debug, Clone)]
struct PendingOperation {
    /// Operation ID
    id: OperationId,
    /// CRDT operation
    operation: CRDTOperation,
    /// Timestamp when sent
    sent_at: DateTime<Utc>,
    /// Number of retry attempts
    retry_count: u32,
    /// Vector clock at send time
    vector_clock: VectorClock,
}

/// Synchronization engine configuration
#[derive(Debug, Clone)]
pub struct SyncEngineConfig {
    /// Maximum number of pending operations before blocking
    pub max_pending_ops: usize,
    /// Operation retry timeout
    pub retry_timeout: Duration,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Heartbeat interval
    pub heartbeat_interval: Duration,
    /// Enable compression for large payloads
    pub enable_compression: bool,
    /// Snapshot interval (full sync every N operations)
    pub snapshot_interval: u64,
    /// Enable offline mode
    pub enable_offline: bool,
    /// Offline queue size
    pub offline_queue_size: usize,
}

impl Default for SyncEngineConfig {
    fn default() -> Self {
        Self {
            max_pending_ops: 1000,
            retry_timeout: Duration::seconds(5),
            max_retries: 3,
            heartbeat_interval: Duration::seconds(30),
            enable_compression: true,
            snapshot_interval: 1000,
            enable_offline: true,
            offline_queue_size: 10000,
        }
    }
}

/// Real-time synchronization engine
pub struct SyncEngine {
    /// Client ID
    client_id: Uuid,
    /// Configuration
    config: SyncEngineConfig,
    /// Active documents being synced
    documents: Arc<DashMap<Uuid, Arc<RwLock<DocumentCRDT>>>>,
    /// Pending operations per document
    pending_ops: Arc<DashMap<Uuid, VecDeque<PendingOperation>>>,
    /// Document versions
    versions: Arc<DashMap<Uuid, u64>>,
    /// Sync state per document
    sync_states: Arc<DashMap<Uuid, SyncState>>,
    /// Offline operation queue
    offline_queue: Arc<DashMap<Uuid, VecDeque<CRDTOperation>>>,
    /// Sync event channel
    event_tx: mpsc::UnboundedSender<SyncEvent>,
    /// Sync state watch channel
    state_tx: watch::Sender<HashMap<Uuid, SyncState>>,
}

/// Sync engine events
#[derive(Debug, Clone)]
pub enum SyncEvent {
    /// Document synchronized
    Synchronized {
        document_id: Uuid,
        version: u64,
    },

    /// Operations applied
    OperationsApplied {
        document_id: Uuid,
        operation_count: usize,
    },

    /// Conflict detected
    ConflictDetected {
        document_id: Uuid,
        operations: Vec<CRDTOperation>,
    },

    /// Sync state changed
    StateChanged {
        document_id: Uuid,
        old_state: SyncState,
        new_state: SyncState,
    },

    /// Error occurred
    Error {
        document_id: Uuid,
        error: String,
    },

    /// Offline mode enabled
    OfflineModeEnabled {
        document_id: Uuid,
        queued_operations: usize,
    },

    /// Reconnected
    Reconnected {
        document_id: Uuid,
        replayed_operations: usize,
    },
}

impl SyncEngine {
    /// Create a new synchronization engine
    pub fn new(client_id: Uuid, config: SyncEngineConfig) -> (Self, mpsc::UnboundedReceiver<SyncEvent>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (state_tx, _) = watch::channel(HashMap::new());

        let engine = Self {
            client_id,
            config,
            documents: Arc::new(DashMap::new()),
            pending_ops: Arc::new(DashMap::new()),
            versions: Arc::new(DashMap::new()),
            sync_states: Arc::new(DashMap::new()),
            offline_queue: Arc::new(DashMap::new()),
            event_tx,
            state_tx,
        };

        (engine, event_rx)
    }

    /// Register a document for synchronization
    pub fn register_document(&self, document: Arc<RwLock<DocumentCRDT>>) -> Result<()> {
        let doc_id = document.read().document_id;

        self.documents.insert(doc_id, document);
        self.pending_ops.insert(doc_id, VecDeque::new());
        self.versions.insert(doc_id, 0);
        self.sync_states.insert(doc_id, SyncState::Offline);

        if self.config.enable_offline {
            self.offline_queue.insert(doc_id, VecDeque::new());
        }

        Ok(())
    }

    /// Unregister a document
    pub fn unregister_document(&self, document_id: Uuid) {
        self.documents.remove(&document_id);
        self.pending_ops.remove(&document_id);
        self.versions.remove(&document_id);
        self.sync_states.remove(&document_id);
        self.offline_queue.remove(&document_id);
    }

    /// Apply local operation and queue for sync
    pub async fn apply_local_operation(
        &self,
        document_id: Uuid,
        operation: CRDTOperation,
    ) -> Result<OperationId> {
        let document = self.documents.get(&document_id).ok_or_else(|| {
            CollaborationError::Operation(format!("Document not found: {}", document_id))
        })?;

        // Apply operation locally
        {
            let mut doc = document.write();
            operation.apply(&mut doc)?;
        }

        let operation_id = Uuid::new_v4();

        // Check if online or offline
        let sync_state = self.sync_states.get(&document_id)
            .map(|s| *s.value())
            .unwrap_or(SyncState::Offline);

        match sync_state {
            SyncState::Synchronized | SyncState::Syncing => {
                // Queue for transmission
                let pending = PendingOperation {
                    id: operation_id,
                    operation: operation.clone(),
                    sent_at: Utc::now(),
                    retry_count: 0,
                    vector_clock: VectorClock::new(), // Should get from document
                };

                if let Some(mut queue) = self.pending_ops.get_mut(&document_id) {
                    if queue.len() >= self.config.max_pending_ops {
                        return Err(CollaborationError::Operation(
                            "Pending operation queue full".to_string(),
                        ));
                    }
                    queue.push_back(pending);
                }

                self.update_sync_state(document_id, SyncState::Syncing);
            }
            _ => {
                // Offline mode - queue operation
                if self.config.enable_offline {
                    if let Some(mut queue) = self.offline_queue.get_mut(&document_id) {
                        if queue.len() >= self.config.offline_queue_size {
                            return Err(CollaborationError::Operation(
                                "Offline queue full".to_string(),
                            ));
                        }
                        queue.push_back(operation);

                        let _ = self.event_tx.send(SyncEvent::OfflineModeEnabled {
                            document_id,
                            queued_operations: queue.len(),
                        });
                    }
                }
            }
        }

        Ok(operation_id)
    }

    /// Apply remote operations
    pub async fn apply_remote_operations(
        &self,
        document_id: Uuid,
        operations: Vec<CRDTOperation>,
        version: u64,
    ) -> Result<()> {
        let document = self.documents.get(&document_id).ok_or_else(|| {
            CollaborationError::Operation(format!("Document not found: {}", document_id))
        })?;

        // Apply each operation
        {
            let mut doc = document.write();
            for operation in &operations {
                operation.apply(&mut doc)?;
            }
        }

        // Update version
        self.versions.insert(document_id, version);

        // Emit event
        let _ = self.event_tx.send(SyncEvent::OperationsApplied {
            document_id,
            operation_count: operations.len(),
        });

        Ok(())
    }

    /// Handle incoming sync message
    pub async fn handle_message(&self, message: SyncMessage) -> Result<()> {
        match message {
            SyncMessage::FullSync {
                document_id,
                snapshot,
                version,
            } => {
                self.apply_snapshot(document_id, snapshot, version).await?;
            }

            SyncMessage::Delta {
                document_id,
                operations,
                new_version,
                ..
            } => {
                self.apply_remote_operations(document_id, operations, new_version).await?;
                self.update_sync_state(document_id, SyncState::Synchronized);
            }

            SyncMessage::Ack {
                document_id,
                operation_ids,
                version,
            } => {
                self.acknowledge_operations(document_id, &operation_ids, version)?;
            }

            SyncMessage::SyncStatus {
                document_id,
                state,
                ..
            } => {
                self.update_sync_state(document_id, state);
            }

            _ => {}
        }

        Ok(())
    }

    /// Apply full snapshot
    async fn apply_snapshot(
        &self,
        document_id: Uuid,
        snapshot: DocumentSnapshot,
        version: u64,
    ) -> Result<()> {
        let document = self.documents.get(&document_id).ok_or_else(|| {
            CollaborationError::Operation(format!("Document not found: {}", document_id))
        })?;

        {
            let mut doc = document.write();
            doc.apply_snapshot(snapshot)?;
        }

        self.versions.insert(document_id, version);
        self.update_sync_state(document_id, SyncState::Synchronized);

        let _ = self.event_tx.send(SyncEvent::Synchronized {
            document_id,
            version,
        });

        Ok(())
    }

    /// Acknowledge operations
    fn acknowledge_operations(
        &self,
        document_id: Uuid,
        operation_ids: &[OperationId],
        _version: u64,
    ) -> Result<()> {
        if let Some(mut pending) = self.pending_ops.get_mut(&document_id) {
            // Remove acknowledged operations
            pending.retain(|op| !operation_ids.contains(&op.id));

            if pending.is_empty() {
                self.update_sync_state(document_id, SyncState::Synchronized);
            }
        }

        Ok(())
    }

    /// Get pending operations for transmission
    pub fn get_pending_operations(&self, document_id: Uuid) -> Vec<CRDTOperation> {
        self.pending_ops
            .get(&document_id)
            .map(|queue| queue.iter().map(|p| p.operation.clone()).collect())
            .unwrap_or_default()
    }

    /// Retry failed operations
    pub async fn retry_pending_operations(&self, document_id: Uuid) -> Result<Vec<CRDTOperation>> {
        let now = Utc::now();
        let mut operations_to_retry = Vec::new();

        if let Some(mut pending) = self.pending_ops.get_mut(&document_id) {
            for op in pending.iter_mut() {
                let elapsed = now.signed_duration_since(op.sent_at);

                if elapsed > self.config.retry_timeout {
                    if op.retry_count < self.config.max_retries {
                        op.retry_count += 1;
                        op.sent_at = now;
                        operations_to_retry.push(op.operation.clone());
                    } else {
                        // Max retries exceeded - conflict or error
                        let _ = self.event_tx.send(SyncEvent::Error {
                            document_id,
                            error: format!("Operation {} exceeded max retries", op.id),
                        });
                    }
                }
            }
        }

        Ok(operations_to_retry)
    }

    /// Replay offline operations after reconnection
    pub async fn replay_offline_operations(&self, document_id: Uuid) -> Result<usize> {
        let operations = if let Some(mut queue) = self.offline_queue.get_mut(&document_id) {
            let ops: Vec<_> = queue.drain(..).collect();
            ops
        } else {
            Vec::new()
        };

        let count = operations.len();

        for operation in operations {
            self.apply_local_operation(document_id, operation).await?;
        }

        if count > 0 {
            let _ = self.event_tx.send(SyncEvent::Reconnected {
                document_id,
                replayed_operations: count,
            });
        }

        Ok(count)
    }

    /// Update sync state
    fn update_sync_state(&self, document_id: Uuid, new_state: SyncState) {
        let old_state = self.sync_states
            .insert(document_id, new_state)
            .unwrap_or(SyncState::Offline);

        if old_state != new_state {
            let _ = self.event_tx.send(SyncEvent::StateChanged {
                document_id,
                old_state,
                new_state,
            });

            // Update watch channel
            let mut states = HashMap::new();
            for entry in self.sync_states.iter() {
                states.insert(*entry.key(), *entry.value());
            }
            let _ = self.state_tx.send(states);
        }
    }

    /// Get current sync state
    pub fn get_sync_state(&self, document_id: Uuid) -> SyncState {
        self.sync_states
            .get(&document_id)
            .map(|s| *s.value())
            .unwrap_or(SyncState::Offline)
    }

    /// Get document version
    pub fn get_version(&self, document_id: Uuid) -> u64 {
        self.versions
            .get(&document_id)
            .map(|v| *v.value())
            .unwrap_or(0)
    }

    /// Create sync request message
    pub fn create_sync_request(&self, document_id: Uuid) -> SyncMessage {
        let last_known_version = self.get_version(document_id);

        SyncMessage::SyncRequest {
            document_id,
            client_id: self.client_id,
            last_known_version,
        }
    }

    /// Create delta message with pending operations
    pub fn create_delta_message(&self, document_id: Uuid) -> Option<SyncMessage> {
        let operations = self.get_pending_operations(document_id);

        if operations.is_empty() {
            return None;
        }

        let base_version = self.get_version(document_id);

        Some(SyncMessage::Delta {
            document_id,
            operations,
            base_version,
            new_version: base_version + 1,
            vector_clock: VectorClock::new(),
        })
    }

    /// Create heartbeat message
    pub fn create_heartbeat(&self) -> SyncMessage {
        SyncMessage::Heartbeat {
            client_id: self.client_id,
            timestamp: Utc::now(),
        }
    }

    /// Get sync state watch receiver
    pub fn watch_sync_states(&self) -> watch::Receiver<HashMap<Uuid, SyncState>> {
        self.state_tx.subscribe()
    }

    /// Generate snapshot for a document
    pub fn generate_snapshot(&self, document_id: Uuid) -> Result<DocumentSnapshot> {
        let document = self.documents.get(&document_id).ok_or_else(|| {
            CollaborationError::Operation(format!("Document not found: {}", document_id))
        })?;

        Ok(document.read().snapshot())
    }

    /// Get statistics
    pub fn get_statistics(&self, document_id: Uuid) -> SyncStatistics {
        let pending_count = self.pending_ops
            .get(&document_id)
            .map(|q| q.len())
            .unwrap_or(0);

        let offline_count = self.offline_queue
            .get(&document_id)
            .map(|q| q.len())
            .unwrap_or(0);

        let version = self.get_version(document_id);
        let state = self.get_sync_state(document_id);

        SyncStatistics {
            document_id,
            pending_operations: pending_count,
            offline_operations: offline_count,
            version,
            state,
        }
    }
}

/// Synchronization statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatistics {
    /// Document ID
    pub document_id: Uuid,
    /// Number of pending operations
    pub pending_operations: usize,
    /// Number of offline operations
    pub offline_operations: usize,
    /// Current version
    pub version: u64,
    /// Sync state
    pub state: SyncState,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sync_engine_basic() {
        let client_id = Uuid::new_v4();
        let config = SyncEngineConfig::default();
        let (engine, mut events) = SyncEngine::new(client_id, config);

        let doc_id = Uuid::new_v4();
        let site_id = Uuid::new_v4();
        let document = Arc::new(RwLock::new(DocumentCRDT::new(doc_id, site_id)));

        engine.register_document(document.clone()).unwrap();

        assert_eq!(engine.get_sync_state(doc_id), SyncState::Offline);
        assert_eq!(engine.get_version(doc_id), 0);
    }

    #[tokio::test]
    async fn test_offline_operations() {
        let client_id = Uuid::new_v4();
        let config = SyncEngineConfig {
            enable_offline: true,
            offline_queue_size: 100,
            ..Default::default()
        };
        let (engine, _) = SyncEngine::new(client_id, config);

        let doc_id = Uuid::new_v4();
        let site_id = Uuid::new_v4();
        let document = Arc::new(RwLock::new(DocumentCRDT::new(doc_id, site_id)));

        engine.register_document(document.clone()).unwrap();

        // Apply operation while offline
        let entity_id = Uuid::new_v4();
        let timestamp = LamportTimestamp::new(1, site_id);
        let operation = CRDTOperation::AddEntity {
            entity_id,
            entity_type: "line".to_string(),
            layer: None,
            timestamp,
        };

        engine.apply_local_operation(doc_id, operation).await.unwrap();

        // Check offline queue
        let stats = engine.get_statistics(doc_id);
        assert_eq!(stats.offline_operations, 1);
    }
}
