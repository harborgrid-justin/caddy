//! Enterprise Authentication and RBAC System for CADDY v0.1.5
//!
//! This module provides a comprehensive authentication and authorization framework including:
//!
//! - **Fine-grained permissions**: Granular permission model for all CAD operations
//! - **Role-based access control (RBAC)**: Hierarchical role system with built-in and custom roles
//! - **User management**: Complete user lifecycle with secure password storage using Argon2
//! - **Session management**: JWT-based authentication with token refresh
//! - **Policy engine**: Attribute-based access control (ABAC) for complex authorization rules
//! - **Multi-provider authentication**: Support for local, LDAP, OAuth2, and OIDC authentication
//!
//! # Architecture
//!
//! The authentication system is composed of several layers:
//!
//! 1. **Permission Layer**: Defines all possible operations in the system
//! 2. **Role Layer**: Groups permissions into roles with hierarchy
//! 3. **User Layer**: Manages user accounts and credentials
//! 4. **Session Layer**: Handles authentication tokens and sessions
//! 5. **Policy Layer**: Evaluates complex authorization rules
//! 6. **Provider Layer**: Integrates with external authentication systems
//!
//! # Example Usage
//!
//! ```rust,ignore
//! use caddy::enterprise::auth::*;
//!
//! // Initialize the authentication system
//! let mut user_manager = UserManager::new();
//! let role_manager = RoleManager::new();
//! let jwt_manager = JwtManager::new("secret_key".to_string());
//! let mut session_manager = SessionManager::new(jwt_manager);
//!
//! // Create a user
//! let user_id = user_manager.create_user(
//!     "user1".to_string(),
//!     "john_doe".to_string(),
//!     "john@example.com".to_string(),
//!     "SecurePass123!@#",
//! ).unwrap();
//!
//! // Assign a role
//! let user = user_manager.get_user_mut(&user_id).unwrap();
//! user.add_role("designer".to_string());
//! user.activate();
//!
//! // Authenticate
//! let authenticated_user = user_manager.authenticate("john_doe", "SecurePass123!@#").unwrap();
//!
//! // Create a session
//! let session = session_manager.create_session(
//!     authenticated_user.id.clone(),
//!     authenticated_user.username.clone(),
//!     authenticated_user.email.clone(),
//!     authenticated_user.roles.clone(),
//!     Some("127.0.0.1".to_string()),
//!     Some("Mozilla/5.0".to_string()),
//! ).unwrap();
//!
//! // Check permissions
//! if authenticated_user.has_permission(&Permission::DrawingCreate, &role_manager) {
//!     println!("User can create drawings");
//! }
//!
//! // Use policy engine for complex rules
//! let mut policy_engine = PolicyEngine::new();
//! let policy = PolicyBuilder::time_based_access(
//!     "work_hours".to_string(),
//!     Permission::DrawingCreate,
//!     "drawing:*".to_string(),
//!     9,  // 9 AM
//!     17, // 5 PM
//! );
//! policy_engine.add_policy(policy).unwrap();
//! ```
//!
//! # Security Considerations
//!
//! - Passwords are hashed using Argon2 (industry-standard)
//! - Sessions use JWT tokens with expiration
//! - Role hierarchy prevents privilege escalation
//! - Policy engine supports explicit deny rules
//! - Failed login attempts trigger account lockout
//! - Multi-factor authentication support
//!
//! # Production Deployment
//!
//! For production use, ensure:
//!
//! 1. Use strong, random JWT secrets (32+ bytes)
//! 2. Enable TLS/SSL for all connections
//! 3. Configure appropriate session timeouts
//! 4. Enable audit logging for security events
//! 5. Regularly rotate JWT signing keys
//! 6. Use secure password policies
//! 7. Enable MFA for privileged accounts
//! 8. Regular security audits and updates

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

// Re-export all public types and functions
pub mod permission;
pub mod role;
pub mod user;
pub mod session;
pub mod policy;
pub mod provider;

// Re-export commonly used types for convenience
pub use permission::{
    Permission, PermissionCategory, PermissionSet, ResourcePermission,
};

pub use role::{
    BuiltInRole, Role, RoleError, RoleManager, RoleResult,
};

pub use user::{
    PasswordPolicy, User, UserError, UserManager, UserResult,
    UserStatus, UserSummary,
};

pub use session::{
    Claims, JwtManager, Session, SessionError, SessionManager,
    SessionResult, Token,
};

