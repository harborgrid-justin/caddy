//! Camera system for CAD viewport navigation

use nalgebra::{Matrix4, Point3, Vector3, Perspective3, Orthographic3};
use std::f32::consts::PI;

/// Camera projection type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionType {
    /// Orthographic projection (standard for CAD)
    Orthographic,
    /// Perspective projection (3D visualization)
    Perspective,
}

/// Named standard views
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamedView {
    /// Top view (looking down Y axis)
    Top,
    /// Bottom view (looking up Y axis)
    Bottom,
    /// Front view (looking down Z axis)
    Front,
    /// Back view (looking up Z axis)
    Back,
    /// Left view (looking right along X axis)
    Left,
    /// Right view (looking left along X axis)
    Right,
    /// Isometric view (SW isometric)
    IsometricSW,
    /// Isometric view (SE isometric)
    IsometricSE,
    /// Isometric view (NE isometric)
    IsometricNE,
    /// Isometric view (NW isometric)
    IsometricNW,
}

/// Camera for viewport navigation
pub struct Camera {
    position: Point3<f32>,
    target: Point3<f32>,
    up: Vector3<f32>,
    projection_type: ProjectionType,
    aspect_ratio: f32,
    fov: f32,
    near: f32,
    far: f32,
    ortho_height: f32,
    view_matrix: Matrix4<f32>,
    proj_matrix: Matrix4<f32>,
    view_proj_matrix: Matrix4<f32>,
    dirty: bool,
}

impl Camera {
    /// Create a new camera with orthographic projection
    pub fn new_orthographic(
        position: Point3<f32>,
        target: Point3<f32>,
        up: Vector3<f32>,
        aspect_ratio: f32,
    ) -> Self {
        let mut camera = Self {
            position,
            target,
            up: up.normalize(),
            projection_type: ProjectionType::Orthographic,
            aspect_ratio,
            fov: 45.0,
            near: 0.1,
            far: 1000.0,
            ortho_height: 100.0,
            view_matrix: Matrix4::identity(),
            proj_matrix: Matrix4::identity(),
            view_proj_matrix: Matrix4::identity(),
            dirty: true,
        };
        camera.update_matrices();
        camera
    }

    /// Create a new camera with perspective projection
    pub fn new_perspective(
        position: Point3<f32>,
        target: Point3<f32>,
        up: Vector3<f32>,
        aspect_ratio: f32,
        fov: f32,
    ) -> Self {
        let mut camera = Self {
            position,
            target,
            up: up.normalize(),
            projection_type: ProjectionType::Perspective,
            aspect_ratio,
            fov,
            near: 0.1,
            far: 1000.0,
            ortho_height: 100.0,
            view_matrix: Matrix4::identity(),
            proj_matrix: Matrix4::identity(),
            view_proj_matrix: Matrix4::identity(),
            dirty: true,
        };
        camera.update_matrices();
        camera
    }

    /// Update view and projection matrices
    fn update_matrices(&mut self) {
        if !self.dirty {
            return;
        }

        // Calculate view matrix
        self.view_matrix = Matrix4::look_at_rh(&self.position, &self.target, &self.up);

        // Calculate projection matrix
        self.proj_matrix = match self.projection_type {
            ProjectionType::Perspective => {
                let perspective = Perspective3::new(
                    self.aspect_ratio,
                    self.fov.to_radians(),
                    self.near,
                    self.far,
                );
                perspective.to_homogeneous()
            }
            ProjectionType::Orthographic => {
                let width = self.ortho_height * self.aspect_ratio;
                let height = self.ortho_height;
                let ortho = Orthographic3::new(
                    -width / 2.0,
                    width / 2.0,
                    -height / 2.0,
                    height / 2.0,
                    self.near,
                    self.far,
                );
                ortho.to_homogeneous()
            }
        };

        // Combine matrices
        self.view_proj_matrix = self.proj_matrix * self.view_matrix;
        self.dirty = false;
    }

    /// Get view matrix
    pub fn view_matrix(&mut self) -> [[f32; 4]; 4] {
        self.update_matrices();
        self.view_matrix.into()
    }

    /// Get projection matrix
    pub fn projection_matrix(&mut self) -> [[f32; 4]; 4] {
        self.update_matrices();
        self.proj_matrix.into()
    }

    /// Get combined view-projection matrix
    pub fn view_projection_matrix(&mut self) -> [[f32; 4]; 4] {
        self.update_matrices();
        self.view_proj_matrix.into()
    }

    /// Set position
    pub fn set_position(&mut self, position: Point3<f32>) {
        self.position = position;
        self.dirty = true;
    }

