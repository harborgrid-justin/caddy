//! Enterprise Performance Analytics System for CADDY v0.1.5
//!
//! This module provides comprehensive performance monitoring, metrics collection,
//! and analytics capabilities for enterprise CAD operations.
//!
//! # Features
//!
//! - Thread-safe metrics collection
//! - Real-time aggregation and analysis
//! - Time-series data storage
//! - Configurable dashboards
//! - Alert system with flexible rules
//! - Comprehensive reporting
//!
//! # Example
//!
//! ```no_run
//! use caddy::enterprise::analytics::{MetricRegistry, SystemCollector};
//!
//! let registry = MetricRegistry::new();
//! let collector = SystemCollector::new(&registry);
//! collector.start();
//! ```

pub mod aggregator;
pub mod alerting;
pub mod collector;
pub mod dashboard;
pub mod metrics;
pub mod reporting;
pub mod storage;

pub use aggregator::{Aggregator, AggregationConfig, TimeWindow, Percentile};
pub use alerting::{AlertManager, AlertRule, AlertCondition, AlertSeverity};
pub use collector::{
    SystemCollector, ApplicationCollector, UserCollector, RenderCollector,
    CollectorConfig,
};
pub use dashboard::{Dashboard, DashboardConfig, Widget, WidgetType};
pub use metrics::{
    Metric, MetricType, MetricRegistry, Counter, Gauge, Histogram, Summary,
    Labels, MetricValue,
};
pub use reporting::{Reporter, Report, ReportConfig, ReportFormat};
pub use storage::{
    MetricStorage, TimeSeriesPoint, RetentionPolicy, StorageBackend,
};

use std::error::Error as StdError;
use std::fmt;

/// Result type for analytics operations
pub type Result<T> = std::result::Result<T, AnalyticsError>;

/// Errors that can occur in the analytics system
#[derive(Debug, Clone)]
pub enum AnalyticsError {
    /// Metric not found
    MetricNotFound(String),
    /// Invalid metric configuration
    InvalidConfig(String),
    /// Storage error
    StorageError(String),
    /// Aggregation error
    AggregationError(String),
    /// Alert error
    AlertError(String),
    /// Report generation error
    ReportError(String),
    /// I/O error
    IoError(String),
    /// Serialization error
    SerializationError(String),
}

impl fmt::Display for AnalyticsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MetricNotFound(name) => write!(f, "Metric not found: {}", name),
            Self::InvalidConfig(msg) => write!(f, "Invalid configuration: {}", msg),
            Self::StorageError(msg) => write!(f, "Storage error: {}", msg),
            Self::AggregationError(msg) => write!(f, "Aggregation error: {}", msg),
            Self::AlertError(msg) => write!(f, "Alert error: {}", msg),
            Self::ReportError(msg) => write!(f, "Report error: {}", msg),
            Self::IoError(msg) => write!(f, "I/O error: {}", msg),
            Self::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl StdError for AnalyticsError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AnalyticsError::MetricNotFound("test_metric".to_string());
        assert_eq!(format!("{}", err), "Metric not found: test_metric");
    }
}
