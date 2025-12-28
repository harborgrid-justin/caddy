//! Mathematics module - vectors, matrices, quaternions, and transformations
//!
//! This module provides comprehensive mathematical types and operations
//! for 2D and 3D CAD geometry, built on top of nalgebra for performance
//! and correctness.

use nalgebra::{
    Matrix3 as NalMatrix3, Matrix4 as NalMatrix4, Quaternion as NalQuaternion,
    UnitQuaternion, Vector2 as NalVector2, Vector3 as NalVector3, Vector4 as NalVector4,
};
use serde::{Deserialize, Serialize};
use std::ops::Mul;

use super::precision::{ApproxEq, EPSILON};

// ============================================================================
// Vector Types
// ============================================================================

/// 2D vector (using nalgebra)
pub type Vector2 = NalVector2<f64>;

/// 3D vector (using nalgebra)
pub type Vector3 = NalVector3<f64>;

/// 4D vector (using nalgebra)
pub type Vector4 = NalVector4<f64>;

/// 3x3 matrix (using nalgebra)
pub type Matrix3 = NalMatrix3<f64>;

/// 4x4 matrix (using nalgebra)
pub type Matrix4 = NalMatrix4<f64>;

/// Quaternion (using nalgebra)
pub type Quaternion = NalQuaternion<f64>;

// ============================================================================
// ApproxEq implementations for nalgebra types
// ============================================================================

impl ApproxEq for Vector2 {
    #[inline]
    fn approx_eq(&self, other: &Self) -> bool {
        self.approx_eq_eps(other, EPSILON)
    }

    #[inline]
    fn approx_eq_eps(&self, other: &Self, epsilon: f64) -> bool {
        (self - other).norm() < epsilon
    }

    #[inline]
    fn approx_zero(&self) -> bool {
        self.approx_zero_eps(EPSILON)
    }

    #[inline]
    fn approx_zero_eps(&self, epsilon: f64) -> bool {
        self.norm() < epsilon
    }
}

impl ApproxEq for Vector3 {
    #[inline]
    fn approx_eq(&self, other: &Self) -> bool {
        self.approx_eq_eps(other, EPSILON)
    }

    #[inline]
    fn approx_eq_eps(&self, other: &Self, epsilon: f64) -> bool {
        (self - other).norm() < epsilon
    }

    #[inline]
    fn approx_zero(&self) -> bool {
        self.approx_zero_eps(EPSILON)
    }

    #[inline]
    fn approx_zero_eps(&self, epsilon: f64) -> bool {
        self.norm() < epsilon
    }
}

impl ApproxEq for Vector4 {
    #[inline]
    fn approx_eq(&self, other: &Self) -> bool {
        self.approx_eq_eps(other, EPSILON)
    }

    #[inline]
    fn approx_eq_eps(&self, other: &Self, epsilon: f64) -> bool {
        (self - other).norm() < epsilon
    }

    #[inline]
    fn approx_zero(&self) -> bool {
        self.approx_zero_eps(EPSILON)
    }

    #[inline]
    fn approx_zero_eps(&self, epsilon: f64) -> bool {
        self.norm() < epsilon
    }
}

// ============================================================================
// 2D Transformation
// ============================================================================

/// 2D transformation matrix
///
/// Represents translation, rotation, and scaling in 2D space.
/// Uses homogeneous coordinates internally (3x3 matrix).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform2D {
    /// Internal 3x3 transformation matrix
    pub matrix: Matrix3,
}

impl Transform2D {
    /// Create identity transform
    #[inline]
    pub fn identity() -> Self {
        Self {
            matrix: Matrix3::identity(),
        }
    }

    /// Create translation transform
    #[inline]
    pub fn translation(x: f64, y: f64) -> Self {
        Self {
            matrix: Matrix3::new(
                1.0, 0.0, x,
                0.0, 1.0, y,
                0.0, 0.0, 1.0,
            ),
        }
    }

