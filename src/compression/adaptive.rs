//! # Adaptive Compression
//!
//! Adaptive algorithm selection based on data patterns.
//! Automatically chooses the best compression algorithm for the input data.
//!
//! ## Features
//! - Automatic data type detection
//! - Algorithm selection based on data characteristics
//! - Sampling for fast decision making
//! - Heuristics for CAD-specific patterns
//! - Performance vs compression trade-offs

use super::{
    Compressor, CompressionError, CompressionLevel, CompressionStats, Result,
    Lz4CustomCompressor, DeltaEncoder, MeshCompressor, DictionaryCompressor,
};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Compression strategy selection mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionStrategy {
    /// Always use LZ4 custom (fast, good general-purpose)
    AlwaysLz4,
    /// Always use delta encoding (best for versioned data)
    AlwaysDelta,
    /// Always use mesh compression (best for 3D geometry)
    AlwaysMesh,
    /// Always use dictionary compression (best for text-heavy data)
    AlwaysDictionary,
    /// Automatically select best algorithm
    Auto,
    /// Fastest possible compression
    Fastest,
    /// Best compression ratio
    BestRatio,
}

/// Adaptive compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveConfig {
    /// Strategy selection mode
    pub strategy: CompressionStrategy,
    /// Compression level
    pub level: CompressionLevel,
    /// Sample size for analysis (bytes)
    pub sample_size: usize,
    /// Enable parallel compression when beneficial
    pub use_parallel: bool,
}

impl Default for AdaptiveConfig {
    fn default() -> Self {
        Self {
            strategy: CompressionStrategy::Auto,
            level: CompressionLevel::Balanced,
            sample_size: 4096, // 4KB sample
            use_parallel: true,
        }
    }
}

/// Data characteristics detected from sampling
#[derive(Debug, Clone)]
struct DataCharacteristics {
    /// Entropy (0.0 to 1.0, higher = more random)
    entropy: f64,
    /// Percentage of printable ASCII characters
    ascii_ratio: f64,
    /// Average byte value
    avg_byte_value: f64,
    /// Standard deviation of byte values
    std_dev: f64,
    /// Repetition score (higher = more repetitive)
    repetition_score: f64,
    /// Looks like float data
    is_float_data: bool,
    /// Looks like text data
    is_text_data: bool,
    /// Looks like structured binary data
    is_structured: bool,
}

/// Adaptive compressor
pub struct AdaptiveCompressor {
    config: AdaptiveConfig,
    last_stats: parking_lot::RwLock<Option<CompressionStats>>,
    last_strategy: parking_lot::RwLock<Option<String>>,
}

impl AdaptiveCompressor {
    /// Create new adaptive compressor with default configuration
    pub fn new() -> Self {
        Self::with_config(AdaptiveConfig::default())
    }

    /// Create new adaptive compressor with custom configuration
    pub fn with_config(config: AdaptiveConfig) -> Self {
        Self {
            config,
            last_stats: parking_lot::RwLock::new(None),
            last_strategy: parking_lot::RwLock::new(None),
        }
    }

    /// Get the last selected strategy
    pub fn last_strategy(&self) -> Option<String> {
        self.last_strategy.read().clone()
    }

    /// Compress data using adaptive algorithm selection
    fn compress_internal(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        let start = Instant::now();

        // Analyze data characteristics
        let characteristics = self.analyze_data(input);

        // Select best algorithm
        let selected_algorithm = self.select_algorithm(&characteristics);
        *self.last_strategy.write() = Some(selected_algorithm.clone());

        // Compress using selected algorithm
        let mut output = Vec::new();

        // Write algorithm marker
        output.push(self.algorithm_to_marker(&selected_algorithm));

        // Compress data
        let compressed = match selected_algorithm.as_str() {
            "lz4" => {
                let compressor = Lz4CustomCompressor::new();
                compressor.compress(input)?
            }
            "delta" => {
                let compressor = DeltaEncoder::new();
                compressor.compress(input)?
            }
            "dictionary" => {
                let compressor = DictionaryCompressor::new();
                compressor.compress(input)?
            }
            "parallel" => {
                let compressor = super::ParallelCompressor::new();
                compressor.compress(input)?
            }
            _ => {
                // Fallback to LZ4
                let compressor = Lz4CustomCompressor::new();
                compressor.compress(input)?
            }
        };

        output.extend_from_slice(&compressed);

        let elapsed = start.elapsed();

        let stats = CompressionStats {
            original_size: input.len() as u64,
            compressed_size: output.len() as u64,
            ratio: output.len() as f64 / input.len() as f64,
            compression_time_ms: elapsed.as_millis() as u64,
            decompression_time_ms: None,
            algorithm: format!("Adaptive({})", selected_algorithm),
            metadata: std::collections::HashMap::from([
                ("selected_algorithm".to_string(), selected_algorithm.clone()),
                ("entropy".to_string(), format!("{:.3}", characteristics.entropy)),
                ("ascii_ratio".to_string(), format!("{:.3}", characteristics.ascii_ratio)),
            ]),
        };

        *self.last_stats.write() = Some(stats);
        Ok(output)
    }

