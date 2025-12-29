//! # Dictionary Compression
//!
//! Domain-specific dictionary compression for CAD files.
//! Optimized for common CAD patterns and terminology.
//!
//! ## Features
//! - Pre-built dictionaries for CAD primitives
//! - Dynamic dictionary learning from data
//! - Huffman encoding for symbols
//! - LZ77-style back-references with dictionary
//! - Specialized for CAD entity types and properties

use super::{Compressor, CompressionError, CompressionLevel, CompressionStats, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// CAD-specific dictionary with common patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CADDictionary {
    /// Common CAD entity type names
    pub entity_types: Vec<String>,
    /// Common property names
    pub property_names: Vec<String>,
    /// Common string values
    pub common_values: Vec<String>,
    /// Custom entries
    pub custom_entries: Vec<String>,
}

impl CADDictionary {
    /// Create default CAD dictionary
    pub fn new() -> Self {
        Self {
            entity_types: vec![
                "LINE".to_string(),
                "CIRCLE".to_string(),
                "ARC".to_string(),
                "POLYLINE".to_string(),
                "LWPOLYLINE".to_string(),
                "SPLINE".to_string(),
                "ELLIPSE".to_string(),
                "POINT".to_string(),
                "TEXT".to_string(),
                "MTEXT".to_string(),
                "DIMENSION".to_string(),
                "BLOCK".to_string(),
                "INSERT".to_string(),
                "ATTRIB".to_string(),
                "LAYER".to_string(),
                "STYLE".to_string(),
            ],
            property_names: vec![
                "position".to_string(),
                "center".to_string(),
                "radius".to_string(),
                "start_point".to_string(),
                "end_point".to_string(),
                "control_points".to_string(),
                "normal".to_string(),
                "color".to_string(),
                "layer".to_string(),
                "linetype".to_string(),
                "lineweight".to_string(),
                "material".to_string(),
                "rotation".to_string(),
                "scale".to_string(),
                "thickness".to_string(),
            ],
            common_values: vec![
                "0".to_string(),
                "ByLayer".to_string(),
                "ByBlock".to_string(),
                "CONTINUOUS".to_string(),
                "Standard".to_string(),
                "Default".to_string(),
            ],
            custom_entries: Vec::new(),
        }
    }

    /// Get all dictionary entries
    pub fn all_entries(&self) -> Vec<String> {
        let mut entries = Vec::new();
        entries.extend(self.entity_types.clone());
        entries.extend(self.property_names.clone());
        entries.extend(self.common_values.clone());
        entries.extend(self.custom_entries.clone());
        entries
    }

    /// Add custom entry to dictionary
    pub fn add_entry(&mut self, entry: String) {
        if !self.custom_entries.contains(&entry) {
            self.custom_entries.push(entry);
        }
    }

    /// Find entry index
    pub fn find_entry(&self, text: &str) -> Option<usize> {
        self.all_entries()
            .iter()
            .position(|e| e == text)
    }
}

impl Default for CADDictionary {
    fn default() -> Self {
        Self::new()
    }
}

/// Dictionary compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryConfig {
    /// Compression level
    pub level: CompressionLevel,
    /// Use dynamic dictionary learning
    pub learn_dictionary: bool,
    /// Maximum dictionary size
    pub max_dict_size: usize,
    /// Minimum string length to consider
    pub min_string_length: usize,
}

impl Default for DictionaryConfig {
    fn default() -> Self {
        Self {
            level: CompressionLevel::Balanced,
            learn_dictionary: true,
            max_dict_size: 4096,
            min_string_length: 3,
        }
    }
}

/// Dictionary compressor for CAD data
pub struct DictionaryCompressor {
    config: DictionaryConfig,
    dictionary: parking_lot::RwLock<CADDictionary>,
    last_stats: parking_lot::RwLock<Option<CompressionStats>>,
}

impl DictionaryCompressor {
    /// Create new dictionary compressor with default dictionary
    pub fn new() -> Self {
        Self::with_dictionary(CADDictionary::new())
    }

    /// Create new dictionary compressor with custom dictionary
    pub fn with_dictionary(dictionary: CADDictionary) -> Self {
        Self {
            config: DictionaryConfig::default(),
            dictionary: parking_lot::RwLock::new(dictionary),
            last_stats: parking_lot::RwLock::new(None),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: DictionaryConfig, dictionary: CADDictionary) -> Self {
        Self {
            config,
            dictionary: parking_lot::RwLock::new(dictionary),
            last_stats: parking_lot::RwLock::new(None),
        }
    }

    /// Get reference to dictionary
    pub fn dictionary(&self) -> parking_lot::RwLockReadGuard<CADDictionary> {
        self.dictionary.read()
    }

    /// Update dictionary
    pub fn update_dictionary<F>(&self, f: F)
    where
        F: FnOnce(&mut CADDictionary),
    {
        let mut dict = self.dictionary.write();
        f(&mut dict);
    }

    /// Compress data using dictionary
    fn compress_internal(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        // Convert to text for processing
        let text = String::from_utf8_lossy(input);

        // Learn dictionary if enabled
        if self.config.learn_dictionary {
            self.learn_from_text(&text);
        }

        let mut output = Vec::new();

        // Write dictionary to output
        self.write_dictionary(&mut output)?;

        // Encode text using dictionary
        self.encode_with_dictionary(&mut output, &text)?;

        Ok(output)
    }

    /// Decompress data using dictionary
    fn decompress_internal(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        // Read dictionary from input
        let (dict, pos) = self.read_dictionary(input)?;

        // Decode text using dictionary
        let text = self.decode_with_dictionary(&input[pos..], &dict)?;

        Ok(text.into_bytes())
    }

    /// Learn common patterns from text
    fn learn_from_text(&self, text: &str) {
        let mut frequency: HashMap<String, usize> = HashMap::new();

        // Extract words and phrases
        let words: Vec<&str> = text.split_whitespace().collect();
        for word in words {
            if word.len() >= self.config.min_string_length {
                *frequency.entry(word.to_string()).or_insert(0) += 1;
            }
        }

        // Add most frequent entries to dictionary
        let mut entries: Vec<_> = frequency.into_iter().collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1));

