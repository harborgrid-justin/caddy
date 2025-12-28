//! 2D Line geometry for CAD operations
//!
//! Provides infinite lines, finite line segments, and polylines with
//! intersection algorithms, perpendicular/parallel tests, and offset operations.

use crate::core::*;
use crate::geometry::point::Point2D;
use nalgebra::Point2 as NPoint2;
use serde::{Deserialize, Serialize};

/// Infinite 2D line defined by a point and direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line2D {
    /// A point on the line
    pub point: Point2D,
    /// Direction vector (normalized)
    pub direction: Vector2,
}

impl serde::Serialize for Line2D {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Line2D", 2)?;
        state.serialize_field("point", &self.point)?;
        state.serialize_field("direction", &(self.direction.x, self.direction.y))?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for Line2D {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Point,
            Direction,
        }

        struct Line2DVisitor;

        impl<'de> serde::de::Visitor<'de> for Line2DVisitor {
            type Value = Line2D;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Line2D")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Line2D, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut point = None;
                let mut direction: Option<(f64, f64)> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Point => {
                            if point.is_some() {
                                return Err(serde::de::Error::duplicate_field("point"));
                            }
                            point = Some(map.next_value()?);
                        }
                        Field::Direction => {
                            if direction.is_some() {
                                return Err(serde::de::Error::duplicate_field("direction"));
                            }
                            direction = Some(map.next_value()?);
                        }
                    }
                }

                let point = point.ok_or_else(|| serde::de::Error::missing_field("point"))?;
                let direction = direction.ok_or_else(|| serde::de::Error::missing_field("direction"))?;

                Ok(Line2D {
                    point,
                    direction: Vector2::new(direction.0, direction.1),
                })
            }
        }

        const FIELDS: &[&str] = &["point", "direction"];
        deserializer.deserialize_struct("Line2D", FIELDS, Line2DVisitor)
    }
}

impl Line2D {
    /// Create a new line from a point and direction
    pub fn new(point: Point2D, direction: Vector2) -> Self {
        Self {
            point,
            direction: direction.normalize(),
        }
    }

    /// Create a line from two points
    pub fn from_points(p1: Point2D, p2: Point2D) -> Option<Self> {
        let direction = Vector2::new(p2.x - p1.x, p2.y - p1.y);
        if direction.norm() < EPSILON {
            None
        } else {
            Some(Self::new(p1, direction))
        }
    }

    /// Get a point on the line at parameter t
    pub fn point_at(&self, t: f64) -> Point2D {
        Point2D::new(
            self.point.x + self.direction.x * t,
            self.point.y + self.direction.y * t,
        )
    }

    /// Get the perpendicular distance from a point to this line
    pub fn distance_to_point(&self, point: &Point2D) -> f64 {
        let v = Vector2::new(point.x - self.point.x, point.y - self.point.y);
        let perp = Vector2::new(-self.direction.y, self.direction.x);
        v.dot(&perp).abs()
    }

    /// Project a point onto the line
    pub fn project_point(&self, point: &Point2D) -> Point2D {
        let v = Vector2::new(point.x - self.point.x, point.y - self.point.y);
        let t = v.dot(&self.direction);
        self.point_at(t)
    }

    /// Check if this line is parallel to another
    pub fn is_parallel(&self, other: &Line2D) -> bool {
        let cross = self.direction.x * other.direction.y - self.direction.y * other.direction.x;
        cross.abs() < EPSILON
    }

    /// Check if this line is perpendicular to another
    pub fn is_perpendicular(&self, other: &Line2D) -> bool {
        let dot = self.direction.dot(&other.direction);
        dot.abs() < EPSILON
    }

    /// Find intersection point with another line
    pub fn intersect(&self, other: &Line2D) -> Option<Point2D> {
        if self.is_parallel(other) {
            return None;
        }

        let dx = other.point.x - self.point.x;
        let dy = other.point.y - self.point.y;

        let det = self.direction.x * other.direction.y - self.direction.y * other.direction.x;
        let t = (dx * other.direction.y - dy * other.direction.x) / det;

        Some(self.point_at(t))
    }

