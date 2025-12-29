//! # Digital Signatures
//!
//! This module provides digital signature algorithms for authentication and integrity.
//!
//! ## Supported Algorithms
//!
//! - **Ed25519**: EdDSA using Curve25519 (fast, small signatures)
//! - **ECDSA P-256**: Elliptic Curve Digital Signature Algorithm
//! - **Multi-signatures**: Threshold and aggregate signatures
//!
//! ## Features
//!
//! - Key pair generation
//! - Message signing
//! - Signature verification
//! - Key serialization (PEM, DER, raw bytes)
//! - Multi-signature support
//!
//! ## Security Considerations
//!
//! - Private keys are zeroized on drop
//! - Use deterministic signatures (RFC 6979) for ECDSA
//! - Never sign untrusted data directly (hash first)
//! - Verify all signatures before accepting them

use ed25519_dalek::{
    Signature as Ed25519Signature,
    SigningKey as Ed25519SigningKey,
    VerifyingKey as Ed25519VerifyingKey,
    Signer,
    Verifier,
};
use p256::ecdsa::{
    SigningKey as P256SigningKey,
    VerifyingKey as P256VerifyingKey,
    Signature as P256Signature,
    signature::{Signer as EcdsaSigner, Verifier as EcdsaVerifier},
};
use rand::rngs::OsRng;
use zeroize::{Zeroize, ZeroizeOnDrop};
use thiserror::Error;
use serde::{Serialize, Deserialize};

/// Signature-specific errors
#[derive(Error, Debug)]
pub enum SignatureError {
    /// Key generation failed
    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),

    /// Signing failed
    #[error("Signing failed: {0}")]
    SigningFailed(String),

    /// Signature verification failed
    #[error("Signature verification failed")]
    VerificationFailed,

    /// Invalid key format
    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),

    /// Invalid signature format
    #[error("Invalid signature format: {0}")]
    InvalidSignatureFormat(String),

    /// Key serialization failed
    #[error("Key serialization failed: {0}")]
    SerializationFailed(String),

    /// Key deserialization failed
    #[error("Key deserialization failed: {0}")]
    DeserializationFailed(String),
}

pub type SignatureResult<T> = Result<T, SignatureError>;

/// Ed25519 key pair for signing
#[derive(ZeroizeOnDrop)]
pub struct Ed25519KeyPair {
    signing_key: Ed25519SigningKey,
    #[zeroize(skip)]
    verifying_key: Ed25519VerifyingKey,
}

impl Ed25519KeyPair {
    /// Generate a new Ed25519 key pair
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use caddy::enterprise::crypto::signature::Ed25519KeyPair;
    ///
    /// let keypair = Ed25519KeyPair::generate();
    /// ```
    pub fn generate() -> Self {
        let mut rng = OsRng;
        let signing_key = Ed25519SigningKey::generate(&mut rng);
        let verifying_key = signing_key.verifying_key();

        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Create key pair from private key bytes (32 bytes)
    pub fn from_bytes(bytes: &[u8]) -> SignatureResult<Self> {
        if bytes.len() != 32 {
            return Err(SignatureError::InvalidKeyFormat(
                format!("Expected 32 bytes, got {}", bytes.len())
            ));
        }

        let signing_key = Ed25519SigningKey::from_bytes(
            bytes.try_into().map_err(|_| SignatureError::InvalidKeyFormat("Invalid key bytes".to_string()))?
        );
        let verifying_key = signing_key.verifying_key();

        Ok(Self {
            signing_key,
            verifying_key,
        })
    }

    /// Get the public (verifying) key
    pub fn verifying_key(&self) -> &Ed25519VerifyingKey {
        &self.verifying_key
    }

    /// Get verifying key as bytes
    pub fn verifying_key_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }

    /// Get signing key as bytes (use with caution - exposes private key)
    pub fn signing_key_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }

    /// Sign a message
    ///
    /// # Arguments
    ///
    /// * `message` - The message to sign
    ///
    /// # Returns
    ///
    /// The signature (64 bytes)
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        let signature = self.signing_key.sign(message);
        signature.to_bytes().to_vec()
    }

    /// Verify a signature
    ///
    /// # Arguments
    ///
    /// * `message` - The original message
    /// * `signature` - The signature to verify
    pub fn verify(verifying_key: &Ed25519VerifyingKey, message: &[u8], signature: &[u8]) -> SignatureResult<()> {
        if signature.len() != 64 {
            return Err(SignatureError::InvalidSignatureFormat(
                format!("Expected 64 bytes, got {}", signature.len())
            ));
        }

        let sig = Ed25519Signature::from_bytes(
            signature.try_into().map_err(|_| SignatureError::InvalidSignatureFormat("Invalid signature".to_string()))?
        );

        verifying_key.verify(message, &sig)
            .map_err(|_| SignatureError::VerificationFailed)
    }

    /// Create verifying key from bytes
    pub fn verifying_key_from_bytes(bytes: &[u8]) -> SignatureResult<Ed25519VerifyingKey> {
        if bytes.len() != 32 {
            return Err(SignatureError::InvalidKeyFormat(
                format!("Expected 32 bytes, got {}", bytes.len())
            ));
        }

        Ed25519VerifyingKey::from_bytes(
            bytes.try_into().map_err(|_| SignatureError::InvalidKeyFormat("Invalid key bytes".to_string()))?
        ).map_err(|e| SignatureError::InvalidKeyFormat(e.to_string()))
    }
}

