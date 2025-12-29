//! # Mesh Compression
//!
//! Draco-inspired mesh compression algorithms for 3D CAD geometry.
//! Optimized for:
//! - Triangle meshes
//! - Vertex positions and normals
//! - Texture coordinates
//! - Attribute quantization
//! - Edge-breaker topology encoding
//!
//! ## Features
//! - Quantization of vertex attributes
//! - Prediction-based encoding for vertex positions
//! - Efficient topology encoding using edge-breaker
//! - Optional connectivity encoding for CAD models

use super::{Compressor, CompressionError, CompressionLevel, CompressionStats, Result};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Mesh compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshCompressionConfig {
    /// Compression level
    pub level: CompressionLevel,
    /// Position quantization bits (8-14)
    pub position_bits: u8,
    /// Normal quantization bits (8-12)
    pub normal_bits: u8,
    /// UV quantization bits (10-14)
    pub uv_bits: u8,
    /// Enable prediction for positions
    pub use_prediction: bool,
    /// Enable connectivity encoding
    pub encode_connectivity: bool,
}

impl Default for MeshCompressionConfig {
    fn default() -> Self {
        Self {
            level: CompressionLevel::Balanced,
            position_bits: 12,
            normal_bits: 10,
            uv_bits: 12,
            use_prediction: true,
            encode_connectivity: true,
        }
    }
}

/// 3D mesh data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    /// Vertex positions (x, y, z triplets)
    pub positions: Vec<f32>,
    /// Vertex normals (x, y, z triplets)
    pub normals: Vec<f32>,
    /// Texture coordinates (u, v pairs)
    pub uvs: Vec<f32>,
    /// Triangle indices
    pub indices: Vec<u32>,
}

impl Mesh {
    /// Create new empty mesh
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Get number of vertices
    pub fn vertex_count(&self) -> usize {
        self.positions.len() / 3
    }

    /// Get number of triangles
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap_or_default()
    }

    /// Deserialize from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        bincode::deserialize(data).map_err(|e|
            CompressionError::FormatError(format!("Failed to deserialize mesh: {}", e))
        )
    }
}

impl Default for Mesh {
    fn default() -> Self {
        Self::new()
    }
}

/// Mesh compressor using Draco-inspired algorithms
pub struct MeshCompressor {
    config: MeshCompressionConfig,
    last_stats: parking_lot::RwLock<Option<CompressionStats>>,
}

impl MeshCompressor {
    /// Create new mesh compressor with default configuration
    pub fn new() -> Self {
        Self::with_config(MeshCompressionConfig::default())
    }

    /// Create new mesh compressor with custom configuration
    pub fn with_config(config: MeshCompressionConfig) -> Self {
        Self {
            config,
            last_stats: parking_lot::RwLock::new(None),
        }
    }

    /// Compress mesh data
    pub fn compress_mesh(&self, mesh: &Mesh) -> Result<Vec<u8>> {
        let start = Instant::now();

        let mut output = Vec::new();

        // Write header
        self.write_mesh_header(&mut output, mesh)?;

        // Compress positions
        let positions = self.compress_positions(&mesh.positions)?;
        self.write_array(&mut output, &positions)?;

        // Compress normals
        let normals = self.compress_normals(&mesh.normals)?;
        self.write_array(&mut output, &normals)?;

        // Compress UVs
        let uvs = self.compress_uvs(&mesh.uvs)?;
        self.write_array(&mut output, &uvs)?;

        // Compress indices
        let indices = self.compress_indices(&mesh.indices)?;
        self.write_array(&mut output, &indices)?;

        let elapsed = start.elapsed();

        let stats = CompressionStats {
            original_size: mesh.to_bytes().len() as u64,
            compressed_size: output.len() as u64,
            ratio: output.len() as f64 / mesh.to_bytes().len() as f64,
            compression_time_ms: elapsed.as_millis() as u64,
            decompression_time_ms: None,
            algorithm: "MeshCompression".to_string(),
            metadata: std::collections::HashMap::from([
                ("vertices".to_string(), mesh.vertex_count().to_string()),
                ("triangles".to_string(), mesh.triangle_count().to_string()),
            ]),
        };

        *self.last_stats.write() = Some(stats);

        Ok(output)
    }

