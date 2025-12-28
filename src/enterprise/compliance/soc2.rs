//! SOC 2 compliance controls and evidence collection
//!
//! This module implements SOC 2 Trust Service Criteria (TSC) controls including
//! security, availability, processing integrity, confidentiality, and privacy.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// SOC 2 Trust Service Category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrustServiceCategory {
    /// Security - Protection against unauthorized access
    Security,
    /// Availability - System availability for operation and use
    Availability,
    /// Processing Integrity - System processing is complete, valid, accurate, timely, and authorized
    ProcessingIntegrity,
    /// Confidentiality - Information designated as confidential is protected
    Confidentiality,
    /// Privacy - Personal information is collected, used, retained, disclosed, and disposed of properly
    Privacy,
}

/// SOC 2 control identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ControlId {
    /// Category
    pub category: TrustServiceCategory,
    /// Control number
    pub number: String,
}

impl ControlId {
    /// Create new control ID
    pub fn new(category: TrustServiceCategory, number: impl Into<String>) -> Self {
        Self {
            category,
            number: number.into(),
        }
    }

    /// Format as string (e.g., "CC6.1", "A1.2")
    pub fn to_string(&self) -> String {
        let prefix = match self.category {
            TrustServiceCategory::Security => "CC",
            TrustServiceCategory::Availability => "A",
            TrustServiceCategory::ProcessingIntegrity => "PI",
            TrustServiceCategory::Confidentiality => "C",
            TrustServiceCategory::Privacy => "P",
        };
        format!("{}{}", prefix, self.number)
    }
}

/// SOC 2 control definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Control {
    /// Unique control ID
    pub id: ControlId,

    /// Control title
    pub title: String,

    /// Control description
    pub description: String,

    /// Implementation details
    pub implementation: String,

    /// Responsible party
    pub owner: String,

    /// Frequency of control execution
    pub frequency: ControlFrequency,

    /// Evidence required
    pub evidence_requirements: Vec<String>,

    /// Related controls
    pub related_controls: Vec<ControlId>,

    /// Whether control is automated
    pub automated: bool,
}

/// Frequency of control execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlFrequency {
    /// Continuous/real-time
    Continuous,
    /// Daily
    Daily,
    /// Weekly
    Weekly,
    /// Monthly
    Monthly,
    /// Quarterly
    Quarterly,
    /// Annually
    Annually,
    /// On-demand/as needed
    OnDemand,
}

/// Control execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlExecution {
    /// Unique execution ID
    pub id: Uuid,

    /// Control executed
    pub control_id: ControlId,

    /// Execution timestamp
    pub executed_at: DateTime<Utc>,

    /// Who executed the control
    pub executed_by: String,

    /// Result of execution
    pub result: ControlResult,

    /// Evidence collected
    pub evidence: Vec<Evidence>,

    /// Notes/comments
    pub notes: Option<String>,

    /// Exceptions/deviations
    pub exceptions: Vec<String>,
}

/// Result of control execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlResult {
    /// Control passed
    Pass,
    /// Control failed
    Fail,
    /// Control not applicable
    NotApplicable,
    /// Control not tested
    NotTested,
}

/// Evidence item for control attestation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    /// Unique evidence ID
    pub id: Uuid,

    /// Evidence type
    pub evidence_type: EvidenceType,

    /// Description
    pub description: String,

    /// Reference to evidence artifact (file path, URL, etc.)
    pub reference: String,

    /// Collection timestamp
    pub collected_at: DateTime<Utc>,

    /// Collected by
    pub collected_by: String,

    /// Hash of evidence for integrity
    pub hash: Option<String>,
}

/// Type of evidence
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceType {
    /// Screenshot
    Screenshot,
    /// Log file
    LogFile,
    /// Configuration export
    ConfigExport,
    /// System output
    SystemOutput,
    /// Document
    Document,
    /// Email
    Email,
    /// Database query result
    QueryResult,
    /// Automated test result
    TestResult,
}

/// Access control evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlEvidence {
    /// Unique ID
    pub id: Uuid,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// User account reviewed
    pub user_id: String,

    /// Roles assigned
    pub roles: Vec<String>,

    /// Permissions granted
    pub permissions: Vec<String>,

    /// Review result
    pub review_result: String,

    /// Reviewer
    pub reviewed_by: String,

    /// Last login
    pub last_login: Option<DateTime<Utc>>,

    /// Account status
    pub account_status: String,
}