    /// Set target
    pub fn set_target(&mut self, target: Point3<f32>) {
        self.target = target;
        self.dirty = true;
    }

    /// Set up vector
    pub fn set_up(&mut self, up: Vector3<f32>) {
        self.up = up.normalize();
        self.dirty = true;
    }

    /// Set aspect ratio
    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
        self.dirty = true;
    }

    /// Set projection type
    pub fn set_projection_type(&mut self, projection_type: ProjectionType) {
        self.projection_type = projection_type;
        self.dirty = true;
    }

    /// Set field of view (degrees)
    pub fn set_fov(&mut self, fov: f32) {
        self.fov = fov.clamp(10.0, 120.0);
        self.dirty = true;
    }

    /// Set orthographic height
    pub fn set_ortho_height(&mut self, height: f32) {
        self.ortho_height = height.max(0.1);
        self.dirty = true;
    }

    /// Get position
    pub fn position(&self) -> Point3<f32> {
        self.position
    }

    /// Get target
    pub fn target(&self) -> Point3<f32> {
        self.target
    }

    /// Get up vector
    pub fn up(&self) -> Vector3<f32> {
        self.up
    }

    /// Get projection type
    pub fn projection_type(&self) -> ProjectionType {
        self.projection_type
    }

    /// Get aspect ratio
    pub fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }

    /// Get field of view (in degrees)
    pub fn fov(&self) -> f32 {
        self.fov
    }

    /// Get orthographic height
    pub fn ortho_height(&self) -> f32 {
        self.ortho_height
    }

    /// Get distance to target
    pub fn distance(&self) -> f32 {
        (self.position - self.target).norm()
    }

    /// Set to a named view
    pub fn set_named_view(&mut self, view: NamedView) {
        let distance = self.distance();
        let target = self.target;

        match view {
            NamedView::Top => {
                self.position = target + Vector3::new(0.0, distance, 0.0);
                self.up = Vector3::new(0.0, 0.0, -1.0);
            }
            NamedView::Bottom => {
                self.position = target + Vector3::new(0.0, -distance, 0.0);
                self.up = Vector3::new(0.0, 0.0, 1.0);
            }
            NamedView::Front => {
                self.position = target + Vector3::new(0.0, 0.0, distance);
                self.up = Vector3::new(0.0, 1.0, 0.0);
            }
            NamedView::Back => {
                self.position = target + Vector3::new(0.0, 0.0, -distance);
                self.up = Vector3::new(0.0, 1.0, 0.0);
            }
            NamedView::Left => {
                self.position = target + Vector3::new(distance, 0.0, 0.0);
                self.up = Vector3::new(0.0, 1.0, 0.0);
            }
            NamedView::Right => {
                self.position = target + Vector3::new(-distance, 0.0, 0.0);
                self.up = Vector3::new(0.0, 1.0, 0.0);
            }
            NamedView::IsometricSW => {
                let angle = 35.264_f32.to_radians(); // arctan(1/sqrt(2))
                let d = distance / angle.cos();
                self.position = target + Vector3::new(-d / 2.0_f32.sqrt(), d * angle.sin(), d / 2.0_f32.sqrt());
                self.up = Vector3::new(0.0, 1.0, 0.0);
            }
            NamedView::IsometricSE => {
                let angle = 35.264_f32.to_radians();
                let d = distance / angle.cos();
                self.position = target + Vector3::new(d / 2.0_f32.sqrt(), d * angle.sin(), d / 2.0_f32.sqrt());
                self.up = Vector3::new(0.0, 1.0, 0.0);
            }
            NamedView::IsometricNE => {
                let angle = 35.264_f32.to_radians();
                let d = distance / angle.cos();
                self.position = target + Vector3::new(d / 2.0_f32.sqrt(), d * angle.sin(), -d / 2.0_f32.sqrt());
                self.up = Vector3::new(0.0, 1.0, 0.0);
            }
            NamedView::IsometricNW => {
                let angle = 35.264_f32.to_radians();
                let d = distance / angle.cos();
                self.position = target + Vector3::new(-d / 2.0_f32.sqrt(), d * angle.sin(), -d / 2.0_f32.sqrt());
                self.up = Vector3::new(0.0, 1.0, 0.0);
            }
        }

        self.dirty = true;
    }
}

/// Camera controller for user interaction
pub struct CameraController {
    orbit_sensitivity: f32,
    pan_sensitivity: f32,
    zoom_sensitivity: f32,
    min_distance: f32,
    max_distance: f32,
}

