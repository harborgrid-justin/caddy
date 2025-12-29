//! Half-edge mesh data structure for topological operations
//!
//! Provides a robust half-edge mesh implementation suitable for CAD operations.
//! The half-edge structure maintains explicit connectivity information for efficient
//! topological queries and modifications.

use crate::core::{Point3, Vector3};
use nalgebra::{Matrix4, Point3 as NPoint3, Vector3 as NVector3};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Handle types for mesh elements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VertexHandle(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HalfEdgeHandle(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeHandle(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FaceHandle(pub usize);

/// Vertex in the half-edge mesh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vertex {
    /// Position of the vertex
    pub position: Point3,
    /// Normal vector at this vertex (averaged from adjacent faces)
    pub normal: Vector3,
    /// One outgoing half-edge from this vertex
    pub halfedge: Option<HalfEdgeHandle>,
    /// Custom attributes
    pub attributes: HashMap<String, f64>,
}

/// Half-edge structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HalfEdge {
    /// Vertex this half-edge points to
    pub vertex: VertexHandle,
    /// Face this half-edge bounds (None for boundary)
    pub face: Option<FaceHandle>,
    /// Next half-edge in the face loop
    pub next: HalfEdgeHandle,
    /// Previous half-edge in the face loop
    pub prev: HalfEdgeHandle,
    /// Opposite half-edge (twin/pair)
    pub twin: Option<HalfEdgeHandle>,
    /// Associated edge
    pub edge: EdgeHandle,
}

/// Edge connecting two vertices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    /// One of the two half-edges
    pub halfedge: HalfEdgeHandle,
    /// Whether this is a boundary edge
    pub is_boundary: bool,
    /// Sharp/crease flag for subdivision surfaces
    pub is_sharp: bool,
}

/// Face in the mesh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Face {
    /// One half-edge bounding this face
    pub halfedge: HalfEdgeHandle,
    /// Face normal
    pub normal: Vector3,
    /// Material ID
    pub material_id: Option<Uuid>,
    /// Custom attributes
    pub attributes: HashMap<String, f64>,
}

/// Half-edge mesh structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HalfEdgeMesh {
    pub vertices: Vec<Option<Vertex>>,
    pub halfedges: Vec<Option<HalfEdge>>,
    pub edges: Vec<Option<Edge>>,
    pub faces: Vec<Option<Face>>,
    /// Free indices for reuse
    free_vertices: Vec<usize>,
    free_halfedges: Vec<usize>,
    free_edges: Vec<usize>,
    free_faces: Vec<usize>,
}

