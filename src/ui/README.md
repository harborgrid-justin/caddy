# CADDY UI Framework

Professional CAD user interface built with egui and winit.

## Overview

The UI module provides a complete, production-ready graphical user interface for CADDY, inspired by industry-standard CAD applications like AutoCAD. It features:

- **Professional CAD Interface**: Familiar layout with toolbars, panels, command line, and status bar
- **Responsive Design**: Smooth, GPU-accelerated rendering with real-time updates
- **Keyboard Shortcuts**: Comprehensive keyboard support for all operations
- **Command Line**: AutoCAD-style command line with autocomplete and history
- **Drawing Canvas**: Interactive canvas with grid, snap, ortho mode, and crosshair
- **Multiple Panels**: Properties, layers, and command history panels
- **Modal Dialogs**: File operations, settings, and configuration dialogs

## Architecture

### Module Structure

```
ui/
├── mod.rs              # Module exports, theme, shortcuts, UI state
├── app.rs              # Main application (CaddyApp)
├── window.rs           # Window management with winit
├── toolbar.rs          # Draw, Modify, and View toolbars
├── panel.rs            # Properties, Layers, and Command panels
├── dialog.rs           # File, Settings, Layer, and Dimension dialogs
├── canvas.rs           # Drawing canvas with mouse/keyboard handling
├── command_line.rs     # AutoCAD-style command line interface
└── status_bar.rs       # Status bar with coordinate display and mode toggles
```

### Component Hierarchy

```
CaddyApp (main application)
├── MenuBar (File, Edit, View, Draw, Modify, Help)
├── DrawToolbar (left)
│   ├── Line, Circle, Arc, Rectangle
│   ├── Polyline, Polygon, Ellipse, Spline
│   └── ... (draw tools)
├── ModifyToolbar (left)
│   ├── Move, Copy, Rotate, Scale
│   ├── Mirror, Array, Trim, Extend
│   └── ... (modify tools)
├── PropertiesPanel (right)
│   ├── General properties
│   └── Geometry properties
├── LayersPanel (right)
│   ├── Layer list with controls
│   └── New, Delete, Properties buttons
├── CommandPanel (right)
│   └── Command history
├── Canvas (center)
│   ├── Grid display
│   ├── Crosshair cursor
│   ├── Entity rendering
│   └── Context menu
├── CommandLine (bottom)
│   ├── Command input with autocomplete
│   ├── Command history (up/down arrows)
│   └── Coordinate parsing
└── StatusBar (very bottom)
    ├── Coordinate display (X, Y, Z)
    ├── Mode toggles (GRID, SNAP, ORTHO, etc.)
    └── Layer indicator
```

## Key Features

### 1. Main Application (app.rs)

The `CaddyApp` struct is the main application state that implements `eframe::App`:

```rust
use caddy::ui::CaddyApp;

// Create and run the application
eframe::run_native(
    "CADDY",
    native_options,
    Box::new(|cc| Box::new(CaddyApp::new(cc))),
);
```

**Features:**
- Menu bar with File, Edit, View, Draw, Modify, Help
- Integration with all UI components
- Event handling and command execution
- Application lifecycle management

### 2. Window Management (window.rs)

Window creation and management using winit:

```rust
use caddy::ui::window::run_app;

// Run the application
run_app()?;
```

**Features:**
- Window creation with appropriate size and icon
- Multi-window support for multi-monitor setups
- Window event handling (resize, minimize, maximize)
- MDI (Multiple Document Interface) manager

### 3. Toolbars (toolbar.rs)

Icon-based toolbars for drawing, modifying, and viewing:

**Draw Toolbar:**
- Line, Circle, Arc, Rectangle
- Polyline, Polygon, Ellipse, Spline

**Modify Toolbar:**
- Move, Copy, Rotate, Scale
- Mirror, Array, Trim, Extend
- Offset, Fillet, Chamfer, Break

**View Toolbar:**
- Zoom In, Zoom Out, Zoom Extents
- Pan, Orbit
- Top, Front, Right, 3D views

**Features:**
- Custom icon rendering (vector graphics)
- Tooltip support
- Click to execute command

### 4. Side Panels (panel.rs)

**Properties Panel:**
- Shows selected entity properties
- Editable fields (layer, color, line type, etc.)
- Geometry properties (coordinates, dimensions)

**Layers Panel:**
- Layer list with visibility, lock, freeze toggles
- Color swatches
- New layer, delete layer, layer properties
- Filter layers by name

**Command Panel:**
- Command history with timestamps
- Success/failure indicators
- Clear history button

### 5. Dialogs (dialog.rs)

**File Dialog:**
- Open, Save, Save As, Export
- File browser with filters
- Common CAD file formats (DXF, DWG, CAD)

**Settings Dialog:**
- Drawing settings (grid size, snap distance)
- Display settings (background color, crosshair size)
- File settings (auto-save, interval)

**Layer Dialog:**
- Layer properties (name, color, line type, line weight)
- New layer creation
- Edit existing layers

**Dimension Style Dialog:**
- Dimension text height, arrow size
- Extension line offset
- Style management

### 6. Drawing Canvas (canvas.rs)

Interactive drawing area with full CAD functionality:

**Features:**
- Grid display with configurable spacing
- Coordinate axes (X in red, Y in green)
- Crosshair cursor
- Mouse interaction (click, drag, scroll)
- Keyboard navigation (arrow keys, +/-)
- Pan with middle mouse or Space+Left mouse
- Zoom with mouse wheel
- Context menu (right-click)
- Coordinate display tooltip
- Selection box for multiple selection
- Snap to grid
- Ortho mode constraint

