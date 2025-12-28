//! Leader line types and annotations
//!
//! Provides leaders, multi-leaders, and various annotation attachments.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::style::{ArrowType, Color};
use super::linear::Point3D;
use super::text::MText;

/// Leader type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeaderType {
    /// Straight line segments
    Straight,
    /// Spline curve
    Spline,
    /// No leader line
    None,
}

/// Leader annotation type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LeaderAnnotation {
    /// Multi-line text
    MText(MText),
    /// Block reference (for symbols)
    Block { name: String, position: Point3D },
    /// Tolerance annotation
    Tolerance(ToleranceAnnotation),
    /// No annotation
    None,
}

/// Geometric tolerance annotation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToleranceAnnotation {
    /// Tolerance symbol
    pub symbol: ToleranceSymbol,
    /// First tolerance value
    pub tolerance1: f64,
    /// Second tolerance value (optional)
    pub tolerance2: Option<f64>,
    /// Material condition
    pub material_condition: MaterialCondition,
    /// Datum references
    pub datums: Vec<DatumReference>,
}

/// Geometric tolerance symbols
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToleranceSymbol {
    /// Position
    Position,
    /// Concentricity
    Concentricity,
    /// Symmetry
    Symmetry,
    /// Parallelism
    Parallelism,
    /// Perpendicularity
    Perpendicularity,
    /// Angularity
    Angularity,
    /// Cylindricity
    Cylindricity,
    /// Flatness
    Flatness,
    /// Circularity
    Circularity,
    /// Straightness
    Straightness,
    /// Profile of a surface
    ProfileSurface,
    /// Profile of a line
    ProfileLine,
    /// Circular runout
    CircularRunout,
    /// Total runout
    TotalRunout,
}

/// Material condition modifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaterialCondition {
    /// Maximum Material Condition
    MMC,
    /// Least Material Condition
    LMC,
    /// Regardless of Feature Size
    RFS,
    /// None
    None,
}

/// Datum reference
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatumReference {
    /// Datum label (A, B, C, etc.)
    pub label: String,
    /// Material condition
    pub material_condition: MaterialCondition,
}

/// Traditional leader entity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Leader {
    /// Unique identifier
    pub id: Uuid,
    /// Leader vertices (first point is arrow location)
    pub vertices: Vec<Point3D>,
    /// Leader type
    pub leader_type: LeaderType,
    /// Arrow type
    pub arrow_type: ArrowType,
    /// Arrow size
    pub arrow_size: f64,
    /// Annotation
    pub annotation: LeaderAnnotation,
    /// Dimension style reference
    pub style_name: String,
    /// Layer name
    pub layer: String,
    /// Leader color
    pub color: Color,
    /// Has hookline (horizontal line at end)
    pub has_hookline: bool,
    /// Hookline direction (1 = right, -1 = left)
    pub hookline_direction: i8,
}

impl Leader {
    /// Create a new leader
    pub fn new(
        vertices: Vec<Point3D>,
        annotation: LeaderAnnotation,
        style_name: impl Into<String>,
    ) -> Self {
        Leader {
            id: Uuid::new_v4(),
            vertices,
            leader_type: LeaderType::Straight,
            arrow_type: ArrowType::ClosedFilled,
            arrow_size: 2.5,
            annotation,
            style_name: style_name.into(),
            layer: "0".to_string(),
            color: Color::WHITE,
            has_hookline: false,
            hookline_direction: 1,
        }
    }

    /// Add a vertex to the leader
    pub fn add_vertex(&mut self, point: Point3D) {
        self.vertices.push(point);
    }

    /// Set arrow type
    pub fn with_arrow(mut self, arrow_type: ArrowType, size: f64) -> Self {
        self.arrow_type = arrow_type;
        self.arrow_size = size;
        self
    }

    /// Enable hookline
    pub fn with_hookline(mut self, direction: i8) -> Self {
        self.has_hookline = true;
        self.hookline_direction = direction;
        self
    }

    /// Get the arrow direction vector
    pub fn get_arrow_direction(&self) -> Option<(f64, f64, f64)> {
        if self.vertices.len() < 2 {
            return None;
        }

        let p1 = &self.vertices[0];
        let p2 = &self.vertices[1];

        let dx = p1.x - p2.x;
        let dy = p1.y - p2.y;
        let dz = p1.z - p2.z;
        let length = (dx * dx + dy * dy + dz * dz).sqrt();

        if length < 1e-10 {
            None
        } else {
            Some((dx / length, dy / length, dz / length))
        }
    }

    /// Get the last segment direction (for hookline)
    pub fn get_last_segment_direction(&self) -> Option<(f64, f64, f64)> {
        if self.vertices.len() < 2 {
            return None;
        }

        let n = self.vertices.len();
        let p1 = &self.vertices[n - 2];
        let p2 = &self.vertices[n - 1];

        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        let dz = p2.z - p1.z;
        let length = (dx * dx + dy * dy + dz * dz).sqrt();

        if length < 1e-10 {
            None
        } else {
            Some((dx / length, dy / length, dz / length))
        }
    }
}

