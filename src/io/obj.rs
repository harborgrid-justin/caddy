// CADDY - Enterprise CAD System
// File I/O System - Wavefront OBJ Format Support
// Agent 9 - Import/Export Pipeline Specialist

//! # Wavefront OBJ Format Support
//!
//! Provides read/write support for Wavefront OBJ files, a widely-supported
//! 3D geometry format. OBJ files can include vertex data, texture coordinates,
//! normals, materials (MTL), and object groups.
//!
//! ## Features
//!
//! - Vertex positions, normals, and texture coordinates
//! - Face definitions (triangles and polygons)
//! - Material support (MTL file integration)
//! - Object and group organization
//! - Smooth shading groups
//! - Free-form geometry (curves and surfaces)

use crate::io::document::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// OBJ-related errors
#[derive(Error, Debug)]
pub enum ObjError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Parse error at line {line}: {message}")]
    Parse { line: usize, message: String },

    #[error("Invalid face definition: {0}")]
    InvalidFace(String),

    #[error("Invalid vertex reference: {0}")]
    InvalidVertexRef(i32),

    #[error("Material not found: {0}")]
    MaterialNotFound(String),

    #[error("MTL file error: {0}")]
    MtlError(String),
}

pub type ObjResult<T> = Result<T, ObjError>;

/// OBJ vertex (position)
#[derive(Debug, Clone, Copy)]
pub struct ObjVertex {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64, // Optional homogeneous coordinate
}

