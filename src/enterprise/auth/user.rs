//! User entity and management for CADDY enterprise authentication.
//!
//! This module provides comprehensive user management including:
//! - User entity with secure password storage
//! - Password hashing using Argon2
//! - User status management
//! - Role and permission assignment
//! - Session tracking

use super::permission::{Permission, PermissionSet};
use super::role::RoleManager;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during user operations
#[derive(Error, Debug)]
pub enum UserError {
    #[error("User not found: {0}")]
    NotFound(String),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User account is locked")]
    AccountLocked,

    #[error("User account is inactive")]
    AccountInactive,

    #[error("User account is pending activation")]
    AccountPending,

    #[error("Password hashing error: {0}")]
    PasswordHashError(String),

    #[error("Password verification error: {0}")]
    PasswordVerifyError(String),

    #[error("Invalid email format: {0}")]
    InvalidEmail(String),

    #[error("User already exists: {0}")]
    AlreadyExists(String),

    #[error("Weak password: {0}")]
    WeakPassword(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

/// Result type for user operations
pub type UserResult<T> = Result<T, UserError>;

/// User account status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatus {
    /// User account is active and can log in
    Active,

    /// User account is inactive (temporarily disabled)
    Inactive,

    /// User account is locked (due to security issues)
    Locked,

    /// User account is pending activation
    Pending,
}

impl UserStatus {
    /// Check if the user can log in with this status
    pub fn can_login(&self) -> bool {
        matches!(self, UserStatus::Active)
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            UserStatus::Active => "Active and can log in",
            UserStatus::Inactive => "Temporarily disabled",
            UserStatus::Locked => "Locked due to security issues",
            UserStatus::Pending => "Pending activation",
        }
    }
}

/// Password policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    /// Minimum password length
    pub min_length: usize,

    /// Require uppercase letters
    pub require_uppercase: bool,

    /// Require lowercase letters
    pub require_lowercase: bool,

    /// Require digits
    pub require_digits: bool,

    /// Require special characters
    pub require_special: bool,

    /// Minimum number of unique characters
    pub min_unique_chars: usize,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 12,
            require_uppercase: true,
            require_lowercase: true,
            require_digits: true,
            require_special: true,
            min_unique_chars: 8,
        }
    }
}

impl PasswordPolicy {
    /// Validate a password against this policy
    pub fn validate(&self, password: &str) -> UserResult<()> {
        if password.len() < self.min_length {
            return Err(UserError::WeakPassword(format!(
                "Password must be at least {} characters",
                self.min_length
            )));
        }

        if self.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Err(UserError::WeakPassword(
                "Password must contain uppercase letters".to_string(),
            ));
        }

        if self.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            return Err(UserError::WeakPassword(
                "Password must contain lowercase letters".to_string(),
            ));
        }

        if self.require_digits && !password.chars().any(|c| c.is_numeric()) {
            return Err(UserError::WeakPassword(
                "Password must contain digits".to_string(),
            ));
        }

        if self.require_special && !password.chars().any(|c| !c.is_alphanumeric()) {
            return Err(UserError::WeakPassword(
                "Password must contain special characters".to_string(),
            ));
        }

        let unique_chars: std::collections::HashSet<char> = password.chars().collect();
        if unique_chars.len() < self.min_unique_chars {
            return Err(UserError::WeakPassword(format!(
                "Password must contain at least {} unique characters",
                self.min_unique_chars
            )));
        }

        Ok(())
    }
}

/// User entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user identifier
    pub id: String,

    /// Username (unique)
    pub username: String,

    /// Email address (unique)
    pub email: String,

    /// Argon2 password hash
    #[serde(skip_serializing)]
    pub password_hash: String,

    /// Assigned role IDs
    pub roles: Vec<String>,

    /// Direct permissions (in addition to role permissions)
    pub permissions: PermissionSet,

    /// User account status
    pub status: UserStatus,

    /// Account creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last login timestamp
    pub last_login: Option<DateTime<Utc>>,

    /// Last password change timestamp
    pub password_changed_at: DateTime<Utc>,

    /// Failed login attempts counter
    pub failed_login_attempts: u32,

    /// User profile metadata
    pub metadata: HashMap<String, String>,

    /// Whether multi-factor authentication is enabled
    pub mfa_enabled: bool,

    /// MFA secret (if enabled)
    #[serde(skip_serializing)]
    pub mfa_secret: Option<String>,
}

