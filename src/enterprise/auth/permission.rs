//! Fine-grained permission system for CADDY enterprise CAD operations.
//!
//! This module provides a comprehensive permission model supporting:
//! - Granular CAD operation permissions
//! - Resource-level access control
//! - Efficient permission checking with bitflags
//! - Permission inheritance and composition

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

/// Represents all possible permissions in the CADDY system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    // Drawing permissions
    DrawingCreate,
    DrawingRead,
    DrawingUpdate,
    DrawingDelete,
    DrawingExport,
    DrawingImport,
    DrawingShare,

    // Geometry permissions
    GeometryCreate,
    GeometryModify,
    GeometryDelete,
    GeometryAnalyze,

    // Layer permissions
    LayerCreate,
    LayerModify,
    LayerDelete,
    LayerReorder,

    // Dimension permissions
    DimensionCreate,
    DimensionModify,
    DimensionDelete,

    // Constraint permissions
    ConstraintCreate,
    ConstraintModify,
    ConstraintDelete,
    ConstraintSolve,

    // Project permissions
    ProjectCreate,
    ProjectRead,
    ProjectUpdate,
    ProjectDelete,
    ProjectArchive,

    // User management permissions
    UserCreate,
    UserRead,
    UserUpdate,
    UserDelete,
    UserManageRoles,

    // Role management permissions
    RoleCreate,
    RoleRead,
    RoleUpdate,
    RoleDelete,
    RoleAssignPermissions,

    // System permissions
    SystemConfigure,
    SystemMonitor,
    SystemBackup,
    SystemRestore,
    SystemAudit,

    // Plugin permissions
    PluginInstall,
    PluginConfigure,
    PluginUninstall,
    PluginDevelop,

    // Rendering permissions
    RenderingExecute,
    RenderingConfigure,

    // Export/Import permissions
    ExportDXF,
    ExportSTEP,
    ExportIGES,
    ExportSTL,
    ExportPDF,
    ImportDXF,
    ImportSTEP,
    ImportIGES,

    // Collaboration permissions
    CollaborationInvite,
    CollaborationComment,
    CollaborationReview,
    CollaborationApprove,

    // Super admin permission
    SuperAdmin,
}

impl Permission {
    /// Get all available permissions
    pub fn all() -> Vec<Permission> {
        vec![
            // Drawing
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
            Permission::ProjectDelete,
            Permission::ProjectArchive,
            // Users
            Permission::UserCreate,
            Permission::UserRead,
            Permission::UserUpdate,
            Permission::UserDelete,
            Permission::UserManageRoles,
            // Roles
            Permission::RoleCreate,
            Permission::RoleRead,
            Permission::RoleUpdate,
            Permission::RoleDelete,
            Permission::RoleAssignPermissions,
            // System
            Permission::SystemConfigure,
            Permission::SystemMonitor,
            Permission::SystemBackup,
            Permission::SystemRestore,
            Permission::SystemAudit,
            // Plugins
            Permission::PluginInstall,
            Permission::PluginConfigure,
            Permission::PluginUninstall,
            Permission::PluginDevelop,
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
            Permission::CollaborationApprove,
            // SuperAdmin
            Permission::SuperAdmin,
        ]
    }

