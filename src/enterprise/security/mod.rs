// CADDY v0.1.5 - Enterprise Security & Encryption Module
// Copyright (c) 2025 CADDY Project
// Production-ready security and encryption services

pub mod encryption;
pub mod keystore;
pub mod vault;
pub mod signing;
pub mod integrity;
pub mod protection;
pub mod compliance;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    #[error("Key management error: {0}")]
    KeyManagement(String),

    #[error("Signature error: {0}")]
    Signature(String),

    #[error("Integrity error: {0}")]
    Integrity(String),

    #[error("Access denied: {0}")]
    AccessDenied(String),

    #[error("Compliance violation: {0}")]
    ComplianceViolation(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("HSM error: {0}")]
    Hsm(String),

    #[error("Certificate error: {0}")]
    Certificate(String),
}

pub type SecurityResult<T> = Result<T, SecurityError>;

/// Security level classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum SecurityLevel {
    Public,
    Internal,
    Confidential,
    Secret,
    TopSecret,
}

/// Encryption algorithm type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
    Rsa2048,
    Rsa4096,
    ChaCha20Poly1305,
}

/// Hash algorithm type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    Sha256,
    Sha3_256,
    Blake3,
    Sha512,
}

/// Key derivation function type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KdfAlgorithm {
    Pbkdf2,
    Argon2id,
    Scrypt,
}

// Re-export commonly used types
pub use encryption::{EncryptionService, EncryptedData, KeyPair};
pub use keystore::{KeyStore, KeyMetadata};
pub use vault::{Vault, Secret, SecretVersion};
pub use signing::{SigningService, Signature, Certificate};
pub use integrity::{IntegrityService, MerkleTree, IntegrityProof};
pub use protection::{DataProtection, DataClassification, RedactionPolicy};
pub use compliance::{ComplianceService, SecurityPolicy, ComplianceReport};
