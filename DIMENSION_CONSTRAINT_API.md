# CADDY Dimensions & Constraints - API Reference

## Dimensions Module (`caddy::dimensions`)

### Dimension Styles

#### `DimensionStyle`
Main style configuration for dimensions.

**Constructors:**
- `DimensionStyle::new(name)` - Create with defaults
- `DimensionStyle::iso()` - ISO 25 standard
- `DimensionStyle::ansi()` - ANSI standard
- `DimensionStyle::din()` - DIN standard (German)
- `DimensionStyle::jis()` - JIS standard (Japanese)
- `DimensionStyle::architectural()` - Architectural style

**Key Properties:**
- `text_height: f64` - Text height in drawing units
- `text_color: Color` - Text color
- `arrow_type_1/2: ArrowType` - Arrow types
- `arrow_size: f64` - Arrow size
- `precision: u8` - Decimal places
- `unit_format: UnitFormat` - Number format
- `tolerance_format: ToleranceFormat` - Tolerance display
- `alt_units_enabled: bool` - Show alternate units

**Methods:**
- `format_linear(value: f64) -> String` - Format linear measurement
- `format_angular(radians: f64) -> String` - Format angle
- `inherit_from(parent: &DimensionStyle)` - Inherit from parent

#### Enums

**`ArrowType`**: `ClosedFilled`, `ClosedBlank`, `Open`, `Dot`, `ArchTick`, `Oblique`, `RightAngle`, `None`

**`UnitFormat`**: `Scientific`, `Decimal`, `Engineering`, `Architectural`, `Fractional`, `WindowsDesktop`

**`AngularUnitFormat`**: `DecimalDegrees`, `DegMinSec`, `Gradians`, `Radians`, `Surveyors`

**`ToleranceFormat`**: `None`, `Symmetrical`, `Deviation`, `Limits`, `Basic`

**`DimTextAlignment`**: `Centered`, `AtExtLine1`, `AtExtLine2`, `Above`, `Outside`

---

### Linear Dimensions

#### `LinearDimension`
Horizontal, vertical, aligned, or rotated dimensions.

**Constructors:**
- `horizontal(p1, p2, dim_line_y, style_name)` - Horizontal dimension
- `vertical(p1, p2, dim_line_x, style_name)` - Vertical dimension
- `aligned(p1, p2, dim_line_point, style_name)` - Aligned dimension
- `rotated(p1, p2, dim_line_point, angle, style_name)` - Rotated dimension

**Properties:**
- `dim_type: LinearDimensionType` - Dimension type
- `ext_line1_point: Point3D` - First extension line origin
- `ext_line2_point: Point3D` - Second extension line origin
- `dim_line_point: Point3D` - Dimension line location
- `text_override: Option<String>` - Custom text
- `associative: bool` - Auto-update with geometry

**Methods:**
- `calculate_measurement() -> f64` - Get measured distance
- `get_text(&style) -> String` - Get formatted text
- `set_text_override(text: Option<String>)` - Override text
- `associate_with(entity_ids: Vec<Uuid>)` - Make associative
- `get_dimension_line_points() -> (Point3D, Point3D)` - Dimension line endpoints

#### `OrdinateDimension`
Datum dimensions (X or Y from origin).

**Constructor:**
- `new(feature_point, leader_endpoint, datum_point, is_x_ordinate, style_name)`

**Methods:**
- `calculate_measurement() -> f64` - Ordinate value
- `get_text(&style) -> String` - Formatted text

#### `BaselineDimension`
Multiple dimensions from same baseline.

**Constructor:**
- `new(base_dimension_id, spacing, style_name)`

**Methods:**
- `add_point(point: Point3D)` - Add dimension point
- `get_dimension_lines(base_point) -> Vec<(Point3D, Point3D)>` - All lines

#### `ContinueDimension`
Chain dimensions end-to-end.

**Constructor:**
- `new(previous_dimension_id, style_name)`

**Methods:**
- `add_point(point: Point3D)` - Add continue point
- `get_dimension_segments(start_point) -> Vec<(Point3D, Point3D)>` - Segments

---

### Angular Dimensions

#### `AngularDimension`
Angle between two lines.

**Constructor:**
- `new(center_point, line1_point, line2_point, arc_point, style_name)`

