//! # Bitbucket Integration
//!
//! Comprehensive Bitbucket integration for CADDY, supporting:
//! - Bitbucket Pipelines integration
//! - Pull Request checks and comments
//! - Commit status updates
//! - Code insights (Bitbucket Cloud)
//! - Repository webhooks
//!
//! ## Setup
//!
//! ### Bitbucket Cloud
//! 1. Create an App Password:
//!    - User Settings > App passwords
//!    - Permissions: Repositories (Read, Write), Pull Requests (Read, Write), Pipelines (Read)
//!
//! ### Bitbucket Server/Data Center
//! 1. Create a Personal Access Token:
//!    - User Settings > Personal access tokens
//!    - Permissions: Repository read/write, Pull request read/write
//!
//! ### Bitbucket Pipelines
//! ```yaml
//! pipelines:
//!   default:
//!     - step:
//!         name: CADDY Design Check
//!         script:
//!           - caddy-cli scan --format bitbucket
//! ```
//!
//! ## Usage
//!
//! ```rust
//! use caddy::integrations::bitbucket::{BitbucketIntegration, BitbucketConfig};
//!
//! let config = BitbucketConfig {
//!     workspace: "myworkspace".to_string(),
//!     repository: "myrepo".to_string(),
//!     username: "user".to_string(),
//!     token: "app-password".to_string(),
//!     ..Default::default()
//! };
//!
//! let mut bitbucket = BitbucketIntegration::new(config);
//! bitbucket.initialize()?;
//! ```

use super::{
    Annotation, AnnotationLevel, CheckResult, CheckStatus, CIIntegration, IntegrationError,
    Result, WebhookEvent,
};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};

const BITBUCKET_CLOUD_API_URL: &str = "https://api.bitbucket.org/2.0";

/// Bitbucket platform type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BitbucketPlatform {
    /// Bitbucket Cloud
    Cloud,
    /// Bitbucket Server/Data Center
    Server,
}

/// Bitbucket-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitbucketConfig {
    /// Platform type
    pub platform: BitbucketPlatform,

    /// API base URL (for Bitbucket Server)
    pub api_url: String,

    /// Workspace (Cloud) or Project (Server)
    pub workspace: String,

    /// Repository slug
    pub repository: String,

    /// Username
    pub username: String,

    /// App password (Cloud) or Personal Access Token (Server)
    pub token: String,

    /// Webhook secret
    pub webhook_secret: Option<String>,

    /// Enable debug logging
    pub debug: bool,
}

impl Default for BitbucketConfig {
    fn default() -> Self {
        Self {
            platform: BitbucketPlatform::Cloud,
            api_url: BITBUCKET_CLOUD_API_URL.to_string(),
            workspace: String::new(),
            repository: String::new(),
            username: String::new(),
            token: String::new(),
            webhook_secret: None,
            debug: false,
        }
    }
}

/// Bitbucket integration implementation
pub struct BitbucketIntegration {
    config: BitbucketConfig,
    client: Client,
}

/// Commit build status
#[derive(Debug, Serialize, Deserialize)]
struct BuildStatus {
    state: String,
    key: String,
    name: String,
    url: String,
    description: String,
}

/// Pull request comment
#[derive(Debug, Serialize, Deserialize)]
struct PullRequestComment {
    content: CommentContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    inline: Option<InlineComment>,
}

/// Comment content
#[derive(Debug, Serialize, Deserialize)]
struct CommentContent {
    raw: String,
}

/// Inline comment position
#[derive(Debug, Serialize, Deserialize)]
struct InlineComment {
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    to: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    from: Option<u32>,
}

/// Code Insights report (Bitbucket Cloud)
#[derive(Debug, Serialize, Deserialize)]
struct CodeInsightsReport {
    title: String,
    details: String,
    result: String,
    reporter: String,
    report_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    link: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    data: Vec<CodeInsightsData>,
}

/// Code Insights data point
#[derive(Debug, Serialize, Deserialize)]
struct CodeInsightsData {
    title: String,
    #[serde(rename = "type")]
    data_type: String,
    value: serde_json::Value,
}

