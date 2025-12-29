//! HIPAA (Health Insurance Portability and Accountability Act) compliance
//!
//! This module implements HIPAA requirements for Protected Health Information (PHI)
//! including access logging, minimum necessary rule, breach notification, and
//! business associate tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Type of Protected Health Information (PHI)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PhiType {
    /// Names
    Names,
    /// Geographic subdivisions smaller than state
    GeographicData,
    /// Dates (birth, admission, discharge, death)
    Dates,
    /// Telephone numbers
    TelephoneNumbers,
    /// Fax numbers
    FaxNumbers,
    /// Email addresses
    EmailAddresses,
    /// Social Security numbers
    SocialSecurityNumbers,
    /// Medical record numbers
    MedicalRecordNumbers,
    /// Health plan beneficiary numbers
    HealthPlanNumbers,
    /// Account numbers
    AccountNumbers,
    /// Certificate/license numbers
    CertificateNumbers,
    /// Vehicle identifiers
    VehicleIdentifiers,
    /// Device identifiers and serial numbers
    DeviceIdentifiers,
    /// Web URLs
    WebUrls,
    /// IP addresses
    IpAddresses,
    /// Biometric identifiers
    BiometricIdentifiers,
    /// Full face photos
    FullFacePhotos,
    /// Any other unique identifying number or code
    OtherIdentifiers,
}

/// PHI access log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhiAccessLog {
    /// Unique log entry ID
    pub id: Uuid,

    /// Timestamp of access
    pub timestamp: DateTime<Utc>,

    /// User who accessed PHI
    pub user_id: String,

    /// Patient/subject whose PHI was accessed
    pub patient_id: String,

    /// Type of PHI accessed
    pub phi_types: Vec<PhiType>,

    /// Purpose of access
    pub access_purpose: AccessPurpose,

    /// Specific purpose description
    pub purpose_description: String,

    /// Action performed
    pub action: PhiAction,

    /// Data accessed
    pub data_accessed: String,

    /// Whether minimum necessary rule was applied
    pub minimum_necessary_applied: bool,

    /// Justification for access
    pub justification: Option<String>,

    /// Source IP address
    pub source_ip: Option<String>,

    /// Session ID
    pub session_id: Option<String>,

    /// Whether access was authorized
    pub authorized: bool,
}

/// Purpose of PHI access
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessPurpose {
    /// Treatment of patient
    Treatment,
    /// Payment processing
    Payment,
    /// Healthcare operations
    HealthcareOperations,
    /// Research (with appropriate authorization)
    Research,
    /// Public health activities
    PublicHealth,
    /// Required by law
    LegalRequirement,
    /// Patient request
    PatientRequest,
    /// Emergency circumstances
    Emergency,
}

/// Action performed on PHI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhiAction {
    /// View/read PHI
    View,
    /// Create new PHI
    Create,
    /// Update existing PHI
    Update,
    /// Delete PHI
    Delete,
    /// Export/download PHI
    Export,
    /// Print PHI
    Print,
    /// Share/disclose PHI
    Disclose,
}

/// Business Associate Agreement (BAA) record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessAssociate {
    /// Unique BA ID
    pub id: Uuid,

    /// Business associate name
    pub name: String,

    /// Contact information
    pub contact_email: String,

    /// Contact phone
    pub contact_phone: String,

    /// BAA effective date
    pub effective_date: DateTime<Utc>,

    /// BAA expiration date
    pub expiration_date: Option<DateTime<Utc>>,

    /// Services provided
    pub services: Vec<String>,

    /// Types of PHI shared
    pub phi_types: Vec<PhiType>,

    /// BAA document reference
    pub baa_document: String,

    /// Whether BA has been audited
    pub audited: bool,

    /// Last audit date
    pub last_audit_date: Option<DateTime<Utc>>,

    /// Status
    pub status: BaaStatus,
}

/// BAA status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BaaStatus {
    /// Active and valid
    Active,
    /// Pending signature
    Pending,
    /// Expired
    Expired,
    /// Terminated
    Terminated,
}

