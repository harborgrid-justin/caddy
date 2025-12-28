//! Geometric constraints
//!
//! Provides geometric relationship constraints between entities such as
//! coincident, parallel, perpendicular, tangent, concentric, etc.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Geometric constraint type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GeometricConstraintType {
    /// Two points must be at the same location
    Coincident,
    /// Two lines must be parallel
    Parallel,
    /// Two lines must be perpendicular
    Perpendicular,
    /// Line must be horizontal
    Horizontal,
    /// Line must be vertical
    Vertical,
    /// Curve must be tangent to another curve
    Tangent,
    /// Two circles/arcs must share the same center
    Concentric,
    /// Two entities must have equal length/radius
    Equal,
    /// Point or entity is fixed in space
    Fixed,
    /// Line or curve is collinear with another
    Collinear,
    /// Three or more points lie on the same line
    Colinear,
    /// Point lies on curve
    PointOnCurve,
    /// Point is at midpoint of line
    Midpoint,
    /// Arc/circle is symmetric about a line
    Symmetric,
    /// Smooth connection between curves (G1 continuity)
    Smooth,
}

/// Reference to a geometric entity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EntityReference {
    /// Point entity
    Point(Uuid),
    /// Line entity
    Line(Uuid),
    /// Arc entity
    Arc(Uuid),
    /// Circle entity
    Circle(Uuid),
    /// Spline entity
    Spline(Uuid),
    /// Ellipse entity
    Ellipse(Uuid),
    /// Point on an entity (entity_id, parameter)
    PointOnEntity {
        entity_id: Uuid,
        parameter: f64,
    },
    /// Start point of entity
    StartPoint(Uuid),
    /// End point of entity
    EndPoint(Uuid),
    /// Center point of arc/circle
    CenterPoint(Uuid),
}

impl EntityReference {
    /// Get the entity ID
    pub fn entity_id(&self) -> Uuid {
        match self {
            EntityReference::Point(id)
            | EntityReference::Line(id)
            | EntityReference::Arc(id)
            | EntityReference::Circle(id)
            | EntityReference::Spline(id)
            | EntityReference::Ellipse(id)
            | EntityReference::StartPoint(id)
            | EntityReference::EndPoint(id)
            | EntityReference::CenterPoint(id) => *id,
            EntityReference::PointOnEntity { entity_id, .. } => *entity_id,
        }
    }
}

/// Geometric constraint between entities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeometricConstraint {
    /// Unique constraint identifier
    pub id: Uuid,
    /// Constraint type
    pub constraint_type: GeometricConstraintType,
    /// Entities involved in the constraint
    pub entities: Vec<EntityReference>,
    /// Is this constraint enabled
    pub enabled: bool,
    /// Constraint name (optional)
    pub name: Option<String>,
    /// Constraint priority (higher = solve first)
    pub priority: i32,
    /// Is this a construction constraint (helper, not user-visible)
    pub is_construction: bool,
}

impl GeometricConstraint {
    /// Create a new geometric constraint
    pub fn new(
        constraint_type: GeometricConstraintType,
        entities: Vec<EntityReference>,
    ) -> Self {
        GeometricConstraint {
            id: Uuid::new_v4(),
            constraint_type,
            entities,
            enabled: true,
            name: None,
            priority: 0,
            is_construction: false,
        }
    }

    /// Create a coincident constraint between two points
    pub fn coincident(point1: EntityReference, point2: EntityReference) -> Self {
        Self::new(GeometricConstraintType::Coincident, vec![point1, point2])
    }

    /// Create a parallel constraint between two lines
    pub fn parallel(line1: EntityReference, line2: EntityReference) -> Self {
        Self::new(GeometricConstraintType::Parallel, vec![line1, line2])
    }

    /// Create a perpendicular constraint between two lines
    pub fn perpendicular(line1: EntityReference, line2: EntityReference) -> Self {
        Self::new(GeometricConstraintType::Perpendicular, vec![line1, line2])
    }

    /// Create a horizontal constraint for a line
    pub fn horizontal(line: EntityReference) -> Self {
        Self::new(GeometricConstraintType::Horizontal, vec![line])
    }