/// Multi-leader content type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MLeaderContent {
    /// Multi-line text
    MText(MText),
    /// Block (symbol)
    Block {
        name: String,
        scale: f64,
        rotation: f64,
    },
    /// No content
    None,
}

/// Multi-leader attachment side
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MLeaderAttachmentSide {
    Left,
    Right,
    Top,
    Bottom,
}

/// Multi-leader text attachment type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MLeaderTextAttachment {
    /// Top of top line
    TopOfTop,
    /// Middle of top line
    MiddleOfTop,
    /// Middle of text
    MiddleOfText,
    /// Middle of bottom line
    MiddleOfBottom,
    /// Bottom of bottom line
    BottomOfBottom,
    /// Bottom line
    BottomLine,
    /// Underline bottom line
    UnderlineBottomLine,
    /// Center
    Center,
}

/// Multi-leader style
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MLeaderStyle {
    /// Style name
    pub name: String,
    /// Leader line type
    pub leader_type: LeaderType,
    /// Leader line color
    pub leader_color: Color,
    /// Leader line weight
    pub leader_weight: f64,
    /// Arrow type
    pub arrow_type: ArrowType,
    /// Arrow size
    pub arrow_size: f64,
    /// Break size (for dimension line breaks)
    pub break_size: f64,
    /// Text style name
    pub text_style_name: String,
    /// Text color
    pub text_color: Color,
    /// Text height
    pub text_height: f64,
    /// Text attachment type
    pub text_attachment: MLeaderTextAttachment,
    /// Landing distance (horizontal landing line length)
    pub landing_distance: f64,
    /// Landing gap (gap between landing and text)
    pub landing_gap: f64,
    /// Extend leader to text
    pub extend_to_text: bool,
    /// Maximum leader points
    pub max_leader_points: Option<usize>,
    /// First segment angle constraint
    pub first_segment_angle_constraint: Option<f64>,
    /// Second segment angle constraint
    pub second_segment_angle_constraint: Option<f64>,
}

impl MLeaderStyle {
    /// Create a new multi-leader style
    pub fn new(name: impl Into<String>) -> Self {
        MLeaderStyle {
            name: name.into(),
            leader_type: LeaderType::Straight,
            leader_color: Color::WHITE,
            leader_weight: 0.25,
            arrow_type: ArrowType::ClosedFilled,
            arrow_size: 2.5,
            break_size: 2.5,
            text_style_name: "Standard".to_string(),
            text_color: Color::WHITE,
            text_height: 2.5,
            text_attachment: MLeaderTextAttachment::MiddleOfText,
            landing_distance: 2.5,
            landing_gap: 1.25,
            extend_to_text: false,
            max_leader_points: None,
            first_segment_angle_constraint: None,
            second_segment_angle_constraint: None,
        }
    }

    /// Create standard multi-leader style
    pub fn standard() -> Self {
        Self::new("Standard")
    }
}

impl Default for MLeaderStyle {
    fn default() -> Self {
        Self::standard()
    }
}

/// Multi-leader entity (modern leader)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiLeader {
    /// Unique identifier
    pub id: Uuid,
    /// Leader lines (can have multiple)
    pub leader_lines: Vec<Vec<Point3D>>,
    /// Content (text or block)
    pub content: MLeaderContent,
    /// Content position
    pub content_position: Point3D,
    /// Content rotation
    pub content_rotation: f64,
    /// Multi-leader style reference
    pub style_name: String,
    /// Layer name
    pub layer: String,
    /// Attachment side
    pub attachment_side: MLeaderAttachmentSide,
    /// Text attachment type
    pub text_attachment: MLeaderTextAttachment,
    /// Landing gap override
    pub landing_gap: Option<f64>,
    /// Arrow heads enabled for each leader
    pub arrow_enabled: Vec<bool>,
}

impl MultiLeader {
    /// Create a new multi-leader
    pub fn new(
        leader_line: Vec<Point3D>,
        content: MLeaderContent,
        content_position: Point3D,
        style_name: impl Into<String>,
    ) -> Self {
        MultiLeader {
            id: Uuid::new_v4(),
            leader_lines: vec![leader_line.clone()],
            content,
            content_position,
            content_rotation: 0.0,
            style_name: style_name.into(),
            layer: "0".to_string(),
            attachment_side: Self::determine_attachment_side(&leader_line, content_position),
            text_attachment: MLeaderTextAttachment::MiddleOfText,
            landing_gap: None,
            arrow_enabled: vec![true],
        }
    }

    /// Add another leader line
    pub fn add_leader_line(&mut self, vertices: Vec<Point3D>) {
        self.leader_lines.push(vertices);
        self.arrow_enabled.push(true);
    }

    /// Remove a leader line
    pub fn remove_leader_line(&mut self, index: usize) {
        if index < self.leader_lines.len() {
            self.leader_lines.remove(index);
            self.arrow_enabled.remove(index);
        }
    }

    /// Set content rotation
    pub fn with_rotation(mut self, angle: f64) -> Self {
        self.content_rotation = angle;
        self
    }

