//! # Room Management
//!
//! Manages collaboration rooms, permissions, and state persistence.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime};
use thiserror::Error;
use uuid::Uuid;

use super::document::DocumentState;
use super::ot::Operation;
use super::presence::{PresenceManager, UserInfo, UserPresence};

/// Errors related to room management
#[derive(Debug, Error)]
pub enum RoomError {
    #[error("Room not found: {0}")]
    RoomNotFound(String),
    #[error("User not found: {0}")]
    UserNotFound(Uuid),
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Room is full")]
    RoomFull,
    #[error("Invalid room state")]
    InvalidState,
    #[error("Room already exists: {0}")]
    RoomExists(String),
}

/// Room access level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AccessLevel {
    /// Can only view
    Viewer,
    /// Can view and comment
    Commenter,
    /// Can edit
    Editor,
    /// Can edit and manage users
    Moderator,
    /// Full control
    Owner,
}

impl AccessLevel {
    /// Check if this level can perform an action
    pub fn can_edit(&self) -> bool {
        matches!(self, AccessLevel::Editor | AccessLevel::Moderator | AccessLevel::Owner)
    }

    pub fn can_moderate(&self) -> bool {
        matches!(self, AccessLevel::Moderator | AccessLevel::Owner)
    }

    pub fn can_delete(&self) -> bool {
        matches!(self, AccessLevel::Owner)
    }
}

/// User permission in a room
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermission {
    /// User ID
    pub user_id: Uuid,
    /// Access level
    pub access_level: AccessLevel,
    /// When permission was granted
    pub granted_at: SystemTime,
    /// Who granted the permission
    pub granted_by: Uuid,
}

impl UserPermission {
    pub fn new(user_id: Uuid, access_level: AccessLevel, granted_by: Uuid) -> Self {
        Self {
            user_id,
            access_level,
            granted_at: SystemTime::now(),
            granted_by,
        }
    }
}

/// Room settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomSettings {
    /// Maximum number of users
    pub max_users: usize,
    /// Whether room is public
    pub is_public: bool,
    /// Whether to allow anonymous users
    pub allow_anonymous: bool,
    /// Require password to join
    pub password_protected: bool,
    /// Auto-save interval
    pub autosave_interval: Duration,
    /// Enable version history
    pub version_history: bool,
    /// Maximum version history size
    pub max_history_size: usize,
}

impl Default for RoomSettings {
    fn default() -> Self {
        Self {
            max_users: 100,
            is_public: false,
            allow_anonymous: false,
            password_protected: false,
            autosave_interval: Duration::from_secs(60),
            version_history: true,
            max_history_size: 1000,
        }
    }
}

/// Room state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomState {
    /// Room is active and accepting connections
    Active,
    /// Room is paused (no edits allowed)
    Paused,
    /// Room is archived (read-only)
    Archived,
    /// Room is closed
    Closed,
}

/// Collaboration room
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    /// Unique room ID
    pub id: String,
    /// Room name
    pub name: String,
    /// Room description
    pub description: Option<String>,
    /// Room owner
    pub owner: Uuid,
    /// Room settings
    pub settings: RoomSettings,
    /// Current state
    pub state: RoomState,
    /// User permissions
    permissions: HashMap<Uuid, UserPermission>,
    /// Document state
    #[serde(skip)]
    document: Option<DocumentState>,
    /// Presence manager (transient)
    #[serde(skip)]
    presence: PresenceManager,
    /// Creation timestamp
    pub created_at: SystemTime,
    /// Last activity timestamp
    pub last_activity: SystemTime,
    /// Password hash (if password protected)
    password_hash: Option<String>,
    /// Room metadata
    pub metadata: HashMap<String, String>,
}

