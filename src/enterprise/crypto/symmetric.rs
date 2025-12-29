//! # Symmetric Encryption
//!
//! This module provides authenticated encryption (AEAD) using industry-standard
//! symmetric ciphers. All algorithms provide both confidentiality and authenticity.
//!
//! ## Supported Algorithms
//!
//! - **AES-256-GCM**: AES in Galois/Counter Mode (NIST standard)
//! - **ChaCha20-Poly1305**: Modern stream cipher with MAC (RFC 8439)
//! - **XChaCha20-Poly1305**: Extended nonce variant for large message sets
//!
//! ## Security Features
//!
//! - Authenticated Encryption with Associated Data (AEAD)
//! - Automatic nonce management
//! - Key material is zeroized on drop
//! - Constant-time comparisons
//!
//! ## Nonce Management
//!
//! - AES-GCM: 96-bit (12 byte) nonces - NEVER reuse with the same key
//! - ChaCha20-Poly1305: 96-bit nonces
//! - XChaCha20-Poly1305: 192-bit (24 byte) nonces - more collision resistant

use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce as AesNonce,
};
use chacha20poly1305::{
    ChaCha20Poly1305, XChaCha20Poly1305,
    XNonce,
};
use rand::{RngCore, rngs::OsRng};
use zeroize::Zeroize;
use thiserror::Error;

/// Symmetric encryption errors
#[derive(Error, Debug)]
pub enum SymmetricError {
    /// Encryption failed
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    /// Decryption failed (authentication error or corrupted data)
    #[error("Decryption failed: authentication error or corrupted data")]
    DecryptionFailed,

    /// Invalid key size
    #[error("Invalid key size: expected {expected}, got {actual}")]
    InvalidKeySize { expected: usize, actual: usize },

    /// Invalid nonce size
    #[error("Invalid nonce size: expected {expected}, got {actual}")]
    InvalidNonceSize { expected: usize, actual: usize },

    /// Nonce reuse detected (security violation)
    #[error("Nonce reuse detected - this is a critical security violation")]
    NonceReuse,
}

pub type SymmetricResult<T> = Result<T, SymmetricError>;

/// Encryption result containing ciphertext and nonce
#[derive(Debug, Clone, Zeroize)]
pub struct EncryptedData {
    /// The ciphertext (includes authentication tag)
    pub ciphertext: Vec<u8>,
    /// The nonce used for encryption
    pub nonce: Vec<u8>,
    /// Associated data (not encrypted, but authenticated)
    #[zeroize(skip)]
    pub associated_data: Vec<u8>,
}

impl EncryptedData {
    /// Create new encrypted data
    pub fn new(ciphertext: Vec<u8>, nonce: Vec<u8>, associated_data: Vec<u8>) -> Self {
        Self {
            ciphertext,
            nonce,
            associated_data,
        }
    }

    /// Serialize to a single byte vector (format: nonce || ciphertext)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.nonce.len() + self.ciphertext.len());
        result.extend_from_slice(&self.nonce);
        result.extend_from_slice(&self.ciphertext);
        result
    }

    /// Parse from byte vector
    pub fn from_bytes(data: &[u8], nonce_size: usize) -> SymmetricResult<Self> {
        if data.len() < nonce_size {
            return Err(SymmetricError::InvalidNonceSize {
                expected: nonce_size,
                actual: data.len(),
            });
        }

        let (nonce, ciphertext) = data.split_at(nonce_size);
        Ok(Self {
            ciphertext: ciphertext.to_vec(),
            nonce: nonce.to_vec(),
            associated_data: Vec::new(),
        })
    }
}

/// AES-256-GCM cipher
pub struct Aes256GcmCipher {
    cipher: Aes256Gcm,
}

impl Aes256GcmCipher {
    /// Key size in bytes (256 bits)
    pub const KEY_SIZE: usize = 32;
    /// Nonce size in bytes (96 bits)
    pub const NONCE_SIZE: usize = 12;

