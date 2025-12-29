//! Cache invalidation protocols and strategies
//!
//! This module provides sophisticated cache invalidation mechanisms:
//! - Tag-based invalidation for grouping related entries
//! - Pattern-based bulk invalidation using wildcards
//! - Pub/sub change notifications for distributed coordination
//! - Cascade invalidation for dependency tracking

use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use std::time::Instant;

use dashmap::DashMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};
use crate::enterprise::error::{EnterpriseError, EnterpriseResult};

/// Invalidation event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvalidationEvent<K> {
    /// Single key invalidation
    Key(K),
    /// Multiple keys invalidation
    Keys(Vec<K>),
    /// Tag-based invalidation
    Tag(String),
    /// Pattern-based invalidation
    Pattern(String),
    /// Full cache clear
    Clear,
}

/// Invalidation reason for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvalidationReason {
    /// Explicit user/application invalidation
    Explicit,
    /// Time-based expiration
    Expired,
    /// Data mutation in backing store
    DataChanged,
    /// Cascading invalidation from dependency
    Cascade,
    /// Policy-based invalidation
    Policy,
}

/// Invalidation metadata
#[derive(Debug, Clone)]
pub struct InvalidationMetadata {
    /// When the invalidation occurred
    pub timestamp: Instant,
    /// Reason for invalidation
    pub reason: InvalidationReason,
    /// Optional source of invalidation
    pub source: Option<String>,
}

/// Cache entry with tags and dependencies
#[derive(Debug, Clone)]
pub struct TaggedEntry<K, V> {
    /// The cached value
    pub value: V,
    /// Associated tags
    pub tags: HashSet<String>,
    /// Keys this entry depends on
    pub dependencies: HashSet<K>,
    /// Keys that depend on this entry
    pub dependents: HashSet<K>,
}

/// Tag-based invalidation manager
pub struct TagInvalidator<K, V> {
    /// Cache entries with tags
    entries: Arc<DashMap<K, TaggedEntry<K, V>>>,
    /// Tag to keys mapping
    tag_index: Arc<DashMap<String, HashSet<K>>>,
    /// Invalidation event broadcaster
    event_tx: broadcast::Sender<InvalidationEvent<K>>,
}