impl Room {
    /// Create a new room
    pub fn new(id: String, name: String, owner: Uuid) -> Self {
        let mut permissions = HashMap::new();
        permissions.insert(
            owner,
            UserPermission::new(owner, AccessLevel::Owner, owner),
        );

        let now = SystemTime::now();

        Self {
            id,
            name,
            description: None,
            owner,
            settings: RoomSettings::default(),
            state: RoomState::Active,
            permissions,
            document: None,
            presence: PresenceManager::new(),
            created_at: now,
            last_activity: now,
            password_hash: None,
            metadata: HashMap::new(),
        }
    }

    /// Initialize room with a document
    pub fn with_document(mut self, document: DocumentState) -> Self {
        self.document = Some(document);
        self
    }

    /// Set room settings
    pub fn with_settings(mut self, settings: RoomSettings) -> Self {
        self.settings = settings;
        self
    }

    /// Set room password
    pub fn set_password(&mut self, password: &str) {
        // In production, use proper password hashing (argon2, bcrypt, etc.)
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        self.password_hash = Some(format!("{:x}", hasher.finalize()));
        self.settings.password_protected = true;
    }

    /// Verify password
    pub fn verify_password(&self, password: &str) -> bool {
        if let Some(hash) = &self.password_hash {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(password.as_bytes());
            let computed_hash = format!("{:x}", hasher.finalize());
            computed_hash == *hash
        } else {
            true // No password set
        }
    }

    /// Add a user to the room
    pub fn add_user(
        &mut self,
        user: UserInfo,
        session_id: Uuid,
        access_level: AccessLevel,
        granted_by: Uuid,
    ) -> Result<(), RoomError> {
        // Check if room is full
        if self.presence.user_count() >= self.settings.max_users {
            return Err(RoomError::RoomFull);
        }

        // Check if room is accepting new users
        if self.state != RoomState::Active {
            return Err(RoomError::InvalidState);
        }

        // Add permission
        self.permissions.insert(
            user.id,
            UserPermission::new(user.id, access_level, granted_by),
        );

        // Add to presence
        self.presence.add_user(user, session_id);

        self.last_activity = SystemTime::now();

        Ok(())
    }

    /// Remove a user from the room
    pub fn remove_user(&mut self, user_id: Uuid) -> Result<(), RoomError> {
        self.permissions.remove(&user_id);
        self.presence.remove_user(user_id);
        self.last_activity = SystemTime::now();
        Ok(())
    }

    /// Get user permission
    pub fn get_permission(&self, user_id: Uuid) -> Option<&UserPermission> {
        self.permissions.get(&user_id)
    }

    /// Check if user has access level
    pub fn check_access(&self, user_id: Uuid, required_level: AccessLevel) -> bool {
        if let Some(perm) = self.permissions.get(&user_id) {
            perm.access_level >= required_level
        } else {
            false
        }
    }

    /// Update user access level
    pub fn update_access(
        &mut self,
        user_id: Uuid,
        new_level: AccessLevel,
        updated_by: Uuid,
    ) -> Result<(), RoomError> {
        // Check if updater has permission
        if !self.check_access(updated_by, AccessLevel::Moderator) {
            return Err(RoomError::PermissionDenied);
        }

        // Cannot change owner's access level
        if user_id == self.owner {
            return Err(RoomError::PermissionDenied);
        }

        if let Some(perm) = self.permissions.get_mut(&user_id) {
            perm.access_level = new_level;
            perm.granted_by = updated_by;
            perm.granted_at = SystemTime::now();
            Ok(())
        } else {
            Err(RoomError::UserNotFound(user_id))
        }
    }

    /// Get all users in the room
    pub fn get_users(&self) -> Vec<&UserPresence> {
        self.presence.get_all_users()
    }

    /// Get active users
    pub fn get_active_users(&self) -> Vec<&UserPresence> {
        self.presence.get_active_users()
    }

    /// Get user count
    pub fn user_count(&self) -> usize {
        self.presence.user_count()
    }

