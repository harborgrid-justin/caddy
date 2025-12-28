/// Command line interface widget for CADDY
///
/// AutoCAD-style command line with autocomplete, history, and coordinate parsing.

use egui::{Ui, TextEdit, Color32, RichText, Key};
use super::UiState;

/// Command line widget
pub struct CommandLine {
    /// Current input text
    input: String,
    /// Command history
    history: Vec<String>,
    /// History index (for up/down arrow navigation)
    history_index: Option<usize>,
    /// Maximum history size
    max_history: usize,
    /// Current prompt
    prompt: String,
    /// Error message (if any)
    error: Option<String>,
    /// Autocomplete suggestions
    suggestions: Vec<String>,
    /// Show suggestions
    show_suggestions: bool,
    /// Recent command output
    output_lines: Vec<OutputLine>,
    /// Maximum output lines
    max_output: usize,
    /// Command registry for autocomplete
    command_registry: Vec<CommandInfo>,
}

#[derive(Debug, Clone)]
struct OutputLine {
    text: String,
    line_type: OutputType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputType {
    Command,
    Response,
    Error,
    Info,
}

#[derive(Debug, Clone)]
struct CommandInfo {
    name: String,
    aliases: Vec<String>,
    description: String,
}

impl CommandLine {
    pub fn new() -> Self {
        let mut cmd_line = Self {
            input: String::new(),
            history: Vec::new(),
            history_index: None,
            max_history: 100,
            prompt: "Command:".to_string(),
            error: None,
            suggestions: Vec::new(),
            show_suggestions: false,
            output_lines: Vec::new(),
            max_output: 100,
            command_registry: Vec::new(),
        };

        cmd_line.initialize_commands();
        cmd_line
    }

    /// Initialize command registry
    fn initialize_commands(&mut self) {
        self.command_registry = vec![
            CommandInfo {
                name: "LINE".to_string(),
                aliases: vec!["L".to_string()],
                description: "Draw a line".to_string(),
            },
            CommandInfo {
                name: "CIRCLE".to_string(),
                aliases: vec!["C".to_string()],
                description: "Draw a circle".to_string(),
            },
            CommandInfo {
                name: "ARC".to_string(),
                aliases: vec!["A".to_string()],
                description: "Draw an arc".to_string(),
            },
            CommandInfo {
                name: "RECTANGLE".to_string(),
                aliases: vec!["REC".to_string(), "RECT".to_string()],
                description: "Draw a rectangle".to_string(),
            },
            CommandInfo {
                name: "POLYLINE".to_string(),
                aliases: vec!["PL".to_string(), "PLINE".to_string()],
                description: "Draw a polyline".to_string(),
            },
            CommandInfo {
                name: "POLYGON".to_string(),
                aliases: vec!["POL".to_string()],
                description: "Draw a polygon".to_string(),
            },
            CommandInfo {
                name: "ELLIPSE".to_string(),
                aliases: vec!["EL".to_string()],
                description: "Draw an ellipse".to_string(),
            },
            CommandInfo {
                name: "SPLINE".to_string(),
                aliases: vec!["SPL".to_string()],
                description: "Draw a spline".to_string(),
            },
            CommandInfo {
                name: "MOVE".to_string(),
                aliases: vec!["M".to_string()],
                description: "Move entities".to_string(),
            },
            CommandInfo {
                name: "COPY".to_string(),
                aliases: vec!["CO".to_string(), "CP".to_string()],
                description: "Copy entities".to_string(),
            },
            CommandInfo {
                name: "ROTATE".to_string(),
                aliases: vec!["RO".to_string()],
                description: "Rotate entities".to_string(),
            },
            CommandInfo {
                name: "SCALE".to_string(),
                aliases: vec!["SC".to_string()],
                description: "Scale entities".to_string(),
            },
            CommandInfo {
                name: "MIRROR".to_string(),
                aliases: vec!["MI".to_string()],
                description: "Mirror entities".to_string(),
            },
            CommandInfo {
                name: "ARRAY".to_string(),
                aliases: vec!["AR".to_string()],
                description: "Create an array".to_string(),
            },
            CommandInfo {
                name: "TRIM".to_string(),
                aliases: vec!["TR".to_string()],
                description: "Trim entities".to_string(),
            },
            CommandInfo {
                name: "EXTEND".to_string(),
                aliases: vec!["EX".to_string()],
                description: "Extend entities".to_string(),
            },
            CommandInfo {
                name: "OFFSET".to_string(),
                aliases: vec!["O".to_string()],
                description: "Offset entities".to_string(),
            },
            CommandInfo {
                name: "FILLET".to_string(),
                aliases: vec!["F".to_string()],
                description: "Fillet entities".to_string(),
            },
            CommandInfo {
                name: "CHAMFER".to_string(),
                aliases: vec!["CHA".to_string()],
                description: "Chamfer entities".to_string(),
            },
            CommandInfo {
                name: "ZOOM".to_string(),
                aliases: vec!["Z".to_string()],
                description: "Zoom view".to_string(),
            },
            CommandInfo {
                name: "PAN".to_string(),
                aliases: vec!["P".to_string()],
                description: "Pan view".to_string(),
            },
            CommandInfo {
                name: "GRID".to_string(),
                aliases: vec![],
                description: "Toggle grid".to_string(),
            },
            CommandInfo {
                name: "SNAP".to_string(),
                aliases: vec![],
                description: "Toggle snap".to_string(),
            },
            CommandInfo {
                name: "ORTHO".to_string(),
                aliases: vec![],
                description: "Toggle ortho mode".to_string(),
            },
            CommandInfo {
                name: "UNDO".to_string(),
                aliases: vec!["U".to_string()],
                description: "Undo last operation".to_string(),
            },
            CommandInfo {
                name: "REDO".to_string(),
                aliases: vec![],
                description: "Redo last undone operation".to_string(),
            },
        ];
    }

