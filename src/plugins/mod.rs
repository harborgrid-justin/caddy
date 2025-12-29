//! Enterprise Plugin System for CADDY v0.2.5
//!
//! A comprehensive plugin architecture providing:
//! - Dynamic plugin loading with hot-reload support
//! - WASM-based sandboxing for security
//! - Fine-grained permission system
//! - Plugin marketplace integration
//! - Dependency resolution and versioning
//! - Lifecycle management
//! - Resource limits and monitoring

pub mod api;
pub mod lifecycle;
pub mod loader;
pub mod marketplace;
pub mod permissions;
pub mod registry;
pub mod sandbox;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;
use thiserror::Error;

pub use api::{ApiVersion, PluginApi};
pub use lifecycle::{PluginLifecycle, PluginState};
pub use loader::{LoadedPlugin, PluginLoader, PluginManifest, PluginType};
pub use marketplace::{PluginMarketplace, MarketplacePlugin, SearchFilters};
pub use permissions::{Permission, PermissionSet};
pub use registry::{PluginRegistry, PluginRegistration};
pub use sandbox::{PluginSandbox, ResourceLimits};

/// Plugin system errors
#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Loader error: {0}")]
    Loader(#[from] loader::LoaderError),

    #[error("Registry error: {0}")]
    Registry(#[from] registry::RegistryError),

    #[error("Marketplace error: {0}")]
    Marketplace(#[from] marketplace::MarketplaceError),

    #[error("Sandbox error: {0}")]
    Sandbox(#[from] sandbox::SandboxError),

    #[error("API error: {0}")]
    Api(#[from] api::ApiError),

    #[error("Lifecycle error: {0}")]
    Lifecycle(#[from] lifecycle::LifecycleError),

    #[error("Permission error: {0}")]
    Permission(#[from] permissions::PermissionError),

    #[error("Plugin system not initialized")]
    NotInitialized,

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type PluginResult<T> = Result<T, PluginError>;

/// Plugin system configuration
#[derive(Debug, Clone)]
pub struct PluginConfig {
    /// Directory where plugins are installed
    pub plugins_dir: PathBuf,

    /// Directory for plugin data/cache
    pub data_dir: PathBuf,

    /// Registry file path
    pub registry_path: PathBuf,

    /// Marketplace base URL
    pub marketplace_url: String,

    /// Enable hot-reload
    pub hot_reload: bool,

    /// Default resource limits for plugins
    pub default_limits: ResourceLimits,

    /// Auto-load enabled plugins on startup
    pub auto_load: bool,

    /// Maximum number of plugins that can be loaded
    pub max_plugins: usize,
}

impl Default for PluginConfig {
    fn default() -> Self {
        let base_dir = std::env::var("CADDY_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs::data_dir().unwrap_or_default().join("caddy"));

        Self {
            plugins_dir: base_dir.join("plugins"),
            data_dir: base_dir.join("plugin-data"),
            registry_path: base_dir.join("plugin-registry.json"),
            marketplace_url: "https://marketplace.caddy.dev".to_string(),
            hot_reload: false,
            default_limits: ResourceLimits::default(),
            auto_load: true,
            max_plugins: 100,
        }
    }
}

/// Main plugin manager - coordinates all plugin system components
pub struct PluginManager {
    config: PluginConfig,
    loader: Arc<PluginLoader>,
    registry: Arc<RwLock<PluginRegistry>>,
    marketplace: Arc<RwLock<PluginMarketplace>>,
    initialized: bool,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new(config: PluginConfig) -> Self {
        let loader = Arc::new(PluginLoader::new(&config.plugins_dir));
        let registry = Arc::new(RwLock::new(PluginRegistry::new(&config.registry_path)));
        let marketplace = Arc::new(RwLock::new(PluginMarketplace::new(
            config.marketplace_url.clone(),
            &config.plugins_dir,
        )));

        Self {
            config,
            loader,
            registry,
            marketplace,
            initialized: false,
        }
    }

    /// Create with default configuration
    pub fn with_default_config() -> Self {
        Self::new(PluginConfig::default())
    }

    /// Initialize the plugin system
    pub async fn initialize(&mut self) -> PluginResult<()> {
        if self.initialized {
            return Ok(());
        }

        // Create directories
        tokio::fs::create_dir_all(&self.config.plugins_dir).await?;
        tokio::fs::create_dir_all(&self.config.data_dir).await?;

        // Load registry
        self.registry.write().load().await?;

        // Note: Marketplace registry sharing disabled due to ownership constraints
        // The marketplace can query the registry through other means if needed

        // Auto-load enabled plugins
        if self.config.auto_load {
            self.load_enabled_plugins().await?;
        }

        self.initialized = true;

        log::info!("Plugin system initialized");

        Ok(())
    }

    /// Load all enabled plugins from registry
    async fn load_enabled_plugins(&self) -> PluginResult<()> {
        let enabled: Vec<_> = self.registry.read().list_enabled().to_vec();

        for registration in &enabled {
            match self.loader.load_plugin(&registration.install_path).await {
                Ok(plugin_id) => {
                    log::info!("Auto-loaded plugin: {}", plugin_id);
                }
                Err(e) => {
                    log::error!(
                        "Failed to auto-load plugin {}: {}",
                        registration.manifest.id,
                        e
                    );
                }
            }
        }

        Ok(())
    }

    /// Install a plugin from marketplace
    pub async fn install_from_marketplace(&self, plugin_id: &str) -> PluginResult<()> {
        self.ensure_initialized()?;

        let plugin_path = self.marketplace.write().install(plugin_id).await?;

        // Load the installed plugin
        self.loader.load_plugin(&plugin_path).await?;

        Ok(())
    }

    /// Install a plugin from local path
    pub async fn install_from_path<P: AsRef<Path>>(&self, path: P) -> PluginResult<()> {
        self.ensure_initialized()?;

        // Load manifest
        let manifest_path = path.as_ref().join("plugin.json");
        let manifest_data = tokio::fs::read_to_string(&manifest_path).await?;
        let manifest: PluginManifest = serde_json::from_str(&manifest_data)?;

        // Register plugin
        self.registry
            .write()
            .register(
                manifest,
                path.as_ref().to_path_buf(),
                registry::InstallationSource::Local,
            )
            .await?;

        // Load plugin
        self.loader.load_plugin(path).await?;

        Ok(())
    }

    /// Uninstall a plugin
    pub async fn uninstall(&self, plugin_id: &str) -> PluginResult<()> {
        self.ensure_initialized()?;

        // Unload if loaded
        if self.loader.get_plugin(plugin_id).is_some() {
            self.loader.unload_plugin(plugin_id).await?;
        }

        // Unregister
        self.registry.write().unregister(plugin_id).await?;

        // Remove plugin files
        if let Some(registration) = self.registry.read().get(plugin_id) {
            if registration.install_path.exists() {
                tokio::fs::remove_dir_all(&registration.install_path).await?;
            }
        }

        Ok(())
    }

    /// Get plugin loader
    pub fn loader(&self) -> &PluginLoader {
        &self.loader
    }

    /// Get plugin registry
    pub fn registry(&self) -> &RwLock<PluginRegistry> {
        &self.registry
    }

    /// Get plugin marketplace
    pub fn marketplace(&self) -> &RwLock<PluginMarketplace> {
        &self.marketplace
    }

    /// Get loaded plugin
    pub fn get_plugin(&self, plugin_id: &str) -> Option<LoadedPlugin> {
        self.loader.get_plugin(plugin_id)
    }

    /// List all loaded plugins
    pub fn list_loaded_plugins(&self) -> Vec<LoadedPlugin> {
        self.loader.list_plugins()
    }

    /// Search marketplace
    pub async fn search_marketplace(&self, filters: SearchFilters) -> PluginResult<marketplace::SearchResults> {
        self.ensure_initialized()?;
        Ok(self.marketplace.read().search(filters).await?)
    }

    /// Check for plugin updates
    pub async fn check_updates(&self) -> PluginResult<Vec<marketplace::PluginUpdate>> {
        self.ensure_initialized()?;
        Ok(self.marketplace.read().check_updates().await?)
    }

    /// Get system statistics
    pub fn stats(&self) -> PluginSystemStats {
        let loaded_plugins = self.loader.list_plugins();
        let registry_stats = self.registry.read().stats();

        PluginSystemStats {
            loaded_plugins: loaded_plugins.len(),
            registered_plugins: registry_stats.total_plugins,
            enabled_plugins: registry_stats.enabled_plugins,
            running_plugins: loaded_plugins.iter().filter(|p| p.is_active()).count(),
            total_downloads: registry_stats.total_downloads,
        }
    }

    /// Ensure plugin system is initialized
    fn ensure_initialized(&self) -> PluginResult<()> {
        if !self.initialized {
            Err(PluginError::NotInitialized)
        } else {
            Ok(())
        }
    }

    /// Shutdown the plugin system
    pub async fn shutdown(&mut self) -> PluginResult<()> {
        if !self.initialized {
            return Ok(());
        }

        log::info!("Shutting down plugin system...");

        // Unload all plugins
        let loaded = self.loader.list_plugins();
        for plugin in loaded {
            if let Err(e) = self.loader.unload_plugin(&plugin.id()).await {
                log::error!("Error unloading plugin {}: {}", plugin.id(), e);
            }
        }

        // Save registry
        if let Err(e) = self.registry.write().save().await {
            log::error!("Error saving registry: {}", e);
        }

        self.initialized = false;

        log::info!("Plugin system shut down");

        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::with_default_config()
    }
}

/// Plugin system statistics
#[derive(Debug, Clone)]
pub struct PluginSystemStats {
    pub loaded_plugins: usize,
    pub registered_plugins: usize,
    pub enabled_plugins: usize,
    pub running_plugins: usize,
    pub total_downloads: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_config_default() {
        let config = PluginConfig::default();
        assert!(config.plugins_dir.ends_with("plugins"));
        assert_eq!(config.max_plugins, 100);
        assert!(config.auto_load);
    }

    #[tokio::test]
    async fn test_plugin_manager_creation() {
        let manager = PluginManager::with_default_config();
        assert!(!manager.initialized);
    }

    #[test]
    fn test_api_version() {
        let version = ApiVersion::CURRENT;
        assert_eq!(version.major, 0);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 5);
    }
}
