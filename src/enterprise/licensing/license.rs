//! # License Types and Core Structures
//!
//! This module defines the core license types, features, and validation logic
//! for the CADDY enterprise licensing system.

use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur during license operations
#[derive(Debug, Error)]
pub enum LicenseError {
    #[error("License has expired on {0}")]
    Expired(DateTime<Utc>),

    #[error("Invalid license key format")]
    InvalidFormat,

    #[error("License signature verification failed")]
    InvalidSignature,

    #[error("Feature {0} not available in this license")]
    FeatureNotAvailable(String),

    #[error("License limit exceeded: {0}")]
    LimitExceeded(String),

    #[error("License not activated")]
    NotActivated,

    #[error("License has been revoked")]
    Revoked,

    #[error("Invalid license type")]
    InvalidType,

    #[error("Hardware mismatch")]
    HardwareMismatch,
}

/// License type indicating the tier of service
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LicenseType {
    /// Trial license with limited features and time
    Trial,

    /// Standard license for individual users
    Standard,

    /// Professional license with advanced features
    Professional,

    /// Enterprise license with all features
    Enterprise,

    /// Site license for unlimited users at a location
    Site,

    /// Educational license for academic institutions
    Educational,

    /// OEM license for embedding in other products
    OEM,
}

impl LicenseType {
    /// Get the default feature set for this license type
    pub fn default_features(&self) -> HashSet<LicenseFeature> {
        let mut features = HashSet::new();

        match self {
            LicenseType::Trial => {
                features.insert(LicenseFeature::BasicDrawing);
                features.insert(LicenseFeature::FileImportExport);
            }
            LicenseType::Standard => {
                features.insert(LicenseFeature::BasicDrawing);
                features.insert(LicenseFeature::FileImportExport);
                features.insert(LicenseFeature::LayerManagement);
                features.insert(LicenseFeature::BasicDimensions);
            }
            LicenseType::Professional => {
                features.insert(LicenseFeature::BasicDrawing);
                features.insert(LicenseFeature::FileImportExport);
                features.insert(LicenseFeature::LayerManagement);
                features.insert(LicenseFeature::BasicDimensions);
                features.insert(LicenseFeature::Advanced3D);
                features.insert(LicenseFeature::ParametricConstraints);
                features.insert(LicenseFeature::Scripting);
            }
            LicenseType::Enterprise | LicenseType::Site => {
                // All features
                features.insert(LicenseFeature::BasicDrawing);
                features.insert(LicenseFeature::FileImportExport);
                features.insert(LicenseFeature::LayerManagement);
                features.insert(LicenseFeature::BasicDimensions);
                features.insert(LicenseFeature::Advanced3D);
                features.insert(LicenseFeature::ParametricConstraints);
                features.insert(LicenseFeature::Scripting);
                features.insert(LicenseFeature::CloudSync);
                features.insert(LicenseFeature::Collaboration);
                features.insert(LicenseFeature::AdvancedRendering);
                features.insert(LicenseFeature::APIAccess);
                features.insert(LicenseFeature::CustomPlugins);
                features.insert(LicenseFeature::PrioritySupport);
            }
            LicenseType::Educational => {
                features.insert(LicenseFeature::BasicDrawing);
                features.insert(LicenseFeature::FileImportExport);
                features.insert(LicenseFeature::LayerManagement);
                features.insert(LicenseFeature::BasicDimensions);
                features.insert(LicenseFeature::Advanced3D);
                features.insert(LicenseFeature::ParametricConstraints);
            }
            LicenseType::OEM => {
                features.insert(LicenseFeature::BasicDrawing);
                features.insert(LicenseFeature::APIAccess);
                features.insert(LicenseFeature::CustomPlugins);
            }
        }

        features
    }

    /// Get the default user limit for this license type
    pub fn default_user_limit(&self) -> Option<u32> {
        match self {
            LicenseType::Trial => Some(1),
            LicenseType::Standard => Some(1),
            LicenseType::Professional => Some(1),
            LicenseType::Enterprise => Some(100),
            LicenseType::Site => None, // Unlimited
            LicenseType::Educational => Some(50),
            LicenseType::OEM => None, // Unlimited
        }
    }

    /// Get the default trial period for this license type
    pub fn default_trial_period(&self) -> Option<Duration> {
        match self {
            LicenseType::Trial => Some(Duration::days(30)),
            _ => None,
        }
    }
}

/// Individual features that can be enabled or disabled in a license
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LicenseFeature {
    // Basic Features
    BasicDrawing,
    FileImportExport,
    LayerManagement,
    BasicDimensions,

    // Advanced Features
    Advanced3D,
    ParametricConstraints,
    Scripting,

    // Enterprise Features
    CloudSync,
    Collaboration,
    AdvancedRendering,
    APIAccess,
    CustomPlugins,
    PrioritySupport,

    // Specialized Features
    BIMIntegration,
    CAMExport,
    FEAAnalysis,
    PDFGeneration,
    BatchProcessing,
}

