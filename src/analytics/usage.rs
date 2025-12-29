//! # Usage Analytics Tracking
//!
//! Track user behavior, feature usage, and application interactions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use chrono::{DateTime, Utc, Duration};

/// Usage event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum EventType {
    /// Application start
    AppStart,
    /// Application stop
    AppStop,
    /// Feature used
    FeatureUsed,
    /// Command executed
    CommandExecuted,
    /// File opened
    FileOpened,
    /// File saved
    FileSaved,
    /// Entity created
    EntityCreated,
    /// Entity modified
    EntityModified,
    /// Entity deleted
    EntityDeleted,
    /// Tool activated
    ToolActivated,
    /// Layer created
    LayerCreated,
    /// View changed
    ViewChanged,
    /// Rendering mode changed
    RenderingModeChanged,
    /// Export operation
    Export,
    /// Import operation
    Import,
    /// Error occurred
    Error,
    /// Custom event
    Custom,
}

/// Usage event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    /// Event ID
    pub id: String,

    /// Event type
    pub event_type: EventType,

    /// Event name
    pub name: String,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// User ID (optional, for multi-user analytics)
    pub user_id: Option<String>,

    /// Session ID
    pub session_id: String,

    /// Properties
    pub properties: HashMap<String, serde_json::Value>,

    /// Duration (for events with duration)
    pub duration_ms: Option<f64>,
}

impl UsageEvent {
    /// Create a new usage event
    pub fn new(event_type: EventType, name: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            name: name.into(),
            timestamp: Utc::now(),
            user_id: None,
            session_id: String::new(),
            properties: HashMap::new(),
            duration_ms: None,
        }
    }

    /// Set user ID
    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set session ID
    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = session_id.into();
        self
    }

    /// Add a property
    pub fn with_property(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.properties.insert(key.into(), value);
        self
    }

    /// Set duration
    pub fn with_duration(mut self, duration_ms: f64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }
}

/// Usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    /// Total events
    pub total_events: u64,

    /// Events by type
    pub events_by_type: HashMap<EventType, u64>,

    /// Active users (unique user IDs)
    pub active_users: usize,

    /// Active sessions
    pub active_sessions: usize,

    /// Total session duration in seconds
    pub total_session_duration_secs: u64,

    /// Average session duration in seconds
    pub avg_session_duration_secs: f64,

    /// Most used features
    pub most_used_features: Vec<(String, u64)>,

    /// Most executed commands
    pub most_executed_commands: Vec<(String, u64)>,

    /// Error rate
    pub error_rate: f64,

    /// First event timestamp
    pub first_event: Option<DateTime<Utc>>,

    /// Last event timestamp
    pub last_event: Option<DateTime<Utc>>,
}

impl Default for UsageStats {
    fn default() -> Self {
        Self {
            total_events: 0,
            events_by_type: HashMap::new(),
            active_users: 0,
            active_sessions: 0,
            total_session_duration_secs: 0,
            avg_session_duration_secs: 0.0,
            most_used_features: Vec::new(),
            most_executed_commands: Vec::new(),
            error_rate: 0.0,
            first_event: None,
            last_event: None,
        }
    }
}

/// Session tracking
#[derive(Debug, Clone)]
struct Session {
    id: String,
    user_id: Option<String>,
    start_time: DateTime<Utc>,
    last_activity: DateTime<Utc>,
    event_count: u64,
}

impl Session {
    fn new(id: String, user_id: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id,
            user_id,
            start_time: now,
            last_activity: now,
            event_count: 0,
        }
    }

    fn update_activity(&mut self) {
        self.last_activity = Utc::now();
        self.event_count += 1;
    }

    fn duration_secs(&self) -> u64 {
        (self.last_activity - self.start_time).num_seconds() as u64
    }

    fn is_active(&self, timeout_minutes: i64) -> bool {
        (Utc::now() - self.last_activity).num_minutes() < timeout_minutes
    }
}

/// Usage tracker
pub struct UsageTracker {
    /// Enable/disable tracking
    enabled: Arc<std::sync::atomic::AtomicBool>,

    /// Current session ID
    current_session_id: Arc<RwLock<String>>,

    /// Active sessions
    sessions: Arc<RwLock<HashMap<String, Session>>>,

    /// Event history
    events: Arc<RwLock<Vec<UsageEvent>>>,

    /// Event counters
    event_counters: Arc<RwLock<HashMap<EventType, u64>>>,

    /// Feature usage counters
    feature_counters: Arc<RwLock<HashMap<String, u64>>>,

    /// Command usage counters
    command_counters: Arc<RwLock<HashMap<String, u64>>>,

    /// User IDs seen
    users: Arc<RwLock<std::collections::HashSet<String>>>,

    /// Maximum event history size
    max_history: usize,

    /// Session timeout in minutes
    session_timeout_minutes: i64,
}

