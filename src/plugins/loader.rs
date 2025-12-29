//! Dynamic plugin loader with hot-reload support
//!
//! Handles loading, unloading, and hot-reloading of plugins at runtime.
//! Supports both WASM and native plugins with proper isolation.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::mpsc;

use super::api::{ApiVersion, PluginApi};
use super::lifecycle::{PluginLifecycle, PluginState};
use super::permissions::PermissionSet;
use super::sandbox::PluginSandbox;

/// Plugin loader errors
#[derive(Debug, Error)]
pub enum LoaderError {
    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Failed to load plugin: {0}")]
    LoadFailed(String),

    #[error("Invalid plugin: {0}")]
    Invalid(String),

    #[error("Plugin already loaded: {0}")]
    AlreadyLoaded(String),

    #[error("Version mismatch: {0}")]
    VersionMismatch(String),

    #[error("Dependency error: {0}")]
    DependencyError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type LoaderResult<T> = Result<T, LoaderError>;

/// Plugin manifest containing metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Unique plugin identifier
    pub id: String,

    /// Plugin name
    pub name: String,

    /// Plugin version
    pub version: String,

    /// Plugin description
    pub description: String,

    /// Plugin author
    pub author: String,

    /// Required API version
    pub api_version: String,

    /// Plugin entry point (for WASM: .wasm file, for native: .so/.dll/.dylib)
    pub entry_point: String,

    /// Plugin type
    pub plugin_type: PluginType,

    /// Required permissions
    pub permissions: Vec<String>,

    /// Plugin dependencies
    #[serde(default)]
    pub dependencies: Vec<PluginDependency>,

    /// Plugin capabilities
    #[serde(default)]
    pub capabilities: Vec<String>,

    /// Plugin icon (base64 encoded or path)
    #[serde(default)]
    pub icon: Option<String>,

    /// Plugin website
    #[serde(default)]
    pub website: Option<String>,

    /// Plugin repository
    #[serde(default)]
    pub repository: Option<String>,

    /// Plugin license
    #[serde(default)]
    pub license: Option<String>,

    /// Minimum CADDY version
    #[serde(default)]
    pub min_caddy_version: Option<String>,

    /// Maximum CADDY version (if specified)
    #[serde(default)]
    pub max_caddy_version: Option<String>,
}

/// Plugin dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub id: String,
    pub version_requirement: String,
    #[serde(default)]
    pub optional: bool,
}

/// Plugin type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginType {
    /// WebAssembly plugin (sandboxed)
    Wasm,
    /// Native plugin (less secure, higher performance)
    Native,
}

/// Loaded plugin instance
#[derive(Clone)]
pub struct LoadedPlugin {
    pub manifest: PluginManifest,
    pub path: PathBuf,
    pub api: Arc<PluginApi>,
    pub sandbox: Arc<PluginSandbox>,
    pub lifecycle: Arc<RwLock<PluginLifecycle>>,
    pub loaded_at: chrono::DateTime<chrono::Utc>,
}

impl LoadedPlugin {
    /// Get plugin ID
    pub fn id(&self) -> &str {
        &self.manifest.id
    }

    /// Get plugin state
    pub fn state(&self) -> PluginState {
        self.lifecycle.read().state()
    }

    /// Check if plugin is active
    pub fn is_active(&self) -> bool {
        matches!(self.state(), PluginState::Running)
    }
}

/// Plugin loader events
#[derive(Debug, Clone)]
pub enum LoaderEvent {
    PluginLoaded { id: String, version: String },
    PluginUnloaded { id: String },
    PluginReloaded { id: String, old_version: String, new_version: String },
    PluginError { id: String, error: String },
    PluginStateChanged { id: String, state: PluginState },
}

/// Plugin loader with hot-reload support
pub struct PluginLoader {
    /// Base directory for plugins
    plugins_dir: PathBuf,

    /// Loaded plugins
    plugins: Arc<RwLock<HashMap<String, LoadedPlugin>>>,

    /// Event channel
    event_tx: mpsc::UnboundedSender<LoaderEvent>,
    event_rx: Arc<RwLock<Option<mpsc::UnboundedReceiver<LoaderEvent>>>>,

    /// Watch for file changes
    #[allow(dead_code)]
    hot_reload_enabled: bool,
}