    /// Get the angle of this line (in radians)
    pub fn angle(&self) -> f64 {
        self.direction.y.atan2(self.direction.x)
    }

    /// Get a perpendicular line through a given point
    pub fn perpendicular_through(&self, point: Point2D) -> Line2D {
        Line2D::new(point, Vector2::new(-self.direction.y, self.direction.x))
    }

    /// Get a parallel line through a given point
    pub fn parallel_through(&self, point: Point2D) -> Line2D {
        Line2D::new(point, self.direction)
    }

    /// Offset the line by a distance (positive = left, negative = right)
    pub fn offset(&self, distance: f64) -> Line2D {
        let perp = Vector2::new(-self.direction.y, self.direction.x);
        let offset_point = Point2D::new(
            self.point.x + perp.x * distance,
            self.point.y + perp.y * distance,
        );
        Line2D::new(offset_point, self.direction)
    }
}

/// Finite 2D line segment
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LineSegment2D {
    /// Start point
    pub start: Point2D,
    /// End point
    pub end: Point2D,
}

impl LineSegment2D {
    /// Create a new line segment
    pub fn new(start: Point2D, end: Point2D) -> Self {
        Self { start, end }
    }

    /// Get the length of the segment
    pub fn length(&self) -> f64 {
        self.start.distance_to(&self.end)
    }

    /// Get the squared length (faster, avoids sqrt)
    pub fn length_squared(&self) -> f64 {
        self.start.distance_squared_to(&self.end)
    }

    /// Get the direction vector (normalized)
    pub fn direction(&self) -> Vector2 {
        let v = Vector2::new(self.end.x - self.start.x, self.end.y - self.start.y);
        if v.norm() < EPSILON {
            Vector2::new(1.0, 0.0)
        } else {
            v.normalize()
        }
    }

    /// Get the direction vector (unnormalized)
    pub fn direction_unnormalized(&self) -> Vector2 {
        Vector2::new(self.end.x - self.start.x, self.end.y - self.start.y)
    }

    /// Get a point on the segment at parameter t [0, 1]
    pub fn point_at(&self, t: f64) -> Point2D {
        self.start.lerp(&self.end, t)
    }

    /// Get the midpoint of the segment
    pub fn midpoint(&self) -> Point2D {
        self.start.midpoint(&self.end)
    }

    /// Convert to an infinite line
    pub fn to_line(&self) -> Option<Line2D> {
        Line2D::from_points(self.start, self.end)
    }

    /// Get the perpendicular distance from a point to this segment
    pub fn distance_to_point(&self, point: &Point2D) -> f64 {
        let v = self.direction_unnormalized();
        let w = Vector2::new(point.x - self.start.x, point.y - self.start.y);

        let length_sq = v.dot(&v);
        if length_sq < EPSILON {
            return self.start.distance_to(point);
        }

        let t = (w.dot(&v) / length_sq).clamp(0.0, 1.0);
        let projection = self.point_at(t);
        point.distance_to(&projection)
    }

    /// Project a point onto the segment (clamped to segment bounds)
    pub fn project_point(&self, point: &Point2D) -> Point2D {
        let v = self.direction_unnormalized();
        let w = Vector2::new(point.x - self.start.x, point.y - self.start.y);

        let length_sq = v.dot(&v);
        if length_sq < EPSILON {
            return self.start;
        }

        let t = (w.dot(&v) / length_sq).clamp(0.0, 1.0);
        self.point_at(t)
    }

    /// Check if this segment is parallel to another
    pub fn is_parallel(&self, other: &LineSegment2D) -> bool {
        let d1 = self.direction();
        let d2 = other.direction();
        let cross = d1.x * d2.y - d1.y * d2.x;
        cross.abs() < EPSILON
    }

    /// Check if this segment is perpendicular to another
    pub fn is_perpendicular(&self, other: &LineSegment2D) -> bool {
        let d1 = self.direction();
        let d2 = other.direction();
        let dot = d1.dot(&d2);
        dot.abs() < EPSILON
    }

