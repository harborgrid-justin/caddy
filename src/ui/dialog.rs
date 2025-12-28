/// Dialog windows for CADDY - File dialogs, settings, and modal windows
///
/// Provides modal dialogs for file operations, settings, and configuration.
use egui::{Window, Context, Color32, RichText};
use super::UiState;
use std::path::PathBuf;

/// Base dialog trait
pub trait Dialog {
    fn show(&mut self, ctx: &Context, state: &mut UiState) -> DialogResult;
    fn is_open(&self) -> bool;
    fn close(&mut self);
}

/// Dialog result
#[derive(Debug, Clone, PartialEq)]
pub enum DialogResult {
    /// Dialog still open, no action
    None,
    /// Dialog confirmed with data
    Ok(DialogData),
    /// Dialog cancelled
    Cancelled,
}

/// Dialog data returned on confirmation
#[derive(Debug, Clone, PartialEq)]
pub enum DialogData {
    None,
    FilePath(PathBuf),
    Settings(SettingsData),
    Layer(LayerData),
    DimensionStyle(DimensionStyleData),
}

/// Settings data
#[derive(Debug, Clone, PartialEq)]
pub struct SettingsData {
    pub grid_size: f64,
    pub snap_distance: f64,
    pub background_color: Color32,
    pub crosshair_size: f32,
    pub auto_save: bool,
    pub auto_save_interval: u32,
}

/// Layer data
#[derive(Debug, Clone, PartialEq)]
pub struct LayerData {
    pub name: String,
    pub color: Color32,
    pub line_type: String,
    pub line_weight: f32,
}

/// Dimension style data
#[derive(Debug, Clone, PartialEq)]
pub struct DimensionStyleData {
    pub name: String,
    pub text_height: f64,
    pub arrow_size: f64,
    pub extension_line_offset: f64,
}

/// File dialog for opening and saving files
pub struct FileDialog {
    open: bool,
    mode: FileDialogMode,
    current_path: PathBuf,
    selected_file: Option<PathBuf>,
    file_filter: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileDialogMode {
    Open,
    Save,
    SaveAs,
    Export,
}

impl FileDialog {
    pub fn new_open() -> Self {
        Self {
            open: true,
            mode: FileDialogMode::Open,
            current_path: std::env::current_dir().unwrap_or_default(),
            selected_file: None,
            file_filter: String::new(),
        }
    }

    pub fn new_save() -> Self {
        Self {
            open: true,
            mode: FileDialogMode::Save,
            current_path: std::env::current_dir().unwrap_or_default(),
            selected_file: None,
            file_filter: String::new(),
        }
    }

    pub fn new_export() -> Self {
        Self {
            open: true,
            mode: FileDialogMode::Export,
            current_path: std::env::current_dir().unwrap_or_default(),
            selected_file: None,
            file_filter: String::new(),
        }
    }

    fn title(&self) -> &str {
        match self.mode {
            FileDialogMode::Open => "Open File",
            FileDialogMode::Save => "Save File",
            FileDialogMode::SaveAs => "Save File As",
            FileDialogMode::Export => "Export File",
        }
    }
}

impl Dialog for FileDialog {
    fn show(&mut self, ctx: &Context, _state: &mut UiState) -> DialogResult {
        let mut result = DialogResult::None;
        let mut should_close = false;

        Window::new(self.title())
            .open(&mut self.open)
            .collapsible(false)
            .resizable(true)
            .default_width(600.0)
            .default_height(400.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // Current path
                    ui.horizontal(|ui| {
                        ui.label("Location:");
                        ui.label(self.current_path.to_string_lossy().to_string());
                    });

                    ui.separator();

                    // File browser (simplified - in production use rfd crate)
                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            // Show parent directory
                            if ui.button("📁 ..").clicked() {
                                if let Some(parent) = self.current_path.parent() {
                                    self.current_path = parent.to_path_buf();
                                }
                            }

                            // Show directories and files
                            if let Ok(entries) = std::fs::read_dir(&self.current_path) {
                                for entry in entries.flatten() {
                                    let path = entry.path();
                                    let name = path.file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy()
                                        .to_string();

                                    if path.is_dir() {
                                        if ui.button(format!("📁 {}", name)).clicked() {
                                            self.current_path = path;
                                        }
                                    } else {
                                        let is_cad_file = name.ends_with(".dxf") ||
                                            name.ends_with(".dwg") ||
                                            name.ends_with(".cad");

                                        if is_cad_file {
                                            let selected = self.selected_file.as_ref()
                                                .map(|p| p == &path)
                                                .unwrap_or(false);

                                            if ui.selectable_label(selected, format!("📄 {}", name)).clicked() {
                                                self.selected_file = Some(path);
                                            }
                                        }
                                    }
                                }
                            }
                        });

