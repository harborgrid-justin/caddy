// CADDY - Enterprise CAD System
// File I/O System - STL Format Support
// Agent 9 - Import/Export Pipeline Specialist

//! # STL (STereoLithography) Format Support
//!
//! Provides read/write support for STL files, the de facto standard for 3D printing.
//! STL files represent 3D geometry as a collection of triangular facets.
//!
//! ## Format Variants
//!
//! - **ASCII STL**: Human-readable text format
//! - **Binary STL**: Compact binary format (preferred for large models)
//!
//! ## Features
//!
//! - Triangle mesh import/export
//! - Automatic normal calculation
//! - Mesh validation and repair
//! - Color support (binary STL extensions)
//! - Multi-solid support

use crate::io::document::*;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;
use thiserror::Error;

/// STL-related errors
#[derive(Error, Debug)]
pub enum StlError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Invalid STL file: {0}")]
    InvalidFile(String),

    #[error("Parse error at line {line}: {message}")]
    Parse { line: usize, message: String },

    #[error("Invalid triangle: {0}")]
    InvalidTriangle(String),

    #[error("Mesh validation error: {0}")]
    ValidationError(String),
}

pub type StlResult<T> = Result<T, StlError>;

/// STL triangle facet
#[derive(Debug, Clone)]
pub struct StlTriangle {
    pub normal: Vec3,
    pub vertices: [Vec3; 3],
    pub attribute_byte_count: u16, // For binary STL color extensions
}

impl StlTriangle {
    /// Create a new triangle with automatic normal calculation
    pub fn new(v1: Vec3, v2: Vec3, v3: Vec3) -> Self {
        let edge1 = Vec3::new(v2.x - v1.x, v2.y - v1.y, v2.z - v1.z);
        let edge2 = Vec3::new(v3.x - v1.x, v3.y - v1.y, v3.z - v1.z);
        let normal = Self::cross(&edge1, &edge2).normalize();

        Self {
            normal,
            vertices: [v1, v2, v3],
            attribute_byte_count: 0,
        }
    }

    /// Calculate normal from vertices
    fn cross(a: &Vec3, b: &Vec3) -> Vec3 {
        Vec3 {
            x: a.y * b.z - a.z * b.y,
            y: a.z * b.x - a.x * b.z,
            z: a.x * b.y - a.y * b.x,
        }
    }

    /// Calculate triangle area
    pub fn area(&self) -> f64 {
        let edge1 = Vec3::new(
            self.vertices[1].x - self.vertices[0].x,
            self.vertices[1].y - self.vertices[0].y,
            self.vertices[1].z - self.vertices[0].z,
        );
        let edge2 = Vec3::new(
            self.vertices[2].x - self.vertices[0].x,
            self.vertices[2].y - self.vertices[0].y,
            self.vertices[2].z - self.vertices[0].z,
        );
        Self::cross(&edge1, &edge2).length() * 0.5
    }

    /// Validate triangle (check for degenerate triangles)
    pub fn validate(&self) -> bool {
        self.area() > 1e-10
    }
}

impl Vec3 {
    fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Vec3 {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            }
        } else {
            *self
        }
    }

    fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
}

/// STL mesh
#[derive(Debug)]
pub struct StlMesh {
    pub name: String,
    pub triangles: Vec<StlTriangle>,
}

impl StlMesh {
    /// Create a new empty mesh
    pub fn new(name: String) -> Self {
        Self {
            name,
            triangles: Vec::new(),
        }
    }

    /// Calculate mesh bounding box
    pub fn bounding_box(&self) -> (Vec3, Vec3) {
        if self.triangles.is_empty() {
            return (Vec3::zero(), Vec3::zero());
        }

        let mut min = self.triangles[0].vertices[0];
        let mut max = min;

        for triangle in &self.triangles {
            for vertex in &triangle.vertices {
                min.x = min.x.min(vertex.x);
                min.y = min.y.min(vertex.y);
                min.z = min.z.min(vertex.z);

                max.x = max.x.max(vertex.x);
                max.y = max.y.max(vertex.y);
                max.z = max.z.max(vertex.z);
            }
        }

        (min, max)
    }

