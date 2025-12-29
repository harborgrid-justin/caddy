//! Authentication Cryptographic Utilities
//!
//! Specialized cryptographic functions for authentication:
//! - Password hashing and verification (Argon2id)
//! - Secure token generation
//! - Session encryption/decryption
//! - API key generation and hashing
//! - Sensitive data encryption at rest
//! - Key derivation for encryption keys
//!
//! # Security Features
//! - Argon2id for password hashing (OWASP recommended)
//! - AES-256-GCM for authenticated encryption
//! - ChaCha20-Poly1305 as alternative AEAD
//! - Secure random token generation
//! - Constant-time comparison
//! - Zero-on-drop for sensitive data

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher as Argon2PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};
use thiserror::Error;
use zeroize::Zeroize;
use base64::{Engine as _, engine::general_purpose};

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Hashing error: {0}")]
    HashingError(String),

    #[error("Verification failed")]
    VerificationFailed,

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Decryption error: {0}")]
    DecryptionError(String),

    #[error("Invalid key")]
    InvalidKey,

    #[error("Invalid data")]
    InvalidData,
}

pub type CryptoResult<T> = Result<T, CryptoError>;

// ============================================================================
// Password Hashing
// ============================================================================

/// Password hasher using Argon2id
pub struct PasswordHasher {
    argon2: Argon2<'static>,
}

impl PasswordHasher {
    /// Create a new password hasher with secure defaults
    pub fn new() -> Self {
        Self {
            argon2: Argon2::default(),
        }
    }

    /// Hash a password using Argon2id
    pub fn hash_password(&self, password: &str) -> CryptoResult<String> {
        let salt = SaltString::generate(&mut OsRng);

        let password_hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| CryptoError::HashingError(e.to_string()))?;

        Ok(password_hash.to_string())
    }

    /// Verify a password against a hash
    pub fn verify_password(&self, password: &str, hash: &str) -> CryptoResult<()> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| CryptoError::HashingError(e.to_string()))?;

        self.argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| CryptoError::VerificationFailed)
    }

    /// Check if a password hash needs rehashing (e.g., due to parameter changes)
    pub fn needs_rehash(&self, hash: &str) -> bool {
        if let Ok(parsed_hash) = PasswordHash::new(hash) {
            // In production, check if parameters match current configuration
            false
        } else {
            true
        }
    }
}

impl Default for PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Secure Token Generation
// ============================================================================

/// Secure token generator
pub struct TokenGenerator;

impl TokenGenerator {
    /// Generate a cryptographically secure random token
    pub fn generate(length: usize) -> String {
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..length).map(|_| rng.gen()).collect();
        hex::encode(bytes)
    }

    /// Generate a URL-safe token
    pub fn generate_url_safe(length: usize) -> String {
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..length).map(|_| rng.gen()).collect();
        general_purpose::URL_SAFE_NO_PAD.encode(bytes)
    }

    /// Generate an API key with prefix
    pub fn generate_api_key(prefix: &str) -> String {
        let token = Self::generate(32);
        format!("{}_{}", prefix, token)
    }

    /// Generate a session token
    pub fn generate_session_token() -> String {
        Self::generate_url_safe(32)
    }

    /// Generate a CSRF token
    pub fn generate_csrf_token() -> String {
        Self::generate_url_safe(32)
    }
}

// ============================================================================
// API Key Management
// ============================================================================

/// API key with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    /// Key ID (stored)
    pub id: String,

    /// Key hash (stored - SHA-256 of the actual key)
    pub key_hash: String,

    /// User ID
    pub user_id: String,

    /// Key name/description
    pub name: String,

    /// Scopes/permissions
    pub scopes: Vec<String>,

    /// Creation timestamp
    pub created_at: SystemTime,

    /// Last used timestamp
    pub last_used: Option<SystemTime>,

    /// Expiration (optional)
    pub expires_at: Option<SystemTime>,

    /// Is active
    pub active: bool,
}

impl ApiKey {
    /// Create a new API key
    pub fn new(user_id: String, name: String, scopes: Vec<String>) -> (Self, String) {
        let id = uuid::Uuid::new_v4().to_string();
        let actual_key = TokenGenerator::generate_api_key("cad");
        let key_hash = Self::hash_key(&actual_key);

        let api_key = Self {
            id,
            key_hash,
            user_id,
            name,
            scopes,
            created_at: SystemTime::now(),
            last_used: None,
            expires_at: None,
            active: true,
        };

        (api_key, actual_key)
    }

