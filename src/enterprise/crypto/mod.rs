//! # CADDY Enterprise Cryptographic Infrastructure
//!
//! This module provides a comprehensive cryptographic infrastructure for CADDY v0.2.0,
//! including encryption, key management, digital signatures, and advanced cryptographic
//! primitives.
//!
//! ## Overview
//!
//! The crypto module is organized into specialized submodules:
//!
//! - **Key Derivation** ([`kdf`]): PBKDF2, Argon2id, scrypt, HKDF
//! - **Symmetric Encryption** ([`symmetric`]): AES-256-GCM, ChaCha20-Poly1305, XChaCha20-Poly1305
//! - **Asymmetric Encryption** ([`asymmetric`]): RSA-OAEP, ECIES (Curve25519)
//! - **Key Management** ([`keystore`]): Secure key storage, rotation, versioning
//! - **Envelope Encryption** ([`envelope`]): Multi-recipient encryption with DEK/KEK
//! - **HSM Integration** ([`hsm`]): Hardware Security Module abstraction (PKCS#11)
//! - **Zero-Knowledge Proofs** ([`zkp`]): Commitments, range proofs, Merkle trees
//! - **Digital Signatures** ([`signature`]): Ed25519, ECDSA P-256, multi-signatures
//!
//! ## Security Features
//!
//! ### Memory Safety
//!
//! - All sensitive data (keys, plaintexts) is zeroized on drop
//! - No sensitive data in debug output or serialization
//! - Constant-time operations where applicable
//!
//! ### Cryptographic Agility
//!
//! - Support for multiple algorithms per operation
//! - Algorithm selection based on security requirements
//! - Forward compatibility for algorithm upgrades
//!
//! ### Key Management
//!
//! - Secure key generation using OS random number generator
//! - Encrypted key storage with master key protection
//! - Automatic key rotation policies
//! - Key versioning and lifecycle management
//!
//! ### Compliance
//!
//! - FIPS 140-2 compliant algorithms (when using certified implementations)
//! - NIST recommendations followed
//! - Industry best practices (OWASP, etc.)
//!
//! ## Usage Examples
//!
//! ### Symmetric Encryption
//!
//! ```rust,ignore
//! use caddy::enterprise::crypto::symmetric::Aes256GcmCipher;
//!
//! // Generate a random key (in practice, use KDF or key management)
//! let key = [0u8; 32];
//! let cipher = Aes256GcmCipher::new(&key)?;
//!
//! // Encrypt data
//! let plaintext = b"Secret data";
//! let encrypted = cipher.encrypt(plaintext, None)?;
//!
//! // Decrypt data
//! let decrypted = cipher.decrypt(&encrypted)?;
//! assert_eq!(plaintext, decrypted.as_slice());
//! ```
//!
//! ### Key Derivation
//!
//! ```rust,ignore
//! use caddy::enterprise::crypto::kdf::{KdfProvider, Argon2Config};
//!
//! let password = b"user_password";
//! let salt = b"random_salt_16_bytes_min";
//!
//! // Derive encryption key from password
//! let key = KdfProvider::derive_argon2id(
//!     password,
//!     salt,
//!     &Argon2Config::default()
//! )?;
//! ```
//!
//! ### Digital Signatures
//!
//! ```rust,ignore
//! use caddy::enterprise::crypto::signature::Ed25519KeyPair;
//!
//! // Generate signing key
//! let keypair = Ed25519KeyPair::generate();
//!
//! // Sign a message
//! let message = b"Message to sign";
//! let signature = keypair.sign(message);
//!
//! // Verify signature
//! Ed25519KeyPair::verify(
//!     keypair.verifying_key(),
//!     message,
//!     &signature
//! )?;
//! ```
//!
//! ### Envelope Encryption (Multi-Recipient)
//!
//! ```rust,ignore
//! use caddy::enterprise::crypto::envelope::{EnvelopeBuilder, EncryptionAlgorithm};
//! use caddy::enterprise::crypto::asymmetric::{RsaKeyPair, RsaKeySize};
//!
//! // Generate recipient keys
//! let alice_key = RsaKeyPair::generate(RsaKeySize::Bits2048)?;
//! let bob_key = RsaKeyPair::generate(RsaKeySize::Bits2048)?;
//!
//! // Encrypt for multiple recipients
//! let plaintext = b"Shared secret";
//! let envelope = EnvelopeBuilder::new(EncryptionAlgorithm::Aes256Gcm)
//!     .add_rsa_recipient("alice".to_string(), alice_key.public_key().clone())
//!     .add_rsa_recipient("bob".to_string(), bob_key.public_key().clone())
//!     .encrypt(plaintext, None)?;
//!
//! // Either Alice or Bob can decrypt
//! let decrypted = envelope::decrypt_with_rsa(&envelope, "alice", &alice_key)?;
//! ```
//!
//! ### Key Store Management
//!
//! ```rust,ignore
//! use caddy::enterprise::crypto::keystore::{KeyStore, KeyPurpose};
//!
//! // Create encrypted key store
//! let mut store = KeyStore::new(b"master_password")?;
//!
//! // Store an encryption key
//! let key_material = [0u8; 32]; // Your key
//! let key_id = store.store_key(
//!     "my_encryption_key".to_string(),
//!     KeyPurpose::Encryption,
//!     "AES-256-GCM".to_string(),
//!     &key_material,
//!     None, // No rotation policy
//! )?;
//!
//! // Retrieve key later
//! let key = store.get_key(&key_id)?;
//! ```
//!
//! ## Security Best Practices
//!
//! ### Encryption
//!
//! 1. **Always use authenticated encryption (AEAD)**
//!    - Use AES-GCM, ChaCha20-Poly1305, or XChaCha20-Poly1305
//!    - Never use unauthenticated modes (ECB, CBC without MAC)
//!
//! 2. **Never reuse nonces**
//!    - Each encryption must use a unique nonce
//!    - Use random nonces or a counter
//!    - Consider XChaCha20 for larger nonce space
//!
//! 3. **Use strong keys**
//!    - Generate keys using cryptographically secure RNG
//!    - Use KDFs (Argon2id) for password-derived keys
//!    - Minimum 256-bit keys for symmetric encryption
//!
//! ### Key Management
//!
//! 1. **Protect private keys**
//!    - Never log or print private keys
//!    - Encrypt keys at rest
//!    - Use HSM for high-security environments
//!
//! 2. **Implement key rotation**
//!    - Regular key rotation (90 days recommended)
//!    - Automated rotation policies
//!    - Keep old keys for decryption only
//!
//! 3. **Separate key purposes**
//!    - Different keys for encryption vs. signing
//!    - Don't reuse keys across systems
//!    - Use envelope encryption for data at rest
//!
//! ### Password Handling
//!
//! 1. **Use Argon2id for password hashing**
//!    - OWASP recommended parameters minimum
//!    - Increase parameters as hardware improves
//!    - Use unique salt per password
//!
//! 2. **Never store plaintext passwords**
//!    - Always hash passwords before storage
//!    - Use timing-safe comparison
//!    - Implement rate limiting
//!
//! ### Digital Signatures
//!
//! 1. **Hash before signing**
//!    - Don't sign untrusted data directly
//!    - Use SHA-256 or stronger
//!    - Prevent signature malleability
//!
//! 2. **Verify all signatures**
//!    - Don't trust data without verification
//!    - Check signer identity
//!    - Validate certificate chains
//!
//! ## Algorithm Selection Guide
//!
//! ### Symmetric Encryption
//!
//! - **AES-256-GCM**: Standard choice, hardware accelerated on most platforms
//! - **ChaCha20-Poly1305**: Constant-time, good for software-only implementations
//! - **XChaCha20-Poly1305**: Like ChaCha20 but with 192-bit nonces (safer for random nonces)
//!
//! ### Key Derivation
//!
//! - **Argon2id**: Best choice for password hashing (recommended)
//! - **PBKDF2**: Legacy support, compatibility
//! - **scrypt**: Memory-hard alternative
//! - **HKDF**: For key expansion, not password hashing
//!
//! ### Asymmetric Encryption
//!
//! - **ECIES (Curve25519)**: Modern, fast, smaller keys
//! - **RSA-OAEP**: Standard, widely supported, larger keys
//!
//! ### Digital Signatures
//!
//! - **Ed25519**: Fast, small signatures, deterministic
//! - **ECDSA P-256**: Standard, widely supported
//!
//! ## Compliance Standards
//!
//! ### FIPS 140-2
//!
//! The following algorithms are FIPS 140-2 approved:
//! - AES-256-GCM
//! - SHA-256, SHA-512
//! - HMAC
//! - RSA (2048+ bits)
//! - ECDSA P-256
//!
//! Note: ChaCha20, Ed25519, and Argon2id are not FIPS approved but are
//! cryptographically sound and recommended by security experts.
//!
//! ### NIST Recommendations
//!
//! - Minimum 128-bit security level (256-bit symmetric keys)
//! - SHA-256 or stronger for hashing
//! - 2048-bit RSA minimum (4096-bit for long-term)
//! - P-256 or stronger for elliptic curves
//!
//! ## Performance Considerations
//!
//! ### Encryption Speed
//!
//! 1. **AES-GCM**: ~3-5 GB/s with AES-NI hardware acceleration
//! 2. **ChaCha20-Poly1305**: ~1-2 GB/s in software
//! 3. **RSA**: ~1000 operations/second (2048-bit)
//! 4. **ECIES**: ~10000 operations/second (Curve25519)
//!
//! ### Key Derivation Tuning
//!
//! - **Argon2id**: Adjust memory_cost and time_cost based on available resources
//! - **PBKDF2**: Increase iterations (aim for 100ms+ derivation time)
//! - **scrypt**: Balance N, r, p parameters for security vs. performance
//!
//! ## Thread Safety
//!
//! All cryptographic operations in this module are thread-safe. Key material is
//! protected with appropriate synchronization primitives where necessary.
//!
//! ## Error Handling
//!
//! All operations return `Result` types with descriptive error messages.
//! Never ignore cryptographic errors - they indicate security-critical failures.
//!
//! ## Security Auditing
//!
//! For security-critical applications:
//! 1. Conduct regular security audits
//! 2. Use static analysis tools
//! 3. Keep dependencies updated
//! 4. Monitor security advisories
//! 5. Consider third-party penetration testing
//!
//! ## Support and Documentation
//!
//! For detailed documentation on each submodule, see the respective module documentation:
//! - [`kdf`] - Key derivation functions
//! - [`symmetric`] - Symmetric encryption
//! - [`asymmetric`] - Asymmetric encryption
//! - [`keystore`] - Key management
//! - [`envelope`] - Envelope encryption
//! - [`hsm`] - HSM integration
//! - [`zkp`] - Zero-knowledge proofs
//! - [`signature`] - Digital signatures

