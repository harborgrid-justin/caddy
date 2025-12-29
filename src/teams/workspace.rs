//! Workspace Management Module
//!
//! Provides multi-workspace support with advanced features including:
//! - Workspace creation and configuration
//! - Workspace templates for quick setup
//! - Cross-workspace resource sharing
//! - Workspace archiving and restoration
//! - Access control and visibility settings

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum WorkspaceError {
    #[error("Workspace not found: {0}")]
    NotFound(String),

    #[error("Workspace already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid workspace configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Workspace is archived: {0}")]
    Archived(String),

    #[error("Invalid template: {0}")]
    InvalidTemplate(String),

    #[error("Slug already in use: {0}")]
    SlugTaken(String),
}

pub type WorkspaceResult<T> = Result<T, WorkspaceError>;

// ============================================================================
// Core Types
// ============================================================================

/// Workspace visibility level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkspaceVisibility {
    /// Only members can see and access
    Private,

    /// Anyone can see, only members can access
    Internal,

    /// Anyone can see and request access
    Public,
}

/// Workspace status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkspaceStatus {
    /// Workspace is active
    Active,

    /// Workspace is archived
    Archived,

    /// Workspace is being set up
    Provisioning,

    /// Workspace is suspended
    Suspended,
}

/// Main workspace structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    /// Unique workspace identifier
    pub id: String,

    /// Workspace name
    pub name: String,

    /// URL-friendly slug
    pub slug: String,

    /// Workspace description
    pub description: Option<String>,

    /// Owner user ID
    pub owner_id: String,

    /// Workspace status
    pub status: WorkspaceStatus,

    /// Visibility level
    pub visibility: WorkspaceVisibility,

    /// Workspace settings
    pub settings: WorkspaceSettings,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Archive information (if archived)
    pub archive_info: Option<ArchiveInfo>,

    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

/// Workspace settings and preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSettings {
    /// Default issue assignment policy
    pub auto_assignment_enabled: bool,

    /// Require approval for new members
    pub require_member_approval: bool,

    /// Allow guest access
    pub allow_guest_access: bool,

    /// Default member role for new joiners
    pub default_member_role: String,

    /// Enable activity tracking
    pub activity_tracking_enabled: bool,

    /// Enable email notifications
    pub email_notifications_enabled: bool,

    /// Enable Slack integration
    pub slack_integration_enabled: bool,

    /// Maximum members allowed
    pub max_members: Option<usize>,

    /// Custom notification settings
    pub notification_settings: HashMap<String, bool>,

    /// Workspace timezone
    pub timezone: String,

    /// Working hours (for SLA calculation)
    pub working_hours: WorkingHours,
}

impl Default for WorkspaceSettings {
    fn default() -> Self {
        Self {
            auto_assignment_enabled: false,
            require_member_approval: true,
            allow_guest_access: false,
            default_member_role: "viewer".to_string(),
            activity_tracking_enabled: true,
            email_notifications_enabled: true,
            slack_integration_enabled: false,
            max_members: None,
            notification_settings: HashMap::new(),
            timezone: "UTC".to_string(),
            working_hours: WorkingHours::default(),
        }
    }
}

/// Working hours configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingHours {
    /// Start hour (0-23)
    pub start_hour: u8,

    /// End hour (0-23)
    pub end_hour: u8,

    /// Working days (0 = Sunday, 6 = Saturday)
    pub working_days: Vec<u8>,
}

impl Default for WorkingHours {
    fn default() -> Self {
        Self {
            start_hour: 9,
            end_hour: 17,
            working_days: vec![1, 2, 3, 4, 5], // Monday to Friday
        }
    }
}

/// Archive information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveInfo {
    /// When the workspace was archived
    pub archived_at: DateTime<Utc>,

    /// Who archived the workspace
    pub archived_by: String,

    /// Reason for archiving
    pub reason: Option<String>,

    /// Auto-delete date (if set)
    pub auto_delete_at: Option<DateTime<Utc>>,
}

