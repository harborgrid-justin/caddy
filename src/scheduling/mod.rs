//! Scheduling and monitoring system for CADDY v0.3.0
//!
//! This module provides comprehensive job scheduling, monitoring, and notification capabilities:
//!
//! ## Scheduler
//! - Cron-based recurring schedules
//! - One-time scheduled jobs
//! - Priority-based job queues
//! - Distributed job locking via Redis
//! - Job persistence and recovery
//!
//! ## Queue
//! - Redis-backed priority queue
//! - Job deduplication
//! - Job cancellation and progress tracking
//! - Delayed job execution
//! - Dead letter queue for failed jobs
//!
//! ## Worker Pool
//! - Worker pool management
//! - Task distribution and load balancing
//! - Failure handling with exponential backoff
//! - Worker health monitoring
//! - Auto-scaling capabilities
//!
//! ## Monitoring
//! - Continuous monitoring mode
//! - Change detection and diffing
//! - Regression alerts
//! - Uptime tracking
//! - Performance metrics collection
//!
//! ## Notifications
//! - Email notifications
//! - Slack integration
//! - Microsoft Teams integration
//! - Custom webhook notifications
//! - User preferences and quiet hours
//!
//! # Examples
//!
//! ## Scheduling a recurring job
//!
//! ```rust,no_run
//! use caddy::scheduling::scheduler::{Job, JobSchedule, JobScheduler};
//! use chrono::Utc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let scheduler = JobScheduler::new("redis://localhost").await?;
//!
//! // Create a cron-based job
//! let job = Job::new(
//!     "daily-scan".to_string(),
//!     "accessibility-scan".to_string(),
//!     JobSchedule::Cron("0 0 * * *".to_string()), // Daily at midnight
//! );
//!
//! let job_id = scheduler.schedule_job(job).await?;
//! println!("Scheduled job: {}", job_id);
//! # Ok(())
//! # }
//! ```
//!
//! ## Setting up a worker pool
//!
//! ```rust,no_run
//! use caddy::scheduling::worker::WorkerPool;
//! use caddy::scheduling::queue::JobQueue;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let queue = Arc::new(JobQueue::new("redis://localhost").await?);
//! let redis = redis::Client::open("redis://localhost")?
//!     .get_tokio_connection_manager().await?;
//!
//! let pool = WorkerPool::new(queue, redis).await;
//!
//! // Add 5 workers
//! pool.scale(5, vec!["default".to_string(), "scans".to_string()]).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Monitoring a website
//!
//! ```rust,no_run
//! use caddy::scheduling::monitor::{Monitor, MonitoringSystem, CheckType};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let monitoring = MonitoringSystem::new("redis://localhost").await?;
//!
//! // Create HTTP monitor
//! let monitor = Monitor::new(
//!     "Production Site".to_string(),
//!     CheckType::Http {
//!         url: "https://example.com".to_string(),
//!         method: "GET".to_string(),
//!         expected_status: 200,
//!         timeout_ms: 5000,
//!     },
//!     60, // Check every 60 seconds
//! );
//!
//! let monitor_id = monitoring.add_monitor(monitor).await?;
//! println!("Created monitor: {}", monitor_id);
//! # Ok(())
//! # }
//! ```
//!
//! ## Sending notifications
//!
//! ```rust,no_run
//! use caddy::scheduling::notifications::{
//!     Notification, NotificationService, NotificationSeverity, SlackConfig, SlackDelivery
//! };
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let service = NotificationService::new();
//!
//! // Configure Slack
//! let slack_config = SlackConfig {
//!     webhook_url: "https://hooks.slack.com/services/YOUR/WEBHOOK/URL".to_string(),
//!     channel: Some("#alerts".to_string()),
//!     username: Some("CADDY Bot".to_string()),
//!     icon_emoji: Some(":robot_face:".to_string()),
//! };
//!
//! service.register_channel(Arc::new(SlackDelivery::new(slack_config))).await;
//!
//! // Send notification
//! let notification = Notification::new(
//!     "Site Down".to_string(),
//!     "Production site is not responding".to_string(),
//!     "monitor-system".to_string(),
//! ).with_severity(NotificationSeverity::Critical);
//!
//! service.send(notification).await?;
//! # Ok(())
//! # }
//! ```

pub mod scheduler;
pub mod queue;
pub mod worker;
pub mod monitor;
pub mod notifications;

// Re-export commonly used types
pub use scheduler::{
    Job, JobExecutor, JobPriority, JobSchedule, JobScheduler, JobStatus, SchedulerError,
    SchedulerResult,
};

pub use queue::{
    JobProgress, JobQueue, QueueError, QueueResult, QueueStats, QueuedJob,
};

pub use worker::{
    PoolStats, TaskHandler, Worker, WorkerError, WorkerHealth, WorkerPool, WorkerResult,
    WorkerStatus,
};

pub use monitor::{
    AlertSeverity, ChangeDetection, CheckResult, CheckType, Monitor, MonitorAlert, MonitorError,
    MonitorResult, MonitorStatus, MonitoringSystem, PerformanceMetrics, UptimeStats,
};

pub use notifications::{
    EmailConfig, EmailDelivery, Notification, NotificationChannel, NotificationDelivery,
    NotificationError, NotificationPreferences, NotificationPriority, NotificationResult,
    NotificationSeverity, NotificationService, QuietHours, SlackConfig, SlackDelivery,
    TeamsConfig, TeamsDelivery, WebhookConfig, WebhookDelivery,
};
