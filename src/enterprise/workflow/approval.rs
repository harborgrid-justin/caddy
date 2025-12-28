use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ApprovalError {
    #[error("Approval request not found: {0}")]
    NotFound(Uuid),
    #[error("Invalid approval action: {0}")]
    InvalidAction(String),
    #[error("User not authorized to approve: {0}")]
    Unauthorized(Uuid),
    #[error("Approval already completed")]
    AlreadyCompleted,
    #[error("Invalid approval level: {0}")]
    InvalidLevel(u32),
    #[error("Delegation failed: {0}")]
    DelegationFailed(String),
    #[error("Escalation failed: {0}")]
    EscalationFailed(String),
}

pub type ApprovalResult<T> = Result<T, ApprovalError>;

/// Status of an approval request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalStatus {
    /// Pending approval
    Pending,
    /// Approved
    Approved,
    /// Rejected
    Rejected,
    /// Cancelled
    Cancelled,
    /// Escalated to higher authority
    Escalated,
    /// Delegated to another user
    Delegated,
}

/// Approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    /// Unique identifier
    pub id: Uuid,
    /// Workflow ID this approval is for
    pub workflow_id: Uuid,
    /// Step ID in the workflow
    pub step_id: Uuid,
    /// Title/subject of approval
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Entity being approved (e.g., drawing, document)
    pub entity_id: Option<Uuid>,
    /// Entity type
    pub entity_type: Option<String>,
    /// Current status
    pub status: ApprovalStatus,
    /// Approval levels (for multi-level approvals)
    pub levels: Vec<ApprovalLevel>,
    /// Current level being processed
    pub current_level: u32,
    /// User who requested approval
    pub requester_id: Uuid,
    /// Due date for approval
    pub due_date: Option<DateTime<Utc>>,
    /// Priority (1-5, 5 being highest)
    pub priority: u32,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Completion timestamp
    pub completed_at: Option<DateTime<Utc>>,
    /// History of all actions
    pub history: Vec<ApprovalAction>,
}

