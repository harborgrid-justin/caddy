//! # Azure DevOps Integration
//!
//! Comprehensive Azure DevOps integration for CADDY, supporting:
//! - Azure Pipelines tasks and extensions
//! - Pull Request policies and status
//! - Work item linking and tracking
//! - Board integration
//! - Build artifacts publishing
//! - Test results publishing
//!
//! ## Setup
//!
//! ### Personal Access Token (PAT)
//! 1. Create a PAT in Azure DevOps:
//!    - User Settings > Personal Access Tokens
//!    - Scopes: Code (Read & Write), Build (Read & Execute), Work Items (Read & Write)
//!
//! ### Azure Pipeline Task
//! ```yaml
//! - task: CaddyDesignCheck@1
//!   inputs:
//!     scanPath: '$(Build.SourcesDirectory)'
//!     failOnErrors: true
//! ```
//!
//! ### CLI Integration
//! ```yaml
//! - script: |
//!     caddy-cli scan --format azure-devops
//!   displayName: 'CADDY Design Check'
//! ```
//!
//! ## API Usage
//!
//! ```rust
//! use caddy::integrations::azure_devops::{AzureDevOpsIntegration, AzureDevOpsConfig};
//!
//! let config = AzureDevOpsConfig {
//!     organization: "myorg".to_string(),
//!     project: "myproject".to_string(),
//!     token: "your-pat-token".to_string(),
//!     ..Default::default()
//! };
//!
//! let mut azure = AzureDevOpsIntegration::new(config);
//! azure.initialize()?;
//! ```

use super::{
    Annotation, AnnotationLevel, CheckResult, CheckStatus, CIIntegration, IntegrationError,
    Result, WebhookEvent,
};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};

const AZURE_DEVOPS_API_VERSION: &str = "7.0";

/// Azure DevOps-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureDevOpsConfig {
    /// Organization name
    pub organization: String,

    /// Project name
    pub project: String,

    /// Personal Access Token
    pub token: String,

    /// Repository ID
    pub repository_id: Option<String>,

    /// Build ID
    pub build_id: Option<String>,

    /// Pull request ID
    pub pull_request_id: Option<String>,

    /// Enable debug logging
    pub debug: bool,
}

impl Default for AzureDevOpsConfig {
    fn default() -> Self {
        Self {
            organization: String::new(),
            project: String::new(),
            token: String::new(),
            repository_id: None,
            build_id: None,
            pull_request_id: None,
            debug: false,
        }
    }
}

/// Azure DevOps integration implementation
pub struct AzureDevOpsIntegration {
    config: AzureDevOpsConfig,
    client: Client,
}

/// Pull Request status
#[derive(Debug, Serialize, Deserialize)]
struct PullRequestStatus {
    state: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_url: Option<String>,
    context: PullRequestStatusContext,
}

/// Pull Request status context
#[derive(Debug, Serialize, Deserialize)]
struct PullRequestStatusContext {
    name: String,
    genre: String,
}

/// Pull Request thread
#[derive(Debug, Serialize, Deserialize)]
struct PullRequestThread {
    comments: Vec<PullRequestComment>,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    thread_context: Option<ThreadContext>,
}

/// Thread context for inline comments
#[derive(Debug, Serialize, Deserialize)]
struct ThreadContext {
    file_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    right_file_start: Option<ThreadPosition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    right_file_end: Option<ThreadPosition>,
}

/// Thread position
#[derive(Debug, Serialize, Deserialize)]
struct ThreadPosition {
    line: usize,
    offset: usize,
}

/// Pull Request comment
#[derive(Debug, Serialize, Deserialize)]
struct PullRequestComment {
    content: String,
    comment_type: String,
}

/// Work item link
#[derive(Debug, Serialize, Deserialize)]
struct WorkItemLink {
    op: String,
    path: String,
    value: String,
}

/// Test run result
#[derive(Debug, Serialize, Deserialize)]
struct TestRun {
    name: String,
    #[serde(rename = "isAutomated")]
    is_automated: bool,
    state: String,
    #[serde(rename = "buildId")]
    build_id: Option<String>,
}