impl std::fmt::Debug for Ed25519KeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ed25519KeyPair")
            .field("verifying_key", &hex::encode(self.verifying_key.to_bytes()))
            .field("signing_key", &"[REDACTED]")
            .finish()
    }
}

/// ECDSA P-256 key pair for signing
#[derive(ZeroizeOnDrop)]
pub struct EcdsaP256KeyPair {
    signing_key: P256SigningKey,
    #[zeroize(skip)]
    verifying_key: P256VerifyingKey,
}

impl EcdsaP256KeyPair {
    /// Generate a new ECDSA P-256 key pair
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use caddy::enterprise::crypto::signature::EcdsaP256KeyPair;
    ///
    /// let keypair = EcdsaP256KeyPair::generate();
    /// ```
    pub fn generate() -> Self {
        let mut rng = OsRng;
        let signing_key = P256SigningKey::random(&mut rng);
        let verifying_key = signing_key.verifying_key().clone();

        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Create key pair from private key bytes (32 bytes)
    pub fn from_bytes(bytes: &[u8]) -> SignatureResult<Self> {
        let signing_key = P256SigningKey::from_bytes(bytes.into())
            .map_err(|e| SignatureError::InvalidKeyFormat(e.to_string()))?;
        let verifying_key = signing_key.verifying_key().clone();

        Ok(Self {
            signing_key,
            verifying_key,
        })
    }

    /// Get the public (verifying) key
    pub fn verifying_key(&self) -> &P256VerifyingKey {
        &self.verifying_key
    }

    /// Get verifying key as bytes (SEC1 encoded point)
    pub fn verifying_key_bytes(&self) -> Vec<u8> {
        use p256::elliptic_curve::sec1::ToEncodedPoint;
        self.verifying_key.to_encoded_point(false).as_bytes().to_vec()
    }

    /// Sign a message using ECDSA
    ///
    /// # Arguments
    ///
    /// * `message` - The message to sign (will be hashed with SHA-256)
    ///
    /// # Returns
    ///
    /// The signature (DER encoded)
    pub fn sign(&self, message: &[u8]) -> SignatureResult<Vec<u8>> {
        let signature: P256Signature = self.signing_key.sign(message);
        Ok(signature.to_der().as_bytes().to_vec())
    }

    /// Verify an ECDSA signature
    ///
    /// # Arguments
    ///
    /// * `verifying_key` - The public key
    /// * `message` - The original message
    /// * `signature` - The signature to verify (DER encoded)
    pub fn verify(
        verifying_key: &P256VerifyingKey,
        message: &[u8],
        signature: &[u8],
    ) -> SignatureResult<()> {
        let sig = P256Signature::from_der(signature)
            .map_err(|e| SignatureError::InvalidSignatureFormat(e.to_string()))?;

        verifying_key.verify(message, &sig)
            .map_err(|_| SignatureError::VerificationFailed)
    }

    /// Create verifying key from bytes (SEC1 encoded point)
    pub fn verifying_key_from_bytes(bytes: &[u8]) -> SignatureResult<P256VerifyingKey> {
        use p256::elliptic_curve::sec1::FromEncodedPoint;
        use p256::EncodedPoint;

        let point = EncodedPoint::from_bytes(bytes)
            .map_err(|e| SignatureError::InvalidKeyFormat(e.to_string()))?;

        P256VerifyingKey::from_encoded_point(&point)
            .map_err(|e| SignatureError::InvalidKeyFormat(e.to_string()))
    }
}

impl std::fmt::Debug for EcdsaP256KeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EcdsaP256KeyPair")
            .field("verifying_key", &hex::encode(self.verifying_key_bytes()))
            .field("signing_key", &"[REDACTED]")
            .finish()
    }
}

