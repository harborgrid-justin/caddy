/// Toolbars for CADDY - Drawing, Modify, and View toolbars
///
/// Provides icon-based toolbars similar to AutoCAD.
use egui::{Ui, Response, Vec2, Color32, Sense, Rect, Pos2, Stroke};
use super::UiState;

/// Toolbar position
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarPosition {
    Top,
    Left,
    Right,
    Bottom,
}

/// Base toolbar trait
pub trait Toolbar {
    fn show(&mut self, ui: &mut Ui, state: &mut UiState);
    fn position(&self) -> ToolbarPosition;
}

/// Tool button with icon and tooltip
struct ToolButton {
    label: &'static str,
    tooltip: &'static str,
    icon: ToolIcon,
    command: &'static str,
}

/// Simple icon representation (will be rendered as shapes)
#[derive(Debug, Clone, Copy)]
enum ToolIcon {
    Line,
    Circle,
    Arc,
    Rectangle,
    Polyline,
    Polygon,
    Ellipse,
    Spline,
    Move,
    Copy,
    Rotate,
    Scale,
    Mirror,
    Array,
    Trim,
    Extend,
    Offset,
    Fillet,
    Chamfer,
    Break,
    ZoomIn,
    ZoomOut,
    ZoomExtents,
    ZoomWindow,
    Pan,
    Orbit,
    ViewTop,
    ViewFront,
    ViewRight,
    View3D,
}

impl ToolButton {
    fn show(&self, ui: &mut Ui) -> Response {
        let button_size = Vec2::new(48.0, 48.0);
        let (rect, response) = ui.allocate_exact_size(button_size, Sense::click());

        let _visuals = ui.style().interact(&response);
        let bg_color = if response.hovered() {
            Color32::from_rgb(60, 60, 60)
        } else if response.is_pointer_button_down_on() {
            Color32::from_rgb(80, 80, 80)
        } else {
            Color32::from_rgb(40, 40, 40)
        };

        // Draw button background
        ui.painter().rect_filled(rect, 2.0, bg_color);

        // Draw icon
        self.draw_icon(ui, rect);

        // Draw border
        ui.painter().rect_stroke(
            rect,
            2.0,
            Stroke::new(1.0, Color32::from_rgb(70, 70, 70)),
        );

        // Show tooltip
        response.on_hover_text(self.tooltip)
    }

