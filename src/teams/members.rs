//! Team Member Management Module
//!
//! Provides comprehensive member management including:
//! - Member invitation system with email verification
//! - Role-based permissions and hierarchies
//! - Team structure and reporting lines
//! - Guest access control
//! - Member activity tracking

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum MemberError {
    #[error("Member not found: {0}")]
    NotFound(String),

    #[error("Member already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid invitation: {0}")]
    InvalidInvitation(String),

    #[error("Invitation expired: {0}")]
    InvitationExpired(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid role: {0}")]
    InvalidRole(String),

    #[error("Member limit reached for workspace")]
    MemberLimitReached,

    #[error("Cannot remove last owner")]
    CannotRemoveLastOwner,
}

pub type MemberResult<T> = Result<T, MemberError>;

// ============================================================================
// Core Types
// ============================================================================

/// Member role in the workspace
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemberRole {
    /// Full workspace control
    Owner,

    /// Administrative access
    Admin,

    /// Project management access
    Manager,

    /// Development access
    Developer,

    /// Review and comment access
    Reviewer,

    /// Design access
    Designer,

    /// Read-only access
    Viewer,

    /// Temporary guest access
    Guest,
}

impl MemberRole {
    /// Get role hierarchy level (higher = more privileges)
    pub fn hierarchy_level(&self) -> u8 {
        match self {
            MemberRole::Owner => 100,
            MemberRole::Admin => 80,
            MemberRole::Manager => 60,
            MemberRole::Developer => 40,
            MemberRole::Designer => 40,
            MemberRole::Reviewer => 30,
            MemberRole::Viewer => 10,
            MemberRole::Guest => 5,
        }
    }

    /// Check if this role can manage another role
    pub fn can_manage(&self, other: &MemberRole) -> bool {
        self.hierarchy_level() > other.hierarchy_level()
    }

    /// Get permissions for this role
    pub fn permissions(&self) -> MemberPermissions {
        match self {
            MemberRole::Owner => MemberPermissions::owner(),
            MemberRole::Admin => MemberPermissions::admin(),
            MemberRole::Manager => MemberPermissions::manager(),
            MemberRole::Developer => MemberPermissions::developer(),
            MemberRole::Designer => MemberPermissions::designer(),
            MemberRole::Reviewer => MemberPermissions::reviewer(),
            MemberRole::Viewer => MemberPermissions::viewer(),
            MemberRole::Guest => MemberPermissions::guest(),
        }
    }
}

/// Member permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberPermissions {
    pub can_read: bool,
    pub can_create: bool,
    pub can_edit: bool,
    pub can_delete: bool,
    pub can_manage_members: bool,
    pub can_manage_settings: bool,
    pub can_assign_issues: bool,
    pub can_comment: bool,
    pub can_approve: bool,
    pub can_export: bool,
}

impl MemberPermissions {
    pub fn owner() -> Self {
        Self {
            can_read: true,
            can_create: true,
            can_edit: true,
            can_delete: true,
            can_manage_members: true,
            can_manage_settings: true,
            can_assign_issues: true,
            can_comment: true,
            can_approve: true,
            can_export: true,
        }
    }

    pub fn admin() -> Self {
        Self {
            can_read: true,
            can_create: true,
            can_edit: true,
            can_delete: true,
            can_manage_members: true,
            can_manage_settings: true,
            can_assign_issues: true,
            can_comment: true,
            can_approve: true,
            can_export: true,
        }
    }

    pub fn manager() -> Self {
        Self {
            can_read: true,
            can_create: true,
            can_edit: true,
            can_delete: false,
            can_manage_members: true,
            can_manage_settings: false,
            can_assign_issues: true,
            can_comment: true,
            can_approve: true,
            can_export: true,
        }
    }

    pub fn developer() -> Self {
        Self {
            can_read: true,
            can_create: true,
            can_edit: true,
            can_delete: false,
            can_manage_members: false,
            can_manage_settings: false,
            can_assign_issues: false,
            can_comment: true,
            can_approve: false,
            can_export: true,
        }
    }

    pub fn designer() -> Self {
        Self {
            can_read: true,
            can_create: true,
            can_edit: true,
            can_delete: false,
            can_manage_members: false,
            can_manage_settings: false,
            can_assign_issues: false,
            can_comment: true,
            can_approve: false,
            can_export: true,
        }
    }

    pub fn reviewer() -> Self {
        Self {
            can_read: true,
            can_create: false,
            can_edit: false,
            can_delete: false,
            can_manage_members: false,
            can_manage_settings: false,
            can_assign_issues: false,
            can_comment: true,
            can_approve: true,
            can_export: false,
        }
    }

