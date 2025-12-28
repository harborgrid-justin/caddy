//! Transport Layer for Collaboration
//!
//! This module provides a robust WebSocket transport layer with connection management,
//! automatic reconnection, state recovery, and heartbeat mechanisms.

use super::{CollaborationError, CollaborationMessage, MessageCodec, Result};
use chrono::{DateTime, Duration, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    /// Not connected
    Disconnected,
    /// Attempting to connect
    Connecting,
    /// Successfully connected
    Connected,
    /// Connection lost, attempting to reconnect
    Reconnecting,
    /// Connection closed intentionally
    Closed,
    /// Connection failed with error
    Failed,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::Disconnected
    }
}

/// Reconnection strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReconnectStrategy {
    /// No automatic reconnection
    None,
    /// Reconnect with fixed delay
    FixedDelay {
        delay_ms: u64,
        max_attempts: Option<usize>,
    },
    /// Exponential backoff
    ExponentialBackoff {
        initial_delay_ms: u64,
        max_delay_ms: u64,
        multiplier: f64,
        max_attempts: Option<usize>,
    },
}

impl Default for ReconnectStrategy {
    fn default() -> Self {
        Self::ExponentialBackoff {
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            multiplier: 2.0,
            max_attempts: Some(10),
        }
    }
}

impl ReconnectStrategy {
    /// Calculate delay for next reconnection attempt
    pub fn calculate_delay(&self, attempt: usize) -> Option<std::time::Duration> {
        match self {
            Self::None => None,

            Self::FixedDelay { delay_ms, max_attempts } => {
                if let Some(max) = max_attempts {
                    if attempt >= *max {
                        return None;
                    }
                }
                Some(std::time::Duration::from_millis(*delay_ms))
            }

            Self::ExponentialBackoff {
                initial_delay_ms,
                max_delay_ms,
                multiplier,
                max_attempts,
            } => {
                if let Some(max) = max_attempts {
                    if attempt >= *max {
                        return None;
                    }
                }

                let delay = (*initial_delay_ms as f64) * multiplier.powi(attempt as i32);
                let delay = delay.min(*max_delay_ms as f64) as u64;
                Some(std::time::Duration::from_millis(delay))
            }
        }
    }
}

/// Transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    /// WebSocket server URL
    pub server_url: String,
    /// Reconnection strategy
    pub reconnect_strategy: ReconnectStrategy,
    /// Heartbeat interval in seconds
    pub heartbeat_interval: u64,
    /// Heartbeat timeout in seconds
    pub heartbeat_timeout: u64,
    /// Maximum message queue size
    pub max_queue_size: usize,
    /// Enable compression
    pub compression_enabled: bool,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            server_url: "ws://localhost:8080".to_string(),
            reconnect_strategy: ReconnectStrategy::default(),
            heartbeat_interval: 30,
            heartbeat_timeout: 10,
            max_queue_size: 1000,
            compression_enabled: true,
            connection_timeout: 30,
        }
    }
}

/// Transport events
#[derive(Debug, Clone)]
pub enum TransportEvent {
    /// Connection established
    Connected { connection_id: Uuid },

    /// Connection lost
    Disconnected { reason: String },

    /// Reconnection attempt
    Reconnecting { attempt: usize },

    /// Reconnection successful
    Reconnected { connection_id: Uuid },

    /// Reconnection failed
    ReconnectFailed { attempts: usize },

    /// Message received
    MessageReceived { message: CollaborationMessage },

    /// Message sent successfully
    MessageSent { message_id: Uuid },

    /// Send failed
    SendFailed { error: String },

    /// Heartbeat sent
    HeartbeatSent { timestamp: i64 },

    /// Heartbeat received
    HeartbeatReceived { latency_ms: u64 },

    /// Heartbeat timeout
    HeartbeatTimeout,

    /// Error occurred
    Error { error: String },
}

/// Callback for transport events
pub type TransportCallback = Box<dyn Fn(TransportEvent) + Send + Sync>;

