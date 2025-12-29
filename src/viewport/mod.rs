//! # Viewport Rendering System
//!
//! Advanced viewport rendering system for CADDY with GPU acceleration,
//! multi-viewport support, and enterprise-grade performance optimization.
//!
//! ## Features
//!
//! - GPU-accelerated 2D/3D rendering with WebGPU
//! - Multiple camera modes (orthographic, perspective, isometric)
//! - Frustum culling and occlusion optimization
//! - Level-of-detail (LOD) management
//! - Custom shader pipeline
//! - Multi-viewport orchestration
//!
//! ## Architecture
//!
//! The viewport system is designed for maximum performance and flexibility:
//!
//! - `renderer`: Core GPU rendering engine
//! - `camera`: Camera management and transformations
//! - `culling`: Visibility determination and optimization
//! - `lod`: Automatic level-of-detail selection
//! - `shaders`: Shader compilation and management

pub mod camera;
pub mod culling;
pub mod lod;
pub mod renderer;
pub mod shaders;

// Re-export key types for convenience
pub use camera::{Camera, CameraMode, CameraProjection, ViewportCamera};
pub use culling::{CullingResult, FrustumCuller, OcclusionCuller, VisibilityTester};
pub use lod::{LodLevel, LodManager, LodMetrics, LodStrategy};
pub use renderer::{
    RenderContext, RenderOptions, RenderStatistics, ViewportRenderer, WebGpuBackend,
};
pub use shaders::{ShaderCompiler, ShaderModule, ShaderPipeline, ShaderSource};

use crate::core::math::Vector2;
use crate::core::primitives::Point2;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

/// Viewport system errors
#[derive(Debug, Error)]
pub enum ViewportError {
    /// Rendering initialization failed
    #[error("Failed to initialize renderer: {0}")]
    RendererInitFailed(String),

    /// Shader compilation failed
    #[error("Shader compilation failed: {0}")]
    ShaderCompilationFailed(String),

    /// GPU device error
    #[error("GPU device error: {0}")]
    GpuDeviceError(String),

    /// Invalid camera configuration
    #[error("Invalid camera configuration: {0}")]
    InvalidCameraConfig(String),

    /// Resource not found
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    /// Invalid viewport dimensions
    #[error("Invalid viewport dimensions: {0}x{1}")]
    InvalidDimensions(u32, u32),

    /// WebGPU backend error
    #[error("WebGPU backend error: {0}")]
    WebGpuError(String),

    /// LOD system error
    #[error("LOD system error: {0}")]
    LodError(String),
}

/// Result type for viewport operations
pub type ViewportResult<T> = Result<T, ViewportError>;

/// Viewport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportConfig {
    /// Viewport width in pixels
    pub width: u32,

    /// Viewport height in pixels
    pub height: u32,

    /// Background color (RGBA)
    pub background_color: [f32; 4],

    /// Enable multi-sampling anti-aliasing
    pub msaa_samples: u32,

    /// Enable vertical sync
    pub vsync: bool,

    /// Maximum frames per second (0 = unlimited)
    pub max_fps: u32,

    /// Enable frustum culling
    pub enable_culling: bool,

    /// Enable occlusion culling
    pub enable_occlusion: bool,

    /// Enable level-of-detail
    pub enable_lod: bool,

    /// Grid size (0 = no grid)
    pub grid_size: f32,

    /// Grid subdivision
    pub grid_subdivisions: u32,

    /// Show axis indicator
    pub show_axis: bool,

    /// Show performance statistics
    pub show_stats: bool,
}

impl Default for ViewportConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            background_color: [0.15, 0.15, 0.18, 1.0],
            msaa_samples: 4,
            vsync: true,
            max_fps: 60,
            enable_culling: true,
            enable_occlusion: true,
            enable_lod: true,
            grid_size: 1.0,
            grid_subdivisions: 10,
            show_axis: true,
            show_stats: false,
        }
    }
}

/// Viewport identification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ViewportId(pub u32);

impl ViewportId {
    /// Create a new viewport ID
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the raw ID value
    pub fn value(&self) -> u32 {
        self.0
    }
}

/// Viewport state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportState {
    /// Viewport identifier
    pub id: ViewportId,

    /// Current configuration
    pub config: ViewportConfig,

    /// Camera state
    pub camera: ViewportCamera,

    /// Is viewport active
    pub active: bool,

    /// Is viewport visible
    pub visible: bool,

    /// Viewport position (for multi-viewport layouts)
    pub position: Point2,

    /// Viewport size
    pub size: Vector2,
}

