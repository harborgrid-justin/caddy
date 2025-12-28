//! # Conflict Resolution
//!
//! Implements various conflict resolution strategies for concurrent edits.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

use super::crdt::LamportTime;
use super::ot::Operation;

/// Errors related to conflict resolution
#[derive(Debug, Error)]
pub enum ConflictError {
    #[error("Unresolvable conflict")]
    UnresolvableConflict,
    #[error("Invalid merge strategy: {0}")]
    InvalidStrategy(String),
    #[error("No resolution available")]
    NoResolution,
    #[error("Manual resolution required")]
    ManualResolutionRequired,
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    /// Last writer wins (based on timestamp)
    LastWriterWins,
    /// First writer wins
    FirstWriterWins,
    /// Prefer local changes
    PreferLocal,
    /// Prefer remote changes
    PreferRemote,
    /// Merge both changes
    Merge,
    /// Require manual resolution
    Manual,
    /// Use custom priority
    Priority,
}

/// A conflict between two operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    /// Unique conflict ID
    pub id: Uuid,
    /// Local operation
    pub local_op: Operation,
    /// Remote operation
    pub remote_op: Operation,
    /// Local timestamp
    pub local_time: LamportTime,
    /// Remote timestamp
    pub remote_time: LamportTime,
    /// Conflict type
    pub conflict_type: ConflictType,
    /// Resolution status
    pub status: ConflictStatus,
    /// Resolved operation (if resolved)
    pub resolution: Option<Operation>,
    /// Timestamp when conflict was detected
    pub detected_at: chrono::DateTime<chrono::Utc>,
}

impl Conflict {
    /// Create a new conflict
    pub fn new(
        local_op: Operation,
        remote_op: Operation,
        local_time: LamportTime,
        remote_time: LamportTime,
    ) -> Self {
        let conflict_type = Self::determine_type(&local_op, &remote_op);

        Self {
            id: Uuid::new_v4(),
            local_op,
            remote_op,
            local_time,
            remote_time,
            conflict_type,
            status: ConflictStatus::Pending,
            resolution: None,
            detected_at: chrono::Utc::now(),
        }
    }

    /// Determine the type of conflict
    fn determine_type(local: &Operation, remote: &Operation) -> ConflictType {
        use super::ot::OpType;

        match (&local.op_type, &remote.op_type) {
            (OpType::Insert { .. }, OpType::Insert { .. }) => ConflictType::ConcurrentInsert,
            (OpType::Delete { .. }, OpType::Delete { .. }) => ConflictType::ConcurrentDelete,
            (OpType::Insert { .. }, OpType::Delete { .. })
            | (OpType::Delete { .. }, OpType::Insert { .. }) => {
                ConflictType::InsertDeleteConflict
            }
            _ => ConflictType::Other,
        }
    }

    /// Resolve the conflict using a strategy
    pub fn resolve(&mut self, strategy: ResolutionStrategy) -> Result<Operation, ConflictError> {
        let resolution = match strategy {
            ResolutionStrategy::LastWriterWins => self.resolve_last_writer_wins()?,
            ResolutionStrategy::FirstWriterWins => self.resolve_first_writer_wins()?,
            ResolutionStrategy::PreferLocal => self.local_op.clone(),
            ResolutionStrategy::PreferRemote => self.remote_op.clone(),
            ResolutionStrategy::Merge => self.resolve_merge()?,
            ResolutionStrategy::Manual => {
                self.status = ConflictStatus::RequiresManual;
                return Err(ConflictError::ManualResolutionRequired);
            }
            ResolutionStrategy::Priority => {
                self.status = ConflictStatus::RequiresManual;
                return Err(ConflictError::ManualResolutionRequired);
            }
        };

        self.resolution = Some(resolution.clone());
        self.status = ConflictStatus::Resolved;

        Ok(resolution)
    }

