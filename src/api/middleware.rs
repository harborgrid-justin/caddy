//! # API Middleware
//!
//! This module provides comprehensive middleware for the CADDY REST API including:
//!
//! - JWT-based authentication with enterprise auth integration
//! - Distributed rate limiting with Redis support
//! - Structured request/response logging with tracing
//! - CORS configuration for cross-origin requests
//! - Request validation and sanitization
//! - Request ID tracking for distributed tracing
//! - Performance metrics collection
//!
//! # Examples
//!
//! ```rust,ignore
//! use caddy::api::middleware::*;
//! use axum::Router;
//!
//! let app = Router::new()
//!     .route("/api/v1/scans", get(list_scans))
//!     .layer(AuthMiddleware::new(auth_config))
//!     .layer(RateLimitMiddleware::new(rate_limit_config))
//!     .layer(RequestLoggingMiddleware::new())
//!     .layer(cors_layer());
//! ```

use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

use crate::enterprise::auth::{JwtManager, TokenClaims, User};
use crate::enterprise::ratelimit::{
    QuotaIdentifier, QuotaLimits, QuotaPeriod, RateLimiter, RateLimiterConfig,
};
use super::responses::ApiError;

// ============================================================================
// Request ID Middleware
// ============================================================================

/// Request ID header name
pub const REQUEST_ID_HEADER: &str = "X-Request-ID";

/// Add request ID to all requests
pub async fn request_id_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    // Get or generate request ID
    let request_id = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Store request ID in extensions for access by handlers
    request.extensions_mut().insert(request_id.clone());

    // Call next middleware/handler
    let mut response = next.run(request).await;

    // Add request ID to response headers
    if let Ok(header_value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert(REQUEST_ID_HEADER, header_value);
    }

    response
}

// ============================================================================
// Authentication Middleware
// ============================================================================

/// Authentication configuration
#[derive(Clone)]
pub struct AuthConfig {
    /// JWT manager for token validation
    pub jwt_manager: Arc<JwtManager>,

    /// Skip authentication for these paths
    pub excluded_paths: Vec<String>,

    /// Optional API key header name
    pub api_key_header: Option<String>,
}

impl AuthConfig {
    /// Create new auth config
    pub fn new(jwt_manager: Arc<JwtManager>) -> Self {
        Self {
            jwt_manager,
            excluded_paths: vec![
                "/health".to_string(),
                "/api/v1/health".to_string(),
                "/api/docs".to_string(),
            ],
            api_key_header: Some("X-API-Key".to_string()),
        }
    }

    /// Add excluded path
    pub fn exclude_path(mut self, path: String) -> Self {
        self.excluded_paths.push(path);
        self
    }

    /// Check if path is excluded
    fn is_excluded(&self, path: &str) -> bool {
        self.excluded_paths.iter().any(|p| path.starts_with(p))
    }
}

/// Authentication middleware
pub async fn auth_middleware(
    State(config): State<Arc<AuthConfig>>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Skip authentication for excluded paths
    let path = request.uri().path();
    if config.is_excluded(path) {
        return Ok(next.run(request).await);
    }

    // Extract token from Authorization header
    let headers = request.headers();
    let token = extract_bearer_token(headers)
        .ok_or_else(|| {
            ApiError::unauthorized("Missing or invalid Authorization header")
        })?;

    // Verify JWT token
    let claims = config
        .jwt_manager
        .verify(&token)
        .map_err(|e| {
            ApiError::unauthorized(format!("Invalid token: {}", e))
        })?;

    // Store claims in request extensions
    request.extensions_mut().insert(claims.clone());

    // Store user context
    let user_context = UserContext {
        user_id: claims.sub.clone(),
        username: claims.username.clone(),
        email: claims.email.clone(),
        roles: claims.roles.clone(),
    };
    request.extensions_mut().insert(user_context);

    Ok(next.run(request).await)
}

/// Extract Bearer token from Authorization header
fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            if s.starts_with("Bearer ") {
                Some(s[7..].to_string())
            } else {
                None
            }
        })
}

/// User context stored in request extensions
#[derive(Clone, Debug)]
pub struct UserContext {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
}

impl UserContext {
    /// Check if user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r.eq_ignore_ascii_case(role))
    }

    /// Check if user has any of the specified roles
    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.iter().any(|&role| self.has_role(role))
    }
}

// ============================================================================
// Rate Limiting Middleware
// ============================================================================

/// Rate limit configuration
#[derive(Clone)]
pub struct RateLimitConfig {
    /// Rate limiter instance
    pub limiter: Arc<RateLimiter>,

    /// Default operation name
    pub default_operation: String,

