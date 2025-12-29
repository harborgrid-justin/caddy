//! # Level-of-Detail (LOD) Management System
//!
//! Automatic level-of-detail selection and management for optimal rendering performance.
//!
//! ## Features
//!
//! - Distance-based LOD selection
//! - Screen-space error metrics
//! - Smooth LOD transitions
//! - Hysteresis to prevent LOD popping
//! - Performance monitoring

use crate::core::math::{Vector3};
use crate::core::primitives::{BoundingBox3, Point3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// LOD level definition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LodLevel(pub u8);

impl LodLevel {
    /// Highest detail level (LOD 0)
    pub const HIGHEST: Self = Self(0);

    /// High detail level (LOD 1)
    pub const HIGH: Self = Self(1);

    /// Medium detail level (LOD 2)
    pub const MEDIUM: Self = Self(2);

    /// Low detail level (LOD 3)
    pub const LOW: Self = Self(3);

    /// Lowest detail level (LOD 4)
    pub const LOWEST: Self = Self(4);

    /// Create a new LOD level
    pub fn new(level: u8) -> Self {
        Self(level)
    }

    /// Get the level value
    pub fn value(&self) -> u8 {
        self.0
    }

    /// Get next lower detail level
    pub fn next_lower(&self) -> Option<Self> {
        if self.0 < 255 {
            Some(Self(self.0 + 1))
        } else {
            None
        }
    }

    /// Get next higher detail level
    pub fn next_higher(&self) -> Option<Self> {
        if self.0 > 0 {
            Some(Self(self.0 - 1))
        } else {
            None
        }
    }
}

/// LOD selection strategy
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LodStrategy {
    /// Distance-based LOD selection
    Distance {
        /// Distance thresholds for each LOD level
        /// [LOD0->1, LOD1->2, LOD2->3, ...]
        thresholds: [f32; 4],
    },

    /// Screen-space error metric
    ScreenSpaceError {
        /// Target pixel error threshold
        pixel_threshold: f32,
    },

    /// Geometric error metric
    GeometricError {
        /// Maximum allowed geometric error
        max_error: f32,
    },

    /// Hybrid approach (distance + screen space)
    Hybrid {
        /// Distance weight (0.0 to 1.0)
        distance_weight: f32,
        /// Distance thresholds
        distance_thresholds: [f32; 4],
        /// Screen space threshold
        screen_threshold: f32,
    },
}

impl Default for LodStrategy {
    fn default() -> Self {
        Self::Distance {
            thresholds: [10.0, 50.0, 200.0, 500.0],
        }
    }
}

/// LOD configuration for an object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodConfig {
    /// LOD selection strategy
    pub strategy: LodStrategy,

    /// Enable LOD transitions
    pub enable_transitions: bool,

    /// Transition duration in seconds
    pub transition_duration: f32,

    /// Hysteresis factor (prevents LOD popping)
    /// Values > 1.0 create a buffer zone
    pub hysteresis: f32,

    /// Minimum LOD level (highest detail)
    pub min_level: LodLevel,

    /// Maximum LOD level (lowest detail)
    pub max_level: LodLevel,

    /// Force a specific LOD level (for debugging)
    pub force_level: Option<LodLevel>,
}

impl Default for LodConfig {
    fn default() -> Self {
        Self {
            strategy: LodStrategy::default(),
            enable_transitions: true,
            transition_duration: 0.3,
            hysteresis: 1.2,
            min_level: LodLevel::HIGHEST,
            max_level: LodLevel::LOWEST,
            force_level: None,
        }
    }
}

/// LOD state for an object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodState {
    /// Current LOD level
    pub current_level: LodLevel,

    /// Target LOD level (for transitions)
    pub target_level: LodLevel,

    /// Transition progress (0.0 to 1.0)
    pub transition_progress: f32,

    /// Is currently transitioning
    pub transitioning: bool,

    /// Last distance to camera
    pub last_distance: f32,

    /// Last screen-space size
    pub last_screen_size: f32,
}

impl Default for LodState {
    fn default() -> Self {
        Self {
            current_level: LodLevel::HIGHEST,
            target_level: LodLevel::HIGHEST,
            transition_progress: 1.0,
            transitioning: false,
            last_distance: 0.0,
            last_screen_size: 0.0,
        }
    }
}

