//! Conflict-Free Replicated Data Types (CRDTs) for CAD Operations
//!
//! This module implements state-based and operation-based CRDTs specifically designed
//! for CAD operations, ensuring eventual consistency in collaborative editing scenarios.

use super::{CollaborationError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// CRDT identifier for unique element identification
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CRDTId {
    /// Site/user identifier
    pub site_id: Uuid,
    /// Logical timestamp
    pub counter: u64,
}

impl CRDTId {
    /// Create a new CRDT ID
    pub fn new(site_id: Uuid, counter: u64) -> Self {
        Self { site_id, counter }
    }
}

/// Lamport timestamp for causal ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LamportTimestamp {
    /// Logical time
    pub time: u64,
    /// Site identifier for tie-breaking
    pub site_id: Uuid,
}

impl LamportTimestamp {
    /// Create a new timestamp
    pub fn new(time: u64, site_id: Uuid) -> Self {
        Self { time, site_id }
    }

    /// Increment timestamp
    pub fn increment(&mut self) {
        self.time += 1;
    }

    /// Merge with another timestamp (take maximum)
    pub fn merge(&mut self, other: &LamportTimestamp) {
        self.time = self.time.max(other.time) + 1;
    }
}

/// Last-Write-Wins Register (LWW-Register)
/// Used for entity properties that should converge to the latest value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LWWRegister<T> {
    /// Current value
    value: T,
    /// Timestamp of last write
    timestamp: LamportTimestamp,
}

impl<T> LWWRegister<T> {
    /// Create a new LWW register
    pub fn new(value: T, timestamp: LamportTimestamp) -> Self {
        Self { value, timestamp }
    }

    /// Get the current value
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Update the value if the new timestamp is newer
    pub fn update(&mut self, value: T, timestamp: LamportTimestamp) -> bool {
        if timestamp > self.timestamp {
            self.value = value;
            self.timestamp = timestamp;
            true
        } else {
            false
        }
    }

    /// Get the timestamp
    pub fn timestamp(&self) -> &LamportTimestamp {
        &self.timestamp
    }

    /// Merge with another register (keep the one with the latest timestamp)
    pub fn merge(&mut self, other: LWWRegister<T>) {
        if other.timestamp > self.timestamp {
            self.value = other.value;
            self.timestamp = other.timestamp;
        }
    }
}

/// Grow-Only Set (G-Set)
/// Elements can only be added, never removed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GSet<T: Eq + std::hash::Hash + Clone> {
    /// Set of elements
    elements: HashSet<T>,
}

impl<T: Eq + std::hash::Hash + Clone> GSet<T> {
    /// Create a new G-Set
    pub fn new() -> Self {
        Self {
            elements: HashSet::new(),
        }
    }

    /// Add an element
    pub fn insert(&mut self, element: T) {
        self.elements.insert(element);
    }

    /// Check if an element exists
    pub fn contains(&self, element: &T) -> bool {
        self.elements.contains(element)
    }

    /// Get all elements
    pub fn elements(&self) -> impl Iterator<Item = &T> {
        self.elements.iter()
    }

    /// Merge with another G-Set (union)
    pub fn merge(&mut self, other: &GSet<T>) {
        for element in &other.elements {
            self.elements.insert(element.clone());
        }
    }
}

impl<T: Eq + std::hash::Hash + Clone> Default for GSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Two-Phase Set (2P-Set)
/// Elements can be added and removed, but once removed cannot be re-added
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoPhaseSet<T: Eq + std::hash::Hash + Clone> {
    /// Added elements
    added: GSet<T>,
    /// Removed elements
    removed: GSet<T>,
}

impl<T: Eq + std::hash::Hash + Clone> TwoPhaseSet<T> {
    /// Create a new 2P-Set
    pub fn new() -> Self {
        Self {
            added: GSet::new(),
            removed: GSet::new(),
        }
    }

    /// Add an element
    pub fn insert(&mut self, element: T) -> bool {
        if self.removed.contains(&element) {
            false
        } else {
            self.added.insert(element);
            true
        }
    }

