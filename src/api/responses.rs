//! # API Response Types
//!
//! This module provides standardized response formats for the CADDY REST API including:
//!
//! - Standard success/error response formats
//! - Pagination structures with cursor and offset pagination
//! - Error codes and messages following RFC 7807
//! - HAL (Hypertext Application Language) support
//! - JSON:API specification compliance
//! - OpenAPI/Swagger schema compatibility
//!
//! # Examples
//!
//! ```rust,ignore
//! use caddy::api::responses::*;
//!
//! // Simple success response
//! let response = ApiResponse::success(data, "Resource created successfully");
//!
//! // Paginated response
//! let paginated = PaginatedResponse::new(items, 100, page_info);
//!
//! // Error response
//! let error = ApiError::not_found("scan/123", "Scan not found");
//! ```

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

// ============================================================================
// Standard API Response Types
// ============================================================================

/// Standard API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    /// Response data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,

    /// Response metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ResponseMeta>,

    /// Links (HAL support)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Links>,

    /// Success status
    pub success: bool,

    /// Response message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Response timestamp
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    /// Create a successful response
    pub fn success(data: T, message: impl Into<String>) -> Self {
        Self {
            data: Some(data),
            meta: None,
            links: None,
            success: true,
            message: Some(message.into()),
            timestamp: Utc::now(),
        }
    }

    /// Create a successful response with metadata
    pub fn success_with_meta(data: T, meta: ResponseMeta, message: impl Into<String>) -> Self {
        Self {
            data: Some(data),
            meta: Some(meta),
            links: None,
            success: true,
            message: Some(message.into()),
            timestamp: Utc::now(),
        }
    }

    /// Create a successful response with links (HAL)
    pub fn success_with_links(data: T, links: Links, message: impl Into<String>) -> Self {
        Self {
            data: Some(data),
            meta: None,
            links: Some(links),
            success: true,
            message: Some(message.into()),
            timestamp: Utc::now(),
        }
    }

    /// Create a successful response with both metadata and links
    pub fn success_with_meta_and_links(
        data: T,
        meta: ResponseMeta,
        links: Links,
        message: impl Into<String>,
    ) -> Self {
        Self {
            data: Some(data),
            meta: Some(meta),
            links: Some(links),
            success: true,
            message: Some(message.into()),
            timestamp: Utc::now(),
        }
    }
}

impl<T> IntoResponse for ApiResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMeta {
    /// Request ID for tracing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,

    /// API version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Processing duration in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,

    /// Rate limit information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<RateLimitMeta>,

    /// Additional metadata
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl ResponseMeta {
    /// Create new metadata
    pub fn new() -> Self {
        Self {
            request_id: None,
            version: Some("v1".to_string()),
            duration_ms: None,
            rate_limit: None,
            extra: HashMap::new(),
        }
    }

    /// Set request ID
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }

    /// Set version
    pub fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

    /// Set duration
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }

    /// Set rate limit info
    pub fn with_rate_limit(mut self, rate_limit: RateLimitMeta) -> Self {
        self.rate_limit = Some(rate_limit);
        self
    }

    /// Add custom field
    pub fn add_field(mut self, key: String, value: serde_json::Value) -> Self {
        self.extra.insert(key, value);
        self
    }
}

impl Default for ResponseMeta {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate limit metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitMeta {
    /// Rate limit
    pub limit: u64,
    /// Remaining requests
    pub remaining: u64,
    /// Reset timestamp (Unix epoch)
    pub reset: u64,
}

// ============================================================================
// HAL (Hypertext Application Language) Support
// ============================================================================

/// HAL links
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Links {
    /// Self link
    #[serde(rename = "self", skip_serializing_if = "Option::is_none")]
    pub self_link: Option<Link>,

    /// Related links
    #[serde(flatten)]
    pub related: HashMap<String, Link>,
}

impl Links {
    /// Create new links
    pub fn new() -> Self {
        Self {
            self_link: None,
            related: HashMap::new(),
        }
    }

    /// Set self link
    pub fn with_self(mut self, href: String) -> Self {
        self.self_link = Some(Link::new(href));
        self
    }

    /// Add related link
    pub fn add_link(mut self, rel: String, href: String) -> Self {
        self.related.insert(rel, Link::new(href));
        self
    }

