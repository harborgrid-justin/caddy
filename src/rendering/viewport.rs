//! Viewport management for multi-viewport layouts

use super::camera::Camera;
use nalgebra::{Point2, Point3, Vector2};

/// Viewport layout types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportLayout {
    /// Single viewport
    Single,
    /// Two viewports side-by-side
    Horizontal2,
    /// Two viewports stacked
    Vertical2,
    /// Four viewports in a grid
    Quad,
}

/// Viewport configuration
#[derive(Debug, Clone)]
pub struct ViewportConfig {
    pub name: String,
    pub show_grid: bool,
    pub show_axes: bool,
    pub grid_size: f32,
    pub grid_spacing: f32,
    pub background_color: [f32; 4],
}

impl Default for ViewportConfig {
    fn default() -> Self {
        Self {
            name: "Viewport".to_string(),
            show_grid: true,
            show_axes: true,
            grid_size: 100.0,
            grid_spacing: 10.0,
            background_color: [0.1, 0.1, 0.1, 1.0],
        }
    }
}

/// A viewport represents a rendering region with its own camera
pub struct Viewport {
    camera: Camera,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    config: ViewportConfig,
    active: bool,
}

impl Viewport {
    /// Create a new viewport
    pub fn new(camera: Camera, x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            camera,
            x,
            y,
            width,
            height,
            config: ViewportConfig::default(),
            active: false,
        }
    }

    /// Resize the viewport
    pub fn resize(&mut self, x: u32, y: u32, width: u32, height: u32) {
        self.x = x;
        self.y = y;
        self.width = width;
        self.height = height;

        // Update camera aspect ratio
        if height > 0 {
            let aspect = width as f32 / height as f32;
            self.camera.set_aspect_ratio(aspect);
        }
    }

    /// Get camera
    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    /// Get mutable camera
    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    /// Get viewport position X
    pub fn x(&self) -> u32 {
        self.x
    }

    /// Get viewport position Y
    pub fn y(&self) -> u32 {
        self.y
    }

    /// Get viewport width
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get viewport height
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get viewport configuration
    pub fn config(&self) -> &ViewportConfig {
        &self.config
    }

    /// Get mutable viewport configuration
    pub fn config_mut(&mut self) -> &mut ViewportConfig {
        &mut self.config
    }

    /// Set viewport active state
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    /// Check if viewport is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Check if a screen point is inside this viewport
    pub fn contains_point(&self, x: u32, y: u32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    /// Convert screen coordinates to viewport coordinates (normalized 0..1)
    pub fn screen_to_viewport(&self, screen_x: u32, screen_y: u32) -> Option<Point2<f32>> {
        if !self.contains_point(screen_x, screen_y) {
            return None;
        }

        let vp_x = (screen_x - self.x) as f32 / self.width as f32;
        let vp_y = (screen_y - self.y) as f32 / self.height as f32;

        Some(Point2::new(vp_x, vp_y))
    }

    /// Convert screen coordinates to normalized device coordinates (-1..1)
    pub fn screen_to_ndc(&self, screen_x: u32, screen_y: u32) -> Option<Point2<f32>> {
        self.screen_to_viewport(screen_x, screen_y).map(|vp| {
            Point2::new(vp.x * 2.0 - 1.0, 1.0 - vp.y * 2.0)
        })
    }

    /// Convert viewport coordinates to screen coordinates
    pub fn viewport_to_screen(&self, vp_x: f32, vp_y: f32) -> Point2<u32> {
        let screen_x = self.x + (vp_x * self.width as f32) as u32;
        let screen_y = self.y + (vp_y * self.height as f32) as u32;
        Point2::new(screen_x, screen_y)
    }

    /// Convert NDC to screen coordinates
    pub fn ndc_to_screen(&self, ndc_x: f32, ndc_y: f32) -> Point2<u32> {
        let vp_x = (ndc_x + 1.0) / 2.0;
        let vp_y = (1.0 - ndc_y) / 2.0;
        self.viewport_to_screen(vp_x, vp_y)
    }

    /// Project a 3D world point to 2D screen coordinates
    pub fn world_to_screen(&mut self, world_point: Point3<f32>) -> Point2<u32> {
        let view_proj = self.camera.view_projection_matrix();
        let view_proj = nalgebra::Matrix4::from(view_proj);

        // Transform to clip space
        let world = nalgebra::Vector4::new(world_point.x, world_point.y, world_point.z, 1.0);
        let clip = view_proj * world;

        // Perspective divide to get NDC
        let ndc = if clip.w != 0.0 {
            Point2::new(clip.x / clip.w, clip.y / clip.w)
        } else {
            Point2::new(0.0, 0.0)
        };

        self.ndc_to_screen(ndc.x, ndc.y)
    }

    /// Unproject a screen point to a ray in world space
    pub fn screen_to_ray(&mut self, screen_x: u32, screen_y: u32) -> Option<(Point3<f32>, Vector2<f32>)> {
        let ndc = self.screen_to_ndc(screen_x, screen_y)?;

        let view_proj = self.camera.view_projection_matrix();
        let view_proj = nalgebra::Matrix4::from(view_proj);
        let inv_view_proj = view_proj.try_inverse()?;

        // Create two points in clip space (near and far)
        let near = nalgebra::Vector4::new(ndc.x, ndc.y, -1.0, 1.0);
        let far = nalgebra::Vector4::new(ndc.x, ndc.y, 1.0, 1.0);

        // Transform to world space
        let near_world = inv_view_proj * near;
        let far_world = inv_view_proj * far;

        // Perspective divide
        let near_point = if near_world.w != 0.0 {
            Point3::new(
                near_world.x / near_world.w,
                near_world.y / near_world.w,
                near_world.z / near_world.w,
            )
        } else {
            Point3::new(0.0, 0.0, 0.0)
        };

        let far_point = if far_world.w != 0.0 {
            Point3::new(
                far_world.x / far_world.w,
                far_world.y / far_world.w,
                far_world.z / far_world.w,
            )
        } else {
            Point3::new(0.0, 0.0, 0.0)
        };

        // Calculate ray direction
        let direction = (far_point - near_point).normalize();

        Some((near_point, Vector2::new(direction.x, direction.y)))
    }

    /// Get the world-space size of a pixel at a given depth
    pub fn pixel_size_at_depth(&self, depth: f32) -> f32 {
        use super::camera::ProjectionType;

        match self.camera.projection_type() {
            ProjectionType::Orthographic => {
                // In orthographic projection, pixel size is constant
                self.camera.ortho_height() / self.height as f32
            }
            ProjectionType::Perspective => {
                // In perspective, pixel size increases with depth
                let fov = self.camera.fov().to_radians();
                let height_at_depth = 2.0 * depth * (fov / 2.0).tan();
                height_at_depth / self.height as f32
            }
        }
    }
}

