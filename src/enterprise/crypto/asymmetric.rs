//! # Asymmetric Encryption
//!
//! This module provides public-key encryption using industry-standard algorithms.
//!
//! ## Supported Algorithms
//!
//! - **RSA-OAEP**: RSA with Optimal Asymmetric Encryption Padding
//! - **ECIES**: Elliptic Curve Integrated Encryption Scheme (Curve25519)
//!
//! ## Key Management
//!
//! - Key pairs are generated securely using OS random number generator
//! - Private keys are zeroized on drop
//! - Public keys can be safely serialized and shared
//!
//! ## Security Considerations
//!
//! - RSA: Minimum 2048-bit keys (4096-bit recommended for long-term security)
//! - ECIES: Uses Curve25519 for key agreement
//! - Never reuse ephemeral keys
//! - Protect private keys with appropriate access controls

use rsa::{
    Oaep, RsaPrivateKey, RsaPublicKey,
    pkcs8::{EncodePublicKey, EncodePrivateKey, DecodePublicKey, DecodePrivateKey},
};
use x25519_dalek::{StaticSecret, PublicKey as X25519PublicKey};
use sha2::Sha256;
use rand::rngs::OsRng;
use zeroize::{Zeroize, ZeroizeOnDrop};
use thiserror::Error;

use super::symmetric::{ChaCha20Poly1305Cipher, SymmetricError};
use super::kdf::KdfProvider;

/// Asymmetric encryption errors
#[derive(Error, Debug)]
pub enum AsymmetricError {
    /// Key generation failed
    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),

    /// Encryption failed
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    /// Decryption failed
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    /// Invalid key format or size
    #[error("Invalid key: {0}")]
    InvalidKey(String),

    /// Key serialization failed
    #[error("Key serialization failed: {0}")]
    SerializationFailed(String),

    /// Key deserialization failed
    #[error("Key deserialization failed: {0}")]
    DeserializationFailed(String),

    /// Symmetric encryption error
    #[error("Symmetric encryption error: {0}")]
    SymmetricError(#[from] SymmetricError),
}

pub type AsymmetricResult<T> = Result<T, AsymmetricError>;

/// RSA key size in bits
#[derive(Debug, Clone, Copy)]
pub enum RsaKeySize {
    /// 2048-bit key (minimum recommended)
    Bits2048,
    /// 3072-bit key (good security)
    Bits3072,
    /// 4096-bit key (high security, slower)
    Bits4096,
}

impl RsaKeySize {
    fn bits(&self) -> usize {
        match self {
            RsaKeySize::Bits2048 => 2048,
            RsaKeySize::Bits3072 => 3072,
            RsaKeySize::Bits4096 => 4096,
        }
    }
}

/// RSA key pair
#[derive(ZeroizeOnDrop)]
pub struct RsaKeyPair {
    private_key: RsaPrivateKey,
    #[zeroize(skip)]
    public_key: RsaPublicKey,
}

impl RsaKeyPair {
    /// Generate a new RSA key pair
    ///
    /// # Arguments
    ///
    /// * `key_size` - The size of the key to generate
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use caddy::enterprise::crypto::asymmetric::{RsaKeyPair, RsaKeySize};
    ///
    /// let keypair = RsaKeyPair::generate(RsaKeySize::Bits2048)?;
    /// ```
    pub fn generate(key_size: RsaKeySize) -> AsymmetricResult<Self> {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, key_size.bits())
            .map_err(|e| AsymmetricError::KeyGenerationFailed(e.to_string()))?;
        let public_key = private_key.to_public_key();