/// HIPAA breach record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreachRecord {
    /// Unique breach ID
    pub id: Uuid,

    /// Discovery date
    pub discovered_at: DateTime<Utc>,

    /// Breach occurrence date (if known)
    pub occurred_at: Option<DateTime<Utc>>,

    /// Breach severity
    pub severity: BreachSeverity,

    /// Number of individuals affected
    pub individuals_affected: u32,

    /// Types of PHI involved
    pub phi_types: Vec<PhiType>,

    /// Description of breach
    pub description: String,

    /// Cause of breach
    pub cause: BreachCause,

    /// Location of breach
    pub location: String,

    /// Whether breach affects 500+ individuals
    pub major_breach: bool,

    /// Notification status
    pub notification_status: NotificationStatus,

    /// OCR (Office for Civil Rights) notification date
    pub ocr_notified_at: Option<DateTime<Utc>>,

    /// Individuals notified date
    pub individuals_notified_at: Option<DateTime<Utc>>,

    /// Media notification date (if major breach)
    pub media_notified_at: Option<DateTime<Utc>>,

    /// Mitigation actions taken
    pub mitigation_actions: Vec<String>,

    /// Whether breach was contained
    pub contained: bool,

    /// Investigation status
    pub investigation_status: InvestigationStatus,
}

/// Breach severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BreachSeverity {
    /// Low risk to individuals
    Low,
    /// Medium risk
    Medium,
    /// High risk
    High,
    /// Critical/severe risk
    Critical,
}

/// Cause of breach
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BreachCause {
    /// Unauthorized access/disclosure
    UnauthorizedAccess,
    /// Theft
    Theft,
    /// Loss of device/media
    Loss,
    /// Hacking/IT incident
    HackingItIncident,
    /// Improper disposal
    ImproperDisposal,
    /// Other
    Other,
}

/// Notification status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationStatus {
    /// Not yet notified
    NotNotified,
    /// Notification in progress
    InProgress,
    /// All notifications completed
    Completed,
}

/// Investigation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvestigationStatus {
    /// New/not started
    New,
    /// Under investigation
    InProgress,
    /// Investigation completed
    Completed,
    /// Reported to authorities
    Reported,
}

/// Minimum necessary determination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinimumNecessaryDetermination {
    /// Unique ID
    pub id: Uuid,

    /// Role or job function
    pub role: String,

    /// Purpose of access
    pub purpose: AccessPurpose,

    /// Allowed PHI types
    pub allowed_phi_types: Vec<PhiType>,

    /// Allowed actions
    pub allowed_actions: Vec<PhiAction>,

    /// Additional restrictions
    pub restrictions: Vec<String>,

    /// Created date
    pub created_at: DateTime<Utc>,

    /// Last reviewed
    pub last_reviewed: DateTime<Utc>,

    /// Approved by
    pub approved_by: String,
}

/// HIPAA compliance manager
pub struct HipaaManager {
    /// PHI access logs
    access_logs: Arc<RwLock<Vec<PhiAccessLog>>>,

    /// Business associates
    business_associates: Arc<RwLock<HashMap<Uuid, BusinessAssociate>>>,

    /// Breach records
    breaches: Arc<RwLock<HashMap<Uuid, BreachRecord>>>,

    /// Minimum necessary determinations
    min_necessary: Arc<RwLock<HashMap<String, MinimumNecessaryDetermination>>>,
}

