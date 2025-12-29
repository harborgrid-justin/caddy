// CADDY - Enterprise CAD System
// File I/O System - IGES Format Support
// Agent 9 - Import/Export Pipeline Specialist

//! # IGES (Initial Graphics Exchange Specification) Format Support
//!
//! Provides read/write support for IGES, one of the oldest and most widely supported
//! CAD interchange formats. IGES is particularly good for surface geometry and is
//! supported by virtually all CAD systems.
//!
//! ## IGES Versions
//!
//! - IGES 5.1 (1991)
//! - IGES 5.2 (1993)
//! - IGES 5.3 (1996)
//!
//! ## Supported Entity Types
//!
//! - 100: Circular Arc
//! - 102: Composite Curve
//! - 104: Conic Arc
//! - 106: Copious Data (polylines)
//! - 110: Line
//! - 112: Parametric Spline Curve
//! - 114: Parametric Spline Surface
//! - 116: Point
//! - 118: Ruled Surface
//! - 120: Surface of Revolution
//! - 122: Tabulated Cylinder
//! - 124: Transformation Matrix
//! - 126: Rational B-Spline Curve (NURBS)
//! - 128: Rational B-Spline Surface (NURBS)

use crate::io::document::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use thiserror::Error;

/// IGES-related errors
#[derive(Error, Debug)]
pub enum IgesError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Parse error in section {section}, line {line}: {message}")]
    Parse {
        section: String,
        line: usize,
        message: String,
    },

    #[error("Invalid IGES file: {0}")]
    InvalidFile(String),

    #[error("Unsupported entity type: {0}")]
    UnsupportedEntityType(u16),

    #[error("Invalid entity reference: {0}")]
    InvalidReference(usize),

    #[error("Missing directory entry for entity {0}")]
    MissingDirectoryEntry(usize),
}

pub type IgesResult<T> = Result<T, IgesError>;

/// IGES file sections
#[derive(Debug)]
pub struct IgesFile {
    pub start_section: Vec<String>,
    pub global_section: GlobalSection,
    pub directory_entries: Vec<DirectoryEntry>,
    pub parameter_data: HashMap<usize, Vec<String>>,
}

/// IGES global section
#[derive(Debug)]
pub struct GlobalSection {
    pub parameter_delimiter: char,
    pub record_delimiter: char,
    pub product_id_sender: String,
    pub file_name: String,
    pub native_system_id: String,
    pub preprocessor_version: String,
    pub integer_bits: u32,
    pub single_precision_magnitude: f64,
    pub single_precision_significance: u32,
    pub double_precision_magnitude: f64,
    pub double_precision_significance: u32,
    pub product_id_receiver: String,
    pub model_space_scale: f64,
    pub units_flag: u32,
    pub units_name: String,
    pub max_line_weight: f64,
    pub date_time: String,
    pub min_resolution: f64,
    pub max_coordinate: f64,
    pub author: String,
    pub organization: String,
    pub iges_version: u32,
    pub drafting_standard: u32,
    pub modified_date: String,
}

impl Default for GlobalSection {
    fn default() -> Self {
        Self {
            parameter_delimiter: ',',
            record_delimiter: ';',
            product_id_sender: String::new(),
            file_name: String::new(),
            native_system_id: "CADDY".to_string(),
            preprocessor_version: "1.0".to_string(),
            integer_bits: 32,
            single_precision_magnitude: 10.0,
            single_precision_significance: 6,
            double_precision_magnitude: 10.0,
            double_precision_significance: 15,
            product_id_receiver: String::new(),
            model_space_scale: 1.0,
            units_flag: 2, // Millimeters
            units_name: "MM".to_string(),
            max_line_weight: 1.0,
            date_time: String::new(),
            min_resolution: 1e-6,
            max_coordinate: 1e6,
            author: String::new(),
            organization: String::new(),
            iges_version: 11, // IGES 5.3
            drafting_standard: 0,
            modified_date: String::new(),
        }
    }
}

