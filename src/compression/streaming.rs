//! # Streaming Compression
//!
//! Streaming compression for large CAD files.
//! Enables processing files that don't fit in memory.
//!
//! ## Features
//! - Chunk-based processing
//! - Async I/O support
//! - Progressive compression/decompression
//! - Memory-efficient operation
//! - Configurable chunk sizes

use super::{Compressor, CompressionError, CompressionLevel, CompressionStats, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::time::Instant;
use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};

/// Streaming compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    /// Compression level
    pub level: CompressionLevel,
    /// Chunk size for streaming (in bytes)
    pub chunk_size: usize,
    /// Buffer size for I/O operations
    pub buffer_size: usize,
    /// Enable checksums for each chunk
    pub use_checksums: bool,
    /// Base compressor to use for chunks
    pub base_algorithm: BaseAlgorithm,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            level: CompressionLevel::Balanced,
            chunk_size: 1024 * 1024, // 1MB chunks
            buffer_size: 64 * 1024,  // 64KB I/O buffer
            use_checksums: true,
            base_algorithm: BaseAlgorithm::Lz4Custom,
        }
    }
}

/// Base compression algorithm for chunks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BaseAlgorithm {
    /// LZ4 custom variant
    Lz4Custom,
    /// Delta encoding
    Delta,
    /// Raw (no compression)
    Raw,
}

/// Chunk metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChunkMetadata {
    /// Original uncompressed size
    original_size: u32,
    /// Compressed size
    compressed_size: u32,
    /// CRC32 checksum (if enabled)
    checksum: Option<u32>,
}

/// Streaming compressor for large files
pub struct StreamingCompressor {
    config: StreamConfig,
    last_stats: parking_lot::RwLock<Option<CompressionStats>>,
}

impl StreamingCompressor {
    /// Create new streaming compressor with default configuration
    pub fn new() -> Self {
        Self::with_config(StreamConfig::default())
    }

    /// Create new streaming compressor with custom configuration
    pub fn with_config(config: StreamConfig) -> Self {
        Self {
            config,
            last_stats: parking_lot::RwLock::new(None),
        }
    }

    /// Compress data from reader to writer (synchronous)
    pub fn compress_stream<R: Read, W: Write>(
        &self,
        reader: &mut R,
        writer: &mut W,
    ) -> Result<CompressionStats> {
        let start = Instant::now();
        let mut total_original = 0u64;
        let mut total_compressed = 0u64;

        // Write header
        self.write_stream_header(writer)?;

        let mut buffer = vec![0u8; self.config.chunk_size];
        let mut chunk_count = 0u32;

        loop {
            // Read chunk
            let bytes_read = reader.read(&mut buffer).map_err(CompressionError::Io)?;
            if bytes_read == 0 {
                break;
            }

            let chunk = &buffer[..bytes_read];
            total_original += bytes_read as u64;

            // Compress chunk
            let compressed = self.compress_chunk(chunk)?;
            total_compressed += compressed.len() as u64;

            // Write chunk metadata and data
            self.write_chunk(writer, chunk, &compressed)?;
            chunk_count += 1;
        }

        // Write end marker
        writer.write_all(&0u32.to_le_bytes()).map_err(CompressionError::Io)?;
        writer.flush().map_err(CompressionError::Io)?;

        let elapsed = start.elapsed();

        let stats = CompressionStats {
            original_size: total_original,
            compressed_size: total_compressed,
            ratio: if total_original > 0 {
                total_compressed as f64 / total_original as f64
            } else {
                0.0
            },
            compression_time_ms: elapsed.as_millis() as u64,
            decompression_time_ms: None,
            algorithm: "Streaming".to_string(),
            metadata: std::collections::HashMap::from([
                ("chunk_count".to_string(), chunk_count.to_string()),
                ("chunk_size".to_string(), self.config.chunk_size.to_string()),
            ]),
        };

        *self.last_stats.write() = Some(stats.clone());
        Ok(stats)
    }

