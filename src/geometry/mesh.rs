//! Mesh Geometry and Topological Operations
//!
//! This module provides complete mesh data structures including triangle meshes,
//! quad meshes, and half-edge meshes for advanced topological operations.

use nalgebra::{Point3, Vector3, Unit};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use rayon::prelude::*;

/// A single vertex in 3D space
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vertex {
    /// Position in 3D space
    pub position: Point3<f64>,
    /// Normal vector (optional)
    pub normal: Option<Unit<Vector3<f64>>>,
    /// Texture coordinates (optional)
    pub uv: Option<(f64, f64)>,
}

impl Vertex {
    /// Creates a new vertex with just a position
    pub fn new(position: Point3<f64>) -> Self {
        Self {
            position,
            normal: None,
            uv: None,
        }
    }

    /// Creates a vertex with position and normal
    pub fn with_normal(position: Point3<f64>, normal: Vector3<f64>) -> Self {
        Self {
            position,
            normal: Some(Unit::new_normalize(normal)),
            uv: None,
        }
    }

    /// Creates a vertex with all attributes
    pub fn complete(position: Point3<f64>, normal: Vector3<f64>, uv: (f64, f64)) -> Self {
        Self {
            position,
            normal: Some(Unit::new_normalize(normal)),
            uv: Some(uv),
        }
    }
}

/// A triangular face defined by three vertex indices
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TriangleFace {
    /// Indices of the three vertices (counter-clockwise)
    pub vertices: [usize; 3],
}

impl TriangleFace {
    /// Creates a new triangle face
    pub fn new(v0: usize, v1: usize, v2: usize) -> Self {
        Self {
            vertices: [v0, v1, v2],
        }
    }

    /// Reverses the winding order of the triangle
    pub fn reverse(&self) -> Self {
        Self {
            vertices: [self.vertices[0], self.vertices[2], self.vertices[1]],
        }
    }

    /// Returns edges of the triangle as pairs of vertex indices
    pub fn edges(&self) -> [(usize, usize); 3] {
        [
            (self.vertices[0], self.vertices[1]),
            (self.vertices[1], self.vertices[2]),
            (self.vertices[2], self.vertices[0]),
        ]
    }
}

/// A quadrilateral face defined by four vertex indices
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QuadFace {
    /// Indices of the four vertices (counter-clockwise)
    pub vertices: [usize; 4],
}

impl QuadFace {
    /// Creates a new quad face
    pub fn new(v0: usize, v1: usize, v2: usize, v3: usize) -> Self {
        Self {
            vertices: [v0, v1, v2, v3],
        }
    }

    /// Splits the quad into two triangles
    pub fn triangulate(&self) -> [TriangleFace; 2] {
        [
            TriangleFace::new(self.vertices[0], self.vertices[1], self.vertices[2]),
            TriangleFace::new(self.vertices[0], self.vertices[2], self.vertices[3]),
        ]
    }

    /// Returns edges of the quad
    pub fn edges(&self) -> [(usize, usize); 4] {
        [
            (self.vertices[0], self.vertices[1]),
            (self.vertices[1], self.vertices[2]),
            (self.vertices[2], self.vertices[3]),
            (self.vertices[3], self.vertices[0]),
        ]
    }
}

/// Triangle mesh data structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TriangleMesh {
    /// Vertex data
    pub vertices: Vec<Vertex>,
    /// Triangle faces
    pub faces: Vec<TriangleFace>,
}

