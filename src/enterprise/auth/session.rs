//! Session management and JWT-based authentication for CADDY.
//!
//! This module provides:
//! - JWT token generation and validation
//! - Session storage and lifecycle management
//! - Token refresh mechanism
//! - Session invalidation and cleanup

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during session operations
#[derive(Error, Debug)]
pub enum SessionError {
    #[error("Session not found: {0}")]
    NotFound(String),

    #[error("Session expired")]
    Expired,

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Token generation error: {0}")]
    TokenGenerationError(String),

    #[error("Token verification error: {0}")]
    TokenVerificationError(String),

    #[error("Refresh token invalid or expired")]
    RefreshTokenInvalid,

    #[error("Session already invalidated")]
    AlreadyInvalidated,
}

/// Result type for session operations
pub type SessionResult<T> = Result<T, SessionError>;

/// JWT Claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,

    /// Username
    pub username: String,

    /// User email
    pub email: String,

    /// User roles
    pub roles: Vec<String>,

    /// Issued at (timestamp)
    pub iat: i64,

    /// Expiration time (timestamp)
    pub exp: i64,

    /// Session ID
    pub sid: String,

    /// Token type (access or refresh)
    pub token_type: String,

    /// Custom claims
    #[serde(flatten)]
    pub custom: HashMap<String, String>,
}

impl Claims {
    /// Create new access token claims
    pub fn new_access(
        user_id: String,
        username: String,
        email: String,
        roles: Vec<String>,
        session_id: String,
        valid_for: Duration,
    ) -> Self {
        let now = Utc::now();
        Self {
            sub: user_id,
            username,
            email,
            roles,
            iat: now.timestamp(),
            exp: (now + valid_for).timestamp(),
            sid: session_id,
            token_type: "access".to_string(),
            custom: HashMap::new(),
        }
    }

    /// Create new refresh token claims
    pub fn new_refresh(
        user_id: String,
        session_id: String,
        valid_for: Duration,
    ) -> Self {
        let now = Utc::now();
        Self {
            sub: user_id,
            username: String::new(),
            email: String::new(),
            roles: Vec::new(),
            iat: now.timestamp(),
            exp: (now + valid_for).timestamp(),
            sid: session_id,
            token_type: "refresh".to_string(),
            custom: HashMap::new(),
        }
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }

    /// Check if token is an access token
    pub fn is_access_token(&self) -> bool {
        self.token_type == "access"
    }

    /// Check if token is a refresh token
    pub fn is_refresh_token(&self) -> bool {
        self.token_type == "refresh"
    }

    /// Get expiration time as DateTime
    pub fn expiration(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.exp, 0).unwrap_or(Utc::now())
    }
}

/// JWT token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    /// The actual JWT token string
    pub token: String,

    /// Token type (Bearer)
    pub token_type: String,

    /// Expiration timestamp
    pub expires_at: DateTime<Utc>,
}

impl Token {
    /// Create a new token
    pub fn new(token: String, expires_at: DateTime<Utc>) -> Self {
        Self {
            token,
            token_type: "Bearer".to_string(),
            expires_at,
        }
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier
    pub id: String,

    /// User ID
    pub user_id: String,

    /// Access token
    pub access_token: Token,

    /// Refresh token
    pub refresh_token: Token,

    /// Session creation time
    pub created_at: DateTime<Utc>,

    /// Last activity time
    pub last_activity: DateTime<Utc>,

    /// IP address of the client
    pub ip_address: Option<String>,

    /// User agent string
    pub user_agent: Option<String>,

    /// Session metadata
    pub metadata: HashMap<String, String>,

    /// Whether the session has been invalidated
    pub invalidated: bool,
}

impl Session {
    /// Create a new session
    pub fn new(
        id: String,
        user_id: String,
        access_token: Token,
        refresh_token: Token,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            user_id,
            access_token,
            refresh_token,
            created_at: now,
            last_activity: now,
            ip_address,
            user_agent,
            metadata: HashMap::new(),
            invalidated: false,
        }
    }

    /// Check if session is valid (not expired and not invalidated)
    pub fn is_valid(&self) -> bool {
        !self.invalidated
            && !self.access_token.is_expired()
            && !self.refresh_token.is_expired()
    }