    /// Create rotation transform (angle in radians)
    #[inline]
    pub fn rotation(angle: f64) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self {
            matrix: Matrix3::new(
                cos, -sin, 0.0,
                sin, cos, 0.0,
                0.0, 0.0, 1.0,
            ),
        }
    }

    /// Create uniform scale transform
    #[inline]
    pub fn scale(scale: f64) -> Self {
        Self::scale_non_uniform(scale, scale)
    }

    /// Create non-uniform scale transform
    #[inline]
    pub fn scale_non_uniform(sx: f64, sy: f64) -> Self {
        Self {
            matrix: Matrix3::new(
                sx, 0.0, 0.0,
                0.0, sy, 0.0,
                0.0, 0.0, 1.0,
            ),
        }
    }

    /// Transform a point (applies translation)
    #[inline]
    pub fn transform_point(&self, point: &Vector2) -> Vector2 {
        let result = self.matrix * Vector3::new(point.x, point.y, 1.0);
        Vector2::new(result.x, result.y)
    }

    /// Transform a vector (ignores translation)
    #[inline]
    pub fn transform_vector(&self, vector: &Vector2) -> Vector2 {
        let result = self.matrix * Vector3::new(vector.x, vector.y, 0.0);
        Vector2::new(result.x, result.y)
    }

    /// Get the inverse of this transform
    #[inline]
    pub fn inverse(&self) -> Option<Self> {
        self.matrix.try_inverse().map(|matrix| Self { matrix })
    }

    /// Append another transform (multiply matrices)
    #[inline]
    pub fn then(&self, other: &Transform2D) -> Self {
        Self {
            matrix: other.matrix * self.matrix,
        }
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::identity()
    }
}

impl Mul for Transform2D {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        self.then(&rhs)
    }
}

impl ApproxEq for Transform2D {
    fn approx_eq(&self, other: &Self) -> bool {
        self.approx_eq_eps(other, EPSILON)
    }

    fn approx_eq_eps(&self, other: &Self, epsilon: f64) -> bool {
        for i in 0..3 {
            for j in 0..3 {
                if !self.matrix[(i, j)].approx_eq_eps(&other.matrix[(i, j)], epsilon) {
                    return false;
                }
            }
        }
        true
    }

    fn approx_zero(&self) -> bool {
        self.approx_zero_eps(EPSILON)
    }

    fn approx_zero_eps(&self, epsilon: f64) -> bool {
        for i in 0..3 {
            for j in 0..3 {
                if !self.matrix[(i, j)].approx_zero_eps(epsilon) {
                    return false;
                }
            }
        }
        true
    }
}

// ============================================================================
// 3D Transformation
// ============================================================================

/// 3D transformation matrix
///
/// Represents translation, rotation, and scaling in 3D space.
/// Uses homogeneous coordinates internally (4x4 matrix).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform3D {
    /// Internal 4x4 transformation matrix
    pub matrix: Matrix4,
}

impl Transform3D {
    /// Create identity transform
    #[inline]
    pub fn identity() -> Self {
        Self {
            matrix: Matrix4::identity(),
        }
    }

    /// Create translation transform
    #[inline]
    pub fn translation(x: f64, y: f64, z: f64) -> Self {
        Self {
            matrix: Matrix4::new_translation(&Vector3::new(x, y, z)),
        }
    }

    /// Create translation from vector
    #[inline]
    pub fn translation_vec(v: &Vector3) -> Self {
        Self::translation(v.x, v.y, v.z)
    }

    /// Create rotation transform from axis and angle
    #[inline]
    pub fn rotation(axis: &Vector3, angle: f64) -> Self {
        let axis_normalized = axis.normalize();
        Self {
            matrix: Matrix4::from_axis_angle(&nalgebra::Unit::new_unchecked(axis_normalized), angle),
        }
    }

    /// Create rotation from quaternion
    #[inline]
    pub fn from_quaternion(quat: &UnitQuaternion<f64>) -> Self {
        Self {
            matrix: quat.to_homogeneous(),
        }
    }

    /// Create rotation around X axis
    #[inline]
    pub fn rotation_x(angle: f64) -> Self {
        Self {
            matrix: Matrix4::from_axis_angle(&Vector3::x_axis(), angle),
        }
    }

    /// Create rotation around Y axis
    #[inline]
    pub fn rotation_y(angle: f64) -> Self {
        Self {
            matrix: Matrix4::from_axis_angle(&Vector3::y_axis(), angle),
        }
    }

    /// Create rotation around Z axis
    #[inline]
    pub fn rotation_z(angle: f64) -> Self {
        Self {
            matrix: Matrix4::from_axis_angle(&Vector3::z_axis(), angle),
        }
    }

    /// Create Euler angle rotation (XYZ order)
    #[inline]
    pub fn from_euler_angles(roll: f64, pitch: f64, yaw: f64) -> Self {
        let rotation = UnitQuaternion::from_euler_angles(roll, pitch, yaw);
        Self::from_quaternion(&rotation)
    }

