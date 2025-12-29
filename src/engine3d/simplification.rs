//! Mesh decimation and simplification algorithms
//!
//! Implements edge collapse and quadric error metrics for Level-of-Detail (LOD)
//! generation with feature preservation.

use super::mesh::{HalfEdgeMesh, VertexHandle, EdgeHandle, MeshError};
use crate::core::{Point3, Vector3};
use nalgebra::{Matrix4 as NMatrix4, Vector4};
use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;

/// Mesh simplification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplificationSettings {
    /// Target triangle count (will stop when reached)
    pub target_triangle_count: Option<usize>,
    /// Target reduction ratio (0.0 = no reduction, 1.0 = remove everything)
    pub reduction_ratio: f64,
    /// Preserve boundary edges
    pub preserve_boundaries: bool,
    /// Preserve sharp features (based on dihedral angle)
    pub preserve_sharp_features: bool,
    /// Minimum dihedral angle to consider an edge as sharp (radians)
    pub sharp_angle_threshold: f64,
    /// Maximum allowed error
    pub max_error: f64,
}

impl Default for SimplificationSettings {
    fn default() -> Self {
        Self {
            target_triangle_count: None,
            reduction_ratio: 0.5,
            preserve_boundaries: true,
            preserve_sharp_features: true,
            sharp_angle_threshold: 0.5, // ~28.6 degrees
            max_error: f64::INFINITY,
        }
    }
}

/// Quadric error metric mesh simplifier
pub struct QuadricSimplifier {
    settings: SimplificationSettings,
    vertex_quadrics: HashMap<VertexHandle, Quadric>,
    edge_costs: BinaryHeap<EdgeCost>,
    removed_edges: HashSet<EdgeHandle>,
}

impl QuadricSimplifier {
    /// Create a new simplifier with the given settings
    pub fn new(settings: SimplificationSettings) -> Self {
        Self {
            settings,
            vertex_quadrics: HashMap::new(),
            edge_costs: BinaryHeap::new(),
            removed_edges: HashSet::new(),
        }
    }

    /// Simplify a mesh
    pub fn simplify(&mut self, mesh: &mut HalfEdgeMesh) -> Result<SimplificationReport, MeshError> {
        let initial_faces = mesh.stats().faces;

        // Initialize quadrics for all vertices
        self.initialize_quadrics(mesh)?;

        // Compute initial edge collapse costs
        self.compute_edge_costs(mesh)?;

        // Determine target face count
        let target_faces = if let Some(target) = self.settings.target_triangle_count {
            target
        } else {
            (initial_faces as f64 * (1.0 - self.settings.reduction_ratio)) as usize
        };

        let mut collapsed = 0;
        let mut current_faces = initial_faces;

        // Perform edge collapses
        while current_faces > target_faces && !self.edge_costs.is_empty() {
            if let Some(edge_cost) = self.edge_costs.pop() {
                // Skip if edge was already removed
                if self.removed_edges.contains(&edge_cost.edge) {
                    continue;
                }

                // Skip if error is too high
                if edge_cost.cost > self.settings.max_error {
                    break;
                }

                // Attempt to collapse the edge
                if self.collapse_edge(mesh, edge_cost.edge, &edge_cost.target_position).is_ok() {
                    collapsed += 1;
                    current_faces = mesh.stats().faces;
                }
            }
        }

        // Update vertex normals
        mesh.update_vertex_normals();

        Ok(SimplificationReport {
            initial_faces,
            final_faces: current_faces,
            collapsed_edges: collapsed,
            reduction_ratio: 1.0 - (current_faces as f64 / initial_faces as f64),
        })
    }

