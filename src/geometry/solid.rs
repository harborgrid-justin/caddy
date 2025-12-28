//! 3D Solid Primitive Geometry
//!
//! This module provides complete implementations of 3D solid primitives for CAD operations.
//! All solids support volume/surface area calculations, transformations, and point containment tests.

use nalgebra::{Point3, Vector3, Matrix4, Unit};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// Axis-aligned bounding box for spatial queries
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: Point3<f64>,
    pub max: Point3<f64>,
}

impl BoundingBox {
    /// Creates a new bounding box from min and max points
    pub fn new(min: Point3<f64>, max: Point3<f64>) -> Self {
        Self { min, max }
    }

    /// Creates a bounding box that contains all given points
    pub fn from_points(points: &[Point3<f64>]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let mut min = points[0];
        let mut max = points[0];

        for point in &points[1..] {
            min.x = min.x.min(point.x);
            min.y = min.y.min(point.y);
            min.z = min.z.min(point.z);
            max.x = max.x.max(point.x);
            max.y = max.y.max(point.y);
            max.z = max.z.max(point.z);
        }

        Some(Self { min, max })
    }

    /// Returns the center of the bounding box
    pub fn center(&self) -> Point3<f64> {
        Point3::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
            (self.min.z + self.max.z) / 2.0,
        )
    }

    /// Returns the dimensions of the bounding box
    pub fn dimensions(&self) -> Vector3<f64> {
        Vector3::new(
            self.max.x - self.min.x,
            self.max.y - self.min.y,
            self.max.z - self.min.z,
        )
    }

    /// Checks if a point is inside the bounding box
    pub fn contains(&self, point: &Point3<f64>) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    /// Checks if this bounding box intersects another
    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// Transforms the bounding box by a matrix
    pub fn transform(&self, matrix: &Matrix4<f64>) -> Self {
        let corners = [
            Point3::new(self.min.x, self.min.y, self.min.z),
            Point3::new(self.max.x, self.min.y, self.min.z),
            Point3::new(self.min.x, self.max.y, self.min.z),
            Point3::new(self.max.x, self.max.y, self.min.z),
            Point3::new(self.min.x, self.min.y, self.max.z),
            Point3::new(self.max.x, self.min.y, self.max.z),
            Point3::new(self.min.x, self.max.y, self.max.z),
            Point3::new(self.max.x, self.max.y, self.max.z),
        ];

        let transformed: Vec<Point3<f64>> = corners
            .iter()
            .map(|p| matrix.transform_point(p))
            .collect();

        Self::from_points(&transformed).unwrap()
    }
}

/// Trait for all 3D solid primitives
pub trait Solid3D {
    /// Calculates the volume of the solid
    fn volume(&self) -> f64;

    /// Calculates the surface area of the solid
    fn surface_area(&self) -> f64;

    /// Returns the bounding box of the solid
    fn bounding_box(&self) -> BoundingBox;

    /// Tests if a point is inside the solid
    fn contains_point(&self, point: &Point3<f64>) -> bool;

    /// Returns the centroid of the solid
    fn centroid(&self) -> Point3<f64>;

    /// Transforms the solid by a matrix
    fn transform(&self, matrix: &Matrix4<f64>) -> Self
    where
        Self: Sized;
}

/// Rectangular prism (box) primitive
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Box3D {
    /// Center point of the box
    pub center: Point3<f64>,
    /// Width (X dimension)
    pub width: f64,
    /// Height (Y dimension)
    pub height: f64,
    /// Depth (Z dimension)
    pub depth: f64,
    /// Rotation matrix
    pub rotation: Matrix4<f64>,
}

impl Box3D {
    /// Creates a new axis-aligned box
    pub fn new(center: Point3<f64>, width: f64, height: f64, depth: f64) -> Self {
        assert!(width > 0.0, "Width must be positive");
        assert!(height > 0.0, "Height must be positive");
        assert!(depth > 0.0, "Depth must be positive");

        Self {
            center,
            width,
            height,
            depth,
            rotation: Matrix4::identity(),
        }
    }

