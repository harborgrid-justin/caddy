//! # Enterprise Plugin Marketplace
//!
//! A comprehensive plugin marketplace system for CADDY Enterprise, providing:
//! - Plugin discovery and installation
//! - Sandboxed execution environment
//! - Rating and review system
//! - Analytics and revenue tracking
//! - Enterprise-grade security and permissions
//!
//! ## Architecture
//!
//! The marketplace is composed of several key components:
//! - **Plugin**: Core plugin definitions and metadata
//! - **Registry**: Local and remote plugin registries
//! - **Installer**: Plugin download, verification, and installation
//! - **Sandbox**: Secure execution environment with resource limits
//! - **Store**: Marketplace catalog and pricing
//! - **Review**: Rating and review system
//! - **Analytics**: Usage tracking and developer metrics

pub mod analytics;
pub mod installer;
pub mod plugin;
pub mod registry;
pub mod review;
pub mod sandbox;
pub mod store;

// Re-export commonly used types
pub use analytics::{AnalyticsEvent, AnalyticsTracker, DeveloperDashboard, RevenueTracker};
pub use installer::{
    DependencyResolver, PluginDownloader, PluginInstaller, PluginVerifier, RollbackManager,
};
pub use plugin::{
    PluginCategory, PluginManifest, PluginMetadata, PluginPermission, PluginStatus,
};
pub use registry::{LocalRegistry, RegistryClient, RegistrySync};
pub use review::{
    AbuseReport, DeveloperResponse, Review, ReviewModeration, ReviewRating, ReviewSystem,
};
pub use sandbox::{
    ApiAccessControl, PermissionEnforcer, ResourceLimits, Sandbox, SandboxedPlugin,
};
pub use store::{PricingTier, StoreCatalog, StorePlugin};

use thiserror::Error;

/// Marketplace error types
#[derive(Debug, Error)]
pub enum MarketplaceError {
    /// Plugin not found
    #[error("Plugin not found: {0}")]
    PluginNotFound(String),

    /// Invalid plugin manifest
    #[error("Invalid plugin manifest: {0}")]
    InvalidManifest(String),

    /// Dependency resolution failed
    #[error("Dependency resolution failed: {0}")]
    DependencyResolutionFailed(String),

    /// Installation failed
    #[error("Installation failed: {0}")]
    InstallationFailed(String),

    /// Verification failed
    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Resource limit exceeded
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Registry error
    #[error("Registry error: {0}")]
    RegistryError(String),

    /// Sandbox error
    #[error("Sandbox error: {0}")]
    SandboxError(String),

    /// Review error
    #[error("Review error: {0}")]
    ReviewError(String),

    /// Analytics error
    #[error("Analytics error: {0}")]
    AnalyticsError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Version error
    #[error("Version error: {0}")]
    VersionError(String),
}

/// Marketplace result type
pub type Result<T> = std::result::Result<T, MarketplaceError>;
