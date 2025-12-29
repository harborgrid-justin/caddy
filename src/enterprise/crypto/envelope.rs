//! # Envelope Encryption
//!
//! This module implements envelope encryption, a secure pattern for encrypting data
//! where:
//! 1. Data is encrypted with a Data Encryption Key (DEK)
//! 2. The DEK is encrypted with one or more Key Encryption Keys (KEK)
//! 3. Both are stored together in an "envelope"
//!
//! ## Benefits
//!
//! - Efficient for large data (symmetric encryption for data)
//! - Easy key rotation (re-encrypt DEK only, not data)
//! - Multi-recipient support (encrypt DEK with multiple KEKs)
//! - Separation of concerns (data key vs. master keys)
//!
//! ## Architecture
//!
//! ```text
//! Plaintext → [Encrypt with DEK] → Ciphertext
//!                                      ↓
//!     DEK → [Encrypt with KEK] → Encrypted DEK
//!                                      ↓
//!                                  Envelope
//!                         (Ciphertext + Encrypted DEKs)
//! ```

use serde::{Serialize, Deserialize};
use rand::{RngCore, rngs::OsRng};
use zeroize::Zeroize;
use thiserror::Error;
use std::collections::HashMap;

use super::symmetric::{Aes256GcmCipher, ChaCha20Poly1305Cipher, SymmetricError};
use super::asymmetric::{RsaKeyPair, EciesKeyPair, AsymmetricError};

/// Envelope encryption errors
#[derive(Error, Debug)]
pub enum EnvelopeError {
    /// No recipients specified
    #[error("No recipients specified for encryption")]
    NoRecipients,

    /// Recipient not found in envelope
    #[error("Recipient {0} not found in envelope")]
    RecipientNotFound(String),

    /// Failed to decrypt DEK
    #[error("Failed to decrypt DEK: {0}")]
    DekDecryptionFailed(String),

    /// Symmetric encryption error
    #[error("Symmetric encryption error: {0}")]
    SymmetricError(#[from] SymmetricError),

    /// Asymmetric encryption error
    #[error("Asymmetric encryption error: {0}")]
    AsymmetricError(#[from] AsymmetricError),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Invalid envelope format
    #[error("Invalid envelope format: {0}")]
    InvalidFormat(String),
}

pub type EnvelopeResult<T> = Result<T, EnvelopeError>;

/// Data Encryption Key (DEK) - used to encrypt the actual data
#[derive(Clone)]
struct DataEncryptionKey {
    key_material: Vec<u8>,
    algorithm: EncryptionAlgorithm,
}

impl DataEncryptionKey {
    /// Generate a new random DEK
    fn generate(algorithm: EncryptionAlgorithm) -> Self {
        let key_size = match algorithm {
            EncryptionAlgorithm::Aes256Gcm => 32,
            EncryptionAlgorithm::ChaCha20Poly1305 => 32,
        };

        let mut key_material = vec![0u8; key_size];
        OsRng.fill_bytes(&mut key_material);

        Self {
            key_material,
            algorithm,
        }
    }

    /// Create from existing key material
    fn from_bytes(key_material: Vec<u8>, algorithm: EncryptionAlgorithm) -> Self {
        Self {
            key_material,
            algorithm,
        }
    }

    /// Get key bytes
    fn as_bytes(&self) -> &[u8] {
        &self.key_material
    }
}

impl Drop for DataEncryptionKey {
    fn drop(&mut self) {
        self.key_material.zeroize();
    }
}

impl std::fmt::Debug for DataEncryptionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataEncryptionKey")
            .field("algorithm", &self.algorithm)
            .field("key_material", &"[REDACTED]")
            .finish()
    }
}

/// Encryption algorithm for data
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum EncryptionAlgorithm {
    /// AES-256 in GCM mode
    Aes256Gcm,
    /// ChaCha20-Poly1305
    ChaCha20Poly1305,
}

/// Key encryption method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyEncryptionMethod {
    /// RSA-OAEP with SHA-256
    RsaOaep,
    /// ECIES with Curve25519
    Ecies,
}

/// Encrypted DEK for a specific recipient
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedDek {
    /// Recipient identifier (e.g., key ID, user ID)
    pub recipient_id: String,
    /// Key encryption method used
    pub method: KeyEncryptionMethod,
    /// Encrypted DEK material
    pub encrypted_key: Vec<u8>,
}

impl Drop for EncryptedDek {
    fn drop(&mut self) {
        self.encrypted_key.zeroize();
    }
}