/// Code Insights annotation
#[derive(Debug, Serialize, Deserialize)]
struct CodeInsightsAnnotation {
    external_id: String,
    annotation_type: String,
    path: String,
    line: u32,
    summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<String>,
    severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    link: Option<String>,
}

impl BitbucketIntegration {
    /// Create a new Bitbucket integration
    pub fn new(config: BitbucketConfig) -> Self {
        let client = Client::builder()
            .user_agent(format!("CADDY-CI/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    /// Create commit build status
    pub async fn create_build_status(
        &self,
        commit_hash: &str,
        result: &CheckResult,
        target_url: Option<String>,
    ) -> Result<()> {
        let url = match self.config.platform {
            BitbucketPlatform::Cloud => format!(
                "{}/repositories/{}/{}/commit/{}/statuses/build",
                self.config.api_url, self.config.workspace, self.config.repository, commit_hash
            ),
            BitbucketPlatform::Server => format!(
                "{}/rest/build-status/1.0/commits/{}",
                self.config.api_url, commit_hash
            ),
        };

        let state = match result.status {
            CheckStatus::Success => "SUCCESSFUL",
            CheckStatus::Warning => "SUCCESSFUL",
            CheckStatus::Failure => "FAILED",
            CheckStatus::InProgress => "INPROGRESS",
            CheckStatus::Queued => "INPROGRESS",
            CheckStatus::Cancelled => "STOPPED",
            CheckStatus::Skipped => "STOPPED",
        };

        let build_status = BuildStatus {
            state: state.to_string(),
            key: "caddy-check".to_string(),
            name: "CADDY Design Check".to_string(),
            url: target_url.unwrap_or_default(),
            description: result.summary.clone(),
        };

        let response = self
            .client
            .post(&url)
            .basic_auth(&self.config.username, Some(&self.config.token))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&build_status)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(IntegrationError::Api(format!(
                "Failed to create build status: {} - {}",
                status, body
            )));
        }

        if self.config.debug {
            println!("✓ Bitbucket build status created successfully");
        }

        Ok(())
    }

    /// Post a comment on a pull request
    pub async fn post_pr_comment(
        &self,
        pr_id: u32,
        comment: &str,
        annotation: Option<&Annotation>,
    ) -> Result<()> {
        let url = match self.config.platform {
            BitbucketPlatform::Cloud => format!(
                "{}/repositories/{}/{}/pullrequests/{}/comments",
                self.config.api_url, self.config.workspace, self.config.repository, pr_id
            ),
            BitbucketPlatform::Server => format!(
                "{}/rest/api/1.0/projects/{}/repos/{}/pull-requests/{}/comments",
                self.config.api_url, self.config.workspace, self.config.repository, pr_id
            ),
        };

        let inline = annotation.map(|a| InlineComment {
            path: a.path.clone(),
            to: Some(a.start_line as u32),
            from: if a.end_line != a.start_line {
                Some(a.end_line as u32)
            } else {
                None
            },
        });

        let pr_comment = PullRequestComment {
            content: CommentContent {
                raw: comment.to_string(),
            },
            inline,
        };

        let response = self
            .client
            .post(&url)
            .basic_auth(&self.config.username, Some(&self.config.token))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&pr_comment)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(IntegrationError::Api(format!(
                "Failed to post PR comment: {} - {}",
                status, body
            )));
        }

        if self.config.debug {
            println!("✓ Bitbucket PR comment posted successfully");
        }

