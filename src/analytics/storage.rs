//! # Efficient Metrics Storage
//!
//! Time-series storage with compression and efficient querying.

use super::{Result, AnalyticsError, AggregatedMetric};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc, Duration, Datelike};
use std::collections::HashMap;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::fs;

/// Time-series data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Value
    pub value: f64,

    /// Labels
    pub labels: HashMap<String, String>,

    /// Metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Storage configuration
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Storage directory path
    pub path: String,

    /// Retention period in days
    pub retention_days: u32,

    /// Maximum storage size in bytes
    pub max_size_bytes: u64,

    /// Enable compression
    pub enable_compression: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            path: "./analytics_data".to_string(),
            retention_days: 30,
            max_size_bytes: 10 * 1024 * 1024 * 1024, // 10 GB
            enable_compression: true,
        }
    }
}

/// Partition key for organizing time-series data
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PartitionKey {
    metric_name: String,
    year: i32,
    month: u32,
    day: u32,
}

impl PartitionKey {
    fn from_timestamp(metric_name: &str, timestamp: DateTime<Utc>) -> Self {
        Self {
            metric_name: metric_name.to_string(),
            year: timestamp.year(),
            month: timestamp.month(),
            day: timestamp.day(),
        }
    }

    fn to_path(&self, base_path: &Path) -> PathBuf {
        base_path
            .join(&self.metric_name)
            .join(format!("{:04}", self.year))
            .join(format!("{:02}", self.month))
            .join(format!("{:02}.dat", self.day))
    }
}

/// Data partition for a specific time range
#[derive(Debug, Serialize, Deserialize)]
struct DataPartition {
    /// Partition metadata
    metric_name: String,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,

    /// Data points
    points: Vec<TimeSeriesPoint>,

    /// Compression applied
    compressed: bool,

    /// Size in bytes
    size_bytes: u64,
}

impl DataPartition {
    fn new(metric_name: String) -> Self {
        Self {
            metric_name,
            start_time: Utc::now(),
            end_time: Utc::now(),
            points: Vec::new(),
            compressed: false,
            size_bytes: 0,
        }
    }

    fn add_point(&mut self, point: TimeSeriesPoint) {
        if self.points.is_empty() {
            self.start_time = point.timestamp;
        }
        self.end_time = point.timestamp;
        self.points.push(point);
    }

    fn compress(&mut self) -> Result<Vec<u8>> {
        let json = serde_json::to_vec(&self.points)?;

        // Use LZ4 compression
        let compressed = lz4::block::compress(&json, Some(lz4::block::CompressionMode::HIGHCOMPRESSION(9)), false)
            .map_err(|e| AnalyticsError::Storage(format!("Compression failed: {}", e)))?;

        self.compressed = true;
        self.size_bytes = compressed.len() as u64;

        Ok(compressed)
    }

    fn decompress(data: &[u8]) -> Result<Vec<TimeSeriesPoint>> {
        let decompressed = lz4::block::decompress(data, None)
            .map_err(|e| AnalyticsError::Storage(format!("Decompression failed: {}", e)))?;

        let points: Vec<TimeSeriesPoint> = serde_json::from_slice(&decompressed)?;
        Ok(points)
    }
}

/// Metrics storage engine
pub struct MetricsStorage {
    config: StorageConfig,
    base_path: PathBuf,

    /// In-memory cache of recent data
    cache: Arc<RwLock<HashMap<PartitionKey, DataPartition>>>,

    /// Write buffer
    write_buffer: Arc<RwLock<HashMap<String, Vec<TimeSeriesPoint>>>>,

    /// Storage statistics
    stats: Arc<RwLock<StorageStats>>,
}

#[derive(Debug, Default)]
struct StorageStats {
    total_points: u64,
    total_partitions: u64,
    total_bytes: u64,
    cache_hits: u64,
    cache_misses: u64,
}

