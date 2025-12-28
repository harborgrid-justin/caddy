// Layer module for CADDY
// Provides layer type with properties and operations

use crate::core::Color;
use super::styles::{LineType, LineWeight};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Layer in a CAD drawing
/// Layers organize entities and control their visual properties
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    /// Layer name (must be unique within a drawing)
    pub name: String,

    /// Layer color (applied to entities with color set to ByLayer)
    pub color: Color,

    /// Layer line type (applied to entities with line type set to ByLayer)
    pub line_type: LineType,

    /// Layer line weight (applied to entities with line weight set to ByLayer)
    pub line_weight: LineWeight,

    /// Visibility flag - if false, entities on this layer are not displayed
    pub visible: bool,

    /// Frozen flag - if true, entities on this layer are not displayed and cannot be selected
    /// Frozen layers are also not considered in regeneration
    pub frozen: bool,

    /// Locked flag - if true, entities on this layer are displayed but cannot be modified
    pub locked: bool,

    /// Printable flag - if false, layer is not included in plots
    pub printable: bool,

    /// Layer description (optional metadata)
    pub description: String,

    /// Transparency (0 = opaque, 255 = fully transparent)
    pub transparency: u8,
}

impl Layer {
    /// Create a new layer with the given name
    pub fn new(name: String) -> Self {
        Self {
            name,
            color: Color::WHITE,
            line_type: LineType::Continuous,
            line_weight: LineWeight::Default,
            visible: true,
            frozen: false,
            locked: false,
            printable: true,
            description: String::new(),
            transparency: 0,
        }
    }

    /// Create the default layer "0"
    /// Layer "0" is special in AutoCAD and cannot be deleted or renamed
    pub fn default_layer() -> Self {
        Self {
            name: "0".to_string(),
            color: Color::WHITE,
            line_type: LineType::Continuous,
            line_weight: LineWeight::Default,
            visible: true,
            frozen: false,
            locked: false,
            printable: true,
            description: "Default layer".to_string(),
            transparency: 0,
        }
    }

    /// Check if this is the default layer "0"
    pub fn is_default_layer(&self) -> bool {
        self.name == "0"
    }

    /// Check if layer is effectively visible (not frozen and visible)
    pub fn is_visible(&self) -> bool {
        self.visible && !self.frozen
    }

    /// Check if layer can be edited
    pub fn is_editable(&self) -> bool {
        !self.locked && !self.frozen
    }

    /// Rename the layer (validates that it's not the default layer)
    pub fn rename(&mut self, new_name: String) -> Result<(), LayerError> {
        if self.is_default_layer() {
            return Err(LayerError::CannotRenameDefaultLayer);
        }
        if new_name.is_empty() {
            return Err(LayerError::InvalidLayerName("Name cannot be empty".to_string()));
        }
        if new_name.contains(|c: char| c.is_whitespace() || c.is_control()) {
            return Err(LayerError::InvalidLayerName(
                "Name cannot contain whitespace or control characters".to_string(),
            ));
        }
        self.name = new_name;
        Ok(())
    }

    /// Create a copy of this layer with a new name
    pub fn copy_with_name(&self, new_name: String) -> Result<Self, LayerError> {
        if new_name.is_empty() {
            return Err(LayerError::InvalidLayerName("Name cannot be empty".to_string()));
        }
        let mut layer = self.clone();
        layer.name = new_name;
        Ok(layer)
    }

    /// Set the layer color
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    /// Set the layer line type
    pub fn set_line_type(&mut self, line_type: LineType) {
        self.line_type = line_type;
    }

    /// Set the layer line weight
    pub fn set_line_weight(&mut self, line_weight: LineWeight) {
        self.line_weight = line_weight;
    }

    /// Toggle visibility
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    /// Toggle frozen state
    pub fn toggle_frozen(&mut self) {
        self.frozen = !self.frozen;
    }

    /// Toggle locked state
    pub fn toggle_locked(&mut self) {
        self.locked = !self.locked;
    }

    /// Toggle printable state
    pub fn toggle_printable(&mut self) {
        self.printable = !self.printable;
    }

    /// Set transparency (0 = opaque, 255 = fully transparent)
    pub fn set_transparency(&mut self, transparency: u8) {
        self.transparency = transparency;
    }

