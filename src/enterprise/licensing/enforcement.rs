//! # License Enforcement
//!
//! This module handles license enforcement including feature gating,
//! usage tracking, soft vs hard limits, and violation handling.

use chrono::{DateTime, Duration, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

use super::{
    entitlement::EntitlementManager,
    license::{License, LicenseFeature},
    validation::{LicenseValidator, ValidationError},
};

/// Enforcement errors
#[derive(Debug, Error)]
pub enum EnforcementError {
    #[error("Feature access denied: {0}")]
    FeatureAccessDenied(String),

    #[error("Usage limit exceeded: {0}")]
    UsageLimitExceeded(String),

    #[error("License violation detected: {0}")]
    ViolationDetected(String),

    #[error("Enforcement policy not found")]
    PolicyNotFound,

    #[error("Validation error: {0}")]
    ValidationError(#[from] ValidationError),

    #[error("Action blocked by enforcement policy")]
    ActionBlocked,
}

/// Enforcement action to take on violation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnforcementAction {
    /// Log the violation but allow
    Log,

    /// Warn the user but allow
    Warn,

    /// Block the action
    Block,

    /// Disable the feature
    DisableFeature,

    /// Suspend the license
    SuspendLicense,
}

/// Limit type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LimitType {
    /// Soft limit - warns when exceeded
    Soft,

    /// Hard limit - blocks when exceeded
    Hard,
}

/// Usage tracking entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEntry {
    /// Feature used
    pub feature: LicenseFeature,

    /// User ID
    pub user_id: Option<String>,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Duration of usage (if applicable)
    pub duration: Option<Duration>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl UsageEntry {
    /// Create a new usage entry
    pub fn new(feature: LicenseFeature, user_id: Option<String>) -> Self {
        Self {
            feature,
            user_id,
            timestamp: Utc::now(),
            duration: None,
            metadata: HashMap::new(),
        }
    }
}

/// Usage tracker
#[derive(Debug, Clone)]
pub struct UsageTracker {
    /// License ID
    license_id: Uuid,

    /// Usage entries
    entries: Arc<RwLock<Vec<UsageEntry>>>,

    /// Feature usage counts
    feature_counts: Arc<RwLock<HashMap<LicenseFeature, u64>>>,

    /// Maximum entries to keep in memory
    max_entries: usize,
}

impl UsageTracker {
    /// Create a new usage tracker
    pub fn new(license_id: Uuid) -> Self {
        Self {
            license_id,
            entries: Arc::new(RwLock::new(Vec::new())),
            feature_counts: Arc::new(RwLock::new(HashMap::new())),
            max_entries: 10000,
        }
    }

    /// Track feature usage
    pub fn track_usage(&self, feature: LicenseFeature, user_id: Option<String>) {
        let entry = UsageEntry::new(feature, user_id);

        // Add to entries
        let mut entries = self.entries.write();
        entries.push(entry);

        // Trim if too many entries
        if entries.len() > self.max_entries {
            let remove_count = entries.len() - self.max_entries;
            entries.drain(0..remove_count);
        }

        // Update counts
        let mut counts = self.feature_counts.write();
        *counts.entry(feature).or_insert(0) += 1;
    }

    /// Get usage count for a feature
    pub fn get_usage_count(&self, feature: LicenseFeature) -> u64 {
        let counts = self.feature_counts.read();
        *counts.get(&feature).unwrap_or(&0)
    }

    /// Get total usage count
    pub fn get_total_usage(&self) -> u64 {
        let counts = self.feature_counts.read();
        counts.values().sum()
    }

    /// Get usage entries for a time period
    pub fn get_entries_since(&self, since: DateTime<Utc>) -> Vec<UsageEntry> {
        let entries = self.entries.read();
        entries
            .iter()
            .filter(|e| e.timestamp >= since)
            .cloned()
            .collect()
    }

    /// Clear usage data
    pub fn clear(&self) {
        self.entries.write().clear();
        self.feature_counts.write().clear();
    }

    /// Get usage statistics
    pub fn get_statistics(&self) -> UsageStatistics {
        let entries = self.entries.read();
        let counts = self.feature_counts.read();

        let total_entries = entries.len();
        let total_features = counts.len();
        let most_used_feature = counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(feature, count)| (*feature, *count));