    /// Extract identifier from request
    pub identifier_extractor: Arc<dyn Fn(&Request) -> QuotaIdentifier + Send + Sync>,
}

impl RateLimitConfig {
    /// Create new rate limit config
    pub fn new(limiter: Arc<RateLimiter>) -> Self {
        let identifier_extractor = Arc::new(|req: &Request| {
            // Try to get user from extensions
            if let Some(user_ctx) = req.extensions().get::<UserContext>() {
                return QuotaIdentifier::User(user_ctx.user_id.clone());
            }

            // Fall back to IP address
            if let Some(ip) = req
                .headers()
                .get("X-Forwarded-For")
                .and_then(|v| v.to_str().ok())
            {
                return QuotaIdentifier::IpAddress(ip.to_string());
            }

            // Default identifier
            QuotaIdentifier::User("anonymous".to_string())
        });

        Self {
            limiter,
            default_operation: "api_request".to_string(),
            identifier_extractor,
        }
    }

    /// Set default operation
    pub fn with_operation(mut self, operation: String) -> Self {
        self.default_operation = operation;
        self
    }
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    State(config): State<Arc<RateLimitConfig>>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Extract identifier
    let identifier = (config.identifier_extractor)(&request);

    // Check rate limit
    let result = config
        .limiter
        .check(&identifier, &config.default_operation, 1, 0)
        .await
        .map_err(|e| {
            ApiError::internal_error(format!("Rate limit check failed: {}", e))
        })?;

    // If rate limited, return error
    if !result.is_allowed() {
        let retry_after = result
            .retry_after()
            .unwrap_or(Duration::from_secs(60))
            .as_secs();

        let mut error = ApiError::rate_limit_exceeded(retry_after);

        // Add rate limit headers if available
        if let Some(request_id) = request.extensions().get::<String>() {
            error = error.with_request_id(request_id.clone());
        }

        return Err(error);
    }

    // Add rate limit headers to response
    let mut response = next.run(request).await;

    if let Some(headers) = result.headers {
        for (key, value) in headers {
            if let Ok(header_value) = HeaderValue::from_str(&value) {
                response.headers_mut().insert(
                    axum::http::HeaderName::from_bytes(key.as_bytes()).unwrap(),
                    header_value,
                );
            }
        }
    }

    Ok(response)
}

// ============================================================================
// Request Logging Middleware
// ============================================================================

/// Request logging middleware with performance tracking
pub async fn request_logging_middleware(
    request: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let request_id = request
        .extensions()
        .get::<String>()
        .cloned()
        .unwrap_or_else(|| "unknown".to_string());

    // Log request
    tracing::info!(
        request_id = %request_id,
        method = %method,
        uri = %uri,
        "Incoming request"
    );

    // Process request
    let response = next.run(request).await;

    // Calculate duration
    let duration = start.elapsed();
    let status = response.status();

    // Log response
    tracing::info!(
        request_id = %request_id,
        method = %method,
        uri = %uri,
        status = %status.as_u16(),
        duration_ms = %duration.as_millis(),
        "Request completed"
    );

    // Add performance header
    let mut response = response;
    if let Ok(duration_str) = HeaderValue::from_str(&duration.as_millis().to_string()) {
        response.headers_mut().insert("X-Response-Time", duration_str);
    }

    response
}

// ============================================================================
// CORS Middleware
// ============================================================================

/// Create CORS layer with common configuration
pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            HeaderValue::from_static("x-api-key"),
            HeaderValue::from_static("x-request-id"),
        ])
        .expose_headers([
            header::CONTENT_TYPE,
            HeaderValue::from_static("x-request-id"),
            HeaderValue::from_static("x-rate-limit-limit"),
            HeaderValue::from_static("x-rate-limit-remaining"),
            HeaderValue::from_static("x-rate-limit-reset"),
            HeaderValue::from_static("x-response-time"),
        ])
        .max_age(Duration::from_secs(3600))
}

/// Create production CORS layer with specific origins
pub fn cors_layer_with_origins(origins: Vec<String>) -> CorsLayer {
    let allowed_origins: Vec<HeaderValue> = origins
        .into_iter()
        .filter_map(|origin| HeaderValue::from_str(&origin).ok())
        .collect();

    CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            HeaderValue::from_static("x-api-key"),
            HeaderValue::from_static("x-request-id"),
        ])
        .expose_headers([
            header::CONTENT_TYPE,
            HeaderValue::from_static("x-request-id"),
            HeaderValue::from_static("x-rate-limit-limit"),
            HeaderValue::from_static("x-rate-limit-remaining"),
            HeaderValue::from_static("x-rate-limit-reset"),
            HeaderValue::from_static("x-response-time"),
        ])
        .max_age(Duration::from_secs(3600))
        .allow_credentials(true)
}

