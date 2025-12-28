// Selection System - Complete implementation
// Handles entity selection with multiple selection modes

use super::{EntityId, Point2, BoundingBox2, Entity};
use std::collections::{HashSet, HashMap};
use std::time::Instant;

/// Selection mode determines how entities are selected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SelectionMode {
    /// Single pick - click to select one entity
    SinglePick,
    /// Window select - entities completely inside rectangle
    Window,
    /// Crossing select - entities touching or inside rectangle
    Crossing,
    /// Fence select - entities crossing a polyline
    Fence,
    /// Select all visible entities
    All,
    /// Previous selection
    Previous,
    /// Add to selection
    Add,
    /// Remove from selection
    Remove,
    /// Toggle selection
    Toggle,
}

/// Selection set - a collection of selected entities
#[derive(Debug, Clone)]
pub struct SelectionSet {
    /// Set of selected entity IDs
    entities: HashSet<EntityId>,
    /// Metadata about selection (order, mode, etc.)
    metadata: HashMap<EntityId, SelectionMetadata>,
    /// Previous selection for undo
    previous: Option<Box<SelectionSet>>,
    /// Total count
    count: usize,
}

#[derive(Debug, Clone)]
struct SelectionMetadata {
    /// When the entity was selected
    timestamp: Instant,
    /// Selection order (for certain operations)
    order: usize,
    /// How it was selected
    mode: SelectionMode,
}

impl SelectionSet {
    /// Create a new empty selection set
    pub fn new() -> Self {
        Self {
            entities: HashSet::new(),
            metadata: HashMap::new(),
            previous: None,
            count: 0,
        }
    }

    /// Add an entity to the selection
    pub fn add(&mut self, entity_id: EntityId, mode: SelectionMode) -> bool {
        if self.entities.insert(entity_id) {
            self.metadata.insert(
                entity_id,
                SelectionMetadata {
                    timestamp: Instant::now(),
                    order: self.count,
                    mode,
                },
            );
            self.count += 1;
            true
        } else {
            false
        }
    }

    /// Add multiple entities
    pub fn add_many(&mut self, entity_ids: &[EntityId], mode: SelectionMode) {
        for &id in entity_ids {
            self.add(id, mode);
        }
    }

    /// Remove an entity from the selection
    pub fn remove(&mut self, entity_id: &EntityId) -> bool {
        if self.entities.remove(entity_id) {
            self.metadata.remove(entity_id);
            true
        } else {
            false
        }
    }

    /// Remove multiple entities
    pub fn remove_many(&mut self, entity_ids: &[EntityId]) {
        for id in entity_ids {
            self.remove(id);
        }
    }

    /// Toggle entity selection
    pub fn toggle(&mut self, entity_id: EntityId, mode: SelectionMode) -> bool {
        if self.contains(&entity_id) {
            self.remove(&entity_id);
            false
        } else {
            self.add(entity_id, mode);
            true
        }
    }

    /// Clear all selections
    pub fn clear(&mut self) {
        self.entities.clear();
        self.metadata.clear();
    }

    /// Save current selection as previous
    pub fn save_previous(&mut self) {
        let prev = SelectionSet {
            entities: self.entities.clone(),
            metadata: self.metadata.clone(),
            previous: None,
            count: self.count,
        };
        self.previous = Some(Box::new(prev));
    }

    /// Restore previous selection
    pub fn restore_previous(&mut self) -> bool {
        if let Some(prev) = self.previous.take() {
            self.entities = prev.entities;
            self.metadata = prev.metadata;
            self.count = prev.count;
            true
        } else {
            false
        }
    }

    /// Check if entity is selected
    pub fn contains(&self, entity_id: &EntityId) -> bool {
        self.entities.contains(entity_id)
    }

    /// Get all selected entity IDs
    pub fn entities(&self) -> Vec<EntityId> {
        self.entities.iter().copied().collect()
    }

    /// Get selected entities in selection order
    pub fn entities_ordered(&self) -> Vec<EntityId> {
        let mut ordered: Vec<_> = self
            .metadata
            .iter()
            .map(|(id, meta)| (*id, meta.order))
            .collect();
        ordered.sort_by_key(|(_, order)| *order);
        ordered.into_iter().map(|(id, _)| id).collect()
    }

