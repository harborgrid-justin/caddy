//! 2D Polygon geometry for CAD operations
//!
//! Provides polygons with hole support, area/centroid calculation,
//! point-in-polygon tests, convex hull algorithm, and offsetting.

use crate::core::*;
use crate::geometry::line::LineSegment2D;
use crate::geometry::point::Point2D;
use nalgebra::Point2 as NPoint2;
use serde::{Deserialize, Serialize};

/// 2D polygon with optional holes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Polygon2D {
    /// Outer boundary vertices (counterclockwise for positive area)
    pub vertices: Vec<Point2D>,
    /// Inner holes (clockwise for negative area)
    pub holes: Vec<Vec<Point2D>>,
}

impl Polygon2D {
    /// Create a new polygon from vertices
    pub fn new(vertices: Vec<Point2D>) -> Self {
        Self {
            vertices,
            holes: Vec::new(),
        }
    }

    /// Create a polygon with holes
    pub fn with_holes(vertices: Vec<Point2D>, holes: Vec<Vec<Point2D>>) -> Self {
        Self { vertices, holes }
    }

    /// Get the number of vertices in the outer boundary
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Get all edges of the outer boundary
    pub fn edges(&self) -> Vec<LineSegment2D> {
        if self.vertices.len() < 2 {
            return Vec::new();
        }

        let mut edges = Vec::new();
        for i in 0..self.vertices.len() {
            let j = (i + 1) % self.vertices.len();
            edges.push(LineSegment2D::new(self.vertices[i], self.vertices[j]));
        }
        edges
    }

    /// Calculate the signed area using the shoelace formula
    /// Positive for CCW, negative for CW
    pub fn signed_area(&self) -> f64 {
        if self.vertices.len() < 3 {
            return 0.0;
        }

        let mut area = 0.0;
        for i in 0..self.vertices.len() {
            let j = (i + 1) % self.vertices.len();
            area += self.vertices[i].x * self.vertices[j].y;
            area -= self.vertices[j].x * self.vertices[i].y;
        }

        area / 2.0
    }

    /// Calculate the area (absolute value)
    pub fn area(&self) -> f64 {
        let mut total_area = self.signed_area().abs();

        // Subtract hole areas
        for hole in &self.holes {
            let hole_polygon = Polygon2D::new(hole.clone());
            total_area -= hole_polygon.signed_area().abs();
        }

        total_area
    }

    /// Calculate the perimeter
    pub fn perimeter(&self) -> f64 {
        self.edges().iter().map(|e| e.length()).sum()
    }

    /// Calculate the centroid
    pub fn centroid(&self) -> Option<Point2D> {
        if self.vertices.len() < 3 {
            return None;
        }

        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut area = 0.0;

        for i in 0..self.vertices.len() {
            let j = (i + 1) % self.vertices.len();
            let cross = self.vertices[i].x * self.vertices[j].y
                - self.vertices[j].x * self.vertices[i].y;

            cx += (self.vertices[i].x + self.vertices[j].x) * cross;
            cy += (self.vertices[i].y + self.vertices[j].y) * cross;
            area += cross;
        }

        area /= 2.0;

        if area.abs() < EPSILON {
            return None;
        }

        cx /= 6.0 * area;
        cy /= 6.0 * area;

        Some(Point2D::new(cx, cy))
    }

    /// Check if the polygon is convex
    pub fn is_convex(&self) -> bool {
        if self.vertices.len() < 3 {
            return false;
        }

        let mut sign = 0i8;

        for i in 0..self.vertices.len() {
            let p1 = self.vertices[i];
            let p2 = self.vertices[(i + 1) % self.vertices.len()];
            let p3 = self.vertices[(i + 2) % self.vertices.len()];

            let v1 = Vector2::new(p2.x - p1.x, p2.y - p1.y);
            let v2 = Vector2::new(p3.x - p2.x, p3.y - p2.y);
            let cross = v1.x * v2.y - v1.y * v2.x;

            if cross.abs() > EPSILON {
                let current_sign = if cross > 0.0 { 1 } else { -1 };
                if sign == 0 {
                    sign = current_sign;
                } else if sign != current_sign {
                    return false;
                }
            }
        }

        true
    }

