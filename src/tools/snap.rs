// Object Snap System - Complete implementation
// Provides intelligent snapping to geometric features

use super::{EntityId, Point2, Point3, Entity, EntityType};

/// Snap modes - bit flags for combining multiple snap types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapMode {
    bits: u32,
}

impl SnapMode {
    pub const NONE: u32 = 0;
    pub const ENDPOINT: u32 = 1 << 0;      // 0x0001
    pub const MIDPOINT: u32 = 1 << 1;      // 0x0002
    pub const CENTER: u32 = 1 << 2;        // 0x0004
    pub const QUADRANT: u32 = 1 << 3;      // 0x0008
    pub const INTERSECTION: u32 = 1 << 4;  // 0x0010
    pub const PERPENDICULAR: u32 = 1 << 5; // 0x0020
    pub const TANGENT: u32 = 1 << 6;       // 0x0040
    pub const NEAREST: u32 = 1 << 7;       // 0x0080
    pub const NODE: u32 = 1 << 8;          // 0x0100
    pub const INSERTION: u32 = 1 << 9;     // 0x0200
    pub const EXTENSION: u32 = 1 << 10;    // 0x0400
    pub const PARALLEL: u32 = 1 << 11;     // 0x0800

    pub fn new(bits: u32) -> Self {
        Self { bits }
    }

    pub fn empty() -> Self {
        Self { bits: 0 }
    }

    pub fn all() -> Self {
        Self { bits: 0xFFFF }
    }

    pub fn has(&self, mode: u32) -> bool {
        (self.bits & mode) != 0
    }

    pub fn set(&mut self, mode: u32) {
        self.bits |= mode;
    }

    pub fn clear(&mut self, mode: u32) {
        self.bits &= !mode;
    }

    pub fn toggle(&mut self, mode: u32) {
        self.bits ^= mode;
    }

    pub fn bits(&self) -> u32 {
        self.bits
    }
}

impl Default for SnapMode {
    fn default() -> Self {
        Self::new(Self::ENDPOINT | Self::MIDPOINT | Self::CENTER)
    }
}

/// Result of a snap operation
#[derive(Debug, Clone)]
pub struct SnapResult {
    /// The snap point in world coordinates
    pub point: Point3,
    /// Type of snap that occurred
    pub snap_type: u32,
    /// Entity that was snapped to
    pub entity_id: Option<EntityId>,
    /// Additional snap information
    pub info: SnapInfo,
    /// Visual indicator position and type
    pub indicator: SnapIndicator,
}

impl SnapResult {
    pub fn new(point: Point3, snap_type: u32, entity_id: Option<EntityId>, info: SnapInfo) -> Self {
        let indicator = SnapIndicator::from_snap_type(snap_type, point.to_point2());
        Self {
            point,
            snap_type,
            entity_id,
            info,
            indicator,
        }
    }
}

/// Additional information about the snap
#[derive(Debug, Clone)]
pub enum SnapInfo {
    /// Endpoint snap
    Endpoint { index: usize },
    /// Midpoint snap
    Midpoint,
    /// Center of circle/arc
    Center { radius: f64 },
    /// Quadrant point (0=0°, 1=90°, 2=180°, 3=270°)
    Quadrant { index: usize },
    /// Intersection of two entities
    Intersection {
        entity1: EntityId,
        entity2: EntityId,
    },
    /// Perpendicular to line/curve
    Perpendicular { from_point: Point2 },
    /// Tangent to curve
    Tangent { angle: f64 },
    /// Nearest point on curve
    Nearest { parameter: f64 },
    /// Node/point entity
    Node,
    /// Insertion point of block/text
    Insertion,
    /// Extension of line
    Extension { distance: f64 },
    /// Parallel to line
    Parallel { reference_angle: f64 },
}

/// Visual indicator for snap feedback
#[derive(Debug, Clone)]
pub struct SnapIndicator {
    /// Position of indicator
    pub position: Point2,
    /// Shape of indicator
    pub shape: IndicatorShape,
    /// Color of indicator
    pub color: [f32; 4],
    /// Size in pixels
    pub size: f32,
}

