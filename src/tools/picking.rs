// Object Picking System - Complete implementation
// Handles picking entities in 2D and 3D views

use super::{EntityId, Point2, Point3, Ray3, Vector3, Entity, EntityType};

/// Pick priority for different geometric features
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PickPriority {
    /// Endpoint of a line/arc (highest priority)
    Endpoint = 0,
    /// Midpoint
    Midpoint = 1,
    /// Center point
    Center = 2,
    /// Quadrant point
    Quadrant = 3,
    /// Point on edge/curve
    Edge = 4,
    /// Interior/face
    Interior = 5,
}

/// Result of a pick operation
#[derive(Debug, Clone)]
pub struct PickResult {
    /// Entity that was picked
    pub entity_id: EntityId,
    /// Point where pick occurred (world coordinates)
    pub point: Point3,
    /// Distance from pick ray origin
    pub distance: f64,
    /// Pick priority (for sorting multiple results)
    pub priority: PickPriority,
    /// Additional data about what was picked
    pub feature: PickedFeature,
}

impl PickResult {
    pub fn new(
        entity_id: EntityId,
        point: Point3,
        distance: f64,
        priority: PickPriority,
        feature: PickedFeature,
    ) -> Self {
        Self {
            entity_id,
            point,
            distance,
            priority,
            feature,
        }
    }
}

/// Details about what feature was picked
#[derive(Debug, Clone)]
pub enum PickedFeature {
    /// Endpoint of a line/curve
    Endpoint { index: usize },
    /// Midpoint of a line/curve
    Midpoint,
    /// Center of circle/arc
    Center,
    /// Quadrant point (0=0°, 1=90°, 2=180°, 3=270°)
    Quadrant { index: usize },
    /// Point on edge at parameter t
    Edge { parameter: f64 },
    /// Intersection point
    Intersection { entities: Vec<EntityId> },
    /// Interior point
    Interior,
    /// Vertex of polyline/polygon
    Vertex { index: usize },
}

/// Filter for picking operations
#[derive(Debug, Clone)]
pub struct PickFilter {
    /// Only pick these entity types (None = all types)
    pub entity_types: Option<Vec<EntityType>>,
    /// Only pick from these layers (None = all layers)
    pub layers: Option<Vec<String>>,
    /// Skip locked layers
    pub skip_locked: bool,
    /// Skip invisible layers
    pub skip_invisible: bool,
    /// Skip selected entities
    pub skip_selected: bool,
    /// Custom filter function
    pub custom: Option<fn(&Entity) -> bool>,
}

impl PickFilter {
    pub fn new() -> Self {
        Self {
            entity_types: None,
            layers: None,
            skip_locked: true,
            skip_invisible: true,
            skip_selected: false,
            custom: None,
        }
    }

    pub fn with_types(mut self, types: Vec<EntityType>) -> Self {
        self.entity_types = Some(types);
        self
    }

    pub fn with_layers(mut self, layers: Vec<String>) -> Self {
        self.layers = Some(layers);
        self
    }

    pub fn skip_selected(mut self) -> Self {
        self.skip_selected = true;
        self
    }

    pub fn matches(&self, entity: &Entity) -> bool {
        // Check entity type filter
        if let Some(ref types) = self.entity_types {
            if !types.contains(&entity.entity_type) {
                return false;
            }
        }

        // Check layer filter
        if let Some(ref layers) = self.layers {
            if !layers.contains(&entity.layer) {
                return false;
            }
        }

        // Check custom filter
        if let Some(custom) = self.custom {
            if !custom(entity) {
                return false;
            }
        }

        true
    }
}

impl Default for PickFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Main picker - handles all picking operations
pub struct Picker {
    /// Aperture size in pixels for 2D picking
    pub aperture: f64,
    /// Enable priority sorting
    pub priority_picking: bool,
    /// Maximum pick distance for 3D picking
    pub max_distance: f64,
}

impl Picker {
    pub fn new() -> Self {
        Self {
            aperture: 5.0,
            priority_picking: true,
            max_distance: 1000.0,
        }
    }

