// Editing commands for CADDY CAD system
// Implements editing operations (ERASE, UNDO, REDO, CUT, COPY, PASTE)

use super::command::*;
use std::any::Any;
use std::collections::HashMap;

// ==================== ERASE COMMAND ====================

pub struct EraseCommand {
    selection: Vec<EntityId>,
    deleted_entities: HashMap<EntityId, Box<dyn Any + Send + Sync>>,
    state: CommandState,
}

impl EraseCommand {
    pub fn new() -> Self {
        Self {
            selection: Vec::new(),
            deleted_entities: HashMap::new(),
            state: CommandState::AwaitingInput,
        }
    }

    pub fn with_selection(selection: Vec<EntityId>) -> Self {
        Self {
            selection,
            deleted_entities: HashMap::new(),
            state: CommandState::AwaitingInput,
        }
    }
}

impl Command for EraseCommand {
    fn name(&self) -> &str {
        "ERASE"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["E", "DELETE", "DEL"]
    }

    fn description(&self) -> &str {
        "Delete selected entities"
    }

    fn usage(&self) -> &str {
        "ERASE (select entities) or DELETE (select entities)"
    }

    fn execute(&mut self, context: &mut CommandContext) -> CommandResult {
        if self.selection.is_empty() {
            self.selection = context.selection.entities.clone();
        }

        if self.selection.is_empty() {
            return Err(CommandError::InvalidSelection("No entities selected".to_string()));
        }

        // Delete entities and store them for undo
        for entity_id in &self.selection {
            if let Some(entity) = context.document.remove_entity(entity_id) {
                self.deleted_entities.insert(*entity_id, entity);
            }
        }

        // Clear selection
        context.selection.clear();

        self.state = CommandState::Completed;
        Ok(())
    }

    fn undo(&mut self, context: &mut CommandContext) -> CommandResult {
        // Restore deleted entities
        for (entity_id, entity) in self.deleted_entities.drain() {
            context.document.entities.insert(entity_id, entity);
        }

        self.state = CommandState::AwaitingInput;
        Ok(())
    }

