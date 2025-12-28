// Layer state module for CADDY
// Provides layer state snapshots and management

use super::layer::Layer;
use super::manager::LayerManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Snapshot of layer settings at a point in time
/// Layer states allow saving and restoring layer configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerState {
    /// Name of this layer state
    pub name: String,

    /// Description of the layer state
    pub description: String,

    /// Map of layer name to layer snapshot
    layers: HashMap<String, LayerSnapshot>,

    /// Current layer at time of snapshot
    current_layer: String,
}

impl LayerState {
    /// Create a new layer state with the given name
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            layers: HashMap::new(),
            current_layer: "0".to_string(),
        }
    }

    /// Create a layer state from the current layer manager state
    pub fn from_manager(name: String, description: String, manager: &LayerManager) -> Self {
        let mut layers = HashMap::new();

        for layer in manager.iter() {
            layers.insert(layer.name.clone(), LayerSnapshot::from_layer(layer));
        }

        Self {
            name,
            description,
            layers,
            current_layer: manager.current_layer_name().to_string(),
        }
    }

    /// Get the number of layers in this state
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Get a layer snapshot by name
    pub fn get_layer(&self, name: &str) -> Option<&LayerSnapshot> {
        self.layers.get(name)
    }

    /// Apply this state to a layer manager
    pub fn apply_to_manager(&self, manager: &mut LayerManager) -> Result<(), String> {
        // Restore each layer's state
        for (name, snapshot) in &self.layers {
            if let Some(layer) = manager.get_layer_mut(name) {
                snapshot.apply_to_layer(layer);
            }
            // Note: We don't create missing layers, only update existing ones
        }

        // Restore current layer if it exists
        if manager.has_layer(&self.current_layer) {
            manager
                .set_current_layer(&self.current_layer)
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    /// Get all layer names in this state
    pub fn layer_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.layers.keys().cloned().collect();
        names.sort();
        names
    }
}

/// Snapshot of a single layer's properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerSnapshot {
    /// Layer visibility
    pub visible: bool,

    /// Layer frozen state
    pub frozen: bool,

    /// Layer locked state
    pub locked: bool,

    /// Layer printable state
    pub printable: bool,

    /// Layer transparency
    pub transparency: u8,
}

impl LayerSnapshot {
    /// Create a snapshot from a layer
    pub fn from_layer(layer: &Layer) -> Self {
        Self {
            visible: layer.visible,
            frozen: layer.frozen,
            locked: layer.locked,
            printable: layer.printable,
            transparency: layer.transparency,
        }
    }

    /// Apply this snapshot to a layer (updates only state, not color/linetype)
    pub fn apply_to_layer(&self, layer: &mut Layer) {
        layer.visible = self.visible;
        layer.frozen = self.frozen;
        layer.locked = self.locked;
        layer.printable = self.printable;
        layer.transparency = self.transparency;
    }
}

/// Manager for layer states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerStateManager {
    /// Map of state name to layer state
    states: HashMap<String, LayerState>,
}

impl LayerStateManager {
    /// Create a new layer state manager
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    /// Save the current layer manager state
    pub fn save_state(
        &mut self,
        name: String,
        description: String,
        manager: &LayerManager,
    ) -> Result<(), LayerStateError> {
        if name.is_empty() {
            return Err(LayerStateError::InvalidStateName(
                "State name cannot be empty".to_string(),
            ));
        }

        if self.states.contains_key(&name) {
            return Err(LayerStateError::StateAlreadyExists(name));
        }

        let state = LayerState::from_manager(name.clone(), description, manager);
        self.states.insert(name, state);
        Ok(())
    }

    /// Update an existing layer state
    pub fn update_state(
        &mut self,
        name: &str,
        manager: &LayerManager,
    ) -> Result<(), LayerStateError> {
        if let Some(state) = self.states.get_mut(name) {
            let description = state.description.clone();
            *state = LayerState::from_manager(name.to_string(), description, manager);
            Ok(())
        } else {
            Err(LayerStateError::StateNotFound(name.to_string()))
        }
    }