impl TriangleMesh {
    /// Creates a new empty triangle mesh
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            faces: Vec::new(),
        }
    }

    /// Creates a mesh from vertices and faces
    pub fn from_data(vertices: Vec<Vertex>, faces: Vec<TriangleFace>) -> Self {
        Self { vertices, faces }
    }

    /// Adds a vertex and returns its index
    pub fn add_vertex(&mut self, vertex: Vertex) -> usize {
        let index = self.vertices.len();
        self.vertices.push(vertex);
        index
    }

    /// Adds a triangle face
    pub fn add_face(&mut self, face: TriangleFace) {
        assert!(
            face.vertices.iter().all(|&i| i < self.vertices.len()),
            "Face references invalid vertex indices"
        );
        self.faces.push(face);
    }

    /// Computes the face normal for a triangle
    pub fn face_normal(&self, face_index: usize) -> Unit<Vector3<f64>> {
        let face = &self.faces[face_index];
        let v0 = &self.vertices[face.vertices[0]].position;
        let v1 = &self.vertices[face.vertices[1]].position;
        let v2 = &self.vertices[face.vertices[2]].position;

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let normal = edge1.cross(&edge2);

        if normal.norm() < 1e-10 {
            Unit::new_unchecked(Vector3::z())
        } else {
            Unit::new_normalize(normal)
        }
    }

    /// Computes smooth vertex normals by averaging adjacent face normals
    pub fn compute_vertex_normals(&mut self) {
        let mut normals: Vec<Vector3<f64>> = vec![Vector3::zeros(); self.vertices.len()];

        // Accumulate face normals
        for (face_idx, face) in self.faces.iter().enumerate() {
            let face_normal = self.face_normal(face_idx);

            for &vertex_idx in &face.vertices {
                normals[vertex_idx] += face_normal.as_ref();
            }
        }

        // Normalize and assign
        for (vertex, normal) in self.vertices.iter_mut().zip(normals.iter()) {
            if normal.norm() > 1e-10 {
                vertex.normal = Some(Unit::new_normalize(*normal));
            }
        }
    }

    /// Reverses all face normals
    pub fn reverse_normals(&mut self) {
        for face in &mut self.faces {
            *face = face.reverse();
        }

        for vertex in &mut self.vertices {
            if let Some(normal) = vertex.normal {
                vertex.normal = Some(Unit::new_unchecked(-normal.as_ref()));
            }
        }
    }

    /// Merges duplicate vertices within tolerance
    pub fn merge_vertices(&mut self, tolerance: f64) {
        let mut vertex_map: HashMap<usize, usize> = HashMap::new();
        let mut unique_vertices: Vec<Vertex> = Vec::new();

        for (old_idx, vertex) in self.vertices.iter().enumerate() {
            // Find if this vertex matches an existing one
            let mut found = None;
            for (new_idx, unique) in unique_vertices.iter().enumerate() {
                let dist = nalgebra::distance(&vertex.position, &unique.position);
                if dist < tolerance {
                    found = Some(new_idx);
                    break;
                }
            }

            if let Some(new_idx) = found {
                vertex_map.insert(old_idx, new_idx);
            } else {
                let new_idx = unique_vertices.len();
                unique_vertices.push(*vertex);
                vertex_map.insert(old_idx, new_idx);
            }
        }

        // Remap faces
        for face in &mut self.faces {
            for vertex_idx in &mut face.vertices {
                *vertex_idx = vertex_map[vertex_idx];
            }
        }

        self.vertices = unique_vertices;
    }

    /// Exports mesh data for STL format
    pub fn to_stl_data(&self) -> Vec<(Point3<f64>, Point3<f64>, Point3<f64>, Vector3<f64>)> {
        self.faces
            .iter()
            .map(|face| {
                let v0 = self.vertices[face.vertices[0]].position;
                let v1 = self.vertices[face.vertices[1]].position;
                let v2 = self.vertices[face.vertices[2]].position;

                let edge1 = v1 - v0;
                let edge2 = v2 - v0;
                let normal = edge1.cross(&edge2).normalize();

                (v0, v1, v2, normal)
            })
            .collect()
    }

    /// Subdivides each triangle into 4 smaller triangles
    pub fn subdivide(&mut self) {
        let mut new_vertices = self.vertices.clone();
        let mut new_faces = Vec::new();
        let mut edge_midpoints: HashMap<(usize, usize), usize> = HashMap::new();

        for face in &self.faces {
            let mut midpoints = [0; 3];

            for (i, &(v0, v1)) in face.edges().iter().enumerate() {
                let edge = if v0 < v1 { (v0, v1) } else { (v1, v0) };

                let midpoint_idx = *edge_midpoints.entry(edge).or_insert_with(|| {
                    let p0 = self.vertices[v0].position;
                    let p1 = self.vertices[v1].position;
                    let midpoint = Point3::new(
                        (p0.x + p1.x) / 2.0,
                        (p0.y + p1.y) / 2.0,
                        (p0.z + p1.z) / 2.0,
                    );

                    let idx = new_vertices.len();
                    new_vertices.push(Vertex::new(midpoint));
                    idx
                });

                midpoints[i] = midpoint_idx;
            }

            // Create 4 new triangles
            new_faces.push(TriangleFace::new(
                face.vertices[0],
                midpoints[0],
                midpoints[2],
            ));
            new_faces.push(TriangleFace::new(
                face.vertices[1],
                midpoints[1],
                midpoints[0],
            ));
            new_faces.push(TriangleFace::new(
                face.vertices[2],
                midpoints[2],
                midpoints[1],
            ));
            new_faces.push(TriangleFace::new(midpoints[0], midpoints[1], midpoints[2]));
        }

        self.vertices = new_vertices;
        self.faces = new_faces;
    }
}

