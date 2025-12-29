//! Background worker system with pool management and task distribution
//!
//! This module provides:
//! - Worker pool management
//! - Task distribution and load balancing
//! - Failure handling and retries
//! - Dead letter queue integration
//! - Worker health checks and monitoring

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{RwLock, Semaphore};
use uuid::Uuid;

use crate::scheduling::queue::{JobQueue, QueuedJob};

/// Worker errors
#[derive(Error, Debug)]
pub enum WorkerError {
    #[error("Worker pool error: {0}")]
    PoolError(String),

    #[error("Task execution error: {0}")]
    TaskError(String),

    #[error("Worker not found: {0}")]
    WorkerNotFound(String),

    #[error("Worker unhealthy: {0}")]
    WorkerUnhealthy(String),

    #[error("Queue error: {0}")]
    QueueError(String),

    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("Shutdown in progress")]
    ShuttingDown,
}

/// Result type for worker operations
pub type WorkerResult<T> = Result<T, WorkerError>;

/// Worker status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerStatus {
    Idle,
    Busy,
    Unhealthy,
    Shutdown,
}

/// Worker health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerHealth {
    pub worker_id: String,
    pub status: WorkerStatus,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub last_heartbeat: DateTime<Utc>,
    pub uptime_seconds: u64,
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f64,
}

impl WorkerHealth {
    /// Create new worker health
    pub fn new(worker_id: String) -> Self {
        Self {
            worker_id,
            status: WorkerStatus::Idle,
            tasks_completed: 0,
            tasks_failed: 0,
            last_heartbeat: Utc::now(),
            uptime_seconds: 0,
            memory_usage_mb: 0,
            cpu_usage_percent: 0.0,
        }
    }

    /// Check if worker is healthy
    pub fn is_healthy(&self) -> bool {
        let now = Utc::now();
        let last_heartbeat = self.last_heartbeat;
        let elapsed = (now - last_heartbeat).num_seconds();

        // Worker is unhealthy if no heartbeat in last 60 seconds
        elapsed < 60 && self.status != WorkerStatus::Unhealthy
    }
}

/// Task handler trait
#[async_trait]
pub trait TaskHandler: Send + Sync {
    /// Handle a task
    async fn handle(&self, job: &QueuedJob) -> WorkerResult<()>;

    /// Get the task type this handler supports
    fn task_type(&self) -> &str;
}

/// Background worker
pub struct Worker {
    id: String,
    queue_names: Vec<String>,
    handlers: Arc<RwLock<HashMap<String, Arc<dyn TaskHandler>>>>,
    health: Arc<RwLock<WorkerHealth>>,
    is_running: Arc<AtomicBool>,
    tasks_completed: Arc<AtomicU64>,
    tasks_failed: Arc<AtomicU64>,
    queue: Arc<JobQueue>,
    redis: ConnectionManager,
    max_concurrent_tasks: usize,
}

impl Worker {
    /// Create a new worker
    pub async fn new(
        queue_names: Vec<String>,
        queue: Arc<JobQueue>,
        redis: ConnectionManager,
        max_concurrent_tasks: usize,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        let health = Arc::new(RwLock::new(WorkerHealth::new(id.clone())));

        Self {
            id,
            queue_names,
            handlers: Arc::new(RwLock::new(HashMap::new())),
            health,
            is_running: Arc::new(AtomicBool::new(false)),
            tasks_completed: Arc::new(AtomicU64::new(0)),
            tasks_failed: Arc::new(AtomicU64::new(0)),
            queue,
            redis,
            max_concurrent_tasks,
        }
    }

    /// Register a task handler
    pub async fn register_handler(&self, handler: Arc<dyn TaskHandler>) {
        let mut handlers = self.handlers.write().await;
        handlers.insert(handler.task_type().to_string(), handler);
    }

    /// Get worker ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get worker health
    pub async fn health(&self) -> WorkerHealth {
        self.health.read().await.clone()
    }