                    ui.separator();

                    // Selected file
                    ui.horizontal(|ui| {
                        ui.label("File:");
                        if let Some(file) = &self.selected_file {
                            ui.label(file.file_name().unwrap_or_default().to_string_lossy().to_string());
                        } else {
                            ui.label("No file selected");
                        }
                    });

                    // File type filter
                    ui.horizontal(|ui| {
                        ui.label("Type:");
                        egui::ComboBox::from_id_source("file_type")
                            .selected_text("CADDY Drawing (*.cad)")
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.file_filter, "*.cad".to_string(), "CADDY Drawing (*.cad)");
                                ui.selectable_value(&mut self.file_filter, "*.dxf".to_string(), "AutoCAD DXF (*.dxf)");
                                ui.selectable_value(&mut self.file_filter, "*.dwg".to_string(), "AutoCAD DWG (*.dwg)");
                                ui.selectable_value(&mut self.file_filter, "*.*".to_string(), "All Files (*.*)");
                            });
                    });

                    ui.separator();

                    // Buttons
                    ui.horizontal(|ui| {
                        let button_text = match self.mode {
                            FileDialogMode::Open => "Open",
                            FileDialogMode::Save | FileDialogMode::SaveAs => "Save",
                            FileDialogMode::Export => "Export",
                        };

                        if ui.button(button_text).clicked() {
                            if let Some(path) = &self.selected_file {
                                result = DialogResult::Ok(DialogData::FilePath(path.clone()));
                                should_close = true;
                            }
                        }

                        if ui.button("Cancel").clicked() {
                            result = DialogResult::Cancelled;
                            should_close = true;
                        }
                    });
                });
            });

        if should_close {
            self.open = false;
        }

        if !self.open && result == DialogResult::None {
            result = DialogResult::Cancelled;
        }

        result
    }

    fn is_open(&self) -> bool {
        self.open
    }

    fn close(&mut self) {
        self.open = false;
    }
}

/// Settings dialog
pub struct SettingsDialog {
    open: bool,
    settings: SettingsData,
}

impl SettingsDialog {
    pub fn new() -> Self {
        Self {
            open: true,
            settings: SettingsData {
                grid_size: 10.0,
                snap_distance: 5.0,
                background_color: Color32::from_rgb(30, 30, 30),
                crosshair_size: 20.0,
                auto_save: true,
                auto_save_interval: 5,
            },
        }
    }
}

