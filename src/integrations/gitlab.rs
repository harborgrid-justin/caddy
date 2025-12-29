//! # GitLab Integration
//!
//! Comprehensive GitLab CI/CD integration for CADDY, supporting:
//! - GitLab CI/CD pipeline integration
//! - Merge request comments and threads
//! - Pipeline status updates
//! - Commit status API
//! - Code quality reports
//! - GitLab webhooks
//!
//! ## Setup
//!
//! 1. Create a GitLab access token with the following scopes:
//!    - api
//!    - read_repository
//!    - write_repository
//! 2. Configure webhook in your GitLab project:
//!    - Settings > Webhooks
//!    - Enable: Pipeline, Merge Request, Push events
//!
//! ## Usage
//!
//! ```rust
//! use caddy::integrations::gitlab::{GitLabIntegration, GitLabConfig};
//!
//! let config = GitLabConfig {
//!     api_url: "https://gitlab.com".to_string(),
//!     token: "your-access-token".to_string(),
//!     project_id: "12345".to_string(),
//!     ..Default::default()
//! };
//!
//! let mut gitlab = GitLabIntegration::new(config);
//! gitlab.initialize()?;
//! ```

use super::{
    Annotation, AnnotationLevel, CheckResult, CheckStatus, CIIntegration, IntegrationError,
    Result, WebhookEvent,
};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};

const GITLAB_API_URL: &str = "https://gitlab.com";

/// GitLab-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabConfig {
    /// GitLab instance URL
    pub api_url: String,

    /// Personal or project access token
    pub token: String,

    /// Project ID
    pub project_id: String,

    /// Webhook secret token
    pub webhook_secret: Option<String>,

    /// Pipeline name
    pub pipeline_name: String,

    /// Enable debug logging
    pub debug: bool,
}

impl Default for GitLabConfig {
    fn default() -> Self {
        Self {
            api_url: GITLAB_API_URL.to_string(),
            token: String::new(),
            project_id: String::new(),
            webhook_secret: None,
            pipeline_name: "CADDY Design Check".to_string(),
            debug: false,
        }
    }
}

/// GitLab integration implementation
pub struct GitLabIntegration {
    config: GitLabConfig,
    client: Client,
}

/// GitLab commit status request
#[derive(Debug, Serialize, Deserialize)]
struct CommitStatusRequest {
    state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    ref_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    coverage: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pipeline_id: Option<String>,
}

/// Merge request note/comment
#[derive(Debug, Serialize, Deserialize)]
struct MergeRequestNote {
    body: String,
}

/// Merge request discussion
#[derive(Debug, Serialize, Deserialize)]
struct MergeRequestDiscussion {
    body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    position: Option<DiffPosition>,
}

/// Position in diff for inline comments
#[derive(Debug, Serialize, Deserialize)]
struct DiffPosition {
    base_sha: String,
    start_sha: String,
    head_sha: String,
    position_type: String,
    new_path: Option<String>,
    old_path: Option<String>,
    new_line: Option<usize>,
    old_line: Option<usize>,
}

/// Code quality report format
#[derive(Debug, Serialize, Deserialize)]
struct CodeQualityReport {
    fingerprint: String,
    severity: String,
    description: String,
    location: CodeQualityLocation,
}

/// Location in code quality report
#[derive(Debug, Serialize, Deserialize)]
struct CodeQualityLocation {
    path: String,
    lines: CodeQualityLines,
}

/// Line range in code quality report
#[derive(Debug, Serialize, Deserialize)]
struct CodeQualityLines {
    begin: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<usize>,
}

