//! Angular dimension types and calculations
//!
//! Provides angular dimensions between lines, 3-point angles, and arc length dimensions.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::style::DimensionStyle;
use super::linear::Point3D;

/// Angular dimension between two lines
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AngularDimension {
    /// Unique identifier
    pub id: Uuid,
    /// Center point (vertex of angle)
    pub center_point: Point3D,
    /// First line point (defines first ray)
    pub line1_point: Point3D,
    /// Second line point (defines second ray)
    pub line2_point: Point3D,
    /// Arc point (where dimension arc is located)
    pub arc_point: Point3D,
    /// Dimension style reference
    pub style_name: String,
    /// Optional text override
    pub text_override: Option<String>,
    /// Layer name
    pub layer: String,
    /// Is this dimension associative
    pub associative: bool,
    /// Associated entity IDs
    pub associated_entities: Vec<Uuid>,
}

impl AngularDimension {
    /// Create a new angular dimension
    pub fn new(
        center_point: Point3D,
        line1_point: Point3D,
        line2_point: Point3D,
        arc_point: Point3D,
        style_name: impl Into<String>,
    ) -> Self {
        AngularDimension {
            id: Uuid::new_v4(),
            center_point,
            line1_point,
            line2_point,
            arc_point,
            style_name: style_name.into(),
            text_override: None,
            layer: "0".to_string(),
            associative: false,
            associated_entities: Vec::new(),
        }
    }

    /// Calculate the angle in radians
    pub fn calculate_angle(&self) -> f64 {
        let dx1 = self.line1_point.x - self.center_point.x;
        let dy1 = self.line1_point.y - self.center_point.y;
        let angle1 = dy1.atan2(dx1);

        let dx2 = self.line2_point.x - self.center_point.x;
        let dy2 = self.line2_point.y - self.center_point.y;
        let angle2 = dy2.atan2(dx2);

        let mut angle = angle2 - angle1;

        // Normalize to 0-2π
        while angle < 0.0 {
            angle += 2.0 * std::f64::consts::PI;
        }
        while angle > 2.0 * std::f64::consts::PI {
            angle -= 2.0 * std::f64::consts::PI;
        }

        // Determine if we should measure the smaller or larger angle
        // based on which side of the angle the arc_point is on
        let mid_angle = (angle1 + angle2) / 2.0;
        let arc_dx = self.arc_point.x - self.center_point.x;
        let arc_dy = self.arc_point.y - self.center_point.y;
        let arc_angle = arc_dy.atan2(arc_dx);

        let diff1 = (arc_angle - angle1).abs();
        let diff2 = (arc_angle - angle2).abs();
        let diff_mid = (arc_angle - mid_angle).abs();

        if diff_mid < diff1.min(diff2) {
            angle
        } else {
            2.0 * std::f64::consts::PI - angle
        }
    }

    /// Get dimension text
    pub fn get_text(&self, style: &DimensionStyle) -> String {
        if let Some(ref override_text) = self.text_override {
            override_text.clone()
        } else {
            let angle = self.calculate_angle();
            style.format_angular(angle)
        }
    }

    /// Get the arc radius for dimension arc
    pub fn get_arc_radius(&self) -> f64 {
        let dx = self.arc_point.x - self.center_point.x;
        let dy = self.arc_point.y - self.center_point.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Get arc start and end angles
    pub fn get_arc_angles(&self) -> (f64, f64) {
        let dx1 = self.line1_point.x - self.center_point.x;
        let dy1 = self.line1_point.y - self.center_point.y;
        let angle1 = dy1.atan2(dx1);

        let dx2 = self.line2_point.x - self.center_point.x;
        let dy2 = self.line2_point.y - self.center_point.y;
        let angle2 = dy2.atan2(dx2);

        (angle1, angle2)
    }

    /// Set text override
    pub fn set_text_override(&mut self, text: Option<String>) {
        self.text_override = text;
    }

    /// Make dimension associative
    pub fn associate_with(&mut self, entity_ids: Vec<Uuid>) {
        self.associative = true;
        self.associated_entities = entity_ids;
    }
}

/// Angular dimension defined by 3 points
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Angular3PointDimension {
    /// Unique identifier
    pub id: Uuid,
    /// Center point (vertex)
    pub center_point: Point3D,
    /// First point
    pub point1: Point3D,
    /// Second point
    pub point2: Point3D,
    /// Arc location point
    pub arc_point: Point3D,
    /// Dimension style reference
    pub style_name: String,
    /// Optional text override
    pub text_override: Option<String>,
    /// Layer name
    pub layer: String,
}

impl Angular3PointDimension {
    /// Create a new 3-point angular dimension
    pub fn new(
        center_point: Point3D,
        point1: Point3D,
        point2: Point3D,
        arc_point: Point3D,
        style_name: impl Into<String>,
    ) -> Self {
        Angular3PointDimension {
            id: Uuid::new_v4(),
            center_point,
            point1,
            point2,
            arc_point,
            style_name: style_name.into(),
            text_override: None,
            layer: "0".to_string(),
        }
    }

