// CADDY - Enterprise CAD System
// File I/O System - glTF 2.0 Format Support
// Agent 9 - Import/Export Pipeline Specialist

//! # glTF (GL Transmission Format) 2.0 Support
//!
//! Provides read/write support for glTF 2.0, the modern standard for 3D asset
//! transmission. glTF is optimized for runtime rendering in web and AR/VR applications.
//!
//! ## Format Variants
//!
//! - **glTF (.gltf)**: JSON format with separate binary and image files
//! - **GLB (.glb)**: Binary format with embedded resources
//!
//! ## Features
//!
//! - PBR materials (Physically Based Rendering)
//! - Animations and skinning
//! - Multiple scenes and cameras
//! - Extensibility through extensions
//! - Efficient binary buffers
//! - Texture embedding

use crate::io::document::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;
use thiserror::Error;

/// glTF-related errors
#[derive(Error, Debug)]
pub enum GltfError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid glTF file: {0}")]
    InvalidFile(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Unsupported glTF version: {0}")]
    UnsupportedVersion(String),

    #[error("Invalid buffer reference: {0}")]
    InvalidBufferRef(usize),

    #[error("Unsupported extension: {0}")]
    UnsupportedExtension(String),
}

pub type GltfResult<T> = Result<T, GltfError>;

/// glTF root structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Gltf {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessors: Option<Vec<Accessor>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub animations: Option<Vec<Animation>>,

    pub asset: Asset,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub buffers: Option<Vec<Buffer>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub buffer_views: Option<Vec<BufferView>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cameras: Option<Vec<Camera>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<Image>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub materials: Option<Vec<Material>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub meshes: Option<Vec<Mesh>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub nodes: Option<Vec<Node>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub samplers: Option<Vec<Sampler>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub scene: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub scenes: Option<Vec<Scene>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skins: Option<Vec<Skin>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub textures: Option<Vec<Texture>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<HashMap<String, serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}

/// glTF asset metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copyright: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub generator: Option<String>,

    pub version: String,

    #[serde(skip_serializing_if = "Option::is_none", rename = "minVersion")]
    pub min_version: Option<String>,
}

/// glTF buffer
#[derive(Debug, Serialize, Deserialize)]
pub struct Buffer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,

    #[serde(rename = "byteLength")]
    pub byte_length: usize,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// glTF buffer view
#[derive(Debug, Serialize, Deserialize)]
pub struct BufferView {
    pub buffer: usize,

    #[serde(skip_serializing_if = "Option::is_none", rename = "byteOffset")]
    pub byte_offset: Option<usize>,

    #[serde(rename = "byteLength")]
    pub byte_length: usize,

    #[serde(skip_serializing_if = "Option::is_none", rename = "byteStride")]
    pub byte_stride: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// glTF accessor
#[derive(Debug, Serialize, Deserialize)]
pub struct Accessor {
    #[serde(skip_serializing_if = "Option::is_none", rename = "bufferView")]
    pub buffer_view: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "byteOffset")]
    pub byte_offset: Option<usize>,

    #[serde(rename = "componentType")]
    pub component_type: u32,

    pub count: usize,

    #[serde(rename = "type")]
    pub accessor_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<Vec<f64>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<Vec<f64>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// glTF mesh
#[derive(Debug, Serialize, Deserialize)]
pub struct Mesh {
    pub primitives: Vec<Primitive>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub weights: Option<Vec<f64>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// glTF mesh primitive
#[derive(Debug, Serialize, Deserialize)]
pub struct Primitive {
    pub attributes: HashMap<String, usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub indices: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub material: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<u32>,
}

/// glTF material
#[derive(Debug, Serialize, Deserialize)]
pub struct Material {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "pbrMetallicRoughness")]
    pub pbr_metallic_roughness: Option<PbrMetallicRoughness>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "normalTexture")]
    pub normal_texture: Option<NormalTextureInfo>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "emissiveTexture")]
    pub emissive_texture: Option<TextureInfo>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "emissiveFactor")]
    pub emissive_factor: Option<[f64; 3]>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "alphaMode")]
    pub alpha_mode: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "alphaCutoff")]
    pub alpha_cutoff: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "doubleSided")]
    pub double_sided: Option<bool>,
}

/// PBR metallic roughness
#[derive(Debug, Serialize, Deserialize)]
pub struct PbrMetallicRoughness {
    #[serde(skip_serializing_if = "Option::is_none", rename = "baseColorFactor")]
    pub base_color_factor: Option<[f64; 4]>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "baseColorTexture")]
    pub base_color_texture: Option<TextureInfo>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "metallicFactor")]
    pub metallic_factor: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "roughnessFactor")]
    pub roughness_factor: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "metallicRoughnessTexture")]
    pub metallic_roughness_texture: Option<TextureInfo>,
}

/// Texture info
#[derive(Debug, Serialize, Deserialize)]
pub struct TextureInfo {
    pub index: usize,

    #[serde(skip_serializing_if = "Option::is_none", rename = "texCoord")]
    pub tex_coord: Option<usize>,
}

/// Normal texture info
#[derive(Debug, Serialize, Deserialize)]
pub struct NormalTextureInfo {
    pub index: usize,

    #[serde(skip_serializing_if = "Option::is_none", rename = "texCoord")]
    pub tex_coord: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<f64>,
}

/// glTF node
#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub camera: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<usize>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skin: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub matrix: Option<[f64; 16]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mesh: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<[f64; 4]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<[f64; 3]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation: Option<[f64; 3]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// glTF scene
#[derive(Debug, Serialize, Deserialize)]
pub struct Scene {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nodes: Option<Vec<usize>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// glTF texture
#[derive(Debug, Serialize, Deserialize)]
pub struct Texture {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampler: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// glTF image
#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "mimeType")]
    pub mime_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "bufferView")]
    pub buffer_view: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// glTF sampler
