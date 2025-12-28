//! Dimensional constraints
//!
//! Provides dimensional constraints (distance, angle, radius) that control
//! the size and shape of geometry with numerical values.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::geometric::EntityReference;

/// Dimensional constraint type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DimensionalConstraintType {
    /// Distance between two points or point and curve
    Distance,
    /// Horizontal distance
    HorizontalDistance,
    /// Vertical distance
    VerticalDistance,
    /// Angle between two lines
    Angle,
    /// Radius of arc or circle
    Radius,
    /// Diameter of arc or circle
    Diameter,
    /// Length of a line or curve
    Length,
    /// Perimeter of a closed shape
    Perimeter,
    /// Area of a closed region
    Area,
}

/// Dimension constraint mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintMode {
    /// Driving dimension - controls the geometry
    Driving,
    /// Driven dimension - reports the value but doesn't control
    Driven,
}

/// Dimensional constraint
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimensionalConstraint {
    /// Unique constraint identifier
    pub id: Uuid,
    /// Constraint type
    pub constraint_type: DimensionalConstraintType,
    /// Entities involved in the constraint
    pub entities: Vec<EntityReference>,
    /// Target value
    pub value: f64,
    /// Constraint mode (driving or driven)
    pub mode: ConstraintMode,
    /// Is this constraint enabled
    pub enabled: bool,
    /// Constraint name (optional)
    pub name: Option<String>,
    /// Tolerance (+/-)
    pub tolerance: Option<f64>,
    /// Upper tolerance
    pub upper_tolerance: Option<f64>,
    /// Lower tolerance
    pub lower_tolerance: Option<f64>,
    /// Constraint priority (higher = solve first)
    pub priority: i32,
    /// Expression for parametric value (e.g., "d1 * 2")
    pub expression: Option<String>,
    /// Is this a construction constraint
    pub is_construction: bool,
}

impl DimensionalConstraint {
    /// Create a new dimensional constraint
    pub fn new(
        constraint_type: DimensionalConstraintType,
        entities: Vec<EntityReference>,
        value: f64,
    ) -> Self {
        DimensionalConstraint {
            id: Uuid::new_v4(),
            constraint_type,
            entities,
            value,
            mode: ConstraintMode::Driving,
            enabled: true,
            name: None,
            tolerance: None,
            upper_tolerance: None,
            lower_tolerance: None,
            priority: 0,
            expression: None,
            is_construction: false,
        }
    }

    /// Create a distance constraint between two entities
    pub fn distance(entity1: EntityReference, entity2: EntityReference, distance: f64) -> Self {
        Self::new(
            DimensionalConstraintType::Distance,
            vec![entity1, entity2],
            distance,
        )
    }

    /// Create a horizontal distance constraint
    pub fn horizontal_distance(
        entity1: EntityReference,
        entity2: EntityReference,
        distance: f64,
    ) -> Self {
        Self::new(
            DimensionalConstraintType::HorizontalDistance,
            vec![entity1, entity2],
            distance,
        )
    }

    /// Create a vertical distance constraint
    pub fn vertical_distance(
        entity1: EntityReference,
        entity2: EntityReference,
        distance: f64,
    ) -> Self {
        Self::new(
            DimensionalConstraintType::VerticalDistance,
            vec![entity1, entity2],
            distance,
        )
    }

    /// Create an angle constraint between two lines
    pub fn angle(line1: EntityReference, line2: EntityReference, angle_radians: f64) -> Self {
        Self::new(
            DimensionalConstraintType::Angle,
            vec![line1, line2],
            angle_radians,
        )
    }

    /// Create a radius constraint for a circle or arc
    pub fn radius(circle: EntityReference, radius: f64) -> Self {
        Self::new(DimensionalConstraintType::Radius, vec![circle], radius)
    }

    /// Create a diameter constraint for a circle or arc
    pub fn diameter(circle: EntityReference, diameter: f64) -> Self {
        Self::new(DimensionalConstraintType::Diameter, vec![circle], diameter)
    }

    /// Create a length constraint for a line or curve
    pub fn length(entity: EntityReference, length: f64) -> Self {
        Self::new(DimensionalConstraintType::Length, vec![entity], length)
    }

