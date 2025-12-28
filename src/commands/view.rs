// View commands for CADDY CAD system
// Implements viewport and view manipulation commands (ZOOM, PAN, REGEN, VIEW)

use super::command::*;
use std::any::Any;
use std::collections::HashMap;

// ==================== ZOOM COMMAND ====================

#[derive(Debug, Clone, PartialEq)]
pub enum ZoomMode {
    /// Zoom in by a factor
    In(f64),
    /// Zoom out by a factor
    Out(f64),
    /// Zoom to show all entities
    Extents,
    /// Zoom to a window defined by two corners
    Window { corner1: Point, corner2: Point },
    /// Zoom to a specific scale
    Scale(f64),
    /// Zoom to center on a point
    Center { center: Point, scale: f64 },
    /// Zoom to previous view
    Previous,
    /// Real-time zoom (dynamic)
    Realtime,
}

#[derive(Clone)]
pub struct ZoomCommand {
    mode: Option<ZoomMode>,
    previous_zoom: Option<(Point, f64)>,
    state: CommandState,
}

impl ZoomCommand {
    pub fn new() -> Self {
        Self {
            mode: None,
            previous_zoom: None,
            state: CommandState::AwaitingParameter("zoom mode".to_string()),
        }
    }

    pub fn in_mode() -> Self {
        Self {
            mode: Some(ZoomMode::In(2.0)),
            previous_zoom: None,
            state: CommandState::AwaitingInput,
        }
    }

    pub fn out_mode() -> Self {
        Self {
            mode: Some(ZoomMode::Out(0.5)),
            previous_zoom: None,
            state: CommandState::AwaitingInput,
        }
    }

    pub fn extents_mode() -> Self {
        Self {
            mode: Some(ZoomMode::Extents),
            previous_zoom: None,
            state: CommandState::AwaitingInput,
        }
    }

    pub fn window_mode(corner1: Point, corner2: Point) -> Self {
        Self {
            mode: Some(ZoomMode::Window { corner1, corner2 }),
            previous_zoom: None,
            state: CommandState::AwaitingInput,
        }
    }
}

impl Command for ZoomCommand {
    fn name(&self) -> &str {
        "ZOOM"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["Z"]
    }

    fn description(&self) -> &str {
        "Change the magnification of the viewport"
    }

    fn usage(&self) -> &str {
        "ZOOM IN | OUT | EXTENTS | WINDOW <x1> <y1> <x2> <y2> | <scale>"
    }

    fn execute(&mut self, context: &mut CommandContext) -> CommandResult {
        let mode = self.mode.as_ref().ok_or_else(||
            CommandError::InvalidInput("Zoom mode not specified".to_string()))?;

        // Store previous zoom for undo
        self.previous_zoom = Some((Point::origin(), 1.0));

        match mode {
            ZoomMode::In(factor) => {
                // Apply zoom in
                println!("Zooming in by factor: {}", factor);
            }
            ZoomMode::Out(factor) => {
                // Apply zoom out
                println!("Zooming out by factor: {}", factor);
            }
            ZoomMode::Extents => {
                // Calculate bounding box of all entities
                if context.document.entities.is_empty() {
                    return Err(CommandError::InvalidState("No entities to zoom to".to_string()));
                }
                println!("Zooming to extents");
            }
            ZoomMode::Window { corner1, corner2 } => {
                // Zoom to window
                println!("Zooming to window: {:?} to {:?}", corner1, corner2);
            }
            ZoomMode::Scale(scale) => {
                println!("Zooming to scale: {}", scale);
            }
            ZoomMode::Center { center, scale } => {
                println!("Zooming to center: {:?}, scale: {}", center, scale);
            }
            ZoomMode::Previous => {
                println!("Zooming to previous view");
            }
            ZoomMode::Realtime => {
                println!("Starting realtime zoom");
            }
        }

        self.state = CommandState::Completed;
        Ok(())
    }

    fn undo(&mut self, _context: &mut CommandContext) -> CommandResult {
        if let Some((center, scale)) = self.previous_zoom {
            println!("Restoring zoom: center {:?}, scale {}", center, scale);
            Ok(())
        } else {
            Err(CommandError::InvalidState("No previous zoom to restore".to_string()))
        }
    }

    fn state(&self) -> CommandState {
        self.state.clone()
    }