/// Viewport manager for handling multiple viewports
pub struct ViewportManager {
    viewports: Vec<Viewport>,
    layout: ViewportLayout,
    active_viewport: usize,
}

impl ViewportManager {
    /// Create a new viewport manager with a single viewport
    pub fn new(width: u32, height: u32) -> Self {
        let camera = Camera::new_orthographic(
            Point3::new(0.0, 0.0, 100.0),
            Point3::new(0.0, 0.0, 0.0),
            nalgebra::Vector3::new(0.0, 1.0, 0.0),
            width as f32 / height as f32,
        );

        let mut viewport = Viewport::new(camera, 0, 0, width, height);
        viewport.set_active(true);

        Self {
            viewports: vec![viewport],
            layout: ViewportLayout::Single,
            active_viewport: 0,
        }
    }

    /// Set viewport layout
    pub fn set_layout(&mut self, layout: ViewportLayout, width: u32, height: u32) {
        self.layout = layout;

        match layout {
            ViewportLayout::Single => {
                if self.viewports.is_empty() {
                    let camera = Camera::new_orthographic(
                        Point3::new(0.0, 0.0, 100.0),
                        Point3::new(0.0, 0.0, 0.0),
                        nalgebra::Vector3::new(0.0, 1.0, 0.0),
                        1.0,
                    );
                    self.viewports.push(Viewport::new(camera, 0, 0, width, height));
                } else {
                    self.viewports[0].resize(0, 0, width, height);
                }
            }
            ViewportLayout::Horizontal2 => {
                let half_width = width / 2;
                self.ensure_viewport_count(2);
                self.viewports[0].resize(0, 0, half_width, height);
                self.viewports[1].resize(half_width, 0, half_width, height);
            }
            ViewportLayout::Vertical2 => {
                let half_height = height / 2;
                self.ensure_viewport_count(2);
                self.viewports[0].resize(0, 0, width, half_height);
                self.viewports[1].resize(0, half_height, width, half_height);
            }
            ViewportLayout::Quad => {
                let half_width = width / 2;
                let half_height = height / 2;
                self.ensure_viewport_count(4);
                self.viewports[0].resize(0, 0, half_width, half_height);
                self.viewports[1].resize(half_width, 0, half_width, half_height);
                self.viewports[2].resize(0, half_height, half_width, half_height);
                self.viewports[3].resize(half_width, half_height, half_width, half_height);
            }
        }
    }

