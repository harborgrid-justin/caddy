// Orthographic Mode and Polar Tracking - Complete implementation
// Constrains input to orthogonal or polar angles

use super::{Point2, Vector2};
use std::f64::consts::PI;

/// Orthographic mode controller
#[derive(Debug, Clone)]
pub struct OrthoMode {
    /// Is ortho mode enabled
    pub enabled: bool,
    /// Ortho constraint angles (in radians)
    pub angles: Vec<f64>,
    /// Current constraint angle
    current_angle: Option<f64>,
    /// Polar tracking
    pub polar_tracking: PolarTracking,
    /// Snap tracking
    pub snap_tracking: SnapTracking,
}

impl OrthoMode {
    pub fn new() -> Self {
        Self {
            enabled: false,
            angles: vec![0.0, PI / 2.0, PI, 3.0 * PI / 2.0], // 0°, 90°, 180°, 270°
            current_angle: None,
            polar_tracking: PolarTracking::new(),
            snap_tracking: SnapTracking::new(),
        }
    }

    /// Constrain point to orthogonal direction from base point
    pub fn constrain_point(&mut self, from: Point2, to: Point2) -> Point2 {
        if !self.enabled && !self.polar_tracking.enabled {
            return to;
        }

        let dx = to.x - from.x;
        let dy = to.y - from.y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance < 1e-10 {
            return to;
        }

        let angle = dy.atan2(dx);

        // Find nearest constraint angle
        let constraint_angle = if self.polar_tracking.enabled {
            self.find_nearest_polar_angle(angle)
        } else {
            self.find_nearest_ortho_angle(angle)
        };

        self.current_angle = Some(constraint_angle);

        // Project point onto constraint angle
        Point2::new(
            from.x + distance * constraint_angle.cos(),
            from.y + distance * constraint_angle.sin(),
        )
    }

    fn find_nearest_ortho_angle(&self, angle: f64) -> f64 {
        let mut normalized = angle;
        while normalized < 0.0 {
            normalized += 2.0 * PI;
        }
        while normalized >= 2.0 * PI {
            normalized -= 2.0 * PI;
        }

        let mut nearest = self.angles[0];
        let mut min_diff = (normalized - nearest).abs();

        for &ortho_angle in &self.angles {
            let diff = (normalized - ortho_angle).abs();
            if diff < min_diff {
                min_diff = diff;
                nearest = ortho_angle;
            }
        }

        nearest
    }

    fn find_nearest_polar_angle(&self, angle: f64) -> f64 {
        self.polar_tracking.find_nearest_angle(angle)
    }

    /// Toggle ortho mode
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
        if self.enabled {
            self.polar_tracking.enabled = false; // Disable polar when ortho enabled
        }
    }

    /// Get current constraint angle
    pub fn current_angle(&self) -> Option<f64> {
        self.current_angle
    }

    /// Get constraint line for display
    pub fn get_constraint_line(&self, from: Point2, length: f64) -> Option<(Point2, Point2)> {
        if let Some(angle) = self.current_angle {
            let end = Point2::new(
                from.x + length * angle.cos(),
                from.y + length * angle.sin(),
            );
            Some((from, end))
        } else {
            None
        }
    }

    /// Clear current constraint
    pub fn clear(&mut self) {
        self.current_angle = None;
    }
}

impl Default for OrthoMode {
    fn default() -> Self {
        Self::new()
    }
}

/// Polar tracking - track at specific angular increments
#[derive(Debug, Clone)]
pub struct PolarTracking {
    /// Is polar tracking enabled
    pub enabled: bool,
    /// Angular increment in degrees
    pub increment: f64,
    /// Additional tracking angles
    pub additional_angles: Vec<f64>,
    /// Tracking aperture (tolerance in radians)
    pub aperture: f64,
    /// Display settings
    pub settings: PolarTrackingSettings,
}

impl PolarTracking {
    pub fn new() -> Self {
        Self {
            enabled: false,
            increment: 45.0, // 45 degrees
            additional_angles: Vec::new(),
            aperture: 5.0_f64.to_radians(), // 5 degrees tolerance
            settings: PolarTrackingSettings::default(),
        }
    }