/// Change management log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeManagementLog {
    /// Unique change ID
    pub id: Uuid,

    /// Change timestamp
    pub timestamp: DateTime<Utc>,

    /// Change type
    pub change_type: ChangeType,

    /// Component changed
    pub component: String,

    /// Change description
    pub description: String,

    /// Change requestor
    pub requested_by: String,

    /// Approved by
    pub approved_by: Option<String>,

    /// Implemented by
    pub implemented_by: String,

    /// Change approval ticket
    pub approval_ticket: Option<String>,

    /// Testing performed
    pub testing: Option<String>,

    /// Rollback plan
    pub rollback_plan: Option<String>,

    /// Success status
    pub success: bool,
}

/// Type of change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// Configuration change
    Configuration,
    /// Code deployment
    CodeDeployment,
    /// Infrastructure change
    Infrastructure,
    /// Security patch
    SecurityPatch,
    /// Emergency change
    Emergency,
}

/// Security incident record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIncident {
    /// Unique incident ID
    pub id: Uuid,

    /// Detection timestamp
    pub detected_at: DateTime<Utc>,

    /// Incident severity
    pub severity: IncidentSeverity,

    /// Incident category
    pub category: String,

    /// Description
    pub description: String,

    /// Affected systems
    pub affected_systems: Vec<String>,

    /// Detection method
    pub detected_by: String,

    /// Assigned to
    pub assigned_to: Option<String>,

    /// Current status
    pub status: IncidentStatus,

    /// Resolution timestamp
    pub resolved_at: Option<DateTime<Utc>>,

    /// Resolution notes
    pub resolution: Option<String>,

    /// Root cause
    pub root_cause: Option<String>,

    /// Remediation actions
    pub remediation_actions: Vec<String>,
}

/// Incident severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IncidentSeverity {
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

/// Incident status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncidentStatus {
    /// New incident
    New,
    /// Under investigation
    Investigating,
    /// Containment in progress
    Containing,
    /// Eradication in progress
    Eradicating,
    /// Recovery in progress
    Recovering,
    /// Resolved
    Resolved,
    /// Closed
    Closed,
}

/// Availability metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilityMetric {
    /// Unique ID
    pub id: Uuid,

    /// Measurement timestamp
    pub timestamp: DateTime<Utc>,

    /// Service/component
    pub service: String,

    /// Uptime percentage
    pub uptime_percentage: f64,

    /// Total time period (seconds)
    pub period_seconds: u64,

    /// Downtime incidents
    pub downtime_incidents: Vec<DowntimeIncident>,
}

/// Downtime incident
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DowntimeIncident {
    /// Start time
    pub start: DateTime<Utc>,

    /// End time
    pub end: DateTime<Utc>,

    /// Duration in seconds
    pub duration_seconds: u64,

    /// Reason
    pub reason: String,

    /// Planned or unplanned
    pub planned: bool,
}

/// SOC 2 compliance manager
pub struct Soc2Manager {
    /// Control definitions
    controls: Arc<RwLock<HashMap<ControlId, Control>>>,

    /// Control executions
    executions: Arc<RwLock<Vec<ControlExecution>>>,

    /// Access control evidence
    access_evidence: Arc<RwLock<Vec<AccessControlEvidence>>>,

    /// Change management logs
    change_logs: Arc<RwLock<Vec<ChangeManagementLog>>>,

    /// Security incidents
    incidents: Arc<RwLock<HashMap<Uuid, SecurityIncident>>>,

    /// Availability metrics
    availability_metrics: Arc<RwLock<Vec<AvailabilityMetric>>>,
}

