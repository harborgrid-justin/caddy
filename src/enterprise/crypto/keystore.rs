//! # Key Store and Management
//!
//! This module provides secure key storage, rotation, versioning, and backup/recovery
//! capabilities for cryptographic keys.
//!
//! ## Features
//!
//! - Encrypted key storage
//! - Key versioning and rotation
//! - Automatic key expiration
//! - Key backup and recovery
//! - Master key protection
//!
//! ## Security Considerations
//!
//! - Keys are encrypted at rest using AES-256-GCM
//! - Master key is derived from a password using Argon2id
//! - All keys are zeroized on drop
//! - Key rotation policies can be enforced

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use zeroize::Zeroize;
use thiserror::Error;
use uuid::Uuid;

use super::symmetric::{Aes256GcmCipher, EncryptedData, SymmetricError};
use super::kdf::{KdfProvider, Argon2Config};

/// KeyStore errors
#[derive(Error, Debug)]
pub enum KeyStoreError {
    /// Key not found
    #[error("Key not found: {0}")]
    KeyNotFound(String),

    /// Key already exists
    #[error("Key already exists: {0}")]
    KeyExists(String),

    /// Key has expired
    #[error("Key has expired: {0}")]
    KeyExpired(String),

    /// Invalid master password
    #[error("Invalid master password")]
    InvalidPassword,

    /// Encryption error
    #[error("Encryption error: {0}")]
    EncryptionError(#[from] SymmetricError),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Key rotation required
    #[error("Key rotation required for key: {0}")]
    RotationRequired(String),

    /// Invalid key version
    #[error("Invalid key version: {0}")]
    InvalidVersion(String),
}

pub type KeyStoreResult<T> = Result<T, KeyStoreError>;

/// Key metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    /// Unique key identifier
    pub id: String,
    /// Key name/label
    pub name: String,
    /// Key purpose (encryption, signing, etc.)
    pub purpose: KeyPurpose,
    /// Key algorithm
    pub algorithm: String,
    /// Key version
    pub version: u32,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last used timestamp
    pub last_used: Option<DateTime<Utc>>,
    /// Expiration timestamp
    pub expires_at: Option<DateTime<Utc>>,
    /// Key rotation policy
    pub rotation_policy: Option<RotationPolicy>,
    /// Whether the key is active
    pub active: bool,
    /// Custom metadata
    pub tags: HashMap<String, String>,
}

impl KeyMetadata {
    /// Check if the key has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            expires_at < Utc::now()
        } else {
            false
        }
    }

    /// Check if rotation is required
    pub fn needs_rotation(&self) -> bool {
        if let Some(policy) = &self.rotation_policy {
            if let Some(last_used) = self.last_used {
                let age = Utc::now() - last_used;
                age > policy.max_age
            } else {
                let age = Utc::now() - self.created_at;
                age > policy.max_age
            }
        } else {
            false
        }
    }
}

/// Key purpose
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum KeyPurpose {
    /// For encrypting data
    Encryption,
    /// For signing data
    Signing,
    /// For key wrapping/unwrapping
    KeyWrapping,
    /// For key derivation
    Derivation,
    /// General purpose
    General,
}

/// Key rotation policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationPolicy {
    /// Maximum age before rotation is required
    pub max_age: Duration,
    /// Automatically rotate when max_age is reached
    pub auto_rotate: bool,
    /// Keep old versions after rotation
    pub keep_old_versions: u32,
}

impl Default for RotationPolicy {
    fn default() -> Self {
        Self {
            max_age: Duration::days(90),
            auto_rotate: false,
            keep_old_versions: 3,
        }
    }
}

/// Stored key (encrypted)
#[derive(Clone, Serialize, Deserialize)]
struct StoredKey {
    metadata: KeyMetadata,
    #[serde(skip)]
    encrypted_key_material: Vec<u8>,
    #[serde(skip)]
    nonce: Vec<u8>,
}

impl Drop for StoredKey {
    fn drop(&mut self) {
        self.encrypted_key_material.zeroize();
        self.nonce.zeroize();
    }
}

/// Key material (decrypted, zeroized on drop)
#[derive(Clone)]
pub struct KeyMaterial {
    pub metadata: KeyMetadata,
    key_bytes: Vec<u8>,
}

impl KeyMaterial {
    /// Create new key material
    pub fn new(metadata: KeyMetadata, key_bytes: Vec<u8>) -> Self {
        Self {
            metadata,
            key_bytes,
        }
    }

    /// Get the key bytes (use with caution)
    pub fn as_bytes(&self) -> &[u8] {
        &self.key_bytes
    }

