//! Split Brain Prevention
//!
//! Implements quorum calculation, partition detection, fencing mechanisms,
//! and witness nodes to prevent split-brain scenarios.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;

use super::config::NodeId;

/// Quorum errors
#[derive(Error, Debug)]
pub enum QuorumError {
    #[error("Quorum not reached: {0}/{1} votes")]
    QuorumNotReached(usize, usize),
    #[error("Network partition detected")]
    NetworkPartition,
    #[error("Node is fenced")]
    NodeFenced,
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

pub type QuorumResult<T> = Result<T, QuorumError>;

/// Vote types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoteType {
    /// Vote to approve
    Approve,
    /// Vote to reject
    Reject,
    /// Abstain from voting
    Abstain,
}

/// Vote record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter_id: NodeId,
    pub vote_type: VoteType,
    pub term: u64,
    pub timestamp: u64,
}

/// Quorum configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuorumConfig {
    /// Total number of voting nodes
    pub total_voters: usize,

    /// Required quorum size (typically majority)
    pub quorum_size: usize,

    /// Use witness nodes
    pub use_witnesses: bool,

    /// Number of witness nodes
    pub witness_count: usize,

    /// Partition detection timeout
    pub partition_timeout: Duration,
}

impl QuorumConfig {
    /// Create a new quorum configuration
    pub fn new(total_voters: usize) -> Self {
        let quorum_size = total_voters / 2 + 1;

        Self {
            total_voters,
            quorum_size,
            use_witnesses: false,
            witness_count: 0,
            partition_timeout: Duration::from_secs(30),
        }
    }

    /// Create configuration with witnesses
    pub fn with_witnesses(total_voters: usize, witness_count: usize) -> Self {
        let mut config = Self::new(total_voters);
        config.use_witnesses = true;
        config.witness_count = witness_count;
        config
    }

    /// Validate configuration
    pub fn validate(&self) -> QuorumResult<()> {
        if self.total_voters == 0 {
            return Err(QuorumError::InvalidConfig(
                "No voting nodes configured".to_string(),
            ));
        }

        if self.quorum_size > self.total_voters {
            return Err(QuorumError::InvalidConfig(
                "Quorum size exceeds total voters".to_string(),
            ));
        }

        if self.total_voters % 2 == 0 {
            log::warn!("Even number of voters may lead to split-brain");
        }

        Ok(())
    }
}

/// Node partition information
#[derive(Debug, Clone)]
pub struct Partition {
    pub id: String,
    pub nodes: HashSet<NodeId>,
    pub detected_at: Instant,
    pub can_form_quorum: bool,
}

impl Partition {
    fn new(id: String, nodes: HashSet<NodeId>) -> Self {
        Self {
            id,
            nodes,
            detected_at: Instant::now(),
            can_form_quorum: false,
        }
    }
}

/// Fencing token for preventing split-brain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FencingToken {
    pub token: u64,
    pub issued_to: NodeId,
    pub term: u64,
    pub issued_at: u64,
    pub expires_at: u64,
}

impl FencingToken {
    /// Create a new fencing token
    pub fn new(token: u64, node_id: NodeId, term: u64, ttl: Duration) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            token,
            issued_to: node_id,
            term,
            issued_at: now,
            expires_at: now + ttl.as_secs(),
        }
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        now >= self.expires_at
    }

    /// Check if token is valid for a specific node
    pub fn is_valid_for(&self, node_id: &NodeId) -> bool {
        !self.is_expired() && self.issued_to == *node_id
    }
}

/// Quorum manager
pub struct QuorumManager {
    /// Local node ID
    node_id: NodeId,

    /// Quorum configuration
    config: QuorumConfig,

    /// All voting nodes
    voters: Arc<RwLock<HashSet<NodeId>>>,

    /// Witness nodes
    witnesses: Arc<RwLock<HashSet<NodeId>>>,

    /// Current votes for a decision
    votes: Arc<RwLock<HashMap<String, Vec<Vote>>>>,

    /// Detected partitions
    partitions: Arc<RwLock<Vec<Partition>>>,

    /// Fenced nodes
    fenced_nodes: Arc<RwLock<HashSet<NodeId>>>,

    /// Current fencing token
    fencing_token: Arc<RwLock<Option<FencingToken>>>,

    /// Node connectivity status
    connectivity: Arc<RwLock<HashMap<NodeId, Instant>>>,
}

