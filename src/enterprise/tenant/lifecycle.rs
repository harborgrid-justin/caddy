//! Tenant Lifecycle Management
//!
//! Handles tenant provisioning, suspension, activation, data export, and deletion (GDPR compliance).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::RwLock;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use chrono::{DateTime, Utc};

use super::context::TenantId;
use super::config::{TenantConfig, Tier};

/// Lifecycle management errors
#[derive(Error, Debug)]
pub enum LifecycleError {
    #[error("Tenant already exists: {0}")]
    AlreadyExists(String),

    #[error("Tenant not found: {0}")]
    NotFound(String),

    #[error("Tenant is in invalid state for operation: {0}")]
    InvalidState(String),

    #[error("Provisioning failed: {0}")]
    ProvisioningFailed(String),

    #[error("Deletion failed: {0}")]
    DeletionFailed(String),

    #[error("Export failed: {0}")]
    ExportFailed(String),

    #[error("Retention policy violation: {0}")]
    RetentionViolation(String),
}

pub type LifecycleResult<T> = Result<T, LifecycleError>;

/// Tenant lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TenantState {
    /// Tenant is being provisioned
    Provisioning,
    /// Tenant is active and operational
    Active,
    /// Tenant is suspended (temporary)
    Suspended,
    /// Tenant is deactivated (can be reactivated)
    Deactivated,
    /// Tenant is being deleted
    Deleting,
    /// Tenant has been deleted
    Deleted,
}

impl std::fmt::Display for TenantState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TenantState::Provisioning => write!(f, "provisioning"),
            TenantState::Active => write!(f, "active"),
            TenantState::Suspended => write!(f, "suspended"),
            TenantState::Deactivated => write!(f, "deactivated"),
            TenantState::Deleting => write!(f, "deleting"),
            TenantState::Deleted => write!(f, "deleted"),
        }
    }
}

/// Tenant information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantInfo {
    /// Tenant identifier
    pub tenant_id: TenantId,
    /// Organization name
    pub name: String,
    /// Current state
    pub state: TenantState,
    /// Subscription tier
    pub tier: Tier,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Suspension reason (if suspended)
    pub suspension_reason: Option<String>,
    /// Deletion scheduled date
    pub deletion_scheduled_at: Option<DateTime<Utc>>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

impl TenantInfo {
    /// Create new tenant info
    pub fn new(tenant_id: TenantId, name: String, tier: Tier) -> Self {
        Self {
            tenant_id,
            name,
            state: TenantState::Provisioning,
            tier,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            suspension_reason: None,
            deletion_scheduled_at: None,
            metadata: HashMap::new(),
        }
    }

    /// Check if tenant is operational
    pub fn is_operational(&self) -> bool {
        matches!(self.state, TenantState::Active)
    }

    /// Check if tenant can be accessed
    pub fn is_accessible(&self) -> bool {
        matches!(self.state, TenantState::Active | TenantState::Suspended)
    }
}

/// Provisioning request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningRequest {
    /// Tenant identifier
    pub tenant_id: TenantId,
    /// Organization name
    pub name: String,
    /// Subscription tier
    pub tier: Tier,
    /// Initial admin user email
    pub admin_email: String,
    /// Configuration overrides
    pub config_overrides: Option<HashMap<String, serde_json::Value>>,
    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

/// Provisioning result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningResult {
    /// Tenant info
    pub tenant_info: TenantInfo,
    /// Configuration
    pub config: TenantConfig,
    /// Initial admin credentials (if created)
    pub admin_credentials: Option<AdminCredentials>,
}

/// Admin credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminCredentials {
    /// User ID
    pub user_id: uuid::Uuid,
    /// Email
    pub email: String,
    /// Temporary password (should be changed on first login)
    pub temp_password: String,
}

/// Data export format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    /// JSON format
    Json,
    /// CSV files
    Csv,
    /// SQL dump
    Sql,
    /// Complete archive (all formats)
    Archive,
}