    /// Decompress data from reader to writer (synchronous)
    pub fn decompress_stream<R: Read, W: Write>(
        &self,
        reader: &mut R,
        writer: &mut W,
    ) -> Result<CompressionStats> {
        let start = Instant::now();
        let mut total_decompressed = 0u64;

        // Read and validate header
        self.read_stream_header(reader)?;

        loop {
            // Read chunk size
            let mut size_buf = [0u8; 4];
            reader.read_exact(&mut size_buf).map_err(CompressionError::Io)?;
            let compressed_size = u32::from_le_bytes(size_buf);

            if compressed_size == 0 {
                // End marker
                break;
            }

            // Read original size
            reader.read_exact(&mut size_buf).map_err(CompressionError::Io)?;
            let original_size = u32::from_le_bytes(size_buf);

            // Read checksum if enabled
            let checksum = if self.config.use_checksums {
                reader.read_exact(&mut size_buf).map_err(CompressionError::Io)?;
                Some(u32::from_le_bytes(size_buf))
            } else {
                None
            };

            // Read compressed chunk
            let mut compressed = vec![0u8; compressed_size as usize];
            reader.read_exact(&mut compressed).map_err(CompressionError::Io)?;

            // Decompress chunk
            let decompressed = self.decompress_chunk(&compressed, original_size as usize)?;

            // Verify checksum
            if let Some(expected_checksum) = checksum {
                let actual_checksum = self.calculate_crc32(&decompressed);
                if actual_checksum != expected_checksum {
                    return Err(CompressionError::FormatError(
                        "Checksum mismatch".to_string()
                    ));
                }
            }

            // Write decompressed data
            writer.write_all(&decompressed).map_err(CompressionError::Io)?;
            total_decompressed += decompressed.len() as u64;
        }

        writer.flush().map_err(CompressionError::Io)?;

        let elapsed = start.elapsed();

        if let Some(stats) = self.last_stats.write().as_mut() {
            stats.decompression_time_ms = Some(elapsed.as_millis() as u64);
        }

        Ok(CompressionStats {
            original_size: total_decompressed,
            compressed_size: 0,
            ratio: 0.0,
            compression_time_ms: 0,
            decompression_time_ms: Some(elapsed.as_millis() as u64),
            algorithm: "Streaming".to_string(),
            metadata: std::collections::HashMap::new(),
        })
    }

    /// Compress data asynchronously
    pub async fn compress_stream_async<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(
        &self,
        reader: &mut R,
        writer: &mut W,
    ) -> Result<CompressionStats> {
        let start = Instant::now();
        let mut total_original = 0u64;
        let mut total_compressed = 0u64;

        // Write header
        self.write_stream_header_async(writer).await?;

        let mut buffer = vec![0u8; self.config.chunk_size];
        let mut chunk_count = 0u32;

        loop {
            // Read chunk
            let bytes_read = reader.read(&mut buffer).await.map_err(CompressionError::Io)?;
            if bytes_read == 0 {
                break;
            }

            let chunk = &buffer[..bytes_read];
            total_original += bytes_read as u64;

            // Compress chunk
            let compressed = self.compress_chunk(chunk)?;
            total_compressed += compressed.len() as u64;

            // Write chunk metadata and data
            self.write_chunk_async(writer, chunk, &compressed).await?;
            chunk_count += 1;
        }

        // Write end marker
        writer.write_all(&0u32.to_le_bytes()).await.map_err(CompressionError::Io)?;
        writer.flush().await.map_err(CompressionError::Io)?;

        let elapsed = start.elapsed();

        let stats = CompressionStats {
            original_size: total_original,
            compressed_size: total_compressed,
            ratio: if total_original > 0 {
                total_compressed as f64 / total_original as f64
            } else {
                0.0
            },
            compression_time_ms: elapsed.as_millis() as u64,
            decompression_time_ms: None,
            algorithm: "StreamingAsync".to_string(),
            metadata: std::collections::HashMap::from([
                ("chunk_count".to_string(), chunk_count.to_string()),
            ]),
        };

        *self.last_stats.write() = Some(stats.clone());
        Ok(stats)
    }

    /// Decompress data asynchronously
    pub async fn decompress_stream_async<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(
        &self,
        reader: &mut R,
        writer: &mut W,
    ) -> Result<CompressionStats> {
        let start = Instant::now();
        let mut total_decompressed = 0u64;

        // Read and validate header
        self.read_stream_header_async(reader).await?;

        loop {
            // Read chunk size
            let mut size_buf = [0u8; 4];
            reader.read_exact(&mut size_buf).await.map_err(CompressionError::Io)?;
            let compressed_size = u32::from_le_bytes(size_buf);

            if compressed_size == 0 {
                break;
            }

            // Read original size
            reader.read_exact(&mut size_buf).await.map_err(CompressionError::Io)?;
            let original_size = u32::from_le_bytes(size_buf);

            // Read checksum if enabled
            let checksum = if self.config.use_checksums {
                reader.read_exact(&mut size_buf).await.map_err(CompressionError::Io)?;
                Some(u32::from_le_bytes(size_buf))
            } else {
                None
            };

            // Read compressed chunk
            let mut compressed = vec![0u8; compressed_size as usize];
            reader.read_exact(&mut compressed).await.map_err(CompressionError::Io)?;

            // Decompress chunk
            let decompressed = self.decompress_chunk(&compressed, original_size as usize)?;

            // Verify checksum
            if let Some(expected_checksum) = checksum {
                let actual_checksum = self.calculate_crc32(&decompressed);
                if actual_checksum != expected_checksum {
                    return Err(CompressionError::FormatError(
                        "Checksum mismatch".to_string()
                    ));
                }
            }

            // Write decompressed data
            writer.write_all(&decompressed).await.map_err(CompressionError::Io)?;
            total_decompressed += decompressed.len() as u64;
        }

        writer.flush().await.map_err(CompressionError::Io)?;

        let elapsed = start.elapsed();

        if let Some(stats) = self.last_stats.write().as_mut() {
            stats.decompression_time_ms = Some(elapsed.as_millis() as u64);
        }

        Ok(CompressionStats {
            original_size: total_decompressed,
            compressed_size: 0,
            ratio: 0.0,
            compression_time_ms: 0,
            decompression_time_ms: Some(elapsed.as_millis() as u64),
            algorithm: "StreamingAsync".to_string(),
            metadata: std::collections::HashMap::new(),
        })
    }

