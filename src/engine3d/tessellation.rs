//! Adaptive tessellation algorithms for converting curved surfaces to triangles
//!
//! Provides enterprise-grade tessellation with adaptive refinement based on
//! curvature, screen-space error, and user-defined quality settings.

use super::mesh::{HalfEdgeMesh, VertexHandle, FaceHandle};
use super::nurbs::{NurbsSurface, NurbsCurve};
use crate::core::{Point3, Vector3, Point2, EPSILON};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tessellation quality settings
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TessellationSettings {
    /// Maximum chord error (distance from curve to chord)
    pub max_chord_error: f64,
    /// Maximum angle between adjacent face normals (in radians)
    pub max_angle_deviation: f64,
    /// Minimum edge length
    pub min_edge_length: f64,
    /// Maximum edge length
    pub max_edge_length: f64,
    /// Target number of triangles (soft limit)
    pub target_triangle_count: Option<usize>,
}

impl Default for TessellationSettings {
    fn default() -> Self {
        Self {
            max_chord_error: 0.01,
            max_angle_deviation: 0.1, // ~5.7 degrees
            min_edge_length: 0.001,
            max_edge_length: 1.0,
            target_triangle_count: None,
        }
    }
}

/// Adaptive tessellator for NURBS surfaces
pub struct AdaptiveTessellator {
    settings: TessellationSettings,
}

impl AdaptiveTessellator {
    /// Create a new tessellator with the given settings
    pub fn new(settings: TessellationSettings) -> Self {
        Self { settings }
    }

    /// Tessellate a NURBS surface adaptively
    pub fn tessellate_surface(&self, surface: &NurbsSurface) -> Result<HalfEdgeMesh, TessellationError> {
        // Start with a coarse grid
        let initial_u = 4;
        let initial_v = 4;

        // Generate initial parameter grid
        let mut grid = ParameterGrid::new(initial_u, initial_v);

        // Refine adaptively based on curvature and error
        self.refine_grid(surface, &mut grid)?;

        // Convert grid to mesh
        self.grid_to_mesh(surface, &grid)
    }

    /// Refine the parameter grid adaptively
    fn refine_grid(&self, surface: &NurbsSurface, grid: &mut ParameterGrid) -> Result<(), TessellationError> {
        let max_iterations = 10;
        let mut iteration = 0;

        loop {
            if iteration >= max_iterations {
                break;
            }

            let mut needs_refinement = Vec::new();

            // Check each quad for refinement criteria
            for i in 0..grid.num_u - 1 {
                for j in 0..grid.num_v - 1 {
                    if self.needs_refinement(surface, grid, i, j)? {
                        needs_refinement.push((i, j));
                    }
                }
            }

            if needs_refinement.is_empty() {
                break;
            }

            // Refine marked quads
            grid.refine_quads(&needs_refinement);

            iteration += 1;
        }

        Ok(())
    }

    /// Check if a quad needs refinement
    fn needs_refinement(
        &self,
        surface: &NurbsSurface,
        grid: &ParameterGrid,
        i: usize,
        j: usize,
    ) -> Result<bool, TessellationError> {
        let u0 = grid.u_params[i];
        let u1 = grid.u_params[i + 1];
        let v0 = grid.v_params[j];
        let v1 = grid.v_params[j + 1];

        let u_mid = (u0 + u1) / 2.0;
        let v_mid = (v0 + v1) / 2.0;

        // Evaluate corners and center
        let p00 = surface.evaluate(u0, v0).map_err(|_| TessellationError::EvaluationFailed)?;
        let p01 = surface.evaluate(u0, v1).map_err(|_| TessellationError::EvaluationFailed)?;
        let p10 = surface.evaluate(u1, v0).map_err(|_| TessellationError::EvaluationFailed)?;
        let p11 = surface.evaluate(u1, v1).map_err(|_| TessellationError::EvaluationFailed)?;
        let p_mid = surface.evaluate(u_mid, v_mid).map_err(|_| TessellationError::EvaluationFailed)?;

        // Compute bilinear interpolation at center
        let p_interp = Point3::from(
            0.25 * (p00.coords + p01.coords + p10.coords + p11.coords)
        );

        // Check chord error
        let chord_error = (p_mid - p_interp).norm();
        if chord_error > self.settings.max_chord_error {
            return Ok(true);
        }

        // Check edge lengths
        let edge_lengths = [
            (p01 - p00).norm(),
            (p11 - p10).norm(),
            (p10 - p00).norm(),
            (p11 - p01).norm(),
        ];

        for &length in &edge_lengths {
            if length > self.settings.max_edge_length {
                return Ok(true);
            }
        }

        // Check angle deviation (curvature)
        let n00 = self.compute_normal(surface, u0, v0)?;
        let n11 = self.compute_normal(surface, u1, v1)?;
        let angle = n00.dot(&n11).acos();

        if angle > self.settings.max_angle_deviation {
            return Ok(true);
        }

        Ok(false)
    }