        Ok(())
    }

    /// Create Code Insights report (Bitbucket Cloud only)
    pub async fn create_code_insights_report(
        &self,
        commit_hash: &str,
        result: &CheckResult,
    ) -> Result<()> {
        if self.config.platform != BitbucketPlatform::Cloud {
            return Err(IntegrationError::Configuration(
                "Code Insights only available on Bitbucket Cloud".to_string(),
            ));
        }

        let report_id = "caddy-design-check";
        let url = format!(
            "{}/repositories/{}/{}/commit/{}/reports/{}",
            self.config.api_url,
            self.config.workspace,
            self.config.repository,
            commit_hash,
            report_id
        );

        let result_state = match result.status {
            CheckStatus::Success => "PASSED",
            CheckStatus::Warning => "PASSED",
            CheckStatus::Failure => "FAILED",
            _ => "PENDING",
        };

        let errors = result
            .annotations
            .iter()
            .filter(|a| a.level == AnnotationLevel::Error)
            .count();
        let warnings = result
            .annotations
            .iter()
            .filter(|a| a.level == AnnotationLevel::Warning)
            .count();

        let report = CodeInsightsReport {
            title: "CADDY Design Check".to_string(),
            details: result.summary.clone(),
            result: result_state.to_string(),
            reporter: "CADDY".to_string(),
            report_type: "SECURITY".to_string(),
            link: None,
            data: vec![
                CodeInsightsData {
                    title: "Errors".to_string(),
                    data_type: "NUMBER".to_string(),
                    value: serde_json::json!(errors),
                },
                CodeInsightsData {
                    title: "Warnings".to_string(),
                    data_type: "NUMBER".to_string(),
                    value: serde_json::json!(warnings),
                },
                CodeInsightsData {
                    title: "Execution Time".to_string(),
                    data_type: "DURATION".to_string(),
                    value: serde_json::json!(result.execution_time_ms),
                },
            ],
        };

        let response = self
            .client
            .put(&url)
            .basic_auth(&self.config.username, Some(&self.config.token))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&report)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(IntegrationError::Api(format!(
                "Failed to create Code Insights report: {} - {}",
                status, body
            )));
        }

        // Add annotations
        self.create_code_insights_annotations(commit_hash, report_id, result)
            .await?;

        if self.config.debug {
            println!("✓ Code Insights report created successfully");
        }

        Ok(())
    }

    /// Create Code Insights annotations
    async fn create_code_insights_annotations(
        &self,
        commit_hash: &str,
        report_id: &str,
        result: &CheckResult,
    ) -> Result<()> {
        let url = format!(
            "{}/repositories/{}/{}/commit/{}/reports/{}/annotations",
            self.config.api_url,
            self.config.workspace,
            self.config.repository,
            commit_hash,
            report_id
        );

        let annotations: Vec<CodeInsightsAnnotation> = result
            .annotations
            .iter()
            .enumerate()
            .map(|(idx, a)| {
                let severity = match a.level {
                    AnnotationLevel::Error => "HIGH",
                    AnnotationLevel::Warning => "MEDIUM",
                    AnnotationLevel::Notice => "LOW",
                };

                let annotation_type = match a.level {
                    AnnotationLevel::Error => "BUG",
                    AnnotationLevel::Warning => "CODE_SMELL",
                    AnnotationLevel::Notice => "VULNERABILITY",
                };

                CodeInsightsAnnotation {
                    external_id: format!("caddy-{}", idx),
                    annotation_type: annotation_type.to_string(),
                    path: a.path.clone(),
                    line: a.start_line as u32,
                    summary: a.message.clone(),
                    details: a.raw_details.clone(),
                    severity: severity.to_string(),
                    link: None,
                }
            })
            .collect();

        // Bitbucket has a limit of 100 annotations per request
        for chunk in annotations.chunks(100) {
            let response = self
                .client
                .post(&url)
                .basic_auth(&self.config.username, Some(&self.config.token))
                .header(header::CONTENT_TYPE, "application/json")
                .json(chunk)
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(IntegrationError::Api(format!(
                    "Failed to create Code Insights annotations: {} - {}",
                    status, body
                )));
            }
        }

        Ok(())
    }

    /// Format result as PR summary comment
    pub fn format_pr_comment(&self, result: &CheckResult) -> String {
        let icon = match result.status {
            CheckStatus::Success => "✅",
            CheckStatus::Warning => "⚠️",
            CheckStatus::Failure => "❌",
            _ => "ℹ️",
        };

        let mut comment = format!(
            "{} **CADDY Design Check**\n\n**Status**: {}\n\n",
            icon, result.summary
        );

        if !result.annotations.is_empty() {
            let errors = result
                .annotations
                .iter()
                .filter(|a| a.level == AnnotationLevel::Error)
                .count();
            let warnings = result
                .annotations
                .iter()
                .filter(|a| a.level == AnnotationLevel::Warning)
                .count();
            let notices = result
                .annotations
                .iter()
                .filter(|a| a.level == AnnotationLevel::Notice)
                .count();

            comment.push_str(&format!(
                "**Summary**: {} errors, {} warnings, {} notices\n\n",
                errors, warnings, notices
            ));

            comment.push_str("See inline comments for details.\n");
        } else {
            comment.push_str("✨ No issues found!\n");
        }

        comment.push_str(&format!(
            "\n---\n*Execution time: {}*",
            super::utils::format_duration(result.execution_time_ms)
        ));

        comment
    }

    /// Verify webhook signature
    pub fn verify_webhook_signature(&self, payload: &[u8], signature: &str) -> Result<bool> {
        let secret = self
            .config
            .webhook_secret
            .as_ref()
            .ok_or_else(|| IntegrationError::Configuration("Webhook secret not set".to_string()))?;

        super::utils::verify_hmac_sha256(secret, payload, signature)
    }
}

