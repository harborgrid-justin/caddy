//! # Metrics Collection Engine
//!
//! High-performance metrics collection with minimal overhead.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use parking_lot::RwLock;
use chrono::{DateTime, Utc};

/// Metric type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType {
    /// Counter - monotonically increasing value
    Counter,
    /// Gauge - arbitrary value that can go up or down
    Gauge,
    /// Histogram - distribution of values
    Histogram,
    /// Summary - statistical summary of values
    Summary,
}

/// Metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    /// Integer value
    Int(i64),
    /// Float value
    Float(f64),
    /// Boolean value
    Bool(bool),
    /// Histogram bucket
    Histogram {
        buckets: Vec<f64>,
        counts: Vec<u64>,
    },
    /// Summary statistics
    Summary {
        count: u64,
        sum: f64,
        min: f64,
        max: f64,
        percentiles: HashMap<String, f64>,
    },
}

impl MetricValue {
    /// Convert to f64 if possible
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            MetricValue::Int(v) => Some(*v as f64),
            MetricValue::Float(v) => Some(*v),
            MetricValue::Bool(v) => Some(if *v { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    /// Add two metric values (for aggregation)
    pub fn add(&self, other: &MetricValue) -> Option<MetricValue> {
        match (self, other) {
            (MetricValue::Int(a), MetricValue::Int(b)) => Some(MetricValue::Int(a + b)),
            (MetricValue::Float(a), MetricValue::Float(b)) => Some(MetricValue::Float(a + b)),
            _ => None,
        }
    }
}

/// A single metric measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Metric name
    pub name: String,

    /// Metric type
    pub metric_type: MetricType,

    /// Metric value
    pub value: MetricValue,

    /// Labels/tags
    pub labels: HashMap<String, String>,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Optional description
    pub description: Option<String>,
}

impl Metric {
    /// Create a new counter metric
    pub fn counter(name: impl Into<String>, value: i64) -> Self {
        Self {
            name: name.into(),
            metric_type: MetricType::Counter,
            value: MetricValue::Int(value),
            labels: HashMap::new(),
            timestamp: Utc::now(),
            description: None,
        }
    }

    /// Create a new gauge metric
    pub fn gauge(name: impl Into<String>, value: f64) -> Self {
        Self {
            name: name.into(),
            metric_type: MetricType::Gauge,
            value: MetricValue::Float(value),
            labels: HashMap::new(),
            timestamp: Utc::now(),
            description: None,
        }
    }

    /// Add a label to the metric
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Add a description to the metric
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set custom timestamp
    pub fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }
}

/// Metrics collector
pub struct MetricsCollector {
    /// Current metrics buffer
    metrics: Arc<RwLock<Vec<Metric>>>,

    /// Total metrics collected
    total_collected: AtomicU64,

    /// Start time
    start_time: DateTime<Utc>,

    /// Named counters for fast access
    counters: Arc<RwLock<HashMap<String, AtomicU64>>>,

