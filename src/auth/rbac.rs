//! # Role-Based Access Control (RBAC) Module
//!
//! Enterprise-grade RBAC system with:
//! - Hierarchical role inheritance
//! - Resource-level permissions
//! - Permission matrix
//! - Custom role creation
//! - Attribute-based access control (ABAC) integration
//!
//! ## Security
//!
//! - Principle of least privilege
//! - Defense in depth
//! - Audit all permission checks

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::auth::{AuthError, UserContext};

/// Permission action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Create,
    Read,
    Update,
    Delete,
    Execute,
    Share,
    Export,
    Import,
    Approve,
    Publish,
    Archive,
    Restore,
}

/// Resource type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    Project,
    Drawing,
    Model,
    Layer,
    Template,
    User,
    Role,
    Team,
    Organization,
    Settings,
    AuditLog,
    Report,
    Plugin,
    Workflow,
    Custom(String),
}

/// Permission
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    /// Resource type
    pub resource_type: ResourceType,

    /// Action
    pub action: Action,

    /// Optional resource ID (for resource-level permissions)
    pub resource_id: Option<Uuid>,

    /// Optional conditions (for ABAC)
    pub conditions: Option<HashMap<String, String>>,
}

impl Permission {
    /// Create a new permission
    pub fn new(resource_type: ResourceType, action: Action) -> Self {
        Self {
            resource_type,
            action,
            resource_id: None,
            conditions: None,
        }
    }

    /// Create a resource-specific permission
    pub fn for_resource(resource_type: ResourceType, action: Action, resource_id: Uuid) -> Self {
        Self {
            resource_type,
            action,
            resource_id: Some(resource_id),
            conditions: None,
        }
    }

    /// Add condition
    pub fn with_condition(mut self, key: String, value: String) -> Self {
        self.conditions
            .get_or_insert_with(HashMap::new)
            .insert(key, value);
        self
    }

    /// Check if permission matches
    pub fn matches(&self, other: &Permission) -> bool {
        self.resource_type == other.resource_type
            && self.action == other.action
            && (self.resource_id.is_none() || self.resource_id == other.resource_id)
    }
}

/// Built-in roles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuiltInRole {
    /// System administrator - full access
    Admin,

    /// Project manager - can manage projects and teams
    Manager,

    /// Editor - can create and edit content
    Editor,

    /// Viewer - read-only access
    Viewer,

    /// Auditor - can view audit logs and reports
    Auditor,

    /// Guest - limited temporary access
    Guest,
}

/// Role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Role ID
    pub id: Uuid,

    /// Role name
    pub name: String,

    /// Role description
    pub description: String,

    /// Built-in role type (if applicable)
    pub built_in: Option<BuiltInRole>,

    /// Permissions granted by this role
    pub permissions: HashSet<Permission>,

    /// Parent roles (for inheritance)
    pub parent_roles: Vec<Uuid>,

    /// Organization ID (None for global roles)
    pub organization_id: Option<Uuid>,

    /// Created by
    pub created_by: Uuid,

    /// Created at
    pub created_at: DateTime<Utc>,

    /// Updated at
    pub updated_at: DateTime<Utc>,

    /// Is active
    pub is_active: bool,
}

