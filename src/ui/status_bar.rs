/// Status bar widget for CADDY
///
/// Shows coordinate display, snap mode indicators, grid toggle,
/// ortho toggle, and current layer display.
use egui::{Ui, Color32, RichText};
use super::UiState;

/// Status bar widget
pub struct StatusBar {
    /// Show coordinate precision
    coordinate_precision: usize,
    /// Units display
    units: Units,
    /// Show mode indicators
    show_mode_indicators: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Units {
    Millimeters,
    Centimeters,
    Meters,
    Inches,
    Feet,
}

impl Units {
    fn display_name(&self) -> &str {
        match self {
            Units::Millimeters => "mm",
            Units::Centimeters => "cm",
            Units::Meters => "m",
            Units::Inches => "in",
            Units::Feet => "ft",
        }
    }
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            coordinate_precision: 2,
            units: Units::Millimeters,
            show_mode_indicators: true,
        }
    }

    /// Set coordinate precision
    pub fn set_precision(&mut self, precision: usize) {
        self.coordinate_precision = precision.min(6);
    }

    /// Set units
    pub fn set_units(&mut self, units: Units) {
        self.units = units;
    }

    /// Show status bar
    pub fn show(&mut self, ui: &mut Ui, state: &mut UiState) {
        ui.horizontal(|ui| {
            // Coordinate display
            self.show_coordinates(ui, state);

            ui.separator();

            // Mode indicators
            if self.show_mode_indicators {
                self.show_mode_indicators_ui(ui, state);
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Units display
                ui.label(RichText::new(self.units.display_name())
                    .color(Color32::from_rgb(150, 150, 150))
                    .monospace());

                ui.separator();

                // Current layer
                self.show_layer_indicator(ui, state);

                ui.separator();

                // Time/status
                self.show_status_info(ui);
            });
        });
    }

    /// Show coordinate display
    fn show_coordinates(&self, ui: &mut Ui, state: &UiState) {
        let (x, y) = state.cursor_pos;

        // X coordinate
        ui.label(RichText::new("X:")
            .color(Color32::from_rgb(255, 100, 100))
            .strong());

        let x_text = format!("{:.*}", self.coordinate_precision, x);
        ui.label(RichText::new(x_text)
            .color(Color32::from_rgb(200, 200, 200))
            .monospace());

        ui.add_space(10.0);

        // Y coordinate
        ui.label(RichText::new("Y:")
            .color(Color32::from_rgb(100, 255, 100))
            .strong());

        let y_text = format!("{:.*}", self.coordinate_precision, y);
        ui.label(RichText::new(y_text)
            .color(Color32::from_rgb(200, 200, 200))
            .monospace());

        ui.add_space(10.0);

        // Z coordinate (for 3D, currently always 0)
        ui.label(RichText::new("Z:")
            .color(Color32::from_rgb(100, 100, 255))
            .strong());

        ui.label(RichText::new(format!("{:.*}", self.coordinate_precision, 0.0))
            .color(Color32::from_rgb(200, 200, 200))
            .monospace());
    }

    /// Show mode indicators
    fn show_mode_indicators_ui(&self, ui: &mut Ui, state: &mut UiState) {
        // Grid toggle
        let grid_color = if state.show_grid {
            Color32::from_rgb(100, 200, 100)
        } else {
            Color32::from_rgb(100, 100, 100)
        };

        let grid_response = ui.add(
            egui::Button::new(
                RichText::new("GRID")
                    .color(grid_color)
                    .monospace()
                    .size(11.0)
            )
            .frame(false)
            .small()
        );

        if grid_response.clicked() {
            state.show_grid = !state.show_grid;
        }

        if grid_response.hovered() {
            egui::show_tooltip(ui.ctx(), egui::Id::new("grid_tooltip"), |ui| {
                ui.label("Toggle Grid (F7)");
            });
        }

        ui.separator();

        // Snap toggle
        let snap_color = if state.snap_to_grid {
            Color32::from_rgb(100, 200, 100)
        } else {
            Color32::from_rgb(100, 100, 100)
        };

        let snap_response = ui.add(
            egui::Button::new(
                RichText::new("SNAP")
                    .color(snap_color)
                    .monospace()
                    .size(11.0)
            )
            .frame(false)
            .small()
        );

        if snap_response.clicked() {
            state.snap_to_grid = !state.snap_to_grid;
        }

        if snap_response.hovered() {
            egui::show_tooltip(ui.ctx(), egui::Id::new("snap_tooltip"), |ui| {
                ui.label("Toggle Snap (F9)");
            });
        }

        ui.separator();

        // Ortho toggle
        let ortho_color = if state.ortho_mode {
            Color32::from_rgb(100, 200, 100)
        } else {
            Color32::from_rgb(100, 100, 100)
        };

        let ortho_response = ui.add(
            egui::Button::new(
                RichText::new("ORTHO")
                    .color(ortho_color)
                    .monospace()
                    .size(11.0)
            )
            .frame(false)
            .small()
        );

        if ortho_response.clicked() {
            state.ortho_mode = !state.ortho_mode;
        }

        if ortho_response.hovered() {
            egui::show_tooltip(ui.ctx(), egui::Id::new("ortho_tooltip"), |ui| {
                ui.label("Toggle Ortho Mode (F8)");
            });
        }

        ui.separator();

        // Object snap (OSNAP)
        let osnap_color = Color32::from_rgb(100, 100, 100);

        let osnap_response = ui.add(
            egui::Button::new(
                RichText::new("OSNAP")
                    .color(osnap_color)
                    .monospace()
                    .size(11.0)
            )
            .frame(false)
            .small()
        );

        if osnap_response.clicked() {
            // Toggle object snap
        }

        if osnap_response.hovered() {
            egui::show_tooltip(ui.ctx(), egui::Id::new("osnap_tooltip"), |ui| {
                ui.label("Object Snap Settings (F3)");
            });
        }

        ui.separator();

        // Polar tracking
        let polar_color = Color32::from_rgb(100, 100, 100);

        let polar_response = ui.add(
            egui::Button::new(
                RichText::new("POLAR")
                    .color(polar_color)
                    .monospace()
                    .size(11.0)
            )
            .frame(false)
            .small()
        );

        if polar_response.clicked() {
            // Toggle polar tracking
        }

        if polar_response.hovered() {
            egui::show_tooltip(ui.ctx(), egui::Id::new("polar_tooltip"), |ui| {
                ui.label("Polar Tracking (F10)");
            });
        }

        ui.separator();

        // Object snap tracking
        let otrack_color = Color32::from_rgb(100, 100, 100);

        let otrack_response = ui.add(
            egui::Button::new(
                RichText::new("OTRACK")
                    .color(otrack_color)
                    .monospace()
                    .size(11.0)
            )
            .frame(false)
            .small()
        );

        if otrack_response.clicked() {
            // Toggle object snap tracking
        }

        if otrack_response.hovered() {
            egui::show_tooltip(ui.ctx(), egui::Id::new("otrack_tooltip"), |ui| {
                ui.label("Object Snap Tracking (F11)");
            });
        }

        ui.separator();

        // Dynamic UCS
        let ducs_color = Color32::from_rgb(100, 100, 100);

        let ducs_response = ui.add(
            egui::Button::new(
                RichText::new("DUCS")
                    .color(ducs_color)
                    .monospace()
                    .size(11.0)
            )
            .frame(false)
            .small()
        );

        if ducs_response.clicked() {
            // Toggle dynamic UCS
        }

        if ducs_response.hovered() {
            egui::show_tooltip(ui.ctx(), egui::Id::new("ducs_tooltip"), |ui| {
                ui.label("Dynamic UCS (F6)");
            });
        }

        ui.separator();

        // Dynamic Input
        let dyn_color = Color32::from_rgb(100, 100, 100);

        let dyn_response = ui.add(
            egui::Button::new(
                RichText::new("DYN")
                    .color(dyn_color)
                    .monospace()
                    .size(11.0)
            )
            .frame(false)
            .small()
        );

        if dyn_response.clicked() {
            // Toggle dynamic input
        }

        if dyn_response.hovered() {
            egui::show_tooltip(ui.ctx(), egui::Id::new("dyn_tooltip"), |ui| {
                ui.label("Dynamic Input (F12)");
            });
        }

        ui.separator();

        // Line weight display
        let lwt_color = Color32::from_rgb(100, 100, 100);

        let lwt_response = ui.add(
            egui::Button::new(
                RichText::new("LWT")
                    .color(lwt_color)
                    .monospace()
                    .size(11.0)
            )
            .frame(false)
            .small()
        );

        if lwt_response.clicked() {
            // Toggle line weight display
        }

        if lwt_response.hovered() {
            egui::show_tooltip(ui.ctx(), egui::Id::new("lwt_tooltip"), |ui| {
                ui.label("Show/Hide Line Weight");
            });
        }
    }

    /// Show current layer indicator
    fn show_layer_indicator(&self, ui: &mut Ui, state: &UiState) {
        ui.label(RichText::new("Layer:")
            .color(Color32::from_rgb(150, 150, 150))
            .size(11.0));

        let layer_color = Color32::from_rgb(255, 255, 255);

        let layer_response = ui.add(
            egui::Button::new(
                RichText::new(&state.current_layer)
                    .color(layer_color)
                    .monospace()
                    .strong()
            )
            .frame(false)
            .small()
        );

        if layer_response.clicked() {
            // Open layer dialog
            log::info!("Layer button clicked");
        }

        if layer_response.hovered() {
            egui::show_tooltip(ui.ctx(), egui::Id::new("layer_tooltip"), |ui| {
                ui.label(format!("Current Layer: {}", state.current_layer));
                ui.label("Click to change layer");
            });
        }
    }

    /// Show status information (time, etc.)
    fn show_status_info(&self, ui: &mut Ui) {
        // Show current time
        let time = Self::current_time();
        ui.label(RichText::new(time)
            .color(Color32::from_rgb(150, 150, 150))
            .monospace()
            .size(11.0));
    }

    /// Get current time as string
    fn current_time() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};

        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap();

        let secs = duration.as_secs();
        let hours = (secs / 3600) % 24;
        let minutes = (secs / 60) % 60;
        let seconds = secs % 60;

        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

