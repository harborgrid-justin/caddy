// Drawing commands for CADDY CAD system
// Implements all primitive drawing commands (LINE, CIRCLE, ARC, etc.)

use super::command::*;
use std::any::Any;

// ==================== LINE COMMAND ====================

#[derive(Clone)]
pub struct LineCommand {
    start: Option<Point>,
    end: Option<Point>,
    created_entity: Option<EntityId>,
    state: CommandState,
}

impl LineCommand {
    pub fn new() -> Self {
        Self {
            start: None,
            end: None,
            created_entity: None,
            state: CommandState::AwaitingParameter("start point".to_string()),
        }
    }

    pub fn with_points(start: Point, end: Point) -> Self {
        Self {
            start: Some(start),
            end: Some(end),
            created_entity: None,
            state: CommandState::AwaitingInput,
        }
    }
}

impl Command for LineCommand {
    fn name(&self) -> &str {
        "LINE"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["L"]
    }

    fn description(&self) -> &str {
        "Create a line between two points"
    }

    fn usage(&self) -> &str {
        "LINE <x1> <y1> <x2> <y2> or LINE (interactive)"
    }

    fn execute(&mut self, context: &mut CommandContext) -> CommandResult {
        let start = self.start.ok_or_else(||
            CommandError::InvalidInput("Start point not specified".to_string()))?;
        let end = self.end.ok_or_else(||
            CommandError::InvalidInput("End point not specified".to_string()))?;

        // Create line entity (placeholder - actual geometry would be from geometry module)
        let line_data = Box::new((start, end));
        let entity_id = context.document.add_entity(line_data);
        self.created_entity = Some(entity_id);
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

    fn process_input(&mut self, input: &str, _context: &mut CommandContext) -> CommandResult {
        let parts: Vec<&str> = input.split_whitespace().collect();

        if self.start.is_none() {
            if parts.len() >= 2 {
                let x = parts[0].parse::<f64>()
                    .map_err(|_| CommandError::InvalidInput("Invalid X coordinate".to_string()))?;
                let y = parts[1].parse::<f64>()
                    .map_err(|_| CommandError::InvalidInput("Invalid Y coordinate".to_string()))?;
                self.start = Some(Point::new_2d(x, y));
                self.state = CommandState::AwaitingParameter("end point".to_string());
            }
        } else if self.end.is_none() {
            if parts.len() >= 2 {
                let x = parts[0].parse::<f64>()
                    .map_err(|_| CommandError::InvalidInput("Invalid X coordinate".to_string()))?;
                let y = parts[1].parse::<f64>()
                    .map_err(|_| CommandError::InvalidInput("Invalid Y coordinate".to_string()))?;
                self.end = Some(Point::new_2d(x, y));
                self.state = CommandState::Executing;
            }
        }

        Ok(())
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ==================== CIRCLE COMMAND ====================

#[derive(Clone)]
pub struct CircleCommand {
    center: Option<Point>,
    radius: Option<f64>,
    created_entity: Option<EntityId>,
    state: CommandState,
}

impl CircleCommand {
    pub fn new() -> Self {
        Self {
            center: None,
            radius: None,
            created_entity: None,
            state: CommandState::AwaitingParameter("center point".to_string()),
        }
    }

    pub fn with_center_radius(center: Point, radius: f64) -> Self {
        Self {
            center: Some(center),
            radius: Some(radius),
            created_entity: None,
            state: CommandState::AwaitingInput,
        }
    }
}

impl Command for CircleCommand {
    fn name(&self) -> &str {
        "CIRCLE"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["C"]
    }

    fn description(&self) -> &str {
        "Create a circle by center point and radius"
    }

    fn usage(&self) -> &str {
        "CIRCLE <cx> <cy> <radius> or CIRCLE (interactive)"
    }

    fn execute(&mut self, context: &mut CommandContext) -> CommandResult {
        let center = self.center.ok_or_else(||
            CommandError::InvalidInput("Center point not specified".to_string()))?;
        let radius = self.radius.ok_or_else(||
            CommandError::InvalidInput("Radius not specified".to_string()))?;

        if radius <= 0.0 {
            return Err(CommandError::InvalidInput("Radius must be positive".to_string()));
        }

        // Create circle entity
        let circle_data = Box::new((center, radius));
        let entity_id = context.document.add_entity(circle_data);
        self.created_entity = Some(entity_id);
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

// ==================== ARC COMMAND ====================

#[derive(Clone)]
pub struct ArcCommand {
    center: Option<Point>,
    radius: Option<f64>,
    start_angle: Option<f64>,
    end_angle: Option<f64>,
    created_entity: Option<EntityId>,
    state: CommandState,
}

impl ArcCommand {
    pub fn new() -> Self {
        Self {
            center: None,
            radius: None,
            start_angle: None,
            end_angle: None,
            created_entity: None,
            state: CommandState::AwaitingParameter("center point".to_string()),
        }
    }
}

impl Command for ArcCommand {
    fn name(&self) -> &str {
        "ARC"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["A"]
    }

    fn description(&self) -> &str {
        "Create an arc by center, radius, and angles"
    }

    fn usage(&self) -> &str {
        "ARC <cx> <cy> <radius> <start_angle> <end_angle>"
    }

    fn execute(&mut self, context: &mut CommandContext) -> CommandResult {
        let center = self.center.ok_or_else(||
            CommandError::InvalidInput("Center point not specified".to_string()))?;
        let radius = self.radius.ok_or_else(||
            CommandError::InvalidInput("Radius not specified".to_string()))?;
        let start_angle = self.start_angle.ok_or_else(||
            CommandError::InvalidInput("Start angle not specified".to_string()))?;
        let end_angle = self.end_angle.ok_or_else(||
            CommandError::InvalidInput("End angle not specified".to_string()))?;

        if radius <= 0.0 {
            return Err(CommandError::InvalidInput("Radius must be positive".to_string()));
        }

        // Create arc entity
        let arc_data = Box::new((center, radius, start_angle, end_angle));
        let entity_id = context.document.add_entity(arc_data);
        self.created_entity = Some(entity_id);
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

// ==================== RECTANGLE COMMAND ====================

#[derive(Clone)]
pub struct RectangleCommand {
    corner1: Option<Point>,
    corner2: Option<Point>,
    created_entities: Vec<EntityId>,
    state: CommandState,
}

impl RectangleCommand {
    pub fn new() -> Self {
        Self {
            corner1: None,
            corner2: None,
            created_entities: Vec::new(),
            state: CommandState::AwaitingParameter("first corner".to_string()),
        }
    }
}

impl Command for RectangleCommand {
    fn name(&self) -> &str {
        "RECTANGLE"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["REC", "RECT"]
    }

    fn description(&self) -> &str {
        "Create a rectangle by two corner points"
    }

    fn usage(&self) -> &str {
        "RECTANGLE <x1> <y1> <x2> <y2>"
    }

    fn execute(&mut self, context: &mut CommandContext) -> CommandResult {
        let c1 = self.corner1.ok_or_else(||
            CommandError::InvalidInput("First corner not specified".to_string()))?;
        let c2 = self.corner2.ok_or_else(||
            CommandError::InvalidInput("Second corner not specified".to_string()))?;

        // Create rectangle as four lines
        let p1 = c1;
        let p2 = Point::new(c2.x, c1.y, c1.z);
        let p3 = c2;
        let p4 = Point::new(c1.x, c2.y, c1.z);

        // Create four line entities
        let line1 = Box::new((p1, p2));
        let line2 = Box::new((p2, p3));
        let line3 = Box::new((p3, p4));
        let line4 = Box::new((p4, p1));

        self.created_entities.push(context.document.add_entity(line1));
        self.created_entities.push(context.document.add_entity(line2));
        self.created_entities.push(context.document.add_entity(line3));
        self.created_entities.push(context.document.add_entity(line4));

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

// ==================== POLYGON COMMAND ====================

#[derive(Clone)]
pub struct PolygonCommand {
    center: Option<Point>,
    sides: Option<i32>,
    radius: Option<f64>,
    created_entities: Vec<EntityId>,
    state: CommandState,
}

impl PolygonCommand {
    pub fn new() -> Self {
        Self {
            center: None,
            sides: None,
            radius: None,
            created_entities: Vec::new(),
            state: CommandState::AwaitingParameter("number of sides".to_string()),
        }
    }
}

impl Command for PolygonCommand {
    fn name(&self) -> &str {
        "POLYGON"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["POL"]
    }

    fn description(&self) -> &str {
        "Create a regular polygon"
    }

    fn usage(&self) -> &str {
        "POLYGON <sides> <cx> <cy> <radius>"
    }

    fn execute(&mut self, context: &mut CommandContext) -> CommandResult {
        let center = self.center.ok_or_else(||
            CommandError::InvalidInput("Center not specified".to_string()))?;
        let sides = self.sides.ok_or_else(||
            CommandError::InvalidInput("Number of sides not specified".to_string()))?;
        let radius = self.radius.ok_or_else(||
            CommandError::InvalidInput("Radius not specified".to_string()))?;

        if sides < 3 {
            return Err(CommandError::InvalidInput("Polygon must have at least 3 sides".to_string()));
        }
        if radius <= 0.0 {
            return Err(CommandError::InvalidInput("Radius must be positive".to_string()));
        }

        // Create polygon vertices
        let angle_step = 2.0 * std::f64::consts::PI / sides as f64;
        let mut vertices = Vec::new();

        for i in 0..sides {
            let angle = i as f64 * angle_step;
            let x = center.x + radius * angle.cos();
            let y = center.y + radius * angle.sin();
            vertices.push(Point::new(x, y, center.z));
        }

        // Create lines between vertices
        for i in 0..sides as usize {
            let next_i = (i + 1) % sides as usize;
            let line = Box::new((vertices[i], vertices[next_i]));
            let entity_id = context.document.add_entity(line);
            self.created_entities.push(entity_id);
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

// ==================== POLYLINE COMMAND ====================

#[derive(Clone)]
pub struct PolylineCommand {
    points: Vec<Point>,
    closed: bool,
    created_entity: Option<EntityId>,
    state: CommandState,
}

impl PolylineCommand {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            closed: false,
            created_entity: None,
            state: CommandState::AwaitingParameter("point".to_string()),
        }
    }
}

impl Command for PolylineCommand {
    fn name(&self) -> &str {
        "POLYLINE"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["PLINE", "PL"]
    }

    fn description(&self) -> &str {
        "Create a polyline through multiple points"
    }

    fn usage(&self) -> &str {
        "POLYLINE (interactive - press Enter to finish)"
    }

    fn execute(&mut self, context: &mut CommandContext) -> CommandResult {
        if self.points.len() < 2 {
            return Err(CommandError::InvalidInput("Polyline needs at least 2 points".to_string()));
        }

        // Create polyline entity
        let polyline_data = Box::new((self.points.clone(), self.closed));
        let entity_id = context.document.add_entity(polyline_data);
        self.created_entity = Some(entity_id);
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

// ==================== SPLINE COMMAND ====================

#[derive(Clone)]
pub struct SplineCommand {
    control_points: Vec<Point>,
    created_entity: Option<EntityId>,
    state: CommandState,
}

impl SplineCommand {
    pub fn new() -> Self {
        Self {
            control_points: Vec::new(),
            created_entity: None,
            state: CommandState::AwaitingParameter("control point".to_string()),
        }
    }
}

impl Command for SplineCommand {
    fn name(&self) -> &str {
        "SPLINE"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["SPL"]
    }

    fn description(&self) -> &str {
        "Create a spline curve through control points"
    }

    fn usage(&self) -> &str {
        "SPLINE (interactive - press Enter to finish)"
    }

    fn execute(&mut self, context: &mut CommandContext) -> CommandResult {
        if self.control_points.len() < 2 {
            return Err(CommandError::InvalidInput("Spline needs at least 2 control points".to_string()));
        }

        // Create spline entity
        let spline_data = Box::new(self.control_points.clone());
        let entity_id = context.document.add_entity(spline_data);
        self.created_entity = Some(entity_id);
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

// ==================== ELLIPSE COMMAND ====================

#[derive(Clone)]
pub struct EllipseCommand {
    center: Option<Point>,
    major_radius: Option<f64>,
    minor_radius: Option<f64>,
    rotation: f64,
    created_entity: Option<EntityId>,
    state: CommandState,
}

impl EllipseCommand {
    pub fn new() -> Self {
        Self {
            center: None,
            major_radius: None,
            minor_radius: None,
            rotation: 0.0,
            created_entity: None,
            state: CommandState::AwaitingParameter("center point".to_string()),
        }
    }
}

impl Command for EllipseCommand {
    fn name(&self) -> &str {
        "ELLIPSE"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["EL"]
    }

    fn description(&self) -> &str {
        "Create an ellipse"
    }

    fn usage(&self) -> &str {
        "ELLIPSE <cx> <cy> <major_radius> <minor_radius> [rotation]"
    }

    fn execute(&mut self, context: &mut CommandContext) -> CommandResult {
        let center = self.center.ok_or_else(||
            CommandError::InvalidInput("Center not specified".to_string()))?;
        let major_radius = self.major_radius.ok_or_else(||
            CommandError::InvalidInput("Major radius not specified".to_string()))?;
        let minor_radius = self.minor_radius.ok_or_else(||
            CommandError::InvalidInput("Minor radius not specified".to_string()))?;

        if major_radius <= 0.0 || minor_radius <= 0.0 {
            return Err(CommandError::InvalidInput("Radii must be positive".to_string()));
        }

        // Create ellipse entity
        let ellipse_data = Box::new((center, major_radius, minor_radius, self.rotation));
        let entity_id = context.document.add_entity(ellipse_data);
        self.created_entity = Some(entity_id);
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

// ==================== TEXT COMMAND ====================

#[derive(Clone)]
pub struct TextCommand {
    position: Option<Point>,
    text: Option<String>,
    height: f64,
    rotation: f64,
    created_entity: Option<EntityId>,
    state: CommandState,
}

impl TextCommand {
    pub fn new() -> Self {
        Self {
            position: None,
            text: None,
            height: 1.0,
            rotation: 0.0,
            created_entity: None,
            state: CommandState::AwaitingParameter("insertion point".to_string()),
        }
    }
}

impl Command for TextCommand {
    fn name(&self) -> &str {
        "TEXT"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["T"]
    }

    fn description(&self) -> &str {
        "Create text annotation"
    }

    fn usage(&self) -> &str {
        "TEXT <x> <y> <height> <text>"
    }

    fn execute(&mut self, context: &mut CommandContext) -> CommandResult {
        let position = self.position.ok_or_else(||
            CommandError::InvalidInput("Position not specified".to_string()))?;
        let text = self.text.as_ref().ok_or_else(||
            CommandError::InvalidInput("Text not specified".to_string()))?;

        if self.height <= 0.0 {
            return Err(CommandError::InvalidInput("Height must be positive".to_string()));
        }

        // Create text entity
        let text_data = Box::new((position, text.clone(), self.height, self.rotation));
        let entity_id = context.document.add_entity(text_data);
        self.created_entity = Some(entity_id);
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
