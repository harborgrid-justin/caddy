//! # Performance Profiling
//!
//! High-precision performance profiling and tracing for CADDY operations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use chrono::{DateTime, Duration, Utc};
use std::time::Instant;

/// Performance profile span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSpan {
    /// Span ID
    pub id: String,

    /// Parent span ID (for nested spans)
    pub parent_id: Option<String>,

    /// Span name
    pub name: String,

    /// Start time
    pub start_time: DateTime<Utc>,

    /// End time (None if still active)
    pub end_time: Option<DateTime<Utc>>,

    /// Duration in microseconds
    pub duration_us: Option<i64>,

    /// Tags/labels
    pub tags: HashMap<String, String>,

    /// Custom metrics
    pub metrics: HashMap<String, f64>,

    /// Error information
    pub error: Option<String>,
}

impl ProfileSpan {
    /// Create a new span
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            parent_id: None,
            name: name.into(),
            start_time: Utc::now(),
            end_time: None,
            duration_us: None,
            tags: HashMap::new(),
            metrics: HashMap::new(),
            error: None,
        }
    }

    /// Set parent span
    pub fn with_parent(mut self, parent_id: String) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }

    /// Add a metric
    pub fn with_metric(mut self, key: impl Into<String>, value: f64) -> Self {
        self.metrics.insert(key.into(), value);
        self
    }

    /// Mark span as complete
    pub fn finish(&mut self) {
        let end_time = Utc::now();
        self.end_time = Some(end_time);
        self.duration_us = Some((end_time - self.start_time).num_microseconds().unwrap_or(0));
    }

    /// Mark span as failed
    pub fn fail(&mut self, error: impl Into<String>) {
        self.error = Some(error.into());
        self.finish();
    }

    /// Check if span is active
    pub fn is_active(&self) -> bool {
        self.end_time.is_none()
    }

    /// Get duration in milliseconds
    pub fn duration_ms(&self) -> Option<f64> {
        self.duration_us.map(|us| us as f64 / 1000.0)
    }
}

/// Active span guard (RAII)
pub struct SpanGuard {
    span: ProfileSpan,
    profiler: Arc<PerformanceProfiler>,
    start: Instant,
}

impl SpanGuard {
    /// Add a tag to the span
    pub fn tag(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.span.tags.insert(key.into(), value.into());
    }

    /// Add a metric to the span
    pub fn metric(&mut self, key: impl Into<String>, value: f64) {
        self.span.metrics.insert(key.into(), value);
    }

    /// Record an error
    pub fn error(&mut self, error: impl Into<String>) {
        self.span.error = Some(error.into());
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_micros() as f64 / 1000.0
    }
}

impl Drop for SpanGuard {
    fn drop(&mut self) {
        self.span.finish();
        if self.profiler.enabled.load(std::sync::atomic::Ordering::Relaxed) {
            self.profiler.record_span(self.span.clone());
        }
    }
}

/// Performance profiler
pub struct PerformanceProfiler {
    /// Enable/disable profiling
    enabled: Arc<std::sync::atomic::AtomicBool>,

    /// Active spans (by ID)
    active_spans: Arc<RwLock<HashMap<String, ProfileSpan>>>,

    /// Completed spans history
    completed_spans: Arc<RwLock<Vec<ProfileSpan>>>,

    /// Performance statistics by operation name
    stats: Arc<RwLock<HashMap<String, OperationStats>>>,

    /// Maximum history size
    max_history: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OperationStats {
    count: u64,
    total_duration_us: i64,
    min_duration_us: i64,
    max_duration_us: i64,
    avg_duration_us: f64,
    error_count: u64,
}

impl OperationStats {
    fn new() -> Self {
        Self {
            count: 0,
            total_duration_us: 0,
            min_duration_us: i64::MAX,
            max_duration_us: 0,
            avg_duration_us: 0.0,
            error_count: 0,
        }
    }

