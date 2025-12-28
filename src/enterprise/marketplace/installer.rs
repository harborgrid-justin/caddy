//! Plugin installer with dependency resolution and rollback
//!
//! This module provides comprehensive plugin installation functionality,
//! including download, verification, dependency resolution, and rollback capabilities.

use super::{LocalRegistry, MarketplaceError, PluginManifest, PluginMetadata, RegistryClient, Result};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use uuid::Uuid;

/// Plugin downloader
#[derive(Debug, Clone)]
pub struct PluginDownloader {
    /// Remote registry client
    client: RegistryClient,

    /// Download cache directory
    cache_dir: PathBuf,
}

impl PluginDownloader {
    /// Create a new plugin downloader
    pub fn new<P: AsRef<Path>>(client: RegistryClient, cache_dir: P) -> Result<Self> {
        let cache_dir = cache_dir.as_ref().to_path_buf();

        // Ensure cache directory exists
        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir)?;
        }

        Ok(Self { client, cache_dir })
    }

    /// Download plugin package
    pub async fn download(&self, plugin_id: Uuid) -> Result<PathBuf> {
        // Check cache first
        let cache_path = self.cache_dir.join(format!("{}.plugin", plugin_id));

        if cache_path.exists() {
            return Ok(cache_path);
        }

        // Download from remote
        let data = self.client.download(plugin_id).await?;

        // Write to cache
        std::fs::write(&cache_path, data)?;

        Ok(cache_path)
    }

    /// Download with progress callback
    pub async fn download_with_progress<F>(
        &self,
        plugin_id: Uuid,
        mut progress_callback: F,
    ) -> Result<PathBuf>
    where
        F: FnMut(u64, u64) + Send,
    {
        let cache_path = self.cache_dir.join(format!("{}.plugin", plugin_id));

        if cache_path.exists() {
            return Ok(cache_path);
        }

        let data = self.client.download(plugin_id).await?;
        let total_size = data.len() as u64;

        // Simulate progress (in real implementation, this would be chunk-based)
        progress_callback(0, total_size);

        std::fs::write(&cache_path, &data)?;

        progress_callback(total_size, total_size);

        Ok(cache_path)
    }

    /// Clear download cache
    pub fn clear_cache(&self) -> Result<()> {
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)?;
            std::fs::create_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }
}

/// Plugin verifier
#[derive(Debug)]
pub struct PluginVerifier {
    /// Trusted signers
    trusted_signers: Arc<RwLock<HashSet<String>>>,
}

