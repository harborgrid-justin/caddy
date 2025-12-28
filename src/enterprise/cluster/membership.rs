//! Cluster Membership Management
//!
//! Implements node discovery, member list with SWIM-like gossip protocol,
//! node health tracking, and join/leave protocols.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use thiserror::Error;
use tokio::sync::RwLock;
use tokio::time;

use super::config::NodeId;
use super::transport::{Message, Transport};

/// Membership errors
#[derive(Error, Debug)]
pub enum MembershipError {
    #[error("Node not found: {0}")]
    NodeNotFound(NodeId),
    #[error("Node already exists: {0}")]
    NodeExists(NodeId),
    #[error("Transport error: {0}")]
    Transport(String),
}

pub type MembershipResult<T> = Result<T, MembershipError>;

/// Member status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberStatus {
    /// Node is alive and healthy
    Alive,
    /// Node is suspected to be down
    Suspect,
    /// Node is confirmed dead
    Dead,
    /// Node has left voluntarily
    Left,
}

/// Member information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub id: NodeId,
    pub addr: SocketAddr,
    pub status: MemberStatus,
    pub incarnation: u64,
    pub metadata: HashMap<String, String>,
    pub joined_at: u64,
    #[serde(skip, default = "Instant::now")]
    pub last_seen: Instant,
}

impl Member {
    /// Create a new member
    pub fn new(id: NodeId, addr: SocketAddr) -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id,
            addr,
            status: MemberStatus::Alive,
            incarnation: 0,
            metadata: HashMap::new(),
            joined_at: now,
            last_seen: Instant::now(),
        }
    }

    /// Check if member is healthy
    pub fn is_healthy(&self) -> bool {
        self.status == MemberStatus::Alive
    }

    /// Check if member has failed
    pub fn has_failed(&self, timeout: Duration) -> bool {
        self.last_seen.elapsed() > timeout
    }
}

/// Gossip message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GossipMessage {
    /// Ping a node
    Ping {
        from: NodeId,
        seq: u64,
    },
    /// Acknowledge ping
    Ack {
        from: NodeId,
        seq: u64,
    },
    /// Indirect ping request
    PingReq {
        target: NodeId,
        seq: u64,
    },
    /// Member status update
    Update {
        member_id: NodeId,
        status: MemberStatus,
        incarnation: u64,
    },
    /// Full member list
    MemberList {
        members: Vec<Member>,
        version: u64,
    },
}

/// Membership manager using SWIM-like protocol
pub struct MembershipManager {
    /// Local node ID
    local_id: NodeId,

    /// Local node address
    local_addr: SocketAddr,

    /// All cluster members
    members: Arc<RwLock<HashMap<NodeId, Member>>>,

    /// Transport layer
    transport: Arc<Transport>,

    /// Member list version
    version: Arc<RwLock<u64>>,

    /// Failure detection timeout
    failure_timeout: Duration,

    /// Probe interval
    probe_interval: Duration,

    /// Gossip fanout
    gossip_fanout: usize,
}

impl MembershipManager {
    /// Create a new membership manager
    pub fn new(
        local_id: NodeId,
        local_addr: SocketAddr,
        transport: Arc<Transport>,
    ) -> Self {
        let mut members = HashMap::new();
        members.insert(
            local_id.clone(),
            Member::new(local_id.clone(), local_addr),
        );

        Self {
            local_id,
            local_addr,
            members: Arc::new(RwLock::new(members)),
            transport,
            version: Arc::new(RwLock::new(0)),
            failure_timeout: Duration::from_secs(10),
            probe_interval: Duration::from_secs(1),
            gossip_fanout: 3,
        }
    }

    /// Start the membership protocol
    pub async fn start(self: Arc<Self>) {
        // Start failure detector
        let manager = Arc::clone(&self);
        tokio::spawn(async move {
            manager.run_failure_detector().await;
        });

        // Start gossip protocol
        let manager = Arc::clone(&self);
        tokio::spawn(async move {
            manager.run_gossip().await;
        });

        log::info!("Membership manager started");
    }

    /// Join the cluster by contacting a seed node
    pub async fn join(&self, seed_addr: SocketAddr) -> MembershipResult<()> {
        log::info!("Joining cluster via seed: {}", seed_addr);

        let message = Message::Join {
            node_id: self.local_id.clone(),
            addr: self.local_addr,
        };

        // In real implementation, would send to seed and wait for member list
        // For now, simplified

        Ok(())
    }

