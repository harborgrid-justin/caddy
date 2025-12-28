//! Document Version Control System
//!
//! This module provides comprehensive version control capabilities for CAD documents
//! including version history, branching, merging, and rollback.

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// Version control error types
#[derive(Debug, thiserror::Error)]
pub enum VersionError {
    #[error("Version not found: {0}")]
    VersionNotFound(String),

    #[error("Branch not found: {0}")]
    BranchNotFound(String),

    #[error("Merge conflict: {0}")]
    MergeConflict(String),

    #[error("Invalid version: {0}")]
    InvalidVersion(String),

    #[error("Detached HEAD state")]
    DetachedHead,

    #[error("No changes to commit")]
    NoChanges,

    #[error("Rollback failed: {0}")]
    RollbackFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

/// Version metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    /// Unique version ID
    pub id: String,
    /// Parent version ID(s) - multiple for merge commits
    pub parent_ids: Vec<String>,
    /// Branch this version belongs to
    pub branch: String,
    /// Commit message
    pub message: String,
    /// Author information
    pub author: String,
    /// Email of the author
    pub email: String,
    /// Timestamp when version was created
    pub timestamp: DateTime<Utc>,
    /// File changes in this version
    pub changes: Vec<FileChange>,
    /// Total size of changes
    pub changes_size: u64,
    /// Tags associated with this version
    pub tags: Vec<String>,
}

/// File change information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    /// File path
    pub path: PathBuf,
    /// Type of change
    pub change_type: ChangeType,
    /// Size before change
    pub size_before: Option<u64>,
    /// Size after change
    pub size_after: Option<u64>,
    /// Content hash
    pub hash: String,
}

/// Type of file change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// File was added
    Added,
    /// File was modified
    Modified,
    /// File was deleted
    Deleted,
    /// File was renamed
    Renamed,
}

/// Branch information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    /// Branch name
    pub name: String,
    /// Current HEAD version ID
    pub head: String,
    /// Branch creation timestamp
    pub created_at: DateTime<Utc>,
    /// Branch description
    pub description: String,
    /// Whether this is a protected branch
    pub protected: bool,
}

/// Merge strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeStrategy {
    /// Fast-forward merge if possible
    FastForward,
    /// Always create a merge commit
    NoFastForward,
    /// Recursive three-way merge
    Recursive,
    /// Ours - prefer our changes in conflicts
    Ours,
    /// Theirs - prefer their changes in conflicts
    Theirs,
}

/// Merge result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    /// Whether merge was successful
    pub success: bool,
    /// New version ID if merge succeeded
    pub version_id: Option<String>,
    /// Conflicts that need resolution
    pub conflicts: Vec<MergeConflict>,
    /// Files successfully merged
    pub merged_files: Vec<PathBuf>,
}

/// Merge conflict information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeConflict {
    /// File path with conflict
    pub path: PathBuf,
    /// Our version
    pub ours: FileChange,
    /// Their version
    pub theirs: FileChange,
    /// Common ancestor version
    pub base: Option<FileChange>,
}

/// Version difference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDiff {
    /// Version being compared from
    pub from_version: String,
    /// Version being compared to
    pub to_version: String,
    /// Files added
    pub added: Vec<PathBuf>,
    /// Files modified
    pub modified: Vec<PathBuf>,
    /// Files deleted
    pub deleted: Vec<PathBuf>,
    /// Files renamed
    pub renamed: Vec<(PathBuf, PathBuf)>,
    /// Total changes
    pub total_changes: usize,
}

/// Tag information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    /// Tag name
    pub name: String,
    /// Version this tag points to
    pub version_id: String,
    /// Tag message
    pub message: String,
    /// Tagger information
    pub tagger: String,
    /// Tag creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Version control system
pub struct VersionControl {
    /// All versions indexed by ID
    versions: RwLock<HashMap<String, Version>>,
    /// All branches indexed by name
    branches: RwLock<HashMap<String, Branch>>,
    /// Current branch name
    current_branch: RwLock<String>,
    /// All tags indexed by name
    tags: RwLock<HashMap<String, Tag>>,
    /// Default author information
    default_author: RwLock<(String, String)>,
}

