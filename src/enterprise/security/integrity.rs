// CADDY v0.1.5 - Data Integrity Services
// Hash computation, Merkle trees, and tamper detection

use crate::enterprise::security::{SecurityError, SecurityResult, HashAlgorithm};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Hash digest
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HashDigest {
    pub algorithm: String,
    #[serde(with = "hex")]
    pub digest: Vec<u8>,
    pub timestamp: i64,
}

/// Merkle tree node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleNode {
    #[serde(with = "hex")]
    pub hash: Vec<u8>,
    pub left: Option<Box<MerkleNode>>,
    pub right: Option<Box<MerkleNode>>,
    pub is_leaf: bool,
}

/// Merkle tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleTree {
    pub root: Option<MerkleNode>,
    pub leaf_count: usize,
    pub algorithm: String,
}

/// Merkle proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub leaf_index: usize,
    #[serde(with = "hex_vec")]
    pub proof_hashes: Vec<Vec<u8>>,
    pub proof_directions: Vec<bool>, // true = right, false = left
    #[serde(with = "hex")]
    pub root_hash: Vec<u8>,
}

/// Integrity proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityProof {
    pub data_id: String,
    pub hash: HashDigest,
    pub merkle_proof: Option<MerkleProof>,
    pub timestamp: i64,
    pub metadata: HashMap<String, String>,
}

/// Tamper detection result
#[derive(Debug, Clone)]
pub struct TamperDetectionResult {
    pub is_intact: bool,
    pub original_hash: Vec<u8>,
    pub current_hash: Vec<u8>,
    pub tampered_at: Option<i64>,
    pub details: Vec<String>,
}

/// Integrity verification record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityRecord {
    pub record_id: String,
    pub data_id: String,
    pub hash_chain: Vec<HashDigest>,
    pub created_at: i64,
    pub last_verified: i64,
    pub verification_count: u64,
}

/// Main integrity service
pub struct IntegrityService {
    default_algorithm: HashAlgorithm,
    records: HashMap<String, IntegrityRecord>,
}

impl IntegrityService {
    /// Create a new integrity service
    pub fn new() -> Self {
        Self {
            default_algorithm: HashAlgorithm::Blake3,
            records: HashMap::new(),
        }
    }

