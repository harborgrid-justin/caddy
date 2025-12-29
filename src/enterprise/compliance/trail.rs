//! Immutable audit trail with cryptographic integrity
//!
//! This module implements a tamper-evident audit trail using chain hashing
//! and digital signatures for compliance requirements.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Immutable audit entry with cryptographic verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique entry identifier
    pub id: Uuid,

    /// Timestamp when entry was created (immutable)
    pub timestamp: DateTime<Utc>,

    /// User or system that created this entry
    pub actor: String,

    /// Action performed
    pub action: String,

    /// Resource affected
    pub resource: String,

    /// Additional metadata
    pub metadata: HashMap<String, String>,

    /// Hash of this entry's content
    pub hash: String,

    /// Hash of the previous entry (chain)
    pub previous_hash: Option<String>,

    /// Sequence number in the chain
    pub sequence: u64,

    /// Digital signature (if signing enabled)
    pub signature: Option<String>,

    /// Whether this entry has been verified
    #[serde(skip)]
    pub verified: bool,
}

impl AuditEntry {
    /// Calculate hash of entry content using BLAKE3
    pub fn calculate_hash(&self) -> String {
        let mut hasher = blake3::Hasher::new();

        hasher.update(self.id.as_bytes());
        hasher.update(self.timestamp.to_rfc3339().as_bytes());
        hasher.update(self.actor.as_bytes());
        hasher.update(self.action.as_bytes());
        hasher.update(self.resource.as_bytes());
        hasher.update(&self.sequence.to_le_bytes());

        // Include metadata in sorted order for deterministic hashing
        let mut keys: Vec<_> = self.metadata.keys().collect();
        keys.sort();
        for key in keys {
            hasher.update(key.as_bytes());
            hasher.update(self.metadata[key].as_bytes());
        }

        // Include previous hash to create the chain
        if let Some(ref prev) = self.previous_hash {
            hasher.update(prev.as_bytes());
        }

        hasher.finalize().to_hex().to_string()
    }

    /// Verify the integrity of this entry
    pub fn verify_integrity(&self) -> bool {
        let calculated_hash = self.calculate_hash();
        calculated_hash == self.hash
    }

    /// Sign this entry (simplified - in production use proper digital signatures)
    pub fn sign(&mut self, _private_key: &[u8]) {
        // In production, use ed25519-dalek or similar
        // For now, just use hash as signature
        self.signature = Some(self.hash.clone());
    }

    /// Verify signature of this entry
    pub fn verify_signature(&self, _public_key: &[u8]) -> bool {
        if let Some(ref sig) = self.signature {
            // In production, verify actual digital signature
            sig == &self.hash
        } else {
            false
        }
    }
}

/// Builder for creating audit entries
pub struct AuditEntryBuilder {
    actor: String,
    action: String,
    resource: String,
    metadata: HashMap<String, String>,
    timestamp: Option<DateTime<Utc>>,
}

impl AuditEntryBuilder {
    /// Create new builder
    pub fn new(actor: impl Into<String>, action: impl Into<String>, resource: impl Into<String>) -> Self {
        Self {
            actor: actor.into(),
            action: action.into(),
            resource: resource.into(),
            metadata: HashMap::new(),
            timestamp: None,
        }
    }

    /// Add metadata
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set custom timestamp
    pub fn timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Build the entry (used internally by AuditTrail)
    fn build_internal(self, sequence: u64, previous_hash: Option<String>) -> AuditEntry {
        let mut entry = AuditEntry {
            id: Uuid::new_v4(),
            timestamp: self.timestamp.unwrap_or_else(Utc::now),
            actor: self.actor,
            action: self.action,
            resource: self.resource,
            metadata: self.metadata,
            hash: String::new(),
            previous_hash,
            sequence,
            signature: None,
            verified: false,
        };

        // Calculate hash after all fields are set
        entry.hash = entry.calculate_hash();
        entry
    }
}

/// Immutable audit trail with chain verification
pub struct AuditTrail {
    /// Chain of audit entries
    entries: Arc<RwLock<Vec<AuditEntry>>>,

    /// Enable digital signatures
    enable_signing: bool,

    /// Signing key (if enabled)
    signing_key: Option<Vec<u8>>,
}

