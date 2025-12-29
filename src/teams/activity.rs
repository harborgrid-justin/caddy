//! Activity Tracking and Audit Trail Module
//!
//! Provides comprehensive activity tracking including:
//! - Real-time activity streams
//! - Filterable activity logs
//! - User activity summaries
//! - Team productivity metrics
//! - Complete audit trail for compliance

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum ActivityError {
    #[error("Activity not found: {0}")]
    NotFound(String),

    #[error("Invalid activity: {0}")]
    Invalid(String),

    #[error("Invalid filter: {0}")]
    InvalidFilter(String),

    #[error("Storage error: {0}")]
    Storage(String),
}

pub type ActivityResult<T> = Result<T, ActivityError>;

// ============================================================================
// Core Types
// ============================================================================

/// Activity type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActivityType {
    // Workspace activities
    WorkspaceCreated,
    WorkspaceUpdated,
    WorkspaceArchived,
    WorkspaceRestored,
    WorkspaceDeleted,

    // Member activities
    MemberInvited,
    MemberJoined,
    MemberLeft,
    MemberRemoved,
    MemberRoleChanged,

    // Issue/Assignment activities
    IssueCreated,
    IssueUpdated,
    IssueAssigned,
    IssueReassigned,
    IssueCompleted,
    IssueClosed,
    IssueReopened,
    IssueEscalated,

    // Comment activities
    CommentCreated,
    CommentEdited,
    CommentDeleted,
    ThreadResolved,
    ThreadLocked,

    // Document/Resource activities
    DocumentCreated,
    DocumentUpdated,
    DocumentDeleted,
    DocumentShared,

    // Settings activities
    SettingsUpdated,
    PermissionChanged,
    RoleCreated,
    RoleUpdated,

    // Integration activities
    IntegrationEnabled,
    IntegrationDisabled,
    WebhookTriggered,

    // Security activities
    LoginSuccess,
    LoginFailure,
    LogoutPerformed,
    PasswordChanged,
    TwoFactorEnabled,
    TwoFactorDisabled,

    // Custom activity
    Custom(String),
}

impl ActivityType {
    /// Get display name for activity type
    pub fn display_name(&self) -> &str {
        match self {
            ActivityType::WorkspaceCreated => "Workspace Created",
            ActivityType::WorkspaceUpdated => "Workspace Updated",
            ActivityType::WorkspaceArchived => "Workspace Archived",
            ActivityType::MemberInvited => "Member Invited",
            ActivityType::MemberJoined => "Member Joined",
            ActivityType::IssueAssigned => "Issue Assigned",
            ActivityType::CommentCreated => "Comment Created",
            ActivityType::LoginSuccess => "Login Success",
            ActivityType::Custom(name) => name,
            _ => "Unknown Activity",
        }
    }

    /// Get category for grouping
    pub fn category(&self) -> &str {
        match self {
            ActivityType::WorkspaceCreated
            | ActivityType::WorkspaceUpdated
            | ActivityType::WorkspaceArchived
            | ActivityType::WorkspaceRestored
            | ActivityType::WorkspaceDeleted => "workspace",

            ActivityType::MemberInvited
            | ActivityType::MemberJoined
            | ActivityType::MemberLeft
            | ActivityType::MemberRemoved
            | ActivityType::MemberRoleChanged => "member",

            ActivityType::IssueCreated
            | ActivityType::IssueUpdated
            | ActivityType::IssueAssigned
            | ActivityType::IssueReassigned
            | ActivityType::IssueCompleted
            | ActivityType::IssueClosed
            | ActivityType::IssueReopened
            | ActivityType::IssueEscalated => "issue",

            ActivityType::CommentCreated
            | ActivityType::CommentEdited
            | ActivityType::CommentDeleted
            | ActivityType::ThreadResolved
            | ActivityType::ThreadLocked => "comment",

            ActivityType::LoginSuccess
            | ActivityType::LoginFailure
            | ActivityType::LogoutPerformed
            | ActivityType::PasswordChanged
            | ActivityType::TwoFactorEnabled
            | ActivityType::TwoFactorDisabled => "security",

            _ => "other",
        }
    }

