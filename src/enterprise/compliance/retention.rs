//! Data retention policies and lifecycle management
//!
//! This module implements configurable retention periods, legal holds,
//! archival strategies, and automated purge scheduling for compliance.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Data retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Unique policy ID
    pub id: Uuid,

    /// Policy name
    pub name: String,

    /// Data category this policy applies to
    pub data_category: String,

    /// Retention period
    pub retention_period: RetentionPeriod,

    /// Actions to take after retention period
    pub post_retention_action: PostRetentionAction,

    /// Whether policy is active
    pub active: bool,

    /// Legal/regulatory basis for retention
    pub legal_basis: Vec<String>,

    /// Priority (higher priority policies take precedence)
    pub priority: u32,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Last modified
    pub updated_at: DateTime<Utc>,

    /// Created by
    pub created_by: String,
}

/// Retention period specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetentionPeriod {
    /// Retain for specific duration
    Duration(Duration),
    /// Retain indefinitely
    Indefinite,
    /// Retain until specific date
    Until(DateTime<Utc>),
    /// Custom rule
    Custom(String),
}

impl RetentionPeriod {
    /// Calculate expiration date from creation date
    pub fn calculate_expiration(&self, created: DateTime<Utc>) -> Option<DateTime<Utc>> {
        match self {
            RetentionPeriod::Duration(duration) => Some(created + *duration),
            RetentionPeriod::Until(date) => Some(*date),
            RetentionPeriod::Indefinite => None,
            RetentionPeriod::Custom(_) => None,
        }
    }

    /// Check if data has expired
    pub fn is_expired(&self, created: DateTime<Utc>, now: DateTime<Utc>) -> bool {
        if let Some(expiration) = self.calculate_expiration(created) {
            now >= expiration
        } else {
            false
        }
    }
}

/// Action to take after retention period expires
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PostRetentionAction {
    /// Delete/purge data
    Delete,
    /// Archive to cold storage
    Archive,
    /// Mark for review
    Review,
    /// No action (manual intervention required)
    NoAction,
}

/// Legal hold on data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalHold {
    /// Unique hold ID
    pub id: Uuid,

    /// Hold name/case reference
    pub name: String,

    /// Description/reason for hold
    pub description: String,

    /// Data categories affected
    pub data_categories: Vec<String>,

    /// Specific data item IDs (if applicable)
    pub data_items: HashSet<String>,

    /// Hold start date
    pub started_at: DateTime<Utc>,

    /// Hold end date (when released)
    pub ended_at: Option<DateTime<Utc>>,

    /// Custodian/legal contact
    pub custodian: String,

    /// Case reference number
    pub case_reference: String,

    /// Whether hold is active
    pub active: bool,
}

impl LegalHold {
    /// Check if data item is under this hold
    pub fn applies_to(&self, data_id: &str, data_category: &str) -> bool {
        self.active
            && (self.data_items.contains(data_id) || self.data_categories.contains(&data_category.to_string()))
    }

    /// Release the legal hold
    pub fn release(&mut self) {
        self.active = false;
        self.ended_at = Some(Utc::now());
    }
}

/// Data lifecycle stage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LifecycleStage {
    /// Active/current data
    Active,
    /// Inactive but retained
    Inactive,
    /// Archived to cold storage
    Archived,
    /// Pending deletion
    PendingDeletion,
    /// Deleted
    Deleted,
    /// Under legal hold
    LegalHold,
}

/// Data item with retention metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionRecord {
    /// Unique record ID
    pub id: String,

    /// Data category
    pub category: String,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last access timestamp
    pub last_accessed: Option<DateTime<Utc>>,

    /// Current lifecycle stage
    pub lifecycle_stage: LifecycleStage,

    /// Applicable retention policy
    pub policy_id: Option<Uuid>,

    /// Calculated expiration date
    pub expires_at: Option<DateTime<Utc>>,

    /// Legal holds affecting this data
    pub legal_holds: Vec<Uuid>,

    /// Archive location (if archived)
    pub archive_location: Option<String>,

    /// Deletion scheduled time
    pub deletion_scheduled_at: Option<DateTime<Utc>>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl RetentionRecord {
    /// Check if record can be deleted
    pub fn can_delete(&self) -> bool {
        self.legal_holds.is_empty()
            && matches!(
                self.lifecycle_stage,
                LifecycleStage::PendingDeletion | LifecycleStage::Inactive
            )
    }

    /// Check if record should be archived
    pub fn should_archive(&self, now: DateTime<Utc>) -> bool {
        if let Some(expires_at) = self.expires_at {
            now >= expires_at && self.legal_holds.is_empty() && self.lifecycle_stage == LifecycleStage::Inactive
        } else {
            false
        }
    }
}