// ============================================================================
// Request Validation Middleware
// ============================================================================

/// Content type validation
pub async fn content_type_validation_middleware(
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Only validate content type for requests with body
    if matches!(
        *request.method(),
        Method::POST | Method::PUT | Method::PATCH
    ) {
        let content_type = request
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        // Require JSON content type
        if !content_type.starts_with("application/json") {
            return Err(ApiError::bad_request(
                "Content-Type must be application/json",
            ));
        }
    }

    Ok(next.run(request).await)
}

/// Request size limit middleware
pub async fn request_size_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    const MAX_REQUEST_SIZE: u64 = 10 * 1024 * 1024; // 10 MB

    if let Some(content_length) = request.headers().get(header::CONTENT_LENGTH) {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<u64>() {
                if length > MAX_REQUEST_SIZE {
                    return Err(ApiError::bad_request(format!(
                        "Request size {} exceeds maximum allowed size of {} bytes",
                        length, MAX_REQUEST_SIZE
                    )));
                }
            }
        }
    }

    Ok(next.run(request).await)
}

// ============================================================================
// Security Headers Middleware
// ============================================================================

/// Add security headers to all responses
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // Add security headers
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        "X-Frame-Options",
        HeaderValue::from_static("DENY"),
    );
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );
    headers.insert(
        "Strict-Transport-Security",
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );
    headers.insert(
        "Content-Security-Policy",
        HeaderValue::from_static("default-src 'self'"),
    );
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        "Permissions-Policy",
        HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
    );

    response
}

// ============================================================================
// Middleware Stack Builder
// ============================================================================

/// Build complete middleware stack
pub fn build_middleware_stack(
    _auth_config: Option<Arc<AuthConfig>>,
    _rate_limit_config: Option<Arc<RateLimitConfig>>,
) -> impl Clone {
    use axum::middleware::from_fn;

    let mut stack = ServiceBuilder::new()
        .layer(from_fn(request_id_middleware))
        .layer(from_fn(security_headers_middleware))
        .layer(from_fn(request_logging_middleware))
        .layer(from_fn(content_type_validation_middleware))
        .layer(from_fn(request_size_limit_middleware));

    // Note: Auth and rate limit middlewares need State, so they should be added
    // directly to the router with .layer(from_fn_with_state(...))

    stack.into_inner()
}

// ============================================================================
// Permission Check Middleware
// ============================================================================

/// Require specific role
pub fn require_role(required_role: &'static str) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, ApiError>> + Send>> + Clone {
    move |request: Request, next: Next| {
        Box::pin(async move {
            let user_ctx = request
                .extensions()
                .get::<UserContext>()
                .ok_or_else(|| ApiError::unauthorized("User context not found"))?;

            if !user_ctx.has_role(required_role) {
                return Err(ApiError::forbidden(format!(
                    "Required role: {}",
                    required_role
                )));
            }

            Ok(next.run(request).await)
        })
    }
}

/// Require any of the specified roles
pub fn require_any_role(required_roles: &'static [&'static str]) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, ApiError>> + Send>> + Clone {
    move |request: Request, next: Next| {
        Box::pin(async move {
            let user_ctx = request
                .extensions()
                .get::<UserContext>()
                .ok_or_else(|| ApiError::unauthorized("User context not found"))?;

            if !user_ctx.has_any_role(required_roles) {
                return Err(ApiError::forbidden(format!(
                    "Required roles: {}",
                    required_roles.join(", ")
                )));
            }

            Ok(next.run(request).await)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_context_roles() {
        let ctx = UserContext {
            user_id: "user1".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            roles: vec!["admin".to_string(), "designer".to_string()],
        };

        assert!(ctx.has_role("admin"));
        assert!(ctx.has_role("ADMIN")); // Case insensitive
        assert!(ctx.has_role("designer"));
        assert!(!ctx.has_role("viewer"));

        assert!(ctx.has_any_role(&["admin", "viewer"]));
        assert!(!ctx.has_any_role(&["viewer", "manager"]));
    }

    #[test]
    fn test_extract_bearer_token() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"),
        );

        let token = extract_bearer_token(&headers);
        assert_eq!(
            token,
            Some("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9".to_string())
        );
    }

    #[test]
    fn test_auth_config_excluded_paths() {
        let jwt_manager = Arc::new(JwtManager::new(
            "test_secret_key_12345678".to_string(),
        ));
        let config = AuthConfig::new(jwt_manager);

        assert!(config.is_excluded("/health"));
        assert!(config.is_excluded("/api/v1/health"));
        assert!(config.is_excluded("/api/docs"));
        assert!(!config.is_excluded("/api/v1/scans"));
    }
}