    /// Create uniform scale transform
    #[inline]
    pub fn scale(scale: f64) -> Self {
        Self::scale_non_uniform(scale, scale, scale)
    }

    /// Create non-uniform scale transform
    #[inline]
    pub fn scale_non_uniform(sx: f64, sy: f64, sz: f64) -> Self {
        Self {
            matrix: Matrix4::new_nonuniform_scaling(&Vector3::new(sx, sy, sz)),
        }
    }

    /// Transform a point (applies translation)
    #[inline]
    pub fn transform_point(&self, point: &Vector3) -> Vector3 {
        let result = self.matrix * Vector4::new(point.x, point.y, point.z, 1.0);
        Vector3::new(result.x, result.y, result.z)
    }

    /// Transform a vector (ignores translation)
    #[inline]
    pub fn transform_vector(&self, vector: &Vector3) -> Vector3 {
        let result = self.matrix * Vector4::new(vector.x, vector.y, vector.z, 0.0);
        Vector3::new(result.x, result.y, result.z)
    }

    /// Get the inverse of this transform
    #[inline]
    pub fn inverse(&self) -> Option<Self> {
        self.matrix.try_inverse().map(|matrix| Self { matrix })
    }

    /// Append another transform (multiply matrices)
    #[inline]
    pub fn then(&self, other: &Transform3D) -> Self {
        Self {
            matrix: other.matrix * self.matrix,
        }
    }

    /// Extract the translation component
    #[inline]
    pub fn translation_component(&self) -> Vector3 {
        Vector3::new(
            self.matrix[(0, 3)],
            self.matrix[(1, 3)],
            self.matrix[(2, 3)],
        )
    }

    /// Create a look-at view matrix
    #[inline]
    pub fn look_at(eye: &Vector3, target: &Vector3, up: &Vector3) -> Self {
        Self {
            matrix: Matrix4::look_at_rh(
                &nalgebra::Point3::from(*eye),
                &nalgebra::Point3::from(*target),
                up,
            ),
        }
    }

    /// Create a perspective projection matrix
    #[inline]
    pub fn perspective(fov_y: f64, aspect: f64, near: f64, far: f64) -> Self {
        Self {
            matrix: Matrix4::new_perspective(aspect, fov_y, near, far),
        }
    }

    /// Create an orthographic projection matrix
    #[inline]
    pub fn orthographic(left: f64, right: f64, bottom: f64, top: f64, near: f64, far: f64) -> Self {
        Self {
            matrix: Matrix4::new_orthographic(left, right, bottom, top, near, far),
        }
    }
}

impl Default for Transform3D {
    fn default() -> Self {
        Self::identity()
    }
}

impl Mul for Transform3D {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        self.then(&rhs)
    }
}

impl ApproxEq for Transform3D {
    fn approx_eq(&self, other: &Self) -> bool {
        self.approx_eq_eps(other, EPSILON)
    }

    fn approx_eq_eps(&self, other: &Self, epsilon: f64) -> bool {
        for i in 0..4 {
            for j in 0..4 {
                if !self.matrix[(i, j)].approx_eq_eps(&other.matrix[(i, j)], epsilon) {
                    return false;
                }
            }
        }
        true
    }

    fn approx_zero(&self) -> bool {
        self.approx_zero_eps(EPSILON)
    }

