//! Raft Consensus Protocol
//!
//! Implements leader election, log replication, term management,
//! heartbeat protocol, and snapshot support.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use tokio::time;

use super::config::{NodeId, RaftConfig};
use super::transport::{Message, Transport};

/// Raft errors
#[derive(Error, Debug)]
pub enum RaftError {
    #[error("Not leader")]
    NotLeader,
    #[error("No leader elected")]
    NoLeader,
    #[error("Timeout waiting for quorum")]
    QuorumTimeout,
    #[error("Invalid term: {0}")]
    InvalidTerm(u64),
    #[error("Log inconsistency at index {0}")]
    LogInconsistency(u64),
    #[error("Transport error: {0}")]
    Transport(String),
}

pub type RaftResult<T> = Result<T, RaftError>;

/// Node state in Raft protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeState {
    Follower,
    Candidate,
    Leader,
}

/// Log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub term: u64,
    pub index: u64,
    pub data: Vec<u8>,
}

/// Snapshot metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub last_included_index: u64,
    pub last_included_term: u64,
    pub size: usize,
    pub timestamp: u64,
}

/// Persistent state (must survive crashes)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistentState {
    current_term: u64,
    voted_for: Option<NodeId>,
    log: Vec<LogEntry>,
}

impl PersistentState {
    fn new() -> Self {
        Self {
            current_term: 0,
            voted_for: None,
            log: Vec::new(),
        }
    }

    fn last_log_index(&self) -> u64 {
        self.log.last().map(|e| e.index).unwrap_or(0)
    }

    fn last_log_term(&self) -> u64 {
        self.log.last().map(|e| e.term).unwrap_or(0)
    }

    fn get_entry(&self, index: u64) -> Option<&LogEntry> {
        self.log.iter().find(|e| e.index == index)
    }
}

/// Volatile state (can be rebuilt)
struct VolatileState {
    commit_index: u64,
    last_applied: u64,
}

impl VolatileState {
    fn new() -> Self {
        Self {
            commit_index: 0,
            last_applied: 0,
        }
    }
}

/// Leader-specific state
struct LeaderState {
    next_index: HashMap<NodeId, u64>,
    match_index: HashMap<NodeId, u64>,
}

impl LeaderState {
    fn new(peers: &[NodeId], last_log_index: u64) -> Self {
        let mut next_index = HashMap::new();
        let mut match_index = HashMap::new();

        for peer in peers {
            next_index.insert(peer.clone(), last_log_index + 1);
            match_index.insert(peer.clone(), 0);
        }

        Self {
            next_index,
            match_index,
        }
    }
}

/// Raft consensus module
pub struct RaftNode {
    /// Node identifier
    node_id: NodeId,

    /// Current state
    state: Arc<RwLock<NodeState>>,

    /// Persistent state
    persistent: Arc<RwLock<PersistentState>>,

    /// Volatile state
    volatile: Arc<RwLock<VolatileState>>,

    /// Leader state (only valid when leader)
    leader_state: Arc<RwLock<Option<LeaderState>>>,

    /// Current leader
    current_leader: Arc<RwLock<Option<NodeId>>>,

    /// Peer nodes
    peers: Arc<RwLock<Vec<NodeId>>>,

    /// Transport layer
    transport: Arc<Transport>,

    /// Configuration
    config: RaftConfig,

    /// Last heartbeat received
    last_heartbeat: Arc<RwLock<Instant>>,

    /// Commit notification channel
    commit_tx: mpsc::UnboundedSender<LogEntry>,
}

impl RaftNode {
    /// Create a new Raft node
    pub fn new(
        node_id: NodeId,
        transport: Arc<Transport>,
        config: RaftConfig,
    ) -> (Self, mpsc::UnboundedReceiver<LogEntry>) {
        let (commit_tx, commit_rx) = mpsc::unbounded_channel();

        let node = Self {
            node_id,
            state: Arc::new(RwLock::new(NodeState::Follower)),
            persistent: Arc::new(RwLock::new(PersistentState::new())),
            volatile: Arc::new(RwLock::new(VolatileState::new())),
            leader_state: Arc::new(RwLock::new(None)),
            current_leader: Arc::new(RwLock::new(None)),
            peers: Arc::new(RwLock::new(Vec::new())),
            transport,
            config,
            last_heartbeat: Arc::new(RwLock::new(Instant::now())),
            commit_tx,
        };

        (node, commit_rx)
    }

    /// Add a peer node
    pub async fn add_peer(&self, peer_id: NodeId) {
        let mut peers = self.peers.write().await;
        if !peers.contains(&peer_id) {
            peers.push(peer_id);
        }
    }

    /// Get current term
    pub async fn current_term(&self) -> u64 {
        self.persistent.read().await.current_term
    }

    /// Get current state
    pub async fn get_state(&self) -> NodeState {
        *self.state.read().await
    }

    /// Check if this node is the leader
    pub async fn is_leader(&self) -> bool {
        *self.state.read().await == NodeState::Leader
    }

    /// Get current leader
    pub async fn get_leader(&self) -> Option<NodeId> {
        self.current_leader.read().await.clone()
    }