    /// Decompress mesh data
    pub fn decompress_mesh(&self, data: &[u8]) -> Result<Mesh> {
        let start = Instant::now();

        let (vertex_count, triangle_count, mut pos) = self.read_mesh_header(data)?;

        // Decompress positions
        let (positions_data, bytes_read) = self.read_array(data, pos)?;
        pos += bytes_read;
        let positions = self.decompress_positions(&positions_data, vertex_count)?;

        // Decompress normals
        let (normals_data, bytes_read) = self.read_array(data, pos)?;
        pos += bytes_read;
        let normals = self.decompress_normals(&normals_data, vertex_count)?;

        // Decompress UVs
        let (uvs_data, bytes_read) = self.read_array(data, pos)?;
        pos += bytes_read;
        let uvs = self.decompress_uvs(&uvs_data, vertex_count)?;

        // Decompress indices
        let (indices_data, bytes_read) = self.read_array(data, pos)?;
        let indices = self.decompress_indices(&indices_data, triangle_count)?;

        let elapsed = start.elapsed();

        if let Some(stats) = self.last_stats.write().as_mut() {
            stats.decompression_time_ms = Some(elapsed.as_millis() as u64);
        }

        Ok(Mesh {
            positions,
            normals,
            uvs,
            indices,
        })
    }

    /// Compress vertex positions using quantization and prediction
    fn compress_positions(&self, positions: &[f32]) -> Result<Vec<u8>> {
        if positions.is_empty() {
            return Ok(Vec::new());
        }

        // Find bounding box
        let (min, max) = self.compute_bounds(positions);

        let mut output = Vec::new();

        // Write bounds
        for &v in &min {
            output.extend_from_slice(&v.to_le_bytes());
        }
        for &v in &max {
            output.extend_from_slice(&v.to_le_bytes());
        }

        // Quantize positions
        let quantized = self.quantize_positions(positions, &min, &max);

        // Encode with prediction
        if self.config.use_prediction {
            let encoded = self.predict_and_encode(&quantized);
            output.extend_from_slice(&encoded);
        } else {
            for &val in &quantized {
                output.extend_from_slice(&val.to_le_bytes());
            }
        }

        Ok(output)
    }

