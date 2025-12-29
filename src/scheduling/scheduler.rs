//! Job scheduler with cron-based scheduling, one-time jobs, and distributed locking
//!
//! This module provides a comprehensive job scheduling system with:
//! - Cron-based recurring schedules
//! - One-time scheduled jobs
//! - Priority-based job queues
//! - Distributed job locking via Redis
//! - Job persistence and recovery

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use cron::Schedule;
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Scheduler errors
#[derive(Error, Debug)]
pub enum SchedulerError {
    #[error("Invalid cron expression: {0}")]
    InvalidCronExpression(String),

    #[error("Job not found: {0}")]
    JobNotFound(String),

    #[error("Failed to acquire lock: {0}")]
    LockAcquisitionFailed(String),

    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Job execution error: {0}")]
    ExecutionError(String),

    #[error("Invalid job state transition")]
    InvalidStateTransition,
}

/// Result type for scheduler operations
pub type SchedulerResult<T> = Result<T, SchedulerError>;

/// Job priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum JobPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Job execution status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Scheduled,
    Running,
    Completed,
    Failed,
    Cancelled,
    Retrying,
}

/// Job schedule type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobSchedule {
    /// One-time execution at specific time
    Once(DateTime<Utc>),

    /// Recurring execution based on cron expression
    Cron(String),

    /// Recurring execution at fixed intervals
    Interval {
        duration: i64, // seconds
        start: Option<DateTime<Utc>>,
    },
}

/// Job metadata and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub name: String,
    pub job_type: String,
    pub schedule: JobSchedule,
    pub priority: JobPriority,
    pub status: JobStatus,
    pub payload: serde_json::Value,
    pub max_retries: u32,
    pub retry_count: u32,
    pub timeout_seconds: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub next_run: Option<DateTime<Utc>>,
    pub last_run: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub tags: HashMap<String, String>,
}

impl Job {
    /// Create a new job with default values
    pub fn new(name: String, job_type: String, schedule: JobSchedule) -> Self {
        let now = Utc::now();
        let next_run = Self::calculate_next_run(&schedule, now);

        Self {
            id: Uuid::new_v4().to_string(),
            name,
            job_type,
            schedule,
            priority: JobPriority::Normal,
            status: JobStatus::Pending,
            payload: serde_json::Value::Null,
            max_retries: 3,
            retry_count: 0,
            timeout_seconds: 300,
            created_at: now,
            updated_at: now,
            next_run,
            last_run: None,
            last_error: None,
            tags: HashMap::new(),
        }
    }

    /// Calculate the next run time for a job
    fn calculate_next_run(schedule: &JobSchedule, from: DateTime<Utc>) -> Option<DateTime<Utc>> {
        match schedule {
            JobSchedule::Once(time) => {
                if *time > from {
                    Some(*time)
                } else {
                    None
                }
            }
            JobSchedule::Cron(expr) => {
                Schedule::from_str(expr)
                    .ok()
                    .and_then(|sched| sched.upcoming(Utc).next())
            }
            JobSchedule::Interval { duration, start } => {
                let start_time = start.unwrap_or(from);
                if start_time > from {
                    Some(start_time)
                } else {
                    Some(from + Duration::seconds(*duration))
                }
            }
        }
    }

    /// Update the next run time based on schedule
    pub fn update_next_run(&mut self) {
        self.next_run = Self::calculate_next_run(&self.schedule, Utc::now());
        self.updated_at = Utc::now();
    }

    /// Mark job as completed
    pub fn mark_completed(&mut self) {
        self.status = JobStatus::Completed;
        self.last_run = Some(Utc::now());
        self.update_next_run();
        self.retry_count = 0;
    }

    /// Mark job as failed
    pub fn mark_failed(&mut self, error: String) {
        self.retry_count += 1;
        self.last_error = Some(error);
        self.last_run = Some(Utc::now());

        if self.retry_count >= self.max_retries {
            self.status = JobStatus::Failed;
        } else {
            self.status = JobStatus::Retrying;
            // Schedule retry with exponential backoff
            let backoff_seconds = 2_i64.pow(self.retry_count) * 60;
            self.next_run = Some(Utc::now() + Duration::seconds(backoff_seconds));
        }

        self.updated_at = Utc::now();
    }
}

/// Trait for job executors
#[async_trait]
pub trait JobExecutor: Send + Sync {
    /// Execute a job with given payload
    async fn execute(&self, job: &Job) -> SchedulerResult<()>;

    /// Get the job type this executor handles
    fn job_type(&self) -> &str;
}

/// Distributed lock for job execution
pub struct DistributedLock {
    redis: ConnectionManager,
    lock_key: String,
    lock_value: String,
    ttl_seconds: usize,
}

