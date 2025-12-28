//! # Zero-Knowledge Proofs
//!
//! This module provides basic zero-knowledge proof primitives and constructions.
//!
//! ## Supported Constructions
//!
//! - **Commitment Schemes**: Pedersen commitments, hash-based commitments
//! - **Range Proofs**: Basic range proofs (value is within a range)
//! - **Merkle Trees**: Merkle tree construction and proof verification
//! - **Proof Verification**: Verify zero-knowledge proofs
//!
//! ## Use Cases
//!
//! - Prove knowledge without revealing the secret
//! - Prove a value is within a range without revealing the value
//! - Prove membership in a set (Merkle tree)
//! - Commitment schemes for secure protocols
//!
//! ## Security Considerations
//!
//! - These are basic implementations for demonstration
//! - Production systems should use specialized ZKP libraries
//! - Always use cryptographically secure randomness
//! - Verify all proofs before accepting them

use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use zeroize::Zeroize;

/// ZKP-specific errors
#[derive(Error, Debug)]
pub enum ZkpError {
    /// Invalid proof
    #[error("Invalid proof")]
    InvalidProof,

    /// Invalid commitment
    #[error("Invalid commitment: {0}")]
    InvalidCommitment(String),

    /// Invalid range
    #[error("Invalid range: {0}")]
    InvalidRange(String),

    /// Merkle proof verification failed
    #[error("Merkle proof verification failed")]
    MerkleProofFailed,

    /// Tree construction error
    #[error("Tree construction error: {0}")]
    TreeConstructionError(String),
}

pub type ZkpResult<T> = Result<T, ZkpError>;

/// Hash-based commitment
///
/// Commitment = Hash(value || nonce)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashCommitment {
    /// The commitment value (hash)
    pub commitment: Vec<u8>,
}

impl HashCommitment {
    /// Create a commitment to a value
    ///
    /// # Arguments
    ///
    /// * `value` - The value to commit to
    /// * `nonce` - Random nonce for hiding
    ///
    /// # Returns
    ///
    /// (commitment, nonce) - The commitment and nonce (save nonce for opening)
    pub fn commit(value: &[u8]) -> (Self, Vec<u8>) {
        use rand::RngCore;

        let mut nonce = vec![0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut nonce);

        let mut hasher = Sha256::new();
        hasher.update(value);
        hasher.update(&nonce);
        let commitment = hasher.finalize().to_vec();

        (Self { commitment }, nonce)
    }

    /// Verify (open) a commitment
    ///
    /// # Arguments
    ///
    /// * `value` - The claimed value
    /// * `nonce` - The nonce used in commitment
    pub fn verify(&self, value: &[u8], nonce: &[u8]) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(value);
        hasher.update(nonce);
        let computed = hasher.finalize();

        computed.as_slice() == self.commitment.as_slice()
    }
}

/// Range proof (simplified)
///
/// Proves that a value is within a specified range without revealing the value.
/// This is a basic implementation for demonstration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeProof {
    /// Commitment to the value
    pub commitment: Vec<u8>,
    /// Proof data (simplified)
    pub proof_data: Vec<u8>,
}

impl RangeProof {
    /// Create a proof that value is in range [min, max]
    ///
    /// Note: This is a simplified implementation for demonstration.
    /// Production systems should use Bulletproofs or similar.
    pub fn prove(value: u64, min: u64, max: u64) -> ZkpResult<(Self, Vec<u8>)> {
        if value < min || value > max {
            return Err(ZkpError::InvalidRange(
                format!("Value {} not in range [{}, {}]", value, min, max)
            ));
        }

        // Create commitment
        let value_bytes = value.to_le_bytes();
        let (commitment, nonce) = HashCommitment::commit(&value_bytes);

        // Simplified proof: hash(value || min || max || nonce)
        let mut hasher = Sha256::new();
        hasher.update(&value_bytes);
        hasher.update(&min.to_le_bytes());
        hasher.update(&max.to_le_bytes());
        hasher.update(&nonce);
        let proof_data = hasher.finalize().to_vec();

        Ok((
            Self {
                commitment: commitment.commitment,
                proof_data,
            },
            nonce,
        ))
    }

    /// Verify a range proof
    ///
    /// Note: This is simplified. Real range proofs don't reveal the value.
    pub fn verify(&self, value: u64, min: u64, max: u64, nonce: &[u8]) -> bool {
        if value < min || value > max {
            return false;
        }

        // Verify commitment
        let value_bytes = value.to_le_bytes();
        let commitment = HashCommitment {
            commitment: self.commitment.clone(),
        };

        if !commitment.verify(&value_bytes, nonce) {
            return false;
        }

        // Verify proof
        let mut hasher = Sha256::new();
        hasher.update(&value_bytes);
        hasher.update(&min.to_le_bytes());
        hasher.update(&max.to_le_bytes());
        hasher.update(nonce);
        let expected_proof = hasher.finalize();

        expected_proof.as_slice() == self.proof_data.as_slice()
    }
}

/// Merkle tree node
#[derive(Debug, Clone)]
enum MerkleNode {
    Leaf { hash: Vec<u8> },
    Internal { hash: Vec<u8>, left: Box<MerkleNode>, right: Box<MerkleNode> },
}

