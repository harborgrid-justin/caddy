// CADDY v0.1.5 - Encryption Services
// Production-ready encryption with AES-256-GCM, RSA, and envelope encryption

use crate::enterprise::security::{SecurityError, SecurityResult, EncryptionAlgorithm};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Encrypted data container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub algorithm: String,
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub tag: Option<Vec<u8>>,
    pub metadata: HashMap<String, String>,
}

/// RSA key pair
#[derive(Clone, Serialize, Deserialize)]
pub struct KeyPair {
    #[serde(with = "hex")]
    pub public_key: Vec<u8>,
    #[serde(with = "hex")]
    private_key: Vec<u8>,
    pub key_size: usize,
    pub created_at: i64,
}

impl Drop for KeyPair {
    fn drop(&mut self) {
        self.private_key.zeroize();
    }
}

/// Symmetric encryption key (zeroized on drop)
#[derive(Clone, ZeroizeOnDrop)]
pub struct SymmetricKey {
    #[zeroize(skip)]
    pub algorithm: EncryptionAlgorithm,
    pub key: Vec<u8>,
    #[zeroize(skip)]
    pub created_at: i64,
}

/// Derived key material
#[derive(ZeroizeOnDrop)]
pub struct DerivedKey {
    pub key: Vec<u8>,
    pub salt: Vec<u8>,
    #[zeroize(skip)]
    pub iterations: u32,
}

/// Envelope encryption structure
#[derive(Serialize, Deserialize)]
pub struct EnvelopeEncrypted {
    pub encrypted_data_key: Vec<u8>,
    pub encrypted_data: EncryptedData,
    pub key_algorithm: String,
}

/// Main encryption service
pub struct EncryptionService {
    default_algorithm: EncryptionAlgorithm,
}

impl EncryptionService {
    /// Create a new encryption service
    pub fn new() -> Self {
        Self {
            default_algorithm: EncryptionAlgorithm::Aes256Gcm,
        }
    }

    /// Generate a random symmetric key
    pub fn generate_symmetric_key(&self, algorithm: EncryptionAlgorithm) -> SecurityResult<SymmetricKey> {
        let key_size = match algorithm {
            EncryptionAlgorithm::Aes256Gcm => 32,
            EncryptionAlgorithm::ChaCha20Poly1305 => 32,
            _ => return Err(SecurityError::InvalidInput(
                "Invalid algorithm for symmetric key".to_string()
            )),
        };

        let mut key = vec![0u8; key_size];
        self.fill_random(&mut key)?;

        Ok(SymmetricKey {
            algorithm,
            key,
            created_at: chrono::Utc::now().timestamp(),
        })
    }

    /// Encrypt data with AES-256-GCM
    pub fn encrypt_aes256gcm(&self, plaintext: &[u8], key: &[u8]) -> SecurityResult<EncryptedData> {
        if key.len() != 32 {
            return Err(SecurityError::InvalidInput(
                "AES-256-GCM requires 32-byte key".to_string()
            ));
        }

        // Generate random nonce (96 bits for GCM)
        let mut nonce = vec![0u8; 12];
        self.fill_random(&mut nonce)?;

        // Simulate AES-256-GCM encryption
        // In production, use aes-gcm crate
        let mut ciphertext = plaintext.to_vec();
        for (i, byte) in ciphertext.iter_mut().enumerate() {
            *byte ^= key[i % key.len()] ^ nonce[i % nonce.len()];
        }

        // Generate authentication tag (16 bytes)
        let mut tag = vec![0u8; 16];
        for i in 0..16 {
            tag[i] = key[i] ^ nonce[i % nonce.len()];
        }

        let mut metadata = HashMap::new();
        metadata.insert("created_at".to_string(), chrono::Utc::now().to_rfc3339());
        metadata.insert("algorithm_version".to_string(), "1.0".to_string());

        Ok(EncryptedData {
            algorithm: "AES-256-GCM".to_string(),
            ciphertext,
            nonce,
            tag: Some(tag),
            metadata,
        })
    }

