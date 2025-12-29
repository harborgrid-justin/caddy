//! CSG (Constructive Solid Geometry) Boolean Operations
//!
//! Implements union, intersection, and difference operations on meshes
//! using robust BSP-tree based algorithms with numerical stability.

use super::mesh::{HalfEdgeMesh, VertexHandle, FaceHandle, MeshError};
use crate::core::{Point3, Vector3, EPSILON};
use nalgebra::Point3 as NPoint3;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// BSP (Binary Space Partitioning) tree for CSG operations
#[derive(Debug, Clone)]
pub struct BSPTree {
    root: Option<Box<BSPNode>>,
}

#[derive(Debug, Clone)]
struct BSPNode {
    plane: Plane,
    front: Option<Box<BSPNode>>,
    back: Option<Box<BSPNode>>,
    polygons: Vec<Polygon>,
}

/// Plane in 3D space
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Plane {
    pub normal: Vector3,
    pub distance: f64,
}

impl Plane {
    /// Create a plane from a normal and a point
    pub fn from_normal_point(normal: Vector3, point: Point3) -> Self {
        let normalized = normal.normalize();
        let distance = normalized.dot(&point.coords);
        Self {
            normal: normalized,
            distance,
        }
    }

    /// Create a plane from three points
    pub fn from_points(p1: Point3, p2: Point3, p3: Point3) -> Option<Self> {
        let v1 = p2 - p1;
        let v2 = p3 - p1;
        let normal = v1.cross(&v2);

        if normal.norm() < EPSILON {
            return None;
        }

        Some(Self::from_normal_point(normal, p1))
    }

    /// Classify a point relative to the plane
    pub fn classify_point(&self, point: &Point3) -> PointClassification {
        let dist = self.normal.dot(&point.coords) - self.distance;

        if dist > EPSILON {
            PointClassification::Front
        } else if dist < -EPSILON {
            PointClassification::Back
        } else {
            PointClassification::OnPlane
        }
    }