        UsageStatistics {
            license_id: self.license_id,
            total_entries,
            total_features,
            most_used_feature,
            generated_at: Utc::now(),
        }
    }
}

/// Usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStatistics {
    pub license_id: Uuid,
    pub total_entries: usize,
    pub total_features: usize,
    pub most_used_feature: Option<(LicenseFeature, u64)>,
    pub generated_at: DateTime<Utc>,
}

/// Violation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationRecord {
    /// Violation ID
    pub id: Uuid,

    /// License ID
    pub license_id: Uuid,

    /// Violation type
    pub violation_type: ViolationType,

    /// Description
    pub description: String,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Action taken
    pub action: EnforcementAction,

    /// User involved (if applicable)
    pub user_id: Option<String>,

    /// Resolved flag
    pub resolved: bool,
}

impl ViolationRecord {
    /// Create a new violation record
    pub fn new(
        license_id: Uuid,
        violation_type: ViolationType,
        description: String,
        action: EnforcementAction,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            license_id,
            violation_type,
            description,
            timestamp: Utc::now(),
            action,
            user_id: None,
            resolved: false,
        }
    }
}

/// Types of violations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationType {
    /// Unauthorized feature access
    UnauthorizedFeatureAccess,

    /// User limit exceeded
    UserLimitExceeded,

    /// Expired license usage
    ExpiredLicenseUsage,

    /// Tampered license
    TamperedLicense,

    /// Hardware mismatch
    HardwareMismatch,

    /// Revoked license usage
    RevokedLicenseUsage,

    /// Quota exceeded
    QuotaExceeded,

    /// Time manipulation detected
    TimeManipulation,
}

/// Enforcement policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementPolicy {
    /// Policy name
    pub name: String,

    /// Feature enforcement rules
    pub feature_rules: HashMap<LicenseFeature, FeatureRule>,

    /// Global enforcement action
    pub default_action: EnforcementAction,

    /// Allow grace period
    pub allow_grace_period: bool,

    /// Grace period duration
    pub grace_period_days: i64,

    /// Violation threshold before escalation
    pub violation_threshold: u32,

    /// Track usage
    pub track_usage: bool,
}

impl EnforcementPolicy {
    /// Create a new enforcement policy
    pub fn new(name: String) -> Self {
        Self {
            name,
            feature_rules: HashMap::new(),
            default_action: EnforcementAction::Block,
            allow_grace_period: true,
            grace_period_days: 7,
            violation_threshold: 3,
            track_usage: true,
        }
    }

    /// Add a feature rule
    pub fn add_feature_rule(&mut self, feature: LicenseFeature, rule: FeatureRule) {
        self.feature_rules.insert(feature, rule);
    }

    /// Get enforcement action for a feature
    pub fn get_action_for_feature(&self, feature: LicenseFeature) -> EnforcementAction {
        self.feature_rules
            .get(&feature)
            .map(|rule| rule.action)
            .unwrap_or(self.default_action)
    }
}

impl Default for EnforcementPolicy {
    fn default() -> Self {
        Self::new("Default Policy".to_string())
    }
}

/// Feature enforcement rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureRule {
    /// Action to take
    pub action: EnforcementAction,

    /// Limit type
    pub limit_type: LimitType,

    /// Usage limit (if applicable)
    pub usage_limit: Option<u64>,

    /// Custom message to show user
    pub message: Option<String>,
}

impl FeatureRule {
    /// Create a new feature rule
    pub fn new(action: EnforcementAction, limit_type: LimitType) -> Self {
        Self {
            action,
            limit_type,
            usage_limit: None,
            message: None,
        }
    }

    /// Create a hard block rule
    pub fn hard_block() -> Self {
        Self::new(EnforcementAction::Block, LimitType::Hard)
    }

    /// Create a soft warning rule
    pub fn soft_warn() -> Self {
        Self::new(EnforcementAction::Warn, LimitType::Soft)
    }
}

/// License enforcement engine
pub struct EnforcementEngine {
    /// License
    license: Arc<RwLock<License>>,

    /// Validator
    validator: Arc<RwLock<LicenseValidator>>,

    /// Entitlement manager
    entitlement_manager: Arc<RwLock<EntitlementManager>>,

    /// Enforcement policy
    policy: EnforcementPolicy,

    /// Usage tracker
    usage_tracker: UsageTracker,

    /// Violation records
    violations: Arc<RwLock<Vec<ViolationRecord>>>,

