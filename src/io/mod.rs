// CADDY - Enterprise CAD System
// File I/O System - Module Exports
// Agent 6 - File I/O System Developer

//! # CADDY File I/O System
//!
//! This module provides comprehensive file input/output functionality for CADDY,
//! including support for:
//!
//! - **Native formats**: Binary (.cdy) and JSON (.cdyj) formats with compression
//! - **DXF support**: Full DXF R12 through R2018 compatibility for AutoCAD interoperability
//! - **Export formats**: SVG, PDF, PNG, JPEG for presentations and sharing
//! - **Import formats**: SVG and image vectorization
//! - **Unit handling**: Comprehensive unit conversion and formatting
//!
//! ## Quick Start
//!
//! ### Loading a file
//! ```no_run
//! use caddy::io::native::FormatDetector;
//!
//! let _doc = FormatDetector::load("drawing.cdy").unwrap();
//! ```
//!
//! ### Saving to native format
//! ```no_run
//! use caddy::io::document::Document;
//! use caddy::io::native::NativeFormat;
//!
//! let _doc = Document::new();
//! let format = NativeFormat::new();
//! format.save(&doc, "drawing.cdy").unwrap();
//! ```
//!
//! ### DXF Import/Export
//! ```no_run
//! use caddy::io::dxf::{DxfReader, DxfWriter, DxfVersion};
//!
//! // Import DXF
//! let _reader = DxfReader::new();
//! let _doc = reader.read_file("drawing.dxf").unwrap();
//!
//! // Export DXF
//! let _writer = DxfWriter::new(DxfVersion::R2018);
//! writer.write_file(&doc, "output.dxf").unwrap();
//! ```
//!
//! ### SVG Export
//! ```no_run
//! use caddy::io::export::{SvgExporter, SvgExportSettings};
//! use caddy::io::document::Document;
//!
//! let _doc = Document::new();
//! let settings = SvgExportSettings::default();
//! let exporter = SvgExporter::new(settings);
//! exporter.export(&doc, "output.svg").unwrap();
//! ```

pub mod document;
pub mod units;
pub mod dxf;
pub mod dwg;
pub mod step;
pub mod iges;
pub mod stl;
pub mod obj;
pub mod gltf;
pub mod native;
pub mod export;
pub mod import;
pub mod batch;
pub mod validation;

// Re-export commonly used types
pub use document::{
    Document, DocumentMetadata, DocumentSettings, Entity, GeometryType,
    Layer, Block, View, Color, LineType, LineWeight, Vec3, BoundingBox,
    // Geometry types
    Point, Line, Circle, Arc, Ellipse, Polyline, Spline,
    Text, MText, Dimension, Insert, Hatch,
    // Settings
    PaperSize, GridSettings, SnapSettings,
};

pub use units::{Unit, UnitConverter, PrecisionSettings};

pub use dxf::{DxfReader, DxfWriter, DxfVersion, DxfError, DxfResult};

pub use native::{
    NativeFormat, JsonFormat, FormatDetector, FileFormat as NativeFileFormat,
    BackupManager, NativeError, NativeResult,
};

pub use export::{
    SvgExporter, SvgExportSettings,
    PdfExporter, PdfExportSettings,
    RasterExporter, RasterExportSettings,
    Exporter, BatchExporter,
    ExportError, ExportResult,
};

pub use import::{
    SvgImporter, SvgImportSettings,
    ImageImporter, ImageImportSettings,
    Importer, BatchImporter,
    ImportError, ImportResult,
};

pub use dwg::{DwgReader, DwgWriter, DwgVersion, DwgError, DwgResult};

pub use step::{StepReader, StepWriter, ApplicationProtocol, StepError, StepResult};

pub use iges::{IgesReader, IgesWriter, IgesError, IgesResult};

pub use stl::{StlReader, StlWriter, StlMesh, StlTriangle, StlError, StlResult};

pub use obj::{ObjReader, ObjWriter, ObjMesh, ObjMaterial, ObjError, ObjResult};

pub use gltf::{GltfReader, GltfWriter, Gltf, GltfError, GltfResult};

pub use batch::{
    BatchConverter, BatchJob, BatchError, BatchResult, BatchStats,
    ConversionResult, FileFormat,
};

pub use validation::{
    Validator, Repairer, ValidationError, ValidationIssue, ValidationResult,
    Severity, ValidationReport,
};

