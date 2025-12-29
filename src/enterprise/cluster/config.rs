//! Cluster Configuration
//!
//! Manages cluster topology, node configuration, dynamic reconfiguration,
//! and rolling updates for the HA clustering system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;
use thiserror::Error;

/// Configuration errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Invalid configuration: {0}")]
    Invalid(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Node not found: {0}")]
    NodeNotFound(String),
}

pub type ConfigResult<T> = Result<T, ConfigError>;

/// Node identifier
pub type NodeId = String;

/// Node role in the cluster
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeRole {
    /// Full voting member
    Voter,
    /// Non-voting replica
    Learner,
    /// Witness node for quorum (doesn't store data)
    Witness,
}

/// Node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Unique node identifier
    pub id: NodeId,
    /// Node address for cluster communication
    pub addr: SocketAddr,
    /// Node role
    pub role: NodeRole,
    /// Node priority for leader election (higher = more likely)
    pub priority: u8,
    /// Node weight for load balancing
    pub weight: u32,
    /// Maximum connections this node can handle
    pub max_connections: usize,
    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

impl NodeConfig {
    /// Create a new node configuration
    pub fn new(id: NodeId, addr: SocketAddr, role: NodeRole) -> Self {
        Self {
            id,
            addr,
            role,
            priority: 100,
            weight: 100,
            max_connections: 10000,
            metadata: HashMap::new(),
        }
    }

    /// Set priority for leader election
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Set weight for load balancing
    pub fn with_weight(mut self, weight: u32) -> Self {
        self.weight = weight;
        self
    }

    /// Set maximum connections
    pub fn with_max_connections(mut self, max_connections: usize) -> Self {
        self.max_connections = max_connections;
        self
    }
}

/// Cluster topology configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterTopology {
    /// All nodes in the cluster
    pub nodes: HashMap<NodeId, NodeConfig>,
    /// Cluster name/identifier
    pub cluster_id: String,
    /// Expected cluster size (for quorum calculation)
    pub expected_size: usize,
}

impl ClusterTopology {
    /// Create a new cluster topology
    pub fn new(cluster_id: String) -> Self {
        Self {
            nodes: HashMap::new(),
            cluster_id,
            expected_size: 3,
        }
    }

    /// Add a node to the topology
    pub fn add_node(&mut self, node: NodeConfig) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// Remove a node from the topology
    pub fn remove_node(&mut self, node_id: &str) -> ConfigResult<NodeConfig> {
        self.nodes
            .remove(node_id)
            .ok_or_else(|| ConfigError::NodeNotFound(node_id.to_string()))
    }

    /// Get a node by ID
    pub fn get_node(&self, node_id: &str) -> Option<&NodeConfig> {
        self.nodes.get(node_id)
    }

    /// Get all voter nodes
    pub fn voters(&self) -> Vec<&NodeConfig> {
        self.nodes
            .values()
            .filter(|n| n.role == NodeRole::Voter)
            .collect()
    }

    /// Get all learner nodes
    pub fn learners(&self) -> Vec<&NodeConfig> {
        self.nodes
            .values()
            .filter(|n| n.role == NodeRole::Learner)
            .collect()
    }

    /// Get all witness nodes
    pub fn witnesses(&self) -> Vec<&NodeConfig> {
        self.nodes
            .values()
            .filter(|n| n.role == NodeRole::Witness)
            .collect()
    }

    /// Calculate quorum size
    pub fn quorum_size(&self) -> usize {
        let voter_count = self.voters().len();
        voter_count / 2 + 1
    }
}

/// Cluster configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    /// Cluster topology
    pub topology: ClusterTopology,

    /// Raft configuration
    pub raft: RaftConfig,

    /// Network configuration
    pub network: NetworkConfig,

    /// Replication configuration
    pub replication: ReplicationConfig,

    /// Failover configuration
    pub failover: FailoverConfig,
}

impl ClusterConfig {
    /// Create a new cluster configuration
    pub fn new(cluster_id: String) -> Self {
        Self {
            topology: ClusterTopology::new(cluster_id),
            raft: RaftConfig::default(),
            network: NetworkConfig::default(),
            replication: ReplicationConfig::default(),
            failover: FailoverConfig::default(),
        }
    }