impl QuorumManager {
    /// Create a new quorum manager
    pub fn new(node_id: NodeId, config: QuorumConfig) -> QuorumResult<Self> {
        config.validate()?;

        Ok(Self {
            node_id,
            config,
            voters: Arc::new(RwLock::new(HashSet::new())),
            witnesses: Arc::new(RwLock::new(HashSet::new())),
            votes: Arc::new(RwLock::new(HashMap::new())),
            partitions: Arc::new(RwLock::new(Vec::new())),
            fenced_nodes: Arc::new(RwLock::new(HashSet::new())),
            fencing_token: Arc::new(RwLock::new(None)),
            connectivity: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Add a voting node
    pub async fn add_voter(&self, node_id: NodeId) {
        let mut voters = self.voters.write().await;
        voters.insert(node_id.clone());

        // Update connectivity
        let mut connectivity = self.connectivity.write().await;
        connectivity.insert(node_id, Instant::now());
    }

    /// Remove a voting node
    pub async fn remove_voter(&self, node_id: &NodeId) {
        let mut voters = self.voters.write().await;
        voters.remove(node_id);

        // Remove connectivity
        let mut connectivity = self.connectivity.write().await;
        connectivity.remove(node_id);
    }

    /// Add a witness node
    pub async fn add_witness(&self, node_id: NodeId) {
        let mut witnesses = self.witnesses.write().await;
        witnesses.insert(node_id);
    }

    /// Remove a witness node
    pub async fn remove_witness(&self, node_id: &NodeId) {
        let mut witnesses = self.witnesses.write().await;
        witnesses.remove(node_id);
    }

    /// Record a vote
    pub async fn record_vote(&self, decision_id: String, vote: Vote) {
        let mut votes = self.votes.write().await;
        votes.entry(decision_id).or_insert_with(Vec::new).push(vote);
    }

    /// Check if quorum is reached for a decision
    pub async fn has_quorum(&self, decision_id: &str) -> QuorumResult<bool> {
        let votes = self.votes.read().await;
        let decision_votes = votes.get(decision_id).cloned().unwrap_or_default();
        drop(votes);

        let approve_count = decision_votes
            .iter()
            .filter(|v| v.vote_type == VoteType::Approve)
            .count();

        Ok(approve_count >= self.config.quorum_size)
    }

    /// Wait for quorum on a decision
    pub async fn wait_for_quorum(
        &self,
        decision_id: &str,
        timeout: Duration,
    ) -> QuorumResult<()> {
        let start = Instant::now();

        loop {
            if self.has_quorum(decision_id).await? {
                return Ok(());
            }

            if start.elapsed() > timeout {
                let votes = self.votes.read().await;
                let vote_count = votes
                    .get(decision_id)
                    .map(|v| v.len())
                    .unwrap_or(0);

                return Err(QuorumError::QuorumNotReached(
                    vote_count,
                    self.config.quorum_size,
                ));
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Clear votes for a decision
    pub async fn clear_votes(&self, decision_id: &str) {
        let mut votes = self.votes.write().await;
        votes.remove(decision_id);
    }

    /// Update node connectivity
    pub async fn update_connectivity(&self, node_id: NodeId) {
        let mut connectivity = self.connectivity.write().await;
        connectivity.insert(node_id, Instant::now());
    }

    /// Detect network partitions
    pub async fn detect_partitions(&self) -> Vec<Partition> {
        let connectivity = self.connectivity.read().await;
        let now = Instant::now();

        // Find nodes that haven't been seen recently
        let mut unreachable = HashSet::new();
        let mut reachable = HashSet::new();

        for (node_id, last_seen) in connectivity.iter() {
            if now.duration_since(*last_seen) > self.config.partition_timeout {
                unreachable.insert(node_id.clone());
            } else {
                reachable.insert(node_id.clone());
            }
        }

        let mut partitions = Vec::new();

        // Create partition for reachable nodes (including self)
        if !reachable.is_empty() {
            let mut nodes = reachable.clone();
            nodes.insert(self.node_id.clone());

            let mut partition = Partition::new("reachable".to_string(), nodes.clone());
            partition.can_form_quorum = nodes.len() >= self.config.quorum_size;
            partitions.push(partition);
        }

        // Create partition for unreachable nodes
        if !unreachable.is_empty() {
            let partition = Partition::new("unreachable".to_string(), unreachable);
            partitions.push(partition);
        }

        // Store detected partitions
        *self.partitions.write().await = partitions.clone();

        partitions
    }

    /// Check if in a viable partition (can form quorum)
    pub async fn in_viable_partition(&self) -> bool {
        let partitions = self.detect_partitions().await;

        // Check if our partition can form quorum
        partitions.iter().any(|p| {
            p.nodes.contains(&self.node_id) && p.can_form_quorum
        })
    }

    /// Issue a fencing token
    pub async fn issue_fencing_token(&self, term: u64) -> FencingToken {
        let token = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        let fencing_token = FencingToken::new(
            token,
            self.node_id.clone(),
            term,
            Duration::from_secs(300), // 5 minutes TTL
        );

        *self.fencing_token.write().await = Some(fencing_token.clone());

        log::info!("Issued fencing token: {}", token);
        fencing_token
    }

    /// Validate a fencing token
    pub async fn validate_fencing_token(
        &self,
        token: &FencingToken,
    ) -> QuorumResult<()> {
        let current_token = self.fencing_token.read().await;

        if let Some(current) = current_token.as_ref() {
            if token.token <= current.token {
                return Err(QuorumError::NodeFenced);
            }
        }

        if token.is_expired() {
            return Err(QuorumError::NodeFenced);
        }

        Ok(())
    }

    /// Fence a node
    pub async fn fence_node(&self, node_id: NodeId) {
        let mut fenced = self.fenced_nodes.write().await;
        fenced.insert(node_id.clone());
        log::warn!("Fenced node: {}", node_id);
    }

    /// Unfence a node
    pub async fn unfence_node(&self, node_id: &NodeId) {
        let mut fenced = self.fenced_nodes.write().await;
        fenced.remove(node_id);
        log::info!("Unfenced node: {}", node_id);
    }

    /// Check if a node is fenced
    pub async fn is_fenced(&self, node_id: &NodeId) -> bool {
        let fenced = self.fenced_nodes.read().await;
        fenced.contains(node_id)
    }

    /// Get quorum size
    pub fn quorum_size(&self) -> usize {
        self.config.quorum_size
    }

    /// Get total voters
    pub async fn total_voters(&self) -> usize {
        self.voters.read().await.len()
    }

    /// Get statistics
    pub async fn stats(&self) -> QuorumStats {
        let voters = self.voters.read().await.len();
        let witnesses = self.witnesses.read().await.len();
        let fenced = self.fenced_nodes.read().await.len();
        let partitions = self.partitions.read().await.len();

        QuorumStats {
            total_voters: voters,
            witness_count: witnesses,
            quorum_size: self.config.quorum_size,
            fenced_nodes: fenced,
            detected_partitions: partitions,
        }
    }
}

/// Quorum statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuorumStats {
    pub total_voters: usize,
    pub witness_count: usize,
    pub quorum_size: usize,
    pub fenced_nodes: usize,
    pub detected_partitions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quorum_config() {
        let config = QuorumConfig::new(5);
        assert_eq!(config.total_voters, 5);
        assert_eq!(config.quorum_size, 3); // 5/2 + 1 = 3
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_quorum_config_with_witnesses() {
        let config = QuorumConfig::with_witnesses(5, 2);
        assert_eq!(config.witness_count, 2);
        assert!(config.use_witnesses);
    }

    #[tokio::test]
    async fn test_quorum_manager_creation() {
        let config = QuorumConfig::new(3);
        let manager = QuorumManager::new("node1".to_string(), config).unwrap();

        assert_eq!(manager.node_id, "node1");
    }

    #[tokio::test]
    async fn test_add_voter() {
        let config = QuorumConfig::new(3);
        let manager = QuorumManager::new("node1".to_string(), config).unwrap();

        manager.add_voter("node2".to_string()).await;
        manager.add_voter("node3".to_string()).await;

        assert_eq!(manager.total_voters().await, 2);
    }

    #[tokio::test]
    async fn test_record_vote() {
        let config = QuorumConfig::new(3);
        let manager = QuorumManager::new("node1".to_string(), config).unwrap();

        let vote = Vote {
            voter_id: "node1".to_string(),
            vote_type: VoteType::Approve,
            term: 1,
            timestamp: 0,
        };

        manager.record_vote("decision1".to_string(), vote).await;

        let votes = manager.votes.read().await;
        assert_eq!(votes.get("decision1").unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_has_quorum() {
        let config = QuorumConfig::new(3);
        let manager = QuorumManager::new("node1".to_string(), config).unwrap();

        // Add 3 approve votes (quorum is 2)
        for i in 1..=3 {
            let vote = Vote {
                voter_id: format!("node{}", i),
                vote_type: VoteType::Approve,
                term: 1,
                timestamp: 0,
            };
            manager.record_vote("decision1".to_string(), vote).await;
        }

        assert!(manager.has_quorum("decision1").await.unwrap());
    }

    #[tokio::test]
    async fn test_fencing_token() {
        let token = FencingToken::new(
            12345,
            "node1".to_string(),
            1,
            Duration::from_secs(60),
        );

        assert!(!token.is_expired());
        assert!(token.is_valid_for(&"node1".to_string()));
        assert!(!token.is_valid_for(&"node2".to_string()));
    }

    #[tokio::test]
    async fn test_fence_node() {
        let config = QuorumConfig::new(3);
        let manager = QuorumManager::new("node1".to_string(), config).unwrap();

        manager.fence_node("node2".to_string()).await;
        assert!(manager.is_fenced(&"node2".to_string()).await);

        manager.unfence_node(&"node2".to_string()).await;
        assert!(!manager.is_fenced(&"node2".to_string()).await);
    }

    #[tokio::test]
    async fn test_partition_detection() {
        let config = QuorumConfig::new(3);
        let manager = QuorumManager::new("node1".to_string(), config).unwrap();

        manager.add_voter("node2".to_string()).await;
        manager.add_voter("node3".to_string()).await;

        // Simulate node2 being unreachable
        let mut connectivity = manager.connectivity.write().await;
        connectivity.insert(
            "node2".to_string(),
            Instant::now() - Duration::from_secs(60),
        );
        drop(connectivity);

        let partitions = manager.detect_partitions().await;
        assert!(!partitions.is_empty());
    }
}
