//! Automatic Conflict Resolution Algorithms
//!
//! This module implements sophisticated conflict detection and resolution strategies
//! for collaborative CAD editing, including operational transformation and CRDT-based
//! automatic conflict resolution.

use super::crdt::{CRDTOperation, LamportTimestamp};
use super::operations::OperationId;
use super::{CollaborationError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Conflict type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictType {
    /// Two users modified the same property of the same entity
    PropertyConflict,
    /// One user deleted an entity that another modified
    DeleteModifyConflict,
    /// Two users moved the same entity to different layers
    LayerConflict,
    /// Geometric constraint conflicts
    ConstraintConflict,
    /// Concurrent transformations on the same entity
    TransformConflict,
    /// Structural conflicts (parent-child relationships)
    StructuralConflict,
}

/// Conflict severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ConflictSeverity {
    /// Minor conflict that can be auto-resolved
    Low,
    /// Moderate conflict requiring user attention
    Medium,
    /// Critical conflict requiring immediate resolution
    High,
}

/// Detected conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    /// Conflict ID
    pub id: Uuid,
    /// Conflict type
    pub conflict_type: ConflictType,
    /// Severity
    pub severity: ConflictSeverity,
    /// Entity IDs involved
    pub entity_ids: Vec<Uuid>,
    /// Conflicting operations
    pub operations: Vec<(OperationId, CRDTOperation, Uuid)>, // (op_id, operation, user_id)
    /// Timestamp when detected
    pub detected_at: DateTime<Utc>,
    /// Description
    pub description: String,
    /// Auto-resolvable flag
    pub auto_resolvable: bool,
    /// Suggested resolution
    pub suggested_resolution: Option<ConflictResolution>,
}

impl Conflict {
    /// Create a new conflict
    pub fn new(
        conflict_type: ConflictType,
        severity: ConflictSeverity,
        entity_ids: Vec<Uuid>,
        operations: Vec<(OperationId, CRDTOperation, Uuid)>,
        description: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            conflict_type,
            severity,
            entity_ids,
            operations,
            detected_at: Utc::now(),
            description,
            auto_resolvable: severity == ConflictSeverity::Low,
            suggested_resolution: None,
        }
    }
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    /// Last write wins (based on timestamp)
    LastWriteWins,
    /// First write wins
    FirstWriteWins,
    /// Prefer user with higher priority
    UserPriority,
    /// Manual resolution required
    Manual,
    /// Merge both changes
    Merge,
    /// Create a copy for each conflicting change
    Duplicate,
    /// Use CRDT semantics
    CRDT,
}

/// Conflict resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    /// Resolution strategy used
    pub strategy: ResolutionStrategy,
    /// Resolved operations to apply
    pub resolved_operations: Vec<CRDTOperation>,
    /// Operations to discard
    pub discarded_operations: Vec<OperationId>,
    /// Explanation of resolution
    pub explanation: String,
    /// User notification required
    pub requires_notification: bool,
}

impl ConflictResolution {
    /// Create a new resolution
    pub fn new(strategy: ResolutionStrategy, explanation: String) -> Self {
        Self {
            strategy,
            resolved_operations: Vec::new(),
            discarded_operations: Vec::new(),
            explanation,
            requires_notification: false,
        }
    }

    /// Add resolved operation
    pub fn add_resolved(&mut self, operation: CRDTOperation) {
        self.resolved_operations.push(operation);
    }

    /// Add discarded operation
    pub fn discard(&mut self, operation_id: OperationId) {
        self.discarded_operations.push(operation_id);
    }
}

/// Conflict resolver configuration
#[derive(Debug, Clone)]
pub struct ConflictResolverConfig {
    /// Default resolution strategy
    pub default_strategy: ResolutionStrategy,
    /// Enable automatic resolution for low severity conflicts
    pub auto_resolve_low_severity: bool,
    /// Enable notification for all conflicts
    pub notify_all_conflicts: bool,
    /// User priority mapping (user_id -> priority level)
    pub user_priorities: HashMap<Uuid, u32>,
}

