//! GraphQL Subscriptions
//!
//! WebSocket-based real-time subscriptions with connection management,
//! filtering, transformations, and lifecycle management.

use super::query::{Document, ExecutionResult, GraphQLError};
use super::schema::{ResolverContext, Value};
use async_trait::async_trait;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::{broadcast, mpsc, RwLock};
use uuid::Uuid;

// ============================================================================
// Error Types
// ============================================================================

/// Subscription errors
#[derive(Error, Debug, Clone)]
pub enum SubscriptionError {
    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// Invalid subscription
    #[error("Invalid subscription: {0}")]
    InvalidSubscription(String),

    /// Subscription not found
    #[error("Subscription not found: {0}")]
    NotFound(String),

    /// Transport error
    #[error("Transport error: {0}")]
    TransportError(String),

    /// Authentication required
    #[error("Authentication required")]
    Unauthenticated,

    /// Authorization failed
    #[error("Not authorized: {0}")]
    Unauthorized(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

pub type SubscriptionResult<T> = Result<T, SubscriptionError>;

// ============================================================================
// WebSocket Message Protocol
// ============================================================================

/// WebSocket message types (GraphQL over WebSocket Protocol)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    /// Connection initialization
    ConnectionInit {
        /// Connection parameters (auth tokens, etc.)
        payload: Option<HashMap<String, Value>>,
    },

    /// Connection acknowledgement
    ConnectionAck,

    /// Connection keep-alive
    ConnectionKeepAlive,

    /// Terminate connection
    ConnectionTerminate,

    /// Start a subscription
    Start {
        /// Subscription ID
        id: String,
        /// GraphQL query/subscription
        payload: SubscriptionPayload,
    },

    /// Stop a subscription
    Stop {
        /// Subscription ID
        id: String,
    },

    /// Subscription data
    Data {
        /// Subscription ID
        id: String,
        /// Execution result
        payload: ExecutionResult,
    },

    /// Subscription error
    Error {
        /// Subscription ID
        id: String,
        /// Error payload
        payload: GraphQLError,
    },

    /// Subscription complete
    Complete {
        /// Subscription ID
        id: String,
    },
}

/// Subscription payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionPayload {
    /// GraphQL query/subscription
    pub query: String,
    /// Variables
    pub variables: Option<HashMap<String, Value>>,
    /// Operation name
    #[serde(rename = "operationName")]
    pub operation_name: Option<String>,
}

// ============================================================================
// Subscription Stream
// ============================================================================

/// Event emitted by a subscription
#[derive(Debug, Clone)]
pub struct SubscriptionEvent {
    /// Event data
    pub data: Value,
    /// Event metadata
    pub metadata: HashMap<String, Value>,
    /// Timestamp
    pub timestamp: Instant,
}

