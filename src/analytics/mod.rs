//! # Analytics and Telemetry System
//!
//! Enterprise-grade analytics, metrics collection, and observability for CADDY.
//!
//! ## Features
//!
//! - Real-time metrics collection and aggregation
//! - Time-series data storage with efficient compression
//! - Performance profiling and tracing
//! - Usage analytics and user behavior tracking
//! - Export to Prometheus, OpenTelemetry, and custom formats
//! - Customizable reports and dashboards
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────┐
//! │  Collector  │ ──> Metrics ingestion from all subsystems
//! └──────┬──────┘
//!        │
//!        ▼
//! ┌─────────────┐
//! │ Aggregator  │ ──> Time-series aggregation and rollups
//! └──────┬──────┘
//!        │
//!        ▼
//! ┌─────────────┐
//! │   Storage   │ ──> Efficient metrics storage with compression
//! └──────┬──────┘
//!        │
//!        ├──> Export (Prometheus, OTLP, JSON)
//!        ├──> Reporting (PDF, HTML, CSV)
//!        └──> Query API (Dashboard, CLI)
//! ```

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

pub mod collector;
pub mod aggregator;
pub mod storage;
pub mod export;
pub mod performance;
pub mod usage;
pub mod reporting;

// Re-exports for convenience
pub use collector::{MetricsCollector, Metric, MetricType, MetricValue};
pub use aggregator::{Aggregator, AggregationWindow, AggregatedMetric};
pub use storage::{MetricsStorage, StorageConfig, TimeSeriesPoint};
pub use export::{MetricsExporter, ExportFormat, PrometheusExporter, OpenTelemetryExporter};
pub use performance::{PerformanceProfiler, ProfileSpan, ProfileReport};
pub use usage::{UsageTracker, UsageEvent, UsageStats};
pub use reporting::{ReportGenerator, ReportFormat, Report, ReportSection};

/// Analytics system errors
#[derive(Debug, Error)]
pub enum AnalyticsError {
    /// Storage error
    #[error("Storage error: {0}")]
    Storage(String),

    /// Export error
    #[error("Export error: {0}")]
    Export(String),

    /// Aggregation error
    #[error("Aggregation error: {0}")]
    Aggregation(String),

    /// Collection error
    #[error("Collection error: {0}")]
    Collection(String),

    /// Invalid metric
    #[error("Invalid metric: {0}")]
    InvalidMetric(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result type for analytics operations
pub type Result<T> = std::result::Result<T, AnalyticsError>;

/// Analytics system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    /// Enable metrics collection
    pub enabled: bool,

    /// Collection interval in seconds
    pub collection_interval_secs: u64,

    /// Aggregation window size in seconds
    pub aggregation_window_secs: u64,

    /// Storage retention period in days
    pub retention_days: u32,

    /// Maximum storage size in bytes
    pub max_storage_bytes: u64,

    /// Enable performance profiling
    pub enable_profiling: bool,

    /// Enable usage tracking
    pub enable_usage_tracking: bool,

    /// Export endpoints
    pub export_endpoints: Vec<ExportEndpoint>,

    /// Storage path
    pub storage_path: String,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval_secs: 10,
            aggregation_window_secs: 60,
            retention_days: 30,
            max_storage_bytes: 10 * 1024 * 1024 * 1024, // 10 GB
            enable_profiling: true,
            enable_usage_tracking: true,
            export_endpoints: Vec::new(),
            storage_path: "./analytics_data".to_string(),
        }
    }
}

/// Export endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportEndpoint {
    /// Endpoint name
    pub name: String,

    /// Endpoint URL
    pub url: String,

    /// Export format
    pub format: ExportFormat,

    /// Export interval in seconds
    pub interval_secs: u64,

    /// Authentication token (optional)
    pub auth_token: Option<String>,
}

