//! Core metrics collection and management
//!
//! This module provides thread-safe metric types and a centralized registry
//! for collecting and managing performance metrics.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Labels for metric dimensions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Labels {
    labels: HashMap<String, String>,
}

impl Labels {
    /// Create a new empty label set
    pub fn new() -> Self {
        Self {
            labels: HashMap::new(),
        }
    }

    /// Create labels from key-value pairs
    pub fn from_pairs(pairs: &[(&str, &str)]) -> Self {
        let mut labels = HashMap::new();
        for (k, v) in pairs {
            labels.insert(k.to_string(), v.to_string());
        }
        Self { labels }
    }

    /// Add a label
    pub fn add(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.labels.insert(key.into(), value.into());
    }

    /// Get a label value
    pub fn get(&self, key: &str) -> Option<&str> {
        self.labels.get(key).map(|s| s.as_str())
    }

    /// Get all labels
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.labels.iter()
    }

    /// Check if labels are empty
    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }
}

impl Default for Labels {
    fn default() -> Self {
        Self::new()
    }
}

/// Metric value types
#[derive(Debug, Clone, Copy)]
pub enum MetricValue {
    Counter(f64),
    Gauge(f64),
    Histogram { sum: f64, count: u64, buckets: [u64; 10] },
    Summary { sum: f64, count: u64, quantiles: [f64; 5] },
}

impl MetricValue {
    /// Get the numeric value for simple metrics
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Counter(v) | Self::Gauge(v) => Some(*v),
            _ => None,
        }
    }
}

/// Metric types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

/// A counter metric that only increases
#[derive(Debug)]
pub struct Counter {
    value: Arc<RwLock<f64>>,
    labels: Labels,
}

impl Counter {
    /// Create a new counter
    pub fn new(labels: Labels) -> Self {
        Self {
            value: Arc::new(RwLock::new(0.0)),
            labels,
        }
    }

    /// Increment the counter by 1
    pub fn inc(&self) {
        self.add(1.0);
    }

    /// Add a value to the counter
    pub fn add(&self, value: f64) {
        if value >= 0.0 {
            let mut v = self.value.write().unwrap();
            *v += value;
        }
    }

    /// Get the current value
    pub fn get(&self) -> f64 {
        *self.value.read().unwrap()
    }

    /// Reset the counter to zero
    pub fn reset(&self) {
        let mut v = self.value.write().unwrap();
        *v = 0.0;
    }

    /// Get labels
    pub fn labels(&self) -> &Labels {
        &self.labels
    }
}

impl Clone for Counter {
    fn clone(&self) -> Self {
        Self {
            value: Arc::clone(&self.value),
            labels: self.labels.clone(),
        }
    }
}

/// A gauge metric that can increase or decrease
#[derive(Debug)]
pub struct Gauge {
    value: Arc<RwLock<f64>>,
    labels: Labels,
}

impl Gauge {
    /// Create a new gauge
    pub fn new(labels: Labels) -> Self {
        Self {
            value: Arc::new(RwLock::new(0.0)),
            labels,
        }
    }

    /// Set the gauge to a specific value
    pub fn set(&self, value: f64) {
        let mut v = self.value.write().unwrap();
        *v = value;
    }

    /// Increment the gauge
    pub fn inc(&self) {
        self.add(1.0);
    }

    /// Decrement the gauge
    pub fn dec(&self) {
        self.sub(1.0);
    }

    /// Add to the gauge
    pub fn add(&self, value: f64) {
        let mut v = self.value.write().unwrap();
        *v += value;
    }

    /// Subtract from the gauge
    pub fn sub(&self, value: f64) {
        let mut v = self.value.write().unwrap();
        *v -= value;
    }

    /// Get the current value
    pub fn get(&self) -> f64 {
        *self.value.read().unwrap()
    }

    /// Get labels
    pub fn labels(&self) -> &Labels {
        &self.labels
    }
}

