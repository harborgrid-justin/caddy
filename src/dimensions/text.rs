//! Text annotation types
//!
//! Provides single-line text, multi-line formatted text, and text styling.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::style::Color;
use super::linear::Point3D;

/// Text horizontal alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextHAlignment {
    /// Left aligned
    Left,
    /// Center aligned
    Center,
    /// Right aligned
    Right,
    /// Justified
    Justified,
    /// Aligned (fit between points)
    Aligned,
    /// Middle (centered both ways)
    Middle,
    /// Fit (squeeze/stretch to fit)
    Fit,
}

/// Text vertical alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextVAlignment {
    /// Baseline
    Baseline,
    /// Bottom
    Bottom,
    /// Middle
    Middle,
    /// Top
    Top,
}

/// Text attachment point (combination of H and V)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAttachment {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    MiddleCenter,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl TextAttachment {
    pub fn to_h_v_alignment(&self) -> (TextHAlignment, TextVAlignment) {
        match self {
            TextAttachment::TopLeft => (TextHAlignment::Left, TextVAlignment::Top),
            TextAttachment::TopCenter => (TextHAlignment::Center, TextVAlignment::Top),
            TextAttachment::TopRight => (TextHAlignment::Right, TextVAlignment::Top),
            TextAttachment::MiddleLeft => (TextHAlignment::Left, TextVAlignment::Middle),
            TextAttachment::MiddleCenter => (TextHAlignment::Center, TextVAlignment::Middle),
            TextAttachment::MiddleRight => (TextHAlignment::Right, TextVAlignment::Middle),
            TextAttachment::BottomLeft => (TextHAlignment::Left, TextVAlignment::Bottom),
            TextAttachment::BottomCenter => (TextHAlignment::Center, TextVAlignment::Bottom),
            TextAttachment::BottomRight => (TextHAlignment::Right, TextVAlignment::Bottom),
        }
    }
}

/// Text style definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextStyle {
    /// Style name
    pub name: String,
    /// Font name
    pub font_name: String,
    /// Font file path (optional)
    pub font_file: Option<String>,
    /// Text height
    pub height: f64,
    /// Width factor (1.0 = normal)
    pub width_factor: f64,
    /// Oblique angle in radians
    pub oblique_angle: f64,
    /// Is text backwards
    pub backwards: bool,
    /// Is text upside down
    pub upside_down: bool,
    /// Is text vertical
    pub vertical: bool,
    /// Text color
    pub color: Color,
    /// Character spacing
    pub char_spacing: f64,
    /// Line spacing factor (for multi-line)
    pub line_spacing: f64,
}

impl TextStyle {
    /// Create a new text style
    pub fn new(name: impl Into<String>) -> Self {
        TextStyle {
            name: name.into(),
            font_name: "Arial".to_string(),
            font_file: None,
            height: 2.5,
            width_factor: 1.0,
            oblique_angle: 0.0,
            backwards: false,
            upside_down: false,
            vertical: false,
            color: Color::WHITE,
            char_spacing: 0.0,
            line_spacing: 1.0,
        }
    }

    /// Create standard text style
    pub fn standard() -> Self {
        Self::new("Standard")
    }

    /// Create annotative text style
    pub fn annotative() -> Self {
        let mut style = Self::new("Annotative");
        style.height = 0.0; // Height set by annotation scale
        style
    }

    /// Apply width factor to height
    pub fn effective_width(&self) -> f64 {
        self.height * self.width_factor
    }
}

impl Default for TextStyle {
    fn default() -> Self {
        Self::standard()
    }
}

/// Single-line text entity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Text {
    /// Unique identifier
    pub id: Uuid,
    /// Text content
    pub content: String,
    /// Insertion point
    pub position: Point3D,
    /// Text height (overrides style if set)
    pub height: Option<f64>,
    /// Rotation angle in radians
    pub rotation: f64,
    /// Horizontal alignment
    pub h_align: TextHAlignment,
    /// Vertical alignment
    pub v_align: TextVAlignment,
    /// Text style reference
    pub style_name: String,
    /// Layer name
    pub layer: String,
    /// Width factor override
    pub width_factor: Option<f64>,
    /// Oblique angle override
    pub oblique_angle: Option<f64>,
}

impl Text {
    /// Create a new text entity
    pub fn new(
        content: impl Into<String>,
        position: Point3D,
        height: f64,
        style_name: impl Into<String>,
    ) -> Self {
        Text {
            id: Uuid::new_v4(),
            content: content.into(),
            position,
            height: Some(height),
            rotation: 0.0,
            h_align: TextHAlignment::Left,
            v_align: TextVAlignment::Baseline,
            style_name: style_name.into(),
            layer: "0".to_string(),
            width_factor: None,
            oblique_angle: None,
        }
    }

    /// Set rotation angle
    pub fn with_rotation(mut self, angle: f64) -> Self {
        self.rotation = angle;
        self
    }

    /// Set alignment
    pub fn with_alignment(mut self, h_align: TextHAlignment, v_align: TextVAlignment) -> Self {
        self.h_align = h_align;
        self.v_align = v_align;
        self
    }