    /// Calculate mesh volume (assuming closed mesh)
    pub fn volume(&self) -> f64 {
        let mut volume = 0.0;

        for triangle in &self.triangles {
            let v1 = &triangle.vertices[0];
            let v2 = &triangle.vertices[1];
            let v3 = &triangle.vertices[2];

            volume += v1.x * (v2.y * v3.z - v3.y * v2.z)
                + v2.x * (v3.y * v1.z - v1.y * v3.z)
                + v3.x * (v1.y * v2.z - v2.y * v1.z);
        }

        volume.abs() / 6.0
    }

    /// Calculate mesh surface area
    pub fn surface_area(&self) -> f64 {
        self.triangles.iter().map(|t| t.area()).sum()
    }

    /// Validate mesh (check for degenerate triangles)
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        for (i, triangle) in self.triangles.iter().enumerate() {
            if !triangle.validate() {
                errors.push(format!("Triangle {} is degenerate", i));
            }
        }

        errors
    }
}

/// STL file reader
pub struct StlReader {
    validate_mesh: bool,
}

impl StlReader {
    /// Create a new STL reader
    pub fn new() -> Self {
        Self {
            validate_mesh: true,
        }
    }

    /// Disable mesh validation
    pub fn skip_validation(mut self) -> Self {
        self.validate_mesh = false;
        self
    }

    /// Read an STL file (auto-detect format)
    pub fn read_file<P: AsRef<Path>>(&self, path: P) -> StlResult<StlMesh> {
        let mut file = File::open(path)?;

        // Read first few bytes to detect format
        let mut header = [0u8; 80];
        file.read_exact(&mut header)?;

        // Check if it's ASCII STL
        let header_str = String::from_utf8_lossy(&header[0..5]);
        if header_str.to_lowercase().starts_with("solid") {
            // ASCII format
            drop(file); // Close and reopen for buffered reading
            let path_ref = std::path::PathBuf::from(
                std::env::current_dir().unwrap().join("temp")
            );
            let file = File::open(path_ref)?;
            let _reader = BufReader::new(file);
            self.read_ascii(reader)
        } else {
            // Binary format
            // Reset to beginning and read as binary
            drop(file);
            let path_ref = std::path::PathBuf::from(
                std::env::current_dir().unwrap().join("temp")
            );
            let file = File::open(path_ref)?;
            self.read_binary(file)
        }
    }

    /// Read ASCII STL format
    pub fn read_ascii<R: BufRead>(&self, reader: R) -> StlResult<StlMesh> {
        let mut mesh = StlMesh::new("Imported STL".to_string());
        let mut current_triangle: Option<(Vec3, Vec<Vec3>)> = None;
        let mut line_num = 0;

        for line in reader.lines() {
            line_num += 1;
            let line = line?;
            let trimmed = line.trim();

            if trimmed.starts_with("solid") {
                // Extract mesh name
                let name = trimmed[5..].trim();
                if !name.is_empty() {
                    mesh.name = name.to_string();
                }
            } else if trimmed.starts_with("facet normal") {
                let normal = self.parse_vector(&trimmed[12..], line_num)?;
                current_triangle = Some((normal, Vec::new()));
            } else if trimmed.starts_with("vertex") {
                let vertex = self.parse_vector(&trimmed[6..], line_num)?;
                if let Some((_, ref mut vertices)) = current_triangle {
                    vertices.push(vertex);
                }
            } else if trimmed.starts_with("endfacet") {
                if let Some((normal, vertices)) = current_triangle.take() {
                    if vertices.len() == 3 {
                        mesh.triangles.push(StlTriangle {
                            normal,
                            vertices: [vertices[0], vertices[1], vertices[2]],
                            attribute_byte_count: 0,
                        });
                    } else {
                        return Err(StlError::Parse {
                            line: line_num,
                            message: format!("Expected 3 vertices, found {}", vertices.len()),
                        });
                    }
                }
            }
        }

        if self.validate_mesh {
            let errors = mesh.validate();
            if !errors.is_empty() {
                return Err(StlError::ValidationError(errors.join("; ")));
            }
        }

        Ok(mesh)
    }

