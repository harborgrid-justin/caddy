//! Collaboration and Comment System
//!
//! Provides rich collaboration features including:
//! - Threaded discussions and comments
//! - @mentions and notifications
//! - Rich text formatting (Markdown)
//! - File attachments
//! - Comment reactions and threading

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum CommentError {
    #[error("Comment not found: {0}")]
    NotFound(String),

    #[error("Thread not found: {0}")]
    ThreadNotFound(String),

    #[error("Invalid comment: {0}")]
    Invalid(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Thread is locked: {0}")]
    ThreadLocked(String),

    #[error("Attachment too large: {0}")]
    AttachmentTooLarge(String),

    #[error("Invalid mention: {0}")]
    InvalidMention(String),
}

pub type CommentResult<T> = Result<T, CommentError>;

// ============================================================================
// Core Types
// ============================================================================

/// Comment on an issue or resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// Comment ID
    pub id: String,

    /// Parent resource (issue, PR, etc.)
    pub resource_id: String,

    /// Resource type
    pub resource_type: String,

    /// Thread ID (for threading)
    pub thread_id: Option<String>,

    /// Parent comment ID (for replies)
    pub parent_id: Option<String>,

    /// Author user ID
    pub author_id: String,

    /// Comment content
    pub content: RichContent,

    /// When comment was created
    pub created_at: DateTime<Utc>,

    /// When comment was last edited
    pub edited_at: Option<DateTime<Utc>>,

    /// Is comment deleted?
    pub deleted: bool,

    /// Mentions in this comment
    pub mentions: Vec<Mention>,

    /// Attached files
    pub attachments: Vec<Attachment>,

    /// Reactions to comment
    pub reactions: HashMap<String, Vec<String>>, // emoji -> user_ids

    /// Comment metadata
    pub metadata: HashMap<String, String>,
}

/// Rich text content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichContent {
    /// Raw markdown text
    pub markdown: String,

    /// Rendered HTML (cached)
    pub html: Option<String>,

    /// Plain text version
    pub plain_text: String,

    /// Content format version
    pub format_version: String,
}

impl RichContent {
    /// Create from markdown
    pub fn from_markdown(markdown: String) -> Self {
        let plain_text = markdown.clone(); // Simplified; would strip markdown
        Self {
            markdown,
            html: None,
            plain_text,
            format_version: "1.0".to_string(),
        }
    }

    /// Create from plain text
    pub fn from_plain(text: String) -> Self {
        Self {
            markdown: text.clone(),
            html: None,
            plain_text: text,
            format_version: "1.0".to_string(),
        }
    }
}

/// Mention of a user in a comment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mention {
    /// Mentioned user ID
    pub user_id: String,

    /// User's display name at time of mention
    pub display_name: String,

    /// Position in text (character offset)
    pub position: usize,

    /// Mention type
    pub mention_type: MentionType,
}

/// Type of mention
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MentionType {
    /// Direct user mention (@username)
    User,

    /// Team mention (@team)
    Team,

    /// Role mention (@role)
    Role,

    /// Everyone mention (@everyone)
    Everyone,
}

/// File attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// Attachment ID
    pub id: String,

    /// Original filename
    pub filename: String,

    /// MIME type
    pub content_type: String,

    /// File size in bytes
    pub size: u64,

    /// Storage URL/path
    pub url: String,

    /// Thumbnail URL (for images)
    pub thumbnail_url: Option<String>,

    /// When uploaded
    pub uploaded_at: DateTime<Utc>,

    /// Who uploaded
    pub uploaded_by: String,

    /// File checksum
    pub checksum: String,
}

/// Comment thread
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentThread {
    /// Thread ID
    pub id: String,

    /// Resource this thread is attached to
    pub resource_id: String,

    /// Resource type
    pub resource_type: String,

    /// Thread title
    pub title: Option<String>,

    /// Thread status
    pub status: ThreadStatus,

    /// When thread was created
    pub created_at: DateTime<Utc>,

    /// Thread creator
    pub created_by: String,

    /// When thread was last updated
    pub updated_at: DateTime<Utc>,

    /// Is thread resolved?
    pub resolved: bool,

    /// Who resolved the thread
    pub resolved_by: Option<String>,

    /// When resolved
    pub resolved_at: Option<DateTime<Utc>>,

    /// Is thread locked?
    pub locked: bool,

    /// Thread participants
    pub participants: Vec<String>,

    /// Number of comments
    pub comment_count: usize,
}

/// Thread status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreadStatus {
    /// Thread is open
    Open,

    /// Thread is resolved
    Resolved,

    /// Thread is locked
    Locked,

    /// Thread is archived
    Archived,
}