    /// Compress a single chunk
    fn compress_chunk(&self, chunk: &[u8]) -> Result<Vec<u8>> {
        match self.config.base_algorithm {
            BaseAlgorithm::Lz4Custom => {
                let compressor = super::Lz4CustomCompressor::new();
                compressor.compress(chunk)
            }
            BaseAlgorithm::Delta => {
                let compressor = super::DeltaEncoder::new();
                compressor.compress(chunk)
            }
            BaseAlgorithm::Raw => {
                Ok(chunk.to_vec())
            }
        }
    }

    /// Decompress a single chunk
    fn decompress_chunk(&self, chunk: &[u8], _expected_size: usize) -> Result<Vec<u8>> {
        match self.config.base_algorithm {
            BaseAlgorithm::Lz4Custom => {
                let compressor = super::Lz4CustomCompressor::new();
                compressor.decompress(chunk)
            }
            BaseAlgorithm::Delta => {
                let compressor = super::DeltaEncoder::new();
                compressor.decompress(chunk)
            }
            BaseAlgorithm::Raw => {
                Ok(chunk.to_vec())
            }
        }
    }

    /// Write stream header
    fn write_stream_header<W: Write>(&self, writer: &mut W) -> Result<()> {
        // Magic bytes "STRM"
        writer.write_all(b"STRM").map_err(CompressionError::Io)?;
        // Version
        writer.write_all(&1u16.to_le_bytes()).map_err(CompressionError::Io)?;
        // Flags
        let mut flags = 0u16;
        if self.config.use_checksums {
            flags |= 0x01;
        }
        writer.write_all(&flags.to_le_bytes()).map_err(CompressionError::Io)?;
        // Chunk size
        writer.write_all(&(self.config.chunk_size as u32).to_le_bytes())
            .map_err(CompressionError::Io)?;
        // Algorithm
        writer.write_all(&[self.config.base_algorithm as u8])
            .map_err(CompressionError::Io)?;
        // Reserved
        writer.write_all(&[0u8; 3]).map_err(CompressionError::Io)?;
        Ok(())
    }