/// Envelope containing encrypted data and DEKs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    /// Encryption algorithm used for data
    pub algorithm: EncryptionAlgorithm,
    /// Encrypted data ciphertext
    pub ciphertext: Vec<u8>,
    /// Nonce used for data encryption
    pub nonce: Vec<u8>,
    /// Optional associated data (authenticated but not encrypted)
    pub associated_data: Vec<u8>,
    /// Encrypted DEKs for each recipient
    pub encrypted_deks: Vec<EncryptedDek>,
    /// Optional metadata
    pub metadata: HashMap<String, String>,
}

impl Envelope {
    /// Serialize envelope to bytes
    pub fn to_bytes(&self) -> EnvelopeResult<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| EnvelopeError::SerializationError(e.to_string()))
    }

    /// Deserialize envelope from bytes
    pub fn from_bytes(data: &[u8]) -> EnvelopeResult<Self> {
        bincode::deserialize(data)
            .map_err(|e| EnvelopeError::SerializationError(e.to_string()))
    }

    /// Add custom metadata
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Get list of recipient IDs
    pub fn recipient_ids(&self) -> Vec<String> {
        self.encrypted_deks.iter().map(|dek| dek.recipient_id.clone()).collect()
    }
}

/// Builder for creating envelopes with multiple recipients
pub struct EnvelopeBuilder {
    algorithm: EncryptionAlgorithm,
    recipients: Vec<(String, Recipient)>,
    metadata: HashMap<String, String>,
}

/// Recipient information
pub enum Recipient {
    /// RSA public key
    Rsa(rsa::RsaPublicKey),
    /// ECIES public key
    Ecies(x25519_dalek::PublicKey),
}

impl EnvelopeBuilder {
    /// Create a new envelope builder
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use caddy::enterprise::crypto::envelope::{EnvelopeBuilder, EncryptionAlgorithm};
    ///
    /// let builder = EnvelopeBuilder::new(EncryptionAlgorithm::Aes256Gcm);
    /// ```
    pub fn new(algorithm: EncryptionAlgorithm) -> Self {
        Self {
            algorithm,
            recipients: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add an RSA recipient
    pub fn add_rsa_recipient(mut self, recipient_id: String, public_key: rsa::RsaPublicKey) -> Self {
        self.recipients.push((recipient_id, Recipient::Rsa(public_key)));
        self
    }

    /// Add an ECIES recipient
    pub fn add_ecies_recipient(mut self, recipient_id: String, public_key: x25519_dalek::PublicKey) -> Self {
        self.recipients.push((recipient_id, Recipient::Ecies(public_key)));
        self
    }

    /// Add metadata
    pub fn add_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Encrypt data and create envelope
    ///
    /// # Arguments
    ///
    /// * `plaintext` - Data to encrypt
    /// * `associated_data` - Optional authenticated but unencrypted data
    pub fn encrypt(self, plaintext: &[u8], associated_data: Option<&[u8]>) -> EnvelopeResult<Envelope> {
        if self.recipients.is_empty() {
            return Err(EnvelopeError::NoRecipients);
        }

        // Generate DEK
        let dek = DataEncryptionKey::generate(self.algorithm);

        // Encrypt data with DEK
        let (ciphertext, nonce) = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                let cipher = Aes256GcmCipher::new(dek.as_bytes())?;
                let encrypted = cipher.encrypt(plaintext, associated_data)?;
                (encrypted.ciphertext.clone(), encrypted.nonce.clone())
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                let cipher = ChaCha20Poly1305Cipher::new(dek.as_bytes())?;
                let encrypted = cipher.encrypt(plaintext, associated_data)?;
                (encrypted.ciphertext.clone(), encrypted.nonce.clone())
            }
        };

        // Encrypt DEK for each recipient
        let mut encrypted_deks = Vec::new();
        for (recipient_id, recipient) in self.recipients {
            let encrypted_dek = match recipient {
                Recipient::Rsa(public_key) => {
                    let encrypted_key = RsaKeyPair::encrypt(&public_key, dek.as_bytes())?;
                    EncryptedDek {
                        recipient_id,
                        method: KeyEncryptionMethod::RsaOaep,
                        encrypted_key,
                    }
                }
                Recipient::Ecies(public_key) => {
                    let encrypted = EciesKeyPair::encrypt(&public_key, dek.as_bytes())?;
                    EncryptedDek {
                        recipient_id,
                        method: KeyEncryptionMethod::Ecies,
                        encrypted_key: encrypted.to_bytes(),
                    }
                }
            };
            encrypted_deks.push(encrypted_dek);
        }

        Ok(Envelope {
            algorithm: self.algorithm,
            ciphertext,
            nonce,
            associated_data: associated_data.unwrap_or(b"").to_vec(),
            encrypted_deks,
            metadata: self.metadata,
        })
    }
}