**Methods:**
- `calculate_angle() -> f64` - Angle in radians
- `get_text(&style) -> String` - Formatted angle
- `get_arc_radius() -> f64` - Dimension arc radius
- `get_arc_angles() -> (f64, f64)` - Start and end angles

#### `Angular3PointDimension`
Three-point angle.

**Constructor:**
- `new(center_point, point1, point2, arc_point, style_name)`

**Methods:**
- `calculate_angle() -> f64` - Angle in radians
- `get_text(&style) -> String` - Formatted angle

#### `ArcLengthDimension`
Arc length with symbol.

**Constructor:**
- `new(center_point, start_point, end_point, dim_arc_point, style_name)`

**Properties:**
- `symbol_location: ArcSymbolLocation` - Symbol position

**Methods:**
- `calculate_length() -> f64` - Arc length
- `get_radius() -> f64` - Arc radius
- `get_text(&style) -> String` - Text with arc symbol

#### `QuadrantAngles`
Angle utilities.

**Static Methods:**
- `normalize_degrees(angle: f64) -> f64` - Normalize to 0-360
- `normalize_radians(angle: f64) -> f64` - Normalize to 0-2π
- `get_quadrant(angle: f64) -> u8` - Get quadrant 1-4
- `angle_between(v1x, v1y, v2x, v2y) -> f64` - Angle between vectors
- `smallest_difference(angle1, angle2) -> f64` - Smallest angle difference

---

### Radial Dimensions

#### `RadiusDimension`
Radius dimension with "R" prefix.

**Constructors:**
- `new(center_point, chord_point, leader_endpoint, style_name)`
- `from_circle(center, radius, angle, style_name)` - From circle parameters

**Methods:**
- `calculate_radius() -> f64` - Radius value
- `get_text(&style) -> String` - "R{value}"
- `associate_with(entity_id: Uuid)` - Make associative
- `update_from_circle(center, radius)` - Update from geometry

#### `DiameterDimension`
Diameter dimension with "Ø" symbol.

**Constructors:**
- `new(center_point, chord_point1, chord_point2, leader_endpoint, style_name)`
- `from_circle(center, radius, angle, style_name)` - From circle parameters

**Methods:**
- `calculate_diameter() -> f64` - Diameter value
- `calculate_radius() -> f64` - Radius value
- `get_text(&style) -> String` - "Ø{value}"

#### `JoggedRadiusDimension`
For large radii with jog line.

**Constructor:**
- `new(center_point, override_center, chord_point, jog_point, leader_endpoint, style_name)`

**Properties:**
- `jog_angle: f64` - Jog angle (default π/4)

**Methods:**
- `calculate_radius() -> f64` - True radius
- `get_leader_segments() -> Vec<(Point3D, Point3D)>` - Leader segments
- `calculate_jog_point(distance_from_chord: f64) -> Point3D` - Optimal jog

#### `RadialHelper`
Circle detection utilities.

**Static Methods:**
- `is_circular_arc(p1, p2, p3) -> bool` - Check if points form arc
- `calculate_center_from_three_points(p1, p2, p3) -> Option<Point3D>` - Find center
- `calculate_radius_from_three_points(p1, p2, p3) -> Option<f64>` - Find radius
- `needs_jogged_dimension(radius, viewport_size) -> bool` - Check if jog needed

---

### Text Annotations

#### `Text`
Single-line text.

**Constructor:**
- `new(content, position, height, style_name)`

**Properties:**
- `content: String` - Text content
- `position: Point3D` - Insertion point
- `rotation: f64` - Rotation angle
- `h_align: TextHAlignment` - Horizontal alignment
- `v_align: TextVAlignment` - Vertical alignment

**Builder Methods:**
- `with_rotation(angle: f64) -> Self`
- `with_alignment(h_align, v_align) -> Self`
- `with_layer(layer: String) -> Self`

**Methods:**
- `get_height(&style) -> f64` - Effective height
- `bounding_box(&style) -> (Point3D, Point3D)` - Bounds

#### `MText`
Multi-line formatted text.

**Constructor:**
- `new(content, position, height, style_name)`

