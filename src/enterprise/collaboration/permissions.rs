//! Collaboration Permissions System
//!
//! This module provides fine-grained permission control for collaborative editing,
//! including edit permissions, view-only mode, region locking, and permission delegation.

use super::{CollaborationError, Result};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

/// Permission types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionType {
    /// Full read and write access
    FullAccess,
    /// Read-only access
    ReadOnly,
    /// Can create new entities
    Create,
    /// Can modify existing entities
    Modify,
    /// Can delete entities
    Delete,
    /// Can manage layers
    ManageLayers,
    /// Can apply constraints
    ApplyConstraints,
    /// Can manage permissions for others
    ManagePermissions,
    /// Can lock/unlock regions
    LockRegions,
    /// Custom permission
    Custom(String),
}

/// View mode for participants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewMode {
    /// Full editing capabilities
    Edit,
    /// Can view and comment, but not edit
    Comment,
    /// View only, no interaction
    ViewOnly,
    /// Presenter mode (others view what they're doing)
    Presenter,
}

impl Default for ViewMode {
    fn default() -> Self {
        Self::Edit
    }
}

/// Edit permission details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditPermission {
    /// Allowed operations
    pub allowed_operations: HashSet<PermissionType>,
    /// View mode
    pub view_mode: ViewMode,
    /// Allowed entity types (None = all)
    pub allowed_entity_types: Option<HashSet<String>>,
    /// Allowed layers (None = all)
    pub allowed_layers: Option<HashSet<Uuid>>,
    /// Denied entity IDs (specific entities blocked)
    pub denied_entities: HashSet<Uuid>,
    /// Maximum operations per minute (rate limiting)
    pub max_operations_per_minute: Option<usize>,
}

impl Default for EditPermission {
    fn default() -> Self {
        let mut allowed = HashSet::new();
        allowed.insert(PermissionType::FullAccess);

        Self {
            allowed_operations: allowed,
            view_mode: ViewMode::Edit,
            allowed_entity_types: None,
            allowed_layers: None,
            denied_entities: HashSet::new(),
            max_operations_per_minute: None,
        }
    }
}

impl EditPermission {
    /// Create a read-only permission
    pub fn read_only() -> Self {
        Self {
            allowed_operations: HashSet::new(),
            view_mode: ViewMode::ViewOnly,
            allowed_entity_types: None,
            allowed_layers: None,
            denied_entities: HashSet::new(),
            max_operations_per_minute: None,
        }
    }

    /// Create a comment-only permission
    pub fn comment_only() -> Self {
        Self {
            allowed_operations: HashSet::new(),
            view_mode: ViewMode::Comment,
            allowed_entity_types: None,
            allowed_layers: None,
            denied_entities: HashSet::new(),
            max_operations_per_minute: None,
        }
    }

    /// Create a full access permission
    pub fn full_access() -> Self {
        Self::default()
    }

    /// Check if a permission type is allowed
    pub fn has_permission(&self, permission: PermissionType) -> bool {
        self.allowed_operations.contains(&PermissionType::FullAccess)
            || self.allowed_operations.contains(&permission)
    }

    /// Check if can edit a specific entity
    pub fn can_edit_entity(&self, entity_id: Uuid, entity_type: &str, layer_id: Option<Uuid>) -> bool {
        // Check view mode
        if self.view_mode != ViewMode::Edit {
            return false;
        }

        // Check denied entities
        if self.denied_entities.contains(&entity_id) {
            return false;
        }

        // Check entity type restrictions
        if let Some(allowed_types) = &self.allowed_entity_types {
            if !allowed_types.contains(entity_type) {
                return false;
            }
        }

        // Check layer restrictions
        if let Some(allowed_layers) = &self.allowed_layers {
            if let Some(layer) = layer_id {
                if !allowed_layers.contains(&layer) {
                    return false;
                }
            } else {
                // Entity not on any layer, and we have layer restrictions
                return false;
            }
        }

        true
    }

    /// Add a permission type
    pub fn add_permission(&mut self, permission: PermissionType) {
        self.allowed_operations.insert(permission);
    }

    /// Remove a permission type
    pub fn remove_permission(&mut self, permission: PermissionType) {
        self.allowed_operations.remove(&permission);
    }

    /// Deny access to a specific entity
    pub fn deny_entity(&mut self, entity_id: Uuid) {
        self.denied_entities.insert(entity_id);
    }

    /// Allow access to a previously denied entity
    pub fn allow_entity(&mut self, entity_id: Uuid) {
        self.denied_entities.remove(&entity_id);
    }
}

