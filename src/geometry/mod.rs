//! Geometry module for CADDY CAD system
//!
//! This module provides comprehensive 2D and 3D geometric primitives and operations
//! optimized for CAD applications, including:
//!
//! ## 2D Geometry
//! - Points with CAD-specific operations
//! - Lines, line segments, and polylines
//! - Arcs, circles, and ellipses
//! - Bezier curves, B-splines, and NURBS
//! - Polygons with advanced algorithms
//!
//! ## 3D Geometry
//! - 3D solid primitives (Box, Sphere, Cylinder, Cone, Torus, Wedge)
//! - Parametric surfaces (Plane, Bezier, B-Spline, NURBS)
//! - Mesh data structures (Triangle, Quad, Half-Edge meshes)
//! - CSG operations (Union, Subtraction, Intersection)
//! - Extrusion operations (Linear, Revolution, Sweep, Loft)

// 2D Geometry modules
pub mod arc;
pub mod curve;
pub mod line;
pub mod point;
pub mod polygon;

// 3D Geometry modules
pub mod solid;
pub mod surface;
pub mod mesh;
pub mod boolean;
pub mod extrude;

// Re-export commonly used 2D types
pub use arc::{Arc2D, Circle2D, Ellipse2D, EllipticalArc2D};
pub use curve::{BezierCurve, BSpline, NurbsCurve};
pub use line::{Line2D, LineSegment2D, Polyline2D};
pub use point::Point2D;
pub use polygon::Polygon2D;

// Re-export commonly used 3D types
pub use solid::{
    BoundingBox, Box3D, Cone3D, Cylinder3D, Solid3D, Sphere3D, Torus3D, Wedge3D,
};

pub use surface::{
    BSplineSurface, BezierSurface, NurbsSurface, ParametricSurface, Plane3D,
    SurfaceCurvature, TrimCurve,
};

pub use mesh::{
    HalfEdge, HEFace, HEVertex, HalfEdgeMesh, MeshSimplifier, QuadFace, QuadMesh,
    TriangleFace, TriangleMesh, Vertex,
};

pub use boolean::{BooleanOperation, CSGNode, CSGOperator, CoplanarHandler};

pub use extrude::{
    LinearExtrude, Loft, Path3D, Profile2D, Revolution, Sweep,
};