impl Clone for Gauge {
    fn clone(&self) -> Self {
        Self {
            value: Arc::clone(&self.value),
            labels: self.labels.clone(),
        }
    }
}

/// A histogram metric for distribution tracking
#[derive(Debug)]
pub struct Histogram {
    sum: Arc<RwLock<f64>>,
    count: Arc<RwLock<u64>>,
    buckets: Arc<RwLock<Vec<(f64, u64)>>>, // (upper_bound, count)
    labels: Labels,
}

impl Histogram {
    /// Create a new histogram with default buckets
    pub fn new(labels: Labels) -> Self {
        Self::with_buckets(
            labels,
            vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        )
    }

    /// Create a histogram with custom buckets
    pub fn with_buckets(labels: Labels, bounds: Vec<f64>) -> Self {
        let buckets = bounds.into_iter().map(|b| (b, 0)).collect();
        Self {
            sum: Arc::new(RwLock::new(0.0)),
            count: Arc::new(RwLock::new(0)),
            buckets: Arc::new(RwLock::new(buckets)),
            labels,
        }
    }

    /// Observe a value
    pub fn observe(&self, value: f64) {
        // Update sum and count
        {
            let mut sum = self.sum.write().unwrap();
            *sum += value;
        }
        {
            let mut count = self.count.write().unwrap();
            *count += 1;
        }

        // Update buckets
        let mut buckets = self.buckets.write().unwrap();
        for (bound, count) in buckets.iter_mut() {
            if value <= *bound {
                *count += 1;
            }
        }
    }

    /// Get the sum of all observed values
    pub fn sum(&self) -> f64 {
        *self.sum.read().unwrap()
    }

    /// Get the count of observations
    pub fn count(&self) -> u64 {
        *self.count.read().unwrap()
    }

    /// Get bucket counts
    pub fn buckets(&self) -> Vec<(f64, u64)> {
        self.buckets.read().unwrap().clone()
    }

    /// Get labels
    pub fn labels(&self) -> &Labels {
        &self.labels
    }

    /// Calculate mean
    pub fn mean(&self) -> f64 {
        let sum = self.sum();
        let count = self.count();
        if count > 0 {
            sum / count as f64
        } else {
            0.0
        }
    }
}

impl Clone for Histogram {
    fn clone(&self) -> Self {
        Self {
            sum: Arc::clone(&self.sum),
            count: Arc::clone(&self.count),
            buckets: Arc::clone(&self.buckets),
            labels: self.labels.clone(),
        }
    }
}

/// A summary metric for quantile tracking
#[derive(Debug)]
pub struct Summary {
    sum: Arc<RwLock<f64>>,
    count: Arc<RwLock<u64>>,
    values: Arc<RwLock<Vec<f64>>>,
    quantiles: Vec<f64>, // e.g., [0.5, 0.9, 0.95, 0.99]
    labels: Labels,
    max_values: usize,
}

impl Summary {
    /// Create a new summary with default quantiles (0.5, 0.9, 0.95, 0.99, 1.0)
    pub fn new(labels: Labels) -> Self {
        Self::with_quantiles(labels, vec![0.5, 0.9, 0.95, 0.99, 1.0], 1000)
    }

    /// Create a summary with custom quantiles
    pub fn with_quantiles(labels: Labels, quantiles: Vec<f64>, max_values: usize) -> Self {
        Self {
            sum: Arc::new(RwLock::new(0.0)),
            count: Arc::new(RwLock::new(0)),
            values: Arc::new(RwLock::new(Vec::new())),
            quantiles,
            labels,
            max_values,
        }
    }

    /// Observe a value
    pub fn observe(&self, value: f64) {
        {
            let mut sum = self.sum.write().unwrap();
            *sum += value;
        }
        {
            let mut count = self.count.write().unwrap();
            *count += 1;
        }
        {
            let mut values = self.values.write().unwrap();
            values.push(value);
            // Keep only recent values to prevent unbounded growth
            if values.len() > self.max_values {
                let drain_count = values.len() - self.max_values;
                values.drain(0..drain_count);
            }
        }
    }

