// Grid System - Complete implementation
// Provides grid display and snapping functionality

use super::Point2;
use std::f64::consts::PI;

/// Grid display and snapping system
pub struct Grid {
    /// Grid settings
    pub settings: GridSettings,
    /// Is grid visible
    pub visible: bool,
    /// Is grid snapping enabled
    pub snap_enabled: bool,
    /// Current adaptive spacing (calculated based on zoom)
    adaptive_spacing: f64,
}

impl Grid {
    pub fn new() -> Self {
        Self {
            settings: GridSettings::default(),
            visible: true,
            snap_enabled: true,
            adaptive_spacing: 1.0,
        }
    }

    /// Snap point to grid
    pub fn snap_point(&self, point: Point2) -> Point2 {
        if !self.snap_enabled {
            return point;
        }

        match self.settings.grid_type {
            GridType::Rectangular => self.snap_rectangular(point),
            GridType::Polar => self.snap_polar(point),
        }
    }

    fn snap_rectangular(&self, point: Point2) -> Point2 {
        let spacing = if self.settings.adaptive {
            self.adaptive_spacing
        } else {
            self.settings.spacing
        };

        let x = (point.x / spacing).round() * spacing;
        let y = (point.y / spacing).round() * spacing;

        Point2::new(x, y)
    }

    fn snap_polar(&self, point: Point2) -> Point2 {
        // Snap to polar grid (concentric circles and radial lines)
        let angle = point.y.atan2(point.x);
        let radius = (point.x * point.x + point.y * point.y).sqrt();

        // Snap radius to grid spacing
        let spacing = if self.settings.adaptive {
            self.adaptive_spacing
        } else {
            self.settings.spacing
        };
        let snapped_radius = (radius / spacing).round() * spacing;

        // Snap angle to angular divisions
        let angle_spacing = 2.0 * PI / self.settings.polar_divisions as f64;
        let snapped_angle = (angle / angle_spacing).round() * angle_spacing;

        Point2::new(
            snapped_radius * snapped_angle.cos(),
            snapped_radius * snapped_angle.sin(),
        )
    }

    /// Update adaptive grid spacing based on zoom level
    pub fn update_adaptive_spacing(&mut self, pixel_size: f64, viewport_width: u32) {
        if !self.settings.adaptive {
            self.adaptive_spacing = self.settings.spacing;
            return;
        }

        // Calculate world units visible in viewport
        let visible_width = pixel_size * viewport_width as f64;

        // Find appropriate grid spacing (powers of 10, 2, or 5)
        let base_spacing = self.settings.spacing;
        let mut spacing = base_spacing;

        // Aim for 10-20 grid lines across viewport
        let target_lines = 15.0;
        let desired_spacing = visible_width / target_lines;

        // Find nearest "nice" number
        let magnitude = 10_f64.powf(desired_spacing.log10().floor());
        let normalized = desired_spacing / magnitude;

        spacing = if normalized < 1.5 {
            magnitude
        } else if normalized < 3.5 {
            magnitude * 2.0
        } else if normalized < 7.5 {
            magnitude * 5.0
        } else {
            magnitude * 10.0
        };

        self.adaptive_spacing = spacing;
    }

    /// Get grid lines for rendering
    pub fn get_grid_lines(&self, viewport: GridViewport) -> GridLines {
        match self.settings.grid_type {
            GridType::Rectangular => self.get_rectangular_lines(viewport),
            GridType::Polar => self.get_polar_lines(viewport),
        }
    }

