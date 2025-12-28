# CADDY Rendering Engine

A production-grade GPU-accelerated rendering system built with wgpu for cross-platform CAD visualization.

## Architecture

The rendering module is organized into several key components:

### Core Components

#### 1. **Renderer** (`renderer.rs`)
The main rendering engine that manages:
- wgpu device, queue, and surface initialization
- Multi-viewport rendering
- MSAA (4x anti-aliasing) support
- Depth buffering
- Render mode switching (Wireframe, Shaded, HiddenLine, ShadedWithEdges)
- Global uniform management (transforms, lighting)

**Key Features:**
- Automatic surface reconfiguration on resize
- Triple-buffered rendering for smooth performance
- Support for millions of vertices
- Target: 60 FPS for interactive CAD work

#### 2. **Camera** (`camera.rs`)
Sophisticated camera system with:
- **Projection Types:**
  - Orthographic (standard for CAD technical drawings)
  - Perspective (3D visualization)

- **Named Views:**
  - Top, Bottom, Front, Back, Left, Right
  - Four isometric views (SW, SE, NE, NW)

- **Camera Controller:**
  - Orbit: Rotate around target point
  - Pan: Translate view parallel to screen
  - Zoom: Perspective distance or orthographic scale
  - Zoom Extents: Frame all geometry

#### 3. **Viewport** (`viewport.rs`)
Multi-viewport management supporting:
- **Layout Types:**
  - Single viewport (full window)
  - Horizontal 2-split
  - Vertical 2-split
  - Quad (4-way split for Top/Front/Right/Isometric)

- **Coordinate Transformations:**
  - Screen to viewport (normalized 0..1)
  - Screen to NDC (normalized device coordinates -1..1)
  - World to screen projection
  - Screen to ray (for picking/selection)

- **Per-Viewport Configuration:**
  - Grid display
  - Axis display
  - Background color
  - Grid size and spacing

#### 4. **Pipeline** (`pipeline.rs`)
Render pipeline cache managing:
- **LinePipeline**: 2D/3D line rendering with thickness
- **MeshPipeline**: Solid mesh rendering with Phong lighting
- **PointPipeline**: Vertex/point rendering
- **TextPipeline**: Text annotation rendering
- **GridPipeline**: Infinite grid effect with distance fade

Each pipeline supports both solid and wireframe modes.

#### 5. **Shaders** (`shaders.rs`)
Complete WGSL shader collection:
- **Line Shader**: Variable thickness lines
- **Mesh Shader**: Phong lighting (ambient, diffuse, specular)
- **Point Shader**: Circular point rendering
- **Text Shader**: Text quad rendering
- **Grid Shader**: Infinite grid with fade
- **Selection Shader**: Animated highlight effect
- **Hidden Line Shader**: Edge detection for technical drawings
- **Construction Shader**: Dashed line effect
- **Axis Shader**: XYZ axis rendering

#### 6. **Buffers** (`buffers.rs`)
Efficient GPU memory management:
- **VertexBuffer<T>**: Dynamic vertex buffer with auto-growth
- **IndexBuffer**: U16/U32 index buffer management
- **UniformBuffer<T>**: Typed uniform buffers
- **DynamicBuffer<T>**: Triple-buffered dynamic updates
- **BufferPool<T>**: Buffer pooling for reduced allocations
- **StagingBuffer**: Efficient CPU-to-GPU transfers

## Vertex Formats

### LineVertex
```rust
struct LineVertex {
    position: [f32; 3],    // World position
    color: [f32; 4],       // RGBA color
    thickness: f32,        // Line thickness
}
```

### MeshVertex
```rust
struct MeshVertex {
    position: [f32; 3],    // World position
    normal: [f32; 3],      // Surface normal
    color: [f32; 4],       // RGBA color
    uv: [f32; 2],          // Texture coordinates
}
```

## Uniform Data

### TransformUniforms
```rust
struct TransformUniforms {
    view_proj: mat4x4,     // Combined view-projection matrix
    model: mat4x4,         // Model transformation matrix
    normal_matrix: mat4x4, // Normal transformation matrix
}
```

### LightUniforms
```rust
struct LightUniforms {
    light_position: vec3,  // World-space light position
    light_color: vec3,     // RGB light color
    ambient_strength: f32, // Ambient lighting factor
    specular_strength: f32,// Specular highlight strength
    shininess: f32,        // Phong shininess exponent
}
```

## Usage Example

