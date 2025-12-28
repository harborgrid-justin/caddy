//! Dimension styling system
//!
//! This module provides comprehensive dimension styling capabilities including
//! text formatting, arrow styles, extension lines, and standard templates.

use serde::{Deserialize, Serialize};

/// Arrow type for dimension lines
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArrowType {
    /// Closed filled arrow
    ClosedFilled,
    /// Closed blank arrow
    ClosedBlank,
    /// Open arrow
    Open,
    /// Dot
    Dot,
    /// Architectural tick
    ArchTick,
    /// Oblique slash
    Oblique,
    /// Right angle
    RightAngle,
    /// No arrow
    None,
}

/// Text alignment relative to dimension line
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DimTextAlignment {
    /// Text centered on dimension line
    Centered,
    /// Text at first extension line
    AtExtLine1,
    /// Text at second extension line
    AtExtLine2,
    /// Text above dimension line
    Above,
    /// Text outside, away from extension lines
    Outside,
}

/// Text vertical position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextVerticalPosition {
    /// Centered on dimension line
    Centered,
    /// Above dimension line
    Above,
    /// Outside extension lines
    Outside,
    /// JIS standard (Japanese Industrial Standard)
    JIS,
}

/// Unit format for dimension text
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitFormat {
    /// Scientific notation
    Scientific,
    /// Decimal
    Decimal,
    /// Engineering (feet and decimal inches)
    Engineering,
    /// Architectural (feet and fractional inches)
    Architectural,
    /// Fractional
    Fractional,
    /// Windows desktop units
    WindowsDesktop,
}

/// Angular unit format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AngularUnitFormat {
    /// Decimal degrees
    DecimalDegrees,
    /// Degrees/Minutes/Seconds
    DegMinSec,
    /// Gradians
    Gradians,
    /// Radians
    Radians,
    /// Surveyor's units
    Surveyors,
}

/// Tolerance display format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToleranceFormat {
    /// No tolerance display
    None,
    /// Symmetrical tolerance (±value)
    Symmetrical,
    /// Deviation tolerance (+upper/-lower)
    Deviation,
    /// Limits (show max and min)
    Limits,
    /// Basic dimension (boxed)
    Basic,
}

/// Color representation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const RED: Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Color = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE: Color = Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const YELLOW: Color = Color { r: 1.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const CYAN: Color = Color { r: 0.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const MAGENTA: Color = Color { r: 1.0, g: 0.0, b: 1.0, a: 1.0 };

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: 1.0,
        }
    }
}

/// Complete dimension style configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimensionStyle {
    /// Style name
    pub name: String,

    // Text properties
    /// Text height in drawing units
    pub text_height: f64,
    /// Text color
    pub text_color: Color,
    /// Font name
    pub font_name: String,
    /// Text alignment
    pub text_alignment: DimTextAlignment,
    /// Text vertical position
    pub text_vertical_position: TextVerticalPosition,
    /// Gap between text and dimension line
    pub text_gap: f64,
    /// Draw text frame/box
    pub text_frame: bool,

    // Arrow properties
    /// Arrow type at first point
    pub arrow_type_1: ArrowType,
    /// Arrow type at second point
    pub arrow_type_2: ArrowType,
    /// Arrow size
    pub arrow_size: f64,
    /// Center mark size for radii/diameters
    pub center_mark_size: f64,

    // Dimension line properties
    /// Dimension line color
    pub dim_line_color: Color,
    /// Dimension line lineweight
    pub dim_line_weight: f64,
    /// Extension beyond extension lines
    pub dim_line_extend: f64,
    /// Suppress first dimension line
    pub suppress_dim_line_1: bool,
    /// Suppress second dimension line
    pub suppress_dim_line_2: bool,

    // Extension line properties
    /// Extension line color
    pub ext_line_color: Color,
    /// Extension line lineweight
    pub ext_line_weight: f64,
    /// Extension beyond dimension line
    pub ext_line_extend: f64,
    /// Offset from origin point
    pub ext_line_offset: f64,
    /// Suppress first extension line
    pub suppress_ext_line_1: bool,
    /// Suppress second extension line
    pub suppress_ext_line_2: bool,
    /// Fixed length extension lines
    pub ext_line_fixed_length: Option<f64>,

    // Unit formatting
    /// Linear unit format
    pub unit_format: UnitFormat,
    /// Decimal precision (number of decimal places)
    pub precision: u8,
    /// Angular unit format
    pub angular_unit_format: AngularUnitFormat,
    /// Angular precision
    pub angular_precision: u8,
    /// Unit scale factor
    pub scale_factor: f64,
    /// Prefix for dimension text
    pub prefix: String,
    /// Suffix for dimension text
    pub suffix: String,
    /// Suppress leading zeros
    pub suppress_leading_zeros: bool,
    /// Suppress trailing zeros
    pub suppress_trailing_zeros: bool,

    // Tolerance
    /// Tolerance format
    pub tolerance_format: ToleranceFormat,
    /// Upper tolerance value
    pub tolerance_upper: f64,
    /// Lower tolerance value
    pub tolerance_lower: f64,
    /// Tolerance precision
    pub tolerance_precision: u8,
    /// Tolerance text height scale
    pub tolerance_text_height: f64,

    // Alternate units
    /// Enable alternate units
    pub alt_units_enabled: bool,
    /// Alternate unit scale factor
    pub alt_units_scale: f64,
    /// Alternate unit precision
    pub alt_units_precision: u8,
    /// Alternate unit prefix
    pub alt_units_prefix: String,
    /// Alternate unit suffix
    pub alt_units_suffix: String,

    // Style inheritance
    /// Parent style name (for inheritance)
    pub parent_style: Option<String>,
}

