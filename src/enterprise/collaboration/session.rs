//! Collaboration Session Management
//!
//! This module provides the core session management functionality for real-time
//! collaboration, including participant management, session lifecycle, and persistence.

use super::{CollaborationError, CollaborationEvent, EventCallback, Result};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Session identifier type
pub type SessionId = Uuid;

/// Session state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    /// Session is being created
    Creating,
    /// Session is active and accepting participants
    Active,
    /// Session is paused (no operations accepted)
    Paused,
    /// Session is being archived
    Archiving,
    /// Session is closed
    Closed,
}

/// Configuration for a collaboration session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Human-readable session name
    pub session_name: String,
    /// Maximum number of participants allowed
    pub max_participants: usize,
    /// Whether to persist session data
    pub persistence_enabled: bool,
    /// Session timeout in seconds (0 = no timeout)
    pub session_timeout: u64,
    /// Whether to allow anonymous participants
    pub allow_anonymous: bool,
    /// Project ID this session is associated with
    pub project_id: Option<Uuid>,
    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            session_name: "Untitled Session".to_string(),
            max_participants: 10,
            persistence_enabled: true,
            session_timeout: 0,
            allow_anonymous: false,
            project_id: None,
            metadata: HashMap::new(),
        }
    }
}

/// Information about a participant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantInfo {
    /// Unique participant ID
    pub user_id: Uuid,
    /// Display name
    pub username: String,
    /// Email (optional)
    pub email: Option<String>,
    /// Avatar URL (optional)
    pub avatar_url: Option<String>,
    /// When the participant joined
    pub joined_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// Whether participant is currently connected
    pub is_connected: bool,
    /// Participant color for UI display
    pub color: String,
    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

/// Internal participant state
#[derive(Debug)]
pub struct Participant {
    /// Participant information
    pub info: ParticipantInfo,
    /// Number of active operations
    pub active_operations: usize,
    /// Total operations contributed
    pub total_operations: usize,
}

impl Participant {
    /// Create a new participant
    fn new(user_id: Uuid, username: String) -> Self {
        let now = Utc::now();
        Self {
            info: ParticipantInfo {
                user_id,
                username,
                email: None,
                avatar_url: None,
                joined_at: now,
                last_activity: now,
                is_connected: true,
                color: Self::generate_color(user_id),
                metadata: HashMap::new(),
            },
            active_operations: 0,
            total_operations: 0,
        }
    }

    /// Generate a unique color for a participant based on their ID
    fn generate_color(user_id: Uuid) -> String {
        let bytes = user_id.as_bytes();
        let hue = (bytes[0] as u16 * 360) / 255;
        format!("hsl({}, 70%, 50%)", hue)
    }

    /// Update last activity timestamp
    fn update_activity(&mut self) {
        self.info.last_activity = Utc::now();
    }
}

/// Session metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetrics {
    /// Total number of participants (current and past)
    pub total_participants: usize,
    /// Current active participants
    pub active_participants: usize,
    /// Total operations performed
    pub total_operations: u64,
    /// Session uptime in seconds
    pub uptime_seconds: u64,
    /// Average operations per minute
    pub operations_per_minute: f64,
    /// Peak concurrent participants
    pub peak_participants: usize,
}

/// Session storage trait for persistence
#[async_trait::async_trait]
pub trait SessionStorage: Send + Sync {
    /// Save session state
    async fn save_session(&self, session: &SessionData) -> Result<()>;

    /// Load session state
    async fn load_session(&self, session_id: SessionId) -> Result<Option<SessionData>>;

    /// Delete session
    async fn delete_session(&self, session_id: SessionId) -> Result<()>;

    /// List all active sessions
    async fn list_sessions(&self) -> Result<Vec<SessionId>>;
}

/// Serializable session data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub id: SessionId,
    pub config: SessionConfig,
    pub state: SessionState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub participants: Vec<ParticipantInfo>,
}

/// Internal session state
struct SessionInner {
    /// Session metadata
    data: SessionData,
    /// Active participants
    participants: HashMap<Uuid, Participant>,
    /// Session metrics
    metrics: SessionMetrics,
    /// Event callbacks
    event_callbacks: Vec<EventCallback>,
}

/// Collaboration session manager
#[derive(Clone)]
pub struct CollaborationSession {
    inner: Arc<RwLock<SessionInner>>,
    storage: Option<Arc<dyn SessionStorage>>,
}

impl CollaborationSession {
    /// Create a new collaboration session
    pub async fn create(config: SessionConfig) -> Result<Self> {
        Self::create_with_storage(config, None).await
    }