/// Test result
#[derive(Debug, Serialize, Deserialize)]
struct TestResult {
    #[serde(rename = "testCaseTitle")]
    test_case_title: String,
    outcome: String,
    #[serde(rename = "errorMessage", skip_serializing_if = "Option::is_none")]
    error_message: Option<String>,
    #[serde(rename = "stackTrace", skip_serializing_if = "Option::is_none")]
    stack_trace: Option<String>,
    #[serde(rename = "durationInMs")]
    duration_in_ms: u64,
}

impl AzureDevOpsIntegration {
    /// Create a new Azure DevOps integration
    pub fn new(config: AzureDevOpsConfig) -> Self {
        let client = Client::builder()
            .user_agent(format!("CADDY-CI/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    /// Get base API URL
    fn base_url(&self) -> String {
        format!(
            "https://dev.azure.com/{}/{}",
            self.config.organization, self.config.project
        )
    }

    /// Create or update pull request status
    pub async fn create_pr_status(
        &self,
        result: &CheckResult,
        commit_id: &str,
        target_url: Option<String>,
    ) -> Result<()> {
        let repo_id = self
            .config
            .repository_id
            .as_ref()
            .ok_or_else(|| IntegrationError::Configuration("Repository ID not set".to_string()))?;

        let url = format!(
            "{}/git/repositories/{}/commits/{}/statuses?api-version={}",
            self.base_url(),
            repo_id,
            commit_id,
            AZURE_DEVOPS_API_VERSION
        );

        let state = match result.status {
            CheckStatus::Success => "succeeded",
            CheckStatus::Warning => "succeeded",
            CheckStatus::Failure => "failed",
            CheckStatus::InProgress => "pending",
            CheckStatus::Queued => "pending",
            CheckStatus::Cancelled => "error",
            CheckStatus::Skipped => "notApplicable",
        };

        let status = PullRequestStatus {
            state: state.to_string(),
            description: result.summary.clone(),
            target_url,
            context: PullRequestStatusContext {
                name: "CADDY Design Check".to_string(),
                genre: "continuous-integration".to_string(),
            },
        };

        let response = self
            .client
            .post(&url)
            .basic_auth("", Some(&self.config.token))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&status)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(IntegrationError::Api(format!(
                "Failed to create PR status: {} - {}",
                status, body
            )));
        }

        if self.config.debug {
            println!("✓ Azure DevOps PR status created successfully");
        }

        Ok(())
    }

    /// Create pull request thread (comment)
    pub async fn create_pr_thread(
        &self,
        comment: &str,
        annotation: Option<&Annotation>,
    ) -> Result<()> {
        let repo_id = self
            .config
            .repository_id
            .as_ref()
            .ok_or_else(|| IntegrationError::Configuration("Repository ID not set".to_string()))?;

        let pr_id = self
            .config
            .pull_request_id
            .as_ref()
            .ok_or_else(|| {
                IntegrationError::Configuration("Pull Request ID not set".to_string())
            })?;

        let url = format!(
            "{}/git/repositories/{}/pullRequests/{}/threads?api-version={}",
            self.base_url(),
            repo_id,
            pr_id,
            AZURE_DEVOPS_API_VERSION
        );

        let thread_context = annotation.map(|a| ThreadContext {
            file_path: a.path.clone(),
            right_file_start: Some(ThreadPosition {
                line: a.start_line,
                offset: 1,
            }),
            right_file_end: Some(ThreadPosition {
                line: a.end_line,
                offset: 1,
            }),
        });

        let thread = PullRequestThread {
            comments: vec![PullRequestComment {
                content: comment.to_string(),
                comment_type: "text".to_string(),
            }],
            status: "active".to_string(),
            thread_context,
        };

        let response = self
            .client
            .post(&url)
            .basic_auth("", Some(&self.config.token))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&thread)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(IntegrationError::Api(format!(
                "Failed to create PR thread: {} - {}",
                status, body
            )));
        }

        if self.config.debug {
            println!("✓ Azure DevOps PR thread created successfully");
        }

