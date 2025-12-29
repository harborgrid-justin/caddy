//! Operational Transformation and CRDTs
//!
//! This module implements operational transformation (OT) algorithms and Conflict-free
//! Replicated Data Types (CRDTs) for maintaining consistency in collaborative editing.

use super::{CollaborationError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Operation identifier
pub type OperationId = Uuid;

/// Vector clock for tracking causal relationships
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorClock {
    /// Map of user ID to sequence number
    clock: HashMap<Uuid, u64>,
}

impl VectorClock {
    /// Create a new empty vector clock
    pub fn new() -> Self {
        Self {
            clock: HashMap::new(),
        }
    }

    /// Increment the clock for a user
    pub fn increment(&mut self, user_id: Uuid) -> u64 {
        let counter = self.clock.entry(user_id).or_insert(0);
        *counter += 1;
        *counter
    }

    /// Get the sequence number for a user
    pub fn get(&self, user_id: &Uuid) -> u64 {
        self.clock.get(user_id).copied().unwrap_or(0)
    }

    /// Merge with another vector clock
    pub fn merge(&mut self, other: &VectorClock) {
        for (user_id, seq) in &other.clock {
            let current = self.clock.entry(*user_id).or_insert(0);
            *current = (*current).max(*seq);
        }
    }

    /// Check if this clock happens before another
    pub fn happens_before(&self, other: &VectorClock) -> bool {
        let mut is_less = false;

        for (user_id, seq) in &self.clock {
            let other_seq = other.get(user_id);
            if *seq > other_seq {
                return false;
            } else if *seq < other_seq {
                is_less = true;
            }
        }

        for (user_id, _) in &other.clock {
            if !self.clock.contains_key(user_id) {
                is_less = true;
            }
        }

        is_less
    }

    /// Check if this clock is concurrent with another
    pub fn is_concurrent(&self, other: &VectorClock) -> bool {
        !self.happens_before(other) && !other.happens_before(self)
    }
}

impl Default for VectorClock {
    fn default() -> Self {
        Self::new()
    }
}

/// CAD operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    /// Create a new entity
    CreateEntity {
        entity_id: Uuid,
        entity_type: String,
        properties: HashMap<String, serde_json::Value>,
        layer_id: Option<Uuid>,
    },

    /// Delete an entity
    DeleteEntity {
        entity_id: Uuid,
    },

    /// Update entity properties
    UpdateEntity {
        entity_id: Uuid,
        property: String,
        old_value: serde_json::Value,
        new_value: serde_json::Value,
    },

    /// Transform entity (move, rotate, scale)
    TransformEntity {
        entity_id: Uuid,
        transform_type: TransformType,
        parameters: Vec<f64>,
    },

    /// Create a layer
    CreateLayer {
        layer_id: Uuid,
        name: String,
        properties: HashMap<String, serde_json::Value>,
    },

    /// Delete a layer
    DeleteLayer {
        layer_id: Uuid,
    },

    /// Move entity to layer
    MoveToLayer {
        entity_id: Uuid,
        old_layer_id: Option<Uuid>,
        new_layer_id: Option<Uuid>,
    },

    /// Apply constraint
    ApplyConstraint {
        constraint_id: Uuid,
        constraint_type: String,
        entities: Vec<Uuid>,
        parameters: HashMap<String, f64>,
    },

    /// Remove constraint
    RemoveConstraint {
        constraint_id: Uuid,
    },

    /// Batch operation
    Batch {
        operations: Vec<Operation>,
    },
}

/// Transform types for entities
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TransformType {
    Translate,
    Rotate,
    Scale,
    Mirror,
    Matrix,
}

/// Operation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationMetadata {
    /// Unique operation ID
    pub id: OperationId,
    /// User who performed the operation
    pub user_id: Uuid,
    /// Timestamp when operation was created
    pub timestamp: DateTime<Utc>,
    /// Vector clock at operation creation
    pub vector_clock: VectorClock,
    /// Session ID
    pub session_id: Uuid,
    /// Parent operation ID (for dependent operations)
    pub parent_id: Option<OperationId>,
}

/// Complete operation with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationWithMetadata {
    pub operation: Operation,
    pub metadata: OperationMetadata,
}