    /// Is this a security-sensitive activity?
    pub fn is_security_event(&self) -> bool {
        self.category() == "security"
            || matches!(
                self,
                ActivityType::PermissionChanged
                    | ActivityType::MemberRoleChanged
                    | ActivityType::RoleUpdated
            )
    }
}

/// Activity entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    /// Activity ID
    pub id: String,

    /// Workspace ID
    pub workspace_id: String,

    /// User who performed the activity
    pub user_id: String,

    /// Activity type
    pub activity_type: ActivityType,

    /// Human-readable description
    pub description: String,

    /// When activity occurred
    pub timestamp: DateTime<Utc>,

    /// Related resource ID (issue, comment, etc.)
    pub resource_id: Option<String>,

    /// Resource type
    pub resource_type: Option<String>,

    /// IP address (for security events)
    pub ip_address: Option<String>,

    /// User agent (for security events)
    pub user_agent: Option<String>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,

    /// Changes made (for audit)
    pub changes: Option<ActivityChanges>,
}

/// Activity changes for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityChanges {
    /// Previous values
    pub before: HashMap<String, String>,

    /// New values
    pub after: HashMap<String, String>,
}

/// Activity filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityFilter {
    /// Filter by workspace
    pub workspace_id: Option<String>,

    /// Filter by user
    pub user_id: Option<String>,

    /// Filter by activity types
    pub activity_types: Option<Vec<ActivityType>>,

    /// Filter by category
    pub categories: Option<Vec<String>>,

    /// Filter by resource
    pub resource_id: Option<String>,

    /// Start date/time
    pub start_date: Option<DateTime<Utc>>,

    /// End date/time
    pub end_date: Option<DateTime<Utc>>,

    /// Security events only?
    pub security_only: bool,

    /// Limit number of results
    pub limit: Option<usize>,

    /// Offset for pagination
    pub offset: Option<usize>,
}

impl Default for ActivityFilter {
    fn default() -> Self {
        Self {
            workspace_id: None,
            user_id: None,
            activity_types: None,
            categories: None,
            resource_id: None,
            start_date: None,
            end_date: None,
            security_only: false,
            limit: Some(100),
            offset: None,
        }
    }
}

/// Activity feed configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityFeed {
    /// Feed ID
    pub id: String,

    /// Feed name
    pub name: String,

    /// Workspace ID
    pub workspace_id: String,

    /// Feed filter
    pub filter: ActivityFilter,

    /// Auto-refresh interval (seconds)
    pub refresh_interval: Option<u64>,

    /// Created by
    pub created_by: String,

    /// Created at
    pub created_at: DateTime<Utc>,
}

/// User activity summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivitySummary {
    /// User ID
    pub user_id: String,

    /// Workspace ID
    pub workspace_id: String,

    /// Time period
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,

    /// Total activities
    pub total_activities: usize,

    /// Activities by type
    pub activities_by_type: HashMap<String, usize>,

    /// Most active day
    pub most_active_day: Option<DateTime<Utc>>,

    /// Average activities per day
    pub avg_activities_per_day: f32,

    /// Last activity timestamp
    pub last_activity: Option<DateTime<Utc>>,
}

/// Team productivity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMetrics {
    /// Workspace ID
    pub workspace_id: String,

    /// Time period
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,

    /// Total team activities
    pub total_activities: usize,

    /// Active members count
    pub active_members: usize,

    /// Issues created
    pub issues_created: usize,

    /// Issues completed
    pub issues_completed: usize,

    /// Comments created
    pub comments_created: usize,

    /// Average response time (hours)
    pub avg_response_time: Option<f32>,

    /// Completion rate (completed / created)
    pub completion_rate: Option<f32>,

    /// Most active users
    pub top_users: Vec<(String, usize)>, // (user_id, activity_count)

    /// Activity trend (daily counts)
    pub activity_trend: Vec<(DateTime<Utc>, usize)>,
}