/// Notification for a comment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentNotification {
    /// Notification ID
    pub id: String,

    /// Comment ID
    pub comment_id: String,

    /// User to notify
    pub user_id: String,

    /// Notification reason
    pub reason: NotificationReason,

    /// When notification was created
    pub created_at: DateTime<Utc>,

    /// Has been read?
    pub read: bool,

    /// When notification was read
    pub read_at: Option<DateTime<Utc>>,
}

/// Reason for notification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationReason {
    /// User was mentioned
    Mentioned,

    /// User is thread participant
    ThreadParticipant,

    /// User is resource owner
    ResourceOwner,

    /// User is subscribed to thread
    Subscribed,

    /// Reply to user's comment
    Reply,
}

// ============================================================================
// Comment Manager
// ============================================================================

/// Manages comments and threads
#[derive(Debug)]
pub struct CommentManager {
    comments: HashMap<String, Comment>,
    threads: HashMap<String, CommentThread>,
    notifications: HashMap<String, CommentNotification>,
    resource_index: HashMap<String, Vec<String>>, // resource_id -> comment_ids
    thread_index: HashMap<String, Vec<String>>,   // thread_id -> comment_ids
    user_index: HashMap<String, Vec<String>>,     // user_id -> comment_ids
}

impl CommentManager {
    /// Create a new comment manager
    pub fn new() -> Self {
        Self {
            comments: HashMap::new(),
            threads: HashMap::new(),
            notifications: HashMap::new(),
            resource_index: HashMap::new(),
            thread_index: HashMap::new(),
            user_index: HashMap::new(),
        }
    }

    /// Create a new comment
    pub fn create_comment(
        &mut self,
        resource_id: String,
        resource_type: String,
        author_id: String,
        content: String,
        parent_id: Option<String>,
    ) -> CommentResult<Comment> {
        let comment_id = Uuid::new_v4().to_string();

        // Extract mentions from content
        let mentions = self.extract_mentions(&content);

        // Create thread if this is a top-level comment
        let thread_id = if parent_id.is_none() {
            let thread = self.create_thread(
                resource_id.clone(),
                resource_type.clone(),
                author_id.clone(),
            )?;
            Some(thread.id)
        } else {
            // Get parent's thread
            parent_id.as_ref().and_then(|pid| {
                self.comments.get(pid).and_then(|c| c.thread_id.clone())
            })
        };

        let comment = Comment {
            id: comment_id.clone(),
            resource_id: resource_id.clone(),
            resource_type,
            thread_id: thread_id.clone(),
            parent_id: parent_id.clone(),
            author_id: author_id.clone(),
            content: RichContent::from_markdown(content),
            created_at: Utc::now(),
            edited_at: None,
            deleted: false,
            mentions: mentions.clone(),
            attachments: Vec::new(),
            reactions: HashMap::new(),
            metadata: HashMap::new(),
        };

        // Update indexes
        self.resource_index
            .entry(resource_id)
            .or_insert_with(Vec::new)
            .push(comment_id.clone());

        if let Some(tid) = &thread_id {
            self.thread_index
                .entry(tid.clone())
                .or_insert_with(Vec::new)
                .push(comment_id.clone());

            // Update thread
            if let Some(thread) = self.threads.get_mut(tid) {
                thread.comment_count += 1;
                thread.updated_at = Utc::now();
                if !thread.participants.contains(&author_id) {
                    thread.participants.push(author_id.clone());
                }
            }
        }

        self.user_index
            .entry(author_id.clone())
            .or_insert_with(Vec::new)
            .push(comment_id.clone());

        self.comments.insert(comment_id.clone(), comment.clone());

        // Create notifications for mentions
        self.create_mention_notifications(&comment, &mentions)?;

        // Notify thread participants
        if let Some(tid) = &thread_id {
            self.notify_thread_participants(tid, &comment)?;
        }

        Ok(comment)
    }

    /// Create a comment thread
    fn create_thread(
        &mut self,
        resource_id: String,
        resource_type: String,
        created_by: String,
    ) -> CommentResult<CommentThread> {
        let thread = CommentThread {
            id: Uuid::new_v4().to_string(),
            resource_id,
            resource_type,
            title: None,
            status: ThreadStatus::Open,
            created_at: Utc::now(),
            created_by: created_by.clone(),
            updated_at: Utc::now(),
            resolved: false,
            resolved_by: None,
            resolved_at: None,
            locked: false,
            participants: vec![created_by],
            comment_count: 0,
        };

        self.threads.insert(thread.id.clone(), thread.clone());

        Ok(thread)
    }

