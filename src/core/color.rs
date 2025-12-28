//! Color module for CAD operations
//!
//! Provides comprehensive color representation with:
//! - RGBA color type with 8-bit channels
//! - AutoCAD Color Index (ACI) support
//! - HSV/HSL color space conversions
//! - Standard CAD color constants
//! - Serialization support

use serde::{Deserialize, Serialize};
use std::fmt;

/// RGBA color representation with 8-bit channels
///
/// Thread-safe, serializable color type for CAD entities.
/// Implements Copy for efficient passing and storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Color {
    /// Red channel (0-255)
    pub r: u8,
    /// Green channel (0-255)
    pub g: u8,
    /// Blue channel (0-255)
    pub b: u8,
    /// Alpha channel (0-255, 255 = opaque)
    pub a: u8,
}

impl Color {
    /// Create a new color from RGBA values
    #[inline]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create a new opaque color from RGB values
    #[inline]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }

    /// Create a grayscale color
    #[inline]
    pub const fn gray(value: u8) -> Self {
        Self::rgb(value, value, value)
    }

    /// Create a transparent color
    #[inline]
    pub const fn transparent() -> Self {
        Self::new(0, 0, 0, 0)
    }

    /// Create a color from AutoCAD Color Index (ACI)
    ///
    /// ACI is a standard color indexing system used in AutoCAD.
    /// Range: 0-255, where:
    /// - 0 = ByBlock
    /// - 1-9 = Standard colors
    /// - 10-249 = Extended color palette
    /// - 250-255 = Grayscale
    pub fn from_aci(aci: u8) -> Self {
        match aci {
            0 => Self::BY_BLOCK,
            1 => Self::RED,
            2 => Self::YELLOW,
            3 => Self::GREEN,
            4 => Self::CYAN,
            5 => Self::BLUE,
            6 => Self::MAGENTA,
            7 => Self::WHITE,
            8 => Self::DARK_GRAY,
            9 => Self::LIGHT_GRAY,
            10..=249 => {
                // Standard AutoCAD color table
                // This is a simplified mapping - full ACI table has 255 colors
                let hue = ((aci - 10) % 10) as f32 * 36.0;
                let sat = if aci < 130 { 1.0 } else { 0.5 };
                let val = if aci < 210 { 1.0 } else { 0.5 };
                Self::from_hsv(hue, sat, val)
            }
            250 => Self::BLACK,
            251..=255 => {
                let gray = ((aci - 250) * 51) as u8;
                Self::rgb(gray, gray, gray)
            }
        }
    }