/// Audit entry (immutable record)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Audit entry ID
    pub id: String,

    /// Activity this audits
    pub activity_id: String,

    /// Workspace ID
    pub workspace_id: String,

    /// User who performed action
    pub user_id: String,

    /// Action performed
    pub action: String,

    /// Resource affected
    pub resource_type: String,
    pub resource_id: String,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// IP address
    pub ip_address: Option<String>,

    /// Changes made
    pub changes: Option<ActivityChanges>,

    /// Success or failure
    pub success: bool,

    /// Error message (if failed)
    pub error_message: Option<String>,

    /// Compliance tags
    pub compliance_tags: Vec<String>,
}

// ============================================================================
// Activity Manager
// ============================================================================

/// Manages activity tracking and reporting
#[derive(Debug)]
pub struct ActivityManager {
    activities: Vec<Activity>,
    audit_entries: Vec<AuditEntry>,
    feeds: HashMap<String, ActivityFeed>,
    activity_index: HashMap<String, usize>, // id -> index
    workspace_index: HashMap<String, Vec<String>>, // workspace_id -> activity_ids
    user_index: HashMap<String, Vec<String>>, // user_id -> activity_ids
}

impl ActivityManager {
    /// Create a new activity manager
    pub fn new() -> Self {
        Self {
            activities: Vec::new(),
            audit_entries: Vec::new(),
            feeds: HashMap::new(),
            activity_index: HashMap::new(),
            workspace_index: HashMap::new(),
            user_index: HashMap::new(),
        }
    }

    /// Log an activity
    pub fn log_activity(
        &mut self,
        workspace_id: String,
        user_id: String,
        activity_type: ActivityType,
        description: String,
        resource_id: Option<String>,
    ) -> ActivityResult<Activity> {
        let activity = Activity {
            id: Uuid::new_v4().to_string(),
            workspace_id: workspace_id.clone(),
            user_id: user_id.clone(),
            activity_type: activity_type.clone(),
            description,
            timestamp: Utc::now(),
            resource_id,
            resource_type: None,
            ip_address: None,
            user_agent: None,
            metadata: HashMap::new(),
            changes: None,
        };

        // Add to storage
        let index = self.activities.len();
        self.activity_index.insert(activity.id.clone(), index);
        self.activities.push(activity.clone());

        // Update indexes
        self.workspace_index
            .entry(workspace_id)
            .or_insert_with(Vec::new)
            .push(activity.id.clone());

        self.user_index
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(activity.id.clone());

        // Create audit entry if it's a security event
        if activity_type.is_security_event() {
            self.create_audit_entry(&activity)?;
        }

        Ok(activity)
    }

    /// Log activity with changes (for audit trail)
    pub fn log_activity_with_changes(
        &mut self,
        workspace_id: String,
        user_id: String,
        activity_type: ActivityType,
        description: String,
        resource_id: Option<String>,
        before: HashMap<String, String>,
        after: HashMap<String, String>,
    ) -> ActivityResult<Activity> {
        let activity = self.log_activity(
            workspace_id,
            user_id,
            activity_type,
            description,
            resource_id,
        )?;

        let index = *self.activity_index.get(&activity.id).unwrap();
        self.activities[index].changes = Some(ActivityChanges { before, after });

        Ok(activity)
    }

    /// Create audit entry
    fn create_audit_entry(&mut self, activity: &Activity) -> ActivityResult<()> {
        let audit = AuditEntry {
            id: Uuid::new_v4().to_string(),
            activity_id: activity.id.clone(),
            workspace_id: activity.workspace_id.clone(),
            user_id: activity.user_id.clone(),
            action: activity.activity_type.display_name().to_string(),
            resource_type: activity.resource_type.clone().unwrap_or_default(),
            resource_id: activity.resource_id.clone().unwrap_or_default(),
            timestamp: activity.timestamp,
            ip_address: activity.ip_address.clone(),
            changes: activity.changes.clone(),
            success: true,
            error_message: None,
            compliance_tags: vec!["audit".to_string()],
        };

        self.audit_entries.push(audit);

        Ok(())
    }

    /// Get recent activity for workspace
    pub fn get_recent_activity(
        &self,
        workspace_id: &str,
        limit: usize,
    ) -> ActivityResult<Vec<&Activity>> {
        let activity_ids = self
            .workspace_index
            .get(workspace_id)
            .cloned()
            .unwrap_or_default();

        let mut activities: Vec<&Activity> = activity_ids
            .iter()
            .filter_map(|id| {
                self.activity_index
                    .get(id)
                    .and_then(|&index| self.activities.get(index))
            })
            .collect();

        // Sort by timestamp descending
        activities.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(activities.into_iter().take(limit).collect())
    }