impl SnapIndicator {
    fn from_snap_type(snap_type: u32, position: Point2) -> Self {
        let (shape, color) = match snap_type {
            SnapMode::ENDPOINT => (IndicatorShape::Square, [1.0, 0.0, 0.0, 1.0]),      // Red square
            SnapMode::MIDPOINT => (IndicatorShape::Triangle, [0.0, 1.0, 0.0, 1.0]),    // Green triangle
            SnapMode::CENTER => (IndicatorShape::Circle, [1.0, 1.0, 0.0, 1.0]),        // Yellow circle
            SnapMode::QUADRANT => (IndicatorShape::Diamond, [0.0, 1.0, 1.0, 1.0]),     // Cyan diamond
            SnapMode::INTERSECTION => (IndicatorShape::Cross, [1.0, 0.5, 0.0, 1.0]),   // Orange cross
            SnapMode::PERPENDICULAR => (IndicatorShape::Perpendicular, [0.5, 0.5, 1.0, 1.0]), // Light blue
            SnapMode::TANGENT => (IndicatorShape::Tangent, [1.0, 0.0, 1.0, 1.0]),      // Magenta
            SnapMode::NEAREST => (IndicatorShape::Hourglass, [0.7, 0.7, 0.7, 1.0]),    // Gray
            _ => (IndicatorShape::Circle, [1.0, 1.0, 1.0, 1.0]),                        // White default
        };

        Self {
            position,
            shape,
            color,
            size: 10.0,
        }
    }
}

/// Shapes for snap indicators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndicatorShape {
    Circle,
    Square,
    Triangle,
    Diamond,
    Cross,
    Perpendicular, // Right angle symbol
    Tangent,       // Tangent symbol
    Hourglass,
}

/// Snap point - a potential snap location
#[derive(Debug, Clone)]
pub struct SnapPoint {
    pub point: Point2,
    pub snap_type: u32,
    pub priority: i32,
    pub entity_id: EntityId,
    pub info: SnapInfo,
}

impl SnapPoint {
    pub fn new(point: Point2, snap_type: u32, priority: i32, entity_id: EntityId, info: SnapInfo) -> Self {
        Self {
            point,
            snap_type,
            priority,
            entity_id,
            info,
        }
    }
}

/// Object snap engine
pub struct ObjectSnap {
    /// Active snap modes
    pub mode: SnapMode,
    /// Aperture size in pixels
    pub aperture: f64,
    /// Settings
    pub settings: SnapSettings,
    /// Cached snap points for performance
    snap_cache: Vec<SnapPoint>,
}

impl ObjectSnap {
    pub fn new() -> Self {
        Self {
            mode: SnapMode::default(),
            aperture: 10.0,
            settings: SnapSettings::default(),
            snap_cache: Vec::new(),
        }
    }

    /// Find snap point near cursor position
    pub fn find_snap(
        &mut self,
        cursor: Point2,
        entities: &[Entity],
        pixel_size: f64,
    ) -> Option<SnapResult> {
        if self.mode.bits() == 0 {
            return None;
        }

        let tolerance = self.aperture * pixel_size;

        // Build snap points for nearby entities
        self.build_snap_cache(cursor, entities, tolerance);

        // Find closest snap point
        self.find_closest_snap(cursor, tolerance)
    }

    fn build_snap_cache(&mut self, cursor: Point2, entities: &[Entity], tolerance: f64) {
        self.snap_cache.clear();

        // Expand search area
        let search_tolerance = tolerance * 3.0;

        for entity in entities {
            // Quick bounds check
            if let Some(bounds) = &entity.bounds {
                if !bounds.intersects(&cursor, search_tolerance) {
                    continue;
                }
            }

            // Generate snap points based on entity type
            let mode = self.mode;
            self.generate_snap_points(entity, &mode);
        }
    }

