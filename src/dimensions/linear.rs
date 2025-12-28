//! Linear dimension types and calculations
//!
//! Provides linear, aligned, rotated, ordinate, baseline, and continue dimensions.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::style::DimensionStyle;

/// Point in 2D/3D space (placeholder until core module is available)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point3D {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Point3D { x, y, z }
    }

    pub fn distance_to(&self, other: &Point3D) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn midpoint(&self, other: &Point3D) -> Point3D {
        Point3D {
            x: (self.x + other.x) / 2.0,
            y: (self.y + other.y) / 2.0,
            z: (self.z + other.z) / 2.0,
        }
    }
}

/// Linear dimension type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinearDimensionType {
    /// Horizontal dimension
    Horizontal,
    /// Vertical dimension
    Vertical,
    /// Aligned with points
    Aligned,
    /// Rotated at specific angle
    Rotated,
}

/// Linear dimension entity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearDimension {
    /// Unique identifier
    pub id: Uuid,
    /// Dimension type
    pub dim_type: LinearDimensionType,
    /// First extension line origin point
    pub ext_line1_point: Point3D,
    /// Second extension line origin point
    pub ext_line2_point: Point3D,
    /// Dimension line location point
    pub dim_line_point: Point3D,
    /// Rotation angle for rotated dimensions (in radians)
    pub rotation_angle: f64,
    /// Dimension style reference
    pub style_name: String,
    /// Optional text override
    pub text_override: Option<String>,
    /// Layer name
    pub layer: String,
    /// Is this dimension associative to geometry
    pub associative: bool,
    /// Associated entity IDs (if associative)
    pub associated_entities: Vec<Uuid>,
}

impl LinearDimension {
    /// Create a new horizontal dimension
    pub fn horizontal(
        point1: Point3D,
        point2: Point3D,
        dim_line_y: f64,
        style_name: impl Into<String>,
    ) -> Self {
        LinearDimension {
            id: Uuid::new_v4(),
            dim_type: LinearDimensionType::Horizontal,
            ext_line1_point: point1,
            ext_line2_point: point2,
            dim_line_point: Point3D::new((point1.x + point2.x) / 2.0, dim_line_y, 0.0),
            rotation_angle: 0.0,
            style_name: style_name.into(),
            text_override: None,
            layer: "0".to_string(),
            associative: false,
            associated_entities: Vec::new(),
        }
    }

    /// Create a new vertical dimension
    pub fn vertical(
        point1: Point3D,
        point2: Point3D,
        dim_line_x: f64,
        style_name: impl Into<String>,
    ) -> Self {
        LinearDimension {
            id: Uuid::new_v4(),
            dim_type: LinearDimensionType::Vertical,
            ext_line1_point: point1,
            ext_line2_point: point2,
            dim_line_point: Point3D::new(dim_line_x, (point1.y + point2.y) / 2.0, 0.0),
            rotation_angle: 0.0,
            style_name: style_name.into(),
            text_override: None,
            layer: "0".to_string(),
            associative: false,
            associated_entities: Vec::new(),
        }
    }

    /// Create a new aligned dimension
    pub fn aligned(
        point1: Point3D,
        point2: Point3D,
        dim_line_point: Point3D,
        style_name: impl Into<String>,
    ) -> Self {
        LinearDimension {
            id: Uuid::new_v4(),
            dim_type: LinearDimensionType::Aligned,
            ext_line1_point: point1,
            ext_line2_point: point2,
            dim_line_point,
            rotation_angle: 0.0,
            style_name: style_name.into(),
            text_override: None,
            layer: "0".to_string(),
            associative: false,
            associated_entities: Vec::new(),
        }
    }

    /// Create a new rotated dimension
    pub fn rotated(
        point1: Point3D,
        point2: Point3D,
        dim_line_point: Point3D,
        angle: f64,
        style_name: impl Into<String>,
    ) -> Self {
        LinearDimension {
            id: Uuid::new_v4(),
            dim_type: LinearDimensionType::Rotated,
            ext_line1_point: point1,
            ext_line2_point: point2,
            dim_line_point,
            rotation_angle: angle,
            style_name: style_name.into(),
            text_override: None,
            layer: "0".to_string(),
            associative: false,
            associated_entities: Vec::new(),
        }
    }

    /// Calculate the measured distance
    pub fn calculate_measurement(&self) -> f64 {
        match self.dim_type {
            LinearDimensionType::Horizontal => {
                (self.ext_line2_point.x - self.ext_line1_point.x).abs()
            }
            LinearDimensionType::Vertical => {
                (self.ext_line2_point.y - self.ext_line1_point.y).abs()
            }
            LinearDimensionType::Aligned => {
                self.ext_line1_point.distance_to(&self.ext_line2_point)
            }
            LinearDimensionType::Rotated => {
                // Project points onto rotated axis
                let dx = self.ext_line2_point.x - self.ext_line1_point.x;
                let dy = self.ext_line2_point.y - self.ext_line1_point.y;
                let cos_angle = self.rotation_angle.cos();
                let sin_angle = self.rotation_angle.sin();
                (dx * cos_angle + dy * sin_angle).abs()
            }
        }
    }

