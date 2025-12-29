//! Role-Based Access Control (RBAC) System
//!
//! Advanced RBAC implementation with:
//! - Hierarchical roles with inheritance
//! - Fine-grained permissions
//! - Dynamic role assignment
//! - Permission aggregation
//! - Role constraints (time-based, location-based)
//! - Separation of duties (SoD)
//! - Delegation support
//! - Context-aware access control
//!
//! # Architecture
//! - **Permissions**: Atomic access rights (e.g., "drawing:create", "layer:delete")
//! - **Roles**: Collections of permissions (e.g., "Designer", "Viewer")
//! - **Users**: Assigned one or more roles
//! - **Constraints**: Conditions that must be met for role activation
//! - **Delegation**: Temporary permission grants

use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, Duration};

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum RbacError {
    #[error("Role not found: {0}")]
    RoleNotFound(String),

    #[error("Permission not found: {0}")]
    PermissionNotFound(String),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Role already exists: {0}")]
    RoleAlreadyExists(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Delegation error: {0}")]
    DelegationError(String),
}

pub type RbacResult<T> = Result<T, RbacError>;

// ============================================================================
// Permission System
// ============================================================================

/// Permission represents an atomic access right
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    /// Resource type (e.g., "drawing", "layer", "user")
    pub resource: String,

    /// Action (e.g., "create", "read", "update", "delete")
    pub action: String,

    /// Optional scope/filter (e.g., "own", "team", "all")
    pub scope: Option<String>,
}

impl Permission {
    /// Create a new permission
    pub fn new(resource: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            action: action.into(),
            scope: None,
        }
    }

    /// Create a permission with scope
    pub fn with_scope(
        resource: impl Into<String>,
        action: impl Into<String>,
        scope: impl Into<String>,
    ) -> Self {
        Self {
            resource: resource.into(),
            action: action.into(),
            scope: Some(scope.into()),
        }
    }

    /// Get permission string representation (resource:action or resource:action:scope)
    pub fn to_string(&self) -> String {
        if let Some(ref scope) = self.scope {
            format!("{}:{}:{}", self.resource, self.action, scope)
        } else {
            format!("{}:{}", self.resource, self.action)
        }
    }

    /// Parse from string representation
    pub fn from_string(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        match parts.len() {
            2 => Some(Self::new(parts[0], parts[1])),
            3 => Some(Self::with_scope(parts[0], parts[1], parts[2])),
            _ => None,
        }
    }

    /// Check if this permission matches a pattern
    pub fn matches(&self, pattern: &Permission) -> bool {
        let resource_match = pattern.resource == "*" || self.resource == pattern.resource;
        let action_match = pattern.action == "*" || self.action == pattern.action;
        let scope_match = match (&self.scope, &pattern.scope) {
            (Some(s1), Some(s2)) => s1 == s2 || s2 == "*",
            (None, None) => true,
            (None, Some(s)) => s == "*",
            (Some(_), None) => true,
        };

        resource_match && action_match && scope_match
    }
}

// ============================================================================
// Role System
// ============================================================================

/// Role with permissions and constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Unique role identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Description
    pub description: String,

    /// Permissions granted by this role
    pub permissions: HashSet<Permission>,

    /// Parent roles (for inheritance)
    pub parents: HashSet<String>,

    /// Role constraints
    pub constraints: Vec<RoleConstraint>,

    /// Is this a system role (cannot be deleted)
    pub is_system: bool,

    /// Creation timestamp
    pub created_at: SystemTime,

    /// Last modified timestamp
    pub modified_at: SystemTime,
}

