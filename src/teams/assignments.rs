//! Issue Assignment Module
//!
//! Provides advanced issue assignment and workload management including:
//! - Smart assignment workflows
//! - Automatic workload balancing
//! - Due date tracking and SLA management
//! - Escalation rules and alerts
//! - Auto-assignment based on skills and availability

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum AssignmentError {
    #[error("Assignment not found: {0}")]
    NotFound(String),

    #[error("Assignment already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid assignment: {0}")]
    Invalid(String),

    #[error("Assignee unavailable: {0}")]
    AssigneeUnavailable(String),

    #[error("Workload capacity exceeded")]
    CapacityExceeded,

    #[error("SLA violation: {0}")]
    SlaViolation(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

pub type AssignmentResult<T> = Result<T, AssignmentError>;

// ============================================================================
// Core Types
// ============================================================================

/// Assignment priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AssignmentPriority {
    Critical,
    High,
    Medium,
    Low,
}

impl AssignmentPriority {
    /// Get default SLA hours for this priority
    pub fn default_sla_hours(&self) -> i64 {
        match self {
            AssignmentPriority::Critical => 4,
            AssignmentPriority::High => 24,
            AssignmentPriority::Medium => 72,
            AssignmentPriority::Low => 168,
        }
    }

    /// Get weight for workload calculation
    pub fn workload_weight(&self) -> f32 {
        match self {
            AssignmentPriority::Critical => 4.0,
            AssignmentPriority::High => 2.0,
            AssignmentPriority::Medium => 1.0,
            AssignmentPriority::Low => 0.5,
        }
    }
}

/// Assignment status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssignmentStatus {
    /// Newly created, not yet accepted
    Pending,

    /// Assignee has accepted
    Accepted,

    /// Work in progress
    InProgress,

    /// Blocked on external dependency
    Blocked,

    /// Ready for review
    InReview,

    /// Completed
    Completed,

    /// Cancelled
    Cancelled,

    /// Escalated to higher priority
    Escalated,
}

/// Issue assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment {
    /// Assignment ID
    pub id: String,

    /// Issue/task ID
    pub issue_id: String,

    /// Workspace ID
    pub workspace_id: String,

    /// Assigned user ID
    pub assignee_id: String,

    /// Who created the assignment
    pub assigner_id: String,

    /// Assignment priority
    pub priority: AssignmentPriority,

    /// Current status
    pub status: AssignmentStatus,

    /// Due date
    pub due_date: Option<DateTime<Utc>>,

    /// SLA deadline
    pub sla_deadline: Option<DateTime<Utc>>,

    /// When assignment was created
    pub created_at: DateTime<Utc>,

    /// When assignment was accepted
    pub accepted_at: Option<DateTime<Utc>>,

    /// When work started
    pub started_at: Option<DateTime<Utc>>,

    /// When assignment was completed
    pub completed_at: Option<DateTime<Utc>>,

    /// Estimated hours to complete
    pub estimated_hours: Option<f32>,

    /// Actual hours spent
    pub actual_hours: Option<f32>,

    /// Assignment tags/labels
    pub tags: Vec<String>,

    /// Required skills
    pub required_skills: Vec<String>,

    /// Assignment metadata
    pub metadata: HashMap<String, String>,

    /// Escalation history
    pub escalations: Vec<EscalationRecord>,
}

/// Escalation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRecord {
    pub escalated_at: DateTime<Utc>,
    pub escalated_by: String,
    pub reason: String,
    pub previous_priority: AssignmentPriority,
    pub new_priority: AssignmentPriority,
}

/// Workload balance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadBalance {
    /// User ID
    pub user_id: String,

    /// Total assigned issues
    pub total_assignments: usize,

    /// Active assignments
    pub active_assignments: usize,

    /// Weighted workload (considering priorities)
    pub workload_score: f32,

    /// Available capacity (0.0 - 1.0)
    pub available_capacity: f32,

    /// Average completion time (hours)
    pub avg_completion_time: Option<f32>,

    /// On-time completion rate (0.0 - 1.0)
    pub on_time_rate: Option<f32>,
}