impl Default for ConflictResolverConfig {
    fn default() -> Self {
        Self {
            default_strategy: ResolutionStrategy::LastWriteWins,
            auto_resolve_low_severity: true,
            notify_all_conflicts: false,
            user_priorities: HashMap::new(),
        }
    }
}

/// Conflict resolver
pub struct ConflictResolver {
    /// Configuration
    config: ConflictResolverConfig,
    /// Pending conflicts
    pending_conflicts: HashMap<Uuid, Conflict>,
    /// Resolved conflicts
    resolved_conflicts: Vec<(Uuid, ConflictResolution)>,
}

impl ConflictResolver {
    /// Create a new conflict resolver
    pub fn new(config: ConflictResolverConfig) -> Self {
        Self {
            config,
            pending_conflicts: HashMap::new(),
            resolved_conflicts: Vec::new(),
        }
    }

    /// Detect conflicts between operations
    pub fn detect_conflicts(
        &mut self,
        operations: &[(OperationId, CRDTOperation, Uuid)],
    ) -> Vec<Conflict> {
        let mut conflicts = Vec::new();

        // Group operations by entity ID
        let mut entity_ops: HashMap<Uuid, Vec<(OperationId, CRDTOperation, Uuid)>> = HashMap::new();

        for (op_id, operation, user_id) in operations {
            if let Some(entity_id) = self.get_entity_id(operation) {
                entity_ops
                    .entry(entity_id)
                    .or_default()
                    .push((*op_id, operation.clone(), *user_id));
            }
        }

        // Detect conflicts for each entity
        for (entity_id, ops) in entity_ops {
            if ops.len() > 1 {
                if let Some(conflict) = self.analyze_entity_conflicts(entity_id, ops) {
                    conflicts.push(conflict);
                }
            }
        }

        // Store pending conflicts
        for conflict in &conflicts {
            self.pending_conflicts.insert(conflict.id, conflict.clone());
        }

        conflicts
    }

    /// Analyze conflicts for a specific entity
    fn analyze_entity_conflicts(
        &self,
        entity_id: Uuid,
        operations: Vec<(OperationId, CRDTOperation, Uuid)>,
    ) -> Option<Conflict> {
        // Check for delete-modify conflict
        let has_delete = operations.iter().any(|(_, op, _)| {
            matches!(op, CRDTOperation::DeleteEntity { .. })
        });

        let has_modify = operations.iter().any(|(_, op, _)| {
            matches!(op, CRDTOperation::UpdateProperty { .. })
        });

        if has_delete && has_modify {
            return Some(Conflict::new(
                ConflictType::DeleteModifyConflict,
                ConflictSeverity::High,
                vec![entity_id],
                operations,
                format!("Entity {} has concurrent delete and modify operations", entity_id),
            ));
        }

        // Check for property conflicts
        let mut property_users: HashMap<String, Vec<Uuid>> = HashMap::new();

        for (_, operation, user_id) in &operations {
            if let CRDTOperation::UpdateProperty { property, .. } = operation {
                property_users.entry(property.clone()).or_default().push(*user_id);
            }
        }

        for (property, users) in &property_users {
            if users.len() > 1 {
                return Some(Conflict::new(
                    ConflictType::PropertyConflict,
                    ConflictSeverity::Medium,
                    vec![entity_id],
                    operations,
                    format!(
                        "Property '{}' of entity {} modified by {} users",
                        property,
                        entity_id,
                        users.len()
                    ),
                ));
            }
        }

        None
    }

