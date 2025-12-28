//! GDPR (General Data Protection Regulation) compliance support
//!
//! This module implements GDPR requirements including data subject access requests,
//! right to deletion, data portability, consent tracking, and processing records.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Legal basis for processing personal data under GDPR
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LegalBasis {
    /// Processing is necessary for contract performance
    Contract,
    /// Data subject has given consent
    Consent,
    /// Processing is necessary for legal obligation
    LegalObligation,
    /// Processing is necessary to protect vital interests
    VitalInterests,
    /// Processing is necessary for public interest
    PublicInterest,
    /// Processing is necessary for legitimate interests
    LegitimateInterests,
}

/// Types of personal data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PersonalDataType {
    /// Basic identifying information
    IdentifyingData,
    /// Contact information
    ContactData,
    /// Financial information
    FinancialData,
    /// Location data
    LocationData,
    /// Usage/behavioral data
    BehavioralData,
    /// Special category data (sensitive)
    SpecialCategory,
    /// Health data
    HealthData,
    /// Biometric data
    BiometricData,
}

/// Consent record for GDPR compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRecord {
    /// Unique consent ID
    pub id: Uuid,

    /// Data subject ID
    pub subject_id: String,

    /// Purpose of processing
    pub purpose: String,

    /// Legal basis
    pub legal_basis: LegalBasis,

    /// Types of data covered
    pub data_types: Vec<PersonalDataType>,

    /// When consent was given
    pub granted_at: DateTime<Utc>,

    /// When consent was withdrawn (if applicable)
    pub withdrawn_at: Option<DateTime<Utc>>,

    /// Whether consent is currently active
    pub is_active: bool,

    /// Version of privacy policy consented to
    pub policy_version: String,

    /// How consent was obtained
    pub consent_method: String,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ConsentRecord {
    /// Create a new consent record
    pub fn new(
        subject_id: impl Into<String>,
        purpose: impl Into<String>,
        legal_basis: LegalBasis,
        data_types: Vec<PersonalDataType>,
        policy_version: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            subject_id: subject_id.into(),
            purpose: purpose.into(),
            legal_basis,
            data_types,
            granted_at: Utc::now(),
            withdrawn_at: None,
            is_active: true,
            policy_version: policy_version.into(),
            consent_method: "explicit".to_string(),
            metadata: HashMap::new(),
        }
    }

    /// Withdraw consent
    pub fn withdraw(&mut self) {
        self.is_active = false;
        self.withdrawn_at = Some(Utc::now());
    }
}

/// Data Subject Access Request (DSAR)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSubjectAccessRequest {
    /// Unique request ID
    pub id: Uuid,

    /// Data subject ID
    pub subject_id: String,

    /// Type of request
    pub request_type: DsarType,

    /// When request was submitted
    pub submitted_at: DateTime<Utc>,

    /// Request status
    pub status: DsarStatus,

    /// When request was completed
    pub completed_at: Option<DateTime<Utc>>,

    /// Verification status
    pub verified: bool,

    /// Verification method used
    pub verification_method: Option<String>,

    /// Notes/comments
    pub notes: Vec<String>,

    /// Data collected for export (if applicable)
    pub exported_data: Option<String>,
}

/// Types of GDPR data subject requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DsarType {
    /// Right to access (Article 15)
    Access,
    /// Right to rectification (Article 16)
    Rectification,
    /// Right to erasure (Article 17)
    Erasure,
    /// Right to restriction of processing (Article 18)
    Restriction,
    /// Right to data portability (Article 20)
    Portability,
    /// Right to object (Article 21)
    Objection,
}

/// Status of a DSAR
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DsarStatus {
    /// Request submitted, pending verification
    Pending,
    /// Identity verified, processing request
    Processing,
    /// Request completed
    Completed,
    /// Request rejected
    Rejected,
    /// Request cancelled by subject
    Cancelled,
}

/// Processing activity record (Article 30)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingActivity {
    /// Unique activity ID
    pub id: Uuid,

    /// Name of processing activity
    pub name: String,

    /// Purpose of processing
    pub purposes: Vec<String>,

    /// Categories of data subjects
    pub data_subject_categories: Vec<String>,

    /// Categories of personal data
    pub personal_data_categories: Vec<PersonalDataType>,

    /// Categories of recipients
    pub recipients: Vec<String>,

    /// International transfers
    pub international_transfers: Vec<InternationalTransfer>,

    /// Retention periods
    pub retention_period: String,

    /// Technical and organizational measures
    pub security_measures: Vec<String>,

    /// Legal basis
    pub legal_basis: LegalBasis,

    /// Data Protection Impact Assessment (DPIA) required
    pub dpia_required: bool,

    /// DPIA reference
    pub dpia_reference: Option<String>,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Last updated
    pub updated_at: DateTime<Utc>,
}

/// International data transfer record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternationalTransfer {
    /// Destination country
    pub country: String,

    /// Safeguard used (e.g., Standard Contractual Clauses)
    pub safeguard: String,

    /// Transfer mechanism
    pub mechanism: String,
}

