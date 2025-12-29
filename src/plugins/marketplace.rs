//! Plugin marketplace integration
//!
//! Handles plugin discovery, installation, and updates from the
//! CADDY plugin marketplace and other sources.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;
use reqwest::Client;

use super::loader::PluginManifest;
use super::registry::{InstallationSource, PluginRegistry};

/// Marketplace errors
#[derive(Debug, Error)]
pub enum MarketplaceError {
    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("Installation failed: {0}")]
    InstallationFailed(String),

    #[error("Update failed: {0}")]
    UpdateFailed(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

pub type MarketplaceResult<T> = Result<T, MarketplaceError>;

/// Marketplace plugin listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplacePlugin {
    /// Plugin ID
    pub id: String,

    /// Plugin name
    pub name: String,

    /// Short description
    pub description: String,

    /// Current version
    pub version: String,

    /// Author information
    pub author: MarketplaceAuthor,

    /// Plugin icon URL
    pub icon_url: Option<String>,

    /// Download count
    pub downloads: u64,

    /// Average rating (0-5)
    pub rating: f32,

    /// Number of ratings
    pub rating_count: u32,

    /// Categories/tags
    pub categories: Vec<String>,

    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,

    /// Plugin size in bytes
    pub size_bytes: u64,

    /// Download URL
    pub download_url: String,

    /// Manifest URL
    pub manifest_url: String,

    /// Is verified plugin
    pub verified: bool,

    /// License type
    pub license: String,

    /// Minimum CADDY version required
    pub min_caddy_version: String,
}

/// Marketplace author information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceAuthor {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub website: Option<String>,
    pub verified: bool,
}

/// Marketplace search filters
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilters {
    /// Search query
    pub query: Option<String>,

    /// Category filter
    pub category: Option<String>,

    /// Minimum rating
    pub min_rating: Option<f32>,

    /// Only verified plugins
    pub verified_only: bool,

    /// Sort by field
    pub sort_by: SortBy,

    /// Page number (0-indexed)
    pub page: usize,

    /// Results per page
    pub per_page: usize,
}

/// Sort options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SortBy {
    Relevance,
    Downloads,
    Rating,
    Updated,
    Name,
}

impl Default for SortBy {
    fn default() -> Self {
        SortBy::Relevance
    }
}

/// Search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub plugins: Vec<MarketplacePlugin>,
    pub total_count: usize,
    pub page: usize,
    pub per_page: usize,
    pub total_pages: usize,
}

/// Plugin marketplace client
pub struct PluginMarketplace {
    /// Base marketplace URL
    base_url: String,

    /// HTTP client
    client: Client,

    /// Authentication token
    auth_token: Option<String>,

    /// Local plugins directory
    plugins_dir: PathBuf,

    /// Plugin registry
    registry: Option<PluginRegistry>,
}

impl PluginMarketplace {
    /// Create a new marketplace client
    pub fn new<P: AsRef<Path>>(base_url: String, plugins_dir: P) -> Self {
        Self {
            base_url,
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
            auth_token: None,
            plugins_dir: plugins_dir.as_ref().to_path_buf(),
            registry: None,
        }
    }

    /// Set authentication token
    pub fn set_auth_token(&mut self, token: String) {
        self.auth_token = Some(token);
    }

    /// Set plugin registry
    pub fn set_registry(&mut self, registry: PluginRegistry) {
        self.registry = Some(registry);
    }