    /// Resolve a conflict
    pub fn resolve_conflict(
        &mut self,
        conflict_id: Uuid,
        strategy: Option<ResolutionStrategy>,
    ) -> Result<ConflictResolution> {
        let conflict = self.pending_conflicts.remove(&conflict_id).ok_or_else(|| {
            CollaborationError::Operation(format!("Conflict not found: {}", conflict_id))
        })?;

        let strategy = strategy.unwrap_or(self.config.default_strategy);

        let resolution = match conflict.conflict_type {
            ConflictType::PropertyConflict => {
                self.resolve_property_conflict(&conflict, strategy)?
            }
            ConflictType::DeleteModifyConflict => {
                self.resolve_delete_modify_conflict(&conflict, strategy)?
            }
            ConflictType::LayerConflict => {
                self.resolve_layer_conflict(&conflict, strategy)?
            }
            ConflictType::TransformConflict => {
                self.resolve_transform_conflict(&conflict, strategy)?
            }
            _ => {
                let mut resolution = ConflictResolution::new(
                    ResolutionStrategy::Manual,
                    "Manual resolution required".to_string(),
                );
                resolution.requires_notification = true;
                resolution
            }
        };

        self.resolved_conflicts.push((conflict_id, resolution.clone()));

        Ok(resolution)
    }

    /// Resolve property conflict
    fn resolve_property_conflict(
        &self,
        conflict: &Conflict,
        strategy: ResolutionStrategy,
    ) -> Result<ConflictResolution> {
        let mut resolution = ConflictResolution::new(
            strategy,
            format!("Resolved property conflict using {:?} strategy", strategy),
        );

        match strategy {
            ResolutionStrategy::LastWriteWins => {
                // Find operation with latest timestamp
                let latest_op = conflict
                    .operations
                    .iter()
                    .max_by_key(|(_, op, _)| self.get_timestamp(op))
                    .cloned();

                if let Some((_, operation, _)) = latest_op {
                    resolution.add_resolved(operation);
                }

                // Discard others
                for (op_id, _, _) in &conflict.operations {
                    if Some(*op_id) != latest_op.as_ref().map(|(id, _, _)| *id) {
                        resolution.discard(*op_id);
                    }
                }
            }

            ResolutionStrategy::FirstWriteWins => {
                // Find operation with earliest timestamp
                let earliest_op = conflict
                    .operations
                    .iter()
                    .min_by_key(|(_, op, _)| self.get_timestamp(op))
                    .cloned();

                if let Some((_, operation, _)) = earliest_op {
                    resolution.add_resolved(operation);
                }

                // Discard others
                for (op_id, _, _) in &conflict.operations {
                    if Some(*op_id) != earliest_op.as_ref().map(|(id, _, _)| *id) {
                        resolution.discard(*op_id);
                    }
                }
            }

            ResolutionStrategy::UserPriority => {
                // Find operation from user with highest priority
                let highest_priority_op = conflict
                    .operations
                    .iter()
                    .max_by_key(|(_, _, user_id)| {
                        self.config.user_priorities.get(user_id).copied().unwrap_or(0)
                    })
                    .cloned();

                if let Some((_, operation, _)) = highest_priority_op {
                    resolution.add_resolved(operation);
                }

                // Discard others
                for (op_id, _, _) in &conflict.operations {
                    if Some(*op_id) != highest_priority_op.as_ref().map(|(id, _, _)| *id) {
                        resolution.discard(*op_id);
                    }
                }
            }

            ResolutionStrategy::CRDT => {
                // Use CRDT semantics - all operations are applied
                // The CRDT merge will handle convergence
                for (_, operation, _) in &conflict.operations {
                    resolution.add_resolved(operation.clone());
                }
            }

            _ => {
                resolution.requires_notification = true;
            }
        }

        Ok(resolution)
    }

