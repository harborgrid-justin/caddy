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
//! - `compression`: Enterprise-grade compression algorithms
//! - `analytics`: Analytics and telemetry system
//! - `database`: Enterprise database layer with caching, replication, and sharding
//! - `scheduling`: Job scheduling, monitoring, and notification system
//! - `accessibility`: WCAG 2.1/2.2 accessibility scanning and remediation engine
//! - `saas`: Multi-tenant SaaS infrastructure with billing, subscriptions, and usage tracking
//! - `api`: REST API gateway with circuit breaker, retry logic, and webhooks
//! - `auth`: Enterprise authentication system with SSO, RBAC, MFA, and session management
//! - `collaboration`: Real-time collaboration features
//! - `teams`: Team collaboration system with workspaces, members, assignments, and activity tracking
//! - `integrations`: CI/CD integrations for GitHub, GitLab, Jenkins, Azure DevOps, Bitbucket
//! - `ai`: AI/ML engine with computer vision, NLP, predictions, and auto-suggestions

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

// Viewport rendering system
pub mod viewport;

// Compression system
pub mod compression;

// Analytics and telemetry
pub mod analytics;

// Database layer
pub mod database;

// 3D modeling engine
pub mod engine3d;

// Scheduling and monitoring system
pub mod scheduling;

// Accessibility scanning and remediation
pub mod accessibility;

// SaaS infrastructure
pub mod saas;

// REST API gateway
pub mod api;

// Authentication and authorization
pub mod auth;

// Team collaboration system
pub mod teams;

// CI/CD integrations
pub mod integrations;

// AI/ML engine
pub mod ai;

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
