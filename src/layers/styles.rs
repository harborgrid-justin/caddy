// Line styles module for CADDY
// Provides line types, line weights, and custom pattern definitions

use serde::{Deserialize, Serialize};
use std::fmt;

/// Line type enumeration with AutoCAD standard line types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LineType {
    /// Continuous line (solid)
    Continuous,
    /// Dashed line
    Dashed,
    /// Hidden line (shorter dashes)
    Hidden,
    /// Center line (long-short-long)
    Center,
    /// Phantom line (long-short-short-long)
    Phantom,
    /// Dot line
    Dot,
    /// Dash-dot line
    DashDot,
    /// Dash-dot-dot line
    DashDotDot,
    /// Border line
    Border,
    /// Divide line
    Divide,
    /// Custom line pattern
    Custom(LinePattern),
    /// ByLayer - inherit from layer
    ByLayer,
    /// ByBlock - inherit from block
    ByBlock,
}

impl LineType {
    /// Get the pattern definition for this line type
    pub fn pattern(&self) -> Vec<f64> {
        match self {
            LineType::Continuous => vec![],
            LineType::Dashed => vec![0.5, -0.25],
            LineType::Hidden => vec![0.25, -0.125],
            LineType::Center => vec![1.25, -0.25, 0.25, -0.25],
            LineType::Phantom => vec![1.25, -0.25, 0.25, -0.25, 0.25, -0.25],
            LineType::Dot => vec![0.0, -0.125],
            LineType::DashDot => vec![0.5, -0.25, 0.0, -0.25],
            LineType::DashDotDot => vec![0.5, -0.25, 0.0, -0.25, 0.0, -0.25],
            LineType::Border => vec![0.5, -0.25, 0.5, -0.25, 0.0, -0.25],
            LineType::Divide => vec![0.5, -0.25, 0.0, -0.25, 0.0, -0.25],
            LineType::Custom(pattern) => pattern.segments.clone(),
            LineType::ByLayer | LineType::ByBlock => vec![],
        }
    }

    /// Get the name of this line type
    pub fn name(&self) -> &str {
        match self {
            LineType::Continuous => "CONTINUOUS",
            LineType::Dashed => "DASHED",
            LineType::Hidden => "HIDDEN",
            LineType::Center => "CENTER",
            LineType::Phantom => "PHANTOM",
            LineType::Dot => "DOT",
            LineType::DashDot => "DASHDOT",
            LineType::DashDotDot => "DASHDOTDOT",
            LineType::Border => "BORDER",
            LineType::Divide => "DIVIDE",
            LineType::Custom(pattern) => &pattern.name,
            LineType::ByLayer => "BYLAYER",
            LineType::ByBlock => "BYBLOCK",
        }
    }

    /// Create a line type from name
    pub fn from_name(name: &str) -> Self {
        match name.to_uppercase().as_str() {
            "CONTINUOUS" => LineType::Continuous,
            "DASHED" => LineType::Dashed,
            "HIDDEN" => LineType::Hidden,
            "CENTER" => LineType::Center,
            "PHANTOM" => LineType::Phantom,
            "DOT" => LineType::Dot,
            "DASHDOT" => LineType::DashDot,
            "DASHDOTDOT" => LineType::DashDotDot,
            "BORDER" => LineType::Border,
            "DIVIDE" => LineType::Divide,
            "BYLAYER" => LineType::ByLayer,
            "BYBLOCK" => LineType::ByBlock,
            _ => LineType::Continuous,
        }
    }

    /// Check if this is a special value (ByLayer/ByBlock)
    pub fn is_special(&self) -> bool {
        matches!(self, LineType::ByLayer | LineType::ByBlock)
    }
}

impl Default for LineType {
    fn default() -> Self {
        LineType::ByLayer
    }
}

impl fmt::Display for LineType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Custom line pattern definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinePattern {
    /// Pattern name
    pub name: String,
    /// Pattern description
    pub description: String,
    /// Pattern segments (positive = dash, negative = gap, zero = dot)
    pub segments: Vec<f64>,
}