    /// Compute surface normal at (u, v)
    fn compute_normal(&self, surface: &NurbsSurface, u: f64, v: f64) -> Result<Vector3, TessellationError> {
        let du = 1e-6;
        let dv = 1e-6;

        let p = surface.evaluate(u, v).map_err(|_| TessellationError::EvaluationFailed)?;
        let pu = surface.evaluate(u + du, v).map_err(|_| TessellationError::EvaluationFailed)?;
        let pv = surface.evaluate(u, v + dv).map_err(|_| TessellationError::EvaluationFailed)?;

        let tangent_u = (pu - p) / du;
        let tangent_v = (pv - p) / dv;

        let normal = tangent_u.cross(&tangent_v);
        let len = normal.norm();

        if len < EPSILON {
            Ok(Vector3::new(0.0, 0.0, 1.0))
        } else {
            Ok(normal / len)
        }
    }

    /// Convert parameter grid to mesh
    fn grid_to_mesh(&self, surface: &NurbsSurface, grid: &ParameterGrid) -> Result<HalfEdgeMesh, TessellationError> {
        let mut mesh = HalfEdgeMesh::new();
        let mut vertex_map: HashMap<(usize, usize), VertexHandle> = HashMap::new();

        // Create vertices
        for i in 0..grid.num_u {
            for j in 0..grid.num_v {
                let u = grid.u_params[i];
                let v = grid.v_params[j];

                let point = surface.evaluate(u, v).map_err(|_| TessellationError::EvaluationFailed)?;
                let vh = mesh.add_vertex(point);
                vertex_map.insert((i, j), vh);
            }
        }

        // Create faces
        for i in 0..grid.num_u - 1 {
            for j in 0..grid.num_v - 1 {
                let v00 = vertex_map[&(i, j)];
                let v01 = vertex_map[&(i, j + 1)];
                let v10 = vertex_map[&(i + 1, j)];
                let v11 = vertex_map[&(i + 1, j + 1)];

                // Create two triangles for the quad
                mesh.add_face(&[v00, v10, v11]).map_err(|_| TessellationError::MeshCreationFailed)?;
                mesh.add_face(&[v00, v11, v01]).map_err(|_| TessellationError::MeshCreationFailed)?;
            }
        }

        // Update vertex normals
        mesh.update_vertex_normals();

        Ok(mesh)
    }

    /// Tessellate a NURBS curve into line segments
    pub fn tessellate_curve(&self, curve: &NurbsCurve) -> Result<Vec<Point3>, TessellationError> {
        let mut points = vec![
            curve.evaluate(0.0).map_err(|_| TessellationError::EvaluationFailed)?
        ];

        self.tessellate_curve_recursive(curve, 0.0, 1.0, &mut points)?;

        points.push(
            curve.evaluate(1.0).map_err(|_| TessellationError::EvaluationFailed)?
        );

        Ok(points)
    }

    /// Recursively tessellate a curve segment
    fn tessellate_curve_recursive(
        &self,
        curve: &NurbsCurve,
        t0: f64,
        t1: f64,
        points: &mut Vec<Point3>,
    ) -> Result<(), TessellationError> {
        let p0 = curve.evaluate(t0).map_err(|_| TessellationError::EvaluationFailed)?;
        let p1 = curve.evaluate(t1).map_err(|_| TessellationError::EvaluationFailed)?;
        let t_mid = (t0 + t1) / 2.0;
        let p_mid = curve.evaluate(t_mid).map_err(|_| TessellationError::EvaluationFailed)?;

        // Compute chord midpoint
        let chord_mid = Point3::from((p0.coords + p1.coords) / 2.0);

        // Compute chord error
        let error = (p_mid - chord_mid).norm();

        if error > self.settings.max_chord_error && (p1 - p0).norm() > self.settings.min_edge_length {
            // Subdivide
            self.tessellate_curve_recursive(curve, t0, t_mid, points)?;
            points.push(p_mid);
            self.tessellate_curve_recursive(curve, t_mid, t1, points)?;
        }

        Ok(())
    }
}

/// Parameter grid for tessellation
struct ParameterGrid {
    u_params: Vec<f64>,
    v_params: Vec<f64>,
    num_u: usize,
    num_v: usize,
}

impl ParameterGrid {
    /// Create a new parameter grid
    fn new(num_u: usize, num_v: usize) -> Self {
        let u_params: Vec<f64> = (0..num_u).map(|i| i as f64 / (num_u - 1) as f64).collect();
        let v_params: Vec<f64> = (0..num_v).map(|i| i as f64 / (num_v - 1) as f64).collect();

        Self {
            u_params,
            v_params,
            num_u,
            num_v,
        }
    }