    /// Decompress data
    fn decompress_internal(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        let start = Instant::now();

        // Read algorithm marker
        let algorithm_marker = input[0];
        let algorithm = self.marker_to_algorithm(algorithm_marker)?;

        // Decompress data
        let decompressed = match algorithm.as_str() {
            "lz4" => {
                let compressor = Lz4CustomCompressor::new();
                compressor.decompress(&input[1..])?
            }
            "delta" => {
                let compressor = DeltaEncoder::new();
                compressor.decompress(&input[1..])?
            }
            "dictionary" => {
                let compressor = DictionaryCompressor::new();
                compressor.decompress(&input[1..])?
            }
            "parallel" => {
                let compressor = super::ParallelCompressor::new();
                compressor.decompress(&input[1..])?
            }
            _ => {
                return Err(CompressionError::FormatError(
                    format!("Unknown algorithm: {}", algorithm)
                ));
            }
        };

        let elapsed = start.elapsed();

        if let Some(stats) = self.last_stats.write().as_mut() {
            stats.decompression_time_ms = Some(elapsed.as_millis() as u64);
        }

        Ok(decompressed)
    }

    /// Analyze data characteristics
    fn analyze_data(&self, input: &[u8]) -> DataCharacteristics {
        let sample_size = self.config.sample_size.min(input.len());
        let sample = &input[..sample_size];

        // Calculate entropy
        let entropy = self.calculate_entropy(sample);

        // Calculate ASCII ratio
        let ascii_count = sample.iter().filter(|&&b| b >= 32 && b <= 126).count();
        let ascii_ratio = ascii_count as f64 / sample.len() as f64;

        // Calculate average and standard deviation
        let sum: u64 = sample.iter().map(|&b| b as u64).sum();
        let avg = sum as f64 / sample.len() as f64;

        let variance: f64 = sample
            .iter()
            .map(|&b| {
                let diff = b as f64 - avg;
                diff * diff
            })
            .sum::<f64>() / sample.len() as f64;
        let std_dev = variance.sqrt();

        // Calculate repetition score
        let repetition_score = self.calculate_repetition(sample);

        // Detect float data pattern
        let is_float_data = self.detect_float_pattern(sample);

        // Detect text data
        let is_text_data = ascii_ratio > 0.8;

        // Detect structured binary
        let is_structured = !is_text_data && repetition_score > 0.3;

        DataCharacteristics {
            entropy,
            ascii_ratio,
            avg_byte_value: avg,
            std_dev,
            repetition_score,
            is_float_data,
            is_text_data,
            is_structured,
        }
    }

    /// Calculate Shannon entropy
    fn calculate_entropy(&self, data: &[u8]) -> f64 {
        let mut counts = [0u32; 256];
        for &byte in data {
            counts[byte as usize] += 1;
        }

        let len = data.len() as f64;
        let mut entropy = 0.0;

        for &count in &counts {
            if count > 0 {
                let p = count as f64 / len;
                entropy -= p * p.log2();
            }
        }

        // Normalize to 0-1 range (max entropy for uniform distribution is 8.0)
        entropy / 8.0
    }

    /// Calculate repetition score
    fn calculate_repetition(&self, data: &[u8]) -> f64 {
        if data.len() < 2 {
            return 0.0;
        }

        let mut same_count = 0;
        for i in 1..data.len() {
            if data[i] == data[i - 1] {
                same_count += 1;
            }
        }

        same_count as f64 / (data.len() - 1) as f64
    }

    /// Detect if data looks like float coordinates
    fn detect_float_pattern(&self, data: &[u8]) -> bool {
        if data.len() < 16 {
            return false;
        }

        // Check if data is aligned to 4-byte boundaries and has float-like patterns
        let mut float_like = 0;
        for chunk in data.chunks_exact(4) {
            let value = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            // Check if it's a reasonable float value (not NaN, not too large)
            if value.is_finite() && value.abs() < 1e6 {
                float_like += 1;
            }
        }

        let total_floats = data.len() / 4;
        float_like as f64 / total_floats as f64 > 0.5
    }

    /// Select best algorithm based on characteristics
    fn select_algorithm(&self, chars: &DataCharacteristics) -> String {
        match self.config.strategy {
            CompressionStrategy::AlwaysLz4 => "lz4".to_string(),
            CompressionStrategy::AlwaysDelta => "delta".to_string(),
            CompressionStrategy::AlwaysMesh => "mesh".to_string(),
            CompressionStrategy::AlwaysDictionary => "dictionary".to_string(),
            CompressionStrategy::Fastest => "lz4".to_string(),
            CompressionStrategy::BestRatio => {
                // Choose algorithm likely to give best compression
                if chars.is_text_data {
                    "dictionary".to_string()
                } else if chars.is_float_data {
                    "delta".to_string()
                } else if chars.repetition_score > 0.4 {
                    "lz4".to_string()
                } else {
                    "parallel".to_string()
                }
            }
            CompressionStrategy::Auto => {
                // Heuristic-based selection
                if chars.is_text_data {
                    // Text data works well with dictionary compression
                    "dictionary".to_string()
                } else if chars.is_float_data {
                    // Float data works well with delta encoding
                    "delta".to_string()
                } else if chars.entropy < 0.5 {
                    // Low entropy (repetitive) - use LZ4
                    "lz4".to_string()
                } else if chars.is_structured && self.config.use_parallel {
                    // Structured binary with good size - use parallel
                    "parallel".to_string()
                } else {
                    // Default to LZ4
                    "lz4".to_string()
                }
            }
        }
    }

