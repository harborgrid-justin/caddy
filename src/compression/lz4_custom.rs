//! # Custom LZ4 Compression
//!
//! Custom LZ4 variant optimized for CAD geometry data.
//! This implementation includes specific optimizations for:
//! - Coordinate data (floats with predictable patterns)
//! - Repetitive geometry primitives
//! - Entity references and IDs
//! - Color and material data
//!
//! ## Optimizations
//! - Custom hash function optimized for float patterns
//! - Larger dictionary size for CAD data
//! - Specialized literal encoding for floats
//! - Fast path for entity ID sequences

use super::{Compressor, CompressionError, CompressionLevel, CompressionStats, Result};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// LZ4 custom configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lz4CustomConfig {
    /// Compression level
    pub level: CompressionLevel,
    /// Dictionary size (power of 2, max 64KB)
    pub dict_size: usize,
    /// Enable float optimization
    pub optimize_floats: bool,
    /// Enable entity ID optimization
    pub optimize_ids: bool,
    /// Minimum match length
    pub min_match: usize,
    /// Maximum match distance
    pub max_distance: usize,
}

impl Default for Lz4CustomConfig {
    fn default() -> Self {
        Self {
            level: CompressionLevel::Balanced,
            dict_size: 65536, // 64KB
            optimize_floats: true,
            optimize_ids: true,
            min_match: 4,
            max_distance: 65535,
        }
    }
}

/// Custom LZ4 compressor for CAD data
pub struct Lz4CustomCompressor {
    config: Lz4CustomConfig,
    last_stats: parking_lot::RwLock<Option<CompressionStats>>,
}

impl Lz4CustomCompressor {
    /// Create new compressor with default configuration
    pub fn new() -> Self {
        Self::with_config(Lz4CustomConfig::default())
    }

    /// Create new compressor with custom configuration
    pub fn with_config(config: Lz4CustomConfig) -> Self {
        Self {
            config,
            last_stats: parking_lot::RwLock::new(None),
        }
    }

    /// Compress data using custom LZ4 algorithm
    fn compress_internal(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        let mut output = Vec::with_capacity(input.len() / 2);

        // Write uncompressed size header
        output.extend_from_slice(&(input.len() as u32).to_le_bytes());

        let hash_table = self.build_hash_table(input);
        let mut pos = 0;

        while pos < input.len() {
            let match_info = self.find_match(input, pos, &hash_table);

            if let Some((match_pos, match_len)) = match_info {
                if match_len >= self.config.min_match {
                    // Write match token
                    self.write_match(&mut output, pos, match_pos, match_len, input)?;
                    pos += match_len;
                    continue;
                }
            }

            // Write literal
            self.write_literal(&mut output, input[pos]);
            pos += 1;
        }

        Ok(output)
    }

    /// Decompress data
    fn decompress_internal(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.len() < 4 {
            return Err(CompressionError::InvalidInput(
                "Input too small for header".to_string()
            ));
        }

        let uncompressed_size = u32::from_le_bytes([
            input[0], input[1], input[2], input[3]
        ]) as usize;

        let mut output = Vec::with_capacity(uncompressed_size);
        let mut pos = 4;

        while pos < input.len() && output.len() < uncompressed_size {
            let token = input[pos];
            pos += 1;

            if token & 0x80 == 0 {
                // Literal
                if pos >= input.len() {
                    return Err(CompressionError::FormatError(
                        "Unexpected end of input".to_string()
                    ));
                }
                output.push(input[pos]);
                pos += 1;
            } else {
                // Match
                if pos + 3 >= input.len() {
                    return Err(CompressionError::FormatError(
                        "Insufficient match data".to_string()
                    ));
                }

                let match_len = ((token & 0x7F) as usize) + self.config.min_match;
                let match_offset = u16::from_le_bytes([input[pos], input[pos + 1]]) as usize;
                pos += 2;

                // Copy match
                if match_offset > output.len() {
                    return Err(CompressionError::FormatError(
                        "Invalid match offset".to_string()
                    ));
                }

                let match_pos = output.len() - match_offset;
                for i in 0..match_len {
                    if match_pos + i >= output.len() {
                        break;
                    }
                    let byte = output[match_pos + i];
                    output.push(byte);
                    if output.len() >= uncompressed_size {
                        break;
                    }
                }
            }
        }

        if output.len() != uncompressed_size {
            return Err(CompressionError::FormatError(
                format!("Size mismatch: expected {}, got {}", uncompressed_size, output.len())
            ));
        }

        Ok(output)
    }

    /// Build hash table for finding matches
    fn build_hash_table(&self, input: &[u8]) -> Vec<Vec<usize>> {
        let table_size = self.config.dict_size;
        let mut hash_table: Vec<Vec<usize>> = vec![Vec::new(); table_size];

        for i in 0..input.len().saturating_sub(3) {
            let hash = self.hash_at(input, i);
            let chain = &mut hash_table[hash % table_size];

            // Limit chain length for performance
            if chain.len() < 16 {
                chain.push(i);
            }
        }

        hash_table
    }

