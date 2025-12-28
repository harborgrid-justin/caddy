//! Serialization and compression codecs for cache values
//!
//! This module provides:
//! - Binary serialization using bincode
//! - Compression algorithms (simulated LZ4, ZSTD)
//! - Schema versioning for backward compatibility
//! - Efficient encoding/decoding with minimal overhead

use std::fmt::Debug;
use std::marker::PhantomData;
use serde::{Deserialize, Serialize};
use crate::enterprise::error::{EnterpriseError, EnterpriseResult};

/// Compression algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    /// No compression
    None,
    /// LZ4 compression (fast, moderate compression)
    Lz4,
    /// ZSTD compression (slower, better compression)
    Zstd,
}

/// Codec configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodecConfig {
    /// Compression algorithm to use
    pub compression: CompressionAlgorithm,
    /// Compression level (1-22 for ZSTD, 1-12 for LZ4)
    pub compression_level: i32,
    /// Schema version for compatibility
    pub schema_version: u32,
    /// Enable checksum validation
    pub enable_checksum: bool,
}

impl Default for CodecConfig {
    fn default() -> Self {
        Self {
            compression: CompressionAlgorithm::None,
            compression_level: 3,
            schema_version: 1,
            enable_checksum: true,
        }
    }
}

/// Encoded data with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodedData {
    /// Schema version
    pub version: u32,
    /// Compression algorithm used
    pub compression: CompressionAlgorithm,
    /// Original data size (before compression)
    pub original_size: usize,
    /// Compressed data size
    pub compressed_size: usize,
    /// Optional checksum
    pub checksum: Option<u64>,
    /// The actual data
    pub data: Vec<u8>,
}

impl EncodedData {
    /// Get compression ratio
    pub fn compression_ratio(&self) -> f64 {
        if self.original_size == 0 {
            return 1.0;
        }
        self.compressed_size as f64 / self.original_size as f64
    }

    /// Verify checksum if enabled
    pub fn verify_checksum(&self) -> bool {
        if let Some(expected) = self.checksum {
            let computed = Self::compute_checksum(&self.data);
            computed == expected
        } else {
            true // No checksum to verify
        }
    }

    /// Compute simple checksum (in production, use CRC32 or similar)
    fn compute_checksum(data: &[u8]) -> u64 {
        data.iter().fold(0u64, |acc, &b| acc.wrapping_add(b as u64))
    }
}

/// Binary codec using bincode serialization
pub struct BincodeCodec<T> {
    config: CodecConfig,
    _phantom: PhantomData<T>,
}

impl<T> BincodeCodec<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    pub fn new() -> Self {
        Self::with_config(CodecConfig::default())
    }

    pub fn with_config(config: CodecConfig) -> Self {
        Self {
            config,
            _phantom: PhantomData,
        }
    }

    /// Encode a value to bytes
    pub fn encode(&self, value: &T) -> EnterpriseResult<EncodedData> {
        // Serialize to bytes
        let serialized = bincode::serialize(value)
            .map_err(|e| EnterpriseError::Other(format!("Serialization error: {}", e)))?;

        let original_size = serialized.len();

        // Apply compression
        let (compressed, compression) = match self.config.compression {
            CompressionAlgorithm::None => (serialized, CompressionAlgorithm::None),
            CompressionAlgorithm::Lz4 => {
                let compressed = self.compress_lz4(&serialized)?;
                (compressed, CompressionAlgorithm::Lz4)
            }
            CompressionAlgorithm::Zstd => {
                let compressed = self.compress_zstd(&serialized)?;
                (compressed, CompressionAlgorithm::Zstd)
            }
        };

        let compressed_size = compressed.len();

        // Compute checksum if enabled
        let checksum = if self.config.enable_checksum {
            Some(EncodedData::compute_checksum(&compressed))
        } else {
            None
        };

        Ok(EncodedData {
            version: self.config.schema_version,
            compression,
            original_size,
            compressed_size,
            checksum,
            data: compressed,
        })
    }

    /// Decode bytes to a value
    pub fn decode(&self, encoded: &EncodedData) -> EnterpriseResult<T> {
        // Verify checksum
        if self.config.enable_checksum && !encoded.verify_checksum() {
            return Err(EnterpriseError::Other("Checksum verification failed".to_string()));
        }

        // Decompress
        let decompressed = match encoded.compression {
            CompressionAlgorithm::None => encoded.data.clone(),
            CompressionAlgorithm::Lz4 => self.decompress_lz4(&encoded.data)?,
            CompressionAlgorithm::Zstd => self.decompress_zstd(&encoded.data)?,
        };

        // Deserialize
        let value = bincode::deserialize(&decompressed)
            .map_err(|e| EnterpriseError::Other(format!("Deserialization error: {}", e)))?;

        Ok(value)
    }

    /// Simulate LZ4 compression (in production, use lz4_flex or similar)
    fn compress_lz4(&self, data: &[u8]) -> EnterpriseResult<Vec<u8>> {
        // Simulated compression - just return the data
        // In production: lz4_flex::compress(data)
        Ok(data.to_vec())
    }

    /// Simulate LZ4 decompression
    fn decompress_lz4(&self, data: &[u8]) -> EnterpriseResult<Vec<u8>> {
        // Simulated decompression - just return the data
        // In production: lz4_flex::decompress(data, original_size)
        Ok(data.to_vec())
    }

    /// Simulate ZSTD compression (in production, use zstd crate)
    fn compress_zstd(&self, data: &[u8]) -> EnterpriseResult<Vec<u8>> {
        // Simulated compression - just return the data
        // In production: zstd::encode_all(data, level)
        Ok(data.to_vec())
    }

    /// Simulate ZSTD decompression
    fn decompress_zstd(&self, data: &[u8]) -> EnterpriseResult<Vec<u8>> {
        // Simulated decompression - just return the data
        // In production: zstd::decode_all(data)
        Ok(data.to_vec())
    }

    /// Get codec configuration
    pub fn config(&self) -> &CodecConfig {
        &self.config
    }
}