    fn draw_icon(&self, ui: &mut Ui, rect: Rect) {
        let painter = ui.painter();
        let center = rect.center();
        let color = Color32::from_rgb(200, 200, 200);
        let size = 20.0;

        match self.icon {
            ToolIcon::Line => {
                // Draw a line
                painter.line_segment(
                    [
                        Pos2::new(center.x - size / 2.0, center.y + size / 2.0),
                        Pos2::new(center.x + size / 2.0, center.y - size / 2.0),
                    ],
                    Stroke::new(2.0, color),
                );
            }
            ToolIcon::Circle => {
                // Draw a circle
                painter.circle_stroke(center, size / 2.0, Stroke::new(2.0, color));
            }
            ToolIcon::Arc => {
                // Draw an arc (partial circle)
                let points: Vec<Pos2> = (0..=8)
                    .map(|i| {
                        let angle = std::f32::consts::PI * (i as f32) / 8.0;
                        Pos2::new(
                            center.x + (size / 2.0) * angle.cos(),
                            center.y + (size / 2.0) * angle.sin(),
                        )
                    })
                    .collect();
                painter.add(egui::Shape::line(points, Stroke::new(2.0, color)));
            }
            ToolIcon::Rectangle => {
                // Draw a rectangle
                painter.rect_stroke(
                    Rect::from_center_size(center, Vec2::new(size, size * 0.7)),
                    0.0,
                    Stroke::new(2.0, color),
                );
            }
            ToolIcon::Polyline => {
                // Draw a polyline (connected lines)
                let points = vec![
                    Pos2::new(center.x - size / 2.0, center.y + size / 2.0),
                    Pos2::new(center.x - size / 4.0, center.y - size / 2.0),
                    Pos2::new(center.x + size / 4.0, center.y + size / 4.0),
                    Pos2::new(center.x + size / 2.0, center.y - size / 4.0),
                ];
                painter.add(egui::Shape::line(points, Stroke::new(2.0, color)));
            }
            ToolIcon::Polygon => {
                // Draw a hexagon
                let points: Vec<Pos2> = (0..=6)
                    .map(|i| {
                        let angle = std::f32::consts::PI * 2.0 * (i as f32) / 6.0 - std::f32::consts::PI / 2.0;
                        Pos2::new(
                            center.x + (size / 2.0) * angle.cos(),
                            center.y + (size / 2.0) * angle.sin(),
                        )
                    })
                    .collect();
                painter.add(egui::Shape::line(points, Stroke::new(2.0, color)));
            }
            ToolIcon::Ellipse => {
                // Draw an ellipse (as an oval) using circle_stroke as approximation
                let ellipse_shape = egui::epaint::EllipseShape::stroke(
                    center,
                    Vec2::new(size / 2.0, size / 3.0),
                    Stroke::new(2.0, color),
                );
                painter.add(ellipse_shape);
            }
            ToolIcon::Spline => {
                // Draw a curved line (bezier approximation)
                let points: Vec<Pos2> = (0..=10)
                    .map(|i| {
                        let t = (i as f32) / 10.0;
                        let x = center.x - size / 2.0 + size * t;
                        let y = center.y + (size / 4.0) * (t * std::f32::consts::PI * 2.0).sin();
                        Pos2::new(x, y)
                    })
                    .collect();
                painter.add(egui::Shape::line(points, Stroke::new(2.0, color)));
            }
            ToolIcon::Move => {
                // Draw cross with arrows
                painter.arrow(
                    Pos2::new(center.x - size / 3.0, center.y),
                    Vec2::new(size * 0.6, 0.0),
                    Stroke::new(2.0, color),
                );
                painter.arrow(
                    Pos2::new(center.x, center.y - size / 3.0),
                    Vec2::new(0.0, size * 0.6),
                    Stroke::new(2.0, color),
                );
            }
            ToolIcon::Copy => {
                // Draw two overlapping rectangles
                painter.rect_stroke(
                    Rect::from_center_size(
                        Pos2::new(center.x - 3.0, center.y - 3.0),
                        Vec2::new(size * 0.6, size * 0.6)
                    ),
                    0.0,
                    Stroke::new(2.0, color),
                );
                painter.rect_stroke(
                    Rect::from_center_size(
                        Pos2::new(center.x + 3.0, center.y + 3.0),
                        Vec2::new(size * 0.6, size * 0.6)
                    ),
                    0.0,
                    Stroke::new(2.0, color),
                );
            }
            ToolIcon::Rotate => {
                // Draw circular arrow
                let points: Vec<Pos2> = (0..=12)
                    .map(|i| {
                        let angle = std::f32::consts::PI * 1.5 * (i as f32) / 12.0;
                        Pos2::new(
                            center.x + (size / 2.0) * angle.cos(),
                            center.y + (size / 2.0) * angle.sin(),
                        )
                    })
                    .collect();
                painter.add(egui::Shape::line(points, Stroke::new(2.0, color)));
                // Arrow head
                painter.add(egui::Shape::convex_polygon(
                    vec![
                        Pos2::new(center.x + size / 2.0 - 3.0, center.y),
                        Pos2::new(center.x + size / 2.0 + 3.0, center.y - 3.0),
                        Pos2::new(center.x + size / 2.0 + 3.0, center.y + 3.0),
                    ],
                    color,
                    Stroke::NONE,
                ));
            }
            ToolIcon::Scale => {
                // Draw expanding squares
                painter.rect_stroke(
                    Rect::from_center_size(center, Vec2::new(size * 0.4, size * 0.4)),
                    0.0,
                    Stroke::new(2.0, color),
                );
                painter.rect_stroke(
                    Rect::from_center_size(center, Vec2::new(size * 0.7, size * 0.7)),
                    0.0,
                    Stroke::new(1.5, color.gamma_multiply(0.7)),
                );
            }
            ToolIcon::Mirror => {
                // Draw shape and its mirror
                let half = size / 4.0;
                painter.line_segment(
                    [Pos2::new(center.x, center.y - half), Pos2::new(center.x, center.y + half)],
                    Stroke::new(1.0, Color32::from_rgb(100, 100, 100)),
                );
                painter.circle_stroke(
                    Pos2::new(center.x - half, center.y),
                    half / 2.0,
                    Stroke::new(2.0, color),
                );
                painter.circle_stroke(
                    Pos2::new(center.x + half, center.y),
                    half / 2.0,
                    Stroke::new(2.0, color),
                );
            }
            ToolIcon::Array => {
                // Draw grid of dots
                for i in 0..3 {
                    for j in 0..3 {
                        painter.circle_filled(
                            Pos2::new(
                                center.x - size / 4.0 + (i as f32) * size / 4.0,
                                center.y - size / 4.0 + (j as f32) * size / 4.0,
                            ),
                            2.0,
                            color,
                        );
                    }
                }
            }
            ToolIcon::Trim => {
                // Draw scissor-like icon
                painter.line_segment(
                    [
                        Pos2::new(center.x - size / 2.0, center.y - size / 4.0),
                        Pos2::new(center.x + size / 2.0, center.y + size / 4.0),
                    ],
                    Stroke::new(2.0, color),
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x - size / 2.0, center.y + size / 4.0),
                        Pos2::new(center.x + size / 2.0, center.y - size / 4.0),
                    ],
                    Stroke::new(2.0, color),
                );
            }
            ToolIcon::Extend => {
                // Draw line with extension arrows
                painter.line_segment(
                    [
                        Pos2::new(center.x - size / 3.0, center.y),
                        Pos2::new(center.x + size / 3.0, center.y),
                    ],
                    Stroke::new(2.0, color),
                );
                painter.arrow(
                    Pos2::new(center.x - size / 3.0, center.y),
                    Vec2::new(-size / 4.0, 0.0),
                    Stroke::new(1.5, color),
                );
                painter.arrow(
                    Pos2::new(center.x + size / 3.0, center.y),
                    Vec2::new(size / 4.0, 0.0),
                    Stroke::new(1.5, color),
                );
            }
            ToolIcon::Offset => {
                // Draw parallel lines
                painter.line_segment(
                    [
                        Pos2::new(center.x - size / 2.0, center.y - size / 4.0),
                        Pos2::new(center.x + size / 2.0, center.y - size / 4.0),
                    ],
                    Stroke::new(2.0, color),
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x - size / 2.0, center.y + size / 4.0),
                        Pos2::new(center.x + size / 2.0, center.y + size / 4.0),
                    ],
                    Stroke::new(2.0, color),
                );
            }
            ToolIcon::Fillet => {
                // Draw rounded corner
                painter.line_segment(
                    [
                        Pos2::new(center.x - size / 2.0, center.y),
                        Pos2::new(center.x, center.y),
                    ],
                    Stroke::new(2.0, color),
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x, center.y),
                        Pos2::new(center.x, center.y - size / 2.0),
                    ],
                    Stroke::new(2.0, color),
                );
                let points: Vec<Pos2> = (0..=8)
                    .map(|i| {
                        let angle = std::f32::consts::PI / 2.0 * (i as f32) / 8.0;
                        Pos2::new(
                            center.x - (size / 4.0) * angle.sin(),
                            center.y - (size / 4.0) * angle.cos(),
                        )
                    })
                    .collect();
                painter.add(egui::Shape::line(points, Stroke::new(2.0, color)));
            }
            ToolIcon::Chamfer => {
                // Draw chamfered corner
                painter.line_segment(
                    [
                        Pos2::new(center.x - size / 2.0, center.y),
                        Pos2::new(center.x - size / 4.0, center.y),
                    ],
                    Stroke::new(2.0, color),
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x - size / 4.0, center.y),
                        Pos2::new(center.x, center.y - size / 4.0),
                    ],
                    Stroke::new(2.0, color),
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x, center.y - size / 4.0),
                        Pos2::new(center.x, center.y - size / 2.0),
                    ],
                    Stroke::new(2.0, color),
                );
            }
            ToolIcon::Break => {
                // Draw broken line
                painter.line_segment(
                    [
                        Pos2::new(center.x - size / 2.0, center.y),
                        Pos2::new(center.x - size / 8.0, center.y),
                    ],
                    Stroke::new(2.0, color),
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x + size / 8.0, center.y),
                        Pos2::new(center.x + size / 2.0, center.y),
                    ],
                    Stroke::new(2.0, color),
                );
            }
            ToolIcon::ZoomIn => {
                // Draw magnifying glass with +
                painter.circle_stroke(center, size / 3.0, Stroke::new(2.0, color));
                painter.line_segment(
                    [
                        Pos2::new(center.x - size / 6.0, center.y),
                        Pos2::new(center.x + size / 6.0, center.y),
                    ],
                    Stroke::new(2.0, color),
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x, center.y - size / 6.0),
                        Pos2::new(center.x, center.y + size / 6.0),
                    ],
                    Stroke::new(2.0, color),
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x + size / 4.0, center.y + size / 4.0),
                        Pos2::new(center.x + size / 2.0, center.y + size / 2.0),
                    ],
                    Stroke::new(2.0, color),
                );
            }
            ToolIcon::ZoomOut => {
                // Draw magnifying glass with -
                painter.circle_stroke(center, size / 3.0, Stroke::new(2.0, color));
                painter.line_segment(
                    [
                        Pos2::new(center.x - size / 6.0, center.y),
                        Pos2::new(center.x + size / 6.0, center.y),
                    ],
                    Stroke::new(2.0, color),
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x + size / 4.0, center.y + size / 4.0),
                        Pos2::new(center.x + size / 2.0, center.y + size / 2.0),
                    ],
                    Stroke::new(2.0, color),
                );
            }
            ToolIcon::ZoomExtents => {
                // Draw four corners expanding
                let offset = size / 3.0;
                painter.line_segment(
                    [
                        Pos2::new(center.x - offset, center.y - offset),
                        Pos2::new(center.x - offset + 5.0, center.y - offset),
                    ],
                    Stroke::new(2.0, color),
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x - offset, center.y - offset),
                        Pos2::new(center.x - offset, center.y - offset + 5.0),
                    ],
                    Stroke::new(2.0, color),
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x + offset, center.y - offset),
                        Pos2::new(center.x + offset - 5.0, center.y - offset),
                    ],
                    Stroke::new(2.0, color),
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x + offset, center.y - offset),
                        Pos2::new(center.x + offset, center.y - offset + 5.0),
                    ],
                    Stroke::new(2.0, color),
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x - offset, center.y + offset),
                        Pos2::new(center.x - offset + 5.0, center.y + offset),
                    ],
                    Stroke::new(2.0, color),
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x - offset, center.y + offset),
                        Pos2::new(center.x - offset, center.y + offset - 5.0),
                    ],
                    Stroke::new(2.0, color),
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x + offset, center.y + offset),
                        Pos2::new(center.x + offset - 5.0, center.y + offset),
                    ],
                    Stroke::new(2.0, color),
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x + offset, center.y + offset),
                        Pos2::new(center.x + offset, center.y + offset - 5.0),
                    ],
                    Stroke::new(2.0, color),
                );
            }
            ToolIcon::ZoomWindow => {
                // Draw zoom window
                painter.rect_stroke(
                    Rect::from_center_size(center, Vec2::new(size * 0.7, size * 0.5)),
                    0.0,
                    Stroke::new(2.0, color),
                );
            }
            ToolIcon::Pan => {
                // Draw hand icon (simplified)
                painter.circle_stroke(center, size / 3.0, Stroke::new(2.0, color));
                painter.arrow(
                    Pos2::new(center.x - size / 4.0, center.y),
                    Vec2::new(-size / 5.0, 0.0),
                    Stroke::new(1.5, color),
                );
                painter.arrow(
                    Pos2::new(center.x + size / 4.0, center.y),
                    Vec2::new(size / 5.0, 0.0),
                    Stroke::new(1.5, color),
                );
            }
            ToolIcon::Orbit => {
                // Draw 3D orbit icon
                painter.circle_stroke(center, size / 3.0, Stroke::new(2.0, color));
                let orbit_ellipse = egui::epaint::EllipseShape::stroke(
                    center,
                    Vec2::new(size / 3.0, size / 6.0),
                    Stroke::new(1.5, color.gamma_multiply(0.7)),
                );
                painter.add(orbit_ellipse);
            }
            ToolIcon::ViewTop => {
                // Draw top view icon (square with Z)
                painter.rect_stroke(
                    Rect::from_center_size(center, Vec2::new(size * 0.6, size * 0.6)),
                    0.0,
                    Stroke::new(2.0, color),
                );
            }
            ToolIcon::ViewFront => {
                // Draw front view icon
                painter.rect_stroke(
                    Rect::from_center_size(center, Vec2::new(size * 0.6, size * 0.6)),
                    0.0,
                    Stroke::new(2.0, color),
                );
            }
            ToolIcon::ViewRight => {
                // Draw right view icon
                painter.rect_stroke(
                    Rect::from_center_size(center, Vec2::new(size * 0.6, size * 0.6)),
                    0.0,
                    Stroke::new(2.0, color),
                );
            }
            ToolIcon::View3D => {
                // Draw 3D view icon (isometric cube)
                let s = size / 4.0;
                let points = vec![
                    Pos2::new(center.x - s, center.y),
                    Pos2::new(center.x, center.y - s * 0.6),
                    Pos2::new(center.x + s, center.y),
                    Pos2::new(center.x, center.y + s * 0.6),
                ];
                painter.add(egui::Shape::closed_line(points, Stroke::new(2.0, color)));
                painter.line_segment(
                    [Pos2::new(center.x, center.y - s * 0.6), Pos2::new(center.x, center.y + s)],
                    Stroke::new(1.5, color),
                );
            }
        }
    }
}