/// Multi-signature scheme (simplified)
///
/// This is a basic implementation for demonstration.
/// Production systems should use specialized libraries like BLS signatures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSignature {
    /// List of individual signatures
    pub signatures: Vec<SignatureEntry>,
    /// Threshold required for validity
    pub threshold: usize,
}

/// Individual signature entry in a multi-signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureEntry {
    /// Signer identifier
    pub signer_id: String,
    /// The signature
    pub signature: Vec<u8>,
    /// Public key (verifying key) bytes
    pub public_key: Vec<u8>,
}

impl MultiSignature {
    /// Create a new multi-signature
    pub fn new(threshold: usize) -> Self {
        Self {
            signatures: Vec::new(),
            threshold,
        }
    }

    /// Add a signature to the multi-signature
    pub fn add_signature(&mut self, entry: SignatureEntry) {
        self.signatures.push(entry);
    }

    /// Verify the multi-signature (Ed25519 version)
    ///
    /// # Arguments
    ///
    /// * `message` - The message that was signed
    pub fn verify_ed25519(&self, message: &[u8]) -> SignatureResult<bool> {
        if self.signatures.len() < self.threshold {
            return Ok(false);
        }

        let mut valid_count = 0;

        for entry in &self.signatures {
            let verifying_key = Ed25519KeyPair::verifying_key_from_bytes(&entry.public_key)?;

            if Ed25519KeyPair::verify(&verifying_key, message, &entry.signature).is_ok() {
                valid_count += 1;
            }
        }

        Ok(valid_count >= self.threshold)
    }

    /// Verify the multi-signature (ECDSA P-256 version)
    pub fn verify_ecdsa_p256(&self, message: &[u8]) -> SignatureResult<bool> {
        if self.signatures.len() < self.threshold {
            return Ok(false);
        }

        let mut valid_count = 0;

        for entry in &self.signatures {
            let verifying_key = EcdsaP256KeyPair::verifying_key_from_bytes(&entry.public_key)?;

            if EcdsaP256KeyPair::verify(&verifying_key, message, &entry.signature).is_ok() {
                valid_count += 1;
            }
        }

        Ok(valid_count >= self.threshold)
    }

    /// Get the number of signatures
    pub fn signature_count(&self) -> usize {
        self.signatures.len()
    }