    /// Create a new collaboration session with storage backend
    pub async fn create_with_storage(
        config: SessionConfig,
        storage: Option<Arc<dyn SessionStorage>>,
    ) -> Result<Self> {
        let now = Utc::now();
        let session_id = Uuid::new_v4();

        let data = SessionData {
            id: session_id,
            config,
            state: SessionState::Creating,
            created_at: now,
            updated_at: now,
            participants: Vec::new(),
        };

        let inner = SessionInner {
            data,
            participants: HashMap::new(),
            metrics: SessionMetrics {
                total_participants: 0,
                active_participants: 0,
                total_operations: 0,
                uptime_seconds: 0,
                operations_per_minute: 0.0,
                peak_participants: 0,
            },
            event_callbacks: Vec::new(),
        };

        let session = Self {
            inner: Arc::new(RwLock::new(inner)),
            storage,
        };

        // Transition to active state
        session.set_state(SessionState::Active).await?;

        // Persist if storage is available
        if let Some(storage) = &session.storage {
            storage.save_session(&session.get_data()).await?;
        }

        Ok(session)
    }

    /// Load an existing session from storage
    pub async fn load(
        session_id: SessionId,
        storage: Arc<dyn SessionStorage>,
    ) -> Result<Option<Self>> {
        if let Some(data) = storage.load_session(session_id).await? {
            let mut participants = HashMap::new();

            // Restore participants
            for info in &data.participants {
                let participant = Participant {
                    info: info.clone(),
                    active_operations: 0,
                    total_operations: 0,
                };
                participants.insert(info.user_id, participant);
            }

            let inner = SessionInner {
                data,
                participants,
                metrics: SessionMetrics {
                    total_participants: 0,
                    active_participants: 0,
                    total_operations: 0,
                    uptime_seconds: 0,
                    operations_per_minute: 0.0,
                    peak_participants: 0,
                },
                event_callbacks: Vec::new(),
            };

            Ok(Some(Self {
                inner: Arc::new(RwLock::new(inner)),
                storage: Some(storage),
            }))
        } else {
            Ok(None)
        }
    }

    /// Get session ID
    pub fn id(&self) -> SessionId {
        self.inner.read().data.id
    }

    /// Get session state
    pub fn state(&self) -> SessionState {
        self.inner.read().data.state
    }

    /// Set session state
    pub async fn set_state(&self, new_state: SessionState) -> Result<()> {
        let (old_state, session_id) = {
            let mut inner = self.inner.write();
            let old_state = inner.data.state;
            inner.data.state = new_state;
            inner.data.updated_at = Utc::now();
            (old_state, inner.data.id)
        };

        // Emit event
        self.emit_event(CollaborationEvent::SessionStateChanged {
            session_id,
            old_state,
            new_state,
        });

        // Persist if storage is available
        if let Some(storage) = &self.storage {
            storage.save_session(&self.get_data()).await?;
        }

        Ok(())
    }

    /// Add a participant to the session
    pub async fn join(&self, user_id: Uuid, username: String) -> Result<ParticipantInfo> {
        let (participant_info, session_id) = {
            let mut inner = self.inner.write();

            // Check session state
            if inner.data.state != SessionState::Active {
                return Err(CollaborationError::InvalidState(
                    "Session is not active".to_string(),
                ));
            }

            // Check if session is full
            if inner.participants.len() >= inner.data.config.max_participants {
                return Err(CollaborationError::SessionFull(
                    inner.data.config.max_participants,
                ));
            }

            // Check if participant already exists
            if let Some(participant) = inner.participants.get_mut(&user_id) {
                participant.info.is_connected = true;
                participant.update_activity();
                return Ok(participant.info.clone());
            }

            // Create new participant
            let participant = Participant::new(user_id, username.clone());
            let info = participant.info.clone();

            inner.participants.insert(user_id, participant);
            inner.metrics.total_participants += 1;
            inner.metrics.active_participants = inner.participants.len();

            if inner.metrics.active_participants > inner.metrics.peak_participants {
                inner.metrics.peak_participants = inner.metrics.active_participants;
            }

            inner.data.updated_at = Utc::now();

            (info, inner.data.id)
        };

        // Emit event
        self.emit_event(CollaborationEvent::UserJoined {
            session_id,
            user_id,
            username,
        });

        // Persist if storage is available
        if let Some(storage) = &self.storage {
            storage.save_session(&self.get_data()).await?;
        }

        Ok(participant_info)
    }

    /// Remove a participant from the session
    pub async fn leave(&self, user_id: Uuid) -> Result<()> {
        let session_id = {
            let mut inner = self.inner.write();

            if let Some(participant) = inner.participants.get_mut(&user_id) {
                participant.info.is_connected = false;
                participant.update_activity();
                inner.metrics.active_participants =
                    inner.participants.values().filter(|p| p.info.is_connected).count();
                inner.data.updated_at = Utc::now();
                inner.data.id
            } else {
                return Err(CollaborationError::ParticipantNotFound(user_id));
            }
        };

        // Emit event
        self.emit_event(CollaborationEvent::UserLeft {
            session_id,
            user_id,
        });

        // Persist if storage is available
        if let Some(storage) = &self.storage {
            storage.save_session(&self.get_data()).await?;
        }

        Ok(())
    }

    /// Get participant information
    pub fn get_participant(&self, user_id: Uuid) -> Result<ParticipantInfo> {
        let inner = self.inner.read();
        inner
            .participants
            .get(&user_id)
            .map(|p| p.info.clone())
            .ok_or(CollaborationError::ParticipantNotFound(user_id))
    }

