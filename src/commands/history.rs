// Undo/Redo history system for CADDY CAD
// Implements memory-efficient command history with grouping support

use super::command::{Command, CommandContext, CommandError, CommandMemento, CommandResult};
use std::collections::VecDeque;

/// Configuration for history limits
#[derive(Debug, Clone)]
pub struct HistoryConfig {
    /// Maximum number of undo levels (0 = unlimited)
    pub max_undo_levels: usize,
    /// Maximum memory usage for history in bytes (0 = unlimited)
    pub max_memory_bytes: usize,
    /// Enable automatic grouping of similar commands
    pub auto_group_similar: bool,
    /// Time window for grouping in milliseconds
    pub group_time_window_ms: u64,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            max_undo_levels: 100,
            max_memory_bytes: 100_000_000, // 100MB
            auto_group_similar: true,
            group_time_window_ms: 1000, // 1 second
        }
    }
}

/// Represents a single command execution in history
struct HistoryEntry {
    /// The executed command
    command: Box<dyn Command>,
    /// Memento storing state before execution
    memento: Option<CommandMemento>,
    /// Timestamp of execution
    timestamp: std::time::Instant,
    /// Description for UI display
    description: String,
    /// Whether this is part of a group
    group_id: Option<usize>,
}

impl HistoryEntry {
    fn new(command: Box<dyn Command>, memento: Option<CommandMemento>, description: String) -> Self {
        Self {
            command,
            memento,
            timestamp: std::time::Instant::now(),
            description,
            group_id: None,
        }
    }

    fn with_group(mut self, group_id: usize) -> Self {
        self.group_id = Some(group_id);
        self
    }
}

/// A group of commands that should be undone/redone together
struct CommandGroup {
    /// Group ID
    id: usize,
    /// Description of the group
    description: String,
    /// Entry indices that belong to this group
    entry_indices: Vec<usize>,
}

impl CommandGroup {
    fn new(id: usize, description: String) -> Self {
        Self {
            id,
            description,
            entry_indices: Vec::new(),
        }
    }
}

/// Undo/Redo stack with history management
pub struct UndoStack {
    /// Configuration
    config: HistoryConfig,
    /// Undo history (commands that can be undone)
    undo_stack: VecDeque<HistoryEntry>,
    /// Redo history (commands that can be redone)
    redo_stack: VecDeque<HistoryEntry>,
    /// Active command groups
    groups: Vec<CommandGroup>,
    /// Next group ID
    next_group_id: usize,
    /// Currently active group ID
    active_group_id: Option<usize>,
    /// Estimated memory usage in bytes
    estimated_memory_bytes: usize,
}

impl UndoStack {
    /// Create a new undo stack with default configuration
    pub fn new() -> Self {
        Self::with_config(HistoryConfig::default())
    }

    /// Create a new undo stack with custom configuration
    pub fn with_config(config: HistoryConfig) -> Self {
        Self {
            config,
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            groups: Vec::new(),
            next_group_id: 1,
            active_group_id: None,
            estimated_memory_bytes: 0,
        }
    }

    /// Add a command to the undo stack
    pub fn push(&mut self, command: Box<dyn Command>, memento: Option<CommandMemento>, description: String) {
        // Clear redo stack when new command is executed
        self.redo_stack.clear();

        // Create history entry
        let mut entry = HistoryEntry::new(command, memento, description);

        // Apply grouping if active
        if let Some(group_id) = self.active_group_id {
            entry = entry.with_group(group_id);
            if let Some(group) = self.groups.iter_mut().find(|g| g.id == group_id) {
                group.entry_indices.push(self.undo_stack.len());
            }
        }

        // Estimate memory usage (rough approximation)
        self.estimated_memory_bytes += std::mem::size_of::<HistoryEntry>();
        if entry.memento.is_some() {
            self.estimated_memory_bytes += 1024; // Rough estimate
        }

        // Add to stack
        self.undo_stack.push_back(entry);

        // Enforce limits
        self.enforce_limits();
    }