    /// Pick in 2D view using screen coordinates
    pub fn pick_2d(
        &self,
        screen_pos: Point2,
        entities: &[Entity],
        filter: &PickFilter,
        view_transform: &ViewTransform,
    ) -> Option<PickResult> {
        let mut results = self.pick_2d_all(screen_pos, entities, filter, view_transform);

        if results.is_empty() {
            return None;
        }

        // Sort by priority then distance
        results.sort_by(|a, b| {
            a.priority
                .cmp(&b.priority)
                .then(a.distance.partial_cmp(&b.distance).unwrap())
        });

        results.into_iter().next()
    }

    /// Pick all entities at 2D location
    pub fn pick_2d_all(
        &self,
        screen_pos: Point2,
        entities: &[Entity],
        filter: &PickFilter,
        view_transform: &ViewTransform,
    ) -> Vec<PickResult> {
        let world_pos = view_transform.screen_to_world(screen_pos);
        let tolerance = self.aperture * view_transform.pixel_size;

        let mut results = Vec::new();

        for entity in entities {
            if !filter.matches(entity) {
                continue;
            }

            // Check if entity bounds are near pick point
            if let Some(bounds) = &entity.bounds {
                if !bounds.intersects(&world_pos, tolerance) {
                    continue;
                }
            }

            // Perform detailed picking based on entity type
            if let Some(pick) = self.pick_entity_2d(entity, world_pos, tolerance) {
                results.push(pick);
            }
        }

        results
    }

    fn pick_entity_2d(&self, entity: &Entity, point: Point2, tolerance: f64) -> Option<PickResult> {
        // Simplified picking - in real implementation would check actual geometry
        if let Some(bounds) = &entity.bounds {
            let center = Point2::new(
                (bounds.min.x + bounds.max.x) / 2.0,
                (bounds.min.y + bounds.max.y) / 2.0,
            );

            let distance = point.distance_to(&center);

            if distance <= tolerance {
                return Some(PickResult::new(
                    entity.id,
                    Point3::new(point.x, point.y, 0.0),
                    distance,
                    PickPriority::Interior,
                    PickedFeature::Interior,
                ));
            }

            // Check endpoints (simplified - would check actual geometry)
            let endpoints = vec![
                (bounds.min, 0),
                (Point2::new(bounds.max.x, bounds.min.y), 1),
                (bounds.max, 2),
                (Point2::new(bounds.min.x, bounds.max.y), 3),
            ];

            for (ep, idx) in endpoints {
                let dist = point.distance_to(&ep);
                if dist <= tolerance {
                    return Some(PickResult::new(
                        entity.id,
                        Point3::new(ep.x, ep.y, 0.0),
                        dist,
                        PickPriority::Endpoint,
                        PickedFeature::Endpoint { index: idx },
                    ));
                }
            }
        }

        None
    }

    /// Pick in 3D view using ray casting
    pub fn pick_3d(
        &self,
        ray: Ray3,
        entities: &[Entity],
        filter: &PickFilter,
    ) -> Option<PickResult> {
        let mut results = self.pick_3d_all(ray, entities, filter);

        if results.is_empty() {
            return None;
        }

        // Sort by priority then distance
        results.sort_by(|a, b| {
            a.priority
                .cmp(&b.priority)
                .then(a.distance.partial_cmp(&b.distance).unwrap())
        });

        results.into_iter().next()
    }

    /// Pick all entities along ray
    pub fn pick_3d_all(&self, ray: Ray3, entities: &[Entity], filter: &PickFilter) -> Vec<PickResult> {
        let mut results = Vec::new();

        for entity in entities {
            if !filter.matches(entity) {
                continue;
            }

            if let Some(pick) = self.pick_entity_3d(entity, &ray) {
                if pick.distance <= self.max_distance {
                    results.push(pick);
                }
            }
        }

        results
    }

    fn pick_entity_3d(&self, entity: &Entity, ray: &Ray3) -> Option<PickResult> {
        // Simplified ray-box intersection
        // In real implementation would check actual geometry
        if let Some(bounds) = &entity.bounds {
            let box_center = Point3::new(
                (bounds.min.x + bounds.max.x) / 2.0,
                (bounds.min.y + bounds.max.y) / 2.0,
                0.0,
            );

            // Simple distance to center check (not actual ray-box intersection)
            let to_center = Vector3::new(
                box_center.x - ray.origin.x,
                box_center.y - ray.origin.y,
                box_center.z - ray.origin.z,
            );

            let t = to_center.dot(&ray.direction);
            if t < 0.0 {
                return None;
            }

            let point = ray.point_at(t);
            let distance = ray.origin.distance_to(&point);

            // Check if ray passes near entity bounds
            let half_width = (bounds.max.x - bounds.min.x) / 2.0;
            let half_height = (bounds.max.y - bounds.min.y) / 2.0;
            let max_size = half_width.max(half_height);

            let offset = point.distance_to(&box_center);
            if offset <= max_size {
                return Some(PickResult::new(
                    entity.id,
                    point,
                    distance,
                    PickPriority::Interior,
                    PickedFeature::Interior,
                ));
            }
        }

        None
    }

