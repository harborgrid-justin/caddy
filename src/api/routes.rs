//! # API Routes
//!
//! This module defines all REST API routes for the CADDY accessibility scanning service.
//!
//! ## Route Structure
//!
//! - `/api/v1/scans` - Accessibility scanning endpoints
//! - `/api/v1/issues` - Issue management endpoints
//! - `/api/v1/reports` - Reporting endpoints
//! - `/api/v1/sites` - Site/project management
//! - `/api/v1/settings` - Configuration endpoints
//! - `/api/v1/webhooks` - Webhook management
//!
//! ## Examples
//!
//! ```rust,ignore
//! use caddy::api::routes::*;
//!
//! let app = create_router(app_state, auth_config, rate_limit_config);
//! ```

use axum::{
    extract::State,
    middleware::{self, from_fn_with_state},
    routing::{delete, get, patch, post, put},
    Router,
};
use std::sync::Arc;

use super::handlers::*;
use super::middleware::{
    auth_middleware, cors_layer, rate_limit_middleware,
    request_id_middleware, request_logging_middleware,
    security_headers_middleware, AuthConfig, RateLimitConfig,
};
use super::webhooks::{
    create_webhook, delete_webhook, list_webhooks, test_webhook,
    trigger_webhook_test, update_webhook,
};

// ============================================================================
// API v1 Routes
// ============================================================================

/// Create complete API v1 router
pub fn create_v1_router(
    app_state: Arc<AppState>,
    auth_config: Arc<AuthConfig>,
    rate_limit_config: Arc<RateLimitConfig>,
) -> Router {
    Router::new()
        // Scan routes
        .nest("/scans", scans_routes())
        // Issue routes
        .nest("/issues", issues_routes())
        // Report routes
        .nest("/reports", reports_routes())
        // Site routes
        .nest("/sites", sites_routes())
        // Settings routes
        .nest("/settings", settings_routes())
        // Webhook routes
        .nest("/webhooks", webhooks_routes())
        // Health check
        .route("/health", get(health_check))
        // Apply authentication middleware to protected routes
        .layer(from_fn_with_state(auth_config.clone(), auth_middleware))
        // Apply rate limiting
        .layer(from_fn_with_state(
            rate_limit_config.clone(),
            rate_limit_middleware,
        ))
        .with_state(app_state)
}

/// Scan routes
fn scans_routes() -> Router<Arc<AppState>> {
    Router::new()
        // List scans
        .route("/", get(list_scans))
        // Create scan
        .route("/", post(create_scan))
        // Get specific scan
        .route("/:id", get(get_scan))
        // Delete scan
        .route("/:id", delete(delete_scan))
        // Get scan issues
        .route("/:id/issues", get(get_scan_issues))
        // Generate scan report
        .route("/:id/report", get(generate_report))
        // Cancel scan
        .route("/:id/cancel", post(cancel_scan))
        // Retry failed scan
        .route("/:id/retry", post(retry_scan))
}

/// Issue routes
fn issues_routes() -> Router<Arc<AppState>> {
    Router::new()
        // List issues
        .route("/", get(list_issues))
        // Get specific issue
        .route("/:id", get(get_issue))
        // Update issue
        .route("/:id", patch(update_issue))
        // Delete issue (admin only)
        .route("/:id", delete(delete_issue))
        // Bulk operations
        .route("/bulk/update", post(bulk_update_issues))
        .route("/bulk/export", post(bulk_export_issues))
        // Issue statistics
        .route("/statistics", get(get_issue_statistics))
}

/// Report routes
fn reports_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Generate report for scan
        .route("/scan/:scan_id", get(generate_report))
        // Generate site summary report
        .route("/site/:site_id", get(generate_site_report))
        // List available reports
        .route("/", get(list_reports))
        // Get specific report
        .route("/:id", get(get_report))
        // Delete report
        .route("/:id", delete(delete_report))
        // Download report
        .route("/:id/download", get(download_report))
        // Schedule recurring report
        .route("/schedule", post(schedule_report))
}

/// Site routes
fn sites_routes() -> Router<Arc<AppState>> {
    Router::new()
        // List sites
        .route("/", get(list_sites))
        // Create site
        .route("/", post(create_site))
        // Get specific site
        .route("/:id", get(get_site))
        // Update site
        .route("/:id", put(update_site))
        // Delete site
        .route("/:id", delete(delete_site))
        // Get site scans
        .route("/:id/scans", get(get_site_scans))
        // Get site statistics
        .route("/:id/statistics", get(get_site_statistics))
}

/// Settings routes
fn settings_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Get settings
        .route("/", get(get_settings))
        // Update settings
        .route("/", put(update_settings))
        // Get notification settings
        .route("/notifications", get(get_notification_settings))
        // Update notification settings
        .route("/notifications", put(update_notification_settings))
        // Get API keys
        .route("/api-keys", get(list_api_keys))
        // Create API key
        .route("/api-keys", post(create_api_key))
        // Revoke API key
        .route("/api-keys/:id", delete(revoke_api_key))
}