    /// Check if access token is expired but refresh token is still valid
    pub fn can_refresh(&self) -> bool {
        !self.invalidated
            && self.access_token.is_expired()
            && !self.refresh_token.is_expired()
    }

    /// Update last activity timestamp
    pub fn update_activity(&mut self) {
        self.last_activity = Utc::now();
    }

    /// Invalidate the session
    pub fn invalidate(&mut self) {
        self.invalidated = true;
    }

    /// Check if session is idle for too long
    pub fn is_idle(&self, max_idle_duration: Duration) -> bool {
        Utc::now() - self.last_activity > max_idle_duration
    }
}

/// JWT token manager
pub struct JwtManager {
    /// Secret key for signing tokens (in production, use a proper key management system)
    secret: String,

    /// Access token validity duration
    access_token_duration: Duration,

    /// Refresh token validity duration
    refresh_token_duration: Duration,
}

impl JwtManager {
    /// Create a new JWT manager
    pub fn new(secret: String) -> Self {
        Self {
            secret,
            access_token_duration: Duration::hours(1),
            refresh_token_duration: Duration::days(7),
        }
    }

    /// Create a new JWT manager with custom durations
    pub fn with_durations(
        secret: String,
        access_token_duration: Duration,
        refresh_token_duration: Duration,
    ) -> Self {
        Self {
            secret,
            access_token_duration,
            refresh_token_duration,
        }
    }

    /// Generate an access token
    pub fn generate_access_token(&self, claims: Claims) -> SessionResult<String> {
        // In production, use jsonwebtoken crate:
        // use jsonwebtoken::{encode, Header, EncodingKey};
        // encode(&Header::default(), &claims, &EncodingKey::from_secret(self.secret.as_ref()))
        //     .map_err(|e| SessionError::TokenGenerationError(e.to_string()))

        // Placeholder implementation
        let claims_json = serde_json::to_string(&claims)
            .map_err(|e| SessionError::TokenGenerationError(e.to_string()))?;

        let encoded = base64_encode(&claims_json);
        Ok(format!("jwt.access.{}.{}", encoded, self.sign(&encoded)))
    }

    /// Generate a refresh token
    pub fn generate_refresh_token(&self, claims: Claims) -> SessionResult<String> {
        // Similar to access token but with different claims
        let claims_json = serde_json::to_string(&claims)
            .map_err(|e| SessionError::TokenGenerationError(e.to_string()))?;

        let encoded = base64_encode(&claims_json);
        Ok(format!("jwt.refresh.{}.{}", encoded, self.sign(&encoded)))
    }

    /// Verify and decode a token
    pub fn verify_token(&self, token: &str) -> SessionResult<Claims> {
        // In production, use jsonwebtoken crate:
        // use jsonwebtoken::{decode, DecodingKey, Validation};
        // let token_data = decode::<Claims>(
        //     token,
        //     &DecodingKey::from_secret(self.secret.as_ref()),
        //     &Validation::default()
        // ).map_err(|e| SessionError::TokenVerificationError(e.to_string()))?;
        // Ok(token_data.claims)

        // Placeholder implementation
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 4 {
            return Err(SessionError::InvalidToken("Invalid token format".to_string()));
        }

        let claims_encoded = parts[2];
        let signature = parts[3];

        // Verify signature
        if signature != self.sign(claims_encoded) {
            return Err(SessionError::InvalidToken("Invalid signature".to_string()));
        }

        let claims_json = base64_decode(claims_encoded)
            .map_err(|e| SessionError::TokenVerificationError(e))?;

        let claims: Claims = serde_json::from_str(&claims_json)
            .map_err(|e| SessionError::TokenVerificationError(e.to_string()))?;

        // Check expiration
        if claims.is_expired() {
            return Err(SessionError::Expired);
        }

        Ok(claims)
    }