/// GDPR compliance manager
pub struct GdprManager {
    /// Consent records
    consents: Arc<RwLock<HashMap<String, Vec<ConsentRecord>>>>,

    /// Data subject access requests
    dsars: Arc<RwLock<HashMap<Uuid, DataSubjectAccessRequest>>>,

    /// Processing activities
    processing_activities: Arc<RwLock<HashMap<Uuid, ProcessingActivity>>>,
}

impl GdprManager {
    /// Create a new GDPR manager
    pub fn new() -> Self {
        Self {
            consents: Arc::new(RwLock::new(HashMap::new())),
            dsars: Arc::new(RwLock::new(HashMap::new())),
            processing_activities: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // ========================================================================
    // Consent Management
    // ========================================================================

    /// Record consent
    pub async fn record_consent(&self, consent: ConsentRecord) -> Result<Uuid, String> {
        let mut consents = self.consents.write().await;
        let subject_id = consent.subject_id.clone();
        let consent_id = consent.id;

        consents
            .entry(subject_id)
            .or_insert_with(Vec::new)
            .push(consent);

        Ok(consent_id)
    }

    /// Withdraw consent
    pub async fn withdraw_consent(&self, subject_id: &str, consent_id: Uuid) -> Result<(), String> {
        let mut consents = self.consents.write().await;

        if let Some(subject_consents) = consents.get_mut(subject_id) {
            if let Some(consent) = subject_consents.iter_mut().find(|c| c.id == consent_id) {
                consent.withdraw();
                return Ok(());
            }
        }

        Err("Consent not found".to_string())
    }

    /// Get active consents for a subject
    pub async fn get_active_consents(&self, subject_id: &str) -> Vec<ConsentRecord> {
        let consents = self.consents.read().await;
        consents
            .get(subject_id)
            .map(|c| c.iter().filter(|c| c.is_active).cloned().collect())
            .unwrap_or_default()
    }

    /// Check if processing is allowed
    pub async fn is_processing_allowed(
        &self,
        subject_id: &str,
        purpose: &str,
    ) -> bool {
        let consents = self.consents.read().await;
        if let Some(subject_consents) = consents.get(subject_id) {
            subject_consents
                .iter()
                .any(|c| c.is_active && c.purpose == purpose)
        } else {
            false
        }
    }

    // ========================================================================
    // Data Subject Access Requests (DSAR)
    // ========================================================================

    /// Submit a DSAR
    pub async fn submit_dsar(
        &self,
        subject_id: impl Into<String>,
        request_type: DsarType,
    ) -> Result<Uuid, String> {
        let dsar = DataSubjectAccessRequest {
            id: Uuid::new_v4(),
            subject_id: subject_id.into(),
            request_type,
            submitted_at: Utc::now(),
            status: DsarStatus::Pending,
            completed_at: None,
            verified: false,
            verification_method: None,
            notes: Vec::new(),
            exported_data: None,
        };

        let dsar_id = dsar.id;
        let mut dsars = self.dsars.write().await;
        dsars.insert(dsar_id, dsar);

        Ok(dsar_id)
    }

    /// Verify a DSAR
    pub async fn verify_dsar(&self, dsar_id: Uuid, method: impl Into<String>) -> Result<(), String> {
        let mut dsars = self.dsars.write().await;
        if let Some(dsar) = dsars.get_mut(&dsar_id) {
            dsar.verified = true;
            dsar.verification_method = Some(method.into());
            dsar.status = DsarStatus::Processing;
            Ok(())
        } else {
            Err("DSAR not found".to_string())
        }
    }

    /// Complete a DSAR
    pub async fn complete_dsar(
        &self,
        dsar_id: Uuid,
        exported_data: Option<String>,
    ) -> Result<(), String> {
        let mut dsars = self.dsars.write().await;
        if let Some(dsar) = dsars.get_mut(&dsar_id) {
            dsar.status = DsarStatus::Completed;
            dsar.completed_at = Some(Utc::now());
            dsar.exported_data = exported_data;
            Ok(())
        } else {
            Err("DSAR not found".to_string())
        }
    }

    /// Get DSAR by ID
    pub async fn get_dsar(&self, dsar_id: Uuid) -> Option<DataSubjectAccessRequest> {
        let dsars = self.dsars.read().await;
        dsars.get(&dsar_id).cloned()
    }

    /// Get all DSARs for a subject
    pub async fn get_dsars_for_subject(&self, subject_id: &str) -> Vec<DataSubjectAccessRequest> {
        let dsars = self.dsars.read().await;
        dsars
            .values()
            .filter(|d| d.subject_id == subject_id)
            .cloned()
            .collect()
    }

    /// Export data for portability request
    pub async fn export_subject_data(&self, subject_id: &str) -> Result<String, String> {
        // In production, this would collect data from all systems
        let consents = self.get_active_consents(subject_id).await;
        let dsars = self.get_dsars_for_subject(subject_id).await;

        let export = serde_json::json!({
            "subject_id": subject_id,
            "exported_at": Utc::now(),
            "consents": consents,
            "requests": dsars,
        });

        serde_json::to_string_pretty(&export).map_err(|e| e.to_string())
    }

    /// Delete subject data (right to erasure)
    pub async fn delete_subject_data(&self, subject_id: &str) -> Result<(), String> {
        // Withdraw all consents
        let mut consents = self.consents.write().await;
        if let Some(subject_consents) = consents.get_mut(subject_id) {
            for consent in subject_consents.iter_mut() {
                consent.withdraw();
            }
        }

        // In production, this would trigger deletion across all systems
        // while respecting legal retention requirements

        Ok(())
    }

    // ========================================================================
    // Processing Activities (Article 30)
    // ========================================================================

    /// Register a processing activity
    pub async fn register_processing_activity(
        &self,
        activity: ProcessingActivity,
    ) -> Result<Uuid, String> {
        let activity_id = activity.id;
        let mut activities = self.processing_activities.write().await;
        activities.insert(activity_id, activity);
        Ok(activity_id)
    }

    /// Get processing activity
    pub async fn get_processing_activity(&self, id: Uuid) -> Option<ProcessingActivity> {
        let activities = self.processing_activities.read().await;
        activities.get(&id).cloned()
    }

    /// Get all processing activities
    pub async fn get_all_processing_activities(&self) -> Vec<ProcessingActivity> {
        let activities = self.processing_activities.read().await;
        activities.values().cloned().collect()
    }

    /// Generate Article 30 record of processing activities
    pub async fn generate_ropa(&self) -> String {
        let activities = self.get_all_processing_activities().await;

        let ropa = serde_json::json!({
            "title": "Record of Processing Activities (Article 30 GDPR)",
            "generated_at": Utc::now(),
            "activities": activities,
        });

        serde_json::to_string_pretty(&ropa).unwrap_or_default()
    }

    /// Generate GDPR audit report
    pub async fn generate_audit_report(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> String {
        let activities = self.get_all_processing_activities().await;
        let consents = self.consents.read().await;
        let dsars = self.dsars.read().await;

        let report = serde_json::json!({
            "title": "GDPR Compliance Audit Report",
            "period": {
                "start": start,
                "end": end,
            },
            "summary": {
                "total_processing_activities": activities.len(),
                "total_consents": consents.values().map(|v| v.len()).sum::<usize>(),
                "total_dsars": dsars.len(),
            },
            "processing_activities": activities,
            "data_subject_requests": dsars.values().collect::<Vec<_>>(),
        });

        serde_json::to_string_pretty(&report).unwrap_or_default()
    }
}

impl Default for GdprManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_consent_management() {
        let manager = GdprManager::new();

        let consent = ConsentRecord::new(
            "user123",
            "analytics",
            LegalBasis::Consent,
            vec![PersonalDataType::BehavioralData],
            "v1.0",
        );

        let consent_id = manager.record_consent(consent).await.unwrap();
        assert!(manager.is_processing_allowed("user123", "analytics").await);

        manager.withdraw_consent("user123", consent_id).await.unwrap();
        assert!(!manager.is_processing_allowed("user123", "analytics").await);
    }

    #[tokio::test]
    async fn test_dsar_workflow() {
        let manager = GdprManager::new();

        let dsar_id = manager.submit_dsar("user123", DsarType::Access).await.unwrap();

        let dsar = manager.get_dsar(dsar_id).await.unwrap();
        assert_eq!(dsar.status, DsarStatus::Pending);

        manager.verify_dsar(dsar_id, "email_verification").await.unwrap();

        let dsar = manager.get_dsar(dsar_id).await.unwrap();
        assert_eq!(dsar.status, DsarStatus::Processing);
        assert!(dsar.verified);
    }

    #[tokio::test]
    async fn test_data_export() {
        let manager = GdprManager::new();

        let consent = ConsentRecord::new(
            "user123",
            "analytics",
            LegalBasis::Consent,
            vec![PersonalDataType::BehavioralData],
            "v1.0",
        );

        manager.record_consent(consent).await.unwrap();
        let export = manager.export_subject_data("user123").await.unwrap();

        assert!(export.contains("user123"));
        assert!(export.contains("consents"));
    }

    #[tokio::test]
    async fn test_processing_activity() {
        let manager = GdprManager::new();

        let activity = ProcessingActivity {
            id: Uuid::new_v4(),
            name: "User Analytics".to_string(),
            purposes: vec!["Service improvement".to_string()],
            data_subject_categories: vec!["Users".to_string()],
            personal_data_categories: vec![PersonalDataType::BehavioralData],
            recipients: vec!["Internal team".to_string()],
            international_transfers: vec![],
            retention_period: "2 years".to_string(),
            security_measures: vec!["Encryption".to_string()],
            legal_basis: LegalBasis::LegitimateInterests,
            dpia_required: false,
            dpia_reference: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let activity_id = manager.register_processing_activity(activity).await.unwrap();
        let retrieved = manager.get_processing_activity(activity_id).await.unwrap();

        assert_eq!(retrieved.name, "User Analytics");
    }
}
