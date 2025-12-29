//! Node Communication Transport
//!
//! Provides TCP transport, message framing, heartbeat management,
//! and connection pooling for cluster communication.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio::time;

use super::config::NodeId;

/// Transport errors
#[derive(Error, Debug)]
pub enum TransportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Message too large: {0} bytes")]
    MessageTooLarge(usize),
    #[error("Timeout")]
    Timeout,
    #[error("Node not found: {0}")]
    NodeNotFound(NodeId),
}

pub type TransportResult<T> = Result<T, TransportError>;

/// Message types for cluster communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    /// Raft messages
    AppendEntries {
        term: u64,
        leader_id: NodeId,
        prev_log_index: u64,
        prev_log_term: u64,
        entries: Vec<Vec<u8>>,
        leader_commit: u64,
    },
    AppendEntriesResponse {
        term: u64,
        success: bool,
        match_index: u64,
    },
    RequestVote {
        term: u64,
        candidate_id: NodeId,
        last_log_index: u64,
        last_log_term: u64,
    },
    RequestVoteResponse {
        term: u64,
        vote_granted: bool,
    },
    InstallSnapshot {
        term: u64,
        leader_id: NodeId,
        last_included_index: u64,
        last_included_term: u64,
        offset: u64,
        data: Vec<u8>,
        done: bool,
    },
    InstallSnapshotResponse {
        term: u64,
    },

    /// Heartbeat messages
    Heartbeat {
        from: NodeId,
        term: u64,
        timestamp: u64,
    },
    HeartbeatAck {
        from: NodeId,
        timestamp: u64,
    },

    /// Cluster membership messages
    Join {
        node_id: NodeId,
        addr: SocketAddr,
    },
    Leave {
        node_id: NodeId,
    },
    MembershipUpdate {
        members: Vec<(NodeId, SocketAddr)>,
        version: u64,
    },

    /// Health check
    Ping,
    Pong,
}

/// Message envelope with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    /// Source node
    pub from: NodeId,
    /// Destination node
    pub to: NodeId,
    /// Message ID for tracking
    pub msg_id: u64,
    /// Message payload
    pub message: Message,
}

/// Connection state
struct Connection {
    stream: TcpStream,
    last_activity: Instant,
    pending_writes: usize,
}

/// Connection pool for a remote node
struct ConnectionPool {
    node_id: NodeId,
    addr: SocketAddr,
    connections: Vec<Connection>,
    max_size: usize,
}

impl ConnectionPool {
    fn new(node_id: NodeId, addr: SocketAddr, max_size: usize) -> Self {
        Self {
            node_id,
            addr,
            connections: Vec::new(),
            max_size,
        }
    }

    async fn get_connection(&mut self) -> TransportResult<&mut Connection> {
        // Remove stale connections
        self.connections.retain(|conn| {
            conn.last_activity.elapsed() < Duration::from_secs(300)
        });

        // Reuse existing idle connection (use position to get index, avoiding borrow conflicts)
        if let Some(idx) = self.connections.iter().position(|c| c.pending_writes == 0) {
            return Ok(&mut self.connections[idx]);
        }

        // Create new connection if under limit
        if self.connections.len() < self.max_size {
            let stream = time::timeout(
                Duration::from_secs(5),
                TcpStream::connect(self.addr),
            )
            .await
            .map_err(|_| TransportError::Timeout)?
            .map_err(|e| TransportError::Connection(e.to_string()))?;

            stream.set_nodelay(true)?;

            let conn = Connection {
                stream,
                last_activity: Instant::now(),
                pending_writes: 0,
            };
            self.connections.push(conn);
            let last_idx = self.connections.len() - 1;
            return Ok(&mut self.connections[last_idx]);
        }

        // Wait for available connection (simple round-robin)
        let idx = self.connections.len() % self.max_size;
        Ok(&mut self.connections[idx])
    }
}