impl OperationWithMetadata {
    /// Create a new operation with metadata
    pub fn new(
        operation: Operation,
        user_id: Uuid,
        session_id: Uuid,
        vector_clock: VectorClock,
    ) -> Self {
        Self {
            operation,
            metadata: OperationMetadata {
                id: Uuid::new_v4(),
                user_id,
                timestamp: Utc::now(),
                vector_clock,
                session_id,
                parent_id: None,
            },
        }
    }

    /// Set parent operation ID
    pub fn with_parent(mut self, parent_id: OperationId) -> Self {
        self.metadata.parent_id = Some(parent_id);
        self
    }
}

/// Result of transforming two operations
#[derive(Debug, Clone)]
pub struct TransformResult {
    /// Transformed version of the first operation
    pub op1_prime: OperationWithMetadata,
    /// Transformed version of the second operation
    pub op2_prime: OperationWithMetadata,
}

/// Operational transformation engine
pub struct OperationTransform;

impl OperationTransform {
    /// Transform two concurrent operations
    ///
    /// Given operations op1 and op2 that were created concurrently,
    /// compute op1' and op2' such that:
    /// apply(op1, apply(op2', _state)) = apply(op2, apply(op1', state))
    pub fn transform(
        op1: &OperationWithMetadata,
        op2: &OperationWithMetadata,
    ) -> Result<TransformResult> {
        // Check if operations are concurrent
        if !op1.metadata.vector_clock.is_concurrent(&op2.metadata.vector_clock) {
            return Err(CollaborationError::Operation(
                "Operations are not concurrent".to_string(),
            ));
        }

        let (op1_prime, op2_prime) = match (&op1.operation, &op2.operation) {
            // Both operations create entities - no conflict
            (Operation::CreateEntity { .. }, Operation::CreateEntity { .. }) => {
                (op1.clone(), op2.clone())
            }

            // Both operations delete the same entity - keep the one with earlier timestamp
            (Operation::DeleteEntity { entity_id: id1 }, Operation::DeleteEntity { entity_id: id2 }) => {
                if id1 == id2 {
                    // Same entity - delete wins based on timestamp
                    if op1.metadata.timestamp < op2.metadata.timestamp {
                        (op1.clone(), Self::make_noop(op2))
                    } else {
                        (Self::make_noop(op1), op2.clone())
                    }
                } else {
                    // Different entities - no conflict
                    (op1.clone(), op2.clone())
                }
            }

            // Delete vs Update on same entity - delete wins
            (Operation::DeleteEntity { entity_id: id1 }, Operation::UpdateEntity { entity_id: id2, .. }) => {
                if id1 == id2 {
                    (op1.clone(), Self::make_noop(op2))
                } else {
                    (op1.clone(), op2.clone())
                }
            }

            (Operation::UpdateEntity { entity_id: id1, .. }, Operation::DeleteEntity { entity_id: id2 }) => {
                if id1 == id2 {
                    (Self::make_noop(op1), op2.clone())
                } else {
                    (op1.clone(), op2.clone())
                }
            }

            // Both update same property of same entity - last-write-wins based on timestamp
            (
                Operation::UpdateEntity { entity_id: id1, property: prop1, .. },
                Operation::UpdateEntity { entity_id: id2, property: prop2, new_value: val2, .. },
            ) => {
                if id1 == id2 && prop1 == prop2 {
                    if op1.metadata.timestamp < op2.metadata.timestamp {
                        // op2 wins - adjust op2's old_value to be op1's new_value
                        let mut op2_prime = op2.clone();
                        if let Operation::UpdateEntity { old_value, .. } = &mut op2_prime.operation {
                            if let Operation::UpdateEntity { new_value: val1, .. } = &op1.operation {
                                *old_value = val1.clone();
                            }
                        }
                        (Self::make_noop(op1), op2_prime)
                    } else {
                        // op1 wins - adjust op1's old_value
                        let mut op1_prime = op1.clone();
                        if let Operation::UpdateEntity { old_value, .. } = &mut op1_prime.operation {
                            if let Operation::UpdateEntity { new_value: val2, .. } = &op2.operation {
                                *old_value = val2.clone();
                            }
                        }
                        (op1_prime, Self::make_noop(op2))
                    }
                } else {
                    // Different properties or entities - no conflict
                    (op1.clone(), op2.clone())
                }
            }

            // Transform operations - compose if same entity
            (
                Operation::TransformEntity { entity_id: id1, .. },
                Operation::TransformEntity { entity_id: id2, .. },
            ) => {
                if id1 == id2 {
                    // Transforms can be composed - for now keep both
                    (op1.clone(), op2.clone())
                } else {
                    (op1.clone(), op2.clone())
                }
            }

            // Layer operations
            (Operation::DeleteLayer { layer_id: id1 }, Operation::MoveToLayer { new_layer_id: Some(id2), .. }) => {
                if id1 == id2 {
                    // Can't move to deleted layer - cancel move
                    (op1.clone(), Self::make_noop(op2))
                } else {
                    (op1.clone(), op2.clone())
                }
            }

            // Default - no transformation needed
            _ => (op1.clone(), op2.clone()),
        };

        Ok(TransformResult {
            op1_prime,
            op2_prime,
        })
    }

