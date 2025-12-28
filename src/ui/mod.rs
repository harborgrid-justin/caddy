/// UI module for CADDY - Enterprise CAD System
///
/// This module provides a professional CAD user interface using egui and winit.
/// It implements:
/// - Main application window and event loop
/// - Drawing toolbars (line, arc, circle, rectangle, etc.)
/// - Modify toolbars (move, copy, rotate, scale, etc.)
/// - Side panels (properties, layers, command history)
/// - Command line interface (AutoCAD-style)
/// - Drawing canvas with mouse/keyboard input
/// - Status bar with coordinate display
/// - Modal dialogs (file operations, settings, etc.)
pub mod app;
pub mod window;
pub mod toolbar;
pub mod panel;
pub mod dialog;
pub mod canvas;
pub mod command_line;
pub mod status_bar;

// Re-export main types
pub use app::CaddyApp;
pub use window::MainWindow;
pub use toolbar::{Toolbar, DrawToolbar, ModifyToolbar, ViewToolbar, ToolbarPosition};
pub use panel::{PropertiesPanel, LayersPanel, CommandPanel, Panel};
pub use dialog::{FileDialog, SettingsDialog, LayerDialog, DimensionStyleDialog, Dialog};
pub use canvas::Canvas;
pub use command_line::CommandLine;
pub use status_bar::StatusBar;

/// UI theme and styling
pub mod theme {
    use egui::{Color32, Style, Visuals, Stroke};

    /// Professional CAD color scheme
    pub struct CaddyTheme;

    impl CaddyTheme {
        /// Get dark theme (default for CAD)
        pub fn dark() -> Visuals {
            Visuals {
                dark_mode: true,
                override_text_color: Some(Color32::from_rgb(220, 220, 220)),
                window_fill: Color32::from_rgb(30, 30, 30),
                panel_fill: Color32::from_rgb(40, 40, 40),
                faint_bg_color: Color32::from_rgb(50, 50, 50),
                extreme_bg_color: Color32::from_rgb(20, 20, 20),
                code_bg_color: Color32::from_rgb(45, 45, 45),
                window_stroke: Stroke::new(1.0, Color32::from_rgb(60, 60, 60)),
                ..Default::default()
            }
        }

        /// Get light theme (optional)
        pub fn light() -> Visuals {
            Visuals {
                dark_mode: false,
                override_text_color: Some(Color32::from_rgb(40, 40, 40)),
                window_fill: Color32::from_rgb(240, 240, 240),
                panel_fill: Color32::from_rgb(250, 250, 250),
                faint_bg_color: Color32::from_rgb(230, 230, 230),
                extreme_bg_color: Color32::from_rgb(255, 255, 255),
                code_bg_color: Color32::from_rgb(245, 245, 245),
                window_stroke: Stroke::new(1.0, Color32::from_rgb(200, 200, 200)),
                ..Default::default()
            }
        }

        /// Apply theme to egui context
        pub fn apply(ctx: &egui::Context, dark: bool) {
            let mut style = Style::default();
            style.visuals = if dark {
                Self::dark()
            } else {
                Self::light()
            };
            ctx.set_style(style);
        }
    }

    /// CAD-specific colors
    pub const GRID_COLOR: Color32 = Color32::from_rgb(60, 60, 60);
    pub const AXIS_X_COLOR: Color32 = Color32::from_rgb(255, 0, 0);
    pub const AXIS_Y_COLOR: Color32 = Color32::from_rgb(0, 255, 0);
    pub const AXIS_Z_COLOR: Color32 = Color32::from_rgb(0, 0, 255);
    pub const SELECTION_COLOR: Color32 = Color32::from_rgb(0, 180, 255);
    pub const HIGHLIGHT_COLOR: Color32 = Color32::from_rgb(255, 200, 0);
    pub const CROSSHAIR_COLOR: Color32 = Color32::from_rgb(200, 200, 200);
}

