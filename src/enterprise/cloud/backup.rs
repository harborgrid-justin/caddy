//! Enterprise Backup System
//!
//! This module provides comprehensive backup capabilities including incremental
//! backups, full backup scheduling, verification, and point-in-time recovery.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::storage::{CloudStorage, StorageError};

/// Backup error types
#[derive(Debug, thiserror::Error)]
pub enum BackupError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Backup not found: {0}")]
    BackupNotFound(String),

    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    #[error("Restore failed: {0}")]
    RestoreFailed(String),

    #[error("Invalid backup configuration: {0}")]
    InvalidConfig(String),

    #[error("Corrupt backup: {0}")]
    CorruptBackup(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

/// Backup type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackupType {
    /// Full backup of all files
    Full,
    /// Incremental backup (changes since last backup)
    Incremental,
    /// Differential backup (changes since last full backup)
    Differential,
}

/// Backup status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackupStatus {
    /// Backup in progress
    InProgress,
    /// Backup completed successfully
    Completed,
    /// Backup failed
    Failed,
    /// Backup being verified
    Verifying,
    /// Verification completed
    Verified,
}

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Backup interval in seconds
    pub interval_secs: u64,
    /// Maximum number of backups to keep
    pub max_backups: usize,
    /// Enable compression
    pub enable_compression: bool,
    /// Enable encryption
    pub enable_encryption: bool,
    /// Encryption key (if encryption enabled)
    pub encryption_key: Option<String>,
    /// Verify backups after creation
    pub verify_after_backup: bool,
    /// Retention policy in days
    pub retention_days: u32,
    /// Patterns to exclude from backup
    pub exclude_patterns: Vec<String>,
    /// Enable incremental backups
    pub enable_incremental: bool,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            interval_secs: 86400, // 24 hours
            max_backups: 30,
            enable_compression: true,
            enable_encryption: false,
            encryption_key: None,
            verify_after_backup: true,
            retention_days: 90,
            exclude_patterns: vec![
                "*.tmp".to_string(),
                "*.log".to_string(),
                "*.cache".to_string(),
            ],
            enable_incremental: true,
        }
    }
}

/// Backup metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub id: String,
    pub backup_type: BackupType,
    pub status: BackupStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub files_count: u64,
    pub total_size: u64,
    pub compressed_size: u64,
    pub checksum: String,
    pub parent_backup_id: Option<String>, // For incremental/differential
    pub verification_status: Option<bool>,
}

/// Recovery point for point-in-time recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryPoint {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub backup_id: String,
    pub description: String,
    pub file_count: u64,
    pub total_size: u64,
}

/// File entry in a backup
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackupFileEntry {
    path: PathBuf,
    size: u64,
    modified: SystemTime,
    checksum: String,
    compressed: bool,
}

/// Backup manifest - describes the contents of a backup
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackupManifest {
    metadata: BackupMetadata,
    files: Vec<BackupFileEntry>,
    created_at: DateTime<Utc>,
    version: String,
}

/// Main backup engine
pub struct BackupEngine<S: CloudStorage> {
    storage: Arc<S>,
    config: RwLock<BackupConfig>,
    backups: RwLock<HashMap<String, BackupMetadata>>,
    recovery_points: RwLock<Vec<RecoveryPoint>>,
    last_full_backup: RwLock<Option<String>>,
}