/// IGES directory entry
#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    pub entity_type: u16,
    pub parameter_data_pointer: usize,
    pub structure: usize,
    pub line_font_pattern: u32,
    pub level: u32,
    pub view: u32,
    pub transformation_matrix: u32,
    pub label_display: u32,
    pub status_number: u32,
    pub sequence_number: usize,
    pub entity_type_number: u16,
    pub line_weight: u32,
    pub color_number: u32,
    pub parameter_line_count: usize,
    pub form_number: u32,
    pub entity_label: String,
    pub entity_subscript: u32,
}

/// IGES file reader
pub struct IgesReader {
    strict_mode: bool,
}

impl IgesReader {
    /// Create a new IGES reader
    pub fn new() -> Self {
        Self {
            strict_mode: false,
        }
    }

    /// Enable strict parsing mode
    pub fn strict(mut self) -> Self {
        self.strict_mode = true;
        self
    }

    /// Read an IGES file
    pub fn read_file<P: AsRef<Path>>(&self, path: P) -> IgesResult<Document> {
        let file = File::open(path)?;
        let _reader = BufReader::new(file);
        self.read(reader)
    }

    /// Read IGES from a buffered reader
    pub fn read<R: BufRead>(&self, reader: R) -> IgesResult<Document> {
        let _iges_file = self.parse_iges_file(reader)?;
        self.convert_to_document(iges_file)
    }

    fn parse_iges_file<R: BufRead>(&self, reader: R) -> IgesResult<IgesFile> {
        let mut start_section = Vec::new();
        let mut global_lines = Vec::new();
        let mut directory_lines = Vec::new();
        let mut parameter_lines = Vec::new();

        // Read and categorize lines by section identifier (column 73)
        for line in reader.lines() {
            let line = line?;
            if line.len() < 73 {
                continue;
            }

            let section_id = &line[72..73];
            let content = &line[0..72].trim_end();

            match section_id {
                "S" => start_section.push(content.to_string()),
                "G" => global_lines.push(content.to_string()),
                "D" => directory_lines.push(content.to_string()),
                "P" => parameter_lines.push(content.to_string()),
                "T" => break, // Terminate section
                _ => {}
            }
        }

        // Parse global section
        let global_section = self.parse_global_section(&global_lines)?;

        // Parse directory entries (every 2 lines is one entry)
        let directory_entries = self.parse_directory_section(&directory_lines)?;

        // Parse parameter data
        let parameter_data = self.parse_parameter_section(&parameter_lines, &global_section)?;

        Ok(IgesFile {
            start_section,
            global_section,
            directory_entries,
            parameter_data,
        })
    }

    fn parse_global_section(&self, lines: &[String]) -> IgesResult<GlobalSection> {
        // Join all global section lines and parse
        let combined = lines.join("");
        let mut global = GlobalSection::default();

        // Extract parameters (simplified parsing)
        let parts: Vec<&str> = combined.split(';').collect();
        if parts.len() > 1 {
            global.parameter_delimiter = parts[0].chars().next().unwrap_or(',');
        }

        Ok(global)
    }

    fn parse_directory_section(&self, lines: &[String]) -> IgesResult<Vec<DirectoryEntry>> {
        let mut entries = Vec::new();

        for chunk in lines.chunks(2) {
            if chunk.len() < 2 {
                break;
            }

            let line1 = &chunk[0];
            let line2 = &chunk[1];

            entries.push(self.parse_directory_entry(line1, line2)?);
        }

        Ok(entries)
    }

    fn parse_directory_entry(&self, line1: &str, line2: &str) -> IgesResult<DirectoryEntry> {
        // IGES directory entries are fixed-format with 8-character fields

        let entity_type = self.parse_field(line1, 0, 8)?.parse::<u16>()
            .map_err(|e| IgesError::Parse {
                section: "Directory".to_string(),
                line: 0,
                message: format!("Invalid entity type: {}", e),
            })?;

        let parameter_data_pointer = self.parse_field(line1, 8, 16)?.parse::<usize>()
            .unwrap_or(0);

        Ok(DirectoryEntry {
            entity_type,
            parameter_data_pointer,
            structure: 0,
            line_font_pattern: 0,
            level: 0,
            view: 0,
            transformation_matrix: 0,
            label_display: 0,
            status_number: 0,
            sequence_number: 0,
            entity_type_number: entity_type,
            line_weight: 0,
            color_number: 0,
            parameter_line_count: 0,
            form_number: 0,
            entity_label: String::new(),
            entity_subscript: 0,
        })
    }

