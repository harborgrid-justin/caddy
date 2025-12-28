//! Metrics collection and exposition
//!
//! This module provides Counter, Gauge, and Histogram metric types with support
//! for labels/dimensions, Prometheus exposition format, and StatsD protocol.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Metric registry for collecting and managing metrics
#[derive(Clone)]
pub struct MetricRegistry {
    metrics: Arc<RwLock<HashMap<String, Metric>>>,
}

impl MetricRegistry {
    /// Create a new metric registry
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a counter metric
    pub fn counter(&self, name: impl Into<String>, help: impl Into<String>) -> Counter {
        let name = name.into();
        let metric = Metric::Counter(CounterMetric {
            name: name.clone(),
            help: help.into(),
            value: Arc::new(RwLock::new(0.0)),
            labels: HashMap::new(),
        });

        self.metrics.write().insert(name.clone(), metric.clone());
        Counter { metric }
    }

    /// Register a gauge metric
    pub fn gauge(&self, name: impl Into<String>, help: impl Into<String>) -> Gauge {
        let name = name.into();
        let metric = Metric::Gauge(GaugeMetric {
            name: name.clone(),
            help: help.into(),
            value: Arc::new(RwLock::new(0.0)),
            labels: HashMap::new(),
        });

        self.metrics.write().insert(name.clone(), metric.clone());
        Gauge { metric }
    }

    /// Register a histogram metric
    pub fn histogram(
        &self,
        name: impl Into<String>,
        help: impl Into<String>,
        buckets: Vec<f64>,
    ) -> Histogram {
        let name = name.into();
        let metric = Metric::Histogram(HistogramMetric {
            name: name.clone(),
            help: help.into(),
            buckets: buckets.clone(),
            observations: Arc::new(RwLock::new(Vec::new())),
            labels: HashMap::new(),
        });

        self.metrics.write().insert(name.clone(), metric.clone());
        Histogram { metric }
    }

    /// Get all metrics
    pub fn metrics(&self) -> Vec<Metric> {
        self.metrics.read().values().cloned().collect()
    }

    /// Export metrics in Prometheus format
    pub fn prometheus_export(&self) -> String {
        let mut output = String::new();

        for metric in self.metrics().iter() {
            match metric {
                Metric::Counter(c) => {
                    output.push_str(&format!("# HELP {} {}\n", c.name, c.help));
                    output.push_str(&format!("# TYPE {} counter\n", c.name));
                    let value = *c.value.read();
                    output.push_str(&format!("{} {}\n", c.name, value));
                }
                Metric::Gauge(g) => {
                    output.push_str(&format!("# HELP {} {}\n", g.name, g.help));
                    output.push_str(&format!("# TYPE {} gauge\n", g.name));
                    let value = *g.value.read();
                    output.push_str(&format!("{} {}\n", g.name, value));
                }
                Metric::Histogram(h) => {
                    output.push_str(&format!("# HELP {} {}\n", h.name, h.help));
                    output.push_str(&format!("# TYPE {} histogram\n", h.name));

                    let observations = h.observations.read();
                    let mut bucket_counts: HashMap<String, u64> = HashMap::new();
                    let mut sum = 0.0;
                    let count = observations.len();

                    for &obs in observations.iter() {
                        sum += obs;
                        for &bucket in &h.buckets {
                            if obs <= bucket {
                                let key = format!("{:.2}", bucket);
                                *bucket_counts.entry(key).or_insert(0) += 1;
                            }
                        }
                    }

                    for bucket in &h.buckets {
                        let key = format!("{:.2}", bucket);
                        let count = bucket_counts.get(&key).unwrap_or(&0);
                        output.push_str(&format!(
                            "{}{{le=\"{}\"}} {}\n",
                            h.name, bucket, count
                        ));
                    }

                    output.push_str(&format!("{}{{le=\"+Inf\"}} {}\n", h.name, count));
                    output.push_str(&format!("{}_sum {}\n", h.name, sum));
                    output.push_str(&format!("{}_count {}\n", h.name, count));
                }
            }
            output.push('\n');
        }

        output
    }

    /// Export metrics in StatsD format
    pub fn statsd_export(&self) -> Vec<String> {
        let mut output = Vec::new();

        for metric in self.metrics().iter() {
            match metric {
                Metric::Counter(c) => {
                    let value = *c.value.read();
                    output.push(format!("{}:{}|c", c.name, value));
                }
                Metric::Gauge(g) => {
                    let value = *g.value.read();
                    output.push(format!("{}:{}|g", g.name, value));
                }
                Metric::Histogram(h) => {
                    let observations = h.observations.read();
                    for &obs in observations.iter() {
                        output.push(format!("{}:{}|h", h.name, obs));
                    }
                }
            }
        }

        output
    }

    /// Clear all metrics
    pub fn clear(&self) {
        self.metrics.write().clear();
    }