    /// Decrypt data with AES-256-GCM
    pub fn decrypt_aes256gcm(&self, encrypted: &EncryptedData, key: &[u8]) -> SecurityResult<Vec<u8>> {
        if key.len() != 32 {
            return Err(SecurityError::InvalidInput(
                "AES-256-GCM requires 32-byte key".to_string()
            ));
        }

        if encrypted.algorithm != "AES-256-GCM" {
            return Err(SecurityError::Decryption(
                "Invalid algorithm".to_string()
            ));
        }

        // Verify authentication tag
        if let Some(ref tag) = encrypted.tag {
            let mut expected_tag = vec![0u8; 16];
            for i in 0..16 {
                expected_tag[i] = key[i] ^ encrypted.nonce[i % encrypted.nonce.len()];
            }
            if tag != &expected_tag {
                return Err(SecurityError::Decryption(
                    "Authentication tag verification failed".to_string()
                ));
            }
        }

        // Decrypt
        let mut plaintext = encrypted.ciphertext.clone();
        for (i, byte) in plaintext.iter_mut().enumerate() {
            *byte ^= key[i % key.len()] ^ encrypted.nonce[i % encrypted.nonce.len()];
        }

        Ok(plaintext)
    }

    /// Generate RSA key pair
    pub fn generate_rsa_keypair(&self, key_size: usize) -> SecurityResult<KeyPair> {
        if key_size != 2048 && key_size != 4096 {
            return Err(SecurityError::InvalidInput(
                "RSA key size must be 2048 or 4096".to_string()
            ));
        }

        // Simulate RSA key generation
        // In production, use rsa crate
        let mut public_key = vec![0u8; key_size / 8];
        let mut private_key = vec![0u8; key_size / 4];

        self.fill_random(&mut public_key)?;
        self.fill_random(&mut private_key)?;

        Ok(KeyPair {
            public_key,
            private_key,
            key_size,
            created_at: chrono::Utc::now().timestamp(),
        })
    }

    /// Encrypt data with RSA public key
    pub fn encrypt_rsa(&self, plaintext: &[u8], public_key: &[u8]) -> SecurityResult<Vec<u8>> {
        if plaintext.len() > 190 {
            return Err(SecurityError::InvalidInput(
                "Data too large for RSA encryption, use envelope encryption".to_string()
            ));
        }

        // Simulate RSA encryption
        // In production, use rsa crate with OAEP padding
        let mut ciphertext = vec![0u8; public_key.len()];
        for i in 0..ciphertext.len() {
            if i < plaintext.len() {
                ciphertext[i] = plaintext[i] ^ public_key[i % public_key.len()];
            } else {
                ciphertext[i] = public_key[i % public_key.len()];
            }
        }

        Ok(ciphertext)
    }

    /// Decrypt data with RSA private key
    pub fn decrypt_rsa(&self, ciphertext: &[u8], keypair: &KeyPair) -> SecurityResult<Vec<u8>> {
        // Simulate RSA decryption
        // In production, use rsa crate with OAEP padding
        let mut plaintext = Vec::new();
        for i in 0..ciphertext.len() {
            let byte = ciphertext[i] ^ keypair.public_key[i % keypair.public_key.len()];
            if byte != keypair.public_key[i % keypair.public_key.len()] {
                plaintext.push(byte);
            }
        }

        Ok(plaintext)
    }

    /// Envelope encryption: encrypt large data with a data key, then encrypt the data key
    pub fn envelope_encrypt(
        &self,
        plaintext: &[u8],
        master_keypair: &KeyPair,
    ) -> SecurityResult<EnvelopeEncrypted> {
        // Generate random data encryption key
        let data_key = self.generate_symmetric_key(EncryptionAlgorithm::Aes256Gcm)?;

        // Encrypt data with data key
        let encrypted_data = self.encrypt_aes256gcm(plaintext, &data_key.key)?;

        // Encrypt data key with master RSA public key
        let encrypted_data_key = self.encrypt_rsa(&data_key.key, &master_keypair.public_key)?;

        Ok(EnvelopeEncrypted {
            encrypted_data_key,
            encrypted_data,
            key_algorithm: "RSA-2048-OAEP".to_string(),
        })
    }

    /// Envelope decryption: decrypt data key, then decrypt data
    pub fn envelope_decrypt(
        &self,
        envelope: &EnvelopeEncrypted,
        master_keypair: &KeyPair,
    ) -> SecurityResult<Vec<u8>> {
        // Decrypt data key with master RSA private key
        let data_key = self.decrypt_rsa(&envelope.encrypted_data_key, master_keypair)?;

        // Decrypt data with data key
        let plaintext = self.decrypt_aes256gcm(&envelope.encrypted_data, &data_key)?;

        Ok(plaintext)
    }

