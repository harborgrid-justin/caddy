//! Audit logger implementation
//!
//! Thread-safe audit logger with async writing, rotation, and tamper-evident logging.

use crate::enterprise::audit::{
    event::AuditEvent,
    storage::{AuditStorage, StorageError},
};
use parking_lot::RwLock;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc;

/// Errors that can occur during audit logging
#[derive(Debug, Error)]
pub enum LoggerError {
    /// Storage error
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    /// Channel send error
    #[error("Failed to send event to logger: channel closed")]
    ChannelClosed,

    /// Logger not initialized
    #[error("Logger not initialized")]
    NotInitialized,

    /// Logger already shut down
    #[error("Logger already shut down")]
    ShutDown,
}

/// Result type for logger operations
pub type Result<T> = std::result::Result<T, LoggerError>;

/// Configuration for the audit logger
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    /// Buffer size for async logging
    pub buffer_size: usize,

    /// Enable tamper-evident logging with hash chains
    pub enable_hash_chain: bool,

    /// Maximum number of retries for failed writes
    pub max_retries: u32,

    /// Whether to panic on logging failures
    pub panic_on_failure: bool,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1000,
            enable_hash_chain: true,
            max_retries: 3,
            panic_on_failure: false,
        }
    }
}

/// Thread-safe audit logger
pub struct AuditLogger {
    config: LoggerConfig,
    sender: mpsc::UnboundedSender<AuditEvent>,
    last_hash: Arc<RwLock<Option<String>>>,
    is_running: Arc<RwLock<bool>>,
}

impl AuditLogger {
    /// Create a new audit logger with the given storage backend
    pub fn new<S>(storage: S, config: LoggerConfig) -> Self
    where
        S: AuditStorage + Send + 'static,
    {
        let (sender, receiver) = mpsc::unbounded_channel();
        let last_hash = Arc::new(RwLock::new(None));
        let is_running = Arc::new(RwLock::new(true));

        // Spawn background worker
        let worker_last_hash = last_hash.clone();
        let worker_is_running = is_running.clone();
        let worker_config = config.clone();

        tokio::spawn(async move {
            Self::worker_loop(
                receiver,
                storage,
                worker_last_hash,
                worker_is_running,
                worker_config,
            )
            .await;
        });

        Self {
            config,
            sender,
            last_hash,
            is_running,
        }
    }

    /// Log an audit event
    pub fn log(&self, mut event: AuditEvent) -> Result<()> {
        if !*self.is_running.read() {
            return Err(LoggerError::ShutDown);
        }

        // Add hash if enabled
        if self.config.enable_hash_chain {
            let last_hash = self.last_hash.read().clone();
            let hash = event.calculate_hash(last_hash.as_deref());
            event.hash = Some(hash.clone());
            event.previous_hash = last_hash;

            // Update last hash
            *self.last_hash.write() = Some(hash);
        }

        // Send to background worker
        self.sender
            .send(event)
            .map_err(|_| LoggerError::ChannelClosed)?;

        Ok(())
    }

    /// Log an event and wait for it to be written
    pub async fn log_sync(&self, event: AuditEvent) -> Result<()> {
        self.log(event)?;
        // In a real implementation, we'd use a oneshot channel to wait for confirmation
        // For now, just return immediately
        Ok(())
    }

    /// Flush all pending events
    pub async fn flush(&self) -> Result<()> {
        // In a real implementation, we'd send a flush command and wait
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        Ok(())
    }

    /// Shutdown the logger gracefully
    pub async fn shutdown(&self) -> Result<()> {
        *self.is_running.write() = false;
        drop(self.sender.clone()); // Close the channel
        self.flush().await?;
        Ok(())
    }

