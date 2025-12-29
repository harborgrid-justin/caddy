/// Side panels for CADDY - Properties, Layers, and Command panels
///
/// Provides side panels for managing layers, viewing properties, and command history.
use egui::{Ui, ScrollArea, CollapsingHeader, Color32, RichText};
use super::UiState;

/// Base panel trait
pub trait Panel {
    fn show(&mut self, ui: &mut Ui, state: &mut UiState);
    fn title(&self) -> &str;
}

/// Properties panel - shows selected entity properties
pub struct PropertiesPanel {
    /// Currently selected entities
    selected_count: usize,
    /// Property values (key-value pairs)
    properties: Vec<Property>,
}

#[derive(Debug, Clone)]
struct Property {
    name: String,
    value: String,
    editable: bool,
    property_type: PropertyType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PropertyType {
    Text,
    Number,
    Color,
    Layer,
    LineType,
    Boolean,
}

impl PropertiesPanel {
    pub fn new() -> Self {
        Self {
            selected_count: 0,
            properties: Vec::new(),
        }
    }

    /// Update properties for selected entities
    pub fn update_selection(&mut self, count: usize) {
        self.selected_count = count;

        // Clear old properties
        self.properties.clear();

        if count == 1 {
            // Single entity selected - show all properties
            self.properties = vec![
                Property {
                    name: "Type".to_string(),
                    value: "Line".to_string(),
                    editable: false,
                    property_type: PropertyType::Text,
                },
                Property {
                    name: "Layer".to_string(),
                    value: "0".to_string(),
                    editable: true,
                    property_type: PropertyType::Layer,
                },
                Property {
                    name: "Color".to_string(),
                    value: "ByLayer".to_string(),
                    editable: true,
                    property_type: PropertyType::Color,
                },
                Property {
                    name: "LineType".to_string(),
                    value: "Continuous".to_string(),
                    editable: true,
                    property_type: PropertyType::LineType,
                },
                Property {
                    name: "LineWeight".to_string(),
                    value: "0.25mm".to_string(),
                    editable: true,
                    property_type: PropertyType::Text,
                },
                Property {
                    name: "Start X".to_string(),
                    value: "0.0".to_string(),
                    editable: true,
                    property_type: PropertyType::Number,
                },
                Property {
                    name: "Start Y".to_string(),
                    value: "0.0".to_string(),
                    editable: true,
                    property_type: PropertyType::Number,
                },
                Property {
                    name: "End X".to_string(),
                    value: "100.0".to_string(),
                    editable: true,
                    property_type: PropertyType::Number,
                },
                Property {
                    name: "End Y".to_string(),
                    value: "100.0".to_string(),
                    editable: true,
                    property_type: PropertyType::Number,
                },
                Property {
                    name: "Length".to_string(),
                    value: "141.42".to_string(),
                    editable: false,
                    property_type: PropertyType::Number,
                },
            ];
        } else if count > 1 {
            // Multiple entities selected - show common properties only
            self.properties = vec![
                Property {
                    name: "Selection".to_string(),
                    value: format!("{} objects", count),
                    editable: false,
                    property_type: PropertyType::Text,
                },
                Property {
                    name: "Layer".to_string(),
                    value: "*VARIES*".to_string(),
                    editable: true,
                    property_type: PropertyType::Layer,
                },
                Property {
                    name: "Color".to_string(),
                    value: "*VARIES*".to_string(),
                    editable: true,
                    property_type: PropertyType::Color,
                },
            ];
        }
    }
}

impl Panel for PropertiesPanel {
    fn show(&mut self, ui: &mut Ui, _state: &mut UiState) {
        ui.heading("Properties");
        ui.separator();

        if self.selected_count == 0 {
            ui.label("No selection");
            return;
        }

        ScrollArea::vertical().show(ui, |ui| {
            // General section
            CollapsingHeader::new("General")
                .default_open(true)
                .show(ui, |ui| {
                    egui::Grid::new("general_properties")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            for prop in &self.properties {
                                if matches!(prop.property_type, PropertyType::Text | PropertyType::Layer | PropertyType::Color | PropertyType::LineType) {
                                    ui.label(&prop.name);
                                    if prop.editable {
                                        ui.text_edit_singleline(&mut prop.value.clone());
                                    } else {
                                        ui.label(&prop.value);
                                    }
                                    ui.end_row();
                                }
                            }
                        });
                });

            // Geometry section
            CollapsingHeader::new("Geometry")
                .default_open(true)
                .show(ui, |ui| {
                    egui::Grid::new("geometry_properties")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            for prop in &self.properties {
                                if matches!(prop.property_type, PropertyType::Number) {
                                    ui.label(&prop.name);
                                    if prop.editable {
                                        ui.text_edit_singleline(&mut prop.value.clone());
                                    } else {
                                        ui.label(&prop.value);
                                    }
                                    ui.end_row();
                                }
                            }
                        });
                });
        });
    }

    fn title(&self) -> &str {
        "Properties"
    }
}

