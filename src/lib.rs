//! # CADDY - Enterprise CAD System
//!
//! A professional Computer-Aided Design system built in Rust.
//!
//! ## Architecture
//!
//! - `core`: Foundational math, primitives, and precision handling
//! - `geometry`: 2D and 3D geometric primitives and operations
//! - `rendering`: GPU-accelerated rendering pipeline
//! - `ui`: User interface framework
//! - `io`: File I/O for DXF and native formats
//! - `commands`: Command system with undo/redo support
//! - `layers`: Layer management system
//! - `tools`: Selection and manipulation tools
//! - `dimensions`: Dimensioning and annotations
//! - `constraints`: Parametric constraint solver

#![warn(missing_docs)]
#![warn(clippy::all)]

// Core modules - foundational math and primitives
pub mod core;

// Geometry modules - 2D and 3D primitives
pub mod geometry;

// Rendering system
pub mod rendering;

// User interface
pub mod ui;

// File I/O
pub mod io;

// Command system
pub mod commands;

// Layer management
pub mod layers;

// Tools and utilities
pub mod tools;

// Dimensions and annotations
pub mod dimensions;

// Constraint solver
pub mod constraints;

// Plugin system
pub mod plugins;

// Enterprise features
pub mod enterprise;

// Re-export commonly used types
pub use core::{
    color::Color,
    math::{Matrix3, Matrix4, Quaternion, Transform2D, Transform3D, Vector2, Vector3, Vector4},
    precision::{ApproxEq, EPSILON, EPSILON_FINE, EPSILON_NORMAL, EPSILON_ROUGH},
    primitives::{
        BoundingBox2, BoundingBox3, EntityId, Plane, Point2, Point3, Ray2, Ray3,
    },
};

/// Re-export version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Re-export crate name
pub const NAME: &str = env!("CARGO_PKG_NAME");