    /// Get all participants
    pub fn get_participants(&self) -> Vec<ParticipantInfo> {
        let inner = self.inner.read();
        inner.participants.values().map(|p| p.info.clone()).collect()
    }

    /// Get active participants (currently connected)
    pub fn get_active_participants(&self) -> Vec<ParticipantInfo> {
        let inner = self.inner.read();
        inner
            .participants
            .values()
            .filter(|p| p.info.is_connected)
            .map(|p| p.info.clone())
            .collect()
    }

    /// Update participant metadata
    pub async fn update_participant_metadata(
        &self,
        user_id: Uuid,
        key: String,
        value: String,
    ) -> Result<()> {
        {
            let mut inner = self.inner.write();
            let participant = inner
                .participants
                .get_mut(&user_id)
                .ok_or(CollaborationError::ParticipantNotFound(user_id))?;

            participant.info.metadata.insert(key, value);
            participant.update_activity();
            inner.data.updated_at = Utc::now();
        }

        // Persist if storage is available
        if let Some(storage) = &self.storage {
            storage.save_session(&self.get_data()).await?;
        }

        Ok(())
    }

    /// Record an operation by a participant
    pub fn record_operation(&self, user_id: Uuid) -> Result<()> {
        let mut inner = self.inner.write();
        let participant = inner
            .participants
            .get_mut(&user_id)
            .ok_or(CollaborationError::ParticipantNotFound(user_id))?;

        participant.active_operations += 1;
        participant.total_operations += 1;
        participant.update_activity();
        inner.metrics.total_operations += 1;

        Ok(())
    }

    /// Get session metrics
    pub fn get_metrics(&self) -> SessionMetrics {
        let inner = self.inner.read();
        let mut metrics = inner.metrics.clone();

        // Calculate uptime
        let uptime = Utc::now()
            .signed_duration_since(inner.data.created_at)
            .num_seconds() as u64;
        metrics.uptime_seconds = uptime;

        // Calculate operations per minute
        if uptime > 0 {
            metrics.operations_per_minute =
                (metrics.total_operations as f64 / uptime as f64) * 60.0;
        }

        metrics
    }

    /// End the session
    pub async fn end(&self) -> Result<()> {
        self.set_state(SessionState::Closed).await?;

        // Persist final state
        if let Some(storage) = &self.storage {
            storage.save_session(&self.get_data()).await?;
        }

        Ok(())
    }

    /// Archive the session
    pub async fn archive(&self) -> Result<()> {
        self.set_state(SessionState::Archiving).await?;

        // Persist and optionally move to archive storage
        if let Some(storage) = &self.storage {
            storage.save_session(&self.get_data()).await?;
        }

        self.set_state(SessionState::Closed).await?;

        Ok(())
    }

    /// Register an event callback
    pub fn on_event(&self, callback: EventCallback) {
        let mut inner = self.inner.write();
        inner.event_callbacks.push(callback);
    }

    /// Emit an event to all registered callbacks
    fn emit_event(&self, event: CollaborationEvent) {
        let inner = self.inner.read();
        for callback in &inner.event_callbacks {
            callback(event.clone());
        }
    }

    /// Get session data for serialization
    fn get_data(&self) -> SessionData {
        let inner = self.inner.read();
        let mut data = inner.data.clone();
        data.participants = inner.participants.values().map(|p| p.info.clone()).collect();
        data
    }

    /// Check if a participant is in the session
    pub fn has_participant(&self, user_id: Uuid) -> bool {
        let inner = self.inner.read();
        inner.participants.contains_key(&user_id)
    }

    /// Get session configuration
    pub fn config(&self) -> SessionConfig {
        let inner = self.inner.read();
        inner.data.config.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_creation() {
        let config = SessionConfig {
            session_name: "Test Session".to_string(),
            max_participants: 5,
            ..Default::default()
        };

        let session = CollaborationSession::create(config).await.unwrap();
        assert_eq!(session.state(), SessionState::Active);
        assert_eq!(session.get_participants().len(), 0);
    }

    #[tokio::test]
    async fn test_join_leave() {
        let config = SessionConfig::default();
        let session = CollaborationSession::create(config).await.unwrap();

        let user_id = Uuid::new_v4();
        let info = session.join(user_id, "Alice".to_string()).await.unwrap();

        assert_eq!(info.username, "Alice");
        assert_eq!(session.get_participants().len(), 1);

        session.leave(user_id).await.unwrap();
        assert_eq!(session.get_active_participants().len(), 0);
    }

    #[tokio::test]
    async fn test_max_participants() {
        let config = SessionConfig {
            max_participants: 2,
            ..Default::default()
        };
        let session = CollaborationSession::create(config).await.unwrap();

        session.join(Uuid::new_v4(), "Alice".to_string()).await.unwrap();
        session.join(Uuid::new_v4(), "Bob".to_string()).await.unwrap();

        let result = session.join(Uuid::new_v4(), "Charlie".to_string()).await;
        assert!(matches!(result, Err(CollaborationError::SessionFull(_))));
    }
}
