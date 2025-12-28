// Command trait and types for CADDY CAD system
// Provides the foundation for all commands with undo/redo support

use std::any::Any;
use std::collections::HashMap;
use std::fmt;

/// Result type for command operations
pub type CommandResult<T = ()> = Result<T, CommandError>;

/// Errors that can occur during command execution
#[derive(Debug, Clone)]
pub enum CommandError {
    /// Invalid input parameters
    InvalidInput(String),
    /// Invalid selection
    InvalidSelection(String),
    /// Operation not allowed in current state
    InvalidState(String),
    /// Geometric operation failed
    GeometricError(String),
    /// Entity not found
    EntityNotFound(String),
    /// Layer not found or invalid
    LayerError(String),
    /// Memory or resource limit exceeded
    ResourceLimit(String),
    /// User cancelled the operation
    Cancelled,
    /// Generic error
    Other(String),
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            CommandError::InvalidSelection(msg) => write!(f, "Invalid selection: {}", msg),
            CommandError::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
            CommandError::GeometricError(msg) => write!(f, "Geometric error: {}", msg),
            CommandError::EntityNotFound(msg) => write!(f, "Entity not found: {}", msg),
            CommandError::LayerError(msg) => write!(f, "Layer error: {}", msg),
            CommandError::ResourceLimit(msg) => write!(f, "Resource limit: {}", msg),
            CommandError::Cancelled => write!(f, "Operation cancelled"),
            CommandError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for CommandError {}

/// State for multi-step commands
#[derive(Debug, Clone, PartialEq)]
pub enum CommandState {
    /// Command is waiting for initial input
    AwaitingInput,
    /// Command is waiting for a specific parameter
    AwaitingParameter(String),
    /// Command is executing
    Executing,
    /// Command completed successfully
    Completed,
    /// Command was cancelled
    Cancelled,
    /// Command failed with error
    Failed(String),
}

/// Point in 2D or 3D space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn new_2d(x: f64, y: f64) -> Self {
        Self { x, y, z: 0.0 }
    }

    pub fn origin() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn distance_to(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

/// Entity ID for identifying geometric entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub u64);

impl EntityId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Selection set containing selected entities
#[derive(Debug, Clone)]
pub struct SelectionSet {
    pub entities: Vec<EntityId>,
}

impl SelectionSet {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }

    pub fn from_entities(entities: Vec<EntityId>) -> Self {
        Self { entities }
    }

    pub fn add(&mut self, entity: EntityId) {
        if !self.entities.contains(&entity) {
            self.entities.push(entity);
        }
    }

    pub fn remove(&mut self, entity: &EntityId) {
        self.entities.retain(|e| e != entity);
    }

    pub fn clear(&mut self) {
        self.entities.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn contains(&self, entity: &EntityId) -> bool {
        self.entities.contains(entity)
    }
}

impl Default for SelectionSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Document state containing all entities and layers
#[derive(Debug)]
pub struct Document {
    pub entities: HashMap<EntityId, Box<dyn Any + Send + Sync>>,
    pub current_layer: String,
    pub layers: Vec<String>,
    next_entity_id: u64,
}

impl Document {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            current_layer: "0".to_string(),
            layers: vec!["0".to_string()],
            next_entity_id: 1,
        }
    }

    pub fn allocate_entity_id(&mut self) -> EntityId {
        let id = EntityId::new(self.next_entity_id);
        self.next_entity_id += 1;
        id
    }

    pub fn add_entity(&mut self, entity: Box<dyn Any + Send + Sync>) -> EntityId {
        let id = self.allocate_entity_id();
        self.entities.insert(id, entity);
        id
    }

    pub fn remove_entity(&mut self, id: &EntityId) -> Option<Box<dyn Any + Send + Sync>> {
        self.entities.remove(id)
    }

    pub fn get_entity(&self, id: &EntityId) -> Option<&Box<dyn Any + Send + Sync>> {
        self.entities.get(id)
    }

    pub fn get_entity_mut(&mut self, id: &EntityId) -> Option<&mut Box<dyn Any + Send + Sync>> {
        self.entities.get_mut(id)
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

/// Context provided to commands during execution
pub struct CommandContext {
    /// The document being modified
    pub document: Document,
    /// Current selection set
    pub selection: SelectionSet,
    /// Command options/parameters
    pub options: HashMap<String, String>,
    /// Whether to prompt for missing inputs
    pub interactive: bool,
}

impl CommandContext {
    pub fn new(document: Document) -> Self {
        Self {
            document,
            selection: SelectionSet::new(),
            options: HashMap::new(),
            interactive: true,
        }
    }

    pub fn with_selection(mut self, selection: SelectionSet) -> Self {
        self.selection = selection;
        self
    }

    pub fn with_option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }

    pub fn get_option(&self, key: &str) -> Option<&String> {
        self.options.get(key)
    }

    pub fn get_option_or(&self, key: &str, default: &str) -> String {
        self.options.get(key).cloned().unwrap_or_else(|| default.to_string())
    }
}