impl CameraController {
    /// Create a new camera controller
    pub fn new() -> Self {
        Self {
            orbit_sensitivity: 0.005,
            pan_sensitivity: 0.01,
            zoom_sensitivity: 0.1,
            min_distance: 1.0,
            max_distance: 10000.0,
        }
    }

    /// Orbit the camera around the target
    pub fn orbit(&self, camera: &mut Camera, delta_x: f32, delta_y: f32) {
        let _distance = camera.distance();
        let target = camera.target();

        // Calculate spherical coordinates
        let offset = camera.position() - target;
        let radius = offset.norm();

        // Avoid gimbal lock
        let theta = offset.z.atan2(offset.x);
        let phi = (offset.y / radius).acos();

        // Apply rotation
        let new_theta = theta - delta_x * self.orbit_sensitivity;
        let new_phi = (phi + delta_y * self.orbit_sensitivity).clamp(0.01, PI - 0.01);

        // Convert back to Cartesian
        let new_offset = Vector3::new(
            radius * new_phi.sin() * new_theta.cos(),
            radius * new_phi.cos(),
            radius * new_phi.sin() * new_theta.sin(),
        );

        camera.set_position(target + new_offset);
    }

    /// Pan the camera
    pub fn pan(&self, camera: &mut Camera, delta_x: f32, delta_y: f32) {
        let view_dir = (camera.target() - camera.position()).normalize();
        let right = view_dir.cross(&camera.up()).normalize();
        let up = right.cross(&view_dir).normalize();

        let distance = camera.distance();
        let scale = distance * self.pan_sensitivity;

        let offset = right * delta_x * scale + up * delta_y * scale;

        camera.set_position(camera.position() + offset);
        camera.set_target(camera.target() + offset);
    }

    /// Zoom the camera
    pub fn zoom(&self, camera: &mut Camera, delta: f32) {
        match camera.projection_type() {
            ProjectionType::Perspective => {
                let view_dir = (camera.target() - camera.position()).normalize();
                let distance = camera.distance();
                let new_distance = (distance * (1.0 - delta * self.zoom_sensitivity))
                    .clamp(self.min_distance, self.max_distance);

                camera.set_position(camera.target() - view_dir * new_distance);
            }
            ProjectionType::Orthographic => {
                let new_height = camera.ortho_height() * (1.0 - delta * self.zoom_sensitivity);
                camera.set_ortho_height(new_height.clamp(0.1, 10000.0));
            }
        }
    }

    /// Zoom to fit a bounding box
    pub fn zoom_extents(
        &self,
        camera: &mut Camera,
        min: Point3<f32>,
        max: Point3<f32>,
    ) {
        let center = Point3::new(
            (min.x + max.x) / 2.0,
            (min.y + max.y) / 2.0,
            (min.z + max.z) / 2.0,
        );

        let size = Vector3::new(
            max.x - min.x,
            max.y - min.y,
            max.z - min.z,
        );

        let max_size = size.x.max(size.y).max(size.z);

        camera.set_target(center);

        match camera.projection_type() {
            ProjectionType::Perspective => {
                let fov_rad = camera.fov.to_radians();
                let distance = max_size / (2.0 * (fov_rad / 2.0).tan());
                let view_dir = (camera.position() - camera.target()).normalize();
                camera.set_position(center + view_dir * distance * 1.5);
            }
            ProjectionType::Orthographic => {
                camera.set_ortho_height(max_size * 1.2);
            }
        }
    }

    /// Set orbit sensitivity
    pub fn set_orbit_sensitivity(&mut self, sensitivity: f32) {
        self.orbit_sensitivity = sensitivity;
    }

    /// Set pan sensitivity
    pub fn set_pan_sensitivity(&mut self, sensitivity: f32) {
        self.pan_sensitivity = sensitivity;
    }

    /// Set zoom sensitivity
    pub fn set_zoom_sensitivity(&mut self, sensitivity: f32) {
        self.zoom_sensitivity = sensitivity;
    }
}

impl Default for CameraController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_creation() {
        let camera = Camera::new_orthographic(
            Point3::new(0.0, 0.0, 100.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            1.0,
        );

        assert_eq!(camera.projection_type(), ProjectionType::Orthographic);
        assert_eq!(camera.position(), Point3::new(0.0, 0.0, 100.0));
    }

    #[test]
    fn test_named_views() {
        let mut camera = Camera::new_orthographic(
            Point3::new(0.0, 0.0, 100.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            1.0,
        );

        camera.set_named_view(NamedView::Top);
        assert!(camera.position().y > 0.0);

        camera.set_named_view(NamedView::Front);
        assert!(camera.position().z > 0.0);
    }
}