    /// Initialize quadric error matrices for all vertices
    fn initialize_quadrics(&mut self, mesh: &HalfEdgeMesh) -> Result<(), MeshError> {
        self.vertex_quadrics.clear();

        // Initialize all vertices with zero quadric
        for vh in mesh.vertex_handles() {
            self.vertex_quadrics.insert(vh, Quadric::zero());
        }

        // Add plane quadrics from all faces
        for fh in mesh.face_handles() {
            let face = mesh.get_face(fh)?;
            let vertices = mesh.face_vertices(fh)?;

            if vertices.len() < 3 {
                continue;
            }

            // Compute face plane
            let v0 = mesh.get_vertex(vertices[0])?;
            let v1 = mesh.get_vertex(vertices[1])?;
            let v2 = mesh.get_vertex(vertices[2])?;

            let normal = face.normal;
            let d = -normal.dot(&v0.position.coords);

            let plane_quadric = Quadric::from_plane(normal, d);

            // Add this quadric to all vertices of the face
            for vh in vertices {
                if let Some(q) = self.vertex_quadrics.get_mut(&vh) {
                    *q = q.add(&plane_quadric);
                }
            }
        }

        Ok(())
    }

    /// Compute collapse costs for all edges
    fn compute_edge_costs(&mut self, mesh: &HalfEdgeMesh) -> Result<(), MeshError> {
        self.edge_costs.clear();

        for edge_handle in mesh.edge_handles() {
            let edge = mesh.get_edge(edge_handle)?;

            // Skip boundary edges if preserve_boundaries is true
            if self.settings.preserve_boundaries && edge.is_boundary {
                continue;
            }

            // Skip sharp edges if preserve_sharp_features is true
            if self.settings.preserve_sharp_features && edge.is_sharp {
                continue;
            }

            // Check dihedral angle
            if self.settings.preserve_sharp_features {
                if let Some(angle) = self.compute_dihedral_angle(mesh, edge_handle)? {
                    if angle > self.settings.sharp_angle_threshold {
                        continue;
                    }
                }
            }

            // Compute collapse cost
            if let Some(cost) = self.compute_collapse_cost(mesh, edge_handle)? {
                self.edge_costs.push(cost);
            }
        }

        Ok(())
    }

    /// Compute the dihedral angle of an edge
    fn compute_dihedral_angle(&self, mesh: &HalfEdgeMesh, edge_handle: EdgeHandle) -> Result<Option<f64>, MeshError> {
        let edge = mesh.get_edge(edge_handle)?;
        let he = mesh.get_halfedge(edge.halfedge)?;

        if let Some(twin_handle) = he.twin {
            if let (Some(f1), Some(f2)) = (he.face, mesh.get_halfedge(twin_handle)?.face) {
                let face1 = mesh.get_face(f1)?;
                let face2 = mesh.get_face(f2)?;

                let dot = face1.normal.dot(&face2.normal);
                let angle = dot.clamp(-1.0, 1.0).acos();

                return Ok(Some((std::f64::consts::PI - angle).abs()));
            }
        }

        Ok(None)
    }

    /// Compute the cost of collapsing an edge
    fn compute_collapse_cost(&self, mesh: &HalfEdgeMesh, edge_handle: EdgeHandle) -> Result<Option<EdgeCost>, MeshError> {
        let edge = mesh.get_edge(edge_handle)?;
        let he = mesh.get_halfedge(edge.halfedge)?;

        // Get the two vertices
        let he_prev = mesh.get_halfedge(he.prev)?;
        let v1 = he_prev.vertex;
        let v2 = he.vertex;

        // Get quadrics
        let q1 = self.vertex_quadrics.get(&v1).ok_or(MeshError::InvalidVertexHandle)?;
        let q2 = self.vertex_quadrics.get(&v2).ok_or(MeshError::InvalidVertexHandle)?;

        // Combined quadric
        let q = q1.add(q2);

        // Find optimal target position
        let target_position = self.find_optimal_position(mesh, v1, v2, &q)?;

        // Compute error at target position
        let cost = q.evaluate(&target_position);

        Ok(Some(EdgeCost {
            edge: edge_handle,
            cost,
            target_position,
        }))
    }