/// Status bar button for mode toggles
struct StatusButton {
    label: String,
    active: bool,
    tooltip: String,
    shortcut: Option<String>,
}

impl StatusButton {
    fn new(label: impl Into<String>, tooltip: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            active: false,
            tooltip: tooltip.into(),
            shortcut: None,
        }
    }

    fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    fn show(&mut self, ui: &mut Ui) -> egui::Response {
        let color = if self.active {
            Color32::from_rgb(100, 200, 100)
        } else {
            Color32::from_rgb(100, 100, 100)
        };

        let button = ui.add(
            egui::Button::new(
                RichText::new(&self.label)
                    .color(color)
                    .monospace()
                    .size(11.0)
            )
            .frame(false)
            .small()
        );

        if button.hovered() {
            let mut tooltip_text = self.tooltip.clone();
            if let Some(shortcut) = &self.shortcut {
                tooltip_text.push_str(&format!(" ({})", shortcut));
            }

            egui::show_tooltip(ui.ctx(), egui::Id::new(&self.label), |ui| {
                ui.label(tooltip_text);
            });
        }

        button
    }
}

/// Model space / Paper space indicator
pub struct WorkspaceIndicator {
    is_model_space: bool,
}

impl WorkspaceIndicator {
    pub fn new() -> Self {
        Self {
            is_model_space: true,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let text = if self.is_model_space {
            "MODEL"
        } else {
            "PAPER"
        };

        let color = if self.is_model_space {
            Color32::from_rgb(100, 200, 255)
        } else {
            Color32::from_rgb(255, 200, 100)
        };

        let response = ui.add(
            egui::Button::new(
                RichText::new(text)
                    .color(color)
                    .monospace()
                    .strong()
            )
            .frame(false)
            .small()
        );

        if response.clicked() {
            self.is_model_space = !self.is_model_space;
        }

        if response.hovered() {
            egui::show_tooltip(ui.ctx(), egui::Id::new("workspace_tooltip"), |ui| {
                ui.label("Click to switch between Model and Paper space");
            });
        }
    }
}

impl Default for WorkspaceIndicator {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick view selector for common views
pub struct QuickViewSelector {
    current_view: ViewType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewType {
    Top,
    Front,
    Right,
    Isometric,
    Custom,
}

impl QuickViewSelector {
    pub fn new() -> Self {
        Self {
            current_view: ViewType::Top,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("View:")
            .color(Color32::from_rgb(150, 150, 150))
            .size(11.0));

        let view_name = match self.current_view {
            ViewType::Top => "Top",
            ViewType::Front => "Front",
            ViewType::Right => "Right",
            ViewType::Isometric => "ISO",
            ViewType::Custom => "Custom",
        };

        egui::ComboBox::from_id_source("quick_view_selector")
            .selected_text(RichText::new(view_name).monospace())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.current_view, ViewType::Top, "Top");
                ui.selectable_value(&mut self.current_view, ViewType::Front, "Front");
                ui.selectable_value(&mut self.current_view, ViewType::Right, "Right");
                ui.selectable_value(&mut self.current_view, ViewType::Isometric, "Isometric");
            });
    }
}

impl Default for QuickViewSelector {
    fn default() -> Self {
        Self::new()
    }
}
