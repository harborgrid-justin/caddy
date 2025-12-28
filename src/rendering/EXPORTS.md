# CADDY Rendering Module - Public API

## Module Exports

All types are accessible via `use caddy::rendering::*;`

## Core Types

### Renderer
```rust
pub struct Renderer { ... }

impl Renderer {
    pub async fn new(window: Arc<winit::window::Window>, width: u32, height: u32) -> RenderResult<Self>;
    pub fn resize(&mut self, width: u32, height: u32);
    pub fn set_viewport_layout(&mut self, layout: ViewportLayout);
    pub fn set_render_mode(&mut self, mode: RenderMode);
    pub fn render<F>(&mut self, render_fn: F) -> RenderResult<()>;
    pub fn viewports(&self) -> &[Viewport];
    pub fn viewports_mut(&mut self) -> &mut [Viewport];
    pub fn device(&self) -> &wgpu::Device;
    pub fn queue(&self) -> &wgpu::Queue;
    pub fn pipeline_cache(&self) -> &Arc<RwLock<PipelineCache>>;
    pub fn update_light(&mut self, light: LightUniforms);
}
```

### RenderMode
```rust
pub enum RenderMode {
    Wireframe,
    Shaded,
    HiddenLine,
    ShadedWithEdges,
}
```

### Camera
```rust
pub struct Camera { ... }

impl Camera {
    pub fn new_orthographic(position: Point3<f32>, target: Point3<f32>, up: Vector3<f32>, aspect_ratio: f32) -> Self;
    pub fn new_perspective(position: Point3<f32>, target: Point3<f32>, up: Vector3<f32>, aspect_ratio: f32, fov: f32) -> Self;
    pub fn view_matrix(&mut self) -> [[f32; 4]; 4];
    pub fn projection_matrix(&mut self) -> [[f32; 4]; 4];
    pub fn view_projection_matrix(&mut self) -> [[f32; 4]; 4];
    pub fn set_position(&mut self, position: Point3<f32>);
    pub fn set_target(&mut self, target: Point3<f32>);
    pub fn set_projection_type(&mut self, projection_type: ProjectionType);
    pub fn set_named_view(&mut self, view: NamedView);
}
```

### ProjectionType
```rust
pub enum ProjectionType {
    Orthographic,
    Perspective,
}
```

### NamedView
```rust
pub enum NamedView {
    Top,
    Bottom,
    Front,
    Back,
    Left,
    Right,
    IsometricSW,
    IsometricSE,
    IsometricNE,
    IsometricNW,
}
```

### CameraController
```rust
pub struct CameraController { ... }

impl CameraController {
    pub fn new() -> Self;
    pub fn orbit(&self, camera: &mut Camera, delta_x: f32, delta_y: f32);
    pub fn pan(&self, camera: &mut Camera, delta_x: f32, delta_y: f32);
    pub fn zoom(&self, camera: &mut Camera, delta: f32);
    pub fn zoom_extents(&self, camera: &mut Camera, min: Point3<f32>, max: Point3<f32>);
}
```

### Viewport
```rust
pub struct Viewport { ... }

impl Viewport {
    pub fn new(camera: Camera, x: u32, y: u32, width: u32, height: u32) -> Self;
    pub fn resize(&mut self, x: u32, y: u32, width: u32, height: u32);
    pub fn camera(&self) -> &Camera;
    pub fn camera_mut(&mut self) -> &mut Camera;
    pub fn config(&self) -> &ViewportConfig;
    pub fn config_mut(&mut self) -> &mut ViewportConfig;
    pub fn contains_point(&self, x: u32, y: u32) -> bool;
    pub fn screen_to_viewport(&self, screen_x: u32, screen_y: u32) -> Option<Point2<f32>>;
    pub fn screen_to_ndc(&self, screen_x: u32, screen_y: u32) -> Option<Point2<f32>>;
    pub fn world_to_screen(&mut self, world_point: Point3<f32>) -> Point2<u32>;
    pub fn screen_to_ray(&mut self, screen_x: u32, screen_y: u32) -> Option<(Point3<f32>, Vector2<f32>)>;
}
```

### ViewportLayout
```rust
pub enum ViewportLayout {
    Single,
    Horizontal2,
    Vertical2,
    Quad,
}
```

### ViewportConfig
```rust
pub struct ViewportConfig {
    pub name: String,
    pub show_grid: bool,
    pub show_axes: bool,
    pub grid_size: f32,
    pub grid_spacing: f32,
    pub background_color: [f32; 4],
}
```

### Pipelines
```rust
pub struct LinePipeline { ... }
pub struct MeshPipeline { ... }
pub struct PointPipeline { ... }
pub struct TextPipeline { ... }

pub struct PipelineCache { ... }

impl PipelineCache {
    pub fn line_pipeline(&self) -> &LinePipeline;
    pub fn mesh_pipeline(&self) -> &MeshPipeline;
    pub fn point_pipeline(&self) -> &PointPipeline;
    pub fn text_pipeline(&self) -> &TextPipeline;
    pub fn grid_pipeline(&self) -> Option<&wgpu::RenderPipeline>;
}
```