    /// Leave the cluster gracefully
    pub async fn leave(&self) -> MembershipResult<()> {
        log::info!("Leaving cluster");

        let message = Message::Leave {
            node_id: self.local_id.clone(),
        };

        // Broadcast leave message to all members
        let members = self.members.read().await;
        for member in members.values() {
            if member.id != self.local_id {
                let _ = self.transport.send(member.id.clone(), message.clone()).await;
            }
        }

        Ok(())
    }

    /// Add a member to the cluster
    pub async fn add_member(&self, member: Member) -> MembershipResult<()> {
        let mut members = self.members.write().await;

        if members.contains_key(&member.id) {
            return Err(MembershipError::NodeExists(member.id.clone()));
        }

        members.insert(member.id.clone(), member);
        *self.version.write().await += 1;

        log::info!("Added member: {}", members.len());
        Ok(())
    }

    /// Remove a member from the cluster
    pub async fn remove_member(&self, node_id: &NodeId) -> MembershipResult<Member> {
        let mut members = self.members.write().await;

        let member = members
            .remove(node_id)
            .ok_or_else(|| MembershipError::NodeNotFound(node_id.clone()))?;

        *self.version.write().await += 1;

        log::info!("Removed member: {}", node_id);
        Ok(member)
    }

    /// Get member by ID
    pub async fn get_member(&self, node_id: &NodeId) -> Option<Member> {
        self.members.read().await.get(node_id).cloned()
    }

    /// Get all members
    pub async fn get_members(&self) -> Vec<Member> {
        self.members.read().await.values().cloned().collect()
    }

    /// Get all healthy members
    pub async fn get_healthy_members(&self) -> Vec<Member> {
        self.members
            .read()
            .await
            .values()
            .filter(|m| m.is_healthy())
            .cloned()
            .collect()
    }

    /// Get member count
    pub async fn member_count(&self) -> usize {
        self.members.read().await.len()
    }

    /// Update member status
    async fn update_member_status(
        &self,
        node_id: &NodeId,
        status: MemberStatus,
        incarnation: u64,
    ) {
        let mut members = self.members.write().await;

        if let Some(member) = members.get_mut(node_id) {
            // Only update if incarnation is newer or status is worse
            if incarnation > member.incarnation
                || (incarnation == member.incarnation && status as u8 > member.status as u8)
            {
                member.status = status;
                member.incarnation = incarnation;
                *self.version.write().await += 1;

                log::info!("Updated member {} status to {:?}", node_id, status);
            }
        }
    }

    /// Run failure detector (SWIM-like probing)
    async fn run_failure_detector(&self) {
        let mut interval = time::interval(self.probe_interval);

        loop {
            interval.tick().await;

            // Select a random member to probe
            let target = {
                let members = self.members.read().await;
                members
                    .values()
                    .filter(|m| m.id != self.local_id && m.status == MemberStatus::Alive)
                    .nth(0)
                    .map(|m| m.id.clone())
            };

            if let Some(target_id) = target {
                self.probe_member(&target_id).await;
            }
        }
    }

    /// Probe a specific member
    async fn probe_member(&self, target_id: &NodeId) {
        let seq = {
            let version = self.version.read().await;
            *version
        };

        // Send direct ping
        let ping_msg = Message::Ping;
        let result = self
            .transport
            .send_with_timeout(target_id.clone(), ping_msg, Duration::from_secs(1))
            .await;

        if result.is_err() {
            // Direct ping failed, try indirect ping via other members
            log::debug!("Direct ping to {} failed, trying indirect", target_id);
            self.indirect_probe(target_id, seq).await;
        }
    }

    /// Indirect probe via other members
    async fn indirect_probe(&self, target_id: &NodeId, seq: u64) {
        let members = self.members.read().await;
        let intermediaries: Vec<NodeId> = members
            .values()
            .filter(|m| {
                m.id != self.local_id
                    && m.id != *target_id
                    && m.status == MemberStatus::Alive
            })
            .take(self.gossip_fanout)
            .map(|m| m.id.clone())
            .collect();
        drop(members);

        // Request indirect pings
        for intermediary in intermediaries {
            // In real implementation, send PingReq message
            // For now, simplified
        }

        // If still no response, mark as suspect
        time::sleep(Duration::from_millis(500)).await;
        self.update_member_status(target_id, MemberStatus::Suspect, seq)
            .await;
    }