/// Workspace template for quick creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceTemplate {
    /// Template ID
    pub id: String,

    /// Template name
    pub name: String,

    /// Template description
    pub description: String,

    /// Pre-configured settings
    pub settings: WorkspaceSettings,

    /// Default member roles to create
    pub default_roles: Vec<String>,

    /// Template metadata
    pub metadata: HashMap<String, String>,

    /// Is this a built-in template?
    pub is_builtin: bool,
}

/// Shared workspace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedWorkspace {
    /// Source workspace ID
    pub workspace_id: String,

    /// Target workspace ID
    pub shared_with_workspace_id: String,

    /// Resources being shared
    pub shared_resources: Vec<String>,

    /// Sharing permissions
    pub permissions: Vec<String>,

    /// When sharing was established
    pub shared_at: DateTime<Utc>,

    /// Who initiated the sharing
    pub shared_by: String,
}

// ============================================================================
// Workspace Manager
// ============================================================================

/// Manages all workspace operations
#[derive(Debug)]
pub struct WorkspaceManager {
    workspaces: HashMap<String, Workspace>,
    slug_index: HashMap<String, String>, // slug -> workspace_id
    templates: HashMap<String, WorkspaceTemplate>,
    shared_workspaces: Vec<SharedWorkspace>,
}

impl WorkspaceManager {
    /// Create a new workspace manager
    pub fn new() -> Self {
        let mut manager = Self {
            workspaces: HashMap::new(),
            slug_index: HashMap::new(),
            templates: HashMap::new(),
            shared_workspaces: Vec::new(),
        };

        // Register built-in templates
        manager.register_builtin_templates();

        manager
    }

    /// Create a new workspace
    pub fn create_workspace(
        &mut self,
        name: String,
        slug: String,
        owner_id: String,
    ) -> WorkspaceResult<Workspace> {
        // Validate slug uniqueness
        if self.slug_index.contains_key(&slug) {
            return Err(WorkspaceError::SlugTaken(slug));
        }

        let workspace = Workspace {
            id: Uuid::new_v4().to_string(),
            name,
            slug: slug.clone(),
            description: None,
            owner_id,
            status: WorkspaceStatus::Active,
            visibility: WorkspaceVisibility::Private,
            settings: WorkspaceSettings::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            archive_info: None,
            metadata: HashMap::new(),
        };

        self.slug_index.insert(slug, workspace.id.clone());
        self.workspaces.insert(workspace.id.clone(), workspace.clone());

        Ok(workspace)
    }

    /// Create workspace from template
    pub fn create_from_template(
        &mut self,
        template_id: &str,
        name: String,
        slug: String,
        owner_id: String,
    ) -> WorkspaceResult<Workspace> {
        let template = self
            .templates
            .get(template_id)
            .ok_or_else(|| WorkspaceError::InvalidTemplate(template_id.to_string()))?
            .clone();

        let mut workspace = self.create_workspace(name, slug, owner_id)?;
        workspace.settings = template.settings;
        workspace.metadata = template.metadata;

        self.workspaces.insert(workspace.id.clone(), workspace.clone());

        Ok(workspace)
    }

    /// Get workspace by ID
    pub fn get_workspace(&self, workspace_id: &str) -> WorkspaceResult<&Workspace> {
        self.workspaces
            .get(workspace_id)
            .ok_or_else(|| WorkspaceError::NotFound(workspace_id.to_string()))
    }

    /// Get mutable workspace by ID
    pub fn get_workspace_mut(&mut self, workspace_id: &str) -> WorkspaceResult<&mut Workspace> {
        self.workspaces
            .get_mut(workspace_id)
            .ok_or_else(|| WorkspaceError::NotFound(workspace_id.to_string()))
    }

    /// Get workspace by slug
    pub fn get_workspace_by_slug(&self, slug: &str) -> WorkspaceResult<&Workspace> {
        let workspace_id = self
            .slug_index
            .get(slug)
            .ok_or_else(|| WorkspaceError::NotFound(slug.to_string()))?;

        self.get_workspace(workspace_id)
    }

