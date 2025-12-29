//! Fine-grained permission system for plugin security
//!
//! Implements a comprehensive permission model that controls what
//! plugins can access and modify in the CADDY system.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

/// Permission errors
#[derive(Debug, Error)]
pub enum PermissionError {
    #[error("Permission denied: {0}")]
    Denied(String),

    #[error("Invalid permission: {0}")]
    Invalid(String),

    #[error("Permission conflict: {0}")]
    Conflict(String),
}

pub type PermissionResult<T> = Result<T, PermissionError>;

/// Individual permissions that can be granted to plugins
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    // Geometry permissions
    GeometryRead,
    GeometryWrite,
    GeometryDelete,

    // Rendering permissions
    RenderingRead,
    RenderingWrite,
    RenderingShaderAccess,

    // UI permissions
    UIRead,
    UIWrite,
    UIMenuAccess,
    UIToolbarAccess,
    UIDialogAccess,

    // File I/O permissions
    FileRead,
    FileWrite,
    FileDelete,
    FileExecute,

    // Command permissions
    CommandExecute,
    CommandRegister,

    // Layer permissions
    LayerRead,
    LayerWrite,
    LayerDelete,

    // Dimension permissions
    DimensionRead,
    DimensionWrite,

    // Constraint permissions
    ConstraintRead,
    ConstraintWrite,

    // Network permissions
    NetworkAccess,
    NetworkHTTP,
    NetworkWebSocket,
    NetworkUnrestricted,

    // System permissions
    SystemClipboard,
    SystemNotifications,
    SystemProcess,
    SystemEnvironment,

    // Database permissions
    DatabaseRead,
    DatabaseWrite,
    DatabaseSchema,

    // Crypto permissions
    CryptoSign,
    CryptoEncrypt,
    CryptoRandom,

    // Enterprise permissions
    EnterpriseAccess,
    EnterpriseLicense,
    EnterpriseAudit,
    EnterpriseTelemetry,

    // Special permissions
    AllGeometry,
    AllRendering,
    AllUI,
    AllFile,
    AllNetwork,
    AllSystem,
    AllDatabase,
    AllCrypto,
    AllEnterprise,
    Administrator,
}

impl Permission {
    /// Check if this permission implies another permission
    pub fn implies(&self, other: &Permission) -> bool {
        match self {
            Permission::Administrator => true,
            Permission::AllGeometry => matches!(
                other,
                Permission::GeometryRead | Permission::GeometryWrite | Permission::GeometryDelete
            ),
            Permission::AllRendering => matches!(
                other,
                Permission::RenderingRead | Permission::RenderingWrite | Permission::RenderingShaderAccess
            ),
            Permission::AllUI => matches!(
                other,
                Permission::UIRead
                    | Permission::UIWrite
                    | Permission::UIMenuAccess
                    | Permission::UIToolbarAccess
                    | Permission::UIDialogAccess
            ),
            Permission::AllFile => matches!(
                other,
                Permission::FileRead | Permission::FileWrite | Permission::FileDelete | Permission::FileExecute
            ),
            Permission::AllNetwork => matches!(
                other,
                Permission::NetworkAccess
                    | Permission::NetworkHTTP
                    | Permission::NetworkWebSocket
                    | Permission::NetworkUnrestricted
            ),
            Permission::AllSystem => matches!(
                other,
                Permission::SystemClipboard
                    | Permission::SystemNotifications
                    | Permission::SystemProcess
                    | Permission::SystemEnvironment
            ),
            Permission::AllDatabase => matches!(
                other,
                Permission::DatabaseRead | Permission::DatabaseWrite | Permission::DatabaseSchema
            ),
            Permission::AllCrypto => matches!(
                other,
                Permission::CryptoSign | Permission::CryptoEncrypt | Permission::CryptoRandom
            ),
            Permission::AllEnterprise => matches!(
                other,
                Permission::EnterpriseAccess
                    | Permission::EnterpriseLicense
                    | Permission::EnterpriseAudit
                    | Permission::EnterpriseTelemetry
            ),
            _ => self == other,
        }
    }