/// Memento storing command state for undo/redo
pub struct CommandMemento {
    /// Stored state data (command-specific)
    pub data: Box<dyn Any + Send>,
    /// Description of the state
    pub description: String,
}

impl CommandMemento {
    pub fn new(data: Box<dyn Any + Send>, description: impl Into<String>) -> Self {
        Self {
            data,
            description: description.into(),
        }
    }
}

/// Core command trait that all commands must implement
pub trait Command: Send {
    /// Get the command name
    fn name(&self) -> &str;

    /// Get command aliases (e.g., "L" for "LINE")
    fn aliases(&self) -> Vec<&str> {
        Vec::new()
    }

    /// Get command description for help
    fn description(&self) -> &str {
        ""
    }

    /// Get command usage syntax
    fn usage(&self) -> &str {
        ""
    }

    /// Execute the command
    fn execute(&mut self, context: &mut CommandContext) -> CommandResult;

    /// Undo the command (restore previous state)
    fn undo(&mut self, context: &mut CommandContext) -> CommandResult;

    /// Redo the command (reapply after undo)
    fn redo(&mut self, context: &mut CommandContext) -> CommandResult {
        self.execute(context)
    }

    /// Check if command can be undone
    fn can_undo(&self) -> bool {
        true
    }

    /// Get current command state (for multi-step commands)
    fn state(&self) -> CommandState {
        CommandState::AwaitingInput
    }

    /// Process input for multi-step commands
    fn process_input(&mut self, _input: &str, _context: &mut CommandContext) -> CommandResult {
        Ok(())
    }

    /// Create a memento of current state before execution
    fn create_memento(&self, _context: &CommandContext) -> Option<CommandMemento> {
        None
    }

    /// Restore state from memento
    fn restore_memento(&mut self, _memento: CommandMemento, _context: &mut CommandContext) -> CommandResult {
        Ok(())
    }

    /// Clone the command into a Box
    fn clone_box(&self) -> Box<dyn Command>;

    /// Get command as Any for downcasting
    fn as_any(&self) -> &dyn Any;
}

/// Helper macro to implement Clone for Command trait objects
#[macro_export]
macro_rules! impl_command_clone {
    ($type:ty) => {
        impl Clone for $type {
            fn clone(&self) -> Self {
                Self { ..*self }
            }
        }
    };
}

/// Input type for commands
#[derive(Debug, Clone)]
pub enum CommandInput {
    /// Point coordinate input
    Point(Point),
    /// Distance/length input
    Distance(f64),
    /// Angle input (in degrees)
    Angle(f64),
    /// Text string input
    Text(String),
    /// Integer input
    Integer(i32),
    /// Option/flag input
    Option(String, String),
    /// Entity selection
    Selection(SelectionSet),
    /// Confirmation (yes/no)
    Confirm(bool),
}

/// Parameter type for command options
#[derive(Debug, Clone)]
pub struct CommandParameter {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<String>,
}

impl CommandParameter {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            required: false,
            default_value: None,
        }
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default_value = Some(default.into());
        self
    }
}
