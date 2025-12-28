use chrono::{DateTime, Datelike, Duration, Timelike, Utc, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum SchedulerError {
    #[error("Schedule not found: {0}")]
    NotFound(Uuid),
    #[error("Invalid cron expression: {0}")]
    InvalidCron(String),
    #[error("Invalid schedule configuration: {0}")]
    InvalidConfiguration(String),
    #[error("Schedule is not active")]
    NotActive,
    #[error("Schedule has expired")]
    Expired,
}

pub type SchedulerResult<T> = Result<T, SchedulerError>;

/// Schedule type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ScheduleType {
    /// One-time execution at specific time
    OneTime {
        /// Execution time
        execute_at: DateTime<Utc>,
    },

    /// Recurring with cron-like expression
    Cron {
        /// Cron expression
        expression: String,
    },

    /// Recurring at fixed interval
    Interval {
        /// Interval in seconds
        interval_seconds: u64,
    },

    /// Daily at specific time
    Daily {
        /// Hour (0-23)
        hour: u32,
        /// Minute (0-59)
        minute: u32,
    },

    /// Weekly on specific days
    Weekly {
        /// Days of week
        days: Vec<Weekday>,
        /// Hour (0-23)
        hour: u32,
        /// Minute (0-59)
        minute: u32,
    },

    /// Monthly on specific day
    Monthly {
        /// Day of month (1-31)
        day: u32,
        /// Hour (0-23)
        hour: u32,
        /// Minute (0-59)
        minute: u32,
    },
}

impl ScheduleType {
    /// Calculate next execution time from a given reference time
    pub fn next_execution(&self, from: DateTime<Utc>) -> Option<DateTime<Utc>> {
        match self {
            ScheduleType::OneTime { execute_at } => {
                if *execute_at > from {
                    Some(*execute_at)
                } else {
                    None
                }
            }

            ScheduleType::Interval { interval_seconds } => {
                Some(from + Duration::seconds(*interval_seconds as i64))
            }

            ScheduleType::Daily { hour, minute } => {
                let mut next = from
                    .date_naive()
                    .and_hms_opt(*hour, *minute, 0)?
                    .and_utc();

                if next <= from {
                    next = (from + Duration::days(1))
                        .date_naive()
                        .and_hms_opt(*hour, *minute, 0)?
                        .and_utc();
                }

                Some(next)
            }

            ScheduleType::Weekly { days, hour, minute } => {
                if days.is_empty() {
                    return None;
                }

                let current_weekday = from.weekday();
                let current_day_num = current_weekday.num_days_from_monday();

                // Find next matching day
                let mut next_day_offset = None;
                for &day in days {
                    let target_day_num = day.num_days_from_monday();
                    let offset = if target_day_num > current_day_num {
                        target_day_num - current_day_num
                    } else {
                        7 + target_day_num - current_day_num
                    };

                    if next_day_offset.is_none() || offset < next_day_offset.unwrap() {
                        next_day_offset = Some(offset);
                    }
                }

                let offset = next_day_offset?;
                let next = (from + Duration::days(offset as i64))
                    .date_naive()
                    .and_hms_opt(*hour, *minute, 0)?
                    .and_utc();

                // If the calculated time is before 'from', we need the next week
                if next <= from {
                    Some(next + Duration::days(7))
                } else {
                    Some(next)
                }
            }

            ScheduleType::Monthly { day, hour, minute } => {
                let mut year = from.year();
                let mut month = from.month();

                // Try current month first
                let mut next = chrono::NaiveDate::from_ymd_opt(year, month, *day)
                    .and_then(|d| d.and_hms_opt(*hour, *minute, 0))
                    .map(|dt| dt.and_utc());

                // If the day doesn't exist in current month or is in the past, go to next month
                if next.is_none() || next.unwrap() <= from {
                    month += 1;
                    if month > 12 {
                        month = 1;
                        year += 1;
                    }

                    next = chrono::NaiveDate::from_ymd_opt(year, month, *day)
                        .and_then(|d| d.and_hms_opt(*hour, *minute, 0))
                        .map(|dt| dt.and_utc());
                }

                next
            }

            ScheduleType::Cron { expression } => {
                // Simplified cron parsing - in production would use a cron library
                // For now, return None to indicate not implemented
                None
            }
        }
    }

