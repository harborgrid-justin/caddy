// Grip edit tool - allows editing entities by dragging grip points

use super::{Point3, EntityId};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GripType {
    Corner,
    Midpoint,
    Center,
    Endpoint,
    ControlPoint,
}

#[derive(Debug, Clone)]
pub struct GripPoint {
    pub position: Point3,
    pub grip_type: GripType,
    pub entity_id: EntityId,
}

#[derive(Debug, Clone)]
pub struct GripSet {
    pub grips: Vec<GripPoint>,
}

pub struct GripEditor {
    active_grips: GripSet,
}

impl GripEditor {
    pub fn new() -> Self {
        Self {
            active_grips: GripSet { grips: Vec::new() },
        }
    }
}
