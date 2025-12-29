//! Job queue implementation with Redis backend
//!
//! This module provides:
//! - Priority queue with Redis backend
//! - Job deduplication
//! - Job cancellation
//! - Progress tracking
//! - Delayed job execution

use chrono::{DateTime, Utc};
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::scheduling::scheduler::JobPriority;

/// Queue errors
#[derive(Error, Debug)]
pub enum QueueError {
    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Job not found: {0}")]
    JobNotFound(String),

    #[error("Duplicate job: {0}")]
    DuplicateJob(String),

    #[error("Invalid priority: {0}")]
    InvalidPriority(i32),

    #[error("Queue operation failed: {0}")]
    OperationFailed(String),
}

/// Result type for queue operations
pub type QueueResult<T> = Result<T, QueueError>;

/// Job progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobProgress {
    pub job_id: String,
    pub current: u64,
    pub total: u64,
    pub percentage: f64,
    pub message: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl JobProgress {
    /// Create new job progress
    pub fn new(job_id: String, total: u64) -> Self {
        Self {
            job_id,
            current: 0,
            total,
            percentage: 0.0,
            message: None,
            updated_at: Utc::now(),
        }
    }

    /// Update progress
    pub fn update(&mut self, current: u64, message: Option<String>) {
        self.current = current;
        self.percentage = if self.total > 0 {
            (current as f64 / self.total as f64) * 100.0
        } else {
            0.0
        };
        self.message = message;
        self.updated_at = Utc::now();
    }

    /// Check if job is complete
    pub fn is_complete(&self) -> bool {
        self.current >= self.total
    }
}

/// Queued job with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedJob {
    pub id: String,
    pub queue_name: String,
    pub job_type: String,
    pub priority: JobPriority,
    pub payload: serde_json::Value,
    pub dedup_key: Option<String>,
    pub delay_until: Option<DateTime<Utc>>,
    pub max_retries: u32,
    pub retry_count: u32,
    pub timeout_seconds: u64,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl QueuedJob {
    /// Create a new queued job
    pub fn new(queue_name: String, job_type: String, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            queue_name,
            job_type,
            priority: JobPriority::Normal,
            payload,
            dedup_key: None,
            delay_until: None,
            max_retries: 3,
            retry_count: 0,
            timeout_seconds: 300,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error: None,
            metadata: HashMap::new(),
        }
    }

    /// Set job priority
    pub fn with_priority(mut self, priority: JobPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set deduplication key
    pub fn with_dedup_key(mut self, key: String) -> Self {
        self.dedup_key = Some(key);
        self
    }

    /// Set delay
    pub fn with_delay(mut self, delay_until: DateTime<Utc>) -> Self {
        self.delay_until = Some(delay_until);
        self
    }

    /// Set max retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Check if job is ready to run
    pub fn is_ready(&self) -> bool {
        self.delay_until.map_or(true, |delay| delay <= Utc::now())
    }

    /// Get priority score for sorting
    pub fn priority_score(&self) -> i64 {
        let priority_value = (self.priority as i64) * 1_000_000;
        let age_penalty = -(self.created_at.timestamp());
        priority_value + age_penalty
    }
}

/// Priority job queue with Redis backend
pub struct JobQueue {
    redis: ConnectionManager,
    queues: Arc<RwLock<HashMap<String, String>>>,
    progress: Arc<RwLock<HashMap<String, JobProgress>>>,
    dedup_keys: Arc<RwLock<HashMap<String, String>>>,
}