    /// Set prompt text
    pub fn set_prompt(&mut self, prompt: &str) {
        self.prompt = prompt.to_string();
        self.add_output(prompt, OutputType::Info);
    }

    /// Set error message
    pub fn set_error(&mut self, error: &str) {
        self.error = Some(error.to_string());
        self.add_output(error, OutputType::Error);
    }

    /// Clear error
    pub fn clear_error(&mut self) {
        self.error = None;
    }

    /// Add line to output
    fn add_output(&mut self, text: &str, line_type: OutputType) {
        self.output_lines.push(OutputLine {
            text: text.to_string(),
            line_type,
        });

        // Keep only last N lines
        if self.output_lines.len() > self.max_output {
            self.output_lines.remove(0);
        }
    }

    /// Add command to history
    fn add_to_history(&mut self, command: &str) {
        if !command.trim().is_empty() {
            // Don't add duplicate of last command
            if self.history.last().map(|s| s.as_str()) != Some(command) {
                self.history.push(command.to_string());
            }

            // Keep only last N commands
            if self.history.len() > self.max_history {
                self.history.remove(0);
            }
        }

        self.history_index = None;
    }

    /// Navigate history up
    fn history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }

        if let Some(idx) = self.history_index {
            if idx > 0 {
                self.history_index = Some(idx - 1);
                self.input = self.history[idx - 1].clone();
            }
        } else {
            let idx = self.history.len() - 1;
            self.history_index = Some(idx);
            self.input = self.history[idx].clone();
        }
    }

    /// Navigate history down
    fn history_down(&mut self) {
        if let Some(idx) = self.history_index {
            if idx < self.history.len() - 1 {
                self.history_index = Some(idx + 1);
                self.input = self.history[idx + 1].clone();
            } else {
                self.history_index = None;
                self.input.clear();
            }
        }
    }

    /// Update autocomplete suggestions
    fn update_suggestions(&mut self) {
        if self.input.is_empty() {
            self.suggestions.clear();
            self.show_suggestions = false;
            return;
        }

        let input_upper = self.input.to_uppercase();
        self.suggestions.clear();

        for cmd_info in &self.command_registry {
            // Check if command name starts with input
            if cmd_info.name.starts_with(&input_upper) {
                self.suggestions.push(cmd_info.name.clone());
            }

            // Check aliases
            for alias in &cmd_info.aliases {
                if alias.starts_with(&input_upper) && !self.suggestions.contains(alias) {
                    self.suggestions.push(alias.clone());
                }
            }
        }

        self.suggestions.sort();
        self.show_suggestions = !self.suggestions.is_empty();
    }

    /// Parse coordinate input (e.g., "100,200" or "100<45" for polar)
    fn parse_coordinate(&self, input: &str) -> Option<(f64, f64)> {
        let input = input.trim();

        // Cartesian coordinates: "x,y"
        if let Some((x_str, y_str)) = input.split_once(',') {
            if let (Ok(x), Ok(y)) = (x_str.trim().parse::<f64>(), y_str.trim().parse::<f64>()) {
                return Some((x, y));
            }
        }

        // Polar coordinates: "distance<angle"
        if let Some((dist_str, angle_str)) = input.split_once('<') {
            if let (Ok(dist), Ok(angle)) = (dist_str.trim().parse::<f64>(), angle_str.trim().parse::<f64>()) {
                let angle_rad = angle.to_radians();
                let x = dist * angle_rad.cos();
                let y = dist * angle_rad.sin();
                return Some((x, y));
            }
        }

        // Relative coordinates: "@x,y"
        if input.starts_with('@') {
            return self.parse_coordinate(&input[1..]);
        }

        None
    }

    /// Show command line widget
    pub fn show(&mut self, ui: &mut Ui, _state: &mut UiState) -> Option<String> {
        let mut command_to_execute = None;

        ui.vertical(|ui| {
            // Output area (command history)
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .max_height(100.0)
                .show(ui, |ui| {
                    for line in &self.output_lines {
                        let color = match line.line_type {
                            OutputType::Command => Color32::from_rgb(200, 200, 255),
                            OutputType::Response => Color32::from_rgb(200, 200, 200),
                            OutputType::Error => Color32::from_rgb(255, 100, 100),
                            OutputType::Info => Color32::from_rgb(100, 200, 255),
                        };

                        ui.label(RichText::new(&line.text).color(color).monospace());
                    }
                });

            ui.separator();

            // Input area
            ui.horizontal(|ui| {
                // Prompt
                let prompt_color = if self.error.is_some() {
                    Color32::from_rgb(255, 100, 100)
                } else {
                    Color32::from_rgb(100, 200, 255)
                };

                ui.label(RichText::new(&self.prompt)
                    .color(prompt_color)
                    .monospace()
                    .strong());

                // Input field
                let input_response = ui.add(
                    TextEdit::singleline(&mut self.input)
                        .desired_width(ui.available_width() - 10.0)
                        .font(egui::TextStyle::Monospace)
                        .hint_text("Type a command...")
                );

                // Request focus on the input field
                if !input_response.has_focus() {
                    input_response.request_focus();
                }

                // Handle keyboard input
                if input_response.changed() {
                    self.update_suggestions();
                    self.clear_error();
                }

                if input_response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                    // Execute command
                    let command = self.input.trim().to_string();
                    if !command.is_empty() {
                        self.add_output(&format!("> {}", command), OutputType::Command);
                        self.add_to_history(&command);
                        command_to_execute = Some(command);
                        self.input.clear();
                        self.suggestions.clear();
                        self.show_suggestions = false;
                    }
                    input_response.request_focus();
                }

                // Handle up/down arrows for history
                if input_response.has_focus() {
                    ui.input(|i| {
                        if i.key_pressed(Key::ArrowUp) {
                            self.history_up();
                        } else if i.key_pressed(Key::ArrowDown) {
                            self.history_down();
                        } else if i.key_pressed(Key::Tab) && !self.suggestions.is_empty() {
                            // Autocomplete with first suggestion
                            self.input = self.suggestions[0].clone();
                            self.suggestions.clear();
                            self.show_suggestions = false;
                        } else if i.key_pressed(Key::Escape) {
                            self.input.clear();
                            self.suggestions.clear();
                            self.show_suggestions = false;
                        }
                    });
                }
            });

            // Show autocomplete suggestions
            if self.show_suggestions && !self.suggestions.is_empty() {
                ui.horizontal(|ui| {
                    ui.add_space(100.0);
                    ui.label(RichText::new("Suggestions:").color(Color32::GRAY).size(10.0));
                    for (i, suggestion) in self.suggestions.iter().take(5).enumerate() {
                        if i > 0 {
                            ui.label(RichText::new("|").color(Color32::GRAY));
                        }
                        ui.label(RichText::new(suggestion)
                            .color(Color32::from_rgb(150, 200, 255))
                            .monospace()
                            .size(10.0));
                    }
                    ui.label(RichText::new("(Tab to complete)").color(Color32::GRAY).size(10.0));
                });
            }

            // Show error message if any
            if let Some(error) = &self.error {
                ui.colored_label(Color32::from_rgb(255, 100, 100), error);
            }
        });

        command_to_execute
    }
}