    /// Get dimension text with proper formatting
    pub fn get_text(&self, style: &DimensionStyle) -> String {
        if let Some(ref override_text) = self.text_override {
            override_text.clone()
        } else {
            let measurement = self.calculate_measurement();
            style.format_linear(measurement)
        }
    }

    /// Set text override
    pub fn set_text_override(&mut self, text: Option<String>) {
        self.text_override = text;
    }

    /// Make dimension associative to entities
    pub fn associate_with(&mut self, entity_ids: Vec<Uuid>) {
        self.associative = true;
        self.associated_entities = entity_ids;
    }

    /// Update dimension when associated geometry changes
    pub fn update_associative_points(&mut self, point1: Point3D, point2: Point3D) {
        if self.associative {
            self.ext_line1_point = point1;
            self.ext_line2_point = point2;
        }
    }

    /// Get the dimension line endpoints
    pub fn get_dimension_line_points(&self) -> (Point3D, Point3D) {
        match self.dim_type {
            LinearDimensionType::Horizontal => {
                let y = self.dim_line_point.y;
                (
                    Point3D::new(self.ext_line1_point.x, y, 0.0),
                    Point3D::new(self.ext_line2_point.x, y, 0.0),
                )
            }
            LinearDimensionType::Vertical => {
                let x = self.dim_line_point.x;
                (
                    Point3D::new(x, self.ext_line1_point.y, 0.0),
                    Point3D::new(x, self.ext_line2_point.y, 0.0),
                )
            }
            LinearDimensionType::Aligned | LinearDimensionType::Rotated => {
                // Calculate parallel offset from dimension line point
                let dx = self.ext_line2_point.x - self.ext_line1_point.x;
                let dy = self.ext_line2_point.y - self.ext_line1_point.y;
                let angle = dy.atan2(dx);

                let perp_x = -angle.sin();
                let perp_y = angle.cos();

                // Find offset distance
                let mid = self.ext_line1_point.midpoint(&self.ext_line2_point);
                let offset_x = self.dim_line_point.x - mid.x;
                let offset_y = self.dim_line_point.y - mid.y;
                let offset_dist = (offset_x * offset_x + offset_y * offset_y).sqrt();

                let sign = if (offset_x * perp_x + offset_y * perp_y) > 0.0 { 1.0 } else { -1.0 };

                (
                    Point3D::new(
                        self.ext_line1_point.x + perp_x * offset_dist * sign,
                        self.ext_line1_point.y + perp_y * offset_dist * sign,
                        0.0,
                    ),
                    Point3D::new(
                        self.ext_line2_point.x + perp_x * offset_dist * sign,
                        self.ext_line2_point.y + perp_y * offset_dist * sign,
                        0.0,
                    ),
                )
            }
        }
    }
}

/// Ordinate dimension (datum dimensions)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrdinateDimension {
    /// Unique identifier
    pub id: Uuid,
    /// Feature point
    pub feature_point: Point3D,
    /// Leader endpoint
    pub leader_endpoint: Point3D,
    /// Origin point (datum)
    pub datum_point: Point3D,
    /// Is this an X ordinate (true) or Y ordinate (false)
    pub is_x_ordinate: bool,
    /// Dimension style reference
    pub style_name: String,
    /// Optional text override
    pub text_override: Option<String>,
    /// Layer name
    pub layer: String,
}

impl OrdinateDimension {
    /// Create a new ordinate dimension
    pub fn new(
        feature_point: Point3D,
        leader_endpoint: Point3D,
        datum_point: Point3D,
        is_x_ordinate: bool,
        style_name: impl Into<String>,
    ) -> Self {
        OrdinateDimension {
            id: Uuid::new_v4(),
            feature_point,
            leader_endpoint,
            datum_point,
            is_x_ordinate,
            style_name: style_name.into(),
            text_override: None,
            layer: "0".to_string(),
        }
    }

    /// Calculate the ordinate value
    pub fn calculate_measurement(&self) -> f64 {
        if self.is_x_ordinate {
            self.feature_point.x - self.datum_point.x
        } else {
            self.feature_point.y - self.datum_point.y
        }
    }

    /// Get dimension text
    pub fn get_text(&self, style: &DimensionStyle) -> String {
        if let Some(ref override_text) = self.text_override {
            override_text.clone()
        } else {
            let measurement = self.calculate_measurement();
            style.format_linear(measurement)
        }
    }
}