    /// Get human-readable description of the permission
    pub fn description(&self) -> &'static str {
        match self {
            Permission::DrawingCreate => "Create new drawings",
            Permission::DrawingRead => "View drawings",
            Permission::DrawingUpdate => "Modify existing drawings",
            Permission::DrawingDelete => "Delete drawings",
            Permission::DrawingExport => "Export drawings to files",
            Permission::DrawingImport => "Import drawings from files",
            Permission::DrawingShare => "Share drawings with others",

            Permission::GeometryCreate => "Create geometric entities",
            Permission::GeometryModify => "Modify geometric entities",
            Permission::GeometryDelete => "Delete geometric entities",
            Permission::GeometryAnalyze => "Analyze geometric properties",

            Permission::LayerCreate => "Create new layers",
            Permission::LayerModify => "Modify layer properties",
            Permission::LayerDelete => "Delete layers",
            Permission::LayerReorder => "Reorder layer hierarchy",

            Permission::DimensionCreate => "Create dimensions",
            Permission::DimensionModify => "Modify dimensions",
            Permission::DimensionDelete => "Delete dimensions",

            Permission::ConstraintCreate => "Create constraints",
            Permission::ConstraintModify => "Modify constraints",
            Permission::ConstraintDelete => "Delete constraints",
            Permission::ConstraintSolve => "Solve constraint systems",

            Permission::ProjectCreate => "Create new projects",
            Permission::ProjectRead => "View projects",
            Permission::ProjectUpdate => "Modify projects",
            Permission::ProjectDelete => "Delete projects",
            Permission::ProjectArchive => "Archive projects",

            Permission::UserCreate => "Create new users",
            Permission::UserRead => "View user information",
            Permission::UserUpdate => "Modify user information",
            Permission::UserDelete => "Delete users",
            Permission::UserManageRoles => "Assign roles to users",

            Permission::RoleCreate => "Create new roles",
            Permission::RoleRead => "View role information",
            Permission::RoleUpdate => "Modify roles",
            Permission::RoleDelete => "Delete roles",
            Permission::RoleAssignPermissions => "Assign permissions to roles",

            Permission::SystemConfigure => "Configure system settings",
            Permission::SystemMonitor => "Monitor system performance",
            Permission::SystemBackup => "Create system backups",
            Permission::SystemRestore => "Restore from backups",
            Permission::SystemAudit => "View audit logs",

            Permission::PluginInstall => "Install plugins",
            Permission::PluginConfigure => "Configure plugins",
            Permission::PluginUninstall => "Uninstall plugins",
            Permission::PluginDevelop => "Develop and debug plugins",

            Permission::RenderingExecute => "Execute rendering operations",
            Permission::RenderingConfigure => "Configure rendering settings",

            Permission::ExportDXF => "Export to DXF format",
            Permission::ExportSTEP => "Export to STEP format",
            Permission::ExportIGES => "Export to IGES format",
            Permission::ExportSTL => "Export to STL format",
            Permission::ExportPDF => "Export to PDF format",
            Permission::ImportDXF => "Import from DXF format",
            Permission::ImportSTEP => "Import from STEP format",
            Permission::ImportIGES => "Import from IGES format",

            Permission::CollaborationInvite => "Invite collaborators",
            Permission::CollaborationComment => "Add comments and annotations",
            Permission::CollaborationReview => "Review changes",
            Permission::CollaborationApprove => "Approve changes",

            Permission::SuperAdmin => "Full system access (supersedes all permissions)",
        }
    }

    /// Get the category of this permission
    pub fn category(&self) -> PermissionCategory {
        match self {
            Permission::DrawingCreate | Permission::DrawingRead | Permission::DrawingUpdate
            | Permission::DrawingDelete | Permission::DrawingExport | Permission::DrawingImport
            | Permission::DrawingShare => PermissionCategory::Drawing,

            Permission::GeometryCreate | Permission::GeometryModify | Permission::GeometryDelete
            | Permission::GeometryAnalyze => PermissionCategory::Geometry,

            Permission::LayerCreate | Permission::LayerModify | Permission::LayerDelete
            | Permission::LayerReorder => PermissionCategory::Layer,

            Permission::DimensionCreate | Permission::DimensionModify | Permission::DimensionDelete => {
                PermissionCategory::Dimension
            }

            Permission::ConstraintCreate | Permission::ConstraintModify | Permission::ConstraintDelete
            | Permission::ConstraintSolve => PermissionCategory::Constraint,

            Permission::ProjectCreate | Permission::ProjectRead | Permission::ProjectUpdate
            | Permission::ProjectDelete | Permission::ProjectArchive => PermissionCategory::Project,

            Permission::UserCreate | Permission::UserRead | Permission::UserUpdate
            | Permission::UserDelete | Permission::UserManageRoles => PermissionCategory::User,

            Permission::RoleCreate | Permission::RoleRead | Permission::RoleUpdate
            | Permission::RoleDelete | Permission::RoleAssignPermissions => PermissionCategory::Role,

            Permission::SystemConfigure | Permission::SystemMonitor | Permission::SystemBackup
            | Permission::SystemRestore | Permission::SystemAudit => PermissionCategory::System,

            Permission::PluginInstall | Permission::PluginConfigure | Permission::PluginUninstall
            | Permission::PluginDevelop => PermissionCategory::Plugin,

            Permission::RenderingExecute | Permission::RenderingConfigure => PermissionCategory::Rendering,

            Permission::ExportDXF | Permission::ExportSTEP | Permission::ExportIGES
            | Permission::ExportSTL | Permission::ExportPDF | Permission::ImportDXF
            | Permission::ImportSTEP | Permission::ImportIGES => PermissionCategory::ImportExport,

            Permission::CollaborationInvite | Permission::CollaborationComment
            | Permission::CollaborationReview | Permission::CollaborationApprove => {
                PermissionCategory::Collaboration
            }

            Permission::SuperAdmin => PermissionCategory::Admin,
        }
    }
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Permission categories for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionCategory {
    Drawing,
    Geometry,
    Layer,
    Dimension,
    Constraint,
    Project,
    User,
    Role,
    System,
    Plugin,
    Rendering,
    ImportExport,
    Collaboration,
    Admin,
}