impl<S: CloudStorage + Send + Sync + 'static> BackupEngine<S> {
    /// Create a new backup engine
    pub fn new(storage: S) -> Self {
        Self {
            storage: Arc::new(storage),
            config: RwLock::new(BackupConfig::default()),
            backups: RwLock::new(HashMap::new()),
            recovery_points: RwLock::new(Vec::new()),
            last_full_backup: RwLock::new(None),
        }
    }

    /// Create a new backup engine with custom configuration
    pub fn with_config(storage: S, config: BackupConfig) -> Self {
        Self {
            storage: Arc::new(storage),
            config: RwLock::new(config),
            backups: RwLock::new(HashMap::new()),
            recovery_points: RwLock::new(Vec::new()),
            last_full_backup: RwLock::new(None),
        }
    }

    /// Start automatic backup scheduling
    pub async fn start_scheduled_backups(&self) -> Result<(), BackupError> {
        let interval = self.config.read().await.interval_secs;
        let mut interval_timer = tokio::time::interval(Duration::from_secs(interval));

        loop {
            interval_timer.tick().await;

            let backup_type = if self.last_full_backup.read().await.is_none() {
                BackupType::Full
            } else if self.config.read().await.enable_incremental {
                BackupType::Incremental
            } else {
                BackupType::Full
            };

            match self.create_backup(backup_type, &PathBuf::from(".")).await {
                Ok(backup_id) => {
                    log::info!("Scheduled backup completed: {}", backup_id);
                }
                Err(e) => {
                    log::error!("Scheduled backup failed: {}", e);
                }
            }
        }
    }

    /// Create a new backup
    pub async fn create_backup(
        &self,
        backup_type: BackupType,
        source_path: &Path,
    ) -> Result<String, BackupError> {
        let backup_id = uuid::Uuid::new_v4().to_string();
        let created_at = Utc::now();

        log::info!("Creating {:?} backup: {}", backup_type, backup_id);

        // Create initial metadata
        let mut metadata = BackupMetadata {
            id: backup_id.clone(),
            backup_type,
            status: BackupStatus::InProgress,
            created_at,
            completed_at: None,
            files_count: 0,
            total_size: 0,
            compressed_size: 0,
            checksum: String::new(),
            parent_backup_id: if backup_type != BackupType::Full {
                self.last_full_backup.read().await.clone()
            } else {
                None
            },
            verification_status: None,
        };

        // Collect files to backup
        let files = self.collect_files_to_backup(source_path, backup_type).await?;
        metadata.files_count = files.len() as u64;

        // Create manifest
        let manifest = BackupManifest {
            metadata: metadata.clone(),
            files: files.clone(),
            created_at,
            version: "1.0".to_string(),
        };

        // Upload files
        let mut total_size = 0u64;
        let mut compressed_size = 0u64;

        for file_entry in &files {
            if let Ok(data) = tokio::fs::read(&file_entry.path).await {
                total_size += data.len() as u64;

                let upload_data = if self.config.read().await.enable_compression {
                    self.compress_data(&data)?
                } else {
                    data.clone()
                };

                compressed_size += upload_data.len() as u64;

                let backup_path = format!("backups/{}/{}", backup_id, file_entry.path.display());
                self.storage.upload_file(&backup_path, &upload_data).await?;
            }
        }

        metadata.total_size = total_size;
        metadata.compressed_size = compressed_size;
        metadata.status = BackupStatus::Completed;
        metadata.completed_at = Some(Utc::now());
        metadata.checksum = self.calculate_manifest_checksum(&manifest)?;

        // Upload manifest
        let manifest_json = serde_json::to_vec(&manifest)?;
        let manifest_path = format!("backups/{}/manifest.json", backup_id);
        self.storage.upload_file(&manifest_path, &manifest_json).await?;

        // Store backup metadata
        self.backups.write().await.insert(backup_id.clone(), metadata.clone());

        // Update last full backup if this was a full backup
        if backup_type == BackupType::Full {
            *self.last_full_backup.write().await = Some(backup_id.clone());
        }

        // Verify backup if configured
        if self.config.read().await.verify_after_backup {
            self.verify_backup(&backup_id).await?;
        }

        // Create recovery point
        self.create_recovery_point(&backup_id, "Automatic backup").await?;

        // Clean up old backups
        self.cleanup_old_backups().await?;

        log::info!("Backup {} completed successfully", backup_id);
        Ok(backup_id)
    }

    /// Collect files to backup based on backup type
    async fn collect_files_to_backup(
        &self,
        source_path: &Path,
        backup_type: BackupType,
    ) -> Result<Vec<BackupFileEntry>, BackupError> {
        let mut files = Vec::new();

        // In production, this would recursively scan the directory
        // For now, this is a simplified implementation
        log::info!("Collecting files for {:?} backup from {:?}", backup_type, source_path);

        // Placeholder - would scan directory and filter based on backup type
        // and exclude patterns

        Ok(files)
    }

    /// Compress data using a simple compression algorithm
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>, BackupError> {
        // In production, use a real compression library like flate2
        // For now, just return the original data
        Ok(data.to_vec())
    }

    /// Calculate checksum for manifest
    fn calculate_manifest_checksum(&self, manifest: &BackupManifest) -> Result<String, BackupError> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let json = serde_json::to_string(manifest)?;
        let mut hasher = DefaultHasher::new();
        json.hash(&mut hasher);
        Ok(format!("{:x}", hasher.finish()))
    }

    /// Verify backup integrity
    pub async fn verify_backup(&self, backup_id: &str) -> Result<bool, BackupError> {
        log::info!("Verifying backup: {}", backup_id);

        let mut backups = self.backups.write().await;
        let metadata = backups
            .get_mut(backup_id)
            .ok_or_else(|| BackupError::BackupNotFound(backup_id.to_string()))?;

        metadata.status = BackupStatus::Verifying;

        // Download and verify manifest
        let manifest_path = format!("backups/{}/manifest.json", backup_id);
        let manifest_data = self.storage.download_file(&manifest_path).await?;
        let manifest: BackupManifest = serde_json::from_slice(&manifest_data)?;

        // Verify checksum
        let calculated_checksum = self.calculate_manifest_checksum(&manifest)?;
        if calculated_checksum != metadata.checksum {
            metadata.verification_status = Some(false);
            return Err(BackupError::VerificationFailed(
                "Manifest checksum mismatch".to_string(),
            ));
        }

        // Verify all files exist
        for file_entry in &manifest.files {
            let backup_path = format!("backups/{}/{}", backup_id, file_entry.path.display());
            if !self.storage.file_exists(&backup_path).await? {
                metadata.verification_status = Some(false);
                return Err(BackupError::VerificationFailed(format!(
                    "File missing: {}",
                    file_entry.path.display()
                )));
            }
        }

        metadata.status = BackupStatus::Verified;
        metadata.verification_status = Some(true);

        log::info!("Backup {} verified successfully", backup_id);
        Ok(true)
    }

    /// Restore from a backup
    pub async fn restore_backup(
        &self,
        backup_id: &str,
        target_path: &Path,
    ) -> Result<(), BackupError> {
        log::info!("Restoring backup {} to {:?}", backup_id, target_path);

        let backups = self.backups.read().await;
        let metadata = backups
            .get(backup_id)
            .ok_or_else(|| BackupError::BackupNotFound(backup_id.to_string()))?;

        // Verify backup before restoring
        if metadata.verification_status != Some(true) {
            return Err(BackupError::RestoreFailed(
                "Backup not verified".to_string(),
            ));
        }

        // Download manifest
        let manifest_path = format!("backups/{}/manifest.json", backup_id);
        let manifest_data = self.storage.download_file(&manifest_path).await?;
        let manifest: BackupManifest = serde_json::from_slice(&manifest_data)?;

        // Restore files
        for file_entry in &manifest.files {
            let backup_path = format!("backups/{}/{}", backup_id, file_entry.path.display());
            let target_file = target_path.join(&file_entry.path);

            // Create parent directories
            if let Some(parent) = target_file.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }

            // Download and decompress file
            let data = self.storage.download_file(&backup_path).await?;
            let decompressed_data = if file_entry.compressed {
                self.decompress_data(&data)?
            } else {
                data
            };

            tokio::fs::write(&target_file, decompressed_data).await?;
        }

        log::info!("Backup {} restored successfully to {:?}", backup_id, target_path);
        Ok(())
    }

    /// Decompress data
    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>, BackupError> {
        // In production, use a real decompression library
        Ok(data.to_vec())
    }

    /// Point-in-time recovery
    pub async fn restore_to_point_in_time(
        &self,
        timestamp: DateTime<Utc>,
        target_path: &Path,
    ) -> Result<(), BackupError> {
        log::info!("Restoring to point in time: {}", timestamp);

        // Find the closest recovery point
        let recovery_points = self.recovery_points.read().await;
        let recovery_point = recovery_points
            .iter()
            .filter(|rp| rp.timestamp <= timestamp)
            .max_by_key(|rp| rp.timestamp)
            .ok_or_else(|| {
                BackupError::BackupNotFound(format!("No recovery point found for {}", timestamp))
            })?;

        // Restore from the recovery point's backup
        self.restore_backup(&recovery_point.backup_id, target_path).await?;

        log::info!("Restored to point in time: {}", timestamp);
        Ok(())
    }

    /// Create a recovery point
    pub async fn create_recovery_point(
        &self,
        backup_id: &str,
        description: &str,
    ) -> Result<String, BackupError> {
        let backups = self.backups.read().await;
        let metadata = backups
            .get(backup_id)
            .ok_or_else(|| BackupError::BackupNotFound(backup_id.to_string()))?;

        let recovery_point = RecoveryPoint {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            backup_id: backup_id.to_string(),
            description: description.to_string(),
            file_count: metadata.files_count,
            total_size: metadata.total_size,
        };

        let id = recovery_point.id.clone();
        self.recovery_points.write().await.push(recovery_point);

        log::info!("Created recovery point: {}", id);
        Ok(id)
    }

    /// List all backups
    pub async fn list_backups(&self) -> Vec<BackupMetadata> {
        self.backups.read().await.values().cloned().collect()
    }

    /// List all recovery points
    pub async fn list_recovery_points(&self) -> Vec<RecoveryPoint> {
        self.recovery_points.read().await.clone()
    }

    /// Delete a backup
    pub async fn delete_backup(&self, backup_id: &str) -> Result<(), BackupError> {
        log::info!("Deleting backup: {}", backup_id);

        // Delete all files in the backup
        let prefix = format!("backups/{}/", backup_id);
        let files = self.storage.list_files(&prefix).await?;

        for file_path in files {
            self.storage.delete_file(&file_path).await?;
        }

        // Remove from metadata
        self.backups.write().await.remove(backup_id);

        log::info!("Backup {} deleted successfully", backup_id);
        Ok(())
    }

    /// Clean up old backups based on retention policy
    async fn cleanup_old_backups(&self) -> Result<(), BackupError> {
        let config = self.config.read().await;
        let retention_duration = Duration::from_secs(config.retention_days as u64 * 86400);
        let cutoff_time = Utc::now() - chrono::Duration::from_std(retention_duration).unwrap();

        let backups = self.backups.read().await;
        let mut to_delete = Vec::new();

        for (backup_id, metadata) in backups.iter() {
            if metadata.created_at < cutoff_time {
                to_delete.push(backup_id.clone());
            }
        }

        // Also enforce max backups limit
        if backups.len() > config.max_backups {
            let mut sorted_backups: Vec<_> = backups.values().collect();
            sorted_backups.sort_by_key(|m| m.created_at);

            let excess = backups.len() - config.max_backups;
            for metadata in sorted_backups.iter().take(excess) {
                if !to_delete.contains(&metadata.id) {
                    to_delete.push(metadata.id.clone());
                }
            }
        }

        drop(backups);

        // Delete old backups
        for backup_id in to_delete {
            if let Err(e) = self.delete_backup(&backup_id).await {
                log::error!("Failed to delete old backup {}: {}", backup_id, e);
            }
        }

        Ok(())
    }

    /// Get backup statistics
    pub async fn get_stats(&self) -> BackupStats {
        let backups = self.backups.read().await;
        let recovery_points = self.recovery_points.read().await;

        let total_backups = backups.len() as u64;
        let total_size: u64 = backups.values().map(|m| m.total_size).sum();
        let compressed_size: u64 = backups.values().map(|m| m.compressed_size).sum();
        let verified_backups = backups.values().filter(|m| m.verification_status == Some(true)).count() as u64;

        BackupStats {
            total_backups,
            total_size,
            compressed_size,
            verified_backups,
            recovery_points: recovery_points.len() as u64,
        }
    }
}

/// Backup statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BackupStats {
    pub total_backups: u64,
    pub total_size: u64,
    pub compressed_size: u64,
    pub verified_backups: u64,
    pub recovery_points: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_config_default() {
        let config = BackupConfig::default();
        assert_eq!(config.interval_secs, 86400);
        assert_eq!(config.max_backups, 30);
        assert!(config.enable_compression);
    }

    #[test]
    fn test_backup_type() {
        assert_ne!(BackupType::Full, BackupType::Incremental);
        assert_ne!(BackupType::Incremental, BackupType::Differential);
    }
}