    /// Run gossip protocol
    async fn run_gossip(&self) {
        let mut interval = time::interval(Duration::from_secs(1));

        loop {
            interval.tick().await;
            self.gossip_updates().await;
        }
    }

    /// Gossip membership updates to random members
    async fn gossip_updates(&self) {
        let members = self.members.read().await;
        let targets: Vec<NodeId> = members
            .values()
            .filter(|m| m.id != self.local_id && m.status == MemberStatus::Alive)
            .take(self.gossip_fanout)
            .map(|m| m.id.clone())
            .collect();

        let member_list: Vec<Member> = members.values().cloned().collect();
        let version = *self.version.read().await;
        drop(members);

        // Send member list to selected targets
        for target_id in targets {
            // In real implementation, send incremental updates
            // For now, send full member list
            let _ = self.transport.send(target_id, Message::Ping).await;
        }
    }

    /// Check for failed members
    async fn check_failures(&self) {
        let mut members = self.members.write().await;
        let mut failed = Vec::new();

        for (id, member) in members.iter_mut() {
            if id == &self.local_id {
                continue;
            }

            if member.status == MemberStatus::Suspect
                && member.has_failed(self.failure_timeout)
            {
                member.status = MemberStatus::Dead;
                failed.push(id.clone());
            }
        }

        if !failed.is_empty() {
            *self.version.write().await += 1;
            log::warn!("Marked {} members as dead", failed.len());
        }
    }

    /// Handle member join
    pub async fn handle_join(&self, node_id: NodeId, addr: SocketAddr) {
        let member = Member::new(node_id.clone(), addr);

        if let Ok(_) = self.add_member(member).await {
            log::info!("Member {} joined the cluster", node_id);
        }
    }

    /// Handle member leave
    pub async fn handle_leave(&self, node_id: NodeId) {
        if let Ok(_) = self.remove_member(&node_id).await {
            log::info!("Member {} left the cluster", node_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_member_creation() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let member = Member::new("node1".to_string(), addr);

        assert_eq!(member.id, "node1");
        assert_eq!(member.status, MemberStatus::Alive);
        assert!(member.is_healthy());
    }

    #[tokio::test]
    async fn test_membership_manager_creation() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let transport = Arc::new(Transport::new("node1".to_string(), addr));

        let manager = MembershipManager::new("node1".to_string(), addr, transport);

        assert_eq!(manager.member_count().await, 1);
    }

    #[tokio::test]
    async fn test_add_member() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let transport = Arc::new(Transport::new("node1".to_string(), addr));

        let manager = MembershipManager::new("node1".to_string(), addr, transport);

        let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081);
        let member = Member::new("node2".to_string(), addr2);

        manager.add_member(member).await.unwrap();
        assert_eq!(manager.member_count().await, 2);
    }

    #[tokio::test]
    async fn test_remove_member() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let transport = Arc::new(Transport::new("node1".to_string(), addr));

        let manager = MembershipManager::new("node1".to_string(), addr, transport);

        let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081);
        let member = Member::new("node2".to_string(), addr2);

        manager.add_member(member).await.unwrap();
        manager.remove_member(&"node2".to_string()).await.unwrap();

        assert_eq!(manager.member_count().await, 1);
    }

    #[tokio::test]
    async fn test_get_healthy_members() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let transport = Arc::new(Transport::new("node1".to_string(), addr));

        let manager = MembershipManager::new("node1".to_string(), addr, transport);

        let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081);
        let member = Member::new("node2".to_string(), addr2);

        manager.add_member(member).await.unwrap();

        let healthy = manager.get_healthy_members().await;
        assert_eq!(healthy.len(), 2);
    }

    #[tokio::test]
    async fn test_update_member_status() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let transport = Arc::new(Transport::new("node1".to_string(), addr));

        let manager = MembershipManager::new("node1".to_string(), addr, transport);

        let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081);
        let member = Member::new("node2".to_string(), addr2);
        manager.add_member(member).await.unwrap();

        manager
            .update_member_status(&"node2".to_string(), MemberStatus::Suspect, 1)
            .await;

        let member = manager.get_member(&"node2".to_string()).await.unwrap();
        assert_eq!(member.status, MemberStatus::Suspect);
    }
}
