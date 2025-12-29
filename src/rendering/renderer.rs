//! Main rendering engine with wgpu context

use super::*;
use camera::Camera;
use viewport::{Viewport, ViewportLayout};
use pipeline::PipelineCache;
use buffers::UniformBuffer;
use std::sync::Arc;
use parking_lot::RwLock;

/// Rendering modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    /// Wireframe only
    Wireframe,
    /// Shaded with lighting
    Shaded,
    /// Hidden line removal
    HiddenLine,
    /// Wireframe on shaded
    ShadedWithEdges,
}

/// Main rendering context
pub struct RenderContext {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub adapter_info: wgpu::AdapterInfo,
}

/// Main renderer
pub struct Renderer {
    context: RenderContext,
    pipeline_cache: Arc<RwLock<PipelineCache>>,
    viewports: Vec<Viewport>,
    viewport_layout: ViewportLayout,
    render_mode: RenderMode,
    msaa_samples: u32,
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
    msaa_texture: Option<wgpu::Texture>,
    msaa_view: Option<wgpu::TextureView>,
    transform_uniform: UniformBuffer<TransformUniforms>,
    light_uniform: UniformBuffer<LightUniforms>,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl Renderer {
    /// Initialize the renderer with a window surface
    pub async fn new(
        window: Arc<winit::window::Window>,
        width: u32,
        height: u32,
    ) -> RenderResult<Self> {
        // Create wgpu instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface
        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| RenderError::SurfaceCreation(e.to_string()))?;

        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(RenderError::NoAdapter)?;

        let adapter_info = adapter.get_info();
        log::info!("Using GPU: {} ({:?})", adapter_info.name, adapter_info.backend);

        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("CADDY Device"),
                    required_features: wgpu::Features::POLYGON_MODE_LINE
                        | wgpu::Features::MULTI_DRAW_INDIRECT
                        | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                    required_limits: wgpu::Limits {
                        max_texture_dimension_2d: 8192,
                        max_bind_groups: 4,
                        ..Default::default()
                    },
                },
                None,
            )
            .await
            .map_err(|e| RenderError::DeviceRequest(e.to_string()))?;

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        let _context = RenderContext {
            device: device.clone(),
            queue: queue.clone(),
            surface,
            surface_config,
            adapter_info,
        };

        // Create depth texture
        let (depth_texture, depth_view) = Self::create_depth_texture(&device, width, height, 1);

        // Create MSAA textures (4x)
        let msaa_samples = 4;
        let (msaa_texture, msaa_view) =
            Self::create_msaa_texture(&device, width, height, msaa_samples, surface_format);

        // Create uniform buffers
        let transform_uniform = UniformBuffer::new(
            device.clone(),
            "Transform Uniform",
            TransformUniforms::default(),
        );

        let light_uniform = UniformBuffer::new(
            device.clone(),
            "Light Uniform",
            LightUniforms::default(),
        );

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Global Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Global Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: transform_uniform.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_uniform.buffer().as_entire_binding(),
                },
            ],
        });

        // Initialize pipeline cache
        let pipeline_cache = Arc::new(RwLock::new(PipelineCache::new(
            device.clone(),
            &bind_group_layout,
            surface_format,
            msaa_samples,
        )?));

        // Create default viewport
        let mut viewports = Vec::new();
        let camera = Camera::new_orthographic(
            [0.0, 0.0, 100.0].into(),
            [0.0, 0.0, 0.0].into(),
            [0.0, 1.0, 0.0].into(),
            width as f32 / height as f32,
        );
        viewports.push(Viewport::new(
            camera,
            0,
            0,
            width,
            height,
        ));

        Ok(Self {
            context,
            pipeline_cache,
            viewports,
            viewport_layout: ViewportLayout::Single,
            render_mode: RenderMode::ShadedWithEdges,
            msaa_samples,
            depth_texture,
            depth_view,
            msaa_texture: Some(msaa_texture),
            msaa_view: Some(msaa_view),
            transform_uniform,
            light_uniform,
            bind_group_layout,
            bind_group,
        })
    }

    /// Create depth texture
    fn create_depth_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        sample_count: u32,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        (texture, view)
    }

    /// Create MSAA texture
    fn create_msaa_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        sample_count: u32,
        format: wgpu::TextureFormat,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("MSAA Texture"),
            size,
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        (texture, view)
    }

    /// Resize the renderer
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.context.surface_config.width = width;
            self.context.surface_config.height = height;
            self.context
                .surface
                .configure(&self.context.device, &self.context.surface_config);

            // Recreate depth texture
            let (depth_texture, depth_view) =
                Self::create_depth_texture(&self.context.device, width, height, 1);
            self.depth_texture = depth_texture;
            self.depth_view = depth_view;

            // Recreate MSAA texture
            let (msaa_texture, msaa_view) = Self::create_msaa_texture(
                &self.context.device,
                width,
                height,
                self.msaa_samples,
                self.context.surface_config.format,
            );
            self.msaa_texture = Some(msaa_texture);
            self.msaa_view = Some(msaa_view);

            // Update viewport layout
            self.update_viewport_layout(width, height);
        }
    }

    /// Update viewport layout based on window size
    fn update_viewport_layout(&mut self, width: u32, height: u32) {
        match self.viewport_layout {
            ViewportLayout::Single => {
                if let Some(viewport) = self.viewports.get_mut(0) {
                    viewport.resize(0, 0, width, height);
                }
            }
            ViewportLayout::Horizontal2 => {
                let half_width = width / 2;
                if self.viewports.len() >= 2 {
                    self.viewports[0].resize(0, 0, half_width, height);
                    self.viewports[1].resize(half_width, 0, half_width, height);
                }
            }
            ViewportLayout::Vertical2 => {
                let half_height = height / 2;
                if self.viewports.len() >= 2 {
                    self.viewports[0].resize(0, 0, width, half_height);
                    self.viewports[1].resize(0, half_height, width, half_height);
                }
            }
            ViewportLayout::Quad => {
                let half_width = width / 2;
                let half_height = height / 2;
                if self.viewports.len() >= 4 {
                    self.viewports[0].resize(0, 0, half_width, half_height);
                    self.viewports[1].resize(half_width, 0, half_width, half_height);
                    self.viewports[2].resize(0, half_height, half_width, half_height);
                    self.viewports[3].resize(half_width, half_height, half_width, half_height);
                }
            }
        }
    }

    /// Set viewport layout
    pub fn set_viewport_layout(&mut self, layout: ViewportLayout) {
        self.viewport_layout = layout;

        // Ensure we have enough viewports
        match layout {
            ViewportLayout::Single => {
                if self.viewports.is_empty() {
                    let camera = Camera::new_orthographic(
                        [0.0, 0.0, 100.0].into(),
                        [0.0, 0.0, 0.0].into(),
                        [0.0, 1.0, 0.0].into(),
                        1.0,
                    );
                    self.viewports.push(Viewport::new(
                        camera,
                        0,
                        0,
                        self.context.surface_config.width,
                        self.context.surface_config.height,
                    ));
                }
            }
            ViewportLayout::Horizontal2 | ViewportLayout::Vertical2 => {
                while self.viewports.len() < 2 {
                    let camera = Camera::new_orthographic(
                        [0.0, 0.0, 100.0].into(),
                        [0.0, 0.0, 0.0].into(),
                        [0.0, 1.0, 0.0].into(),
                        1.0,
                    );
                    self.viewports.push(Viewport::new(camera, 0, 0, 100, 100));
                }
            }
            ViewportLayout::Quad => {
                while self.viewports.len() < 4 {
                    let camera = Camera::new_orthographic(
                        [0.0, 0.0, 100.0].into(),
                        [0.0, 0.0, 0.0].into(),
                        [0.0, 1.0, 0.0].into(),
                        1.0,
                    );
                    self.viewports.push(Viewport::new(camera, 0, 0, 100, 100));
                }
            }
        }

        self.update_viewport_layout(
            self.context.surface_config.width,
            self.context.surface_config.height,
        );
    }

    /// Set render mode
    pub fn set_render_mode(&mut self, mode: RenderMode) {
        self.render_mode = mode;
    }

    /// Get render mode
    pub fn render_mode(&self) -> RenderMode {
        self.render_mode
    }

    /// Get viewports
    pub fn viewports(&self) -> &[Viewport] {
        &self.viewports
    }

    /// Get mutable viewports
    pub fn viewports_mut(&mut self) -> &mut [Viewport] {
        &mut self.viewports
    }

    /// Begin a render pass
    pub fn render<F>(&mut self, mut render_fn: F) -> RenderResult<()>
    where
        F: FnMut(&mut wgpu::RenderPass, &Viewport, &wgpu::BindGroup),
    {
        // Get current surface texture
        let output = self
            .context
            .surface
            .get_current_texture()
            .map_err(|e| RenderError::RenderFailure(e.to_string()))?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create command encoder
        let mut encoder = self
            .context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Render each viewport
        for viewport in &mut self.viewports {
            // Update transform uniform
            let view_proj = viewport.camera_mut().view_projection_matrix();
            let mut transform = TransformUniforms::default();
            transform.view_proj = view_proj;
            self.transform_uniform.update(&self.context.queue, transform);

            // Create render pass
            let color_attachment = if self.msaa_samples > 1 {
                wgpu::RenderPassColorAttachment {
                    view: self.msaa_view.as_ref().unwrap(),
                    resolve_target: Some(&view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                }
            } else {
                wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                }
            };

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(color_attachment)],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });

            // Set viewport
            render_pass.set_viewport(
                viewport.x() as f32,
                viewport.y() as f32,
                viewport.width() as f32,
                viewport.height() as f32,
                0.0,
                1.0,
            );

            // Call user render function
            render_fn(&mut render_pass, viewport, &self.bind_group);
        }

        // Submit commands
        self.context.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Get device
    pub fn device(&self) -> &wgpu::Device {
        &self.context.device
    }

    /// Get queue
    pub fn queue(&self) -> &wgpu::Queue {
        &self.context.queue
    }

    /// Get pipeline cache
    pub fn pipeline_cache(&self) -> &Arc<RwLock<PipelineCache>> {
        &self.pipeline_cache
    }

    /// Get surface format
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.context.surface_config.format
    }

    /// Get bind group layout
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    /// Update light uniforms
    pub fn update_light(&mut self, light: LightUniforms) {
        self.light_uniform.update(&self.context.queue, light);
    }
}