impl VersionControl {
    /// Create a new version control system
    pub fn new() -> Self {
        let mut branches = HashMap::new();
        let initial_version_id = uuid::Uuid::new_v4().to_string();

        // Create initial version
        let initial_version = Version {
            id: initial_version_id.clone(),
            parent_ids: vec![],
            branch: "main".to_string(),
            message: "Initial commit".to_string(),
            author: "System".to_string(),
            email: "system@caddy".to_string(),
            timestamp: Utc::now(),
            changes: vec![],
            changes_size: 0,
            tags: vec![],
        };

        // Create main branch
        let main_branch = Branch {
            name: "main".to_string(),
            head: initial_version_id.clone(),
            created_at: Utc::now(),
            description: "Main branch".to_string(),
            protected: true,
        };

        branches.insert("main".to_string(), main_branch);

        let mut versions = HashMap::new();
        versions.insert(initial_version_id, initial_version);

        Self {
            versions: RwLock::new(versions),
            branches: RwLock::new(branches),
            current_branch: RwLock::new("main".to_string()),
            tags: RwLock::new(HashMap::new()),
            default_author: RwLock::new(("User".to_string(), "user@example.com".to_string())),
        }
    }

    /// Create a new version (commit)
    pub async fn commit(
        &self,
        message: &str,
        changes: Vec<FileChange>,
    ) -> Result<String, VersionError> {
        if changes.is_empty() {
            return Err(VersionError::NoChanges);
        }

        let current_branch = self.current_branch.read().await.clone();
        let branches = self.branches.read().await;
        let current_head = branches
            .get(&current_branch)
            .ok_or_else(|| VersionError::BranchNotFound(current_branch.clone()))?
            .head
            .clone();
        drop(branches);

        let (author, email) = self.default_author.read().await.clone();
        let version_id = uuid::Uuid::new_v4().to_string();

        let changes_size: u64 = changes
            .iter()
            .filter_map(|c| c.size_after)
            .sum();

        let version = Version {
            id: version_id.clone(),
            parent_ids: vec![current_head],
            branch: current_branch.clone(),
            message: message.to_string(),
            author,
            email,
            timestamp: Utc::now(),
            changes,
            changes_size,
            tags: vec![],
        };

        // Store version
        self.versions.write().await.insert(version_id.clone(), version);

        // Update branch HEAD
        self.branches.write().await.get_mut(&current_branch).unwrap().head = version_id.clone();

        log::info!("Created version {} on branch {}", version_id, current_branch);
        Ok(version_id)
    }

    /// Create a new branch
    pub async fn create_branch(
        &self,
        name: &str,
        description: &str,
    ) -> Result<(), VersionError> {
        let current_branch = self.current_branch.read().await.clone();
        let branches = self.branches.read().await;
        let current_head = branches
            .get(&current_branch)
            .ok_or_else(|| VersionError::BranchNotFound(current_branch.clone()))?
            .head
            .clone();
        drop(branches);

        let branch = Branch {
            name: name.to_string(),
            head: current_head,
            created_at: Utc::now(),
            description: description.to_string(),
            protected: false,
        };

        self.branches.write().await.insert(name.to_string(), branch);

        log::info!("Created branch: {}", name);
        Ok(())
    }

    /// Switch to a different branch
    pub async fn checkout_branch(&self, name: &str) -> Result<(), VersionError> {
        let branches = self.branches.read().await;
        if !branches.contains_key(name) {
            return Err(VersionError::BranchNotFound(name.to_string()));
        }
        drop(branches);

        *self.current_branch.write().await = name.to_string();

        log::info!("Switched to branch: {}", name);
        Ok(())
    }

    /// Delete a branch
    pub async fn delete_branch(&self, name: &str) -> Result<(), VersionError> {
        if name == "main" {
            return Err(VersionError::Other("Cannot delete main branch".to_string()));
        }

        let current_branch = self.current_branch.read().await.clone();
        if name == current_branch {
            return Err(VersionError::Other(
                "Cannot delete current branch".to_string(),
            ));
        }

        let mut branches = self.branches.write().await;
        let branch = branches
            .get(name)
            .ok_or_else(|| VersionError::BranchNotFound(name.to_string()))?;

        if branch.protected {
            return Err(VersionError::Other(
                "Cannot delete protected branch".to_string(),
            ));
        }

        branches.remove(name);

        log::info!("Deleted branch: {}", name);
        Ok(())
    }

