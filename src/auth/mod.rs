//! # CADDY Enterprise Authentication System
//!
//! Production-grade authentication and authorization system following OWASP security guidelines.
//!
//! ## Modules
//!
//! - `sso`: Single Sign-On with SAML 2.0, OAuth 2.0/OIDC, AD/LDAP
//! - `rbac`: Role-Based Access Control with resource-level permissions
//! - `mfa`: Multi-Factor Authentication with TOTP, SMS, Email, FIDO2
//! - `sessions`: Secure session management with audit logging
//! - `audit`: Comprehensive security audit logging and anomaly detection

pub mod sso;
pub mod rbac;
pub mod mfa;
pub mod sessions;
pub mod audit;

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use thiserror::Error;

/// Authentication result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResult {
    /// Whether authentication was successful
    pub success: bool,
    /// User ID if successful
    pub user_id: Option<Uuid>,
    /// Session token if successful
    pub session_token: Option<String>,
    /// Refresh token if successful
    pub refresh_token: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
    /// Whether MFA is required
    pub mfa_required: bool,
    /// MFA methods available
    pub mfa_methods: Vec<String>,
    /// Session expiry time
    pub expires_at: Option<DateTime<Utc>>,
}

/// User context for authorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    /// User ID
    pub user_id: Uuid,
    /// Username
    pub username: String,
    /// Email address
    pub email: String,
    /// User roles
    pub roles: Vec<String>,
    /// Custom permissions
    pub permissions: Vec<String>,
    /// Organization ID
    pub organization_id: Option<Uuid>,
    /// Department
    pub department: Option<String>,
    /// IP address
    pub ip_address: IpAddr,
    /// User agent
    pub user_agent: String,
    /// Session ID
    pub session_id: Uuid,
    /// Authentication timestamp
    pub authenticated_at: DateTime<Utc>,
}

/// Authentication error types
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Account locked due to failed attempts")]
    AccountLocked,

    #[error("MFA required")]
    MfaRequired,

    #[error("Invalid MFA code")]
    InvalidMfaCode,

    #[error("Session expired")]
    SessionExpired,

    #[error("Invalid session token")]
    InvalidSessionToken,

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid SSO provider: {0}")]
    InvalidSsoProvider(String),

    #[error("SSO authentication failed: {0}")]
    SsoAuthFailed(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("IP address blocked")]
    IpBlocked,

    #[error("Anomalous activity detected")]
    AnomalousActivity,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Cryptography error: {0}")]
    CryptoError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<sqlx::Error> for AuthError {
    fn from(err: sqlx::Error) -> Self {
        AuthError::DatabaseError(err.to_string())
    }
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Session timeout in seconds
    pub session_timeout: u64,

    /// Refresh token lifetime in seconds
    pub refresh_token_lifetime: u64,

    /// Maximum concurrent sessions per user
    pub max_concurrent_sessions: usize,

    /// Maximum failed login attempts before lockout
    pub max_failed_attempts: u32,

    /// Account lockout duration in seconds
    pub lockout_duration: u64,

    /// Password minimum length
    pub password_min_length: usize,

    /// Password complexity requirements
    pub password_require_uppercase: bool,
    pub password_require_lowercase: bool,
    pub password_require_numbers: bool,
    pub password_require_special: bool,

    /// MFA enforcement
    pub mfa_required_for_admin: bool,
    pub mfa_required_for_all: bool,

    /// IP-based rate limiting
    pub rate_limit_enabled: bool,
    pub rate_limit_requests: u32,
    pub rate_limit_window_secs: u64,

    /// Anomaly detection
    pub anomaly_detection_enabled: bool,

    /// JWT signing key (keep secret!)
    #[serde(skip_serializing)]
    pub jwt_secret: String,

    /// JWT issuer
    pub jwt_issuer: String,

    /// JWT audience
    pub jwt_audience: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            session_timeout: 3600,              // 1 hour
            refresh_token_lifetime: 604800,     // 7 days
            max_concurrent_sessions: 5,
            max_failed_attempts: 5,
            lockout_duration: 900,              // 15 minutes
            password_min_length: 12,
            password_require_uppercase: true,
            password_require_lowercase: true,
            password_require_numbers: true,
            password_require_special: true,
            mfa_required_for_admin: true,
            mfa_required_for_all: false,
            rate_limit_enabled: true,
            rate_limit_requests: 10,
            rate_limit_window_secs: 60,
            anomaly_detection_enabled: true,
            jwt_secret: String::new(), // Must be set!
            jwt_issuer: "caddy-auth".to_string(),
            jwt_audience: "caddy-users".to_string(),
        }
    }
}

/// Password validation
pub fn validate_password(password: &str, config: &AuthConfig) -> Result<(), String> {
    if password.len() < config.password_min_length {
        return Err(format!(
            "Password must be at least {} characters",
            config.password_min_length
        ));
    }

    if config.password_require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
        return Err("Password must contain at least one uppercase letter".to_string());
    }

    if config.password_require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
        return Err("Password must contain at least one lowercase letter".to_string());
    }

    if config.password_require_numbers && !password.chars().any(|c| c.is_numeric()) {
        return Err("Password must contain at least one number".to_string());
    }

    if config.password_require_special
        && !password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c)) {
        return Err("Password must contain at least one special character".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_validation() {
        let config = AuthConfig::default();

        // Valid password
        assert!(validate_password("MyP@ssw0rd123!", &config).is_ok());

        // Too short
        assert!(validate_password("Short1!", &config).is_err());

        // No uppercase
        assert!(validate_password("myp@ssw0rd123!", &config).is_err());

        // No lowercase
        assert!(validate_password("MYP@SSW0RD123!", &config).is_err());

        // No numbers
        assert!(validate_password("MyP@ssword!!!", &config).is_err());

        // No special characters
        assert!(validate_password("MyPassword123", &config).is_err());
    }
}