impl LodState {
    /// Update transition
    pub fn update_transition(&mut self, delta_time: f32, duration: f32) {
        if !self.transitioning {
            return;
        }

        self.transition_progress += delta_time / duration;

        if self.transition_progress >= 1.0 {
            self.transition_progress = 1.0;
            self.current_level = self.target_level;
            self.transitioning = false;
        }
    }

    /// Start transition to new LOD level
    pub fn transition_to(&mut self, new_level: LodLevel) {
        if new_level != self.current_level {
            self.target_level = new_level;
            self.transition_progress = 0.0;
            self.transitioning = true;
        }
    }

    /// Immediately set LOD level without transition
    pub fn set_immediate(&mut self, level: LodLevel) {
        self.current_level = level;
        self.target_level = level;
        self.transition_progress = 1.0;
        self.transitioning = false;
    }
}

/// LOD metrics and statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LodMetrics {
    /// Number of objects at each LOD level
    pub objects_per_level: HashMap<u8, u32>,

    /// Total number of LOD selections this frame
    pub selections_this_frame: u32,

    /// Number of LOD transitions this frame
    pub transitions_this_frame: u32,

    /// Average distance for LOD selection
    pub average_distance: f32,

    /// Total vertices rendered (considering LOD)
    pub total_vertices: u32,

    /// Estimated vertices saved by LOD
    pub vertices_saved: u32,
}

impl LodMetrics {
    /// Reset metrics for new frame
    pub fn reset(&mut self) {
        self.objects_per_level.clear();
        self.selections_this_frame = 0;
        self.transitions_this_frame = 0;
        self.average_distance = 0.0;
        self.total_vertices = 0;
        self.vertices_saved = 0;
    }

    /// Record LOD selection
    pub fn record_selection(&mut self, level: LodLevel, distance: f32) {
        *self.objects_per_level.entry(level.0).or_insert(0) += 1;
        self.selections_this_frame += 1;

        // Update average distance
        let n = self.selections_this_frame as f32;
        self.average_distance = (self.average_distance * (n - 1.0) + distance) / n;
    }

    /// Record transition
    pub fn record_transition(&mut self) {
        self.transitions_this_frame += 1;
    }

    /// Record vertex counts
    pub fn record_vertices(&mut self, rendered: u32, saved: u32) {
        self.total_vertices += rendered;
        self.vertices_saved += saved;
    }

    /// Get LOD efficiency (percentage of vertices saved)
    pub fn efficiency(&self) -> f32 {
        let total = self.total_vertices + self.vertices_saved;
        if total == 0 {
            0.0
        } else {
            (self.vertices_saved as f32 / total as f32) * 100.0
        }
    }
}

/// LOD manager for scene-wide LOD management
pub struct LodManager {
    /// Configuration
    config: LodConfig,

    /// LOD states for all objects
    states: HashMap<u32, LodState>,

    /// Metrics
    metrics: LodMetrics,

    /// Camera position (for distance calculations)
    camera_position: Point3,

    /// Viewport dimensions (for screen-space calculations)
    viewport_size: (f32, f32),

    /// Field of view (for screen-space calculations)
    fov: f32,
}

impl LodManager {
    /// Create a new LOD manager
    pub fn new(config: LodConfig) -> Self {
        Self {
            config,
            states: HashMap::new(),
            metrics: LodMetrics::default(),
            camera_position: Point3::new(0.0, 0.0, 0.0),
            viewport_size: (1920.0, 1080.0),
            fov: std::f32::consts::PI / 4.0,
        }
    }

    /// Update camera information
    pub fn update_camera(&mut self, position: Point3, viewport_size: (f32, f32), fov: f32) {
        self.camera_position = position;
        self.viewport_size = viewport_size;
        self.fov = fov;
    }