    /// Extract mentions from comment text
    fn extract_mentions(&self, content: &str) -> Vec<Mention> {
        let mut mentions = Vec::new();
        let mut _pos = 0;

        for (i, c) in content.chars().enumerate() {
            if c == '@' {
                // Find end of mention
                let rest: String = content.chars().skip(i + 1).take_while(|&ch| {
                    ch.is_alphanumeric() || ch == '_' || ch == '-'
                }).collect();

                if !rest.is_empty() {
                    let mention_type = if rest == "everyone" {
                        MentionType::Everyone
                    } else if rest.starts_with("team-") {
                        MentionType::Team
                    } else if rest.starts_with("role-") {
                        MentionType::Role
                    } else {
                        MentionType::User
                    };

                    mentions.push(Mention {
                        user_id: rest.clone(),
                        display_name: rest,
                        position: i,
                        mention_type,
                    });
                }
            }
            _pos = i;
        }

        mentions
    }

    /// Edit a comment
    pub fn edit_comment(&mut self, comment_id: &str, new_content: String) -> CommentResult<()> {
        let comment = self
            .comments
            .get_mut(comment_id)
            .ok_or_else(|| CommentError::NotFound(comment_id.to_string()))?;

        if comment.deleted {
            return Err(CommentError::Invalid("Cannot edit deleted comment".to_string()));
        }

        comment.content = RichContent::from_markdown(new_content.clone());
        comment.edited_at = Some(Utc::now());
        comment.mentions = self.extract_mentions(&new_content);

        Ok(())
    }

    /// Delete a comment
    pub fn delete_comment(&mut self, comment_id: &str) -> CommentResult<()> {
        let comment = self
            .comments
            .get_mut(comment_id)
            .ok_or_else(|| CommentError::NotFound(comment_id.to_string()))?;

        comment.deleted = true;
        comment.content = RichContent::from_plain("[deleted]".to_string());

        Ok(())
    }

    /// Add attachment to comment
    pub fn add_attachment(
        &mut self,
        comment_id: &str,
        filename: String,
        content_type: String,
        size: u64,
        url: String,
        uploaded_by: String,
    ) -> CommentResult<Attachment> {
        const MAX_SIZE: u64 = 100 * 1024 * 1024; // 100MB

        if size > MAX_SIZE {
            return Err(CommentError::AttachmentTooLarge(format!(
                "{} bytes exceeds {} bytes limit",
                size, MAX_SIZE
            )));
        }

        let attachment = Attachment {
            id: Uuid::new_v4().to_string(),
            filename,
            content_type,
            size,
            url,
            thumbnail_url: None,
            uploaded_at: Utc::now(),
            uploaded_by,
            checksum: String::new(), // Would compute actual checksum
        };

        let comment = self
            .comments
            .get_mut(comment_id)
            .ok_or_else(|| CommentError::NotFound(comment_id.to_string()))?;

        comment.attachments.push(attachment.clone());

        Ok(attachment)
    }

    /// Add reaction to comment
    pub fn add_reaction(
        &mut self,
        comment_id: &str,
        user_id: &str,
        emoji: &str,
    ) -> CommentResult<()> {
        let comment = self
            .comments
            .get_mut(comment_id)
            .ok_or_else(|| CommentError::NotFound(comment_id.to_string()))?;

        comment
            .reactions
            .entry(emoji.to_string())
            .or_insert_with(Vec::new)
            .push(user_id.to_string());

        Ok(())
    }

    /// Remove reaction from comment
    pub fn remove_reaction(
        &mut self,
        comment_id: &str,
        user_id: &str,
        emoji: &str,
    ) -> CommentResult<()> {
        let comment = self
            .comments
            .get_mut(comment_id)
            .ok_or_else(|| CommentError::NotFound(comment_id.to_string()))?;

        if let Some(users) = comment.reactions.get_mut(emoji) {
            users.retain(|id| id != user_id);
        }

        Ok(())
    }

    /// Resolve a thread
    pub fn resolve_thread(&mut self, thread_id: &str, resolved_by: &str) -> CommentResult<()> {
        let thread = self
            .threads
            .get_mut(thread_id)
            .ok_or_else(|| CommentError::ThreadNotFound(thread_id.to_string()))?;

        thread.resolved = true;
        thread.resolved_by = Some(resolved_by.to_string());
        thread.resolved_at = Some(Utc::now());
        thread.status = ThreadStatus::Resolved;

        Ok(())
    }