    pub fn viewer() -> Self {
        Self {
            can_read: true,
            can_create: false,
            can_edit: false,
            can_delete: false,
            can_manage_members: false,
            can_manage_settings: false,
            can_assign_issues: false,
            can_comment: true,
            can_approve: false,
            can_export: false,
        }
    }

    pub fn guest() -> Self {
        Self {
            can_read: true,
            can_create: false,
            can_edit: false,
            can_delete: false,
            can_manage_members: false,
            can_manage_settings: false,
            can_assign_issues: false,
            can_comment: true,
            can_approve: false,
            can_export: false,
        }
    }
}

/// Member status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberStatus {
    /// Active member
    Active,

    /// Invitation pending
    Pending,

    /// Member deactivated
    Inactive,

    /// Member suspended
    Suspended,
}

/// Team member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    /// Member ID
    pub id: String,

    /// Workspace ID
    pub workspace_id: String,

    /// User ID
    pub user_id: String,

    /// Email address
    pub email: String,

    /// Display name
    pub display_name: Option<String>,

    /// Member role
    pub role: MemberRole,

    /// Member status
    pub status: MemberStatus,

    /// When member joined
    pub joined_at: DateTime<Utc>,

    /// Last activity timestamp
    pub last_active_at: Option<DateTime<Utc>>,

    /// Invitation ID (if pending)
    pub invitation_id: Option<String>,

    /// Manager/supervisor user ID
    pub manager_id: Option<String>,

    /// Team/department
    pub team: Option<String>,

    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

/// Member invitation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberInvitation {
    /// Invitation ID
    pub id: String,

    /// Workspace ID
    pub workspace_id: String,

    /// Invited user ID
    pub user_id: String,

    /// Email address
    pub email: String,

    /// Proposed role
    pub role: MemberRole,

    /// Invitation token
    pub token: String,

    /// Who sent the invitation
    pub invited_by: String,

    /// When invitation was sent
    pub created_at: DateTime<Utc>,

    /// When invitation expires
    pub expires_at: DateTime<Utc>,

    /// Invitation status
    pub status: InvitationStatus,

    /// Custom message
    pub message: Option<String>,
}

/// Invitation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Declined,
    Expired,
    Cancelled,
}

/// Team hierarchy node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamHierarchy {
    /// Member ID
    pub member_id: String,

    /// Direct reports
    pub reports: Vec<String>,

    /// Manager ID
    pub manager_id: Option<String>,

    /// Organizational level
    pub level: u8,
}

/// Guest access configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestAccess {
    /// Guest member ID
    pub member_id: String,

    /// Access expiration
    pub expires_at: DateTime<Utc>,

    /// Restricted resources
    pub allowed_resources: Vec<String>,

    /// Access granted by
    pub granted_by: String,

    /// Grant reason
    pub reason: Option<String>,
}

/// Member activity tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberActivity {
    /// Member ID
    pub member_id: String,

    /// Last login
    pub last_login: Option<DateTime<Utc>>,

    /// Total actions performed
    pub total_actions: u64,

    /// Actions this week
    pub weekly_actions: u64,

    /// Actions today
    pub daily_actions: u64,

    /// Last action timestamp
    pub last_action_at: Option<DateTime<Utc>>,

    /// Last action type
    pub last_action_type: Option<String>,
}

// ============================================================================
// Member Manager
// ============================================================================

/// Manages team members and invitations
#[derive(Debug)]
pub struct MemberManager {
    members: HashMap<String, Member>,
    invitations: HashMap<String, MemberInvitation>,
    activities: HashMap<String, MemberActivity>,
    guest_access: HashMap<String, GuestAccess>,
    hierarchy: HashMap<String, TeamHierarchy>,
    workspace_index: HashMap<String, Vec<String>>, // workspace_id -> member_ids
}

impl MemberManager {
    /// Create a new member manager
    pub fn new() -> Self {
        Self {
            members: HashMap::new(),
            invitations: HashMap::new(),
            activities: HashMap::new(),
            guest_access: HashMap::new(),
            hierarchy: HashMap::new(),
            workspace_index: HashMap::new(),
        }
    }

    /// Invite a member to workspace
    pub fn invite_member(
        &mut self,
        workspace_id: &str,
        user_id: &str,
        email: &str,
        role: MemberRole,
        invited_by: &str,
    ) -> MemberResult<MemberInvitation> {
        // Check if member already exists
        if self.is_member(workspace_id, user_id) {
            return Err(MemberError::AlreadyExists(user_id.to_string()));
        }

        let invitation = MemberInvitation {
            id: Uuid::new_v4().to_string(),
            workspace_id: workspace_id.to_string(),
            user_id: user_id.to_string(),
            email: email.to_string(),
            role,
            token: Uuid::new_v4().to_string(),
            invited_by: invited_by.to_string(),
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::days(7),
            status: InvitationStatus::Pending,
            message: None,
        };

        self.invitations.insert(invitation.id.clone(), invitation.clone());

        Ok(invitation)
    }

