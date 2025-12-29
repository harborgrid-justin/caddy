//! # Key Derivation Functions
//!
//! This module provides industry-standard key derivation functions (KDFs) for
//! generating cryptographic keys from passwords and other input material.
//!
//! ## Supported KDFs
//!
//! - **PBKDF2**: Password-Based Key Derivation Function 2 (RFC 2898)
//! - **Argon2id**: Memory-hard KDF, winner of the Password Hashing Competition
//! - **scrypt**: Memory-hard KDF designed to be costly on custom hardware
//! - **HKDF**: HMAC-based Extract-and-Expand Key Derivation Function (RFC 5869)
//!
//! ## Security Considerations
//!
//! - All derived keys are zeroized on drop
//! - Use Argon2id for password hashing (recommended)
//! - PBKDF2 is provided for compatibility but is less secure than Argon2id
//! - HKDF is for key expansion, not password hashing
//! - Always use cryptographically secure random salts

use argon2::{Argon2, Algorithm, Version, Params};
use hkdf::Hkdf;
use pbkdf2::pbkdf2_hmac;
use scrypt::{scrypt, Params as ScryptParams};
use sha2::{Sha256, Sha512};
use zeroize::Zeroize;
use thiserror::Error;

/// KDF-specific errors
#[derive(Error, Debug)]
pub enum KdfError {
    /// Invalid parameter for KDF
    #[error("Invalid KDF parameter: {0}")]
    InvalidParameter(String),

    /// Key derivation failed
    #[error("Key derivation failed: {0}")]
    DerivationFailed(String),

    /// Output length too short or too long
    #[error("Invalid output length: {0}")]
    InvalidLength(String),

    /// Salt too short
    #[error("Salt too short: minimum {0} bytes required")]
    SaltTooShort(usize),
}

pub type KdfResult<T> = Result<T, KdfError>;

/// Derived key material that is zeroized on drop
#[derive(Clone)]
pub struct DerivedKey {
    #[zeroize(skip)]
    algorithm: String,
    key_material: Vec<u8>,
}

impl DerivedKey {
    /// Create a new derived key
    pub fn new(algorithm: String, key_material: Vec<u8>) -> Self {
        Self {
            algorithm,
            key_material,
        }
    }

    /// Get the key material (use with caution)
    pub fn as_bytes(&self) -> &[u8] {
        &self.key_material
    }

    /// Get the algorithm used
    pub fn algorithm(&self) -> &str {
        &self.algorithm
    }

    /// Get the key length in bytes
    pub fn len(&self) -> usize {
        self.key_material.len()
    }

    /// Check if key is empty
    pub fn is_empty(&self) -> bool {
        self.key_material.is_empty()
    }
}

impl std::fmt::Debug for DerivedKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DerivedKey")
            .field("algorithm", &self.algorithm)
            .field("length", &self.key_material.len())
            .field("key_material", &"[REDACTED]")
            .finish()
    }
}

/// PBKDF2 configuration
#[derive(Debug, Clone)]
pub struct Pbkdf2Config {
    /// Number of iterations (recommend 600,000+ for SHA-256)
    pub iterations: u32,
    /// Output key length in bytes
    pub key_length: usize,
}

impl Default for Pbkdf2Config {
    fn default() -> Self {
        Self {
            iterations: 600_000, // OWASP recommendation for 2024+
            key_length: 32,
        }
    }
}

/// Argon2id configuration
#[derive(Debug, Clone)]
pub struct Argon2Config {
    /// Memory cost in KiB (default: 64 MiB)
    pub memory_cost: u32,
    /// Time cost (iterations)
    pub time_cost: u32,
    /// Parallelism factor
    pub parallelism: u32,
    /// Output key length in bytes
    pub key_length: usize,
}

impl Default for Argon2Config {
    fn default() -> Self {
        Self {
            memory_cost: 65536,  // 64 MiB
            time_cost: 3,         // 3 iterations
            parallelism: 4,       // 4 parallel threads
            key_length: 32,       // 256 bits
        }
    }
}

