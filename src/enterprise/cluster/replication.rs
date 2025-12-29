//! State Replication
//!
//! Implements state machine interface, command log, snapshot transfer,
//! and catch-up replication for distributed state management.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};

use super::config::NodeId;

/// Replication errors
#[derive(Error, Debug)]
pub enum ReplicationError {
    #[error("State machine error: {0}")]
    StateMachine(String),
    #[error("Snapshot error: {0}")]
    Snapshot(String),
    #[error("Index out of range: {0}")]
    IndexOutOfRange(u64),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

pub type ReplicationResult<T> = Result<T, ReplicationError>;

/// Command to be replicated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub id: u64,
    pub data: Vec<u8>,
    pub timestamp: u64,
}

/// Command log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub index: u64,
    pub term: u64,
    pub command: Command,
}

/// Snapshot metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub last_included_index: u64,
    pub last_included_term: u64,
    pub data: Vec<u8>,
    pub size: usize,
    pub timestamp: u64,
}

/// State machine trait
pub trait StateMachine: Send + Sync {
    /// Apply a command to the state machine
    fn apply(&mut self, command: &Command) -> ReplicationResult<Vec<u8>>;

    /// Create a snapshot of current state
    fn snapshot(&self) -> ReplicationResult<Vec<u8>>;

    /// Restore from snapshot
    fn restore(&mut self, snapshot: &[u8]) -> ReplicationResult<()>;
}

/// In-memory state machine for testing
pub struct MemoryStateMachine {
    state: HashMap<Vec<u8>, Vec<u8>>,
}

use std::collections::HashMap;

impl MemoryStateMachine {
    pub fn new() -> Self {
        Self {
            state: HashMap::new(),
        }
    }
}

impl StateMachine for MemoryStateMachine {
    fn apply(&mut self, command: &Command) -> ReplicationResult<Vec<u8>> {
        // Simple key-value store operations
        // Command format: [op_type, key_len, key, value_len, value]
        if command.data.is_empty() {
            return Ok(Vec::new());
        }

        let op_type = command.data[0];
        match op_type {
            0 => {
                // GET
                if command.data.len() < 2 {
                    return Ok(Vec::new());
                }
                let key_len = command.data[1] as usize;
                if command.data.len() < 2 + key_len {
                    return Ok(Vec::new());
                }
                let key = command.data[2..2 + key_len].to_vec();
                Ok(self.state.get(&key).cloned().unwrap_or_default())
            }
            1 => {
                // SET
                if command.data.len() < 2 {
                    return Ok(Vec::new());
                }
                let key_len = command.data[1] as usize;
                if command.data.len() < 3 + key_len {
                    return Ok(Vec::new());
                }
                let key = command.data[2..2 + key_len].to_vec();
                let value_len = command.data[2 + key_len] as usize;
                if command.data.len() < 3 + key_len + value_len {
                    return Ok(Vec::new());
                }
                let value = command.data[3 + key_len..3 + key_len + value_len].to_vec();
                self.state.insert(key, value.clone());
                Ok(value)
            }
            2 => {
                // DELETE
                if command.data.len() < 2 {
                    return Ok(Vec::new());
                }
                let key_len = command.data[1] as usize;
                if command.data.len() < 2 + key_len {
                    return Ok(Vec::new());
                }
                let key = command.data[2..2 + key_len].to_vec();
                self.state.remove(&key);
                Ok(Vec::new())
            }
            _ => Ok(Vec::new()),
        }
    }

    fn snapshot(&self) -> ReplicationResult<Vec<u8>> {
        bincode::serialize(&self.state)
            .map_err(|e| ReplicationError::Serialization(e.to_string()))
    }

    fn restore(&mut self, snapshot: &[u8]) -> ReplicationResult<()> {
        self.state = bincode::deserialize(snapshot)
            .map_err(|e| ReplicationError::Serialization(e.to_string()))?;
        Ok(())
    }
}

/// Replication log
pub struct ReplicationLog {
    /// Log entries
    entries: VecDeque<LogEntry>,

    /// First log index
    first_index: u64,

    /// Last log index
    last_index: u64,

    /// Current snapshot
    snapshot: Option<Snapshot>,

    /// Maximum log size before compaction
    max_size: usize,
}