/// Webhook routes
fn webhooks_routes() -> Router<Arc<AppState>> {
    Router::new()
        // List webhooks
        .route("/", get(list_webhooks))
        // Create webhook
        .route("/", post(create_webhook))
        // Get specific webhook
        .route("/:id", get(get_webhook))
        // Update webhook
        .route("/:id", put(update_webhook))
        // Delete webhook
        .route("/:id", delete(delete_webhook))
        // Test webhook
        .route("/:id/test", post(test_webhook))
        // Get webhook delivery logs
        .route("/:id/deliveries", get(get_webhook_deliveries))
        // Trigger test webhook (system use)
        .route("/trigger/:id", post(trigger_webhook_test))
}

// ============================================================================
// Public Routes (No Authentication Required)
// ============================================================================

/// Create public router (no authentication)
pub fn create_public_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_check))
        // API docs (if enabled)
        .route("/docs", get(api_documentation))
        // OpenAPI spec
        .route("/openapi.json", get(openapi_spec))
        .with_state(app_state)
}

// ============================================================================
// Complete Application Router
// ============================================================================

/// Create complete application router with all routes and middleware
pub fn create_app_router(
    app_state: Arc<AppState>,
    auth_config: Arc<AuthConfig>,
    rate_limit_config: Arc<RateLimitConfig>,
) -> Router {
    // Create API v1 router (protected)
    let api_v1 = create_v1_router(
        app_state.clone(),
        auth_config.clone(),
        rate_limit_config.clone(),
    );

    // Create public router
    let public = create_public_router(app_state.clone());

    // Combine routers
    Router::new()
        .nest("/api/v1", api_v1)
        .merge(public)
        // Apply global middleware
        .layer(middleware::from_fn(request_logging_middleware))
        .layer(middleware::from_fn(security_headers_middleware))
        .layer(middleware::from_fn(request_id_middleware))
        .layer(cors_layer())
}

/// Create router for specific API version
pub fn create_versioned_router(
    version: ApiVersion,
    app_state: Arc<AppState>,
    auth_config: Arc<AuthConfig>,
    rate_limit_config: Arc<RateLimitConfig>,
) -> Router {
    match version {
        ApiVersion::V1 => create_v1_router(app_state, auth_config, rate_limit_config),
        ApiVersion::V2 => {
            // V2 routes would be defined here
            create_v1_router(app_state, auth_config, rate_limit_config)
        }
    }
}

/// API version
#[derive(Debug, Clone, Copy)]
pub enum ApiVersion {
    V1,
    V2,
}

impl ApiVersion {
    /// Get version string
    pub fn as_str(&self) -> &'static str {
        match self {
            ApiVersion::V1 => "v1",
            ApiVersion::V2 => "v2",
        }
    }

    /// Get version path
    pub fn path(&self) -> String {
        format!("/api/{}", self.as_str())
    }
}

// ============================================================================
// Additional Handler Stubs (To Be Implemented)
// ============================================================================

use axum::extract::Path;
use axum::http::StatusCode;
use super::responses::ApiError;

async fn get_scan_issues(
    State(state): State<Arc<AppState>>,
    Path(scan_id): Path<String>,
    Query(params): Query<ListIssuesQuery>,
) -> Result<impl IntoResponse, ApiError> {
    // Filter issues by scan_id
    let mut query = params;
    query.scan_id = Some(scan_id);
    list_issues(State(state), Query(query)).await
}