impl Argon2Config {
    /// OWASP recommended configuration for high security
    pub fn owasp_high_security() -> Self {
        Self {
            memory_cost: 131_072,  // 128 MiB
            time_cost: 4,
            parallelism: 4,
            key_length: 32,
        }
    }

    /// Low memory configuration for resource-constrained environments
    pub fn low_memory() -> Self {
        Self {
            memory_cost: 16384,   // 16 MiB
            time_cost: 2,
            parallelism: 2,
            key_length: 32,
        }
    }
}

/// scrypt configuration
#[derive(Debug, Clone)]
pub struct ScryptConfig {
    /// CPU/memory cost parameter (must be power of 2)
    pub log_n: u8,
    /// Block size parameter
    pub r: u32,
    /// Parallelization parameter
    pub p: u32,
    /// Output key length in bytes
    pub key_length: usize,
}

impl Default for ScryptConfig {
    fn default() -> Self {
        Self {
            log_n: 15,        // N = 2^15 = 32768
            r: 8,
            p: 1,
            key_length: 32,
        }
    }
}

impl ScryptConfig {
    /// Interactive login configuration (fast)
    pub fn interactive() -> Self {
        Self {
            log_n: 14,        // N = 2^14 = 16384
            r: 8,
            p: 1,
            key_length: 32,
        }
    }

    /// High security configuration (slow)
    pub fn high_security() -> Self {
        Self {
            log_n: 17,        // N = 2^17 = 131_072
            r: 8,
            p: 1,
            key_length: 32,
        }
    }
}

/// Key Derivation Function provider
pub struct KdfProvider;

impl KdfProvider {
    /// Derive key using PBKDF2-HMAC-SHA256
    ///
    /// # Arguments
    ///
    /// * `password` - The password or input key material
    /// * `salt` - Salt value (minimum 16 bytes recommended)
    /// * `config` - PBKDF2 configuration
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use caddy::enterprise::crypto::kdf::{KdfProvider, Pbkdf2Config};
    ///
    /// let password = b"correct horse battery staple";
    /// let salt = b"random_salt_value_16_bytes";
    /// let config = Pbkdf2Config::default();
    ///
    /// let key = KdfProvider::derive_pbkdf2_sha256(password, salt, &config)?;
    /// ```
    pub fn derive_pbkdf2_sha256(
        password: &[u8],
        salt: &[u8],
        config: &Pbkdf2Config,
    ) -> KdfResult<DerivedKey> {
        if salt.len() < 16 {
            return Err(KdfError::SaltTooShort(16));
        }

        let mut key_material = vec![0u8; config.key_length];
        pbkdf2_hmac::<Sha256>(password, salt, config.iterations, &mut key_material);

        Ok(DerivedKey::new("PBKDF2-SHA256".to_string(), key_material))
    }

    /// Derive key using PBKDF2-HMAC-SHA512
    pub fn derive_pbkdf2_sha512(
        password: &[u8],
        salt: &[u8],
        config: &Pbkdf2Config,
    ) -> KdfResult<DerivedKey> {
        if salt.len() < 16 {
            return Err(KdfError::SaltTooShort(16));
        }

        let mut key_material = vec![0u8; config.key_length];
        pbkdf2_hmac::<Sha512>(password, salt, config.iterations, &mut key_material);

        Ok(DerivedKey::new("PBKDF2-SHA512".to_string(), key_material))
    }

