//! # License Validation
//!
//! This module provides comprehensive license validation including
//! signature verification, expiry checking, feature validation,
//! and concurrent user validation.

use chrono::{DateTime, Datelike, Utc};
use ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

use super::{
    activation::ActivationManager,
    key::{KeyValidator, LicenseKey},
    license::{License, LicenseError, LicenseFeature},
};

/// Validation errors
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("License validation failed: {0}")]
    ValidationFailed(String),

    #[error("License has expired on {0}")]
    Expired(DateTime<Utc>),

    #[error("License not activated")]
    NotActivated,

    #[error("Hardware mismatch")]
    HardwareMismatch,

    #[error("Concurrent user limit exceeded")]
    ConcurrentUserLimitExceeded,

    #[error("Feature not available: {0}")]
    FeatureNotAvailable(String),

    #[error("License has been revoked")]
    Revoked,

    #[error("Key validation error: {0}")]
    KeyError(#[from] super::key::KeyError),

    #[error("License error: {0}")]
    LicenseError(#[from] LicenseError),

    #[error("Signature verification failed")]
    SignatureVerificationFailed,

    #[error("Time synchronization issue detected")]
    TimeSyncIssue,

    #[error("License blacklisted")]
    Blacklisted,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the license is valid
    pub valid: bool,

    /// Validation timestamp
    pub validated_at: DateTime<Utc>,

    /// Warnings (non-fatal issues)
    pub warnings: Vec<String>,

    /// Errors (fatal issues)
    pub errors: Vec<String>,

    /// License information
    pub license_info: Option<LicenseInfo>,

    /// Time until expiry (if applicable)
    pub days_until_expiry: Option<i64>,

    /// Grace period active
    pub grace_period_active: bool,
}

impl ValidationResult {
    /// Create a successful validation result
    pub fn success(license: &License) -> Self {
        Self {
            valid: true,
            validated_at: Utc::now(),
            warnings: Vec::new(),
            errors: Vec::new(),
            license_info: Some(LicenseInfo::from_license(license)),
            days_until_expiry: license.days_until_expiry(),
            grace_period_active: license.is_in_grace_period(),
        }
    }

    /// Create a failed validation result
    pub fn failure(errors: Vec<String>) -> Self {
        Self {
            valid: false,
            validated_at: Utc::now(),
            warnings: Vec::new(),
            errors,
            license_info: None,
            days_until_expiry: None,
            grace_period_active: false,
        }
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Add an error
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.valid = false;
    }
}

/// License information for validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub license_id: Uuid,
    pub license_type: String,
    pub licensee_name: String,
    pub licensee_email: String,
    pub issued_at: DateTime<Utc>,
    pub expiry: Option<DateTime<Utc>>,
    pub activated: bool,
}

impl LicenseInfo {
    fn from_license(license: &License) -> Self {
        Self {
            license_id: license.id,
            license_type: format!("{:?}", license.license_type),
            licensee_name: license.licensee.name.clone(),
            licensee_email: license.licensee.email.clone(),
            issued_at: license.issued_at,
            expiry: license.expiry,
            activated: license.activated,
        }
    }
}

/// Validation policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationPolicy {
    /// Require activation
    pub require_activation: bool,

    /// Allow expired licenses (with warning)
    pub allow_expired: bool,

    /// Check hardware binding
    pub check_hardware_binding: bool,

    /// Offline validation grace period (days)
    pub offline_grace_period_days: i64,

    /// Maximum clock drift tolerance (seconds)
    pub max_clock_drift_seconds: i64,

    /// Enable revocation checking
    pub check_revocation: bool,

    /// Enable signature verification
    pub verify_signatures: bool,
}

impl Default for ValidationPolicy {
    fn default() -> Self {
        Self {
            require_activation: true,
            allow_expired: false,
            check_hardware_binding: true,
            offline_grace_period_days: 30,
            max_clock_drift_seconds: 300, // 5 minutes
            check_revocation: true,
            verify_signatures: true,
        }
    }
}

/// License validator
#[derive(Clone)]
pub struct LicenseValidator {
    /// Key validator
    key_validator: KeyValidator,

    /// Validation policy
    policy: ValidationPolicy,

    /// Blacklisted license IDs
    blacklist: Vec<Uuid>,

    /// Last validation cache
    validation_cache: HashMap<Uuid, ValidationResult>,
}

impl LicenseValidator {
    /// Create a new license validator
    pub fn new(verifying_key: VerifyingKey, policy: ValidationPolicy) -> Self {
        Self {
            key_validator: KeyValidator::new(verifying_key),
            policy,
            blacklist: Vec::new(),
            validation_cache: HashMap::new(),
        }
    }

    /// Create a validator from public key bytes
    pub fn from_bytes(
        public_key_bytes: &[u8; 32],
        policy: ValidationPolicy,
    ) -> Result<Self, ValidationError> {
        Ok(Self {
            key_validator: KeyValidator::from_bytes(public_key_bytes)?,
            policy,
            blacklist: Vec::new(),
            validation_cache: HashMap::new(),
        })
    }