    /// Filter activities
    pub fn filter_activities(&self, filter: &ActivityFilter) -> ActivityResult<Vec<&Activity>> {
        let mut activities: Vec<&Activity> = self.activities.iter().collect();

        // Apply filters
        if let Some(ref workspace_id) = filter.workspace_id {
            activities.retain(|a| &a.workspace_id == workspace_id);
        }

        if let Some(ref user_id) = filter.user_id {
            activities.retain(|a| &a.user_id == user_id);
        }

        if let Some(ref types) = filter.activity_types {
            activities.retain(|a| types.contains(&a.activity_type));
        }

        if let Some(ref categories) = filter.categories {
            activities.retain(|a| categories.contains(&a.activity_type.category().to_string()));
        }

        if let Some(ref resource_id) = filter.resource_id {
            activities.retain(|a| a.resource_id.as_ref() == Some(resource_id));
        }

        if let Some(start) = filter.start_date {
            activities.retain(|a| a.timestamp >= start);
        }

        if let Some(end) = filter.end_date {
            activities.retain(|a| a.timestamp <= end);
        }

        if filter.security_only {
            activities.retain(|a| a.activity_type.is_security_event());
        }

        // Sort by timestamp descending
        activities.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply pagination
        let offset = filter.offset.unwrap_or(0);
        let limit = filter.limit.unwrap_or(100);

        Ok(activities
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect())
    }

    /// Get user activity summary
    pub fn get_user_summary(
        &self,
        user_id: &str,
        workspace_id: &str,
        days: i64,
    ) -> ActivityResult<UserActivitySummary> {
        let end_date = Utc::now();
        let start_date = end_date - Duration::days(days);

        let filter = ActivityFilter {
            workspace_id: Some(workspace_id.to_string()),
            user_id: Some(user_id.to_string()),
            start_date: Some(start_date),
            end_date: Some(end_date),
            ..Default::default()
        };

        let activities = self.filter_activities(&filter)?;

        let total_activities = activities.len();

        let mut activities_by_type: HashMap<String, usize> = HashMap::new();
        let mut last_activity: Option<DateTime<Utc>> = None;

        for activity in &activities {
            *activities_by_type
                .entry(activity.activity_type.display_name().to_string())
                .or_insert(0) += 1;

            if last_activity.is_none() || activity.timestamp > last_activity.unwrap() {
                last_activity = Some(activity.timestamp);
            }
        }

        let avg_activities_per_day = total_activities as f32 / days as f32;

        Ok(UserActivitySummary {
            user_id: user_id.to_string(),
            workspace_id: workspace_id.to_string(),
            period_start: start_date,
            period_end: end_date,
            total_activities,
            activities_by_type,
            most_active_day: None, // Would calculate from daily aggregations
            avg_activities_per_day,
            last_activity,
        })
    }

    /// Get team metrics
    pub fn get_team_metrics(
        &self,
        workspace_id: &str,
        days: i64,
    ) -> ActivityResult<TeamMetrics> {
        let end_date = Utc::now();
        let start_date = end_date - Duration::days(days);

        let filter = ActivityFilter {
            workspace_id: Some(workspace_id.to_string()),
            start_date: Some(start_date),
            end_date: Some(end_date),
            ..Default::default()
        };

        let activities = self.filter_activities(&filter)?;

        let total_activities = activities.len();

        let mut active_members = std::collections::HashSet::new();
        let mut user_activity_counts: HashMap<String, usize> = HashMap::new();

        let mut issues_created = 0;
        let mut issues_completed = 0;
        let mut comments_created = 0;

        for activity in &activities {
            active_members.insert(activity.user_id.clone());
            *user_activity_counts
                .entry(activity.user_id.clone())
                .or_insert(0) += 1;

            match activity.activity_type {
                ActivityType::IssueCreated => issues_created += 1,
                ActivityType::IssueCompleted => issues_completed += 1,
                ActivityType::CommentCreated => comments_created += 1,
                _ => {}
            }
        }

        let completion_rate = if issues_created > 0 {
            Some(issues_completed as f32 / issues_created as f32)
        } else {
            None
        };

        let mut top_users: Vec<(String, usize)> = user_activity_counts.into_iter().collect();
        top_users.sort_by(|a, b| b.1.cmp(&a.1));
        top_users.truncate(10);

        Ok(TeamMetrics {
            workspace_id: workspace_id.to_string(),
            period_start: start_date,
            period_end: end_date,
            total_activities,
            active_members: active_members.len(),
            issues_created,
            issues_completed,
            comments_created,
            avg_response_time: None, // Would calculate from issue/comment timestamps
            completion_rate,
            top_users,
            activity_trend: Vec::new(), // Would calculate daily aggregations
        })
    }