    /// Get key length
    pub fn len(&self) -> usize {
        self.key_bytes.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.key_bytes.is_empty()
    }
}

impl Drop for KeyMaterial {
    fn drop(&mut self) {
        self.key_bytes.zeroize();
    }
}

impl std::fmt::Debug for KeyMaterial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyMaterial")
            .field("metadata", &self.metadata)
            .field("key_bytes", &"[REDACTED]")
            .finish()
    }
}

/// Secure key store
pub struct KeyStore {
    master_key: Vec<u8>,
    keys: HashMap<String, StoredKey>,
    salt: Vec<u8>,
}

impl KeyStore {
    /// Create a new key store with a master password
    ///
    /// # Arguments
    ///
    /// * `master_password` - Password to derive the master encryption key
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use caddy::enterprise::crypto::keystore::KeyStore;
    ///
    /// let store = KeyStore::new(b"strong_master_password")?;
    /// ```
    pub fn new(master_password: &[u8]) -> KeyStoreResult<Self> {
        // Generate random salt for master key derivation
        let mut salt = vec![0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut salt);

        // Derive master key using Argon2id
        let master_key_derived = KdfProvider::derive_argon2id(
            master_password,
            &salt,
            &Argon2Config::owasp_high_security(),
        ).map_err(|e| KeyStoreError::SerializationError(e.to_string()))?;

        Ok(Self {
            master_key: master_key_derived.as_bytes().to_vec(),
            keys: HashMap::new(),
            salt,
        })
    }

    /// Load key store from existing salt and password
    pub fn load(master_password: &[u8], salt: &[u8]) -> KeyStoreResult<Self> {
        let master_key_derived = KdfProvider::derive_argon2id(
            master_password,
            salt,
            &Argon2Config::owasp_high_security(),
        ).map_err(|e| KeyStoreError::SerializationError(e.to_string()))?;

        Ok(Self {
            master_key: master_key_derived.as_bytes().to_vec(),
            keys: HashMap::new(),
            salt: salt.to_vec(),
        })
    }

    /// Get the salt (needed for reloading the keystore)
    pub fn salt(&self) -> &[u8] {
        &self.salt
    }

    /// Store a new key
    ///
    /// # Arguments
    ///
    /// * `name` - Human-readable key name
    /// * `purpose` - Key purpose
    /// * `algorithm` - Algorithm identifier
    /// * `key_material` - The actual key bytes
    /// * `rotation_policy` - Optional rotation policy
    pub fn store_key(
        &mut self,
        name: String,
        purpose: KeyPurpose,
        algorithm: String,
        key_material: &[u8],
        rotation_policy: Option<RotationPolicy>,
    ) -> KeyStoreResult<String> {
        let key_id = Uuid::new_v4().to_string();

        // Check if a key with this name already exists
        if self.keys.values().any(|k| k.metadata.name == name) {
            return Err(KeyStoreError::KeyExists(name));
        }

        let metadata = KeyMetadata {
            id: key_id.clone(),
            name,
            purpose,
            algorithm,
            version: 1,
            created_at: Utc::now(),
            last_used: None,
            expires_at: None,
            rotation_policy,
            active: true,
            tags: HashMap::new(),
        };

        // Encrypt key material
        let cipher = Aes256GcmCipher::new(&self.master_key)
            .map_err(|e| KeyStoreError::EncryptionError(e))?;

        let encrypted = cipher.encrypt(key_material, None)?;

        let stored_key = StoredKey {
            metadata,
            encrypted_key_material: encrypted.ciphertext.clone(),
            nonce: encrypted.nonce.clone(),
        };

        self.keys.insert(key_id.clone(), stored_key);

        Ok(key_id)
    }

    /// Retrieve a key by ID
    pub fn get_key(&mut self, key_id: &str) -> KeyStoreResult<KeyMaterial> {
        let stored_key = self.keys
            .get_mut(key_id)
            .ok_or_else(|| KeyStoreError::KeyNotFound(key_id.to_string()))?;

        // Check expiration
        if stored_key.metadata.is_expired() {
            return Err(KeyStoreError::KeyExpired(key_id.to_string()));
        }

        // Check if rotation is required
        if stored_key.metadata.needs_rotation() {
            return Err(KeyStoreError::RotationRequired(key_id.to_string()));
        }

        // Decrypt key material
        let cipher = Aes256GcmCipher::new(&self.master_key)
            .map_err(|e| KeyStoreError::EncryptionError(e))?;

        let encrypted = EncryptedData::new(
            stored_key.encrypted_key_material.clone(),
            stored_key.nonce.clone(),
            Vec::new(),
        );

        let key_bytes = cipher.decrypt(&encrypted)?;

        // Update last used timestamp
        stored_key.metadata.last_used = Some(Utc::now());

        Ok(KeyMaterial::new(stored_key.metadata.clone(), key_bytes))
    }