    /// Add templated link
    pub fn add_templated_link(mut self, rel: String, href: String) -> Self {
        self.related.insert(rel, Link::templated(href));
        self
    }
}

impl Default for Links {
    fn default() -> Self {
        Self::new()
    }
}

/// HAL link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    /// Link URL
    pub href: String,

    /// Link is templated (RFC 6570)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub templated: Option<bool>,

    /// Link type (media type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,

    /// Human-readable title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

impl Link {
    /// Create new link
    pub fn new(href: String) -> Self {
        Self {
            href,
            templated: None,
            type_: None,
            title: None,
        }
    }

    /// Create templated link
    pub fn templated(href: String) -> Self {
        Self {
            href,
            templated: Some(true),
            type_: None,
            title: None,
        }
    }

    /// Set type
    pub fn with_type(mut self, type_: String) -> Self {
        self.type_ = Some(type_);
        self
    }

    /// Set title
    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }
}

// ============================================================================
// Pagination Support
// ============================================================================

/// Paginated response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResponse<T> {
    /// Items in current page
    pub data: Vec<T>,

    /// Pagination metadata
    pub pagination: PaginationMeta,

    /// Links to other pages
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<PaginationLinks>,

    /// Success status
    pub success: bool,

    /// Response timestamp
    pub timestamp: DateTime<Utc>,
}

impl<T> PaginatedResponse<T>
where
    T: Serialize,
{
    /// Create new paginated response
    pub fn new(data: Vec<T>, total: u64, pagination: PaginationMeta) -> Self {
        Self {
            data,
            pagination,
            links: None,
            success: true,
            timestamp: Utc::now(),
        }
    }

    /// Create with links
    pub fn with_links(mut self, links: PaginationLinks) -> Self {
        self.links = Some(links);
        self
    }
}

impl<T> IntoResponse for PaginatedResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

/// Pagination metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationMeta {
    /// Current page (1-indexed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u64>,

    /// Items per page
    pub per_page: u64,

    /// Total number of items
    pub total: u64,

    /// Total number of pages
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_pages: Option<u64>,

    /// Cursor for cursor-based pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,

    /// Next cursor
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,

    /// Has more items
    pub has_more: bool,
}

impl PaginationMeta {
    /// Create offset-based pagination
    pub fn offset(page: u64, per_page: u64, total: u64) -> Self {
        let total_pages = (total + per_page - 1) / per_page;
        Self {
            page: Some(page),
            per_page,
            total,
            total_pages: Some(total_pages),
            cursor: None,
            next_cursor: None,
            has_more: page < total_pages,
        }
    }

    /// Create cursor-based pagination
    pub fn cursor(per_page: u64, total: u64, cursor: Option<String>, next_cursor: Option<String>) -> Self {
        Self {
            page: None,
            per_page,
            total,
            total_pages: None,
            cursor,
            next_cursor: next_cursor.clone(),
            has_more: next_cursor.is_some(),
        }
    }
}

/// Pagination links
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationLinks {
    /// First page link
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first: Option<String>,

    /// Previous page link
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev: Option<String>,

    /// Current page link
    #[serde(rename = "self")]
    pub self_link: String,

    /// Next page link
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,

    /// Last page link
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last: Option<String>,
}

impl PaginationLinks {
    /// Create new pagination links
    pub fn new(base_url: &str, page: u64, total_pages: u64) -> Self {
        Self {
            first: Some(format!("{}?page=1", base_url)),
            prev: if page > 1 {
                Some(format!("{}?page={}", base_url, page - 1))
            } else {
                None
            },
            self_link: format!("{}?page={}", base_url, page),
            next: if page < total_pages {
                Some(format!("{}?page={}", base_url, page + 1))
            } else {
                None
            },
            last: Some(format!("{}?page={}", base_url, total_pages)),
        }
    }

    /// Create cursor-based links
    pub fn cursor_based(base_url: &str, cursor: Option<&str>, next_cursor: Option<&str>) -> Self {
        Self {
            first: None,
            prev: None,
            self_link: if let Some(c) = cursor {
                format!("{}?cursor={}", base_url, c)
            } else {
                base_url.to_string()
            },
            next: next_cursor.map(|c| format!("{}?cursor={}", base_url, c)),
            last: None,
        }
    }
}

// ============================================================================
// Error Response (RFC 7807)
// ============================================================================