    /// Start the worker
    pub async fn start(&self) -> WorkerResult<()> {
        if self.is_running.load(Ordering::SeqCst) {
            return Err(WorkerError::PoolError("Worker already running".to_string()));
        }

        self.is_running.store(true, Ordering::SeqCst);

        // Update health status
        {
            let mut health = self.health.write().await;
            health.status = WorkerStatus::Idle;
            health.last_heartbeat = Utc::now();
        }

        // Register worker in Redis
        self.register_worker().await?;

        // Start heartbeat task
        let worker_id = self.id.clone();
        let health = Arc::clone(&self.health);
        let redis = self.redis.clone();
        let is_running = Arc::clone(&self.is_running);

        tokio::spawn(async move {
            Self::heartbeat_loop(worker_id, health, redis, is_running).await;
        });

        // Start main worker loop
        self.run_loop().await?;

        Ok(())
    }

    /// Stop the worker
    pub async fn stop(&self) -> WorkerResult<()> {
        self.is_running.store(false, Ordering::SeqCst);

        // Update health status
        {
            let mut health = self.health.write().await;
            health.status = WorkerStatus::Shutdown;
        }

        // Deregister worker from Redis
        self.deregister_worker().await?;

        Ok(())
    }

    /// Main worker loop
    async fn run_loop(&self) -> WorkerResult<()> {
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent_tasks));

        while self.is_running.load(Ordering::SeqCst) {
            // Try to get a job from any of the queues
            let job = self.try_dequeue_job().await?;

            if let Some(job) = job {
                // Acquire semaphore permit
                let permit = semaphore.clone().acquire_owned().await.unwrap();

                // Process job in background
                let worker = self.clone_worker();
                tokio::spawn(async move {
                    worker.process_job(job).await;
                    drop(permit);
                });
            } else {
                // No jobs available, sleep briefly
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }

        Ok(())
    }

    /// Try to dequeue a job from any queue
    async fn try_dequeue_job(&self) -> WorkerResult<Option<QueuedJob>> {
        for queue_name in &self.queue_names {
            match self.queue.dequeue(queue_name).await {
                Ok(Some(job)) => return Ok(Some(job)),
                Ok(None) => continue,
                Err(e) => {
                    eprintln!("Error dequeuing from {}: {}", queue_name, e);
                    continue;
                }
            }
        }
        Ok(None)
    }

    /// Process a job
    async fn process_job(&self, job: QueuedJob) {
        // Update health status
        {
            let mut health = self.health.write().await;
            health.status = WorkerStatus::Busy;
        }

        // Get handler for job type
        let handler = {
            let handlers = self.handlers.read().await;
            handlers.get(&job.job_type).cloned()
        };

        let result = match handler {
            Some(h) => {
                // Execute task with timeout
                tokio::time::timeout(
                    tokio::time::Duration::from_secs(job.timeout_seconds),
                    h.handle(&job),
                )
                .await
            }
            None => {
                let error = format!("No handler found for job type: {}", job.job_type);
                Err(WorkerError::TaskError(error))
            }
        };

        // Handle result
        match result {
            Ok(Ok(())) => {
                // Task completed successfully
                self.tasks_completed.fetch_add(1, Ordering::SeqCst);
                if let Err(e) = self.queue.complete_job(&job.id).await {
                    eprintln!("Error completing job {}: {}", job.id, e);
                }
            }
            Ok(Err(e)) => {
                // Task failed
                self.tasks_failed.fetch_add(1, Ordering::SeqCst);
                let error_msg = e.to_string();

                if let Err(e) = self.queue.fail_job(&job.id, error_msg).await {
                    eprintln!("Error failing job {}: {}", job.id, e);
                }
            }
            Err(_) => {
                // Task timeout
                self.tasks_failed.fetch_add(1, Ordering::SeqCst);
                let error_msg = "Task timeout".to_string();

                if let Err(e) = self.queue.fail_job(&job.id, error_msg).await {
                    eprintln!("Error failing job {}: {}", job.id, e);
                }
            }
        }

        // Update health status
        {
            let mut health = self.health.write().await;
            health.status = WorkerStatus::Idle;
            health.tasks_completed = self.tasks_completed.load(Ordering::SeqCst);
            health.tasks_failed = self.tasks_failed.load(Ordering::SeqCst);
        }
    }

    /// Heartbeat loop
    async fn heartbeat_loop(
        worker_id: String,
        health: Arc<RwLock<WorkerHealth>>,
        mut redis: ConnectionManager,
        is_running: Arc<AtomicBool>,
    ) {
        let start_time = Utc::now();

        while is_running.load(Ordering::SeqCst) {
            // Update health metrics
            {
                let mut h = health.write().await;
                h.last_heartbeat = Utc::now();
                h.uptime_seconds = (Utc::now() - start_time).num_seconds() as u64;

                // Update system metrics (simplified)
                h.memory_usage_mb = Self::get_memory_usage();
                h.cpu_usage_percent = Self::get_cpu_usage();
            }

            // Send heartbeat to Redis
            let health_data = {
                let h = health.read().await;
                serde_json::to_string(&*h).unwrap_or_default()
            };

            let heartbeat_key = format!("worker:{}:heartbeat", worker_id);
            let _: Result<(), _> = redis::cmd("SET")
                .arg(&heartbeat_key)
                .arg(&health_data)
                .arg("EX")
                .arg(120) // Expire after 2 minutes
                .query_async(&mut redis)
                .await;

            // Sleep for 30 seconds
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
    }

    /// Get memory usage (simplified)
    fn get_memory_usage() -> u64 {
        // In production, use a proper system metrics library
        0
    }

    /// Get CPU usage (simplified)
    fn get_cpu_usage() -> f64 {
        // In production, use a proper system metrics library
        0.0
    }

    /// Register worker in Redis
    async fn register_worker(&self) -> WorkerResult<()> {
        let worker_key = format!("worker:{}", self.id);
        let worker_data = serde_json::to_string(&self.queue_names).unwrap_or_default();

        redis::cmd("SET")
            .arg(&worker_key)
            .arg(&worker_data)
            .query_async(&mut self.redis.clone())
            .await?;

        // Add to workers set
        redis::cmd("SADD")
            .arg("workers")
            .arg(&self.id)
            .query_async(&mut self.redis.clone())
            .await?;

        Ok(())
    }

    /// Deregister worker from Redis
    async fn deregister_worker(&self) -> WorkerResult<()> {
        let worker_key = format!("worker:{}", self.id);

        redis::cmd("DEL")
            .arg(&worker_key)
            .query_async(&mut self.redis.clone())
            .await?;

        // Remove from workers set
        redis::cmd("SREM")
            .arg("workers")
            .arg(&self.id)
            .query_async(&mut self.redis.clone())
            .await?;

        Ok(())
    }

    /// Clone worker for spawning tasks
    fn clone_worker(&self) -> Self {
        Self {
            id: self.id.clone(),
            queue_names: self.queue_names.clone(),
            handlers: Arc::clone(&self.handlers),
            health: Arc::clone(&self.health),
            is_running: Arc::clone(&self.is_running),
            tasks_completed: Arc::clone(&self.tasks_completed),
            tasks_failed: Arc::clone(&self.tasks_failed),
            queue: Arc::clone(&self.queue),
            redis: self.redis.clone(),
            max_concurrent_tasks: self.max_concurrent_tasks,
        }
    }
}

