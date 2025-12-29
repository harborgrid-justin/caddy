//! Analytics reporting and export
//!
//! This module provides scheduled reports, ad-hoc queries,
//! export capabilities, and trend analysis.

use super::{Result, AnalyticsError, MetricRegistry, aggregator::*};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Report format types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    /// PDF format
    Pdf,
    /// HTML format
    Html,
    /// CSV format
    Csv,
    /// JSON format
    Json,
    /// Markdown format
    Markdown,
    /// Excel format
    Excel,
}

impl ReportFormat {
    /// Get file extension
    pub fn extension(&self) -> &str {
        match self {
            Self::Pdf => "pdf",
            Self::Html => "html",
            Self::Csv => "csv",
            Self::Json => "json",
            Self::Markdown => "md",
            Self::Excel => "xlsx",
        }
    }

    /// Get MIME type
    pub fn mime_type(&self) -> &str {
        match self {
            Self::Pdf => "application/pdf",
            Self::Html => "text/html",
            Self::Csv => "text/csv",
            Self::Json => "application/json",
            Self::Markdown => "text/markdown",
            Self::Excel => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        }
    }
}

/// Report schedule
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportSchedule {
    /// One-time report
    Once,
    /// Daily at specific hour
    Daily { hour: u8 },
    /// Weekly on specific day and hour
    Weekly { day: u8, hour: u8 },
    /// Monthly on specific day and hour
    Monthly { day: u8, hour: u8 },
    /// Custom interval in seconds
    Custom(u64),
}

impl ReportSchedule {
    /// Check if report should run now
    pub fn should_run(&self, last_run: Option<u64>) -> bool {
        let now = current_timestamp();

        match self {
            Self::Once => last_run.is_none(),
            Self::Custom(interval) => {
                if let Some(last) = last_run {
                    now >= last + interval
                } else {
                    true
                }
            }
            Self::Daily { .. } | Self::Weekly { .. } | Self::Monthly { .. } => {
                // Simplified - in production, use proper date/time library
                if let Some(last) = last_run {
                    now >= last + 86400 // Daily for now
                } else {
                    true
                }
            }
        }
    }
}

/// Report section types
#[derive(Debug, Clone)]
pub enum ReportSection {
    /// Summary statistics
    Summary {
        metrics: Vec<String>,
        window: TimeWindow,
    },
    /// Time-series chart
    TimeSeries {
        metrics: Vec<String>,
        window: TimeWindow,
        aggregation: AggregationType,
    },
    /// Comparison table
    Comparison {
        metrics: Vec<String>,
        windows: Vec<TimeWindow>,
    },
    /// Top N items
    TopN {
        metric: String,
        n: usize,
        window: TimeWindow,
    },
    /// Trend analysis
    Trends {
        metrics: Vec<String>,
        window: TimeWindow,
    },
    /// Custom text
    Text {
        title: String,
        content: String,
    },
}

/// Aggregation type for time-series
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregationType {
    Mean,
    Sum,
    Min,
    Max,
    Median,
    P95,
    P99,
}

/// Report configuration
#[derive(Debug, Clone)]
pub struct ReportConfig {
    /// Report name
    pub name: String,
    /// Report description
    pub description: String,
    /// Report format
    pub format: ReportFormat,
    /// Report schedule
    pub schedule: ReportSchedule,
    /// Report sections
    pub sections: Vec<ReportSection>,
    /// Recipients (email addresses, etc.)
    pub recipients: Vec<String>,
    /// Include charts
    pub include_charts: bool,
    /// Include raw data
    pub include_raw_data: bool,
}

impl ReportConfig {
    /// Create a new report configuration
    pub fn new(name: impl Into<String>, format: ReportFormat) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            format,
            schedule: ReportSchedule::Once,
            sections: Vec::new(),
            recipients: Vec::new(),
            include_charts: true,
            include_raw_data: false,
        }
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set schedule
    pub fn schedule(mut self, schedule: ReportSchedule) -> Self {
        self.schedule = schedule;
        self
    }

    /// Add a section
    pub fn section(mut self, section: ReportSection) -> Self {
        self.sections.push(section);
        self
    }

    /// Add a recipient
    pub fn recipient(mut self, email: impl Into<String>) -> Self {
        self.recipients.push(email.into());
        self
    }

    /// Set include charts
    pub fn include_charts(mut self, include: bool) -> Self {
        self.include_charts = include;
        self
    }

    /// Set include raw data
    pub fn include_raw_data(mut self, include: bool) -> Self {
        self.include_raw_data = include;
        self
    }
}

/// Generated report
#[derive(Debug, Clone)]
pub struct Report {
    /// Report ID
    pub id: String,
    /// Report configuration
    pub config: ReportConfig,
    /// Generated at timestamp
    pub generated_at: u64,
    /// Report content
    pub content: String,
    /// File size in bytes
    pub size: usize,
}

