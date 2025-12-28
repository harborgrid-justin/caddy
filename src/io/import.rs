// CADDY - Enterprise CAD System
// File I/O System - Import Formats Module
// Agent 6 - File I/O System Developer

use crate::io::document::*;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use thiserror::Error;

/// Import-related errors
#[derive(Error, Debug)]
pub enum ImportError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    #[error("Invalid file format: {0}")]
    InvalidFormat(String),
}

pub type ImportResult<T> = Result<T, ImportError>;

/// SVG import settings
#[derive(Debug, Clone)]
pub struct SvgImportSettings {
    /// Scale factor
    pub scale: f64,
    /// Default layer for imported entities
    pub default_layer: String,
    /// Simplification tolerance
    pub tolerance: f64,
    /// Convert text to paths
    pub convert_text_to_paths: bool,
}

impl Default for SvgImportSettings {
    fn default() -> Self {
        Self {
            scale: 1.0,
            default_layer: "0".to_string(),
            tolerance: 0.01,
            convert_text_to_paths: false,
        }
    }
}

/// SVG importer (basic implementation)
pub struct SvgImporter {
    settings: SvgImportSettings,
}

impl SvgImporter {
    /// Create a new SVG importer
    pub fn new(settings: SvgImportSettings) -> Self {
        Self { settings }
    }