    /// Remove an element
    pub fn remove(&mut self, element: T) -> bool {
        if self.added.contains(&element) {
            self.removed.insert(element);
            true
        } else {
            false
        }
    }

    /// Check if an element exists
    pub fn contains(&self, element: &T) -> bool {
        self.added.contains(element) && !self.removed.contains(element)
    }

    /// Get all elements
    pub fn elements(&self) -> Vec<&T> {
        self.added
            .elements()
            .filter(|e| !self.removed.contains(e))
            .collect()
    }

    /// Merge with another 2P-Set
    pub fn merge(&mut self, other: &TwoPhaseSet<T>) {
        self.added.merge(&other.added);
        self.removed.merge(&other.removed);
    }
}

impl<T: Eq + std::hash::Hash + Clone> Default for TwoPhaseSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Observed-Remove Set (OR-Set)
/// Elements can be added and removed multiple times
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ORSet<T: Eq + std::hash::Hash + Clone> {
    /// Map of elements to their unique identifiers
    elements: HashMap<T, HashSet<CRDTId>>,
}

impl<T: Eq + std::hash::Hash + Clone> ORSet<T> {
    /// Create a new OR-Set
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
        }
    }

    /// Add an element with a unique identifier
    pub fn insert(&mut self, element: T, id: CRDTId) {
        self.elements.entry(element).or_default().insert(id);
    }

    /// Remove an element (removes all instances with given IDs)
    pub fn remove(&mut self, element: &T, ids: &HashSet<CRDTId>) {
        if let Some(element_ids) = self.elements.get_mut(element) {
            for id in ids {
                element_ids.remove(id);
            }
            if element_ids.is_empty() {
                self.elements.remove(element);
            }
        }
    }

    /// Check if an element exists
    pub fn contains(&self, element: &T) -> bool {
        self.elements
            .get(element)
            .map(|ids| !ids.is_empty())
            .unwrap_or(false)
    }

    /// Get all elements
    pub fn elements(&self) -> impl Iterator<Item = &T> {
        self.elements.keys()
    }

    /// Get IDs for an element
    pub fn get_ids(&self, element: &T) -> Option<&HashSet<CRDTId>> {
        self.elements.get(element)
    }

    /// Merge with another OR-Set
    pub fn merge(&mut self, other: &ORSet<T>) {
        for (element, ids) in &other.elements {
            self.elements
                .entry(element.clone())
                .or_default()
                .extend(ids.clone());
        }
    }
}

impl<T: Eq + std::hash::Hash + Clone> Default for ORSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// CAD-specific CRDT for entity management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CADEntityCRDT {
    /// Entity ID
    pub entity_id: Uuid,
    /// Entity type (line, circle, arc, etc.)
    pub entity_type: String,
    /// Properties using LWW registers
    pub properties: HashMap<String, LWWRegister<serde_json::Value>>,
    /// Layer assignment
    pub layer: LWWRegister<Option<Uuid>>,
    /// Tombstone flag for deletion
    pub tombstone: Option<LamportTimestamp>,
    /// Creation timestamp
    pub created_at: LamportTimestamp,
    /// Last modified timestamp
    pub modified_at: LamportTimestamp,
}

impl CADEntityCRDT {
    /// Create a new CAD entity CRDT
    pub fn new(
        entity_id: Uuid,
        entity_type: String,
        timestamp: LamportTimestamp,
        layer: Option<Uuid>,
    ) -> Self {
        Self {
            entity_id,
            entity_type,
            properties: HashMap::new(),
            layer: LWWRegister::new(layer, timestamp),
            tombstone: None,
            created_at: timestamp,
            modified_at: timestamp,
        }
    }

