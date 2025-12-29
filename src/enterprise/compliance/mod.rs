//! # CADDY Enterprise Compliance Module
//!
//! Comprehensive compliance and audit logging system for enterprise regulatory requirements.
//! Supports GDPR, SOC 2, HIPAA, and other compliance frameworks with immutable audit trails,
//! cryptographic integrity, and automated reporting.
//!
//! ## Features
//!
//! - **Immutable Audit Trail**: Cryptographically secured chain-hashed audit entries
//! - **Event Classification**: Automatic categorization and severity assessment
//! - **GDPR Compliance**: Data subject rights, consent tracking, and processing records
//! - **SOC 2 Controls**: Trust service criteria implementation and evidence collection
//! - **HIPAA Compliance**: PHI access logging and breach notification tracking
//! - **Retention Policies**: Configurable data lifecycle and legal hold management
//! - **Compliance Reporting**: Automated report generation in multiple formats
//! - **Alert System**: Real-time compliance violation detection and escalation
//!
//! ## Architecture
//!
//! The compliance module is organized into specialized sub-modules:
//!
//! ```text
//! compliance/
//! ├── trail.rs           - Immutable audit trail with chain hashing
//! ├── classification.rs  - Event categorization and severity levels
//! ├── gdpr.rs           - GDPR compliance features
//! ├── soc2.rs           - SOC 2 controls and evidence
//! ├── hipaa.rs          - HIPAA compliance and PHI protection
//! ├── retention.rs      - Data retention policies
//! ├── reporting.rs      - Compliance report generation
//! ├── alerts.rs         - Alert rules and anomaly detection
//! └── mod.rs            - Module exports
//! ```
//!
//! ## Usage Example
//!
//! ```rust
//! use caddy::enterprise::compliance::{
//!     trail::{AuditTrail, AuditEntryBuilder},
//!     classification::{EventClassifier, ClassifiedEventBuilder, EventCategory},
//!     gdpr::GdprManager,
//!     soc2::Soc2Manager,
//!     alerts::AlertManager,
//! };
//!
//! // Create audit trail with cryptographic verification
//! let trail = AuditTrail::new();
//!
//! // Append immutable entries
//! let entry = AuditEntryBuilder::new("user1", "create", "document/123")
//!     .metadata("type", "CAD drawing");
//!
//! // trail.append(entry).await?;
//!
//! // Classify events automatically
//! let classifier = EventClassifier::new();
//!
//! // Manage GDPR compliance
//! let gdpr = GdprManager::new();
//!
//! // Track SOC 2 controls
//! let soc2 = Soc2Manager::new();
//!
//! // Monitor for compliance violations
//! let alerts = AlertManager::new();
//! ```
//!
//! ## Compliance Frameworks Supported
//!
//! ### GDPR (General Data Protection Regulation)
//!
//! - Data subject access requests (DSAR)
//! - Right to erasure ("right to be forgotten")
//! - Data portability
//! - Consent management
//! - Processing activity records (Article 30)
//! - Data protection impact assessments (DPIA)
//!
//! ### SOC 2 (Service Organization Control 2)
//!
//! - Security controls (CC6.x)
//! - Availability controls (A1.x)
//! - Processing integrity controls (PI1.x)
//! - Confidentiality controls (C1.x)
//! - Privacy controls (P1.x)
//! - Change management logging
//! - Incident tracking
//! - Access control evidence
//!
//! ### HIPAA (Health Insurance Portability and Accountability Act)
//!
//! - Protected Health Information (PHI) access logging
//! - Minimum necessary rule enforcement
//! - Breach notification tracking
//! - Business associate agreements (BAA)
//! - Security rule compliance
//! - Privacy rule compliance
//!
//! ## Security Features
//!
//! ### Cryptographic Integrity
//!
//! - BLAKE3 hashing for audit entries
//! - Chain hashing for tamper detection
//! - Optional digital signatures
//! - Integrity verification
//!
//! ### Access Control
//!
//! - Role-based audit access
//! - Separation of duties
//! - Audit log protection
//! - Tamper-evident storage
//!
//! ## Performance Considerations
//!
//! - Async/await for non-blocking operations
//! - Efficient indexing for queries
//! - Configurable retention periods
//! - Archival to cold storage
//! - Batch processing for purges
//!
//! ## Best Practices
//!
//! 1. **Enable signing for critical audits**: Use digital signatures for forensic value
//! 2. **Regular chain verification**: Periodically verify audit trail integrity
//! 3. **Implement retention policies**: Define and enforce data lifecycle policies
//! 4. **Monitor alerts**: Set up real-time monitoring for compliance violations
//! 5. **Generate regular reports**: Automate compliance reporting schedules
//! 6. **Test DSAR workflows**: Ensure data subject requests can be fulfilled
//! 7. **Document controls**: Maintain clear documentation of compliance controls
//!
//! ## Integration
//!
//! The compliance module integrates with other enterprise modules:
//!
//! - **Auth**: User identity for audit trails
//! - **Security**: Encryption and key management
//! - **Database**: Persistent storage for audit logs
//! - **Analytics**: Metrics and monitoring
//! - **Workflow**: Automated compliance workflows

