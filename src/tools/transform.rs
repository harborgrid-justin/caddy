// Transformation Tools - Complete implementation
// Handles Move, Rotate, Scale, Mirror operations

use super::{EntityId, Point2, Point3, Matrix4};
use std::f64::consts::PI;

/// Transformation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformMode {
    /// Move entities
    Move,
    /// Rotate entities
    Rotate,
    /// Scale entities
    Scale,
    /// Mirror entities
    Mirror,
    /// Stretch entities
    Stretch,
    /// Array (rectangular or polar)
    Array,
}

/// Transform operation state
#[derive(Debug, Clone)]
pub struct TransformOperation {
    /// Current mode
    pub mode: TransformMode,
    /// Base point (pivot/reference point)
    pub base_point: Option<Point2>,
    /// Second point (for direction, distance, etc.)
    pub second_point: Option<Point2>,
    /// Third point (for some operations)
    pub third_point: Option<Point2>,
    /// Copy mode (create copies instead of moving)
    pub copy_mode: bool,
    /// Multiple copies count
    pub copy_count: u32,
    /// Preview enabled
    pub preview_enabled: bool,
    /// Current transformation matrix
    pub transform: Matrix4,
    /// Selected entities
    pub entities: Vec<EntityId>,
}

impl TransformOperation {
    pub fn new(mode: TransformMode) -> Self {
        Self {
            mode,
            base_point: None,
            second_point: None,
            third_point: None,
            copy_mode: false,
            copy_count: 1,
            preview_enabled: true,
            transform: Matrix4::identity(),
            entities: Vec::new(),
        }
    }

    /// Set base point
    pub fn set_base_point(&mut self, point: Point2) {
        self.base_point = Some(point);
    }

    /// Set second point and update transform
    pub fn set_second_point(&mut self, point: Point2) {
        self.second_point = Some(point);
        self.update_transform();
    }

    /// Set third point (for operations like align, scale with reference)
    pub fn set_third_point(&mut self, point: Point2) {
        self.third_point = Some(point);
        self.update_transform();
    }

    /// Update current point (for preview during drag)
    pub fn update_current_point(&mut self, point: Point2) {
        if self.base_point.is_some() {
            self.second_point = Some(point);
            self.update_transform();
        }
    }

    /// Toggle copy mode
    pub fn toggle_copy_mode(&mut self) {
        self.copy_mode = !self.copy_mode;
    }

    /// Set number of copies
    pub fn set_copy_count(&mut self, count: u32) {
        self.copy_count = count.max(1);
    }

    /// Calculate transformation matrix based on current state
    fn update_transform(&mut self) {
        if self.base_point.is_none() {
            return;
        }

        let base = self.base_point.unwrap();

        self.transform = match self.mode {
            TransformMode::Move => self.calculate_move_transform(base),
            TransformMode::Rotate => self.calculate_rotate_transform(base),
            TransformMode::Scale => self.calculate_scale_transform(base),
            TransformMode::Mirror => self.calculate_mirror_transform(base),
            TransformMode::Stretch => self.calculate_stretch_transform(base),
            TransformMode::Array => Matrix4::identity(), // Handled separately
        };
    }

    fn calculate_move_transform(&self, base: Point2) -> Matrix4 {
        if let Some(second) = self.second_point {
            let dx = second.x - base.x;
            let dy = second.y - base.y;
            Matrix4::translation(dx, dy, 0.0)
        } else {
            Matrix4::identity()
        }
    }

    fn calculate_rotate_transform(&self, base: Point2) -> Matrix4 {
        if let Some(second) = self.second_point {
            let angle = self.calculate_angle(base, second);

            // Translate to origin, rotate, translate back
            let to_origin = Matrix4::translation(-base.x, -base.y, 0.0);
            let rotation = Matrix4::rotation_z(angle);
            let from_origin = Matrix4::translation(base.x, base.y, 0.0);

            self.multiply_matrices(&[from_origin, rotation, to_origin])
        } else {
            Matrix4::identity()
        }
    }

    fn calculate_scale_transform(&self, base: Point2) -> Matrix4 {
        if let Some(second) = self.second_point {
            let scale_factor = if let Some(third) = self.third_point {
                // Scale with reference length
                let ref_length = base.distance_to(&second);
                let new_length = base.distance_to(&third);
                if ref_length > 1e-10 {
                    new_length / ref_length
                } else {
                    1.0
                }
            } else {
                // Direct scale factor from distance
                let distance = base.distance_to(&second);
                (distance / 10.0).max(0.001) // Arbitrary scaling
            };

            let to_origin = Matrix4::translation(-base.x, -base.y, 0.0);
            let scale = Matrix4::scale(scale_factor, scale_factor, 1.0);
            let from_origin = Matrix4::translation(base.x, base.y, 0.0);

            self.multiply_matrices(&[from_origin, scale, to_origin])
        } else {
            Matrix4::identity()
        }
    }

