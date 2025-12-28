//! # CRDT (Conflict-free Replicated Data Types)
//!
//! Implements various CRDTs for real-time collaboration without coordination.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::cmp::Ordering;
use uuid::Uuid;

/// Unique identifier for a replica/node
pub type ReplicaId = Uuid;

/// Lamport timestamp for causality tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LamportTime(pub u64);

impl LamportTime {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn tick(&mut self) {
        self.0 += 1;
    }

    pub fn update(&mut self, other: LamportTime) {
        self.0 = self.0.max(other.0) + 1;
    }
}

impl Default for LamportTime {
    fn default() -> Self {
        Self::new()
    }
}

/// Vector clock for tracking causality across replicas
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorClock {
    clock: BTreeMap<ReplicaId, u64>,
}

impl VectorClock {
    pub fn new() -> Self {
        Self {
            clock: BTreeMap::new(),
        }
    }

    /// Increment the local clock for a replica
    pub fn increment(&mut self, replica: ReplicaId) {
        *self.clock.entry(replica).or_insert(0) += 1;
    }

    /// Get the current value for a replica
    pub fn get(&self, replica: &ReplicaId) -> u64 {
        self.clock.get(replica).copied().unwrap_or(0)
    }

    /// Update this clock with another clock (take maximum)
    pub fn merge(&mut self, other: &VectorClock) {
        for (replica, &value) in &other.clock {
            let entry = self.clock.entry(*replica).or_insert(0);
            *entry = (*entry).max(value);
        }
    }

    /// Check if this clock happened before another
    pub fn happens_before(&self, other: &VectorClock) -> bool {
        let mut strictly_less = false;

        // Get all replica IDs from both clocks
        let mut all_replicas = HashSet::new();
        all_replicas.extend(self.clock.keys());
        all_replicas.extend(other.clock.keys());

        for replica in all_replicas {
            let self_val = self.get(&replica);
            let other_val = other.get(&replica);

            if self_val > other_val {
                return false;
            }
            if self_val < other_val {
                strictly_less = true;
            }
        }

        strictly_less
    }

    /// Check if two clocks are concurrent (neither happens before the other)
    pub fn is_concurrent(&self, other: &VectorClock) -> bool {
        !self.happens_before(other) && !other.happens_before(self)
    }
}

impl Default for VectorClock {
    fn default() -> Self {
        Self::new()
    }
}

/// G-Counter: Grow-only counter (increment only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GCounter {
    replica_id: ReplicaId,
    counts: HashMap<ReplicaId, u64>,
}

impl GCounter {
    pub fn new(replica_id: ReplicaId) -> Self {
        Self {
            replica_id,
            counts: HashMap::new(),
        }
    }

    /// Increment the counter
    pub fn increment(&mut self) {
        *self.counts.entry(self.replica_id).or_insert(0) += 1;
    }

    /// Get the total value across all replicas
    pub fn value(&self) -> u64 {
        self.counts.values().sum()
    }

    /// Merge with another G-Counter
    pub fn merge(&mut self, other: &GCounter) {
        for (replica, &count) in &other.counts {
            let entry = self.counts.entry(*replica).or_insert(0);
            *entry = (*entry).max(count);
        }
    }
}

/// PN-Counter: Positive-Negative counter (increment and decrement)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PNCounter {
    replica_id: ReplicaId,
    positive: HashMap<ReplicaId, u64>,
    negative: HashMap<ReplicaId, u64>,
}

impl PNCounter {
    pub fn new(replica_id: ReplicaId) -> Self {
        Self {
            replica_id,
            positive: HashMap::new(),
            negative: HashMap::new(),
        }
    }

    /// Increment the counter
    pub fn increment(&mut self) {
        *self.positive.entry(self.replica_id).or_insert(0) += 1;
    }

    /// Decrement the counter
    pub fn decrement(&mut self) {
        *self.negative.entry(self.replica_id).or_insert(0) += 1;
    }

    /// Get the current value (positive - negative)
    pub fn value(&self) -> i64 {
        let pos: u64 = self.positive.values().sum();
        let neg: u64 = self.negative.values().sum();
        pos as i64 - neg as i64
    }

    /// Merge with another PN-Counter
    pub fn merge(&mut self, other: &PNCounter) {
        for (replica, &count) in &other.positive {
            let entry = self.positive.entry(*replica).or_insert(0);
            *entry = (*entry).max(count);
        }
        for (replica, &count) in &other.negative {
            let entry = self.negative.entry(*replica).or_insert(0);
            *entry = (*entry).max(count);
        }
    }
}