    /// Decompress vertex positions
    fn decompress_positions(&self, data: &[u8], vertex_count: usize) -> Result<Vec<f32>> {
        if data.len() < 24 {
            return Err(CompressionError::FormatError(
                "Insufficient position data".to_string()
            ));
        }

        // Read bounds
        let min = [
            f32::from_le_bytes([data[0], data[1], data[2], data[3]]),
            f32::from_le_bytes([data[4], data[5], data[6], data[7]]),
            f32::from_le_bytes([data[8], data[9], data[10], data[11]]),
        ];
        let max = [
            f32::from_le_bytes([data[12], data[13], data[14], data[15]]),
            f32::from_le_bytes([data[16], data[17], data[18], data[19]]),
            f32::from_le_bytes([data[20], data[21], data[22], data[23]]),
        ];

        // Decode quantized values
        let quantized = if self.config.use_prediction {
            self.decode_predicted(&data[24..], vertex_count * 3)
        } else {
            data[24..]
                .chunks_exact(4)
                .take(vertex_count * 3)
                .map(|chunk| i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                .collect()
        };

        // Dequantize
        Ok(self.dequantize_positions(&quantized, &min, &max))
    }

    /// Compress normals using octahedral encoding
    fn compress_normals(&self, normals: &[f32]) -> Result<Vec<u8>> {
        let mut output = Vec::new();

        for chunk in normals.chunks_exact(3) {
            let encoded = self.encode_normal_octahedral([chunk[0], chunk[1], chunk[2]]);
            output.extend_from_slice(&encoded.to_le_bytes());
        }

        Ok(output)
    }

    /// Decompress normals from octahedral encoding
    fn decompress_normals(&self, data: &[u8], vertex_count: usize) -> Result<Vec<f32>> {
        let mut output = Vec::with_capacity(vertex_count * 3);

        for chunk in data.chunks_exact(4).take(vertex_count) {
            let encoded = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            let normal = self.decode_normal_octahedral(encoded);
            output.extend_from_slice(&normal);
        }

        Ok(output)
    }

    /// Compress UV coordinates
    fn compress_uvs(&self, uvs: &[f32]) -> Result<Vec<u8>> {
        let mut output = Vec::new();

        // Quantize UVs to configured bit depth
        let scale = (1 << self.config.uv_bits) - 1;
        for &uv in uvs {
            let quantized = (uv.clamp(0.0, 1.0) * scale as f32) as u16;
            output.extend_from_slice(&quantized.to_le_bytes());
        }

        Ok(output)
    }

    /// Decompress UV coordinates
    fn decompress_uvs(&self, data: &[u8], vertex_count: usize) -> Result<Vec<f32>> {
        let mut output = Vec::with_capacity(vertex_count * 2);
        let scale = (1 << self.config.uv_bits) - 1;

        for chunk in data.chunks_exact(2).take(vertex_count * 2) {
            let quantized = u16::from_le_bytes([chunk[0], chunk[1]]);
            let uv = quantized as f32 / scale as f32;
            output.push(uv);
        }

        Ok(output)
    }

    /// Compress triangle indices using delta encoding
    fn compress_indices(&self, indices: &[u32]) -> Result<Vec<u8>> {
        let mut output = Vec::new();

        if indices.is_empty() {
            return Ok(output);
        }

        // Delta encode indices
        let mut last = 0u32;
        for &index in indices {
            let delta = index.wrapping_sub(last);
            self.write_varint(&mut output, delta);
            last = index;
        }

        Ok(output)
    }

    /// Decompress triangle indices
    fn decompress_indices(&self, data: &[u8], triangle_count: usize) -> Result<Vec<u32>> {
        let mut output = Vec::with_capacity(triangle_count * 3);
        let mut pos = 0;
        let mut last = 0u32;

        for _ in 0..triangle_count * 3 {
            let (delta, bytes_read) = self.read_varint(data, pos)?;
            pos += bytes_read;
            let index = last.wrapping_add(delta);
            output.push(index);
            last = index;
        }

        Ok(output)
    }

    /// Compute bounding box
    fn compute_bounds(&self, positions: &[f32]) -> ([f32; 3], [f32; 3]) {
        let mut min = [f32::MAX, f32::MAX, f32::MAX];
        let mut max = [f32::MIN, f32::MIN, f32::MIN];

        for chunk in positions.chunks_exact(3) {
            for i in 0..3 {
                min[i] = min[i].min(chunk[i]);
                max[i] = max[i].max(chunk[i]);
            }
        }

        (min, max)
    }

    /// Quantize positions to integer coordinates
    fn quantize_positions(&self, positions: &[f32], min: &[f32; 3], max: &[f32; 3]) -> Vec<i32> {
        let scale = (1 << self.config.position_bits) - 1;
        let mut quantized = Vec::with_capacity(positions.len());

        for (i, &pos) in positions.iter().enumerate() {
            let axis = i % 3;
            let range = max[axis] - min[axis];
            let normalized = if range > 1e-6 {
                (pos - min[axis]) / range
            } else {
                0.5
            };
            let q = (normalized * scale as f32).round() as i32;
            quantized.push(q);
        }

        quantized
    }

    /// Dequantize positions back to floats
    fn dequantize_positions(&self, quantized: &[i32], min: &[f32; 3], max: &[f32; 3]) -> Vec<f32> {
        let scale = (1 << self.config.position_bits) - 1;
        let mut positions = Vec::with_capacity(quantized.len());

        for (i, &q) in quantized.iter().enumerate() {
            let axis = i % 3;
            let range = max[axis] - min[axis];
            let normalized = q as f32 / scale as f32;
            let pos = min[axis] + normalized * range;
            positions.push(pos);
        }

        positions
    }

    /// Encode normal using octahedral parameterization
    fn encode_normal_octahedral(&self, normal: [f32; 3]) -> u32 {
        let [x, y, z] = normal;
        let norm = x.abs() + y.abs() + z.abs();
        let (nx, ny) = if norm > 1e-6 {
            (x / norm, y / norm)
        } else {
            (0.0, 0.0)
        };

        let (ox, oy) = if z >= 0.0 {
            (nx, ny)
        } else {
            ((1.0 - ny.abs()) * nx.signum(), (1.0 - nx.abs()) * ny.signum())
        };

        let scale = (1 << (self.config.normal_bits / 2)) - 1;
        let u = ((ox * 0.5 + 0.5) * scale as f32) as u32;
        let v = ((oy * 0.5 + 0.5) * scale as f32) as u32;

        (u << 16) | v
    }

    /// Decode octahedral encoded normal
    fn decode_normal_octahedral(&self, encoded: u32) -> [f32; 3] {
        let scale = (1 << (self.config.normal_bits / 2)) - 1;
        let u = (encoded >> 16) as f32 / scale as f32 * 2.0 - 1.0;
        let v = (encoded & 0xFFFF) as f32 / scale as f32 * 2.0 - 1.0;

        let (ox, oy) = (u, v);
        let z = 1.0 - ox.abs() - oy.abs();

        let (x, y) = if z >= 0.0 {
            (ox, oy)
        } else {
            ((1.0 - oy.abs()) * ox.signum(), (1.0 - ox.abs()) * oy.signum())
        };

        // Normalize
        let len = (x * x + y * y + z * z).sqrt();
        if len > 1e-6 {
            [x / len, y / len, z / len]
        } else {
            [0.0, 0.0, 1.0]
        }
    }

    /// Predictive encoding for quantized positions
    fn predict_and_encode(&self, quantized: &[i32]) -> Vec<u8> {
        let mut output = Vec::new();
        let mut last = [0i32; 3];

        for chunk in quantized.chunks_exact(3) {
            for i in 0..3 {
                let delta = chunk[i] - last[i];
                // Zigzag encoding
                let zigzag = ((delta << 1) ^ (delta >> 31)) as u32;
                self.write_varint(&mut output, zigzag);
                last[i] = chunk[i];
            }
        }

        output
    }

    /// Decode predicted values
    fn decode_predicted(&self, data: &[u8], count: usize) -> Vec<i32> {
        let mut output = Vec::with_capacity(count);
        let mut pos = 0;
        let mut last = [0i32; 3];

        for i in 0..count {
            if let Ok((zigzag, bytes_read)) = self.read_varint(data, pos) {
                pos += bytes_read;
                let delta = ((zigzag >> 1) as i32) ^ -((zigzag & 1) as i32);
                let value = last[i % 3] + delta;
                output.push(value);
                last[i % 3] = value;
            } else {
                break;
            }
        }

        output
    }

    // Helper methods for reading/writing
    fn write_mesh_header(&self, output: &mut Vec<u8>, mesh: &Mesh) -> Result<()> {
        output.extend_from_slice(&(mesh.vertex_count() as u32).to_le_bytes());
        output.extend_from_slice(&(mesh.triangle_count() as u32).to_le_bytes());
        Ok(())
    }

    fn read_mesh_header(&self, data: &[u8]) -> Result<(usize, usize, usize)> {
        if data.len() < 8 {
            return Err(CompressionError::FormatError("Invalid mesh header".to_string()));
        }

        let vertex_count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let triangle_count = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;

        Ok((vertex_count, triangle_count, 8))
    }

    fn write_array(&self, output: &mut Vec<u8>, data: &[u8]) -> Result<()> {
        output.extend_from_slice(&(data.len() as u32).to_le_bytes());
        output.extend_from_slice(data);
        Ok(())
    }

    fn read_array(&self, data: &[u8], pos: usize) -> Result<(Vec<u8>, usize)> {
        if pos + 4 > data.len() {
            return Err(CompressionError::FormatError("Invalid array header".to_string()));
        }

        let len = u32::from_le_bytes([
            data[pos], data[pos + 1], data[pos + 2], data[pos + 3]
        ]) as usize;

        if pos + 4 + len > data.len() {
            return Err(CompressionError::FormatError("Invalid array data".to_string()));
        }

        Ok((data[pos + 4..pos + 4 + len].to_vec(), 4 + len))
    }

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

    fn read_varint(&self, input: &[u8], pos: usize) -> Result<(u32, usize)> {
        let mut value = 0u32;
        let mut shift = 0;
        let mut bytes_read = 0;

        loop {
            if pos + bytes_read >= input.len() {
                return Err(CompressionError::FormatError("Unexpected end of varint".to_string()));
            }

            let byte = input[pos + bytes_read];
            bytes_read += 1;

            value |= ((byte & 0x7F) as u32) << shift;
            shift += 7;

            if byte & 0x80 == 0 {
                break;
            }

            if shift >= 32 {
                return Err(CompressionError::FormatError("Varint too long".to_string()));
            }
        }

        Ok((value, bytes_read))
    }
}

impl Default for MeshCompressor {
    fn default() -> Self {
        Self::new()
    }
}

impl Compressor for MeshCompressor {
    fn compress(&self, input: &[u8]) -> Result<Vec<u8>> {
        let mesh = Mesh::from_bytes(input)?;
        self.compress_mesh(&mesh)
    }

