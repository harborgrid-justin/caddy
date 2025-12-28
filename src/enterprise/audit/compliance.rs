//! Compliance features for audit logging
//!
//! Implements compliance helpers for SOX, GDPR, HIPAA, and other regulations.

use crate::enterprise::audit::{
    event::{AuditEvent, EventSeverity, EventType},
    query::{AuditQuery, QueryAggregation},
    storage::AuditStorage,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Compliance errors
#[derive(Debug, Error)]
pub enum ComplianceError {
    /// Query error
    #[error("Query error: {0}")]
    Query(#[from] crate::enterprise::audit::query::QueryError),

    /// Compliance violation detected
    #[error("Compliance violation: {0}")]
    Violation(String),

    /// Retention policy violation
    #[error("Retention policy violation: {0}")]
    RetentionViolation(String),
}

/// Result type for compliance operations
pub type Result<T> = std::result::Result<T, ComplianceError>;

/// Retention policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Minimum retention period in days
    pub min_retention_days: i64,

    /// Maximum retention period in days (0 = unlimited)
    pub max_retention_days: i64,

    /// Auto-archive after this many days
    pub archive_after_days: Option<i64>,

    /// Auto-delete after this many days (must be >= min_retention_days)
    pub delete_after_days: Option<i64>,

    /// Special retention for security events
    pub security_retention_days: Option<i64>,
}

impl RetentionPolicy {
    /// Create a SOX-compliant retention policy (7 years)
    pub fn sox_compliant() -> Self {
        Self {
            min_retention_days: 7 * 365,     // 7 years
            max_retention_days: 0,           // Unlimited
            archive_after_days: Some(365),   // Archive after 1 year
            delete_after_days: Some(10 * 365), // Delete after 10 years
            security_retention_days: Some(10 * 365),
        }
    }

    /// Create a GDPR-compliant retention policy
    pub fn gdpr_compliant() -> Self {
        Self {
            min_retention_days: 30,          // 30 days minimum
            max_retention_days: 365,         // 1 year maximum for personal data
            archive_after_days: Some(90),    // Archive after 90 days
            delete_after_days: Some(365),    // Delete after 1 year
            security_retention_days: Some(2 * 365), // 2 years for security
        }
    }

    /// Create a HIPAA-compliant retention policy (6 years)
    pub fn hipaa_compliant() -> Self {
        Self {
            min_retention_days: 6 * 365,     // 6 years
            max_retention_days: 0,           // Unlimited
            archive_after_days: Some(365),   // Archive after 1 year
            delete_after_days: Some(7 * 365), // Delete after 7 years
            security_retention_days: Some(10 * 365),
        }
    }

    /// Create a custom retention policy
    pub fn custom(min_days: i64, max_days: i64) -> Self {
        Self {
            min_retention_days: min_days,
            max_retention_days: max_days,
            archive_after_days: None,
            delete_after_days: None,
            security_retention_days: None,
        }
    }

    /// Check if an event should be archived
    pub fn should_archive(&self, event: &AuditEvent) -> bool {
        if let Some(archive_days) = self.archive_after_days {
            let age = Utc::now() - event.timestamp;
            age.num_days() >= archive_days
        } else {
            false
        }
    }

    /// Check if an event should be deleted
    pub fn should_delete(&self, event: &AuditEvent) -> bool {
        // Security events have special retention
        if event.severity == EventSeverity::Security {
            if let Some(security_days) = self.security_retention_days {
                let age = Utc::now() - event.timestamp;
                return age.num_days() >= security_days;
            }
        }

        // Normal deletion policy
        if let Some(delete_days) = self.delete_after_days {
            let age = Utc::now() - event.timestamp;
            age.num_days() >= delete_days
        } else {
            false
        }
    }

    /// Validate the policy configuration
    pub fn validate(&self) -> Result<()> {
        if self.min_retention_days < 0 {
            return Err(ComplianceError::RetentionViolation(
                "Minimum retention days cannot be negative".to_string(),
            ));
        }

        if self.max_retention_days < 0 {
            return Err(ComplianceError::RetentionViolation(
                "Maximum retention days cannot be negative".to_string(),
            ));
        }

        if self.max_retention_days > 0 && self.max_retention_days < self.min_retention_days {
            return Err(ComplianceError::RetentionViolation(
                "Maximum retention must be >= minimum retention".to_string(),
            ));
        }

        if let Some(delete_days) = self.delete_after_days {
            if delete_days < self.min_retention_days {
                return Err(ComplianceError::RetentionViolation(
                    "Delete days must be >= minimum retention days".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// SOX (Sarbanes-Oxley) compliance helper
pub struct SoxCompliance;

impl SoxCompliance {
    /// Get recommended retention policy
    pub fn retention_policy() -> RetentionPolicy {
        RetentionPolicy::sox_compliant()
    }

    /// Verify SOX compliance for financial transactions
    pub async fn verify_financial_controls<S>(
        storage: &S,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<ComplianceReport>
    where
        S: AuditStorage,
    {
        let events = AuditQuery::new()
            .time_range(start, end)
            .actions(vec![
                EventType::Create,
                EventType::Update,
                EventType::Delete,
            ])
            .execute(storage)
            .await?;

        let mut report = ComplianceReport::new("SOX Financial Controls");

        // Check for proper authorization
        let unauthorized: Vec<_> = events
            .iter()
            .filter(|e| {
                !e.success
                    || e.details.get("authorization").is_none()
            })
            .collect();

        if !unauthorized.is_empty() {
            report.add_violation(format!(
                "{} events without proper authorization",
                unauthorized.len()
            ));
        }

        // Check for segregation of duties
        let agg = QueryAggregation::new(events.clone());
        let by_user = agg.by_user();

        for (user, count) in by_user {
            if count > 100 {
                report.add_warning(format!(
                    "User {} performed {} actions (potential SoD violation)",
                    user, count
                ));
            }
        }

        // Check for change tracking
        let changes_without_tracking: Vec<_> = events
            .iter()
            .filter(|e| {
                e.action == EventType::Update
                    && (e.previous_state.is_none() || e.new_state.is_none())
            })
            .collect();

        if !changes_without_tracking.is_empty() {
            report.add_violation(format!(
                "{} updates without proper change tracking",
                changes_without_tracking.len()
            ));
        }

        report.compliant = report.violations.is_empty();
        Ok(report)
    }
}

/// GDPR (General Data Protection Regulation) compliance helper
pub struct GdprCompliance;

impl GdprCompliance {
    /// Get recommended retention policy
    pub fn retention_policy() -> RetentionPolicy {
        RetentionPolicy::gdpr_compliant()
    }

    /// Generate GDPR audit trail for a user (right to access)
    pub async fn user_audit_trail<S>(
        storage: &S,
        user_id: &str,
    ) -> Result<Vec<AuditEvent>>
    where
        S: AuditStorage,
    {
        let events = AuditQuery::new()
            .user_id(user_id)
            .last_days(365) // Last year
            .execute(storage)
            .await?;

        Ok(events)
    }

    /// Find all data related to a user for deletion (right to be forgotten)
    pub async fn find_user_data<S>(
        storage: &S,
        user_id: &str,
    ) -> Result<Vec<AuditEvent>>
    where
        S: AuditStorage,
    {
        let events = AuditQuery::new()
            .user_id(user_id)
            .execute(storage)
            .await?;

        Ok(events)
    }

    /// Verify GDPR compliance
    pub async fn verify_compliance<S>(
        storage: &S,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<ComplianceReport>
    where
        S: AuditStorage,
    {
        let events = AuditQuery::new()
            .time_range(start, end)
            .execute(storage)
            .await?;

        let mut report = ComplianceReport::new("GDPR Compliance");

        // Check for data access logging
        let access_events: Vec<_> = events
            .iter()
            .filter(|e| e.action == EventType::Read)
            .collect();

        report.add_info(format!(
            "{} data access events recorded",
            access_events.len()
        ));

        // Check for consent tracking
        let events_without_consent: Vec<_> = events
            .iter()
            .filter(|e| {
                matches!(e.action, EventType::Create | EventType::Update)
                    && e.details.get("consent").is_none()
                    && e.resource_type.as_deref() == Some("personal_data")
            })
            .collect();

        if !events_without_consent.is_empty() {
            report.add_warning(format!(
                "{} personal data operations without consent tracking",
                events_without_consent.len()
            ));
        }

        // Check retention policy adherence
        let policy = Self::retention_policy();
        let old_events: Vec<_> = events
            .iter()
            .filter(|e| policy.should_delete(e))
            .collect();

        if !old_events.is_empty() {
            report.add_violation(format!(
                "{} events exceed maximum retention period",
                old_events.len()
            ));
        }

        report.compliant = report.violations.is_empty();
        Ok(report)
    }
}

/// HIPAA (Health Insurance Portability and Accountability Act) compliance helper
pub struct HipaaCompliance;

impl HipaaCompliance {
    /// Get recommended retention policy
    pub fn retention_policy() -> RetentionPolicy {
        RetentionPolicy::hipaa_compliant()
    }

    /// Verify HIPAA compliance
    pub async fn verify_compliance<S>(
        storage: &S,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<ComplianceReport>
    where
        S: AuditStorage,
    {
        let events = AuditQuery::new()
            .time_range(start, end)
            .execute(storage)
            .await?;

        let mut report = ComplianceReport::new("HIPAA Compliance");

        // Check for PHI access logging
        let phi_access: Vec<_> = events
            .iter()
            .filter(|e| {
                e.resource_type.as_deref() == Some("phi")
                    || e.resource_type.as_deref() == Some("patient_record")
            })
            .collect();

        report.add_info(format!(
            "{} PHI access events recorded",
            phi_access.len()
        ));

        // Verify all PHI access includes user authentication
        let unauthenticated: Vec<_> = phi_access
            .iter()
            .filter(|e| e.session_id.is_none())
            .collect();

        if !unauthenticated.is_empty() {
            report.add_violation(format!(
                "{} PHI accesses without session tracking",
                unauthenticated.len()
            ));
        }

        // Check for encryption of sensitive data
        let unencrypted: Vec<_> = events
            .iter()
            .filter(|e| {
                matches!(e.action, EventType::Export | EventType::Share)
                    && e.details.get("encrypted") != Some(&"true".to_string())
                    && e.resource_type.as_deref() == Some("phi")
            })
            .collect();

        if !unencrypted.is_empty() {
            report.add_violation(format!(
                "{} PHI exports/shares without encryption",
                unencrypted.len()
            ));
        }

        // Check for security incident logging
        let security_events: Vec<_> = events
            .iter()
            .filter(|e| e.severity == EventSeverity::Security)
            .collect();

        report.add_info(format!(
            "{} security events recorded",
            security_events.len()
        ));

        report.compliant = report.violations.is_empty();
        Ok(report)
    }

    /// Generate audit log for patient record access
    pub async fn patient_access_log<S>(
        storage: &S,
        patient_id: &str,
        days: i64,
    ) -> Result<Vec<AuditEvent>>
    where
        S: AuditStorage,
    {
        let events = AuditQuery::new()
            .resource(format!("patient/{}", patient_id))
            .last_days(days)
            .execute(storage)
            .await?;

        Ok(events)
    }
}

/// Compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    /// Report name
    pub name: String,

    /// Timestamp of report generation
    pub generated_at: DateTime<Utc>,

    /// Overall compliance status
    pub compliant: bool,

    /// List of violations found
    pub violations: Vec<String>,

    /// List of warnings
    pub warnings: Vec<String>,

    /// Informational messages
    pub info: Vec<String>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ComplianceReport {
    /// Create a new compliance report
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            generated_at: Utc::now(),
            compliant: true,
            violations: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a violation
    pub fn add_violation(&mut self, message: impl Into<String>) {
        self.violations.push(message.into());
        self.compliant = false;
    }

    /// Add a warning
    pub fn add_warning(&mut self, message: impl Into<String>) {
        self.warnings.push(message.into());
    }

    /// Add informational message
    pub fn add_info(&mut self, message: impl Into<String>) {
        self.info.push(message.into());
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

    /// Generate a summary of the report
    pub fn summary(&self) -> String {
        format!(
            "Compliance Report: {}\n\
             Generated: {}\n\
             Status: {}\n\
             Violations: {}\n\
             Warnings: {}\n",
            self.name,
            self.generated_at,
            if self.compliant { "COMPLIANT" } else { "NON-COMPLIANT" },
            self.violations.len(),
            self.warnings.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retention_policy_sox() {
        let policy = RetentionPolicy::sox_compliant();
        assert_eq!(policy.min_retention_days, 7 * 365);
        assert!(policy.validate().is_ok());
    }

    #[test]
    fn test_retention_policy_gdpr() {
        let policy = RetentionPolicy::gdpr_compliant();
        assert_eq!(policy.min_retention_days, 30);
        assert_eq!(policy.max_retention_days, 365);
        assert!(policy.validate().is_ok());
    }

    #[test]
    fn test_retention_policy_hipaa() {
        let policy = RetentionPolicy::hipaa_compliant();
        assert_eq!(policy.min_retention_days, 6 * 365);
        assert!(policy.validate().is_ok());
    }

    #[test]
    fn test_compliance_report() {
        let mut report = ComplianceReport::new("Test Report");
        assert!(report.compliant);

        report.add_warning("Test warning");
        assert!(report.compliant);

        report.add_violation("Test violation");
        assert!(!report.compliant);

        assert_eq!(report.violations.len(), 1);
        assert_eq!(report.warnings.len(), 1);
    }
}