/// Layers panel - manages drawing layers
pub struct LayersPanel {
    layers: Vec<Layer>,
    filter: String,
}

#[derive(Debug, Clone)]
struct Layer {
    name: String,
    visible: bool,
    locked: bool,
    frozen: bool,
    color: Color32,
    line_type: String,
    line_weight: f32,
    current: bool,
}

impl LayersPanel {
    pub fn new() -> Self {
        let mut layers = Vec::new();

        // Add default layers
        layers.push(Layer {
            name: "0".to_string(),
            visible: true,
            locked: false,
            frozen: false,
            color: Color32::WHITE,
            line_type: "Continuous".to_string(),
            line_weight: 0.25,
            current: true,
        });

        layers.push(Layer {
            name: "Construction".to_string(),
            visible: true,
            locked: false,
            frozen: false,
            color: Color32::from_rgb(128, 128, 128),
            line_type: "Dashed".to_string(),
            line_weight: 0.13,
            current: false,
        });

        layers.push(Layer {
            name: "Dimensions".to_string(),
            visible: true,
            locked: false,
            frozen: false,
            color: Color32::from_rgb(0, 255, 255),
            line_type: "Continuous".to_string(),
            line_weight: 0.18,
            current: false,
        });

        layers.push(Layer {
            name: "Text".to_string(),
            visible: true,
            locked: false,
            frozen: false,
            color: Color32::from_rgb(255, 255, 0),
            line_type: "Continuous".to_string(),
            line_weight: 0.25,
            current: false,
        });

        Self {
            layers,
            filter: String::new(),
        }
    }

    /// Add a new layer
    pub fn add_layer(&mut self, name: String) {
        self.layers.push(Layer {
            name,
            visible: true,
            locked: false,
            frozen: false,
            color: Color32::WHITE,
            line_type: "Continuous".to_string(),
            line_weight: 0.25,
            current: false,
        });
    }

    /// Set current layer
    pub fn set_current(&mut self, name: &str) {
        for layer in &mut self.layers {
            layer.current = layer.name == name;
        }
    }
}

impl Panel for LayersPanel {
    fn show(&mut self, ui: &mut Ui, state: &mut UiState) {
        ui.heading("Layers");
        ui.separator();

        // Toolbar
        ui.horizontal(|ui| {
            if ui.button("➕ New").clicked() {
                let new_name = format!("Layer {}", self.layers.len());
                self.add_layer(new_name);
            }
            if ui.button("🗑 Delete").clicked() {
                // Delete selected non-current layer
            }
            if ui.button("⚙ Properties").clicked() {
                // Open layer properties dialog
            }
        });

        ui.add_space(5.0);

        // Filter
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.text_edit_singleline(&mut self.filter);
        });

        ui.separator();

        // Layer list with headers
        ScrollArea::vertical().show(ui, |ui| {
            // Header
            ui.horizontal(|ui| {
                ui.label(RichText::new("Name").strong());
                ui.add_space(60.0);
                ui.label(RichText::new("On").strong());
                ui.label(RichText::new("Lock").strong());
                ui.label(RichText::new("Freeze").strong());
                ui.label(RichText::new("Color").strong());
            });

            ui.separator();

            // Layer rows - clone filter to avoid borrow conflict
            let filter_lower = self.filter.to_lowercase();
            let has_filter = !self.filter.is_empty();
            let mut new_current_layer: Option<String> = None;

            for layer in &mut self.layers {
                // Apply filter
                if has_filter && !layer.name.to_lowercase().contains(&filter_lower) {
                    continue;
                }

                ui.horizontal(|ui| {
                    // Current layer indicator
                    if layer.current {
                        ui.label(RichText::new("▶").color(Color32::from_rgb(0, 200, 0)));
                    } else {
                        ui.label("  ");
                    }

                    // Layer name (clickable to make current)
                    let name_response = ui.selectable_label(layer.current, &layer.name);
                    if name_response.clicked() {
                        new_current_layer = Some(layer.name.clone());
                        state.current_layer = layer.name.clone();
                    }

                    ui.add_space(10.0);

                    // Visible toggle
                    let visible_text = if layer.visible { "👁" } else { "🚫" };
                    if ui.small_button(visible_text).clicked() {
                        layer.visible = !layer.visible;
                    }

                    // Lock toggle
                    let lock_text = if layer.locked { "🔒" } else { "🔓" };
                    if ui.small_button(lock_text).clicked() {
                        layer.locked = !layer.locked;
                    }

                    // Freeze toggle
                    let freeze_text = if layer.frozen { "❄" } else { "☀" };
                    if ui.small_button(freeze_text).clicked() {
                        layer.frozen = !layer.frozen;
                    }

                    // Color swatch
                    let color_size = egui::vec2(20.0, 15.0);
                    let (rect, response) = ui.allocate_exact_size(color_size, egui::Sense::click());
                    ui.painter().rect_filled(rect, 2.0, layer.color);
                    ui.painter().rect_stroke(rect, 2.0, egui::Stroke::new(1.0, Color32::GRAY));

                    if response.clicked() {
                        // Open color picker
                    }
                });

                ui.add_space(2.0);
            }

            // Apply deferred current layer change
            if let Some(name) = new_current_layer {
                self.set_current(&name);
            }
        });
    }

    fn title(&self) -> &str {
        "Layers"
    }
}