impl LinePattern {
    /// Create a new custom line pattern
    pub fn new(name: String, description: String, segments: Vec<f64>) -> Self {
        Self {
            name,
            description,
            segments,
        }
    }

    /// Get the total length of one pattern repetition
    pub fn pattern_length(&self) -> f64 {
        self.segments.iter().map(|s| s.abs()).sum()
    }

    /// Validate the pattern
    pub fn is_valid(&self) -> bool {
        !self.segments.is_empty() && self.pattern_length() > 0.0
    }
}

/// Line weight enumeration (thickness in mm)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LineWeight {
    /// Line weight determined by layer
    ByLayer,
    /// Line weight determined by block
    ByBlock,
    /// Default line weight
    Default,
    /// 0.00mm (hairline)
    W0_00,
    /// 0.05mm
    W0_05,
    /// 0.09mm
    W0_09,
    /// 0.13mm
    W0_13,
    /// 0.15mm
    W0_15,
    /// 0.18mm
    W0_18,
    /// 0.20mm
    W0_20,
    /// 0.25mm
    W0_25,
    /// 0.30mm
    W0_30,
    /// 0.35mm
    W0_35,
    /// 0.40mm
    W0_40,
    /// 0.50mm
    W0_50,
    /// 0.53mm
    W0_53,
    /// 0.60mm
    W0_60,
    /// 0.70mm
    W0_70,
    /// 0.80mm
    W0_80,
    /// 0.90mm
    W0_90,
    /// 1.00mm
    W1_00,
    /// 1.06mm
    W1_06,
    /// 1.20mm
    W1_20,
    /// 1.40mm
    W1_40,
    /// 1.58mm
    W1_58,
    /// 2.00mm
    W2_00,
    /// 2.11mm
    W2_11,
}

impl LineWeight {
    /// Get the line weight value in millimeters
    pub fn to_mm(&self) -> Option<f64> {
        match self {
            LineWeight::ByLayer | LineWeight::ByBlock | LineWeight::Default => None,
            LineWeight::W0_00 => Some(0.00),
            LineWeight::W0_05 => Some(0.05),
            LineWeight::W0_09 => Some(0.09),
            LineWeight::W0_13 => Some(0.13),
            LineWeight::W0_15 => Some(0.15),
            LineWeight::W0_18 => Some(0.18),
            LineWeight::W0_20 => Some(0.20),
            LineWeight::W0_25 => Some(0.25),
            LineWeight::W0_30 => Some(0.30),
            LineWeight::W0_35 => Some(0.35),
            LineWeight::W0_40 => Some(0.40),
            LineWeight::W0_50 => Some(0.50),
            LineWeight::W0_53 => Some(0.53),
            LineWeight::W0_60 => Some(0.60),
            LineWeight::W0_70 => Some(0.70),
            LineWeight::W0_80 => Some(0.80),
            LineWeight::W0_90 => Some(0.90),
            LineWeight::W1_00 => Some(1.00),
            LineWeight::W1_06 => Some(1.06),
            LineWeight::W1_20 => Some(1.20),
            LineWeight::W1_40 => Some(1.40),
            LineWeight::W1_58 => Some(1.58),
            LineWeight::W2_00 => Some(2.00),
            LineWeight::W2_11 => Some(2.11),
        }
    }

    /// Create line weight from millimeter value (finds closest match)
    pub fn from_mm(mm: f64) -> Self {
        let weights = [
            (0.00, LineWeight::W0_00),
            (0.05, LineWeight::W0_05),
            (0.09, LineWeight::W0_09),
            (0.13, LineWeight::W0_13),
            (0.15, LineWeight::W0_15),
            (0.18, LineWeight::W0_18),
            (0.20, LineWeight::W0_20),
            (0.25, LineWeight::W0_25),
            (0.30, LineWeight::W0_30),
            (0.35, LineWeight::W0_35),
            (0.40, LineWeight::W0_40),
            (0.50, LineWeight::W0_50),
            (0.53, LineWeight::W0_53),
            (0.60, LineWeight::W0_60),
            (0.70, LineWeight::W0_70),
            (0.80, LineWeight::W0_80),
            (0.90, LineWeight::W0_90),
            (1.00, LineWeight::W1_00),
            (1.06, LineWeight::W1_06),
            (1.20, LineWeight::W1_20),
            (1.40, LineWeight::W1_40),
            (1.58, LineWeight::W1_58),
            (2.00, LineWeight::W2_00),
            (2.11, LineWeight::W2_11),
        ];

        weights
            .iter()
            .min_by(|(a, _), (b, _)| {
                (a - mm).abs().partial_cmp(&(b - mm).abs()).unwrap()
            })
            .map(|(_, w)| *w)
            .unwrap_or(LineWeight::Default)
    }

