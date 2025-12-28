/// Drawing canvas widget for CADDY
///
/// Handles the main drawing area with mouse/keyboard interaction,
/// grid display, crosshair cursor, and context menus.

use egui::{Ui, Sense, Rect, Pos2, Vec2, Color32, Stroke, Response, Key};
use super::{UiState, InteractionMode, theme};

/// Drawing canvas widget
pub struct Canvas {
    /// Canvas size
    size: Vec2,
    /// Zoom level
    zoom: f32,
    /// Pan offset (in world coordinates)
    pan_offset: Vec2,
    /// Last mouse position
    last_mouse_pos: Option<Pos2>,
    /// Mouse down position (for drag operations)
    mouse_down_pos: Option<Pos2>,
    /// Is panning
    is_panning: bool,
    /// Is zooming
    is_zooming: bool,
    /// Grid spacing (in world units)
    grid_spacing: f32,
    /// Crosshair enabled
    show_crosshair: bool,
    /// Context menu position
    context_menu_pos: Option<Pos2>,
    /// Selected entities (simplified - will be entity IDs)
    selected_entities: Vec<usize>,
    /// Hover entity
    hover_entity: Option<usize>,
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            size: Vec2::new(800.0, 600.0),
            zoom: 1.0,
            pan_offset: Vec2::ZERO,
            last_mouse_pos: None,
            mouse_down_pos: None,
            is_panning: false,
            is_zooming: false,
            grid_spacing: 10.0,
            show_crosshair: true,
            context_menu_pos: None,
            selected_entities: Vec::new(),
            hover_entity: None,
        }
    }

    /// Clear canvas
    pub fn clear(&mut self) {
        self.selected_entities.clear();
        self.hover_entity = None;
    }

    /// Zoom to extents
    pub fn zoom_extents(&mut self) {
        self.zoom = 1.0;
        self.pan_offset = Vec2::ZERO;
        log::info!("Zoom extents");
    }

    /// Convert screen coordinates to world coordinates
    fn screen_to_world(&self, screen_pos: Pos2, canvas_rect: Rect) -> Pos2 {
        let rel_x = screen_pos.x - canvas_rect.left();
        let rel_y = screen_pos.y - canvas_rect.top();

        let world_x = (rel_x - canvas_rect.width() / 2.0) / self.zoom + self.pan_offset.x;
        let world_y = -(rel_y - canvas_rect.height() / 2.0) / self.zoom + self.pan_offset.y;

        Pos2::new(world_x, world_y)
    }

    /// Convert world coordinates to screen coordinates
    fn world_to_screen(&self, world_pos: Pos2, canvas_rect: Rect) -> Pos2 {
        let screen_x = (world_pos.x - self.pan_offset.x) * self.zoom + canvas_rect.left() + canvas_rect.width() / 2.0;
        let screen_y = -(world_pos.y - self.pan_offset.y) * self.zoom + canvas_rect.top() + canvas_rect.height() / 2.0;

        Pos2::new(screen_x, screen_y)
    }

    /// Snap point to grid
    fn snap_to_grid(&self, pos: Pos2, state: &UiState) -> Pos2 {
        if !state.snap_to_grid {
            return pos;
        }

        let spacing = self.grid_spacing;
        let snapped_x = (pos.x / spacing).round() * spacing;
        let snapped_y = (pos.y / spacing).round() * spacing;

        Pos2::new(snapped_x, snapped_y)
    }

    /// Apply ortho constraint
    fn apply_ortho(&self, start: Pos2, current: Pos2, state: &UiState) -> Pos2 {
        if !state.ortho_mode {
            return current;
        }

        let dx = (current.x - start.x).abs();
        let dy = (current.y - start.y).abs();

        if dx > dy {
            // Horizontal
            Pos2::new(current.x, start.y)
        } else {
            // Vertical
            Pos2::new(start.x, current.y)
        }
    }

    /// Draw grid
    fn draw_grid(&self, ui: &mut Ui, canvas_rect: Rect, state: &UiState) {
        if !state.show_grid {
            return;
        }

        let painter = ui.painter();
        let spacing = self.grid_spacing * self.zoom;

        // Don't draw if too dense or too sparse
        if spacing < 2.0 || spacing > 200.0 {
            return;
        }

        // Calculate grid range
        let center = canvas_rect.center();
        let start_x = (canvas_rect.left() - center.x) / spacing;
        let end_x = (canvas_rect.right() - center.x) / spacing;
        let start_y = (canvas_rect.top() - center.y) / spacing;
        let end_y = (canvas_rect.bottom() - center.y) / spacing;

        // Draw vertical lines
        for i in start_x.floor() as i32..=end_x.ceil() as i32 {
            let x = center.x + i as f32 * spacing;
            if x >= canvas_rect.left() && x <= canvas_rect.right() {
                let color = if i == 0 {
                    theme::AXIS_X_COLOR.gamma_multiply(0.3)
                } else {
                    theme::GRID_COLOR
                };

                painter.line_segment(
                    [
                        Pos2::new(x, canvas_rect.top()),
                        Pos2::new(x, canvas_rect.bottom()),
                    ],
                    Stroke::new(if i == 0 { 1.5 } else { 0.5 }, color),
                );
            }
        }

        // Draw horizontal lines
        for i in start_y.floor() as i32..=end_y.ceil() as i32 {
            let y = center.y + i as f32 * spacing;
            if y >= canvas_rect.top() && y <= canvas_rect.bottom() {
                let color = if i == 0 {
                    theme::AXIS_Y_COLOR.gamma_multiply(0.3)
                } else {
                    theme::GRID_COLOR
                };

                painter.line_segment(
                    [
                        Pos2::new(canvas_rect.left(), y),
                        Pos2::new(canvas_rect.right(), y),
                    ],
                    Stroke::new(if i == 0 { 1.5 } else { 0.5 }, color),
                );
            }
        }
    }

    /// Draw axes
    fn draw_axes(&self, ui: &mut Ui, canvas_rect: Rect) {
        let painter = ui.painter();
        let origin = self.world_to_screen(Pos2::new(0.0, 0.0), canvas_rect);

        // X axis (red)
        if origin.x >= canvas_rect.left() && origin.x <= canvas_rect.right() {
            painter.line_segment(
                [
                    Pos2::new(origin.x, canvas_rect.top()),
                    Pos2::new(origin.x, canvas_rect.bottom()),
                ],
                Stroke::new(2.0, theme::AXIS_X_COLOR),
            );
        }

        // Y axis (green)
        if origin.y >= canvas_rect.top() && origin.y <= canvas_rect.bottom() {
            painter.line_segment(
                [
                    Pos2::new(canvas_rect.left(), origin.y),
                    Pos2::new(canvas_rect.right(), origin.y),
                ],
                Stroke::new(2.0, theme::AXIS_Y_COLOR),
            );
        }
    }

    /// Draw crosshair cursor
    fn draw_crosshair(&self, ui: &mut Ui, canvas_rect: Rect, mouse_pos: Pos2) {
        if !self.show_crosshair || !canvas_rect.contains(mouse_pos) {
            return;
        }

        let painter = ui.painter();
        let size = 20.0;
        let color = theme::CROSSHAIR_COLOR;

        // Horizontal line
        painter.line_segment(
            [
                Pos2::new(mouse_pos.x - size, mouse_pos.y),
                Pos2::new(mouse_pos.x + size, mouse_pos.y),
            ],
            Stroke::new(1.0, color),
        );

        // Vertical line
        painter.line_segment(
            [
                Pos2::new(mouse_pos.x, mouse_pos.y - size),
                Pos2::new(mouse_pos.x, mouse_pos.y + size),
            ],
            Stroke::new(1.0, color),
        );
    }

    /// Draw sample entities (for demonstration)
    fn draw_entities(&self, ui: &mut Ui, canvas_rect: Rect) {
        let painter = ui.painter();

        // Draw a sample line
        let p1 = self.world_to_screen(Pos2::new(-50.0, -50.0), canvas_rect);
        let p2 = self.world_to_screen(Pos2::new(50.0, 50.0), canvas_rect);
        painter.line_segment([p1, p2], Stroke::new(2.0, Color32::WHITE));

        // Draw a sample circle
        let center = self.world_to_screen(Pos2::new(0.0, 0.0), canvas_rect);
        let radius = 30.0 * self.zoom;
        painter.circle_stroke(center, radius, Stroke::new(2.0, Color32::from_rgb(0, 180, 255)));

        // Draw a sample rectangle
        let corner1 = self.world_to_screen(Pos2::new(-80.0, 30.0), canvas_rect);
        let corner2 = self.world_to_screen(Pos2::new(-30.0, 80.0), canvas_rect);
        painter.rect_stroke(
            Rect::from_two_pos(corner1, corner2),
            0.0,
            Stroke::new(2.0, Color32::from_rgb(255, 200, 0)),
        );
    }

    /// Handle mouse input
    fn handle_mouse_input(&mut self, response: &Response, canvas_rect: Rect, state: &mut UiState) {
        // Get current mouse position
        if let Some(mouse_pos) = response.hover_pos() {
            self.last_mouse_pos = Some(mouse_pos);

            // Update cursor position in state (world coordinates)
            let world_pos = self.screen_to_world(mouse_pos, canvas_rect);
            let snapped_pos = self.snap_to_grid(world_pos, state);
            state.cursor_pos = (snapped_pos.x as f64, snapped_pos.y as f64);

            // Handle panning with middle mouse button or space + left mouse
            if response.dragged_by(egui::PointerButton::Middle) ||
               (response.dragged_by(egui::PointerButton::Primary) && state.mode == InteractionMode::Pan) {
                let delta = response.drag_delta();
                self.pan_offset.x -= delta.x / self.zoom;
                self.pan_offset.y += delta.y / self.zoom;
                self.is_panning = true;
            } else {
                self.is_panning = false;
            }

            // Handle zooming with scroll wheel
            let scroll = response.ctx.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                let zoom_factor = 1.0 + scroll * 0.001;
                self.zoom *= zoom_factor;
                self.zoom = self.zoom.clamp(0.1, 100.0);
            }

            // Handle left click (selection or drawing)
            if response.clicked_by(egui::PointerButton::Primary) {
                log::info!("Canvas clicked at world: ({:.2}, {:.2})", world_pos.x, world_pos.y);
                self.mouse_down_pos = Some(mouse_pos);
            }

            // Handle right click (context menu)
            if response.clicked_by(egui::PointerButton::Secondary) {
                self.context_menu_pos = Some(mouse_pos);
            }
        }
    }

    /// Handle keyboard input
    fn handle_keyboard_input(&mut self, ui: &Ui, state: &mut UiState) {
        ui.input(|i| {
            // Pan with arrow keys
            let pan_speed = 10.0 / self.zoom;
            if i.key_down(Key::ArrowLeft) {
                self.pan_offset.x -= pan_speed;
            }
            if i.key_down(Key::ArrowRight) {
                self.pan_offset.x += pan_speed;
            }
            if i.key_down(Key::ArrowUp) {
                self.pan_offset.y += pan_speed;
            }
            if i.key_down(Key::ArrowDown) {
                self.pan_offset.y -= pan_speed;
            }

            // Zoom with + and -
            if i.key_pressed(Key::Plus) || i.key_pressed(Key::Equals) {
                self.zoom *= 1.2;
                self.zoom = self.zoom.clamp(0.1, 100.0);
            }
            if i.key_pressed(Key::Minus) {
                self.zoom /= 1.2;
                self.zoom = self.zoom.clamp(0.1, 100.0);
            }
        });
    }

    /// Show context menu
    fn show_context_menu(&mut self, ui: &Ui) {
        if let Some(pos) = self.context_menu_pos {
            egui::Area::new("canvas_context_menu".into())
                .fixed_pos(pos)
                .order(egui::Order::Foreground)
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.set_min_width(150.0);

                        if ui.button("Zoom Extents").clicked() {
                            self.zoom_extents();
                            self.context_menu_pos = None;
                        }

                        ui.separator();

                        if ui.button("Pan").clicked() {
                            self.context_menu_pos = None;
                        }

                        if ui.button("Zoom Window").clicked() {
                            self.context_menu_pos = None;
                        }

                        ui.separator();

                        if ui.button("Properties").clicked() {
                            self.context_menu_pos = None;
                        }

                        ui.separator();

                        if ui.button("Cancel").clicked() {
                            self.context_menu_pos = None;
                        }
                    });
                });

            // Close context menu if clicked elsewhere
            if ui.input(|i| i.pointer.any_click()) {
                self.context_menu_pos = None;
            }
        }
    }

    /// Show canvas
    pub fn show(&mut self, ui: &mut Ui, state: &mut UiState) {
        let available_size = ui.available_size();
        self.size = available_size;

        let (response, painter) = ui.allocate_painter(available_size, Sense::click_and_drag());
        let canvas_rect = response.rect;

        // Fill background
        painter.rect_filled(
            canvas_rect,
            0.0,
            Color32::from_rgb(20, 20, 20),
        );

        // Draw grid
        self.draw_grid(ui, canvas_rect, state);

        // Draw axes
        self.draw_axes(ui, canvas_rect);

        // Draw entities (sample)
        self.draw_entities(ui, canvas_rect);

        // Draw crosshair
        if let Some(mouse_pos) = self.last_mouse_pos {
            self.draw_crosshair(ui, canvas_rect, mouse_pos);
        }

        // Handle input
        self.handle_mouse_input(&response, canvas_rect, state);
        self.handle_keyboard_input(ui, state);

        // Show context menu
        self.show_context_menu(ui);

        // Show coordinate display
        if let Some(mouse_pos) = self.last_mouse_pos {
            if canvas_rect.contains(mouse_pos) {
                let world_pos = self.screen_to_world(mouse_pos, canvas_rect);
                let snapped_pos = self.snap_to_grid(world_pos, state);

                // Draw coordinate tooltip near cursor
                let tooltip_pos = Pos2::new(mouse_pos.x + 15.0, mouse_pos.y - 20.0);
                painter.text(
                    tooltip_pos,
                    egui::Align2::LEFT_BOTTOM,
                    format!("X: {:.2}  Y: {:.2}", snapped_pos.x, snapped_pos.y),
                    egui::FontId::monospace(12.0),
                    Color32::from_rgb(200, 200, 200),
                );
            }
        }

        // Draw zoom level indicator
        let zoom_text = format!("Zoom: {:.1}x", self.zoom);
        painter.text(
            Pos2::new(canvas_rect.right() - 10.0, canvas_rect.top() + 10.0),
            egui::Align2::RIGHT_TOP,
            zoom_text,
            egui::FontId::monospace(12.0),
            Color32::from_rgb(150, 150, 150),
        );
    }
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}