/// LWW-Register: Last-Write-Wins Register
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LWWRegister<T: Clone> {
    value: T,
    timestamp: LamportTime,
    replica_id: ReplicaId,
}

impl<T: Clone> LWWRegister<T> {
    pub fn new(initial: T, replica_id: ReplicaId) -> Self {
        Self {
            value: initial,
            timestamp: LamportTime::new(),
            replica_id,
        }
    }

    /// Set a new value with the current timestamp
    pub fn set(&mut self, value: T, timestamp: LamportTime) {
        if timestamp > self.timestamp
            || (timestamp == self.timestamp && self.replica_id < self.replica_id) {
            self.value = value;
            self.timestamp = timestamp;
        }
    }

    /// Get the current value
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Merge with another LWW-Register (keep the one with highest timestamp)
    pub fn merge(&mut self, other: &LWWRegister<T>) {
        if other.timestamp > self.timestamp
            || (other.timestamp == self.timestamp && other.replica_id > self.replica_id) {
            self.value = other.value.clone();
            self.timestamp = other.timestamp;
            self.replica_id = other.replica_id;
        }
    }
}

/// OR-Set: Observed-Remove Set
/// Elements are tagged with unique identifiers to distinguish additions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ORSet<T: Clone + Eq + std::hash::Hash> {
    elements: HashMap<T, HashSet<Uuid>>,
}

impl<T: Clone + Eq + std::hash::Hash> ORSet<T> {
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
        }
    }

    /// Add an element with a unique tag
    pub fn add(&mut self, element: T) -> Uuid {
        let tag = Uuid::new_v4();
        self.elements
            .entry(element)
            .or_insert_with(HashSet::new)
            .insert(tag);
        tag
    }

    /// Remove an element by removing all its tags
    pub fn remove(&mut self, element: &T) {
        self.elements.remove(element);
    }

    /// Remove specific tags for an element
    pub fn remove_tags(&mut self, element: &T, tags: &HashSet<Uuid>) {
        if let Some(element_tags) = self.elements.get_mut(element) {
            for tag in tags {
                element_tags.remove(tag);
            }
            if element_tags.is_empty() {
                self.elements.remove(element);
            }
        }
    }

    /// Check if an element is present
    pub fn contains(&self, element: &T) -> bool {
        self.elements.contains_key(element)
    }

    /// Get all elements
    pub fn elements(&self) -> Vec<T> {
        self.elements.keys().cloned().collect()
    }

    /// Get tags for an element
    pub fn get_tags(&self, element: &T) -> Option<&HashSet<Uuid>> {
        self.elements.get(element)
    }

    /// Merge with another OR-Set
    pub fn merge(&mut self, other: &ORSet<T>) {
        for (element, tags) in &other.elements {
            self.elements
                .entry(element.clone())
                .or_insert_with(HashSet::new)
                .extend(tags);
        }
    }
}

impl<T: Clone + Eq + std::hash::Hash> Default for ORSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// RGA: Replicated Growable Array
/// A CRDT for ordered sequences (like text or lists)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RGANode<T: Clone> {
    id: Uuid,
    value: T,
    timestamp: LamportTime,
    replica_id: ReplicaId,
    deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RGA<T: Clone> {
    nodes: Vec<RGANode<T>>,
    replica_id: ReplicaId,
    clock: LamportTime,
}

impl<T: Clone> RGA<T> {
    pub fn new(replica_id: ReplicaId) -> Self {
        Self {
            nodes: Vec::new(),
            replica_id,
            clock: LamportTime::new(),
        }
    }

    /// Insert a value at a specific position
    pub fn insert(&mut self, position: usize, value: T) -> Uuid {
        self.clock.tick();
        let id = Uuid::new_v4();

        let node = RGANode {
            id,
            value,
            timestamp: self.clock,
            replica_id: self.replica_id,
            deleted: false,
        };

        // Find actual position considering deleted nodes
        let actual_pos = self.logical_to_physical_pos(position);
        self.nodes.insert(actual_pos, node);
        id
    }

    /// Delete a value at a specific position
    pub fn delete(&mut self, position: usize) -> Option<Uuid> {
        let actual_pos = self.logical_to_physical_pos(position);
        if actual_pos < self.nodes.len() {
            self.nodes[actual_pos].deleted = true;
            Some(self.nodes[actual_pos].id)
        } else {
            None
        }
    }