/// Drawing toolbar
pub struct DrawToolbar {
    buttons: Vec<ToolButton>,
}

impl DrawToolbar {
    pub fn new() -> Self {
        let buttons = vec![
            ToolButton {
                label: "Line",
                tooltip: "Line (L)",
                icon: ToolIcon::Line,
                command: "LINE",
            },
            ToolButton {
                label: "Circle",
                tooltip: "Circle (C)",
                icon: ToolIcon::Circle,
                command: "CIRCLE",
            },
            ToolButton {
                label: "Arc",
                tooltip: "Arc (A)",
                icon: ToolIcon::Arc,
                command: "ARC",
            },
            ToolButton {
                label: "Rectangle",
                tooltip: "Rectangle (REC)",
                icon: ToolIcon::Rectangle,
                command: "RECTANGLE",
            },
            ToolButton {
                label: "Polyline",
                tooltip: "Polyline (PL)",
                icon: ToolIcon::Polyline,
                command: "PLINE",
            },
            ToolButton {
                label: "Polygon",
                tooltip: "Polygon (POL)",
                icon: ToolIcon::Polygon,
                command: "POLYGON",
            },
            ToolButton {
                label: "Ellipse",
                tooltip: "Ellipse (EL)",
                icon: ToolIcon::Ellipse,
                command: "ELLIPSE",
            },
            ToolButton {
                label: "Spline",
                tooltip: "Spline (SPL)",
                icon: ToolIcon::Spline,
                command: "SPLINE",
            },
        ];

        Self { buttons }
    }
}