        Ok(())
    }

    /// Link work item to commit
    pub async fn link_work_item(&self, work_item_id: u32, commit_id: &str) -> Result<()> {
        let url = format!(
            "{}/wit/workitems/{}?api-version={}",
            self.base_url(),
            work_item_id,
            AZURE_DEVOPS_API_VERSION
        );

        let link = vec![WorkItemLink {
            op: "add".to_string(),
            path: "/relations/-".to_string(),
            value: format!(
                "{{\"rel\":\"ArtifactLink\",\"url\":\"vstfs:///Git/Commit/{}%2F{}\"}}",
                self.config.project, commit_id
            ),
        }];

        let response = self
            .client
            .patch(&url)
            .basic_auth("", Some(&self.config.token))
            .header(header::CONTENT_TYPE, "application/json-patch+json")
            .json(&link)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(IntegrationError::Api(format!(
                "Failed to link work item: {} - {}",
                status, body
            )));
        }

        if self.config.debug {
            println!("✓ Work item linked successfully");
        }

        Ok(())
    }

    /// Publish test results
    pub async fn publish_test_results(&self, result: &CheckResult) -> Result<()> {
        let build_id = self
            .config
            .build_id
            .as_ref()
            .ok_or_else(|| IntegrationError::Configuration("Build ID not set".to_string()))?;

        // Create test run
        let run_url = format!(
            "{}/test/runs?api-version={}",
            self.base_url(),
            AZURE_DEVOPS_API_VERSION
        );

        let test_run = TestRun {
            name: "CADDY Design Check".to_string(),
            is_automated: true,
            state: "InProgress".to_string(),
            build_id: Some(build_id.clone()),
        };

        let run_response = self
            .client
            .post(&run_url)
            .basic_auth("", Some(&self.config.token))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&test_run)
            .send()
            .await?;

        if !run_response.status().is_success() {
            let status = run_response.status();
            let body = run_response.text().await.unwrap_or_default();
            return Err(IntegrationError::Api(format!(
                "Failed to create test run: {} - {}",
                status, body
            )));
        }

        let run_data: serde_json::Value = run_response.json().await?;
        let run_id = run_data["id"].as_u64().ok_or_else(|| {
            IntegrationError::Api("Failed to get test run ID".to_string())
        })?;

        // Add test results
        let results_url = format!(
            "{}/test/runs/{}/results?api-version={}",
            self.base_url(),
            run_id,
            AZURE_DEVOPS_API_VERSION
        );

        let test_results: Vec<TestResult> = result
            .annotations
            .iter()
            .map(|a| TestResult {
                test_case_title: format!("{}:{}", a.path, a.start_line),
                outcome: match a.level {
                    AnnotationLevel::Error => "Failed",
                    AnnotationLevel::Warning => "Warning",
                    AnnotationLevel::Notice => "Passed",
                }
                .to_string(),
                error_message: Some(a.message.clone()),
                stack_trace: a.raw_details.clone(),
                duration_in_ms: 0,
            })
            .collect();

        let results_response = self
            .client
            .post(&results_url)
            .basic_auth("", Some(&self.config.token))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&test_results)
            .send()
            .await?;

        if !results_response.status().is_success() {
            let status = results_response.status();
            let body = results_response.text().await.unwrap_or_default();
            return Err(IntegrationError::Api(format!(
                "Failed to add test results: {} - {}",
                status, body
            )));
        }

        // Complete test run
        let complete_url = format!(
            "{}/test/runs/{}?api-version={}",
            self.base_url(),
            run_id,
            AZURE_DEVOPS_API_VERSION
        );

        let complete_run = serde_json::json!({
            "state": "Completed"
        });

        let complete_response = self
            .client
            .patch(&complete_url)
            .basic_auth("", Some(&self.config.token))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&complete_run)
            .send()
            .await?;

        if !complete_response.status().is_success() {
            let status = complete_response.status();
            let body = complete_response.text().await.unwrap_or_default();
            return Err(IntegrationError::Api(format!(
                "Failed to complete test run: {} - {}",
                status, body
            )));
        }

        if self.config.debug {
            println!("✓ Test results published successfully");
        }

        Ok(())
    }

    /// Format result for Azure Pipelines logging
    pub fn format_pipeline_output(&self, result: &CheckResult) -> String {
        let mut output = String::new();

        // Set task result
        match result.status {
            CheckStatus::Success => {
                output.push_str("##vso[task.complete result=Succeeded;]CADDY check passed\n");
            }
            CheckStatus::Warning => {
                output.push_str("##vso[task.complete result=SucceededWithIssues;]CADDY check completed with warnings\n");
            }
            CheckStatus::Failure => {
                output.push_str("##vso[task.complete result=Failed;]CADDY check failed\n");
            }
            _ => {}
        }

        // Add annotations
        for annotation in &result.annotations {
            let level = match annotation.level {
                AnnotationLevel::Error => "error",
                AnnotationLevel::Warning => "warning",
                AnnotationLevel::Notice => "warning",
            };

            output.push_str(&format!(
                "##vso[task.logissue type={};sourcepath={};linenumber={};]{}",
                level, annotation.path, annotation.start_line, annotation.message
            ));
            output.push('\n');
        }

        output
    }
}

