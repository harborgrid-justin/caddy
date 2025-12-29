//! # Metrics Export
//!
//! Export metrics to various formats including Prometheus, OpenTelemetry, and custom formats.

use super::{Result, AnalyticsError, Metric, MetricType, ExportEndpoint};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use parking_lot::RwLock;

/// Export format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    /// Prometheus text format
    Prometheus,
    /// OpenTelemetry Protocol (OTLP)
    OpenTelemetry,
    /// JSON format
    Json,
    /// CSV format
    Csv,
    /// Custom binary format
    Binary,
}

impl ExportFormat {
    /// Get the file extension for this format
    pub fn extension(&self) -> &str {
        match self {
            Self::Prometheus => "prom",
            Self::OpenTelemetry => "otlp",
            Self::Json => "json",
            Self::Csv => "csv",
            Self::Binary => "bin",
        }
    }

    /// Get the MIME type for this format
    pub fn mime_type(&self) -> &str {
        match self {
            Self::Prometheus => "text/plain; version=0.0.4",
            Self::OpenTelemetry => "application/x-protobuf",
            Self::Json => "application/json",
            Self::Csv => "text/csv",
            Self::Binary => "application/octet-stream",
        }
    }
}

/// Metrics exporter
pub struct MetricsExporter {
    endpoints: Vec<ExportEndpoint>,
    last_export: Arc<RwLock<Option<DateTime<Utc>>>>,
    export_stats: Arc<RwLock<ExportStats>>,
}

#[derive(Debug, Default)]
struct ExportStats {
    total_exports: u64,
    total_failures: u64,
    total_bytes_exported: u64,
}

impl MetricsExporter {
    /// Create a new metrics exporter
    pub fn new(endpoints: Vec<ExportEndpoint>) -> Self {
        Self {
            endpoints,
            last_export: Arc::new(RwLock::new(None)),
            export_stats: Arc::new(RwLock::new(ExportStats::default())),
        }
    }

    /// Export metrics to all configured endpoints
    pub async fn export(&self, metrics: Vec<Metric>) -> Result<()> {
        for endpoint in &self.endpoints {
            self.export_to_endpoint(&endpoint, &metrics).await?;
        }

        *self.last_export.write() = Some(Utc::now());
        self.export_stats.write().total_exports += 1;

        Ok(())
    }

    /// Export to a specific endpoint
    async fn export_to_endpoint(
        &self,
        endpoint: &ExportEndpoint,
        metrics: &[Metric],
    ) -> Result<()> {
        let data = match endpoint.format {
            ExportFormat::Prometheus => self.export_prometheus(metrics)?,
            ExportFormat::OpenTelemetry => self.export_opentelemetry(metrics)?,
            ExportFormat::Json => self.export_json(metrics)?,
            ExportFormat::Csv => self.export_csv(metrics)?,
            ExportFormat::Binary => self.export_binary(metrics)?,
        };

        // Send to endpoint
        self.send_to_endpoint(endpoint, &data).await?;

        self.export_stats.write().total_bytes_exported += data.len() as u64;

        Ok(())
    }

    /// Send data to an endpoint
    async fn send_to_endpoint(&self, endpoint: &ExportEndpoint, data: &[u8]) -> Result<()> {
        let client = reqwest::Client::new();
        let mut request = client
            .post(&endpoint.url)
            .header("Content-Type", endpoint.format.mime_type())
            .body(data.to_vec());

        if let Some(token) = &endpoint.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        request
            .send()
            .await
            .map_err(|e| AnalyticsError::Export(format!("Failed to send data: {}", e)))?;

        Ok(())
    }

    /// Export metrics in Prometheus format
    fn export_prometheus(&self, metrics: &[Metric]) -> Result<Vec<u8>> {
        let mut output = String::new();

        for metric in metrics {
            let metric_name = Self::sanitize_prometheus_name(&metric.name);

            // Add HELP line (description)
            if let Some(desc) = &metric.description {
                output.push_str(&format!("# HELP {} {}\n", metric_name, desc));
            }

            // Add TYPE line
            let metric_type = match metric.metric_type {
                MetricType::Counter => "counter",
                MetricType::Gauge => "gauge",
                MetricType::Histogram => "histogram",
                MetricType::Summary => "summary",
            };
            output.push_str(&format!("# TYPE {} {}\n", metric_name, metric_type));

            // Add metric line with labels
            let labels_str = Self::format_prometheus_labels(&metric.labels);
            let value = metric.value.as_f64().unwrap_or(0.0);
            let timestamp_ms = metric.timestamp.timestamp_millis();

            output.push_str(&format!(
                "{}{} {} {}\n",
                metric_name, labels_str, value, timestamp_ms
            ));
        }

        Ok(output.into_bytes())
    }

    /// Export metrics in OpenTelemetry format (simplified JSON representation)
    fn export_opentelemetry(&self, metrics: &[Metric]) -> Result<Vec<u8>> {
        #[derive(Serialize)]
        struct OtlpMetric {
            name: String,
            #[serde(rename = "type")]
            metric_type: String,
            value: f64,
            labels: HashMap<String, String>,
            timestamp: i64,
        }

        #[derive(Serialize)]
        struct OtlpExport {
            resource_metrics: Vec<OtlpMetric>,
        }

        let resource_metrics = metrics
            .iter()
            .filter_map(|m| {
                m.value.as_f64().map(|value| OtlpMetric {
                    name: m.name.clone(),
                    metric_type: format!("{:?}", m.metric_type),
                    value,
                    labels: m.labels.clone(),
                    timestamp: m.timestamp.timestamp_nanos_opt().unwrap_or(0),
                })
            })
            .collect();

        let export = OtlpExport { resource_metrics };
        let json = serde_json::to_vec(&export)?;

        Ok(json)
    }