    /// Creates a box from two corner points
    pub fn from_corners(corner1: Point3<f64>, corner2: Point3<f64>) -> Self {
        let min = Point3::new(
            corner1.x.min(corner2.x),
            corner1.y.min(corner2.y),
            corner1.z.min(corner2.z),
        );
        let max = Point3::new(
            corner1.x.max(corner2.x),
            corner1.y.max(corner2.y),
            corner1.z.max(corner2.z),
        );

        let center = Point3::new(
            (min.x + max.x) / 2.0,
            (min.y + max.y) / 2.0,
            (min.z + max.z) / 2.0,
        );

        Self::new(
            center,
            max.x - min.x,
            max.y - min.y,
            max.z - min.z,
        )
    }

    /// Creates a cube with all sides equal
    pub fn cube(center: Point3<f64>, size: f64) -> Self {
        Self::new(center, size, size, size)
    }

    /// Sets the rotation of the box
    pub fn with_rotation(mut self, rotation: Matrix4<f64>) -> Self {
        self.rotation = rotation;
        self
    }

    /// Gets the 8 corner vertices of the box
    pub fn vertices(&self) -> [Point3<f64>; 8] {
        let hw = self.width / 2.0;
        let hh = self.height / 2.0;
        let hd = self.depth / 2.0;

        let local_vertices = [
            Point3::new(-hw, -hh, -hd),
            Point3::new(hw, -hh, -hd),
            Point3::new(hw, hh, -hd),
            Point3::new(-hw, hh, -hd),
            Point3::new(-hw, -hh, hd),
            Point3::new(hw, -hh, hd),
            Point3::new(hw, hh, hd),
            Point3::new(-hw, hh, hd),
        ];

        let mut vertices = [Point3::origin(); 8];
        for (i, v) in local_vertices.iter().enumerate() {
            let rotated = self.rotation.transform_point(v);
            vertices[i] = Point3::new(
                rotated.x + self.center.x,
                rotated.y + self.center.y,
                rotated.z + self.center.z,
            );
        }

        vertices
    }
}

impl Solid3D for Box3D {
    fn volume(&self) -> f64 {
        self.width * self.height * self.depth
    }

    fn surface_area(&self) -> f64 {
        2.0 * (self.width * self.height + self.width * self.depth + self.height * self.depth)
    }

    fn bounding_box(&self) -> BoundingBox {
        let vertices = self.vertices();
        BoundingBox::from_points(&vertices).unwrap()
    }

    fn contains_point(&self, point: &Point3<f64>) -> bool {
        // Transform point to local coordinates
        let inv_rotation = self.rotation.try_inverse().unwrap();
        let local_point = inv_rotation.transform_point(&Point3::new(
            point.x - self.center.x,
            point.y - self.center.y,
            point.z - self.center.z,
        ));

        let hw = self.width / 2.0;
        let hh = self.height / 2.0;
        let hd = self.depth / 2.0;

        local_point.x >= -hw
            && local_point.x <= hw
            && local_point.y >= -hh
            && local_point.y <= hh
            && local_point.z >= -hd
            && local_point.z <= hd
    }

    fn centroid(&self) -> Point3<f64> {
        self.center
    }

    fn transform(&self, matrix: &Matrix4<f64>) -> Self {
        let new_center = matrix.transform_point(&self.center);
        let new_rotation = matrix * self.rotation;

        Self {
            center: new_center,
            width: self.width,
            height: self.height,
            depth: self.depth,
            rotation: new_rotation,
        }
    }
}

/// Sphere primitive
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Sphere3D {
    /// Center point of the sphere
    pub center: Point3<f64>,
    /// Radius of the sphere
    pub radius: f64,
}

