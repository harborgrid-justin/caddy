//! # CI/CD Integration System
//!
//! Enterprise-grade CI/CD integrations for CADDY v0.3.0
//!
//! This module provides comprehensive integrations with major CI/CD platforms:
//! - GitHub Actions and GitHub App
//! - GitLab CI/CD
//! - Jenkins
//! - Azure DevOps
//! - Bitbucket Pipelines
//!
//! ## Features
//!
//! - Automated pull request checks
//! - Status reporting and commit annotations
//! - Pipeline integration
//! - Webhook handlers
//! - CLI tool for standalone scanning
//!
//! ## Usage
//!
//! ```rust
//! use caddy::integrations::{github::GitHubIntegration, IntegrationConfig};
//!
//! let config = IntegrationConfig::default();
//! let github = GitHubIntegration::new(config);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

pub mod github;
pub mod gitlab;
pub mod jenkins;
pub mod azure_devops;
pub mod bitbucket;
pub mod cli;

/// Integration error types
#[derive(Error, Debug)]
pub enum IntegrationError {
    /// Network communication error
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Authentication error
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// API error
    #[error("API error: {0}")]
    Api(String),

    /// Webhook error
    #[error("Webhook error: {0}")]
    Webhook(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for integration operations
pub type Result<T> = std::result::Result<T, IntegrationError>;

/// Integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    /// API endpoint base URL
    pub api_url: String,

    /// Authentication token
    pub token: Option<String>,

    /// Application ID (for GitHub Apps)
    pub app_id: Option<String>,

    /// Private key (for GitHub Apps)
    pub private_key: Option<String>,

    /// Webhook secret
    pub webhook_secret: Option<String>,

    /// Additional custom settings
    pub custom_settings: HashMap<String, String>,

    /// Enable debug logging
    pub debug: bool,
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            api_url: String::new(),
            token: None,
            app_id: None,
            private_key: None,
            webhook_secret: None,
            custom_settings: HashMap::new(),
            debug: false,
        }
    }
}

/// Check status for CI/CD systems
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CheckStatus {
    /// Check is queued
    Queued,

    /// Check is in progress
    InProgress,

    /// Check completed successfully
    Success,

    /// Check completed with warnings
    Warning,

    /// Check failed
    Failure,

    /// Check was cancelled
    Cancelled,

    /// Check was skipped
    Skipped,
}

impl CheckStatus {
    /// Convert to exit code for CLI
    pub fn to_exit_code(&self) -> i32 {
        match self {
            CheckStatus::Success => 0,
            CheckStatus::Warning => 1,
            CheckStatus::Failure => 2,
            CheckStatus::Cancelled => 3,
            CheckStatus::Skipped => 4,
            _ => 5,
        }
    }
}

/// Check result with details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Check status
    pub status: CheckStatus,

    /// Summary message
    pub summary: String,

    /// Detailed description
    pub details: Option<String>,

    /// Annotations (file, line, message)
    pub annotations: Vec<Annotation>,

    /// Execution time in milliseconds
    pub execution_time_ms: u64,

    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Code annotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    /// File path
    pub path: String,

    /// Start line (1-indexed)
    pub start_line: usize,

    /// End line (1-indexed)
    pub end_line: usize,

    /// Start column (1-indexed, optional)
    pub start_column: Option<usize>,

    /// End column (1-indexed, optional)
    pub end_column: Option<usize>,

    /// Annotation level
    pub level: AnnotationLevel,

    /// Annotation message
    pub message: String,

    /// Optional title
    pub title: Option<String>,

    /// Raw details
    pub raw_details: Option<String>,
}

/// Annotation severity level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnnotationLevel {
    /// Notice level
    Notice,

    /// Warning level
    Warning,

    /// Error level
    Error,
}

/// Webhook event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvent {
    /// Event type
    pub event_type: String,

    /// Event payload
    pub payload: serde_json::Value,

    /// Delivery ID
    pub delivery_id: Option<String>,

    /// Signature for verification
    pub signature: Option<String>,
}

/// Common trait for all CI/CD integrations
pub trait CIIntegration: Send + Sync {
    /// Get integration name
    fn name(&self) -> &str;

    /// Initialize the integration
    fn initialize(&mut self) -> Result<()>;

    /// Send check status
    fn send_status(&self, result: &CheckResult) -> Result<()>;

    /// Handle webhook event
    fn handle_webhook(&self, event: WebhookEvent) -> Result<()>;

    /// Verify webhook signature
    fn verify_webhook(&self, payload: &[u8], signature: &str) -> Result<bool>;
}

/// Utility functions
pub mod utils {
    use super::*;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    /// Verify HMAC SHA-256 signature
    pub fn verify_hmac_sha256(secret: &str, payload: &[u8], signature: &str) -> Result<bool> {
        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|e| IntegrationError::Authentication(e.to_string()))?;

        mac.update(payload);

        // Parse signature (usually in format "sha256=...")
        let sig_bytes = if signature.starts_with("sha256=") {
            hex::decode(&signature[7..])
                .map_err(|e| IntegrationError::Authentication(e.to_string()))?
        } else {
            hex::decode(signature)
                .map_err(|e| IntegrationError::Authentication(e.to_string()))?
        };

        Ok(mac.verify_slice(&sig_bytes).is_ok())
    }

    /// Format check duration
    pub fn format_duration(ms: u64) -> String {
        if ms < 1000 {
            format!("{}ms", ms)
        } else if ms < 60_000 {
            format!("{:.2}s", ms as f64 / 1000.0)
        } else {
            let minutes = ms / 60_000;
            let seconds = (ms % 60_000) / 1000;
            format!("{}m {}s", minutes, seconds)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_status_exit_codes() {
        assert_eq!(CheckStatus::Success.to_exit_code(), 0);
        assert_eq!(CheckStatus::Warning.to_exit_code(), 1);
        assert_eq!(CheckStatus::Failure.to_exit_code(), 2);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(utils::format_duration(500), "500ms");
        assert_eq!(utils::format_duration(1500), "1.50s");
        assert_eq!(utils::format_duration(65000), "1m 5s");
    }

    #[test]
    fn test_verify_hmac() {
        let secret = "my-secret-key";
        let payload = b"test payload";

        // Generate valid signature
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload);
        let result = mac.finalize();
        let signature = format!("sha256={}", hex::encode(result.into_bytes()));

        assert!(utils::verify_hmac_sha256(secret, payload, &signature).unwrap());
        assert!(!utils::verify_hmac_sha256(secret, payload, "sha256=invalid").unwrap_or(false));
    }
}