impl CIIntegration for BitbucketIntegration {
    fn name(&self) -> &str {
        "Bitbucket"
    }

    fn initialize(&mut self) -> Result<()> {
        if self.config.workspace.is_empty() {
            return Err(IntegrationError::Configuration(
                "Workspace not set".to_string(),
            ));
        }

        if self.config.repository.is_empty() {
            return Err(IntegrationError::Configuration(
                "Repository not set".to_string(),
            ));
        }

        if self.config.username.is_empty() || self.config.token.is_empty() {
            return Err(IntegrationError::Configuration(
                "Credentials not set".to_string(),
            ));
        }

        if self.config.debug {
            println!("✓ Bitbucket integration initialized successfully");
        }

        Ok(())
    }

    fn send_status(fn send_status(&self, result: &CheckResult)self, _result: fn send_status(&self, result: &CheckResult)CheckResult) -> Result<()> {
        eprintln!("Warning: send_status called on async Bitbucket integration");
        eprintln!("Use create_build_status or create_code_insights_report directly instead");
        Ok(())
    }

    fn handle_webhook(&self, event: WebhookEvent) -> Result<()> {
        if self.config.debug {
            println!("Received Bitbucket webhook: {}", event.event_type);
        }

        match event.event_type.as_str() {
            "repo:push" => {
                if self.config.debug {
                    println!("Push event received");
                }
            }
            "pullrequest:created" | "pullrequest:updated" => {
                if self.config.debug {
                    println!("Pull request event received");
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

/// Bitbucket Pipelines helpers
pub mod pipelines {
    /// Generate Bitbucket Pipelines YAML
    pub fn pipeline_yaml() -> &'static str {
        r#"
pipelines:
  default:
    - step:
        name: CADDY Design Check
        script:
          - caddy-cli scan --format bitbucket
        after-script:
          - pipe: atlassian/bitbucket-upload-file:0.3.2
            variables:
              FILENAME: 'target/caddy-reports/report.html'
              ARTIFACT_NAME: 'caddy-report'

  pull-requests:
    '**':
      - step:
          name: CADDY PR Check
          script:
            - caddy-cli scan --format bitbucket --pr $BITBUCKET_PR_ID
"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitbucket_config_default() {
        let config = BitbucketConfig::default();
        assert_eq!(config.platform, BitbucketPlatform::Cloud);
        assert_eq!(config.api_url, BITBUCKET_CLOUD_API_URL);
    }

    #[test]
    fn test_pr_comment_formatting() {
        let integration = BitbucketIntegration::new(BitbucketConfig::default());
        let result = CheckResult {
            status: CheckStatus::Success,
            summary: "All checks passed".to_string(),
            details: None,
            annotations: vec![],
            execution_time_ms: 1500,
            metadata: std::collections::HashMap::new(),
        };

        let comment = integration.format_pr_comment(&result);
        assert!(comment.contains("✅"));
        assert!(comment.contains("All checks passed"));
        assert!(comment.contains("No issues found"));
    }
}
