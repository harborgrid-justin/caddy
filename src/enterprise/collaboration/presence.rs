//! User Presence Tracking
//!
//! This module provides real-time tracking of user presence, including cursor positions,
//! selections, and activity status for collaborative editing.

use super::{CollaborationError, Result};
use chrono::{DateTime, Duration, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Cursor position in 2D space
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CursorPosition {
    /// X coordinate
    pub x: f64,
    /// Y coordinate
    pub y: f64,
    /// Z coordinate (for 3D)
    pub z: Option<f64>,
    /// Viewport ID (for multi-viewport scenarios)
    pub viewport_id: Option<Uuid>,
}

impl CursorPosition {
    /// Create a new 2D cursor position
    pub fn new_2d(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            z: None,
            viewport_id: None,
        }
    }

    /// Create a new 3D cursor position
    pub fn new_3d(x: f64, y: f64, z: f64) -> Self {
        Self {
            x,
            y,
            z: Some(z),
            viewport_id: None,
        }
    }

    /// Set viewport ID
    pub fn with_viewport(mut self, viewport_id: Uuid) -> Self {
        self.viewport_id = Some(viewport_id);
        self
    }
}

/// Selection range in a document
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectionRange {
    /// Start entity ID
    pub start_entity: Option<Uuid>,
    /// End entity ID
    pub end_entity: Option<Uuid>,
    /// List of selected entity IDs
    pub selected_entities: Vec<Uuid>,
    /// Selection bounding box (min and max points)
    pub bounding_box: Option<(CursorPosition, CursorPosition)>,
}

impl SelectionRange {
    /// Create an empty selection
    pub fn empty() -> Self {
        Self {
            start_entity: None,
            end_entity: None,
            selected_entities: Vec::new(),
            bounding_box: None,
        }
    }

    /// Create a single entity selection
    pub fn single(entity_id: Uuid) -> Self {
        Self {
            start_entity: Some(entity_id),
            end_entity: Some(entity_id),
            selected_entities: vec![entity_id],
            bounding_box: None,
        }
    }

    /// Create a multi-entity selection
    pub fn multiple(entities: Vec<Uuid>) -> Self {
        Self {
            start_entity: entities.first().copied(),
            end_entity: entities.last().copied(),
            selected_entities: entities,
            bounding_box: None,
        }
    }

    /// Check if selection is empty
    pub fn is_empty(&self) -> bool {
        self.selected_entities.is_empty()
    }

    /// Get number of selected entities
    pub fn count(&self) -> usize {
        self.selected_entities.len()
    }
}

/// User activity status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityStatus {
    /// User is actively editing
    Active,
    /// User is viewing but not editing
    Idle,
    /// User is away from keyboard
    Away,
    /// User is offline
    Offline,
}

impl Default for ActivityStatus {
    fn default() -> Self {
        Self::Active
    }
}

/// Current viewport information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportInfo {
    /// Viewport ID
    pub id: Uuid,
    /// Camera position
    pub camera_position: (f64, f64, f64),
    /// Camera target/look-at point
    pub camera_target: (f64, f64, f64),
    /// Zoom level
    pub zoom: f64,
    /// Rotation (euler angles in degrees)
    pub rotation: (f64, f64, f64),
}

/// User presence information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPresence {
    /// User ID
    pub user_id: Uuid,
    /// Current cursor position
    pub cursor: Option<CursorPosition>,
    /// Current selection
    pub selection: SelectionRange,
    /// Activity status
    pub status: ActivityStatus,
    /// Current viewport
    pub viewport: Option<ViewportInfo>,
    /// Custom status message
    pub status_message: Option<String>,
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
    /// User color for UI display
    pub color: String,
}

impl UserPresence {
    /// Create a new user presence
    pub fn new(user_id: Uuid, color: String) -> Self {
        Self {
            user_id,
            cursor: None,
            selection: SelectionRange::empty(),
            status: ActivityStatus::Active,
            viewport: None,
            status_message: None,
            last_updated: Utc::now(),
            color,
        }
    }

    /// Update cursor position
    pub fn update_cursor(&mut self, position: CursorPosition) {
        self.cursor = Some(position);
        self.last_updated = Utc::now();
        self.status = ActivityStatus::Active;
    }

    /// Update selection
    pub fn update_selection(&mut self, selection: SelectionRange) {
        self.selection = selection;
        self.last_updated = Utc::now();
        self.status = ActivityStatus::Active;
    }

    /// Update activity status
    pub fn update_status(&mut self, status: ActivityStatus) {
        self.status = status;
        self.last_updated = Utc::now();
    }

    /// Check if presence is stale (not updated in a while)
    pub fn is_stale(&self, timeout: Duration) -> bool {
        Utc::now().signed_duration_since(self.last_updated) > timeout
    }
}