impl PluginVerifier {
    /// Create a new plugin verifier
    pub fn new() -> Self {
        Self {
            trusted_signers: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Add trusted signer
    pub fn add_trusted_signer(&self, signer: String) {
        self.trusted_signers.write().insert(signer);
    }

    /// Verify plugin package
    pub fn verify(&self, plugin_path: &Path, manifest: &PluginManifest) -> Result<()> {
        // Verify file exists
        if !plugin_path.exists() {
            return Err(MarketplaceError::VerificationFailed(
                "Plugin file not found".to_string()
            ));
        }

        // Verify checksum
        let actual_checksum = self.calculate_checksum(plugin_path)?;
        if actual_checksum != manifest.checksum {
            return Err(MarketplaceError::VerificationFailed(
                format!("Checksum mismatch: expected {}, got {}",
                    manifest.checksum, actual_checksum)
            ));
        }

        // Verify signature if present
        if let Some(signature) = &manifest.signature {
            self.verify_signature(plugin_path, signature, &manifest.author)?;
        }

        // Verify file size
        let metadata = std::fs::metadata(plugin_path)?;
        if metadata.len() != manifest.size {
            return Err(MarketplaceError::VerificationFailed(
                format!("Size mismatch: expected {}, got {}",
                    manifest.size, metadata.len())
            ));
        }

        Ok(())
    }

    /// Calculate file checksum (SHA-256)
    fn calculate_checksum(&self, path: &Path) -> Result<String> {
        use sha2::{Digest, Sha256};

        let data = std::fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let result = hasher.finalize();

        Ok(format!("{:x}", result))
    }

    /// Verify plugin signature
    fn verify_signature(&self, _path: &Path, _signature: &str, author: &str) -> Result<()> {
        // Check if author is trusted
        let trusted = self.trusted_signers.read();
        if !trusted.contains(author) {
            return Err(MarketplaceError::VerificationFailed(
                format!("Untrusted author: {}", author)
            ));
        }

        // In a real implementation, this would verify the cryptographic signature
        // using public key cryptography (e.g., RSA, Ed25519)

        Ok(())
    }
}

impl Default for PluginVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Dependency resolver
#[derive(Debug)]
pub struct DependencyResolver {
    /// Local registry
    registry: LocalRegistry,

    /// Remote client
    client: RegistryClient,
}

impl DependencyResolver {
    /// Create a new dependency resolver
    pub fn new(registry: LocalRegistry, client: RegistryClient) -> Self {
        Self { registry, client }
    }

    /// Resolve dependencies for a plugin
    pub async fn resolve(&self, manifest: &PluginManifest) -> Result<Vec<PluginManifest>> {
        let mut resolved = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        // Start with the plugin's dependencies
        for (dep_name, version_req) in &manifest.dependencies {
            queue.push_back((dep_name.clone(), version_req.clone()));
        }

        while let Some((dep_name, version_req)) = queue.pop_front() {
            if visited.contains(&dep_name) {
                continue;
            }

            visited.insert(dep_name.clone());

            // Try to find dependency locally
            let dep_metadata = if let Some(local) = self.registry.get_by_name(&dep_name) {
                if self.satisfies_version(&local.manifest.version, &version_req)? {
                    local
                } else {
                    // Need to fetch newer version
                    self.fetch_dependency(&dep_name, &version_req).await?
                }
            } else {
                // Fetch from remote
                self.fetch_dependency(&dep_name, &version_req).await?
            };

            // Add to resolved list
            resolved.push(dep_metadata.manifest.clone());

            // Add transitive dependencies to queue
            for (trans_dep_name, trans_version_req) in &dep_metadata.manifest.dependencies {
                if !visited.contains(trans_dep_name) {
                    queue.push_back((trans_dep_name.clone(), trans_version_req.clone()));
                }
            }
        }

        Ok(resolved)
    }

    /// Check if version satisfies requirement
    fn satisfies_version(&self, version: &str, requirement: &str) -> Result<bool> {
        // Simple version comparison (in production, use semver crate)
        if requirement.starts_with(">=") {
            let req_version = requirement.trim_start_matches(">=");
            Ok(version >= req_version)
        } else if requirement.starts_with('>') {
            let req_version = requirement.trim_start_matches('>');
            Ok(version > req_version)
        } else if requirement.starts_with("<=") {
            let req_version = requirement.trim_start_matches("<=");
            Ok(version <= req_version)
        } else if requirement.starts_with('<') {
            let req_version = requirement.trim_start_matches('<');
            Ok(version < req_version)
        } else if requirement.starts_with('=') {
            let req_version = requirement.trim_start_matches('=');
            Ok(version == req_version)
        } else {
            Ok(version == requirement)
        }
    }

    /// Fetch dependency from remote
    async fn fetch_dependency(&self, name: &str, _version_req: &str) -> Result<PluginMetadata> {
        // Search for plugin by name
        let results = self.client.search(name, 1).await?;

        if let Some(metadata) = results.into_iter().next() {
            Ok(metadata)
        } else {
            Err(MarketplaceError::DependencyResolutionFailed(
                format!("Dependency not found: {}", name)
            ))
        }
    }
}

/// Plugin installer
#[derive(Debug)]
pub struct PluginInstaller {
    /// Installation directory
    install_dir: PathBuf,

    /// Local registry
    registry: LocalRegistry,

    /// Downloader
    downloader: PluginDownloader,

    /// Verifier
    verifier: PluginVerifier,

    /// Dependency resolver
    resolver: DependencyResolver,

    /// Rollback manager
    rollback: RollbackManager,
}

impl PluginInstaller {
    /// Create a new plugin installer
    pub fn new(
        install_dir: PathBuf,
        registry: LocalRegistry,
        downloader: PluginDownloader,
        verifier: PluginVerifier,
        resolver: DependencyResolver,
        rollback: RollbackManager,
    ) -> Result<Self> {
        // Ensure installation directory exists
        if !install_dir.exists() {
            std::fs::create_dir_all(&install_dir)?;
        }

        Ok(Self {
            install_dir,
            registry,
            downloader,
            verifier,
            resolver,
            rollback,
        })
    }

    /// Install a plugin
    pub async fn install(&self, plugin_id: Uuid) -> Result<InstallationResult> {
        // Start installation transaction
        let transaction_id = self.rollback.begin_transaction()?;

        let result = self.install_internal(plugin_id, transaction_id).await;

        match result {
            Ok(result) => {
                self.rollback.commit_transaction(transaction_id)?;
                Ok(result)
            }
            Err(e) => {
                self.rollback.rollback_transaction(transaction_id)?;
                Err(e)
            }
        }
    }

    /// Internal installation logic
    async fn install_internal(
        &self,
        plugin_id: Uuid,
        transaction_id: Uuid,
    ) -> Result<InstallationResult> {
        let mut result = InstallationResult::default();

        // Fetch plugin metadata
        let metadata = self.registry.get(plugin_id)
            .ok_or_else(|| MarketplaceError::PluginNotFound(plugin_id.to_string()))?;

        // Resolve dependencies
        let dependencies = self.resolver.resolve(&metadata.manifest).await?;
        result.dependencies_installed = dependencies.len();

        // Install dependencies first
        for dep_manifest in dependencies {
            if self.registry.get_by_name(&dep_manifest.name).is_none() {
                // Dependency not installed, install it (boxed to avoid recursion)
                Box::pin(self.install_internal(dep_manifest.id, transaction_id)).await?;
            }
        }

        // Download plugin
        let plugin_path = self.downloader.download(plugin_id).await?;

        // Verify plugin
        self.verifier.verify(&plugin_path, &metadata.manifest)?;

        // Install plugin files
        let plugin_install_dir = self.install_dir.join(plugin_id.to_string());
        std::fs::create_dir_all(&plugin_install_dir)?;

        // Record for rollback
        self.rollback.record_installation(transaction_id, plugin_install_dir.clone())?;

        // Copy plugin files (in production, would extract archive)
        std::fs::copy(&plugin_path, plugin_install_dir.join("plugin.bin"))?;

        // Register in local registry
        self.registry.register(metadata.clone())?;

        result.plugin_id = plugin_id;
        result.success = true;
        result.installed_at = Utc::now();

        Ok(result)
    }

    /// Uninstall a plugin
    pub fn uninstall(&self, plugin_id: Uuid) -> Result<()> {
        // Check if plugin is installed
        let metadata = self.registry.get(plugin_id)
            .ok_or_else(|| MarketplaceError::PluginNotFound(plugin_id.to_string()))?;

        // Remove plugin files
        let plugin_dir = self.install_dir.join(plugin_id.to_string());
        if plugin_dir.exists() {
            std::fs::remove_dir_all(plugin_dir)?;
        }

        // Unregister from registry
        self.registry.unregister(plugin_id)?;

        Ok(())
    }

    /// Upgrade a plugin
    pub async fn upgrade(&self, plugin_id: Uuid) -> Result<InstallationResult> {
        // Uninstall old version
        self.uninstall(plugin_id)?;

        // Install new version
        self.install(plugin_id).await
    }

    /// Downgrade a plugin to specific version
    pub async fn downgrade(&self, plugin_id: Uuid, _target_version: String) -> Result<InstallationResult> {
        // In production, would fetch specific version
        // For now, just reinstall current
        self.upgrade(plugin_id).await
    }
}

/// Rollback manager
#[derive(Debug)]
pub struct RollbackManager {
    /// Active transactions
    transactions: Arc<RwLock<HashMap<Uuid, Transaction>>>,

    /// Rollback log directory
    log_dir: PathBuf,
}

impl RollbackManager {
    /// Create a new rollback manager
    pub fn new<P: AsRef<Path>>(log_dir: P) -> Result<Self> {
        let log_dir = log_dir.as_ref().to_path_buf();

        if !log_dir.exists() {
            std::fs::create_dir_all(&log_dir)?;
        }

        Ok(Self {
            transactions: Arc::new(RwLock::new(HashMap::new())),
            log_dir,
        })
    }

    /// Begin a new transaction
    pub fn begin_transaction(&self) -> Result<Uuid> {
        let transaction_id = Uuid::new_v4();
        let transaction = Transaction {
            id: transaction_id,
            started_at: Utc::now(),
            operations: Vec::new(),
        };

        self.transactions.write().insert(transaction_id, transaction);

        Ok(transaction_id)
    }

    /// Record an installation
    pub fn record_installation(&self, transaction_id: Uuid, path: PathBuf) -> Result<()> {
        let mut transactions = self.transactions.write();

        if let Some(transaction) = transactions.get_mut(&transaction_id) {
            transaction.operations.push(RollbackOperation::Install { path });
            Ok(())
        } else {
            Err(MarketplaceError::InstallationFailed(
                "Transaction not found".to_string()
            ))
        }
    }

    /// Commit a transaction
    pub fn commit_transaction(&self, transaction_id: Uuid) -> Result<()> {
        self.transactions.write().remove(&transaction_id);
        Ok(())
    }

    /// Rollback a transaction
    pub fn rollback_transaction(&self, transaction_id: Uuid) -> Result<()> {
        let mut transactions = self.transactions.write();

        if let Some(transaction) = transactions.remove(&transaction_id) {
            // Execute rollback operations in reverse order
            for operation in transaction.operations.iter().rev() {
                match operation {
                    RollbackOperation::Install { path } => {
                        if path.exists() {
                            std::fs::remove_dir_all(path)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

/// Transaction record
#[derive(Debug, Clone)]
struct Transaction {
    /// Transaction ID
    id: Uuid,

    /// Start timestamp
    started_at: DateTime<Utc>,

    /// Operations performed
    operations: Vec<RollbackOperation>,
}

/// Rollback operation
#[derive(Debug, Clone)]
enum RollbackOperation {
    /// Installation operation
    Install { path: PathBuf },
}

/// Installation result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstallationResult {
    /// Plugin ID
    pub plugin_id: Uuid,

    /// Success flag
    pub success: bool,

    /// Installation timestamp
    pub installed_at: DateTime<Utc>,

    /// Number of dependencies installed
    pub dependencies_installed: usize,

    /// Error message if failed
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_satisfaction() {
        let resolver = DependencyResolver {
            registry: LocalRegistry::new(std::env::temp_dir()).unwrap(),
            client: RegistryClient::new("http://example.com".to_string()),
        };

        assert!(resolver.satisfies_version("1.2.3", ">=1.0.0").unwrap());
        assert!(resolver.satisfies_version("1.2.3", ">1.0.0").unwrap());
        assert!(!resolver.satisfies_version("1.2.3", ">2.0.0").unwrap());
        assert!(resolver.satisfies_version("1.2.3", "=1.2.3").unwrap());
    }

    #[test]
    fn test_rollback_manager() -> Result<()> {
        let temp_dir = std::env::temp_dir().join("caddy_rollback_test");
        let manager = RollbackManager::new(&temp_dir)?;

        let tx_id = manager.begin_transaction()?;

        let test_path = temp_dir.join("test_install");
        std::fs::create_dir_all(&test_path)?;

        manager.record_installation(tx_id, test_path.clone())?;

        assert!(test_path.exists());

        manager.rollback_transaction(tx_id)?;

        assert!(!test_path.exists());

        std::fs::remove_dir_all(temp_dir)?;
        Ok(())
    }
}
