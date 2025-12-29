//! # CLI Tool for CI/CD Integration
//!
//! Standalone CLI scanner for CADDY with CI-friendly output formats.
//!
//! ## Features
//!
//! - Multiple output formats (JSON, JUnit XML, SARIF, plain text)
//! - Configurable exit codes
//! - Threshold settings for warnings/errors
//! - CI platform detection and automatic formatting
//! - Colored terminal output
//! - Progress indicators
//!
//! ## Usage
//!
//! ```bash
//! # Basic scan
//! caddy-cli scan
//!
//! # Scan with specific format
//! caddy-cli scan --format json --output results.json
//!
//! # Scan with thresholds
//! caddy-cli scan --max-warnings 10 --fail-on-error
//!
//! # Platform-specific output
//! caddy-cli scan --format github-actions
//! caddy-cli scan --format gitlab-ci
//! caddy-cli scan --format azure-devops
//! ```
//!
//! ## Exit Codes
//!
//! - 0: Success (no errors)
//! - 1: Warning (warnings found but under threshold)
//! - 2: Failure (errors found or threshold exceeded)
//! - 3: Configuration error
//! - 4: Runtime error

use super::{Annotation, AnnotationLevel, CheckResult, CheckStatus, IntegrationError, Result};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};

/// CLI output format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable text
    Text,

    /// JSON format
    Json,

    /// Pretty-printed JSON
    JsonPretty,

    /// JUnit XML format
    JunitXml,

    /// SARIF format (Static Analysis Results Interchange Format)
    Sarif,

    /// GitHub Actions format
    GitHubActions,

    /// GitLab CI format
    GitLabCI,

    /// Azure DevOps format
    AzureDevOps,

    /// Bitbucket Pipelines format
    Bitbucket,

    /// Jenkins format
    Jenkins,
}

impl OutputFormat {
    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "text" | "plain" => Some(Self::Text),
            "json" => Some(Self::Json),
            "json-pretty" => Some(Self::JsonPretty),
            "junit" | "junit-xml" => Some(Self::JunitXml),
            "sarif" => Some(Self::Sarif),
            "github" | "github-actions" => Some(Self::GitHubActions),
            "gitlab" | "gitlab-ci" => Some(Self::GitLabCI),
            "azure" | "azure-devops" => Some(Self::AzureDevOps),
            "bitbucket" => Some(Self::Bitbucket),
            "jenkins" => Some(Self::Jenkins),
            _ => None,
        }
    }
}

/// CLI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CLIConfig {
    /// Output format
    pub format: OutputFormat,

    /// Output file path (None = stdout)
    pub output: Option<PathBuf>,

    /// Maximum warnings before failing
    pub max_warnings: Option<usize>,

    /// Maximum errors before failing
    pub max_errors: Option<usize>,

    /// Fail on any error
    pub fail_on_error: bool,

    /// Fail on any warning
    pub fail_on_warning: bool,

    /// Enable colored output
    pub color: bool,

    /// Verbose output
    pub verbose: bool,

    /// Quiet mode (suppress non-error output)
    pub quiet: bool,

    /// Scan path
    pub scan_path: PathBuf,
}

impl Default for CLIConfig {
    fn default() -> Self {
        Self {
            format: OutputFormat::Text,
            output: None,
            max_warnings: None,
            max_errors: None,
            fail_on_error: true,
            fail_on_warning: false,
            color: true,
            verbose: false,
            quiet: false,
            scan_path: PathBuf::from("."),
        }
    }
}

/// CLI formatter
pub struct CLIFormatter {
    config: CLIConfig,
}

impl CLIFormatter {
    /// Create a new CLI formatter
    pub fn new(config: CLIConfig) -> Self {
        Self { config }
    }