impl SubscriptionEvent {
    /// Create a new event
    pub fn new(data: Value) -> Self {
        Self {
            data,
            metadata: HashMap::new(),
            timestamp: Instant::now(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

/// Subscription event stream
pub type SubscriptionStream = Pin<Box<dyn Stream<Item = SubscriptionEvent> + Send>>;

/// Trait for creating subscription streams
#[async_trait]
pub trait SubscriptionSource: Send + Sync {
    /// Create a subscription stream
    async fn subscribe(
        &self,
        ctx: &ResolverContext,
        args: &HashMap<String, Value>,
    ) -> SubscriptionResult<SubscriptionStream>;
}

// ============================================================================
// Event Bus for Broadcasting
// ============================================================================

/// Topic-based event bus for subscriptions
pub struct EventBus {
    /// Broadcast channels by topic
    channels: Arc<RwLock<HashMap<String, broadcast::Sender<SubscriptionEvent>>>>,
    /// Channel capacity
    capacity: usize,
}

impl EventBus {
    /// Create a new event bus
    pub fn new(capacity: usize) -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            capacity,
        }
    }

    /// Publish an event to a topic
    pub async fn publish(&self, topic: impl Into<String>, event: SubscriptionEvent) {
        let topic = topic.into();
        let channels = self.channels.read().await;

        if let Some(tx) = channels.get(&topic) {
            // Ignore if no subscribers
            let _ = tx.send(event);
        }
    }

    /// Subscribe to a topic
    pub async fn subscribe(&self, topic: impl Into<String>) -> broadcast::Receiver<SubscriptionEvent> {
        let topic = topic.into();
        let mut channels = self.channels.write().await;

        let tx = channels.entry(topic).or_insert_with(|| {
            let (tx, _) = broadcast::channel(self.capacity);
            tx
        });

        tx.subscribe()
    }

    /// Unsubscribe from a topic (clean up if no more subscribers)
    pub async fn cleanup_topic(&self, topic: &str) {
        let mut channels = self.channels.write().await;
        if let Some(tx) = channels.get(topic) {
            if tx.receiver_count() == 0 {
                channels.remove(topic);
            }
        }
    }

    /// Get number of active topics
    pub async fn topic_count(&self) -> usize {
        self.channels.read().await.len()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(100)
    }
}

// ============================================================================
// Connection Management
// ============================================================================

/// WebSocket connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Connection not initialized
    NotInitialized,
    /// Connection initialized and ready
    Ready,
    /// Connection closing
    Closing,
    /// Connection closed
    Closed,
}

/// Active subscription
struct ActiveSubscription {
    /// Subscription ID
    id: String,
    /// Document
    document: Document,
    /// Variables
    variables: HashMap<String, Value>,
    /// Resolver context
    context: ResolverContext,
    /// Cancel sender
    cancel_tx: mpsc::Sender<()>,
}

/// WebSocket connection
pub struct WsConnection {
    /// Connection ID
    id: String,
    /// Connection state
    state: RwLock<ConnectionState>,
    /// Active subscriptions
    subscriptions: RwLock<HashMap<String, ActiveSubscription>>,
    /// User ID (if authenticated)
    user_id: RwLock<Option<String>>,
    /// Connection metadata
    metadata: RwLock<HashMap<String, Value>>,
    /// Created timestamp
    created_at: Instant,
    /// Last activity timestamp
    last_activity: RwLock<Instant>,
}

impl WsConnection {
    /// Create a new connection
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            state: RwLock::new(ConnectionState::NotInitialized),
            subscriptions: RwLock::new(HashMap::new()),
            user_id: RwLock::new(None),
            metadata: RwLock::new(HashMap::new()),
            created_at: Instant::now(),
            last_activity: RwLock::new(Instant::now()),
        }
    }

    /// Get connection ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get connection state
    pub async fn state(&self) -> ConnectionState {
        *self.state.read().await
    }

    /// Set connection state
    pub async fn set_state(&self, state: ConnectionState) {
        *self.state.write().await = state;
        self.touch().await;
    }

    /// Set user ID
    pub async fn set_user_id(&self, user_id: Option<String>) {
        *self.user_id.write().await = user_id;
    }

    /// Get user ID
    pub async fn user_id(&self) -> Option<String> {
        self.user_id.read().await.clone()
    }

    /// Update last activity timestamp
    pub async fn touch(&self) {
        *self.last_activity.write().await = Instant::now();
    }

    /// Get time since last activity
    pub async fn idle_duration(&self) -> Duration {
        self.last_activity.read().await.elapsed()
    }

    /// Get connection age
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Add a subscription
    pub async fn add_subscription(
        &self,
        id: String,
        document: Document,
        variables: HashMap<String, Value>,
        context: ResolverContext,
        cancel_tx: mpsc::Sender<()>,
    ) {
        {
            let mut subs = self.subscriptions.write().await;
            subs.insert(
                id.clone(),
                ActiveSubscription {
                    id,
                    document,
                    variables,
                    context,
                    cancel_tx,
                },
            );
        }
        self.touch().await;
    }

    /// Remove a subscription
    pub async fn remove_subscription(&self, id: &str) -> Option<ActiveSubscription> {
        let sub = {
            let mut subs = self.subscriptions.write().await;
            subs.remove(id)
        };
        self.touch().await;
        sub
    }

    /// Get subscription count
    pub async fn subscription_count(&self) -> usize {
        self.subscriptions.read().await.len()
    }

    /// Close all subscriptions
    pub async fn close_all_subscriptions(&self) {
        let mut subs = self.subscriptions.write().await;
        for (_, sub) in subs.drain() {
            let _ = sub.cancel_tx.send(()).await;
        }
    }
}

