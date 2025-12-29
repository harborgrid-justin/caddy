//! # Camera System
//!
//! Advanced camera system supporting multiple projection modes and view transformations.
//!
//! ## Features
//!
//! - Orthographic, perspective, and isometric projections
//! - Smooth camera transitions and animations
//! - Camera controls (pan, zoom, rotate, orbit)
//! - View frustum calculation
//! - Screen-to-world and world-to-screen coordinate transformations

use crate::core::math::{Matrix4, Quaternion, Vector3};
use crate::core::primitives::{Point2, Point3};
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// Camera projection mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CameraMode {
    /// Orthographic projection (parallel lines stay parallel)
    Orthographic,

    /// Perspective projection (realistic depth)
    Perspective,

    /// Isometric projection (30° angle, no perspective)
    Isometric,
}

/// Camera projection parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CameraProjection {
    /// Orthographic projection
    Orthographic {
        /// Left plane
        left: f32,
        /// Right plane
        right: f32,
        /// Bottom plane
        bottom: f32,
        /// Top plane
        top: f32,
        /// Near clipping plane
        near: f32,
        /// Far clipping plane
        far: f32,
    },

    /// Perspective projection
    Perspective {
        /// Field of view in radians
        fov: f32,
        /// Aspect ratio (width / height)
        aspect: f32,
        /// Near clipping plane
        near: f32,
        /// Far clipping plane
        far: f32,
    },

    /// Isometric projection
    Isometric {
        /// Scale factor
        scale: f32,
        /// Aspect ratio
        aspect: f32,
        /// Near clipping plane
        near: f32,
        /// Far clipping plane
        far: f32,
    },
}

impl CameraProjection {
    /// Create a perspective projection
    pub fn perspective(fov: f32, aspect: f32, near: f32, far: f32) -> Self {
        Self::Perspective {
            fov,
            aspect,
            near,
            far,
        }
    }

    /// Create an orthographic projection
    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        Self::Orthographic {
            left,
            right,
            bottom,
            top,
            near,
            far,
        }
    }

    /// Create an isometric projection
    pub fn isometric(scale: f32, aspect: f32, near: f32, far: f32) -> Self {
        Self::Isometric {
            scale,
            aspect,
            near,
            far,
        }
    }

    /// Get the projection matrix
    pub fn matrix(&self) -> Matrix4 {
        match self {
            CameraProjection::Orthographic {
                left,
                right,
                bottom,
                top,
                near,
                far,
            } => {
                let width = right - left;
                let height = top - bottom;
                let depth = far - near;

                Matrix4::new(
                    2.0 / width, 0.0, 0.0, -(right + left) / width,
                    0.0, 2.0 / height, 0.0, -(top + bottom) / height,
                    0.0, 0.0, -2.0 / depth, -(far + near) / depth,
                    0.0, 0.0, 0.0, 1.0,
                )
            }
            CameraProjection::Perspective {
                fov,
                aspect,
                near,
                far,
            } => {
                let f = 1.0 / (fov / 2.0).tan();
                let depth = near - far;

                Matrix4::new(
                    f / aspect, 0.0, 0.0, 0.0,
                    0.0, f, 0.0, 0.0,
                    0.0, 0.0, (far + near) / depth, (2.0 * far * near) / depth,
                    0.0, 0.0, -1.0, 0.0,
                )
            }
            CameraProjection::Isometric {
                scale,
                aspect,
                near,
                far,
            } => {
                // Isometric uses orthographic with specific angles
                let half_width = scale * aspect;
                let half_height = scale;

                Matrix4::new(
                    1.0 / half_width, 0.0, 0.0, 0.0,
                    0.0, 1.0 / half_height, 0.0, 0.0,
                    0.0, 0.0, -2.0 / (far - near), -(far + near) / (far - near),
                    0.0, 0.0, 0.0, 1.0,
                )
            }
        }
    }

    /// Update aspect ratio
    pub fn set_aspect_ratio(&mut self, aspect: f32) {
        match self {
            CameraProjection::Perspective { aspect: a, .. } => *a = aspect,
            CameraProjection::Isometric { aspect: a, .. } => *a = aspect,
            CameraProjection::Orthographic {
                left,
                right,
                bottom,
                top,
                ..
            } => {
                let height = *top - *bottom;
                let width = height * aspect;
                *left = -width / 2.0;
                *right = width / 2.0;
            }
        }
    }

    /// Get near clipping plane
    pub fn near(&self) -> f32 {
        match self {
            CameraProjection::Orthographic { near, .. }
            | CameraProjection::Perspective { near, .. }
            | CameraProjection::Isometric { near, .. } => *near,
        }
    }

    /// Get far clipping plane
    pub fn far(&self) -> f32 {
        match self {
            CameraProjection::Orthographic { far, .. }
            | CameraProjection::Perspective { far, .. }
            | CameraProjection::Isometric { far, .. } => *far,
        }
    }
}

