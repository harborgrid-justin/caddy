//! # Shader Pipeline Management
//!
//! Custom shader compilation, caching, and pipeline management for the viewport renderer.
//!
//! ## Features
//!
//! - WGSL shader compilation and validation
//! - Shader hot-reloading for development
//! - Pipeline state caching
//! - Uniform buffer management
//! - Multi-pass rendering support

use crate::viewport::{ViewportError, ViewportResult};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Shader language type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShaderLanguage {
    /// WebGPU Shading Language
    WGSL,

    /// SPIR-V bytecode
    SPIRV,

    /// GLSL (converted to WGSL)
    GLSL,
}

/// Shader stage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShaderStage {
    /// Vertex shader
    Vertex,

    /// Fragment/Pixel shader
    Fragment,

    /// Compute shader
    Compute,
}

/// Shader source code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderSource {
    /// Shader language
    pub language: ShaderLanguage,

    /// Shader stage
    pub stage: ShaderStage,

    /// Source code
    pub code: String,

    /// Entry point function name
    pub entry_point: String,

    /// Source file path (for reloading)
    pub source_path: Option<PathBuf>,
}

impl ShaderSource {
    /// Create a new shader source
    pub fn new(language: ShaderLanguage, stage: ShaderStage, code: String) -> Self {
        let entry_point = match stage {
            ShaderStage::Vertex => "vs_main",
            ShaderStage::Fragment => "fs_main",
            ShaderStage::Compute => "cs_main",
        }
        .to_string();

        Self {
            language,
            stage,
            code,
            entry_point,
            source_path: None,
        }
    }

    /// Create from file
    pub fn from_file(path: &Path, language: ShaderLanguage, stage: ShaderStage) -> ViewportResult<Self> {
        let code = std::fs::read_to_string(path).map_err(|e| {
            ViewportError::ShaderCompilationFailed(format!("Failed to read shader file: {}", e))
        })?;

        let mut source = Self::new(language, stage, code);
        source.source_path = Some(path.to_path_buf());

        Ok(source)
    }

    /// Reload from file
    pub fn reload(&mut self) -> ViewportResult<()> {
        if let Some(path) = &self.source_path {
            self.code = std::fs::read_to_string(path).map_err(|e| {
                ViewportError::ShaderCompilationFailed(format!("Failed to reload shader: {}", e))
            })?;
        }

        Ok(())
    }

    /// Validate shader source
    pub fn validate(&self) -> ViewportResult<()> {
        // Basic validation
        if self.code.is_empty() {
            return Err(ViewportError::ShaderCompilationFailed(
                "Shader source is empty".to_string(),
            ));
        }

        if !self.code.contains(&self.entry_point) {
            return Err(ViewportError::ShaderCompilationFailed(format!(
                "Entry point '{}' not found in shader",
                self.entry_point
            )));
        }

        match self.language {
            ShaderLanguage::WGSL => self.validate_wgsl(),
            ShaderLanguage::SPIRV => self.validate_spirv(),
            ShaderLanguage::GLSL => self.validate_glsl(),
        }
    }

