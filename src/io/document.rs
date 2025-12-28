// CADDY - Enterprise CAD System
// File I/O System - Document Structure Module
// Agent 6 - File I/O System Developer

use crate::io::units::{Unit, PrecisionSettings};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Main CAD document structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Unique document identifier
    pub id: Uuid,
    /// Document metadata
    pub metadata: DocumentMetadata,
    /// Document settings
    pub settings: DocumentSettings,
    /// All entities in the document
    pub entities: Vec<Entity>,
    /// Layer definitions
    pub layers: HashMap<String, Layer>,
    /// Block definitions (for reusable components)
    pub blocks: HashMap<String, Block>,
    /// Named views
    pub views: HashMap<String, View>,
    /// Variables (custom properties)
    pub variables: HashMap<String, String>,
}

impl Document {
    /// Create a new empty document
    pub fn new() -> Self {
        let mut layers = HashMap::new();
        layers.insert(
            "0".to_string(),
            Layer {
                name: "0".to_string(),
                color: Color::white(),
                line_type: LineType::Continuous,
                line_weight: LineWeight::Default,
                visible: true,
                locked: false,
                frozen: false,
                plottable: true,
            },
        );

        Self {
            id: Uuid::new_v4(),
            metadata: DocumentMetadata::default(),
            settings: DocumentSettings::default(),
            entities: Vec::new(),
            layers,
            blocks: HashMap::new(),
            views: HashMap::new(),
            variables: HashMap::new(),
        }
    }

    /// Add an entity to the document
    pub fn add_entity(&mut self, entity: Entity) -> Uuid {
        let id = entity.id;
        self.entities.push(entity);
        id
    }

    /// Remove an entity by ID
    pub fn remove_entity(&mut self, id: Uuid) -> Option<Entity> {
        if let Some(pos) = self.entities.iter().position(|e| e.id == id) {
            Some(self.entities.remove(pos))
        } else {
            None
        }
    }

    /// Get an entity by ID
    pub fn get_entity(&self, id: Uuid) -> Option<&Entity> {
        self.entities.iter().find(|e| e.id == id)
    }

    /// Get a mutable reference to an entity by ID
    pub fn get_entity_mut(&mut self, id: Uuid) -> Option<&mut Entity> {
        self.entities.iter_mut().find(|e| e.id == id)
    }

    /// Get all entities on a specific layer
    pub fn entities_on_layer(&self, layer_name: &str) -> Vec<&Entity> {
        self.entities
            .iter()
            .filter(|e| e.layer == layer_name)
            .collect()
    }

    /// Add a layer
    pub fn add_layer(&mut self, layer: Layer) {
        self.layers.insert(layer.name.clone(), layer);
    }

    /// Get a layer by name
    pub fn get_layer(&self, name: &str) -> Option<&Layer> {
        self.layers.get(name)
    }

    /// Add a block definition
    pub fn add_block(&mut self, block: Block) {
        self.blocks.insert(block.name.clone(), block);
    }

    /// Get a block by name
    pub fn get_block(&self, name: &str) -> Option<&Block> {
        self.blocks.get(name)
    }

    /// Calculate bounding box of all entities
    pub fn bounding_box(&self) -> Option<BoundingBox> {
        if self.entities.is_empty() {
            return None;
        }

        let mut bbox = BoundingBox::invalid();
        for entity in &self.entities {
            bbox = bbox.union(&entity.bounding_box());
        }

        Some(bbox)
    }

    /// Get all layer names
    pub fn layer_names(&self) -> Vec<String> {
        self.layers.keys().cloned().collect()
    }

    /// Count entities by type
    pub fn entity_count_by_type(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for entity in &self.entities {
            let type_name = entity.geometry.type_name();
            *counts.entry(type_name.to_string()).or_insert(0) += 1;
        }
        counts
    }

    /// Validate document integrity
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // Check that all entities reference valid layers
        for entity in &self.entities {
            if !self.layers.contains_key(&entity.layer) {
                errors.push(format!(
                    "Entity {} references non-existent layer '{}'",
                    entity.id, entity.layer
                ));
            }
        }

        // Check for duplicate layer names
        let layer_count = self.layers.len();
        let unique_names: HashSet<_> = self.layers.keys().collect();
        if layer_count != unique_names.len() {
            errors.push("Duplicate layer names detected".to_string());
        }