/// Archival strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivalStrategy {
    /// Strategy name
    pub name: String,

    /// Archive format (zip, tar, etc.)
    pub format: String,

    /// Compression algorithm
    pub compression: Option<String>,

    /// Encryption enabled
    pub encrypt: bool,

    /// Archive storage location
    pub storage_location: String,

    /// Retention period in archive
    pub archive_retention: RetentionPeriod,
}

/// Purge schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurgeSchedule {
    /// Schedule ID
    pub id: Uuid,

    /// Schedule name
    pub name: String,

    /// Cron expression for schedule
    pub cron_expression: String,

    /// Data categories to purge
    pub categories: Vec<String>,

    /// Batch size (number of records per run)
    pub batch_size: u32,

    /// Whether schedule is enabled
    pub enabled: bool,

    /// Last run timestamp
    pub last_run: Option<DateTime<Utc>>,

    /// Next scheduled run
    pub next_run: Option<DateTime<Utc>>,
}

/// Retention manager
pub struct RetentionManager {
    /// Retention policies
    policies: Arc<RwLock<HashMap<Uuid, RetentionPolicy>>>,

    /// Legal holds
    legal_holds: Arc<RwLock<HashMap<Uuid, LegalHold>>>,

    /// Retention records
    records: Arc<RwLock<HashMap<String, RetentionRecord>>>,

    /// Purge schedules
    schedules: Arc<RwLock<HashMap<Uuid, PurgeSchedule>>>,

    /// Archival strategies
    archival_strategies: Arc<RwLock<HashMap<String, ArchivalStrategy>>>,
}

