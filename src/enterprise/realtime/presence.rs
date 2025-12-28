//! # Presence System
//!
//! Tracks user presence, cursors, selections, and activity status.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use thiserror::Error;
use uuid::Uuid;

/// Errors related to presence management
#[derive(Debug, Error)]
pub enum PresenceError {
    #[error("User not found: {0}")]
    UserNotFound(Uuid),
    #[error("Invalid cursor position: {0}")]
    InvalidCursorPosition(usize),
    #[error("Session expired")]
    SessionExpired,
}

/// User status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatus {
    /// User is actively editing
    Editing,
    /// User is viewing but not editing
    Viewing,
    /// User is idle (no activity)
    Idle,
    /// User is away
    Away,
    /// User is offline
    Offline,
}

/// Cursor position in document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CursorPosition {
    /// Line number (0-indexed)
    pub line: usize,
    /// Column number (0-indexed)
    pub column: usize,
    /// Absolute character position
    pub offset: usize,
}

impl CursorPosition {
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }

    /// Create cursor at document start
    pub fn start() -> Self {
        Self {
            line: 0,
            column: 0,
            offset: 0,
        }
    }
}

/// Selection range in document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selection {
    /// Start position (anchor)
    pub anchor: CursorPosition,
    /// End position (head)
    pub head: CursorPosition,
}

impl Selection {
    pub fn new(anchor: CursorPosition, head: CursorPosition) -> Self {
        Self { anchor, head }
    }

    /// Check if selection is collapsed (cursor with no selection)
    pub fn is_collapsed(&self) -> bool {
        self.anchor == self.head
    }

    /// Get the range as (start_offset, end_offset)
    pub fn range(&self) -> (usize, usize) {
        if self.anchor.offset <= self.head.offset {
            (self.anchor.offset, self.head.offset)
        } else {
            (self.head.offset, self.anchor.offset)
        }
    }

    /// Get the length of the selection
    pub fn len(&self) -> usize {
        let (start, end) = self.range();
        end - start
    }

    /// Check if selection is empty
    pub fn is_empty(&self) -> bool {
        self.is_collapsed()
    }
}

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    /// User ID
    pub id: Uuid,
    /// Display name
    pub name: String,
    /// User color (for cursor/selection highlighting)
    pub color: String,
    /// Avatar URL
    pub avatar: Option<String>,
    /// Email
    pub email: Option<String>,
}

impl UserInfo {
    pub fn new(id: Uuid, name: String) -> Self {
        Self {
            id,
            name,
            color: Self::generate_color(id),
            avatar: None,
            email: None,
        }
    }

    /// Generate a deterministic color from user ID
    fn generate_color(id: Uuid) -> String {
        let bytes = id.as_bytes();
        let r = bytes[0];
        let g = bytes[1];
        let b = bytes[2];
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }
}

/// User presence state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPresence {
    /// User information
    pub user: UserInfo,
    /// Current cursor position
    pub cursor: CursorPosition,
    /// Current selection (if any)
    pub selection: Option<Selection>,
    /// Current status
    pub status: UserStatus,
    /// Last activity timestamp
    pub last_activity: SystemTime,
    /// Last heartbeat timestamp
    pub last_heartbeat: SystemTime,
    /// Session ID
    pub session_id: Uuid,
    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

impl UserPresence {
    /// Create new user presence
    pub fn new(user: UserInfo, session_id: Uuid) -> Self {
        let now = SystemTime::now();
        Self {
            user,
            cursor: CursorPosition::start(),
            selection: None,
            status: UserStatus::Viewing,
            last_activity: now,
            last_heartbeat: now,
            session_id,
            metadata: HashMap::new(),
        }
    }

    /// Update cursor position
    pub fn update_cursor(&mut self, position: CursorPosition) {
        self.cursor = position;
        self.last_activity = SystemTime::now();
        self.status = UserStatus::Editing;
    }

    /// Update selection
    pub fn update_selection(&mut self, selection: Option<Selection>) {
        self.selection = selection;
        self.last_activity = SystemTime::now();
    }

    /// Update status
    pub fn update_status(&mut self, status: UserStatus) {
        self.status = status;
        self.last_activity = SystemTime::now();
    }

    /// Send heartbeat
    pub fn heartbeat(&mut self) {
        self.last_heartbeat = SystemTime::now();
    }