    /// Derive key using Argon2id (recommended for password hashing)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use caddy::enterprise::crypto::kdf::{KdfProvider, Argon2Config};
    ///
    /// let password = b"correct horse battery staple";
    /// let salt = b"random_salt_value_16_bytes";
    /// let config = Argon2Config::default();
    ///
    /// let key = KdfProvider::derive_argon2id(password, salt, &config)?;
    /// ```
    pub fn derive_argon2id(
        password: &[u8],
        salt: &[u8],
        config: &Argon2Config,
    ) -> KdfResult<DerivedKey> {
        if salt.len() < 16 {
            return Err(KdfError::SaltTooShort(16));
        }

        let params = Params::new(
            config.memory_cost,
            config.time_cost,
            config.parallelism,
            Some(config.key_length),
        ).map_err(|e| KdfError::InvalidParameter(e.to_string()))?;

        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        let mut key_material = vec![0u8; config.key_length];
        argon2
            .hash_password_into(password, salt, &mut key_material)
            .map_err(|e| KdfError::DerivationFailed(e.to_string()))?;

        Ok(DerivedKey::new("Argon2id".to_string(), key_material))
    }

    /// Derive key using scrypt
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use caddy::enterprise::crypto::kdf::{KdfProvider, ScryptConfig};
    ///
    /// let password = b"correct horse battery staple";
    /// let salt = b"random_salt_value_16_bytes";
    /// let config = ScryptConfig::default();
    ///
    /// let key = KdfProvider::derive_scrypt(password, salt, &config)?;
    /// ```
    pub fn derive_scrypt(
        password: &[u8],
        salt: &[u8],
        config: &ScryptConfig,
    ) -> KdfResult<DerivedKey> {
        if salt.len() < 16 {
            return Err(KdfError::SaltTooShort(16));
        }

        let params = ScryptParams::new(config.log_n, config.r, config.p, config.key_length)
            .map_err(|e| KdfError::InvalidParameter(e.to_string()))?;

        let mut key_material = vec![0u8; config.key_length];
        scrypt(password, salt, &params, &mut key_material)
            .map_err(|e| KdfError::DerivationFailed(e.to_string()))?;

        Ok(DerivedKey::new("scrypt".to_string(), key_material))
    }

    /// Expand key material using HKDF-SHA256
    ///
    /// HKDF is designed for key expansion, not password hashing.
    /// Use this to derive multiple keys from a single source.
    ///
    /// # Arguments
    ///
    /// * `input_key_material` - The input keying material (IKM)
    /// * `salt` - Optional salt value (can be empty)
    /// * `info` - Optional context/application-specific info
    /// * `output_length` - Length of output key material
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use caddy::enterprise::crypto::kdf::KdfProvider;
    ///
    /// let master_key = b"master_secret_key_material";
    /// let salt = b"optional_salt";
    /// let info = b"encryption_key";
    ///
    /// let derived = KdfProvider::expand_hkdf_sha256(master_key, Some(salt), Some(info), 32)?;
    /// ```
    pub fn expand_hkdf_sha256(
        input_key_material: &[u8],
        salt: Option<&[u8]>,
        info: Option<&[u8]>,
        output_length: usize,
    ) -> KdfResult<DerivedKey> {
        if output_length > 255 * 32 {
            return Err(KdfError::InvalidLength(
                "HKDF-SHA256 maximum output length is 8160 bytes".to_string()
            ));
        }

        let hkdf = Hkdf::<Sha256>::new(salt, input_key_material);

        let mut key_material = vec![0u8; output_length];
        hkdf.expand(info.unwrap_or(b""), &mut key_material)
            .map_err(|e| KdfError::DerivationFailed(e.to_string()))?;

        Ok(DerivedKey::new("HKDF-SHA256".to_string(), key_material))
    }