    /// Import an SVG file
    pub fn import<P: AsRef<Path>>(&self, path: P) -> ImportResult<Document> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        self.import_from_reader(reader)
    }

    /// Import from a reader
    pub fn import_from_reader<R: BufRead>(&self, mut reader: R) -> ImportResult<Document> {
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        self.import_from_string(&content)
    }

    /// Import from SVG string
    pub fn import_from_string(&self, svg: &str) -> ImportResult<Document> {
        let mut doc = Document::new();

        // Basic SVG parsing - in production, use an XML/SVG parsing library
        // This is a simplified implementation

        // Extract view box for scaling
        if let Some(viewbox) = self.extract_viewbox(svg) {
            doc.variables
                .insert("svg_viewbox".to_string(), format!("{:?}", viewbox));
        }

        // Parse basic shapes
        self.parse_lines(svg, &mut doc)?;
        self.parse_circles(svg, &mut doc)?;
        self.parse_rects(svg, &mut doc)?;
        self.parse_polygons(svg, &mut doc)?;
        self.parse_polylines(svg, &mut doc)?;
        self.parse_paths(svg, &mut doc)?;

        Ok(doc)
    }

    fn extract_viewbox(&self, svg: &str) -> Option<(f64, f64, f64, f64)> {
        // Simple regex-like extraction (in production, use proper XML parsing)
        if let Some(start) = svg.find("viewBox=\"") {
            let start = start + 9;
            if let Some(end) = svg[start..].find('"') {
                let viewbox_str = &svg[start..start + end];
                let parts: Vec<&str> = viewbox_str.split_whitespace().collect();
                if parts.len() == 4 {
                    let x = parts[0].parse().ok()?;
                    let y = parts[1].parse().ok()?;
                    let w = parts[2].parse().ok()?;
                    let h = parts[3].parse().ok()?;
                    return Some((x, y, w, h));
                }
            }
        }
        None
    }

    fn parse_lines(&self, svg: &str, doc: &mut Document) -> ImportResult<()> {
        // Find all <line> elements
        let mut pos = 0;
        while let Some(line_start) = svg[pos..].find("<line ") {
            pos += line_start;
            if let Some(line_end) = svg[pos..].find('>') {
                let line_tag = &svg[pos..pos + line_end];

                if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
                    self.extract_attribute(line_tag, "x1"),
                    self.extract_attribute(line_tag, "y1"),
                    self.extract_attribute(line_tag, "x2"),
                    self.extract_attribute(line_tag, "y2"),
                ) {
                    let line = Line {
                        start: Vec3::new(
                            x1 * self.settings.scale,
                            -y1 * self.settings.scale, // Invert Y
                            0.0,
                        ),
                        end: Vec3::new(
                            x2 * self.settings.scale,
                            -y2 * self.settings.scale,
                            0.0,
                        ),
                    };

                    doc.add_entity(Entity::new(
                        GeometryType::Line(line),
                        self.settings.default_layer.clone(),
                    ));
                }

                pos += line_end + 1;
            } else {
                break;
            }
        }

        Ok(())
    }

    fn parse_circles(&self, svg: &str, doc: &mut Document) -> ImportResult<()> {
        let mut pos = 0;
        while let Some(circle_start) = svg[pos..].find("<circle ") {
            pos += circle_start;
            if let Some(circle_end) = svg[pos..].find('>') {
                let circle_tag = &svg[pos..pos + circle_end];

                if let (Some(cx), Some(cy), Some(r)) = (
                    self.extract_attribute(circle_tag, "cx"),
                    self.extract_attribute(circle_tag, "cy"),
                    self.extract_attribute(circle_tag, "r"),
                ) {
                    let circle = Circle {
                        center: Vec3::new(
                            cx * self.settings.scale,
                            -cy * self.settings.scale,
                            0.0,
                        ),
                        radius: r * self.settings.scale,
                        normal: Vec3::unit_z(),
                    };

                    doc.add_entity(Entity::new(
                        GeometryType::Circle(circle),
                        self.settings.default_layer.clone(),
                    ));
                }

                pos += circle_end + 1;
            } else {
                break;
            }
        }

        Ok(())
    }

    fn parse_rects(&self, svg: &str, doc: &mut Document) -> ImportResult<()> {
        let mut pos = 0;
        while let Some(rect_start) = svg[pos..].find("<rect ") {
            pos += rect_start;
            if let Some(rect_end) = svg[pos..].find('>') {
                let rect_tag = &svg[pos..pos + rect_end];

                if let (Some(x), Some(y), Some(w), Some(h)) = (
                    self.extract_attribute(rect_tag, "x"),
                    self.extract_attribute(rect_tag, "y"),
                    self.extract_attribute(rect_tag, "width"),
                    self.extract_attribute(rect_tag, "height"),
                ) {
                    // Convert rectangle to polyline
                    let vertices = vec![
                        Vertex {
                            position: Vec3::new(x * self.settings.scale, -y * self.settings.scale, 0.0),
                            bulge: 0.0,
                        },
                        Vertex {
                            position: Vec3::new(
                                (x + w) * self.settings.scale,
                                -y * self.settings.scale,
                                0.0,
                            ),
                            bulge: 0.0,
                        },
                        Vertex {
                            position: Vec3::new(
                                (x + w) * self.settings.scale,
                                -(y + h) * self.settings.scale,
                                0.0,
                            ),
                            bulge: 0.0,
                        },
                        Vertex {
                            position: Vec3::new(
                                x * self.settings.scale,
                                -(y + h) * self.settings.scale,
                                0.0,
                            ),
                            bulge: 0.0,
                        },
                    ];

                    let polyline = Polyline {
                        vertices,
                        closed: true,
                    };

                    doc.add_entity(Entity::new(
                        GeometryType::Polyline(polyline),
                        self.settings.default_layer.clone(),
                    ));
                }

                pos += rect_end + 1;
            } else {
                break;
            }
        }

        Ok(())
    }

    fn parse_polygons(&self, svg: &str, doc: &mut Document) -> ImportResult<()> {
        let mut pos = 0;
        while let Some(poly_start) = svg[pos..].find("<polygon ") {
            pos += poly_start;
            if let Some(poly_end) = svg[pos..].find('>') {
                let poly_tag = &svg[pos..pos + poly_end];

                if let Some(points_str) = self.extract_attribute_string(poly_tag, "points") {
                    let vertices = self.parse_points(&points_str)?;

                    let polyline = Polyline {
                        vertices,
                        closed: true,
                    };

                    doc.add_entity(Entity::new(
                        GeometryType::Polyline(polyline),
                        self.settings.default_layer.clone(),
                    ));
                }

                pos += poly_end + 1;
            } else {
                break;
            }
        }

        Ok(())
    }

    fn parse_polylines(&self, svg: &str, doc: &mut Document) -> ImportResult<()> {
        let mut pos = 0;
        while let Some(poly_start) = svg[pos..].find("<polyline ") {
            pos += poly_start;
            if let Some(poly_end) = svg[pos..].find('>') {
                let poly_tag = &svg[pos..pos + poly_end];

                if let Some(points_str) = self.extract_attribute_string(poly_tag, "points") {
                    let vertices = self.parse_points(&points_str)?;

                    let polyline = Polyline {
                        vertices,
                        closed: false,
                    };

                    doc.add_entity(Entity::new(
                        GeometryType::Polyline(polyline),
                        self.settings.default_layer.clone(),
                    ));
                }

                pos += poly_end + 1;
            } else {
                break;
            }
        }

        Ok(())
    }

    fn parse_paths(&self, svg: &str, doc: &mut Document) -> ImportResult<()> {
        let mut pos = 0;
        while let Some(path_start) = svg[pos..].find("<path ") {
            pos += path_start;
            if let Some(path_end) = svg[pos..].find('>') {
                let path_tag = &svg[pos..pos + path_end];

                if let Some(d) = self.extract_attribute_string(path_tag, "d") {
                    // Basic path parsing - convert to polyline
                    if let Ok(vertices) = self.parse_path_data(&d) {
                        let polyline = Polyline {
                            vertices,
                            closed: d.contains('Z') || d.contains('z'),
                        };

                        doc.add_entity(Entity::new(
                            GeometryType::Polyline(polyline),
                            self.settings.default_layer.clone(),
                        ));
                    }
                }

                pos += path_end + 1;
            } else {
                break;
            }
        }

        Ok(())
    }

    fn parse_points(&self, points_str: &str) -> ImportResult<Vec<Vertex>> {
        let mut vertices = Vec::new();
        let parts: Vec<&str> = points_str.split(|c| c == ',' || c == ' ').collect();

        let mut i = 0;
        while i + 1 < parts.len() {
            if let (Ok(x), Ok(y)) = (parts[i].parse::<f64>(), parts[i + 1].parse::<f64>()) {
                vertices.push(Vertex {
                    position: Vec3::new(
                        x * self.settings.scale,
                        -y * self.settings.scale,
                        0.0,
                    ),
                    bulge: 0.0,
                });
            }
            i += 2;
        }

        Ok(vertices)
    }

    fn parse_path_data(&self, d: &str) -> ImportResult<Vec<Vertex>> {
        let mut vertices = Vec::new();
        let mut current_x = 0.0;
        let mut current_y = 0.0;

        // Very simplified path parser - only handles M and L commands
        let commands: Vec<&str> = d.split_whitespace().collect();
        let mut i = 0;

        while i < commands.len() {
            let cmd = commands[i];

            match cmd {
                "M" | "m" => {
                    // Move to
                    if i + 2 < commands.len() {
                        if let (Ok(x), Ok(y)) = (
                            commands[i + 1].parse::<f64>(),
                            commands[i + 2].parse::<f64>(),
                        ) {
                            current_x = x * self.settings.scale;
                            current_y = -y * self.settings.scale;
                            vertices.push(Vertex {
                                position: Vec3::new(current_x, current_y, 0.0),
                                bulge: 0.0,
                            });
                            i += 2;
                        }
                    }
                }
                "L" | "l" => {
                    // Line to
                    if i + 2 < commands.len() {
                        if let (Ok(x), Ok(y)) = (
                            commands[i + 1].parse::<f64>(),
                            commands[i + 2].parse::<f64>(),
                        ) {
                            current_x = x * self.settings.scale;
                            current_y = -y * self.settings.scale;
                            vertices.push(Vertex {
                                position: Vec3::new(current_x, current_y, 0.0),
                                bulge: 0.0,
                            });
                            i += 2;
                        }
                    }
                }
                _ => {
                    // Skip unsupported commands
                }
            }

            i += 1;
        }

        Ok(vertices)
    }

    fn extract_attribute(&self, tag: &str, attr: &str) -> Option<f64> {
        self.extract_attribute_string(tag, attr)?
            .parse()
            .ok()
    }

    fn extract_attribute_string(&self, tag: &str, attr: &str) -> Option<String> {
        let pattern = format!("{}=\"", attr);
        if let Some(start) = tag.find(&pattern) {
            let start = start + pattern.len();
            if let Some(end) = tag[start..].find('"') {
                return Some(tag[start..start + end].to_string());
            }
        }
        None
    }
}

