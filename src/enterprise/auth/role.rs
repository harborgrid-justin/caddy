//! Role-based access control (RBAC) system for CADDY.
//!
//! This module provides hierarchical role management with:
//! - Built-in roles with predefined permissions
//! - Custom role creation
//! - Role hierarchy and inheritance
//! - Permission aggregation

use super::permission::{Permission, PermissionSet};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during role operations
#[derive(Error, Debug)]
pub enum RoleError {
    #[error("Role not found: {0}")]
    NotFound(String),

    #[error("Invalid role hierarchy: {0}")]
    InvalidHierarchy(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Role already exists: {0}")]
    AlreadyExists(String),

    #[error("Cannot modify built-in role: {0}")]
    CannotModifyBuiltIn(String),
}

/// Result type for role operations
pub type RoleResult<T> = Result<T, RoleError>;

/// Built-in system roles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuiltInRole {
    SuperAdmin,
    Admin,
    Designer,
    Viewer,
    Guest,
}

impl BuiltInRole {
    /// Get the string representation of the role
    pub fn as_str(&self) -> &'static str {
        match self {
            BuiltInRole::SuperAdmin => "SuperAdmin",
            BuiltInRole::Admin => "Admin",
            BuiltInRole::Designer => "Designer",
            BuiltInRole::Viewer => "Viewer",
            BuiltInRole::Guest => "Guest",
        }
    }

    /// Get the hierarchy level (higher = more privileged)
    pub fn hierarchy_level(&self) -> u8 {
        match self {
            BuiltInRole::SuperAdmin => 100,
            BuiltInRole::Admin => 80,
            BuiltInRole::Designer => 50,
            BuiltInRole::Viewer => 30,
            BuiltInRole::Guest => 10,
        }
    }

    /// Get the description of the role
    pub fn description(&self) -> &'static str {
        match self {
            BuiltInRole::SuperAdmin => "Full system access with all permissions",
            BuiltInRole::Admin => "Administrative access for user and system management",
            BuiltInRole::Designer => "Full CAD design capabilities",
            BuiltInRole::Viewer => "Read-only access to drawings and projects",
            BuiltInRole::Guest => "Limited guest access",
        }
    }

    /// Get the default permissions for this role
    pub fn default_permissions(&self) -> PermissionSet {
        match self {
            BuiltInRole::SuperAdmin => {
                // SuperAdmin gets all permissions
                PermissionSet::from_vec(vec![Permission::SuperAdmin])
            }
            BuiltInRole::Admin => {
                PermissionSet::from_vec(vec![
                    // User management
                    Permission::UserCreate,
                    Permission::UserRead,
                    Permission::UserUpdate,
                    Permission::UserDelete,
                    Permission::UserManageRoles,
                    // Role management
                    Permission::RoleCreate,
                    Permission::RoleRead,
                    Permission::RoleUpdate,
                    Permission::RoleDelete,
                    Permission::RoleAssignPermissions,
                    // Project management
                    Permission::ProjectCreate,
                    Permission::ProjectRead,
                    Permission::ProjectUpdate,
                    Permission::ProjectDelete,
                    Permission::ProjectArchive,
                    // System
                    Permission::SystemConfigure,
                    Permission::SystemMonitor,
                    Permission::SystemBackup,
                    Permission::SystemAudit,
                    // Plugins
                    Permission::PluginInstall,
                    Permission::PluginConfigure,
                    Permission::PluginUninstall,
                    // All drawing permissions
                    Permission::DrawingCreate,
                    Permission::DrawingRead,
                    Permission::DrawingUpdate,
                    Permission::DrawingDelete,
                    Permission::DrawingExport,
                    Permission::DrawingImport,
                    Permission::DrawingShare,
                ])
            }
            BuiltInRole::Designer => {
                PermissionSet::from_vec(vec![
                    // Full CAD operations
                    Permission::DrawingCreate,
                    Permission::DrawingRead,
                    Permission::DrawingUpdate,
                    Permission::DrawingDelete,
                    Permission::DrawingExport,
                    Permission::DrawingImport,
                    Permission::DrawingShare,
                    // Geometry
                    Permission::GeometryCreate,
                    Permission::GeometryModify,
                    Permission::GeometryDelete,
                    Permission::GeometryAnalyze,
                    // Layers
                    Permission::LayerCreate,
                    Permission::LayerModify,
                    Permission::LayerDelete,
                    Permission::LayerReorder,
                    // Dimensions
                    Permission::DimensionCreate,
                    Permission::DimensionModify,
                    Permission::DimensionDelete,
                    // Constraints
                    Permission::ConstraintCreate,
                    Permission::ConstraintModify,
                    Permission::ConstraintDelete,
                    Permission::ConstraintSolve,
                    // Projects
                    Permission::ProjectCreate,
                    Permission::ProjectRead,
                    Permission::ProjectUpdate,
                    Permission::ProjectArchive,
                    // Rendering
                    Permission::RenderingExecute,
                    Permission::RenderingConfigure,
                    // Export/Import
                    Permission::ExportDXF,
                    Permission::ExportSTEP,
                    Permission::ExportIGES,
                    Permission::ExportSTL,
                    Permission::ExportPDF,
                    Permission::ImportDXF,
                    Permission::ImportSTEP,
                    Permission::ImportIGES,
                    // Collaboration
                    Permission::CollaborationInvite,
                    Permission::CollaborationComment,
                    Permission::CollaborationReview,
                ])
            }
            BuiltInRole::Viewer => {
                PermissionSet::from_vec(vec![
                    // Read-only access
                    Permission::DrawingRead,
                    Permission::ProjectRead,
                    Permission::GeometryAnalyze,
                    Permission::DrawingExport,
                    Permission::ExportPDF,
                    Permission::CollaborationComment,
                ])
            }
            BuiltInRole::Guest => {
                PermissionSet::from_vec(vec![
                    // Very limited access
                    Permission::DrawingRead,
                    Permission::ProjectRead,
                ])
            }
        }
    }
}