    /// Determine which side the leader attaches to
    fn determine_attachment_side(leader_line: &[Point3D], content_pos: Point3D) -> MLeaderAttachmentSide {
        if leader_line.is_empty() {
            return MLeaderAttachmentSide::Left;
        }

        let last_point = leader_line.last().unwrap();
        let dx = content_pos.x - last_point.x;
        let dy = content_pos.y - last_point.y;

        if dx.abs() > dy.abs() {
            if dx > 0.0 {
                MLeaderAttachmentSide::Left
            } else {
                MLeaderAttachmentSide::Right
            }
        } else {
            if dy > 0.0 {
                MLeaderAttachmentSide::Bottom
            } else {
                MLeaderAttachmentSide::Top
            }
        }
    }

    /// Calculate landing line endpoints for each leader
    pub fn get_landing_lines(&self, style: &MLeaderStyle) -> Vec<(Point3D, Point3D)> {
        let mut landings = Vec::new();
        let landing_distance = style.landing_distance;

        for leader_line in &self.leader_lines {
            if leader_line.is_empty() {
                continue;
            }

            let last_point = leader_line.last().unwrap();

            // Calculate landing endpoint based on attachment side
            let landing_end = match self.attachment_side {
                MLeaderAttachmentSide::Left => Point3D::new(
                    last_point.x + landing_distance,
                    last_point.y,
                    last_point.z,
                ),
                MLeaderAttachmentSide::Right => Point3D::new(
                    last_point.x - landing_distance,
                    last_point.y,
                    last_point.z,
                ),
                MLeaderAttachmentSide::Top => Point3D::new(
                    last_point.x,
                    last_point.y - landing_distance,
                    last_point.z,
                ),
                MLeaderAttachmentSide::Bottom => Point3D::new(
                    last_point.x,
                    last_point.y + landing_distance,
                    last_point.z,
                ),
            };

            landings.push((*last_point, landing_end));
        }

        landings
    }

    /// Get arrow positions and directions
    pub fn get_arrow_positions(&self) -> Vec<(Point3D, f64)> {
        let mut arrows = Vec::new();

        for (i, leader_line) in self.leader_lines.iter().enumerate() {
            if !self.arrow_enabled[i] || leader_line.len() < 2 {
                continue;
            }

            let p1 = leader_line[0];
            let p2 = leader_line[1];
            let angle = (p1.y - p2.y).atan2(p1.x - p2.x);

            arrows.push((p1, angle));
        }

        arrows
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leader_creation() {
        let vertices = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(10.0, 10.0, 0.0),
            Point3D::new(20.0, 10.0, 0.0),
        ];

        let leader = Leader::new(
            vertices.clone(),
            LeaderAnnotation::None,
            "ISO-25",
        );

        assert_eq!(leader.vertices.len(), 3);
        assert_eq!(leader.leader_type, LeaderType::Straight);
    }

    #[test]
    fn test_leader_arrow_direction() {
        let vertices = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(10.0, 0.0, 0.0),
        ];

        let leader = Leader::new(vertices, LeaderAnnotation::None, "ISO-25");
        let dir = leader.get_arrow_direction();

        assert!(dir.is_some());
        let (dx, dy, _) = dir.unwrap();
        assert!((dx - (-1.0)).abs() < 0.01); // Points left
        assert!(dy.abs() < 0.01);
    }

    #[test]
    fn test_multileader_creation() {
        let leader_line = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(10.0, 10.0, 0.0),
            Point3D::new(20.0, 10.0, 0.0),
        ];

        let content_pos = Point3D::new(25.0, 10.0, 0.0);
        let mtext = MText::new("Note", content_pos, 2.5, "Standard");

        let mleader = MultiLeader::new(
            leader_line,
            MLeaderContent::MText(mtext),
            content_pos,
            "Standard",
        );

        assert_eq!(mleader.leader_lines.len(), 1);
        assert_eq!(mleader.attachment_side, MLeaderAttachmentSide::Left);
    }

    #[test]
    fn test_multileader_add_leader() {
        let leader_line = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(10.0, 10.0, 0.0),
        ];

        let content_pos = Point3D::new(15.0, 10.0, 0.0);
        let mut mleader = MultiLeader::new(
            leader_line,
            MLeaderContent::None,
            content_pos,
            "Standard",
        );

        let second_leader = vec![
            Point3D::new(0.0, 5.0, 0.0),
            Point3D::new(10.0, 10.0, 0.0),
        ];

        mleader.add_leader_line(second_leader);
        assert_eq!(mleader.leader_lines.len(), 2);
        assert_eq!(mleader.arrow_enabled.len(), 2);
    }

    #[test]
    fn test_tolerance_annotation() {
        let tolerance = ToleranceAnnotation {
            symbol: ToleranceSymbol::Position,
            tolerance1: 0.05,
            tolerance2: Some(0.02),
            material_condition: MaterialCondition::MMC,
            datums: vec![
                DatumReference {
                    label: "A".to_string(),
                    material_condition: MaterialCondition::MMC,
                },
                DatumReference {
                    label: "B".to_string(),
                    material_condition: MaterialCondition::RFS,
                },
            ],
        };

        assert_eq!(tolerance.datums.len(), 2);
        assert_eq!(tolerance.tolerance2, Some(0.02));
    }
}