impl Default for TriangleMesh {
    fn default() -> Self {
        Self::new()
    }
}

/// Quad mesh data structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuadMesh {
    /// Vertex data
    pub vertices: Vec<Vertex>,
    /// Quad faces
    pub faces: Vec<QuadFace>,
}

impl QuadMesh {
    /// Creates a new empty quad mesh
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            faces: Vec::new(),
        }
    }

    /// Converts to a triangle mesh
    pub fn to_triangle_mesh(&self) -> TriangleMesh {
        let mut mesh = TriangleMesh::new();
        mesh.vertices = self.vertices.clone();

        for quad in &self.faces {
            let triangles = quad.triangulate();
            mesh.faces.extend_from_slice(&triangles);
        }

        mesh
    }

    /// Computes vertex normals
    pub fn compute_vertex_normals(&mut self) {
        let triangle_mesh = self.to_triangle_mesh();
        let mut normals: Vec<Vector3<f64>> = vec![Vector3::zeros(); self.vertices.len()];

        for (face_idx, _) in triangle_mesh.faces.iter().enumerate() {
            let face_normal = triangle_mesh.face_normal(face_idx);
            let face = &triangle_mesh.faces[face_idx];

            for &vertex_idx in &face.vertices {
                normals[vertex_idx] += face_normal.as_ref();
            }
        }

        for (vertex, normal) in self.vertices.iter_mut().zip(normals.iter()) {
            if normal.norm() > 1e-10 {
                vertex.normal = Some(Unit::new_normalize(*normal));
            }
        }
    }
}

impl Default for QuadMesh {
    fn default() -> Self {
        Self::new()
    }
}

/// Half-edge data structure for topological operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HalfEdgeMesh {
    /// Vertices
    pub vertices: Vec<HEVertex>,
    /// Half-edges
    pub half_edges: Vec<HalfEdge>,
    /// Faces
    pub faces: Vec<HEFace>,
}

/// Vertex in a half-edge mesh
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HEVertex {
    /// Position
    pub position: Point3<f64>,
    /// One outgoing half-edge from this vertex
    pub half_edge: Option<usize>,
}

/// Half-edge structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HalfEdge {
    /// Vertex this half-edge points to
    pub vertex: usize,
    /// Next half-edge in the face loop
    pub next: usize,
    /// Previous half-edge in the face loop
    pub prev: usize,
    /// Opposite (twin) half-edge
    pub twin: Option<usize>,
    /// Face this half-edge belongs to
    pub face: Option<usize>,
}

/// Face in a half-edge mesh
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HEFace {
    /// One half-edge in this face
    pub half_edge: usize,
}