        Ok(Self {
            private_key,
            public_key,
        })
    }

    /// Get the public key
    pub fn public_key(&self) -> &RsaPublicKey {
        &self.public_key
    }

    /// Serialize public key to PEM format
    pub fn public_key_to_pem(&self) -> AsymmetricResult<String> {
        self.public_key
            .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
            .map_err(|e| AsymmetricError::SerializationFailed(e.to_string()))
    }

    /// Serialize private key to PEM format (PKCS#8)
    pub fn private_key_to_pem(&self) -> AsymmetricResult<String> {
        self.private_key
            .to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)
            .map_err(|e| AsymmetricError::SerializationFailed(e.to_string()))
            .map(|pem| pem.to_string())
    }

    /// Load public key from PEM
    pub fn public_key_from_pem(pem: &str) -> AsymmetricResult<RsaPublicKey> {
        RsaPublicKey::from_public_key_pem(pem)
            .map_err(|e| AsymmetricError::DeserializationFailed(e.to_string()))
    }

    /// Load private key from PEM
    pub fn private_key_from_pem(pem: &str) -> AsymmetricResult<Self> {
        let private_key = RsaPrivateKey::from_pkcs8_pem(pem)
            .map_err(|e| AsymmetricError::DeserializationFailed(e.to_string()))?;
        let public_key = private_key.to_public_key();

        Ok(Self {
            private_key,
            public_key,
        })
    }

    /// Encrypt data using RSA-OAEP
    ///
    /// # Arguments
    ///
    /// * `public_key` - The recipient's public key
    /// * `plaintext` - Data to encrypt (must be smaller than key size - padding)
    pub fn encrypt(public_key: &RsaPublicKey, plaintext: &[u8]) -> AsymmetricResult<Vec<u8>> {
        let mut rng = OsRng;
        let padding = Oaep::new::<Sha256>();

        public_key
            .encrypt(&mut rng, padding, plaintext)
            .map_err(|e| AsymmetricError::EncryptionFailed(e.to_string()))
    }

    /// Decrypt data using RSA-OAEP
    ///
    /// # Arguments
    ///
    /// * `ciphertext` - The encrypted data
    pub fn decrypt(&self, ciphertext: &[u8]) -> AsymmetricResult<Vec<u8>> {
        let padding = Oaep::new::<Sha256>();

        self.private_key
            .decrypt(padding, ciphertext)
            .map_err(|e| AsymmetricError::DecryptionFailed(e.to_string()))
    }
}

impl std::fmt::Debug for RsaKeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RsaKeyPair")
            .field("private_key", &"[REDACTED]")
            .finish()
    }
}

/// X25519 key pair for ECIES
#[derive(ZeroizeOnDrop)]
pub struct EciesKeyPair {
    private_key: StaticSecret,
    #[zeroize(skip)]
    public_key: X25519PublicKey,
}

impl EciesKeyPair {
    /// Generate a new X25519 key pair
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use caddy::enterprise::crypto::asymmetric::EciesKeyPair;
    ///
    /// let keypair = EciesKeyPair::generate();
    /// ```
    pub fn generate() -> Self {
        let mut rng = OsRng;
        let private_key = StaticSecret::random_from_rng(&mut rng);
        let public_key = X25519PublicKey::from(&private_key);

        Self {
            private_key,
            public_key,
        }
    }

    /// Get the public key
    pub fn public_key(&self) -> &X25519PublicKey {
        &self.public_key
    }

    /// Get public key as bytes
    pub fn public_key_bytes(&self) -> [u8; 32] {
        *self.public_key.as_bytes()
    }

    /// Create key pair from private key bytes
    pub fn from_private_bytes(bytes: &[u8; 32]) -> Self {
        let private_key = StaticSecret::from(*bytes);
        let public_key = X25519PublicKey::from(&private_key);

        Self {
            private_key,
            public_key,
        }
    }

    /// Create public key from bytes
    pub fn public_key_from_bytes(bytes: &[u8; 32]) -> X25519PublicKey {
        X25519PublicKey::from(*bytes)
    }