    /// Create a no-op version of an operation (for cancelled operations)
    fn make_noop(op: &OperationWithMetadata) -> OperationWithMetadata {
        let mut noop = op.clone();
        noop.operation = Operation::Batch {
            operations: Vec::new(),
        };
        noop
    }
}

/// Operation composition
pub struct OperationComposer;

impl OperationComposer {
    /// Compose two sequential operations into one
    pub fn compose(
        op1: &Operation,
        op2: &Operation,
    ) -> Result<Option<Operation>> {
        match (op1, op2) {
            // Update followed by another update on same property
            (
                Operation::UpdateEntity {
                    entity_id: id1,
                    property: prop1,
                    old_value,
                    ..
                },
                Operation::UpdateEntity {
                    entity_id: id2,
                    property: prop2,
                    new_value,
                    ..
                },
            ) if id1 == id2 && prop1 == prop2 => {
                Ok(Some(Operation::UpdateEntity {
                    entity_id: *id1,
                    property: prop1.clone(),
                    old_value: old_value.clone(),
                    new_value: new_value.clone(),
                }))
            }

            // Create followed by delete - cancel out
            (
                Operation::CreateEntity { entity_id: id1, .. },
                Operation::DeleteEntity { entity_id: id2 },
            ) if id1 == id2 => Ok(None),

            // Create followed by update - merge into create
            (
                Operation::CreateEntity {
                    entity_id: id1,
                    entity_type,
                    properties,
                    layer_id,
                },
                Operation::UpdateEntity {
                    entity_id: id2,
                    property,
                    new_value,
                    ..
                },
            ) if id1 == id2 => {
                let mut new_props = properties.clone();
                new_props.insert(property.clone(), new_value.clone());
                Ok(Some(Operation::CreateEntity {
                    entity_id: *id1,
                    entity_type: entity_type.clone(),
                    properties: new_props,
                    layer_id: *layer_id,
                }))
            }

            // Can't compose - return None
            _ => Ok(None),
        }
    }
}

/// Operation inversion for undo functionality
pub struct OperationInverter;

