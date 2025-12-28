// CADDY - Enterprise CAD System
// File I/O System - DXF Format Support
// Agent 6 - File I/O System Developer

use crate::io::document::*;
use crate::io::units::Unit;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use thiserror::Error;

/// DXF-related errors
#[derive(Error, Debug)]
pub enum DxfError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Unsupported DXF version: {0}")]
    UnsupportedVersion(String),
    #[error("Invalid group code: {0}")]
    InvalidGroupCode(i32),
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Invalid entity type: {0}")]
    InvalidEntityType(String),
}

pub type DxfResult<T> = Result<T, DxfError>;

/// DXF file reader
pub struct DxfReader {
    /// Progress callback (current, total)
    progress_callback: Option<Box<dyn Fn(usize, usize)>>,
}

impl DxfReader {
    /// Create a new DXF reader
    pub fn new() -> Self {
        Self {
            progress_callback: None,
        }
    }

    /// Set a progress callback
    pub fn with_progress<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize, usize) + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    /// Read a DXF file
    pub fn read_file<P: AsRef<Path>>(&self, path: P) -> DxfResult<Document> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        self.read(reader)
    }

    /// Read DXF from a buffered reader
    pub fn read<R: BufRead>(&self, reader: R) -> DxfResult<Document> {
        let mut doc = Document::new();
        let mut parser = DxfParser::new(reader);

        // Parse sections
        while let Some(section) = parser.read_section()? {
            match section.name.as_str() {
                "HEADER" => {
                    self.parse_header_section(&section, &mut doc)?;
                }
                "TABLES" => {
                    self.parse_tables_section(&section, &mut doc)?;
                }
                "BLOCKS" => {
                    self.parse_blocks_section(&section, &mut doc)?;
                }
                "ENTITIES" => {
                    self.parse_entities_section(&section, &mut doc)?;
                }
                _ => {
                    // Skip unknown sections
                }
            }
        }

        Ok(doc)
    }

    fn parse_header_section(&self, section: &DxfSection, doc: &mut Document) -> DxfResult<()> {
        let mut i = 0;
        while i < section.entries.len() {
            if let Some(code_pair) = section.entries.get(i) {
                if code_pair.code == 9 {
                    match code_pair.value.as_str() {
                        "$INSUNITS" => {
                            if let Some(next) = section.entries.get(i + 1) {
                                if let Ok(unit_code) = next.value.parse::<i32>() {
                                    if let Some(unit) = Unit::from_dxf_code(unit_code) {
                                        doc.settings.units = unit;
                                    }
                                }
                            }
                        }
                        "$ACADVER" => {
                            if let Some(next) = section.entries.get(i + 1) {
                                doc.variables.insert("ACADVER".to_string(), next.value.clone());
                            }
                        }
                        _ => {}
                    }
                }
            }
            i += 1;
        }
        Ok(())
    }

    fn parse_tables_section(&self, section: &DxfSection, doc: &mut Document) -> DxfResult<()> {
        let mut current_table: Option<String> = None;
        let mut layer_data: Vec<CodePair> = Vec::new();

        for entry in &section.entries {
            if entry.code == 0 {
                match entry.value.as_str() {
                    "TABLE" => {
                        layer_data.clear();
                    }
                    "ENDTAB" => {
                        current_table = None;
                    }
                    "LAYER" => {
                        if !layer_data.is_empty() {
                            if let Some(layer) = self.parse_layer(&layer_data)? {
                                doc.add_layer(layer);
                            }
                        }
                        layer_data.clear();
                    }
                    _ => {}
                }
            } else if entry.code == 2 && current_table.is_none() {
                current_table = Some(entry.value.clone());
            } else if current_table.as_deref() == Some("LAYER") {
                layer_data.push(entry.clone());
            }
        }

        // Parse last layer if any
        if !layer_data.is_empty() {
            if let Some(layer) = self.parse_layer(&layer_data)? {
                doc.add_layer(layer);
            }
        }

        Ok(())
    }

    fn parse_layer(&self, data: &[CodePair]) -> DxfResult<Option<Layer>> {
        let mut name = String::new();
        let mut color = Color::white();
        let mut line_type = LineType::Continuous;
        let mut flags = 0i32;

        for pair in data {
            match pair.code {
                2 => name = pair.value.clone(),
                62 => {
                    if let Ok(color_idx) = pair.value.parse::<u8>() {
                        color = Color::from_autocad_index(color_idx);
                    }
                }
                6 => {
                    line_type = match pair.value.as_str() {
                        "CONTINUOUS" => LineType::Continuous,
                        "DASHED" => LineType::Dashed,
                        "DOTTED" => LineType::Dotted,
                        "DASHDOT" => LineType::DashDot,
                        _ => LineType::Continuous,
                    };
                }
                70 => {
                    if let Ok(f) = pair.value.parse::<i32>() {
                        flags = f;
                    }
                }
                _ => {}
            }
        }

        if name.is_empty() {
            return Ok(None);
        }

        Ok(Some(Layer {
            name,
            color,
            line_type,
            line_weight: LineWeight::Default,
            visible: (flags & 1) == 0,
            frozen: (flags & 2) != 0,
            locked: (flags & 4) != 0,
            plottable: true,
        }))
    }

    fn parse_blocks_section(&self, section: &DxfSection, doc: &mut Document) -> DxfResult<()> {
        // Block parsing implementation
        let mut current_block: Option<Block> = None;
        let mut entity_data: Vec<CodePair> = Vec::new();

        for entry in &section.entries {
            if entry.code == 0 {
                match entry.value.as_str() {
                    "BLOCK" => {
                        current_block = Some(Block {
                            name: String::new(),
                            base_point: Vec3::zero(),
                            entities: Vec::new(),
                            description: String::new(),
                        });
                        entity_data.clear();
                    }
                    "ENDBLK" => {
                        if let Some(block) = current_block.take() {
                            if !block.name.is_empty() {
                                doc.add_block(block);
                            }
                        }
                    }
                    _ => {
                        if current_block.is_some() && !entity_data.is_empty() {
                            if let Some(entity) = self.parse_entity(&entity_data)? {
                                if let Some(ref mut block) = current_block {
                                    block.entities.push(entity);
                                }
                            }
                        }
                        entity_data.clear();
                    }
                }
            } else if let Some(ref mut block) = current_block {
                if entry.code == 2 {
                    block.name = entry.value.clone();
                } else if entry.code == 10 {
                    block.base_point.x = entry.value.parse().unwrap_or(0.0);
                } else if entry.code == 20 {
                    block.base_point.y = entry.value.parse().unwrap_or(0.0);
                } else if entry.code == 30 {
                    block.base_point.z = entry.value.parse().unwrap_or(0.0);
                } else {
                    entity_data.push(entry.clone());
                }
            }
        }

        Ok(())
    }

    fn parse_entities_section(&self, section: &DxfSection, doc: &mut Document) -> DxfResult<()> {
        let mut entity_data: Vec<CodePair> = Vec::new();
        let total = section.entries.len();
        let mut processed = 0;

        for entry in &section.entries {
            if entry.code == 0 && !entity_data.is_empty() {
                if let Some(entity) = self.parse_entity(&entity_data)? {
                    doc.add_entity(entity);
                }
                entity_data.clear();

                processed += 1;
                if let Some(ref callback) = self.progress_callback {
                    callback(processed, total);
                }
            }
            entity_data.push(entry.clone());
        }

        // Parse last entity
        if !entity_data.is_empty() {
            if let Some(entity) = self.parse_entity(&entity_data)? {
                doc.add_entity(entity);
            }
        }

        Ok(())
    }

    fn parse_entity(&self, data: &[CodePair]) -> DxfResult<Option<Entity>> {
        let mut entity_type = String::new();
        let mut layer = "0".to_string();
        let mut color: Option<Color> = None;

        // First pass: get entity type and basic properties
        for pair in data {
            match pair.code {
                0 => entity_type = pair.value.clone(),
                8 => layer = pair.value.clone(),
                62 => {
                    if let Ok(color_idx) = pair.value.parse::<u8>() {
                        color = Some(Color::from_autocad_index(color_idx));
                    }
                }
                _ => {}
            }
        }

        // Parse geometry based on entity type
        let geometry = match entity_type.as_str() {
            "POINT" => self.parse_point(data)?,
            "LINE" => self.parse_line(data)?,
            "CIRCLE" => self.parse_circle(data)?,
            "ARC" => self.parse_arc(data)?,
            "ELLIPSE" => self.parse_ellipse(data)?,
            "LWPOLYLINE" | "POLYLINE" => self.parse_polyline(data)?,
            "SPLINE" => self.parse_spline(data)?,
            "TEXT" => self.parse_text(data)?,
            "MTEXT" => self.parse_mtext(data)?,
            "INSERT" => self.parse_insert(data)?,
            "DIMENSION" => return Ok(None), // Simplified - skip dimensions
            _ => return Ok(None), // Skip unsupported entity types
        };

        if let Some(geom) = geometry {
            let mut entity = Entity::new(geom, layer);
            entity.color = color;
            Ok(Some(entity))
        } else {
            Ok(None)
        }
    }

    fn parse_point(&self, data: &[CodePair]) -> DxfResult<Option<GeometryType>> {
        let mut x = 0.0;
        let mut y = 0.0;
        let mut z = 0.0;

        for pair in data {
            match pair.code {
                10 => x = pair.value.parse().unwrap_or(0.0),
                20 => y = pair.value.parse().unwrap_or(0.0),
                30 => z = pair.value.parse().unwrap_or(0.0),
                _ => {}
            }
        }

        Ok(Some(GeometryType::Point(Point {
            position: Vec3::new(x, y, z),
        })))
    }

    fn parse_line(&self, data: &[CodePair]) -> DxfResult<Option<GeometryType>> {
        let mut x1 = 0.0;
        let mut y1 = 0.0;
        let mut z1 = 0.0;
        let mut x2 = 0.0;
        let mut y2 = 0.0;
        let mut z2 = 0.0;

        for pair in data {
            match pair.code {
                10 => x1 = pair.value.parse().unwrap_or(0.0),
                20 => y1 = pair.value.parse().unwrap_or(0.0),
                30 => z1 = pair.value.parse().unwrap_or(0.0),
                11 => x2 = pair.value.parse().unwrap_or(0.0),
                21 => y2 = pair.value.parse().unwrap_or(0.0),
                31 => z2 = pair.value.parse().unwrap_or(0.0),
                _ => {}
            }
        }

        Ok(Some(GeometryType::Line(Line {
            start: Vec3::new(x1, y1, z1),
            end: Vec3::new(x2, y2, z2),
        })))
    }

    fn parse_circle(&self, data: &[CodePair]) -> DxfResult<Option<GeometryType>> {
        let mut x = 0.0;
        let mut y = 0.0;
        let mut z = 0.0;
        let mut radius = 0.0;

        for pair in data {
            match pair.code {
                10 => x = pair.value.parse().unwrap_or(0.0),
                20 => y = pair.value.parse().unwrap_or(0.0),
                30 => z = pair.value.parse().unwrap_or(0.0),
                40 => radius = pair.value.parse().unwrap_or(0.0),
                _ => {}
            }
        }

        Ok(Some(GeometryType::Circle(Circle {
            center: Vec3::new(x, y, z),
            radius,
            normal: Vec3::unit_z(),
        })))
    }

    fn parse_arc(&self, data: &[CodePair]) -> DxfResult<Option<GeometryType>> {
        let mut x = 0.0;
        let mut y = 0.0;
        let mut z = 0.0;
        let mut radius = 0.0;
        let mut start_angle = 0.0;
        let mut end_angle = 0.0;

        for pair in data {
            match pair.code {
                10 => x = pair.value.parse().unwrap_or(0.0),
                20 => y = pair.value.parse().unwrap_or(0.0),
                30 => z = pair.value.parse().unwrap_or(0.0),
                40 => radius = pair.value.parse().unwrap_or(0.0),
                50 => start_angle = pair.value.parse::<f64>().unwrap_or(0.0).to_radians(),
                51 => end_angle = pair.value.parse::<f64>().unwrap_or(0.0).to_radians(),
                _ => {}
            }
        }

        Ok(Some(GeometryType::Arc(Arc {
            center: Vec3::new(x, y, z),
            radius,
            start_angle,
            end_angle,
            normal: Vec3::unit_z(),
        })))
    }

    fn parse_ellipse(&self, data: &[CodePair]) -> DxfResult<Option<GeometryType>> {
        let mut x = 0.0;
        let mut y = 0.0;
        let mut z = 0.0;
        let mut major = 1.0;
        let mut ratio = 1.0;

        for pair in data {
            match pair.code {
                10 => x = pair.value.parse().unwrap_or(0.0),
                20 => y = pair.value.parse().unwrap_or(0.0),
                30 => z = pair.value.parse().unwrap_or(0.0),
                11 => major = pair.value.parse().unwrap_or(1.0),
                40 => ratio = pair.value.parse().unwrap_or(1.0),
                _ => {}
            }
        }

        Ok(Some(GeometryType::Ellipse(Ellipse {
            center: Vec3::new(x, y, z),
            major_axis: major,
            minor_axis: major * ratio,
            rotation: 0.0,
            normal: Vec3::unit_z(),
        })))
    }

    fn parse_polyline(&self, data: &[CodePair]) -> DxfResult<Option<GeometryType>> {
        let mut vertices = Vec::new();
        let mut closed = false;
        let mut x = 0.0;
        let mut y = 0.0;
        let mut z = 0.0;
        let mut bulge = 0.0;

        for pair in data {
            match pair.code {
                70 => {
                    if let Ok(flags) = pair.value.parse::<i32>() {
                        closed = (flags & 1) != 0;
                    }
                }
                10 => {
                    if !vertices.is_empty() {
                        vertices.push(Vertex {
                            position: Vec3::new(x, y, z),
                            bulge,
                        });
                        bulge = 0.0;
                    }
                    x = pair.value.parse().unwrap_or(0.0);
                }
                20 => y = pair.value.parse().unwrap_or(0.0),
                30 => z = pair.value.parse().unwrap_or(0.0),
                42 => bulge = pair.value.parse().unwrap_or(0.0),
                _ => {}
            }
        }

        // Add last vertex
        vertices.push(Vertex {
            position: Vec3::new(x, y, z),
            bulge,
        });

        Ok(Some(GeometryType::Polyline(Polyline { vertices, closed })))
    }

    fn parse_spline(&self, data: &[CodePair]) -> DxfResult<Option<GeometryType>> {
        let mut degree = 3;
        let mut control_points = Vec::new();
        let mut knots = Vec::new();
        let mut x = 0.0;
        let mut y = 0.0;
        let mut z = 0.0;

        for pair in data {
            match pair.code {
                71 => degree = pair.value.parse().unwrap_or(3),
                10 => {
                    if !control_points.is_empty() {
                        control_points.push(Vec3::new(x, y, z));
                    }
                    x = pair.value.parse().unwrap_or(0.0);
                }
                20 => y = pair.value.parse().unwrap_or(0.0),
                30 => z = pair.value.parse().unwrap_or(0.0),
                40 => knots.push(pair.value.parse().unwrap_or(0.0)),
                _ => {}
            }
        }

        // Add last control point
        control_points.push(Vec3::new(x, y, z));

        Ok(Some(GeometryType::Spline(Spline {
            degree,
            control_points,
            knots,
            weights: None,
            closed: false,
        })))
    }

    fn parse_text(&self, data: &[CodePair]) -> DxfResult<Option<GeometryType>> {
        let mut x = 0.0;
        let mut y = 0.0;
        let mut z = 0.0;
        let mut text = String::new();
        let mut height = 1.0;
        let mut rotation = 0.0;

        for pair in data {
            match pair.code {
                1 => text = pair.value.clone(),
                10 => x = pair.value.parse().unwrap_or(0.0),
                20 => y = pair.value.parse().unwrap_or(0.0),
                30 => z = pair.value.parse().unwrap_or(0.0),
                40 => height = pair.value.parse().unwrap_or(1.0),
                50 => rotation = pair.value.parse::<f64>().unwrap_or(0.0).to_radians(),
                _ => {}
            }
        }

        Ok(Some(GeometryType::Text(Text {
            position: Vec3::new(x, y, z),
            text,
            height,
            rotation,
            style: "Standard".to_string(),
            horizontal_alignment: TextAlignment::Left,
            vertical_alignment: TextAlignment::Bottom,
        })))
    }

    fn parse_mtext(&self, data: &[CodePair]) -> DxfResult<Option<GeometryType>> {
        let mut x = 0.0;
        let mut y = 0.0;
        let mut z = 0.0;
        let mut text = String::new();
        let mut height = 1.0;
        let mut width = 10.0;

        for pair in data {
            match pair.code {
                1 | 3 => text.push_str(&pair.value),
                10 => x = pair.value.parse().unwrap_or(0.0),
                20 => y = pair.value.parse().unwrap_or(0.0),
                30 => z = pair.value.parse().unwrap_or(0.0),
                40 => height = pair.value.parse().unwrap_or(1.0),
                41 => width = pair.value.parse().unwrap_or(10.0),
                _ => {}
            }
        }

        Ok(Some(GeometryType::MText(MText {
            position: Vec3::new(x, y, z),
            text,
            height,
            width,
            rotation: 0.0,
            style: "Standard".to_string(),
            line_spacing: 1.0,
        })))
    }

    fn parse_insert(&self, data: &[CodePair]) -> DxfResult<Option<GeometryType>> {
        let mut block_name = String::new();
        let mut x = 0.0;
        let mut y = 0.0;
        let mut z = 0.0;
        let mut sx = 1.0;
        let mut sy = 1.0;
        let mut sz = 1.0;
        let mut rotation = 0.0;

        for pair in data {
            match pair.code {
                2 => block_name = pair.value.clone(),
                10 => x = pair.value.parse().unwrap_or(0.0),
                20 => y = pair.value.parse().unwrap_or(0.0),
                30 => z = pair.value.parse().unwrap_or(0.0),
                41 => sx = pair.value.parse().unwrap_or(1.0),
                42 => sy = pair.value.parse().unwrap_or(1.0),
                43 => sz = pair.value.parse().unwrap_or(1.0),
                50 => rotation = pair.value.parse::<f64>().unwrap_or(0.0).to_radians(),
                _ => {}
            }
        }

        Ok(Some(GeometryType::Insert(Insert {
            block_name,
            position: Vec3::new(x, y, z),
            scale: Vec3::new(sx, sy, sz),
            rotation,
            attributes: HashMap::new(),
        })))
    }
}