impl HipaaManager {
    /// Create new HIPAA manager
    pub fn new() -> Self {
        Self {
            access_logs: Arc::new(RwLock::new(Vec::new())),
            business_associates: Arc::new(RwLock::new(HashMap::new())),
            breaches: Arc::new(RwLock::new(HashMap::new())),
            min_necessary: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // ========================================================================
    // PHI Access Logging
    // ========================================================================

    /// Log PHI access
    pub async fn log_phi_access(&self, log: PhiAccessLog) -> Result<Uuid, String> {
        let log_id = log.id;
        let mut logs = self.access_logs.write().await;
        logs.push(log);
        Ok(log_id)
    }

    /// Get access logs for a patient
    pub async fn get_patient_access_logs(&self, patient_id: &str) -> Vec<PhiAccessLog> {
        let logs = self.access_logs.read().await;
        logs.iter()
            .filter(|l| l.patient_id == patient_id)
            .cloned()
            .collect()
    }

    /// Get access logs for a user
    pub async fn get_user_access_logs(&self, user_id: &str) -> Vec<PhiAccessLog> {
        let logs = self.access_logs.read().await;
        logs.iter()
            .filter(|l| l.user_id == user_id)
            .cloned()
            .collect()
    }

    /// Get unauthorized access attempts
    pub async fn get_unauthorized_access_attempts(&self) -> Vec<PhiAccessLog> {
        let logs = self.access_logs.read().await;
        logs.iter()
            .filter(|l| !l.authorized)
            .cloned()
            .collect()
    }

    /// Get access logs in time range
    pub async fn get_access_logs_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<PhiAccessLog> {
        let logs = self.access_logs.read().await;
        logs.iter()
            .filter(|l| l.timestamp >= start && l.timestamp <= end)
            .cloned()
            .collect()
    }

    // ========================================================================
    // Minimum Necessary Rule
    // ========================================================================

    /// Set minimum necessary determination for a role
    pub async fn set_minimum_necessary(
        &self,
        determination: MinimumNecessaryDetermination,
    ) -> Result<(), String> {
        let mut min_necessary = self.min_necessary.write().await;
        min_necessary.insert(determination.role.clone(), determination);
        Ok(())
    }

    /// Check if access complies with minimum necessary rule
    pub async fn check_minimum_necessary(
        &self,
        role: &str,
        purpose: AccessPurpose,
        phi_type: PhiType,
        action: PhiAction,
    ) -> bool {
        let min_necessary = self.min_necessary.read().await;

        if let Some(determination) = min_necessary.get(role) {
            determination.purpose == purpose
                && determination.allowed_phi_types.contains(&phi_type)
                && determination.allowed_actions.contains(&action)
        } else {
            // If no determination exists, deny access
            false
        }
    }

    /// Get minimum necessary determination for role
    pub async fn get_minimum_necessary(&self, role: &str) -> Option<MinimumNecessaryDetermination> {
        let min_necessary = self.min_necessary.read().await;
        min_necessary.get(role).cloned()
    }

    // ========================================================================
    // Business Associate Management
    // ========================================================================

    /// Register a business associate
    pub async fn register_business_associate(&self, ba: BusinessAssociate) -> Result<Uuid, String> {
        let ba_id = ba.id;
        let mut bas = self.business_associates.write().await;
        bas.insert(ba_id, ba);
        Ok(ba_id)
    }

    /// Get business associate
    pub async fn get_business_associate(&self, id: Uuid) -> Option<BusinessAssociate> {
        let bas = self.business_associates.read().await;
        bas.get(&id).cloned()
    }

    /// Get all active business associates
    pub async fn get_active_business_associates(&self) -> Vec<BusinessAssociate> {
        let bas = self.business_associates.read().await;
        bas.values()
            .filter(|ba| ba.status == BaaStatus::Active)
            .cloned()
            .collect()
    }

    /// Update BA status
    pub async fn update_ba_status(&self, id: Uuid, status: BaaStatus) -> Result<(), String> {
        let mut bas = self.business_associates.write().await;
        if let Some(ba) = bas.get_mut(&id) {
            ba.status = status;
            Ok(())
        } else {
            Err("Business associate not found".to_string())
        }
    }

    /// Get BAs requiring audit
    pub async fn get_bas_requiring_audit(&self) -> Vec<BusinessAssociate> {
        let bas = self.business_associates.read().await;
        let now = Utc::now();
        let one_year_ago = now - chrono::Duration::days(365);

        bas.values()
            .filter(|ba| {
                ba.status == BaaStatus::Active
                    && (ba.last_audit_date.is_none()
                        || ba.last_audit_date.unwrap() < one_year_ago)
            })
            .cloned()
            .collect()
    }

    // ========================================================================
    // Breach Notification
    // ========================================================================

    /// Report a breach
    pub async fn report_breach(&self, breach: BreachRecord) -> Result<Uuid, String> {
        let breach_id = breach.id;
        let mut breaches = self.breaches.write().await;
        breaches.insert(breach_id, breach);
        Ok(breach_id)
    }

    /// Update breach notification status
    pub async fn update_breach_notification(
        &self,
        breach_id: Uuid,
        status: NotificationStatus,
    ) -> Result<(), String> {
        let mut breaches = self.breaches.write().await;
        if let Some(breach) = breaches.get_mut(&breach_id) {
            breach.notification_status = status;

            // Automatically set notification dates
            if status == NotificationStatus::Completed {
                let now = Utc::now();
                if breach.individuals_notified_at.is_none() {
                    breach.individuals_notified_at = Some(now);
                }
                if breach.major_breach && breach.media_notified_at.is_none() {
                    breach.media_notified_at = Some(now);
                }
                if breach.ocr_notified_at.is_none() {
                    breach.ocr_notified_at = Some(now);
                }
            }

            Ok(())
        } else {
            Err("Breach not found".to_string())
        }
    }

    /// Get active breaches (not completed)
    pub async fn get_active_breaches(&self) -> Vec<BreachRecord> {
        let breaches = self.breaches.read().await;
        breaches
            .values()
            .filter(|b| b.notification_status != NotificationStatus::Completed)
            .cloned()
            .collect()
    }

    /// Get major breaches (500+ individuals)
    pub async fn get_major_breaches(&self) -> Vec<BreachRecord> {
        let breaches = self.breaches.read().await;
        breaches
            .values()
            .filter(|b| b.major_breach)
            .cloned()
            .collect()
    }

    /// Check if breach notification is overdue
    pub async fn get_overdue_notifications(&self) -> Vec<BreachRecord> {
        let breaches = self.breaches.read().await;
        let now = Utc::now();

        breaches
            .values()
            .filter(|b| {
                b.notification_status != NotificationStatus::Completed
                    && (now - b.discovered_at).num_days() > 60
            })
            .cloned()
            .collect()
    }

    // ========================================================================
    // Reporting
    // ========================================================================

    /// Generate HIPAA audit report
    pub async fn generate_audit_report(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> String {
        let access_logs = self.get_access_logs_in_range(start, end).await;
        let unauthorized_attempts = self.get_unauthorized_access_attempts().await;
        let active_bas = self.get_active_business_associates().await;
        let active_breaches = self.get_active_breaches().await;

        let report = serde_json::json!({
            "title": "HIPAA Compliance Audit Report",
            "period": {
                "start": start,
                "end": end,
            },
            "summary": {
                "total_phi_accesses": access_logs.len(),
                "unauthorized_attempts": unauthorized_attempts.len(),
                "active_business_associates": active_bas.len(),
                "active_breaches": active_breaches.len(),
            },
            "access_logs": access_logs,
            "unauthorized_attempts": unauthorized_attempts,
            "business_associates": active_bas,
            "breaches": active_breaches,
        });

        serde_json::to_string_pretty(&report).unwrap_or_default()
    }

    /// Generate breach notification report for OCR
    pub async fn generate_breach_notification_report(&self) -> String {
        let major_breaches = self.get_major_breaches().await;

        let report = serde_json::json!({
            "title": "HIPAA Breach Notification Report",
            "generated_at": Utc::now(),
            "breaches": major_breaches,
        });

        serde_json::to_string_pretty(&report).unwrap_or_default()
    }
}

impl Default for HipaaManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_phi_access_logging() {
        let manager = HipaaManager::new();

        let log = PhiAccessLog {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            user_id: "doctor1".to_string(),
            patient_id: "patient123".to_string(),
            phi_types: vec![PhiType::MedicalRecordNumbers],
            access_purpose: AccessPurpose::Treatment,
            purpose_description: "Review medical history".to_string(),
            action: PhiAction::View,
            data_accessed: "Medical record".to_string(),
            minimum_necessary_applied: true,
            justification: Some("Treatment of patient".to_string()),
            source_ip: Some("192.168.1.1".to_string()),
            session_id: Some("session123".to_string()),
            authorized: true,
        };

        manager.log_phi_access(log).await.unwrap();

        let patient_logs = manager.get_patient_access_logs("patient123").await;
        assert_eq!(patient_logs.len(), 1);
    }

    #[tokio::test]
    async fn test_minimum_necessary() {
        let manager = HipaaManager::new();

        let determination = MinimumNecessaryDetermination {
            id: Uuid::new_v4(),
            role: "nurse".to_string(),
            purpose: AccessPurpose::Treatment,
            allowed_phi_types: vec![PhiType::MedicalRecordNumbers, PhiType::Names],
            allowed_actions: vec![PhiAction::View],
            restrictions: vec![],
            created_at: Utc::now(),
            last_reviewed: Utc::now(),
            approved_by: "privacy_officer".to_string(),
        };

        manager.set_minimum_necessary(determination).await.unwrap();

        let allowed = manager
            .check_minimum_necessary(
                "nurse",
                AccessPurpose::Treatment,
                PhiType::MedicalRecordNumbers,
                PhiAction::View,
            )
            .await;

        assert!(allowed);

        let not_allowed = manager
            .check_minimum_necessary(
                "nurse",
                AccessPurpose::Treatment,
                PhiType::SocialSecurityNumbers,
                PhiAction::View,
            )
            .await;

        assert!(!not_allowed);
    }

    #[tokio::test]
    async fn test_breach_notification() {
        let manager = HipaaManager::new();

        let breach = BreachRecord {
            id: Uuid::new_v4(),
            discovered_at: Utc::now(),
            occurred_at: Some(Utc::now() - chrono::Duration::days(1)),
            severity: BreachSeverity::High,
            individuals_affected: 600,
            phi_types: vec![PhiType::MedicalRecordNumbers, PhiType::Names],
            description: "Unauthorized access to database".to_string(),
            cause: BreachCause::HackingItIncident,
            location: "Database server".to_string(),
            major_breach: true,
            notification_status: NotificationStatus::NotNotified,
            ocr_notified_at: None,
            individuals_notified_at: None,
            media_notified_at: None,
            mitigation_actions: vec!["Reset passwords".to_string()],
            contained: true,
            investigation_status: InvestigationStatus::InProgress,
        };

        let breach_id = manager.report_breach(breach).await.unwrap();

        manager
            .update_breach_notification(breach_id, NotificationStatus::Completed)
            .await
            .unwrap();

        let updated_breach = manager.breaches.read().await.get(&breach_id).cloned().unwrap();
        assert_eq!(updated_breach.notification_status, NotificationStatus::Completed);
        assert!(updated_breach.ocr_notified_at.is_some());
    }

    #[tokio::test]
    async fn test_business_associate_management() {
        let manager = HipaaManager::new();

        let ba = BusinessAssociate {
            id: Uuid::new_v4(),
            name: "Cloud Storage Provider".to_string(),
            contact_email: "contact@provider.com".to_string(),
            contact_phone: "555-1234".to_string(),
            effective_date: Utc::now(),
            expiration_date: Some(Utc::now() + chrono::Duration::days(365)),
            services: vec!["Data storage".to_string()],
            phi_types: vec![PhiType::MedicalRecordNumbers],
            baa_document: "BAA-2024-001".to_string(),
            audited: false,
            last_audit_date: None,
            status: BaaStatus::Active,
        };

        let ba_id = manager.register_business_associate(ba).await.unwrap();

        let active_bas = manager.get_active_business_associates().await;
        assert_eq!(active_bas.len(), 1);

        let requiring_audit = manager.get_bas_requiring_audit().await;
        assert_eq!(requiring_audit.len(), 1);
    }
}
