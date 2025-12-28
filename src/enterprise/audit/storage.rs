//! Audit storage backends
//!
//! Provides various storage implementations for audit logs.

use crate::enterprise::audit::event::AuditEvent;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

/// Errors that can occur during storage operations
#[derive(Debug, Error)]
pub enum StorageError {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Storage not initialized
    #[error("Storage not initialized")]
    NotInitialized,

    /// Storage is full
    #[error("Storage is full")]
    Full,

    /// Invalid query
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
}

/// Result type for storage operations
pub type Result<T> = std::result::Result<T, StorageError>;

/// Trait for audit event storage backends
#[async_trait]
pub trait AuditStorage: Send + Sync {
    /// Store an audit event
    async fn store(&mut self, event: &AuditEvent) -> Result<()>;

    /// Retrieve events within a time range
    async fn retrieve(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AuditEvent>>;

    /// Retrieve events by user ID
    async fn retrieve_by_user(&self, user_id: &str) -> Result<Vec<AuditEvent>>;

    /// Retrieve events by resource
    async fn retrieve_by_resource(&self, resource: &str) -> Result<Vec<AuditEvent>>;

    /// Count total events
    async fn count(&self) -> Result<usize>;

    /// Rotate logs (for backends that support rotation)
    async fn rotate(&mut self) -> Result<()>;

    /// Verify integrity of stored events
    async fn verify_integrity(&self) -> Result<bool>;
}

/// In-memory storage (for testing)
#[derive(Debug, Clone)]
pub struct MemoryStorage {
    events: Arc<RwLock<Vec<AuditEvent>>>,
    max_size: usize,
}

impl MemoryStorage {
    /// Create a new memory storage
    pub fn new() -> Self {
        Self::with_capacity(10000)
    }