impl Toolbar for DrawToolbar {
    fn show(&mut self, ui: &mut Ui, _state: &mut UiState) {
        ui.heading("Draw");
        ui.separator();

        for button in &self.buttons {
            if button.show(ui).clicked() {
                log::info!("Draw tool clicked: {}", button.command);
            }
            ui.add_space(2.0);
        }
    }

    fn position(&self) -> ToolbarPosition {
        ToolbarPosition::Left
    }
}

/// Modify toolbar
pub struct ModifyToolbar {
    buttons: Vec<ToolButton>,
}

impl ModifyToolbar {
    pub fn new() -> Self {
        let buttons = vec![
            ToolButton {
                label: "Move",
                tooltip: "Move (M)",
                icon: ToolIcon::Move,
                command: "MOVE",
            },
            ToolButton {
                label: "Copy",
                tooltip: "Copy (CO)",
                icon: ToolIcon::Copy,
                command: "COPY",
            },
            ToolButton {
                label: "Rotate",
                tooltip: "Rotate (RO)",
                icon: ToolIcon::Rotate,
                command: "ROTATE",
            },
            ToolButton {
                label: "Scale",
                tooltip: "Scale (SC)",
                icon: ToolIcon::Scale,
                command: "SCALE",
            },
            ToolButton {
                label: "Mirror",
                tooltip: "Mirror (MI)",
                icon: ToolIcon::Mirror,
                command: "MIRROR",
            },
            ToolButton {
                label: "Array",
                tooltip: "Array (AR)",
                icon: ToolIcon::Array,
                command: "ARRAY",
            },
            ToolButton {
                label: "Trim",
                tooltip: "Trim (TR)",
                icon: ToolIcon::Trim,
                command: "TRIM",
            },
            ToolButton {
                label: "Extend",
                tooltip: "Extend (EX)",
                icon: ToolIcon::Extend,
                command: "EXTEND",
            },
            ToolButton {
                label: "Offset",
                tooltip: "Offset (O)",
                icon: ToolIcon::Offset,
                command: "OFFSET",
            },
            ToolButton {
                label: "Fillet",
                tooltip: "Fillet (F)",
                icon: ToolIcon::Fillet,
                command: "FILLET",
            },
            ToolButton {
                label: "Chamfer",
                tooltip: "Chamfer (CHA)",
                icon: ToolIcon::Chamfer,
                command: "CHAMFER",
            },
            ToolButton {
                label: "Break",
                tooltip: "Break (BR)",
                icon: ToolIcon::Break,
                command: "BREAK",
            },
        ];

        Self { buttons }
    }
}