/// Export request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    /// Tenant to export
    pub tenant_id: TenantId,
    /// Export format
    pub format: ExportFormat,
    /// Include metadata
    pub include_metadata: bool,
    /// Include user data
    pub include_users: bool,
    /// Include audit logs
    pub include_audit_logs: bool,
    /// Encryption key for export (optional)
    pub encryption_key: Option<String>,
}

/// Export result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    /// Export ID
    pub export_id: uuid::Uuid,
    /// Tenant ID
    pub tenant_id: TenantId,
    /// File path or URL
    pub location: String,
    /// File size in bytes
    pub size_bytes: u64,
    /// Export timestamp
    pub exported_at: DateTime<Utc>,
    /// Checksum (SHA256)
    pub checksum: String,
}

/// Deletion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletionRequest {
    /// Tenant to delete
    pub tenant_id: TenantId,
    /// Reason for deletion
    pub reason: String,
    /// Immediate deletion (bypass grace period)
    pub immediate: bool,
    /// Export data before deletion
    pub export_before_delete: bool,
}

/// Tenant lifecycle manager
pub struct TenantLifecycleManager {
    /// Tenant information registry
    tenants: DashMap<TenantId, Arc<RwLock<TenantInfo>>>,
    /// Deletion grace period in days
    deletion_grace_period_days: u32,
    /// Data retention period in days (for compliance)
    data_retention_days: u32,
}

impl TenantLifecycleManager {
    /// Create a new lifecycle manager
    pub fn new() -> Self {
        Self {
            tenants: DashMap::new(),
            deletion_grace_period_days: 30,
            data_retention_days: 2555, // 7 years for compliance
        }
    }

    /// Provision a new tenant
    pub fn provision(&self, request: ProvisioningRequest) -> LifecycleResult<ProvisioningResult> {
        // Check if tenant already exists
        if self.tenants.contains_key(&request.tenant_id) {
            return Err(LifecycleError::AlreadyExists(request.tenant_id.to_string()));
        }

        // Create tenant info
        let mut tenant_info = TenantInfo::new(
            request.tenant_id.clone(),
            request.name.clone(),
            request.tier,
        );
        tenant_info.metadata = request.metadata;

        // Create configuration
        let config = TenantConfig::with_tier(request.tenant_id.clone(), request.tier);

        // Generate admin credentials
        let admin_credentials = AdminCredentials {
            user_id: uuid::Uuid::new_v4(),
            email: request.admin_email.clone(),
            temp_password: self.generate_temp_password(),
        };

        // TODO: In production, this would:
        // 1. Create database schema/namespace
        // 2. Initialize storage buckets
        // 3. Create admin user
        // 4. Set up encryption keys
        // 5. Initialize audit logging
        // 6. Send welcome email

        // Update state to active
        tenant_info.state = TenantState::Active;
        tenant_info.updated_at = Utc::now();

        // Store tenant info
        self.tenants.insert(
            request.tenant_id.clone(),
            Arc::new(RwLock::new(tenant_info.clone())),
        );

        Ok(ProvisioningResult {
            tenant_info,
            config,
            admin_credentials: Some(admin_credentials),
        })
    }

    /// Get tenant information
    pub fn get_tenant(&self, tenant_id: &TenantId) -> LifecycleResult<TenantInfo> {
        self.tenants
            .get(tenant_id)
            .map(|info| info.read().clone())
            .ok_or_else(|| LifecycleError::NotFound(tenant_id.to_string()))
    }

