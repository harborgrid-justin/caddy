//! Plugin sandbox environment
//!
//! This module provides a secure sandboxed execution environment for plugins,
//! including resource limits, permission enforcement, and API access control.

use super::{MarketplaceError, PluginManifest, PluginPermission, Result};
use parking_lot::RwLock;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Resource limits for sandboxed plugins
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum CPU time (milliseconds)
    pub max_cpu_time_ms: u64,

    /// Maximum memory usage (bytes)
    pub max_memory_bytes: u64,

    /// Maximum disk space (bytes)
    pub max_disk_bytes: u64,

    /// Maximum network bandwidth (bytes/sec)
    pub max_network_bandwidth_bps: u64,

    /// Maximum number of threads
    pub max_threads: usize,

    /// Maximum file handles
    pub max_file_handles: usize,

    /// Maximum execution time (milliseconds)
    pub max_execution_time_ms: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu_time_ms: 60_000,           // 1 minute
            max_memory_bytes: 512 * 1024 * 1024, // 512 MB
            max_disk_bytes: 1024 * 1024 * 1024,  // 1 GB
            max_network_bandwidth_bps: 10 * 1024 * 1024, // 10 MB/s
            max_threads: 4,
            max_file_handles: 100,
            max_execution_time_ms: 300_000,    // 5 minutes
        }
    }
}

impl ResourceLimits {
    /// Create conservative limits for untrusted plugins
    pub fn conservative() -> Self {
        Self {
            max_cpu_time_ms: 10_000,            // 10 seconds
            max_memory_bytes: 128 * 1024 * 1024, // 128 MB
            max_disk_bytes: 256 * 1024 * 1024,   // 256 MB
            max_network_bandwidth_bps: 1024 * 1024, // 1 MB/s
            max_threads: 2,
            max_file_handles: 20,
            max_execution_time_ms: 60_000,      // 1 minute
        }
    }

    /// Create generous limits for trusted plugins
    pub fn generous() -> Self {
        Self {
            max_cpu_time_ms: 300_000,           // 5 minutes
            max_memory_bytes: 2 * 1024 * 1024 * 1024, // 2 GB
            max_disk_bytes: 10 * 1024 * 1024 * 1024,  // 10 GB
            max_network_bandwidth_bps: 100 * 1024 * 1024, // 100 MB/s
            max_threads: 16,
            max_file_handles: 500,
            max_execution_time_ms: 3_600_000,    // 1 hour
        }
    }
}

/// Resource usage tracking
#[derive(Debug, Clone, Default)]
pub struct ResourceUsage {
    /// CPU time used (milliseconds)
    pub cpu_time_ms: u64,

    /// Memory used (bytes)
    pub memory_bytes: u64,

    /// Disk space used (bytes)
    pub disk_bytes: u64,

    /// Network bytes transferred
    pub network_bytes: u64,

    /// Active threads
    pub active_threads: usize,

    /// Open file handles
    pub open_file_handles: usize,

    /// Execution start time
    pub started_at: Option<Instant>,
}

impl ResourceUsage {
    /// Check if usage exceeds limits
    pub fn exceeds_limits(&self, limits: &ResourceLimits) -> Option<String> {
        if self.cpu_time_ms > limits.max_cpu_time_ms {
            return Some(format!(
                "CPU time limit exceeded: {} ms > {} ms",
                self.cpu_time_ms, limits.max_cpu_time_ms
            ));
        }

        if self.memory_bytes > limits.max_memory_bytes {
            return Some(format!(
                "Memory limit exceeded: {} bytes > {} bytes",
                self.memory_bytes, limits.max_memory_bytes
            ));
        }

        if self.disk_bytes > limits.max_disk_bytes {
            return Some(format!(
                "Disk space limit exceeded: {} bytes > {} bytes",
                self.disk_bytes, limits.max_disk_bytes
            ));
        }

        if self.active_threads > limits.max_threads {
            return Some(format!(
                "Thread limit exceeded: {} > {}",
                self.active_threads, limits.max_threads
            ));
        }

        if self.open_file_handles > limits.max_file_handles {
            return Some(format!(
                "File handle limit exceeded: {} > {}",
                self.open_file_handles, limits.max_file_handles
            ));
        }

        if let Some(started_at) = self.started_at {
            let elapsed_ms = started_at.elapsed().as_millis() as u64;
            if elapsed_ms > limits.max_execution_time_ms {
                return Some(format!(
                    "Execution time limit exceeded: {} ms > {} ms",
                    elapsed_ms, limits.max_execution_time_ms
                ));
            }
        }

        None
    }
}

