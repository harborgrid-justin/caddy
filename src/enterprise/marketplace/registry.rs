//! Plugin registry management
//!
//! This module provides local and remote plugin registry functionality,
//! including synchronization and plugin discovery.

use super::{MarketplaceError, PluginManifest, PluginMetadata, PluginStatus, Result};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use uuid::Uuid;

/// Local plugin registry
#[derive(Debug, Clone)]
pub struct LocalRegistry {
    /// Registry storage path
    storage_path: PathBuf,

    /// Installed plugins (plugin_id -> metadata)
    installed: Arc<RwLock<HashMap<Uuid, PluginMetadata>>>,

    /// Plugin index by name (name -> plugin_id)
    name_index: Arc<RwLock<HashMap<String, Uuid>>>,

    /// Last sync timestamp
    last_sync: Arc<RwLock<Option<DateTime<Utc>>>>,
}

impl LocalRegistry {
    /// Create a new local registry
    pub fn new<P: AsRef<Path>>(storage_path: P) -> Result<Self> {
        let storage_path = storage_path.as_ref().to_path_buf();

        // Ensure storage directory exists
        if !storage_path.exists() {
            std::fs::create_dir_all(&storage_path)?;
        }

        let registry = Self {
            storage_path,
            installed: Arc::new(RwLock::new(HashMap::new())),
            name_index: Arc::new(RwLock::new(HashMap::new())),
            last_sync: Arc::new(RwLock::new(None)),
        };

        // Load existing registry
        registry.load()?;

        Ok(registry)
    }

    /// Load registry from disk
    pub fn load(&self) -> Result<()> {
        let registry_file = self.storage_path.join("registry.json");

        if !registry_file.exists() {
            return Ok(());
        }

        let contents = std::fs::read_to_string(&registry_file)?;
        let registry_data: RegistryData = serde_json::from_str(&contents)
            .map_err(|e| MarketplaceError::SerializationError(e.to_string()))?;

        let mut installed = self.installed.write();
        let mut name_index = self.name_index.write();

        for metadata in registry_data.plugins {
            let plugin_id = metadata.manifest.id;
            let plugin_name = metadata.manifest.name.clone();

            name_index.insert(plugin_name, plugin_id);
            installed.insert(plugin_id, metadata);
        }

        *self.last_sync.write() = Some(registry_data.last_updated);

        Ok(())
    }

    /// Save registry to disk
    pub fn save(&self) -> Result<()> {
        let registry_file = self.storage_path.join("registry.json");

        let installed = self.installed.read();
        let plugins: Vec<PluginMetadata> = installed.values().cloned().collect();

        let registry_data = RegistryData {
            version: "1.0".to_string(),
            last_updated: Utc::now(),
            plugins,
        };

        let contents = serde_json::to_string_pretty(&registry_data)
            .map_err(|e| MarketplaceError::SerializationError(e.to_string()))?;

        std::fs::write(&registry_file, contents)?;

        Ok(())
    }

    /// Register a plugin
    pub fn register(&self, metadata: PluginMetadata) -> Result<()> {
        let plugin_id = metadata.manifest.id;
        let plugin_name = metadata.manifest.name.clone();

        // Validate manifest
        metadata.manifest.validate()
            .map_err(|e| MarketplaceError::InvalidManifest(e))?;

        // Check for name conflicts
        let name_index = self.name_index.read();
        if let Some(existing_id) = name_index.get(&plugin_name) {
            if *existing_id != plugin_id {
                return Err(MarketplaceError::RegistryError(
                    format!("Plugin name '{}' already registered", plugin_name)
                ));
            }
        }
        drop(name_index);

        // Register plugin
        self.name_index.write().insert(plugin_name, plugin_id);
        self.installed.write().insert(plugin_id, metadata);

        // Save to disk
        self.save()?;

        Ok(())
    }

    /// Unregister a plugin
    pub fn unregister(&self, plugin_id: Uuid) -> Result<()> {
        let mut installed = self.installed.write();

        if let Some(metadata) = installed.remove(&plugin_id) {
            let plugin_name = metadata.manifest.name;
            self.name_index.write().remove(&plugin_name);
            drop(installed);
            self.save()?;
            Ok(())
        } else {
            Err(MarketplaceError::PluginNotFound(plugin_id.to_string()))
        }
    }