/// Selection box for multiple entity selection
pub struct SelectionBox {
    start: Option<Pos2>,
    current: Option<Pos2>,
    active: bool,
}

impl SelectionBox {
    pub fn new() -> Self {
        Self {
            start: None,
            current: None,
            active: false,
        }
    }

    pub fn start(&mut self, pos: Pos2) {
        self.start = Some(pos);
        self.current = Some(pos);
        self.active = true;
    }

    pub fn update(&mut self, pos: Pos2) {
        if self.active {
            self.current = Some(pos);
        }
    }

    pub fn finish(&mut self) -> Option<Rect> {
        if let (Some(start), Some(end)) = (self.start, self.current) {
            self.active = false;
            self.start = None;
            self.current = None;
            Some(Rect::from_two_pos(start, end))
        } else {
            None
        }
    }

    pub fn cancel(&mut self) {
        self.active = false;
        self.start = None;
        self.current = None;
    }

    pub fn draw(&self, painter: &egui::Painter) {
        if let (Some(start), Some(end)) = (self.start, self.current) {
            let rect = Rect::from_two_pos(start, end);

            // Draw selection box
            painter.rect_stroke(
                rect,
                0.0,
                Stroke::new(1.0, theme::SELECTION_COLOR),
            );

            // Fill with transparent color
            painter.rect_filled(
                rect,
                0.0,
                theme::SELECTION_COLOR.gamma_multiply(0.1),
            );
        }
    }
}

impl Default for SelectionBox {
    fn default() -> Self {
        Self::new()
    }
}