    /// Restore a saved layer state
    pub fn restore_state(
        &self,
        name: &str,
        manager: &mut LayerManager,
    ) -> Result<(), LayerStateError> {
        if let Some(state) = self.states.get(name) {
            state
                .apply_to_manager(manager)
                .map_err(|e| LayerStateError::RestoreError(e))?;
            Ok(())
        } else {
            Err(LayerStateError::StateNotFound(name.to_string()))
        }
    }

    /// Delete a layer state
    pub fn delete_state(&mut self, name: &str) -> Result<(), LayerStateError> {
        if self.states.remove(name).is_some() {
            Ok(())
        } else {
            Err(LayerStateError::StateNotFound(name.to_string()))
        }
    }

    /// Rename a layer state
    pub fn rename_state(&mut self, old_name: &str, new_name: String) -> Result<(), LayerStateError> {
        if new_name.is_empty() {
            return Err(LayerStateError::InvalidStateName(
                "State name cannot be empty".to_string(),
            ));
        }

        if !self.states.contains_key(old_name) {
            return Err(LayerStateError::StateNotFound(old_name.to_string()));
        }

        if self.states.contains_key(&new_name) {
            return Err(LayerStateError::StateAlreadyExists(new_name));
        }

        if let Some(mut state) = self.states.remove(old_name) {
            state.name = new_name.clone();
            self.states.insert(new_name, state);
            Ok(())
        } else {
            Err(LayerStateError::StateNotFound(old_name.to_string()))
        }
    }

    /// Get a layer state by name
    pub fn get_state(&self, name: &str) -> Option<&LayerState> {
        self.states.get(name)
    }

    /// Check if a state exists
    pub fn has_state(&self, name: &str) -> bool {
        self.states.contains_key(name)
    }

    /// Get all state names
    pub fn state_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.states.keys().cloned().collect();
        names.sort();
        names
    }

    /// Get number of states
    pub fn state_count(&self) -> usize {
        self.states.len()
    }

    /// Export a layer state to JSON
    pub fn export_state(&self, name: &str) -> Result<String, LayerStateError> {
        if let Some(state) = self.states.get(name) {
            serde_json::to_string_pretty(state)
                .map_err(|e| LayerStateError::SerializationError(e.to_string()))
        } else {
            Err(LayerStateError::StateNotFound(name.to_string()))
        }
    }

    /// Import a layer state from JSON
    pub fn import_state(&mut self, json: &str) -> Result<String, LayerStateError> {
        let state: LayerState = serde_json::from_str(json)
            .map_err(|e| LayerStateError::SerializationError(e.to_string()))?;

        let name = state.name.clone();

        if self.states.contains_key(&name) {
            return Err(LayerStateError::StateAlreadyExists(name));
        }

        self.states.insert(name.clone(), state);
        Ok(name)
    }

    /// Export all states to JSON
    pub fn export_all(&self) -> Result<String, LayerStateError> {
        serde_json::to_string_pretty(&self.states)
            .map_err(|e| LayerStateError::SerializationError(e.to_string()))
    }

    /// Import states from JSON (merges with existing states)
    pub fn import_all(&mut self, json: &str, overwrite: bool) -> Result<usize, LayerStateError> {
        let imported: HashMap<String, LayerState> = serde_json::from_str(json)
            .map_err(|e| LayerStateError::SerializationError(e.to_string()))?;

        let mut count = 0;
        for (name, state) in imported {
            if overwrite || !self.states.contains_key(&name) {
                self.states.insert(name, state);
                count += 1;
            }
        }

        Ok(count)
    }
}