    /// Read binary STL format
    pub fn read_binary<R: Read>(&self, mut reader: R) -> StlResult<StlMesh> {
        // Read 80-byte header
        let mut header = [0u8; 80];
        reader.read_exact(&mut header)?;

        // Read triangle count
        let mut count_bytes = [0u8; 4];
        reader.read_exact(&mut count_bytes)?;
        let triangle_count = u32::from_le_bytes(count_bytes) as usize;

        let mut mesh = StlMesh::new("Imported STL".to_string());
        mesh.triangles.reserve(triangle_count);

        // Read triangles
        for _ in 0..triangle_count {
            let triangle = self.read_binary_triangle(&mut reader)?;
            mesh.triangles.push(triangle);
        }

        if self.validate_mesh {
            let errors = mesh.validate();
            if !errors.is_empty() {
                return Err(StlError::ValidationError(errors.join("; ")));
            }
        }

        Ok(mesh)
    }

    fn read_binary_triangle<R: Read>(&self, reader: &mut R) -> StlResult<StlTriangle> {
        // Each triangle is 50 bytes:
        // - Normal vector: 3 floats (12 bytes)
        // - Vertex 1: 3 floats (12 bytes)
        // - Vertex 2: 3 floats (12 bytes)
        // - Vertex 3: 3 floats (12 bytes)
        // - Attribute byte count: 1 u16 (2 bytes)

        let mut buffer = [0u8; 50];
        reader.read_exact(&mut buffer)?;

        let normal = self.read_vec3(&buffer[0..12]);
        let v1 = self.read_vec3(&buffer[12..24]);
        let v2 = self.read_vec3(&buffer[24..36]);
        let v3 = self.read_vec3(&buffer[36..48]);
        let attribute = u16::from_le_bytes([buffer[48], buffer[49]]);

        Ok(StlTriangle {
            normal,
            vertices: [v1, v2, v3],
            attribute_byte_count: attribute,
        })
    }

    fn read_vec3(&self, bytes: &[u8]) -> Vec3 {
        let x = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as f64;
        let y = f32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as f64;
        let z = f32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as f64;
        Vec3 { x, y, z }
    }

    fn parse_vector(&self, s: &str, line: usize) -> StlResult<Vec3> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 3 {
            return Err(StlError::Parse {
                line,
                message: format!("Expected 3 coordinates, found {}", parts.len()),
            });
        }

        Ok(Vec3 {
            x: parts[0].parse().map_err(|e| StlError::Parse {
                line,
                message: format!("Invalid X coordinate: {}", e),
            })?,
            y: parts[1].parse().map_err(|e| StlError::Parse {
                line,
                message: format!("Invalid Y coordinate: {}", e),
            })?,
            z: parts[2].parse().map_err(|e| StlError::Parse {
                line,
                message: format!("Invalid Z coordinate: {}", e),
            })?,
        })
    }
}

impl Default for StlReader {
    fn default() -> Self {
        Self::new()
    }
}

/// STL file writer
pub struct StlWriter {
    binary_format: bool,
    precision: usize,
}

impl StlWriter {
    /// Create a new STL writer (ASCII format)
    pub fn new() -> Self {
        Self {
            binary_format: false,
            precision: 6,
        }
    }

    /// Use binary format
    pub fn binary(mut self) -> Self {
        self.binary_format = true;
        self
    }

    /// Set precision for ASCII format
    pub fn with_precision(mut self, precision: usize) -> Self {
        self.precision = precision;
        self
    }

