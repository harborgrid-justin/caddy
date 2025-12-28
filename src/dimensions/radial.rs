//! Radial dimension types (radius and diameter)
//!
//! Provides radius, diameter, and jogged radius dimensions for circles and arcs.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::style::DimensionStyle;
use super::linear::Point3D;

/// Radius dimension for circles and arcs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RadiusDimension {
    /// Unique identifier
    pub id: Uuid,
    /// Center point of circle/arc
    pub center_point: Point3D,
    /// Point on the circle/arc perimeter
    pub chord_point: Point3D,
    /// Leader endpoint (text location)
    pub leader_endpoint: Point3D,
    /// Dimension style reference
    pub style_name: String,
    /// Optional text override
    pub text_override: Option<String>,
    /// Layer name
    pub layer: String,
    /// Is this dimension associative
    pub associative: bool,
    /// Associated circle/arc entity ID
    pub associated_entity: Option<Uuid>,
}

impl RadiusDimension {
    /// Create a new radius dimension
    pub fn new(
        center_point: Point3D,
        chord_point: Point3D,
        leader_endpoint: Point3D,
        style_name: impl Into<String>,
    ) -> Self {
        RadiusDimension {
            id: Uuid::new_v4(),
            center_point,
            chord_point,
            leader_endpoint,
            style_name: style_name.into(),
            text_override: None,
            layer: "0".to_string(),
            associative: false,
            associated_entity: None,
        }
    }

    /// Create radius dimension from circle center and radius
    pub fn from_circle(
        center: Point3D,
        radius: f64,
        angle: f64,
        style_name: impl Into<String>,
    ) -> Self {
        let chord_point = Point3D::new(
            center.x + radius * angle.cos(),
            center.y + radius * angle.sin(),
            center.z,
        );
        let leader_endpoint = Point3D::new(
            center.x + (radius * 1.5) * angle.cos(),
            center.y + (radius * 1.5) * angle.sin(),
            center.z,
        );

        RadiusDimension::new(center, chord_point, leader_endpoint, style_name)
    }

    /// Calculate the radius
    pub fn calculate_radius(&self) -> f64 {
        let dx = self.chord_point.x - self.center_point.x;
        let dy = self.chord_point.y - self.center_point.y;
        let dz = self.chord_point.z - self.center_point.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Get dimension text with radius prefix
    pub fn get_text(&self, style: &DimensionStyle) -> String {
        if let Some(ref override_text) = self.text_override {
            override_text.clone()
        } else {
            let radius = self.calculate_radius();
            format!("R{}", style.format_linear(radius))
        }
    }

    /// Set text override
    pub fn set_text_override(&mut self, text: Option<String>) {
        self.text_override = text;
    }

    /// Associate with circle or arc entity
    pub fn associate_with(&mut self, entity_id: Uuid) {
        self.associative = true;
        self.associated_entity = Some(entity_id);
    }

    /// Update dimension when associated geometry changes
    pub fn update_from_circle(&mut self, center: Point3D, radius: f64) {
        if self.associative {
            self.center_point = center;
            // Maintain the same angle
            let current_angle = self.get_leader_angle();
            self.chord_point = Point3D::new(
                center.x + radius * current_angle.cos(),
                center.y + radius * current_angle.sin(),
                center.z,
            );
        }
    }

    /// Get the angle of the leader line
    fn get_leader_angle(&self) -> f64 {
        let dx = self.chord_point.x - self.center_point.x;
        let dy = self.chord_point.y - self.center_point.y;
        dy.atan2(dx)
    }

    /// Check if leader should show center mark
    pub fn should_show_center_mark(&self, style: &DimensionStyle) -> bool {
        style.center_mark_size > 0.0
    }
}

/// Diameter dimension for circles and arcs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiameterDimension {
    /// Unique identifier
    pub id: Uuid,
    /// Center point of circle/arc
    pub center_point: Point3D,
    /// First point on circle/arc perimeter
    pub chord_point1: Point3D,
    /// Second point on circle/arc perimeter (opposite side)
    pub chord_point2: Point3D,
    /// Leader endpoint (text location)
    pub leader_endpoint: Point3D,
    /// Dimension style reference
    pub style_name: String,
    /// Optional text override
    pub text_override: Option<String>,
    /// Layer name
    pub layer: String,
    /// Is this dimension associative
    pub associative: bool,
    /// Associated circle/arc entity ID
    pub associated_entity: Option<Uuid>,
}