impl Report {
    /// Create a new report
    pub fn new(config: ReportConfig, content: String) -> Self {
        let size = content.len();
        Self {
            id: format!("report_{}", current_timestamp()),
            config,
            generated_at: current_timestamp(),
            content,
            size,
        }
    }

    /// Get filename
    pub fn filename(&self) -> String {
        format!(
            "{}_{}.{}",
            self.config.name.replace(' ', "_").to_lowercase(),
            self.generated_at,
            self.config.format.extension()
        )
    }

    /// Save to file
    pub fn save(&self, path: &str) -> Result<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(path).map_err(|e| {
            AnalyticsError::IoError(format!("Failed to create file: {}", e))
        })?;

        file.write_all(self.content.as_bytes()).map_err(|e| {
            AnalyticsError::IoError(format!("Failed to write file: {}", e))
        })?;

        Ok(())
    }
}

/// Report generator
pub struct Reporter {
    registry: MetricRegistry,
    aggregators: HashMap<String, Aggregator>,
}

impl Reporter {
    /// Create a new reporter
    pub fn new(registry: MetricRegistry) -> Self {
        Self {
            registry,
            aggregators: HashMap::new(),
        }
    }

    /// Register an aggregator for a metric
    pub fn register_aggregator(&mut self, metric: impl Into<String>, aggregator: Aggregator) {
        self.aggregators.insert(metric.into(), aggregator);
    }

    /// Generate a report
    pub fn generate(&self, config: ReportConfig) -> Result<Report> {
        match config.format {
            ReportFormat::Csv => self.generate_csv(&config),
            ReportFormat::Json => self.generate_json(&config),
            ReportFormat::Html => self.generate_html(&config),
            ReportFormat::Markdown => self.generate_markdown(&config),
            ReportFormat::Pdf => Err(AnalyticsError::ReportError(
                "PDF generation not yet implemented".to_string(),
            )),
            ReportFormat::Excel => Err(AnalyticsError::ReportError(
                "Excel generation not yet implemented".to_string(),
            )),
        }
    }