impl MerkleNode {
    fn hash(&self) -> &[u8] {
        match self {
            MerkleNode::Leaf { hash } => hash,
            MerkleNode::Internal { hash, .. } => hash,
        }
    }
}

/// Merkle tree for proving set membership
#[derive(Debug, Clone)]
pub struct MerkleTree {
    root: Option<MerkleNode>,
    leaves: Vec<Vec<u8>>,
}

impl MerkleTree {
    /// Create a new Merkle tree from a list of values
    ///
    /// # Arguments
    ///
    /// * `values` - List of values to include in the tree
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use caddy::enterprise::crypto::zkp::MerkleTree;
    ///
    /// let values = vec![b"value1".to_vec(), b"value2".to_vec(), b"value3".to_vec()];
    /// let tree = MerkleTree::new(&values)?;
    /// ```
    pub fn new(values: &[Vec<u8>]) -> ZkpResult<Self> {
        if values.is_empty() {
            return Err(ZkpError::TreeConstructionError("Cannot create tree from empty values".to_string()));
        }

        let leaves: Vec<Vec<u8>> = values.iter().map(|v| Self::hash_leaf(v)).collect();
        let root = Self::build_tree(&leaves)?;

        Ok(Self {
            root: Some(root),
            leaves: values.to_vec(),
        })
    }

    /// Get the Merkle root hash
    pub fn root_hash(&self) -> Option<Vec<u8>> {
        self.root.as_ref().map(|node| node.hash().to_vec())
    }

    /// Generate a Merkle proof for a value
    ///
    /// # Arguments
    ///
    /// * `value` - The value to prove membership for
    ///
    /// # Returns
    ///
    /// A proof of membership (list of sibling hashes)
    pub fn prove(&self, value: &[u8]) -> ZkpResult<MerkleProof> {
        let leaf_hash = Self::hash_leaf(value);

        // Find the index of this leaf
        let index = self.leaves.iter().position(|v| {
            Self::hash_leaf(v) == leaf_hash
        }).ok_or(ZkpError::InvalidProof)?;

        let leaf_hashes: Vec<Vec<u8>> = self.leaves.iter().map(|v| Self::hash_leaf(v)).collect();
        let siblings = Self::generate_proof_siblings(&leaf_hashes, index);

        Ok(MerkleProof {
            value: value.to_vec(),
            index,
            siblings,
        })
    }

    /// Verify a Merkle proof
    ///
    /// # Arguments
    ///
    /// * `proof` - The Merkle proof to verify
    /// * `root_hash` - The expected root hash
    pub fn verify_proof(proof: &MerkleProof, root_hash: &[u8]) -> bool {
        let mut current_hash = Self::hash_leaf(&proof.value);
        let mut index = proof.index;

        for sibling in &proof.siblings {
            current_hash = if index % 2 == 0 {
                Self::hash_pair(&current_hash, sibling)
            } else {
                Self::hash_pair(sibling, &current_hash)
            };
            index /= 2;
        }

        current_hash == root_hash
    }

    // Helper: Hash a leaf value
    fn hash_leaf(value: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(b"leaf:");
        hasher.update(value);
        hasher.finalize().to_vec()
    }

    // Helper: Hash two nodes together
    fn hash_pair(left: &[u8], right: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(b"node:");
        hasher.update(left);
        hasher.update(right);
        hasher.finalize().to_vec()
    }

    // Helper: Build tree recursively
    fn build_tree(hashes: &[Vec<u8>]) -> ZkpResult<MerkleNode> {
        if hashes.is_empty() {
            return Err(ZkpError::TreeConstructionError("Empty hashes".to_string()));
        }

        if hashes.len() == 1 {
            return Ok(MerkleNode::Leaf {
                hash: hashes[0].clone(),
            });
        }

        let mid = (hashes.len() + 1) / 2;
        let left = Self::build_tree(&hashes[..mid])?;
        let right = if mid < hashes.len() {
            Self::build_tree(&hashes[mid..])?
        } else {
            // If odd number, duplicate last node
            left.clone()
        };

        let hash = Self::hash_pair(left.hash(), right.hash());

        Ok(MerkleNode::Internal {
            hash,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    // Helper: Generate sibling hashes for proof
    fn generate_proof_siblings(hashes: &[Vec<u8>], index: usize) -> Vec<Vec<u8>> {
        let mut siblings = Vec::new();
        let mut current_hashes = hashes.to_vec();
        let mut current_index = index;

        while current_hashes.len() > 1 {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            if sibling_index < current_hashes.len() {
                siblings.push(current_hashes[sibling_index].clone());
            } else {
                siblings.push(current_hashes[current_index].clone());
            }

            // Build next level
            let mut next_level = Vec::new();
            for i in (0..current_hashes.len()).step_by(2) {
                let left = &current_hashes[i];
                let right = if i + 1 < current_hashes.len() {
                    &current_hashes[i + 1]
                } else {
                    left
                };
                next_level.push(Self::hash_pair(left, right));
            }

            current_hashes = next_level;
            current_index /= 2;
        }

        siblings
    }
}

/// Merkle proof for set membership
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize)]
pub struct MerkleProof {
    /// The value being proven
    pub value: Vec<u8>,
    /// The index of the value in the tree
    pub index: usize,
    /// Sibling hashes along the path to root
    pub siblings: Vec<Vec<u8>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_commitment() {
        let value = b"secret value";
        let (commitment, nonce) = HashCommitment::commit(value);

        // Valid opening should succeed
        assert!(commitment.verify(value, &nonce));

        // Wrong value should fail
        assert!(!commitment.verify(b"wrong value", &nonce));

        // Wrong nonce should fail
        let wrong_nonce = vec![0u8; 32];
        assert!(!commitment.verify(value, &wrong_nonce));
    }