    /// Get number of selected entities
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Check if selection is empty
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Get first selected entity (by order)
    pub fn first(&self) -> Option<EntityId> {
        self.metadata
            .iter()
            .min_by_key(|(_, meta)| meta.order)
            .map(|(id, _)| *id)
    }

    /// Get last selected entity (by order)
    pub fn last(&self) -> Option<EntityId> {
        self.metadata
            .iter()
            .max_by_key(|(_, meta)| meta.order)
            .map(|(id, _)| *id)
    }

    /// Filter selection by predicate
    pub fn filter<F>(&mut self, predicate: F)
    where
        F: Fn(&EntityId) -> bool,
    {
        self.entities.retain(|id| predicate(id));
        self.metadata.retain(|id, _| self.entities.contains(id));
    }

    /// Get selection statistics
    pub fn stats(&self) -> SelectionStats {
        SelectionStats {
            total: self.len(),
            by_mode: self.count_by_mode(),
        }
    }

    fn count_by_mode(&self) -> HashMap<SelectionMode, usize> {
        let mut counts = HashMap::new();
        for meta in self.metadata.values() {
            *counts.entry(meta.mode).or_insert(0) += 1;
        }
        counts
    }
}

impl Default for SelectionSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Selection statistics
#[derive(Debug, Clone)]
pub struct SelectionStats {
    pub total: usize,
    pub by_mode: HashMap<SelectionMode, usize>,
}

/// Selection manager - handles interactive selection
#[derive(Debug)]
pub struct Selection {
    /// Current selection set
    pub set: SelectionSet,
    /// Current selection mode
    pub mode: SelectionMode,
    /// Preview state
    pub preview: SelectionPreview,
    /// Selection settings
    pub settings: SelectionSettings,
}

impl Selection {
    /// Create a new selection manager
    pub fn new() -> Self {
        Self {
            set: SelectionSet::new(),
            mode: SelectionMode::SinglePick,
            preview: SelectionPreview::new(),
            settings: SelectionSettings::default(),
        }
    }

    /// Begin window/crossing selection
    pub fn begin_window(&mut self, start: Point2, is_crossing: bool) {
        self.mode = if is_crossing {
            SelectionMode::Crossing
        } else {
            SelectionMode::Window
        };
        self.preview.begin_window(start);
    }

    /// Update window selection
    pub fn update_window(&mut self, current: Point2) {
        self.preview.update_window(current);
    }

    /// Complete window selection and select entities
    pub fn complete_window(&mut self, entities: &[Entity]) -> Vec<EntityId> {
        let selected = match self.mode {
            SelectionMode::Window => self.select_window(entities, false),
            SelectionMode::Crossing => self.select_window(entities, true),
            _ => Vec::new(),
        };

        self.preview.end_window();
        selected
    }

    /// Select entities within window
    fn select_window(&self, entities: &[Entity], crossing: bool) -> Vec<EntityId> {
        if let Some(window) = self.preview.window_bounds() {
            entities
                .iter()
                .filter(|e| {
                    if let Some(bounds) = &e.bounds {
                        if crossing {
                            // Crossing: any part touches window
                            self.bounds_intersect_window(bounds, &window)
                        } else {
                            // Window: completely inside
                            self.bounds_inside_window(bounds, &window)
                        }
                    } else {
                        false
                    }
                })
                .map(|e| e.id)
                .collect()
        } else {
            Vec::new()
        }
    }

    fn bounds_inside_window(&self, bounds: &BoundingBox2, window: &BoundingBox2) -> bool {
        bounds.min.x >= window.min.x
            && bounds.max.x <= window.max.x
            && bounds.min.y >= window.min.y
            && bounds.max.y <= window.max.y
    }

    fn bounds_intersect_window(&self, bounds: &BoundingBox2, window: &BoundingBox2) -> bool {
        !(bounds.max.x < window.min.x
            || bounds.min.x > window.max.x
            || bounds.max.y < window.min.y
            || bounds.min.y > window.max.y)
    }

    /// Begin fence selection
    pub fn begin_fence(&mut self, start: Point2) {
        self.mode = SelectionMode::Fence;
        self.preview.begin_fence(start);
    }

    /// Add point to fence
    pub fn add_fence_point(&mut self, point: Point2) {
        self.preview.add_fence_point(point);
    }