    /// Create a new memory storage with specified capacity
    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            max_size,
        }
    }

    /// Get all events (for testing)
    pub async fn get_all(&self) -> Vec<AuditEvent> {
        self.events.read().clone()
    }

    /// Clear all events
    pub async fn clear(&mut self) {
        self.events.write().clear();
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuditStorage for MemoryStorage {
    async fn store(&mut self, event: &AuditEvent) -> Result<()> {
        let mut events = self.events.write();
        if events.len() >= self.max_size {
            return Err(StorageError::Full);
        }
        events.push(event.clone());
        Ok(())
    }

    async fn retrieve(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AuditEvent>> {
        let events = self.events.read();
        Ok(events
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .cloned()
            .collect())
    }

    async fn retrieve_by_user(&self, user_id: &str) -> Result<Vec<AuditEvent>> {
        let events = self.events.read();
        Ok(events
            .iter()
            .filter(|e| e.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn retrieve_by_resource(&self, resource: &str) -> Result<Vec<AuditEvent>> {
        let events = self.events.read();
        Ok(events
            .iter()
            .filter(|e| e.resource == resource)
            .cloned()
            .collect())
    }

    async fn count(&self) -> Result<usize> {
        Ok(self.events.read().len())
    }

    async fn rotate(&mut self) -> Result<()> {
        // Memory storage doesn't need rotation
        Ok(())
    }

    async fn verify_integrity(&self) -> Result<bool> {
        let events = self.events.read();
        let mut prev_hash: Option<String> = None;

        for event in events.iter() {
            if let Some(ref hash) = event.hash {
                // Verify current hash
                let calculated = event.calculate_hash(prev_hash.as_deref());
                if &calculated != hash {
                    return Ok(false);
                }
                prev_hash = Some(hash.clone());
            }
        }

        Ok(true)
    }
}

/// File-based storage with rotation
pub struct FileStorage {
    base_path: PathBuf,
    current_file: Option<BufWriter<File>>,
    current_size: usize,
    max_file_size: usize,
    max_files: usize,
    file_index: usize,
}

impl FileStorage {
    /// Create a new file storage
    pub fn new(base_path: impl AsRef<Path>) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        std::fs::create_dir_all(&base_path)?;

        Ok(Self {
            base_path,
            current_file: None,
            current_size: 0,
            max_file_size: 100 * 1024 * 1024, // 100MB
            max_files: 100,
            file_index: 0,
        })
    }

    /// Set maximum file size before rotation
    pub fn with_max_file_size(mut self, size: usize) -> Self {
        self.max_file_size = size;
        self
    }

    /// Set maximum number of files to keep
    pub fn with_max_files(mut self, count: usize) -> Self {
        self.max_files = count;
        self
    }

    /// Get the current log file path
    fn get_file_path(&self, index: usize) -> PathBuf {
        self.base_path
            .join(format!("audit_{:06}.jsonl", index))
    }

    /// Open or create the current log file
    fn ensure_file(&mut self) -> Result<&mut BufWriter<File>> {
        if self.current_file.is_none() {
            let path = self.get_file_path(self.file_index);
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?;
            self.current_file = Some(BufWriter::new(file));
            self.current_size = 0;
        }

        Ok(self.current_file.as_mut().unwrap())
    }

    /// Rotate to a new log file
    fn rotate_file(&mut self) -> Result<()> {
        // Flush and close current file
        if let Some(mut file) = self.current_file.take() {
            file.flush()?;
        }

        // Increment index
        self.file_index += 1;

        // Remove old files if we exceed max_files
        if self.file_index > self.max_files {
            let old_file = self.get_file_path(self.file_index - self.max_files);
            let _ = std::fs::remove_file(old_file);
        }

        // Reset size counter
        self.current_size = 0;

        Ok(())
    }

    /// Read events from a file
    fn read_file(&self, path: &Path) -> Result<Vec<AuditEvent>> {
        if !path.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(path)?;
        let events: Vec<AuditEvent> = content
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();

        Ok(events)
    }
}

#[async_trait]
impl AuditStorage for FileStorage {
    async fn store(&mut self, event: &AuditEvent) -> Result<()> {
        // Serialize event
        let json = serde_json::to_string(event)?;
        let size = json.len() + 1; // +1 for newline

        // Check if rotation is needed
        if self.current_size + size > self.max_file_size {
            self.rotate_file()?;
        }

        // Write to file
        let file = self.ensure_file()?;
        writeln!(file, "{}", json)?;
        file.flush()?;

        self.current_size += size;

        Ok(())
    }

    async fn retrieve(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AuditEvent>> {
        let mut all_events = Vec::new();

        // Read all log files
        for i in 0..=self.file_index {
            let path = self.get_file_path(i);
            let events = self.read_file(&path)?;
            all_events.extend(events);
        }

        // Filter by time range
        Ok(all_events
            .into_iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .collect())
    }

    async fn retrieve_by_user(&self, user_id: &str) -> Result<Vec<AuditEvent>> {
        let mut all_events = Vec::new();

        for i in 0..=self.file_index {
            let path = self.get_file_path(i);
            let events = self.read_file(&path)?;
            all_events.extend(events);
        }

        Ok(all_events
            .into_iter()
            .filter(|e| e.user_id == user_id)
            .collect())
    }

    async fn retrieve_by_resource(&self, resource: &str) -> Result<Vec<AuditEvent>> {
        let mut all_events = Vec::new();

        for i in 0..=self.file_index {
            let path = self.get_file_path(i);
            let events = self.read_file(&path)?;
            all_events.extend(events);
        }

        Ok(all_events
            .into_iter()
            .filter(|e| e.resource == resource)
            .collect())
    }

    async fn count(&self) -> Result<usize> {
        let mut count = 0;

        for i in 0..=self.file_index {
            let path = self.get_file_path(i);
            count += self.read_file(&path)?.len();
        }

        Ok(count)
    }

    async fn rotate(&mut self) -> Result<()> {
        self.rotate_file()
    }

    async fn verify_integrity(&self) -> Result<bool> {
        let mut all_events = Vec::new();

        for i in 0..=self.file_index {
            let path = self.get_file_path(i);
            let events = self.read_file(&path)?;
            all_events.extend(events);
        }

        let mut prev_hash: Option<String> = None;

        for event in all_events.iter() {
            if let Some(ref hash) = event.hash {
                let calculated = event.calculate_hash(prev_hash.as_deref());
                if &calculated != hash {
                    return Ok(false);
                }
                prev_hash = Some(hash.clone());
            }
        }

        Ok(true)
    }
}

/// Database storage stub (for future implementation)
pub struct DatabaseStorage {
    connection_string: String,
}

impl DatabaseStorage {
    /// Create a new database storage
    pub fn new(connection_string: String) -> Self {
        Self { connection_string }
    }
}

#[async_trait]
impl AuditStorage for DatabaseStorage {
    async fn store(&mut self, _event: &AuditEvent) -> Result<()> {
        // TODO: Implement database storage
        log::warn!("Database storage not yet implemented");
        Ok(())
    }

    async fn retrieve(
        &self,
        _start: DateTime<Utc>,
        _end: DateTime<Utc>,
    ) -> Result<Vec<AuditEvent>> {
        Ok(Vec::new())
    }

    async fn retrieve_by_user(&self, _user_id: &str) -> Result<Vec<AuditEvent>> {
        Ok(Vec::new())
    }

    async fn retrieve_by_resource(&self, _resource: &str) -> Result<Vec<AuditEvent>> {
        Ok(Vec::new())
    }

    async fn count(&self) -> Result<usize> {
        Ok(0)
    }

    async fn rotate(&mut self) -> Result<()> {
        Ok(())
    }

    async fn verify_integrity(&self) -> Result<bool> {
        Ok(true)
    }
}

/// S3/Cloud storage stub (for future implementation)
pub struct CloudStorage {
    bucket: String,
    region: String,
}

impl CloudStorage {
    /// Create a new cloud storage
    pub fn new(bucket: String, region: String) -> Self {
        Self { bucket, region }
    }
}

#[async_trait]
impl AuditStorage for CloudStorage {
    async fn store(&mut self, _event: &AuditEvent) -> Result<()> {
        // TODO: Implement cloud storage
        log::warn!("Cloud storage not yet implemented");
        Ok(())
    }

    async fn retrieve(
        &self,
        _start: DateTime<Utc>,
        _end: DateTime<Utc>,
    ) -> Result<Vec<AuditEvent>> {
        Ok(Vec::new())
    }

    async fn retrieve_by_user(&self, _user_id: &str) -> Result<Vec<AuditEvent>> {
        Ok(Vec::new())
    }

    async fn retrieve_by_resource(&self, _resource: &str) -> Result<Vec<AuditEvent>> {
        Ok(Vec::new())
    }

    async fn count(&self) -> Result<usize> {
        Ok(0)
    }

    async fn rotate(&mut self) -> Result<()> {
        Ok(())
    }

    async fn verify_integrity(&self) -> Result<bool> {
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enterprise::audit::event::{EventType, AuditEvent};

    #[tokio::test]
    async fn test_memory_storage() {
        let mut storage = MemoryStorage::new();

        let event = AuditEvent::builder()
            .user_id("user1")
            .action(EventType::Create)
            .resource("resource1")
            .build();

        storage.store(&event).await.unwrap();

        let count = storage.count().await.unwrap();
        assert_eq!(count, 1);

        let events = storage.retrieve_by_user("user1").await.unwrap();
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn test_file_storage() {
        let temp_dir = std::env::temp_dir().join("caddy_audit_test");
        let _ = std::fs::remove_dir_all(&temp_dir);

        let mut storage = FileStorage::new(&temp_dir).unwrap();

        let event = AuditEvent::builder()
            .user_id("user1")
            .action(EventType::Create)
            .resource("resource1")
            .build();

        storage.store(&event).await.unwrap();

        let count = storage.count().await.unwrap();
        assert_eq!(count, 1);

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }
}
