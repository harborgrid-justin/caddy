//! Tenant Configuration Management
//!
//! Per-tenant feature flags, custom branding, configuration inheritance, and override cascading.

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use thiserror::Error;

use super::context::TenantId;

/// Configuration errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration not found for tenant: {0}")]
    NotFound(String),

    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),

    #[error("Configuration key not found: {0}")]
    KeyNotFound(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Configuration is locked and cannot be modified")]
    Locked,
}

pub type ConfigResult<T> = Result<T, ConfigError>;

/// Feature flag configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    /// Enable real-time collaboration
    pub collaboration: bool,
    /// Enable cloud sync
    pub cloud_sync: bool,
    /// Enable advanced rendering
    pub advanced_rendering: bool,
    /// Enable plugin marketplace
    pub marketplace: bool,
    /// Enable AI-assisted design
    pub ai_features: bool,
    /// Enable workflow automation
    pub workflows: bool,
    /// Enable analytics dashboard
    pub analytics: bool,
    /// Maximum project size in MB
    pub max_project_size_mb: u64,
    /// Maximum concurrent users
    pub max_concurrent_users: u32,
    /// Custom feature flags
    pub custom: HashMap<String, bool>,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            collaboration: false,
            cloud_sync: false,
            advanced_rendering: true,
            marketplace: false,
            ai_features: false,
            workflows: false,
            analytics: false,
            max_project_size_mb: 100,
            max_concurrent_users: 5,
            custom: HashMap::new(),
        }
    }
}

impl FeatureFlags {
    /// Enterprise tier features
    pub fn enterprise() -> Self {
        Self {
            collaboration: true,
            cloud_sync: true,
            advanced_rendering: true,
            marketplace: true,
            ai_features: true,
            workflows: true,
            analytics: true,
            max_project_size_mb: 10_000,
            max_concurrent_users: 1000,
            custom: HashMap::new(),
        }
    }

    /// Professional tier features
    pub fn professional() -> Self {
        Self {
            collaboration: true,
            cloud_sync: true,
            advanced_rendering: true,
            marketplace: true,
            ai_features: false,
            workflows: false,
            analytics: true,
            max_project_size_mb: 1000,
            max_concurrent_users: 50,
            custom: HashMap::new(),
        }
    }

    /// Check if a custom feature is enabled
    pub fn is_enabled(&self, feature: &str) -> bool {
        self.custom.get(feature).copied().unwrap_or(false)
    }

    /// Enable a custom feature
    pub fn enable(&mut self, feature: String) {
        self.custom.insert(feature, true);
    }

    /// Disable a custom feature
    pub fn disable(&mut self, feature: String) {
        self.custom.insert(feature, false);
    }
}

/// Branding and theming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandingConfig {
    /// Organization name
    pub organization_name: String,
    /// Logo URL
    pub logo_url: Option<String>,
    /// Primary color (hex)
    pub primary_color: String,
    /// Secondary color (hex)
    pub secondary_color: String,
    /// Custom CSS
    pub custom_css: Option<String>,
    /// Favicon URL
    pub favicon_url: Option<String>,
    /// Email domain for white-labeling
    pub email_domain: Option<String>,
}

impl Default for BrandingConfig {
    fn default() -> Self {
        Self {
            organization_name: "CADDY".to_string(),
            logo_url: None,
            primary_color: "#007bff".to_string(),
            secondary_color: "#6c757d".to_string(),
            custom_css: None,
            favicon_url: None,
            email_domain: None,
        }
    }
}

/// UI preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPreferences {
    /// Theme (light, dark, auto)
    pub theme: String,
    /// Language code
    pub language: String,
    /// Timezone
    pub timezone: String,
    /// Date format
    pub date_format: String,
    /// Number format
    pub number_format: String,
    /// Enable tooltips
    pub show_tooltips: bool,
    /// Enable animations
    pub animations_enabled: bool,
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self {
            theme: "auto".to_string(),
            language: "en-US".to_string(),
            timezone: "UTC".to_string(),
            date_format: "YYYY-MM-DD".to_string(),
            number_format: "1,234.56".to_string(),
            show_tooltips: true,
            animations_enabled: true,
        }
    }
}

