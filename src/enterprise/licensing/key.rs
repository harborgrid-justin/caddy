//! # License Key Management
//!
//! This module handles license key generation, validation, encoding, and parsing.
//! Keys are cryptographically signed and include checksums for integrity verification.

use base32::Alphabet;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;
use thiserror::Error;

use super::license::LicenseType;

/// Errors that can occur during key operations
#[derive(Debug, Error)]
pub enum KeyError {
    #[error("Invalid key format")]
    InvalidFormat,

    #[error("Invalid checksum")]
    InvalidChecksum,

    #[error("Key decoding failed: {0}")]
    DecodingError(String),

    #[error("Key encoding failed: {0}")]
    EncodingError(String),

    #[error("Signature verification failed")]
    SignatureVerificationFailed,

    #[error("Key generation failed: {0}")]
    GenerationError(String),

    #[error("Invalid key version")]
    InvalidVersion,
}

/// License key version for forward compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyVersion {
    V1 = 1,
}

/// Structure representing a parsed license key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseKey {
    /// Key version
    pub version: KeyVersion,

    /// Product identifier
    pub product_id: u16,

    /// License type
    pub license_type: LicenseType,

    /// Unique key identifier
    pub key_id: u64,

    /// Expiry timestamp (Unix timestamp, 0 = perpetual)
    pub expiry_timestamp: u64,

    /// Feature flags (bit mask)
    pub feature_flags: u64,

    /// Checksum for integrity
    pub checksum: u32,

    /// Ed25519 signature
    pub signature: Vec<u8>,
}

impl LicenseKey {
    /// Product ID for CADDY
    pub const PRODUCT_ID: u16 = 0xCADD;

    /// Create a new license key
    pub fn new(
        license_type: LicenseType,
        key_id: u64,
        expiry_timestamp: u64,
        feature_flags: u64,
    ) -> Self {
        let mut key = Self {
            version: KeyVersion::V1,
            product_id: Self::PRODUCT_ID,
            license_type,
            key_id,
            expiry_timestamp,
            feature_flags,
            checksum: 0,
            signature: Vec::new(),
        };

        key.checksum = key.calculate_checksum();
        key
    }