impl LicenseFeature {
    /// Get a human-readable name for the feature
    pub fn name(&self) -> &'static str {
        match self {
            LicenseFeature::BasicDrawing => "Basic Drawing",
            LicenseFeature::FileImportExport => "File Import/Export",
            LicenseFeature::LayerManagement => "Layer Management",
            LicenseFeature::BasicDimensions => "Basic Dimensions",
            LicenseFeature::Advanced3D => "Advanced 3D Modeling",
            LicenseFeature::ParametricConstraints => "Parametric Constraints",
            LicenseFeature::Scripting => "Scripting",
            LicenseFeature::CloudSync => "Cloud Synchronization",
            LicenseFeature::Collaboration => "Real-time Collaboration",
            LicenseFeature::AdvancedRendering => "Advanced Rendering",
            LicenseFeature::APIAccess => "API Access",
            LicenseFeature::CustomPlugins => "Custom Plugins",
            LicenseFeature::PrioritySupport => "Priority Support",
            LicenseFeature::BIMIntegration => "BIM Integration",
            LicenseFeature::CAMExport => "CAM Export",
            LicenseFeature::FEAAnalysis => "FEA Analysis",
            LicenseFeature::PDFGeneration => "PDF Generation",
            LicenseFeature::BatchProcessing => "Batch Processing",
        }
    }
}

/// Usage limits that can be applied to a license
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseLimits {
    /// Maximum number of concurrent users (None = unlimited)
    pub max_users: Option<u32>,

    /// Maximum number of projects (None = unlimited)
    pub max_projects: Option<u32>,

    /// Maximum file size in MB (None = unlimited)
    pub max_file_size_mb: Option<u32>,

    /// Maximum cloud storage in GB (None = unlimited)
    pub max_cloud_storage_gb: Option<u32>,

    /// Maximum API calls per day (None = unlimited)
    pub max_api_calls_per_day: Option<u64>,

    /// Maximum number of activations allowed
    pub max_activations: u32,
}

impl Default for LicenseLimits {
    fn default() -> Self {
        Self {
            max_users: Some(1),
            max_projects: None,
            max_file_size_mb: None,
            max_cloud_storage_gb: Some(10),
            max_api_calls_per_day: Some(1000),
            max_activations: 2,
        }
    }
}

/// Main license structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    /// Unique license identifier
    pub id: Uuid,

    /// License key string
    pub key: String,

    /// Type of license
    pub license_type: LicenseType,

    /// Enabled features
    pub features: HashSet<LicenseFeature>,

    /// Usage limits
    pub limits: LicenseLimits,

    /// License expiry date (None = perpetual)
    pub expiry: Option<DateTime<Utc>>,

    /// Issue date
    pub issued_at: DateTime<Utc>,

    /// Licensee information
    pub licensee: LicenseeInfo,

    /// Activation status
    pub activated: bool,

    /// Activation date
    pub activated_at: Option<DateTime<Utc>>,

    /// Hardware fingerprint (for activation binding)
    pub hardware_id: Option<String>,

    /// Number of activations used
    pub activation_count: u32,

    /// Whether the license has been revoked
    pub revoked: bool,

    /// Revocation date
    pub revoked_at: Option<DateTime<Utc>>,

    /// Digital signature for verification
    pub signature: Option<Vec<u8>>,

    /// Metadata for additional information
    pub metadata: serde_json::Value,
}

/// Information about the licensee
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseeInfo {
    /// Company or individual name
    pub name: String,

    /// Email address
    pub email: String,

    /// Organization (optional)
    pub organization: Option<String>,

    /// Country code
    pub country: Option<String>,
}

impl License {
    /// Create a new license
    pub fn new(
        key: String,
        license_type: LicenseType,
        licensee: LicenseeInfo,
    ) -> Self {
        let features = license_type.default_features();
        let mut limits = LicenseLimits::default();
        limits.max_users = license_type.default_user_limit();

        let expiry = license_type.default_trial_period()
            .map(|period| Utc::now() + period);

        Self {
            id: Uuid::new_v4(),
            key,
            license_type,
            features,
            limits,
            expiry,
            issued_at: Utc::now(),
            licensee,
            activated: false,
            activated_at: None,
            hardware_id: None,
            activation_count: 0,
            revoked: false,
            revoked_at: None,
            signature: None,
            metadata: serde_json::json!({}),
        }
    }