impl DistributedLock {
    /// Acquire a distributed lock
    pub async fn acquire(
        redis: ConnectionManager,
        resource: &str,
        ttl_seconds: usize,
    ) -> SchedulerResult<Option<Self>> {
        let lock_key = format!("lock:{}", resource);
        let lock_value = Uuid::new_v4().to_string();

        let result: Option<String> = redis::cmd("SET")
            .arg(&lock_key)
            .arg(&lock_value)
            .arg("NX")
            .arg("EX")
            .arg(ttl_seconds)
            .query_async(&mut redis.clone())
            .await?;

        if result.is_some() {
            Ok(Some(Self {
                redis,
                lock_key,
                lock_value,
                ttl_seconds,
            }))
        } else {
            Ok(None)
        }
    }

    /// Extend the lock TTL
    pub async fn extend(&mut self) -> SchedulerResult<bool> {
        let script = r#"
            if redis.call("get", KEYS[1]) == ARGV[1] then
                return redis.call("expire", KEYS[1], ARGV[2])
            else
                return 0
            end
        "#;

        let result: i32 = redis::cmd("EVAL")
            .arg(script)
            .arg(1)
            .arg(&self.lock_key)
            .arg(&self.lock_value)
            .arg(self.ttl_seconds)
            .query_async(&mut self.redis)
            .await?;

        Ok(result == 1)
    }

    /// Release the lock
    pub async fn release(mut self) -> SchedulerResult<()> {
        let script = r#"
            if redis.call("get", KEYS[1]) == ARGV[1] then
                return redis.call("del", KEYS[1])
            else
                return 0
            end
        "#;

        let _: i32 = redis::cmd("EVAL")
            .arg(script)
            .arg(1)
            .arg(&self.lock_key)
            .arg(&self.lock_value)
            .query_async(&mut self.redis)
            .await?;

        Ok(())
    }
}

/// Job scheduler
pub struct JobScheduler {
    jobs: Arc<RwLock<HashMap<String, Job>>>,
    executors: Arc<RwLock<HashMap<String, Arc<dyn JobExecutor>>>>,
    redis: ConnectionManager,
}