    /// Format and output the check result
    pub fn format_and_output(&self, result: &CheckResult) -> Result<()> {
        let output = match self.config.format {
            OutputFormat::Text => self.format_text(result),
            OutputFormat::Json => self.format_json(result, false)?,
            OutputFormat::JsonPretty => self.format_json(result, true)?,
            OutputFormat::JunitXml => self.format_junit_xml(result)?,
            OutputFormat::Sarif => self.format_sarif(result)?,
            OutputFormat::GitHubActions => self.format_github_actions(result),
            OutputFormat::GitLabCI => self.format_gitlab_ci(result),
            OutputFormat::AzureDevOps => self.format_azure_devops(result),
            OutputFormat::Bitbucket => self.format_bitbucket(result),
            OutputFormat::Jenkins => self.format_jenkins(result),
        };

        self.write_output(&output)?;

        Ok(())
    }

    /// Write output to file or stdout
    fn write_output(&self, content: &str) -> Result<()> {
        if let Some(output_path) = &self.config.output {
            std::fs::write(output_path, content)?;
            if !self.config.quiet {
                eprintln!("Output written to: {}", output_path.display());
            }
        } else {
            println!("{}", content);
        }

        Ok(())
    }

    /// Format as human-readable text
    fn format_text(&self, result: &CheckResult) -> String {
        let mut output = String::new();

        // Header
        let status_symbol = match result.status {
            CheckStatus::Success => if self.config.color { "\x1b[32m✓\x1b[0m" } else { "✓" },
            CheckStatus::Warning => if self.config.color { "\x1b[33m⚠\x1b[0m" } else { "⚠" },
            CheckStatus::Failure => if self.config.color { "\x1b[31m✗\x1b[0m" } else { "✗" },
            _ => "•",
        };

        output.push_str(&format!(
            "{} CADDY Design Check: {}\n\n",
            status_symbol, result.summary
        ));

        // Statistics
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

        output.push_str(&format!(
            "Summary: {} errors, {} warnings, {} notices\n",
            errors, warnings, notices
        ));
        output.push_str(&format!(
            "Execution time: {}\n\n",
            super::utils::format_duration(result.execution_time_ms)
        ));

        // Annotations
        if !result.annotations.is_empty() {
            output.push_str("Issues:\n\n");

            for annotation in &result.annotations {
                let level_prefix = match annotation.level {
                    AnnotationLevel::Error => {
                        if self.config.color {
                            "\x1b[31mError\x1b[0m"
                        } else {
                            "Error"
                        }
                    }
                    AnnotationLevel::Warning => {
                        if self.config.color {
                            "\x1b[33mWarning\x1b[0m"
                        } else {
                            "Warning"
                        }
                    }
                    AnnotationLevel::Notice => {
                        if self.config.color {
                            "\x1b[36mNotice\x1b[0m"
                        } else {
                            "Notice"
                        }
                    }
                };

                output.push_str(&format!(
                    "  {} {}:{}\n",
                    level_prefix, annotation.path, annotation.start_line
                ));
                output.push_str(&format!("    {}\n\n", annotation.message));
            }
        }

        output
    }

    /// Format as JSON
    fn format_json(&self, result: &CheckResult, pretty: bool) -> Result<String> {
        if pretty {
            Ok(serde_json::to_string_pretty(result)?)
        } else {
            Ok(serde_json::to_string(result)?)
        }
    }

    /// Format as JUnit XML
    fn format_junit_xml(&self, result: &CheckResult) -> Result<String> {
        // Reuse Jenkins integration's JUnit XML generation
        use super::jenkins::{JenkinsConfig, JenkinsIntegration};

        let jenkins_config = JenkinsConfig {
            output_dir: self.config.scan_path.clone(),
            ..Default::default()
        };

        let jenkins = JenkinsIntegration::new(jenkins_config);
        jenkins.generate_junit_xml(result)
    }

    /// Format as SARIF (Static Analysis Results Interchange Format)
    fn format_sarif(&self, result: &CheckResult) -> Result<String> {
        let sarif = serde_json::json!({
            "version": "2.1.0",
            "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
            "runs": [{
                "tool": {
                    "driver": {
                        "name": "CADDY",
                        "version": env!("CARGO_PKG_VERSION"),
                        "informationUri": "https://github.com/caddy-cad/caddy",
                        "rules": self.generate_sarif_rules(result)
                    }
                },
                "results": self.generate_sarif_results(result)
            }]
        });

        Ok(serde_json::to_string_pretty(&sarif)?)
    }