    /// Ensure we have at least the specified number of viewports
    fn ensure_viewport_count(&mut self, count: usize) {
        while self.viewports.len() < count {
            let camera = Camera::new_orthographic(
                Point3::new(0.0, 0.0, 100.0),
                Point3::new(0.0, 0.0, 0.0),
                nalgebra::Vector3::new(0.0, 1.0, 0.0),
                1.0,
            );
            self.viewports.push(Viewport::new(camera, 0, 0, 100, 100));
        }
    }

    /// Get viewport at index
    pub fn get(&self, index: usize) -> Option<&Viewport> {
        self.viewports.get(index)
    }

    /// Get mutable viewport at index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Viewport> {
        self.viewports.get_mut(index)
    }

    /// Get all viewports
    pub fn viewports(&self) -> &[Viewport] {
        &self.viewports
    }

    /// Get all viewports mutably
    pub fn viewports_mut(&mut self) -> &mut [Viewport] {
        &mut self.viewports
    }

    /// Get active viewport
    pub fn active_viewport(&self) -> &Viewport {
        &self.viewports[self.active_viewport]
    }

    /// Get active viewport mutably
    pub fn active_viewport_mut(&mut self) -> &mut Viewport {
        &mut self.viewports[self.active_viewport]
    }

    /// Set active viewport
    pub fn set_active_viewport(&mut self, index: usize) {
        if index < self.viewports.len() {
            self.viewports[self.active_viewport].set_active(false);
            self.active_viewport = index;
            self.viewports[self.active_viewport].set_active(true);
        }
    }

    /// Find viewport containing a screen point
    pub fn viewport_at_point(&self, x: u32, y: u32) -> Option<usize> {
        self.viewports
            .iter()
            .position(|vp| vp.contains_point(x, y))
    }

    /// Get current layout
    pub fn layout(&self) -> ViewportLayout {
        self.layout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_creation() {
        let camera = Camera::new_orthographic(
            Point3::new(0.0, 0.0, 100.0),
            Point3::new(0.0, 0.0, 0.0),
            nalgebra::Vector3::new(0.0, 1.0, 0.0),
            1.0,
        );

        let viewport = Viewport::new(camera, 0, 0, 800, 600);
        assert_eq!(viewport.width(), 800);
        assert_eq!(viewport.height(), 600);
    }

    #[test]
    fn test_contains_point() {
        let camera = Camera::new_orthographic(
            Point3::new(0.0, 0.0, 100.0),
            Point3::new(0.0, 0.0, 0.0),
            nalgebra::Vector3::new(0.0, 1.0, 0.0),
            1.0,
        );

        let viewport = Viewport::new(camera, 100, 100, 200, 200);
        assert!(viewport.contains_point(150, 150));
        assert!(viewport.contains_point(100, 100));
        assert!(!viewport.contains_point(50, 50));
        assert!(!viewport.contains_point(350, 350));
    }

    #[test]
    fn test_screen_to_viewport() {
        let camera = Camera::new_orthographic(
            Point3::new(0.0, 0.0, 100.0),
            Point3::new(0.0, 0.0, 0.0),
            nalgebra::Vector3::new(0.0, 1.0, 0.0),
            1.0,
        );

        let viewport = Viewport::new(camera, 0, 0, 800, 600);
        let vp = viewport.screen_to_viewport(400, 300).unwrap();
        assert!((vp.x - 0.5).abs() < 0.01);
        assert!((vp.y - 0.5).abs() < 0.01);
    }
}
