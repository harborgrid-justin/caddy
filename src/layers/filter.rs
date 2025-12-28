// Layer filter module for CADDY
// Provides layer filtering and grouping capabilities

use super::layer::Layer;
use crate::core::Color;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Layer filter for selecting subsets of layers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerFilter {
    /// Filter name
    pub name: String,

    /// Filter description
    pub description: String,

    /// Filter criteria
    criteria: Vec<FilterCriterion>,
}

impl LayerFilter {
    /// Create a new layer filter
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            criteria: Vec::new(),
        }
    }

    /// Add a filter criterion
    pub fn add_criterion(&mut self, criterion: FilterCriterion) {
        self.criteria.push(criterion);
    }

    /// Remove all criteria
    pub fn clear_criteria(&mut self) {
        self.criteria.clear();
    }

    /// Check if a layer matches this filter
    pub fn matches(&self, layer: &Layer) -> bool {
        if self.criteria.is_empty() {
            // Empty filter matches all layers
            return true;
        }

        // All criteria must match (AND logic)
        self.criteria.iter().all(|c| c.matches(layer))
    }

    /// Filter a collection of layers
    pub fn filter_layers<'a>(&self, layers: &'a [&'a Layer]) -> Vec<&'a Layer> {
        layers
            .iter()
            .filter(|layer| self.matches(layer))
            .copied()
            .collect()
    }

    /// Get the number of criteria
    pub fn criterion_count(&self) -> usize {
        self.criteria.len()
    }

    /// Check if filter is empty (no criteria)
    pub fn is_empty(&self) -> bool {
        self.criteria.is_empty()
    }
}

/// Filter criterion for layer selection
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FilterCriterion {
    /// Filter by exact name match
    NameEquals(String),

    /// Filter by name pattern (wildcards: * and ?)
    NamePattern(String),

    /// Filter by name prefix
    NameStartsWith(String),

    /// Filter by name suffix
    NameEndsWith(String),

    /// Filter by name contains substring
    NameContains(String),

    /// Filter by color
    ColorEquals(Color),

    /// Filter by visibility
    Visible(bool),

    /// Filter by frozen state
    Frozen(bool),

    /// Filter by locked state
    Locked(bool),

    /// Filter by printable state
    Printable(bool),

    /// Filter by whether layer is in a specific group
    InGroup(String),

    /// Negation of another criterion
    Not(Box<FilterCriterion>),
}

impl FilterCriterion {
    /// Check if a layer matches this criterion
    pub fn matches(&self, layer: &Layer) -> bool {
        match self {
            FilterCriterion::NameEquals(name) => layer.name == *name,
            FilterCriterion::NamePattern(pattern) => match_pattern(&layer.name, pattern),
            FilterCriterion::NameStartsWith(prefix) => layer.name.starts_with(prefix),
            FilterCriterion::NameEndsWith(suffix) => layer.name.ends_with(suffix),
            FilterCriterion::NameContains(substring) => layer.name.contains(substring),
            FilterCriterion::ColorEquals(color) => layer.color == *color,
            FilterCriterion::Visible(visible) => layer.visible == *visible,
            FilterCriterion::Frozen(frozen) => layer.frozen == *frozen,
            FilterCriterion::Locked(locked) => layer.locked == *locked,
            FilterCriterion::Printable(printable) => layer.printable == *printable,
            FilterCriterion::InGroup(_group) => {
                // Group membership would be tracked separately
                // For now, return false
                false
            }
            FilterCriterion::Not(criterion) => !criterion.matches(layer),
        }
    }
}

/// Match a string against a pattern with wildcards (* and ?)
fn match_pattern(text: &str, pattern: &str) -> bool {
    let text_chars: Vec<char> = text.chars().collect();
    let pattern_chars: Vec<char> = pattern.chars().collect();

    match_pattern_impl(&text_chars, &pattern_chars, 0, 0)
}

fn match_pattern_impl(
    text: &[char],
    pattern: &[char],
    text_pos: usize,
    pattern_pos: usize,
) -> bool {
    // If both exhausted, match found
    if pattern_pos >= pattern.len() && text_pos >= text.len() {
        return true;
    }

    // If pattern exhausted but text remains, no match
    if pattern_pos >= pattern.len() {
        return false;
    }

    // Handle wildcard *
    if pattern[pattern_pos] == '*' {
        // Try matching zero characters
        if match_pattern_impl(text, pattern, text_pos, pattern_pos + 1) {
            return true;
        }

        // Try matching one or more characters
        for i in text_pos..text.len() {
            if match_pattern_impl(text, pattern, i + 1, pattern_pos + 1) {
                return true;
            }
        }

        return false;
    }

    // If text exhausted but pattern remains, no match (unless remaining is all *)
    if text_pos >= text.len() {
        return pattern[pattern_pos..].iter().all(|&c| c == '*');
    }

    // Handle wildcard ?
    if pattern[pattern_pos] == '?' {
        return match_pattern_impl(text, pattern, text_pos + 1, pattern_pos + 1);
    }

    // Handle regular character match
    if text[text_pos] == pattern[pattern_pos] {
        return match_pattern_impl(text, pattern, text_pos + 1, pattern_pos + 1);
    }

    false
}

