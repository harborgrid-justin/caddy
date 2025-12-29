//! # Operational Transformation (OT)
//!
//! Implements operational transformation for real-time collaborative editing.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use thiserror::Error;

/// Errors that can occur during OT operations
#[derive(Debug, Error)]
pub enum OTError {
    #[error("Invalid operation position: {0}")]
    InvalidPosition(usize),
    #[error("Operation index out of bounds: {0}")]
    IndexOutOfBounds(usize),
    #[error("Cannot transform incompatible operations")]
    IncompatibleOperations,
    #[error("History buffer overflow")]
    HistoryOverflow,
}

/// Type of operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpType {
    /// Insert text at position
    Insert { pos: usize, text: String },
    /// Delete text at position with length
    Delete { pos: usize, len: usize },
    /// Retain (skip) characters
    Retain { count: usize },
}

/// An operation that can be applied to a document
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Operation {
    /// The operation type
    pub op_type: OpType,
    /// Version/revision number when this operation was created
    pub version: u64,
    /// Unique identifier for this operation
    pub id: uuid::Uuid,
    /// Client/user who created this operation
    pub client_id: uuid::Uuid,
    /// Timestamp when operation was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Operation {
    /// Create a new Insert operation
    pub fn insert(pos: usize, text: String, version: u64, client_id: uuid::Uuid) -> Self {
        Self {
            op_type: OpType::Insert { pos, text },
            version,
            id: uuid::Uuid::new_v4(),
            client_id,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create a new Delete operation
    pub fn delete(pos: usize, len: usize, version: u64, client_id: uuid::Uuid) -> Self {
        Self {
            op_type: OpType::Delete { pos, len },
            version,
            id: uuid::Uuid::new_v4(),
            client_id,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create a new Retain operation
    pub fn retain(count: usize, version: u64, client_id: uuid::Uuid) -> Self {
        Self {
            op_type: OpType::Retain { count },
            version,
            id: uuid::Uuid::new_v4(),
            client_id,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Get the length this operation affects
    pub fn len(&self) -> usize {
        match &self.op_type {
            OpType::Insert { text, .. } => text.len(),
            OpType::Delete { len, .. } => *len,
            OpType::Retain { count } => *count,
        }
    }

    /// Check if operation is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Apply this operation to a string
    pub fn apply(&self, text: &str) -> Result<String, OTError> {
        match &self.op_type {
            OpType::Insert { pos, text: ins } => {
                if *pos > text.len() {
                    return Err(OTError::InvalidPosition(*pos));
                }
                let mut result = String::with_capacity(text.len() + ins.len());
                result.push_str(&text[..*pos]);
                result.push_str(ins);
                result.push_str(&text[*pos..]);
                Ok(result)
            }
            OpType::Delete { pos, len } => {
                if *pos + *len > text.len() {
                    return Err(OTError::InvalidPosition(*pos));
                }
                let mut result = String::with_capacity(text.len());
                result.push_str(&text[..*pos]);
                result.push_str(&text[*pos + *len..]);
                Ok(result)
            }
            OpType::Retain { .. } => Ok(text.to_string()),
        }
    }
}

/// Transform two operations against each other
/// Returns (transformed_a, transformed_b)
pub fn transform(
    op_a: &Operation,
    op_b: &Operation,
) -> Result<(Operation, Operation), OTError> {
    match (&op_a.op_type, &op_b.op_type) {
        // Insert vs Insert
        (OpType::Insert { pos: pos_a, text: text_a }, OpType::Insert { pos: pos_b, text: text_b }) => {
            let new_a = if *pos_a < *pos_b || (*pos_a == *pos_b && op_a.client_id < op_b.client_id) {
                Operation::insert(*pos_a, text_a.clone(), op_a.version, op_a.client_id)
            } else {
                Operation::insert(pos_a + text_b.len(), text_a.clone(), op_a.version, op_a.client_id)
            };

            let new_b = if *pos_b < *pos_a || (*pos_b == *pos_a && op_b.client_id < op_a.client_id) {
                Operation::insert(*pos_b, text_b.clone(), op_b.version, op_b.client_id)
            } else {
                Operation::insert(pos_b + text_a.len(), text_b.clone(), op_b.version, op_b.client_id)
            };

            Ok((new_a, new_b))
        }

        // Insert vs Delete
        (OpType::Insert { pos: pos_a, text: text_a }, OpType::Delete { pos: pos_b, len: len_b }) => {
            let new_a = if *pos_a <= *pos_b {
                Operation::insert(*pos_a, text_a.clone(), op_a.version, op_a.client_id)
            } else if *pos_a > *pos_b + *len_b {
                Operation::insert(pos_a - *len_b, text_a.clone(), op_a.version, op_a.client_id)
            } else {
                Operation::insert(*pos_b, text_a.clone(), op_a.version, op_a.client_id)
            };

            let new_b = if *pos_b >= *pos_a {
                Operation::delete(pos_b + text_a.len(), *len_b, op_b.version, op_b.client_id)
            } else {
                Operation::delete(*pos_b, *len_b, op_b.version, op_b.client_id)
            };

            Ok((new_a, new_b))
        }

        // Delete vs Insert
        (OpType::Delete { pos: pos_a, len: len_a }, OpType::Insert { pos: pos_b, text: text_b }) => {
            let new_a = if *pos_a >= *pos_b {
                Operation::delete(pos_a + text_b.len(), *len_a, op_a.version, op_a.client_id)
            } else {
                Operation::delete(*pos_a, *len_a, op_a.version, op_a.client_id)
            };

            let new_b = if *pos_b <= *pos_a {
                Operation::insert(*pos_b, text_b.clone(), op_b.version, op_b.client_id)
            } else if *pos_b > *pos_a + *len_a {
                Operation::insert(pos_b - *len_a, text_b.clone(), op_b.version, op_b.client_id)
            } else {
                Operation::insert(*pos_a, text_b.clone(), op_b.version, op_b.client_id)
            };

            Ok((new_a, new_b))
        }

        // Delete vs Delete
        (OpType::Delete { pos: pos_a, len: len_a }, OpType::Delete { pos: pos_b, len: len_b }) => {
            let new_a = if *pos_a >= *pos_b + *len_b {
                Operation::delete(pos_a - *len_b, *len_a, op_a.version, op_a.client_id)
            } else if *pos_a + *len_a <= *pos_b {
                Operation::delete(*pos_a, *len_a, op_a.version, op_a.client_id)
            } else {
                // Overlapping deletes - adjust based on overlap
                let new_pos = (*pos_a).min(*pos_b);
                let overlap_start = (*pos_a).max(*pos_b);
                let overlap_end = (*pos_a + *len_a).min(*pos_b + *len_b);
                let overlap = if overlap_end > overlap_start {
                    overlap_end - overlap_start
                } else {
                    0
                };
                let new_len = if *len_a > overlap { *len_a - overlap } else { 0 };

                Operation::delete(new_pos, new_len, op_a.version, op_a.client_id)
            };

            let new_b = if *pos_b >= *pos_a + *len_a {
                Operation::delete(pos_b - *len_a, *len_b, op_b.version, op_b.client_id)
            } else if *pos_b + *len_b <= *pos_a {
                Operation::delete(*pos_b, *len_b, op_b.version, op_b.client_id)
            } else {
                let new_pos = (*pos_a).min(*pos_b);
                let overlap_start = (*pos_a).max(*pos_b);
                let overlap_end = (*pos_a + *len_a).min(*pos_b + *len_b);
                let overlap = if overlap_end > overlap_start {
                    overlap_end - overlap_start
                } else {
                    0
                };
                let new_len = if *len_b > overlap { *len_b - overlap } else { 0 };

                Operation::delete(new_pos, new_len, op_b.version, op_b.client_id)
            };

            Ok((new_a, new_b))
        }

        // Retain operations don't need transformation
        _ => Ok((op_a.clone(), op_b.clone())),
    }
}

/// Compose two operations into a single operation
pub fn compose(op1: &Operation, op2: &Operation) -> Result<Vec<Operation>, OTError> {
    let mut result = Vec::new();

    match (&op1.op_type, &op2.op_type) {
        // Insert followed by Insert
        (OpType::Insert { pos: pos1, text: text1 }, OpType::Insert { pos: pos2, text: text2 }) => {
            if *pos2 >= *pos1 && *pos2 <= *pos1 + text1.len() {
                // Insert into the first insert
                let offset = *pos2 - *pos1;
                let mut new_text = text1.clone();
                new_text.insert_str(offset, text2);
                result.push(Operation::insert(*pos1, new_text, op2.version, op2.client_id));
            } else {
                // Separate inserts
                result.push(op1.clone());
                result.push(op2.clone());
            }
        }

        // Insert followed by Delete
        (OpType::Insert { pos: pos1, text: text1 }, OpType::Delete { pos: pos2, len: len2 }) => {
            if *pos2 >= *pos1 && *pos2 < *pos1 + text1.len() {
                // Delete affects the insert
                let offset = *pos2 - *pos1;
                let end = (offset + *len2).min(text1.len());
                let mut new_text = text1.clone();
                new_text.replace_range(offset..end, "");

                if !new_text.is_empty() {
                    result.push(Operation::insert(*pos1, new_text, op2.version, op2.client_id));
                }

                // If delete extends beyond insert
                if offset + *len2 > text1.len() {
                    let remaining = offset + *len2 - text1.len();
                    result.push(Operation::delete(*pos1, remaining, op2.version, op2.client_id));
                }
            } else {
                result.push(op1.clone());
                result.push(op2.clone());
            }
        }

        // Other compositions
        _ => {
            result.push(op1.clone());
            result.push(op2.clone());
        }
    }

    Ok(result)
}

/// History buffer for tracking operations
#[derive(Debug, Clone)]
pub struct HistoryBuffer {
    operations: VecDeque<Operation>,
    max_size: usize,
    current_version: u64,
}

impl HistoryBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            operations: VecDeque::new(),
            max_size,
            current_version: 0,
        }
    }

    /// Add an operation to the history
    pub fn push(&mut self, op: Operation) -> Result<(), OTError> {
        if self.operations.len() >= self.max_size {
            self.operations.pop_front();
        }

        self.current_version = self.current_version.max(op.version + 1);
        self.operations.push_back(op);
        Ok(())
    }

    /// Get the current version
    pub fn version(&self) -> u64 {
        self.current_version
    }

    /// Get all operations after a specific version
    pub fn get_since(&self, version: u64) -> Vec<Operation> {
        self.operations
            .iter()
            .filter(|op| op.version >= version)
            .cloned()
            .collect()
    }

    /// Get operation by ID
    pub fn get_by_id(&self, id: uuid::Uuid) -> Option<&Operation> {
        self.operations.iter().find(|op| op.id == id)
    }

    /// Clear all operations
    pub fn clear(&mut self) {
        self.operations.clear();
    }

    /// Get the number of operations in the buffer
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_operation() {
        let client = uuid::Uuid::new_v4();
        let op = Operation::insert(0, "hello".to_string(), 1, client);

        let result = op.apply("").unwrap();
        assert_eq!(result, "hello");

        let result = op.apply("world").unwrap();
        assert_eq!(result, "helloworld");
    }

    #[test]
    fn test_delete_operation() {
        let client = uuid::Uuid::new_v4();
        let op = Operation::delete(0, 5, 1, client);

        let result = op.apply("hello world").unwrap();
        assert_eq!(result, " world");
    }

    #[test]
    fn test_transform_insert_insert() {
        let client_a = uuid::Uuid::new_v4();
        let client_b = uuid::Uuid::new_v4();

        let op_a = Operation::insert(0, "a".to_string(), 1, client_a);
        let op_b = Operation::insert(0, "b".to_string(), 1, client_b);

        let (transformed_a, transformed_b) = transform(&op_a, &op_b).unwrap();

        // The transformation should ensure both operations can be applied
        let text = "";
        let text = op_a.apply(text).unwrap();
        let text = transformed_b.apply(&text).unwrap();
        let result1 = text;

        let text = "";
        let text = op_b.apply(text).unwrap();
        let text = transformed_a.apply(&text).unwrap();
        let result2 = text;

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_transform_insert_delete() {
        let client_a = uuid::Uuid::new_v4();
        let client_b = uuid::Uuid::new_v4();

        let op_a = Operation::insert(5, "X".to_string(), 1, client_a);
        let op_b = Operation::delete(0, 3, 1, client_b);

        let (transformed_a, transformed_b) = transform(&op_a, &op_b).unwrap();

        // Verify convergence
        let text = "hello world";
        let text1 = op_a.apply(text).unwrap();
        let text1 = transformed_b.apply(&text1).unwrap();

        let text2 = op_b.apply(text).unwrap();
        let text2 = transformed_a.apply(&text2).unwrap();

        assert_eq!(text1, text2);
    }

    #[test]
    fn test_history_buffer() {
        let mut buffer = HistoryBuffer::new(10);
        let client = uuid::Uuid::new_v4();

        let op1 = Operation::insert(0, "a".to_string(), 0, client);
        let op2 = Operation::insert(1, "b".to_string(), 1, client);

        buffer.push(op1.clone()).unwrap();
        buffer.push(op2.clone()).unwrap();

        assert_eq!(buffer.len(), 2);

        let ops = buffer.get_since(1);
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].id, op2.id);
    }

    #[test]
    fn test_compose_operations() {
        let client = uuid::Uuid::new_v4();

        let op1 = Operation::insert(0, "hello".to_string(), 0, client);
        let op2 = Operation::insert(5, " world".to_string(), 1, client);

        let composed = compose(&op1, &op2).unwrap();

        // Apply composed operations
        let mut text = String::new();
        for op in composed {
            text = op.apply(&text).unwrap();
        }

        assert!(text.contains("hello") && text.contains("world"));
    }
}