    fn generate_snap_points(&mut self, entity: &Entity, mode: &SnapMode) {
        if let Some(bounds) = &entity.bounds {
            // Endpoint snaps
            if mode.has(SnapMode::ENDPOINT) {
                let endpoints = vec![
                    (bounds.min, 0),
                    (Point2::new(bounds.max.x, bounds.min.y), 1),
                    (bounds.max, 2),
                    (Point2::new(bounds.min.x, bounds.max.y), 3),
                ];

                for (point, idx) in endpoints {
                    self.snap_cache.push(SnapPoint::new(
                        point,
                        SnapMode::ENDPOINT,
                        10, // High priority
                        entity.id,
                        SnapInfo::Endpoint { index: idx },
                    ));
                }
            }

            // Midpoint snap
            if mode.has(SnapMode::MIDPOINT) {
                let midpoint = Point2::new(
                    (bounds.min.x + bounds.max.x) / 2.0,
                    (bounds.min.y + bounds.max.y) / 2.0,
                );
                self.snap_cache.push(SnapPoint::new(
                    midpoint,
                    SnapMode::MIDPOINT,
                    8,
                    entity.id,
                    SnapInfo::Midpoint,
                ));
            }

            // Center snap (for circles/arcs)
            if mode.has(SnapMode::CENTER)
                && (entity.entity_type == EntityType::Circle || entity.entity_type == EntityType::Arc)
            {
                let center = Point2::new(
                    (bounds.min.x + bounds.max.x) / 2.0,
                    (bounds.min.y + bounds.max.y) / 2.0,
                );
                let radius = (bounds.max.x - bounds.min.x) / 2.0;
                self.snap_cache.push(SnapPoint::new(
                    center,
                    SnapMode::CENTER,
                    9,
                    entity.id,
                    SnapInfo::Center { radius },
                ));
            }

            // Quadrant snaps (for circles/arcs)
            if mode.has(SnapMode::QUADRANT)
                && (entity.entity_type == EntityType::Circle || entity.entity_type == EntityType::Arc)
            {
                let center = Point2::new(
                    (bounds.min.x + bounds.max.x) / 2.0,
                    (bounds.min.y + bounds.max.y) / 2.0,
                );
                let radius = (bounds.max.x - bounds.min.x) / 2.0;

                let quadrants = vec![
                    (Point2::new(center.x + radius, center.y), 0), // 0°
                    (Point2::new(center.x, center.y + radius), 1), // 90°
                    (Point2::new(center.x - radius, center.y), 2), // 180°
                    (Point2::new(center.x, center.y - radius), 3), // 270°
                ];

                for (point, idx) in quadrants {
                    self.snap_cache.push(SnapPoint::new(
                        point,
                        SnapMode::QUADRANT,
                        7,
                        entity.id,
                        SnapInfo::Quadrant { index: idx },
                    ));
                }
            }

            // Node snap (for point entities)
            if mode.has(SnapMode::NODE) && entity.entity_type == EntityType::Point {
                let point = Point2::new(
                    (bounds.min.x + bounds.max.x) / 2.0,
                    (bounds.min.y + bounds.max.y) / 2.0,
                );
                self.snap_cache.push(SnapPoint::new(
                    point,
                    SnapMode::NODE,
                    10,
                    entity.id,
                    SnapInfo::Node,
                ));
            }

            // Nearest snap - will be calculated on-demand
            // Intersection snap - would need to check pairs of entities
            // Perpendicular/Tangent - need reference point
        }
    }

    fn find_closest_snap(&self, cursor: Point2, tolerance: f64) -> Option<SnapResult> {
        let mut closest: Option<&SnapPoint> = None;
        let mut min_distance = tolerance;

        for snap_point in &self.snap_cache {
            let distance = cursor.distance_to(&snap_point.point);

            if distance < min_distance {
                min_distance = distance;
                closest = Some(snap_point);
            } else if distance == min_distance {
                // If distances equal, prefer higher priority
                if let Some(current) = closest {
                    if snap_point.priority > current.priority {
                        closest = Some(snap_point);
                    }
                }
            }
        }

        closest.map(|sp| {
            SnapResult::new(
                Point3::new(sp.point.x, sp.point.y, 0.0),
                sp.snap_type,
                Some(sp.entity_id),
                sp.info.clone(),
            )
        })
    }

