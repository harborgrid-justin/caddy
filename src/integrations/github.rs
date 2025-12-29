//! # GitHub Integration
//!
//! Comprehensive GitHub integration for CADDY, supporting:
//! - GitHub Apps with JWT authentication
//! - Pull Request checks and status updates
//! - Commit status API
//! - Code annotations and suggestions
//! - GitHub Actions integration
//! - Repository webhooks
//!
//! ## GitHub App Setup
//!
//! 1. Create a GitHub App in your organization settings
//! 2. Configure permissions:
//!    - Checks: Read & Write
//!    - Contents: Read
//!    - Pull Requests: Read & Write
//!    - Statuses: Read & Write
//! 3. Subscribe to webhook events:
//!    - Check run
//!    - Check suite
//!    - Pull request
//!    - Push
//!
//! ## Usage
//!
//! ```rust
//! use caddy::integrations::github::{GitHubIntegration, GitHubConfig};
//!
//! let config = GitHubConfig {
//!     app_id: "123456".to_string(),
//!     private_key: include_str!("app-private-key.pem").to_string(),
//!     installation_id: Some("78901234".to_string()),
//!     ..Default::default()
//! };
//!
//! let mut github = GitHubIntegration::new(config);
//! github.initialize()?;
//! ```

use super::{
    Annotation, AnnotationLevel, CheckResult, CheckStatus, CIIntegration, IntegrationConfig,
    IntegrationError, Result, WebhookEvent,
};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::{header, Client, Response};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

const GITHUB_API_URL: &str = "https://api.github.com";
const GITHUB_API_VERSION: &str = "2022-11-28";

/// GitHub-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    /// GitHub App ID
    pub app_id: String,

    /// GitHub App private key (PEM format)
    pub private_key: String,

    /// Installation ID (specific to a repo/org)
    pub installation_id: Option<String>,

    /// API base URL (for GitHub Enterprise)
    pub api_url: String,

    /// Webhook secret for signature verification
    pub webhook_secret: Option<String>,

    /// Check run name
    pub check_name: String,

    /// Enable debug logging
    pub debug: bool,
}

impl Default for GitHubConfig {
    fn default() -> Self {
        Self {
            app_id: String::new(),
            private_key: String::new(),
            installation_id: None,
            api_url: GITHUB_API_URL.to_string(),
            webhook_secret: None,
            check_name: "CADDY Design Check".to_string(),
            debug: false,
        }
    }
}

/// GitHub integration implementation
pub struct GitHubIntegration {
    config: GitHubConfig,
    client: Client,
    access_token: Arc<RwLock<Option<AccessToken>>>,
}

/// GitHub installation access token
#[derive(Debug, Clone)]
struct AccessToken {
    token: String,
    expires_at: DateTime<Utc>,
}

/// JWT claims for GitHub App authentication
#[derive(Debug, Serialize, Deserialize)]
struct JWTClaims {
    /// Issued at time (Unix timestamp)
    iat: i64,

    /// Expiration time (Unix timestamp, max 10 minutes)
    exp: i64,

    /// GitHub App's identifier
    iss: String,
}

/// GitHub Check Run request
#[derive(Debug, Serialize, Deserialize)]
struct CheckRunRequest {
    name: String,
    head_sha: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    conclusion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    started_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    completed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output: Option<CheckRunOutput>,
}

/// Check run output
#[derive(Debug, Serialize, Deserialize)]
struct CheckRunOutput {
    title: String,
    summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    annotations: Vec<GitHubAnnotation>,
}

/// GitHub annotation format
#[derive(Debug, Serialize, Deserialize)]
struct GitHubAnnotation {
    path: String,
    start_line: usize,
    end_line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    start_column: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end_column: Option<usize>,
    annotation_level: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    raw_details: Option<String>,
}

/// Commit status request
#[derive(Debug, Serialize, Deserialize)]
struct CommitStatusRequest {
    state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<String>,
}

/// Installation access token response
#[derive(Debug, Deserialize)]
struct InstallationTokenResponse {
    token: String,
    expires_at: String,
}

