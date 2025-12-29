//! # Document State Management
//!
//! Manages document versions, revisions, branching, and merging.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use thiserror::Error;
use uuid::Uuid;

use super::ot::{Operation, OTError};

/// Errors that can occur in document management
#[derive(Debug, Error)]
pub enum DocumentError {
    #[error("Revision not found: {0}")]
    RevisionNotFound(u64),
    #[error("Branch not found: {0}")]
    BranchNotFound(String),
    #[error("Invalid version: {0}")]
    InvalidVersion(u64),
    #[error("Merge conflict detected")]
    MergeConflict,
    #[error("OT error: {0}")]
    OTError(#[from] OTError),
    #[error("Cannot merge: {0}")]
    MergeError(String),
}

/// A specific revision of a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revision {
    /// Unique revision ID
    pub id: Uuid,
    /// Version number
    pub version: u64,
    /// Parent revision ID
    pub parent: Option<Uuid>,
    /// Content at this revision
    pub content: String,
    /// Operations applied to reach this revision
    pub operations: Vec<Operation>,
    /// Author of this revision
    pub author: Uuid,
    /// Timestamp when revision was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Commit message/description
    pub message: String,
    /// Hash of the content for integrity
    pub content_hash: String,
}

impl Revision {
    /// Create a new revision
    pub fn new(
        version: u64,
        parent: Option<Uuid>,
        content: String,
        operations: Vec<Operation>,
        author: Uuid,
        message: String,
    ) -> Self {
        let content_hash = Self::hash_content(&content);
        Self {
            id: Uuid::new_v4(),
            version,
            parent,
            content,
            operations,
            author,
            timestamp: chrono::Utc::now(),
            message,
            content_hash,
        }
    }

    /// Calculate content hash
    fn hash_content(content: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Verify content integrity
    pub fn verify_integrity(&self) -> bool {
        self.content_hash == Self::hash_content(&self.content)
    }
}

/// A branch of document development
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    /// Branch name
    pub name: String,
    /// Current HEAD revision
    pub head: Uuid,
    /// Parent branch
    pub parent: Option<String>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Branch metadata
    pub metadata: HashMap<String, String>,
}

impl Branch {
    /// Create a new branch
    pub fn new(name: String, head: Uuid, parent: Option<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            name,
            head,
            parent,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    /// Update the HEAD to a new revision
    pub fn update_head(&mut self, revision_id: Uuid) {
        self.head = revision_id;
        self.updated_at = chrono::Utc::now();
    }
}

/// Conflict marker in document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictMarker {
    /// Position in document
    pub position: usize,
    /// Content from current branch
    pub current: String,
    /// Content from incoming branch
    pub incoming: String,
    /// Common ancestor content
    pub base: Option<String>,
    /// Conflict ID
    pub id: Uuid,
}

impl ConflictMarker {
    /// Create a new conflict marker
    pub fn new(position: usize, current: String, incoming: String, base: Option<String>) -> Self {
        Self {
            position,
            current,
            incoming,
            base,
            id: Uuid::new_v4(),
        }
    }

    /// Format conflict as text with markers
    pub fn format(&self) -> String {
        let mut result = String::new();
        result.push_str("<<<<<<< CURRENT\n");
        result.push_str(&self.current);
        result.push_str("\n=======\n");
        result.push_str(&self.incoming);
        result.push_str("\n>>>>>>> INCOMING\n");
        result
    }
}

/// Document state manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentState {
    /// Document ID
    pub id: Uuid,
    /// Document name
    pub name: String,
    /// All revisions
    revisions: BTreeMap<u64, Revision>,
    /// Branches
    branches: HashMap<String, Branch>,
    /// Current active branch
    current_branch: String,
    /// Current version
    current_version: u64,
    /// Conflict markers
    conflicts: Vec<ConflictMarker>,
}

impl DocumentState {
    /// Create a new document state
    pub fn new(name: String, initial_content: String, author: Uuid) -> Self {
        let mut state = Self {
            id: Uuid::new_v4(),
            name,
            revisions: BTreeMap::new(),
            branches: HashMap::new(),
            current_branch: "main".to_string(),
            current_version: 0,
            conflicts: Vec::new(),
        };

        // Create initial revision
        let revision = Revision::new(
            0,
            None,
            initial_content,
            Vec::new(),
            author,
            "Initial revision".to_string(),
        );

        state.revisions.insert(0, revision.clone());

        // Create main branch
        let branch = Branch::new("main".to_string(), revision.id, None);
        state.branches.insert("main".to_string(), branch);

        state
    }

    /// Get the current revision
    pub fn current_revision(&self) -> Result<&Revision, DocumentError> {
        self.revisions
            .get(&self.current_version)
            .ok_or(DocumentError::RevisionNotFound(self.current_version))
    }

    /// Get the current content
    pub fn current_content(&self) -> Result<String, DocumentError> {
        Ok(self.current_revision()?.content.clone())
    }