    /// Convert algorithm name to marker byte
    fn algorithm_to_marker(&self, algorithm: &str) -> u8 {
        match algorithm {
            "lz4" => 0x01,
            "delta" => 0x02,
            "mesh" => 0x03,
            "dictionary" => 0x04,
            "parallel" => 0x05,
            _ => 0x00,
        }
    }

    /// Convert marker byte to algorithm name
    fn marker_to_algorithm(&self, marker: u8) -> Result<String> {
        match marker {
            0x01 => Ok("lz4".to_string()),
            0x02 => Ok("delta".to_string()),
            0x03 => Ok("mesh".to_string()),
            0x04 => Ok("dictionary".to_string()),
            0x05 => Ok("parallel".to_string()),
            _ => Err(CompressionError::FormatError(
                format!("Unknown algorithm marker: {}", marker)
            )),
        }
    }
}

impl Default for AdaptiveCompressor {
    fn default() -> Self {
        Self::new()
    }
}

impl Compressor for AdaptiveCompressor {
    fn compress(&self, input: &[u8]) -> Result<Vec<u8>> {
        self.compress_internal(input)
    }

    fn decompress(&self, input: &[u8]) -> Result<Vec<u8>> {
        self.decompress_internal(input)
    }

    fn stats(&self) -> Option<CompressionStats> {
        self.last_stats.read().clone()
    }

    fn algorithm_name(&self) -> &str {
        "Adaptive"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_compression_text() {
        let compressor = AdaptiveCompressor::new();
        let input = b"LINE CIRCLE ARC POLYLINE LINE CIRCLE ARC POLYLINE".repeat(100);

        let compressed = compressor.compress(&input).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(input.to_vec(), decompressed);

        let strategy = compressor.last_strategy().unwrap();
        println!("Selected strategy for text: {}", strategy);
        assert!(strategy == "dictionary" || strategy == "lz4");

        let stats = compressor.stats().unwrap();
        println!("Compression ratio: {:.2}%", stats.compression_percentage());
    }

    #[test]
    fn test_adaptive_compression_floats() {
        let compressor = AdaptiveCompressor::new();

        // Generate float data
        let mut input = Vec::new();
        for i in 0..1000 {
            let value = (i as f32) * 0.1;
            input.extend_from_slice(&value.to_le_bytes());
        }

        let compressed = compressor.compress(&input).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(input, decompressed);

        let strategy = compressor.last_strategy().unwrap();
        println!("Selected strategy for floats: {}", strategy);
        // Should select delta or lz4
        assert!(strategy == "delta" || strategy == "lz4");
    }

    #[test]
    fn test_adaptive_compression_random() {
        let compressor = AdaptiveCompressor::new();
        let input: Vec<u8> = (0..10000).map(|i| (i * 7919) as u8).collect();

        let compressed = compressor.compress(&input).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(input, decompressed);

        let strategy = compressor.last_strategy().unwrap();
        println!("Selected strategy for random data: {}", strategy);

        let stats = compressor.stats().unwrap();
        println!("Compression ratio: {:.2}%", stats.compression_percentage());
    }

    #[test]
    fn test_data_analysis() {
        let compressor = AdaptiveCompressor::new();

        // Test text data
        let text_data = b"Hello World! This is some text data.";
        let chars = compressor.analyze_data(text_data);
        assert!(chars.is_text_data);
        assert!(chars.ascii_ratio > 0.8);

        // Test repetitive data
        let repetitive = vec![42u8; 1000];
        let chars = compressor.analyze_data(&repetitive);
        assert!(chars.repetition_score > 0.9);
        assert!(chars.entropy < 0.1);

        // Test float data
        let mut float_data = Vec::new();
        for i in 0..100 {
            float_data.extend_from_slice(&((i as f32) * 0.1).to_le_bytes());
        }
        let chars = compressor.analyze_data(&float_data);
        assert!(chars.is_float_data);
    }

    #[test]
    fn test_strategy_override() {
        let mut config = AdaptiveConfig::default();
        config.strategy = CompressionStrategy::AlwaysLz4;

        let compressor = AdaptiveCompressor::with_config(config);
        let input = b"Some test data";

        let _compressed = compressor.compress(input).unwrap();
        assert_eq!(compressor.last_strategy().unwrap(), "lz4");
    }
}
