//! Plugin registry and discovery system
//!
//! Maintains a registry of available and installed plugins, handles
//! plugin discovery, versioning, and dependency resolution.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use thiserror::Error;

use super::loader::PluginManifest;

/// Registry errors
#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Plugin already registered: {0}")]
    AlreadyRegistered(String),

    #[error("Version conflict: {0}")]
    VersionConflict(String),

    #[error("Dependency not satisfied: {0}")]
    DependencyNotSatisfied(String),

    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type RegistryResult<T> = Result<T, RegistryError>;

/// Plugin registration entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRegistration {
    /// Plugin manifest
    pub manifest: PluginManifest,

    /// Installation path
    pub install_path: PathBuf,

    /// Registration timestamp
    pub registered_at: chrono::DateTime<chrono::Utc>,

    /// Last update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,

    /// Whether plugin is enabled
    pub enabled: bool,

    /// Installation source (marketplace, local, etc.)
    pub source: InstallationSource,

    /// Checksum of plugin files
    pub checksum: String,

    /// Plugin tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,

    /// Download count (for marketplace plugins)
    #[serde(default)]
    pub download_count: u64,

    /// User rating (0-5)
    #[serde(default)]
    pub rating: Option<f32>,
}

/// Installation source
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InstallationSource {
    /// Installed from official marketplace
    Marketplace { url: String },

    /// Installed from local file system
    Local,

    /// Installed from git repository
    Git { repo: String, commit: String },

    /// Installed from URL
    Url { url: String },

    /// Built-in plugin
    BuiltIn,
}

/// Plugin registry maintaining all available plugins
pub struct PluginRegistry {
    /// Registry file path
    registry_path: PathBuf,

    /// Registered plugins by ID
    plugins: HashMap<String, PluginRegistration>,

    /// Plugin categories
    categories: HashMap<String, HashSet<String>>,

    /// Version index (plugin_id -> versions)
    version_index: HashMap<String, Vec<String>>,

    /// Dependency graph
    dependency_graph: HashMap<String, HashSet<String>>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new<P: AsRef<Path>>(registry_path: P) -> Self {
        Self {
            registry_path: registry_path.as_ref().to_path_buf(),
            plugins: HashMap::new(),
            categories: HashMap::new(),
            version_index: HashMap::new(),
            dependency_graph: HashMap::new(),
        }
    }

    /// Load registry from disk
    pub async fn load(&mut self) -> RegistryResult<()> {
        if !self.registry_path.exists() {
            // Create empty registry
            self.save().await?;
            return Ok(());
        }

        let data = tokio::fs::read_to_string(&self.registry_path).await?;
        let saved_data: SavedRegistryData = serde_json::from_str(&data)?;

        self.plugins = saved_data.plugins;
        self.rebuild_indices();

        log::info!("Loaded plugin registry with {} plugins", self.plugins.len());

        Ok(())
    }

    /// Save registry to disk
    pub async fn save(&self) -> RegistryResult<()> {
        let data = SavedRegistryData {
            version: "1.0".to_string(),
            plugins: self.plugins.clone(),
        };

        let json = serde_json::to_string_pretty(&data)?;
        tokio::fs::write(&self.registry_path, json).await?;

        log::debug!("Saved plugin registry");

        Ok(())
    }

    /// Register a new plugin
    pub async fn register(
        &mut self,
        manifest: PluginManifest,
        install_path: PathBuf,
        source: InstallationSource,
    ) -> RegistryResult<()> {
        // Check if already registered
        if self.plugins.contains_key(&manifest.id) {
            return Err(RegistryError::AlreadyRegistered(manifest.id.clone()));
        }

        // Validate dependencies
        self.validate_dependencies(&manifest.dependencies)?;

        // Calculate checksum
        let checksum = self.calculate_checksum(&install_path).await?;

        // Extract tags from manifest
        let tags = manifest.capabilities.clone();

        let registration = PluginRegistration {
            manifest,
            install_path,
            registered_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            enabled: true,
            source,
            checksum,
            tags,
            download_count: 0,
            rating: None,
        };

        let plugin_id = registration.manifest.id.clone();
        self.plugins.insert(plugin_id.clone(), registration);
        self.rebuild_indices();

        // Save registry
        self.save().await?;

        log::info!("Registered plugin: {}", plugin_id);

        Ok(())
    }