    /// Check if user is active (has recent activity)
    pub fn is_active(&self, timeout: Duration) -> bool {
        SystemTime::now()
            .duration_since(self.last_activity)
            .map(|d| d < timeout)
            .unwrap_or(false)
    }

    /// Check if session is alive (has recent heartbeat)
    pub fn is_alive(&self, timeout: Duration) -> bool {
        SystemTime::now()
            .duration_since(self.last_heartbeat)
            .map(|d| d < timeout)
            .unwrap_or(false)
    }

    /// Get time since last activity
    pub fn idle_time(&self) -> Option<Duration> {
        SystemTime::now().duration_since(self.last_activity).ok()
    }
}

/// Presence manager
#[derive(Debug, Clone)]
pub struct PresenceManager {
    /// Active user presences
    presences: HashMap<Uuid, UserPresence>,
    /// Heartbeat timeout duration
    heartbeat_timeout: Duration,
    /// Activity timeout duration
    activity_timeout: Duration,
    /// Idle timeout duration
    idle_timeout: Duration,
}

impl PresenceManager {
    /// Create a new presence manager
    pub fn new() -> Self {
        Self {
            presences: HashMap::new(),
            heartbeat_timeout: Duration::from_secs(30),
            activity_timeout: Duration::from_secs(300), // 5 minutes
            idle_timeout: Duration::from_secs(60),      // 1 minute
        }
    }

    /// Configure timeouts
    pub fn with_timeouts(
        mut self,
        heartbeat: Duration,
        activity: Duration,
        idle: Duration,
    ) -> Self {
        self.heartbeat_timeout = heartbeat;
        self.activity_timeout = activity;
        self.idle_timeout = idle;
        self
    }

    /// Add a user presence
    pub fn add_user(&mut self, user: UserInfo, session_id: Uuid) -> Uuid {
        let presence = UserPresence::new(user.clone(), session_id);
        let user_id = user.id;
        self.presences.insert(user_id, presence);
        user_id
    }

    /// Remove a user presence
    pub fn remove_user(&mut self, user_id: Uuid) -> Option<UserPresence> {
        self.presences.remove(&user_id)
    }

    /// Get user presence
    pub fn get_user(&self, user_id: Uuid) -> Option<&UserPresence> {
        self.presences.get(&user_id)
    }

    /// Get mutable user presence
    pub fn get_user_mut(&mut self, user_id: Uuid) -> Option<&mut UserPresence> {
        self.presences.get_mut(&user_id)
    }

    /// Update user cursor
    pub fn update_cursor(
        &mut self,
        user_id: Uuid,
        position: CursorPosition,
    ) -> Result<(), PresenceError> {
        let presence = self
            .presences
            .get_mut(&user_id)
            .ok_or(PresenceError::UserNotFound(user_id))?;

        presence.update_cursor(position);
        Ok(())
    }

    /// Update user selection
    pub fn update_selection(
        &mut self,
        user_id: Uuid,
        selection: Option<Selection>,
    ) -> Result<(), PresenceError> {
        let presence = self
            .presences
            .get_mut(&user_id)
            .ok_or(PresenceError::UserNotFound(user_id))?;

        presence.update_selection(selection);
        Ok(())
    }

    /// Update user status
    pub fn update_status(
        &mut self,
        user_id: Uuid,
        status: UserStatus,
    ) -> Result<(), PresenceError> {
        let presence = self
            .presences
            .get_mut(&user_id)
            .ok_or(PresenceError::UserNotFound(user_id))?;

        presence.update_status(status);
        Ok(())
    }

    /// Process heartbeat from user
    pub fn heartbeat(&mut self, user_id: Uuid) -> Result<(), PresenceError> {
        let presence = self
            .presences
            .get_mut(&user_id)
            .ok_or(PresenceError::UserNotFound(user_id))?;

        presence.heartbeat();
        Ok(())
    }

    /// Get all active users
    pub fn get_active_users(&self) -> Vec<&UserPresence> {
        self.presences
            .values()
            .filter(|p| p.is_active(self.activity_timeout))
            .collect()
    }

    /// Get all users
    pub fn get_all_users(&self) -> Vec<&UserPresence> {
        self.presences.values().collect()
    }