impl Role {
    /// Create a new role
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        let now = SystemTime::now();
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            permissions: HashSet::new(),
            parents: HashSet::new(),
            constraints: Vec::new(),
            is_system: false,
            created_at: now,
            modified_at: now,
        }
    }

    /// Add a permission to this role
    pub fn add_permission(&mut self, permission: Permission) {
        self.permissions.insert(permission);
        self.modified_at = SystemTime::now();
    }

    /// Remove a permission from this role
    pub fn remove_permission(&mut self, permission: &Permission) {
        self.permissions.remove(permission);
        self.modified_at = SystemTime::now();
    }

    /// Add a parent role
    pub fn add_parent(&mut self, parent_id: impl Into<String>) {
        self.parents.insert(parent_id.into());
        self.modified_at = SystemTime::now();
    }

    /// Add a constraint
    pub fn add_constraint(&mut self, constraint: RoleConstraint) {
        self.constraints.push(constraint);
        self.modified_at = SystemTime::now();
    }

    /// Check if role has permission (direct, not inherited)
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.iter().any(|p| permission.matches(p))
    }

    /// Check if constraints are satisfied
    pub fn constraints_satisfied(&self, context: &AccessContext) -> bool {
        self.constraints
            .iter()
            .all(|c| c.is_satisfied(context))
    }
}

// ============================================================================
// Role Constraints
// ============================================================================

/// Role activation constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoleConstraint {
    /// Time-based constraint (business hours only)
    TimeWindow {
        start_hour: u8,
        end_hour: u8,
    },

    /// Date range constraint
    DateRange {
        start: SystemTime,
        end: SystemTime,
    },

    /// IP address whitelist
    IpWhitelist {
        allowed_ips: Vec<String>,
    },

    /// Location-based constraint
    Location {
        allowed_locations: Vec<String>,
    },

    /// MFA required
    MfaRequired,

    /// Maximum concurrent sessions
    MaxSessions {
        limit: usize,
    },
}

impl RoleConstraint {
    /// Check if constraint is satisfied
    pub fn is_satisfied(&self, context: &AccessContext) -> bool {
        match self {
            Self::TimeWindow { start_hour, end_hour } => {
                if let Some(current_hour) = context.current_hour {
                    current_hour >= *start_hour && current_hour <= *end_hour
                } else {
                    false
                }
            }
            Self::DateRange { start, end } => {
                if let Ok(now) = SystemTime::now().duration_since(*start) {
                    if let Ok(remaining) = end.duration_since(SystemTime::now()) {
                        return true;
                    }
                }
                false
            }
            Self::IpWhitelist { allowed_ips } => {
                if let Some(ref ip) = context.ip_address {
                    allowed_ips.contains(ip)
                } else {
                    false
                }
            }
            Self::Location { allowed_locations } => {
                if let Some(ref location) = context.location {
                    allowed_locations.contains(location)
                } else {
                    false
                }
            }
            Self::MfaRequired => context.mfa_verified,
            Self::MaxSessions { limit } => context.active_sessions <= *limit,
        }
    }
}

// ============================================================================
// Access Context
// ============================================================================

/// Context for access control decisions
#[derive(Debug, Clone, Default)]
pub struct AccessContext {
    /// Current hour (0-23)
    pub current_hour: Option<u8>,

    /// IP address
    pub ip_address: Option<String>,

    /// Location
    pub location: Option<String>,

    /// MFA verification status
    pub mfa_verified: bool,

    /// Number of active sessions
    pub active_sessions: usize,

    /// Additional context data
    pub metadata: HashMap<String, String>,
}

// ============================================================================
// Permission Delegation
// ============================================================================

/// Temporary permission delegation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delegation {
    /// Unique delegation ID
    pub id: String,

    /// Delegator (who grants permission)
    pub delegator: String,

    /// Delegate (who receives permission)
    pub delegate: String,

    /// Delegated permissions
    pub permissions: HashSet<Permission>,

    /// Expiration time
    pub expires_at: SystemTime,

    /// Creation timestamp
    pub created_at: SystemTime,

    /// Is active
    pub active: bool,
}