    fn update(&mut self, duration_us: i64, has_error: bool) {
        self.count += 1;
        self.total_duration_us += duration_us;
        self.min_duration_us = self.min_duration_us.min(duration_us);
        self.max_duration_us = self.max_duration_us.max(duration_us);
        self.avg_duration_us = self.total_duration_us as f64 / self.count as f64;

        if has_error {
            self.error_count += 1;
        }
    }
}

impl PerformanceProfiler {
    /// Create a new performance profiler
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled: Arc::new(std::sync::atomic::AtomicBool::new(enabled)),
            active_spans: Arc::new(RwLock::new(HashMap::new())),
            completed_spans: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(HashMap::new())),
            max_history: 10000,
        }
    }

    /// Enable profiling
    pub fn enable(&self) {
        self.enabled.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// Disable profiling
    pub fn disable(&self) {
        self.enabled.store(false, std::sync::atomic::Ordering::Relaxed);
    }

    /// Check if profiling is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Start a new span
    pub fn start_span(&self, name: impl Into<String>) -> SpanGuard {
        let span = ProfileSpan::new(name);
        let id = span.id.clone();

        if self.is_enabled() {
            self.active_spans.write().insert(id, span.clone());
        }

        SpanGuard {
            span,
            profiler: Arc::new(self.clone_minimal()),
            start: Instant::now(),
        }
    }

    /// Start a child span
    pub fn start_child_span(&self, parent_id: String, name: impl Into<String>) -> SpanGuard {
        let mut span = ProfileSpan::new(name);
        span.parent_id = Some(parent_id);
        let id = span.id.clone();

        if self.is_enabled() {
            self.active_spans.write().insert(id, span.clone());
        }

        SpanGuard {
            span,
            profiler: Arc::new(self.clone_minimal()),
            start: Instant::now(),
        }
    }

    /// Record a completed span
    fn record_span(&self, span: ProfileSpan) {
        // Remove from active spans
        self.active_spans.write().remove(&span.id);

        // Update statistics
        if let Some(duration_us) = span.duration_us {
            let mut stats = self.stats.write();
            let op_stats = stats.entry(span.name.clone()).or_insert_with(OperationStats::new);
            op_stats.update(duration_us, span.error.is_some());
        }

        // Add to history
        let mut history = self.completed_spans.write();
        history.push(span);

        // Trim history if needed
        if history.len() > self.max_history {
            history.drain(0..history.len() - self.max_history);
        }
    }

    /// Get active span count
    pub fn active_count(&self) -> usize {
        self.active_spans.read().len()
    }

    /// Get operation statistics
    pub fn get_stats(&self, operation_name: &str) -> Option<ProfileReport> {
        self.stats.read().get(operation_name).map(|stats| ProfileReport {
            operation_name: operation_name.to_string(),
            total_calls: stats.count,
            total_duration_ms: stats.total_duration_us as f64 / 1000.0,
            avg_duration_ms: stats.avg_duration_us / 1000.0,
            min_duration_ms: stats.min_duration_us as f64 / 1000.0,
            max_duration_ms: stats.max_duration_us as f64 / 1000.0,
            error_rate: if stats.count > 0 {
                stats.error_count as f64 / stats.count as f64
            } else {
                0.0
            },
            recent_spans: Vec::new(),
        })
    }

    /// Get all operation statistics
    pub fn all_stats(&self) -> Vec<ProfileReport> {
        self.stats
            .read()
            .keys()
            .filter_map(|name| self.get_stats(name))
            .collect()
    }

    /// Get recent spans for an operation
    pub fn recent_spans(&self, operation_name: &str, limit: usize) -> Vec<ProfileSpan> {
        self.completed_spans
            .read()
            .iter()
            .rev()
            .filter(|span| span.name == operation_name)
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get all recent spans
    pub fn all_recent_spans(&self, limit: usize) -> Vec<ProfileSpan> {
        self.completed_spans
            .read()
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get slowest operations
    pub fn slowest_operations(&self, limit: usize) -> Vec<ProfileReport> {
        let mut reports = self.all_stats();
        reports.sort_by(|a, b| {
            b.avg_duration_ms
                .partial_cmp(&a.avg_duration_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        reports.into_iter().take(limit).collect()
    }

    /// Clear all profiling data
    pub fn clear(&self) {
        self.active_spans.write().clear();
        self.completed_spans.write().clear();
        self.stats.write().clear();
    }

    /// Generate a flame graph representation
    pub fn flame_graph(&self) -> FlameGraph {
        let spans = self.completed_spans.read();

        // Build hierarchical structure
        let mut root_spans = Vec::new();
        let mut span_children: HashMap<String, Vec<ProfileSpan>> = HashMap::new();

        for span in spans.iter() {
            if let Some(parent_id) = &span.parent_id {
                span_children
                    .entry(parent_id.clone())
                    .or_insert_with(Vec::new)
                    .push(span.clone());
            } else {
                root_spans.push(span.clone());
            }
        }

        FlameGraph {
            roots: root_spans
                .into_iter()
                .map(|span| self.build_flame_node(span, &span_children))
                .collect(),
        }
    }

    fn build_flame_node(
        &self,
        span: ProfileSpan,
        children_map: &HashMap<String, Vec<ProfileSpan>>,
    ) -> FlameNode {
        let children = children_map
            .get(&span.id)
            .map(|children| {
                children
                    .iter()
                    .map(|child| self.build_flame_node(child.clone(), children_map))
                    .collect()
            })
            .unwrap_or_default();

        FlameNode {
            name: span.name,
            duration_ms: span.duration_ms().unwrap_or(0.0),
            children,
        }
    }

    // Helper to create a minimal clone for SpanGuard
    fn clone_minimal(&self) -> Self {
        Self {
            enabled: Arc::clone(&self.enabled),
            active_spans: Arc::clone(&self.active_spans),
            completed_spans: Arc::clone(&self.completed_spans),
            stats: Arc::clone(&self.stats),
            max_history: self.max_history,
        }
    }
}

/// Profile report for an operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileReport {
    pub operation_name: String,
    pub total_calls: u64,
    pub total_duration_ms: f64,
    pub avg_duration_ms: f64,
    pub min_duration_ms: f64,
    pub max_duration_ms: f64,
    pub error_rate: f64,
    pub recent_spans: Vec<ProfileSpan>,
}

/// Flame graph representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlameGraph {
    pub roots: Vec<FlameNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlameNode {
    pub name: String,
    pub duration_ms: f64,
    pub children: Vec<FlameNode>,
}

/// Convenience macro for profiling a block of code
#[macro_export]
macro_rules! profile {
    ($profiler:expr, $name:expr, $block:expr) => {{
        let _span = $profiler.start_span($name);
        $block
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration as StdDuration;

    #[test]
    fn test_span_creation() {
        let span = ProfileSpan::new("test_operation");
        assert_eq!(span.name, "test_operation");
        assert!(span.is_active());
    }

    #[test]
    fn test_span_finish() {
        let mut span = ProfileSpan::new("test_operation");
        thread::sleep(StdDuration::from_millis(10));
        span.finish();

        assert!(!span.is_active());
        assert!(span.duration_us.is_some());
        assert!(span.duration_ms().unwrap() >= 10.0);
    }

    #[test]
    fn test_profiler_creation() {
        let profiler = PerformanceProfiler::new(true);
        assert!(profiler.is_enabled());
    }

    #[test]
    fn test_span_guard() {
        let profiler = PerformanceProfiler::new(true);
        {
            let _guard = profiler.start_span("test_op");
            thread::sleep(StdDuration::from_millis(10));
        }

        // Span should be recorded after guard is dropped
        thread::sleep(StdDuration::from_millis(10));
        let stats = profiler.get_stats("test_op");
        assert!(stats.is_some());
    }

    #[test]
    fn test_operation_stats() {
        let profiler = PerformanceProfiler::new(true);

        for _ in 0..5 {
            let _guard = profiler.start_span("repeated_op");
            thread::sleep(StdDuration::from_millis(5));
        }

        thread::sleep(StdDuration::from_millis(10));
        let stats = profiler.get_stats("repeated_op");
        assert!(stats.is_some());

        let stats = stats.unwrap();
        assert_eq!(stats.total_calls, 5);
    }
}