    /// Check if the license has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expiry) = self.expiry {
            Utc::now() > expiry
        } else {
            false
        }
    }

    /// Check if the license is currently valid
    pub fn is_valid(&self) -> Result<(), LicenseError> {
        if self.revoked {
            return Err(LicenseError::Revoked);
        }

        if !self.activated {
            return Err(LicenseError::NotActivated);
        }

        if self.is_expired() {
            return Err(LicenseError::Expired(self.expiry.unwrap()));
        }

        Ok(())
    }

    /// Check if a specific feature is enabled
    pub fn has_feature(&self, feature: LicenseFeature) -> bool {
        self.features.contains(&feature)
    }

    /// Require a specific feature, returning an error if not available
    pub fn require_feature(&self, feature: LicenseFeature) -> Result<(), LicenseError> {
        if self.has_feature(feature) {
            Ok(())
        } else {
            Err(LicenseError::FeatureNotAvailable(feature.name().to_string()))
        }
    }

    /// Check if within user limit
    pub fn check_user_limit(&self, current_users: u32) -> Result<(), LicenseError> {
        if let Some(max_users) = self.limits.max_users {
            if current_users > max_users {
                return Err(LicenseError::LimitExceeded(
                    format!("Maximum {} concurrent users allowed", max_users)
                ));
            }
        }
        Ok(())
    }

    /// Check if within activation limit
    pub fn check_activation_limit(&self) -> Result<(), LicenseError> {
        if self.activation_count >= self.limits.max_activations {
            return Err(LicenseError::LimitExceeded(
                format!("Maximum {} activations allowed", self.limits.max_activations)
            ));
        }
        Ok(())
    }

    /// Get days until expiry (None if perpetual)
    pub fn days_until_expiry(&self) -> Option<i64> {
        self.expiry.map(|expiry| {
            let duration = expiry.signed_duration_since(Utc::now());
            duration.num_days()
        })
    }

    /// Check if license is in grace period (within 7 days of expiry)
    pub fn is_in_grace_period(&self) -> bool {
        if let Some(days) = self.days_until_expiry() {
            days >= 0 && days <= 7
        } else {
            false
        }
    }

    /// Activate the license with hardware binding
    pub fn activate(&mut self, hardware_id: String) -> Result<(), LicenseError> {
        self.check_activation_limit()?;

        if self.activated {
            // Check hardware match
            if let Some(existing_hw) = &self.hardware_id {
                if existing_hw != &hardware_id {
                    return Err(LicenseError::HardwareMismatch);
                }
            }
        }

        self.activated = true;
        self.activated_at = Some(Utc::now());
        self.hardware_id = Some(hardware_id);
        self.activation_count += 1;

        Ok(())
    }

    /// Deactivate the license
    pub fn deactivate(&mut self) {
        self.activated = false;
        self.hardware_id = None;
    }

    /// Revoke the license
    pub fn revoke(&mut self) {
        self.revoked = true;
        self.revoked_at = Some(Utc::now());
        self.activated = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_license_creation() {
        let licensee = LicenseeInfo {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            organization: Some("Test Corp".to_string()),
            country: Some("US".to_string()),
        };

        let license = License::new(
            "TEST-KEY-123".to_string(),
            LicenseType::Professional,
            licensee,
        );

        assert_eq!(license.license_type, LicenseType::Professional);
        assert!(!license.activated);
        assert!(!license.revoked);
    }

    #[test]
    fn test_feature_checking() {
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

        assert!(license.has_feature(LicenseFeature::Advanced3D));
        assert!(license.has_feature(LicenseFeature::ParametricConstraints));
        assert!(!license.has_feature(LicenseFeature::CloudSync));
    }

    #[test]
    fn test_trial_expiry() {
        let licensee = LicenseeInfo {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            organization: None,
            country: None,
        };

        let license = License::new(
            "TEST-KEY-123".to_string(),
            LicenseType::Trial,
            licensee,
        );

        assert!(license.expiry.is_some());
        assert!(!license.is_expired());
    }

    #[test]
    fn test_license_activation() {
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

        assert!(!license.activated);

        license.activate("HW-123".to_string()).unwrap();

        assert!(license.activated);
        assert_eq!(license.hardware_id, Some("HW-123".to_string()));
        assert_eq!(license.activation_count, 1);
    }

    #[test]
    fn test_license_revocation() {
        let licensee = LicenseeInfo {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            organization: None,
            country: None,
        };

        let mut license = License::new(
            "TEST-KEY-123".to_string(),
            LicenseType::Enterprise,
            licensee,
        );

        license.activate("HW-123".to_string()).unwrap();
        assert!(license.is_valid().is_ok());

        license.revoke();
        assert!(license.revoked);
        assert!(license.is_valid().is_err());
    }
}