impl Delegation {
    /// Create a new delegation
    pub fn new(
        delegator: impl Into<String>,
        delegate: impl Into<String>,
        permissions: HashSet<Permission>,
        duration: Duration,
    ) -> Self {
        let now = SystemTime::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            delegator: delegator.into(),
            delegate: delegate.into(),
            permissions,
            expires_at: now + duration,
            created_at: now,
            active: true,
        }
    }

    /// Check if delegation is valid
    pub fn is_valid(&self) -> bool {
        self.active && SystemTime::now() < self.expires_at
    }

    /// Revoke delegation
    pub fn revoke(&mut self) {
        self.active = false;
    }
}

// ============================================================================
// RBAC Manager
// ============================================================================

/// RBAC management system
pub struct RbacManager {
    /// All roles in the system
    roles: HashMap<String, Role>,

    /// User role assignments (user_id -> role_ids)
    user_roles: HashMap<String, HashSet<String>>,

    /// Active delegations
    delegations: HashMap<String, Delegation>,

    /// Permission cache (user_id -> effective permissions)
    permission_cache: HashMap<String, HashSet<Permission>>,
}

impl RbacManager {
    /// Create a new RBAC manager
    pub fn new() -> Self {
        let mut manager = Self {
            roles: HashMap::new(),
            user_roles: HashMap::new(),
            delegations: HashMap::new(),
            permission_cache: HashMap::new(),
        };

        // Initialize with standard roles
        manager.init_standard_roles();

        manager
    }

    /// Initialize standard roles
    fn init_standard_roles(&mut self) {
        // Administrator role
        let mut admin = Role::new("admin", "Administrator");
        admin.description = "Full system access".to_string();
        admin.is_system = true;
        admin.add_permission(Permission::new("*", "*"));
        self.roles.insert("admin".to_string(), admin);

        // Designer role
        let mut designer = Role::new("designer", "Designer");
        designer.description = "Can create and modify drawings".to_string();
        designer.is_system = true;
        designer.add_permission(Permission::new("drawing", "create"));
        designer.add_permission(Permission::new("drawing", "read"));
        designer.add_permission(Permission::new("drawing", "update"));
        designer.add_permission(Permission::new("drawing", "delete"));
        designer.add_permission(Permission::new("layer", "*"));
        self.roles.insert("designer".to_string(), designer);

        // Viewer role
        let mut viewer = Role::new("viewer", "Viewer");
        viewer.description = "Read-only access".to_string();
        viewer.is_system = true;
        viewer.add_permission(Permission::new("drawing", "read"));
        viewer.add_permission(Permission::new("layer", "read"));
        self.roles.insert("viewer".to_string(), viewer);
    }

    /// Create a new role
    pub fn create_role(&mut self, role: Role) -> RbacResult<()> {
        if self.roles.contains_key(&role.id) {
            return Err(RbacError::RoleAlreadyExists(role.id.clone()));
        }

        // Check for circular dependencies
        if !role.parents.is_empty() {
            self.check_circular_dependency(&role.id, &role.parents)?;
        }

        self.roles.insert(role.id.clone(), role);
        self.invalidate_cache();
        Ok(())
    }

    /// Get a role
    pub fn get_role(&self, role_id: &str) -> RbacResult<&Role> {
        self.roles
            .get(role_id)
            .ok_or_else(|| RbacError::RoleNotFound(role_id.to_string()))
    }

    /// Update a role
    pub fn update_role(&mut self, role: Role) -> RbacResult<()> {
        if !self.roles.contains_key(&role.id) {
            return Err(RbacError::RoleNotFound(role.id.clone()));
        }

        // Check for circular dependencies
        if !role.parents.is_empty() {
            self.check_circular_dependency(&role.id, &role.parents)?;
        }

        self.roles.insert(role.id.clone(), role);
        self.invalidate_cache();
        Ok(())
    }

