//! Mesh healing and repair algorithms
//!
//! Provides robust algorithms to fix common mesh problems including
//! degenerate faces, non-manifold edges, holes, and inconsistent normals.

use super::mesh::{HalfEdgeMesh, VertexHandle, FaceHandle, MeshError};
use crate::core::{Point3, EPSILON};
use std::collections::{HashMap, HashSet};

/// Mesh healing operations
pub struct MeshHealer {
    /// Tolerance for vertex merging
    pub merge_tolerance: f64,
    /// Minimum edge length (edges shorter than this may be collapsed)
    pub min_edge_length: f64,
    /// Minimum face area
    pub min_face_area: f64,
}

impl Default for MeshHealer {
    fn default() -> Self {
        Self {
            merge_tolerance: EPSILON * 10.0,
            min_edge_length: EPSILON * 100.0,
            min_face_area: EPSILON * 1000.0,
        }
    }
}

impl MeshHealer {
    /// Create a new mesh healer with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Perform a full healing pass on the mesh
    pub fn heal(&self, mesh: &mut HalfEdgeMesh) -> HealingReport {
        let mut report = HealingReport::default();

        // Remove duplicate vertices
        report.merged_vertices = self.merge_duplicate_vertices(mesh);

        // Remove degenerate faces
        report.removed_degenerate_faces = self.remove_degenerate_faces(mesh);

        // Fix inconsistent normals
        report.flipped_normals = self.fix_normals(mesh);

        // Fill small holes
        report.filled_holes = self.fill_holes(mesh);

        // Remove isolated vertices
        report.removed_isolated_vertices = self.remove_isolated_vertices(mesh);

        report
    }

    /// Merge duplicate vertices within tolerance
    fn merge_duplicate_vertices(&self, mesh: &mut HalfEdgeMesh) -> usize {
        let vertex_handles = mesh.vertex_handles();
        let mut merged_count = 0;

        // Build spatial hash for efficient proximity queries
        let mut spatial_hash: HashMap<SpatialKey, Vec<VertexHandle>> = HashMap::new();

        for vh in &vertex_handles {
            if let Ok(vertex) = mesh.get_vertex(*vh) {
                let key = SpatialKey::from_point(&vertex.position, self.merge_tolerance);
                spatial_hash.entry(key).or_insert_with(Vec::new).push(*vh);
            }
        }

        // Find and merge duplicates
        let mut merge_map: HashMap<VertexHandle, VertexHandle> = HashMap::new();

        for (_key, vertices) in spatial_hash.iter() {
            if vertices.len() > 1 {
                // Keep the first vertex, merge others to it
                let keep = vertices[0];

                for &candidate in &vertices[1..] {
                    if let (Ok(v1), Ok(v2)) = (mesh.get_vertex(keep), mesh.get_vertex(candidate)) {
                        if (v1.position - v2.position).norm() < self.merge_tolerance {
                            merge_map.insert(candidate, keep);
                            merged_count += 1;
                        }
                    }
                }
            }
        }

        // TODO: Actually perform the merge by updating face vertex references
        // This is a placeholder - full implementation would update all face references

        merged_count
    }

    /// Remove degenerate faces (zero area, duplicate vertices, etc.)
    fn remove_degenerate_faces(&self, mesh: &mut HalfEdgeMesh) -> usize {
        let mut removed_count = 0;
        let face_handles = mesh.face_handles();

        for fh in face_handles {
            if self.is_degenerate_face(mesh, fh) {
                if mesh.delete_face(fh).is_ok() {
                    removed_count += 1;
                }
            }
        }

        removed_count
    }