    /// Get the sum of all observed values
    pub fn sum(&self) -> f64 {
        *self.sum.read().unwrap()
    }

    /// Get the count of observations
    pub fn count(&self) -> u64 {
        *self.count.read().unwrap()
    }

    /// Calculate quantile values
    pub fn quantiles(&self) -> Vec<(f64, f64)> {
        let mut values = self.values.read().unwrap().clone();
        if values.is_empty() {
            return self.quantiles.iter().map(|q| (*q, 0.0)).collect();
        }

        values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        self.quantiles
            .iter()
            .map(|&q| {
                let idx = ((values.len() as f64 - 1.0) * q) as usize;
                (q, values[idx])
            })
            .collect()
    }

    /// Get labels
    pub fn labels(&self) -> &Labels {
        &self.labels
    }

    /// Calculate mean
    pub fn mean(&self) -> f64 {
        let sum = self.sum();
        let count = self.count();
        if count > 0 {
            sum / count as f64
        } else {
            0.0
        }
    }
}

impl Clone for Summary {
    fn clone(&self) -> Self {
        Self {
            sum: Arc::clone(&self.sum),
            count: Arc::clone(&self.count),
            values: Arc::clone(&self.values),
            quantiles: self.quantiles.clone(),
            labels: self.labels.clone(),
            max_values: self.max_values,
        }
    }
}

/// Generic metric wrapper
#[derive(Debug, Clone)]
pub enum Metric {
    Counter(Counter),
    Gauge(Gauge),
    Histogram(Histogram),
    Summary(Summary),
}

impl Metric {
    /// Get the metric type
    pub fn metric_type(&self) -> MetricType {
        match self {
            Self::Counter(_) => MetricType::Counter,
            Self::Gauge(_) => MetricType::Gauge,
            Self::Histogram(_) => MetricType::Histogram,
            Self::Summary(_) => MetricType::Summary,
        }
    }

    /// Get the metric labels
    pub fn labels(&self) -> &Labels {
        match self {
            Self::Counter(m) => m.labels(),
            Self::Gauge(m) => m.labels(),
            Self::Histogram(m) => m.labels(),
            Self::Summary(m) => m.labels(),
        }
    }
}

/// Centralized metric registry
#[derive(Debug, Clone)]
pub struct MetricRegistry {
    metrics: Arc<RwLock<HashMap<String, Metric>>>,
    namespace: String,
}

impl MetricRegistry {
    /// Create a new metric registry
    pub fn new() -> Self {
        Self::with_namespace("caddy")
    }

