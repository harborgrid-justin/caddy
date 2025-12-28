//! 2D Arc and Circle geometry for CAD operations
//!
//! Provides circular arcs, circles, ellipses, and elliptical arcs with
//! intersection algorithms, tangent calculations, and geometric operations.

use crate::core::*;
use crate::core::precision::normalize_angle;
use crate::geometry::line::{Line2D, LineSegment2D};
use crate::geometry::point::Point2D;
use nalgebra::Point2 as NPoint2;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// 2D circular arc
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Arc2D {
    /// Center point
    pub center: Point2D,
    /// Radius
    pub radius: f64,
    /// Start angle in radians
    pub start_angle: f64,
    /// End angle in radians
    pub end_angle: f64,
    /// Counterclockwise direction
    pub ccw: bool,
}

impl Arc2D {
    /// Create a new arc
    pub fn new(center: Point2D, radius: f64, start_angle: f64, end_angle: f64, ccw: bool) -> Self {
        Self {
            center,
            radius,
            start_angle: normalize_angle(start_angle),
            end_angle: normalize_angle(end_angle),
            ccw,
        }
    }

    /// Create an arc from three points
    pub fn from_three_points(p1: Point2D, p2: Point2D, p3: Point2D) -> Option<Self> {
        // Find center of circle passing through three points
        let center = circle_center_from_three_points(p1, p2, p3)?;
        let radius = center.distance_to(&p1);

        let start_angle = (p1.y - center.y).atan2(p1.x - center.x);
        let end_angle = (p3.y - center.y).atan2(p3.x - center.x);

        // Determine if arc is CCW by checking if p2 is on the CCW side
        let mid_angle = (p2.y - center.y).atan2(p2.x - center.x);
        let ccw = is_angle_between(mid_angle, start_angle, end_angle, true);

        Some(Arc2D::new(center, radius, start_angle, end_angle, ccw))
    }

    /// Get a point on the arc at the given angle
    pub fn point_at_angle(&self, angle: f64) -> Point2D {
        Point2D::new(
            self.center.x + self.radius * angle.cos(),
            self.center.y + self.radius * angle.sin(),
        )
    }

    /// Get a point on the arc at parameter t [0, 1]
    pub fn point_at(&self, t: f64) -> Point2D {
        let angle = self.angle_at(t);
        self.point_at_angle(angle)
    }

    /// Get the angle at parameter t [0, 1]
    pub fn angle_at(&self, t: f64) -> f64 {
        let sweep = self.sweep_angle();
        self.start_angle + sweep * t
    }

    /// Get the start point
    pub fn start_point(&self) -> Point2D {
        self.point_at_angle(self.start_angle)
    }

    /// Get the end point
    pub fn end_point(&self) -> Point2D {
        self.point_at_angle(self.end_angle)
    }

    /// Get the sweep angle (unsigned)
    pub fn sweep_angle(&self) -> f64 {
        let mut sweep = if self.ccw {
            self.end_angle - self.start_angle
        } else {
            self.start_angle - self.end_angle
        };

        while sweep < 0.0 {
            sweep += 2.0 * PI;
        }
        while sweep > 2.0 * PI {
            sweep -= 2.0 * PI;
        }

        sweep
    }

    /// Get the arc length
    pub fn length(&self) -> f64 {
        self.radius * self.sweep_angle()
    }

    /// Get the midpoint of the arc
    pub fn midpoint(&self) -> Point2D {
        self.point_at(0.5)
    }

    /// Check if an angle is on the arc
    pub fn contains_angle(&self, angle: f64) -> bool {
        is_angle_between(angle, self.start_angle, self.end_angle, self.ccw)
    }

    /// Check if a point is on the arc
    pub fn contains_point(&self, point: &Point2D) -> bool {
        let dist = self.center.distance_to(point);
        if (dist - self.radius).abs() > EPSILON {
            return false;
        }

        let angle = (point.y - self.center.y).atan2(point.x - self.center.x);
        self.contains_angle(angle)
    }

    /// Get the tangent vector at a given angle
    pub fn tangent_at_angle(&self, angle: f64) -> Vector2 {
        if self.ccw {
            Vector2::new(-angle.sin(), angle.cos())
        } else {
            Vector2::new(angle.sin(), -angle.cos())
        }
    }

    /// Get the tangent vector at parameter t
    pub fn tangent_at(&self, t: f64) -> Vector2 {
        self.tangent_at_angle(self.angle_at(t))
    }