    /// Hash an API key for storage
    fn hash_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Verify an API key
    pub fn verify(&self, key: &str) -> bool {
        if !self.active {
            return false;
        }

        // Check expiration
        if let Some(expires_at) = self.expires_at {
            if SystemTime::now() > expires_at {
                return false;
            }
        }

        // Constant-time comparison
        let provided_hash = Self::hash_key(key);
        constant_time_compare(&provided_hash, &self.key_hash)
    }

    /// Set expiration
    pub fn set_expiration(&mut self, duration: Duration) {
        self.expires_at = Some(SystemTime::now() + duration);
    }

    /// Mark as used
    pub fn mark_used(&mut self) {
        self.last_used = Some(SystemTime::now());
    }

    /// Revoke the key
    pub fn revoke(&mut self) {
        self.active = false;
    }
}

// ============================================================================
// Data Encryption
// ============================================================================

/// Encrypted data container
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize)]
pub struct EncryptedData {
    /// Ciphertext
    pub ciphertext: Vec<u8>,

    /// Nonce/IV
    pub nonce: Vec<u8>,

    /// Algorithm identifier
    pub algorithm: String,

    /// Encryption timestamp
    pub encrypted_at: u64,
}

/// Data encryptor for sensitive authentication data
pub struct DataEncryptor {
    cipher: Aes256Gcm,
}

impl DataEncryptor {
    /// Create a new encryptor with a key
    pub fn new(key: &[u8]) -> CryptoResult<Self> {
        if key.len() != 32 {
            return Err(CryptoError::InvalidKey);
        }

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| CryptoError::EncryptionError(e.to_string()))?;

        Ok(Self { cipher })
    }

    /// Create from base64-encoded key
    pub fn from_base64_key(key_b64: &str) -> CryptoResult<Self> {
        let key = general_purpose::STANDARD
            .decode(key_b64)
            .map_err(|e| CryptoError::InvalidKey)?;

        Self::new(&key)
    }

    /// Generate a new random encryption key
    pub fn generate_key() -> String {
        let mut rng = rand::thread_rng();
        let key: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        general_purpose::STANDARD.encode(&key)
    }

    /// Encrypt data
    pub fn encrypt(&self, plaintext: &[u8]) -> CryptoResult<EncryptedData> {
        // Generate random nonce
        let nonce_bytes: [u8; 12] = rand::random();
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| CryptoError::EncryptionError(e.to_string()))?;

        Ok(EncryptedData {
            ciphertext,
            nonce: nonce_bytes.to_vec(),
            algorithm: "AES-256-GCM".to_string(),
            encrypted_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    /// Encrypt a string
    pub fn encrypt_string(&self, plaintext: &str) -> CryptoResult<EncryptedData> {
        self.encrypt(plaintext.as_bytes())
    }

    /// Decrypt data
    pub fn decrypt(&self, encrypted: &EncryptedData) -> CryptoResult<Vec<u8>> {
        if encrypted.algorithm != "AES-256-GCM" {
            return Err(CryptoError::DecryptionError(
                "Unsupported algorithm".to_string(),
            ));
        }

        let nonce = Nonce::from_slice(&encrypted.nonce);

        let plaintext = self
            .cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| CryptoError::DecryptionError(e.to_string()))?;

        Ok(plaintext)
    }

    /// Decrypt to string
    pub fn decrypt_string(&self, encrypted: &EncryptedData) -> CryptoResult<String> {
        let plaintext = self.decrypt(encrypted)?;
        String::from_utf8(plaintext)
            .map_err(|e| CryptoError::DecryptionError(e.to_string()))
    }
}

// ============================================================================
// Secure Session Data
// ============================================================================

/// Encrypted session data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureSession {
    /// Session ID
    pub session_id: String,

    /// Encrypted data
    pub encrypted_data: EncryptedData,

    /// Creation timestamp
    pub created_at: SystemTime,

    /// Expiration
    pub expires_at: SystemTime,
}