/// Worker pool manager
pub struct WorkerPool {
    workers: Arc<RwLock<HashMap<String, Arc<Worker>>>>,
    queue: Arc<JobQueue>,
    redis: ConnectionManager,
}

impl WorkerPool {
    /// Create a new worker pool
    pub async fn new(queue: Arc<JobQueue>, redis: ConnectionManager) -> Self {
        Self {
            workers: Arc::new(RwLock::new(HashMap::new())),
            queue,
            redis,
        }
    }

    /// Add a worker to the pool
    pub async fn add_worker(
        &self,
        queue_names: Vec<String>,
        max_concurrent_tasks: usize,
    ) -> WorkerResult<String> {
        let worker = Arc::new(
            Worker::new(
                queue_names,
                Arc::clone(&self.queue),
                self.redis.clone(),
                max_concurrent_tasks,
            )
            .await,
        );

        let worker_id = worker.id().to_string();

        let mut workers = self.workers.write().await;
        workers.insert(worker_id.clone(), worker);

        Ok(worker_id)
    }

    /// Start a worker
    pub async fn start_worker(&self, worker_id: &str) -> WorkerResult<()> {
        let worker = {
            let workers = self.workers.read().await;
            workers
                .get(worker_id)
                .cloned()
                .ok_or_else(|| WorkerError::WorkerNotFound(worker_id.to_string()))?
        };

        // Start worker in background
        let worker_clone = Arc::clone(&worker);
        tokio::spawn(async move {
            if let Err(e) = worker_clone.start().await {
                eprintln!("Worker {} error: {}", worker_clone.id(), e);
            }
        });

        Ok(())
    }