impl Soc2Manager {
    /// Create new SOC 2 manager
    pub fn new() -> Self {
        let manager = Self {
            controls: Arc::new(RwLock::new(HashMap::new())),
            executions: Arc::new(RwLock::new(Vec::new())),
            access_evidence: Arc::new(RwLock::new(Vec::new())),
            change_logs: Arc::new(RwLock::new(Vec::new())),
            incidents: Arc::new(RwLock::new(HashMap::new())),
            availability_metrics: Arc::new(RwLock::new(Vec::new())),
        };

        // Initialize with common SOC 2 controls
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                manager.initialize_common_controls().await;
            })
        });

        manager
    }

    /// Initialize common SOC 2 controls
    async fn initialize_common_controls(&self) {
        let common_controls = vec![
            Control {
                id: ControlId::new(TrustServiceCategory::Security, "6.1"),
                title: "Logical and Physical Access Controls".to_string(),
                description: "Restrict logical and physical access to authorized personnel".to_string(),
                implementation: "Role-based access control with MFA".to_string(),
                owner: "Security Team".to_string(),
                frequency: ControlFrequency::Continuous,
                evidence_requirements: vec!["Access logs".to_string(), "User list".to_string()],
                related_controls: vec![],
                automated: true,
            },
            Control {
                id: ControlId::new(TrustServiceCategory::Security, "6.2"),
                title: "System Operation Monitoring".to_string(),
                description: "Monitor system operations and detect anomalies".to_string(),
                implementation: "Automated monitoring and alerting".to_string(),
                owner: "Operations Team".to_string(),
                frequency: ControlFrequency::Continuous,
                evidence_requirements: vec!["Monitoring logs".to_string()],
                related_controls: vec![],
                automated: true,
            },
            Control {
                id: ControlId::new(TrustServiceCategory::Availability, "1.1"),
                title: "System Availability Monitoring".to_string(),
                description: "Monitor and maintain system availability".to_string(),
                implementation: "Uptime monitoring and incident response".to_string(),
                owner: "Operations Team".to_string(),
                frequency: ControlFrequency::Continuous,
                evidence_requirements: vec!["Uptime reports".to_string()],
                related_controls: vec![],
                automated: true,
            },
        ];

        let mut controls = self.controls.write().await;
        for control in common_controls {
            controls.insert(control.id.clone(), control);
        }
    }

    // ========================================================================
    // Control Management
    // ========================================================================

    /// Register a control
    pub async fn register_control(&self, control: Control) -> Result<(), String> {
        let mut controls = self.controls.write().await;
        controls.insert(control.id.clone(), control);
        Ok(())
    }

    /// Execute a control
    pub async fn execute_control(
        &self,
        control_id: ControlId,
        executed_by: impl Into<String>,
        result: ControlResult,
        evidence: Vec<Evidence>,
    ) -> Result<Uuid, String> {
        let execution = ControlExecution {
            id: Uuid::new_v4(),
            control_id,
            executed_at: Utc::now(),
            executed_by: executed_by.into(),
            result,
            evidence,
            notes: None,
            exceptions: Vec::new(),
        };

        let execution_id = execution.id;
        let mut executions = self.executions.write().await;
        executions.push(execution);

        Ok(execution_id)
    }

    /// Get control executions for a period
    pub async fn get_executions_for_period(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<ControlExecution> {
        let executions = self.executions.read().await;
        executions
            .iter()
            .filter(|e| e.executed_at >= start && e.executed_at <= end)
            .cloned()
            .collect()
    }

    // ========================================================================
    // Access Control Evidence
    // ========================================================================

    /// Record access control evidence
    pub async fn record_access_evidence(&self, evidence: AccessControlEvidence) -> Result<Uuid, String> {
        let evidence_id = evidence.id;
        let mut access_evidence = self.access_evidence.write().await;
        access_evidence.push(evidence);
        Ok(evidence_id)
    }

    /// Get access evidence for a period
    pub async fn get_access_evidence_for_period(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<AccessControlEvidence> {
        let evidence = self.access_evidence.read().await;
        evidence
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .cloned()
            .collect()
    }

    // ========================================================================
    // Change Management
    // ========================================================================

    /// Log a change
    pub async fn log_change(&self, change: ChangeManagementLog) -> Result<Uuid, String> {
        let change_id = change.id;
        let mut logs = self.change_logs.write().await;
        logs.push(change);
        Ok(change_id)
    }

    /// Get changes for a period
    pub async fn get_changes_for_period(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<ChangeManagementLog> {
        let logs = self.change_logs.read().await;
        logs
            .iter()
            .filter(|l| l.timestamp >= start && l.timestamp <= end)
            .cloned()
            .collect()
    }

    // ========================================================================
    // Incident Tracking
    // ========================================================================

    /// Record a security incident
    pub async fn record_incident(&self, incident: SecurityIncident) -> Result<Uuid, String> {
        let incident_id = incident.id;
        let mut incidents = self.incidents.write().await;
        incidents.insert(incident_id, incident);
        Ok(incident_id)
    }

    /// Update incident status
    pub async fn update_incident_status(
        &self,
        incident_id: Uuid,
        status: IncidentStatus,
    ) -> Result<(), String> {
        let mut incidents = self.incidents.write().await;
        if let Some(incident) = incidents.get_mut(&incident_id) {
            incident.status = status;
            if status == IncidentStatus::Resolved || status == IncidentStatus::Closed {
                incident.resolved_at = Some(Utc::now());
            }
            Ok(())
        } else {
            Err("Incident not found".to_string())
        }
    }

    /// Get open incidents
    pub async fn get_open_incidents(&self) -> Vec<SecurityIncident> {
        let incidents = self.incidents.read().await;
        incidents
            .values()
            .filter(|i| !matches!(i.status, IncidentStatus::Resolved | IncidentStatus::Closed))
            .cloned()
            .collect()
    }

    // ========================================================================
    // Availability Monitoring
    // ========================================================================

    /// Record availability metric
    pub async fn record_availability(&self, metric: AvailabilityMetric) -> Result<Uuid, String> {
        let metric_id = metric.id;
        let mut metrics = self.availability_metrics.write().await;
        metrics.push(metric);
        Ok(metric_id)
    }

    /// Calculate uptime for period
    pub async fn calculate_uptime(
        &self,
        service: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> f64 {
        let metrics = self.availability_metrics.read().await;
        let service_metrics: Vec<_> = metrics
            .iter()
            .filter(|m| m.service == service && m.timestamp >= start && m.timestamp <= end)
            .collect();

        if service_metrics.is_empty() {
            return 100.0;
        }

        let avg_uptime: f64 = service_metrics.iter().map(|m| m.uptime_percentage).sum::<f64>()
            / service_metrics.len() as f64;

        avg_uptime
    }

    // ========================================================================
    // Reporting
    // ========================================================================

    /// Generate control attestation report
    pub async fn generate_attestation_report(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> String {
        let executions = self.get_executions_for_period(start, end).await;
        let changes = self.get_changes_for_period(start, end).await;
        let incidents = self.incidents.read().await;

        let report = serde_json::json!({
            "title": "SOC 2 Control Attestation Report",
            "period": {
                "start": start,
                "end": end,
            },
            "control_executions": executions,
            "changes": changes,
            "incidents": incidents.values().collect::<Vec<_>>(),
        });

        serde_json::to_string_pretty(&report).unwrap_or_default()
    }
}

impl Default for Soc2Manager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_control_execution() {
        let manager = Soc2Manager::new();

        let control_id = ControlId::new(TrustServiceCategory::Security, "6.1");
        let evidence = vec![Evidence {
            id: Uuid::new_v4(),
            evidence_type: EvidenceType::LogFile,
            description: "Access log".to_string(),
            reference: "/var/log/access.log".to_string(),
            collected_at: Utc::now(),
            collected_by: "system".to_string(),
            hash: None,
        }];

        let execution_id = manager
            .execute_control(control_id, "auditor", ControlResult::Pass, evidence)
            .await
            .unwrap();

        assert!(execution_id != Uuid::nil());
    }

    #[tokio::test]
    async fn test_incident_management() {
        let manager = Soc2Manager::new();

        let incident = SecurityIncident {
            id: Uuid::new_v4(),
            detected_at: Utc::now(),
            severity: IncidentSeverity::High,
            category: "Unauthorized Access".to_string(),
            description: "Failed login attempts detected".to_string(),
            affected_systems: vec!["auth-service".to_string()],
            detected_by: "IDS".to_string(),
            assigned_to: Some("security-team".to_string()),
            status: IncidentStatus::New,
            resolved_at: None,
            resolution: None,
            root_cause: None,
            remediation_actions: vec![],
        };

        let incident_id = manager.record_incident(incident).await.unwrap();
        manager
            .update_incident_status(incident_id, IncidentStatus::Resolved)
            .await
            .unwrap();

        let open_incidents = manager.get_open_incidents().await;
        assert_eq!(open_incidents.len(), 0);
    }

    #[tokio::test]
    async fn test_change_management() {
        let manager = Soc2Manager::new();

        let change = ChangeManagementLog {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            change_type: ChangeType::Configuration,
            component: "database".to_string(),
            description: "Updated connection pool size".to_string(),
            requested_by: "dev-team".to_string(),
            approved_by: Some("manager".to_string()),
            implemented_by: "ops-team".to_string(),
            approval_ticket: Some("CHG-123".to_string()),
            testing: Some("Tested in staging".to_string()),
            rollback_plan: Some("Revert config".to_string()),
            success: true,
        };

        manager.log_change(change).await.unwrap();

        let start = Utc::now() - chrono::Duration::hours(1);
        let end = Utc::now() + chrono::Duration::hours(1);
        let changes = manager.get_changes_for_period(start, end).await;

        assert_eq!(changes.len(), 1);
    }
}