impl DiameterDimension {
    /// Create a new diameter dimension
    pub fn new(
        center_point: Point3D,
        chord_point1: Point3D,
        chord_point2: Point3D,
        leader_endpoint: Point3D,
        style_name: impl Into<String>,
    ) -> Self {
        DiameterDimension {
            id: Uuid::new_v4(),
            center_point,
            chord_point1,
            chord_point2,
            leader_endpoint,
            style_name: style_name.into(),
            text_override: None,
            layer: "0".to_string(),
            associative: false,
            associated_entity: None,
        }
    }

    /// Create diameter dimension from circle center and radius
    pub fn from_circle(
        center: Point3D,
        radius: f64,
        angle: f64,
        style_name: impl Into<String>,
    ) -> Self {
        let chord_point1 = Point3D::new(
            center.x + radius * angle.cos(),
            center.y + radius * angle.sin(),
            center.z,
        );
        let chord_point2 = Point3D::new(
            center.x - radius * angle.cos(),
            center.y - radius * angle.sin(),
            center.z,
        );
        let leader_endpoint = Point3D::new(
            center.x + (radius * 1.5) * angle.cos(),
            center.y + (radius * 1.5) * angle.sin(),
            center.z,
        );

        DiameterDimension::new(center, chord_point1, chord_point2, leader_endpoint, style_name)
    }

    /// Calculate the diameter
    pub fn calculate_diameter(&self) -> f64 {
        self.chord_point1.distance_to(&self.chord_point2)
    }

    /// Calculate the radius
    pub fn calculate_radius(&self) -> f64 {
        self.calculate_diameter() / 2.0
    }

    /// Get dimension text with diameter symbol
    pub fn get_text(&self, style: &DimensionStyle) -> String {
        if let Some(ref override_text) = self.text_override {
            override_text.clone()
        } else {
            let diameter = self.calculate_diameter();
            format!("Ø{}", style.format_linear(diameter))
        }
    }

    /// Set text override
    pub fn set_text_override(&mut self, text: Option<String>) {
        self.text_override = text;
    }

    /// Associate with circle or arc entity
    pub fn associate_with(&mut self, entity_id: Uuid) {
        self.associative = true;
        self.associated_entity = Some(entity_id);
    }

    /// Update dimension when associated geometry changes
    pub fn update_from_circle(&mut self, center: Point3D, radius: f64) {
        if self.associative {
            self.center_point = center;
            let current_angle = self.get_leader_angle();
            self.chord_point1 = Point3D::new(
                center.x + radius * current_angle.cos(),
                center.y + radius * current_angle.sin(),
                center.z,
            );
            self.chord_point2 = Point3D::new(
                center.x - radius * current_angle.cos(),
                center.y - radius * current_angle.sin(),
                center.z,
            );
        }
    }

    /// Get the angle of the leader line
    fn get_leader_angle(&self) -> f64 {
        let dx = self.chord_point1.x - self.center_point.x;
        let dy = self.chord_point1.y - self.center_point.y;
        dy.atan2(dx)
    }
}