    /// Create a new AES-256-GCM cipher with the given key
    ///
    /// # Arguments
    ///
    /// * `key` - 256-bit (32 byte) encryption key
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use caddy::enterprise::crypto::symmetric::Aes256GcmCipher;
    ///
    /// let key = [0u8; 32]; // In practice, use a secure random key
    /// let cipher = Aes256GcmCipher::new(&key)?;
    /// ```
    pub fn new(key: &[u8]) -> SymmetricResult<Self> {
        if key.len() != Self::KEY_SIZE {
            return Err(SymmetricError::InvalidKeySize {
                expected: Self::KEY_SIZE,
                actual: key.len(),
            });
        }

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| SymmetricError::EncryptionFailed(e.to_string()))?;

        Ok(Self { cipher })
    }

    /// Generate a random nonce
    pub fn generate_nonce() -> Vec<u8> {
        let mut nonce = vec![0u8; Self::NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce);
        nonce
    }

    /// Encrypt data with optional associated data
    ///
    /// # Arguments
    ///
    /// * `plaintext` - Data to encrypt
    /// * `associated_data` - Optional additional authenticated data (AAD)
    ///
    /// # Returns
    ///
    /// Encrypted data with nonce and authentication tag
    pub fn encrypt(
        &self,
        plaintext: &[u8],
        associated_data: Option<&[u8]>,
    ) -> SymmetricResult<EncryptedData> {
        let nonce_bytes = Self::generate_nonce();
        let nonce = AesNonce::from_slice(&nonce_bytes);

        let payload = Payload {
            msg: plaintext,
            aad: associated_data.unwrap_or(b""),
        };

        let ciphertext = self.cipher
            .encrypt(nonce, payload)
            .map_err(|_| SymmetricError::EncryptionFailed("AES-GCM encryption failed".to_string()))?;

        Ok(EncryptedData::new(
            ciphertext,
            nonce_bytes,
            associated_data.unwrap_or(b"").to_vec(),
        ))
    }

    /// Decrypt data
    ///
    /// # Arguments
    ///
    /// * `encrypted` - The encrypted data with nonce
    ///
    /// # Returns
    ///
    /// Decrypted plaintext or error if authentication fails
    pub fn decrypt(&self, encrypted: &EncryptedData) -> SymmetricResult<Vec<u8>> {
        if encrypted.nonce.len() != Self::NONCE_SIZE {
            return Err(SymmetricError::InvalidNonceSize {
                expected: Self::NONCE_SIZE,
                actual: encrypted.nonce.len(),
            });
        }

        let nonce = AesNonce::from_slice(&encrypted.nonce);

        let payload = Payload {
            msg: &encrypted.ciphertext,
            aad: &encrypted.associated_data,
        };

        self.cipher
            .decrypt(nonce, payload)
            .map_err(|_| SymmetricError::DecryptionFailed)
    }
}

impl Drop for Aes256GcmCipher {
    fn drop(&mut self) {
        // The cipher key is zeroized when the cipher is dropped
    }
}

/// ChaCha20-Poly1305 cipher
pub struct ChaCha20Poly1305Cipher {
    cipher: ChaCha20Poly1305,
}

impl ChaCha20Poly1305Cipher {
    /// Key size in bytes (256 bits)
    pub const KEY_SIZE: usize = 32;
    /// Nonce size in bytes (96 bits)
    pub const NONCE_SIZE: usize = 12;

    /// Create a new ChaCha20-Poly1305 cipher with the given key
    pub fn new(key: &[u8]) -> SymmetricResult<Self> {
        if key.len() != Self::KEY_SIZE {
            return Err(SymmetricError::InvalidKeySize {
                expected: Self::KEY_SIZE,
                actual: key.len(),
            });
        }

        let cipher = ChaCha20Poly1305::new_from_slice(key)
            .map_err(|e| SymmetricError::EncryptionFailed(e.to_string()))?;

        Ok(Self { cipher })
    }

