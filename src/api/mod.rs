//! # CADDY REST API v0.3.0
//!
//! Enterprise-grade REST API for accessibility scanning and compliance management.
//!
//! ## Features
//!
//! - **REST API Routes**: Complete REST endpoints for all operations
//! - **API Gateway**: Circuit breaker, retry logic, and load balancing
//! - **Middleware**: Authentication, rate limiting, logging, CORS
//! - **Standardized Responses**: HAL, JSON:API, and RFC 7807 support
//! - **Webhook System**: Event-driven integrations with retry and verification
//! - **Request Handlers**: Comprehensive handlers for all resources
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use caddy::api::*;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Configure application
//!     let app_config = AppConfig::default();
//!     let app_state = Arc::new(AppState {
//!         db_pool: Arc::new(()),
//!         config: Arc::new(app_config),
//!     });
//!
//!     // Configure authentication
//!     let jwt_manager = Arc::new(JwtManager::new("your-secret-key".to_string()));
//!     let auth_config = Arc::new(AuthConfig::new(jwt_manager));
//!
//!     // Configure rate limiting
//!     let rate_limiter = Arc::new(RateLimiter::new(RateLimiterConfig::default()));
//!     let rate_limit_config = Arc::new(RateLimitConfig::new(rate_limiter));
//!
//!     // Create router
//!     let app = create_app_router(app_state, auth_config, rate_limit_config);
//!
//!     // Start server
//!     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
//!         .await
//!         .unwrap();
//!
//!     println!("API server listening on http://0.0.0.0:3000");
//!     axum::serve(listener, app).await.unwrap();
//! }
//! ```
//!
//! ## API Endpoints
//!
//! ### Scans
//! - `POST /api/v1/scans` - Create new scan
//! - `GET /api/v1/scans` - List scans
//! - `GET /api/v1/scans/:id` - Get scan details
//! - `DELETE /api/v1/scans/:id` - Delete scan
//! - `POST /api/v1/scans/:id/cancel` - Cancel scan
//! - `POST /api/v1/scans/:id/retry` - Retry failed scan
//!
//! ### Issues
//! - `GET /api/v1/issues` - List issues
//! - `GET /api/v1/issues/:id` - Get issue details
//! - `PATCH /api/v1/issues/:id` - Update issue
//! - `POST /api/v1/issues/bulk/update` - Bulk update issues
//!
//! ### Reports
//! - `GET /api/v1/reports/scan/:scan_id` - Generate scan report
//! - `GET /api/v1/reports` - List reports
//! - `GET /api/v1/reports/:id/download` - Download report
//!
//! ### Sites
//! - `POST /api/v1/sites` - Create site
//! - `GET /api/v1/sites` - List sites
//! - `GET /api/v1/sites/:id` - Get site details
//! - `PUT /api/v1/sites/:id` - Update site
//! - `DELETE /api/v1/sites/:id` - Delete site
//!
//! ### Settings
//! - `GET /api/v1/settings` - Get settings
//! - `PUT /api/v1/settings` - Update settings
//!
//! ### Webhooks
//! - `POST /api/v1/webhooks` - Create webhook
//! - `GET /api/v1/webhooks` - List webhooks
//! - `GET /api/v1/webhooks/:id` - Get webhook details
//! - `PUT /api/v1/webhooks/:id` - Update webhook
//! - `DELETE /api/v1/webhooks/:id` - Delete webhook
//! - `POST /api/v1/webhooks/:id/test` - Test webhook
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                     API Gateway                          │
//! │  - Request Routing                                       │
//! │  - Circuit Breaker                                       │
//! │  - Retry Logic                                          │
//! │  - Load Balancing                                       │
//! └─────────────────┬───────────────────────────────────────┘
//!                   │
//! ┌─────────────────┴───────────────────────────────────────┐
//! │                   Middleware Stack                       │
//! │  - Request ID                                           │
//! │  - Authentication (JWT)                                 │
//! │  - Rate Limiting                                        │
//! │  - Logging                                             │
//! │  - CORS                                                │
//! │  - Security Headers                                     │
//! └─────────────────┬───────────────────────────────────────┘
//!                   │
//! ┌─────────────────┴───────────────────────────────────────┐
//! │                    Route Handlers                        │
//! │  - Scans                                               │
//! │  - Issues                                              │
//! │  - Reports                                             │
//! │  - Sites                                               │
//! │  - Settings                                            │
//! │  - Webhooks                                            │
//! └─────────────────┬───────────────────────────────────────┘
//!                   │
//! ┌─────────────────┴───────────────────────────────────────┐
//! │              Standardized Responses                      │
//! │  - Success/Error formats                                │
//! │  - Pagination                                           │
//! │  - HAL links                                            │
//! │  - JSON:API                                             │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Security
//!
//! - JWT-based authentication with enterprise auth integration
//! - Rate limiting with distributed Redis support
//! - CORS configuration for cross-origin requests
//! - Security headers (CSP, HSTS, X-Frame-Options, etc.)
//! - Request validation and sanitization
//! - Webhook signature verification (HMAC-SHA256)
//!
//! ## Error Handling
//!
//! All errors follow RFC 7807 Problem Details specification:
//!
//! ```json
//! {
//!   "type": "https://api.caddy.io/errors/NOT_FOUND",
//!   "title": "Resource Not Found",
//!   "status": 404,
//!   "detail": "Scan not found",
//!   "instance": "/api/v1/scans/123",
//!   "code": "NOT_FOUND",
//!   "timestamp": "2025-12-29T12:00:00Z",
//!   "requestId": "req-123"
//! }
//! ```
//!
//! ## Rate Limiting
//!
//! Rate limit information is included in response headers:
//!
//! - `X-RateLimit-Limit`: Maximum requests allowed
//! - `X-RateLimit-Remaining`: Remaining requests
//! - `X-RateLimit-Reset`: Unix timestamp when limit resets
//! - `Retry-After`: Seconds to wait before retry (when rate limited)
//!
//! ## Pagination
//!
//! List endpoints support both offset and cursor-based pagination:
//!
//! **Offset-based:**
//! ```
//! GET /api/v1/scans?page=2&per_page=20
//! ```
//!
//! **Cursor-based:**
//! ```
//! GET /api/v1/scans?cursor=eyJpZCI6MTIzfQ==
//! ```
//!
//! ## Webhooks
//!
//! Webhooks are signed with HMAC-SHA256. Verify signatures:
//!
//! ```rust,ignore
//! use caddy::api::webhooks::verify_signature;
//!
//! let payload = r#"{"event":"scan_completed","data":{...}}"#;
//! let signature = "sha256=abc123...";
//! let secret = "your-webhook-secret";
//!
//! if verify_signature(secret, payload, signature) {
//!     // Process webhook
//! }
//! ```
//!
//! ## OpenAPI Documentation
//!
//! OpenAPI/Swagger specification available at:
//! - `/api/docs` - Interactive documentation
//! - `/openapi.json` - OpenAPI 3.0 specification