    /// Apply an operation to the document
    pub fn apply_operation(
        &mut self,
        operation: Operation,
        user_id: Uuid,
    ) -> Result<u64, RoomError> {
        // Check if user has edit permission
        if !self.check_access(user_id, AccessLevel::Editor) {
            return Err(RoomError::PermissionDenied);
        }

        // Check if room is active
        if self.state != RoomState::Active {
            return Err(RoomError::InvalidState);
        }

        // Apply to document
        if let Some(doc) = &mut self.document {
            doc.apply_operation(operation, user_id, "User edit".to_string())
                .map_err(|_| RoomError::InvalidState)
        } else {
            Err(RoomError::InvalidState)
        }
    }

    /// Get current document content
    pub fn get_content(&self) -> Option<String> {
        self.document.as_ref()?.current_content().ok()
    }

    /// Get document state
    pub fn document(&self) -> Option<&DocumentState> {
        self.document.as_ref()
    }

    /// Get mutable document state
    pub fn document_mut(&mut self) -> Option<&mut DocumentState> {
        self.document.as_mut()
    }

    /// Get presence manager
    pub fn presence(&self) -> &PresenceManager {
        &self.presence
    }

    /// Get mutable presence manager
    pub fn presence_mut(&mut self) -> &mut PresenceManager {
        &mut self.presence
    }

    /// Change room state
    pub fn set_state(&mut self, state: RoomState, user_id: Uuid) -> Result<(), RoomError> {
        if !self.check_access(user_id, AccessLevel::Moderator) {
            return Err(RoomError::PermissionDenied);
        }

        self.state = state;
        self.last_activity = SystemTime::now();
        Ok(())
    }

    /// Cleanup expired sessions
    pub fn cleanup(&mut self) {
        let expired = self.presence.cleanup_expired();
        for user_id in expired {
            self.permissions.remove(&user_id);
        }
    }

    /// Get room statistics
    pub fn stats(&self) -> RoomStats {
        RoomStats {
            total_users: self.user_count(),
            active_users: self.get_active_users().len(),
            document_version: self.document.as_ref().map(|d| d.current_version()).unwrap_or(0),
            state: self.state,
        }
    }
}

/// Room statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomStats {
    pub total_users: usize,
    pub active_users: usize,
    pub document_version: u64,
    pub state: RoomState,
}

/// Room manager
#[derive(Debug)]
pub struct RoomManager {
    /// Active rooms
    rooms: HashMap<String, Room>,
    /// Room ownership index
    owner_index: HashMap<Uuid, HashSet<String>>,
}