/// Camera structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    /// Camera position in world space
    pub position: Point3,

    /// Camera target (look-at point)
    pub target: Point3,

    /// Up vector
    pub up: Vector3,

    /// Camera orientation (quaternion)
    pub rotation: Quaternion,

    /// Projection parameters
    pub projection: CameraProjection,

    /// Camera mode
    pub mode: CameraMode,

    /// Zoom level (for orthographic/isometric)
    pub zoom: f32,

    /// Minimum zoom
    pub min_zoom: f32,

    /// Maximum zoom
    pub max_zoom: f32,
}

impl Camera {
    /// Create a new camera
    pub fn new(mode: CameraMode, aspect: f32) -> Self {
        let projection = match mode {
            CameraMode::Perspective => {
                CameraProjection::perspective(PI / 4.0, aspect, 0.1, 1000.0)
            }
            CameraMode::Orthographic => {
                let height = 10.0;
                let width = height * aspect;
                CameraProjection::orthographic(
                    -width / 2.0,
                    width / 2.0,
                    -height / 2.0,
                    height / 2.0,
                    0.1,
                    1000.0,
                )
            }
            CameraMode::Isometric => CameraProjection::isometric(10.0, aspect, 0.1, 1000.0),
        };

        Self {
            position: Point3::new(0.0, 0.0, 10.0),
            target: Point3::new(0.0, 0.0, 0.0),
            up: Vector3::new(0.0, 1.0, 0.0),
            rotation: Quaternion::identity(),
            projection,
            mode,
            zoom: 1.0,
            min_zoom: 0.01,
            max_zoom: 100.0,
        }
    }

    /// Get the view matrix
    pub fn view_matrix(&self) -> Matrix4 {
        let forward = (self.target - self.position).normalize();
        let right = forward.cross(&self.up).normalize();
        let up = right.cross(&forward);

        let tx = -right.dot(&self.position.coords);
        let ty = -up.dot(&self.position.coords);
        let tz = forward.dot(&self.position.coords);

        Matrix4::new(
            right.x, right.y, right.z, tx,
            up.x, up.y, up.z, ty,
            -forward.x, -forward.y, -forward.z, tz,
            0.0, 0.0, 0.0, 1.0,
        )
    }

    /// Get the projection matrix
    pub fn projection_matrix(&self) -> Matrix4 {
        self.projection.matrix()
    }

    /// Get the combined view-projection matrix
    pub fn view_projection_matrix(&self) -> Matrix4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Pan the camera (move in screen space)
    pub fn pan(&mut self, delta_x: f32, delta_y: f32) {
        let forward = (self.target - self.position).normalize();
        let right = forward.cross(&self.up).normalize();
        let up = right.cross(&forward);

        let pan_speed = match self.mode {
            CameraMode::Perspective => {
                let distance = (self.target - self.position).magnitude();
                distance * 0.001
            }
            _ => 0.01 * self.zoom,
        };

        let offset = right * delta_x * pan_speed + up * delta_y * pan_speed;

        self.position += offset;
        self.target += offset;
    }

    /// Zoom the camera
    pub fn zoom_camera(&mut self, delta: f32) {
        match self.mode {
            CameraMode::Perspective => {
                let direction = (self.target - self.position).normalize();
                let distance = (self.target - self.position).magnitude();
                let new_distance = (distance - delta * distance * 0.1).max(0.1);

                self.position = self.target - direction * new_distance;
            }
            _ => {
                self.zoom = (self.zoom + delta * 0.1).clamp(self.min_zoom, self.max_zoom);

                // Update projection
                match &mut self.projection {
                    CameraProjection::Orthographic {
                        left,
                        right,
                        bottom,
                        top,
                        ..
                    } => {
                        let base_size = 10.0;
                        let height = base_size * self.zoom;
                        let aspect = (*right - *left) / (*top - *bottom);
                        let width = height * aspect;

                        *left = -width / 2.0;
                        *right = width / 2.0;
                        *bottom = -height / 2.0;
                        *top = height / 2.0;
                    }
                    CameraProjection::Isometric { scale, .. } => {
                        *scale = 10.0 * self.zoom;
                    }
                    _ => {}
                }
            }
        }
    }