impl Default for WsConnection {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Connection Manager
// ============================================================================

/// Connection manager for all WebSocket connections
pub struct ConnectionManager {
    /// Active connections
    connections: RwLock<HashMap<String, Arc<WsConnection>>>,
    /// Maximum connections per user
    max_connections_per_user: usize,
    /// Maximum idle time before disconnect
    max_idle_duration: Duration,
    /// Maximum connection age
    max_connection_age: Duration,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new() -> Self {
        Self {
            connections: RwLock::new(HashMap::new()),
            max_connections_per_user: 10,
            max_idle_duration: Duration::from_secs(300), // 5 minutes
            max_connection_age: Duration::from_secs(3600 * 24), // 24 hours
        }
    }

    /// Create a new connection
    pub async fn create_connection(&self) -> SubscriptionResult<Arc<WsConnection>> {
        let conn = Arc::new(WsConnection::new());
        let mut conns = self.connections.write().await;
        conns.insert(conn.id().to_string(), Arc::clone(&conn));
        Ok(conn)
    }

    /// Get a connection by ID
    pub async fn get_connection(&self, id: &str) -> Option<Arc<WsConnection>> {
        let conns = self.connections.read().await;
        conns.get(id).cloned()
    }

    /// Remove a connection
    pub async fn remove_connection(&self, id: &str) -> Option<Arc<WsConnection>> {
        let mut conns = self.connections.write().await;
        conns.remove(id)
    }

    /// Get total connection count
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// Get connections for a user
    pub async fn user_connections(&self, user_id: &str) -> Vec<Arc<WsConnection>> {
        // Clone all connections first to avoid holding lock across await
        let conns = {
            let guard = self.connections.read().await;
            guard.values().cloned().collect::<Vec<_>>()
        };

        let mut user_conns = Vec::new();
        for conn in conns {
            if let Some(uid) = conn.user_id().await {
                if uid == user_id {
                    user_conns.push(conn);
                }
            }
        }

        user_conns
    }

    /// Cleanup idle and old connections
    pub async fn cleanup(&self) {
        // Clone all connections first to avoid holding lock across await
        let conns = {
            let guard = self.connections.read().await;
            guard.iter().map(|(id, conn)| (id.clone(), Arc::clone(conn))).collect::<Vec<_>>()
        };

        let mut to_remove = Vec::new();
        for (id, conn) in conns {
            let idle = conn.idle_duration().await;
            let age = conn.age();

            if idle > self.max_idle_duration || age > self.max_connection_age {
                to_remove.push(id);
            }
        }

        // Remove stale connections
        for id in to_remove {
            if let Some(conn) = self.remove_connection(&id).await {
                conn.close_all_subscriptions().await;
                conn.set_state(ConnectionState::Closed).await;
            }
        }
    }