    /// Resolve using last-writer-wins strategy
    fn resolve_last_writer_wins(&self) -> Result<Operation, ConflictError> {
        if self.remote_time > self.local_time {
            Ok(self.remote_op.clone())
        } else if self.local_time > self.remote_time {
            Ok(self.local_op.clone())
        } else {
            // Same timestamp, use client ID as tiebreaker
            if self.remote_op.client_id > self.local_op.client_id {
                Ok(self.remote_op.clone())
            } else {
                Ok(self.local_op.clone())
            }
        }
    }

    /// Resolve using first-writer-wins strategy
    fn resolve_first_writer_wins(&self) -> Result<Operation, ConflictError> {
        if self.local_time < self.remote_time {
            Ok(self.local_op.clone())
        } else if self.remote_time < self.local_time {
            Ok(self.remote_op.clone())
        } else {
            if self.local_op.client_id < self.remote_op.client_id {
                Ok(self.local_op.clone())
            } else {
                Ok(self.remote_op.clone())
            }
        }
    }

    /// Resolve by merging operations
    fn resolve_merge(&self) -> Result<Operation, ConflictError> {
        use super::ot::{transform, OpType};

        match (&self.local_op.op_type, &self.remote_op.op_type) {
            (OpType::Insert { .. }, OpType::Insert { .. }) => {
                // Transform and keep both
                let (transformed_local, _) = transform(&self.local_op, &self.remote_op)
                    .map_err(|_| ConflictError::UnresolvableConflict)?;
                Ok(transformed_local)
            }
            _ => Err(ConflictError::UnresolvableConflict),
        }
    }

    /// Set manual resolution
    pub fn set_manual_resolution(&mut self, resolution: Operation) {
        self.resolution = Some(resolution);
        self.status = ConflictStatus::Resolved;
    }
}

/// Type of conflict
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictType {
    /// Two concurrent insertions
    ConcurrentInsert,
    /// Two concurrent deletions
    ConcurrentDelete,
    /// Insert vs delete conflict
    InsertDeleteConflict,
    /// Other types
    Other,
}

/// Status of conflict resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictStatus {
    /// Conflict is pending resolution
    Pending,
    /// Conflict has been resolved automatically
    Resolved,
    /// Conflict requires manual resolution
    RequiresManual,
    /// Conflict was manually resolved
    ManuallyResolved,
}

/// Conflict resolver
#[derive(Debug)]
pub struct ConflictResolver {
    /// Default resolution strategy
    default_strategy: ResolutionStrategy,
    /// Active conflicts
    conflicts: HashMap<Uuid, Conflict>,
    /// Resolved conflicts history
    resolved: Vec<Conflict>,
    /// Maximum history size
    max_history: usize,
}

impl ConflictResolver {
    /// Create a new conflict resolver
    pub fn new(default_strategy: ResolutionStrategy) -> Self {
        Self {
            default_strategy,
            conflicts: HashMap::new(),
            resolved: Vec::new(),
            max_history: 1000,
        }
    }

    /// Set default strategy
    pub fn set_default_strategy(&mut self, strategy: ResolutionStrategy) {
        self.default_strategy = strategy;
    }

    /// Detect and register a conflict
    pub fn detect_conflict(
        &mut self,
        local_op: Operation,
        remote_op: Operation,
        local_time: LamportTime,
        remote_time: LamportTime,
    ) -> Uuid {
        let conflict = Conflict::new(local_op, remote_op, local_time, remote_time);
        let id = conflict.id;
        self.conflicts.insert(id, conflict);
        id
    }

    /// Resolve a conflict using default strategy
    pub fn resolve_conflict(&mut self, conflict_id: Uuid) -> Result<Operation, ConflictError> {
        self.resolve_conflict_with(conflict_id, self.default_strategy)
    }

