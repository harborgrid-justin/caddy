//! Constraint system for parametric design
//!
//! This module provides comprehensive constraint solving capabilities including:
//! - Geometric constraints (coincident, parallel, perpendicular, etc.)
//! - Dimensional constraints (distance, angle, radius, etc.)
//! - Constraint solver with Newton-Raphson iteration
//! - Degree of freedom analysis
//! - Over/under-constrained detection
//!
//! # Example
//!
//! ```rust,ignore
//! use caddy::constraints::{ConstraintSolver, GeometricConstraint, DimensionalConstraint};
//! use caddy::constraints::geometric::EntityReference;
//! use uuid::Uuid;
//!
//! // Create a solver
//! let mut solver = ConstraintSolver::new();
//!
//! // Add constraints
//! let p1 = EntityReference::Point(Uuid::new_v4());
//! let p2 = EntityReference::Point(Uuid::new_v4());
//!
//! solver.add_dimensional_constraint(
//!     DimensionalConstraint::distance(p1, p2, 50.0)
//! );
//!
//! // Solve
//! let status = solver.solve();
//! ```

pub mod geometric;
pub mod dimensional;
pub mod solver;

// Re-export commonly used types
pub use geometric::{
    GeometricConstraint,
    GeometricConstraintType,
    EntityReference,
    ConstraintGroup,
    ConstraintConflict,
};

pub use dimensional::{
    DimensionalConstraint,
    DimensionalConstraintType,
    ConstraintMode,
    ConstraintEquation,
    ParameterTable,
};

pub use solver::{
    ConstraintSolver,
    SolverStatus,
    SolverConfig,
    SolverDiagnostics,
    EntityDOF,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_system_integration() {
        let mut solver = ConstraintSolver::new();

        // Create some entities
        let p1 = EntityReference::Point(uuid::Uuid::new_v4());
        let p2 = EntityReference::Point(uuid::Uuid::new_v4());
        let line = EntityReference::Line(uuid::Uuid::new_v4());

        // Add geometric constraints
        solver.add_geometric_constraint(
            GeometricConstraint::horizontal(line.clone())
        );

        // Add dimensional constraints
        solver.add_dimensional_constraint(
            DimensionalConstraint::distance(p1, p2, 100.0)
        );

        // Analyze DOF
        let dof = solver.analyze_dof();
        assert!(!dof.is_empty());

        // Get diagnostics
        let diag = solver.diagnostics();
        assert_eq!(diag.geometric_constraint_count, 1);
        assert_eq!(diag.dimensional_constraint_count, 1);
    }
}
