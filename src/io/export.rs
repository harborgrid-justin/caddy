// CADDY - Enterprise CAD System
// File I/O System - Export Formats Module
// Agent 6 - File I/O System Developer

use crate::io::document::*;
use std::f64::consts::PI;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use thiserror::Error;

/// Export-related errors
#[derive(Error, Debug)]
pub enum ExportError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Unsupported entity type: {0}")]
    UnsupportedEntity(String),
    #[error("Invalid export settings: {0}")]
    InvalidSettings(String),
    #[error("Rendering error: {0}")]
    Rendering(String),
}

pub type ExportResult<T> = Result<T, ExportError>;

/// SVG export settings
#[derive(Debug, Clone)]
pub struct SvgExportSettings {
    /// Output width in pixels (or SVG units)
    pub width: f64,
    /// Output height in pixels (or SVG units)
    pub height: f64,
    /// Scale factor
    pub scale: f64,
    /// Background color
    pub background: Option<Color>,
    /// Stroke width
    pub stroke_width: f64,
    /// Include layer names as comments
    pub include_layer_comments: bool,
    /// View box (auto-calculate if None)
    pub view_box: Option<(f64, f64, f64, f64)>,
    /// Precision (decimal places)
    pub precision: usize,
}

impl Default for SvgExportSettings {
    fn default() -> Self {
        Self {
            width: 800.0,
            height: 600.0,
            scale: 1.0,
            background: Some(Color::white()),
            stroke_width: 1.0,
            include_layer_comments: true,
            view_box: None,
            precision: 3,
        }
    }
}

/// SVG exporter
pub struct SvgExporter {
    settings: SvgExportSettings,
}

impl SvgExporter {
    /// Create a new SVG exporter with settings
    pub fn new(settings: SvgExportSettings) -> Self {
        Self { settings }
    }

    /// Export a document to SVG
    pub fn export<P: AsRef<Path>>(&self, doc: &Document, path: P) -> ExportResult<()> {
        let mut file = File::create(path)?;
        self.export_to_writer(doc, &mut file)
    }

