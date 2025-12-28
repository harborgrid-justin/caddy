//! Metrics storage and retention
//!
//! This module provides time-series database abstraction, retention policies,
//! downsampling, and export functionality.

use super::{Result, AnalyticsError};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Time-series data point
#[derive(Debug, Clone, Copy)]
pub struct TimeSeriesPoint {
    /// Timestamp (Unix epoch seconds)
    pub timestamp: u64,
    /// Metric value
    pub value: f64,
}

impl TimeSeriesPoint {
    /// Create a new time-series point
    pub fn new(timestamp: u64, value: f64) -> Self {
        Self { timestamp, value }
    }

    /// Create a point with current timestamp
    pub fn now(value: f64) -> Self {
        Self {
            timestamp: current_timestamp(),
            value,
        }
    }

    /// Serialize to string
    pub fn to_string(&self) -> String {
        format!("{},{}", self.timestamp, self.value)
    }

    /// Deserialize from string
    pub fn from_string(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 2 {
            return None;
        }

        let timestamp = parts[0].parse().ok()?;
        let value = parts[1].parse().ok()?;

        Some(Self { timestamp, value })
    }
}

/// Retention policy for metric data
#[derive(Debug, Clone)]
pub struct RetentionPolicy {
    /// Name of the retention policy
    pub name: String,
    /// Duration to keep data (in seconds)
    pub duration: u64,
    /// Resolution (downsampling interval in seconds)
    pub resolution: u64,
}

impl RetentionPolicy {
    /// Create a new retention policy
    pub fn new(name: impl Into<String>, duration: u64, resolution: u64) -> Self {
        Self {
            name: name.into(),
            duration,
            resolution,
        }
    }

    /// Default policies for common use cases
    pub fn defaults() -> Vec<Self> {
        vec![
            Self::new("raw", 3600, 1),           // 1 hour at 1s resolution
            Self::new("1min", 86400, 60),        // 1 day at 1min resolution
            Self::new("5min", 604800, 300),      // 1 week at 5min resolution
            Self::new("1hour", 2592000, 3600),   // 30 days at 1hour resolution
            Self::new("1day", 31536000, 86400),  // 1 year at 1day resolution
        ]
    }

    /// Check if a timestamp should be retained
    pub fn should_retain(&self, timestamp: u64) -> bool {
        let now = current_timestamp();
        timestamp >= now.saturating_sub(self.duration)
    }
}

/// Export formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// Comma-separated values
    Csv,
    /// JSON format
    Json,
    /// Prometheus text format
    Prometheus,
    /// InfluxDB line protocol
    InfluxDB,
}

/// Storage backend trait
pub trait StorageBackend: Send + Sync {
    /// Write a data point
    fn write(&mut self, metric: &str, point: TimeSeriesPoint) -> Result<()>;

    /// Write multiple data points
    fn write_batch(&mut self, metric: &str, points: &[TimeSeriesPoint]) -> Result<()>;

    /// Read data points for a metric
    fn read(&self, metric: &str, start: u64, end: u64) -> Result<Vec<TimeSeriesPoint>>;

    /// Delete old data points based on retention policy
    fn apply_retention(&mut self, policy: &RetentionPolicy) -> Result<usize>;

    /// Export data in specified format
    fn export(&self, metric: &str, format: ExportFormat) -> Result<String>;

    /// List all metrics
    fn list_metrics(&self) -> Result<Vec<String>>;

    /// Delete a metric
    fn delete_metric(&mut self, metric: &str) -> Result<()>;
}

/// In-memory storage backend
pub struct MemoryBackend {
    data: Arc<RwLock<HashMap<String, Vec<TimeSeriesPoint>>>>,
}

