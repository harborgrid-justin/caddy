// Layer manager module for CADDY
// Provides centralized layer management and operations

use super::layer::{Layer, LayerError};
use super::styles::{LineType, LineWeight};
use crate::core::Color;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Layer manager - manages all layers in a drawing
#[derive(Serialize, Deserialize)]
pub struct LayerManager {
    /// Map of layer name to layer
    layers: HashMap<String, Layer>,

    /// Current active layer name
    current_layer: String,

    /// Event listeners for layer changes (not serialized or cloned)
    #[serde(skip)]
    listeners: Vec<Box<dyn LayerEventListener>>,
}

impl std::fmt::Debug for LayerManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LayerManager")
            .field("layers", &self.layers)
            .field("current_layer", &self.current_layer)
            .field("listeners", &format!("[{} listeners]", self.listeners.len()))
            .finish()
    }
}

impl Clone for LayerManager {
    fn clone(&self) -> Self {
        Self {
            layers: self.layers.clone(),
            current_layer: self.current_layer.clone(),
            listeners: Vec::new(), // Listeners are not cloned
        }
    }
}

impl LayerManager {
    /// Create a new layer manager with the default layer "0"
    pub fn new() -> Self {
        let mut layers = HashMap::new();
        let default_layer = Layer::default_layer();
        layers.insert("0".to_string(), default_layer);

        Self {
            layers,
            current_layer: "0".to_string(),
            listeners: Vec::new(),
        }
    }

    /// Get the current layer
    pub fn current_layer(&self) -> &Layer {
        self.layers
            .get(&self.current_layer)
            .expect("Current layer must always exist")
    }

    /// Get the current layer name
    pub fn current_layer_name(&self) -> &str {
        &self.current_layer
    }

    /// Set the current layer by name
    pub fn set_current_layer(&mut self, name: &str) -> Result<(), LayerError> {
        if !self.layers.contains_key(name) {
            return Err(LayerError::LayerNotFound(name.to_string()));
        }

        // Cannot set frozen layer as current
        if let Some(layer) = self.layers.get(name) {
            if layer.frozen {
                return Err(LayerError::LayerFrozen(name.to_string()));
            }
        }

        let old_layer = self.current_layer.clone();
        self.current_layer = name.to_string();
        self.notify_current_layer_changed(&old_layer, name);
        Ok(())
    }

    /// Create a new layer
    pub fn create_layer(&mut self, name: String) -> Result<&Layer, LayerError> {
        // Validate name
        Layer::validate_name(&name)?;

        // Check if layer already exists
        if self.layers.contains_key(&name) {
            return Err(LayerError::LayerAlreadyExists(name));
        }

        // Create layer
        let layer = Layer::new(name.clone());
        self.layers.insert(name.clone(), layer);
        self.notify_layer_added(&name);

        Ok(self.layers.get(&name).unwrap())
    }

    /// Create a new layer with specific properties
    pub fn create_layer_with_properties(
        &mut self,
        name: String,
        color: Color,
        line_type: LineType,
        line_weight: LineWeight,
    ) -> Result<&Layer, LayerError> {
        // Validate name
        Layer::validate_name(&name)?;

        // Check if layer already exists
        if self.layers.contains_key(&name) {
            return Err(LayerError::LayerAlreadyExists(name));
        }

        // Create layer with properties
        let mut layer = Layer::new(name.clone());
        layer.color = color;
        layer.line_type = line_type;
        layer.line_weight = line_weight;

        self.layers.insert(name.clone(), layer);
        self.notify_layer_added(&name);

        Ok(self.layers.get(&name).unwrap())
    }

    /// Delete a layer
    pub fn delete_layer(&mut self, name: &str) -> Result<(), LayerError> {
        // Cannot delete default layer
        if name == "0" {
            return Err(LayerError::CannotDeleteDefaultLayer);
        }

        // Check if layer exists
        if !self.layers.contains_key(name) {
            return Err(LayerError::LayerNotFound(name.to_string()));
        }

        // If deleting current layer, switch to layer "0"
        if self.current_layer == name {
            self.current_layer = "0".to_string();
        }

        self.layers.remove(name);
        self.notify_layer_deleted(name);
        Ok(())
    }

