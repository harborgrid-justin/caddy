//! Plugin definitions and metadata
//!
//! This module defines the core plugin types, including manifests, metadata,
//! categories, and status tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Plugin manifest containing metadata and requirements
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginManifest {
    /// Unique plugin identifier
    pub id: Uuid,

    /// Plugin name (must be unique in registry)
    pub name: String,

    /// Semantic version (e.g., "1.2.3")
    pub version: String,

    /// Plugin author/organization
    pub author: String,

    /// Author email
    pub email: String,

    /// Short description
    pub description: String,

    /// Detailed description (markdown supported)
    pub long_description: Option<String>,

    /// Plugin homepage URL
    pub homepage: Option<String>,

    /// Repository URL
    pub repository: Option<String>,

    /// License identifier (SPDX format)
    pub license: String,

    /// Keywords for search
    pub keywords: Vec<String>,

    /// Plugin category
    pub category: PluginCategory,

    /// Additional categories
    pub categories: Vec<PluginCategory>,

    /// Dependencies with version requirements
    pub dependencies: HashMap<String, String>,

    /// Required CADDY version
    pub caddy_version: String,

    /// Required permissions
    pub permissions: HashSet<PluginPermission>,

    /// Minimum system requirements
    pub system_requirements: SystemRequirements,

    /// Entry point (main module/class)
    pub entry_point: String,

    /// Plugin assets (icons, screenshots, etc.)
    pub assets: PluginAssets,

    /// Plugin signature for verification
    pub signature: Option<String>,

    /// Checksum (SHA-256)
    pub checksum: String,

    /// Plugin size in bytes
    pub size: u64,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl PluginManifest {
    /// Create a new plugin manifest
    pub fn new(name: String, version: String, author: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            version,
            author,
            email: String::new(),
            description: String::new(),
            long_description: None,
            homepage: None,
            repository: None,
            license: "MIT".to_string(),
            keywords: Vec::new(),
            category: PluginCategory::Utility,
            categories: Vec::new(),
            dependencies: HashMap::new(),
            caddy_version: ">=0.1.0".to_string(),
            permissions: HashSet::new(),
            system_requirements: SystemRequirements::default(),
            entry_point: "main".to_string(),
            assets: PluginAssets::default(),
            signature: None,
            checksum: String::new(),
            size: 0,
            created_at: now,
            updated_at: now,
        }
    }

    /// Validate the manifest
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Plugin name cannot be empty".to_string());
        }

        if self.version.is_empty() {
            return Err("Plugin version cannot be empty".to_string());
        }

        if self.author.is_empty() {
            return Err("Plugin author cannot be empty".to_string());
        }

        if self.license.is_empty() {
            return Err("Plugin license cannot be empty".to_string());
        }

        // Validate version format (basic semver check)
        if !self.is_valid_semver(&self.version) {
            return Err(format!("Invalid version format: {}", self.version));
        }

        Ok(())
    }

    /// Check if a version string is valid semver
    fn is_valid_semver(&self, version: &str) -> bool {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return false;
        }

        parts.iter().all(|p| p.parse::<u32>().is_ok())
    }
}

/// Plugin metadata including ratings and statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginMetadata {
    /// Plugin manifest
    pub manifest: PluginManifest,

    /// Current status
    pub status: PluginStatus,

    /// Average rating (0.0 - 5.0)
    pub average_rating: f32,

    /// Total number of ratings
    pub rating_count: u32,

    /// Total downloads
    pub download_count: u64,

    /// Active installations
    pub active_installs: u64,

    /// Number of reviews
    pub review_count: u32,

    /// Featured plugin flag
    pub featured: bool,

    /// Editor's choice flag
    pub editors_choice: bool,

    /// Verified developer flag
    pub verified_developer: bool,

    /// Last verified timestamp
    pub last_verified: Option<DateTime<Utc>>,

    /// Plugin tags
    pub tags: Vec<String>,

    /// Compatibility information
    pub compatibility: CompatibilityInfo,

    /// Support information
    pub support: SupportInfo,
}

impl PluginMetadata {
    /// Create new metadata from manifest
    pub fn from_manifest(manifest: PluginManifest) -> Self {
        Self {
            manifest,
            status: PluginStatus::Draft,
            average_rating: 0.0,
            rating_count: 0,
            download_count: 0,
            active_installs: 0,
            review_count: 0,
            featured: false,
            editors_choice: false,
            verified_developer: false,
            last_verified: None,
            tags: Vec::new(),
            compatibility: CompatibilityInfo::default(),
            support: SupportInfo::default(),
        }
    }