    /// Update workspace settings
    pub fn update_settings(
        &mut self,
        workspace_id: &str,
        settings: WorkspaceSettings,
    ) -> WorkspaceResult<()> {
        let workspace = self.get_workspace_mut(workspace_id)?;

        if workspace.status == WorkspaceStatus::Archived {
            return Err(WorkspaceError::Archived(workspace_id.to_string()));
        }

        workspace.settings = settings;
        workspace.updated_at = Utc::now();

        Ok(())
    }

    /// Archive workspace
    pub fn archive_workspace(
        &mut self,
        workspace_id: &str,
        archived_by: String,
        reason: Option<String>,
    ) -> WorkspaceResult<()> {
        let workspace = self.get_workspace_mut(workspace_id)?;

        workspace.status = WorkspaceStatus::Archived;
        workspace.archive_info = Some(ArchiveInfo {
            archived_at: Utc::now(),
            archived_by,
            reason,
            auto_delete_at: None,
        });
        workspace.updated_at = Utc::now();

        Ok(())
    }

    /// Restore archived workspace
    pub fn restore_workspace(&mut self, workspace_id: &str) -> WorkspaceResult<()> {
        let workspace = self.get_workspace_mut(workspace_id)?;

        if workspace.status != WorkspaceStatus::Archived {
            return Err(WorkspaceError::InvalidConfiguration(
                "Workspace is not archived".to_string(),
            ));
        }

        workspace.status = WorkspaceStatus::Active;
        workspace.archive_info = None;
        workspace.updated_at = Utc::now();

        Ok(())
    }

    /// Share workspace with another workspace
    pub fn share_workspace(
        &mut self,
        workspace_id: &str,
        target_workspace_id: &str,
        resources: Vec<String>,
        permissions: Vec<String>,
        shared_by: String,
    ) -> WorkspaceResult<()> {
        // Validate both workspaces exist
        self.get_workspace(workspace_id)?;
        self.get_workspace(target_workspace_id)?;

        let shared = SharedWorkspace {
            workspace_id: workspace_id.to_string(),
            shared_with_workspace_id: target_workspace_id.to_string(),
            shared_resources: resources,
            permissions,
            shared_at: Utc::now(),
            shared_by,
        };

        self.shared_workspaces.push(shared);

        Ok(())
    }

    /// List workspaces for a user
    pub fn list_user_workspaces(&self, user_id: &str) -> Vec<&Workspace> {
        self.workspaces
            .values()
            .filter(|w| {
                w.owner_id == user_id && w.status != WorkspaceStatus::Archived
            })
            .collect()
    }

    /// List all active workspaces
    pub fn list_active_workspaces(&self) -> Vec<&Workspace> {
        self.workspaces
            .values()
            .filter(|w| w.status == WorkspaceStatus::Active)
            .collect()
    }

    /// Register a workspace template
    pub fn register_template(&mut self, template: WorkspaceTemplate) -> WorkspaceResult<()> {
        if self.templates.contains_key(&template.id) {
            return Err(WorkspaceError::AlreadyExists(template.id.clone()));
        }

        self.templates.insert(template.id.clone(), template);
        Ok(())
    }

    /// List available templates
    pub fn list_templates(&self) -> Vec<&WorkspaceTemplate> {
        self.templates.values().collect()
    }

