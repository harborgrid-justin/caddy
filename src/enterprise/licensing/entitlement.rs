//! # Entitlement Management
//!
//! This module manages feature entitlements, seat/user limits,
//! usage quotas, and grace periods for licenses.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

use super::license::{License, LicenseFeature, LicenseError};

/// Errors that can occur with entitlements
#[derive(Debug, Error)]
pub enum EntitlementError {
    #[error("Feature not entitled: {0}")]
    FeatureNotEntitled(String),

    #[error("Seat limit exceeded: {current}/{max}")]
    SeatLimitExceeded { current: u32, max: u32 },

    #[error("Quota exceeded for {0}: {1}/{2}")]
    QuotaExceeded(String, u64, u64),

    #[error("Grace period has expired")]
    GracePeriodExpired,

    #[error("Entitlement not found: {0}")]
    NotFound(String),

    #[error("License error: {0}")]
    LicenseError(#[from] LicenseError),
}

/// Grace period status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GracePeriodStatus {
    /// No grace period active
    NotActive,

    /// Grace period is active
    Active,

    /// Grace period has expired
    Expired,
}

/// Grace period configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GracePeriod {
    /// Whether grace period is enabled
    pub enabled: bool,

    /// Duration of grace period
    pub duration: Duration,

    /// Start of grace period
    pub started_at: Option<DateTime<Utc>>,

    /// Features available during grace period
    pub allowed_features: Vec<LicenseFeature>,

    /// Reduced functionality during grace period
    pub reduced_functionality: bool,
}

impl GracePeriod {
    /// Create a new grace period configuration
    pub fn new(duration_days: i64) -> Self {
        Self {
            enabled: true,
            duration: Duration::days(duration_days),
            started_at: None,
            allowed_features: vec![
                LicenseFeature::BasicDrawing,
                LicenseFeature::FileImportExport,
            ],
            reduced_functionality: true,
        }
    }

    /// Start the grace period
    pub fn start(&mut self) {
        if self.started_at.is_none() {
            self.started_at = Some(Utc::now());
        }
    }

    /// Get the status of the grace period
    pub fn status(&self) -> GracePeriodStatus {
        if !self.enabled {
            return GracePeriodStatus::NotActive;
        }

        match self.started_at {
            None => GracePeriodStatus::NotActive,
            Some(start) => {
                let elapsed = Utc::now().signed_duration_since(start);
                if elapsed > self.duration {
                    GracePeriodStatus::Expired
                } else {
                    GracePeriodStatus::Active
                }
            }
        }
    }

    /// Get days remaining in grace period
    pub fn days_remaining(&self) -> Option<i64> {
        if let Some(start) = self.started_at {
            let end = start + self.duration;
            let remaining = end.signed_duration_since(Utc::now());
            if remaining.num_days() >= 0 {
                Some(remaining.num_days())
            } else {
                Some(0)
            }
        } else {
            None
        }
    }

    /// Check if a feature is allowed during grace period
    pub fn is_feature_allowed(&self, feature: LicenseFeature) -> bool {
        self.allowed_features.contains(&feature)
    }
}

impl Default for GracePeriod {
    fn default() -> Self {
        Self::new(7) // 7 day grace period by default
    }
}

/// Seat/user entitlement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeatEntitlement {
    /// Maximum number of seats
    pub max_seats: Option<u32>,

    /// Currently used seats
    pub used_seats: u32,

    /// Active user sessions
    pub active_users: HashMap<String, UserSession>,

    /// Named users (for named user licenses)
    pub named_users: Option<Vec<String>>,
}

impl SeatEntitlement {
    /// Create a new seat entitlement
    pub fn new(max_seats: Option<u32>) -> Self {
        Self {
            max_seats,
            used_seats: 0,
            active_users: HashMap::new(),
            named_users: None,
        }
    }

    /// Check if a seat is available
    pub fn has_available_seat(&self) -> bool {
        match self.max_seats {
            None => true, // Unlimited
            Some(max) => self.used_seats < max,
        }
    }

    /// Acquire a seat for a user
    pub fn acquire_seat(&mut self, user_id: String) -> Result<(), EntitlementError> {
        // Check if user already has a session
        if self.active_users.contains_key(&user_id) {
            return Ok(());
        }

        // Check seat availability
        if !self.has_available_seat() {
            return Err(EntitlementError::SeatLimitExceeded {
                current: self.used_seats,
                max: self.max_seats.unwrap(),
            });
        }

        // Check named user list if applicable
        if let Some(named_users) = &self.named_users {
            if !named_users.contains(&user_id) {
                return Err(EntitlementError::FeatureNotEntitled(
                    "User not in named user list".to_string(),
                ));
            }
        }

        // Create user session
        let session = UserSession::new(user_id.clone());
        self.active_users.insert(user_id, session);
        self.used_seats += 1;

        Ok(())
    }