    /// Update a property
    pub fn update_property(
        &mut self,
        key: String,
        value: serde_json::Value,
        timestamp: LamportTimestamp,
    ) -> bool {
        if self.tombstone.is_some() {
            return false; // Cannot modify deleted entities
        }

        let updated = self
            .properties
            .entry(key)
            .or_insert(LWWRegister::new(value.clone(), timestamp))
            .update(value, timestamp);

        if updated && timestamp > self.modified_at {
            self.modified_at = timestamp;
        }

        updated
    }

    /// Get a property value
    pub fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.get(key).map(|reg| reg.value())
    }

    /// Update layer assignment
    pub fn update_layer(&mut self, layer: Option<Uuid>, timestamp: LamportTimestamp) -> bool {
        if self.tombstone.is_some() {
            return false;
        }

        let updated = self.layer.update(layer, timestamp);

        if updated && timestamp > self.modified_at {
            self.modified_at = timestamp;
        }

        updated
    }

    /// Mark entity as deleted
    pub fn delete(&mut self, timestamp: LamportTimestamp) -> bool {
        if let Some(tombstone_ts) = &self.tombstone {
            if timestamp > *tombstone_ts {
                self.tombstone = Some(timestamp);
                true
            } else {
                false
            }
        } else {
            self.tombstone = Some(timestamp);
            true
        }
    }

    /// Check if entity is deleted
    pub fn is_deleted(&self) -> bool {
        self.tombstone.is_some()
    }

    /// Merge with another entity CRDT
    pub fn merge(&mut self, other: &CADEntityCRDT) -> Result<()> {
        if self.entity_id != other.entity_id {
            return Err(CollaborationError::Operation(
                "Cannot merge CRDTs with different entity IDs".to_string(),
            ));
        }

        // Merge properties
        for (key, other_reg) in &other.properties {
            self.properties
                .entry(key.clone())
                .or_insert(other_reg.clone())
                .merge(other_reg.clone());
        }

        // Merge layer
        self.layer.merge(other.layer.clone());

        // Merge tombstone (keep the latest)
        match (&self.tombstone, &other.tombstone) {
            (Some(ts1), Some(ts2)) => {
                if ts2 > ts1 {
                    self.tombstone = Some(*ts2);
                }
            }
            (None, Some(ts)) => {
                self.tombstone = Some(*ts);
            }
            _ => {}
        }

        // Update timestamps
        self.created_at = self.created_at.min(other.created_at);
        self.modified_at = self.modified_at.max(other.modified_at);

        Ok(())
    }
}

/// Document CRDT that manages all entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentCRDT {
    /// Document ID
    pub document_id: Uuid,
    /// All entities indexed by ID
    pub entities: HashMap<Uuid, CADEntityCRDT>,
    /// Layer set
    pub layers: ORSet<Uuid>,
    /// Site ID for this replica
    pub site_id: Uuid,
    /// Local counter for generating unique IDs
    pub counter: u64,
}

impl DocumentCRDT {
    /// Create a new document CRDT
    pub fn new(document_id: Uuid, site_id: Uuid) -> Self {
        Self {
            document_id,
            entities: HashMap::new(),
            layers: ORSet::new(),
            site_id,
            counter: 0,
        }
    }

    /// Generate a new lamport timestamp
    pub fn next_timestamp(&mut self) -> LamportTimestamp {
        self.counter += 1;
        LamportTimestamp::new(self.counter, self.site_id)
    }

    /// Generate a new CRDT ID
    pub fn next_crdt_id(&mut self) -> CRDTId {
        self.counter += 1;
        CRDTId::new(self.site_id, self.counter)
    }

    /// Add a new entity
    pub fn add_entity(
        &mut self,
        entity_id: Uuid,
        entity_type: String,
        layer: Option<Uuid>,
    ) -> LamportTimestamp {
        let timestamp = self.next_timestamp();
        let entity = CADEntityCRDT::new(entity_id, entity_type, timestamp, layer);
        self.entities.insert(entity_id, entity);
        timestamp
    }