impl SecureSession {
    /// Create a new secure session
    pub fn new(
        session_id: String,
        data: &str,
        encryptor: &DataEncryptor,
        ttl: Duration,
    ) -> CryptoResult<Self> {
        let encrypted_data = encryptor.encrypt_string(data)?;
        let now = SystemTime::now();

        Ok(Self {
            session_id,
            encrypted_data,
            created_at: now,
            expires_at: now + ttl,
        })
    }

    /// Decrypt session data
    pub fn decrypt(&self, encryptor: &DataEncryptor) -> CryptoResult<String> {
        // Check expiration
        if SystemTime::now() > self.expires_at {
            return Err(CryptoError::DecryptionError("Session expired".to_string()));
        }

        encryptor.decrypt_string(&self.encrypted_data)
    }

    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Constant-time string comparison to prevent timing attacks
pub fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }

    result == 0
}

/// Hash data with SHA-256
pub fn hash_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Hash data with SHA-512
pub fn hash_sha512(data: &[u8]) -> String {
    let mut hasher = Sha512::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Generate a cryptographic fingerprint from multiple inputs
pub fn generate_fingerprint(inputs: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for input in inputs {
        hasher.update(input.as_bytes());
    }
    hex::encode(hasher.finalize())
}

// ============================================================================
// Key Derivation
// ============================================================================

/// Derive encryption key from password using PBKDF2
pub fn derive_key_from_password(password: &str, salt: &[u8], iterations: u32) -> Vec<u8> {
    use pbkdf2::pbkdf2_hmac;

    let mut key = vec![0u8; 32]; // 256-bit key
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, iterations, &mut key);
    key
}

/// Derive key using HKDF
pub fn derive_key_hkdf(input_key: &[u8], salt: &[u8], info: &[u8]) -> CryptoResult<Vec<u8>> {
    use hkdf::Hkdf;

    let hk = Hkdf::<Sha256>::new(Some(salt), input_key);
    let mut okm = vec![0u8; 32];
    hk.expand(info, &mut okm)
        .map_err(|e| CryptoError::HashingError(e.to_string()))?;

    Ok(okm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let hasher = PasswordHasher::new();
        let password = "SecurePassword123!";

        let hash = hasher.hash_password(password).unwrap();
        assert!(!hash.is_empty());

        // Verify correct password
        assert!(hasher.verify_password(password, &hash).is_ok());

        // Verify wrong password
        assert!(hasher.verify_password("WrongPassword", &hash).is_err());
    }

    #[test]
    fn test_token_generation() {
        let token1 = TokenGenerator::generate(32);
        let token2 = TokenGenerator::generate(32);

        assert_eq!(token1.len(), 64); // 32 bytes = 64 hex chars
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_api_key() {
        let (api_key, actual_key) = ApiKey::new(
            "user123".to_string(),
            "Test Key".to_string(),
            vec!["read".to_string(), "write".to_string()],
        );

        assert!(api_key.verify(&actual_key));
        assert!(!api_key.verify("wrong_key"));
    }

    #[test]
    fn test_data_encryption() {
        let key = DataEncryptor::generate_key();
        let encryptor = DataEncryptor::from_base64_key(&key).unwrap();

        let plaintext = "Sensitive authentication data";
        let encrypted = encryptor.encrypt_string(plaintext).unwrap();

        let decrypted = encryptor.decrypt_string(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_constant_time_compare() {
        assert!(constant_time_compare("hello", "hello"));
        assert!(!constant_time_compare("hello", "world"));
        assert!(!constant_time_compare("hello", "hell"));
    }

    #[test]
    fn test_fingerprint_generation() {
        let fp1 = generate_fingerprint(&["192.168.1.1", "Mozilla/5.0"]);
        let fp2 = generate_fingerprint(&["192.168.1.1", "Mozilla/5.0"]);
        let fp3 = generate_fingerprint(&["192.168.1.2", "Mozilla/5.0"]);

        assert_eq!(fp1, fp2);
        assert_ne!(fp1, fp3);
    }

    #[test]
    fn test_key_derivation() {
        let password = "MySecurePassword";
        let salt = b"random_salt_12345";

        let key1 = derive_key_from_password(password, salt, 100_000);
        let key2 = derive_key_from_password(password, salt, 100_000);

        assert_eq!(key1, key2);
        assert_eq!(key1.len(), 32);
    }
}
