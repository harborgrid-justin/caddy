//! # Visibility Culling System
//!
//! Advanced culling algorithms for optimizing rendering performance.
//!
//! ## Features
//!
//! - Frustum culling for view-dependent visibility
//! - Occlusion culling for hidden surface removal
//! - Hierarchical culling for complex scenes
//! - Bounding volume tests (AABB, OBB, spheres)

use crate::core::math::{Matrix4, Vector3, Vector4};
use crate::core::primitives::{BoundingBox3, Point3, Plane};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Plane representation for frustum culling
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FrustumPlane {
    /// Plane normal
    pub normal: Vector3,

    /// Distance from origin
    pub distance: f32,
}

impl FrustumPlane {
    /// Create a new frustum plane
    pub fn new(normal: Vector3, distance: f32) -> Self {
        Self { normal, distance }
    }

    /// Create from four coefficients (ax + by + cz + d = 0)
    pub fn from_coefficients(a: f32, b: f32, c: f32, d: f32) -> Self {
        let normal = Vector3::new(a, b, c);
        let length = normal.magnitude();

        Self {
            normal: normal / length,
            distance: d / length,
        }
    }

    /// Calculate signed distance from point to plane
    pub fn distance_to_point(&self, point: &Point3) -> f32 {
        self.normal.dot(&point.coords) + self.distance
    }

    /// Test if point is on positive side of plane
    pub fn is_point_in_front(&self, point: &Point3) -> bool {
        self.distance_to_point(point) >= 0.0
    }

    /// Normalize the plane
    pub fn normalize(&mut self) {
        let length = self.normal.magnitude();
        if length > 0.0 {
            self.normal /= length;
            self.distance /= length;
        }
    }
}

/// View frustum for culling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewFrustum {
    /// Frustum planes: [left, right, bottom, top, near, far]
    pub planes: [FrustumPlane; 6],
}

impl ViewFrustum {
    /// Create a frustum from a view-projection matrix
    pub fn from_matrix(view_proj: &Matrix4) -> Self {
        // Extract frustum planes from view-projection matrix
        // Using Gribb-Hartmann method

        let m = view_proj;

        let planes = [
            // Left plane: m[3] + m[0]
            FrustumPlane::from_coefficients(
                m[(0, 3)] + m[(0, 0)],
                m[(1, 3)] + m[(1, 0)],
                m[(2, 3)] + m[(2, 0)],
                m[(3, 3)] + m[(3, 0)],
            ),
            // Right plane: m[3] - m[0]
            FrustumPlane::from_coefficients(
                m[(0, 3)] - m[(0, 0)],
                m[(1, 3)] - m[(1, 0)],
                m[(2, 3)] - m[(2, 0)],
                m[(3, 3)] - m[(3, 0)],
            ),
            // Bottom plane: m[3] + m[1]
            FrustumPlane::from_coefficients(
                m[(0, 3)] + m[(0, 1)],
                m[(1, 3)] + m[(1, 1)],
                m[(2, 3)] + m[(2, 1)],
                m[(3, 3)] + m[(3, 1)],
            ),
            // Top plane: m[3] - m[1]
            FrustumPlane::from_coefficients(
                m[(0, 3)] - m[(0, 1)],
                m[(1, 3)] - m[(1, 1)],
                m[(2, 3)] - m[(2, 1)],
                m[(3, 3)] - m[(3, 1)],
            ),
            // Near plane: m[3] + m[2]
            FrustumPlane::from_coefficients(
                m[(0, 3)] + m[(0, 2)],
                m[(1, 3)] + m[(1, 2)],
                m[(2, 3)] + m[(2, 2)],
                m[(3, 3)] + m[(3, 2)],
            ),
            // Far plane: m[3] - m[2]
            FrustumPlane::from_coefficients(
                m[(0, 3)] - m[(0, 2)],
                m[(1, 3)] - m[(1, 2)],
                m[(2, 3)] - m[(2, 2)],
                m[(3, 3)] - m[(3, 2)],
            ),
        ];

        Self { planes }
    }

