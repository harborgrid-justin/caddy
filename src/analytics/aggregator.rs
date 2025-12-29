//! # Time-Series Aggregation
//!
//! Aggregates metrics over time windows for efficient querying and visualization.

use super::{Metric, MetricType, MetricValue, Result, AnalyticsError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Duration, Utc};
use parking_lot::RwLock;
use std::sync::Arc;

/// Aggregation window configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregationWindow {
    /// 1 second resolution
    Second,
    /// 1 minute resolution
    Minute,
    /// 5 minute resolution
    FiveMinute,
    /// 15 minute resolution
    FifteenMinute,
    /// 1 hour resolution
    Hour,
    /// 1 day resolution
    Day,
    /// 1 week resolution
    Week,
}

impl AggregationWindow {
    /// Get the duration of this window
    pub fn duration(&self) -> Duration {
        match self {
            Self::Second => Duration::seconds(1),
            Self::Minute => Duration::minutes(1),
            Self::FiveMinute => Duration::minutes(5),
            Self::FifteenMinute => Duration::minutes(15),
            Self::Hour => Duration::hours(1),
            Self::Day => Duration::days(1),
            Self::Week => Duration::weeks(1),
        }
    }

    /// Align a timestamp to the window boundary
    pub fn align(&self, timestamp: DateTime<Utc>) -> DateTime<Utc> {
        let duration = self.duration();
        let seconds = timestamp.timestamp();
        let window_seconds = duration.num_seconds();
        let aligned_seconds = (seconds / window_seconds) * window_seconds;

        DateTime::from_timestamp(aligned_seconds, 0).unwrap_or(timestamp)
    }

    /// Get all standard rollup windows
    pub fn rollup_hierarchy() -> Vec<Self> {
        vec![
            Self::Second,
            Self::Minute,
            Self::FiveMinute,
            Self::FifteenMinute,
            Self::Hour,
            Self::Day,
            Self::Week,
        ]
    }
}

/// Aggregated metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetric {
    /// Metric name
    pub name: String,

    /// Aggregation window
    pub window: AggregationWindow,

    /// Window start time
    pub timestamp: DateTime<Utc>,

    /// Aggregated statistics
    pub stats: AggregationStats,

    /// Labels
    pub labels: HashMap<String, String>,
}

/// Aggregation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationStats {
    /// Number of data points
    pub count: u64,

    /// Sum of all values
    pub sum: f64,

    /// Minimum value
    pub min: f64,

    /// Maximum value
    pub max: f64,

    /// Average value
    pub avg: f64,

    /// Median value (50th percentile)
    pub median: f64,

    /// 95th percentile
    pub p95: f64,

    /// 99th percentile
    pub p99: f64,

    /// Standard deviation
    pub stddev: f64,

    /// First value in window
    pub first: f64,

    /// Last value in window
    pub last: f64,
}

impl AggregationStats {
    /// Create stats from a collection of values
    pub fn from_values(values: &[f64]) -> Self {
        if values.is_empty() {
            return Self::default();
        }

        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let count = sorted.len() as u64;
        let sum: f64 = sorted.iter().sum();
        let min = sorted[0];
        let max = sorted[sorted.len() - 1];
        let avg = sum / count as f64;

        let median = Self::percentile(&sorted, 0.5);
        let p95 = Self::percentile(&sorted, 0.95);
        let p99 = Self::percentile(&sorted, 0.99);

        let variance: f64 = sorted.iter().map(|v| (v - avg).powi(2)).sum::<f64>() / count as f64;
        let stddev = variance.sqrt();

        let first = values[0];
        let last = values[values.len() - 1];

        Self {
            count,
            sum,
            min,
            max,
            avg,
            median,
            p95,
            p99,
            stddev,
            first,
            last,
        }
    }

    fn percentile(sorted_values: &[f64], p: f64) -> f64 {
        if sorted_values.is_empty() {
            return 0.0;
        }

        let index = (p * (sorted_values.len() - 1) as f64) as usize;
        sorted_values[index.min(sorted_values.len() - 1)]
    }
}

impl Default for AggregationStats {
    fn default() -> Self {
        Self {
            count: 0,
            sum: 0.0,
            min: 0.0,
            max: 0.0,
            avg: 0.0,
            median: 0.0,
            p95: 0.0,
            p99: 0.0,
            stddev: 0.0,
            first: 0.0,
            last: 0.0,
        }
    }
}

/// Metrics aggregator
pub struct Aggregator {
    /// Default aggregation window
    default_window: AggregationWindow,

    /// Aggregated data by window and metric name
    aggregated_data: Arc<RwLock<HashMap<(AggregationWindow, String), Vec<AggregatedMetric>>>>,

    /// Raw data buffer for aggregation
    raw_buffer: Arc<RwLock<Vec<Metric>>>,
}