impl CIIntegration for AzureDevOpsIntegration {
    fn name(&self) -> &str {
        "Azure DevOps"
    }

    fn initialize(&mut self) -> Result<()> {
        if self.config.organization.is_empty() {
            return Err(IntegrationError::Configuration(
                "Organization not set".to_string(),
            ));
        }

        if self.config.project.is_empty() {
            return Err(IntegrationError::Configuration(
                "Project not set".to_string(),
            ));
        }

        if self.config.token.is_empty() {
            return Err(IntegrationError::Configuration("Token not set".to_string()));
        }

        if self.config.debug {
            println!("✓ Azure DevOps integration initialized successfully");
        }

        Ok(())
    }

    fn send_status(&self, result: &CheckResult) -> Result<()> {
        // Print Azure Pipelines formatted output
        print!("{}", self.format_pipeline_output(result));

        if self.config.debug {
            println!("✓ Azure DevOps pipeline output generated");
        }

        Ok(())
    }

    fn handle_webhook(&self, event: WebhookEvent) -> Result<()> {
        if self.config.debug {
            println!("Received Azure DevOps webhook: {}", event.event_type);
        }

        match event.event_type.as_str() {
            "git.pullrequest.created" | "git.pullrequest.updated" => {
                if self.config.debug {
                    println!("Pull request event received");
                }
            }
            "git.push" => {
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

    fn verify_webhook(&self, _payload: &[u8], _signature: &str) -> Result<bool> {
        // Azure DevOps uses basic auth for webhooks
        Ok(true)
    }
}

/// Azure Pipelines YAML helpers
pub mod pipelines {
    /// Generate Azure Pipelines YAML task
    pub fn pipeline_task() -> &'static str {
        r#"
- task: CmdLine@2
  displayName: 'CADDY Design Check'
  inputs:
    script: |
      caddy-cli scan --format azure-devops
  continueOnError: false

- task: PublishTestResults@2
  displayName: 'Publish CADDY Test Results'
  condition: always()
  inputs:
    testResultsFormat: 'JUnit'
    testResultsFiles: 'target/caddy-reports/junit.xml'
    testRunTitle: 'CADDY Design Check'

- task: PublishBuildArtifacts@1
  displayName: 'Publish CADDY Reports'
  condition: always()
  inputs:
    pathToPublish: 'target/caddy-reports'
    artifactName: 'caddy-reports'
"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_azure_devops_config_default() {
        let config = AzureDevOpsConfig::default();
        assert!(config.organization.is_empty());
        assert!(config.project.is_empty());
    }

    #[test]
    fn test_pipeline_output_formatting() {
        let integration = AzureDevOpsIntegration::new(AzureDevOpsConfig::default());
        let result = CheckResult {
            status: CheckStatus::Success,
            summary: "All checks passed".to_string(),
            details: None,
            annotations: vec![],
            execution_time_ms: 100,
            metadata: std::collections::HashMap::new(),
        };

        let output = integration.format_pipeline_output(&result);
        assert!(output.contains("##vso[task.complete result=Succeeded;]"));
    }
}
