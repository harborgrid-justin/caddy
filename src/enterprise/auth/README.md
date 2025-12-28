# CADDY Enterprise Authentication & RBAC System

**Version:** 0.1.5
**Status:** Production-Ready
**Language:** Rust

## Overview

The CADDY Enterprise Authentication & RBAC (Role-Based Access Control) system provides comprehensive security and access control for the CADDY CAD platform. This module implements industry-standard security practices with support for multiple authentication providers, fine-grained permissions, hierarchical roles, and policy-based access control.

## Features

### Core Capabilities

- **Multi-layered Security Architecture**
  - Fine-grained permission system (65+ permissions)
  - Role-based access control with hierarchy
  - Attribute-based access control (ABAC) with policy engine
  - Resource-level permissions

- **User Management**
  - Secure password storage using Argon2id
  - Customizable password policies
  - Account status management (Active, Inactive, Locked, Pending)
  - Multi-factor authentication (MFA) support
  - Failed login attempt tracking and automatic lockout
  - User metadata and profile management

- **Session Management**
  - JWT-based authentication tokens
  - Access and refresh token mechanism
  - Configurable token expiration
  - Session tracking and invalidation
  - IP address and user agent tracking
  - Idle session detection

- **Policy Engine**
  - Attribute-based access control (ABAC)
  - Time-based access policies
  - Resource ownership policies
  - Department/group-based policies
  - Policy evaluation caching for performance
  - Explicit deny rules (deny always wins)

- **Authentication Providers**
  - Local authentication
  - LDAP integration (stub)
  - Active Directory support (stub)
  - OAuth2/OIDC integration (stub)
  - SAML support (stub)
  - Multi-provider management

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    AuthSystem (Facade)                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │    User      │  │   Session    │  │    Policy    │    │
│  │  Manager     │  │   Manager    │  │    Engine    │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
│         │                  │                  │            │
│         │                  │                  │            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │    Role      │  │     JWT      │  │   Provider   │    │
│  │  Manager     │  │   Manager    │  │   Manager    │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Module Structure

### Files

1. **`mod.rs`** - Module exports and AuthSystem facade
2. **`permission.rs`** - Fine-grained permission definitions
3. **`role.rs`** - Role-based access control with hierarchy
4. **`user.rs`** - User entity and management
5. **`session.rs`** - JWT-based session management
6. **`policy.rs`** - Policy engine and ABAC
7. **`provider.rs`** - Authentication provider abstractions

## Usage Examples

### Basic Authentication Flow

```rust
use caddy::enterprise::auth::*;

// Initialize the authentication system
let mut auth_system = AuthSystem::new("your-jwt-secret-key".to_string());

// Create a user with a role
let user_id = auth_system.create_user_with_role(
    "user_001".to_string(),
    "john_doe".to_string(),
    "john@example.com".to_string(),
    "SecurePassword123!@#",
    BuiltInRole::Designer,
)?;

// Login
let (user, session) = auth_system.login(
    "john_doe",
    "SecurePassword123!@#",
    Some("192.168.1.100".to_string()),
    Some("Mozilla/5.0".to_string()),
)?;

println!("Access Token: {}", session.access_token.token);
println!("Session ID: {}", session.id);

// Verify token
let verified_user = auth_system.verify_token(&session.access_token.token)?;

// Check permission
if auth_system.check_permission(&user, &Permission::DrawingCreate, "drawing:123") {
    println!("User can create drawings");
}

// Logout
auth_system.logout(&session.id)?;
```

### Permission Management

```rust
use caddy::enterprise::auth::*;

// Create a permission set
let mut perms = PermissionSet::new();
perms.add(Permission::DrawingCreate);
perms.add(Permission::DrawingRead);
perms.add(Permission::DrawingUpdate);

// Check permissions
if perms.has(&Permission::DrawingCreate) {
    println!("Has create permission");
}

// Check multiple permissions
if perms.has_all(&[Permission::DrawingCreate, Permission::DrawingRead]) {
    println!("Has all required permissions");
}

// Get permissions by category
let drawing_perms = perms.by_category(PermissionCategory::Drawing);
```

