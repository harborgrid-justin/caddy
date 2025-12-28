// CADDY - Enterprise CAD System
// File I/O System - Unit Handling Module
// Agent 6 - File I/O System Developer

use serde::{Deserialize, Serialize};
use std::fmt;

/// Supported measurement units in CADDY
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Unit {
    /// Imperial units - inches
    Inches,
    /// Imperial units - feet
    Feet,
    /// Metric units - millimeters
    Millimeters,
    /// Metric units - centimeters
    Centimeters,
    /// Metric units - meters
    Meters,
    /// Decimal units (unitless)
    Decimal,
    /// Engineering units
    Engineering,
    /// Architectural units
    Architectural,
    /// Fractional inches
    Fractional,
}

impl Unit {
    /// Returns the conversion factor to convert from this unit to meters
    pub fn to_meters(&self) -> f64 {
        match self {
            Unit::Inches => 0.0254,
            Unit::Feet => 0.3048,
            Unit::Millimeters => 0.001,
            Unit::Centimeters => 0.01,
            Unit::Meters => 1.0,
            Unit::Decimal => 1.0,
            Unit::Engineering => 0.3048, // Same as feet
            Unit::Architectural => 0.3048, // Same as feet
            Unit::Fractional => 0.0254, // Same as inches
        }
    }

    /// Returns the conversion factor to convert from meters to this unit
    pub fn from_meters(&self) -> f64 {
        1.0 / self.to_meters()
    }

    /// Convert a value from this unit to another unit
    pub fn convert_to(&self, value: f64, target: Unit) -> f64 {
        let in_meters = value * self.to_meters();
        in_meters * target.from_meters()
    }

    /// Get the default precision (decimal places) for this unit
    pub fn default_precision(&self) -> usize {
        match self {
            Unit::Inches => 4,
            Unit::Feet => 4,
            Unit::Millimeters => 2,
            Unit::Centimeters => 3,
            Unit::Meters => 4,
            Unit::Decimal => 6,
            Unit::Engineering => 4,
            Unit::Architectural => 4,
            Unit::Fractional => 4,
        }
    }

    /// Get the unit abbreviation
    pub fn abbreviation(&self) -> &'static str {
        match self {
            Unit::Inches => "in",
            Unit::Feet => "ft",
            Unit::Millimeters => "mm",
            Unit::Centimeters => "cm",
            Unit::Meters => "m",
            Unit::Decimal => "",
            Unit::Engineering => "ft",
            Unit::Architectural => "ft",
            Unit::Fractional => "in",
        }
    }

    /// Get the full name of the unit
    pub fn full_name(&self) -> &'static str {
        match self {
            Unit::Inches => "Inches",
            Unit::Feet => "Feet",
            Unit::Millimeters => "Millimeters",
            Unit::Centimeters => "Centimeters",
            Unit::Meters => "Meters",
            Unit::Decimal => "Decimal",
            Unit::Engineering => "Engineering",
            Unit::Architectural => "Architectural",
            Unit::Fractional => "Fractional",
        }
    }

    /// Parse a unit from DXF unit code
    pub fn from_dxf_code(code: i32) -> Option<Self> {
        match code {
            0 => Some(Unit::Decimal),
            1 => Some(Unit::Inches),
            2 => Some(Unit::Feet),
            3 => Some(Unit::Feet), // Miles -> Feet
            4 => Some(Unit::Millimeters),
            5 => Some(Unit::Centimeters),
            6 => Some(Unit::Meters),
            7 => Some(Unit::Meters), // Kilometers -> Meters
            8 => Some(Unit::Inches), // Microinches -> Inches
            9 => Some(Unit::Millimeters), // Mils -> Millimeters
            10 => Some(Unit::Meters), // Yards -> Meters (approximation)
            11 => Some(Unit::Meters), // Angstroms -> Meters
            12 => Some(Unit::Meters), // Nanometers -> Meters
            13 => Some(Unit::Meters), // Microns -> Meters
            14 => Some(Unit::Decimal), // Decimeters -> Decimal
            _ => None,
        }
    }

    /// Convert to DXF unit code
    pub fn to_dxf_code(&self) -> i32 {
        match self {
            Unit::Decimal => 0,
            Unit::Inches | Unit::Fractional => 1,
            Unit::Feet | Unit::Engineering | Unit::Architectural => 2,
            Unit::Millimeters => 4,
            Unit::Centimeters => 5,
            Unit::Meters => 6,
        }
    }

    /// Format a value with this unit
    pub fn format(&self, value: f64, precision: Option<usize>) -> String {
        let prec = precision.unwrap_or_else(|| self.default_precision());

        match self {
            Unit::Fractional => {
                // Convert to fractional inches (e.g., 1-1/2")
                format_fractional(value, 16) // 1/16" precision
            }
            Unit::Architectural => {
                // Convert to architectural format (e.g., 1'-6")
                format_architectural(value)
            }
            Unit::Engineering => {
                // Engineering format (decimal feet with inches, e.g., 1.5')
                format!("{:.prec$}{}", value, self.abbreviation(), prec = prec)
            }
            _ => {
                if self.abbreviation().is_empty() {
                    format!("{:.prec$}", value, prec = prec)
                } else {
                    format!("{:.prec$} {}", value, self.abbreviation(), prec = prec)
                }
            }
        }
    }

    /// All available units
    pub fn all() -> &'static [Unit] {
        &[
            Unit::Inches,
            Unit::Feet,
            Unit::Millimeters,
            Unit::Centimeters,
            Unit::Meters,
            Unit::Decimal,
            Unit::Engineering,
            Unit::Architectural,
            Unit::Fractional,
        ]
    }
}