impl ReplicationLog {
    /// Create a new replication log
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            first_index: 1,
            last_index: 0,
            snapshot: None,
            max_size,
        }
    }

    /// Append an entry to the log
    pub fn append(&mut self, entry: LogEntry) -> ReplicationResult<u64> {
        let index = entry.index;

        // Ensure sequential indexing
        if index != self.last_index + 1 {
            return Err(ReplicationError::IndexOutOfRange(index));
        }

        self.entries.push_back(entry);
        self.last_index = index;

        // Check if compaction is needed
        if self.entries.len() > self.max_size {
            log::debug!("Log size exceeded threshold, compaction recommended");
        }

        Ok(index)
    }

    /// Get an entry by index
    pub fn get(&self, index: u64) -> Option<&LogEntry> {
        if index < self.first_index || index > self.last_index {
            return None;
        }

        let offset = (index - self.first_index) as usize;
        self.entries.get(offset)
    }

    /// Get entries in a range
    pub fn range(&self, start: u64, end: u64) -> Vec<&LogEntry> {
        let start_offset = start.saturating_sub(self.first_index) as usize;
        let end_offset = (end - self.first_index + 1) as usize;

        self.entries
            .range(start_offset..end_offset.min(self.entries.len()))
            .collect()
    }

    /// Get last entry
    pub fn last(&self) -> Option<&LogEntry> {
        self.entries.back()
    }

    /// Get first index
    pub fn first_index(&self) -> u64 {
        self.first_index
    }

    /// Get last index
    pub fn last_index(&self) -> u64 {
        self.last_index
    }

    /// Truncate log from index
    pub fn truncate(&mut self, from_index: u64) {
        if from_index <= self.first_index {
            self.entries.clear();
            self.last_index = self.first_index - 1;
            return;
        }

        let offset = (from_index - self.first_index) as usize;
        self.entries.truncate(offset);
        self.last_index = from_index - 1;
    }

    /// Compact log by creating snapshot
    pub fn compact(&mut self, snapshot: Snapshot) -> ReplicationResult<()> {
        let last_included = snapshot.last_included_index;

        if last_included < self.first_index {
            return Ok(());
        }

        // Remove entries up to snapshot point
        let remove_count = (last_included - self.first_index + 1) as usize;
        for _ in 0..remove_count.min(self.entries.len()) {
            self.entries.pop_front();
        }

        self.first_index = last_included + 1;
        self.snapshot = Some(snapshot);

        log::info!("Log compacted, new first_index: {}", self.first_index);
        Ok(())
    }

    /// Get current snapshot
    pub fn get_snapshot(&self) -> Option<&Snapshot> {
        self.snapshot.as_ref()
    }

    /// Get log size
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if log is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Replication manager
pub struct ReplicationManager {
    /// Node identifier
    node_id: NodeId,

    /// State machine
    state_machine: Arc<RwLock<Box<dyn StateMachine>>>,

    /// Replication log
    log: Arc<RwLock<ReplicationLog>>,

    /// Last applied index
    last_applied: Arc<RwLock<u64>>,

    /// Snapshot threshold
    snapshot_threshold: usize,

    /// Apply notification channel
    apply_tx: mpsc::UnboundedSender<(u64, Vec<u8>)>,
}

impl ReplicationManager {
    /// Create a new replication manager
    pub fn new(
        node_id: NodeId,
        state_machine: Box<dyn StateMachine>,
        snapshot_threshold: usize,
    ) -> (Self, mpsc::UnboundedReceiver<(u64, Vec<u8>)>) {
        let (apply_tx, apply_rx) = mpsc::unbounded_channel();

        let manager = Self {
            node_id,
            state_machine: Arc::new(RwLock::new(state_machine)),
            log: Arc::new(RwLock::new(ReplicationLog::new(snapshot_threshold))),
            last_applied: Arc::new(RwLock::new(0)),
            snapshot_threshold,
            apply_tx,
        };

        (manager, apply_rx)
    }

    /// Append command to log
    pub async fn append_command(
        &self,
        term: u64,
        command: Command,
    ) -> ReplicationResult<u64> {
        let mut log = self.log.write().await;
        let index = log.last_index() + 1;

        let _entry = LogEntry {
            index,
            term,
            command,
        };

        log.append(entry)?;
        log::debug!("Appended command at index {}", index);

        Ok(index)
    }

    /// Apply committed entries to state machine
    pub async fn apply_committed(&self, commit_index: u64) -> ReplicationResult<()> {
        let mut last_applied = self.last_applied.write().await;

        if commit_index <= *last_applied {
            return Ok(());
        }

        let log = self.log.read().await;
        let entries: Vec<LogEntry> = log
            .range(*last_applied + 1, commit_index)
            .into_iter()
            .cloned()
            .collect();
        drop(log);

        let mut state_machine = self.state_machine.write().await;

        for entry in entries {
            let result = state_machine.apply(&entry.command)?;
            *last_applied = entry.index;

            // Notify of application
            let _ = self.apply_tx.send((entry.index, result));

            log::debug!("Applied entry at index {}", entry.index);
        }

        drop(state_machine);

        // Check if snapshot is needed
        if *last_applied % self.snapshot_threshold as u64 == 0 {
            self.create_snapshot(*last_applied).await?;
        }

        Ok(())
    }