    /// Check if the polygon vertices are ordered counterclockwise
    pub fn is_ccw(&self) -> bool {
        self.signed_area() > 0.0
    }

    /// Reverse the vertex order
    pub fn reverse(&mut self) {
        self.vertices.reverse();
    }

    /// Point-in-polygon test using ray casting algorithm
    pub fn contains_point(&self, point: &Point2D) -> bool {
        if !self.contains_point_boundary(point) {
            return false;
        }

        // Check if point is in any hole
        for hole in &self.holes {
            let hole_polygon = Polygon2D::new(hole.clone());
            if hole_polygon.contains_point_boundary(point) {
                return false;
            }
        }

        true
    }

    /// Point-in-polygon test for boundary only (ignoring holes)
    fn contains_point_boundary(&self, point: &Point2D) -> bool {
        if self.vertices.len() < 3 {
            return false;
        }

        let mut inside = false;
        let n = self.vertices.len();

        for i in 0..n {
            let j = (i + 1) % n;
            let vi = &self.vertices[i];
            let vj = &self.vertices[j];

            if ((vi.y > point.y) != (vj.y > point.y))
                && (point.x < (vj.x - vi.x) * (point.y - vi.y) / (vj.y - vi.y) + vi.x)
            {
                inside = !inside;
            }
        }

        inside
    }

    /// Get the bounding box
    pub fn bounding_box(&self) -> Option<BoundingBox2> {
        if self.vertices.is_empty() {
            return None;
        }

        let points: Vec<NPoint2<f64>> = self
            .vertices
            .iter()
            .map(|p| NPoint2::new(p.x, p.y))
            .collect();

        BoundingBox2::from_points(&points)
    }

    /// Compute convex hull using Graham scan algorithm
    pub fn convex_hull(&self) -> Polygon2D {
        graham_scan(&self.vertices)
    }

    /// Offset the polygon (positive = outward, negative = inward)
    /// This is a simplified implementation - production CAD would use more sophisticated algorithms
    pub fn offset(&self, distance: f64) -> Polygon2D {
        if self.vertices.len() < 3 {
            return self.clone();
        }

        let mut offset_vertices = Vec::new();

        for i in 0..self.vertices.len() {
            let prev_idx = if i == 0 {
                self.vertices.len() - 1
            } else {
                i - 1
            };
            let next_idx = (i + 1) % self.vertices.len();

            let p_prev = self.vertices[prev_idx];
            let p_curr = self.vertices[i];
            let p_next = self.vertices[next_idx];

            // Get edge vectors
            let v1 = Vector2::new(p_curr.x - p_prev.x, p_curr.y - p_prev.y).normalize();
            let v2 = Vector2::new(p_next.x - p_curr.x, p_next.y - p_curr.y).normalize();

            // Get perpendicular vectors (pointing outward for CCW polygon)
            let n1 = Vector2::new(-v1.y, v1.x);
            let n2 = Vector2::new(-v2.y, v2.x);

            // Compute bisector
            let bisector = (n1 + n2).normalize();

            // Compute offset distance along bisector
            let sin_half_angle = n1.x * bisector.x + n1.y * bisector.y;
            let offset_dist = if sin_half_angle.abs() > EPSILON {
                distance / sin_half_angle
            } else {
                distance
            };

            // Offset vertex
            let offset_point = Point2D::new(
                p_curr.x + bisector.x * offset_dist,
                p_curr.y + bisector.y * offset_dist,
            );

            offset_vertices.push(offset_point);
        }

        Polygon2D::new(offset_vertices)
    }

    /// Triangulate the polygon using ear clipping algorithm
    pub fn triangulate(&self) -> Vec<[Point2D; 3]> {
        ear_clipping(&self.vertices)
    }

    /// Check if the polygon is simple (non-self-intersecting)
    pub fn is_simple(&self) -> bool {
        let edges = self.edges();

        for i in 0..edges.len() {
            for j in (i + 2)..edges.len() {
                // Skip adjacent edges
                if i == 0 && j == edges.len() - 1 {
                    continue;
                }

                if edges[i].intersect(&edges[j]).is_some() {
                    return false;
                }
            }
        }

        true
    }

    /// Add a hole to the polygon
    pub fn add_hole(&mut self, hole: Vec<Point2D>) {
        self.holes.push(hole);
    }