impl MetricsStorage {
    /// Create a new metrics storage
    pub fn new(config: StorageConfig) -> Result<Self> {
        let base_path = PathBuf::from(&config.path);

        Ok(Self {
            config,
            base_path,
            cache: Arc::new(RwLock::new(HashMap::new())),
            write_buffer: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(StorageStats::default())),
        })
    }

    /// Initialize storage (create directories)
    pub async fn initialize(&self) -> Result<()> {
        fs::create_dir_all(&self.base_path).await?;
        Ok(())
    }

    /// Write a time-series point
    pub async fn write(
        &self,
        metric_name: &str,
        timestamp: DateTime<Utc>,
        value: f64,
        labels: HashMap<String, String>,
    ) -> Result<()> {
        let point = TimeSeriesPoint {
            timestamp,
            value,
            labels,
            metadata: None,
        };

        // Add to write buffer
        let mut buffer = self.write_buffer.write();
        buffer
            .entry(metric_name.to_string())
            .or_insert_with(Vec::new)
            .push(point);

        // Flush if buffer is large
        if buffer.get(metric_name).map(|v| v.len()).unwrap_or(0) >= 1000 {
            drop(buffer);
            self.flush_metric(metric_name).await?;
        }

        Ok(())
    }

    /// Write aggregated metrics
    pub async fn write_aggregated(&self, metrics: Vec<AggregatedMetric>) -> Result<()> {
        for metric in metrics {
            self.write(
                &metric.name,
                metric.timestamp,
                metric.stats.avg,
                metric.labels,
            )
            .await?;
        }
        Ok(())
    }

    /// Query time-series data
    pub async fn query(
        &self,
        metric_name: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<TimeSeriesPoint>> {
        let mut result = Vec::new();

        // First, check write buffer
        let buffer = self.write_buffer.read();
        if let Some(points) = buffer.get(metric_name) {
            result.extend(
                points
                    .iter()
                    .filter(|p| p.timestamp >= start_time && p.timestamp <= end_time)
                    .cloned(),
            );
        }
        drop(buffer);

        // Then, query partitions for the date range
        let mut current = start_time.date_naive();
        let end = end_time.date_naive();

        while current <= end {
            let partition_key = PartitionKey {
                metric_name: metric_name.to_string(),
                year: current.year(),
                month: current.month(),
                day: current.day(),
            };

            // Check cache first
            let cache = self.cache.read();
            if let Some(partition) = cache.get(&partition_key) {
                result.extend(
                    partition
                        .points
                        .iter()
                        .filter(|p| p.timestamp >= start_time && p.timestamp <= end_time)
                        .cloned(),
                );
                self.stats.write().cache_hits += 1;
            } else {
                drop(cache);
                self.stats.write().cache_misses += 1;

                // Load from disk
                if let Ok(partition) = self.load_partition(&partition_key).await {
                    result.extend(
                        partition
                            .points
                            .iter()
                            .filter(|p| p.timestamp >= start_time && p.timestamp <= end_time)
                            .cloned(),
                    );

                    // Cache the loaded partition
                    self.cache.write().insert(partition_key.clone(), partition);
                }
            }

            current = current.succ_opt().unwrap_or(current);
        }

        // Sort by timestamp
        result.sort_by_key(|p| p.timestamp);

        Ok(result)
    }

    /// Flush write buffer to disk
    pub async fn flush(&self) -> Result<()> {
        let buffer = self.write_buffer.write();
        let metric_names: Vec<String> = buffer.keys().cloned().collect();
        drop(buffer);

        for metric_name in metric_names {
            self.flush_metric(&metric_name).await?;
        }

        Ok(())
    }

    /// Flush a specific metric
    async fn flush_metric(&self, metric_name: &str) -> Result<()> {
        let mut buffer = self.write_buffer.write();
        let points = buffer.remove(metric_name).unwrap_or_default();
        drop(buffer);

        if points.is_empty() {
            return Ok(());
        }

        // Group points by partition
        let mut partitions: HashMap<PartitionKey, Vec<TimeSeriesPoint>> = HashMap::new();

        for point in points {
            let key = PartitionKey::from_timestamp(metric_name, point.timestamp);
            partitions.entry(key).or_insert_with(Vec::new).push(point);
        }

        // Write each partition
        for (key, points) in partitions {
            self.write_partition(&key, points).await?;
        }

        Ok(())
    }

    /// Write a partition to disk
    async fn write_partition(&self, key: &PartitionKey, mut new_points: Vec<TimeSeriesPoint>) -> Result<()> {
        let path = key.to_path(&self.base_path);

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Load existing partition if it exists
        let mut partition = if path.exists() {
            self.load_partition(key).await?
        } else {
            DataPartition::new(key.metric_name.clone())
        };

        // Add new points
        partition.points.append(&mut new_points);
        partition.points.sort_by_key(|p| p.timestamp);

        // Compress and write
        let compressed = partition.compress()?;
        fs::write(&path, &compressed).await?;

        // Update cache
        self.cache.write().insert(key.clone(), partition);

        // Update stats
        let mut stats = self.stats.write();
        stats.total_partitions += 1;
        stats.total_bytes += compressed.len() as u64;

        Ok(())
    }

    /// Load a partition from disk
    async fn load_partition(&self, key: &PartitionKey) -> Result<DataPartition> {
        let path = key.to_path(&self.base_path);

        if !path.exists() {
            return Ok(DataPartition::new(key.metric_name.clone()));
        }

        let compressed = fs::read(&path).await?;
        let points = DataPartition::decompress(&compressed)?;

        let mut partition = DataPartition::new(key.metric_name.clone());
        if !points.is_empty() {
            partition.start_time = points[0].timestamp;
            partition.end_time = points[points.len() - 1].timestamp;
            partition.points = points;
            partition.compressed = true;
            partition.size_bytes = compressed.len() as u64;
        }

        Ok(partition)
    }

    /// Clean up old data based on retention policy
    pub async fn cleanup(&self) -> Result<()> {
        let cutoff = Utc::now() - Duration::days(self.config.retention_days as i64);

        // Remove old partitions
        // Implementation would walk the directory tree and delete old files

        Ok(())
    }

    /// Get storage size in bytes
    pub async fn size_bytes(&self) -> Result<u64> {
        Ok(self.stats.read().total_bytes)
    }

    /// Get storage statistics
    pub fn statistics(&self) -> StorageStatistics {
        let stats = self.stats.read();
        StorageStatistics {
            total_points: stats.total_points,
            total_partitions: stats.total_partitions,
            total_bytes: stats.total_bytes,
            cache_hits: stats.cache_hits,
            cache_misses: stats.cache_misses,
            cache_hit_rate: if stats.cache_hits + stats.cache_misses > 0 {
                stats.cache_hits as f64 / (stats.cache_hits + stats.cache_misses) as f64
            } else {
                0.0
            },
        }
    }

    /// Compact storage by merging small partitions
    pub async fn compact(&self) -> Result<()> {
        // Implementation would merge small partitions to reduce file count
        Ok(())
    }
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStatistics {
    pub total_points: u64,
    pub total_partitions: u64,
    pub total_bytes: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partition_key() {
        let timestamp = DateTime::from_timestamp(1234567890, 0).unwrap();
        let key = PartitionKey::from_timestamp("test_metric", timestamp);

        assert_eq!(key.metric_name, "test_metric");
        assert_eq!(key.year, 2009);
    }

    #[tokio::test]
    async fn test_storage_creation() {
        let config = StorageConfig::default();
        let storage = MetricsStorage::new(config);
        assert!(storage.is_ok());
    }

    #[test]
    fn test_data_partition() {
        let mut partition = DataPartition::new("test".to_string());
        assert_eq!(partition.points.len(), 0);

        let point = TimeSeriesPoint {
            timestamp: Utc::now(),
            value: 42.0,
            labels: HashMap::new(),
            metadata: None,
        };

        partition.add_point(point);
        assert_eq!(partition.points.len(), 1);
    }
}