impl Sphere3D {
    /// Creates a new sphere
    pub fn new(center: Point3<f64>, radius: f64) -> Self {
        assert!(radius > 0.0, "Radius must be positive");
        Self { center, radius }
    }

    /// Returns the diameter of the sphere
    pub fn diameter(&self) -> f64 {
        self.radius * 2.0
    }

    /// Returns the distance from the center to a point
    pub fn distance_to_point(&self, point: &Point3<f64>) -> f64 {
        nalgebra::distance(&self.center, point)
    }
}

impl Solid3D for Sphere3D {
    fn volume(&self) -> f64 {
        (4.0 / 3.0) * PI * self.radius.powi(3)
    }

    fn surface_area(&self) -> f64 {
        4.0 * PI * self.radius.powi(2)
    }

    fn bounding_box(&self) -> BoundingBox {
        BoundingBox::new(
            Point3::new(
                self.center.x - self.radius,
                self.center.y - self.radius,
                self.center.z - self.radius,
            ),
            Point3::new(
                self.center.x + self.radius,
                self.center.y + self.radius,
                self.center.z + self.radius,
            ),
        )
    }

    fn contains_point(&self, point: &Point3<f64>) -> bool {
        self.distance_to_point(point) <= self.radius
    }

    fn centroid(&self) -> Point3<f64> {
        self.center
    }

    fn transform(&self, matrix: &Matrix4<f64>) -> Self {
        let new_center = matrix.transform_point(&self.center);

        // For uniform scaling, use the scale factor
        // For non-uniform scaling, this becomes an ellipsoid (not implemented here)
        let scale_x = (matrix.column(0).xyz().norm() +
                      matrix.column(1).xyz().norm() +
                      matrix.column(2).xyz().norm()) / 3.0;

        Self {
            center: new_center,
            radius: self.radius * scale_x,
        }
    }
}

/// Cylinder primitive
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Cylinder3D {
    /// Base center point
    pub base: Point3<f64>,
    /// Axis direction (normalized)
    pub axis: Unit<Vector3<f64>>,
    /// Radius of the cylinder
    pub radius: f64,
    /// Height along the axis
    pub height: f64,
}

impl Cylinder3D {
    /// Creates a new cylinder
    pub fn new(base: Point3<f64>, axis: Vector3<f64>, radius: f64, height: f64) -> Self {
        assert!(radius > 0.0, "Radius must be positive");
        assert!(height > 0.0, "Height must be positive");
        assert!(axis.norm() > 0.0, "Axis must be non-zero");

        Self {
            base,
            axis: Unit::new_normalize(axis),
            radius,
            height,
        }
    }

    /// Creates a cylinder aligned with the Z axis
    pub fn z_aligned(base: Point3<f64>, radius: f64, height: f64) -> Self {
        Self::new(base, Vector3::z(), radius, height)
    }

    /// Returns the top center point
    pub fn top(&self) -> Point3<f64> {
        self.base + self.axis.as_ref() * self.height
    }

    /// Returns the lateral surface area (excluding caps)
    pub fn lateral_area(&self) -> f64 {
        2.0 * PI * self.radius * self.height
    }

    /// Returns the area of one cap
    pub fn cap_area(&self) -> f64 {
        PI * self.radius.powi(2)
    }
}

impl Solid3D for Cylinder3D {
    fn volume(&self) -> f64 {
        PI * self.radius.powi(2) * self.height
    }

    fn surface_area(&self) -> f64 {
        self.lateral_area() + 2.0 * self.cap_area()
    }

    fn bounding_box(&self) -> BoundingBox {
        // Generate points around the base and top circles
        let mut points = Vec::with_capacity(16);

        // Find perpendicular vectors to the axis
        let perp1 = if self.axis.x.abs() < 0.9 {
            Unit::new_normalize(self.axis.cross(&Vector3::x()))
        } else {
            Unit::new_normalize(self.axis.cross(&Vector3::y()))
        };
        let perp2 = Unit::new_normalize(self.axis.cross(perp1.as_ref()));

        // Sample points around the circles
        for i in 0..8 {
            let angle = (i as f64) * PI / 4.0;
            let offset = perp1.as_ref() * angle.cos() * self.radius
                + perp2.as_ref() * angle.sin() * self.radius;

            points.push(self.base + offset);
            points.push(self.top() + offset);
        }

        BoundingBox::from_points(&points).unwrap()
    }