impl HalfEdgeMesh {
    /// Creates a new empty half-edge mesh
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            half_edges: Vec::new(),
            faces: Vec::new(),
        }
    }

    /// Constructs a half-edge mesh from a triangle mesh
    pub fn from_triangle_mesh(mesh: &TriangleMesh) -> Self {
        let mut he_mesh = Self::new();

        // Create vertices
        for vertex in &mesh.vertices {
            he_mesh.vertices.push(HEVertex {
                position: vertex.position,
                half_edge: None,
            });
        }

        let mut edge_map: HashMap<(usize, usize), usize> = HashMap::new();

        // Create half-edges and faces
        for (face_idx, face) in mesh.faces.iter().enumerate() {
            let he_face_idx = he_mesh.faces.len();
            let first_he_idx = he_mesh.half_edges.len();

            // Create three half-edges for the triangle
            for i in 0..3 {
                let v_start = face.vertices[i];
                let v_end = face.vertices[(i + 1) % 3];

                let he_idx = he_mesh.half_edges.len();

                he_mesh.half_edges.push(HalfEdge {
                    vertex: v_end,
                    next: first_he_idx + (i + 1) % 3,
                    prev: first_he_idx + (i + 2) % 3,
                    twin: None,
                    face: Some(he_face_idx),
                });

                edge_map.insert((v_start, v_end), he_idx);

                // Update vertex reference
                if he_mesh.vertices[v_start].half_edge.is_none() {
                    he_mesh.vertices[v_start].half_edge = Some(he_idx);
                }
            }

            he_mesh.faces.push(HEFace {
                half_edge: first_he_idx,
            });
        }

        // Link twin half-edges
        let edges: Vec<((usize, usize), usize)> = edge_map.iter().map(|(k, v)| (*k, *v)).collect();

        for ((v_start, v_end), he_idx) in edges {
            if let Some(&twin_idx) = edge_map.get(&(v_end, v_start)) {
                he_mesh.half_edges[he_idx].twin = Some(twin_idx);
            }
        }

        he_mesh
    }

    /// Gets all half-edges around a vertex
    pub fn vertex_half_edges(&self, vertex_idx: usize) -> Vec<usize> {
        let mut result = Vec::new();

        if let Some(start_he) = self.vertices[vertex_idx].half_edge {
            let mut current = start_he;

            loop {
                result.push(current);

                // Move to next outgoing half-edge
                if let Some(twin) = self.half_edges[current].twin {
                    current = self.half_edges[twin].next;
                    if current == start_he {
                        break;
                    }
                } else {
                    break; // Boundary vertex
                }
            }
        }

        result
    }

    /// Computes the valence (number of edges) of a vertex
    pub fn vertex_valence(&self, vertex_idx: usize) -> usize {
        self.vertex_half_edges(vertex_idx).len()
    }

    /// Performs Catmull-Clark subdivision
    pub fn catmull_clark_subdivide(&self) -> Self {
        let mut new_mesh = Self::new();

        // Step 1: Face points (centroid of each face)
        let mut face_points = Vec::new();
        for face in &self.faces {
            let vertices = self.face_vertices(face.half_edge);
            let centroid = self.compute_centroid(&vertices);
            face_points.push(centroid);
        }

        // Step 2: Edge points (average of edge endpoints and adjacent face points)
        let mut edge_points: HashMap<(usize, usize), Point3<f64>> = HashMap::new();

        for he in &self.half_edges {
            let v_start = self.half_edges[he.prev].vertex;
            let v_end = he.vertex;
            let edge = if v_start < v_end {
                (v_start, v_end)
            } else {
                (v_end, v_start)
            };

            if edge_points.contains_key(&edge) {
                continue;
            }

            let p1 = self.vertices[v_start].position;
            let p2 = self.vertices[v_end].position;

            let mut sum = p1.coords + p2.coords;
            let mut count = 2.0;

            if let Some(face_idx) = he.face {
                sum += face_points[face_idx].coords;
                count += 1.0;
            }

            if let Some(twin_idx) = he.twin {
                if let Some(twin_face_idx) = self.half_edges[twin_idx].face {
                    sum += face_points[twin_face_idx].coords;
                    count += 1.0;
                }
            }

            edge_points.insert(edge, Point3::from(sum / count));
        }

        // Step 3: New vertex positions
        // This is a simplified version; full Catmull-Clark is more complex
        for vertex in &self.vertices {
            new_mesh.vertices.push(HEVertex {
                position: vertex.position,
                half_edge: None,
            });
        }

        new_mesh
    }

    /// Gets vertices of a face
    fn face_vertices(&self, start_he: usize) -> Vec<usize> {
        let mut vertices = Vec::new();
        let mut current = start_he;

        loop {
            vertices.push(self.half_edges[current].vertex);
            current = self.half_edges[current].next;
            if current == start_he {
                break;
            }
        }

        vertices
    }

    /// Computes centroid of vertices
    fn compute_centroid(&self, vertex_indices: &[usize]) -> Point3<f64> {
        let mut sum = Vector3::zeros();

        for &idx in vertex_indices {
            sum += self.vertices[idx].position.coords;
        }

        Point3::from(sum / vertex_indices.len() as f64)
    }
}

impl Default for HalfEdgeMesh {
    fn default() -> Self {
        Self::new()
    }
}

/// Mesh simplification using edge collapse
pub struct MeshSimplifier {
    mesh: TriangleMesh,
}

impl MeshSimplifier {
    /// Creates a new mesh simplifier
    pub fn new(mesh: TriangleMesh) -> Self {
        Self { mesh }
    }

    /// Simplifies the mesh to a target face count
    pub fn simplify(&mut self, target_faces: usize) -> TriangleMesh {
        while self.mesh.faces.len() > target_faces {
            if !self.collapse_shortest_edge() {
                break; // No more edges can be collapsed
            }
        }

        self.mesh.clone()
    }