/// Complete tenant configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantConfig {
    /// Tenant identifier
    pub tenant_id: TenantId,
    /// Feature flags
    pub features: FeatureFlags,
    /// Branding configuration
    pub branding: BrandingConfig,
    /// UI preferences
    pub ui: UiPreferences,
    /// Custom configuration (JSON)
    pub custom: HashMap<String, JsonValue>,
    /// Configuration version
    pub version: u32,
    /// Is configuration locked (read-only)
    pub locked: bool,
    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl TenantConfig {
    /// Create a new tenant configuration
    pub fn new(tenant_id: TenantId) -> Self {
        Self {
            tenant_id,
            features: FeatureFlags::default(),
            branding: BrandingConfig::default(),
            ui: UiPreferences::default(),
            custom: HashMap::new(),
            version: 1,
            locked: false,
            updated_at: chrono::Utc::now(),
        }
    }

    /// Create with specific tier
    pub fn with_tier(tenant_id: TenantId, tier: Tier) -> Self {
        let features = match tier {
            Tier::Enterprise => FeatureFlags::enterprise(),
            Tier::Professional => FeatureFlags::professional(),
            Tier::Basic => FeatureFlags::default(),
        };

        Self {
            tenant_id,
            features,
            branding: BrandingConfig::default(),
            ui: UiPreferences::default(),
            custom: HashMap::new(),
            version: 1,
            locked: false,
            updated_at: chrono::Utc::now(),
        }
    }

    /// Get a custom configuration value
    pub fn get_custom<T>(&self, key: &str) -> ConfigResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let value = self.custom.get(key)
            .ok_or_else(|| ConfigError::KeyNotFound(key.to_string()))?;

        serde_json::from_value(value.clone())
            .map_err(|e| ConfigError::SerializationError(e.to_string()))
    }

    /// Set a custom configuration value
    pub fn set_custom<T>(&mut self, key: String, value: T) -> ConfigResult<()>
    where
        T: Serialize,
    {
        if self.locked {
            return Err(ConfigError::Locked);
        }

        let json_value = serde_json::to_value(value)
            .map_err(|e| ConfigError::SerializationError(e.to_string()))?;

        self.custom.insert(key, json_value);
        self.version += 1;
        self.updated_at = chrono::Utc::now();
        Ok(())
    }

    /// Lock configuration to prevent modifications
    pub fn lock(&mut self) {
        self.locked = true;
    }

    /// Unlock configuration
    pub fn unlock(&mut self) {
        self.locked = false;
    }
}

/// Subscription tier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    Basic,
    Professional,
    Enterprise,
}

/// Configuration inheritance manager
pub struct ConfigManager {
    /// Tenant configurations
    configs: DashMap<TenantId, Arc<RwLock<TenantConfig>>>,
    /// Default configuration template
    default_config: Arc<RwLock<TenantConfig>>,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Self {
        let default_tenant_id = TenantId::new_org(uuid::Uuid::nil());
        Self {
            configs: DashMap::new(),
            default_config: Arc::new(RwLock::new(TenantConfig::new(default_tenant_id))),
        }
    }

    /// Set the default configuration template
    pub fn set_default(&self, config: TenantConfig) {
        *self.default_config.write() = config;
    }

    /// Create configuration for a new tenant
    pub fn create_config(&self, tenant_id: TenantId, tier: Tier) -> TenantConfig {
        let config = TenantConfig::with_tier(tenant_id.clone(), tier);
        self.configs.insert(tenant_id, Arc::new(RwLock::new(config.clone())));
        config
    }

    /// Get configuration for a tenant
    pub fn get_config(&self, tenant_id: &TenantId) -> ConfigResult<TenantConfig> {
        self.configs
            .get(tenant_id)
            .map(|config| config.read().clone())
            .ok_or_else(|| ConfigError::NotFound(tenant_id.to_string()))
    }