    fn calculate_mirror_transform(&self, base: Point2) -> Matrix4 {
        if let Some(second) = self.second_point {
            // Mirror across line from base to second point
            let dx = second.x - base.x;
            let dy = second.y - base.y;
            let length = (dx * dx + dy * dy).sqrt();

            if length < 1e-10 {
                return Matrix4::identity();
            }

            // Normalize direction
            let nx = dx / length;
            let ny = dy / length;

            // Mirror matrix components
            let data = [
                [1.0 - 2.0 * nx * nx, -2.0 * nx * ny, 0.0, 0.0],
                [-2.0 * nx * ny, 1.0 - 2.0 * ny * ny, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ];

            let to_origin = Matrix4::translation(-base.x, -base.y, 0.0);
            let mirror = Matrix4 { data };
            let from_origin = Matrix4::translation(base.x, base.y, 0.0);

            self.multiply_matrices(&[from_origin, mirror, to_origin])
        } else {
            Matrix4::identity()
        }
    }

    fn calculate_stretch_transform(&self, base: Point2) -> Matrix4 {
        if let Some(second) = self.second_point {
            // Stretch along direction from base to second
            let dx = second.x - base.x;
            let dy = second.y - base.y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance < 1e-10 {
                return Matrix4::identity();
            }

            // For now, simple translation (real stretch would be more complex)
            Matrix4::translation(dx, dy, 0.0)
        } else {
            Matrix4::identity()
        }
    }

    fn calculate_angle(&self, p1: Point2, p2: Point2) -> f64 {
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        dy.atan2(dx)
    }

    fn multiply_matrices(&self, matrices: &[Matrix4]) -> Matrix4 {
        let mut result = Matrix4::identity();
        for matrix in matrices {
            result = self.multiply_matrix(&result, matrix);
        }
        result
    }

    fn multiply_matrix(&self, a: &Matrix4, b: &Matrix4) -> Matrix4 {
        let mut result = Matrix4::identity();
        for i in 0..4 {
            for j in 0..4 {
                result.data[i][j] = 0.0;
                for k in 0..4 {
                    result.data[i][j] += a.data[i][k] * b.data[k][j];
                }
            }
        }
        result
    }

    /// Apply transformation to a point
    pub fn transform_point(&self, point: Point2) -> Point2 {
        let p3 = Point3::new(point.x, point.y, 0.0);
        let transformed = self.transform.transform_point(&p3);
        Point2::new(transformed.x, transformed.y)
    }

    /// Check if operation is ready to execute
    pub fn is_ready(&self) -> bool {
        self.base_point.is_some() && self.second_point.is_some()
    }

    /// Reset operation
    pub fn reset(&mut self) {
        self.base_point = None;
        self.second_point = None;
        self.third_point = None;
        self.transform = Matrix4::identity();
    }
}

/// Transform gizmo for 3D manipulation
#[derive(Debug, Clone)]
pub struct TransformGizmo {
    /// Gizmo position
    pub position: Point3,
    /// Gizmo size (scale)
    pub size: f64,
    /// Active axis (None, Some(0)=X, Some(1)=Y, Some(2)=Z)
    pub active_axis: Option<usize>,
    /// Gizmo mode
    pub mode: TransformMode,
    /// Visibility
    pub visible: bool,
}

impl TransformGizmo {
    pub fn new(position: Point3) -> Self {
        Self {
            position,
            size: 1.0,
            active_axis: None,
            mode: TransformMode::Move,
            visible: true,
        }
    }

    /// Set gizmo position
    pub fn set_position(&mut self, position: Point3) {
        self.position = position;
    }

    /// Set gizmo mode
    pub fn set_mode(&mut self, mode: TransformMode) {
        self.mode = mode;
    }

    /// Get gizmo handles for rendering
    pub fn get_handles(&self) -> Vec<GizmoHandle> {
        match self.mode {
            TransformMode::Move => self.get_move_handles(),
            TransformMode::Rotate => self.get_rotate_handles(),
            TransformMode::Scale => self.get_scale_handles(),
            _ => Vec::new(),
        }
    }

