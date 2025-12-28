//! # High-Availability Clustering Module
//!
//! Provides a complete distributed clustering system for CADDY with Raft consensus,
//! automatic failover, load balancing, and split-brain prevention.
//!
//! ## Overview
//!
//! This module implements a production-ready HA clustering solution that includes:
//!
//! - **Raft Consensus**: Leader election, log replication, and term management
//! - **Cluster Membership**: SWIM-like gossip protocol for membership management
//! - **State Replication**: Replicated state machine with snapshot support
//! - **Automatic Failover**: Leader failover with session migration
//! - **Load Balancing**: Multiple strategies including round-robin and least-connections
//! - **Split-Brain Prevention**: Quorum-based decision making with fencing
//! - **Network Transport**: Reliable TCP-based cluster communication
//!
//! ## Architecture
//!
//! The clustering system is built on several key components:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     Cluster Manager                         │
//! ├─────────────────────────────────────────────────────────────┤
//! │                                                             │
//! │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
//! │  │   Raft   │  │Membership│  │Failover  │  │  Quorum  │  │
//! │  │Consensus │  │ Manager  │  │ Manager  │  │ Manager  │  │
//! │  └──────────┘  └──────────┘  └──────────┘  └──────────┘  │
//! │                                                             │
//! │  ┌──────────┐  ┌──────────┐  ┌──────────┐                │
//! │  │   State  │  │   Load   │  │Transport │                │
//! │  │Replication│  │Balancer  │  │  Layer   │                │
//! │  └──────────┘  └──────────┘  └──────────┘                │
//! │                                                             │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use caddy::enterprise::cluster::{ClusterManager, ClusterConfig, NodeConfig, NodeRole};
//! use std::net::SocketAddr;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create cluster configuration
//!     let mut config = ClusterConfig::new("my-cluster".to_string());
//!
//!     // Add nodes to the cluster
//!     let node1 = NodeConfig::new(
//!         "node1".to_string(),
//!         "127.0.0.1:8080".parse()?,
//!         NodeRole::Voter,
//!     );
//!     config.topology.add_node(node1);
//!
//!     // Create and start cluster manager
//!     let manager = ClusterManager::new("node1".to_string(), config).await?;
//!     manager.start().await?;
//!
//!     // Manager is now running and handling cluster operations
//!     Ok(())
//! }
//! ```
//!
//! ## Features
//!
//! ### Raft Consensus
//!
//! The Raft module implements the Raft consensus algorithm for distributed agreement:
//!
//! - Leader election with randomized timeouts
//! - Log replication with consistency checking
//! - Term management and conflict resolution
//! - Snapshot support for log compaction
//!
//! ### Membership Management
//!
//! SWIM-like gossip protocol for efficient membership:
//!
//! - Failure detection with direct and indirect probing
//! - Gossip-based information dissemination
//! - Automatic node discovery
//! - Graceful join/leave handling
//!
//! ### Automatic Failover
//!
//! Intelligent failover system:
//!
//! - Health monitoring with configurable thresholds
//! - Automatic leader failover
//! - Session migration support
//! - Connection draining with timeout
//!
//! ### Load Balancing
//!
//! Multiple load balancing strategies:
//!
//! - Round-robin
//! - Least connections
//! - Weighted distribution
//! - Random selection
//! - IP hash (sticky sessions)
//!
//! ### Split-Brain Prevention
//!
//! Robust quorum-based protection:
//!
//! - Majority quorum requirements
//! - Partition detection
//! - Fencing mechanisms
//! - Witness node support
//!
//! ## Configuration
//!
//! The cluster can be configured through various configuration structures:
//!
//! ```rust
//! use caddy::enterprise::cluster::{ClusterConfig, RaftConfig, FailoverConfig};
//! use std::time::Duration;
//!
//! let mut config = ClusterConfig::new("my-cluster".to_string());
//!
//! // Configure Raft
//! config.raft.heartbeat_interval = Duration::from_millis(50);
//! config.raft.election_timeout_min = Duration::from_millis(150);
//! config.raft.election_timeout_max = Duration::from_millis(300);
//!
//! // Configure Failover
//! config.failover.enable_auto_failover = true;
//! config.failover.failure_threshold = 3;
//! ```
//!
//! ## Safety & Reliability
//!
//! The clustering system is designed with safety in mind:
//!
//! - **No split-brain**: Quorum-based decisions prevent split-brain scenarios
//! - **Graceful degradation**: System continues operating with partial failures
//! - **Data consistency**: Raft ensures strong consistency guarantees
//! - **Fault tolerance**: Can tolerate (N-1)/2 failures in an N-node cluster
//!
//! ## Performance
//!
//! The system is optimized for performance:
//!
//! - Async/await throughout for high concurrency
//! - Connection pooling for efficient resource usage
//! - Pipelined replication for throughput
//! - Batched log entries for efficiency
//! - Snapshot transfer for fast catch-up
//!
//! ## Best Practices
//!
//! ### Cluster Size
//!
//! - Use odd number of nodes (3, 5, 7) for optimal quorum
//! - 3 nodes: tolerates 1 failure
//! - 5 nodes: tolerates 2 failures
//! - 7 nodes: tolerates 3 failures
//!
//! ### Network Requirements
//!
//! - Low latency between nodes (< 10ms recommended)
//! - Reliable network connections
//! - Sufficient bandwidth for replication
//! - Stable network topology
//!
//! ### Monitoring
//!
//! - Monitor leader elections
//! - Track replication lag
//! - Watch for partition events
//! - Monitor failover events
//!
//! ## Example: Full Cluster Setup
//!
//! ```rust,no_run
//! use caddy::enterprise::cluster::*;
//! use std::net::SocketAddr;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a 3-node cluster
//! let mut config = ClusterConfig::new("production-cluster".to_string());
//!
//! // Add nodes
//! for i in 0..3 {
//!     let addr: SocketAddr = format!("10.0.0.{}:8080", i + 1).parse()?;
//!     let node = NodeConfig::new(
//!         format!("node{}", i),
//!         addr,
//!         NodeRole::Voter,
//!     );
//!     config.topology.add_node(node);
//! }
//!
//! // Configure for production
//! config.raft.heartbeat_interval = std::time::Duration::from_millis(50);
//! config.failover.enable_auto_failover = true;
//! config.replication.enable_compression = true;
//!
//! // Validate configuration
//! config.validate()?;
//!
//! // Create and start cluster
//! let manager = ClusterManager::new("node0".to_string(), config).await?;
//! manager.start().await?;
//!
//! # Ok(())
//! # }
//! ```

