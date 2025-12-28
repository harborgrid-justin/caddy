//! 2D Point geometry for CAD operations
//!
//! Provides Point2D with CAD-specific operations including distance calculations,
//! interpolation, polar coordinate conversion, and transformation support.

use crate::core::*;
use nalgebra::Point2 as NPoint2;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Mul, Neg, Sub};

/// 2D point with CAD-specific operations
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point2D {
    /// X coordinate
    pub x: f64,
    /// Y coordinate
    pub y: f64,
}

impl Point2D {
    /// Create a new point
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Create a point at the origin
    pub fn origin() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Create a point from polar coordinates (radius, angle in radians)
    pub fn from_polar(radius: f64, angle: f64) -> Self {
        Self {
            x: radius * angle.cos(),
            y: radius * angle.sin(),
        }
    }

    /// Convert to polar coordinates (radius, angle in radians)
    pub fn to_polar(&self) -> (f64, f64) {
        let radius = self.distance_to_origin();
        let angle = self.y.atan2(self.x);
        (radius, angle)
    }

    /// Calculate distance to another point
    pub fn distance_to(&self, other: &Point2D) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Calculate squared distance to another point (faster, avoids sqrt)
    pub fn distance_squared_to(&self, other: &Point2D) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    /// Calculate distance to origin
    pub fn distance_to_origin(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Calculate the midpoint between this and another point
    pub fn midpoint(&self, other: &Point2D) -> Point2D {
        Point2D {
            x: (self.x + other.x) / 2.0,
            y: (self.y + other.y) / 2.0,
        }
    }

    /// Linear interpolation between this and another point
    /// t = 0.0 returns self, t = 1.0 returns other
    pub fn lerp(&self, other: &Point2D, t: f64) -> Point2D {
        Point2D {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }

    /// Check if this point is approximately equal to another
    pub fn approx_eq(&self, other: &Point2D) -> bool {
        self.approx_eq_eps(other, EPSILON)
    }

    /// Check if this point is approximately equal to another with custom epsilon
    pub fn approx_eq_eps(&self, other: &Point2D, epsilon: f64) -> bool {
        (self.x - other.x).abs() < epsilon && (self.y - other.y).abs() < epsilon
    }

    /// Check if the point is at the origin
    pub fn is_origin(&self) -> bool {
        self.x.approx_zero() && self.y.approx_zero()
    }

    /// Convert to a vector from origin
    pub fn to_vector(&self) -> Vector2 {
        Vector2::new(self.x, self.y)
    }

    /// Create from a vector
    pub fn from_vector(v: Vector2) -> Self {
        Self { x: v.x, y: v.y }
    }

    /// Convert to nalgebra Point2
    pub fn to_nalgebra(&self) -> NPoint2<f64> {
        NPoint2::new(self.x, self.y)
    }

    /// Create from nalgebra Point2
    pub fn from_nalgebra(p: NPoint2<f64>) -> Self {
        Self { x: p.x, y: p.y }
    }

    /// Apply a 2D transformation
    pub fn transform(&self, transform: &Transform2D) -> Point2D {
        let v = Vector2::new(self.x, self.y);
        let transformed = transform.transform_point(&v);
        Point2D {
            x: transformed.x,
            y: transformed.y,
        }
    }

    /// Translate by a vector
    pub fn translate(&self, dx: f64, dy: f64) -> Point2D {
        Point2D {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    /// Rotate around origin by angle (radians)
    pub fn rotate(&self, angle: f64) -> Point2D {
        let cos = angle.cos();
        let sin = angle.sin();
        Point2D {
            x: self.x * cos - self.y * sin,
            y: self.x * sin + self.y * cos,
        }
    }

    /// Rotate around a center point by angle (radians)
    pub fn rotate_around(&self, center: &Point2D, angle: f64) -> Point2D {
        let translated = self.translate(-center.x, -center.y);
        let rotated = translated.rotate(angle);
        rotated.translate(center.x, center.y)
    }

    /// Scale from origin
    pub fn scale(&self, sx: f64, sy: f64) -> Point2D {
        Point2D {
            x: self.x * sx,
            y: self.y * sy,
        }
    }

    /// Scale uniformly from origin
    pub fn scale_uniform(&self, s: f64) -> Point2D {
        self.scale(s, s)
    }

    /// Mirror across the X axis
    pub fn mirror_x(&self) -> Point2D {
        Point2D {
            x: self.x,
            y: -self.y,
        }
    }

    /// Mirror across the Y axis
    pub fn mirror_y(&self) -> Point2D {
        Point2D {
            x: -self.x,
            y: self.y,
        }
    }

    /// Calculate angle from this point to another (in radians)
    pub fn angle_to(&self, other: &Point2D) -> f64 {
        (other.y - self.y).atan2(other.x - self.x)
    }

    /// Project this point onto a line defined by two points
    pub fn project_onto_line(&self, line_start: &Point2D, line_end: &Point2D) -> Point2D {
        let line_vec = Vector2::new(line_end.x - line_start.x, line_end.y - line_start.y);
        let point_vec = Vector2::new(self.x - line_start.x, self.y - line_start.y);

        let line_length_sq = line_vec.dot(&line_vec);
        if line_length_sq.approx_zero() {
            return *line_start;
        }

        let t = point_vec.dot(&line_vec) / line_length_sq;
        let projection = line_vec * t;

        Point2D {
            x: line_start.x + projection.x,
            y: line_start.y + projection.y,
        }
    }

    /// Calculate the cross product of vectors from origin to two points
    /// Useful for determining point orientation
    pub fn cross(&self, other: &Point2D) -> f64 {
        self.x * other.y - self.y * other.x
    }

    /// Calculate the dot product of vectors from origin to two points
    pub fn dot(&self, other: &Point2D) -> f64 {
        self.x * other.x + self.y * other.y
    }

    /// Normalize to unit distance from origin
    pub fn normalize(&self) -> Point2D {
        let dist = self.distance_to_origin();
        if dist.approx_zero() {
            *self
        } else {
            Point2D {
                x: self.x / dist,
                y: self.y / dist,
            }
        }
    }

    /// Get the bounding box containing just this point
    pub fn bounding_box(&self) -> BoundingBox2 {
        BoundingBox2::new(self.to_nalgebra(), self.to_nalgebra())
    }

    /// Round coordinates to a specific number of decimal places
    pub fn round(&self, decimals: u32) -> Point2D {
        let multiplier = 10_f64.powi(decimals as i32);
        Point2D {
            x: (self.x * multiplier).round() / multiplier,
            y: (self.y * multiplier).round() / multiplier,
        }
    }

    /// Snap to grid
    pub fn snap_to_grid(&self, grid_size: f64) -> Point2D {
        Point2D {
            x: (self.x / grid_size).round() * grid_size,
            y: (self.y / grid_size).round() * grid_size,
        }
    }
}

impl Default for Point2D {
    fn default() -> Self {
        Self::origin()
    }
}

impl Add for Point2D {
    type Output = Point2D;

    fn add(self, other: Point2D) -> Point2D {
        Point2D {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Point2D {
    type Output = Point2D;

    fn sub(self, other: Point2D) -> Point2D {
        Point2D {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<f64> for Point2D {
    type Output = Point2D;

    fn mul(self, scalar: f64) -> Point2D {
        Point2D {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl Div<f64> for Point2D {
    type Output = Point2D;

    fn div(self, scalar: f64) -> Point2D {
        Point2D {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}

impl Neg for Point2D {
    type Output = Point2D;

    fn neg(self) -> Point2D {
        Point2D {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl From<(f64, f64)> for Point2D {
    fn from((x, y): (f64, f64)) -> Self {
        Point2D::new(x, y)
    }
}

impl From<Point2D> for (f64, f64) {
    fn from(p: Point2D) -> (f64, f64) {
        (p.x, p.y)
    }
}

impl From<Vector2> for Point2D {
    fn from(v: Vector2) -> Self {
        Point2D { x: v.x, y: v.y }
    }
}

impl From<Point2D> for Vector2 {
    fn from(p: Point2D) -> Vector2 {
        Vector2::new(p.x, p.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_point_creation() {
        let p = Point2D::new(3.0, 4.0);
        assert_eq!(p.x, 3.0);
        assert_eq!(p.y, 4.0);
    }

    #[test]
    fn test_distance() {
        let p1 = Point2D::new(0.0, 0.0);
        let p2 = Point2D::new(3.0, 4.0);
        assert!((p1.distance_to(&p2) - 5.0).abs() < EPSILON);
    }

    #[test]
    fn test_midpoint() {
        let p1 = Point2D::new(0.0, 0.0);
        let p2 = Point2D::new(10.0, 10.0);
        let mid = p1.midpoint(&p2);
        assert!(mid.approx_eq(&Point2D::new(5.0, 5.0)));
    }

    #[test]
    fn test_lerp() {
        let p1 = Point2D::new(0.0, 0.0);
        let p2 = Point2D::new(10.0, 10.0);
        let interpolated = p1.lerp(&p2, 0.25);
        assert!(interpolated.approx_eq(&Point2D::new(2.5, 2.5)));
    }

    #[test]
    fn test_polar_conversion() {
        let p = Point2D::from_polar(5.0, PI / 4.0);
        assert!((p.x - 5.0 * (PI / 4.0).cos()).abs() < EPSILON);
        assert!((p.y - 5.0 * (PI / 4.0).sin()).abs() < EPSILON);

        let (r, theta) = p.to_polar();
        assert!((r - 5.0).abs() < EPSILON);
        assert!((theta - PI / 4.0).abs() < EPSILON);
    }

    #[test]
    fn test_rotation() {
        let p = Point2D::new(1.0, 0.0);
        let rotated = p.rotate(PI / 2.0);
        assert!(rotated.x.abs() < EPSILON);
        assert!((rotated.y - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_rotation_around_center() {
        let p = Point2D::new(2.0, 0.0);
        let center = Point2D::new(1.0, 0.0);
        let rotated = p.rotate_around(&center, PI / 2.0);
        assert!((rotated.x - 1.0).abs() < EPSILON);
        assert!((rotated.y - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_scaling() {
        let p = Point2D::new(2.0, 3.0);
        let scaled = p.scale(2.0, 3.0);
        assert!(scaled.approx_eq(&Point2D::new(4.0, 9.0)));
    }

    #[test]
    fn test_snap_to_grid() {
        let p = Point2D::new(12.7, 18.3);
        let snapped = p.snap_to_grid(5.0);
        assert!(snapped.approx_eq(&Point2D::new(15.0, 20.0)));
    }

    #[test]
    fn test_angle_to() {
        let p1 = Point2D::new(0.0, 0.0);
        let p2 = Point2D::new(1.0, 0.0);
        let angle = p1.angle_to(&p2);
        assert!(angle.abs() < EPSILON);

        let p3 = Point2D::new(0.0, 1.0);
        let angle2 = p1.angle_to(&p3);
        assert!((angle2 - PI / 2.0).abs() < EPSILON);
    }

    #[test]
    fn test_cross_product() {
        let p1 = Point2D::new(1.0, 0.0);
        let p2 = Point2D::new(0.0, 1.0);
        let cross = p1.cross(&p2);
        assert!((cross - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_dot_product() {
        let p1 = Point2D::new(1.0, 0.0);
        let p2 = Point2D::new(0.0, 1.0);
        let dot = p1.dot(&p2);
        assert!(dot.abs() < EPSILON);

        let p3 = Point2D::new(1.0, 0.0);
        let dot2 = p1.dot(&p3);
        assert!((dot2 - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_normalize() {
        let p = Point2D::new(3.0, 4.0);
        let normalized = p.normalize();
        assert!((normalized.distance_to_origin() - 1.0).abs() < EPSILON);
        assert!((normalized.x - 0.6).abs() < EPSILON);
        assert!((normalized.y - 0.8).abs() < EPSILON);
    }

    #[test]
    fn test_projection() {
        let p = Point2D::new(2.0, 2.0);
        let line_start = Point2D::new(0.0, 0.0);
        let line_end = Point2D::new(4.0, 0.0);
        let projection = p.project_onto_line(&line_start, &line_end);
        assert!(projection.approx_eq(&Point2D::new(2.0, 0.0)));
    }
}