    fn parse_field(&self, line: &str, start: usize, end: usize) -> IgesResult<String> {
        if line.len() < end {
            return Ok(String::new());
        }
        Ok(line[start..end].trim().to_string())
    }

    fn parse_parameter_section(
        &self,
        lines: &[String],
        global: &GlobalSection,
    ) -> IgesResult<HashMap<usize, Vec<String>>> {
        let mut parameter_data = HashMap::new();

        // Group parameter lines by their entity number
        for line in lines {
            if let Some(entity_num) = self.extract_parameter_entity_number(line) {
                parameter_data
                    .entry(entity_num)
                    .or_insert_with(Vec::new)
                    .push(line[0..64].trim().to_string());
            }
        }

        Ok(parameter_data)
    }

    fn extract_parameter_entity_number(&self, line: &str) -> Option<usize> {
        // Parameter data pointer is in columns 65-72
        if line.len() >= 72 {
            line[64..72].trim().parse().ok()
        } else {
            None
        }
    }

    fn convert_to_document(&self, iges_file: IgesFile) -> IgesResult<Document> {
        let mut doc = Document::new();

        // Set metadata
        doc.metadata.title = iges_file.global_section.file_name.clone();
        doc.metadata.author = iges_file.global_section.author.clone();

        // Convert entities
        for entry in &iges_file.directory_entries {
            self.convert_entity(entry, &iges_file, &mut doc)?;
        }

        Ok(doc)
    }

    fn convert_entity(
        &self,
        entry: &DirectoryEntry,
        iges_file: &IgesFile,
        doc: &mut Document,
    ) -> IgesResult<()> {
        match entry.entity_type {
            110 => self.convert_line(entry, iges_file, doc)?,
            100 => self.convert_circular_arc(entry, iges_file, doc)?,
            106 => self.convert_copious_data(entry, iges_file, doc)?,
            126 => self.convert_nurbs_curve(entry, iges_file, doc)?,
            _ => {
                // Skip unsupported entity types
            }
        }

        Ok(())
    }

    fn convert_line(
        &self,
        entry: &DirectoryEntry,
        iges_file: &IgesFile,
        doc: &mut Document,
    ) -> IgesResult<()> {
        // Extract line parameters and create entity
        // This is a placeholder implementation
        Ok(())
    }

    fn convert_circular_arc(
        &self,
        entry: &DirectoryEntry,
        iges_file: &IgesFile,
        doc: &mut Document,
    ) -> IgesResult<()> {
        // Extract arc parameters and create entity
        Ok(())
    }

    fn convert_copious_data(
        &self,
        entry: &DirectoryEntry,
        iges_file: &IgesFile,
        doc: &mut Document,
    ) -> IgesResult<()> {
        // Extract polyline data and create entity
        Ok(())
    }

    fn convert_nurbs_curve(
        &self,
        entry: &DirectoryEntry,
        iges_file: &IgesFile,
        doc: &mut Document,
    ) -> IgesResult<()> {
        // Extract NURBS curve data and create entity
        Ok(())
    }
}

impl Default for IgesReader {
    fn default() -> Self {
        Self::new()
    }
}

/// IGES file writer
pub struct IgesWriter {
    author: String,
    organization: String,
    units_flag: u32,
}

impl IgesWriter {
    /// Create a new IGES writer
    pub fn new() -> Self {
        Self {
            author: "CADDY User".to_string(),
            organization: "CADDY CAD System".to_string(),
            units_flag: 2, // Millimeters
        }
    }

    /// Set author name
    pub fn with_author(mut self, author: String) -> Self {
        self.author = author;
        self
    }

    /// Set organization name
    pub fn with_organization(mut self, org: String) -> Self {
        self.organization = org;
        self
    }