    /// Merge a branch into the current branch
    pub async fn merge(
        &self,
        source_branch: &str,
        strategy: MergeStrategy,
    ) -> Result<MergeResult, VersionError> {
        let current_branch = self.current_branch.read().await.clone();

        if current_branch == source_branch {
            return Err(VersionError::Other(
                "Cannot merge branch into itself".to_string(),
            ));
        }

        let branches = self.branches.read().await;
        let current_head = branches
            .get(&current_branch)
            .ok_or_else(|| VersionError::BranchNotFound(current_branch.clone()))?
            .head
            .clone();
        let source_head = branches
            .get(source_branch)
            .ok_or_else(|| VersionError::BranchNotFound(source_branch.to_string()))?
            .head
            .clone();
        drop(branches);

        // Find common ancestor
        let common_ancestor = self.find_common_ancestor(&current_head, &source_head).await?;

        // Check if fast-forward is possible
        if common_ancestor == current_head && strategy == MergeStrategy::FastForward {
            // Fast-forward merge
            self.branches.write().await.get_mut(&current_branch).unwrap().head = source_head.clone();

            return Ok(MergeResult {
                success: true,
                version_id: Some(source_head),
                conflicts: vec![],
                merged_files: vec![],
            });
        }

        // Perform three-way merge
        let conflicts = self.detect_merge_conflicts(&common_ancestor, &current_head, &source_head).await?;

        if !conflicts.is_empty() && strategy != MergeStrategy::Ours && strategy != MergeStrategy::Theirs {
            return Ok(MergeResult {
                success: false,
                version_id: None,
                conflicts,
                merged_files: vec![],
            });
        }

        // Create merge commit
        let (author, email) = self.default_author.read().await.clone();
        let version_id = uuid::Uuid::new_v4().to_string();

        let version = Version {
            id: version_id.clone(),
            parent_ids: vec![current_head, source_head],
            branch: current_branch.clone(),
            message: format!("Merge branch '{}' into '{}'", source_branch, current_branch),
            author,
            email,
            timestamp: Utc::now(),
            changes: vec![],
            changes_size: 0,
            tags: vec![],
        };

        self.versions.write().await.insert(version_id.clone(), version);
        self.branches.write().await.get_mut(&current_branch).unwrap().head = version_id.clone();

        log::info!("Merged branch {} into {}", source_branch, current_branch);

        Ok(MergeResult {
            success: true,
            version_id: Some(version_id),
            conflicts: vec![],
            merged_files: vec![],
        })
    }

    /// Find common ancestor of two versions
    async fn find_common_ancestor(
        &self,
        version1: &str,
        version2: &str,
    ) -> Result<String, VersionError> {
        let versions = self.versions.read().await;

        // Build ancestry sets for both versions
        let mut ancestors1 = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(version1.to_string());

        while let Some(vid) = queue.pop_front() {
            if !ancestors1.insert(vid.clone()) {
                continue;
            }

            if let Some(version) = versions.get(&vid) {
                for parent_id in &version.parent_ids {
                    queue.push_back(parent_id.clone());
                }
            }
        }

        // Find first common ancestor in version2's ancestry
        let mut queue = VecDeque::new();
        queue.push_back(version2.to_string());
        let mut visited = HashSet::new();

        while let Some(vid) = queue.pop_front() {
            if !visited.insert(vid.clone()) {
                continue;
            }

            if ancestors1.contains(&vid) {
                return Ok(vid);
            }

            if let Some(version) = versions.get(&vid) {
                for parent_id in &version.parent_ids {
                    queue.push_back(parent_id.clone());
                }
            }
        }

        Err(VersionError::Other("No common ancestor found".to_string()))
    }

    /// Detect merge conflicts
    async fn detect_merge_conflicts(
        &self,
        _base: &str,
        _ours: &str,
        _theirs: &str,
    ) -> Result<Vec<MergeConflict>, VersionError> {
        // Simplified conflict detection
        // In production, this would analyze file changes from all three versions
        Ok(vec![])
    }

    /// Rollback to a specific version
    pub async fn rollback(&self, version_id: &str) -> Result<(), VersionError> {
        let versions = self.versions.read().await;
        if !versions.contains_key(version_id) {
            return Err(VersionError::VersionNotFound(version_id.to_string()));
        }
        drop(versions);

        let current_branch = self.current_branch.read().await.clone();
        self.branches.write().await.get_mut(&current_branch).unwrap().head = version_id.to_string();

        log::info!("Rolled back to version: {}", version_id);
        Ok(())
    }