    /// Refine the grid by subdividing marked quads
    fn refine_quads(&mut self, quads: &[(usize, usize)]) {
        // Collect unique split points
        let mut u_splits = std::collections::HashSet::new();
        let mut v_splits = std::collections::HashSet::new();

        for &(i, j) in quads {
            let u_mid = (self.u_params[i] + self.u_params[i + 1]) / 2.0;
            let v_mid = (self.v_params[j] + self.v_params[j + 1]) / 2.0;

            u_splits.insert(ordered_float::OrderedFloat(u_mid));
            v_splits.insert(ordered_float::OrderedFloat(v_mid));
        }

        // Insert split points
        for &u in &u_splits {
            if !self.u_params.contains(&u.0) {
                self.u_params.push(u.0);
            }
        }

        for &v in &v_splits {
            if !self.v_params.contains(&v.0) {
                self.v_params.push(v.0);
            }
        }

        // Sort parameters
        self.u_params.sort_by(|a, b| a.partial_cmp(b).unwrap());
        self.v_params.sort_by(|a, b| a.partial_cmp(b).unwrap());

        self.num_u = self.u_params.len();
        self.num_v = self.v_params.len();
    }
}

/// Delaunay triangulation for point clouds
pub struct DelaunayTriangulator;

impl DelaunayTriangulator {
    /// Triangulate a set of 2D points using Delaunay triangulation
    pub fn triangulate_2d(points: &[Point2]) -> Result<Vec<[usize; 3]>, TessellationError> {
        if points.len() < 3 {
            return Err(TessellationError::InsufficientPoints);
        }

        // Simple ear-clipping for now (TODO: implement proper Delaunay)
        let mut triangles = Vec::new();
        let n = points.len();

        // Create a simple fan triangulation for convex polygons
        for i in 1..n - 1 {
            triangles.push([0, i, i + 1]);
        }

        Ok(triangles)
    }

    /// Project 3D points onto best-fit plane and triangulate
    pub fn triangulate_3d(points: &[Point3]) -> Result<Vec<[usize; 3]>, TessellationError> {
        if points.len() < 3 {
            return Err(TessellationError::InsufficientPoints);
        }

        // Compute best-fit plane
        let centroid = Self::compute_centroid(points);
        let normal = Self::compute_normal(points, &centroid)?;

        // Create coordinate system on the plane
        let (u_axis, v_axis) = Self::create_plane_basis(&normal);

        // Project points onto plane
        let projected: Vec<Point2> = points
            .iter()
            .map(|p| {
                let v = p - centroid;
                Point2::new(v.dot(&u_axis), v.dot(&v_axis))
            })
            .collect();

        // Triangulate 2D points
        Self::triangulate_2d(&projected)
    }

    fn compute_centroid(points: &[Point3]) -> Point3 {
        let sum: Vector3 = points.iter().map(|p| p.coords).sum();
        Point3::from(sum / points.len() as f64)
    }

    fn compute_normal(points: &[Point3], centroid: &Point3) -> Result<Vector3, TessellationError> {
        // Use Newell's method for robust normal computation
        let mut normal = Vector3::zeros();

        for i in 0..points.len() {
            let j = (i + 1) % points.len();
            let pi = points[i] - centroid;
            let pj = points[j] - centroid;

            normal.x += (pi.y - pj.y) * (pi.z + pj.z);
            normal.y += (pi.z - pj.z) * (pi.x + pj.x);
            normal.z += (pi.x - pj.x) * (pi.y + pj.y);
        }

        let len = normal.norm();
        if len < EPSILON {
            return Err(TessellationError::DegeneratePolygon);
        }

        Ok(normal / len)
    }

    fn create_plane_basis(normal: &Vector3) -> (Vector3, Vector3) {
        // Choose an arbitrary vector not parallel to normal
        let arbitrary = if normal.x.abs() < 0.9 {
            Vector3::new(1.0, 0.0, 0.0)
        } else {
            Vector3::new(0.0, 1.0, 0.0)
        };

        let u = normal.cross(&arbitrary).normalize();
        let v = normal.cross(&u).normalize();

        (u, v)
    }
}

/// Tessellation errors
#[derive(Debug, thiserror::Error)]
pub enum TessellationError {
    #[error("Surface evaluation failed")]
    EvaluationFailed,

    #[error("Mesh creation failed")]
    MeshCreationFailed,

    #[error("Insufficient points for triangulation")]
    InsufficientPoints,

    #[error("Degenerate polygon (collinear points)")]
    DegeneratePolygon,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tessellation_settings() {
        let settings = TessellationSettings::default();
        assert!(settings.max_chord_error > 0.0);
        assert!(settings.max_angle_deviation > 0.0);
    }

    #[test]
    fn test_delaunay_triangulation() {
        let points = vec![
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 0.0),
            Point2::new(1.0, 1.0),
            Point2::new(0.0, 1.0),
        ];

        let triangles = DelaunayTriangulator::triangulate_2d(&points).unwrap();
        assert!(!triangles.is_empty());
    }
}
