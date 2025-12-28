//! Core module - foundational math, primitives, and precision handling
//!
//! This module provides the mathematical foundation for the entire CAD system,
//! including vector/matrix operations, geometric primitives, precision handling,
//! and color types.

pub mod color;
pub mod math;
pub mod precision;
pub mod primitives;

// Re-export commonly used types
pub use color::Color;
pub use math::{Matrix3, Matrix4, Quaternion, Transform2D, Transform3D, Vector2, Vector3, Vector4};
pub use precision::{ApproxEq, EPSILON, EPSILON_FINE, EPSILON_NORMAL, EPSILON_ROUGH};
pub use primitives::{
    BoundingBox2, BoundingBox3, EntityId, Plane, Point2, Point3, Ray2, Ray3,
};
