//! # Enterprise License Management System
//!
//! This module provides a complete enterprise-grade license management system
//! for CADDY v0.1.5, including:
//!
//! - License key generation and validation with cryptographic signing
//! - Online and offline activation with hardware fingerprinting
//! - Entitlement management with seat limits and usage quotas
//! - Subscription management with renewal and billing integration
//! - Comprehensive license validation
//! - License enforcement with feature gating and violation handling
//!
//! ## Overview
//!
//! The licensing system is designed with security, flexibility, and ease of use in mind.
//! It supports multiple license types (Trial, Standard, Professional, Enterprise, Site, etc.),
//! various billing intervals, and comprehensive feature control.
//!
//! ## Architecture
//!
//! The system is organized into several key modules:
//!
//! - **license**: Core license types and structures
//! - **key**: License key generation, encoding, and validation
//! - **activation**: Online/offline activation with hardware binding
//! - **entitlement**: Feature entitlements, seat limits, and usage quotas
//! - **subscription**: Subscription lifecycle and billing management
//! - **validation**: Comprehensive license validation
//! - **enforcement**: Feature gating and license enforcement
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use caddy::enterprise::licensing::{
//!     license::{License, LicenseType, LicenseeInfo},
//!     key::KeyGenerator,
//!     activation::ActivationManager,
//!     enforcement::{EnforcementEngine, EnforcementPolicy},
//!     entitlement::EntitlementManager,
//!     validation::{LicenseValidator, ValidationPolicy},
//! };
//!
//! // Generate a license key
//! let mut generator = KeyGenerator::new();
//! let key = generator.generate(
//!     LicenseType::Professional,
//!     0, // perpetual
//!     0xFFFF, // all features
//! ).unwrap();
//!
//! let key_string = key.encode().unwrap();
//!
//! // Create a license
//! let licensee = LicenseeInfo {
//!     name: "John Doe".to_string(),
//!     email: "john@example.com".to_string(),
//!     organization: Some("Acme Corp".to_string()),
//!     country: Some("US".to_string()),
//! };
//!
//! let mut license = License::new(
//!     key_string,
//!     LicenseType::Professional,
//!     licensee,
//! );
//!
//! // Activate the license
//! let mut activation_manager = ActivationManager::new();
//! // For online activation:
//! // let activation = activation_manager.activate_online(&mut license).await.unwrap();
//!
//! // Set up enforcement
//! let validator = LicenseValidator::new(
//!     generator.verifying_key(),
//!     ValidationPolicy::default(),
//! );
//! let entitlement_manager = EntitlementManager::new(&license);
//! let enforcement_policy = EnforcementPolicy::default();
//!
//! let engine = EnforcementEngine::new(
//!     license,
//!     validator,
//!     entitlement_manager,
//!     enforcement_policy,
//! );
//!
//! // Check feature access
//! use caddy::enterprise::licensing::license::LicenseFeature;
//! engine.require_feature(LicenseFeature::Advanced3D, None).unwrap();
//! ```
//!
//! ## Security Considerations
//!
//! - License keys are cryptographically signed using Ed25519
//! - Hardware binding prevents license sharing
//! - Signature verification prevents tampering
//! - Revocation lists can be maintained server-side
//! - Usage tracking provides audit trails
//!
//! ## Production Deployment
//!
//! For production use, ensure:
//!
//! 1. **Secure Key Storage**: Store the signing keypair securely
//! 2. **Server Infrastructure**: Set up activation and validation servers
//! 3. **Monitoring**: Monitor license usage and violations
//! 4. **Backup**: Maintain backups of license database
//! 5. **Revocation**: Implement revocation checking for critical scenarios

// Core license types and structures
pub mod license;

// License key generation and validation
pub mod key;

// Activation system
pub mod activation;

// Entitlement management
pub mod entitlement;

// Subscription management
pub mod subscription;

// License validation
pub mod validation;

// License enforcement
pub mod enforcement;

// Re-export commonly used types
pub use license::{
    License, LicenseError, LicenseFeature, LicenseLimits, LicenseType, LicenseeInfo,
};

pub use key::{KeyError, KeyGenerator, KeyValidator, LicenseKey};

pub use activation::{
    ActivationChallenge, ActivationError, ActivationManager, ActivationMethod, ActivationRecord,
    ActivationResponse, HardwareFingerprint,
};

pub use entitlement::{
    EntitlementError, EntitlementManager, GracePeriod, GracePeriodStatus, SeatEntitlement,
    UsageQuota, UserSession,
};

pub use subscription::{
    BillingInterval, Invoice, InvoiceStatus, Subscription, SubscriptionError, SubscriptionManager,
    SubscriptionPlan, SubscriptionStatus,
};

pub use validation::{
    LicenseValidator, OnlineValidationService, ValidationError, ValidationPolicy,
    ValidationReport, ValidationResult, ValidationSummary,
};

pub use enforcement::{
    EnforcementAction, EnforcementEngine, EnforcementError, EnforcementPolicy, FeatureRule,
    LimitType, UsageEntry, UsageStatistics, UsageTracker, ViolationRecord, ViolationType,
};

/// Licensing system version
pub const LICENSING_VERSION: &str = "1.0.0";

/// Product identifier for CADDY
pub const PRODUCT_ID: u16 = 0xCADD;

/// Default public key for license validation
/// In production, this should be loaded from a secure configuration
pub const DEFAULT_PUBLIC_KEY: Option<&[u8; 32]> = None;

/// Initialize the licensing system
///
/// This is a convenience function to set up the basic licensing infrastructure.
/// Returns a tuple of (KeyGenerator, LicenseValidator, ActivationManager)
pub fn initialize_licensing_system() -> Result<
    (KeyGenerator, LicenseValidator, ActivationManager),
    Box<dyn std::error::Error>,
> {
    let generator = KeyGenerator::new();
    let verifying_key = generator.verifying_key();

    let validator = LicenseValidator::new(verifying_key, ValidationPolicy::default());
    let activation_manager = ActivationManager::new();

    Ok((generator, validator, activation_manager))
}

/// Create a complete licensing setup for a new license
///
/// This helper function creates and activates a new license with all necessary components.
pub fn create_license_setup(
    key_string: String,
    license_type: LicenseType,
    licensee: LicenseeInfo,
    public_key_bytes: &[u8; 32],
) -> Result<
    (
        License,
        LicenseValidator,
        EntitlementManager,
        EnforcementEngine,
    ),
    Box<dyn std::error::Error>,
> {
    // Create license
    let license = License::new(key_string, license_type, licensee);

    // Create validator
    let validator = LicenseValidator::from_bytes(public_key_bytes, ValidationPolicy::default())?;

    // Create entitlement manager
    let entitlement_manager = EntitlementManager::new(&license);

    // Create enforcement engine
    let enforcement_engine = EnforcementEngine::new(
        license.clone(),
        validator.clone(),
        entitlement_manager.clone(),
        EnforcementPolicy::default(),
    );

    Ok((license, validator, entitlement_manager, enforcement_engine))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_licensing_system_initialization() {
        let result = initialize_licensing_system();
        assert!(result.is_ok());
    }

    #[test]
    fn test_licensing_version() {
        assert!(!LICENSING_VERSION.is_empty());
    }

    #[test]
    fn test_product_id() {
        assert_eq!(PRODUCT_ID, 0xCADD);
    }

    #[test]
    fn test_end_to_end_license_flow() {
        // Generate a key
        let mut generator = KeyGenerator::new();
        let key = generator
            .generate(LicenseType::Professional, 0, 0xFFFF)
            .unwrap();
        let key_string = key.encode().unwrap();

        // Create licensee
        let licensee = LicenseeInfo {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            organization: Some("Test Corp".to_string()),
            country: Some("US".to_string()),
        };

        // Create license
        let mut license = License::new(key_string.clone(), LicenseType::Professional, licensee);

        // Activate
        license.activate("hardware-123".to_string()).unwrap();

        // Create validator
        let validator = LicenseValidator::new(generator.verifying_key(), ValidationPolicy::default());

        // Create entitlement manager
        let entitlement_manager = EntitlementManager::new(&license);

        // Create enforcement engine
        let engine = EnforcementEngine::new(
            license,
            validator,
            entitlement_manager,
            EnforcementPolicy::default(),
        );

        // Test feature access
        assert!(engine
            .check_feature_access(LicenseFeature::Advanced3D, None)
            .is_ok());
    }
}
