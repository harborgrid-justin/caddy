/// Main window management using winit
///
/// Handles window creation, event loop, and multiple window support.

use std::sync::Arc;
use winit::{
    event::WindowEvent,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
    dpi::{LogicalSize, PhysicalSize},
};

use super::CaddyApp;

/// Main window for CADDY application
pub struct MainWindow {
    /// Winit window
    window: Arc<Window>,
    /// Window title
    title: String,
    /// Window size
    size: PhysicalSize<u32>,
    /// Maximized state
    maximized: bool,
}

impl MainWindow {
    /// Create a new main window
    pub fn new(event_loop: &EventLoop<()>) -> anyhow::Result<Self> {
        // Create window with appropriate size
        let window = WindowBuilder::new()
            .with_title("CADDY - Enterprise CAD System")
            .with_inner_size(LogicalSize::new(1600, 900))
            .with_min_inner_size(LogicalSize::new(800, 600))
            .build(event_loop)?;

        let size = window.inner_size();

        Ok(Self {
            window: Arc::new(window),
            title: "CADDY - Enterprise CAD System".to_string(),
            size,
            maximized: false,
        })
    }

    /// Get window reference
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Get window Arc
    pub fn window_arc(&self) -> Arc<Window> {
        Arc::clone(&self.window)
    }

    /// Set window title
    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
        self.window.set_title(title);
    }

    /// Get current size
    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    /// Handle resize event
    pub fn handle_resize(&mut self, new_size: PhysicalSize<u32>) {
        self.size = new_size;
    }

    /// Toggle fullscreen
    pub fn toggle_fullscreen(&mut self) {
        if self.window.fullscreen().is_some() {
            self.window.set_fullscreen(None);
        } else {
            self.window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
        }
    }

    /// Maximize window
    pub fn maximize(&mut self) {
        self.window.set_maximized(true);
        self.maximized = true;
    }

    /// Minimize window
    pub fn minimize(&mut self) {
        self.window.set_minimized(true);
    }

    /// Center window on screen
    pub fn center(&self) {
        if let Some(monitor) = self.window.current_monitor() {
            let monitor_size = monitor.size();
            let window_size = self.window.outer_size();

            let x = (monitor_size.width.saturating_sub(window_size.width)) / 2;
            let y = (monitor_size.height.saturating_sub(window_size.height)) / 2;

            self.window.set_outer_position(winit::dpi::PhysicalPosition::new(x, y));
        }
    }
}

/// Window manager for handling multiple windows
pub struct WindowManager {
    /// Main window
    main_window: Option<MainWindow>,
    /// Additional windows (for multi-monitor support)
    additional_windows: Vec<MainWindow>,
}

impl WindowManager {
    /// Create new window manager
    pub fn new() -> Self {
        Self {
            main_window: None,
            additional_windows: Vec::new(),
        }
    }

    /// Create main window
    pub fn create_main_window(&mut self, event_loop: &EventLoop<()>) -> anyhow::Result<&MainWindow> {
        let window = MainWindow::new(event_loop)?;
        self.main_window = Some(window);
        Ok(self.main_window.as_ref().unwrap())
    }

    /// Get main window
    pub fn main_window(&self) -> Option<&MainWindow> {
        self.main_window.as_ref()
    }

    /// Get main window mutably
    pub fn main_window_mut(&mut self) -> Option<&mut MainWindow> {
        self.main_window.as_mut()
    }

    /// Create additional window
    pub fn create_window(&mut self, event_loop: &EventLoop<()>) -> anyhow::Result<usize> {
        let window = MainWindow::new(event_loop)?;
        self.additional_windows.push(window);
        Ok(self.additional_windows.len() - 1)
    }

    /// Get window count
    pub fn window_count(&self) -> usize {
        self.main_window.is_some() as usize + self.additional_windows.len()
    }
}

/// Run the CADDY application with eframe
pub fn run_app() -> anyhow::Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1600.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("CADDY - Enterprise CAD System")
            .with_icon(load_icon()),
        multisampling: 4,
        depth_buffer: 24,
        vsync: true,
        ..Default::default()
    };

    eframe::run_native(
        "CADDY",
        native_options,
        Box::new(|cc| Box::new(CaddyApp::new(cc))),
    ).map_err(|e| anyhow::anyhow!("Failed to run application: {}", e))
}