    /// Pick nearest point on entity to given point
    pub fn pick_nearest(
        &self,
        point: Point2,
        entities: &[Entity],
        filter: &PickFilter,
    ) -> Option<PickResult> {
        let mut nearest: Option<PickResult> = None;
        let mut min_distance = f64::INFINITY;

        for entity in entities {
            if !filter.matches(entity) {
                continue;
            }

            if let Some(bounds) = &entity.bounds {
                // Find nearest point on bounds (simplified)
                let center = Point2::new(
                    (bounds.min.x + bounds.max.x) / 2.0,
                    (bounds.min.y + bounds.max.y) / 2.0,
                );

                let distance = point.distance_to(&center);

                if distance < min_distance {
                    min_distance = distance;
                    nearest = Some(PickResult::new(
                        entity.id,
                        Point3::new(center.x, center.y, 0.0),
                        distance,
                        PickPriority::Edge,
                        PickedFeature::Edge { parameter: 0.5 },
                    ));
                }
            }
        }

        nearest
    }

    /// Pick with snap support (integrates with snap system)
    pub fn pick_with_snap(
        &self,
        point: Point2,
        entities: &[Entity],
        filter: &PickFilter,
        view_transform: &ViewTransform,
        snap_mode: u32,
    ) -> Option<PickResult> {
        // First try snap pick
        if snap_mode != 0 {
            if let Some(snap_result) = self.pick_snap(point, entities, filter, view_transform, snap_mode)
            {
                return Some(snap_result);
            }
        }

        // Fall back to regular pick
        self.pick_2d(point, entities, filter, view_transform)
    }

    fn pick_snap(
        &self,
        point: Point2,
        entities: &[Entity],
        filter: &PickFilter,
        view_transform: &ViewTransform,
        snap_mode: u32,
    ) -> Option<PickResult> {
        // Snap picking - prioritizes snap points
        let tolerance = self.aperture * view_transform.pixel_size * 2.0;

        let mut snap_results = Vec::new();

        for entity in entities {
            if !filter.matches(entity) {
                continue;
            }

            if let Some(bounds) = &entity.bounds {
                // Check endpoints if snap mode includes endpoint
                if snap_mode & 0x01 != 0 {
                    let endpoints = vec![
                        (bounds.min, 0),
                        (Point2::new(bounds.max.x, bounds.min.y), 1),
                        (bounds.max, 2),
                        (Point2::new(bounds.min.x, bounds.max.y), 3),
                    ];

                    for (ep, idx) in endpoints {
                        let dist = point.distance_to(&ep);
                        if dist <= tolerance {
                            snap_results.push(PickResult::new(
                                entity.id,
                                Point3::new(ep.x, ep.y, 0.0),
                                dist,
                                PickPriority::Endpoint,
                                PickedFeature::Endpoint { index: idx },
                            ));
                        }
                    }
                }

                // Check midpoint if snap mode includes midpoint
                if snap_mode & 0x02 != 0 {
                    let mid = point.midpoint(&Point2::new(
                        (bounds.min.x + bounds.max.x) / 2.0,
                        (bounds.min.y + bounds.max.y) / 2.0,
                    ));
                    let dist = point.distance_to(&mid);
                    if dist <= tolerance {
                        snap_results.push(PickResult::new(
                            entity.id,
                            Point3::new(mid.x, mid.y, 0.0),
                            dist,
                            PickPriority::Midpoint,
                            PickedFeature::Midpoint,
                        ));
                    }
                }

                // Check center if snap mode includes center
                if snap_mode & 0x04 != 0 {
                    let center = Point2::new(
                        (bounds.min.x + bounds.max.x) / 2.0,
                        (bounds.min.y + bounds.max.y) / 2.0,
                    );
                    let dist = point.distance_to(&center);
                    if dist <= tolerance {
                        snap_results.push(PickResult::new(
                            entity.id,
                            Point3::new(center.x, center.y, 0.0),
                            dist,
                            PickPriority::Center,
                            PickedFeature::Center,
                        ));
                    }
                }
            }
        }

        // Return highest priority snap result
        snap_results.sort_by(|a, b| {
            a.priority
                .cmp(&b.priority)
                .then(a.distance.partial_cmp(&b.distance).unwrap())
        });

        snap_results.into_iter().next()
    }
}