    /// Validate layer name
    pub fn validate_name(name: &str) -> Result<(), LayerError> {
        if name.is_empty() {
            return Err(LayerError::InvalidLayerName("Name cannot be empty".to_string()));
        }
        if name.len() > 255 {
            return Err(LayerError::InvalidLayerName("Name too long (max 255 characters)".to_string()));
        }
        if name.contains(|c: char| c == '<' || c == '>' || c == '/' || c == '\\' || c == '"' || c == ':' || c == ';' || c == '?' || c == '*' || c == '|' || c == '=' || c == '`') {
            return Err(LayerError::InvalidLayerName(
                "Name contains invalid characters".to_string(),
            ));
        }
        Ok(())
    }

    /// Get a summary of the layer state
    pub fn status_summary(&self) -> String {
        let mut flags = Vec::new();
        if self.frozen {
            flags.push("Frozen");
        }
        if self.locked {
            flags.push("Locked");
        }
        if !self.visible {
            flags.push("Hidden");
        }
        if !self.printable {
            flags.push("No Plot");
        }
        if flags.is_empty() {
            "Active".to_string()
        } else {
            flags.join(", ")
        }
    }
}

impl Default for Layer {
    fn default() -> Self {
        Layer::default_layer()
    }
}

impl fmt::Display for Layer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Layer '{}' [{}] - {}",
            self.name,
            self.color,
            self.status_summary()
        )
    }
}

/// Layer-related errors
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum LayerError {
    #[error("Cannot rename default layer '0'")]
    CannotRenameDefaultLayer,

    #[error("Cannot delete default layer '0'")]
    CannotDeleteDefaultLayer,

    #[error("Layer '{0}' not found")]
    LayerNotFound(String),

    #[error("Layer '{0}' already exists")]
    LayerAlreadyExists(String),

    #[error("Invalid layer name: {0}")]
    InvalidLayerName(String),

    #[error("Layer '{0}' is in use and cannot be deleted")]
    LayerInUse(String),

    #[error("Cannot perform operation on frozen layer '{0}'")]
    LayerFrozen(String),

    #[error("Cannot perform operation on locked layer '{0}'")]
    LayerLocked(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_creation() {
        let layer = Layer::new("MyLayer".to_string());
        assert_eq!(layer.name, "MyLayer");
        assert!(layer.visible);
        assert!(!layer.frozen);
        assert!(!layer.locked);
        assert!(layer.printable);
    }

    #[test]
    fn test_default_layer() {
        let layer = Layer::default_layer();
        assert_eq!(layer.name, "0");
        assert!(layer.is_default_layer());
    }

    #[test]
    fn test_layer_rename() {
        let mut layer = Layer::new("Old".to_string());
        assert!(layer.rename("New".to_string()).is_ok());
        assert_eq!(layer.name, "New");
    }

    #[test]
    fn test_cannot_rename_default_layer() {
        let mut layer = Layer::default_layer();
        assert!(layer.rename("NewName".to_string()).is_err());
    }

    #[test]
    fn test_layer_visibility() {
        let mut layer = Layer::new("Test".to_string());
        assert!(layer.is_visible());

        layer.frozen = true;
        assert!(!layer.is_visible());

        layer.frozen = false;
        layer.visible = false;
        assert!(!layer.is_visible());
    }

    #[test]
    fn test_layer_editable() {
        let mut layer = Layer::new("Test".to_string());
        assert!(layer.is_editable());

        layer.locked = true;
        assert!(!layer.is_editable());

        layer.locked = false;
        layer.frozen = true;
        assert!(!layer.is_editable());
    }

    #[test]
    fn test_layer_name_validation() {
        assert!(Layer::validate_name("ValidName").is_ok());
        assert!(Layer::validate_name("").is_err());
        assert!(Layer::validate_name("Invalid<Name").is_err());
        assert!(Layer::validate_name("Invalid*Name").is_err());
    }

    #[test]
    fn test_layer_copy() {
        let layer = Layer::new("Original".to_string());
        let copy = layer.copy_with_name("Copy".to_string()).unwrap();
        assert_eq!(copy.name, "Copy");
        assert_eq!(copy.color, layer.color);
        assert_eq!(copy.line_type, layer.line_type);
    }

    #[test]
    fn test_layer_toggles() {
        let mut layer = Layer::new("Test".to_string());

        layer.toggle_visibility();
        assert!(!layer.visible);

        layer.toggle_frozen();
        assert!(layer.frozen);

        layer.toggle_locked();
        assert!(layer.locked);

        layer.toggle_printable();
        assert!(!layer.printable);
    }
}