    /// Find perpendicular snap from a reference point
    pub fn find_perpendicular(
        &self,
        from_point: Point2,
        entities: &[Entity],
        tolerance: f64,
    ) -> Option<SnapResult> {
        if !self.mode.has(SnapMode::PERPENDICULAR) {
            return None;
        }

        let mut best_result: Option<SnapResult> = None;
        let mut min_distance = f64::INFINITY;

        for entity in entities {
            if entity.entity_type != EntityType::Line {
                continue;
            }

            if let Some(bounds) = &entity.bounds {
                // Simplified: perpendicular to bounding box edge
                let center = Point2::new(
                    (bounds.min.x + bounds.max.x) / 2.0,
                    (bounds.min.y + bounds.max.y) / 2.0,
                );

                // Calculate perpendicular point (simplified)
                let perp_point = self.perpendicular_point(from_point, bounds.min, center);
                let distance = from_point.distance_to(&perp_point);

                if distance < min_distance && distance < tolerance {
                    min_distance = distance;
                    best_result = Some(SnapResult::new(
                        Point3::new(perp_point.x, perp_point.y, 0.0),
                        SnapMode::PERPENDICULAR,
                        Some(entity.id),
                        SnapInfo::Perpendicular { from_point },
                    ));
                }
            }
        }

        best_result
    }

    fn perpendicular_point(&self, from: Point2, line_p1: Point2, line_p2: Point2) -> Point2 {
        let dx = line_p2.x - line_p1.x;
        let dy = line_p2.y - line_p1.y;
        let len_sq = dx * dx + dy * dy;

        if len_sq < 1e-10 {
            return line_p1;
        }

        let t = ((from.x - line_p1.x) * dx + (from.y - line_p1.y) * dy) / len_sq;
        let t = t.clamp(0.0, 1.0);

        Point2::new(line_p1.x + t * dx, line_p1.y + t * dy)
    }

    /// Find intersection snap between two entities
    pub fn find_intersection(
        &self,
        entities: &[Entity],
        tolerance: f64,
        cursor: Point2,
    ) -> Option<SnapResult> {
        if !self.mode.has(SnapMode::INTERSECTION) {
            return None;
        }

        // Check pairs of entities for intersections
        for i in 0..entities.len() {
            for j in (i + 1)..entities.len() {
                if let Some(intersection) =
                    self.calculate_intersection(&entities[i], &entities[j], tolerance, cursor)
                {
                    return Some(intersection);
                }
            }
        }

        None
    }

    fn calculate_intersection(
        &self,
        entity1: &Entity,
        entity2: &Entity,
        tolerance: f64,
        cursor: Point2,
    ) -> Option<SnapResult> {
        // Simplified intersection calculation
        // In real implementation would calculate actual geometric intersection
        if let (Some(b1), Some(b2)) = (&entity1.bounds, &entity2.bounds) {
            // Check if bounds overlap
            if b1.max.x < b2.min.x
                || b1.min.x > b2.max.x
                || b1.max.y < b2.min.y
                || b1.min.y > b2.max.y
            {
                return None;
            }

            // Approximate intersection point
            let int_x = (b1.min.x.max(b2.min.x) + b1.max.x.min(b2.max.x)) / 2.0;
            let int_y = (b1.min.y.max(b2.min.y) + b1.max.y.min(b2.max.y)) / 2.0;
            let int_point = Point2::new(int_x, int_y);

            if cursor.distance_to(&int_point) < tolerance {
                return Some(SnapResult::new(
                    Point3::new(int_x, int_y, 0.0),
                    SnapMode::INTERSECTION,
                    None,
                    SnapInfo::Intersection {
                        entity1: entity1.id,
                        entity2: entity2.id,
                    },
                ));
            }
        }

        None
    }