    /// Create a snapshot
    async fn create_snapshot(&self, last_index: u64) -> ReplicationResult<()> {
        log::info!("Creating snapshot at index {}", last_index);

        let state_machine = self.state_machine.read().await;
        let data = state_machine.snapshot()?;
        drop(state_machine);

        let log = self.log.read().await;
        let _entry = log.get(last_index)
            .ok_or(ReplicationError::IndexOutOfRange(last_index))?;
        let last_term = entry.term;
        drop(log);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let snapshot = Snapshot {
            last_included_index: last_index,
            last_included_term: last_term,
            size: data.len(),
            data,
            timestamp,
        };

        let mut log = self.log.write().await;
        log.compact(snapshot)?;

        log::info!("Snapshot created successfully");
        Ok(())
    }

    /// Install a snapshot from leader
    pub async fn install_snapshot(&self, snapshot: Snapshot) -> ReplicationResult<()> {
        log::info!(
            "Installing snapshot at index {}",
            snapshot.last_included_index
        );

        // Restore state machine
        let mut state_machine = self.state_machine.write().await;
        state_machine.restore(&snapshot.data)?;
        drop(state_machine);

        // Update last applied
        *self.last_applied.write().await = snapshot.last_included_index;

        // Compact log
        let mut log = self.log.write().await;
        log.compact(snapshot)?;

        log::info!("Snapshot installed successfully");
        Ok(())
    }

    /// Get current log state
    pub async fn log_state(&self) -> (u64, u64) {
        let log = self.log.read().await;
        (log.first_index(), log.last_index())
    }

    /// Get last applied index
    pub async fn last_applied(&self) -> u64 {
        *self.last_applied.read().await
    }

    /// Get snapshot if available
    pub async fn get_snapshot(&self) -> Option<Snapshot> {
        self.log.read().await.get_snapshot().cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_state_machine() {
        let mut sm = MemoryStateMachine::new();

        // SET operation
        let set_cmd = Command {
            id: 1,
            data: vec![1, 3, b'k', b'e', b'y', 5, b'v', b'a', b'l', b'u', b'e'],
            timestamp: 0,
        };

        let result = sm.apply(&set_cmd).unwrap();
        assert_eq!(result, b"value");

        // GET operation
        let get_cmd = Command {
            id: 2,
            data: vec![0, 3, b'k', b'e', b'y'],
            timestamp: 0,
        };

        let result = sm.apply(&get_cmd).unwrap();
        assert_eq!(result, b"value");
    }

    #[test]
    fn test_replication_log() {
        let mut log = ReplicationLog::new(1000);

        let _entry = LogEntry {
            index: 1,
            term: 1,
            command: Command {
                id: 1,
                data: vec![1, 2, 3],
                timestamp: 0,
            },
        };

        let index = log.append(entry).unwrap();
        assert_eq!(index, 1);
        assert_eq!(log.len(), 1);
    }

    #[test]
    fn test_log_range() {
        let mut log = ReplicationLog::new(1000);

        for i in 1..=5 {
            let _entry = LogEntry {
                index: i,
                term: 1,
                command: Command {
                    id: i,
                    data: vec![i as u8],
                    timestamp: 0,
                },
            };
            log.append(entry).unwrap();
        }

        let entries = log.range(2, 4);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].index, 2);
        assert_eq!(entries[2].index, 4);
    }

    #[tokio::test]
    async fn test_replication_manager() {
        let sm = Box::new(MemoryStateMachine::new());
        let (manager, _rx) = ReplicationManager::new("node1".to_string(), sm, 1000);

        let command = Command {
            id: 1,
            data: vec![1, 2, 3],
            timestamp: 0,
        };

        let index = manager.append_command(1, command).await.unwrap();
        assert_eq!(index, 1);
    }

    #[tokio::test]
    async fn test_apply_committed() {
        let sm = Box::new(MemoryStateMachine::new());
        let (manager, mut rx) = ReplicationManager::new("node1".to_string(), sm, 1000);

        let command = Command {
            id: 1,
            data: vec![1, 3, b'k', b'e', b'y', 5, b'v', b'a', b'l', b'u', b'e'],
            timestamp: 0,
        };

        manager.append_command(1, command).await.unwrap();
        manager.apply_committed(1).await.unwrap();

        // Check notification
        let (index, result) = rx.recv().await.unwrap();
        assert_eq!(index, 1);
        assert_eq!(result, b"value");
    }
}