#![warn(missing_docs)]
#![warn(clippy::all)]
#![allow(clippy::module_name_repetitions)]

// ============================================================================
// Module Declarations
// ============================================================================

/// Key Derivation Functions (KDF)
///
/// Provides PBKDF2, Argon2id, scrypt, and HKDF for deriving cryptographic
/// keys from passwords and other input material.
pub mod kdf;

/// Symmetric Encryption
///
/// Authenticated encryption using AES-256-GCM, ChaCha20-Poly1305, and
/// XChaCha20-Poly1305. All algorithms provide AEAD (Authenticated Encryption
/// with Associated Data).
pub mod symmetric;

/// Asymmetric Encryption
///
/// Public-key encryption using RSA-OAEP and ECIES (Elliptic Curve Integrated
/// Encryption Scheme) with Curve25519.
pub mod asymmetric;

/// Key Store and Management
///
/// Secure storage, rotation, versioning, and lifecycle management for
/// cryptographic keys. Keys are encrypted at rest using a master key.
pub mod keystore;

/// Envelope Encryption
///
/// Implementation of envelope encryption pattern where data is encrypted with
/// a Data Encryption Key (DEK), and the DEK is encrypted with one or more
/// Key Encryption Keys (KEK). Supports multi-recipient encryption.
pub mod envelope;

