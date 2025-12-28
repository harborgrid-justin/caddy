// Command processor for CADDY CAD system
// Handles command parsing, execution, queueing, and input processing

use super::command::{Command, CommandContext, CommandError, CommandResult, CommandState, Point};
use super::history::UndoStack;
use super::registry::CommandRegistry;
use std::collections::VecDeque;

/// Input parser for command arguments
pub struct InputParser {
    tokens: Vec<String>,
    position: usize,
}

impl InputParser {
    /// Create a new input parser from a string
    pub fn new(input: &str) -> Self {
        let tokens = Self::tokenize(input);
        Self {
            tokens,
            position: 0,
        }
    }

    /// Tokenize input string
    fn tokenize(input: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current_token = String::new();
        let mut in_quotes = false;

        for ch in input.chars() {
            match ch {
                '"' => {
                    in_quotes = !in_quotes;
                }
                ' ' | '\t' | ',' if !in_quotes => {
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                }
                _ => {
                    current_token.push(ch);
                }
            }
        }

        if !current_token.is_empty() {
            tokens.push(current_token);
        }

        tokens
    }

    /// Get next token
    pub fn next(&mut self) -> Option<String> {
        if self.position < self.tokens.len() {
            let token = self.tokens[self.position].clone();
            self.position += 1;
            Some(token)
        } else {
            None
        }
    }

    /// Peek at next token without consuming
    pub fn peek(&self) -> Option<&str> {
        if self.position < self.tokens.len() {
            Some(&self.tokens[self.position])
        } else {
            None
        }
    }

    /// Check if more tokens are available
    pub fn has_more(&self) -> bool {
        self.position < self.tokens.len()
    }

