//! Dimension and annotation system
//!
//! This module provides comprehensive dimensioning and annotation capabilities
//! including linear, angular, and radial dimensions, text annotations, and leaders.
//!
//! # Features
//!
//! - **Dimension Styles**: Comprehensive styling with standard templates (ISO, ANSI, DIN, JIS)
//! - **Linear Dimensions**: Horizontal, vertical, aligned, rotated, ordinate, baseline, continue
//! - **Angular Dimensions**: 2-line, 3-point, arc length
//! - **Radial Dimensions**: Radius, diameter, jogged radius
//! - **Text Annotations**: Single-line and multi-line formatted text
//! - **Leaders**: Traditional leaders and modern multi-leaders
//! - **Associativity**: Dimensions automatically update with geometry changes
//!
//! # Example
//!
//! ```rust,ignore
//! use caddy::dimensions::{LinearDimension, DimensionStyle};
//! use caddy::dimensions::linear::Point3D;
//!
//! // Create dimension style
//! let style = DimensionStyle::iso();
//!
//! // Create horizontal dimension
//! let p1 = Point3D::new(0.0, 0.0, 0.0);
//! let p2 = Point3D::new(100.0, 0.0, 0.0);
//! let dim = LinearDimension::horizontal(p1, p2, 20.0, "ISO-25");
//!
//! // Get formatted dimension text
//! let text = dim.get_text(&style); // "100"
//! ```

pub mod style;
pub mod linear;
pub mod angular;
pub mod radial;
pub mod text;
pub mod leader;

// Re-export commonly used types
pub use style::{
    DimensionStyle,
    ArrowType,
    DimTextAlignment,
    TextVerticalPosition,
    UnitFormat,
    AngularUnitFormat,
    ToleranceFormat,
    Color,
};

pub use linear::{
    Point3D,
    LinearDimension,
    LinearDimensionType,
    OrdinateDimension,
    BaselineDimension,
    ContinueDimension,
};

pub use angular::{
    AngularDimension,
    Angular3PointDimension,
    ArcLengthDimension,
    ArcSymbolLocation,
    QuadrantAngles,
};

pub use radial::{
    RadiusDimension,
    DiameterDimension,
    JoggedRadiusDimension,
    RadialHelper,
};

pub use text::{
    Text,
    MText,
    TextStyle,
    TextHAlignment,
    TextVAlignment,
    TextAttachment,
    TextFlowDirection,
    MTextFormat,
    FormattedText,
    FieldText,
    TextField,
};

pub use leader::{
    Leader,
    MultiLeader,
    LeaderType,
    LeaderAnnotation,
    MLeaderContent,
    MLeaderStyle,
    MLeaderAttachmentSide,
    MLeaderTextAttachment,
    ToleranceAnnotation,
    ToleranceSymbol,
    MaterialCondition,
    DatumReference,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimension_workflow() {
        // Create a dimension style
        let style = DimensionStyle::iso();

        // Create a linear dimension
        let p1 = Point3D::new(0.0, 0.0, 0.0);
        let p2 = Point3D::new(50.0, 0.0, 0.0);
        let dim = LinearDimension::horizontal(p1, p2, 10.0, "ISO-25");

        // Verify measurement
        assert_eq!(dim.calculate_measurement(), 50.0);

        // Get formatted text
        let text = dim.get_text(&style);
        assert_eq!(text, "50");
    }

    #[test]
    fn test_angular_dimension_workflow() {
        let style = DimensionStyle::iso();

        let center = Point3D::new(0.0, 0.0, 0.0);
        let p1 = Point3D::new(10.0, 0.0, 0.0);
        let p2 = Point3D::new(0.0, 10.0, 0.0);
        let arc = Point3D::new(7.0, 7.0, 0.0);

        let dim = AngularDimension::new(center, p1, p2, arc, "ISO-25");

        let angle = dim.calculate_angle();
        assert!((angle - std::f64::consts::PI / 2.0).abs() < 0.01);

        let text = dim.get_text(&style);
        assert_eq!(text, "90°");
    }

    #[test]
    fn test_radial_dimension_workflow() {
        let style = DimensionStyle::iso();

        let center = Point3D::new(0.0, 0.0, 0.0);
        let radius = 25.0;
        let angle = 0.0;

        let dim = RadiusDimension::from_circle(center, radius, angle, "ISO-25");

        assert_eq!(dim.calculate_radius(), 25.0);

        let text = dim.get_text(&style);
        assert_eq!(text, "R25");
    }

    #[test]
    fn test_text_annotation_workflow() {
        let style = TextStyle::standard();

        let text = Text::new("Test Note", Point3D::new(0.0, 0.0, 0.0), 2.5, "Standard")
            .with_rotation(std::f64::consts::PI / 4.0);

        assert_eq!(text.content, "Test Note");
        assert_eq!(text.rotation, std::f64::consts::PI / 4.0);
    }

    #[test]
    fn test_leader_workflow() {
        let vertices = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(10.0, 10.0, 0.0),
            Point3D::new(20.0, 10.0, 0.0),
        ];

        let mtext = MText::new("Leader Note", Point3D::new(20.0, 10.0, 0.0), 2.5, "Standard");
        let annotation = LeaderAnnotation::MText(mtext);

        let leader = Leader::new(vertices, annotation, "ISO-25")
            .with_arrow(ArrowType::ClosedFilled, 2.5)
            .with_hookline(1);

        assert!(leader.has_hookline);
        assert_eq!(leader.arrow_type, ArrowType::ClosedFilled);
    }

    #[test]
    fn test_multileader_workflow() {
        let leader_line = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(10.0, 10.0, 0.0),
            Point3D::new(20.0, 10.0, 0.0),
        ];

        let content_pos = Point3D::new(25.0, 10.0, 0.0);
        let mtext = MText::new("MultiLeader Note", content_pos, 2.5, "Standard");

        let mleader = MultiLeader::new(
            leader_line,
            MLeaderContent::MText(mtext),
            content_pos,
            "Standard",
        );

        assert_eq!(mleader.leader_lines.len(), 1);
        assert_eq!(mleader.attachment_side, MLeaderAttachmentSide::Left);
    }
}