    /// Complete fence selection
    pub fn complete_fence(&mut self, entities: &[Entity]) -> Vec<EntityId> {
        let selected = self.select_fence(entities);
        self.preview.end_fence();
        selected
    }

    fn select_fence(&self, entities: &[Entity]) -> Vec<EntityId> {
        let fence_points = self.preview.fence_points();
        if fence_points.len() < 2 {
            return Vec::new();
        }

        entities
            .iter()
            .filter(|e| {
                if let Some(bounds) = &e.bounds {
                    self.bounds_cross_fence(bounds, &fence_points)
                } else {
                    false
                }
            })
            .map(|e| e.id)
            .collect()
    }

    fn bounds_cross_fence(&self, bounds: &BoundingBox2, fence: &[Point2]) -> bool {
        // Simple check: if any fence segment intersects bounding box
        for i in 0..fence.len() - 1 {
            if self.segment_intersects_box(&fence[i], &fence[i + 1], bounds) {
                return true;
            }
        }
        false
    }

    fn segment_intersects_box(&self, p1: &Point2, p2: &Point2, bounds: &BoundingBox2) -> bool {
        // Cohen-Sutherland algorithm for line-box intersection
        let outcode = |p: &Point2| {
            let mut code = 0;
            if p.x < bounds.min.x {
                code |= 1;
            } // left
            if p.x > bounds.max.x {
                code |= 2;
            } // right
            if p.y < bounds.min.y {
                code |= 4;
            } // bottom
            if p.y > bounds.max.y {
                code |= 8;
            } // top
            code
        };

        let code1 = outcode(p1);
        let code2 = outcode(p2);

        // If both endpoints inside, intersects
        if code1 == 0 || code2 == 0 {
            return true;
        }

        // If both endpoints on same side outside, doesn't intersect
        if (code1 & code2) != 0 {
            return false;
        }

        // Otherwise, might intersect (simplified check)
        true
    }

    /// Select all entities
    pub fn select_all(&mut self, entities: &[Entity]) {
        self.set.save_previous();
        self.set.clear();
        for entity in entities {
            self.set.add(entity.id, SelectionMode::All);
        }
    }

    /// Clear selection
    pub fn clear(&mut self) {
        self.set.save_previous();
        self.set.clear();
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::new()
    }
}

/// Selection preview - visual feedback during selection
#[derive(Debug, Clone)]
pub struct SelectionPreview {
    /// Hover entity (for highlight)
    pub hover_entity: Option<EntityId>,
    /// Window selection start point
    window_start: Option<Point2>,
    /// Window selection current point
    window_current: Option<Point2>,
    /// Fence selection points
    fence_points: Vec<Point2>,
    /// Preview active
    active: bool,
}

impl SelectionPreview {
    pub fn new() -> Self {
        Self {
            hover_entity: None,
            window_start: None,
            window_current: None,
            fence_points: Vec::new(),
            active: false,
        }
    }

    /// Set hover entity for highlight
    pub fn set_hover(&mut self, entity_id: Option<EntityId>) {
        self.hover_entity = entity_id;
    }

    /// Begin window selection
    pub fn begin_window(&mut self, start: Point2) {
        self.window_start = Some(start);
        self.window_current = Some(start);
        self.active = true;
    }

    /// Update window selection
    pub fn update_window(&mut self, current: Point2) {
        self.window_current = Some(current);
    }

    /// End window selection
    pub fn end_window(&mut self) {
        self.window_start = None;
        self.window_current = None;
        self.active = false;
    }

    /// Get window bounds
    pub fn window_bounds(&self) -> Option<BoundingBox2> {
        if let (Some(start), Some(current)) = (self.window_start, self.window_current) {
            Some(BoundingBox2::from_points(start, current))
        } else {
            None
        }
    }

    /// Check if window is crossing (right to left)
    pub fn is_crossing_window(&self) -> bool {
        if let (Some(start), Some(current)) = (self.window_start, self.window_current) {
            current.x < start.x
        } else {
            false
        }
    }

    /// Begin fence selection
    pub fn begin_fence(&mut self, start: Point2) {
        self.fence_points.clear();
        self.fence_points.push(start);
        self.active = true;
    }