    /// Suspend a tenant
    pub fn suspend(&self, tenant_id: &TenantId, reason: String) -> LifecycleResult<()> {
        let info_ref = self.tenants
            .get(tenant_id)
            .ok_or_else(|| LifecycleError::NotFound(tenant_id.to_string()))?;

        let mut info = info_ref.write();

        if !matches!(info.state, TenantState::Active) {
            return Err(LifecycleError::InvalidState(
                format!("Cannot suspend tenant in state: {}", info.state)
            ));
        }

        info.state = TenantState::Suspended;
        info.suspension_reason = Some(reason);
        info.updated_at = Utc::now();

        // TODO: In production, this would:
        // 1. Block all API access
        // 2. Pause scheduled jobs
        // 3. Send notification to admins
        // 4. Log suspension event

        Ok(())
    }

    /// Reactivate a suspended tenant
    pub fn reactivate(&self, tenant_id: &TenantId) -> LifecycleResult<()> {
        let info_ref = self.tenants
            .get(tenant_id)
            .ok_or_else(|| LifecycleError::NotFound(tenant_id.to_string()))?;

        let mut info = info_ref.write();

        if !matches!(info.state, TenantState::Suspended | TenantState::Deactivated) {
            return Err(LifecycleError::InvalidState(
                format!("Cannot reactivate tenant in state: {}", info.state)
            ));
        }

        info.state = TenantState::Active;
        info.suspension_reason = None;
        info.updated_at = Utc::now();

        // TODO: Restore access and resume operations

        Ok(())
    }

    /// Deactivate a tenant (long-term suspension)
    pub fn deactivate(&self, tenant_id: &TenantId) -> LifecycleResult<()> {
        let info_ref = self.tenants
            .get(tenant_id)
            .ok_or_else(|| LifecycleError::NotFound(tenant_id.to_string()))?;

        let mut info = info_ref.write();

        info.state = TenantState::Deactivated;
        info.updated_at = Utc::now();

        // TODO: Archive data, reduce resource allocation

        Ok(())
    }

    /// Export tenant data
    pub fn export_data(&self, request: ExportRequest) -> LifecycleResult<ExportResult> {
        let _tenant_info = self.get_tenant(&request.tenant_id)?;

        // TODO: In production, this would:
        // 1. Collect all tenant data
        // 2. Format according to requested format
        // 3. Encrypt if requested
        // 4. Upload to secure storage
        // 5. Generate download link

        let export_id = uuid::Uuid::new_v4();
        let location = format!("/exports/{}.{:?}", export_id, request.format);

        Ok(ExportResult {
            export_id,
            tenant_id: request.tenant_id,
            location,
            size_bytes: 0, // Would be calculated
            exported_at: Utc::now(),
            checksum: "placeholder".to_string(),
        })
    }

    /// Schedule tenant deletion (GDPR right to be forgotten)
    pub fn schedule_deletion(&self, request: DeletionRequest) -> LifecycleResult<DateTime<Utc>> {
        let info_ref = self.tenants
            .get(&request.tenant_id)
            .ok_or_else(|| LifecycleError::NotFound(request.tenant_id.to_string()))?;

        let mut info = info_ref.write();

        // Calculate deletion date
        let deletion_date = if request.immediate {
            Utc::now()
        } else {
            Utc::now() + chrono::Duration::days(self.deletion_grace_period_days as i64)
        };

        info.state = TenantState::Deleting;
        info.deletion_scheduled_at = Some(deletion_date);
        info.updated_at = Utc::now();

        // TODO: Schedule background job for deletion

        Ok(deletion_date)
    }

    /// Cancel scheduled deletion
    pub fn cancel_deletion(&self, tenant_id: &TenantId) -> LifecycleResult<()> {
        let info_ref = self.tenants
            .get(tenant_id)
            .ok_or_else(|| LifecycleError::NotFound(tenant_id.to_string()))?;

        let mut info = info_ref.write();

        if info.state != TenantState::Deleting {
            return Err(LifecycleError::InvalidState(
                "Tenant is not scheduled for deletion".to_string()
            ));
        }

        info.state = TenantState::Active;
        info.deletion_scheduled_at = None;
        info.updated_at = Utc::now();

        Ok(())
    }