    /// Get metric by name
    pub fn get(&self, name: &str) -> Option<Metric> {
        self.metrics.read().get(name).cloned()
    }
}

impl Default for MetricRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Metric types
#[derive(Debug, Clone)]
pub enum Metric {
    /// Counter metric (monotonically increasing)
    Counter(CounterMetric),
    /// Gauge metric (can go up or down)
    Gauge(GaugeMetric),
    /// Histogram metric (distribution of values)
    Histogram(HistogramMetric),
}

/// Counter metric (monotonically increasing value)
#[derive(Debug, Clone)]
pub struct CounterMetric {
    name: String,
    help: String,
    value: Arc<RwLock<f64>>,
    labels: HashMap<String, String>,
}

/// Counter handle
#[derive(Clone)]
pub struct Counter {
    metric: Metric,
}

impl Counter {
    /// Increment counter by 1
    pub fn inc(&self) {
        self.add(1.0);
    }

    /// Add a value to the counter
    pub fn add(&self, value: f64) {
        if let Metric::Counter(ref counter) = self.metric {
            let mut val = counter.value.write();
            *val += value;
        }
    }

    /// Get current value
    pub fn get(&self) -> f64 {
        if let Metric::Counter(ref counter) = self.metric {
            *counter.value.read()
        } else {
            0.0
        }
    }

    /// Create a labeled version of this counter
    pub fn with_labels(self, labels: HashMap<String, String>) -> Self {
        if let Metric::Counter(mut counter) = self.metric {
            counter.labels = labels;
            Self {
                metric: Metric::Counter(counter),
            }
        } else {
            self
        }
    }
}

/// Gauge metric (value that can go up or down)
#[derive(Debug, Clone)]
pub struct GaugeMetric {
    name: String,
    help: String,
    value: Arc<RwLock<f64>>,
    labels: HashMap<String, String>,
}

/// Gauge handle
#[derive(Clone)]
pub struct Gauge {
    metric: Metric,
}

impl Gauge {
    /// Set gauge to a specific value
    pub fn set(&self, value: f64) {
        if let Metric::Gauge(ref gauge) = self.metric {
            let mut val = gauge.value.write();
            *val = value;
        }
    }

    /// Increment gauge by 1
    pub fn inc(&self) {
        self.add(1.0);
    }

    /// Decrement gauge by 1
    pub fn dec(&self) {
        self.sub(1.0);
    }

    /// Add to gauge
    pub fn add(&self, value: f64) {
        if let Metric::Gauge(ref gauge) = self.metric {
            let mut val = gauge.value.write();
            *val += value;
        }
    }

    /// Subtract from gauge
    pub fn sub(&self, value: f64) {
        if let Metric::Gauge(ref gauge) = self.metric {
            let mut val = gauge.value.write();
            *val -= value;
        }
    }

    /// Get current value
    pub fn get(&self) -> f64 {
        if let Metric::Gauge(ref gauge) = self.metric {
            *gauge.value.read()
        } else {
            0.0
        }
    }

    /// Create a labeled version of this gauge
    pub fn with_labels(self, labels: HashMap<String, String>) -> Self {
        if let Metric::Gauge(mut gauge) = self.metric {
            gauge.labels = labels;
            Self {
                metric: Metric::Gauge(gauge),
            }
        } else {
            self
        }
    }

    /// Set to current Unix timestamp
    pub fn set_to_current_time(&self) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        self.set(timestamp);
    }
}

/// Histogram metric (distribution of values)
#[derive(Debug, Clone)]
pub struct HistogramMetric {
    name: String,
    help: String,
    buckets: Vec<f64>,
    observations: Arc<RwLock<Vec<f64>>>,
    labels: HashMap<String, String>,
}

/// Histogram handle
#[derive(Clone)]
pub struct Histogram {
    metric: Metric,
}

impl Histogram {
    /// Observe a value
    pub fn observe(&self, value: f64) {
        if let Metric::Histogram(ref histogram) = self.metric {
            histogram.observations.write().push(value);
        }
    }

    /// Time a duration and observe it
    pub fn time<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        self.observe(duration.as_secs_f64());
        result
    }

    /// Create a timer that observes when dropped
    pub fn start_timer(&self) -> HistogramTimer {
        HistogramTimer {
            histogram: self.clone(),
            start: Instant::now(),
        }
    }

    /// Get observations
    pub fn observations(&self) -> Vec<f64> {
        if let Metric::Histogram(ref histogram) = self.metric {
            histogram.observations.read().clone()
        } else {
            Vec::new()
        }
    }

    /// Calculate statistics
    pub fn stats(&self) -> HistogramStats {
        let observations = self.observations();

        if observations.is_empty() {
            return HistogramStats::default();
        }

        let mut sorted = observations.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let count = sorted.len();
        let sum: f64 = sorted.iter().sum();
        let mean = sum / count as f64;

        let min = sorted[0];
        let max = sorted[count - 1];

        let p50 = sorted[count / 2];
        let p95 = sorted[(count as f64 * 0.95) as usize];
        let p99 = sorted[(count as f64 * 0.99) as usize];

        HistogramStats {
            count,
            sum,
            mean,
            min,
            max,
            p50,
            p95,
            p99,
        }
    }

    /// Create a labeled version of this histogram
    pub fn with_labels(self, labels: HashMap<String, String>) -> Self {
        if let Metric::Histogram(mut histogram) = self.metric {
            histogram.labels = labels;
            Self {
                metric: Metric::Histogram(histogram),
            }
        } else {
            self
        }
    }
}