/// Jogged radius dimension (for large radii)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JoggedRadiusDimension {
    /// Unique identifier
    pub id: Uuid,
    /// True center point of circle/arc (may be off-screen)
    pub center_point: Point3D,
    /// Override center point (jog center)
    pub override_center: Point3D,
    /// Point on circle/arc perimeter
    pub chord_point: Point3D,
    /// Jog point (where the leader bends)
    pub jog_point: Point3D,
    /// Leader endpoint (text location)
    pub leader_endpoint: Point3D,
    /// Jog angle (default is 45 degrees)
    pub jog_angle: f64,
    /// Dimension style reference
    pub style_name: String,
    /// Optional text override
    pub text_override: Option<String>,
    /// Layer name
    pub layer: String,
    /// Is this dimension associative
    pub associative: bool,
    /// Associated circle/arc entity ID
    pub associated_entity: Option<Uuid>,
}

impl JoggedRadiusDimension {
    /// Create a new jogged radius dimension
    pub fn new(
        center_point: Point3D,
        override_center: Point3D,
        chord_point: Point3D,
        jog_point: Point3D,
        leader_endpoint: Point3D,
        style_name: impl Into<String>,
    ) -> Self {
        JoggedRadiusDimension {
            id: Uuid::new_v4(),
            center_point,
            override_center,
            chord_point,
            jog_point,
            leader_endpoint,
            jog_angle: std::f64::consts::PI / 4.0, // 45 degrees default
            style_name: style_name.into(),
            text_override: None,
            layer: "0".to_string(),
            associative: false,
            associated_entity: None,
        }
    }

    /// Calculate the actual radius
    pub fn calculate_radius(&self) -> f64 {
        let dx = self.chord_point.x - self.center_point.x;
        let dy = self.chord_point.y - self.center_point.y;
        let dz = self.chord_point.z - self.center_point.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Get dimension text with radius prefix
    pub fn get_text(&self, style: &DimensionStyle) -> String {
        if let Some(ref override_text) = self.text_override {
            override_text.clone()
        } else {
            let radius = self.calculate_radius();
            format!("R{}", style.format_linear(radius))
        }
    }

    /// Set text override
    pub fn set_text_override(&mut self, text: Option<String>) {
        self.text_override = text;
    }

    /// Associate with circle or arc entity
    pub fn associate_with(&mut self, entity_id: Uuid) {
        self.associative = true;
        self.associated_entity = Some(entity_id);
    }

    /// Get the leader segments (center to jog, jog to endpoint)
    pub fn get_leader_segments(&self) -> Vec<(Point3D, Point3D)> {
        vec![
            (self.override_center, self.jog_point),
            (self.jog_point, self.chord_point),
            (self.jog_point, self.leader_endpoint),
        ]
    }

    /// Calculate optimal jog point position
    pub fn calculate_jog_point(&self, distance_from_chord: f64) -> Point3D {
        let dx = self.chord_point.x - self.override_center.x;
        let dy = self.chord_point.y - self.override_center.y;
        let angle = dy.atan2(dx);

        Point3D::new(
            self.chord_point.x - distance_from_chord * angle.cos(),
            self.chord_point.y - distance_from_chord * angle.sin(),
            self.chord_point.z,
        )
    }
}

/// Helper for detecting arc and circle properties
pub struct RadialHelper;

impl RadialHelper {
    /// Detect if three points form a circular arc
    pub fn is_circular_arc(p1: Point3D, p2: Point3D, p3: Point3D) -> bool {
        let center = Self::calculate_center_from_three_points(p1, p2, p3);
        if let Some(c) = center {
            let r1 = p1.distance_to(&c);
            let r2 = p2.distance_to(&c);
            let r3 = p3.distance_to(&c);

            // Check if radii are approximately equal
            let tolerance = 0.0001;
            (r1 - r2).abs() < tolerance && (r2 - r3).abs() < tolerance
        } else {
            false
        }
    }