    /// Resolve a conflict with a specific strategy
    pub fn resolve_conflict_with(
        &mut self,
        conflict_id: Uuid,
        strategy: ResolutionStrategy,
    ) -> Result<Operation, ConflictError> {
        let conflict = self
            .conflicts
            .get_mut(&conflict_id)
            .ok_or(ConflictError::NoResolution)?;

        let resolution = conflict.resolve(strategy)?;

        // Move to resolved history
        if let Some(resolved_conflict) = self.conflicts.remove(&conflict_id) {
            self.add_to_history(resolved_conflict);
        }

        Ok(resolution)
    }

    /// Manually resolve a conflict
    pub fn manual_resolve(
        &mut self,
        conflict_id: Uuid,
        resolution: Operation,
    ) -> Result<(), ConflictError> {
        let conflict = self
            .conflicts
            .get_mut(&conflict_id)
            .ok_or(ConflictError::NoResolution)?;

        conflict.set_manual_resolution(resolution);
        conflict.status = ConflictStatus::ManuallyResolved;

        // Move to resolved history
        if let Some(resolved_conflict) = self.conflicts.remove(&conflict_id) {
            self.add_to_history(resolved_conflict);
        }

        Ok(())
    }

    /// Get all pending conflicts
    pub fn pending_conflicts(&self) -> Vec<&Conflict> {
        self.conflicts.values().collect()
    }

    /// Get a specific conflict
    pub fn get_conflict(&self, id: Uuid) -> Option<&Conflict> {
        self.conflicts.get(&id)
    }

    /// Get conflict count
    pub fn conflict_count(&self) -> usize {
        self.conflicts.len()
    }

    /// Get resolved conflicts
    pub fn resolved_conflicts(&self) -> &[Conflict] {
        &self.resolved
    }

    /// Add to resolved history
    fn add_to_history(&mut self, conflict: Conflict) {
        self.resolved.push(conflict);
        if self.resolved.len() > self.max_history {
            self.resolved.remove(0);
        }
    }

    /// Clear all conflicts
    pub fn clear(&mut self) {
        self.conflicts.clear();
    }

    /// Get statistics
    pub fn stats(&self) -> ConflictStats {
        let pending = self.conflicts.len();
        let resolved = self.resolved.len();
        let requires_manual = self
            .conflicts
            .values()
            .filter(|c| c.status == ConflictStatus::RequiresManual)
            .count();

        ConflictStats {
            pending,
            resolved,
            requires_manual,
        }
    }
}

/// Conflict statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictStats {
    pub pending: usize,
    pub resolved: usize,
    pub requires_manual: usize,
}

/// Three-way merge helper
pub struct ThreeWayMerge {
    /// Base (common ancestor) content
    base: String,
    /// Local changes
    local: String,
    /// Remote changes
    remote: String,
}

impl ThreeWayMerge {
    /// Create a new three-way merge
    pub fn new(base: String, local: String, remote: String) -> Self {
        Self {
            base,
            local,
            remote,
        }
    }

    /// Perform the merge
    pub fn merge(&self) -> Result<String, ConflictError> {
        // Simple line-based three-way merge
        let base_lines: Vec<&str> = self.base.lines().collect();
        let local_lines: Vec<&str> = self.local.lines().collect();
        let remote_lines: Vec<&str> = self.remote.lines().collect();

        let mut result: Vec<String> = Vec::new();
        let max_len = base_lines.len().max(local_lines.len()).max(remote_lines.len());

        for i in 0..max_len {
            let base_line = base_lines.get(i).copied();
            let local_line = local_lines.get(i).copied();
            let remote_line = remote_lines.get(i).copied();

            match (base_line, local_line, remote_line) {
                (Some(b), Some(l), Some(r)) => {
                    if l == r {
                        result.push(l.to_string());
                    } else if l == b {
                        result.push(r.to_string());
                    } else if r == b {
                        result.push(l.to_string());
                    } else {
                        // Conflict
                        result.push(format!("<<<<<<< LOCAL\n{}\n=======\n{}\n>>>>>>> REMOTE", l, r));
                    }
                }
                (_, Some(l), Some(r)) if l == r => result.push(l.to_string()),
                (_, Some(l), None) => result.push(l.to_string()),
                (_, None, Some(r)) => result.push(r.to_string()),
                _ => {}
            }
        }

        Ok(result.join("\n"))
    }