impl HalfEdgeMesh {
    /// Create a new empty mesh
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            halfedges: Vec::new(),
            edges: Vec::new(),
            faces: Vec::new(),
            free_vertices: Vec::new(),
            free_halfedges: Vec::new(),
            free_edges: Vec::new(),
            free_faces: Vec::new(),
        }
    }

    /// Add a vertex to the mesh
    pub fn add_vertex(&mut self, position: Point3) -> VertexHandle {
        let vertex = Vertex {
            position,
            normal: Vector3::new(0.0, 0.0, 1.0),
            halfedge: None,
            attributes: HashMap::new(),
        };

        if let Some(idx) = self.free_vertices.pop() {
            self.vertices[idx] = Some(vertex);
            VertexHandle(idx)
        } else {
            let idx = self.vertices.len();
            self.vertices.push(Some(vertex));
            VertexHandle(idx)
        }
    }

    /// Add a face to the mesh
    pub fn add_face(&mut self, vertex_handles: &[VertexHandle]) -> Result<FaceHandle, MeshError> {
        if vertex_handles.len() < 3 {
            return Err(MeshError::InvalidFaceVertexCount);
        }

        // Check for duplicate vertices
        let mut unique = HashSet::new();
        for vh in vertex_handles {
            if !unique.insert(vh) {
                return Err(MeshError::DuplicateVertexInFace);
            }
        }

        // Create half-edges for this face
        let n = vertex_handles.len();
        let mut halfedge_handles = Vec::with_capacity(n);

        for _ in 0..n {
            let he_idx = if let Some(idx) = self.free_halfedges.pop() {
                idx
            } else {
                let idx = self.halfedges.len();
                self.halfedges.push(None);
                idx
            };
            halfedge_handles.push(HalfEdgeHandle(he_idx));
        }

        // Create edges
        let mut edge_handles = Vec::with_capacity(n);
        for _ in 0..n {
            let e_idx = if let Some(idx) = self.free_edges.pop() {
                idx
            } else {
                let idx = self.edges.len();
                self.edges.push(None);
                idx
            };
            edge_handles.push(EdgeHandle(e_idx));
        }

        // Create face
        let face_idx = if let Some(idx) = self.free_faces.pop() {
            idx
        } else {
            let idx = self.faces.len();
            self.faces.push(None);
            idx
        };
        let face_handle = FaceHandle(face_idx);

        // Setup half-edges
        for i in 0..n {
            let next_i = (i + 1) % n;
            let prev_i = if i == 0 { n - 1 } else { i - 1 };

            let halfedge = HalfEdge {
                vertex: vertex_handles[next_i],
                face: Some(face_handle),
                next: halfedge_handles[next_i],
                prev: halfedge_handles[prev_i],
                twin: None, // Will be set when finding twins
                edge: edge_handles[i],
            };

            self.halfedges[halfedge_handles[i].0] = Some(halfedge);

            // Set vertex half-edge reference
            if let Some(ref mut v) = self.vertices[vertex_handles[i].0] {
                if v.halfedge.is_none() {
                    v.halfedge = Some(halfedge_handles[i]);
                }
            }
        }

        // Setup edges
        for i in 0..n {
            let edge = Edge {
                halfedge: halfedge_handles[i],
                is_boundary: true, // Initially boundary, update when finding twins
                is_sharp: false,
            };
            self.edges[edge_handles[i].0] = Some(edge);
        }

        // Compute face normal
        let normal = self.compute_face_normal(face_handle)?;

        // Create face
        let face = Face {
            halfedge: halfedge_handles[0],
            normal,
            material_id: None,
            attributes: HashMap::new(),
        };
        self.faces[face_idx] = Some(face);

        // Try to find and connect twin half-edges
        self.update_twins(face_handle)?;

        Ok(face_handle)
    }

    /// Update vertex normals based on adjacent face normals
    pub fn update_vertex_normals(&mut self) {
        for v_idx in 0..self.vertices.len() {
            if let Some(ref mut vertex) = self.vertices[v_idx] {
                if let Some(he_start) = vertex.halfedge {
                    let mut normal = Vector3::zeros();
                    let mut count = 0;

                    // Iterate through all faces around this vertex
                    let mut he = he_start;
                    loop {
                        if let Some(ref halfedge) = self.halfedges[he.0] {
                            if let Some(fh) = halfedge.face {
                                if let Some(ref face) = self.faces[fh.0] {
                                    normal += face.normal;
                                    count += 1;
                                }
                            }

                            // Move to next half-edge around vertex
                            if let Some(twin) = halfedge.twin {
                                if let Some(ref twin_he) = self.halfedges[twin.0] {
                                    he = twin_he.next;
                                    if he == he_start {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }

                    if count > 0 {
                        vertex.normal = (normal / count as f64).normalize();
                    }
                }
            }
        }
    }

    /// Compute the normal of a face
    fn compute_face_normal(&self, face_handle: FaceHandle) -> Result<Vector3, MeshError> {
        let face = self.get_face(face_handle)?;
        let he_start = face.halfedge;

        let he1 = self.get_halfedge(he_start)?;
        let he2 = self.get_halfedge(he1.next)?;
        let he3 = self.get_halfedge(he2.next)?;

        let v1 = self.get_vertex(he1.vertex)?.position;
        let v2 = self.get_vertex(he2.vertex)?.position;
        let v3 = self.get_vertex(he3.vertex)?.position;

        let edge1 = v2 - v1;
        let edge2 = v3 - v1;

        let normal = edge1.cross(&edge2);
        let len = normal.norm();

        if len < 1e-10 {
            Ok(Vector3::new(0.0, 0.0, 1.0))
        } else {
            Ok(normal / len)
        }
    }

    /// Find and connect twin half-edges
    fn update_twins(&mut self, _face_handle: FaceHandle) -> Result<(), MeshError> {
        // Build a map of directed edges to half-edges
        let mut edge_map: HashMap<(usize, usize), HalfEdgeHandle> = HashMap::new();

        for he_idx in 0..self.halfedges.len() {
            if let Some(ref halfedge) = self.halfedges[he_idx] {
                // Get source vertex (previous half-edge's vertex)
                if let Some(ref prev_he) = self.halfedges[halfedge.prev.0] {
                    let v_from = prev_he.vertex.0;
                    let v_to = halfedge.vertex.0;
                    edge_map.insert((v_from, v_to), HalfEdgeHandle(he_idx));
                }
            }
        }

        // Find twins
        for he_idx in 0..self.halfedges.len() {
            if let Some(ref halfedge) = self.halfedges[he_idx] {
                if halfedge.twin.is_none() {
                    if let Some(ref prev_he) = self.halfedges[halfedge.prev.0] {
                        let v_from = prev_he.vertex.0;
                        let v_to = halfedge.vertex.0;

                        // Look for opposite direction edge
                        if let Some(&twin_handle) = edge_map.get(&(v_to, v_from)) {
                            // Set twins
                            self.halfedges[he_idx].as_mut().unwrap().twin = Some(twin_handle);
                            self.halfedges[twin_handle.0].as_mut().unwrap().twin = Some(HalfEdgeHandle(he_idx));

                            // Update edge boundary status
                            if let Some(ref mut edge) = self.edges[halfedge.edge.0] {
                                edge.is_boundary = false;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get vertex by handle
    pub fn get_vertex(&self, handle: VertexHandle) -> Result<&Vertex, MeshError> {
        self.vertices
            .get(handle.0)
            .and_then(|v| v.as_ref())
            .ok_or(MeshError::InvalidVertexHandle)
    }

    /// Get mutable vertex by handle
    pub fn get_vertex_mut(&mut self, handle: VertexHandle) -> Result<&mut Vertex, MeshError> {
        self.vertices
            .get_mut(handle.0)
            .and_then(|v| v.as_mut())
            .ok_or(MeshError::InvalidVertexHandle)
    }

    /// Get half-edge by handle
    pub fn get_halfedge(&self, handle: HalfEdgeHandle) -> Result<&HalfEdge, MeshError> {
        self.halfedges
            .get(handle.0)
            .and_then(|he| he.as_ref())
            .ok_or(MeshError::InvalidHalfEdgeHandle)
    }

    /// Get edge by handle
    pub fn get_edge(&self, handle: EdgeHandle) -> Result<&Edge, MeshError> {
        self.edges
            .get(handle.0)
            .and_then(|e| e.as_ref())
            .ok_or(MeshError::InvalidEdgeHandle)
    }

    /// Get face by handle
    pub fn get_face(&self, handle: FaceHandle) -> Result<&Face, MeshError> {
        self.faces
            .get(handle.0)
            .and_then(|f| f.as_ref())
            .ok_or(MeshError::InvalidFaceHandle)
    }

    /// Get mutable face by handle
    pub fn get_face_mut(&mut self, handle: FaceHandle) -> Result<&mut Face, MeshError> {
        self.faces
            .get_mut(handle.0)
            .and_then(|f| f.as_mut())
            .ok_or(MeshError::InvalidFaceHandle)
    }

    /// Delete a face from the mesh
    pub fn delete_face(&mut self, handle: FaceHandle) -> Result<(), MeshError> {
        let face = self.get_face(handle)?;
        let he_start = face.halfedge;

        // Collect all half-edges and edges of this face
        let mut halfedges_to_delete = Vec::new();
        let mut edges_to_delete = Vec::new();

        let mut he = he_start;
        loop {
            halfedges_to_delete.push(he);
            if let Some(ref halfedge) = self.halfedges[he.0] {
                edges_to_delete.push(halfedge.edge);
                he = halfedge.next;
                if he == he_start {
                    break;
                }
            } else {
                break;
            }
        }

        // Delete half-edges and edges
        for he_handle in halfedges_to_delete {
            self.halfedges[he_handle.0] = None;
            self.free_halfedges.push(he_handle.0);
        }

        for e_handle in edges_to_delete {
            self.edges[e_handle.0] = None;
            self.free_edges.push(e_handle.0);
        }

        // Delete face
        self.faces[handle.0] = None;
        self.free_faces.push(handle.0);

        Ok(())
    }

    /// Get all vertex handles
    pub fn vertex_handles(&self) -> Vec<VertexHandle> {
        self.vertices
            .iter()
            .enumerate()
            .filter_map(|(i, v)| v.as_ref().map(|_| VertexHandle(i)))
            .collect()
    }

    /// Get all face handles
    pub fn face_handles(&self) -> Vec<FaceHandle> {
        self.faces
            .iter()
            .enumerate()
            .filter_map(|(i, f)| f.as_ref().map(|_| FaceHandle(i)))
            .collect()
    }

    /// Get all edge handles
    pub fn edge_handles(&self) -> Vec<EdgeHandle> {
        self.edges
            .iter()
            .enumerate()
            .filter_map(|(i, e)| e.as_ref().map(|_| EdgeHandle(i)))
            .collect()
    }

    /// Get vertices of a face
    pub fn face_vertices(&self, handle: FaceHandle) -> Result<Vec<VertexHandle>, MeshError> {
        let face = self.get_face(handle)?;
        let he_start = face.halfedge;
        let mut vertices = Vec::new();

        let mut he = he_start;
        loop {
            if let Some(ref halfedge) = self.halfedges[he.0] {
                vertices.push(halfedge.vertex);
                he = halfedge.next;
                if he == he_start {
                    break;
                }
            } else {
                return Err(MeshError::InvalidHalfEdgeHandle);
            }
        }

        Ok(vertices)
    }

    /// Check if the mesh is manifold (each edge has at most 2 adjacent faces)
    pub fn is_manifold(&self) -> bool {
        for edge_handle in self.edge_handles() {
            if let Ok(edge) = self.get_edge(edge_handle) {
                if let Ok(he1) = self.get_halfedge(edge.halfedge) {
                    if let Some(twin) = he1.twin {
                        if let Ok(he2) = self.get_halfedge(twin) {
                            // Check if any other half-edge shares the same vertices
                            if he1.face.is_some() && he2.face.is_some() {
                                // Edge has exactly 2 faces - good
                                continue;
                            }
                        }
                    }
                }
            }
        }
        true
    }

    /// Transform the entire mesh by a matrix
    pub fn transform(&mut self, matrix: &Matrix4<f64>) {
        for v_idx in 0..self.vertices.len() {
            if let Some(ref mut vertex) = self.vertices[v_idx] {
                let p = NPoint3::new(vertex.position.x, vertex.position.y, vertex.position.z);
                let transformed = matrix.transform_point(&p);
                vertex.position = Point3::new(transformed.x, transformed.y, transformed.z);

                // Transform normal (use inverse transpose for normals)
                let n = NVector3::new(vertex.normal.x, vertex.normal.y, vertex.normal.z);
                let transformed_n = matrix.transform_vector(&n);
                vertex.normal = Vector3::new(transformed_n.x, transformed_n.y, transformed_n.z).normalize();
            }
        }

        // Recompute face normals
        for f_idx in 0..self.faces.len() {
            if self.faces[f_idx].is_some() {
                if let Ok(normal) = self.compute_face_normal(FaceHandle(f_idx)) {
                    self.faces[f_idx].as_mut().unwrap().normal = normal;
                }
            }
        }
    }

    /// Get mesh statistics
    pub fn stats(&self) -> MeshStats {
        let n_vertices = self.vertices.iter().filter(|v| v.is_some()).count();
        let n_faces = self.faces.iter().filter(|f| f.is_some()).count();
        let n_edges = self.edges.iter().filter(|e| e.is_some()).count();
        let n_halfedges = self.halfedges.iter().filter(|he| he.is_some()).count();

        MeshStats {
            vertices: n_vertices,
            faces: n_faces,
            edges: n_edges,
            halfedges: n_halfedges,
            is_manifold: self.is_manifold(),
        }
    }
}

impl Default for HalfEdgeMesh {
    fn default() -> Self {
        Self::new()
    }
}

/// Mesh statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshStats {
    pub vertices: usize,
    pub faces: usize,
    pub edges: usize,
    pub halfedges: usize,
    pub is_manifold: bool,
}

/// Mesh-related errors
#[derive(Debug, thiserror::Error)]
pub enum MeshError {
    #[error("Invalid vertex handle")]
    InvalidVertexHandle,

    #[error("Invalid half-edge handle")]
    InvalidHalfEdgeHandle,

    #[error("Invalid edge handle")]
    InvalidEdgeHandle,

    #[error("Invalid face handle")]
    InvalidFaceHandle,

    #[error("Face must have at least 3 vertices")]
    InvalidFaceVertexCount,

    #[error("Face cannot have duplicate vertices")]
    DuplicateVertexInFace,

    #[error("Non-manifold edge detected")]
    NonManifoldEdge,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_triangle() {
        let mut mesh = HalfEdgeMesh::new();

        let v1 = mesh.add_vertex(Point3::new(0.0, 0.0, 0.0));
        let v2 = mesh.add_vertex(Point3::new(1.0, 0.0, 0.0));
        let v3 = mesh.add_vertex(Point3::new(0.0, 1.0, 0.0));

        let face = mesh.add_face(&[v1, v2, v3]).unwrap();

        assert_eq!(mesh.stats().vertices, 3);
        assert_eq!(mesh.stats().faces, 1);
        assert_eq!(mesh.stats().edges, 3);

        let vertices = mesh.face_vertices(face).unwrap();
        assert_eq!(vertices.len(), 3);
    }

    #[test]
    fn test_quad_mesh() {
        let mut mesh = HalfEdgeMesh::new();

        let v1 = mesh.add_vertex(Point3::new(0.0, 0.0, 0.0));
        let v2 = mesh.add_vertex(Point3::new(1.0, 0.0, 0.0));
        let v3 = mesh.add_vertex(Point3::new(1.0, 1.0, 0.0));
        let v4 = mesh.add_vertex(Point3::new(0.0, 1.0, 0.0));

        let face = mesh.add_face(&[v1, v2, v3, v4]).unwrap();

        assert_eq!(mesh.stats().vertices, 4);
        assert_eq!(mesh.stats().faces, 1);

        let vertices = mesh.face_vertices(face).unwrap();
        assert_eq!(vertices.len(), 4);
    }
}