    /// Find intersection point with another line segment
    pub fn intersect(&self, other: &LineSegment2D) -> Option<Point2D> {
        let p1 = self.start;
        let p2 = self.end;
        let p3 = other.start;
        let p4 = other.end;

        let d1 = Vector2::new(p2.x - p1.x, p2.y - p1.y);
        let d2 = Vector2::new(p4.x - p3.x, p4.y - p3.y);

        let det = d1.x * d2.y - d1.y * d2.x;
        if det.abs() < EPSILON {
            return None; // Parallel or collinear
        }

        let dx = p3.x - p1.x;
        let dy = p3.y - p1.y;

        let t1 = (dx * d2.y - dy * d2.x) / det;
        let t2 = (dx * d1.y - dy * d1.x) / det;

        if t1 >= 0.0 && t1 <= 1.0 && t2 >= 0.0 && t2 <= 1.0 {
            Some(self.point_at(t1))
        } else {
            None
        }
    }

    /// Get the angle of this segment (in radians)
    pub fn angle(&self) -> f64 {
        self.start.angle_to(&self.end)
    }

    /// Reverse the segment direction
    pub fn reverse(&self) -> LineSegment2D {
        LineSegment2D::new(self.end, self.start)
    }

    /// Offset the segment by a distance (positive = left, negative = right)
    pub fn offset(&self, distance: f64) -> LineSegment2D {
        let dir = self.direction();
        let perp = Vector2::new(-dir.y, dir.x);
        let offset_start = Point2D::new(
            self.start.x + perp.x * distance,
            self.start.y + perp.y * distance,
        );
        let offset_end = Point2D::new(
            self.end.x + perp.x * distance,
            self.end.y + perp.y * distance,
        );
        LineSegment2D::new(offset_start, offset_end)
    }

    /// Get bounding box
    pub fn bounding_box(&self) -> BoundingBox2 {
        let min_x = self.start.x.min(self.end.x);
        let min_y = self.start.y.min(self.end.y);
        let max_x = self.start.x.max(self.end.x);
        let max_y = self.start.y.max(self.end.y);

        BoundingBox2::new(
            NPoint2::new(min_x, min_y),
            NPoint2::new(max_x, max_y),
        )
    }

    /// Check if a point is on the segment
    pub fn contains_point(&self, point: &Point2D) -> bool {
        self.distance_to_point(point) < EPSILON
    }

    /// Extend the segment by a distance on each end
    pub fn extend(&self, start_distance: f64, end_distance: f64) -> LineSegment2D {
        let dir = self.direction();
        let new_start = Point2D::new(
            self.start.x - dir.x * start_distance,
            self.start.y - dir.y * start_distance,
        );
        let new_end = Point2D::new(
            self.end.x + dir.x * end_distance,
            self.end.y + dir.y * end_distance,
        );
        LineSegment2D::new(new_start, new_end)
    }
}

/// 2D polyline (connected line segments)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Polyline2D {
    /// Vertices of the polyline
    pub vertices: Vec<Point2D>,
    /// Whether the polyline is closed (last vertex connects to first)
    pub closed: bool,
}

impl Polyline2D {
    /// Create a new polyline from vertices
    pub fn new(vertices: Vec<Point2D>, closed: bool) -> Self {
        Self { vertices, closed }
    }

    /// Create an open polyline
    pub fn open(vertices: Vec<Point2D>) -> Self {
        Self::new(vertices, false)
    }

    /// Create a closed polyline
    pub fn closed(vertices: Vec<Point2D>) -> Self {
        Self::new(vertices, true)
    }

    /// Get the number of vertices
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Get the number of segments
    pub fn segment_count(&self) -> usize {
        if self.vertices.len() < 2 {
            0
        } else if self.closed {
            self.vertices.len()
        } else {
            self.vertices.len() - 1
        }
    }

    /// Get a segment by index
    pub fn segment(&self, index: usize) -> Option<LineSegment2D> {
        if index >= self.segment_count() {
            return None;
        }

        let start = self.vertices[index];
        let end = if index == self.vertices.len() - 1 && self.closed {
            self.vertices[0]
        } else {
            self.vertices[index + 1]
        };

        Some(LineSegment2D::new(start, end))
    }