    /// Register built-in templates
    fn register_builtin_templates(&mut self) {
        // Software development template
        let dev_template = WorkspaceTemplate {
            id: "template-dev".to_string(),
            name: "Software Development".to_string(),
            description: "Optimized for software development teams".to_string(),
            settings: WorkspaceSettings {
                auto_assignment_enabled: true,
                require_member_approval: false,
                allow_guest_access: true,
                default_member_role: "developer".to_string(),
                activity_tracking_enabled: true,
                email_notifications_enabled: true,
                slack_integration_enabled: false,
                max_members: Some(50),
                notification_settings: HashMap::new(),
                timezone: "UTC".to_string(),
                working_hours: WorkingHours::default(),
            },
            default_roles: vec![
                "developer".to_string(),
                "reviewer".to_string(),
                "tester".to_string(),
            ],
            metadata: HashMap::new(),
            is_builtin: true,
        };

        // Design team template
        let design_template = WorkspaceTemplate {
            id: "template-design".to_string(),
            name: "Design Team".to_string(),
            description: "Optimized for design collaboration".to_string(),
            settings: WorkspaceSettings {
                auto_assignment_enabled: false,
                require_member_approval: true,
                allow_guest_access: true,
                default_member_role: "designer".to_string(),
                activity_tracking_enabled: true,
                email_notifications_enabled: true,
                slack_integration_enabled: false,
                max_members: Some(25),
                notification_settings: HashMap::new(),
                timezone: "UTC".to_string(),
                working_hours: WorkingHours::default(),
            },
            default_roles: vec![
                "designer".to_string(),
                "reviewer".to_string(),
            ],
            metadata: HashMap::new(),
            is_builtin: true,
        };

        // Enterprise template
        let enterprise_template = WorkspaceTemplate {
            id: "template-enterprise".to_string(),
            name: "Enterprise".to_string(),
            description: "Full-featured enterprise workspace".to_string(),
            settings: WorkspaceSettings {
                auto_assignment_enabled: true,
                require_member_approval: true,
                allow_guest_access: false,
                default_member_role: "viewer".to_string(),
                activity_tracking_enabled: true,
                email_notifications_enabled: true,
                slack_integration_enabled: true,
                max_members: None,
                notification_settings: HashMap::new(),
                timezone: "UTC".to_string(),
                working_hours: WorkingHours::default(),
            },
            default_roles: vec![
                "admin".to_string(),
                "manager".to_string(),
                "developer".to_string(),
                "viewer".to_string(),
            ],
            metadata: HashMap::new(),
            is_builtin: true,
        };

        self.templates.insert(dev_template.id.clone(), dev_template);
        self.templates.insert(design_template.id.clone(), design_template);
        self.templates.insert(enterprise_template.id.clone(), enterprise_template);
    }
}

impl Default for WorkspaceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_workspace() {
        let mut manager = WorkspaceManager::new();

        let workspace = manager
            .create_workspace(
                "Test Workspace".to_string(),
                "test-workspace".to_string(),
                "user123".to_string(),
            )
            .unwrap();

        assert_eq!(workspace.name, "Test Workspace");
        assert_eq!(workspace.slug, "test-workspace");
        assert_eq!(workspace.status, WorkspaceStatus::Active);
    }

    #[test]
    fn test_slug_uniqueness() {
        let mut manager = WorkspaceManager::new();

        manager
            .create_workspace(
                "First".to_string(),
                "same-slug".to_string(),
                "user1".to_string(),
            )
            .unwrap();

        let result = manager.create_workspace(
            "Second".to_string(),
            "same-slug".to_string(),
            "user2".to_string(),
        );

        assert!(matches!(result, Err(WorkspaceError::SlugTaken(_))));
    }

    #[test]
    fn test_archive_restore() {
        let mut manager = WorkspaceManager::new();

        let workspace = manager
            .create_workspace(
                "Test".to_string(),
                "test".to_string(),
                "user1".to_string(),
            )
            .unwrap();

        manager
            .archive_workspace(&workspace.id, "user1".to_string(), Some("Testing".to_string()))
            .unwrap();

        let archived = manager.get_workspace(&workspace.id).unwrap();
        assert_eq!(archived.status, WorkspaceStatus::Archived);

        manager.restore_workspace(&workspace.id).unwrap();

        let restored = manager.get_workspace(&workspace.id).unwrap();
        assert_eq!(restored.status, WorkspaceStatus::Active);
    }

    #[test]
    fn test_templates() {
        let manager = WorkspaceManager::new();
        let templates = manager.list_templates();

        assert!(templates.len() >= 3); // At least 3 built-in templates
        assert!(templates.iter().any(|t| t.id == "template-dev"));
    }

    #[test]
    fn test_create_from_template() {
        let mut manager = WorkspaceManager::new();

        let workspace = manager
            .create_from_template(
                "template-dev",
                "Dev Team".to_string(),
                "dev-team".to_string(),
                "user1".to_string(),
            )
            .unwrap();

        assert!(workspace.settings.auto_assignment_enabled);
        assert_eq!(workspace.settings.default_member_role, "developer");
    }
}