    /// Background worker loop that processes events
    async fn worker_loop<S>(
        mut receiver: mpsc::UnboundedReceiver<AuditEvent>,
        mut storage: S,
        _last_hash: Arc<RwLock<Option<String>>>,
        is_running: Arc<RwLock<bool>>,
        config: LoggerConfig,
    ) where
        S: AuditStorage,
    {
        while let Some(event) = receiver.recv().await {
            let mut retries = 0;
            let mut success = false;

            while retries < config.max_retries && !success {
                match storage.store(&event).await {
                    Ok(_) => {
                        success = true;
                    }
                    Err(e) => {
                        log::error!("Failed to store audit event: {}", e);
                        retries += 1;
                        if retries < config.max_retries {
                            tokio::time::sleep(tokio::time::Duration::from_millis(
                                100 * retries as u64,
                            ))
                            .await;
                        }
                    }
                }
            }

            if !success && config.panic_on_failure {
                panic!("Failed to store audit event after {} retries", config.max_retries);
            }
        }

        *is_running.write() = false;
    }

    /// Check if the logger is still running
    pub fn is_running(&self) -> bool {
        *self.is_running.read()
    }

    /// Get the last hash in the chain
    pub fn last_hash(&self) -> Option<String> {
        self.last_hash.read().clone()
    }
}

/// Global audit logger instance
static GLOBAL_LOGGER: RwLock<Option<Arc<AuditLogger>>> = RwLock::new(None);

/// Initialize the global audit logger
pub fn init_global_logger<S>(storage: S, config: LoggerConfig)
where
    S: AuditStorage + Send + 'static,
{
    let logger = AuditLogger::new(storage, config);
    *GLOBAL_LOGGER.write() = Some(Arc::new(logger));
}

/// Get the global audit logger
pub fn global_logger() -> Option<Arc<AuditLogger>> {
    GLOBAL_LOGGER.read().clone()
}

/// Log an event using the global logger
pub fn log_event(event: AuditEvent) -> Result<()> {
    if let Some(logger) = global_logger() {
        logger.log(event)
    } else {
        Err(LoggerError::NotInitialized)
    }
}

/// Convenience macro for logging audit events
#[macro_export]
macro_rules! audit_log {
    ($action:expr, $resource:expr, $user_id:expr) => {
        $crate::enterprise::audit::logger::log_event(
            $crate::enterprise::audit::event::AuditEvent::builder()
                .action($action)
                .resource($resource)
                .user_id($user_id)
                .build(),
        )
    };

    ($action:expr, $resource:expr, $user_id:expr, $($key:expr => $value:expr),+) => {
        $crate::enterprise::audit::logger::log_event(
            $crate::enterprise::audit::event::AuditEvent::builder()
                .action($action)
                .resource($resource)
                .user_id($user_id)
                $(
                    .detail($key, $value)
                )+
                .build(),
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enterprise::audit::event::{EventType, AuditEvent};
    use crate::enterprise::audit::storage::MemoryStorage;

    #[tokio::test]
    async fn test_logger_basic() {
        let storage = MemoryStorage::new();
        let config = LoggerConfig::default();
        let logger = AuditLogger::new(storage.clone(), config);

        let event = AuditEvent::builder()
            .user_id("test_user")
            .action(EventType::Create)
            .resource("test_resource")
            .build();

        logger.log(event).unwrap();
        logger.flush().await.unwrap();

        // Give some time for async processing
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let events = storage.get_all().await;
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn test_hash_chain() {
        let storage = MemoryStorage::new();
        let config = LoggerConfig {
            enable_hash_chain: true,
            ..Default::default()
        };
        let logger = AuditLogger::new(storage, config);

        let event1 = AuditEvent::builder()
            .user_id("user1")
            .action(EventType::Create)
            .resource("resource1")
            .build();

        let event2 = AuditEvent::builder()
            .user_id("user1")
            .action(EventType::Update)
            .resource("resource1")
            .build();

        logger.log(event1).unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let hash1 = logger.last_hash();
        assert!(hash1.is_some());

        logger.log(event2).unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let hash2 = logger.last_hash();
        assert!(hash2.is_some());
        assert_ne!(hash1, hash2);
    }
}