impl PluginLoader {
    /// Create a new plugin loader
    pub fn new<P: AsRef<Path>>(plugins_dir: P) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        Self {
            plugins_dir: plugins_dir.as_ref().to_path_buf(),
            plugins: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            event_rx: Arc::new(RwLock::new(Some(event_rx))),
            hot_reload_enabled: false,
        }
    }

    /// Enable hot-reload (file watching)
    pub fn enable_hot_reload(&mut self) {
        self.hot_reload_enabled = true;
        // In a real implementation, we would set up file watching here
    }

    /// Get event receiver
    pub fn take_event_receiver(&self) -> Option<mpsc::UnboundedReceiver<LoaderEvent>> {
        self.event_rx.write().take()
    }

    /// Load a plugin from a directory
    pub async fn load_plugin<P: AsRef<Path>>(&self, plugin_path: P) -> LoaderResult<String> {
        let plugin_path = plugin_path.as_ref();

        // Read manifest
        let manifest_path = plugin_path.join("plugin.json");
        if !manifest_path.exists() {
            return Err(LoaderError::Invalid("Missing plugin.json manifest".to_string()));
        }

        let manifest_data = tokio::fs::read_to_string(&manifest_path).await?;
        let manifest: PluginManifest = serde_json::from_str(&manifest_data)?;

        // Check if already loaded
        if self.plugins.read().contains_key(&manifest.id) {
            return Err(LoaderError::AlreadyLoaded(manifest.id.clone()));
        }

        // Validate API version
        let required_api_version = ApiVersion::parse(&manifest.api_version)
            .map_err(|e| LoaderError::VersionMismatch(format!("Invalid API version: {}", e)))?;

        if !ApiVersion::CURRENT.is_compatible_with(&required_api_version) {
            return Err(LoaderError::VersionMismatch(format!(
                "Plugin requires API version {}, but current is {}",
                manifest.api_version,
                ApiVersion::CURRENT
            )));
        }

        // Check dependencies
        self.check_dependencies(&manifest.dependencies)?;

        // Parse permissions
        let permissions = self.parse_permissions(&manifest.permissions)?;

        // Create plugin API
        let api = Arc::new(PluginApi::new(permissions));

        // Create sandbox
        let sandbox = Arc::new(PluginSandbox::new(
            manifest.id.clone(),
            manifest.plugin_type,
            plugin_path.to_path_buf(),
        ));

        // Initialize lifecycle
        let lifecycle = Arc::new(RwLock::new(PluginLifecycle::new(manifest.id.clone())));

        // Load plugin binary
        let entry_point = plugin_path.join(&manifest.entry_point);
        if !entry_point.exists() {
            return Err(LoaderError::Invalid(format!(
                "Entry point not found: {}",
                manifest.entry_point
            )));
        }

        // Initialize plugin in sandbox
        sandbox.initialize(&entry_point).await
            .map_err(|e| LoaderError::LoadFailed(e.to_string()))?;

        let loaded_plugin = LoadedPlugin {
            manifest: manifest.clone(),
            path: plugin_path.to_path_buf(),
            api,
            sandbox,
            lifecycle,
            loaded_at: chrono::Utc::now(),
        };

        // Store loaded plugin
        let plugin_id = manifest.id.clone();
        self.plugins.write().insert(plugin_id.clone(), loaded_plugin);

        // Send event
        let _ = self.event_tx.send(LoaderEvent::PluginLoaded {
            id: plugin_id.clone(),
            version: manifest.version.clone(),
        });

        log::info!("Loaded plugin: {} v{}", manifest.name, manifest.version);

        Ok(plugin_id)
    }

    /// Unload a plugin
    pub async fn unload_plugin(&self, plugin_id: &str) -> LoaderResult<()> {
        let plugin = self.plugins.write().remove(plugin_id)
            .ok_or_else(|| LoaderError::NotFound(plugin_id.to_string()))?;

        // Stop plugin
        plugin.lifecycle.write().stop();

        // Cleanup sandbox
        plugin.sandbox.cleanup().await
            .map_err(|e| LoaderError::LoadFailed(format!("Cleanup failed: {}", e)))?;

        // Send event
        let _ = self.event_tx.send(LoaderEvent::PluginUnloaded {
            id: plugin_id.to_string(),
        });

        log::info!("Unloaded plugin: {}", plugin_id);

        Ok(())
    }

    /// Reload a plugin (hot-reload)
    pub async fn reload_plugin(&self, plugin_id: &str) -> LoaderResult<()> {
        let plugin = self.plugins.read().get(plugin_id)
            .ok_or_else(|| LoaderError::NotFound(plugin_id.to_string()))?
            .clone();

        let old_version = plugin.manifest.version.clone();
        let plugin_path = plugin.path.clone();

        // Unload
        self.unload_plugin(plugin_id).await?;

        // Reload
        let new_plugin_id = self.load_plugin(&plugin_path).await?;

        let new_version = self.plugins.read()
            .get(&new_plugin_id)
            .map(|p| p.manifest.version.clone())
            .unwrap_or_default();

        // Send event
        let _ = self.event_tx.send(LoaderEvent::PluginReloaded {
            id: plugin_id.to_string(),
            old_version,
            new_version,
        });

        log::info!("Reloaded plugin: {}", plugin_id);

        Ok(())
    }

    /// Get a loaded plugin
    pub fn get_plugin(&self, plugin_id: &str) -> Option<LoadedPlugin> {
        self.plugins.read().get(plugin_id).cloned()
    }

    /// List all loaded plugins
    pub fn list_plugins(&self) -> Vec<LoadedPlugin> {
        self.plugins.read().values().cloned().collect()
    }

    /// Scan plugins directory and discover all plugins
    pub async fn discover_plugins(&self) -> LoaderResult<Vec<PluginManifest>> {
        let mut discovered = Vec::new();

        if !self.plugins_dir.exists() {
            tokio::fs::create_dir_all(&self.plugins_dir).await?;
            return Ok(discovered);
        }

        let mut entries = tokio::fs::read_dir(&self.plugins_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join("plugin.json");
            if !manifest_path.exists() {
                continue;
            }

            match tokio::fs::read_to_string(&manifest_path).await {
                Ok(data) => {
                    match serde_json::from_str::<PluginManifest>(&data) {
                        Ok(manifest) => discovered.push(manifest),
                        Err(e) => log::warn!("Invalid manifest in {:?}: {}", path, e),
                    }
                }
                Err(e) => log::warn!("Failed to read manifest {:?}: {}", manifest_path, e),
            }
        }

        Ok(discovered)
    }

    /// Load all discovered plugins
    pub async fn load_all_plugins(&self) -> Vec<(String, LoaderResult<String>)> {
        let mut results = Vec::new();

        match self.discover_plugins().await {
            Ok(manifests) => {
                for manifest in manifests {
                    let plugin_path = self.plugins_dir.join(&manifest.id);
                    let result = self.load_plugin(&plugin_path).await;
                    results.push((manifest.id.clone(), result));
                }
            }
            Err(e) => {
                log::error!("Failed to discover plugins: {}", e);
            }
        }

        results
    }

    /// Check plugin dependencies
    fn check_dependencies(&self, dependencies: &[PluginDependency]) -> LoaderResult<()> {
        let plugins = self.plugins.read();

        for dep in dependencies {
            if dep.optional {
                continue;
            }

            let loaded = plugins.get(&dep.id)
                .ok_or_else(|| LoaderError::DependencyError(format!(
                    "Required dependency not loaded: {}",
                    dep.id
                )))?;

            // Simple version check (in production, use semver crate)
            if loaded.manifest.version != dep.version_requirement && !dep.version_requirement.starts_with('^') {
                return Err(LoaderError::DependencyError(format!(
                    "Dependency version mismatch: {} requires {}, but {} is loaded",
                    dep.id, dep.version_requirement, loaded.manifest.version
                )));
            }
        }

        Ok(())
    }

    /// Parse permission strings into PermissionSet
    fn parse_permissions(&self, permission_strings: &[String]) -> LoaderResult<PermissionSet> {
        use super::permissions::Permission;

        let mut permissions = PermissionSet::new();

        for perm_str in permission_strings {
            let permission = match perm_str.as_str() {
                "geometry:read" => Permission::GeometryRead,
                "geometry:write" => Permission::GeometryWrite,
                "geometry:delete" => Permission::GeometryDelete,
                "rendering:read" => Permission::RenderingRead,
                "rendering:write" => Permission::RenderingWrite,
                "ui:read" => Permission::UIRead,
                "ui:write" => Permission::UIWrite,
                "ui:menu" => Permission::UIMenuAccess,
                "ui:toolbar" => Permission::UIToolbarAccess,
                "file:read" => Permission::FileRead,
                "file:write" => Permission::FileWrite,
                "command:execute" => Permission::CommandExecute,
                "command:register" => Permission::CommandRegister,
                "network:http" => Permission::NetworkHTTP,
                "network:websocket" => Permission::NetworkWebSocket,
                "database:read" => Permission::DatabaseRead,
                "database:write" => Permission::DatabaseWrite,
                "system:notifications" => Permission::SystemNotifications,
                "system:clipboard" => Permission::SystemClipboard,
                _ => {
                    log::warn!("Unknown permission: {}", perm_str);
                    continue;
                }
            };

            permissions.grant(permission);
        }

        Ok(permissions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin_manifest_parse() {
        let json = r#"{
            "id": "test-plugin",
            "name": "Test Plugin",
            "version": "1.0.0",
            "description": "A test plugin",
            "author": "Test Author",
            "api_version": "0.2.5",
            "entry_point": "plugin.wasm",
            "plugin_type": "Wasm",
            "permissions": ["geometry:read", "ui:write"]
        }"#;

        let manifest: PluginManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.id, "test-plugin");
        assert_eq!(manifest.permissions.len(), 2);
    }

    #[test]
    fn test_plugin_type_serialization() {
        let wasm = PluginType::Wasm;
        let json = serde_json::to_string(&wasm).unwrap();
        assert_eq!(json, r#""Wasm""#);
    }
}