    /// Add fence point
    pub fn add_fence_point(&mut self, point: Point2) {
        self.fence_points.push(point);
    }

    /// End fence selection
    pub fn end_fence(&mut self) {
        self.fence_points.clear();
        self.active = false;
    }

    /// Get fence points
    pub fn fence_points(&self) -> &[Point2] {
        &self.fence_points
    }

    /// Check if preview is active
    pub fn is_active(&self) -> bool {
        self.active
    }
}

impl Default for SelectionPreview {
    fn default() -> Self {
        Self::new()
    }
}

/// Selection settings
#[derive(Debug, Clone)]
pub struct SelectionSettings {
    /// Pick box size in pixels
    pub pick_box_size: f64,
    /// Enable selection preview
    pub preview_enabled: bool,
    /// Highlight color for hover
    pub highlight_color: [f32; 4],
    /// Selection color
    pub selection_color: [f32; 4],
    /// Window selection color (inside)
    pub window_color: [f32; 4],
    /// Crossing selection color
    pub crossing_color: [f32; 4],
    /// Allow selection of locked layers
    pub select_locked: bool,
    /// Noun-verb selection (select then command)
    pub noun_verb: bool,
    /// Implied windowing (auto window on empty pick)
    pub implied_windowing: bool,
}

impl Default for SelectionSettings {
    fn default() -> Self {
        Self {
            pick_box_size: 5.0,
            preview_enabled: true,
            highlight_color: [1.0, 1.0, 0.0, 0.5], // Yellow
            selection_color: [0.0, 0.5, 1.0, 0.8], // Blue
            window_color: [0.0, 0.0, 1.0, 0.3],    // Blue transparent
            crossing_color: [0.0, 1.0, 0.0, 0.3],  // Green transparent
            select_locked: false,
            noun_verb: true,
            implied_windowing: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_set_add_remove() {
        let mut set = SelectionSet::new();
        let id1 = EntityId::new_v4();
        let id2 = EntityId::new_v4();

        assert!(set.add(id1, SelectionMode::SinglePick));
        assert_eq!(set.len(), 1);
        assert!(set.contains(&id1));

        assert!(set.add(id2, SelectionMode::SinglePick));
        assert_eq!(set.len(), 2);

        assert!(set.remove(&id1));
        assert_eq!(set.len(), 1);
        assert!(!set.contains(&id1));
    }

    #[test]
    fn test_selection_set_toggle() {
        let mut set = SelectionSet::new();
        let id = EntityId::new_v4();

        assert!(set.toggle(id, SelectionMode::Toggle));
        assert!(set.contains(&id));

        assert!(!set.toggle(id, SelectionMode::Toggle));
        assert!(!set.contains(&id));
    }

    #[test]
    fn test_selection_set_clear() {
        let mut set = SelectionSet::new();
        set.add(EntityId::new_v4(), SelectionMode::SinglePick);
        set.add(EntityId::new_v4(), SelectionMode::SinglePick);

        assert_eq!(set.len(), 2);
        set.clear();
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn test_selection_set_order() {
        let mut set = SelectionSet::new();
        let id1 = EntityId::new_v4();
        let id2 = EntityId::new_v4();
        let id3 = EntityId::new_v4();

        set.add(id1, SelectionMode::SinglePick);
        set.add(id2, SelectionMode::SinglePick);
        set.add(id3, SelectionMode::SinglePick);

        let ordered = set.entities_ordered();
        assert_eq!(ordered[0], id1);
        assert_eq!(ordered[1], id2);
        assert_eq!(ordered[2], id3);
    }

    #[test]
    fn test_window_bounds() {
        let mut preview = SelectionPreview::new();
        preview.begin_window(Point2::new(0.0, 0.0));
        preview.update_window(Point2::new(10.0, 10.0));

        let bounds = preview.window_bounds().unwrap();
        assert_eq!(bounds.min, Point2::new(0.0, 0.0));
        assert_eq!(bounds.max, Point2::new(10.0, 10.0));
    }

    #[test]
    fn test_crossing_window_detection() {
        let mut preview = SelectionPreview::new();
        preview.begin_window(Point2::new(10.0, 0.0));
        preview.update_window(Point2::new(0.0, 10.0));

        assert!(preview.is_crossing_window());
    }
}