    /// Named gauges for fast access
    gauges: Arc<RwLock<HashMap<String, Arc<RwLock<f64>>>>>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Vec::new())),
            total_collected: AtomicU64::new(0),
            start_time: Utc::now(),
            counters: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a metric
    pub fn record(&self, metric: Metric) {
        self.metrics.write().push(metric);
        self.total_collected.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment a counter by name
    pub fn increment_counter(&self, name: &str, value: u64) {
        let counters = self.counters.read();
        if let Some(counter) = counters.get(name) {
            counter.fetch_add(value, Ordering::Relaxed);
        } else {
            drop(counters);
            let mut counters = self.counters.write();
            counters.insert(name.to_string(), AtomicU64::new(value));
        }
    }

    /// Set a gauge value by name
    pub fn set_gauge(&self, name: &str, value: f64) {
        let gauges = self.gauges.read();
        if let Some(gauge) = gauges.get(name) {
            *gauge.write() = value;
        } else {
            drop(gauges);
            let mut gauges = self.gauges.write();
            gauges.insert(name.to_string(), Arc::new(RwLock::new(value)));
        }
    }

    /// Get counter value
    pub fn get_counter(&self, name: &str) -> Option<u64> {
        self.counters.read().get(name).map(|c| c.load(Ordering::Relaxed))
    }

    /// Get gauge value
    pub fn get_gauge(&self, name: &str) -> Option<f64> {
        self.gauges.read().get(name).map(|g| *g.read())
    }

    /// Collect and drain all metrics
    pub fn collect(&self) -> Vec<Metric> {
        let mut metrics = self.metrics.write();

        // Add current counter and gauge snapshots
        let mut snapshot = Vec::new();

        for (name, counter) in self.counters.read().iter() {
            snapshot.push(Metric::counter(
                name.clone(),
                counter.load(Ordering::Relaxed) as i64,
            ));
        }

        for (name, gauge) in self.gauges.read().iter() {
            snapshot.push(Metric::gauge(name.clone(), *gauge.read()));
        }

        // Add custom metrics
        snapshot.extend(metrics.drain(..));

        snapshot
    }

    /// Get total metrics collected
    pub fn total_metrics(&self) -> u64 {
        self.total_collected.load(Ordering::Relaxed)
    }

    /// Get uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        (Utc::now() - self.start_time).num_seconds() as u64
    }

    /// Clear all metrics
    pub fn clear(&self) {
        self.metrics.write().clear();
        self.counters.write().clear();
        self.gauges.write().clear();
    }

    /// Get current buffer size
    pub fn buffer_size(&self) -> usize {
        self.metrics.read().len()
    }

    /// Record system metrics
    pub fn record_system_metrics(&self) {
        // CPU usage
        self.set_gauge("system.cpu.usage_percent", Self::get_cpu_usage());

        // Memory usage
        self.set_gauge("system.memory.used_bytes", Self::get_memory_used() as f64);
        self.set_gauge("system.memory.total_bytes", Self::get_memory_total() as f64);

        // Disk usage
        self.set_gauge("system.disk.used_bytes", Self::get_disk_used() as f64);
        self.set_gauge("system.disk.total_bytes", Self::get_disk_total() as f64);
    }

    /// Record CAD-specific metrics
    pub fn record_cad_metrics(
        &self,
        entities_count: usize,
        layers_count: usize,
        viewport_fps: f64,
    ) {
        self.set_gauge("cad.entities.total", entities_count as f64);
        self.set_gauge("cad.layers.total", layers_count as f64);
        self.set_gauge("cad.viewport.fps", viewport_fps);
    }

    /// Record command execution
    pub fn record_command_execution(&self, command_name: &str, duration_ms: f64, success: bool) {
        self.increment_counter(
            &format!("commands.{}.total", command_name),
            1,
        );

        if success {
            self.increment_counter(
                &format!("commands.{}.success", command_name),
                1,
            );
        } else {
            self.increment_counter(
                &format!("commands.{}.failure", command_name),
                1,
            );
        }

        self.set_gauge(
            &format!("commands.{}.duration_ms", command_name),
            duration_ms,
        );
    }

    // Helper methods for system metrics (would use proper system APIs in production)

    fn get_cpu_usage() -> f64 {
        // Placeholder - would use sysinfo or similar crate
        0.0
    }

    fn get_memory_used() -> u64 {
        // Placeholder - would use sysinfo or similar crate
        0
    }

    fn get_memory_total() -> u64 {
        // Placeholder - would use sysinfo or similar crate
        0
    }

    fn get_disk_used() -> u64 {
        // Placeholder - would use sysinfo or similar crate
        0
    }

    fn get_disk_total() -> u64 {
        // Placeholder - would use sysinfo or similar crate
        0
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_creation() {
        let metric = Metric::counter("test_counter", 42)
            .with_label("env", "test")
            .with_description("Test counter");

        assert_eq!(metric.name, "test_counter");
        assert_eq!(metric.metric_type, MetricType::Counter);
        assert!(metric.labels.contains_key("env"));
    }

    #[test]
    fn test_collector_counters() {
        let collector = MetricsCollector::new();

        collector.increment_counter("requests", 1);
        collector.increment_counter("requests", 5);

        assert_eq!(collector.get_counter("requests"), Some(6));
    }

    #[test]
    fn test_collector_gauges() {
        let collector = MetricsCollector::new();

        collector.set_gauge("temperature", 25.5);
        assert_eq!(collector.get_gauge("temperature"), Some(25.5));

        collector.set_gauge("temperature", 30.0);
        assert_eq!(collector.get_gauge("temperature"), Some(30.0));
    }

    #[test]
    fn test_collect_metrics() {
        let collector = MetricsCollector::new();

        collector.increment_counter("test_counter", 10);
        collector.set_gauge("test_gauge", 42.0);

        let metrics = collector.collect();
        assert!(metrics.len() >= 2);
    }
}
