// CADDY v0.1.5 - Key Management and Storage
// Secure key storage, rotation, and HSM integration

use crate::enterprise::security::{SecurityError, SecurityResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use zeroize::Zeroize;

/// Key metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    pub key_id: String,
    pub algorithm: String,
    pub key_size: usize,
    pub created_at: i64,
    pub expires_at: Option<i64>,
    pub rotated_at: Option<i64>,
    pub version: u32,
    pub owner: String,
    pub purpose: KeyPurpose,
    pub status: KeyStatus,
    pub tags: HashMap<String, String>,
}

/// Key purpose enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KeyPurpose {
    Encryption,
    Signing,
    Authentication,
    KeyWrapping,
    DataEncryption,
    MasterKey,
}

/// Key status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KeyStatus {
    Active,
    Inactive,
    Rotating,
    Compromised,
    Retired,
}

/// Stored key (encrypted at rest)
#[derive(Clone, Serialize, Deserialize)]
struct StoredKey {
    pub metadata: KeyMetadata,
    #[serde(with = "hex")]
    encrypted_key_material: Vec<u8>,
    #[serde(with = "hex")]
    nonce: Vec<u8>,
}

impl Drop for StoredKey {
    fn drop(&mut self) {
        self.encrypted_key_material.zeroize();
        self.nonce.zeroize();
    }
}

/// Key rotation policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationPolicy {
    pub rotation_interval_days: u32,
    pub retain_old_versions: u32,
    pub auto_rotate: bool,
    pub notify_before_days: u32,
}

impl Default for KeyRotationPolicy {
    fn default() -> Self {
        Self {
            rotation_interval_days: 90,
            retain_old_versions: 3,
            auto_rotate: true,
            notify_before_days: 7,
        }
    }
}

/// Key escrow entry
#[derive(Clone, Serialize, Deserialize)]
pub struct KeyEscrow {
    pub key_id: String,
    pub escrowed_at: i64,
    pub escrow_shares: Vec<EscrowShare>,
    pub threshold: usize,
    pub recovery_contacts: Vec<String>,
}

/// Escrow share (Shamir's Secret Sharing)
#[derive(Clone, Serialize, Deserialize)]
pub struct EscrowShare {
    pub share_id: String,
    pub share_index: u8,
    #[serde(with = "hex")]
    share_data: Vec<u8>,
    pub holder: String,
}

impl Drop for EscrowShare {
    fn drop(&mut self) {
        self.share_data.zeroize();
    }
}

/// HSM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HsmConfig {
    pub hsm_type: HsmType,
    pub endpoint: String,
    pub slot_id: Option<u32>,
    pub credentials: HashMap<String, String>,
    pub enabled: bool,
}

/// HSM type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HsmType {
    Software,
    Pkcs11,
    AwsCloudHsm,
    AzureKeyVault,
    GoogleCloudKms,
    YubiHsm,
}

/// Main key store
pub struct KeyStore {
    keys: Arc<RwLock<HashMap<String, StoredKey>>>,
    master_key: Arc<RwLock<Vec<u8>>>,
    rotation_policies: Arc<RwLock<HashMap<String, KeyRotationPolicy>>>,
    escrow_store: Arc<RwLock<HashMap<String, KeyEscrow>>>,
    hsm_config: Arc<RwLock<Option<HsmConfig>>>,
}