    /// Find nearest polar tracking angle
    pub fn find_nearest_angle(&self, angle: f64) -> f64 {
        if !self.enabled {
            return angle;
        }

        let increment_rad = self.increment.to_radians();

        // Normalize angle to 0..2π
        let mut normalized = angle;
        while normalized < 0.0 {
            normalized += 2.0 * PI;
        }
        while normalized >= 2.0 * PI {
            normalized -= 2.0 * PI;
        }

        // Find nearest increment
        let steps = (normalized / increment_rad).round();
        let snap_angle = steps * increment_rad;

        // Check if within aperture
        let diff = (normalized - snap_angle).abs();
        if diff <= self.aperture {
            snap_angle
        } else {
            // Check additional angles
            for &add_angle in &self.additional_angles {
                let diff = (normalized - add_angle).abs();
                if diff <= self.aperture {
                    return add_angle;
                }
            }
            angle // Return original if no snap
        }
    }

    /// Add tracking angle
    pub fn add_angle(&mut self, angle: f64) {
        self.additional_angles.push(angle);
    }

    /// Clear additional angles
    pub fn clear_angles(&mut self) {
        self.additional_angles.clear();
    }

    /// Set increment
    pub fn set_increment(&mut self, degrees: f64) {
        self.increment = degrees.clamp(1.0, 360.0);
    }

    /// Toggle polar tracking
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    /// Get tracking angles for display
    pub fn get_tracking_angles(&self) -> Vec<f64> {
        let increment_rad = self.increment.to_radians();
        let count = (2.0 * PI / increment_rad).round() as usize;
        let mut angles = Vec::with_capacity(count);

        for i in 0..count {
            angles.push(i as f64 * increment_rad);
        }

        angles.extend(&self.additional_angles);
        angles
    }
}

impl Default for PolarTracking {
    fn default() -> Self {
        Self::new()
    }
}

/// Polar tracking display settings
#[derive(Debug, Clone)]
pub struct PolarTrackingSettings {
    /// Show tracking vectors
    pub show_vectors: bool,
    /// Show angle tooltip
    pub show_tooltip: bool,
    /// Tracking line color
    pub line_color: [f32; 4],
    /// Tracking line width
    pub line_width: f32,
    /// Tracking line style
    pub line_style: LineStyle,
}