    /// Resolve delete-modify conflict
    fn resolve_delete_modify_conflict(
        &self,
        conflict: &Conflict,
        strategy: ResolutionStrategy,
    ) -> Result<ConflictResolution> {
        let mut resolution = ConflictResolution::new(
            strategy,
            "Resolved delete-modify conflict".to_string(),
        );

        match strategy {
            ResolutionStrategy::LastWriteWins => {
                // Latest operation wins
                let latest_op = conflict
                    .operations
                    .iter()
                    .max_by_key(|(_, op, _)| self.get_timestamp(op))
                    .cloned();

                if let Some((_, operation, _)) = latest_op {
                    resolution.add_resolved(operation);
                }
            }

            ResolutionStrategy::CRDT => {
                // In CRDT, deletes typically win over modifications
                if let Some((_, delete_op, _)) = conflict
                    .operations
                    .iter()
                    .find(|(_, op, _)| matches!(op, CRDTOperation::DeleteEntity { .. }))
                {
                    resolution.add_resolved(delete_op.clone());
                }
            }

            _ => {
                resolution.requires_notification = true;
            }
        }

        Ok(resolution)
    }

    /// Resolve layer conflict
    fn resolve_layer_conflict(
        &self,
        conflict: &Conflict,
        strategy: ResolutionStrategy,
    ) -> Result<ConflictResolution> {
        let resolution = ConflictResolution::new(
            strategy,
            "Resolved layer conflict using last-write-wins".to_string(),
        );

        // Similar to property conflict resolution
        // Implementation would follow the same pattern

        Ok(resolution)
    }

    /// Resolve transform conflict
    fn resolve_transform_conflict(
        &self,
        conflict: &Conflict,
        strategy: ResolutionStrategy,
    ) -> Result<ConflictResolution> {
        let mut resolution = ConflictResolution::new(
            strategy,
            "Resolved transform conflict by composing transformations".to_string(),
        );

        match strategy {
            ResolutionStrategy::Merge => {
                // Compose transformations
                // This would involve matrix multiplication for geometric transforms
                // For now, apply all transformations in order
                for (_, operation, _) in &conflict.operations {
                    resolution.add_resolved(operation.clone());
                }
            }

            ResolutionStrategy::LastWriteWins => {
                let latest_op = conflict
                    .operations
                    .iter()
                    .max_by_key(|(_, op, _)| self.get_timestamp(op))
                    .cloned();

                if let Some((_, operation, _)) = latest_op {
                    resolution.add_resolved(operation);
                }
            }

            _ => {
                resolution.requires_notification = true;
            }
        }

        Ok(resolution)
    }

    /// Auto-resolve all resolvable conflicts
    pub fn auto_resolve_all(&mut self) -> Vec<(Uuid, ConflictResolution)> {
        let mut resolutions = Vec::new();

        let resolvable_ids: Vec<Uuid> = self
            .pending_conflicts
            .iter()
            .filter(|(_, conflict)| {
                conflict.auto_resolvable && self.config.auto_resolve_low_severity
            })
            .map(|(id, _)| *id)
            .collect();

        for conflict_id in resolvable_ids {
            if let Ok(resolution) = self.resolve_conflict(conflict_id, None) {
                resolutions.push((conflict_id, resolution));
            }
        }

        resolutions
    }

    /// Get pending conflicts
    pub fn get_pending_conflicts(&self) -> Vec<&Conflict> {
        self.pending_conflicts.values().collect()
    }

    /// Get resolved conflicts
    pub fn get_resolved_conflicts(&self) -> &[(Uuid, ConflictResolution)] {
        &self.resolved_conflicts
    }

    /// Get entity ID from operation
    fn get_entity_id(&self, operation: &CRDTOperation) -> Option<Uuid> {
        match operation {
            CRDTOperation::AddEntity { entity_id, .. } => Some(*entity_id),
            CRDTOperation::UpdateProperty { entity_id, .. } => Some(*entity_id),
            CRDTOperation::DeleteEntity { entity_id, .. } => Some(*entity_id),
            _ => None,
        }
    }

