//! Constraint solver system
//!
//! Provides constraint solving using iterative methods (Newton-Raphson),
//! degree of freedom analysis, and over/under-constrained detection.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::geometric::{GeometricConstraint, GeometricConstraintType};
use super::dimensional::{DimensionalConstraint, ConstraintEquation, ConstraintMode};

/// Solver status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SolverStatus {
    /// Solver has not been run
    NotSolved,
    /// Solve succeeded - all constraints satisfied
    Solved,
    /// Solve partially succeeded - some constraints satisfied
    PartiallySolved,
    /// Solve failed - constraints could not be satisfied
    Failed,
    /// System is over-constrained
    OverConstrained,
    /// System is under-constrained
    UnderConstrained,
    /// Solver reached maximum iterations
    MaxIterationsReached,
}

/// Degree of freedom information for an entity
#[derive(Debug, Clone, PartialEq)]
pub struct EntityDOF {
    /// Entity ID
    pub entity_id: Uuid,
    /// Total degrees of freedom
    pub total_dof: usize,
    /// Constrained degrees of freedom
    pub constrained_dof: usize,
    /// Free degrees of freedom
    pub free_dof: usize,
    /// Is entity fully constrained
    pub fully_constrained: bool,
    /// Is entity over-constrained
    pub over_constrained: bool,
}

impl EntityDOF {
    /// Create DOF info for an entity
    pub fn new(entity_id: Uuid, total_dof: usize) -> Self {
        EntityDOF {
            entity_id,
            total_dof,
            constrained_dof: 0,
            free_dof: total_dof,
            fully_constrained: false,
            over_constrained: false,
        }
    }

    /// Add a constraint
    pub fn add_constraint(&mut self, dof_count: usize) {
        self.constrained_dof += dof_count;
        self.free_dof = self.total_dof.saturating_sub(self.constrained_dof);
        self.fully_constrained = self.free_dof == 0;
        self.over_constrained = self.constrained_dof > self.total_dof;
    }
}

/// Solver configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolverConfig {
    /// Maximum number of iterations
    pub max_iterations: usize,
    /// Convergence tolerance
    pub tolerance: f64,
    /// Relaxation factor (0.0 to 1.0)
    pub relaxation: f64,
    /// Enable rigid body suppression
    pub suppress_rigid_body: bool,
    /// Maximum step size for movement
    pub max_step_size: f64,
    /// Minimum improvement required to continue
    pub min_improvement: f64,
}

impl Default for SolverConfig {
    fn default() -> Self {
        SolverConfig {
            max_iterations: 100,
            tolerance: 1e-6,
            relaxation: 0.5,
            suppress_rigid_body: true,
            max_step_size: 10.0,
            min_improvement: 1e-8,
        }
    }
}

/// Constraint solver
#[derive(Debug, Clone)]
pub struct ConstraintSolver {
    /// Geometric constraints
    geometric_constraints: Vec<GeometricConstraint>,
    /// Dimensional constraints
    dimensional_constraints: Vec<DimensionalConstraint>,
    /// Solver configuration
    config: SolverConfig,
    /// Current solver status
    status: SolverStatus,
    /// Number of iterations used
    iterations_used: usize,
    /// Final residual error
    final_error: f64,
    /// Entity DOF information
    entity_dof: HashMap<Uuid, EntityDOF>,
}

impl ConstraintSolver {
    /// Create a new constraint solver
    pub fn new() -> Self {
        ConstraintSolver {
            geometric_constraints: Vec::new(),
            dimensional_constraints: Vec::new(),
            config: SolverConfig::default(),
            status: SolverStatus::NotSolved,
            iterations_used: 0,
            final_error: 0.0,
            entity_dof: HashMap::new(),
        }
    }

    /// Create solver with custom configuration
    pub fn with_config(config: SolverConfig) -> Self {
        ConstraintSolver {
            geometric_constraints: Vec::new(),
            dimensional_constraints: Vec::new(),
            config,
            status: SolverStatus::NotSolved,
            iterations_used: 0,
            final_error: 0.0,
            entity_dof: HashMap::new(),
        }
    }

    /// Add a geometric constraint
    pub fn add_geometric_constraint(&mut self, constraint: GeometricConstraint) {
        self.geometric_constraints.push(constraint);
        self.status = SolverStatus::NotSolved;
    }

    /// Add a dimensional constraint
    pub fn add_dimensional_constraint(&mut self, constraint: DimensionalConstraint) {
        self.dimensional_constraints.push(constraint);
        self.status = SolverStatus::NotSolved;
    }