    /// Rotate the camera around the target (orbit)
    pub fn orbit(&mut self, delta_yaw: f32, delta_pitch: f32) {
        let offset = self.position - self.target;
        let distance = offset.magnitude();

        // Convert to spherical coordinates
        let mut phi = offset.y.atan2((offset.x.powi(2) + offset.z.powi(2)).sqrt());
        let mut theta = offset.z.atan2(offset.x);

        // Apply rotation
        theta += delta_yaw;
        phi = (phi + delta_pitch).clamp(-PI / 2.0 + 0.01, PI / 2.0 - 0.01);

        // Convert back to Cartesian
        let x = distance * phi.cos() * theta.cos();
        let y = distance * phi.sin();
        let z = distance * phi.cos() * theta.sin();

        self.position = self.target + Vector3::new(x, y, z);
    }

    /// Rotate the camera around its position (look around)
    pub fn rotate(&mut self, delta_yaw: f32, delta_pitch: f32) {
        let forward = (self.target - self.position).normalize();
        let right = forward.cross(&self.up).normalize();

        // Apply yaw rotation
        let yaw_rotation = Quaternion::from_axis_angle(&self.up, delta_yaw);
        let pitch_rotation = Quaternion::from_axis_angle(&right, delta_pitch);

        let combined_rotation = yaw_rotation * pitch_rotation;
        let new_forward = combined_rotation.transform_vector(&forward);

        let distance = (self.target - self.position).magnitude();
        self.target = self.position + new_forward * distance;
    }

    /// Set camera to look at a specific point
    pub fn look_at(&mut self, eye: Point3, target: Point3, up: Vector3) {
        self.position = eye;
        self.target = target;
        self.up = up;
    }

    /// Set aspect ratio
    pub fn set_aspect_ratio(&mut self, aspect: f32) {
        self.projection.set_aspect_ratio(aspect);
    }

    /// Convert screen coordinates to world ray
    pub fn screen_to_world_ray(&self, screen_pos: Point2, viewport_size: (f32, f32)) -> (Point3, Vector3) {
        let (width, height) = viewport_size;

        // Normalize screen coordinates to [-1, 1]
        let ndc_x = (2.0 * screen_pos.x) / width - 1.0;
        let ndc_y = 1.0 - (2.0 * screen_pos.y) / height;

        // Create clip space position
        let clip_near = Vector3::new(ndc_x, ndc_y, -1.0);
        let clip_far = Vector3::new(ndc_x, ndc_y, 1.0);

        // Get inverse view-projection matrix
        let view_proj = self.view_projection_matrix();
        let inv_view_proj = view_proj.try_inverse().unwrap_or(Matrix4::identity());

        // Transform to world space
        let near = inv_view_proj.transform_point(&Point3::from(clip_near));
        let far = inv_view_proj.transform_point(&Point3::from(clip_far));

        let direction = (far - near).normalize();

        (near, direction)
    }

    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, world_pos: Point3, viewport_size: (f32, f32)) -> Option<Point2> {
        let (width, height) = viewport_size;

        let view_proj = self.view_projection_matrix();
        let clip_pos = view_proj.transform_point(&world_pos);

        // Check if behind camera
        if clip_pos.z < 0.0 {
            return None;
        }

        // Convert to NDC
        let ndc_x = clip_pos.x / clip_pos.z;
        let ndc_y = clip_pos.y / clip_pos.z;

        // Convert to screen coordinates
        let screen_x = (ndc_x + 1.0) * width / 2.0;
        let screen_y = (1.0 - ndc_y) * height / 2.0;

        Some(Point2::new(screen_x, screen_y))
    }

    /// Get camera frustum planes for culling
    pub fn frustum_planes(&self) -> [Vector3; 6] {
        let view_proj = self.view_projection_matrix();

        // Extract frustum planes from view-projection matrix
        // Format: [left, right, bottom, top, near, far]
        [
            Vector3::new(
                view_proj[(0, 3)] + view_proj[(0, 0)],
                view_proj[(1, 3)] + view_proj[(1, 0)],
                view_proj[(2, 3)] + view_proj[(2, 0)],
            ).normalize(),
            Vector3::new(
                view_proj[(0, 3)] - view_proj[(0, 0)],
                view_proj[(1, 3)] - view_proj[(1, 0)],
                view_proj[(2, 3)] - view_proj[(2, 0)],
            ).normalize(),
            Vector3::new(
                view_proj[(0, 3)] + view_proj[(0, 1)],
                view_proj[(1, 3)] + view_proj[(1, 1)],
                view_proj[(2, 3)] + view_proj[(2, 1)],
            ).normalize(),
            Vector3::new(
                view_proj[(0, 3)] - view_proj[(0, 1)],
                view_proj[(1, 3)] - view_proj[(1, 1)],
                view_proj[(2, 3)] - view_proj[(2, 1)],
            ).normalize(),
            Vector3::new(
                view_proj[(0, 3)] + view_proj[(0, 2)],
                view_proj[(1, 3)] + view_proj[(1, 2)],
                view_proj[(2, 3)] + view_proj[(2, 2)],
            ).normalize(),
            Vector3::new(
                view_proj[(0, 3)] - view_proj[(0, 2)],
                view_proj[(1, 3)] - view_proj[(1, 2)],
                view_proj[(2, 3)] - view_proj[(2, 2)],
            ).normalize(),
        ]
    }
}

