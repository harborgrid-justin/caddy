//! Document Versioning with Branching and Merging
//!
//! This module implements a Git-like versioning system for CAD documents,
//! allowing users to create branches, merge changes, and navigate version history.

use super::crdt::{CRDTOperation, DocumentSnapshot};
use super::{CollaborationError, Result};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use uuid::Uuid;

/// Version identifier (commit hash equivalent)
pub type VersionId = Uuid;

/// Branch name
pub type BranchName = String;

/// Version metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    /// Version ID
    pub id: VersionId,
    /// Parent version ID(s) - multiple parents for merge commits
    pub parents: Vec<VersionId>,
    /// Author information
    pub author: AuthorInfo,
    /// Commit message
    pub message: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Operations included in this version
    pub operations: Vec<CRDTOperation>,
    /// Document snapshot at this version
    pub snapshot: Option<Arc<DocumentSnapshot>>,
    /// Tags associated with this version
    pub tags: Vec<String>,
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Version {
    /// Create a new version
    pub fn new(
        id: VersionId,
        parents: Vec<VersionId>,
        author: AuthorInfo,
        message: String,
        operations: Vec<CRDTOperation>,
    ) -> Self {
        Self {
            id,
            parents,
            author,
            message,
            timestamp: Utc::now(),
            operations,
            snapshot: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Check if this is a merge commit
    pub fn is_merge(&self) -> bool {
        self.parents.len() > 1
    }

    /// Check if this is the root commit
    pub fn is_root(&self) -> bool {
        self.parents.is_empty()
    }

    /// Add a tag
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Remove a tag
    pub fn remove_tag(&mut self, tag: &str) -> bool {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            true
        } else {
            false
        }
    }
}

/// Author information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorInfo {
    /// User ID
    pub user_id: Uuid,
    /// Display name
    pub name: String,
    /// Email address
    pub email: String,
}

impl AuthorInfo {
    /// Create new author info
    pub fn new(user_id: Uuid, name: String, email: String) -> Self {
        Self {
            user_id,
            name,
            email,
        }
    }
}

/// Branch information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    /// Branch name
    pub name: BranchName,
    /// Current HEAD version
    pub head: VersionId,
    /// Branch description
    pub description: Option<String>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Branch protection rules
    pub protected: bool,
}

impl Branch {
    /// Create a new branch
    pub fn new(name: BranchName, head: VersionId) -> Self {
        let now = Utc::now();
        Self {
            name,
            head,
            description: None,
            created_at: now,
            updated_at: now,
            protected: false,
        }
    }

    /// Update HEAD to a new version
    pub fn update_head(&mut self, version_id: VersionId) {
        self.head = version_id;
        self.updated_at = Utc::now();
    }
}

/// Diff between two versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDiff {
    /// From version
    pub from_version: VersionId,
    /// To version
    pub to_version: VersionId,
    /// Added operations
    pub added_operations: Vec<CRDTOperation>,
    /// Number of entities added
    pub entities_added: usize,
    /// Number of entities modified
    pub entities_modified: usize,
    /// Number of entities deleted
    pub entities_deleted: usize,
    /// Changed properties
    pub changed_properties: HashMap<Uuid, Vec<String>>,
}

impl VersionDiff {
    /// Create an empty diff
    pub fn empty(from: VersionId, to: VersionId) -> Self {
        Self {
            from_version: from,
            to_version: to,
            added_operations: Vec::new(),
            entities_added: 0,
            entities_modified: 0,
            entities_deleted: 0,
            changed_properties: HashMap::new(),
        }
    }

    /// Check if diff is empty
    pub fn is_empty(&self) -> bool {
        self.added_operations.is_empty()
    }

    /// Get total changes
    pub fn total_changes(&self) -> usize {
        self.entities_added + self.entities_modified + self.entities_deleted
    }
}

/// Merge strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeStrategy {
    /// Fast-forward merge (no conflicts possible)
    FastForward,
    /// Three-way merge using common ancestor
    ThreeWay,
    /// Recursive merge for complex histories
    Recursive,
    /// Ours - prefer current branch on conflicts
    Ours,
    /// Theirs - prefer merged branch on conflicts
    Theirs,
}

/// Merge result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    /// Merge commit version ID
    pub merge_version: VersionId,
    /// Strategy used
    pub strategy: MergeStrategy,
    /// Was fast-forward merge
    pub fast_forward: bool,
    /// Number of conflicts
    pub conflicts: usize,
    /// Conflicting operations
    pub conflicting_operations: Vec<(CRDTOperation, CRDTOperation)>,
    /// Successfully merged operations
    pub merged_operations: Vec<CRDTOperation>,
}