/// API Error following RFC 7807 Problem Details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    /// Error type URI
    #[serde(rename = "type")]
    pub type_: String,

    /// Human-readable title
    pub title: String,

    /// HTTP status code
    pub status: u16,

    /// Detailed error message
    pub detail: String,

    /// Instance URI (specific occurrence)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,

    /// Error code
    pub code: String,

    /// Additional error fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<FieldError>>,

    /// Response timestamp
    pub timestamp: DateTime<Utc>,

    /// Request ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl ApiError {
    /// Create a new error
    pub fn new(
        status: StatusCode,
        code: impl Into<String>,
        title: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        let code_str = code.into();
        Self {
            type_: format!("https://api.caddy.io/errors/{}", code_str),
            title: title.into(),
            status: status.as_u16(),
            detail: detail.into(),
            instance: None,
            code: code_str,
            errors: None,
            timestamp: Utc::now(),
            request_id: None,
        }
    }

    /// Bad request error
    pub fn bad_request(detail: impl Into<String>) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "BAD_REQUEST",
            "Bad Request",
            detail,
        )
    }

    /// Validation error
    pub fn validation_error(errors: Vec<FieldError>) -> Self {
        let mut error = Self::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "VALIDATION_ERROR",
            "Validation Failed",
            "The request contains invalid fields",
        );
        error.errors = Some(errors);
        error
    }

    /// Unauthorized error
    pub fn unauthorized(detail: impl Into<String>) -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "UNAUTHORIZED",
            "Unauthorized",
            detail,
        )
    }

    /// Forbidden error
    pub fn forbidden(detail: impl Into<String>) -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            "FORBIDDEN",
            "Forbidden",
            detail,
        )
    }

    /// Not found error
    pub fn not_found(resource: impl Into<String>, detail: impl Into<String>) -> Self {
        let mut error = Self::new(
            StatusCode::NOT_FOUND,
            "NOT_FOUND",
            "Resource Not Found",
            detail,
        );
        error.instance = Some(format!("/api/v1/{}", resource.into()));
        error
    }

    /// Conflict error
    pub fn conflict(detail: impl Into<String>) -> Self {
        Self::new(
            StatusCode::CONFLICT,
            "CONFLICT",
            "Resource Conflict",
            detail,
        )
    }

    /// Rate limit error
    pub fn rate_limit_exceeded(retry_after: u64) -> Self {
        Self::new(
            StatusCode::TOO_MANY_REQUESTS,
            "RATE_LIMIT_EXCEEDED",
            "Rate Limit Exceeded",
            format!("Rate limit exceeded. Please retry after {} seconds", retry_after),
        )
    }

    /// Internal server error
    pub fn internal_error(detail: impl Into<String>) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_ERROR",
            "Internal Server Error",
            detail,
        )
    }

    /// Service unavailable error
    pub fn service_unavailable(detail: impl Into<String>) -> Self {
        Self::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "SERVICE_UNAVAILABLE",
            "Service Unavailable",
            detail,
        )
    }

    /// Set instance
    pub fn with_instance(mut self, instance: String) -> Self {
        self.instance = Some(instance);
        self
    }

    /// Set request ID
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self)).into_response()
    }
}

/// Field-level validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldError {
    /// Field name
    pub field: String,

    /// Error code
    pub code: String,

    /// Error message
    pub message: String,

    /// Rejected value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected_value: Option<serde_json::Value>,
}

impl FieldError {
    /// Create new field error
    pub fn new(field: impl Into<String>, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            code: code.into(),
            message: message.into(),
            rejected_value: None,
        }
    }

    /// Set rejected value
    pub fn with_value(mut self, value: serde_json::Value) -> Self {
        self.rejected_value = Some(value);
        self
    }
}

// ============================================================================
// JSON:API Support
// ============================================================================

/// JSON:API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonApiResponse<T> {
    /// Primary data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<JsonApiResource<T>>,

    /// Included resources
    #[serde(skip_serializing_if = "Option::is_none")]
    pub included: Option<Vec<serde_json::Value>>,

    /// Links
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<HashMap<String, String>>,

    /// Meta information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,

    /// Errors (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<JsonApiError>>,
}

/// JSON:API resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonApiResource<T> {
    /// Resource type
    #[serde(rename = "type")]
    pub type_: String,

    /// Resource ID
    pub id: String,

    /// Resource attributes
    pub attributes: T,

    /// Relationships
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationships: Option<HashMap<String, serde_json::Value>>,

    /// Links
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<HashMap<String, String>>,
}

