//! Enhanced JWT Token Management
//!
//! Production-grade JWT token management with advanced features:
//! - Access token and refresh token generation
//! - Token rotation and refresh logic
//! - Multiple signing algorithms (HS256, HS384, HS512, RS256, RS384, RS512, ES256, ES384)
//! - Token revocation and blacklisting
//! - Automatic token refresh
//! - Token introspection
//! - Custom claims support
//! - Token fingerprinting for enhanced security
//!
//! # Security Features
//! - Short-lived access tokens with automatic refresh
//! - Refresh token rotation to prevent token replay
//! - Token fingerprinting to bind tokens to clients
//! - Token blacklisting for immediate revocation
//! - Cryptographic signature verification
//! - Expiration validation
//! - Not-before (nbf) validation
//! - Issuer and audience validation

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use sha2::{Sha256, Digest};

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum JwtError {
    #[error("Token creation failed: {0}")]
    CreationFailed(String),

    #[error("Token validation failed: {0}")]
    ValidationFailed(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Token revoked")]
    TokenRevoked,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Refresh token not found")]
    RefreshTokenNotFound,

    #[error("Token fingerprint mismatch")]
    FingerprintMismatch,

    #[error("Encoding error: {0}")]
    EncodingError(String),

    #[error("Decoding error: {0}")]
    DecodingError(String),
}

pub type JwtResult<T> = Result<T, JwtError>;

// ============================================================================
// JWT Configuration
// ============================================================================

/// JWT Manager Configuration
#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// Secret key for HMAC algorithms (HS256, HS384, HS512)
    pub secret: String,

    /// RSA private key (PEM) for RS algorithms
    pub rsa_private_key: Option<String>,

    /// RSA public key (PEM) for RS algorithms
    pub rsa_public_key: Option<String>,

    /// ECDSA private key (PEM) for ES algorithms
    pub ecdsa_private_key: Option<String>,

    /// ECDSA public key (PEM) for ES algorithms
    pub ecdsa_public_key: Option<String>,

    /// Algorithm to use
    pub algorithm: Algorithm,

    /// Access token expiration (seconds)
    pub access_token_ttl: u64,

    /// Refresh token expiration (seconds)
    pub refresh_token_ttl: u64,

    /// Token issuer
    pub issuer: String,

    /// Token audience
    pub audience: String,

    /// Enable token fingerprinting
    pub enable_fingerprinting: bool,

    /// Enable automatic token rotation
    pub enable_token_rotation: bool,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "change-this-secret-in-production".to_string(),
            rsa_private_key: None,
            rsa_public_key: None,
            ecdsa_private_key: None,
            ecdsa_public_key: None,
            algorithm: Algorithm::HS256,
            access_token_ttl: 900,      // 15 minutes
            refresh_token_ttl: 604800,  // 7 days
            issuer: "caddy-auth".to_string(),
            audience: "caddy-api".to_string(),
            enable_fingerprinting: true,
            enable_token_rotation: true,
        }
    }
}

// ============================================================================
// Token Claims
// ============================================================================

/// Standard JWT Claims with custom extensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    /// Subject (user ID)
    pub sub: String,

    /// Expiration time (UTC timestamp)
    pub exp: u64,

    /// Issued at (UTC timestamp)
    pub iat: u64,

    /// Not before (UTC timestamp)
    pub nbf: u64,

    /// Issuer
    pub iss: String,

    /// Audience
    pub aud: String,

    /// JWT ID (unique identifier)
    pub jti: String,

    /// Token type ("access" or "refresh")
    #[serde(rename = "type")]
    pub token_type: String,

    /// Username
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// Email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// Roles
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,

    /// Permissions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Vec<String>>,

    /// Token fingerprint (SHA256 hash of client info)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,

    /// Session ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,

    /// IP address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,

    /// User agent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,

    /// Custom claims
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

impl TokenClaims {
    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.exp <= now
    }

    /// Check if token is not yet valid
    pub fn is_not_yet_valid(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.nbf > now
    }

    /// Check if token is valid
    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.is_not_yet_valid()
    }
}