    /// Calculate the angle in radians
    pub fn calculate_angle(&self) -> f64 {
        // Vector from center to point1
        let v1x = self.point1.x - self.center_point.x;
        let v1y = self.point1.y - self.center_point.y;

        // Vector from center to point2
        let v2x = self.point2.x - self.center_point.x;
        let v2y = self.point2.y - self.center_point.y;

        // Calculate angle using atan2
        let angle1 = v1y.atan2(v1x);
        let angle2 = v2y.atan2(v2x);

        let mut angle = angle2 - angle1;

        // Normalize to 0-2π
        while angle < 0.0 {
            angle += 2.0 * std::f64::consts::PI;
        }
        while angle > 2.0 * std::f64::consts::PI {
            angle -= 2.0 * std::f64::consts::PI;
        }

        // Choose smaller or larger angle based on arc_point
        if angle > std::f64::consts::PI {
            2.0 * std::f64::consts::PI - angle
        } else {
            angle
        }
    }

    /// Get dimension text
    pub fn get_text(&self, style: &DimensionStyle) -> String {
        if let Some(ref override_text) = self.text_override {
            override_text.clone()
        } else {
            let angle = self.calculate_angle();
            style.format_angular(angle)
        }
    }
}

/// Arc length dimension
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArcLengthDimension {
    /// Unique identifier
    pub id: Uuid,
    /// Arc center point
    pub center_point: Point3D,
    /// Arc start point
    pub start_point: Point3D,
    /// Arc end point
    pub end_point: Point3D,
    /// Dimension arc location point
    pub dim_arc_point: Point3D,
    /// Arc symbol location (start, middle, end)
    pub symbol_location: ArcSymbolLocation,
    /// Dimension style reference
    pub style_name: String,
    /// Optional text override
    pub text_override: Option<String>,
    /// Layer name
    pub layer: String,
    /// Is this dimension associative
    pub associative: bool,
    /// Associated arc entity ID
    pub associated_arc: Option<Uuid>,
}

/// Location for arc length symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArcSymbolLocation {
    /// Before text
    BeforeText,
    /// Above text
    AboveText,
    /// No symbol
    None,
}

impl ArcLengthDimension {
    /// Create a new arc length dimension
    pub fn new(
        center_point: Point3D,
        start_point: Point3D,
        end_point: Point3D,
        dim_arc_point: Point3D,
        style_name: impl Into<String>,
    ) -> Self {
        ArcLengthDimension {
            id: Uuid::new_v4(),
            center_point,
            start_point,
            end_point,
            dim_arc_point,
            symbol_location: ArcSymbolLocation::BeforeText,
            style_name: style_name.into(),
            text_override: None,
            layer: "0".to_string(),
            associative: false,
            associated_arc: None,
        }
    }

    /// Calculate the arc length
    pub fn calculate_length(&self) -> f64 {
        let radius = self.get_radius();
        let angle = self.calculate_angle();
        radius * angle
    }

    /// Get the arc radius
    pub fn get_radius(&self) -> f64 {
        let dx = self.start_point.x - self.center_point.x;
        let dy = self.start_point.y - self.center_point.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Calculate the subtended angle
    fn calculate_angle(&self) -> f64 {
        let dx1 = self.start_point.x - self.center_point.x;
        let dy1 = self.start_point.y - self.center_point.y;
        let angle1 = dy1.atan2(dx1);

        let dx2 = self.end_point.x - self.center_point.x;
        let dy2 = self.end_point.y - self.center_point.y;
        let angle2 = dy2.atan2(dx2);

        let mut angle = angle2 - angle1;

        // Normalize to positive
        while angle < 0.0 {
            angle += 2.0 * std::f64::consts::PI;
        }
        while angle > 2.0 * std::f64::consts::PI {
            angle -= 2.0 * std::f64::consts::PI;
        }

        angle
    }

    /// Get dimension text with arc symbol
    pub fn get_text(&self, style: &DimensionStyle) -> String {
        let base_text = if let Some(ref override_text) = self.text_override {
            override_text.clone()
        } else {
            let length = self.calculate_length();
            style.format_linear(length)
        };

        match self.symbol_location {
            ArcSymbolLocation::BeforeText => format!("⌒{}", base_text),
            ArcSymbolLocation::AboveText => format!("⌒\n{}", base_text),
            ArcSymbolLocation::None => base_text,
        }
    }

    /// Set text override
    pub fn set_text_override(&mut self, text: Option<String>) {
        self.text_override = text;
    }

    /// Associate with arc entity
    pub fn associate_with_arc(&mut self, arc_id: Uuid) {
        self.associative = true;
        self.associated_arc = Some(arc_id);
    }
}

/// Quadrant angles helper
pub struct QuadrantAngles;

impl QuadrantAngles {
    /// Normalize angle to 0-360 degrees
    pub fn normalize_degrees(angle: f64) -> f64 {
        let mut result = angle % 360.0;
        if result < 0.0 {
            result += 360.0;
        }
        result
    }