    /// Execute tenant deletion (permanent)
    pub fn execute_deletion(&self, tenant_id: &TenantId) -> LifecycleResult<()> {
        let info_ref = self.tenants
            .get(tenant_id)
            .ok_or_else(|| LifecycleError::NotFound(tenant_id.to_string()))?;

        let mut info = info_ref.write();

        // TODO: In production, this would:
        // 1. Delete all tenant data from database
        // 2. Remove storage buckets and files
        // 3. Revoke encryption keys
        // 4. Remove from all indexes
        // 5. Log deletion for compliance
        // 6. Send confirmation email

        info.state = TenantState::Deleted;
        info.updated_at = Utc::now();

        Ok(())
    }

    /// Permanently remove tenant record
    pub fn purge_tenant(&self, tenant_id: &TenantId) -> LifecycleResult<()> {
        let info = self.get_tenant(tenant_id)?;

        if info.state != TenantState::Deleted {
            return Err(LifecycleError::InvalidState(
                "Tenant must be in deleted state before purging".to_string()
            ));
        }

        self.tenants.remove(tenant_id);
        Ok(())
    }

    /// List all tenants
    pub fn list_tenants(&self) -> Vec<TenantInfo> {
        self.tenants
            .iter()
            .map(|entry| entry.read().clone())
            .collect()
    }

    /// List tenants by state
    pub fn list_by_state(&self, state: TenantState) -> Vec<TenantInfo> {
        self.tenants
            .iter()
            .filter(|entry| entry.read().state == state)
            .map(|entry| entry.read().clone())
            .collect()
    }

    /// Get tenants scheduled for deletion
    pub fn get_pending_deletions(&self) -> Vec<TenantInfo> {
        self.tenants
            .iter()
            .filter(|entry| {
                let info = entry.read();
                info.state == TenantState::Deleting
                    && info.deletion_scheduled_at.is_some()
                    && info.deletion_scheduled_at.unwrap() <= Utc::now()
            })
            .map(|entry| entry.read().clone())
            .collect()
    }

    /// Generate temporary password
    fn generate_temp_password(&self) -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                 abcdefghijklmnopqrstuvwxyz\
                                 0123456789\
                                 !@#$%^&*";
        let mut rng = rand::thread_rng();

        (0..16)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }
}

impl Default for TenantLifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_tenant_provisioning() {
        let manager = TenantLifecycleManager::new();
        let tenant_id = TenantId::new_org(Uuid::new_v4());

        let request = ProvisioningRequest {
            tenant_id: tenant_id.clone(),
            name: "Test Org".to_string(),
            tier: Tier::Professional,
            admin_email: "admin@example.com".to_string(),
            config_overrides: None,
            metadata: HashMap::new(),
        };

        let result = manager.provision(request).unwrap();
        assert_eq!(result.tenant_info.state, TenantState::Active);
        assert!(result.admin_credentials.is_some());