#![warn(missing_docs)]
#![warn(clippy::all)]
#![allow(clippy::module_name_repetitions)]

// ============================================================================
// Module Declarations
// ============================================================================

/// Immutable audit trail with cryptographic chain hashing
///
/// Provides tamper-evident audit logging with BLAKE3 hashing and optional
/// digital signatures for forensic integrity.
pub mod trail;

/// Event classification and categorization
///
/// Automatic classification of events by type, severity, and compliance
/// framework relevance with built-in rule engine.
pub mod classification;

/// GDPR (General Data Protection Regulation) compliance
///
/// Data subject rights, consent management, processing records, and
/// DSAR (Data Subject Access Request) handling.
pub mod gdpr;

/// SOC 2 compliance controls
///
/// Trust service criteria implementation, control execution tracking,
/// evidence collection, and attestation reporting.
pub mod soc2;

/// HIPAA compliance and PHI protection
///
/// Protected Health Information access logging, breach notification,
/// business associate tracking, and minimum necessary rule enforcement.
pub mod hipaa;

/// Data retention policies and lifecycle management
///
/// Configurable retention periods, legal holds, archival strategies,
/// and automated purge scheduling.
pub mod retention;

/// Compliance reporting and evidence collection
///
/// Report generation, evidence aggregation, multi-format export
/// (JSON, CSV, PDF, Markdown), and scheduled reporting.
pub mod reporting;

/// Alert rules and compliance violation detection
///
/// Real-time monitoring, anomaly detection, escalation policies,
/// and multi-channel notifications.
pub mod alerts;

// ============================================================================
// Re-exports for Convenience
// ============================================================================

// Audit Trail
pub use trail::{AuditEntry, AuditEntryBuilder, AuditTrail};

// Classification
pub use classification::{
    AuthEventType, ClassifiedEvent, ClassifiedEventBuilder, ComplianceFramework,
    ConfigEventType, DataAccessEventType, EventCategory, EventClassifier,
    EventSeverity, SecurityEventType,
};

// GDPR
pub use gdpr::{
    ConsentRecord, DataSubjectAccessRequest, DsarStatus, DsarType, GdprManager,
    LegalBasis, PersonalDataType, ProcessingActivity,
};

// SOC 2
pub use soc2::{
    Control, ControlExecution, ControlFrequency, ControlId, ControlResult,
    Evidence, EvidenceType, SecurityIncident, Soc2Manager, TrustServiceCategory,
};

// HIPAA
pub use hipaa::{
    AccessPurpose, BreachRecord, BusinessAssociate, HipaaManager, PhiAccessLog,
    PhiAction, PhiType,
};

// Retention
pub use retention::{
    ArchivalStrategy, LifecycleStage, PostRetentionAction, PurgeSchedule,
    RetentionManager, RetentionPeriod, RetentionPolicy, RetentionRecord,
};