    /// Normalize angle to 0-2π radians
    pub fn normalize_radians(angle: f64) -> f64 {
        let two_pi = 2.0 * std::f64::consts::PI;
        let mut result = angle % two_pi;
        if result < 0.0 {
            result += two_pi;
        }
        result
    }

    /// Get the quadrant of an angle (1-4)
    pub fn get_quadrant(angle: f64) -> u8 {
        let normalized = Self::normalize_degrees(angle.to_degrees());
        if normalized < 90.0 {
            1
        } else if normalized < 180.0 {
            2
        } else if normalized < 270.0 {
            3
        } else {
            4
        }
    }

    /// Calculate the angle between two vectors
    pub fn angle_between(v1x: f64, v1y: f64, v2x: f64, v2y: f64) -> f64 {
        let dot = v1x * v2x + v1y * v2y;
        let cross = v1x * v2y - v1y * v2x;
        cross.atan2(dot)
    }

    /// Calculate the smallest angle between two angles
    pub fn smallest_difference(angle1: f64, angle2: f64) -> f64 {
        let diff = (angle2 - angle1).abs();
        if diff > std::f64::consts::PI {
            2.0 * std::f64::consts::PI - diff
        } else {
            diff
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_angular_dimension_90_degrees() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let p1 = Point3D::new(10.0, 0.0, 0.0);
        let p2 = Point3D::new(0.0, 10.0, 0.0);
        let arc = Point3D::new(7.0, 7.0, 0.0);

        let dim = AngularDimension::new(center, p1, p2, arc, "ISO-25");
        let angle = dim.calculate_angle();

        // Should be approximately π/2 (90 degrees)
        assert!((angle - std::f64::consts::PI / 2.0).abs() < 0.01);
    }

    #[test]
    fn test_angular_dimension_45_degrees() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let p1 = Point3D::new(10.0, 0.0, 0.0);
        let p2 = Point3D::new(10.0, 10.0, 0.0);
        let arc = Point3D::new(8.0, 4.0, 0.0);

        let dim = AngularDimension::new(center, p1, p2, arc, "ISO-25");
        let angle = dim.calculate_angle();

        // Should be approximately π/4 (45 degrees)
        assert!((angle - std::f64::consts::PI / 4.0).abs() < 0.01);
    }

    #[test]
    fn test_arc_length_dimension() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let start = Point3D::new(10.0, 0.0, 0.0);
        let end = Point3D::new(0.0, 10.0, 0.0);
        let arc = Point3D::new(7.0, 7.0, 0.0);

        let dim = ArcLengthDimension::new(center, start, end, arc, "ISO-25");
        let length = dim.calculate_length();

        // Arc length = radius * angle
        // radius = 10, angle = π/2
        // length ≈ 15.708
        assert!((length - 10.0 * std::f64::consts::PI / 2.0).abs() < 0.01);
    }

    #[test]
    fn test_3point_angular_dimension() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let p1 = Point3D::new(10.0, 0.0, 0.0);
        let p2 = Point3D::new(0.0, 10.0, 0.0);
        let arc = Point3D::new(7.0, 7.0, 0.0);

        let dim = Angular3PointDimension::new(center, p1, p2, arc, "ISO-25");
        let angle = dim.calculate_angle();

        // Should be approximately π/2 (90 degrees)
        assert!((angle - std::f64::consts::PI / 2.0).abs() < 0.01);
    }

    #[test]
    fn test_quadrant_angles() {
        assert_eq!(QuadrantAngles::get_quadrant(45.0), 1);
        assert_eq!(QuadrantAngles::get_quadrant(135.0), 2);
        assert_eq!(QuadrantAngles::get_quadrant(225.0), 3);
        assert_eq!(QuadrantAngles::get_quadrant(315.0), 4);
    }

    #[test]
    fn test_normalize_angle() {
        assert!((QuadrantAngles::normalize_degrees(370.0) - 10.0).abs() < 0.01);
        assert!((QuadrantAngles::normalize_degrees(-45.0) - 315.0).abs() < 0.01);
    }
}