    /// Get key by name
    pub fn get_key_by_name(&mut self, name: &str) -> KeyStoreResult<KeyMaterial> {
        let key_id = self.keys
            .values()
            .find(|k| k.metadata.name == name)
            .map(|k| k.metadata.id.clone())
            .ok_or_else(|| KeyStoreError::KeyNotFound(name.to_string()))?;

        self.get_key(&key_id)
    }

    /// List all key metadata
    pub fn list_keys(&self) -> Vec<KeyMetadata> {
        self.keys.values().map(|k| k.metadata.clone()).collect()
    }

    /// Rotate a key (creates new version)
    pub fn rotate_key(&mut self, key_id: &str, new_key_material: &[u8]) -> KeyStoreResult<String> {
        let old_key = self.keys
            .get(key_id)
            .ok_or_else(|| KeyStoreError::KeyNotFound(key_id.to_string()))?;

        let mut new_metadata = old_key.metadata.clone();
        new_metadata.id = Uuid::new_v4().to_string();
        new_metadata.version += 1;
        new_metadata.created_at = Utc::now();
        new_metadata.last_used = None;

        // Encrypt new key material
        let cipher = Aes256GcmCipher::new(&self.master_key)
            .map_err(|e| KeyStoreError::EncryptionError(e))?;

        let encrypted = cipher.encrypt(new_key_material, None)?;

        let new_stored_key = StoredKey {
            metadata: new_metadata.clone(),
            encrypted_key_material: encrypted.ciphertext.clone(),
            nonce: encrypted.nonce.clone(),
        };

        // Deactivate old key
        if let Some(old_key) = self.keys.get_mut(key_id) {
            old_key.metadata.active = false;
        }

        let new_id = new_metadata.id.clone();
        self.keys.insert(new_id.clone(), new_stored_key);

        Ok(new_id)
    }

    /// Delete a key
    pub fn delete_key(&mut self, key_id: &str) -> KeyStoreResult<()> {
        self.keys
            .remove(key_id)
            .ok_or_else(|| KeyStoreError::KeyNotFound(key_id.to_string()))?;

        Ok(())
    }

    /// Set key expiration
    pub fn set_expiration(&mut self, key_id: &str, expires_at: DateTime<Utc>) -> KeyStoreResult<()> {
        let key = self.keys
            .get_mut(key_id)
            .ok_or_else(|| KeyStoreError::KeyNotFound(key_id.to_string()))?;

        key.metadata.expires_at = Some(expires_at);
        Ok(())
    }

    /// Get all active keys for a specific purpose
    pub fn get_keys_by_purpose(&self, purpose: KeyPurpose) -> Vec<KeyMetadata> {
        self.keys
            .values()
            .filter(|k| k.metadata.purpose == purpose && k.metadata.active && !k.metadata.is_expired())
            .map(|k| k.metadata.clone())
            .collect()
    }

    /// Export key store to encrypted bytes (for backup)
    pub fn export(&self) -> KeyStoreResult<Vec<u8>> {
        let data = bincode::serialize(&self.keys)
            .map_err(|e| KeyStoreError::SerializationError(e.to_string()))?;

        Ok(data)
    }

    /// Import key store from encrypted bytes (for recovery)
    pub fn import(&mut self, data: &[u8]) -> KeyStoreResult<()> {
        let keys: HashMap<String, StoredKey> = bincode::deserialize(data)
            .map_err(|e| KeyStoreError::SerializationError(e.to_string()))?;

        self.keys = keys;
        Ok(())
    }
}

impl Drop for KeyStore {
    fn drop(&mut self) {
        self.master_key.zeroize();
    }
}