    /// Enabled flag
    enforcement_enabled: bool,
}

impl EnforcementEngine {
    /// Create a new enforcement engine
    pub fn new(
        license: License,
        validator: LicenseValidator,
        entitlement_manager: EntitlementManager,
        policy: EnforcementPolicy,
    ) -> Self {
        let license_id = license.id;

        Self {
            license: Arc::new(RwLock::new(license)),
            validator: Arc::new(RwLock::new(validator)),
            entitlement_manager: Arc::new(RwLock::new(entitlement_manager)),
            policy,
            usage_tracker: UsageTracker::new(license_id),
            violations: Arc::new(RwLock::new(Vec::new())),
            enforcement_enabled: true,
        }
    }

    /// Enable or disable enforcement
    pub fn set_enforcement_enabled(&mut self, enabled: bool) {
        self.enforcement_enabled = enabled;
    }

    /// Check if a feature is allowed
    pub fn check_feature_access(
        &self,
        feature: LicenseFeature,
        user_id: Option<String>,
    ) -> Result<(), EnforcementError> {
        if !self.enforcement_enabled {
            return Ok(());
        }

        let license = self.license.read();
        let entitlement_manager = self.entitlement_manager.read();

        // Validate license is still valid
        if let Err(e) = license.is_valid() {
            self.record_violation(
                ViolationType::ExpiredLicenseUsage,
                format!("License validation failed: {}", e),
                EnforcementAction::Block,
            );
            return Err(EnforcementError::ViolationDetected(e.to_string()));
        }

        // Check feature entitlement
        if let Err(e) = entitlement_manager.check_feature(&license, feature) {
            let action = self.policy.get_action_for_feature(feature);

            self.record_violation(
                ViolationType::UnauthorizedFeatureAccess,
                format!("Unauthorized access to feature: {}", feature.name()),
                action,
            );

            match action {
                EnforcementAction::Block | EnforcementAction::DisableFeature => {
                    return Err(EnforcementError::FeatureAccessDenied(feature.name().to_string()));
                }
                EnforcementAction::Warn => {
                    // Log warning but allow
                    log::warn!("Feature access warning: {}", e);
                }
                _ => {}
            }
        }

        // Track usage
        if self.policy.track_usage {
            self.usage_tracker.track_usage(feature, user_id);
        }

        Ok(())
    }

    /// Require feature access (blocks if not allowed)
    pub fn require_feature(
        &self,
        feature: LicenseFeature,
        user_id: Option<String>,
    ) -> Result<(), EnforcementError> {
        self.check_feature_access(feature, user_id)
    }

    /// Check user limit
    pub fn check_user_limit(&self, current_users: u32) -> Result<(), EnforcementError> {
        if !self.enforcement_enabled {
            return Ok(());
        }

        let license = self.license.read();

        if let Err(e) = license.check_user_limit(current_users) {
            self.record_violation(
                ViolationType::UserLimitExceeded,
                format!("User limit exceeded: {}", e),
                EnforcementAction::Block,
            );

            return Err(EnforcementError::UsageLimitExceeded(e.to_string()));
        }

        Ok(())
    }

    /// Record a violation
    fn record_violation(
        &self,
        violation_type: ViolationType,
        description: String,
        action: EnforcementAction,
    ) {
        let license_id = self.license.read().id;
        let violation = ViolationRecord::new(license_id, violation_type, description, action);

        let mut violations = self.violations.write();
        violations.push(violation);

        // Check if threshold exceeded
        if violations.len() as u32 >= self.policy.violation_threshold {
            log::error!("Violation threshold exceeded for license {}", license_id);
        }
    }

    /// Get all violations
    pub fn get_violations(&self) -> Vec<ViolationRecord> {
        self.violations.read().clone()
    }

    /// Get unresolved violations
    pub fn get_unresolved_violations(&self) -> Vec<ViolationRecord> {
        self.violations
            .read()
            .iter()
            .filter(|v| !v.resolved)
            .cloned()
            .collect()
    }

    /// Mark violation as resolved
    pub fn resolve_violation(&self, violation_id: Uuid) -> Result<(), EnforcementError> {
        let mut violations = self.violations.write();

        if let Some(violation) = violations.iter_mut().find(|v| v.id == violation_id) {
            violation.resolved = true;
            Ok(())
        } else {
            Err(EnforcementError::PolicyNotFound)
        }
    }