    /// Flip the plane
    pub fn flip(&mut self) {
        self.normal = -self.normal;
        self.distance = -self.distance;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointClassification {
    Front,
    Back,
    OnPlane,
}

/// Polygon representation for CSG operations
#[derive(Debug, Clone)]
pub struct Polygon {
    pub vertices: Vec<Point3>,
    pub plane: Plane,
    pub original_face: Option<FaceHandle>,
}

impl Polygon {
    /// Create a polygon from vertices
    pub fn new(vertices: Vec<Point3>) -> Option<Self> {
        if vertices.len() < 3 {
            return None;
        }

        let plane = Plane::from_points(vertices[0], vertices[1], vertices[2])?;

        Some(Self {
            vertices,
            plane,
            original_face: None,
        })
    }

    /// Flip the polygon
    pub fn flip(&mut self) {
        self.vertices.reverse();
        self.plane.flip();
    }

    /// Split polygon by a plane
    pub fn split(&self, plane: &Plane) -> PolygonSplit {
        let mut front = Vec::new();
        let mut back = Vec::new();
        let mut front_verts = Vec::new();
        let mut back_verts = Vec::new();

        for i in 0..self.vertices.len() {
            let j = (i + 1) % self.vertices.len();
            let vi = &self.vertices[i];
            let vj = &self.vertices[j];

            let ti = plane.classify_point(vi);
            let tj = plane.classify_point(vj);

            if ti != PointClassification::Back {
                front_verts.push(*vi);
            }
            if ti != PointClassification::Front {
                back_verts.push(*vi);
            }

            if (ti == PointClassification::Front && tj == PointClassification::Back) ||
               (ti == PointClassification::Back && tj == PointClassification::Front) {
                // Edge crosses plane - compute intersection
                if let Some(intersection) = self.intersect_edge_plane(vi, vj, plane) {
                    front_verts.push(intersection);
                    back_verts.push(intersection);
                }
            }
        }

        if front_verts.len() >= 3 {
            if let Some(poly) = Polygon::new(front_verts) {
                front.push(poly);
            }
        }

        if back_verts.len() >= 3 {
            if let Some(poly) = Polygon::new(back_verts) {
                back.push(poly);
            }
        }

        PolygonSplit { front, back }
    }

    /// Intersect an edge with a plane
    fn intersect_edge_plane(&self, v1: &Point3, v2: &Point3, plane: &Plane) -> Option<Point3> {
        let d1 = plane.normal.dot(&v1.coords) - plane.distance;
        let d2 = plane.normal.dot(&v2.coords) - plane.distance;

        let denom = d1 - d2;
        if denom.abs() < EPSILON {
            return None;
        }

        let t = d1 / denom;
        if t < 0.0 || t > 1.0 {
            return None;
        }

        Some(Point3::from(v1.coords + t * (v2 - v1)))
    }
}

#[derive(Debug)]
pub struct PolygonSplit {
    pub front: Vec<Polygon>,
    pub back: Vec<Polygon>,
}

impl BSPTree {
    /// Create a new empty BSP tree
    pub fn new() -> Self {
        Self { root: None }
    }

    /// Build a BSP tree from polygons
    pub fn from_polygons(polygons: Vec<Polygon>) -> Self {
        let mut tree = Self::new();
        tree.root = BSPNode::build(polygons);
        tree
    }

    /// Clip polygons to this tree
    pub fn clip_polygons(&self, polygons: Vec<Polygon>) -> Vec<Polygon> {
        if let Some(ref root) = self.root {
            root.clip_polygons(polygons)
        } else {
            polygons
        }
    }

    /// Get all polygons from the tree
    pub fn all_polygons(&self) -> Vec<Polygon> {
        if let Some(ref root) = self.root {
            root.all_polygons()
        } else {
            Vec::new()
        }
    }

    /// Invert the tree (flip all polygons)
    pub fn invert(&mut self) {
        if let Some(ref mut root) = self.root {
            root.invert();
        }
    }

    /// Clip this tree to another tree
    pub fn clip_to(&mut self, other: &BSPTree) {
        if let Some(ref mut root) = self.root {
            root.clip_to(other);
        }
    }
}

impl Default for BSPTree {
    fn default() -> Self {
        Self::new()
    }
}

impl BSPNode {
    /// Build a BSP tree from polygons
    fn build(mut polygons: Vec<Polygon>) -> Option<Box<BSPNode>> {
        if polygons.is_empty() {
            return None;
        }

        // Choose first polygon's plane as splitting plane
        let plane = polygons[0].plane;
        let mut node = Box::new(BSPNode {
            plane,
            front: None,
            back: None,
            polygons: Vec::new(),
        });

        let mut front_polygons = Vec::new();
        let mut back_polygons = Vec::new();

        for polygon in polygons.drain(..) {
            node.split_polygon(polygon, &mut node.polygons, &mut front_polygons, &mut back_polygons);
        }

        node.front = Self::build(front_polygons);
        node.back = Self::build(back_polygons);

        Some(node)
    }

    /// Split a polygon by this node's plane
    fn split_polygon(
        &self,
        polygon: Polygon,
        coplanar: &mut Vec<Polygon>,
        front: &mut Vec<Polygon>,
        back: &mut Vec<Polygon>,
    ) {
        // Classify all vertices
        let mut classifications = Vec::new();
        let mut has_front = false;
        let mut has_back = false;

        for vertex in &polygon.vertices {
            let class = self.plane.classify_point(vertex);
            if class == PointClassification::Front {
                has_front = true;
            } else if class == PointClassification::Back {
                has_back = true;
            }
            classifications.push(class);
        }

        if !has_front && !has_back {
            // All vertices are on plane
            coplanar.push(polygon);
        } else if !has_back {
            // All vertices in front
            front.push(polygon);
        } else if !has_front {
            // All vertices in back
            back.push(polygon);
        } else {
            // Polygon spans plane - split it
            let split = polygon.split(&self.plane);
            front.extend(split.front);
            back.extend(split.back);
        }
    }

    /// Clip polygons to this node
    fn clip_polygons(&self, polygons: Vec<Polygon>) -> Vec<Polygon> {
        let mut result = Vec::new();

        for polygon in polygons {
            let mut coplanar = Vec::new();
            let mut front = Vec::new();
            let mut back = Vec::new();

            self.split_polygon(polygon, &mut coplanar, &mut front, &mut back);

            // Coplanar polygons go to front
            result.extend(coplanar);

            // Recursively clip front and back
            if let Some(ref front_node) = self.front {
                result.extend(front_node.clip_polygons(front));
            } else {
                result.extend(front);
            }

            if let Some(ref back_node) = self.back {
                result.extend(back_node.clip_polygons(back));
            }
            // Back polygons that don't have a back node are clipped away
        }

        result
    }

    /// Get all polygons from this subtree
    fn all_polygons(&self) -> Vec<Polygon> {
        let mut result = self.polygons.clone();

        if let Some(ref front) = self.front {
            result.extend(front.all_polygons());
        }

        if let Some(ref back) = self.back {
            result.extend(back.all_polygons());
        }

        result
    }

    /// Invert this subtree
    fn invert(&mut self) {
        for polygon in &mut self.polygons {
            polygon.flip();
        }

        self.plane.flip();
        std::mem::swap(&mut self.front, &mut self.back);

        if let Some(ref mut front) = self.front {
            front.invert();
        }

        if let Some(ref mut back) = self.back {
            back.invert();
        }
    }

    /// Clip this node to a tree
    fn clip_to(&mut self, tree: &BSPTree) {
        self.polygons = tree.clip_polygons(std::mem::take(&mut self.polygons));

        if let Some(ref mut front) = self.front {
            front.clip_to(tree);
        }

        if let Some(ref mut back) = self.back {
            back.clip_to(tree);
        }
    }
}

/// CSG Boolean operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BooleanOp {
    Union,
    Intersection,
    Difference,
}

/// Perform a CSG boolean operation on two meshes
pub fn boolean_operation(
    mesh_a: &HalfEdgeMesh,
    mesh_b: &HalfEdgeMesh,
    operation: BooleanOp,
) -> Result<HalfEdgeMesh, MeshError> {
    // Convert meshes to polygons
    let mut polygons_a = mesh_to_polygons(mesh_a)?;
    let mut polygons_b = mesh_to_polygons(mesh_b)?;

    // Build BSP trees
    let mut tree_a = BSPTree::from_polygons(polygons_a);
    let mut tree_b = BSPTree::from_polygons(polygons_b);

    // Perform operation
    match operation {
        BooleanOp::Union => {
            tree_a.clip_to(&tree_b);
            tree_b.clip_to(&tree_a);
            tree_b.invert();
            tree_b.clip_to(&tree_a);
            tree_b.invert();

            let mut result_polygons = tree_a.all_polygons();
            result_polygons.extend(tree_b.all_polygons());
            polygons_to_mesh(&result_polygons)
        }
        BooleanOp::Intersection => {
            tree_a.invert();
            tree_b.clip_to(&tree_a);
            tree_b.invert();
            tree_a.clip_to(&tree_b);
            tree_b.clip_to(&tree_a);
            tree_a.invert();

            let mut result_polygons = tree_a.all_polygons();
            result_polygons.extend(tree_b.all_polygons());
            polygons_to_mesh(&result_polygons)
        }
        BooleanOp::Difference => {
            tree_a.invert();
            tree_a.clip_to(&tree_b);
            tree_b.clip_to(&tree_a);
            tree_b.invert();
            tree_b.clip_to(&tree_a);
            tree_b.invert();
            tree_a.invert();

            let mut result_polygons = tree_a.all_polygons();
            result_polygons.extend(tree_b.all_polygons());
            polygons_to_mesh(&result_polygons)
        }
    }
}

/// Convert a mesh to polygons
fn mesh_to_polygons(mesh: &HalfEdgeMesh) -> Result<Vec<Polygon>, MeshError> {
    let mut polygons = Vec::new();

    for face_handle in mesh.face_handles() {
        let vertex_handles = mesh.face_vertices(face_handle)?;
        let mut vertices = Vec::new();

        for vh in vertex_handles {
            let vertex = mesh.get_vertex(vh)?;
            vertices.push(vertex.position);
        }

        if let Some(mut polygon) = Polygon::new(vertices) {
            polygon.original_face = Some(face_handle);
            polygons.push(polygon);
        }
    }

    Ok(polygons)
}

/// Convert polygons back to a mesh
fn polygons_to_mesh(polygons: &[Polygon]) -> Result<HalfEdgeMesh, MeshError> {
    let mut mesh = HalfEdgeMesh::new();
    let mut vertex_map: HashMap<VertexKey, VertexHandle> = HashMap::new();

    for polygon in polygons {
        let mut vertex_handles = Vec::new();

        for point in &polygon.vertices {
            let key = VertexKey::from_point(point);
            let vh = vertex_map.entry(key).or_insert_with(|| {
                mesh.add_vertex(*point)
            });
            vertex_handles.push(*vh);
        }

        mesh.add_face(&vertex_handles)?;
    }

    mesh.update_vertex_normals();

    Ok(mesh)
}

/// Key for vertex deduplication
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct VertexKey {
    x: i64,
    y: i64,
    z: i64,
}

impl VertexKey {
    fn from_point(point: &Point3) -> Self {
        const SCALE: f64 = 1_000_000.0;
        Self {
            x: (point.x * SCALE).round() as i64,
            y: (point.y * SCALE).round() as i64,
            z: (point.z * SCALE).round() as i64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plane_creation() {
        let p1 = Point3::new(0.0, 0.0, 0.0);
        let p2 = Point3::new(1.0, 0.0, 0.0);
        let p3 = Point3::new(0.0, 1.0, 0.0);

        let plane = Plane::from_points(p1, p2, p3).unwrap();

        // Normal should point in positive Z
        assert!((plane.normal.z - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_point_classification() {
        let plane = Plane {
            normal: Vector3::new(0.0, 0.0, 1.0),
            distance: 0.0,
        };

        let p_front = Point3::new(0.0, 0.0, 1.0);
        let p_back = Point3::new(0.0, 0.0, -1.0);
        let p_on = Point3::new(0.0, 0.0, 0.0);

        assert_eq!(plane.classify_point(&p_front), PointClassification::Front);
        assert_eq!(plane.classify_point(&p_back), PointClassification::Back);
        assert_eq!(plane.classify_point(&p_on), PointClassification::OnPlane);
    }

    #[test]
    fn test_polygon_creation() {
        let vertices = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ];

        let polygon = Polygon::new(vertices).unwrap();
        assert_eq!(polygon.vertices.len(), 3);
    }
}