    /// Undo the last command
    pub fn undo(&mut self, context: &mut CommandContext) -> CommandResult<String> {
        if self.undo_stack.is_empty() {
            return Err(CommandError::InvalidState("Nothing to undo".to_string()));
        }

        // Get the last entry
        let mut entry = self.undo_stack.pop_back()
            .ok_or_else(|| CommandError::InvalidState("Undo stack empty".to_string()))?;

        // Check if this is part of a group - if so, undo entire group
        if let Some(group_id) = entry.group_id {
            // Put entry back temporarily
            self.undo_stack.push_back(entry);
            return self.undo_group(group_id, context);
        }

        let description = entry.description.clone();

        // Restore memento if available
        if let Some(memento) = entry.memento.take() {
            entry.command.restore_memento(memento, context)?;
        }

        // Execute undo
        entry.command.undo(context)?;

        // Move to redo stack
        self.redo_stack.push_back(entry);

        Ok(description)
    }

    /// Undo an entire command group
    fn undo_group(&mut self, group_id: usize, context: &mut CommandContext) -> CommandResult<String> {
        // Find the group
        let group = self.groups.iter()
            .find(|g| g.id == group_id)
            .ok_or_else(|| CommandError::InvalidState("Group not found".to_string()))?;

        let description = group.description.clone();
        let entry_count = group.entry_indices.len();

        // Undo all entries in reverse order
        for _ in 0..entry_count {
            if let Some(mut entry) = self.undo_stack.pop_back() {
                // Restore memento if available
                if let Some(memento) = entry.memento.take() {
                    entry.command.restore_memento(memento, context)?;
                }

                // Execute undo
                entry.command.undo(context)?;

                // Move to redo stack
                self.redo_stack.push_back(entry);
            }
        }

        Ok(description)
    }

    /// Redo the last undone command
    pub fn redo(&mut self, context: &mut CommandContext) -> CommandResult<String> {
        if self.redo_stack.is_empty() {
            return Err(CommandError::InvalidState("Nothing to redo".to_string()));
        }

        // Get the last undone entry
        let mut entry = self.redo_stack.pop_back()
            .ok_or_else(|| CommandError::InvalidState("Redo stack empty".to_string()))?;

        // Check if this is part of a group - if so, redo entire group
        if let Some(group_id) = entry.group_id {
            // Put entry back temporarily
            self.redo_stack.push_back(entry);
            return self.redo_group(group_id, context);
        }

        let description = entry.description.clone();

        // Execute redo
        entry.command.redo(context)?;

        // Move back to undo stack
        self.undo_stack.push_back(entry);

        Ok(description)
    }

    /// Redo an entire command group
    fn redo_group(&mut self, group_id: usize, context: &mut CommandContext) -> CommandResult<String> {
        // Find the group
        let group = self.groups.iter()
            .find(|g| g.id == group_id)
            .ok_or_else(|| CommandError::InvalidState("Group not found".to_string()))?;

        let description = group.description.clone();
        let entry_count = group.entry_indices.len();

        // Collect entries in correct order (redo stack is reversed)
        let mut entries = Vec::new();
        for _ in 0..entry_count {
            if let Some(entry) = self.redo_stack.pop_back() {
                entries.push(entry);
            }
        }

        // Redo in correct order
        for mut entry in entries.into_iter().rev() {
            entry.command.redo(context)?;
            self.undo_stack.push_back(entry);
        }

        Ok(description)
    }

    /// Start a command group (compound operation)
    pub fn begin_group(&mut self, description: impl Into<String>) {
        let group_id = self.next_group_id;
        self.next_group_id += 1;

        let group = CommandGroup::new(group_id, description.into());
        self.groups.push(group);
        self.active_group_id = Some(group_id);
    }