    /// Get all segments as a vector
    pub fn segments(&self) -> Vec<LineSegment2D> {
        (0..self.segment_count())
            .filter_map(|i| self.segment(i))
            .collect()
    }

    /// Calculate the total length
    pub fn length(&self) -> f64 {
        self.segments().iter().map(|seg| seg.length()).sum()
    }

    /// Add a vertex to the end
    pub fn add_vertex(&mut self, vertex: Point2D) {
        self.vertices.push(vertex);
    }

    /// Insert a vertex at a specific index
    pub fn insert_vertex(&mut self, index: usize, vertex: Point2D) {
        if index <= self.vertices.len() {
            self.vertices.insert(index, vertex);
        }
    }

    /// Remove a vertex at a specific index
    pub fn remove_vertex(&mut self, index: usize) -> Option<Point2D> {
        if index < self.vertices.len() {
            Some(self.vertices.remove(index))
        } else {
            None
        }
    }

    /// Close the polyline
    pub fn close(&mut self) {
        self.closed = true;
    }

    /// Open the polyline (set closed to false)
    pub fn set_open(&mut self) {
        self.closed = false;
    }

    /// Reverse the vertex order
    pub fn reverse(&mut self) {
        self.vertices.reverse();
    }

    /// Get the bounding box
    pub fn bounding_box(&self) -> Option<BoundingBox2> {
        if self.vertices.is_empty() {
            return None;
        }

        let points: Vec<NPoint2<f64>> = self.vertices.iter()
            .map(|p| NPoint2::new(p.x, p.y))
            .collect();

        BoundingBox2::from_points(&points)
    }

    /// Get the closest point on the polyline to a given point
    pub fn closest_point(&self, point: &Point2D) -> Option<Point2D> {
        let segments = self.segments();
        if segments.is_empty() {
            return None;
        }

        segments
            .iter()
            .map(|seg| seg.project_point(point))
            .min_by(|a, b| {
                let dist_a = point.distance_to(a);
                let dist_b = point.distance_to(b);
                dist_a.partial_cmp(&dist_b).unwrap()
            })
    }

    /// Get the distance from a point to the polyline
    pub fn distance_to_point(&self, point: &Point2D) -> Option<f64> {
        self.closest_point(point).map(|p| point.distance_to(&p))
    }

    /// Offset the polyline by a distance
    pub fn offset(&self, distance: f64) -> Polyline2D {
        let offset_segments: Vec<LineSegment2D> = self
            .segments()
            .iter()
            .map(|seg| seg.offset(distance))
            .collect();

        if offset_segments.is_empty() {
            return Polyline2D::new(Vec::new(), self.closed);
        }

        // Connect offset segments with miter joins
        let mut vertices = Vec::new();
        vertices.push(offset_segments[0].start);

        for i in 0..offset_segments.len() {
            let current = &offset_segments[i];
            let next_idx = (i + 1) % offset_segments.len();

            if i == offset_segments.len() - 1 && !self.closed {
                vertices.push(current.end);
                break;
            }

            let next = &offset_segments[next_idx];

            // Try to find intersection for miter join
            if let Some(line1) = current.to_line() {
                if let Some(line2) = next.to_line() {
                    if let Some(intersection) = line1.intersect(&line2) {
                        vertices.push(intersection);
                        continue;
                    }
                }
            }

            // Fallback: just use the end point
            vertices.push(current.end);
        }

        Polyline2D::new(vertices, self.closed)
    }

    /// Simplify the polyline using Douglas-Peucker algorithm
    pub fn simplify(&self, tolerance: f64) -> Polyline2D {
        if self.vertices.len() < 3 {
            return self.clone();
        }

        let simplified = douglas_peucker(&self.vertices, tolerance);
        Polyline2D::new(simplified, self.closed)
    }