impl Aggregator {
    /// Create a new aggregator
    pub fn new(window_seconds: u64) -> Self {
        let default_window = match window_seconds {
            1 => AggregationWindow::Second,
            60 => AggregationWindow::Minute,
            300 => AggregationWindow::FiveMinute,
            900 => AggregationWindow::FifteenMinute,
            3600 => AggregationWindow::Hour,
            86400 => AggregationWindow::Day,
            604800 => AggregationWindow::Week,
            _ => AggregationWindow::Minute,
        };

        Self {
            default_window,
            aggregated_data: Arc::new(RwLock::new(HashMap::new())),
            raw_buffer: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add metrics to the aggregation buffer
    pub fn add_metrics(&self, metrics: Vec<Metric>) {
        self.raw_buffer.write().extend(metrics);
    }

    /// Perform aggregation on buffered metrics
    pub fn aggregate(&self) -> Result<Vec<AggregatedMetric>> {
        let mut buffer = self.raw_buffer.write();
        let metrics = buffer.drain(..).collect::<Vec<_>>();
        drop(buffer);

        let mut result = Vec::new();

        // Group metrics by name and window
        for window in AggregationWindow::rollup_hierarchy() {
            let aggregated = self.aggregate_for_window(&metrics, window)?;
            result.extend(aggregated.clone());

            // Store aggregated data
            for agg in aggregated {
                let mut data = self.aggregated_data.write();
                let key = (window, agg.name.clone());
                data.entry(key).or_insert_with(Vec::new).push(agg);
            }
        }

        Ok(result)
    }

    /// Aggregate metrics for a specific window
    fn aggregate_for_window(
        &self,
        metrics: &[Metric],
        window: AggregationWindow,
    ) -> Result<Vec<AggregatedMetric>> {
        let mut grouped: HashMap<(String, DateTime<Utc>, String), Vec<f64>> = HashMap::new();

        for metric in metrics {
            let aligned_time = window.align(metric.timestamp);

            // Create a key from metric name, aligned time, and labels
            let labels_key = serde_json::to_string(&metric.labels)
                .unwrap_or_else(|_| String::new());

            let key = (metric.name.clone(), aligned_time, labels_key);

            if let Some(value) = metric.value.as_f64() {
                grouped.entry(key).or_insert_with(Vec::new).push(value);
            }
        }

        let result = grouped
            .into_iter()
            .map(|((name, timestamp, labels_key), values)| {
                let stats = AggregationStats::from_values(&values);
                let labels: HashMap<String, String> =
                    serde_json::from_str(&labels_key).unwrap_or_default();

                AggregatedMetric {
                    name,
                    window,
                    timestamp,
                    stats,
                    labels,
                }
            })
            .collect();

        Ok(result)
    }

    /// Query aggregated data
    pub fn query(
        &self,
        metric_name: &str,
        window: AggregationWindow,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Vec<AggregatedMetric> {
        let data = self.aggregated_data.read();
        let key = (window, metric_name.to_string());

        data.get(&key)
            .map(|metrics| {
                metrics
                    .iter()
                    .filter(|m| m.timestamp >= start_time && m.timestamp <= end_time)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get the latest aggregated value for a metric
    pub fn latest(&self, metric_name: &str) -> Option<AggregatedMetric> {
        let data = self.aggregated_data.read();
        let key = (self.default_window, metric_name.to_string());

        data.get(&key)
            .and_then(|metrics| metrics.last().cloned())
    }

    /// Calculate rate of change
    pub fn rate(
        &self,
        metric_name: &str,
        window: AggregationWindow,
        period: Duration,
    ) -> Option<f64> {
        let end_time = Utc::now();
        let start_time = end_time - period;

        let data = self.query(metric_name, window, start_time, end_time);

        if data.len() < 2 {
            return None;
        }

        let first = &data[0];
        let last = &data[data.len() - 1];

        let value_delta = last.stats.last - first.stats.first;
        let time_delta = (last.timestamp - first.timestamp).num_seconds() as f64;

        if time_delta > 0.0 {
            Some(value_delta / time_delta)
        } else {
            None
        }
    }

    /// Downsample aggregated data to a coarser window
    pub fn downsample(
        &self,
        from_window: AggregationWindow,
        to_window: AggregationWindow,
    ) -> Result<()> {
        if to_window.duration() <= from_window.duration() {
            return Err(AnalyticsError::Aggregation(
                "Target window must be larger than source window".to_string(),
            ));
        }

        // Implementation would group by coarser window and re-aggregate
        Ok(())
    }

    /// Clear old aggregated data
    pub fn cleanup(&self, retention_period: Duration) {
        let cutoff = Utc::now() - retention_period;
        let mut data = self.aggregated_data.write();

        for metrics in data.values_mut() {
            metrics.retain(|m| m.timestamp >= cutoff);
        }
    }

    /// Get total aggregated metrics count
    pub fn total_aggregated(&self) -> usize {
        self.aggregated_data
            .read()
            .values()
            .map(|v| v.len())
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregation_window_alignment() {
        let window = AggregationWindow::Minute;
        let timestamp = DateTime::from_timestamp(1234567, 0).unwrap();
        let aligned = window.align(timestamp);

        assert_eq!(aligned.timestamp() % 60, 0);
    }

    #[test]
    fn test_aggregation_stats() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = AggregationStats::from_values(&values);

        assert_eq!(stats.count, 5);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
        assert_eq!(stats.avg, 3.0);
        assert_eq!(stats.sum, 15.0);
    }

    #[test]
    fn test_aggregator_creation() {
        let aggregator = Aggregator::new(60);
        assert_eq!(aggregator.default_window, AggregationWindow::Minute);
    }

    #[test]
    fn test_empty_stats() {
        let stats = AggregationStats::from_values(&[]);
        assert_eq!(stats.count, 0);
        assert_eq!(stats.sum, 0.0);
    }
}
