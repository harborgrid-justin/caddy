//! Enterprise Cloud Sync & Backup System
//!
//! This module provides comprehensive cloud synchronization, backup, and storage
//! capabilities for CADDY enterprise deployments. It includes:
//!
//! - **Sync Engine**: Bidirectional synchronization with conflict resolution
//! - **Cloud Storage**: Abstraction over S3, Azure Blob, and GCS
//! - **Backup System**: Incremental and full backups with verification
//! - **Versioning**: Complete version control with branching and merging
//! - **Transfer Management**: Efficient chunked transfers with resume capability
//! - **Local Cache**: LRU cache with offline mode support
//!
//! # Examples
//!
//! ```no_run
//! use caddy::enterprise::cloud::{SyncEngine, S3Storage};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let storage = S3Storage::new("my-bucket", "us-east-1").await?;
//!     let sync_engine = SyncEngine::new(storage);
//!     sync_engine.start_sync().await?;
//!     Ok(())
//! }
//! ```

pub mod backup;
pub mod cache;
pub mod storage;
pub mod sync;
pub mod transfer;
pub mod versioning;

// Re-export commonly used types
pub use backup::{BackupEngine, BackupConfig, BackupType, RecoveryPoint};
pub use cache::{CloudCache, CacheConfig, CachePolicy};
pub use storage::{CloudStorage, S3Storage, AzureBlobStorage, GCSStorage, StorageError};
pub use sync::{SyncEngine, SyncConfig, ConflictStrategy, SyncState};
pub use transfer::{TransferManager, TransferConfig, TransferProgress, ChunkInfo};
pub use versioning::{VersionControl, Version, VersionDiff, MergeStrategy};

/// Cloud sync and backup result type
pub type CloudResult<T> = Result<T, CloudError>;

/// Unified error type for cloud operations
#[derive(Debug, thiserror::Error)]
pub enum CloudError {
    #[error("Storage error: {0}")]
    Storage(#[from] storage::StorageError),

    #[error("Sync error: {0}")]
    Sync(#[from] sync::SyncError),

    #[error("Backup error: {0}")]
    Backup(#[from] backup::BackupError),

    #[error("Version control error: {0}")]
    Versioning(#[from] versioning::VersionError),

    #[error("Transfer error: {0}")]
    Transfer(#[from] transfer::TransferError),

    #[error("Cache error: {0}")]
    Cache(#[from] cache::CacheError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

impl CloudError {
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}