    /// Test if a point is inside the frustum
    pub fn contains_point(&self, point: &Point3) -> bool {
        for plane in &self.planes {
            if !plane.is_point_in_front(point) {
                return false;
            }
        }
        true
    }

    /// Test if a sphere is inside the frustum
    pub fn contains_sphere(&self, center: &Point3, radius: f32) -> IntersectionResult {
        let mut intersects = false;

        for plane in &self.planes {
            let distance = plane.distance_to_point(center);

            if distance < -radius {
                return IntersectionResult::Outside;
            }

            if distance.abs() < radius {
                intersects = true;
            }
        }

        if intersects {
            IntersectionResult::Intersects
        } else {
            IntersectionResult::Inside
        }
    }

    /// Test if an AABB is inside the frustum
    pub fn contains_aabb(&self, bbox: &BoundingBox3) -> IntersectionResult {
        let mut result = IntersectionResult::Inside;

        for plane in &self.planes {
            // Get the positive vertex (furthest in the direction of the normal)
            let p_vertex = Point3::new(
                if plane.normal.x >= 0.0 {
                    bbox.max.x
                } else {
                    bbox.min.x
                },
                if plane.normal.y >= 0.0 {
                    bbox.max.y
                } else {
                    bbox.min.y
                },
                if plane.normal.z >= 0.0 {
                    bbox.max.z
                } else {
                    bbox.min.z
                },
            );

            // Get the negative vertex (closest in the direction of the normal)
            let n_vertex = Point3::new(
                if plane.normal.x >= 0.0 {
                    bbox.min.x
                } else {
                    bbox.max.x
                },
                if plane.normal.y >= 0.0 {
                    bbox.min.y
                } else {
                    bbox.max.y
                },
                if plane.normal.z >= 0.0 {
                    bbox.min.z
                } else {
                    bbox.max.z
                },
            );

            // If p-vertex is outside, the whole box is outside
            if plane.distance_to_point(&p_vertex) < 0.0 {
                return IntersectionResult::Outside;
            }

            // If n-vertex is outside, the box intersects the plane
            if plane.distance_to_point(&n_vertex) < 0.0 {
                result = IntersectionResult::Intersects;
            }
        }

        result
    }
}

/// Intersection test result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntersectionResult {
    /// Completely outside
    Outside,

    /// Partially inside (intersects)
    Intersects,

    /// Completely inside
    Inside,
}

/// Frustum culler
pub struct FrustumCuller {
    /// Current view frustum
    frustum: ViewFrustum,

    /// Culling statistics
    stats: CullingStatistics,
}

impl FrustumCuller {
    /// Create a new frustum culler
    pub fn new(view_proj: &Matrix4) -> Self {
        Self {
            frustum: ViewFrustum::from_matrix(view_proj),
            stats: CullingStatistics::default(),
        }
    }

    /// Update the frustum
    pub fn update(&mut self, view_proj: &Matrix4) {
        self.frustum = ViewFrustum::from_matrix(view_proj);
        self.stats.reset();
    }

    /// Test if a point is visible
    pub fn test_point(&mut self, point: &Point3) -> bool {
        self.stats.tests_performed += 1;

        let visible = self.frustum.contains_point(point);

        if !visible {
            self.stats.objects_culled += 1;
        }

        visible
    }

    /// Test if a sphere is visible
    pub fn test_sphere(&mut self, center: &Point3, radius: f32) -> IntersectionResult {
        self.stats.tests_performed += 1;

        let result = self.frustum.contains_sphere(center, radius);

        if result == IntersectionResult::Outside {
            self.stats.objects_culled += 1;
        }

        result
    }