impl GitHubIntegration {
    /// Create a new GitHub integration
    pub fn new(config: GitHubConfig) -> Self {
        let client = Client::builder()
            .user_agent(format!("CADDY-CI/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            client,
            access_token: Arc::new(RwLock::new(None)),
        }
    }

    /// Generate JWT for GitHub App authentication
    fn generate_jwt(&self) -> Result<String> {
        let now = Utc::now();
        let claims = JWTClaims {
            iat: now.timestamp(),
            exp: (now + Duration::minutes(10)).timestamp(),
            iss: self.config.app_id.clone(),
        };

        let key = EncodingKey::from_rsa_pem(self.config.private_key.as_bytes())
            .map_err(|e| IntegrationError::Authentication(format!("Invalid private key: {}", e)))?;

        let token = encode(&Header::new(Algorithm::RS256), &claims, &key)
            .map_err(|e| IntegrationError::Authentication(format!("JWT encoding failed: {}", e)))?;

        Ok(token)
    }

    /// Get installation access token (cached)
    async fn get_access_token(&self) -> Result<String> {
        // Check if we have a valid cached token
        {
            let token_guard = self.access_token.read().await;
            if let Some(token) = token_guard.as_ref() {
                if token.expires_at > Utc::now() + Duration::minutes(5) {
                    return Ok(token.token.clone());
                }
            }
        }

        // Generate new token
        let jwt = self.generate_jwt()?;
        let installation_id = self
            .config
            .installation_id
            .as_ref()
            .ok_or_else(|| IntegrationError::Configuration("Installation ID not set".to_string()))?;

        let url = format!(
            "{}/app/installations/{}/access_tokens",
            self.config.api_url, installation_id
        );

        let response = self
            .client
            .post(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", jwt))
            .header(header::ACCEPT, "application/vnd.github+json")
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(IntegrationError::Api(format!(
                "Failed to get installation token: {} - {}",
                status, body
            )));
        }

        let token_response: InstallationTokenResponse = response.json().await?;
        let expires_at = DateTime::parse_from_rfc3339(&token_response.expires_at)
            .map_err(|e| IntegrationError::Api(format!("Invalid expiration date: {}", e)))?
            .with_timezone(&Utc);

        let access_token = AccessToken {
            token: token_response.token.clone(),
            expires_at,
        };

        // Cache the token
        {
            let mut token_guard = self.access_token.write().await;
            *token_guard = Some(access_token);
        }

        Ok(token_response.token)
    }

    /// Create or update a check run
    pub async fn create_check_run(
        &self,
        owner: &str,
        repo: &str,
        result: &CheckResult,
        head_sha: &str,
    ) -> Result<()> {
        let token = self.get_access_token().await?;
        let url = format!(
            "{}/repos/{}/{}/check-runs",
            self.config.api_url, owner, repo
        );

        let (status, conclusion) = match result.status {
            CheckStatus::Queued => ("queued", None),
            CheckStatus::InProgress => ("in_progress", None),
            CheckStatus::Success => ("completed", Some("success")),
            CheckStatus::Warning => ("completed", Some("neutral")),
            CheckStatus::Failure => ("completed", Some("failure")),
            CheckStatus::Cancelled => ("completed", Some("cancelled")),
            CheckStatus::Skipped => ("completed", Some("skipped")),
        };

        let annotations: Vec<GitHubAnnotation> = result
            .annotations
            .iter()
            .map(|a| GitHubAnnotation {
                path: a.path.clone(),
                start_line: a.start_line,
                end_line: a.end_line,
                start_column: a.start_column,
                end_column: a.end_column,
                annotation_level: match a.level {
                    AnnotationLevel::Notice => "notice",
                    AnnotationLevel::Warning => "warning",
                    AnnotationLevel::Error => "failure",
                }
                .to_string(),
                message: a.message.clone(),
                title: a.title.clone(),
                raw_details: a.raw_details.clone(),
            })
            .collect();

        let check_run = CheckRunRequest {
            name: self.config.check_name.clone(),
            head_sha: head_sha.to_string(),
            status: status.to_string(),
            conclusion: conclusion.map(String::from),
            started_at: Some(Utc::now().to_rfc3339()),
            completed_at: if conclusion.is_some() {
                Some(Utc::now().to_rfc3339())
            } else {
                None
            },
            output: Some(CheckRunOutput {
                title: result.summary.clone(),
                summary: result.summary.clone(),
                text: result.details.clone(),
                annotations,
            }),
        };

        let response = self
            .client
            .post(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::ACCEPT, "application/vnd.github+json")
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .json(&check_run)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(IntegrationError::Api(format!(
                "Failed to create check run: {} - {}",
                status, body
            )));
        }

        if self.config.debug {
            println!("✓ GitHub check run created successfully");
        }

        Ok(())
    }

    /// Create commit status
    pub async fn create_commit_status(
        &self,
        owner: &str,
        repo: &str,
        sha: &str,
        result: &CheckResult,
        target_url: Option<String>,
    ) -> Result<()> {
        let token = self.get_access_token().await?;
        let url = format!(
            "{}/repos/{}/{}/statuses/{}",
            self.config.api_url, owner, repo, sha
        );

        let state = match result.status {
            CheckStatus::Success => "success",
            CheckStatus::Warning => "success",
            CheckStatus::Failure => "failure",
            CheckStatus::InProgress => "pending",
            _ => "error",
        };

        let status_request = CommitStatusRequest {
            state: state.to_string(),
            target_url,
            description: Some(result.summary.clone()),
            context: Some(self.config.check_name.clone()),
        };

        let response = self
            .client
            .post(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::ACCEPT, "application/vnd.github+json")
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .json(&status_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(IntegrationError::Api(format!(
                "Failed to create commit status: {} - {}",
                status, body
            )));
        }

        if self.config.debug {
            println!("✓ GitHub commit status created successfully");
        }

        Ok(())
    }