    /// Validate WGSL shader
    fn validate_wgsl(&self) -> ViewportResult<()> {
        // Basic WGSL syntax checks
        match self.stage {
            ShaderStage::Vertex => {
                if !self.code.contains("@vertex") && !self.code.contains("[[stage(vertex)]]") {
                    return Err(ViewportError::ShaderCompilationFailed(
                        "Vertex shader missing @vertex attribute".to_string(),
                    ));
                }
            }
            ShaderStage::Fragment => {
                if !self.code.contains("@fragment") && !self.code.contains("[[stage(fragment)]]") {
                    return Err(ViewportError::ShaderCompilationFailed(
                        "Fragment shader missing @fragment attribute".to_string(),
                    ));
                }
            }
            ShaderStage::Compute => {
                if !self.code.contains("@compute") && !self.code.contains("[[stage(compute)]]") {
                    return Err(ViewportError::ShaderCompilationFailed(
                        "Compute shader missing @compute attribute".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Validate SPIR-V shader
    fn validate_spirv(&self) -> ViewportResult<()> {
        // SPIR-V is binary, basic length check
        if self.code.len() < 20 {
            return Err(ViewportError::ShaderCompilationFailed(
                "SPIR-V binary too short".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate GLSL shader
    fn validate_glsl(&self) -> ViewportResult<()> {
        // Basic GLSL syntax checks
        if !self.code.contains("void main()") && !self.code.contains(&format!("void {}()", self.entry_point)) {
            return Err(ViewportError::ShaderCompilationFailed(
                "GLSL shader missing main function".to_string(),
            ));
        }

        Ok(())
    }
}

/// Compiled shader module
pub struct ShaderModule {
    /// Module identifier
    pub id: String,

    /// Shader source
    pub source: ShaderSource,

    /// Compilation timestamp
    pub compiled_at: std::time::SystemTime,

    /// Is valid
    pub valid: bool,

    /// Compilation errors (if any)
    pub errors: Vec<String>,
}

impl ShaderModule {
    /// Create a new shader module
    pub fn new(id: String, source: ShaderSource) -> Self {
        Self {
            id,
            source,
            compiled_at: std::time::SystemTime::now(),
            valid: false,
            errors: Vec::new(),
        }
    }

    /// Compile the shader
    pub fn compile(&mut self) -> ViewportResult<()> {
        self.errors.clear();

        // Validate source
        if let Err(e) = self.source.validate() {
            self.valid = false;
            self.errors.push(e.to_string());
            return Err(e);
        }

        // In a real implementation, we would compile to SPIR-V or WGSL here
        // For now, we just mark as valid
        self.valid = true;
        self.compiled_at = std::time::SystemTime::now();

        Ok(())
    }

    /// Check if needs recompilation
    pub fn needs_recompilation(&self) -> bool {
        !self.valid
    }

    /// Get age since compilation
    pub fn age(&self) -> std::time::Duration {
        std::time::SystemTime::now()
            .duration_since(self.compiled_at)
            .unwrap_or_default()
    }
}

/// Shader pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    /// Vertex shader module ID
    pub vertex_shader: String,

    /// Fragment shader module ID
    pub fragment_shader: String,

    /// Primitive topology
    pub topology: PrimitiveTopology,

    /// Depth testing enabled
    pub depth_test: bool,

    /// Depth write enabled
    pub depth_write: bool,

    /// Blending enabled
    pub blending: bool,

    /// Culling mode
    pub cull_mode: CullMode,

    /// Polygon mode
    pub polygon_mode: PolygonMode,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            vertex_shader: "default_vs".to_string(),
            fragment_shader: "default_fs".to_string(),
            topology: PrimitiveTopology::TriangleList,
            depth_test: true,
            depth_write: true,
            blending: false,
            cull_mode: CullMode::Back,
            polygon_mode: PolygonMode::Fill,
        }
    }
}

/// Primitive topology
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimitiveTopology {
    /// Point list
    PointList,

    /// Line list
    LineList,

    /// Line strip
    LineStrip,

    /// Triangle list
    TriangleList,

    /// Triangle strip
    TriangleStrip,
}

/// Culling mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CullMode {
    /// No culling
    None,

    /// Cull front faces
    Front,

    /// Cull back faces
    Back,
}

/// Polygon rendering mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolygonMode {
    /// Fill polygons
    Fill,

    /// Draw lines
    Line,

    /// Draw points
    Point,
}

/// Render pipeline
pub struct ShaderPipeline {
    /// Pipeline identifier
    pub id: String,

    /// Configuration
    pub config: PipelineConfig,

    /// Vertex shader module
    pub vertex_module: Arc<RwLock<ShaderModule>>,

    /// Fragment shader module
    pub fragment_module: Arc<RwLock<ShaderModule>>,

    /// Is valid
    pub valid: bool,

    /// Creation timestamp
    pub created_at: std::time::SystemTime,
}

impl ShaderPipeline {
    /// Create a new shader pipeline
    pub fn new(
        id: String,
        config: PipelineConfig,
        vertex_module: Arc<RwLock<ShaderModule>>,
        fragment_module: Arc<RwLock<ShaderModule>>,
    ) -> Self {
        Self {
            id,
            config,
            vertex_module,
            fragment_module,
            valid: false,
            created_at: std::time::SystemTime::now(),
        }
    }

    /// Validate pipeline
    pub fn validate(&mut self) -> ViewportResult<()> {
        // Check if modules are valid
        let vertex_valid = self.vertex_module.read().valid;
        let fragment_valid = self.fragment_module.read().valid;

        if !vertex_valid {
            return Err(ViewportError::ShaderCompilationFailed(
                "Vertex shader module is invalid".to_string(),
            ));
        }

        if !fragment_valid {
            return Err(ViewportError::ShaderCompilationFailed(
                "Fragment shader module is invalid".to_string(),
            ));
        }

        self.valid = true;
        Ok(())
    }

    /// Check if pipeline needs rebuild
    pub fn needs_rebuild(&self) -> bool {
        !self.valid
            || self.vertex_module.read().needs_recompilation()
            || self.fragment_module.read().needs_recompilation()
    }
}

/// Shader compiler and cache
pub struct ShaderCompiler {
    /// Compiled shader modules
    modules: HashMap<String, Arc<RwLock<ShaderModule>>>,

    /// Active pipelines
    pipelines: HashMap<String, Arc<RwLock<ShaderPipeline>>>,

    /// Shader search paths
    search_paths: Vec<PathBuf>,

    /// Enable hot reloading
    hot_reload: bool,

    /// Compilation statistics
    stats: CompilationStats,
}

impl ShaderCompiler {
    /// Create a new shader compiler
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            pipelines: HashMap::new(),
            search_paths: Vec::new(),
            hot_reload: false,
            stats: CompilationStats::default(),
        }
    }

    /// Add shader search path
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    /// Enable hot reloading
    pub fn enable_hot_reload(&mut self, enable: bool) {
        self.hot_reload = enable;
    }

    /// Compile shader source
    pub fn compile_shader(
        &mut self,
        id: String,
        source: ShaderSource,
    ) -> ViewportResult<Arc<RwLock<ShaderModule>>> {
        self.stats.compilations_attempted += 1;

        let mut module = ShaderModule::new(id.clone(), source);

        match module.compile() {
            Ok(_) => {
                self.stats.compilations_succeeded += 1;
                let module = Arc::new(RwLock::new(module));
                self.modules.insert(id, module.clone());
                Ok(module)
            }
            Err(e) => {
                self.stats.compilations_failed += 1;
                Err(e)
            }
        }
    }

    /// Load shader from file
    pub fn load_shader(
        &mut self,
        id: String,
        path: &Path,
        language: ShaderLanguage,
        stage: ShaderStage,
    ) -> ViewportResult<Arc<RwLock<ShaderModule>>> {
        let source = ShaderSource::from_file(path, language, stage)?;
        self.compile_shader(id, source)
    }

    /// Create pipeline
    pub fn create_pipeline(
        &mut self,
        id: String,
        config: PipelineConfig,
    ) -> ViewportResult<Arc<RwLock<ShaderPipeline>>> {
        // Get shader modules
        let vertex_module = self
            .modules
            .get(&config.vertex_shader)
            .ok_or_else(|| {
                ViewportError::ResourceNotFound(format!(
                    "Vertex shader '{}' not found",
                    config.vertex_shader
                ))
            })?
            .clone();

        let fragment_module = self
            .modules
            .get(&config.fragment_shader)
            .ok_or_else(|| {
                ViewportError::ResourceNotFound(format!(
                    "Fragment shader '{}' not found",
                    config.fragment_shader
                ))
            })?
            .clone();

        let mut pipeline = ShaderPipeline::new(id.clone(), config, vertex_module, fragment_module);

        pipeline.validate()?;

        let pipeline = Arc::new(RwLock::new(pipeline));
        self.pipelines.insert(id, pipeline.clone());

        Ok(pipeline)
    }

    /// Get shader module
    pub fn get_module(&self, id: &str) -> Option<Arc<RwLock<ShaderModule>>> {
        self.modules.get(id).cloned()
    }

    /// Get pipeline
    pub fn get_pipeline(&self, id: &str) -> Option<Arc<RwLock<ShaderPipeline>>> {
        self.pipelines.get(id).cloned()
    }

    /// Reload all shaders (for hot reload)
    pub fn reload_all(&mut self) -> ViewportResult<()> {
        if !self.hot_reload {
            return Ok(());
        }

        for module in self.modules.values() {
            let mut module = module.write();
            if let Err(e) = module.source.reload() {
                eprintln!("Failed to reload shader '{}': {}", module.id, e);
                continue;
            }

            if let Err(e) = module.compile() {
                eprintln!("Failed to compile shader '{}': {}", module.id, e);
            }
        }

        // Validate pipelines
        for pipeline in self.pipelines.values() {
            let mut pipeline = pipeline.write();
            if let Err(e) = pipeline.validate() {
                eprintln!("Pipeline '{}' validation failed: {}", pipeline.id, e);
            }
        }

        Ok(())
    }

    /// Get statistics
    pub fn statistics(&self) -> &CompilationStats {
        &self.stats
    }

    /// Clear all cached shaders and pipelines
    pub fn clear(&mut self) {
        self.modules.clear();
        self.pipelines.clear();
    }
}

impl Default for ShaderCompiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Shader compilation statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompilationStats {
    /// Total compilation attempts
    pub compilations_attempted: u32,

    /// Successful compilations
    pub compilations_succeeded: u32,

    /// Failed compilations
    pub compilations_failed: u32,
}

impl CompilationStats {
    /// Get success rate
    pub fn success_rate(&self) -> f32 {
        if self.compilations_attempted == 0 {
            0.0
        } else {
            (self.compilations_succeeded as f32 / self.compilations_attempted as f32) * 100.0
        }
    }
}

/// Default shader sources
pub mod default_shaders {
    /// Default vertex shader (WGSL)
    pub const DEFAULT_VERTEX_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) color: vec4<f32>,
}

struct Uniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    let world_pos = uniforms.model * vec4<f32>(input.position, 1.0);
    output.clip_position = uniforms.view_proj * world_pos;
    output.world_position = world_pos.xyz;
    output.normal = (uniforms.model * vec4<f32>(input.normal, 0.0)).xyz;
    output.tex_coords = input.tex_coords;
    output.color = input.color;

    return output;
}
"#;