    /// Calculate circle center from three points
    pub fn calculate_center_from_three_points(
        p1: Point3D,
        p2: Point3D,
        p3: Point3D,
    ) -> Option<Point3D> {
        // Calculate perpendicular bisectors
        let mid1 = p1.midpoint(&p2);
        let mid2 = p2.midpoint(&p3);

        let dx1 = p2.x - p1.x;
        let dy1 = p2.y - p1.y;
        let dx2 = p3.x - p2.x;
        let dy2 = p3.y - p2.y;

        // Perpendicular slopes
        let slope1 = if dx1.abs() < 1e-10 {
            0.0
        } else {
            -dy1 / dx1
        };
        let slope2 = if dx2.abs() < 1e-10 {
            0.0
        } else {
            -dy2 / dx2
        };

        // Check if slopes are parallel (no unique center)
        if (slope1 - slope2).abs() < 1e-10 {
            return None;
        }

        // Calculate intersection point
        let x = (slope1 * mid1.x - slope2 * mid2.x + mid2.y - mid1.y) / (slope1 - slope2);
        let y = slope1 * (x - mid1.x) + mid1.y;

        Some(Point3D::new(x, y, (p1.z + p2.z + p3.z) / 3.0))
    }

    /// Calculate radius from three points
    pub fn calculate_radius_from_three_points(
        p1: Point3D,
        p2: Point3D,
        p3: Point3D,
    ) -> Option<f64> {
        if let Some(center) = Self::calculate_center_from_three_points(p1, p2, p3) {
            Some(p1.distance_to(&center))
        } else {
            None
        }
    }

    /// Determine if radius is too large for standard dimension
    pub fn needs_jogged_dimension(radius: f64, viewport_size: f64) -> bool {
        radius > viewport_size * 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radius_dimension() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let chord = Point3D::new(10.0, 0.0, 0.0);
        let leader = Point3D::new(15.0, 0.0, 0.0);

        let dim = RadiusDimension::new(center, chord, leader, "ISO-25");
        assert_eq!(dim.calculate_radius(), 10.0);

        let style = DimensionStyle::iso();
        assert_eq!(dim.get_text(&style), "R10");
    }

    #[test]
    fn test_diameter_dimension() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let chord1 = Point3D::new(10.0, 0.0, 0.0);
        let chord2 = Point3D::new(-10.0, 0.0, 0.0);
        let leader = Point3D::new(15.0, 0.0, 0.0);

        let dim = DiameterDimension::new(center, chord1, chord2, leader, "ISO-25");
        assert_eq!(dim.calculate_diameter(), 20.0);
        assert_eq!(dim.calculate_radius(), 10.0);

        let style = DimensionStyle::iso();
        assert_eq!(dim.get_text(&style), "Ø20");
    }

    #[test]
    fn test_radius_from_circle() {
        let center = Point3D::new(5.0, 5.0, 0.0);
        let radius = 10.0;
        let angle = std::f64::consts::PI / 4.0; // 45 degrees

        let dim = RadiusDimension::from_circle(center, radius, angle, "ISO-25");
        assert!((dim.calculate_radius() - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_center_from_three_points() {
        // Three points on a circle with center (0, 0) and radius 5
        let p1 = Point3D::new(5.0, 0.0, 0.0);
        let p2 = Point3D::new(0.0, 5.0, 0.0);
        let p3 = Point3D::new(-5.0, 0.0, 0.0);

        let center = RadialHelper::calculate_center_from_three_points(p1, p2, p3);
        assert!(center.is_some());

        let c = center.unwrap();
        assert!((c.x - 0.0).abs() < 0.001);
        assert!((c.y - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_is_circular_arc() {
        // Three points on a circle
        let p1 = Point3D::new(5.0, 0.0, 0.0);
        let p2 = Point3D::new(0.0, 5.0, 0.0);
        let p3 = Point3D::new(-5.0, 0.0, 0.0);

        assert!(RadialHelper::is_circular_arc(p1, p2, p3));

        // Three collinear points (not a circle)
        let p4 = Point3D::new(0.0, 0.0, 0.0);
        let p5 = Point3D::new(1.0, 1.0, 0.0);
        let p6 = Point3D::new(2.0, 2.0, 0.0);

        assert!(!RadialHelper::is_circular_arc(p4, p5, p6));
    }
}
