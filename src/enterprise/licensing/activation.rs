//! # License Activation System
//!
//! This module handles both online and offline license activation,
//! including hardware fingerprinting and activation limits.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

use super::license::{License, LicenseError};

/// Errors that can occur during activation
#[derive(Debug, Error)]
pub enum ActivationError {
    #[error("Activation limit exceeded")]
    LimitExceeded,

    #[error("License already activated on different hardware")]
    HardwareMismatch,

    #[error("Invalid activation code")]
    InvalidActivationCode,

    #[error("Online activation failed: {0}")]
    OnlineActivationFailed(String),

    #[error("Offline activation challenge generation failed")]
    ChallengeGenerationFailed,

    #[error("Invalid offline response")]
    InvalidOfflineResponse,

    #[error("Hardware fingerprint generation failed")]
    FingerprintGenerationFailed,

    #[error("License error: {0}")]
    LicenseError(#[from] LicenseError),
}

/// Activation method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivationMethod {
    /// Online activation via server
    Online,

    /// Offline activation with challenge/response
    Offline,

    /// Automatic activation (for site licenses)
    Automatic,
}

/// Activation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationRecord {
    /// Unique activation ID
    pub id: Uuid,

    /// License ID
    pub license_id: Uuid,

    /// Hardware fingerprint
    pub hardware_id: String,

    /// Activation method used
    pub method: ActivationMethod,

    /// Activation timestamp
    pub activated_at: DateTime<Utc>,

    /// Last verification timestamp
    pub last_verified_at: Option<DateTime<Utc>>,

    /// Device name (optional)
    pub device_name: Option<String>,

    /// IP address used for activation (optional)
    pub ip_address: Option<String>,

    /// Geographic location (optional)
    pub location: Option<String>,

    /// Metadata
    pub metadata: HashMap<String, String>,
}

impl ActivationRecord {
    /// Create a new activation record
    pub fn new(
        license_id: Uuid,
        hardware_id: String,
        method: ActivationMethod,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            license_id,
            hardware_id,
            method,
            activated_at: Utc::now(),
            last_verified_at: None,
            device_name: None,
            ip_address: None,
            location: None,
            metadata: HashMap::new(),
        }
    }

    /// Update last verification time
    pub fn verify(&mut self) {
        self.last_verified_at = Some(Utc::now());
    }
}

/// Hardware fingerprint components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareFingerprint {
    /// CPU identifier
    pub cpu_id: Option<String>,

    /// Motherboard serial
    pub motherboard_serial: Option<String>,

    /// MAC addresses
    pub mac_addresses: Vec<String>,

    /// Hard drive serial numbers
    pub disk_serials: Vec<String>,

    /// Operating system
    pub os_info: String,

    /// Machine name
    pub machine_name: Option<String>,
}

impl HardwareFingerprint {
    /// Create a new hardware fingerprint
    pub fn new() -> Result<Self, ActivationError> {
        Ok(Self {
            cpu_id: Self::get_cpu_id(),
            motherboard_serial: Self::get_motherboard_serial(),
            mac_addresses: Self::get_mac_addresses(),
            disk_serials: Self::get_disk_serials(),
            os_info: Self::get_os_info(),
            machine_name: Self::get_machine_name(),
        })
    }

    /// Generate a stable hardware ID hash
    pub fn generate_id(&self) -> String {
        let mut hasher = Sha256::new();

        if let Some(cpu_id) = &self.cpu_id {
            hasher.update(cpu_id.as_bytes());
        }

        if let Some(mb_serial) = &self.motherboard_serial {
            hasher.update(mb_serial.as_bytes());
        }

        // Use first MAC address for stability
        if let Some(mac) = self.mac_addresses.first() {
            hasher.update(mac.as_bytes());
        }

        // Use first disk serial for stability
        if let Some(disk) = self.disk_serials.first() {
            hasher.update(disk.as_bytes());
        }

        hasher.update(self.os_info.as_bytes());

        hex::encode(hasher.finalize())
    }

    /// Get CPU identifier (platform-specific)
    fn get_cpu_id() -> Option<String> {
        // TODO: Implement platform-specific CPU ID retrieval
        // For now, return None (would use cpuid on x86, /proc/cpuinfo on Linux, etc.)
        None
    }

    /// Get motherboard serial (platform-specific)
    fn get_motherboard_serial() -> Option<String> {
        // TODO: Implement platform-specific motherboard serial retrieval
        // Would use WMI on Windows, dmidecode on Linux, etc.
        None
    }

    /// Get MAC addresses
    fn get_mac_addresses() -> Vec<String> {
        // TODO: Implement MAC address retrieval
        // Would enumerate network interfaces
        Vec::new()
    }