    #[test]
    fn test_hash_commitment_uniqueness() {
        let value = b"same value";

        let (commitment1, _nonce1) = HashCommitment::commit(value);
        let (commitment2, _nonce2) = HashCommitment::commit(value);

        // Same value with different nonces should produce different commitments
        assert_ne!(commitment1.commitment, commitment2.commitment);
    }

    #[test]
    fn test_range_proof_valid() {
        let value = 50u64;
        let min = 0u64;
        let max = 100u64;

        let (proof, nonce) = RangeProof::prove(value, min, max).unwrap();
        assert!(proof.verify(value, min, max, &nonce));
    }

    #[test]
    fn test_range_proof_out_of_range() {
        let value = 150u64;
        let min = 0u64;
        let max = 100u64;

        let result = RangeProof::prove(value, min, max);
        assert!(result.is_err());
    }

    #[test]
    fn test_range_proof_wrong_value() {
        let value = 50u64;
        let min = 0u64;
        let max = 100u64;

        let (proof, nonce) = RangeProof::prove(value, min, max).unwrap();

        // Verifying with wrong value should fail
        assert!(!proof.verify(40u64, min, max, &nonce));
    }

    #[test]
    fn test_merkle_tree_creation() {
        let values = vec![
            b"value1".to_vec(),
            b"value2".to_vec(),
            b"value3".to_vec(),
            b"value4".to_vec(),
        ];

        let tree = MerkleTree::new(&values).unwrap();
        assert!(tree.root_hash().is_some());
    }

    #[test]
    fn test_merkle_proof_valid() {
        let values = vec![
            b"alice".to_vec(),
            b"bob".to_vec(),
            b"charlie".to_vec(),
            b"david".to_vec(),
        ];

        let tree = MerkleTree::new(&values).unwrap();
        let root_hash = tree.root_hash().unwrap();

        // Prove membership of "bob"
        let proof = tree.prove(b"bob").unwrap();
        assert!(MerkleTree::verify_proof(&proof, &root_hash));
    }

    #[test]
    fn test_merkle_proof_all_values() {
        let values = vec![
            b"value1".to_vec(),
            b"value2".to_vec(),
            b"value3".to_vec(),
        ];

        let tree = MerkleTree::new(&values).unwrap();
        let root_hash = tree.root_hash().unwrap();

        // All values should have valid proofs
        for value in &values {
            let proof = tree.prove(value).unwrap();
            assert!(MerkleTree::verify_proof(&proof, &root_hash));
        }
    }

    #[test]
    fn test_merkle_proof_invalid_value() {
        let values = vec![
            b"value1".to_vec(),
            b"value2".to_vec(),
            b"value3".to_vec(),
        ];

        let tree = MerkleTree::new(&values).unwrap();

        // Value not in tree should fail
        let result = tree.prove(b"not_in_tree");
        assert!(result.is_err());
    }

    #[test]
    fn test_merkle_proof_wrong_root() {
        let values = vec![
            b"value1".to_vec(),
            b"value2".to_vec(),
        ];

        let tree = MerkleTree::new(&values).unwrap();
        let proof = tree.prove(b"value1").unwrap();

        // Wrong root hash should fail verification
        let wrong_root = vec![0u8; 32];
        assert!(!MerkleTree::verify_proof(&proof, &wrong_root));
    }

    #[test]
    fn test_merkle_single_value() {
        let values = vec![b"single".to_vec()];
        let tree = MerkleTree::new(&values).unwrap();
        let root_hash = tree.root_hash().unwrap();

        let proof = tree.prove(b"single").unwrap();
        assert!(MerkleTree::verify_proof(&proof, &root_hash));
    }

    #[test]
    fn test_merkle_odd_number_of_values() {
        let values = vec![
            b"one".to_vec(),
            b"two".to_vec(),
            b"three".to_vec(),
            b"four".to_vec(),
            b"five".to_vec(),
        ];

        let tree = MerkleTree::new(&values).unwrap();
        let root_hash = tree.root_hash().unwrap();

        for value in &values {
            let proof = tree.prove(value).unwrap();
            assert!(MerkleTree::verify_proof(&proof, &root_hash));
        }
    }

    #[test]
    fn test_merkle_empty_values() {
        let values: Vec<Vec<u8>> = vec![];
        let result = MerkleTree::new(&values);
        assert!(result.is_err());
    }
}
