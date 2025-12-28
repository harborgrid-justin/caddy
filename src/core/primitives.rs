//! Geometric primitives module
//!
//! Provides fundamental geometric types for CAD operations:
//! - Points (distinct from vectors for CAD semantics)
//! - Rays for picking and intersection testing
//! - Bounding boxes for spatial queries
//! - Planes for 3D geometry
//! - Entity IDs for unique identification

use nalgebra::{Point2 as NPoint2, Point3 as NPoint3, Vector2, Vector3};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::precision::{ApproxEq, EPSILON};

// ============================================================================
// Entity Identification
// ============================================================================

/// Unique identifier for CAD entities
///
/// Uses UUID v4 for globally unique, non-sequential IDs.
/// Thread-safe and serializable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(Uuid);

impl EntityId {
    /// Create a new unique entity ID
    #[inline]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from an existing UUID
    #[inline]
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID
    #[inline]
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }

    /// Get as bytes
    #[inline]
    pub fn as_bytes(&self) -> &[u8; 16] {
        self.0.as_bytes()
    }

    /// Create from bytes
    #[inline]
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(Uuid::from_bytes(bytes))
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// Point Types
// ============================================================================

/// 2D point (distinct from Vector2 for CAD semantics)
pub type Point2 = NPoint2<f64>;

/// 3D point (distinct from Vector3 for CAD semantics)
pub type Point3 = NPoint3<f64>;

// ============================================================================
// Ray Types
// ============================================================================

/// 2D ray for picking and intersection testing
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Ray2 {
    /// Ray origin point
    pub origin: Point2,
    /// Normalized direction vector
    pub direction: Vector2<f64>,
}

impl Ray2 {
    /// Create a new ray with normalized direction
    #[inline]
    pub fn new(origin: Point2, direction: Vector2<f64>) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    /// Get a point along the ray at parameter t
    #[inline]
    pub fn point_at(&self, t: f64) -> Point2 {
        self.origin + self.direction * t
    }

    /// Find the closest point on the ray to a given point
    #[inline]
    pub fn closest_point(&self, point: &Point2) -> Point2 {
        let v = point - self.origin;
        let t = v.dot(&self.direction).max(0.0);
        self.point_at(t)
    }

    /// Calculate distance from ray to a point
    #[inline]
    pub fn distance_to_point(&self, point: &Point2) -> f64 {
        let closest = self.closest_point(point);
        nalgebra::distance(&closest, point)
    }
}

impl ApproxEq for Ray2 {
    fn approx_eq(&self, other: &Self) -> bool {
        self.origin.coords.approx_eq(&other.origin.coords)
            && self.direction.approx_eq(&other.direction)
    }

    fn approx_eq_eps(&self, other: &Self, epsilon: f64) -> bool {
        self.origin.coords.approx_eq_eps(&other.origin.coords, epsilon)
            && self.direction.approx_eq_eps(&other.direction, epsilon)
    }

    fn approx_zero(&self) -> bool {
        self.origin.coords.approx_zero() && self.direction.approx_zero()
    }

    fn approx_zero_eps(&self, epsilon: f64) -> bool {
        self.origin.coords.approx_zero_eps(epsilon) && self.direction.approx_zero_eps(epsilon)
    }
}

/// 3D ray for picking and intersection testing
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Ray3 {
    /// Ray origin point
    pub origin: Point3,
    /// Normalized direction vector
    pub direction: Vector3<f64>,
}

impl Ray3 {
    /// Create a new ray with normalized direction
    #[inline]
    pub fn new(origin: Point3, direction: Vector3<f64>) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    /// Get a point along the ray at parameter t
    #[inline]
    pub fn point_at(&self, t: f64) -> Point3 {
        self.origin + self.direction * t
    }

    /// Find the closest point on the ray to a given point
    #[inline]
    pub fn closest_point(&self, point: &Point3) -> Point3 {
        let v = point - self.origin;
        let t = v.dot(&self.direction).max(0.0);
        self.point_at(t)
    }