    /// Validate the schedule configuration
    pub fn validate(&self) -> SchedulerResult<()> {
        match self {
            ScheduleType::Daily { hour, minute } => {
                if *hour > 23 {
                    return Err(SchedulerError::InvalidConfiguration(
                        "Hour must be 0-23".to_string(),
                    ));
                }
                if *minute > 59 {
                    return Err(SchedulerError::InvalidConfiguration(
                        "Minute must be 0-59".to_string(),
                    ));
                }
            }

            ScheduleType::Weekly { days, hour, minute } => {
                if days.is_empty() {
                    return Err(SchedulerError::InvalidConfiguration(
                        "At least one day must be specified".to_string(),
                    ));
                }
                if *hour > 23 {
                    return Err(SchedulerError::InvalidConfiguration(
                        "Hour must be 0-23".to_string(),
                    ));
                }
                if *minute > 59 {
                    return Err(SchedulerError::InvalidConfiguration(
                        "Minute must be 0-59".to_string(),
                    ));
                }
            }

            ScheduleType::Monthly { day, hour, minute } => {
                if *day < 1 || *day > 31 {
                    return Err(SchedulerError::InvalidConfiguration(
                        "Day must be 1-31".to_string(),
                    ));
                }
                if *hour > 23 {
                    return Err(SchedulerError::InvalidConfiguration(
                        "Hour must be 0-23".to_string(),
                    ));
                }
                if *minute > 59 {
                    return Err(SchedulerError::InvalidConfiguration(
                        "Minute must be 0-59".to_string(),
                    ));
                }
            }

            ScheduleType::Interval { interval_seconds } => {
                if *interval_seconds == 0 {
                    return Err(SchedulerError::InvalidConfiguration(
                        "Interval must be greater than 0".to_string(),
                    ));
                }
            }

            _ => {}
        }

        Ok(())
    }
}

/// Workflow schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSchedule {
    /// Schedule ID
    pub id: Uuid,
    /// Workflow ID to execute
    pub workflow_id: Uuid,
    /// Schedule name
    pub name: String,
    /// Description
    pub description: String,
    /// Schedule type
    pub schedule_type: ScheduleType,
    /// Whether this schedule is active
    pub is_active: bool,
    /// Start date (schedule won't execute before this)
    pub start_date: Option<DateTime<Utc>>,
    /// End date (schedule won't execute after this)
    pub end_date: Option<DateTime<Utc>>,
    /// Last execution time
    pub last_execution: Option<DateTime<Utc>>,
    /// Next scheduled execution
    pub next_execution: Option<DateTime<Utc>>,
    /// Execution count
    pub execution_count: u64,
    /// Success count
    pub success_count: u64,
    /// Failure count
    pub failure_count: u64,
    /// Maximum number of executions (None = unlimited)
    pub max_executions: Option<u64>,
    /// Timezone for schedule (default UTC)
    pub timezone: String,
    /// User who created the schedule
    pub created_by: Uuid,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Execution history (limited to recent entries)
    pub execution_history: Vec<ScheduleExecution>,
    /// Configuration for workflow execution
    pub execution_config: HashMap<String, serde_json::Value>,
}

impl WorkflowSchedule {
    /// Create a new workflow schedule
    pub fn new(
        workflow_id: Uuid,
        name: String,
        schedule_type: ScheduleType,
        created_by: Uuid,
    ) -> SchedulerResult<Self> {
        schedule_type.validate()?;

        let next_execution = schedule_type.next_execution(Utc::now());

        Ok(Self {
            id: Uuid::new_v4(),
            workflow_id,
            name,
            description: String::new(),
            schedule_type,
            is_active: true,
            start_date: None,
            end_date: None,
            last_execution: None,
            next_execution,
            execution_count: 0,
            success_count: 0,
            failure_count: 0,
            max_executions: None,
            timezone: "UTC".to_string(),
            created_by,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            execution_history: Vec::new(),
            execution_config: HashMap::new(),
        })
    }

    /// Check if schedule should execute now
    pub fn should_execute_now(&self) -> bool {
        if !self.is_active {
            return false;
        }

        // Check if max executions reached
        if let Some(max) = self.max_executions {
            if self.execution_count >= max {
                return false;
            }
        }

        // Check start date
        if let Some(start) = self.start_date {
            if Utc::now() < start {
                return false;
            }
        }

        // Check end date
        if let Some(end) = self.end_date {
            if Utc::now() > end {
                return false;
            }
        }

        // Check next execution time
        if let Some(next) = self.next_execution {
            Utc::now() >= next
        } else {
            false
        }
    }

    /// Update next execution time
    pub fn calculate_next_execution(&mut self) {
        if let Some(last) = self.last_execution {
            self.next_execution = self.schedule_type.next_execution(last);
        } else {
            self.next_execution = self.schedule_type.next_execution(Utc::now());
        }
        self.updated_at = Utc::now();
    }