/// Auto-assignment rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentRule {
    /// Rule ID
    pub id: String,

    /// Rule name
    pub name: String,

    /// Workspace ID
    pub workspace_id: String,

    /// Is rule enabled?
    pub enabled: bool,

    /// Priority levels this rule applies to
    pub priorities: Vec<AssignmentPriority>,

    /// Required skills match
    pub skill_match: Vec<String>,

    /// Tags that trigger this rule
    pub trigger_tags: Vec<String>,

    /// Assignment strategy
    pub strategy: AssignmentStrategy,

    /// Maximum workload score for auto-assignment
    pub max_workload_score: f32,

    /// Created by
    pub created_by: String,

    /// Created at
    pub created_at: DateTime<Utc>,
}

/// Assignment strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssignmentStrategy {
    /// Round-robin assignment
    RoundRobin,

    /// Assign to least loaded team member
    LeastLoaded,

    /// Assign based on skills match
    SkillBased,

    /// Assign to best performer
    BestPerformer,

    /// Random assignment
    Random,
}

/// Escalation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRule {
    /// Rule ID
    pub id: String,

    /// Workspace ID
    pub workspace_id: String,

    /// Trigger condition
    pub trigger: EscalationTrigger,

    /// Action to take
    pub action: EscalationAction,

    /// Is rule enabled?
    pub enabled: bool,
}

/// Escalation trigger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EscalationTrigger {
    /// Time before SLA deadline (hours)
    TimeBeforeSla(i64),

    /// Assignment in status for duration (hours)
    StatusDuration(AssignmentStatus, i64),

    /// Blocked for duration (hours)
    BlockedDuration(i64),

    /// No progress for duration (hours)
    NoProgressDuration(i64),
}

/// Escalation action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EscalationAction {
    /// Increase priority
    IncreasePriority,

    /// Notify manager
    NotifyManager,

    /// Reassign to different user
    Reassign(String),

    /// Send alert
    SendAlert(String),
}

/// SLA configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaConfig {
    /// Workspace ID
    pub workspace_id: String,

    /// SLA hours by priority
    pub sla_hours: HashMap<AssignmentPriority, i64>,

    /// Business hours only?
    pub business_hours_only: bool,

    /// Timezone for SLA calculation
    pub timezone: String,

    /// Warning threshold (% of SLA time remaining)
    pub warning_threshold: f32,
}

impl Default for SlaConfig {
    fn default() -> Self {
        let mut sla_hours = HashMap::new();
        sla_hours.insert(AssignmentPriority::Critical, 4);
        sla_hours.insert(AssignmentPriority::High, 24);
        sla_hours.insert(AssignmentPriority::Medium, 72);
        sla_hours.insert(AssignmentPriority::Low, 168);

        Self {
            workspace_id: String::new(),
            sla_hours,
            business_hours_only: true,
            timezone: "UTC".to_string(),
            warning_threshold: 0.25, // Warn at 25% remaining
        }
    }
}

/// Auto-assignment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoAssignmentConfig {
    /// Is auto-assignment enabled?
    pub enabled: bool,

    /// Default assignment strategy
    pub default_strategy: AssignmentStrategy,

    /// Maximum assignments per user
    pub max_assignments_per_user: Option<usize>,

    /// Maximum workload score per user
    pub max_workload_score: f32,

    /// Consider user skills?
    pub consider_skills: bool,

    /// Consider user availability?
    pub consider_availability: bool,
}

impl Default for AutoAssignmentConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            default_strategy: AssignmentStrategy::LeastLoaded,
            max_assignments_per_user: Some(10),
            max_workload_score: 20.0,
            consider_skills: true,
            consider_availability: true,
        }
    }
}

// ============================================================================
// Assignment Manager
// ============================================================================

/// Manages issue assignments and workload
#[derive(Debug)]
pub struct AssignmentManager {
    assignments: HashMap<String, Assignment>,
    rules: HashMap<String, AssignmentRule>,
    escalation_rules: HashMap<String, EscalationRule>,
    sla_configs: HashMap<String, SlaConfig>,
    auto_assign_configs: HashMap<String, AutoAssignmentConfig>,
    issue_index: HashMap<String, String>, // issue_id -> assignment_id
    assignee_index: HashMap<String, Vec<String>>, // user_id -> assignment_ids
    workspace_index: HashMap<String, Vec<String>>, // workspace_id -> assignment_ids
}