        errors
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    /// Document title
    pub title: String,
    /// Author name
    pub author: String,
    /// Company/organization
    pub company: String,
    /// Subject/description
    pub subject: String,
    /// Keywords
    pub keywords: Vec<String>,
    /// Comments
    pub comments: String,
    /// Creation date
    pub created: DateTime<Utc>,
    /// Last modification date
    pub modified: DateTime<Utc>,
    /// Application that created the document
    pub application: String,
    /// Application version
    pub application_version: String,
    /// Custom properties
    pub custom_properties: HashMap<String, String>,
}

impl Default for DocumentMetadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            title: "Untitled".to_string(),
            author: String::new(),
            company: String::new(),
            subject: String::new(),
            keywords: Vec::new(),
            comments: String::new(),
            created: now,
            modified: now,
            application: "CADDY".to_string(),
            application_version: env!("CARGO_PKG_VERSION").to_string(),
            custom_properties: HashMap::new(),
        }
    }
}

/// Document settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSettings {
    /// Drawing units
    pub units: Unit,
    /// Precision settings
    pub precision: PrecisionSettings,
    /// Paper space settings
    pub paper_size: PaperSize,
    /// Background color
    pub background_color: Color,
    /// Grid settings
    pub grid: GridSettings,
    /// Snap settings
    pub snap: SnapSettings,
    /// Automatic save interval (seconds), None = disabled
    pub autosave_interval: Option<u64>,
    /// Create backup on save
    pub create_backup: bool,
}

impl Default for DocumentSettings {
    fn default() -> Self {
        Self {
            units: Unit::Millimeters,
            precision: PrecisionSettings::default(),
            paper_size: PaperSize::A4,
            background_color: Color::new(0, 0, 0),
            grid: GridSettings::default(),
            snap: SnapSettings::default(),
            autosave_interval: Some(300), // 5 minutes
            create_backup: true,
        }
    }
}

/// Standard paper sizes
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PaperSize {
    A4,
    A3,
    A2,
    A1,
    A0,
    Letter,
    Legal,
    Tabloid,
    Custom { width: f64, height: f64 },
}

impl PaperSize {
    /// Get dimensions in millimeters (width, height)
    pub fn dimensions_mm(&self) -> (f64, f64) {
        match self {
            PaperSize::A4 => (210.0, 297.0),
            PaperSize::A3 => (297.0, 420.0),
            PaperSize::A2 => (420.0, 594.0),
            PaperSize::A1 => (594.0, 841.0),
            PaperSize::A0 => (841.0, 1189.0),
            PaperSize::Letter => (215.9, 279.4),
            PaperSize::Legal => (215.9, 355.6),
            PaperSize::Tabloid => (279.4, 431.8),
            PaperSize::Custom { width, height } => (*width, *height),
        }
    }
}

/// Grid settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridSettings {
    /// Grid enabled
    pub enabled: bool,
    /// Major grid spacing
    pub major_spacing: f64,
    /// Minor grid spacing
    pub minor_spacing: f64,
    /// Grid color
    pub color: Color,
    /// Snap to grid
    pub snap: bool,
}

impl Default for GridSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            major_spacing: 10.0,
            minor_spacing: 1.0,
            color: Color::new(128, 128, 128),
            snap: false,
        }
    }
}

/// Snap settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapSettings {
    /// Snap enabled
    pub enabled: bool,
    /// Snap to endpoints
    pub endpoint: bool,
    /// Snap to midpoints
    pub midpoint: bool,
    /// Snap to center
    pub center: bool,
    /// Snap to intersections
    pub intersection: bool,
    /// Snap to perpendicular
    pub perpendicular: bool,
    /// Snap to tangent
    pub tangent: bool,
    /// Snap distance threshold
    pub threshold: f64,
}

impl Default for SnapSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: true,
            midpoint: true,
            center: true,
            intersection: true,
            perpendicular: true,
            tangent: true,
            threshold: 5.0, // pixels
        }
    }
}

/// CAD entity (geometric object)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Unique entity identifier
    pub id: Uuid,
    /// Layer name
    pub layer: String,
    /// Entity color (None = use layer color)
    pub color: Option<Color>,
    /// Line type (None = use layer line type)
    pub line_type: Option<LineType>,
    /// Line weight (None = use layer line weight)
    pub line_weight: Option<LineWeight>,
    /// Visibility
    pub visible: bool,
    /// Geometric data
    pub geometry: GeometryType,
    /// Custom attributes
    pub attributes: HashMap<String, String>,
}