impl ApprovalRequest {
    /// Create a new approval request
    pub fn new(
        workflow_id: Uuid,
        step_id: Uuid,
        title: String,
        description: String,
        requester_id: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            workflow_id,
            step_id,
            title,
            description,
            entity_id: None,
            entity_type: None,
            status: ApprovalStatus::Pending,
            levels: Vec::new(),
            current_level: 0,
            requester_id,
            due_date: None,
            priority: 3,
            metadata: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
            history: Vec::new(),
        }
    }

    /// Add an approval level
    pub fn add_level(&mut self, level: ApprovalLevel) {
        self.levels.push(level);
        self.updated_at = Utc::now();
    }

    /// Get current approval level
    pub fn get_current_level(&self) -> Option<&ApprovalLevel> {
        self.levels.get(self.current_level as usize)
    }

    /// Get current approval level (mutable)
    pub fn get_current_level_mut(&mut self) -> Option<&mut ApprovalLevel> {
        self.levels.get_mut(self.current_level as usize)
    }

    /// Check if user is authorized to approve at current level
    pub fn is_authorized(&self, user_id: Uuid) -> bool {
        if let Some(level) = self.get_current_level() {
            level.approvers.contains(&user_id)
        } else {
            false
        }
    }

    /// Approve the request
    pub fn approve(&mut self, user_id: Uuid, comments: Option<String>) -> ApprovalResult<()> {
        if self.status != ApprovalStatus::Pending {
            return Err(ApprovalError::AlreadyCompleted);
        }

        if !self.is_authorized(user_id) {
            return Err(ApprovalError::Unauthorized(user_id));
        }

        // Record the approval action
        let action = ApprovalAction {
            id: Uuid::new_v4(),
            action_type: ApprovalActionType::Approved,
            user_id,
            level: self.current_level,
            comments,
            timestamp: Utc::now(),
        };
        self.history.push(action);

        // Update current level
        if let Some(level) = self.get_current_level_mut() {
            level.status = ApprovalStatus::Approved;
            level.approved_by = Some(user_id);
            level.approved_at = Some(Utc::now());
        }

        // Check if there are more levels
        if (self.current_level as usize) < self.levels.len() - 1 {
            // Move to next level
            self.current_level += 1;
            self.updated_at = Utc::now();
        } else {
            // All levels approved - complete the request
            self.status = ApprovalStatus::Approved;
            self.completed_at = Some(Utc::now());
            self.updated_at = Utc::now();
        }

        Ok(())
    }

    /// Reject the request
    pub fn reject(&mut self, user_id: Uuid, comments: Option<String>) -> ApprovalResult<()> {
        if self.status != ApprovalStatus::Pending {
            return Err(ApprovalError::AlreadyCompleted);
        }

        if !self.is_authorized(user_id) {
            return Err(ApprovalError::Unauthorized(user_id));
        }

        // Record the rejection action
        let action = ApprovalAction {
            id: Uuid::new_v4(),
            action_type: ApprovalActionType::Rejected,
            user_id,
            level: self.current_level,
            comments,
            timestamp: Utc::now(),
        };
        self.history.push(action);

        // Update status
        self.status = ApprovalStatus::Rejected;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();

        if let Some(level) = self.get_current_level_mut() {
            level.status = ApprovalStatus::Rejected;
        }

        Ok(())
    }

    /// Delegate to another user
    pub fn delegate(&mut self, from_user: Uuid, to_user: Uuid, reason: Option<String>) -> ApprovalResult<()> {
        if self.status != ApprovalStatus::Pending {
            return Err(ApprovalError::AlreadyCompleted);
        }

        if !self.is_authorized(from_user) {
            return Err(ApprovalError::Unauthorized(from_user));
        }

        // Record the delegation action
        let action = ApprovalAction {
            id: Uuid::new_v4(),
            action_type: ApprovalActionType::Delegated { to_user },
            user_id: from_user,
            level: self.current_level,
            comments: reason,
            timestamp: Utc::now(),
        };
        self.history.push(action);

        // Update current level approvers
        if let Some(level) = self.get_current_level_mut() {
            // Remove the delegating user and add the delegate
            level.approvers.retain(|id| *id != from_user);
            level.approvers.push(to_user);
            level.delegated_from = Some(from_user);
            level.delegated_to = Some(to_user);
        }

        self.status = ApprovalStatus::Delegated;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Escalate to higher authority
    pub fn escalate(&mut self, user_id: Uuid, escalate_to: Vec<Uuid>, reason: String) -> ApprovalResult<()> {
        if self.status != ApprovalStatus::Pending {
            return Err(ApprovalError::AlreadyCompleted);
        }

        if escalate_to.is_empty() {
            return Err(ApprovalError::EscalationFailed(
                "No escalation users provided".to_string(),
            ));
        }

        // Record the escalation action
        let action = ApprovalAction {
            id: Uuid::new_v4(),
            action_type: ApprovalActionType::Escalated {
                to_users: escalate_to.clone(),
            },
            user_id,
            level: self.current_level,
            comments: Some(reason),
            timestamp: Utc::now(),
        };
        self.history.push(action);

        // Create new escalation level
        let escalation_level = ApprovalLevel {
            level: self.levels.len() as u32,
            name: format!("Escalation Level {}", self.levels.len()),
            approvers: escalate_to,
            required_approvals: 1,
            status: ApprovalStatus::Pending,
            approved_by: None,
            approved_at: None,
            delegated_from: None,
            delegated_to: None,
        };

        self.levels.push(escalation_level);
        self.current_level = (self.levels.len() - 1) as u32;
        self.status = ApprovalStatus::Escalated;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Cancel the approval request
    pub fn cancel(&mut self, user_id: Uuid, reason: Option<String>) -> ApprovalResult<()> {
        if self.status != ApprovalStatus::Pending {
            return Err(ApprovalError::AlreadyCompleted);
        }

        // Record the cancellation action
        let action = ApprovalAction {
            id: Uuid::new_v4(),
            action_type: ApprovalActionType::Cancelled,
            user_id,
            level: self.current_level,
            comments: reason,
            timestamp: Utc::now(),
        };
        self.history.push(action);

        self.status = ApprovalStatus::Cancelled;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Check if approval is overdue
    pub fn is_overdue(&self) -> bool {
        if let Some(due_date) = self.due_date {
            if self.status == ApprovalStatus::Pending {
                return Utc::now() > due_date;
            }
        }
        false
    }

    /// Get approval duration
    pub fn duration(&self) -> Option<chrono::Duration> {
        self.completed_at.map(|end| end - self.created_at)
    }
}

/// Approval level for multi-level approvals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalLevel {
    /// Level number (0-indexed)
    pub level: u32,
    /// Level name/description
    pub name: String,
    /// Users who can approve at this level
    pub approvers: Vec<Uuid>,
    /// Number of approvals required (for parallel approvals)
    pub required_approvals: u32,
    /// Current status of this level
    pub status: ApprovalStatus,
    /// User who approved (if approved)
    pub approved_by: Option<Uuid>,
    /// Approval timestamp
    pub approved_at: Option<DateTime<Utc>>,
    /// Delegation source user
    pub delegated_from: Option<Uuid>,
    /// Delegation target user
    pub delegated_to: Option<Uuid>,
}

impl ApprovalLevel {
    /// Create a new approval level
    pub fn new(level: u32, name: String, approvers: Vec<Uuid>) -> Self {
        Self {
            level,
            name,
            approvers,
            required_approvals: 1,
            status: ApprovalStatus::Pending,
            approved_by: None,
            approved_at: None,
            delegated_from: None,
            delegated_to: None,
        }
    }

    /// Create level with multiple required approvals
    pub fn with_required_approvals(mut self, count: u32) -> Self {
        self.required_approvals = count;
        self
    }
}

/// Approval action in history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalAction {
    /// Action ID
    pub id: Uuid,
    /// Type of action
    pub action_type: ApprovalActionType,
    /// User who performed the action
    pub user_id: Uuid,
    /// Approval level when action occurred
    pub level: u32,
    /// Optional comments
    pub comments: Option<String>,
    /// Action timestamp
    pub timestamp: DateTime<Utc>,
}