impl AssignmentManager {
    /// Create a new assignment manager
    pub fn new() -> Self {
        Self {
            assignments: HashMap::new(),
            rules: HashMap::new(),
            escalation_rules: HashMap::new(),
            sla_configs: HashMap::new(),
            auto_assign_configs: HashMap::new(),
            issue_index: HashMap::new(),
            assignee_index: HashMap::new(),
            workspace_index: HashMap::new(),
        }
    }

    /// Assign an issue to a user
    pub fn assign_issue(
        &mut self,
        issue_id: String,
        assignee_id: String,
        priority: AssignmentPriority,
        due_date: Option<DateTime<Utc>>,
        assigner_id: String,
    ) -> AssignmentResult<Assignment> {
        // Check if issue already assigned
        if self.issue_index.contains_key(&issue_id) {
            return Err(AssignmentError::AlreadyExists(issue_id));
        }

        let sla_deadline = Some(Utc::now() + Duration::hours(priority.default_sla_hours()));

        let assignment = Assignment {
            id: Uuid::new_v4().to_string(),
            issue_id: issue_id.clone(),
            workspace_id: "default".to_string(),
            assignee_id: assignee_id.clone(),
            assigner_id,
            priority,
            status: AssignmentStatus::Pending,
            due_date,
            sla_deadline,
            created_at: Utc::now(),
            accepted_at: None,
            started_at: None,
            completed_at: None,
            estimated_hours: None,
            actual_hours: None,
            tags: Vec::new(),
            required_skills: Vec::new(),
            metadata: HashMap::new(),
            escalations: Vec::new(),
        };

        // Update indexes
        self.issue_index.insert(issue_id, assignment.id.clone());
        self.assignee_index
            .entry(assignee_id)
            .or_insert_with(Vec::new)
            .push(assignment.id.clone());
        self.workspace_index
            .entry(assignment.workspace_id.clone())
            .or_insert_with(Vec::new)
            .push(assignment.id.clone());

        self.assignments.insert(assignment.id.clone(), assignment.clone());

        Ok(assignment)
    }

    /// Update assignment status
    pub fn update_status(
        &mut self,
        assignment_id: &str,
        new_status: AssignmentStatus,
    ) -> AssignmentResult<()> {
        let assignment = self
            .assignments
            .get_mut(assignment_id)
            .ok_or_else(|| AssignmentError::NotFound(assignment_id.to_string()))?;

        let now = Utc::now();

        match new_status {
            AssignmentStatus::Accepted => {
                assignment.accepted_at = Some(now);
            }
            AssignmentStatus::InProgress => {
                if assignment.started_at.is_none() {
                    assignment.started_at = Some(now);
                }
            }
            AssignmentStatus::Completed => {
                assignment.completed_at = Some(now);
            }
            _ => {}
        }

        assignment.status = new_status;

        Ok(())
    }

    /// Escalate assignment
    pub fn escalate_assignment(
        &mut self,
        assignment_id: &str,
        new_priority: AssignmentPriority,
        escalated_by: &str,
        reason: String,
    ) -> AssignmentResult<()> {
        let assignment = self
            .assignments
            .get_mut(assignment_id)
            .ok_or_else(|| AssignmentError::NotFound(assignment_id.to_string()))?;

        let record = EscalationRecord {
            escalated_at: Utc::now(),
            escalated_by: escalated_by.to_string(),
            reason,
            previous_priority: assignment.priority,
            new_priority,
        };

        assignment.escalations.push(record);
        assignment.priority = new_priority;
        assignment.status = AssignmentStatus::Escalated;

        // Update SLA deadline
        assignment.sla_deadline = Some(Utc::now() + Duration::hours(new_priority.default_sla_hours()));

        Ok(())
    }