    /// Release a seat
    pub fn release_seat(&mut self, user_id: &str) {
        if self.active_users.remove(user_id).is_some() {
            self.used_seats = self.used_seats.saturating_sub(1);
        }
    }

    /// Get seat utilization percentage
    pub fn utilization_percent(&self) -> Option<f32> {
        self.max_seats.map(|max| {
            if max == 0 {
                100.0
            } else {
                (self.used_seats as f32 / max as f32) * 100.0
            }
        })
    }
}

/// User session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    /// User identifier
    pub user_id: String,

    /// Session start time
    pub started_at: DateTime<Utc>,

    /// Last activity time
    pub last_activity: DateTime<Utc>,

    /// Session metadata
    pub metadata: HashMap<String, String>,
}

impl UserSession {
    /// Create a new user session
    pub fn new(user_id: String) -> Self {
        Self {
            user_id,
            started_at: Utc::now(),
            last_activity: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Update last activity time
    pub fn update_activity(&mut self) {
        self.last_activity = Utc::now();
    }

    /// Check if session is idle
    pub fn is_idle(&self, idle_threshold_minutes: i64) -> bool {
        let elapsed = Utc::now().signed_duration_since(self.last_activity);
        elapsed.num_minutes() > idle_threshold_minutes
    }
}

/// Usage quota tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageQuota {
    /// Quota name
    pub name: String,

    /// Maximum allowed usage (None = unlimited)
    pub max_usage: Option<u64>,

    /// Current usage
    pub current_usage: u64,

    /// Reset period (None = never resets)
    pub reset_period: Option<Duration>,

    /// Last reset time
    pub last_reset: DateTime<Utc>,

    /// Usage history
    pub history: Vec<UsageRecord>,
}

impl UsageQuota {
    /// Create a new usage quota
    pub fn new(name: String, max_usage: Option<u64>, reset_period: Option<Duration>) -> Self {
        Self {
            name,
            max_usage,
            current_usage: 0,
            reset_period,
            last_reset: Utc::now(),
            history: Vec::new(),
        }
    }

    /// Check if quota is available
    pub fn is_available(&mut self) -> bool {
        self.check_reset();

        match self.max_usage {
            None => true,
            Some(max) => self.current_usage < max,
        }
    }

    /// Consume quota
    pub fn consume(&mut self, amount: u64) -> Result<(), EntitlementError> {
        self.check_reset();

        if let Some(max) = self.max_usage {
            if self.current_usage + amount > max {
                return Err(EntitlementError::QuotaExceeded(
                    self.name.clone(),
                    self.current_usage + amount,
                    max,
                ));
            }
        }

        self.current_usage += amount;

        // Record usage
        self.history.push(UsageRecord {
            timestamp: Utc::now(),
            amount,
            total: self.current_usage,
        });

        Ok(())
    }

    /// Check if quota needs to be reset
    fn check_reset(&mut self) {
        if let Some(period) = self.reset_period {
            let elapsed = Utc::now().signed_duration_since(self.last_reset);
            if elapsed >= period {
                self.current_usage = 0;
                self.last_reset = Utc::now();
            }
        }
    }

    /// Get remaining quota
    pub fn remaining(&self) -> Option<u64> {
        self.max_usage.map(|max| max.saturating_sub(self.current_usage))
    }

    /// Get usage percentage
    pub fn usage_percent(&self) -> Option<f32> {
        self.max_usage.map(|max| {
            if max == 0 {
                100.0
            } else {
                (self.current_usage as f32 / max as f32) * 100.0
            }
        })
    }
}

/// Usage record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Amount used
    pub amount: u64,

    /// Total usage at time of record
    pub total: u64,
}

/// Entitlement manager
#[derive(Clone)]
pub struct EntitlementManager {
    /// License reference
    license_id: Uuid,

    /// Seat entitlement
    pub seats: SeatEntitlement,

    /// Grace period
    pub grace_period: GracePeriod,

    /// Usage quotas
    pub quotas: HashMap<String, UsageQuota>,

    /// Feature overrides (can grant or deny specific features)
    pub feature_overrides: HashMap<LicenseFeature, bool>,
}

impl EntitlementManager {
    /// Create a new entitlement manager for a license
    pub fn new(license: &License) -> Self {
        let seats = SeatEntitlement::new(license.limits.max_users);

        let mut quotas = HashMap::new();

        // Add API call quota if limited
        if let Some(max_api_calls) = license.limits.max_api_calls_per_day {
            quotas.insert(
                "api_calls".to_string(),
                UsageQuota::new(
                    "API Calls".to_string(),
                    Some(max_api_calls),
                    Some(Duration::days(1)),
                ),
            );
        }

        // Add cloud storage quota if limited
        if let Some(max_storage_gb) = license.limits.max_cloud_storage_gb {
            quotas.insert(
                "cloud_storage".to_string(),
                UsageQuota::new(
                    "Cloud Storage (GB)".to_string(),
                    Some(max_storage_gb as u64),
                    None, // Doesn't reset
                ),
            );
        }

        Self {
            license_id: license.id,
            seats,
            grace_period: GracePeriod::default(),
            quotas,
            feature_overrides: HashMap::new(),
        }
    }