    /// Encrypt data using ECIES
    ///
    /// This uses X25519 key agreement + HKDF + ChaCha20-Poly1305
    ///
    /// # Arguments
    ///
    /// * `recipient_public_key` - The recipient's public key
    /// * `plaintext` - Data to encrypt
    pub fn encrypt(recipient_public_key: &X25519PublicKey, plaintext: &[u8]) -> AsymmetricResult<EciesEncrypted> {
        // Generate ephemeral key pair
        let ephemeral_secret = StaticSecret::random_from_rng(&mut OsRng);
        let ephemeral_public = X25519PublicKey::from(&ephemeral_secret);

        // Perform key agreement
        let shared_secret = ephemeral_secret.diffie_hellman(recipient_public_key);

        // Derive encryption key using HKDF
        let derived_key = KdfProvider::expand_hkdf_sha256(
            shared_secret.as_bytes(),
            Some(b"ECIES-v1"),
            Some(b"encryption"),
            32,
        ).map_err(|e| AsymmetricError::EncryptionFailed(e.to_string()))?;

        // Encrypt using ChaCha20-Poly1305
        let cipher = ChaCha20Poly1305Cipher::new(derived_key.as_bytes())?;
        let encrypted = cipher.encrypt(plaintext, None)?;

        Ok(EciesEncrypted {
            ephemeral_public_key: ephemeral_public.as_bytes().to_owned(),
            ciphertext: encrypted.ciphertext,
            nonce: encrypted.nonce,
        })
    }

    /// Decrypt data using ECIES
    ///
    /// # Arguments
    ///
    /// * `encrypted` - The encrypted data with ephemeral public key
    pub fn decrypt(&self, encrypted: &EciesEncrypted) -> AsymmetricResult<Vec<u8>> {
        // Reconstruct ephemeral public key
        let ephemeral_public = X25519PublicKey::from(encrypted.ephemeral_public_key);

        // Perform key agreement
        let shared_secret = self.private_key.diffie_hellman(&ephemeral_public);

        // Derive decryption key using HKDF
        let derived_key = KdfProvider::expand_hkdf_sha256(
            shared_secret.as_bytes(),
            Some(b"ECIES-v1"),
            Some(b"encryption"),
            32,
        ).map_err(|e| AsymmetricError::DecryptionFailed(e.to_string()))?;

        // Decrypt using ChaCha20-Poly1305
        let cipher = ChaCha20Poly1305Cipher::new(derived_key.as_bytes())?;
        let encrypted_data = super::symmetric::EncryptedData::new(
            encrypted.ciphertext.clone(),
            encrypted.nonce.clone(),
            Vec::new(),
        );

        cipher.decrypt(&encrypted_data)
            .map_err(|e| AsymmetricError::DecryptionFailed(e.to_string()))
    }
}

impl std::fmt::Debug for EciesKeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EciesKeyPair")
            .field("public_key", &hex::encode(self.public_key.as_bytes()))
            .field("private_key", &"[REDACTED]")
            .finish()
    }
}

/// ECIES encrypted data
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct EciesEncrypted {
    /// Ephemeral public key (32 bytes)
    pub ephemeral_public_key: [u8; 32],
    /// Encrypted data
    pub ciphertext: Vec<u8>,
    /// Nonce for ChaCha20-Poly1305
    pub nonce: Vec<u8>,
}