impl Role {
    /// Create a new custom role
    pub fn new(name: String, description: String, created_by: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            built_in: None,
            permissions: HashSet::new(),
            parent_roles: Vec::new(),
            organization_id: None,
            created_by,
            created_at: now,
            updated_at: now,
            is_active: true,
        }
    }

    /// Create a built-in role
    pub fn built_in(role_type: BuiltInRole) -> Self {
        let now = Utc::now();
        let (name, description, permissions) = match role_type {
            BuiltInRole::Admin => (
                "Administrator".to_string(),
                "Full system access".to_string(),
                Self::admin_permissions(),
            ),
            BuiltInRole::Manager => (
                "Manager".to_string(),
                "Project and team management".to_string(),
                Self::manager_permissions(),
            ),
            BuiltInRole::Editor => (
                "Editor".to_string(),
                "Create and edit content".to_string(),
                Self::editor_permissions(),
            ),
            BuiltInRole::Viewer => (
                "Viewer".to_string(),
                "Read-only access".to_string(),
                Self::viewer_permissions(),
            ),
            BuiltInRole::Auditor => (
                "Auditor".to_string(),
                "Audit log and report access".to_string(),
                Self::auditor_permissions(),
            ),
            BuiltInRole::Guest => (
                "Guest".to_string(),
                "Limited temporary access".to_string(),
                Self::guest_permissions(),
            ),
        };

        Self {
            id: Uuid::new_v4(),
            name,
            description,
            built_in: Some(role_type),
            permissions,
            parent_roles: Vec::new(),
            organization_id: None,
            created_by: Uuid::nil(), // System
            created_at: now,
            updated_at: now,
            is_active: true,
        }
    }

    /// Admin permissions - full access
    fn admin_permissions() -> HashSet<Permission> {
        let mut perms = HashSet::new();
        let resources = vec![
            ResourceType::Project,
            ResourceType::Drawing,
            ResourceType::Model,
            ResourceType::Layer,
            ResourceType::Template,
            ResourceType::User,
            ResourceType::Role,
            ResourceType::Team,
            ResourceType::Organization,
            ResourceType::Settings,
            ResourceType::AuditLog,
            ResourceType::Report,
            ResourceType::Plugin,
            ResourceType::Workflow,
        ];

        let actions = vec![
            Action::Create,
            Action::Read,
            Action::Update,
            Action::Delete,
            Action::Execute,
            Action::Share,
            Action::Export,
            Action::Import,
            Action::Approve,
            Action::Publish,
            Action::Archive,
            Action::Restore,
        ];

        for resource in resources {
            for action in &actions {
                perms.insert(Permission::new(resource.clone(), *action));
            }
        }

        perms
    }

    /// Manager permissions
    fn manager_permissions() -> HashSet<Permission> {
        let mut perms = HashSet::new();
        let resources = vec![
            ResourceType::Project,
            ResourceType::Drawing,
            ResourceType::Model,
            ResourceType::Layer,
            ResourceType::Template,
            ResourceType::Team,
            ResourceType::Report,
            ResourceType::Workflow,
        ];

        let actions = vec![
            Action::Create,
            Action::Read,
            Action::Update,
            Action::Delete,
            Action::Share,
            Action::Export,
            Action::Import,
            Action::Approve,
            Action::Publish,
        ];

        for resource in resources {
            for action in &actions {
                perms.insert(Permission::new(resource.clone(), *action));
            }
        }

        // Can view users and audit logs
        perms.insert(Permission::new(ResourceType::User, Action::Read));
        perms.insert(Permission::new(ResourceType::AuditLog, Action::Read));

        perms
    }

    /// Editor permissions
    fn editor_permissions() -> HashSet<Permission> {
        let mut perms = HashSet::new();
        let resources = vec![
            ResourceType::Project,
            ResourceType::Drawing,
            ResourceType::Model,
            ResourceType::Layer,
            ResourceType::Template,
        ];

        let actions = vec![
            Action::Create,
            Action::Read,
            Action::Update,
            Action::Share,
            Action::Export,
        ];

        for resource in resources {
            for action in &actions {
                perms.insert(Permission::new(resource.clone(), *action));
            }
        }

        perms
    }

    /// Viewer permissions
    fn viewer_permissions() -> HashSet<Permission> {
        let mut perms = HashSet::new();
        let resources = vec![
            ResourceType::Project,
            ResourceType::Drawing,
            ResourceType::Model,
            ResourceType::Layer,
            ResourceType::Template,
            ResourceType::Report,
        ];

        for resource in resources {
            perms.insert(Permission::new(resource, Action::Read));
        }

        perms
    }

    /// Auditor permissions
    fn auditor_permissions() -> HashSet<Permission> {
        let mut perms = HashSet::new();

        perms.insert(Permission::new(ResourceType::AuditLog, Action::Read));
        perms.insert(Permission::new(ResourceType::Report, Action::Read));
        perms.insert(Permission::new(ResourceType::Report, Action::Create));
        perms.insert(Permission::new(ResourceType::Report, Action::Export));
        perms.insert(Permission::new(ResourceType::User, Action::Read));
        perms.insert(Permission::new(ResourceType::Project, Action::Read));

        perms
    }

    /// Guest permissions
    fn guest_permissions() -> HashSet<Permission> {
        let mut perms = HashSet::new();

        // Very limited read-only access
        perms.insert(Permission::new(ResourceType::Drawing, Action::Read));
        perms.insert(Permission::new(ResourceType::Model, Action::Read));

        perms
    }

    /// Add permission
    pub fn add_permission(&mut self, permission: Permission) {
        self.permissions.insert(permission);
        self.updated_at = Utc::now();
    }

    /// Remove permission
    pub fn remove_permission(&mut self, permission: &Permission) -> bool {
        let removed = self.permissions.remove(permission);
        if removed {
            self.updated_at = Utc::now();
        }
        removed
    }

    /// Add parent role (for inheritance)
    pub fn add_parent(&mut self, parent_id: Uuid) {
        if !self.parent_roles.contains(&parent_id) {
            self.parent_roles.push(parent_id);
            self.updated_at = Utc::now();
        }
    }
}