    /// Check if a feature is entitled
    pub fn check_feature(
        &self,
        license: &License,
        feature: LicenseFeature,
    ) -> Result<(), EntitlementError> {
        // Check feature overrides first
        if let Some(&override_value) = self.feature_overrides.get(&feature) {
            if !override_value {
                return Err(EntitlementError::FeatureNotEntitled(feature.name().to_string()));
            }
            return Ok(());
        }

        // Check grace period
        match self.grace_period.status() {
            GracePeriodStatus::Active => {
                if !self.grace_period.is_feature_allowed(feature) {
                    return Err(EntitlementError::FeatureNotEntitled(
                        format!("{} not available during grace period", feature.name()),
                    ));
                }
            }
            GracePeriodStatus::Expired => {
                return Err(EntitlementError::GracePeriodExpired);
            }
            GracePeriodStatus::NotActive => {
                // Check normal license features
                license.require_feature(feature)?;
            }
        }

        Ok(())
    }

    /// Acquire a seat for a user
    pub fn acquire_seat(&mut self, user_id: String) -> Result<(), EntitlementError> {
        self.seats.acquire_seat(user_id)
    }

    /// Release a seat
    pub fn release_seat(&mut self, user_id: &str) {
        self.seats.release_seat(user_id);
    }

    /// Consume usage quota
    pub fn consume_quota(&mut self, quota_name: &str, amount: u64) -> Result<(), EntitlementError> {
        if let Some(quota) = self.quotas.get_mut(quota_name) {
            quota.consume(amount)
        } else {
            // No quota limit
            Ok(())
        }
    }

    /// Get usage quota
    pub fn get_quota(&mut self, quota_name: &str) -> Option<&mut UsageQuota> {
        self.quotas.get_mut(quota_name)
    }

    /// Override a feature entitlement
    pub fn override_feature(&mut self, feature: LicenseFeature, enabled: bool) {
        self.feature_overrides.insert(feature, enabled);
    }

    /// Clean up idle sessions
    pub fn cleanup_idle_sessions(&mut self, idle_threshold_minutes: i64) {
        let idle_users: Vec<String> = self
            .seats
            .active_users
            .values()
            .filter(|session| session.is_idle(idle_threshold_minutes))
            .map(|session| session.user_id.clone())
            .collect();

        for user_id in idle_users {
            self.release_seat(&user_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enterprise::licensing::license::{LicenseType, LicenseeInfo};

    #[test]
    fn test_seat_entitlement() {
        let mut seats = SeatEntitlement::new(Some(2));

        assert!(seats.acquire_seat("user1".to_string()).is_ok());
        assert_eq!(seats.used_seats, 1);

        assert!(seats.acquire_seat("user2".to_string()).is_ok());
        assert_eq!(seats.used_seats, 2);

        // Should fail - no more seats
        assert!(seats.acquire_seat("user3".to_string()).is_err());

        // Release a seat
        seats.release_seat("user1");
        assert_eq!(seats.used_seats, 1);

        // Should succeed now
        assert!(seats.acquire_seat("user3".to_string()).is_ok());
    }

    #[test]
    fn test_usage_quota() {
        let mut quota = UsageQuota::new(
            "API Calls".to_string(),
            Some(100),
            Some(Duration::days(1)),
        );

        assert!(quota.consume(50).is_ok());
        assert_eq!(quota.current_usage, 50);

        assert!(quota.consume(40).is_ok());
        assert_eq!(quota.current_usage, 90);

        // Should fail - exceeds quota
        assert!(quota.consume(20).is_err());

        // Should still be at 90
        assert_eq!(quota.current_usage, 90);
    }

    #[test]
    fn test_grace_period() {
        let mut grace = GracePeriod::new(7);

        assert_eq!(grace.status(), GracePeriodStatus::NotActive);

        grace.start();
        assert_eq!(grace.status(), GracePeriodStatus::Active);

        assert!(grace.is_feature_allowed(LicenseFeature::BasicDrawing));
        assert!(!grace.is_feature_allowed(LicenseFeature::Advanced3D));
    }

    #[test]
    fn test_entitlement_manager() {
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

        let mut manager = EntitlementManager::new(&license);

        // Should allow professional features
        assert!(manager
            .check_feature(&license, LicenseFeature::Advanced3D)
            .is_ok());

        // Should deny enterprise features
        assert!(manager
            .check_feature(&license, LicenseFeature::CloudSync)
            .is_err());

        // Test feature override
        manager.override_feature(LicenseFeature::CloudSync, true);
        assert!(manager
            .check_feature(&license, LicenseFeature::CloudSync)
            .is_ok());
    }
}