    fn get_rectangular_lines(&self, viewport: GridViewport) -> GridLines {
        let spacing = if self.settings.adaptive {
            self.adaptive_spacing
        } else {
            self.settings.spacing
        };

        let mut major_lines = Vec::new();
        let mut minor_lines = Vec::new();

        // Calculate grid bounds
        let min_x = (viewport.min.x / spacing).floor() * spacing;
        let max_x = (viewport.max.x / spacing).ceil() * spacing;
        let min_y = (viewport.min.y / spacing).floor() * spacing;
        let max_y = (viewport.max.y / spacing).ceil() * spacing;

        // Generate vertical lines
        let mut x = min_x;
        let mut count = 0;
        while x <= max_x {
            let is_major = if self.settings.subdivisions > 0 {
                count % self.settings.subdivisions == 0
            } else {
                (x / spacing).abs() % 10.0 < 0.01
            };

            let line = GridLine {
                start: Point2::new(x, min_y),
                end: Point2::new(x, max_y),
                is_axis: x.abs() < 1e-10,
            };

            if is_major {
                major_lines.push(line);
            } else {
                minor_lines.push(line);
            }

            x += spacing;
            count += 1;
        }

        // Generate horizontal lines
        let mut y = min_y;
        count = 0;
        while y <= max_y {
            let is_major = if self.settings.subdivisions > 0 {
                count % self.settings.subdivisions == 0
            } else {
                (y / spacing).abs() % 10.0 < 0.01
            };

            let line = GridLine {
                start: Point2::new(min_x, y),
                end: Point2::new(max_x, y),
                is_axis: y.abs() < 1e-10,
            };

            if is_major {
                major_lines.push(line);
            } else {
                minor_lines.push(line);
            }

            y += spacing;
            count += 1;
        }

        GridLines {
            major: major_lines,
            minor: minor_lines,
            dots: Vec::new(),
        }
    }

    fn get_polar_lines(&self, viewport: GridViewport) -> GridLines {
        let spacing = if self.settings.adaptive {
            self.adaptive_spacing
        } else {
            self.settings.spacing
        };

        let mut major_lines = Vec::new();
        let mut minor_lines = Vec::new();

        // Calculate maximum radius needed
        let center = Point2::new(0.0, 0.0);
        let max_radius = viewport
            .min
            .distance_to(&center)
            .max(viewport.max.distance_to(&center))
            * 1.5;

        // Generate concentric circles
        let mut radius = spacing;
        let mut count = 1;
        while radius <= max_radius {
            let is_major = count % self.settings.subdivisions == 0;

            // Create circle as line segments
            let segments = 64;
            for i in 0..segments {
                let angle1 = 2.0 * PI * i as f64 / segments as f64;
                let angle2 = 2.0 * PI * (i + 1) as f64 / segments as f64;

                let p1 = Point2::new(radius * angle1.cos(), radius * angle1.sin());
                let p2 = Point2::new(radius * angle2.cos(), radius * angle2.sin());

                let line = GridLine {
                    start: p1,
                    end: p2,
                    is_axis: false,
                };

                if is_major {
                    major_lines.push(line);
                } else {
                    minor_lines.push(line);
                }
            }

            radius += spacing;
            count += 1;
        }

        // Generate radial lines
        let angle_spacing = 2.0 * PI / self.settings.polar_divisions as f64;
        for i in 0..self.settings.polar_divisions {
            let angle = i as f64 * angle_spacing;
            let end_x = max_radius * angle.cos();
            let end_y = max_radius * angle.sin();

            let line = GridLine {
                start: center,
                end: Point2::new(end_x, end_y),
                is_axis: i % (self.settings.polar_divisions / 4) == 0,
            };

            major_lines.push(line);
        }

        GridLines {
            major: major_lines,
            minor: minor_lines,
            dots: Vec::new(),
        }
    }

    /// Get grid dots for dot-style grid display
    pub fn get_grid_dots(&self, viewport: GridViewport) -> Vec<Point2> {
        let spacing = if self.settings.adaptive {
            self.adaptive_spacing
        } else {
            self.settings.spacing
        };

        let mut dots = Vec::new();

        let min_x = (viewport.min.x / spacing).floor() * spacing;
        let max_x = (viewport.max.x / spacing).ceil() * spacing;
        let min_y = (viewport.min.y / spacing).floor() * spacing;
        let max_y = (viewport.max.y / spacing).ceil() * spacing;

        let mut y = min_y;
        while y <= max_y {
            let mut x = min_x;
            while x <= max_x {
                dots.push(Point2::new(x, y));
                x += spacing;
            }
            y += spacing;
        }

        dots
    }