    /// Sign data (placeholder for HMAC)
    fn sign(&self, data: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        format!("{}{}", data, self.secret).hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Get access token duration
    pub fn access_token_duration(&self) -> Duration {
        self.access_token_duration
    }

    /// Get refresh token duration
    pub fn refresh_token_duration(&self) -> Duration {
        self.refresh_token_duration
    }
}

/// Session manager for session lifecycle management
pub struct SessionManager {
    sessions: HashMap<String, Session>,
    jwt_manager: JwtManager,
    max_idle_duration: Duration,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(jwt_manager: JwtManager) -> Self {
        Self {
            sessions: HashMap::new(),
            jwt_manager,
            max_idle_duration: Duration::hours(24),
        }
    }

    /// Create a new session for a user
    pub fn create_session(
        &mut self,
        user_id: String,
        username: String,
        email: String,
        roles: Vec<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> SessionResult<Session> {
        let session_id = generate_session_id();

        // Create access token claims
        let access_claims = Claims::new_access(
            user_id.clone(),
            username,
            email,
            roles,
            session_id.clone(),
            self.jwt_manager.access_token_duration(),
        );

        // Create refresh token claims
        let refresh_claims = Claims::new_refresh(
            user_id.clone(),
            session_id.clone(),
            self.jwt_manager.refresh_token_duration(),
        );

        // Generate tokens
        let access_token_str = self.jwt_manager.generate_access_token(access_claims.clone())?;
        let refresh_token_str = self.jwt_manager.generate_refresh_token(refresh_claims.clone())?;

        let access_token = Token::new(access_token_str, access_claims.expiration());
        let refresh_token = Token::new(refresh_token_str, refresh_claims.expiration());

        // Create session
        let session = Session::new(
            session_id.clone(),
            user_id,
            access_token,
            refresh_token,
            ip_address,
            user_agent,
        );

        self.sessions.insert(session_id, session.clone());

        Ok(session)
    }

    /// Get a session by ID
    pub fn get_session(&self, session_id: &str) -> SessionResult<&Session> {
        self.sessions
            .get(session_id)
            .ok_or_else(|| SessionError::NotFound(session_id.to_string()))
    }

    /// Get a mutable session by ID
    pub fn get_session_mut(&mut self, session_id: &str) -> SessionResult<&mut Session> {
        self.sessions
            .get_mut(session_id)
            .ok_or_else(|| SessionError::NotFound(session_id.to_string()))
    }

    /// Verify an access token and return the claims
    pub fn verify_access_token(&mut self, token: &str) -> SessionResult<Claims> {
        let claims = self.jwt_manager.verify_token(token)?;

        if !claims.is_access_token() {
            return Err(SessionError::InvalidToken("Not an access token".to_string()));
        }

        // Update session activity
        if let Ok(session) = self.get_session_mut(&claims.sid) {
            if session.invalidated {
                return Err(SessionError::AlreadyInvalidated);
            }
            session.update_activity();
        }

        Ok(claims)
    }

    /// Refresh an access token using a refresh token
    pub fn refresh_access_token(
        &mut self,
        refresh_token: &str,
        username: String,
        email: String,
        roles: Vec<String>,
    ) -> SessionResult<Token> {
        let claims = self.jwt_manager.verify_token(refresh_token)?;

        if !claims.is_refresh_token() {
            return Err(SessionError::InvalidToken("Not a refresh token".to_string()));
        }

        // Check session validity first
        {
            let session = self.get_session(&claims.sid)?;

            if session.invalidated {
                return Err(SessionError::AlreadyInvalidated);
            }

            if !session.can_refresh() {
                return Err(SessionError::RefreshTokenInvalid);
            }
        }

        // Generate new access token
        let access_claims = Claims::new_access(
            claims.sub.clone(),
            username,
            email,
            roles,
            claims.sid.clone(),
            self.jwt_manager.access_token_duration(),
        );

        let access_token_str = self.jwt_manager.generate_access_token(access_claims.clone())?;
        let access_token = Token::new(access_token_str, access_claims.expiration());

        // Update session
        let session = self.get_session_mut(&claims.sid)?;
        session.access_token = access_token.clone();
        session.update_activity();

        Ok(access_token)
    }

    /// Invalidate a session
    pub fn invalidate_session(&mut self, session_id: &str) -> SessionResult<()> {
        let session = self.get_session_mut(session_id)?;
        session.invalidate();
        Ok(())
    }

    /// Invalidate all sessions for a user
    pub fn invalidate_user_sessions(&mut self, user_id: &str) {
        for session in self.sessions.values_mut() {
            if session.user_id == user_id {
                session.invalidate();
            }
        }
    }

    /// Clean up expired and idle sessions
    pub fn cleanup(&mut self) {
        self.sessions.retain(|_, session| {
            !session.refresh_token.is_expired()
                && !session.is_idle(self.max_idle_duration)
        });
    }

    /// Get all active sessions for a user
    pub fn get_user_sessions(&self, user_id: &str) -> Vec<&Session> {
        self.sessions
            .values()
            .filter(|s| s.user_id == user_id && s.is_valid())
            .collect()
    }

    /// Get session count
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Get active session count
    pub fn active_session_count(&self) -> usize {
        self.sessions.values().filter(|s| s.is_valid()).count()
    }
}

/// Generate a unique session ID
fn generate_session_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    format!("sess_{:x}", timestamp)
}

/// Base64 encode (placeholder)
fn base64_encode(data: &str) -> String {
    // In production, use base64 crate
    // base64::encode(data)
    data.chars().map(|c| c as u8).map(|b| format!("{:02x}", b)).collect()
}

/// Base64 decode (placeholder)
fn base64_decode(data: &str) -> Result<String, String> {
    // In production, use base64 crate
    // base64::decode(data).map(|v| String::from_utf8(v).unwrap())
    let bytes: Result<Vec<u8>, _> = (0..data.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&data[i..i + 2], 16))
        .collect();