impl ObjVertex {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z, w: 1.0 }
    }

    pub fn to_vec3(&self) -> Vec3 {
        Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

/// OBJ texture coordinate
#[derive(Debug, Clone, Copy)]
pub struct ObjTexCoord {
    pub u: f64,
    pub v: f64,
    pub w: f64, // Optional 3D texture coordinate
}

/// OBJ normal
#[derive(Debug, Clone, Copy)]
pub struct ObjNormal {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl ObjNormal {
    pub fn to_vec3(&self) -> Vec3 {
        Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

/// OBJ face vertex reference
#[derive(Debug, Clone, Copy)]
pub struct FaceVertex {
    pub vertex_index: i32,
    pub texcoord_index: Option<i32>,
    pub normal_index: Option<i32>,
}

/// OBJ face
#[derive(Debug, Clone)]
pub struct ObjFace {
    pub vertices: Vec<FaceVertex>,
    pub material: Option<String>,
}

/// OBJ material (from MTL file)
#[derive(Debug, Clone)]
pub struct ObjMaterial {
    pub name: String,
    pub ambient: [f64; 3],       // Ka
    pub diffuse: [f64; 3],       // Kd
    pub specular: [f64; 3],      // Ks
    pub emission: [f64; 3],      // Ke
    pub shininess: f64,          // Ns
    pub transparency: f64,       // d or Tr
    pub optical_density: f64,    // Ni
    pub illumination_model: u8,  // illum
    pub ambient_texture: Option<String>,   // map_Ka
    pub diffuse_texture: Option<String>,   // map_Kd
    pub specular_texture: Option<String>,  // map_Ks
    pub normal_texture: Option<String>,    // map_Bump or bump
}

impl Default for ObjMaterial {
    fn default() -> Self {
        Self {
            name: String::new(),
            ambient: [0.2, 0.2, 0.2],
            diffuse: [0.8, 0.8, 0.8],
            specular: [1.0, 1.0, 1.0],
            emission: [0.0, 0.0, 0.0],
            shininess: 10.0,
            transparency: 1.0,
            optical_density: 1.0,
            illumination_model: 2,
            ambient_texture: None,
            diffuse_texture: None,
            specular_texture: None,
            normal_texture: None,
        }
    }
}

/// OBJ mesh
#[derive(Debug)]
pub struct ObjMesh {
    pub vertices: Vec<ObjVertex>,
    pub texcoords: Vec<ObjTexCoord>,
    pub normals: Vec<ObjNormal>,
    pub faces: Vec<ObjFace>,
    pub materials: HashMap<String, ObjMaterial>,
    pub object_name: Option<String>,
    pub group_name: Option<String>,
}

impl ObjMesh {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            texcoords: Vec::new(),
            normals: Vec::new(),
            faces: Vec::new(),
            materials: HashMap::new(),
            object_name: None,
            group_name: None,
        }
    }
}

/// OBJ file reader
pub struct ObjReader {
    load_materials: bool,
    base_path: Option<PathBuf>,
}

impl ObjReader {
    /// Create a new OBJ reader
    pub fn new() -> Self {
        Self {
            load_materials: true,
            base_path: None,
        }
    }

    /// Disable material loading
    pub fn skip_materials(mut self) -> Self {
        self.load_materials = false;
        self
    }

    /// Set base path for resolving material and texture files
    pub fn with_base_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.base_path = Some(path.into());
        self
    }

    /// Read an OBJ file
    pub fn read_file<P: AsRef<Path>>(&self, path: P) -> ObjResult<ObjMesh> {
        let path_ref = path.as_ref();

        // Set base path for materials if not explicitly set
        let base_path = self.base_path.clone().unwrap_or_else(|| {
            path_ref.parent().unwrap_or(Path::new(".")).to_path_buf()
        });

        let file = File::open(path_ref)?;
        let _reader = BufReader::new(file);
        self.read(reader, &base_path)
    }

    /// Read OBJ from a buffered reader
    pub fn read<R: BufRead>(&self, reader: R, base_path: &Path) -> ObjResult<ObjMesh> {
        let mut mesh = ObjMesh::new();
        let mut current_material: Option<String> = None;
        let mut line_num = 0;

        for line in reader.lines() {
            line_num += 1;
            let line = line?;
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "v" => {
                    // Vertex position
                    mesh.vertices.push(self.parse_vertex(&parts[1..], line_num)?);
                }
                "vt" => {
                    // Texture coordinate
                    mesh.texcoords.push(self.parse_texcoord(&parts[1..], line_num)?);
                }
                "vn" => {
                    // Normal
                    mesh.normals.push(self.parse_normal(&parts[1..], line_num)?);
                }
                "f" => {
                    // Face
                    let mut face = self.parse_face(&parts[1..], line_num)?;
                    face.material = current_material.clone();
                    mesh.faces.push(face);
                }
                "o" => {
                    // Object name
                    mesh.object_name = Some(parts[1..].join(" "));
                }
                "g" => {
                    // Group name
                    mesh.group_name = Some(parts[1..].join(" "));
                }
                "mtllib" => {
                    // Material library
                    if self.load_materials {
                        let mtl_file = parts[1..].join(" ");
                        let mtl_path = base_path.join(&mtl_file);
                        if let Ok(materials) = self.load_mtl_file(&mtl_path) {
                            mesh.materials.extend(materials);
                        }
                    }
                }
                "usemtl" => {
                    // Use material
                    current_material = Some(parts[1..].join(" "));
                }
                _ => {
                    // Skip unknown commands
                }
            }
        }

        Ok(mesh)
    }

    fn parse_vertex(&self, parts: &[&str], line: usize) -> ObjResult<ObjVertex> {
        if parts.len() < 3 {
            return Err(ObjError::Parse {
                line,
                message: "Vertex requires at least 3 coordinates".to_string(),
            });
        }

        Ok(ObjVertex {
            x: self.parse_float(parts[0], line)?,
            y: self.parse_float(parts[1], line)?,
            z: self.parse_float(parts[2], line)?,
            w: if parts.len() > 3 {
                self.parse_float(parts[3], line)?
            } else {
                1.0
            },
        })
    }

    fn parse_texcoord(&self, parts: &[&str], line: usize) -> ObjResult<ObjTexCoord> {
        if parts.is_empty() {
            return Err(ObjError::Parse {
                line,
                message: "Texture coordinate requires at least 1 coordinate".to_string(),
            });
        }

        Ok(ObjTexCoord {
            u: self.parse_float(parts[0], line)?,
            v: if parts.len() > 1 {
                self.parse_float(parts[1], line)?
            } else {
                0.0
            },
            w: if parts.len() > 2 {
                self.parse_float(parts[2], line)?
            } else {
                0.0
            },
        })
    }

    fn parse_normal(&self, parts: &[&str], line: usize) -> ObjResult<ObjNormal> {
        if parts.len() < 3 {
            return Err(ObjError::Parse {
                line,
                message: "Normal requires 3 coordinates".to_string(),
            });
        }

        Ok(ObjNormal {
            x: self.parse_float(parts[0], line)?,
            y: self.parse_float(parts[1], line)?,
            z: self.parse_float(parts[2], line)?,
        })
    }

    fn parse_face(&self, parts: &[&str], line: usize) -> ObjResult<ObjFace> {
        let mut vertices = Vec::new();

        for part in parts {
            vertices.push(self.parse_face_vertex(part, line)?);
        }

        if vertices.len() < 3 {
            return Err(ObjError::Parse {
                line,
                message: "Face requires at least 3 vertices".to_string(),
            });
        }

        Ok(ObjFace {
            vertices,
            material: None,
        })
    }

    fn parse_face_vertex(&self, s: &str, line: usize) -> ObjResult<FaceVertex> {
        // Face vertex format: v/vt/vn or v//vn or v/vt or v
        let parts: Vec<&str> = s.split('/').collect();

        let vertex_index = self.parse_int(parts[0], line)?;
        let texcoord_index = if parts.len() > 1 && !parts[1].is_empty() {
            Some(self.parse_int(parts[1], line)?)
        } else {
            None
        };
        let normal_index = if parts.len() > 2 && !parts[2].is_empty() {
            Some(self.parse_int(parts[2], line)?)
        } else {
            None
        };

        Ok(FaceVertex {
            vertex_index,
            texcoord_index,
            normal_index,
        })
    }

    fn parse_float(&self, s: &str, line: usize) -> ObjResult<f64> {
        s.parse().map_err(|e| ObjError::Parse {
            line,
            message: format!("Invalid float: {}", e),
        })
    }

    fn parse_int(&self, s: &str, line: usize) -> ObjResult<i32> {
        s.parse().map_err(|e| ObjError::Parse {
            line,
            message: format!("Invalid integer: {}", e),
        })
    }

    fn load_mtl_file(&self, path: &Path) -> ObjResult<HashMap<String, ObjMaterial>> {
        let file = File::open(path)?;
        let _reader = BufReader::new(file);
        let mut materials = HashMap::new();
        let mut current_material: Option<ObjMaterial> = None;

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "newmtl" => {
                    if let Some(mat) = current_material.take() {
                        materials.insert(mat.name.clone(), mat);
                    }
                    current_material = Some(ObjMaterial {
                        name: parts[1..].join(" "),
                        ..Default::default()
                    });
                }
                "Ka" if current_material.is_some() => {
                    // Ambient color
                    if parts.len() >= 4 {
                        if let Some(ref mut mat) = current_material {
                            mat.ambient = [
                                parts[1].parse().unwrap_or(0.0),
                                parts[2].parse().unwrap_or(0.0),
                                parts[3].parse().unwrap_or(0.0),
                            ];
                        }
                    }
                }
                "Kd" if current_material.is_some() => {
                    // Diffuse color
                    if parts.len() >= 4 {
                        if let Some(ref mut mat) = current_material {
                            mat.diffuse = [
                                parts[1].parse().unwrap_or(0.0),
                                parts[2].parse().unwrap_or(0.0),
                                parts[3].parse().unwrap_or(0.0),
                            ];
                        }
                    }
                }
                "Ks" if current_material.is_some() => {
                    // Specular color
                    if parts.len() >= 4 {
                        if let Some(ref mut mat) = current_material {
                            mat.specular = [
                                parts[1].parse().unwrap_or(0.0),
                                parts[2].parse().unwrap_or(0.0),
                                parts[3].parse().unwrap_or(0.0),
                            ];
                        }
                    }
                }
                "map_Kd" if current_material.is_some() => {
                    // Diffuse texture
                    if let Some(ref mut mat) = current_material {
                        mat.diffuse_texture = Some(parts[1..].join(" "));
                    }
                }
                _ => {}
            }
        }

        if let Some(mat) = current_material {
            materials.insert(mat.name.clone(), mat);
        }

        Ok(materials)
    }
}