impl JobScheduler {
    /// Create a new job scheduler
    pub async fn new(redis_url: &str) -> SchedulerResult<Self> {
        let client = redis::Client::open(redis_url)?;
        let redis = ConnectionManager::new(client).await?;

        Ok(Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            executors: Arc::new(RwLock::new(HashMap::new())),
            redis,
        })
    }

    /// Register a job executor
    pub async fn register_executor(&self, executor: Arc<dyn JobExecutor>) {
        let mut executors = self.executors.write().await;
        executors.insert(executor.job_type().to_string(), executor);
    }

    /// Schedule a new job
    pub async fn schedule_job(&self, mut job: Job) -> SchedulerResult<String> {
        // Validate cron expression if applicable
        if let JobSchedule::Cron(ref expr) = job.schedule {
            Schedule::from_str(expr)
                .map_err(|_| SchedulerError::InvalidCronExpression(expr.clone()))?;
        }

        job.status = JobStatus::Scheduled;
        job.update_next_run();

        let job_id = job.id.clone();

        // Persist job to Redis
        self.persist_job(&job).await?;

        // Store in memory
        let mut jobs = self.jobs.write().await;
        jobs.insert(job_id.clone(), job);

        Ok(job_id)
    }

    /// Cancel a scheduled job
    pub async fn cancel_job(&self, job_id: &str) -> SchedulerResult<()> {
        let mut jobs = self.jobs.write().await;

        if let Some(job) = jobs.get_mut(job_id) {
            job.status = JobStatus::Cancelled;
            job.updated_at = Utc::now();

            // Update in Redis
            self.persist_job(job).await?;

            Ok(())
        } else {
            Err(SchedulerError::JobNotFound(job_id.to_string()))
        }
    }

    /// Get job by ID
    pub async fn get_job(&self, job_id: &str) -> Option<Job> {
        let jobs = self.jobs.read().await;
        jobs.get(job_id).cloned()
    }

    /// List all jobs
    pub async fn list_jobs(&self) -> Vec<Job> {
        let jobs = self.jobs.read().await;
        jobs.values().cloned().collect()
    }

    /// List jobs by status
    pub async fn list_jobs_by_status(&self, status: JobStatus) -> Vec<Job> {
        let jobs = self.jobs.read().await;
        jobs.values()
            .filter(|job| job.status == status)
            .cloned()
            .collect()
    }

    /// Get jobs ready to run
    pub async fn get_ready_jobs(&self) -> Vec<Job> {
        let now = Utc::now();
        let jobs = self.jobs.read().await;

        jobs.values()
            .filter(|job| {
                matches!(job.status, JobStatus::Scheduled | JobStatus::Retrying)
                    && job.next_run.map_or(false, |next| next <= now)
            })
            .cloned()
            .collect()
    }

    /// Execute a job with distributed locking
    pub async fn execute_job(&self, job_id: &str) -> SchedulerResult<()> {
        // Acquire distributed lock
        let lock = DistributedLock::acquire(
            self.redis.clone(),
            &format!("job:{}", job_id),
            300, // 5 minutes
        )
        .await?;

        let lock = match lock {
            Some(l) => l,
            None => {
                return Err(SchedulerError::LockAcquisitionFailed(
                    "Job already running".to_string(),
                ))
            }
        };

        // Get job
        let job = {
            let jobs = self.jobs.read().await;
            jobs.get(job_id)
                .cloned()
                .ok_or_else(|| SchedulerError::JobNotFound(job_id.to_string()))?
        };

        // Get executor
        let executor = {
            let executors = self.executors.read().await;
            executors
                .get(&job.job_type)
                .cloned()
                .ok_or_else(|| {
                    SchedulerError::ExecutionError(format!(
                        "No executor found for job type: {}",
                        job.job_type
                    ))
                })?
        };

        // Update job status
        {
            let mut jobs = self.jobs.write().await;
            if let Some(job) = jobs.get_mut(job_id) {
                job.status = JobStatus::Running;
                job.updated_at = Utc::now();
                self.persist_job(job).await?;
            }
        }

        // Execute job with timeout
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(job.timeout_seconds),
            executor.execute(&job),
        )
        .await;

        // Update job based on result
        {
            let mut jobs = self.jobs.write().await;
            if let Some(job) = jobs.get_mut(job_id) {
                match result {
                    Ok(Ok(())) => {
                        job.mark_completed();
                    }
                    Ok(Err(e)) => {
                        job.mark_failed(e.to_string());
                    }
                    Err(_) => {
                        job.mark_failed("Job execution timeout".to_string());
                    }
                }
                self.persist_job(job).await?;
            }
        }

        // Release lock
        lock.release().await?;

        Ok(())
    }

    /// Persist job to Redis
    async fn persist_job(&self, job: &Job) -> SchedulerResult<()> {
        let key = format!("job:{}", job.id);
        let value = serde_json::to_string(job)?;

        redis::cmd("SET")
            .arg(&key)
            .arg(&value)
            .query_async(&mut self.redis.clone())
            .await?;

        Ok(())
    }

    /// Load jobs from Redis
    pub async fn load_jobs(&self) -> SchedulerResult<()> {
        let mut con = self.redis.clone();
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg("job:*")
            .query_async(&mut con)
            .await?;

        let mut jobs = self.jobs.write().await;

        for key in keys {
            let value: String = redis::cmd("GET")
                .arg(&key)
                .query_async(&mut con)
                .await?;

            if let Ok(job) = serde_json::from_str::<Job>(&value) {
                jobs.insert(job.id.clone(), job);
            }
        }

        Ok(())
    }

    /// Run the scheduler loop
    pub async fn run(&self) -> SchedulerResult<()> {
        loop {
            let ready_jobs = self.get_ready_jobs().await;

            for job in ready_jobs {
                let scheduler = self.clone();
                let job_id = job.id.clone();

                // Spawn job execution in background
                tokio::spawn(async move {
                    if let Err(e) = scheduler.execute_job(&job_id).await {
                        eprintln!("Job execution failed: {} - {}", job_id, e);
                    }
                });
            }

            // Sleep for 1 second before next iteration
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
}

impl Clone for JobScheduler {
    fn clone(&self) -> Self {
        Self {
            jobs: Arc::clone(&self.jobs),
            executors: Arc::clone(&self.executors),
            redis: self.redis.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_creation() {
        let schedule = JobSchedule::Once(Utc::now() + Duration::hours(1));
        let job = Job::new("test-job".to_string(), "test".to_string(), schedule);

        assert_eq!(job.name, "test-job");
        assert_eq!(job.job_type, "test");
        assert_eq!(job.status, JobStatus::Pending);
        assert_eq!(job.retry_count, 0);
    }

    #[test]
    fn test_cron_schedule() {
        let schedule = JobSchedule::Cron("0 0 * * *".to_string());
        let next_run = Job::calculate_next_run(&schedule, Utc::now());
        assert!(next_run.is_some());
    }

    #[test]
    fn test_job_retry_logic() {
        let schedule = JobSchedule::Once(Utc::now() + Duration::hours(1));
        let mut job = Job::new("test-job".to_string(), "test".to_string(), schedule);
        job.max_retries = 3;

        job.mark_failed("Test error".to_string());
        assert_eq!(job.status, JobStatus::Retrying);
        assert_eq!(job.retry_count, 1);

        job.mark_failed("Test error 2".to_string());
        assert_eq!(job.retry_count, 2);

        job.mark_failed("Test error 3".to_string());
        assert_eq!(job.status, JobStatus::Failed);
        assert_eq!(job.retry_count, 3);
    }
}