    /// Select LOD level for an object
    pub fn select_lod(
        &mut self,
        object_id: u32,
        bounds: &BoundingBox3,
        delta_time: f32,
    ) -> LodLevel {
        // Check for forced LOD level
        if let Some(forced) = self.config.force_level {
            return forced;
        }

        // Get or create state
        let state = self.states.entry(object_id).or_default();

        // Calculate distance to camera
        let center = bounds.center();
        let distance = (center - self.camera_position).magnitude();

        // Calculate screen-space size
        let screen_size = self.calculate_screen_size(bounds, distance);

        // Update state
        state.last_distance = distance;
        state.last_screen_size = screen_size;

        // Select LOD based on strategy
        let selected_level = match self.config.strategy {
            LodStrategy::Distance { thresholds } => {
                self.select_by_distance(distance, &thresholds, state)
            }
            LodStrategy::ScreenSpaceError { pixel_threshold } => {
                self.select_by_screen_space(screen_size, pixel_threshold)
            }
            LodStrategy::GeometricError { max_error } => {
                self.select_by_geometric_error(bounds, distance, max_error)
            }
            LodStrategy::Hybrid {
                distance_weight,
                distance_thresholds,
                screen_threshold,
            } => self.select_hybrid(
                distance,
                screen_size,
                distance_weight,
                &distance_thresholds,
                screen_threshold,
                state,
            ),
        };

        // Clamp to min/max levels
        let clamped_level = LodLevel::new(
            selected_level
                .0
                .max(self.config.min_level.0)
                .min(self.config.max_level.0),
        );

        // Handle transitions
        if self.config.enable_transitions {
            if clamped_level != state.current_level {
                state.transition_to(clamped_level);
                self.metrics.record_transition();
            }
            state.update_transition(delta_time, self.config.transition_duration);
        } else {
            state.set_immediate(clamped_level);
        }

        // Record metrics
        self.metrics.record_selection(state.current_level, distance);

        state.current_level
    }

    /// Select LOD by distance
    fn select_by_distance(
        &self,
        distance: f32,
        thresholds: &[f32; 4],
        state: &LodState,
    ) -> LodLevel {
        // Apply hysteresis to prevent LOD popping
        let hysteresis_factor = if state.transitioning {
            1.0
        } else if distance > state.last_distance {
            self.config.hysteresis
        } else {
            1.0 / self.config.hysteresis
        };

        for (i, &threshold) in thresholds.iter().enumerate() {
            if distance < threshold * hysteresis_factor {
                return LodLevel::new(i as u8);
            }
        }

        LodLevel::new(thresholds.len() as u8)
    }

    /// Select LOD by screen-space size
    fn select_by_screen_space(&self, screen_size: f32, pixel_threshold: f32) -> LodLevel {
        // More pixels = higher detail needed
        if screen_size > pixel_threshold * 4.0 {
            LodLevel::HIGHEST
        } else if screen_size > pixel_threshold * 2.0 {
            LodLevel::HIGH
        } else if screen_size > pixel_threshold {
            LodLevel::MEDIUM
        } else if screen_size > pixel_threshold * 0.5 {
            LodLevel::LOW
        } else {
            LodLevel::LOWEST
        }
    }

    /// Select LOD by geometric error
    fn select_by_geometric_error(
        &self,
        bounds: &BoundingBox3,
        distance: f32,
        max_error: f32,
    ) -> LodLevel {
        // Calculate acceptable error based on distance
        let acceptable_error = max_error * (distance / 100.0);

        let size = bounds.size();
        let max_dimension = size.x.max(size.y).max(size.z);

        // Higher LOD for smaller acceptable error
        if acceptable_error < max_dimension * 0.001 {
            LodLevel::HIGHEST
        } else if acceptable_error < max_dimension * 0.01 {
            LodLevel::HIGH
        } else if acceptable_error < max_dimension * 0.05 {
            LodLevel::MEDIUM
        } else if acceptable_error < max_dimension * 0.1 {
            LodLevel::LOW
        } else {
            LodLevel::LOWEST
        }
    }

    /// Hybrid LOD selection
    fn select_hybrid(
        &self,
        distance: f32,
        screen_size: f32,
        distance_weight: f32,
        distance_thresholds: &[f32; 4],
        screen_threshold: f32,
        state: &LodState,
    ) -> LodLevel {
        let distance_lod = self.select_by_distance(distance, distance_thresholds, state);
        let screen_lod = self.select_by_screen_space(screen_size, screen_threshold);

        // Weighted combination (take higher detail)
        let weighted_level =
            (distance_lod.0 as f32 * distance_weight + screen_lod.0 as f32 * (1.0 - distance_weight))
                .round() as u8;

        LodLevel::new(weighted_level)
    }