impl Dialog for SettingsDialog {
    fn show(&mut self, ctx: &Context, _state: &mut UiState) -> DialogResult {
        let mut result = DialogResult::None;
        let mut should_close = false;

        Window::new("Settings")
            .open(&mut self.open)
            .collapsible(false)
            .resizable(false)
            .default_width(500.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Drawing settings
                    ui.heading("Drawing");
                    ui.separator();

                    egui::Grid::new("drawing_settings")
                        .num_columns(2)
                        .spacing([40.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Grid Size:");
                            ui.add(egui::DragValue::new(&mut self.settings.grid_size)
                                .speed(0.1)
                                .clamp_range(1.0..=100.0));
                            ui.end_row();

                            ui.label("Snap Distance:");
                            ui.add(egui::DragValue::new(&mut self.settings.snap_distance)
                                .speed(0.1)
                                .clamp_range(1.0..=50.0));
                            ui.end_row();

                            ui.label("Crosshair Size:");
                            ui.add(egui::Slider::new(&mut self.settings.crosshair_size, 10.0..=50.0));
                            ui.end_row();
                        });

                    ui.add_space(10.0);

                    // Display settings
                    ui.heading("Display");
                    ui.separator();

                    egui::Grid::new("display_settings")
                        .num_columns(2)
                        .spacing([40.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Background Color:");
                            ui.color_edit_button_srgba(&mut self.settings.background_color);
                            ui.end_row();
                        });

                    ui.add_space(10.0);

                    // File settings
                    ui.heading("File");
                    ui.separator();

                    egui::Grid::new("file_settings")
                        .num_columns(2)
                        .spacing([40.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Auto Save:");
                            ui.checkbox(&mut self.settings.auto_save, "");
                            ui.end_row();

                            if self.settings.auto_save {
                                ui.label("Auto Save Interval (min):");
                                ui.add(egui::DragValue::new(&mut self.settings.auto_save_interval)
                                    .clamp_range(1..=60));
                                ui.end_row();
                            }
                        });

                    ui.add_space(20.0);

                    // Buttons
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("   OK   ").clicked() {
                            result = DialogResult::Ok(DialogData::Settings(self.settings.clone()));
                            should_close = true;
                        }

                        if ui.button(" Cancel ").clicked() {
                            result = DialogResult::Cancelled;
                            should_close = true;
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Reset to Defaults").clicked() {
                                self.settings = SettingsData {
                                    grid_size: 10.0,
                                    snap_distance: 5.0,
                                    background_color: Color32::from_rgb(30, 30, 30),
                                    crosshair_size: 20.0,
                                    auto_save: true,
                                    auto_save_interval: 5,
                                };
                            }
                        });
                    });
                });
            });

        if should_close {
            self.open = false;
        }

        if !self.open && result == DialogResult::None {
            result = DialogResult::Cancelled;
        }

        result
    }

    fn is_open(&self) -> bool {
        self.open
    }

    fn close(&mut self) {
        self.open = false;
    }
}

/// Layer properties dialog
pub struct LayerDialog {
    open: bool,
    layer: LayerData,
    is_new: bool,
}

impl LayerDialog {
    pub fn new() -> Self {
        Self {
            open: true,
            layer: LayerData {
                name: "New Layer".to_string(),
                color: Color32::WHITE,
                line_type: "Continuous".to_string(),
                line_weight: 0.25,
            },
            is_new: true,
        }
    }

    pub fn edit(layer: LayerData) -> Self {
        Self {
            open: true,
            layer,
            is_new: false,
        }
    }
}

impl Dialog for LayerDialog {
    fn show(&mut self, ctx: &Context, _state: &mut UiState) -> DialogResult {
        let mut result = DialogResult::None;
        let mut should_close = false;

        let title = if self.is_new { "New Layer" } else { "Layer Properties" };

        Window::new(title)
            .open(&mut self.open)
            .collapsible(false)
            .resizable(false)
            .default_width(400.0)
            .show(ctx, |ui| {
                egui::Grid::new("layer_properties")
                    .num_columns(2)
                    .spacing([40.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.layer.name);
                        ui.end_row();

                        ui.label("Color:");
                        ui.color_edit_button_srgba(&mut self.layer.color);
                        ui.end_row();

                        ui.label("Line Type:");
                        egui::ComboBox::from_id_source("line_type")
                            .selected_text(&self.layer.line_type)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.layer.line_type, "Continuous".to_string(), "Continuous");
                                ui.selectable_value(&mut self.layer.line_type, "Dashed".to_string(), "Dashed");
                                ui.selectable_value(&mut self.layer.line_type, "Dotted".to_string(), "Dotted");
                                ui.selectable_value(&mut self.layer.line_type, "DashDot".to_string(), "Dash-Dot");
                                ui.selectable_value(&mut self.layer.line_type, "Hidden".to_string(), "Hidden");
                            });
                        ui.end_row();

                        ui.label("Line Weight:");
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut self.layer.line_weight)
                                .speed(0.01)
                                .clamp_range(0.05..=2.0));
                            ui.label("mm");
                        });
                        ui.end_row();
                    });

                ui.add_space(20.0);

                // Buttons
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("   OK   ").clicked() {
                        result = DialogResult::Ok(DialogData::Layer(self.layer.clone()));
                        should_close = true;
                    }

                    if ui.button(" Cancel ").clicked() {
                        result = DialogResult::Cancelled;
                        should_close = true;
                    }
                });
            });

        if should_close {
            self.open = false;
        }

        if !self.open && result == DialogResult::None {
            result = DialogResult::Cancelled;
        }

        result
    }

    fn is_open(&self) -> bool {
        self.open
    }

    fn close(&mut self) {
        self.open = false;
    }
}