    /// Default fragment shader (WGSL)
    pub const DEFAULT_FRAGMENT_SHADER: &str = r#"
struct FragmentInput {
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) color: vec4<f32>,
}

@fragment
fn fs_main(input: FragmentInput) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let normal = normalize(input.normal);
    let diffuse = max(dot(normal, light_dir), 0.0);

    let ambient = 0.3;
    let lighting = ambient + diffuse * 0.7;

    return vec4<f32>(input.color.rgb * lighting, input.color.a);
}
"#;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_source_creation() {
        let source = ShaderSource::new(
            ShaderLanguage::WGSL,
            ShaderStage::Vertex,
            "fn vs_main() {}".to_string(),
        );

        assert_eq!(source.language, ShaderLanguage::WGSL);
        assert_eq!(source.stage, ShaderStage::Vertex);
        assert_eq!(source.entry_point, "vs_main");
    }

    #[test]
    fn test_shader_module_compilation() {
        let source = ShaderSource::new(
            ShaderLanguage::WGSL,
            ShaderStage::Vertex,
            default_shaders::DEFAULT_VERTEX_SHADER.to_string(),
        );

        let mut module = ShaderModule::new("test".to_string(), source);
        assert!(module.compile().is_ok());
        assert!(module.valid);
    }

    #[test]
    fn test_shader_compiler() {
        let mut compiler = ShaderCompiler::new();

        let vertex_source = ShaderSource::new(
            ShaderLanguage::WGSL,
            ShaderStage::Vertex,
            default_shaders::DEFAULT_VERTEX_SHADER.to_string(),
        );

        let result = compiler.compile_shader("test_vs".to_string(), vertex_source);
        assert!(result.is_ok());

        let module = compiler.get_module("test_vs");
        assert!(module.is_some());
    }

    #[test]
    fn test_compilation_stats() {
        let mut stats = CompilationStats::default();

        stats.compilations_attempted = 10;
        stats.compilations_succeeded = 8;
        stats.compilations_failed = 2;

        assert_eq!(stats.success_rate(), 80.0);
    }
}
