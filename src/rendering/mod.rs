//! GPU-accelerated rendering engine for CADDY
//!
//! This module provides a complete rendering system built on wgpu for cross-platform
//! GPU acceleration. It handles multi-viewport rendering, camera management, and
//! efficient rendering of CAD entities.

pub mod renderer;
pub mod camera;
pub mod viewport;
pub mod pipeline;
pub mod shaders;
pub mod buffers;

// Re-export main types
pub use renderer::{Renderer, RenderContext, RenderMode};
pub use camera::{Camera, ProjectionType, CameraController, NamedView};
pub use viewport::{Viewport, ViewportLayout, ViewportConfig};
pub use pipeline::{LinePipeline, MeshPipeline, PointPipeline, TextPipeline, PipelineCache};
pub use buffers::{VertexBuffer, IndexBuffer, UniformBuffer, DynamicBuffer};

use thiserror::Error;

/// Rendering errors
#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Failed to initialize renderer: {0}")]
    InitializationError(String),

    #[error("Failed to create adapter: no suitable GPU found")]
    NoAdapter,

    #[error("Failed to request device: {0}")]
    DeviceRequest(String),

    #[error("Failed to create surface: {0}")]
    SurfaceCreation(String),

    #[error("Failed to create pipeline: {0}")]
    PipelineCreation(String),

    #[error("Failed to create buffer: {0}")]
    BufferCreation(String),

    #[error("Failed to create shader: {0}")]
    ShaderCreation(String),

    #[error("Failed to render: {0}")]
    RenderFailure(String),

    #[error("Viewport error: {0}")]
    ViewportError(String),
}

pub type RenderResult<T> = Result<T, RenderError>;

/// Vertex format for line rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LineVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub thickness: f32,
    pub _padding: [f32; 3],
}

impl LineVertex {
    pub fn new(position: [f32; 3], color: [f32; 4], thickness: f32) -> Self {
        Self {
            position,
            color,
            thickness,
            _padding: [0.0; 3],
        }
    }

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<LineVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

/// Vertex format for mesh rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 4],
    pub uv: [f32; 2],
}

impl MeshVertex {
    pub fn new(position: [f32; 3], normal: [f32; 3], color: [f32; 4], uv: [f32; 2]) -> Self {
        Self {
            position,
            normal,
            color,
            uv,
        }
    }

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<MeshVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 10]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

/// Uniform data for transformations
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformUniforms {
    pub view_proj: [[f32; 4]; 4],
    pub model: [[f32; 4]; 4],
    pub normal_matrix: [[f32; 4]; 4],
}

impl TransformUniforms {
    pub fn new() -> Self {
        Self {
            view_proj: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            normal_matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

impl Default for TransformUniforms {
    fn default() -> Self {
        Self::new()
    }
}

/// Lighting uniforms
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniforms {
    pub light_position: [f32; 3],
    pub _padding1: f32,
    pub light_color: [f32; 3],
    pub _padding2: f32,
    pub ambient_strength: f32,
    pub specular_strength: f32,
    pub shininess: f32,
    pub _padding3: f32,
}

impl Default for LightUniforms {
    fn default() -> Self {
        Self {
            light_position: [10.0, 10.0, 10.0],
            _padding1: 0.0,
            light_color: [1.0, 1.0, 1.0],
            _padding2: 0.0,
            ambient_strength: 0.3,
            specular_strength: 0.5,
            shininess: 32.0,
            _padding3: 0.0,
        }
    }
}
