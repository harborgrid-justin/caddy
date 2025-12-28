//! Enterprise Audit Logging System
//!
//! This module provides comprehensive audit logging capabilities for CADDY,
//! including event tracking, storage, querying, compliance features, and reporting.
//!
//! # Features
//!
//! - **Event Tracking**: Comprehensive audit event types with tamper-evident hashing
//! - **Thread-Safe Logging**: Async audit logger with automatic rotation and archival
//! - **Multiple Storage Backends**: File, database, and cloud storage options
//! - **Powerful Querying**: Flexible query builder with filtering and aggregation
//! - **Compliance**: Built-in SOX, GDPR, and HIPAA compliance helpers
//! - **Reporting**: Generate reports in JSON, CSV, HTML, and PDF formats
//! - **Alerting**: Real-time security event monitoring and alerting
//!
//! # Example
//!
//! ```no_run
//! use caddy::enterprise::audit::{
//!     event::{AuditEvent, EventType, EventSeverity},
//!     logger::{AuditLogger, LoggerConfig},
//!     storage::FileStorage,
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize storage
//! let storage = FileStorage::new("/var/log/caddy/audit")?;
//!
//! // Create logger
//! let logger = AuditLogger::new(storage, LoggerConfig::default());
//!
//! // Log an event
//! let event = AuditEvent::builder()
//!     .user_id("user123")
//!     .action(EventType::Create)
//!     .resource("drawing/floor-plan")
//!     .severity(EventSeverity::Info)
//!     .build();
//!
//! logger.log(event)?;
//! # Ok(())
//! # }
//! ```
//!
//! # Compliance
//!
//! ```no_run
//! use caddy::enterprise::audit::{
//!     compliance::{SoxCompliance, GdprCompliance, HipaaCompliance},
//!     storage::FileStorage,
//! };
//! use chrono::Utc;
//!
//! # async fn compliance_example() -> Result<(), Box<dyn std::error::Error>> {
//! let storage = FileStorage::new("/var/log/caddy/audit")?;
//!
//! // Generate SOX compliance report
//! let start = Utc::now() - chrono::Duration::days(30);
//! let end = Utc::now();
//! let report = SoxCompliance::verify_financial_controls(&storage, start, end).await?;
//!
//! println!("SOX Compliance: {}", if report.compliant { "PASS" } else { "FAIL" });
//! # Ok(())
//! # }
//! ```
//!
//! # Querying
//!
//! ```no_run
//! use caddy::enterprise::audit::{
//!     query::AuditQuery,
//!     event::EventType,
//!     storage::FileStorage,
//! };
//!
//! # async fn query_example() -> Result<(), Box<dyn std::error::Error>> {
//! let storage = FileStorage::new("/var/log/caddy/audit")?;
//!
//! // Query events
//! let events = AuditQuery::new()
//!     .user_id("user123")
//!     .action(EventType::Delete)
//!     .last_days(7)
//!     .execute(&storage)
//!     .await?;
//!
//! println!("Found {} events", events.len());
//! # Ok(())
//! # }
//! ```
//!
//! # Reporting
//!
//! ```no_run
//! use caddy::enterprise::audit::{
//!     reporter::{ReportGenerator, ReportConfig, ReportFormat},
//!     query::AuditQuery,
//!     storage::FileStorage,
//! };
//!
//! # async fn report_example() -> Result<(), Box<dyn std::error::Error>> {
//! let storage = FileStorage::new("/var/log/caddy/audit")?;
//!
//! let query = AuditQuery::new().last_days(30);
//! let config = ReportConfig {
//!     title: "Monthly Audit Report".to_string(),
//!     format: ReportFormat::Html,
//!     include_summary: true,
//!     include_details: true,
//!     include_charts: false,
//!     max_events: Some(1000),
//! };
//!
//! let report = ReportGenerator::generate(&storage, query, config).await?;
//! ReportGenerator::save_to_file(&report, ReportFormat::Html, "/tmp/report.html").await?;
//! # Ok(())
//! # }
//! ```

pub mod compliance;
pub mod event;
pub mod logger;
pub mod query;
pub mod reporter;
pub mod storage;

// Re-export commonly used types
pub use event::{AuditEvent, EventSeverity, EventType};
pub use logger::{AuditLogger, LoggerConfig, LoggerError};
pub use query::AuditQuery;
pub use storage::{AuditStorage, FileStorage, MemoryStorage, StorageError};

// Re-export compliance helpers
pub use compliance::{
    ComplianceReport, GdprCompliance, HipaaCompliance, RetentionPolicy, SoxCompliance,
};

// Re-export reporting
pub use reporter::{
    AlertConfig, AlertTrigger, ReportConfig, ReportFormat, ReportGenerator,
};

/// Version of the audit system
pub const AUDIT_VERSION: &str = "1.0.0";

/// Initialize the audit system with default file storage
///
/// # Arguments
///
/// * `log_dir` - Directory to store audit logs
///
/// # Example
///
/// ```no_run
/// use caddy::enterprise::audit;
///
/// # async fn example() {
/// audit::init("/var/log/caddy/audit").await.unwrap();
/// # }
/// ```
pub async fn init(log_dir: impl AsRef<std::path::Path>) -> Result<(), storage::StorageError> {
    let storage = storage::FileStorage::new(log_dir)?;
    let config = LoggerConfig::default();
    logger::init_global_logger(storage, config);
    Ok(())
}

/// Log an audit event using the global logger
///
/// This is a convenience function that uses the global logger instance.
///
/// # Example
///
/// ```no_run
/// use caddy::enterprise::audit::{self, event::{AuditEvent, EventType}};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let event = AuditEvent::builder()
///     .user_id("user123")
///     .action(EventType::Create)
///     .resource("drawing/123")
///     .build();
///
/// audit::log(event)?;
/// # Ok(())
/// # }
/// ```
pub fn log(event: AuditEvent) -> Result<(), LoggerError> {
    logger::log_event(event)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify that all main types are accessible
        let _event = AuditEvent::builder()
            .user_id("test")
            .action(EventType::Create)
            .resource("test")
            .build();

        let _query = AuditQuery::new();
        let _config = LoggerConfig::default();
    }
}