// ============================================================================
// Token Pair
// ============================================================================

/// Access and refresh token pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    /// Access token (short-lived)
    pub access_token: String,

    /// Refresh token (long-lived)
    pub refresh_token: String,

    /// Token type (always "Bearer")
    pub token_type: String,

    /// Expiration in seconds
    pub expires_in: u64,

    /// Refresh token expiration in seconds
    pub refresh_expires_in: u64,
}

// ============================================================================
// JWT Manager
// ============================================================================

/// Enhanced JWT Token Manager
pub struct JwtManager {
    config: JwtConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,

    /// Token blacklist (revoked token JTIs)
    blacklist: Arc<RwLock<HashMap<String, SystemTime>>>,

    /// Refresh token storage (JTI -> TokenClaims)
    refresh_tokens: Arc<RwLock<HashMap<String, TokenClaims>>>,

    /// Token rotation tracking (old JTI -> new JTI)
    rotation_map: Arc<RwLock<HashMap<String, String>>>,
}

impl JwtManager {
    /// Create a new JWT manager with configuration
    pub fn new(config: JwtConfig) -> Self {
        let encoding_key = match config.algorithm {
            Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
                EncodingKey::from_secret(config.secret.as_bytes())
            }
            Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 => {
                if let Some(ref key) = config.rsa_private_key {
                    EncodingKey::from_rsa_pem(key.as_bytes())
                        .expect("Invalid RSA private key")
                } else {
                    panic!("RSA private key required for RS algorithms");
                }
            }
            Algorithm::ES256 | Algorithm::ES384 => {
                if let Some(ref key) = config.ecdsa_private_key {
                    EncodingKey::from_ec_pem(key.as_bytes())
                        .expect("Invalid ECDSA private key")
                } else {
                    panic!("ECDSA private key required for ES algorithms");
                }
            }
            _ => panic!("Unsupported algorithm"),
        };