    /// Export metrics in JSON format
    fn export_json(&self, metrics: &[Metric]) -> Result<Vec<u8>> {
        let json = serde_json::to_vec(metrics)?;
        Ok(json)
    }

    /// Export metrics in CSV format
    fn export_csv(&self, metrics: &[Metric]) -> Result<Vec<u8>> {
        let mut csv = String::from("timestamp,name,type,value,labels\n");

        for metric in metrics {
            let timestamp = metric.timestamp.to_rfc3339();
            let name = &metric.name;
            let metric_type = format!("{:?}", metric.metric_type);
            let value = metric.value.as_f64().unwrap_or(0.0);
            let labels = serde_json::to_string(&metric.labels).unwrap_or_default();

            csv.push_str(&format!(
                "{},{},{},{},{}\n",
                timestamp, name, metric_type, value, labels
            ));
        }

        Ok(csv.into_bytes())
    }

    /// Export metrics in binary format (using bincode)
    fn export_binary(&self, metrics: &[Metric]) -> Result<Vec<u8>> {
        let binary = bincode::serialize(metrics)
            .map_err(|e| AnalyticsError::Export(format!("Binary serialization failed: {}", e)))?;

        Ok(binary)
    }

    /// Sanitize metric name for Prometheus
    fn sanitize_prometheus_name(name: &str) -> String {
        name.replace('.', "_")
            .replace('-', "_")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == ':')
            .collect()
    }

    /// Format labels for Prometheus
    fn format_prometheus_labels(labels: &HashMap<String, String>) -> String {
        if labels.is_empty() {
            return String::new();
        }

        let labels_vec: Vec<String> = labels
            .iter()
            .map(|(k, v)| format!("{}=\"{}\"", k, v.replace('\"', "\\\"")))
            .collect();

        format!("{{{}}}", labels_vec.join(","))
    }

    /// Get last export time
    pub async fn last_export_time(&self) -> Option<DateTime<Utc>> {
        *self.last_export.read()
    }

    /// Get export statistics
    pub fn statistics(&self) -> ExportStatistics {
        let stats = self.export_stats.read();
        ExportStatistics {
            total_exports: stats.total_exports,
            total_failures: stats.total_failures,
            total_bytes_exported: stats.total_bytes_exported,
        }
    }
}

/// Export statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportStatistics {
    pub total_exports: u64,
    pub total_failures: u64,
    pub total_bytes_exported: u64,
}

/// Prometheus exporter (specialized)
pub struct PrometheusExporter {
    endpoint: String,
    auth_token: Option<String>,
}

impl PrometheusExporter {
    /// Create a new Prometheus exporter
    pub fn new(endpoint: String, auth_token: Option<String>) -> Self {
        Self {
            endpoint,
            auth_token,
        }
    }

    /// Push metrics to Prometheus Pushgateway
    pub async fn push(&self, job: &str, metrics: Vec<Metric>) -> Result<()> {
        let exporter = MetricsExporter::new(vec![ExportEndpoint {
            name: "prometheus".to_string(),
            url: format!("{}/metrics/job/{}", self.endpoint, job),
            format: ExportFormat::Prometheus,
            interval_secs: 0,
            auth_token: self.auth_token.clone(),
        }]);

        exporter.export(metrics).await
    }
}

/// OpenTelemetry exporter (specialized)
pub struct OpenTelemetryExporter {
    endpoint: String,
    service_name: String,
    headers: HashMap<String, String>,
}

impl OpenTelemetryExporter {
    /// Create a new OpenTelemetry exporter
    pub fn new(endpoint: String, service_name: String) -> Self {
        Self {
            endpoint,
            service_name,
            headers: HashMap::new(),
        }
    }

    /// Add a custom header
    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    /// Export metrics via OTLP
    pub async fn export(&self, metrics: Vec<Metric>) -> Result<()> {
        let exporter = MetricsExporter::new(vec![ExportEndpoint {
            name: "otlp".to_string(),
            url: self.endpoint.clone(),
            format: ExportFormat::OpenTelemetry,
            interval_secs: 0,
            auth_token: None,
        }]);

        exporter.export(metrics).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prometheus_name_sanitization() {
        let name = "test.metric-name";
        let sanitized = MetricsExporter::sanitize_prometheus_name(name);
        assert_eq!(sanitized, "test_metric_name");
    }

    #[test]
    fn test_prometheus_labels_format() {
        let mut labels = HashMap::new();
        labels.insert("env".to_string(), "prod".to_string());
        labels.insert("region".to_string(), "us-west".to_string());

        let formatted = MetricsExporter::format_prometheus_labels(&labels);
        assert!(formatted.contains("env=\"prod\""));
        assert!(formatted.contains("region=\"us-west\""));
    }

    #[test]
    fn test_export_format_extensions() {
        assert_eq!(ExportFormat::Prometheus.extension(), "prom");
        assert_eq!(ExportFormat::Json.extension(), "json");
        assert_eq!(ExportFormat::Csv.extension(), "csv");
    }

    #[test]
    fn test_json_export() {
        let exporter = MetricsExporter::new(vec![]);
        let metrics = vec![Metric::counter("test_counter", 42)];

        let result = exporter.export_json(&metrics);
        assert!(result.is_ok());
    }

    #[test]
    fn test_csv_export() {
        let exporter = MetricsExporter::new(vec![]);
        let metrics = vec![Metric::gauge("test_gauge", 3.14)];

        let result = exporter.export_csv(&metrics);
        assert!(result.is_ok());

        let csv_str = String::from_utf8(result.unwrap()).unwrap();
        assert!(csv_str.contains("test_gauge"));
    }
}