    /// Get plugin by ID
    pub fn get(&self, plugin_id: Uuid) -> Option<PluginMetadata> {
        self.installed.read().get(&plugin_id).cloned()
    }

    /// Get plugin by name
    pub fn get_by_name(&self, name: &str) -> Option<PluginMetadata> {
        let name_index = self.name_index.read();
        let plugin_id = name_index.get(name)?;
        self.installed.read().get(plugin_id).cloned()
    }

    /// List all installed plugins
    pub fn list(&self) -> Vec<PluginMetadata> {
        self.installed.read().values().cloned().collect()
    }

    /// Search plugins by keyword
    pub fn search(&self, query: &str) -> Vec<PluginMetadata> {
        let query_lower = query.to_lowercase();

        self.installed.read()
            .values()
            .filter(|metadata| {
                metadata.manifest.name.to_lowercase().contains(&query_lower)
                    || metadata.manifest.description.to_lowercase().contains(&query_lower)
                    || metadata.manifest.keywords.iter().any(|k| k.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect()
    }

    /// Get last sync timestamp
    pub fn last_sync(&self) -> Option<DateTime<Utc>> {
        *self.last_sync.read()
    }

    /// Update last sync timestamp
    pub fn update_sync_time(&self) {
        *self.last_sync.write() = Some(Utc::now());
    }
}

/// Remote registry client
#[derive(Debug, Clone)]
pub struct RegistryClient {
    /// Registry URL
    registry_url: String,

    /// API key for authentication
    api_key: Option<String>,

    /// HTTP client
    client: Arc<reqwest::Client>,
}

impl RegistryClient {
    /// Create a new registry client
    pub fn new(registry_url: String) -> Self {
        Self {
            registry_url,
            api_key: None,
            client: Arc::new(reqwest::Client::new()),
        }
    }

    /// Set API key
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// Fetch plugin metadata from remote registry
    pub async fn fetch_plugin(&self, plugin_id: Uuid) -> Result<PluginMetadata> {
        let url = format!("{}/plugins/{}", self.registry_url, plugin_id);

        let mut request = self.client.get(&url);

        if let Some(api_key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await
            .map_err(|e| MarketplaceError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(MarketplaceError::NetworkError(
                format!("HTTP {}: {}", response.status(), response.status().canonical_reason().unwrap_or("Unknown"))
            ));
        }

        let metadata = response.json::<PluginMetadata>().await
            .map_err(|e| MarketplaceError::SerializationError(e.to_string()))?;

        Ok(metadata)
    }

    /// Search plugins in remote registry
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<PluginMetadata>> {
        let url = format!("{}/search?q={}&limit={}", self.registry_url, query, limit);

        let mut request = self.client.get(&url);

        if let Some(api_key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await
            .map_err(|e| MarketplaceError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(MarketplaceError::NetworkError(
                format!("HTTP {}", response.status())
            ));
        }

        let results = response.json::<Vec<PluginMetadata>>().await
            .map_err(|e| MarketplaceError::SerializationError(e.to_string()))?;

        Ok(results)
    }

    /// List all available plugins
    pub async fn list_all(&self) -> Result<Vec<PluginMetadata>> {
        let url = format!("{}/plugins", self.registry_url);

        let mut request = self.client.get(&url);

        if let Some(api_key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await
            .map_err(|e| MarketplaceError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(MarketplaceError::NetworkError(
                format!("HTTP {}", response.status())
            ));
        }

        let plugins = response.json::<Vec<PluginMetadata>>().await
            .map_err(|e| MarketplaceError::SerializationError(e.to_string()))?;

        Ok(plugins.into_iter()
            .filter(|p| p.status == PluginStatus::Published)
            .collect())
    }

    /// Publish a plugin to the remote registry
    pub async fn publish(&self, manifest: &PluginManifest) -> Result<()> {
        let url = format!("{}/plugins", self.registry_url);

        let mut request = self.client.post(&url)
            .json(manifest);

        if let Some(api_key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        } else {
            return Err(MarketplaceError::PermissionDenied(
                "API key required for publishing".to_string()
            ));
        }

        let response = request.send().await
            .map_err(|e| MarketplaceError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(MarketplaceError::NetworkError(
                format!("HTTP {}", response.status())
            ));
        }

        Ok(())
    }

    /// Download plugin package
    pub async fn download(&self, plugin_id: Uuid) -> Result<Vec<u8>> {
        let url = format!("{}/plugins/{}/download", self.registry_url, plugin_id);

        let mut request = self.client.get(&url);

        if let Some(api_key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await
            .map_err(|e| MarketplaceError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(MarketplaceError::NetworkError(
                format!("HTTP {}", response.status())
            ));
        }

        let bytes = response.bytes().await
            .map_err(|e| MarketplaceError::NetworkError(e.to_string()))?;

        Ok(bytes.to_vec())
    }
}

/// Registry synchronization
#[derive(Debug)]
pub struct RegistrySync {
    /// Local registry
    local: LocalRegistry,

    /// Remote registry client
    remote: RegistryClient,
}

impl RegistrySync {
    /// Create a new registry sync
    pub fn new(local: LocalRegistry, remote: RegistryClient) -> Self {
        Self { local, remote }
    }

    /// Sync with remote registry
    pub async fn sync(&self) -> Result<SyncResult> {
        let mut result = SyncResult::default();

        // Fetch all plugins from remote
        let remote_plugins = self.remote.list_all().await?;

        for remote_metadata in remote_plugins {
            let plugin_id = remote_metadata.manifest.id;

            // Check if plugin exists locally
            if let Some(local_metadata) = self.local.get(plugin_id) {
                // Compare versions
                if remote_metadata.manifest.version != local_metadata.manifest.version {
                    result.updates_available.push(remote_metadata);
                }
            } else {
                // New plugin available
                result.new_plugins.push(remote_metadata);
            }
        }

        // Update sync timestamp
        self.local.update_sync_time();
        self.local.save()?;

        Ok(result)
    }

    /// Sync specific plugin
    pub async fn sync_plugin(&self, plugin_id: Uuid) -> Result<()> {
        let remote_metadata = self.remote.fetch_plugin(plugin_id).await?;
        self.local.register(remote_metadata)?;
        Ok(())
    }
}

/// Sync result
#[derive(Debug, Default)]
pub struct SyncResult {
    /// New plugins available
    pub new_plugins: Vec<PluginMetadata>,

    /// Updates available for installed plugins
    pub updates_available: Vec<PluginMetadata>,
}

/// Registry data for serialization
#[derive(Debug, Serialize, Deserialize)]
struct RegistryData {
    /// Registry format version
    version: String,

    /// Last update timestamp
    last_updated: DateTime<Utc>,

    /// Plugin list
    plugins: Vec<PluginMetadata>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enterprise::marketplace::plugin::PluginManifest;

    #[test]
    fn test_local_registry() -> Result<()> {
        let temp_dir = std::env::temp_dir().join("caddy_test_registry");
        let registry = LocalRegistry::new(&temp_dir)?;

        let manifest = PluginManifest::new(
            "test-plugin".to_string(),
            "1.0.0".to_string(),
            "Test Author".to_string(),
        );
        let metadata = PluginMetadata::from_manifest(manifest);

        registry.register(metadata.clone())?;

        let retrieved = registry.get(metadata.manifest.id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().manifest.name, "test-plugin");

        std::fs::remove_dir_all(temp_dir)?;
        Ok(())
    }

    #[test]
    fn test_registry_search() -> Result<()> {
        let temp_dir = std::env::temp_dir().join("caddy_test_search");
        let registry = LocalRegistry::new(&temp_dir)?;

        let manifest1 = PluginManifest::new(
            "awesome-plugin".to_string(),
            "1.0.0".to_string(),
            "Author".to_string(),
        );
        let manifest2 = PluginManifest::new(
            "another-tool".to_string(),
            "1.0.0".to_string(),
            "Author".to_string(),
        );

        registry.register(PluginMetadata::from_manifest(manifest1))?;
        registry.register(PluginMetadata::from_manifest(manifest2))?;

        let results = registry.search("awesome");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].manifest.name, "awesome-plugin");

        std::fs::remove_dir_all(temp_dir)?;
        Ok(())
    }
}