    /// Toggle grid visibility
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    /// Toggle grid snap
    pub fn toggle_snap(&mut self) {
        self.snap_enabled = !self.snap_enabled;
    }

    /// Set grid spacing
    pub fn set_spacing(&mut self, spacing: f64) {
        self.settings.spacing = spacing.max(0.001); // Prevent zero spacing
        self.adaptive_spacing = self.settings.spacing;
    }

    /// Set grid subdivisions
    pub fn set_subdivisions(&mut self, subdivisions: u32) {
        self.settings.subdivisions = subdivisions;
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self::new()
    }
}

/// Grid configuration settings
#[derive(Debug, Clone)]
pub struct GridSettings {
    /// Grid type (rectangular or polar)
    pub grid_type: GridType,
    /// Base grid spacing
    pub spacing: f64,
    /// Number of minor divisions between major lines
    pub subdivisions: u32,
    /// Enable adaptive grid (auto-adjust spacing based on zoom)
    pub adaptive: bool,
    /// Polar grid: number of angular divisions
    pub polar_divisions: u32,
    /// Display style
    pub display_style: GridDisplayStyle,
    /// Major line color
    pub major_color: [f32; 4],
    /// Minor line color
    pub minor_color: [f32; 4],
    /// Axis line color
    pub axis_color: [f32; 4],
    /// Major line width
    pub major_width: f32,
    /// Minor line width
    pub minor_width: f32,
    /// Axis line width
    pub axis_width: f32,
    /// Fade grid at distance
    pub fade_distance: bool,
}

impl Default for GridSettings {
    fn default() -> Self {
        Self {
            grid_type: GridType::Rectangular,
            spacing: 1.0,
            subdivisions: 10,
            adaptive: true,
            polar_divisions: 12,
            display_style: GridDisplayStyle::Lines,
            major_color: [0.4, 0.4, 0.4, 0.8],
            minor_color: [0.3, 0.3, 0.3, 0.5],
            axis_color: [0.6, 0.6, 0.6, 1.0],
            major_width: 1.5,
            minor_width: 0.8,
            axis_width: 2.0,
            fade_distance: true,
        }
    }
}

/// Type of grid
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridType {
    /// Standard rectangular grid
    Rectangular,
    /// Polar grid (concentric circles and radial lines)
    Polar,
}

/// Grid display style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridDisplayStyle {
    /// Draw grid as lines
    Lines,
    /// Draw grid as dots
    Dots,
    /// Draw grid as crosses
    Crosses,
}

/// Viewport bounds for grid generation
#[derive(Debug, Clone, Copy)]
pub struct GridViewport {
    pub min: Point2,
    pub max: Point2,
}

impl GridViewport {
    pub fn new(min: Point2, max: Point2) -> Self {
        Self { min, max }
    }

    pub fn from_center(center: Point2, width: f64, height: f64) -> Self {
        let half_w = width / 2.0;
        let half_h = height / 2.0;
        Self {
            min: Point2::new(center.x - half_w, center.y - half_h),
            max: Point2::new(center.x + half_w, center.y + half_h),
        }
    }

    pub fn width(&self) -> f64 {
        self.max.x - self.min.x
    }

    pub fn height(&self) -> f64 {
        self.max.y - self.min.y
    }

    pub fn center(&self) -> Point2 {
        Point2::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
        )
    }
}

/// Grid lines for rendering
#[derive(Debug, Clone)]
pub struct GridLines {
    /// Major grid lines
    pub major: Vec<GridLine>,
    /// Minor grid lines
    pub minor: Vec<GridLine>,
    /// Grid dots (for dot display style)
    pub dots: Vec<Point2>,
}

impl GridLines {
    pub fn new() -> Self {
        Self {
            major: Vec::new(),
            minor: Vec::new(),
            dots: Vec::new(),
        }
    }

    pub fn total_lines(&self) -> usize {
        self.major.len() + self.minor.len()
    }
}

impl Default for GridLines {
    fn default() -> Self {
        Self::new()
    }
}

/// Single grid line
#[derive(Debug, Clone, Copy)]
pub struct GridLine {
    pub start: Point2,
    pub end: Point2,
    pub is_axis: bool,
}

