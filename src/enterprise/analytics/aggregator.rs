//! Data aggregation and statistical analysis
//!
//! This module provides time-series aggregation, rolling windows,
//! and statistical functions for metrics analysis.

use super::Result;
use std::collections::VecDeque;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Time window for aggregation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeWindow {
    /// Last minute
    Minute,
    /// Last 5 minutes
    FiveMinutes,
    /// Last 15 minutes
    FifteenMinutes,
    /// Last hour
    Hour,
    /// Last 6 hours
    SixHours,
    /// Last day
    Day,
    /// Last week
    Week,
    /// Custom duration in seconds
    Custom(u64),
}

impl TimeWindow {
    /// Get the duration in seconds
    pub fn as_secs(&self) -> u64 {
        match self {
            Self::Minute => 60,
            Self::FiveMinutes => 300,
            Self::FifteenMinutes => 900,
            Self::Hour => 3600,
            Self::SixHours => 21600,
            Self::Day => 86400,
            Self::Week => 604800,
            Self::Custom(secs) => *secs,
        }
    }

    /// Get the duration
    pub fn as_duration(&self) -> Duration {
        Duration::from_secs(self.as_secs())
    }
}

/// Percentile values
#[derive(Debug, Clone, Copy)]
pub struct Percentile {
    /// p50 (median)
    pub p50: f64,
    /// p90
    pub p90: f64,
    /// p95
    pub p95: f64,
    /// p99
    pub p99: f64,
    /// p999
    pub p999: f64,
}

impl Percentile {
    /// Calculate percentiles from a sorted vector
    pub fn from_sorted(values: &[f64]) -> Self {
        if values.is_empty() {
            return Self {
                p50: 0.0,
                p90: 0.0,
                p95: 0.0,
                p99: 0.0,
                p999: 0.0,
            };
        }

        Self {
            p50: Self::percentile(values, 0.50),
            p90: Self::percentile(values, 0.90),
            p95: Self::percentile(values, 0.95),
            p99: Self::percentile(values, 0.99),
            p999: Self::percentile(values, 0.999),
        }
    }

    fn percentile(sorted_values: &[f64], p: f64) -> f64 {
        let idx = ((sorted_values.len() as f64 - 1.0) * p) as usize;
        sorted_values[idx.min(sorted_values.len() - 1)]
    }
}

/// Time-series data point
#[derive(Debug, Clone, Copy)]
pub struct DataPoint {
    /// Timestamp (Unix epoch seconds)
    pub timestamp: u64,
    /// Value
    pub value: f64,
}

impl DataPoint {
    /// Create a new data point
    pub fn new(timestamp: u64, value: f64) -> Self {
        Self { timestamp, value }
    }

    /// Create a data point with current timestamp
    pub fn now(value: f64) -> Self {
        Self {
            timestamp: current_timestamp(),
            value,
        }
    }
}

/// Aggregation configuration
#[derive(Debug, Clone)]
pub struct AggregationConfig {
    /// Time window for aggregation
    pub window: TimeWindow,
    /// Maximum number of data points to keep
    pub max_points: usize,
    /// Enable automatic downsampling
    pub downsample: bool,
    /// Downsampling interval in seconds
    pub downsample_interval: u64,
}

impl Default for AggregationConfig {
    fn default() -> Self {
        Self {
            window: TimeWindow::Hour,
            max_points: 1000,
            downsample: true,
            downsample_interval: 60,
        }
    }
}

/// Data aggregator for time-series analysis
pub struct Aggregator {
    config: AggregationConfig,
    data_points: VecDeque<DataPoint>,
}

impl Aggregator {
    /// Create a new aggregator
    pub fn new(config: AggregationConfig) -> Self {
        Self {
            config,
            data_points: VecDeque::new(),
        }
    }

    /// Create an aggregator with default configuration
    pub fn default() -> Self {
        Self::new(AggregationConfig::default())
    }

    /// Add a data point
    pub fn add(&mut self, point: DataPoint) {
        self.data_points.push_back(point);
        self.cleanup();

        if self.config.downsample && self.data_points.len() > self.config.max_points {
            self.downsample();
        }
    }

    /// Add a value with current timestamp
    pub fn add_value(&mut self, value: f64) {
        self.add(DataPoint::now(value));
    }

    /// Get all data points within the time window
    pub fn data_points(&self, window: Option<TimeWindow>) -> Vec<DataPoint> {
        let window = window.unwrap_or(self.config.window);
        let cutoff = current_timestamp().saturating_sub(window.as_secs());

        self.data_points
            .iter()
            .filter(|p| p.timestamp >= cutoff)
            .copied()
            .collect()
    }