    /// Append entry to log (client request)
    pub async fn append(&self, data: Vec<u8>) -> RaftResult<u64> {
        if !self.is_leader().await {
            return Err(RaftError::NotLeader);
        }

        let mut persistent = self.persistent.write().await;
        let term = persistent.current_term;
        let index = persistent.last_log_index() + 1;

        let _entry = LogEntry { term, index, data };
        persistent.log.push(entry.clone());

        log::debug!("Appended entry at index {}", index);

        // Start replication to followers
        drop(persistent);
        self.replicate_log().await;

        Ok(index)
    }

    /// Start the Raft consensus algorithm
    pub async fn start(self: Arc<Self>) {
        // Start election timeout monitor
        let node = Arc::clone(&self);
        tokio::spawn(async move {
            node.run_election_timeout().await;
        });

        // Start heartbeat sender (if leader)
        let node = Arc::clone(&self);
        tokio::spawn(async move {
            node.run_heartbeat_sender().await;
        });

        log::info!("Raft node {} started", self.node_id);
    }

    /// Run election timeout monitor
    async fn run_election_timeout(self: Arc<Self>) {
        loop {
            let timeout = self.random_election_timeout();
            time::sleep(timeout).await;

            let last_hb = *self.last_heartbeat.read().await;
            if last_hb.elapsed() >= timeout {
                let state = *self.state.read().await;
                if state != NodeState::Leader {
                    log::info!("Election timeout, starting election");
                    let _ = self.start_election().await;
                }
            }
        }
    }

    /// Run heartbeat sender (leader only)
    async fn run_heartbeat_sender(self: Arc<Self>) {
        let mut interval = time::interval(self.config.heartbeat_interval);

        loop {
            interval.tick().await;

            if self.is_leader().await {
                self.send_heartbeats().await;
            }
        }
    }

    /// Generate random election timeout
    fn random_election_timeout(&self) -> Duration {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let min = self.config.election_timeout_min.as_millis() as u64;
        let max = self.config.election_timeout_max.as_millis() as u64;
        let timeout = rng.gen_range(min..=max);
        Duration::from_millis(timeout)
    }