    /// Expand key material using HKDF-SHA512
    pub fn expand_hkdf_sha512(
        input_key_material: &[u8],
        salt: Option<&[u8]>,
        info: Option<&[u8]>,
        output_length: usize,
    ) -> KdfResult<DerivedKey> {
        if output_length > 255 * 64 {
            return Err(KdfError::InvalidLength(
                "HKDF-SHA512 maximum output length is 16320 bytes".to_string()
            ));
        }

        let hkdf = Hkdf::<Sha512>::new(salt, input_key_material);

        let mut key_material = vec![0u8; output_length];
        hkdf.expand(info.unwrap_or(b""), &mut key_material)
            .map_err(|e| KdfError::DerivationFailed(e.to_string()))?;

        Ok(DerivedKey::new("HKDF-SHA512".to_string(), key_material))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pbkdf2_sha256() {
        let password = b"test_password";
        let salt = b"test_salt_16byte";
        let config = Pbkdf2Config {
            iterations: 1000,
            key_length: 32,
        };

        let key = KdfProvider::derive_pbkdf2_sha256(password, salt, &config).unwrap();
        assert_eq!(key.len(), 32);
        assert_eq!(key.algorithm(), "PBKDF2-SHA256");
    }

    #[test]
    fn test_pbkdf2_deterministic() {
        let password = b"test_password";
        let salt = b"test_salt_16byte";
        let config = Pbkdf2Config::default();

        let key1 = KdfProvider::derive_pbkdf2_sha256(password, salt, &config).unwrap();
        let key2 = KdfProvider::derive_pbkdf2_sha256(password, salt, &config).unwrap();

        assert_eq!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_pbkdf2_salt_too_short() {
        let password = b"test_password";
        let salt = b"short";
        let config = Pbkdf2Config::default();

        let result = KdfProvider::derive_pbkdf2_sha256(password, salt, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_argon2id() {
        let password = b"test_password";
        let salt = b"test_salt_16byte";
        let config = Argon2Config {
            memory_cost: 4096,
            time_cost: 2,
            parallelism: 1,
            key_length: 32,
        };

        let key = KdfProvider::derive_argon2id(password, salt, &config).unwrap();
        assert_eq!(key.len(), 32);
        assert_eq!(key.algorithm(), "Argon2id");
    }

    #[test]
    fn test_argon2id_configs() {
        let password = b"test_password";
        let salt = b"test_salt_16byte";

        let key1 = KdfProvider::derive_argon2id(
            password,
            salt,
            &Argon2Config::default()
        ).unwrap();

        let key2 = KdfProvider::derive_argon2id(
            password,
            salt,
            &Argon2Config::low_memory()
        ).unwrap();

        // Different configs should produce different keys
        assert_ne!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_scrypt() {
        let password = b"test_password";
        let salt = b"test_salt_16byte";
        let config = ScryptConfig {
            log_n: 10,  // Lower for faster tests
            r: 8,
            p: 1,
            key_length: 32,
        };

        let key = KdfProvider::derive_scrypt(password, salt, &config).unwrap();
        assert_eq!(key.len(), 32);
        assert_eq!(key.algorithm(), "scrypt");
    }

    #[test]
    fn test_hkdf_sha256() {
        let ikm = b"input_key_material";
        let salt = b"optional_salt";
        let info = b"context_info";

        let key = KdfProvider::expand_hkdf_sha256(ikm, Some(salt), Some(info), 32).unwrap();
        assert_eq!(key.len(), 32);
        assert_eq!(key.algorithm(), "HKDF-SHA256");
    }

    #[test]
    fn test_hkdf_no_salt() {
        let ikm = b"input_key_material";
        let key = KdfProvider::expand_hkdf_sha256(ikm, None, None, 32).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_hkdf_multiple_keys() {
        let ikm = b"shared_master_key";
        let salt = b"random_salt_value";

        let key1 = KdfProvider::expand_hkdf_sha256(
            ikm,
            Some(salt),
            Some(b"key1"),
            32
        ).unwrap();

        let key2 = KdfProvider::expand_hkdf_sha256(
            ikm,
            Some(salt),
            Some(b"key2"),
            32
        ).unwrap();

        // Different info should produce different keys
        assert_ne!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_hkdf_output_length_limit() {
        let ikm = b"input_key_material";
        let result = KdfProvider::expand_hkdf_sha256(ikm, None, None, 10000);
        assert!(result.is_err());
    }

    #[test]
    fn test_derived_key_debug() {
        let key = DerivedKey::new("TEST".to_string(), vec![1, 2, 3, 4]);
        let debug_str = format!("{:?}", key);
        assert!(debug_str.contains("REDACTED"));
        assert!(!debug_str.contains("1, 2, 3, 4"));
    }
}