/// Network transport layer
pub struct Transport {
    local_id: NodeId,
    local_addr: SocketAddr,
    pools: Arc<RwLock<HashMap<NodeId, ConnectionPool>>>,
    incoming_tx: mpsc::UnboundedSender<Envelope>,
    incoming_rx: Option<mpsc::UnboundedReceiver<Envelope>>,
    max_message_size: usize,
    next_msg_id: Arc<RwLock<u64>>,
}

impl Transport {
    /// Create a new transport
    pub fn new(local_id: NodeId, local_addr: SocketAddr) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            local_id,
            local_addr,
            pools: Arc::new(RwLock::new(HashMap::new())),
            incoming_tx: tx,
            incoming_rx: Some(rx),
            max_message_size: 16 * 1024 * 1024, // 16 MB
            next_msg_id: Arc::new(RwLock::new(0)),
        }
    }

    /// Start listening for incoming connections
    pub async fn start(&self) -> TransportResult<()> {
        let listener = TcpListener::bind(self.local_addr).await?;
        let incoming_tx = self.incoming_tx.clone();
        let max_message_size = self.max_message_size;

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _addr)) => {
                        let tx = incoming_tx.clone();
                        tokio::spawn(Self::handle_connection(stream, tx, max_message_size));
                    }
                    Err(e) => {
                        log::error!("Accept error: {}", e);
                    }
                }
            }
        });

        log::info!("Transport listening on {}", self.local_addr);
        Ok(())
    }

    /// Take the incoming message receiver
    pub fn take_receiver(&mut self) -> Option<mpsc::UnboundedReceiver<Envelope>> {
        self.incoming_rx.take()
    }

    /// Handle incoming connection
    async fn handle_connection(
        mut stream: TcpStream,
        tx: mpsc::UnboundedSender<Envelope>,
        max_message_size: usize,
    ) {
        loop {
            match Self::read_message(&mut stream, max_message_size).await {
                Ok(envelope) => {
                    if tx.send(envelope).is_err() {
                        break;
                    }
                }
                Err(e) => {
                    log::debug!("Connection read error: {}", e);
                    break;
                }
            }
        }
    }

    /// Read a message from stream
    async fn read_message(
        stream: &mut TcpStream,
        max_message_size: usize,
    ) -> TransportResult<Envelope> {
        // Read message length (4 bytes)
        let len = stream.read_u32().await? as usize;

        if len > max_message_size {
            return Err(TransportError::MessageTooLarge(len));
        }

        // Read message data
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf).await?;

        // Deserialize
        bincode::deserialize(&buf)
            .map_err(|e| TransportError::Serialization(e.to_string()))
    }

    /// Write a message to stream
    async fn write_message(
        stream: &mut TcpStream,
        envelope: &Envelope,
    ) -> TransportResult<()> {
        // Serialize message
        let data = bincode::serialize(envelope)
            .map_err(|e| TransportError::Serialization(e.to_string()))?;

        // Write length prefix
        stream.write_u32(data.len() as u32).await?;

        // Write data
        stream.write_all(&data).await?;
        stream.flush().await?;

        Ok(())
    }

    /// Send message to a node
    pub async fn send(
        &self,
        to: NodeId,
        message: Message,
    ) -> TransportResult<()> {
        let msg_id = {
            let mut next_id = self.next_msg_id.write().await;
            let id = *next_id;
            *next_id += 1;
            id
        };

        let envelope = Envelope {
            from: self.local_id.clone(),
            to: to.clone(),
            msg_id,
            message,
        };

        // Get connection from pool
        let mut pools = self.pools.write().await;
        let pool = pools.get_mut(&to)
            .ok_or_else(|| TransportError::NodeNotFound(to.clone()))?;

        let conn = pool.get_connection().await?;
        conn.pending_writes += 1;

        // Send message
        let result = Self::write_message(&mut conn.stream, &envelope).await;

        conn.pending_writes -= 1;
        conn.last_activity = Instant::now();

        result
    }

    /// Send message with timeout
    pub async fn send_with_timeout(
        &self,
        to: NodeId,
        message: Message,
        timeout: Duration,
    ) -> TransportResult<()> {
        time::timeout(timeout, self.send(to, message))
            .await
            .map_err(|_| TransportError::Timeout)?
    }

    /// Add a node to the connection pool
    pub async fn add_node(&self, node_id: NodeId, addr: SocketAddr) {
        let mut pools = self.pools.write().await;
        pools.insert(
            node_id.clone(),
            ConnectionPool::new(node_id, addr, 4),
        );
    }

    /// Remove a node from the connection pool
    pub async fn remove_node(&self, node_id: &NodeId) {
        let mut pools = self.pools.write().await;
        pools.remove(node_id);
    }

    /// Get connection statistics
    pub async fn stats(&self) -> HashMap<NodeId, ConnectionStats> {
        let pools = self.pools.read().await;
        pools.iter().map(|(id, pool)| {
            let stats = ConnectionStats {
                active_connections: pool.connections.len(),
                max_connections: pool.max_size,
            };
            (id.clone(), stats)
        }).collect()
    }
}