/// Types of approval actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalActionType {
    /// Approved the request
    Approved,
    /// Rejected the request
    Rejected,
    /// Delegated to another user
    Delegated { to_user: Uuid },
    /// Escalated to higher authority
    Escalated { to_users: Vec<Uuid> },
    /// Cancelled the request
    Cancelled,
    /// Commented on the request
    Commented,
}

/// Approval policy for automatic routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalPolicy {
    /// Policy ID
    pub id: Uuid,
    /// Policy name
    pub name: String,
    /// Description
    pub description: String,
    /// Conditions for this policy to apply
    pub conditions: Vec<PolicyCondition>,
    /// Approval levels to create
    pub levels: Vec<ApprovalLevelTemplate>,
    /// Auto-escalation settings
    pub auto_escalation: Option<AutoEscalation>,
    /// Is this policy active
    pub is_active: bool,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

impl ApprovalPolicy {
    /// Create a new approval policy
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            conditions: Vec::new(),
            levels: Vec::new(),
            auto_escalation: None,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Check if policy matches a request
    pub fn matches(&self, request: &ApprovalRequest) -> bool {
        // All conditions must be true
        self.conditions.iter().all(|condition| condition.evaluate(request))
    }

    /// Apply policy to create approval levels
    pub fn apply(&self, request: &mut ApprovalRequest) {
        for (index, level_template) in self.levels.iter().enumerate() {
            let level = ApprovalLevel {
                level: index as u32,
                name: level_template.name.clone(),
                approvers: level_template.approvers.clone(),
                required_approvals: level_template.required_approvals,
                status: ApprovalStatus::Pending,
                approved_by: None,
                approved_at: None,
                delegated_from: None,
                delegated_to: None,
            };
            request.add_level(level);
        }
    }
}

/// Policy condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCondition {
    /// Field to check
    pub field: String,
    /// Operator
    pub operator: String,
    /// Expected value
    pub value: serde_json::Value,
}