    /// Remove a constraint by ID
    pub fn remove_constraint(&mut self, constraint_id: Uuid) {
        self.geometric_constraints.retain(|c| c.id != constraint_id);
        self.dimensional_constraints.retain(|c| c.id != constraint_id);
        self.status = SolverStatus::NotSolved;
    }

    /// Clear all constraints
    pub fn clear_constraints(&mut self) {
        self.geometric_constraints.clear();
        self.dimensional_constraints.clear();
        self.entity_dof.clear();
        self.status = SolverStatus::NotSolved;
    }

    /// Get solver status
    pub fn status(&self) -> SolverStatus {
        self.status
    }

    /// Get number of iterations used
    pub fn iterations_used(&self) -> usize {
        self.iterations_used
    }

    /// Get final error
    pub fn final_error(&self) -> f64 {
        self.final_error
    }

    /// Analyze degrees of freedom
    pub fn analyze_dof(&mut self) -> HashMap<Uuid, EntityDOF> {
        self.entity_dof.clear();

        // Collect all entities
        let mut entities = HashSet::new();
        for constraint in &self.geometric_constraints {
            for entity_ref in &constraint.entities {
                entities.insert(entity_ref.entity_id());
            }
        }
        for constraint in &self.dimensional_constraints {
            for entity_ref in &constraint.entities {
                entities.insert(entity_ref.entity_id());
            }
        }

        // Initialize DOF for each entity (simplified - assumes 2D points)
        for entity_id in entities {
            self.entity_dof.insert(entity_id, EntityDOF::new(entity_id, 2));
        }

        // Count constraints for each entity
        for constraint in &self.geometric_constraints {
            let dof_removed = self.geometric_constraint_dof(&constraint.constraint_type);
            for entity_ref in &constraint.entities {
                if let Some(dof) = self.entity_dof.get_mut(&entity_ref.entity_id()) {
                    dof.add_constraint(dof_removed);
                }
            }
        }

        for constraint in &self.dimensional_constraints {
            let dof_removed = 1; // Each dimensional constraint removes 1 DOF
            for entity_ref in &constraint.entities {
                if let Some(dof) = self.entity_dof.get_mut(&entity_ref.entity_id()) {
                    dof.add_constraint(dof_removed);
                }
            }
        }

        self.entity_dof.clone()
    }

    /// Get DOF removed by a geometric constraint type
    fn geometric_constraint_dof(&self, constraint_type: &GeometricConstraintType) -> usize {
        match constraint_type {
            GeometricConstraintType::Coincident => 2, // Removes 2 DOF (x and y)
            GeometricConstraintType::Parallel => 1,    // Removes 1 DOF (angle)
            GeometricConstraintType::Perpendicular => 1,
            GeometricConstraintType::Horizontal => 1,  // Removes 1 DOF (y angle)
            GeometricConstraintType::Vertical => 1,    // Removes 1 DOF (x angle)
            GeometricConstraintType::Tangent => 1,
            GeometricConstraintType::Concentric => 2,  // Removes 2 DOF (x and y of center)
            GeometricConstraintType::Equal => 1,       // Removes 1 DOF (size)
            GeometricConstraintType::Fixed => 2,       // Fixes all DOF
            GeometricConstraintType::Collinear => 1,
            GeometricConstraintType::Colinear => 1,
            GeometricConstraintType::PointOnCurve => 1,
            GeometricConstraintType::Midpoint => 2,
            GeometricConstraintType::Symmetric => 2,
            GeometricConstraintType::Smooth => 1,
        }
    }

    /// Check if system is well-constrained
    pub fn is_well_constrained(&mut self) -> bool {
        let dof = self.analyze_dof();

        // Check if any entity is over or under constrained
        for entity_dof in dof.values() {
            if entity_dof.over_constrained {
                self.status = SolverStatus::OverConstrained;
                return false;
            }
            // Allow some under-constraint (for dragging, etc.)
        }

        true
    }

