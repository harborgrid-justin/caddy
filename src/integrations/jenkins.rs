//! # Jenkins Integration
//!
//! Comprehensive Jenkins integration for CADDY, supporting:
//! - Jenkins plugin integration
//! - Build step execution
//! - Results publishing (JUnit XML, HTML reports)
//! - Pipeline support (Declarative and Scripted)
//! - Blue Ocean integration
//! - Build annotations
//!
//! ## Setup
//!
//! ### Jenkins Plugin (Recommended)
//! 1. Install the CADDY Jenkins plugin
//! 2. Configure in job configuration:
//!    ```groovy
//!    stage('CADDY Check') {
//!        steps {
//!            caddyCheck()
//!        }
//!    }
//!    ```
//!
//! ### CLI Integration
//! ```groovy
//! stage('CADDY Check') {
//!     steps {
//!         sh 'caddy-cli scan --format junit --output results.xml'
//!         junit 'results.xml'
//!     }
//! }
//! ```
//!
//! ## API Usage
//!
//! ```rust
//! use caddy::integrations::jenkins::{JenkinsIntegration, JenkinsConfig};
//!
//! let config = JenkinsConfig {
//!     url: "https://jenkins.example.com".to_string(),
//!     user: "admin".to_string(),
//!     token: "your-api-token".to_string(),
//!     ..Default::default()
//! };
//!
//! let mut jenkins = JenkinsIntegration::new(config);
//! jenkins.initialize()?;
//! ```

use super::{
    Annotation, AnnotationLevel, CheckResult, CheckStatus, CIIntegration, IntegrationError,
    Result, WebhookEvent,
};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Jenkins-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JenkinsConfig {
    /// Jenkins instance URL
    pub url: String,

    /// Jenkins user
    pub user: String,

    /// API token or password
    pub token: String,

    /// Job name
    pub job_name: Option<String>,

    /// Build number
    pub build_number: Option<String>,

    /// Enable debug logging
    pub debug: bool,

    /// Results output directory
    pub output_dir: PathBuf,
}

impl Default for JenkinsConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            user: String::new(),
            token: String::new(),
            job_name: None,
            build_number: None,
            debug: false,
            output_dir: PathBuf::from("target/caddy-reports"),
        }
    }
}

/// Jenkins integration implementation
pub struct JenkinsIntegration {
    config: JenkinsConfig,
    client: Client,
}

/// JUnit XML test suite format (Jenkins-compatible)
#[derive(Debug, Serialize, Deserialize)]
struct JUnitTestSuite {
    #[serde(rename = "@name")]
    name: String,

    #[serde(rename = "@tests")]
    tests: usize,

    #[serde(rename = "@failures")]
    failures: usize,

    #[serde(rename = "@errors")]
    errors: usize,

    #[serde(rename = "@time")]
    time: f64,

    #[serde(rename = "@timestamp")]
    timestamp: String,

    #[serde(rename = "testcase", default)]
    testcases: Vec<JUnitTestCase>,
}

/// JUnit test case
#[derive(Debug, Serialize, Deserialize)]
struct JUnitTestCase {
    #[serde(rename = "@name")]
    name: String,

    #[serde(rename = "@classname")]
    classname: String,

    #[serde(rename = "@time")]
    time: f64,

    #[serde(rename = "failure", skip_serializing_if = "Option::is_none")]
    failure: Option<JUnitFailure>,

    #[serde(rename = "error", skip_serializing_if = "Option::is_none")]
    error: Option<JUnitError>,

    #[serde(rename = "system-out", skip_serializing_if = "Option::is_none")]
    system_out: Option<String>,
}

/// JUnit failure
#[derive(Debug, Serialize, Deserialize)]
struct JUnitFailure {
    #[serde(rename = "@message")]
    message: String,

    #[serde(rename = "@type")]
    failure_type: String,

    #[serde(rename = "$text")]
    text: String,
}

/// JUnit error
#[derive(Debug, Serialize, Deserialize)]
struct JUnitError {
    #[serde(rename = "@message")]
    message: String,

    #[serde(rename = "@type")]
    error_type: String,

    #[serde(rename = "$text")]
    text: String,
}

/// Build status request
#[derive(Debug, Serialize, Deserialize)]
struct BuildStatusRequest {
    status: String,
    description: String,
}