/// Command panel - shows command history
pub struct CommandPanel {
    commands: Vec<CommandEntry>,
    max_commands: usize,
}

#[derive(Debug, Clone)]
struct CommandEntry {
    command: String,
    timestamp: String,
    success: bool,
}

impl CommandPanel {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            max_commands: 100,
        }
    }

    /// Add command to history
    pub fn add_command(&mut self, command: &str) {
        let entry = CommandEntry {
            command: command.to_string(),
            timestamp: Self::current_time(),
            success: true,
        };

        self.commands.push(entry);

        // Keep only last N commands
        if self.commands.len() > self.max_commands {
            self.commands.remove(0);
        }
    }

    /// Add failed command to history
    pub fn add_failed_command(&mut self, command: &str) {
        let entry = CommandEntry {
            command: command.to_string(),
            timestamp: Self::current_time(),
            success: false,
        };

        self.commands.push(entry);

        if self.commands.len() > self.max_commands {
            self.commands.remove(0);
        }
    }

    /// Clear command history
    pub fn clear(&mut self) {
        self.commands.clear();
    }

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

impl Panel for CommandPanel {
    fn show(&mut self, ui: &mut Ui, _state: &mut UiState) {
        ui.heading("Command History");
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("Clear").clicked() {
                self.clear();
            }
            ui.label(format!("{} commands", self.commands.len()));
        });

        ui.separator();

        ScrollArea::vertical()
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for entry in &self.commands {
                    ui.horizontal(|ui| {
                        // Timestamp
                        ui.label(RichText::new(&entry.timestamp)
                            .color(Color32::GRAY)
                            .monospace()
                            .size(10.0));

                        // Status indicator
                        if entry.success {
                            ui.label(RichText::new("✓").color(Color32::GREEN));
                        } else {
                            ui.label(RichText::new("✗").color(Color32::RED));
                        }

                        // Command text
                        let color = if entry.success {
                            Color32::WHITE
                        } else {
                            Color32::from_rgb(255, 100, 100)
                        };
                        ui.label(RichText::new(&entry.command)
                            .color(color)
                            .monospace());
                    });
                }
            });
    }

    fn title(&self) -> &str {
        "Command History"
    }
}

/// Quick access panel for frequently used commands
pub struct QuickAccessPanel {
    commands: Vec<QuickCommand>,
}

#[derive(Debug, Clone)]
struct QuickCommand {
    name: String,
    command: String,
    icon: String,
}

impl QuickAccessPanel {
    pub fn new() -> Self {
        let commands = vec![
            QuickCommand {
                name: "Line".to_string(),
                command: "LINE".to_string(),
                icon: "📏".to_string(),
            },
            QuickCommand {
                name: "Circle".to_string(),
                command: "CIRCLE".to_string(),
                icon: "⭕".to_string(),
            },
            QuickCommand {
                name: "Rectangle".to_string(),
                command: "RECTANGLE".to_string(),
                icon: "▭".to_string(),
            },
            QuickCommand {
                name: "Move".to_string(),
                command: "MOVE".to_string(),
                icon: "➡".to_string(),
            },
            QuickCommand {
                name: "Copy".to_string(),
                command: "COPY".to_string(),
                icon: "📋".to_string(),
            },
            QuickCommand {
                name: "Rotate".to_string(),
                command: "ROTATE".to_string(),
                icon: "🔄".to_string(),
            },
        ];

        Self { commands }
    }
}

impl Panel for QuickAccessPanel {
    fn show(&mut self, ui: &mut Ui, _state: &mut UiState) {
        ui.heading("Quick Access");
        ui.separator();

        egui::Grid::new("quick_commands")
            .num_columns(2)
            .spacing([4.0, 4.0])
            .show(ui, |ui| {
                for (i, cmd) in self.commands.iter().enumerate() {
                    if ui.button(format!("{} {}", cmd.icon, cmd.name)).clicked() {
                        log::info!("Quick command: {}", cmd.command);
                    }

                    if (i + 1) % 2 == 0 {
                        ui.end_row();
                    }
                }
            });
    }

    fn title(&self) -> &str {
        "Quick Access"
    }
}
