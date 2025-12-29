//! # Compression Module
//!
//! Enterprise-grade compression algorithms optimized for CAD file formats.
//!
//! ## Features
//! - Custom LZ4 variant optimized for CAD geometry data
//! - Delta encoding for versioned CAD data
//! - Draco-inspired mesh compression algorithms
//! - Streaming compression for large files
//! - Domain-specific dictionary compression
//! - Multi-threaded parallel compression pipeline
//! - Adaptive algorithm selection based on data patterns
//!
//! ## Target Performance
//! - Compression ratio: 70-90% for typical CAD files
//! - Decompression speed: < 1 second for most files
//! - Multi-threaded scaling: Near-linear up to 8 cores

use serde::{Deserialize, Serialize};
use thiserror::Error;

// Module declarations
pub mod lz4_custom;
pub mod delta_encoding;
pub mod mesh_compression;
pub mod streaming;
pub mod dictionary;
pub mod parallel;
pub mod adaptive;

// Re-exports for convenience
pub use lz4_custom::{Lz4CustomCompressor, Lz4CustomConfig};
pub use delta_encoding::{DeltaEncoder, DeltaConfig};
pub use mesh_compression::{MeshCompressor, MeshCompressionConfig};
pub use streaming::{StreamingCompressor, StreamConfig};
pub use dictionary::{DictionaryCompressor, CADDictionary};
pub use parallel::{ParallelCompressor, ParallelConfig};
pub use adaptive::{AdaptiveCompressor, CompressionStrategy};

/// Compression error types
#[derive(Debug, Error)]
pub enum CompressionError {
    /// I/O error during compression/decompression
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid input data
    #[error("Invalid input data: {0}")]
    InvalidInput(String),

    /// Compression format error
    #[error("Compression format error: {0}")]
    FormatError(String),

    /// Buffer size error
    #[error("Buffer size error: {0}")]
    BufferError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Algorithm-specific error
    #[error("Algorithm error: {0}")]
    AlgorithmError(String),
}

/// Result type for compression operations
pub type Result<T> = std::result::Result<T, CompressionError>;

/// Compression level from 1 (fastest) to 9 (best compression)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionLevel {
    /// Fastest compression (level 1)
    Fastest,
    /// Fast compression (level 3)
    Fast,
    /// Balanced compression (level 5)
    Balanced,
    /// Best compression (level 7)
    Best,
    /// Maximum compression (level 9)
    Maximum,
}

impl CompressionLevel {
    /// Convert to numeric level (1-9)
    pub fn to_level(&self) -> u8 {
        match self {
            CompressionLevel::Fastest => 1,
            CompressionLevel::Fast => 3,
            CompressionLevel::Balanced => 5,
            CompressionLevel::Best => 7,
            CompressionLevel::Maximum => 9,
        }
    }
}

/// Compression statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionStats {
    /// Original uncompressed size in bytes
    pub original_size: u64,
    /// Compressed size in bytes
    pub compressed_size: u64,
    /// Compression ratio (0.0 to 1.0)
    pub ratio: f64,
    /// Compression time in milliseconds
    pub compression_time_ms: u64,
    /// Decompression time in milliseconds (if available)
    pub decompression_time_ms: Option<u64>,
    /// Algorithm used
    pub algorithm: String,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl CompressionStats {
    /// Calculate compression percentage (e.g., 75% compression)
    pub fn compression_percentage(&self) -> f64 {
        (1.0 - self.ratio) * 100.0
    }

    /// Calculate compression throughput in MB/s
    pub fn throughput_mbps(&self) -> f64 {
        if self.compression_time_ms == 0 {
            return 0.0;
        }
        (self.original_size as f64 / 1_000_000.0) / (self.compression_time_ms as f64 / 1000.0)
    }
}

/// Main compression trait
pub trait Compressor: Send + Sync {
    /// Compress data
    fn compress(&self, input: &[u8]) -> Result<Vec<u8>>;

    /// Decompress data
    fn decompress(&self, input: &[u8]) -> Result<Vec<u8>>;

    /// Get compression statistics for the last operation
    fn stats(&self) -> Option<CompressionStats>;

    /// Get algorithm name
    fn algorithm_name(&self) -> &str;
}