    fn process_input(&mut self, input: &str, _context: &mut CommandContext) -> CommandResult {
        let input_upper = input.to_uppercase();

        match input_upper.as_str() {
            "IN" | "I" => {
                self.mode = Some(ZoomMode::In(2.0));
                self.state = CommandState::Executing;
            }
            "OUT" | "O" => {
                self.mode = Some(ZoomMode::Out(0.5));
                self.state = CommandState::Executing;
            }
            "EXTENTS" | "E" | "ALL" | "A" => {
                self.mode = Some(ZoomMode::Extents);
                self.state = CommandState::Executing;
            }
            "WINDOW" | "W" => {
                self.state = CommandState::AwaitingParameter("first corner".to_string());
            }
            "PREVIOUS" | "P" => {
                self.mode = Some(ZoomMode::Previous);
                self.state = CommandState::Executing;
            }
            "REALTIME" | "R" => {
                self.mode = Some(ZoomMode::Realtime);
                self.state = CommandState::Executing;
            }
            _ => {
                // Try to parse as scale factor
                if let Ok(scale) = input.parse::<f64>() {
                    self.mode = Some(ZoomMode::Scale(scale));
                    self.state = CommandState::Executing;
                } else {
                    return Err(CommandError::InvalidInput(format!("Invalid zoom mode: {}", input)));
                }
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

// ==================== PAN COMMAND ====================

#[derive(Clone)]
pub struct PanCommand {
    from_point: Option<Point>,
    to_point: Option<Point>,
    previous_pan: Option<(f64, f64)>,
    state: CommandState,
}

impl PanCommand {
    pub fn new() -> Self {
        Self {
            from_point: None,
            to_point: None,
            previous_pan: None,
            state: CommandState::AwaitingParameter("base point".to_string()),
        }
    }

    pub fn realtime() -> Self {
        Self {
            from_point: None,
            to_point: None,
            previous_pan: None,
            state: CommandState::Executing,
        }
    }
}

impl Command for PanCommand {
    fn name(&self) -> &str {
        "PAN"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["P"]
    }

    fn description(&self) -> &str {
        "Pan the view by displacement"
    }

    fn usage(&self) -> &str {
        "PAN <from_x> <from_y> <to_x> <to_y> or PAN (realtime)"
    }

    fn execute(&mut self, _context: &mut CommandContext) -> CommandResult {
        let from = self.from_point.unwrap_or_else(|| Point::origin());
        let to = self.to_point.unwrap_or_else(|| Point::origin());

        let dx = to.x - from.x;
        let dy = to.y - from.y;

        // Store previous pan for undo
        self.previous_pan = Some((dx, dy));

        println!("Panning view by: ({}, {})", dx, dy);

        self.state = CommandState::Completed;
        Ok(())
    }

    fn undo(&mut self, _context: &mut CommandContext) -> CommandResult {
        if let Some((dx, dy)) = self.previous_pan {
            println!("Restoring pan: ({}, {})", -dx, -dy);
            Ok(())
        } else {
            Err(CommandError::InvalidState("No previous pan to restore".to_string()))
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

// ==================== REGEN COMMAND ====================

#[derive(Clone)]
pub struct RegenCommand {
    state: CommandState,
}

impl RegenCommand {
    pub fn new() -> Self {
        Self {
            state: CommandState::AwaitingInput,
        }
    }
}

impl Command for RegenCommand {
    fn name(&self) -> &str {
        "REGEN"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["RE"]
    }

    fn description(&self) -> &str {
        "Regenerate the drawing display"
    }

    fn usage(&self) -> &str {
        "REGEN"
    }

    fn execute(&mut self, _context: &mut CommandContext) -> CommandResult {
        println!("Regenerating drawing...");
        self.state = CommandState::Completed;
        Ok(())
    }

    fn undo(&mut self, _context: &mut CommandContext) -> CommandResult {
        // Regeneration cannot be undone
        Ok(())
    }

    fn can_undo(&self) -> bool {
        false
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

// ==================== REDRAW COMMAND ====================

#[derive(Clone)]
pub struct RedrawCommand {
    state: CommandState,
}

impl RedrawCommand {
    pub fn new() -> Self {
        Self {
            state: CommandState::AwaitingInput,
        }
    }
}

impl Command for RedrawCommand {
    fn name(&self) -> &str {
        "REDRAW"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["R"]
    }

    fn description(&self) -> &str {
        "Redraw the current viewport"
    }

    fn usage(&self) -> &str {
        "REDRAW"
    }

    fn execute(&mut self, _context: &mut CommandContext) -> CommandResult {
        println!("Redrawing viewport...");
        self.state = CommandState::Completed;
        Ok(())
    }

    fn undo(&mut self, _context: &mut CommandContext) -> CommandResult {
        // Redraw cannot be undone
        Ok(())
    }

    fn can_undo(&self) -> bool {
        false
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

// ==================== VIEW COMMAND ====================

#[derive(Debug, Clone)]
pub struct NamedView {
    pub name: String,
    pub center: Point,
    pub scale: f64,
    pub rotation: f64,
}

#[derive(Clone)]
pub struct ViewCommand {
    view_name: Option<String>,
    saved_views: HashMap<String, NamedView>,
    action: ViewAction,
    state: CommandState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewAction {
    Save,
    Restore,
    Delete,
    List,
}

impl ViewCommand {
    pub fn new() -> Self {
        Self {
            view_name: None,
            saved_views: HashMap::new(),
            action: ViewAction::List,
            state: CommandState::AwaitingParameter("action".to_string()),
        }
    }

    pub fn save(name: String) -> Self {
        Self {
            view_name: Some(name),
            saved_views: HashMap::new(),
            action: ViewAction::Save,
            state: CommandState::AwaitingInput,
        }
    }

    pub fn restore(name: String) -> Self {
        Self {
            view_name: Some(name),
            saved_views: HashMap::new(),
            action: ViewAction::Restore,
            state: CommandState::AwaitingInput,
        }
    }
}

impl Command for ViewCommand {
    fn name(&self) -> &str {
        "VIEW"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["V"]
    }

    fn description(&self) -> &str {
        "Save and restore named views"
    }

    fn usage(&self) -> &str {
        "VIEW SAVE <name> | RESTORE <name> | DELETE <name> | LIST"
    }

    fn execute(&mut self, _context: &mut CommandContext) -> CommandResult {
        match self.action {
            ViewAction::Save => {
                let name = self.view_name.as_ref().ok_or_else(||
                    CommandError::InvalidInput("View name not specified".to_string()))?;

                let view = NamedView {
                    name: name.clone(),
                    center: Point::origin(),
                    scale: 1.0,
                    rotation: 0.0,
                };

                self.saved_views.insert(name.clone(), view);
                println!("View '{}' saved", name);
            }
            ViewAction::Restore => {
                let name = self.view_name.as_ref().ok_or_else(||
                    CommandError::InvalidInput("View name not specified".to_string()))?;

                if let Some(view) = self.saved_views.get(name) {
                    println!("Restoring view '{}'", name);
                    println!("  Center: {:?}", view.center);
                    println!("  Scale: {}", view.scale);
                    println!("  Rotation: {}", view.rotation);
                } else {
                    return Err(CommandError::InvalidInput(format!("View '{}' not found", name)));
                }
            }
            ViewAction::Delete => {
                let name = self.view_name.as_ref().ok_or_else(||
                    CommandError::InvalidInput("View name not specified".to_string()))?;

                if self.saved_views.remove(name).is_some() {
                    println!("View '{}' deleted", name);
                } else {
                    return Err(CommandError::InvalidInput(format!("View '{}' not found", name)));
                }
            }
            ViewAction::List => {
                println!("Saved views:");
                for (name, view) in &self.saved_views {
                    println!("  {}: center={:?}, scale={}", name, view.center, view.scale);
                }
            }
        }

        self.state = CommandState::Completed;
        Ok(())
    }

    fn undo(&mut self, _context: &mut CommandContext) -> CommandResult {
        // View commands have limited undo support
        Ok(())
    }

    fn state(&self) -> CommandState {
        self.state.clone()
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(ViewCommand {
            view_name: self.view_name.clone(),
            saved_views: HashMap::new(),
            action: self.action.clone(),
            state: self.state.clone(),
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ==================== VIEWRES COMMAND ====================

#[derive(Clone)]
pub struct ViewResCommand {
    resolution: Option<i32>,
    state: CommandState,
}

impl ViewResCommand {
    pub fn new() -> Self {
        Self {
            resolution: None,
            state: CommandState::AwaitingParameter("resolution".to_string()),
        }
    }
}

impl Command for ViewResCommand {
    fn name(&self) -> &str {
        "VIEWRES"
    }

    fn aliases(&self) -> Vec<&str> {
        vec![]
    }

    fn description(&self) -> &str {
        "Set viewport resolution for circles and arcs"
    }

    fn usage(&self) -> &str {
        "VIEWRES <1-20000>"
    }

    fn execute(&mut self, _context: &mut CommandContext) -> CommandResult {
        let resolution = self.resolution.ok_or_else(||
            CommandError::InvalidInput("Resolution not specified".to_string()))?;

        if !(1..=20000).contains(&resolution) {
            return Err(CommandError::InvalidInput("Resolution must be between 1 and 20000".to_string()));
        }

        println!("Viewport resolution set to: {}", resolution);

        self.state = CommandState::Completed;
        Ok(())
    }

    fn undo(&mut self, _context: &mut CommandContext) -> CommandResult {
        Ok(())
    }

    fn can_undo(&self) -> bool {
        false
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