impl JobQueue {
    /// Create a new job queue
    pub async fn new(redis_url: &str) -> QueueResult<Self> {
        let client = redis::Client::open(redis_url)?;
        let redis = ConnectionManager::new(client).await?;

        Ok(Self {
            redis,
            queues: Arc::new(RwLock::new(HashMap::new())),
            progress: Arc::new(RwLock::new(HashMap::new())),
            dedup_keys: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Enqueue a job
    pub async fn enqueue(&self, job: QueuedJob) -> QueueResult<String> {
        // Check for duplicate
        if let Some(dedup_key) = &job.dedup_key {
            let mut dedup_keys = self.dedup_keys.write().await;
            if dedup_keys.contains_key(dedup_key) {
                return Err(QueueError::DuplicateJob(dedup_key.clone()));
            }
            dedup_keys.insert(dedup_key.clone(), job.id.clone());
        }

        let queue_key = format!("queue:{}", job.queue_name);
        let job_id = job.id.clone();
        let score = job.priority_score() as f64;

        // Serialize job
        let job_data = serde_json::to_string(&job)?;

        // Store job data
        let job_key = format!("job:{}", job_id);
        redis::cmd("SET")
            .arg(&job_key)
            .arg(&job_data)
            .query_async(&mut self.redis.clone())
            .await?;

        // Add to sorted set (priority queue)
        redis::cmd("ZADD")
            .arg(&queue_key)
            .arg(score)
            .arg(&job_id)
            .query_async(&mut self.redis.clone())
            .await?;

        // Store queue reference
        let mut queues = self.queues.write().await;
        queues.insert(job.queue_name.clone(), queue_key);

        Ok(job_id)
    }

    /// Dequeue next job from queue
    pub async fn dequeue(&self, queue_name: &str) -> QueueResult<Option<QueuedJob>> {
        let queue_key = format!("queue:{}", queue_name);

        // Get highest priority job (highest score)
        let result: Vec<String> = redis::cmd("ZRANGE")
            .arg(&queue_key)
            .arg(0)
            .arg(0)
            .arg("REV")
            .query_async(&mut self.redis.clone())
            .await?;

        if result.is_empty() {
            return Ok(None);
        }

        let job_id = &result[0];
        let job_key = format!("job:{}", job_id);

        // Get job data
        let job_data: Option<String> = redis::cmd("GET")
            .arg(&job_key)
            .query_async(&mut self.redis.clone())
            .await?;

        let job_data = match job_data {
            Some(data) => data,
            None => return Ok(None),
        };

        let mut job: QueuedJob = serde_json::from_str(&job_data)?;

        // Check if job is ready (delay)
        if !job.is_ready() {
            return Ok(None);
        }

        // Remove from queue
        redis::cmd("ZREM")
            .arg(&queue_key)
            .arg(job_id)
            .query_async(&mut self.redis.clone())
            .await?;

        // Mark as started
        job.started_at = Some(Utc::now());

        // Update job data
        let updated_data = serde_json::to_string(&job)?;
        redis::cmd("SET")
            .arg(&job_key)
            .arg(&updated_data)
            .query_async(&mut self.redis.clone())
            .await?;

        Ok(Some(job))
    }

    /// Cancel a job
    pub async fn cancel(&self, job_id: &str) -> QueueResult<()> {
        let job_key = format!("job:{}", job_id);

        // Get job data
        let job_data: Option<String> = redis::cmd("GET")
            .arg(&job_key)
            .query_async(&mut self.redis.clone())
            .await?;

        let job_data = job_data.ok_or_else(|| QueueError::JobNotFound(job_id.to_string()))?;

        let job: QueuedJob = serde_json::from_str(&job_data)?;
        let queue_key = format!("queue:{}", job.queue_name);

        // Remove from queue
        redis::cmd("ZREM")
            .arg(&queue_key)
            .arg(job_id)
            .query_async(&mut self.redis.clone())
            .await?;

        // Delete job data
        redis::cmd("DEL")
            .arg(&job_key)
            .query_async(&mut self.redis.clone())
            .await?;

        // Remove dedup key if present
        if let Some(dedup_key) = job.dedup_key {
            let mut dedup_keys = self.dedup_keys.write().await;
            dedup_keys.remove(&dedup_key);
        }

        // Remove progress
        let mut progress = self.progress.write().await;
        progress.remove(job_id);

        Ok(())
    }

    /// Get job by ID
    pub async fn get_job(&self, job_id: &str) -> QueueResult<Option<QueuedJob>> {
        let job_key = format!("job:{}", job_id);

        let job_data: Option<String> = redis::cmd("GET")
            .arg(&job_key)
            .query_async(&mut self.redis.clone())
            .await?;

        match job_data {
            Some(data) => Ok(Some(serde_json::from_str(&data)?)),
            None => Ok(None),
        }
    }

    /// Update job progress
    pub async fn update_progress(
        &self,
        job_id: &str,
        current: u64,
        total: u64,
        message: Option<String>,
    ) -> QueueResult<()> {
        let mut progress = self.progress.write().await;

        let job_progress = progress
            .entry(job_id.to_string())
            .or_insert_with(|| JobProgress::new(job_id.to_string(), total));

        job_progress.update(current, message);

        // Persist to Redis
        let progress_key = format!("progress:{}", job_id);
        let progress_data = serde_json::to_string(&job_progress)?;

        redis::cmd("SET")
            .arg(&progress_key)
            .arg(&progress_data)
            .arg("EX")
            .arg(3600) // Expire after 1 hour
            .query_async(&mut self.redis.clone())
            .await?;

        Ok(())
    }

    /// Get job progress
    pub async fn get_progress(&self, job_id: &str) -> QueueResult<Option<JobProgress>> {
        // Check memory first
        {
            let progress = self.progress.read().await;
            if let Some(p) = progress.get(job_id) {
                return Ok(Some(p.clone()));
            }
        }

        // Check Redis
        let progress_key = format!("progress:{}", job_id);
        let progress_data: Option<String> = redis::cmd("GET")
            .arg(&progress_key)
            .query_async(&mut self.redis.clone())
            .await?;

        match progress_data {
            Some(data) => {
                let job_progress: JobProgress = serde_json::from_str(&data)?;
                // Cache in memory
                let mut progress = self.progress.write().await;
                progress.insert(job_id.to_string(), job_progress.clone());
                Ok(Some(job_progress))
            }
            None => Ok(None),
        }
    }

    /// Mark job as completed
    pub async fn complete_job(&self, job_id: &str) -> QueueResult<()> {
        let job_key = format!("job:{}", job_id);

        let job_data: Option<String> = redis::cmd("GET")
            .arg(&job_key)
            .query_async(&mut self.redis.clone())
            .await?;

        let job_data = job_data.ok_or_else(|| QueueError::JobNotFound(job_id.to_string()))?;

        let mut job: QueuedJob = serde_json::from_str(&job_data)?;
        job.completed_at = Some(Utc::now());

        // Update job data
        let updated_data = serde_json::to_string(&job)?;
        redis::cmd("SET")
            .arg(&job_key)
            .arg(&updated_data)
            .arg("EX")
            .arg(86400) // Keep completed jobs for 24 hours
            .query_async(&mut self.redis.clone())
            .await?;

        // Remove dedup key
        if let Some(dedup_key) = job.dedup_key {
            let mut dedup_keys = self.dedup_keys.write().await;
            dedup_keys.remove(&dedup_key);
        }

        Ok(())
    }

    /// Mark job as failed and optionally requeue
    pub async fn fail_job(&self, job_id: &str, error: String) -> QueueResult<()> {
        let job_key = format!("job:{}", job_id);

        let job_data: Option<String> = redis::cmd("GET")
            .arg(&job_key)
            .query_async(&mut self.redis.clone())
            .await?;

        let job_data = job_data.ok_or_else(|| QueueError::JobNotFound(job_id.to_string()))?;

        let mut job: QueuedJob = serde_json::from_str(&job_data)?;
        job.retry_count += 1;
        job.error = Some(error);

        if job.retry_count < job.max_retries {
            // Requeue with exponential backoff
            let backoff_seconds = 2_i64.pow(job.retry_count) * 60;
            job.delay_until = Some(Utc::now() + chrono::Duration::seconds(backoff_seconds));

            // Enqueue again
            self.enqueue(job).await?;
        } else {
            // Move to dead letter queue
            self.move_to_dead_letter(&job).await?;

            // Remove from main queue
            redis::cmd("DEL")
                .arg(&job_key)
                .query_async(&mut self.redis.clone())
                .await?;
        }

        Ok(())
    }

    /// Move job to dead letter queue
    async fn move_to_dead_letter(&self, job: &QueuedJob) -> QueueResult<()> {
        let dlq_key = format!("dlq:{}", job.queue_name);
        let job_data = serde_json::to_string(job)?;

        redis::cmd("LPUSH")
            .arg(&dlq_key)
            .arg(&job_data)
            .query_async(&mut self.redis.clone())
            .await?;

        // Keep only last 1000 failed jobs
        redis::cmd("LTRIM")
            .arg(&dlq_key)
            .arg(0)
            .arg(999)
            .query_async(&mut self.redis.clone())
            .await?;

        Ok(())
    }

    /// Get dead letter queue jobs
    pub async fn get_dead_letter_jobs(&self, queue_name: &str) -> QueueResult<Vec<QueuedJob>> {
        let dlq_key = format!("dlq:{}", queue_name);

        let jobs_data: Vec<String> = redis::cmd("LRANGE")
            .arg(&dlq_key)
            .arg(0)
            .arg(-1)
            .query_async(&mut self.redis.clone())
            .await?;

        let mut jobs = Vec::new();
        for data in jobs_data {
            if let Ok(job) = serde_json::from_str::<QueuedJob>(&data) {
                jobs.push(job);
            }
        }

        Ok(jobs)
    }

    /// Get queue size
    pub async fn queue_size(&self, queue_name: &str) -> QueueResult<usize> {
        let queue_key = format!("queue:{}", queue_name);

        let size: usize = redis::cmd("ZCARD")
            .arg(&queue_key)
            .query_async(&mut self.redis.clone())
            .await?;

        Ok(size)
    }

    /// Get queue statistics
    pub async fn queue_stats(&self, queue_name: &str) -> QueueResult<QueueStats> {
        let queue_size = self.queue_size(queue_name).await?;
        let dlq_size = self.dead_letter_queue_size(queue_name).await?;

        Ok(QueueStats {
            queue_name: queue_name.to_string(),
            pending_jobs: queue_size,
            failed_jobs: dlq_size,
        })
    }

    /// Get dead letter queue size
    async fn dead_letter_queue_size(&self, queue_name: &str) -> QueueResult<usize> {
        let dlq_key = format!("dlq:{}", queue_name);

        let size: usize = redis::cmd("LLEN")
            .arg(&dlq_key)
            .query_async(&mut self.redis.clone())
            .await?;

        Ok(size)
    }

    /// Clear queue
    pub async fn clear_queue(&self, queue_name: &str) -> QueueResult<()> {
        let queue_key = format!("queue:{}", queue_name);

        redis::cmd("DEL")
            .arg(&queue_key)
            .query_async(&mut self.redis.clone())
            .await?;

        Ok(())
    }
}

/// Queue statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    pub queue_name: String,
    pub pending_jobs: usize,
    pub failed_jobs: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_creation() {
        let job = QueuedJob::new(
            "test".to_string(),
            "scan".to_string(),
            serde_json::json!({"url": "https://example.com"}),
        );

        assert_eq!(job.queue_name, "test");
        assert_eq!(job.job_type, "scan");
        assert_eq!(job.priority, JobPriority::Normal);
        assert_eq!(job.retry_count, 0);
    }

    #[test]
    fn test_job_builder() {
        let job = QueuedJob::new(
            "test".to_string(),
            "scan".to_string(),
            serde_json::json!({}),
        )
        .with_priority(JobPriority::High)
        .with_dedup_key("unique-key".to_string())
        .with_max_retries(5)
        .with_timeout(600);

        assert_eq!(job.priority, JobPriority::High);
        assert_eq!(job.dedup_key, Some("unique-key".to_string()));
        assert_eq!(job.max_retries, 5);
        assert_eq!(job.timeout_seconds, 600);
    }

    #[test]
    fn test_progress_tracking() {
        let mut progress = JobProgress::new("job-1".to_string(), 100);

        assert_eq!(progress.percentage, 0.0);
        assert!(!progress.is_complete());

        progress.update(50, Some("Half done".to_string()));
        assert_eq!(progress.percentage, 50.0);
        assert!(!progress.is_complete());

        progress.update(100, Some("Complete".to_string()));
        assert_eq!(progress.percentage, 100.0);
        assert!(progress.is_complete());
    }
}