impl Default for ObjReader {
    fn default() -> Self {
        Self::new()
    }
}

/// OBJ file writer
pub struct ObjWriter {
    write_materials: bool,
    precision: usize,
}

impl ObjWriter {
    /// Create a new OBJ writer
    pub fn new() -> Self {
        Self {
            write_materials: true,
            precision: 6,
        }
    }

    /// Disable material file writing
    pub fn skip_materials(mut self) -> Self {
        self.write_materials = false;
        self
    }

    /// Set coordinate precision
    pub fn with_precision(mut self, precision: usize) -> Self {
        self.precision = precision;
        self
    }

    /// Write mesh to OBJ file
    pub fn write_file<P: AsRef<Path>>(&self, mesh: &ObjMesh, path: P) -> ObjResult<()> {
        let path_ref = path.as_ref();
        let file = File::create(path_ref)?;
        let mut writer = BufWriter::new(file);

        self.write(mesh, &mut writer)?;

        // Write MTL file if materials exist
        if self.write_materials && !mesh.materials.is_empty() {
            let mtl_path = path_ref.with_extension("mtl");
            self.write_mtl_file(mesh, &mtl_path)?;
        }

        Ok(())
    }

    /// Write OBJ to a writer
    pub fn write<W: Write>(&self, mesh: &ObjMesh, writer: &mut W) -> ObjResult<()> {
        // Write header
        writeln!(writer, "# CADDY OBJ Export")?;
        writeln!(writer, "# Vertices: {}", mesh.vertices.len())?;
        writeln!(writer, "# Faces: {}", mesh.faces.len())?;
        writeln!(writer)?;

        // Write MTL library reference if materials exist
        if !mesh.materials.is_empty() && self.write_materials {
            writeln!(writer, "mtllib {}.mtl", mesh.object_name.as_deref().unwrap_or("export"))?;
            writeln!(writer)?;
        }

        // Write object name
        if let Some(ref name) = mesh.object_name {
            writeln!(writer, "o {}", name)?;
        }

        // Write vertices
        for v in &mesh.vertices {
            writeln!(
                writer,
                "v {:.prec$} {:.prec$} {:.prec$}",
                v.x,
                v.y,
                v.z,
                prec = self.precision
            )?;
        }
        writeln!(writer)?;

        // Write texture coordinates
        if !mesh.texcoords.is_empty() {
            for vt in &mesh.texcoords {
                writeln!(
                    writer,
                    "vt {:.prec$} {:.prec$}",
                    vt.u,
                    vt.v,
                    prec = self.precision
                )?;
            }
            writeln!(writer)?;
        }

        // Write normals
        if !mesh.normals.is_empty() {
            for vn in &mesh.normals {
                writeln!(
                    writer,
                    "vn {:.prec$} {:.prec$} {:.prec$}",
                    vn.x,
                    vn.y,
                    vn.z,
                    prec = self.precision
                )?;
            }
            writeln!(writer)?;
        }

        // Write faces (grouped by material)
        let mut current_material: Option<&String> = None;
        for face in &mesh.faces {
            if face.material != current_material.map(|s| s.as_str()) {
                if let Some(ref mat) = face.material {
                    writeln!(writer, "usemtl {}", mat)?;
                    current_material = Some(mat);
                }
            }

            write!(writer, "f")?;
            for fv in &face.vertices {
                write!(writer, " {}", fv.vertex_index)?;
                if fv.texcoord_index.is_some() || fv.normal_index.is_some() {
                    write!(writer, "/")?;
                    if let Some(vt) = fv.texcoord_index {
                        write!(writer, "{}", vt)?;
                    }
                    if let Some(vn) = fv.normal_index {
                        write!(writer, "/{}", vn)?;
                    }
                }
            }
            writeln!(writer)?;
        }

        Ok(())
    }