    /// Reassign to different user
    pub fn reassign(
        &mut self,
        assignment_id: &str,
        new_assignee_id: String,
    ) -> AssignmentResult<()> {
        let assignment = self
            .assignments
            .get_mut(assignment_id)
            .ok_or_else(|| AssignmentError::NotFound(assignment_id.to_string()))?;

        let old_assignee_id = assignment.assignee_id.clone();

        // Update assignee index
        if let Some(assignments) = self.assignee_index.get_mut(&old_assignee_id) {
            assignments.retain(|id| id != assignment_id);
        }

        self.assignee_index
            .entry(new_assignee_id.clone())
            .or_insert_with(Vec::new)
            .push(assignment_id.to_string());

        assignment.assignee_id = new_assignee_id;
        assignment.status = AssignmentStatus::Pending;
        assignment.accepted_at = None;

        Ok(())
    }

    /// Calculate workload for a user
    pub fn calculate_workload(&self, user_id: &str) -> WorkloadBalance {
        let assignment_ids = self.assignee_index.get(user_id).cloned().unwrap_or_default();

        let assignments: Vec<&Assignment> = assignment_ids
            .iter()
            .filter_map(|id| self.assignments.get(id))
            .collect();

        let total_assignments = assignments.len();
        let active_assignments = assignments
            .iter()
            .filter(|a| {
                matches!(
                    a.status,
                    AssignmentStatus::Pending
                        | AssignmentStatus::Accepted
                        | AssignmentStatus::InProgress
                        | AssignmentStatus::Blocked
                )
            })
            .count();

        let workload_score: f32 = assignments
            .iter()
            .filter(|a| {
                matches!(
                    a.status,
                    AssignmentStatus::Pending
                        | AssignmentStatus::Accepted
                        | AssignmentStatus::InProgress
                )
            })
            .map(|a| a.priority.workload_weight())
            .sum();

        let completed: Vec<&Assignment> = assignments
            .iter()
            .filter(|a| a.status == AssignmentStatus::Completed)
            .copied()
            .collect();

        let avg_completion_time = if !completed.is_empty() {
            let total_hours: f32 = completed
                .iter()
                .filter_map(|a| a.actual_hours)
                .sum();
            Some(total_hours / completed.len() as f32)
        } else {
            None
        };

        let on_time_count = completed
            .iter()
            .filter(|a| {
                if let (Some(completed), Some(sla)) = (a.completed_at, a.sla_deadline) {
                    completed <= sla
                } else {
                    false
                }
            })
            .count();

        let on_time_rate = if !completed.is_empty() {
            Some(on_time_count as f32 / completed.len() as f32)
        } else {
            None
        };

        WorkloadBalance {
            user_id: user_id.to_string(),
            total_assignments,
            active_assignments,
            workload_score,
            available_capacity: (20.0 - workload_score).max(0.0) / 20.0,
            avg_completion_time,
            on_time_rate,
        }
    }

    /// Auto-assign issue using configured strategy
    pub fn auto_assign(
        &mut self,
        issue_id: String,
        workspace_id: &str,
        candidate_users: Vec<String>,
        priority: AssignmentPriority,
        required_skills: Vec<String>,
    ) -> AssignmentResult<Assignment> {
        let config = self
            .auto_assign_configs
            .get(workspace_id)
            .cloned()
            .unwrap_or_default();

        if !config.enabled {
            return Err(AssignmentError::Invalid("Auto-assignment disabled".to_string()));
        }

        let assignee_id = match config.default_strategy {
            AssignmentStrategy::LeastLoaded => self.find_least_loaded(&candidate_users),
            AssignmentStrategy::SkillBased => {
                self.find_best_skill_match(&candidate_users, &required_skills)
            }
            AssignmentStrategy::BestPerformer => self.find_best_performer(&candidate_users),
            AssignmentStrategy::RoundRobin => self.find_round_robin(&candidate_users, workspace_id),
            AssignmentStrategy::Random => {
                candidate_users.first().cloned().unwrap_or_default()
            }
        };

        self.assign_issue(
            issue_id,
            assignee_id,
            priority,
            None,
            "auto-assign".to_string(),
        )
    }