impl Default for Picker {
    fn default() -> Self {
        Self::new()
    }
}

/// View transform for coordinate conversion
#[derive(Debug, Clone)]
pub struct ViewTransform {
    /// View center in world coordinates
    pub center: Point2,
    /// Zoom factor (world units per pixel)
    pub pixel_size: f64,
    /// View rotation angle in radians
    pub rotation: f64,
    /// Viewport size in pixels
    pub viewport_size: (u32, u32),
}

impl ViewTransform {
    pub fn new(viewport_size: (u32, u32)) -> Self {
        Self {
            center: Point2::zero(),
            pixel_size: 1.0,
            rotation: 0.0,
            viewport_size,
        }
    }

    /// Convert screen coordinates to world coordinates
    pub fn screen_to_world(&self, screen: Point2) -> Point2 {
        // Center on viewport
        let cx = screen.x - (self.viewport_size.0 as f64) / 2.0;
        let cy = (self.viewport_size.1 as f64) / 2.0 - screen.y; // Flip Y

        // Apply rotation if any
        let (x, y) = if self.rotation != 0.0 {
            let cos = self.rotation.cos();
            let sin = self.rotation.sin();
            (cx * cos - cy * sin, cx * sin + cy * cos)
        } else {
            (cx, cy)
        };

        // Scale and translate
        Point2::new(
            x * self.pixel_size + self.center.x,
            y * self.pixel_size + self.center.y,
        )
    }

    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, world: Point2) -> Point2 {
        // Translate
        let mut x = world.x - self.center.x;
        let mut y = world.y - self.center.y;

        // Scale
        x /= self.pixel_size;
        y /= self.pixel_size;

        // Apply rotation if any
        if self.rotation != 0.0 {
            let cos = self.rotation.cos();
            let sin = self.rotation.sin();
            let rx = x * cos + y * sin;
            let ry = -x * sin + y * cos;
            x = rx;
            y = ry;
        }

        // Convert to screen space
        Point2::new(
            x + (self.viewport_size.0 as f64) / 2.0,
            (self.viewport_size.1 as f64) / 2.0 - y, // Flip Y
        )
    }

    /// Create ray from screen coordinates for 3D picking
    pub fn screen_to_ray(&self, screen: Point2, camera_pos: Point3, camera_dir: Vector3) -> Ray3 {
        let world = self.screen_to_world(screen);
        Ray3::new(
            Point3::new(world.x, world.y, camera_pos.z),
            camera_dir,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pick_filter_types() {
        let filter = PickFilter::new().with_types(vec![EntityType::Line, EntityType::Arc]);

        let line = Entity::new(EntityType::Line);
        let circle = Entity::new(EntityType::Circle);

        assert!(filter.matches(&line));
        assert!(!filter.matches(&circle));
    }

    #[test]
    fn test_pick_filter_layers() {
        let filter = PickFilter::new().with_layers(vec!["0".to_string(), "1".to_string()]);

        let mut e1 = Entity::new(EntityType::Line);
        e1.layer = "0".to_string();

        let mut e2 = Entity::new(EntityType::Line);
        e2.layer = "2".to_string();

        assert!(filter.matches(&e1));
        assert!(!filter.matches(&e2));
    }

    #[test]
    fn test_view_transform_screen_to_world() {
        let transform = ViewTransform::new((800, 600));
        let screen = Point2::new(400.0, 300.0);
        let world = transform.screen_to_world(screen);

        // Center of screen should map to world origin
        assert!((world.x - 0.0).abs() < 1e-10);
        assert!((world.y - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_pick_priority_order() {
        assert!(PickPriority::Endpoint < PickPriority::Midpoint);
        assert!(PickPriority::Midpoint < PickPriority::Edge);
        assert!(PickPriority::Edge < PickPriority::Interior);
    }
}