**Properties:**
- `raw_content: String` - Text with formatting codes
- `attachment: TextAttachment` - Attachment point
- `width: Option<f64>` - Wrapping width
- `line_spacing: f64` - Line spacing factor

**Builder Methods:**
- `with_width(width: f64) -> Self`
- `with_attachment(attachment: TextAttachment) -> Self`
- `with_rotation(angle: f64) -> Self`
- `with_line_spacing(spacing: f64) -> Self`

**Methods:**
- `get_plain_text() -> String` - Text without formatting
- `get_wrapped_lines(&style) -> Vec<String>` - Lines after wrapping
- `set_content(content: String)` - Update content
- `bounding_box(&style) -> (Point3D, Point3D)` - Bounds

#### `TextStyle`
Text formatting.

**Constructor:**
- `new(name)` - Create new style
- `standard()` - Standard text style
- `annotative()` - Annotative style

**Properties:**
- `font_name: String` - Font name
- `height: f64` - Text height
- `width_factor: f64` - Width factor (1.0 = normal)
- `oblique_angle: f64` - Oblique angle
- `backwards: bool` - Backwards text
- `upside_down: bool` - Upside down text
- `color: Color` - Text color

#### `FieldText`
Dynamic text with fields.

**Constructor:**
- `new(content, position, height, style_name)`

**Methods:**
- `add_field(placeholder: String, field: TextField)` - Add field
- `evaluate() -> String` - Evaluate fields

**TextField Variants:**
- `Date`, `Time`, `FileName`, `FilePath`, `DrawingNumber`, `SheetNumber`, `Custom(String)`

---

### Leaders

#### `Leader`
Traditional leader line.

**Constructor:**
- `new(vertices: Vec<Point3D>, annotation: LeaderAnnotation, style_name)`

**Properties:**
- `leader_type: LeaderType` - `Straight`, `Spline`, or `None`
- `arrow_type: ArrowType` - Arrow style
- `has_hookline: bool` - Horizontal landing line

**Builder Methods:**
- `with_arrow(arrow_type: ArrowType, size: f64) -> Self`
- `with_hookline(direction: i8) -> Self` - 1 = right, -1 = left

**Methods:**
- `add_vertex(point: Point3D)` - Add vertex
- `get_arrow_direction() -> Option<(f64, f64, f64)>` - Arrow direction vector

#### `MultiLeader`
Modern multi-leader (MLEADER).

**Constructor:**
- `new(leader_line: Vec<Point3D>, content: MLeaderContent, content_position, style_name)`

**Properties:**
- `leader_lines: Vec<Vec<Point3D>>` - Multiple leader lines
- `content: MLeaderContent` - MText or Block
- `attachment_side: MLeaderAttachmentSide` - Attachment side

**Builder Methods:**
- `with_rotation(angle: f64) -> Self`

**Methods:**
- `add_leader_line(vertices: Vec<Point3D>)` - Add leader
- `remove_leader_line(index: usize)` - Remove leader
- `get_landing_lines(&style) -> Vec<(Point3D, Point3D)>` - Landing lines
- `get_arrow_positions() -> Vec<(Point3D, f64)>` - Arrow positions and angles

#### `MLeaderStyle`
Multi-leader style.

**Constructor:**
- `new(name)` - Create new style
- `standard()` - Standard style

**Properties:**
- `leader_type: LeaderType` - Leader line type
- `arrow_type: ArrowType` - Arrow style
- `text_attachment: MLeaderTextAttachment` - Text attachment
- `landing_distance: f64` - Landing line length
- `landing_gap: f64` - Gap between landing and text

#### `ToleranceAnnotation`
GD&T tolerance.

**Properties:**
- `symbol: ToleranceSymbol` - Geometric characteristic
- `tolerance1: f64` - Primary tolerance
- `tolerance2: Option<f64>` - Secondary tolerance
- `material_condition: MaterialCondition` - MMC, LMC, RFS
- `datums: Vec<DatumReference>` - Datum references

**ToleranceSymbol Variants:**
`Position`, `Concentricity`, `Symmetry`, `Parallelism`, `Perpendicularity`, `Angularity`, `Cylindricity`, `Flatness`, `Circularity`, `Straightness`, `ProfileSurface`, `ProfileLine`, `CircularRunout`, `TotalRunout`

