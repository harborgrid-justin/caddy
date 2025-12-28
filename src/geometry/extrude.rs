//! Extrusion and Sweep Operations
//!
//! This module provides complete implementations of extrusion operations including
//! linear extrusion, path extrusion, revolution, sweeping, and lofting.

use super::mesh::{TriangleMesh, TriangleFace, Vertex};
use nalgebra::{Point2, Point3, Vector3, Rotation3, Unit};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// 2D profile for extrusion operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile2D {
    /// Points defining the profile in 2D (in XY plane)
    pub points: Vec<Point2<f64>>,
    /// Whether the profile is closed
    pub closed: bool,
}

impl Profile2D {
    /// Creates a new profile
    pub fn new(points: Vec<Point2<f64>>, closed: bool) -> Self {
        assert!(points.len() >= 2, "Profile must have at least 2 points");
        Self { points, closed }
    }

    /// Creates a rectangular profile
    pub fn rectangle(width: f64, height: f64) -> Self {
        Self::new(
            vec![
                Point2::new(-width / 2.0, -height / 2.0),
                Point2::new(width / 2.0, -height / 2.0),
                Point2::new(width / 2.0, height / 2.0),
                Point2::new(-width / 2.0, height / 2.0),
            ],
            true,
        )
    }

    /// Creates a circular profile
    pub fn circle(radius: f64, segments: usize) -> Self {
        let mut points = Vec::with_capacity(segments);

        for i in 0..segments {
            let angle = 2.0 * PI * (i as f64) / (segments as f64);
            points.push(Point2::new(radius * angle.cos(), radius * angle.sin()));
        }

        Self::new(points, true)
    }

    /// Creates an elliptical profile
    pub fn ellipse(radius_x: f64, radius_y: f64, segments: usize) -> Self {
        let mut points = Vec::with_capacity(segments);

        for i in 0..segments {
            let angle = 2.0 * PI * (i as f64) / (segments as f64);
            points.push(Point2::new(
                radius_x * angle.cos(),
                radius_y * angle.sin(),
            ));
        }

        Self::new(points, true)
    }

    /// Creates a regular polygon profile
    pub fn regular_polygon(radius: f64, sides: usize) -> Self {
        assert!(sides >= 3, "Polygon must have at least 3 sides");

        let mut points = Vec::with_capacity(sides);

        for i in 0..sides {
            let angle = 2.0 * PI * (i as f64) / (sides as f64);
            points.push(Point2::new(radius * angle.cos(), radius * angle.sin()));
        }

        Self::new(points, true)
    }

    /// Converts 2D points to 3D points in the XY plane
    pub fn to_3d(&self, z: f64) -> Vec<Point3<f64>> {
        self.points
            .iter()
            .map(|p| Point3::new(p.x, p.y, z))
            .collect()
    }

    /// Calculates the area of the profile (for closed profiles)
    pub fn area(&self) -> f64 {
        if !self.closed || self.points.len() < 3 {
            return 0.0;
        }

        let mut area = 0.0;
        let n = self.points.len();

        for i in 0..n {
            let j = (i + 1) % n;
            area += self.points[i].x * self.points[j].y;
            area -= self.points[j].x * self.points[i].y;
        }

        area.abs() / 2.0
    }
}

/// 3D path for sweep and extrusion operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Path3D {
    /// Points defining the path
    pub points: Vec<Point3<f64>>,
    /// Tangent vectors at each point (optional)
    pub tangents: Option<Vec<Vector3<f64>>>,
}

impl Path3D {
    /// Creates a new path
    pub fn new(points: Vec<Point3<f64>>) -> Self {
        assert!(points.len() >= 2, "Path must have at least 2 points");
        Self {
            points,
            tangents: None,
        }
    }

    /// Creates a straight line path
    pub fn line(start: Point3<f64>, end: Point3<f64>) -> Self {
        Self::new(vec![start, end])
    }