    /// Get the current version
    pub fn current_version(&self) -> u64 {
        self.current_version
    }

    /// Apply an operation and create a new revision
    pub fn apply_operation(
        &mut self,
        operation: Operation,
        author: Uuid,
        message: String,
    ) -> Result<u64, DocumentError> {
        let current_content = self.current_content()?;
        let new_content = operation.apply(&current_content)?;

        let new_version = self.current_version + 1;
        let parent_id = self.current_revision()?.id;

        let revision = Revision::new(
            new_version,
            Some(parent_id),
            new_content,
            vec![operation],
            author,
            message,
        );

        self.revisions.insert(new_version, revision.clone());
        self.current_version = new_version;

        // Update current branch HEAD
        if let Some(branch) = self.branches.get_mut(&self.current_branch) {
            branch.update_head(revision.id);
        }

        Ok(new_version)
    }

    /// Create a new branch from the current revision
    pub fn create_branch(&mut self, name: String) -> Result<(), DocumentError> {
        if self.branches.contains_key(&name) {
            return Err(DocumentError::BranchNotFound(format!(
                "Branch '{}' already exists",
                name
            )));
        }

        let current_head = self.current_revision()?.id;
        let branch = Branch::new(name.clone(), current_head, Some(self.current_branch.clone()));

        self.branches.insert(name, branch);
        Ok(())
    }

    /// Switch to a different branch
    pub fn checkout_branch(&mut self, name: &str) -> Result<(), DocumentError> {
        let branch = self
            .branches
            .get(name)
            .ok_or_else(|| DocumentError::BranchNotFound(name.to_string()))?;

        // Find the revision with the branch's HEAD
        let head_revision = self
            .revisions
            .values()
            .find(|r| r.id == branch.head)
            .ok_or_else(|| DocumentError::RevisionNotFound(0))?;

        self.current_branch = name.to_string();
        self.current_version = head_revision.version;

        Ok(())
    }

    /// Merge another branch into the current branch
    pub fn merge_branch(&mut self, source_branch: &str, author: Uuid) -> Result<u64, DocumentError> {
        let source = self
            .branches
            .get(source_branch)
            .ok_or_else(|| DocumentError::BranchNotFound(source_branch.to_string()))?
            .clone();

        let current = self
            .branches
            .get(&self.current_branch)
            .ok_or_else(|| DocumentError::BranchNotFound(self.current_branch.clone()))?
            .clone();

        // Get revisions
        let source_revision = self
            .revisions
            .values()
            .find(|r| r.id == source.head)
            .ok_or(DocumentError::RevisionNotFound(0))?
            .clone();

        let current_revision = self
            .revisions
            .values()
            .find(|r| r.id == current.head)
            .ok_or(DocumentError::RevisionNotFound(0))?
            .clone();

        // Simple merge: if no conflicts, concatenate changes
        let merged_content = self.merge_contents(
            &current_revision.content,
            &source_revision.content,
        )?;

        let new_version = self.current_version + 1;

        let revision = Revision::new(
            new_version,
            Some(current_revision.id),
            merged_content,
            Vec::new(),
            author,
            format!("Merge branch '{}' into '{}'", source_branch, self.current_branch),
        );

        self.revisions.insert(new_version, revision.clone());
        self.current_version = new_version;

        // Update current branch HEAD
        if let Some(branch) = self.branches.get_mut(&self.current_branch) {
            branch.update_head(revision.id);
        }

        Ok(new_version)
    }

    /// Merge two content strings
    fn merge_contents(&mut self, current: &str, incoming: &str) -> Result<String, DocumentError> {
        // Simple line-based merge
        let current_lines: Vec<&str> = current.lines().collect();
        let incoming_lines: Vec<&str> = incoming.lines().collect();

        let mut result: Vec<String> = Vec::new();
        let mut i = 0;
        let mut j = 0;

        while i < current_lines.len() || j < incoming_lines.len() {
            if i < current_lines.len() && j < incoming_lines.len() {
                if current_lines[i] == incoming_lines[j] {
                    result.push(current_lines[i].to_string());
                    i += 1;
                    j += 1;
                } else {
                    // Conflict detected
                    let conflict = ConflictMarker::new(
                        result.len(),
                        current_lines[i].to_string(),
                        incoming_lines[j].to_string(),
                        None,
                    );
                    self.conflicts.push(conflict.clone());
                    result.push(conflict.format());
                    i += 1;
                    j += 1;
                }
            } else if i < current_lines.len() {
                result.push(current_lines[i].to_string());
                i += 1;
            } else {
                result.push(incoming_lines[j].to_string());
                j += 1;
            }
        }

        Ok(result.join("\n"))
    }