    /// Parse a point coordinate (x,y or x,y,z)
    pub fn parse_point(&mut self) -> CommandResult<Point> {
        let x_str = self.next()
            .ok_or_else(|| CommandError::InvalidInput("Expected X coordinate".to_string()))?;
        let x = x_str.parse::<f64>()
            .map_err(|_| CommandError::InvalidInput(format!("Invalid X coordinate: {}", x_str)))?;

        let y_str = self.next()
            .ok_or_else(|| CommandError::InvalidInput("Expected Y coordinate".to_string()))?;
        let y = y_str.parse::<f64>()
            .map_err(|_| CommandError::InvalidInput(format!("Invalid Y coordinate: {}", y_str)))?;

        // Z coordinate is optional
        let z = if self.has_more() {
            if let Some(z_str) = self.peek() {
                if let Ok(z_val) = z_str.parse::<f64>() {
                    self.next(); // Consume the token
                    z_val
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        };

        Ok(Point::new(x, y, z))
    }

    /// Parse a distance/length value
    pub fn parse_distance(&mut self) -> CommandResult<f64> {
        let dist_str = self.next()
            .ok_or_else(|| CommandError::InvalidInput("Expected distance value".to_string()))?;
        dist_str.parse::<f64>()
            .map_err(|_| CommandError::InvalidInput(format!("Invalid distance: {}", dist_str)))
    }

    /// Parse an angle value (in degrees)
    pub fn parse_angle(&mut self) -> CommandResult<f64> {
        let angle_str = self.next()
            .ok_or_else(|| CommandError::InvalidInput("Expected angle value".to_string()))?;
        angle_str.parse::<f64>()
            .map_err(|_| CommandError::InvalidInput(format!("Invalid angle: {}", angle_str)))
    }

    /// Parse an integer value
    pub fn parse_integer(&mut self) -> CommandResult<i32> {
        let int_str = self.next()
            .ok_or_else(|| CommandError::InvalidInput("Expected integer value".to_string()))?;
        int_str.parse::<i32>()
            .map_err(|_| CommandError::InvalidInput(format!("Invalid integer: {}", int_str)))
    }

    /// Parse a text string
    pub fn parse_text(&mut self) -> CommandResult<String> {
        self.next()
            .ok_or_else(|| CommandError::InvalidInput("Expected text value".to_string()))
    }

    /// Parse an option flag (e.g., /OPTION)
    pub fn parse_option(&mut self) -> CommandResult<(String, String)> {
        let token = self.next()
            .ok_or_else(|| CommandError::InvalidInput("Expected option".to_string()))?;

        if let Some(equals_pos) = token.find('=') {
            let key = token[..equals_pos].to_string();
            let value = token[equals_pos + 1..].to_string();
            Ok((key, value))
        } else {
            Ok((token, "true".to_string()))
        }
    }

    /// Get remaining tokens as a single string
    pub fn remaining(&mut self) -> String {
        let remaining: Vec<String> = self.tokens[self.position..].to_vec();
        self.position = self.tokens.len();
        remaining.join(" ")
    }

    /// Reset parser position
    pub fn reset(&mut self) {
        self.position = 0;
    }

    /// Get all tokens
    pub fn tokens(&self) -> &[String] {
        &self.tokens
    }
}

/// Command processor with queue management
pub struct CommandProcessor {
    /// Command registry
    registry: CommandRegistry,
    /// Undo/redo history
    history: UndoStack,
    /// Command queue for batch processing
    queue: VecDeque<Box<dyn Command>>,
    /// Currently executing command
    current_command: Option<Box<dyn Command>>,
    /// Last successfully executed command (for repeat)
    last_command: Option<Box<dyn Command>>,
    /// Command chaining enabled
    chaining_enabled: bool,
}

impl CommandProcessor {
    /// Create a new command processor
    pub fn new(registry: CommandRegistry) -> Self {
        Self {
            registry,
            history: UndoStack::new(),
            queue: VecDeque::new(),
            current_command: None,
            last_command: None,
            chaining_enabled: false,
        }
    }

    /// Create with custom undo stack
    pub fn with_history(registry: CommandRegistry, history: UndoStack) -> Self {
        Self {
            registry,
            history,
            queue: VecDeque::new(),
            current_command: None,
            last_command: None,
            chaining_enabled: false,
        }
    }

    /// Execute a command by name with arguments
    pub fn execute(&mut self, command_line: &str, context: &mut CommandContext) -> CommandResult {
        let mut parser = InputParser::new(command_line);

        // Get command name
        let cmd_name = parser.next()
            .ok_or_else(|| CommandError::InvalidInput("No command specified".to_string()))?;

        // Handle special commands
        if cmd_name.to_uppercase() == "UNDO" {
            return self.undo(context);
        }
        if cmd_name.to_uppercase() == "REDO" {
            return self.redo(context);
        }
        if cmd_name.to_uppercase() == "REPEAT" || cmd_name.is_empty() {
            return self.repeat_last(context);
        }

        // Look up command
        let mut command = self.registry.clone_command(&cmd_name)
            .ok_or_else(|| {
                // Try fuzzy matching for suggestions
                let suggestions = self.registry.fuzzy_match(&cmd_name, 2);
                if !suggestions.is_empty() {
                    CommandError::InvalidInput(format!(
                        "Unknown command: {}. Did you mean: {}?",
                        cmd_name,
                        suggestions.join(", ")
                    ))
                } else {
                    CommandError::InvalidInput(format!("Unknown command: {}", cmd_name))
                }
            })?;

        // Parse remaining arguments into context options
        while parser.has_more() {
            if let Ok((key, value)) = parser.parse_option() {
                context.options.insert(key, value);
            } else {
                break;
            }
        }

        // Create memento before execution
        let memento = command.create_memento(context);

        // Execute command
        command.execute(context)?;

        // Add to history if the command can be undone
        if command.can_undo() {
            let description = format!("{}", command.name());
            self.history.push(command.clone_box(), memento, description);
        }

        // Store as last command for repeat
        self.last_command = Some(command.clone_box());

        // Store current command
        self.current_command = Some(command);

        Ok(())
    }

    /// Process input for multi-step command
    pub fn process_input(&mut self, input: &str, context: &mut CommandContext) -> CommandResult {
        if let Some(ref mut command) = self.current_command {
            command.process_input(input, context)?;

            // Check if command is complete
            match command.state() {
                CommandState::Completed => {
                    self.current_command = None;
                }
                CommandState::Failed(msg) => {
                    self.current_command = None;
                    return Err(CommandError::Other(msg));
                }
                CommandState::Cancelled => {
                    self.current_command = None;
                    return Err(CommandError::Cancelled);
                }
                _ => {
                    // Command still active
                }
            }

            Ok(())
        } else {
            Err(CommandError::InvalidState("No active command".to_string()))
        }
    }

    /// Undo last command
    pub fn undo(&mut self, context: &mut CommandContext) -> CommandResult {
        let description = self.history.undo(context)?;
        println!("Undid: {}", description);
        Ok(())
    }

    /// Redo last undone command
    pub fn redo(&mut self, context: &mut CommandContext) -> CommandResult {
        let description = self.history.redo(context)?;
        println!("Redid: {}", description);
        Ok(())
    }

    /// Repeat last command
    pub fn repeat_last(&mut self, context: &mut CommandContext) -> CommandResult {
        if let Some(ref command) = self.last_command {
            let mut command_clone = command.clone_box();

            // Create memento before execution
            let memento = command_clone.create_memento(context);

            // Execute command
            command_clone.execute(context)?;

            // Add to history
            if command_clone.can_undo() {
                let description = format!("{}", command_clone.name());
                self.history.push(command_clone, memento, description);
            }

            Ok(())
        } else {
            Err(CommandError::InvalidState("No previous command to repeat".to_string()))
        }
    }

    /// Cancel current command
    pub fn cancel_current(&mut self) -> CommandResult {
        if self.current_command.is_some() {
            self.current_command = None;
            Ok(())
        } else {
            Err(CommandError::InvalidState("No active command to cancel".to_string()))
        }
    }

    /// Queue a command for batch execution
    pub fn queue_command(&mut self, command: Box<dyn Command>) {
        self.queue.push_back(command);
    }

    /// Execute all queued commands
    pub fn execute_queue(&mut self, context: &mut CommandContext) -> CommandResult {
        // Start a history group for batch operations
        self.history.begin_group("Batch operations");

        while let Some(mut command) = self.queue.pop_front() {
            // Create memento
            let memento = command.create_memento(context);

            // Execute
            if let Err(e) = command.execute(context) {
                // End group and return error
                self.history.end_group();
                self.queue.clear(); // Clear remaining queue on error
                return Err(e);
            }

            // Add to history
            if command.can_undo() {
                let description = format!("{}", command.name());
                self.history.push(command, memento, description);
            }
        }

        // End history group
        self.history.end_group();

        Ok(())
    }

    /// Clear command queue
    pub fn clear_queue(&mut self) {
        self.queue.clear();
    }

    /// Get queue size
    pub fn queue_size(&self) -> usize {
        self.queue.len()
    }

    /// Enable/disable command chaining
    pub fn set_chaining(&mut self, enabled: bool) {
        self.chaining_enabled = enabled;
    }

    /// Check if command chaining is enabled
    pub fn is_chaining_enabled(&self) -> bool {
        self.chaining_enabled
    }

    /// Get current command state
    pub fn current_state(&self) -> Option<CommandState> {
        self.current_command.as_ref().map(|cmd| cmd.state())
    }

    /// Get current command name
    pub fn current_command_name(&self) -> Option<&str> {
        self.current_command.as_ref().map(|cmd| cmd.name())
    }

    /// Get command registry
    pub fn registry(&self) -> &CommandRegistry {
        &self.registry
    }

    /// Get mutable command registry
    pub fn registry_mut(&mut self) -> &mut CommandRegistry {
        &mut self.registry
    }

    /// Get history
    pub fn history(&self) -> &UndoStack {
        &self.history
    }

    /// Get mutable history
    pub fn history_mut(&mut self) -> &mut UndoStack {
        &mut self.history
    }

    /// Begin a command group (for compound operations)
    pub fn begin_group(&mut self, description: impl Into<String>) {
        self.history.begin_group(description);
    }

    /// End current command group
    pub fn end_group(&mut self) {
        self.history.end_group();
    }

    /// Get autocomplete suggestions
    pub fn autocomplete(&self, partial: &str) -> Vec<String> {
        self.registry.autocomplete(partial)
    }

    /// Get command help
    pub fn get_help(&self, command: &str) -> Option<String> {
        self.registry.get_help(command)
    }

    /// Get all available commands
    pub fn list_commands(&self) -> Vec<String> {
        self.registry.command_names()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_parser_tokenize() {
        let parser = InputParser::new("LINE 0 0 10 10");
        assert_eq!(parser.tokens().len(), 5);
        assert_eq!(parser.tokens()[0], "LINE");
    }

    #[test]
    fn test_input_parser_point() {
        let mut parser = InputParser::new("10.5 20.5 30.5");
        let point = parser.parse_point().unwrap();
        assert_eq!(point.x, 10.5);
        assert_eq!(point.y, 20.5);
        assert_eq!(point.z, 30.5);
    }

    #[test]
    fn test_input_parser_quoted_text() {
        let parser = InputParser::new(r#"TEXT "Hello World" 10 20"#);
        assert_eq!(parser.tokens().len(), 4);
        assert_eq!(parser.tokens()[1], "Hello World");
    }

    #[test]
    fn test_command_processor_creation() {
        let registry = CommandRegistry::new();
        let processor = CommandProcessor::new(registry);
        assert_eq!(processor.queue_size(), 0);
    }
}