    /// Rename a layer
    pub fn rename_layer(&mut self, old_name: &str, new_name: String) -> Result<(), LayerError> {
        // Validate new name
        Layer::validate_name(&new_name)?;

        // Check if old layer exists
        if !self.layers.contains_key(old_name) {
            return Err(LayerError::LayerNotFound(old_name.to_string()));
        }

        // Check if new name already exists
        if self.layers.contains_key(&new_name) {
            return Err(LayerError::LayerAlreadyExists(new_name));
        }

        // Cannot rename default layer
        if old_name == "0" {
            return Err(LayerError::CannotRenameDefaultLayer);
        }

        // Remove and re-insert with new name
        if let Some(mut layer) = self.layers.remove(old_name) {
            layer.name = new_name.clone();
            self.layers.insert(new_name.clone(), layer);

            // Update current layer if needed
            if self.current_layer == old_name {
                self.current_layer = new_name.clone();
            }

            self.notify_layer_renamed(old_name, &new_name);
        }

        Ok(())
    }

    /// Get a layer by name
    pub fn get_layer(&self, name: &str) -> Option<&Layer> {
        self.layers.get(name)
    }

    /// Get a mutable layer by name
    pub fn get_layer_mut(&mut self, name: &str) -> Option<&mut Layer> {
        self.layers.get_mut(name)
    }

    /// Check if a layer exists
    pub fn has_layer(&self, name: &str) -> bool {
        self.layers.contains_key(name)
    }