    /// Test if an AABB is visible
    pub fn test_aabb(&mut self, bbox: &BoundingBox3) -> IntersectionResult {
        self.stats.tests_performed += 1;

        let result = self.frustum.contains_aabb(bbox);

        if result == IntersectionResult::Outside {
            self.stats.objects_culled += 1;
        }

        result
    }

    /// Get culling statistics
    pub fn statistics(&self) -> &CullingStatistics {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats.reset();
    }
}

/// Occlusion culler using hierarchical Z-buffer
pub struct OcclusionCuller {
    /// Depth pyramid (mipmapped depth buffer)
    depth_pyramid: Vec<Vec<f32>>,

    /// Pyramid dimensions at each level
    pyramid_sizes: Vec<(usize, usize)>,

    /// Culling statistics
    stats: CullingStatistics,

    /// Is enabled
    enabled: bool,
}

impl OcclusionCuller {
    /// Create a new occlusion culler
    pub fn new(width: usize, height: usize) -> Self {
        let mut pyramid_sizes = Vec::new();
        let mut w = width;
        let mut h = height;

        // Build mipmap chain
        while w > 1 && h > 1 {
            pyramid_sizes.push((w, h));
            w = (w / 2).max(1);
            h = (h / 2).max(1);
        }

        pyramid_sizes.push((w, h));

        let depth_pyramid = pyramid_sizes
            .iter()
            .map(|(w, h)| vec![f32::MAX; w * h])
            .collect();

        Self {
            depth_pyramid,
            pyramid_sizes,
            stats: CullingStatistics::default(),
            enabled: true,
        }
    }

    /// Update the depth pyramid from a depth buffer
    pub fn update(&mut self, depth_buffer: &[f32], width: usize, height: usize) {
        if !self.enabled {
            return;
        }

        // Copy base level
        if !self.depth_pyramid.is_empty() && self.pyramid_sizes[0] == (width, height) {
            self.depth_pyramid[0].copy_from_slice(depth_buffer);
        }

        // Build mipmap chain (max reduction for conservative occlusion)
        for level in 1..self.depth_pyramid.len() {
            let (prev_w, prev_h) = self.pyramid_sizes[level - 1];
            let (curr_w, curr_h) = self.pyramid_sizes[level];

            for y in 0..curr_h {
                for x in 0..curr_w {
                    let src_x = x * 2;
                    let src_y = y * 2;

                    // Sample 2x2 region and take maximum depth
                    let mut max_depth = f32::MIN;

                    for dy in 0..2 {
                        for dx in 0..2 {
                            let sx = (src_x + dx).min(prev_w - 1);
                            let sy = (src_y + dy).min(prev_h - 1);
                            let idx = sy * prev_w + sx;

                            if idx < self.depth_pyramid[level - 1].len() {
                                max_depth = max_depth.max(self.depth_pyramid[level - 1][idx]);
                            }
                        }
                    }

                    let dst_idx = y * curr_w + x;
                    if dst_idx < self.depth_pyramid[level].len() {
                        self.depth_pyramid[level][dst_idx] = max_depth;
                    }
                }
            }
        }

        self.stats.reset();
    }