/// Region lock for collaborative editing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionLock {
    /// Lock ID
    pub lock_id: Uuid,
    /// User who owns the lock
    pub owner_id: Uuid,
    /// Locked entity IDs
    pub locked_entities: HashSet<Uuid>,
    /// Locked region (bounding box)
    pub locked_region: Option<(f64, f64, f64, f64)>, // (x_min, y_min, x_max, y_max)
    /// Lock timestamp
    pub locked_at: DateTime<Utc>,
    /// Lock expiration (None = no expiration)
    pub expires_at: Option<DateTime<Utc>>,
    /// Lock reason/description
    pub reason: Option<String>,
}

impl RegionLock {
    /// Create a new region lock
    pub fn new(owner_id: Uuid) -> Self {
        Self {
            lock_id: Uuid::new_v4(),
            owner_id,
            locked_entities: HashSet::new(),
            locked_region: None,
            locked_at: Utc::now(),
            expires_at: None,
            reason: None,
        }
    }

    /// Create a lock for specific entities
    pub fn for_entities(owner_id: Uuid, entities: HashSet<Uuid>) -> Self {
        Self {
            lock_id: Uuid::new_v4(),
            owner_id,
            locked_entities: entities,
            locked_region: None,
            locked_at: Utc::now(),
            expires_at: None,
            reason: None,
        }
    }

    /// Create a lock for a region
    pub fn for_region(owner_id: Uuid, x_min: f64, y_min: f64, x_max: f64, y_max: f64) -> Self {
        Self {
            lock_id: Uuid::new_v4(),
            owner_id,
            locked_entities: HashSet::new(),
            locked_region: Some((x_min, y_min, x_max, y_max)),
            locked_at: Utc::now(),
            expires_at: None,
            reason: None,
        }
    }

    /// Set expiration time
    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Set reason
    pub fn with_reason(mut self, reason: String) -> Self {
        self.reason = Some(reason);
        self
    }

    /// Check if lock is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Check if a point is in the locked region
    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        if let Some((x_min, y_min, x_max, y_max)) = self.locked_region {
            x >= x_min && x <= x_max && y >= y_min && y <= y_max
        } else {
            false
        }
    }

    /// Check if an entity is locked
    pub fn is_entity_locked(&self, entity_id: Uuid) -> bool {
        self.locked_entities.contains(&entity_id)
    }
}

/// Permission delegation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionDelegation {
    /// Delegation ID
    pub delegation_id: Uuid,
    /// User granting the permission
    pub grantor_id: Uuid,
    /// User receiving the permission
    pub grantee_id: Uuid,
    /// Delegated permission
    pub permission: EditPermission,
    /// Delegation timestamp
    pub granted_at: DateTime<Utc>,
    /// Expiration (None = no expiration)
    pub expires_at: Option<DateTime<Utc>>,
    /// Can the grantee delegate this permission further?
    pub can_delegate: bool,
}

impl PermissionDelegation {
    /// Create a new delegation
    pub fn new(grantor_id: Uuid, grantee_id: Uuid, permission: EditPermission) -> Self {
        Self {
            delegation_id: Uuid::new_v4(),
            grantor_id,
            grantee_id,
            permission,
            granted_at: Utc::now(),
            expires_at: None,
            can_delegate: false,
        }
    }

    /// Check if delegation is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }
}

/// Permission grant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    /// User ID
    pub user_id: Uuid,
    /// Edit permissions
    pub edit_permission: EditPermission,
    /// Granted at
    pub granted_at: DateTime<Utc>,
    /// Granted by (None = system)
    pub granted_by: Option<Uuid>,
}

/// Internal permission manager state
struct PermissionManagerInner {
    /// User permissions
    permissions: HashMap<Uuid, Permission>,
    /// Active region locks
    locks: HashMap<Uuid, RegionLock>,
    /// Permission delegations
    delegations: HashMap<Uuid, PermissionDelegation>,
    /// Entity ownership (for tracking who created what)
    entity_owners: HashMap<Uuid, Uuid>,
    /// Operation counters for rate limiting
    operation_counters: HashMap<Uuid, Vec<DateTime<Utc>>>,
}

/// Permission manager for collaboration sessions
#[derive(Clone)]
pub struct PermissionManager {
    inner: Arc<RwLock<PermissionManagerInner>>,
}