    /// Get the bounding box
    pub fn bounding_box(&self) -> BoundingBox2 {
        let mut min_x = self.start_point().x.min(self.end_point().x);
        let mut max_x = self.start_point().x.max(self.end_point().x);
        let mut min_y = self.start_point().y.min(self.end_point().y);
        let mut max_y = self.start_point().y.max(self.end_point().y);

        // Check if arc crosses cardinal directions
        if self.contains_angle(0.0) {
            max_x = max_x.max(self.center.x + self.radius);
        }
        if self.contains_angle(PI / 2.0) {
            max_y = max_y.max(self.center.y + self.radius);
        }
        if self.contains_angle(PI) {
            min_x = min_x.min(self.center.x - self.radius);
        }
        if self.contains_angle(3.0 * PI / 2.0) {
            min_y = min_y.min(self.center.y - self.radius);
        }

        BoundingBox2::new(NPoint2::new(min_x, min_y), NPoint2::new(max_x, max_y))
    }

    /// Intersect with a line
    pub fn intersect_line(&self, line: &Line2D) -> Vec<Point2D> {
        let circle = Circle2D::new(self.center, self.radius);
        circle
            .intersect_line(line)
            .into_iter()
            .filter(|p| self.contains_point(p))
            .collect()
    }

    /// Intersect with a line segment
    pub fn intersect_segment(&self, segment: &LineSegment2D) -> Vec<Point2D> {
        let circle = Circle2D::new(self.center, self.radius);
        circle
            .intersect_segment(segment)
            .into_iter()
            .filter(|p| self.contains_point(p))
            .collect()
    }

    /// Intersect with another arc
    pub fn intersect_arc(&self, other: &Arc2D) -> Vec<Point2D> {
        let circle1 = Circle2D::new(self.center, self.radius);
        let circle2 = Circle2D::new(other.center, other.radius);
        circle1
            .intersect_circle(&circle2)
            .into_iter()
            .filter(|p| self.contains_point(p) && other.contains_point(p))
            .collect()
    }

    /// Reverse the arc direction
    pub fn reverse(&self) -> Arc2D {
        Arc2D::new(
            self.center,
            self.radius,
            self.end_angle,
            self.start_angle,
            !self.ccw,
        )
    }

    /// Convert to a circle if it's a full circle
    pub fn to_circle(&self) -> Option<Circle2D> {
        if (self.sweep_angle() - 2.0 * PI).abs() < EPSILON {
            Some(Circle2D::new(self.center, self.radius))
        } else {
            None
        }
    }
}

/// 2D circle
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Circle2D {
    /// Center point
    pub center: Point2D,
    /// Radius
    pub radius: f64,
}

impl Circle2D {
    /// Create a new circle
    pub fn new(center: Point2D, radius: f64) -> Self {
        Self { center, radius }
    }

    /// Create a circle from three points
    pub fn from_three_points(p1: Point2D, p2: Point2D, p3: Point2D) -> Option<Self> {
        let center = circle_center_from_three_points(p1, p2, p3)?;
        let radius = center.distance_to(&p1);
        Some(Circle2D::new(center, radius))
    }

    /// Get the circumference
    pub fn circumference(&self) -> f64 {
        2.0 * PI * self.radius
    }

    /// Get the area
    pub fn area(&self) -> f64 {
        PI * self.radius * self.radius
    }

    /// Get a point on the circle at the given angle
    pub fn point_at_angle(&self, angle: f64) -> Point2D {
        Point2D::new(
            self.center.x + self.radius * angle.cos(),
            self.center.y + self.radius * angle.sin(),
        )
    }

    /// Check if a point is inside the circle
    pub fn contains_point(&self, point: &Point2D) -> bool {
        self.center.distance_to(point) <= self.radius + EPSILON
    }

    /// Check if a point is on the circle
    pub fn point_on_circle(&self, point: &Point2D) -> bool {
        (self.center.distance_to(point) - self.radius).abs() < EPSILON
    }

    /// Get the tangent lines from an external point
    pub fn tangent_lines_from_point(&self, point: &Point2D) -> Option<(Line2D, Line2D)> {
        let dist = self.center.distance_to(point);
        if dist <= self.radius + EPSILON {
            return None; // Point is inside or on circle
        }

        let angle_to_center = self.center.angle_to(point);
        let angle_offset = (self.radius / dist).asin();

        let angle1 = angle_to_center + angle_offset;
        let angle2 = angle_to_center - angle_offset;

        let tangent_point1 = self.point_at_angle(angle1);
        let tangent_point2 = self.point_at_angle(angle2);

        let line1 = Line2D::from_points(*point, tangent_point1)?;
        let line2 = Line2D::from_points(*point, tangent_point2)?;

        Some((line1, line2))
    }