/// Decrypt an envelope using RSA private key
///
/// # Arguments
///
/// * `envelope` - The envelope to decrypt
/// * `recipient_id` - The recipient ID to decrypt for
/// * `rsa_keypair` - The RSA private key
pub fn decrypt_with_rsa(
    envelope: &Envelope,
    recipient_id: &str,
    rsa_keypair: &RsaKeyPair,
) -> EnvelopeResult<Vec<u8>> {
    // Find the encrypted DEK for this recipient
    let encrypted_dek = envelope
        .encrypted_deks
        .iter()
        .find(|dek| dek.recipient_id == recipient_id && matches!(dek.method, KeyEncryptionMethod::RsaOaep))
        .ok_or_else(|| EnvelopeError::RecipientNotFound(recipient_id.to_string()))?;

    // Decrypt DEK
    let dek_bytes = rsa_keypair
        .decrypt(&encrypted_dek.encrypted_key)
        .map_err(|e| EnvelopeError::DekDecryptionFailed(e.to_string()))?;

    let dek = DataEncryptionKey::from_bytes(dek_bytes, envelope.algorithm);

    // Decrypt data with DEK
    decrypt_data_with_dek(envelope, &dek)
}

/// Decrypt an envelope using ECIES private key
///
/// # Arguments
///
/// * `envelope` - The envelope to decrypt
/// * `recipient_id` - The recipient ID to decrypt for
/// * `ecies_keypair` - The ECIES private key
pub fn decrypt_with_ecies(
    envelope: &Envelope,
    recipient_id: &str,
    ecies_keypair: &EciesKeyPair,
) -> EnvelopeResult<Vec<u8>> {
    // Find the encrypted DEK for this recipient
    let encrypted_dek = envelope
        .encrypted_deks
        .iter()
        .find(|dek| dek.recipient_id == recipient_id && matches!(dek.method, KeyEncryptionMethod::Ecies))
        .ok_or_else(|| EnvelopeError::RecipientNotFound(recipient_id.to_string()))?;

    // Decrypt DEK
    let ecies_encrypted = super::asymmetric::EciesEncrypted::from_bytes(&encrypted_dek.encrypted_key)?;
    let dek_bytes = ecies_keypair
        .decrypt(&ecies_encrypted)
        .map_err(|e| EnvelopeError::DekDecryptionFailed(e.to_string()))?;

    let dek = DataEncryptionKey::from_bytes(dek_bytes, envelope.algorithm);

    // Decrypt data with DEK
    decrypt_data_with_dek(envelope, &dek)
}

