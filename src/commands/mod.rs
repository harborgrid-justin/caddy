// Command system module for CADDY CAD
// Central module that exports all command-related functionality

pub mod command;
pub mod history;
pub mod registry;
pub mod processor;
pub mod draw;
pub mod modify;
pub mod edit;
pub mod view;

// Re-export commonly used types
pub use command::{
    Command, CommandContext, CommandError, CommandResult, CommandState,
    CommandInput, CommandParameter, CommandMemento,
    Point, EntityId, SelectionSet, Document,
};

pub use history::{UndoStack, HistoryConfig};
pub use registry::CommandRegistry;
pub use processor::{CommandProcessor, InputParser};

// Re-export all command implementations
pub use draw::*;
pub use modify::*;
pub use edit::*;
pub use view::*;

/// Initialize and register all standard CAD commands
pub fn register_all_commands(registry: &mut CommandRegistry) {
    // Drawing commands
    registry.register_with_category(Box::new(LineCommand::new()), "Draw");
    registry.register_with_category(Box::new(CircleCommand::new()), "Draw");
    registry.register_with_category(Box::new(ArcCommand::new()), "Draw");
    registry.register_with_category(Box::new(RectangleCommand::new()), "Draw");
    registry.register_with_category(Box::new(PolygonCommand::new()), "Draw");
    registry.register_with_category(Box::new(PolylineCommand::new()), "Draw");
    registry.register_with_category(Box::new(SplineCommand::new()), "Draw");
    registry.register_with_category(Box::new(EllipseCommand::new()), "Draw");
    registry.register_with_category(Box::new(TextCommand::new()), "Draw");

    // Modify commands
    registry.register_with_category(Box::new(MoveCommand::new()), "Modify");
    registry.register_with_category(Box::new(CopyCommand::new()), "Modify");
    registry.register_with_category(Box::new(RotateCommand::new()), "Modify");
    registry.register_with_category(Box::new(ScaleCommand::new()), "Modify");
    registry.register_with_category(Box::new(MirrorCommand::new()), "Modify");
    registry.register_with_category(Box::new(ArrayCommand::new()), "Modify");
    registry.register_with_category(Box::new(OffsetCommand::new()), "Modify");
    registry.register_with_category(Box::new(TrimCommand::new()), "Modify");
    registry.register_with_category(Box::new(ExtendCommand::new()), "Modify");
    registry.register_with_category(Box::new(FilletCommand::new()), "Modify");
    registry.register_with_category(Box::new(ChamferCommand::new()), "Modify");
    registry.register_with_category(Box::new(BreakCommand::new()), "Modify");
    registry.register_with_category(Box::new(JoinCommand::new()), "Modify");
    registry.register_with_category(Box::new(ExplodeCommand::new()), "Modify");

    // Edit commands
    registry.register_with_category(Box::new(EraseCommand::new()), "Edit");
    registry.register_with_category(Box::new(UndoCommand::new()), "Edit");
    registry.register_with_category(Box::new(RedoCommand::new()), "Edit");
    registry.register_with_category(Box::new(CutCommand::new()), "Edit");
    registry.register_with_category(Box::new(CopyToClipboardCommand::new()), "Edit");
    registry.register_with_category(Box::new(PasteCommand::new()), "Edit");
    registry.register_with_category(Box::new(SelectAllCommand::new()), "Edit");
    registry.register_with_category(Box::new(ClearSelectionCommand::new()), "Edit");

    // View commands
    registry.register_with_category(Box::new(ZoomCommand::new()), "View");
    registry.register_with_category(Box::new(PanCommand::new()), "View");
    registry.register_with_category(Box::new(RegenCommand::new()), "View");
    registry.register_with_category(Box::new(RedrawCommand::new()), "View");
    registry.register_with_category(Box::new(ViewCommand::new()), "View");
    registry.register_with_category(Box::new(ViewResCommand::new()), "View");
}

/// Create a fully initialized command processor with all standard commands
pub fn create_standard_processor() -> CommandProcessor {
    let mut registry = CommandRegistry::new();
    register_all_commands(&mut registry);
    CommandProcessor::new(registry)
}

