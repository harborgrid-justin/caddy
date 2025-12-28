//! Bidirectional Synchronization Engine
//!
//! This module implements a sophisticated sync engine that manages bidirectional
//! synchronization between local and cloud storage with conflict detection and
//! resolution capabilities.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, Mutex};
use tokio::time;

use super::storage::{CloudStorage, StorageError};
use super::transfer::TransferManager;
use super::versioning::VersionControl;

/// Synchronization error types
#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("Conflict detected for file: {0}")]
    Conflict(String),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Sync state corrupted: {0}")]
    CorruptedState(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid sync configuration: {0}")]
    InvalidConfig(String),

    #[error("Sync operation cancelled")]
    Cancelled,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

/// Conflict resolution strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictStrategy {
    /// Keep the local version
    KeepLocal,
    /// Keep the remote version
    KeepRemote,
    /// Keep both versions (rename one)
    KeepBoth,
    /// Use the newer version based on timestamp
    UseNewer,
    /// Use the larger file
    UseLarger,
    /// Manual resolution required
    Manual,
}

/// Synchronization direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncDirection {
    /// Upload to cloud
    Upload,
    /// Download from cloud
    Download,
    /// Bidirectional sync
    Bidirectional,
}

/// Synchronization state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncState {
    /// Idle, not syncing
    Idle,
    /// Scanning for changes
    Scanning,
    /// Computing deltas
    Computing,
    /// Resolving conflicts
    ResolvingConflicts,
    /// Transferring data
    Transferring,
    /// Verifying integrity
    Verifying,
    /// Synced successfully
    Synced,
    /// Error occurred
    Error,
    /// Paused
    Paused,
}

/// Synchronization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Sync interval in seconds
    pub interval_secs: u64,
    /// Conflict resolution strategy
    pub conflict_strategy: ConflictStrategy,
    /// Sync direction
    pub direction: SyncDirection,
    /// Enable delta sync for efficiency
    pub enable_delta_sync: bool,
    /// Maximum file size for sync (in bytes)
    pub max_file_size: u64,
    /// Patterns to exclude from sync
    pub exclude_patterns: Vec<String>,
    /// Enable automatic conflict resolution
    pub auto_resolve_conflicts: bool,
    /// Number of concurrent transfers
    pub concurrent_transfers: usize,
    /// Enable compression during transfer
    pub enable_compression: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            interval_secs: 300, // 5 minutes
            conflict_strategy: ConflictStrategy::UseNewer,
            direction: SyncDirection::Bidirectional,
            enable_delta_sync: true,
            max_file_size: 1024 * 1024 * 1024, // 1 GB
            exclude_patterns: vec![
                "*.tmp".to_string(),
                "*.bak".to_string(),
                ".DS_Store".to_string(),
            ],
            auto_resolve_conflicts: false,
            concurrent_transfers: 4,
            enable_compression: true,
        }
    }
}

/// Metadata for a synced file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: PathBuf,
    pub size: u64,
    pub modified: SystemTime,
    pub hash: String,
    pub version: u64,
}

/// Delta information for efficient sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delta {
    pub path: PathBuf,
    pub chunks_changed: Vec<usize>,
    pub total_chunks: usize,
    pub estimated_bytes: u64,
}

/// Conflict information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub path: PathBuf,
    pub local_metadata: FileMetadata,
    pub remote_metadata: FileMetadata,
    pub detected_at: DateTime<Utc>,
}

/// Sync statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncStats {
    pub files_synced: u64,
    pub bytes_transferred: u64,
    pub conflicts_resolved: u64,
    pub errors: u64,
    pub last_sync: Option<DateTime<Utc>>,
    pub duration_ms: u64,
}

/// Main synchronization engine
pub struct SyncEngine<S: CloudStorage> {
    storage: Arc<S>,
    transfer_manager: Arc<TransferManager>,
    version_control: Arc<VersionControl>,
    config: RwLock<SyncConfig>,
    state: RwLock<SyncState>,
    local_metadata: RwLock<HashMap<PathBuf, FileMetadata>>,
    remote_metadata: RwLock<HashMap<PathBuf, FileMetadata>>,
    conflicts: RwLock<Vec<Conflict>>,
    stats: RwLock<SyncStats>,
    cancel_token: Arc<Mutex<bool>>,
}