    /// End the current command group
    pub fn end_group(&mut self) {
        self.active_group_id = None;
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get the description of the next undo operation
    pub fn undo_description(&self) -> Option<&str> {
        self.undo_stack.back().map(|entry| entry.description.as_str())
    }

    /// Get the description of the next redo operation
    pub fn redo_description(&self) -> Option<&str> {
        self.redo_stack.back().map(|entry| entry.description.as_str())
    }

    /// Get the number of undo levels available
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the number of redo levels available
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.groups.clear();
        self.active_group_id = None;
        self.estimated_memory_bytes = 0;
    }

    /// Clear redo history only
    pub fn clear_redo(&mut self) {
        self.redo_stack.clear();
    }

    /// Get all undo descriptions for UI display
    pub fn get_undo_list(&self) -> Vec<String> {
        self.undo_stack.iter()
            .map(|entry| entry.description.clone())
            .collect()
    }

    /// Get all redo descriptions for UI display
    pub fn get_redo_list(&self) -> Vec<String> {
        self.redo_stack.iter()
            .map(|entry| entry.description.clone())
            .collect()
    }

    /// Enforce configured limits on history size
    fn enforce_limits(&mut self) {
        // Enforce max undo levels
        if self.config.max_undo_levels > 0 {
            while self.undo_stack.len() > self.config.max_undo_levels {
                if let Some(entry) = self.undo_stack.pop_front() {
                    // Update memory estimate
                    self.estimated_memory_bytes = self.estimated_memory_bytes.saturating_sub(
                        std::mem::size_of::<HistoryEntry>()
                    );
                    if entry.memento.is_some() {
                        self.estimated_memory_bytes = self.estimated_memory_bytes.saturating_sub(1024);
                    }
                }
            }
        }

        // Enforce max memory usage
        if self.config.max_memory_bytes > 0 {
            while self.estimated_memory_bytes > self.config.max_memory_bytes && !self.undo_stack.is_empty() {
                if let Some(entry) = self.undo_stack.pop_front() {
                    self.estimated_memory_bytes = self.estimated_memory_bytes.saturating_sub(
                        std::mem::size_of::<HistoryEntry>()
                    );
                    if entry.memento.is_some() {
                        self.estimated_memory_bytes = self.estimated_memory_bytes.saturating_sub(1024);
                    }
                }
            }
        }

        // Clean up groups that no longer have entries
        self.groups.retain(|group| {
            group.entry_indices.iter().any(|&idx| idx < self.undo_stack.len())
        });
    }

    /// Get estimated memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        self.estimated_memory_bytes
    }

    /// Get configuration
    pub fn config(&self) -> &HistoryConfig {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: HistoryConfig) {
        self.config = config;
        self.enforce_limits();
    }

    /// Undo multiple levels
    pub fn undo_multiple(&mut self, count: usize, context: &mut CommandContext) -> CommandResult<Vec<String>> {
        let mut descriptions = Vec::new();

        for _ in 0..count {
            if self.can_undo() {
                let desc = self.undo(context)?;
                descriptions.push(desc);
            } else {
                break;
            }
        }

        Ok(descriptions)
    }

    /// Redo multiple levels
    pub fn redo_multiple(&mut self, count: usize, context: &mut CommandContext) -> CommandResult<Vec<String>> {
        let mut descriptions = Vec::new();

        for _ in 0..count {
            if self.can_redo() {
                let desc = self.redo(context)?;
                descriptions.push(desc);
            } else {
                break;
            }
        }

        Ok(descriptions)
    }
}

impl Default for UndoStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undo_stack_creation() {
        let stack = UndoStack::new();
        assert!(!stack.can_undo());
        assert!(!stack.can_redo());
        assert_eq!(stack.undo_count(), 0);
        assert_eq!(stack.redo_count(), 0);
    }

    #[test]
    fn test_history_limits() {
        let config = HistoryConfig {
            max_undo_levels: 5,
            max_memory_bytes: 0,
            auto_group_similar: false,
            group_time_window_ms: 1000,
        };

        let stack = UndoStack::with_config(config);
        assert_eq!(stack.config().max_undo_levels, 5);
    }
}
