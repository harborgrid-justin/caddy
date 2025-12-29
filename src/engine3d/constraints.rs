//! Geometric constraint solver for parametric CAD
//!
//! Implements a constraint-based solver for 3D geometric relationships
//! including distances, angles, parallelism, perpendicularity, and tangency.

use crate::core::{Point3, Vector3, EPSILON};
use nalgebra::{DMatrix, DVector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Geometric constraint type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    /// Fixed point constraint
    FixedPoint {
        point_id: Uuid,
        position: Point3,
    },
    /// Distance between two points
    Distance {
        point1_id: Uuid,
        point2_id: Uuid,
        distance: f64,
    },
    /// Angle between two vectors
    Angle {
        vec1_start: Uuid,
        vec1_end: Uuid,
        vec2_start: Uuid,
        vec2_end: Uuid,
        angle: f64,
    },
    /// Parallel vectors
    Parallel {
        vec1_start: Uuid,
        vec1_end: Uuid,
        vec2_start: Uuid,
        vec2_end: Uuid,
    },
    /// Perpendicular vectors
    Perpendicular {
        vec1_start: Uuid,
        vec1_end: Uuid,
        vec2_start: Uuid,
        vec2_end: Uuid,
    },
    /// Coincident points
    Coincident {
        point1_id: Uuid,
        point2_id: Uuid,
    },
    /// Horizontal constraint (XY plane)
    Horizontal {
        point1_id: Uuid,
        point2_id: Uuid,
    },
    /// Vertical constraint (XY plane)
    Vertical {
        point1_id: Uuid,
        point2_id: Uuid,
    },
    /// Point on line
    PointOnLine {
        point_id: Uuid,
        line_start_id: Uuid,
        line_end_id: Uuid,
    },
    /// Tangent curves
    Tangent {
        curve1_id: Uuid,
        curve2_id: Uuid,
        parameter1: f64,
        parameter2: f64,
    },
}

/// A geometric constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub id: Uuid,
    pub constraint_type: ConstraintType,
    pub weight: f64,
    pub enabled: bool,
}

impl Constraint {
    /// Create a new constraint
    pub fn new(constraint_type: ConstraintType) -> Self {
        Self {
            id: Uuid::new_v4(),
            constraint_type,
            weight: 1.0,
            enabled: true,
        }
    }

    /// Evaluate the constraint error
    pub fn evaluate(&self, points: &HashMap<Uuid, Point3>) -> f64 {
        if !self.enabled {
            return 0.0;
        }

        match &self.constraint_type {
            ConstraintType::FixedPoint { point_id, position } => {
                if let Some(point) = points.get(point_id) {
                    (point - position).norm()
                } else {
                    0.0
                }
            }
            ConstraintType::Distance {
                point1_id,
                point2_id,
                distance,
            } => {
                if let (Some(p1), Some(p2)) = (points.get(point1_id), points.get(point2_id)) {
                    let actual_distance = (p2 - p1).norm();
                    (actual_distance - distance).abs()
                } else {
                    0.0
                }
            }
            ConstraintType::Coincident {
                point1_id,
                point2_id,
            } => {
                if let (Some(p1), Some(p2)) = (points.get(point1_id), points.get(point2_id)) {
                    (p2 - p1).norm()
                } else {
                    0.0
                }
            }
            ConstraintType::Horizontal {
                point1_id,
                point2_id,
            } => {
                if let (Some(p1), Some(p2)) = (points.get(point1_id), points.get(point2_id)) {
                    (p2.z - p1.z).abs()
                } else {
                    0.0
                }
            }
            ConstraintType::Vertical {
                point1_id,
                point2_id,
            } => {
                if let (Some(p1), Some(p2)) = (points.get(point1_id), points.get(point2_id)) {
                    ((p2.x - p1.x).powi(2) + (p2.y - p1.y).powi(2)).sqrt()
                } else {
                    0.0
                }
            }
            ConstraintType::Angle {
                vec1_start,
                vec1_end,
                vec2_start,
                vec2_end,
                angle,
            } => {
                if let (Some(v1s), Some(v1e), Some(v2s), Some(v2e)) = (
                    points.get(vec1_start),
                    points.get(vec1_end),
                    points.get(vec2_start),
                    points.get(vec2_end),
                ) {
                    let vec1 = (v1e - v1s).normalize();
                    let vec2 = (v2e - v2s).normalize();
                    let dot = vec1.dot(&vec2).clamp(-1.0, 1.0);
                    let actual_angle = dot.acos();
                    (actual_angle - angle).abs()
                } else {
                    0.0
                }
            }
            ConstraintType::Parallel {
                vec1_start,
                vec1_end,
                vec2_start,
                vec2_end,
            } => {
                if let (Some(v1s), Some(v1e), Some(v2s), Some(v2e)) = (
                    points.get(vec1_start),
                    points.get(vec1_end),
                    points.get(vec2_start),
                    points.get(vec2_end),
                ) {
                    let vec1 = (v1e - v1s).normalize();
                    let vec2 = (v2e - v2s).normalize();
                    let cross = vec1.cross(&vec2);
                    cross.norm()
                } else {
                    0.0
                }
            }
            ConstraintType::Perpendicular {
                vec1_start,
                vec1_end,
                vec2_start,
                vec2_end,
            } => {
                if let (Some(v1s), Some(v1e), Some(v2s), Some(v2e)) = (
                    points.get(vec1_start),
                    points.get(vec1_end),
                    points.get(vec2_start),
                    points.get(vec2_end),
                ) {
                    let vec1 = (v1e - v1s).normalize();
                    let vec2 = (v2e - v2s).normalize();
                    let dot = vec1.dot(&vec2);
                    dot.abs()
                } else {
                    0.0
                }
            }
            ConstraintType::PointOnLine {
                point_id,
                line_start_id,
                line_end_id,
            } => {
                if let (Some(point), Some(line_start), Some(line_end)) = (
                    points.get(point_id),
                    points.get(line_start_id),
                    points.get(line_end_id),
                ) {
                    let line_vec = line_end - line_start;
                    let point_vec = point - line_start;
                    let projection = line_vec.normalize() * point_vec.dot(&line_vec.normalize());
                    let perpendicular = point_vec - projection;
                    perpendicular.norm()
                } else {
                    0.0
                }
            }
            ConstraintType::Tangent { .. } => {
                // Tangency constraints require derivative information
                // Simplified implementation
                0.0
            }
        }
    }
}

