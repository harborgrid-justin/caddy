// Integration tests for the layer management system

#[cfg(test)]
mod layer_integration_tests {
    use caddy::core::Color;
    use caddy::layers::{
        EntityProperties, FilterCriterion, Layer, LayerFilter, LayerGroupManager, LayerManager,
        LayerStateManager, LineType, LineWeight,
    };

    #[test]
    fn test_complete_layer_workflow() {
        // Create layer manager
        let mut manager = LayerManager::new();

        // Verify default layer exists
        assert_eq!(manager.layer_count(), 1);
        assert_eq!(manager.current_layer_name(), "0");
        assert!(manager.has_layer("0"));

        // Create new layers
        manager.create_layer("Walls".to_string()).unwrap();
        manager.create_layer("Doors".to_string()).unwrap();
        manager.create_layer("Windows".to_string()).unwrap();
        manager.create_layer("Dimensions".to_string()).unwrap();

        assert_eq!(manager.layer_count(), 5);

        // Configure wall layer
        if let Some(layer) = manager.get_layer_mut("Walls") {
            layer.set_color(Color::RED);
            layer.set_line_type(LineType::Continuous);
            layer.set_line_weight(LineWeight::W0_50);
        }

        // Configure dimension layer
        if let Some(layer) = manager.get_layer_mut("Dimensions") {
            layer.set_color(Color::CYAN);
            layer.set_line_type(LineType::Continuous);
            layer.set_line_weight(LineWeight::W0_25);
        }

        // Set current layer
        manager.set_current_layer("Walls").unwrap();
        assert_eq!(manager.current_layer_name(), "Walls");

        // Freeze a layer
        manager.freeze_layer("Windows").unwrap();
        assert!(manager.get_layer("Windows").unwrap().frozen);

        // Lock a layer
        manager.lock_layer("Doors").unwrap();
        assert!(manager.get_layer("Doors").unwrap().locked);

        // Try to set frozen layer as current (should fail)
        assert!(manager.set_current_layer("Windows").is_err());

        // Rename a layer
        manager
            .rename_layer("Dimensions", "DIM".to_string())
            .unwrap();
        assert!(manager.has_layer("DIM"));
        assert!(!manager.has_layer("Dimensions"));

        // Delete a layer
        manager.delete_layer("Windows").unwrap();
        assert_eq!(manager.layer_count(), 4);

        // Try to delete default layer (should fail)
        assert!(manager.delete_layer("0").is_err());
    }

    #[test]
    fn test_entity_properties_and_inheritance() {
        let mut manager = LayerManager::new();
        manager.create_layer("TestLayer".to_string()).unwrap();

        // Configure layer
        if let Some(layer) = manager.get_layer_mut("TestLayer") {
            layer.set_color(Color::BLUE);
            layer.set_line_type(LineType::Dashed);
            layer.set_line_weight(LineWeight::W0_35);
            layer.transparency = 50;
        }

        // Create entity with ByLayer properties
        let props = EntityProperties::new("TestLayer".to_string());
        assert!(props.is_all_by_layer());

        // Resolve properties
        let layer = manager.get_layer("TestLayer").unwrap();
        let resolved = props.resolve(layer);

        assert_eq!(resolved.color, Color::BLUE);
        assert_eq!(resolved.line_type, LineType::Dashed);
        assert_eq!(resolved.line_weight, LineWeight::W0_35);
        assert_eq!(resolved.transparency, 50);

        // Override color
        let mut props2 = EntityProperties::new("TestLayer".to_string());
        props2.set_color(Color::RED);
        assert!(!props2.is_all_by_layer());

        let resolved2 = props2.resolve(layer);
        assert_eq!(resolved2.color, Color::RED);
        assert_eq!(resolved2.line_type, LineType::Dashed); // Still from layer
    }

    #[test]
    fn test_layer_states() {
        let mut manager = LayerManager::new();
        let mut state_mgr = LayerStateManager::new();

        // Create layers
        manager.create_layer("Layer1".to_string()).unwrap();
        manager.create_layer("Layer2".to_string()).unwrap();

        // Configure layers
        manager.freeze_layer("Layer1").unwrap();
        manager.lock_layer("Layer2").unwrap();

        // Save state
        state_mgr
            .save_state(
                "ConfigA".to_string(),
                "Configuration A".to_string(),
                &manager,
            )
            .unwrap();

        // Modify layers
        manager.thaw_layer("Layer1").unwrap();
        manager.unlock_layer("Layer2").unwrap();
        assert!(!manager.get_layer("Layer1").unwrap().frozen);
        assert!(!manager.get_layer("Layer2").unwrap().locked);

        // Save another state
        state_mgr
            .save_state(
                "ConfigB".to_string(),
                "Configuration B".to_string(),
                &manager,
            )
            .unwrap();

        // Restore first state
        state_mgr.restore_state("ConfigA", &mut manager).unwrap();
        assert!(manager.get_layer("Layer1").unwrap().frozen);
        assert!(manager.get_layer("Layer2").unwrap().locked);

        // Restore second state
        state_mgr.restore_state("ConfigB", &mut manager).unwrap();
        assert!(!manager.get_layer("Layer1").unwrap().frozen);
        assert!(!manager.get_layer("Layer2").unwrap().locked);

        // Export and import state
        let json = state_mgr.export_state("ConfigA").unwrap();
        let mut new_state_mgr = LayerStateManager::new();
        let name = new_state_mgr.import_state(&json).unwrap();
        assert_eq!(name, "ConfigA");
    }