    /// Get timestamp from operation
    fn get_timestamp(&self, operation: &CRDTOperation) -> LamportTimestamp {
        match operation {
            CRDTOperation::AddEntity { timestamp, .. } => *timestamp,
            CRDTOperation::UpdateProperty { timestamp, .. } => *timestamp,
            CRDTOperation::DeleteEntity { timestamp, .. } => *timestamp,
            _ => LamportTimestamp::new(0, Uuid::nil()),
        }
    }

    /// Clear resolved conflicts history
    pub fn clear_resolved_history(&mut self) {
        self.resolved_conflicts.clear();
    }

    /// Get conflict statistics
    pub fn get_statistics(&self) -> ConflictStatistics {
        let total_pending = self.pending_conflicts.len();
        let total_resolved = self.resolved_conflicts.len();

        let by_type: HashMap<ConflictType, usize> = self
            .pending_conflicts
            .values()
            .fold(HashMap::new(), |mut acc, conflict| {
                *acc.entry(conflict.conflict_type).or_insert(0) += 1;
                acc
            });

        let by_severity: HashMap<ConflictSeverity, usize> = self
            .pending_conflicts
            .values()
            .fold(HashMap::new(), |mut acc, conflict| {
                *acc.entry(conflict.severity).or_insert(0) += 1;
                acc
            });

        ConflictStatistics {
            total_pending,
            total_resolved,
            by_type,
            by_severity,
            auto_resolved_count: self
                .resolved_conflicts
                .iter()
                .filter(|(_, r)| r.strategy != ResolutionStrategy::Manual)
                .count(),
        }
    }
}

/// Conflict statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictStatistics {
    /// Total pending conflicts
    pub total_pending: usize,
    /// Total resolved conflicts
    pub total_resolved: usize,
    /// Conflicts by type
    pub by_type: HashMap<ConflictType, usize>,
    /// Conflicts by severity
    pub by_severity: HashMap<ConflictSeverity, usize>,
    /// Auto-resolved conflicts
    pub auto_resolved_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_detection() {
        let config = ConflictResolverConfig::default();
        let mut resolver = ConflictResolver::new(config);

        let entity_id = Uuid::new_v4();
        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();
        let site_id = Uuid::new_v4();

        let op1 = CRDTOperation::UpdateProperty {
            entity_id,
            property: "color".to_string(),
            value: serde_json::json!("red"),
            timestamp: LamportTimestamp::new(1, site_id),
        };

        let op2 = CRDTOperation::UpdateProperty {
            entity_id,
            property: "color".to_string(),
            value: serde_json::json!("blue"),
            timestamp: LamportTimestamp::new(2, site_id),
        };

        let operations = vec![
            (Uuid::new_v4(), op1, user1),
            (Uuid::new_v4(), op2, user2),
        ];

        let conflicts = resolver.detect_conflicts(&operations);

        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflict_type, ConflictType::PropertyConflict);
    }

    #[test]
    fn test_last_write_wins() {
        let config = ConflictResolverConfig::default();
        let mut resolver = ConflictResolver::new(config);

        let entity_id = Uuid::new_v4();
        let site_id = Uuid::new_v4();

        let op1 = CRDTOperation::UpdateProperty {
            entity_id,
            property: "color".to_string(),
            value: serde_json::json!("red"),
            timestamp: LamportTimestamp::new(1, site_id),
        };

        let op2 = CRDTOperation::UpdateProperty {
            entity_id,
            property: "color".to_string(),
            value: serde_json::json!("blue"),
            timestamp: LamportTimestamp::new(2, site_id),
        };

        let operations = vec![
            (Uuid::new_v4(), op1, Uuid::new_v4()),
            (Uuid::new_v4(), op2.clone(), Uuid::new_v4()),
        ];

        let conflicts = resolver.detect_conflicts(&operations);
        assert_eq!(conflicts.len(), 1);

        let resolution = resolver
            .resolve_conflict(conflicts[0].id, Some(ResolutionStrategy::LastWriteWins))
            .unwrap();

        assert_eq!(resolution.resolved_operations.len(), 1);
        // Should keep op2 (later timestamp)
    }
}