/// Timer for histogram that observes on drop
pub struct HistogramTimer {
    histogram: Histogram,
    start: Instant,
}

impl Drop for HistogramTimer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        self.histogram.observe(duration.as_secs_f64());
    }
}

/// Histogram statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramStats {
    /// Number of observations
    pub count: usize,
    /// Sum of all observations
    pub sum: f64,
    /// Mean value
    pub mean: f64,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// 50th percentile
    pub p50: f64,
    /// 95th percentile
    pub p95: f64,
    /// 99th percentile
    pub p99: f64,
}

impl Default for HistogramStats {
    fn default() -> Self {
        Self {
            count: 0,
            sum: 0.0,
            mean: 0.0,
            min: 0.0,
            max: 0.0,
            p50: 0.0,
            p95: 0.0,
            p99: 0.0,
        }
    }
}

/// Standard histogram buckets for different use cases
pub mod buckets {
    /// Default buckets (0.005 to 10 seconds)
    pub const DEFAULT: &[f64] = &[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0];

    /// Linear buckets from start to end with count buckets
    pub fn linear(start: f64, width: f64, count: usize) -> Vec<f64> {
        (0..count).map(|i| start + width * i as f64).collect()
    }

    /// Exponential buckets from start to end with count buckets
    pub fn exponential(start: f64, factor: f64, count: usize) -> Vec<f64> {
        (0..count).map(|i| start * factor.powi(i as i32)).collect()
    }
}

/// Label builder for creating metric labels
pub struct Labels {
    labels: HashMap<String, String>,
}

impl Labels {
    /// Create a new label builder
    pub fn new() -> Self {
        Self {
            labels: HashMap::new(),
        }
    }

    /// Add a label
    pub fn add(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Build the labels
    pub fn build(self) -> HashMap<String, String> {
        self.labels
    }
}

impl Default for Labels {
    fn default() -> Self {
        Self::new()
    }
}

/// StatsD client for sending metrics
pub struct StatsdClient {
    address: String,
    prefix: String,
}

impl StatsdClient {
    /// Create a new StatsD client
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            prefix: String::new(),
        }
    }

    /// Set metric prefix
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// Send a counter metric
    pub async fn counter(&self, name: &str, value: i64) -> Result<(), MetricError> {
        let metric = format!("{}{}", self.prefix, name);
        self.send(&format!("{}:{}|c", metric, value)).await
    }

    /// Send a gauge metric
    pub async fn gauge(&self, name: &str, value: f64) -> Result<(), MetricError> {
        let metric = format!("{}{}", self.prefix, name);
        self.send(&format!("{}:{}|g", metric, value)).await
    }

    /// Send a timing metric (in milliseconds)
    pub async fn timing(&self, name: &str, duration: Duration) -> Result<(), MetricError> {
        let metric = format!("{}{}", self.prefix, name);
        let ms = duration.as_millis();
        self.send(&format!("{}:{}|ms", metric, ms)).await
    }

    /// Send a histogram metric
    pub async fn histogram(&self, name: &str, value: f64) -> Result<(), MetricError> {
        let metric = format!("{}{}", self.prefix, name);
        self.send(&format!("{}:{}|h", metric, value)).await
    }

    async fn send(&self, message: &str) -> Result<(), MetricError> {
        // In a real implementation, this would send UDP packets
        // For now, just log it
        log::debug!("StatsD: {}", message);
        Ok(())
    }
}

/// Metric errors
#[derive(Debug, thiserror::Error)]
pub enum MetricError {
    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Invalid metric
    #[error("Invalid metric: {0}")]
    Invalid(String),

    /// Generic error
    #[error("Metric error: {0}")]
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_registry_creation() {
        let registry = MetricRegistry::new();
        assert_eq!(registry.metrics().len(), 0);
    }

    #[test]
    fn test_counter_metric() {
        let registry = MetricRegistry::new();
        let counter = registry.counter("requests_total", "Total number of requests");

        counter.inc();
        assert_eq!(counter.get(), 1.0);

        counter.add(5.0);
        assert_eq!(counter.get(), 6.0);
    }