impl Default for LayerStateManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Layer state errors
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum LayerStateError {
    #[error("Layer state '{0}' not found")]
    StateNotFound(String),

    #[error("Layer state '{0}' already exists")]
    StateAlreadyExists(String),

    #[error("Invalid state name: {0}")]
    InvalidStateName(String),

    #[error("Failed to restore layer state: {0}")]
    RestoreError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_snapshot() {
        let mut layer = Layer::new("Test".to_string());
        layer.visible = false;
        layer.frozen = true;

        let snapshot = LayerSnapshot::from_layer(&layer);
        assert!(!snapshot.visible);
        assert!(snapshot.frozen);

        let mut layer2 = Layer::new("Test2".to_string());
        snapshot.apply_to_layer(&mut layer2);
        assert!(!layer2.visible);
        assert!(layer2.frozen);
    }

    #[test]
    fn test_layer_state_creation() {
        let manager = LayerManager::new();
        let state = LayerState::from_manager(
            "MyState".to_string(),
            "Test state".to_string(),
            &manager,
        );

        assert_eq!(state.name, "MyState");
        assert_eq!(state.layer_count(), 1);
    }

    #[test]
    fn test_layer_state_apply() {
        let mut manager1 = LayerManager::new();
        manager1.create_layer("Layer1".to_string()).unwrap();
        manager1.freeze_layer("Layer1").unwrap();

        let state = LayerState::from_manager(
            "State1".to_string(),
            "Test".to_string(),
            &manager1,
        );

        let mut manager2 = LayerManager::new();
        manager2.create_layer("Layer1".to_string()).unwrap();

        state.apply_to_manager(&mut manager2).unwrap();
        assert!(manager2.get_layer("Layer1").unwrap().frozen);
    }

    #[test]
    fn test_layer_state_manager() {
        let mut state_mgr = LayerStateManager::new();
        let mut layer_mgr = LayerManager::new();

        assert!(state_mgr
            .save_state("State1".to_string(), "Test".to_string(), &layer_mgr)
            .is_ok());
        assert_eq!(state_mgr.state_count(), 1);
    }

    #[test]
    fn test_save_and_restore_state() {
        let mut state_mgr = LayerStateManager::new();
        let mut layer_mgr = LayerManager::new();

        layer_mgr.create_layer("Layer1".to_string()).unwrap();
        layer_mgr.freeze_layer("Layer1").unwrap();

        state_mgr
            .save_state("Frozen".to_string(), "Frozen state".to_string(), &layer_mgr)
            .unwrap();

        layer_mgr.thaw_layer("Layer1").unwrap();
        assert!(!layer_mgr.get_layer("Layer1").unwrap().frozen);

        state_mgr.restore_state("Frozen", &mut layer_mgr).unwrap();
        assert!(layer_mgr.get_layer("Layer1").unwrap().frozen);
    }

    #[test]
    fn test_rename_state() {
        let mut state_mgr = LayerStateManager::new();
        let layer_mgr = LayerManager::new();

        state_mgr
            .save_state("Old".to_string(), "Test".to_string(), &layer_mgr)
            .unwrap();
        state_mgr.rename_state("Old", "New".to_string()).unwrap();

        assert!(state_mgr.has_state("New"));
        assert!(!state_mgr.has_state("Old"));
    }

    #[test]
    fn test_export_import_state() {
        let mut state_mgr = LayerStateManager::new();
        let layer_mgr = LayerManager::new();

        state_mgr
            .save_state("Test".to_string(), "Description".to_string(), &layer_mgr)
            .unwrap();

        let json = state_mgr.export_state("Test").unwrap();
        assert!(json.contains("Test"));

        let mut state_mgr2 = LayerStateManager::new();
        let name = state_mgr2.import_state(&json).unwrap();
        assert_eq!(name, "Test");
        assert!(state_mgr2.has_state("Test"));
    }

    #[test]
    fn test_duplicate_state_error() {
        let mut state_mgr = LayerStateManager::new();
        let layer_mgr = LayerManager::new();

        state_mgr
            .save_state("Dup".to_string(), "Test".to_string(), &layer_mgr)
            .unwrap();
        let result = state_mgr.save_state("Dup".to_string(), "Test".to_string(), &layer_mgr);
        assert!(result.is_err());
    }
}