    fn get_move_handles(&self) -> Vec<GizmoHandle> {
        vec![
            GizmoHandle {
                axis: 0,
                color: [1.0, 0.0, 0.0, 1.0], // Red X
                start: self.position,
                end: Point3::new(
                    self.position.x + self.size,
                    self.position.y,
                    self.position.z,
                ),
                handle_type: GizmoHandleType::Arrow,
            },
            GizmoHandle {
                axis: 1,
                color: [0.0, 1.0, 0.0, 1.0], // Green Y
                start: self.position,
                end: Point3::new(
                    self.position.x,
                    self.position.y + self.size,
                    self.position.z,
                ),
                handle_type: GizmoHandleType::Arrow,
            },
            GizmoHandle {
                axis: 2,
                color: [0.0, 0.0, 1.0, 1.0], // Blue Z
                start: self.position,
                end: Point3::new(
                    self.position.x,
                    self.position.y,
                    self.position.z + self.size,
                ),
                handle_type: GizmoHandleType::Arrow,
            },
        ]
    }

    fn get_rotate_handles(&self) -> Vec<GizmoHandle> {
        vec![
            GizmoHandle {
                axis: 0,
                color: [1.0, 0.0, 0.0, 0.7],
                start: self.position,
                end: self.position, // Circle, not line
                handle_type: GizmoHandleType::Circle,
            },
            GizmoHandle {
                axis: 1,
                color: [0.0, 1.0, 0.0, 0.7],
                start: self.position,
                end: self.position,
                handle_type: GizmoHandleType::Circle,
            },
            GizmoHandle {
                axis: 2,
                color: [0.0, 0.0, 1.0, 0.7],
                start: self.position,
                end: self.position,
                handle_type: GizmoHandleType::Circle,
            },
        ]
    }

    fn get_scale_handles(&self) -> Vec<GizmoHandle> {
        vec![
            GizmoHandle {
                axis: 0,
                color: [1.0, 0.0, 0.0, 1.0],
                start: self.position,
                end: Point3::new(
                    self.position.x + self.size,
                    self.position.y,
                    self.position.z,
                ),
                handle_type: GizmoHandleType::Box,
            },
            GizmoHandle {
                axis: 1,
                color: [0.0, 1.0, 0.0, 1.0],
                start: self.position,
                end: Point3::new(
                    self.position.x,
                    self.position.y + self.size,
                    self.position.z,
                ),
                handle_type: GizmoHandleType::Box,
            },
            GizmoHandle {
                axis: 2,
                color: [0.0, 0.0, 1.0, 1.0],
                start: self.position,
                end: Point3::new(
                    self.position.x,
                    self.position.y,
                    self.position.z + self.size,
                ),
                handle_type: GizmoHandleType::Box,
            },
        ]
    }

    /// Pick gizmo handle at screen position
    pub fn pick_handle(&mut self, screen_pos: Point2, tolerance: f64) -> Option<usize> {
        // Simplified picking - would use proper ray casting
        // Returns axis index if hit
        let pos_2d = Point2::new(self.position.x, self.position.y);
        let distance = screen_pos.distance_to(&pos_2d);

        if distance < tolerance {
            // Determine which axis based on angle
            let dx = screen_pos.x - pos_2d.x;
            let dy = screen_pos.y - pos_2d.y;
            let angle = dy.atan2(dx);

            let axis = if angle.abs() < PI / 4.0 {
                0 // X axis
            } else if angle > PI / 4.0 && angle < 3.0 * PI / 4.0 {
                1 // Y axis
            } else {
                2 // Z axis
            };

            self.active_axis = Some(axis);
            Some(axis)
        } else {
            None
        }
    }

    /// Clear active axis
    pub fn clear_active(&mut self) {
        self.active_axis = None;
    }
}

/// Gizmo handle for rendering
#[derive(Debug, Clone)]
pub struct GizmoHandle {
    /// Axis index (0=X, 1=Y, 2=Z)
    pub axis: usize,
    /// Handle color
    pub color: [f32; 4],
    /// Start point
    pub start: Point3,
    /// End point
    pub end: Point3,
    /// Handle type
    pub handle_type: GizmoHandleType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoHandleType {
    Arrow,
    Circle,
    Box,
}

/// Array operation (rectangular or polar)
#[derive(Debug, Clone)]
pub struct ArrayOperation {
    /// Array type
    pub array_type: ArrayType,
    /// Number of rows
    pub rows: u32,
    /// Number of columns
    pub columns: u32,
    /// Row spacing
    pub row_spacing: f64,
    /// Column spacing
    pub column_spacing: f64,
    /// Angle for polar array
    pub angle: f64,
    /// Number of items in polar array
    pub polar_count: u32,
    /// Base point
    pub base_point: Point2,
}

impl ArrayOperation {
    pub fn rectangular(rows: u32, columns: u32, row_spacing: f64, col_spacing: f64) -> Self {
        Self {
            array_type: ArrayType::Rectangular,
            rows,
            columns,
            row_spacing,
            column_spacing: col_spacing,
            angle: 0.0,
            polar_count: 0,
            base_point: Point2::zero(),
        }
    }

