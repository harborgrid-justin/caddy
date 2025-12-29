//! Plugin API surface and capabilities system
//!
//! Defines the API surface that plugins can interact with, including
//! capabilities, versioning, and safe access to CADDY functionality.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

use super::permissions::{Permission, PermissionSet};

/// Errors that can occur in plugin API operations
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid API version: {0}")]
    InvalidVersion(String),

    #[error("Capability not available: {0}")]
    CapabilityUnavailable(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("API call failed: {0}")]
    CallFailed(String),
}

pub type ApiResult<T> = Result<T, ApiError>;

/// API version following semantic versioning
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl ApiVersion {
    /// Current API version
    pub const CURRENT: ApiVersion = ApiVersion {
        major: 0,
        minor: 2,
        patch: 5,
    };

    /// Check if this version is compatible with another
    pub fn is_compatible_with(&self, other: &ApiVersion) -> bool {
        // Major versions must match
        if self.major != other.major {
            return false;
        }

        // Minor version must be >= required
        if self.minor < other.minor {
            return false;
        }

        true
    }

    /// Parse version from string (e.g., "0.2.5")
    pub fn parse(s: &str) -> Result<Self, ApiError> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(ApiError::InvalidVersion(s.to_string()));
        }

        Ok(ApiVersion {
            major: parts[0].parse().map_err(|_| ApiError::InvalidVersion(s.to_string()))?,
            minor: parts[1].parse().map_err(|_| ApiError::InvalidVersion(s.to_string()))?,
            patch: parts[2].parse().map_err(|_| ApiError::InvalidVersion(s.to_string()))?,
        })
    }
}

impl std::fmt::Display for ApiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Plugin API capability
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Capability {
    /// Access to geometry creation and manipulation
    Geometry,
    /// Access to rendering pipeline
    Rendering,
    /// Access to UI system
    UI,
    /// Access to file I/O operations
    FileIO,
    /// Access to command system
    Commands,
    /// Access to layer management
    Layers,
    /// Access to dimension tools
    Dimensions,
    /// Access to constraint solver
    Constraints,
    /// Access to network operations
    Network,
    /// Access to clipboard operations
    Clipboard,
    /// Access to system notifications
    Notifications,
    /// Access to database operations
    Database,
    /// Access to cryptographic operations
    Crypto,
    /// Access to enterprise features
    Enterprise,
    /// Custom capability (for future extensions)
    Custom(String),
}

/// Plugin API context - provides access to CADDY functionality
#[derive(Clone)]
pub struct PluginApi {
    version: ApiVersion,
    permissions: Arc<PermissionSet>,
    capabilities: Arc<HashMap<Capability, Box<dyn ApiCapability>>>,
}

impl PluginApi {
    /// Create a new plugin API context
    pub fn new(permissions: PermissionSet) -> Self {
        Self {
            version: ApiVersion::CURRENT,
            permissions: Arc::new(permissions),
            capabilities: Arc::new(HashMap::new()),
        }
    }

    /// Get API version
    pub fn version(&self) -> &ApiVersion {
        &self.version
    }

    /// Check if a permission is granted
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.has(permission)
    }

    /// Require a specific permission
    pub fn require_permission(&self, permission: &Permission) -> ApiResult<()> {
        if self.has_permission(permission) {
            Ok(())
        } else {
            Err(ApiError::PermissionDenied(format!("{:?}", permission)))
        }
    }

    /// Check if a capability is available
    pub fn has_capability(&self, capability: &Capability) -> bool {
        self.capabilities.contains_key(capability)
    }

    /// Get geometry API
    pub fn geometry(&self) -> ApiResult<GeometryApi> {
        self.require_permission(&Permission::GeometryRead)?;
        Ok(GeometryApi {
            api: self.clone(),
        })
    }

    /// Get rendering API
    pub fn rendering(&self) -> ApiResult<RenderingApi> {
        self.require_permission(&Permission::RenderingRead)?;
        Ok(RenderingApi {
            api: self.clone(),
        })
    }

    /// Get UI API
    pub fn ui(&self) -> ApiResult<UiApi> {
        self.require_permission(&Permission::UIRead)?;
        Ok(UiApi {
            api: self.clone(),
        })
    }

    /// Get file I/O API
    pub fn file_io(&self) -> ApiResult<FileIoApi> {
        self.require_permission(&Permission::FileRead)?;
        Ok(FileIoApi {
            api: self.clone(),
        })
    }

    /// Get command API
    pub fn commands(&self) -> ApiResult<CommandApi> {
        self.require_permission(&Permission::CommandExecute)?;
        Ok(CommandApi {
            api: self.clone(),
        })
    }

    /// Get network API
    pub fn network(&self) -> ApiResult<NetworkApi> {
        self.require_permission(&Permission::NetworkAccess)?;
        Ok(NetworkApi {
            api: self.clone(),
        })
    }
}