impl Default for CommandLine {
    fn default() -> Self {
        Self::new()
    }
}

/// Coordinate parser for various input formats
pub struct CoordinateParser;

impl CoordinateParser {
    /// Parse various coordinate formats
    pub fn parse(input: &str) -> Option<CoordinateInput> {
        let input = input.trim();

        // Absolute Cartesian: "100,200"
        if let Some((x_str, y_str)) = input.split_once(',') {
            if let (Ok(x), Ok(y)) = (x_str.trim().parse::<f64>(), y_str.trim().parse::<f64>()) {
                return Some(CoordinateInput::Absolute(x, y));
            }
        }

        // Relative Cartesian: "@100,200"
        if input.starts_with('@') {
            if let Some((x_str, y_str)) = input[1..].split_once(',') {
                if let (Ok(x), Ok(y)) = (x_str.trim().parse::<f64>(), y_str.trim().parse::<f64>()) {
                    return Some(CoordinateInput::Relative(x, y));
                }
            }
        }

        // Absolute Polar: "100<45"
        if let Some((dist_str, angle_str)) = input.split_once('<') {
            if let (Ok(dist), Ok(angle)) = (dist_str.trim().parse::<f64>(), angle_str.trim().parse::<f64>()) {
                return Some(CoordinateInput::Polar(dist, angle));
            }
        }

        // Relative Polar: "@100<45"
        if input.starts_with('@') {
            if let Some((dist_str, angle_str)) = input[1..].split_once('<') {
                if let (Ok(dist), Ok(angle)) = (dist_str.trim().parse::<f64>(), angle_str.trim().parse::<f64>()) {
                    return Some(CoordinateInput::RelativePolar(dist, angle));
                }
            }
        }

        None
    }
}