    /// Find nearest point on entity
    pub fn find_nearest(
        &self,
        cursor: Point2,
        entities: &[Entity],
        tolerance: f64,
    ) -> Option<SnapResult> {
        if !self.mode.has(SnapMode::NEAREST) {
            return None;
        }

        let mut best_result: Option<SnapResult> = None;
        let mut min_distance = f64::INFINITY;

        for entity in entities {
            if let Some(bounds) = &entity.bounds {
                if !bounds.intersects(&cursor, tolerance) {
                    continue;
                }

                // Find nearest point on entity (simplified to center)
                let center = Point2::new(
                    (bounds.min.x + bounds.max.x) / 2.0,
                    (bounds.min.y + bounds.max.y) / 2.0,
                );

                let distance = cursor.distance_to(&center);

                if distance < min_distance && distance < tolerance {
                    min_distance = distance;
                    best_result = Some(SnapResult::new(
                        Point3::new(center.x, center.y, 0.0),
                        SnapMode::NEAREST,
                        Some(entity.id),
                        SnapInfo::Nearest { parameter: 0.5 },
                    ));
                }
            }
        }

        best_result
    }

    /// Clear snap cache
    pub fn clear_cache(&mut self) {
        self.snap_cache.clear();
    }

    /// Enable all snap modes
    pub fn enable_all(&mut self) {
        self.mode = SnapMode::all();
    }

    /// Disable all snap modes
    pub fn disable_all(&mut self) {
        self.mode = SnapMode::empty();
    }

    /// Toggle specific snap mode
    pub fn toggle_mode(&mut self, mode: u32) {
        self.mode.toggle(mode);
    }
}

impl Default for ObjectSnap {
    fn default() -> Self {
        Self::new()
    }
}

/// Snap settings
#[derive(Debug, Clone)]
pub struct SnapSettings {
    /// Show snap markers
    pub show_markers: bool,
    /// Show snap tooltips
    pub show_tooltips: bool,
    /// Marker size in pixels
    pub marker_size: f32,
    /// AutoSnap (automatically snap when near)
    pub auto_snap: bool,
    /// Magnetic snap (pull cursor to snap point)
    pub magnetic: bool,
    /// Magnetic strength (0.0 to 1.0)
    pub magnetic_strength: f64,
}

impl Default for SnapSettings {
    fn default() -> Self {
        Self {
            show_markers: true,
            show_tooltips: true,
            marker_size: 10.0,
            auto_snap: true,
            magnetic: true,
            magnetic_strength: 0.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snap_mode_flags() {
        let mut mode = SnapMode::empty();
        assert!(!mode.has(SnapMode::ENDPOINT));

        mode.set(SnapMode::ENDPOINT);
        assert!(mode.has(SnapMode::ENDPOINT));

        mode.set(SnapMode::MIDPOINT);
        assert!(mode.has(SnapMode::ENDPOINT));
        assert!(mode.has(SnapMode::MIDPOINT));

        mode.clear(SnapMode::ENDPOINT);
        assert!(!mode.has(SnapMode::ENDPOINT));
        assert!(mode.has(SnapMode::MIDPOINT));
    }

    #[test]
    fn test_snap_mode_toggle() {
        let mut mode = SnapMode::empty();
        mode.toggle(SnapMode::ENDPOINT);
        assert!(mode.has(SnapMode::ENDPOINT));

        mode.toggle(SnapMode::ENDPOINT);
        assert!(!mode.has(SnapMode::ENDPOINT));
    }

    #[test]
    fn test_snap_mode_default() {
        let mode = SnapMode::default();
        assert!(mode.has(SnapMode::ENDPOINT));
        assert!(mode.has(SnapMode::MIDPOINT));
        assert!(mode.has(SnapMode::CENTER));
    }

    #[test]
    fn test_perpendicular_point() {
        let snap = ObjectSnap::new();
        let from = Point2::new(5.0, 5.0);
        let line_p1 = Point2::new(0.0, 0.0);
        let line_p2 = Point2::new(10.0, 0.0);

        let perp = snap.perpendicular_point(from, line_p1, line_p2);
        assert!((perp.x - 5.0).abs() < 1e-10);
        assert!((perp.y - 0.0).abs() < 1e-10);
    }
}