/// Connection metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConnectionMetrics {
    /// Total messages sent
    pub messages_sent: u64,
    /// Total messages received
    pub messages_received: u64,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Current latency in milliseconds
    pub latency_ms: u64,
    /// Average latency in milliseconds
    pub avg_latency_ms: u64,
    /// Reconnection count
    pub reconnect_count: usize,
    /// Time connected
    pub connected_duration: std::time::Duration,
}

/// Internal transport state
struct TransportState {
    /// Current connection state
    connection_state: ConnectionState,
    /// Current connection ID
    connection_id: Option<Uuid>,
    /// Reconnection attempt count
    reconnect_attempts: usize,
    /// Last heartbeat sent
    last_heartbeat_sent: Option<DateTime<Utc>>,
    /// Last heartbeat received
    last_heartbeat_received: Option<DateTime<Utc>>,
    /// Connection established time
    connected_at: Option<DateTime<Utc>>,
    /// Event callbacks
    callbacks: Vec<TransportCallback>,
    /// Connection metrics
    metrics: ConnectionMetrics,
}

/// Abstract transport trait
#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    /// Connect to the server
    async fn connect(&self) -> Result<()>;

    /// Disconnect from the server
    async fn disconnect(&self) -> Result<()>;

    /// Send a message
    async fn send(&self, message: CollaborationMessage) -> Result<()>;

    /// Receive a message (blocking)
    async fn receive(&self) -> Result<Option<CollaborationMessage>>;

    /// Get current connection state
    fn state(&self) -> ConnectionState;

    /// Check if connected
    fn is_connected(&self) -> bool {
        self.state() == ConnectionState::Connected
    }

    /// Register event callback
    fn on_event(&self, callback: TransportCallback);

    /// Get connection metrics
    fn metrics(&self) -> ConnectionMetrics;
}

/// WebSocket transport implementation
pub struct WebSocketTransport {
    config: Arc<TransportConfig>,
    state: Arc<RwLock<TransportState>>,
    tx: mpsc::UnboundedSender<CollaborationMessage>,
    rx: Arc<Mutex<mpsc::UnboundedReceiver<CollaborationMessage>>>,
}

impl WebSocketTransport {
    /// Create a new WebSocket transport
    pub fn new(config: TransportConfig) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        let state = TransportState {
            connection_state: ConnectionState::Disconnected,
            connection_id: None,
            reconnect_attempts: 0,
            last_heartbeat_sent: None,
            last_heartbeat_received: None,
            connected_at: None,
            callbacks: Vec::new(),
            metrics: ConnectionMetrics::default(),
        };