impl OperationInverter {
    /// Invert an operation to create its undo
    pub fn invert(op: &Operation) -> Result<Operation> {
        match op {
            Operation::CreateEntity { entity_id, .. } => {
                Ok(Operation::DeleteEntity {
                    entity_id: *entity_id,
                })
            }

            Operation::DeleteEntity { entity_id } => {
                // Need to store entity data to properly invert
                // For now, return an error
                Err(CollaborationError::Operation(
                    "Cannot invert delete without entity data".to_string(),
                ))
            }

            Operation::UpdateEntity {
                entity_id,
                property,
                old_value,
                new_value,
            } => Ok(Operation::UpdateEntity {
                entity_id: *entity_id,
                property: property.clone(),
                old_value: new_value.clone(),
                new_value: old_value.clone(),
            }),

            Operation::TransformEntity {
                entity_id,
                transform_type,
                parameters,
            } => {
                // Invert transform based on type
                let inverted_params = match transform_type {
                    TransformType::Translate => {
                        // Negate translation
                        parameters.iter().map(|x| -x).collect()
                    }
                    TransformType::Rotate => {
                        // Negate rotation
                        parameters.iter().map(|x| -x).collect()
                    }
                    TransformType::Scale => {
                        // Reciprocal of scale
                        parameters.iter().map(|x| 1.0 / x).collect()
                    }
                    _ => parameters.clone(),
                };

                Ok(Operation::TransformEntity {
                    entity_id: *entity_id,
                    transform_type: *transform_type,
                    parameters: inverted_params,
                })
            }

            Operation::MoveToLayer {
                entity_id,
                old_layer_id,
                new_layer_id,
            } => Ok(Operation::MoveToLayer {
                entity_id: *entity_id,
                old_layer_id: *new_layer_id,
                new_layer_id: *old_layer_id,
            }),

            Operation::ApplyConstraint { constraint_id, .. } => {
                Ok(Operation::RemoveConstraint {
                    constraint_id: *constraint_id,
                })
            }

            Operation::RemoveConstraint { constraint_id } => {
                Err(CollaborationError::Operation(
                    "Cannot invert constraint removal without constraint data".to_string(),
                ))
            }

            Operation::Batch { operations } => {
                let mut inverted = Vec::new();
                for op in operations.iter().rev() {
                    inverted.push(Self::invert(op)?);
                }
                Ok(Operation::Batch {
                    operations: inverted,
                })
            }

            _ => Err(CollaborationError::Operation(
                "Operation inversion not implemented".to_string(),
            )),
        }
    }
}

/// CRDT operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CRDTOperation {
    /// G-Counter (Grow-only counter)
    GCounterIncrement { counter_id: Uuid, user_id: Uuid, value: u64 },

    /// PN-Counter (Positive-Negative counter)
    PNCounterIncrement { counter_id: Uuid, user_id: Uuid, value: i64 },

    /// LWW-Element-Set (Last-Write-Wins Element Set)
    LWWSetAdd { set_id: Uuid, element: String, timestamp: DateTime<Utc> },
    LWWSetRemove { set_id: Uuid, element: String, timestamp: DateTime<Utc> },

    /// OR-Set (Observed-Remove Set)
    ORSetAdd { set_id: Uuid, element: String, unique_tag: Uuid },
    ORSetRemove { set_id: Uuid, element: String, tags: Vec<Uuid> },
}

/// CRDT state manager
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CRDTState {
    /// G-Counter states
    g_counters: HashMap<Uuid, HashMap<Uuid, u64>>,
    /// PN-Counter states
    pn_counters: HashMap<Uuid, (HashMap<Uuid, u64>, HashMap<Uuid, u64>)>,
    /// LWW-Set states
    lww_sets: HashMap<Uuid, HashMap<String, (DateTime<Utc>, bool)>>,
    /// OR-Set states
    or_sets: HashMap<Uuid, HashMap<String, Vec<Uuid>>>,
}

impl CRDTState {
    /// Create a new CRDT state
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply a CRDT operation
    pub fn apply(&mut self, op: &CRDTOperation) {
        match op {
            CRDTOperation::GCounterIncrement { counter_id, user_id, value } => {
                let counter = self.g_counters.entry(*counter_id).or_insert_with(HashMap::new);
                let current = counter.entry(*user_id).or_insert(0);
                *current = (*current).max(*value);
            }

            CRDTOperation::PNCounterIncrement { counter_id, user_id, value } => {
                let (pos, neg) = self.pn_counters.entry(*counter_id).or_insert_with(|| (HashMap::new(), HashMap::new()));
                if *value >= 0 {
                    let current = pos.entry(*user_id).or_insert(0);
                    *current = (*current).max(*value as u64);
                } else {
                    let current = neg.entry(*user_id).or_insert(0);
                    *current = (*current).max(value.abs() as u64);
                }
            }

            CRDTOperation::LWWSetAdd { set_id, element, timestamp } => {
                let set = self.lww_sets.entry(*set_id).or_insert_with(HashMap::new);
                let current = set.entry(element.clone()).or_insert((*timestamp, false));
                if timestamp >= &current.0 {
                    *current = (*timestamp, true);
                }
            }

            CRDTOperation::LWWSetRemove { set_id, element, timestamp } => {
                let set = self.lww_sets.entry(*set_id).or_insert_with(HashMap::new);
                let current = set.entry(element.clone()).or_insert((*timestamp, true));
                if timestamp >= &current.0 {
                    *current = (*timestamp, false);
                }
            }

            CRDTOperation::ORSetAdd { set_id, element, unique_tag } => {
                let set = self.or_sets.entry(*set_id).or_insert_with(HashMap::new);
                let tags = set.entry(element.clone()).or_insert_with(Vec::new);
                if !tags.contains(unique_tag) {
                    tags.push(*unique_tag);
                }
            }

            CRDTOperation::ORSetRemove { set_id, element, tags } => {
                if let Some(set) = self.or_sets.get_mut(set_id) {
                    if let Some(element_tags) = set.get_mut(element) {
                        element_tags.retain(|tag| !tags.contains(tag));
                        if element_tags.is_empty() {
                            set.remove(element);
                        }
                    }
                }
            }
        }
    }