impl<K, V> TagInvalidator<K, V>
where
    K: Eq + Hash + Clone + Debug + Send + Sync + 'static,
    V: Clone + Send + Sync,
{
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(1000);

        Self {
            entries: Arc::new(DashMap::new()),
            tag_index: Arc::new(DashMap::new()),
            event_tx,
        }
    }

    /// Insert an entry with tags
    pub fn insert(&self, key: K, value: V, tags: HashSet<String>) {
        // Update tag index
        for tag in &tags {
            self.tag_index
                .entry(tag.clone())
                .or_insert_with(HashSet::new)
                .insert(key.clone());
        }

        // Insert entry
        let _entry = TaggedEntry {
            value,
            tags,
            dependencies: HashSet::new(),
            dependents: HashSet::new(),
        };

        self.entries.insert(key, entry);
    }

    /// Get an entry
    pub fn get(&self, key: &K) -> Option<V> {
        self.entries.get(key).map(|entry| entry.value.clone())
    }

    /// Invalidate by single key
    pub fn invalidate_key(&self, key: &K) -> EnterpriseResult<()> {
        if let Some((_, entry)) = self.entries.remove(key) {
            // Remove from tag index
            for tag in &entry.tags {
                if let Some(mut keys) = self.tag_index.get_mut(tag) {
                    keys.remove(key);
                }
            }

            // Broadcast invalidation event
            let _ = self.event_tx.send(InvalidationEvent::Key(key.clone()));
        }

        Ok(())
    }

    /// Invalidate all entries with a specific tag
    pub fn invalidate_tag(&self, tag: &str) -> EnterpriseResult<usize> {
        let keys_to_invalidate = if let Some(keys) = self.tag_index.get(tag) {
            keys.clone()
        } else {
            return Ok(0);
        };

        let count = keys_to_invalidate.len();

        for key in &keys_to_invalidate {
            self.invalidate_key(key)?;
        }

        // Broadcast tag invalidation event
        let _ = self.event_tx.send(InvalidationEvent::Tag(tag.to_string()));

        Ok(count)
    }

    /// Invalidate all entries matching any of the given tags
    pub fn invalidate_tags(&self, tags: &[String]) -> EnterpriseResult<usize> {
        let mut keys_to_invalidate = HashSet::new();

        for tag in tags {
            if let Some(keys) = self.tag_index.get(tag) {
                keys_to_invalidate.extend(keys.iter().cloned());
            }
        }

        let count = keys_to_invalidate.len();

        for key in &keys_to_invalidate {
            self.invalidate_key(key)?;
        }

        Ok(count)
    }

    /// Add tags to an existing entry
    pub fn add_tags(&self, key: &K, tags: HashSet<String>) -> EnterpriseResult<()> {
        if let Some(mut entry) = self.entries.get_mut(key) {
            for tag in tags {
                entry.tags.insert(tag.clone());
                self.tag_index
                    .entry(tag)
                    .or_insert_with(HashSet::new)
                    .insert(key.clone());
            }
        }

        Ok(())
    }

    /// Remove tags from an entry
    pub fn remove_tags(&self, key: &K, tags: &HashSet<String>) -> EnterpriseResult<()> {
        if let Some(mut entry) = self.entries.get_mut(key) {
            for tag in tags {
                entry.tags.remove(tag);
                if let Some(mut keys) = self.tag_index.get_mut(tag) {
                    keys.remove(key);
                }
            }
        }

        Ok(())
    }

    /// Get all tags for an entry
    pub fn get_tags(&self, key: &K) -> Option<HashSet<String>> {
        self.entries.get(key).map(|entry| entry.tags.clone())
    }

    /// Subscribe to invalidation events
    pub fn subscribe(&self) -> broadcast::Receiver<InvalidationEvent<K>> {
        self.event_tx.subscribe()
    }

    /// Clear all entries
    pub fn clear(&self) {
        self.entries.clear();
        self.tag_index.clear();
        let _ = self.event_tx.send(InvalidationEvent::Clear);
    }

    /// Get number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl<K, V> Default for TagInvalidator<K, V>
where
    K: Eq + Hash + Clone + Debug + Send + Sync + 'static,
    V: Clone + Send + Sync,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Pattern-based invalidation manager
pub struct PatternInvalidator<K, V> {
    /// Cache entries
    entries: Arc<DashMap<K, V>>,
    /// Invalidation event broadcaster
    event_tx: broadcast::Sender<InvalidationEvent<K>>,
}