/// Coordinate input types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoordinateInput {
    /// Absolute Cartesian (x, y)
    Absolute(f64, f64),
    /// Relative Cartesian (@x, y)
    Relative(f64, f64),
    /// Absolute Polar (distance<angle)
    Polar(f64, f64),
    /// Relative Polar (@distance<angle)
    RelativePolar(f64, f64),
}

impl CoordinateInput {
    /// Convert to absolute Cartesian coordinates
    pub fn to_absolute(&self, last_point: Option<(f64, f64)>) -> (f64, f64) {
        match self {
            CoordinateInput::Absolute(x, y) => (*x, *y),
            CoordinateInput::Relative(dx, dy) => {
                if let Some((last_x, last_y)) = last_point {
                    (last_x + dx, last_y + dy)
                } else {
                    (*dx, *dy)
                }
            }
            CoordinateInput::Polar(dist, angle) => {
                let angle_rad = angle.to_radians();
                (dist * angle_rad.cos(), dist * angle_rad.sin())
            }
            CoordinateInput::RelativePolar(dist, angle) => {
                let angle_rad = angle.to_radians();
                let dx = dist * angle_rad.cos();
                let dy = dist * angle_rad.sin();

                if let Some((last_x, last_y)) = last_point {
                    (last_x + dx, last_y + dy)
                } else {
                    (dx, dy)
                }
            }
        }
    }
}