    /// Get disk serial numbers
    fn get_disk_serials() -> Vec<String> {
        // TODO: Implement disk serial retrieval
        // Would use platform-specific APIs
        Vec::new()
    }

    /// Get OS information
    fn get_os_info() -> String {
        std::env::consts::OS.to_string()
    }

    /// Get machine name
    fn get_machine_name() -> Option<String> {
        // Use environment variable as fallback since hostname crate is not available
        std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .ok()
    }
}

impl Default for HardwareFingerprint {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            cpu_id: None,
            motherboard_serial: None,
            mac_addresses: Vec::new(),
            disk_serials: Vec::new(),
            os_info: "unknown".to_string(),
            machine_name: None,
        })
    }
}

/// Offline activation challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationChallenge {
    /// Challenge ID
    pub id: Uuid,

    /// License key
    pub license_key: String,

    /// Hardware ID
    pub hardware_id: String,

    /// Challenge code (to be sent to vendor)
    pub challenge_code: String,

    /// Created timestamp
    pub created_at: DateTime<Utc>,
}

impl ActivationChallenge {
    /// Generate a new activation challenge
    pub fn generate(license_key: String, hardware_id: String) -> Self {
        let id = Uuid::new_v4();
        let created_at = Utc::now();

        // Generate challenge code from license key and hardware ID
        let mut hasher = Sha256::new();
        hasher.update(license_key.as_bytes());
        hasher.update(hardware_id.as_bytes());
        hasher.update(id.as_bytes());
        hasher.update(created_at.to_rfc3339().as_bytes());

        let hash = hasher.finalize();
        let challenge_code = base32::encode(
            base32::Alphabet::RFC4648 { padding: false },
            &hash[..16],
        );

        Self {
            id,
            license_key,
            hardware_id,
            challenge_code,
            created_at,
        }
    }
}

/// Offline activation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationResponse {
    /// Response to challenge ID
    pub challenge_id: Uuid,

    /// Response code (from vendor)
    pub response_code: String,

    /// Generated timestamp
    pub generated_at: DateTime<Utc>,

    /// Signature
    pub signature: Vec<u8>,
}

impl ActivationResponse {
    /// Verify the activation response
    pub fn verify(&self, challenge: &ActivationChallenge) -> Result<(), ActivationError> {
        if self.challenge_id != challenge.id {
            return Err(ActivationError::InvalidOfflineResponse);
        }

        // TODO: Verify signature with vendor public key

        Ok(())
    }
}

/// Online activation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineActivationRequest {
    /// License key
    pub license_key: String,

    /// Hardware fingerprint
    pub hardware_id: String,

    /// Device information
    pub device_name: Option<String>,

    /// Product version
    pub product_version: String,

    /// Client IP (filled by server)
    pub client_ip: Option<String>,
}

/// Online activation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineActivationResponse {
    /// Success status
    pub success: bool,

    /// Activation record (if successful)
    pub activation: Option<ActivationRecord>,

    /// Error message (if failed)
    pub error: Option<String>,

    /// Server timestamp
    pub server_time: DateTime<Utc>,
}

/// Activation manager
pub struct ActivationManager {
    /// Active activations
    activations: HashMap<Uuid, ActivationRecord>,

    /// Activation server URL (for online activation)
    server_url: Option<String>,
}

impl ActivationManager {
    /// Create a new activation manager
    pub fn new() -> Self {
        Self {
            activations: HashMap::new(),
            server_url: None,
        }
    }

    /// Set the activation server URL
    pub fn set_server_url(&mut self, url: String) {
        self.server_url = Some(url);
    }

    /// Perform online activation
    pub async fn activate_online(
        &mut self,
        license: &mut License,
    ) -> Result<ActivationRecord, ActivationError> {
        // Generate hardware fingerprint
        let fingerprint = HardwareFingerprint::new()?;
        let hardware_id = fingerprint.generate_id();

        // Check activation limit
        license.check_activation_limit()?;

        // TODO: Make HTTP request to activation server
        // For now, simulate successful activation

        // Activate the license
        license.activate(hardware_id.clone())?;

        // Create activation record
        let record = ActivationRecord::new(
            license.id,
            hardware_id,
            ActivationMethod::Online,
        );

        self.activations.insert(license.id, record.clone());

        Ok(record)
    }

    /// Generate offline activation challenge
    pub fn generate_challenge(
        &self,
        license: &License,
    ) -> Result<ActivationChallenge, ActivationError> {
        let fingerprint = HardwareFingerprint::new()?;
        let hardware_id = fingerprint.generate_id();

        Ok(ActivationChallenge::generate(
            license.key.clone(),
            hardware_id,
        ))
    }