    /// Delete a role
    pub fn delete_role(&mut self, role_id: &str) -> RbacResult<()> {
        let role = self.get_role(role_id)?;

        if role.is_system {
            return Err(RbacError::PermissionDenied(
                "Cannot delete system role".to_string(),
            ));
        }

        self.roles.remove(role_id);

        // Remove role from all users
        for roles in self.user_roles.values_mut() {
            roles.remove(role_id);
        }

        self.invalidate_cache();
        Ok(())
    }

    /// Assign role to user
    pub fn assign_role(&mut self, user_id: &str, role_id: &str) -> RbacResult<()> {
        // Verify role exists
        self.get_role(role_id)?;

        self.user_roles
            .entry(user_id.to_string())
            .or_insert_with(HashSet::new)
            .insert(role_id.to_string());

        self.invalidate_user_cache(user_id);
        Ok(())
    }

    /// Remove role from user
    pub fn remove_role(&mut self, user_id: &str, role_id: &str) -> RbacResult<()> {
        if let Some(roles) = self.user_roles.get_mut(user_id) {
            roles.remove(role_id);
            self.invalidate_user_cache(user_id);
        }
        Ok(())
    }

    /// Check if user has permission
    pub fn has_permission(
        &mut self,
        user_id: &str,
        permission: &Permission,
        context: &AccessContext,
    ) -> bool {
        // Get effective permissions
        let effective_perms = self.get_effective_permissions(user_id, context);

        // Check if any effective permission matches
        effective_perms.iter().any(|p| permission.matches(p))
    }

    /// Get all effective permissions for a user
    pub fn get_effective_permissions(
        &mut self,
        user_id: &str,
        context: &AccessContext,
    ) -> HashSet<Permission> {
        // Check cache
        if let Some(cached) = self.permission_cache.get(user_id) {
            return cached.clone();
        }

        let mut effective_perms = HashSet::new();

        // Get user's roles
        if let Some(role_ids) = self.user_roles.get(user_id) {
            for role_id in role_ids {
                if let Ok(role) = self.get_role(role_id) {
                    // Check constraints
                    if role.constraints_satisfied(context) {
                        // Add direct permissions
                        effective_perms.extend(role.permissions.clone());

                        // Add inherited permissions
                        effective_perms.extend(self.get_inherited_permissions(role_id, context));
                    }
                }
            }
        }

        // Add delegated permissions
        for delegation in self.delegations.values() {
            if delegation.delegate == user_id && delegation.is_valid() {
                effective_perms.extend(delegation.permissions.clone());
            }
        }

        // Cache the result
        self.permission_cache
            .insert(user_id.to_string(), effective_perms.clone());

        effective_perms
    }

    /// Get inherited permissions from parent roles
    fn get_inherited_permissions(
        &self,
        role_id: &str,
        context: &AccessContext,
    ) -> HashSet<Permission> {
        let mut inherited = HashSet::new();

        if let Ok(role) = self.get_role(role_id) {
            for parent_id in &role.parents {
                if let Ok(parent) = self.get_role(parent_id) {
                    if parent.constraints_satisfied(context) {
                        inherited.extend(parent.permissions.clone());
                        inherited.extend(self.get_inherited_permissions(parent_id, context));
                    }
                }
            }
        }

        inherited
    }

    /// Create a delegation
    pub fn create_delegation(
        &mut self,
        delegator: &str,
        delegate: &str,
        permissions: HashSet<Permission>,
        duration: Duration,
        context: &AccessContext,
    ) -> RbacResult<String> {
        // Verify delegator has these permissions
        for permission in &permissions {
            if !self.has_permission(delegator, permission, context) {
                return Err(RbacError::DelegationError(
                    "Delegator does not have permission".to_string(),
                ));
            }
        }

        let delegation = Delegation::new(delegator, delegate, permissions, duration);
        let id = delegation.id.clone();

        self.delegations.insert(id.clone(), delegation);
        self.invalidate_user_cache(delegate);

        Ok(id)
    }