    #[test]
    fn test_gauge_metric() {
        let registry = MetricRegistry::new();
        let gauge = registry.gauge("temperature", "Current temperature");

        gauge.set(25.5);
        assert_eq!(gauge.get(), 25.5);

        gauge.inc();
        assert_eq!(gauge.get(), 26.5);

        gauge.dec();
        assert_eq!(gauge.get(), 25.5);

        gauge.add(10.0);
        assert_eq!(gauge.get(), 35.5);

        gauge.sub(5.0);
        assert_eq!(gauge.get(), 30.5);
    }

    #[test]
    fn test_histogram_metric() {
        let registry = MetricRegistry::new();
        let histogram = registry.histogram(
            "request_duration",
            "Request duration in seconds",
            vec![0.1, 0.5, 1.0, 5.0],
        );

        histogram.observe(0.05);
        histogram.observe(0.3);
        histogram.observe(0.8);
        histogram.observe(2.0);

        let observations = histogram.observations();
        assert_eq!(observations.len(), 4);

        let stats = histogram.stats();
        assert_eq!(stats.count, 4);
        assert!(stats.mean > 0.0);
        assert_eq!(stats.min, 0.05);
        assert_eq!(stats.max, 2.0);
    }

    #[test]
    fn test_histogram_timer() {
        let registry = MetricRegistry::new();
        let histogram = registry.histogram(
            "operation_duration",
            "Operation duration",
            vec![0.1, 0.5, 1.0],
        );

        {
            let _timer = histogram.start_timer();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        let observations = histogram.observations();
        assert_eq!(observations.len(), 1);
        assert!(observations[0] > 0.0);
    }

    #[test]
    fn test_histogram_time_function() {
        let registry = MetricRegistry::new();
        let histogram = registry.histogram(
            "computation",
            "Computation time",
            vec![0.1, 0.5, 1.0],
        );

        let result = histogram.time(|| {
            std::thread::sleep(std::time::Duration::from_millis(10));
            42
        });

        assert_eq!(result, 42);
        assert_eq!(histogram.observations().len(), 1);
    }

    #[test]
    fn test_prometheus_export() {
        let registry = MetricRegistry::new();
        let counter = registry.counter("test_counter", "Test counter metric");
        counter.add(42.0);

        let gauge = registry.gauge("test_gauge", "Test gauge metric");
        gauge.set(3.14);

        let output = registry.prometheus_export();
        assert!(output.contains("test_counter"));
        assert!(output.contains("test_gauge"));
        assert!(output.contains("42"));
        assert!(output.contains("3.14"));
    }

    #[test]
    fn test_statsd_export() {
        let registry = MetricRegistry::new();
        let counter = registry.counter("test_counter", "Test counter");
        counter.add(10.0);

        let output = registry.statsd_export();
        assert_eq!(output.len(), 1);
        assert!(output[0].contains("test_counter:10"));
    }

    #[test]
    fn test_linear_buckets() {
        let buckets = buckets::linear(0.0, 10.0, 5);
        assert_eq!(buckets, vec![0.0, 10.0, 20.0, 30.0, 40.0]);
    }

    #[test]
    fn test_exponential_buckets() {
        let buckets = buckets::exponential(1.0, 2.0, 4);
        assert_eq!(buckets, vec![1.0, 2.0, 4.0, 8.0]);
    }

    #[test]
    fn test_labels() {
        let labels = Labels::new()
            .add("method", "GET")
            .add("status", "200")
            .build();

        assert_eq!(labels.len(), 2);
        assert_eq!(labels.get("method"), Some(&"GET".to_string()));
    }

    #[test]
    fn test_counter_with_labels() {
        let registry = MetricRegistry::new();
        let counter = registry.counter("http_requests", "HTTP requests")
            .with_labels(
                Labels::new()
                    .add("method", "GET")
                    .add("status", "200")
                    .build()
            );

        counter.inc();
        assert_eq!(counter.get(), 1.0);
    }

    #[tokio::test]
    async fn test_statsd_client() {
        let client = StatsdClient::new("localhost:8125")
            .with_prefix("caddy.");

        assert!(client.counter("requests", 1).await.is_ok());
        assert!(client.gauge("cpu", 0.75).await.is_ok());
        assert!(client.timing("request_duration", Duration::from_millis(100)).await.is_ok());
    }

    #[test]
    fn test_histogram_stats_empty() {
        let stats = HistogramStats::default();
        assert_eq!(stats.count, 0);
        assert_eq!(stats.sum, 0.0);
    }

    #[test]
    fn test_gauge_set_to_current_time() {
        let registry = MetricRegistry::new();
        let gauge = registry.gauge("timestamp", "Current timestamp");

        gauge.set_to_current_time();
        let value = gauge.get();

        assert!(value > 0.0);
    }
}