    fn contains_point(&self, point: &Point3<f64>) -> bool {
        let v = point - self.base;
        let projection = v.dot(self.axis.as_ref());

        if projection < 0.0 || projection > self.height {
            return false;
        }

        let axis_point = self.base + self.axis.as_ref() * projection;
        let radial_distance = nalgebra::distance(point, &axis_point);

        radial_distance <= self.radius
    }

    fn centroid(&self) -> Point3<f64> {
        self.base + self.axis.as_ref() * (self.height / 2.0)
    }

    fn transform(&self, matrix: &Matrix4<f64>) -> Self {
        let new_base = matrix.transform_point(&self.base);
        let new_axis = matrix.transform_vector(self.axis.as_ref());

        // Approximate scale
        let scale = new_axis.norm();

        Self {
            base: new_base,
            axis: Unit::new_normalize(new_axis),
            radius: self.radius * scale,
            height: self.height * scale,
        }
    }
}

/// Cone primitive
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Cone3D {
    /// Base center point
    pub base: Point3<f64>,
    /// Axis direction (normalized)
    pub axis: Unit<Vector3<f64>>,
    /// Radius of the base
    pub radius: f64,
    /// Height along the axis
    pub height: f64,
}

impl Cone3D {
    /// Creates a new cone
    pub fn new(base: Point3<f64>, axis: Vector3<f64>, radius: f64, height: f64) -> Self {
        assert!(radius > 0.0, "Radius must be positive");
        assert!(height > 0.0, "Height must be positive");
        assert!(axis.norm() > 0.0, "Axis must be non-zero");

        Self {
            base,
            axis: Unit::new_normalize(axis),
            radius,
            height,
        }
    }

    /// Creates a cone aligned with the Z axis
    pub fn z_aligned(base: Point3<f64>, radius: f64, height: f64) -> Self {
        Self::new(base, Vector3::z(), radius, height)
    }

    /// Returns the apex (tip) point
    pub fn apex(&self) -> Point3<f64> {
        self.base + self.axis.as_ref() * self.height
    }

    /// Returns the slant height
    pub fn slant_height(&self) -> f64 {
        (self.radius.powi(2) + self.height.powi(2)).sqrt()
    }

    /// Returns the lateral surface area
    pub fn lateral_area(&self) -> f64 {
        PI * self.radius * self.slant_height()
    }

    /// Returns the base area
    pub fn base_area(&self) -> f64 {
        PI * self.radius.powi(2)
    }
}

impl Solid3D for Cone3D {
    fn volume(&self) -> f64 {
        (PI * self.radius.powi(2) * self.height) / 3.0
    }

    fn surface_area(&self) -> f64 {
        self.lateral_area() + self.base_area()
    }

    fn bounding_box(&self) -> BoundingBox {
        let mut points = vec![self.apex()];

        // Find perpendicular vectors
        let perp1 = if self.axis.x.abs() < 0.9 {
            Unit::new_normalize(self.axis.cross(&Vector3::x()))
        } else {
            Unit::new_normalize(self.axis.cross(&Vector3::y()))
        };
        let perp2 = Unit::new_normalize(self.axis.cross(perp1.as_ref()));

        // Sample points around the base circle
        for i in 0..8 {
            let angle = (i as f64) * PI / 4.0;
            let offset = perp1.as_ref() * angle.cos() * self.radius
                + perp2.as_ref() * angle.sin() * self.radius;
            points.push(self.base + offset);
        }

        BoundingBox::from_points(&points).unwrap()
    }