/// Prelude module for convenient imports
pub mod prelude {
    pub use super::document::{Document, Entity, GeometryType, Layer, Color};
    pub use super::units::Unit;
    pub use super::dxf::{DxfReader, DxfWriter, DxfVersion};
    pub use super::native::{NativeFormat, JsonFormat, FormatDetector};
    pub use super::export::{SvgExporter, Exporter};
    pub use super::import::{SvgImporter, Importer};
}

/// I/O module version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get information about all supported file formats
pub fn supported_formats() -> FileFormatInfo {
    FileFormatInfo::default()
}

/// Information about supported file formats
#[derive(Debug, Clone)]
pub struct FileFormatInfo {
    /// Formats that can be opened
    pub readable: Vec<FormatEntry>,
    /// Formats that can be saved
    pub writable: Vec<FormatEntry>,
}

impl FileFormatInfo {
    /// Get file dialog filters for opening files
    pub fn open_filters(&self) -> Vec<String> {
        let mut filters = vec!["All Supported (*.cdy *.cdyj *.dxf *.svg)".to_string()];
        filters.extend(self.readable.iter().map(|f| f.filter_string()));
        filters
    }

    /// Get file dialog filters for saving files
    pub fn save_filters(&self) -> Vec<String> {
        self.writable.iter().map(|f| f.filter_string()).collect()
    }
}

impl Default for FileFormatInfo {
    fn default() -> Self {
        Self {
            readable: vec![
                FormatEntry {
                    name: "CADDY Binary".to_string(),
                    extension: "cdy".to_string(),
                    description: "CADDY native binary format with compression".to_string(),
                },
                FormatEntry {
                    name: "CADDY JSON".to_string(),
                    extension: "cdyj".to_string(),
                    description: "CADDY JSON format for debugging and interchange".to_string(),
                },
                FormatEntry {
                    name: "AutoCAD DXF".to_string(),
                    extension: "dxf".to_string(),
                    description: "AutoCAD Drawing Exchange Format (R12-R2018)".to_string(),
                },
                FormatEntry {
                    name: "AutoCAD DWG".to_string(),
                    extension: "dwg".to_string(),
                    description: "AutoCAD Drawing format (R14-R2021)".to_string(),
                },
                FormatEntry {
                    name: "STEP".to_string(),
                    extension: "step".to_string(),
                    description: "STEP/AP214 3D solid models (ISO 10303)".to_string(),
                },
                FormatEntry {
                    name: "IGES".to_string(),
                    extension: "iges".to_string(),
                    description: "IGES surface geometry (ANSI/US PRO/IPO-100)".to_string(),
                },
                FormatEntry {
                    name: "STL".to_string(),
                    extension: "stl".to_string(),
                    description: "STereoLithography format for 3D printing".to_string(),
                },
                FormatEntry {
                    name: "Wavefront OBJ".to_string(),
                    extension: "obj".to_string(),
                    description: "Wavefront OBJ 3D geometry with materials".to_string(),
                },
                FormatEntry {
                    name: "glTF 2.0".to_string(),
                    extension: "gltf".to_string(),
                    description: "GL Transmission Format for web/AR/VR".to_string(),
                },
                FormatEntry {
                    name: "SVG".to_string(),
                    extension: "svg".to_string(),
                    description: "Scalable Vector Graphics".to_string(),
                },
            ],
            writable: vec![
                FormatEntry {
                    name: "CADDY Binary".to_string(),
                    extension: "cdy".to_string(),
                    description: "CADDY native binary format with compression".to_string(),
                },
                FormatEntry {
                    name: "CADDY JSON".to_string(),
                    extension: "cdyj".to_string(),
                    description: "CADDY JSON format for debugging and interchange".to_string(),
                },
                FormatEntry {
                    name: "AutoCAD DXF".to_string(),
                    extension: "dxf".to_string(),
                    description: "AutoCAD Drawing Exchange Format (R12-R2018)".to_string(),
                },
                FormatEntry {
                    name: "AutoCAD DWG".to_string(),
                    extension: "dwg".to_string(),
                    description: "AutoCAD Drawing format (R14-R2021)".to_string(),
                },
                FormatEntry {
                    name: "STEP".to_string(),
                    extension: "step".to_string(),
                    description: "STEP/AP214 3D solid models (ISO 10303)".to_string(),
                },
                FormatEntry {
                    name: "IGES".to_string(),
                    extension: "iges".to_string(),
                    description: "IGES surface geometry (ANSI/US PRO/IPO-100)".to_string(),
                },
                FormatEntry {
                    name: "STL Binary".to_string(),
                    extension: "stl".to_string(),
                    description: "STereoLithography format for 3D printing".to_string(),
                },
                FormatEntry {
                    name: "Wavefront OBJ".to_string(),
                    extension: "obj".to_string(),
                    description: "Wavefront OBJ 3D geometry with materials".to_string(),
                },
                FormatEntry {
                    name: "glTF 2.0".to_string(),
                    extension: "gltf".to_string(),
                    description: "GL Transmission Format for web/AR/VR".to_string(),
                },
                FormatEntry {
                    name: "SVG".to_string(),
                    extension: "svg".to_string(),
                    description: "Scalable Vector Graphics".to_string(),
                },
                FormatEntry {
                    name: "PDF".to_string(),
                    extension: "pdf".to_string(),
                    description: "Portable Document Format with layers".to_string(),
                },
                FormatEntry {
                    name: "PNG".to_string(),
                    extension: "png".to_string(),
                    description: "Portable Network Graphics (raster export)".to_string(),
                },
            ],
        }
    }
}