/// Keyboard shortcut handling
pub mod shortcuts {
    use winit::keyboard::{KeyCode, ModifiersState};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Shortcut {
        pub key: KeyCode,
        pub modifiers: ModifiersState,
    }

    impl Shortcut {
        pub fn new(key: KeyCode, modifiers: ModifiersState) -> Self {
            Self { key, modifiers }
        }

        pub fn ctrl(key: KeyCode) -> Self {
            Self::new(key, ModifiersState::CONTROL)
        }

        pub fn shift(key: KeyCode) -> Self {
            Self::new(key, ModifiersState::SHIFT)
        }

        pub fn alt(key: KeyCode) -> Self {
            Self::new(key, ModifiersState::ALT)
        }

        pub fn none(key: KeyCode) -> Self {
            Self::new(key, ModifiersState::empty())
        }
    }

    impl From<&Shortcut> for egui::KeyboardShortcut {
        fn from(shortcut: &Shortcut) -> Self {
            let key = winit_key_to_egui(shortcut.key);
            let modifiers = egui::Modifiers {
                alt: shortcut.modifiers.contains(ModifiersState::ALT),
                ctrl: shortcut.modifiers.contains(ModifiersState::CONTROL),
                shift: shortcut.modifiers.contains(ModifiersState::SHIFT),
                mac_cmd: false,
                command: shortcut.modifiers.contains(ModifiersState::CONTROL),
            };
            egui::KeyboardShortcut::new(modifiers, key)
        }
    }

    impl From<Shortcut> for egui::KeyboardShortcut {
        fn from(shortcut: Shortcut) -> Self {
            (&shortcut).into()
        }
    }

    /// Convert winit KeyCode to egui Key
    fn winit_key_to_egui(key: KeyCode) -> egui::Key {
        match key {
            KeyCode::KeyA => egui::Key::A,
            KeyCode::KeyB => egui::Key::B,
            KeyCode::KeyC => egui::Key::C,
            KeyCode::KeyD => egui::Key::D,
            KeyCode::KeyE => egui::Key::E,
            KeyCode::KeyF => egui::Key::F,
            KeyCode::KeyG => egui::Key::G,
            KeyCode::KeyH => egui::Key::H,
            KeyCode::KeyI => egui::Key::I,
            KeyCode::KeyJ => egui::Key::J,
            KeyCode::KeyK => egui::Key::K,
            KeyCode::KeyL => egui::Key::L,
            KeyCode::KeyM => egui::Key::M,
            KeyCode::KeyN => egui::Key::N,
            KeyCode::KeyO => egui::Key::O,
            KeyCode::KeyP => egui::Key::P,
            KeyCode::KeyQ => egui::Key::Q,
            KeyCode::KeyR => egui::Key::R,
            KeyCode::KeyS => egui::Key::S,
            KeyCode::KeyT => egui::Key::T,
            KeyCode::KeyU => egui::Key::U,
            KeyCode::KeyV => egui::Key::V,
            KeyCode::KeyW => egui::Key::W,
            KeyCode::KeyX => egui::Key::X,
            KeyCode::KeyY => egui::Key::Y,
            KeyCode::KeyZ => egui::Key::Z,
            KeyCode::Delete => egui::Key::Delete,
            KeyCode::Space => egui::Key::Space,
            KeyCode::Equal => egui::Key::Plus,
            KeyCode::Minus => egui::Key::Minus,
            KeyCode::F8 => egui::Key::F8,
            KeyCode::F9 => egui::Key::F9,
            KeyCode::F7 => egui::Key::F7,
            _ => egui::Key::Space, // Default fallback
        }
    }

    /// Standard CAD shortcuts
    pub struct StandardShortcuts;

    impl StandardShortcuts {
        // File operations
        pub const NEW: Shortcut = Shortcut { key: KeyCode::KeyN, modifiers: ModifiersState::CONTROL };
        pub const OPEN: Shortcut = Shortcut { key: KeyCode::KeyO, modifiers: ModifiersState::CONTROL };
        pub const SAVE: Shortcut = Shortcut { key: KeyCode::KeyS, modifiers: ModifiersState::CONTROL };
        pub const SAVE_AS: Shortcut = Shortcut {
            key: KeyCode::KeyS,
            modifiers: ModifiersState::CONTROL.union(ModifiersState::SHIFT)
        };