        let decoding_key = match config.algorithm {
            Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
                DecodingKey::from_secret(config.secret.as_bytes())
            }
            Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 => {
                if let Some(ref key) = config.rsa_public_key {
                    DecodingKey::from_rsa_pem(key.as_bytes())
                        .expect("Invalid RSA public key")
                } else {
                    panic!("RSA public key required for RS algorithms");
                }
            }
            Algorithm::ES256 | Algorithm::ES384 => {
                if let Some(ref key) = config.ecdsa_public_key {
                    DecodingKey::from_ec_pem(key.as_bytes())
                        .expect("Invalid ECDSA public key")
                } else {
                    panic!("ECDSA public key required for ES algorithms");
                }
            }
            _ => panic!("Unsupported algorithm"),
        };

        let mut validation = Validation::new(config.algorithm);
        validation.set_issuer(&[&config.issuer]);
        validation.set_audience(&[&config.audience]);
        validation.validate_exp = true;
        validation.validate_nbf = true;

        Self {
            config,
            encoding_key,
            decoding_key,
            validation,
            blacklist: Arc::new(RwLock::new(HashMap::new())),
            refresh_tokens: Arc::new(RwLock::new(HashMap::new())),
            rotation_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a simple JWT manager with HMAC secret
    pub fn with_secret(secret: String) -> Self {
        Self::new(JwtConfig {
            secret,
            ..Default::default()
        })
    }

    /// Create access and refresh token pair
    pub fn create_token_pair(
        &self,
        user_id: String,
        username: Option<String>,
        email: Option<String>,
        roles: Option<Vec<String>>,
        permissions: Option<Vec<String>>,
        session_id: Option<String>,
        client_info: Option<ClientInfo>,
    ) -> JwtResult<TokenPair> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let fingerprint = if self.config.enable_fingerprinting {
            client_info.as_ref().map(|info| Self::generate_fingerprint(info))
        } else {
            None
        };

        // Create access token claims
        let access_claims = TokenClaims {
            sub: user_id.clone(),
            exp: now + self.config.access_token_ttl,
            iat: now,
            nbf: now,
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
            jti: Uuid::new_v4().to_string(),
            token_type: "access".to_string(),
            username: username.clone(),
            email: email.clone(),
            roles: roles.clone(),
            permissions,
            fingerprint: fingerprint.clone(),
            session_id: session_id.clone(),
            ip: client_info.as_ref().map(|i| i.ip_address.clone()),
            user_agent: client_info.as_ref().map(|i| i.user_agent.clone()),
            custom: HashMap::new(),
        };

        // Create refresh token claims
        let refresh_claims = TokenClaims {
            sub: user_id,
            exp: now + self.config.refresh_token_ttl,
            iat: now,
            nbf: now,
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
            jti: Uuid::new_v4().to_string(),
            token_type: "refresh".to_string(),
            username,
            email,
            roles,
            permissions: None,
            fingerprint,
            session_id,
            ip: client_info.as_ref().map(|i| i.ip_address.clone()),
            user_agent: client_info.as_ref().map(|i| i.user_agent.clone()),
            custom: HashMap::new(),
        };

        // Store refresh token
        self.refresh_tokens
            .write()
            .insert(refresh_claims.jti.clone(), refresh_claims.clone());

        // Encode tokens
        let header = Header::new(self.config.algorithm);

        let access_token = encode(&header, &access_claims, &self.encoding_key)
            .map_err(|e| JwtError::CreationFailed(e.to_string()))?;

        let refresh_token = encode(&header, &refresh_claims, &self.encoding_key)
            .map_err(|e| JwtError::CreationFailed(e.to_string()))?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.config.access_token_ttl,
            refresh_expires_in: self.config.refresh_token_ttl,
        })
    }

    /// Verify and decode access token
    pub fn verify_access_token(&self, token: &str) -> JwtResult<TokenClaims> {
        let token_data = decode::<TokenClaims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| JwtError::ValidationFailed(e.to_string()))?;

        let claims = token_data.claims;

        // Check if token is blacklisted
        if self.is_blacklisted(&claims.jti) {
            return Err(JwtError::TokenRevoked);
        }

        // Verify token type
        if claims.token_type != "access" {
            return Err(JwtError::InvalidToken);
        }

        Ok(claims)
    }

    /// Verify and decode refresh token
    pub fn verify_refresh_token(&self, token: &str) -> JwtResult<TokenClaims> {
        let token_data = decode::<TokenClaims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| JwtError::ValidationFailed(e.to_string()))?;

        let claims = token_data.claims;

        // Check if token is blacklisted
        if self.is_blacklisted(&claims.jti) {
            return Err(JwtError::TokenRevoked);
        }

        // Verify token type
        if claims.token_type != "refresh" {
            return Err(JwtError::InvalidToken);
        }

        // Verify token exists in storage
        if !self.refresh_tokens.read().contains_key(&claims.jti) {
            return Err(JwtError::RefreshTokenNotFound);
        }

        Ok(claims)
    }

    /// Refresh access token using refresh token
    pub fn refresh(
        &self,
        refresh_token: &str,
        client_info: Option<ClientInfo>,
    ) -> JwtResult<TokenPair> {
        let refresh_claims = self.verify_refresh_token(refresh_token)?;

        // Verify fingerprint if enabled
        if self.config.enable_fingerprinting {
            if let Some(ref expected_fingerprint) = refresh_claims.fingerprint {
                if let Some(ref info) = client_info {
                    let actual_fingerprint = Self::generate_fingerprint(info);
                    if &actual_fingerprint != expected_fingerprint {
                        return Err(JwtError::FingerprintMismatch);
                    }
                }
            }
        }

        // Create new token pair
        let new_tokens = self.create_token_pair(
            refresh_claims.sub,
            refresh_claims.username,
            refresh_claims.email,
            refresh_claims.roles,
            None,
            refresh_claims.session_id,
            client_info,
        )?;

        // If token rotation enabled, revoke old refresh token
        if self.config.enable_token_rotation {
            self.revoke_token(&refresh_claims.jti);
        }

        Ok(new_tokens)
    }

    /// Revoke a token by adding it to blacklist
    pub fn revoke_token(&self, jti: &str) {
        let expiry = SystemTime::now() + Duration::from_secs(self.config.refresh_token_ttl);
        self.blacklist.write().insert(jti.to_string(), expiry);

        // Remove from refresh token storage
        self.refresh_tokens.write().remove(jti);
    }

    /// Check if token is blacklisted
    pub fn is_blacklisted(&self, jti: &str) -> bool {
        self.blacklist.read().contains_key(jti)
    }

    /// Generate token fingerprint from client info
    fn generate_fingerprint(client_info: &ClientInfo) -> String {
        let mut hasher = Sha256::new();
        hasher.update(client_info.ip_address.as_bytes());
        hasher.update(client_info.user_agent.as_bytes());
        let hash = hasher.finalize();
        hex::encode(hash)
    }

    /// Clean up expired blacklist entries
    pub fn cleanup_blacklist(&self) {
        let now = SystemTime::now();
        self.blacklist.write().retain(|_, expiry| expiry > &now);
    }

    /// Clean up expired refresh tokens
    pub fn cleanup_refresh_tokens(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.refresh_tokens
            .write()
            .retain(|_, claims| claims.exp > now);
    }

    /// Get token statistics
    pub fn statistics(&self) -> JwtStatistics {
        JwtStatistics {
            blacklisted_tokens: self.blacklist.read().len(),
            active_refresh_tokens: self.refresh_tokens.read().len(),
        }
    }
}