    /// Stop a worker
    pub async fn stop_worker(&self, worker_id: &str) -> WorkerResult<()> {
        let worker = {
            let workers = self.workers.read().await;
            workers
                .get(worker_id)
                .cloned()
                .ok_or_else(|| WorkerError::WorkerNotFound(worker_id.to_string()))?
        };

        worker.stop().await?;

        Ok(())
    }

    /// Register a task handler with all workers
    pub async fn register_handler(&self, handler: Arc<dyn TaskHandler>) {
        let workers = self.workers.read().await;
        for worker in workers.values() {
            worker.register_handler(Arc::clone(&handler)).await;
        }
    }

    /// Get worker health
    pub async fn worker_health(&self, worker_id: &str) -> WorkerResult<WorkerHealth> {
        let workers = self.workers.read().await;
        let worker = workers
            .get(worker_id)
            .ok_or_else(|| WorkerError::WorkerNotFound(worker_id.to_string()))?;

        Ok(worker.health().await)
    }

    /// Get all workers health
    pub async fn all_workers_health(&self) -> Vec<WorkerHealth> {
        let workers = self.workers.read().await;
        let mut healths = Vec::new();

        for worker in workers.values() {
            healths.push(worker.health().await);
        }

        healths
    }

    /// Scale worker pool
    pub async fn scale(&self, target_workers: usize, queue_names: Vec<String>) -> WorkerResult<()> {
        let current_count = {
            let workers = self.workers.read().await;
            workers.len()
        };

        if target_workers > current_count {
            // Scale up
            let to_add = target_workers - current_count;
            for _ in 0..to_add {
                let worker_id = self.add_worker(queue_names.clone(), 10).await?;
                self.start_worker(&worker_id).await?;
            }
        } else if target_workers < current_count {
            // Scale down
            let to_remove = current_count - target_workers;
            let worker_ids: Vec<String> = {
                let workers = self.workers.read().await;
                workers.keys().take(to_remove).cloned().collect()
            };

            for worker_id in worker_ids {
                self.stop_worker(&worker_id).await?;
                let mut workers = self.workers.write().await;
                workers.remove(&worker_id);
            }
        }

        Ok(())
    }

    /// Get pool statistics
    pub async fn stats(&self) -> PoolStats {
        let workers = self.workers.read().await;
        let total_workers = workers.len();

        let mut active_workers = 0;
        let mut total_tasks_completed = 0;
        let mut total_tasks_failed = 0;

        for worker in workers.values() {
            let health = worker.health().await;
            if health.status == WorkerStatus::Busy {
                active_workers += 1;
            }
            total_tasks_completed += health.tasks_completed;
            total_tasks_failed += health.tasks_failed;
        }

        PoolStats {
            total_workers,
            active_workers,
            idle_workers: total_workers - active_workers,
            total_tasks_completed,
            total_tasks_failed,
        }
    }
}

/// Worker pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    pub total_workers: usize,
    pub active_workers: usize,
    pub idle_workers: usize,
    pub total_tasks_completed: u64,
    pub total_tasks_failed: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_health() {
        let health = WorkerHealth::new("worker-1".to_string());
        assert_eq!(health.worker_id, "worker-1");
        assert_eq!(health.status, WorkerStatus::Idle);
        assert!(health.is_healthy());
    }

    #[test]
    fn test_pool_stats() {
        let stats = PoolStats {
            total_workers: 10,
            active_workers: 3,
            idle_workers: 7,
            total_tasks_completed: 1000,
            total_tasks_failed: 10,
        };

        assert_eq!(stats.total_workers, 10);
        assert_eq!(stats.active_workers, 3);
    }
}