impl PermissionManager {
    /// Create a new permission manager
    pub fn new() -> Self {
        let inner = PermissionManagerInner {
            permissions: HashMap::new(),
            locks: HashMap::new(),
            delegations: HashMap::new(),
            entity_owners: HashMap::new(),
            operation_counters: HashMap::new(),
        };

        Self {
            inner: Arc::new(RwLock::new(inner)),
        }
    }

    /// Grant permission to a user
    pub fn grant_permission(
        &self,
        user_id: Uuid,
        permission: EditPermission,
        granted_by: Option<Uuid>,
    ) -> Result<()> {
        let mut inner = self.inner.write();

        let perm = Permission {
            user_id,
            edit_permission: permission,
            granted_at: Utc::now(),
            granted_by,
        };

        inner.permissions.insert(user_id, perm);
        Ok(())
    }

    /// Revoke permission from a user
    pub fn revoke_permission(&self, user_id: Uuid) -> Result<()> {
        let mut inner = self.inner.write();
        inner.permissions.remove(&user_id);
        Ok(())
    }

    /// Get user permission
    pub fn get_permission(&self, user_id: Uuid) -> Option<Permission> {
        let inner = self.inner.read();
        inner.permissions.get(&user_id).cloned()
    }

    /// Check if user has a specific permission
    pub fn has_permission(&self, user_id: Uuid, permission: PermissionType) -> bool {
        let inner = self.inner.read();
        if let Some(perm) = inner.permissions.get(&user_id) {
            perm.edit_permission.has_permission(permission)
        } else {
            false
        }
    }

    /// Check if user can edit an entity
    pub fn can_edit_entity(
        &self,
        user_id: Uuid,
        entity_id: Uuid,
        entity_type: &str,
        layer_id: Option<Uuid>,
    ) -> bool {
        let inner = self.inner.read();

        // Check basic permission
        if let Some(perm) = inner.permissions.get(&user_id) {
            if !perm.edit_permission.can_edit_entity(entity_id, entity_type, layer_id) {
                return false;
            }
        } else {
            return false;
        }

        // Check if entity is locked by someone else
        for lock in inner.locks.values() {
            if lock.is_expired() {
                continue;
            }

            if lock.owner_id != user_id && lock.is_entity_locked(entity_id) {
                return false;
            }
        }

        true
    }

    /// Create a region lock
    pub fn create_lock(&self, lock: RegionLock) -> Result<Uuid> {
        let mut inner = self.inner.write();
        let lock_id = lock.lock_id;
        inner.locks.insert(lock_id, lock);
        Ok(lock_id)
    }

    /// Release a lock
    pub fn release_lock(&self, lock_id: Uuid, user_id: Uuid) -> Result<()> {
        let mut inner = self.inner.write();

        if let Some(lock) = inner.locks.get(&lock_id) {
            if lock.owner_id != user_id {
                return Err(CollaborationError::PermissionDenied(
                    "Cannot release lock owned by another user".to_string(),
                ));
            }
        }

        inner.locks.remove(&lock_id);
        Ok(())
    }

    /// Get all active locks
    pub fn get_active_locks(&self) -> Vec<RegionLock> {
        let inner = self.inner.read();
        inner
            .locks
            .values()
            .filter(|lock| !lock.is_expired())
            .cloned()
            .collect()
    }

    /// Get locks for a specific user
    pub fn get_user_locks(&self, user_id: Uuid) -> Vec<RegionLock> {
        let inner = self.inner.read();
        inner
            .locks
            .values()
            .filter(|lock| lock.owner_id == user_id && !lock.is_expired())
            .cloned()
            .collect()
    }

    /// Clean up expired locks
    pub fn cleanup_expired_locks(&self) -> usize {
        let mut inner = self.inner.write();
        let before_count = inner.locks.len();

        inner.locks.retain(|_, lock| !lock.is_expired());

        before_count - inner.locks.len()
    }

    /// Delegate permission to another user
    pub fn delegate_permission(&self, delegation: PermissionDelegation) -> Result<Uuid> {
        let mut inner = self.inner.write();

        // Check if grantor can delegate
        if let Some(perm) = inner.permissions.get(&delegation.grantor_id) {
            if perm.edit_permission.view_mode != ViewMode::Edit {
                return Err(CollaborationError::PermissionDenied(
                    "User cannot delegate permissions".to_string(),
                ));
            }
        } else {
            return Err(CollaborationError::PermissionDenied(
                "Grantor has no permissions".to_string(),
            ));
        }

        let delegation_id = delegation.delegation_id;
        inner.delegations.insert(delegation_id, delegation);

        Ok(delegation_id)
    }