/// Represents a role in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Unique identifier for the role
    pub id: String,

    /// Role name
    pub name: String,

    /// Role description
    pub description: String,

    /// Permissions granted by this role
    pub permissions: PermissionSet,

    /// Hierarchy level (0-100, higher = more privileged)
    pub hierarchy_level: u8,

    /// Whether this is a built-in role (cannot be deleted)
    pub is_built_in: bool,

    /// Parent role for inheritance (optional)
    pub parent_role: Option<String>,

    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

impl Role {
    /// Create a new custom role
    pub fn new(
        id: String,
        name: String,
        description: String,
        hierarchy_level: u8,
    ) -> Self {
        Self {
            id,
            name,
            description,
            permissions: PermissionSet::new(),
            hierarchy_level,
            is_built_in: false,
            parent_role: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a built-in role
    pub fn built_in(built_in_role: BuiltInRole) -> Self {
        Self {
            id: built_in_role.as_str().to_lowercase(),
            name: built_in_role.as_str().to_string(),
            description: built_in_role.description().to_string(),
            permissions: built_in_role.default_permissions(),
            hierarchy_level: built_in_role.hierarchy_level(),
            is_built_in: true,
            parent_role: None,
            metadata: HashMap::new(),
        }
    }

    /// Add a permission to the role
    pub fn add_permission(&mut self, permission: Permission) -> RoleResult<()> {
        if self.is_built_in {
            return Err(RoleError::CannotModifyBuiltIn(self.name.clone()));
        }
        self.permissions.add(permission);
        Ok(())
    }

    /// Remove a permission from the role
    pub fn remove_permission(&mut self, permission: &Permission) -> RoleResult<()> {
        if self.is_built_in {
            return Err(RoleError::CannotModifyBuiltIn(self.name.clone()));
        }
        self.permissions.remove(permission);
        Ok(())
    }

    /// Check if the role has a specific permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.has(permission)
    }

    /// Set parent role for inheritance
    pub fn set_parent(&mut self, parent_id: String) -> RoleResult<()> {
        if self.is_built_in {
            return Err(RoleError::CannotModifyBuiltIn(self.name.clone()));
        }
        self.parent_role = Some(parent_id);
        Ok(())
    }

    /// Get effective permissions including inherited ones
    pub fn effective_permissions(&self, role_manager: &RoleManager) -> PermissionSet {
        let mut effective = self.permissions.clone();

        if let Some(parent_id) = &self.parent_role {
            if let Ok(parent) = role_manager.get_role(parent_id) {
                let parent_perms = parent.effective_permissions(role_manager);
                effective.merge(&parent_perms);
            }
        }

        effective
    }

    /// Check if this role can manage another role (based on hierarchy)
    pub fn can_manage(&self, other: &Role) -> bool {
        self.hierarchy_level > other.hierarchy_level
    }
}

/// Manages roles in the system
#[derive(Debug)]
pub struct RoleManager {
    roles: HashMap<String, Role>,
}

impl RoleManager {
    /// Create a new role manager with built-in roles
    pub fn new() -> Self {
        let mut manager = Self {
            roles: HashMap::new(),
        };

        // Initialize built-in roles
        manager.add_role(Role::built_in(BuiltInRole::SuperAdmin)).ok();
        manager.add_role(Role::built_in(BuiltInRole::Admin)).ok();
        manager.add_role(Role::built_in(BuiltInRole::Designer)).ok();
        manager.add_role(Role::built_in(BuiltInRole::Viewer)).ok();
        manager.add_role(Role::built_in(BuiltInRole::Guest)).ok();

        manager
    }