    /// Complete offline activation with response
    pub fn activate_offline(
        &mut self,
        license: &mut License,
        challenge: &ActivationChallenge,
        response: &ActivationResponse,
    ) -> Result<ActivationRecord, ActivationError> {
        // Verify response
        response.verify(challenge)?;

        // Check activation limit
        license.check_activation_limit()?;

        // Activate the license
        license.activate(challenge.hardware_id.clone())?;

        // Create activation record
        let record = ActivationRecord::new(
            license.id,
            challenge.hardware_id.clone(),
            ActivationMethod::Offline,
        );

        self.activations.insert(license.id, record.clone());

        Ok(record)
    }

    /// Deactivate a license
    pub fn deactivate(&mut self, license: &mut License) -> Result<(), ActivationError> {
        license.deactivate();
        self.activations.remove(&license.id);
        Ok(())
    }

    /// Verify current activation
    pub fn verify_activation(
        &mut self,
        license: &License,
    ) -> Result<(), ActivationError> {
        // Check if activated
        license.is_valid()?;

        // Get current hardware ID
        let fingerprint = HardwareFingerprint::new()?;
        let current_hw_id = fingerprint.generate_id();

        // Check hardware match
        if let Some(license_hw_id) = &license.hardware_id {
            if license_hw_id != &current_hw_id {
                return Err(ActivationError::HardwareMismatch);
            }
        }

        // Update last verification time
        if let Some(record) = self.activations.get_mut(&license.id) {
            record.verify();
        }

        Ok(())
    }

    /// Get activation record for a license
    pub fn get_activation(&self, license_id: Uuid) -> Option<&ActivationRecord> {
        self.activations.get(&license_id)
    }

    /// Get all activations
    pub fn get_all_activations(&self) -> Vec<&ActivationRecord> {
        self.activations.values().collect()
    }

    /// Transfer activation to new hardware
    pub async fn transfer_activation(
        &mut self,
        license: &mut License,
    ) -> Result<ActivationRecord, ActivationError> {
        // Deactivate current
        self.deactivate(license)?;

        // Activate on new hardware
        self.activate_online(license).await
    }
}

impl Default for ActivationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enterprise::licensing::license::{LicenseType, LicenseeInfo};

    #[test]
    fn test_hardware_fingerprint() {
        let fingerprint = HardwareFingerprint::new().unwrap();
        let hw_id = fingerprint.generate_id();

        assert!(!hw_id.is_empty());
        assert_eq!(hw_id.len(), 64); // SHA256 hex string
    }

    #[test]
    fn test_activation_challenge() {
        let challenge = ActivationChallenge::generate(
            "TEST-KEY-12345".to_string(),
            "hardware123".to_string(),
        );

        assert!(!challenge.challenge_code.is_empty());
        assert_eq!(challenge.license_key, "TEST-KEY-12345");
        assert_eq!(challenge.hardware_id, "hardware123");
    }

    #[test]
    fn test_activation_manager() {
        let mut manager = ActivationManager::new();

        let licensee = LicenseeInfo {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            organization: None,
            country: None,
        };

        let mut license = License::new(
            "TEST-KEY-123".to_string(),
            LicenseType::Professional,
            licensee,
        );

        // Generate challenge
        let challenge = manager.generate_challenge(&license).unwrap();
        assert!(!challenge.challenge_code.is_empty());

        // Simulate offline activation
        let response = ActivationResponse {
            challenge_id: challenge.id,
            response_code: "RESPONSE-CODE".to_string(),
            generated_at: Utc::now(),
            signature: vec![0; 64],
        };

        let record = manager
            .activate_offline(&mut license, &challenge, &response)
            .unwrap();

        assert!(license.activated);
        assert_eq!(record.license_id, license.id);
        assert_eq!(record.method, ActivationMethod::Offline);
    }

    #[test]
    fn test_activation_limit() {
        let mut manager = ActivationManager::new();

        let licensee = LicenseeInfo {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            organization: None,
            country: None,
        };

        let mut license = License::new(
            "TEST-KEY-123".to_string(),
            LicenseType::Standard,
            licensee,
        );

        // Set activation limit to 1
        license.limits.max_activations = 1;

        // First activation should succeed
        let hw_id = "hardware1".to_string();
        assert!(license.activate(hw_id).is_ok());

        // Second activation should fail (limit exceeded)
        let hw_id2 = "hardware2".to_string();
        assert!(license.activate(hw_id2).is_err());
    }
}