impl RoomManager {
    /// Create a new room manager
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
            owner_index: HashMap::new(),
        }
    }

    /// Create a new room
    pub fn create_room(&mut self, id: String, name: String, owner: Uuid) -> Result<&Room, RoomError> {
        if self.rooms.contains_key(&id) {
            return Err(RoomError::RoomExists(id));
        }

        let room = Room::new(id.clone(), name, owner);
        self.rooms.insert(id.clone(), room);

        // Update owner index
        self.owner_index
            .entry(owner)
            .or_insert_with(HashSet::new)
            .insert(id.clone());

        Ok(self.rooms.get(&id).unwrap())
    }

    /// Get a room
    pub fn get_room(&self, room_id: &str) -> Option<&Room> {
        self.rooms.get(room_id)
    }

    /// Get a mutable room
    pub fn get_room_mut(&mut self, room_id: &str) -> Option<&mut Room> {
        self.rooms.get_mut(room_id)
    }

    /// Delete a room
    pub fn delete_room(&mut self, room_id: &str, user_id: Uuid) -> Result<(), RoomError> {
        let room = self.rooms.get(room_id).ok_or_else(|| RoomError::RoomNotFound(room_id.to_string()))?;

        // Only owner can delete
        if room.owner != user_id {
            return Err(RoomError::PermissionDenied);
        }

        // Remove from owner index
        if let Some(rooms) = self.owner_index.get_mut(&room.owner) {
            rooms.remove(room_id);
        }

        self.rooms.remove(room_id);
        Ok(())
    }

    /// Get all rooms owned by a user
    pub fn get_user_rooms(&self, user_id: Uuid) -> Vec<&Room> {
        if let Some(room_ids) = self.owner_index.get(&user_id) {
            room_ids
                .iter()
                .filter_map(|id| self.rooms.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all rooms where user has access
    pub fn get_accessible_rooms(&self, user_id: Uuid) -> Vec<&Room> {
        self.rooms
            .values()
            .filter(|r| r.permissions.contains_key(&user_id))
            .collect()
    }

    /// Get room count
    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }

    /// Cleanup all rooms
    pub fn cleanup_all(&mut self) {
        for room in self.rooms.values_mut() {
            room.cleanup();
        }

        // Remove empty closed rooms
        let to_remove: Vec<String> = self
            .rooms
            .iter()
            .filter(|(_, r)| r.state == RoomState::Closed && r.user_count() == 0)
            .map(|(id, _)| id.clone())
            .collect();

        for room_id in to_remove {
            if let Some(room) = self.rooms.remove(&room_id) {
                if let Some(rooms) = self.owner_index.get_mut(&room.owner) {
                    rooms.remove(&room_id);
                }
            }
        }
    }
}

impl Default for RoomManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_room() {
        let owner = Uuid::new_v4();
        let room = Room::new("room1".to_string(), "Test Room".to_string(), owner);

        assert_eq!(room.id, "room1");
        assert_eq!(room.owner, owner);
        assert_eq!(room.state, RoomState::Active);
    }

    #[test]
    fn test_add_user_to_room() {
        let owner = Uuid::new_v4();
        let mut room = Room::new("room1".to_string(), "Test Room".to_string(), owner);

        let user = UserInfo::new(Uuid::new_v4(), "Alice".to_string());
        let session = Uuid::new_v4();

        room.add_user(user.clone(), session, AccessLevel::Editor, owner)
            .unwrap();

        assert_eq!(room.user_count(), 1);
        assert!(room.check_access(user.id, AccessLevel::Editor));
    }

    #[test]
    fn test_permission_check() {
        let owner = Uuid::new_v4();
        let mut room = Room::new("room1".to_string(), "Test Room".to_string(), owner);

        let user = UserInfo::new(Uuid::new_v4(), "Alice".to_string());
        let session = Uuid::new_v4();

        room.add_user(user.clone(), session, AccessLevel::Viewer, owner)
            .unwrap();

        assert!(room.check_access(user.id, AccessLevel::Viewer));
        assert!(!room.check_access(user.id, AccessLevel::Editor));
    }

    #[test]
    fn test_room_manager() {
        let mut manager = RoomManager::new();
        let owner = Uuid::new_v4();

        manager
            .create_room("room1".to_string(), "Test".to_string(), owner)
            .unwrap();

        assert_eq!(manager.room_count(), 1);

        let room = manager.get_room("room1").unwrap();
        assert_eq!(room.name, "Test");
    }

    #[test]
    fn test_password_protection() {
        let owner = Uuid::new_v4();
        let mut room = Room::new("room1".to_string(), "Test Room".to_string(), owner);

        room.set_password("secret123");
        assert!(room.settings.password_protected);
        assert!(room.verify_password("secret123"));
        assert!(!room.verify_password("wrong"));
    }

    #[test]
    fn test_access_levels() {
        assert!(AccessLevel::Owner.can_edit());
        assert!(AccessLevel::Editor.can_edit());
        assert!(!AccessLevel::Viewer.can_edit());

        assert!(AccessLevel::Owner.can_moderate());
        assert!(!AccessLevel::Editor.can_moderate());
    }

    #[test]
    fn test_update_access() {
        let owner = Uuid::new_v4();
        let mut room = Room::new("room1".to_string(), "Test Room".to_string(), owner);

        let user = UserInfo::new(Uuid::new_v4(), "Alice".to_string());
        let session = Uuid::new_v4();

        room.add_user(user.clone(), session, AccessLevel::Viewer, owner)
            .unwrap();

        room.update_access(user.id, AccessLevel::Editor, owner)
            .unwrap();

        assert!(room.check_access(user.id, AccessLevel::Editor));
    }
}