/// Presence update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PresenceUpdate {
    /// Cursor moved
    CursorMoved {
        user_id: Uuid,
        position: CursorPosition,
    },
    /// Selection changed
    SelectionChanged {
        user_id: Uuid,
        selection: SelectionRange,
    },
    /// Status changed
    StatusChanged {
        user_id: Uuid,
        status: ActivityStatus,
    },
    /// Viewport changed
    ViewportChanged {
        user_id: Uuid,
        viewport: ViewportInfo,
    },
    /// User left
    UserLeft { user_id: Uuid },
}

/// Compact presence info for efficient transmission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceInfo {
    pub user_id: Uuid,
    pub cursor: Option<CursorPosition>,
    pub selection_count: usize,
    pub status: ActivityStatus,
    pub color: String,
}

impl From<&UserPresence> for PresenceInfo {
    fn from(presence: &UserPresence) -> Self {
        Self {
            user_id: presence.user_id,
            cursor: presence.cursor,
            selection_count: presence.selection.count(),
            status: presence.status,
            color: presence.color.clone(),
        }
    }
}

/// Callback for presence updates
pub type PresenceCallback = Box<dyn Fn(PresenceUpdate) + Send + Sync>;

/// Internal presence manager state
struct PresenceManagerInner {
    /// Map of user ID to presence
    presences: HashMap<Uuid, UserPresence>,
    /// Presence update callbacks
    callbacks: Vec<PresenceCallback>,
    /// Timeout duration for stale presence
    stale_timeout: Duration,
}

/// Manager for tracking user presence in collaboration sessions
#[derive(Clone)]
pub struct PresenceManager {
    inner: Arc<RwLock<PresenceManagerInner>>,
}

impl PresenceManager {
    /// Create a new presence manager
    pub fn new() -> Self {
        Self::with_timeout(Duration::seconds(30))
    }

    /// Create a presence manager with custom timeout
    pub fn with_timeout(stale_timeout: Duration) -> Self {
        let inner = PresenceManagerInner {
            presences: HashMap::new(),
            callbacks: Vec::new(),
            stale_timeout,
        };

        Self {
            inner: Arc::new(RwLock::new(inner)),
        }
    }

    /// Add a user to presence tracking
    pub fn add_user(&self, user_id: Uuid, color: String) -> Result<()> {
        let mut inner = self.inner.write();
        let presence = UserPresence::new(user_id, color);
        inner.presences.insert(user_id, presence);
        Ok(())
    }

    /// Remove a user from presence tracking
    pub fn remove_user(&self, user_id: Uuid) -> Result<()> {
        let mut inner = self.inner.write();
        inner.presences.remove(&user_id);

        // Emit event
        let update = PresenceUpdate::UserLeft { user_id };
        Self::emit_update(&inner.callbacks, update);

        Ok(())
    }

    /// Update cursor position
    pub fn update_cursor(&self, user_id: Uuid, position: CursorPosition) -> Result<()> {
        let mut inner = self.inner.write();
        let presence = inner
            .presences
            .get_mut(&user_id)
            .ok_or_else(|| CollaborationError::ParticipantNotFound(user_id))?;

        presence.update_cursor(position);

        // Emit event
        let update = PresenceUpdate::CursorMoved { user_id, position };
        Self::emit_update(&inner.callbacks, update);

        Ok(())
    }

    /// Update selection
    pub fn update_selection(&self, user_id: Uuid, selection: SelectionRange) -> Result<()> {
        let mut inner = self.inner.write();
        let presence = inner
            .presences
            .get_mut(&user_id)
            .ok_or_else(|| CollaborationError::ParticipantNotFound(user_id))?;

        presence.update_selection(selection.clone());

        // Emit event
        let update = PresenceUpdate::SelectionChanged { user_id, selection };
        Self::emit_update(&inner.callbacks, update);

        Ok(())
    }

    /// Update activity status
    pub fn update_status(&self, user_id: Uuid, status: ActivityStatus) -> Result<()> {
        let mut inner = self.inner.write();
        let presence = inner
            .presences
            .get_mut(&user_id)
            .ok_or_else(|| CollaborationError::ParticipantNotFound(user_id))?;

        presence.update_status(status);

        // Emit event
        let update = PresenceUpdate::StatusChanged { user_id, status };
        Self::emit_update(&inner.callbacks, update);

        Ok(())
    }