impl DimensionStyle {
    /// Create a new dimension style with default values
    pub fn new(name: impl Into<String>) -> Self {
        DimensionStyle {
            name: name.into(),

            // Text defaults
            text_height: 2.5,
            text_color: Color::WHITE,
            font_name: "Arial".to_string(),
            text_alignment: DimTextAlignment::Centered,
            text_vertical_position: TextVerticalPosition::Above,
            text_gap: 0.625,
            text_frame: false,

            // Arrow defaults
            arrow_type_1: ArrowType::ClosedFilled,
            arrow_type_2: ArrowType::ClosedFilled,
            arrow_size: 2.5,
            center_mark_size: 2.5,

            // Dimension line defaults
            dim_line_color: Color::WHITE,
            dim_line_weight: 0.25,
            dim_line_extend: 0.0,
            suppress_dim_line_1: false,
            suppress_dim_line_2: false,

            // Extension line defaults
            ext_line_color: Color::WHITE,
            ext_line_weight: 0.25,
            ext_line_extend: 1.25,
            ext_line_offset: 0.625,
            suppress_ext_line_1: false,
            suppress_ext_line_2: false,
            ext_line_fixed_length: None,

            // Unit formatting defaults
            unit_format: UnitFormat::Decimal,
            precision: 2,
            angular_unit_format: AngularUnitFormat::DecimalDegrees,
            angular_precision: 0,
            scale_factor: 1.0,
            prefix: String::new(),
            suffix: String::new(),
            suppress_leading_zeros: false,
            suppress_trailing_zeros: true,

            // Tolerance defaults
            tolerance_format: ToleranceFormat::None,
            tolerance_upper: 0.0,
            tolerance_lower: 0.0,
            tolerance_precision: 2,
            tolerance_text_height: 0.7,

            // Alternate units defaults
            alt_units_enabled: false,
            alt_units_scale: 25.4, // mm to inches
            alt_units_precision: 2,
            alt_units_prefix: "[".to_string(),
            alt_units_suffix: "]".to_string(),

            parent_style: None,
        }
    }

    /// Create ISO standard dimension style
    pub fn iso() -> Self {
        let mut style = DimensionStyle::new("ISO-25");
        style.text_height = 2.5;
        style.arrow_size = 2.5;
        style.arrow_type_1 = ArrowType::ClosedFilled;
        style.arrow_type_2 = ArrowType::ClosedFilled;
        style.ext_line_extend = 1.25;
        style.ext_line_offset = 0.625;
        style.dim_line_extend = 0.0;
        style.precision = 2;
        style.suppress_trailing_zeros = true;
        style
    }

    /// Create ANSI standard dimension style
    pub fn ansi() -> Self {
        let mut style = DimensionStyle::new("ANSI");
        style.text_height = 0.18;
        style.arrow_size = 0.18;
        style.arrow_type_1 = ArrowType::ClosedFilled;
        style.arrow_type_2 = ArrowType::ClosedFilled;
        style.ext_line_extend = 0.18;
        style.ext_line_offset = 0.0625;
        style.dim_line_extend = 0.0;
        style.unit_format = UnitFormat::Decimal;
        style.precision = 2;
        style.suppress_trailing_zeros = true;
        style
    }

    /// Create DIN standard dimension style (German)
    pub fn din() -> Self {
        let mut style = DimensionStyle::new("DIN");
        style.text_height = 3.5;
        style.arrow_size = 3.0;
        style.arrow_type_1 = ArrowType::ClosedFilled;
        style.arrow_type_2 = ArrowType::ClosedFilled;
        style.ext_line_extend = 2.0;
        style.ext_line_offset = 1.0;
        style.precision = 2;
        style.suppress_trailing_zeros = true;
        style
    }