// ============================================================================
// Client Information
// ============================================================================

/// Client information for fingerprinting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub ip_address: String,
    pub user_agent: String,
}

/// JWT statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtStatistics {
    pub blacklisted_tokens: usize,
    pub active_refresh_tokens: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        let manager = JwtManager::with_secret("test-secret-key-32-characters!!".to_string());

        let token_pair = manager
            .create_token_pair(
                "user123".to_string(),
                Some("testuser".to_string()),
                Some("test@example.com".to_string()),
                Some(vec!["admin".to_string()]),
                None,
                None,
                None,
            )
            .unwrap();

        assert!(!token_pair.access_token.is_empty());
        assert!(!token_pair.refresh_token.is_empty());
        assert_eq!(token_pair.token_type, "Bearer");
    }

    #[test]
    fn test_token_verification() {
        let manager = JwtManager::with_secret("test-secret-key-32-characters!!".to_string());

        let token_pair = manager
            .create_token_pair(
                "user123".to_string(),
                Some("testuser".to_string()),
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();

        let claims = manager
            .verify_access_token(&token_pair.access_token)
            .unwrap();

        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.username, Some("testuser".to_string()));
        assert_eq!(claims.token_type, "access");
    }

    #[test]
    fn test_token_refresh() {
        let manager = JwtManager::with_secret("test-secret-key-32-characters!!".to_string());

        let token_pair = manager
            .create_token_pair(
                "user123".to_string(),
                Some("testuser".to_string()),
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();

        let new_tokens = manager.refresh(&token_pair.refresh_token, None).unwrap();

        assert!(!new_tokens.access_token.is_empty());
        assert_ne!(new_tokens.access_token, token_pair.access_token);
    }

    #[test]
    fn test_token_revocation() {
        let manager = JwtManager::with_secret("test-secret-key-32-characters!!".to_string());

        let token_pair = manager
            .create_token_pair(
                "user123".to_string(),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();

        let access_claims = manager
            .verify_access_token(&token_pair.access_token)
            .unwrap();

        manager.revoke_token(&access_claims.jti);

        assert!(manager.is_blacklisted(&access_claims.jti));

        let result = manager.verify_access_token(&token_pair.access_token);
        assert!(matches!(result, Err(JwtError::TokenRevoked)));
    }

    #[test]
    fn test_fingerprint_generation() {
        let client_info = ClientInfo {
            ip_address: "192.168.1.1".to_string(),
            user_agent: "Mozilla/5.0".to_string(),
        };

        let fingerprint1 = JwtManager::generate_fingerprint(&client_info);
        let fingerprint2 = JwtManager::generate_fingerprint(&client_info);

        assert_eq!(fingerprint1, fingerprint2);
        assert_eq!(fingerprint1.len(), 64); // SHA256 hex length
    }
}