/// Layer group - a named collection of layers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerGroup {
    /// Group name
    pub name: String,

    /// Group description
    pub description: String,

    /// Layer names in this group
    layers: HashSet<String>,
}

impl LayerGroup {
    /// Create a new layer group
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            layers: HashSet::new(),
        }
    }

    /// Add a layer to this group
    pub fn add_layer(&mut self, layer_name: String) {
        self.layers.insert(layer_name);
    }

    /// Remove a layer from this group
    pub fn remove_layer(&mut self, layer_name: &str) -> bool {
        self.layers.remove(layer_name)
    }

    /// Check if a layer is in this group
    pub fn contains_layer(&self, layer_name: &str) -> bool {
        self.layers.contains(layer_name)
    }

    /// Get all layer names in this group
    pub fn layer_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.layers.iter().cloned().collect();
        names.sort();
        names
    }

    /// Get the number of layers in this group
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Clear all layers from this group
    pub fn clear(&mut self) {
        self.layers.clear();
    }

    /// Check if group is empty
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }
}

/// Manager for layer groups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerGroupManager {
    /// Map of group name to group
    groups: std::collections::HashMap<String, LayerGroup>,
}

impl LayerGroupManager {
    /// Create a new layer group manager
    pub fn new() -> Self {
        Self {
            groups: std::collections::HashMap::new(),
        }
    }

    /// Create a new group
    pub fn create_group(
        &mut self,
        name: String,
        description: String,
    ) -> Result<(), LayerFilterError> {
        if self.groups.contains_key(&name) {
            return Err(LayerFilterError::GroupAlreadyExists(name));
        }

        let group = LayerGroup::new(name.clone(), description);
        self.groups.insert(name, group);
        Ok(())
    }

    /// Delete a group
    pub fn delete_group(&mut self, name: &str) -> Result<(), LayerFilterError> {
        if self.groups.remove(name).is_some() {
            Ok(())
        } else {
            Err(LayerFilterError::GroupNotFound(name.to_string()))
        }
    }

    /// Get a group by name
    pub fn get_group(&self, name: &str) -> Option<&LayerGroup> {
        self.groups.get(name)
    }

    /// Get a mutable group by name
    pub fn get_group_mut(&mut self, name: &str) -> Option<&mut LayerGroup> {
        self.groups.get_mut(name)
    }

    /// Add a layer to a group
    pub fn add_layer_to_group(
        &mut self,
        group_name: &str,
        layer_name: String,
    ) -> Result<(), LayerFilterError> {
        if let Some(group) = self.groups.get_mut(group_name) {
            group.add_layer(layer_name);
            Ok(())
        } else {
            Err(LayerFilterError::GroupNotFound(group_name.to_string()))
        }
    }

    /// Remove a layer from a group
    pub fn remove_layer_from_group(
        &mut self,
        group_name: &str,
        layer_name: &str,
    ) -> Result<(), LayerFilterError> {
        if let Some(group) = self.groups.get_mut(group_name) {
            group.remove_layer(layer_name);
            Ok(())
        } else {
            Err(LayerFilterError::GroupNotFound(group_name.to_string()))
        }
    }

    /// Get all group names
    pub fn group_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.groups.keys().cloned().collect();
        names.sort();
        names
    }

    /// Get number of groups
    pub fn group_count(&self) -> usize {
        self.groups.len()
    }

    /// Find all groups containing a layer
    pub fn groups_containing_layer(&self, layer_name: &str) -> Vec<String> {
        self.groups
            .iter()
            .filter(|(_, group)| group.contains_layer(layer_name))
            .map(|(name, _)| name.clone())
            .collect()
    }
}