impl ViewportState {
    /// Create a new viewport state
    pub fn new(id: ViewportId, config: ViewportConfig) -> Self {
        Self {
            id,
            config: config.clone(),
            camera: ViewportCamera::new(
                CameraMode::Perspective,
                config.width as f32 / config.height as f32,
            ),
            active: true,
            visible: true,
            position: Point2::new(0.0, 0.0),
            size: Vector2::new(config.width as f32, config.height as f32),
        }
    }

    /// Update viewport dimensions
    pub fn resize(&mut self, width: u32, height: u32) -> ViewportResult<()> {
        if width == 0 || height == 0 {
            return Err(ViewportError::InvalidDimensions(width, height));
        }

        self.config.width = width;
        self.config.height = height;
        self.size = Vector2::new(width as f32, height as f32);
        self.camera.set_aspect_ratio(width as f32 / height as f32);

        Ok(())
    }

    /// Get aspect ratio
    pub fn aspect_ratio(&self) -> f32 {
        self.config.width as f32 / self.config.height as f32
    }
}

/// Viewport manager for orchestrating multiple viewports
pub struct ViewportManager {
    viewports: Vec<ViewportState>,
    active_viewport: Option<ViewportId>,
    renderer: Arc<ViewportRenderer>,
}

impl ViewportManager {
    /// Create a new viewport manager
    pub fn new(renderer: Arc<ViewportRenderer>) -> Self {
        Self {
            viewports: Vec::new(),
            active_viewport: None,
            renderer,
        }
    }

    /// Add a new viewport
    pub fn add_viewport(&mut self, config: ViewportConfig) -> ViewportId {
        let id = ViewportId::new(self.viewports.len() as u32);
        let state = ViewportState::new(id, config);
        self.viewports.push(state);

        if self.active_viewport.is_none() {
            self.active_viewport = Some(id);
        }

        id
    }

    /// Remove a viewport
    pub fn remove_viewport(&mut self, id: ViewportId) -> ViewportResult<()> {
        let index = self.viewports
            .iter()
            .position(|v| v.id == id)
            .ok_or_else(|| ViewportError::ResourceNotFound(format!("Viewport {}", id.0)))?;

        self.viewports.remove(index);

        if self.active_viewport == Some(id) {
            self.active_viewport = self.viewports.first().map(|v| v.id);
        }

        Ok(())
    }

    /// Get viewport state
    pub fn get_viewport(&self, id: ViewportId) -> Option<&ViewportState> {
        self.viewports.iter().find(|v| v.id == id)
    }

    /// Get mutable viewport state
    pub fn get_viewport_mut(&mut self, id: ViewportId) -> Option<&mut ViewportState> {
        self.viewports.iter_mut().find(|v| v.id == id)
    }

    /// Set active viewport
    pub fn set_active_viewport(&mut self, id: ViewportId) -> ViewportResult<()> {
        if !self.viewports.iter().any(|v| v.id == id) {
            return Err(ViewportError::ResourceNotFound(format!("Viewport {}", id.0)));
        }

        self.active_viewport = Some(id);
        Ok(())
    }

    /// Get active viewport
    pub fn active_viewport(&self) -> Option<&ViewportState> {
        self.active_viewport.and_then(|id| self.get_viewport(id))
    }

    /// Get all viewports
    pub fn viewports(&self) -> &[ViewportState] {
        &self.viewports
    }

    /// Get renderer
    pub fn renderer(&self) -> &Arc<ViewportRenderer> {
        &self.renderer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_config_default() {
        let config = ViewportConfig::default();
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
        assert!(config.enable_culling);
        assert!(config.enable_lod);
    }

    #[test]
    fn test_viewport_state_creation() {
        let id = ViewportId::new(0);
        let config = ViewportConfig::default();
        let state = ViewportState::new(id, config);

        assert_eq!(state.id, id);
        assert!(state.active);
        assert!(state.visible);
    }

    #[test]
    fn test_viewport_resize() {
        let id = ViewportId::new(0);
        let config = ViewportConfig::default();
        let mut state = ViewportState::new(id, config);

        assert!(state.resize(1280, 720).is_ok());
        assert_eq!(state.config.width, 1280);
        assert_eq!(state.config.height, 720);

        assert!(state.resize(0, 720).is_err());
        assert!(state.resize(1280, 0).is_err());
    }

    #[test]
    fn test_viewport_aspect_ratio() {
        let id = ViewportId::new(0);
        let config = ViewportConfig::default();
        let state = ViewportState::new(id, config);

        assert!((state.aspect_ratio() - 16.0 / 9.0).abs() < 0.01);
    }
}