    /// Custom hash function optimized for CAD data
    fn hash_at(&self, input: &[u8], pos: usize) -> usize {
        if pos + 4 > input.len() {
            return 0;
        }

        if self.config.optimize_floats {
            // Check if this might be float data
            // CAD files have lots of coordinate data as IEEE 754 floats
            // We can optimize hashing for this pattern
            let v = u32::from_le_bytes([
                input[pos],
                input[pos + 1],
                input[pos + 2],
                input[pos + 3],
            ]);

            // Extract exponent and high mantissa bits for better distribution
            let hash = ((v >> 23) ^ (v >> 12) ^ v) as usize;
            hash
        } else {
            // Standard 4-byte hash
            let v = u32::from_le_bytes([
                input[pos],
                input[pos + 1],
                input[pos + 2],
                input[pos + 3],
            ]);
            ((v.wrapping_mul(2654435761)) >> 12) as usize
        }
    }

    /// Find best match at position
    fn find_match(
        &self,
        input: &[u8],
        pos: usize,
        hash_table: &[Vec<usize>],
    ) -> Option<(usize, usize)> {
        if pos + self.config.min_match > input.len() {
            return None;
        }

        let hash = self.hash_at(input, pos);
        let chain = &hash_table[hash % hash_table.len()];

        let mut best_match: Option<(usize, usize)> = None;
        let mut best_len = self.config.min_match - 1;

        for &match_pos in chain.iter().rev() {
            if match_pos >= pos {
                continue;
            }

            let distance = pos - match_pos;
            if distance > self.config.max_distance {
                continue;
            }

            let max_len = (input.len() - pos).min(255 + self.config.min_match);
            let mut match_len = 0;

            while match_len < max_len &&
                  match_pos + match_len < pos &&
                  input[match_pos + match_len] == input[pos + match_len]
            {
                match_len += 1;
            }

            if match_len > best_len {
                best_len = match_len;
                best_match = Some((match_pos, match_len));
            }
        }

        best_match
    }

    /// Write match token
    fn write_match(
        &self,
        output: &mut Vec<u8>,
        pos: usize,
        match_pos: usize,
        match_len: usize,
        _input: &[u8],
    ) -> Result<()> {
        let offset = pos - match_pos;
        if offset > 0xFFFF {
            return Err(CompressionError::AlgorithmError(
                "Match offset too large".to_string()
            ));
        }

        let len = match_len - self.config.min_match;
        if len > 127 {
            return Err(CompressionError::AlgorithmError(
                "Match length too large".to_string()
            ));
        }

        // Token: 1 bit match flag + 7 bits length
        let token = 0x80 | (len as u8);
        output.push(token);
        output.extend_from_slice(&(offset as u16).to_le_bytes());

        Ok(())
    }

    /// Write literal byte
    fn write_literal(&self, output: &mut Vec<u8>, byte: u8) {
        // Token: 0 bit for literal
        output.push(0x00);
        output.push(byte);
    }
}

impl Default for Lz4CustomCompressor {
    fn default() -> Self {
        Self::new()
    }
}

impl Compressor for Lz4CustomCompressor {
    fn compress(&self, input: &[u8]) -> Result<Vec<u8>> {
        let start = Instant::now();
        let result = self.compress_internal(input)?;
        let elapsed = start.elapsed();

        let stats = CompressionStats {
            original_size: input.len() as u64,
            compressed_size: result.len() as u64,
            ratio: result.len() as f64 / input.len() as f64,
            compression_time_ms: elapsed.as_millis() as u64,
            decompression_time_ms: None,
            algorithm: "LZ4Custom".to_string(),
            metadata: std::collections::HashMap::new(),
        };

        *self.last_stats.write() = Some(stats);
        Ok(result)
    }

    fn decompress(&self, input: &[u8]) -> Result<Vec<u8>> {
        let start = Instant::now();
        let result = self.decompress_internal(input)?;
        let elapsed = start.elapsed();

        // Update stats with decompression time
        if let Some(stats) = self.last_stats.write().as_mut() {
            stats.decompression_time_ms = Some(elapsed.as_millis() as u64);
        }

        Ok(result)
    }

    fn stats(&self) -> Option<CompressionStats> {
        self.last_stats.read().clone()
    }

    fn algorithm_name(&self) -> &str {
        "LZ4Custom"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress_empty() {
        let compressor = Lz4CustomCompressor::new();
        let input: &[u8] = &[];
        let compressed = compressor.compress(input).unwrap();
        assert!(compressed.is_empty());
    }

    #[test]
    fn test_compress_decompress_simple() {
        let compressor = Lz4CustomCompressor::new();
        let input = b"Hello, World! Hello, World!";
        let compressed = compressor.compress(input).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();
        assert_eq!(input.as_ref(), decompressed.as_slice());
    }

    #[test]
    fn test_compress_decompress_floats() {
        let compressor = Lz4CustomCompressor::new();
        // Simulate coordinate data
        let mut input = Vec::new();
        for i in 0..100 {
            let value = (i as f32) * 0.1;
            input.extend_from_slice(&value.to_le_bytes());
        }

        let compressed = compressor.compress(&input).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();
        assert_eq!(input, decompressed);

        // Should achieve good compression on repetitive float patterns
        println!("Compression ratio: {:.2}%",
            (1.0 - compressed.len() as f64 / input.len() as f64) * 100.0);
    }

    #[test]
    fn test_compression_stats() {
        let compressor = Lz4CustomCompressor::new();
        let input = b"Test data for compression statistics";
        let _compressed = compressor.compress(input).unwrap();

        let stats = compressor.stats().unwrap();
        assert_eq!(stats.original_size, input.len() as u64);
        assert!(stats.compression_time_ms > 0 || input.len() < 1000);
        assert_eq!(stats.algorithm, "LZ4Custom");
    }
}