impl EciesEncrypted {
    /// Serialize to bytes (format: ephemeral_pk || nonce || ciphertext)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(32 + self.nonce.len() + self.ciphertext.len());
        result.extend_from_slice(&self.ephemeral_public_key);
        result.extend_from_slice(&self.nonce);
        result.extend_from_slice(&self.ciphertext);
        result
    }

    /// Parse from bytes
    pub fn from_bytes(data: &[u8]) -> AsymmetricResult<Self> {
        if data.len() < 32 + 12 {
            return Err(AsymmetricError::InvalidKey(
                "Data too short for ECIES encrypted format".to_string()
            ));
        }

        let mut ephemeral_public_key = [0u8; 32];
        ephemeral_public_key.copy_from_slice(&data[0..32]);

        let nonce = data[32..44].to_vec();
        let ciphertext = data[44..].to_vec();

        Ok(Self {
            ephemeral_public_key,
            ciphertext,
            nonce,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsa_key_generation() {
        let keypair = RsaKeyPair::generate(RsaKeySize::Bits2048).unwrap();
        assert_eq!(keypair.private_key.size(), 256); // 2048 bits = 256 bytes
    }

    #[test]
    fn test_rsa_encryption_decryption() {
        let keypair = RsaKeyPair::generate(RsaKeySize::Bits2048).unwrap();
        let plaintext = b"Hello, RSA!";

        let ciphertext = RsaKeyPair::encrypt(keypair.public_key(), plaintext).unwrap();
        let decrypted = keypair.decrypt(&ciphertext).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_rsa_public_key_serialization() {
        let keypair = RsaKeyPair::generate(RsaKeySize::Bits2048).unwrap();
        let pem = keypair.public_key_to_pem().unwrap();

        assert!(pem.contains("BEGIN PUBLIC KEY"));
        assert!(pem.contains("END PUBLIC KEY"));

        // Deserialize and verify it works
        let public_key = RsaKeyPair::public_key_from_pem(&pem).unwrap();
        let plaintext = b"Test";
        let ciphertext = RsaKeyPair::encrypt(&public_key, plaintext).unwrap();
        let decrypted = keypair.decrypt(&ciphertext).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_rsa_private_key_serialization() {
        let keypair = RsaKeyPair::generate(RsaKeySize::Bits2048).unwrap();
        let pem = keypair.private_key_to_pem().unwrap();

        assert!(pem.contains("BEGIN PRIVATE KEY"));
        assert!(pem.contains("END PRIVATE KEY"));

        // Deserialize and verify it works
        let restored_keypair = RsaKeyPair::private_key_from_pem(&pem).unwrap();
        let plaintext = b"Test";
        let ciphertext = RsaKeyPair::encrypt(restored_keypair.public_key(), plaintext).unwrap();
        let decrypted = restored_keypair.decrypt(&ciphertext).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_ecies_key_generation() {
        let keypair = EciesKeyPair::generate();
        assert_eq!(keypair.public_key_bytes().len(), 32);
    }

    #[test]
    fn test_ecies_encryption_decryption() {
        let keypair = EciesKeyPair::generate();
        let plaintext = b"Hello, ECIES!";

        let encrypted = EciesKeyPair::encrypt(keypair.public_key(), plaintext).unwrap();
        let decrypted = keypair.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_ecies_different_recipients() {
        let alice = EciesKeyPair::generate();
        let bob = EciesKeyPair::generate();

        let plaintext = b"Secret message for Bob";

        // Encrypt for Bob
        let encrypted = EciesKeyPair::encrypt(bob.public_key(), plaintext).unwrap();

        // Bob can decrypt
        let decrypted = bob.decrypt(&encrypted).unwrap();
        assert_eq!(plaintext, decrypted.as_slice());

        // Alice cannot decrypt (will fail authentication)
        let result = alice.decrypt(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_ecies_serialization() {
        let keypair = EciesKeyPair::generate();
        let plaintext = b"Test message";

        let encrypted = EciesKeyPair::encrypt(keypair.public_key(), plaintext).unwrap();
        let bytes = encrypted.to_bytes();
        let recovered = EciesEncrypted::from_bytes(&bytes).unwrap();

        let decrypted = keypair.decrypt(&recovered).unwrap();
        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_ecies_key_from_bytes() {
        let keypair = EciesKeyPair::generate();
        let public_bytes = keypair.public_key_bytes();

        let restored_public = EciesKeyPair::public_key_from_bytes(&public_bytes);
        assert_eq!(restored_public.as_bytes(), &public_bytes);
    }

    #[test]
    fn test_rsa_wrong_key_decryption() {
        let keypair1 = RsaKeyPair::generate(RsaKeySize::Bits2048).unwrap();
        let keypair2 = RsaKeyPair::generate(RsaKeySize::Bits2048).unwrap();

        let plaintext = b"Secret";
        let ciphertext = RsaKeyPair::encrypt(keypair1.public_key(), plaintext).unwrap();

        // Different key should fail to decrypt
        let result = keypair2.decrypt(&ciphertext);
        assert!(result.is_err());
    }
}