    fn contains_point(&self, point: &Point3<f64>) -> bool {
        let v = point - self.base;
        let projection = v.dot(self.axis.as_ref());

        if projection < 0.0 || projection > self.height {
            return false;
        }

        let axis_point = self.base + self.axis.as_ref() * projection;
        let radial_distance = nalgebra::distance(point, &axis_point);

        // Radius decreases linearly from base to apex
        let radius_at_height = self.radius * (1.0 - projection / self.height);

        radial_distance <= radius_at_height
    }

    fn centroid(&self) -> Point3<f64> {
        // Centroid is at 1/4 of the height from the base
        self.base + self.axis.as_ref() * (self.height / 4.0)
    }

    fn transform(&self, matrix: &Matrix4<f64>) -> Self {
        let new_base = matrix.transform_point(&self.base);
        let new_axis = matrix.transform_vector(self.axis.as_ref());

        let scale = new_axis.norm();

        Self {
            base: new_base,
            axis: Unit::new_normalize(new_axis),
            radius: self.radius * scale,
            height: self.height * scale,
        }
    }
}

/// Torus primitive (donut shape)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Torus3D {
    /// Center point of the torus
    pub center: Point3<f64>,
    /// Normal direction (perpendicular to the torus plane)
    pub normal: Unit<Vector3<f64>>,
    /// Major radius (from center to tube center)
    pub major_radius: f64,
    /// Minor radius (tube radius)
    pub minor_radius: f64,
}

impl Torus3D {
    /// Creates a new torus
    pub fn new(
        center: Point3<f64>,
        normal: Vector3<f64>,
        major_radius: f64,
        minor_radius: f64,
    ) -> Self {
        assert!(major_radius > 0.0, "Major radius must be positive");
        assert!(minor_radius > 0.0, "Minor radius must be positive");
        assert!(
            major_radius > minor_radius,
            "Major radius must be greater than minor radius"
        );
        assert!(normal.norm() > 0.0, "Normal must be non-zero");

        Self {
            center,
            normal: Unit::new_normalize(normal),
            major_radius,
            minor_radius,
        }
    }

    /// Creates a torus in the XY plane
    pub fn xy_plane(center: Point3<f64>, major_radius: f64, minor_radius: f64) -> Self {
        Self::new(center, Vector3::z(), major_radius, minor_radius)
    }
}

impl Solid3D for Torus3D {
    fn volume(&self) -> f64 {
        2.0 * PI.powi(2) * self.major_radius * self.minor_radius.powi(2)
    }

    fn surface_area(&self) -> f64 {
        4.0 * PI.powi(2) * self.major_radius * self.minor_radius
    }

    fn bounding_box(&self) -> BoundingBox {
        // Find perpendicular vectors in the torus plane
        let perp1 = if self.normal.x.abs() < 0.9 {
            Unit::new_normalize(self.normal.cross(&Vector3::x()))
        } else {
            Unit::new_normalize(self.normal.cross(&Vector3::y()))
        };
        let perp2 = Unit::new_normalize(self.normal.cross(perp1.as_ref()));

        let outer_radius = self.major_radius + self.minor_radius;

        let mut points = Vec::new();

        // Sample points around the outer edge
        for i in 0..8 {
            let angle = (i as f64) * PI / 4.0;
            let offset = perp1.as_ref() * angle.cos() * outer_radius
                + perp2.as_ref() * angle.sin() * outer_radius;

            points.push(self.center + offset);
            points.push(self.center + offset + self.normal.as_ref() * self.minor_radius);
            points.push(self.center + offset - self.normal.as_ref() * self.minor_radius);
        }

        BoundingBox::from_points(&points).unwrap()
    }

    fn contains_point(&self, point: &Point3<f64>) -> bool {
        let v = point - self.center;

        // Project onto the torus plane
        let height = v.dot(self.normal.as_ref());
        if height.abs() > self.minor_radius {
            return false;
        }

        // Distance in the plane
        let plane_distance = (v - self.normal.as_ref() * height).norm();
        let distance_to_tube = (plane_distance - self.major_radius).abs();

        distance_to_tube <= self.minor_radius
    }