/// Connection statistics
#[derive(Debug, Clone)]
pub struct ConnectionStats {
    pub active_connections: usize,
    pub max_connections: usize,
}

/// Heartbeat manager
pub struct HeartbeatManager {
    node_id: NodeId,
    interval: Duration,
    transport: Arc<Transport>,
    peers: Arc<RwLock<Vec<NodeId>>>,
}

impl HeartbeatManager {
    /// Create a new heartbeat manager
    pub fn new(
        node_id: NodeId,
        interval: Duration,
        transport: Arc<Transport>,
    ) -> Self {
        Self {
            node_id,
            interval,
            transport,
            peers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add a peer to send heartbeats to
    pub async fn add_peer(&self, peer_id: NodeId) {
        let mut peers = self.peers.write().await;
        if !peers.contains(&peer_id) {
            peers.push(peer_id);
        }
    }

    /// Remove a peer
    pub async fn remove_peer(&self, peer_id: &NodeId) {
        let mut peers = self.peers.write().await;
        peers.retain(|p| p != peer_id);
    }

    /// Start sending heartbeats
    pub async fn start(&self, term: u64) {
        let node_id = self.node_id.clone();
        let interval = self.interval;
        let transport = Arc::clone(&self.transport);
        let peers = Arc::clone(&self.peers);

        tokio::spawn(async move {
            let mut ticker = time::interval(interval);
            loop {
                ticker.tick().await;

                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                let message = Message::Heartbeat {
                    from: node_id.clone(),
                    term,
                    timestamp,
                };

                let peer_list = peers.read().await;
                for peer_id in peer_list.iter() {
                    let _ = transport.send(peer_id.clone(), message.clone()).await;
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_message_serialization() {
        let msg = Message::Ping;
        let envelope = Envelope {
            from: "node1".to_string(),
            to: "node2".to_string(),
            msg_id: 42,
            message: msg,
        };

        let serialized = bincode::serialize(&envelope).unwrap();
        let deserialized: Envelope = bincode::deserialize(&serialized).unwrap();

        assert_eq!(deserialized.from, "node1");
        assert_eq!(deserialized.to, "node2");
        assert_eq!(deserialized.msg_id, 42);
    }

    #[tokio::test]
    async fn test_transport_creation() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
        let transport = Transport::new("node1".to_string(), addr);

        assert_eq!(transport.local_id, "node1");
    }

    #[tokio::test]
    async fn test_connection_pool() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let mut pool = ConnectionPool::new("node1".to_string(), addr, 4);

        assert_eq!(pool.connections.len(), 0);
        assert_eq!(pool.max_size, 4);
    }

    #[tokio::test]
    async fn test_heartbeat_manager() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
        let transport = Arc::new(Transport::new("node1".to_string(), addr));

        let hb_manager = HeartbeatManager::new(
            "node1".to_string(),
            Duration::from_millis(100),
            transport,
        );

        hb_manager.add_peer("node2".to_string()).await;
        let peers = hb_manager.peers.read().await;
        assert_eq!(peers.len(), 1);
    }
}