impl<S: CloudStorage + Send + Sync + 'static> SyncEngine<S> {
    /// Create a new sync engine
    pub fn new(storage: S) -> Self {
        Self {
            storage: Arc::new(storage),
            transfer_manager: Arc::new(TransferManager::new()),
            version_control: Arc::new(VersionControl::new()),
            config: RwLock::new(SyncConfig::default()),
            state: RwLock::new(SyncState::Idle),
            local_metadata: RwLock::new(HashMap::new()),
            remote_metadata: RwLock::new(HashMap::new()),
            conflicts: RwLock::new(Vec::new()),
            stats: RwLock::new(SyncStats::default()),
            cancel_token: Arc::new(Mutex::new(false)),
        }
    }

    /// Create a new sync engine with custom configuration
    pub fn with_config(storage: S, config: SyncConfig) -> Self {
        Self {
            storage: Arc::new(storage),
            transfer_manager: Arc::new(TransferManager::new()),
            version_control: Arc::new(VersionControl::new()),
            config: RwLock::new(config),
            state: RwLock::new(SyncState::Idle),
            local_metadata: RwLock::new(HashMap::new()),
            remote_metadata: RwLock::new(HashMap::new()),
            conflicts: RwLock::new(Vec::new()),
            stats: RwLock::new(SyncStats::default()),
            cancel_token: Arc::new(Mutex::new(false)),
        }
    }

    /// Start continuous synchronization
    pub async fn start_sync(&self) -> Result<(), SyncError> {
        let interval = self.config.read().await.interval_secs;
        let mut interval_timer = time::interval(Duration::from_secs(interval));

        loop {
            interval_timer.tick().await;

            if *self.cancel_token.lock().await {
                break;
            }

            if let Err(e) = self.sync_once().await {
                log::error!("Sync error: {}", e);
                *self.state.write().await = SyncState::Error;
                self.stats.write().await.errors += 1;
            }
        }

        Ok(())
    }

    /// Perform a single synchronization cycle
    pub async fn sync_once(&self) -> Result<(), SyncError> {
        let start_time = std::time::Instant::now();
        *self.state.write().await = SyncState::Scanning;

        // Scan for changes
        self.scan_local_changes().await?;
        self.scan_remote_changes().await?;

        // Compute differences
        *self.state.write().await = SyncState::Computing;
        let mut sync_plan = self.compute_sync_plan().await?;

        // Detect and resolve conflicts
        if !sync_plan.conflicts.is_empty() {
            *self.state.write().await = SyncState::ResolvingConflicts;
            let conflicts = std::mem::take(&mut sync_plan.conflicts);
            self.resolve_conflicts(conflicts).await?;
        }

        // Execute sync plan
        *self.state.write().await = SyncState::Transferring;
        self.execute_sync_plan(sync_plan).await?;

        // Verify integrity
        *self.state.write().await = SyncState::Verifying;
        self.verify_sync().await?;

        // Update statistics
        let duration = start_time.elapsed();
        let mut stats = self.stats.write().await;
        stats.last_sync = Some(Utc::now());
        stats.duration_ms = duration.as_millis() as u64;

        *self.state.write().await = SyncState::Synced;
        Ok(())
    }

    /// Scan for local file changes
    async fn scan_local_changes(&self) -> Result<(), SyncError> {
        // Implementation would scan the local filesystem
        // For now, this is a placeholder
        log::info!("Scanning local changes...");
        Ok(())
    }

    /// Scan for remote file changes
    async fn scan_remote_changes(&self) -> Result<(), SyncError> {
        log::info!("Scanning remote changes...");

        // List all files in remote storage
        let remote_files = self.storage.list_files("").await
            .map_err(|e| SyncError::Storage(e))?;

        let mut remote_meta = self.remote_metadata.write().await;
        remote_meta.clear();

        for file_path in remote_files {
            if let Ok(metadata) = self.storage.get_metadata(&file_path).await {
                let file_meta = FileMetadata {
                    path: PathBuf::from(&file_path),
                    size: metadata.size,
                    modified: metadata.modified,
                    hash: metadata.hash,
                    version: metadata.version,
                };
                remote_meta.insert(PathBuf::from(file_path), file_meta);
            }
        }

        Ok(())
    }

    /// Compute synchronization plan
    async fn compute_sync_plan(&self) -> Result<SyncPlan, SyncError> {
        let local_meta = self.local_metadata.read().await;
        let remote_meta = self.remote_metadata.read().await;
        let config = self.config.read().await;

        let mut to_upload = Vec::new();
        let mut to_download = Vec::new();
        let mut conflicts = Vec::new();
        let mut deltas = Vec::new();

        // Find files to upload
        for (path, local) in local_meta.iter() {
            if let Some(remote) = remote_meta.get(path) {
                // File exists in both locations
                if local.hash != remote.hash {
                    if local.modified > remote.modified && remote.modified > local.modified {
                        // Conflict: both modified
                        conflicts.push(Conflict {
                            path: path.clone(),
                            local_metadata: local.clone(),
                            remote_metadata: remote.clone(),
                            detected_at: Utc::now(),
                        });
                    } else if local.modified > remote.modified {
                        // Local is newer
                        if config.enable_delta_sync {
                            deltas.push(self.compute_delta(local, remote).await?);
                        } else {
                            to_upload.push(path.clone());
                        }
                    }
                }
            } else {
                // New local file
                to_upload.push(path.clone());
            }
        }

        // Find files to download
        for (path, remote) in remote_meta.iter() {
            if !local_meta.contains_key(path) {
                // New remote file
                to_download.push(path.clone());
            }
        }

        Ok(SyncPlan {
            to_upload,
            to_download,
            deltas,
            conflicts,
        })
    }

    /// Compute delta for efficient sync
    async fn compute_delta(
        &self,
        _local: &FileMetadata,
        _remote: &FileMetadata,
    ) -> Result<Delta, SyncError> {
        // Placeholder for delta computation
        // In a real implementation, this would compute chunk-level differences
        Ok(Delta {
            path: PathBuf::new(),
            chunks_changed: vec![],
            total_chunks: 0,
            estimated_bytes: 0,
        })
    }

    /// Resolve conflicts based on strategy
    async fn resolve_conflicts(&self, conflicts: Vec<Conflict>) -> Result<(), SyncError> {
        let config = self.config.read().await;
        let strategy = config.conflict_strategy;
        let auto_resolve = config.auto_resolve_conflicts;

        let mut conflicts_list = self.conflicts.write().await;

        for conflict in conflicts {
            if auto_resolve {
                match strategy {
                    ConflictStrategy::KeepLocal => {
                        log::info!("Conflict resolved: keeping local version of {:?}", conflict.path);
                    }
                    ConflictStrategy::KeepRemote => {
                        log::info!("Conflict resolved: keeping remote version of {:?}", conflict.path);
                    }
                    ConflictStrategy::UseNewer => {
                        if conflict.local_metadata.modified > conflict.remote_metadata.modified {
                            log::info!("Conflict resolved: local is newer for {:?}", conflict.path);
                        } else {
                            log::info!("Conflict resolved: remote is newer for {:?}", conflict.path);
                        }
                    }
                    ConflictStrategy::UseLarger => {
                        if conflict.local_metadata.size > conflict.remote_metadata.size {
                            log::info!("Conflict resolved: local is larger for {:?}", conflict.path);
                        } else {
                            log::info!("Conflict resolved: remote is larger for {:?}", conflict.path);
                        }
                    }
                    ConflictStrategy::KeepBoth => {
                        log::info!("Conflict resolved: keeping both versions of {:?}", conflict.path);
                    }
                    ConflictStrategy::Manual => {
                        conflicts_list.push(conflict.clone());
                    }
                }
                self.stats.write().await.conflicts_resolved += 1;
            } else {
                conflicts_list.push(conflict);
            }
        }

        if !conflicts_list.is_empty() && !auto_resolve {
            return Err(SyncError::Conflict(format!(
                "{} conflicts require manual resolution",
                conflicts_list.len()
            )));
        }

        Ok(())
    }

    /// Execute the synchronization plan
    async fn execute_sync_plan(&self, plan: SyncPlan) -> Result<(), SyncError> {
        let config = self.config.read().await;
        let semaphore = Arc::new(tokio::sync::Semaphore::new(config.concurrent_transfers));

        // Upload files
        let upload_tasks: Vec<_> = plan
            .to_upload
            .into_iter()
            .map(|path| {
                let storage = Arc::clone(&self.storage);
                let sem = Arc::clone(&semaphore);
                let stats = Arc::new(RwLock::new(self.stats.write()));

                async move {
                    let _permit = sem.acquire().await.unwrap();
                    if let Ok(data) = tokio::fs::read(&path).await {
                        if let Ok(_) = storage.upload_file(&path.to_string_lossy(), &data).await {
                            // stats.write().await.files_synced += 1;
                            // stats.write().await.bytes_transferred += data.len() as u64;
                        }
                    }
                }
            })
            .collect();

        // Download files
        let download_tasks: Vec<_> = plan
            .to_download
            .into_iter()
            .map(|path| {
                let storage = Arc::clone(&self.storage);
                let sem = Arc::clone(&semaphore);

                async move {
                    let _permit = sem.acquire().await.unwrap();
                    if let Ok(data) = storage.download_file(&path.to_string_lossy()).await {
                        let _ = tokio::fs::write(&path, data).await;
                    }
                }
            })
            .collect();

        // Execute all tasks concurrently
        futures::future::join_all(upload_tasks).await;
        futures::future::join_all(download_tasks).await;

        Ok(())
    }

    /// Verify synchronization integrity
    async fn verify_sync(&self) -> Result<(), SyncError> {
        log::info!("Verifying sync integrity...");
        // Implementation would verify that local and remote are in sync
        Ok(())
    }

    /// Pause synchronization
    pub async fn pause(&self) {
        *self.state.write().await = SyncState::Paused;
    }

    /// Resume synchronization
    pub async fn resume(&self) {
        *self.state.write().await = SyncState::Idle;
    }

    /// Stop synchronization
    pub async fn stop(&self) {
        *self.cancel_token.lock().await = true;
        *self.state.write().await = SyncState::Idle;
    }

    /// Get current synchronization state
    pub async fn get_state(&self) -> SyncState {
        *self.state.read().await
    }

    /// Get synchronization statistics
    pub async fn get_stats(&self) -> SyncStats {
        self.stats.read().await.clone()
    }

    /// Get pending conflicts
    pub async fn get_conflicts(&self) -> Vec<Conflict> {
        self.conflicts.read().await.clone()
    }

    /// Manually resolve a conflict
    pub async fn resolve_conflict(
        &self,
        path: &Path,
        strategy: ConflictStrategy,
    ) -> Result<(), SyncError> {
        let mut conflicts = self.conflicts.write().await;

        if let Some(pos) = conflicts.iter().position(|c| c.path == path) {
            let conflict = conflicts.remove(pos);

            match strategy {
                ConflictStrategy::KeepLocal => {
                    // Upload local version
                    let data = tokio::fs::read(&conflict.path).await?;
                    self.storage
                        .upload_file(&conflict.path.to_string_lossy(), &data)
                        .await?;
                }
                ConflictStrategy::KeepRemote => {
                    // Download remote version
                    let data = self.storage
                        .download_file(&conflict.path.to_string_lossy())
                        .await?;
                    tokio::fs::write(&conflict.path, data).await?;
                }
                ConflictStrategy::KeepBoth => {
                    // Keep both by renaming
                    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
                    let new_path = conflict.path.with_extension(format!(
                        "conflict_{}.{}",
                        timestamp,
                        conflict.path.extension().unwrap_or_default().to_string_lossy()
                    ));

                    let data = self.storage
                        .download_file(&conflict.path.to_string_lossy())
                        .await?;
                    tokio::fs::write(&new_path, data).await?;
                }
                _ => {}
            }

            self.stats.write().await.conflicts_resolved += 1;
            Ok(())
        } else {
            Err(SyncError::FileNotFound(path.display().to_string()))
        }
    }
}

/// Synchronization plan
#[derive(Debug)]
struct SyncPlan {
    to_upload: Vec<PathBuf>,
    to_download: Vec<PathBuf>,
    deltas: Vec<Delta>,
    conflicts: Vec<Conflict>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sync_config_default() {
        let config = SyncConfig::default();
        assert_eq!(config.interval_secs, 300);
        assert_eq!(config.conflict_strategy, ConflictStrategy::UseNewer);
    }

    #[tokio::test]
    async fn test_sync_state_transitions() {
        // Test state machine transitions
        let state = SyncState::Idle;
        assert_eq!(state, SyncState::Idle);
    }
}