    fn centroid(&self) -> Point3<f64> {
        self.center
    }

    fn transform(&self, matrix: &Matrix4<f64>) -> Self {
        let new_center = matrix.transform_point(&self.center);
        let new_normal = matrix.transform_vector(self.normal.as_ref());

        let scale = new_normal.norm();

        Self {
            center: new_center,
            normal: Unit::new_normalize(new_normal),
            major_radius: self.major_radius * scale,
            minor_radius: self.minor_radius * scale,
        }
    }
}

/// Wedge primitive (triangular prism)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Wedge3D {
    /// Base corner point
    pub base: Point3<f64>,
    /// Width along X axis
    pub width: f64,
    /// Height along Y axis
    pub height: f64,
    /// Depth along Z axis
    pub depth: f64,
    /// Rotation matrix
    pub rotation: Matrix4<f64>,
}

impl Wedge3D {
    /// Creates a new wedge
    pub fn new(base: Point3<f64>, width: f64, height: f64, depth: f64) -> Self {
        assert!(width > 0.0, "Width must be positive");
        assert!(height > 0.0, "Height must be positive");
        assert!(depth > 0.0, "Depth must be positive");

        Self {
            base,
            width,
            height,
            depth,
            rotation: Matrix4::identity(),
        }
    }

    /// Sets the rotation of the wedge
    pub fn with_rotation(mut self, rotation: Matrix4<f64>) -> Self {
        self.rotation = rotation;
        self
    }

    /// Gets the 6 corner vertices of the wedge
    pub fn vertices(&self) -> [Point3<f64>; 6] {
        let local_vertices = [
            Point3::new(0.0, 0.0, 0.0),           // Base corner 1
            Point3::new(self.width, 0.0, 0.0),    // Base corner 2
            Point3::new(0.0, 0.0, self.depth),    // Base corner 3
            Point3::new(self.width, 0.0, self.depth), // Base corner 4
            Point3::new(0.0, self.height, 0.0),   // Top corner 1
            Point3::new(0.0, self.height, self.depth), // Top corner 2
        ];

        let mut vertices = [Point3::origin(); 6];
        for (i, v) in local_vertices.iter().enumerate() {
            let rotated = self.rotation.transform_point(v);
            vertices[i] = Point3::new(
                rotated.x + self.base.x,
                rotated.y + self.base.y,
                rotated.z + self.base.z,
            );
        }

        vertices
    }
}

impl Solid3D for Wedge3D {
    fn volume(&self) -> f64 {
        0.5 * self.width * self.height * self.depth
    }

    fn surface_area(&self) -> f64 {
        let base = self.width * self.depth;
        let back = 0.5 * self.height * self.depth;
        let bottom = self.width * self.depth;
        let front = self.width * (self.height.powi(2) + self.depth.powi(2)).sqrt();
        let side1 = 0.5 * self.height * self.depth;
        let side2 = self.width * self.height;

        base + back + bottom + front + side1 + side2
    }

    fn bounding_box(&self) -> BoundingBox {
        let vertices = self.vertices();
        BoundingBox::from_points(&vertices).unwrap()
    }

    fn contains_point(&self, point: &Point3<f64>) -> bool {
        // Transform to local coordinates
        let inv_rotation = self.rotation.try_inverse().unwrap();
        let local_point = inv_rotation.transform_point(&Point3::new(
            point.x - self.base.x,
            point.y - self.base.y,
            point.z - self.base.z,
        ));

        // Check basic bounds
        if local_point.x < 0.0
            || local_point.x > self.width
            || local_point.y < 0.0
            || local_point.y > self.height
            || local_point.z < 0.0
            || local_point.z > self.depth
        {
            return false;
        }

        // Check wedge constraint: point must be below the slant face
        // The slant face goes from (width, 0, z) to (0, height, z)
        // Equation: x/width + y/height <= 1
        local_point.x / self.width + local_point.y / self.height <= 1.0
    }