    /// Lock a thread
    pub fn lock_thread(&mut self, thread_id: &str) -> CommentResult<()> {
        let thread = self
            .threads
            .get_mut(thread_id)
            .ok_or_else(|| CommentError::ThreadNotFound(thread_id.to_string()))?;

        thread.locked = true;
        thread.status = ThreadStatus::Locked;

        Ok(())
    }

    /// Get comments for a resource
    pub fn get_resource_comments(&self, resource_id: &str) -> Vec<&Comment> {
        self.resource_index
            .get(resource_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.comments.get(id))
                    .filter(|c| !c.deleted)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get thread comments
    pub fn get_thread_comments(&self, thread_id: &str) -> Vec<&Comment> {
        self.thread_index
            .get(thread_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.comments.get(id))
                    .filter(|c| !c.deleted)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get user's notifications
    pub fn get_user_notifications(&self, user_id: &str, unread_only: bool) -> Vec<&CommentNotification> {
        self.notifications
            .values()
            .filter(|n| {
                n.user_id == user_id && (!unread_only || !n.read)
            })
            .collect()
    }

    /// Mark notification as read
    pub fn mark_notification_read(&mut self, notification_id: &str) -> CommentResult<()> {
        let notification = self
            .notifications
            .get_mut(notification_id)
            .ok_or_else(|| CommentError::NotFound(notification_id.to_string()))?;

        notification.read = true;
        notification.read_at = Some(Utc::now());

        Ok(())
    }

    /// Create notifications for mentions
    fn create_mention_notifications(
        &mut self,
        comment: &Comment,
        mentions: &[Mention],
    ) -> CommentResult<()> {
        for mention in mentions {
            if mention.mention_type == MentionType::User {
                let notification = CommentNotification {
                    id: Uuid::new_v4().to_string(),
                    comment_id: comment.id.clone(),
                    user_id: mention.user_id.clone(),
                    reason: NotificationReason::Mentioned,
                    created_at: Utc::now(),
                    read: false,
                    read_at: None,
                };

                self.notifications.insert(notification.id.clone(), notification);
            }
        }

        Ok(())
    }

    /// Notify thread participants
    fn notify_thread_participants(
        &mut self,
        thread_id: &str,
        comment: &Comment,
    ) -> CommentResult<()> {
        if let Some(thread) = self.threads.get(thread_id) {
            for participant in &thread.participants {
                if participant != &comment.author_id {
                    let notification = CommentNotification {
                        id: Uuid::new_v4().to_string(),
                        comment_id: comment.id.clone(),
                        user_id: participant.clone(),
                        reason: NotificationReason::ThreadParticipant,
                        created_at: Utc::now(),
                        read: false,
                        read_at: None,
                    };

                    self.notifications.insert(notification.id.clone(), notification);
                }
            }
        }

        Ok(())
    }
}

impl Default for CommentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_comment() {
        let mut manager = CommentManager::new();

        let comment = manager
            .create_comment(
                "issue1".to_string(),
                "issue".to_string(),
                "user1".to_string(),
                "This is a test comment".to_string(),
                None,
            )
            .unwrap();

        assert_eq!(comment.resource_id, "issue1");
        assert_eq!(comment.author_id, "user1");
        assert!(!comment.deleted);
    }

    #[test]
    fn test_mention_extraction() {
        let manager = CommentManager::new();

        let content = "Hey @john, can you help @jane with this?";
        let mentions = manager.extract_mentions(content);

        assert_eq!(mentions.len(), 2);
        assert_eq!(mentions[0].user_id, "john");
        assert_eq!(mentions[1].user_id, "jane");
    }

    #[test]
    fn test_comment_threading() {
        let mut manager = CommentManager::new();

        let parent = manager
            .create_comment(
                "issue1".to_string(),
                "issue".to_string(),
                "user1".to_string(),
                "Parent comment".to_string(),
                None,
            )
            .unwrap();

        let reply = manager
            .create_comment(
                "issue1".to_string(),
                "issue".to_string(),
                "user2".to_string(),
                "Reply to parent".to_string(),
                Some(parent.id.clone()),
            )
            .unwrap();

        assert_eq!(reply.parent_id, Some(parent.id));
    }

    #[test]
    fn test_reactions() {
        let mut manager = CommentManager::new();

        let comment = manager
            .create_comment(
                "issue1".to_string(),
                "issue".to_string(),
                "user1".to_string(),
                "Test".to_string(),
                None,
            )
            .unwrap();

        manager.add_reaction(&comment.id, "user2", "👍").unwrap();
        manager.add_reaction(&comment.id, "user3", "👍").unwrap();

        let updated = manager.comments.get(&comment.id).unwrap();
        assert_eq!(updated.reactions.get("👍").unwrap().len(), 2);
    }
}