    /// Creates a circular arc path
    pub fn arc(center: Point3<f64>, radius: f64, start_angle: f64, end_angle: f64, segments: usize) -> Self {
        let mut points = Vec::with_capacity(segments + 1);

        for i in 0..=segments {
            let t = i as f64 / segments as f64;
            let angle = start_angle + (end_angle - start_angle) * t;
            points.push(Point3::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
                center.z,
            ));
        }

        Self::new(points)
    }

    /// Creates a helix path
    pub fn helix(radius: f64, pitch: f64, turns: f64, segments_per_turn: usize) -> Self {
        let total_segments = (turns * segments_per_turn as f64) as usize;
        let mut points = Vec::with_capacity(total_segments + 1);

        for i in 0..=total_segments {
            let t = i as f64 / segments_per_turn as f64;
            let angle = 2.0 * PI * t;
            let z = pitch * t;

            points.push(Point3::new(
                radius * angle.cos(),
                radius * angle.sin(),
                z,
            ));
        }

        Self::new(points)
    }

    /// Computes tangent vectors at each point
    pub fn compute_tangents(&mut self) {
        let mut tangents = Vec::with_capacity(self.points.len());

        for i in 0..self.points.len() {
            let tangent = if i == 0 {
                // First point: use forward difference
                (self.points[1] - self.points[0]).normalize()
            } else if i == self.points.len() - 1 {
                // Last point: use backward difference
                (self.points[i] - self.points[i - 1]).normalize()
            } else {
                // Middle points: use central difference
                (self.points[i + 1] - self.points[i - 1]).normalize()
            };

            tangents.push(tangent);
        }

        self.tangents = Some(tangents);
    }

    /// Returns the total length of the path
    pub fn length(&self) -> f64 {
        let mut length = 0.0;

        for i in 0..self.points.len() - 1 {
            length += nalgebra::distance(&self.points[i], &self.points[i + 1]);
        }

        length
    }
}

/// Linear extrusion operation
pub struct LinearExtrude {
    /// Distance to extrude
    pub distance: f64,
    /// Direction vector (will be normalized)
    pub direction: Vector3<f64>,
    /// Whether to cap the ends
    pub capped: bool,
}

impl LinearExtrude {
    /// Creates a new linear extrusion
    pub fn new(distance: f64, direction: Vector3<f64>) -> Self {
        assert!(distance > 0.0, "Distance must be positive");
        assert!(direction.norm() > 0.0, "Direction must be non-zero");

        Self {
            distance,
            direction: direction.normalize(),
            capped: true,
        }
    }

    /// Creates a vertical (Z-axis) extrusion
    pub fn vertical(distance: f64) -> Self {
        Self::new(distance, Vector3::z())
    }

    /// Sets whether to cap the ends
    pub fn with_caps(mut self, capped: bool) -> Self {
        self.capped = capped;
        self
    }

    /// Extrudes a 2D profile to create a 3D mesh
    pub fn extrude(&self, profile: &Profile2D) -> TriangleMesh {
        let mut mesh = TriangleMesh::new();

        let bottom_points = profile.to_3d(0.0);
        let top_points: Vec<Point3<f64>> = bottom_points
            .iter()
            .map(|p| p + self.direction * self.distance)
            .collect();

        // Add vertices
        for p in &bottom_points {
            mesh.add_vertex(Vertex::new(*p));
        }
        for p in &top_points {
            mesh.add_vertex(Vertex::new(*p));
        }

        let n = profile.points.len();
        let end = if profile.closed { n } else { n - 1 };

        // Create side faces
        for i in 0..end {
            let next = (i + 1) % n;

            let v0 = i;
            let v1 = next;
            let v2 = n + next;
            let v3 = n + i;

            // Two triangles per quad
            mesh.add_face(TriangleFace::new(v0, v1, v2));
            mesh.add_face(TriangleFace::new(v0, v2, v3));
        }

        // Add caps if requested and profile is closed
        if self.capped && profile.closed {
            // Bottom cap (triangulate)
            for i in 1..n - 1 {
                mesh.add_face(TriangleFace::new(0, i + 1, i));
            }

            // Top cap (triangulate, reversed winding)
            for i in 1..n - 1 {
                mesh.add_face(TriangleFace::new(n, n + i, n + i + 1));
            }
        }

        mesh.compute_vertex_normals();
        mesh
    }
}