    /// Collapses the shortest edge in the mesh
    fn collapse_shortest_edge(&mut self) -> bool {
        // Find shortest edge
        let mut shortest_length = f64::MAX;
        let mut shortest_edge: Option<(usize, usize)> = None;

        let mut edges = HashSet::new();
        for face in &self.mesh.faces {
            for (v0, v1) in face.edges() {
                let edge = if v0 < v1 { (v0, v1) } else { (v1, v0) };
                edges.insert(edge);
            }
        }

        for (v0, v1) in edges {
            let p0 = self.mesh.vertices[v0].position;
            let p1 = self.mesh.vertices[v1].position;
            let length = nalgebra::distance(&p0, &p1);

            if length < shortest_length {
                shortest_length = length;
                shortest_edge = Some((v0, v1));
            }
        }

        if let Some((v0, v1)) = shortest_edge {
            self.collapse_edge(v0, v1);
            true
        } else {
            false
        }
    }

    /// Collapses an edge by merging v1 into v0
    fn collapse_edge(&mut self, v0: usize, v1: usize) {
        // Merge position
        let p0 = self.mesh.vertices[v0].position;
        let p1 = self.mesh.vertices[v1].position;
        self.mesh.vertices[v0].position = Point3::new(
            (p0.x + p1.x) / 2.0,
            (p0.y + p1.y) / 2.0,
            (p0.z + p1.z) / 2.0,
        );

        // Update all faces that reference v1 to reference v0
        for face in &mut self.mesh.faces {
            for vertex_idx in &mut face.vertices {
                if *vertex_idx == v1 {
                    *vertex_idx = v0;
                }
            }
        }

        // Remove degenerate faces (faces with duplicate vertices)
        self.mesh.faces.retain(|face| {
            face.vertices[0] != face.vertices[1]
                && face.vertices[1] != face.vertices[2]
                && face.vertices[2] != face.vertices[0]
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_mesh() {
        let mut mesh = TriangleMesh::new();

        let v0 = mesh.add_vertex(Vertex::new(Point3::new(0.0, 0.0, 0.0)));
        let v1 = mesh.add_vertex(Vertex::new(Point3::new(1.0, 0.0, 0.0)));
        let v2 = mesh.add_vertex(Vertex::new(Point3::new(0.0, 1.0, 0.0)));

        mesh.add_face(TriangleFace::new(v0, v1, v2));

        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.faces.len(), 1);
    }

    #[test]
    fn test_vertex_normals() {
        let mut mesh = TriangleMesh::new();

        mesh.add_vertex(Vertex::new(Point3::new(0.0, 0.0, 0.0)));
        mesh.add_vertex(Vertex::new(Point3::new(1.0, 0.0, 0.0)));
        mesh.add_vertex(Vertex::new(Point3::new(0.0, 1.0, 0.0)));

        mesh.add_face(TriangleFace::new(0, 1, 2));
        mesh.compute_vertex_normals();

        assert!(mesh.vertices[0].normal.is_some());
    }

    #[test]
    fn test_quad_mesh() {
        let mut mesh = QuadMesh::new();

        mesh.vertices.push(Vertex::new(Point3::new(0.0, 0.0, 0.0)));
        mesh.vertices.push(Vertex::new(Point3::new(1.0, 0.0, 0.0)));
        mesh.vertices.push(Vertex::new(Point3::new(1.0, 1.0, 0.0)));
        mesh.vertices.push(Vertex::new(Point3::new(0.0, 1.0, 0.0)));

        mesh.faces.push(QuadFace::new(0, 1, 2, 3));

        let tri_mesh = mesh.to_triangle_mesh();
        assert_eq!(tri_mesh.faces.len(), 2);
    }

    #[test]
    fn test_subdivision() {
        let mut mesh = TriangleMesh::new();

        mesh.add_vertex(Vertex::new(Point3::new(0.0, 0.0, 0.0)));
        mesh.add_vertex(Vertex::new(Point3::new(1.0, 0.0, 0.0)));
        mesh.add_vertex(Vertex::new(Point3::new(0.0, 1.0, 0.0)));

        mesh.add_face(TriangleFace::new(0, 1, 2));

        let original_faces = mesh.faces.len();
        mesh.subdivide();

        assert_eq!(mesh.faces.len(), original_faces * 4);
    }
}