    /// Generate CSV report
    fn generate_csv(&self, config: &ReportConfig) -> Result<Report> {
        let mut content = String::new();

        // Header
        content.push_str("Metric,Value,Timestamp\n");

        // Collect data from sections
        for section in &config.sections {
            match section {
                ReportSection::Summary { metrics, window } => {
                    for metric_name in metrics {
                        if let Some(agg) = self.aggregators.get(metric_name) {
                            let mean = agg.mean(Some(*window));
                            content.push_str(&format!(
                                "{},mean,{:.2}\n",
                                metric_name, mean
                            ));

                            let min = agg.min(Some(*window)).unwrap_or(0.0);
                            let max = agg.max(Some(*window)).unwrap_or(0.0);
                            content.push_str(&format!("{},min,{:.2}\n", metric_name, min));
                            content.push_str(&format!("{},max,{:.2}\n", metric_name, max));
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(Report::new(config.clone(), content))
    }

    /// Generate JSON report
    fn generate_json(&self, config: &ReportConfig) -> Result<Report> {
        let mut data = HashMap::new();

        data.insert("report_name", config.name.clone());
        data.insert("generated_at", current_timestamp().to_string());

        let mut sections = Vec::new();

        for section in &config.sections {
            match section {
                ReportSection::Summary { metrics, window } => {
                    let mut section_data = HashMap::new();
                    section_data.insert("type".to_string(), "summary".to_string());

                    for metric_name in metrics {
                        if let Some(agg) = self.aggregators.get(metric_name) {
                            let stats = StatisticalSummary::from_aggregator(agg, Some(*window));
                            let stats_str = format!(
                                r#"{{"mean":{:.2},"median":{:.2},"min":{:.2},"max":{:.2},"count":{}}}"#,
                                stats.mean, stats.median, stats.min, stats.max, stats.count
                            );
                            section_data.insert(metric_name.clone(), stats_str);
                        }
                    }

                    sections.push(format!("{:?}", section_data));
                }
                _ => {}
            }
        }

        let content = format!(
            r#"{{"report":"{}","generated_at":{},"sections_count":{}}}"#,
            config.name,
            current_timestamp(),
            sections.len()
        );

        Ok(Report::new(config.clone(), content))
    }

    /// Generate HTML report
    fn generate_html(&self, config: &ReportConfig) -> Result<Report> {
        let mut html = String::new();

        // HTML header
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str(&format!("<title>{}</title>\n", config.name));
        html.push_str("<style>\n");
        html.push_str("body { font-family: Arial, sans-serif; margin: 20px; }\n");
        html.push_str("h1 { color: #333; }\n");
        html.push_str("table { border-collapse: collapse; width: 100%; margin: 20px 0; }\n");
        html.push_str("th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
        html.push_str("th { background-color: #4CAF50; color: white; }\n");
        html.push_str("</style>\n");
        html.push_str("</head>\n<body>\n");

        // Report title
        html.push_str(&format!("<h1>{}</h1>\n", config.name));
        html.push_str(&format!("<p>{}</p>\n", config.description));
        html.push_str(&format!(
            "<p><em>Generated: {}</em></p>\n",
            format_timestamp(current_timestamp())
        ));

        // Sections
        for section in &config.sections {
            match section {
                ReportSection::Summary { metrics, window } => {
                    html.push_str("<h2>Summary Statistics</h2>\n");
                    html.push_str(&format!(
                        "<p>Time Window: {} seconds</p>\n",
                        window.as_secs()
                    ));
                    html.push_str("<table>\n");
                    html.push_str("<tr><th>Metric</th><th>Mean</th><th>Min</th><th>Max</th><th>Count</th></tr>\n");

                    for metric_name in metrics {
                        if let Some(agg) = self.aggregators.get(metric_name) {
                            let stats = StatisticalSummary::from_aggregator(agg, Some(*window));
                            html.push_str(&format!(
                                "<tr><td>{}</td><td>{:.2}</td><td>{:.2}</td><td>{:.2}</td><td>{}</td></tr>\n",
                                metric_name, stats.mean, stats.min, stats.max, stats.count
                            ));
                        }
                    }

                    html.push_str("</table>\n");
                }
                ReportSection::Text { title, content } => {
                    html.push_str(&format!("<h2>{}</h2>\n", title));
                    html.push_str(&format!("<p>{}</p>\n", content));
                }
                _ => {}
            }
        }

        // HTML footer
        html.push_str("</body>\n</html>");

        Ok(Report::new(config.clone(), html))
    }

    /// Generate Markdown report
    fn generate_markdown(&self, config: &ReportConfig) -> Result<Report> {
        let mut md = String::new();

        // Title
        md.push_str(&format!("# {}\n\n", config.name));
        md.push_str(&format!("{}\n\n", config.description));
        md.push_str(&format!(
            "*Generated: {}*\n\n",
            format_timestamp(current_timestamp())
        ));

        // Sections
        for section in &config.sections {
            match section {
                ReportSection::Summary { metrics, window } => {
                    md.push_str("## Summary Statistics\n\n");
                    md.push_str(&format!(
                        "**Time Window:** {} seconds\n\n",
                        window.as_secs()
                    ));
                    md.push_str("| Metric | Mean | Min | Max | Count |\n");
                    md.push_str("|--------|------|-----|-----|-------|\n");

                    for metric_name in metrics {
                        if let Some(agg) = self.aggregators.get(metric_name) {
                            let stats = StatisticalSummary::from_aggregator(agg, Some(*window));
                            md.push_str(&format!(
                                "| {} | {:.2} | {:.2} | {:.2} | {} |\n",
                                metric_name, stats.mean, stats.min, stats.max, stats.count
                            ));
                        }
                    }

                    md.push_str("\n");
                }
                ReportSection::Trends { metrics, window } => {
                    md.push_str("## Trend Analysis\n\n");

                    for metric_name in metrics {
                        if let Some(agg) = self.aggregators.get(metric_name) {
                            let trend = agg.trend(Some(*window));
                            let trend_str = match trend {
                                Trend::Increasing => "📈 Increasing",
                                Trend::Decreasing => "📉 Decreasing",
                                Trend::Stable => "➡️ Stable",
                            };
                            md.push_str(&format!("- **{}**: {}\n", metric_name, trend_str));
                        }
                    }

                    md.push_str("\n");
                }
                ReportSection::Text { title, content } => {
                    md.push_str(&format!("## {}\n\n", title));
                    md.push_str(&format!("{}\n\n", content));
                }
                _ => {}
            }
        }

        Ok(Report::new(config.clone(), md))
    }

    /// Create a quick summary report
    pub fn quick_summary(&self, metric: &str, window: TimeWindow) -> Result<String> {
        let agg = self.aggregators.get(metric).ok_or_else(|| {
            AnalyticsError::MetricNotFound(metric.to_string())
        })?;

        let stats = StatisticalSummary::from_aggregator(agg, Some(window));

        Ok(format!(
            r#"Metric: {}
Count: {}
Mean: {:.2}
Median: {:.2}
Min: {:.2}
Max: {:.2}
Std Dev: {:.2}
P50: {:.2}
P90: {:.2}
P95: {:.2}
P99: {:.2}"#,
            metric,
            stats.count,
            stats.mean,
            stats.median,
            stats.min,
            stats.max,
            stats.std_dev,
            stats.percentiles.p50,
            stats.percentiles.p90,
            stats.percentiles.p95,
            stats.percentiles.p99
        ))
    }
}

/// Pre-configured report templates
pub struct ReportTemplates;

impl ReportTemplates {
    /// Daily system performance report
    pub fn daily_system_performance() -> ReportConfig {
        ReportConfig::new("Daily System Performance", ReportFormat::Html)
            .description("Daily overview of system performance metrics")
            .schedule(ReportSchedule::Daily { hour: 9 })
            .section(ReportSection::Summary {
                metrics: vec![
                    "system_cpu_usage_percent".to_string(),
                    "system_memory_usage_bytes".to_string(),
                    "system_disk_usage_bytes".to_string(),
                ],
                window: TimeWindow::Day,
            })
            .section(ReportSection::Trends {
                metrics: vec![
                    "system_cpu_usage_percent".to_string(),
                    "system_memory_usage_bytes".to_string(),
                ],
                window: TimeWindow::Day,
            })
            .include_charts(true)
    }

    /// Weekly application report
    pub fn weekly_application_report() -> ReportConfig {
        ReportConfig::new("Weekly Application Report", ReportFormat::Pdf)
            .description("Weekly application performance and usage statistics")
            .schedule(ReportSchedule::Weekly { day: 1, hour: 9 })
            .section(ReportSection::Summary {
                metrics: vec![
                    "app_operations_total".to_string(),
                    "app_operations_failed".to_string(),
                    "app_operation_duration_seconds".to_string(),
                ],
                window: TimeWindow::Week,
            })
            .include_charts(true)
            .include_raw_data(false)
    }

    /// Monthly executive summary
    pub fn monthly_executive_summary() -> ReportConfig {
        ReportConfig::new("Monthly Executive Summary", ReportFormat::Pdf)
            .description("High-level monthly performance overview")
            .schedule(ReportSchedule::Monthly { day: 1, hour: 9 })
            .section(ReportSection::Text {
                title: "Executive Summary".to_string(),
                content: "This report provides a high-level overview of system performance and usage for the past month.".to_string(),
            })
            .section(ReportSection::Summary {
                metrics: vec![
                    "user_sessions_total".to_string(),
                    "user_actions_total".to_string(),
                    "app_operations_total".to_string(),
                ],
                window: TimeWindow::Custom(2592000), // 30 days
            })
            .include_charts(true)
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn format_timestamp(timestamp: u64) -> String {
    // Simplified - in production, use chrono or similar
    format!("Timestamp: {}", timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_format() {
        assert_eq!(ReportFormat::Csv.extension(), "csv");
        assert_eq!(ReportFormat::Html.extension(), "html");
        assert_eq!(ReportFormat::Json.extension(), "json");

        assert_eq!(ReportFormat::Csv.mime_type(), "text/csv");
        assert_eq!(ReportFormat::Json.mime_type(), "application/json");
    }

    #[test]
    fn test_report_schedule() {
        let schedule = ReportSchedule::Once;
        assert!(schedule.should_run(None));
        assert!(!schedule.should_run(Some(current_timestamp())));

        let schedule = ReportSchedule::Custom(3600);
        assert!(schedule.should_run(None));
        assert!(schedule.should_run(Some(current_timestamp() - 7200)));
        assert!(!schedule.should_run(Some(current_timestamp())));
    }

    #[test]
    fn test_report_config() {
        let config = ReportConfig::new("Test Report", ReportFormat::Html)
            .description("Test description")
            .schedule(ReportSchedule::Daily { hour: 9 })
            .recipient("admin@example.com")
            .include_charts(true);

        assert_eq!(config.name, "Test Report");
        assert_eq!(config.format, ReportFormat::Html);
        assert_eq!(config.recipients.len(), 1);
        assert!(config.include_charts);
    }

    #[test]
    fn test_reporter() {
        let registry = MetricRegistry::new();
        let reporter = Reporter::new(registry);

        let config = ReportConfig::new("Test", ReportFormat::Json);
        let result = reporter.generate(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_report_templates() {
        let daily = ReportTemplates::daily_system_performance();
        assert_eq!(daily.format, ReportFormat::Html);

        let weekly = ReportTemplates::weekly_application_report();
        assert_eq!(weekly.format, ReportFormat::Pdf);

        let monthly = ReportTemplates::monthly_executive_summary();
        assert_eq!(monthly.format, ReportFormat::Pdf);
    }

    #[test]
    fn test_report_filename() {
        let config = ReportConfig::new("Test Report", ReportFormat::Html);
        let report = Report::new(config, "<html></html>".to_string());

        let filename = report.filename();
        assert!(filename.contains("test_report"));
        assert!(filename.ends_with(".html"));
    }
}