    /// Calculate distance from ray to a point
    #[inline]
    pub fn distance_to_point(&self, point: &Point3) -> f64 {
        let closest = self.closest_point(point);
        nalgebra::distance(&closest, point)
    }

    /// Intersect with a plane, returns t parameter if intersection exists
    #[inline]
    pub fn intersect_plane(&self, plane: &Plane) -> Option<f64> {
        let denom = self.direction.dot(&plane.normal);
        if denom.abs() < EPSILON {
            return None; // Ray is parallel to plane
        }
        let t = (plane.distance - self.origin.coords.dot(&plane.normal)) / denom;
        if t >= 0.0 {
            Some(t)
        } else {
            None
        }
    }
}

impl ApproxEq for Ray3 {
    fn approx_eq(&self, other: &Self) -> bool {
        self.origin.coords.approx_eq(&other.origin.coords)
            && self.direction.approx_eq(&other.direction)
    }

    fn approx_eq_eps(&self, other: &Self, epsilon: f64) -> bool {
        self.origin.coords.approx_eq_eps(&other.origin.coords, epsilon)
            && self.direction.approx_eq_eps(&other.direction, epsilon)
    }

    fn approx_zero(&self) -> bool {
        self.origin.coords.approx_zero() && self.direction.approx_zero()
    }

    fn approx_zero_eps(&self, epsilon: f64) -> bool {
        self.origin.coords.approx_zero_eps(epsilon) && self.direction.approx_zero_eps(epsilon)
    }
}

// ============================================================================
// Bounding Box Types
// ============================================================================

/// 2D axis-aligned bounding box
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoundingBox2 {
    /// Minimum corner
    pub min: Point2,
    /// Maximum corner
    pub max: Point2,
}

impl BoundingBox2 {
    /// Create a new bounding box
    #[inline]
    pub fn new(min: Point2, max: Point2) -> Self {
        Self { min, max }
    }

    /// Create from a collection of points
    pub fn from_points(points: &[Point2]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let mut min_x = points[0].x;
        let mut min_y = points[0].y;
        let mut max_x = points[0].x;
        let mut max_y = points[0].y;

        for p in points.iter().skip(1) {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }

        Some(Self {
            min: Point2::new(min_x, min_y),
            max: Point2::new(max_x, max_y),
        })
    }

    /// Get the width of the bounding box
    #[inline]
    pub fn width(&self) -> f64 {
        self.max.x - self.min.x
    }

    /// Get the height of the bounding box
    #[inline]
    pub fn height(&self) -> f64 {
        self.max.y - self.min.y
    }

    /// Get the center point
    #[inline]
    pub fn center(&self) -> Point2 {
        Point2::new(
            (self.min.x + self.max.x) * 0.5,
            (self.min.y + self.max.y) * 0.5,
        )
    }

    /// Get the area
    #[inline]
    pub fn area(&self) -> f64 {
        self.width() * self.height()
    }

    /// Check if a point is contained within the box
    #[inline]
    pub fn contains(&self, point: &Point2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    /// Check if this box intersects another
    #[inline]
    pub fn intersects(&self, other: &BoundingBox2) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    /// Expand the box to include a point
    #[inline]
    pub fn expand_to_include(&mut self, point: &Point2) {
        self.min.x = self.min.x.min(point.x);
        self.min.y = self.min.y.min(point.y);
        self.max.x = self.max.x.max(point.x);
        self.max.y = self.max.y.max(point.y);
    }

    /// Expand the box by a margin in all directions
    #[inline]
    pub fn expand(&self, margin: f64) -> Self {
        Self {
            min: Point2::new(self.min.x - margin, self.min.y - margin),
            max: Point2::new(self.max.x + margin, self.max.y + margin),
        }
    }

    /// Get the four corners of the box
    #[inline]
    pub fn corners(&self) -> [Point2; 4] {
        [
            self.min,
            Point2::new(self.max.x, self.min.y),
            self.max,
            Point2::new(self.min.x, self.max.y),
        ]
    }
}

impl ApproxEq for BoundingBox2 {
    fn approx_eq(&self, other: &Self) -> bool {
        self.min.coords.approx_eq(&other.min.coords) && self.max.coords.approx_eq(&other.max.coords)
    }