    /// Create activity feed
    pub fn create_feed(
        &mut self,
        name: String,
        workspace_id: String,
        filter: ActivityFilter,
        created_by: String,
    ) -> ActivityResult<ActivityFeed> {
        let feed = ActivityFeed {
            id: Uuid::new_v4().to_string(),
            name,
            workspace_id,
            filter,
            refresh_interval: Some(30), // 30 seconds
            created_by,
            created_at: Utc::now(),
        };

        self.feeds.insert(feed.id.clone(), feed.clone());

        Ok(feed)
    }

    /// Get audit trail
    pub fn get_audit_trail(
        &self,
        workspace_id: &str,
        limit: usize,
    ) -> Vec<&AuditEntry> {
        let mut entries: Vec<&AuditEntry> = self
            .audit_entries
            .iter()
            .filter(|e| e.workspace_id == workspace_id)
            .collect();

        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        entries.into_iter().take(limit).collect()
    }
}

impl Default for ActivityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_activity() {
        let mut manager = ActivityManager::new();

        let activity = manager
            .log_activity(
                "workspace1".to_string(),
                "user1".to_string(),
                ActivityType::IssueCreated,
                "Created issue #123".to_string(),
                Some("issue-123".to_string()),
            )
            .unwrap();

        assert_eq!(activity.workspace_id, "workspace1");
        assert_eq!(activity.user_id, "user1");
    }

    #[test]
    fn test_filter_activities() {
        let mut manager = ActivityManager::new();

        manager
            .log_activity(
                "workspace1".to_string(),
                "user1".to_string(),
                ActivityType::IssueCreated,
                "Issue 1".to_string(),
                None,
            )
            .unwrap();

        manager
            .log_activity(
                "workspace1".to_string(),
                "user2".to_string(),
                ActivityType::CommentCreated,
                "Comment 1".to_string(),
                None,
            )
            .unwrap();

        let filter = ActivityFilter {
            workspace_id: Some("workspace1".to_string()),
            user_id: Some("user1".to_string()),
            ..Default::default()
        };

        let filtered = manager.filter_activities(&filter).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].user_id, "user1");
    }

    #[test]
    fn test_user_summary() {
        let mut manager = ActivityManager::new();

        for i in 0..5 {
            manager
                .log_activity(
                    "workspace1".to_string(),
                    "user1".to_string(),
                    ActivityType::IssueCreated,
                    format!("Issue {}", i),
                    None,
                )
                .unwrap();
        }

        let summary = manager
            .get_user_summary("user1", "workspace1", 7)
            .unwrap();

        assert_eq!(summary.total_activities, 5);
        assert!(summary.avg_activities_per_day > 0.0);
    }

    #[test]
    fn test_team_metrics() {
        let mut manager = ActivityManager::new();

        manager
            .log_activity(
                "workspace1".to_string(),
                "user1".to_string(),
                ActivityType::IssueCreated,
                "Issue 1".to_string(),
                None,
            )
            .unwrap();

        manager
            .log_activity(
                "workspace1".to_string(),
                "user2".to_string(),
                ActivityType::IssueCompleted,
                "Issue 1 completed".to_string(),
                None,
            )
            .unwrap();

        let metrics = manager.get_team_metrics("workspace1", 7).unwrap();

        assert_eq!(metrics.active_members, 2);
        assert_eq!(metrics.issues_created, 1);
        assert_eq!(metrics.issues_completed, 1);
    }
}