impl Toolbar for ModifyToolbar {
    fn show(&mut self, ui: &mut Ui, _state: &mut UiState) {
        ui.heading("Modify");
        ui.separator();

        for button in &self.buttons {
            if button.show(ui).clicked() {
                log::info!("Modify tool clicked: {}", button.command);
            }
            ui.add_space(2.0);
        }
    }

    fn position(&self) -> ToolbarPosition {
        ToolbarPosition::Left
    }
}

/// View toolbar
pub struct ViewToolbar {
    buttons: Vec<ToolButton>,
}

impl ViewToolbar {
    pub fn new() -> Self {
        let buttons = vec![
            ToolButton {
                label: "Zoom In",
                tooltip: "Zoom In (Ctrl +)",
                icon: ToolIcon::ZoomIn,
                command: "ZOOM",
            },
            ToolButton {
                label: "Zoom Out",
                tooltip: "Zoom Out (Ctrl -)",
                icon: ToolIcon::ZoomOut,
                command: "ZOOM",
            },
            ToolButton {
                label: "Zoom Extents",
                tooltip: "Zoom Extents (Ctrl E)",
                icon: ToolIcon::ZoomExtents,
                command: "ZOOM",
            },
            ToolButton {
                label: "Zoom Window",
                tooltip: "Zoom Window (Z W)",
                icon: ToolIcon::ZoomWindow,
                command: "ZOOM",
            },
            ToolButton {
                label: "Pan",
                tooltip: "Pan (P)",
                icon: ToolIcon::Pan,
                command: "PAN",
            },
            ToolButton {
                label: "Orbit",
                tooltip: "Orbit (3DO)",
                icon: ToolIcon::Orbit,
                command: "3DORBIT",
            },
            ToolButton {
                label: "Top View",
                tooltip: "Top View",
                icon: ToolIcon::ViewTop,
                command: "VIEW",
            },
            ToolButton {
                label: "Front View",
                tooltip: "Front View",
                icon: ToolIcon::ViewFront,
                command: "VIEW",
            },
            ToolButton {
                label: "Right View",
                tooltip: "Right View",
                icon: ToolIcon::ViewRight,
                command: "VIEW",
            },
            ToolButton {
                label: "3D View",
                tooltip: "3D View",
                icon: ToolIcon::View3D,
                command: "VIEW",
            },
        ];

        Self { buttons }
    }
}

impl Toolbar for ViewToolbar {
    fn show(&mut self, ui: &mut Ui, _state: &mut UiState) {
        ui.heading("View");
        ui.separator();

        for button in &self.buttons {
            if button.show(ui).clicked() {
                log::info!("View tool clicked: {}", button.command);
            }
            ui.add_space(2.0);
        }
    }

    fn position(&self) -> ToolbarPosition {
        ToolbarPosition::Right
    }
}