    fn centroid(&self) -> Point3<f64> {
        // Centroid of a wedge is at (w/3, h/3, d/2)
        let local_centroid = Point3::new(
            self.width / 3.0,
            self.height / 3.0,
            self.depth / 2.0,
        );

        let rotated = self.rotation.transform_point(&local_centroid);
        Point3::new(
            rotated.x + self.base.x,
            rotated.y + self.base.y,
            rotated.z + self.base.z,
        )
    }

    fn transform(&self, matrix: &Matrix4<f64>) -> Self {
        let new_base = matrix.transform_point(&self.base);
        let new_rotation = matrix * self.rotation;

        Self {
            base: new_base,
            width: self.width,
            height: self.height,
            depth: self.depth,
            rotation: new_rotation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_bounding_box() {
        let bb = BoundingBox::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(10.0, 10.0, 10.0),
        );

        assert_eq!(bb.center(), Point3::new(5.0, 5.0, 5.0));
        assert!(bb.contains(&Point3::new(5.0, 5.0, 5.0)));
        assert!(!bb.contains(&Point3::new(15.0, 5.0, 5.0)));
    }

    #[test]
    fn test_box3d() {
        let b = Box3D::new(Point3::origin(), 2.0, 4.0, 6.0);

        assert_relative_eq!(b.volume(), 48.0, epsilon = 1e-10);
        assert_relative_eq!(b.surface_area(), 88.0, epsilon = 1e-10);
        assert!(b.contains_point(&Point3::origin()));
        assert!(!b.contains_point(&Point3::new(10.0, 0.0, 0.0)));
    }

    #[test]
    fn test_sphere3d() {
        let s = Sphere3D::new(Point3::origin(), 5.0);

        assert_relative_eq!(s.volume(), (4.0 / 3.0) * PI * 125.0, epsilon = 1e-10);
        assert_relative_eq!(s.surface_area(), 4.0 * PI * 25.0, epsilon = 1e-10);
        assert!(s.contains_point(&Point3::new(3.0, 0.0, 0.0)));
        assert!(!s.contains_point(&Point3::new(10.0, 0.0, 0.0)));
    }

    #[test]
    fn test_cylinder3d() {
        let c = Cylinder3D::z_aligned(Point3::origin(), 2.0, 10.0);

        assert_relative_eq!(c.volume(), PI * 4.0 * 10.0, epsilon = 1e-10);
        assert!(c.contains_point(&Point3::new(1.0, 0.0, 5.0)));
        assert!(!c.contains_point(&Point3::new(3.0, 0.0, 5.0)));
    }

    #[test]
    fn test_cone3d() {
        let c = Cone3D::z_aligned(Point3::origin(), 3.0, 6.0);

        assert_relative_eq!(c.volume(), PI * 9.0 * 2.0, epsilon = 1e-10);
        assert!(c.contains_point(&Point3::new(0.5, 0.0, 5.0)));
        assert!(!c.contains_point(&Point3::new(2.0, 0.0, 5.0)));
    }

    #[test]
    fn test_torus3d() {
        let t = Torus3D::xy_plane(Point3::origin(), 10.0, 2.0);

        assert_relative_eq!(t.volume(), 2.0 * PI.powi(2) * 10.0 * 4.0, epsilon = 1e-10);
        assert!(t.contains_point(&Point3::new(10.0, 0.0, 0.0)));
    }

    #[test]
    fn test_wedge3d() {
        let w = Wedge3D::new(Point3::origin(), 4.0, 6.0, 8.0);

        assert_relative_eq!(w.volume(), 0.5 * 4.0 * 6.0 * 8.0, epsilon = 1e-10);
        assert!(w.contains_point(&Point3::new(1.0, 1.0, 4.0)));
    }
}