---

## Constraints Module (`caddy::constraints`)

### Geometric Constraints

#### `GeometricConstraint`
Geometric relationship between entities.

**Factory Methods:**
- `coincident(point1, point2)` - Points at same location
- `parallel(line1, line2)` - Lines parallel
- `perpendicular(line1, line2)` - Lines perpendicular
- `horizontal(line)` - Line horizontal
- `vertical(line)` - Line vertical
- `tangent(curve1, curve2)` - Curves tangent
- `concentric(circle1, circle2)` - Circles share center
- `equal(entity1, entity2)` - Equal size
- `fixed(entity)` - Fixed in space
- `point_on_curve(point, curve)` - Point on curve
- `midpoint(point, line)` - Point at midpoint
- `symmetric(entity1, entity2, axis)` - Symmetric about axis

**Builder Methods:**
- `with_name(name: String) -> Self`
- `with_priority(priority: i32) -> Self`
- `as_construction() -> Self`

**Methods:**
- `set_enabled(enabled: bool)` - Enable/disable
- `is_satisfied(tolerance: f64) -> bool` - Check if satisfied
- `involves_entity(entity_id: Uuid) -> bool` - Check entity involvement
- `description() -> String` - Human-readable description

#### `EntityReference`
Reference to geometric entity.

**Variants:**
- `Point(Uuid)` - Point entity
- `Line(Uuid)` - Line entity
- `Arc(Uuid)` - Arc entity
- `Circle(Uuid)` - Circle entity
- `Spline(Uuid)` - Spline entity
- `Ellipse(Uuid)` - Ellipse entity
- `PointOnEntity { entity_id, parameter }` - Point on entity
- `StartPoint(Uuid)` - Start point
- `EndPoint(Uuid)` - End point
- `CenterPoint(Uuid)` - Center point

**Methods:**
- `entity_id() -> Uuid` - Get entity ID

#### `ConstraintGroup`
Group of related constraints.

**Constructor:**
- `new(name: String)`

**Methods:**
- `add_constraint(constraint_id: Uuid)` - Add constraint
- `remove_constraint(constraint_id: Uuid)` - Remove constraint
- `contains(constraint_id: Uuid) -> bool` - Check membership

---

### Dimensional Constraints

#### `DimensionalConstraint`
Numerical constraint with value.

**Factory Methods:**
- `distance(entity1, entity2, distance)` - Distance constraint
- `horizontal_distance(entity1, entity2, distance)` - Horizontal distance
- `vertical_distance(entity1, entity2, distance)` - Vertical distance
- `angle(line1, line2, angle_radians)` - Angle constraint
- `radius(circle, radius)` - Radius constraint
- `diameter(circle, diameter)` - Diameter constraint
- `length(entity, length)` - Length constraint

**Properties:**
- `value: f64` - Target value
- `mode: ConstraintMode` - `Driving` or `Driven`
- `tolerance: Option<f64>` - Symmetrical tolerance
- `expression: Option<String>` - Parametric expression

**Builder Methods:**
- `with_name(name: String) -> Self`
- `with_mode(mode: ConstraintMode) -> Self`
- `as_driven() -> Self` - Make driven (reference only)
- `with_tolerance(tolerance: f64) -> Self`
- `with_tolerances(upper: f64, lower: f64) -> Self`
- `with_priority(priority: i32) -> Self`
- `with_expression(expression: String) -> Self`
- `as_construction() -> Self`

**Methods:**
- `set_value(value: f64)` - Update value (if driving)
- `get_effective_value(&params: &ParameterTable) -> f64` - Evaluate expression
- `is_satisfied(actual_value: f64) -> bool` - Check tolerance
- `description() -> String` - Human-readable description

#### `ParameterTable`
Named parameters for expressions.

**Constructor:**
- `new()` - Create empty table

**Methods:**
- `set(name: String, value: f64)` - Set parameter
- `get(name: &str) -> Option<f64>` - Get parameter
- `remove(name: &str) -> Option<f64>` - Remove parameter
- `contains(name: &str) -> bool` - Check exists
- `parameter_names() -> Vec<String>` - All names
- `evaluate(expression: &str) -> Result<f64, String>` - Evaluate expression