    /// Load configuration from file
    pub fn from_file(path: &str) -> ConfigResult<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config = serde_json::from_str(&contents)?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self, path: &str) -> ConfigResult<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> ConfigResult<()> {
        // Check minimum cluster size
        if self.topology.voters().is_empty() {
            return Err(ConfigError::Invalid("No voter nodes configured".to_string()));
        }

        // Check odd number of voters for better quorum
        let voter_count = self.topology.voters().len();
        if voter_count % 2 == 0 {
            log::warn!("Even number of voter nodes ({}) may lead to split-brain", voter_count);
        }

        // Validate timeouts
        if self.raft.election_timeout_min >= self.raft.election_timeout_max {
            return Err(ConfigError::Invalid(
                "election_timeout_min must be < election_timeout_max".to_string(),
            ));
        }

        Ok(())
    }
}

/// Raft protocol configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaftConfig {
    /// Heartbeat interval
    pub heartbeat_interval: Duration,

    /// Minimum election timeout
    pub election_timeout_min: Duration,

    /// Maximum election timeout
    pub election_timeout_max: Duration,

    /// Maximum entries per append
    pub max_append_entries: usize,

    /// Snapshot threshold (log entries before snapshot)
    pub snapshot_threshold: usize,

    /// Snapshot interval
    pub snapshot_interval: Duration,

    /// Enable pipeline optimization
    pub enable_pipelining: bool,

    /// Maximum inflight requests
    pub max_inflight: usize,
}

impl Default for RaftConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval: Duration::from_millis(50),
            election_timeout_min: Duration::from_millis(150),
            election_timeout_max: Duration::from_millis(300),
            max_append_entries: 64,
            snapshot_threshold: 10000,
            snapshot_interval: Duration::from_secs(300),
            enable_pipelining: true,
            max_inflight: 256,
        }
    }
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Connection timeout
    pub connect_timeout: Duration,

    /// Request timeout
    pub request_timeout: Duration,

    /// Keep-alive interval
    pub keepalive_interval: Duration,

    /// Maximum message size
    pub max_message_size: usize,

    /// Connection pool size per node
    pub connection_pool_size: usize,

    /// Enable TCP_NODELAY
    pub tcp_nodelay: bool,

    /// Send buffer size
    pub send_buffer_size: usize,

    /// Receive buffer size
    pub recv_buffer_size: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(30),
            keepalive_interval: Duration::from_secs(10),
            max_message_size: 16 * 1024 * 1024, // 16 MB
            connection_pool_size: 4,
            tcp_nodelay: true,
            send_buffer_size: 256 * 1024,
            recv_buffer_size: 256 * 1024,
        }
    }
}

/// Replication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationConfig {
    /// Maximum batch size for replication
    pub max_batch_size: usize,

    /// Batch timeout
    pub batch_timeout: Duration,

    /// Enable compression
    pub enable_compression: bool,

    /// Compression threshold (bytes)
    pub compression_threshold: usize,

    /// Maximum concurrent snapshots
    pub max_concurrent_snapshots: usize,

    /// Snapshot chunk size
    pub snapshot_chunk_size: usize,
}

impl Default for ReplicationConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 1000,
            batch_timeout: Duration::from_millis(10),
            enable_compression: true,
            compression_threshold: 4096,
            max_concurrent_snapshots: 2,
            snapshot_chunk_size: 1024 * 1024, // 1 MB
        }
    }
}

/// Failover configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverConfig {
    /// Enable automatic failover
    pub enable_auto_failover: bool,

    /// Failure detection threshold
    pub failure_threshold: usize,

    /// Health check interval
    pub health_check_interval: Duration,

    /// Grace period before failover
    pub failover_grace_period: Duration,

    /// Connection drain timeout
    pub drain_timeout: Duration,

    /// Enable session migration
    pub enable_session_migration: bool,
}

impl Default for FailoverConfig {
    fn default() -> Self {
        Self {
            enable_auto_failover: true,
            failure_threshold: 3,
            health_check_interval: Duration::from_secs(1),
            failover_grace_period: Duration::from_secs(5),
            drain_timeout: Duration::from_secs(30),
            enable_session_migration: true,
        }
    }
}