    /// Check if the polyline is self-intersecting
    pub fn is_self_intersecting(&self) -> bool {
        let segments = self.segments();
        for i in 0..segments.len() {
            for j in (i + 2)..segments.len() {
                // Skip adjacent segments
                if self.closed && i == 0 && j == segments.len() - 1 {
                    continue;
                }
                if segments[i].intersect(&segments[j]).is_some() {
                    return true;
                }
            }
        }
        false
    }
}

/// Douglas-Peucker line simplification algorithm
fn douglas_peucker(points: &[Point2D], tolerance: f64) -> Vec<Point2D> {
    if points.len() < 3 {
        return points.to_vec();
    }

    let first = points.first().unwrap();
    let last = points.last().unwrap();
    let line = LineSegment2D::new(*first, *last);

    let (max_distance, max_index) = points
        .iter()
        .enumerate()
        .skip(1)
        .take(points.len() - 2)
        .map(|(i, p)| (line.distance_to_point(p), i))
        .max_by(|(d1, _), (d2, _)| d1.partial_cmp(d2).unwrap())
        .unwrap_or((0.0, 0));

    if max_distance > tolerance {
        let mut left = douglas_peucker(&points[0..=max_index], tolerance);
        let right = douglas_peucker(&points[max_index..], tolerance);
        left.pop(); // Remove duplicate point
        left.extend(right);
        left
    } else {
        vec![*first, *last]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_line_from_points() {
        let p1 = Point2D::new(0.0, 0.0);
        let p2 = Point2D::new(1.0, 1.0);
        let line = Line2D::from_points(p1, p2).unwrap();
        assert!((line.direction.x - line.direction.y).abs() < EPSILON);
    }

    #[test]
    fn test_line_intersection() {
        let line1 = Line2D::new(Point2D::new(0.0, 0.0), Vector2::new(1.0, 0.0));
        let line2 = Line2D::new(Point2D::new(5.0, -5.0), Vector2::new(0.0, 1.0));
        let intersection = line1.intersect(&line2).unwrap();
        assert!(intersection.approx_eq(&Point2D::new(5.0, 0.0)));
    }

    #[test]
    fn test_line_parallel() {
        let line1 = Line2D::new(Point2D::new(0.0, 0.0), Vector2::new(1.0, 0.0));
        let line2 = Line2D::new(Point2D::new(0.0, 1.0), Vector2::new(1.0, 0.0));
        assert!(line1.is_parallel(&line2));
    }

    #[test]
    fn test_segment_length() {
        let seg = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(3.0, 4.0));
        assert!((seg.length() - 5.0).abs() < EPSILON);
    }

    #[test]
    fn test_segment_intersection() {
        let seg1 = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(10.0, 0.0));
        let seg2 = LineSegment2D::new(Point2D::new(5.0, -5.0), Point2D::new(5.0, 5.0));
        let intersection = seg1.intersect(&seg2).unwrap();
        assert!(intersection.approx_eq(&Point2D::new(5.0, 0.0)));
    }

    #[test]
    fn test_segment_no_intersection() {
        let seg1 = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(1.0, 0.0));
        let seg2 = LineSegment2D::new(Point2D::new(0.0, 1.0), Point2D::new(1.0, 1.0));
        assert!(seg1.intersect(&seg2).is_none());
    }

    #[test]
    fn test_polyline_length() {
        let vertices = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(3.0, 0.0),
            Point2D::new(3.0, 4.0),
        ];
        let polyline = Polyline2D::open(vertices);
        assert!((polyline.length() - 7.0).abs() < EPSILON);
    }

    #[test]
    fn test_polyline_closed() {
        let vertices = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 0.0),
            Point2D::new(1.0, 1.0),
        ];
        let polyline = Polyline2D::closed(vertices);
        assert_eq!(polyline.segment_count(), 3);
    }

    #[test]
    fn test_polyline_simplify() {
        let vertices = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 0.1),
            Point2D::new(2.0, 0.0),
        ];
        let polyline = Polyline2D::open(vertices);
        let simplified = polyline.simplify(0.2);
        assert_eq!(simplified.vertices.len(), 2);
    }
}