/// RBAC Manager
pub struct RbacManager {
    roles: HashMap<Uuid, Role>,
    user_roles: HashMap<Uuid, Vec<Uuid>>,
}

impl RbacManager {
    /// Create a new RBAC manager
    pub fn new() -> Self {
        let mut manager = Self {
            roles: HashMap::new(),
            user_roles: HashMap::new(),
        };

        // Initialize built-in roles
        manager.add_role(Role::built_in(BuiltInRole::Admin));
        manager.add_role(Role::built_in(BuiltInRole::Manager));
        manager.add_role(Role::built_in(BuiltInRole::Editor));
        manager.add_role(Role::built_in(BuiltInRole::Viewer));
        manager.add_role(Role::built_in(BuiltInRole::Auditor));
        manager.add_role(Role::built_in(BuiltInRole::Guest));

        manager
    }

    /// Add a role
    pub fn add_role(&mut self, role: Role) {
        self.roles.insert(role.id, role);
    }

    /// Get a role
    pub fn get_role(&self, role_id: &Uuid) -> Option<&Role> {
        self.roles.get(role_id)
    }

    /// Get role by name
    pub fn get_role_by_name(&self, name: &str) -> Option<&Role> {
        self.roles.values().find(|r| r.name == name)
    }

    /// Update a role
    pub fn update_role(&mut self, role: Role) -> Result<(), AuthError> {
        if role.built_in.is_some() {
            return Err(AuthError::PermissionDenied(
                "Cannot modify built-in roles".to_string(),
            ));
        }

        self.roles.insert(role.id, role);
        Ok(())
    }

    /// Delete a role
    pub fn delete_role(&mut self, role_id: &Uuid) -> Result<(), AuthError> {
        let role = self
            .roles
            .get(role_id)
            .ok_or_else(|| AuthError::InternalError("Role not found".to_string()))?;

        if role.built_in.is_some() {
            return Err(AuthError::PermissionDenied(
                "Cannot delete built-in roles".to_string(),
            ));
        }

        self.roles.remove(role_id);
        Ok(())
    }

    /// Assign role to user
    pub fn assign_role(&mut self, user_id: Uuid, role_id: Uuid) -> Result<(), AuthError> {
        // Verify role exists
        if !self.roles.contains_key(&role_id) {
            return Err(AuthError::InternalError("Role not found".to_string()));
        }

        let user_roles = self.user_roles.entry(user_id).or_insert_with(Vec::new);
        if !user_roles.contains(&role_id) {
            user_roles.push(role_id);
        }

        Ok(())
    }

    /// Revoke role from user
    pub fn revoke_role(&mut self, user_id: &Uuid, role_id: &Uuid) -> Result<(), AuthError> {
        if let Some(user_roles) = self.user_roles.get_mut(user_id) {
            user_roles.retain(|id| id != role_id);
        }
        Ok(())
    }

    /// Get user roles
    pub fn get_user_roles(&self, user_id: &Uuid) -> Vec<&Role> {
        let role_ids = self.user_roles.get(user_id).map(|v| v.as_slice()).unwrap_or(&[]);

        role_ids
            .iter()
            .filter_map(|id| self.roles.get(id))
            .collect()
    }

    /// Get all permissions for a user (including inherited)
    pub fn get_user_permissions(&self, user_id: &Uuid) -> HashSet<Permission> {
        let mut permissions = HashSet::new();
        let mut processed_roles = HashSet::new();

        let role_ids = self.user_roles.get(user_id).map(|v| v.as_slice()).unwrap_or(&[]);

        for role_id in role_ids {
            self.collect_permissions(*role_id, &mut permissions, &mut processed_roles);
        }

        permissions
    }