        Self {
            config: Arc::new(config),
            state: Arc::new(RwLock::new(state)),
            tx,
            rx: Arc::new(Mutex::new(rx)),
        }
    }

    /// Start heartbeat task
    async fn start_heartbeat_task(&self) -> Result<()> {
        let state = self.state.clone();
        let tx = self.tx.clone();
        let interval = self.config.heartbeat_interval;
        let timeout = self.config.heartbeat_timeout;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(std::time::Duration::from_secs(interval));

            loop {
                interval_timer.tick().await;

                let should_send = {
                    let state = state.read();
                    state.connection_state == ConnectionState::Connected
                };

                if should_send {
                    let timestamp = Utc::now().timestamp();
                    let heartbeat = CollaborationMessage::Heartbeat { timestamp };

                    if tx.send(heartbeat).is_ok() {
                        let mut state = state.write();
                        state.last_heartbeat_sent = Some(Utc::now());

                        // Emit event
                        Self::emit_event_static(&state.callbacks, TransportEvent::HeartbeatSent { timestamp });

                        // Check for timeout
                        if let Some(last_received) = state.last_heartbeat_received {
                            let elapsed = Utc::now().signed_duration_since(last_received);
                            if elapsed > Duration::seconds(timeout as i64) {
                                Self::emit_event_static(&state.callbacks, TransportEvent::HeartbeatTimeout);
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Handle reconnection
    async fn handle_reconnect(&self) -> Result<()> {
        let (attempt, delay) = {
            let mut state = self.state.write();
            state.connection_state = ConnectionState::Reconnecting;
            state.reconnect_attempts += 1;
            let attempt = state.reconnect_attempts;

            let delay = self.config.reconnect_strategy.calculate_delay(attempt);

            Self::emit_event_static(&state.callbacks, TransportEvent::Reconnecting { attempt });

            (attempt, delay)
        };

        if let Some(delay) = delay {
            tokio::time::sleep(delay).await;

            match self.connect().await {
                Ok(_) => {
                    let mut state = self.state.write();
                    state.reconnect_attempts = 0;
                    state.metrics.reconnect_count += 1;

                    if let Some(conn_id) = state.connection_id {
                        Self::emit_event_static(&state.callbacks, TransportEvent::Reconnected {
                            connection_id: conn_id,
                        });
                    }

                    Ok(())
                }
                Err(e) => {
                    let state = self.state.read();
                    Self::emit_event_static(&state.callbacks, TransportEvent::Error {
                        error: e.to_string(),
                    });
                    Err(e)
                }
            }
        } else {
            let state = self.state.read();
            Self::emit_event_static(&state.callbacks, TransportEvent::ReconnectFailed {
                attempts: attempt,
            });

            Err(CollaborationError::Connection(format!(
                "Reconnection failed after {} attempts",
                attempt
            )))
        }
    }

    /// Emit event to all callbacks (static version)
    fn emit_event_static(callbacks: &[TransportCallback], event: TransportEvent) {
        for callback in callbacks {
            callback(event.clone());
        }
    }

    /// Emit event to all callbacks
    fn emit_event(&self, event: TransportEvent) {
        let state = self.state.read();
        Self::emit_event_static(&state.callbacks, event);
    }

    /// Update metrics
    fn update_metrics(&self, sent: bool, bytes: u64) {
        let mut state = self.state.write();
        if sent {
            state.metrics.messages_sent += 1;
            state.metrics.bytes_sent += bytes;
        } else {
            state.metrics.messages_received += 1;
            state.metrics.bytes_received += bytes;
        }
    }
}

#[async_trait::async_trait]
impl Transport for WebSocketTransport {
    async fn connect(&self) -> Result<()> {
        {
            let mut state = self.state.write();
            state.connection_state = ConnectionState::Connecting;
            state.connection_id = Some(Uuid::new_v4());
        }

        // Simulate connection (in real implementation, use tokio-tungstenite)
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        {
            let mut state = self.state.write();
            state.connection_state = ConnectionState::Connected;
            state.connected_at = Some(Utc::now());

            if let Some(conn_id) = state.connection_id {
                Self::emit_event_static(&state.callbacks, TransportEvent::Connected {
                    connection_id: conn_id,
                });
            }
        }

        // Start heartbeat
        self.start_heartbeat_task().await?;

        Ok(())
    }

    async fn disconnect(&self) -> Result<()> {
        {
            let mut state = self.state.write();
            state.connection_state = ConnectionState::Closed;

            Self::emit_event_static(&state.callbacks, TransportEvent::Disconnected {
                reason: "User requested disconnect".to_string(),
            });
        }

        Ok(())
    }

    async fn send(&self, message: CollaborationMessage) -> Result<()> {
        // Check if connected
        if !self.is_connected() {
            return Err(CollaborationError::Connection(
                "Not connected".to_string(),
            ));
        }

        // Serialize message
        let data = MessageCodec::serialize(&message)
            .map_err(|e| CollaborationError::Serialization(e.to_string()))?;

        // Update metrics
        self.update_metrics(true, data.len() as u64);

        // In real implementation, send via WebSocket
        // For now, just emit event
        let message_id = Uuid::new_v4();
        self.emit_event(TransportEvent::MessageSent { message_id });

        Ok(())
    }

    async fn receive(&self) -> Result<Option<CollaborationMessage>> {
        let mut rx = self.rx.lock().await;

        match rx.recv().await {
            Some(message) => {
                // Update metrics (approximate size)
                self.update_metrics(false, 100);

                self.emit_event(TransportEvent::MessageReceived {
                    message: message.clone(),
                });

                Ok(Some(message))
            }
            None => Ok(None),
        }
    }

    fn state(&self) -> ConnectionState {
        let state = self.state.read();
        state.connection_state
    }

    fn on_event(&self, callback: TransportCallback) {
        let mut state = self.state.write();
        state.callbacks.push(callback);
    }

    fn metrics(&self) -> ConnectionMetrics {
        let state = self.state.read();
        let mut metrics = state.metrics.clone();

        // Calculate connected duration
        if let Some(connected_at) = state.connected_at {
            if state.connection_state == ConnectionState::Connected {
                let duration = Utc::now().signed_duration_since(connected_at);
                metrics.connected_duration = duration.to_std().unwrap_or_default();
            }
        }

        metrics
    }
}

/// Transport factory for creating transports
pub struct TransportFactory;

impl TransportFactory {
    /// Create a WebSocket transport
    pub fn create_websocket(config: TransportConfig) -> Arc<dyn Transport> {
        Arc::new(WebSocketTransport::new(config))
    }

    /// Create a WebSocket transport with default config
    pub fn create_default_websocket(server_url: String) -> Arc<dyn Transport> {
        let config = TransportConfig {
            server_url,
            ..Default::default()
        };
        Self::create_websocket(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reconnect_strategy_fixed_delay() {
        let strategy = ReconnectStrategy::FixedDelay {
            delay_ms: 1000,
            max_attempts: Some(3),
        };

        assert_eq!(strategy.calculate_delay(0), Some(std::time::Duration::from_millis(1000)));
        assert_eq!(strategy.calculate_delay(1), Some(std::time::Duration::from_millis(1000)));
        assert_eq!(strategy.calculate_delay(2), Some(std::time::Duration::from_millis(1000)));
        assert_eq!(strategy.calculate_delay(3), None);
    }

    #[test]
    fn test_reconnect_strategy_exponential_backoff() {
        let strategy = ReconnectStrategy::ExponentialBackoff {
            initial_delay_ms: 1000,
            max_delay_ms: 10000,
            multiplier: 2.0,
            max_attempts: None,
        };

        assert_eq!(strategy.calculate_delay(0), Some(std::time::Duration::from_millis(1000)));
        assert_eq!(strategy.calculate_delay(1), Some(std::time::Duration::from_millis(2000)));
        assert_eq!(strategy.calculate_delay(2), Some(std::time::Duration::from_millis(4000)));
        assert_eq!(strategy.calculate_delay(3), Some(std::time::Duration::from_millis(8000)));
        // Should cap at max_delay_ms
        assert_eq!(strategy.calculate_delay(4), Some(std::time::Duration::from_millis(10000)));
    }

    #[tokio::test]
    async fn test_transport_lifecycle() {
        let config = TransportConfig::default();
        let transport = WebSocketTransport::new(config);

        assert_eq!(transport.state(), ConnectionState::Disconnected);

        transport.connect().await.unwrap();
        assert_eq!(transport.state(), ConnectionState::Connected);

        transport.disconnect().await.unwrap();
        assert_eq!(transport.state(), ConnectionState::Closed);
    }

    #[tokio::test]
    async fn test_transport_metrics() {
        let config = TransportConfig::default();
        let transport = WebSocketTransport::new(config);

        transport.connect().await.unwrap();

        let message = CollaborationMessage::Heartbeat {
            timestamp: Utc::now().timestamp(),
        };

        transport.send(message).await.unwrap();

        let metrics = transport.metrics();
        assert_eq!(metrics.messages_sent, 1);
        assert!(metrics.bytes_sent > 0);
    }
}