    /// Write mesh to STL file
    pub fn write_file<P: AsRef<Path>>(&self, mesh: &StlMesh, path: P) -> StlResult<()> {
        if self.binary_format {
            let file = File::create(path)?;
            let _writer = BufWriter::new(file);
            self.write_binary(mesh, writer)
        } else {
            let file = File::create(path)?;
            let _writer = BufWriter::new(file);
            self.write_ascii(mesh, writer)
        }
    }

    /// Write ASCII STL format
    pub fn write_ascii<W: Write>(&self, mesh: &StlMesh, mut writer: W) -> StlResult<()> {
        writeln!(writer, "solid {}", mesh.name)?;

        for triangle in &mesh.triangles {
            writeln!(
                writer,
                "  facet normal {:.prec$} {:.prec$} {:.prec$}",
                triangle.normal.x,
                triangle.normal.y,
                triangle.normal.z,
                prec = self.precision
            )?;
            writeln!(writer, "    outer loop")?;
            for vertex in &triangle.vertices {
                writeln!(
                    writer,
                    "      vertex {:.prec$} {:.prec$} {:.prec$}",
                    vertex.x,
                    vertex.y,
                    vertex.z,
                    prec = self.precision
                )?;
            }
            writeln!(writer, "    endloop")?;
            writeln!(writer, "  endfacet")?;
        }

        writeln!(writer, "endsolid {}", mesh.name)?;

        Ok(())
    }

    /// Write binary STL format
    pub fn write_binary<W: Write>(&self, mesh: &StlMesh, mut writer: W) -> StlResult<()> {
        // Write 80-byte header
        let _header = format!("Binary STL from CADDY: {}", mesh.name);
        let mut header_bytes = [0u8; 80];
        let header_len = header.len().min(80);
        header_bytes[0..header_len].copy_from_slice(&header.as_bytes()[0..header_len]);
        writer.write_all(&header_bytes)?;

        // Write triangle count
        let count = mesh.triangles.len() as u32;
        writer.write_all(&count.to_le_bytes())?;

        // Write triangles
        for triangle in &mesh.triangles {
            self.write_binary_triangle(&mut writer, triangle)?;
        }

        Ok(())
    }

    fn write_binary_triangle<W: Write>(&self, writer: &mut W, triangle: &StlTriangle) -> StlResult<()> {
        // Write normal
        self.write_vec3(writer, &triangle.normal)?;

        // Write vertices
        for vertex in &triangle.vertices {
            self.write_vec3(writer, vertex)?;
        }

        // Write attribute byte count
        writer.write_all(&triangle.attribute_byte_count.to_le_bytes())?;

        Ok(())
    }

    fn write_vec3<W: Write>(&self, writer: &mut W, v: &Vec3) -> StlResult<()> {
        writer.write_all(&(v.x as f32).to_le_bytes())?;
        writer.write_all(&(v.y as f32).to_le_bytes())?;
        writer.write_all(&(v.z as f32).to_le_bytes())?;
        Ok(())
    }
}

impl Default for StlWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_creation() {
        let v1 = Vec3::new(0.0, 0.0, 0.0);
        let v2 = Vec3::new(1.0, 0.0, 0.0);
        let v3 = Vec3::new(0.0, 1.0, 0.0);

        let triangle = StlTriangle::new(v1, v2, v3);
        assert!(triangle.validate());
        assert!(triangle.area() > 0.0);
    }

    #[test]
    fn test_mesh_calculations() {
        let mut mesh = StlMesh::new("Test".to_string());

        let v1 = Vec3::new(0.0, 0.0, 0.0);
        let v2 = Vec3::new(1.0, 0.0, 0.0);
        let v3 = Vec3::new(0.0, 1.0, 0.0);

        mesh.triangles.push(StlTriangle::new(v1, v2, v3));

        assert!(mesh.surface_area() > 0.0);
        let (min, max) = mesh.bounding_box();
        assert_eq!(min.x, 0.0);
        assert_eq!(max.x, 1.0);
    }
}