    /// Accept invitation and add member
    pub fn accept_invitation(&mut self, invitation_id: &str) -> MemberResult<Member> {
        let invitation = self
            .invitations
            .get_mut(invitation_id)
            .ok_or_else(|| MemberError::InvalidInvitation(invitation_id.to_string()))?;

        if invitation.status != InvitationStatus::Pending {
            return Err(MemberError::InvalidInvitation(
                "Invitation already processed".to_string(),
            ));
        }

        if invitation.expires_at < Utc::now() {
            invitation.status = InvitationStatus::Expired;
            return Err(MemberError::InvitationExpired(invitation_id.to_string()));
        }

        invitation.status = InvitationStatus::Accepted;

        let member = self.add_member(
            invitation.workspace_id.clone(),
            invitation.user_id.clone(),
            invitation.email.clone(),
            invitation.role,
        )?;

        Ok(member)
    }

    /// Add member directly (bypassing invitation)
    pub fn add_member(
        &mut self,
        workspace_id: String,
        user_id: String,
        email: String,
        role: MemberRole,
    ) -> MemberResult<Member> {
        let member = Member {
            id: Uuid::new_v4().to_string(),
            workspace_id: workspace_id.clone(),
            user_id,
            email,
            display_name: None,
            role,
            status: MemberStatus::Active,
            joined_at: Utc::now(),
            last_active_at: None,
            invitation_id: None,
            manager_id: None,
            team: None,
            metadata: HashMap::new(),
        };

        self.members.insert(member.id.clone(), member.clone());

        // Update workspace index
        self.workspace_index
            .entry(workspace_id)
            .or_insert_with(Vec::new)
            .push(member.id.clone());

        // Initialize activity tracking
        self.activities.insert(
            member.id.clone(),
            MemberActivity {
                member_id: member.id.clone(),
                last_login: None,
                total_actions: 0,
                weekly_actions: 0,
                daily_actions: 0,
                last_action_at: None,
                last_action_type: None,
            },
        );

        Ok(member)
    }

    /// Remove member from workspace
    pub fn remove_member(&mut self, workspace_id: &str, member_id: &str) -> MemberResult<()> {
        let member = self
            .members
            .get(member_id)
            .ok_or_else(|| MemberError::NotFound(member_id.to_string()))?;

        // Check if this is the last owner
        if member.role == MemberRole::Owner {
            let owner_count = self
                .list_workspace_members(workspace_id)?
                .iter()
                .filter(|m| m.role == MemberRole::Owner && m.id != member_id)
                .count();

            if owner_count == 0 {
                return Err(MemberError::CannotRemoveLastOwner);
            }
        }

        self.members.remove(member_id);

        // Update workspace index
        if let Some(members) = self.workspace_index.get_mut(workspace_id) {
            members.retain(|id| id != member_id);
        }

        Ok(())
    }

    /// Update member role
    pub fn update_member_role(
        &mut self,
        member_id: &str,
        new_role: MemberRole,
        updated_by: &str,
    ) -> MemberResult<()> {
        let member = self
            .members
            .get_mut(member_id)
            .ok_or_else(|| MemberError::NotFound(member_id.to_string()))?;

        // Get updater's role to check permissions
        let updater = self
            .members
            .values()
            .find(|m| m.user_id == updated_by && m.workspace_id == member.workspace_id)
            .ok_or_else(|| MemberError::PermissionDenied("Updater not found".to_string()))?;

        if !updater.role.can_manage(&member.role) {
            return Err(MemberError::PermissionDenied(
                "Insufficient privileges".to_string(),
            ));
        }

        member.role = new_role;

        Ok(())
    }

    /// Set member hierarchy
    pub fn set_manager(&mut self, member_id: &str, manager_id: Option<String>) -> MemberResult<()> {
        let member = self
            .members
            .get_mut(member_id)
            .ok_or_else(|| MemberError::NotFound(member_id.to_string()))?;

        member.manager_id = manager_id;

        // Update hierarchy
        self.update_hierarchy(member_id)?;

        Ok(())
    }

    /// Update team hierarchy
    fn update_hierarchy(&mut self, member_id: &str) -> MemberResult<()> {
        let member = self.members.get(member_id).unwrap();

        let hierarchy = TeamHierarchy {
            member_id: member_id.to_string(),
            reports: self.get_direct_reports(member_id),
            manager_id: member.manager_id.clone(),
            level: self.calculate_hierarchy_level(member_id),
        };

        self.hierarchy.insert(member_id.to_string(), hierarchy);

        Ok(())
    }