    /// Create a registry with a custom namespace
    pub fn with_namespace(namespace: impl Into<String>) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            namespace: namespace.into(),
        }
    }

    /// Register or get a counter
    pub fn counter(&self, name: &str, labels: Labels) -> Counter {
        let full_name = self.make_name(name);
        let mut metrics = self.metrics.write().unwrap();

        match metrics.get(&full_name) {
            Some(Metric::Counter(c)) => c.clone(),
            _ => {
                let counter = Counter::new(labels);
                metrics.insert(full_name, Metric::Counter(counter.clone()));
                counter
            }
        }
    }

    /// Register or get a gauge
    pub fn gauge(&self, name: &str, labels: Labels) -> Gauge {
        let full_name = self.make_name(name);
        let mut metrics = self.metrics.write().unwrap();

        match metrics.get(&full_name) {
            Some(Metric::Gauge(g)) => g.clone(),
            _ => {
                let gauge = Gauge::new(labels);
                metrics.insert(full_name, Metric::Gauge(gauge.clone()));
                gauge
            }
        }
    }

    /// Register or get a histogram
    pub fn histogram(&self, name: &str, labels: Labels) -> Histogram {
        let full_name = self.make_name(name);
        let mut metrics = self.metrics.write().unwrap();

        match metrics.get(&full_name) {
            Some(Metric::Histogram(h)) => h.clone(),
            _ => {
                let histogram = Histogram::new(labels);
                metrics.insert(full_name, Metric::Histogram(histogram.clone()));
                histogram
            }
        }
    }

    /// Register or get a summary
    pub fn summary(&self, name: &str, labels: Labels) -> Summary {
        let full_name = self.make_name(name);
        let mut metrics = self.metrics.write().unwrap();

        match metrics.get(&full_name) {
            Some(Metric::Summary(s)) => s.clone(),
            _ => {
                let summary = Summary::new(labels);
                metrics.insert(full_name, Metric::Summary(summary.clone()));
                summary
            }
        }
    }

    /// Get all registered metrics
    pub fn metrics(&self) -> Vec<(String, Metric)> {
        self.metrics
            .read()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Get a specific metric by name
    pub fn get(&self, name: &str) -> Option<Metric> {
        let full_name = self.make_name(name);
        self.metrics.read().unwrap().get(&full_name).cloned()
    }

    /// Remove a metric
    pub fn remove(&self, name: &str) -> Option<Metric> {
        let full_name = self.make_name(name);
        self.metrics.write().unwrap().remove(&full_name)
    }

    /// Clear all metrics
    pub fn clear(&self) {
        self.metrics.write().unwrap().clear();
    }

    /// Get the number of registered metrics
    pub fn len(&self) -> usize {
        self.metrics.read().unwrap().len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.metrics.read().unwrap().is_empty()
    }

    /// Get current timestamp
    pub fn timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn make_name(&self, name: &str) -> String {
        format!("{}_{}", self.namespace, name)
    }
}

impl Default for MetricRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_labels() {
        let mut labels = Labels::new();
        labels.add("env", "prod");
        labels.add("region", "us-west");

        assert_eq!(labels.get("env"), Some("prod"));
        assert_eq!(labels.get("region"), Some("us-west"));
        assert_eq!(labels.get("missing"), None);
    }

    #[test]
    fn test_counter() {
        let counter = Counter::new(Labels::new());
        assert_eq!(counter.get(), 0.0);

        counter.inc();
        assert_eq!(counter.get(), 1.0);

        counter.add(5.0);
        assert_eq!(counter.get(), 6.0);

        counter.reset();
        assert_eq!(counter.get(), 0.0);
    }

    #[test]
    fn test_gauge() {
        let gauge = Gauge::new(Labels::new());
        assert_eq!(gauge.get(), 0.0);

        gauge.set(10.0);
        assert_eq!(gauge.get(), 10.0);

        gauge.inc();
        assert_eq!(gauge.get(), 11.0);

        gauge.dec();
        assert_eq!(gauge.get(), 10.0);

        gauge.add(5.0);
        assert_eq!(gauge.get(), 15.0);

        gauge.sub(3.0);
        assert_eq!(gauge.get(), 12.0);
    }

    #[test]
    fn test_histogram() {
        let histogram = Histogram::new(Labels::new());

        histogram.observe(0.5);
        histogram.observe(1.5);
        histogram.observe(3.0);

        assert_eq!(histogram.count(), 3);
        assert_eq!(histogram.sum(), 5.0);
        assert!((histogram.mean() - 1.666).abs() < 0.01);
    }

    #[test]
    fn test_summary() {
        let summary = Summary::new(Labels::new());

        for i in 1..=100 {
            summary.observe(i as f64);
        }

        assert_eq!(summary.count(), 100);
        assert_eq!(summary.sum(), 5050.0);
        assert_eq!(summary.mean(), 50.5);

        let quantiles = summary.quantiles();
        assert!(!quantiles.is_empty());
    }

    #[test]
    fn test_registry() {
        let registry = MetricRegistry::new();

        let counter = registry.counter("requests", Labels::new());
        counter.inc();

        assert_eq!(registry.len(), 1);

        let gauge = registry.gauge("memory", Labels::new());
        gauge.set(100.0);

        assert_eq!(registry.len(), 2);

        registry.clear();
        assert_eq!(registry.len(), 0);
    }
}
