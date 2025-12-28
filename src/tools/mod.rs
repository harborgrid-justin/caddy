// CADDY - Selection & Tools Module
// Agent 9 - Complete selection system and manipulation tools

pub mod selection;
pub mod picking;
pub mod snap;
pub mod grid;
pub mod transform;
pub mod ortho;
pub mod grip_edit;

// Re-export commonly used types
pub use selection::{Selection, SelectionSet, SelectionMode, SelectionPreview};
pub use picking::{PickResult, PickFilter, PickPriority, Picker};
pub use snap::{SnapMode, SnapResult, SnapPoint, ObjectSnap};
pub use grid::{Grid, GridSettings, PolarGrid};
pub use transform::{TransformMode, TransformGizmo, TransformOperation};
pub use ortho::{OrthoMode, PolarTracking, SnapTracking};
pub use grip_edit::{GripPoint, GripType, GripSet, GripEditor};

use uuid::Uuid;

/// Entity identifier
pub type EntityId = Uuid;

/// 2D point
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

impl Point2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn distance_to(&self, other: &Point2) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn midpoint(&self, other: &Point2) -> Point2 {
        Point2::new((self.x + other.x) / 2.0, (self.y + other.y) / 2.0)
    }
}

/// 3D point
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }

    pub fn distance_to(&self, other: &Point3) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn to_point2(&self) -> Point2 {
        Point2::new(self.x, self.y)
    }
}

/// 2D vector
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

impl Vector2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(&self) -> Vector2 {
        let len = self.length();
        if len > 0.0 {
            Vector2::new(self.x / len, self.y / len)
        } else {
            Vector2::new(0.0, 0.0)
        }
    }

    pub fn dot(&self, other: &Vector2) -> f64 {
        self.x * other.x + self.y * other.y
    }

    pub fn angle(&self) -> f64 {
        self.y.atan2(self.x)
    }
}

/// 3D vector
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(&self) -> Vector3 {
        let len = self.length();
        if len > 0.0 {
            Vector3::new(self.x / len, self.y / len, self.z / len)
        } else {
            Vector3::new(0.0, 0.0, 0.0)
        }
    }

    pub fn dot(&self, other: &Vector3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Vector3) -> Vector3 {
        Vector3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }
}

/// Bounding box in 2D
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox2 {
    pub min: Point2,
    pub max: Point2,
}

impl BoundingBox2 {
    pub fn new(min: Point2, max: Point2) -> Self {
        Self { min, max }
    }

    pub fn from_points(p1: Point2, p2: Point2) -> Self {
        let min_x = p1.x.min(p2.x);
        let min_y = p1.y.min(p2.y);
        let max_x = p1.x.max(p2.x);
        let max_y = p1.y.max(p2.y);
        Self {
            min: Point2::new(min_x, min_y),
            max: Point2::new(max_x, max_y),
        }
    }

    pub fn contains(&self, point: &Point2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    pub fn intersects(&self, point: &Point2, tolerance: f64) -> bool {
        point.x >= self.min.x - tolerance
            && point.x <= self.max.x + tolerance
            && point.y >= self.min.y - tolerance
            && point.y <= self.max.y + tolerance
    }
}

/// Ray for 3D picking
#[derive(Debug, Clone, Copy)]
pub struct Ray3 {
    pub origin: Point3,
    pub direction: Vector3,
}

impl Ray3 {
    pub fn new(origin: Point3, direction: Vector3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    pub fn point_at(&self, t: f64) -> Point3 {
        Point3::new(
            self.origin.x + self.direction.x * t,
            self.origin.y + self.direction.y * t,
            self.origin.z + self.direction.z * t,
        )
    }
}

/// Entity type for filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityType {
    Point,
    Line,
    Arc,
    Circle,
    Polyline,
    Spline,
    Text,
    Hatch,
    Dimension,
    Block,
    Solid,
    Surface,
    Mesh,
}

/// Layer information stub
#[derive(Debug, Clone)]
pub struct LayerInfo {
    pub name: String,
    pub visible: bool,
    pub locked: bool,
}

/// Entity data stub
#[derive(Debug, Clone)]
pub struct Entity {
    pub id: EntityId,
    pub entity_type: EntityType,
    pub layer: String,
    pub bounds: Option<BoundingBox2>,
}

impl Entity {
    pub fn new(entity_type: EntityType) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_type,
            layer: "0".to_string(),
            bounds: None,
        }
    }

    pub fn with_bounds(mut self, bounds: BoundingBox2) -> Self {
        self.bounds = Some(bounds);
        self
    }
}

/// Matrix 4x4 for transformations
#[derive(Debug, Clone, Copy)]
pub struct Matrix4 {
    pub data: [[f64; 4]; 4],
}

impl Matrix4 {
    pub fn identity() -> Self {
        Self {
            data: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn translation(x: f64, y: f64, z: f64) -> Self {
        Self {
            data: [
                [1.0, 0.0, 0.0, x],
                [0.0, 1.0, 0.0, y],
                [0.0, 0.0, 1.0, z],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn rotation_z(angle: f64) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self {
            data: [
                [cos, -sin, 0.0, 0.0],
                [sin, cos, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn scale(sx: f64, sy: f64, sz: f64) -> Self {
        Self {
            data: [
                [sx, 0.0, 0.0, 0.0],
                [0.0, sy, 0.0, 0.0],
                [0.0, 0.0, sz, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn transform_point(&self, point: &Point3) -> Point3 {
        let x = self.data[0][0] * point.x
            + self.data[0][1] * point.y
            + self.data[0][2] * point.z
            + self.data[0][3];
        let y = self.data[1][0] * point.x
            + self.data[1][1] * point.y
            + self.data[1][2] * point.z
            + self.data[1][3];
        let z = self.data[2][0] * point.x
            + self.data[2][1] * point.y
            + self.data[2][2] * point.z
            + self.data[2][3];
        Point3::new(x, y, z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point2_distance() {
        let p1 = Point2::new(0.0, 0.0);
        let p2 = Point2::new(3.0, 4.0);
        assert!((p1.distance_to(&p2) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_vector2_normalize() {
        let v = Vector2::new(3.0, 4.0);
        let n = v.normalize();
        assert!((n.length() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_bounding_box() {
        let bbox = BoundingBox2::from_points(Point2::new(0.0, 0.0), Point2::new(10.0, 10.0));
        assert!(bbox.contains(&Point2::new(5.0, 5.0)));
        assert!(!bbox.contains(&Point2::new(15.0, 5.0)));
    }
}
