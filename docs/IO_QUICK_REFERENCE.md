# CADDY I/O System - Quick Reference Guide

## Common Operations

### Opening Files

```rust
use caddy::io::prelude::*;

// Auto-detect format
let doc = FormatDetector::load("file.cdy")?;

// Specific formats
let doc = DxfReader::new().read_file("file.dxf")?;
let doc = NativeFormat::new().load("file.cdy")?;
let doc = JsonFormat::new().load("file.cdyj")?;
```

### Saving Files

```rust
// Native binary (recommended for production)
NativeFormat::new()
    .with_compression(6)
    .save(&doc, "output.cdy")?;

// JSON (for debugging/interchange)
JsonFormat::new()
    .pretty(true)
    .save(&doc, "output.cdyj")?;

// DXF (for AutoCAD compatibility)
DxfWriter::new(DxfVersion::R2018)
    .write_file(&doc, "output.dxf")?;
```

### Creating Documents

```rust
use caddy::io::prelude::*;

let mut doc = Document::new();

// Set metadata
doc.metadata.title = "My Drawing".to_string();
doc.metadata.author = "John Doe".to_string();

// Set units
doc.settings.units = Unit::Millimeters;
```

### Adding Entities

```rust
// Line
doc.add_entity(Entity::new(
    GeometryType::Line(Line {
        start: Vec3::new(0.0, 0.0, 0.0),
        end: Vec3::new(100.0, 100.0, 0.0),
    }),
    "0".to_string()
));

// Circle
doc.add_entity(Entity::new(
    GeometryType::Circle(Circle {
        center: Vec3::new(50.0, 50.0, 0.0),
        radius: 25.0,
        normal: Vec3::unit_z(),
    }),
    "0".to_string()
));

// Polyline
doc.add_entity(Entity::new(
    GeometryType::Polyline(Polyline {
        vertices: vec![
            Vertex { position: Vec3::new(0.0, 0.0, 0.0), bulge: 0.0 },
            Vertex { position: Vec3::new(100.0, 0.0, 0.0), bulge: 0.0 },
            Vertex { position: Vec3::new(100.0, 100.0, 0.0), bulge: 0.0 },
        ],
        closed: false,
    }),
    "0".to_string()
));
```

### Working with Layers

```rust
// Add a layer
doc.add_layer(Layer {
    name: "Dimensions".to_string(),
    color: Color::new(255, 0, 0),
    line_type: LineType::Continuous,
    line_weight: LineWeight::Default,
    visible: true,
    locked: false,
    frozen: false,
    plottable: true,
});

// Get entities on a layer
let entities = doc.entities_on_layer("0");
```

### Exporting

```rust
use caddy::io::export::*;

// SVG
let svg_settings = SvgExportSettings {
    width: 800.0,
    height: 600.0,
    scale: 1.0,
    background: Some(Color::white()),
    stroke_width: 1.0,
    include_layer_comments: true,
    view_box: None,
    precision: 3,
};
SvgExporter::new(svg_settings).export(&doc, "output.svg")?;

// Batch export
let exporter = Exporter;
exporter.export(&doc, "output.svg")?;
```

### Unit Conversion

```rust
use caddy::io::units::*;

// Convert values
let mm = Unit::Millimeters;
let inches = Unit::Inches;
let value_in_inches = mm.convert_to(25.4, inches); // 1.0

// Format values
let formatted = mm.format(10.5, None); // "10.50 mm"
let formatted = Unit::Architectural.format(1.5, None); // "1'-6""
```

### Progress Callbacks

```rust
// With progress reporting
let reader = DxfReader::new()
    .with_progress(|current, total| {
        println!("Loading: {}/{}", current, total);
    });
let doc = reader.read_file("large_file.dxf")?;
```

### Backups

```rust
use caddy::io::native::BackupManager;

let backup_mgr = BackupManager::new(3); // Keep 3 backups

// Create backup before saving
backup_mgr.create_backup("drawing.cdy")?;

// Restore from backup
backup_mgr.restore_backup("drawing.cdy", 1)?;

// List backups
let backups = backup_mgr.list_backups("drawing.cdy");
```

## Supported File Formats

### Input (Read)
- ✅ `.cdy` - CADDY native binary
- ✅ `.cdyj` - CADDY JSON
- ✅ `.dxf` - AutoCAD DXF (R12-R2018)
- ✅ `.svg` - Scalable Vector Graphics (basic)

### Output (Write)
- ✅ `.cdy` - CADDY native binary
- ✅ `.cdyj` - CADDY JSON
- ✅ `.dxf` - AutoCAD DXF (R12-R2018)
- ✅ `.svg` - Scalable Vector Graphics
- 🚧 `.pdf` - PDF (framework ready)
- 🚧 `.png` - PNG (framework ready)

## Geometry Types

| Type | Description | 3D Support |
|------|-------------|-----------|
| Point | Single point | ✅ |
| Line | Two points | ✅ |
| Circle | Center + radius | ✅ |
| Arc | Center + radius + angles | ✅ |
| Ellipse | Center + major/minor axis | ✅ |
| Polyline | Multiple vertices | ✅ |
| Spline | NURBS curve | ✅ |
| Text | Single line text | ✅ |
| MText | Multi-line text | ✅ |
| Dimension | Measurement annotation | ✅ |
| Insert | Block instance | ✅ |
| Hatch | Fill pattern | ✅ |

## Units Supported

- **Metric**: Millimeters, Centimeters, Meters
- **Imperial**: Inches, Feet
- **Special**: Decimal, Engineering, Architectural, Fractional

## Error Handling

```rust
use caddy::io::dxf::DxfError;

match DxfReader::new().read_file("file.dxf") {
    Ok(doc) => println!("Loaded {} entities", doc.entities.len()),
    Err(DxfError::Io(e)) => eprintln!("I/O error: {}", e),
    Err(DxfError::Parse(e)) => eprintln!("Parse error: {}", e),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Best Practices

1. **Use native format (.cdy) for production**
   - Fastest load/save
   - Best compression
   - Full fidelity

2. **Use JSON (.cdyj) for debugging**
   - Human-readable
   - Easy to inspect
   - Version control friendly

3. **Use DXF for interchange**
   - AutoCAD compatible
   - Industry standard
   - Wide support

4. **Always validate after import**
   ```rust
   let errors = doc.validate();
   if !errors.is_empty() {
       eprintln!("Validation errors: {:?}", errors);
   }
   ```

5. **Use progress callbacks for large files**
   ```rust
   let reader = DxfReader::new()
       .with_progress(|cur, tot| {
           println!("{}%", cur * 100 / tot);
       });
   ```

## Performance Tips

- Use compression level 6 for balanced speed/size
- Use compression level 9 for maximum compression
- Use compression level 0 for maximum speed
- Stream large files in chunks (when implemented)
- Use batch operations for multiple files

## Module Import Paths

```rust
// Everything
use caddy::io::prelude::*;

// Specific
use caddy::io::document::Document;
use caddy::io::dxf::{DxfReader, DxfWriter};
use caddy::io::native::NativeFormat;
use caddy::io::export::SvgExporter;
use caddy::io::units::Unit;
```