impl Default for SvgImporter {
    fn default() -> Self {
        Self::new(SvgImportSettings::default())
    }
}

/// Image import settings
#[derive(Debug, Clone)]
pub struct ImageImportSettings {
    /// Trace as vectors
    pub vectorize: bool,
    /// Tracing threshold
    pub threshold: u8,
    /// Simplification tolerance
    pub tolerance: f64,
    /// Default layer
    pub default_layer: String,
    /// Scale factor (pixels to units)
    pub scale: f64,
}

impl Default for ImageImportSettings {
    fn default() -> Self {
        Self {
            vectorize: false,
            threshold: 128,
            tolerance: 1.0,
            default_layer: "0".to_string(),
            scale: 1.0,
        }
    }
}

/// Image importer (for bitmap to vector conversion)
pub struct ImageImporter {
    settings: ImageImportSettings,
}

impl ImageImporter {
    /// Create a new image importer
    pub fn new(settings: ImageImportSettings) -> Self {
        Self { settings }
    }

    /// Import an image file
    pub fn import<P: AsRef<Path>>(&self, _path: P) -> ImportResult<Document> {
        // This would require image processing and vectorization
        // Using the image crate and a vectorization algorithm (e.g., Potrace)
        Err(ImportError::UnsupportedFormat(
            "Image vectorization not yet implemented".to_string(),
        ))
    }