impl GitLabIntegration {
    /// Create a new GitLab integration
    pub fn new(config: GitLabConfig) -> Self {
        let client = Client::builder()
            .user_agent(format!("CADDY-CI/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    /// Create or update commit status
    pub async fn create_commit_status(
        &self,
        sha: &str,
        result: &CheckResult,
        ref_name: Option<&str>,
        pipeline_id: Option<&str>,
    ) -> Result<()> {
        let url = format!(
            "{}/api/v4/projects/{}/statuses/{}",
            self.config.api_url, self.config.project_id, sha
        );

        let state = match result.status {
            CheckStatus::Success => "success",
            CheckStatus::Warning => "success",
            CheckStatus::Failure => "failed",
            CheckStatus::InProgress => "running",
            CheckStatus::Queued => "pending",
            CheckStatus::Cancelled => "canceled",
            CheckStatus::Skipped => "skipped",
        };

        let status_request = CommitStatusRequest {
            state: state.to_string(),
            ref_: ref_name.map(String::from),
            name: Some(self.config.pipeline_name.clone()),
            target_url: None,
            description: Some(result.summary.clone()),
            coverage: None,
            pipeline_id: pipeline_id.map(String::from),
        };

        let response = self
            .client
            .post(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.config.token))
            .header(header::CONTENT_TYPE, "application/json")
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
            println!("✓ GitLab commit status created successfully");
        }

        Ok(())
    }

    /// Post a comment on a merge request
    pub async fn post_merge_request_comment(
        &self,
        merge_request_iid: u64,
        comment: &str,
    ) -> Result<()> {
        let url = format!(
            "{}/api/v4/projects/{}/merge_requests/{}/notes",
            self.config.api_url, self.config.project_id, merge_request_iid
        );

        let note = MergeRequestNote {
            body: comment.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.config.token))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&note)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(IntegrationError::Api(format!(
                "Failed to post merge request comment: {} - {}",
                status, body
            )));
        }

        if self.config.debug {
            println!("✓ GitLab merge request comment posted successfully");
        }

        Ok(())
    }

    /// Create an inline comment on a merge request
    pub async fn create_inline_comment(
        &self,
        merge_request_iid: u64,
        annotation: &Annotation,
        base_sha: &str,
        start_sha: &str,
        head_sha: &str,
    ) -> Result<()> {
        let url = format!(
            "{}/api/v4/projects/{}/merge_requests/{}/discussions",
            self.config.api_url, self.config.project_id, merge_request_iid
        );

        let position = DiffPosition {
            base_sha: base_sha.to_string(),
            start_sha: start_sha.to_string(),
            head_sha: head_sha.to_string(),
            position_type: "text".to_string(),
            new_path: Some(annotation.path.clone()),
            old_path: Some(annotation.path.clone()),
            new_line: Some(annotation.start_line),
            old_line: None,
        };

        let discussion = MergeRequestDiscussion {
            body: format!(
                "**{}**: {}\n\n{}",
                match annotation.level {
                    AnnotationLevel::Error => "Error",
                    AnnotationLevel::Warning => "Warning",
                    AnnotationLevel::Notice => "Notice",
                },
                annotation.title.as_ref().unwrap_or(&annotation.message),
                annotation.message
            ),
            position: Some(position),
        };

        let response = self
            .client
            .post(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.config.token))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&discussion)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(IntegrationError::Api(format!(
                "Failed to create inline comment: {} - {}",
                status, body
            )));
        }

        if self.config.debug {
            println!("✓ GitLab inline comment created successfully");
        }

        Ok(())
    }

    /// Generate code quality report (GitLab format)
    pub fn generate_code_quality_report(&self, result: &CheckResult) -> Vec<CodeQualityReport> {
        result
            .annotations
            .iter()
            .map(|annotation| {
                let severity = match annotation.level {
                    AnnotationLevel::Error => "major",
                    AnnotationLevel::Warning => "minor",
                    AnnotationLevel::Notice => "info",
                };

                // Generate fingerprint for deduplication
                let fingerprint = format!(
                    "{:x}",
                    md5::compute(format!(
                        "{}:{}:{}",
                        annotation.path, annotation.start_line, annotation.message
                    ))
                );

                CodeQualityReport {
                    fingerprint,
                    severity: severity.to_string(),
                    description: annotation.message.clone(),
                    location: CodeQualityLocation {
                        path: annotation.path.clone(),
                        lines: CodeQualityLines {
                            begin: annotation.start_line,
                            end: if annotation.end_line != annotation.start_line {
                                Some(annotation.end_line)
                            } else {
                                None
                            },
                        },
                    },
                }
            })
            .collect()
    }

    /// Format check result as merge request comment
    pub fn format_merge_request_comment(&self, result: &CheckResult) -> String {
        let icon = match result.status {
            CheckStatus::Success => "✅",
            CheckStatus::Warning => "⚠️",
            CheckStatus::Failure => "❌",
            CheckStatus::InProgress => "⏳",
            CheckStatus::Queued => "⏱️",
            CheckStatus::Cancelled => "⛔",
            CheckStatus::Skipped => "⏭️",
        };

        let mut comment = format!(
            "## {} CADDY Design Check\n\n**Status**: {}\n\n",
            icon, result.summary
        );

        if !result.annotations.is_empty() {
            comment.push_str(&format!("### Issues Found ({})\n\n", result.annotations.len()));

            // Group by level
            let errors: Vec<_> = result
                .annotations
                .iter()
                .filter(|a| a.level == AnnotationLevel::Error)
                .collect();
            let warnings: Vec<_> = result
                .annotations
                .iter()
                .filter(|a| a.level == AnnotationLevel::Warning)
                .collect();
            let notices: Vec<_> = result
                .annotations
                .iter()
                .filter(|a| a.level == AnnotationLevel::Notice)
                .collect();

            if !errors.is_empty() {
                comment.push_str(&format!("#### ❌ Errors ({})\n\n", errors.len()));
                for error in errors.iter().take(10) {
                    comment.push_str(&format!(
                        "- `{}:{}` - {}\n",
                        error.path, error.start_line, error.message
                    ));
                }
                if errors.len() > 10 {
                    comment.push_str(&format!("\n*... and {} more errors*\n", errors.len() - 10));
                }
                comment.push('\n');
            }

            if !warnings.is_empty() {
                comment.push_str(&format!("#### ⚠️ Warnings ({})\n\n", warnings.len()));
                for warning in warnings.iter().take(10) {
                    comment.push_str(&format!(
                        "- `{}:{}` - {}\n",
                        warning.path, warning.start_line, warning.message
                    ));
                }
                if warnings.len() > 10 {
                    comment.push_str(&format!(
                        "\n*... and {} more warnings*\n",
                        warnings.len() - 10
                    ));
                }
                comment.push('\n');
            }

            if !notices.is_empty() && notices.len() <= 5 {
                comment.push_str(&format!("#### ℹ️ Notices ({})\n\n", notices.len()));
                for notice in &notices {
                    comment.push_str(&format!(
                        "- `{}:{}` - {}\n",
                        notice.path, notice.start_line, notice.message
                    ));
                }
                comment.push('\n');
            }
        } else {
            comment.push_str("✨ No issues found!\n\n");
        }

        if let Some(details) = &result.details {
            comment.push_str("### Details\n\n");
            comment.push_str(details);
            comment.push_str("\n\n");
        }

        comment.push_str(&format!(
            "---\n*Execution time: {}*\n",
            super::utils::format_duration(result.execution_time_ms)
        ));

        comment
    }

    /// Verify GitLab webhook token
    pub fn verify_webhook_token(&self, token: &str) -> Result<bool> {
        if let Some(secret) = &self.config.webhook_secret {
            Ok(token == secret)
        } else {
            Err(IntegrationError::Configuration(
                "Webhook secret not configured".to_string(),
            ))
        }
    }
}

