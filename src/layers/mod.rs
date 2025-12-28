//! Layer management system for CADDY
//!
//! This module provides a comprehensive layer management system matching AutoCAD semantics:
//!
//! # Key Features
//!
//! - **Layer Management**: Create, rename, delete, and organize layers
//! - **Visual Properties**: Control color, line type, line weight per layer
//! - **Layer States**: Visibility, frozen, locked, printable flags
//! - **Property Inheritance**: ByLayer and ByBlock property resolution
//! - **Layer States**: Save and restore layer configurations
//! - **Layer Filtering**: Filter layers by name patterns and properties
//! - **Layer Groups**: Organize layers into named groups
//!
//! # Module Structure
//!
//! - `layer`: Core layer type with properties and operations
//! - `manager`: Central layer manager with event system
//! - `styles`: Line types, line weights, and pattern definitions
//! - `properties`: Entity properties with ByLayer/ByBlock inheritance
//! - `state`: Layer state snapshots and management
//! - `filter`: Layer filtering and grouping
//!
//! # Examples
//!
//! ## Creating and Using Layers
//!
//! ```rust
//! use caddy::layers::{LayerManager, Layer};
//! use caddy::core::Color;
//!
//! let mut manager = LayerManager::new();
//!
//! // Create a new layer
//! let layer = manager.create_layer("Walls".to_string()).unwrap();
//!
//! // Set layer properties
//! if let Some(layer) = manager.get_layer_mut("Walls") {
//!     layer.set_color(Color::RED);
//!     layer.frozen = false;
//! }
//!
//! // Set as current layer
//! manager.set_current_layer("Walls").unwrap();
//! ```
//!
//! ## Entity Properties with ByLayer
//!
//! ```rust
//! use caddy::layers::{EntityProperties, LayerManager};
//! use caddy::core::Color;
//!
//! let mut manager = LayerManager::new();
//! manager.create_layer("Dimensions".to_string()).unwrap();
//!
//! // Create entity properties on the layer
//! let mut props = EntityProperties::new("Dimensions".to_string());
//!
//! // By default, all properties are ByLayer
//! assert!(props.is_all_by_layer());
//!
//! // Override color for this entity
//! props.set_color(Color::BLUE);
//!
//! // Resolve properties (get actual values)
//! let layer = manager.get_layer("Dimensions").unwrap();
//! let resolved = props.resolve(layer);
//! ```
//!
//! ## Layer States
//!
//! ```rust
//! use caddy::layers::{LayerManager, LayerStateManager};
//!
//! let mut manager = LayerManager::new();
//! let mut state_mgr = LayerStateManager::new();
//!
//! // Save current layer configuration
//! state_mgr.save_state(
//!     "AllVisible".to_string(),
//!     "All layers visible".to_string(),
//!     &manager
//! ).unwrap();
//!
//! // Make changes...
//! // ...
//!
//! // Restore saved state
//! state_mgr.restore_state("AllVisible", &mut manager).unwrap();
//! ```
//!
//! ## Layer Filtering
//!
//! ```rust
//! use caddy::layers::{LayerFilter, FilterCriterion};
//!
//! let mut filter = LayerFilter::new(
//!     "DimensionLayers".to_string(),
//!     "All dimension layers".to_string()
//! );
//!
//! // Add criteria (all must match)
//! filter.add_criterion(FilterCriterion::NamePattern("DIM_*".to_string()));
//! filter.add_criterion(FilterCriterion::Visible(true));
//!
//! // Filter layers
//! let layers = vec![/* ... */];
//! let filtered = filter.filter_layers(&layers);
//! ```

pub mod filter;
pub mod layer;
pub mod manager;
pub mod properties;
pub mod state;
pub mod styles;