    /// Import as raster reference (placeholder)
    pub fn import_as_reference<P: AsRef<Path>>(&self, _path: P) -> ImportResult<Document> {
        // This would import the image as a reference entity in the document
        Err(ImportError::UnsupportedFormat(
            "Image reference import not yet implemented".to_string(),
        ))
    }
}

/// General import dispatcher
pub struct Importer;

impl Importer {
    /// Import a file with automatic format detection
    pub fn import<P: AsRef<Path>>(path: P) -> ImportResult<Document> {
        let path = path.as_ref();
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "svg" => {
                let importer = SvgImporter::default();
                importer.import(path)
            }
            "png" | "jpg" | "jpeg" | "bmp" | "gif" => {
                let importer = ImageImporter::new(ImageImportSettings::default());
                importer.import(path)
            }
            _ => Err(ImportError::UnsupportedFormat(format!(
                "Unsupported import format: {}",
                ext
            ))),
        }
    }

    /// Get supported import formats
    pub fn supported_formats() -> Vec<(&'static str, &'static str)> {
        vec![
            ("svg", "Scalable Vector Graphics"),
            ("png", "Portable Network Graphics (requires vectorization)"),
            ("jpg", "JPEG Image (requires vectorization)"),
        ]
    }
}

/// Batch import utility
pub struct BatchImporter {
    /// Default settings
    svg_settings: SvgImportSettings,
}

impl BatchImporter {
    /// Create a new batch importer
    pub fn new() -> Self {
        Self {
            svg_settings: SvgImportSettings::default(),
        }
    }

    /// Import multiple files
    pub fn import_multiple<P: AsRef<Path>>(&self, paths: &[P]) -> Vec<ImportResult<Document>> {
        paths.iter().map(|p| Importer::import(p)).collect()
    }

    /// Merge multiple imported documents into one
    pub fn merge_imports<P: AsRef<Path>>(&self, paths: &[P]) -> ImportResult<Document> {
        let mut merged = Document::new();
        let mut layer_counter = 0;

        for path in paths {
            let mut doc = Importer::import(path)?;

            // Rename layers to avoid conflicts
            layer_counter += 1;
            let prefix = format!("import{}_", layer_counter);

            for entity in &mut doc.entities {
                if !entity.layer.starts_with(&prefix) {
                    entity.layer = format!("{}{}", prefix, entity.layer);
                }
            }

            // Merge entities
            for entity in doc.entities {
                merged.add_entity(entity);
            }

            // Merge layers
            for (name, layer) in doc.layers {
                let mut new_layer = layer;
                new_layer.name = format!("{}{}", prefix, name);
                merged.add_layer(new_layer);
            }
        }

        Ok(merged)
    }
}

impl Default for BatchImporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_import_line() {
        let svg = r#"<svg><line x1="0" y1="0" x2="100" y2="100" /></svg>"#;
        let importer = SvgImporter::default();
        let doc = importer.import_from_string(svg).unwrap();

        assert_eq!(doc.entities.len(), 1);
        match &doc.entities[0].geometry {
            GeometryType::Line(_) => {}
            _ => panic!("Expected line"),
        }
    }

    #[test]
    fn test_svg_import_circle() {
        let svg = r#"<svg><circle cx="50" cy="50" r="25" /></svg>"#;
        let importer = SvgImporter::default();
        let doc = importer.import_from_string(svg).unwrap();

        assert_eq!(doc.entities.len(), 1);
        match &doc.entities[0].geometry {
            GeometryType::Circle(c) => {
                assert_eq!(c.center.x, 50.0);
                assert_eq!(c.radius, 25.0);
            }
            _ => panic!("Expected circle"),
        }
    }

    #[test]
    fn test_svg_import_polygon() {
        let svg = r#"<svg><polygon points="0,0 100,0 100,100 0,100" /></svg>"#;
        let importer = SvgImporter::default();
        let doc = importer.import_from_string(svg).unwrap();

        assert_eq!(doc.entities.len(), 1);
        match &doc.entities[0].geometry {
            GeometryType::Polyline(p) => {
                assert_eq!(p.vertices.len(), 4);
                assert!(p.closed);
            }
            _ => panic!("Expected polyline"),
        }
    }
}