    /// Update rating statistics
    pub fn update_rating(&mut self, new_rating: f32) {
        let total = self.average_rating * self.rating_count as f32 + new_rating;
        self.rating_count += 1;
        self.average_rating = total / self.rating_count as f32;
    }

    /// Increment download count
    pub fn increment_downloads(&mut self) {
        self.download_count += 1;
    }
}

/// Plugin category
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PluginCategory {
    /// 2D drawing tools
    Drawing2D,

    /// 3D modeling tools
    Modeling3D,

    /// Rendering and visualization
    Rendering,

    /// File import/export
    FileIO,

    /// Analysis and simulation
    Analysis,

    /// Automation and scripting
    Automation,

    /// Collaboration tools
    Collaboration,

    /// CAM and manufacturing
    Manufacturing,

    /// Documentation and reporting
    Documentation,

    /// UI enhancements
    UserInterface,

    /// Data management
    DataManagement,

    /// Integration with other tools
    Integration,

    /// Utilities and helpers
    Utility,

    /// Education and training
    Education,

    /// Custom category
    Custom,
}

impl PluginCategory {
    /// Get category display name
    pub fn display_name(&self) -> &str {
        match self {
            Self::Drawing2D => "2D Drawing",
            Self::Modeling3D => "3D Modeling",
            Self::Rendering => "Rendering & Visualization",
            Self::FileIO => "File Import/Export",
            Self::Analysis => "Analysis & Simulation",
            Self::Automation => "Automation & Scripting",
            Self::Collaboration => "Collaboration",
            Self::Manufacturing => "CAM & Manufacturing",
            Self::Documentation => "Documentation & Reporting",
            Self::UserInterface => "UI Enhancements",
            Self::DataManagement => "Data Management",
            Self::Integration => "Integration",
            Self::Utility => "Utilities",
            Self::Education => "Education & Training",
            Self::Custom => "Custom",
        }
    }
}

/// Plugin status in the marketplace
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginStatus {
    /// Draft (not published)
    Draft,

    /// Under review
    UnderReview,

    /// Published and available
    Published,

    /// Deprecated (no longer maintained)
    Deprecated,

    /// Suspended (temporarily unavailable)
    Suspended,

    /// Banned (security/policy violation)
    Banned,

    /// Archived (historical)
    Archived,
}

impl PluginStatus {
    /// Check if plugin is available for download
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Published)
    }

    /// Check if plugin is visible in store
    pub fn is_visible(&self) -> bool {
        matches!(self, Self::Published | Self::Deprecated)
    }
}

/// Plugin permissions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PluginPermission {
    /// Read file system
    ReadFileSystem,

    /// Write file system
    WriteFileSystem,

    /// Network access
    NetworkAccess,

    /// Execute external processes
    ExecuteProcess,

    /// Access user data
    AccessUserData,

    /// Modify CAD documents
    ModifyDocuments,

    /// Access system clipboard
    AccessClipboard,

    /// Camera/microphone access
    MediaAccess,

    /// Database access
    DatabaseAccess,

    /// Registry access (Windows)
    RegistryAccess,

    /// Native UI access
    NativeUI,

    /// GPU access
    GpuAccess,
}

impl PluginPermission {
    /// Get permission display name
    pub fn display_name(&self) -> &str {
        match self {
            Self::ReadFileSystem => "Read files",
            Self::WriteFileSystem => "Write files",
            Self::NetworkAccess => "Network access",
            Self::ExecuteProcess => "Execute processes",
            Self::AccessUserData => "Access user data",
            Self::ModifyDocuments => "Modify documents",
            Self::AccessClipboard => "Access clipboard",
            Self::MediaAccess => "Camera/Microphone",
            Self::DatabaseAccess => "Database access",
            Self::RegistryAccess => "Registry access",
            Self::NativeUI => "Native UI",
            Self::GpuAccess => "GPU access",
        }
    }

