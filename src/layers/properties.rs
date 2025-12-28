// Entity properties module for CADDY
// Provides property inheritance and override support (ByLayer, ByBlock, Direct)

use crate::core::Color;
use super::layer::Layer;
use super::styles::{LineType, LineWeight};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Color source for entities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColorSource {
    /// Inherit color from layer
    ByLayer,
    /// Inherit color from block
    ByBlock,
    /// Direct color assignment
    Direct(Color),
}

impl ColorSource {
    /// Resolve the actual color given a layer
    pub fn resolve(&self, layer: &Layer) -> Color {
        match self {
            ColorSource::ByLayer => layer.color,
            ColorSource::ByBlock => Color::BY_BLOCK,
            ColorSource::Direct(color) => *color,
        }
    }

    /// Check if this is a special value (ByLayer/ByBlock)
    pub fn is_special(&self) -> bool {
        matches!(self, ColorSource::ByLayer | ColorSource::ByBlock)
    }
}

impl Default for ColorSource {
    fn default() -> Self {
        ColorSource::ByLayer
    }
}

impl From<Color> for ColorSource {
    fn from(color: Color) -> Self {
        if color == Color::BY_LAYER {
            ColorSource::ByLayer
        } else if color == Color::BY_BLOCK {
            ColorSource::ByBlock
        } else {
            ColorSource::Direct(color)
        }
    }
}

impl fmt::Display for ColorSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColorSource::ByLayer => write!(f, "ByLayer"),
            ColorSource::ByBlock => write!(f, "ByBlock"),
            ColorSource::Direct(color) => write!(f, "{}", color),
        }
    }
}

/// Line type source for entities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LineTypeSource {
    /// Inherit line type from layer
    ByLayer,
    /// Inherit line type from block
    ByBlock,
    /// Direct line type assignment
    Direct(LineType),
}

impl LineTypeSource {
    /// Resolve the actual line type given a layer
    pub fn resolve(&self, layer: &Layer) -> LineType {
        match self {
            LineTypeSource::ByLayer => layer.line_type.clone(),
            LineTypeSource::ByBlock => LineType::ByBlock,
            LineTypeSource::Direct(line_type) => line_type.clone(),
        }
    }

    /// Check if this is a special value (ByLayer/ByBlock)
    pub fn is_special(&self) -> bool {
        matches!(self, LineTypeSource::ByLayer | LineTypeSource::ByBlock)
    }
}

impl Default for LineTypeSource {
    fn default() -> Self {
        LineTypeSource::ByLayer
    }
}

impl From<LineType> for LineTypeSource {
    fn from(line_type: LineType) -> Self {
        if line_type == LineType::ByLayer {
            LineTypeSource::ByLayer
        } else if line_type == LineType::ByBlock {
            LineTypeSource::ByBlock
        } else {
            LineTypeSource::Direct(line_type)
        }
    }
}

impl fmt::Display for LineTypeSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LineTypeSource::ByLayer => write!(f, "ByLayer"),
            LineTypeSource::ByBlock => write!(f, "ByBlock"),
            LineTypeSource::Direct(line_type) => write!(f, "{}", line_type),
        }
    }
}

/// Line weight source for entities
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LineWeightSource {
    /// Inherit line weight from layer
    ByLayer,
    /// Inherit line weight from block
    ByBlock,
    /// Direct line weight assignment
    Direct(LineWeight),
}

impl LineWeightSource {
    /// Resolve the actual line weight given a layer
    pub fn resolve(&self, layer: &Layer) -> LineWeight {
        match self {
            LineWeightSource::ByLayer => layer.line_weight,
            LineWeightSource::ByBlock => LineWeight::ByBlock,
            LineWeightSource::Direct(line_weight) => *line_weight,
        }
    }

    /// Check if this is a special value (ByLayer/ByBlock)
    pub fn is_special(&self) -> bool {
        matches!(
            self,
            LineWeightSource::ByLayer | LineWeightSource::ByBlock
        )
    }
}

impl Default for LineWeightSource {
    fn default() -> Self {
        LineWeightSource::ByLayer
    }
}

impl From<LineWeight> for LineWeightSource {
    fn from(line_weight: LineWeight) -> Self {
        if line_weight == LineWeight::ByLayer {
            LineWeightSource::ByLayer
        } else if line_weight == LineWeight::ByBlock {
            LineWeightSource::ByBlock
        } else {
            LineWeightSource::Direct(line_weight)
        }
    }
}

impl fmt::Display for LineWeightSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LineWeightSource::ByLayer => write!(f, "ByLayer"),
            LineWeightSource::ByBlock => write!(f, "ByBlock"),
            LineWeightSource::Direct(line_weight) => write!(f, "{}", line_weight),
        }
    }
}

/// Entity properties with layer and override support
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityProperties {
    /// Layer name this entity belongs to
    pub layer: String,

    /// Color source (ByLayer, ByBlock, or Direct)
    pub color: ColorSource,

    /// Line type source (ByLayer, ByBlock, or Direct)
    pub line_type: LineTypeSource,

    /// Line weight source (ByLayer, ByBlock, or Direct)
    pub line_weight: LineWeightSource,

    /// Transparency override (0 = opaque, 255 = fully transparent)
    /// None means use layer transparency
    pub transparency: Option<u8>,
}

impl EntityProperties {
    /// Create new entity properties on the specified layer
    pub fn new(layer: String) -> Self {
        Self {
            layer,
            color: ColorSource::ByLayer,
            line_type: LineTypeSource::ByLayer,
            line_weight: LineWeightSource::ByLayer,
            transparency: None,
        }
    }

    /// Create entity properties on the default layer "0"
    pub fn on_default_layer() -> Self {
        Self::new("0".to_string())
    }