impl RetentionManager {
    /// Create new retention manager
    pub fn new() -> Self {
        Self {
            policies: Arc::new(RwLock::new(HashMap::new())),
            legal_holds: Arc::new(RwLock::new(HashMap::new())),
            records: Arc::new(RwLock::new(HashMap::new())),
            schedules: Arc::new(RwLock::new(HashMap::new())),
            archival_strategies: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // ========================================================================
    // Policy Management
    // ========================================================================

    /// Add a retention policy
    pub async fn add_policy(&self, policy: RetentionPolicy) -> Result<Uuid, String> {
        let policy_id = policy.id;
        let mut policies = self.policies.write().await;
        policies.insert(policy_id, policy);
        Ok(policy_id)
    }

    /// Get policy by ID
    pub async fn get_policy(&self, id: Uuid) -> Option<RetentionPolicy> {
        let policies = self.policies.read().await;
        policies.get(&id).cloned()
    }

    /// Get applicable policy for data category
    pub async fn get_applicable_policy(&self, category: &str) -> Option<RetentionPolicy> {
        let policies = self.policies.read().await;

        // Find highest priority active policy for category
        policies
            .values()
            .filter(|p| p.active && p.data_category == category)
            .max_by_key(|p| p.priority)
            .cloned()
    }

    /// Update policy
    pub async fn update_policy(&self, id: Uuid, mut policy: RetentionPolicy) -> Result<(), String> {
        policy.updated_at = Utc::now();
        let mut policies = self.policies.write().await;
        policies.insert(id, policy);
        Ok(())
    }

    // ========================================================================
    // Legal Holds
    // ========================================================================

    /// Place legal hold
    pub async fn place_legal_hold(&self, hold: LegalHold) -> Result<Uuid, String> {
        let hold_id = hold.id;

        // Update affected records
        {
            let mut records = self.records.write().await;
            for record in records.values_mut() {
                if hold.applies_to(&record.id, &record.category) {
                    record.legal_holds.push(hold_id);
                    record.lifecycle_stage = LifecycleStage::LegalHold;
                }
            }
        }

        let mut holds = self.legal_holds.write().await;
        holds.insert(hold_id, hold);
        Ok(hold_id)
    }

    /// Release legal hold
    pub async fn release_legal_hold(&self, hold_id: Uuid) -> Result<(), String> {
        let mut holds = self.legal_holds.write().await;

        if let Some(hold) = holds.get_mut(&hold_id) {
            hold.release();

            // Update affected records
            drop(holds); // Release lock before acquiring records lock
            let mut records = self.records.write().await;
            for record in records.values_mut() {
                record.legal_holds.retain(|h| *h != hold_id);
                if record.legal_holds.is_empty() && record.lifecycle_stage == LifecycleStage::LegalHold {
                    record.lifecycle_stage = LifecycleStage::Inactive;
                }
            }

            Ok(())
        } else {
            Err("Legal hold not found".to_string())
        }
    }

    /// Check if data is under legal hold
    pub async fn is_under_legal_hold(&self, data_id: &str, category: &str) -> bool {
        let holds = self.legal_holds.read().await;
        holds.values().any(|h| h.applies_to(data_id, category))
    }

    // ========================================================================
    // Record Management
    // ========================================================================

    /// Register data for retention tracking
    pub async fn register_data(
        &self,
        data_id: impl Into<String>,
        category: impl Into<String>,
    ) -> Result<(), String> {
        let data_id = data_id.into();
        let category = category.into();

        // Find applicable policy
        let policy = self.get_applicable_policy(&category).await;

        let expires_at = policy.as_ref().and_then(|p| {
            p.retention_period.calculate_expiration(Utc::now())
        });

        let record = RetentionRecord {
            id: data_id.clone(),
            category,
            created_at: Utc::now(),
            last_accessed: None,
            lifecycle_stage: LifecycleStage::Active,
            policy_id: policy.map(|p| p.id),
            expires_at,
            legal_holds: Vec::new(),
            archive_location: None,
            deletion_scheduled_at: None,
            metadata: HashMap::new(),
        };

        let mut records = self.records.write().await;
        records.insert(data_id, record);

        Ok(())
    }

    /// Update last access time
    pub async fn update_access(&self, data_id: &str) -> Result<(), String> {
        let mut records = self.records.write().await;
        if let Some(record) = records.get_mut(data_id) {
            record.last_accessed = Some(Utc::now());
            Ok(())
        } else {
            Err("Record not found".to_string())
        }
    }

    /// Get records eligible for deletion
    pub async fn get_records_for_deletion(&self) -> Vec<RetentionRecord> {
        let records = self.records.read().await;
        let now = Utc::now();

        records
            .values()
            .filter(|r| {
                r.can_delete()
                    && r.expires_at.map_or(false, |exp| now >= exp)
            })
            .cloned()
            .collect()
    }

    /// Get records eligible for archival
    pub async fn get_records_for_archival(&self) -> Vec<RetentionRecord> {
        let records = self.records.read().await;
        let now = Utc::now();

        records
            .values()
            .filter(|r| r.should_archive(now))
            .cloned()
            .collect()
    }

    /// Mark record as archived
    pub async fn mark_archived(&self, data_id: &str, location: impl Into<String>) -> Result<(), String> {
        let mut records = self.records.write().await;
        if let Some(record) = records.get_mut(data_id) {
            record.lifecycle_stage = LifecycleStage::Archived;
            record.archive_location = Some(location.into());
            Ok(())
        } else {
            Err("Record not found".to_string())
        }
    }

    /// Schedule deletion
    pub async fn schedule_deletion(&self, data_id: &str, when: DateTime<Utc>) -> Result<(), String> {
        let mut records = self.records.write().await;
        if let Some(record) = records.get_mut(data_id) {
            if !record.can_delete() {
                return Err("Record cannot be deleted (legal hold)".to_string());
            }
            record.lifecycle_stage = LifecycleStage::PendingDeletion;
            record.deletion_scheduled_at = Some(when);
            Ok(())
        } else {
            Err("Record not found".to_string())
        }
    }

    /// Execute deletion
    pub async fn execute_deletion(&self, data_id: &str) -> Result<(), String> {
        let mut records = self.records.write().await;
        if let Some(record) = records.get_mut(data_id) {
            if !record.can_delete() {
                return Err("Record cannot be deleted (legal hold)".to_string());
            }
            record.lifecycle_stage = LifecycleStage::Deleted;
            // In production, actually delete the data here
            Ok(())
        } else {
            Err("Record not found".to_string())
        }
    }

    // ========================================================================
    // Archival
    // ========================================================================

    /// Add archival strategy
    pub async fn add_archival_strategy(&self, strategy: ArchivalStrategy) -> Result<(), String> {
        let name = strategy.name.clone();
        let mut strategies = self.archival_strategies.write().await;
        strategies.insert(name, strategy);
        Ok(())
    }

    /// Get archival strategy
    pub async fn get_archival_strategy(&self, name: &str) -> Option<ArchivalStrategy> {
        let strategies = self.archival_strategies.read().await;
        strategies.get(name).cloned()
    }

    // ========================================================================
    // Purge Scheduling
    // ========================================================================

    /// Add purge schedule
    pub async fn add_purge_schedule(&self, schedule: PurgeSchedule) -> Result<Uuid, String> {
        let schedule_id = schedule.id;
        let mut schedules = self.schedules.write().await;
        schedules.insert(schedule_id, schedule);
        Ok(schedule_id)
    }

    /// Execute purge schedule
    pub async fn execute_purge_schedule(&self, schedule_id: Uuid) -> Result<u32, String> {
        let mut schedules = self.schedules.write().await;

        if let Some(schedule) = schedules.get_mut(&schedule_id) {
            if !schedule.enabled {
                return Err("Schedule is disabled".to_string());
            }

            schedule.last_run = Some(Utc::now());
            let batch_size = schedule.batch_size;

            // Get eligible records
            drop(schedules); // Release lock
            let to_delete = self.get_records_for_deletion().await;

            let mut deleted_count = 0;
            for record in to_delete.iter().take(batch_size as usize) {
                if self.execute_deletion(&record.id).await.is_ok() {
                    deleted_count += 1;
                }
            }

            Ok(deleted_count)
        } else {
            Err("Schedule not found".to_string())
        }
    }

    // ========================================================================
    // Reporting
    // ========================================================================

    /// Generate retention report
    pub async fn generate_retention_report(&self) -> String {
        let policies = self.policies.read().await;
        let holds = self.legal_holds.read().await;
        let records = self.records.read().await;

        let active_policies = policies.values().filter(|p| p.active).count();
        let active_holds = holds.values().filter(|h| h.active).count();

        let stage_counts: HashMap<_, _> = records
            .values()
            .fold(HashMap::new(), |mut acc, r| {
                *acc.entry(r.lifecycle_stage).or_insert(0) += 1;
                acc
            });

        let report = serde_json::json!({
            "title": "Data Retention Report",
            "generated_at": Utc::now(),
            "summary": {
                "total_records": records.len(),
                "active_policies": active_policies,
                "active_legal_holds": active_holds,
                "lifecycle_stages": stage_counts,
            },
            "policies": policies.values().collect::<Vec<_>>(),
            "legal_holds": holds.values().filter(|h| h.active).collect::<Vec<_>>(),
        });

        serde_json::to_string_pretty(&report).unwrap_or_default()
    }
}

impl Default for RetentionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retention_policy() {
        let manager = RetentionManager::new();

        let policy = RetentionPolicy {
            id: Uuid::new_v4(),
            name: "CAD Files Retention".to_string(),
            data_category: "cad_files".to_string(),
            retention_period: RetentionPeriod::Duration(Duration::days(365 * 7)),
            post_retention_action: PostRetentionAction::Archive,
            active: true,
            legal_basis: vec!["Business requirement".to_string()],
            priority: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: "admin".to_string(),
        };

        manager.add_policy(policy).await.unwrap();

        let found = manager.get_applicable_policy("cad_files").await;
        assert!(found.is_some());
    }

    #[tokio::test]
    async fn test_legal_hold() {
        let manager = RetentionManager::new();

        manager.register_data("doc1", "legal_docs").await.unwrap();

        let hold = LegalHold {
            id: Uuid::new_v4(),
            name: "Case 2024-001".to_string(),
            description: "Litigation hold".to_string(),
            data_categories: vec!["legal_docs".to_string()],
            data_items: HashSet::new(),
            started_at: Utc::now(),
            ended_at: None,
            custodian: "legal@company.com".to_string(),
            case_reference: "CASE-001".to_string(),
            active: true,
        };

        let hold_id = manager.place_legal_hold(hold).await.unwrap();

        assert!(manager.is_under_legal_hold("doc1", "legal_docs").await);

        manager.release_legal_hold(hold_id).await.unwrap();

        assert!(!manager.is_under_legal_hold("doc1", "legal_docs").await);
    }

    #[tokio::test]
    async fn test_data_lifecycle() {
        let manager = RetentionManager::new();

        // Add policy with short retention
        let policy = RetentionPolicy {
            id: Uuid::new_v4(),
            name: "Test Policy".to_string(),
            data_category: "test_data".to_string(),
            retention_period: RetentionPeriod::Duration(Duration::seconds(1)),
            post_retention_action: PostRetentionAction::Delete,
            active: true,
            legal_basis: vec![],
            priority: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: "admin".to_string(),
        };

        manager.add_policy(policy).await.unwrap();
        manager.register_data("item1", "test_data").await.unwrap();

        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let for_deletion = manager.get_records_for_deletion().await;
        assert!(for_deletion.is_empty()); // Still active

        // Mark as inactive first
        {
            let mut records = manager.records.write().await;
            if let Some(record) = records.get_mut("item1") {
                record.lifecycle_stage = LifecycleStage::Inactive;
            }
        }

        let for_deletion = manager.get_records_for_deletion().await;
        assert!(!for_deletion.is_empty());
    }

    #[tokio::test]
    async fn test_archival() {
        let manager = RetentionManager::new();

        let strategy = ArchivalStrategy {
            name: "standard".to_string(),
            format: "tar.gz".to_string(),
            compression: Some("gzip".to_string()),
            encrypt: true,
            storage_location: "s3://archive-bucket".to_string(),
            archive_retention: RetentionPeriod::Duration(Duration::days(365 * 10)),
        };

        manager.add_archival_strategy(strategy).await.unwrap();

        manager.register_data("doc1", "documents").await.unwrap();
        manager.mark_archived("doc1", "s3://archive-bucket/doc1.tar.gz").await.unwrap();

        let records = manager.records.read().await;
        let record = records.get("doc1").unwrap();
        assert_eq!(record.lifecycle_stage, LifecycleStage::Archived);
    }
}