    /// Solve constraints using Newton-Raphson iteration
    pub fn solve(&mut self) -> SolverStatus {
        // Check if well-constrained
        if !self.is_well_constrained() {
            return self.status;
        }

        self.iterations_used = 0;
        self.final_error = f64::MAX;

        // Build constraint equations
        let mut equations = self.build_equations();

        // Newton-Raphson iteration
        for iteration in 0..self.config.max_iterations {
            self.iterations_used = iteration + 1;

            // Evaluate current error
            let current_error = self.evaluate_error(&equations);

            // Check for convergence
            if current_error < self.config.tolerance {
                self.status = SolverStatus::Solved;
                self.final_error = current_error;
                return self.status;
            }

            // Check for improvement
            if iteration > 0 {
                let improvement = self.final_error - current_error;
                if improvement < self.config.min_improvement {
                    self.status = SolverStatus::PartiallySolved;
                    self.final_error = current_error;
                    return self.status;
                }
            }

            self.final_error = current_error;

            // Compute Jacobian and solve
            self.newton_raphson_step(&mut equations);
        }

        // Max iterations reached
        self.status = SolverStatus::MaxIterationsReached;
        self.status
    }

    /// Build constraint equations
    fn build_equations(&self) -> Vec<ConstraintEquation> {
        let mut equations = Vec::new();

        // Add geometric constraint equations
        for constraint in &self.geometric_constraints {
            if !constraint.enabled {
                continue;
            }

            let mut eq = ConstraintEquation::new(constraint.id);
            // In real implementation, would set up variables and compute residual
            equations.push(eq);
        }

        // Add dimensional constraint equations (only driving constraints)
        for constraint in &self.dimensional_constraints {
            if !constraint.enabled || constraint.mode == ConstraintMode::Driven {
                continue;
            }

            let mut eq = ConstraintEquation::new(constraint.id);
            // In real implementation, would set up variables and compute residual
            equations.push(eq);
        }

        equations
    }

    /// Evaluate total error
    fn evaluate_error(&self, equations: &[ConstraintEquation]) -> f64 {
        equations
            .iter()
            .map(|eq| eq.residual * eq.residual * eq.weight)
            .sum::<f64>()
            .sqrt()
    }

    /// Perform one Newton-Raphson iteration step
    fn newton_raphson_step(&self, _equations: &mut [ConstraintEquation]) {
        // In a real implementation, this would:
        // 1. Compute the Jacobian matrix (partial derivatives)
        // 2. Solve the linear system J * delta = -residual
        // 3. Update variables with relaxation factor
        // 4. Clamp updates to max_step_size
        // 5. Update residuals

        // This is a placeholder - real implementation would use
        // sparse matrix solver or iterative method
    }

    /// Get conflicting constraints
    pub fn find_conflicts(&self) -> Vec<(Uuid, Uuid)> {
        let mut conflicts = Vec::new();

        // Simple conflict detection - checks for contradictory constraints
        // In real implementation, would do more sophisticated analysis

        for i in 0..self.geometric_constraints.len() {
            for j in (i + 1)..self.geometric_constraints.len() {
                let c1 = &self.geometric_constraints[i];
                let c2 = &self.geometric_constraints[j];

                if self.are_conflicting(c1, c2) {
                    conflicts.push((c1.id, c2.id));
                }
            }
        }

        conflicts
    }

    /// Check if two geometric constraints conflict
    fn are_conflicting(&self, c1: &GeometricConstraint, c2: &GeometricConstraint) -> bool {
        // Check if constraints share entities
        let c1_entities: HashSet<_> = c1.entities.iter().map(|e| e.entity_id()).collect();
        let c2_entities: HashSet<_> = c2.entities.iter().map(|e| e.entity_id()).collect();

        if c1_entities.is_disjoint(&c2_entities) {
            return false;
        }

        // Check for contradictory constraints
        use GeometricConstraintType::*;
        match (&c1.constraint_type, &c2.constraint_type) {
            (Horizontal, Vertical) => true,
            (Vertical, Horizontal) => true,
            (Fixed, _) | (_, Fixed) => {
                // Fixed conflicts with any other constraint on the same entity
                !c1_entities.is_disjoint(&c2_entities)
            }
            _ => false,
        }
    }

    /// Get diagnostic information
    pub fn diagnostics(&self) -> SolverDiagnostics {
        SolverDiagnostics {
            status: self.status,
            iterations: self.iterations_used,
            final_error: self.final_error,
            geometric_constraint_count: self.geometric_constraints.len(),
            dimensional_constraint_count: self.dimensional_constraints.len(),
            entity_count: self.entity_dof.len(),
            over_constrained_entities: self
                .entity_dof
                .values()
                .filter(|d| d.over_constrained)
                .count(),
            under_constrained_entities: self
                .entity_dof
                .values()
                .filter(|d| d.free_dof > 0 && !d.fully_constrained)
                .count(),
        }
    }