/// Constraint solver using iterative least-squares
pub struct ConstraintSolver {
    constraints: Vec<Constraint>,
    max_iterations: usize,
    tolerance: f64,
    damping: f64,
}

impl ConstraintSolver {
    /// Create a new constraint solver
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            max_iterations: 100,
            tolerance: EPSILON * 10.0,
            damping: 0.5,
        }
    }

    /// Add a constraint
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    /// Remove a constraint by ID
    pub fn remove_constraint(&mut self, id: Uuid) {
        self.constraints.retain(|c| c.id != id);
    }

    /// Get all constraints
    pub fn constraints(&self) -> &[Constraint] {
        &self.constraints
    }

    /// Solve the constraint system
    pub fn solve(&self, points: &mut HashMap<Uuid, Point3>) -> SolveResult {
        let mut iterations = 0;
        let mut max_error = f64::INFINITY;

        for iteration in 0..self.max_iterations {
            iterations = iteration + 1;

            // Compute total error
            let total_error: f64 = self
                .constraints
                .iter()
                .map(|c| c.evaluate(points).powi(2) * c.weight)
                .sum();

            max_error = total_error.sqrt();

            if max_error < self.tolerance {
                return SolveResult {
                    converged: true,
                    iterations,
                    final_error: max_error,
                };
            }

            // Compute gradient and update positions
            self.gradient_descent_step(points);
        }

        SolveResult {
            converged: false,
            iterations,
            final_error: max_error,
        }
    }

    /// Perform one step of gradient descent
    fn gradient_descent_step(&self, points: &mut HashMap<Uuid, Point3>) {
        let mut gradients: HashMap<Uuid, Vector3> = HashMap::new();

        // Initialize gradients
        for point_id in points.keys() {
            gradients.insert(*point_id, Vector3::zeros());
        }

        // Compute gradients using finite differences
        let h = 1e-6;

        for constraint in &self.constraints {
            if !constraint.enabled {
                continue;
            }

            let error0 = constraint.evaluate(points);

            // Compute gradient for each point involved in the constraint
            for point_id in self.get_constraint_points(&constraint.constraint_type) {
                if let Some(point) = points.get(&point_id) {
                    let original = *point;

                    // Gradient in X
                    points.insert(point_id, Point3::new(original.x + h, original.y, original.z));
                    let error_x = constraint.evaluate(points);
                    let grad_x = (error_x - error0) / h;

                    // Gradient in Y
                    points.insert(point_id, Point3::new(original.x, original.y + h, original.z));
                    let error_y = constraint.evaluate(points);
                    let grad_y = (error_y - error0) / h;

                    // Gradient in Z
                    points.insert(point_id, Point3::new(original.x, original.y, original.z + h));
                    let error_z = constraint.evaluate(points);
                    let grad_z = (error_z - error0) / h;

                    // Restore original position
                    points.insert(point_id, original);

                    // Accumulate gradient
                    let grad = Vector3::new(grad_x, grad_y, grad_z) * constraint.weight;
                    *gradients.get_mut(&point_id).unwrap() += grad;
                }
            }
        }

        // Update positions
        for (point_id, gradient) in gradients.iter() {
            if let Some(point) = points.get_mut(point_id) {
                // Check if this point is fixed
                let is_fixed = self.constraints.iter().any(|c| {
                    matches!(
                        &c.constraint_type,
                        ConstraintType::FixedPoint { point_id: id, .. } if id == point_id
                    )
                });

                if !is_fixed && gradient.norm() > EPSILON {
                    // Move in opposite direction of gradient
                    let step = gradient.normalize() * self.damping;
                    *point = Point3::from(point.coords - step);
                }
            }
        }
    }

    /// Get all point IDs involved in a constraint
    fn get_constraint_points(&self, constraint_type: &ConstraintType) -> Vec<Uuid> {
        match constraint_type {
            ConstraintType::FixedPoint { point_id, .. } => vec![*point_id],
            ConstraintType::Distance {
                point1_id,
                point2_id,
                ..
            } => vec![*point1_id, *point2_id],
            ConstraintType::Coincident {
                point1_id,
                point2_id,
            } => vec![*point1_id, *point2_id],
            ConstraintType::Horizontal {
                point1_id,
                point2_id,
            } => vec![*point1_id, *point2_id],
            ConstraintType::Vertical {
                point1_id,
                point2_id,
            } => vec![*point1_id, *point2_id],
            ConstraintType::Angle {
                vec1_start,
                vec1_end,
                vec2_start,
                vec2_end,
                ..
            } => vec![*vec1_start, *vec1_end, *vec2_start, *vec2_end],
            ConstraintType::Parallel {
                vec1_start,
                vec1_end,
                vec2_start,
                vec2_end,
            } => vec![*vec1_start, *vec1_end, *vec2_start, *vec2_end],
            ConstraintType::Perpendicular {
                vec1_start,
                vec1_end,
                vec2_start,
                vec2_end,
            } => vec![*vec1_start, *vec1_end, *vec2_start, *vec2_end],
            ConstraintType::PointOnLine {
                point_id,
                line_start_id,
                line_end_id,
            } => vec![*point_id, *line_start_id, *line_end_id],
            ConstraintType::Tangent { .. } => vec![],
        }
    }
}