// Re-export commonly used types
pub use filter::{FilterCriterion, LayerFilter, LayerFilterError, LayerGroup, LayerGroupManager};
pub use layer::{Layer, LayerError};
pub use manager::{LayerEventListener, LayerManager};
pub use properties::{
    ColorSource, EntityProperties, LineTypeSource, LineWeightSource, ResolvedProperties,
};
pub use state::{LayerSnapshot, LayerState, LayerStateError, LayerStateManager};
pub use styles::{LinePattern, LineType, LineTypeScale, LineWeight};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Color;

    #[test]
    fn test_full_workflow() {
        // Create layer manager
        let mut manager = LayerManager::new();

        // Create layers
        manager.create_layer("Walls".to_string()).unwrap();
        manager.create_layer("Doors".to_string()).unwrap();
        manager.create_layer("Windows".to_string()).unwrap();

        assert_eq!(manager.layer_count(), 4); // Including default "0"

        // Configure wall layer
        if let Some(layer) = manager.get_layer_mut("Walls") {
            layer.set_color(Color::RED);
            layer.set_line_weight(LineWeight::W0_50);
        }

        // Set current layer
        manager.set_current_layer("Walls").unwrap();
        assert_eq!(manager.current_layer_name(), "Walls");

        // Create entity properties
        let mut props = EntityProperties::new("Walls".to_string());
        assert!(props.is_all_by_layer());

        // Resolve properties
        let layer = manager.get_layer("Walls").unwrap();
        let resolved = props.resolve(layer);
        assert_eq!(resolved.color, Color::RED);
        assert_eq!(resolved.line_weight, LineWeight::W0_50);
    }

    #[test]
    fn test_layer_states_workflow() {
        let mut manager = LayerManager::new();
        let mut state_mgr = LayerStateManager::new();

        // Create and configure layers
        manager.create_layer("Layer1".to_string()).unwrap();
        manager.freeze_layer("Layer1").unwrap();

        // Save state
        state_mgr
            .save_state("Frozen".to_string(), "Test".to_string(), &manager)
            .unwrap();

        // Modify layers
        manager.thaw_layer("Layer1").unwrap();
        assert!(!manager.get_layer("Layer1").unwrap().frozen);

        // Restore state
        state_mgr.restore_state("Frozen", &mut manager).unwrap();
        assert!(manager.get_layer("Layer1").unwrap().frozen);
    }

    #[test]
    fn test_layer_filtering_workflow() {
        let manager = LayerManager::new();

        let mut filter = LayerFilter::new("Test".to_string(), "Test filter".to_string());
        filter.add_criterion(FilterCriterion::Visible(true));

        let mut layer1 = Layer::new("L1".to_string());
        layer1.visible = true;
        let mut layer2 = Layer::new("L2".to_string());
        layer2.visible = false;

        assert!(filter.matches(&layer1));
        assert!(!filter.matches(&layer2));
    }

    #[test]
    fn test_property_inheritance() {
        let mut manager = LayerManager::new();
        manager.create_layer("Test".to_string()).unwrap();

        if let Some(layer) = manager.get_layer_mut("Test") {
            layer.set_color(Color::BLUE);
            layer.set_line_weight(LineWeight::W0_25);
        }

        // Entity with ByLayer properties
        let props = EntityProperties::new("Test".to_string());
        let layer = manager.get_layer("Test").unwrap();
        let resolved = props.resolve(layer);

        assert_eq!(resolved.color, Color::BLUE);
        assert_eq!(resolved.line_weight, LineWeight::W0_25);
    }

    #[test]
    fn test_layer_groups() {
        let mut group_mgr = LayerGroupManager::new();

        group_mgr
            .create_group("Architectural".to_string(), "Arch layers".to_string())
            .unwrap();

        group_mgr
            .add_layer_to_group("Architectural", "Walls".to_string())
            .unwrap();
        group_mgr
            .add_layer_to_group("Architectural", "Doors".to_string())
            .unwrap();

        let group = group_mgr.get_group("Architectural").unwrap();
        assert_eq!(group.layer_count(), 2);
        assert!(group.contains_layer("Walls"));
    }
}