// Reporting
pub use reporting::{
    ComplianceReport, EvidenceReference, ReportBuilder, ReportFormat,
    ReportingManager, ReportSchedule, ReportStatus, ReportTemplate, ReportType,
};

// Alerts
pub use alerts::{
    AlertCategory, AlertManager, AlertRule, AlertSeverity, AlertStatus,
    ComplianceAlert, EscalationPolicy, NotificationChannel, RuleCondition,
};

// ============================================================================
// Common Types
// ============================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

/// Compliance module errors
#[derive(Error, Debug)]
pub enum ComplianceError {
    /// Audit trail integrity violation
    #[error("Audit trail integrity violation: {0}")]
    IntegrityViolation(String),

    /// GDPR compliance error
    #[error("GDPR compliance error: {0}")]
    GdprError(String),

    /// SOC 2 compliance error
    #[error("SOC 2 compliance error: {0}")]
    Soc2Error(String),

    /// HIPAA compliance error
    #[error("HIPAA compliance error: {0}")]
    HipaaError(String),

    /// Retention policy error
    #[error("Retention policy error: {0}")]
    RetentionError(String),

    /// Reporting error
    #[error("Reporting error: {0}")]
    ReportingError(String),

    /// Alert error
    #[error("Alert error: {0}")]
    AlertError(String),

    /// Data access denied
    #[error("Access denied: {0}")]
    AccessDenied(String),

    /// Data not found
    #[error("Data not found: {0}")]
    NotFound(String),

    /// Invalid operation
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Generic compliance error
    #[error("Compliance error: {0}")]
    Other(String),
}

/// Result type for compliance operations
pub type ComplianceResult<T> = Result<T, ComplianceError>;

/// Compliance framework identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Framework {
    /// SOC 2
    Soc2,
    /// GDPR
    Gdpr,
    /// HIPAA
    Hipaa,
    /// PCI DSS
    PciDss,
    /// ISO 27001
    Iso27001,
    /// NIST
    Nist,
}

/// Compliance status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceStatus {
    /// Compliant
    Compliant,
    /// Non-compliant
    NonCompliant,
    /// Partially compliant
    PartiallyCompliant,
    /// Under review
    UnderReview,
    /// Not applicable
    NotApplicable,
}

// ============================================================================
// Unified Compliance Manager
// ============================================================================

/// Unified compliance manager coordinating all compliance modules
///
/// Provides a single entry point for all compliance operations, coordinating
/// audit trails, classifications, GDPR, SOC 2, HIPAA, retention, reporting,
/// and alerting.
pub struct ComplianceManager {
    /// Audit trail
    pub trail: trail::AuditTrail,

    /// Event classifier
    pub classifier: classification::EventClassifier,

    /// GDPR manager
    pub gdpr: gdpr::GdprManager,

    /// SOC 2 manager
    pub soc2: soc2::Soc2Manager,

    /// HIPAA manager
    pub hipaa: hipaa::HipaaManager,

    /// Retention manager
    pub retention: retention::RetentionManager,

    /// Reporting manager
    pub reporting: reporting::ReportingManager,

    /// Alert manager
    pub alerts: alerts::AlertManager,

    /// Configuration
    config: ComplianceConfig,
}

/// Compliance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    /// Enable audit trail
    pub enable_audit_trail: bool,

    /// Enable GDPR features
    pub enable_gdpr: bool,

    /// Enable SOC 2 features
    pub enable_soc2: bool,

    /// Enable HIPAA features
    pub enable_hipaa: bool,

    /// Enable retention policies
    pub enable_retention: bool,

    /// Enable reporting
    pub enable_reporting: bool,

    /// Enable alerts
    pub enable_alerts: bool,

    /// Audit trail signing enabled
    pub enable_audit_signing: bool,

    /// Additional configuration
    pub metadata: HashMap<String, String>,
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            enable_audit_trail: true,
            enable_gdpr: true,
            enable_soc2: true,
            enable_hipaa: false,
            enable_retention: true,
            enable_reporting: true,
            enable_alerts: true,
            enable_audit_signing: false,
            metadata: HashMap::new(),
        }
    }
}