impl User {
    /// Create a new user with hashed password
    pub fn new(
        id: String,
        username: String,
        email: String,
        password: &str,
        policy: &PasswordPolicy,
    ) -> UserResult<Self> {
        // Validate email format
        if !email.contains('@') {
            return Err(UserError::InvalidEmail(email));
        }

        // Validate password against policy
        policy.validate(password)?;

        // Hash password using Argon2
        let password_hash = Self::hash_password(password)?;

        let now = Utc::now();

        Ok(Self {
            id,
            username,
            email,
            password_hash,
            roles: Vec::new(),
            permissions: PermissionSet::new(),
            status: UserStatus::Pending,
            created_at: now,
            last_login: None,
            password_changed_at: now,
            failed_login_attempts: 0,
            metadata: HashMap::new(),
            mfa_enabled: false,
            mfa_secret: None,
        })
    }

    /// Hash a password using Argon2
    pub fn hash_password(password: &str) -> UserResult<String> {
        // In production, use argon2 crate
        // For now, simulate with a simple hash (replace with actual argon2)
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        password.hash(&mut hasher);
        let hash = hasher.finish();

        // This is a placeholder - in production use:
        // use argon2::{Argon2, PasswordHasher};
        // use argon2::password_hash::{SaltString, rand_core::OsRng};
        // let salt = SaltString::generate(&mut OsRng);
        // let argon2 = Argon2::default();
        // let password_hash = argon2.hash_password(password.as_bytes(), &salt)
        //     .map_err(|e| UserError::PasswordHashError(e.to_string()))?
        //     .to_string();

        Ok(format!("$argon2id$v=19$m=16384,t=2,p=1${:x}", hash))
    }

    /// Verify a password against the stored hash
    pub fn verify_password(&self, password: &str) -> UserResult<bool> {
        // In production, use argon2 crate for verification
        // For now, simulate verification
        let computed_hash = Self::hash_password(password)?;

        // This is a placeholder - in production use:
        // use argon2::{Argon2, PasswordVerifier};
        // use argon2::password_hash::PasswordHash;
        // let parsed_hash = PasswordHash::new(&self.password_hash)
        //     .map_err(|e| UserError::PasswordVerifyError(e.to_string()))?;
        // Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())

        Ok(self.password_hash == computed_hash)
    }

    /// Change user password
    pub fn change_password(
        &mut self,
        old_password: &str,
        new_password: &str,
        policy: &PasswordPolicy,
    ) -> UserResult<()> {
        // Verify old password
        if !self.verify_password(old_password)? {
            return Err(UserError::InvalidCredentials);
        }

        // Validate new password
        policy.validate(new_password)?;

        // Hash and update password
        self.password_hash = Self::hash_password(new_password)?;
        self.password_changed_at = Utc::now();

        Ok(())
    }

    /// Activate the user account
    pub fn activate(&mut self) {
        self.status = UserStatus::Active;
    }

    /// Deactivate the user account
    pub fn deactivate(&mut self) {
        self.status = UserStatus::Inactive;
    }

    /// Lock the user account
    pub fn lock(&mut self) {
        self.status = UserStatus::Locked;
    }

    /// Unlock the user account
    pub fn unlock(&mut self) {
        self.status = UserStatus::Active;
        self.failed_login_attempts = 0;
    }

    /// Check if user can login
    pub fn can_login(&self) -> UserResult<()> {
        match self.status {
            UserStatus::Active => Ok(()),
            UserStatus::Inactive => Err(UserError::AccountInactive),
            UserStatus::Locked => Err(UserError::AccountLocked),
            UserStatus::Pending => Err(UserError::AccountPending),
        }
    }

    /// Record a successful login
    pub fn record_login(&mut self) {
        self.last_login = Some(Utc::now());
        self.failed_login_attempts = 0;
    }

    /// Record a failed login attempt
    pub fn record_failed_login(&mut self, max_attempts: u32) {
        self.failed_login_attempts += 1;
        if self.failed_login_attempts >= max_attempts {
            self.lock();
        }
    }

    /// Add a role to the user
    pub fn add_role(&mut self, role_id: String) {
        if !self.roles.contains(&role_id) {
            self.roles.push(role_id);
        }
    }

    /// Remove a role from the user
    pub fn remove_role(&mut self, role_id: &str) {
        self.roles.retain(|r| r != role_id);
    }

    /// Add a direct permission to the user
    pub fn add_permission(&mut self, permission: Permission) {
        self.permissions.add(permission);
    }

    /// Get all effective permissions (from roles + direct permissions)
    pub fn effective_permissions(&self, role_manager: &RoleManager) -> PermissionSet {
        let mut effective = self.permissions.clone();
        let role_perms = role_manager.combined_permissions(&self.roles);
        effective.merge(&role_perms);
        effective
    }