**Supported Operators:** `+`, `-`, `*`, `/`

---

### Constraint Solver

#### `ConstraintSolver`
Main constraint solver engine.

**Constructors:**
- `new()` - Create with default config
- `with_config(config: SolverConfig)` - Create with custom config

**Methods:**
- `add_geometric_constraint(constraint: GeometricConstraint)` - Add constraint
- `add_dimensional_constraint(constraint: DimensionalConstraint)` - Add constraint
- `remove_constraint(constraint_id: Uuid)` - Remove constraint
- `clear_constraints()` - Remove all constraints
- `analyze_dof() -> HashMap<Uuid, EntityDOF>` - Analyze degrees of freedom
- `is_well_constrained() -> bool` - Check if well-constrained
- `solve() -> SolverStatus` - Solve constraints
- `find_conflicts() -> Vec<(Uuid, Uuid)>` - Find conflicting constraints
- `diagnostics() -> SolverDiagnostics` - Get diagnostics
- `reset()` - Reset solver state

**Accessors:**
- `status() -> SolverStatus` - Current status
- `iterations_used() -> usize` - Iterations used
- `final_error() -> f64` - Final error

#### `SolverConfig`
Solver configuration.

**Properties:**
- `max_iterations: usize` - Max iterations (default: 100)
- `tolerance: f64` - Convergence tolerance (default: 1e-6)
- `relaxation: f64` - Relaxation factor 0-1 (default: 0.5)
- `suppress_rigid_body: bool` - Suppress rigid body motion (default: true)
- `max_step_size: f64` - Max movement per step (default: 10.0)
- `min_improvement: f64` - Min improvement to continue (default: 1e-8)

#### `SolverStatus`
Solver result status.

**Variants:**
- `NotSolved` - Not yet solved
- `Solved` - Successfully solved
- `PartiallySolved` - Some constraints satisfied
- `Failed` - Solve failed
- `OverConstrained` - Too many constraints
- `UnderConstrained` - Too few constraints
- `MaxIterationsReached` - Hit iteration limit

#### `EntityDOF`
Degrees of freedom for entity.

**Properties:**
- `entity_id: Uuid` - Entity ID
- `total_dof: usize` - Total DOF
- `constrained_dof: usize` - Constrained DOF
- `free_dof: usize` - Free DOF
- `fully_constrained: bool` - No free DOF
- `over_constrained: bool` - Too many constraints

#### `SolverDiagnostics`
Solver diagnostic information.

**Properties:**
- `status: SolverStatus` - Solver status
- `iterations: usize` - Iterations used
- `final_error: f64` - Final error
- `geometric_constraint_count: usize` - Number of geometric constraints
- `dimensional_constraint_count: usize` - Number of dimensional constraints
- `entity_count: usize` - Number of entities
- `over_constrained_entities: usize` - Over-constrained count
- `under_constrained_entities: usize` - Under-constrained count

---

## Common Types

### `Point3D`
3D point (placeholder - will integrate with core module).

**Constructor:**
- `new(x: f64, y: f64, z: f64)`

**Methods:**
- `distance_to(&other) -> f64` - Distance to another point
- `midpoint(&other) -> Point3D` - Midpoint with another point

### `Color`
RGBA color.

**Constants:**
- `WHITE`, `BLACK`, `RED`, `GREEN`, `BLUE`, `YELLOW`, `CYAN`, `MAGENTA`

**Constructors:**
- `new(r: f32, g: f32, b: f32, a: f32)` - From floats 0-1
- `rgb(r: u8, g: u8, b: u8)` - From bytes 0-255

---

## Quick Reference Card

### Most Common Operations

```rust
// Dimension
let dim = LinearDimension::horizontal(p1, p2, y, "ISO-25");
let text = dim.get_text(&style);

// Constraint
let constraint = DimensionalConstraint::distance(p1, p2, 50.0);
solver.add_dimensional_constraint(constraint);

// Solve
let status = solver.solve();
```

### Module Imports

```rust
use caddy::dimensions::{DimensionStyle, LinearDimension, Point3D};
use caddy::constraints::{ConstraintSolver, GeometricConstraint, EntityReference};
```

---

Complete API documentation is available in the source code rustdoc comments.