    /// Update viewport
    pub fn update_viewport(&self, user_id: Uuid, viewport: ViewportInfo) -> Result<()> {
        let mut inner = self.inner.write();
        let presence = inner
            .presences
            .get_mut(&user_id)
            .ok_or_else(|| CollaborationError::ParticipantNotFound(user_id))?;

        presence.viewport = Some(viewport.clone());
        presence.last_updated = Utc::now();

        // Emit event
        let update = PresenceUpdate::ViewportChanged { user_id, viewport };
        Self::emit_update(&inner.callbacks, update);

        Ok(())
    }

    /// Get presence for a specific user
    pub fn get_presence(&self, user_id: Uuid) -> Result<UserPresence> {
        let inner = self.inner.read();
        inner
            .presences
            .get(&user_id)
            .cloned()
            .ok_or_else(|| CollaborationError::ParticipantNotFound(user_id))
    }

    /// Get all presences
    pub fn get_all_presences(&self) -> Vec<UserPresence> {
        let inner = self.inner.read();
        inner.presences.values().cloned().collect()
    }

    /// Get active presences (not stale or offline)
    pub fn get_active_presences(&self) -> Vec<UserPresence> {
        let inner = self.inner.read();
        inner
            .presences
            .values()
            .filter(|p| {
                p.status != ActivityStatus::Offline && !p.is_stale(inner.stale_timeout)
            })
            .cloned()
            .collect()
    }

    /// Get presence info for all users (compact format)
    pub fn get_presence_info(&self) -> Vec<PresenceInfo> {
        let inner = self.inner.read();
        inner
            .presences
            .values()
            .map(|p| PresenceInfo::from(p))
            .collect()
    }

    /// Clean up stale presences
    pub fn cleanup_stale(&self) -> Vec<Uuid> {
        let mut inner = self.inner.write();
        let stale_timeout = inner.stale_timeout;

        let stale_users: Vec<Uuid> = inner
            .presences
            .iter_mut()
            .filter(|(_, p)| p.is_stale(stale_timeout) && p.status != ActivityStatus::Offline)
            .map(|(id, p)| {
                p.status = ActivityStatus::Offline;
                *id
            })
            .collect();

        // Emit offline events for stale users
        for user_id in &stale_users {
            let update = PresenceUpdate::StatusChanged {
                user_id: *user_id,
                status: ActivityStatus::Offline,
            };
            Self::emit_update(&inner.callbacks, update);
        }

        stale_users
    }

    /// Register a callback for presence updates
    pub fn on_update(&self, callback: PresenceCallback) {
        let mut inner = self.inner.write();
        inner.callbacks.push(callback);
    }

    /// Emit a presence update to all callbacks
    fn emit_update(callbacks: &[PresenceCallback], update: PresenceUpdate) {
        for callback in callbacks {
            callback(update.clone());
        }
    }

    /// Get number of tracked users
    pub fn user_count(&self) -> usize {
        let inner = self.inner.read();
        inner.presences.len()
    }

    /// Get number of active users
    pub fn active_user_count(&self) -> usize {
        let inner = self.inner.read();
        inner
            .presences
            .values()
            .filter(|p| {
                p.status != ActivityStatus::Offline && !p.is_stale(inner.stale_timeout)
            })
            .count()
    }

    /// Clear all presence data
    pub fn clear(&self) {
        let mut inner = self.inner.write();
        inner.presences.clear();
    }
}

impl Default for PresenceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_position() {
        let cursor = CursorPosition::new_2d(100.0, 200.0);
        assert_eq!(cursor.x, 100.0);
        assert_eq!(cursor.y, 200.0);
        assert_eq!(cursor.z, None);
    }

    #[test]
    fn test_selection_range() {
        let entity1 = Uuid::new_v4();
        let entity2 = Uuid::new_v4();

        let selection = SelectionRange::multiple(vec![entity1, entity2]);
        assert_eq!(selection.count(), 2);
        assert!(!selection.is_empty());
    }

    #[test]
    fn test_presence_manager() {
        let manager = PresenceManager::new();
        let user_id = Uuid::new_v4();

        manager.add_user(user_id, "#ff0000".to_string()).unwrap();
        assert_eq!(manager.user_count(), 1);

        let cursor = CursorPosition::new_2d(50.0, 75.0);
        manager.update_cursor(user_id, cursor).unwrap();

        let presence = manager.get_presence(user_id).unwrap();
        assert_eq!(presence.cursor, Some(cursor));
    }

    #[test]
    fn test_stale_cleanup() {
        let manager = PresenceManager::with_timeout(Duration::milliseconds(100));
        let user_id = Uuid::new_v4();

        manager.add_user(user_id, "#00ff00".to_string()).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(150));

        let stale = manager.cleanup_stale();
        assert_eq!(stale.len(), 1);
        assert_eq!(stale[0], user_id);

        let presence = manager.get_presence(user_id).unwrap();
        assert_eq!(presence.status, ActivityStatus::Offline);
    }
}