/// Permission enforcer
#[derive(Debug)]
pub struct PermissionEnforcer {
    /// Granted permissions
    granted: Arc<RwLock<HashSet<PluginPermission>>>,

    /// Permission audit log
    audit_log: Arc<RwLock<Vec<PermissionAudit>>>,
}

impl PermissionEnforcer {
    /// Create a new permission enforcer
    pub fn new() -> Self {
        Self {
            granted: Arc::new(RwLock::new(HashSet::new())),
            audit_log: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Grant a permission
    pub fn grant(&self, permission: PluginPermission) {
        self.granted.write().insert(permission);
    }

    /// Revoke a permission
    pub fn revoke(&self, permission: PluginPermission) {
        self.granted.write().remove(&permission);
    }

    /// Check if permission is granted
    pub fn check(&self, permission: PluginPermission) -> Result<()> {
        let granted = self.granted.read();

        if granted.contains(&permission) {
            // Log successful check
            self.audit_log.write().push(PermissionAudit {
                permission,
                granted: true,
                timestamp: Instant::now(),
            });
            Ok(())
        } else {
            // Log denied check
            self.audit_log.write().push(PermissionAudit {
                permission,
                granted: false,
                timestamp: Instant::now(),
            });
            Err(MarketplaceError::PermissionDenied(
                format!("Permission not granted: {:?}", permission)
            ))
        }
    }

    /// Get audit log
    pub fn audit_log(&self) -> Vec<PermissionAudit> {
        self.audit_log.read().clone()
    }

    /// Clear audit log
    pub fn clear_audit_log(&self) {
        self.audit_log.write().clear();
    }
}

impl Default for PermissionEnforcer {
    fn default() -> Self {
        Self::new()
    }
}

/// Permission audit entry
#[derive(Debug, Clone)]
pub struct PermissionAudit {
    /// Permission checked
    pub permission: PluginPermission,

    /// Whether it was granted
    pub granted: bool,

    /// Timestamp
    pub timestamp: Instant,
}

/// API access control
#[derive(Debug)]
pub struct ApiAccessControl {
    /// Allowed API endpoints
    allowed_apis: Arc<RwLock<HashSet<String>>>,

    /// API call rate limits (calls per second)
    rate_limits: Arc<RwLock<HashMap<String, RateLimit>>>,
}

impl ApiAccessControl {
    /// Create a new API access control
    pub fn new() -> Self {
        Self {
            allowed_apis: Arc::new(RwLock::new(HashSet::new())),
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Allow an API endpoint
    pub fn allow_api(&self, api: String) {
        self.allowed_apis.write().insert(api);
    }

    /// Deny an API endpoint
    pub fn deny_api(&self, api: &str) {
        self.allowed_apis.write().remove(api);
    }

    /// Set rate limit for an API
    pub fn set_rate_limit(&self, api: String, calls_per_second: u32) {
        self.rate_limits.write().insert(api, RateLimit::new(calls_per_second));
    }

    /// Check if API call is allowed
    pub fn check_api_access(&self, api: &str) -> Result<()> {
        // Check if API is allowed
        if !self.allowed_apis.read().contains(api) {
            return Err(MarketplaceError::PermissionDenied(
                format!("API access denied: {}", api)
            ));
        }

        // Check rate limit
        let mut rate_limits = self.rate_limits.write();
        if let Some(rate_limit) = rate_limits.get_mut(api) {
            if !rate_limit.allow_call() {
                return Err(MarketplaceError::ResourceLimitExceeded(
                    format!("Rate limit exceeded for API: {}", api)
                ));
            }
        }

        Ok(())
    }
}

impl Default for ApiAccessControl {
    fn default() -> Self {
        Self::new()
    }
}

use std::collections::HashMap;

/// Rate limiter
#[derive(Debug)]
struct RateLimit {
    /// Maximum calls per second
    max_calls_per_second: u32,

    /// Call timestamps within current window
    calls: Vec<Instant>,

    /// Window duration
    window: Duration,
}

impl RateLimit {
    /// Create a new rate limit
    fn new(max_calls_per_second: u32) -> Self {
        Self {
            max_calls_per_second,
            calls: Vec::new(),
            window: Duration::from_secs(1),
        }
    }

    /// Check if a call is allowed
    fn allow_call(&mut self) -> bool {
        let now = Instant::now();

        // Remove old calls outside the window
        self.calls.retain(|&timestamp| now.duration_since(timestamp) < self.window);

        // Check if we're under the limit
        if self.calls.len() < self.max_calls_per_second as usize {
            self.calls.push(now);
            true
        } else {
            false
        }
    }
}

/// Sandboxed plugin instance
#[derive(Debug)]
pub struct SandboxedPlugin {
    /// Plugin ID
    pub plugin_id: Uuid,

    /// Plugin manifest
    pub manifest: PluginManifest,

    /// Resource limits
    pub limits: ResourceLimits,

    /// Current resource usage
    pub usage: Arc<RwLock<ResourceUsage>>,

    /// Permission enforcer
    pub permissions: PermissionEnforcer,

    /// API access control
    pub api_access: ApiAccessControl,

    /// Sandbox root directory
    pub sandbox_root: PathBuf,
}

impl SandboxedPlugin {
    /// Create a new sandboxed plugin instance
    pub fn new(
        plugin_id: Uuid,
        manifest: PluginManifest,
        limits: ResourceLimits,
        sandbox_root: PathBuf,
    ) -> Result<Self> {
        // Create sandbox directory
        std::fs::create_dir_all(&sandbox_root)?;

        // Create permission enforcer with manifest permissions
        let permissions = PermissionEnforcer::new();
        for permission in &manifest.permissions {
            permissions.grant(*permission);
        }

        let mut usage = ResourceUsage::default();
        usage.started_at = Some(Instant::now());

        Ok(Self {
            plugin_id,
            manifest,
            limits,
            usage: Arc::new(RwLock::new(usage)),
            permissions,
            api_access: ApiAccessControl::new(),
            sandbox_root,
        })
    }

    /// Check resource limits
    pub fn check_resources(&self) -> Result<()> {
        let usage = self.usage.read();

        if let Some(error) = usage.exceeds_limits(&self.limits) {
            return Err(MarketplaceError::ResourceLimitExceeded(error));
        }

        Ok(())
    }

    /// Update resource usage
    pub fn update_usage<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut ResourceUsage),
    {
        let mut usage = self.usage.write();
        update_fn(&mut *usage);
    }

    /// Execute plugin in sandbox
    pub fn execute<F, R>(&self, operation: F) -> Result<R>
    where
        F: FnOnce() -> Result<R>,
    {
        // Check resources before execution
        self.check_resources()?;

        // Execute operation
        let result = operation()?;

        // Check resources after execution
        self.check_resources()?;

        Ok(result)
    }

    /// Get sandbox path for file
    pub fn sandbox_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.sandbox_root.join(path)
    }
}

/// Sandbox manager
#[derive(Debug)]
pub struct Sandbox {
    /// Active sandboxed plugins
    plugins: Arc<RwLock<HashMap<Uuid, Arc<SandboxedPlugin>>>>,

    /// Sandbox root directory
    root_dir: PathBuf,
}

impl Sandbox {
    /// Create a new sandbox manager
    pub fn new<P: AsRef<Path>>(root_dir: P) -> Result<Self> {
        let root_dir = root_dir.as_ref().to_path_buf();

        if !root_dir.exists() {
            std::fs::create_dir_all(&root_dir)?;
        }

        Ok(Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            root_dir,
        })
    }

    /// Create a sandboxed plugin instance
    pub fn create_sandbox(
        &self,
        plugin_id: Uuid,
        manifest: PluginManifest,
        limits: ResourceLimits,
    ) -> Result<Arc<SandboxedPlugin>> {
        let sandbox_root = self.root_dir.join(plugin_id.to_string());

        let sandboxed = Arc::new(SandboxedPlugin::new(
            plugin_id,
            manifest,
            limits,
            sandbox_root,
        )?);

        self.plugins.write().insert(plugin_id, sandboxed.clone());

        Ok(sandboxed)
    }

    /// Get sandboxed plugin
    pub fn get(&self, plugin_id: Uuid) -> Option<Arc<SandboxedPlugin>> {
        self.plugins.read().get(&plugin_id).cloned()
    }

    /// Remove sandboxed plugin
    pub fn remove(&self, plugin_id: Uuid) -> Result<()> {
        if let Some(sandboxed) = self.plugins.write().remove(&plugin_id) {
            // Clean up sandbox directory
            if sandboxed.sandbox_root.exists() {
                std::fs::remove_dir_all(&sandboxed.sandbox_root)?;
            }
        }

        Ok(())
    }

    /// List all active sandboxed plugins
    pub fn list(&self) -> Vec<Uuid> {
        self.plugins.read().keys().copied().collect()
    }

    /// Get total resource usage across all plugins
    pub fn total_usage(&self) -> ResourceUsage {
        let plugins = self.plugins.read();
        let mut total = ResourceUsage::default();

        for plugin in plugins.values() {
            let usage = plugin.usage.read();
            total.cpu_time_ms += usage.cpu_time_ms;
            total.memory_bytes += usage.memory_bytes;
            total.disk_bytes += usage.disk_bytes;
            total.network_bytes += usage.network_bytes;
            total.active_threads += usage.active_threads;
            total.open_file_handles += usage.open_file_handles;
        }

        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enterprise::marketplace::plugin::PluginManifest;

    #[test]
    fn test_resource_limits() {
        let limits = ResourceLimits::default();
        let mut usage = ResourceUsage::default();

        assert!(usage.exceeds_limits(&limits).is_none());

        usage.cpu_time_ms = limits.max_cpu_time_ms + 1;
        assert!(usage.exceeds_limits(&limits).is_some());
    }

    #[test]
    fn test_permission_enforcer() {
        let enforcer = PermissionEnforcer::new();

        assert!(enforcer.check(PluginPermission::ReadFileSystem).is_err());

        enforcer.grant(PluginPermission::ReadFileSystem);
        assert!(enforcer.check(PluginPermission::ReadFileSystem).is_ok());

        enforcer.revoke(PluginPermission::ReadFileSystem);
        assert!(enforcer.check(PluginPermission::ReadFileSystem).is_err());
    }

    #[test]
    fn test_rate_limit() {
        let mut rate_limit = RateLimit::new(2);

        assert!(rate_limit.allow_call());
        assert!(rate_limit.allow_call());
        assert!(!rate_limit.allow_call()); // Third call should be denied
    }

    #[test]
    fn test_api_access_control() {
        let api_access = ApiAccessControl::new();

        assert!(api_access.check_api_access("test_api").is_err());

        api_access.allow_api("test_api".to_string());
        assert!(api_access.check_api_access("test_api").is_ok());

        api_access.deny_api("test_api");
        assert!(api_access.check_api_access("test_api").is_err());
    }
}