    /// Get all layer names
    pub fn layer_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.layers.keys().cloned().collect();
        names.sort();
        names
    }

    /// Get all layers
    pub fn layers(&self) -> Vec<&Layer> {
        let mut layers: Vec<&Layer> = self.layers.values().collect();
        layers.sort_by(|a, b| a.name.cmp(&b.name));
        layers
    }

    /// Get all mutable layers
    pub fn layers_mut(&mut self) -> Vec<&mut Layer> {
        self.layers.values_mut().collect()
    }

    /// Get number of layers
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Iterate over all layers
    pub fn iter(&self) -> impl Iterator<Item = &Layer> {
        self.layers.values()
    }

    /// Freeze a layer
    pub fn freeze_layer(&mut self, name: &str) -> Result<(), LayerError> {
        if let Some(layer) = self.layers.get_mut(name) {
            layer.frozen = true;

            // If freezing current layer, switch to layer "0"
            if self.current_layer == name {
                self.current_layer = "0".to_string();
            }

            self.notify_layer_modified(name);
            Ok(())
        } else {
            Err(LayerError::LayerNotFound(name.to_string()))
        }
    }

    /// Thaw (unfreeze) a layer
    pub fn thaw_layer(&mut self, name: &str) -> Result<(), LayerError> {
        if let Some(layer) = self.layers.get_mut(name) {
            layer.frozen = false;
            self.notify_layer_modified(name);
            Ok(())
        } else {
            Err(LayerError::LayerNotFound(name.to_string()))
        }
    }

    /// Lock a layer
    pub fn lock_layer(&mut self, name: &str) -> Result<(), LayerError> {
        if let Some(layer) = self.layers.get_mut(name) {
            layer.locked = true;
            self.notify_layer_modified(name);
            Ok(())
        } else {
            Err(LayerError::LayerNotFound(name.to_string()))
        }
    }

    /// Unlock a layer
    pub fn unlock_layer(&mut self, name: &str) -> Result<(), LayerError> {
        if let Some(layer) = self.layers.get_mut(name) {
            layer.locked = false;
            self.notify_layer_modified(name);
            Ok(())
        } else {
            Err(LayerError::LayerNotFound(name.to_string()))
        }
    }

    /// Toggle layer visibility
    pub fn toggle_layer_visibility(&mut self, name: &str) -> Result<(), LayerError> {
        if let Some(layer) = self.layers.get_mut(name) {
            layer.visible = !layer.visible;
            self.notify_layer_modified(name);
            Ok(())
        } else {
            Err(LayerError::LayerNotFound(name.to_string()))
        }
    }

    /// Purge unused layers (layers with no entities)
    /// Note: This requires entity usage tracking which would be implemented elsewhere
    /// For now, this is a placeholder that doesn't delete layers
    pub fn purge_unused_layers(&mut self, used_layers: &[String]) -> usize {
        let mut deleted_count = 0;
        let mut to_delete = Vec::new();

        for name in self.layers.keys() {
            // Never delete default layer
            if name == "0" {
                continue;
            }

            // Delete if not in used list
            if !used_layers.contains(name) {
                to_delete.push(name.clone());
            }
        }

        for name in to_delete {
            if self.delete_layer(&name).is_ok() {
                deleted_count += 1;
            }
        }

        deleted_count
    }

    /// Freeze all layers except the specified one
    pub fn isolate_layer(&mut self, name: &str) -> Result<(), LayerError> {
        if !self.layers.contains_key(name) {
            return Err(LayerError::LayerNotFound(name.to_string()));
        }

        for (layer_name, layer) in &mut self.layers {
            if layer_name != name {
                layer.frozen = true;
            } else {
                layer.frozen = false;
            }
        }

        self.set_current_layer(name)?;
        Ok(())
    }

    /// Thaw all layers
    pub fn thaw_all_layers(&mut self) {
        for layer in self.layers.values_mut() {
            layer.frozen = false;
        }
    }

    /// Turn on all layers
    pub fn turn_on_all_layers(&mut self) {
        for layer in self.layers.values_mut() {
            layer.visible = true;
        }
    }

    /// Lock all layers except the specified one
    pub fn lock_all_except(&mut self, name: &str) -> Result<(), LayerError> {
        if !self.layers.contains_key(name) {
            return Err(LayerError::LayerNotFound(name.to_string()));
        }

        for (layer_name, layer) in &mut self.layers {
            layer.locked = layer_name != name;
        }

        Ok(())
    }

    /// Unlock all layers
    pub fn unlock_all_layers(&mut self) {
        for layer in self.layers.values_mut() {
            layer.locked = false;
        }
    }

    // Event system for UI updates

    /// Add an event listener
    pub fn add_listener(&mut self, listener: Box<dyn LayerEventListener>) {
        self.listeners.push(listener);
    }

    fn notify_layer_added(&mut self, name: &str) {
        for listener in &self.listeners {
            listener.on_layer_added(name);
        }
    }

    fn notify_layer_deleted(&mut self, name: &str) {
        for listener in &self.listeners {
            listener.on_layer_deleted(name);
        }
    }

    fn notify_layer_modified(&mut self, name: &str) {
        for listener in &self.listeners {
            listener.on_layer_modified(name);
        }
    }

    fn notify_layer_renamed(&mut self, old_name: &str, new_name: &str) {
        for listener in &self.listeners {
            listener.on_layer_renamed(old_name, new_name);
        }
    }

    fn notify_current_layer_changed(&mut self, old_name: &str, new_name: &str) {
        for listener in &self.listeners {
            listener.on_current_layer_changed(old_name, new_name);
        }
    }
}

impl Default for LayerManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for layer event listeners (for UI updates)
pub trait LayerEventListener: Send + Sync {
    fn on_layer_added(&self, name: &str);
    fn on_layer_deleted(&self, name: &str);
    fn on_layer_modified(&self, name: &str);
    fn on_layer_renamed(&self, old_name: &str, new_name: &str);
    fn on_current_layer_changed(&self, old_name: &str, new_name: &str);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_manager_creation() {
        let manager = LayerManager::new();
        assert_eq!(manager.layer_count(), 1);
        assert_eq!(manager.current_layer_name(), "0");
    }

    #[test]
    fn test_create_layer() {
        let mut manager = LayerManager::new();
        assert!(manager.create_layer("NewLayer".to_string()).is_ok());
        assert_eq!(manager.layer_count(), 2);
        assert!(manager.has_layer("NewLayer"));
    }

