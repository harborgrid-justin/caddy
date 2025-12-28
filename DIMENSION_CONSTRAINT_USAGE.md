# CADDY Dimensions & Constraints - Usage Guide

## Quick Start Examples

### Creating Dimensions

#### 1. Linear Dimensions

```rust
use caddy::dimensions::{LinearDimension, DimensionStyle, Point3D};

// Create dimension style
let style = DimensionStyle::iso();

// Create horizontal dimension
let p1 = Point3D::new(0.0, 0.0, 0.0);
let p2 = Point3D::new(100.0, 0.0, 0.0);
let dim = LinearDimension::horizontal(p1, p2, 20.0, "ISO-25");

// Get formatted text
let text = dim.get_text(&style); // "100"

// Make it associative
let mut dim = dim;
dim.associate_with(vec![entity1_id, entity2_id]);
```

#### 2. Angular Dimensions

```rust
use caddy::dimensions::{AngularDimension, Point3D};

let center = Point3D::new(0.0, 0.0, 0.0);
let line1_point = Point3D::new(10.0, 0.0, 0.0);
let line2_point = Point3D::new(0.0, 10.0, 0.0);
let arc_point = Point3D::new(7.0, 7.0, 0.0);

let dim = AngularDimension::new(center, line1_point, line2_point, arc_point, "ISO-25");
let angle = dim.calculate_angle(); // Returns radians
```

#### 3. Radial Dimensions

```rust
use caddy::dimensions::{RadiusDimension, DiameterDimension};

// Radius dimension
let center = Point3D::new(50.0, 50.0, 0.0);
let radius = 25.0;
let angle = 0.0; // 0 degrees
let rad_dim = RadiusDimension::from_circle(center, radius, angle, "ISO-25");

// Diameter dimension
let dia_dim = DiameterDimension::from_circle(center, radius, angle, "ISO-25");
let text = dia_dim.get_text(&style); // "Ø50"
```

#### 4. Leaders and Annotations

```rust
use caddy::dimensions::{MultiLeader, MText, MLeaderContent, Point3D};

// Create multi-line text
let text_pos = Point3D::new(50.0, 50.0, 0.0);
let mtext = MText::new("This is a note\nabout the feature", text_pos, 2.5, "Standard")
    .with_width(100.0); // Word wrap at 100 units

// Create leader line
let leader_line = vec![
    Point3D::new(0.0, 0.0, 0.0),   // Arrow point
    Point3D::new(30.0, 30.0, 0.0), // Bend point
    Point3D::new(45.0, 45.0, 0.0), // Landing point
];

// Create multi-leader
let mleader = MultiLeader::new(
    leader_line,
    MLeaderContent::MText(mtext),
    text_pos,
    "Standard"
);
```

### Creating Dimension Styles

```rust
use caddy::dimensions::{DimensionStyle, ArrowType, UnitFormat, ToleranceFormat};

// Start with a standard
let mut style = DimensionStyle::iso();

// Customize
style.text_height = 3.5;
style.arrow_type_1 = ArrowType::ClosedFilled;
style.arrow_size = 3.0;
style.precision = 3; // 3 decimal places
style.unit_format = UnitFormat::Architectural;
style.tolerance_format = ToleranceFormat::Deviation;
style.tolerance_upper = 0.1;
style.tolerance_lower = 0.05;

// Enable alternate units (show metric and imperial)
style.alt_units_enabled = true;
style.alt_units_scale = 25.4; // mm to inches
style.alt_units_prefix = "[".to_string();
style.alt_units_suffix = " mm]".to_string();

// Use the style
let dim = LinearDimension::horizontal(p1, p2, 20.0, "Custom");
let text = dim.get_text(&style); // "100.000 ±0.1 -0.05 [2540.000 mm]"
```

### Working with Constraints

#### 1. Geometric Constraints

```rust
use caddy::constraints::{GeometricConstraint, EntityReference};
use uuid::Uuid;

// Create entity references
let line1 = EntityReference::Line(Uuid::new_v4());
let line2 = EntityReference::Line(Uuid::new_v4());
let point1 = EntityReference::Point(Uuid::new_v4());
let point2 = EntityReference::Point(Uuid::new_v4());

// Make lines parallel
let parallel = GeometricConstraint::parallel(line1.clone(), line2.clone())
    .with_name("Parallel Edges")
    .with_priority(10);

// Make line horizontal
let horizontal = GeometricConstraint::horizontal(line1);

// Make points coincident
let coincident = GeometricConstraint::coincident(point1, point2);
```