    /// Set the layer
    pub fn set_layer(&mut self, layer: String) {
        self.layer = layer;
    }

    /// Set direct color
    pub fn set_color(&mut self, color: Color) {
        self.color = ColorSource::Direct(color);
    }

    /// Set color to ByLayer
    pub fn set_color_by_layer(&mut self) {
        self.color = ColorSource::ByLayer;
    }

    /// Set color to ByBlock
    pub fn set_color_by_block(&mut self) {
        self.color = ColorSource::ByBlock;
    }

    /// Set direct line type
    pub fn set_line_type(&mut self, line_type: LineType) {
        self.line_type = LineTypeSource::Direct(line_type);
    }

    /// Set line type to ByLayer
    pub fn set_line_type_by_layer(&mut self) {
        self.line_type = LineTypeSource::ByLayer;
    }

    /// Set line type to ByBlock
    pub fn set_line_type_by_block(&mut self) {
        self.line_type = LineTypeSource::ByBlock;
    }

    /// Set direct line weight
    pub fn set_line_weight(&mut self, line_weight: LineWeight) {
        self.line_weight = LineWeightSource::Direct(line_weight);
    }

    /// Set line weight to ByLayer
    pub fn set_line_weight_by_layer(&mut self) {
        self.line_weight = LineWeightSource::ByLayer;
    }

    /// Set line weight to ByBlock
    pub fn set_line_weight_by_block(&mut self) {
        self.line_weight = LineWeightSource::ByBlock;
    }

    /// Set transparency override
    pub fn set_transparency(&mut self, transparency: u8) {
        self.transparency = Some(transparency);
    }

    /// Clear transparency override (use layer transparency)
    pub fn clear_transparency(&mut self) {
        self.transparency = None;
    }

    /// Resolve all properties given a layer reference
    pub fn resolve(&self, layer: &Layer) -> ResolvedProperties {
        ResolvedProperties {
            layer: self.layer.clone(),
            color: self.color.resolve(layer),
            line_type: self.line_type.resolve(layer),
            line_weight: self.line_weight.resolve(layer),
            transparency: self.transparency.unwrap_or(layer.transparency),
        }
    }

    /// Check if all properties are set to ByLayer
    pub fn is_all_by_layer(&self) -> bool {
        self.color == ColorSource::ByLayer
            && self.line_type == LineTypeSource::ByLayer
            && self.line_weight == LineWeightSource::ByLayer
            && self.transparency.is_none()
    }

    /// Reset all properties to ByLayer
    pub fn reset_to_by_layer(&mut self) {
        self.color = ColorSource::ByLayer;
        self.line_type = LineTypeSource::ByLayer;
        self.line_weight = LineWeightSource::ByLayer;
        self.transparency = None;
    }
}

impl Default for EntityProperties {
    fn default() -> Self {
        Self::on_default_layer()
    }
}

impl fmt::Display for EntityProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Layer: {}, Color: {}, LineType: {}, LineWeight: {}",
            self.layer, self.color, self.line_type, self.line_weight
        )
    }
}

/// Resolved entity properties (all ByLayer/ByBlock references resolved)
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedProperties {
    pub layer: String,
    pub color: Color,
    pub line_type: LineType,
    pub line_weight: LineWeight,
    pub transparency: u8,
}

impl ResolvedProperties {
    /// Get the effective color with transparency applied
    pub fn effective_color(&self) -> Color {
        let mut color = self.color;
        color.a = 255 - self.transparency;
        color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_source_resolve() {
        let layer = Layer::new("TestLayer".to_string());
        let by_layer = ColorSource::ByLayer;
        assert_eq!(by_layer.resolve(&layer), layer.color);

        let direct = ColorSource::Direct(Color::RED);
        assert_eq!(direct.resolve(&layer), Color::RED);
    }

    #[test]
    fn test_entity_properties_creation() {
        let props = EntityProperties::new("MyLayer".to_string());
        assert_eq!(props.layer, "MyLayer");
        assert_eq!(props.color, ColorSource::ByLayer);
    }

    #[test]
    fn test_entity_properties_resolve() {
        let mut layer = Layer::new("TestLayer".to_string());
        layer.color = Color::BLUE;
        layer.line_weight = LineWeight::W0_25;

        let props = EntityProperties::new("TestLayer".to_string());
        let resolved = props.resolve(&layer);

        assert_eq!(resolved.color, Color::BLUE);
        assert_eq!(resolved.line_weight, LineWeight::W0_25);
    }

    #[test]
    fn test_property_override() {
        let mut layer = Layer::new("TestLayer".to_string());
        layer.color = Color::BLUE;

        let mut props = EntityProperties::new("TestLayer".to_string());
        props.set_color(Color::RED);

        let resolved = props.resolve(&layer);
        assert_eq!(resolved.color, Color::RED);
    }

    #[test]
    fn test_is_all_by_layer() {
        let props = EntityProperties::new("Test".to_string());
        assert!(props.is_all_by_layer());

        let mut props2 = EntityProperties::new("Test".to_string());
        props2.set_color(Color::RED);
        assert!(!props2.is_all_by_layer());
    }

    #[test]
    fn test_reset_to_by_layer() {
        let mut props = EntityProperties::new("Test".to_string());
        props.set_color(Color::RED);
        props.set_line_weight(LineWeight::W0_50);
        props.set_transparency(128);

        props.reset_to_by_layer();
        assert!(props.is_all_by_layer());
    }

    #[test]
    fn test_transparency() {
        let layer = Layer::default_layer();
        let mut props = EntityProperties::on_default_layer();
        props.set_transparency(128);

        let resolved = props.resolve(&layer);
        assert_eq!(resolved.transparency, 128);

        let effective = resolved.effective_color();
        assert_eq!(effective.a, 127);
    }
}