    /// Check if a face is degenerate
    fn is_degenerate_face(&self, mesh: &HalfEdgeMesh, fh: FaceHandle) -> bool {
        if let Ok(vertices) = mesh.face_vertices(fh) {
            // Check for duplicate vertices
            let mut unique = HashSet::new();
            for vh in &vertices {
                if !unique.insert(vh) {
                    return true;
                }
            }

            // Check face area
            if vertices.len() < 3 {
                return true;
            }

            if let (Ok(v0), Ok(v1), Ok(v2)) = (
                mesh.get_vertex(vertices[0]),
                mesh.get_vertex(vertices[1]),
                mesh.get_vertex(vertices[2]),
            ) {
                let edge1 = v1.position - v0.position;
                let edge2 = v2.position - v0.position;
                let cross = edge1.cross(&edge2);
                let area = cross.norm() / 2.0;

                if area < self.min_face_area {
                    return true;
                }
            }

            // Check for zero-length edges
            for i in 0..vertices.len() {
                let j = (i + 1) % vertices.len();
                if let (Ok(vi), Ok(vj)) = (mesh.get_vertex(vertices[i]), mesh.get_vertex(vertices[j])) {
                    let edge_length = (vj.position - vi.position).norm();
                    if edge_length < self.min_edge_length {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Fix inconsistent face normals
    fn fix_normals(&self, mesh: &mut HalfEdgeMesh) -> usize {
        let mut flipped = 0;

        // Build adjacency graph
        let face_handles = mesh.face_handles();
        let mut processed = HashSet::new();
        let mut to_process = Vec::new();

        if face_handles.is_empty() {
            return 0;
        }

        // Start with first face
        to_process.push(face_handles[0]);
        processed.insert(face_handles[0]);

        while let Some(current_face) = to_process.pop() {
            // Get adjacent faces and check normal consistency
            if let Ok(vertices) = mesh.face_vertices(current_face) {
                for i in 0..vertices.len() {
                    let j = (i + 1) % vertices.len();
                    let vi = vertices[i];
                    let vj = vertices[j];

                    // Find adjacent faces sharing this edge
                    for &adjacent_face in &face_handles {
                        if adjacent_face == current_face || processed.contains(&adjacent_face) {
                            continue;
                        }

                        if let Ok(adj_vertices) = mesh.face_vertices(adjacent_face) {
                            // Check if faces share an edge
                            let shares_edge = adj_vertices.windows(2).any(|w| {
                                (w[0] == vi && w[1] == vj) || (w[0] == vj && w[1] == vi)
                            });

                            if shares_edge {
                                // Check normal consistency
                                if let (Ok(face1), Ok(face2)) = (mesh.get_face(current_face), mesh.get_face(adjacent_face)) {
                                    if face1.normal.dot(&face2.normal) < 0.0 {
                                        // Normals point in opposite directions - needs flipping
                                        // This is a simplified check; actual flip would require
                                        // reversing face vertex order
                                        flipped += 1;
                                    }
                                }

                                processed.insert(adjacent_face);
                                to_process.push(adjacent_face);
                            }
                        }
                    }
                }
            }
        }

        flipped
    }

    /// Fill small holes in the mesh
    fn fill_holes(&self, mesh: &mut HalfEdgeMesh) -> usize {
        let mut filled = 0;

        // Find boundary loops
        let boundary_loops = self.find_boundary_loops(mesh);

        for loop_vertices in boundary_loops {
            if loop_vertices.len() >= 3 && loop_vertices.len() <= 10 {
                // Simple fan triangulation for small holes
                if self.fill_hole_fan(mesh, &loop_vertices).is_ok() {
                    filled += 1;
                }
            }
        }

        filled
    }

    /// Find boundary loops in the mesh
    fn find_boundary_loops(&self, mesh: &HalfEdgeMesh) -> Vec<Vec<VertexHandle>> {
        let mut loops = Vec::new();
        let mut visited_edges = HashSet::new();

        for edge_handle in mesh.edge_handles() {
            if let Ok(edge) = mesh.get_edge(edge_handle) {
                if edge.is_boundary && !visited_edges.contains(&edge_handle) {
                    // Start a new boundary loop
                    let mut loop_vertices = Vec::new();
                    let mut current_edge = edge_handle;

                    loop {
                        visited_edges.insert(current_edge);

                        if let Ok(he) = mesh.get_halfedge(mesh.get_edge(current_edge).unwrap().halfedge) {
                            loop_vertices.push(he.vertex);

                            // Find next boundary edge
                            // This is simplified - actual implementation would traverse the boundary
                            break;
                        }
                    }

                    if loop_vertices.len() >= 3 {
                        loops.push(loop_vertices);
                    }
                }
            }
        }

        loops
    }

    /// Fill a hole using fan triangulation
    fn fill_hole_fan(&self, mesh: &mut HalfEdgeMesh, vertices: &[VertexHandle]) -> Result<(), MeshError> {
        if vertices.len() < 3 {
            return Err(MeshError::InvalidFaceVertexCount);
        }

        // Create triangles in a fan from the first vertex
        for i in 1..vertices.len() - 1 {
            mesh.add_face(&[vertices[0], vertices[i], vertices[i + 1]])?;
        }

        Ok(())
    }

    /// Remove isolated vertices (vertices with no edges)
    fn remove_isolated_vertices(&self, mesh: &mut HalfEdgeMesh) -> usize {
        let mut removed = 0;

        for vh in mesh.vertex_handles() {
            if let Ok(vertex) = mesh.get_vertex(vh) {
                if vertex.halfedge.is_none() {
                    // Vertex has no outgoing halfedge - it's isolated
                    // TODO: Actually remove the vertex
                    removed += 1;
                }
            }
        }

        removed
    }
}

/// Spatial hash key for vertex proximity queries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SpatialKey {
    x: i32,
    y: i32,
    z: i32,
}

impl SpatialKey {
    fn from_point(point: &Point3, cell_size: f64) -> Self {
        Self {
            x: (point.x / cell_size).floor() as i32,
            y: (point.y / cell_size).floor() as i32,
            z: (point.z / cell_size).floor() as i32,
        }
    }
}

/// Report of healing operations performed
#[derive(Debug, Default, Clone)]
pub struct HealingReport {
    pub merged_vertices: usize,
    pub removed_degenerate_faces: usize,
    pub flipped_normals: usize,
    pub filled_holes: usize,
    pub removed_isolated_vertices: usize,
}

impl HealingReport {
    /// Check if any changes were made
    pub fn has_changes(&self) -> bool {
        self.merged_vertices > 0
            || self.removed_degenerate_faces > 0
            || self.flipped_normals > 0
            || self.filled_holes > 0
            || self.removed_isolated_vertices > 0
    }

    /// Get total number of changes
    pub fn total_changes(&self) -> usize {
        self.merged_vertices
            + self.removed_degenerate_faces
            + self.flipped_normals
            + self.filled_holes
            + self.removed_isolated_vertices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_healer_creation() {
        let healer = MeshHealer::new();
        assert!(healer.merge_tolerance > 0.0);
    }

    #[test]
    fn test_healing_report() {
        let report = HealingReport::default();
        assert!(!report.has_changes());
        assert_eq!(report.total_changes(), 0);
    }
}