    /// Export to a writer
    pub fn export_to_writer<W: Write>(&self, doc: &Document, writer: &mut W) -> ExportResult<()> {
        // Calculate view box if not specified
        let view_box = if let Some(vb) = self.settings.view_box {
            vb
        } else if let Some(bbox) = doc.bounding_box() {
            let margin = 10.0;
            (
                bbox.min.x - margin,
                bbox.min.y - margin,
                (bbox.max.x - bbox.min.x) + 2.0 * margin,
                (bbox.max.y - bbox.min.y) + 2.0 * margin,
            )
        } else {
            (0.0, 0.0, self.settings.width, self.settings.height)
        };

        // Write SVG header
        writeln!(writer, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
        writeln!(
            writer,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" \
             width=\"{}\" height=\"{}\" \
             viewBox=\"{} {} {} {}\">",
            self.settings.width,
            self.settings.height,
            self.fmt(view_box.0),
            self.fmt(view_box.1),
            self.fmt(view_box.2),
            self.fmt(view_box.3)
        )?;

        // Add background if specified
        if let Some(bg) = self.settings.background {
            writeln!(
                writer,
                "  <rect width=\"100%\" height=\"100%\" fill=\"{}\"/>",
                self.color_to_svg(&bg)
            )?;
        }

        // Group entities by layer
        let mut layers_written = std::collections::HashSet::new();

        for layer_name in doc.layer_names() {
            let entities = doc.entities_on_layer(&layer_name);
            if entities.is_empty() {
                continue;
            }

            if self.settings.include_layer_comments {
                writeln!(writer, "  <!-- Layer: {} -->", layer_name)?;
            }

            writeln!(writer, "  <g id=\"layer-{}\">", self.escape_xml(&layer_name))?;

            for entity in entities {
                if entity.visible {
                    self.write_entity(writer, entity, doc)?;
                }
            }

            writeln!(writer, "  </g>")?;
            layers_written.insert(layer_name);
        }

        // Write closing tag
        writeln!(writer, "</svg>")?;

        Ok(())
    }

    fn write_entity<W: Write>(
        &self,
        writer: &mut W,
        entity: &Entity,
        doc: &Document,
    ) -> ExportResult<()> {
        let stroke = self.get_entity_color(entity, doc);
        let stroke_width = self.settings.stroke_width;

        match &entity.geometry {
            GeometryType::Point(p) => {
                writeln!(
                    writer,
                    "    <circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\" />",
                    self.fmt(p.position.x),
                    self.fmt(-p.position.y), // SVG Y is inverted
                    stroke_width,
                    self.color_to_svg(&stroke)
                )?;
            }
            GeometryType::Line(l) => {
                writeln!(
                    writer,
                    "    <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"{}\" />",
                    self.fmt(l.start.x),
                    self.fmt(-l.start.y),
                    self.fmt(l.end.x),
                    self.fmt(-l.end.y),
                    self.color_to_svg(&stroke),
                    stroke_width
                )?;
            }
            GeometryType::Circle(c) => {
                writeln!(
                    writer,
                    "    <circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" />",
                    self.fmt(c.center.x),
                    self.fmt(-c.center.y),
                    self.fmt(c.radius),
                    self.color_to_svg(&stroke),
                    stroke_width
                )?;
            }
            GeometryType::Arc(a) => {
                let path = self.arc_to_path(a);
                writeln!(
                    writer,
                    "    <path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" />",
                    path,
                    self.color_to_svg(&stroke),
                    stroke_width
                )?;
            }
            GeometryType::Ellipse(e) => {
                writeln!(
                    writer,
                    "    <ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" transform=\"rotate({} {} {})\" \
                     fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" />",
                    self.fmt(e.center.x),
                    self.fmt(-e.center.y),
                    self.fmt(e.major_axis),
                    self.fmt(e.minor_axis),
                    self.fmt(-e.rotation.to_degrees()),
                    self.fmt(e.center.x),
                    self.fmt(-e.center.y),
                    self.color_to_svg(&stroke),
                    stroke_width
                )?;
            }
            GeometryType::Polyline(p) => {
                let points: Vec<String> = p
                    .vertices
                    .iter()
                    .map(|v| format!("{},{}", self.fmt(v.position.x), self.fmt(-v.position.y)))
                    .collect();

                let poly_type = if p.closed { "polygon" } else { "polyline" };
                writeln!(
                    writer,
                    "    <{} points=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" />",
                    poly_type,
                    points.join(" "),
                    self.color_to_svg(&stroke),
                    stroke_width
                )?;
            }
            GeometryType::Spline(s) => {
                // Simplified spline rendering as polyline
                let path = self.spline_to_path(s);
                writeln!(
                    writer,
                    "    <path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" />",
                    path,
                    self.color_to_svg(&stroke),
                    stroke_width
                )?;
            }
            GeometryType::Text(t) => {
                writeln!(
                    writer,
                    "    <text x=\"{}\" y=\"{}\" font-size=\"{}\" fill=\"{}\" transform=\"rotate({} {} {})\">{}</text>",
                    self.fmt(t.position.x),
                    self.fmt(-t.position.y),
                    self.fmt(t.height),
                    self.color_to_svg(&stroke),
                    self.fmt(-t.rotation.to_degrees()),
                    self.fmt(t.position.x),
                    self.fmt(-t.position.y),
                    self.escape_xml(&t.text)
                )?;
            }
            GeometryType::MText(t) => {
                // Basic MText rendering
                writeln!(
                    writer,
                    "    <text x=\"{}\" y=\"{}\" font-size=\"{}\" fill=\"{}\">{}</text>",
                    self.fmt(t.position.x),
                    self.fmt(-t.position.y),
                    self.fmt(t.height),
                    self.color_to_svg(&stroke),
                    self.escape_xml(&t.text)
                )?;
            }
            _ => {
                // Unsupported entity types are silently skipped
            }
        }

        Ok(())
    }

    fn arc_to_path(&self, arc: &Arc) -> String {
        let start_x = arc.center.x + arc.radius * arc.start_angle.cos();
        let start_y = arc.center.y + arc.radius * arc.start_angle.sin();
        let end_x = arc.center.x + arc.radius * arc.end_angle.cos();
        let end_y = arc.center.y + arc.radius * arc.end_angle.sin();

        let mut angle_diff = arc.end_angle - arc.start_angle;
        while angle_diff < 0.0 {
            angle_diff += 2.0 * PI;
        }

        let large_arc = if angle_diff > PI { 1 } else { 0 };

        format!(
            "M {} {} A {} {} 0 {} 1 {} {}",
            self.fmt(start_x),
            self.fmt(-start_y),
            self.fmt(arc.radius),
            self.fmt(arc.radius),
            large_arc,
            self.fmt(end_x),
            self.fmt(-end_y)
        )
    }

    fn spline_to_path(&self, spline: &Spline) -> String {
        if spline.control_points.is_empty() {
            return String::new();
        }

        let mut path = format!(
            "M {} {}",
            self.fmt(spline.control_points[0].x),
            self.fmt(-spline.control_points[0].y)
        );

        // Simplified: render as polyline through control points
        for point in &spline.control_points[1..] {
            path.push_str(&format!(" L {} {}", self.fmt(point.x), self.fmt(-point.y)));
        }

        path
    }

    fn get_entity_color(&self, entity: &Entity, doc: &Document) -> Color {
        entity.color.unwrap_or_else(|| {
            doc.get_layer(&entity.layer)
                .map(|l| l.color)
                .unwrap_or(Color::white())
        })
    }

    fn color_to_svg(&self, color: &Color) -> String {
        format!("rgb({},{},{})", color.r, color.g, color.b)
    }

    fn fmt(&self, value: f64) -> String {
        format!("{:.prec$}", value, prec = self.settings.precision)
    }

    fn escape_xml(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
}

impl Default for SvgExporter {
    fn default() -> Self {
        Self::new(SvgExportSettings::default())
    }
}

/// PDF export settings
#[derive(Debug, Clone)]
pub struct PdfExportSettings {
    /// Page size
    pub page_size: PaperSize,
    /// Scale
    pub scale: f64,
    /// Margins (left, top, right, bottom) in mm
    pub margins: (f64, f64, f64, f64),
    /// Line width
    pub line_width: f64,
    /// Include metadata
    pub include_metadata: bool,
}

impl Default for PdfExportSettings {
    fn default() -> Self {
        Self {
            page_size: PaperSize::A4,
            scale: 1.0,
            margins: (10.0, 10.0, 10.0, 10.0),
            line_width: 0.5,
            include_metadata: true,
        }
    }
}

/// PDF exporter (basic implementation)
/// Note: A full implementation would use a library like printpdf
pub struct PdfExporter {
    settings: PdfExportSettings,
}

impl PdfExporter {
    /// Create a new PDF exporter
    pub fn new(settings: PdfExportSettings) -> Self {
        Self { settings }
    }

    /// Export to PDF (placeholder - requires printpdf or similar)
    pub fn export<P: AsRef<Path>>(&self, _doc: &Document, _path: P) -> ExportResult<()> {
        // This would require adding a PDF library dependency
        // For now, return an error indicating it's not implemented
        Err(ExportError::Rendering(
            "PDF export requires additional dependencies (printpdf)".to_string(),
        ))
    }
}

/// Raster export settings
#[derive(Debug, Clone)]
pub struct RasterExportSettings {
    /// Image width in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
    /// Scale factor
    pub scale: f64,
    /// Background color
    pub background: Color,
    /// Anti-aliasing samples (1 = none, higher = more)
    pub anti_aliasing: u8,
    /// DPI (for print quality)
    pub dpi: u32,
}

impl Default for RasterExportSettings {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            scale: 1.0,
            background: Color::white(),
            anti_aliasing: 4,
            dpi: 96,
        }
    }
}