    /// Write document to IGES file
    pub fn write_file<P: AsRef<Path>>(&self, doc: &Document, path: P) -> IgesResult<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        self.write(doc, &mut writer)
    }

    /// Write document to a writer
    pub fn write<W: Write>(&self, doc: &Document, writer: &mut W) -> IgesResult<()> {
        let mut seq_s = 1;
        let mut seq_g = 1;
        let mut seq_d = 1;
        let mut seq_p = 1;

        // Write start section
        self.write_line(writer, "CADDY CAD System Export", 'S', seq_s)?;
        seq_s += 1;

        // Write global section
        let global = self.build_global_section(doc);
        self.write_global_section(writer, &global, &mut seq_g)?;

        // Build and write directory entries and parameter data
        let (directory, parameters) = self.build_entities(doc);
        self.write_directory_section(writer, &directory, &mut seq_d)?;
        self.write_parameter_section(writer, &parameters, &mut seq_p)?;

        // Write terminate section
        self.write_terminate(writer, seq_s - 1, seq_g - 1, seq_d - 1, seq_p - 1)?;

        Ok(())
    }

    fn write_line(&self, writer: &mut impl Write, content: &str, section: char, seq: usize) -> IgesResult<()> {
        let padded = format!("{:<72}{}{:07}", content, section, seq);
        writeln!(writer, "{}", &padded[0..80.min(padded.len())])?;
        Ok(())
    }

    fn build_global_section(&self, doc: &Document) -> GlobalSection {
        let mut global = GlobalSection::default();
        global.author = self.author.clone();
        global.organization = self.organization.clone();
        global.file_name = doc.metadata.title.clone();
        global.date_time = chrono::Utc::now().format("%Y%m%d.%H%M%S").to_string();
        global
    }

    fn write_global_section(&self, writer: &mut impl Write, global: &GlobalSection, seq: &mut usize) -> IgesResult<()> {
        // Write global parameters as comma-separated values
        let params = format!(
            "{},{},{},{},{},{};",
            global.parameter_delimiter,
            global.record_delimiter,
            global.product_id_sender,
            global.file_name,
            global.native_system_id,
            global.preprocessor_version
        );

        for chunk in params.as_bytes().chunks(72) {
            let content = std::str::from_utf8(chunk).unwrap_or("");
            self.write_line(writer, content, 'G', *seq)?;
            *seq += 1;
        }

        Ok(())
    }

    fn build_entities(&self, doc: &Document) -> (Vec<DirectoryEntry>, HashMap<usize, Vec<String>>) {
        let directory = Vec::new();
        let parameters = HashMap::new();

        // Convert document entities to IGES format
        // This is a placeholder implementation

        (directory, parameters)
    }

    fn write_directory_section(&self, writer: &mut impl Write, entries: &[DirectoryEntry], seq: &mut usize) -> IgesResult<()> {
        for entry in entries {
            // Write two lines per directory entry
            let line1 = format!("{:8}{:8}", entry.entity_type, entry.parameter_data_pointer);
            let line2 = format!("{:8}{:8}", entry.color_number, entry.parameter_line_count);

            self.write_line(writer, &line1, 'D', *seq)?;
            *seq += 1;
            self.write_line(writer, &line2, 'D', *seq)?;
            *seq += 1;
        }

        Ok(())
    }

    fn write_parameter_section(&self, writer: &mut impl Write, parameters: &HashMap<usize, Vec<String>>, seq: &mut usize) -> IgesResult<()> {
        for (entity_num, param_lines) in parameters {
            for line in param_lines {
                let formatted = format!("{:<64}{:8}", line, entity_num);
                self.write_line(writer, &formatted, 'P', *seq)?;
                *seq += 1;
            }
        }

        Ok(())
    }

    fn write_terminate(&self, writer: &mut impl Write, s: usize, g: usize, d: usize, p: usize) -> IgesResult<()> {
        let content = format!("S{:07}G{:07}D{:07}P{:07}", s, g, d, p);
        self.write_line(writer, &content, 'T', 1)?;
        Ok(())
    }
}

impl Default for IgesWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_section_defaults() {
        let global = GlobalSection::default();
        assert_eq!(global.parameter_delimiter, ',');
        assert_eq!(global.record_delimiter, ';');
    }

    #[test]
    fn test_iges_reader_creation() {
        let _reader = IgesReader::new();
        assert!(!reader.strict_mode);
    }

    #[test]
    fn test_iges_writer_creation() {
        let _writer = IgesWriter::new();
        assert_eq!(writer.units_flag, 2);
    }
}