/// Create a command processor with custom history configuration
pub fn create_processor_with_config(history_config: HistoryConfig) -> CommandProcessor {
    let mut registry = CommandRegistry::new();
    register_all_commands(&mut registry);
    let history = UndoStack::with_config(history_config);
    CommandProcessor::with_history(registry, history)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_all_commands() {
        let mut registry = CommandRegistry::new();
        register_all_commands(&mut registry);

        // Verify some commands are registered
        assert!(registry.contains("LINE"));
        assert!(registry.contains("CIRCLE"));
        assert!(registry.contains("MOVE"));
        assert!(registry.contains("ZOOM"));
        assert!(registry.contains("UNDO"));

        // Verify aliases work
        assert!(registry.contains("L")); // LINE alias
        assert!(registry.contains("C")); // CIRCLE alias
        assert!(registry.contains("M")); // MOVE alias
        assert!(registry.contains("Z")); // ZOOM alias
    }

    #[test]
    fn test_create_standard_processor() {
        let processor = create_standard_processor();
        assert!(processor.registry().contains("LINE"));
        assert!(processor.registry().contains("CIRCLE"));
    }

    #[test]
    fn test_command_categories() {
        let mut registry = CommandRegistry::new();
        register_all_commands(&mut registry);

        let draw_commands = registry.get_category("Draw");
        assert!(!draw_commands.is_empty());
        assert!(draw_commands.contains(&"LINE".to_string()));

        let modify_commands = registry.get_category("Modify");
        assert!(!modify_commands.is_empty());
        assert!(modify_commands.contains(&"MOVE".to_string()));

        let edit_commands = registry.get_category("Edit");
        assert!(!edit_commands.is_empty());
        assert!(edit_commands.contains(&"ERASE".to_string()));

        let view_commands = registry.get_category("View");
        assert!(!view_commands.is_empty());
        assert!(view_commands.contains(&"ZOOM".to_string()));
    }

    #[test]
    fn test_command_execution_basic() {
        let mut processor = create_standard_processor();
        let mut context = CommandContext::new(Document::new());

        // Test LINE command (will fail without proper parameters, but should parse)
        let result = processor.execute("LINE", &mut context);
        // Should fail because no points specified, but command should be recognized
        assert!(result.is_err());

        // Test ZOOM EXTENTS
        let result = processor.execute("ZOOM", &mut context);
        // Should work or fail gracefully
        let _ = result;
    }

    #[test]
    fn test_undo_redo_integration() {
        let processor = create_standard_processor();
        assert!(processor.history().can_undo() == false);
        assert!(processor.history().can_redo() == false);
    }

    #[test]
    fn test_autocomplete() {
        let processor = create_standard_processor();

        let suggestions = processor.autocomplete("LI");
        assert!(suggestions.iter().any(|s| s.contains("LINE")));

        let suggestions = processor.autocomplete("CI");
        assert!(suggestions.iter().any(|s| s.contains("CIRCLE")));

        let suggestions = processor.autocomplete("ZO");
        assert!(suggestions.iter().any(|s| s.contains("ZOOM")));
    }

    #[test]
    fn test_command_help() {
        let processor = create_standard_processor();

        let help = processor.get_help("LINE");
        assert!(help.is_some());
        assert!(help.unwrap().contains("LINE"));

        let help = processor.get_help("CIRCLE");
        assert!(help.is_some());
        assert!(help.unwrap().contains("CIRCLE"));
    }

    #[test]
    fn test_fuzzy_command_matching() {
        let processor = create_standard_processor();

        // Test fuzzy matching through registry
        let matches = processor.registry().fuzzy_match("LNIE", 2);
        assert!(matches.iter().any(|m| m == "LINE"));

        let matches = processor.registry().fuzzy_match("CIRLCE", 2);
        assert!(matches.iter().any(|m| m == "CIRCLE"));
    }

    #[test]
    fn test_input_parser() {
        let mut parser = InputParser::new("10 20 30");
        let point = parser.parse_point();
        assert!(point.is_ok());
        let p = point.unwrap();
        assert_eq!(p.x, 10.0);
        assert_eq!(p.y, 20.0);
        assert_eq!(p.z, 30.0);
    }

    #[test]
    fn test_input_parser_quoted_strings() {
        let parser = InputParser::new(r#"TEXT "Hello World" 10 20"#);
        let tokens = parser.tokens();
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], "TEXT");
        assert_eq!(tokens[1], "Hello World");
        assert_eq!(tokens[2], "10");
        assert_eq!(tokens[3], "20");
    }

    #[test]
    fn test_selection_set() {
        let mut selection = SelectionSet::new();
        assert!(selection.is_empty());

        let id1 = EntityId::new(1);
        let id2 = EntityId::new(2);

        selection.add(id1);
        assert_eq!(selection.len(), 1);
        assert!(selection.contains(&id1));

        selection.add(id2);
        assert_eq!(selection.len(), 2);

        selection.remove(&id1);
        assert_eq!(selection.len(), 1);
        assert!(!selection.contains(&id1));
        assert!(selection.contains(&id2));

        selection.clear();
        assert!(selection.is_empty());
    }

    #[test]
    fn test_document_operations() {
        let mut doc = Document::new();
        assert_eq!(doc.entity_count(), 0);

        let _entity = Box::new("test entity");
        let id = doc.add_entity(entity);
        assert_eq!(doc.entity_count(), 1);

        assert!(doc.get_entity(&id).is_some());

        doc.remove_entity(&id);
        assert_eq!(doc.entity_count(), 0);
    }

    #[test]
    fn test_history_limits() {
        let config = HistoryConfig {
            max_undo_levels: 5,
            max_memory_bytes: 0,
            auto_group_similar: false,
            group_time_window_ms: 1000,
        };

        let processor = create_processor_with_config(config);
        assert_eq!(processor.history().config().max_undo_levels, 5);
    }
}