    /// Find least loaded user
    fn find_least_loaded(&self, users: &[String]) -> String {
        users
            .iter()
            .min_by_key(|user_id| {
                let workload = self.calculate_workload(user_id);
                (workload.workload_score * 100.0) as i32
            })
            .cloned()
            .unwrap_or_default()
    }

    /// Find best skill match
    fn find_best_skill_match(&self, users: &[String], _skills: &[String]) -> String {
        // Simplified: just return first user
        // In production, would match against user skill profiles
        users.first().cloned().unwrap_or_default()
    }

    /// Find best performer
    fn find_best_performer(&self, users: &[String]) -> String {
        users
            .iter()
            .max_by(|a, b| {
                let workload_a = self.calculate_workload(a);
                let workload_b = self.calculate_workload(b);

                let score_a = workload_a.on_time_rate.unwrap_or(0.0);
                let score_b = workload_b.on_time_rate.unwrap_or(0.0);

                score_a.partial_cmp(&score_b).unwrap()
            })
            .cloned()
            .unwrap_or_default()
    }

    /// Round-robin assignment
    fn find_round_robin(&self, users: &[String], _workspace_id: &str) -> String {
        // Simplified: cycle through users
        // In production, would track last assigned user per workspace
        users.first().cloned().unwrap_or_default()
    }

    /// Get active assignments for workspace
    pub fn get_active_assignments(&self, workspace_id: &str) -> AssignmentResult<Vec<&Assignment>> {
        Ok(self
            .assignments
            .values()
            .filter(|a| {
                a.workspace_id == workspace_id
                    && matches!(
                        a.status,
                        AssignmentStatus::Pending
                            | AssignmentStatus::Accepted
                            | AssignmentStatus::InProgress
                            | AssignmentStatus::Blocked
                    )
            })
            .collect())
    }

    /// Get user's assignments
    pub fn get_user_assignments(&self, user_id: &str) -> Vec<&Assignment> {
        self.assignee_index
            .get(user_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.assignments.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Configure SLA for workspace
    pub fn configure_sla(&mut self, workspace_id: String, config: SlaConfig) {
        self.sla_configs.insert(workspace_id, config);
    }

    /// Configure auto-assignment for workspace
    pub fn configure_auto_assignment(&mut self, workspace_id: String, config: AutoAssignmentConfig) {
        self.auto_assign_configs.insert(workspace_id, config);
    }
}

impl Default for AssignmentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assign_issue() {
        let mut manager = AssignmentManager::new();

        let assignment = manager
            .assign_issue(
                "issue1".to_string(),
                "user1".to_string(),
                AssignmentPriority::High,
                None,
                "admin".to_string(),
            )
            .unwrap();

        assert_eq!(assignment.priority, AssignmentPriority::High);
        assert_eq!(assignment.status, AssignmentStatus::Pending);
    }

    #[test]
    fn test_workload_calculation() {
        let mut manager = AssignmentManager::new();

        manager
            .assign_issue(
                "issue1".to_string(),
                "user1".to_string(),
                AssignmentPriority::High,
                None,
                "admin".to_string(),
            )
            .unwrap();

        manager
            .assign_issue(
                "issue2".to_string(),
                "user1".to_string(),
                AssignmentPriority::Medium,
                None,
                "admin".to_string(),
            )
            .unwrap();

        let workload = manager.calculate_workload("user1");

        assert_eq!(workload.total_assignments, 2);
        assert_eq!(workload.active_assignments, 2);
        assert_eq!(workload.workload_score, 3.0); // High(2.0) + Medium(1.0)
    }

    #[test]
    fn test_escalation() {
        let mut manager = AssignmentManager::new();

        let assignment = manager
            .assign_issue(
                "issue1".to_string(),
                "user1".to_string(),
                AssignmentPriority::Medium,
                None,
                "admin".to_string(),
            )
            .unwrap();

        manager
            .escalate_assignment(
                &assignment.id,
                AssignmentPriority::Critical,
                "manager",
                "Customer escalation".to_string(),
            )
            .unwrap();

        let updated = manager.assignments.get(&assignment.id).unwrap();
        assert_eq!(updated.priority, AssignmentPriority::Critical);
        assert_eq!(updated.escalations.len(), 1);
    }
}