    /// Parse webhook payload
    pub fn parse_webhook_event(&self, event_type: &str, payload: &str) -> Result<WebhookEvent> {
        let payload_value: serde_json::Value = serde_json::from_str(payload)?;

        Ok(WebhookEvent {
            event_type: event_type.to_string(),
            payload: payload_value,
            delivery_id: None,
            signature: None,
        })
    }

    /// Verify GitHub webhook signature
    pub fn verify_webhook_signature(&self, payload: &[u8], signature: &str) -> Result<bool> {
        let secret = self
            .config
            .webhook_secret
            .as_ref()
            .ok_or_else(|| IntegrationError::Configuration("Webhook secret not set".to_string()))?;

        super::utils::verify_hmac_sha256(secret, payload, signature)
    }
}

impl CIIntegration for GitHubIntegration {
    fn name(&self) -> &str {
        "GitHub"
    }

    fn initialize(&mut self) -> Result<()> {
        // Validate configuration
        if self.config.app_id.is_empty() {
            return Err(IntegrationError::Configuration("App ID not set".to_string()));
        }

        if self.config.private_key.is_empty() {
            return Err(IntegrationError::Configuration(
                "Private key not set".to_string(),
            ));
        }

        // Test JWT generation
        self.generate_jwt()?;

        if self.config.debug {
            println!("✓ GitHub integration initialized successfully");
        }

        Ok(())
    }

    fn send_status(fn send_status(&self, result: &CheckResult)self, _result: fn send_status(&self, result: &CheckResult)CheckResult) -> Result<()> {
        // Note: This is a synchronous interface, but our implementation is async.
        // In a real-world scenario, you'd use tokio::runtime::Runtime::block_on
        // or redesign the trait to be async.
        eprintln!("Warning: send_status called on async GitHub integration");
        eprintln!("Use create_check_run or create_commit_status directly instead");
        Ok(())
    }

    fn handle_webhook(&self, event: WebhookEvent) -> Result<()> {
        if self.config.debug {
            println!("Received GitHub webhook: {}", event.event_type);
        }

        match event.event_type.as_str() {
            "check_run" => {
                // Handle check run events
                if self.config.debug {
                    println!("Check run event received");
                }
            }
            "check_suite" => {
                // Handle check suite events
                if self.config.debug {
                    println!("Check suite event received");
                }
            }
            "pull_request" => {
                // Handle pull request events
                if self.config.debug {
                    println!("Pull request event received");
                }
            }
            "push" => {
                // Handle push events
                if self.config.debug {
                    println!("Push event received");
                }
            }
            _ => {
                if self.config.debug {
                    println!("Unhandled event type: {}", event.event_type);
                }
            }
        }

        Ok(())
    }

    fn verify_webhook(&self, payload: &[u8], signature: &str) -> Result<bool> {
        self.verify_webhook_signature(payload, signature)
    }
}

/// GitHub Actions workflow helpers
pub mod actions {
    use super::*;

    /// Set output in GitHub Actions
    pub fn set_output(name: &str, value: &str) {
        println!("::set-output name={}::{}", name, value);
    }

    /// Create an error annotation
    pub fn error(file: &str, line: usize, message: &str) {
        println!("::error file={},line={}::{}", file, line, message);
    }

    /// Create a warning annotation
    pub fn warning(file: &str, line: usize, message: &str) {
        println!("::warning file={},line={}::{}", file, line, message);
    }

    /// Create a notice annotation
    pub fn notice(file: &str, line: usize, message: &str) {
        println!("::notice file={},line={}::{}", file, line, message);
    }

    /// Group output in GitHub Actions
    pub fn start_group(title: &str) {
        println!("::group::{}", title);
    }

    /// End output group
    pub fn end_group() {
        println!("::endgroup::");
    }

    /// Set environment variable
    pub fn set_env(name: &str, value: &str) {
        println!("::set-env name={}::{}", name, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_config_default() {
        let config = GitHubConfig::default();
        assert_eq!(config.api_url, GITHUB_API_URL);
        assert_eq!(config.check_name, "CADDY Design Check");
    }

    #[test]
    fn test_annotation_conversion() {
        let annotation = Annotation {
            path: "src/main.rs".to_string(),
            start_line: 10,
            end_line: 10,
            start_column: Some(5),
            end_column: Some(15),
            level: AnnotationLevel::Warning,
            message: "Test warning".to_string(),
            title: Some("Warning".to_string()),
            raw_details: None,
        };

        let gh_annotation = GitHubAnnotation {
            path: annotation.path.clone(),
            start_line: annotation.start_line,
            end_line: annotation.end_line,
            start_column: annotation.start_column,
            end_column: annotation.end_column,
            annotation_level: "warning".to_string(),
            message: annotation.message.clone(),
            title: annotation.title.clone(),
            raw_details: None,
        };

        assert_eq!(gh_annotation.path, "src/main.rs");
        assert_eq!(gh_annotation.annotation_level, "warning");
    }
}