        let tenant_info = manager.get_tenant(&tenant_id).unwrap();
        assert_eq!(tenant_info.name, "Test Org");
    }

    #[test]
    fn test_duplicate_provisioning() {
        let manager = TenantLifecycleManager::new();
        let tenant_id = TenantId::new_org(Uuid::new_v4());

        let request = ProvisioningRequest {
            tenant_id: tenant_id.clone(),
            name: "Test Org".to_string(),
            tier: Tier::Professional,
            admin_email: "admin@example.com".to_string(),
            config_overrides: None,
            metadata: HashMap::new(),
        };

        manager.provision(request.clone()).unwrap();
        assert!(manager.provision(request).is_err());
    }

    #[test]
    fn test_tenant_suspension() {
        let manager = TenantLifecycleManager::new();
        let tenant_id = TenantId::new_org(Uuid::new_v4());

        let request = ProvisioningRequest {
            tenant_id: tenant_id.clone(),
            name: "Test Org".to_string(),
            tier: Tier::Professional,
            admin_email: "admin@example.com".to_string(),
            config_overrides: None,
            metadata: HashMap::new(),
        };

        manager.provision(request).unwrap();
        manager.suspend(&tenant_id, "Payment overdue".to_string()).unwrap();

        let tenant_info = manager.get_tenant(&tenant_id).unwrap();
        assert_eq!(tenant_info.state, TenantState::Suspended);
        assert_eq!(tenant_info.suspension_reason, Some("Payment overdue".to_string()));
    }

    #[test]
    fn test_tenant_reactivation() {
        let manager = TenantLifecycleManager::new();
        let tenant_id = TenantId::new_org(Uuid::new_v4());

        let request = ProvisioningRequest {
            tenant_id: tenant_id.clone(),
            name: "Test Org".to_string(),
            tier: Tier::Professional,
            admin_email: "admin@example.com".to_string(),
            config_overrides: None,
            metadata: HashMap::new(),
        };

        manager.provision(request).unwrap();
        manager.suspend(&tenant_id, "Test".to_string()).unwrap();
        manager.reactivate(&tenant_id).unwrap();

        let tenant_info = manager.get_tenant(&tenant_id).unwrap();
        assert_eq!(tenant_info.state, TenantState::Active);
        assert_eq!(tenant_info.suspension_reason, None);
    }

    #[test]
    fn test_deletion_scheduling() {
        let manager = TenantLifecycleManager::new();
        let tenant_id = TenantId::new_org(Uuid::new_v4());

        let request = ProvisioningRequest {
            tenant_id: tenant_id.clone(),
            name: "Test Org".to_string(),
            tier: Tier::Professional,
            admin_email: "admin@example.com".to_string(),
            config_overrides: None,
            metadata: HashMap::new(),
        };

        manager.provision(request).unwrap();

        let deletion_request = DeletionRequest {
            tenant_id: tenant_id.clone(),
            reason: "User requested".to_string(),
            immediate: false,
            export_before_delete: true,
        };

        let deletion_date = manager.schedule_deletion(deletion_request).unwrap();
        assert!(deletion_date > Utc::now());

        let tenant_info = manager.get_tenant(&tenant_id).unwrap();
        assert_eq!(tenant_info.state, TenantState::Deleting);
    }

    #[test]
    fn test_deletion_cancellation() {
        let manager = TenantLifecycleManager::new();
        let tenant_id = TenantId::new_org(Uuid::new_v4());

        let request = ProvisioningRequest {
            tenant_id: tenant_id.clone(),
            name: "Test Org".to_string(),
            tier: Tier::Professional,
            admin_email: "admin@example.com".to_string(),
            config_overrides: None,
            metadata: HashMap::new(),
        };

        manager.provision(request).unwrap();

        let deletion_request = DeletionRequest {
            tenant_id: tenant_id.clone(),
            reason: "Test".to_string(),
            immediate: false,
            export_before_delete: false,
        };

        manager.schedule_deletion(deletion_request).unwrap();
        manager.cancel_deletion(&tenant_id).unwrap();

        let tenant_info = manager.get_tenant(&tenant_id).unwrap();
        assert_eq!(tenant_info.state, TenantState::Active);
    }

    #[test]
    fn test_list_by_state() {
        let manager = TenantLifecycleManager::new();

        for i in 0..3 {
            let tenant_id = TenantId::new_org(Uuid::new_v4());
            let request = ProvisioningRequest {
                tenant_id: tenant_id.clone(),
                name: format!("Org {}", i),
                tier: Tier::Professional,
                admin_email: format!("admin{}@example.com", i),
                config_overrides: None,
                metadata: HashMap::new(),
            };
            manager.provision(request).unwrap();
        }

        let active_tenants = manager.list_by_state(TenantState::Active);
        assert_eq!(active_tenants.len(), 3);
    }
}
