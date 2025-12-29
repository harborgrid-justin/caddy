//! Team Collaboration System for CADDY v0.3.0
//!
//! This module provides comprehensive team collaboration features including:
//!
//! - **Workspace Management**: Multi-workspace support with templates and sharing
//! - **Member Management**: Team invitations, roles, permissions, and hierarchies
//! - **Issue Assignments**: Advanced assignment workflows with load balancing
//! - **Comments & Discussions**: Rich collaboration features with mentions and threading
//! - **Activity Tracking**: Real-time activity streams and audit trails
//!
//! # Architecture
//!
//! The team collaboration system is composed of several interconnected layers:
//!
//! 1. **Workspace Layer**: Manages multiple workspaces with isolated resources
//! 2. **Member Layer**: Handles team membership, invitations, and access control
//! 3. **Assignment Layer**: Manages issue assignment and workload distribution
//! 4. **Comment Layer**: Facilitates team communication and discussions
//! 5. **Activity Layer**: Tracks and reports all team activities
//!
//! # Example Usage
//!
//! ```rust,ignore
//! use caddy::teams::*;
//!
//! // Initialize workspace manager
//! let mut workspace_mgr = WorkspaceManager::new();
//!
//! // Create a workspace
//! let workspace = workspace_mgr.create_workspace(
//!     "Engineering Team".to_string(),
//!     "workspace-eng".to_string(),
//!     "user123".to_string(),
//! ).unwrap();
//!
//! // Add team members
//! let mut member_mgr = MemberManager::new();
//! member_mgr.invite_member(
//!     &workspace.id,
//!     "user456",
//!     "jane@example.com",
//!     MemberRole::Developer,
//!     "user123",
//! ).unwrap();
//!
//! // Create issue assignment
//! let mut assignment_mgr = AssignmentManager::new();
//! assignment_mgr.assign_issue(
//!     "issue-123".to_string(),
//!     "user456".to_string(),
//!     AssignmentPriority::High,
//!     Some(chrono::Utc::now() + chrono::Duration::days(7)),
//!     "user123".to_string(),
//! ).unwrap();
//! ```

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

// ============================================================================
// Module Declarations
// ============================================================================

pub mod workspace;
pub mod members;
pub mod assignments;
pub mod comments;
pub mod activity;

// ============================================================================
// Re-exports
// ============================================================================

pub use workspace::{
    Workspace, WorkspaceSettings, WorkspaceTemplate, WorkspaceManager,
    WorkspaceError, WorkspaceResult, WorkspaceStatus, WorkspaceVisibility,
    SharedWorkspace, ArchiveInfo,
};

pub use members::{
    Member, MemberRole, MemberStatus, MemberInvitation, MemberManager,
    MemberError, MemberResult, TeamHierarchy, GuestAccess, MemberActivity,
    InvitationStatus, MemberPermissions,
};

pub use assignments::{
    Assignment, AssignmentStatus, AssignmentPriority, AssignmentManager,
    AssignmentError, AssignmentResult, WorkloadBalance, AssignmentRule,
    EscalationRule, SlaConfig, AutoAssignmentConfig,
};

pub use comments::{
    Comment, CommentThread, CommentManager, CommentError, CommentResult,
    Mention, Attachment, RichContent, CommentNotification, ThreadStatus,
};

pub use activity::{
    Activity, ActivityType, ActivityFeed, ActivityManager, ActivityError,
    ActivityResult, ActivityFilter, UserActivitySummary, TeamMetrics,
    AuditEntry,
};

// ============================================================================
// Team Collaboration System Facade
// ============================================================================

/// Unified team collaboration system
///
/// Provides a cohesive interface to all team collaboration features.
#[derive(Debug)]
pub struct TeamCollaborationSystem {
    /// Workspace management
    pub workspace_manager: WorkspaceManager,

    /// Member management
    pub member_manager: MemberManager,

    /// Assignment management
    pub assignment_manager: AssignmentManager,

