//! # Delta Encoding Compression
//!
//! Delta encoding for versioned CAD data.
//! This is particularly effective for:
//! - Version control and incremental saves
//! - Time-series CAD data
//! - Coordinate sequences with small differences
//! - Similar geometry with minor variations
//!
//! ## Algorithm
//! - XOR-based delta for binary data
//! - Predictive delta for coordinate data
//! - Run-length encoding for zero deltas
//! - Context-aware prediction for better compression

use super::{Compressor, CompressionError, CompressionLevel, CompressionStats, Result};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Delta encoding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaConfig {
    /// Compression level
    pub level: CompressionLevel,
    /// Use predictive delta (vs simple XOR)
    pub use_prediction: bool,
    /// Enable run-length encoding for zeros
    pub use_rle: bool,
    /// Context window size for prediction
    pub context_size: usize,
    /// Data type hint for optimization
    pub data_type: DeltaDataType,
}

impl Default for DeltaConfig {
    fn default() -> Self {
        Self {
            level: CompressionLevel::Balanced,
            use_prediction: true,
            use_rle: true,
            context_size: 8,
            data_type: DeltaDataType::Mixed,
        }
    }
}

/// Data type hint for delta encoding optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeltaDataType {
    /// Mixed data (default)
    Mixed,
    /// Floating point coordinates
    Coordinates,
    /// Integer IDs/indices
    Integers,
    /// Color/material data
    Colors,
    /// Text/metadata
    Text,
}

/// Delta encoder for versioned CAD data
pub struct DeltaEncoder {
    config: DeltaConfig,
    last_stats: parking_lot::RwLock<Option<CompressionStats>>,
    // Previous version for incremental encoding
    previous_data: parking_lot::RwLock<Option<Vec<u8>>>,
}

impl DeltaEncoder {
    /// Create new delta encoder with default configuration
    pub fn new() -> Self {
        Self::with_config(DeltaConfig::default())
    }

    /// Create new delta encoder with custom configuration
    pub fn with_config(config: DeltaConfig) -> Self {
        Self {
            config,
            last_stats: parking_lot::RwLock::new(None),
            previous_data: parking_lot::RwLock::new(None),
        }
    }

    /// Set previous version for incremental encoding
    pub fn set_previous(&self, data: Vec<u8>) {
        *self.previous_data.write() = Some(data);
    }

    /// Clear previous version
    pub fn clear_previous(&self) {
        *self.previous_data.write() = None;
    }

    /// Compress using delta encoding
    fn compress_internal(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        let previous = self.previous_data.read();
        let base = previous.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);

        let mut output = Vec::with_capacity(input.len() / 2);

        // Write header
        self.write_header(&mut output, input.len(), base.len())?;

        if base.is_empty() {
            // No previous version, just store the data
            output.extend_from_slice(input);
        } else {
            // Compute and encode deltas
            self.encode_deltas(&mut output, input, base)?;
        }