    /// Add a license to the blacklist
    pub fn blacklist_license(&mut self, license_id: Uuid) {
        if !self.blacklist.contains(&license_id) {
            self.blacklist.push(license_id);
        }
    }

    /// Remove a license from the blacklist
    pub fn remove_from_blacklist(&mut self, license_id: Uuid) {
        self.blacklist.retain(|id| *id != license_id);
    }

    /// Validate a license key string
    pub fn validate_key(&self, key_string: &str) -> Result<LicenseKey, ValidationError> {
        let key = if self.policy.verify_signatures {
            self.key_validator.validate(key_string)?
        } else {
            LicenseKey::decode(key_string)?
        };
        Ok(key)
    }

    /// Comprehensive license validation
    pub fn validate_license(&mut self, license: &License) -> ValidationResult {
        let mut result = ValidationResult::success(license);

        // Check blacklist
        if self.blacklist.contains(&license.id) {
            result.add_error("License is blacklisted".to_string());
            return result;
        }

        // Check revocation
        if self.policy.check_revocation && license.revoked {
            result.add_error("License has been revoked".to_string());
            return result;
        }

        // Check activation
        if self.policy.require_activation && !license.activated {
            result.add_error("License is not activated".to_string());
            return result;
        }

        // Check expiry
        if license.is_expired() {
            if self.policy.allow_expired {
                result.add_warning("License has expired".to_string());
            } else {
                result.add_error("License has expired".to_string());
            }
        }

        // Check if near expiry
        if let Some(days) = license.days_until_expiry() {
            if days <= 30 && days > 0 {
                result.add_warning(format!("License expires in {} days", days));
            }
        }

        // Check grace period
        if license.is_in_grace_period() {
            result.add_warning("License is in grace period".to_string());
            result.grace_period_active = true;
        }

        // Verify signature if available
        if self.policy.verify_signatures {
            if let Some(signature) = &license.signature {
                // TODO: Implement signature verification
                // This would verify the license data against the signature
                if signature.is_empty() {
                    result.add_warning("License signature is empty".to_string());
                }
            } else {
                result.add_warning("License has no signature".to_string());
            }
        }

        // Cache result
        self.validation_cache.insert(license.id, result.clone());

        result
    }

    /// Validate license with activation manager
    pub fn validate_with_activation(
        &mut self,
        license: &License,
        activation_manager: &mut ActivationManager,
    ) -> Result<ValidationResult, ValidationError> {
        // First do basic validation
        let mut result = self.validate_license(license);

        if !result.valid {
            return Ok(result);
        }

        // Verify activation if policy requires
        if self.policy.check_hardware_binding && license.activated {
            match activation_manager.verify_activation(license) {
                Ok(()) => {}
                Err(e) => {
                    result.add_error(format!("Activation verification failed: {}", e));
                }
            }
        }

        Ok(result)
    }

    /// Validate feature access
    pub fn validate_feature(
        &self,
        license: &License,
        feature: LicenseFeature,
    ) -> Result<(), ValidationError> {
        // Basic license validity check
        license.is_valid()?;

        // Check if feature is available
        license.require_feature(feature)?;

        Ok(())
    }

    /// Validate concurrent users
    pub fn validate_concurrent_users(
        &self,
        license: &License,
        current_users: u32,
    ) -> Result<(), ValidationError> {
        license.check_user_limit(current_users)?;
        Ok(())
    }

    /// Batch validate multiple licenses
    pub fn batch_validate(&mut self, licenses: &[License]) -> Vec<ValidationResult> {
        licenses
            .iter()
            .map(|license| self.validate_license(license))
            .collect()
    }

    /// Get cached validation result
    pub fn get_cached_result(&self, license_id: Uuid) -> Option<&ValidationResult> {
        self.validation_cache.get(&license_id)
    }

    /// Clear validation cache
    pub fn clear_cache(&mut self) {
        self.validation_cache.clear();
    }

    /// Perform time synchronization check
    pub fn check_time_sync(&self) -> Result<(), ValidationError> {
        // In a real implementation, this would check against a trusted time source
        // For now, we just verify the system time seems reasonable
        let now = Utc::now();

        // Check if time is in a reasonable range (not before 2020, not too far in future)
        if now.year() < 2020 || now.year() > 2100 {
            return Err(ValidationError::TimeSyncIssue);
        }

        Ok(())
    }
}

/// Online validation service (stub for server-side validation)
pub struct OnlineValidationService {
    /// Server URL
    server_url: String,

    /// API key
    api_key: String,
}

impl OnlineValidationService {
    /// Create a new online validation service
    pub fn new(server_url: String, api_key: String) -> Self {
        Self {
            server_url,
            api_key,
        }
    }

    /// Validate license online
    pub async fn validate_online(
        &self,
        license_key: &str,
    ) -> Result<ValidationResult, ValidationError> {
        // TODO: Implement actual HTTP request to validation server
        // For now, return a stub response
        Err(ValidationError::ValidationFailed(
            "Online validation not implemented".to_string(),
        ))
    }