    /// Intersect with a line
    pub fn intersect_line(&self, line: &Line2D) -> Vec<Point2D> {
        let closest = line.project_point(&self.center);
        let dist = self.center.distance_to(&closest);

        if dist > self.radius + EPSILON {
            return Vec::new();
        }

        if (dist - self.radius).abs() < EPSILON {
            return vec![closest];
        }

        let offset = (self.radius * self.radius - dist * dist).sqrt();
        let p1 = Point2D::new(
            closest.x + line.direction.x * offset,
            closest.y + line.direction.y * offset,
        );
        let p2 = Point2D::new(
            closest.x - line.direction.x * offset,
            closest.y - line.direction.y * offset,
        );

        vec![p1, p2]
    }

    /// Intersect with a line segment
    pub fn intersect_segment(&self, segment: &LineSegment2D) -> Vec<Point2D> {
        if let Some(line) = segment.to_line() {
            self.intersect_line(&line)
                .into_iter()
                .filter(|p| segment.contains_point(p))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Intersect with another circle
    pub fn intersect_circle(&self, other: &Circle2D) -> Vec<Point2D> {
        let dist = self.center.distance_to(&other.center);

        if dist > self.radius + other.radius + EPSILON {
            return Vec::new(); // Too far apart
        }

        if dist < (self.radius - other.radius).abs() - EPSILON {
            return Vec::new(); // One circle inside the other
        }

        if dist < EPSILON {
            return Vec::new(); // Concentric circles
        }

        let a = (self.radius * self.radius - other.radius * other.radius + dist * dist) / (2.0 * dist);
        let h = (self.radius * self.radius - a * a).sqrt();

        let dx = (other.center.x - self.center.x) / dist;
        let dy = (other.center.y - self.center.y) / dist;

        let px = self.center.x + a * dx;
        let py = self.center.y + a * dy;

        if h.abs() < EPSILON {
            return vec![Point2D::new(px, py)];
        }

        let p1 = Point2D::new(px + h * dy, py - h * dx);
        let p2 = Point2D::new(px - h * dy, py + h * dx);

        vec![p1, p2]
    }

    /// Get the bounding box
    pub fn bounding_box(&self) -> BoundingBox2 {
        BoundingBox2::new(
            NPoint2::new(self.center.x - self.radius, self.center.y - self.radius),
            NPoint2::new(self.center.x + self.radius, self.center.y + self.radius),
        )
    }

    /// Convert to an arc (full circle)
    pub fn to_arc(&self) -> Arc2D {
        Arc2D::new(self.center, self.radius, 0.0, 2.0 * PI, true)
    }
}

/// 2D ellipse
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ellipse2D {
    /// Center point
    pub center: Point2D,
    /// Semi-major axis length
    pub semi_major: f64,
    /// Semi-minor axis length
    pub semi_minor: f64,
    /// Rotation angle in radians
    pub rotation: f64,
}

impl Ellipse2D {
    /// Create a new ellipse
    pub fn new(center: Point2D, semi_major: f64, semi_minor: f64, rotation: f64) -> Self {
        Self {
            center,
            semi_major,
            semi_minor,
            rotation,
        }
    }

    /// Get a point on the ellipse at the given angle (in ellipse parameter space)
    pub fn point_at_angle(&self, angle: f64) -> Point2D {
        let x = self.semi_major * angle.cos();
        let y = self.semi_minor * angle.sin();

        let cos_rot = self.rotation.cos();
        let sin_rot = self.rotation.sin();

        Point2D::new(
            self.center.x + x * cos_rot - y * sin_rot,
            self.center.y + x * sin_rot + y * cos_rot,
        )
    }

    /// Get a point on the ellipse at parameter t [0, 1]
    pub fn point_at(&self, t: f64) -> Point2D {
        self.point_at_angle(t * 2.0 * PI)
    }

    /// Get the circumference (approximation using Ramanujan's formula)
    pub fn circumference(&self) -> f64 {
        let a = self.semi_major;
        let b = self.semi_minor;
        let h = ((a - b) * (a - b)) / ((a + b) * (a + b));
        PI * (a + b) * (1.0 + (3.0 * h) / (10.0 + (4.0 - 3.0 * h).sqrt()))
    }

    /// Get the area
    pub fn area(&self) -> f64 {
        PI * self.semi_major * self.semi_minor
    }

    /// Get the eccentricity
    pub fn eccentricity(&self) -> f64 {
        let a = self.semi_major;
        let b = self.semi_minor;
        (1.0 - (b * b) / (a * a)).sqrt()
    }

    /// Get the bounding box
    pub fn bounding_box(&self) -> BoundingBox2 {
        // For rotated ellipse, we need to find the extrema
        let cos_rot = self.rotation.cos();
        let sin_rot = self.rotation.sin();

        let a = self.semi_major;
        let b = self.semi_minor;

        let ux = a * cos_rot;
        let uy = a * sin_rot;
        let vx = b * -sin_rot;
        let vy = b * cos_rot;

        let half_width = (ux * ux + vx * vx).sqrt();
        let half_height = (uy * uy + vy * vy).sqrt();

        BoundingBox2::new(
            NPoint2::new(
                self.center.x - half_width,
                self.center.y - half_height,
            ),
            NPoint2::new(
                self.center.x + half_width,
                self.center.y + half_height,
            ),
        )
    }

    /// Check if a point is inside the ellipse
    pub fn contains_point(&self, point: &Point2D) -> bool {
        // Transform point to ellipse coordinate system
        let cos_rot = self.rotation.cos();
        let sin_rot = self.rotation.sin();

        let dx = point.x - self.center.x;
        let dy = point.y - self.center.y;

        let x = dx * cos_rot + dy * sin_rot;
        let y = -dx * sin_rot + dy * cos_rot;

        let value = (x * x) / (self.semi_major * self.semi_major)
            + (y * y) / (self.semi_minor * self.semi_minor);

        value <= 1.0 + EPSILON
    }

    /// Convert to a circle if semi-major equals semi-minor
    pub fn to_circle(&self) -> Option<Circle2D> {
        if (self.semi_major - self.semi_minor).abs() < EPSILON {
            Some(Circle2D::new(self.center, self.semi_major))
        } else {
            None
        }
    }
}

/// 2D elliptical arc
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EllipticalArc2D {
    /// Center point
    pub center: Point2D,
    /// Semi-major axis length
    pub semi_major: f64,
    /// Semi-minor axis length
    pub semi_minor: f64,
    /// Rotation angle in radians
    pub rotation: f64,
    /// Start angle in radians (in ellipse parameter space)
    pub start_angle: f64,
    /// End angle in radians (in ellipse parameter space)
    pub end_angle: f64,
    /// Counterclockwise direction
    pub ccw: bool,
}

impl EllipticalArc2D {
    /// Create a new elliptical arc
    pub fn new(
        center: Point2D,
        semi_major: f64,
        semi_minor: f64,
        rotation: f64,
        start_angle: f64,
        end_angle: f64,
        ccw: bool,
    ) -> Self {
        Self {
            center,
            semi_major,
            semi_minor,
            rotation,
            start_angle: normalize_angle(start_angle),
            end_angle: normalize_angle(end_angle),
            ccw,
        }
    }

    /// Get a point on the arc at the given angle
    pub fn point_at_angle(&self, angle: f64) -> Point2D {
        let ellipse = Ellipse2D::new(self.center, self.semi_major, self.semi_minor, self.rotation);
        ellipse.point_at_angle(angle)
    }

    /// Get a point on the arc at parameter t [0, 1]
    pub fn point_at(&self, t: f64) -> Point2D {
        let angle = self.angle_at(t);
        self.point_at_angle(angle)
    }

    /// Get the angle at parameter t [0, 1]
    pub fn angle_at(&self, t: f64) -> f64 {
        let sweep = self.sweep_angle();
        self.start_angle + sweep * t
    }

    /// Get the sweep angle
    pub fn sweep_angle(&self) -> f64 {
        let mut sweep = if self.ccw {
            self.end_angle - self.start_angle
        } else {
            self.start_angle - self.end_angle
        };

        while sweep < 0.0 {
            sweep += 2.0 * PI;
        }
        while sweep > 2.0 * PI {
            sweep -= 2.0 * PI;
        }

        sweep
    }

    /// Get the start point
    pub fn start_point(&self) -> Point2D {
        self.point_at_angle(self.start_angle)
    }

    /// Get the end point
    pub fn end_point(&self) -> Point2D {
        self.point_at_angle(self.end_angle)
    }

    /// Convert to a full ellipse if it's a full arc
    pub fn to_ellipse(&self) -> Option<Ellipse2D> {
        if (self.sweep_angle() - 2.0 * PI).abs() < EPSILON {
            Some(Ellipse2D::new(
                self.center,
                self.semi_major,
                self.semi_minor,
                self.rotation,
            ))
        } else {
            None
        }
    }

    /// Convert to a circular arc if semi-major equals semi-minor
    pub fn to_arc(&self) -> Option<Arc2D> {
        if (self.semi_major - self.semi_minor).abs() < EPSILON {
            Some(Arc2D::new(
                self.center,
                self.semi_major,
                self.start_angle,
                self.end_angle,
                self.ccw,
            ))
        } else {
            None
        }
    }
}

// Helper functions

/// Find the center of a circle passing through three points
fn circle_center_from_three_points(p1: Point2D, p2: Point2D, p3: Point2D) -> Option<Point2D> {
    let ax = p1.x;
    let ay = p1.y;
    let bx = p2.x;
    let by = p2.y;
    let cx = p3.x;
    let cy = p3.y;

    let d = 2.0 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by));

