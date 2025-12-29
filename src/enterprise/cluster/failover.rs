//! Automatic Failover
//!
//! Implements automatic leader failover, read replica promotion,
//! session migration, and connection draining.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use tokio::time;

use super::config::{FailoverConfig, NodeId};


/// Failover errors
#[derive(Error, Debug)]
pub enum FailoverError {
    #[error("No healthy nodes available")]
    NoHealthyNodes,
    #[error("Failover already in progress")]
    InProgress,
    #[error("Failover disabled")]
    Disabled,
    #[error("Session migration failed: {0}")]
    SessionMigration(String),
}

pub type FailoverResult<T> = Result<T, FailoverError>;

/// Failover event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailoverEvent {
    /// Leader failure detected
    LeaderFailed {
        node_id: NodeId,
        timestamp: u64,
    },
    /// Failover started
    FailoverStarted {
        from_node: NodeId,
        to_node: NodeId,
        timestamp: u64,
    },
    /// Failover completed
    FailoverCompleted {
        new_leader: NodeId,
        timestamp: u64,
        duration_ms: u64,
    },
    /// Failover failed
    FailoverFailed {
        reason: String,
        timestamp: u64,
    },
    /// Node promoted
    NodePromoted {
        node_id: NodeId,
        timestamp: u64,
    },
}

/// Node health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub node_id: NodeId,
    pub is_healthy: bool,
    #[serde(skip, default = "Instant::now")]
    pub last_check: Instant,
    pub failure_count: usize,
    pub response_time_ms: u64,
}

impl HealthStatus {
    fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            is_healthy: true,
            last_check: Instant::now(),
            failure_count: 0,
            response_time_ms: 0,
        }
    }

    fn mark_failed(&mut self) {
        self.failure_count += 1;
        self.last_check = Instant::now();
    }

    fn mark_healthy(&mut self, response_time_ms: u64) {
        self.is_healthy = true;
        self.failure_count = 0;
        self.response_time_ms = response_time_ms;
        self.last_check = Instant::now();
    }

    fn should_failover(&self, threshold: usize) -> bool {
        self.failure_count >= threshold
    }
}

/// Session information for migration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    pub user_id: String,
    pub state: Vec<u8>,
    pub created_at: u64,
    pub last_activity: u64,
}

/// Connection information for draining
#[derive(Debug, Clone)]
pub struct Connection {
    pub conn_id: String,
    pub established_at: Instant,
    pub last_activity: Instant,
    pub active: bool,
}

impl Connection {
    fn new(conn_id: String) -> Self {
        Self {
            conn_id,
            established_at: Instant::now(),
            last_activity: Instant::now(),
            active: true,
        }
    }

    fn is_idle(&self, timeout: Duration) -> bool {
        self.last_activity.elapsed() > timeout
    }
}

/// Failover manager
pub struct FailoverManager {
    /// Local node ID
    node_id: NodeId,

    /// Configuration
    config: FailoverConfig,

    /// Health status of nodes
    health_status: Arc<RwLock<HashMap<NodeId, HealthStatus>>>,

    /// Current leader
    current_leader: Arc<RwLock<Option<NodeId>>>,

    /// Failover in progress
    failover_in_progress: Arc<RwLock<bool>>,

    /// Active sessions (for migration)
    sessions: Arc<RwLock<HashMap<String, Session>>>,

    /// Active connections (for draining)
    connections: Arc<RwLock<HashMap<String, Connection>>>,

    /// Event notification channel
    event_tx: mpsc::UnboundedSender<FailoverEvent>,
}

impl FailoverManager {
    /// Create a new failover manager
    pub fn new(
        node_id: NodeId,
        config: FailoverConfig,
    ) -> (Self, mpsc::UnboundedReceiver<FailoverEvent>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let manager = Self {
            node_id,
            config,
            health_status: Arc::new(RwLock::new(HashMap::new())),
            current_leader: Arc::new(RwLock::new(None)),
            failover_in_progress: Arc::new(RwLock::new(false)),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
        };

        (manager, event_rx)
    }

    /// Start the failover manager
    pub async fn start(self: Arc<Self>) {
        if !self.config.enable_auto_failover {
            log::info!("Auto-failover is disabled");
            return;
        }

        // Start health checker
        let manager = Arc::clone(&self);
        tokio::spawn(async move {
            manager.run_health_checker().await;
        });

        // Start session cleanup
        let manager = Arc::clone(&self);
        tokio::spawn(async move {
            manager.run_session_cleanup().await;
        });

        log::info!("Failover manager started");
    }

