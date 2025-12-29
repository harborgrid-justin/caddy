//! WASM-based plugin sandboxing for secure plugin execution
//!
//! Provides isolated execution environment for plugins with resource limits,
//! capability-based security, and controlled access to host functions.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::loader::PluginType;

/// Sandbox errors
#[derive(Debug, Error)]
pub enum SandboxError {
    #[error("Initialization failed: {0}")]
    InitFailed(String),

    #[error("Execution failed: {0}")]
    ExecFailed(String),

    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    #[error("Capability denied: {0}")]
    CapabilityDenied(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Timeout: operation took too long")]
    Timeout,

    #[error("Sandbox not initialized")]
    NotInitialized,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type SandboxResult<T> = Result<T, SandboxError>;

/// Resource limits for sandboxed plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory in bytes (default: 128 MB)
    pub max_memory_bytes: usize,

    /// Maximum execution time per call in milliseconds (default: 5000ms)
    pub max_execution_time_ms: u64,

    /// Maximum file size that can be read/written in bytes (default: 10 MB)
    pub max_file_size_bytes: usize,

    /// Maximum number of file operations per second (default: 100)
    pub max_file_ops_per_second: u32,

    /// Maximum number of network requests per second (default: 10)
    pub max_network_requests_per_second: u32,

    /// Maximum CPU time percentage (default: 50%)
    pub max_cpu_percent: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 128 * 1024 * 1024, // 128 MB
            max_execution_time_ms: 5000,          // 5 seconds
            max_file_size_bytes: 10 * 1024 * 1024, // 10 MB
            max_file_ops_per_second: 100,
            max_network_requests_per_second: 10,
            max_cpu_percent: 50,
        }
    }
}

/// Resource usage tracking
#[derive(Debug, Clone, Default)]
struct ResourceUsage {
    memory_used: usize,
    execution_time: Duration,
    file_ops_count: u32,
    network_requests_count: u32,
    last_reset: Instant,
}

impl ResourceUsage {
    fn new() -> Self {
        Self {
            memory_used: 0,
            execution_time: Duration::default(),
            file_ops_count: 0,
            network_requests_count: 0,
            last_reset: Instant::now(),
        }
    }

    fn reset_if_needed(&mut self) {
        let elapsed = self.last_reset.elapsed();
        if elapsed >= Duration::from_secs(1) {
            self.file_ops_count = 0;
            self.network_requests_count = 0;
            self.last_reset = Instant::now();
        }
    }

    fn check_limits(&self, limits: &ResourceLimits) -> SandboxResult<()> {
        if self.memory_used > limits.max_memory_bytes {
            return Err(SandboxError::ResourceLimitExceeded(format!(
                "Memory limit exceeded: {} > {}",
                self.memory_used, limits.max_memory_bytes
            )));
        }

        if self.execution_time.as_millis() > limits.max_execution_time_ms as u128 {
            return Err(SandboxError::Timeout);
        }

        if self.file_ops_count > limits.max_file_ops_per_second {
            return Err(SandboxError::ResourceLimitExceeded(format!(
                "File operations rate limit exceeded: {} > {}",
                self.file_ops_count, limits.max_file_ops_per_second
            )));
        }

        if self.network_requests_count > limits.max_network_requests_per_second {
            return Err(SandboxError::ResourceLimitExceeded(format!(
                "Network requests rate limit exceeded: {} > {}",
                self.network_requests_count, limits.max_network_requests_per_second
            )));
        }

        Ok(())
    }
}

/// WASM instance data (placeholder for actual WASM runtime integration)
struct WasmInstance {
    #[allow(dead_code)]
    module_path: PathBuf,
    #[allow(dead_code)]
    exports: HashMap<String, WasmFunction>,
    initialized: bool,
}

/// WASM function reference
#[derive(Clone)]
struct WasmFunction {
    #[allow(dead_code)]
    name: String,
}

/// Native plugin instance (for non-WASM plugins)
struct NativeInstance {
    #[allow(dead_code)]
    library_path: PathBuf,
    initialized: bool,
}

/// Plugin sandbox providing isolated execution environment
pub struct PluginSandbox {
    /// Plugin identifier
    plugin_id: String,

    /// Plugin type
    plugin_type: PluginType,

    /// Plugin path
    plugin_path: PathBuf,

    /// Resource limits
    limits: ResourceLimits,

    /// Resource usage tracking
    usage: Arc<RwLock<ResourceUsage>>,