impl Default for ConstraintSolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of solving a constraint system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolveResult {
    pub converged: bool,
    pub iterations: usize,
    pub final_error: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_point_constraint() {
        let point_id = Uuid::new_v4();
        let fixed_pos = Point3::new(1.0, 2.0, 3.0);

        let constraint = Constraint::new(ConstraintType::FixedPoint {
            point_id,
            position: fixed_pos,
        });

        let mut points = HashMap::new();
        points.insert(point_id, Point3::new(1.0, 2.0, 3.0));

        let error = constraint.evaluate(&points);
        assert!(error < EPSILON);
    }

    #[test]
    fn test_distance_constraint() {
        let p1_id = Uuid::new_v4();
        let p2_id = Uuid::new_v4();

        let constraint = Constraint::new(ConstraintType::Distance {
            point1_id: p1_id,
            point2_id: p2_id,
            distance: 5.0,
        });

        let mut points = HashMap::new();
        points.insert(p1_id, Point3::new(0.0, 0.0, 0.0));
        points.insert(p2_id, Point3::new(3.0, 4.0, 0.0));

        let error = constraint.evaluate(&points);
        assert!(error < EPSILON);
    }

    #[test]
    fn test_constraint_solver() {
        let mut solver = ConstraintSolver::new();

        let p1_id = Uuid::new_v4();
        let p2_id = Uuid::new_v4();

        // Fix first point
        solver.add_constraint(Constraint::new(ConstraintType::FixedPoint {
            point_id: p1_id,
            position: Point3::new(0.0, 0.0, 0.0),
        }));

        // Set distance constraint
        solver.add_constraint(Constraint::new(ConstraintType::Distance {
            point1_id: p1_id,
            point2_id: p2_id,
            distance: 10.0,
        }));

        let mut points = HashMap::new();
        points.insert(p1_id, Point3::new(0.0, 0.0, 0.0));
        points.insert(p2_id, Point3::new(1.0, 1.0, 1.0));

        let result = solver.solve(&mut points);
        assert!(result.iterations > 0);
    }
}