impl Default for PolarTrackingSettings {
    fn default() -> Self {
        Self {
            show_vectors: true,
            show_tooltip: true,
            line_color: [1.0, 1.0, 0.0, 0.6], // Yellow transparent
            line_width: 1.0,
            line_style: LineStyle::Dashed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStyle {
    Solid,
    Dashed,
    Dotted,
}

/// Object snap tracking - track alignment with snap points
#[derive(Debug, Clone)]
pub struct SnapTracking {
    /// Is snap tracking enabled
    pub enabled: bool,
    /// Tracked points (from previous snaps)
    pub tracked_points: Vec<Point2>,
    /// Maximum tracked points
    pub max_tracked: usize,
    /// Tracking tolerance
    pub tolerance: f64,
    /// Display settings
    pub settings: SnapTrackingSettings,
}

impl SnapTracking {
    pub fn new() -> Self {
        Self {
            enabled: false,
            tracked_points: Vec::new(),
            max_tracked: 5,
            tolerance: 0.5_f64.to_radians(), // 0.5 degrees
            settings: SnapTrackingSettings::default(),
        }
    }

    /// Add tracked point
    pub fn add_point(&mut self, point: Point2) {
        if self.tracked_points.len() >= self.max_tracked {
            self.tracked_points.remove(0);
        }
        self.tracked_points.push(point);
    }

    /// Clear tracked points
    pub fn clear(&mut self) {
        self.tracked_points.clear();
    }

    /// Find alignment with tracked points
    pub fn find_alignment(&self, from: Point2, to: Point2) -> Option<AlignmentInfo> {
        if !self.enabled || self.tracked_points.is_empty() {
            return None;
        }

        let direction = Vector2::new(to.x - from.x, to.y - from.y);
        let angle = direction.angle();

        for tracked in &self.tracked_points {
            // Check horizontal alignment
            if (to.y - tracked.y).abs() < self.tolerance {
                return Some(AlignmentInfo {
                    alignment_type: AlignmentType::Horizontal,
                    point: *tracked,
                    constraint_point: Point2::new(to.x, tracked.y),
                });
            }

            // Check vertical alignment
            if (to.x - tracked.x).abs() < self.tolerance {
                return Some(AlignmentInfo {
                    alignment_type: AlignmentType::Vertical,
                    point: *tracked,
                    constraint_point: Point2::new(tracked.x, to.y),
                });
            }

            // Check angular alignment
            let tracked_direction = Vector2::new(tracked.x - from.x, tracked.y - from.y);
            let tracked_angle = tracked_direction.angle();
            let angle_diff = (angle - tracked_angle).abs();

            if angle_diff < self.tolerance || (2.0 * PI - angle_diff) < self.tolerance {
                return Some(AlignmentInfo {
                    alignment_type: AlignmentType::Angular { angle: tracked_angle },
                    point: *tracked,
                    constraint_point: to,
                });
            }
        }

        None
    }

    /// Toggle snap tracking
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }
}

impl Default for SnapTracking {
    fn default() -> Self {
        Self::new()
    }
}

/// Snap tracking display settings
#[derive(Debug, Clone)]
pub struct SnapTrackingSettings {
    /// Show tracking lines
    pub show_lines: bool,
    /// Show alignment points
    pub show_points: bool,
    /// Tracking line color
    pub line_color: [f32; 4],
    /// Point color
    pub point_color: [f32; 4],
    /// Line width
    pub line_width: f32,
}

impl Default for SnapTrackingSettings {
    fn default() -> Self {
        Self {
            show_lines: true,
            show_points: true,
            line_color: [0.0, 1.0, 0.0, 0.5], // Green transparent
            point_color: [0.0, 1.0, 0.0, 1.0], // Green solid
            line_width: 1.0,
        }
    }
}

/// Information about snap alignment
#[derive(Debug, Clone)]
pub struct AlignmentInfo {
    /// Type of alignment
    pub alignment_type: AlignmentType,
    /// Reference point
    pub point: Point2,
    /// Constrained point
    pub constraint_point: Point2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlignmentType {
    Horizontal,
    Vertical,
    Angular { angle: f64 },
}

/// Dynamic input display - show coordinates and dimensions during drawing
#[derive(Debug, Clone)]
pub struct DynamicInput {
    /// Is dynamic input enabled
    pub enabled: bool,
    /// Current display mode
    pub mode: DynamicInputMode,
    /// Input fields
    pub fields: DynamicInputFields,
    /// Settings
    pub settings: DynamicInputSettings,
}

impl DynamicInput {
    pub fn new() -> Self {
        Self {
            enabled: true,
            mode: DynamicInputMode::Cartesian,
            fields: DynamicInputFields::default(),
            settings: DynamicInputSettings::default(),
        }
    }

    /// Update input fields based on current point
    pub fn update(&mut self, from: Point2, to: Point2) {
        match self.mode {
            DynamicInputMode::Cartesian => {
                self.fields.x = Some(to.x);
                self.fields.y = Some(to.y);
                self.fields.distance = None;
                self.fields.angle = None;
            }
            DynamicInputMode::Polar => {
                let dx = to.x - from.x;
                let dy = to.y - from.y;
                let distance = (dx * dx + dy * dy).sqrt();
                let angle = dy.atan2(dx).to_degrees();

                self.fields.distance = Some(distance);
                self.fields.angle = Some(angle);
                self.fields.x = None;
                self.fields.y = None;
            }
            DynamicInputMode::Relative => {
                self.fields.x = Some(to.x - from.x);
                self.fields.y = Some(to.y - from.y);
                self.fields.distance = None;
                self.fields.angle = None;
            }
        }
    }

    /// Toggle dynamic input
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    /// Cycle input mode
    pub fn cycle_mode(&mut self) {
        self.mode = match self.mode {
            DynamicInputMode::Cartesian => DynamicInputMode::Polar,
            DynamicInputMode::Polar => DynamicInputMode::Relative,
            DynamicInputMode::Relative => DynamicInputMode::Cartesian,
        };
    }
}

impl Default for DynamicInput {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DynamicInputMode {
    /// Show absolute X, Y coordinates
    Cartesian,
    /// Show distance and angle
    Polar,
    /// Show relative X, Y from base point
    Relative,
}

/// Dynamic input field values
#[derive(Debug, Clone, Default)]
pub struct DynamicInputFields {
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub z: Option<f64>,
    pub distance: Option<f64>,
    pub angle: Option<f64>,
}

/// Dynamic input display settings
#[derive(Debug, Clone)]
pub struct DynamicInputSettings {
    /// Show input near cursor
    pub show_near_cursor: bool,
    /// Show input in status bar
    pub show_in_status: bool,
    /// Input box color
    pub box_color: [f32; 4],
    /// Text color
    pub text_color: [f32; 4],
    /// Font size
    pub font_size: f32,
}

impl Default for DynamicInputSettings {
    fn default() -> Self {
        Self {
            show_near_cursor: true,
            show_in_status: true,
            box_color: [0.2, 0.2, 0.2, 0.9],
            text_color: [1.0, 1.0, 1.0, 1.0],
            font_size: 12.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ortho_constraint() {
        let mut ortho = OrthoMode::new();
        ortho.enabled = true;

        let from = Point2::new(0.0, 0.0);
        let to = Point2::new(10.0, 3.0);

        let constrained = ortho.constrain_point(from, to);

        // Should snap to nearest ortho angle (0° in this case)
        assert!((constrained.y - 0.0).abs() < 1e-10);
        assert!(constrained.x > 0.0);
    }

    #[test]
    fn test_polar_tracking_increment() {
        let mut polar = PolarTracking::new();
        polar.enabled = true;
        polar.set_increment(45.0);

        let angle = 47.0_f64.to_radians(); // Close to 45°
        let snapped = polar.find_nearest_angle(angle);

        assert!((snapped - 45.0_f64.to_radians()).abs() < 1e-10);
    }

    #[test]
    fn test_polar_tracking_toggle() {
        let mut polar = PolarTracking::new();
        assert!(!polar.enabled);

        polar.toggle();
        assert!(polar.enabled);
    }

    #[test]
    fn test_snap_tracking_alignment() {
        let mut tracking = SnapTracking::new();
        tracking.enabled = true;
        tracking.add_point(Point2::new(10.0, 10.0));

        let from = Point2::new(0.0, 0.0);
        let to = Point2::new(5.0, 10.01); // Close to horizontal alignment

        let alignment = tracking.find_alignment(from, to);
        assert!(alignment.is_some());

        if let Some(info) = alignment {
            assert_eq!(info.alignment_type, AlignmentType::Horizontal);
        }
    }

    #[test]
    fn test_dynamic_input_modes() {
        let mut input = DynamicInput::new();

        assert_eq!(input.mode, DynamicInputMode::Cartesian);

        input.cycle_mode();
        assert_eq!(input.mode, DynamicInputMode::Polar);

        input.cycle_mode();
        assert_eq!(input.mode, DynamicInputMode::Relative);

        input.cycle_mode();
        assert_eq!(input.mode, DynamicInputMode::Cartesian);
    }

    #[test]
    fn test_dynamic_input_update_cartesian() {
        let mut input = DynamicInput::new();
        input.mode = DynamicInputMode::Cartesian;

        let from = Point2::new(0.0, 0.0);
        let to = Point2::new(10.0, 5.0);

        input.update(from, to);

        assert_eq!(input.fields.x, Some(10.0));
        assert_eq!(input.fields.y, Some(5.0));
        assert!(input.fields.distance.is_none());
    }

    #[test]
    fn test_dynamic_input_update_polar() {
        let mut input = DynamicInput::new();
        input.mode = DynamicInputMode::Polar;

        let from = Point2::new(0.0, 0.0);
        let to = Point2::new(3.0, 4.0);

        input.update(from, to);

        assert_eq!(input.fields.distance, Some(5.0));
        assert!(input.fields.angle.is_some());
        assert!(input.fields.x.is_none());
    }
}