// ============================================================================
// Module Declarations
// ============================================================================

/// Standardized API response types
pub mod responses;

/// API middleware (auth, rate limiting, logging, CORS)
pub mod middleware;

/// Request handlers for all endpoints
pub mod handlers;

/// REST API route definitions
pub mod routes;

/// API gateway with circuit breaker and retry logic
pub mod gateway;

/// Webhook system with event dispatching
pub mod webhooks;

// ============================================================================
// Re-exports for Convenience
// ============================================================================

// Response types
pub use responses::{
    ApiError, ApiResponse, ComponentHealth, FieldError, HealthResponse, HealthStatus, Link, Links,
    PaginatedResponse, PaginationLinks, PaginationMeta, RateLimitMeta, ResponseMeta,
};

// Middleware types and functions
pub use middleware::{
    auth_middleware, cors_layer, cors_layer_with_origins, rate_limit_middleware,
    request_id_middleware, request_logging_middleware, require_any_role, require_role,
    security_headers_middleware, AuthConfig, RateLimitConfig, UserContext,
};

// Handler types
pub use handlers::{
    AppConfig, AppState, CreateScanRequest, CreateSiteRequest, Issue, IssueStatus, IssueSeverity,
    ReportFormat, ScanResponse, ScanStatus, Settings, Site, WcagLevel,
};

// Route functions
pub use routes::{
    create_app_router, create_public_router, create_v1_router, create_versioned_router, ApiVersion,
};

// Gateway types
pub use gateway::{
    ApiGateway, CircuitBreaker, CircuitBreakerConfig, CircuitState, GatewayConfig, GatewayError,
    GatewayRequest, GatewayResponse, LoadBalancingStrategy, RetryConfig as GatewayRetryConfig,
};