    /// Test if a bounding box is occluded
    pub fn test_occlusion(
        &mut self,
        bbox: &BoundingBox3,
        view_proj: &Matrix4,
    ) -> bool {
        if !self.enabled {
            return false;
        }

        self.stats.tests_performed += 1;

        // Project bounding box to screen space
        let corners = [
            Point3::new(bbox.min.x, bbox.min.y, bbox.min.z),
            Point3::new(bbox.max.x, bbox.min.y, bbox.min.z),
            Point3::new(bbox.min.x, bbox.max.y, bbox.min.z),
            Point3::new(bbox.max.x, bbox.max.y, bbox.min.z),
            Point3::new(bbox.min.x, bbox.min.y, bbox.max.z),
            Point3::new(bbox.max.x, bbox.min.y, bbox.max.z),
            Point3::new(bbox.min.x, bbox.max.y, bbox.max.z),
            Point3::new(bbox.max.x, bbox.max.y, bbox.max.z),
        ];

        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;
        let mut min_z = f32::MAX;

        // Project all corners
        for corner in &corners {
            let projected = view_proj.transform_point(corner);

            // Convert to NDC
            if projected.z != 0.0 {
                let ndc_x = projected.x / projected.z;
                let ndc_y = projected.y / projected.z;
                let ndc_z = projected.z;

                min_x = min_x.min(ndc_x);
                max_x = max_x.max(ndc_x);
                min_y = min_y.min(ndc_y);
                max_y = max_y.max(ndc_y);
                min_z = min_z.min(ndc_z);
            }
        }

        // Check if behind camera
        if min_z < 0.0 {
            return false;
        }

        // Convert to screen space
        let (width, height) = self.pyramid_sizes[0];
        let screen_min_x = ((min_x + 1.0) * 0.5 * width as f32).floor() as usize;
        let screen_max_x = ((max_x + 1.0) * 0.5 * width as f32).ceil() as usize;
        let screen_min_y = ((1.0 - max_y) * 0.5 * height as f32).floor() as usize;
        let screen_max_y = ((1.0 - min_y) * 0.5 * height as f32).ceil() as usize;

        // Clamp to screen bounds
        let screen_min_x = screen_min_x.min(width - 1);
        let screen_max_x = screen_max_x.min(width);
        let screen_min_y = screen_min_y.min(height - 1);
        let screen_max_y = screen_max_y.min(height);

        // Calculate appropriate mip level based on screen area
        let screen_width = (screen_max_x - screen_min_x).max(1);
        let screen_height = (screen_max_y - screen_min_y).max(1);
        let area = screen_width * screen_height;

        let mip_level = (area as f32).log2().floor() as usize;
        let mip_level = mip_level.min(self.depth_pyramid.len() - 1);

        // Sample depth pyramid at appropriate level
        let (level_w, level_h) = self.pyramid_sizes[mip_level];
        let scale_x = level_w as f32 / width as f32;
        let scale_y = level_h as f32 / height as f32;

        let sample_x = ((screen_min_x as f32 * scale_x) as usize).min(level_w - 1);
        let sample_y = ((screen_min_y as f32 * scale_y) as usize).min(level_h - 1);

        let idx = sample_y * level_w + sample_x;
        if idx < self.depth_pyramid[mip_level].len() {
            let stored_depth = self.depth_pyramid[mip_level][idx];

            // If object's nearest depth is greater than stored depth, it's occluded
            let is_occluded = min_z > stored_depth;

            if is_occluded {
                self.stats.objects_culled += 1;
            }

            is_occluded
        } else {
            false
        }
    }

    /// Enable or disable occlusion culling
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Get statistics
    pub fn statistics(&self) -> &CullingStatistics {
        &self.stats
    }
}

/// Culling statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CullingStatistics {
    /// Number of culling tests performed
    pub tests_performed: u32,

    /// Number of objects culled
    pub objects_culled: u32,

    /// Number of objects passed
    pub objects_passed: u32,
}

impl CullingStatistics {
    /// Reset statistics
    pub fn reset(&mut self) {
        self.tests_performed = 0;
        self.objects_culled = 0;
        self.objects_passed = 0;
    }

    /// Calculate culling efficiency (percentage culled)
    pub fn efficiency(&self) -> f32 {
        if self.tests_performed == 0 {
            0.0
        } else {
            (self.objects_culled as f32 / self.tests_performed as f32) * 100.0
        }
    }
}

/// Combined visibility tester
pub struct VisibilityTester {
    /// Frustum culler
    frustum_culler: FrustumCuller,

    /// Occlusion culler
    occlusion_culler: Option<OcclusionCuller>,

    /// Enable frustum culling
    enable_frustum: bool,