    /// Get permission category
    pub fn category(&self) -> PermissionCategory {
        match self {
            Permission::GeometryRead | Permission::GeometryWrite | Permission::GeometryDelete | Permission::AllGeometry => {
                PermissionCategory::Geometry
            }
            Permission::RenderingRead
            | Permission::RenderingWrite
            | Permission::RenderingShaderAccess
            | Permission::AllRendering => PermissionCategory::Rendering,
            Permission::UIRead
            | Permission::UIWrite
            | Permission::UIMenuAccess
            | Permission::UIToolbarAccess
            | Permission::UIDialogAccess
            | Permission::AllUI => PermissionCategory::UI,
            Permission::FileRead
            | Permission::FileWrite
            | Permission::FileDelete
            | Permission::FileExecute
            | Permission::AllFile => PermissionCategory::File,
            Permission::CommandExecute | Permission::CommandRegister => PermissionCategory::Command,
            Permission::NetworkAccess
            | Permission::NetworkHTTP
            | Permission::NetworkWebSocket
            | Permission::NetworkUnrestricted
            | Permission::AllNetwork => PermissionCategory::Network,
            Permission::SystemClipboard
            | Permission::SystemNotifications
            | Permission::SystemProcess
            | Permission::SystemEnvironment
            | Permission::AllSystem => PermissionCategory::System,
            Permission::DatabaseRead | Permission::DatabaseWrite | Permission::DatabaseSchema | Permission::AllDatabase => {
                PermissionCategory::Database
            }
            Permission::CryptoSign | Permission::CryptoEncrypt | Permission::CryptoRandom | Permission::AllCrypto => {
                PermissionCategory::Crypto
            }
            Permission::EnterpriseAccess
            | Permission::EnterpriseLicense
            | Permission::EnterpriseAudit
            | Permission::EnterpriseTelemetry
            | Permission::AllEnterprise => PermissionCategory::Enterprise,
            Permission::LayerRead | Permission::LayerWrite | Permission::LayerDelete => PermissionCategory::Layer,
            Permission::DimensionRead | Permission::DimensionWrite => PermissionCategory::Dimension,
            Permission::ConstraintRead | Permission::ConstraintWrite => PermissionCategory::Constraint,
            Permission::Administrator => PermissionCategory::Special,
        }
    }

