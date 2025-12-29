//! # Multi-Tenant Isolation Engine
//!
//! Comprehensive multi-tenancy system for CADDY v0.2.0 with:
//!
//! - **Tenant Context Management**: Thread-local context, async propagation, hierarchical tenants
//! - **Resource Isolation**: Memory quotas, CPU limits, storage quotas, network bandwidth limits
//! - **Data Partitioning**: Schema-based isolation, row-level security, encryption key separation
//! - **Configuration Management**: Per-tenant feature flags, branding, configuration inheritance
//! - **Billing & Metering**: Resource usage tracking, API call counting, billing event generation
//! - **Lifecycle Management**: Provisioning, suspension, data export, GDPR-compliant deletion
//!
//! ## Architecture
//!
//! The multi-tenancy system implements a hierarchical tenant model:
//!
//! ```text
//! Organization
//!   └── Workspace
//!         └── Project
//! ```
//!
//! Each level can have its own configuration, quotas, and isolation policies.
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use caddy::enterprise::tenant::{
//!     TenantManager,
//!     context::{TenantId, TenantContext},
//!     config::Tier,
//!     lifecycle::ProvisioningRequest,
//! };
//! use uuid::Uuid;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create tenant manager
//! let manager = TenantManager::new();
//!
//! // Provision a new tenant
//! let org_id = Uuid::new_v4();
//! let tenant_id = TenantId::new_org(org_id);
//!
//! let request = ProvisioningRequest {
//!     tenant_id: tenant_id.clone(),
//!     name: "Acme Corporation".to_string(),
//!     tier: Tier::Enterprise,
//!     admin_email: "admin@acme.com".to_string(),
//!     config_overrides: None,
//!     metadata: std::collections::HashMap::new(),
//! };
//!
//! let result = manager.provision_tenant(request).await?;
//!
//! // Set tenant context for current operation
//! let context = TenantContext::new(tenant_id.clone());
//! manager.set_context(context)?;
//!
//! // All operations now automatically isolated to this tenant
//! // ...
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Security
//!
//! - **Isolation**: Complete data isolation between tenants
//! - **Encryption**: Separate encryption keys per tenant
//! - **Access Control**: Hierarchical permission model
//! - **Audit**: All tenant operations are logged
//!
//! ## Compliance
//!
//! - **GDPR**: Right to access, portability, and deletion
//! - **SOC 2**: Audit logging and access controls
//! - **Data Residency**: Configurable data storage locations

#![warn(missing_docs)]

// Module declarations
pub mod context;
pub mod isolation;
pub mod partition;
pub mod config;
pub mod metering;
pub mod lifecycle;

// Re-exports for convenience
pub use context::{
    TenantId, TenantContext, ContextError, ContextResult,
    get_context, set_context, clear_context, with_context,
    ContextGuard, ContextSwitcher,
};

pub use isolation::{
    TenantIsolationManager, ResourceQuotas, ResourceUsageSnapshot,
    IsolationError, IsolationResult, MemoryGuard, ConnectionGuard,
};

pub use partition::{
    DataPartitionManager, SchemaConfig, PartitionStrategy,
    TenantPartitionInfo, RlsPolicy, TenantQuery, TenantDao,
    PartitionError, PartitionResult,
};

pub use config::{
    ConfigManager, TenantConfig, FeatureFlags, BrandingConfig,
    UiPreferences, Tier, ConfigError, ConfigResult,
};

pub use metering::{
    MeteringManager, MetricType, UsageRecord, BillingPeriodUsage,
    PricingModel, MetricPricing, BillingEvent, BillingEventType,
    MeteringError, MeteringResult,
};

pub use lifecycle::{
    TenantLifecycleManager, TenantInfo, TenantState,
    ProvisioningRequest, ProvisioningResult, ExportRequest,
    ExportResult, ExportFormat, DeletionRequest,
    LifecycleError, LifecycleResult,
};

use std::sync::Arc;
use parking_lot::RwLock;
use thiserror::Error;

/// Unified tenant management errors
#[derive(Error, Debug)]
pub enum TenantError {
    /// Context-related error
    #[error("Context error: {0}")]
    Context(#[from] ContextError),

    /// Isolation/quota error
    #[error("Isolation error: {0}")]
    Isolation(#[from] IsolationError),

    /// Partition/data error
    #[error("Partition error: {0}")]
    Partition(#[from] PartitionError),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Metering error
    #[error("Metering error: {0}")]
    Metering(#[from] MeteringError),

    /// Lifecycle error
    #[error("Lifecycle error: {0}")]
    Lifecycle(#[from] LifecycleError),

    /// Generic tenant error
    #[error("Tenant error: {0}")]
    Other(String),
}

/// Result type for tenant operations
pub type TenantResult<T> = Result<T, TenantError>;

/// Central tenant management system
///
/// Coordinates all tenant-related operations including lifecycle, isolation,
/// configuration, and metering.
pub struct TenantManager {
    lifecycle: Arc<TenantLifecycleManager>,
    isolation: Arc<TenantIsolationManager>,
    partition: Arc<DataPartitionManager>,
    config: Arc<ConfigManager>,
    metering: Arc<MeteringManager>,
}

impl TenantManager {
    /// Create a new tenant manager with default settings
    pub fn new() -> Self {
        Self {
            lifecycle: Arc::new(TenantLifecycleManager::new()),
            isolation: Arc::new(TenantIsolationManager::new()),
            partition: Arc::new(DataPartitionManager::new(SchemaConfig::default())),
            config: Arc::new(ConfigManager::new()),
            metering: Arc::new(MeteringManager::default()),
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        schema_config: SchemaConfig,
        pricing: PricingModel,
    ) -> Self {
        Self {
            lifecycle: Arc::new(TenantLifecycleManager::new()),
            isolation: Arc::new(TenantIsolationManager::new()),
            partition: Arc::new(DataPartitionManager::new(schema_config)),
            config: Arc::new(ConfigManager::new()),
            metering: Arc::new(MeteringManager::new(pricing)),
        }
    }

    /// Provision a new tenant (async for future DB operations)
    pub async fn provision_tenant(
        &self,
        request: ProvisioningRequest,
    ) -> TenantResult<ProvisioningResult> {
        // Provision tenant
        let result = self.lifecycle.provision(request.clone())?;

        // Set up quotas based on tier
        let quotas = match request.tier {
            Tier::Enterprise => ResourceQuotas::enterprise(),
            Tier::Professional => ResourceQuotas::default(),
            Tier::Basic => ResourceQuotas::basic(),
        };
        self.isolation.set_quotas(request.tenant_id.clone(), quotas);

        // Initialize data partition
        self.partition.initialize_tenant(request.tenant_id.clone())?;

        // Create configuration
        self.config.create_config(request.tenant_id.clone(), request.tier);

        // Initialize metering
        self.metering.initialize_tenant(request.tenant_id.clone());

        Ok(result)
    }

    /// Get tenant information
    pub fn get_tenant(&self, tenant_id: &TenantId) -> TenantResult<TenantInfo> {
        Ok(self.lifecycle.get_tenant(tenant_id)?)
    }

    /// Set tenant context for current thread
    pub fn set_context(&self, context: TenantContext) -> TenantResult<()> {
        set_context(context)?;
        Ok(())
    }

    /// Get current tenant context
    pub fn get_current_context(&self) -> TenantResult<TenantContext> {
        Ok(get_context()?)
    }

    /// Check resource quota and allocate if available
    pub fn allocate_memory(&self, tenant_id: &TenantId, bytes: u64) -> TenantResult<MemoryGuard> {
        Ok(MemoryGuard::new(&self.isolation, tenant_id.clone(), bytes)?)
    }

    /// Record API call for metering
    pub fn record_api_call(&self, tenant_id: &TenantId) -> TenantResult<()> {
        self.isolation.check_rate_limit(tenant_id)?;
        self.metering.record_usage(tenant_id, MetricType::ApiCalls, 1)?;
        Ok(())
    }

    /// Get tenant configuration
    pub fn get_config(&self, tenant_id: &TenantId) -> TenantResult<TenantConfig> {
        Ok(self.config.get_config(tenant_id)?)
    }

    /// Check if feature is enabled for tenant
    pub fn is_feature_enabled(&self, tenant_id: &TenantId, feature: &str) -> bool {
        self.config.is_feature_enabled(tenant_id, feature)
    }

    /// Get current billing period usage
    pub fn get_current_usage(&self, tenant_id: &TenantId) -> Option<BillingPeriodUsage> {
        self.metering.get_current_usage(tenant_id)
    }

    /// Suspend a tenant
    pub fn suspend_tenant(&self, tenant_id: &TenantId, reason: String) -> TenantResult<()> {
        Ok(self.lifecycle.suspend(tenant_id, reason)?)
    }

    /// Reactivate a suspended tenant
    pub fn reactivate_tenant(&self, tenant_id: &TenantId) -> TenantResult<()> {
        Ok(self.lifecycle.reactivate(tenant_id)?)
    }

    /// Export tenant data
    pub fn export_tenant_data(&self, request: ExportRequest) -> TenantResult<ExportResult> {
        Ok(self.lifecycle.export_data(request)?)
    }

    /// Schedule tenant deletion
    pub fn delete_tenant(&self, request: DeletionRequest) -> TenantResult<chrono::DateTime<chrono::Utc>> {
        Ok(self.lifecycle.schedule_deletion(request)?)
    }

    /// Get lifecycle manager
    pub fn lifecycle(&self) -> &Arc<TenantLifecycleManager> {
        &self.lifecycle
    }

    /// Get isolation manager
    pub fn isolation(&self) -> &Arc<TenantIsolationManager> {
        &self.isolation
    }

    /// Get partition manager
    pub fn partition(&self) -> &Arc<DataPartitionManager> {
        &self.partition
    }

    /// Get config manager
    pub fn config_manager(&self) -> &Arc<ConfigManager> {
        &self.config
    }

    /// Get metering manager
    pub fn metering(&self) -> &Arc<MeteringManager> {
        &self.metering
    }

    /// List all tenants
    pub fn list_tenants(&self) -> Vec<TenantInfo> {
        self.lifecycle.list_tenants()
    }

    /// List tenants by state
    pub fn list_by_state(&self, state: TenantState) -> Vec<TenantInfo> {
        self.lifecycle.list_by_state(state)
    }
}

impl Default for TenantManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global tenant manager instance
static GLOBAL_TENANT_MANAGER: once_cell::sync::Lazy<Arc<RwLock<Option<TenantManager>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(None)));

/// Initialize the global tenant manager
pub fn initialize_global_manager(manager: TenantManager) {
    *GLOBAL_TENANT_MANAGER.write() = Some(manager);
}

/// Get the global tenant manager
pub fn global_manager() -> Option<TenantManager> {
    GLOBAL_TENANT_MANAGER.read().as_ref().map(|m| {
        // Clone the Arc references, not the actual data
        TenantManager {
            lifecycle: Arc::clone(&m.lifecycle),
            isolation: Arc::clone(&m.isolation),
            partition: Arc::clone(&m.partition),
            config: Arc::clone(&m.config),
            metering: Arc::clone(&m.metering),
        }
    })
}

/// Middleware for automatic tenant context injection
pub mod middleware {
    use super::*;

    /// Extract tenant ID from request header
    pub fn extract_tenant_from_header(header_value: &str) -> Option<TenantId> {
        // Parse format: "org:UUID" or "org:UUID/ws:UUID" or "org:UUID/ws:UUID/proj:UUID"
        let parts: Vec<&str> = header_value.split('/').collect();

        if parts.is_empty() {
            return None;
        }

        // Parse org
        let org_part = parts[0].strip_prefix("org:")?;
        let org_id = uuid::Uuid::parse_str(org_part).ok()?;

        if parts.len() == 1 {
            return Some(TenantId::new_org(org_id));
        }

        // Parse workspace
        let ws_part = parts[1].strip_prefix("ws:")?;
        let ws_id = uuid::Uuid::parse_str(ws_part).ok()?;

        if parts.len() == 2 {
            return Some(TenantId::new_workspace(org_id, ws_id));
        }

        // Parse project
        let proj_part = parts[2].strip_prefix("proj:")?;
        let proj_id = uuid::Uuid::parse_str(proj_part).ok()?;

        Some(TenantId::new_project(org_id, ws_id, proj_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_end_to_end_tenant_lifecycle() {
        let manager = TenantManager::new();
        let org_id = Uuid::new_v4();
        let tenant_id = TenantId::new_org(org_id);

        // Provision
        let request = ProvisioningRequest {
            tenant_id: tenant_id.clone(),
            name: "Test Organization".to_string(),
            tier: Tier::Professional,
            admin_email: "admin@test.com".to_string(),
            config_overrides: None,
            metadata: HashMap::new(),
        };

        let result = manager.provision_tenant(request).await.unwrap();
        assert_eq!(result.tenant_info.state, TenantState::Active);

        // Set context
        let context = TenantContext::new(tenant_id.clone());
        manager.set_context(context).unwrap();

        // Check feature
        assert!(manager.is_feature_enabled(&tenant_id, "collaboration"));

        // Record usage
        manager.record_api_call(&tenant_id).unwrap();

        let usage = manager.get_current_usage(&tenant_id).unwrap();
        assert_eq!(usage.get_usage(MetricType::ApiCalls), 1);

        // Suspend
        manager.suspend_tenant(&tenant_id, "Test suspension".to_string()).unwrap();
        let info = manager.get_tenant(&tenant_id).unwrap();
        assert_eq!(info.state, TenantState::Suspended);

        // Reactivate
        manager.reactivate_tenant(&tenant_id).unwrap();
        let info = manager.get_tenant(&tenant_id).unwrap();
        assert_eq!(info.state, TenantState::Active);
    }

    #[test]
    fn test_tenant_hierarchy() {
        let org_id = Uuid::new_v4();
        let ws_id = Uuid::new_v4();
        let proj_id = Uuid::new_v4();

        let org_tenant = TenantId::new_org(org_id);
        let ws_tenant = TenantId::new_workspace(org_id, ws_id);
        let proj_tenant = TenantId::new_project(org_id, ws_id, proj_id);

        assert!(org_tenant.is_ancestor_of(&ws_tenant));
        assert!(org_tenant.is_ancestor_of(&proj_tenant));
        assert!(ws_tenant.is_ancestor_of(&proj_tenant));
    }

    #[test]
    fn test_middleware_tenant_extraction() {
        use middleware::extract_tenant_from_header;

        let org_id = Uuid::new_v4();
        let header = format!("org:{}", org_id);

        let tenant_id = extract_tenant_from_header(&header).unwrap();
        assert_eq!(tenant_id.org_id, org_id);
        assert_eq!(tenant_id.level(), 1);

        let ws_id = Uuid::new_v4();
        let header = format!("org:{}/ws:{}", org_id, ws_id);

        let tenant_id = extract_tenant_from_header(&header).unwrap();
        assert_eq!(tenant_id.level(), 2);
    }

    #[test]
    fn test_global_manager() {
        let manager = TenantManager::new();
        initialize_global_manager(manager);

        let global = global_manager().unwrap();
        assert!(global.list_tenants().is_empty());
    }
}