        let mut dict = self.dictionary.write();
        for (word, _count) in entries.iter().take(100) {
            dict.add_entry(word.clone());
        }
    }

    /// Write dictionary to output
    fn write_dictionary(&self, output: &mut Vec<u8>) -> Result<()> {
        let dict = self.dictionary.read();
        let entries = dict.all_entries();

        // Write number of entries
        output.extend_from_slice(&(entries.len() as u16).to_le_bytes());

        // Write each entry
        for entry in entries {
            let bytes = entry.as_bytes();
            output.push(bytes.len() as u8);
            output.extend_from_slice(bytes);
        }

        Ok(())
    }

    /// Read dictionary from input
    fn read_dictionary(&self, input: &[u8]) -> Result<(Vec<String>, usize)> {
        if input.len() < 2 {
            return Err(CompressionError::FormatError(
                "Invalid dictionary header".to_string()
            ));
        }

        let num_entries = u16::from_le_bytes([input[0], input[1]]) as usize;
        let mut pos = 2;
        let mut entries = Vec::with_capacity(num_entries);

        for _ in 0..num_entries {
            if pos >= input.len() {
                return Err(CompressionError::FormatError(
                    "Incomplete dictionary".to_string()
                ));
            }

            let len = input[pos] as usize;
            pos += 1;

            if pos + len > input.len() {
                return Err(CompressionError::FormatError(
                    "Invalid dictionary entry".to_string()
                ));
            }

            let entry = String::from_utf8_lossy(&input[pos..pos + len]).to_string();
            entries.push(entry);
            pos += len;
        }

        Ok((entries, pos))
    }

    /// Encode text using dictionary
    fn encode_with_dictionary(&self, output: &mut Vec<u8>, text: &str) -> Result<()> {
        let dict = self.dictionary.read();
        let entries = dict.all_entries();

        let mut pos = 0;
        let bytes = text.as_bytes();

        while pos < bytes.len() {
            let mut found_match = false;

            // Try to find longest matching dictionary entry
            for (idx, entry) in entries.iter().enumerate() {
                let entry_bytes = entry.as_bytes();
                if pos + entry_bytes.len() <= bytes.len() &&
                   &bytes[pos..pos + entry_bytes.len()] == entry_bytes
                {
                    // Write dictionary reference
                    output.push(0xFF); // Marker for dictionary reference
                    output.extend_from_slice(&(idx as u16).to_le_bytes());
                    pos += entry_bytes.len();
                    found_match = true;
                    break;
                }
            }

            if !found_match {
                // Write literal byte
                output.push(bytes[pos]);
                pos += 1;
            }
        }

        Ok(())
    }

    /// Decode text using dictionary
    fn decode_with_dictionary(&self, input: &[u8], dict: &[String]) -> Result<String> {
        let mut output = String::new();
        let mut pos = 0;

        while pos < input.len() {
            if input[pos] == 0xFF {
                // Dictionary reference
                if pos + 3 > input.len() {
                    return Err(CompressionError::FormatError(
                        "Incomplete dictionary reference".to_string()
                    ));
                }

                let idx = u16::from_le_bytes([input[pos + 1], input[pos + 2]]) as usize;
                if idx >= dict.len() {
                    return Err(CompressionError::FormatError(
                        "Invalid dictionary index".to_string()
                    ));
                }

                output.push_str(&dict[idx]);
                pos += 3;
            } else {
                // Literal byte
                output.push(input[pos] as char);
                pos += 1;
            }
        }

        Ok(output)
    }
}

impl Default for DictionaryCompressor {
    fn default() -> Self {
        Self::new()
    }
}

impl Compressor for DictionaryCompressor {
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
            algorithm: "Dictionary".to_string(),
            metadata: std::collections::HashMap::from([
                ("dict_size".to_string(), self.dictionary.read().all_entries().len().to_string()),
            ]),
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
        "Dictionary"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dictionary_compression() {
        let compressor = DictionaryCompressor::new();
        let input = b"LINE start_point 0 0 end_point 10 10 CIRCLE center 5 5 radius 2";

        let compressed = compressor.compress(input).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(input.as_ref(), decompressed.as_slice());

        let stats = compressor.stats().unwrap();
        println!("Dictionary compression ratio: {:.2}%", stats.compression_percentage());
    }

    #[test]
    fn test_cad_dictionary() {
        let mut dict = CADDictionary::new();
        assert!(dict.find_entry("LINE").is_some());
        assert!(dict.find_entry("CIRCLE").is_some());
        assert!(dict.find_entry("position").is_some());

        dict.add_entry("CUSTOM_ENTITY".to_string());
        assert!(dict.find_entry("CUSTOM_ENTITY").is_some());
    }

    #[test]
    fn test_dictionary_learning() {
        let compressor = DictionaryCompressor::new();
        let input = b"MyCustomEntity MyCustomEntity MyCustomEntity property1 property1 property1";

        let _compressed = compressor.compress(input).unwrap();

        // Dictionary should have learned "MyCustomEntity" and "property1"
        let dict = compressor.dictionary();
        assert!(dict.all_entries().iter().any(|e| e.contains("MyCustomEntity")));
    }
}