    /// Delete by node ID
    pub fn delete_by_id(&mut self, id: Uuid) {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == id) {
            node.deleted = true;
        }
    }

    /// Get the current value sequence (excluding deleted nodes)
    pub fn value(&self) -> Vec<T> {
        self.nodes
            .iter()
            .filter(|n| !n.deleted)
            .map(|n| n.value.clone())
            .collect()
    }

    /// Get length (excluding deleted nodes)
    pub fn len(&self) -> usize {
        self.nodes.iter().filter(|n| !n.deleted).count()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Convert logical position to physical position (accounting for deleted nodes)
    fn logical_to_physical_pos(&self, logical: usize) -> usize {
        let mut logical_count = 0;
        for (physical, node) in self.nodes.iter().enumerate() {
            if !node.deleted {
                if logical_count == logical {
                    return physical;
                }
                logical_count += 1;
            }
        }
        self.nodes.len()
    }

    /// Merge with another RGA
    pub fn merge(&mut self, other: &RGA<T>) {
        // Update local clock
        self.clock.update(other.clock);

        // Merge nodes by timestamp and replica ID
        for other_node in &other.nodes {
            if let Some(local_node) = self.nodes.iter_mut().find(|n| n.id == other_node.id) {
                // Node exists, update deleted status if necessary
                local_node.deleted = local_node.deleted || other_node.deleted;
            } else {
                // New node, insert in correct position
                let insert_pos = self.find_insert_position(other_node);
                self.nodes.insert(insert_pos, other_node.clone());
            }
        }
    }

    /// Find the correct insertion position for a node based on causality
    fn find_insert_position(&self, node: &RGANode<T>) -> usize {
        for (i, existing) in self.nodes.iter().enumerate() {
            match node.timestamp.cmp(&existing.timestamp) {
                Ordering::Less => return i,
                Ordering::Equal => {
                    if node.replica_id < existing.replica_id {
                        return i;
                    }
                }
                Ordering::Greater => continue,
            }
        }
        self.nodes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_clock_happens_before() {
        let mut vc1 = VectorClock::new();
        let mut vc2 = VectorClock::new();

        let r1 = Uuid::new_v4();
        let r2 = Uuid::new_v4();

        vc1.increment(r1);
        assert!(vc1.happens_before(&vc2) == false);

        vc2 = vc1.clone();
        vc2.increment(r2);
        assert!(vc1.happens_before(&vc2));
    }

    #[test]
    fn test_g_counter() {
        let r1 = Uuid::new_v4();
        let r2 = Uuid::new_v4();

        let mut c1 = GCounter::new(r1);
        let mut c2 = GCounter::new(r2);

        c1.increment();
        c1.increment();
        c2.increment();

        assert_eq!(c1.value(), 2);
        assert_eq!(c2.value(), 1);

        c1.merge(&c2);
        assert_eq!(c1.value(), 3);
    }

    #[test]
    fn test_pn_counter() {
        let r1 = Uuid::new_v4();
        let mut counter = PNCounter::new(r1);

        counter.increment();
        counter.increment();
        counter.decrement();

        assert_eq!(counter.value(), 1);
    }

    #[test]
    fn test_lww_register() {
        let r1 = Uuid::new_v4();
        let mut reg = LWWRegister::new(10, r1);

        assert_eq!(*reg.get(), 10);

        reg.set(20, LamportTime(1));
        assert_eq!(*reg.get(), 20);
    }

    #[test]
    fn test_or_set() {
        let mut set = ORSet::new();

        set.add("apple");
        set.add("banana");

        assert!(set.contains(&"apple"));
        assert!(set.contains(&"banana"));
        assert!(!set.contains(&"orange"));

        set.remove(&"apple");
        assert!(!set.contains(&"apple"));
    }

    #[test]
    fn test_rga_insert_delete() {
        let r1 = Uuid::new_v4();
        let mut rga = RGA::new(r1);

        rga.insert(0, 'a');
        rga.insert(1, 'b');
        rga.insert(2, 'c');

        assert_eq!(rga.value(), vec!['a', 'b', 'c']);

        rga.delete(1);
        assert_eq!(rga.value(), vec!['a', 'c']);
    }

    #[test]
    fn test_rga_merge() {
        let r1 = Uuid::new_v4();
        let r2 = Uuid::new_v4();

        let mut rga1 = RGA::new(r1);
        let mut rga2 = RGA::new(r2);

        rga1.insert(0, 'a');
        rga1.insert(1, 'b');

        rga2.insert(0, 'x');
        rga2.insert(1, 'y');

        rga1.merge(&rga2);

        // Should contain all elements
        assert_eq!(rga1.len(), 4);
    }
}