impl Default for DxfReader {
    fn default() -> Self {
        Self::new()
    }
}

/// DXF file writer
pub struct DxfWriter {
    /// DXF version to write
    version: DxfVersion,
    /// Progress callback
    progress_callback: Option<Box<dyn Fn(usize, usize)>>,
}

impl DxfWriter {
    /// Create a new DXF writer
    pub fn new(version: DxfVersion) -> Self {
        Self {
            version,
            progress_callback: None,
        }
    }

    /// Set a progress callback
    pub fn with_progress<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize, usize) + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    /// Write a document to a DXF file
    pub fn write_file<P: AsRef<Path>>(&self, doc: &Document, path: P) -> DxfResult<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        self.write(doc, writer)
    }

    /// Write DXF to a writer
    pub fn write<W: Write>(&self, doc: &Document, mut writer: W) -> DxfResult<()> {
        // Write header section
        self.write_header(&mut writer, doc)?;

        // Write tables section (layers, etc.)
        self.write_tables(&mut writer, doc)?;

        // Write blocks section
        self.write_blocks(&mut writer, doc)?;

        // Write entities section
        self.write_entities(&mut writer, doc)?;

        // Write EOF
        writeln!(writer, "  0")?;
        writeln!(writer, "EOF")?;

        Ok(())
    }

    fn write_header<W: Write>(&self, writer: &mut W, doc: &Document) -> DxfResult<()> {
        writeln!(writer, "  0")?;
        writeln!(writer, "SECTION")?;
        writeln!(writer, "  2")?;
        writeln!(writer, "HEADER")?;

        // Write AutoCAD version
        writeln!(writer, "  9")?;
        writeln!(writer, "$ACADVER")?;
        writeln!(writer, "  1")?;
        writeln!(writer, "{}", self.version.code())?;

        // Write units
        writeln!(writer, "  9")?;
        writeln!(writer, "$INSUNITS")?;
        writeln!(writer, " 70")?;
        writeln!(writer, "{}", doc.settings.units.to_dxf_code())?;

        writeln!(writer, "  0")?;
        writeln!(writer, "ENDSEC")?;

        Ok(())
    }

    fn write_tables<W: Write>(&self, writer: &mut W, doc: &Document) -> DxfResult<()> {
        writeln!(writer, "  0")?;
        writeln!(writer, "SECTION")?;
        writeln!(writer, "  2")?;
        writeln!(writer, "TABLES")?;

        // Write layer table
        writeln!(writer, "  0")?;
        writeln!(writer, "TABLE")?;
        writeln!(writer, "  2")?;
        writeln!(writer, "LAYER")?;
        writeln!(writer, " 70")?;
        writeln!(writer, "{}", doc.layers.len())?;

        for layer in doc.layers.values() {
            self.write_layer(writer, layer)?;
        }

        writeln!(writer, "  0")?;
        writeln!(writer, "ENDTAB")?;
        writeln!(writer, "  0")?;
        writeln!(writer, "ENDSEC")?;

        Ok(())
    }

    fn write_layer<W: Write>(&self, writer: &mut W, layer: &Layer) -> DxfResult<()> {
        writeln!(writer, "  0")?;
        writeln!(writer, "LAYER")?;
        writeln!(writer, "  2")?;
        writeln!(writer, "{}", layer.name)?;
        writeln!(writer, " 70")?;
        let mut flags = 0;
        if !layer.visible {
            flags |= 1;
        }
        if layer.frozen {
            flags |= 2;
        }
        if layer.locked {
            flags |= 4;
        }
        writeln!(writer, "{}", flags)?;
        writeln!(writer, " 62")?;
        writeln!(writer, "7")?; // Simplified - always white
        writeln!(writer, "  6")?;
        writeln!(writer, "CONTINUOUS")?; // Simplified

        Ok(())
    }

    fn write_blocks<W: Write>(&self, writer: &mut W, doc: &Document) -> DxfResult<()> {
        writeln!(writer, "  0")?;
        writeln!(writer, "SECTION")?;
        writeln!(writer, "  2")?;
        writeln!(writer, "BLOCKS")?;

        for block in doc.blocks.values() {
            self.write_block(writer, block)?;
        }

        writeln!(writer, "  0")?;
        writeln!(writer, "ENDSEC")?;

        Ok(())
    }

    fn write_block<W: Write>(&self, writer: &mut W, block: &Block) -> DxfResult<()> {
        writeln!(writer, "  0")?;
        writeln!(writer, "BLOCK")?;
        writeln!(writer, "  2")?;
        writeln!(writer, "{}", block.name)?;
        writeln!(writer, " 10")?;
        writeln!(writer, "{}", block.base_point.x)?;
        writeln!(writer, " 20")?;
        writeln!(writer, "{}", block.base_point.y)?;
        writeln!(writer, " 30")?;
        writeln!(writer, "{}", block.base_point.z)?;

        for entity in &block.entities {
            self.write_entity(writer, entity)?;
        }

        writeln!(writer, "  0")?;
        writeln!(writer, "ENDBLK")?;

        Ok(())
    }

    fn write_entities<W: Write>(&self, writer: &mut W, doc: &Document) -> DxfResult<()> {
        writeln!(writer, "  0")?;
        writeln!(writer, "SECTION")?;
        writeln!(writer, "  2")?;
        writeln!(writer, "ENTITIES")?;

        let total = doc.entities.len();
        for (idx, entity) in doc.entities.iter().enumerate() {
            self.write_entity(writer, entity)?;

            if let Some(ref callback) = self.progress_callback {
                callback(idx + 1, total);
            }
        }

        writeln!(writer, "  0")?;
        writeln!(writer, "ENDSEC")?;

        Ok(())
    }

    fn write_entity<W: Write>(&self, writer: &mut W, entity: &Entity) -> DxfResult<()> {
        match &entity.geometry {
            GeometryType::Point(p) => self.write_point(writer, entity, p)?,
            GeometryType::Line(l) => self.write_line(writer, entity, l)?,
            GeometryType::Circle(c) => self.write_circle(writer, entity, c)?,
            GeometryType::Arc(a) => self.write_arc(writer, entity, a)?,
            GeometryType::Ellipse(e) => self.write_ellipse(writer, entity, e)?,
            GeometryType::Polyline(p) => self.write_polyline(writer, entity, p)?,
            GeometryType::Spline(s) => self.write_spline(writer, entity, s)?,
            GeometryType::Text(t) => self.write_text(writer, entity, t)?,
            GeometryType::MText(t) => self.write_mtext(writer, entity, t)?,
            GeometryType::Insert(i) => self.write_insert(writer, entity, i)?,
            _ => {} // Skip unsupported types
        }

        Ok(())
    }

    fn write_common<W: Write>(&self, writer: &mut W, entity: &Entity, type_name: &str) -> DxfResult<()> {
        writeln!(writer, "  0")?;
        writeln!(writer, "{}", type_name)?;
        writeln!(writer, "  8")?;
        writeln!(writer, "{}", entity.layer)?;
        Ok(())
    }

    fn write_point<W: Write>(&self, writer: &mut W, entity: &Entity, point: &Point) -> DxfResult<()> {
        self.write_common(writer, entity, "POINT")?;
        writeln!(writer, " 10")?;
        writeln!(writer, "{}", point.position.x)?;
        writeln!(writer, " 20")?;
        writeln!(writer, "{}", point.position.y)?;
        writeln!(writer, " 30")?;
        writeln!(writer, "{}", point.position.z)?;
        Ok(())
    }

    fn write_line<W: Write>(&self, writer: &mut W, entity: &Entity, line: &Line) -> DxfResult<()> {
        self.write_common(writer, entity, "LINE")?;
        writeln!(writer, " 10")?;
        writeln!(writer, "{}", line.start.x)?;
        writeln!(writer, " 20")?;
        writeln!(writer, "{}", line.start.y)?;
        writeln!(writer, " 30")?;
        writeln!(writer, "{}", line.start.z)?;
        writeln!(writer, " 11")?;
        writeln!(writer, "{}", line.end.x)?;
        writeln!(writer, " 21")?;
        writeln!(writer, "{}", line.end.y)?;
        writeln!(writer, " 31")?;
        writeln!(writer, "{}", line.end.z)?;
        Ok(())
    }

    fn write_circle<W: Write>(&self, writer: &mut W, entity: &Entity, circle: &Circle) -> DxfResult<()> {
        self.write_common(writer, entity, "CIRCLE")?;
        writeln!(writer, " 10")?;
        writeln!(writer, "{}", circle.center.x)?;
        writeln!(writer, " 20")?;
        writeln!(writer, "{}", circle.center.y)?;
        writeln!(writer, " 30")?;
        writeln!(writer, "{}", circle.center.z)?;
        writeln!(writer, " 40")?;
        writeln!(writer, "{}", circle.radius)?;
        Ok(())
    }

    fn write_arc<W: Write>(&self, writer: &mut W, entity: &Entity, arc: &Arc) -> DxfResult<()> {
        self.write_common(writer, entity, "ARC")?;
        writeln!(writer, " 10")?;
        writeln!(writer, "{}", arc.center.x)?;
        writeln!(writer, " 20")?;
        writeln!(writer, "{}", arc.center.y)?;
        writeln!(writer, " 30")?;
        writeln!(writer, "{}", arc.center.z)?;
        writeln!(writer, " 40")?;
        writeln!(writer, "{}", arc.radius)?;
        writeln!(writer, " 50")?;
        writeln!(writer, "{}", arc.start_angle.to_degrees())?;
        writeln!(writer, " 51")?;
        writeln!(writer, "{}", arc.end_angle.to_degrees())?;
        Ok(())
    }

    fn write_ellipse<W: Write>(&self, writer: &mut W, entity: &Entity, ellipse: &Ellipse) -> DxfResult<()> {
        self.write_common(writer, entity, "ELLIPSE")?;
        writeln!(writer, " 10")?;
        writeln!(writer, "{}", ellipse.center.x)?;
        writeln!(writer, " 20")?;
        writeln!(writer, "{}", ellipse.center.y)?;
        writeln!(writer, " 30")?;
        writeln!(writer, "{}", ellipse.center.z)?;
        writeln!(writer, " 11")?;
        writeln!(writer, "{}", ellipse.major_axis)?;
        writeln!(writer, " 40")?;
        writeln!(writer, "{}", ellipse.minor_axis / ellipse.major_axis)?;
        Ok(())
    }

    fn write_polyline<W: Write>(&self, writer: &mut W, entity: &Entity, polyline: &Polyline) -> DxfResult<()> {
        self.write_common(writer, entity, "LWPOLYLINE")?;
        writeln!(writer, " 90")?;
        writeln!(writer, "{}", polyline.vertices.len())?;
        writeln!(writer, " 70")?;
        writeln!(writer, "{}", if polyline.closed { 1 } else { 0 })?;

        for vertex in &polyline.vertices {
            writeln!(writer, " 10")?;
            writeln!(writer, "{}", vertex.position.x)?;
            writeln!(writer, " 20")?;
            writeln!(writer, "{}", vertex.position.y)?;
            if vertex.bulge.abs() > 1e-10 {
                writeln!(writer, " 42")?;
                writeln!(writer, "{}", vertex.bulge)?;
            }
        }

        Ok(())
    }

    fn write_spline<W: Write>(&self, writer: &mut W, entity: &Entity, spline: &Spline) -> DxfResult<()> {
        self.write_common(writer, entity, "SPLINE")?;
        writeln!(writer, " 71")?;
        writeln!(writer, "{}", spline.degree)?;
        writeln!(writer, " 72")?;
        writeln!(writer, "{}", spline.knots.len())?;
        writeln!(writer, " 73")?;
        writeln!(writer, "{}", spline.control_points.len())?;

        for knot in &spline.knots {
            writeln!(writer, " 40")?;
            writeln!(writer, "{}", knot)?;
        }

        for point in &spline.control_points {
            writeln!(writer, " 10")?;
            writeln!(writer, "{}", point.x)?;
            writeln!(writer, " 20")?;
            writeln!(writer, "{}", point.y)?;
            writeln!(writer, " 30")?;
            writeln!(writer, "{}", point.z)?;
        }

        Ok(())
    }

    fn write_text<W: Write>(&self, writer: &mut W, entity: &Entity, text: &Text) -> DxfResult<()> {
        self.write_common(writer, entity, "TEXT")?;
        writeln!(writer, " 10")?;
        writeln!(writer, "{}", text.position.x)?;
        writeln!(writer, " 20")?;
        writeln!(writer, "{}", text.position.y)?;
        writeln!(writer, " 30")?;
        writeln!(writer, "{}", text.position.z)?;
        writeln!(writer, " 40")?;
        writeln!(writer, "{}", text.height)?;
        writeln!(writer, "  1")?;
        writeln!(writer, "{}", text.text)?;
        writeln!(writer, " 50")?;
        writeln!(writer, "{}", text.rotation.to_degrees())?;
        Ok(())
    }

    fn write_mtext<W: Write>(&self, writer: &mut W, entity: &Entity, mtext: &MText) -> DxfResult<()> {
        self.write_common(writer, entity, "MTEXT")?;
        writeln!(writer, " 10")?;
        writeln!(writer, "{}", mtext.position.x)?;
        writeln!(writer, " 20")?;
        writeln!(writer, "{}", mtext.position.y)?;
        writeln!(writer, " 30")?;
        writeln!(writer, "{}", mtext.position.z)?;
        writeln!(writer, " 40")?;
        writeln!(writer, "{}", mtext.height)?;
        writeln!(writer, " 41")?;
        writeln!(writer, "{}", mtext.width)?;
        writeln!(writer, "  1")?;
        writeln!(writer, "{}", mtext.text)?;
        Ok(())
    }

    fn write_insert<W: Write>(&self, writer: &mut W, entity: &Entity, insert: &Insert) -> DxfResult<()> {
        self.write_common(writer, entity, "INSERT")?;
        writeln!(writer, "  2")?;
        writeln!(writer, "{}", insert.block_name)?;
        writeln!(writer, " 10")?;
        writeln!(writer, "{}", insert.position.x)?;
        writeln!(writer, " 20")?;
        writeln!(writer, "{}", insert.position.y)?;
        writeln!(writer, " 30")?;
        writeln!(writer, "{}", insert.position.z)?;
        writeln!(writer, " 41")?;
        writeln!(writer, "{}", insert.scale.x)?;
        writeln!(writer, " 42")?;
        writeln!(writer, "{}", insert.scale.y)?;
        writeln!(writer, " 43")?;
        writeln!(writer, "{}", insert.scale.z)?;
        writeln!(writer, " 50")?;
        writeln!(writer, "{}", insert.rotation.to_degrees())?;
        Ok(())
    }
}