/// Hardware Security Module (HSM) Integration
///
/// Abstraction layer for PKCS#11 compatible Hardware Security Modules.
/// Provides secure key generation, storage, and cryptographic operations
/// in tamper-resistant hardware.
pub mod hsm;

/// Zero-Knowledge Proofs (ZKP)
///
/// Basic zero-knowledge proof primitives including commitment schemes,
/// range proofs, and Merkle tree proofs. Allows proving statements
/// without revealing underlying data.
pub mod zkp;

/// Digital Signatures
///
/// Digital signature algorithms including Ed25519 (EdDSA) and ECDSA P-256.
/// Provides signing, verification, and multi-signature support.
pub mod signature;

// ============================================================================
// Re-exports for Convenience
// ============================================================================

// KDF exports
pub use kdf::{
    KdfProvider, KdfError, KdfResult,
    DerivedKey, Pbkdf2Config, Argon2Config, ScryptConfig,
};

// Symmetric encryption exports
pub use symmetric::{
    Aes256GcmCipher, ChaCha20Poly1305Cipher, XChaCha20Poly1305Cipher,
    EncryptedData, SymmetricError, SymmetricResult,
};

// Asymmetric encryption exports
pub use asymmetric::{
    RsaKeyPair, RsaKeySize, EciesKeyPair, EciesEncrypted,
    AsymmetricError, AsymmetricResult,
};