/// Load application icon
fn load_icon() -> Arc<egui::IconData> {
    // Create a simple icon (32x32 blue square with white 'C')
    // In production, load from PNG file
    let width = 32;
    let height = 32;
    let mut rgba = vec![0u8; width * height * 4];

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 4;
            // Blue background
            rgba[idx] = 30;      // R
            rgba[idx + 1] = 144; // G
            rgba[idx + 2] = 255; // B
            rgba[idx + 3] = 255; // A

            // Draw simple 'C' shape in white
            if (x > 8 && x < 24 && (y == 8 || y == 24)) ||
               (x == 8 && y > 8 && y < 24) {
                rgba[idx] = 255;     // R
                rgba[idx + 1] = 255; // G
                rgba[idx + 2] = 255; // B
            }
        }
    }

    Arc::new(egui::IconData {
        rgba,
        width: width as u32,
        height: height as u32,
    })
}

/// Window event handler
pub struct WindowEventHandler {
    /// Last cursor position
    last_cursor_pos: Option<(f64, f64)>,
    /// Mouse button states
    mouse_buttons: [bool; 3],
    /// Modifier keys state
    modifiers: winit::keyboard::ModifiersState,
}

impl WindowEventHandler {
    pub fn new() -> Self {
        Self {
            last_cursor_pos: None,
            mouse_buttons: [false; 3],
            modifiers: winit::keyboard::ModifiersState::empty(),
        }
    }

    /// Handle window event
    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.last_cursor_pos = Some((position.x, position.y));
                true
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let pressed = *state == winit::event::ElementState::Pressed;
                match button {
                    winit::event::MouseButton::Left => self.mouse_buttons[0] = pressed,
                    winit::event::MouseButton::Right => self.mouse_buttons[1] = pressed,
                    winit::event::MouseButton::Middle => self.mouse_buttons[2] = pressed,
                    _ => {}
                }
                true
            }
            WindowEvent::ModifiersChanged(new_modifiers) => {
                self.modifiers = new_modifiers.state();
                true
            }
            _ => false,
        }
    }

    /// Get last cursor position
    pub fn cursor_position(&self) -> Option<(f64, f64)> {
        self.last_cursor_pos
    }

    /// Check if mouse button is pressed
    pub fn is_mouse_button_pressed(&self, button: usize) -> bool {
        if button < 3 {
            self.mouse_buttons[button]
        } else {
            false
        }
    }

    /// Get modifiers state
    pub fn modifiers(&self) -> winit::keyboard::ModifiersState {
        self.modifiers
    }
}

impl Default for WindowEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Multiple document interface (MDI) support
pub struct MdiManager {
    /// Active document tabs
    documents: Vec<DocumentTab>,
    /// Active document index
    active_index: usize,
}

/// A document tab in MDI mode
pub struct DocumentTab {
    /// Document title
    pub title: String,
    /// File path (if saved)
    pub file_path: Option<String>,
    /// Modified flag
    pub modified: bool,
    /// Document ID
    pub id: uuid::Uuid,
}

impl MdiManager {
    pub fn new() -> Self {
        let mut docs = Vec::new();
        docs.push(DocumentTab {
            title: "Untitled".to_string(),
            file_path: None,
            modified: false,
            id: uuid::Uuid::new_v4(),
        });

        Self {
            documents: docs,
            active_index: 0,
        }
    }

    /// Create new document tab
    pub fn new_document(&mut self) -> uuid::Uuid {
        let id = uuid::Uuid::new_v4();
        self.documents.push(DocumentTab {
            title: format!("Untitled {}", self.documents.len()),
            file_path: None,
            modified: false,
            id,
        });
        self.active_index = self.documents.len() - 1;
        id
    }

    /// Close document tab
    pub fn close_document(&mut self, index: usize) -> bool {
        if self.documents.len() > 1 && index < self.documents.len() {
            self.documents.remove(index);
            if self.active_index >= self.documents.len() {
                self.active_index = self.documents.len() - 1;
            }
            true
        } else {
            false
        }
    }

    /// Get active document
    pub fn active_document(&self) -> Option<&DocumentTab> {
        self.documents.get(self.active_index)
    }

    /// Get active document mutably
    pub fn active_document_mut(&mut self) -> Option<&mut DocumentTab> {
        self.documents.get_mut(self.active_index)
    }

    /// Set active document
    pub fn set_active(&mut self, index: usize) {
        if index < self.documents.len() {
            self.active_index = index;
        }
    }

    /// Get all documents
    pub fn documents(&self) -> &[DocumentTab] {
        &self.documents
    }

    /// Get document count
    pub fn count(&self) -> usize {
        self.documents.len()
    }
}

impl Default for MdiManager {
    fn default() -> Self {
        Self::new()
    }
}