/// Compression format identifier (magic bytes)
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionFormat {
    /// Custom LZ4 variant
    Lz4Custom = 0x4C5A3443, // "LZ4C"
    /// Delta encoding
    Delta = 0x44454C54, // "DELT"
    /// Mesh compression
    Mesh = 0x4D455348, // "MESH"
    /// Dictionary compression
    Dictionary = 0x44494354, // "DICT"
    /// Adaptive multi-algorithm
    Adaptive = 0x41445054, // "ADPT"
}

impl CompressionFormat {
    /// Get magic bytes for the format
    pub fn magic_bytes(&self) -> [u8; 4] {
        (*self as u32).to_le_bytes()
    }

    /// Parse format from magic bytes
    pub fn from_magic_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 4 {
            return None;
        }
        let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        match magic {
            0x4C5A3443 => Some(CompressionFormat::Lz4Custom),
            0x44454C54 => Some(CompressionFormat::Delta),
            0x4D455348 => Some(CompressionFormat::Mesh),
            0x44494354 => Some(CompressionFormat::Dictionary),
            0x41445054 => Some(CompressionFormat::Adaptive),
            _ => None,
        }
    }
}

/// Compressed data container with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedData {
    /// Format identifier
    pub format: u32,
    /// Version number
    pub version: u16,
    /// Flags (reserved for future use)
    pub flags: u16,
    /// Original uncompressed size
    pub original_size: u64,
    /// Compressed payload
    pub data: Vec<u8>,
    /// Optional checksum (CRC32 or similar)
    pub checksum: Option<u32>,
}

impl CompressedData {
    /// Create new compressed data container
    pub fn new(format: CompressionFormat, data: Vec<u8>, original_size: u64) -> Self {
        Self {
            format: format as u32,
            version: 1,
            flags: 0,
            original_size,
            data,
            checksum: None,
        }
    }

    /// Serialize to bytes with header
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.data.len() + 24);
        result.extend_from_slice(&self.format.to_le_bytes());
        result.extend_from_slice(&self.version.to_le_bytes());
        result.extend_from_slice(&self.flags.to_le_bytes());
        result.extend_from_slice(&self.original_size.to_le_bytes());
        result.extend_from_slice(&(self.data.len() as u64).to_le_bytes());
        if let Some(checksum) = self.checksum {
            result.extend_from_slice(&checksum.to_le_bytes());
        }
        result.extend_from_slice(&self.data);
        result
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 24 {
            return Err(CompressionError::FormatError(
                "Insufficient header data".to_string()
            ));
        }

        let format = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let version = u16::from_le_bytes([bytes[4], bytes[5]]);
        let flags = u16::from_le_bytes([bytes[6], bytes[7]]);
        let original_size = u64::from_le_bytes([
            bytes[8], bytes[9], bytes[10], bytes[11],
            bytes[12], bytes[13], bytes[14], bytes[15],
        ]);
        let data_len = u64::from_le_bytes([
            bytes[16], bytes[17], bytes[18], bytes[19],
            bytes[20], bytes[21], bytes[22], bytes[23],
        ]) as usize;

        let data_start = 24;
        if bytes.len() < data_start + data_len {
            return Err(CompressionError::FormatError(
                "Insufficient data payload".to_string()
            ));
        }

        let data = bytes[data_start..data_start + data_len].to_vec();

        Ok(Self {
            format,
            version,
            flags,
            original_size,
            data,
            checksum: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_format_magic_bytes() {
        let format = CompressionFormat::Lz4Custom;
        let magic = format.magic_bytes();
        assert_eq!(CompressionFormat::from_magic_bytes(&magic), Some(format));
    }

    #[test]
    fn test_compressed_data_serialization() {
        let data = CompressedData::new(
            CompressionFormat::Lz4Custom,
            vec![1, 2, 3, 4, 5],
            100,
        );
        let bytes = data.to_bytes();
        let recovered = CompressedData::from_bytes(&bytes).unwrap();
        assert_eq!(recovered.format, data.format);
        assert_eq!(recovered.original_size, data.original_size);
        assert_eq!(recovered.data, data.data);
    }

    #[test]
    fn test_compression_stats() {
        let stats = CompressionStats {
            original_size: 1000,
            compressed_size: 250,
            ratio: 0.25,
            compression_time_ms: 10,
            decompression_time_ms: Some(5),
            algorithm: "test".to_string(),
            metadata: std::collections::HashMap::new(),
        };

        assert_eq!(stats.compression_percentage(), 75.0);
        assert_eq!(stats.throughput_mbps(), 0.1);
    }
}