impl Default for Unit {
    fn default() -> Self {
        Unit::Millimeters
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.full_name())
    }
}

/// Format a value as fractional inches
fn format_fractional(value: f64, denominator: i32) -> String {
    let whole = value.floor() as i32;
    let fraction = value - whole as f64;

    if fraction < 0.001 {
        format!("{}\"", whole)
    } else {
        let numerator = (fraction * denominator as f64).round() as i32;

        // Simplify fraction
        let gcd = gcd(numerator, denominator);
        let num = numerator / gcd;
        let den = denominator / gcd;

        if whole == 0 {
            format!("{}/{}\"", num, den)
        } else {
            format!("{}-{}/{}\"", whole, num, den)
        }
    }
}

/// Format a value as architectural (feet and inches)
fn format_architectural(value_in_feet: f64) -> String {
    let feet = value_in_feet.floor() as i32;
    let inches = (value_in_feet - feet as f64) * 12.0;

    if inches < 0.001 {
        format!("{}'", feet)
    } else {
        let whole_inches = inches.floor() as i32;
        let fraction = inches - whole_inches as f64;

        if fraction < 0.001 {
            format!("{}'{}\"", feet, whole_inches)
        } else {
            let numerator = (fraction * 16.0).round() as i32;
            let gcd = gcd(numerator, 16);
            let num = numerator / gcd;
            let den = 16 / gcd;

            if whole_inches == 0 {
                format!("{}'{}/{}\"", feet, num, den)
            } else {
                format!("{}'{}^{}/{}\"", feet, whole_inches, num, den)
            }
        }
    }
}

/// Greatest common divisor
fn gcd(mut a: i32, mut b: i32) -> i32 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a.abs()
}

/// Unit conversion utility
#[derive(Debug, Clone)]
pub struct UnitConverter {
    from: Unit,
    to: Unit,
}

impl UnitConverter {
    /// Create a new unit converter
    pub fn new(from: Unit, to: Unit) -> Self {
        Self { from, to }
    }

    /// Convert a value
    pub fn convert(&self, value: f64) -> f64 {
        self.from.convert_to(value, self.to)
    }

    /// Convert multiple values
    pub fn convert_vec(&self, values: &[f64]) -> Vec<f64> {
        values.iter().map(|&v| self.convert(v)).collect()
    }
}

/// Precision settings for display
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PrecisionSettings {
    /// Number of decimal places to display
    pub decimal_places: usize,
    /// Whether to suppress trailing zeros
    pub suppress_trailing_zeros: bool,
    /// Whether to suppress leading zeros
    pub suppress_leading_zeros: bool,
    /// Tolerance for geometric operations
    pub geometric_tolerance: f64,
}

impl PrecisionSettings {
    /// Create precision settings for a specific unit
    pub fn for_unit(unit: Unit) -> Self {
        Self {
            decimal_places: unit.default_precision(),
            suppress_trailing_zeros: true,
            suppress_leading_zeros: false,
            geometric_tolerance: match unit {
                Unit::Millimeters => 0.01,
                Unit::Centimeters => 0.001,
                Unit::Meters => 0.0001,
                Unit::Inches => 0.0001,
                Unit::Feet => 0.0001,
                _ => 0.000001,
            },
        }
    }

    /// Format a number according to these precision settings
    pub fn format(&self, value: f64) -> String {
        let mut formatted = format!("{:.prec$}", value, prec = self.decimal_places);

        if self.suppress_trailing_zeros {
            formatted = formatted.trim_end_matches('0').trim_end_matches('.').to_string();
        }

        if self.suppress_leading_zeros && formatted.starts_with("0.") {
            formatted = formatted.trim_start_matches('0').to_string();
        }

        formatted
    }
}

impl Default for PrecisionSettings {
    fn default() -> Self {
        Self::for_unit(Unit::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_conversion() {
        let inches = Unit::Inches;
        let mm = Unit::Millimeters;

        let value_in_inches = 1.0;
        let value_in_mm = inches.convert_to(value_in_inches, mm);

        assert!((value_in_mm - 25.4).abs() < 0.001);
    }

    #[test]
    fn test_fractional_format() {
        let result = format_fractional(1.5, 16);
        assert!(result.contains("1-8/16") || result.contains("1-1/2"));
    }

    #[test]
    fn test_architectural_format() {
        let result = format_architectural(1.5); // 1.5 feet = 1' 6"
        assert!(result.contains("1'"));
        assert!(result.contains("6"));
    }

    #[test]
    fn test_precision_settings() {
        let prec = PrecisionSettings {
            decimal_places: 2,
            suppress_trailing_zeros: true,
            suppress_leading_zeros: false,
            geometric_tolerance: 0.01,
        };

        assert_eq!(prec.format(1.50), "1.5");
        assert_eq!(prec.format(1.00), "1");
    }
}