    /// Calculate mean value
    pub fn mean(&self, window: Option<TimeWindow>) -> f64 {
        let points = self.data_points(window);
        if points.is_empty() {
            return 0.0;
        }

        let sum: f64 = points.iter().map(|p| p.value).sum();
        sum / points.len() as f64
    }

    /// Calculate median value
    pub fn median(&self, window: Option<TimeWindow>) -> f64 {
        let mut values: Vec<f64> = self.data_points(window)
            .iter()
            .map(|p| p.value)
            .collect();

        if values.is_empty() {
            return 0.0;
        }

        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = values.len() / 2;

        if values.len() % 2 == 0 {
            (values[mid - 1] + values[mid]) / 2.0
        } else {
            values[mid]
        }
    }

    /// Calculate standard deviation
    pub fn std_dev(&self, window: Option<TimeWindow>) -> f64 {
        let points = self.data_points(window);
        if points.len() < 2 {
            return 0.0;
        }

        let mean = self.mean(window);
        let variance: f64 = points
            .iter()
            .map(|p| {
                let diff = p.value - mean;
                diff * diff
            })
            .sum::<f64>()
            / points.len() as f64;

        variance.sqrt()
    }

    /// Calculate min value
    pub fn min(&self, window: Option<TimeWindow>) -> Option<f64> {
        self.data_points(window)
            .iter()
            .map(|p| p.value)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
    }

    /// Calculate max value
    pub fn max(&self, window: Option<TimeWindow>) -> Option<f64> {
        self.data_points(window)
            .iter()
            .map(|p| p.value)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
    }

    /// Calculate percentiles
    pub fn percentiles(&self, window: Option<TimeWindow>) -> Percentile {
        let mut values: Vec<f64> = self.data_points(window)
            .iter()
            .map(|p| p.value)
            .collect();

        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        Percentile::from_sorted(&values)
    }

    /// Calculate sum
    pub fn sum(&self, window: Option<TimeWindow>) -> f64 {
        self.data_points(window)
            .iter()
            .map(|p| p.value)
            .sum()
    }

    /// Calculate count
    pub fn count(&self, window: Option<TimeWindow>) -> usize {
        self.data_points(window).len()
    }

    /// Calculate rate of change (per second)
    pub fn rate(&self, window: Option<TimeWindow>) -> f64 {
        let points = self.data_points(window);
        if points.len() < 2 {
            return 0.0;
        }

        let first = points.first().unwrap();
        let last = points.last().unwrap();

        let value_diff = last.value - first.value;
        let time_diff = last.timestamp.saturating_sub(first.timestamp) as f64;

        if time_diff > 0.0 {
            value_diff / time_diff
        } else {
            0.0
        }
    }

    /// Calculate moving average
    pub fn moving_average(&self, window: TimeWindow, points: usize) -> Vec<DataPoint> {
        let all_points = self.data_points(Some(window));
        if all_points.len() < points {
            return Vec::new();
        }

        let mut result = Vec::new();
        for i in points..=all_points.len() {
            let slice = &all_points[i - points..i];
            let avg = slice.iter().map(|p| p.value).sum::<f64>() / points as f64;
            let timestamp = slice.last().unwrap().timestamp;
            result.push(DataPoint::new(timestamp, avg));
        }

        result
    }

    /// Detect anomalies using standard deviation
    pub fn detect_anomalies(&self, window: Option<TimeWindow>, threshold: f64) -> Vec<DataPoint> {
        let mean = self.mean(window);
        let std_dev = self.std_dev(window);
        let points = self.data_points(window);

        points
            .into_iter()
            .filter(|p| {
                let z_score = (p.value - mean).abs() / std_dev;
                z_score > threshold
            })
            .collect()
    }

    /// Get trend direction (positive, negative, or neutral)
    pub fn trend(&self, window: Option<TimeWindow>) -> Trend {
        let rate = self.rate(window);
        let threshold = 0.01; // 1% change threshold

        if rate > threshold {
            Trend::Increasing
        } else if rate < -threshold {
            Trend::Decreasing
        } else {
            Trend::Stable
        }
    }