    #[test]
    fn test_layer_filtering() {
        let manager = LayerManager::new();

        // Create test layers
        let mut wall = Layer::new("WALL-01".to_string());
        wall.set_color(Color::RED);
        wall.visible = true;

        let mut door = Layer::new("DOOR-01".to_string());
        door.set_color(Color::BLUE);
        door.visible = true;

        let mut dim = Layer::new("DIM-Linear".to_string());
        dim.set_color(Color::CYAN);
        dim.visible = false;

        // Filter by name pattern
        let mut filter1 = LayerFilter::new("WallFilter".to_string(), "All walls".to_string());
        filter1.add_criterion(FilterCriterion::NamePattern("WALL*".to_string()));
        assert!(filter1.matches(&wall));
        assert!(!filter1.matches(&door));

        // Filter by visibility
        let mut filter2 = LayerFilter::new("Visible".to_string(), "Visible only".to_string());
        filter2.add_criterion(FilterCriterion::Visible(true));
        assert!(filter2.matches(&wall));
        assert!(filter2.matches(&door));
        assert!(!filter2.matches(&dim));

        // Combined filter
        let mut filter3 = LayerFilter::new("Combined".to_string(), "Combined filter".to_string());
        filter3.add_criterion(FilterCriterion::NameStartsWith("DIM".to_string()));
        filter3.add_criterion(FilterCriterion::ColorEquals(Color::CYAN));
        assert!(filter3.matches(&dim));
        assert!(!filter3.matches(&wall));

        // Filter collection
        let layers = vec![&wall, &door, &dim];
        let visible_layers = filter2.filter_layers(&layers);
        assert_eq!(visible_layers.len(), 2);
    }

    #[test]
    fn test_layer_groups() {
        let mut group_mgr = LayerGroupManager::new();

        // Create groups
        group_mgr
            .create_group("Architectural".to_string(), "Arch layers".to_string())
            .unwrap();
        group_mgr
            .create_group("Mechanical".to_string(), "Mech layers".to_string())
            .unwrap();

        // Add layers to groups
        group_mgr
            .add_layer_to_group("Architectural", "Walls".to_string())
            .unwrap();
        group_mgr
            .add_layer_to_group("Architectural", "Doors".to_string())
            .unwrap();
        group_mgr
            .add_layer_to_group("Architectural", "Windows".to_string())
            .unwrap();

        group_mgr
            .add_layer_to_group("Mechanical", "HVAC".to_string())
            .unwrap();
        group_mgr
            .add_layer_to_group("Mechanical", "Plumbing".to_string())
            .unwrap();

        // Verify group membership
        let arch_group = group_mgr.get_group("Architectural").unwrap();
        assert_eq!(arch_group.layer_count(), 3);
        assert!(arch_group.contains_layer("Walls"));
        assert!(arch_group.contains_layer("Doors"));

        let mech_group = group_mgr.get_group("Mechanical").unwrap();
        assert_eq!(mech_group.layer_count(), 2);

        // Find groups containing a layer
        let groups = group_mgr.groups_containing_layer("Walls");
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0], "Architectural");

        // Remove layer from group
        group_mgr
            .remove_layer_from_group("Architectural", "Windows")
            .unwrap();
        assert_eq!(arch_group.layer_count(), 2);
    }

    #[test]
    fn test_line_types_and_weights() {
        // Test line types
        let continuous = LineType::Continuous;
        assert_eq!(continuous.name(), "CONTINUOUS");
        assert!(continuous.pattern().is_empty());

        let dashed = LineType::Dashed;
        assert!(!dashed.pattern().is_empty());

        let from_name = LineType::from_name("HIDDEN");
        assert_eq!(from_name, LineType::Hidden);

        // Test line weights
        let weight = LineWeight::W0_25;
        assert_eq!(weight.to_mm(), Some(0.25));

        let from_mm = LineWeight::from_mm(0.26);
        assert_eq!(from_mm, LineWeight::W0_25);

        let standard = LineWeight::standard_weights();
        assert!(!standard.is_empty());
        assert!(standard.contains(&LineWeight::W0_25));
    }

    #[test]
    fn test_layer_operations() {
        let mut manager = LayerManager::new();

        // Isolate layer
        manager.create_layer("Keep".to_string()).unwrap();
        manager.create_layer("Freeze1".to_string()).unwrap();
        manager.create_layer("Freeze2".to_string()).unwrap();

        manager.isolate_layer("Keep").unwrap();
        assert!(!manager.get_layer("Keep").unwrap().frozen);
        assert!(manager.get_layer("Freeze1").unwrap().frozen);
        assert!(manager.get_layer("Freeze2").unwrap().frozen);

        // Thaw all
        manager.thaw_all_layers();
        assert!(!manager.get_layer("Freeze1").unwrap().frozen);
        assert!(!manager.get_layer("Freeze2").unwrap().frozen);

        // Lock all except one
        manager.lock_all_except("Keep").unwrap();
        assert!(!manager.get_layer("Keep").unwrap().locked);
        assert!(manager.get_layer("Freeze1").unwrap().locked);

        // Unlock all
        manager.unlock_all_layers();
        assert!(!manager.get_layer("Freeze1").unwrap().locked);

        // Purge unused layers
        let used = vec!["0".to_string(), "Keep".to_string()];
        let deleted = manager.purge_unused_layers(&used);
        assert_eq!(deleted, 2);
        assert!(manager.has_layer("Keep"));
        assert!(!manager.has_layer("Freeze1"));
    }

    #[test]
    fn test_layer_validation() {
        // Valid names
        assert!(Layer::validate_name("Layer1").is_ok());
        assert!(Layer::validate_name("WALL-01").is_ok());
        assert!(Layer::validate_name("A_B_C").is_ok());

        // Invalid names
        assert!(Layer::validate_name("").is_err());
        assert!(Layer::validate_name("Invalid<Name").is_err());
        assert!(Layer::validate_name("Invalid*Name").is_err());
        assert!(Layer::validate_name("Invalid/Name").is_err());
    }
}
