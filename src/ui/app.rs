/// Main CADDY application
///
/// This module implements the main application state and lifecycle.
use egui::Context;

use super::{
    Canvas, CommandLine, StatusBar, DrawToolbar, ModifyToolbar, ViewToolbar,
    PropertiesPanel, LayersPanel, CommandPanel, UiState, theme::CaddyTheme,
    toolbar::Toolbar, panel::Panel,
};

/// Main CADDY application state
pub struct CaddyApp {
    /// UI state
    ui_state: UiState,

    /// Drawing canvas
    canvas: Canvas,

    /// Command line interface
    command_line: CommandLine,

    /// Status bar
    status_bar: StatusBar,

    /// Drawing toolbar
    draw_toolbar: DrawToolbar,

    /// Modify toolbar
    modify_toolbar: ModifyToolbar,

    /// View toolbar
    view_toolbar: ViewToolbar,

    /// Properties panel
    properties_panel: PropertiesPanel,

    /// Layers panel
    layers_panel: LayersPanel,

    /// Command history panel
    command_panel: CommandPanel,

    /// Document title
    document_title: String,

    /// Document modified flag
    document_modified: bool,

    /// Current file path (if saved)
    current_file: Option<String>,

    /// Active command (if any)
    active_command: Option<String>,

    /// Frame counter for animations
    frame_count: u64,
}

impl CaddyApp {
    /// Create a new CADDY application
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Apply dark theme
        CaddyTheme::apply(&cc.egui_ctx, true);