    /// Run health checker
    async fn run_health_checker(self: Arc<Self>) {
        let mut interval = time::interval(self.config.health_check_interval);

        loop {
            interval.tick().await;
            self.check_health().await;
        }
    }

    /// Check health of all nodes
    async fn check_health(&self) {
        let mut health_status = self.health_status.write().await;

        for (node_id, status) in health_status.iter_mut() {
            // Simulate health check (in real implementation, ping node)
            let start = Instant::now();
            let is_healthy = self.perform_health_check(node_id).await;
            let elapsed = start.elapsed().as_millis() as u64;

            if is_healthy {
                status.mark_healthy(elapsed);
            } else {
                status.mark_failed();

                // Check if failover is needed
                if status.should_failover(self.config.failure_threshold) {
                    let is_leader = {
                        let leader = self.current_leader.read().await;
                        leader.as_ref() == Some(node_id)
                    };

                    if is_leader {
                        log::warn!("Leader {} failed, initiating failover", node_id);
                        let failed_node = node_id.clone();
                        drop(health_status);
                        let _ = self.initiate_failover(failed_node).await;
                        return;
                    }
                }
            }
        }
    }

    /// Perform health check on a node
    async fn perform_health_check(&self, _node_id: &NodeId) -> bool {
        // Simplified: In real implementation, send health check request
        // For now, assume healthy
        true
    }

    /// Add node to health monitoring
    pub async fn add_node(&self, node_id: NodeId) {
        let mut health_status = self.health_status.write().await;
        health_status.insert(node_id.clone(), HealthStatus::new(node_id));
    }

    /// Remove node from health monitoring
    pub async fn remove_node(&self, node_id: &NodeId) {
        let mut health_status = self.health_status.write().await;
        health_status.remove(node_id);
    }

    /// Set current leader
    pub async fn set_leader(&self, leader_id: Option<NodeId>) {
        *self.current_leader.write().await = leader_id;
    }

    /// Initiate failover
    async fn initiate_failover(&self, failed_node: NodeId) -> FailoverResult<()> {
        // Check if failover is already in progress
        let mut in_progress = self.failover_in_progress.write().await;
        if *in_progress {
            return Err(FailoverError::InProgress);
        }
        *in_progress = true;
        drop(in_progress);

        let start_time = Instant::now();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Emit failure event
        let _ = self.event_tx.send(FailoverEvent::LeaderFailed {
            node_id: failed_node.clone(),
            timestamp,
        });

        // Wait grace period
        time::sleep(self.config.failover_grace_period).await;

        // Select new leader
        let new_leader = self.select_new_leader().await?;

        // Emit failover started event
        let _ = self.event_tx.send(FailoverEvent::FailoverStarted {
            from_node: failed_node.clone(),
            to_node: new_leader.clone(),
            timestamp,
        });

        // Drain connections from failed node
        if self.config.enable_session_migration {
            self.drain_connections().await?;
        }

        // Promote new leader
        self.promote_node(&new_leader).await?;

        // Update current leader
        *self.current_leader.write().await = Some(new_leader.clone());

        // Complete failover
        *self.failover_in_progress.write().await = false;

        let duration_ms = start_time.elapsed().as_millis() as u64;

        // Emit completion event
        let _ = self.event_tx.send(FailoverEvent::FailoverCompleted {
            new_leader: new_leader.clone(),
            timestamp,
            duration_ms,
        });

        log::info!(
            "Failover completed: {} -> {} ({}ms)",
            failed_node,
            new_leader,
            duration_ms
        );

        Ok(())
    }

    /// Select new leader from healthy nodes
    async fn select_new_leader(&self) -> FailoverResult<NodeId> {
        let health_status = self.health_status.read().await;

        // Find healthy nodes
        let mut candidates: Vec<(&NodeId, &HealthStatus)> = health_status
            .iter()
            .filter(|(id, status)| {
                status.is_healthy && **id != self.node_id
            })
            .collect();

        if candidates.is_empty() {
            return Err(FailoverError::NoHealthyNodes);
        }

        // Sort by response time (lower is better)
        candidates.sort_by_key(|(_, status)| status.response_time_ms);

        Ok(candidates[0].0.clone())
    }

    /// Promote a node to leader
    async fn promote_node(&self, node_id: &NodeId) -> FailoverResult<()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        log::info!("Promoting node {} to leader", node_id);

        // In real implementation, send promotion request to node
        // For now, just emit event