#### 2. Dimensional Constraints

```rust
use caddy::constraints::{DimensionalConstraint, ConstraintMode};

// Create distance constraint
let p1 = EntityReference::Point(Uuid::new_v4());
let p2 = EntityReference::Point(Uuid::new_v4());

let distance = DimensionalConstraint::distance(p1, p2, 50.0)
    .with_name("Width")
    .with_tolerance(0.1);

// Create driving constraint (controls geometry)
let driving = DimensionalConstraint::radius(circle, 25.0)
    .with_mode(ConstraintMode::Driving);

// Create driven constraint (reference only)
let driven = DimensionalConstraint::length(line, 100.0)
    .as_driven(); // Just reports the value, doesn't change geometry
```

#### 3. Parametric Constraints

```rust
use caddy::constraints::{DimensionalConstraint, ParameterTable};

// Set up parameter table
let mut params = ParameterTable::new();
params.set("width", 100.0);
params.set("height", 50.0);

// Create constraint with expression
let constraint = DimensionalConstraint::distance(p1, p2, 0.0)
    .with_expression("width * 2")
    .with_name("Length");

// Evaluate expression
let value = params.evaluate("width + height").unwrap(); // 150.0
let complex = params.evaluate("width * 2 - height / 2").unwrap(); // 175.0
```

#### 4. Using the Constraint Solver

```rust
use caddy::constraints::{ConstraintSolver, SolverConfig, SolverStatus};

// Create solver with custom config
let config = SolverConfig {
    max_iterations: 100,
    tolerance: 1e-6,
    relaxation: 0.5,
    max_step_size: 10.0,
    ..Default::default()
};

let mut solver = ConstraintSolver::with_config(config);

// Add constraints
solver.add_geometric_constraint(parallel_constraint);
solver.add_dimensional_constraint(distance_constraint);

// Analyze degrees of freedom
let dof_map = solver.analyze_dof();
for (entity_id, dof) in &dof_map {
    println!("Entity {:?}: {} free DOF, {} constrained",
             entity_id, dof.free_dof, dof.constrained_dof);
}

// Check if well-constrained
if !solver.is_well_constrained() {
    println!("Warning: System is over or under-constrained!");
}

// Solve
let status = solver.solve();
match status {
    SolverStatus::Solved => {
        println!("Solved in {} iterations", solver.iterations_used());
        println!("Final error: {}", solver.final_error());
    }
    SolverStatus::OverConstrained => {
        println!("System is over-constrained!");
        let conflicts = solver.find_conflicts();
        println!("Found {} conflicts", conflicts.len());
    }
    _ => println!("Solve failed: {:?}", status),
}

// Get diagnostics
let diag = solver.diagnostics();
println!("Geometric constraints: {}", diag.geometric_constraint_count);
println!("Dimensional constraints: {}", diag.dimensional_constraint_count);
println!("Over-constrained entities: {}", diag.over_constrained_entities);
```

## Common Workflows

### 1. Creating a Fully Dimensioned Rectangle

```rust
// Create rectangle points
let p1 = Point3D::new(0.0, 0.0, 0.0);
let p2 = Point3D::new(100.0, 0.0, 0.0);
let p3 = Point3D::new(100.0, 50.0, 0.0);
let p4 = Point3D::new(0.0, 50.0, 0.0);

// Dimension width
let width_dim = LinearDimension::horizontal(p1, p2, -10.0, "ISO-25");

// Dimension height
let height_dim = LinearDimension::vertical(p2, p3, 110.0, "ISO-25");

// Add note with leader
let note_pos = Point3D::new(120.0, 25.0, 0.0);
let note_text = MText::new("100x50 Rectangle", note_pos, 2.5, "Standard");
let leader_points = vec![
    Point3D::new(100.0, 25.0, 0.0),
    Point3D::new(110.0, 25.0, 0.0),
];
let leader = MultiLeader::new(leader_points, MLeaderContent::MText(note_text), note_pos, "Standard");
```

### 2. Parametric Design Pattern