        Ok(output)
    }

    /// Decompress delta-encoded data
    fn decompress_internal(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.len() < 12 {
            return Err(CompressionError::InvalidInput(
                "Input too small for delta header".to_string()
            ));
        }

        let (original_size, base_size, pos) = self.read_header(input)?;

        let previous = self.previous_data.read();
        let base = previous.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);

        if base_size == 0 {
            // No delta, just return the stored data
            if input.len() - pos < original_size {
                return Err(CompressionError::FormatError(
                    "Insufficient data".to_string()
                ));
            }
            Ok(input[pos..pos + original_size].to_vec())
        } else {
            // Decode deltas
            if base.len() != base_size {
                return Err(CompressionError::InvalidInput(
                    format!("Base size mismatch: expected {}, got {}", base_size, base.len())
                ));
            }
            self.decode_deltas(&input[pos..], original_size, base)
        }
    }

    /// Write delta encoding header
    fn write_header(
        &self,
        output: &mut Vec<u8>,
        original_size: usize,
        base_size: usize,
    ) -> Result<()> {
        output.extend_from_slice(&(original_size as u32).to_le_bytes());
        output.extend_from_slice(&(base_size as u32).to_le_bytes());
        output.push(self.config.data_type as u8);
        output.push(if self.config.use_prediction { 1 } else { 0 });
        output.push(if self.config.use_rle { 1 } else { 0 });
        output.push(0); // Reserved
        Ok(())
    }

    /// Read delta encoding header
    fn read_header(&self, input: &[u8]) -> Result<(usize, usize, usize)> {
        let original_size = u32::from_le_bytes([
            input[0], input[1], input[2], input[3]
        ]) as usize;
        let base_size = u32::from_le_bytes([
            input[4], input[5], input[6], input[7]
        ]) as usize;
        // Skip flags at positions 8, 9, 10, 11
        Ok((original_size, base_size, 12))
    }

    /// Encode deltas between input and base
    fn encode_deltas(
        &self,
        output: &mut Vec<u8>,
        input: &[u8],
        base: &[u8],
    ) -> Result<()> {
        match self.config.data_type {
            DeltaDataType::Coordinates => {
                self.encode_float_deltas(output, input, base)
            }
            DeltaDataType::Integers => {
                self.encode_integer_deltas(output, input, base)
            }
            _ => {
                self.encode_xor_deltas(output, input, base)
            }
        }
    }

    /// Encode using XOR delta (general purpose)
    fn encode_xor_deltas(
        &self,
        output: &mut Vec<u8>,
        input: &[u8],
        base: &[u8],
    ) -> Result<()> {
        let max_len = input.len().max(base.len());
        let mut zero_run = 0;

        for i in 0..max_len {
            let input_byte = input.get(i).copied().unwrap_or(0);
            let base_byte = base.get(i).copied().unwrap_or(0);
            let delta = input_byte ^ base_byte;

            if delta == 0 && self.config.use_rle {
                zero_run += 1;
                if zero_run == 255 {
                    output.push(0x00);
                    output.push(255);
                    zero_run = 0;
                }
            } else {
                if zero_run > 0 {
                    output.push(0x00);
                    output.push(zero_run);
                    zero_run = 0;
                }
                output.push(delta);
            }
        }

        if zero_run > 0 {
            output.push(0x00);
            output.push(zero_run);
        }

        Ok(())
    }

    /// Encode deltas for floating point coordinates
    fn encode_float_deltas(
        &self,
        output: &mut Vec<u8>,
        input: &[u8],
        base: &[u8],
    ) -> Result<()> {
        if input.len() % 4 != 0 || base.len() % 4 != 0 {
            return self.encode_xor_deltas(output, input, base);
        }

        let input_floats: Vec<f32> = input
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        let base_floats: Vec<f32> = base
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        // Predictive delta encoding for floats
        let mut predictor = 0.0f32;
        for i in 0..input_floats.len() {
            let input_val = input_floats[i];
            let base_val = base_floats.get(i).copied().unwrap_or(0.0);

            let predicted = if self.config.use_prediction && i > 0 {
                base_val + predictor
            } else {
                base_val
            };

            let delta = input_val - predicted;
            predictor = delta; // Update predictor

            // Encode delta as bytes
            output.extend_from_slice(&delta.to_le_bytes());
        }

        Ok(())
    }

    /// Encode deltas for integer data
    fn encode_integer_deltas(
        &self,
        output: &mut Vec<u8>,
        input: &[u8],
        base: &[u8],
    ) -> Result<()> {
        if input.len() % 4 != 0 || base.len() % 4 != 0 {
            return self.encode_xor_deltas(output, input, base);
        }

        let input_ints: Vec<i32> = input
            .chunks_exact(4)
            .map(|chunk| i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        let base_ints: Vec<i32> = base
            .chunks_exact(4)
            .map(|chunk| i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        for i in 0..input_ints.len() {
            let input_val = input_ints[i];
            let base_val = base_ints.get(i).copied().unwrap_or(0);
            let delta = input_val.wrapping_sub(base_val);

            // Use zigzag encoding for better compression of small deltas
            let zigzag = ((delta << 1) ^ (delta >> 31)) as u32;
            self.write_varint(output, zigzag);
        }

        Ok(())
    }

    /// Decode deltas to reconstruct original data
    fn decode_deltas(
        &self,
        input: &[u8],
        original_size: usize,
        base: &[u8],
    ) -> Result<Vec<u8>> {
        match self.config.data_type {
            DeltaDataType::Coordinates => {
                self.decode_float_deltas(input, original_size, base)
            }
            DeltaDataType::Integers => {
                self.decode_integer_deltas(input, original_size, base)
            }
            _ => {
                self.decode_xor_deltas(input, original_size, base)
            }
        }
    }

    /// Decode XOR deltas
    fn decode_xor_deltas(
        &self,
        input: &[u8],
        original_size: usize,
        base: &[u8],
    ) -> Result<Vec<u8>> {
        let mut output = Vec::with_capacity(original_size);
        let mut pos = 0;

        while output.len() < original_size && pos < input.len() {
            let delta = input[pos];
            pos += 1;

            if delta == 0x00 && self.config.use_rle && pos < input.len() {
                // RLE zero run
                let count = input[pos] as usize;
                pos += 1;
                for i in 0..count {
                    if output.len() < original_size {
                        let base_byte = base.get(output.len()).copied().unwrap_or(0);
                        output.push(base_byte);
                    }
                }
            } else {
                let base_byte = base.get(output.len()).copied().unwrap_or(0);
                output.push(base_byte ^ delta);
            }
        }

        Ok(output)
    }

    /// Decode float deltas
    fn decode_float_deltas(
        &self,
        input: &[u8],
        original_size: usize,
        base: &[u8],
    ) -> Result<Vec<u8>> {
        let num_floats = original_size / 4;
        let base_floats: Vec<f32> = base
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        let mut output = Vec::with_capacity(original_size);
        let mut predictor = 0.0f32;

        for i in 0..num_floats {
            if i * 4 + 4 > input.len() {
                break;
            }

            let delta = f32::from_le_bytes([
                input[i * 4],
                input[i * 4 + 1],
                input[i * 4 + 2],
                input[i * 4 + 3],
            ]);

            let base_val = base_floats.get(i).copied().unwrap_or(0.0);
            let predicted = if self.config.use_prediction && i > 0 {
                base_val + predictor
            } else {
                base_val
            };

            let value = predicted + delta;
            predictor = delta;

            output.extend_from_slice(&value.to_le_bytes());
        }

        Ok(output)
    }

    /// Decode integer deltas
    fn decode_integer_deltas(
        &self,
        input: &[u8],
        original_size: usize,
        base: &[u8],
    ) -> Result<Vec<u8>> {
        let num_ints = original_size / 4;
        let base_ints: Vec<i32> = base
            .chunks_exact(4)
            .map(|chunk| i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        let mut output = Vec::with_capacity(original_size);
        let mut pos = 0;

        for i in 0..num_ints {
            let (zigzag, bytes_read) = self.read_varint(input, pos)?;
            pos += bytes_read;

            // Decode zigzag
            let delta = ((zigzag >> 1) as i32) ^ -((zigzag & 1) as i32);
            let base_val = base_ints.get(i).copied().unwrap_or(0);
            let value = base_val.wrapping_add(delta);

            output.extend_from_slice(&value.to_le_bytes());
        }

        Ok(output)
    }

    /// Write variable-length integer
    fn write_varint(&self, output: &mut Vec<u8>, mut value: u32) {
        loop {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            output.push(byte);
            if value == 0 {
                break;
            }
        }
    }

    /// Read variable-length integer
    fn read_varint(&self, input: &[u8], pos: usize) -> Result<(u32, usize)> {
        let mut value = 0u32;
        let mut shift = 0;
        let mut bytes_read = 0;

        loop {
            if pos + bytes_read >= input.len() {
                return Err(CompressionError::FormatError(
                    "Unexpected end of varint".to_string()
                ));
            }

            let byte = input[pos + bytes_read];
            bytes_read += 1;

            value |= ((byte & 0x7F) as u32) << shift;
            shift += 7;

            if byte & 0x80 == 0 {
                break;
            }

            if shift >= 32 {
                return Err(CompressionError::FormatError(
                    "Varint too long".to_string()
                ));
            }
        }

        Ok((value, bytes_read))
    }
}

impl Default for DeltaEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Compressor for DeltaEncoder {
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
            algorithm: "DeltaEncoding".to_string(),
            metadata: std::collections::HashMap::new(),
        };

        *self.last_stats.write() = Some(stats);
        Ok(result)
    }

    fn decompress(&self, input: &[u8]) -> Result<Vec<u8>> {
        let start = Instant::now();
        let result = self.decompress_internal(input)?;
        let elapsed = start.elapsed();

        if let Some(stats) = self.last_stats.write().as_mut() {
            stats.decompression_time_ms = Some(elapsed.as_millis() as u64);
        }

        Ok(result)
    }

    fn stats(&self) -> Option<CompressionStats> {
        self.last_stats.read().clone()
    }

    fn algorithm_name(&self) -> &str {
        "DeltaEncoding"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_encode_no_base() {
        let encoder = DeltaEncoder::new();
        let input = b"Test data";
        let compressed = encoder.compress(input).unwrap();
        let decompressed = encoder.decompress(&compressed).unwrap();
        assert_eq!(input.as_ref(), decompressed.as_slice());
    }

    #[test]
    fn test_delta_encode_with_base() {
        let encoder = DeltaEncoder::new();
        let base = b"Test data version 1";
        let input = b"Test data version 2";

        encoder.set_previous(base.to_vec());
        let compressed = encoder.compress(input).unwrap();
        let decompressed = encoder.decompress(&compressed).unwrap();

        assert_eq!(input.as_ref(), decompressed.as_slice());
        assert!(compressed.len() < input.len());
    }

    #[test]
    fn test_delta_encode_floats() {
        let mut config = DeltaConfig::default();
        config.data_type = DeltaDataType::Coordinates;
        let encoder = DeltaEncoder::with_config(config);

        // Create base and modified coordinate data
        let mut base = Vec::new();
        let mut input = Vec::new();
        for i in 0..100 {
            let base_val = (i as f32) * 0.1;
            let input_val = base_val + 0.001; // Small delta
            base.extend_from_slice(&base_val.to_le_bytes());
            input.extend_from_slice(&input_val.to_le_bytes());
        }

        encoder.set_previous(base);
        let compressed = encoder.compress(&input).unwrap();
        let decompressed = encoder.decompress(&compressed).unwrap();

        assert_eq!(input, decompressed);
        println!("Float delta compression: {}%",
            (1.0 - compressed.len() as f64 / input.len() as f64) * 100.0);
    }
}