/// Polar grid implementation
pub struct PolarGrid {
    /// Grid settings
    settings: PolarGridSettings,
    /// Is grid visible
    visible: bool,
    /// Is snapping enabled
    snap_enabled: bool,
}

impl PolarGrid {
    pub fn new() -> Self {
        Self {
            settings: PolarGridSettings::default(),
            visible: true,
            snap_enabled: true,
        }
    }

    /// Snap point to polar grid
    pub fn snap_point(&self, point: Point2) -> Point2 {
        if !self.snap_enabled {
            return point;
        }

        let angle = point.y.atan2(point.x);
        let radius = (point.x * point.x + point.y * point.y).sqrt();

        // Snap radius
        let snapped_radius = (radius / self.settings.radial_spacing).round() * self.settings.radial_spacing;

        // Snap angle
        let angle_spacing = 2.0 * PI / self.settings.angular_divisions as f64;
        let snapped_angle = (angle / angle_spacing).round() * angle_spacing;

        Point2::new(
            snapped_radius * snapped_angle.cos(),
            snapped_radius * snapped_angle.sin(),
        )
    }

    /// Get closest grid point
    pub fn nearest_grid_point(&self, point: Point2) -> Point2 {
        self.snap_point(point)
    }
}

impl Default for PolarGrid {
    fn default() -> Self {
        Self::new()
    }
}

/// Polar grid settings
#[derive(Debug, Clone)]
pub struct PolarGridSettings {
    /// Radial spacing (distance between circles)
    pub radial_spacing: f64,
    /// Number of angular divisions
    pub angular_divisions: u32,
    /// Display concentric circles
    pub show_circles: bool,
    /// Display radial lines
    pub show_radials: bool,
}

impl Default for PolarGridSettings {
    fn default() -> Self {
        Self {
            radial_spacing: 1.0,
            angular_divisions: 12,
            show_circles: true,
            show_radials: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rectangular_snap() {
        let grid = Grid::new();
        let point = Point2::new(1.3, 2.7);
        let snapped = grid.snap_point(point);

        assert!((snapped.x - 1.0).abs() < 1e-10);
        assert!((snapped.y - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_grid_spacing() {
        let mut grid = Grid::new();
        grid.set_spacing(5.0);

        let point = Point2::new(7.0, 13.0);
        let snapped = grid.snap_point(point);

        assert!((snapped.x - 5.0).abs() < 1e-10);
        assert!((snapped.y - 15.0).abs() < 1e-10);
    }

    #[test]
    fn test_adaptive_spacing() {
        let mut grid = Grid::new();
        grid.settings.adaptive = true;
        grid.update_adaptive_spacing(1.0, 800);

        // Adaptive spacing should be calculated
        assert!(grid.adaptive_spacing > 0.0);
    }

    #[test]
    fn test_grid_viewport() {
        let viewport = GridViewport::from_center(Point2::zero(), 100.0, 100.0);

        assert!((viewport.width() - 100.0).abs() < 1e-10);
        assert!((viewport.height() - 100.0).abs() < 1e-10);

        let center = viewport.center();
        assert!((center.x - 0.0).abs() < 1e-10);
        assert!((center.y - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_polar_snap() {
        let mut grid = Grid::new();
        grid.settings.grid_type = GridType::Polar;
        grid.settings.polar_divisions = 12;
        grid.settings.spacing = 1.0;

        let point = Point2::new(1.3, 0.1);
        let snapped = grid.snap_point(point);

        // Should snap to nearest radial grid point
        let radius = (snapped.x * snapped.x + snapped.y * snapped.y).sqrt();
        assert!((radius - 1.0).abs() < 0.2); // Close to 1.0 radius
    }

    #[test]
    fn test_grid_lines_generation() {
        let grid = Grid::new();
        let viewport = GridViewport::new(Point2::new(-10.0, -10.0), Point2::new(10.0, 10.0));

        let lines = grid.get_grid_lines(viewport);

        assert!(lines.total_lines() > 0);
        assert!(!lines.major.is_empty());
    }
}