impl std::fmt::Debug for KeyStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyStore")
            .field("key_count", &self.keys.len())
            .field("master_key", &"[REDACTED]")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keystore_creation() {
        let store = KeyStore::new(b"test_password").unwrap();
        assert_eq!(store.keys.len(), 0);
    }

    #[test]
    fn test_store_and_retrieve_key() {
        let mut store = KeyStore::new(b"test_password").unwrap();

        let key_material = b"secret_key_material_32_bytes!!!";
        let key_id = store.store_key(
            "test_key".to_string(),
            KeyPurpose::Encryption,
            "AES-256".to_string(),
            key_material,
            None,
        ).unwrap();

        let retrieved = store.get_key(&key_id).unwrap();
        assert_eq!(retrieved.as_bytes(), key_material);
        assert_eq!(retrieved.metadata.name, "test_key");
    }

    #[test]
    fn test_get_key_by_name() {
        let mut store = KeyStore::new(b"test_password").unwrap();

        let key_material = b"secret_key_material_32_bytes!!!";
        store.store_key(
            "my_encryption_key".to_string(),
            KeyPurpose::Encryption,
            "AES-256".to_string(),
            key_material,
            None,
        ).unwrap();

        let retrieved = store.get_key_by_name("my_encryption_key").unwrap();
        assert_eq!(retrieved.as_bytes(), key_material);
    }

    #[test]
    fn test_key_rotation() {
        let mut store = KeyStore::new(b"test_password").unwrap();

        let old_key = b"old_key_material_32_bytes!!!!!!";
        let key_id = store.store_key(
            "rotating_key".to_string(),
            KeyPurpose::Encryption,
            "AES-256".to_string(),
            old_key,
            None,
        ).unwrap();

        let new_key = b"new_key_material_32_bytes!!!!!!";
        let new_key_id = store.rotate_key(&key_id, new_key).unwrap();

        // New key should have higher version
        let retrieved = store.get_key(&new_key_id).unwrap();
        assert_eq!(retrieved.metadata.version, 2);
        assert_eq!(retrieved.as_bytes(), new_key);

        // Old key should be deactivated
        let old_stored = store.keys.get(&key_id).unwrap();
        assert!(!old_stored.metadata.active);
    }

    #[test]
    fn test_key_deletion() {
        let mut store = KeyStore::new(b"test_password").unwrap();

        let key_material = b"secret_key_material_32_bytes!!!";
        let key_id = store.store_key(
            "temp_key".to_string(),
            KeyPurpose::Encryption,
            "AES-256".to_string(),
            key_material,
            None,
        ).unwrap();

        assert!(store.delete_key(&key_id).is_ok());
        assert!(store.get_key(&key_id).is_err());
    }

    #[test]
    fn test_key_expiration() {
        let mut store = KeyStore::new(b"test_password").unwrap();

        let key_material = b"secret_key_material_32_bytes!!!";
        let key_id = store.store_key(
            "expiring_key".to_string(),
            KeyPurpose::Encryption,
            "AES-256".to_string(),
            key_material,
            None,
        ).unwrap();

        // Set expiration to the past
        let past = Utc::now() - Duration::days(1);
        store.set_expiration(&key_id, past).unwrap();

        // Should fail to retrieve expired key
        let result = store.get_key(&key_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_keys() {
        let mut store = KeyStore::new(b"test_password").unwrap();

        store.store_key(
            "key1".to_string(),
            KeyPurpose::Encryption,
            "AES-256".to_string(),
            b"key1_material_32_bytes!!!!!!!!!",
            None,
        ).unwrap();

        store.store_key(
            "key2".to_string(),
            KeyPurpose::Signing,
            "Ed25519".to_string(),
            b"key2_material_32_bytes!!!!!!!!!",
            None,
        ).unwrap();

        let keys = store.list_keys();
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn test_get_keys_by_purpose() {
        let mut store = KeyStore::new(b"test_password").unwrap();

        store.store_key(
            "enc_key".to_string(),
            KeyPurpose::Encryption,
            "AES-256".to_string(),
            b"enc_key_material_32_bytes!!!!!!",
            None,
        ).unwrap();

        store.store_key(
            "sign_key".to_string(),
            KeyPurpose::Signing,
            "Ed25519".to_string(),
            b"sign_key_material_32_bytes!!!!!",
            None,
        ).unwrap();

        let enc_keys = store.get_keys_by_purpose(KeyPurpose::Encryption);
        assert_eq!(enc_keys.len(), 1);
        assert_eq!(enc_keys[0].name, "enc_key");
    }

    #[test]
    fn test_export_import() {
        let mut store = KeyStore::new(b"test_password").unwrap();

        let key_material = b"secret_key_material_32_bytes!!!";
        store.store_key(
            "test_key".to_string(),
            KeyPurpose::Encryption,
            "AES-256".to_string(),
            key_material,
            None,
        ).unwrap();

        let exported = store.export().unwrap();

        let mut new_store = KeyStore::new(b"test_password").unwrap();
        new_store.import(&exported).unwrap();

        let retrieved = new_store.get_key_by_name("test_key").unwrap();
        assert_eq!(retrieved.as_bytes(), key_material);
    }
}