/// Internal helper to decrypt data with a DEK
fn decrypt_data_with_dek(envelope: &Envelope, dek: &DataEncryptionKey) -> EnvelopeResult<Vec<u8>> {
    let encrypted_data = super::symmetric::EncryptedData::new(
        envelope.ciphertext.clone(),
        envelope.nonce.clone(),
        envelope.associated_data.clone(),
    );

    match envelope.algorithm {
        EncryptionAlgorithm::Aes256Gcm => {
            let cipher = Aes256GcmCipher::new(dek.as_bytes())?;
            Ok(cipher.decrypt(&encrypted_data)?)
        }
        EncryptionAlgorithm::ChaCha20Poly1305 => {
            let cipher = ChaCha20Poly1305Cipher::new(dek.as_bytes())?;
            Ok(cipher.decrypt(&encrypted_data)?)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enterprise::crypto::asymmetric::{RsaKeySize};

    #[test]
    fn test_envelope_single_rsa_recipient() {
        let keypair = RsaKeyPair::generate(RsaKeySize::Bits2048).unwrap();
        let plaintext = b"Secret message for envelope encryption";

        let envelope = EnvelopeBuilder::new(EncryptionAlgorithm::Aes256Gcm)
            .add_rsa_recipient("user1".to_string(), keypair.public_key().clone())
            .encrypt(plaintext, None)
            .unwrap();

        assert_eq!(envelope.encrypted_deks.len(), 1);
        assert_eq!(envelope.encrypted_deks[0].recipient_id, "user1");

        let decrypted = decrypt_with_rsa(&envelope, "user1", &keypair).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_envelope_single_ecies_recipient() {
        let keypair = EciesKeyPair::generate();
        let plaintext = b"Secret message for ECIES envelope";

        let envelope = EnvelopeBuilder::new(EncryptionAlgorithm::ChaCha20Poly1305)
            .add_ecies_recipient("user1".to_string(), *keypair.public_key())
            .encrypt(plaintext, None)
            .unwrap();

        assert_eq!(envelope.encrypted_deks.len(), 1);

        let decrypted = decrypt_with_ecies(&envelope, "user1", &keypair).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_envelope_multiple_recipients() {
        let rsa_keypair1 = RsaKeyPair::generate(RsaKeySize::Bits2048).unwrap();
        let rsa_keypair2 = RsaKeyPair::generate(RsaKeySize::Bits2048).unwrap();
        let ecies_keypair = EciesKeyPair::generate();

        let plaintext = b"Multi-recipient secret message";

        let envelope = EnvelopeBuilder::new(EncryptionAlgorithm::Aes256Gcm)
            .add_rsa_recipient("alice".to_string(), rsa_keypair1.public_key().clone())
            .add_rsa_recipient("bob".to_string(), rsa_keypair2.public_key().clone())
            .add_ecies_recipient("charlie".to_string(), *ecies_keypair.public_key())
            .encrypt(plaintext, None)
            .unwrap();

        assert_eq!(envelope.encrypted_deks.len(), 3);

        // All recipients can decrypt
        let decrypted1 = decrypt_with_rsa(&envelope, "alice", &rsa_keypair1).unwrap();
        let decrypted2 = decrypt_with_rsa(&envelope, "bob", &rsa_keypair2).unwrap();
        let decrypted3 = decrypt_with_ecies(&envelope, "charlie", &ecies_keypair).unwrap();

        assert_eq!(decrypted1, plaintext);
        assert_eq!(decrypted2, plaintext);
        assert_eq!(decrypted3, plaintext);
    }

    #[test]
    fn test_envelope_with_associated_data() {
        let keypair = RsaKeyPair::generate(RsaKeySize::Bits2048).unwrap();
        let plaintext = b"Secret message";
        let aad = b"Public metadata";

        let envelope = EnvelopeBuilder::new(EncryptionAlgorithm::Aes256Gcm)
            .add_rsa_recipient("user1".to_string(), keypair.public_key().clone())
            .encrypt(plaintext, Some(aad))
            .unwrap();

        assert_eq!(envelope.associated_data, aad);

        let decrypted = decrypt_with_rsa(&envelope, "user1", &keypair).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_envelope_with_metadata() {
        let keypair = RsaKeyPair::generate(RsaKeySize::Bits2048).unwrap();
        let plaintext = b"Test";

        let envelope = EnvelopeBuilder::new(EncryptionAlgorithm::Aes256Gcm)
            .add_rsa_recipient("user1".to_string(), keypair.public_key().clone())
            .add_metadata("created_by".to_string(), "system".to_string())
            .add_metadata("version".to_string(), "1.0".to_string())
            .encrypt(plaintext, None)
            .unwrap();

        assert_eq!(envelope.metadata.get("created_by").unwrap(), "system");
        assert_eq!(envelope.metadata.get("version").unwrap(), "1.0");
    }

    #[test]
    fn test_envelope_serialization() {
        let keypair = RsaKeyPair::generate(RsaKeySize::Bits2048).unwrap();
        let plaintext = b"Serialization test";

        let envelope = EnvelopeBuilder::new(EncryptionAlgorithm::ChaCha20Poly1305)
            .add_rsa_recipient("user1".to_string(), keypair.public_key().clone())
            .encrypt(plaintext, None)
            .unwrap();

        let bytes = envelope.to_bytes().unwrap();
        let recovered = Envelope::from_bytes(&bytes).unwrap();

        let decrypted = decrypt_with_rsa(&recovered, "user1", &keypair).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_envelope_wrong_recipient() {
        let keypair1 = RsaKeyPair::generate(RsaKeySize::Bits2048).unwrap();
        let keypair2 = RsaKeyPair::generate(RsaKeySize::Bits2048).unwrap();

        let plaintext = b"Secret for user1 only";

        let envelope = EnvelopeBuilder::new(EncryptionAlgorithm::Aes256Gcm)
            .add_rsa_recipient("user1".to_string(), keypair1.public_key().clone())
            .encrypt(plaintext, None)
            .unwrap();

        // user2 should not be able to decrypt
        let result = decrypt_with_rsa(&envelope, "user2", &keypair2);
        assert!(result.is_err());
    }

    #[test]
    fn test_envelope_no_recipients_error() {
        let plaintext = b"Test";
        let result = EnvelopeBuilder::new(EncryptionAlgorithm::Aes256Gcm)
            .encrypt(plaintext, None);

        assert!(result.is_err());
    }

    #[test]
    fn test_recipient_ids() {
        let keypair1 = RsaKeyPair::generate(RsaKeySize::Bits2048).unwrap();
        let keypair2 = RsaKeyPair::generate(RsaKeySize::Bits2048).unwrap();

        let envelope = EnvelopeBuilder::new(EncryptionAlgorithm::Aes256Gcm)
            .add_rsa_recipient("alice".to_string(), keypair1.public_key().clone())
            .add_rsa_recipient("bob".to_string(), keypair2.public_key().clone())
            .encrypt(b"Test", None)
            .unwrap();

        let ids = envelope.recipient_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"alice".to_string()));
        assert!(ids.contains(&"bob".to_string()));
    }
}