impl<T> Default for BincodeCodec<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Schema-versioned codec for backward compatibility
pub struct VersionedCodec<T> {
    /// Codecs for different schema versions
    codecs: Vec<(u32, BincodeCodec<T>)>,
    /// Current/latest version
    current_version: u32,
}

impl<T> VersionedCodec<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    pub fn new(current_version: u32) -> Self {
        let mut config = CodecConfig::default();
        config.schema_version = current_version;

        let codec = BincodeCodec::with_config(config);

        Self {
            codecs: vec![(current_version, codec)],
            current_version,
        }
    }

    /// Register a codec for a specific schema version
    pub fn register_version(&mut self, version: u32, codec: BincodeCodec<T>) {
        self.codecs.push((version, codec));
        self.codecs.sort_by_key(|(v, _)| *v);
    }

    /// Encode using the current version
    pub fn encode(&self, value: &T) -> EnterpriseResult<EncodedData> {
        let codec = self.get_codec(self.current_version)?;
        codec.encode(value)
    }

    /// Decode, automatically selecting the right codec based on version
    pub fn decode(&self, encoded: &EncodedData) -> EnterpriseResult<T> {
        let codec = self.get_codec(encoded.version)?;
        codec.decode(encoded)
    }

    /// Get codec for a specific version
    fn get_codec(&self, version: u32) -> EnterpriseResult<&BincodeCodec<T>> {
        self.codecs
            .iter()
            .find(|(v, _)| *v == version)
            .map(|(_, codec)| codec)
            .ok_or_else(|| {
                EnterpriseError::Other(format!("No codec found for version {}", version))
            })
    }

    /// Get current schema version
    pub fn current_version(&self) -> u32 {
        self.current_version
    }

    /// Get all supported versions
    pub fn supported_versions(&self) -> Vec<u32> {
        self.codecs.iter().map(|(v, _)| *v).collect()
    }
}

/// Codec statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct CodecStats {
    /// Total encode operations
    pub encode_count: u64,
    /// Total decode operations
    pub decode_count: u64,
    /// Total bytes encoded
    pub bytes_encoded: u64,
    /// Total bytes decoded
    pub bytes_decoded: u64,
    /// Total compression time (microseconds)
    pub compression_time_us: u64,
    /// Total decompression time (microseconds)
    pub decompression_time_us: u64,
}

impl CodecStats {
    /// Get average compression ratio
    pub fn avg_compression_ratio(&self) -> f64 {
        if self.bytes_encoded == 0 {
            return 1.0;
        }
        self.bytes_decoded as f64 / self.bytes_encoded as f64
    }

    /// Get average encode time
    pub fn avg_encode_time_us(&self) -> f64 {
        if self.encode_count == 0 {
            return 0.0;
        }
        self.compression_time_us as f64 / self.encode_count as f64
    }

    /// Get average decode time
    pub fn avg_decode_time_us(&self) -> f64 {
        if self.decode_count == 0 {
            return 0.0;
        }
        self.decompression_time_us as f64 / self.decode_count as f64
    }
}

/// Codec with statistics tracking
pub struct TrackedCodec<T> {
    codec: BincodeCodec<T>,
    stats: CodecStats,
}