    fn approx_eq_eps(&self, other: &Self, epsilon: f64) -> bool {
        self.min.coords.approx_eq_eps(&other.min.coords, epsilon)
            && self.max.coords.approx_eq_eps(&other.max.coords, epsilon)
    }

    fn approx_zero(&self) -> bool {
        self.min.coords.approx_zero() && self.max.coords.approx_zero()
    }

    fn approx_zero_eps(&self, epsilon: f64) -> bool {
        self.min.coords.approx_zero_eps(epsilon) && self.max.coords.approx_zero_eps(epsilon)
    }
}

/// 3D axis-aligned bounding box
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoundingBox3 {
    /// Minimum corner
    pub min: Point3,
    /// Maximum corner
    pub max: Point3,
}

impl BoundingBox3 {
    /// Create a new bounding box
    #[inline]
    pub fn new(min: Point3, max: Point3) -> Self {
        Self { min, max }
    }

    /// Create from a collection of points
    pub fn from_points(points: &[Point3]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let mut min_x = points[0].x;
        let mut min_y = points[0].y;
        let mut min_z = points[0].z;
        let mut max_x = points[0].x;
        let mut max_y = points[0].y;
        let mut max_z = points[0].z;

        for p in points.iter().skip(1) {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            min_z = min_z.min(p.z);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
            max_z = max_z.max(p.z);
        }

        Some(Self {
            min: Point3::new(min_x, min_y, min_z),
            max: Point3::new(max_x, max_y, max_z),
        })
    }

    /// Get the width (x dimension)
    #[inline]
    pub fn width(&self) -> f64 {
        self.max.x - self.min.x
    }

    /// Get the height (y dimension)
    #[inline]
    pub fn height(&self) -> f64 {
        self.max.y - self.min.y
    }

    /// Get the depth (z dimension)
    #[inline]
    pub fn depth(&self) -> f64 {
        self.max.z - self.min.z
    }

    /// Get the center point
    #[inline]
    pub fn center(&self) -> Point3 {
        Point3::new(
            (self.min.x + self.max.x) * 0.5,
            (self.min.y + self.max.y) * 0.5,
            (self.min.z + self.max.z) * 0.5,
        )
    }

    /// Get the volume
    #[inline]
    pub fn volume(&self) -> f64 {
        self.width() * self.height() * self.depth()
    }

    /// Get the surface area
    #[inline]
    pub fn surface_area(&self) -> f64 {
        let w = self.width();
        let h = self.height();
        let d = self.depth();
        2.0 * (w * h + w * d + h * d)
    }