    /// Get risk level for this permission
    pub fn risk_level(&self) -> RiskLevel {
        match self {
            Permission::GeometryRead
            | Permission::RenderingRead
            | Permission::UIRead
            | Permission::LayerRead
            | Permission::DimensionRead
            | Permission::ConstraintRead
            | Permission::DatabaseRead => RiskLevel::Low,

            Permission::GeometryWrite
            | Permission::RenderingWrite
            | Permission::UIWrite
            | Permission::UIMenuAccess
            | Permission::UIToolbarAccess
            | Permission::FileRead
            | Permission::CommandExecute
            | Permission::LayerWrite
            | Permission::DimensionWrite
            | Permission::ConstraintWrite
            | Permission::SystemClipboard
            | Permission::SystemNotifications => RiskLevel::Medium,

            Permission::GeometryDelete
            | Permission::FileWrite
            | Permission::FileDelete
            | Permission::NetworkHTTP
            | Permission::NetworkWebSocket
            | Permission::DatabaseWrite
            | Permission::CryptoSign
            | Permission::CryptoEncrypt
            | Permission::LayerDelete
            | Permission::CommandRegister
            | Permission::UIDialogAccess => RiskLevel::High,

            Permission::FileExecute
            | Permission::NetworkUnrestricted
            | Permission::SystemProcess
            | Permission::SystemEnvironment
            | Permission::DatabaseSchema
            | Permission::RenderingShaderAccess
            | Permission::AllGeometry
            | Permission::AllRendering
            | Permission::AllUI
            | Permission::AllFile
            | Permission::AllNetwork
            | Permission::AllSystem
            | Permission::AllDatabase
            | Permission::AllCrypto
            | Permission::AllEnterprise
            | Permission::CryptoRandom
            | Permission::EnterpriseAccess
            | Permission::EnterpriseLicense
            | Permission::EnterpriseAudit
            | Permission::EnterpriseTelemetry
            | Permission::NetworkAccess
            | Permission::Administrator => RiskLevel::Critical,
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Permission::GeometryRead => "Read geometry data",
            Permission::GeometryWrite => "Create and modify geometry",
            Permission::GeometryDelete => "Delete geometry entities",
            Permission::RenderingRead => "Read rendering state",
            Permission::RenderingWrite => "Modify rendering settings",
            Permission::RenderingShaderAccess => "Access and modify shaders",
            Permission::UIRead => "Read UI state",
            Permission::UIWrite => "Modify UI elements",
            Permission::UIMenuAccess => "Add/modify menu items",
            Permission::UIToolbarAccess => "Add/modify toolbar buttons",
            Permission::UIDialogAccess => "Show custom dialogs",
            Permission::FileRead => "Read files from disk",
            Permission::FileWrite => "Write files to disk",
            Permission::FileDelete => "Delete files from disk",
            Permission::FileExecute => "Execute external programs",
            Permission::CommandExecute => "Execute commands",
            Permission::CommandRegister => "Register new commands",
            Permission::LayerRead => "Read layer information",
            Permission::LayerWrite => "Create and modify layers",
            Permission::LayerDelete => "Delete layers",
            Permission::DimensionRead => "Read dimension data",
            Permission::DimensionWrite => "Create and modify dimensions",
            Permission::ConstraintRead => "Read constraint data",
            Permission::ConstraintWrite => "Create and modify constraints",
            Permission::NetworkAccess => "General network access",
            Permission::NetworkHTTP => "Make HTTP requests",
            Permission::NetworkWebSocket => "Use WebSocket connections",
            Permission::NetworkUnrestricted => "Unrestricted network access",
            Permission::SystemClipboard => "Access system clipboard",
            Permission::SystemNotifications => "Show system notifications",
            Permission::SystemProcess => "Spawn and manage processes",
            Permission::SystemEnvironment => "Access environment variables",
            Permission::DatabaseRead => "Read from database",
            Permission::DatabaseWrite => "Write to database",
            Permission::DatabaseSchema => "Modify database schema",
            Permission::CryptoSign => "Sign data cryptographically",
            Permission::CryptoEncrypt => "Encrypt/decrypt data",
            Permission::CryptoRandom => "Generate cryptographic random data",
            Permission::EnterpriseAccess => "Access enterprise features",
            Permission::EnterpriseLicense => "Manage licenses",
            Permission::EnterpriseAudit => "Access audit logs",
            Permission::EnterpriseTelemetry => "Send telemetry data",
            Permission::AllGeometry => "Full geometry access",
            Permission::AllRendering => "Full rendering access",
            Permission::AllUI => "Full UI access",
            Permission::AllFile => "Full file system access",
            Permission::AllNetwork => "Full network access",
            Permission::AllSystem => "Full system access",
            Permission::AllDatabase => "Full database access",
            Permission::AllCrypto => "Full cryptographic access",
            Permission::AllEnterprise => "Full enterprise access",
            Permission::Administrator => "Complete system access",
        }
    }
}

/// Permission categories for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionCategory {
    Geometry,
    Rendering,
    UI,
    File,
    Command,
    Layer,
    Dimension,
    Constraint,
    Network,
    System,
    Database,
    Crypto,
    Enterprise,
    Special,
}

/// Risk levels for permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// A set of permissions granted to a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionSet {
    permissions: HashSet<Permission>,
    #[serde(default)]
    allow_list: Vec<String>,
    #[serde(default)]
    deny_list: Vec<String>,
}

impl PermissionSet {
    /// Create a new empty permission set
    pub fn new() -> Self {
        Self {
            permissions: HashSet::new(),
            allow_list: Vec::new(),
            deny_list: Vec::new(),
        }
    }

    /// Create permission set with specific permissions
    pub fn with_permissions(permissions: Vec<Permission>) -> Self {
        Self {
            permissions: permissions.into_iter().collect(),
            allow_list: Vec::new(),
            deny_list: Vec::new(),
        }
    }

    /// Create a minimal permission set (read-only access)
    pub fn minimal() -> Self {
        Self::with_permissions(vec![
            Permission::GeometryRead,
            Permission::RenderingRead,
            Permission::UIRead,
            Permission::LayerRead,
        ])
    }

    /// Create a standard permission set for typical plugins
    pub fn standard() -> Self {
        Self::with_permissions(vec![
            Permission::GeometryRead,
            Permission::GeometryWrite,
            Permission::RenderingRead,
            Permission::RenderingWrite,
            Permission::UIRead,
            Permission::UIWrite,
            Permission::UIMenuAccess,
            Permission::UIToolbarAccess,
            Permission::LayerRead,
            Permission::LayerWrite,
            Permission::CommandExecute,
            Permission::SystemNotifications,
        ])
    }