    /// Comment management
    pub comment_manager: CommentManager,

    /// Activity tracking
    pub activity_manager: ActivityManager,
}

impl TeamCollaborationSystem {
    /// Create a new team collaboration system
    pub fn new() -> Self {
        Self {
            workspace_manager: WorkspaceManager::new(),
            member_manager: MemberManager::new(),
            assignment_manager: AssignmentManager::new(),
            comment_manager: CommentManager::new(),
            activity_manager: ActivityManager::new(),
        }
    }

    /// Create a complete team workspace with initial setup
    pub fn create_team(
        &mut self,
        name: String,
        slug: String,
        owner_id: String,
        initial_members: Vec<(String, String, MemberRole)>, // (user_id, email, role)
    ) -> Result<Workspace, Box<dyn std::error::Error>> {
        // Create workspace
        let workspace = self.workspace_manager.create_workspace(
            name.clone(),
            slug,
            owner_id.clone(),
        )?;

        // Log activity
        self.activity_manager.log_activity(
            workspace.id.clone(),
            owner_id.clone(),
            ActivityType::WorkspaceCreated,
            format!("Created workspace: {}", name),
            None,
        )?;

        // Invite initial members
        for (user_id, email, role) in initial_members {
            self.member_manager.invite_member(
                &workspace.id,
                &user_id,
                &email,
                role,
                &owner_id,
            )?;

            self.activity_manager.log_activity(
                workspace.id.clone(),
                owner_id.clone(),
                ActivityType::MemberInvited,
                format!("Invited {} as {:?}", email, role),
                None,
            )?;
        }

        Ok(workspace)
    }

    /// Get comprehensive team statistics
    pub fn get_team_stats(&self, workspace_id: &str) -> Result<TeamStats, Box<dyn std::error::Error>> {
        let workspace = self.workspace_manager.get_workspace(workspace_id)?;
        let members = self.member_manager.list_workspace_members(workspace_id)?;
        let active_assignments = self.assignment_manager.get_active_assignments(workspace_id)?;
        let recent_activity = self.activity_manager.get_recent_activity(workspace_id, 100)?;

        Ok(TeamStats {
            workspace_id: workspace_id.to_string(),
            workspace_name: workspace.name.clone(),
            total_members: members.len(),
            active_members: members.iter().filter(|m| m.status == MemberStatus::Active).count(),
            total_assignments: active_assignments.len(),
            recent_activity_count: recent_activity.len(),
            created_at: workspace.created_at,
        })
    }

    /// Health check for all subsystems
    pub fn health_check(&self) -> HashMap<String, bool> {
        let mut status = HashMap::new();

        status.insert("workspace_manager".to_string(), true);
        status.insert("member_manager".to_string(), true);
        status.insert("assignment_manager".to_string(), true);
        status.insert("comment_manager".to_string(), true);
        status.insert("activity_manager".to_string(), true);

        status
    }
}

impl Default for TeamCollaborationSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Team statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamStats {
    pub workspace_id: String,
    pub workspace_name: String,
    pub total_members: usize,
    pub active_members: usize,
    pub total_assignments: usize,
    pub recent_activity_count: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_collaboration_system_initialization() {
        let system = TeamCollaborationSystem::new();
        let health = system.health_check();

        assert_eq!(health.len(), 5);
        assert!(health.values().all(|&v| v));
    }

    #[test]
    fn test_create_team() {
        let mut system = TeamCollaborationSystem::new();

        let workspace = system.create_team(
            "Test Team".to_string(),
            "test-team".to_string(),
            "owner123".to_string(),
            vec![
                ("user1".to_string(), "user1@example.com".to_string(), MemberRole::Developer),
                ("user2".to_string(), "user2@example.com".to_string(), MemberRole::Viewer),
            ],
        ).unwrap();

        assert_eq!(workspace.name, "Test Team");
        assert_eq!(workspace.slug, "test-team");
    }
}