    /// Check if threshold is met
    pub fn is_threshold_met(&self) -> bool {
        self.signatures.len() >= self.threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ed25519_key_generation() {
        let keypair = Ed25519KeyPair::generate();
        assert_eq!(keypair.verifying_key_bytes().len(), 32);
    }

    #[test]
    fn test_ed25519_sign_verify() {
        let keypair = Ed25519KeyPair::generate();
        let message = b"Hello, Ed25519!";

        let signature = keypair.sign(message);
        assert_eq!(signature.len(), 64);

        let result = Ed25519KeyPair::verify(keypair.verifying_key(), message, &signature);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ed25519_verify_wrong_message() {
        let keypair = Ed25519KeyPair::generate();
        let message = b"Original message";
        let wrong_message = b"Wrong message";

        let signature = keypair.sign(message);

        let result = Ed25519KeyPair::verify(keypair.verifying_key(), wrong_message, &signature);
        assert!(result.is_err());
    }

    #[test]
    fn test_ed25519_verify_wrong_signature() {
        let keypair = Ed25519KeyPair::generate();
        let message = b"Test message";

        let wrong_signature = vec![0u8; 64];

        let result = Ed25519KeyPair::verify(keypair.verifying_key(), message, &wrong_signature);
        assert!(result.is_err());
    }

    #[test]
    fn test_ed25519_key_serialization() {
        let keypair = Ed25519KeyPair::generate();
        let signing_key_bytes = keypair.signing_key_bytes();
        let verifying_key_bytes = keypair.verifying_key_bytes();

        // Reconstruct from bytes
        let restored_keypair = Ed25519KeyPair::from_bytes(&signing_key_bytes).unwrap();
        assert_eq!(restored_keypair.verifying_key_bytes(), verifying_key_bytes);

        // Verify signing still works
        let message = b"Test";
        let signature = restored_keypair.sign(message);
        assert!(Ed25519KeyPair::verify(restored_keypair.verifying_key(), message, &signature).is_ok());
    }

    #[test]
    fn test_ecdsa_p256_key_generation() {
        let keypair = EcdsaP256KeyPair::generate();
        let pubkey_bytes = keypair.verifying_key_bytes();
        assert!(!pubkey_bytes.is_empty());
    }

    #[test]
    fn test_ecdsa_p256_sign_verify() {
        let keypair = EcdsaP256KeyPair::generate();
        let message = b"Hello, ECDSA P-256!";

        let signature = keypair.sign(message).unwrap();
        assert!(!signature.is_empty());

        let result = EcdsaP256KeyPair::verify(keypair.verifying_key(), message, &signature);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ecdsa_p256_verify_wrong_message() {
        let keypair = EcdsaP256KeyPair::generate();
        let message = b"Original message";
        let wrong_message = b"Wrong message";

        let signature = keypair.sign(message).unwrap();

        let result = EcdsaP256KeyPair::verify(keypair.verifying_key(), wrong_message, &signature);
        assert!(result.is_err());
    }

    #[test]
    fn test_ecdsa_p256_different_keys() {
        let keypair1 = EcdsaP256KeyPair::generate();
        let keypair2 = EcdsaP256KeyPair::generate();

        let message = b"Test message";
        let signature = keypair1.sign(message).unwrap();

        // Verification with different key should fail
        let result = EcdsaP256KeyPair::verify(keypair2.verifying_key(), message, &signature);
        assert!(result.is_err());
    }

    #[test]
    fn test_multi_signature_ed25519() {
        let message = b"Multi-signature test message";

        // Create 3 signers
        let keypair1 = Ed25519KeyPair::generate();
        let keypair2 = Ed25519KeyPair::generate();
        let keypair3 = Ed25519KeyPair::generate();

        // Create signatures
        let sig1 = keypair1.sign(message);
        let sig2 = keypair2.sign(message);
        let sig3 = keypair3.sign(message);

        // Create multi-signature with threshold 2
        let mut multi_sig = MultiSignature::new(2);

        multi_sig.add_signature(SignatureEntry {
            signer_id: "signer1".to_string(),
            signature: sig1,
            public_key: keypair1.verifying_key_bytes().to_vec(),
        });

        multi_sig.add_signature(SignatureEntry {
            signer_id: "signer2".to_string(),
            signature: sig2,
            public_key: keypair2.verifying_key_bytes().to_vec(),
        });

        multi_sig.add_signature(SignatureEntry {
            signer_id: "signer3".to_string(),
            signature: sig3,
            public_key: keypair3.verifying_key_bytes().to_vec(),
        });

        // Should be valid (3 signatures >= threshold of 2)
        assert!(multi_sig.verify_ed25519(message).unwrap());
    }

    #[test]
    fn test_multi_signature_threshold_not_met() {
        let message = b"Test message";

        let keypair = Ed25519KeyPair::generate();
        let sig = keypair.sign(message);

        // Create multi-signature with threshold 2 but only 1 signature
        let mut multi_sig = MultiSignature::new(2);

        multi_sig.add_signature(SignatureEntry {
            signer_id: "signer1".to_string(),
            signature: sig,
            public_key: keypair.verifying_key_bytes().to_vec(),
        });

        // Should be invalid (1 signature < threshold of 2)
        assert!(!multi_sig.verify_ed25519(message).unwrap());
    }

    #[test]
    fn test_multi_signature_ecdsa_p256() {
        let message = b"Multi-signature ECDSA test";

        let keypair1 = EcdsaP256KeyPair::generate();
        let keypair2 = EcdsaP256KeyPair::generate();

        let sig1 = keypair1.sign(message).unwrap();
        let sig2 = keypair2.sign(message).unwrap();

        let mut multi_sig = MultiSignature::new(2);

        multi_sig.add_signature(SignatureEntry {
            signer_id: "signer1".to_string(),
            signature: sig1,
            public_key: keypair1.verifying_key_bytes(),
        });

        multi_sig.add_signature(SignatureEntry {
            signer_id: "signer2".to_string(),
            signature: sig2,
            public_key: keypair2.verifying_key_bytes(),
        });

        assert!(multi_sig.verify_ecdsa_p256(message).unwrap());
    }

    #[test]
    fn test_multi_signature_one_invalid() {
        let message = b"Test message";

        let keypair1 = Ed25519KeyPair::generate();
        let keypair2 = Ed25519KeyPair::generate();

        let sig1 = keypair1.sign(message);
        let invalid_sig = vec![0u8; 64]; // Invalid signature

        let mut multi_sig = MultiSignature::new(2);

        multi_sig.add_signature(SignatureEntry {
            signer_id: "signer1".to_string(),
            signature: sig1,
            public_key: keypair1.verifying_key_bytes().to_vec(),
        });

        multi_sig.add_signature(SignatureEntry {
            signer_id: "signer2".to_string(),
            signature: invalid_sig,
            public_key: keypair2.verifying_key_bytes().to_vec(),
        });

        // Should be invalid (only 1 valid signature < threshold of 2)
        assert!(!multi_sig.verify_ed25519(message).unwrap());
    }
}