    /// Add a new role
    pub fn add_role(&mut self, role: Role) -> RoleResult<()> {
        if self.roles.contains_key(&role.id) {
            return Err(RoleError::AlreadyExists(role.id.clone()));
        }
        self.roles.insert(role.id.clone(), role);
        Ok(())
    }

    /// Get a role by ID
    pub fn get_role(&self, id: &str) -> RoleResult<&Role> {
        self.roles
            .get(id)
            .ok_or_else(|| RoleError::NotFound(id.to_string()))
    }

    /// Get a mutable reference to a role
    pub fn get_role_mut(&mut self, id: &str) -> RoleResult<&mut Role> {
        self.roles
            .get_mut(id)
            .ok_or_else(|| RoleError::NotFound(id.to_string()))
    }

    /// Delete a role
    pub fn delete_role(&mut self, id: &str) -> RoleResult<()> {
        let role = self.get_role(id)?;
        if role.is_built_in {
            return Err(RoleError::CannotModifyBuiltIn(role.name.clone()));
        }
        self.roles.remove(id);
        Ok(())
    }

    /// List all roles
    pub fn list_roles(&self) -> Vec<&Role> {
        self.roles.values().collect()
    }

    /// List roles by hierarchy level
    pub fn list_roles_by_hierarchy(&self) -> Vec<&Role> {
        let mut roles: Vec<&Role> = self.roles.values().collect();
        roles.sort_by(|a, b| b.hierarchy_level.cmp(&a.hierarchy_level));
        roles
    }

    /// Check if a user with given roles has a permission
    pub fn has_permission(&self, role_ids: &[String], permission: &Permission) -> bool {
        role_ids.iter().any(|role_id| {
            if let Ok(role) = self.get_role(role_id) {
                role.effective_permissions(self).has(permission)
            } else {
                false
            }
        })
    }

    /// Get combined permissions for multiple roles
    pub fn combined_permissions(&self, role_ids: &[String]) -> PermissionSet {
        let mut combined = PermissionSet::new();

        for role_id in role_ids {
            if let Ok(role) = self.get_role(role_id) {
                combined.merge(&role.effective_permissions(self));
            }
        }

        combined
    }