    /// Get permission description
    pub fn description(&self) -> &str {
        match self {
            Self::ReadFileSystem => "Read files and directories on your computer",
            Self::WriteFileSystem => "Write and modify files on your computer",
            Self::NetworkAccess => "Connect to the internet",
            Self::ExecuteProcess => "Run external programs",
            Self::AccessUserData => "Access your personal data and preferences",
            Self::ModifyDocuments => "Edit and modify CAD documents",
            Self::AccessClipboard => "Read and write clipboard contents",
            Self::MediaAccess => "Access camera and microphone",
            Self::DatabaseAccess => "Connect to databases",
            Self::RegistryAccess => "Modify Windows registry",
            Self::NativeUI => "Create native UI windows",
            Self::GpuAccess => "Direct GPU access",
        }
    }
}

/// System requirements
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemRequirements {
    /// Minimum CPU cores
    pub min_cpu_cores: u32,

    /// Minimum RAM in MB
    pub min_ram_mb: u64,

    /// Minimum disk space in MB
    pub min_disk_mb: u64,

    /// GPU required
    pub gpu_required: bool,

    /// Supported platforms
    pub platforms: Vec<String>,

    /// Minimum OS versions
    pub min_os_versions: HashMap<String, String>,
}

impl Default for SystemRequirements {
    fn default() -> Self {
        Self {
            min_cpu_cores: 1,
            min_ram_mb: 256,
            min_disk_mb: 100,
            gpu_required: false,
            platforms: vec!["linux".to_string(), "windows".to_string(), "macos".to_string()],
            min_os_versions: HashMap::new(),
        }
    }
}

/// Plugin assets
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PluginAssets {
    /// Icon URL (256x256)
    pub icon: Option<String>,

    /// Screenshot URLs
    pub screenshots: Vec<String>,

    /// Video demo URL
    pub video: Option<String>,

    /// Documentation URL
    pub documentation: Option<String>,

    /// Banner image URL
    pub banner: Option<String>,
}

/// Compatibility information
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CompatibilityInfo {
    /// Compatible CADDY versions
    pub caddy_versions: Vec<String>,

    /// Compatible with other plugins
    pub compatible_plugins: Vec<String>,

    /// Known incompatibilities
    pub incompatible_plugins: Vec<String>,

    /// Platform-specific notes
    pub platform_notes: HashMap<String, String>,
}

/// Support information
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SupportInfo {
    /// Support email
    pub email: Option<String>,

    /// Support URL/forum
    pub url: Option<String>,

    /// Documentation URL
    pub documentation: Option<String>,

    /// FAQ URL
    pub faq: Option<String>,

    /// Issue tracker URL
    pub issue_tracker: Option<String>,

    /// Response time (in hours)
    pub response_time_hours: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manifest_creation() {
        let manifest = PluginManifest::new(
            "test-plugin".to_string(),
            "1.0.0".to_string(),
            "Test Author".to_string(),
        );

        assert_eq!(manifest.name, "test-plugin");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.author, "Test Author");
    }

    #[test]
    fn test_manifest_validation() {
        let manifest = PluginManifest::new(
            "test-plugin".to_string(),
            "1.0.0".to_string(),
            "Test Author".to_string(),
        );

        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_invalid_semver() {
        let mut manifest = PluginManifest::new(
            "test-plugin".to_string(),
            "invalid".to_string(),
            "Test Author".to_string(),
        );

        assert!(manifest.validate().is_err());

        manifest.version = "1.0".to_string();
        assert!(manifest.validate().is_err());

        manifest.version = "1.0.0".to_string();
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_plugin_status() {
        assert!(PluginStatus::Published.is_available());
        assert!(!PluginStatus::Draft.is_available());
        assert!(!PluginStatus::Banned.is_available());

        assert!(PluginStatus::Published.is_visible());
        assert!(PluginStatus::Deprecated.is_visible());
        assert!(!PluginStatus::Banned.is_visible());
    }

    #[test]
    fn test_metadata_rating_update() {
        let manifest = PluginManifest::new(
            "test".to_string(),
            "1.0.0".to_string(),
            "author".to_string(),
        );
        let mut metadata = PluginMetadata::from_manifest(manifest);

        metadata.update_rating(5.0);
        assert_eq!(metadata.average_rating, 5.0);
        assert_eq!(metadata.rating_count, 1);

        metadata.update_rating(3.0);
        assert_eq!(metadata.average_rating, 4.0);
        assert_eq!(metadata.rating_count, 2);
    }
}