impl MergeResult {
    /// Check if merge was successful without conflicts
    pub fn is_clean(&self) -> bool {
        self.conflicts == 0
    }

    /// Check if merge had conflicts
    pub fn has_conflicts(&self) -> bool {
        self.conflicts > 0
    }
}

/// Version control system for documents
pub struct VersionControl {
    /// Document ID
    document_id: Uuid,
    /// All versions indexed by ID
    versions: Arc<RwLock<HashMap<VersionId, Version>>>,
    /// Branches
    branches: Arc<RwLock<HashMap<BranchName, Branch>>>,
    /// Current branch name
    current_branch: Arc<RwLock<BranchName>>,
    /// Tags to version mapping
    tags: Arc<RwLock<HashMap<String, VersionId>>>,
    /// Topologically sorted version history
    history: Arc<RwLock<Vec<VersionId>>>,
}

impl VersionControl {
    /// Create a new version control system
    pub fn new(document_id: Uuid, initial_author: AuthorInfo) -> Self {
        // Create initial commit
        let root_version_id = Uuid::new_v4();
        let root_version = Version::new(
            root_version_id,
            vec![],
            initial_author,
            "Initial commit".to_string(),
            vec![],
        );

        // Create main branch
        let main_branch = Branch::new("main".to_string(), root_version_id);

        let mut versions = HashMap::new();
        versions.insert(root_version_id, root_version);

        let mut branches = HashMap::new();
        branches.insert("main".to_string(), main_branch);

        let history = vec![root_version_id];

        Self {
            document_id,
            versions: Arc::new(RwLock::new(versions)),
            branches: Arc::new(RwLock::new(branches)),
            current_branch: Arc::new(RwLock::new("main".to_string())),
            tags: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(history)),
        }
    }

    /// Create a new commit
    pub fn commit(
        &self,
        author: AuthorInfo,
        message: String,
        operations: Vec<CRDTOperation>,
    ) -> Result<VersionId> {
        let version_id = Uuid::new_v4();

        let current_branch_name = self.current_branch.read().clone();
        let current_head = {
            let branches = self.branches.read();
            branches
                .get(&current_branch_name)
                .ok_or_else(|| {
                    CollaborationError::InvalidState(format!(
                        "Branch not found: {}",
                        current_branch_name
                    ))
                })?
                .head
        };

        // Create new version
        let version = Version::new(version_id, vec![current_head], author, message, operations);

        // Update versions
        {
            let mut versions = self.versions.write();
            versions.insert(version_id, version);
        }

        // Update branch HEAD
        {
            let mut branches = self.branches.write();
            if let Some(branch) = branches.get_mut(&current_branch_name) {
                branch.update_head(version_id);
            }
        }

        // Update history
        {
            let mut history = self.history.write();
            history.push(version_id);
        }

        Ok(version_id)
    }

    /// Create a new branch
    pub fn create_branch(&self, name: BranchName, start_version: Option<VersionId>) -> Result<()> {
        let start_version = start_version.unwrap_or_else(|| {
            let current_branch_name = self.current_branch.read().clone();
            let branches = self.branches.read();
            branches
                .get(&current_branch_name)
                .map(|b| b.head)
                .unwrap_or_else(|| {
                    // Get root version
                    self.history.read().first().copied().unwrap()
                })
        });

        let branch = Branch::new(name.clone(), start_version);

        let mut branches = self.branches.write();
        if branches.contains_key(&name) {
            return Err(CollaborationError::InvalidState(format!(
                "Branch already exists: {}",
                name
            )));
        }

        branches.insert(name, branch);
        Ok(())
    }

    /// Switch to a different branch
    pub fn checkout(&self, branch_name: &str) -> Result<VersionId> {
        let branches = self.branches.read();
        let branch = branches.get(branch_name).ok_or_else(|| {
            CollaborationError::InvalidState(format!("Branch not found: {}", branch_name))
        })?;

        let head = branch.head;

        *self.current_branch.write() = branch_name.to_string();

        Ok(head)
    }

    /// Delete a branch
    pub fn delete_branch(&self, branch_name: &str) -> Result<()> {
        if branch_name == "main" {
            return Err(CollaborationError::InvalidState(
                "Cannot delete main branch".to_string(),
            ));
        }

        let current = self.current_branch.read().clone();
        if current == branch_name {
            return Err(CollaborationError::InvalidState(
                "Cannot delete current branch".to_string(),
            ));
        }

        let mut branches = self.branches.write();
        branches.remove(branch_name).ok_or_else(|| {
            CollaborationError::InvalidState(format!("Branch not found: {}", branch_name))
        })?;

        Ok(())
    }

    /// Merge a branch into current branch
    pub fn merge(
        &self,
        source_branch: &str,
        author: AuthorInfo,
        strategy: MergeStrategy,
    ) -> Result<MergeResult> {
        let current_branch_name = self.current_branch.read().clone();

        let (current_head, source_head) = {
            let branches = self.branches.read();

            let current = branches.get(&current_branch_name).ok_or_else(|| {
                CollaborationError::InvalidState(format!(
                    "Current branch not found: {}",
                    current_branch_name
                ))
            })?;

            let source = branches.get(source_branch).ok_or_else(|| {
                CollaborationError::InvalidState(format!("Source branch not found: {}", source_branch))
            })?;

            (current.head, source.head)
        };

        // Check if fast-forward is possible
        if self.is_ancestor(current_head, source_head)? {
            // Fast-forward merge
            let mut branches = self.branches.write();
            if let Some(branch) = branches.get_mut(&current_branch_name) {
                branch.update_head(source_head);
            }

            return Ok(MergeResult {
                merge_version: source_head,
                strategy,
                fast_forward: true,
                conflicts: 0,
                conflicting_operations: Vec::new(),
                merged_operations: Vec::new(),
            });
        }

        // Find common ancestor
        let common_ancestor = self.find_common_ancestor(current_head, source_head)?;

        // Get operations from common ancestor to both heads
        let current_ops = self.get_operations_since(common_ancestor, current_head)?;
        let source_ops = self.get_operations_since(common_ancestor, source_head)?;

        // Perform three-way merge
        let (merged_operations, conflicting_operations) =
            self.three_way_merge(&current_ops, &source_ops, strategy)?;

        // Create merge commit
        let merge_version_id = Uuid::new_v4();
        let merge_message = format!("Merge {} into {}", source_branch, current_branch_name);

        let merge_version = Version::new(
            merge_version_id,
            vec![current_head, source_head],
            author,
            merge_message,
            merged_operations.clone(),
        );

        // Update versions
        {
            let mut versions = self.versions.write();
            versions.insert(merge_version_id, merge_version);
        }

        // Update branch HEAD
        {
            let mut branches = self.branches.write();
            if let Some(branch) = branches.get_mut(&current_branch_name) {
                branch.update_head(merge_version_id);
            }
        }

        // Update history
        {
            let mut history = self.history.write();
            history.push(merge_version_id);
        }

        Ok(MergeResult {
            merge_version: merge_version_id,
            strategy,
            fast_forward: false,
            conflicts: conflicting_operations.len(),
            conflicting_operations,
            merged_operations,
        })
    }

    /// Find common ancestor of two versions
    fn find_common_ancestor(&self, v1: VersionId, v2: VersionId) -> Result<VersionId> {
        let versions = self.versions.read();

        let mut ancestors1 = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(v1);

        // Collect all ancestors of v1
        while let Some(current) = queue.pop_front() {
            if ancestors1.contains(&current) {
                continue;
            }
            ancestors1.insert(current);

            if let Some(version) = versions.get(&current) {
                for parent in &version.parents {
                    queue.push_back(*parent);
                }
            }
        }

        // Find first common ancestor with v2
        queue.clear();
        queue.push_back(v2);
        let mut visited = HashSet::new();

        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);

            if ancestors1.contains(&current) {
                return Ok(current);
            }

            if let Some(version) = versions.get(&current) {
                for parent in &version.parents {
                    queue.push_back(*parent);
                }
            }
        }

        Err(CollaborationError::InvalidState(
            "No common ancestor found".to_string(),
        ))
    }

    /// Check if v1 is an ancestor of v2
    fn is_ancestor(&self, v1: VersionId, v2: VersionId) -> Result<bool> {
        if v1 == v2 {
            return Ok(true);
        }

        let versions = self.versions.read();
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        queue.push_back(v2);

        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);

            if current == v1 {
                return Ok(true);
            }

            if let Some(version) = versions.get(&current) {
                for parent in &version.parents {
                    queue.push_back(*parent);
                }
            }
        }

        Ok(false)
    }

    /// Get all operations from ancestor to descendant
    fn get_operations_since(
        &self,
        ancestor: VersionId,
        descendant: VersionId,
    ) -> Result<Vec<CRDTOperation>> {
        let versions = self.versions.read();
        let mut operations = Vec::new();
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        queue.push_back(descendant);

        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) || current == ancestor {
                continue;
            }
            visited.insert(current);

            if let Some(version) = versions.get(&current) {
                operations.extend(version.operations.clone());

                for parent in &version.parents {
                    queue.push_back(*parent);
                }
            }
        }

        Ok(operations)
    }

    /// Perform three-way merge
    fn three_way_merge(
        &self,
        current_ops: &[CRDTOperation],
        source_ops: &[CRDTOperation],
        strategy: MergeStrategy,
    ) -> Result<(Vec<CRDTOperation>, Vec<(CRDTOperation, CRDTOperation)>)> {
        let mut merged = Vec::new();
        let mut conflicts = Vec::new();

        // Simple merge: combine all operations
        // In a real implementation, we'd detect conflicts and apply strategy
        match strategy {
            MergeStrategy::Ours => {
                merged.extend(current_ops.iter().cloned());
            }
            MergeStrategy::Theirs => {
                merged.extend(source_ops.iter().cloned());
            }
            _ => {
                merged.extend(current_ops.iter().cloned());
                merged.extend(source_ops.iter().cloned());
            }
        }

        Ok((merged, conflicts))
    }

    /// Create a tag
    pub fn create_tag(&self, tag_name: String, version_id: VersionId) -> Result<()> {
        let mut tags = self.tags.write();

        if tags.contains_key(&tag_name) {
            return Err(CollaborationError::InvalidState(format!(
                "Tag already exists: {}",
                tag_name
            )));
        }

        tags.insert(tag_name.clone(), version_id);

        // Add tag to version
        let mut versions = self.versions.write();
        if let Some(version) = versions.get_mut(&version_id) {
            version.add_tag(tag_name);
        }

        Ok(())
    }

    /// Get version by tag
    pub fn get_version_by_tag(&self, tag_name: &str) -> Option<VersionId> {
        self.tags.read().get(tag_name).copied()
    }

    /// Get version history
    pub fn get_history(&self, limit: Option<usize>) -> Vec<Version> {
        let history = self.history.read();
        let versions = self.versions.read();

        let iter = history.iter().rev();
        let limited_iter = if let Some(limit) = limit {
            Box::new(iter.take(limit)) as Box<dyn Iterator<Item = _>>
        } else {
            Box::new(iter) as Box<dyn Iterator<Item = _>>
        };

        limited_iter
            .filter_map(|id| versions.get(id).cloned())
            .collect()
    }

    /// Get all branches
    pub fn get_branches(&self) -> Vec<Branch> {
        self.branches.read().values().cloned().collect()
    }

    /// Get current branch
    pub fn get_current_branch(&self) -> BranchName {
        self.current_branch.read().clone()
    }

    /// Compute diff between two versions
    pub fn diff(&self, from: VersionId, to: VersionId) -> Result<VersionDiff> {
        let operations = self.get_operations_since(from, to)?;

        let mut diff = VersionDiff::empty(from, to);
        diff.added_operations = operations.clone();

        // Analyze operations
        for op in &operations {
            match op {
                CRDTOperation::AddEntity { .. } => {
                    diff.entities_added += 1;
                }
                CRDTOperation::UpdateProperty { entity_id, property, .. } => {
                    diff.entities_modified += 1;
                    diff.changed_properties
                        .entry(*entity_id)
                        .or_default()
                        .push(property.clone());
                }
                CRDTOperation::DeleteEntity { .. } => {
                    diff.entities_deleted += 1;
                }
                _ => {}
            }
        }

        Ok(diff)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_control_basic() {
        let doc_id = Uuid::new_v4();
        let author = AuthorInfo::new(
            Uuid::new_v4(),
            "Test User".to_string(),
            "test@example.com".to_string(),
        );

        let vc = VersionControl::new(doc_id, author.clone());

        // Create a commit
        let v1 = vc
            .commit(author.clone(), "First commit".to_string(), vec![])
            .unwrap();

        let history = vc.get_history(None);
        assert_eq!(history.len(), 2); // Root + first commit
    }

    #[test]
    fn test_branching() {
        let doc_id = Uuid::new_v4();
        let author = AuthorInfo::new(
            Uuid::new_v4(),
            "Test User".to_string(),
            "test@example.com".to_string(),
        );

        let vc = VersionControl::new(doc_id, author.clone());

        // Create a branch
        vc.create_branch("feature".to_string(), None).unwrap();

        // Switch to branch
        vc.checkout("feature").unwrap();

        assert_eq!(vc.get_current_branch(), "feature");
    }
}