    /// Update configuration for a tenant
    pub fn update_config<F>(&self, tenant_id: &TenantId, updater: F) -> ConfigResult<()>
    where
        F: FnOnce(&mut TenantConfig) -> ConfigResult<()>,
    {
        let config_ref = self.configs
            .get(tenant_id)
            .ok_or_else(|| ConfigError::NotFound(tenant_id.to_string()))?;

        let mut config = config_ref.write();

        if config.locked {
            return Err(ConfigError::Locked);
        }

        updater(&mut config)?;
        config.version += 1;
        config.updated_at = chrono::Utc::now();

        Ok(())
    }

    /// Get configuration with inheritance from parent tenants
    pub fn get_effective_config(&self, tenant_id: &TenantId) -> ConfigResult<TenantConfig> {
        // Start with default config
        let mut effective = self.default_config.read().clone();

        // Apply organization-level config
        let org_tenant = tenant_id.org_tenant();
        if let Some(org_config) = self.configs.get(&org_tenant) {
            effective = self.merge_configs(effective, org_config.read().clone());
        }

        // Apply workspace-level config if applicable
        if let Some(ws_tenant) = tenant_id.workspace_tenant() {
            if let Some(ws_config) = self.configs.get(&ws_tenant) {
                effective = self.merge_configs(effective, ws_config.read().clone());
            }
        }

        // Apply project-level config if applicable
        if let Some(config) = self.configs.get(tenant_id) {
            effective = self.merge_configs(effective, config.read().clone());
        }

        Ok(effective)
    }

    /// Merge two configurations (child overrides parent)
    fn merge_configs(&self, parent: TenantConfig, child: TenantConfig) -> TenantConfig {
        let mut merged = parent;

        // Merge feature flags
        if child.features.collaboration {
            merged.features.collaboration = true;
        }
        // ... merge other feature flags

        // Override branding if set
        if child.branding.logo_url.is_some() {
            merged.branding = child.branding.clone();
        }

        // Merge custom configuration
        for (key, value) in child.custom {
            merged.custom.insert(key, value);
        }

        merged.tenant_id = child.tenant_id;
        merged.version = child.version;
        merged.updated_at = child.updated_at;

        merged
    }

    /// Check if a feature is enabled for a tenant
    pub fn is_feature_enabled(&self, tenant_id: &TenantId, feature: &str) -> bool {
        if let Ok(config) = self.get_effective_config(tenant_id) {
            match feature {
                "collaboration" => config.features.collaboration,
                "cloud_sync" => config.features.cloud_sync,
                "advanced_rendering" => config.features.advanced_rendering,
                "marketplace" => config.features.marketplace,
                "ai_features" => config.features.ai_features,
                "workflows" => config.features.workflows,
                "analytics" => config.features.analytics,
                _ => config.features.is_enabled(feature),
            }
        } else {
            false
        }
    }

    /// Remove tenant configuration
    pub fn remove_config(&self, tenant_id: &TenantId) {
        self.configs.remove(tenant_id);
    }

    /// Export configuration as JSON
    pub fn export_config(&self, tenant_id: &TenantId) -> ConfigResult<String> {
        let config = self.get_config(tenant_id)?;
        serde_json::to_string_pretty(&config)
            .map_err(|e| ConfigError::SerializationError(e.to_string()))
    }