    /// Clean up expired sessions
    pub fn cleanup_expired(&mut self) -> Vec<Uuid> {
        let expired: Vec<Uuid> = self
            .presences
            .iter()
            .filter(|(_, p)| !p.is_alive(self.heartbeat_timeout))
            .map(|(id, _)| *id)
            .collect();

        for user_id in &expired {
            self.presences.remove(user_id);
        }

        expired
    }

    /// Update idle statuses based on activity
    pub fn update_idle_statuses(&mut self) {
        for presence in self.presences.values_mut() {
            if let Some(idle_time) = presence.idle_time() {
                if idle_time > self.idle_timeout && presence.status == UserStatus::Editing {
                    presence.status = UserStatus::Idle;
                }
            }
        }
    }

    /// Get user count
    pub fn user_count(&self) -> usize {
        self.presences.len()
    }

    /// Get active user count
    pub fn active_user_count(&self) -> usize {
        self.get_active_users().len()
    }

    /// Get users by status
    pub fn get_users_by_status(&self, status: UserStatus) -> Vec<&UserPresence> {
        self.presences
            .values()
            .filter(|p| p.status == status)
            .collect()
    }
}

impl Default for PresenceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Heartbeat manager for automated heartbeat sending
#[derive(Debug)]
pub struct HeartbeatManager {
    /// Interval between heartbeats
    interval: Duration,
    /// Last heartbeat time
    last_heartbeat: SystemTime,
    /// User ID
    user_id: Uuid,
}

impl HeartbeatManager {
    /// Create a new heartbeat manager
    pub fn new(user_id: Uuid, interval: Duration) -> Self {
        Self {
            interval,
            last_heartbeat: SystemTime::now(),
            user_id,
        }
    }

    /// Check if it's time to send a heartbeat
    pub fn should_send(&self) -> bool {
        SystemTime::now()
            .duration_since(self.last_heartbeat)
            .map(|d| d >= self.interval)
            .unwrap_or(true)
    }

    /// Mark heartbeat as sent
    pub fn mark_sent(&mut self) {
        self.last_heartbeat = SystemTime::now();
    }

    /// Get user ID
    pub fn user_id(&self) -> Uuid {
        self.user_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_position() {
        let cursor = CursorPosition::new(10, 5, 105);
        assert_eq!(cursor.line, 10);
        assert_eq!(cursor.column, 5);
        assert_eq!(cursor.offset, 105);
    }

    #[test]
    fn test_selection() {
        let anchor = CursorPosition::new(0, 0, 0);
        let head = CursorPosition::new(0, 5, 5);
        let selection = Selection::new(anchor, head);

        assert!(!selection.is_collapsed());
        assert_eq!(selection.len(), 5);
        assert_eq!(selection.range(), (0, 5));
    }

    #[test]
    fn test_user_presence() {
        let user = UserInfo::new(Uuid::new_v4(), "Alice".to_string());
        let session = Uuid::new_v4();
        let mut presence = UserPresence::new(user, session);

        let cursor = CursorPosition::new(1, 2, 3);
        presence.update_cursor(cursor);

        assert_eq!(presence.cursor, cursor);
        assert_eq!(presence.status, UserStatus::Editing);
    }

    #[test]
    fn test_presence_manager() {
        let mut manager = PresenceManager::new();

        let user = UserInfo::new(Uuid::new_v4(), "Bob".to_string());
        let session = Uuid::new_v4();
        let user_id = manager.add_user(user, session);

        assert_eq!(manager.user_count(), 1);

        let cursor = CursorPosition::new(5, 10, 60);
        manager.update_cursor(user_id, cursor).unwrap();

        let presence = manager.get_user(user_id).unwrap();
        assert_eq!(presence.cursor, cursor);
    }

    #[test]
    fn test_heartbeat_manager() {
        let user_id = Uuid::new_v4();
        let mut hb_manager = HeartbeatManager::new(user_id, Duration::from_millis(100));

        assert!(hb_manager.should_send());
        hb_manager.mark_sent();
        assert!(!hb_manager.should_send());

        std::thread::sleep(Duration::from_millis(150));
        assert!(hb_manager.should_send());
    }

    #[test]
    fn test_user_color_generation() {
        let user_id = Uuid::new_v4();
        let user = UserInfo::new(user_id, "Test".to_string());

        assert!(user.color.starts_with('#'));
        assert_eq!(user.color.len(), 7); // #RRGGBB
    }
}