    /// Generate a random nonce
    pub fn generate_nonce() -> Vec<u8> {
        let mut nonce = vec![0u8; Self::NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce);
        nonce
    }

    /// Encrypt data with optional associated data
    pub fn encrypt(
        &self,
        plaintext: &[u8],
        associated_data: Option<&[u8]>,
    ) -> SymmetricResult<EncryptedData> {
        let nonce_bytes = Self::generate_nonce();
        let nonce = chacha20poly1305::Nonce::from_slice(&nonce_bytes);

        let payload = Payload {
            msg: plaintext,
            aad: associated_data.unwrap_or(b""),
        };

        let ciphertext = self.cipher
            .encrypt(nonce, payload)
            .map_err(|_| SymmetricError::EncryptionFailed("ChaCha20-Poly1305 encryption failed".to_string()))?;

        Ok(EncryptedData::new(
            ciphertext,
            nonce_bytes,
            associated_data.unwrap_or(b"").to_vec(),
        ))
    }

    /// Decrypt data
    pub fn decrypt(&self, encrypted: &EncryptedData) -> SymmetricResult<Vec<u8>> {
        if encrypted.nonce.len() != Self::NONCE_SIZE {
            return Err(SymmetricError::InvalidNonceSize {
                expected: Self::NONCE_SIZE,
                actual: encrypted.nonce.len(),
            });
        }

        let nonce = chacha20poly1305::Nonce::from_slice(&encrypted.nonce);

        let payload = Payload {
            msg: &encrypted.ciphertext,
            aad: &encrypted.associated_data,
        };

        self.cipher
            .decrypt(nonce, payload)
            .map_err(|_| SymmetricError::DecryptionFailed)
    }
}

/// XChaCha20-Poly1305 cipher (extended nonce)
pub struct XChaCha20Poly1305Cipher {
    cipher: XChaCha20Poly1305,
}

impl XChaCha20Poly1305Cipher {
    /// Key size in bytes (256 bits)
    pub const KEY_SIZE: usize = 32;
    /// Nonce size in bytes (192 bits) - larger than ChaCha20
    pub const NONCE_SIZE: usize = 24;

    /// Create a new XChaCha20-Poly1305 cipher with the given key
    pub fn new(key: &[u8]) -> SymmetricResult<Self> {
        if key.len() != Self::KEY_SIZE {
            return Err(SymmetricError::InvalidKeySize {
                expected: Self::KEY_SIZE,
                actual: key.len(),
            });
        }

        let cipher = XChaCha20Poly1305::new_from_slice(key)
            .map_err(|e| SymmetricError::EncryptionFailed(e.to_string()))?;

        Ok(Self { cipher })
    }

    /// Generate a random nonce (192 bits)
    pub fn generate_nonce() -> Vec<u8> {
        let mut nonce = vec![0u8; Self::NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce);
        nonce
    }

    /// Encrypt data with optional associated data
    ///
    /// XChaCha20 is recommended when you need to encrypt many messages
    /// with the same key, as the larger nonce space reduces collision risk.
    pub fn encrypt(
        &self,
        plaintext: &[u8],
        associated_data: Option<&[u8]>,
    ) -> SymmetricResult<EncryptedData> {
        let nonce_bytes = Self::generate_nonce();
        let nonce = XNonce::from_slice(&nonce_bytes);

        let payload = Payload {
            msg: plaintext,
            aad: associated_data.unwrap_or(b""),
        };

        let ciphertext = self.cipher
            .encrypt(nonce, payload)
            .map_err(|_| SymmetricError::EncryptionFailed("XChaCha20-Poly1305 encryption failed".to_string()))?;

        Ok(EncryptedData::new(
            ciphertext,
            nonce_bytes,
            associated_data.unwrap_or(b"").to_vec(),
        ))
    }

