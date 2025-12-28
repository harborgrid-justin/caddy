// Modification commands for CADDY CAD system
// Implements entity modification commands (MOVE, COPY, ROTATE, SCALE, etc.)

use super::command::*;
use std::any::Any;
use std::collections::HashMap;

// ==================== MOVE COMMAND ====================

pub struct MoveCommand {
    selection: Vec<EntityId>,
    from_point: Option<Point>,
    to_point: Option<Point>,
    original_positions: HashMap<EntityId, Box<dyn Any + Send + Sync>>,
    state: CommandState,
}

impl MoveCommand {
    pub fn new() -> Self {
        Self {
            selection: Vec::new(),
            from_point: None,
            to_point: None,
            original_positions: HashMap::new(),
            state: CommandState::AwaitingParameter("base point".to_string()),
        }
    }

    pub fn with_selection(selection: Vec<EntityId>) -> Self {
        Self {
            selection,
            from_point: None,
            to_point: None,
            original_positions: HashMap::new(),
            state: CommandState::AwaitingParameter("base point".to_string()),
        }
    }
}

impl Command for MoveCommand {
    fn name(&self) -> &str {
        "MOVE"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["M"]
    }

    fn description(&self) -> &str {
        "Move selected entities"
    }

    fn usage(&self) -> &str {
        "MOVE (select entities) <base_x> <base_y> <target_x> <target_y>"
    }

    fn execute(&mut self, context: &mut CommandContext) -> CommandResult {
        if self.selection.is_empty() {
            self.selection = context.selection.entities.clone();
        }

        if self.selection.is_empty() {
            return Err(CommandError::InvalidSelection("No entities selected".to_string()));
        }

        let from = self.from_point.ok_or_else(||
            CommandError::InvalidInput("Base point not specified".to_string()))?;
        let to = self.to_point.ok_or_else(||
            CommandError::InvalidInput("Target point not specified".to_string()))?;

        let dx = to.x - from.x;
        let dy = to.y - from.y;
        let dz = to.z - from.z;

        // Store original positions and move entities
        for entity_id in &self.selection {
            // In a real implementation, we would transform the actual geometry
            // For now, we just mark that the entity was moved
            self.original_positions.insert(*entity_id, Box::new((dx, dy, dz)));
        }

        self.state = CommandState::Completed;
        Ok(())
    }

    fn undo(&mut self, context: &mut CommandContext) -> CommandResult {
        // Restore original positions
        for (entity_id, _offset) in &self.original_positions {
            // In real implementation, move entities back
            if context.document.get_entity(entity_id).is_none() {
                return Err(CommandError::EntityNotFound(format!("Entity {:?} not found", entity_id)));
            }
        }
        self.original_positions.clear();
        Ok(())
    }