    /// Create JIS standard dimension style (Japanese)
    pub fn jis() -> Self {
        let mut style = DimensionStyle::new("JIS");
        style.text_height = 2.5;
        style.arrow_size = 2.5;
        style.arrow_type_1 = ArrowType::ClosedFilled;
        style.arrow_type_2 = ArrowType::ClosedFilled;
        style.text_vertical_position = TextVerticalPosition::JIS;
        style.ext_line_extend = 1.25;
        style.ext_line_offset = 0.625;
        style.precision = 0;
        style
    }

    /// Create architectural dimension style
    pub fn architectural() -> Self {
        let mut style = DimensionStyle::new("Architectural");
        style.text_height = 0.125;
        style.arrow_size = 0.125;
        style.arrow_type_1 = ArrowType::ArchTick;
        style.arrow_type_2 = ArrowType::ArchTick;
        style.ext_line_extend = 0.125;
        style.ext_line_offset = 0.0625;
        style.unit_format = UnitFormat::Architectural;
        style.precision = 4;
        style.suffix = "\"".to_string();
        style
    }

    /// Inherit properties from parent style
    pub fn inherit_from(&mut self, parent: &DimensionStyle) {
        // This would selectively copy properties from parent
        // For now, we'll just note the parent relationship
        self.parent_style = Some(parent.name.clone());
    }

    /// Format a linear measurement value according to this style
    pub fn format_linear(&self, value: f64) -> String {
        let scaled_value = value * self.scale_factor;
        let formatted = match self.unit_format {
            UnitFormat::Decimal | UnitFormat::Scientific => {
                self.format_decimal(scaled_value)
            }
            UnitFormat::Engineering => {
                self.format_engineering(scaled_value)
            }
            UnitFormat::Architectural => {
                self.format_architectural(scaled_value)
            }
            UnitFormat::Fractional => {
                self.format_fractional(scaled_value)
            }
            UnitFormat::WindowsDesktop => {
                self.format_decimal(scaled_value)
            }
        };

        let mut result = format!("{}{}{}", self.prefix, formatted, self.suffix);

        // Add tolerance if enabled
        if self.tolerance_format != ToleranceFormat::None {
            result = self.add_tolerance(result, scaled_value);
        }

        // Add alternate units if enabled
        if self.alt_units_enabled {
            let alt_value = scaled_value * self.alt_units_scale;
            let alt_formatted = self.format_decimal_with_precision(alt_value, self.alt_units_precision);
            result = format!("{} {}{}{}",
                result,
                self.alt_units_prefix,
                alt_formatted,
                self.alt_units_suffix
            );
        }

        result
    }

    /// Format an angular measurement (in radians) according to this style
    pub fn format_angular(&self, radians: f64) -> String {
        let degrees = radians.to_degrees();

        match self.angular_unit_format {
            AngularUnitFormat::DecimalDegrees => {
                format!("{}°", self.format_decimal_with_precision(degrees, self.angular_precision))
            }
            AngularUnitFormat::DegMinSec => {
                self.format_deg_min_sec(degrees)
            }
            AngularUnitFormat::Gradians => {
                let gradians = degrees * 10.0 / 9.0;
                format!("{}g", self.format_decimal_with_precision(gradians, self.angular_precision))
            }
            AngularUnitFormat::Radians => {
                format!("{}r", self.format_decimal_with_precision(radians, self.angular_precision))
            }
            AngularUnitFormat::Surveyors => {
                self.format_surveyors(degrees)
            }
        }
    }

    fn format_decimal(&self, value: f64) -> String {
        self.format_decimal_with_precision(value, self.precision)
    }

    fn format_decimal_with_precision(&self, value: f64, precision: u8) -> String {
        let mut formatted = format!("{:.prec$}", value, prec = precision as usize);

        if self.suppress_leading_zeros && formatted.starts_with("0.") {
            formatted = formatted.trim_start_matches('0').to_string();
        }

        if self.suppress_trailing_zeros {
            formatted = formatted.trim_end_matches('0').trim_end_matches('.').to_string();
        }

        formatted
    }

    fn format_engineering(&self, value: f64) -> String {
        // Engineering: feet and decimal inches
        let feet = (value / 12.0).floor();
        let inches = value - (feet * 12.0);
        format!("{:.0}'-{:.prec$}\"", feet, inches, prec = self.precision as usize)
    }