    /// Set constraint name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set constraint mode
    pub fn with_mode(mut self, mode: ConstraintMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set as driven constraint (reference only, doesn't drive geometry)
    pub fn as_driven(mut self) -> Self {
        self.mode = ConstraintMode::Driven;
        self
    }

    /// Set symmetrical tolerance
    pub fn with_tolerance(mut self, tolerance: f64) -> Self {
        self.tolerance = Some(tolerance);
        self
    }

    /// Set asymmetrical tolerance
    pub fn with_tolerances(mut self, upper: f64, lower: f64) -> Self {
        self.upper_tolerance = Some(upper);
        self.lower_tolerance = Some(lower);
        self
    }

    /// Set constraint priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Set parametric expression
    pub fn with_expression(mut self, expression: impl Into<String>) -> Self {
        self.expression = Some(expression.into());
        self
    }

    /// Mark as construction constraint
    pub fn as_construction(mut self) -> Self {
        self.is_construction = true;
        self
    }

    /// Enable or disable constraint
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Update constraint value
    pub fn set_value(&mut self, value: f64) {
        if self.mode == ConstraintMode::Driving {
            self.value = value;
        }
    }

    /// Get effective value (evaluates expression if present)
    pub fn get_effective_value(&self, _parameters: &ParameterTable) -> f64 {
        // In real implementation, would evaluate expression
        // For now, just return the stored value
        self.value
    }

    /// Check if current geometry satisfies constraint
    pub fn is_satisfied(&self, actual_value: f64) -> bool {
        if !self.enabled {
            return true;
        }

        if let Some(tol) = self.tolerance {
            (actual_value - self.value).abs() <= tol
        } else if let (Some(upper), Some(lower)) = (self.upper_tolerance, self.lower_tolerance) {
            let diff = actual_value - self.value;
            diff >= -lower && diff <= upper
        } else {
            // Default tolerance
            (actual_value - self.value).abs() < 1e-6
        }
    }

    /// Check if this constraint involves a specific entity
    pub fn involves_entity(&self, entity_id: Uuid) -> bool {
        self.entities.iter().any(|e| e.entity_id() == entity_id)
    }

    /// Get constraint description
    pub fn description(&self) -> String {
        if let Some(ref name) = self.name {
            return format!("{} = {}", name, self.value);
        }

        let value_str = if let Some(ref expr) = self.expression {
            expr.clone()
        } else {
            format!("{:.3}", self.value)
        };

        match self.constraint_type {
            DimensionalConstraintType::Distance => format!("Distance = {}", value_str),
            DimensionalConstraintType::HorizontalDistance => {
                format!("Horizontal Distance = {}", value_str)
            }
            DimensionalConstraintType::VerticalDistance => {
                format!("Vertical Distance = {}", value_str)
            }
            DimensionalConstraintType::Angle => {
                format!("Angle = {}°", value_str)
            }
            DimensionalConstraintType::Radius => format!("Radius = {}", value_str),
            DimensionalConstraintType::Diameter => format!("Diameter = {}", value_str),
            DimensionalConstraintType::Length => format!("Length = {}", value_str),
            DimensionalConstraintType::Perimeter => format!("Perimeter = {}", value_str),
            DimensionalConstraintType::Area => format!("Area = {}", value_str),
        }
    }
}

/// Parameter table for parametric constraints
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParameterTable {
    /// Named parameters
    parameters: std::collections::HashMap<String, f64>,
}

impl ParameterTable {
    /// Create a new parameter table
    pub fn new() -> Self {
        ParameterTable {
            parameters: std::collections::HashMap::new(),
        }
    }

    /// Set a parameter value
    pub fn set(&mut self, name: impl Into<String>, value: f64) {
        self.parameters.insert(name.into(), value);
    }

    /// Get a parameter value
    pub fn get(&self, name: &str) -> Option<f64> {
        self.parameters.get(name).copied()
    }

    /// Remove a parameter
    pub fn remove(&mut self, name: &str) -> Option<f64> {
        self.parameters.remove(name)
    }

    /// Check if parameter exists
    pub fn contains(&self, name: &str) -> bool {
        self.parameters.contains_key(name)
    }

    /// Get all parameter names
    pub fn parameter_names(&self) -> Vec<String> {
        self.parameters.keys().cloned().collect()
    }

    /// Evaluate a simple expression (simplified parser)
    pub fn evaluate(&self, expression: &str) -> Result<f64, String> {
        // Simple implementation - real one would use a proper expression parser
        // For now, just handle parameter lookups and simple arithmetic

        let expr = expression.trim();

        // Try to parse as number
        if let Ok(value) = expr.parse::<f64>() {
            return Ok(value);
        }

        // Try to lookup as parameter
        if let Some(value) = self.get(expr) {
            return Ok(value);
        }

        // Simple arithmetic operators
        if let Some(pos) = expr.find('+') {
            let left = self.evaluate(&expr[..pos])?;
            let right = self.evaluate(&expr[pos + 1..])?;
            return Ok(left + right);
        }

        if let Some(pos) = expr.rfind('-') {
            if pos > 0 {
                let left = self.evaluate(&expr[..pos])?;
                let right = self.evaluate(&expr[pos + 1..])?;
                return Ok(left - right);
            }
        }

        if let Some(pos) = expr.find('*') {
            let left = self.evaluate(&expr[..pos])?;
            let right = self.evaluate(&expr[pos + 1..])?;
            return Ok(left * right);
        }

        if let Some(pos) = expr.find('/') {
            let left = self.evaluate(&expr[..pos])?;
            let right = self.evaluate(&expr[pos + 1..])?;
            if right.abs() < 1e-10 {
                return Err("Division by zero".to_string());
            }
            return Ok(left / right);
        }

        Err(format!("Cannot evaluate expression: {}", expr))
    }
}

/// Constraint equation for solver
#[derive(Debug, Clone, PartialEq)]
pub struct ConstraintEquation {
    /// Constraint ID this equation represents
    pub constraint_id: Uuid,
    /// Variables involved (entity IDs and their DOF indices)
    pub variables: Vec<(Uuid, usize)>,
    /// Current error/residual
    pub residual: f64,
    /// Weight/importance of this constraint
    pub weight: f64,
}

impl ConstraintEquation {
    /// Create a new constraint equation
    pub fn new(constraint_id: Uuid) -> Self {
        ConstraintEquation {
            constraint_id,
            variables: Vec::new(),
            residual: 0.0,
            weight: 1.0,
        }
    }

    /// Add a variable to the equation
    pub fn add_variable(&mut self, entity_id: Uuid, dof_index: usize) {
        self.variables.push((entity_id, dof_index));
    }

    /// Set the residual value
    pub fn set_residual(&mut self, residual: f64) {
        self.residual = residual;
    }

    /// Set the weight
    pub fn set_weight(&mut self, weight: f64) {
        self.weight = weight;
    }

    /// Check if equation is satisfied
    pub fn is_satisfied(&self, tolerance: f64) -> bool {
        self.residual.abs() < tolerance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_constraint() {
        let p1 = EntityReference::Point(Uuid::new_v4());
        let p2 = EntityReference::Point(Uuid::new_v4());

        let constraint = DimensionalConstraint::distance(p1, p2, 50.0)
            .with_name("D1")
            .with_tolerance(0.1);

        assert_eq!(constraint.value, 50.0);
        assert_eq!(constraint.tolerance, Some(0.1));
        assert_eq!(constraint.mode, ConstraintMode::Driving);
    }

    #[test]
    fn test_radius_constraint() {
        let circle = EntityReference::Circle(Uuid::new_v4());
        let constraint = DimensionalConstraint::radius(circle, 25.0);

        assert_eq!(constraint.constraint_type, DimensionalConstraintType::Radius);
        assert_eq!(constraint.value, 25.0);
    }

    #[test]
    fn test_driven_constraint() {
        let line = EntityReference::Line(Uuid::new_v4());
        let constraint = DimensionalConstraint::length(line, 100.0).as_driven();

        assert_eq!(constraint.mode, ConstraintMode::Driven);
    }

    #[test]
    fn test_parametric_constraint() {
        let p1 = EntityReference::Point(Uuid::new_v4());
        let p2 = EntityReference::Point(Uuid::new_v4());

        let constraint = DimensionalConstraint::distance(p1, p2, 50.0)
            .with_expression("width * 2");

        assert_eq!(constraint.expression, Some("width * 2".to_string()));
    }

    #[test]
    fn test_tolerance_check() {
        let p1 = EntityReference::Point(Uuid::new_v4());
        let p2 = EntityReference::Point(Uuid::new_v4());

        let constraint = DimensionalConstraint::distance(p1, p2, 50.0)
            .with_tolerance(0.5);

        assert!(constraint.is_satisfied(50.0));
        assert!(constraint.is_satisfied(50.3));
        assert!(constraint.is_satisfied(49.7));
        assert!(!constraint.is_satisfied(51.0));
    }

    #[test]
    fn test_asymmetric_tolerance() {
        let p1 = EntityReference::Point(Uuid::new_v4());
        let p2 = EntityReference::Point(Uuid::new_v4());

        let constraint = DimensionalConstraint::distance(p1, p2, 50.0)
            .with_tolerances(0.5, 0.2);

        assert!(constraint.is_satisfied(50.0));
        assert!(constraint.is_satisfied(50.4));
        assert!(constraint.is_satisfied(49.9));
        assert!(!constraint.is_satisfied(50.6));
        assert!(!constraint.is_satisfied(49.7));
    }

    #[test]
    fn test_parameter_table() {
        let mut params = ParameterTable::new();
        params.set("width", 100.0);
        params.set("height", 50.0);

        assert_eq!(params.get("width"), Some(100.0));
        assert_eq!(params.get("height"), Some(50.0));
        assert_eq!(params.get("depth"), None);
    }

    #[test]
    fn test_expression_evaluation() {
        let mut params = ParameterTable::new();
        params.set("d1", 10.0);
        params.set("d2", 5.0);

        assert_eq!(params.evaluate("d1").unwrap(), 10.0);
        assert_eq!(params.evaluate("d1 + d2").unwrap(), 15.0);
        assert_eq!(params.evaluate("d1 - d2").unwrap(), 5.0);
        assert_eq!(params.evaluate("d1 * d2").unwrap(), 50.0);
        assert_eq!(params.evaluate("d1 / d2").unwrap(), 2.0);
    }

    #[test]
    fn test_constraint_equation() {
        let mut eq = ConstraintEquation::new(Uuid::new_v4());
        eq.add_variable(Uuid::new_v4(), 0);
        eq.add_variable(Uuid::new_v4(), 1);
        eq.set_residual(0.001);

        assert!(eq.is_satisfied(0.01));
        assert!(!eq.is_satisfied(0.0001));
    }
}