// Key store exports
pub use keystore::{
    KeyStore, KeyMetadata, KeyMaterial, KeyPurpose, RotationPolicy,
    KeyStoreError, KeyStoreResult,
};

// Envelope encryption exports
pub use envelope::{
    Envelope, EnvelopeBuilder, EncryptedDek, EncryptionAlgorithm,
    KeyEncryptionMethod, EnvelopeError, EnvelopeResult,
    decrypt_with_rsa, decrypt_with_ecies,
};

// HSM exports
pub use hsm::{
    HsmInterface, SessionHandle, KeyHandle, HsmKeyType,
    KeyAttributes, SignatureAlgorithm as HsmSignatureAlgorithm,
    HsmError, HsmResult,
};

// ZKP exports
pub use zkp::{
    HashCommitment, RangeProof, MerkleTree, MerkleProof,
    ZkpError, ZkpResult,
};

// Signature exports
pub use signature::{
    Ed25519KeyPair, EcdsaP256KeyPair,
    MultiSignature, SignatureEntry,
    SignatureError, SignatureResult,
};

// ============================================================================
// Module Version and Metadata
// ============================================================================

/// Crypto module version
pub const CRYPTO_VERSION: &str = "0.2.0";

/// Build date
pub const BUILD_DATE: &str = "2025-12-28";

/// Supported cryptographic algorithms
pub const SUPPORTED_ALGORITHMS: &[&str] = &[
    "AES-256-GCM",
    "ChaCha20-Poly1305",
    "XChaCha20-Poly1305",
    "RSA-OAEP",
    "ECIES-Curve25519",
    "Ed25519",
    "ECDSA-P256",
    "PBKDF2-SHA256",
    "PBKDF2-SHA512",
    "Argon2id",
    "scrypt",
    "HKDF-SHA256",
    "HKDF-SHA512",
];

/// Get module version information
pub fn version_info() -> String {
    format!(
        "CADDY Crypto Module v{} (built {})",
        CRYPTO_VERSION,
        BUILD_DATE
    )
}