    /// Calculate checksum for the key data
    fn calculate_checksum(&self) -> u32 {
        let mut hasher = Sha256::new();

        hasher.update(&(self.version as u8).to_le_bytes());
        hasher.update(&self.product_id.to_le_bytes());
        hasher.update(&(self.license_type as u8).to_le_bytes());
        hasher.update(&self.key_id.to_le_bytes());
        hasher.update(&self.expiry_timestamp.to_le_bytes());
        hasher.update(&self.feature_flags.to_le_bytes());

        let hash = hasher.finalize();
        u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]])
    }

    /// Verify the checksum
    pub fn verify_checksum(&self) -> Result<(), KeyError> {
        let expected = self.calculate_checksum();
        if self.checksum == expected {
            Ok(())
        } else {
            Err(KeyError::InvalidChecksum)
        }
    }

    /// Get the data to be signed (everything except signature)
    fn signing_data(&self) -> Vec<u8> {
        let mut data = Vec::new();

        data.extend_from_slice(&(self.version as u8).to_le_bytes());
        data.extend_from_slice(&self.product_id.to_le_bytes());
        data.extend_from_slice(&(self.license_type as u8).to_le_bytes());
        data.extend_from_slice(&self.key_id.to_le_bytes());
        data.extend_from_slice(&self.expiry_timestamp.to_le_bytes());
        data.extend_from_slice(&self.feature_flags.to_le_bytes());
        data.extend_from_slice(&self.checksum.to_le_bytes());

        data
    }

    /// Sign the key with a private key
    pub fn sign(&mut self, signing_key: &SigningKey) -> Result<(), KeyError> {
        let data = self.signing_data();
        let signature = signing_key.sign(&data);
        self.signature = signature.to_bytes().to_vec();
        Ok(())
    }

    /// Verify the signature with a public key
    pub fn verify_signature(&self, verifying_key: &VerifyingKey) -> Result<(), KeyError> {
        if self.signature.len() != 64 {
            return Err(KeyError::SignatureVerificationFailed);
        }

        let data = self.signing_data();
        let mut sig_bytes = [0u8; 64];
        sig_bytes.copy_from_slice(&self.signature);

        let signature = Signature::from_bytes(&sig_bytes);

        verifying_key
            .verify(&data, &signature)
            .map_err(|_| KeyError::SignatureVerificationFailed)
    }

    /// Encode the key as a formatted license key string
    pub fn encode(&self) -> Result<String, KeyError> {
        let mut data = Vec::new();

        // Encode all fields
        data.push(self.version as u8);
        data.extend_from_slice(&self.product_id.to_le_bytes());
        data.push(self.license_type as u8);
        data.extend_from_slice(&self.key_id.to_le_bytes());
        data.extend_from_slice(&self.expiry_timestamp.to_le_bytes());
        data.extend_from_slice(&self.feature_flags.to_le_bytes());
        data.extend_from_slice(&self.checksum.to_le_bytes());
        data.extend_from_slice(&self.signature);

        // Encode as base32 for better human readability
        let encoded = base32::encode(Alphabet::RFC4648 { padding: false }, &data);

        // Format as XXXXX-XXXXX-XXXXX-XXXXX-XXXXX
        Ok(Self::format_key(&encoded))
    }

    /// Format a key string with dashes
    fn format_key(key: &str) -> String {
        let mut formatted = String::new();
        for (i, chunk) in key.chars().collect::<Vec<_>>().chunks(5).enumerate() {
            if i > 0 {
                formatted.push('-');
            }
            formatted.extend(chunk);
        }
        formatted
    }

    /// Remove formatting from a key string
    fn unformat_key(key: &str) -> String {
        key.chars().filter(|c| *c != '-' && !c.is_whitespace()).collect()
    }

    /// Decode a license key string
    pub fn decode(key_string: &str) -> Result<Self, KeyError> {
        let unformatted = Self::unformat_key(key_string);

        // Decode from base32
        let data = base32::decode(Alphabet::RFC4648 { padding: false }, &unformatted)
            .ok_or_else(|| KeyError::DecodingError("Base32 decoding failed".to_string()))?;

        // Minimum size check (version + product_id + license_type + key_id + expiry + features + checksum + signature)
        // 1 + 2 + 1 + 8 + 8 + 8 + 4 + 64 = 96 bytes
        if data.len() < 96 {
            return Err(KeyError::InvalidFormat);
        }

        let mut offset = 0;

        // Parse version
        let version = match data[offset] {
            1 => KeyVersion::V1,
            _ => return Err(KeyError::InvalidVersion),
        };
        offset += 1;

        // Parse product ID
        let product_id = u16::from_le_bytes([data[offset], data[offset + 1]]);
        offset += 2;

        if product_id != Self::PRODUCT_ID {
            return Err(KeyError::InvalidFormat);
        }

        // Parse license type
        let license_type = match data[offset] {
            0 => LicenseType::Trial,
            1 => LicenseType::Standard,
            2 => LicenseType::Professional,
            3 => LicenseType::Enterprise,
            4 => LicenseType::Site,
            5 => LicenseType::Educational,
            6 => LicenseType::OEM,
            _ => return Err(KeyError::InvalidFormat),
        };
        offset += 1;

        // Parse key ID
        let key_id = u64::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]);
        offset += 8;

        // Parse expiry timestamp
        let expiry_timestamp = u64::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]);
        offset += 8;

        // Parse feature flags
        let feature_flags = u64::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]);
        offset += 8;

        // Parse checksum
        let checksum = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;

        // Parse signature (remaining bytes)
        let signature = data[offset..].to_vec();

        let key = Self {
            version,
            product_id,
            license_type,
            key_id,
            expiry_timestamp,
            feature_flags,
            checksum,
            signature,
        };

        // Verify checksum
        key.verify_checksum()?;

        Ok(key)
    }
}

impl fmt::Display for LicenseKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.encode() {
            Ok(key) => write!(f, "{}", key),
            Err(_) => write!(f, "<invalid key>"),
        }
    }
}

/// License key generator
pub struct KeyGenerator {
    signing_key: SigningKey,
    next_key_id: u64,
}

