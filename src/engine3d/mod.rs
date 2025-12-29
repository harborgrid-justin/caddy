//! 3D Modeling Engine Module
//!
//! Enterprise-grade 3D engine for CADDY providing comprehensive
//! mesh operations, NURBS, tessellation, and topological operations.
//!
//! ## Modules
//!
//! - `mesh`: Half-edge mesh data structure for robust topology
//! - `boolean`: CSG boolean operations (union, intersection, difference)
//! - `nurbs`: NURBS curves and surfaces with evaluation
//! - `tessellation`: Adaptive tessellation algorithms
//! - `topology`: Extrude, revolve, sweep, loft operations
//! - `healing`: Mesh repair and healing algorithms
//! - `simplification`: LOD generation and mesh decimation
//! - `analysis`: Geometry analysis and mass properties
//! - `constraints`: Geometric constraint solver
//!
//! ## Example
//!
//! ```rust,no_run
//! use caddy::engine3d::{
//!     mesh::HalfEdgeMesh,
//!     topology::ExtrudeOperation,
//! };
//! use caddy::core::{Point3, Vector3};
//!
//! // Create a square profile
//! let profile = vec![
//!     Point3::new(-1.0, -1.0, 0.0),
//!     Point3::new(1.0, -1.0, 0.0),
//!     Point3::new(1.0, 1.0, 0.0),
//!     Point3::new(-1.0, 1.0, 0.0),
//! ];
//!
//! // Extrude to create a box
//! let extrude = ExtrudeOperation {
//!     direction: Vector3::new(0.0, 0.0, 2.0),
//!     capped: true,
//!     ..Default::default()
//! };
//!
//! let mesh = extrude.extrude_profile(&profile).unwrap();
//! println!("Created mesh with {} vertices", mesh.stats().vertices);
//! ```

pub mod mesh;
pub mod boolean;
pub mod nurbs;
pub mod tessellation;
pub mod topology;
pub mod healing;
pub mod simplification;
pub mod analysis;
pub mod constraints;

// Re-export commonly used types
pub use mesh::{
    HalfEdgeMesh, VertexHandle, EdgeHandle, FaceHandle, HalfEdgeHandle,
    Vertex, Edge, Face, HalfEdge, MeshError, MeshStats,
};

pub use boolean::{
    BooleanOp, Plane, BSPTree, boolean_operation,
};

pub use nurbs::{
    NurbsCurve, NurbsSurface, NurbsError, SurfaceMesh,
};

pub use tessellation::{
    TessellationSettings, AdaptiveTessellator, DelaunayTriangulator,
    TessellationError,
};

pub use topology::{
    ExtrudeOperation, RevolveOperation, SweepOperation,
    LoftOperation, ShellOperation, TopologyError,
};

pub use healing::{
    MeshHealer, HealingReport,
};

pub use simplification::{
    SimplificationSettings, QuadricSimplifier, SimplificationReport,
};

pub use analysis::{
    GeometryAnalyzer, MassProperties, AnalysisError,
};

pub use constraints::{
    Constraint, ConstraintType, ConstraintSolver, SolveResult,
};