    /// Generate SARIF rules
    fn generate_sarif_rules(&self, result: &CheckResult) -> Vec<serde_json::Value> {
        let mut rules = Vec::new();
        let mut rule_ids = std::collections::HashSet::new();

        for annotation in &result.annotations {
            let rule_id = format!("CADDY/{:?}", annotation.level);
            if rule_ids.insert(rule_id.clone()) {
                rules.push(serde_json::json!({
                    "id": rule_id,
                    "name": format!("{:?}", annotation.level),
                    "shortDescription": {
                        "text": format!("CADDY {:?}", annotation.level)
                    }
                }));
            }
        }

        rules
    }

    /// Generate SARIF results
    fn generate_sarif_results(&self, result: &CheckResult) -> Vec<serde_json::Value> {
        result
            .annotations
            .iter()
            .map(|annotation| {
                let level = match annotation.level {
                    AnnotationLevel::Error => "error",
                    AnnotationLevel::Warning => "warning",
                    AnnotationLevel::Notice => "note",
                };

                serde_json::json!({
                    "ruleId": format!("CADDY/{:?}", annotation.level),
                    "level": level,
                    "message": {
                        "text": annotation.message.clone()
                    },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": annotation.path.clone()
                            },
                            "region": {
                                "startLine": annotation.start_line,
                                "endLine": annotation.end_line,
                                "startColumn": annotation.start_column,
                                "endColumn": annotation.end_column
                            }
                        }
                    }]
                })
            })
            .collect()
    }

    /// Format for GitHub Actions
    fn format_github_actions(&self, result: &CheckResult) -> String {
        let mut output = String::new();

        for annotation in &result.annotations {
            let level = match annotation.level {
                AnnotationLevel::Error => "error",
                AnnotationLevel::Warning => "warning",
                AnnotationLevel::Notice => "notice",
            };

            output.push_str(&format!(
                "::{} file={},line={}::{}\n",
                level, annotation.path, annotation.start_line, annotation.message
            ));
        }

        output.push_str(&format!("::set-output name=status::{:?}\n", result.status));
        output.push_str(&format!(
            "::set-output name=errors::{}\n",
            result
                .annotations
                .iter()
                .filter(|a| a.level == AnnotationLevel::Error)
                .count()
        ));
        output.push_str(&format!(
            "::set-output name=warnings::{}\n",
            result
                .annotations
                .iter()
                .filter(|a| a.level == AnnotationLevel::Warning)
                .count()
        ));

        output
    }

    /// Format for GitLab CI
    fn format_gitlab_ci(&self, result: &CheckResult) -> String {
        // GitLab CI uses standard text output with special formatting
        self.format_text(result)
    }

    /// Format for Azure DevOps
    fn format_azure_devops(&self, result: &CheckResult) -> String {
        use super::azure_devops::{AzureDevOpsConfig, AzureDevOpsIntegration};

        let azure = AzureDevOpsIntegration::new(AzureDevOpsConfig::default());
        azure.format_pipeline_output(result)
    }

    /// Format for Bitbucket
    fn format_bitbucket(&self, result: &CheckResult) -> String {
        // Bitbucket Pipelines uses standard text output
        self.format_text(result)
    }

    /// Format for Jenkins
    fn format_jenkins(&self, result: &CheckResult) -> String {
        // Jenkins uses standard text output
        self.format_text(result)
    }

    /// Determine exit code based on result and configuration
    pub fn determine_exit_code(&self, result: &CheckResult) -> i32 {
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

        // Check error threshold
        if errors > 0 && self.config.fail_on_error {
            return 2;
        }

        if let Some(max_errors) = self.config.max_errors {
            if errors > max_errors {
                return 2;
            }
        }

        // Check warning threshold
        if warnings > 0 && self.config.fail_on_warning {
            return 1;
        }

        if let Some(max_warnings) = self.config.max_warnings {
            if warnings > max_warnings {
                return 1;
            }
        }

        // Success
        0
    }
}