/// Main analytics system
pub struct AnalyticsSystem {
    config: AnalyticsConfig,
    collector: Arc<MetricsCollector>,
    aggregator: Arc<Aggregator>,
    storage: Arc<MetricsStorage>,
    exporter: Arc<MetricsExporter>,
    profiler: Arc<PerformanceProfiler>,
    usage_tracker: Arc<UsageTracker>,
    report_generator: Arc<ReportGenerator>,
}

impl AnalyticsSystem {
    /// Create a new analytics system
    pub fn new(config: AnalyticsConfig) -> Result<Self> {
        let storage = Arc::new(MetricsStorage::new(StorageConfig {
            path: config.storage_path.clone(),
            retention_days: config.retention_days,
            max_size_bytes: config.max_storage_bytes,
            enable_compression: true,
        })?);

        let collector = Arc::new(MetricsCollector::new());
        let aggregator = Arc::new(Aggregator::new(config.aggregation_window_secs));
        let exporter = Arc::new(MetricsExporter::new(config.export_endpoints.clone()));
        let profiler = Arc::new(PerformanceProfiler::new(config.enable_profiling));
        let usage_tracker = Arc::new(UsageTracker::new(config.enable_usage_tracking));
        let report_generator = Arc::new(ReportGenerator::new());

        Ok(Self {
            config,
            collector,
            aggregator,
            storage,
            exporter,
            profiler,
            usage_tracker,
            report_generator,
        })
    }

    /// Start the analytics system
    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // Start background tasks for collection, aggregation, and export
        self.start_collection_task().await?;
        self.start_aggregation_task().await?;
        self.start_export_task().await?;

        Ok(())
    }

    /// Stop the analytics system
    pub async fn stop(&self) -> Result<()> {
        // Flush all pending metrics
        self.storage.flush().await?;
        Ok(())
    }

    /// Get the metrics collector
    pub fn collector(&self) -> Arc<MetricsCollector> {
        Arc::clone(&self.collector)
    }

    /// Get the performance profiler
    pub fn profiler(&self) -> Arc<PerformanceProfiler> {
        Arc::clone(&self.profiler)
    }

    /// Get the usage tracker
    pub fn usage_tracker(&self) -> Arc<UsageTracker> {
        Arc::clone(&self.usage_tracker)
    }

    /// Get the report generator
    pub fn report_generator(&self) -> Arc<ReportGenerator> {
        Arc::clone(&self.report_generator)
    }

    /// Query metrics for a time range
    pub async fn query_metrics(
        &self,
        metric_name: &str,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<TimeSeriesPoint>> {
        self.storage.query(metric_name, start_time, end_time).await
    }

    /// Get current system health status
    pub async fn health_status(&self) -> HealthStatus {
        HealthStatus {
            metrics_collected: self.collector.total_metrics(),
            storage_size_bytes: self.storage.size_bytes().await.unwrap_or(0),
            uptime_seconds: self.collector.uptime_seconds(),
            active_profiles: self.profiler.active_count(),
            last_export: self.exporter.last_export_time().await,
        }
    }

    // Private methods for background tasks

    async fn start_collection_task(&self) -> Result<()> {
        // Implementation would spawn a background task
        Ok(())
    }

    async fn start_aggregation_task(&self) -> Result<()> {
        // Implementation would spawn a background task
        Ok(())
    }

    async fn start_export_task(&self) -> Result<()> {
        // Implementation would spawn a background task
        Ok(())
    }
}

/// Health status of the analytics system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Total metrics collected
    pub metrics_collected: u64,

    /// Current storage size in bytes
    pub storage_size_bytes: u64,

    /// System uptime in seconds
    pub uptime_seconds: u64,

    /// Number of active performance profiles
    pub active_profiles: usize,

    /// Last export timestamp
    pub last_export: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AnalyticsConfig::default();
        assert!(config.enabled);
        assert_eq!(config.collection_interval_secs, 10);
        assert_eq!(config.retention_days, 30);
    }

    #[tokio::test]
    async fn test_analytics_system_creation() {
        let config = AnalyticsConfig::default();
        let system = AnalyticsSystem::new(config);
        assert!(system.is_ok());
    }
}