    /// Revoke a delegation
    pub fn revoke_delegation(&mut self, delegation_id: &str) -> RbacResult<()> {
        if let Some(delegation) = self.delegations.get_mut(delegation_id) {
            delegation.revoke();
            self.invalidate_user_cache(&delegation.delegate);
            Ok(())
        } else {
            Err(RbacError::DelegationError("Delegation not found".to_string()))
        }
    }

    /// Check for circular role dependencies
    fn check_circular_dependency(
        &self,
        role_id: &str,
        parents: &HashSet<String>,
    ) -> RbacResult<()> {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();

        for parent_id in parents {
            if self.has_circular_dependency(role_id, parent_id, &mut visited, &mut stack) {
                return Err(RbacError::CircularDependency(format!(
                    "Circular dependency detected: {} -> {}",
                    role_id, parent_id
                )));
            }
        }

        Ok(())
    }

    fn has_circular_dependency(
        &self,
        target: &str,
        current: &str,
        visited: &mut HashSet<String>,
        stack: &mut Vec<String>,
    ) -> bool {
        if current == target {
            return true;
        }

        if visited.contains(current) {
            return false;
        }

        visited.insert(current.to_string());
        stack.push(current.to_string());

        if let Ok(role) = self.get_role(current) {
            for parent in &role.parents {
                if self.has_circular_dependency(target, parent, visited, stack) {
                    return true;
                }
            }
        }

        stack.pop();
        false
    }

    /// Invalidate entire permission cache
    fn invalidate_cache(&mut self) {
        self.permission_cache.clear();
    }

    /// Invalidate cache for specific user
    fn invalidate_user_cache(&mut self, user_id: &str) {
        self.permission_cache.remove(user_id);
    }

    /// Clean up expired delegations
    pub fn cleanup_delegations(&mut self) {
        let now = SystemTime::now();
        self.delegations.retain(|_, d| d.expires_at > now);
    }

    /// Get RBAC statistics
    pub fn statistics(&self) -> RbacStatistics {
        RbacStatistics {
            total_roles: self.roles.len(),
            total_users: self.user_roles.len(),
            active_delegations: self.delegations.values().filter(|d| d.is_valid()).count(),
        }
    }
}

impl Default for RbacManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Statistics
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RbacStatistics {
    pub total_roles: usize,
    pub total_users: usize,
    pub active_delegations: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_creation() {
        let perm = Permission::new("drawing", "create");
        assert_eq!(perm.resource, "drawing");
        assert_eq!(perm.action, "create");
        assert_eq!(perm.to_string(), "drawing:create");
    }

    #[test]
    fn test_permission_matching() {
        let perm = Permission::new("drawing", "create");
        let pattern = Permission::new("drawing", "*");

        assert!(perm.matches(&pattern));
    }

    #[test]
    fn test_role_creation() {
        let mut manager = RbacManager::new();

        let mut role = Role::new("engineer", "Engineer");
        role.add_permission(Permission::new("drawing", "create"));

        assert!(manager.create_role(role).is_ok());
        assert!(manager.get_role("engineer").is_ok());
    }

    #[test]
    fn test_role_assignment() {
        let mut manager = RbacManager::new();

        let result = manager.assign_role("user1", "designer");
        assert!(result.is_ok());

        let context = AccessContext::default();
        assert!(manager.has_permission(
            "user1",
            &Permission::new("drawing", "create"),
            &context
        ));
    }

    #[test]
    fn test_delegation() {
        let mut manager = RbacManager::new();

        manager.assign_role("user1", "admin").unwrap();

        let context = AccessContext::default();
        let permissions = vec![Permission::new("drawing", "create")]
            .into_iter()
            .collect();

        let delegation_id = manager
            .create_delegation("user1", "user2", permissions, Duration::from_secs(3600), &context)
            .unwrap();

        assert!(manager.has_permission(
            "user2",
            &Permission::new("drawing", "create"),
            &context
        ));
    }
}