    /// Create a vertical constraint for a line
    pub fn vertical(line: EntityReference) -> Self {
        Self::new(GeometricConstraintType::Vertical, vec![line])
    }

    /// Create a tangent constraint between two curves
    pub fn tangent(curve1: EntityReference, curve2: EntityReference) -> Self {
        Self::new(GeometricConstraintType::Tangent, vec![curve1, curve2])
    }

    /// Create a concentric constraint between two circles/arcs
    pub fn concentric(circle1: EntityReference, circle2: EntityReference) -> Self {
        Self::new(GeometricConstraintType::Concentric, vec![circle1, circle2])
    }

    /// Create an equal constraint between two entities
    pub fn equal(entity1: EntityReference, entity2: EntityReference) -> Self {
        Self::new(GeometricConstraintType::Equal, vec![entity1, entity2])
    }

    /// Create a fixed constraint for an entity
    pub fn fixed(entity: EntityReference) -> Self {
        Self::new(GeometricConstraintType::Fixed, vec![entity])
    }

    /// Create a point on curve constraint
    pub fn point_on_curve(point: EntityReference, curve: EntityReference) -> Self {
        Self::new(GeometricConstraintType::PointOnCurve, vec![point, curve])
    }

    /// Create a midpoint constraint
    pub fn midpoint(point: EntityReference, line: EntityReference) -> Self {
        Self::new(GeometricConstraintType::Midpoint, vec![point, line])
    }

    /// Create a symmetric constraint
    pub fn symmetric(entity1: EntityReference, entity2: EntityReference, axis: EntityReference) -> Self {
        Self::new(GeometricConstraintType::Symmetric, vec![entity1, entity2, axis])
    }

    /// Set constraint name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set constraint priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
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

    /// Check if constraint is satisfied (stub - real check would use actual geometry)
    pub fn is_satisfied(&self, tolerance: f64) -> bool {
        // In a real implementation, this would check the actual geometry
        // For now, we'll just return true as a placeholder
        true
    }

    /// Get the number of entities this constraint affects
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Check if this constraint involves a specific entity
    pub fn involves_entity(&self, entity_id: Uuid) -> bool {
        self.entities.iter().any(|e| e.entity_id() == entity_id)
    }

    /// Get constraint description
    pub fn description(&self) -> String {
        if let Some(ref name) = self.name {
            return name.clone();
        }

        match self.constraint_type {
            GeometricConstraintType::Coincident => "Coincident".to_string(),
            GeometricConstraintType::Parallel => "Parallel".to_string(),
            GeometricConstraintType::Perpendicular => "Perpendicular".to_string(),
            GeometricConstraintType::Horizontal => "Horizontal".to_string(),
            GeometricConstraintType::Vertical => "Vertical".to_string(),
            GeometricConstraintType::Tangent => "Tangent".to_string(),
            GeometricConstraintType::Concentric => "Concentric".to_string(),
            GeometricConstraintType::Equal => "Equal".to_string(),
            GeometricConstraintType::Fixed => "Fixed".to_string(),
            GeometricConstraintType::Collinear => "Collinear".to_string(),
            GeometricConstraintType::Colinear => "Colinear".to_string(),
            GeometricConstraintType::PointOnCurve => "Point On Curve".to_string(),
            GeometricConstraintType::Midpoint => "Midpoint".to_string(),
            GeometricConstraintType::Symmetric => "Symmetric".to_string(),
            GeometricConstraintType::Smooth => "Smooth".to_string(),
        }
    }
}

/// Constraint group for managing related constraints
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstraintGroup {
    /// Group identifier
    pub id: Uuid,
    /// Group name
    pub name: String,
    /// Constraints in this group
    pub constraint_ids: Vec<Uuid>,
    /// Is group enabled
    pub enabled: bool,
}

impl ConstraintGroup {
    /// Create a new constraint group
    pub fn new(name: impl Into<String>) -> Self {
        ConstraintGroup {
            id: Uuid::new_v4(),
            name: name.into(),
            constraint_ids: Vec::new(),
            enabled: true,
        }
    }