    /// Check if a point is contained within the box
    #[inline]
    pub fn contains(&self, point: &Point3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    /// Check if this box intersects another
    #[inline]
    pub fn intersects(&self, other: &BoundingBox3) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// Expand the box to include a point
    #[inline]
    pub fn expand_to_include(&mut self, point: &Point3) {
        self.min.x = self.min.x.min(point.x);
        self.min.y = self.min.y.min(point.y);
        self.min.z = self.min.z.min(point.z);
        self.max.x = self.max.x.max(point.x);
        self.max.y = self.max.y.max(point.y);
        self.max.z = self.max.z.max(point.z);
    }

    /// Expand the box by a margin in all directions
    #[inline]
    pub fn expand(&self, margin: f64) -> Self {
        Self {
            min: Point3::new(self.min.x - margin, self.min.y - margin, self.min.z - margin),
            max: Point3::new(self.max.x + margin, self.max.y + margin, self.max.z + margin),
        }
    }

    /// Get the eight corners of the box
    #[inline]
    pub fn corners(&self) -> [Point3; 8] {
        [
            self.min,
            Point3::new(self.max.x, self.min.y, self.min.z),
            Point3::new(self.max.x, self.max.y, self.min.z),
            Point3::new(self.min.x, self.max.y, self.min.z),
            Point3::new(self.min.x, self.min.y, self.max.z),
            Point3::new(self.max.x, self.min.y, self.max.z),
            self.max,
            Point3::new(self.min.x, self.max.y, self.max.z),
        ]
    }

    /// Intersect with a ray, returns (t_min, t_max) if intersection exists
    pub fn intersect_ray(&self, ray: &Ray3) -> Option<(f64, f64)> {
        let inv_dir = Vector3::new(
            1.0 / ray.direction.x,
            1.0 / ray.direction.y,
            1.0 / ray.direction.z,
        );

        let t1 = (self.min.x - ray.origin.x) * inv_dir.x;
        let t2 = (self.max.x - ray.origin.x) * inv_dir.x;
        let t3 = (self.min.y - ray.origin.y) * inv_dir.y;
        let t4 = (self.max.y - ray.origin.y) * inv_dir.y;
        let t5 = (self.min.z - ray.origin.z) * inv_dir.z;
        let t6 = (self.max.z - ray.origin.z) * inv_dir.z;

        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

        if tmax < 0.0 || tmin > tmax {
            None
        } else {
            Some((tmin.max(0.0), tmax))
        }
    }
}

impl ApproxEq for BoundingBox3 {
    fn approx_eq(&self, other: &Self) -> bool {
        self.min.coords.approx_eq(&other.min.coords) && self.max.coords.approx_eq(&other.max.coords)
    }

    fn approx_eq_eps(&self, other: &Self, epsilon: f64) -> bool {
        self.min.coords.approx_eq_eps(&other.min.coords, epsilon)
            && self.max.coords.approx_eq_eps(&other.max.coords, epsilon)
    }

    fn approx_zero(&self) -> bool {
        self.min.coords.approx_zero() && self.max.coords.approx_zero()
    }

    fn approx_zero_eps(&self, epsilon: f64) -> bool {
        self.min.coords.approx_zero_eps(epsilon) && self.max.coords.approx_zero_eps(epsilon)
    }
}

// ============================================================================
// Plane
// ============================================================================

/// 3D plane defined by normal and distance from origin
///
/// Plane equation: normal · p = distance
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Plane {
    /// Unit normal vector
    pub normal: Vector3<f64>,
    /// Signed distance from origin
    pub distance: f64,
}

impl Plane {
    /// Create a plane from a normal and distance
    #[inline]
    pub fn new(normal: Vector3<f64>, distance: f64) -> Self {
        Self {
            normal: normal.normalize(),
            distance,
        }
    }

    /// Create a plane from a point and normal
    #[inline]
    pub fn from_point_normal(point: Point3, normal: Vector3<f64>) -> Self {
        let normal = normal.normalize();
        let distance = point.coords.dot(&normal);
        Self { normal, distance }
    }

    /// Create a plane from three points
    pub fn from_points(p0: Point3, p1: Point3, p2: Point3) -> Option<Self> {
        let v1 = p1 - p0;
        let v2 = p2 - p0;
        let normal = v1.cross(&v2);

        if normal.norm() < EPSILON {
            return None; // Points are colinear
        }

        Some(Self::from_point_normal(p0, normal))
    }

    /// Calculate signed distance from plane to a point
    #[inline]
    pub fn distance_to_point(&self, point: &Point3) -> f64 {
        self.normal.dot(&point.coords) - self.distance
    }

    /// Project a point onto the plane
    #[inline]
    pub fn project_point(&self, point: &Point3) -> Point3 {
        let dist = self.distance_to_point(point);
        point - self.normal * dist
    }

    /// Check if a point is on the plane (within epsilon)
    #[inline]
    pub fn contains_point(&self, point: &Point3) -> bool {
        self.distance_to_point(point).abs() < EPSILON
    }

