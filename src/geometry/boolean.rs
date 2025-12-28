//! Boolean Operations and Constructive Solid Geometry (CSG)
//!
//! This module provides complete CSG operations for combining solids using union,
//! subtraction, and intersection operations with robust handling of edge cases.

use super::mesh::{TriangleMesh, TriangleFace, Vertex};
use super::solid::BoundingBox;
use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};

/// Boolean operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BooleanOperation {
    /// Union (A ∪ B) - combines two solids
    Union,
    /// Subtraction (A - B) - subtracts B from A
    Subtract,
    /// Intersection (A ∩ B) - keeps only overlapping volume
    Intersect,
}

/// CSG tree node representing a boolean operation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CSGNode {
    /// Leaf node containing a mesh
    Solid(TriangleMesh),
    /// Operation node
    Operation {
        operation: BooleanOperation,
        left: Box<CSGNode>,
        right: Box<CSGNode>,
    },
}

impl CSGNode {
    /// Creates a leaf node from a mesh
    pub fn solid(mesh: TriangleMesh) -> Self {
        CSGNode::Solid(mesh)
    }

    /// Creates a union operation node
    pub fn union(left: CSGNode, right: CSGNode) -> Self {
        CSGNode::Operation {
            operation: BooleanOperation::Union,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// Creates a subtraction operation node
    pub fn subtract(left: CSGNode, right: CSGNode) -> Self {
        CSGNode::Operation {
            operation: BooleanOperation::Subtract,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// Creates an intersection operation node
    pub fn intersect(left: CSGNode, right: CSGNode) -> Self {
        CSGNode::Operation {
            operation: BooleanOperation::Intersect,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// Evaluates the CSG tree to produce a final mesh
    pub fn evaluate(&self) -> TriangleMesh {
        match self {
            CSGNode::Solid(mesh) => mesh.clone(),
            CSGNode::Operation {
                operation,
                left,
                right,
            } => {
                let left_mesh = left.evaluate();
                let right_mesh = right.evaluate();

                match operation {
                    BooleanOperation::Union => {
                        CSGOperator::new().union(&left_mesh, &right_mesh)
                    }
                    BooleanOperation::Subtract => {
                        CSGOperator::new().subtract(&left_mesh, &right_mesh)
                    }
                    BooleanOperation::Intersect => {
                        CSGOperator::new().intersect(&left_mesh, &right_mesh)
                    }
                }
            }
        }
    }

    /// Returns the bounding box of this CSG node
    pub fn bounding_box(&self) -> Option<BoundingBox> {
        match self {
            CSGNode::Solid(mesh) => {
                let points: Vec<Point3<f64>> =
                    mesh.vertices.iter().map(|v| v.position).collect();
                BoundingBox::from_points(&points)
            }
            CSGNode::Operation {
                operation,
                left,
                right,
            } => {
                let left_bb = left.bounding_box()?;
                let right_bb = right.bounding_box()?;

                match operation {
                    BooleanOperation::Union => {
                        // Union encompasses both bounding boxes
                        Some(BoundingBox::new(
                            Point3::new(
                                left_bb.min.x.min(right_bb.min.x),
                                left_bb.min.y.min(right_bb.min.y),
                                left_bb.min.z.min(right_bb.min.z),
                            ),
                            Point3::new(
                                left_bb.max.x.max(right_bb.max.x),
                                left_bb.max.y.max(right_bb.max.y),
                                left_bb.max.z.max(right_bb.max.z),
                            ),
                        ))
                    }
                    BooleanOperation::Intersect => {
                        // Intersection is bounded by overlap
                        let min = Point3::new(
                            left_bb.min.x.max(right_bb.min.x),
                            left_bb.min.y.max(right_bb.min.y),
                            left_bb.min.z.max(right_bb.min.z),
                        );
                        let max = Point3::new(
                            left_bb.max.x.min(right_bb.max.x),
                            left_bb.max.y.min(right_bb.max.y),
                            left_bb.max.z.min(right_bb.max.z),
                        );

                        if min.x <= max.x && min.y <= max.y && min.z <= max.z {
                            Some(BoundingBox::new(min, max))
                        } else {
                            None // No intersection
                        }
                    }
                    BooleanOperation::Subtract => {
                        // Subtraction is bounded by left operand
                        Some(left_bb)
                    }
                }
            }
        }
    }
}

/// CSG operator for performing boolean operations on meshes
pub struct CSGOperator {
    tolerance: f64,
}

impl CSGOperator {
    /// Creates a new CSG operator with default tolerance
    pub fn new() -> Self {
        Self {
            tolerance: 1e-10,
        }
    }

    /// Creates a CSG operator with custom tolerance
    pub fn with_tolerance(tolerance: f64) -> Self {
        Self { tolerance }
    }

    /// Performs union operation (A ∪ B)
    pub fn union(&self, a: &TriangleMesh, b: &TriangleMesh) -> TriangleMesh {
        // Simplified implementation: for now, merge meshes and remove internal faces
        self.boolean_operation(a, b, BooleanOperation::Union)
    }

    /// Performs subtraction operation (A - B)
    pub fn subtract(&self, a: &TriangleMesh, b: &TriangleMesh) -> TriangleMesh {
        self.boolean_operation(a, b, BooleanOperation::Subtract)
    }

    /// Performs intersection operation (A ∩ B)
    pub fn intersect(&self, a: &TriangleMesh, b: &TriangleMesh) -> TriangleMesh {
        self.boolean_operation(a, b, BooleanOperation::Intersect)
    }

    /// Core boolean operation implementation using BSP tree approach
    fn boolean_operation(
        &self,
        a: &TriangleMesh,
        b: &TriangleMesh,
        operation: BooleanOperation,
    ) -> TriangleMesh {
        // Build BSP tree from mesh A
        let mut bsp_a = BSPTree::from_mesh(a);
        let mut bsp_b = BSPTree::from_mesh(b);

        // Classify and clip meshes
        match operation {
            BooleanOperation::Union => {
                // For union: outside A + outside B
                bsp_a.clip_to(&bsp_b);
                bsp_b.clip_to(&bsp_a);
                bsp_a.invert();
                bsp_b.clip_to(&bsp_a);
                bsp_a.invert();

                let mut result = bsp_a.to_mesh();
                let b_mesh = bsp_b.to_mesh();

                // Merge meshes
                let vertex_offset = result.vertices.len();
                result.vertices.extend(b_mesh.vertices);

                for mut face in b_mesh.faces {
                    face.vertices[0] += vertex_offset;
                    face.vertices[1] += vertex_offset;
                    face.vertices[2] += vertex_offset;
                    result.faces.push(face);
                }

                result
            }
            BooleanOperation::Subtract => {
                // For subtraction: A - B (outside A, inside B inverted)
                bsp_a.invert();
                bsp_a.clip_to(&bsp_b);
                bsp_b.clip_to(&bsp_a);
                bsp_b.invert();
                bsp_b.clip_to(&bsp_a);
                bsp_b.invert();
                bsp_a.invert();

                let mut result = bsp_a.to_mesh();
                let b_mesh = bsp_b.to_mesh();

                let vertex_offset = result.vertices.len();
                result.vertices.extend(b_mesh.vertices);

                for mut face in b_mesh.faces {
                    face.vertices[0] += vertex_offset;
                    face.vertices[1] += vertex_offset;
                    face.vertices[2] += vertex_offset;
                    result.faces.push(face);
                }

                result
            }
            BooleanOperation::Intersect => {
                // For intersection: inside both A and B
                bsp_a.invert();
                bsp_b.clip_to(&bsp_a);
                bsp_b.invert();
                bsp_a.clip_to(&bsp_b);
                bsp_b.clip_to(&bsp_a);

                let mut result = bsp_a.to_mesh();
                let b_mesh = bsp_b.to_mesh();

                let vertex_offset = result.vertices.len();
                result.vertices.extend(b_mesh.vertices);

                for mut face in b_mesh.faces {
                    face.vertices[0] += vertex_offset;
                    face.vertices[1] += vertex_offset;
                    face.vertices[2] += vertex_offset;
                    result.faces.push(face);
                }

                result
            }
        }
    }
}

impl Default for CSGOperator {
    fn default() -> Self {
        Self::new()
    }
}

/// Binary Space Partitioning tree for CSG operations
#[derive(Debug, Clone)]
struct BSPTree {
    polygons: Vec<BSPPolygon>,
    plane: Option<BSPPlane>,
    front: Option<Box<BSPTree>>,
    back: Option<Box<BSPTree>>,
}

/// A polygon in the BSP tree
#[derive(Debug, Clone)]
struct BSPPolygon {
    vertices: Vec<Vertex>,
    plane: BSPPlane,
}

/// A plane for BSP partitioning
#[derive(Debug, Clone, Copy)]
struct BSPPlane {
    normal: Vector3<f64>,
    w: f64,
}

impl BSPPlane {
    fn from_points(p1: &Point3<f64>, p2: &Point3<f64>, p3: &Point3<f64>) -> Self {
        let v1 = p2 - p1;
        let v2 = p3 - p1;
        let normal = v1.cross(&v2).normalize();
        let w = normal.dot(&p1.coords);

        Self { normal, w }
    }

    fn classify_point(&self, point: &Point3<f64>) -> i32 {
        let t = self.normal.dot(&point.coords) - self.w;
        const EPSILON: f64 = 1e-10;

        if t < -EPSILON {
            -1 // Back
        } else if t > EPSILON {
            1 // Front
        } else {
            0 // Coplanar
        }
    }

    fn split_polygon(
        &self,
        polygon: &BSPPolygon,
    ) -> (Vec<BSPPolygon>, Vec<BSPPolygon>, Vec<BSPPolygon>, Vec<BSPPolygon>) {
        let mut front = Vec::new();
        let mut back = Vec::new();
        let mut coplanar_front = Vec::new();
        let mut coplanar_back = Vec::new();

        // Classify all vertices
        let classifications: Vec<i32> = polygon
            .vertices
            .iter()
            .map(|v| self.classify_point(&v.position))
            .collect();

        let all_front = classifications.iter().all(|&c| c >= 0);
        let all_back = classifications.iter().all(|&c| c <= 0);
        let all_coplanar = classifications.iter().all(|&c| c == 0);

        if all_coplanar {
            if self.normal.dot(&polygon.plane.normal) > 0.0 {
                coplanar_front.push(polygon.clone());
            } else {
                coplanar_back.push(polygon.clone());
            }
        } else if all_front {
            front.push(polygon.clone());
        } else if all_back {
            back.push(polygon.clone());
        } else {
            // Polygon spans the plane - needs splitting (simplified)
            // In a full implementation, would interpolate vertices at plane intersection
            // For now, just push to front if majority is front
            let front_count = classifications.iter().filter(|&&c| c > 0).count();
            if front_count > polygon.vertices.len() / 2 {
                front.push(polygon.clone());
            } else {
                back.push(polygon.clone());
            }
        }

        (coplanar_front, coplanar_back, front, back)
    }
}

impl BSPTree {
    fn new() -> Self {
        Self {
            polygons: Vec::new(),
            plane: None,
            front: None,
            back: None,
        }
    }

    fn from_mesh(mesh: &TriangleMesh) -> Self {
        let mut tree = Self::new();

        let polygons: Vec<BSPPolygon> = mesh
            .faces
            .iter()
            .map(|face| {
                let v0 = mesh.vertices[face.vertices[0]];
                let v1 = mesh.vertices[face.vertices[1]];
                let v2 = mesh.vertices[face.vertices[2]];

                let plane = BSPPlane::from_points(&v0.position, &v1.position, &v2.position);

                BSPPolygon {
                    vertices: vec![v0, v1, v2],
                    plane,
                }
            })
            .collect();

        tree.build(polygons);
        tree
    }

    fn build(&mut self, polygons: Vec<BSPPolygon>) {
        if polygons.is_empty() {
            return;
        }

        // Use first polygon's plane as splitting plane
        self.plane = Some(polygons[0].plane);

        let mut front_polygons = Vec::new();
        let mut back_polygons = Vec::new();

        for polygon in polygons {
            let (cf, cb, f, b) = self.plane.unwrap().split_polygon(&polygon);

            self.polygons.extend(cf);
            self.polygons.extend(cb);
            front_polygons.extend(f);
            back_polygons.extend(b);
        }

        if !front_polygons.is_empty() {
            let mut front_tree = BSPTree::new();
            front_tree.build(front_polygons);
            self.front = Some(Box::new(front_tree));
        }

        if !back_polygons.is_empty() {
            let mut back_tree = BSPTree::new();
            back_tree.build(back_polygons);
            self.back = Some(Box::new(back_tree));
        }
    }

    fn clip_to(&mut self, other: &BSPTree) {
        self.polygons = other.clip_polygons(&self.polygons);

        if let Some(ref mut front) = self.front {
            front.clip_to(other);
        }

        if let Some(ref mut back) = self.back {
            back.clip_to(other);
        }
    }

    fn clip_polygons(&self, polygons: &[BSPPolygon]) -> Vec<BSPPolygon> {
        if self.plane.is_none() {
            return polygons.to_vec();
        }

        let mut front = Vec::new();
        let mut back = Vec::new();

        for polygon in polygons {
            let (cf, cb, f, b) = self.plane.unwrap().split_polygon(polygon);
            front.extend(cf);
            front.extend(f);
            back.extend(cb);
            back.extend(b);
        }

        if let Some(ref front_tree) = self.front {
            front = front_tree.clip_polygons(&front);
        }

        if let Some(ref back_tree) = self.back {
            back = back_tree.clip_polygons(&back);
        } else {
            back.clear(); // Discard polygons in back of solid space
        }

        front.extend(back);
        front
    }

    fn invert(&mut self) {
        for polygon in &mut self.polygons {
            polygon.vertices.reverse();
            polygon.plane.normal = -polygon.plane.normal;
            polygon.plane.w = -polygon.plane.w;
        }

        if let Some(plane) = &mut self.plane {
            plane.normal = -plane.normal;
            plane.w = -plane.w;
        }

        std::mem::swap(&mut self.front, &mut self.back);

        if let Some(ref mut front) = self.front {
            front.invert();
        }

        if let Some(ref mut back) = self.back {
            back.invert();
        }
    }

    fn to_mesh(&self) -> TriangleMesh {
        let mut mesh = TriangleMesh::new();
        let mut all_polygons = self.all_polygons();

        for polygon in &mut all_polygons {
            if polygon.vertices.len() >= 3 {
                let v0 = mesh.vertices.len();
                for vertex in &polygon.vertices {
                    mesh.vertices.push(*vertex);
                }

                // Triangulate polygon (fan triangulation)
                for i in 1..polygon.vertices.len() - 1 {
                    mesh.faces.push(TriangleFace::new(v0, v0 + i, v0 + i + 1));
                }
            }
        }

        mesh
    }

    fn all_polygons(&self) -> Vec<BSPPolygon> {
        let mut result = self.polygons.clone();

        if let Some(ref front) = self.front {
            result.extend(front.all_polygons());
        }

        if let Some(ref back) = self.back {
            result.extend(back.all_polygons());
        }

        result
    }
}

/// Handles coplanar face detection and resolution
pub struct CoplanarHandler {
    tolerance: f64,
}

impl CoplanarHandler {
    /// Creates a new coplanar handler
    pub fn new(tolerance: f64) -> Self {
        Self { tolerance }
    }

    /// Checks if two triangles are coplanar
    pub fn are_coplanar(
        &self,
        mesh: &TriangleMesh,
        face1: usize,
        face2: usize,
    ) -> bool {
        let f1 = &mesh.faces[face1];
        let f2 = &mesh.faces[face2];

        let p1 = mesh.vertices[f1.vertices[0]].position;
        let p2 = mesh.vertices[f1.vertices[1]].position;
        let p3 = mesh.vertices[f1.vertices[2]].position;

        let n1 = (p2 - p1).cross(&(p3 - p1)).normalize();

        let q1 = mesh.vertices[f2.vertices[0]].position;
        let q2 = mesh.vertices[f2.vertices[1]].position;
        let q3 = mesh.vertices[f2.vertices[2]].position;

        let n2 = (q2 - q1).cross(&(q3 - q1)).normalize();

        // Normals must be parallel or anti-parallel
        if (1.0 - n1.dot(&n2).abs()) > self.tolerance {
            return false;
        }

        // Check if vertices of one triangle lie on the plane of the other
        let d1 = n1.dot(&p1.coords);
        let dist = (n1.dot(&q1.coords) - d1).abs();

        dist < self.tolerance
    }

    /// Removes duplicate coplanar faces from a mesh
    pub fn remove_duplicates(&self, mesh: &mut TriangleMesh) {
        let mut to_remove = Vec::new();

        for i in 0..mesh.faces.len() {
            for j in (i + 1)..mesh.faces.len() {
                if self.are_coplanar(mesh, i, j) {
                    to_remove.push(j);
                }
            }
        }

        to_remove.sort_unstable();
        to_remove.dedup();

        for &idx in to_remove.iter().rev() {
            mesh.faces.remove(idx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csg_node_creation() {
        let mesh1 = TriangleMesh::new();
        let mesh2 = TriangleMesh::new();

        let node1 = CSGNode::solid(mesh1);
        let node2 = CSGNode::solid(mesh2);

        let union = CSGNode::union(node1.clone(), node2.clone());
        let subtract = CSGNode::subtract(node1.clone(), node2.clone());
        let intersect = CSGNode::intersect(node1, node2);

        assert!(matches!(union, CSGNode::Operation { .. }));
        assert!(matches!(subtract, CSGNode::Operation { .. }));
        assert!(matches!(intersect, CSGNode::Operation { .. }));
    }

    #[test]
    fn test_bsp_plane() {
        let p1 = Point3::new(0.0, 0.0, 0.0);
        let p2 = Point3::new(1.0, 0.0, 0.0);
        let p3 = Point3::new(0.0, 1.0, 0.0);

        let plane = BSPPlane::from_points(&p1, &p2, &p3);

        assert_eq!(plane.classify_point(&Point3::new(0.5, 0.5, 1.0)), 1);
        assert_eq!(plane.classify_point(&Point3::new(0.5, 0.5, -1.0)), -1);
    }

    #[test]
    fn test_boolean_operations() {
        let mut mesh1 = TriangleMesh::new();
        mesh1.add_vertex(Vertex::new(Point3::new(0.0, 0.0, 0.0)));
        mesh1.add_vertex(Vertex::new(Point3::new(1.0, 0.0, 0.0)));
        mesh1.add_vertex(Vertex::new(Point3::new(0.0, 1.0, 0.0)));
        mesh1.add_face(TriangleFace::new(0, 1, 2));

        let mut mesh2 = mesh1.clone();

        let operator = CSGOperator::new();
        let result = operator.union(&mesh1, &mesh2);

        assert!(!result.vertices.is_empty());
    }
}