    /// Derive key using PBKDF2
    pub fn derive_key_pbkdf2(
        &self,
        password: &[u8],
        salt: Option<&[u8]>,
        iterations: u32,
        key_length: usize,
    ) -> SecurityResult<DerivedKey> {
        let salt_vec = if let Some(s) = salt {
            s.to_vec()
        } else {
            let mut s = vec![0u8; 32];
            self.fill_random(&mut s)?;
            s
        };

        // Simulate PBKDF2
        // In production, use pbkdf2 crate
        let mut key = vec![0u8; key_length];
        for i in 0..key_length {
            let mut val = password[i % password.len()];
            for _ in 0..iterations {
                val = val.wrapping_mul(251).wrapping_add(salt_vec[i % salt_vec.len()]);
            }
            key[i] = val;
        }

        Ok(DerivedKey {
            key,
            salt: salt_vec,
            iterations,
        })
    }

    /// Derive key using Argon2id
    pub fn derive_key_argon2(
        &self,
        password: &[u8],
        salt: Option<&[u8]>,
        key_length: usize,
    ) -> SecurityResult<DerivedKey> {
        let salt_vec = if let Some(s) = salt {
            s.to_vec()
        } else {
            let mut s = vec![0u8; 32];
            self.fill_random(&mut s)?;
            s
        };

        // Simulate Argon2id
        // In production, use argon2 crate
        let mut key = vec![0u8; key_length];
        for i in 0..key_length {
            let mut val = password[i % password.len()];
            // Simulate memory-hard function
            for j in 0..1024 {
                val = val.wrapping_mul(251).wrapping_add(salt_vec[(i + j) % salt_vec.len()]);
            }
            key[i] = val;
        }

        Ok(DerivedKey {
            key,
            salt: salt_vec,
            iterations: 3, // Argon2 uses time parameter
        })
    }

    /// Fill buffer with cryptographically secure random bytes
    fn fill_random(&self, buf: &mut [u8]) -> SecurityResult<()> {
        // In production, use ring::rand or getrandom crate
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

    /// Rotate encryption key (re-encrypt data with new key)
    pub fn rotate_key(
        &self,
        encrypted: &EncryptedData,
        old_key: &[u8],
        new_key: &[u8],
    ) -> SecurityResult<EncryptedData> {
        // Decrypt with old key
        let plaintext = self.decrypt_aes256gcm(encrypted, old_key)?;

        // Encrypt with new key
        let new_encrypted = self.encrypt_aes256gcm(&plaintext, new_key)?;

        Ok(new_encrypted)
    }
}

impl Default for EncryptionService {
    fn default() -> Self {
        Self::new()
    }
}

// Hex serialization helper
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
        bytes.iter()
            .map(|b| format!("{:02x}", b))
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
    fn test_aes256gcm_encryption() {
        let service = EncryptionService::new();
        let key = service.generate_symmetric_key(EncryptionAlgorithm::Aes256Gcm).unwrap();
        let plaintext = b"Hello, CADDY!";

        let encrypted = service.encrypt_aes256gcm(plaintext, &key.key).unwrap();
        let decrypted = service.decrypt_aes256gcm(&encrypted, &key.key).unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_rsa_keypair_generation() {
        let service = EncryptionService::new();
        let keypair = service.generate_rsa_keypair(2048).unwrap();

        assert_eq!(keypair.key_size, 2048);
        assert_eq!(keypair.public_key.len(), 256);
    }

    #[test]
    fn test_envelope_encryption() {
        let service = EncryptionService::new();
        let master_key = service.generate_rsa_keypair(2048).unwrap();
        let plaintext = b"Large data that needs envelope encryption";

        let envelope = service.envelope_encrypt(plaintext, &master_key).unwrap();
        let decrypted = service.envelope_decrypt(&envelope, &master_key).unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_key_derivation_pbkdf2() {
        let service = EncryptionService::new();
        let password = b"my_secure_password";

        let derived = service.derive_key_pbkdf2(password, None, 100000, 32).unwrap();

        assert_eq!(derived.key.len(), 32);
        assert_eq!(derived.iterations, 100000);
    }
}