    /// Merge with another CRDT state
    pub fn merge(&mut self, other: &CRDTState) {
        // Merge G-Counters
        for (counter_id, other_counter) in &other.g_counters {
            let counter = self.g_counters.entry(*counter_id).or_insert_with(HashMap::new);
            for (user_id, value) in other_counter {
                let current = counter.entry(*user_id).or_insert(0);
                *current = (*current).max(*value);
            }
        }

        // Merge PN-Counters
        for (counter_id, (other_pos, other_neg)) in &other.pn_counters {
            let (pos, neg) = self.pn_counters.entry(*counter_id).or_insert_with(|| (HashMap::new(), HashMap::new()));
            for (user_id, value) in other_pos {
                let current = pos.entry(*user_id).or_insert(0);
                *current = (*current).max(*value);
            }
            for (user_id, value) in other_neg {
                let current = neg.entry(*user_id).or_insert(0);
                *current = (*current).max(*value);
            }
        }

        // Merge LWW-Sets
        for (set_id, other_set) in &other.lww_sets {
            let set = self.lww_sets.entry(*set_id).or_insert_with(HashMap::new);
            for (element, (timestamp, present)) in other_set {
                let current = set.entry(element.clone()).or_insert((*timestamp, *present));
                if timestamp >= &current.0 {
                    *current = (*timestamp, *present);
                }
            }
        }

        // Merge OR-Sets
        for (set_id, other_set) in &other.or_sets {
            let set = self.or_sets.entry(*set_id).or_insert_with(HashMap::new);
            for (element, other_tags) in other_set {
                let tags = set.entry(element.clone()).or_insert_with(Vec::new);
                for tag in other_tags {
                    if !tags.contains(tag) {
                        tags.push(*tag);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_clock() {
        let mut clock1 = VectorClock::new();
        let mut clock2 = VectorClock::new();

        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();

        clock1.increment(user1);
        clock2.increment(user2);

        assert!(clock1.is_concurrent(&clock2));
        assert!(!clock1.happens_before(&clock2));
    }

    #[test]
    fn test_operation_composition() {
        let entity_id = Uuid::new_v4();

        let op1 = Operation::UpdateEntity {
            entity_id,
            property: "width".to_string(),
            old_value: serde_json::json!(10.0),
            new_value: serde_json::json!(20.0),
        };

        let op2 = Operation::UpdateEntity {
            entity_id,
            property: "width".to_string(),
            old_value: serde_json::json!(20.0),
            new_value: serde_json::json!(30.0),
        };

        let composed = OperationComposer::compose(&op1, &op2).unwrap();
        assert!(composed.is_some());

        if let Some(Operation::UpdateEntity { new_value, .. }) = composed {
            assert_eq!(new_value, serde_json::json!(30.0));
        }
    }

    #[test]
    fn test_operation_inversion() {
        let entity_id = Uuid::new_v4();

        let op = Operation::UpdateEntity {
            entity_id,
            property: "width".to_string(),
            old_value: serde_json::json!(10.0),
            new_value: serde_json::json!(20.0),
        };

        let inverted = OperationInverter::invert(&op).unwrap();

        if let Operation::UpdateEntity { old_value, new_value, .. } = inverted {
            assert_eq!(old_value, serde_json::json!(20.0));
            assert_eq!(new_value, serde_json::json!(10.0));
        } else {
            panic!("Expected UpdateEntity operation");
        }
    }
}