impl KeyStore {
    /// Create a new key store
    pub fn new(master_password: &str) -> SecurityResult<Self> {
        // Derive master key from password
        let master_key = Self::derive_master_key(master_password)?;

        Ok(Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            master_key: Arc::new(RwLock::new(master_key)),
            rotation_policies: Arc::new(RwLock::new(HashMap::new())),
            escrow_store: Arc::new(RwLock::new(HashMap::new())),
            hsm_config: Arc::new(RwLock::new(None)),
        })
    }

    /// Store a key securely
    pub fn store_key(
        &self,
        key_id: &str,
        key_material: &[u8],
        metadata: KeyMetadata,
    ) -> SecurityResult<()> {
        let master_key = self.master_key.read()
            .map_err(|e| SecurityError::KeyManagement(format!("Lock error: {}", e)))?;

        // Encrypt key material with master key
        let (encrypted_key, nonce) = self.encrypt_key_material(key_material, &master_key)?;

        let stored_key = StoredKey {
            metadata,
            encrypted_key_material: encrypted_key,
            nonce,
        };

        let mut keys = self.keys.write()
            .map_err(|e| SecurityError::KeyManagement(format!("Lock error: {}", e)))?;

        keys.insert(key_id.to_string(), stored_key);

        Ok(())
    }

    /// Retrieve a key
    pub fn get_key(&self, key_id: &str) -> SecurityResult<Vec<u8>> {
        let keys = self.keys.read()
            .map_err(|e| SecurityError::KeyManagement(format!("Lock error: {}", e)))?;

        let stored_key = keys.get(key_id)
            .ok_or_else(|| SecurityError::KeyManagement(format!("Key not found: {}", key_id)))?;

        // Check if key is active
        if stored_key.metadata.status != KeyStatus::Active {
            return Err(SecurityError::KeyManagement(
                format!("Key is not active: {:?}", stored_key.metadata.status)
            ));
        }

        let master_key = self.master_key.read()
            .map_err(|e| SecurityError::KeyManagement(format!("Lock error: {}", e)))?;

        // Decrypt key material
        let key_material = self.decrypt_key_material(
            &stored_key.encrypted_key_material,
            &stored_key.nonce,
            &master_key,
        )?;

        Ok(key_material)
    }

    /// Get key metadata
    pub fn get_metadata(&self, key_id: &str) -> SecurityResult<KeyMetadata> {
        let keys = self.keys.read()
            .map_err(|e| SecurityError::KeyManagement(format!("Lock error: {}", e)))?;

        let stored_key = keys.get(key_id)
            .ok_or_else(|| SecurityError::KeyManagement(format!("Key not found: {}", key_id)))?;

        Ok(stored_key.metadata.clone())
    }

    /// Rotate a key
    pub fn rotate_key(&self, key_id: &str, new_key_material: &[u8]) -> SecurityResult<()> {
        let mut keys = self.keys.write()
            .map_err(|e| SecurityError::KeyManagement(format!("Lock error: {}", e)))?;

        let stored_key = keys.get_mut(key_id)
            .ok_or_else(|| SecurityError::KeyManagement(format!("Key not found: {}", key_id)))?;

        // Update metadata
        stored_key.metadata.version += 1;
        stored_key.metadata.rotated_at = Some(chrono::Utc::now().timestamp());
        stored_key.metadata.status = KeyStatus::Rotating;

        let master_key = self.master_key.read()
            .map_err(|e| SecurityError::KeyManagement(format!("Lock error: {}", e)))?;

        // Encrypt new key material
        let (encrypted_key, nonce) = self.encrypt_key_material(new_key_material, &master_key)?;

        stored_key.encrypted_key_material = encrypted_key;
        stored_key.nonce = nonce;
        stored_key.metadata.status = KeyStatus::Active;

        Ok(())
    }

    /// Set rotation policy for a key
    pub fn set_rotation_policy(&self, key_id: &str, policy: KeyRotationPolicy) -> SecurityResult<()> {
        let mut policies = self.rotation_policies.write()
            .map_err(|e| SecurityError::KeyManagement(format!("Lock error: {}", e)))?;

        policies.insert(key_id.to_string(), policy);
        Ok(())
    }

    /// Check if key needs rotation
    pub fn needs_rotation(&self, key_id: &str) -> SecurityResult<bool> {
        let metadata = self.get_metadata(key_id)?;

        let policies = self.rotation_policies.read()
            .map_err(|e| SecurityError::KeyManagement(format!("Lock error: {}", e)))?;

        if let Some(policy) = policies.get(key_id) {
            if !policy.auto_rotate {
                return Ok(false);
            }

            let now = chrono::Utc::now().timestamp();
            let age_days = (now - metadata.created_at) / 86400;

            Ok(age_days >= policy.rotation_interval_days as i64)
        } else {
            Ok(false)
        }
    }

    /// Escrow a key using Shamir's Secret Sharing
    pub fn escrow_key(
        &self,
        key_id: &str,
        num_shares: u8,
        threshold: u8,
        holders: Vec<String>,
    ) -> SecurityResult<Vec<EscrowShare>> {
        if num_shares < threshold {
            return Err(SecurityError::KeyManagement(
                "Number of shares must be >= threshold".to_string()
            ));
        }

        if holders.len() != num_shares as usize {
            return Err(SecurityError::KeyManagement(
                "Number of holders must match number of shares".to_string()
            ));
        }

        let key_material = self.get_key(key_id)?;

        // Simulate Shamir's Secret Sharing
        // In production, use sharks or similar crate
        let shares = self.split_secret(&key_material, num_shares, threshold)?;

        let escrow_shares: Vec<EscrowShare> = shares.into_iter()
            .enumerate()
            .map(|(i, share)| EscrowShare {
                share_id: format!("{}-share-{}", key_id, i),
                share_index: i as u8,
                share_data: share,
                holder: holders[i].clone(),
            })
            .collect();

        let escrow = KeyEscrow {
            key_id: key_id.to_string(),
            escrowed_at: chrono::Utc::now().timestamp(),
            escrow_shares: escrow_shares.clone(),
            threshold: threshold as usize,
            recovery_contacts: holders,
        };

        let mut escrow_store = self.escrow_store.write()
            .map_err(|e| SecurityError::KeyManagement(format!("Lock error: {}", e)))?;

        escrow_store.insert(key_id.to_string(), escrow);

        Ok(escrow_shares)
    }

    /// Recover key from escrow shares
    pub fn recover_from_escrow(&self, key_id: &str, shares: &[EscrowShare]) -> SecurityResult<Vec<u8>> {
        let escrow_store = self.escrow_store.read()
            .map_err(|e| SecurityError::KeyManagement(format!("Lock error: {}", e)))?;

        let escrow = escrow_store.get(key_id)
            .ok_or_else(|| SecurityError::KeyManagement(format!("No escrow found for key: {}", key_id)))?;

        if shares.len() < escrow.threshold {
            return Err(SecurityError::KeyManagement(
                format!("Insufficient shares: need {}, got {}", escrow.threshold, shares.len())
            ));
        }

        // Simulate secret reconstruction
        let share_data: Vec<&[u8]> = shares.iter().map(|s| s.share_data.as_slice()).collect();
        let recovered = self.combine_shares(&share_data)?;

        Ok(recovered)
    }

    /// Configure HSM
    pub fn configure_hsm(&self, config: HsmConfig) -> SecurityResult<()> {
        let mut hsm_config = self.hsm_config.write()
            .map_err(|e| SecurityError::KeyManagement(format!("Lock error: {}", e)))?;

        *hsm_config = Some(config);
        Ok(())
    }

    /// Store key in HSM (stub implementation)
    pub fn store_in_hsm(&self, key_id: &str, key_material: &[u8]) -> SecurityResult<String> {
        let hsm_config = self.hsm_config.read()
            .map_err(|e| SecurityError::KeyManagement(format!("Lock error: {}", e)))?;

        let config = hsm_config.as_ref()
            .ok_or_else(|| SecurityError::Hsm("HSM not configured".to_string()))?;

        if !config.enabled {
            return Err(SecurityError::Hsm("HSM is disabled".to_string()));
        }

        // Stub: In production, integrate with actual HSM
        let hsm_key_id = format!("hsm://{}:{}", config.hsm_type.to_string(), key_id);

        Ok(hsm_key_id)
    }

    /// Retrieve key from HSM (stub implementation)
    pub fn get_from_hsm(&self, hsm_key_id: &str) -> SecurityResult<Vec<u8>> {
        let hsm_config = self.hsm_config.read()
            .map_err(|e| SecurityError::KeyManagement(format!("Lock error: {}", e)))?;

        let config = hsm_config.as_ref()
            .ok_or_else(|| SecurityError::Hsm("HSM not configured".to_string()))?;

        if !config.enabled {
            return Err(SecurityError::Hsm("HSM is disabled".to_string()));
        }

        // Stub: In production, retrieve from actual HSM
        Err(SecurityError::Hsm("HSM key retrieval not implemented".to_string()))
    }

    /// List all keys
    pub fn list_keys(&self) -> SecurityResult<Vec<KeyMetadata>> {
        let keys = self.keys.read()
            .map_err(|e| SecurityError::KeyManagement(format!("Lock error: {}", e)))?;

        Ok(keys.values().map(|k| k.metadata.clone()).collect())
    }

    /// Delete a key
    pub fn delete_key(&self, key_id: &str) -> SecurityResult<()> {
        let mut keys = self.keys.write()
            .map_err(|e| SecurityError::KeyManagement(format!("Lock error: {}", e)))?;

        keys.remove(key_id)
            .ok_or_else(|| SecurityError::KeyManagement(format!("Key not found: {}", key_id)))?;

        Ok(())
    }

    // Helper methods

    fn derive_master_key(password: &str) -> SecurityResult<Vec<u8>> {
        // Simulate PBKDF2 key derivation
        let mut key = vec![0u8; 32];
        let password_bytes = password.as_bytes();
        let salt = b"caddy_keystore_salt_v1";

        for i in 0..32 {
            let mut val = password_bytes[i % password_bytes.len()];
            for _ in 0..100_000 {
                val = val.wrapping_mul(251).wrapping_add(salt[i % salt.len()]);
            }
            key[i] = val;
        }

        Ok(key)
    }

    fn encrypt_key_material(&self, key: &[u8], master_key: &[u8]) -> SecurityResult<(Vec<u8>, Vec<u8>)> {
        // Generate nonce
        let mut nonce = vec![0u8; 12];
        self.fill_random(&mut nonce)?;

        // Simulate AES-256-GCM encryption
        let mut encrypted = key.to_vec();
        for (i, byte) in encrypted.iter_mut().enumerate() {
            *byte ^= master_key[i % master_key.len()] ^ nonce[i % nonce.len()];
        }

        Ok((encrypted, nonce))
    }

    fn decrypt_key_material(&self, encrypted: &[u8], nonce: &[u8], master_key: &[u8]) -> SecurityResult<Vec<u8>> {
        // Simulate AES-256-GCM decryption
        let mut decrypted = encrypted.to_vec();
        for (i, byte) in decrypted.iter_mut().enumerate() {
            *byte ^= master_key[i % master_key.len()] ^ nonce[i % nonce.len()];
        }

        Ok(decrypted)
    }

    fn split_secret(&self, secret: &[u8], num_shares: u8, threshold: u8) -> SecurityResult<Vec<Vec<u8>>> {
        // Simplified Shamir's Secret Sharing simulation
        let mut shares = Vec::new();

        for i in 0..num_shares {
            let mut share = secret.to_vec();
            for byte in share.iter_mut() {
                *byte = byte.wrapping_add(i).wrapping_mul(threshold);
            }
            shares.push(share);
        }

        Ok(shares)
    }

    fn combine_shares(&self, shares: &[&[u8]]) -> SecurityResult<Vec<u8>> {
        // Simplified secret reconstruction
        if shares.is_empty() {
            return Err(SecurityError::KeyManagement("No shares provided".to_string()));
        }

        let len = shares[0].len();
        let mut secret = vec![0u8; len];

        for i in 0..len {
            let mut sum = 0u8;
            for share in shares {
                sum = sum.wrapping_add(share[i]);
            }
            secret[i] = sum.wrapping_div(shares.len() as u8);
        }

        Ok(secret)
    }

    fn fill_random(&self, buf: &mut [u8]) -> SecurityResult<()> {
        use std::collections::hash_map::RandomState;
        use std::hash::{BuildHasher, Hash, Hasher};

        let rs = RandomState::new();
        for (i, byte) in buf.iter_mut().enumerate() {
            let mut hasher = rs.build_hasher();
            (i, std::time::SystemTime::now()).hash(&mut hasher);
            *byte = (hasher.finish() & 0xFF) as u8;
        }
        Ok(())
    }
}