impl PolicyCondition {
    /// Evaluate condition against approval request
    pub fn evaluate(&self, request: &ApprovalRequest) -> bool {
        // Simplified evaluation - in production would use expression parser
        match self.field.as_str() {
            "priority" => {
                if let Some(priority) = self.value.as_u64() {
                    match self.operator.as_str() {
                        ">" => request.priority as u64 > priority,
                        ">=" => request.priority as u64 >= priority,
                        "=" => request.priority as u64 == priority,
                        "<" => (request.priority as u64) < priority,
                        "<=" => request.priority as u64 <= priority,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            _ => true, // Default to true for unknown fields
        }
    }
}

/// Template for approval level in policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalLevelTemplate {
    /// Level name
    pub name: String,
    /// Approvers (user IDs or roles)
    pub approvers: Vec<Uuid>,
    /// Required number of approvals
    pub required_approvals: u32,
}

/// Auto-escalation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoEscalation {
    /// Time to wait before escalation (in seconds)
    pub timeout_seconds: u64,
    /// Users to escalate to
    pub escalate_to: Vec<Uuid>,
    /// Whether to send notification before escalating
    pub notify_before_escalation: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approval_request_creation() {
        let workflow_id = Uuid::new_v4();
        let step_id = Uuid::new_v4();
        let requester_id = Uuid::new_v4();

        let request = ApprovalRequest::new(
            workflow_id,
            step_id,
            "Test Approval".to_string(),
            "Test Description".to_string(),
            requester_id,
        );

        assert_eq!(request.status, ApprovalStatus::Pending);
        assert_eq!(request.workflow_id, workflow_id);
    }

    #[test]
    fn test_single_level_approval() {
        let workflow_id = Uuid::new_v4();
        let step_id = Uuid::new_v4();
        let requester_id = Uuid::new_v4();
        let approver_id = Uuid::new_v4();

        let mut request = ApprovalRequest::new(
            workflow_id,
            step_id,
            "Test".to_string(),
            "Test".to_string(),
            requester_id,
        );

        let level = ApprovalLevel::new(
            0,
            "Manager Approval".to_string(),
            vec![approver_id],
        );
        request.add_level(level);

        // Approve
        assert!(request.approve(approver_id, Some("Looks good".to_string())).is_ok());
        assert_eq!(request.status, ApprovalStatus::Approved);
    }

    #[test]
    fn test_multi_level_approval() {
        let workflow_id = Uuid::new_v4();
        let step_id = Uuid::new_v4();
        let requester_id = Uuid::new_v4();
        let manager_id = Uuid::new_v4();
        let director_id = Uuid::new_v4();

        let mut request = ApprovalRequest::new(
            workflow_id,
            step_id,
            "Test".to_string(),
            "Test".to_string(),
            requester_id,
        );

        request.add_level(ApprovalLevel::new(0, "Manager".to_string(), vec![manager_id]));
        request.add_level(ApprovalLevel::new(1, "Director".to_string(), vec![director_id]));

        // Manager approves
        assert!(request.approve(manager_id, None).is_ok());
        assert_eq!(request.status, ApprovalStatus::Pending);
        assert_eq!(request.current_level, 1);

        // Director approves
        assert!(request.approve(director_id, None).is_ok());
        assert_eq!(request.status, ApprovalStatus::Approved);
    }

    #[test]
    fn test_delegation() {
        let workflow_id = Uuid::new_v4();
        let step_id = Uuid::new_v4();
        let requester_id = Uuid::new_v4();
        let approver_id = Uuid::new_v4();
        let delegate_id = Uuid::new_v4();

        let mut request = ApprovalRequest::new(
            workflow_id,
            step_id,
            "Test".to_string(),
            "Test".to_string(),
            requester_id,
        );

        request.add_level(ApprovalLevel::new(0, "Manager".to_string(), vec![approver_id]));

        // Delegate
        assert!(request.delegate(approver_id, delegate_id, Some("On vacation".to_string())).is_ok());
        assert_eq!(request.status, ApprovalStatus::Delegated);
        assert!(request.is_authorized(delegate_id));
        assert!(!request.is_authorized(approver_id));
    }
}