    /// Reset solver state
    pub fn reset(&mut self) {
        self.status = SolverStatus::NotSolved;
        self.iterations_used = 0;
        self.final_error = 0.0;
    }
}

impl Default for ConstraintSolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Solver diagnostic information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolverDiagnostics {
    /// Solver status
    pub status: SolverStatus,
    /// Number of iterations
    pub iterations: usize,
    /// Final error
    pub final_error: f64,
    /// Number of geometric constraints
    pub geometric_constraint_count: usize,
    /// Number of dimensional constraints
    pub dimensional_constraint_count: usize,
    /// Number of entities
    pub entity_count: usize,
    /// Number of over-constrained entities
    pub over_constrained_entities: usize,
    /// Number of under-constrained entities
    pub under_constrained_entities: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solver_creation() {
        let solver = ConstraintSolver::new();
        assert_eq!(solver.status(), SolverStatus::NotSolved);
        assert_eq!(solver.iterations_used(), 0);
    }

    #[test]
    fn test_add_constraints() {
        let mut solver = ConstraintSolver::new();

        let p1 = EntityReference::Point(Uuid::new_v4());
        let p2 = EntityReference::Point(Uuid::new_v4());

        let geom_constraint = GeometricConstraint::coincident(p1.clone(), p2.clone());
        solver.add_geometric_constraint(geom_constraint);

        let dim_constraint = DimensionalConstraint::distance(p1, p2, 50.0);
        solver.add_dimensional_constraint(dim_constraint);

        let diag = solver.diagnostics();
        assert_eq!(diag.geometric_constraint_count, 1);
        assert_eq!(diag.dimensional_constraint_count, 1);
    }

    #[test]
    fn test_dof_analysis() {
        let mut solver = ConstraintSolver::new();

        let p1 = EntityReference::Point(Uuid::new_v4());
        let p2 = EntityReference::Point(Uuid::new_v4());

        // Add a distance constraint
        let constraint = DimensionalConstraint::distance(p1, p2, 50.0);
        solver.add_dimensional_constraint(constraint);

        let dof_map = solver.analyze_dof();
        assert_eq!(dof_map.len(), 2); // Two points
    }

    #[test]
    fn test_entity_dof() {
        let entity_id = Uuid::new_v4();
        let mut dof = EntityDOF::new(entity_id, 2);

        assert_eq!(dof.total_dof, 2);
        assert_eq!(dof.free_dof, 2);
        assert!(!dof.fully_constrained);
        assert!(!dof.over_constrained);

        dof.add_constraint(1);
        assert_eq!(dof.free_dof, 1);
        assert!(!dof.fully_constrained);

        dof.add_constraint(1);
        assert_eq!(dof.free_dof, 0);
        assert!(dof.fully_constrained);

        dof.add_constraint(1);
        assert!(dof.over_constrained);
    }

    #[test]
    fn test_conflicting_constraints() {
        let mut solver = ConstraintSolver::new();

        let line = EntityReference::Line(Uuid::new_v4());

        let h_constraint = GeometricConstraint::horizontal(line.clone());
        let v_constraint = GeometricConstraint::vertical(line);

        solver.add_geometric_constraint(h_constraint);
        solver.add_geometric_constraint(v_constraint);

        let conflicts = solver.find_conflicts();
        assert_eq!(conflicts.len(), 1);
    }

    #[test]
    fn test_solver_config() {
        let config = SolverConfig {
            max_iterations: 50,
            tolerance: 1e-4,
            relaxation: 0.7,
            ..Default::default()
        };

        let solver = ConstraintSolver::with_config(config.clone());
        assert_eq!(solver.config.max_iterations, 50);
        assert_eq!(solver.config.tolerance, 1e-4);
    }

    #[test]
    fn test_solver_diagnostics() {
        let mut solver = ConstraintSolver::new();

        let p1 = EntityReference::Point(Uuid::new_v4());
        let p2 = EntityReference::Point(Uuid::new_v4());

        solver.add_geometric_constraint(GeometricConstraint::coincident(p1.clone(), p2.clone()));
        solver.add_dimensional_constraint(DimensionalConstraint::distance(p1, p2, 50.0));

        solver.analyze_dof();

        let diag = solver.diagnostics();
        assert_eq!(diag.geometric_constraint_count, 1);
        assert_eq!(diag.dimensional_constraint_count, 1);
        assert_eq!(diag.status, SolverStatus::NotSolved);
    }
}