    /// Downsample data points by averaging over intervals
    fn downsample(&mut self) {
        let interval = self.config.downsample_interval;
        let mut downsampled = VecDeque::new();
        let mut current_bucket: Vec<DataPoint> = Vec::new();
        let mut current_bucket_start = 0u64;

        for point in &self.data_points {
            let bucket_start = (point.timestamp / interval) * interval;

            if current_bucket_start == 0 {
                current_bucket_start = bucket_start;
            }

            if bucket_start != current_bucket_start {
                if !current_bucket.is_empty() {
                    let avg = current_bucket.iter().map(|p| p.value).sum::<f64>()
                        / current_bucket.len() as f64;
                    downsampled.push_back(DataPoint::new(current_bucket_start, avg));
                }
                current_bucket.clear();
                current_bucket_start = bucket_start;
            }

            current_bucket.push(*point);
        }

        // Add final bucket
        if !current_bucket.is_empty() {
            let avg = current_bucket.iter().map(|p| p.value).sum::<f64>()
                / current_bucket.len() as f64;
            downsampled.push_back(DataPoint::new(current_bucket_start, avg));
        }

        self.data_points = downsampled;
    }

    /// Remove data points outside the time window
    fn cleanup(&mut self) {
        let cutoff = current_timestamp().saturating_sub(self.config.window.as_secs());

        while let Some(point) = self.data_points.front() {
            if point.timestamp < cutoff {
                self.data_points.pop_front();
            } else {
                break;
            }
        }
    }
}

/// Trend direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trend {
    Increasing,
    Decreasing,
    Stable,
}

/// Get current Unix timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Statistical summary
#[derive(Debug, Clone)]
pub struct StatisticalSummary {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub median: f64,
    pub min: f64,
    pub max: f64,
    pub std_dev: f64,
    pub percentiles: Percentile,
}

impl StatisticalSummary {
    /// Create a summary from an aggregator
    pub fn from_aggregator(agg: &Aggregator, window: Option<TimeWindow>) -> Self {
        Self {
            count: agg.count(window),
            sum: agg.sum(window),
            mean: agg.mean(window),
            median: agg.median(window),
            min: agg.min(window).unwrap_or(0.0),
            max: agg.max(window).unwrap_or(0.0),
            std_dev: agg.std_dev(window),
            percentiles: agg.percentiles(window),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_window() {
        assert_eq!(TimeWindow::Minute.as_secs(), 60);
        assert_eq!(TimeWindow::Hour.as_secs(), 3600);
        assert_eq!(TimeWindow::Day.as_secs(), 86400);
        assert_eq!(TimeWindow::Custom(300).as_secs(), 300);
    }

    #[test]
    fn test_aggregator_stats() {
        let mut agg = Aggregator::default();

        // Add test data
        for i in 1..=10 {
            agg.add_value(i as f64);
        }

        assert_eq!(agg.count(None), 10);
        assert_eq!(agg.sum(None), 55.0);
        assert_eq!(agg.mean(None), 5.5);
        assert_eq!(agg.median(None), 5.5);
        assert_eq!(agg.min(None), Some(1.0));
        assert_eq!(agg.max(None), Some(10.0));
    }

    #[test]
    fn test_percentiles() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let p = Percentile::from_sorted(&values);

        assert!((p.p50 - 5.0).abs() < 0.5);
        assert!((p.p90 - 9.0).abs() < 0.5);
        assert!((p.p99 - 10.0).abs() < 0.5);
    }

    #[test]
    fn test_moving_average() {
        let mut agg = Aggregator::default();

        for i in 1..=10 {
            agg.add_value(i as f64);
        }

        let ma = agg.moving_average(TimeWindow::Hour, 3);
        assert!(!ma.is_empty());
    }

    #[test]
    fn test_trend_detection() {
        let mut agg = Aggregator::default();

        // Increasing trend
        for i in 1..=10 {
            agg.add(DataPoint::new(i, i as f64 * 10.0));
        }

        let trend = agg.trend(None);
        assert_eq!(trend, Trend::Increasing);
    }

    #[test]
    fn test_anomaly_detection() {
        let mut agg = Aggregator::default();

        // Normal values
        for _ in 0..100 {
            agg.add_value(50.0);
        }

        // Anomaly
        agg.add_value(500.0);

        let anomalies = agg.detect_anomalies(None, 3.0);
        assert_eq!(anomalies.len(), 1);
    }

    #[test]
    fn test_standard_deviation() {
        let mut agg = Aggregator::default();

        agg.add_value(10.0);
        agg.add_value(20.0);
        agg.add_value(30.0);

        let std_dev = agg.std_dev(None);
        assert!(std_dev > 0.0);
    }
}