    /// Get the highest hierarchy level among given roles
    pub fn max_hierarchy_level(&self, role_ids: &[String]) -> u8 {
        role_ids
            .iter()
            .filter_map(|id| self.get_role(id).ok())
            .map(|role| role.hierarchy_level)
            .max()
            .unwrap_or(0)
    }
}

impl Default for RoleManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_built_in_roles() {
        let super_admin = Role::built_in(BuiltInRole::SuperAdmin);
        assert!(super_admin.is_built_in);
        assert_eq!(super_admin.hierarchy_level, 100);
        assert!(super_admin.has_permission(&Permission::SuperAdmin));

        let designer = Role::built_in(BuiltInRole::Designer);
        assert!(designer.has_permission(&Permission::DrawingCreate));
        assert!(designer.has_permission(&Permission::GeometryCreate));
    }

    #[test]
    fn test_custom_role() {
        let mut role = Role::new(
            "custom".to_string(),
            "Custom Role".to_string(),
            "A custom role".to_string(),
            40,
        );

        assert!(!role.is_built_in);
        assert!(role.add_permission(Permission::DrawingRead).is_ok());
        assert!(role.has_permission(&Permission::DrawingRead));
    }

    #[test]
    fn test_cannot_modify_built_in() {
        let mut role = Role::built_in(BuiltInRole::Designer);
        let result = role.add_permission(Permission::SystemConfigure);
        assert!(result.is_err());
    }

    #[test]
    fn test_role_hierarchy() {
        let admin = Role::built_in(BuiltInRole::Admin);
        let designer = Role::built_in(BuiltInRole::Designer);
        let viewer = Role::built_in(BuiltInRole::Viewer);

        assert!(admin.can_manage(&designer));
        assert!(admin.can_manage(&viewer));
        assert!(designer.can_manage(&viewer));
        assert!(!viewer.can_manage(&designer));
    }

    #[test]
    fn test_role_manager() {
        let mut manager = RoleManager::new();

        // Check built-in roles are initialized
        assert!(manager.get_role("superadmin").is_ok());
        assert!(manager.get_role("admin").is_ok());
        assert!(manager.get_role("designer").is_ok());

        // Add custom role
        let custom = Role::new(
            "custom".to_string(),
            "Custom".to_string(),
            "Custom role".to_string(),
            45,
        );
        assert!(manager.add_role(custom).is_ok());
        assert!(manager.get_role("custom").is_ok());
    }

    #[test]
    fn test_combined_permissions() {
        let manager = RoleManager::new();
        let role_ids = vec!["designer".to_string(), "viewer".to_string()];

        let combined = manager.combined_permissions(&role_ids);

        // Should have designer permissions
        assert!(combined.has(&Permission::DrawingCreate));
        // Should have viewer permissions
        assert!(combined.has(&Permission::DrawingRead));
    }

    #[test]
    fn test_permission_checking() {
        let manager = RoleManager::new();
        let role_ids = vec!["designer".to_string()];

        assert!(manager.has_permission(&role_ids, &Permission::DrawingCreate));
        assert!(!manager.has_permission(&role_ids, &Permission::SystemConfigure));
    }

    #[test]
    fn test_role_inheritance() {
        let mut manager = RoleManager::new();

        let mut parent = Role::new(
            "parent".to_string(),
            "Parent".to_string(),
            "Parent role".to_string(),
            60,
        );
        parent.permissions.add(Permission::DrawingRead);
        manager.add_role(parent).ok();

        let mut child = Role::new(
            "child".to_string(),
            "Child".to_string(),
            "Child role".to_string(),
            50,
        );
        child.set_parent("parent".to_string()).ok();
        child.permissions.add(Permission::DrawingCreate);
        manager.add_role(child).ok();

        let child_role = manager.get_role("child").unwrap();
        let effective = child_role.effective_permissions(&manager);

        // Should have both child and parent permissions
        assert!(effective.has(&Permission::DrawingCreate));
        assert!(effective.has(&Permission::DrawingRead));
    }
}