    /// Unregister a plugin
    pub async fn unregister(&mut self, plugin_id: &str) -> RegistryResult<()> {
        // Check if plugin is a dependency of others
        let dependents = self.find_dependents(plugin_id);
        if !dependents.is_empty() {
            return Err(RegistryError::DependencyNotSatisfied(format!(
                "Cannot unregister {}: required by {:?}",
                plugin_id, dependents
            )));
        }

        self.plugins
            .remove(plugin_id)
            .ok_or_else(|| RegistryError::NotFound(plugin_id.to_string()))?;

        self.rebuild_indices();
        self.save().await?;

        log::info!("Unregistered plugin: {}", plugin_id);

        Ok(())
    }

    /// Update plugin registration
    pub async fn update(
        &mut self,
        plugin_id: &str,
        new_manifest: PluginManifest,
    ) -> RegistryResult<()> {
        // Validate new dependencies before getting mutable reference
        self.validate_dependencies(&new_manifest.dependencies)?;

        let registration = self
            .plugins
            .get_mut(plugin_id)
            .ok_or_else(|| RegistryError::NotFound(plugin_id.to_string()))?;

        registration.manifest = new_manifest;
        registration.updated_at = chrono::Utc::now();

        self.rebuild_indices();
        self.save().await?;

        log::info!("Updated plugin: {}", plugin_id);

        Ok(())
    }

    /// Get plugin registration
    pub fn get(&self, plugin_id: &str) -> Option<&PluginRegistration> {
        self.plugins.get(plugin_id)
    }

    /// List all registered plugins
    pub fn list_all(&self) -> Vec<&PluginRegistration> {
        self.plugins.values().collect()
    }

    /// List enabled plugins
    pub fn list_enabled(&self) -> Vec<&PluginRegistration> {
        self.plugins
            .values()
            .filter(|p| p.enabled)
            .collect()
    }