    /// Record successful execution
    pub fn record_execution(&mut self, success: bool, error: Option<String>) {
        self.execution_count += 1;
        if success {
            self.success_count += 1;
        } else {
            self.failure_count += 1;
        }

        self.last_execution = Some(Utc::now());

        // Add to history (keep last 100 executions)
        let execution = ScheduleExecution {
            id: Uuid::new_v4(),
            executed_at: Utc::now(),
            success,
            error_message: error,
            duration_ms: 0, // Would be populated by actual execution
        };

        self.execution_history.push(execution);
        if self.execution_history.len() > 100 {
            self.execution_history.remove(0);
        }

        // Calculate next execution
        self.calculate_next_execution();
    }

    /// Activate the schedule
    pub fn activate(&mut self) {
        self.is_active = true;
        self.calculate_next_execution();
        self.updated_at = Utc::now();
    }

    /// Deactivate the schedule
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.execution_count == 0 {
            return 0.0;
        }
        (self.success_count as f64) / (self.execution_count as f64)
    }

    /// Check if schedule has expired
    pub fn is_expired(&self) -> bool {
        if let Some(end) = self.end_date {
            if Utc::now() > end {
                return true;
            }
        }

        if let Some(max) = self.max_executions {
            if self.execution_count >= max {
                return true;
            }
        }

        false
    }
}

/// Schedule execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleExecution {
    /// Execution ID
    pub id: Uuid,
    /// Execution timestamp
    pub executed_at: DateTime<Utc>,
    /// Success status
    pub success: bool,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
}

/// Scheduler manager for managing multiple schedules
pub struct Scheduler {
    schedules: HashMap<Uuid, WorkflowSchedule>,
    workflow_schedules: HashMap<Uuid, Vec<Uuid>>, // workflow_id -> schedule_ids
}

impl Scheduler {
    /// Create a new scheduler
    pub fn new() -> Self {
        Self {
            schedules: HashMap::new(),
            workflow_schedules: HashMap::new(),
        }
    }

    /// Add a schedule
    pub fn add_schedule(&mut self, schedule: WorkflowSchedule) -> SchedulerResult<()> {
        schedule.schedule_type.validate()?;

        let workflow_id = schedule.workflow_id;
        let schedule_id = schedule.id;

        // Index by workflow
        self.workflow_schedules
            .entry(workflow_id)
            .or_insert_with(Vec::new)
            .push(schedule_id);

        self.schedules.insert(schedule_id, schedule);
        Ok(())
    }

    /// Remove a schedule
    pub fn remove_schedule(&mut self, schedule_id: Uuid) -> SchedulerResult<WorkflowSchedule> {
        let schedule = self
            .schedules
            .remove(&schedule_id)
            .ok_or(SchedulerError::NotFound(schedule_id))?;

        // Remove from workflow index
        if let Some(schedules) = self.workflow_schedules.get_mut(&schedule.workflow_id) {
            schedules.retain(|id| *id != schedule_id);
        }

        Ok(schedule)
    }

    /// Get a schedule
    pub fn get_schedule(&self, schedule_id: Uuid) -> Option<&WorkflowSchedule> {
        self.schedules.get(&schedule_id)
    }

    /// Get a mutable schedule
    pub fn get_schedule_mut(&mut self, schedule_id: Uuid) -> Option<&mut WorkflowSchedule> {
        self.schedules.get_mut(&schedule_id)
    }