/// JSON:API error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonApiError {
    /// Error ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// HTTP status code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Error code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// Error title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Error detail
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,

    /// Source location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<ErrorSource>,
}

/// Error source location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSource {
    /// JSON Pointer to the value in request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pointer: Option<String>,

    /// Parameter name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter: Option<String>,
}

// ============================================================================
// Health Check Response
// ============================================================================

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    /// Overall health status
    pub status: HealthStatus,

    /// Service version
    pub version: String,

    /// Uptime in seconds
    pub uptime_seconds: u64,

    /// Component health checks
    pub checks: HashMap<String, ComponentHealth>,

    /// Response timestamp
    pub timestamp: DateTime<Utc>,
}

/// Health status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// All systems operational
    Healthy,
    /// Some non-critical issues
    Degraded,
    /// Critical issues
    Unhealthy,
}

/// Component health
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentHealth {
    /// Component status
    pub status: HealthStatus,

    /// Component message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Response time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_time_ms: Option<u64>,
}

impl ComponentHealth {
    /// Create healthy component
    pub fn healthy() -> Self {
        Self {
            status: HealthStatus::Healthy,
            message: None,
            response_time_ms: None,
        }
    }

    /// Create degraded component
    pub fn degraded(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Degraded,
            message: Some(message.into()),
            response_time_ms: None,
        }
    }

    /// Create unhealthy component
    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            message: Some(message.into()),
            response_time_ms: None,
        }
    }

    /// Set response time
    pub fn with_response_time(mut self, ms: u64) -> Self {
        self.response_time_ms = Some(ms);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_success() {
        let response = ApiResponse::success("test data", "Success");
        assert!(response.success);
        assert_eq!(response.data, Some("test data"));
        assert_eq!(response.message, Some("Success".to_string()));
    }

    #[test]
    fn test_pagination_offset() {
        let meta = PaginationMeta::offset(2, 10, 50);
        assert_eq!(meta.page, Some(2));
        assert_eq!(meta.per_page, 10);
        assert_eq!(meta.total, 50);
        assert_eq!(meta.total_pages, Some(5));
        assert!(meta.has_more);
    }

    #[test]
    fn test_pagination_cursor() {
        let meta = PaginationMeta::cursor(
            10,
            50,
            Some("abc123".to_string()),
            Some("xyz789".to_string()),
        );
        assert_eq!(meta.cursor, Some("abc123".to_string()));
        assert_eq!(meta.next_cursor, Some("xyz789".to_string()));
        assert!(meta.has_more);
    }

    #[test]
    fn test_api_error_creation() {
        let error = ApiError::not_found("scans/123", "Scan not found");
        assert_eq!(error.status, 404);
        assert_eq!(error.code, "NOT_FOUND");
        assert!(error.instance.is_some());
    }

    #[test]
    fn test_validation_error() {
        let field_errors = vec![
            FieldError::new("email", "INVALID_FORMAT", "Invalid email format"),
            FieldError::new("age", "OUT_OF_RANGE", "Age must be between 0 and 120"),
        ];

        let error = ApiError::validation_error(field_errors);
        assert_eq!(error.status, 422);
        assert!(error.errors.is_some());
        assert_eq!(error.errors.unwrap().len(), 2);
    }

    #[test]
    fn test_hal_links() {
        let links = Links::new()
            .with_self("/api/v1/scans/123".to_string())
            .add_link("issues".to_string(), "/api/v1/scans/123/issues".to_string())
            .add_templated_link("issue".to_string(), "/api/v1/issues/{id}".to_string());

        assert!(links.self_link.is_some());
        assert_eq!(links.related.len(), 2);
    }

    #[test]
    fn test_health_response() {
        let mut checks = HashMap::new();
        checks.insert("database".to_string(), ComponentHealth::healthy());
        checks.insert("redis".to_string(), ComponentHealth::degraded("High latency"));

        let health = HealthResponse {
            status: HealthStatus::Degraded,
            version: "1.0.0".to_string(),
            uptime_seconds: 3600,
            checks,
            timestamp: Utc::now(),
        };

        assert_eq!(health.status, HealthStatus::Degraded);
        assert_eq!(health.checks.len(), 2);
    }
}