    if d.abs() < EPSILON {
        return None; // Points are collinear
    }

    let ux = ((ax * ax + ay * ay) * (by - cy)
        + (bx * bx + by * by) * (cy - ay)
        + (cx * cx + cy * cy) * (ay - by))
        / d;

    let uy = ((ax * ax + ay * ay) * (cx - bx)
        + (bx * bx + by * by) * (ax - cx)
        + (cx * cx + cy * cy) * (bx - ax))
        / d;

    Some(Point2D::new(ux, uy))
}

/// Check if an angle is between two other angles
fn is_angle_between(angle: f64, start: f64, end: f64, ccw: bool) -> bool {
    let angle = normalize_angle(angle);
    let start = normalize_angle(start);
    let end = normalize_angle(end);

    if ccw {
        if start <= end {
            angle >= start && angle <= end
        } else {
            angle >= start || angle <= end
        }
    } else {
        if start >= end {
            angle <= start && angle >= end
        } else {
            angle <= start || angle >= end
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_creation() {
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 5.0);
        assert_eq!(circle.radius, 5.0);
        assert!((circle.area() - 78.53981633974483).abs() < EPSILON);
    }

    #[test]
    fn test_circle_from_three_points() {
        let p1 = Point2D::new(1.0, 0.0);
        let p2 = Point2D::new(0.0, 1.0);
        let p3 = Point2D::new(-1.0, 0.0);
        let circle = Circle2D::from_three_points(p1, p2, p3).unwrap();
        assert!(circle.center.approx_eq(&Point2D::new(0.0, 0.0)));
        assert!((circle.radius - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_circle_contains_point() {
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 5.0);
        assert!(circle.contains_point(&Point2D::new(3.0, 4.0)));
        assert!(!circle.contains_point(&Point2D::new(4.0, 4.0)));
    }

    #[test]
    fn test_circle_line_intersection() {
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 5.0);
        let line = Line2D::new(Point2D::new(-10.0, 0.0), Vector2::new(1.0, 0.0));
        let intersections = circle.intersect_line(&line);
        assert_eq!(intersections.len(), 2);
    }

    #[test]
    fn test_arc_sweep_angle() {
        let arc = Arc2D::new(Point2D::new(0.0, 0.0), 5.0, 0.0, PI / 2.0, true);
        assert!((arc.sweep_angle() - PI / 2.0).abs() < EPSILON);
    }

    #[test]
    fn test_arc_from_three_points() {
        let p1 = Point2D::new(1.0, 0.0);
        let p2 = Point2D::new(0.0, 1.0);
        let p3 = Point2D::new(-1.0, 0.0);
        let arc = Arc2D::from_three_points(p1, p2, p3).unwrap();
        assert!(arc.center.approx_eq(&Point2D::new(0.0, 0.0)));
        assert!((arc.radius - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_ellipse_area() {
        let ellipse = Ellipse2D::new(Point2D::new(0.0, 0.0), 5.0, 3.0, 0.0);
        assert!((ellipse.area() - 47.12388980384690).abs() < EPSILON);
    }

    #[test]
    fn test_ellipse_eccentricity() {
        let ellipse = Ellipse2D::new(Point2D::new(0.0, 0.0), 5.0, 3.0, 0.0);
        let e = ellipse.eccentricity();
        assert!((e - 0.8).abs() < EPSILON);
    }

    #[test]
    fn test_ellipse_contains_point() {
        let ellipse = Ellipse2D::new(Point2D::new(0.0, 0.0), 5.0, 3.0, 0.0);
        assert!(ellipse.contains_point(&Point2D::new(0.0, 0.0)));
        assert!(ellipse.contains_point(&Point2D::new(4.0, 0.0)));
        assert!(!ellipse.contains_point(&Point2D::new(6.0, 0.0)));
    }
}
