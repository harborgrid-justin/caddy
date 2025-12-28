//! Audit query engine
//!
//! Provides powerful querying capabilities for audit logs.

use crate::enterprise::audit::{
    event::{AuditEvent, EventSeverity, EventType},
    storage::{AuditStorage, StorageError},
};
use chrono::{DateTime, Duration, Utc};
use std::collections::HashSet;
use thiserror::Error;

/// Errors that can occur during query operations
#[derive(Debug, Error)]
pub enum QueryError {
    /// Storage error
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    /// Invalid query parameter
    #[error("Invalid query: {0}")]
    Invalid(String),
}

/// Result type for query operations
pub type Result<T> = std::result::Result<T, QueryError>;

/// Query builder for audit logs
#[derive(Debug, Clone)]
pub struct AuditQuery {
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    user_ids: Vec<String>,
    actions: Vec<EventType>,
    resources: Vec<String>,
    resource_types: Vec<String>,
    severities: Vec<EventSeverity>,
    success_only: Option<bool>,
    session_ids: Vec<String>,
    ip_addresses: Vec<String>,
    search_text: Option<String>,
    limit: Option<usize>,
    offset: usize,
}

impl AuditQuery {
    /// Create a new query builder
    pub fn new() -> Self {
        Self {
            start_time: None,
            end_time: None,
            user_ids: Vec::new(),
            actions: Vec::new(),
            resources: Vec::new(),
            resource_types: Vec::new(),
            severities: Vec::new(),
            success_only: None,
            session_ids: Vec::new(),
            ip_addresses: Vec::new(),
            search_text: None,
            limit: None,
            offset: 0,
        }
    }

    /// Filter by start time
    pub fn start_time(mut self, time: DateTime<Utc>) -> Self {
        self.start_time = Some(time);
        self
    }

    /// Filter by end time
    pub fn end_time(mut self, time: DateTime<Utc>) -> Self {
        self.end_time = Some(time);
        self
    }

    /// Filter by time range
    pub fn time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    /// Filter events from the last N hours
    pub fn last_hours(mut self, hours: i64) -> Self {
        let now = Utc::now();
        self.start_time = Some(now - Duration::hours(hours));
        self.end_time = Some(now);
        self
    }

    /// Filter events from the last N days
    pub fn last_days(mut self, days: i64) -> Self {
        let now = Utc::now();
        self.start_time = Some(now - Duration::days(days));
        self.end_time = Some(now);
        self
    }