    /// Convert HSV to RGB color
    ///
    /// # Arguments
    /// - `h`: Hue in degrees (0-360)
    /// - `s`: Saturation (0.0-1.0)
    /// - `v`: Value/Brightness (0.0-1.0)
    pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r, g, b) = match h as u32 {
            0..=59 => (c, x, 0.0),
            60..=119 => (x, c, 0.0),
            120..=179 => (0.0, c, x),
            180..=239 => (0.0, x, c),
            240..=299 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Self::rgb(
            ((r + m) * 255.0) as u8,
            ((g + m) * 255.0) as u8,
            ((b + m) * 255.0) as u8,
        )
    }

    /// Convert HSL to RGB color
    ///
    /// # Arguments
    /// - `h`: Hue in degrees (0-360)
    /// - `s`: Saturation (0.0-1.0)
    /// - `l`: Lightness (0.0-1.0)
    pub fn from_hsl(h: f32, s: f32, l: f32) -> Self {
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        let (r, g, b) = match h as u32 {
            0..=59 => (c, x, 0.0),
            60..=119 => (x, c, 0.0),
            120..=179 => (0.0, c, x),
            180..=239 => (0.0, x, c),
            240..=299 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Self::rgb(
            ((r + m) * 255.0) as u8,
            ((g + m) * 255.0) as u8,
            ((b + m) * 255.0) as u8,
        )
    }

    /// Convert color to HSV
    ///
    /// Returns (hue, saturation, value) where:
    /// - hue: 0-360 degrees
    /// - saturation: 0.0-1.0
    /// - value: 0.0-1.0
    pub fn to_hsv(&self) -> (f32, f32, f32) {
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let hue = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };

        let saturation = if max == 0.0 { 0.0 } else { delta / max };

        (if hue < 0.0 { hue + 360.0 } else { hue }, saturation, max)
    }

    /// Convert color to AutoCAD Color Index (ACI) - best match
    pub fn to_aci(&self) -> u8 {
        // Find closest ACI color using simple matching
        if *self == Self::BY_BLOCK {
            return 0;
        }
        if *self == Self::RED {
            return 1;
        }
        if *self == Self::YELLOW {
            return 2;
        }
        if *self == Self::GREEN {
            return 3;
        }
        if *self == Self::CYAN {
            return 4;
        }
        if *self == Self::BLUE {
            return 5;
        }
        if *self == Self::MAGENTA {
            return 6;
        }
        if *self == Self::WHITE {
            return 7;
        }
        if *self == Self::BLACK {
            return 250;
        }

        // Default to white for unknown colors
        7
    }

    /// Convert to floating point RGBA [0.0, 1.0]
    #[inline]
    pub fn to_f32_array(&self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }

    /// Convert to floating point RGB [0.0, 1.0] (ignoring alpha)
    #[inline]
    pub fn to_f32_rgb(&self) -> [f32; 3] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        ]
    }

    /// Create from floating point RGBA [0.0, 1.0]
    pub fn from_f32_array(rgba: [f32; 4]) -> Self {
        Self::new(
            (rgba[0].clamp(0.0, 1.0) * 255.0) as u8,
            (rgba[1].clamp(0.0, 1.0) * 255.0) as u8,
            (rgba[2].clamp(0.0, 1.0) * 255.0) as u8,
            (rgba[3].clamp(0.0, 1.0) * 255.0) as u8,
        )
    }

    /// Create from floating point RGB [0.0, 1.0] with full opacity
    pub fn from_f32_rgb(rgb: [f32; 3]) -> Self {
        Self::rgb(
            (rgb[0].clamp(0.0, 1.0) * 255.0) as u8,
            (rgb[1].clamp(0.0, 1.0) * 255.0) as u8,
            (rgb[2].clamp(0.0, 1.0) * 255.0) as u8,
        )
    }

    /// Convert to 32-bit RGBA integer (0xRRGGBBAA)
    #[inline]
    pub fn to_rgba_u32(&self) -> u32 {
        ((self.r as u32) << 24) | ((self.g as u32) << 16) | ((self.b as u32) << 8) | (self.a as u32)
    }

    /// Create from 32-bit RGBA integer (0xRRGGBBAA)
    #[inline]
    pub fn from_rgba_u32(rgba: u32) -> Self {
        Self::new(
            ((rgba >> 24) & 0xFF) as u8,
            ((rgba >> 16) & 0xFF) as u8,
            ((rgba >> 8) & 0xFF) as u8,
            (rgba & 0xFF) as u8,
        )
    }

    /// Convert to hex string (#RRGGBB or #RRGGBBAA)
    pub fn to_hex(&self, include_alpha: bool) -> String {
        if include_alpha {
            format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        } else {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        }
    }

    /// Create from hex string (#RGB, #RRGGBB, or #RRGGBBAA)
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        match hex.len() {
            3 => {
                // #RGB -> #RRGGBB
                let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
                Some(Self::rgb(r, g, b))
            }
            6 => {
                // #RRGGBB
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self::rgb(r, g, b))
            }
            8 => {
                // #RRGGBBAA
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self::new(r, g, b, a))
            }
            _ => None,
        }
    }

    /// Linear interpolation between two colors
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self::new(
            (self.r as f32 + (other.r as f32 - self.r as f32) * t) as u8,
            (self.g as f32 + (other.g as f32 - self.g as f32) * t) as u8,
            (self.b as f32 + (other.b as f32 - self.b as f32) * t) as u8,
            (self.a as f32 + (other.a as f32 - self.a as f32) * t) as u8,
        )
    }

    /// Create a new color with different alpha
    #[inline]
    pub fn with_alpha(&self, alpha: u8) -> Self {
        Self::new(self.r, self.g, self.b, alpha)
    }

    /// Get luminance (perceived brightness) using ITU-R BT.709
    #[inline]
    pub fn luminance(&self) -> f32 {
        0.2126 * (self.r as f32 / 255.0)
            + 0.7152 * (self.g as f32 / 255.0)
            + 0.0722 * (self.b as f32 / 255.0)
    }

    /// Check if color is dark (luminance < 0.5)
    #[inline]
    pub fn is_dark(&self) -> bool {
        self.luminance() < 0.5
    }

    /// Check if color is light (luminance >= 0.5)
    #[inline]
    pub fn is_light(&self) -> bool {
        !self.is_dark()
    }

    // ========================================================================
    // Standard AutoCAD Colors
    // ========================================================================

    /// ByBlock color (special value)
    pub const BY_BLOCK: Self = Self::new(0, 0, 0, 0);

    /// ByLayer color (special value)
    pub const BY_LAYER: Self = Self::new(255, 255, 255, 0);

    /// Red (ACI 1)
    pub const RED: Self = Self::rgb(255, 0, 0);

    /// Yellow (ACI 2)
    pub const YELLOW: Self = Self::rgb(255, 255, 0);

    /// Green (ACI 3)
    pub const GREEN: Self = Self::rgb(0, 255, 0);

    /// Cyan (ACI 4)
    pub const CYAN: Self = Self::rgb(0, 255, 255);

    /// Blue (ACI 5)
    pub const BLUE: Self = Self::rgb(0, 0, 255);

    /// Magenta (ACI 6)
    pub const MAGENTA: Self = Self::rgb(255, 0, 255);

    /// White (ACI 7)
    pub const WHITE: Self = Self::rgb(255, 255, 255);

    /// Black (ACI 250)
    pub const BLACK: Self = Self::rgb(0, 0, 0);

    /// Dark Gray (ACI 8)
    pub const DARK_GRAY: Self = Self::rgb(64, 64, 64);

    /// Light Gray (ACI 9)
    pub const LIGHT_GRAY: Self = Self::rgb(192, 192, 192);

    // Additional standard colors
    /// Orange
    pub const ORANGE: Self = Self::rgb(255, 165, 0);

    /// Purple
    pub const PURPLE: Self = Self::rgb(128, 0, 128);

    /// Brown
    pub const BROWN: Self = Self::rgb(165, 42, 42);

    /// Pink
    pub const PINK: Self = Self::rgb(255, 192, 203);
}