/// Single format entry
#[derive(Debug, Clone)]
pub struct FormatEntry {
    /// Display name
    pub name: String,
    /// File extension (without dot)
    pub extension: String,
    /// Format description
    pub description: String,
}

impl FormatEntry {
    /// Get file dialog filter string
    pub fn filter_string(&self) -> String {
        format!("{} (*.{})", self.name, self.extension)
    }
}

/// Progress callback type for long-running operations
pub type ProgressCallback = Box<dyn Fn(usize, usize)>;

/// File operation statistics
#[derive(Debug, Clone, Default)]
pub struct FileStats {
    /// File size in bytes
    pub file_size: u64,
    /// Number of entities
    pub entity_count: usize,
    /// Number of layers
    pub layer_count: usize,
    /// Number of blocks
    pub block_count: usize,
    /// Time to load/save in milliseconds
    pub operation_time_ms: u64,
}

impl FileStats {
    /// Create file stats from a document
    pub fn from_document(doc: &Document) -> Self {
        Self {
            file_size: 0, // Would be set during actual file operations
            entity_count: doc.entities.len(),
            layer_count: doc.layers.len(),
            block_count: doc.blocks.len(),
            operation_time_ms: 0,
        }
    }

    /// Create stats from a file path
    pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<Self> {
        let metadata = std::fs::metadata(path)?;
        Ok(Self {
            file_size: metadata.len(),
            entity_count: 0,
            layer_count: 0,
            block_count: 0,
            operation_time_ms: 0,
        })
    }

    /// Format file size as human-readable string
    pub fn format_file_size(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if self.file_size >= GB {
            format!("{:.2} GB", self.file_size as f64 / GB as f64)
        } else if self.file_size >= MB {
            format!("{:.2} MB", self.file_size as f64 / MB as f64)
        } else if self.file_size >= KB {
            format!("{:.2} KB", self.file_size as f64 / KB as f64)
        } else {
            format!("{} bytes", self.file_size)
        }
    }

    /// Format operation time as human-readable string
    pub fn format_operation_time(&self) -> String {
        if self.operation_time_ms >= 1000 {
            format!("{:.2} s", self.operation_time_ms as f64 / 1000.0)
        } else {
            format!("{} ms", self.operation_time_ms)
        }
    }
}

/// File operation result with statistics
pub struct FileOperationResult {
    /// The loaded document (for load operations)
    pub document: Option<Document>,
    /// Operation statistics
    pub stats: FileStats,
    /// Any warnings generated during the operation
    pub warnings: Vec<String>,
}

impl FileOperationResult {
    /// Create a successful result
    pub fn success(doc: Document, stats: FileStats) -> Self {
        Self {
            document: Some(doc),
            stats,
            warnings: Vec::new(),
        }
    }

    /// Add a warning
    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_stats_format() {
        let stats = FileStats {
            file_size: 1024 * 1024 * 5, // 5 MB
            entity_count: 100,
            layer_count: 5,
            block_count: 2,
            operation_time_ms: 1500,
        };

        assert!(stats.format_file_size().contains("MB"));
        assert!(stats.format_operation_time().contains("s"));
    }

    #[test]
    fn test_format_info() {
        let info = supported_formats();
        assert!(!info.readable.is_empty());
        assert!(!info.writable.is_empty());
        assert!(!info.open_filters().is_empty());
        assert!(!info.save_filters().is_empty());
    }

    #[test]
    fn test_format_entry() {
        let _entry = FormatEntry {
            name: "Test Format".to_string(),
            extension: "test".to_string(),
            description: "Test description".to_string(),
        };

        assert_eq!(entry.filter_string(), "Test Format (*.test)");
    }
}