    #[test]
    fn test_create_duplicate_layer() {
        let mut manager = LayerManager::new();
        manager.create_layer("Test".to_string()).unwrap();
        assert!(manager.create_layer("Test".to_string()).is_err());
    }

    #[test]
    fn test_delete_layer() {
        let mut manager = LayerManager::new();
        manager.create_layer("ToDelete".to_string()).unwrap();
        assert!(manager.delete_layer("ToDelete").is_ok());
        assert_eq!(manager.layer_count(), 1);
    }

    #[test]
    fn test_cannot_delete_default_layer() {
        let mut manager = LayerManager::new();
        assert!(manager.delete_layer("0").is_err());
    }

    #[test]
    fn test_rename_layer() {
        let mut manager = LayerManager::new();
        manager.create_layer("OldName".to_string()).unwrap();
        assert!(manager.rename_layer("OldName", "NewName".to_string()).is_ok());
        assert!(manager.has_layer("NewName"));
        assert!(!manager.has_layer("OldName"));
    }

    #[test]
    fn test_set_current_layer() {
        let mut manager = LayerManager::new();
        manager.create_layer("Layer1".to_string()).unwrap();
        assert!(manager.set_current_layer("Layer1").is_ok());
        assert_eq!(manager.current_layer_name(), "Layer1");
    }

    #[test]
    fn test_cannot_set_frozen_layer_as_current() {
        let mut manager = LayerManager::new();
        manager.create_layer("Frozen".to_string()).unwrap();
        manager.freeze_layer("Frozen").unwrap();
        assert!(manager.set_current_layer("Frozen").is_err());
    }

    #[test]
    fn test_freeze_thaw_layer() {
        let mut manager = LayerManager::new();
        manager.create_layer("Test".to_string()).unwrap();
        manager.freeze_layer("Test").unwrap();
        assert!(manager.get_layer("Test").unwrap().frozen);

        manager.thaw_layer("Test").unwrap();
        assert!(!manager.get_layer("Test").unwrap().frozen);
    }

    #[test]
    fn test_lock_unlock_layer() {
        let mut manager = LayerManager::new();
        manager.create_layer("Test".to_string()).unwrap();
        manager.lock_layer("Test").unwrap();
        assert!(manager.get_layer("Test").unwrap().locked);

        manager.unlock_layer("Test").unwrap();
        assert!(!manager.get_layer("Test").unwrap().locked);
    }

    #[test]
    fn test_isolate_layer() {
        let mut manager = LayerManager::new();
        manager.create_layer("Layer1".to_string()).unwrap();
        manager.create_layer("Layer2".to_string()).unwrap();

        manager.isolate_layer("Layer1").unwrap();
        assert!(!manager.get_layer("Layer1").unwrap().frozen);
        assert!(manager.get_layer("Layer2").unwrap().frozen);
        assert!(manager.get_layer("0").unwrap().frozen);
    }

    #[test]
    fn test_thaw_all_layers() {
        let mut manager = LayerManager::new();
        manager.create_layer("Layer1".to_string()).unwrap();
        manager.freeze_layer("Layer1").unwrap();

        manager.thaw_all_layers();
        assert!(!manager.get_layer("Layer1").unwrap().frozen);
    }

    #[test]
    fn test_purge_unused_layers() {
        let mut manager = LayerManager::new();
        manager.create_layer("Used".to_string()).unwrap();
        manager.create_layer("Unused".to_string()).unwrap();

        let used = vec!["0".to_string(), "Used".to_string()];
        let deleted = manager.purge_unused_layers(&used);
        assert_eq!(deleted, 1);
        assert!(!manager.has_layer("Unused"));
        assert!(manager.has_layer("Used"));
    }

    #[test]
    fn test_layer_iteration() {
        let mut manager = LayerManager::new();
        manager.create_layer("A".to_string()).unwrap();
        manager.create_layer("B".to_string()).unwrap();

        let names: Vec<String> = manager.iter().map(|l| l.name.clone()).collect();
        assert_eq!(names.len(), 3);
    }
}