/// Dimension style dialog
pub struct DimensionStyleDialog {
    open: bool,
    style: DimensionStyleData,
}

impl DimensionStyleDialog {
    pub fn new() -> Self {
        Self {
            open: true,
            style: DimensionStyleData {
                name: "Standard".to_string(),
                text_height: 2.5,
                arrow_size: 2.5,
                extension_line_offset: 1.25,
            },
        }
    }
}

impl Dialog for DimensionStyleDialog {
    fn show(&mut self, ctx: &Context, _state: &mut UiState) -> DialogResult {
        let mut result = DialogResult::None;
        let mut should_close = false;

        Window::new("Dimension Style")
            .open(&mut self.open)
            .collapsible(false)
            .resizable(false)
            .default_width(500.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("General");
                    ui.separator();

                    egui::Grid::new("dim_general")
                        .num_columns(2)
                        .spacing([40.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Style Name:");
                            ui.text_edit_singleline(&mut self.style.name);
                            ui.end_row();
                        });

                    ui.add_space(10.0);

                    ui.heading("Text");
                    ui.separator();

                    egui::Grid::new("dim_text")
                        .num_columns(2)
                        .spacing([40.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Text Height:");
                            ui.add(egui::DragValue::new(&mut self.style.text_height)
                                .speed(0.1)
                                .clamp_range(0.5..=10.0));
                            ui.end_row();
                        });

                    ui.add_space(10.0);

                    ui.heading("Arrows");
                    ui.separator();

                    egui::Grid::new("dim_arrows")
                        .num_columns(2)
                        .spacing([40.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Arrow Size:");
                            ui.add(egui::DragValue::new(&mut self.style.arrow_size)
                                .speed(0.1)
                                .clamp_range(0.5..=10.0));
                            ui.end_row();
                        });

                    ui.add_space(10.0);

                    ui.heading("Lines");
                    ui.separator();

                    egui::Grid::new("dim_lines")
                        .num_columns(2)
                        .spacing([40.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Extension Line Offset:");
                            ui.add(egui::DragValue::new(&mut self.style.extension_line_offset)
                                .speed(0.1)
                                .clamp_range(0.0..=5.0));
                            ui.end_row();
                        });

                    ui.add_space(20.0);

                    // Buttons
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("   OK   ").clicked() {
                            result = DialogResult::Ok(DialogData::DimensionStyle(self.style.clone()));
                            should_close = true;
                        }

                        if ui.button(" Cancel ").clicked() {
                            result = DialogResult::Cancelled;
                            should_close = true;
                        }
                    });
                });
            });

        if should_close {
            self.open = false;
        }

        if !self.open && result == DialogResult::None {
            result = DialogResult::Cancelled;
        }

        result
    }

    fn is_open(&self) -> bool {
        self.open
    }

    fn close(&mut self) {
        self.open = false;
    }
}

/// About dialog
pub struct AboutDialog {
    open: bool,
}

impl AboutDialog {
    pub fn new() -> Self {
        Self { open: true }
    }
}

impl Dialog for AboutDialog {
    fn show(&mut self, ctx: &Context, _state: &mut UiState) -> DialogResult {
        let mut result = DialogResult::None;
        let mut should_close = false;

        Window::new("About CADDY")
            .open(&mut self.open)
            .collapsible(false)
            .resizable(false)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(RichText::new("CADDY").size(24.0));
                    ui.label("Computer-Aided Design Done Yourself");
                    ui.add_space(10.0);
                    ui.label("Version 0.1.0");
                    ui.add_space(10.0);
                    ui.label("Enterprise CAD System");
                    ui.label("Built with Rust");
                    ui.add_space(20.0);
                    ui.label("© 2024 CADDY Team");
                    ui.add_space(10.0);
                    ui.hyperlink_to("GitHub", "https://github.com/caddy-cad/caddy");
                    ui.add_space(20.0);

                    if ui.button("   OK   ").clicked() {
                        result = DialogResult::Ok(DialogData::None);
                        should_close = true;
                    }
                });
            });

        if should_close {
            self.open = false;
        }

        if !self.open && result == DialogResult::None {
            result = DialogResult::Cancelled;
        }

        result
    }

    fn is_open(&self) -> bool {
        self.open
    }

    fn close(&mut self) {
        self.open = false;
    }
}