impl Default for Color {
    fn default() -> Self {
        Self::BY_LAYER
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == Self::BY_BLOCK {
            write!(f, "ByBlock")
        } else if *self == Self::BY_LAYER {
            write!(f, "ByLayer")
        } else if self.a == 255 {
            write!(f, "RGB({}, {}, {})", self.r, self.g, self.b)
        } else {
            write!(f, "RGBA({}, {}, {}, {})", self.r, self.g, self.b, self.a)
        }
    }
}

// Color is automatically Send + Sync because it only contains primitive types

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_creation() {
        let color = Color::rgb(255, 0, 0);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
        assert_eq!(color.a, 255);
    }

    #[test]
    fn test_aci_conversion() {
        assert_eq!(Color::from_aci(1), Color::RED);
        assert_eq!(Color::from_aci(3), Color::GREEN);
        assert_eq!(Color::from_aci(5), Color::BLUE);
    }

    #[test]
    fn test_f32_conversion() {
        let color = Color::rgb(128, 64, 192);
        let f32_array = color.to_f32_array();
        let converted = Color::from_f32_array(f32_array);
        assert_eq!(color, converted);
    }

    #[test]
    fn test_hex_conversion() {
        let color = Color::rgb(255, 128, 64);
        let hex = color.to_hex(false);
        assert_eq!(hex, "#FF8040");

        let parsed = Color::from_hex("#FF8040").unwrap();
        assert_eq!(parsed, color);

        // Test short format
        let short = Color::from_hex("#F80").unwrap();
        assert_eq!(short, Color::rgb(255, 136, 0));
    }

    #[test]
    fn test_hsv_conversion() {
        // Red
        let red = Color::RED;
        let (h, s, v) = red.to_hsv();
        assert!((h - 0.0).abs() < 1.0);
        assert!((s - 1.0).abs() < 0.01);
        assert!((v - 1.0).abs() < 0.01);

        // Create from HSV
        let color = Color::from_hsv(120.0, 1.0, 1.0);
        assert_eq!(color, Color::GREEN);
    }

    #[test]
    fn test_hsl_conversion() {
        let color = Color::from_hsl(240.0, 1.0, 0.5);
        assert_eq!(color, Color::BLUE);
    }

    #[test]
    fn test_lerp() {
        let black = Color::BLACK;
        let white = Color::WHITE;
        let gray = black.lerp(&white, 0.5);
        assert_eq!(gray.r, 127);
        assert_eq!(gray.g, 127);
        assert_eq!(gray.b, 127);
    }

    #[test]
    fn test_luminance() {
        assert!(Color::WHITE.is_light());
        assert!(Color::BLACK.is_dark());
        assert!(Color::YELLOW.is_light());
        assert!(Color::BLUE.is_dark());
    }

    #[test]
    fn test_rgba_u32() {
        let color = Color::new(0xAB, 0xCD, 0xEF, 0x12);
        let u32_val = color.to_rgba_u32();
        let converted = Color::from_rgba_u32(u32_val);
        assert_eq!(color, converted);
    }

    #[test]
    fn test_with_alpha() {
        let red = Color::RED;
        let semi_transparent = red.with_alpha(128);
        assert_eq!(semi_transparent.r, 255);
        assert_eq!(semi_transparent.a, 128);
    }
}