impl JenkinsIntegration {
    /// Create a new Jenkins integration
    pub fn new(config: JenkinsConfig) -> Self {
        let client = Client::builder()
            .user_agent(format!("CADDY-CI/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    /// Generate JUnit XML report
    pub fn generate_junit_xml(&self, result: &CheckResult) -> Result<String> {
        let test_suite = self.create_test_suite(result);

        let xml = quick_xml::se::to_string(&test_suite)
            .map_err(|e| IntegrationError::Serialization(serde_json::Error::custom(e)))?;

        Ok(format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
{}"#,
            xml
        ))
    }

    /// Create JUnit test suite from check result
    fn create_test_suite(&self, result: &CheckResult) -> JUnitTestSuite {
        let mut testcases = Vec::new();
        let mut failures = 0;
        let mut errors = 0;

        // Group annotations by file
        let mut file_annotations: std::collections::HashMap<String, Vec<&Annotation>> =
            std::collections::HashMap::new();

        for annotation in &result.annotations {
            file_annotations
                .entry(annotation.path.clone())
                .or_default()
                .push(annotation);
        }

        // Create test cases for each file
        for (file, annotations) in file_annotations {
            for (_idx, annotation) in annotations.iter().enumerate() {
                let test_name = format!("{}:{}", file, annotation.start_line);
                let classname = file.replace('/', ".").replace(".rs", "");

                let (failure, error) = match annotation.level {
                    AnnotationLevel::Error => {
                        errors += 1;
                        (
                            None,
                            Some(JUnitError {
                                message: annotation.message.clone(),
                                error_type: "DesignError".to_string(),
                                text: annotation
                                    .raw_details
                                    .clone()
                                    .unwrap_or_else(|| annotation.message.clone()),
                            }),
                        )
                    }
                    AnnotationLevel::Warning => {
                        failures += 1;
                        (
                            Some(JUnitFailure {
                                message: annotation.message.clone(),
                                failure_type: "DesignWarning".to_string(),
                                text: annotation
                                    .raw_details
                                    .clone()
                                    .unwrap_or_else(|| annotation.message.clone()),
                            }),
                            None,
                        )
                    }
                    AnnotationLevel::Notice => (None, None),
                };

                testcases.push(JUnitTestCase {
                    name: test_name,
                    classname,
                    time: 0.0,
                    failure,
                    error,
                    system_out: annotation.title.clone(),
                });
            }
        }

        // Add overall summary test case
        testcases.push(JUnitTestCase {
            name: "Overall Check".to_string(),
            classname: "CADDY".to_string(),
            time: result.execution_time_ms as f64 / 1000.0,
            failure: if result.status == CheckStatus::Failure {
                Some(JUnitFailure {
                    message: result.summary.clone(),
                    failure_type: "CheckFailed".to_string(),
                    text: result.details.clone().unwrap_or_default(),
                })
            } else {
                None
            },
            error: None,
            system_out: Some(result.summary.clone()),
        });

        JUnitTestSuite {
            name: "CADDY Design Check".to_string(),
            tests: testcases.len(),
            failures,
            errors,
            time: result.execution_time_ms as f64 / 1000.0,
            timestamp: chrono::Utc::now().to_rfc3339(),
            testcases,
        }
    }

    /// Generate HTML report
    pub fn generate_html_report(&self, result: &CheckResult) -> String {
        let status_color = match result.status {
            CheckStatus::Success => "#4caf50",
            CheckStatus::Warning => "#ff9800",
            CheckStatus::Failure => "#f44336",
            _ => "#9e9e9e",
        };

        let status_text = match result.status {
            CheckStatus::Success => "Success",
            CheckStatus::Warning => "Warning",
            CheckStatus::Failure => "Failure",
            CheckStatus::InProgress => "In Progress",
            CheckStatus::Queued => "Queued",
            CheckStatus::Cancelled => "Cancelled",
            CheckStatus::Skipped => "Skipped",
        };

        let mut html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>CADDY Design Check Report</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 20px; background: #f5f5f5; }}
        .container {{ max-width: 1200px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .header {{ border-bottom: 2px solid #e0e0e0; padding-bottom: 20px; margin-bottom: 30px; }}
        .status {{ display: inline-block; padding: 8px 16px; border-radius: 4px; color: white; font-weight: bold; background: {}; }}
        .summary {{ font-size: 24px; margin: 20px 0; }}
        .metadata {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 15px; margin: 20px 0; }}
        .metadata-item {{ padding: 15px; background: #f9f9f9; border-radius: 4px; }}
        .metadata-label {{ font-size: 12px; color: #666; text-transform: uppercase; margin-bottom: 5px; }}
        .metadata-value {{ font-size: 18px; font-weight: bold; }}
        .annotations {{ margin-top: 30px; }}
        .annotation {{ margin: 10px 0; padding: 15px; border-left: 4px solid; border-radius: 4px; background: #f9f9f9; }}
        .annotation.error {{ border-color: #f44336; background: #ffebee; }}
        .annotation.warning {{ border-color: #ff9800; background: #fff3e0; }}
        .annotation.notice {{ border-color: #2196f3; background: #e3f2fd; }}
        .annotation-header {{ font-weight: bold; margin-bottom: 5px; }}
        .annotation-location {{ color: #666; font-size: 14px; font-family: monospace; }}
        .annotation-message {{ margin-top: 8px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>CADDY Design Check Report</h1>
            <div class="status">{}</div>
        </div>
        <div class="summary">{}</div>
"#,
            status_color, status_text, result.summary
        );

        // Metadata
        html.push_str(r#"<div class="metadata">"#);
        html.push_str(&format!(
            r#"<div class="metadata-item">
                <div class="metadata-label">Execution Time</div>
                <div class="metadata-value">{}</div>
            </div>"#,
            super::utils::format_duration(result.execution_time_ms)
        ));

        html.push_str(&format!(
            r#"<div class="metadata-item">
                <div class="metadata-label">Total Issues</div>
                <div class="metadata-value">{}</div>
            </div>"#,
            result.annotations.len()
        ));

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

        html.push_str(&format!(
            r#"<div class="metadata-item">
                <div class="metadata-label">Errors</div>
                <div class="metadata-value" style="color: #f44336;">{}</div>
            </div>"#,
            errors
        ));

        html.push_str(&format!(
            r#"<div class="metadata-item">
                <div class="metadata-label">Warnings</div>
                <div class="metadata-value" style="color: #ff9800;">{}</div>
            </div>"#,
            warnings
        ));

        html.push_str("</div>");

        // Annotations
        if !result.annotations.is_empty() {
            html.push_str(r#"<div class="annotations"><h2>Issues</h2>"#);

            for annotation in &result.annotations {
                let level_class = match annotation.level {
                    AnnotationLevel::Error => "error",
                    AnnotationLevel::Warning => "warning",
                    AnnotationLevel::Notice => "notice",
                };

                let level_text = match annotation.level {
                    AnnotationLevel::Error => "Error",
                    AnnotationLevel::Warning => "Warning",
                    AnnotationLevel::Notice => "Notice",
                };

                html.push_str(&format!(
                    r#"<div class="annotation {}">
                        <div class="annotation-header">{}: {}</div>
                        <div class="annotation-location">{}:{}</div>
                        <div class="annotation-message">{}</div>
                    </div>"#,
                    level_class,
                    level_text,
                    annotation.title.as_ref().unwrap_or(&"".to_string()),
                    annotation.path,
                    annotation.start_line,
                    annotation.message
                ));
            }

            html.push_str("</div>");
        }

        html.push_str(
            r#"
        <div style="margin-top: 40px; padding-top: 20px; border-top: 1px solid #e0e0e0; color: #666; font-size: 12px;">
            Generated by CADDY CI/CD Integration v"#,
        );
        html.push_str(env!("CARGO_PKG_VERSION"));
        html.push_str(
            r#"
        </div>
    </div>
</body>
</html>
"#,
        );

        html
    }

    /// Write report to output directory
    pub fn write_reports(&self, result: &CheckResult) -> Result<()> {
        std::fs::create_dir_all(&self.config.output_dir)?;

        // Write JUnit XML
        let junit_xml = self.generate_junit_xml(result)?;
        let junit_path = self.config.output_dir.join("junit.xml");
        std::fs::write(&junit_path, junit_xml)?;

        // Write HTML report
        let html_report = self.generate_html_report(result);
        let html_path = self.config.output_dir.join("report.html");
        std::fs::write(&html_path, html_report)?;

        // Write JSON report
        let json_report = serde_json::to_string_pretty(result)?;
        let json_path = self.config.output_dir.join("report.json");
        std::fs::write(&json_path, json_report)?;

        if self.config.debug {
            println!("✓ Reports written to {:?}", self.config.output_dir);
        }

        Ok(())
    }

    /// Update build status via Jenkins API
    pub async fn update_build_status(&self, result: &CheckResult) -> Result<()> {
        if self.config.job_name.is_none() || self.config.build_number.is_none() {
            return Err(IntegrationError::Configuration(
                "Job name and build number required".to_string(),
            ));
        }

        let job_name = self.config.job_name.as_ref().unwrap();
        let build_number = self.config.build_number.as_ref().unwrap();

        let url = format!(
            "{}/job/{}/{}/api/json",
            self.config.url, job_name, build_number
        );

        let status_request = BuildStatusRequest {
            status: format!("{:?}", result.status),
            description: result.summary.clone(),
        };

        let response = self
            .client
            .post(&url)
            .basic_auth(&self.config.user, Some(&self.config.token))
            .json(&status_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(IntegrationError::Api(format!(
                "Failed to update build status: {} - {}",
                status, body
            )));
        }

        if self.config.debug {
            println!("✓ Jenkins build status updated successfully");
        }

        Ok(())
    }
}

impl CIIntegration for JenkinsIntegration {
    fn name(&self) -> &str {
        "Jenkins"
    }

    fn initialize(&mut self) -> Result<()> {
        if self.config.url.is_empty() {
            return Err(IntegrationError::Configuration("URL not set".to_string()));
        }

        // Create output directory
        std::fs::create_dir_all(&self.config.output_dir)?;

        if self.config.debug {
            println!("✓ Jenkins integration initialized successfully");
        }

        Ok(())
    }

    fn send_status(&self, result: &CheckResult) -> Result<()> {
        // Write reports synchronously
        self.write_reports(result)?;

        if self.config.debug {
            println!("✓ Jenkins reports generated");
        }

        Ok(())
    }

    fn handle_webhook(&self, event: WebhookEvent) -> Result<()> {
        if self.config.debug {
            println!("Received Jenkins webhook: {}", event.event_type);
        }

        // Jenkins webhook handling
        match event.event_type.as_str() {
            "build" => {
                if self.config.debug {
                    println!("Build event received");
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
        // Jenkins typically doesn't use webhook signatures
        Ok(true)
    }
}

/// Pipeline helpers for Jenkinsfile
pub mod pipeline {
    /// Generate Declarative Pipeline snippet
    pub fn declarative_pipeline_snippet() -> &'static str {
        r#"
stage('CADDY Design Check') {
    steps {
        sh 'caddy-cli scan --format junit --output target/caddy-reports/junit.xml'
    }
    post {
        always {
            junit 'target/caddy-reports/junit.xml'
            publishHTML([
                allowMissing: false,
                alwaysLinkToLastBuild: true,
                keepAll: true,
                reportDir: 'target/caddy-reports',
                reportFiles: 'report.html',
                reportName: 'CADDY Design Check Report'
            ])
        }
    }
}
"#
    }

    /// Generate Scripted Pipeline snippet
    pub fn scripted_pipeline_snippet() -> &'static str {
        r#"
stage('CADDY Design Check') {
    try {
        sh 'caddy-cli scan --format junit --output target/caddy-reports/junit.xml'
    } finally {
        junit 'target/caddy-reports/junit.xml'
        publishHTML([
            allowMissing: false,
            alwaysLinkToLastBuild: true,
            keepAll: true,
            reportDir: 'target/caddy-reports',
            reportFiles: 'report.html',
            reportName: 'CADDY Design Check Report'
        ])
    }
}
"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jenkins_config_default() {
        let config = JenkinsConfig::default();
        assert_eq!(config.output_dir, PathBuf::from("target/caddy-reports"));
    }

    #[test]
    fn test_junit_xml_generation() {
        let integration = JenkinsIntegration::new(JenkinsConfig::default());
        let result = CheckResult {
            status: CheckStatus::Success,
            summary: "Test passed".to_string(),
            details: None,
            annotations: vec![],
            execution_time_ms: 100,
            metadata: std::collections::HashMap::new(),
        };

        let xml = integration.generate_junit_xml(&result).unwrap();
        assert!(xml.contains("<?xml version"));
        assert!(xml.contains("testsuite"));
    }

    #[test]
    fn test_html_report_generation() {
        let integration = JenkinsIntegration::new(JenkinsConfig::default());
        let result = CheckResult {
            status: CheckStatus::Success,
            summary: "All checks passed".to_string(),
            details: None,
            annotations: vec![],
            execution_time_ms: 1500,
            metadata: std::collections::HashMap::new(),
        };

        let html = integration.generate_html_report(&result);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("CADDY Design Check Report"));
        assert!(html.contains("All checks passed"));
    }
}