    fn state(&self) -> CommandState {
        self.state.clone()
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(MoveCommand {
            selection: self.selection.clone(),
            from_point: self.from_point,
            to_point: self.to_point,
            original_positions: HashMap::new(),
            state: self.state.clone(),
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ==================== COPY COMMAND ====================

#[derive(Clone)]
pub struct CopyCommand {
    selection: Vec<EntityId>,
    from_point: Option<Point>,
    to_point: Option<Point>,
    created_entities: Vec<EntityId>,
    state: CommandState,
}

impl CopyCommand {
    pub fn new() -> Self {
        Self {
            selection: Vec::new(),
            from_point: None,
            to_point: None,
            created_entities: Vec::new(),
            state: CommandState::AwaitingParameter("base point".to_string()),
        }
    }
}

impl Command for CopyCommand {
    fn name(&self) -> &str {
        "COPY"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["CO", "CP"]
    }

    fn description(&self) -> &str {
        "Copy selected entities"
    }

    fn usage(&self) -> &str {
        "COPY (select entities) <base_x> <base_y> <target_x> <target_y>"
    }

    fn execute(&mut self, context: &mut CommandContext) -> CommandResult {
        if self.selection.is_empty() {
            self.selection = context.selection.entities.clone();
        }

        if self.selection.is_empty() {
            return Err(CommandError::InvalidSelection("No entities selected".to_string()));
        }

        let _from = self.from_point.ok_or_else(||
            CommandError::InvalidInput("Base point not specified".to_string()))?;
        let _to = self.to_point.ok_or_else(||
            CommandError::InvalidInput("Target point not specified".to_string()))?;

        // Create copies of selected entities
        for entity_id in &self.selection {
            if let Some(entity) = context.document.get_entity(entity_id) {
                // In real implementation, clone and transform the entity
                let copy_data = Box::new(());
                let new_id = context.document.add_entity(copy_data);
                self.created_entities.push(new_id);
            }
        }

        self.state = CommandState::Completed;
        Ok(())
    }

    fn undo(&mut self, context: &mut CommandContext) -> CommandResult {
        for entity_id in &self.created_entities {
            context.document.remove_entity(entity_id);
        }
        self.created_entities.clear();
        Ok(())
    }

    fn state(&self) -> CommandState {
        self.state.clone()
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ==================== ROTATE COMMAND ====================

#[derive(Clone)]
pub struct RotateCommand {
    selection: Vec<EntityId>,
    base_point: Option<Point>,
    angle: Option<f64>,
    original_rotations: HashMap<EntityId, f64>,
    state: CommandState,
}

impl RotateCommand {
    pub fn new() -> Self {
        Self {
            selection: Vec::new(),
            base_point: None,
            angle: None,
            original_rotations: HashMap::new(),
            state: CommandState::AwaitingParameter("base point".to_string()),
        }
    }
}

impl Command for RotateCommand {
    fn name(&self) -> &str {
        "ROTATE"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["RO"]
    }

    fn description(&self) -> &str {
        "Rotate selected entities around a base point"
    }

    fn usage(&self) -> &str {
        "ROTATE (select entities) <base_x> <base_y> <angle_degrees>"
    }

    fn execute(&mut self, context: &mut CommandContext) -> CommandResult {
        if self.selection.is_empty() {
            self.selection = context.selection.entities.clone();
        }

        if self.selection.is_empty() {
            return Err(CommandError::InvalidSelection("No entities selected".to_string()));
        }

        let _base = self.base_point.ok_or_else(||
            CommandError::InvalidInput("Base point not specified".to_string()))?;
        let angle = self.angle.ok_or_else(||
            CommandError::InvalidInput("Angle not specified".to_string()))?;

        // Rotate entities
        for entity_id in &self.selection {
            self.original_rotations.insert(*entity_id, angle);
        }

        self.state = CommandState::Completed;
        Ok(())
    }

    fn undo(&mut self, context: &mut CommandContext) -> CommandResult {
        // Rotate back
        for (entity_id, _angle) in &self.original_rotations {
            if context.document.get_entity(entity_id).is_none() {
                return Err(CommandError::EntityNotFound(format!("Entity {:?} not found", entity_id)));
            }
        }
        self.original_rotations.clear();
        Ok(())
    }

    fn state(&self) -> CommandState {
        self.state.clone()
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(RotateCommand {
            selection: self.selection.clone(),
            base_point: self.base_point,
            angle: self.angle,
            original_rotations: HashMap::new(),
            state: self.state.clone(),
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ==================== SCALE COMMAND ====================

#[derive(Clone)]
pub struct ScaleCommand {
    selection: Vec<EntityId>,
    base_point: Option<Point>,
    scale_factor: Option<f64>,
    original_scales: HashMap<EntityId, f64>,
    state: CommandState,
}

impl ScaleCommand {
    pub fn new() -> Self {
        Self {
            selection: Vec::new(),
            base_point: None,
            scale_factor: None,
            original_scales: HashMap::new(),
            state: CommandState::AwaitingParameter("base point".to_string()),
        }
    }
}

impl Command for ScaleCommand {
    fn name(&self) -> &str {
        "SCALE"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["SC"]
    }

    fn description(&self) -> &str {
        "Scale selected entities"
    }

    fn usage(&self) -> &str {
        "SCALE (select entities) <base_x> <base_y> <scale_factor>"
    }

    fn execute(&mut self, context: &mut CommandContext) -> CommandResult {
        if self.selection.is_empty() {
            self.selection = context.selection.entities.clone();
        }

        if self.selection.is_empty() {
            return Err(CommandError::InvalidSelection("No entities selected".to_string()));
        }

        let _base = self.base_point.ok_or_else(||
            CommandError::InvalidInput("Base point not specified".to_string()))?;
        let scale = self.scale_factor.ok_or_else(||
            CommandError::InvalidInput("Scale factor not specified".to_string()))?;

        if scale <= 0.0 {
            return Err(CommandError::InvalidInput("Scale factor must be positive".to_string()));
        }

        // Scale entities
        for entity_id in &self.selection {
            self.original_scales.insert(*entity_id, scale);
        }

        self.state = CommandState::Completed;
        Ok(())
    }

    fn undo(&mut self, context: &mut CommandContext) -> CommandResult {
        // Scale back
        for (entity_id, _scale) in &self.original_scales {
            if context.document.get_entity(entity_id).is_none() {
                return Err(CommandError::EntityNotFound(format!("Entity {:?} not found", entity_id)));
            }
        }
        self.original_scales.clear();
        Ok(())
    }

    fn state(&self) -> CommandState {
        self.state.clone()
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(ScaleCommand {
            selection: self.selection.clone(),
            base_point: self.base_point,
            scale_factor: self.scale_factor,
            original_scales: HashMap::new(),
            state: self.state.clone(),
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ==================== MIRROR COMMAND ====================

#[derive(Clone)]
pub struct MirrorCommand {
    selection: Vec<EntityId>,
    mirror_point1: Option<Point>,
    mirror_point2: Option<Point>,
    delete_source: bool,
    created_entities: Vec<EntityId>,
    state: CommandState,
}

impl MirrorCommand {
    pub fn new() -> Self {
        Self {
            selection: Vec::new(),
            mirror_point1: None,
            mirror_point2: None,
            delete_source: false,
            created_entities: Vec::new(),
            state: CommandState::AwaitingParameter("first mirror line point".to_string()),
        }
    }
}

impl Command for MirrorCommand {
    fn name(&self) -> &str {
        "MIRROR"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["MI"]
    }

    fn description(&self) -> &str {
        "Mirror selected entities across a line"
    }

    fn usage(&self) -> &str {
        "MIRROR (select entities) <x1> <y1> <x2> <y2>"
    }

    fn execute(&mut self, context: &mut CommandContext) -> CommandResult {
        if self.selection.is_empty() {
            self.selection = context.selection.entities.clone();
        }

        if self.selection.is_empty() {
            return Err(CommandError::InvalidSelection("No entities selected".to_string()));
        }

        let _p1 = self.mirror_point1.ok_or_else(||
            CommandError::InvalidInput("First mirror point not specified".to_string()))?;
        let _p2 = self.mirror_point2.ok_or_else(||
            CommandError::InvalidInput("Second mirror point not specified".to_string()))?;

        // Create mirrored copies
        for entity_id in &self.selection {
            if context.document.get_entity(entity_id).is_some() {
                let mirrored_data = Box::new(());
                let new_id = context.document.add_entity(mirrored_data);
                self.created_entities.push(new_id);
            }
        }

        // Delete source if requested
        if self.delete_source {
            for entity_id in &self.selection {
                context.document.remove_entity(entity_id);
            }
        }

        self.state = CommandState::Completed;
        Ok(())
    }

    fn undo(&mut self, context: &mut CommandContext) -> CommandResult {
        // Remove mirrored entities
        for entity_id in &self.created_entities {
            context.document.remove_entity(entity_id);
        }
        self.created_entities.clear();

        // Restore deleted source entities if applicable
        // (In real implementation, we would have stored them)

        Ok(())
    }

    fn state(&self) -> CommandState {
        self.state.clone()
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ==================== ARRAY COMMAND ====================

#[derive(Clone)]
pub enum ArrayType {
    Rectangular { rows: i32, columns: i32, row_spacing: f64, col_spacing: f64 },
    Polar { count: i32, angle: f64, center: Point },
}

#[derive(Clone)]
pub struct ArrayCommand {
    selection: Vec<EntityId>,
    array_type: Option<ArrayType>,
    created_entities: Vec<EntityId>,
    state: CommandState,
}

impl ArrayCommand {
    pub fn new() -> Self {
        Self {
            selection: Vec::new(),
            array_type: None,
            created_entities: Vec::new(),
            state: CommandState::AwaitingParameter("array type (R/P)".to_string()),
        }
    }
}

impl Command for ArrayCommand {
    fn name(&self) -> &str {
        "ARRAY"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["AR"]
    }

    fn description(&self) -> &str {
        "Create rectangular or polar array of entities"
    }

    fn usage(&self) -> &str {
        "ARRAY (select entities) R <rows> <cols> <row_spacing> <col_spacing>\n       \
         ARRAY (select entities) P <count> <angle> <center_x> <center_y>"
    }

    fn execute(&mut self, context: &mut CommandContext) -> CommandResult {
        if self.selection.is_empty() {
            self.selection = context.selection.entities.clone();
        }

        if self.selection.is_empty() {
            return Err(CommandError::InvalidSelection("No entities selected".to_string()));
        }

        let array_type = self.array_type.as_ref().ok_or_else(||
            CommandError::InvalidInput("Array type not specified".to_string()))?;

        // Create array based on type
        match array_type {
            ArrayType::Rectangular { rows, columns, row_spacing: _, col_spacing: _ } => {
                let count = (rows * columns) as usize;
                for _ in 0..count {
                    for entity_id in &self.selection {
                        if context.document.get_entity(entity_id).is_some() {
                            let copy_data = Box::new(());
                            let new_id = context.document.add_entity(copy_data);
                            self.created_entities.push(new_id);
                        }
                    }
                }
            }
            ArrayType::Polar { count, angle: _, center: _ } => {
                for _ in 0..*count {
                    for entity_id in &self.selection {
                        if context.document.get_entity(entity_id).is_some() {
                            let copy_data = Box::new(());
                            let new_id = context.document.add_entity(copy_data);
                            self.created_entities.push(new_id);
                        }
                    }
                }
            }
        }

        self.state = CommandState::Completed;
        Ok(())
    }

    fn undo(&mut self, context: &mut CommandContext) -> CommandResult {
        for entity_id in &self.created_entities {
            context.document.remove_entity(entity_id);
        }
        self.created_entities.clear();
        Ok(())
    }

    fn state(&self) -> CommandState {
        self.state.clone()
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ==================== OFFSET COMMAND ====================

#[derive(Clone)]
pub struct OffsetCommand {
    entity: Option<EntityId>,
    distance: Option<f64>,
    created_entity: Option<EntityId>,
    state: CommandState,
}

impl OffsetCommand {
    pub fn new() -> Self {
        Self {
            entity: None,
            distance: None,
            created_entity: None,
            state: CommandState::AwaitingParameter("offset distance".to_string()),
        }
    }
}

impl Command for OffsetCommand {
    fn name(&self) -> &str {
        "OFFSET"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["O"]
    }

    fn description(&self) -> &str {
        "Create parallel offset of entity"
    }

    fn usage(&self) -> &str {
        "OFFSET <distance> (select entity)"
    }

    fn execute(&mut self, context: &mut CommandContext) -> CommandResult {
        let entity_id = self.entity.ok_or_else(||
            CommandError::InvalidInput("No entity selected".to_string()))?;
        let distance = self.distance.ok_or_else(||
            CommandError::InvalidInput("Distance not specified".to_string()))?;

        if distance <= 0.0 {
            return Err(CommandError::InvalidInput("Distance must be positive".to_string()));
        }

        if context.document.get_entity(&entity_id).is_none() {
            return Err(CommandError::EntityNotFound("Entity not found".to_string()));
        }

        // Create offset entity
        let offset_data = Box::new((entity_id, distance));
        let new_id = context.document.add_entity(offset_data);
        self.created_entity = Some(new_id);

        self.state = CommandState::Completed;
        Ok(())
    }

    fn undo(&mut self, context: &mut CommandContext) -> CommandResult {
        if let Some(entity_id) = self.created_entity {
            context.document.remove_entity(&entity_id);
            self.created_entity = None;
            Ok(())
        } else {
            Err(CommandError::InvalidState("No entity to undo".to_string()))
        }
    }

    fn state(&self) -> CommandState {
        self.state.clone()
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ==================== TRIM COMMAND ====================

pub struct TrimCommand {
    cutting_edges: Vec<EntityId>,
    entities_to_trim: Vec<EntityId>,
    trimmed_portions: HashMap<EntityId, Box<dyn Any + Send + Sync>>,
    state: CommandState,
}

impl TrimCommand {
    pub fn new() -> Self {
        Self {
            cutting_edges: Vec::new(),
            entities_to_trim: Vec::new(),
            trimmed_portions: HashMap::new(),
            state: CommandState::AwaitingParameter("cutting edges".to_string()),
        }
    }
}

impl Command for TrimCommand {
    fn name(&self) -> &str {
        "TRIM"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["TR"]
    }

    fn description(&self) -> &str {
        "Trim entities at cutting edges"
    }

    fn usage(&self) -> &str {
        "TRIM (select cutting edges) (select entities to trim)"
    }

    fn execute(&mut self, context: &mut CommandContext) -> CommandResult {
        if self.cutting_edges.is_empty() {
            return Err(CommandError::InvalidSelection("No cutting edges selected".to_string()));
        }

        if self.entities_to_trim.is_empty() {
            return Err(CommandError::InvalidSelection("No entities to trim selected".to_string()));
        }

        // Trim entities (placeholder logic)
        for entity_id in &self.entities_to_trim {
            if context.document.get_entity(entity_id).is_some() {
                self.trimmed_portions.insert(*entity_id, Box::new(()));
            }
        }

        self.state = CommandState::Completed;
        Ok(())
    }

    fn undo(&mut self, _context: &mut CommandContext) -> CommandResult {
        // Restore trimmed portions
        self.trimmed_portions.clear();
        Ok(())
    }

    fn state(&self) -> CommandState {
        self.state.clone()
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(TrimCommand {
            cutting_edges: self.cutting_edges.clone(),
            entities_to_trim: self.entities_to_trim.clone(),
            trimmed_portions: HashMap::new(),
            state: self.state.clone(),
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ==================== EXTEND, FILLET, CHAMFER, BREAK, JOIN, EXPLODE ====================
// These follow similar patterns to TRIM - implementation abbreviated for brevity

#[derive(Clone)]
pub struct ExtendCommand {
    state: CommandState,
}

impl ExtendCommand {
    pub fn new() -> Self {
        Self {
            state: CommandState::AwaitingInput,
        }
    }
}

impl Command for ExtendCommand {
    fn name(&self) -> &str { "EXTEND" }
    fn aliases(&self) -> Vec<&str> { vec!["EX"] }
    fn description(&self) -> &str { "Extend entities to boundary edges" }
    fn usage(&self) -> &str { "EXTEND (select boundary) (select entities)" }
    fn execute(&mut self, _context: &mut CommandContext) -> CommandResult { Ok(()) }
    fn undo(&mut self, _context: &mut CommandContext) -> CommandResult { Ok(()) }
    fn state(&self) -> CommandState { self.state.clone() }
    fn clone_box(&self) -> Box<dyn Command> { Box::new(self.clone()) }
    fn as_any(&self) -> &dyn Any { self }
}

#[derive(Clone)]
pub struct FilletCommand {
    state: CommandState,
}

impl FilletCommand {
    pub fn new() -> Self {
        Self { state: CommandState::AwaitingInput }
    }
}

impl Command for FilletCommand {
    fn name(&self) -> &str { "FILLET" }
    fn aliases(&self) -> Vec<&str> { vec!["F"] }
    fn description(&self) -> &str { "Create rounded corner between two entities" }
    fn usage(&self) -> &str { "FILLET <radius> (select entities)" }
    fn execute(&mut self, _context: &mut CommandContext) -> CommandResult { Ok(()) }
    fn undo(&mut self, _context: &mut CommandContext) -> CommandResult { Ok(()) }
    fn state(&self) -> CommandState { self.state.clone() }
    fn clone_box(&self) -> Box<dyn Command> { Box::new(self.clone()) }
    fn as_any(&self) -> &dyn Any { self }
}

#[derive(Clone)]
pub struct ChamferCommand {
    state: CommandState,
}

impl ChamferCommand {
    pub fn new() -> Self {
        Self { state: CommandState::AwaitingInput }
    }
}

impl Command for ChamferCommand {
    fn name(&self) -> &str { "CHAMFER" }
    fn aliases(&self) -> Vec<&str> { vec!["CHA"] }
    fn description(&self) -> &str { "Create beveled corner between two entities" }
    fn usage(&self) -> &str { "CHAMFER <distance1> <distance2> (select entities)" }
    fn execute(&mut self, _context: &mut CommandContext) -> CommandResult { Ok(()) }
    fn undo(&mut self, _context: &mut CommandContext) -> CommandResult { Ok(()) }
    fn state(&self) -> CommandState { self.state.clone() }
    fn clone_box(&self) -> Box<dyn Command> { Box::new(self.clone()) }
    fn as_any(&self) -> &dyn Any { self }
}

#[derive(Clone)]
pub struct BreakCommand {
    state: CommandState,
}

impl BreakCommand {
    pub fn new() -> Self {
        Self { state: CommandState::AwaitingInput }
    }
}

impl Command for BreakCommand {
    fn name(&self) -> &str { "BREAK" }
    fn aliases(&self) -> Vec<&str> { vec!["BR"] }
    fn description(&self) -> &str { "Break entity into two parts" }
    fn usage(&self) -> &str { "BREAK (select entity) <point1> <point2>" }
    fn execute(&mut self, _context: &mut CommandContext) -> CommandResult { Ok(()) }
    fn undo(&mut self, _context: &mut CommandContext) -> CommandResult { Ok(()) }
    fn state(&self) -> CommandState { self.state.clone() }
    fn clone_box(&self) -> Box<dyn Command> { Box::new(self.clone()) }
    fn as_any(&self) -> &dyn Any { self }
}

#[derive(Clone)]
pub struct JoinCommand {
    state: CommandState,
}

impl JoinCommand {
    pub fn new() -> Self {
        Self { state: CommandState::AwaitingInput }
    }
}

impl Command for JoinCommand {
    fn name(&self) -> &str { "JOIN" }
    fn aliases(&self) -> Vec<&str> { vec!["J"] }
    fn description(&self) -> &str { "Join multiple entities into one" }
    fn usage(&self) -> &str { "JOIN (select entities)" }
    fn execute(&mut self, _context: &mut CommandContext) -> CommandResult { Ok(()) }
    fn undo(&mut self, _context: &mut CommandContext) -> CommandResult { Ok(()) }
    fn state(&self) -> CommandState { self.state.clone() }
    fn clone_box(&self) -> Box<dyn Command> { Box::new(self.clone()) }
    fn as_any(&self) -> &dyn Any { self }
}

#[derive(Clone)]
pub struct ExplodeCommand {
    state: CommandState,
}

impl ExplodeCommand {
    pub fn new() -> Self {
        Self { state: CommandState::AwaitingInput }
    }
}

impl Command for ExplodeCommand {
    fn name(&self) -> &str { "EXPLODE" }
    fn aliases(&self) -> Vec<&str> { vec!["X"] }
    fn description(&self) -> &str { "Break compound entity into individual parts" }
    fn usage(&self) -> &str { "EXPLODE (select entities)" }
    fn execute(&mut self, _context: &mut CommandContext) -> CommandResult { Ok(()) }
    fn undo(&mut self, _context: &mut CommandContext) -> CommandResult { Ok(()) }
    fn state(&self) -> CommandState { self.state.clone() }
    fn clone_box(&self) -> Box<dyn Command> { Box::new(self.clone()) }
    fn as_any(&self) -> &dyn Any { self }
}