#[derive(Debug, Serialize, Deserialize)]
pub struct Sampler {
    #[serde(skip_serializing_if = "Option::is_none", rename = "magFilter")]
    pub mag_filter: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "minFilter")]
    pub min_filter: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "wrapS")]
    pub wrap_s: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "wrapT")]
    pub wrap_t: Option<u32>,
}

/// glTF camera
#[derive(Debug, Serialize, Deserialize)]
pub struct Camera {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orthographic: Option<OrthographicCamera>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub perspective: Option<PerspectiveCamera>,

    #[serde(rename = "type")]
    pub camera_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrthographicCamera {
    pub xmag: f64,
    pub ymag: f64,
    pub zfar: f64,
    pub znear: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerspectiveCamera {
    #[serde(skip_serializing_if = "Option::is_none", rename = "aspectRatio")]
    pub aspect_ratio: Option<f64>,

    pub yfov: f64,
    pub zfar: Option<f64>,
    pub znear: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Animation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channels: Option<Vec<AnimationChannel>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub samplers: Option<Vec<AnimationSampler>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnimationChannel {
    pub sampler: usize,
    pub target: AnimationTarget,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnimationTarget {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node: Option<usize>,

    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnimationSampler {
    pub input: usize,
    pub output: usize,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub interpolation: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Skin {
    #[serde(skip_serializing_if = "Option::is_none", rename = "inverseBindMatrices")]
    pub inverse_bind_matrices: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skeleton: Option<usize>,

    pub joints: Vec<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// glTF file reader
pub struct GltfReader {
    load_buffers: bool,
}

impl GltfReader {
    /// Create a new glTF reader
    pub fn new() -> Self {
        Self {
            load_buffers: true,
        }
    }

    /// Skip loading external buffer files
    pub fn skip_buffers(mut self) -> Self {
        self.load_buffers = false;
        self
    }

    /// Read a glTF file
    pub fn read_file<P: AsRef<Path>>(&self, path: P) -> GltfResult<Gltf> {
        let file = File::open(path)?;
        let _reader = BufReader::new(file);
        self.read(reader)
    }

    /// Read glTF from a reader
    pub fn read<R: Read>(&self, reader: R) -> GltfResult<Gltf> {
        let gltf: Gltf = serde_json::from_reader(reader)?;

        // Validate version
        if gltf.asset.version != "2.0" {
            return Err(GltfError::UnsupportedVersion(gltf.asset.version.clone()));
        }

        Ok(gltf)
    }

    /// Convert glTF to CADDY document
    pub fn to_document(&self, gltf: &Gltf) -> GltfResult<Document> {
        let mut doc = Document::new();

        // Set metadata
        if let Some(ref copyright) = gltf.asset.copyright {
            doc.metadata.title = copyright.clone();
        }

        // Convert meshes to entities
        if let Some(ref meshes) = gltf.meshes {
            for mesh in meshes {
                // Convert mesh primitives to CAD entities
                // This is a placeholder implementation
            }
        }

        Ok(doc)
    }
}

impl Default for GltfReader {
    fn default() -> Self {
        Self::new()
    }
}

/// glTF file writer
pub struct GltfWriter {
    binary_format: bool,
    pretty_print: bool,
}

impl GltfWriter {
    /// Create a new glTF writer
    pub fn new() -> Self {
        Self {
            binary_format: false,
            pretty_print: true,
        }
    }

    /// Use binary GLB format
    pub fn binary(mut self) -> Self {
        self.binary_format = true;
        self
    }

    /// Disable pretty printing
    pub fn compact(mut self) -> Self {
        self.pretty_print = false;
        self
    }

    /// Write glTF to file
    pub fn write_file<P: AsRef<Path>>(&self, gltf: &Gltf, path: P) -> GltfResult<()> {
        let file = File::create(path)?;
        let _writer = BufWriter::new(file);
        self.write(gltf, writer)
    }

    /// Write glTF to a writer
    pub fn write<W: Write>(&self, gltf: &Gltf, writer: W) -> GltfResult<()> {
        if self.pretty_print {
            serde_json::to_writer_pretty(writer, gltf)?;
        } else {
            serde_json::to_writer(writer, gltf)?;
        }
        Ok(())
    }

    /// Convert CADDY document to glTF
    pub fn from_document(&self, doc: &Document) -> GltfResult<Gltf> {
        let gltf = Gltf {
            asset: Asset {
                generator: Some("CADDY v0.2.5".to_string()),
                version: "2.0".to_string(),
                copyright: Some(doc.metadata.title.clone()),
                min_version: None,
            },
            accessors: None,
            animations: None,
            buffers: None,
            buffer_views: None,
            cameras: None,
            images: None,
            materials: None,
            meshes: None,
            nodes: None,
            samplers: None,
            scene: Some(0),
            scenes: Some(vec![Scene {
                nodes: Some(Vec::new()),
                name: Some("Main Scene".to_string()),
            }]),
            skins: None,
            textures: None,
            extensions: None,
            extras: None,
        };

        Ok(gltf)
    }
}

impl Default for GltfWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gltf_reader_creation() {
        let _reader = GltfReader::new();
        assert!(reader.load_buffers);
    }

    #[test]
    fn test_gltf_writer_creation() {
        let _writer = GltfWriter::new();
        assert!(writer.pretty_print);
        assert!(!writer.binary_format);
    }

    #[test]
    fn test_asset_version() {
        let asset = Asset {
            version: "2.0".to_string(),
            generator: Some("CADDY".to_string()),
            copyright: None,
            min_version: None,
        };
        assert_eq!(asset.version, "2.0");
    }
}