impl MemoryBackend {
    /// Create a new memory backend
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageBackend for MemoryBackend {
    fn write(&mut self, metric: &str, point: TimeSeriesPoint) -> Result<()> {
        let mut data = self.data.write().unwrap();
        data.entry(metric.to_string())
            .or_insert_with(Vec::new)
            .push(point);
        Ok(())
    }

    fn write_batch(&mut self, metric: &str, points: &[TimeSeriesPoint]) -> Result<()> {
        let mut data = self.data.write().unwrap();
        data.entry(metric.to_string())
            .or_insert_with(Vec::new)
            .extend_from_slice(points);
        Ok(())
    }

    fn read(&self, metric: &str, start: u64, end: u64) -> Result<Vec<TimeSeriesPoint>> {
        let data = self.data.read().unwrap();
        Ok(data
            .get(metric)
            .map(|points| {
                points
                    .iter()
                    .filter(|p| p.timestamp >= start && p.timestamp <= end)
                    .copied()
                    .collect()
            })
            .unwrap_or_default())
    }

    fn apply_retention(&mut self, policy: &RetentionPolicy) -> Result<usize> {
        let mut data = self.data.write().unwrap();
        let mut removed = 0;

        for points in data.values_mut() {
            let original_len = points.len();
            points.retain(|p| policy.should_retain(p.timestamp));
            removed += original_len - points.len();
        }

        Ok(removed)
    }

    fn export(&self, metric: &str, format: ExportFormat) -> Result<String> {
        let data = self.data.read().unwrap();
        let points = data.get(metric).ok_or_else(|| {
            AnalyticsError::StorageError(format!("Metric not found: {}", metric))
        })?;

        match format {
            ExportFormat::Csv => {
                let mut csv = String::from("timestamp,value\n");
                for point in points {
                    csv.push_str(&format!("{},{}\n", point.timestamp, point.value));
                }
                Ok(csv)
            }
            ExportFormat::Json => {
                let json_points: Vec<String> = points
                    .iter()
                    .map(|p| format!("{{\"timestamp\":{},\"value\":{}}}", p.timestamp, p.value))
                    .collect();
                Ok(format!("[{}]", json_points.join(",")))
            }
            ExportFormat::Prometheus => {
                let mut prom = String::new();
                for point in points {
                    prom.push_str(&format!(
                        "{} {} {}\n",
                        metric, point.value, point.timestamp
                    ));
                }
                Ok(prom)
            }
            ExportFormat::InfluxDB => {
                let mut influx = String::new();
                for point in points {
                    influx.push_str(&format!(
                        "{} value={} {}\n",
                        metric,
                        point.value,
                        point.timestamp * 1_000_000_000 // Convert to nanoseconds
                    ));
                }
                Ok(influx)
            }
        }
    }

    fn list_metrics(&self) -> Result<Vec<String>> {
        let data = self.data.read().unwrap();
        Ok(data.keys().cloned().collect())
    }

    fn delete_metric(&mut self, metric: &str) -> Result<()> {
        let mut data = self.data.write().unwrap();
        data.remove(metric);
        Ok(())
    }
}

/// File-based storage backend
pub struct FileBackend {
    base_path: PathBuf,
    data_cache: Arc<RwLock<HashMap<String, Vec<TimeSeriesPoint>>>>,
}

impl FileBackend {
    /// Create a new file backend
    pub fn new(base_path: impl AsRef<Path>) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        fs::create_dir_all(&base_path).map_err(|e| {
            AnalyticsError::IoError(format!("Failed to create directory: {}", e))
        })?;

        Ok(Self {
            base_path,
            data_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    fn metric_path(&self, metric: &str) -> PathBuf {
        self.base_path.join(format!("{}.tsv", metric))
    }

    fn load_metric(&self, metric: &str) -> Result<Vec<TimeSeriesPoint>> {
        let path = self.metric_path(metric);
        if !path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&path).map_err(|e| {
            AnalyticsError::IoError(format!("Failed to open file: {}", e))
        })?;

        let reader = BufReader::new(file);
        let mut points = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| {
                AnalyticsError::IoError(format!("Failed to read line: {}", e))
            })?;

            if let Some(point) = TimeSeriesPoint::from_string(&line) {
                points.push(point);
            }
        }

        Ok(points)
    }

    fn save_metric(&self, metric: &str, points: &[TimeSeriesPoint]) -> Result<()> {
        let path = self.metric_path(metric);
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .map_err(|e| AnalyticsError::IoError(format!("Failed to open file: {}", e)))?;

        for point in points {
            writeln!(file, "{}", point.to_string()).map_err(|e| {
                AnalyticsError::IoError(format!("Failed to write: {}", e))
            })?;
        }

        Ok(())
    }
}