    bytes
        .map_err(|e| e.to_string())
        .and_then(|v| String::from_utf8(v).map_err(|e| e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claims_creation() {
        let claims = Claims::new_access(
            "user123".to_string(),
            "testuser".to_string(),
            "test@example.com".to_string(),
            vec!["designer".to_string()],
            "session123".to_string(),
            Duration::hours(1),
        );

        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.username, "testuser");
        assert!(claims.is_access_token());
        assert!(!claims.is_refresh_token());
    }

    #[test]
    fn test_token_expiration() {
        let mut claims = Claims::new_access(
            "user123".to_string(),
            "testuser".to_string(),
            "test@example.com".to_string(),
            vec![],
            "session123".to_string(),
            Duration::hours(1),
        );

        assert!(!claims.is_expired());

        // Manually set expiration to past
        claims.exp = (Utc::now() - Duration::hours(1)).timestamp();
        assert!(claims.is_expired());
    }

    #[test]
    fn test_session_manager() {
        let jwt_manager = JwtManager::new("test_secret".to_string());
        let mut session_manager = SessionManager::new(jwt_manager);

        let session = session_manager
            .create_session(
                "user123".to_string(),
                "testuser".to_string(),
                "test@example.com".to_string(),
                vec!["designer".to_string()],
                Some("127.0.0.1".to_string()),
                Some("TestAgent/1.0".to_string()),
            )
            .unwrap();

        assert!(session.is_valid());
        assert_eq!(session.user_id, "user123");
    }

    #[test]
    fn test_session_invalidation() {
        let jwt_manager = JwtManager::new("test_secret".to_string());
        let mut session_manager = SessionManager::new(jwt_manager);

        let session = session_manager
            .create_session(
                "user123".to_string(),
                "testuser".to_string(),
                "test@example.com".to_string(),
                vec![],
                None,
                None,
            )
            .unwrap();

        let session_id = session.id.clone();
        assert!(session_manager.invalidate_session(&session_id).is_ok());

        let session = session_manager.get_session(&session_id).unwrap();
        assert!(!session.is_valid());
    }

    #[test]
    fn test_token_verification() {
        let jwt_manager = JwtManager::new("test_secret".to_string());

        let claims = Claims::new_access(
            "user123".to_string(),
            "testuser".to_string(),
            "test@example.com".to_string(),
            vec!["designer".to_string()],
            "session123".to_string(),
            Duration::hours(1),
        );

        let token = jwt_manager.generate_access_token(claims.clone()).unwrap();
        let verified = jwt_manager.verify_token(&token).unwrap();

        assert_eq!(verified.sub, claims.sub);
        assert_eq!(verified.username, claims.username);
    }
}