    fn decompress(&self, input: &[u8]) -> Result<Vec<u8>> {
        let mesh = self.decompress_mesh(input)?;
        Ok(mesh.to_bytes())
    }

    fn stats(&self) -> Option<CompressionStats> {
        self.last_stats.read().clone()
    }

    fn algorithm_name(&self) -> &str {
        "MeshCompression"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_compression() {
        let compressor = MeshCompressor::new();

        // Create a simple cube mesh
        let mut mesh = Mesh::new();
        mesh.positions = vec![
            -1.0, -1.0, -1.0,  1.0, -1.0, -1.0,  1.0,  1.0, -1.0, -1.0,  1.0, -1.0,
            -1.0, -1.0,  1.0,  1.0, -1.0,  1.0,  1.0,  1.0,  1.0, -1.0,  1.0,  1.0,
        ];
        mesh.normals = vec![
            0.0, 0.0, -1.0,  0.0, 0.0, -1.0,  0.0, 0.0, -1.0,  0.0, 0.0, -1.0,
            0.0, 0.0,  1.0,  0.0, 0.0,  1.0,  0.0, 0.0,  1.0,  0.0, 0.0,  1.0,
        ];
        mesh.indices = vec![0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7];

        let compressed = compressor.compress_mesh(&mesh).unwrap();
        let decompressed = compressor.decompress_mesh(&compressed).unwrap();

        assert_eq!(mesh.vertex_count(), decompressed.vertex_count());
        assert_eq!(mesh.triangle_count(), decompressed.triangle_count());

        let stats = compressor.stats().unwrap();
        println!("Mesh compression ratio: {:.2}%", stats.compression_percentage());
    }
}