    fn approx_zero_eps(&self, epsilon: f64) -> bool {
        for i in 0..4 {
            for j in 0..4 {
                if !self.matrix[(i, j)].approx_zero_eps(epsilon) {
                    return false;
                }
            }
        }
        true
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Calculate the cross product of two 2D vectors (returns scalar z-component)
#[inline]
pub fn cross_2d(a: &Vector2, b: &Vector2) -> f64 {
    a.x * b.y - a.y * b.x
}

/// Calculate the angle between two 2D vectors (in radians)
#[inline]
pub fn angle_between_2d(a: &Vector2, b: &Vector2) -> f64 {
    let dot = a.dot(b);
    let cross = cross_2d(a, b);
    cross.atan2(dot)
}

/// Calculate the angle between two 3D vectors (in radians)
#[inline]
pub fn angle_between_3d(a: &Vector3, b: &Vector3) -> f64 {
    let dot = a.dot(b);
    let magnitude_product = a.norm() * b.norm();
    if magnitude_product < EPSILON {
        0.0
    } else {
        (dot / magnitude_product).clamp(-1.0, 1.0).acos()
    }
}

/// Linear interpolation between two vectors
#[inline]
pub fn lerp_vec2(a: &Vector2, b: &Vector2, t: f64) -> Vector2 {
    a + (b - a) * t
}

/// Linear interpolation between two 3D vectors
#[inline]
pub fn lerp_vec3(a: &Vector3, b: &Vector3, t: f64) -> Vector3 {
    a + (b - a) * t
}

/// Spherical linear interpolation between two unit quaternions
#[inline]
pub fn slerp_quat(a: &UnitQuaternion<f64>, b: &UnitQuaternion<f64>, t: f64) -> UnitQuaternion<f64> {
    a.slerp(b, t)
}

/// Project vector a onto vector b
#[inline]
pub fn project_vec3(a: &Vector3, b: &Vector3) -> Vector3 {
    let b_normalized = b.normalize();
    b_normalized * a.dot(&b_normalized)
}

/// Reflect vector v across normal n (n must be normalized)
#[inline]
pub fn reflect_vec3(v: &Vector3, n: &Vector3) -> Vector3 {
    v - 2.0 * v.dot(n) * n
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_transform2d_identity() {
        let transform = Transform2D::identity();
        let point = Vector2::new(1.0, 2.0);
        let transformed = transform.transform_point(&point);
        assert!(transformed.approx_eq(&point));
    }

    #[test]
    fn test_transform2d_translation() {
        let transform = Transform2D::translation(10.0, 20.0);
        let point = Vector2::new(1.0, 2.0);
        let transformed = transform.transform_point(&point);
        assert!(transformed.approx_eq(&Vector2::new(11.0, 22.0)));
    }

    #[test]
    fn test_transform2d_rotation() {
        let transform = Transform2D::rotation(PI / 2.0);
        let point = Vector2::new(1.0, 0.0);
        let transformed = transform.transform_point(&point);
        assert!(transformed.approx_eq_eps(&Vector2::new(0.0, 1.0), 1e-10));
    }

    #[test]
    fn test_transform2d_scale() {
        let transform = Transform2D::scale(2.0);
        let point = Vector2::new(1.0, 2.0);
        let transformed = transform.transform_point(&point);
        assert!(transformed.approx_eq(&Vector2::new(2.0, 4.0)));
    }

    #[test]
    fn test_transform3d_identity() {
        let transform = Transform3D::identity();
        let point = Vector3::new(1.0, 2.0, 3.0);
        let transformed = transform.transform_point(&point);
        assert!(transformed.approx_eq(&point));
    }

    #[test]
    fn test_transform3d_translation() {
        let transform = Transform3D::translation(10.0, 20.0, 30.0);
        let point = Vector3::new(1.0, 2.0, 3.0);
        let transformed = transform.transform_point(&point);
        assert!(transformed.approx_eq(&Vector3::new(11.0, 22.0, 33.0)));
    }

    #[test]
    fn test_transform3d_rotation_z() {
        let transform = Transform3D::rotation_z(PI / 2.0);
        let point = Vector3::new(1.0, 0.0, 0.0);
        let transformed = transform.transform_point(&point);
        assert!(transformed.approx_eq_eps(&Vector3::new(0.0, 1.0, 0.0), 1e-10));
    }

    #[test]
    fn test_cross_2d() {
        let a = Vector2::new(1.0, 0.0);
        let b = Vector2::new(0.0, 1.0);
        assert!(cross_2d(&a, &b).approx_eq(&1.0));
    }

    #[test]
    fn test_angle_between_3d() {
        let a = Vector3::new(1.0, 0.0, 0.0);
        let b = Vector3::new(0.0, 1.0, 0.0);
        let angle = angle_between_3d(&a, &b);
        assert!(angle.approx_eq_eps(&(PI / 2.0), 1e-10));
    }

    #[test]
    fn test_lerp_vec3() {
        let a = Vector3::new(0.0, 0.0, 0.0);
        let b = Vector3::new(10.0, 10.0, 10.0);
        let mid = lerp_vec3(&a, &b, 0.5);
        assert!(mid.approx_eq(&Vector3::new(5.0, 5.0, 5.0)));
    }

    #[test]
    fn test_project_vec3() {
        let a = Vector3::new(1.0, 1.0, 0.0);
        let b = Vector3::new(1.0, 0.0, 0.0);
        let projected = project_vec3(&a, &b);
        assert!(projected.approx_eq(&Vector3::new(1.0, 0.0, 0.0)));
    }
}