async fn cancel_scan(
    State(_state): State<Arc<AppState>>,
    Path(scan_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    // TODO: Implement scan cancellation
    Ok(ApiResponse::success(
        serde_json::json!({"id": scan_id, "status": "cancelled"}),
        "Scan cancelled successfully",
    ))
}

async fn retry_scan(
    State(_state): State<Arc<AppState>>,
    Path(scan_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    // TODO: Implement scan retry
    Ok(ApiResponse::success(
        serde_json::json!({"id": scan_id, "status": "queued"}),
        "Scan queued for retry",
    ))
}

async fn delete_issue(
    State(_state): State<Arc<AppState>>,
    Path(issue_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    // TODO: Implement issue deletion
    Ok(StatusCode::NO_CONTENT)
}

async fn bulk_export_issues(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<BulkExportRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // TODO: Implement bulk export
    Ok(ApiResponse::success(
        serde_json::json!({"url": "/exports/issues.csv"}),
        "Export initiated",
    ))
}

async fn get_issue_statistics(
    State(_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    // TODO: Implement issue statistics
    Ok(ApiResponse::success(
        serde_json::json!({
            "total": 0,
            "by_severity": {},
            "by_status": {}
        }),
        "Statistics retrieved",
    ))
}

async fn generate_site_report(
    State(_state): State<Arc<AppState>>,
    Path(site_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    // TODO: Implement site report generation
    Ok(ApiResponse::success(
        serde_json::json!({"url": format!("/reports/site-{}.pdf", site_id)}),
        "Site report generated",
    ))
}

async fn list_reports(
    State(_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(PaginatedResponse::new(
        Vec::<serde_json::Value>::new(),
        0,
        PaginationMeta::offset(1, 20, 0),
    ))
}

async fn get_report(
    State(_state): State<Arc<AppState>>,
    Path(report_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    Err(ApiError::not_found(
        format!("reports/{}", report_id),
        "Report not found",
    ))
}

async fn delete_report(
    State(_state): State<Arc<AppState>>,
    Path(_report_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    Ok(StatusCode::NO_CONTENT)
}

async fn download_report(
    State(_state): State<Arc<AppState>>,
    Path(_report_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    Err(ApiError::not_found("report", "Report not found"))
}

async fn schedule_report(
    State(_state): State<Arc<AppState>>,
    Json(_request): Json<ScheduleReportRequest>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(ApiResponse::success(
        serde_json::json!({"id": Uuid::new_v4().to_string()}),
        "Report scheduled",
    ))
}

async fn get_site(
    State(_state): State<Arc<AppState>>,
    Path(site_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    Err(ApiError::not_found(
        format!("sites/{}", site_id),
        "Site not found",
    ))
}

async fn update_site(
    State(_state): State<Arc<AppState>>,
    Path(_site_id): Path<String>,
    Json(request): Json<CreateSiteRequest>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(ApiResponse::success(request, "Site updated"))
}

async fn delete_site(
    State(_state): State<Arc<AppState>>,
    Path(_site_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    Ok(StatusCode::NO_CONTENT)
}

async fn get_site_scans(
    State(state): State<Arc<AppState>>,
    Path(site_id): Path<String>,
    Query(mut params): Query<ListScansQuery>,
) -> Result<impl IntoResponse, ApiError> {
    params.site_id = Some(site_id);
    list_scans(State(state), Query(params)).await
}

async fn get_site_statistics(
    State(_state): State<Arc<AppState>>,
    Path(_site_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(ApiResponse::success(
        serde_json::json!({
            "total_scans": 0,
            "total_issues": 0,
            "avg_issues_per_scan": 0
        }),
        "Statistics retrieved",
    ))
}

async fn get_notification_settings(
    State(_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    use crate::api::handlers::NotificationPreferences;
    Ok(ApiResponse::success(
        NotificationPreferences {
            email_on_scan_complete: true,
            email_on_new_issues: true,
            webhook_enabled: false,
        },
        "Notification settings retrieved",
    ))
}

async fn update_notification_settings(
    State(_state): State<Arc<AppState>>,
    Json(settings): Json<NotificationPreferences>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(ApiResponse::success(settings, "Notification settings updated"))
}

async fn list_api_keys(
    State(_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(ApiResponse::success(Vec::<ApiKeyInfo>::new(), "API keys retrieved"))
}

async fn create_api_key(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<CreateApiKeyRequest>,
) -> Result<impl IntoResponse, ApiError> {
    Ok((
        StatusCode::CREATED,
        ApiResponse::success(
            ApiKeyInfo {
                id: Uuid::new_v4().to_string(),
                name: request.name,
                key: format!("sk_{}", Uuid::new_v4()),
                created_at: Utc::now(),
            },
            "API key created",
        ),
    ))
}

async fn revoke_api_key(
    State(_state): State<Arc<AppState>>,
    Path(_key_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    Ok(StatusCode::NO_CONTENT)
}

async fn get_webhook(
    State(_state): State<Arc<AppState>>,
    Path(webhook_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    Err(ApiError::not_found(
        format!("webhooks/{}", webhook_id),
        "Webhook not found",
    ))
}

async fn get_webhook_deliveries(
    State(_state): State<Arc<AppState>>,
    Path(_webhook_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(PaginatedResponse::new(
        Vec::<serde_json::Value>::new(),
        0,
        PaginationMeta::offset(1, 20, 0),
    ))
}

async fn api_documentation(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    "API Documentation - Coming Soon"
}

async fn openapi_spec(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    Json(serde_json::json!({
        "openapi": "3.0.0",
        "info": {
            "title": "CADDY Accessibility API",
            "version": "1.0.0"
        }
    }))
}

// ============================================================================
// Request/Response Types for Stubs
// ============================================================================

use serde::{Deserialize, Serialize};
use axum::{extract::Query, Json};
use super::responses::{ApiResponse, PaginatedResponse, PaginationMeta};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use axum::response::IntoResponse;

#[derive(Debug, Deserialize)]
struct BulkExportRequest {
    issue_ids: Vec<String>,
    format: String,
}

#[derive(Debug, Deserialize)]
struct ScheduleReportRequest {
    scan_id: String,
    schedule: String,
}

#[derive(Debug, Serialize)]
struct ApiKeyInfo {
    id: String,
    name: String,
    key: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct CreateApiKeyRequest {
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_version_string() {
        assert_eq!(ApiVersion::V1.as_str(), "v1");
        assert_eq!(ApiVersion::V2.as_str(), "v2");
    }

    #[test]
    fn test_api_version_path() {
        assert_eq!(ApiVersion::V1.path(), "/api/v1");
        assert_eq!(ApiVersion::V2.path(), "/api/v2");
    }
}