/// Revolution (lathe) operation
pub struct Revolution {
    /// Axis of revolution (will be normalized)
    pub axis: Vector3<f64>,
    /// Point on the axis
    pub center: Point3<f64>,
    /// Angle to revolve (in radians)
    pub angle: f64,
    /// Number of segments
    pub segments: usize,
}

impl Revolution {
    /// Creates a new revolution operation
    pub fn new(axis: Vector3<f64>, center: Point3<f64>, angle: f64, segments: usize) -> Self {
        assert!(axis.norm() > 0.0, "Axis must be non-zero");
        assert!(segments >= 3, "Must have at least 3 segments");

        Self {
            axis: axis.normalize(),
            center,
            angle,
            segments,
        }
    }

    /// Creates a full revolution (360 degrees) around Z-axis
    pub fn full_z(segments: usize) -> Self {
        Self::new(Vector3::z(), Point3::origin(), 2.0 * PI, segments)
    }

    /// Revolves a 2D profile around an axis to create a 3D mesh
    pub fn revolve(&self, profile: &Profile2D) -> TriangleMesh {
        let mut mesh = TriangleMesh::new();

        // Generate vertices at each rotation step
        for seg in 0..=self.segments {
            let angle = (seg as f64 / self.segments as f64) * self.angle;
            let rotation = Rotation3::from_axis_angle(&Unit::new_normalize(self.axis), angle);

            for point in &profile.points {
                let p3d = Point3::new(point.x, point.y, 0.0);
                let rotated = rotation * (p3d - self.center) + self.center.coords;
                mesh.add_vertex(Vertex::new(Point3::from(rotated)));
            }
        }

        let n = profile.points.len();

        // Create faces
        for seg in 0..self.segments {
            for i in 0..n - 1 {
                let v0 = seg * n + i;
                let v1 = seg * n + i + 1;
                let v2 = (seg + 1) * n + i + 1;
                let v3 = (seg + 1) * n + i;

                mesh.add_face(TriangleFace::new(v0, v1, v2));
                mesh.add_face(TriangleFace::new(v0, v2, v3));
            }

            // Close the loop if profile is closed
            if profile.closed {
                let v0 = seg * n + n - 1;
                let v1 = seg * n;
                let v2 = (seg + 1) * n;
                let v3 = (seg + 1) * n + n - 1;

                mesh.add_face(TriangleFace::new(v0, v1, v2));
                mesh.add_face(TriangleFace::new(v0, v2, v3));
            }
        }

        mesh.compute_vertex_normals();
        mesh
    }
}

/// Sweep operation along a path
pub struct Sweep {
    /// Whether to keep the profile perpendicular to the path
    pub perpendicular: bool,
    /// Whether to cap the ends
    pub capped: bool,
    /// Scale factor along the path (optional)
    pub scale: Option<Vec<f64>>,
    /// Twist angle along the path (optional, in radians)
    pub twist: Option<f64>,
}

impl Sweep {
    /// Creates a new sweep operation
    pub fn new() -> Self {
        Self {
            perpendicular: true,
            capped: true,
            scale: None,
            twist: None,
        }
    }

    /// Sets whether to keep profile perpendicular to path
    pub fn with_perpendicular(mut self, perpendicular: bool) -> Self {
        self.perpendicular = perpendicular;
        self
    }

    /// Sets whether to cap the ends
    pub fn with_caps(mut self, capped: bool) -> Self {
        self.capped = capped;
        self
    }

    /// Sets a uniform scale along the path
    pub fn with_scale(mut self, start_scale: f64, end_scale: f64, steps: usize) -> Self {
        let mut scales = Vec::with_capacity(steps);
        for i in 0..steps {
            let t = i as f64 / (steps - 1).max(1) as f64;
            scales.push(start_scale + (end_scale - start_scale) * t);
        }
        self.scale = Some(scales);
        self
    }