```rust
// Set up parameters
let mut params = ParameterTable::new();
params.set("base_width", 100.0);
params.set("base_height", 50.0);
params.set("offset", 10.0);

// Create parametric constraints
let width = DimensionalConstraint::horizontal_distance(p1_ref, p2_ref, 0.0)
    .with_expression("base_width")
    .with_name("Width");

let height = DimensionalConstraint::vertical_distance(p2_ref, p3_ref, 0.0)
    .with_expression("base_height")
    .with_name("Height");

let margin = DimensionalConstraint::distance(p1_ref, inner_p1_ref, 0.0)
    .with_expression("offset")
    .with_name("Margin");

// Now changing base_width will update all related dimensions
params.set("base_width", 150.0);
// Re-solve constraints...
```

### 3. GD&T Annotation

```rust
use caddy::dimensions::{ToleranceAnnotation, ToleranceSymbol, MaterialCondition, DatumReference};

// Create geometric tolerance
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

// Attach to leader
let leader_annotation = LeaderAnnotation::Tolerance(tolerance);
let leader = Leader::new(vertices, leader_annotation, "ISO-25");
```

## Advanced Features

### Baseline Dimensions

```rust
use caddy::dimensions::BaselineDimension;

// Create base dimension
let base_dim_id = Uuid::new_v4();
let mut baseline = BaselineDimension::new(base_dim_id, 10.0, "ISO-25");

// Add more dimension points
baseline.add_point(Point3D::new(50.0, 0.0, 0.0));
baseline.add_point(Point3D::new(100.0, 0.0, 0.0));
baseline.add_point(Point3D::new(150.0, 0.0, 0.0));

// All dimensions are spaced 10 units apart vertically
```

### Continue Dimensions

```rust
use caddy::dimensions::ContinueDimension;

let prev_dim_id = Uuid::new_v4();
let mut continue_dim = ContinueDimension::new(prev_dim_id, "ISO-25");

// Chain dimensions end-to-end
continue_dim.add_point(Point3D::new(50.0, 0.0, 0.0));
continue_dim.add_point(Point3D::new(100.0, 0.0, 0.0));
continue_dim.add_point(Point3D::new(125.0, 0.0, 0.0));
```

### Field Text (Dynamic Content)

```rust
use caddy::dimensions::{FieldText, TextField};

let mut field_text = FieldText::new(
    "Drawing: {FILENAME}\nDate: {DATE}\nSheet: {SHEET}",
    Point3D::new(0.0, 0.0, 0.0),
    2.5,
    "Standard"
);

field_text.add_field("{FILENAME}", TextField::FileName);
field_text.add_field("{DATE}", TextField::Date);
field_text.add_field("{SHEET}", TextField::SheetNumber);

// Evaluate to get actual text
let final_text = field_text.evaluate();
// "Drawing: drawing.cad\nDate: 2025-12-28\nSheet: 1"
```

## Best Practices

1. **Always use styles** - Don't hardcode dimension appearance
2. **Name your constraints** - Makes debugging easier
3. **Check solver status** - Don't assume constraints solved
4. **Use priorities** - Order constraint solving for better results
5. **Group related constraints** - Easier management
6. **Use driven dimensions** - For reference measurements
7. **Set tolerances** - Manufacturing requires precision
8. **Enable associativity** - Dimensions update with geometry

## Error Handling

```rust
// Check if constraint is satisfied
if !constraint.is_satisfied(actual_value) {
    eprintln!("Constraint '{}' not satisfied!", constraint.description());
}

// Check for conflicts
let conflicts = solver.find_conflicts();
if !conflicts.is_empty() {
    eprintln!("Found {} constraint conflicts!", conflicts.len());
    for (c1, c2) in conflicts {
        eprintln!("Conflict between {:?} and {:?}", c1, c2);
    }
}

// Handle solver failure
match solver.solve() {
    SolverStatus::Solved => { /* Success */ },
    SolverStatus::OverConstrained => {
        eprintln!("System over-constrained!");
        let diag = solver.diagnostics();
        eprintln!("{} entities over-constrained", diag.over_constrained_entities);
    },
    SolverStatus::MaxIterationsReached => {
        eprintln!("Solver didn't converge in {} iterations", solver.iterations_used());
        eprintln!("Final error: {}", solver.final_error());
    },
    _ => eprintln!("Solve failed"),
}
```

---

This guide covers the essential usage patterns for the CADDY dimension and constraint system. For more details, refer to the rustdoc documentation in the source files.