impl KeyGenerator {
    /// Create a new key generator with a random keypair
    pub fn new() -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);

        Self {
            signing_key,
            next_key_id: 1,
        }
    }

    /// Create a key generator from existing keypair bytes
    pub fn from_keypair_bytes(secret_bytes: &[u8; 32]) -> Result<Self, KeyError> {
        let signing_key = SigningKey::from_bytes(secret_bytes);

        Ok(Self {
            signing_key,
            next_key_id: 1,
        })
    }

    /// Get the public key for verification
    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// Get the public key bytes
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.signing_key.verifying_key().to_bytes()
    }

    /// Get the secret key bytes (for backup/storage)
    pub fn secret_key_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }

    /// Generate a new license key
    pub fn generate(
        &mut self,
        license_type: LicenseType,
        expiry_timestamp: u64,
        feature_flags: u64,
    ) -> Result<LicenseKey, KeyError> {
        let key_id = self.next_key_id;
        self.next_key_id += 1;

        let mut key = LicenseKey::new(license_type, key_id, expiry_timestamp, feature_flags);
        key.sign(&self.signing_key)?;

        Ok(key)
    }
}

impl Default for KeyGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Key validator for verifying license keys
#[derive(Clone)]
pub struct KeyValidator {
    verifying_key: VerifyingKey,
}

impl KeyValidator {
    /// Create a new validator with a public key
    pub fn new(verifying_key: VerifyingKey) -> Self {
        Self { verifying_key }
    }

    /// Create a validator from public key bytes
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, KeyError> {
        let verifying_key = VerifyingKey::from_bytes(bytes)
            .map_err(|e| KeyError::GenerationError(e.to_string()))?;

        Ok(Self { verifying_key })
    }

    /// Validate a license key string
    pub fn validate(&self, key_string: &str) -> Result<LicenseKey, KeyError> {
        let key = LicenseKey::decode(key_string)?;
        key.verify_signature(&self.verifying_key)?;
        Ok(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation_and_validation() {
        let mut generator = KeyGenerator::new();
        let public_key = *generator.public_key();

        let key = generator
            .generate(LicenseType::Professional, 0, 0xFFFF)
            .unwrap();

        let encoded = key.encode().unwrap();
        println!("Generated key: {}", encoded);

        let validator = KeyValidator::new(public_key);
        let decoded = validator.validate(&encoded).unwrap();

        assert_eq!(decoded.license_type, LicenseType::Professional);
        assert_eq!(decoded.key_id, 1);
        assert_eq!(decoded.feature_flags, 0xFFFF);
    }

    #[test]
    fn test_key_checksum() {
        let key = LicenseKey::new(LicenseType::Enterprise, 12345, 0, 0);
        assert!(key.verify_checksum().is_ok());

        let mut bad_key = key.clone();
        bad_key.checksum = 0;
        assert!(bad_key.verify_checksum().is_err());
    }

    #[test]
    fn test_key_formatting() {
        let mut generator = KeyGenerator::new();
        let key = generator
            .generate(LicenseType::Standard, 1234567890, 0)
            .unwrap();

        let encoded = key.encode().unwrap();
        assert!(encoded.contains('-'));

        let parts: Vec<&str> = encoded.split('-').collect();
        assert!(parts.len() > 1);
        for part in parts {
            assert!(part.len() <= 5);
        }
    }

    #[test]
    fn test_invalid_signature() {
        let mut generator = KeyGenerator::new();
        let key = generator
            .generate(LicenseType::Professional, 0, 0)
            .unwrap();

        // Create a different public key
        let other_generator = KeyGenerator::new();
        let validator = KeyValidator::new(*other_generator.public_key());

        let encoded = key.encode().unwrap();
        assert!(validator.validate(&encoded).is_err());
    }

    #[test]
    fn test_key_roundtrip() {
        let mut generator = KeyGenerator::new();
        let original = generator
            .generate(LicenseType::Enterprise, 9999999999, 0x123456789ABCDEF0)
            .unwrap();

        let encoded = original.encode().unwrap();
        let decoded = LicenseKey::decode(&encoded).unwrap();

        assert_eq!(decoded.version, original.version);
        assert_eq!(decoded.product_id, original.product_id);
        assert_eq!(decoded.license_type, original.license_type);
        assert_eq!(decoded.key_id, original.key_id);
        assert_eq!(decoded.expiry_timestamp, original.expiry_timestamp);
        assert_eq!(decoded.feature_flags, original.feature_flags);
        assert_eq!(decoded.checksum, original.checksum);
        assert_eq!(decoded.signature, original.signature);
    }
}