    /// Get all schedules for a workflow
    pub fn get_workflow_schedules(&self, workflow_id: Uuid) -> Vec<&WorkflowSchedule> {
        self.workflow_schedules
            .get(&workflow_id)
            .map(|schedule_ids| {
                schedule_ids
                    .iter()
                    .filter_map(|id| self.schedules.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all active schedules
    pub fn get_active_schedules(&self) -> Vec<&WorkflowSchedule> {
        self.schedules
            .values()
            .filter(|s| s.is_active && !s.is_expired())
            .collect()
    }

    /// Get schedules that should execute now
    pub fn get_due_schedules(&self) -> Vec<&WorkflowSchedule> {
        self.schedules
            .values()
            .filter(|s| s.should_execute_now())
            .collect()
    }

    /// Update schedule after execution
    pub fn record_execution(
        &mut self,
        schedule_id: Uuid,
        success: bool,
        error: Option<String>,
    ) -> SchedulerResult<()> {
        let schedule = self
            .schedules
            .get_mut(&schedule_id)
            .ok_or(SchedulerError::NotFound(schedule_id))?;

        schedule.record_execution(success, error);
        Ok(())
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Cron expression parser (simplified)
#[derive(Debug, Clone)]
pub struct CronExpression {
    /// Minute (0-59)
    pub minute: CronField,
    /// Hour (0-23)
    pub hour: CronField,
    /// Day of month (1-31)
    pub day: CronField,
    /// Month (1-12)
    pub month: CronField,
    /// Day of week (0-6, 0 = Sunday)
    pub weekday: CronField,
}

impl CronExpression {
    /// Parse a cron expression
    pub fn parse(expression: &str) -> SchedulerResult<Self> {
        let parts: Vec<&str> = expression.split_whitespace().collect();
        if parts.len() != 5 {
            return Err(SchedulerError::InvalidCron(
                "Expected 5 fields (minute hour day month weekday)".to_string(),
            ));
        }

        Ok(Self {
            minute: CronField::parse(parts[0])?,
            hour: CronField::parse(parts[1])?,
            day: CronField::parse(parts[2])?,
            month: CronField::parse(parts[3])?,
            weekday: CronField::parse(parts[4])?,
        })
    }

    /// Check if a datetime matches this cron expression
    pub fn matches(&self, dt: &DateTime<Utc>) -> bool {
        self.minute.matches(dt.minute())
            && self.hour.matches(dt.hour())
            && self.day.matches(dt.day())
            && self.month.matches(dt.month())
            && self.weekday.matches(dt.weekday().num_days_from_sunday())
    }
}

/// Cron field (simplified)
#[derive(Debug, Clone)]
pub enum CronField {
    /// Any value (*)
    Any,
    /// Specific value
    Value(u32),
    /// List of values
    Values(Vec<u32>),
    /// Range
    Range(u32, u32),
    /// Step
    Step(u32),
}

impl CronField {
    /// Parse a cron field
    fn parse(field: &str) -> SchedulerResult<Self> {
        if field == "*" {
            return Ok(CronField::Any);
        }

        if let Ok(value) = field.parse::<u32>() {
            return Ok(CronField::Value(value));
        }

        // Simplified - would need more complex parsing for ranges, lists, steps
        Ok(CronField::Any)
    }

    /// Check if a value matches this field
    fn matches(&self, value: u32) -> bool {
        match self {
            CronField::Any => true,
            CronField::Value(v) => *v == value,
            CronField::Values(values) => values.contains(&value),
            CronField::Range(start, end) => value >= *start && value <= *end,
            CronField::Step(step) => value % step == 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daily_schedule() {
        let schedule_type = ScheduleType::Daily {
            hour: 9,
            minute: 0,
        };

        assert!(schedule_type.validate().is_ok());

        let now = Utc::now();
        let next = schedule_type.next_execution(now);
        assert!(next.is_some());
    }

    #[test]
    fn test_interval_schedule() {
        let schedule_type = ScheduleType::Interval {
            interval_seconds: 3600, // 1 hour
        };

        let now = Utc::now();
        let next = schedule_type.next_execution(now);
        assert!(next.is_some());

        let next_time = next.unwrap();
        assert!(next_time > now);
    }

    #[test]
    fn test_workflow_schedule_creation() {
        let workflow_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let schedule = WorkflowSchedule::new(
            workflow_id,
            "Daily Report".to_string(),
            ScheduleType::Daily {
                hour: 9,
                minute: 0,
            },
            user_id,
        );

        assert!(schedule.is_ok());
        let schedule = schedule.unwrap();
        assert_eq!(schedule.workflow_id, workflow_id);
        assert!(schedule.is_active);
    }

    #[test]
    fn test_scheduler() {
        let mut scheduler = Scheduler::new();
        let workflow_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let schedule = WorkflowSchedule::new(
            workflow_id,
            "Test Schedule".to_string(),
            ScheduleType::Interval {
                interval_seconds: 60,
            },
            user_id,
        )
        .unwrap();

        let schedule_id = schedule.id;
        assert!(scheduler.add_schedule(schedule).is_ok());

        assert!(scheduler.get_schedule(schedule_id).is_some());

        let workflow_schedules = scheduler.get_workflow_schedules(workflow_id);
        assert_eq!(workflow_schedules.len(), 1);
    }

    #[test]
    fn test_weekly_schedule() {
        let schedule_type = ScheduleType::Weekly {
            days: vec![Weekday::Mon, Weekday::Wed, Weekday::Fri],
            hour: 10,
            minute: 30,
        };

        assert!(schedule_type.validate().is_ok());
    }
}