pub use policy::{
    Condition, Effect, Operator, Policy, PolicyBuilder,
    PolicyContext, PolicyEngine, PolicyError, PolicyResult,
    Statement,
};

pub use provider::{
    AuthProvider, AuthProviderManager, AuthenticationResult,
    Credentials, LDAPAuthProvider, LDAPProviderConfig,
    LocalAuthProvider, LocalProviderConfig, OAuth2AuthProvider,
    OAuth2ProviderConfig, ProviderError, ProviderResult,
    ProviderType,
};

/// Enterprise authentication system facade
///
/// Provides a unified interface to all authentication and authorization components.
pub struct AuthSystem {
    /// User management
    pub user_manager: UserManager,

    /// Role management
    pub role_manager: RoleManager,

    /// Session management
    pub session_manager: SessionManager,

    /// Policy engine
    pub policy_engine: PolicyEngine,

    /// Authentication providers
    pub provider_manager: AuthProviderManager,
}

impl AuthSystem {
    /// Create a new authentication system with default configuration
    pub fn new(jwt_secret: String) -> Self {
        let user_manager = UserManager::new();
        let role_manager = RoleManager::new();
        let jwt_manager = JwtManager::new(jwt_secret);
        let session_manager = SessionManager::new(jwt_manager);
        let policy_engine = PolicyEngine::new();
        let mut provider_manager = AuthProviderManager::new();

        // Register default local provider
        let local_provider = Box::new(LocalAuthProvider::new(LocalProviderConfig::default()));
        provider_manager.register_provider("local".to_string(), local_provider);
        provider_manager.set_default_provider("local".to_string()).ok();

        Self {
            user_manager,
            role_manager,
            session_manager,
            policy_engine,
            provider_manager,
        }
    }

    /// Create a new authentication system with custom configuration
    pub fn with_config(
        jwt_secret: String,
        password_policy: PasswordPolicy,
        max_failed_attempts: u32,
    ) -> Self {
        let user_manager = UserManager::with_policy(password_policy, max_failed_attempts);
        let role_manager = RoleManager::new();
        let jwt_manager = JwtManager::new(jwt_secret);
        let session_manager = SessionManager::new(jwt_manager);
        let policy_engine = PolicyEngine::new();
        let mut provider_manager = AuthProviderManager::new();

        // Register default local provider
        let local_provider = Box::new(LocalAuthProvider::new(LocalProviderConfig::default()));
        provider_manager.register_provider("local".to_string(), local_provider);
        provider_manager.set_default_provider("local".to_string()).ok();

        Self {
            user_manager,
            role_manager,
            session_manager,
            policy_engine,
            provider_manager,
        }
    }