**Coordinate Systems:**
- Screen to world coordinate conversion
- World to screen coordinate conversion
- Grid snapping
- Ortho constraint

### 7. Command Line (command_line.rs)

AutoCAD-style command line interface:

**Features:**
- Command input with focus management
- Autocomplete suggestions (Tab to complete)
- Command history (Up/Down arrows)
- Coordinate parsing:
  - Cartesian: `100,200`
  - Relative: `@100,200`
  - Polar: `100<45`
  - Relative Polar: `@100<45`
- Command registry with aliases
- Error display
- Output history

**Supported Commands:**
- Drawing: LINE, CIRCLE, ARC, RECTANGLE, POLYLINE, POLYGON, ELLIPSE, SPLINE
- Modify: MOVE, COPY, ROTATE, SCALE, MIRROR, ARRAY, TRIM, EXTEND, OFFSET, FILLET, CHAMFER
- View: ZOOM, PAN
- Mode: GRID, SNAP, ORTHO
- Edit: UNDO, REDO

### 8. Status Bar (status_bar.rs)

Shows current state and allows quick toggles:

**Features:**
- Coordinate display (X, Y, Z with configurable precision)
- Mode indicators (clickable to toggle):
  - GRID (F7)
  - SNAP (F9)
  - ORTHO (F8)
  - OSNAP (F3)
  - POLAR (F10)
  - OTRACK (F11)
  - DUCS (F6)
  - DYN (F12)
  - LWT
- Current layer display (clickable to change)
- Units display (mm, cm, m, in, ft)
- Current time display

## Keyboard Shortcuts

### File Operations
- **Ctrl+N**: New document
- **Ctrl+O**: Open document
- **Ctrl+S**: Save document
- **Ctrl+Shift+S**: Save As

### Edit Operations
- **Ctrl+Z**: Undo
- **Ctrl+Y**: Redo
- **Ctrl+C**: Copy
- **Ctrl+X**: Cut
- **Ctrl+V**: Paste
- **Del**: Delete

### View Operations
- **Ctrl++**: Zoom In
- **Ctrl+-**: Zoom Out
- **Ctrl+E**: Zoom Extents
- **Space**: Pan mode

### Toggle Modes
- **F7**: Grid
- **F8**: Ortho
- **F9**: Snap
- **F3**: Object Snap
- **F10**: Polar Tracking
- **F11**: Object Snap Tracking
- **F12**: Dynamic Input

### Drawing Commands
- **L**: Line
- **C**: Circle
- **A**: Arc
- **R**: Rectangle

### Navigation
- **Arrow Keys**: Pan view
- **+/-**: Zoom in/out
- **Esc**: Cancel command

## Theme and Styling

The UI uses a professional dark theme optimized for CAD work:

```rust
use caddy::ui::theme::CaddyTheme;

// Apply dark theme
CaddyTheme::apply(ctx, true);

// Apply light theme
CaddyTheme::apply(ctx, false);
```

**Color Scheme:**
- Background: Dark gray (20, 20, 20)
- Panels: Medium gray (40, 40, 40)
- Text: Light gray (220, 220, 220)
- Grid: Dark gray (60, 60, 60)
- X Axis: Red (255, 0, 0)
- Y Axis: Green (0, 255, 0)
- Z Axis: Blue (0, 0, 255)
- Selection: Cyan (0, 180, 255)
- Highlight: Orange (255, 200, 0)

## UI State Management

The `UiState` struct maintains global UI state:

```rust
pub struct UiState {
    pub mode: InteractionMode,
    pub show_grid: bool,
    pub snap_to_grid: bool,
    pub ortho_mode: bool,
    pub show_layers: bool,
    pub show_properties: bool,
    pub show_command_history: bool,
    pub dark_theme: bool,
    pub current_layer: String,
    pub cursor_pos: (f64, f64),
}
```

## Integration with Other Modules

The UI module integrates with other CADDY modules:

```rust
// Hypothetical integration (to be implemented by other agents)
use crate::core::*;        // Math, primitives, precision
use crate::rendering::*;   // Renderer, camera, viewport
use crate::commands::*;    // Command processor, history
use crate::layers::*;      // Layer manager
use crate::geometry::*;    // Geometric entities
```

## Usage Example

```rust
use caddy::ui::window::run_app;

fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::init();

    // Run the CADDY application
    run_app()?;

    Ok(())
}
```

## Best Practices

1. **Responsive UI**: All UI updates happen at 60 FPS
2. **Keyboard First**: All operations accessible via keyboard
3. **Professional Look**: Follows CAD industry standards
4. **Accessibility**: Clear visual feedback, tooltips, proper contrast
5. **Performance**: Efficient rendering, minimal allocations
6. **Error Handling**: Clear error messages, graceful degradation

## Future Enhancements

- [ ] Customizable toolbars and panels
- [ ] Ribbon interface option
- [ ] Workspace presets
- [ ] Quick access toolbar
- [ ] Status bar customization
- [ ] Multiple command line modes
- [ ] Gesture support for touch devices
- [ ] Internationalization (i18n)
- [ ] High DPI display support
- [ ] Theme editor
- [ ] Macro recording and playback

## Testing

The UI can be tested manually by running:

```bash
cargo run
```

For automated testing, use the egui testing framework (to be implemented).

## License

MIT License - See LICENSE file for details.