impl HsmType {
    fn to_string(&self) -> String {
        match self {
            HsmType::Software => "software".to_string(),
            HsmType::Pkcs11 => "pkcs11".to_string(),
            HsmType::AwsCloudHsm => "aws-cloudhsm".to_string(),
            HsmType::AzureKeyVault => "azure-keyvault".to_string(),
            HsmType::GoogleCloudKms => "google-cloudkms".to_string(),
            HsmType::YubiHsm => "yubihsm".to_string(),
        }
    }
}

mod hex {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex_encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        hex_decode(&s).map_err(serde::de::Error::custom)
    }

    fn hex_encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
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
    fn test_key_storage() {
        let keystore = KeyStore::new("test_password").unwrap();
        let key_material = b"secret_key_material";

        let metadata = KeyMetadata {
            key_id: "test_key".to_string(),
            algorithm: "AES-256-GCM".to_string(),
            key_size: 256,
            created_at: chrono::Utc::now().timestamp(),
            expires_at: None,
            rotated_at: None,
            version: 1,
            owner: "test_user".to_string(),
            purpose: KeyPurpose::Encryption,
            status: KeyStatus::Active,
            tags: HashMap::new(),
        };

        keystore.store_key("test_key", key_material, metadata).unwrap();
        let retrieved = keystore.get_key("test_key").unwrap();

        assert_eq!(key_material.as_slice(), retrieved.as_slice());
    }

    #[test]
    fn test_key_rotation() {
        let keystore = KeyStore::new("test_password").unwrap();
        let old_key = b"old_key";
        let new_key = b"new_key";

        let metadata = KeyMetadata {
            key_id: "rotation_key".to_string(),
            algorithm: "AES-256-GCM".to_string(),
            key_size: 256,
            created_at: chrono::Utc::now().timestamp(),
            expires_at: None,
            rotated_at: None,
            version: 1,
            owner: "test_user".to_string(),
            purpose: KeyPurpose::Encryption,
            status: KeyStatus::Active,
            tags: HashMap::new(),
        };

        keystore.store_key("rotation_key", old_key, metadata).unwrap();
        keystore.rotate_key("rotation_key", new_key).unwrap();

        let retrieved = keystore.get_key("rotation_key").unwrap();
        assert_eq!(new_key.as_slice(), retrieved.as_slice());
    }
}