    /// Authenticate a user and create a session
    pub fn login(
        &mut self,
        username: &str,
        password: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(User, Session), UserError> {
        // Authenticate user
        let user = self.user_manager.authenticate(username, password)?;

        // Create session
        let session = self
            .session_manager
            .create_session(
                user.id.clone(),
                user.username.clone(),
                user.email.clone(),
                user.roles.clone(),
                ip_address,
                user_agent,
            )
            .map_err(|e| UserError::PermissionDenied(e.to_string()))?;

        Ok((user.clone(), session))
    }

    /// Logout a user by invalidating their session
    pub fn logout(&mut self, session_id: &str) -> Result<(), SessionError> {
        self.session_manager.invalidate_session(session_id)
    }

    /// Verify a token and get user information
    pub fn verify_token(&mut self, token: &str) -> Result<User, UserError> {
        let claims = self
            .session_manager
            .verify_access_token(token)
            .map_err(|e| UserError::PermissionDenied(e.to_string()))?;

        self.user_manager
            .get_user(&claims.sub)
            .map(|u| u.clone())
    }

    /// Check if a user has permission for an action
    pub fn check_permission(
        &mut self,
        user: &User,
        permission: &Permission,
        resource: &str,
    ) -> bool {
        self.policy_engine.check_permission(user, permission, resource, &self.role_manager)
    }

    /// Create a new user with role assignment
    pub fn create_user_with_role(
        &mut self,
        id: String,
        username: String,
        email: String,
        password: &str,
        role: BuiltInRole,
    ) -> UserResult<String> {
        let user_id = self.user_manager.create_user(id, username, email, password)?;
        let user = self.user_manager.get_user_mut(&user_id)?;
        user.add_role(role.as_str().to_lowercase());
        user.activate();
        Ok(user_id)
    }

    /// Perform system health check
    pub fn health_check(&self) -> HashMap<String, bool> {
        let mut status = HashMap::new();

        status.insert("user_manager".to_string(), true);
        status.insert("role_manager".to_string(), true);
        status.insert("session_manager".to_string(), true);
        status.insert("policy_engine".to_string(), true);

        // Check auth providers
        for (name, healthy) in self.provider_manager.health_check_all() {
            status.insert(format!("provider_{}", name), healthy);
        }

        status
    }

    /// Get system statistics
    pub fn statistics(&self) -> AuthSystemStatistics {
        AuthSystemStatistics {
            total_users: self.user_manager.list_users().len(),
            active_users: self.user_manager.list_users_by_status(UserStatus::Active).len(),
            total_roles: self.role_manager.list_roles().len(),
            active_sessions: self.session_manager.active_session_count(),
            total_sessions: self.session_manager.session_count(),
            total_policies: self.policy_engine.list_policies().len(),
            registered_providers: self.provider_manager.list_providers().len(),
        }
    }

    /// Clean up expired sessions and cache
    pub fn cleanup(&mut self) {
        self.session_manager.cleanup();
        self.policy_engine.cleanup_cache();
    }
}

/// System statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSystemStatistics {
    pub total_users: usize,
    pub active_users: usize,
    pub total_roles: usize,
    pub active_sessions: usize,
    pub total_sessions: usize,
    pub total_policies: usize,
    pub registered_providers: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_system_initialization() {
        let auth_system = AuthSystem::new("test_secret_key_12345678".to_string());

        let stats = auth_system.statistics();
        assert_eq!(stats.total_roles, 5); // 5 built-in roles
        assert_eq!(stats.registered_providers, 1); // local provider
    }

    #[test]
    fn test_complete_auth_flow() {
        let mut auth_system = AuthSystem::new("test_secret_key_12345678".to_string());

        // Create a user
        let user_id = auth_system
            .create_user_with_role(
                "user1".to_string(),
                "testuser".to_string(),
                "test@example.com".to_string(),
                "SecurePass123!@#",
                BuiltInRole::Designer,
            )
            .unwrap();

        // Login
        let (user, session) = auth_system
            .login(
                "testuser",
                "SecurePass123!@#",
                Some("127.0.0.1".to_string()),
                Some("TestAgent/1.0".to_string()),
            )
            .unwrap();

        assert_eq!(user.id, user_id);
        assert!(session.is_valid());

        // Verify token
        let verified_user = auth_system.verify_token(&session.access_token.token).unwrap();
        assert_eq!(verified_user.id, user_id);

        // Check permission
        assert!(auth_system.check_permission(&user, &Permission::DrawingCreate, "drawing:123"));

        // Logout
        assert!(auth_system.logout(&session.id).is_ok());
    }

    #[test]
    fn test_permission_checking() {
        let mut auth_system = AuthSystem::new("test_secret_key_12345678".to_string());

        let user_id = auth_system
            .create_user_with_role(
                "user1".to_string(),
                "viewer".to_string(),
                "viewer@example.com".to_string(),
                "SecurePass123!@#",
                BuiltInRole::Viewer,
            )
            .unwrap();

        let user = auth_system.user_manager.get_user(&user_id).unwrap();

        // Viewer can read
        assert!(auth_system.check_permission(user, &Permission::DrawingRead, "drawing:123"));

        // Viewer cannot create
        assert!(!user.has_permission(&Permission::DrawingCreate, &auth_system.role_manager));
    }

    #[test]
    fn test_health_check() {
        let auth_system = AuthSystem::new("test_secret_key_12345678".to_string());
        let health = auth_system.health_check();

        assert!(health.get("user_manager").unwrap());
        assert!(health.get("provider_local").unwrap());
    }

    #[test]
    fn test_statistics() {
        let mut auth_system = AuthSystem::new("test_secret_key_12345678".to_string());

        auth_system
            .create_user_with_role(
                "user1".to_string(),
                "user1".to_string(),
                "user1@example.com".to_string(),
                "SecurePass123!@#",
                BuiltInRole::Designer,
            )
            .unwrap();

        let stats = auth_system.statistics();
        assert_eq!(stats.total_users, 1);
        assert_eq!(stats.active_users, 1);
        assert_eq!(stats.total_roles, 5);
    }
}