    /// Check if this is a special value (ByLayer/ByBlock/Default)
    pub fn is_special(&self) -> bool {
        matches!(
            self,
            LineWeight::ByLayer | LineWeight::ByBlock | LineWeight::Default
        )
    }

    /// Get all standard line weights
    pub fn standard_weights() -> Vec<LineWeight> {
        vec![
            LineWeight::W0_00,
            LineWeight::W0_05,
            LineWeight::W0_09,
            LineWeight::W0_13,
            LineWeight::W0_15,
            LineWeight::W0_18,
            LineWeight::W0_20,
            LineWeight::W0_25,
            LineWeight::W0_30,
            LineWeight::W0_35,
            LineWeight::W0_40,
            LineWeight::W0_50,
            LineWeight::W0_53,
            LineWeight::W0_60,
            LineWeight::W0_70,
            LineWeight::W0_80,
            LineWeight::W0_90,
            LineWeight::W1_00,
            LineWeight::W1_06,
            LineWeight::W1_20,
            LineWeight::W1_40,
            LineWeight::W1_58,
            LineWeight::W2_00,
            LineWeight::W2_11,
        ]
    }
}

impl Default for LineWeight {
    fn default() -> Self {
        LineWeight::ByLayer
    }
}

impl fmt::Display for LineWeight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LineWeight::ByLayer => write!(f, "ByLayer"),
            LineWeight::ByBlock => write!(f, "ByBlock"),
            LineWeight::Default => write!(f, "Default"),
            _ => {
                if let Some(mm) = self.to_mm() {
                    write!(f, "{:.2}mm", mm)
                } else {
                    write!(f, "Unknown")
                }
            }
        }
    }
}

/// Line type scale factor
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LineTypeScale {
    /// Global scale factor
    pub global: f64,
    /// Object-specific scale factor
    pub object: f64,
}

impl LineTypeScale {
    /// Create a new line type scale
    pub fn new(global: f64, object: f64) -> Self {
        Self { global, object }
    }

    /// Get the combined scale factor
    pub fn combined(&self) -> f64 {
        self.global * self.object
    }
}

impl Default for LineTypeScale {
    fn default() -> Self {
        Self {
            global: 1.0,
            object: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_type_pattern() {
        let dashed = LineType::Dashed;
        let pattern = dashed.pattern();
        assert!(!pattern.is_empty());
    }

    #[test]
    fn test_line_type_from_name() {
        let lt = LineType::from_name("DASHED");
        assert_eq!(lt, LineType::Dashed);
    }

    #[test]
    fn test_line_weight_conversion() {
        let weight = LineWeight::W0_25;
        assert_eq!(weight.to_mm(), Some(0.25));
    }

    #[test]
    fn test_line_weight_from_mm() {
        let weight = LineWeight::from_mm(0.26);
        assert_eq!(weight, LineWeight::W0_25);
    }

    #[test]
    fn test_custom_line_pattern() {
        let pattern = LinePattern::new(
            "CUSTOM".to_string(),
            "Custom pattern".to_string(),
            vec![0.5, -0.25, 0.125, -0.25],
        );
        assert!(pattern.is_valid());
        assert_eq!(pattern.pattern_length(), 1.125);
    }

    #[test]
    fn test_line_type_scale() {
        let scale = LineTypeScale::new(2.0, 1.5);
        assert_eq!(scale.combined(), 3.0);
    }
}