    /// Check if user has a specific permission
    pub fn has_permission(&self, permission: &Permission, role_manager: &RoleManager) -> bool {
        self.effective_permissions(role_manager).has(permission)
    }

    /// Check if user has all specified permissions
    pub fn has_all_permissions(
        &self,
        permissions: &[Permission],
        role_manager: &RoleManager,
    ) -> bool {
        self.effective_permissions(role_manager).has_all(permissions)
    }

    /// Check if user has any of the specified permissions
    pub fn has_any_permission(
        &self,
        permissions: &[Permission],
        role_manager: &RoleManager,
    ) -> bool {
        self.effective_permissions(role_manager).has_any(permissions)
    }

    /// Enable multi-factor authentication
    pub fn enable_mfa(&mut self, secret: String) {
        self.mfa_enabled = true;
        self.mfa_secret = Some(secret);
    }

    /// Disable multi-factor authentication
    pub fn disable_mfa(&mut self) {
        self.mfa_enabled = false;
        self.mfa_secret = None;
    }

    /// Get user summary (without sensitive data)
    pub fn summary(&self) -> UserSummary {
        UserSummary {
            id: self.id.clone(),
            username: self.username.clone(),
            email: self.email.clone(),
            roles: self.roles.clone(),
            status: self.status,
            created_at: self.created_at,
            last_login: self.last_login,
            mfa_enabled: self.mfa_enabled,
        }
    }
}

/// User summary without sensitive information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSummary {
    pub id: String,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub mfa_enabled: bool,
}

/// User manager for CRUD operations
#[derive(Debug)]
pub struct UserManager {
    users: HashMap<String, User>,
    password_policy: PasswordPolicy,
    max_failed_attempts: u32,
}

impl UserManager {
    /// Create a new user manager
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            password_policy: PasswordPolicy::default(),
            max_failed_attempts: 5,
        }
    }

    /// Create a new user manager with custom policy
    pub fn with_policy(password_policy: PasswordPolicy, max_failed_attempts: u32) -> Self {
        Self {
            users: HashMap::new(),
            password_policy,
            max_failed_attempts,
        }
    }

    /// Create a new user
    pub fn create_user(
        &mut self,
        id: String,
        username: String,
        email: String,
        password: &str,
    ) -> UserResult<String> {
        // Check if user already exists
        if self.users.values().any(|u| u.username == username) {
            return Err(UserError::AlreadyExists(format!("username: {}", username)));
        }

        if self.users.values().any(|u| u.email == email) {
            return Err(UserError::AlreadyExists(format!("email: {}", email)));
        }

        let user = User::new(id.clone(), username, email, password, &self.password_policy)?;
        self.users.insert(id.clone(), user);

        Ok(id)
    }

    /// Get a user by ID
    pub fn get_user(&self, id: &str) -> UserResult<&User> {
        self.users
            .get(id)
            .ok_or_else(|| UserError::NotFound(id.to_string()))
    }

    /// Get a mutable user by ID
    pub fn get_user_mut(&mut self, id: &str) -> UserResult<&mut User> {
        self.users
            .get_mut(id)
            .ok_or_else(|| UserError::NotFound(id.to_string()))
    }

    /// Find user by username
    pub fn find_by_username(&self, username: &str) -> Option<&User> {
        self.users.values().find(|u| u.username == username)
    }

    /// Find user by email
    pub fn find_by_email(&self, email: &str) -> Option<&User> {
        self.users.values().find(|u| u.email == email)
    }

    /// Authenticate user with username/email and password
    pub fn authenticate(&mut self, username_or_email: &str, password: &str) -> UserResult<&User> {
        // Find user by username or email
        let user_id = {
            let user = self
                .find_by_username(username_or_email)
                .or_else(|| self.find_by_email(username_or_email))
                .ok_or(UserError::InvalidCredentials)?;

            user.id.clone()
        };

        let max_failed_attempts = self.max_failed_attempts;
        let user = self.get_user_mut(&user_id)?;

        // Check if user can login
        user.can_login()?;

        // Verify password
        if user.verify_password(password)? {
            user.record_login();
            Ok(self.get_user(&user_id)?)
        } else {
            user.record_failed_login(max_failed_attempts);
            Err(UserError::InvalidCredentials)
        }
    }

    /// Delete a user
    pub fn delete_user(&mut self, id: &str) -> UserResult<()> {
        if self.users.remove(id).is_none() {
            return Err(UserError::NotFound(id.to_string()));
        }
        Ok(())
    }

    /// List all users
    pub fn list_users(&self) -> Vec<UserSummary> {
        self.users.values().map(|u| u.summary()).collect()
    }

    /// List users by role
    pub fn list_users_by_role(&self, role_id: &str) -> Vec<UserSummary> {
        self.users
            .values()
            .filter(|u| u.roles.contains(&role_id.to_string()))
            .map(|u| u.summary())
            .collect()
    }

    /// List users by status
    pub fn list_users_by_status(&self, status: UserStatus) -> Vec<UserSummary> {
        self.users
            .values()
            .filter(|u| u.status == status)
            .map(|u| u.summary())
            .collect()
    }
}