impl AuditTrail {
    /// Create a new audit trail
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
            enable_signing: false,
            signing_key: None,
        }
    }

    /// Create audit trail with signing enabled
    pub fn with_signing(signing_key: Vec<u8>) -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
            enable_signing: true,
            signing_key: Some(signing_key),
        }
    }

    /// Append a new entry to the trail
    pub async fn append(&self, builder: AuditEntryBuilder) -> Result<Uuid, String> {
        let mut entries = self.entries.write().await;

        let sequence = entries.len() as u64;
        let previous_hash = entries.last().map(|e| e.hash.clone());

        let mut entry = builder.build_internal(sequence, previous_hash);

        // Sign if enabled
        if self.enable_signing {
            if let Some(ref key) = self.signing_key {
                entry.sign(key);
            }
        }

        let entry_id = entry.id;
        entries.push(entry);

        Ok(entry_id)
    }

    /// Get entry by ID
    pub async fn get_entry(&self, id: Uuid) -> Option<AuditEntry> {
        let entries = self.entries.read().await;
        entries.iter().find(|e| e.id == id).cloned()
    }

    /// Get entries in a time range
    pub async fn get_entries_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<AuditEntry> {
        let entries = self.entries.read().await;
        entries
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .cloned()
            .collect()
    }

    /// Get entries by actor
    pub async fn get_entries_by_actor(&self, actor: &str) -> Vec<AuditEntry> {
        let entries = self.entries.read().await;
        entries
            .iter()
            .filter(|e| e.actor == actor)
            .cloned()
            .collect()
    }

    /// Get entries by resource
    pub async fn get_entries_by_resource(&self, resource: &str) -> Vec<AuditEntry> {
        let entries = self.entries.read().await;
        entries
            .iter()
            .filter(|e| e.resource == resource)
            .cloned()
            .collect()
    }

    /// Verify the integrity of the entire chain
    pub async fn verify_chain(&self) -> Result<(), Vec<String>> {
        let entries = self.entries.read().await;
        let mut errors = Vec::new();

        for (i, entry) in entries.iter().enumerate() {
            // Verify hash
            if !entry.verify_integrity() {
                errors.push(format!("Entry {} (seq {}) has invalid hash", entry.id, i));
            }

            // Verify chain linkage
            if i > 0 {
                let prev_hash = &entries[i - 1].hash;
                if entry.previous_hash.as_ref() != Some(prev_hash) {
                    errors.push(format!(
                        "Entry {} (seq {}) has broken chain link",
                        entry.id, i
                    ));
                }
            } else if entry.previous_hash.is_some() {
                errors.push(format!("First entry {} should have no previous hash", entry.id));
            }

            // Verify signature if present
            if entry.signature.is_some() {
                if let Some(ref key) = self.signing_key {
                    if !entry.verify_signature(key) {
                        errors.push(format!("Entry {} (seq {}) has invalid signature", entry.id, i));
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get total number of entries
    pub async fn len(&self) -> usize {
        let entries = self.entries.read().await;
        entries.len()
    }

    /// Check if trail is empty
    pub async fn is_empty(&self) -> bool {
        let entries = self.entries.read().await;
        entries.is_empty()
    }

    /// Get all entries (for export/backup)
    pub async fn export_all(&self) -> Vec<AuditEntry> {
        let entries = self.entries.read().await;
        entries.clone()
    }

    /// Import entries (for restore) - validates chain integrity
    pub async fn import(&self, imported_entries: Vec<AuditEntry>) -> Result<(), String> {
        // Verify imported chain first
        let mut prev_hash: Option<String> = None;
        for (i, entry) in imported_entries.iter().enumerate() {
            if !entry.verify_integrity() {
                return Err(format!("Imported entry {} has invalid hash", i));
            }

            if entry.previous_hash != prev_hash {
                return Err(format!("Imported entry {} has broken chain", i));
            }

            prev_hash = Some(entry.hash.clone());
        }

        // If valid, replace current chain
        let mut entries = self.entries.write().await;
        *entries = imported_entries;

        Ok(())
    }
}

impl Default for AuditTrail {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_entry_hash() {
        let builder = AuditEntryBuilder::new("user1", "create", "document/123");
        let _entry = builder.build_internal(0, None);

        assert!(entry.verify_integrity());
    }

    #[tokio::test]
    async fn test_audit_trail_append() {
        let trail = AuditTrail::new();

        let builder = AuditEntryBuilder::new("user1", "create", "doc1");
        let id = trail.append(builder).await.unwrap();

        assert_eq!(trail.len().await, 1);
        let _entry = trail.get_entry(id).await.unwrap();
        assert_eq!(entry.actor, "user1");
    }

    #[tokio::test]
    async fn test_chain_integrity() {
        let trail = AuditTrail::new();

        trail
            .append(AuditEntryBuilder::new("user1", "create", "doc1"))
            .await
            .unwrap();
        trail
            .append(AuditEntryBuilder::new("user2", "update", "doc1"))
            .await
            .unwrap();
        trail
            .append(AuditEntryBuilder::new("user1", "delete", "doc2"))
            .await
            .unwrap();

        assert!(trail.verify_chain().await.is_ok());
    }

    #[tokio::test]
    async fn test_chain_tamper_detection() {
        let trail = AuditTrail::new();

        trail
            .append(AuditEntryBuilder::new("user1", "create", "doc1"))
            .await
            .unwrap();
        trail
            .append(AuditEntryBuilder::new("user2", "update", "doc1"))
            .await
            .unwrap();

        // Tamper with an entry
        {
            let mut entries = trail.entries.write().await;
            entries[0].action = "modified".to_string();
        }

        let result = trail.verify_chain().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_query_by_actor() {
        let trail = AuditTrail::new();

        trail
            .append(AuditEntryBuilder::new("user1", "create", "doc1"))
            .await
            .unwrap();
        trail
            .append(AuditEntryBuilder::new("user2", "update", "doc1"))
            .await
            .unwrap();
        trail
            .append(AuditEntryBuilder::new("user1", "delete", "doc2"))
            .await
            .unwrap();

        let user1_entries = trail.get_entries_by_actor("user1").await;
        assert_eq!(user1_entries.len(), 2);
    }

    #[tokio::test]
    async fn test_export_import() {
        let trail1 = AuditTrail::new();

        trail1
            .append(AuditEntryBuilder::new("user1", "create", "doc1"))
            .await
            .unwrap();
        trail1
            .append(AuditEntryBuilder::new("user2", "update", "doc1"))
            .await
            .unwrap();

        let exported = trail1.export_all().await;

        let trail2 = AuditTrail::new();
        trail2.import(exported).await.unwrap();

        assert_eq!(trail2.len().await, 2);
        assert!(trail2.verify_chain().await.is_ok());
    }
}