    /// Check user connection limit
    pub async fn check_user_limit(&self, user_id: &str) -> SubscriptionResult<()> {
        let user_conns = self.user_connections(user_id).await;
        if user_conns.len() >= self.max_connections_per_user {
            return Err(SubscriptionError::RateLimitExceeded);
        }
        Ok(())
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Subscription Filter
// ============================================================================

/// Filter for subscription events
#[async_trait]
pub trait SubscriptionFilter: Send + Sync {
    /// Check if event should be sent to subscriber
    async fn should_emit(&self, event: &SubscriptionEvent, ctx: &ResolverContext) -> bool;
}

/// Filter based on field value
pub struct FieldFilter {
    /// Field path to check
    field_path: Vec<String>,
    /// Expected value
    expected_value: Value,
}

impl FieldFilter {
    /// Create a new field filter
    pub fn new(field_path: Vec<String>, expected_value: Value) -> Self {
        Self {
            field_path,
            expected_value,
        }
    }
}

#[async_trait]
impl SubscriptionFilter for FieldFilter {
    async fn should_emit(&self, event: &SubscriptionEvent, _ctx: &ResolverContext) -> bool {
        // Navigate field path and compare value
        let mut current = &event.data;

        for field in &self.field_path {
            match current {
                Value::Object(obj) => {
                    if let Some(value) = obj.get(field) {
                        current = value;
                    } else {
                        return false;
                    }
                }
                _ => return false,
            }
        }

        current == &self.expected_value
    }
}

// ============================================================================
// Subscription Manager
// ============================================================================

/// Subscription manager
pub struct SubscriptionManager {
    /// Connection manager
    connections: Arc<ConnectionManager>,
    /// Event bus
    event_bus: Arc<EventBus>,
}

impl SubscriptionManager {
    /// Create a new subscription manager
    pub fn new() -> Self {
        Self {
            connections: Arc::new(ConnectionManager::new()),
            event_bus: Arc::new(EventBus::new(100)),
        }
    }

    /// Get connection manager
    pub fn connections(&self) -> &Arc<ConnectionManager> {
        &self.connections
    }

    /// Get event bus
    pub fn event_bus(&self) -> &Arc<EventBus> {
        &self.event_bus
    }

    /// Start cleanup task
    pub fn start_cleanup_task(&self) {
        let connections = Arc::clone(&self.connections);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(60)).await;
                connections.cleanup().await;
            }
        });
    }
}

impl Default for SubscriptionManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_creation() {
        let conn = WsConnection::new();
        assert!(!conn.id().is_empty());
        assert_eq!(conn.state().await, ConnectionState::NotInitialized);
    }

    #[tokio::test]
    async fn test_connection_state() {
        let conn = WsConnection::new();
        conn.set_state(ConnectionState::Ready).await;
        assert_eq!(conn.state().await, ConnectionState::Ready);
    }

    #[tokio::test]
    async fn test_connection_manager() {
        let manager = ConnectionManager::new();
        let conn = manager.create_connection().await.unwrap();
        assert_eq!(manager.connection_count().await, 1);

        let retrieved = manager.get_connection(conn.id()).await;
        assert!(retrieved.is_some());

        manager.remove_connection(conn.id()).await;
        assert_eq!(manager.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_event_bus() {
        let bus = EventBus::new(10);
        let mut rx = bus.subscribe("test_topic").await;

        let event = SubscriptionEvent::new(Value::String("test".to_string()));
        bus.publish("test_topic", event.clone()).await;

        let received = rx.recv().await.unwrap();
        assert_eq!(received.data, event.data);
    }

    #[tokio::test]
    async fn test_connection_idle() {
        let conn = WsConnection::new();
        tokio::time::sleep(Duration::from_millis(100)).await;

        let idle = conn.idle_duration().await;
        assert!(idle >= Duration::from_millis(100));

        conn.touch().await;
        let idle_after = conn.idle_duration().await;
        assert!(idle_after < idle);
    }

    #[tokio::test]
    async fn test_subscription_manager() {
        let manager = SubscriptionManager::new();
        let conn = manager.connections().create_connection().await.unwrap();
        assert_eq!(manager.connections().connection_count().await, 1);

        let event = SubscriptionEvent::new(Value::Int(42));
        manager.event_bus().publish("test", event).await;
        assert!(manager.event_bus().topic_count().await > 0);
    }
}