        // Edit operations
        pub const UNDO: Shortcut = Shortcut { key: KeyCode::KeyZ, modifiers: ModifiersState::CONTROL };
        pub const REDO: Shortcut = Shortcut { key: KeyCode::KeyY, modifiers: ModifiersState::CONTROL };
        pub const COPY: Shortcut = Shortcut { key: KeyCode::KeyC, modifiers: ModifiersState::CONTROL };
        pub const CUT: Shortcut = Shortcut { key: KeyCode::KeyX, modifiers: ModifiersState::CONTROL };
        pub const PASTE: Shortcut = Shortcut { key: KeyCode::KeyV, modifiers: ModifiersState::CONTROL };
        pub const DELETE: Shortcut = Shortcut { key: KeyCode::Delete, modifiers: ModifiersState::empty() };

        // View operations
        pub const ZOOM_IN: Shortcut = Shortcut { key: KeyCode::Equal, modifiers: ModifiersState::CONTROL };
        pub const ZOOM_OUT: Shortcut = Shortcut { key: KeyCode::Minus, modifiers: ModifiersState::CONTROL };
        pub const ZOOM_EXTENTS: Shortcut = Shortcut { key: KeyCode::KeyE, modifiers: ModifiersState::CONTROL };
        pub const PAN: Shortcut = Shortcut { key: KeyCode::Space, modifiers: ModifiersState::empty() };

        // Toggle modes
        pub const ORTHO: Shortcut = Shortcut { key: KeyCode::F8, modifiers: ModifiersState::empty() };
        pub const SNAP: Shortcut = Shortcut { key: KeyCode::F9, modifiers: ModifiersState::empty() };
        pub const GRID: Shortcut = Shortcut { key: KeyCode::F7, modifiers: ModifiersState::empty() };

        // Drawing commands (can be activated by key)
        pub const LINE: Shortcut = Shortcut { key: KeyCode::KeyL, modifiers: ModifiersState::empty() };
        pub const CIRCLE: Shortcut = Shortcut { key: KeyCode::KeyC, modifiers: ModifiersState::empty() };
        pub const ARC: Shortcut = Shortcut { key: KeyCode::KeyA, modifiers: ModifiersState::empty() };
        pub const RECTANGLE: Shortcut = Shortcut { key: KeyCode::KeyR, modifiers: ModifiersState::empty() };
    }
}

/// Mouse interaction modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionMode {
    /// Normal selection and navigation
    Normal,
    /// Drawing mode (waiting for points)
    Drawing,
    /// Editing mode (modifying entities)
    Editing,
    /// Pan mode (dragging view)
    Pan,
    /// Zoom mode
    Zoom,
    /// Rotate mode (3D)
    Rotate,
}

/// UI state shared across components
#[derive(Debug, Clone)]
pub struct UiState {
    /// Current interaction mode
    pub mode: InteractionMode,
    /// Show grid
    pub show_grid: bool,
    /// Grid snap enabled
    pub snap_to_grid: bool,
    /// Ortho mode (restrict to horizontal/vertical)
    pub ortho_mode: bool,
    /// Show layer panel
    pub show_layers: bool,
    /// Show properties panel
    pub show_properties: bool,
    /// Show command panel
    pub show_command_history: bool,
    /// Dark theme enabled
    pub dark_theme: bool,
    /// Current layer name
    pub current_layer: String,
    /// Cursor position in world coordinates
    pub cursor_pos: (f64, f64),
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            mode: InteractionMode::Normal,
            show_grid: true,
            snap_to_grid: false,
            ortho_mode: false,
            show_layers: true,
            show_properties: true,
            show_command_history: true,
            dark_theme: true,
            current_layer: "0".to_string(),
            cursor_pos: (0.0, 0.0),
        }
    }
}