/// Dynamic configuration manager
pub struct ConfigManager {
    config: ClusterConfig,
    version: u64,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new(config: ClusterConfig) -> ConfigResult<Self> {
        config.validate()?;
        Ok(Self { config, version: 0 })
    }

    /// Get current configuration
    pub fn get_config(&self) -> &ClusterConfig {
        &self.config
    }

    /// Get configuration version
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Update configuration (atomic)
    pub fn update_config(&mut self, new_config: ClusterConfig) -> ConfigResult<()> {
        new_config.validate()?;
        self.config = new_config;
        self.version += 1;
        log::info!("Configuration updated to version {}", self.version);
        Ok(())
    }

    /// Add node to cluster
    pub fn add_node(&mut self, node: NodeConfig) -> ConfigResult<()> {
        self.config.topology.add_node(node);
        self.version += 1;
        Ok(())
    }

    /// Remove node from cluster
    pub fn remove_node(&mut self, node_id: &str) -> ConfigResult<NodeConfig> {
        let node = self.config.topology.remove_node(node_id)?;
        self.version += 1;
        Ok(node)
    }

    /// Check if rolling update is safe
    pub fn can_update_node(&self, node_id: &str) -> bool {
        let voters = self.config.topology.voters();
        let quorum = self.config.topology.quorum_size();

        // Can update if removing this node still maintains quorum
        let remaining = voters.iter().filter(|n| n.id != node_id).count();
        remaining >= quorum
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_node_config_creation() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let node = NodeConfig::new("node1".to_string(), addr, NodeRole::Voter)
            .with_priority(150)
            .with_weight(200);

        assert_eq!(node.id, "node1");
        assert_eq!(node.priority, 150);
        assert_eq!(node.weight, 200);
    }

    #[test]
    fn test_cluster_topology() {
        let mut topology = ClusterTopology::new("test-cluster".to_string());

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let node1 = NodeConfig::new("node1".to_string(), addr, NodeRole::Voter);
        topology.add_node(node1);

        assert_eq!(topology.nodes.len(), 1);
        assert_eq!(topology.voters().len(), 1);
        assert_eq!(topology.quorum_size(), 1);
    }

    #[test]
    fn test_quorum_calculation() {
        let mut topology = ClusterTopology::new("test".to_string());

        // Add 5 voters
        for i in 0..5 {
            let addr = SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                8080 + i,
            );
            let node = NodeConfig::new(format!("node{}", i), addr, NodeRole::Voter);
            topology.add_node(node);
        }

        assert_eq!(topology.voters().len(), 5);
        assert_eq!(topology.quorum_size(), 3); // 5/2 + 1 = 3
    }

    #[test]
    fn test_config_validation() {
        let config = ClusterConfig::new("test".to_string());

        // Should fail with no voters
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_manager() {
        let mut config = ClusterConfig::new("test".to_string());
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let node = NodeConfig::new("node1".to_string(), addr, NodeRole::Voter);
        config.topology.add_node(node);

        let mut manager = ConfigManager::new(config).unwrap();
        assert_eq!(manager.version(), 0);

        let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081);
        let node2 = NodeConfig::new("node2".to_string(), addr2, NodeRole::Voter);
        manager.add_node(node2).unwrap();

        assert_eq!(manager.version(), 1);
        assert_eq!(manager.get_config().topology.nodes.len(), 2);
    }

    #[test]
    fn test_rolling_update_safety() {
        let mut config = ClusterConfig::new("test".to_string());

        // Add 3 voters
        for i in 0..3 {
            let addr = SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                8080 + i,
            );
            let node = NodeConfig::new(format!("node{}", i), addr, NodeRole::Voter);
            config.topology.add_node(node);
        }

        let manager = ConfigManager::new(config).unwrap();

        // With 3 nodes (quorum=2), can update 1 node safely
        assert!(manager.can_update_node("node0"));

        // But not safe to update if it would break quorum
        // (This is a simplified check - real implementation needs more logic)
    }
}