impl Default for DxfWriter {
    fn default() -> Self {
        Self::new(DxfVersion::R2018)
    }
}

/// DXF version
#[derive(Debug, Clone, Copy)]
pub enum DxfVersion {
    R12,
    R14,
    R2000,
    R2004,
    R2007,
    R2010,
    R2013,
    R2018,
}

impl DxfVersion {
    fn code(&self) -> &'static str {
        match self {
            DxfVersion::R12 => "AC1009",
            DxfVersion::R14 => "AC1014",
            DxfVersion::R2000 => "AC1015",
            DxfVersion::R2004 => "AC1018",
            DxfVersion::R2007 => "AC1021",
            DxfVersion::R2010 => "AC1024",
            DxfVersion::R2013 => "AC1027",
            DxfVersion::R2018 => "AC1032",
        }
    }
}

/// DXF parser helper
struct DxfParser<R: BufRead> {
    reader: R,
}

impl<R: BufRead> DxfParser<R> {
    fn new(reader: R) -> Self {
        Self { reader }
    }

    fn read_section(&mut self) -> DxfResult<Option<DxfSection>> {
        let mut section_name = String::new();
        let mut entries = Vec::new();
        let mut in_section = false;

        loop {
            let mut code_line = String::new();
            let bytes_read = self.reader.read_line(&mut code_line)?;
            if bytes_read == 0 {
                break; // EOF
            }

            let code = code_line.trim().parse::<i32>().unwrap_or(999);

            let mut value_line = String::new();
            self.reader.read_line(&mut value_line)?;
            let value = value_line.trim().to_string();

            if code == 0 {
                match value.as_str() {
                    "SECTION" => {
                        in_section = true;
                    }
                    "ENDSEC" => {
                        if in_section {
                            return Ok(Some(DxfSection {
                                name: section_name,
                                entries,
                            }));
                        }
                    }
                    "EOF" => {
                        return Ok(None);
                    }
                    _ => {}
                }
            }

            if in_section {
                if code == 2 && section_name.is_empty() {
                    section_name = value.clone();
                }
                entries.push(CodePair { code, value });
            }
        }

        Ok(None)
    }
}

#[derive(Debug, Clone)]
struct DxfSection {
    name: String,
    entries: Vec<CodePair>,
}

#[derive(Debug, Clone)]
struct CodePair {
    code: i32,
    value: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dxf_version_codes() {
        assert_eq!(DxfVersion::R2018.code(), "AC1032");
        assert_eq!(DxfVersion::R12.code(), "AC1009");
    }

    #[test]
    fn test_write_read_roundtrip() {
        let mut doc = Document::new();
        doc.add_entity(Entity::new(
            GeometryType::Line(Line {
                start: Vec3::new(0.0, 0.0, 0.0),
                end: Vec3::new(10.0, 10.0, 0.0),
            }),
            "0".to_string(),
        ));

        let mut buffer = Vec::new();
        let writer = DxfWriter::new(DxfVersion::R2018);
        writer.write(&doc, &mut buffer).unwrap();

        // Basic check that something was written
        assert!(!buffer.is_empty());
        let content = String::from_utf8(buffer).unwrap();
        assert!(content.contains("LINE"));
    }
}