/// Detect CI platform from environment variables
pub fn detect_ci_platform() -> Option<OutputFormat> {
    if std::env::var("GITHUB_ACTIONS").is_ok() {
        Some(OutputFormat::GitHubActions)
    } else if std::env::var("GITLAB_CI").is_ok() {
        Some(OutputFormat::GitLabCI)
    } else if std::env::var("AZURE_HTTP_USER_AGENT").is_ok()
        || std::env::var("TF_BUILD").is_ok()
    {
        Some(OutputFormat::AzureDevOps)
    } else if std::env::var("BITBUCKET_BUILD_NUMBER").is_ok() {
        Some(OutputFormat::Bitbucket)
    } else if std::env::var("JENKINS_HOME").is_ok() || std::env::var("JENKINS_URL").is_ok() {
        Some(OutputFormat::Jenkins)
    } else {
        None
    }
}

/// Progress reporter for CLI
pub struct ProgressReporter {
    quiet: bool,
    total_files: usize,
    processed_files: usize,
}

impl ProgressReporter {
    /// Create a new progress reporter
    pub fn new(quiet: bool, total_files: usize) -> Self {
        Self {
            quiet,
            total_files,
            processed_files: 0,
        }
    }

    /// Update progress
    pub fn update(&mut self, files_processed: usize) {
        self.processed_files = files_processed;
        if !self.quiet && self.total_files > 0 {
            eprint!(
                "\rScanning: {}/{} files ({:.1}%)",
                self.processed_files,
                self.total_files,
                (self.processed_files as f64 / self.total_files as f64) * 100.0
            );
            std::io::stderr().flush().ok();
        }
    }

    /// Mark as complete
    pub fn complete(&self) {
        if !self.quiet {
            eprintln!("\rScan complete: {} files processed", self.total_files);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_output_format_from_str() {
        assert_eq!(
            OutputFormat::from_str("json"),
            Some(OutputFormat::Json)
        );
        assert_eq!(
            OutputFormat::from_str("junit-xml"),
            Some(OutputFormat::JunitXml)
        );
        assert_eq!(
            OutputFormat::from_str("github-actions"),
            Some(OutputFormat::GitHubActions)
        );
        assert_eq!(OutputFormat::from_str("invalid"), None);
    }

    #[test]
    fn test_exit_code_determination() {
        let config = CLIConfig::default();
        let formatter = CLIFormatter::new(config);

        let result_success = CheckResult {
            status: CheckStatus::Success,
            summary: "Success".to_string(),
            details: None,
            annotations: vec![],
            execution_time_ms: 100,
            metadata: HashMap::new(),
        };

        assert_eq!(formatter.determine_exit_code(&result_success), 0);

        let result_error = CheckResult {
            status: CheckStatus::Failure,
            summary: "Failed".to_string(),
            details: None,
            annotations: vec![Annotation {
                path: "test.rs".to_string(),
                start_line: 1,
                end_line: 1,
                start_column: None,
                end_column: None,
                level: AnnotationLevel::Error,
                message: "Error".to_string(),
                title: None,
                raw_details: None,
            }],
            execution_time_ms: 100,
            metadata: HashMap::new(),
        };

        assert_eq!(formatter.determine_exit_code(&result_error), 2);
    }

    #[test]
    fn test_json_formatting() {
        let config = CLIConfig {
            format: OutputFormat::Json,
            ..Default::default()
        };
        let formatter = CLIFormatter::new(config);

        let result = CheckResult {
            status: CheckStatus::Success,
            summary: "Test".to_string(),
            details: None,
            annotations: vec![],
            execution_time_ms: 100,
            metadata: HashMap::new(),
        };

        let output = formatter.format_json(&result, false).unwrap();
        assert!(output.contains("\"summary\":\"Test\""));
    }
}