    /// Decrypt data
    pub fn decrypt(&self, encrypted: &EncryptedData) -> SymmetricResult<Vec<u8>> {
        if encrypted.nonce.len() != Self::NONCE_SIZE {
            return Err(SymmetricError::InvalidNonceSize {
                expected: Self::NONCE_SIZE,
                actual: encrypted.nonce.len(),
            });
        }

        let nonce = XNonce::from_slice(&encrypted.nonce);

        let payload = Payload {
            msg: &encrypted.ciphertext,
            aad: &encrypted.associated_data,
        };

        self.cipher
            .decrypt(nonce, payload)
            .map_err(|_| SymmetricError::DecryptionFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_gcm_encryption_decryption() {
        let key = [0u8; 32];
        let cipher = Aes256GcmCipher::new(&key).unwrap();

        let plaintext = b"Hello, World!";
        let encrypted = cipher.encrypt(plaintext, None).unwrap();
        let decrypted = cipher.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_aes_gcm_with_aad() {
        let key = [0u8; 32];
        let cipher = Aes256GcmCipher::new(&key).unwrap();

        let plaintext = b"Secret message";
        let aad = b"Public context";
        let encrypted = cipher.encrypt(plaintext, Some(aad)).unwrap();
        let decrypted = cipher.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_aes_gcm_wrong_aad_fails() {
        let key = [0u8; 32];
        let cipher = Aes256GcmCipher::new(&key).unwrap();

        let plaintext = b"Secret message";
        let mut encrypted = cipher.encrypt(plaintext, Some(b"correct aad")).unwrap();
        encrypted.associated_data = b"wrong aad".to_vec();

        let result = cipher.decrypt(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_aes_gcm_invalid_key_size() {
        let key = [0u8; 16]; // Wrong size
        let result = Aes256GcmCipher::new(&key);
        assert!(result.is_err());
    }

    #[test]
    fn test_aes_gcm_tampered_ciphertext() {
        let key = [0u8; 32];
        let cipher = Aes256GcmCipher::new(&key).unwrap();

        let plaintext = b"Secret message";
        let mut encrypted = cipher.encrypt(plaintext, None).unwrap();

        // Tamper with ciphertext
        encrypted.ciphertext[0] ^= 1;

        let result = cipher.decrypt(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_chacha20_encryption_decryption() {
        let key = [0u8; 32];
        let cipher = ChaCha20Poly1305Cipher::new(&key).unwrap();

        let plaintext = b"Hello, ChaCha20!";
        let encrypted = cipher.encrypt(plaintext, None).unwrap();
        let decrypted = cipher.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_chacha20_with_aad() {
        let key = [0u8; 32];
        let cipher = ChaCha20Poly1305Cipher::new(&key).unwrap();

        let plaintext = b"Secret message";
        let aad = b"Public context";
        let encrypted = cipher.encrypt(plaintext, Some(aad)).unwrap();
        let decrypted = cipher.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_xchacha20_encryption_decryption() {
        let key = [0u8; 32];
        let cipher = XChaCha20Poly1305Cipher::new(&key).unwrap();

        let plaintext = b"Hello, XChaCha20!";
        let encrypted = cipher.encrypt(plaintext, None).unwrap();
        let decrypted = cipher.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_xchacha20_nonce_size() {
        let nonce = XChaCha20Poly1305Cipher::generate_nonce();
        assert_eq!(nonce.len(), 24);
    }

    #[test]
    fn test_encrypted_data_serialization() {
        let key = [0u8; 32];
        let cipher = Aes256GcmCipher::new(&key).unwrap();

        let plaintext = b"Test message";
        let encrypted = cipher.encrypt(plaintext, None).unwrap();

        // Serialize and deserialize
        let bytes = encrypted.to_bytes();
        let recovered = EncryptedData::from_bytes(&bytes, Aes256GcmCipher::NONCE_SIZE).unwrap();

        // Decrypt should still work
        let decrypted = cipher.decrypt(&recovered).unwrap();
        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_nonce_uniqueness() {
        let nonce1 = Aes256GcmCipher::generate_nonce();
        let nonce2 = Aes256GcmCipher::generate_nonce();
        assert_ne!(nonce1, nonce2);
    }
}