/// Viewport camera wrapper with additional state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportCamera {
    /// Underlying camera
    pub camera: Camera,

    /// Is camera animated
    pub animated: bool,

    /// Animation duration in seconds
    pub animation_duration: f32,

    /// Animation elapsed time
    pub animation_elapsed: f32,

    /// Animation start position
    pub animation_start_pos: Point3,

    /// Animation target position
    pub animation_target_pos: Point3,
}

impl ViewportCamera {
    /// Create a new viewport camera
    pub fn new(mode: CameraMode, aspect: f32) -> Self {
        Self {
            camera: Camera::new(mode, aspect),
            animated: false,
            animation_duration: 0.3,
            animation_elapsed: 0.0,
            animation_start_pos: Point3::new(0.0, 0.0, 0.0),
            animation_target_pos: Point3::new(0.0, 0.0, 0.0),
        }
    }

    /// Animate to a new position
    pub fn animate_to(&mut self, target: Point3, duration: f32) {
        self.animation_start_pos = self.camera.position;
        self.animation_target_pos = target;
        self.animation_duration = duration;
        self.animation_elapsed = 0.0;
        self.animated = true;
    }

    /// Update animation
    pub fn update(&mut self, delta_time: f32) {
        if !self.animated {
            return;
        }

        self.animation_elapsed += delta_time;

        if self.animation_elapsed >= self.animation_duration {
            self.camera.position = self.animation_target_pos;
            self.animated = false;
        } else {
            // Smooth interpolation using ease-in-out
            let t = self.animation_elapsed / self.animation_duration;
            let smoothed_t = t * t * (3.0 - 2.0 * t);

            self.camera.position = self.animation_start_pos.coords.lerp(
                &self.animation_target_pos.coords,
                smoothed_t,
            ).into();
        }
    }

    /// Delegate to underlying camera
    pub fn set_aspect_ratio(&mut self, aspect: f32) {
        self.camera.set_aspect_ratio(aspect);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_creation() {
        let camera = Camera::new(CameraMode::Perspective, 16.0 / 9.0);
        assert_eq!(camera.mode, CameraMode::Perspective);
        assert_eq!(camera.zoom, 1.0);
    }

    #[test]
    fn test_camera_pan() {
        let mut camera = Camera::new(CameraMode::Orthographic, 1.0);
        let initial_pos = camera.position;

        camera.pan(1.0, 0.0);

        assert_ne!(camera.position, initial_pos);
    }

    #[test]
    fn test_camera_zoom() {
        let mut camera = Camera::new(CameraMode::Orthographic, 1.0);
        let initial_zoom = camera.zoom;

        camera.zoom_camera(1.0);

        assert_ne!(camera.zoom, initial_zoom);
    }

    #[test]
    fn test_projection_matrices() {
        let camera = Camera::new(CameraMode::Perspective, 16.0 / 9.0);

        let view_matrix = camera.view_matrix();
        let proj_matrix = camera.projection_matrix();
        let view_proj = camera.view_projection_matrix();

        // Matrices should not be identity
        assert_ne!(view_matrix, Matrix4::identity());
        assert_ne!(proj_matrix, Matrix4::identity());
        assert_ne!(view_proj, Matrix4::identity());
    }

    #[test]
    fn test_viewport_camera_animation() {
        let mut viewport_camera = ViewportCamera::new(CameraMode::Perspective, 1.0);

        let target = Point3::new(10.0, 10.0, 10.0);
        viewport_camera.animate_to(target, 1.0);

        assert!(viewport_camera.animated);

        viewport_camera.update(1.5);

        assert!(!viewport_camera.animated);
        assert_eq!(viewport_camera.camera.position, target);
    }
}