impl StorageBackend for FileBackend {
    fn write(&mut self, metric: &str, point: TimeSeriesPoint) -> Result<()> {
        let mut cache = self.data_cache.write().unwrap();
        let points = cache
            .entry(metric.to_string())
            .or_insert_with(|| self.load_metric(metric).unwrap_or_default());

        points.push(point);

        // Persist to disk
        self.save_metric(metric, points)?;

        Ok(())
    }

    fn write_batch(&mut self, metric: &str, points: &[TimeSeriesPoint]) -> Result<()> {
        let mut cache = self.data_cache.write().unwrap();
        let cached_points = cache
            .entry(metric.to_string())
            .or_insert_with(|| self.load_metric(metric).unwrap_or_default());

        cached_points.extend_from_slice(points);

        // Persist to disk
        self.save_metric(metric, cached_points)?;

        Ok(())
    }

    fn read(&self, metric: &str, start: u64, end: u64) -> Result<Vec<TimeSeriesPoint>> {
        let cache = self.data_cache.read().unwrap();
        let points = if let Some(cached) = cache.get(metric) {
            cached.clone()
        } else {
            drop(cache);
            self.load_metric(metric)?
        };

        Ok(points
            .into_iter()
            .filter(|p| p.timestamp >= start && p.timestamp <= end)
            .collect())
    }

    fn apply_retention(&mut self, policy: &RetentionPolicy) -> Result<usize> {
        let mut cache = self.data_cache.write().unwrap();
        let mut total_removed = 0;

        for (metric, points) in cache.iter_mut() {
            let original_len = points.len();
            points.retain(|p| policy.should_retain(p.timestamp));
            let removed = original_len - points.len();
            total_removed += removed;

            if removed > 0 {
                self.save_metric(metric, points)?;
            }
        }

        Ok(total_removed)
    }

    fn export(&self, metric: &str, format: ExportFormat) -> Result<String> {
        let points = self.load_metric(metric)?;

        match format {
            ExportFormat::Csv => {
                let mut csv = String::from("timestamp,value\n");
                for point in points {
                    csv.push_str(&format!("{},{}\n", point.timestamp, point.value));
                }
                Ok(csv)
            }
            ExportFormat::Json => {
                let json_points: Vec<String> = points
                    .iter()
                    .map(|p| format!("{{\"timestamp\":{},\"value\":{}}}", p.timestamp, p.value))
                    .collect();
                Ok(format!("[{}]", json_points.join(",")))
            }
            ExportFormat::Prometheus => {
                let mut prom = String::new();
                for point in points {
                    prom.push_str(&format!(
                        "{} {} {}\n",
                        metric, point.value, point.timestamp
                    ));
                }
                Ok(prom)
            }
            ExportFormat::InfluxDB => {
                let mut influx = String::new();
                for point in points {
                    influx.push_str(&format!(
                        "{} value={} {}\n",
                        metric,
                        point.value,
                        point.timestamp * 1_000_000_000
                    ));
                }
                Ok(influx)
            }
        }
    }

    fn list_metrics(&self) -> Result<Vec<String>> {
        let entries = fs::read_dir(&self.base_path).map_err(|e| {
            AnalyticsError::IoError(format!("Failed to read directory: {}", e))
        })?;

        let mut metrics = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| {
                AnalyticsError::IoError(format!("Failed to read entry: {}", e))
            })?;

            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".tsv") {
                    metrics.push(name.trim_end_matches(".tsv").to_string());
                }
            }
        }

        Ok(metrics)
    }

    fn delete_metric(&mut self, metric: &str) -> Result<()> {
        let path = self.metric_path(metric);
        if path.exists() {
            fs::remove_file(&path).map_err(|e| {
                AnalyticsError::IoError(format!("Failed to delete file: {}", e))
            })?;
        }

        let mut cache = self.data_cache.write().unwrap();
        cache.remove(metric);

        Ok(())
    }
}

/// Metric storage with retention policies
pub struct MetricStorage {
    backend: Box<dyn StorageBackend>,
    policies: Vec<RetentionPolicy>,
}

impl MetricStorage {
    /// Create a new metric storage with memory backend
    pub fn new_memory() -> Self {
        Self {
            backend: Box::new(MemoryBackend::new()),
            policies: RetentionPolicy::defaults(),
        }
    }