    /// Read stream header
    fn read_stream_header<R: Read>(&self, reader: &mut R) -> Result<()> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic).map_err(CompressionError::Io)?;
        if &magic != b"STRM" {
            return Err(CompressionError::FormatError(
                "Invalid stream header".to_string()
            ));
        }

        let mut version = [0u8; 2];
        reader.read_exact(&mut version).map_err(CompressionError::Io)?;

        let mut flags = [0u8; 2];
        reader.read_exact(&mut flags).map_err(CompressionError::Io)?;

        let mut chunk_size = [0u8; 4];
        reader.read_exact(&mut chunk_size).map_err(CompressionError::Io)?;

        let mut algorithm = [0u8; 1];
        reader.read_exact(&mut algorithm).map_err(CompressionError::Io)?;

        let mut reserved = [0u8; 3];
        reader.read_exact(&mut reserved).map_err(CompressionError::Io)?;

        Ok(())
    }

    /// Write chunk with metadata
    fn write_chunk<W: Write>(&self, writer: &mut W, original: &[u8], compressed: &[u8]) -> Result<()> {
        writer.write_all(&(compressed.len() as u32).to_le_bytes())
            .map_err(CompressionError::Io)?;
        writer.write_all(&(original.len() as u32).to_le_bytes())
            .map_err(CompressionError::Io)?;

        if self.config.use_checksums {
            let checksum = self.calculate_crc32(original);
            writer.write_all(&checksum.to_le_bytes()).map_err(CompressionError::Io)?;
        }

        writer.write_all(compressed).map_err(CompressionError::Io)?;
        Ok(())
    }

    /// Write stream header (async)
    async fn write_stream_header_async<W: AsyncWrite + Unpin>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(b"STRM").await.map_err(CompressionError::Io)?;
        writer.write_all(&1u16.to_le_bytes()).await.map_err(CompressionError::Io)?;

        let mut flags = 0u16;
        if self.config.use_checksums {
            flags |= 0x01;
        }
        writer.write_all(&flags.to_le_bytes()).await.map_err(CompressionError::Io)?;
        writer.write_all(&(self.config.chunk_size as u32).to_le_bytes()).await
            .map_err(CompressionError::Io)?;
        writer.write_all(&[self.config.base_algorithm as u8]).await
            .map_err(CompressionError::Io)?;
        writer.write_all(&[0u8; 3]).await.map_err(CompressionError::Io)?;
        Ok(())
    }

    /// Read stream header (async)
    async fn read_stream_header_async<R: AsyncRead + Unpin>(&self, reader: &mut R) -> Result<()> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic).await.map_err(CompressionError::Io)?;
        if &magic != b"STRM" {
            return Err(CompressionError::FormatError("Invalid stream header".to_string()));
        }

        let mut buf = [0u8; 12];
        reader.read_exact(&mut buf).await.map_err(CompressionError::Io)?;
        Ok(())
    }

    /// Write chunk (async)
    async fn write_chunk_async<W: AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
        original: &[u8],
        compressed: &[u8],
    ) -> Result<()> {
        writer.write_all(&(compressed.len() as u32).to_le_bytes()).await
            .map_err(CompressionError::Io)?;
        writer.write_all(&(original.len() as u32).to_le_bytes()).await
            .map_err(CompressionError::Io)?;

        if self.config.use_checksums {
            let checksum = self.calculate_crc32(original);
            writer.write_all(&checksum.to_le_bytes()).await.map_err(CompressionError::Io)?;
        }

        writer.write_all(compressed).await.map_err(CompressionError::Io)?;
        Ok(())
    }

    /// Calculate CRC32 checksum
    fn calculate_crc32(&self, data: &[u8]) -> u32 {
        const CRC32_TABLE: [u32; 256] = generate_crc32_table();

        let mut crc = 0xFFFFFFFFu32;
        for &byte in data {
            let index = ((crc ^ byte as u32) & 0xFF) as usize;
            crc = (crc >> 8) ^ CRC32_TABLE[index];
        }
        !crc
    }
}

/// Generate CRC32 lookup table at compile time
const fn generate_crc32_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut crc = i as u32;
        let mut j = 0;
        while j < 8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
            j += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
}

impl Default for StreamingCompressor {
    fn default() -> Self {
        Self::new()
    }
}

impl Compressor for StreamingCompressor {
    fn compress(&self, input: &[u8]) -> Result<Vec<u8>> {
        let mut reader = std::io::Cursor::new(input);
        let mut writer = std::io::Cursor::new(Vec::new());
        self.compress_stream(&mut reader, &mut writer)?;
        Ok(writer.into_inner())
    }

    fn decompress(&self, input: &[u8]) -> Result<Vec<u8>> {
        let mut reader = std::io::Cursor::new(input);
        let mut writer = std::io::Cursor::new(Vec::new());
        self.decompress_stream(&mut reader, &mut writer)?;
        Ok(writer.into_inner())
    }

    fn stats(&self) -> Option<CompressionStats> {
        self.last_stats.read().clone()
    }

    fn algorithm_name(&self) -> &str {
        "Streaming"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_compression() {
        let compressor = StreamingCompressor::new();
        let input = vec![42u8; 10_000];

        let mut reader = std::io::Cursor::new(&input);
        let mut writer = std::io::Cursor::new(Vec::new());

        let stats = compressor.compress_stream(&mut reader, &mut writer).unwrap();
        assert!(stats.compressed_size < stats.original_size);

        let compressed = writer.into_inner();
        let mut reader = std::io::Cursor::new(&compressed);
        let mut writer = std::io::Cursor::new(Vec::new());

        compressor.decompress_stream(&mut reader, &mut writer).unwrap();
        let decompressed = writer.into_inner();

        assert_eq!(input, decompressed);
    }

    #[tokio::test]
    async fn test_async_streaming_compression() {
        let compressor = StreamingCompressor::new();
        let input = vec![42u8; 10_000];

        let mut reader = tokio::io::BufReader::new(&input[..]);
        let mut writer = Vec::new();

        let stats = compressor.compress_stream_async(&mut reader, &mut writer).await.unwrap();
        assert!(stats.compressed_size < stats.original_size);

        let mut reader = tokio::io::BufReader::new(&writer[..]);
        let mut decompressed = Vec::new();

        compressor.decompress_stream_async(&mut reader, &mut decompressed).await.unwrap();
        assert_eq!(input, decompressed);
    }
}