    /// Calculate screen-space size of bounding box
    fn calculate_screen_size(&self, bounds: &BoundingBox3, distance: f32) -> f32 {
        if distance <= 0.0 {
            return self.viewport_size.1; // Return max if at camera
        }

        let size = bounds.size();
        let max_dimension = size.x.max(size.y).max(size.z);

        // Project to screen space using perspective projection
        let half_fov_tan = (self.fov / 2.0).tan();
        let screen_height = self.viewport_size.1;

        let projected_size = (max_dimension / distance) / half_fov_tan * (screen_height / 2.0);

        projected_size
    }

    /// Get LOD state for an object
    pub fn get_state(&self, object_id: u32) -> Option<&LodState> {
        self.states.get(&object_id)
    }

    /// Get metrics
    pub fn metrics(&self) -> &LodMetrics {
        &self.metrics
    }

    /// Reset metrics
    pub fn reset_metrics(&mut self) {
        self.metrics.reset();
    }

    /// Update all LOD states (for transitions)
    pub fn update(&mut self, delta_time: f32) {
        for state in self.states.values_mut() {
            state.update_transition(delta_time, self.config.transition_duration);
        }
    }

    /// Clear all LOD states
    pub fn clear(&mut self) {
        self.states.clear();
        self.metrics.reset();
    }

    /// Get configuration
    pub fn config(&self) -> &LodConfig {
        &self.config
    }

    /// Set configuration
    pub fn set_config(&mut self, config: LodConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lod_level() {
        let lod = LodLevel::HIGHEST;
        assert_eq!(lod.value(), 0);

        let next = lod.next_lower().unwrap();
        assert_eq!(next.value(), 1);

        let prev = next.next_higher().unwrap();
        assert_eq!(prev.value(), 0);
    }

    #[test]
    fn test_lod_state_transition() {
        let mut state = LodState::default();

        state.transition_to(LodLevel::MEDIUM);
        assert!(state.transitioning);
        assert_eq!(state.target_level, LodLevel::MEDIUM);

        state.update_transition(0.3, 0.3);
        assert!(!state.transitioning);
        assert_eq!(state.current_level, LodLevel::MEDIUM);
    }

    #[test]
    fn test_lod_manager() {
        let config = LodConfig::default();
        let mut manager = LodManager::new(config);

        manager.update_camera(Point3::new(0.0, 0.0, 0.0), (1920.0, 1080.0), std::f32::consts::PI / 4.0);

        let bounds = BoundingBox3::new(
            Point3::new(-1.0, -1.0, -1.0),
            Point3::new(1.0, 1.0, 1.0),
        );

        // Close object should be high detail
        manager.camera_position = Point3::new(0.0, 0.0, 5.0);
        let lod = manager.select_lod(0, &bounds, 0.016);
        assert_eq!(lod, LodLevel::HIGHEST);

        // Far object should be low detail
        manager.camera_position = Point3::new(0.0, 0.0, 1000.0);
        let lod = manager.select_lod(1, &bounds, 0.016);
        assert!(lod.value() > LodLevel::MEDIUM.value());
    }

    #[test]
    fn test_lod_metrics() {
        let mut metrics = LodMetrics::default();

        metrics.record_selection(LodLevel::HIGHEST, 10.0);
        metrics.record_selection(LodLevel::HIGH, 20.0);

        assert_eq!(metrics.selections_this_frame, 2);
        assert_eq!(metrics.average_distance, 15.0);
    }

    #[test]
    fn test_screen_size_calculation() {
        let config = LodConfig::default();
        let mut manager = LodManager::new(config);

        manager.update_camera(
            Point3::new(0.0, 0.0, 0.0),
            (1920.0, 1080.0),
            std::f32::consts::PI / 4.0,
        );

        let bounds = BoundingBox3::new(
            Point3::new(-1.0, -1.0, -1.0),
            Point3::new(1.0, 1.0, 1.0),
        );

        let screen_size = manager.calculate_screen_size(&bounds, 10.0);
        assert!(screen_size > 0.0);
    }
}