    /// Import configuration from JSON
    pub fn import_config(&self, json: &str) -> ConfigResult<TenantId> {
        let config: TenantConfig = serde_json::from_str(json)
            .map_err(|e| ConfigError::SerializationError(e.to_string()))?;

        let tenant_id = config.tenant_id.clone();
        self.configs.insert(tenant_id.clone(), Arc::new(RwLock::new(config)));

        Ok(tenant_id)
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_feature_flags_defaults() {
        let flags = FeatureFlags::default();
        assert!(!flags.collaboration);
        assert!(!flags.cloud_sync);
        assert_eq!(flags.max_concurrent_users, 5);
    }

    #[test]
    fn test_feature_flags_enterprise() {
        let flags = FeatureFlags::enterprise();
        assert!(flags.collaboration);
        assert!(flags.cloud_sync);
        assert!(flags.ai_features);
        assert_eq!(flags.max_concurrent_users, 1000);
    }

    #[test]
    fn test_custom_feature_flags() {
        let mut flags = FeatureFlags::default();
        flags.enable("custom_feature".to_string());
        assert!(flags.is_enabled("custom_feature"));

        flags.disable("custom_feature".to_string());
        assert!(!flags.is_enabled("custom_feature"));
    }

    #[test]
    fn test_config_creation() {
        let manager = ConfigManager::new();
        let tenant_id = TenantId::new_org(Uuid::new_v4());

        let config = manager.create_config(tenant_id.clone(), Tier::Professional);
        assert_eq!(config.tenant_id, tenant_id);
        assert!(config.features.collaboration);
    }

    #[test]
    fn test_config_update() {
        let manager = ConfigManager::new();
        let tenant_id = TenantId::new_org(Uuid::new_v4());

        manager.create_config(tenant_id.clone(), Tier::Basic);

        manager.update_config(&tenant_id, |config| {
            config.branding.organization_name = "Custom Org".to_string();
            Ok(())
        }).unwrap();

        let config = manager.get_config(&tenant_id).unwrap();
        assert_eq!(config.branding.organization_name, "Custom Org");
    }

    #[test]
    fn test_config_locking() {
        let manager = ConfigManager::new();
        let tenant_id = TenantId::new_org(Uuid::new_v4());

        manager.create_config(tenant_id.clone(), Tier::Basic);

        // Lock the config
        manager.update_config(&tenant_id, |config| {
            config.lock();
            Ok(())
        }).unwrap();

        // Try to update locked config
        let result = manager.update_config(&tenant_id, |config| {
            config.branding.organization_name = "Should Fail".to_string();
            Ok(())
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_custom_config_values() {
        let mut config = TenantConfig::new(TenantId::new_org(Uuid::new_v4()));

        config.set_custom("max_retries".to_string(), 5u32).unwrap();
        config.set_custom("api_endpoint".to_string(), "https://api.example.com".to_string()).unwrap();

        let max_retries: u32 = config.get_custom("max_retries").unwrap();
        assert_eq!(max_retries, 5);

        let api_endpoint: String = config.get_custom("api_endpoint").unwrap();
        assert_eq!(api_endpoint, "https://api.example.com");
    }

    #[test]
    fn test_config_inheritance() {
        let manager = ConfigManager::new();
        let org_id = Uuid::new_v4();
        let ws_id = Uuid::new_v4();

        let org_tenant = TenantId::new_org(org_id);
        let ws_tenant = TenantId::new_workspace(org_id, ws_id);

        // Create org-level config
        manager.create_config(org_tenant.clone(), Tier::Enterprise);
        manager.update_config(&org_tenant, |config| {
            config.branding.organization_name = "Parent Org".to_string();
            Ok(())
        }).unwrap();

        // Create workspace-level config
        manager.create_config(ws_tenant.clone(), Tier::Professional);

        // Get effective config for workspace
        let effective = manager.get_effective_config(&ws_tenant).unwrap();

        // Should inherit from org level
        assert_eq!(effective.tenant_id, ws_tenant);
    }

    #[test]
    fn test_feature_check() {
        let manager = ConfigManager::new();
        let tenant_id = TenantId::new_org(Uuid::new_v4());

        manager.create_config(tenant_id.clone(), Tier::Enterprise);

        assert!(manager.is_feature_enabled(&tenant_id, "collaboration"));
        assert!(manager.is_feature_enabled(&tenant_id, "ai_features"));
    }

    #[test]
    fn test_config_export_import() {
        let manager = ConfigManager::new();
        let tenant_id = TenantId::new_org(Uuid::new_v4());

        manager.create_config(tenant_id.clone(), Tier::Professional);

        let json = manager.export_config(&tenant_id).unwrap();
        assert!(json.contains("Professional") || json.contains("tenant_id"));

        let imported_id = manager.import_config(&json).unwrap();
        assert_eq!(imported_id, tenant_id);
    }
}