/// Baseline dimension (multiple dimensions from same baseline)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaselineDimension {
    /// Unique identifier
    pub id: Uuid,
    /// Base dimension
    pub base_dimension_id: Uuid,
    /// Additional dimension points
    pub dimension_points: Vec<Point3D>,
    /// Spacing between dimension lines
    pub spacing: f64,
    /// Dimension style reference
    pub style_name: String,
    /// Layer name
    pub layer: String,
}

impl BaselineDimension {
    /// Create a new baseline dimension
    pub fn new(
        base_dimension_id: Uuid,
        spacing: f64,
        style_name: impl Into<String>,
    ) -> Self {
        BaselineDimension {
            id: Uuid::new_v4(),
            base_dimension_id,
            dimension_points: Vec::new(),
            spacing,
            style_name: style_name.into(),
            layer: "0".to_string(),
        }
    }

    /// Add a dimension point
    pub fn add_point(&mut self, point: Point3D) {
        self.dimension_points.push(point);
    }

    /// Get all dimension lines
    pub fn get_dimension_lines(&self, base_point: Point3D) -> Vec<(Point3D, Point3D)> {
        let mut lines = Vec::new();
        for (i, point) in self.dimension_points.iter().enumerate() {
            let offset = self.spacing * (i + 1) as f64;
            lines.push((
                Point3D::new(base_point.x, base_point.y + offset, 0.0),
                Point3D::new(point.x, point.y + offset, 0.0),
            ));
        }
        lines
    }
}

/// Continue dimension (chain dimensions)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContinueDimension {
    /// Unique identifier
    pub id: Uuid,
    /// Previous dimension ID
    pub previous_dimension_id: Uuid,
    /// Continue points
    pub continue_points: Vec<Point3D>,
    /// Dimension style reference
    pub style_name: String,
    /// Layer name
    pub layer: String,
}

impl ContinueDimension {
    /// Create a new continue dimension
    pub fn new(
        previous_dimension_id: Uuid,
        style_name: impl Into<String>,
    ) -> Self {
        ContinueDimension {
            id: Uuid::new_v4(),
            previous_dimension_id,
            continue_points: Vec::new(),
            style_name: style_name.into(),
            layer: "0".to_string(),
        }
    }

    /// Add a continue point
    pub fn add_point(&mut self, point: Point3D) {
        self.continue_points.push(point);
    }

    /// Get all dimension segments
    pub fn get_dimension_segments(&self, start_point: Point3D) -> Vec<(Point3D, Point3D)> {
        let mut segments = Vec::new();
        let mut current = start_point;

        for point in &self.continue_points {
            segments.push((current, *point));
            current = *point;
        }

        segments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_horizontal_dimension() {
        let p1 = Point3D::new(0.0, 0.0, 0.0);
        let p2 = Point3D::new(10.0, 5.0, 0.0);
        let dim = LinearDimension::horizontal(p1, p2, 10.0, "ISO-25");

        assert_eq!(dim.calculate_measurement(), 10.0);
        assert_eq!(dim.dim_type, LinearDimensionType::Horizontal);
    }

    #[test]
    fn test_vertical_dimension() {
        let p1 = Point3D::new(0.0, 0.0, 0.0);
        let p2 = Point3D::new(5.0, 10.0, 0.0);
        let dim = LinearDimension::vertical(p1, p2, 10.0, "ISO-25");

        assert_eq!(dim.calculate_measurement(), 10.0);
        assert_eq!(dim.dim_type, LinearDimensionType::Vertical);
    }

    #[test]
    fn test_aligned_dimension() {
        let p1 = Point3D::new(0.0, 0.0, 0.0);
        let p2 = Point3D::new(3.0, 4.0, 0.0);
        let dim_point = Point3D::new(1.5, 2.0, 0.0);
        let dim = LinearDimension::aligned(p1, p2, dim_point, "ISO-25");

        assert_eq!(dim.calculate_measurement(), 5.0); // 3-4-5 triangle
        assert_eq!(dim.dim_type, LinearDimensionType::Aligned);
    }

    #[test]
    fn test_ordinate_dimension() {
        let feature = Point3D::new(10.0, 20.0, 0.0);
        let leader = Point3D::new(15.0, 20.0, 0.0);
        let datum = Point3D::new(0.0, 0.0, 0.0);
        let dim = OrdinateDimension::new(feature, leader, datum, true, "ISO-25");

        assert_eq!(dim.calculate_measurement(), 10.0);
    }

    #[test]
    fn test_text_override() {
        let p1 = Point3D::new(0.0, 0.0, 0.0);
        let p2 = Point3D::new(10.0, 0.0, 0.0);
        let mut dim = LinearDimension::horizontal(p1, p2, 5.0, "ISO-25");

        dim.set_text_override(Some("Custom Text".to_string()));
        let style = DimensionStyle::iso();
        assert_eq!(dim.get_text(&style), "Custom Text");
    }
}