    /// Create an extended permission set
    pub fn extended() -> Self {
        let mut set = Self::standard();
        set.grant(Permission::FileRead);
        set.grant(Permission::FileWrite);
        set.grant(Permission::NetworkHTTP);
        set.grant(Permission::DatabaseRead);
        set.grant(Permission::DatabaseWrite);
        set.grant(Permission::CommandRegister);
        set
    }

    /// Grant a permission
    pub fn grant(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }

    /// Revoke a permission
    pub fn revoke(&mut self, permission: &Permission) {
        self.permissions.remove(permission);
    }

    /// Check if a permission is granted
    pub fn has(&self, permission: &Permission) -> bool {
        // Check if explicitly granted
        if self.permissions.contains(permission) {
            return true;
        }

        // Check if any granted permission implies this one
        self.permissions.iter().any(|p| p.implies(permission))
    }

    /// Get all granted permissions
    pub fn permissions(&self) -> &HashSet<Permission> {
        &self.permissions
    }

    /// Get maximum risk level in this permission set
    pub fn max_risk_level(&self) -> RiskLevel {
        self.permissions
            .iter()
            .map(|p| p.risk_level())
            .max()
            .unwrap_or(RiskLevel::Low)
    }

    /// Check if permission set is safe (no critical permissions)
    pub fn is_safe(&self) -> bool {
        self.max_risk_level() < RiskLevel::Critical
    }

    /// Add resource to allow list
    pub fn allow_resource(&mut self, resource: String) {
        self.allow_list.push(resource);
    }

    /// Add resource to deny list
    pub fn deny_resource(&mut self, resource: String) {
        self.deny_list.push(resource);
    }

    /// Check if resource is allowed
    pub fn is_resource_allowed(&self, resource: &str) -> bool {
        // Deny list takes precedence
        if self.deny_list.iter().any(|d| resource.starts_with(d)) {
            return false;
        }

        // If allow list is empty, allow everything not in deny list
        if self.allow_list.is_empty() {
            return true;
        }

        // Check if in allow list
        self.allow_list.iter().any(|a| resource.starts_with(a))
    }

    /// Validate this permission set
    pub fn validate(&self) -> PermissionResult<()> {
        // Check for conflicting permissions
        if self.has(&Permission::Administrator) && self.permissions.len() > 1 {
            return Err(PermissionError::Conflict(
                "Administrator permission should not be combined with other permissions".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for PermissionSet {
    fn default() -> Self {
        Self::minimal()
    }
}

/// Permission request from a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequest {
    pub plugin_id: String,
    pub requested_permissions: Vec<Permission>,
    pub justification: String,
}

/// Permission grant decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionGrant {
    Granted(PermissionSet),
    Denied(String),
    Partial {
        granted: PermissionSet,
        denied: Vec<Permission>,
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_implies() {
        assert!(Permission::Administrator.implies(&Permission::GeometryRead));
        assert!(Permission::AllGeometry.implies(&Permission::GeometryWrite));
        assert!(!Permission::GeometryRead.implies(&Permission::GeometryWrite));
    }

    #[test]
    fn test_permission_set() {
        let mut set = PermissionSet::new();
        set.grant(Permission::GeometryRead);

        assert!(set.has(&Permission::GeometryRead));
        assert!(!set.has(&Permission::GeometryWrite));

        set.grant(Permission::AllGeometry);
        assert!(set.has(&Permission::GeometryWrite));
    }

    #[test]
    fn test_risk_levels() {
        assert_eq!(Permission::GeometryRead.risk_level(), RiskLevel::Low);
        assert_eq!(Permission::FileWrite.risk_level(), RiskLevel::High);
        assert_eq!(Permission::Administrator.risk_level(), RiskLevel::Critical);
    }

    #[test]
    fn test_resource_filtering() {
        let mut set = PermissionSet::new();
        set.allow_resource("/home/user/plugins/".to_string());
        set.deny_resource("/home/user/plugins/system/".to_string());

        assert!(set.is_resource_allowed("/home/user/plugins/my-plugin/data.json"));
        assert!(!set.is_resource_allowed("/home/user/plugins/system/config.json"));
        assert!(!set.is_resource_allowed("/etc/passwd"));
    }
}