    /// Sets twist along the path
    pub fn with_twist(mut self, twist: f64) -> Self {
        self.twist = Some(twist);
        self
    }

    /// Sweeps a profile along a path to create a 3D mesh
    pub fn sweep(&self, profile: &Profile2D, path: &mut Path3D) -> TriangleMesh {
        let mut mesh = TriangleMesh::new();

        // Ensure tangents are computed
        if path.tangents.is_none() {
            path.compute_tangents();
        }

        let tangents = path.tangents.as_ref().unwrap();
        let n_profile = profile.points.len();
        let n_path = path.points.len();

        // Generate vertices along the path
        for (path_idx, path_point) in path.points.iter().enumerate() {
            let tangent = tangents[path_idx];

            // Create a coordinate system at this point
            let up = if tangent.z.abs() < 0.9 {
                Vector3::z()
            } else {
                Vector3::x()
            };

            let right = tangent.cross(&up).normalize();
            let actual_up = right.cross(&tangent).normalize();

            // Get scale for this point
            let scale = if let Some(ref scales) = self.scale {
                let idx = (path_idx * scales.len() / n_path).min(scales.len() - 1);
                scales[idx]
            } else {
                1.0
            };

            // Get twist for this point
            let twist_angle = if let Some(twist) = self.twist {
                twist * (path_idx as f64 / (n_path - 1) as f64)
            } else {
                0.0
            };

            let twist_rotation = Rotation3::from_axis_angle(&Unit::new_normalize(tangent), twist_angle);

            // Transform profile points
            for profile_point in &profile.points {
                let mut p3d = right * profile_point.x * scale + actual_up * profile_point.y * scale;
                p3d = twist_rotation * p3d;
                let final_point = path_point + p3d;

                mesh.add_vertex(Vertex::new(final_point));
            }
        }

        // Create faces
        for path_idx in 0..n_path - 1 {
            let end = if profile.closed {
                n_profile
            } else {
                n_profile - 1
            };

            for prof_idx in 0..end {
                let next_prof = (prof_idx + 1) % n_profile;

                let v0 = path_idx * n_profile + prof_idx;
                let v1 = path_idx * n_profile + next_prof;
                let v2 = (path_idx + 1) * n_profile + next_prof;
                let v3 = (path_idx + 1) * n_profile + prof_idx;

                mesh.add_face(TriangleFace::new(v0, v1, v2));
                mesh.add_face(TriangleFace::new(v0, v2, v3));
            }
        }

        // Add caps if requested and profile is closed
        if self.capped && profile.closed {
            // Start cap
            for i in 1..n_profile - 1 {
                mesh.add_face(TriangleFace::new(0, i + 1, i));
            }

            // End cap
            let last_offset = (n_path - 1) * n_profile;
            for i in 1..n_profile - 1 {
                mesh.add_face(TriangleFace::new(
                    last_offset,
                    last_offset + i,
                    last_offset + i + 1,
                ));
            }
        }

        mesh.compute_vertex_normals();
        mesh
    }
}

impl Default for Sweep {
    fn default() -> Self {
        Self::new()
    }
}

/// Loft operation between multiple profiles
pub struct Loft {
    /// Whether to cap the ends
    pub capped: bool,
    /// Interpolation method
    pub smooth: bool,
}

impl Loft {
    /// Creates a new loft operation
    pub fn new() -> Self {
        Self {
            capped: true,
            smooth: true,
        }
    }

    /// Sets whether to cap the ends
    pub fn with_caps(mut self, capped: bool) -> Self {
        self.capped = capped;
        self
    }