    fn format_architectural(&self, value: f64) -> String {
        // Architectural: feet and fractional inches
        let feet = (value / 12.0).floor();
        let inches = value - (feet * 12.0);
        let fraction = self.decimal_to_fraction(inches.fract());
        let whole_inches = inches.floor();

        if feet > 0.0 {
            if whole_inches > 0.0 {
                format!("{:.0}'-{:.0} {}", feet, whole_inches, fraction)
            } else if !fraction.is_empty() {
                format!("{:.0}'-{}", feet, fraction)
            } else {
                format!("{:.0}'", feet)
            }
        } else {
            if whole_inches > 0.0 {
                format!("{:.0} {}", whole_inches, fraction)
            } else {
                fraction
            }
        }
    }

    fn format_fractional(&self, value: f64) -> String {
        let whole = value.floor();
        let fraction = self.decimal_to_fraction(value.fract());

        if whole > 0.0 {
            if !fraction.is_empty() {
                format!("{:.0} {}", whole, fraction)
            } else {
                format!("{:.0}", whole)
            }
        } else {
            fraction
        }
    }

    fn decimal_to_fraction(&self, decimal: f64) -> String {
        if decimal < 0.001 {
            return String::new();
        }

        // Common fractions based on precision
        let denominators = match self.precision {
            0 => vec![1],
            1 => vec![2, 4, 8],
            2 => vec![2, 4, 8, 16],
            3 => vec![2, 4, 8, 16, 32],
            _ => vec![2, 4, 8, 16, 32, 64],
        };

        let mut best_num = 0;
        let mut best_den = 1;
        let mut best_error = decimal;

        for &den in &denominators {
            let num = (decimal * den as f64).round() as i32;
            let error = (decimal - num as f64 / den as f64).abs();
            if error < best_error {
                best_error = error;
                best_num = num;
                best_den = den;
            }
        }

        if best_num == 0 {
            String::new()
        } else if best_num == best_den {
            "1".to_string()
        } else {
            // Simplify fraction
            let gcd = Self::gcd(best_num.abs(), best_den);
            format!("{}/{}", best_num / gcd, best_den / gcd)
        }
    }

    fn gcd(mut a: i32, mut b: i32) -> i32 {
        while b != 0 {
            let t = b;
            b = a % b;
            a = t;
        }
        a
    }

    fn format_deg_min_sec(&self, degrees: f64) -> String {
        let deg = degrees.floor();
        let min_decimal = (degrees - deg) * 60.0;
        let min = min_decimal.floor();
        let sec = (min_decimal - min) * 60.0;
        format!("{}°{}'{}\"", deg, min, sec.round())
    }

    fn format_surveyors(&self, degrees: f64) -> String {
        // Surveyor's units: N/S direction, angle, E/W direction
        // Simplified implementation
        format!("N {}° E", self.format_decimal_with_precision(degrees, self.angular_precision))
    }

    fn add_tolerance(&self, base: String, value: f64) -> String {
        match self.tolerance_format {
            ToleranceFormat::None => base,
            ToleranceFormat::Symmetrical => {
                let tol = self.format_decimal_with_precision(self.tolerance_upper, self.tolerance_precision);
                format!("{} ±{}", base, tol)
            }
            ToleranceFormat::Deviation => {
                let upper = self.format_decimal_with_precision(self.tolerance_upper, self.tolerance_precision);
                let lower = self.format_decimal_with_precision(self.tolerance_lower, self.tolerance_precision);
                format!("{} +{} -{}", base, upper, lower)
            }
            ToleranceFormat::Limits => {
                let max = value + self.tolerance_upper;
                let min = value - self.tolerance_lower;
                let max_str = self.format_decimal_with_precision(max, self.tolerance_precision);
                let min_str = self.format_decimal_with_precision(min, self.tolerance_precision);
                format!("{}/{}", max_str, min_str)
            }
            ToleranceFormat::Basic => {
                format!("[{}]", base)
            }
        }
    }
}

impl Default for DimensionStyle {
    fn default() -> Self {
        DimensionStyle::iso()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_decimal() {
        let style = DimensionStyle::iso();
        assert_eq!(style.format_linear(10.5), "10.5");
        assert_eq!(style.format_linear(10.0), "10");
    }

    #[test]
    fn test_format_angular() {
        let style = DimensionStyle::iso();
        let radians = std::f64::consts::PI / 4.0; // 45 degrees
        assert_eq!(style.format_angular(radians), "45°");
    }

    #[test]
    fn test_tolerance_symmetrical() {
        let mut style = DimensionStyle::iso();
        style.tolerance_format = ToleranceFormat::Symmetrical;
        style.tolerance_upper = 0.1;
        assert_eq!(style.format_linear(10.0), "10 ±0.1");
    }

    #[test]
    fn test_architectural_format() {
        let mut style = DimensionStyle::architectural();
        style.suppress_trailing_zeros = false;
        let result = style.format_linear(12.5); // 1 foot 0.5 inches
        assert!(result.contains('\''));
    }
}