/// Get list of supported algorithms
pub fn supported_algorithms() -> &'static [&'static str] {
    SUPPORTED_ALGORITHMS
}

// ============================================================================
// Security Level Helpers
// ============================================================================

/// Security level for cryptographic operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecurityLevel {
    /// 128-bit security (minimum recommended)
    Standard,
    /// 192-bit security (high security)
    High,
    /// 256-bit security (maximum security)
    Maximum,
}

impl SecurityLevel {
    /// Get recommended symmetric key size in bytes
    pub fn symmetric_key_size(&self) -> usize {
        match self {
            SecurityLevel::Standard => 16,  // 128 bits
            SecurityLevel::High => 24,      // 192 bits
            SecurityLevel::Maximum => 32,   // 256 bits
        }
    }

    /// Get recommended RSA key size in bits
    pub fn rsa_key_size(&self) -> RsaKeySize {
        match self {
            SecurityLevel::Standard => RsaKeySize::Bits2048,
            SecurityLevel::High => RsaKeySize::Bits3072,
            SecurityLevel::Maximum => RsaKeySize::Bits4096,
        }
    }

    /// Get recommended Argon2 configuration
    pub fn argon2_config(&self) -> Argon2Config {
        match self {
            SecurityLevel::Standard => Argon2Config::default(),
            SecurityLevel::High => Argon2Config::owasp_high_security(),
            SecurityLevel::Maximum => Argon2Config {
                memory_cost: 262_144,  // 256 MiB
                time_cost: 5,
                parallelism: 4,
                key_length: 32,
            },
        }
    }
}

// ============================================================================
// Module Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        let version = version_info();
        assert!(version.contains("0.2.0"));
    }

    #[test]
    fn test_supported_algorithms() {
        let algorithms = supported_algorithms();
        assert!(algorithms.contains(&"AES-256-GCM"));
        assert!(algorithms.contains(&"Ed25519"));
        assert!(algorithms.contains(&"Argon2id"));
    }

    #[test]
    fn test_security_level_key_sizes() {
        assert_eq!(SecurityLevel::Standard.symmetric_key_size(), 16);
        assert_eq!(SecurityLevel::High.symmetric_key_size(), 24);
        assert_eq!(SecurityLevel::Maximum.symmetric_key_size(), 32);
    }

    #[test]
    fn test_security_level_ordering() {
        assert!(SecurityLevel::Standard < SecurityLevel::High);
        assert!(SecurityLevel::High < SecurityLevel::Maximum);
    }

    /// Integration test: Full encryption workflow
    #[test]
    fn test_integration_symmetric_encryption() {
        let key = [0u8; 32];
        let cipher = Aes256GcmCipher::new(&key).unwrap();

        let plaintext = b"Integration test message";
        let encrypted = cipher.encrypt(plaintext, None).unwrap();
        let decrypted = cipher.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    /// Integration test: Key derivation and encryption
    #[test]
    fn test_integration_kdf_and_encryption() {
        let password = b"test_password";
        let salt = b"test_salt_16byte";

        // Derive key from password
        let derived = KdfProvider::derive_argon2id(
            password,
            salt,
            &Argon2Config::low_memory()
        ).unwrap();

        // Use derived key for encryption
        let cipher = Aes256GcmCipher::new(derived.as_bytes()).unwrap();
        let plaintext = b"Secret data";
        let encrypted = cipher.encrypt(plaintext, None).unwrap();
        let decrypted = cipher.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    /// Integration test: Digital signatures
    #[test]
    fn test_integration_signatures() {
        let keypair = Ed25519KeyPair::generate();
        let message = b"Message to sign";

        let signature = keypair.sign(message);
        let result = Ed25519KeyPair::verify(
            keypair.verifying_key(),
            message,
            &signature
        );

        assert!(result.is_ok());
    }
}
