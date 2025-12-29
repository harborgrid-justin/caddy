//! Topological operations for 3D modeling
//!
//! Implements extrude, revolve, sweep, loft, and other operations
//! that create 3D geometry from 2D profiles or transform existing geometry.

use super::mesh::{HalfEdgeMesh, VertexHandle, FaceHandle, MeshError};
use super::nurbs::{NurbsCurve, NurbsSurface};
use crate::core::{Point3, Vector3, Matrix4, EPSILON};
use nalgebra::{Matrix4 as NMatrix4, Vector3 as NVector3, UnitQuaternion, Translation3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f64::consts::PI;

/// Extrude a 2D profile along a direction
pub struct ExtrudeOperation {
    /// Direction and distance of extrusion
    pub direction: Vector3,
    /// Whether to cap the ends
    pub capped: bool,
    /// Twist angle in radians per unit length
    pub twist: f64,
    /// Scale factor along extrusion
    pub taper: f64,
}

impl Default for ExtrudeOperation {
    fn default() -> Self {
        Self {
            direction: Vector3::new(0.0, 0.0, 1.0),
            capped: true,
            twist: 0.0,
            taper: 1.0,
        }
    }
}

impl ExtrudeOperation {
    /// Extrude a profile represented as a list of vertices
    pub fn extrude_profile(&self, profile: &[Point3]) -> Result<HalfEdgeMesh, TopologyError> {
        if profile.len() < 3 {
            return Err(TopologyError::InsufficientVertices);
        }

        let mut mesh = HalfEdgeMesh::new();
        let n = profile.len();

        // Create bottom vertices
        let mut bottom_vertices = Vec::new();
        for point in profile {
            bottom_vertices.push(mesh.add_vertex(*point));
        }

        // Create top vertices with optional twist and taper
        let mut top_vertices = Vec::new();
        let length = self.direction.norm();

        for point in profile {
            let mut transformed = *point;

            // Apply taper (scale from center)
            if (self.taper - 1.0).abs() > EPSILON {
                let center = Self::compute_centroid(profile);
                let offset = transformed - center;
                transformed = center + offset * self.taper;
            }

            // Apply twist
            if self.twist.abs() > EPSILON {
                let angle = self.twist * length;
                let center = Self::compute_centroid(profile);
                transformed = Self::rotate_around_axis(transformed, center, &self.direction, angle);
            }

            // Translate
            transformed += self.direction;

            top_vertices.push(mesh.add_vertex(transformed));
        }

        // Create side faces
        for i in 0..n {
            let next_i = (i + 1) % n;

            let v0 = bottom_vertices[i];
            let v1 = bottom_vertices[next_i];
            let v2 = top_vertices[next_i];
            let v3 = top_vertices[i];

            // Create a quad (two triangles)
            mesh.add_face(&[v0, v1, v2, v3])
                .map_err(|_| TopologyError::MeshCreationFailed)?;
        }

        // Create caps if requested
        if self.capped {
            // Bottom cap (reversed winding)
            let bottom_verts: Vec<_> = bottom_vertices.iter().rev().copied().collect();
            mesh.add_face(&bottom_verts)
                .map_err(|_| TopologyError::MeshCreationFailed)?;

            // Top cap
            mesh.add_face(&top_vertices)
                .map_err(|_| TopologyError::MeshCreationFailed)?;
        }

        mesh.update_vertex_normals();

        Ok(mesh)
    }

    fn compute_centroid(points: &[Point3]) -> Point3 {
        let sum: Vector3 = points.iter().map(|p| p.coords).sum();
        Point3::from(sum / points.len() as f64)
    }

    fn rotate_around_axis(point: Point3, center: Point3, axis: &Vector3, angle: f64) -> Point3 {
        let offset = point - center;
        let axis_normalized = axis.normalize();

        let rotation = UnitQuaternion::from_axis_angle(
            &nalgebra::Unit::new_normalize(NVector3::new(
                axis_normalized.x,
                axis_normalized.y,
                axis_normalized.z,
            )),
            angle,
        );

        let rotated = rotation * NVector3::new(offset.x, offset.y, offset.z);
        Point3::new(
            center.x + rotated.x,
            center.y + rotated.y,
            center.z + rotated.z,
        )
    }
}

/// Revolve a 2D profile around an axis
pub struct RevolveOperation {
    /// Axis of revolution (origin)
    pub axis_origin: Point3,
    /// Axis direction
    pub axis_direction: Vector3,
    /// Angle of revolution in radians
    pub angle: f64,
    /// Number of segments around the axis
    pub segments: usize,
}

impl Default for RevolveOperation {
    fn default() -> Self {
        Self {
            axis_origin: Point3::new(0.0, 0.0, 0.0),
            axis_direction: Vector3::new(0.0, 0.0, 1.0),
            angle: 2.0 * PI,
            segments: 32,
        }
    }
}

impl RevolveOperation {
    /// Revolve a profile around the specified axis
    pub fn revolve_profile(&self, profile: &[Point3]) -> Result<HalfEdgeMesh, TopologyError> {
        if profile.len() < 2 {
            return Err(TopologyError::InsufficientVertices);
        }

        if self.segments < 3 {
            return Err(TopologyError::InvalidSegmentCount);
        }

        let mut mesh = HalfEdgeMesh::new();
        let n = profile.len();
        let m = self.segments;

        // Create vertex grid
        let mut vertex_grid = vec![vec![VertexHandle(0); m]; n];

        for i in 0..n {
            for j in 0..m {
                let angle = (j as f64 / m as f64) * self.angle;
                let point = ExtrudeOperation::rotate_around_axis(
                    profile[i],
                    self.axis_origin,
                    &self.axis_direction,
                    angle,
                );
                vertex_grid[i][j] = mesh.add_vertex(point);
            }
        }

        // Create faces
        for i in 0..n - 1 {
            for j in 0..m {
                let next_j = (j + 1) % m;

                // Skip last column if not a full revolution
                if self.angle < 2.0 * PI - EPSILON && next_j == 0 {
                    continue;
                }

                let v0 = vertex_grid[i][j];
                let v1 = vertex_grid[i][next_j];
                let v2 = vertex_grid[i + 1][next_j];
                let v3 = vertex_grid[i + 1][j];

                mesh.add_face(&[v0, v1, v2, v3])
                    .map_err(|_| TopologyError::MeshCreationFailed)?;
            }
        }

        mesh.update_vertex_normals();

        Ok(mesh)
    }
}

/// Sweep a profile along a path
pub struct SweepOperation {
    /// Path curve to sweep along
    pub path: NurbsCurve,
    /// Number of samples along the path
    pub samples: usize,
    /// Whether to keep the profile perpendicular to the path
    pub frenet_frame: bool,
}

impl SweepOperation {
    /// Sweep a profile along the path curve
    pub fn sweep_profile(&self, profile: &[Point3]) -> Result<HalfEdgeMesh, TopologyError> {
        if profile.len() < 3 {
            return Err(TopologyError::InsufficientVertices);
        }

        if self.samples < 2 {
            return Err(TopologyError::InvalidSegmentCount);
        }

        let mut mesh = HalfEdgeMesh::new();
        let n = profile.len();
        let m = self.samples;

        // Sample the path
        let path_points = self.path
            .tessellate(m)
            .map_err(|_| TopologyError::PathEvaluationFailed)?;

        if path_points.len() < 2 {
            return Err(TopologyError::PathEvaluationFailed);
        }

        // Create vertex grid
        let mut vertex_grid = vec![vec![VertexHandle(0); m]; n];

        for i in 0..m {
            let t = i as f64 / (m - 1) as f64;

            // Get path point and tangent
            let path_point = path_points[i];
            let tangent = if i < m - 1 {
                (path_points[i + 1] - path_points[i]).normalize()
            } else {
                (path_points[i] - path_points[i - 1]).normalize()
            };

            // Compute frame at this point
            let (up, right) = if self.frenet_frame {
                Self::compute_frenet_frame(&tangent)
            } else {
                Self::compute_simple_frame(&tangent)
            };

            // Transform profile to this frame
            for j in 0..n {
                let local_point = profile[j];
                let transformed = path_point
                    + right * local_point.x
                    + up * local_point.y
                    + tangent * local_point.z;

                vertex_grid[j][i] = mesh.add_vertex(transformed);
            }
        }

        // Create faces
        for i in 0..n {
            for j in 0..m - 1 {
                let next_i = (i + 1) % n;

                let v0 = vertex_grid[i][j];
                let v1 = vertex_grid[next_i][j];
                let v2 = vertex_grid[next_i][j + 1];
                let v3 = vertex_grid[i][j + 1];

                mesh.add_face(&[v0, v1, v2, v3])
                    .map_err(|_| TopologyError::MeshCreationFailed)?;
            }
        }

        mesh.update_vertex_normals();

        Ok(mesh)
    }

    fn compute_frenet_frame(tangent: &Vector3) -> (Vector3, Vector3) {
        // Simplified Frenet frame (constant normal)
        let arbitrary = if tangent.x.abs() < 0.9 {
            Vector3::new(1.0, 0.0, 0.0)
        } else {
            Vector3::new(0.0, 1.0, 0.0)
        };

        let right = tangent.cross(&arbitrary).normalize();
        let up = right.cross(tangent).normalize();

        (up, right)
    }

    fn compute_simple_frame(tangent: &Vector3) -> (Vector3, Vector3) {
        Self::compute_frenet_frame(tangent)
    }
}

/// Loft between multiple profiles
pub struct LoftOperation {
    /// Profiles to loft between
    pub profiles: Vec<Vec<Point3>>,
    /// Whether to close the loft (connect first and last profiles)
    pub closed: bool,
}

impl LoftOperation {
    /// Create a lofted surface between profiles
    pub fn loft(&self) -> Result<HalfEdgeMesh, TopologyError> {
        if self.profiles.len() < 2 {
            return Err(TopologyError::InsufficientProfiles);
        }

        // All profiles must have the same number of vertices
        let n = self.profiles[0].len();
        for profile in &self.profiles {
            if profile.len() != n {
                return Err(TopologyError::ProfileSizeMismatch);
            }
        }

        if n < 3 {
            return Err(TopologyError::InsufficientVertices);
        }

        let mut mesh = HalfEdgeMesh::new();
        let m = self.profiles.len();

        // Create vertex grid
        let mut vertex_grid = vec![vec![VertexHandle(0); m]; n];

        for i in 0..n {
            for j in 0..m {
                vertex_grid[i][j] = mesh.add_vertex(self.profiles[j][i]);
            }
        }

        // Create faces between profiles
        let profile_count = if self.closed { m } else { m - 1 };

        for i in 0..n {
            for j in 0..profile_count {
                let next_i = (i + 1) % n;
                let next_j = (j + 1) % m;

                let v0 = vertex_grid[i][j];
                let v1 = vertex_grid[next_i][j];
                let v2 = vertex_grid[next_i][next_j];
                let v3 = vertex_grid[i][next_j];

                mesh.add_face(&[v0, v1, v2, v3])
                    .map_err(|_| TopologyError::MeshCreationFailed)?;
            }
        }

        mesh.update_vertex_normals();

        Ok(mesh)
    }
}

/// Shell operation (offset surfaces)
pub struct ShellOperation {
    /// Thickness of the shell
    pub thickness: f64,
    /// Whether to create inward or outward offset
    pub outward: bool,
}

impl ShellOperation {
    /// Create a shell from a mesh
    pub fn shell_mesh(&self, mesh: &HalfEdgeMesh) -> Result<HalfEdgeMesh, TopologyError> {
        let mut result = HalfEdgeMesh::new();
        let mut vertex_map = HashMap::new();

        // Create offset vertices
        for vh in mesh.vertex_handles() {
            let vertex = mesh.get_vertex(vh).map_err(|_| TopologyError::InvalidMesh)?;
            let offset_direction = if self.outward {
                vertex.normal
            } else {
                -vertex.normal
            };

            let new_pos = Point3::from(vertex.position.coords + offset_direction * self.thickness);
            let new_vh = result.add_vertex(new_pos);
            vertex_map.insert(vh, new_vh);
        }

        // Copy faces with new vertices
        for fh in mesh.face_handles() {
            let vertices = mesh.face_vertices(fh).map_err(|_| TopologyError::InvalidMesh)?;
            let new_vertices: Vec<_> = vertices.iter().map(|vh| vertex_map[vh]).collect();

            result.add_face(&new_vertices)
                .map_err(|_| TopologyError::MeshCreationFailed)?;
        }

        result.update_vertex_normals();

        Ok(result)
    }
}

/// Topology operation errors
#[derive(Debug, thiserror::Error)]
pub enum TopologyError {
    #[error("Insufficient vertices for operation")]
    InsufficientVertices,

    #[error("Invalid segment count")]
    InvalidSegmentCount,

    #[error("Mesh creation failed")]
    MeshCreationFailed,

    #[error("Path evaluation failed")]
    PathEvaluationFailed,

    #[error("Insufficient profiles for loft operation")]
    InsufficientProfiles,

    #[error("Profile size mismatch")]
    ProfileSizeMismatch,

    #[error("Invalid mesh")]
    InvalidMesh,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extrude_square() {
        let profile = vec![
            Point3::new(-1.0, -1.0, 0.0),
            Point3::new(1.0, -1.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(-1.0, 1.0, 0.0),
        ];

        let extrude = ExtrudeOperation {
            direction: Vector3::new(0.0, 0.0, 2.0),
            ..Default::default()
        };

        let mesh = extrude.extrude_profile(&profile);
        assert!(mesh.is_ok());

        let mesh = mesh.unwrap();
        let stats = mesh.stats();
        assert_eq!(stats.vertices, 8); // 4 bottom + 4 top
    }

    #[test]
    fn test_revolve_line() {
        let profile = vec![
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 1.0),
        ];

        let revolve = RevolveOperation {
            segments: 16,
            ..Default::default()
        };

        let mesh = revolve.revolve_profile(&profile);
        assert!(mesh.is_ok());
    }

    #[test]
    fn test_loft_circles() {
        let n = 8;
        let mut profiles = Vec::new();

        for z in 0..3 {
            let mut profile = Vec::new();
            let radius = 1.0 - z as f64 * 0.2;

            for i in 0..n {
                let angle = 2.0 * PI * (i as f64 / n as f64);
                profile.push(Point3::new(
                    radius * angle.cos(),
                    radius * angle.sin(),
                    z as f64,
                ));
            }
            profiles.push(profile);
        }

        let loft = LoftOperation {
            profiles,
            closed: false,
        };

        let mesh = loft.loft();
        assert!(mesh.is_ok());
    }
}