    /// Get revision history
    pub fn get_history(&self, limit: Option<usize>) -> Vec<&Revision> {
        let mut revisions: Vec<&Revision> = self.revisions.values().collect();
        revisions.sort_by(|a, b| b.version.cmp(&a.version));

        if let Some(limit) = limit {
            revisions.truncate(limit);
        }

        revisions
    }

    /// Get a specific revision
    pub fn get_revision(&self, version: u64) -> Result<&Revision, DocumentError> {
        self.revisions
            .get(&version)
            .ok_or(DocumentError::RevisionNotFound(version))
    }

    /// Revert to a specific revision
    pub fn revert_to(&mut self, version: u64, author: Uuid) -> Result<u64, DocumentError> {
        let target_revision = self.get_revision(version)?;
        let content = target_revision.content.clone();

        let new_version = self.current_version + 1;
        let parent_id = self.current_revision()?.id;

        let revision = Revision::new(
            new_version,
            Some(parent_id),
            content,
            Vec::new(),
            author,
            format!("Revert to version {}", version),
        );

        self.revisions.insert(new_version, revision.clone());
        self.current_version = new_version;

        // Update current branch HEAD
        if let Some(branch) = self.branches.get_mut(&self.current_branch) {
            branch.update_head(revision.id);
        }

        Ok(new_version)
    }

    /// Get all branches
    pub fn get_branches(&self) -> Vec<&Branch> {
        self.branches.values().collect()
    }

    /// Get current branch name
    pub fn current_branch_name(&self) -> &str {
        &self.current_branch
    }

    /// Get conflict markers
    pub fn get_conflicts(&self) -> &[ConflictMarker] {
        &self.conflicts
    }

    /// Resolve a conflict
    pub fn resolve_conflict(&mut self, conflict_id: Uuid, resolution: String) -> Result<(), DocumentError> {
        if let Some(pos) = self.conflicts.iter().position(|c| c.id == conflict_id) {
            self.conflicts.remove(pos);
            Ok(())
        } else {
            Err(DocumentError::MergeConflict)
        }
    }

    /// Clear all conflicts
    pub fn clear_conflicts(&mut self) {
        self.conflicts.clear();
    }

    /// Get document statistics
    pub fn stats(&self) -> DocumentStats {
        DocumentStats {
            total_revisions: self.revisions.len(),
            total_branches: self.branches.len(),
            current_version: self.current_version,
            conflicts_count: self.conflicts.len(),
        }
    }
}

/// Document statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentStats {
    pub total_revisions: usize,
    pub total_branches: usize,
    pub current_version: u64,
    pub conflicts_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_document() {
        let author = Uuid::new_v4();
        let _doc = DocumentState::new(
            "test.cad".to_string(),
            "Initial content".to_string(),
            author,
        );

        assert_eq!(doc.current_version(), 0);
        assert_eq!(doc.current_content().unwrap(), "Initial content");
    }

    #[test]
    fn test_apply_operation() {
        let author = Uuid::new_v4();
        let mut doc = DocumentState::new(
            "test.cad".to_string(),
            "Hello".to_string(),
            author,
        );

        let op = Operation::insert(5, " World".to_string(), 1, author);
        let version = doc.apply_operation(op, author, "Add world".to_string()).unwrap();

        assert_eq!(version, 1);
        assert_eq!(doc.current_content().unwrap(), "Hello World");
    }

    #[test]
    fn test_create_branch() {
        let author = Uuid::new_v4();
        let mut doc = DocumentState::new(
            "test.cad".to_string(),
            "Initial".to_string(),
            author,
        );

        doc.create_branch("feature".to_string()).unwrap();
        assert_eq!(doc.get_branches().len(), 2);
    }

    #[test]
    fn test_checkout_branch() {
        let author = Uuid::new_v4();
        let mut doc = DocumentState::new(
            "test.cad".to_string(),
            "Initial".to_string(),
            author,
        );

        doc.create_branch("feature".to_string()).unwrap();
        doc.checkout_branch("feature").unwrap();

        assert_eq!(doc.current_branch_name(), "feature");
    }

    #[test]
    fn test_revision_integrity() {
        let author = Uuid::new_v4();
        let revision = Revision::new(
            0,
            None,
            "test content".to_string(),
            Vec::new(),
            author,
            "test".to_string(),
        );

        assert!(revision.verify_integrity());
    }

    #[test]
    fn test_history() {
        let author = Uuid::new_v4();
        let mut doc = DocumentState::new(
            "test.cad".to_string(),
            "Initial".to_string(),
            author,
        );

        let op1 = Operation::insert(7, " v1".to_string(), 1, author);
        doc.apply_operation(op1, author, "v1".to_string()).unwrap();

        let op2 = Operation::insert(10, " v2".to_string(), 2, author);
        doc.apply_operation(op2, author, "v2".to_string()).unwrap();

        let history = doc.get_history(None);
        assert_eq!(history.len(), 3); // Initial + 2 operations
    }
}