        let _ = self.event_tx.send(FailoverEvent::NodePromoted {
            node_id: node_id.clone(),
            timestamp,
        });

        Ok(())
    }

    /// Drain connections with timeout
    async fn drain_connections(&self) -> FailoverResult<()> {
        log::info!("Draining connections...");

        let start = Instant::now();
        let timeout = self.config.drain_timeout;

        loop {
            let connections = self.connections.read().await;
            let active_count = connections.values().filter(|c| c.active).count();
            drop(connections);

            if active_count == 0 {
                log::info!("All connections drained");
                return Ok(());
            }

            if start.elapsed() > timeout {
                log::warn!("Connection drain timeout, {} active connections remaining", active_count);
                return Ok(());
            }

            time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Add a connection for tracking
    pub async fn add_connection(&self, conn_id: String) {
        let mut connections = self.connections.write().await;
        connections.insert(conn_id.clone(), Connection::new(conn_id));
    }

    /// Remove a connection
    pub async fn remove_connection(&self, conn_id: &str) {
        let mut connections = self.connections.write().await;
        connections.remove(conn_id);
    }

    /// Mark connection as inactive
    pub async fn mark_connection_inactive(&self, conn_id: &str) {
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.get_mut(conn_id) {
            conn.active = false;
        }
    }

    /// Add a session
    pub async fn add_session(&self, session: Session) {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.session_id.clone(), session);
    }

    /// Remove a session
    pub async fn remove_session(&self, session_id: &str) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
    }

    /// Migrate sessions to new leader
    async fn migrate_sessions(&self, _target_node: &NodeId) -> FailoverResult<()> {
        let sessions = self.sessions.read().await;

        log::info!("Migrating {} sessions", sessions.len());

        // In real implementation, transfer sessions to target node
        // For now, simplified

        Ok(())
    }

    /// Run session cleanup
    async fn run_session_cleanup(self: Arc<Self>) {
        let mut interval = time::interval(Duration::from_secs(60));

        loop {
            interval.tick().await;

            let mut sessions = self.sessions.write().await;
            let before = sessions.len();

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            // Remove sessions inactive for > 1 hour
            sessions.retain(|_, session| {
                now - session.last_activity < 3600
            });

            let removed = before - sessions.len();
            if removed > 0 {
                log::debug!("Cleaned up {} inactive sessions", removed);
            }
        }
    }

    /// Get health status of all nodes
    pub async fn get_health_status(&self) -> HashMap<NodeId, HealthStatus> {
        self.health_status.read().await.clone()
    }

    /// Get current leader
    pub async fn get_leader(&self) -> Option<NodeId> {
        self.current_leader.read().await.clone()
    }

    /// Check if failover is in progress
    pub async fn is_failover_in_progress(&self) -> bool {
        *self.failover_in_progress.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_failover_manager_creation() {
        let config = FailoverConfig::default();
        let (manager, _rx) = FailoverManager::new("node1".to_string(), config);

        assert_eq!(manager.node_id, "node1");
        assert!(!manager.is_failover_in_progress().await);
    }

    #[tokio::test]
    async fn test_add_node() {
        let config = FailoverConfig::default();
        let (manager, _rx) = FailoverManager::new("node1".to_string(), config);

        manager.add_node("node2".to_string()).await;

        let health = manager.get_health_status().await;
        assert_eq!(health.len(), 1);
    }

    #[tokio::test]
    async fn test_connection_tracking() {
        let config = FailoverConfig::default();
        let (manager, _rx) = FailoverManager::new("node1".to_string(), config);

        manager.add_connection("conn1".to_string()).await;

        let connections = manager.connections.read().await;
        assert_eq!(connections.len(), 1);
    }

    #[tokio::test]
    async fn test_session_tracking() {
        let config = FailoverConfig::default();
        let (manager, _rx) = FailoverManager::new("node1".to_string(), config);

        let session = Session {
            session_id: "session1".to_string(),
            user_id: "user1".to_string(),
            state: vec![],
            created_at: 0,
            last_activity: 0,
        };

        manager.add_session(session).await;

        let sessions = manager.sessions.read().await;
        assert_eq!(sessions.len(), 1);
    }

    #[tokio::test]
    async fn test_health_status() {
        let mut status = HealthStatus::new("node1".to_string());

        assert!(status.is_healthy);
        assert_eq!(status.failure_count, 0);

        status.mark_failed();
        assert_eq!(status.failure_count, 1);

        status.mark_healthy(50);
        assert!(status.is_healthy);
        assert_eq!(status.failure_count, 0);
    }
}