    /// Filter by user ID
    pub fn user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_ids.push(user_id.into());
        self
    }

    /// Filter by multiple user IDs
    pub fn user_ids(mut self, user_ids: Vec<String>) -> Self {
        self.user_ids.extend(user_ids);
        self
    }

    /// Filter by action type
    pub fn action(mut self, action: EventType) -> Self {
        self.actions.push(action);
        self
    }

    /// Filter by multiple action types
    pub fn actions(mut self, actions: Vec<EventType>) -> Self {
        self.actions.extend(actions);
        self
    }

    /// Filter by resource
    pub fn resource(mut self, resource: impl Into<String>) -> Self {
        self.resources.push(resource.into());
        self
    }

    /// Filter by multiple resources
    pub fn resources(mut self, resources: Vec<String>) -> Self {
        self.resources.extend(resources);
        self
    }

    /// Filter by resource type
    pub fn resource_type(mut self, resource_type: impl Into<String>) -> Self {
        self.resource_types.push(resource_type.into());
        self
    }

    /// Filter by severity
    pub fn severity(mut self, severity: EventSeverity) -> Self {
        self.severities.push(severity);
        self
    }

    /// Filter by multiple severities
    pub fn severities(mut self, severities: Vec<EventSeverity>) -> Self {
        self.severities.extend(severities);
        self
    }

    /// Filter only successful events
    pub fn success_only(mut self) -> Self {
        self.success_only = Some(true);
        self
    }

    /// Filter only failed events
    pub fn failures_only(mut self) -> Self {
        self.success_only = Some(false);
        self
    }

    /// Filter by session ID
    pub fn session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_ids.push(session_id.into());
        self
    }

    /// Filter by IP address
    pub fn ip_address(mut self, ip: impl Into<String>) -> Self {
        self.ip_addresses.push(ip.into());
        self
    }

    /// Full-text search in event details
    pub fn search(mut self, text: impl Into<String>) -> Self {
        self.search_text = Some(text.into());
        self
    }

    /// Limit number of results
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set offset for pagination
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    /// Execute the query against a storage backend
    pub async fn execute<S>(&self, storage: &S) -> Result<Vec<AuditEvent>>
    where
        S: AuditStorage,
    {
        // Get initial set of events from storage
        let mut events = if let (Some(start), Some(end)) = (self.start_time, self.end_time) {
            storage.retrieve(start, end).await?
        } else {
            // If no time range specified, use a reasonable default (last 30 days)
            let end = Utc::now();
            let start = end - Duration::days(30);
            storage.retrieve(start, end).await?
        };

        // Apply filters
        events.retain(|e| self.matches_filters(e));

        // Sort by timestamp (newest first)
        events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply pagination
        let start_idx = self.offset;
        let end_idx = if let Some(limit) = self.limit {
            (start_idx + limit).min(events.len())
        } else {
            events.len()
        };

        Ok(events[start_idx..end_idx].to_vec())
    }

    /// Check if an event matches all filters
    fn matches_filters(&self, event: &AuditEvent) -> bool {
        // User ID filter
        if !self.user_ids.is_empty() && !self.user_ids.contains(&event.user_id) {
            return false;
        }

        // Action filter
        if !self.actions.is_empty() && !self.actions.contains(&event.action) {
            return false;
        }

        // Resource filter
        if !self.resources.is_empty() && !self.resources.contains(&event.resource) {
            return false;
        }

        // Resource type filter
        if !self.resource_types.is_empty() {
            if let Some(ref resource_type) = event.resource_type {
                if !self.resource_types.contains(resource_type) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Severity filter
        if !self.severities.is_empty() && !self.severities.contains(&event.severity) {
            return false;
        }

        // Success filter
        if let Some(success) = self.success_only {
            if event.success != success {
                return false;
            }
        }

        // Session ID filter
        if !self.session_ids.is_empty() {
            if let Some(ref session_id) = event.session_id {
                if !self.session_ids.contains(session_id) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // IP address filter
        if !self.ip_addresses.is_empty() {
            if let Some(ref ip) = event.ip_address {
                if !self.ip_addresses.contains(ip) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Text search
        if let Some(ref search_text) = self.search_text {
            if !self.matches_search(event, search_text) {
                return false;
            }
        }

        true
    }

    /// Check if event matches search text
    fn matches_search(&self, event: &AuditEvent, search: &str) -> bool {
        let search_lower = search.to_lowercase();

        // Search in user ID
        if event.user_id.to_lowercase().contains(&search_lower) {
            return true;
        }

        // Search in resource
        if event.resource.to_lowercase().contains(&search_lower) {
            return true;
        }

        // Search in details
        for (key, value) in &event.details {
            if key.to_lowercase().contains(&search_lower)
                || value.to_lowercase().contains(&search_lower)
            {
                return true;
            }
        }

        // Search in error message
        if let Some(ref error) = event.error_message {
            if error.to_lowercase().contains(&search_lower) {
                return true;
            }
        }

        false
    }
}

impl Default for AuditQuery {
    fn default() -> Self {
        Self::new()
    }
}

/// Aggregate query results
pub struct QueryAggregation {
    events: Vec<AuditEvent>,
}

impl QueryAggregation {
    /// Create aggregation from events
    pub fn new(events: Vec<AuditEvent>) -> Self {
        Self { events }
    }

    /// Count events by user
    pub fn by_user(&self) -> Vec<(String, usize)> {
        let mut counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for event in &self.events {
            *counts.entry(event.user_id.clone()).or_insert(0) += 1;
        }

        let mut result: Vec<_> = counts.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result
    }

    /// Count events by action type
    pub fn by_action(&self) -> Vec<(EventType, usize)> {
        let mut counts: std::collections::HashMap<EventType, usize> =
            std::collections::HashMap::new();

        for event in &self.events {
            *counts.entry(event.action).or_insert(0) += 1;
        }

        let mut result: Vec<_> = counts.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result
    }

    /// Count events by resource
    pub fn by_resource(&self) -> Vec<(String, usize)> {
        let mut counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for event in &self.events {
            *counts.entry(event.resource.clone()).or_insert(0) += 1;
        }

        let mut result: Vec<_> = counts.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result
    }

    /// Count events by severity
    pub fn by_severity(&self) -> Vec<(EventSeverity, usize)> {
        let mut counts: std::collections::HashMap<EventSeverity, usize> =
            std::collections::HashMap::new();

        for event in &self.events {
            *counts.entry(event.severity).or_insert(0) += 1;
        }

        let mut result: Vec<_> = counts.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result
    }

    /// Get unique users
    pub fn unique_users(&self) -> HashSet<String> {
        self.events.iter().map(|e| e.user_id.clone()).collect()
    }

    /// Get unique resources
    pub fn unique_resources(&self) -> HashSet<String> {
        self.events.iter().map(|e| e.resource.clone()).collect()
    }

    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.events.is_empty() {
            return 0.0;
        }

        let successful = self.events.iter().filter(|e| e.success).count();
        successful as f64 / self.events.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enterprise::audit::storage::MemoryStorage;
    use crate::enterprise::audit::event::AuditEvent;

    #[tokio::test]
    async fn test_query_by_user() {
        let mut storage = MemoryStorage::new();

        let event1 = AuditEvent::builder()
            .user_id("user1")
            .action(EventType::Create)
            .resource("resource1")
            .build();

        let event2 = AuditEvent::builder()
            .user_id("user2")
            .action(EventType::Update)
            .resource("resource2")
            .build();

        storage.store(&event1).await.unwrap();
        storage.store(&event2).await.unwrap();

        let results = AuditQuery::new()
            .user_id("user1")
            .last_days(1)
            .execute(&storage)
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].user_id, "user1");
    }

    #[tokio::test]
    async fn test_query_multiple_filters() {
        let mut storage = MemoryStorage::new();

        let event = AuditEvent::builder()
            .user_id("user1")
            .action(EventType::Create)
            .resource("drawing/123")
            .resource_type("drawing")
            .severity(EventSeverity::Info)
            .build();

        storage.store(&event).await.unwrap();

        let results = AuditQuery::new()
            .user_id("user1")
            .action(EventType::Create)
            .resource_type("drawing")
            .last_days(1)
            .execute(&storage)
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_aggregation() {
        let events = vec![
            AuditEvent::builder()
                .user_id("user1")
                .action(EventType::Create)
                .resource("res1")
                .build(),
            AuditEvent::builder()
                .user_id("user1")
                .action(EventType::Update)
                .resource("res1")
                .build(),
            AuditEvent::builder()
                .user_id("user2")
                .action(EventType::Create)
                .resource("res2")
                .build(),
        ];

        let agg = QueryAggregation::new(events);

        let by_user = agg.by_user();
        assert_eq!(by_user.len(), 2);
        assert_eq!(by_user[0].0, "user1");
        assert_eq!(by_user[0].1, 2);

        let unique_users = agg.unique_users();
        assert_eq!(unique_users.len(), 2);
    }
}