    /// Get version history for current branch
    pub async fn get_history(&self, limit: Option<usize>) -> Result<Vec<Version>, VersionError> {
        let current_branch = self.current_branch.read().await.clone();
        let branches = self.branches.read().await;
        let mut current_head = branches
            .get(&current_branch)
            .ok_or_else(|| VersionError::BranchNotFound(current_branch.clone()))?
            .head
            .clone();
        drop(branches);

        let versions = self.versions.read().await;
        let mut history = Vec::new();

        while let Some(version) = versions.get(&current_head) {
            history.push(version.clone());

            if let Some(limit) = limit {
                if history.len() >= limit {
                    break;
                }
            }

            // Follow first parent
            if version.parent_ids.is_empty() {
                break;
            }
            current_head = version.parent_ids[0].clone();
        }

        Ok(history)
    }

    /// Compare two versions
    pub async fn diff(
        &self,
        from_version: &str,
        to_version: &str,
    ) -> Result<VersionDiff, VersionError> {
        let versions = self.versions.read().await;

        let from = versions
            .get(from_version)
            .ok_or_else(|| VersionError::VersionNotFound(from_version.to_string()))?;
        let to = versions
            .get(to_version)
            .ok_or_else(|| VersionError::VersionNotFound(to_version.to_string()))?;

        let mut added = Vec::new();
        let mut modified = Vec::new();
        let mut deleted = Vec::new();
        let mut renamed = Vec::new();

        // Build file maps
        let from_files: HashMap<_, _> = from
            .changes
            .iter()
            .map(|c| (c.path.clone(), c))
            .collect();
        let to_files: HashMap<_, _> = to
            .changes
            .iter()
            .map(|c| (c.path.clone(), c))
            .collect();

        // Find added, modified, and renamed files
        for (path, to_change) in &to_files {
            if let Some(from_change) = from_files.get(path) {
                if from_change.hash != to_change.hash {
                    modified.push(path.clone());
                }
            } else {
                added.push(path.clone());
            }
        }

        // Find deleted files
        for path in from_files.keys() {
            if !to_files.contains_key(path) {
                deleted.push(path.clone());
            }
        }

        let total_changes = added.len() + modified.len() + deleted.len() + renamed.len();

        Ok(VersionDiff {
            from_version: from_version.to_string(),
            to_version: to_version.to_string(),
            added,
            modified,
            deleted,
            renamed,
            total_changes,
        })
    }

    /// Create a tag
    pub async fn create_tag(
        &self,
        name: &str,
        version_id: &str,
        message: &str,
    ) -> Result<(), VersionError> {
        let versions = self.versions.read().await;
        if !versions.contains_key(version_id) {
            return Err(VersionError::VersionNotFound(version_id.to_string()));
        }
        drop(versions);

        let (tagger, _) = self.default_author.read().await.clone();

        let tag = Tag {
            name: name.to_string(),
            version_id: version_id.to_string(),
            message: message.to_string(),
            tagger,
            created_at: Utc::now(),
        };

        self.tags.write().await.insert(name.to_string(), tag);

        // Add tag to version
        self.versions.write().await.get_mut(version_id).unwrap().tags.push(name.to_string());

        log::info!("Created tag: {} -> {}", name, version_id);
        Ok(())
    }

    /// List all branches
    pub async fn list_branches(&self) -> Vec<Branch> {
        self.branches.read().await.values().cloned().collect()
    }

    /// List all tags
    pub async fn list_tags(&self) -> Vec<Tag> {
        self.tags.read().await.values().cloned().collect()
    }

    /// Get current branch name
    pub async fn current_branch(&self) -> String {
        self.current_branch.read().await.clone()
    }

    /// Set default author information
    pub async fn set_author(&self, name: &str, email: &str) {
        *self.default_author.write().await = (name.to_string(), email.to_string());
    }
}

impl Default for VersionControl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_version_control_init() {
        let vc = VersionControl::new();
        assert_eq!(vc.current_branch().await, "main");
    }

    #[tokio::test]
    async fn test_create_branch() {
        let vc = VersionControl::new();
        vc.create_branch("feature", "Feature branch").await.unwrap();
        let branches = vc.list_branches().await;
        assert_eq!(branches.len(), 2);
    }

    #[tokio::test]
    async fn test_commit() {
        let vc = VersionControl::new();
        let changes = vec![FileChange {
            path: PathBuf::from("test.txt"),
            change_type: ChangeType::Added,
            size_before: None,
            size_after: Some(100),
            hash: "abc123".to_string(),
        }];

        let version_id = vc.commit("Test commit", changes).await.unwrap();
        assert!(!version_id.is_empty());
    }
}
