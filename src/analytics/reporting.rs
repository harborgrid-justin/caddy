//! # Report Generation
//!
//! Generate comprehensive analytics reports in various formats.

use super::{Result, AnalyticsError, UsageStats, ProfileReport};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Report format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportFormat {
    /// HTML format
    Html,
    /// PDF format (requires HTML conversion)
    Pdf,
    /// Markdown format
    Markdown,
    /// JSON format
    Json,
    /// CSV format
    Csv,
    /// Plain text format
    Text,
}

impl ReportFormat {
    /// Get file extension for the format
    pub fn extension(&self) -> &str {
        match self {
            Self::Html => "html",
            Self::Pdf => "pdf",
            Self::Markdown => "md",
            Self::Json => "json",
            Self::Csv => "csv",
            Self::Text => "txt",
        }
    }
}

/// Report type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportType {
    /// Usage statistics report
    Usage,
    /// Performance analysis report
    Performance,
    /// Error analysis report
    Errors,
    /// Custom report
    Custom,
    /// Executive summary
    ExecutiveSummary,
    /// Detailed analytics
    DetailedAnalytics,
}

/// Report section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    /// Section title
    pub title: String,

    /// Section content
    pub content: String,

    /// Section data (for charts/tables)
    pub data: Option<serde_json::Value>,

    /// Section type (text, chart, table, etc.)
    pub section_type: SectionType,

    /// Subsections
    pub subsections: Vec<ReportSection>,
}

/// Section type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SectionType {
    /// Text content
    Text,
    /// Chart visualization
    Chart,
    /// Data table
    Table,
    /// Key-value pairs
    KeyValue,
    /// List
    List,
}

/// Complete report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    /// Report ID
    pub id: String,

    /// Report type
    pub report_type: ReportType,

    /// Report title
    pub title: String,

    /// Report description
    pub description: Option<String>,

    /// Generation timestamp
    pub generated_at: DateTime<Utc>,

    /// Time range covered
    pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,

    /// Report sections
    pub sections: Vec<ReportSection>,

    /// Metadata
    pub metadata: HashMap<String, String>,
}