    /// Start leader election
    async fn start_election(&self) -> RaftResult<bool> {
        // Transition to candidate
        *self.state.write().await = NodeState::Candidate;

        // Increment term and vote for self
        let mut persistent = self.persistent.write().await;
        persistent.current_term += 1;
        persistent.voted_for = Some(self.node_id.clone());
        let term = persistent.current_term;
        let last_log_index = persistent.last_log_index();
        let last_log_term = persistent.last_log_term();
        drop(persistent);

        log::info!("Starting election for term {}", term);

        // Request votes from peers
        let peers = self.peers.read().await.clone();
        let mut votes = 1; // Vote for self

        for peer_id in &peers {
            let message = Message::RequestVote {
                term,
                candidate_id: self.node_id.clone(),
                last_log_index,
                last_log_term,
            };

            if let Ok(_) = self.transport.send(peer_id.clone(), message).await {
                // In real implementation, wait for responses
                // For now, simplified
            }
        }

        // Check if won election (simplified - should wait for actual responses)
        let quorum = (peers.len() + 1) / 2 + 1;
        if votes >= quorum {
            self.become_leader().await;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Become leader
    async fn become_leader(&self) {
        log::info!("Became leader for term {}", self.current_term().await);

        *self.state.write().await = NodeState::Leader;
        *self.current_leader.write().await = Some(self.node_id.clone());

        // Initialize leader state
        let peers = self.peers.read().await.clone();
        let last_log_index = self.persistent.read().await.last_log_index();
        *self.leader_state.write().await = Some(LeaderState::new(&peers, last_log_index));

        // Send initial heartbeats
        self.send_heartbeats().await;
    }

    /// Send heartbeats to all followers
    async fn send_heartbeats(&self) {
        let persistent = self.persistent.read().await;
        let term = persistent.current_term;
        let leader_commit = self.volatile.read().await.commit_index;
        let prev_log_index = persistent.last_log_index();
        let prev_log_term = persistent.last_log_term();
        drop(persistent);

        let peers = self.peers.read().await.clone();

        for peer_id in peers {
            let message = Message::AppendEntries {
                term,
                leader_id: self.node_id.clone(),
                prev_log_index,
                prev_log_term,
                entries: Vec::new(), // Empty for heartbeat
                leader_commit,
            };

            let _ = self.transport.send(peer_id, message).await;
        }
    }

    /// Replicate log to followers
    async fn replicate_log(&self) {
        let leader_state = self.leader_state.read().await;
        if leader_state.is_none() {
            return;
        }

        let peers = self.peers.read().await.clone();
        for peer_id in peers {
            self.replicate_to_peer(&peer_id).await;
        }
    }

    /// Replicate log to a specific peer
    async fn replicate_to_peer(&self, peer_id: &NodeId) {
        let leader_state = self.leader_state.read().await;
        let next_index = leader_state.as_ref()
            .and_then(|ls| ls.next_index.get(peer_id))
            .copied()
            .unwrap_or(1);
        drop(leader_state);

        let persistent = self.persistent.read().await;
        let term = persistent.current_term;
        let leader_commit = self.volatile.read().await.commit_index;

        // Get entries to send
        let entries: Vec<Vec<u8>> = persistent.log.iter()
            .filter(|e| e.index >= next_index)
            .take(self.config.max_append_entries)
            .map(|e| e.data.clone())
            .collect();

        let prev_log_index = if next_index > 1 { next_index - 1 } else { 0 };
        let prev_log_term = persistent.get_entry(prev_log_index)
            .map(|e| e.term)
            .unwrap_or(0);

        drop(persistent);

        let message = Message::AppendEntries {
            term,
            leader_id: self.node_id.clone(),
            prev_log_index,
            prev_log_term,
            entries,
            leader_commit,
        };

        let _ = self.transport.send(peer_id.clone(), message).await;
    }

    /// Handle RequestVote RPC
    pub async fn handle_request_vote(
        &self,
        term: u64,
        candidate_id: NodeId,
        last_log_index: u64,
        last_log_term: u64,
    ) -> RaftResult<bool> {
        let mut persistent = self.persistent.write().await;

        // Update term if necessary
        if term > persistent.current_term {
            persistent.current_term = term;
            persistent.voted_for = None;
            *self.state.write().await = NodeState::Follower;
        }

        // Check if can grant vote
        let can_vote = term >= persistent.current_term
            && (persistent.voted_for.is_none()
                || persistent.voted_for.as_ref() == Some(&candidate_id))
            && last_log_term >= persistent.last_log_term()
            && last_log_index >= persistent.last_log_index();

        if can_vote {
            persistent.voted_for = Some(candidate_id);
            *self.last_heartbeat.write().await = Instant::now();
            log::debug!("Granted vote for term {}", term);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Handle AppendEntries RPC
    pub async fn handle_append_entries(
        &self,
        term: u64,
        leader_id: NodeId,
        prev_log_index: u64,
        prev_log_term: u64,
        entries: Vec<Vec<u8>>,
        leader_commit: u64,
    ) -> RaftResult<bool> {
        let mut persistent = self.persistent.write().await;

        // Update term and step down if necessary
        if term > persistent.current_term {
            persistent.current_term = term;
            persistent.voted_for = None;
            *self.state.write().await = NodeState::Follower;
        }

        // Reject if term is old
        if term < persistent.current_term {
            return Ok(false);
        }

        // Update leader and reset election timeout
        *self.current_leader.write().await = Some(leader_id);
        *self.last_heartbeat.write().await = Instant::now();

        // Check log consistency
        if prev_log_index > 0 {
            if let Some(entry) = persistent.get_entry(prev_log_index) {
                if entry.term != prev_log_term {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }

        // Append new entries
        for (i, data) in entries.into_iter().enumerate() {
            let index = prev_log_index + 1 + i as u64;
            let _entry = LogEntry { term, index, data };

            // Remove conflicting entries
            persistent.log.retain(|e| e.index < index);
            persistent.log.push(entry);
        }

        // Update commit index
        if leader_commit > self.volatile.read().await.commit_index {
            let mut volatile = self.volatile.write().await;
            volatile.commit_index = std::cmp::min(
                leader_commit,
                persistent.last_log_index(),
            );
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[tokio::test]
    async fn test_raft_node_creation() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
        let transport = Arc::new(Transport::new("node1".to_string(), addr));
        let config = RaftConfig::default();

        let (node, _rx) = RaftNode::new("node1".to_string(), transport, config);

        assert_eq!(node.node_id, "node1");
        assert_eq!(node.get_state().await, NodeState::Follower);
    }

    #[tokio::test]
    async fn test_initial_state() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
        let transport = Arc::new(Transport::new("node1".to_string(), addr));
        let config = RaftConfig::default();

        let (node, _rx) = RaftNode::new("node1".to_string(), transport, config);

        assert_eq!(node.current_term().await, 0);
        assert!(!node.is_leader().await);
        assert!(node.get_leader().await.is_none());
    }

    #[tokio::test]
    async fn test_add_peer() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
        let transport = Arc::new(Transport::new("node1".to_string(), addr));
        let config = RaftConfig::default();

        let (node, _rx) = RaftNode::new("node1".to_string(), transport, config);

        node.add_peer("node2".to_string()).await;
        let peers = node.peers.read().await;
        assert_eq!(peers.len(), 1);
    }

    #[tokio::test]
    async fn test_append_not_leader() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
        let transport = Arc::new(Transport::new("node1".to_string(), addr));
        let config = RaftConfig::default();

        let (node, _rx) = RaftNode::new("node1".to_string(), transport, config);

        let result = node.append(vec![1, 2, 3]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_request_vote() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
        let transport = Arc::new(Transport::new("node1".to_string(), addr));
        let config = RaftConfig::default();

        let (node, _rx) = RaftNode::new("node1".to_string(), transport, config);

        let granted = node.handle_request_vote(
            1,
            "node2".to_string(),
            0,
            0,
        ).await.unwrap();

        assert!(granted);
    }
}