/// Raster image exporter
/// Note: Actual rendering would require a graphics library
pub struct RasterExporter {
    settings: RasterExportSettings,
}

impl RasterExporter {
    /// Create a new raster exporter
    pub fn new(settings: RasterExportSettings) -> Self {
        Self { settings }
    }

    /// Export to PNG
    pub fn export_png<P: AsRef<Path>>(&self, _doc: &Document, _path: P) -> ExportResult<()> {
        // This would require implementing actual rasterization
        // Using the image crate or similar
        Err(ExportError::Rendering(
            "PNG export requires rendering implementation".to_string(),
        ))
    }

    /// Export to JPEG
    pub fn export_jpeg<P: AsRef<Path>>(
        &self,
        _doc: &Document,
        _path: P,
        _quality: u8,
    ) -> ExportResult<()> {
        Err(ExportError::Rendering(
            "JPEG export requires rendering implementation".to_string(),
        ))
    }
}

/// Export format detector and dispatcher
pub struct Exporter;

impl Exporter {
    /// Export a document to the appropriate format based on file extension
    pub fn export<P: AsRef<Path>>(doc: &Document, path: P) -> ExportResult<()> {
        let path = path.as_ref();
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "svg" => {
                let exporter = SvgExporter::default();
                exporter.export(doc, path)
            }
            "pdf" => {
                let exporter = PdfExporter::new(PdfExportSettings::default());
                exporter.export(doc, path)
            }
            "png" => {
                let exporter = RasterExporter::new(RasterExportSettings::default());
                exporter.export_png(doc, path)
            }
            "jpg" | "jpeg" => {
                let exporter = RasterExporter::new(RasterExportSettings::default());
                exporter.export_jpeg(doc, path, 90)
            }
            _ => Err(ExportError::InvalidSettings(format!(
                "Unsupported export format: {}",
                ext
            ))),
        }
    }

    /// Get supported export formats
    pub fn supported_formats() -> Vec<(&'static str, &'static str)> {
        vec![
            ("svg", "Scalable Vector Graphics"),
            ("pdf", "Portable Document Format"),
            ("png", "Portable Network Graphics"),
            ("jpg", "JPEG Image"),
        ]
    }
}