```rust
use caddy::rendering::*;
use std::sync::Arc;

// Initialize renderer
let renderer = Renderer::new(window, 1920, 1080).await?;

// Set up multi-viewport layout
renderer.set_viewport_layout(ViewportLayout::Quad);

// Configure viewports with named views
let viewports = renderer.viewports_mut();
viewports[0].camera_mut().set_named_view(NamedView::Top);
viewports[1].camera_mut().set_named_view(NamedView::Front);
viewports[2].camera_mut().set_named_view(NamedView::Right);
viewports[3].camera_mut().set_named_view(NamedView::IsometricSW);

// Set render mode
renderer.set_render_mode(RenderMode::ShadedWithEdges);

// Create geometry buffers
let mut vertices = Vec::new();
let mut indices = Vec::new();

// ... populate vertices and indices ...

let vertex_buffer = VertexBuffer::new_with_data(
    renderer.device(),
    "Model Vertices",
    &vertices,
);

let index_buffer = IndexBuffer::new_with_data_u32(
    renderer.device(),
    "Model Indices",
    &indices,
);

// Render frame
renderer.render(|render_pass, viewport, bind_group| {
    // Set pipeline
    let cache = renderer.pipeline_cache().read();
    let pipeline = cache.mesh_pipeline().pipeline();
    render_pass.set_pipeline(pipeline);

    // Bind global uniforms
    render_pass.set_bind_group(0, bind_group, &[]);

    // Draw geometry
    render_pass.set_vertex_buffer(0, vertex_buffer.buffer().slice(..));
    render_pass.set_index_buffer(
        index_buffer.buffer().slice(..),
        index_buffer.format(),
    );
    render_pass.draw_indexed(0..index_buffer.count() as u32, 0, 0..1);
})?;
```

## Camera Controls

```rust
use caddy::rendering::CameraController;

let mut controller = CameraController::new();

// Orbit camera (mouse drag)
controller.orbit(camera, delta_x, delta_y);

// Pan camera (middle mouse drag)
controller.pan(camera, delta_x, delta_y);

// Zoom camera (mouse wheel)
controller.zoom(camera, wheel_delta);

// Zoom to fit geometry
controller.zoom_extents(camera, bbox_min, bbox_max);
```

## Performance Characteristics

- **Target Frame Rate**: 60 FPS
- **Vertex Capacity**: Millions (tested with 5M+ vertices)
- **Anti-Aliasing**: 4x MSAA
- **Viewport Switching**: < 1ms overhead
- **Buffer Growth**: 1.5x geometric growth
- **Depth Precision**: 32-bit float depth buffer

## Render Modes

1. **Wireframe**: Lines only, no surface fill
2. **Shaded**: Fully lit surfaces with Phong shading
3. **HiddenLine**: Edges with hidden line removal
4. **ShadedWithEdges**: Shaded surfaces + visible edges (best for CAD)

## Coordinate Systems

- **World Space**: Right-handed, Y-up
- **View Space**: Camera-relative
- **Clip Space**: Normalized device coordinates
- **Screen Space**: Pixel coordinates

## Future Enhancements

- [ ] Shadow mapping for realistic shadows
- [ ] Screen-space ambient occlusion (SSAO)
- [ ] Render-to-texture for thumbnails
- [ ] Compute shader support for GPU tessellation
- [ ] Multi-threaded command buffer generation
- [ ] Occlusion culling for large models
- [ ] Level-of-detail (LOD) system
- [ ] GPU-based picking with selection buffer

## Dependencies

- `wgpu` 0.18: Core GPU abstraction
- `winit` 0.29: Window management
- `nalgebra` 0.32: Linear algebra
- `bytemuck`: Safe type casting for GPU data
- `parking_lot`: High-performance locks

## Thread Safety

All renderer components use `Arc` and `RwLock` for thread-safe access:
- Device and Queue are wrapped in `Arc` for shared ownership
- PipelineCache uses `RwLock` for concurrent read access
- Buffers can be safely updated from background threads

## Error Handling

All operations return `RenderResult<T>` with comprehensive error types:
- `InitializationError`: GPU setup failures
- `NoAdapter`: No suitable GPU found
- `DeviceRequest`: GPU device request failed
- `SurfaceCreation`: Window surface creation failed
- `PipelineCreation`: Shader compilation errors
- `BufferCreation`: Buffer allocation errors
- `RenderFailure`: Runtime rendering errors

## Testing

Run the test suite:
```bash
cargo test --package caddy --lib rendering
```

## License

MIT License - See LICENSE file for details