    /// Check if merge would have conflicts
    pub fn has_conflicts(&self) -> bool {
        self.merge()
            .map(|m| m.contains("<<<<<<< LOCAL"))
            .unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_creation() {
        let client1 = Uuid::new_v4();
        let client2 = Uuid::new_v4();

        let op1 = Operation::insert(0, "hello".to_string(), 1, client1);
        let op2 = Operation::insert(0, "world".to_string(), 1, client2);

        let conflict = Conflict::new(op1, op2, LamportTime(1), LamportTime(2));

        assert_eq!(conflict.status, ConflictStatus::Pending);
        assert_eq!(conflict.conflict_type, ConflictType::ConcurrentInsert);
    }

    #[test]
    fn test_last_writer_wins() {
        let client1 = Uuid::new_v4();
        let client2 = Uuid::new_v4();

        let op1 = Operation::insert(0, "hello".to_string(), 1, client1);
        let op2 = Operation::insert(0, "world".to_string(), 1, client2);

        let mut conflict = Conflict::new(op1, op2.clone(), LamportTime(1), LamportTime(2));

        let resolution = conflict.resolve(ResolutionStrategy::LastWriterWins).unwrap();

        // Remote has later timestamp, should win
        assert_eq!(resolution.id, op2.id);
    }

    #[test]
    fn test_conflict_resolver() {
        let mut resolver = ConflictResolver::new(ResolutionStrategy::LastWriterWins);

        let client1 = Uuid::new_v4();
        let client2 = Uuid::new_v4();

        let op1 = Operation::insert(0, "hello".to_string(), 1, client1);
        let op2 = Operation::insert(0, "world".to_string(), 1, client2);

        let conflict_id = resolver.detect_conflict(op1, op2, LamportTime(1), LamportTime(2));

        assert_eq!(resolver.conflict_count(), 1);

        resolver.resolve_conflict(conflict_id).unwrap();

        assert_eq!(resolver.conflict_count(), 0);
        assert_eq!(resolver.resolved_conflicts().len(), 1);
    }

    #[test]
    fn test_three_way_merge() {
        let base = "line1\nline2\nline3".to_string();
        let local = "line1\nmodified2\nline3".to_string();
        let remote = "line1\nline2\nmodified3".to_string();

        let merge = ThreeWayMerge::new(base, local, remote);
        let result = merge.merge().unwrap();

        assert!(result.contains("modified2"));
        assert!(result.contains("modified3"));
        assert!(!merge.has_conflicts());
    }

    #[test]
    fn test_three_way_merge_with_conflict() {
        let base = "line1\nline2\nline3".to_string();
        let local = "line1\nlocal2\nline3".to_string();
        let remote = "line1\nremote2\nline3".to_string();

        let merge = ThreeWayMerge::new(base, local, remote);
        let result = merge.merge().unwrap();

        assert!(merge.has_conflicts());
        assert!(result.contains("<<<<<<< LOCAL"));
    }

    #[test]
    fn test_manual_resolution() {
        let mut resolver = ConflictResolver::new(ResolutionStrategy::Manual);

        let client1 = Uuid::new_v4();
        let client2 = Uuid::new_v4();

        let op1 = Operation::insert(0, "hello".to_string(), 1, client1);
        let op2 = Operation::insert(0, "world".to_string(), 1, client2);

        let conflict_id = resolver.detect_conflict(op1.clone(), op2, LamportTime(1), LamportTime(2));

        // Manual resolution
        let manual_op = Operation::insert(0, "merged".to_string(), 1, client1);
        resolver.manual_resolve(conflict_id, manual_op).unwrap();

        assert_eq!(resolver.conflict_count(), 0);
    }
}