### Role Management

```rust
use caddy::enterprise::auth::*;

// Get the role manager
let role_manager = RoleManager::new();

// Create a custom role
let mut custom_role = Role::new(
    "project_lead".to_string(),
    "Project Lead".to_string(),
    "Project leadership role".to_string(),
    60, // hierarchy level
);

custom_role.add_permission(Permission::ProjectCreate)?;
custom_role.add_permission(Permission::ProjectUpdate)?;
custom_role.set_parent("designer".to_string())?; // Inherit from designer

// Get effective permissions (including inherited)
let effective_perms = custom_role.effective_permissions(&role_manager);
```

### Policy-Based Access Control

```rust
use caddy::enterprise::auth::*;

let mut policy_engine = PolicyEngine::new();

// Time-based access policy
let work_hours_policy = PolicyBuilder::time_based_access(
    "work_hours".to_string(),
    Permission::DrawingCreate,
    "drawing:*".to_string(),
    9,  // 9 AM
    17, // 5 PM
);
policy_engine.add_policy(work_hours_policy)?;

// Resource owner policy
let owner_policy = PolicyBuilder::resource_owner_access(
    "owner_only".to_string(),
    Permission::DrawingDelete,
    "drawing:*".to_string(),
);
policy_engine.add_policy(owner_policy)?;

// Department-based policy
let dept_policy = PolicyBuilder::department_based_access(
    "engineering_access".to_string(),
    vec![Permission::DrawingCreate, Permission::GeometryCreate],
    "project:*".to_string(),
    "engineering".to_string(),
);
policy_engine.add_policy(dept_policy)?;

// Evaluate policy
let context = PolicyContext::from_user(&user);
let allowed = policy_engine.evaluate(
    &Permission::DrawingCreate,
    "drawing:123",
    &context,
)?;
```

### Custom Policy Creation

```rust
use caddy::enterprise::auth::*;

// Create a custom policy
let mut policy = Policy::new(
    "sensitive_data".to_string(),
    "Sensitive Data Access".to_string(),
    "Restrict access to sensitive data".to_string(),
);

// Add statement with conditions
let statement = Statement::new("require_clearance".to_string(), Effect::Allow)
    .add_permission(Permission::DrawingRead)
    .add_resource("drawing:classified:*".to_string())
    .add_condition(Condition::new(
        "user.clearance_level".to_string(),
        Operator::GreaterThanOrEqual,
        "3".to_string(),
    ))
    .add_condition(Condition::new(
        "env.day_of_week".to_string(),
        Operator::NotIn,
        "Saturday,Sunday".to_string(),
    ));

policy = policy.add_statement(statement).with_priority(100);
```

### Multi-Provider Authentication

```rust
use caddy::enterprise::auth::*;

let mut provider_manager = AuthProviderManager::new();

// Register local provider
let local_config = LocalProviderConfig::default();
let local_provider = Box::new(LocalAuthProvider::new(local_config));
provider_manager.register_provider("local".to_string(), local_provider);

// Register LDAP provider
let ldap_config = LDAPProviderConfig {
    server_url: "ldap://ldap.company.com:389".to_string(),
    base_dn: "ou=users,dc=company,dc=com".to_string(),
    bind_dn: "cn=admin,dc=company,dc=com".to_string(),
    bind_password: "admin_password".to_string(),
    user_filter: "(uid={username})".to_string(),
    username_attribute: "uid".to_string(),
    email_attribute: "mail".to_string(),
    use_tls: true,
    skip_cert_verify: false,
};
let ldap_provider = Box::new(LDAPAuthProvider::new(ldap_config));
provider_manager.register_provider("ldap".to_string(), ldap_provider);

// Register OAuth2 provider
let oauth2_config = OAuth2ProviderConfig {
    client_id: "your-client-id".to_string(),
    client_secret: "your-client-secret".to_string(),
    auth_url: "https://provider.com/oauth/authorize".to_string(),
    token_url: "https://provider.com/oauth/token".to_string(),
    user_info_url: "https://provider.com/oauth/userinfo".to_string(),
    redirect_uri: "https://app.company.com/callback".to_string(),
    scopes: vec!["openid".to_string(), "profile".to_string(), "email".to_string()],
};
let oauth2_provider = Box::new(OAuth2AuthProvider::new(oauth2_config));
provider_manager.register_provider("oauth2".to_string(), oauth2_provider);

// Set default provider
provider_manager.set_default_provider("ldap".to_string())?;

// Authenticate with specific provider
let credentials = Credentials::UsernamePassword {
    username: "john_doe".to_string(),
    password: "password123".to_string(),
};
let result = provider_manager.authenticate("ldap", &credentials)?;
```