    /// Set layer
    pub fn with_layer(mut self, layer: impl Into<String>) -> Self {
        self.layer = layer.into();
        self
    }

    /// Get effective height
    pub fn get_height(&self, style: &TextStyle) -> f64 {
        self.height.unwrap_or(style.height)
    }

    /// Get effective width factor
    pub fn get_width_factor(&self, style: &TextStyle) -> f64 {
        self.width_factor.unwrap_or(style.width_factor)
    }

    /// Calculate text bounding box
    pub fn bounding_box(&self, style: &TextStyle) -> (Point3D, Point3D) {
        let height = self.get_height(style);
        let width_factor = self.get_width_factor(style);
        let estimated_width = self.content.len() as f64 * height * width_factor * 0.6;

        let min = self.position;
        let max = Point3D::new(
            self.position.x + estimated_width,
            self.position.y + height,
            self.position.z,
        );

        (min, max)
    }
}

/// Multi-line text formatting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MTextFormat {
    Bold,
    Italic,
    Underline,
    Overline,
    Strikethrough,
}

/// Multi-line text content with formatting
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormattedText {
    /// Text content
    pub text: String,
    /// Start index in parent string
    pub start: usize,
    /// End index in parent string
    pub end: usize,
    /// Applied formats
    pub formats: Vec<MTextFormat>,
    /// Font override
    pub font: Option<String>,
    /// Color override
    pub color: Option<Color>,
    /// Height override
    pub height: Option<f64>,
}

/// Multi-line text entity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MText {
    /// Unique identifier
    pub id: Uuid,
    /// Raw text content (with formatting codes)
    pub raw_content: String,
    /// Parsed formatted text segments
    pub formatted_segments: Vec<FormattedText>,
    /// Insertion point
    pub position: Point3D,
    /// Text height
    pub height: f64,
    /// Rotation angle in radians
    pub rotation: f64,
    /// Attachment point
    pub attachment: TextAttachment,
    /// Text width (for wrapping)
    pub width: Option<f64>,
    /// Line spacing factor (1.0 = normal, 1.5 = 1.5x, etc.)
    pub line_spacing: f64,
    /// Text style reference
    pub style_name: String,
    /// Layer name
    pub layer: String,
    /// Text flow direction
    pub flow_direction: TextFlowDirection,
}

/// Text flow direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextFlowDirection {
    /// Left to right
    LeftToRight,
    /// Right to left
    RightToLeft,
    /// Top to bottom
    TopToBottom,
    /// Bottom to top
    BottomToTop,
}

impl MText {
    /// Create a new multi-line text entity
    pub fn new(
        content: impl Into<String>,
        position: Point3D,
        height: f64,
        style_name: impl Into<String>,
    ) -> Self {
        let raw_content = content.into();
        let formatted_segments = Self::parse_formatting(&raw_content);

        MText {
            id: Uuid::new_v4(),
            raw_content,
            formatted_segments,
            position,
            height,
            rotation: 0.0,
            attachment: TextAttachment::TopLeft,
            width: None,
            line_spacing: 1.0,
            style_name: style_name.into(),
            layer: "0".to_string(),
            flow_direction: TextFlowDirection::LeftToRight,
        }
    }

    /// Set text width for wrapping
    pub fn with_width(mut self, width: f64) -> Self {
        self.width = Some(width);
        self
    }

    /// Set attachment point
    pub fn with_attachment(mut self, attachment: TextAttachment) -> Self {
        self.attachment = attachment;
        self
    }

    /// Set rotation
    pub fn with_rotation(mut self, angle: f64) -> Self {
        self.rotation = angle;
        self
    }

    /// Set line spacing
    pub fn with_line_spacing(mut self, spacing: f64) -> Self {
        self.line_spacing = spacing;
        self
    }

    /// Parse formatting codes from text
    /// Simple implementation - real CAD systems use complex formatting
    fn parse_formatting(text: &str) -> Vec<FormattedText> {
        // For now, just treat as plain text
        // In a full implementation, this would parse codes like:
        // \fFontName|b1|i1 for bold italic
        // \C1; for color
        // etc.
        vec![FormattedText {
            text: text.to_string(),
            start: 0,
            end: text.len(),
            formats: Vec::new(),
            font: None,
            color: None,
            height: None,
        }]
    }

    /// Get plain text without formatting codes
    pub fn get_plain_text(&self) -> String {
        self.formatted_segments
            .iter()
            .map(|seg| seg.text.as_str())
            .collect::<Vec<_>>()
            .join("")
    }