/// Efficient permission set using HashSet for O(1) lookups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionSet {
    permissions: HashSet<Permission>,
}

impl PermissionSet {
    /// Create a new empty permission set
    pub fn new() -> Self {
        Self {
            permissions: HashSet::new(),
        }
    }

    /// Create a permission set from a vector of permissions
    pub fn from_vec(permissions: Vec<Permission>) -> Self {
        Self {
            permissions: permissions.into_iter().collect(),
        }
    }

    /// Create a permission set with all permissions
    pub fn all() -> Self {
        Self::from_vec(Permission::all())
    }

    /// Add a permission to the set
    pub fn add(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }

    /// Remove a permission from the set
    pub fn remove(&mut self, permission: &Permission) {
        self.permissions.remove(permission);
    }

    /// Check if the set contains a specific permission
    pub fn has(&self, permission: &Permission) -> bool {
        // SuperAdmin permission grants all permissions
        if self.permissions.contains(&Permission::SuperAdmin) {
            return true;
        }
        self.permissions.contains(permission)
    }

    /// Check if the set contains all of the given permissions
    pub fn has_all(&self, permissions: &[Permission]) -> bool {
        if self.permissions.contains(&Permission::SuperAdmin) {
            return true;
        }
        permissions.iter().all(|p| self.permissions.contains(p))
    }

    /// Check if the set contains any of the given permissions
    pub fn has_any(&self, permissions: &[Permission]) -> bool {
        if self.permissions.contains(&Permission::SuperAdmin) {
            return true;
        }
        permissions.iter().any(|p| self.permissions.contains(p))
    }

    /// Get all permissions in the set
    pub fn list(&self) -> Vec<Permission> {
        self.permissions.iter().copied().collect()
    }

    /// Merge another permission set into this one
    pub fn merge(&mut self, other: &PermissionSet) {
        self.permissions.extend(&other.permissions);
    }

    /// Create a new permission set that is the union of this and another
    pub fn union(&self, other: &PermissionSet) -> PermissionSet {
        let mut result = self.clone();
        result.merge(other);
        result
    }

    /// Check if this set is empty
    pub fn is_empty(&self) -> bool {
        self.permissions.is_empty()
    }

    /// Get the count of permissions
    pub fn count(&self) -> usize {
        self.permissions.len()
    }

