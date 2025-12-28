//! Precision handling for CAD operations
//!
//! This module provides epsilon constants and traits for floating-point
//! comparison with configurable tolerance levels, essential for robust
//! CAD geometry operations.

use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// Rough precision - for coarse comparisons (1e-6)
pub const EPSILON_ROUGH: f64 = 1e-6;

/// Normal precision - standard CAD operations (1e-9)
pub const EPSILON_NORMAL: f64 = 1e-9;

/// Fine precision - high-precision operations (1e-12)
pub const EPSILON_FINE: f64 = 1e-12;

/// Default epsilon value (normal precision)
pub const EPSILON: f64 = EPSILON_NORMAL;

/// Two times PI
pub const TAU: f64 = 2.0 * PI;

/// Trait for approximate equality comparison with tolerance
///
/// This trait should be implemented for all floating-point types
/// used in CAD operations to ensure robust comparisons.
pub trait ApproxEq {
    /// Check if two values are approximately equal using default epsilon
    fn approx_eq(&self, other: &Self) -> bool;

    /// Check if two values are approximately equal using custom epsilon
    fn approx_eq_eps(&self, other: &Self, epsilon: f64) -> bool;

    /// Check if value is approximately zero using default epsilon
    fn approx_zero(&self) -> bool;

    /// Check if value is approximately zero using custom epsilon
    fn approx_zero_eps(&self, epsilon: f64) -> bool;
}

impl ApproxEq for f64 {
    #[inline]
    fn approx_eq(&self, other: &Self) -> bool {
        (self - other).abs() < EPSILON
    }

    #[inline]
    fn approx_eq_eps(&self, other: &Self, epsilon: f64) -> bool {
        (self - other).abs() < epsilon
    }

    #[inline]
    fn approx_zero(&self) -> bool {
        self.abs() < EPSILON
    }

    #[inline]
    fn approx_zero_eps(&self, epsilon: f64) -> bool {
        self.abs() < epsilon
    }
}

impl ApproxEq for f32 {
    #[inline]
    fn approx_eq(&self, other: &Self) -> bool {
        (self - other).abs() < EPSILON as f32
    }

    #[inline]
    fn approx_eq_eps(&self, other: &Self, epsilon: f64) -> bool {
        (self - other).abs() < epsilon as f32
    }

    #[inline]
    fn approx_zero(&self) -> bool {
        self.abs() < EPSILON as f32
    }

    #[inline]
    fn approx_zero_eps(&self, epsilon: f64) -> bool {
        self.abs() < epsilon as f32
    }
}

/// Normalize an angle to the range [0, 2π)
#[inline]
pub fn normalize_angle(angle: f64) -> f64 {
    let mut normalized = angle % TAU;
    if normalized < 0.0 {
        normalized += TAU;
    }
    normalized
}

/// Normalize an angle to the range [-π, π)
#[inline]
pub fn normalize_angle_signed(angle: f64) -> f64 {
    let mut normalized = angle % TAU;
    if normalized >= PI {
        normalized -= TAU;
    } else if normalized < -PI {
        normalized += TAU;
    }
    normalized
}

/// Clamp a value between min and max
#[inline]
pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Linear interpolation between two values
#[inline]
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Inverse linear interpolation - find t where lerp(a, b, t) = value
#[inline]
pub fn inverse_lerp(a: f64, b: f64, value: f64) -> f64 {
    if (b - a).abs() < EPSILON {
        0.0
    } else {
        (value - a) / (b - a)
    }
}

/// Remap a value from one range to another
#[inline]
pub fn remap(value: f64, from_min: f64, from_max: f64, to_min: f64, to_max: f64) -> f64 {
    let t = inverse_lerp(from_min, from_max, value);
    lerp(to_min, to_max, t)
}

/// Smoothstep interpolation (cubic Hermite)
#[inline]
pub fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Tolerance specification for geometric operations
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Tolerance {
    /// Distance tolerance
    pub distance: f64,
    /// Angular tolerance in radians
    pub angle: f64,
}

impl Tolerance {
    /// Create a new tolerance with custom values
    pub fn new(distance: f64, angle: f64) -> Self {
        Self { distance, angle }
    }

    /// Rough tolerance preset
    pub fn rough() -> Self {
        Self {
            distance: EPSILON_ROUGH,
            angle: 1e-4, // ~0.0057 degrees
        }
    }

    /// Normal tolerance preset (default)
    pub fn normal() -> Self {
        Self {
            distance: EPSILON_NORMAL,
            angle: 1e-7, // ~0.0000057 degrees
        }
    }

    /// Fine tolerance preset
    pub fn fine() -> Self {
        Self {
            distance: EPSILON_FINE,
            angle: 1e-10,
        }
    }
}

impl Default for Tolerance {
    fn default() -> Self {
        Self::normal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approx_eq_f64() {
        assert!(1.0.approx_eq(&1.0));
        assert!(1.0.approx_eq(&(1.0 + EPSILON / 2.0)));
        assert!(!1.0.approx_eq(&(1.0 + EPSILON * 2.0)));
    }

    #[test]
    fn test_approx_zero() {
        assert!(0.0.approx_zero());
        assert!((EPSILON / 2.0).approx_zero());
        assert!(!(EPSILON * 2.0).approx_zero());
    }

    #[test]
    fn test_normalize_angle() {
        assert!(normalize_angle(0.0).approx_eq(&0.0));
        assert!(normalize_angle(TAU).approx_eq(&0.0));
        assert!(normalize_angle(-PI).approx_eq(&PI));
        assert!(normalize_angle(PI).approx_eq(&PI));
    }

    #[test]
    fn test_normalize_angle_signed() {
        assert!(normalize_angle_signed(0.0).approx_eq(&0.0));
        assert!(normalize_angle_signed(PI).approx_eq(&-PI));
        assert!(normalize_angle_signed(-PI).approx_eq(&-PI));
        assert!(normalize_angle_signed(TAU).approx_eq(&0.0));
    }

    #[test]
    fn test_lerp() {
        assert!(lerp(0.0, 10.0, 0.0).approx_eq(&0.0));
        assert!(lerp(0.0, 10.0, 0.5).approx_eq(&5.0));
        assert!(lerp(0.0, 10.0, 1.0).approx_eq(&10.0));
    }

    #[test]
    fn test_inverse_lerp() {
        assert!(inverse_lerp(0.0, 10.0, 0.0).approx_eq(&0.0));
        assert!(inverse_lerp(0.0, 10.0, 5.0).approx_eq(&0.5));
        assert!(inverse_lerp(0.0, 10.0, 10.0).approx_eq(&1.0));
    }

    #[test]
    fn test_remap() {
        assert!(remap(5.0, 0.0, 10.0, 0.0, 100.0).approx_eq(&50.0));
        assert!(remap(0.0, -1.0, 1.0, 0.0, 10.0).approx_eq(&5.0));
    }
}