    /// Enable occlusion culling
    enable_occlusion: bool,
}

impl VisibilityTester {
    /// Create a new visibility tester
    pub fn new(view_proj: &Matrix4, viewport_size: Option<(usize, usize)>) -> Self {
        let occlusion_culler = viewport_size.map(|(w, h)| OcclusionCuller::new(w, h));

        Self {
            frustum_culler: FrustumCuller::new(view_proj),
            occlusion_culler,
            enable_frustum: true,
            enable_occlusion: true,
        }
    }

    /// Update the tester
    pub fn update(&mut self, view_proj: &Matrix4, depth_buffer: Option<&[f32]>, width: usize, height: usize) {
        self.frustum_culler.update(view_proj);

        if let (Some(culler), Some(buffer)) = (&mut self.occlusion_culler, depth_buffer) {
            culler.update(buffer, width, height);
        }
    }

    /// Test if an AABB is visible
    pub fn is_visible(&mut self, bbox: &BoundingBox3, view_proj: &Matrix4) -> bool {
        // First test frustum culling (fast)
        if self.enable_frustum {
            let frustum_result = self.frustum_culler.test_aabb(bbox);

            if frustum_result == IntersectionResult::Outside {
                return false;
            }
        }

        // Then test occlusion culling (slower)
        if self.enable_occlusion {
            if let Some(culler) = &mut self.occlusion_culler {
                if culler.test_occlusion(bbox, view_proj) {
                    return false;
                }
            }
        }

        true
    }

    /// Get combined statistics
    pub fn statistics(&self) -> CombinedCullingStats {
        let occlusion_stats = self
            .occlusion_culler
            .as_ref()
            .map(|c| c.statistics().clone())
            .unwrap_or_default();

        CombinedCullingStats {
            frustum: self.frustum_culler.statistics().clone(),
            occlusion: occlusion_stats,
        }
    }
}

/// Combined culling statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinedCullingStats {
    /// Frustum culling stats
    pub frustum: CullingStatistics,

    /// Occlusion culling stats
    pub occlusion: CullingStatistics,
}

/// Culling result for rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CullingResult {
    /// Visible object IDs
    pub visible_objects: Vec<u32>,

    /// Culled object IDs
    pub culled_objects: Vec<u32>,

    /// Statistics
    pub statistics: CombinedCullingStats,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frustum_plane() {
        let plane = FrustumPlane::new(Vector3::new(0.0, 1.0, 0.0), 1.0);
        let point_above = Point3::new(0.0, 2.0, 0.0);
        let point_below = Point3::new(0.0, 0.0, 0.0);

        assert!(plane.is_point_in_front(&point_above));
        assert!(!plane.is_point_in_front(&point_below));
    }

    #[test]
    fn test_frustum_contains_point() {
        let view_proj = Matrix4::identity();
        let frustum = ViewFrustum::from_matrix(&view_proj);
        let point = Point3::new(0.0, 0.0, 0.0);

        // With identity matrix, origin should be inside
        assert!(frustum.contains_point(&point));
    }

    #[test]
    fn test_frustum_culler() {
        let view_proj = Matrix4::identity();
        let mut culler = FrustumCuller::new(&view_proj);

        let point = Point3::new(0.0, 0.0, 0.0);
        assert!(culler.test_point(&point));

        let stats = culler.statistics();
        assert_eq!(stats.tests_performed, 1);
    }

    #[test]
    fn test_culling_statistics() {
        let mut stats = CullingStatistics::default();

        stats.tests_performed = 100;
        stats.objects_culled = 75;

        assert_eq!(stats.efficiency(), 75.0);
    }

    #[test]
    fn test_occlusion_culler() {
        let culler = OcclusionCuller::new(1024, 768);

        assert!(culler.pyramid_sizes.len() > 1);
        assert_eq!(culler.pyramid_sizes[0], (1024, 768));
    }
}