impl Entity {
    /// Create a new entity
    pub fn new(geometry: GeometryType, layer: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            layer,
            color: None,
            line_type: None,
            line_weight: None,
            visible: true,
            geometry,
            attributes: HashMap::new(),
        }
    }

    /// Get the bounding box of this entity
    pub fn bounding_box(&self) -> BoundingBox {
        self.geometry.bounding_box()
    }
}

/// Geometry type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GeometryType {
    Point(Point),
    Line(Line),
    Circle(Circle),
    Arc(Arc),
    Ellipse(Ellipse),
    Polyline(Polyline),
    Spline(Spline),
    Text(Text),
    MText(MText),
    Dimension(Dimension),
    Insert(Insert),
    Hatch(Hatch),
}

impl GeometryType {
    /// Get the type name as a string
    pub fn type_name(&self) -> &str {
        match self {
            GeometryType::Point(_) => "Point",
            GeometryType::Line(_) => "Line",
            GeometryType::Circle(_) => "Circle",
            GeometryType::Arc(_) => "Arc",
            GeometryType::Ellipse(_) => "Ellipse",
            GeometryType::Polyline(_) => "Polyline",
            GeometryType::Spline(_) => "Spline",
            GeometryType::Text(_) => "Text",
            GeometryType::MText(_) => "MText",
            GeometryType::Dimension(_) => "Dimension",
            GeometryType::Insert(_) => "Insert",
            GeometryType::Hatch(_) => "Hatch",
        }
    }

    /// Get the bounding box
    pub fn bounding_box(&self) -> BoundingBox {
        match self {
            GeometryType::Point(p) => BoundingBox::from_point(p.position),
            GeometryType::Line(l) => BoundingBox::from_points(&[l.start, l.end]),
            GeometryType::Circle(c) => BoundingBox::from_circle(c.center, c.radius),
            GeometryType::Arc(a) => BoundingBox::from_circle(a.center, a.radius),
            GeometryType::Ellipse(e) => BoundingBox::from_circle(e.center, e.major_axis),
            GeometryType::Polyline(p) => BoundingBox::from_points(&p.vertices.iter().map(|v| v.position).collect::<Vec<_>>()),
            GeometryType::Spline(s) => BoundingBox::from_points(&s.control_points),
            GeometryType::Text(t) => BoundingBox::from_point(t.position),
            GeometryType::MText(t) => BoundingBox::from_point(t.position),
            GeometryType::Dimension(d) => d.bounding_box(),
            GeometryType::Insert(i) => BoundingBox::from_point(i.position),
            GeometryType::Hatch(_) => BoundingBox::invalid(),
        }
    }
}

/// 3D Point
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point {
    pub position: Vec3,
}

/// 3D Line
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Line {
    pub start: Vec3,
    pub end: Vec3,
}

/// Circle
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Circle {
    pub center: Vec3,
    pub radius: f64,
    pub normal: Vec3,
}

/// Arc
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Arc {
    pub center: Vec3,
    pub radius: f64,
    pub start_angle: f64, // radians
    pub end_angle: f64,   // radians
    pub normal: Vec3,
}

/// Ellipse
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Ellipse {
    pub center: Vec3,
    pub major_axis: f64,
    pub minor_axis: f64,
    pub rotation: f64, // radians
    pub normal: Vec3,
}

/// Polyline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Polyline {
    pub vertices: Vec<Vertex>,
    pub closed: bool,
}

/// Polyline vertex
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vertex {
    pub position: Vec3,
    pub bulge: f64, // For arc segments
}

/// Spline curve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spline {
    pub degree: usize,
    pub control_points: Vec<Vec3>,
    pub knots: Vec<f64>,
    pub weights: Option<Vec<f64>>, // For NURBS
    pub closed: bool,
}

/// Text entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Text {
    pub position: Vec3,
    pub text: String,
    pub height: f64,
    pub rotation: f64,
    pub style: String,
    pub horizontal_alignment: TextAlignment,
    pub vertical_alignment: TextAlignment,
}

/// Multi-line text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MText {
    pub position: Vec3,
    pub text: String,
    pub height: f64,
    pub width: f64,
    pub rotation: f64,
    pub style: String,
    pub line_spacing: f64,
}

/// Text alignment
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Top,
    Middle,
    Bottom,
}

/// Dimension entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dimension {
    pub dim_type: DimensionType,
    pub definition_point: Vec3,
    pub text_position: Vec3,
    pub text_override: Option<String>,
}

impl Dimension {
    fn bounding_box(&self) -> BoundingBox {
        BoundingBox::from_points(&[self.definition_point, self.text_position])
    }
}