        Self {
            ui_state: UiState::default(),
            canvas: Canvas::new(),
            command_line: CommandLine::new(),
            status_bar: StatusBar::new(),
            draw_toolbar: DrawToolbar::new(),
            modify_toolbar: ModifyToolbar::new(),
            view_toolbar: ViewToolbar::new(),
            properties_panel: PropertiesPanel::new(),
            layers_panel: LayersPanel::new(),
            command_panel: CommandPanel::new(),
            document_title: "Untitled".to_string(),
            document_modified: false,
            current_file: None,
            active_command: None,
            frame_count: 0,
        }
    }

    /// Get window title with document name
    pub fn window_title(&self) -> String {
        let modified = if self.document_modified { "*" } else { "" };
        let filename = self.current_file
            .as_ref()
            .and_then(|p| std::path::Path::new(p).file_name())
            .and_then(|n| n.to_str())
            .unwrap_or(&self.document_title);
        format!("CADDY - {}{}", filename, modified)
    }

    /// Handle keyboard shortcuts
    fn handle_shortcuts(&mut self, ctx: &Context) {
        use crate::ui::shortcuts::StandardShortcuts;

        // Collect which shortcuts were triggered
        let (new_doc, open_doc, save_doc, save_as, undo_cmd, redo_cmd, toggle_grid, toggle_ortho, toggle_snap) =
            ctx.input_mut(|i| {
                (
                    i.consume_shortcut(&StandardShortcuts::NEW.into()),
                    i.consume_shortcut(&StandardShortcuts::OPEN.into()),
                    i.consume_shortcut(&StandardShortcuts::SAVE.into()),
                    i.consume_shortcut(&StandardShortcuts::SAVE_AS.into()),
                    i.consume_shortcut(&StandardShortcuts::UNDO.into()),
                    i.consume_shortcut(&StandardShortcuts::REDO.into()),
                    i.key_pressed(egui::Key::F7),
                    i.key_pressed(egui::Key::F8),
                    i.key_pressed(egui::Key::F9),
                )
            });

        // File operations
        if new_doc {
            self.new_document();
        }
        if open_doc {
            self.open_document();
        }
        if save_doc {
            self.save_document();
        }
        if save_as {
            self.save_document_as();
        }

        // Edit operations
        if undo_cmd {
            self.undo();
        }
        if redo_cmd {
            self.redo();
        }

        // Toggle modes
        if toggle_grid {
            self.ui_state.show_grid = !self.ui_state.show_grid;
        }
        if toggle_ortho {
            self.ui_state.ortho_mode = !self.ui_state.ortho_mode;
        }
        if toggle_snap {
            self.ui_state.snap_to_grid = !self.ui_state.snap_to_grid;
        }

        // Escape to cancel current command
        let escape_pressed = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        if escape_pressed {
            self.cancel_command();
        }
    }

    /// New document
    fn new_document(&mut self) {
        if self.document_modified {
            // TODO: Show save dialog
        }
        self.document_title = "Untitled".to_string();
        self.document_modified = false;
        self.current_file = None;
        self.canvas.clear();
        log::info!("New document created");
    }

    /// Open document
    fn open_document(&mut self) {
        // This will be handled by file dialog
        log::info!("Open document requested");
    }

    /// Save document
    fn save_document(&mut self) {
        if let Some(path) = &self.current_file {
            // TODO: Implement actual save
            log::info!("Saving to: {}", path);
            self.document_modified = false;
        } else {
            self.save_document_as();
        }
    }

    /// Save document as
    fn save_document_as(&mut self) {
        // This will be handled by file dialog
        log::info!("Save as requested");
    }

    /// Undo last operation
    fn undo(&mut self) {
        log::info!("Undo");
        // TODO: Implement undo
    }

    /// Redo last undone operation
    fn redo(&mut self) {
        log::info!("Redo");
        // TODO: Implement redo
    }

    /// Cancel current command
    fn cancel_command(&mut self) {
        self.active_command = None;
        log::info!("Command cancelled");
    }

    /// Execute a command from the command line
    pub fn execute_command(&mut self, command: &str) {
        log::info!("Executing command: {}", command);
        self.active_command = Some(command.to_string());
        self.command_panel.add_command(command);

        // Parse and execute command
        match command.to_uppercase().as_str() {
            "LINE" | "L" => self.start_line_command(),
            "CIRCLE" | "C" => self.start_circle_command(),
            "ARC" | "A" => self.start_arc_command(),
            "RECTANGLE" | "REC" => self.start_rectangle_command(),
            "ZOOM" | "Z" => self.canvas.zoom_extents(),
            "PAN" | "P" => self.start_pan_command(),
            "GRID" => self.ui_state.show_grid = !self.ui_state.show_grid,
            "SNAP" => self.ui_state.snap_to_grid = !self.ui_state.snap_to_grid,
            "ORTHO" => self.ui_state.ortho_mode = !self.ui_state.ortho_mode,
            _ => {
                log::warn!("Unknown command: {}", command);
                self.command_line.set_error(&format!("Unknown command: {}", command));
            }
        }
    }

    fn start_line_command(&mut self) {
        log::info!("Starting LINE command");
        self.command_line.set_prompt("Specify first point:");
    }

    fn start_circle_command(&mut self) {
        log::info!("Starting CIRCLE command");
        self.command_line.set_prompt("Specify center point:");
    }

    fn start_arc_command(&mut self) {
        log::info!("Starting ARC command");
        self.command_line.set_prompt("Specify start point:");
    }

    fn start_rectangle_command(&mut self) {
        log::info!("Starting RECTANGLE command");
        self.command_line.set_prompt("Specify first corner:");
    }

    fn start_pan_command(&mut self) {
        log::info!("Starting PAN command");
        self.ui_state.mode = crate::ui::InteractionMode::Pan;
    }
}