use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

// Module declarations
pub mod config;
pub mod transport;
pub mod raft;
pub mod membership;
pub mod replication;
pub mod failover;
pub mod loadbalance;
pub mod quorum;

// Re-exports for convenience
pub use config::{
    ClusterConfig, ClusterTopology, ConfigManager, FailoverConfig, NetworkConfig, NodeConfig,
    NodeId, NodeRole, RaftConfig, ReplicationConfig,
};
pub use transport::{Envelope, HeartbeatManager, Message, Transport, TransportError};
pub use raft::{LogEntry, RaftNode, RaftError, NodeState, SnapshotMetadata};
pub use membership::{Member, MemberStatus, MembershipManager, MembershipError};
pub use replication::{
    Command, ReplicationLog, ReplicationManager, Snapshot, StateMachine, MemoryStateMachine,
    ReplicationError,
};
pub use failover::{FailoverEvent, FailoverManager, FailoverError, HealthStatus, Session};
pub use loadbalance::{Backend, LoadBalancer, LoadBalancerStats, Strategy, LoadBalanceError};
pub use quorum::{
    FencingToken, Partition, QuorumConfig, QuorumManager, QuorumStats, Vote, VoteType,
    QuorumError,
};

/// Cluster errors
#[derive(Error, Debug)]
pub enum ClusterError {
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),
    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),
    #[error("Raft error: {0}")]
    Raft(#[from] RaftError),
    #[error("Membership error: {0}")]
    Membership(#[from] MembershipError),
    #[error("Replication error: {0}")]
    Replication(#[from] ReplicationError),
    #[error("Failover error: {0}")]
    Failover(#[from] FailoverError),
    #[error("Load balance error: {0}")]
    LoadBalance(#[from] LoadBalanceError),
    #[error("Quorum error: {0}")]
    Quorum(#[from] QuorumError),
    #[error("Cluster not initialized")]
    NotInitialized,
    #[error("Already initialized")]
    AlreadyInitialized,
}

pub type ClusterResult<T> = Result<T, ClusterError>;

/// Main cluster manager that coordinates all clustering components
pub struct ClusterManager {
    /// Node identifier
    node_id: NodeId,

    /// Cluster configuration
    config: ClusterConfig,

    /// Transport layer
    transport: Arc<Transport>,

    /// Raft consensus
    raft: Option<Arc<RaftNode>>,

    /// Membership manager
    membership: Option<Arc<MembershipManager>>,

    /// Replication manager
    replication: Option<Arc<ReplicationManager>>,

    /// Failover manager
    failover: Option<Arc<FailoverManager>>,

    /// Load balancer
    load_balancer: Option<Arc<LoadBalancer>>,

    /// Quorum manager
    quorum: Option<Arc<QuorumManager>>,

    /// Initialization state
    initialized: Arc<RwLock<bool>>,
}

impl ClusterManager {
    /// Create a new cluster manager
    pub async fn new(node_id: NodeId, config: ClusterConfig) -> ClusterResult<Self> {
        // Validate configuration
        config.validate()?;

        // Get node configuration
        let node_config = config
            .topology
            .get_node(&node_id)
            .ok_or_else(|| {
                config::ConfigError::NodeNotFound(node_id.clone())
            })?;

        // Create transport
        let transport = Arc::new(Transport::new(node_id.clone(), node_config.addr));

        Ok(Self {
            node_id,
            config,
            transport,
            raft: None,
            membership: None,
            replication: None,
            failover: None,
            load_balancer: None,
            quorum: None,
            initialized: Arc::new(RwLock::new(false)),
        })
    }

    /// Initialize and start all cluster components
    pub async fn start(&mut self) -> ClusterResult<()> {
        let mut initialized = self.initialized.write().await;
        if *initialized {
            return Err(ClusterError::AlreadyInitialized);
        }

        log::info!("Starting cluster manager for node: {}", self.node_id);

        // Start transport
        self.transport.start().await?;

        // Initialize Raft
        let (raft_node, _commit_rx) = RaftNode::new(
            self.node_id.clone(),
            Arc::clone(&self.transport),
            self.config.raft.clone(),
        );
        let raft = Arc::new(raft_node);

        // Add peers to Raft
        for node in self.config.topology.nodes.values() {
            if node.id != self.node_id {
                raft.add_peer(node.id.clone()).await;
                self.transport.add_node(node.id.clone(), node.addr).await;
            }
        }

        raft.clone().start().await;
        self.raft = Some(raft);

        // Initialize membership manager
        let membership = Arc::new(MembershipManager::new(
            self.node_id.clone(),
            self.config.topology.get_node(&self.node_id).unwrap().addr,
            Arc::clone(&self.transport),
        ));

        // Add all nodes to membership
        for node in self.config.topology.nodes.values() {
            if node.id != self.node_id {
                let member = Member::new(node.id.clone(), node.addr);
                let _ = membership.add_member(member).await;
            }
        }

        membership.clone().start().await;
        self.membership = Some(membership);

        // Initialize replication manager
        let state_machine = Box::new(MemoryStateMachine::new());
        let (replication, _apply_rx) = ReplicationManager::new(
            self.node_id.clone(),
            state_machine,
            self.config.raft.snapshot_threshold,
        );
        self.replication = Some(Arc::new(replication));

        // Initialize failover manager
        let (failover, _event_rx) = FailoverManager::new(
            self.node_id.clone(),
            self.config.failover.clone(),
        );
        let failover = Arc::new(failover);

        // Add all nodes to failover monitoring
        for node in self.config.topology.nodes.values() {
            failover.add_node(node.id.clone()).await;
        }

        failover.clone().start().await;
        self.failover = Some(failover);

        // Initialize load balancer
        let load_balancer = Arc::new(LoadBalancer::new(Strategy::LeastConnections));

        // Add backends
        for node in self.config.topology.nodes.values() {
            let backend = Backend::new(
                node.id.clone(),
                node.weight,
                node.max_connections,
            );
            load_balancer.add_backend(backend).await?;
        }

        self.load_balancer = Some(load_balancer);

        // Initialize quorum manager
        let quorum_config = QuorumConfig::new(self.config.topology.voters().len());
        let quorum = Arc::new(QuorumManager::new(
            self.node_id.clone(),
            quorum_config,
        )?);

        // Add voters
        for node in self.config.topology.voters() {
            quorum.add_voter(node.id.clone()).await;
        }

        // Add witnesses
        for node in self.config.topology.witnesses() {
            quorum.add_witness(node.id.clone()).await;
        }

        self.quorum = Some(quorum);

        *initialized = true;
        log::info!("Cluster manager started successfully");

        Ok(())
    }

    /// Shutdown the cluster manager
    pub async fn shutdown(&mut self) -> ClusterResult<()> {
        let mut initialized = self.initialized.write().await;
        if !*initialized {
            return Ok(());
        }

        log::info!("Shutting down cluster manager");

        // Gracefully leave cluster
        if let Some(membership) = &self.membership {
            let _ = membership.leave().await;
        }

        // Clean up components
        self.raft = None;
        self.membership = None;
        self.replication = None;
        self.failover = None;
        self.load_balancer = None;
        self.quorum = None;

        *initialized = false;
        log::info!("Cluster manager shutdown complete");

        Ok(())
    }

    /// Get reference to Raft node
    pub fn raft(&self) -> Option<&Arc<RaftNode>> {
        self.raft.as_ref()
    }

    /// Get reference to membership manager
    pub fn membership(&self) -> Option<&Arc<MembershipManager>> {
        self.membership.as_ref()
    }

    /// Get reference to replication manager
    pub fn replication(&self) -> Option<&Arc<ReplicationManager>> {
        self.replication.as_ref()
    }

    /// Get reference to failover manager
    pub fn failover(&self) -> Option<&Arc<FailoverManager>> {
        self.failover.as_ref()
    }

    /// Get reference to load balancer
    pub fn load_balancer(&self) -> Option<&Arc<LoadBalancer>> {
        self.load_balancer.as_ref()
    }

    /// Get reference to quorum manager
    pub fn quorum(&self) -> Option<&Arc<QuorumManager>> {
        self.quorum.as_ref()
    }

    /// Check if cluster is initialized
    pub async fn is_initialized(&self) -> bool {
        *self.initialized.read().await
    }

    /// Get cluster configuration
    pub fn config(&self) -> &ClusterConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[tokio::test]
    async fn test_cluster_manager_creation() {
        let mut config = ClusterConfig::new("test-cluster".to_string());
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let node = NodeConfig::new("node1".to_string(), addr, NodeRole::Voter);
        config.topology.add_node(node);

        let manager = ClusterManager::new("node1".to_string(), config).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_cluster_initialization() {
        let mut config = ClusterConfig::new("test-cluster".to_string());

        // Add 3 nodes
        for i in 0..3 {
            let addr = SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                8080 + i,
            );
            let node = NodeConfig::new(format!("node{}", i), addr, NodeRole::Voter);
            config.topology.add_node(node);
        }

        let mut manager = ClusterManager::new("node0".to_string(), config)
            .await
            .unwrap();

        assert!(!manager.is_initialized().await);

        // Note: Full initialization would require network setup
        // In production testing, use integration tests
    }

    #[test]
    fn test_node_config() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let node = NodeConfig::new("node1".to_string(), addr, NodeRole::Voter)
            .with_priority(150)
            .with_weight(200);

        assert_eq!(node.priority, 150);
        assert_eq!(node.weight, 200);
    }
}