    /// Revoke a delegation
    pub fn revoke_delegation(&self, delegation_id: Uuid, user_id: Uuid) -> Result<()> {
        let mut inner = self.inner.write();

        if let Some(delegation) = inner.delegations.get(&delegation_id) {
            if delegation.grantor_id != user_id {
                return Err(CollaborationError::PermissionDenied(
                    "Only grantor can revoke delegation".to_string(),
                ));
            }
        }

        inner.delegations.remove(&delegation_id);
        Ok(())
    }

    /// Record entity ownership
    pub fn set_entity_owner(&self, entity_id: Uuid, owner_id: Uuid) {
        let mut inner = self.inner.write();
        inner.entity_owners.insert(entity_id, owner_id);
    }

    /// Get entity owner
    pub fn get_entity_owner(&self, entity_id: Uuid) -> Option<Uuid> {
        let inner = self.inner.read();
        inner.entity_owners.get(&entity_id).copied()
    }

    /// Check rate limit for user
    pub fn check_rate_limit(&self, user_id: Uuid) -> Result<()> {
        let mut inner = self.inner.write();

        // Get user's rate limit
        let limit = if let Some(perm) = inner.permissions.get(&user_id) {
            perm.edit_permission.max_operations_per_minute
        } else {
            None
        };

        if let Some(max_ops) = limit {
            let now = Utc::now();
            let one_minute_ago = now - chrono::Duration::seconds(60);

            // Get or create counter
            let counter = inner.operation_counters.entry(user_id).or_insert_with(Vec::new);

            // Remove operations older than 1 minute
            counter.retain(|&timestamp| timestamp > one_minute_ago);

            // Check if limit exceeded
            if counter.len() >= max_ops {
                return Err(CollaborationError::Operation(format!(
                    "Rate limit exceeded: {} operations per minute",
                    max_ops
                )));
            }

            // Record this operation
            counter.push(now);
        }

        Ok(())
    }

    /// Clear all permissions and locks
    pub fn clear(&self) {
        let mut inner = self.inner.write();
        inner.permissions.clear();
        inner.locks.clear();
        inner.delegations.clear();
        inner.entity_owners.clear();
        inner.operation_counters.clear();
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_permission() {
        let perm = EditPermission::full_access();
        assert!(perm.has_permission(PermissionType::Create));
        assert!(perm.has_permission(PermissionType::Modify));

        let perm = EditPermission::read_only();
        assert!(!perm.has_permission(PermissionType::Modify));
        assert_eq!(perm.view_mode, ViewMode::ViewOnly);
    }

    #[test]
    fn test_region_lock() {
        let user_id = Uuid::new_v4();
        let lock = RegionLock::for_region(user_id, 0.0, 0.0, 100.0, 100.0);

        assert!(lock.contains_point(50.0, 50.0));
        assert!(!lock.contains_point(150.0, 150.0));
        assert!(!lock.is_expired());
    }

    #[test]
    fn test_permission_manager() {
        let manager = PermissionManager::new();
        let user_id = Uuid::new_v4();

        let perm = EditPermission::full_access();
        manager.grant_permission(user_id, perm, None).unwrap();

        assert!(manager.has_permission(user_id, PermissionType::Create));
        assert!(manager.has_permission(user_id, PermissionType::Modify));

        manager.revoke_permission(user_id).unwrap();
        assert!(!manager.has_permission(user_id, PermissionType::Create));
    }

    #[test]
    fn test_region_locking() {
        let manager = PermissionManager::new();
        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();
        let _entity = Uuid::new_v4();

        // Grant permissions to both users
        manager.grant_permission(user1, EditPermission::full_access(), None).unwrap();
        manager.grant_permission(user2, EditPermission::full_access(), None).unwrap();

        // User1 locks the entity
        let lock = RegionLock::for_entities(user1, [entity].into_iter().collect());
        manager.create_lock(lock).unwrap();

        // User1 can still edit
        assert!(manager.can_edit_entity(user1, entity, "Line", None));

        // User2 cannot edit
        assert!(!manager.can_edit_entity(user2, entity, "Line", None));
    }

    #[test]
    fn test_rate_limiting() {
        let manager = PermissionManager::new();
        let user_id = Uuid::new_v4();

        let mut perm = EditPermission::full_access();
        perm.max_operations_per_minute = Some(2);

        manager.grant_permission(user_id, perm, None).unwrap();

        // First two operations should succeed
        assert!(manager.check_rate_limit(user_id).is_ok());
        assert!(manager.check_rate_limit(user_id).is_ok());

        // Third should fail
        assert!(manager.check_rate_limit(user_id).is_err());
    }
}