impl Default for LayerGroupManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Layer filter errors
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum LayerFilterError {
    #[error("Layer group '{0}' not found")]
    GroupNotFound(String),

    #[error("Layer group '{0}' already exists")]
    GroupAlreadyExists(String),

    #[error("Invalid filter pattern: {0}")]
    InvalidPattern(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_by_name() {
        let layer1 = Layer::new("Layer1".to_string());
        let layer2 = Layer::new("Layer2".to_string());

        let criterion = FilterCriterion::NameEquals("Layer1".to_string());
        assert!(criterion.matches(&layer1));
        assert!(!criterion.matches(&layer2));
    }

    #[test]
    fn test_filter_by_pattern() {
        let layer = Layer::new("DIM_Wall_001".to_string());

        let criterion = FilterCriterion::NamePattern("DIM_*".to_string());
        assert!(criterion.matches(&layer));

        let criterion2 = FilterCriterion::NamePattern("*_Wall_*".to_string());
        assert!(criterion2.matches(&layer));

        let criterion3 = FilterCriterion::NamePattern("ARCH_*".to_string());
        assert!(!criterion3.matches(&layer));
    }

    #[test]
    fn test_filter_by_state() {
        let mut layer = Layer::new("Test".to_string());
        layer.frozen = true;

        let criterion = FilterCriterion::Frozen(true);
        assert!(criterion.matches(&layer));

        let criterion2 = FilterCriterion::Visible(true);
        assert!(criterion2.matches(&layer));
    }

    #[test]
    fn test_filter_negation() {
        let mut layer = Layer::new("Test".to_string());
        layer.frozen = true;

        let criterion = FilterCriterion::Not(Box::new(FilterCriterion::Frozen(false)));
        assert!(criterion.matches(&layer));
    }

    #[test]
    fn test_layer_filter() {
        let mut filter = LayerFilter::new("MyFilter".to_string(), "Test".to_string());
        filter.add_criterion(FilterCriterion::NameStartsWith("DIM_".to_string()));
        filter.add_criterion(FilterCriterion::Visible(true));

        let mut layer1 = Layer::new("DIM_Wall".to_string());
        layer1.visible = true;
        let mut layer2 = Layer::new("DIM_Door".to_string());
        layer2.visible = false;
        let layer3 = Layer::new("ARCH_Wall".to_string());

        assert!(filter.matches(&layer1));
        assert!(!filter.matches(&layer2));
        assert!(!filter.matches(&layer3));
    }

    #[test]
    fn test_filter_layers() {
        let mut filter = LayerFilter::new("Visible".to_string(), "All visible".to_string());
        filter.add_criterion(FilterCriterion::Visible(true));

        let mut layer1 = Layer::new("L1".to_string());
        layer1.visible = true;
        let mut layer2 = Layer::new("L2".to_string());
        layer2.visible = false;
        let mut layer3 = Layer::new("L3".to_string());
        layer3.visible = true;

        let layers = vec![&layer1, &layer2, &layer3];
        let filtered = filter.filter_layers(&layers);

        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].name, "L1");
        assert_eq!(filtered[1].name, "L3");
    }

    #[test]
    fn test_pattern_matching() {
        assert!(match_pattern("Layer1", "Layer*"));
        assert!(match_pattern("Layer123", "Layer*"));
        assert!(match_pattern("Layer", "Layer*"));
        assert!(match_pattern("Layer1", "Layer?"));
        assert!(!match_pattern("Layer12", "Layer?"));
        assert!(match_pattern("DIM_Wall_001", "*_Wall_*"));
        assert!(match_pattern("test", "*"));
        assert!(!match_pattern("test", "x*"));
    }

    #[test]
    fn test_layer_group() {
        let mut group = LayerGroup::new("Dimensions".to_string(), "All dim layers".to_string());
        group.add_layer("DIM_1".to_string());
        group.add_layer("DIM_2".to_string());

        assert_eq!(group.layer_count(), 2);
        assert!(group.contains_layer("DIM_1"));
        assert!(!group.contains_layer("ARCH_1"));

        group.remove_layer("DIM_1");
        assert_eq!(group.layer_count(), 1);
    }

    #[test]
    fn test_layer_group_manager() {
        let mut mgr = LayerGroupManager::new();
        mgr.create_group("Group1".to_string(), "Test".to_string())
            .unwrap();

        assert_eq!(mgr.group_count(), 1);

        mgr.add_layer_to_group("Group1", "Layer1".to_string())
            .unwrap();
        assert!(mgr.get_group("Group1").unwrap().contains_layer("Layer1"));
    }

    #[test]
    fn test_groups_containing_layer() {
        let mut mgr = LayerGroupManager::new();
        mgr.create_group("Group1".to_string(), "Test".to_string())
            .unwrap();
        mgr.create_group("Group2".to_string(), "Test".to_string())
            .unwrap();

        mgr.add_layer_to_group("Group1", "LayerA".to_string())
            .unwrap();
        mgr.add_layer_to_group("Group2", "LayerA".to_string())
            .unwrap();

        let groups = mgr.groups_containing_layer("LayerA");
        assert_eq!(groups.len(), 2);
    }
}