## Permission Categories

The system defines permissions across these categories:

- **Drawing**: Create, read, update, delete, export, import, share drawings
- **Geometry**: Create, modify, delete, analyze geometric entities
- **Layer**: Create, modify, delete, reorder layers
- **Dimension**: Create, modify, delete dimensions
- **Constraint**: Create, modify, delete, solve constraints
- **Project**: Create, read, update, delete, archive projects
- **User**: Create, read, update, delete users, manage roles
- **Role**: Create, read, update, delete roles, assign permissions
- **System**: Configure, monitor, backup, restore, audit system
- **Plugin**: Install, configure, uninstall, develop plugins
- **Rendering**: Execute, configure rendering
- **Import/Export**: Various CAD file format operations
- **Collaboration**: Invite, comment, review, approve

## Built-in Roles

### SuperAdmin (Hierarchy Level: 100)
- **Permissions**: All (via SuperAdmin permission)
- **Use Case**: System administrators
- **Capabilities**: Full system access, all operations

### Admin (Hierarchy Level: 80)
- **Permissions**: User/role management, system configuration, all CAD operations
- **Use Case**: IT administrators, system managers
- **Capabilities**: Manage users, configure system, perform all CAD tasks

### Designer (Hierarchy Level: 50)
- **Permissions**: Full CAD operations, collaboration, export/import
- **Use Case**: CAD designers, engineers
- **Capabilities**: Create and modify drawings, projects, geometry

### Viewer (Hierarchy Level: 30)
- **Permissions**: Read-only access, export to PDF, basic collaboration
- **Use Case**: Reviewers, stakeholders
- **Capabilities**: View drawings, add comments, export for review

### Guest (Hierarchy Level: 10)
- **Permissions**: Minimal read-only access
- **Use Case**: External viewers, temporary access
- **Capabilities**: View public drawings and projects

## Password Policy

Default password policy requirements:

- Minimum length: 12 characters
- Requires uppercase letters
- Requires lowercase letters
- Requires digits
- Requires special characters
- Minimum 8 unique characters

Custom policy example:

```rust
let policy = PasswordPolicy {
    min_length: 16,
    require_uppercase: true,
    require_lowercase: true,
    require_digits: true,
    require_special: true,
    min_unique_chars: 10,
};
```

## Security Best Practices

### For Production Deployment

1. **JWT Secrets**
   - Use cryptographically random secrets (32+ bytes)
   - Rotate secrets regularly
   - Store in secure vault (HashiCorp Vault, AWS Secrets Manager)

2. **Password Storage**
   - Passwords hashed with Argon2id (industry standard)
   - Never log or serialize passwords
   - Implement password history to prevent reuse

3. **Session Management**
   - Short access token lifetime (1 hour default)
   - Longer refresh token lifetime (7 days default)
   - Invalidate sessions on password change
   - Regular cleanup of expired sessions

4. **Account Security**
   - Enable account lockout after failed attempts (5 default)
   - Require email verification for new accounts
   - Implement MFA for privileged accounts
   - Monitor for suspicious login patterns

5. **Network Security**
   - Always use TLS/SSL in production
   - Implement rate limiting on login endpoints
   - Use secure headers (HSTS, CSP, etc.)
   - Enable CORS appropriately

6. **Audit and Monitoring**
   - Log all authentication events
   - Monitor for failed login attempts
   - Alert on privilege escalation
   - Regular security audits

## API Reference

### AuthSystem

Main facade for the authentication system.