/// Trait for API capabilities
pub trait ApiCapability: Send + Sync {
    /// Get capability name
    fn name(&self) -> &str;

    /// Check if capability is available
    fn is_available(&self) -> bool;
}

/// Geometry manipulation API
#[derive(Clone)]
pub struct GeometryApi {
    api: PluginApi,
}

impl GeometryApi {
    /// Create a new entity
    pub fn create_entity(&self, entity_type: &str, params: HashMap<String, serde_json::Value>) -> ApiResult<String> {
        self.api.require_permission(&Permission::GeometryWrite)?;
        // Implementation would interact with CADDY's geometry system
        Ok(format!("entity_{}", uuid::Uuid::new_v4()))
    }

    /// Get entity by ID
    pub fn get_entity(&self, id: &str) -> ApiResult<serde_json::Value> {
        self.api.require_permission(&Permission::GeometryRead)?;
        // Implementation would fetch from CADDY's entity system
        Ok(serde_json::json!({
            "id": id,
            "type": "unknown"
        }))
    }

    /// Update entity
    pub fn update_entity(&self, id: &str, updates: HashMap<String, serde_json::Value>) -> ApiResult<()> {
        self.api.require_permission(&Permission::GeometryWrite)?;
        // Implementation would update CADDY's entity
        Ok(())
    }

    /// Delete entity
    pub fn delete_entity(&self, id: &str) -> ApiResult<()> {
        self.api.require_permission(&Permission::GeometryWrite)?;
        // Implementation would delete from CADDY's entity system
        Ok(())
    }
}

/// Rendering API
#[derive(Clone)]
pub struct RenderingApi {
    api: PluginApi,
}

impl RenderingApi {
    /// Request a render update
    pub fn request_render(&self) -> ApiResult<()> {
        self.api.require_permission(&Permission::RenderingWrite)?;
        // Implementation would trigger CADDY's rendering system
        Ok(())
    }

    /// Set render mode
    pub fn set_render_mode(&self, mode: &str) -> ApiResult<()> {
        self.api.require_permission(&Permission::RenderingWrite)?;
        // Implementation would change rendering mode
        Ok(())
    }

    /// Get viewport information
    pub fn get_viewport(&self) -> ApiResult<serde_json::Value> {
        self.api.require_permission(&Permission::RenderingRead)?;
        Ok(serde_json::json!({
            "width": 1920,
            "height": 1080,
            "scale": 1.0
        }))
    }
}

/// UI API
#[derive(Clone)]
pub struct UiApi {
    api: PluginApi,
}

impl UiApi {
    /// Show notification
    pub fn show_notification(&self, title: &str, message: &str, level: NotificationLevel) -> ApiResult<()> {
        self.api.require_permission(&Permission::UIWrite)?;
        log::info!("Plugin notification: {} - {}", title, message);
        Ok(())
    }

    /// Show dialog
    pub fn show_dialog(&self, config: DialogConfig) -> ApiResult<DialogResult> {
        self.api.require_permission(&Permission::UIWrite)?;
        // Implementation would show dialog in CADDY's UI
        Ok(DialogResult::Ok)
    }

    /// Register menu item
    pub fn register_menu_item(&self, path: &str, label: &str, callback_id: &str) -> ApiResult<()> {
        self.api.require_permission(&Permission::UIWrite)?;
        // Implementation would add menu item to CADDY's UI
        Ok(())
    }

    /// Register toolbar button
    pub fn register_toolbar_button(&self, config: ToolbarButtonConfig) -> ApiResult<String> {
        self.api.require_permission(&Permission::UIWrite)?;
        Ok(format!("toolbar_button_{}", uuid::Uuid::new_v4()))
    }
}

/// File I/O API
#[derive(Clone)]
pub struct FileIoApi {
    api: PluginApi,
}

impl FileIoApi {
    /// Read file
    pub async fn read_file(&self, path: &str) -> ApiResult<Vec<u8>> {
        self.api.require_permission(&Permission::FileRead)?;
        tokio::fs::read(path)
            .await
            .map_err(|e| ApiError::CallFailed(e.to_string()))
    }