    fn state(&self) -> CommandState {
        self.state.clone()
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(EraseCommand {
            selection: self.selection.clone(),
            deleted_entities: HashMap::new(),
            state: self.state.clone(),
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ==================== UNDO COMMAND ====================

#[derive(Clone)]
pub struct UndoCommand {
    state: CommandState,
}

impl UndoCommand {
    pub fn new() -> Self {
        Self {
            state: CommandState::AwaitingInput,
        }
    }
}

impl Command for UndoCommand {
    fn name(&self) -> &str {
        "UNDO"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["U"]
    }

    fn description(&self) -> &str {
        "Undo the last command"
    }

    fn usage(&self) -> &str {
        "UNDO or U"
    }

    fn execute(&mut self, _context: &mut CommandContext) -> CommandResult {
        // Undo is handled by the command processor
        // This command itself doesn't perform the undo operation
        self.state = CommandState::Completed;
        Ok(())
    }

    fn undo(&mut self, _context: &mut CommandContext) -> CommandResult {
        // Cannot undo an undo command
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

// ==================== REDO COMMAND ====================

#[derive(Clone)]
pub struct RedoCommand {
    state: CommandState,
}

impl RedoCommand {
    pub fn new() -> Self {
        Self {
            state: CommandState::AwaitingInput,
        }
    }
}

impl Command for RedoCommand {
    fn name(&self) -> &str {
        "REDO"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["R"]
    }

    fn description(&self) -> &str {
        "Redo the last undone command"
    }

    fn usage(&self) -> &str {
        "REDO or R"
    }

    fn execute(&mut self, _context: &mut CommandContext) -> CommandResult {
        // Redo is handled by the command processor
        self.state = CommandState::Completed;
        Ok(())
    }

    fn undo(&mut self, _context: &mut CommandContext) -> CommandResult {
        // Cannot undo a redo command
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

// ==================== CUT COMMAND ====================

pub struct CutCommand {
    selection: Vec<EntityId>,
    deleted_entities: HashMap<EntityId, Box<dyn Any + Send + Sync>>,
    clipboard_data: Vec<u8>,
    state: CommandState,
}

impl CutCommand {
    pub fn new() -> Self {
        Self {
            selection: Vec::new(),
            deleted_entities: HashMap::new(),
            clipboard_data: Vec::new(),
            state: CommandState::AwaitingInput,
        }
    }
}

impl Command for CutCommand {
    fn name(&self) -> &str {
        "CUT"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["CUTCLIP"]
    }

    fn description(&self) -> &str {
        "Cut selected entities to clipboard"
    }

    fn usage(&self) -> &str {
        "CUT (select entities)"
    }

    fn execute(&mut self, context: &mut CommandContext) -> CommandResult {
        if self.selection.is_empty() {
            self.selection = context.selection.entities.clone();
        }

        if self.selection.is_empty() {
            return Err(CommandError::InvalidSelection("No entities selected".to_string()));
        }

        // Serialize entities to clipboard (placeholder)
        // In real implementation, this would serialize the actual entity data
        self.clipboard_data = vec![0u8; 1024];

        // Delete entities
        for entity_id in &self.selection {
            if let Some(entity) = context.document.remove_entity(entity_id) {
                self.deleted_entities.insert(*entity_id, entity);
            }
        }

        // Clear selection
        context.selection.clear();

        self.state = CommandState::Completed;
        Ok(())
    }

    fn undo(&mut self, context: &mut CommandContext) -> CommandResult {
        // Restore deleted entities
        for (entity_id, entity) in self.deleted_entities.drain() {
            context.document.entities.insert(entity_id, entity);
        }

        self.state = CommandState::AwaitingInput;
        Ok(())
    }

    fn state(&self) -> CommandState {
        self.state.clone()
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(CutCommand {
            selection: self.selection.clone(),
            deleted_entities: HashMap::new(),
            clipboard_data: Vec::new(),
            state: self.state.clone(),
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ==================== COPY TO CLIPBOARD COMMAND ====================

#[derive(Clone)]
pub struct CopyToClipboardCommand {
    selection: Vec<EntityId>,
    clipboard_data: Vec<u8>,
    state: CommandState,
}

impl CopyToClipboardCommand {
    pub fn new() -> Self {
        Self {
            selection: Vec::new(),
            clipboard_data: Vec::new(),
            state: CommandState::AwaitingInput,
        }
    }
}

impl Command for CopyToClipboardCommand {
    fn name(&self) -> &str {
        "COPYCLIP"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["CTRL+C"]
    }

    fn description(&self) -> &str {
        "Copy selected entities to clipboard"
    }

    fn usage(&self) -> &str {
        "COPYCLIP (select entities)"
    }

    fn execute(&mut self, context: &mut CommandContext) -> CommandResult {
        if self.selection.is_empty() {
            self.selection = context.selection.entities.clone();
        }

        if self.selection.is_empty() {
            return Err(CommandError::InvalidSelection("No entities selected".to_string()));
        }

        // Serialize entities to clipboard (placeholder)
        // In real implementation, this would serialize the actual entity data
        self.clipboard_data = vec![0u8; 1024];

        self.state = CommandState::Completed;
        Ok(())
    }

    fn undo(&mut self, _context: &mut CommandContext) -> CommandResult {
        // Copy to clipboard cannot be undone
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

// ==================== PASTE COMMAND ====================

#[derive(Clone)]
pub struct PasteCommand {
    insertion_point: Option<Point>,
    clipboard_data: Vec<u8>,
    created_entities: Vec<EntityId>,
    state: CommandState,
}

impl PasteCommand {
    pub fn new() -> Self {
        Self {
            insertion_point: None,
            clipboard_data: Vec::new(),
            created_entities: Vec::new(),
            state: CommandState::AwaitingParameter("insertion point".to_string()),
        }
    }

    pub fn with_clipboard_data(clipboard_data: Vec<u8>) -> Self {
        Self {
            insertion_point: None,
            clipboard_data,
            created_entities: Vec::new(),
            state: CommandState::AwaitingParameter("insertion point".to_string()),
        }
    }
}

impl Command for PasteCommand {
    fn name(&self) -> &str {
        "PASTE"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["PASTECLIP", "CTRL+V"]
    }

    fn description(&self) -> &str {
        "Paste entities from clipboard"
    }

    fn usage(&self) -> &str {
        "PASTE <x> <y> or PASTE (interactive)"
    }

    fn execute(&mut self, context: &mut CommandContext) -> CommandResult {
        let insertion_point = self.insertion_point.unwrap_or_else(|| Point::origin());

        if self.clipboard_data.is_empty() {
            return Err(CommandError::InvalidState("Clipboard is empty".to_string()));
        }

        // Deserialize entities from clipboard and add to document (placeholder)
        // In real implementation, this would deserialize and transform entities
        let pasted_data = Box::new(insertion_point);
        let entity_id = context.document.add_entity(pasted_data);
        self.created_entities.push(entity_id);

        self.state = CommandState::Completed;
        Ok(())
    }

    fn undo(&mut self, context: &mut CommandContext) -> CommandResult {
        for entity_id in &self.created_entities {
            context.document.remove_entity(entity_id);
        }
        self.created_entities.clear();

        self.state = CommandState::AwaitingParameter("insertion point".to_string());
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

// ==================== SELECT ALL COMMAND ====================

#[derive(Clone)]
pub struct SelectAllCommand {
    state: CommandState,
}

impl SelectAllCommand {
    pub fn new() -> Self {
        Self {
            state: CommandState::AwaitingInput,
        }
    }
}

impl Command for SelectAllCommand {
    fn name(&self) -> &str {
        "SELECTALL"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["CTRL+A"]
    }

    fn description(&self) -> &str {
        "Select all entities in the drawing"
    }

    fn usage(&self) -> &str {
        "SELECTALL"
    }

    fn execute(&mut self, context: &mut CommandContext) -> CommandResult {
        // Select all entities
        context.selection.entities.clear();
        for entity_id in context.document.entities.keys() {
            context.selection.add(*entity_id);
        }

        self.state = CommandState::Completed;
        Ok(())
    }

    fn undo(&mut self, context: &mut CommandContext) -> CommandResult {
        // Clear selection
        context.selection.clear();
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

// ==================== CLEAR SELECTION COMMAND ====================

#[derive(Clone)]
pub struct ClearSelectionCommand {
    state: CommandState,
}

impl ClearSelectionCommand {
    pub fn new() -> Self {
        Self {
            state: CommandState::AwaitingInput,
        }
    }
}

impl Command for ClearSelectionCommand {
    fn name(&self) -> &str {
        "CLEARSELECTION"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["ESC"]
    }

    fn description(&self) -> &str {
        "Clear current selection"
    }

    fn usage(&self) -> &str {
        "CLEARSELECTION or press ESC"
    }

    fn execute(&mut self, context: &mut CommandContext) -> CommandResult {
        context.selection.clear();
        self.state = CommandState::Completed;
        Ok(())
    }

    fn undo(&mut self, _context: &mut CommandContext) -> CommandResult {
        // Cannot undo clearing selection
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