impl CIIntegration for GitLabIntegration {
    fn name(&self) -> &str {
        "GitLab"
    }

    fn initialize(&mut self) -> Result<()> {
        if self.config.token.is_empty() {
            return Err(IntegrationError::Configuration("Token not set".to_string()));
        }

        if self.config.project_id.is_empty() {
            return Err(IntegrationError::Configuration(
                "Project ID not set".to_string(),
            ));
        }

        if self.config.debug {
            println!("✓ GitLab integration initialized successfully");
        }

        Ok(())
    }

    fn send_status(fn send_status(&self, result: &CheckResult)self, _result: fn send_status(&self, result: &CheckResult)CheckResult) -> Result<()> {
        eprintln!("Warning: send_status called on async GitLab integration");
        eprintln!("Use create_commit_status or post_merge_request_comment directly instead");
        Ok(())
    }

    fn handle_webhook(&self, event: WebhookEvent) -> Result<()> {
        if self.config.debug {
            println!("Received GitLab webhook: {}", event.event_type);
        }

        match event.event_type.as_str() {
            "Pipeline Hook" => {
                if self.config.debug {
                    println!("Pipeline event received");
                }
            }
            "Merge Request Hook" => {
                if self.config.debug {
                    println!("Merge request event received");
                }
            }
            "Push Hook" => {
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

    fn verify_webhook(&self, _payload: &[u8], signature: &str) -> Result<bool> {
        self.verify_webhook_token(signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gitlab_config_default() {
        let config = GitLabConfig::default();
        assert_eq!(config.api_url, GITLAB_API_URL);
        assert_eq!(config.pipeline_name, "CADDY Design Check");
    }

    #[test]
    fn test_code_quality_report_generation() {
        let integration = GitLabIntegration::new(GitLabConfig::default());
        let result = CheckResult {
            status: CheckStatus::Success,
            summary: "Test".to_string(),
            details: None,
            annotations: vec![Annotation {
                path: "test.rs".to_string(),
                start_line: 10,
                end_line: 10,
                start_column: None,
                end_column: None,
                level: AnnotationLevel::Warning,
                message: "Test warning".to_string(),
                title: None,
                raw_details: None,
            }],
            execution_time_ms: 100,
            metadata: std::collections::HashMap::new(),
        };

        let report = integration.generate_code_quality_report(&result);
        assert_eq!(report.len(), 1);
        assert_eq!(report[0].severity, "minor");
        assert_eq!(report[0].location.path, "test.rs");
    }

    #[test]
    fn test_merge_request_comment_formatting() {
        let integration = GitLabIntegration::new(GitLabConfig::default());
        let result = CheckResult {
            status: CheckStatus::Success,
            summary: "All checks passed".to_string(),
            details: None,
            annotations: vec![],
            execution_time_ms: 1500,
            metadata: std::collections::HashMap::new(),
        };

        let comment = integration.format_merge_request_comment(&result);
        assert!(comment.contains("✅"));
        assert!(comment.contains("All checks passed"));
        assert!(comment.contains("No issues found"));
    }
}