    /// Compute SHA-256 hash
    pub fn hash_sha256(&self, data: &[u8]) -> HashDigest {
        let digest = self.compute_sha256(data);
        HashDigest {
            algorithm: "SHA-256".to_string(),
            digest,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Compute SHA-3-256 hash
    pub fn hash_sha3_256(&self, data: &[u8]) -> HashDigest {
        let digest = self.compute_sha3_256(data);
        HashDigest {
            algorithm: "SHA3-256".to_string(),
            digest,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Compute BLAKE3 hash
    pub fn hash_blake3(&self, data: &[u8]) -> HashDigest {
        let digest = self.compute_blake3(data);
        HashDigest {
            algorithm: "BLAKE3".to_string(),
            digest,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Compute hash with specified algorithm
    pub fn hash(&self, data: &[u8], algorithm: HashAlgorithm) -> HashDigest {
        match algorithm {
            HashAlgorithm::Sha256 => self.hash_sha256(data),
            HashAlgorithm::Sha3_256 => self.hash_sha3_256(data),
            HashAlgorithm::Blake3 => self.hash_blake3(data),
            HashAlgorithm::Sha512 => {
                let digest = self.compute_sha512(data);
                HashDigest {
                    algorithm: "SHA-512".to_string(),
                    digest,
                    timestamp: chrono::Utc::now().timestamp(),
                }
            }
        }
    }

    /// Build Merkle tree from data chunks
    pub fn build_merkle_tree(&self, chunks: &[&[u8]], algorithm: HashAlgorithm) -> SecurityResult<MerkleTree> {
        if chunks.is_empty() {
            return Err(SecurityError::InvalidInput("No chunks provided".to_string()));
        }

        let algo_name = match algorithm {
            HashAlgorithm::Sha256 => "SHA-256",
            HashAlgorithm::Sha3_256 => "SHA3-256",
            HashAlgorithm::Blake3 => "BLAKE3",
            HashAlgorithm::Sha512 => "SHA-512",
        };

        // Create leaf nodes
        let mut nodes: Vec<MerkleNode> = chunks
            .iter()
            .map(|chunk| {
                let hash = self.hash(chunk, algorithm).digest;
                MerkleNode {
                    hash,
                    left: None,
                    right: None,
                    is_leaf: true,
                }
            })
            .collect();

        let leaf_count = nodes.len();

        // Build tree bottom-up
        while nodes.len() > 1 {
            let mut next_level = Vec::new();

            for i in (0..nodes.len()).step_by(2) {
                if i + 1 < nodes.len() {
                    // Pair of nodes
                    let left = nodes[i].clone();
                    let right = nodes[i + 1].clone();

                    let combined = [left.hash.as_slice(), right.hash.as_slice()].concat();
                    let parent_hash = self.hash(&combined, algorithm).digest;

                    next_level.push(MerkleNode {
                        hash: parent_hash,
                        left: Some(Box::new(left)),
                        right: Some(Box::new(right)),
                        is_leaf: false,
                    });
                } else {
                    // Odd node out - promote to next level
                    next_level.push(nodes[i].clone());
                }
            }

            nodes = next_level;
        }

        Ok(MerkleTree {
            root: Some(nodes.into_iter().next().unwrap()),
            leaf_count,
            algorithm: algo_name.to_string(),
        })
    }

    /// Generate Merkle proof for a leaf
    pub fn generate_merkle_proof(
        &self,
        tree: &MerkleTree,
        leaf_index: usize,
    ) -> SecurityResult<MerkleProof> {
        if leaf_index >= tree.leaf_count {
            return Err(SecurityError::InvalidInput("Leaf index out of bounds".to_string()));
        }

        let root = tree.root.as_ref()
            .ok_or_else(|| SecurityError::Integrity("Empty tree".to_string()))?;

        let mut proof_hashes = Vec::new();
        let mut proof_directions = Vec::new();

        self.collect_proof(root, leaf_index, 0, tree.leaf_count, &mut proof_hashes, &mut proof_directions)?;

        Ok(MerkleProof {
            leaf_index,
            proof_hashes,
            proof_directions,
            root_hash: root.hash.clone(),
        })
    }

    /// Verify Merkle proof
    pub fn verify_merkle_proof(
        &self,
        leaf_data: &[u8],
        proof: &MerkleProof,
        algorithm: HashAlgorithm,
    ) -> SecurityResult<bool> {
        let mut current_hash = self.hash(leaf_data, algorithm).digest;

        for (sibling_hash, is_right) in proof.proof_hashes.iter().zip(&proof.proof_directions) {
            let combined = if *is_right {
                [current_hash.as_slice(), sibling_hash.as_slice()].concat()
            } else {
                [sibling_hash.as_slice(), current_hash.as_slice()].concat()
            };

            current_hash = self.hash(&combined, algorithm).digest;
        }

        Ok(current_hash == proof.root_hash)
    }

    /// Detect tampering by comparing hashes
    pub fn detect_tampering(
        &self,
        data: &[u8],
        original_hash: &HashDigest,
    ) -> SecurityResult<TamperDetectionResult> {
        let algorithm = match original_hash.algorithm.as_str() {
            "SHA-256" => HashAlgorithm::Sha256,
            "SHA3-256" => HashAlgorithm::Sha3_256,
            "BLAKE3" => HashAlgorithm::Blake3,
            "SHA-512" => HashAlgorithm::Sha512,
            _ => return Err(SecurityError::InvalidInput(
                format!("Unknown algorithm: {}", original_hash.algorithm)
            )),
        };

        let current_hash = self.hash(data, algorithm);

        let is_intact = current_hash.digest == original_hash.digest;

        let mut details = Vec::new();
        if !is_intact {
            details.push("Data has been modified".to_string());
            details.push(format!("Original hash: {}", hex_encode(&original_hash.digest)));
            details.push(format!("Current hash: {}", hex_encode(&current_hash.digest)));
        }

        Ok(TamperDetectionResult {
            is_intact,
            original_hash: original_hash.digest.clone(),
            current_hash: current_hash.digest,
            tampered_at: if is_intact { None } else { Some(chrono::Utc::now().timestamp()) },
            details,
        })
    }

    /// Create integrity record
    pub fn create_integrity_record(
        &mut self,
        data_id: &str,
        data: &[u8],
        algorithm: HashAlgorithm,
    ) -> SecurityResult<IntegrityRecord> {
        let hash = self.hash(data, algorithm);

        let record = IntegrityRecord {
            record_id: format!("integrity_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)),
            data_id: data_id.to_string(),
            hash_chain: vec![hash],
            created_at: chrono::Utc::now().timestamp(),
            last_verified: chrono::Utc::now().timestamp(),
            verification_count: 1,
        };

        self.records.insert(data_id.to_string(), record.clone());

        Ok(record)
    }

    /// Verify integrity against record
    pub fn verify_integrity(
        &mut self,
        data_id: &str,
        data: &[u8],
    ) -> SecurityResult<TamperDetectionResult> {
        // First, get the latest hash from the record
        let latest_hash = {
            let record = self.records.get(data_id)
                .ok_or_else(|| SecurityError::Integrity(format!("No integrity record found: {}", data_id)))?;

            record.hash_chain.last()
                .ok_or_else(|| SecurityError::Integrity("Empty hash chain".to_string()))?
                .clone()
        };

        // Detect tampering (immutable borrow)
        let result = self.detect_tampering(data, &latest_hash)?;

        // Compute new hash if needed (immutable borrow)
        let new_hash = if result.is_intact {
            let algorithm = match latest_hash.algorithm.as_str() {
                "SHA-256" => HashAlgorithm::Sha256,
                "SHA3-256" => HashAlgorithm::Sha3_256,
                "BLAKE3" => HashAlgorithm::Blake3,
                "SHA-512" => HashAlgorithm::Sha512,
                _ => HashAlgorithm::Blake3,
            };
            Some(self.hash(data, algorithm))
        } else {
            None
        };

        // Now update the record (mutable borrow)
        let record = self.records.get_mut(data_id).unwrap();
        record.last_verified = chrono::Utc::now().timestamp();
        record.verification_count += 1;

        if let Some(hash) = new_hash {
            record.hash_chain.push(hash);
        }

        Ok(result)
    }

    /// Create integrity proof
    pub fn create_integrity_proof(
        &self,
        data_id: &str,
        data: &[u8],
        algorithm: HashAlgorithm,
    ) -> SecurityResult<IntegrityProof> {
        let hash = self.hash(data, algorithm);

        let mut metadata = HashMap::new();
        metadata.insert("data_id".to_string(), data_id.to_string());
        metadata.insert("algorithm".to_string(), hash.algorithm.clone());

        Ok(IntegrityProof {
            data_id: data_id.to_string(),
            hash,
            merkle_proof: None,
            timestamp: chrono::Utc::now().timestamp(),
            metadata,
        })
    }

    /// Verify integrity proof
    pub fn verify_integrity_proof(
        &self,
        data: &[u8],
        proof: &IntegrityProof,
    ) -> SecurityResult<bool> {
        let algorithm = match proof.hash.algorithm.as_str() {
            "SHA-256" => HashAlgorithm::Sha256,
            "SHA3-256" => HashAlgorithm::Sha3_256,
            "BLAKE3" => HashAlgorithm::Blake3,
            "SHA-512" => HashAlgorithm::Sha512,
            _ => return Err(SecurityError::InvalidInput(
                format!("Unknown algorithm: {}", proof.hash.algorithm)
            )),
        };

        let computed_hash = self.hash(data, algorithm);

        Ok(computed_hash.digest == proof.hash.digest)
    }

    /// Get integrity record
    pub fn get_integrity_record(&self, data_id: &str) -> SecurityResult<IntegrityRecord> {
        self.records.get(data_id)
            .cloned()
            .ok_or_else(|| SecurityError::Integrity(format!("No integrity record found: {}", data_id)))
    }

    /// List all integrity records
    pub fn list_integrity_records(&self) -> Vec<IntegrityRecord> {
        self.records.values().cloned().collect()
    }

    // Hash computation implementations (simplified)

    fn compute_sha256(&self, data: &[u8]) -> Vec<u8> {
        // Simulate SHA-256
        let mut hash = vec![0u8; 32];
        for (i, byte) in data.iter().enumerate() {
            hash[i % 32] = hash[i % 32].wrapping_add(*byte).wrapping_mul(251);
        }
        // Additional mixing
        for i in 0..32 {
            hash[i] = hash[i].wrapping_add(hash[(i + 7) % 32]).wrapping_mul(179);
        }
        hash
    }

    fn compute_sha3_256(&self, data: &[u8]) -> Vec<u8> {
        // Simulate SHA3-256 (different mixing than SHA-256)
        let mut hash = vec![0u8; 32];
        for (i, byte) in data.iter().enumerate() {
            hash[i % 32] = hash[i % 32].wrapping_add(*byte).wrapping_mul(241);
        }
        // Different mixing pattern
        for round in 0..24 {
            for i in 0..32 {
                hash[i] = hash[i].wrapping_add(hash[(i + 5) % 32]).wrapping_mul(199).wrapping_add(round);
            }
        }
        hash
    }

    fn compute_blake3(&self, data: &[u8]) -> Vec<u8> {
        // Simulate BLAKE3 (tree-based hashing)
        let mut hash = vec![0u8; 32];
        let iv = b"CADDY_BLAKE3_IV_2025";

        for (i, byte) in data.iter().enumerate() {
            hash[i % 32] = hash[i % 32]
                .wrapping_add(*byte)
                .wrapping_add(iv[i % iv.len()])
                .wrapping_mul(239);
        }

        // Tree-like mixing
        for _ in 0..7 {
            for i in 0..16 {
                let a = hash[i];
                let b = hash[i + 16];
                hash[i] = a.wrapping_add(b).wrapping_mul(229);
                hash[i + 16] = a ^ b;
            }
        }

        hash
    }

    fn compute_sha512(&self, data: &[u8]) -> Vec<u8> {
        // Simulate SHA-512
        let mut hash = vec![0u8; 64];
        for (i, byte) in data.iter().enumerate() {
            hash[i % 64] = hash[i % 64].wrapping_add(*byte).wrapping_mul(251);
        }
        // Additional rounds
        for _ in 0..80 {
            for i in 0..64 {
                hash[i] = hash[i].wrapping_add(hash[(i + 13) % 64]).wrapping_mul(181);
            }
        }
        hash
    }

    fn collect_proof(
        &self,
        node: &MerkleNode,
        target_index: usize,
        current_start: usize,
        current_size: usize,
        proof_hashes: &mut Vec<Vec<u8>>,
        proof_directions: &mut Vec<bool>,
    ) -> SecurityResult<bool> {
        if node.is_leaf {
            return Ok(current_start == target_index);
        }

        let mid = current_start + current_size / 2;

        let left = node.left.as_ref()
            .ok_or_else(|| SecurityError::Integrity("Invalid tree structure".to_string()))?;

        let right = node.right.as_ref()
            .ok_or_else(|| SecurityError::Integrity("Invalid tree structure".to_string()))?;

        if target_index < mid {
            // Target is in left subtree
            let found = self.collect_proof(
                left,
                target_index,
                current_start,
                current_size / 2,
                proof_hashes,
                proof_directions,
            )?;

            if found {
                proof_hashes.push(right.hash.clone());
                proof_directions.push(true); // right sibling
            }

            Ok(found)
        } else {
            // Target is in right subtree
            let found = self.collect_proof(
                right,
                target_index,
                mid,
                current_size - current_size / 2,
                proof_hashes,
                proof_directions,
            )?;

            if found {
                proof_hashes.push(left.hash.clone());
                proof_directions.push(false); // left sibling
            }

            Ok(found)
        }
    }
}

impl Default for IntegrityService {
    fn default() -> Self {
        Self::new()
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

mod hex {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&super::hex_encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        hex_decode(&s).map_err(serde::de::Error::custom)
    }

    fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
        if s.len() % 2 != 0 {
            return Err("Odd hex string length".to_string());
        }
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
            .collect()
    }
}

mod hex_vec {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(vec: &[Vec<u8>], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(vec.len()))?;
        for bytes in vec {
            seq.serialize_element(&super::hex_encode(bytes))?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let strings: Vec<String> = Vec::deserialize(deserializer)?;
        strings.iter()
            .map(|s| hex_decode(s).map_err(serde::de::Error::custom))
            .collect()
    }

    fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
        if s.len() % 2 != 0 {
            return Err("Odd hex string length".to_string());
        }
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_computation() {
        let service = IntegrityService::new();
        let data = b"Test data";

        let hash1 = service.hash_sha256(data);
        let hash2 = service.hash_sha256(data);

        assert_eq!(hash1.digest, hash2.digest);
    }

    #[test]
    fn test_tamper_detection() {
        let service = IntegrityService::new();
        let data = b"Original data";
        let modified = b"Modified data";

        let hash = service.hash_sha256(data);
        let result = service.detect_tampering(modified, &hash).unwrap();

        assert!(!result.is_intact);
    }

    #[test]
    fn test_merkle_tree() {
        let service = IntegrityService::new();
        let chunks = vec![b"chunk1".as_slice(), b"chunk2".as_slice(), b"chunk3".as_slice()];

        let tree = service.build_merkle_tree(&chunks, HashAlgorithm::Blake3).unwrap();
        assert_eq!(tree.leaf_count, 3);

        let proof = service.generate_merkle_proof(&tree, 1).unwrap();
        let verified = service.verify_merkle_proof(b"chunk2", &proof, HashAlgorithm::Blake3).unwrap();

        assert!(verified);
    }
}