/// Dimension type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DimensionType {
    Linear {
        start: Vec3,
        end: Vec3,
        angle: f64,
    },
    Aligned {
        start: Vec3,
        end: Vec3,
    },
    Angular {
        center: Vec3,
        start: Vec3,
        end: Vec3,
    },
    Radial {
        center: Vec3,
        radius: f64,
    },
    Diameter {
        center: Vec3,
        diameter: f64,
    },
}

/// Block insert (instance of a block definition)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insert {
    pub block_name: String,
    pub position: Vec3,
    pub scale: Vec3,
    pub rotation: f64,
    pub attributes: HashMap<String, String>,
}

/// Hatch pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hatch {
    pub pattern: String,
    pub scale: f64,
    pub angle: f64,
    pub boundaries: Vec<Vec<Vec3>>,
}

/// Layer definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    /// Layer name
    pub name: String,
    /// Layer color
    pub color: Color,
    /// Line type
    pub line_type: LineType,
    /// Line weight
    pub line_weight: LineWeight,
    /// Visibility
    pub visible: bool,
    /// Locked (cannot be edited)
    pub locked: bool,
    /// Frozen (not visible, not calculated)
    pub frozen: bool,
    /// Plottable
    pub plottable: bool,
}

/// Block definition (reusable component)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Block name
    pub name: String,
    /// Base point
    pub base_point: Vec3,
    /// Entities in the block
    pub entities: Vec<Entity>,
    /// Description
    pub description: String,
}

/// Named view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct View {
    /// View name
    pub name: String,
    /// Center point
    pub center: Vec3,
    /// View height
    pub height: f64,
    /// View width
    pub width: f64,
    /// Target point (for 3D views)
    pub target: Vec3,
    /// Camera direction
    pub direction: Vec3,
    /// Twist angle
    pub twist: f64,
}

/// RGB Color
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn white() -> Self {
        Self::new(255, 255, 255)
    }

    pub fn black() -> Self {
        Self::new(0, 0, 0)
    }

    pub fn red() -> Self {
        Self::new(255, 0, 0)
    }

    pub fn green() -> Self {
        Self::new(0, 255, 0)
    }

    pub fn blue() -> Self {
        Self::new(0, 0, 255)
    }

    pub fn from_autocad_index(index: u8) -> Self {
        // Simplified AutoCAD color index to RGB conversion
        match index {
            1 => Self::red(),
            2 => Color::new(255, 255, 0), // Yellow
            3 => Self::green(),
            4 => Color::new(0, 255, 255), // Cyan
            5 => Self::blue(),
            6 => Color::new(255, 0, 255), // Magenta
            7 => Self::white(),
            _ => Self::white(),
        }
    }

    pub fn to_f32_array(&self) -> [f32; 3] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        ]
    }
}

/// Line type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LineType {
    Continuous,
    Dashed,
    Dotted,
    DashDot,
    Custom { name: String, pattern: Vec<f64> },
}

/// Line weight
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineWeight {
    Default,
    ByLayer,
    ByBlock,
    Hairline,
    Width(u16), // in 1/100 mm
}

/// 3D Vector
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn unit_x() -> Self {
        Self::new(1.0, 0.0, 0.0)
    }

    pub fn unit_y() -> Self {
        Self::new(0.0, 1.0, 0.0)
    }

    pub fn unit_z() -> Self {
        Self::new(0.0, 0.0, 1.0)
    }
}

/// Axis-aligned bounding box
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl BoundingBox {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn invalid() -> Self {
        Self {
            min: Vec3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY),
            max: Vec3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
        }
    }

    pub fn from_point(point: Vec3) -> Self {
        Self {
            min: point,
            max: point,
        }
    }

    pub fn from_points(points: &[Vec3]) -> Self {
        let mut bbox = Self::invalid();
        for &point in points {
            bbox = bbox.union(&Self::from_point(point));
        }
        bbox
    }

    pub fn from_circle(center: Vec3, radius: f64) -> Self {
        Self {
            min: Vec3::new(center.x - radius, center.y - radius, center.z),
            max: Vec3::new(center.x + radius, center.y + radius, center.z),
        }
    }

    pub fn union(&self, other: &BoundingBox) -> Self {
        Self {
            min: Vec3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            max: Vec3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        }
    }

    pub fn center(&self) -> Vec3 {
        Vec3::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
            (self.min.z + self.max.z) / 2.0,
        )
    }

    pub fn size(&self) -> Vec3 {
        Vec3::new(
            self.max.x - self.min.x,
            self.max.y - self.min.y,
            self.max.z - self.min.z,
        )
    }
}