    /// Write file
    pub async fn write_file(&self, path: &str, data: &[u8]) -> ApiResult<()> {
        self.api.require_permission(&Permission::FileWrite)?;
        tokio::fs::write(path, data)
            .await
            .map_err(|e| ApiError::CallFailed(e.to_string()))
    }

    /// List directory
    pub async fn list_directory(&self, path: &str) -> ApiResult<Vec<String>> {
        self.api.require_permission(&Permission::FileRead)?;
        let mut entries = Vec::new();
        let mut dir = tokio::fs::read_dir(path)
            .await
            .map_err(|e| ApiError::CallFailed(e.to_string()))?;

        while let Some(entry) = dir.next_entry()
            .await
            .map_err(|e| ApiError::CallFailed(e.to_string()))? {
            if let Some(name) = entry.file_name().to_str() {
                entries.push(name.to_string());
            }
        }

        Ok(entries)
    }
}

/// Command API
#[derive(Clone)]
pub struct CommandApi {
    api: PluginApi,
}

impl CommandApi {
    /// Execute a command
    pub fn execute(&self, command: &str, params: HashMap<String, serde_json::Value>) -> ApiResult<serde_json::Value> {
        self.api.require_permission(&Permission::CommandExecute)?;
        // Implementation would execute command in CADDY's command system
        Ok(serde_json::json!({"status": "success"}))
    }

    /// Register a new command
    pub fn register(&self, name: &str, description: &str, callback_id: &str) -> ApiResult<()> {
        self.api.require_permission(&Permission::CommandExecute)?;
        // Implementation would register command in CADDY's command system
        Ok(())
    }
}

/// Network API
#[derive(Clone)]
pub struct NetworkApi {
    api: PluginApi,
}

impl NetworkApi {
    /// Make HTTP request
    pub async fn http_request(&self, config: HttpRequestConfig) -> ApiResult<HttpResponse> {
        self.api.require_permission(&Permission::NetworkAccess)?;

        let client = reqwest::Client::new();
        let mut request = match config.method.as_str() {
            "GET" => client.get(&config.url),
            "POST" => client.post(&config.url),
            "PUT" => client.put(&config.url),
            "DELETE" => client.delete(&config.url),
            _ => return Err(ApiError::InvalidParameter(format!("Invalid HTTP method: {}", config.method))),
        };

        if let Some(headers) = config.headers {
            for (key, value) in headers {
                request = request.header(key, value);
            }
        }

        if let Some(body) = config.body {
            request = request.body(body);
        }

        let response = request
            .send()
            .await
            .map_err(|e| ApiError::CallFailed(e.to_string()))?;

        let status = response.status().as_u16();
        let body = response.bytes()
            .await
            .map_err(|e| ApiError::CallFailed(e.to_string()))?
            .to_vec();

        Ok(HttpResponse { status, body })
    }
}

/// Notification levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Success,
}

/// Dialog configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogConfig {
    pub title: String,
    pub message: String,
    pub dialog_type: DialogType,
    pub buttons: Vec<String>,
}

/// Dialog types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DialogType {
    Info,
    Warning,
    Error,
    Question,
}

/// Dialog result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DialogResult {
    Ok,
    Cancel,
    Custom(String),
}

/// Toolbar button configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolbarButtonConfig {
    pub label: String,
    pub icon: Option<String>,
    pub tooltip: Option<String>,
    pub callback_id: String,
}

/// HTTP request configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequestConfig {
    pub url: String,
    pub method: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<Vec<u8>>,
    pub timeout_ms: Option<u64>,
}

/// HTTP response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_version_compatibility() {
        let v1 = ApiVersion { major: 0, minor: 2, patch: 5 };
        let v2 = ApiVersion { major: 0, minor: 2, patch: 0 };
        let v3 = ApiVersion { major: 0, minor: 3, patch: 0 };
        let v4 = ApiVersion { major: 1, minor: 0, patch: 0 };

        assert!(v1.is_compatible_with(&v2));
        assert!(!v2.is_compatible_with(&v3));
        assert!(!v1.is_compatible_with(&v4));
    }

    #[test]
    fn test_api_version_parse() {
        let version = ApiVersion::parse("0.2.5").unwrap();
        assert_eq!(version.major, 0);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 5);
    }
}