### Buffers
```rust
pub struct VertexBuffer<T> { ... }

impl<T: bytemuck::Pod> VertexBuffer<T> {
    pub fn new(device: Arc<wgpu::Device>, label: &str, capacity: usize) -> Self;
    pub fn new_with_data(device: Arc<wgpu::Device>, label: &str, data: &[T]) -> Self;
    pub fn update(&mut self, queue: &wgpu::Queue, data: &[T]);
    pub fn append(&mut self, queue: &wgpu::Queue, data: &[T]);
    pub fn clear(&mut self);
    pub fn buffer(&self) -> &wgpu::Buffer;
    pub fn count(&self) -> usize;
}

pub struct IndexBuffer { ... }

impl IndexBuffer {
    pub fn new_u32(device: Arc<wgpu::Device>, label: &str, capacity: usize) -> Self;
    pub fn new_u16(device: Arc<wgpu::Device>, label: &str, capacity: usize) -> Self;
    pub fn new_with_data_u32(device: Arc<wgpu::Device>, label: &str, data: &[u32]) -> Self;
    pub fn new_with_data_u16(device: Arc<wgpu::Device>, label: &str, data: &[u16]) -> Self;
    pub fn update_u32(&mut self, queue: &wgpu::Queue, data: &[u32]);
    pub fn update_u16(&mut self, queue: &wgpu::Queue, data: &[u16]);
    pub fn buffer(&self) -> &wgpu::Buffer;
    pub fn format(&self) -> wgpu::IndexFormat;
}

pub struct UniformBuffer<T> { ... }

impl<T: bytemuck::Pod> UniformBuffer<T> {
    pub fn new(device: Arc<wgpu::Device>, label: &str, initial_data: T) -> Self;
    pub fn update(&self, queue: &wgpu::Queue, data: T);
    pub fn buffer(&self) -> &wgpu::Buffer;
}

pub struct DynamicBuffer<T> { ... }

impl<T: bytemuck::Pod> DynamicBuffer<T> {
    pub fn new(device: Arc<wgpu::Device>, label: &str, capacity: usize) -> Self;
    pub fn current_buffer(&self) -> &wgpu::Buffer;
    pub fn update(&mut self, queue: &wgpu::Queue, data: &[T]) -> RenderResult<()>;
}
```

## Vertex Formats

### LineVertex
```rust
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LineVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub thickness: f32,
    pub _padding: [f32; 3],
}

impl LineVertex {
    pub fn new(position: [f32; 3], color: [f32; 4], thickness: f32) -> Self;
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}
```

### MeshVertex
```rust
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 4],
    pub uv: [f32; 2],
}

impl MeshVertex {
    pub fn new(position: [f32; 3], normal: [f32; 3], color: [f32; 4], uv: [f32; 2]) -> Self;
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}
```

## Uniform Data

### TransformUniforms
```rust
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformUniforms {
    pub view_proj: [[f32; 4]; 4],
    pub model: [[f32; 4]; 4],
    pub normal_matrix: [[f32; 4]; 4],
}
```

### LightUniforms
```rust
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniforms {
    pub light_position: [f32; 3],
    pub light_color: [f32; 3],
    pub ambient_strength: f32,
    pub specular_strength: f32,
    pub shininess: f32,
}
```

## Error Types

```rust
pub enum RenderError {
    InitializationError(String),
    NoAdapter,
    DeviceRequest(String),
    SurfaceCreation(String),
    PipelineCreation(String),
    BufferCreation(String),
    ShaderCreation(String),
    RenderFailure(String),
    ViewportError(String),
}

pub type RenderResult<T> = Result<T, RenderError>;
```

## Shaders

All shaders are accessible via the `Shaders` struct:

```rust
pub struct Shaders;

impl Shaders {
    pub fn line_shader() -> &'static str;
    pub fn mesh_shader() -> &'static str;
    pub fn point_shader() -> &'static str;
    pub fn text_shader() -> &'static str;
    pub fn grid_shader() -> &'static str;
    pub fn selection_shader() -> &'static str;
    pub fn hidden_line_shader() -> &'static str;
    pub fn construction_shader() -> &'static str;
    pub fn axis_shader() -> &'static str;
}
```

## Complete Usage Example

```rust
use caddy::rendering::*;
use std::sync::Arc;

// Initialize renderer
let window = Arc::new(window); // winit window
let mut renderer = Renderer::new(window, 1920, 1080).await?;

// Configure viewports
renderer.set_viewport_layout(ViewportLayout::Quad);
let viewports = renderer.viewports_mut();
viewports[0].camera_mut().set_named_view(NamedView::Top);
viewports[1].camera_mut().set_named_view(NamedView::Front);
viewports[2].camera_mut().set_named_view(NamedView::Right);
viewports[3].camera_mut().set_named_view(NamedView::IsometricSW);

// Set render mode
renderer.set_render_mode(RenderMode::ShadedWithEdges);

// Create geometry
let vertices = vec![
    MeshVertex::new(
        [0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 0.0, 0.0, 1.0],
        [0.0, 0.0],
    ),
    // ... more vertices
];

let indices: Vec<u32> = vec![0, 1, 2, /* ... */];

let vertex_buffer = VertexBuffer::new_with_data(
    renderer.device().clone(),
    "Vertices",
    &vertices,
);

let index_buffer = IndexBuffer::new_with_data_u32(
    renderer.device().clone(),
    "Indices",
    &indices,
);

// Render loop
loop {
    renderer.render(|render_pass, viewport, bind_group| {
        let cache = renderer.pipeline_cache().read();
        let pipeline = cache.mesh_pipeline().pipeline();

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.buffer().slice(..));
        render_pass.set_index_buffer(
            index_buffer.buffer().slice(..),
            index_buffer.format(),
        );
        render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
    })?;
}
```

## Thread Safety

All types are thread-safe:
- `Arc<wgpu::Device>` and `Arc<wgpu::Queue>` for shared GPU access
- `Arc<RwLock<PipelineCache>>` for concurrent pipeline access
- All buffers can be updated from any thread

## Performance Notes

- Vertex buffers auto-grow with 1.5x expansion factor
- Triple-buffering prevents pipeline stalls
- MSAA 4x for high-quality anti-aliasing
- 32-bit depth buffer for precision
- Target: 60 FPS with millions of vertices