    /// Clear all violations
    pub fn clear_violations(&self) {
        self.violations.write().clear();
    }

    /// Get usage tracker
    pub fn get_usage_tracker(&self) -> &UsageTracker {
        &self.usage_tracker
    }

    /// Get usage statistics
    pub fn get_usage_statistics(&self) -> UsageStatistics {
        self.usage_tracker.get_statistics()
    }

    /// Perform periodic validation
    pub fn periodic_validation(&mut self) -> Result<(), EnforcementError> {
        let license = self.license.read().clone();
        let mut validator = self.validator.write();

        let result = validator.validate_license(&license);

        if !result.valid {
            for error in &result.errors {
                self.record_violation(
                    ViolationType::ExpiredLicenseUsage,
                    error.clone(),
                    EnforcementAction::Block,
                );
            }

            return Err(EnforcementError::ViolationDetected(
                result.errors.join(", "),
            ));
        }

        Ok(())
    }
}

/// Feature gate macro helper
#[macro_export]
macro_rules! require_license_feature {
    ($engine:expr, $feature:expr) => {
        $engine.require_feature($feature, None)?
    };
    ($engine:expr, $feature:expr, $user_id:expr) => {
        $engine.require_feature($feature, Some($user_id.to_string()))?
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enterprise::licensing::{
        key::KeyGenerator,
        license::{LicenseType, LicenseeInfo},
        validation::ValidationPolicy,
    };

    #[test]
    fn test_usage_tracker() {
        let tracker = UsageTracker::new(Uuid::new_v4());

        tracker.track_usage(LicenseFeature::BasicDrawing, Some("user1".to_string()));
        tracker.track_usage(LicenseFeature::BasicDrawing, Some("user2".to_string()));
        tracker.track_usage(LicenseFeature::Advanced3D, Some("user1".to_string()));

        assert_eq!(tracker.get_usage_count(LicenseFeature::BasicDrawing), 2);
        assert_eq!(tracker.get_usage_count(LicenseFeature::Advanced3D), 1);
        assert_eq!(tracker.get_total_usage(), 3);
    }

    #[test]
    fn test_enforcement_policy() {
        let mut policy = EnforcementPolicy::new("Test Policy".to_string());

        policy.add_feature_rule(LicenseFeature::Advanced3D, FeatureRule::hard_block());
        policy.add_feature_rule(LicenseFeature::CloudSync, FeatureRule::soft_warn());

        assert_eq!(
            policy.get_action_for_feature(LicenseFeature::Advanced3D),
            EnforcementAction::Block
        );
        assert_eq!(
            policy.get_action_for_feature(LicenseFeature::CloudSync),
            EnforcementAction::Warn
        );
    }

    #[test]
    fn test_enforcement_engine() {
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

        let generator = KeyGenerator::new();
        let validator = LicenseValidator::new(
            generator.verifying_key(),
            ValidationPolicy::default(),
        );

        let entitlement_manager = EntitlementManager::new(&license);
        let policy = EnforcementPolicy::default();

        let engine = EnforcementEngine::new(license, validator, entitlement_manager, policy);

        // Should allow professional features
        assert!(engine
            .check_feature_access(LicenseFeature::Advanced3D, Some("user1".to_string()))
            .is_ok());

        // Should deny enterprise features
        assert!(engine
            .check_feature_access(LicenseFeature::CloudSync, Some("user1".to_string()))
            .is_err());

        // Check violations were recorded
        let violations = engine.get_unresolved_violations();
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_violation_recording() {
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

        license.activate("HW-123".to_string()).unwrap();

        let generator = KeyGenerator::new();
        let validator = LicenseValidator::new(
            generator.verifying_key(),
            ValidationPolicy::default(),
        );

        let entitlement_manager = EntitlementManager::new(&license);
        let policy = EnforcementPolicy::default();

        let engine = EnforcementEngine::new(license, validator, entitlement_manager, policy);

        // Attempt to access unauthorized feature multiple times
        for _ in 0..3 {
            let _ = engine.check_feature_access(LicenseFeature::CloudSync, None);
        }

        let violations = engine.get_violations();
        assert_eq!(violations.len(), 3);
        assert_eq!(violations[0].violation_type, ViolationType::UnauthorizedFeatureAccess);
    }
}