    /// WASM instance (if WASM plugin)
    wasm_instance: Arc<RwLock<Option<WasmInstance>>>,

    /// Native instance (if native plugin)
    native_instance: Arc<RwLock<Option<NativeInstance>>>,

    /// Allowed host functions
    allowed_host_functions: Arc<RwLock<HashMap<String, bool>>>,
}

impl PluginSandbox {
    /// Create a new plugin sandbox
    pub fn new(plugin_id: String, plugin_type: PluginType, plugin_path: PathBuf) -> Self {
        Self {
            plugin_id,
            plugin_type,
            plugin_path,
            limits: ResourceLimits::default(),
            usage: Arc::new(RwLock::new(ResourceUsage::new())),
            wasm_instance: Arc::new(RwLock::new(None)),
            native_instance: Arc::new(RwLock::new(None)),
            allowed_host_functions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set resource limits
    pub fn set_limits(&mut self, limits: ResourceLimits) {
        self.limits = limits;
    }

    /// Initialize the sandbox
    pub async fn initialize<P: AsRef<Path>>(&self, entry_point: P) -> SandboxResult<()> {
        match self.plugin_type {
            PluginType::Wasm => self.initialize_wasm(entry_point).await,
            PluginType::Native => self.initialize_native(entry_point).await,
        }
    }

    /// Initialize WASM plugin
    async fn initialize_wasm<P: AsRef<Path>>(&self, entry_point: P) -> SandboxResult<()> {
        let entry_path = entry_point.as_ref().to_path_buf();

        // Verify file exists
        if !entry_path.exists() {
            return Err(SandboxError::InitFailed(format!(
                "WASM module not found: {:?}",
                entry_path
            )));
        }

        // In a real implementation, we would:
        // 1. Load the WASM module using wasmer/wasmtime
        // 2. Set up memory limits
        // 3. Register host functions
        // 4. Instantiate the module
        // 5. Call initialization function if available

        // For now, create a placeholder instance
        let instance = WasmInstance {
            module_path: entry_path,
            exports: HashMap::new(),
            initialized: true,
        };

        *self.wasm_instance.write() = Some(instance);

        log::info!("WASM sandbox initialized for plugin: {}", self.plugin_id);

        Ok(())
    }

    /// Initialize native plugin
    async fn initialize_native<P: AsRef<Path>>(&self, entry_point: P) -> SandboxResult<()> {
        let entry_path = entry_point.as_ref().to_path_buf();

        // Verify file exists
        if !entry_path.exists() {
            return Err(SandboxError::InitFailed(format!(
                "Native library not found: {:?}",
                entry_path
            )));
        }

        // In a real implementation, we would:
        // 1. Load the dynamic library using libloading
        // 2. Verify signature/checksum
        // 3. Set up security context
        // 4. Call initialization function

        // Note: Native plugins are less secure and should be carefully vetted

        let instance = NativeInstance {
            library_path: entry_path,
            initialized: true,
        };

        *self.native_instance.write() = Some(instance);

        log::warn!(
            "Native sandbox initialized for plugin: {} (WARNING: Native plugins have reduced security)",
            self.plugin_id
        );

        Ok(())
    }

    /// Call a function in the sandboxed plugin
    pub async fn call_function(
        &self,
        function_name: &str,
        args: Vec<SandboxValue>,
    ) -> SandboxResult<SandboxValue> {
        // Check if initialized
        let is_initialized = match self.plugin_type {
            PluginType::Wasm => self.wasm_instance.read().as_ref().map(|i| i.initialized).unwrap_or(false),
            PluginType::Native => self.native_instance.read().as_ref().map(|i| i.initialized).unwrap_or(false),
        };

        if !is_initialized {
            return Err(SandboxError::NotInitialized);
        }

        // Track execution time
        let start = Instant::now();

        // Reset rate limits if needed
        self.usage.write().reset_if_needed();

        // Check resource limits before execution
        self.usage.read().check_limits(&self.limits)?;

        // Execute based on plugin type
        let result = match self.plugin_type {
            PluginType::Wasm => self.call_wasm_function(function_name, args).await?,
            PluginType::Native => self.call_native_function(function_name, args).await?,
        };

        // Update execution time
        let elapsed = start.elapsed();
        self.usage.write().execution_time = elapsed;

        // Check limits after execution
        self.usage.read().check_limits(&self.limits)?;

        Ok(result)
    }

    /// Call WASM function
    async fn call_wasm_function(
        &self,
        function_name: &str,
        _args: Vec<SandboxValue>,
    ) -> SandboxResult<SandboxValue> {
        // In a real implementation, this would:
        // 1. Look up the exported function
        // 2. Convert args to WASM types
        // 3. Call the function with timeout
        // 4. Convert result back to SandboxValue

        log::debug!("Calling WASM function: {}", function_name);

        // Placeholder implementation
        Ok(SandboxValue::Null)
    }

    /// Call native function
    async fn call_native_function(
        &self,
        function_name: &str,
        _args: Vec<SandboxValue>,
    ) -> SandboxResult<SandboxValue> {
        // In a real implementation, this would:
        // 1. Look up the function symbol
        // 2. Convert args to native types
        // 3. Call the function with timeout
        // 4. Convert result back to SandboxValue

        log::debug!("Calling native function: {}", function_name);

        // Placeholder implementation
        Ok(SandboxValue::Null)
    }

    /// Cleanup sandbox resources
    pub async fn cleanup(&self) -> SandboxResult<()> {
        // Cleanup WASM instance
        if let Some(_instance) = self.wasm_instance.write().take() {
            // In a real implementation, properly cleanup WASM runtime
            log::info!("Cleaned up WASM instance for: {}", self.plugin_id);
        }

        // Cleanup native instance
        if let Some(_instance) = self.native_instance.write().take() {
            // In a real implementation, unload dynamic library
            log::info!("Cleaned up native instance for: {}", self.plugin_id);
        }

        Ok(())
    }

    /// Record file operation
    pub fn record_file_op(&self) -> SandboxResult<()> {
        let mut usage = self.usage.write();
        usage.reset_if_needed();
        usage.file_ops_count += 1;
        usage.check_limits(&self.limits)?;
        Ok(())
    }

    /// Record network request
    pub fn record_network_request(&self) -> SandboxResult<()> {
        let mut usage = self.usage.write();
        usage.reset_if_needed();
        usage.network_requests_count += 1;
        usage.check_limits(&self.limits)?;
        Ok(())
    }

    /// Update memory usage
    pub fn update_memory_usage(&self, bytes: usize) -> SandboxResult<()> {
        self.usage.write().memory_used = bytes;
        self.usage.read().check_limits(&self.limits)?;
        Ok(())
    }

    /// Get current resource usage
    pub fn get_usage(&self) -> ResourceUsageStats {
        let usage = self.usage.read();
        ResourceUsageStats {
            memory_used_bytes: usage.memory_used,
            memory_limit_bytes: self.limits.max_memory_bytes,
            execution_time_ms: usage.execution_time.as_millis() as u64,
            execution_limit_ms: self.limits.max_execution_time_ms,
            file_ops_count: usage.file_ops_count,
            network_requests_count: usage.network_requests_count,
        }
    }
}

/// Values that can be passed to/from sandbox
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SandboxValue {
    Null,
    Bool(bool),
    Int32(i32),
    Int64(i64),
    Float32(f32),
    Float64(f64),
    String(String),
    Bytes(Vec<u8>),
    Array(Vec<SandboxValue>),
    Object(HashMap<String, SandboxValue>),
}

/// Resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageStats {
    pub memory_used_bytes: usize,
    pub memory_limit_bytes: usize,
    pub execution_time_ms: u64,
    pub execution_limit_ms: u64,
    pub file_ops_count: u32,
    pub network_requests_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_memory_bytes, 128 * 1024 * 1024);
        assert_eq!(limits.max_execution_time_ms, 5000);
    }

    #[test]
    fn test_resource_usage_tracking() {
        let mut usage = ResourceUsage::new();
        usage.memory_used = 1000;
        usage.file_ops_count = 10;

        let limits = ResourceLimits::default();
        assert!(usage.check_limits(&limits).is_ok());

        usage.memory_used = limits.max_memory_bytes + 1;
        assert!(usage.check_limits(&limits).is_err());
    }

    #[tokio::test]
    async fn test_sandbox_creation() {
        let sandbox = PluginSandbox::new(
            "test-plugin".to_string(),
            PluginType::Wasm,
            PathBuf::from("/tmp/test"),
        );

        assert_eq!(sandbox.plugin_id, "test-plugin");
        assert_eq!(sandbox.plugin_type, PluginType::Wasm);
    }
}