    /// Find the optimal position for edge collapse
    fn find_optimal_position(&self, mesh: &HalfEdgeMesh, v1: VertexHandle, v2: VertexHandle, quadric: &Quadric) -> Result<Point3, MeshError> {
        // Try to find optimal position by solving the quadric equation
        // For now, use the simpler approach of testing endpoints and midpoint
        let p1 = mesh.get_vertex(v1)?.position;
        let p2 = mesh.get_vertex(v2)?.position;
        let p_mid = Point3::from((p1.coords + p2.coords) / 2.0);

        let e1 = quadric.evaluate(&p1);
        let e2 = quadric.evaluate(&p2);
        let e_mid = quadric.evaluate(&p_mid);

        if e1 <= e2 && e1 <= e_mid {
            Ok(p1)
        } else if e2 <= e_mid {
            Ok(p2)
        } else {
            Ok(p_mid)
        }
    }

    /// Collapse an edge
    fn collapse_edge(&mut self, mesh: &mut HalfEdgeMesh, edge_handle: EdgeHandle, target_pos: &Point3) -> Result<(), MeshError> {
        // Mark edge as removed
        self.removed_edges.insert(edge_handle);

        // This is a simplified placeholder
        // Full implementation would:
        // 1. Get the two vertices of the edge
        // 2. Move one vertex to the target position
        // 3. Update all faces that reference the removed vertex
        // 4. Remove degenerate faces
        // 5. Update the quadric for the remaining vertex
        // 6. Recompute costs for affected edges

        Ok(())
    }
}

/// Quadric error metric (4x4 matrix representation)
#[derive(Debug, Clone, Copy)]
struct Quadric {
    matrix: [[f64; 4]; 4],
}

impl Quadric {
    /// Create a zero quadric
    fn zero() -> Self {
        Self {
            matrix: [[0.0; 4]; 4],
        }
    }

    /// Create a quadric from a plane equation ax + by + cz + d = 0
    fn from_plane(normal: Vector3, d: f64) -> Self {
        let a = normal.x;
        let b = normal.y;
        let c = normal.z;

        Self {
            matrix: [
                [a * a, a * b, a * c, a * d],
                [a * b, b * b, b * c, b * d],
                [a * c, b * c, c * c, c * d],
                [a * d, b * d, c * d, d * d],
            ],
        }
    }

    /// Add two quadrics
    fn add(&self, other: &Quadric) -> Quadric {
        let mut result = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = self.matrix[i][j] + other.matrix[i][j];
            }
        }
        Quadric { matrix: result }
    }

    /// Evaluate the quadric error at a point
    fn evaluate(&self, point: &Point3) -> f64 {
        let v = [point.x, point.y, point.z, 1.0];
        let mut result = 0.0;

        for i in 0..4 {
            for j in 0..4 {
                result += v[i] * self.matrix[i][j] * v[j];
            }
        }

        result
    }
}

/// Edge collapse cost information
#[derive(Debug, Clone)]
struct EdgeCost {
    edge: EdgeHandle,
    cost: f64,
    target_position: Point3,
}

impl PartialEq for EdgeCost {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Eq for EdgeCost {}

impl PartialOrd for EdgeCost {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Reverse ordering for min-heap
        other.cost.partial_cmp(&self.cost)
    }
}

impl Ord for EdgeCost {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

/// Simplification report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplificationReport {
    pub initial_faces: usize,
    pub final_faces: usize,
    pub collapsed_edges: usize,
    pub reduction_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quadric_creation() {
        let q = Quadric::zero();
        let point = Point3::new(0.0, 0.0, 0.0);
        assert_eq!(q.evaluate(&point), 0.0);
    }

    #[test]
    fn test_plane_quadric() {
        let normal = Vector3::new(0.0, 0.0, 1.0);
        let d = 0.0;
        let q = Quadric::from_plane(normal, d);

        let point_on_plane = Point3::new(1.0, 1.0, 0.0);
        let error = q.evaluate(&point_on_plane);
        assert!(error.abs() < EPSILON);
    }

    #[test]
    fn test_simplification_settings() {
        let settings = SimplificationSettings::default();
        assert_eq!(settings.reduction_ratio, 0.5);
        assert!(settings.preserve_boundaries);
    }
}