    /// Split text into lines based on width
    pub fn get_wrapped_lines(&self, style: &TextStyle) -> Vec<String> {
        let plain_text = self.get_plain_text();

        if let Some(max_width) = self.width {
            let mut lines = Vec::new();
            let mut current_line = String::new();
            let char_width = self.height * style.width_factor * 0.6;

            for word in plain_text.split_whitespace() {
                let word_width = word.len() as f64 * char_width;
                let current_width = current_line.len() as f64 * char_width;

                if current_width + word_width > max_width && !current_line.is_empty() {
                    lines.push(current_line.clone());
                    current_line.clear();
                }

                if !current_line.is_empty() {
                    current_line.push(' ');
                }
                current_line.push_str(word);
            }

            if !current_line.is_empty() {
                lines.push(current_line);
            }

            lines
        } else {
            // No wrapping - split on newlines
            plain_text.lines().map(|s| s.to_string()).collect()
        }
    }

    /// Calculate bounding box
    pub fn bounding_box(&self, style: &TextStyle) -> (Point3D, Point3D) {
        let lines = self.get_wrapped_lines(style);
        let num_lines = lines.len() as f64;
        let max_line_length = lines.iter().map(|l| l.len()).max().unwrap_or(0) as f64;

        let char_width = self.height * style.width_factor * 0.6;
        let total_width = max_line_length * char_width;
        let total_height = num_lines * self.height * self.line_spacing;

        let min = self.position;
        let max = Point3D::new(
            self.position.x + total_width,
            self.position.y + total_height,
            self.position.z,
        );

        (min, max)
    }

    /// Update text content and reparse formatting
    pub fn set_content(&mut self, content: impl Into<String>) {
        self.raw_content = content.into();
        self.formatted_segments = Self::parse_formatting(&self.raw_content);
    }
}

/// Text field (dynamic text)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextField {
    /// Current date
    Date,
    /// Current time
    Time,
    /// File name
    FileName,
    /// File path
    FilePath,
    /// Drawing number
    DrawingNumber,
    /// Sheet number
    SheetNumber,
    /// Custom field
    Custom(String),
}

/// Text with fields (for title blocks, etc.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldText {
    /// Base MText
    pub mtext: MText,
    /// Fields in the text
    pub fields: Vec<(String, TextField)>,
}

impl FieldText {
    /// Create a new field text
    pub fn new(
        content: impl Into<String>,
        position: Point3D,
        height: f64,
        style_name: impl Into<String>,
    ) -> Self {
        FieldText {
            mtext: MText::new(content, position, height, style_name),
            fields: Vec::new(),
        }
    }

    /// Add a field
    pub fn add_field(&mut self, placeholder: impl Into<String>, field: TextField) {
        self.fields.push((placeholder.into(), field));
    }

    /// Evaluate fields and get final text
    pub fn evaluate(&self) -> String {
        let mut result = self.mtext.raw_content.clone();

        for (placeholder, field) in &self.fields {
            let value = self.evaluate_field(field);
            result = result.replace(placeholder, &value);
        }

        result
    }

    fn evaluate_field(&self, field: &TextField) -> String {
        match field {
            TextField::Date => {
                // In real implementation, get current date
                "2025-12-28".to_string()
            }
            TextField::Time => {
                // In real implementation, get current time
                "12:00:00".to_string()
            }
            TextField::FileName => "drawing.cad".to_string(),
            TextField::FilePath => "/path/to/drawing.cad".to_string(),
            TextField::DrawingNumber => "DWG-001".to_string(),
            TextField::SheetNumber => "1".to_string(),
            TextField::Custom(value) => value.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_creation() {
        let text = Text::new("Hello World", Point3D::new(0.0, 0.0, 0.0), 2.5, "Standard");
        assert_eq!(text.content, "Hello World");
        assert_eq!(text.height, Some(2.5));
    }

    #[test]
    fn test_text_with_rotation() {
        let text = Text::new("Test", Point3D::new(0.0, 0.0, 0.0), 2.5, "Standard")
            .with_rotation(std::f64::consts::PI / 4.0);
        assert_eq!(text.rotation, std::f64::consts::PI / 4.0);
    }

    #[test]
    fn test_mtext_creation() {
        let mtext = MText::new(
            "Line 1\nLine 2\nLine 3",
            Point3D::new(0.0, 0.0, 0.0),
            2.5,
            "Standard",
        );
        assert_eq!(mtext.get_plain_text(), "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_mtext_wrapping() {
        let mtext = MText::new(
            "This is a long line of text that should wrap",
            Point3D::new(0.0, 0.0, 0.0),
            2.5,
            "Standard",
        )
        .with_width(50.0);

        let style = TextStyle::standard();
        let lines = mtext.get_wrapped_lines(&style);
        assert!(lines.len() > 1);
    }

    #[test]
    fn test_text_attachment() {
        let (h, v) = TextAttachment::MiddleCenter.to_h_v_alignment();
        assert_eq!(h, TextHAlignment::Center);
        assert_eq!(v, TextVAlignment::Middle);
    }

    #[test]
    fn test_field_text() {
        let mut field_text = FieldText::new(
            "Date: {DATE}",
            Point3D::new(0.0, 0.0, 0.0),
            2.5,
            "Standard",
        );
        field_text.add_field("{DATE}", TextField::Date);

        let evaluated = field_text.evaluate();
        assert!(evaluated.contains("2025-12-28"));
    }
}