impl Report {
    /// Create a new report
    pub fn new(report_type: ReportType, title: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            report_type,
            title: title.into(),
            description: None,
            generated_at: Utc::now(),
            time_range: None,
            sections: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a section
    pub fn add_section(mut self, section: ReportSection) -> Self {
        self.sections.push(section);
        self
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set time range
    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.time_range = Some((start, end));
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Report generator
pub struct ReportGenerator {
    // Configuration or dependencies can go here
}

impl ReportGenerator {
    /// Create a new report generator
    pub fn new() -> Self {
        Self {}
    }

    /// Generate a usage report
    pub fn generate_usage_report(
        &self,
        stats: &UsageStats,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Report {
        let mut report = Report::new(ReportType::Usage, "Usage Analytics Report")
            .with_description("Comprehensive usage statistics and analytics")
            .with_time_range(start_time, end_time);

        // Summary section
        let summary = ReportSection {
            title: "Summary".to_string(),
            content: format!(
                "Total events: {}\nActive users: {}\nActive sessions: {}\nError rate: {:.2}%",
                stats.total_events,
                stats.active_users,
                stats.active_sessions,
                stats.error_rate * 100.0
            ),
            data: Some(serde_json::json!({
                "total_events": stats.total_events,
                "active_users": stats.active_users,
                "active_sessions": stats.active_sessions,
                "error_rate": stats.error_rate,
            })),
            section_type: SectionType::KeyValue,
            subsections: Vec::new(),
        };

        // Events by type section
        let events_by_type = ReportSection {
            title: "Events by Type".to_string(),
            content: Self::format_event_type_table(&stats.events_by_type),
            data: Some(serde_json::to_value(&stats.events_by_type).unwrap()),
            section_type: SectionType::Table,
            subsections: Vec::new(),
        };

        // Most used features section
        let features = ReportSection {
            title: "Most Used Features".to_string(),
            content: Self::format_usage_list(&stats.most_used_features),
            data: Some(serde_json::to_value(&stats.most_used_features).unwrap()),
            section_type: SectionType::Chart,
            subsections: Vec::new(),
        };

        // Most executed commands section
        let commands = ReportSection {
            title: "Most Executed Commands".to_string(),
            content: Self::format_usage_list(&stats.most_executed_commands),
            data: Some(serde_json::to_value(&stats.most_executed_commands).unwrap()),
            section_type: SectionType::Chart,
            subsections: Vec::new(),
        };

        // Session statistics
        let sessions = ReportSection {
            title: "Session Statistics".to_string(),
            content: format!(
                "Active sessions: {}\nTotal session duration: {} hours\nAverage session duration: {:.1} minutes",
                stats.active_sessions,
                stats.total_session_duration_secs / 3600,
                stats.avg_session_duration_secs / 60.0
            ),
            data: Some(serde_json::json!({
                "active_sessions": stats.active_sessions,
                "total_duration_secs": stats.total_session_duration_secs,
                "avg_duration_secs": stats.avg_session_duration_secs,
            })),
            section_type: SectionType::KeyValue,
            subsections: Vec::new(),
        };

        report = report
            .add_section(summary)
            .add_section(events_by_type)
            .add_section(features)
            .add_section(commands)
            .add_section(sessions);

        report
    }

    /// Generate a performance report
    pub fn generate_performance_report(
        &self,
        profiles: &[ProfileReport],
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Report {
        let mut report = Report::new(ReportType::Performance, "Performance Analysis Report")
            .with_description("Detailed performance profiling and analysis")
            .with_time_range(start_time, end_time);

        // Summary
        let total_calls: u64 = profiles.iter().map(|p| p.total_calls).sum();
        let avg_duration: f64 = if !profiles.is_empty() {
            profiles.iter().map(|p| p.avg_duration_ms).sum::<f64>() / profiles.len() as f64
        } else {
            0.0
        };

        let summary = ReportSection {
            title: "Summary".to_string(),
            content: format!(
                "Total operations profiled: {}\nTotal calls: {}\nAverage duration: {:.2} ms",
                profiles.len(),
                total_calls,
                avg_duration
            ),
            data: Some(serde_json::json!({
                "operations_count": profiles.len(),
                "total_calls": total_calls,
                "avg_duration_ms": avg_duration,
            })),
            section_type: SectionType::KeyValue,
            subsections: Vec::new(),
        };

        // Slowest operations
        let mut sorted_profiles = profiles.to_vec();
        sorted_profiles.sort_by(|a, b| {
            b.avg_duration_ms
                .partial_cmp(&a.avg_duration_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let slowest = sorted_profiles.iter().take(10).map(|p| {
            format!(
                "{}: avg={:.2}ms, max={:.2}ms, calls={}",
                p.operation_name, p.avg_duration_ms, p.max_duration_ms, p.total_calls
            )
        }).collect::<Vec<_>>().join("\n");

        let slowest_section = ReportSection {
            title: "Slowest Operations (Top 10)".to_string(),
            content: slowest,
            data: Some(serde_json::to_value(&sorted_profiles[..sorted_profiles.len().min(10)]).unwrap()),
            section_type: SectionType::Table,
            subsections: Vec::new(),
        };

        // Error rates
        let errors: Vec<_> = profiles
            .iter()
            .filter(|p| p.error_rate > 0.0)
            .map(|p| {
                format!(
                    "{}: {:.2}% error rate",
                    p.operation_name,
                    p.error_rate * 100.0
                )
            })
            .collect();

        let errors_content = if errors.is_empty() {
            "No errors detected".to_string()
        } else {
            errors.join("\n")
        };

        let errors_section = ReportSection {
            title: "Operations with Errors".to_string(),
            content: errors_content,
            data: Some(serde_json::to_value(
                &profiles
                    .iter()
                    .filter(|p| p.error_rate > 0.0)
                    .collect::<Vec<_>>()
            ).unwrap()),
            section_type: SectionType::List,
            subsections: Vec::new(),
        };

        report = report
            .add_section(summary)
            .add_section(slowest_section)
            .add_section(errors_section);

        report
    }

    /// Generate an executive summary
    pub fn generate_executive_summary(
        &self,
        usage_stats: &UsageStats,
        performance_profiles: &[ProfileReport],
    ) -> Report {
        let mut report = Report::new(ReportType::ExecutiveSummary, "Executive Summary")
            .with_description("High-level overview of system usage and performance");

        // Key metrics
        let key_metrics = ReportSection {
            title: "Key Metrics".to_string(),
            content: format!(
                "Total Events: {}\nActive Users: {}\nActive Sessions: {}\nError Rate: {:.2}%\nOperations Profiled: {}",
                usage_stats.total_events,
                usage_stats.active_users,
                usage_stats.active_sessions,
                usage_stats.error_rate * 100.0,
                performance_profiles.len()
            ),
            data: Some(serde_json::json!({
                "total_events": usage_stats.total_events,
                "active_users": usage_stats.active_users,
                "error_rate": usage_stats.error_rate,
                "operations_profiled": performance_profiles.len(),
            })),
            section_type: SectionType::KeyValue,
            subsections: Vec::new(),
        };

        // Recommendations
        let mut recommendations = Vec::new();

        if usage_stats.error_rate > 0.05 {
            recommendations.push("High error rate detected (>5%). Investigate error sources.");
        }

        if usage_stats.active_sessions > 1000 {
            recommendations.push("High concurrent session count. Consider resource scaling.");
        }

        let slowest_ops: Vec<_> = performance_profiles
            .iter()
            .filter(|p| p.avg_duration_ms > 1000.0)
            .collect();

        if !slowest_ops.is_empty() {
            recommendations.push("Some operations averaging >1s. Performance optimization recommended.");
        }

        let recommendations_section = ReportSection {
            title: "Recommendations".to_string(),
            content: if recommendations.is_empty() {
                "System performing within normal parameters.".to_string()
            } else {
                recommendations.join("\n")
            },
            data: None,
            section_type: SectionType::List,
            subsections: Vec::new(),
        };

        report = report
            .add_section(key_metrics)
            .add_section(recommendations_section);

        report
    }

    /// Render report to specified format
    pub fn render(&self, report: &Report, format: ReportFormat) -> Result<Vec<u8>> {
        match format {
            ReportFormat::Json => self.render_json(report),
            ReportFormat::Html => self.render_html(report),
            ReportFormat::Markdown => self.render_markdown(report),
            ReportFormat::Text => self.render_text(report),
            ReportFormat::Csv => self.render_csv(report),
            ReportFormat::Pdf => self.render_pdf(report),
        }
    }

    fn render_json(&self, report: &Report) -> Result<Vec<u8>> {
        let json = serde_json::to_vec_pretty(report)?;
        Ok(json)
    }

    fn render_html(&self, report: &Report) -> Result<Vec<u8>> {
        let mut html = String::from("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str(&format!("<title>{}</title>\n", report.title));
        html.push_str("<style>\nbody { font-family: Arial, sans-serif; margin: 40px; }\n");
        html.push_str("h1 { color: #333; }\nh2 { color: #666; margin-top: 30px; }\n");
        html.push_str("table { border-collapse: collapse; width: 100%; margin: 20px 0; }\n");
        html.push_str("th, td { border: 1px solid #ddd; padding: 12px; text-align: left; }\n");
        html.push_str("th { background-color: #4CAF50; color: white; }\n");
        html.push_str(".metadata { color: #888; font-size: 0.9em; }\n");
        html.push_str("</style>\n</head>\n<body>\n");

        html.push_str(&format!("<h1>{}</h1>\n", report.title));

        if let Some(desc) = &report.description {
            html.push_str(&format!("<p>{}</p>\n", desc));
        }

        html.push_str(&format!(
            "<p class='metadata'>Generated: {}</p>\n",
            report.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        for section in &report.sections {
            html.push_str(&Self::render_section_html(section));
        }

        html.push_str("</body>\n</html>");

        Ok(html.into_bytes())
    }

    fn render_section_html(section: &ReportSection) -> String {
        let mut html = format!("<h2>{}</h2>\n", section.title);
        html.push_str(&format!("<pre>{}</pre>\n", section.content));

        for subsection in &section.subsections {
            html.push_str(&Self::render_section_html(subsection));
        }

        html
    }

    fn render_markdown(&self, report: &Report) -> Result<Vec<u8>> {
        let mut md = format!("# {}\n\n", report.title);

        if let Some(desc) = &report.description {
            md.push_str(&format!("{}\n\n", desc));
        }

        md.push_str(&format!(
            "*Generated: {}*\n\n",
            report.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        for section in &report.sections {
            md.push_str(&Self::render_section_markdown(section, 2));
        }

        Ok(md.into_bytes())
    }

    fn render_section_markdown(section: &ReportSection, level: usize) -> String {
        let heading = "#".repeat(level);
        let mut md = format!("{} {}\n\n{}\n\n", heading, section.title, section.content);

        for subsection in &section.subsections {
            md.push_str(&Self::render_section_markdown(subsection, level + 1));
        }

        md
    }

    fn render_text(&self, report: &Report) -> Result<Vec<u8>> {
        let mut text = format!("{}\n{}\n\n", report.title, "=".repeat(report.title.len()));

        if let Some(desc) = &report.description {
            text.push_str(&format!("{}\n\n", desc));
        }

        text.push_str(&format!(
            "Generated: {}\n\n",
            report.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        for section in &report.sections {
            text.push_str(&Self::render_section_text(section, 0));
        }

        Ok(text.into_bytes())
    }

    fn render_section_text(section: &ReportSection, indent: usize) -> String {
        let indent_str = "  ".repeat(indent);
        let mut text = format!("{}{}\n{}{}\n\n{}{}\n\n",
            indent_str, section.title,
            indent_str, "-".repeat(section.title.len()),
            indent_str, section.content.replace('\n', &format!("\n{}", indent_str))
        );

        for subsection in &section.subsections {
            text.push_str(&Self::render_section_text(subsection, indent + 1));
        }

        text
    }

    fn render_csv(&self, report: &Report) -> Result<Vec<u8>> {
        // Simple CSV rendering - would need more sophistication for complex reports
        let mut csv = format!("Section,Content\n");

        for section in &report.sections {
            csv.push_str(&format!("\"{}\",\"{}\"\n",
                section.title.replace('"', "\"\""),
                section.content.replace('"', "\"\"").replace('\n', " ")
            ));
        }

        Ok(csv.into_bytes())
    }

    fn render_pdf(&self, _report: &Report) -> Result<Vec<u8>> {
        // PDF rendering would require additional dependencies
        // For now, return an error
        Err(AnalyticsError::Export(
            "PDF rendering not implemented. Use HTML format and convert externally.".to_string()
        ))
    }

    // Helper methods

    fn format_event_type_table(events: &HashMap<super::usage::EventType, u64>) -> String {
        let mut items: Vec<_> = events.iter().collect();
        items.sort_by(|a, b| b.1.cmp(a.1));

        items
            .iter()
            .map(|(k, v)| format!("{:?}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_usage_list(items: &[(String, u64)]) -> String {
        items
            .iter()
            .enumerate()
            .map(|(i, (name, count))| format!("{}. {}: {}", i + 1, name, count))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_creation() {
        let report = Report::new(ReportType::Usage, "Test Report")
            .with_description("Test description");

        assert_eq!(report.title, "Test Report");
        assert_eq!(report.description, Some("Test description".to_string()));
    }

    #[test]
    fn test_report_generator() {
        let generator = ReportGenerator::new();
        let report = Report::new(ReportType::Custom, "Test");

        let json = generator.render(&report, ReportFormat::Json);
        assert!(json.is_ok());
    }

    #[test]
    fn test_html_rendering() {
        let generator = ReportGenerator::new();
        let report = Report::new(ReportType::Usage, "Test Report");

        let html = generator.render(&report, ReportFormat::Html);
        assert!(html.is_ok());

        let html_str = String::from_utf8(html.unwrap()).unwrap();
        assert!(html_str.contains("<!DOCTYPE html>"));
        assert!(html_str.contains("Test Report"));
    }

    #[test]
    fn test_markdown_rendering() {
        let generator = ReportGenerator::new();
        let report = Report::new(ReportType::Usage, "Test Report");

        let md = generator.render(&report, ReportFormat::Markdown);
        assert!(md.is_ok());

        let md_str = String::from_utf8(md.unwrap()).unwrap();
        assert!(md_str.contains("# Test Report"));
    }
}