    /// Search plugins by query
    pub fn search(&self, query: &str) -> Vec<&PluginRegistration> {
        let query_lower = query.to_lowercase();

        self.plugins
            .values()
            .filter(|p| {
                p.manifest.name.to_lowercase().contains(&query_lower)
                    || p.manifest.description.to_lowercase().contains(&query_lower)
                    || p.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    /// Filter plugins by category
    pub fn filter_by_category(&self, category: &str) -> Vec<&PluginRegistration> {
        if let Some(plugin_ids) = self.categories.get(category) {
            plugin_ids
                .iter()
                .filter_map(|id| self.plugins.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get plugin dependencies
    pub fn get_dependencies(&self, plugin_id: &str) -> Vec<&PluginRegistration> {
        if let Some(deps) = self.dependency_graph.get(plugin_id) {
            deps.iter()
                .filter_map(|id| self.plugins.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Find plugins that depend on this plugin
    pub fn find_dependents(&self, plugin_id: &str) -> Vec<String> {
        self.dependency_graph
            .iter()
            .filter(|(_, deps)| deps.contains(plugin_id))
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Enable a plugin
    pub async fn enable(&mut self, plugin_id: &str) -> RegistryResult<()> {
        let registration = self
            .plugins
            .get_mut(plugin_id)
            .ok_or_else(|| RegistryError::NotFound(plugin_id.to_string()))?;

        registration.enabled = true;
        self.save().await?;

        log::info!("Enabled plugin: {}", plugin_id);

        Ok(())
    }

    /// Disable a plugin
    pub async fn disable(&mut self, plugin_id: &str) -> RegistryResult<()> {
        let registration = self
            .plugins
            .get_mut(plugin_id)
            .ok_or_else(|| RegistryError::NotFound(plugin_id.to_string()))?;

        registration.enabled = false;
        self.save().await?;

        log::info!("Disabled plugin: {}", plugin_id);

        Ok(())
    }

    /// Validate plugin dependencies
    fn validate_dependencies(
        &self,
        dependencies: &[super::loader::PluginDependency],
    ) -> RegistryResult<()> {
        for dep in dependencies {
            if dep.optional {
                continue;
            }

            // Check if dependency is registered
            if !self.plugins.contains_key(&dep.id) {
                return Err(RegistryError::DependencyNotSatisfied(format!(
                    "Required dependency not registered: {}",
                    dep.id
                )));
            }

            // TODO: Validate version requirements using semver
        }

        Ok(())
    }

    /// Rebuild internal indices
    fn rebuild_indices(&mut self) {
        self.categories.clear();
        self.version_index.clear();
        self.dependency_graph.clear();

        for (plugin_id, registration) in &self.plugins {
            // Build category index
            for tag in &registration.tags {
                self.categories
                    .entry(tag.clone())
                    .or_insert_with(HashSet::new)
                    .insert(plugin_id.clone());
            }

            // Build version index
            self.version_index
                .entry(plugin_id.clone())
                .or_insert_with(Vec::new)
                .push(registration.manifest.version.clone());

            // Build dependency graph
            let deps: HashSet<String> = registration
                .manifest
                .dependencies
                .iter()
                .map(|d| d.id.clone())
                .collect();

            self.dependency_graph.insert(plugin_id.clone(), deps);
        }
    }

    /// Calculate checksum of plugin files
    async fn calculate_checksum(&self, _path: &Path) -> RegistryResult<String> {
        // In a real implementation, we would:
        // 1. Hash all files in the plugin directory
        // 2. Create a composite checksum

        // Placeholder implementation
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(b"placeholder");
        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    /// Get statistics
    pub fn stats(&self) -> RegistryStats {
        let enabled_count = self.plugins.values().filter(|p| p.enabled).count();
        let total_downloads: u64 = self.plugins.values().map(|p| p.download_count).sum();

        let source_breakdown: HashMap<String, usize> = self
            .plugins
            .values()
            .map(|p| match &p.source {
                InstallationSource::Marketplace { .. } => "marketplace",
                InstallationSource::Local => "local",
                InstallationSource::Git { .. } => "git",
                InstallationSource::Url { .. } => "url",
                InstallationSource::BuiltIn => "builtin",
            })
            .fold(HashMap::new(), |mut acc, source| {
                *acc.entry(source.to_string()).or_insert(0) += 1;
                acc
            });

        RegistryStats {
            total_plugins: self.plugins.len(),
            enabled_plugins: enabled_count,
            disabled_plugins: self.plugins.len() - enabled_count,
            total_downloads,
            categories_count: self.categories.len(),
            source_breakdown,
        }
    }
}

/// Saved registry data format
#[derive(Debug, Serialize, Deserialize)]
struct SavedRegistryData {
    version: String,
    plugins: HashMap<String, PluginRegistration>,
}

/// Registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    pub total_plugins: usize,
    pub enabled_plugins: usize,
    pub disabled_plugins: usize,
    pub total_downloads: u64,
    pub categories_count: usize,
    pub source_breakdown: HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_installation_source_serialization() {
        let source = InstallationSource::Marketplace {
            url: "https://marketplace.caddy.dev".to_string(),
        };

        let json = serde_json::to_string(&source).unwrap();
        let deserialized: InstallationSource = serde_json::from_str(&json).unwrap();

        assert_eq!(source, deserialized);
    }

    #[tokio::test]
    async fn test_registry_creation() {
        let temp_dir = std::env::temp_dir();
        let registry_path = temp_dir.join("test_registry.json");

        let mut registry = PluginRegistry::new(&registry_path);
        registry.save().await.unwrap();

        assert!(registry_path.exists());

        // Cleanup
        let _ = tokio::fs::remove_file(registry_path).await;
    }
}