    /// Remove all holes
    pub fn clear_holes(&mut self) {
        self.holes.clear();
    }

    /// Create a rectangle
    pub fn rectangle(min: Point2D, max: Point2D) -> Polygon2D {
        Polygon2D::new(vec![
            Point2D::new(min.x, min.y),
            Point2D::new(max.x, min.y),
            Point2D::new(max.x, max.y),
            Point2D::new(min.x, max.y),
        ])
    }

    /// Create a regular polygon
    pub fn regular(center: Point2D, radius: f64, sides: usize) -> Polygon2D {
        use std::f64::consts::PI;

        let mut vertices = Vec::new();
        for i in 0..sides {
            let angle = 2.0 * PI * (i as f64) / (sides as f64);
            vertices.push(Point2D::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            ));
        }

        Polygon2D::new(vertices)
    }

    /// Create a circle approximation
    pub fn circle(center: Point2D, radius: f64, segments: usize) -> Polygon2D {
        Polygon2D::regular(center, radius, segments)
    }
}

/// Graham scan algorithm for convex hull
fn graham_scan(points: &[Point2D]) -> Polygon2D {
    if points.len() < 3 {
        return Polygon2D::new(points.to_vec());
    }

    // Find the point with the lowest y-coordinate (and leftmost if tie)
    let mut start_idx = 0;
    for i in 1..points.len() {
        if points[i].y < points[start_idx].y
            || (points[i].y == points[start_idx].y && points[i].x < points[start_idx].x)
        {
            start_idx = i;
        }
    }

    let start_point = points[start_idx];

    // Sort points by polar angle with respect to start point
    let mut sorted_points: Vec<Point2D> = points
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != start_idx)
        .map(|(_, p)| *p)
        .collect();

    sorted_points.sort_by(|a, b| {
        let angle_a = (a.y - start_point.y).atan2(a.x - start_point.x);
        let angle_b = (b.y - start_point.y).atan2(b.x - start_point.x);
        angle_a.partial_cmp(&angle_b).unwrap()
    });

    // Build convex hull
    let mut hull = vec![start_point];
    if !sorted_points.is_empty() {
        hull.push(sorted_points[0]);
    }

    for point in sorted_points.iter().skip(1) {
        while hull.len() >= 2 {
            let p1 = hull[hull.len() - 2];
            let p2 = hull[hull.len() - 1];

            let v1 = Vector2::new(p2.x - p1.x, p2.y - p1.y);
            let v2 = Vector2::new(point.x - p2.x, point.y - p2.y);
            let cross = v1.x * v2.y - v1.y * v2.x;

            if cross <= 0.0 {
                hull.pop();
            } else {
                break;
            }
        }

        hull.push(*point);
    }

    Polygon2D::new(hull)
}

/// Ear clipping triangulation algorithm
fn ear_clipping(vertices: &[Point2D]) -> Vec<[Point2D; 3]> {
    if vertices.len() < 3 {
        return Vec::new();
    }

    let mut triangles = Vec::new();
    let mut remaining: Vec<usize> = (0..vertices.len()).collect();

    while remaining.len() > 3 {
        let mut ear_found = false;

        for i in 0..remaining.len() {
            let prev_idx = if i == 0 {
                remaining.len() - 1
            } else {
                i - 1
            };
            let next_idx = (i + 1) % remaining.len();

            let p_prev = vertices[remaining[prev_idx]];
            let p_curr = vertices[remaining[i]];
            let p_next = vertices[remaining[next_idx]];

            // Check if this is an ear
            if is_ear(p_prev, p_curr, p_next, vertices, &remaining) {
                triangles.push([p_prev, p_curr, p_next]);
                remaining.remove(i);
                ear_found = true;
                break;
            }
        }

        if !ear_found {
            // Couldn't find an ear - polygon might be invalid
            break;
        }
    }

    // Add the last triangle
    if remaining.len() == 3 {
        triangles.push([
            vertices[remaining[0]],
            vertices[remaining[1]],
            vertices[remaining[2]],
        ]);
    }

    triangles
}