    /// Lofts between multiple profiles to create a 3D mesh
    pub fn loft(&self, profiles: &[Profile2D], z_positions: &[f64]) -> TriangleMesh {
        assert_eq!(
            profiles.len(),
            z_positions.len(),
            "Number of profiles must match number of Z positions"
        );
        assert!(profiles.len() >= 2, "Must have at least 2 profiles");

        let mut mesh = TriangleMesh::new();

        // Verify all profiles have the same number of points
        let n_points = profiles[0].points.len();
        for profile in profiles {
            assert_eq!(
                profile.points.len(),
                n_points,
                "All profiles must have the same number of points"
            );
        }

        // Add all vertices
        for (profile, &z) in profiles.iter().zip(z_positions.iter()) {
            for point in &profile.points {
                mesh.add_vertex(Vertex::new(Point3::new(point.x, point.y, z)));
            }
        }

        // Create faces between consecutive profiles
        for prof_idx in 0..profiles.len() - 1 {
            let closed = profiles[prof_idx].closed;
            let end = if closed { n_points } else { n_points - 1 };

            for i in 0..end {
                let next = (i + 1) % n_points;

                let v0 = prof_idx * n_points + i;
                let v1 = prof_idx * n_points + next;
                let v2 = (prof_idx + 1) * n_points + next;
                let v3 = (prof_idx + 1) * n_points + i;

                mesh.add_face(TriangleFace::new(v0, v1, v2));
                mesh.add_face(TriangleFace::new(v0, v2, v3));
            }
        }

        // Add caps if requested
        if self.capped && profiles[0].closed {
            // Bottom cap
            for i in 1..n_points - 1 {
                mesh.add_face(TriangleFace::new(0, i + 1, i));
            }

            // Top cap
            let last_offset = (profiles.len() - 1) * n_points;
            for i in 1..n_points - 1 {
                mesh.add_face(TriangleFace::new(
                    last_offset,
                    last_offset + i,
                    last_offset + i + 1,
                ));
            }
        }

        mesh.compute_vertex_normals();
        mesh
    }
}

impl Default for Loft {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_profile_creation() {
        let rect = Profile2D::rectangle(10.0, 5.0);
        assert_eq!(rect.points.len(), 4);
        assert!(rect.closed);

        let circle = Profile2D::circle(5.0, 16);
        assert_eq!(circle.points.len(), 16);
    }

    #[test]
    fn test_profile_area() {
        let rect = Profile2D::rectangle(10.0, 5.0);
        assert_relative_eq!(rect.area(), 50.0, epsilon = 1e-10);
    }

    #[test]
    fn test_linear_extrude() {
        let profile = Profile2D::circle(1.0, 8);
        let extruder = LinearExtrude::vertical(5.0);
        let mesh = extruder.extrude(&profile);

        assert!(mesh.vertices.len() > 0);
        assert!(mesh.faces.len() > 0);
    }

    #[test]
    fn test_revolution() {
        let profile = Profile2D::new(
            vec![Point2::new(2.0, 0.0), Point2::new(2.0, 1.0), Point2::new(3.0, 1.0)],
            false,
        );

        let revolution = Revolution::full_z(16);
        let mesh = revolution.revolve(&profile);

        assert!(mesh.vertices.len() > 0);
        assert!(mesh.faces.len() > 0);
    }

    #[test]
    fn test_sweep() {
        let profile = Profile2D::circle(0.5, 8);
        let mut path = Path3D::line(Point3::new(0.0, 0.0, 0.0), Point3::new(5.0, 0.0, 0.0));

        let sweep = Sweep::new();
        let mesh = sweep.sweep(&profile, &mut path);

        assert!(mesh.vertices.len() > 0);
        assert!(mesh.faces.len() > 0);
    }

    #[test]
    fn test_loft() {
        let profile1 = Profile2D::circle(1.0, 8);
        let profile2 = Profile2D::circle(0.5, 8);

        let loft = Loft::new();
        let mesh = loft.loft(&[profile1, profile2], &[0.0, 5.0]);

        assert!(mesh.vertices.len() > 0);
        assert!(mesh.faces.len() > 0);
    }

    #[test]
    fn test_path_creation() {
        let line = Path3D::line(Point3::origin(), Point3::new(10.0, 0.0, 0.0));
        assert_eq!(line.points.len(), 2);
        assert_relative_eq!(line.length(), 10.0, epsilon = 1e-10);
    }
}