    pub fn polar(count: u32, angle: f64, base: Point2) -> Self {
        Self {
            array_type: ArrayType::Polar,
            rows: 0,
            columns: 0,
            row_spacing: 0.0,
            column_spacing: 0.0,
            angle,
            polar_count: count,
            base_point: base,
        }
    }

    /// Generate transformation matrices for array
    pub fn get_transforms(&self) -> Vec<Matrix4> {
        match self.array_type {
            ArrayType::Rectangular => self.get_rectangular_transforms(),
            ArrayType::Polar => self.get_polar_transforms(),
        }
    }

    fn get_rectangular_transforms(&self) -> Vec<Matrix4> {
        let mut transforms = Vec::new();

        for row in 0..self.rows {
            for col in 0..self.columns {
                if row == 0 && col == 0 {
                    continue; // Skip original
                }

                let dx = col as f64 * self.column_spacing;
                let dy = row as f64 * self.row_spacing;
                transforms.push(Matrix4::translation(dx, dy, 0.0));
            }
        }

        transforms
    }

    fn get_polar_transforms(&self) -> Vec<Matrix4> {
        let mut transforms = Vec::new();
        let angle_step = self.angle / (self.polar_count - 1) as f64;

        for i in 1..self.polar_count {
            let angle = i as f64 * angle_step;
            let rotation = Matrix4::rotation_z(angle);

            // Translate to origin, rotate, translate back
            let to_origin = Matrix4::translation(-self.base_point.x, -self.base_point.y, 0.0);
            let from_origin = Matrix4::translation(self.base_point.x, self.base_point.y, 0.0);

            // Multiply matrices
            let mut result = Matrix4::identity();
            for matrix in &[from_origin, rotation, to_origin] {
                result = multiply_4x4(&result, matrix);
            }

            transforms.push(result);
        }

        transforms
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrayType {
    Rectangular,
    Polar,
}

/// Helper function to multiply 4x4 matrices
fn multiply_4x4(a: &Matrix4, b: &Matrix4) -> Matrix4 {
    let mut result = Matrix4::identity();
    for i in 0..4 {
        for j in 0..4 {
            result.data[i][j] = 0.0;
            for k in 0..4 {
                result.data[i][j] += a.data[i][k] * b.data[k][j];
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_transform() {
        let mut op = TransformOperation::new(TransformMode::Move);
        op.set_base_point(Point2::new(0.0, 0.0));
        op.set_second_point(Point2::new(10.0, 5.0));

        let point = Point2::new(5.0, 5.0);
        let transformed = op.transform_point(point);

        assert!((transformed.x - 15.0).abs() < 1e-10);
        assert!((transformed.y - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_copy_mode() {
        let mut op = TransformOperation::new(TransformMode::Move);
        assert!(!op.copy_mode);

        op.toggle_copy_mode();
        assert!(op.copy_mode);
    }

    #[test]
    fn test_array_rectangular() {
        let array = ArrayOperation::rectangular(3, 4, 10.0, 15.0);
        let transforms = array.get_transforms();

        // 3x4 array minus original = 11 transforms
        assert_eq!(transforms.len(), 11);
    }

    #[test]
    fn test_array_polar() {
        let base = Point2::new(0.0, 0.0);
        let array = ArrayOperation::polar(6, 2.0 * PI, base);
        let transforms = array.get_transforms();

        // 6 items minus original = 5 transforms
        assert_eq!(transforms.len(), 5);
    }

    #[test]
    fn test_gizmo_handles() {
        let gizmo = TransformGizmo::new(Point3::zero());
        let handles = gizmo.get_handles();

        // Move mode should have 3 axis handles
        assert_eq!(handles.len(), 3);
    }

    #[test]
    fn test_operation_ready() {
        let mut op = TransformOperation::new(TransformMode::Move);
        assert!(!op.is_ready());

        op.set_base_point(Point2::new(0.0, 0.0));
        assert!(!op.is_ready());

        op.set_second_point(Point2::new(10.0, 10.0));
        assert!(op.is_ready());
    }
}