    /// Add a constraint to the group
    pub fn add_constraint(&mut self, constraint_id: Uuid) {
        if !self.constraint_ids.contains(&constraint_id) {
            self.constraint_ids.push(constraint_id);
        }
    }

    /// Remove a constraint from the group
    pub fn remove_constraint(&mut self, constraint_id: Uuid) {
        self.constraint_ids.retain(|id| *id != constraint_id);
    }

    /// Check if group contains a constraint
    pub fn contains(&self, constraint_id: Uuid) -> bool {
        self.constraint_ids.contains(&constraint_id)
    }
}

/// Constraint conflict information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstraintConflict {
    /// Conflicting constraints
    pub constraint_ids: Vec<Uuid>,
    /// Conflict description
    pub description: String,
    /// Severity (1-10, 10 = critical)
    pub severity: u8,
}

impl ConstraintConflict {
    /// Create a new constraint conflict
    pub fn new(constraint_ids: Vec<Uuid>, description: impl Into<String>, severity: u8) -> Self {
        ConstraintConflict {
            constraint_ids,
            description: description.into(),
            severity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coincident_constraint() {
        let p1 = EntityReference::Point(Uuid::new_v4());
        let p2 = EntityReference::Point(Uuid::new_v4());

        let constraint = GeometricConstraint::coincident(p1, p2);

        assert_eq!(constraint.constraint_type, GeometricConstraintType::Coincident);
        assert_eq!(constraint.entities.len(), 2);
        assert!(constraint.enabled);
    }

    #[test]
    fn test_parallel_constraint() {
        let l1 = EntityReference::Line(Uuid::new_v4());
        let l2 = EntityReference::Line(Uuid::new_v4());

        let constraint = GeometricConstraint::parallel(l1, l2)
            .with_name("Parallel Lines")
            .with_priority(10);

        assert_eq!(constraint.constraint_type, GeometricConstraintType::Parallel);
        assert_eq!(constraint.name, Some("Parallel Lines".to_string()));
        assert_eq!(constraint.priority, 10);
    }

    #[test]
    fn test_horizontal_constraint() {
        let line = EntityReference::Line(Uuid::new_v4());
        let constraint = GeometricConstraint::horizontal(line);

        assert_eq!(constraint.constraint_type, GeometricConstraintType::Horizontal);
        assert_eq!(constraint.entities.len(), 1);
    }

    #[test]
    fn test_constraint_group() {
        let mut group = ConstraintGroup::new("Profile Constraints");

        let c1_id = Uuid::new_v4();
        let c2_id = Uuid::new_v4();

        group.add_constraint(c1_id);
        group.add_constraint(c2_id);

        assert_eq!(group.constraint_ids.len(), 2);
        assert!(group.contains(c1_id));
        assert!(group.contains(c2_id));

        group.remove_constraint(c1_id);
        assert_eq!(group.constraint_ids.len(), 1);
        assert!(!group.contains(c1_id));
    }

    #[test]
    fn test_entity_reference() {
        let entity_id = Uuid::new_v4();
        let point_ref = EntityReference::Point(entity_id);

        assert_eq!(point_ref.entity_id(), entity_id);
    }

    #[test]
    fn test_symmetric_constraint() {
        let e1 = EntityReference::Line(Uuid::new_v4());
        let e2 = EntityReference::Line(Uuid::new_v4());
        let axis = EntityReference::Line(Uuid::new_v4());

        let constraint = GeometricConstraint::symmetric(e1, e2, axis);

        assert_eq!(constraint.constraint_type, GeometricConstraintType::Symmetric);
        assert_eq!(constraint.entities.len(), 3);
    }

    #[test]
    fn test_constraint_conflict() {
        let c1 = Uuid::new_v4();
        let c2 = Uuid::new_v4();

        let conflict = ConstraintConflict::new(
            vec![c1, c2],
            "Cannot be both horizontal and vertical",
            10,
        );

        assert_eq!(conflict.constraint_ids.len(), 2);
        assert_eq!(conflict.severity, 10);
    }
}