    /// Flip the plane (reverse normal)
    #[inline]
    pub fn flip(&self) -> Self {
        Self {
            normal: -self.normal,
            distance: -self.distance,
        }
    }
}

impl ApproxEq for Plane {
    fn approx_eq(&self, other: &Self) -> bool {
        self.normal.approx_eq(&other.normal) && self.distance.approx_eq(&other.distance)
    }

    fn approx_eq_eps(&self, other: &Self, epsilon: f64) -> bool {
        self.normal.approx_eq_eps(&other.normal, epsilon)
            && self.distance.approx_eq_eps(&other.distance, epsilon)
    }

    fn approx_zero(&self) -> bool {
        self.normal.approx_zero() && self.distance.approx_zero()
    }

    fn approx_zero_eps(&self, epsilon: f64) -> bool {
        self.normal.approx_zero_eps(epsilon) && self.distance.approx_zero_eps(epsilon)
    }
}

// All types are automatically Send + Sync because they contain only primitive types
// and nalgebra types which are Send + Sync

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_id() {
        let id1 = EntityId::new();
        let id2 = EntityId::new();
        assert_ne!(id1, id2);

        let uuid = id1.as_uuid();
        let id3 = EntityId::from_uuid(uuid);
        assert_eq!(id1, id3);
    }

    #[test]
    fn test_ray2_distance() {
        let ray = Ray2::new(Point2::new(0.0, 0.0), Vector2::new(1.0, 0.0));
        let point = Point2::new(5.0, 3.0);
        let dist = ray.distance_to_point(&point);
        assert!(dist.approx_eq(&3.0));
    }

    #[test]
    fn test_ray3_plane_intersection() {
        let ray = Ray3::new(Point3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0));
        let plane = Plane::from_point_normal(Point3::new(0.0, 0.0, 5.0), Vector3::new(0.0, 0.0, 1.0));
        let t = ray.intersect_plane(&plane).unwrap();
        assert!(t.approx_eq(&5.0));
    }

    #[test]
    fn test_bounding_box_2d() {
        let points = vec![
            Point2::new(0.0, 0.0),
            Point2::new(10.0, 5.0),
            Point2::new(5.0, 10.0),
        ];
        let bbox = BoundingBox2::from_points(&points).unwrap();
        assert_eq!(bbox.min, Point2::new(0.0, 0.0));
        assert_eq!(bbox.max, Point2::new(10.0, 10.0));
        assert!(bbox.contains(&Point2::new(5.0, 5.0)));
        assert!(bbox.area().approx_eq(&100.0));
    }

    #[test]
    fn test_bounding_box_3d() {
        let points = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(10.0, 10.0, 10.0),
        ];
        let bbox = BoundingBox3::from_points(&points).unwrap();
        assert!(bbox.volume().approx_eq(&1000.0));
        assert!(bbox.contains(&Point3::new(5.0, 5.0, 5.0)));
    }

    #[test]
    fn test_plane_from_points() {
        let p0 = Point3::new(0.0, 0.0, 0.0);
        let p1 = Point3::new(1.0, 0.0, 0.0);
        let p2 = Point3::new(0.0, 1.0, 0.0);
        let plane = Plane::from_points(p0, p1, p2).unwrap();

        assert!(plane.contains_point(&p0));
        assert!(plane.contains_point(&p1));
        assert!(plane.contains_point(&p2));
    }

    #[test]
    fn test_bbox_ray_intersection() {
        let bbox = BoundingBox3::new(
            Point3::new(-1.0, -1.0, -1.0),
            Point3::new(1.0, 1.0, 1.0),
        );
        let ray = Ray3::new(Point3::new(0.0, 0.0, -5.0), Vector3::new(0.0, 0.0, 1.0));
        let (tmin, tmax) = bbox.intersect_ray(&ray).unwrap();
        assert!(tmin.approx_eq(&4.0));
        assert!(tmax.approx_eq(&6.0));
    }
}