    /// Recursively collect permissions from role hierarchy
    fn collect_permissions(
        &self,
        role_id: Uuid,
        permissions: &mut HashSet<Permission>,
        processed: &mut HashSet<Uuid>,
    ) {
        if processed.contains(&role_id) {
            return; // Prevent circular inheritance
        }

        processed.insert(role_id);

        if let Some(role) = self.roles.get(&role_id) {
            if !role.is_active {
                return;
            }

            // Add role's own permissions
            permissions.extend(role.permissions.iter().cloned());

            // Recursively add parent role permissions
            for parent_id in &role.parent_roles {
                self.collect_permissions(*parent_id, permissions, processed);
            }
        }
    }

    /// Check if user has permission
    pub fn has_permission(
        &self,
        user_id: &Uuid,
        required_permission: &Permission,
    ) -> bool {
        let user_permissions = self.get_user_permissions(user_id);

        user_permissions.iter().any(|p| p.matches(required_permission))
    }

    /// Check if user can perform action on resource
    pub fn can_perform(
        &self,
        user_id: &Uuid,
        resource_type: ResourceType,
        action: Action,
        resource_id: Option<Uuid>,
    ) -> bool {
        let required = if let Some(id) = resource_id {
            Permission::for_resource(resource_type.clone(), action, id)
        } else {
            Permission::new(resource_type.clone(), action)
        };

        self.has_permission(user_id, &required)
    }

    /// Enforce permission (returns error if denied)
    pub fn enforce_permission(
        &self,
        user_id: &Uuid,
        required_permission: &Permission,
    ) -> Result<(), AuthError> {
        if self.has_permission(user_id, required_permission) {
            Ok(())
        } else {
            Err(AuthError::PermissionDenied(format!(
                "User lacks permission: {:?} on {:?}",
                required_permission.action, required_permission.resource_type
            )))
        }
    }

    /// List all roles
    pub fn list_roles(&self) -> Vec<&Role> {
        self.roles.values().collect()
    }

    /// List active roles
    pub fn list_active_roles(&self) -> Vec<&Role> {
        self.roles.values().filter(|r| r.is_active).collect()
    }
}

impl Default for RbacManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_has_all_permissions() {
        let mut manager = RbacManager::new();
        let admin_role = Role::built_in(BuiltInRole::Admin);
        let admin_role_id = admin_role.id;
        manager.add_role(admin_role);

        let user_id = Uuid::new_v4();
        manager.assign_role(user_id, admin_role_id).unwrap();

        assert!(manager.can_perform(
            &user_id,
            ResourceType::Project,
            Action::Delete,
            None
        ));
        assert!(manager.can_perform(
            &user_id,
            ResourceType::User,
            Action::Update,
            None
        ));
    }

    #[test]
    fn test_viewer_cannot_delete() {
        let mut manager = RbacManager::new();
        let viewer_role = Role::built_in(BuiltInRole::Viewer);
        let viewer_role_id = viewer_role.id;
        manager.add_role(viewer_role);

        let user_id = Uuid::new_v4();
        manager.assign_role(user_id, viewer_role_id).unwrap();

        assert!(manager.can_perform(
            &user_id,
            ResourceType::Project,
            Action::Read,
            None
        ));
        assert!(!manager.can_perform(
            &user_id,
            ResourceType::Project,
            Action::Delete,
            None
        ));
    }

    #[test]
    fn test_role_inheritance() {
        let mut manager = RbacManager::new();

        let mut parent_role = Role::new(
            "Parent".to_string(),
            "Parent role".to_string(),
            Uuid::nil(),
        );
        parent_role.add_permission(Permission::new(ResourceType::Project, Action::Read));
        let parent_id = parent_role.id;
        manager.add_role(parent_role);

        let mut child_role = Role::new(
            "Child".to_string(),
            "Child role".to_string(),
            Uuid::nil(),
        );
        child_role.add_parent(parent_id);
        child_role.add_permission(Permission::new(ResourceType::Project, Action::Update));
        let child_id = child_role.id;
        manager.add_role(child_role);

        let user_id = Uuid::new_v4();
        manager.assign_role(user_id, child_id).unwrap();

        // Should have both parent and child permissions
        assert!(manager.can_perform(
            &user_id,
            ResourceType::Project,
            Action::Read,
            None
        ));
        assert!(manager.can_perform(
            &user_id,
            ResourceType::Project,
            Action::Update,
            None
        ));
    }
}