    /// Update an entity property
    pub fn update_entity_property(
        &mut self,
        entity_id: Uuid,
        property: String,
        value: serde_json::Value,
    ) -> Result<LamportTimestamp> {
        let timestamp = self.next_timestamp();

        let entity = self.entities.get_mut(&entity_id).ok_or_else(|| {
            CollaborationError::Operation(format!("Entity not found: {}", entity_id))
        })?;

        entity.update_property(property, value, timestamp);
        Ok(timestamp)
    }

    /// Delete an entity
    pub fn delete_entity(&mut self, entity_id: Uuid) -> Result<LamportTimestamp> {
        let timestamp = self.next_timestamp();

        let entity = self.entities.get_mut(&entity_id).ok_or_else(|| {
            CollaborationError::Operation(format!("Entity not found: {}", entity_id))
        })?;

        entity.delete(timestamp);
        Ok(timestamp)
    }

    /// Add a layer
    pub fn add_layer(&mut self, layer_id: Uuid) -> CRDTId {
        let crdt_id = self.next_crdt_id();
        self.layers.insert(layer_id, crdt_id.clone());
        crdt_id
    }

    /// Remove a layer
    pub fn remove_layer(&mut self, layer_id: Uuid, ids: &HashSet<CRDTId>) {
        self.layers.remove(&layer_id, ids);
    }

    /// Get all active entities (not tombstoned)
    pub fn active_entities(&self) -> impl Iterator<Item = (&Uuid, &CADEntityCRDT)> {
        self.entities
            .iter()
            .filter(|(_, entity)| !entity.is_deleted())
    }

    /// Merge with another document CRDT
    pub fn merge(&mut self, other: &DocumentCRDT) -> Result<()> {
        if self.document_id != other.document_id {
            return Err(CollaborationError::Operation(
                "Cannot merge CRDTs with different document IDs".to_string(),
            ));
        }

        // Merge entities
        for (entity_id, other_entity) in &other.entities {
            if let Some(entity) = self.entities.get_mut(entity_id) {
                entity.merge(other_entity)?;
            } else {
                self.entities.insert(*entity_id, other_entity.clone());
            }
        }

        // Merge layers
        self.layers.merge(&other.layers);

        // Update counter to avoid conflicts
        self.counter = self.counter.max(other.counter);

        Ok(())
    }

    /// Export snapshot for synchronization
    pub fn snapshot(&self) -> DocumentSnapshot {
        DocumentSnapshot {
            document_id: self.document_id,
            entities: self.entities.clone(),
            layers: self.layers.clone(),
            site_id: self.site_id,
            counter: self.counter,
            timestamp: Utc::now(),
        }
    }

    /// Apply a snapshot
    pub fn apply_snapshot(&mut self, snapshot: DocumentSnapshot) -> Result<()> {
        if self.document_id != snapshot.document_id {
            return Err(CollaborationError::Operation(
                "Cannot apply snapshot with different document ID".to_string(),
            ));
        }

        self.entities = snapshot.entities;
        self.layers = snapshot.layers;
        self.counter = self.counter.max(snapshot.counter);

        Ok(())
    }
}

/// Document snapshot for synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSnapshot {
    /// Document ID
    pub document_id: Uuid,
    /// All entities
    pub entities: HashMap<Uuid, CADEntityCRDT>,
    /// Layers
    pub layers: ORSet<Uuid>,
    /// Site ID
    pub site_id: Uuid,
    /// Counter at snapshot time
    pub counter: u64,
    /// Snapshot timestamp
    pub timestamp: DateTime<Utc>,
}

/// CRDT operation for transmission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CRDTOperation {
    /// Add entity
    AddEntity {
        entity_id: Uuid,
        entity_type: String,
        layer: Option<Uuid>,
        timestamp: LamportTimestamp,
    },

    /// Update entity property
    UpdateProperty {
        entity_id: Uuid,
        property: String,
        value: serde_json::Value,
        timestamp: LamportTimestamp,
    },

    /// Delete entity
    DeleteEntity {
        entity_id: Uuid,
        timestamp: LamportTimestamp,
    },

    /// Add layer
    AddLayer {
        layer_id: Uuid,
        crdt_id: CRDTId,
    },

    /// Remove layer
    RemoveLayer {
        layer_id: Uuid,
        crdt_ids: HashSet<CRDTId>,
    },

    /// Full snapshot
    Snapshot(DocumentSnapshot),
}