/// Check if three consecutive vertices form an ear
fn is_ear(p1: Point2D, p2: Point2D, p3: Point2D, vertices: &[Point2D], remaining: &[usize]) -> bool {
    // Check if the triangle is oriented correctly (CCW)
    let v1 = Vector2::new(p2.x - p1.x, p2.y - p1.y);
    let v2 = Vector2::new(p3.x - p2.x, p3.y - p2.y);
    let cross = v1.x * v2.y - v1.y * v2.x;

    if cross <= 0.0 {
        return false; // Not a convex vertex
    }

    // Check if any other vertex is inside the triangle
    for &idx in remaining {
        let p = vertices[idx];
        if p.approx_eq(&p1) || p.approx_eq(&p2) || p.approx_eq(&p3) {
            continue;
        }

        if point_in_triangle(&p, &p1, &p2, &p3) {
            return false;
        }
    }

    true
}

/// Check if a point is inside a triangle
fn point_in_triangle(p: &Point2D, a: &Point2D, b: &Point2D, c: &Point2D) -> bool {
    let v0 = Vector2::new(c.x - a.x, c.y - a.y);
    let v1 = Vector2::new(b.x - a.x, b.y - a.y);
    let v2 = Vector2::new(p.x - a.x, p.y - a.y);

    let dot00 = v0.dot(&v0);
    let dot01 = v0.dot(&v1);
    let dot02 = v0.dot(&v2);
    let dot11 = v1.dot(&v1);
    let dot12 = v1.dot(&v2);

    let inv_denom = 1.0 / (dot00 * dot11 - dot01 * dot01);
    let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
    let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;

    u >= 0.0 && v >= 0.0 && (u + v) < 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polygon_area() {
        let square = Polygon2D::new(vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 0.0),
            Point2D::new(1.0, 1.0),
            Point2D::new(0.0, 1.0),
        ]);

        assert!((square.area() - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_polygon_centroid() {
        let square = Polygon2D::new(vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(2.0, 0.0),
            Point2D::new(2.0, 2.0),
            Point2D::new(0.0, 2.0),
        ]);

        let centroid = square.centroid().unwrap();
        assert!(centroid.approx_eq(&Point2D::new(1.0, 1.0)));
    }

    #[test]
    fn test_polygon_contains_point() {
        let square = Polygon2D::new(vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 0.0),
            Point2D::new(1.0, 1.0),
            Point2D::new(0.0, 1.0),
        ]);

        assert!(square.contains_point(&Point2D::new(0.5, 0.5)));
        assert!(!square.contains_point(&Point2D::new(1.5, 0.5)));
    }

    #[test]
    fn test_polygon_is_convex() {
        let square = Polygon2D::new(vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 0.0),
            Point2D::new(1.0, 1.0),
            Point2D::new(0.0, 1.0),
        ]);

        assert!(square.is_convex());

        let concave = Polygon2D::new(vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(2.0, 0.0),
            Point2D::new(2.0, 2.0),
            Point2D::new(1.0, 1.0),
            Point2D::new(0.0, 2.0),
        ]);

        assert!(!concave.is_convex());
    }

    #[test]
    fn test_convex_hull() {
        let points = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 1.0),
            Point2D::new(2.0, 0.0),
            Point2D::new(1.0, 0.5), // Interior point
            Point2D::new(0.5, 0.5), // Interior point
        ];

        let polygon = Polygon2D::new(points);
        let hull = polygon.convex_hull();

        // Hull should have 3 vertices (the triangle)
        assert_eq!(hull.vertices.len(), 3);
    }

    #[test]
    fn test_regular_polygon() {
        let hexagon = Polygon2D::regular(Point2D::new(0.0, 0.0), 1.0, 6);
        assert_eq!(hexagon.vertices.len(), 6);
    }

    #[test]
    fn test_rectangle() {
        let rect = Polygon2D::rectangle(Point2D::new(0.0, 0.0), Point2D::new(2.0, 3.0));
        assert_eq!(rect.vertices.len(), 4);
        assert!((rect.area() - 6.0).abs() < EPSILON);
    }

    #[test]
    fn test_triangulation() {
        let square = Polygon2D::new(vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 0.0),
            Point2D::new(1.0, 1.0),
            Point2D::new(0.0, 1.0),
        ]);

        let triangles = square.triangulate();
        assert_eq!(triangles.len(), 2);
    }
}