    /// Check if license is revoked
    pub async fn check_revocation(&self, license_id: Uuid) -> Result<bool, ValidationError> {
        // TODO: Implement revocation checking
        // This would query a revocation list from the server
        Ok(false)
    }
}

/// Validation report generator
pub struct ValidationReport {
    /// Validation results
    pub results: Vec<ValidationResult>,

    /// Generated at
    pub generated_at: DateTime<Utc>,

    /// Summary statistics
    pub summary: ValidationSummary,
}

impl ValidationReport {
    /// Create a new validation report
    pub fn new(results: Vec<ValidationResult>) -> Self {
        let summary = ValidationSummary::from_results(&results);

        Self {
            results,
            generated_at: Utc::now(),
            summary,
        }
    }

    /// Generate HTML report
    pub fn to_html(&self) -> String {
        // Simple HTML report generation
        let mut html = String::from("<html><head><title>License Validation Report</title></head><body>");
        html.push_str("<h1>License Validation Report</h1>");
        html.push_str(&format!("<p>Generated: {}</p>", self.generated_at));

        html.push_str("<h2>Summary</h2>");
        html.push_str(&format!("<p>Total: {}</p>", self.summary.total));
        html.push_str(&format!("<p>Valid: {}</p>", self.summary.valid));
        html.push_str(&format!("<p>Invalid: {}</p>", self.summary.invalid));
        html.push_str(&format!("<p>Warnings: {}</p>", self.summary.warnings));

        html.push_str("</body></html>");
        html
    }

    /// Generate JSON report
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl Serialize for ValidationReport {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ValidationReport", 3)?;
        state.serialize_field("generated_at", &self.generated_at)?;
        state.serialize_field("summary", &self.summary)?;
        state.serialize_field("results", &self.results)?;
        state.end()
    }
}

/// Validation summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    pub total: usize,
    pub valid: usize,
    pub invalid: usize,
    pub warnings: usize,
}

impl ValidationSummary {
    fn from_results(results: &[ValidationResult]) -> Self {
        let total = results.len();
        let valid = results.iter().filter(|r| r.valid).count();
        let invalid = total - valid;
        let warnings = results.iter().map(|r| r.warnings.len()).sum();

        Self {
            total,
            valid,
            invalid,
            warnings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enterprise::licensing::{
        key::KeyGenerator,
        license::{LicenseType, LicenseeInfo},
    };

    #[test]
    fn test_validation_result() {
        let licensee = LicenseeInfo {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            organization: None,
            country: None,
        };

        let license = License::new(
            "TEST-KEY-123".to_string(),
            LicenseType::Professional,
            licensee,
        );

        let result = ValidationResult::success(&license);
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_license_validator() {
        let generator = KeyGenerator::new();
        let public_key = *generator.public_key();
        let policy = ValidationPolicy::default();

        let mut validator = LicenseValidator::new(public_key, policy);

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

        // Should fail - not activated
        let result = validator.validate_license(&license);
        assert!(!result.valid);

        // Activate and try again
        license.activate("HW-123".to_string()).unwrap();
        let result = validator.validate_license(&license);
        assert!(result.valid);
    }

    #[test]
    fn test_feature_validation() {
        let generator = KeyGenerator::new();
        let public_key = *generator.public_key();
        let policy = ValidationPolicy::default();

        let validator = LicenseValidator::new(public_key, policy);

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

        license.activate("HW-123".to_string()).unwrap();

        // Should allow professional features
        assert!(validator
            .validate_feature(&license, LicenseFeature::Advanced3D)
            .is_ok());

        // Should deny enterprise features
        assert!(validator
            .validate_feature(&license, LicenseFeature::CloudSync)
            .is_err());
    }

    #[test]
    fn test_blacklist() {
        let generator = KeyGenerator::new();
        let public_key = *generator.public_key();
        let policy = ValidationPolicy::default();

        let mut validator = LicenseValidator::new(public_key, policy);

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

        license.activate("HW-123".to_string()).unwrap();

        // Should be valid
        let result = validator.validate_license(&license);
        assert!(result.valid);

        // Blacklist and try again
        validator.blacklist_license(license.id);
        let result = validator.validate_license(&license);
        assert!(!result.valid);
    }

    #[test]
    fn test_validation_report() {
        let licensee = LicenseeInfo {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            organization: None,
            country: None,
        };

        let license1 = License::new(
            "TEST-KEY-1".to_string(),
            LicenseType::Professional,
            licensee.clone(),
        );

        let license2 = License::new(
            "TEST-KEY-2".to_string(),
            LicenseType::Enterprise,
            licensee,
        );

        let result1 = ValidationResult::success(&license1);
        let result2 = ValidationResult::success(&license2);

        let report = ValidationReport::new(vec![result1, result2]);

        assert_eq!(report.summary.total, 2);
        assert_eq!(report.summary.valid, 2);
        assert_eq!(report.summary.invalid, 0);
    }
}