// Webhook types
pub use webhooks::{
    verify_signature, DeliveryStatus, EventType, Webhook, WebhookDelivery, WebhookError,
    WebhookEvent, WebhookManager, WebhookStats,
};

// ============================================================================
// Version Information
// ============================================================================

/// API version
pub const API_VERSION: &str = "v0.3.0";

/// API major version
pub const API_MAJOR_VERSION: u32 = 0;

/// API minor version
pub const API_MINOR_VERSION: u32 = 3;

/// API patch version
pub const API_PATCH_VERSION: u32 = 0;

// ============================================================================
// Server Builder
// ============================================================================

use std::sync::Arc;
use axum::Router;

/// API server builder for easy setup
pub struct ApiServerBuilder {
    app_state: Option<Arc<AppState>>,
    auth_config: Option<Arc<AuthConfig>>,
    rate_limit_config: Option<Arc<RateLimitConfig>>,
    port: u16,
    host: String,
}

impl ApiServerBuilder {
    /// Create new API server builder
    pub fn new() -> Self {
        Self {
            app_state: None,
            auth_config: None,
            rate_limit_config: None,
            port: 3000,
            host: "0.0.0.0".to_string(),
        }
    }

    /// Set application state
    pub fn with_state(mut self, state: Arc<AppState>) -> Self {
        self.app_state = Some(state);
        self
    }

    /// Set authentication configuration
    pub fn with_auth(mut self, config: Arc<AuthConfig>) -> Self {
        self.auth_config = Some(config);
        self
    }

    /// Set rate limit configuration
    pub fn with_rate_limit(mut self, config: Arc<RateLimitConfig>) -> Self {
        self.rate_limit_config = Some(config);
        self
    }

    /// Set server port
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set server host
    pub fn host(mut self, host: String) -> Self {
        self.host = host;
        self
    }

    /// Build the router
    pub fn build_router(self) -> Router {
        let app_state = self.app_state.expect("Application state is required");
        let auth_config = self.auth_config.expect("Auth config is required");
        let rate_limit_config = self.rate_limit_config.expect("Rate limit config is required");

        create_app_router(app_state, auth_config, rate_limit_config)
    }

    /// Build and start the server
    pub async fn serve(self) -> Result<(), std::io::Error> {
        let addr = format!("{}:{}", self.host, self.port);
        let router = self.build_router();

        let listener = tokio::net::TcpListener::bind(&addr).await?;

        tracing::info!("CADDY API v{} listening on http://{}", API_VERSION, addr);

        axum::serve(listener, router).await
    }
}

impl Default for ApiServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Create default application state
pub fn create_default_app_state() -> Arc<AppState> {
    Arc::new(AppState {
        db_pool: Arc::new(()),
        config: Arc::new(AppConfig::default()),
    })
}

/// Create default auth config
pub fn create_default_auth_config() -> Arc<AuthConfig> {
    use crate::enterprise::auth::JwtManager;

    let jwt_manager = Arc::new(JwtManager::new(
        std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "default-secret-change-in-production".to_string()),
    ));

    Arc::new(AuthConfig::new(jwt_manager))
}

/// Create default rate limit config
pub fn create_default_rate_limit_config() -> Arc<RateLimitConfig> {
    use crate::enterprise::ratelimit::{RateLimiter, RateLimiterConfig};

    let limiter = Arc::new(RateLimiter::new(RateLimiterConfig::default()));
    Arc::new(RateLimitConfig::new(limiter))
}

/// Quick start server with default configuration
pub async fn quick_start_server(port: u16) -> Result<(), std::io::Error> {
    ApiServerBuilder::new()
        .with_state(create_default_app_state())
        .with_auth(create_default_auth_config())
        .with_rate_limit(create_default_rate_limit_config())
        .port(port)
        .serve()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_version() {
        assert_eq!(API_VERSION, "v0.3.0");
        assert_eq!(API_MAJOR_VERSION, 0);
        assert_eq!(API_MINOR_VERSION, 3);
        assert_eq!(API_PATCH_VERSION, 0);
    }

    #[test]
    fn test_server_builder() {
        let builder = ApiServerBuilder::new()
            .port(8080)
            .host("127.0.0.1".to_string());

        assert_eq!(builder.port, 8080);
        assert_eq!(builder.host, "127.0.0.1");
    }

    #[test]
    fn test_default_app_state() {
        let state = create_default_app_state();
        assert_eq!(state.config.max_scan_depth, 10);
        assert_eq!(state.config.default_page_size, 20);
    }
}