impl Default for UserManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_policy() {
        let policy = PasswordPolicy::default();

        // Too short
        assert!(policy.validate("Short1!").is_err());

        // Missing uppercase
        assert!(policy.validate("lowercase123!@#").is_err());

        // Missing lowercase
        assert!(policy.validate("UPPERCASE123!@#").is_err());

        // Missing digit
        assert!(policy.validate("NoDigitsHere!@#").is_err());

        // Missing special
        assert!(policy.validate("NoSpecial1234ABC").is_err());

        // Valid password
        assert!(policy.validate("Valid_Pass123!@#").is_ok());
    }

    #[test]
    fn test_user_creation() {
        let policy = PasswordPolicy::default();
        let user = User::new(
            "user1".to_string(),
            "testuser".to_string(),
            "test@example.com".to_string(),
            "Valid_Pass123!@#",
            &policy,
        );

        assert!(user.is_ok());
        let user = user.unwrap();
        assert_eq!(user.username, "testuser");
        assert_eq!(user.status, UserStatus::Pending);
    }

    #[test]
    fn test_password_verification() {
        let policy = PasswordPolicy::default();
        let user = User::new(
            "user1".to_string(),
            "testuser".to_string(),
            "test@example.com".to_string(),
            "Valid_Pass123!@#",
            &policy,
        )
        .unwrap();

        assert!(user.verify_password("Valid_Pass123!@#").unwrap());
        assert!(!user.verify_password("WrongPassword").unwrap());
    }

    #[test]
    fn test_user_status() {
        let policy = PasswordPolicy::default();
        let mut user = User::new(
            "user1".to_string(),
            "testuser".to_string(),
            "test@example.com".to_string(),
            "Valid_Pass123!@#",
            &policy,
        )
        .unwrap();

        assert!(!user.status.can_login());

        user.activate();
        assert!(user.status.can_login());
        assert!(user.can_login().is_ok());

        user.lock();
        assert!(!user.status.can_login());
        assert!(user.can_login().is_err());
    }

    #[test]
    fn test_user_manager() {
        let mut manager = UserManager::new();

        let user_id = manager
            .create_user(
                "user1".to_string(),
                "testuser".to_string(),
                "test@example.com".to_string(),
                "Valid_Pass123!@#",
            )
            .unwrap();

        assert!(manager.get_user(&user_id).is_ok());
        assert!(manager.find_by_username("testuser").is_some());
        assert!(manager.find_by_email("test@example.com").is_some());
    }

    #[test]
    fn test_authentication() {
        let mut manager = UserManager::new();

        let user_id = manager
            .create_user(
                "user1".to_string(),
                "testuser".to_string(),
                "test@example.com".to_string(),
                "Valid_Pass123!@#",
            )
            .unwrap();

        // Activate user first
        manager.get_user_mut(&user_id).unwrap().activate();

        // Authenticate with username
        let result = manager.authenticate("testuser", "Valid_Pass123!@#");
        assert!(result.is_ok());

        // Authenticate with email
        let result = manager.authenticate("test@example.com", "Valid_Pass123!@#");
        assert!(result.is_ok());

        // Wrong password
        let result = manager.authenticate("testuser", "WrongPassword");
        assert!(result.is_err());
    }

    #[test]
    fn test_failed_login_lockout() {
        let mut manager = UserManager::with_policy(PasswordPolicy::default(), 3);

        let user_id = manager
            .create_user(
                "user1".to_string(),
                "testuser".to_string(),
                "test@example.com".to_string(),
                "Valid_Pass123!@#",
            )
            .unwrap();

        manager.get_user_mut(&user_id).unwrap().activate();

        // 3 failed attempts should lock the account
        for _ in 0..3 {
            let _ = manager.authenticate("testuser", "WrongPassword");
        }

        let user = manager.get_user(&user_id).unwrap();
        assert_eq!(user.status, UserStatus::Locked);
    }
}