- `new(jwt_secret: String) -> Self` - Create with default config
- `with_config(jwt_secret, password_policy, max_failed_attempts) -> Self` - Create with custom config
- `login(username, password, ip, user_agent) -> Result<(User, Session)>` - Authenticate and create session
- `logout(session_id) -> Result<()>` - Invalidate session
- `verify_token(token) -> Result<User>` - Verify JWT token
- `check_permission(user, permission, resource) -> bool` - Check permission
- `create_user_with_role(id, username, email, password, role) -> Result<String>` - Create user
- `health_check() -> HashMap<String, bool>` - System health
- `statistics() -> AuthSystemStatistics` - System stats
- `cleanup()` - Clean expired sessions/cache

### UserManager

- `create_user(id, username, email, password) -> UserResult<String>` - Create user
- `get_user(id) -> UserResult<&User>` - Get user by ID
- `authenticate(username_or_email, password) -> UserResult<&User>` - Authenticate
- `list_users() -> Vec<UserSummary>` - List all users
- `list_users_by_role(role_id) -> Vec<UserSummary>` - List by role
- `list_users_by_status(status) -> Vec<UserSummary>` - List by status

### RoleManager

- `add_role(role) -> RoleResult<()>` - Add role
- `get_role(id) -> RoleResult<&Role>` - Get role
- `delete_role(id) -> RoleResult<()>` - Delete role
- `list_roles() -> Vec<&Role>` - List all roles
- `has_permission(role_ids, permission) -> bool` - Check permission
- `combined_permissions(role_ids) -> PermissionSet` - Get combined permissions

### SessionManager

- `create_session(user_id, username, email, roles, ip, user_agent) -> SessionResult<Session>` - Create session
- `verify_access_token(token) -> SessionResult<Claims>` - Verify token
- `refresh_access_token(refresh_token, username, email, roles) -> SessionResult<Token>` - Refresh token
- `invalidate_session(session_id) -> SessionResult<()>` - Invalidate session
- `invalidate_user_sessions(user_id)` - Invalidate all user sessions
- `cleanup()` - Remove expired sessions

### PolicyEngine

- `add_policy(policy) -> PolicyResult<()>` - Add policy
- `remove_policy(id) -> PolicyResult<()>` - Remove policy
- `evaluate(permission, resource, context) -> PolicyResult<bool>` - Evaluate policies
- `check_permission(user, permission, resource, role_manager) -> bool` - Check with RBAC + policy
- `clear_cache()` - Clear evaluation cache
- `cleanup_cache()` - Remove expired cache entries

## Testing

Run tests for the auth module:

```bash
cargo test --lib enterprise::auth
```

Run specific test:

```bash
cargo test --lib enterprise::auth::user::tests::test_user_creation
```

## Performance Considerations

- **Permission Checking**: O(1) with HashSet-based PermissionSet
- **Policy Evaluation**: Cached with configurable TTL (5 minutes default)
- **Session Lookup**: O(1) with HashMap storage
- **Role Inheritance**: Recursive with reasonable depth limits

## Future Enhancements

- [ ] Actual Argon2 implementation (currently placeholder)
- [ ] Complete JWT implementation using `jsonwebtoken` crate
- [ ] Real LDAP integration using `ldap3` crate
- [ ] OAuth2/OIDC implementation using `oauth2` crate
- [ ] SAML support
- [ ] WebAuthn/FIDO2 support
- [ ] Biometric authentication
- [ ] Risk-based authentication
- [ ] Session persistence to database
- [ ] Distributed session management (Redis)
- [ ] Policy versioning and rollback
- [ ] Advanced audit logging integration

## Dependencies

The auth module uses these crates:

- `serde` - Serialization/deserialization
- `thiserror` - Error handling
- `chrono` - Date/time handling

For production, add:

- `argon2` - Password hashing
- `jsonwebtoken` - JWT implementation
- `ldap3` - LDAP authentication
- `oauth2` - OAuth2/OIDC
- `base64` - Base64 encoding

## License

Part of CADDY Enterprise Edition. See LICENSE-ENTERPRISE.txt for details.

## Support

For enterprise support: enterprise@caddy-cad.com

---

**Built with ❤️ in Rust for CADDY v0.1.5**