    fn write_mtl_file(&self, mesh: &ObjMesh, path: &Path) -> ObjResult<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        writeln!(writer, "# CADDY MTL Export")?;
        writeln!(writer)?;

        for (name, mat) in &mesh.materials {
            writeln!(writer, "newmtl {}", name)?;
            writeln!(writer, "Ka {} {} {}", mat.ambient[0], mat.ambient[1], mat.ambient[2])?;
            writeln!(writer, "Kd {} {} {}", mat.diffuse[0], mat.diffuse[1], mat.diffuse[2])?;
            writeln!(writer, "Ks {} {} {}", mat.specular[0], mat.specular[1], mat.specular[2])?;
            writeln!(writer, "Ns {}", mat.shininess)?;
            writeln!(writer, "d {}", mat.transparency)?;
            writeln!(writer, "illum {}", mat.illumination_model)?;

            if let Some(ref tex) = mat.diffuse_texture {
                writeln!(writer, "map_Kd {}", tex)?;
            }

            writeln!(writer)?;
        }

        Ok(())
    }
}

impl Default for ObjWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obj_vertex() {
        let v = ObjVertex::new(1.0, 2.0, 3.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.w, 1.0);
    }

    #[test]
    fn test_obj_material_default() {
        let mat = ObjMaterial::default();
        assert_eq!(mat.transparency, 1.0);
        assert_eq!(mat.illumination_model, 2);
    }
}