/// Batch export utility
pub struct BatchExporter {
    /// Output directory
    output_dir: std::path::PathBuf,
}

impl BatchExporter {
    /// Create a new batch exporter
    pub fn new<P: AsRef<Path>>(output_dir: P) -> Self {
        Self {
            output_dir: output_dir.as_ref().to_path_buf(),
        }
    }

    /// Export a document to multiple formats
    pub fn export_all(
        &self,
        doc: &Document,
        base_name: &str,
        formats: &[&str],
    ) -> Vec<ExportResult<()>> {
        let mut results = Vec::new();

        for format in formats {
            let mut path = self.output_dir.clone();
            path.push(format!("{}.{}", base_name, format));

            results.push(Exporter::export(doc, &path));
        }

        results
    }

    /// Export with different settings for each format
    pub fn export_with_settings(
        &self,
        doc: &Document,
        base_name: &str,
        svg_settings: Option<SvgExportSettings>,
        pdf_settings: Option<PdfExportSettings>,
        raster_settings: Option<RasterExportSettings>,
    ) -> ExportResult<()> {
        // Export to SVG if settings provided
        if let Some(settings) = svg_settings {
            let mut path = self.output_dir.clone();
            path.push(format!("{}.svg", base_name));
            SvgExporter::new(settings).export(doc, &path)?;
        }

        // Export to PDF if settings provided
        if let Some(settings) = pdf_settings {
            let mut path = self.output_dir.clone();
            path.push(format!("{}.pdf", base_name));
            PdfExporter::new(settings).export(doc, &path)?;
        }

        // Export to PNG if settings provided
        if let Some(settings) = raster_settings {
            let mut path = self.output_dir.clone();
            path.push(format!("{}.png", base_name));
            RasterExporter::new(settings).export_png(doc, &path)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_export() {
        let mut doc = Document::new();
        doc.add_entity(Entity::new(
            GeometryType::Line(Line {
                start: Vec3::new(0.0, 0.0, 0.0),
                end: Vec3::new(100.0, 100.0, 0.0),
            }),
            "0".to_string(),
        ));

        let exporter = SvgExporter::default();
        let mut buffer = Vec::new();
        exporter.export_to_writer(&doc, &mut buffer).unwrap();

        let svg = String::from_utf8(buffer).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("<line"));
    }

    #[test]
    fn test_color_to_svg() {
        let exporter = SvgExporter::default();
        let color = Color::new(255, 128, 64);
        assert_eq!(exporter.color_to_svg(&color), "rgb(255,128,64)");
    }

    #[test]
    fn test_xml_escape() {
        let exporter = SvgExporter::default();
        assert_eq!(exporter.escape_xml("A & B"), "A &amp; B");
        assert_eq!(exporter.escape_xml("<tag>"), "&lt;tag&gt;");
    }
}