    /// Get direct reports for a member
    fn get_direct_reports(&self, member_id: &str) -> Vec<String> {
        self.members
            .values()
            .filter(|m| m.manager_id.as_ref() == Some(&member_id.to_string()))
            .map(|m| m.id.clone())
            .collect()
    }

    /// Calculate hierarchy level
    fn calculate_hierarchy_level(&self, member_id: &str) -> u8 {
        let member = match self.members.get(member_id) {
            Some(m) => m,
            None => return 0,
        };

        if let Some(manager_id) = &member.manager_id {
            self.calculate_hierarchy_level(manager_id) + 1
        } else {
            0
        }
    }

    /// Grant guest access
    pub fn grant_guest_access(
        &mut self,
        member_id: &str,
        duration_days: i64,
        allowed_resources: Vec<String>,
        granted_by: &str,
    ) -> MemberResult<()> {
        let member = self
            .members
            .get_mut(member_id)
            .ok_or_else(|| MemberError::NotFound(member_id.to_string()))?;

        member.role = MemberRole::Guest;

        let guest_access = GuestAccess {
            member_id: member_id.to_string(),
            expires_at: Utc::now() + Duration::days(duration_days),
            allowed_resources,
            granted_by: granted_by.to_string(),
            reason: None,
        };

        self.guest_access.insert(member_id.to_string(), guest_access);

        Ok(())
    }

    /// Track member activity
    pub fn track_activity(&mut self, member_id: &str, action_type: &str) -> MemberResult<()> {
        let activity = self
            .activities
            .entry(member_id.to_string())
            .or_insert_with(|| MemberActivity {
                member_id: member_id.to_string(),
                last_login: None,
                total_actions: 0,
                weekly_actions: 0,
                daily_actions: 0,
                last_action_at: None,
                last_action_type: None,
            });

        activity.total_actions += 1;
        activity.weekly_actions += 1;
        activity.daily_actions += 1;
        activity.last_action_at = Some(Utc::now());
        activity.last_action_type = Some(action_type.to_string());

        // Update member last active
        if let Some(member) = self.members.get_mut(member_id) {
            member.last_active_at = Some(Utc::now());
        }

        Ok(())
    }

    /// List workspace members
    pub fn list_workspace_members(&self, workspace_id: &str) -> MemberResult<Vec<&Member>> {
        Ok(self
            .members
            .values()
            .filter(|m| m.workspace_id == workspace_id)
            .collect())
    }

    /// Check if user is a member
    pub fn is_member(&self, workspace_id: &str, user_id: &str) -> bool {
        self.members
            .values()
            .any(|m| m.workspace_id == workspace_id && m.user_id == user_id)
    }

    /// Get member by user ID
    pub fn get_member_by_user_id(&self, workspace_id: &str, user_id: &str) -> MemberResult<&Member> {
        self.members
            .values()
            .find(|m| m.workspace_id == workspace_id && m.user_id == user_id)
            .ok_or_else(|| MemberError::NotFound(user_id.to_string()))
    }

    /// Get member activity
    pub fn get_member_activity(&self, member_id: &str) -> Option<&MemberActivity> {
        self.activities.get(member_id)
    }
}

impl Default for MemberManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_member_invitation() {
        let mut manager = MemberManager::new();

        let invitation = manager
            .invite_member(
                "workspace1",
                "user1",
                "user1@example.com",
                MemberRole::Developer,
                "admin",
            )
            .unwrap();

        assert_eq!(invitation.status, InvitationStatus::Pending);
        assert_eq!(invitation.role, MemberRole::Developer);
    }

    #[test]
    fn test_accept_invitation() {
        let mut manager = MemberManager::new();

        let invitation = manager
            .invite_member(
                "workspace1",
                "user1",
                "user1@example.com",
                MemberRole::Developer,
                "admin",
            )
            .unwrap();

        let member = manager.accept_invitation(&invitation.id).unwrap();

        assert_eq!(member.status, MemberStatus::Active);
        assert_eq!(member.role, MemberRole::Developer);
    }

    #[test]
    fn test_role_hierarchy() {
        assert!(MemberRole::Owner.can_manage(&MemberRole::Admin));
        assert!(MemberRole::Admin.can_manage(&MemberRole::Developer));
        assert!(!MemberRole::Developer.can_manage(&MemberRole::Admin));
    }

    #[test]
    fn test_permissions() {
        let owner_perms = MemberRole::Owner.permissions();
        let viewer_perms = MemberRole::Viewer.permissions();

        assert!(owner_perms.can_delete);
        assert!(!viewer_perms.can_delete);
        assert!(viewer_perms.can_read);
    }
}