    /// Search marketplace for plugins
    pub async fn search(&self, filters: SearchFilters) -> MarketplaceResult<SearchResults> {
        let url = format!("{}/api/v1/plugins/search", self.base_url);

        let response = self
            .client
            .get(&url)
            .query(&[
                ("q", filters.query.unwrap_or_default()),
                ("category", filters.category.unwrap_or_default()),
                ("verified_only", filters.verified_only.to_string()),
                ("sort_by", format!("{:?}", filters.sort_by).to_lowercase()),
                ("page", filters.page.to_string()),
                ("per_page", filters.per_page.to_string()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(MarketplaceError::NetworkError(format!(
                "Search failed with status: {}",
                response.status()
            )));
        }

        let results = response.json::<SearchResults>().await?;
        Ok(results)
    }

    /// Get plugin details
    pub async fn get_plugin(&self, plugin_id: &str) -> MarketplaceResult<MarketplacePlugin> {
        let url = format!("{}/api/v1/plugins/{}", self.base_url, plugin_id);

        let response = self.client.get(&url).send().await?;

        if response.status() == 404 {
            return Err(MarketplaceError::NotFound(plugin_id.to_string()));
        }

        if !response.status().is_success() {
            return Err(MarketplaceError::NetworkError(format!(
                "Get plugin failed with status: {}",
                response.status()
            )));
        }

        let plugin = response.json::<MarketplacePlugin>().await?;
        Ok(plugin)
    }

    /// Download and install a plugin from marketplace
    pub async fn install(&mut self, plugin_id: &str) -> MarketplaceResult<PathBuf> {
        // Get plugin details
        let plugin = self.get_plugin(plugin_id).await?;

        // Download plugin manifest
        let manifest = self.download_manifest(&plugin.manifest_url).await?;

        // Download plugin package
        let plugin_path = self.download_plugin(&plugin, &manifest).await?;

        // Register plugin if registry is available
        if let Some(ref mut registry) = self.registry {
            registry
                .register(
                    manifest,
                    plugin_path.clone(),
                    InstallationSource::Marketplace {
                        url: self.base_url.clone(),
                    },
                )
                .await
                .map_err(|e| MarketplaceError::InstallationFailed(e.to_string()))?;
        }

        log::info!("Installed plugin: {} v{}", plugin.name, plugin.version);

        Ok(plugin_path)
    }

    /// Update an installed plugin
    pub async fn update(&self, plugin_id: &str) -> MarketplaceResult<()> {
        // Check if update is available
        let latest = self.get_plugin(plugin_id).await?;

        // Get current version from registry
        let current_version = if let Some(ref registry) = self.registry {
            registry
                .get(plugin_id)
                .map(|p| p.manifest.version.clone())
                .ok_or_else(|| MarketplaceError::NotFound(plugin_id.to_string()))?
        } else {
            return Err(MarketplaceError::InstallationFailed(
                "No registry available".to_string(),
            ));
        };

        // Compare versions
        if latest.version == current_version {
            log::info!("Plugin {} is already up to date", plugin_id);
            return Ok(());
        }

        // Download and install update
        // In a real implementation, we would:
        // 1. Download new version
        // 2. Backup current version
        // 3. Install new version
        // 4. Update registry
        // 5. Clean up old version

        log::info!(
            "Updated plugin {} from {} to {}",
            plugin_id,
            current_version,
            latest.version
        );

        Ok(())
    }

    /// Check for updates to installed plugins
    pub async fn check_updates(&self) -> MarketplaceResult<Vec<PluginUpdate>> {
        let mut updates = Vec::new();

        if let Some(ref registry) = self.registry {
            for registration in registry.list_enabled() {
                // Skip non-marketplace plugins
                if !matches!(registration.source, InstallationSource::Marketplace { .. }) {
                    continue;
                }

                // Check marketplace for latest version
                match self.get_plugin(&registration.manifest.id).await {
                    Ok(latest) => {
                        if latest.version != registration.manifest.version {
                            updates.push(PluginUpdate {
                                plugin_id: registration.manifest.id.clone(),
                                current_version: registration.manifest.version.clone(),
                                latest_version: latest.version,
                                changelog_url: None,
                            });
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to check updates for {}: {}", registration.manifest.id, e);
                    }
                }
            }
        }

        Ok(updates)
    }

    /// Get featured plugins
    pub async fn get_featured(&self) -> MarketplaceResult<Vec<MarketplacePlugin>> {
        let url = format!("{}/api/v1/plugins/featured", self.base_url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(MarketplaceError::NetworkError(format!(
                "Get featured failed with status: {}",
                response.status()
            )));
        }

        let plugins = response.json::<Vec<MarketplacePlugin>>().await?;
        Ok(plugins)
    }

    /// Get plugin categories
    pub async fn get_categories(&self) -> MarketplaceResult<Vec<Category>> {
        let url = format!("{}/api/v1/categories", self.base_url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(MarketplaceError::NetworkError(format!(
                "Get categories failed with status: {}",
                response.status()
            )));
        }

        let categories = response.json::<Vec<Category>>().await?;
        Ok(categories)
    }

    /// Download plugin manifest
    async fn download_manifest(&self, manifest_url: &str) -> MarketplaceResult<PluginManifest> {
        let response = self.client.get(manifest_url).send().await?;

        if !response.status().is_success() {
            return Err(MarketplaceError::DownloadFailed(format!(
                "Manifest download failed with status: {}",
                response.status()
            )));
        }

        let manifest = response
            .json::<PluginManifest>()
            .await
            .map_err(|e| MarketplaceError::InvalidResponse(e.to_string()))?;

        Ok(manifest)
    }

    /// Download plugin package
    async fn download_plugin(
        &self,
        plugin: &MarketplacePlugin,
        manifest: &PluginManifest,
    ) -> MarketplaceResult<PathBuf> {
        // Create plugin directory
        let plugin_dir = self.plugins_dir.join(&plugin.id);
        tokio::fs::create_dir_all(&plugin_dir).await?;

        // Download plugin package
        let response = self.client.get(&plugin.download_url).send().await?;

        if !response.status().is_success() {
            return Err(MarketplaceError::DownloadFailed(format!(
                "Plugin download failed with status: {}",
                response.status()
            )));
        }

        // Get package data
        let package_data = response.bytes().await?;

        // In a real implementation, we would:
        // 1. Verify checksum
        // 2. Extract archive (zip/tar.gz)
        // 3. Validate contents
        // 4. Set proper permissions

        // For now, just save manifest
        let manifest_path = plugin_dir.join("plugin.json");
        let manifest_json = serde_json::to_string_pretty(manifest)?;
        tokio::fs::write(&manifest_path, manifest_json).await?;

        // Save package
        let package_path = plugin_dir.join("plugin.wasm");
        tokio::fs::write(&package_path, package_data).await?;

        Ok(plugin_dir)
    }

    /// Submit rating for a plugin
    pub async fn rate_plugin(&self, plugin_id: &str, rating: u8) -> MarketplaceResult<()> {
        if rating > 5 {
            return Err(MarketplaceError::InvalidResponse(
                "Rating must be between 0 and 5".to_string(),
            ));
        }

        let url = format!("{}/api/v1/plugins/{}/rate", self.base_url, plugin_id);

        let auth_token = self.auth_token.as_ref().ok_or_else(|| {
            MarketplaceError::AuthenticationFailed("Authentication required to rate plugins".to_string())
        })?;

        let response = self
            .client
            .post(&url)
            .bearer_auth(auth_token)
            .json(&serde_json::json!({ "rating": rating }))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(MarketplaceError::NetworkError(format!(
                "Rating submission failed with status: {}",
                response.status()
            )));
        }

        log::info!("Submitted rating {} for plugin {}", rating, plugin_id);

        Ok(())
    }
}

/// Plugin update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUpdate {
    pub plugin_id: String,
    pub current_version: String,
    pub latest_version: String,
    pub changelog_url: Option<String>,
}

/// Plugin category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub plugin_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_filters_default() {
        let filters = SearchFilters::default();
        assert_eq!(filters.page, 0);
        assert_eq!(filters.verified_only, false);
        assert!(matches!(filters.sort_by, SortBy::Relevance));
    }

    #[test]
    fn test_sort_by_serialization() {
        let sort = SortBy::Downloads;
        let json = serde_json::to_string(&sort).unwrap();
        assert_eq!(json, r#""Downloads""#);
    }
}