impl UsageTracker {
    /// Create a new usage tracker
    pub fn new(enabled: bool) -> Self {
        let session_id = uuid::Uuid::new_v4().to_string();

        Self {
            enabled: Arc::new(std::sync::atomic::AtomicBool::new(enabled)),
            current_session_id: Arc::new(RwLock::new(session_id.clone())),
            sessions: Arc::new(RwLock::new({
                let mut sessions = HashMap::new();
                sessions.insert(session_id.clone(), Session::new(session_id, None));
                sessions
            })),
            events: Arc::new(RwLock::new(Vec::new())),
            event_counters: Arc::new(RwLock::new(HashMap::new())),
            feature_counters: Arc::new(RwLock::new(HashMap::new())),
            command_counters: Arc::new(RwLock::new(HashMap::new())),
            users: Arc::new(RwLock::new(std::collections::HashSet::new())),
            max_history: 100000,
            session_timeout_minutes: 30,
        }
    }

    /// Enable tracking
    pub fn enable(&self) {
        self.enabled.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// Disable tracking
    pub fn disable(&self) {
        self.enabled.store(false, std::sync::atomic::Ordering::Relaxed);
    }

    /// Check if tracking is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Track an event
    pub fn track(&self, mut event: UsageEvent) {
        if !self.is_enabled() {
            return;
        }

        // Set session ID if not set
        if event.session_id.is_empty() {
            event.session_id = self.current_session_id.read().clone();
        }

        // Update session activity
        if let Some(session) = self.sessions.write().get_mut(&event.session_id) {
            session.update_activity();
        }

        // Track user
        if let Some(user_id) = &event.user_id {
            self.users.write().insert(user_id.clone());
        }

        // Update counters
        *self.event_counters.write().entry(event.event_type).or_insert(0) += 1;

        // Track feature usage
        if event.event_type == EventType::FeatureUsed {
            *self.feature_counters.write().entry(event.name.clone()).or_insert(0) += 1;
        }

        // Track command execution
        if event.event_type == EventType::CommandExecuted {
            *self.command_counters.write().entry(event.name.clone()).or_insert(0) += 1;
        }

        // Add to history
        let mut events = self.events.write();
        events.push(event);

        // Trim history if needed
        if events.len() > self.max_history {
            events.drain(0..events.len() - self.max_history);
        }
    }

    /// Track a simple event
    pub fn track_simple(&self, event_type: EventType, name: impl Into<String>) {
        let event = UsageEvent::new(event_type, name);
        self.track(event);
    }

    /// Track feature usage
    pub fn track_feature(&self, feature_name: impl Into<String>) {
        self.track_simple(EventType::FeatureUsed, feature_name);
    }

    /// Track command execution
    pub fn track_command(
        &self,
        command_name: impl Into<String>,
        duration_ms: f64,
        success: bool,
    ) {
        let mut event = UsageEvent::new(EventType::CommandExecuted, command_name)
            .with_duration(duration_ms)
            .with_property("success", serde_json::json!(success));

        if !success {
            self.track(UsageEvent::new(EventType::Error, "command_failed"));
        }

        self.track(event);
    }

    /// Start a new session
    pub fn start_session(&self, user_id: Option<String>) -> String {
        let session_id = uuid::Uuid::new_v4().to_string();
        let session = Session::new(session_id.clone(), user_id);

        self.sessions.write().insert(session_id.clone(), session);
        *self.current_session_id.write() = session_id.clone();

        self.track_simple(EventType::AppStart, "session_started");

        session_id
    }

    /// End current session
    pub fn end_session(&self) {
        self.track_simple(EventType::AppStop, "session_ended");

        let session_id = self.current_session_id.read().clone();
        self.sessions.write().remove(&session_id);
    }

    /// Clean up inactive sessions
    pub fn cleanup_sessions(&self) {
        let mut sessions = self.sessions.write();
        sessions.retain(|_, session| session.is_active(self.session_timeout_minutes));
    }

    /// Get usage statistics
    pub fn statistics(&self) -> UsageStats {
        let events = self.events.read();
        let event_counters = self.event_counters.read();
        let feature_counters = self.feature_counters.read();
        let command_counters = self.command_counters.read();
        let sessions = self.sessions.read();

        let total_events = events.len() as u64;

        // Calculate session duration
        let total_session_duration_secs: u64 = sessions.values().map(|s| s.duration_secs()).sum();
        let avg_session_duration_secs = if !sessions.is_empty() {
            total_session_duration_secs as f64 / sessions.len() as f64
        } else {
            0.0
        };

        // Most used features
        let mut most_used_features: Vec<(String, u64)> = feature_counters
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        most_used_features.sort_by(|a, b| b.1.cmp(&a.1));
        most_used_features.truncate(10);

        // Most executed commands
        let mut most_executed_commands: Vec<(String, u64)> = command_counters
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        most_executed_commands.sort_by(|a, b| b.1.cmp(&a.1));
        most_executed_commands.truncate(10);

        // Error rate
        let error_count = event_counters.get(&EventType::Error).copied().unwrap_or(0);
        let error_rate = if total_events > 0 {
            error_count as f64 / total_events as f64
        } else {
            0.0
        };

        // First and last events
        let first_event = events.first().map(|e| e.timestamp);
        let last_event = events.last().map(|e| e.timestamp);

        UsageStats {
            total_events,
            events_by_type: event_counters.clone(),
            active_users: self.users.read().len(),
            active_sessions: sessions.len(),
            total_session_duration_secs,
            avg_session_duration_secs,
            most_used_features,
            most_executed_commands,
            error_rate,
            first_event,
            last_event,
        }
    }

    /// Get events for a time range
    pub fn events_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<UsageEvent> {
        self.events
            .read()
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .cloned()
            .collect()
    }

    /// Get events by type
    pub fn events_by_type(&self, event_type: EventType) -> Vec<UsageEvent> {
        self.events
            .read()
            .iter()
            .filter(|e| e.event_type == event_type)
            .cloned()
            .collect()
    }

    /// Get feature usage over time
    pub fn feature_usage_timeline(&self, feature_name: &str) -> Vec<(DateTime<Utc>, u64)> {
        let events = self.events.read();
        let mut timeline: HashMap<DateTime<Utc>, u64> = HashMap::new();

        for event in events.iter() {
            if event.event_type == EventType::FeatureUsed && event.name == feature_name {
                // Round to hour
                let hour = event.timestamp
                    .with_minute(0)
                    .unwrap()
                    .with_second(0)
                    .unwrap()
                    .with_nanosecond(0)
                    .unwrap();

                *timeline.entry(hour).or_insert(0) += 1;
            }
        }

        let mut result: Vec<_> = timeline.into_iter().collect();
        result.sort_by_key(|(ts, _)| *ts);
        result
    }

    /// Get user retention (users active in consecutive periods)
    pub fn user_retention(&self, period_days: i64) -> Vec<(DateTime<Utc>, usize)> {
        // Simplified retention calculation
        let events = self.events.read();
        let mut retention: HashMap<DateTime<Utc>, std::collections::HashSet<String>> = HashMap::new();

        for event in events.iter() {
            if let Some(user_id) = &event.user_id {
                let period_start = event.timestamp.date_naive();
                let period_key = DateTime::from_timestamp(
                    period_start.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp(),
                    0,
                )
                .unwrap();

                retention
                    .entry(period_key)
                    .or_insert_with(std::collections::HashSet::new)
                    .insert(user_id.clone());
            }
        }

        let mut result: Vec<_> = retention
            .into_iter()
            .map(|(ts, users)| (ts, users.len()))
            .collect();
        result.sort_by_key(|(ts, _)| *ts);
        result
    }

    /// Clear all tracking data
    pub fn clear(&self) {
        self.events.write().clear();
        self.event_counters.write().clear();
        self.feature_counters.write().clear();
        self.command_counters.write().clear();
        self.users.write().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_event_creation() {
        let event = UsageEvent::new(EventType::FeatureUsed, "test_feature")
            .with_property("key", serde_json::json!("value"));

        assert_eq!(event.event_type, EventType::FeatureUsed);
        assert_eq!(event.name, "test_feature");
        assert!(event.properties.contains_key("key"));
    }

    #[test]
    fn test_usage_tracker_creation() {
        let tracker = UsageTracker::new(true);
        assert!(tracker.is_enabled());
    }

    #[test]
    fn test_track_event() {
        let tracker = UsageTracker::new(true);
        tracker.track_feature("test_feature");

        let stats = tracker.statistics();
        assert_eq!(stats.total_events, 1);
    }

    #[test]
    fn test_track_command() {
        let tracker = UsageTracker::new(true);
        tracker.track_command("draw_line", 150.5, true);

        let stats = tracker.statistics();
        assert!(stats.total_events > 0);
    }

    #[test]
    fn test_session_management() {
        let tracker = UsageTracker::new(true);
        let session_id = tracker.start_session(Some("user123".to_string()));

        assert!(!session_id.is_empty());

        tracker.end_session();
        let stats = tracker.statistics();
        assert!(stats.active_sessions < 2);
    }

    #[test]
    fn test_statistics() {
        let tracker = UsageTracker::new(true);

        tracker.track_feature("feature1");
        tracker.track_feature("feature2");
        tracker.track_feature("feature1");
        tracker.track_command("cmd1", 100.0, true);

        let stats = tracker.statistics();
        assert_eq!(stats.total_events, 4);
        assert!(!stats.most_used_features.is_empty());
    }
}