impl CRDTOperation {
    /// Apply this operation to a document CRDT
    pub fn apply(&self, doc: &mut DocumentCRDT) -> Result<()> {
        match self {
            CRDTOperation::AddEntity {
                entity_id,
                entity_type,
                layer,
                timestamp,
            } => {
                let entity = CADEntityCRDT::new(*entity_id, entity_type.clone(), *timestamp, *layer);
                if let Some(existing) = doc.entities.get_mut(entity_id) {
                    existing.merge(&entity)?;
                } else {
                    doc.entities.insert(*entity_id, entity);
                }
                doc.counter = doc.counter.max(timestamp.time);
                Ok(())
            }

            CRDTOperation::UpdateProperty {
                entity_id,
                property,
                value,
                timestamp,
            } => {
                if let Some(entity) = doc.entities.get_mut(entity_id) {
                    entity.update_property(property.clone(), value.clone(), *timestamp);
                    doc.counter = doc.counter.max(timestamp.time);
                }
                Ok(())
            }

            CRDTOperation::DeleteEntity {
                entity_id,
                timestamp,
            } => {
                if let Some(entity) = doc.entities.get_mut(entity_id) {
                    entity.delete(*timestamp);
                    doc.counter = doc.counter.max(timestamp.time);
                }
                Ok(())
            }

            CRDTOperation::AddLayer { layer_id, crdt_id } => {
                doc.layers.insert(*layer_id, crdt_id.clone());
                doc.counter = doc.counter.max(crdt_id.counter);
                Ok(())
            }

            CRDTOperation::RemoveLayer { layer_id, crdt_ids } => {
                doc.layers.remove(layer_id, crdt_ids);
                Ok(())
            }

            CRDTOperation::Snapshot(snapshot) => doc.apply_snapshot(snapshot.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lww_register() {
        let site1 = Uuid::new_v4();
        let site2 = Uuid::new_v4();

        let mut reg = LWWRegister::new(10, LamportTimestamp::new(1, site1));

        // Later timestamp should update
        assert!(reg.update(20, LamportTimestamp::new(2, site1)));
        assert_eq!(*reg.value(), 20);

        // Earlier timestamp should not update
        assert!(!reg.update(30, LamportTimestamp::new(1, site2)));
        assert_eq!(*reg.value(), 20);
    }

    #[test]
    fn test_or_set() {
        let mut set = ORSet::new();
        let site_id = Uuid::new_v4();

        let id1 = CRDTId::new(site_id, 1);
        let id2 = CRDTId::new(site_id, 2);

        set.insert(42, id1.clone());
        assert!(set.contains(&42));

        set.insert(42, id2.clone());

        let mut ids_to_remove = HashSet::new();
        ids_to_remove.insert(id1);
        set.remove(&42, &ids_to_remove);

        assert!(set.contains(&42)); // Still contains because id2 remains
    }

    #[test]
    fn test_document_crdt_merge() {
        let doc_id = Uuid::new_v4();
        let site1 = Uuid::new_v4();
        let site2 = Uuid::new_v4();

        let mut doc1 = DocumentCRDT::new(doc_id, site1);
        let mut doc2 = DocumentCRDT::new(doc_id, site2);

        // Add entity on doc1
        let entity_id = Uuid::new_v4();
        doc1.add_entity(entity_id, "line".to_string(), None);

        // Add different entity on doc2
        let entity_id2 = Uuid::new_v4();
        doc2.add_entity(entity_id2, "circle".to_string(), None);

        // Merge
        doc1.merge(&doc2).unwrap();

        // Both entities should exist
        assert!(doc1.entities.contains_key(&entity_id));
        assert!(doc1.entities.contains_key(&entity_id2));
    }
}