    /// Get permissions by category
    pub fn by_category(&self, category: PermissionCategory) -> Vec<Permission> {
        self.permissions
            .iter()
            .filter(|p| p.category() == category)
            .copied()
            .collect()
    }
}

impl Default for PermissionSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Resource-level permission for fine-grained access control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePermission {
    /// Resource type (e.g., "drawing", "project")
    pub resource_type: String,
    /// Resource ID
    pub resource_id: String,
    /// Allowed permissions on this resource
    pub permissions: PermissionSet,
}

impl ResourcePermission {
    /// Create a new resource permission
    pub fn new(resource_type: String, resource_id: String, permissions: PermissionSet) -> Self {
        Self {
            resource_type,
            resource_id,
            permissions,
        }
    }

    /// Check if a permission is granted for this resource
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.has(permission)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_set_basic() {
        let mut perms = PermissionSet::new();
        assert!(perms.is_empty());

        perms.add(Permission::DrawingCreate);
        assert!(perms.has(&Permission::DrawingCreate));
        assert!(!perms.has(&Permission::DrawingDelete));
        assert_eq!(perms.count(), 1);
    }

    #[test]
    fn test_super_admin_grants_all() {
        let mut perms = PermissionSet::new();
        perms.add(Permission::SuperAdmin);

        // SuperAdmin should grant all permissions
        assert!(perms.has(&Permission::DrawingCreate));
        assert!(perms.has(&Permission::UserDelete));
        assert!(perms.has(&Permission::SystemConfigure));
    }

    #[test]
    fn test_permission_set_operations() {
        let mut set1 = PermissionSet::new();
        set1.add(Permission::DrawingCreate);
        set1.add(Permission::DrawingRead);

        let mut set2 = PermissionSet::new();
        set2.add(Permission::DrawingUpdate);
        set2.add(Permission::DrawingDelete);

        let merged = set1.union(&set2);
        assert_eq!(merged.count(), 4);
        assert!(merged.has(&Permission::DrawingCreate));
        assert!(merged.has(&Permission::DrawingDelete));
    }

    #[test]
    fn test_permission_has_all_any() {
        let mut perms = PermissionSet::new();
        perms.add(Permission::DrawingCreate);
        perms.add(Permission::DrawingRead);

        assert!(perms.has_all(&[Permission::DrawingCreate, Permission::DrawingRead]));
        assert!(!perms.has_all(&[Permission::DrawingCreate, Permission::DrawingDelete]));

        assert!(perms.has_any(&[Permission::DrawingCreate, Permission::DrawingDelete]));
        assert!(!perms.has_any(&[Permission::DrawingDelete, Permission::DrawingUpdate]));
    }

    #[test]
    fn test_permission_categories() {
        assert_eq!(Permission::DrawingCreate.category(), PermissionCategory::Drawing);
        assert_eq!(Permission::SystemConfigure.category(), PermissionCategory::System);
        assert_eq!(Permission::UserCreate.category(), PermissionCategory::User);
    }

    #[test]
    fn test_permission_by_category() {
        let mut perms = PermissionSet::new();
        perms.add(Permission::DrawingCreate);
        perms.add(Permission::DrawingRead);
        perms.add(Permission::UserCreate);

        let drawing_perms = perms.by_category(PermissionCategory::Drawing);
        assert_eq!(drawing_perms.len(), 2);

        let user_perms = perms.by_category(PermissionCategory::User);
        assert_eq!(user_perms.len(), 1);
    }

    #[test]
    fn test_resource_permission() {
        let mut perms = PermissionSet::new();
        perms.add(Permission::DrawingRead);
        perms.add(Permission::DrawingUpdate);

        let resource_perm = ResourcePermission::new(
            "drawing".to_string(),
            "drawing-123".to_string(),
            perms,
        );

        assert!(resource_perm.has_permission(&Permission::DrawingRead));
        assert!(resource_perm.has_permission(&Permission::DrawingUpdate));
        assert!(!resource_perm.has_permission(&Permission::DrawingDelete));
    }
}