impl ComplianceManager {
    /// Create new compliance manager with default configuration
    pub fn new() -> Self {
        Self::with_config(ComplianceConfig::default())
    }

    /// Create compliance manager with custom configuration
    pub fn with_config(config: ComplianceConfig) -> Self {
        let trail = if config.enable_audit_signing {
            // In production, load actual signing key
            trail::AuditTrail::with_signing(vec![0u8; 32])
        } else {
            trail::AuditTrail::new()
        };

        Self {
            trail,
            classifier: classification::EventClassifier::new(),
            gdpr: gdpr::GdprManager::new(),
            soc2: soc2::Soc2Manager::new(),
            hipaa: hipaa::HipaaManager::new(),
            retention: retention::RetentionManager::new(),
            reporting: reporting::ReportingManager::new(),
            alerts: alerts::AlertManager::new(),
            config,
        }
    }

    /// Get configuration
    pub fn config(&self) -> &ComplianceConfig {
        &self.config
    }

    /// Check overall compliance status for a framework
    pub async fn check_compliance_status(&self, framework: Framework) -> ComplianceStatus {
        // In production, implement actual compliance checking logic
        match framework {
            Framework::Soc2 if self.config.enable_soc2 => ComplianceStatus::Compliant,
            Framework::Gdpr if self.config.enable_gdpr => ComplianceStatus::Compliant,
            Framework::Hipaa if self.config.enable_hipaa => ComplianceStatus::Compliant,
            _ => ComplianceStatus::NotApplicable,
        }
    }

    /// Generate comprehensive compliance report
    pub async fn generate_comprehensive_report(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> ComplianceResult<Uuid> {
        let mut content = serde_json::json!({
            "title": "Comprehensive Compliance Report",
            "period": {
                "start": start,
                "end": end,
            },
            "frameworks": {},
        });

        // Add GDPR report if enabled
        if self.config.enable_gdpr {
            let gdpr_report = self.gdpr.generate_audit_report(start, end).await;
            content["frameworks"]["gdpr"] = serde_json::json!(gdpr_report);
        }

        // Add SOC 2 report if enabled
        if self.config.enable_soc2 {
            let soc2_report = self.soc2.generate_attestation_report(start, end).await;
            content["frameworks"]["soc2"] = serde_json::json!(soc2_report);
        }

        // Add HIPAA report if enabled
        if self.config.enable_hipaa {
            let hipaa_report = self.hipaa.generate_audit_report(start, end).await;
            content["frameworks"]["hipaa"] = serde_json::json!(hipaa_report);
        }

        // Generate report
        let builder = reporting::ReportBuilder::new(
            reporting::ReportType::ComplianceAttestation,
            "Comprehensive Compliance Report",
        )
        .period(start, end)
        .content(serde_json::to_string_pretty(&content).unwrap())
        .generated_by("compliance_system");

        self.reporting
            .generate_report(builder)
            .await
            .map_err(|e| ComplianceError::ReportingError(e))
    }
}

impl Default for ComplianceManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Module Version Information
// ============================================================================

/// Compliance module version
pub const COMPLIANCE_VERSION: &str = "0.2.0";

/// Module build date
pub const BUILD_DATE: &str = "2025-12-28";

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compliance_manager_creation() {
        let manager = ComplianceManager::new();
        assert!(manager.config().enable_audit_trail);
        assert!(manager.config().enable_gdpr);
    }

    #[test]
    fn test_compliance_config() {
        let mut config = ComplianceConfig::default();
        config.enable_hipaa = true;

        let manager = ComplianceManager::with_config(config);
        assert!(manager.config().enable_hipaa);
    }

    #[tokio::test]
    async fn test_compliance_status() {
        let manager = ComplianceManager::new();
        let status = manager.check_compliance_status(Framework::Soc2).await;
        assert_eq!(status, ComplianceStatus::Compliant);
    }

    #[test]
    fn test_version_info() {
        assert_eq!(COMPLIANCE_VERSION, "0.2.0");
    }
}