impl<K, V> PatternInvalidator<K, V>
where
    K: Eq + Hash + Clone + Debug + Send + Sync + 'static + AsRef<str>,
    V: Clone + Send + Sync,
{
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(1000);

        Self {
            entries: Arc::new(DashMap::new()),
            event_tx,
        }
    }

    /// Insert an entry
    pub fn insert(&self, key: K, value: V) {
        self.entries.insert(key, value);
    }

    /// Get an entry
    pub fn get(&self, key: &K) -> Option<V> {
        self.entries.get(key).map(|entry| entry.clone())
    }

    /// Invalidate entries matching a wildcard pattern
    /// Supports * (any characters) and ? (single character)
    pub fn invalidate_pattern(&self, pattern: &str) -> EnterpriseResult<usize> {
        // Convert wildcard pattern to regex
        let regex_pattern = pattern
            .replace(".", "\\.")
            .replace("*", ".*")
            .replace("?", ".");

        let regex = Regex::new(&format!("^{}$", regex_pattern))
            .map_err(|e| EnterpriseError::Other(format!("Invalid pattern: {}", e)))?;

        let mut invalidated = 0;

        // Find matching keys
        let keys_to_remove: Vec<K> = self.entries
            .iter()
            .filter(|entry| regex.is_match(entry.key().as_ref()))
            .map(|entry| entry.key().clone())
            .collect();

        // Remove matching entries
        for key in keys_to_remove {
            self.entries.remove(&key);
            invalidated += 1;
        }

        if invalidated > 0 {
            let _ = self.event_tx.send(InvalidationEvent::Pattern(pattern.to_string()));
        }

        Ok(invalidated)
    }

    /// Invalidate entries matching a regex pattern
    pub fn invalidate_regex(&self, regex: &Regex) -> EnterpriseResult<usize> {
        let mut invalidated = 0;

        let keys_to_remove: Vec<K> = self.entries
            .iter()
            .filter(|entry| regex.is_match(entry.key().as_ref()))
            .map(|entry| entry.key().clone())
            .collect();

        for key in keys_to_remove {
            self.entries.remove(&key);
            invalidated += 1;
        }

        Ok(invalidated)
    }

    /// Subscribe to invalidation events
    pub fn subscribe(&self) -> broadcast::Receiver<InvalidationEvent<K>> {
        self.event_tx.subscribe()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl<K, V> Default for PatternInvalidator<K, V>
where
    K: Eq + Hash + Clone + Debug + Send + Sync + 'static + AsRef<str>,
    V: Clone + Send + Sync,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Cascade invalidation manager with dependency tracking
pub struct CascadeInvalidator<K, V> {
    /// Cache entries
    entries: Arc<DashMap<K, TaggedEntry<K, V>>>,
    /// Dependency graph (key -> dependencies)
    dependencies: Arc<DashMap<K, HashSet<K>>>,
    /// Reverse dependency graph (key -> dependents)
    dependents: Arc<DashMap<K, HashSet<K>>>,
    /// Invalidation event broadcaster
    event_tx: broadcast::Sender<InvalidationEvent<K>>,
}

impl<K, V> CascadeInvalidator<K, V>
where
    K: Eq + Hash + Clone + Debug + Send + Sync + 'static,
    V: Clone + Send + Sync,
{
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(1000);

        Self {
            entries: Arc::new(DashMap::new()),
            dependencies: Arc::new(DashMap::new()),
            dependents: Arc::new(DashMap::new()),
            event_tx,
        }
    }

    /// Insert an entry with dependencies
    pub fn insert(&self, key: K, value: V, depends_on: HashSet<K>) {
        // Create entry
        let _entry = TaggedEntry {
            value,
            tags: HashSet::new(),
            dependencies: depends_on.clone(),
            dependents: HashSet::new(),
        };

        self.entries.insert(key.clone(), entry);

        // Update dependency graph
        if !depends_on.is_empty() {
            self.dependencies.insert(key.clone(), depends_on.clone());

            // Update reverse dependencies
            for dep in depends_on {
                self.dependents
                    .entry(dep)
                    .or_insert_with(HashSet::new)
                    .insert(key.clone());
            }
        }
    }

    /// Get an entry
    pub fn get(&self, key: &K) -> Option<V> {
        self.entries.get(key).map(|entry| entry.value.clone())
    }

    /// Invalidate a key and all entries that depend on it (cascade)
    pub fn invalidate_cascade(&self, key: &K) -> EnterpriseResult<usize> {
        let mut invalidated = HashSet::new();
        self.invalidate_recursive(key, &mut invalidated)?;

        let count = invalidated.len();
        if count > 0 {
            let keys: Vec<K> = invalidated.into_iter().collect();
            let _ = self.event_tx.send(InvalidationEvent::Keys(keys));
        }

        Ok(count)
    }

    /// Recursive invalidation helper
    fn invalidate_recursive(&self, key: &K, invalidated: &mut HashSet<K>) -> EnterpriseResult<()> {
        if invalidated.contains(key) {
            return Ok(()); // Already invalidated
        }

        // Get all entries that depend on this key
        let dependents = if let Some(deps) = self.dependents.get(key) {
            deps.clone()
        } else {
            HashSet::new()
        };

        // Invalidate dependents first (depth-first)
        for dependent in &dependents {
            self.invalidate_recursive(dependent, invalidated)?;
        }

        // Remove the entry
        self.entries.remove(key);
        self.dependencies.remove(key);
        self.dependents.remove(key);

        invalidated.insert(key.clone());

        Ok(())
    }

    /// Add a dependency relationship
    pub fn add_dependency(&self, key: &K, depends_on: K) -> EnterpriseResult<()> {
        // Update dependency graph
        self.dependencies
            .entry(key.clone())
            .or_insert_with(HashSet::new)
            .insert(depends_on.clone());

        // Update reverse dependencies
        self.dependents
            .entry(depends_on.clone())
            .or_insert_with(HashSet::new)
            .insert(key.clone());

        // Update entry
        if let Some(mut entry) = self.entries.get_mut(key) {
            entry.dependencies.insert(depends_on);
        }

        Ok(())
    }

    /// Remove a dependency relationship
    pub fn remove_dependency(&self, key: &K, dependency: &K) -> EnterpriseResult<()> {
        if let Some(mut deps) = self.dependencies.get_mut(key) {
            deps.remove(dependency);
        }

        if let Some(mut dependents) = self.dependents.get_mut(dependency) {
            dependents.remove(key);
        }

        if let Some(mut entry) = self.entries.get_mut(key) {
            entry.dependencies.remove(dependency);
        }

        Ok(())
    }

    /// Get all dependencies for a key
    pub fn get_dependencies(&self, key: &K) -> HashSet<K> {
        self.dependencies.get(key).map_or_else(HashSet::new, |deps| deps.clone())
    }

    /// Get all dependents for a key
    pub fn get_dependents(&self, key: &K) -> HashSet<K> {
        self.dependents.get(key).map_or_else(HashSet::new, |deps| deps.clone())
    }

    /// Detect circular dependencies
    pub fn has_circular_dependency(&self, key: &K) -> bool {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        self.detect_cycle(key, &mut visited, &mut rec_stack)
    }

    fn detect_cycle(&self, key: &K, visited: &mut HashSet<K>, rec_stack: &mut HashSet<K>) -> bool {
        if rec_stack.contains(key) {
            return true; // Found a cycle
        }

        if visited.contains(key) {
            return false; // Already checked this path
        }

        visited.insert(key.clone());
        rec_stack.insert(key.clone());

        if let Some(deps) = self.dependencies.get(key) {
            for dep in deps.iter() {
                if self.detect_cycle(dep, visited, rec_stack) {
                    return true;
                }
            }
        }

        rec_stack.remove(key);
        false
    }

    /// Subscribe to invalidation events
    pub fn subscribe(&self) -> broadcast::Receiver<InvalidationEvent<K>> {
        self.event_tx.subscribe()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl<K, V> Default for CascadeInvalidator<K, V>
where
    K: Eq + Hash + Clone + Debug + Send + Sync + 'static,
    V: Clone + Send + Sync,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Pub/Sub invalidation coordinator for distributed caches
pub struct PubSubInvalidator<K> {
    /// Local subscribers
    subscribers: Arc<RwLock<Vec<broadcast::Sender<InvalidationEvent<K>>>>>,
    /// Event broadcaster
    event_tx: broadcast::Sender<InvalidationEvent<K>>,
}

impl<K> PubSubInvalidator<K>
where
    K: Clone + Send + Sync + Debug + 'static,
{
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(1000);

        Self {
            subscribers: Arc::new(RwLock::new(Vec::new())),
            event_tx,
        }
    }

    /// Publish an invalidation event
    pub async fn publish(&self, event: InvalidationEvent<K>) -> EnterpriseResult<()> {
        // Broadcast to local subscribers
        let _ = self.event_tx.send(event.clone());

        // Broadcast to all registered subscribers
        let subscribers = self.subscribers.read().await;
        for tx in subscribers.iter() {
            let _ = tx.send(event.clone());
        }

        Ok(())
    }

    /// Subscribe to invalidation events
    pub fn subscribe(&self) -> broadcast::Receiver<InvalidationEvent<K>> {
        self.event_tx.subscribe()
    }

    /// Register a new subscriber channel
    pub async fn register_subscriber(&self, tx: broadcast::Sender<InvalidationEvent<K>>) {
        let mut subscribers = self.subscribers.write().await;
        subscribers.push(tx);
    }

    /// Get subscriber count
    pub async fn subscriber_count(&self) -> usize {
        self.subscribers.read().await.len()
    }
}

impl<K> Default for PubSubInvalidator<K>
where
    K: Clone + Send + Sync + Debug + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_invalidator_basic() {
        let invalidator = TagInvalidator::new();

        let mut tags = HashSet::new();
        tags.insert("user:123".to_string());
        tags.insert("type:drawing".to_string());

        invalidator.insert(1, "value1".to_string(), tags);

        assert_eq!(invalidator.get(&1), Some("value1".to_string()));
        assert_eq!(invalidator.len(), 1);
    }

    #[test]
    fn test_tag_invalidator_by_tag() {
        let invalidator = TagInvalidator::new();

        let mut tags1 = HashSet::new();
        tags1.insert("user:123".to_string());

        let mut tags2 = HashSet::new();
        tags2.insert("user:123".to_string());

        invalidator.insert(1, "value1".to_string(), tags1);
        invalidator.insert(2, "value2".to_string(), tags2);

        let count = invalidator.invalidate_tag("user:123").unwrap();
        assert_eq!(count, 2);
        assert_eq!(invalidator.len(), 0);
    }

    #[test]
    fn test_pattern_invalidator() {
        let invalidator = PatternInvalidator::new();

        invalidator.insert("user:123:session", "value1".to_string());
        invalidator.insert("user:123:profile", "value2".to_string());
        invalidator.insert("user:456:session", "value3".to_string());

        let count = invalidator.invalidate_pattern("user:123:*").unwrap();
        assert_eq!(count, 2);
        assert_eq!(invalidator.len(), 1);
    }

    #[test]
    fn test_cascade_invalidator_basic() {
        let invalidator = CascadeInvalidator::new();

        invalidator.insert(1, "value1".to_string(), HashSet::new());
        invalidator.insert(2, "value2".to_string(), HashSet::from([1]));
        invalidator.insert(3, "value3".to_string(), HashSet::from([2]));

        assert_eq!(invalidator.len(), 3);

        // Invalidate key 1, should cascade to 2 and 3
        let count = invalidator.invalidate_cascade(&1).unwrap();
        assert_eq!(count, 3);
        assert_eq!(invalidator.len(), 0);
    }

    #[test]
    fn test_cascade_invalidator_dependencies() {
        let invalidator = CascadeInvalidator::new();

        invalidator.insert(1, "value1".to_string(), HashSet::new());
        invalidator.insert(2, "value2".to_string(), HashSet::from([1]));

        let deps = invalidator.get_dependencies(&2);
        assert!(deps.contains(&1));

        let dependents = invalidator.get_dependents(&1);
        assert!(dependents.contains(&2));
    }

    #[test]
    fn test_cascade_invalidator_circular_dependency() {
        let invalidator = CascadeInvalidator::new();

        invalidator.insert(1, "value1".to_string(), HashSet::from([2]));
        invalidator.insert(2, "value2".to_string(), HashSet::from([3]));
        invalidator.insert(3, "value3".to_string(), HashSet::from([1]));

        assert!(invalidator.has_circular_dependency(&1));
    }

    #[tokio::test]
    async fn test_pubsub_invalidator() {
        let invalidator = PubSubInvalidator::new();

        let mut rx = invalidator.subscribe();

        invalidator.publish(InvalidationEvent::Key(1)).await.unwrap();

        let event = rx.recv().await.unwrap();
        match event {
            InvalidationEvent::Key(k) => assert_eq!(k, 1),
            _ => panic!("Expected Key event"),
        }
    }
}