    /// Create a new metric storage with file backend
    pub fn new_file(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            backend: Box::new(FileBackend::new(path)?),
            policies: RetentionPolicy::defaults(),
        })
    }

    /// Create with custom backend
    pub fn with_backend(backend: Box<dyn StorageBackend>) -> Self {
        Self {
            backend,
            policies: RetentionPolicy::defaults(),
        }
    }

    /// Set retention policies
    pub fn set_policies(&mut self, policies: Vec<RetentionPolicy>) {
        self.policies = policies;
    }

    /// Write a metric value
    pub fn write(&mut self, metric: &str, value: f64) -> Result<()> {
        self.backend.write(metric, TimeSeriesPoint::now(value))
    }

    /// Write a timestamped metric value
    pub fn write_timestamped(&mut self, metric: &str, timestamp: u64, value: f64) -> Result<()> {
        self.backend
            .write(metric, TimeSeriesPoint::new(timestamp, value))
    }

    /// Write multiple values
    pub fn write_batch(&mut self, metric: &str, points: &[TimeSeriesPoint]) -> Result<()> {
        self.backend.write_batch(metric, points)
    }

    /// Read metric values in time range
    pub fn read(&self, metric: &str, start: u64, end: u64) -> Result<Vec<TimeSeriesPoint>> {
        self.backend.read(metric, start, end)
    }

    /// Apply retention policies
    pub fn apply_retention(&mut self) -> Result<usize> {
        let mut total_removed = 0;
        for policy in &self.policies {
            total_removed += self.backend.apply_retention(policy)?;
        }
        Ok(total_removed)
    }

    /// Export metric data
    pub fn export(&self, metric: &str, format: ExportFormat) -> Result<String> {
        self.backend.export(metric, format)
    }

    /// List all metrics
    pub fn list_metrics(&self) -> Result<Vec<String>> {
        self.backend.list_metrics()
    }

    /// Delete a metric
    pub fn delete_metric(&mut self, metric: &str) -> Result<()> {
        self.backend.delete_metric(metric)
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_series_point() {
        let point = TimeSeriesPoint::new(1000, 42.5);
        assert_eq!(point.timestamp, 1000);
        assert_eq!(point.value, 42.5);

        let serialized = point.to_string();
        let deserialized = TimeSeriesPoint::from_string(&serialized).unwrap();
        assert_eq!(deserialized.timestamp, point.timestamp);
        assert_eq!(deserialized.value, point.value);
    }

    #[test]
    fn test_retention_policy() {
        let policy = RetentionPolicy::new("test", 3600, 60);
        let now = current_timestamp();

        assert!(policy.should_retain(now));
        assert!(policy.should_retain(now - 1800));
        assert!(!policy.should_retain(now - 7200));
    }

    #[test]
    fn test_memory_backend() {
        let mut backend = MemoryBackend::new();

        backend
            .write("cpu", TimeSeriesPoint::new(1000, 50.0))
            .unwrap();
        backend
            .write("cpu", TimeSeriesPoint::new(2000, 60.0))
            .unwrap();

        let points = backend.read("cpu", 0, 3000).unwrap();
        assert_eq!(points.len(), 2);
        assert_eq!(points[0].value, 50.0);
        assert_eq!(points[1].value, 60.0);
    }

    #[test]
    fn test_export_csv() {
        let mut backend = MemoryBackend::new();
        backend
            .write("test", TimeSeriesPoint::new(1000, 42.0))
            .unwrap();

        let csv = backend.export("test", ExportFormat::Csv).unwrap();
        assert!(csv.contains("timestamp,value"));
        assert!(csv.contains("1000,42"));
    }

    #[test]
    fn test_export_json() {
        let mut backend = MemoryBackend::new();
        backend
            .write("test", TimeSeriesPoint::new(1000, 42.0))
            .unwrap();

        let json = backend.export("test", ExportFormat::Json).unwrap();
        assert!(json.contains("\"timestamp\":1000"));
        assert!(json.contains("\"value\":42"));
    }

    #[test]
    fn test_metric_storage() {
        let mut storage = MetricStorage::new_memory();

        storage.write("cpu", 50.0).unwrap();
        storage.write("cpu", 60.0).unwrap();

        let metrics = storage.list_metrics().unwrap();
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0], "cpu");
    }
}