impl<T> TrackedCodec<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    pub fn new() -> Self {
        Self {
            codec: BincodeCodec::new(),
            stats: CodecStats::default(),
        }
    }

    pub fn with_config(config: CodecConfig) -> Self {
        Self {
            codec: BincodeCodec::with_config(config),
            stats: CodecStats::default(),
        }
    }

    /// Encode with statistics tracking
    pub fn encode(&mut self, value: &T) -> EnterpriseResult<EncodedData> {
        let start = std::time::Instant::now();
        let encoded = self.codec.encode(value)?;
        let elapsed = start.elapsed().as_micros() as u64;

        self.stats.encode_count += 1;
        self.stats.bytes_encoded += encoded.compressed_size as u64;
        self.stats.compression_time_us += elapsed;

        Ok(encoded)
    }

    /// Decode with statistics tracking
    pub fn decode(&mut self, encoded: &EncodedData) -> EnterpriseResult<T> {
        let start = std::time::Instant::now();
        let value = self.codec.decode(encoded)?;
        let elapsed = start.elapsed().as_micros() as u64;

        self.stats.decode_count += 1;
        self.stats.bytes_decoded += encoded.original_size as u64;
        self.stats.decompression_time_us += elapsed;

        Ok(value)
    }

    /// Get current statistics
    pub fn stats(&self) -> &CodecStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = CodecStats::default();
    }
}

impl<T> Default for TrackedCodec<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestData {
        id: u64,
        name: String,
        values: Vec<f64>,
    }

    #[test]
    fn test_bincode_codec_basic() {
        let codec = BincodeCodec::new();

        let data = TestData {
            id: 123,
            name: "test".to_string(),
            values: vec![1.0, 2.0, 3.0],
        };

        let encoded = codec.encode(&data).unwrap();
        let decoded: TestData = codec.decode(&encoded).unwrap();

        assert_eq!(data, decoded);
    }

    #[test]
    fn test_bincode_codec_with_compression() {
        let mut config = CodecConfig::default();
        config.compression = CompressionAlgorithm::Lz4;

        let codec = BincodeCodec::with_config(config);

        let data = TestData {
            id: 456,
            name: "compressed".to_string(),
            values: vec![4.0, 5.0, 6.0],
        };

        let encoded = codec.encode(&data).unwrap();
        assert_eq!(encoded.compression, CompressionAlgorithm::Lz4);

        let decoded: TestData = codec.decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_encoded_data_compression_ratio() {
        let encoded = EncodedData {
            version: 1,
            compression: CompressionAlgorithm::Lz4,
            original_size: 1000,
            compressed_size: 500,
            checksum: None,
            data: vec![],
        };

        assert_eq!(encoded.compression_ratio(), 0.5);
    }

    #[test]
    fn test_checksum_validation() {
        let mut config = CodecConfig::default();
        config.enable_checksum = true;

        let codec = BincodeCodec::<TestData>::with_config(config);

        let data = TestData {
            id: 789,
            name: "checksum".to_string(),
            values: vec![7.0, 8.0, 9.0],
        };

        let encoded = codec.encode(&data).unwrap();
        assert!(encoded.checksum.is_some());
        assert!(encoded.verify_checksum());

        // Decode should succeed with valid checksum
        let decoded: TestData = codec.decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_versioned_codec() {
        let mut codec = VersionedCodec::<TestData>::new(1);

        let data = TestData {
            id: 100,
            name: "versioned".to_string(),
            values: vec![10.0, 20.0],
        };

        let encoded = codec.encode(&data).unwrap();
        assert_eq!(encoded.version, 1);

        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_versioned_codec_multiple_versions() {
        let mut codec = VersionedCodec::<TestData>::new(2);

        // Register version 1 codec
        let v1_codec = BincodeCodec::new();
        codec.register_version(1, v1_codec);

        assert_eq!(codec.current_version(), 2);
        assert!(codec.supported_versions().contains(&1));
        assert!(codec.supported_versions().contains(&2));
    }

    #[test]
    fn test_tracked_codec() {
        let mut codec = TrackedCodec::new();

        let data = TestData {
            id: 200,
            name: "tracked".to_string(),
            values: vec![30.0, 40.0],
        };

        // Encode
        let encoded = codec.encode(&data).unwrap();
        assert_eq!(codec.stats().encode_count, 1);

        // Decode
        let _decoded: TestData = codec.decode(&encoded).unwrap();
        assert_eq!(codec.stats().decode_count, 1);

        // Check stats
        assert!(codec.stats().bytes_encoded > 0);
        assert!(codec.stats().bytes_decoded > 0);
    }

    #[test]
    fn test_codec_stats() {
        let mut stats = CodecStats::default();

        stats.encode_count = 10;
        stats.bytes_encoded = 1000;
        stats.compression_time_us = 5000;

        stats.decode_count = 10;
        stats.bytes_decoded = 2000;
        stats.decompression_time_us = 3000;

        assert_eq!(stats.avg_compression_ratio(), 2.0);
        assert_eq!(stats.avg_encode_time_us(), 500.0);
        assert_eq!(stats.avg_decode_time_us(), 300.0);
    }
}