impl eframe::App for CaddyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.frame_count += 1;

        // Handle keyboard shortcuts
        self.handle_shortcuts(ctx);

        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New\tCtrl+N").clicked() {
                        self.new_document();
                        ui.close_menu();
                    }
                    if ui.button("Open...\tCtrl+O").clicked() {
                        self.open_document();
                        ui.close_menu();
                    }
                    if ui.button("Save\tCtrl+S").clicked() {
                        self.save_document();
                        ui.close_menu();
                    }
                    if ui.button("Save As...\tCtrl+Shift+S").clicked() {
                        self.save_document_as();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Export...").clicked() {
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo\tCtrl+Z").clicked() {
                        self.undo();
                        ui.close_menu();
                    }
                    if ui.button("Redo\tCtrl+Y").clicked() {
                        self.redo();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Cut\tCtrl+X").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Copy\tCtrl+C").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Paste\tCtrl+V").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Delete\tDel").clicked() {
                        ui.close_menu();
                    }
                });

                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.ui_state.show_grid, "Grid\tF7");
                    ui.checkbox(&mut self.ui_state.snap_to_grid, "Snap\tF9");
                    ui.checkbox(&mut self.ui_state.ortho_mode, "Ortho\tF8");
                    ui.separator();
                    if ui.button("Zoom Extents\tCtrl+E").clicked() {
                        self.canvas.zoom_extents();
                        ui.close_menu();
                    }
                    ui.separator();
                    ui.checkbox(&mut self.ui_state.show_layers, "Layers Panel");
                    ui.checkbox(&mut self.ui_state.show_properties, "Properties Panel");
                    ui.checkbox(&mut self.ui_state.show_command_history, "Command History");
                });

                ui.menu_button("Draw", |ui| {
                    if ui.button("Line\tL").clicked() {
                        self.execute_command("LINE");
                        ui.close_menu();
                    }
                    if ui.button("Circle\tC").clicked() {
                        self.execute_command("CIRCLE");
                        ui.close_menu();
                    }
                    if ui.button("Arc\tA").clicked() {
                        self.execute_command("ARC");
                        ui.close_menu();
                    }
                    if ui.button("Rectangle\tR").clicked() {
                        self.execute_command("RECTANGLE");
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Polyline").clicked() {
                        self.execute_command("PLINE");
                        ui.close_menu();
                    }
                    if ui.button("Polygon").clicked() {
                        self.execute_command("POLYGON");
                        ui.close_menu();
                    }
                });

                ui.menu_button("Modify", |ui| {
                    if ui.button("Move").clicked() {
                        self.execute_command("MOVE");
                        ui.close_menu();
                    }
                    if ui.button("Copy").clicked() {
                        self.execute_command("COPY");
                        ui.close_menu();
                    }
                    if ui.button("Rotate").clicked() {
                        self.execute_command("ROTATE");
                        ui.close_menu();
                    }
                    if ui.button("Scale").clicked() {
                        self.execute_command("SCALE");
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Mirror").clicked() {
                        self.execute_command("MIRROR");
                        ui.close_menu();
                    }
                    if ui.button("Array").clicked() {
                        self.execute_command("ARRAY");
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Trim").clicked() {
                        self.execute_command("TRIM");
                        ui.close_menu();
                    }
                    if ui.button("Extend").clicked() {
                        self.execute_command("EXTEND");
                        ui.close_menu();
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("Documentation").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("About CADDY").clicked() {
                        ui.close_menu();
                    }
                });
            });
        });

        // Drawing toolbar (left side)
        egui::SidePanel::left("draw_toolbar")
            .resizable(false)
            .default_width(60.0)
            .show(ctx, |ui| {
                self.draw_toolbar.show(ui, &mut self.ui_state);
            });

        // Modify toolbar (left side, below draw toolbar)
        egui::SidePanel::left("modify_toolbar")
            .resizable(false)
            .default_width(60.0)
            .show(ctx, |ui| {
                self.modify_toolbar.show(ui, &mut self.ui_state);
            });

        // Layers panel (right side)
        if self.ui_state.show_layers {
            egui::SidePanel::right("layers_panel")
                .resizable(true)
                .default_width(250.0)
                .show(ctx, |ui| {
                    self.layers_panel.show(ui, &mut self.ui_state);
                });
        }

        // Properties panel (right side)
        if self.ui_state.show_properties {
            egui::SidePanel::right("properties_panel")
                .resizable(true)
                .default_width(250.0)
                .show(ctx, |ui| {
                    self.properties_panel.show(ui, &mut self.ui_state);
                });
        }

        // Command history panel (right side, bottom)
        if self.ui_state.show_command_history {
            egui::SidePanel::right("command_panel")
                .resizable(true)
                .default_width(250.0)
                .show(ctx, |ui| {
                    self.command_panel.show(ui, &mut self.ui_state);
                });
        }

        // Command line (bottom)
        egui::TopBottomPanel::bottom("command_line")
            .resizable(false)
            .default_height(60.0)
            .show(ctx, |ui| {
                if let Some(cmd) = self.command_line.show(ui, &mut self.ui_state) {
                    self.execute_command(&cmd);
                }
            });

        // Status bar (very bottom)
        egui::TopBottomPanel::bottom("status_bar")
            .resizable(false)
            .default_height(25.0)
            .show(ctx, |ui| {
                self.status_bar.show(ui, &mut self.ui_state);
            });

        // Central canvas
        egui::CentralPanel::default().show(ctx, |ui| {
            self.canvas.show(ui, &mut self.ui_state);
        });

        // Request repaint for smooth animations
        ctx.request_repaint();
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        // Save application state
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Cleanup
    }
}
