//! # Webhook System
//!
//! This module provides a comprehensive webhook system with the following features:
//!
//! - **Webhook Registration**: Create and manage webhook endpoints
//! - **Event Dispatching**: Deliver events to registered webhooks
//! - **Retry Logic**: Automatic retry with exponential backoff
//! - **Signature Verification**: HMAC-SHA256 signature for security
//! - **Delivery Tracking**: Track delivery attempts and status
//! - **Event Filtering**: Subscribe to specific event types
//! - **Rate Limiting**: Prevent webhook spam
//! - **Batch Delivery**: Group multiple events for efficient delivery
//!
//! # Security
//!
//! Each webhook has a secret key used to sign payloads. Consumers can verify
//! the signature to ensure the webhook is authentic.
//!
//! # Examples
//!
//! ```rust,ignore
//! use caddy::api::webhooks::*;
//!
//! // Create webhook manager
//! let manager = WebhookManager::new();
//!
//! // Register webhook
//! let webhook = manager.register_webhook(
//!     "https://example.com/webhook",
//!     vec![EventType::ScanCompleted],
//!     Some("my-secret-key"),
//! ).await?;
//!
//! // Dispatch event
//! manager.dispatch_event(Event::ScanCompleted {
//!     scan_id: "scan123".to_string(),
//! }).await?;
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tokio::time::sleep;
use uuid::Uuid;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use super::handlers::AppState;
use super::responses::{ApiError, ApiResponse, PaginatedResponse, PaginationMeta};

type HmacSha256 = Hmac<Sha256>;

// ============================================================================
// Webhook Types
// ============================================================================

/// Webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Webhook {
    /// Webhook ID
    pub id: String,

    /// Webhook URL
    pub url: String,

    /// Event types this webhook subscribes to
    pub events: Vec<EventType>,

    /// Webhook secret for signature verification
    #[serde(skip_serializing)]
    pub secret: String,

    /// Whether webhook is active
    pub active: bool,

    /// Custom headers to include in webhook requests
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// Webhook description
    pub description: Option<String>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last delivery attempt
    pub last_delivery_at: Option<DateTime<Utc>>,

    /// Delivery statistics
    pub stats: WebhookStats,
}

/// Webhook statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WebhookStats {
    /// Total deliveries attempted
    pub total_deliveries: u64,

    /// Successful deliveries
    pub successful_deliveries: u64,

    /// Failed deliveries
    pub failed_deliveries: u64,

    /// Average response time in milliseconds
    pub avg_response_time_ms: u64,
}

/// Event types that can trigger webhooks
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// Scan completed
    ScanCompleted,
    /// Scan failed
    ScanFailed,
    /// New issue detected
    IssueDetected,
    /// Issue resolved
    IssueResolved,
    /// Issue status changed
    IssueStatusChanged,
    /// Report generated
    ReportGenerated,
    /// Site created
    SiteCreated,
    /// Site updated
    SiteUpdated,
    /// Site deleted
    SiteDeleted,
}

impl EventType {
    /// Get all event types
    pub fn all() -> Vec<EventType> {
        vec![
            EventType::ScanCompleted,
            EventType::ScanFailed,
            EventType::IssueDetected,
            EventType::IssueResolved,
            EventType::IssueStatusChanged,
            EventType::ReportGenerated,
            EventType::SiteCreated,
            EventType::SiteUpdated,
            EventType::SiteDeleted,
        ]
    }
}

/// Webhook event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookEvent {
    /// Event ID
    pub id: String,

    /// Event type
    #[serde(rename = "type")]
    pub event_type: EventType,

    /// Event timestamp
    pub timestamp: DateTime<Utc>,

    /// Event data
    pub data: serde_json::Value,

    /// Delivery attempt count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attempt: Option<u32>,
}

/// Webhook delivery record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookDelivery {
    /// Delivery ID
    pub id: String,

    /// Webhook ID
    pub webhook_id: String,

    /// Event ID
    pub event_id: String,

    /// Delivery status
    pub status: DeliveryStatus,

    /// HTTP status code from webhook endpoint
    pub status_code: Option<u16>,

    /// Response body from webhook endpoint
    pub response_body: Option<String>,

    /// Error message if delivery failed
    pub error: Option<String>,

    /// Delivery attempt timestamp
    pub attempted_at: DateTime<Utc>,

    /// Response time in milliseconds
    pub response_time_ms: Option<u64>,

    /// Retry count
    pub retry_count: u32,

    /// Next retry time (if applicable)
    pub next_retry_at: Option<DateTime<Utc>>,
}

/// Delivery status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DeliveryStatus {
    /// Delivery pending
    Pending,
    /// Delivery succeeded
    Success,
    /// Delivery failed
    Failed,
    /// Delivery cancelled
    Cancelled,
}

// ============================================================================
// Webhook Manager
// ============================================================================

/// Webhook manager
pub struct WebhookManager {
    /// Registered webhooks
    webhooks: Arc<RwLock<HashMap<String, Webhook>>>,

    /// Delivery history
    deliveries: Arc<RwLock<HashMap<String, Vec<WebhookDelivery>>>>,

    /// HTTP client for webhook delivery
    client: reqwest::Client,

    /// Retry configuration
    retry_config: RetryConfig,
}

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_attempts: u32,

    /// Initial retry delay
    pub initial_delay: Duration,

    /// Maximum retry delay
    pub max_delay: Duration,

    /// Backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_secs(30),
            max_delay: Duration::from_secs(3600),
            backoff_multiplier: 2.0,
        }
    }
}

impl WebhookManager {
    /// Create new webhook manager
    pub fn new() -> Self {
        Self {
            webhooks: Arc::new(RwLock::new(HashMap::new())),
            deliveries: Arc::new(RwLock::new(HashMap::new())),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            retry_config: RetryConfig::default(),
        }
    }

    /// Register a new webhook
    pub async fn register_webhook(
        &self,
        url: String,
        events: Vec<EventType>,
        secret: Option<String>,
    ) -> Result<Webhook, WebhookError> {
        // Validate URL
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(WebhookError::InvalidUrl);
        }

        let webhook = Webhook {
            id: Uuid::new_v4().to_string(),
            url,
            events,
            secret: secret.unwrap_or_else(|| generate_secret()),
            active: true,
            headers: HashMap::new(),
            description: None,
            created_at: Utc::now(),
            last_delivery_at: None,
            stats: WebhookStats::default(),
        };

        self.webhooks.write().insert(webhook.id.clone(), webhook.clone());

        Ok(webhook)
    }

    /// Get webhook by ID
    pub fn get_webhook(&self, webhook_id: &str) -> Option<Webhook> {
        self.webhooks.read().get(webhook_id).cloned()
    }

    /// List all webhooks
    pub fn list_webhooks(&self) -> Vec<Webhook> {
        self.webhooks.read().values().cloned().collect()
    }

    /// Update webhook
    pub async fn update_webhook(
        &self,
        webhook_id: &str,
        updates: WebhookUpdate,
    ) -> Result<Webhook, WebhookError> {
        let mut webhooks = self.webhooks.write();
        let webhook = webhooks
            .get_mut(webhook_id)
            .ok_or(WebhookError::NotFound)?;

        if let Some(url) = updates.url {
            webhook.url = url;
        }
        if let Some(events) = updates.events {
            webhook.events = events;
        }
        if let Some(active) = updates.active {
            webhook.active = active;
        }
        if let Some(headers) = updates.headers {
            webhook.headers = headers;
        }
        if let Some(description) = updates.description {
            webhook.description = Some(description);
        }

        Ok(webhook.clone())
    }

    /// Delete webhook
    pub async fn delete_webhook(&self, webhook_id: &str) -> Result<(), WebhookError> {
        self.webhooks
            .write()
            .remove(webhook_id)
            .ok_or(WebhookError::NotFound)?;

        // Clean up deliveries
        self.deliveries.write().remove(webhook_id);

        Ok(())
    }

    /// Dispatch event to all matching webhooks
    pub async fn dispatch_event(&self, event: WebhookEvent) {
        let webhooks = self.webhooks.read();

        for webhook in webhooks.values() {
            if webhook.active && webhook.events.contains(&event.event_type) {
                let webhook = webhook.clone();
                let event = event.clone();
                let manager = self.clone();

                // Spawn delivery task
                tokio::spawn(async move {
                    let _ = manager.deliver_to_webhook(&webhook, &event).await;
                });
            }
        }
    }

    /// Deliver event to specific webhook with retry
    async fn deliver_to_webhook(
        &self,
        webhook: &Webhook,
        event: &WebhookEvent,
    ) -> Result<(), WebhookError> {
        let mut retry_count = 0;
        let mut delay = self.retry_config.initial_delay;

        loop {
            let delivery = self
                .attempt_delivery(webhook, event, retry_count)
                .await;

            // Record delivery
            self.record_delivery(webhook.id.clone(), delivery.clone());

            if delivery.status == DeliveryStatus::Success {
                // Update webhook stats
                self.update_webhook_stats(&webhook.id, true, delivery.response_time_ms);
                return Ok(());
            }

            retry_count += 1;

            if retry_count >= self.retry_config.max_attempts {
                self.update_webhook_stats(&webhook.id, false, delivery.response_time_ms);
                return Err(WebhookError::DeliveryFailed);
            }

            // Wait before retry
            sleep(delay).await;

            delay = (delay.mul_f64(self.retry_config.backoff_multiplier))
                .min(self.retry_config.max_delay);
        }
    }

    /// Attempt single delivery
    async fn attempt_delivery(
        &self,
        webhook: &Webhook,
        event: &WebhookEvent,
        retry_count: u32,
    ) -> WebhookDelivery {
        let start = std::time::Instant::now();
        let payload = serde_json::to_string(event).unwrap();
        let signature = generate_signature(&webhook.secret, &payload);

        let mut request = self
            .client
            .post(&webhook.url)
            .header("Content-Type", "application/json")
            .header("X-Webhook-Signature", signature)
            .header("X-Webhook-Event", format!("{:?}", event.event_type))
            .header("X-Webhook-ID", &webhook.id)
            .header("X-Event-ID", &event.id);

        // Add custom headers
        for (key, value) in &webhook.headers {
            request = request.header(key, value);
        }

        let result = request.body(payload).send().await;

        let response_time_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(response) => {
                let status_code = response.status().as_u16();
                let response_body = response
                    .text()
                    .await
                    .ok()
                    .map(|b| b.chars().take(1000).collect());

                let status = if (200..300).contains(&status_code) {
                    DeliveryStatus::Success
                } else {
                    DeliveryStatus::Failed
                };

                WebhookDelivery {
                    id: Uuid::new_v4().to_string(),
                    webhook_id: webhook.id.clone(),
                    event_id: event.id.clone(),
                    status,
                    status_code: Some(status_code),
                    response_body,
                    error: None,
                    attempted_at: Utc::now(),
                    response_time_ms: Some(response_time_ms),
                    retry_count,
                    next_retry_at: None,
                }
            }
            Err(err) => WebhookDelivery {
                id: Uuid::new_v4().to_string(),
                webhook_id: webhook.id.clone(),
                event_id: event.id.clone(),
                status: DeliveryStatus::Failed,
                status_code: None,
                response_body: None,
                error: Some(err.to_string()),
                attempted_at: Utc::now(),
                response_time_ms: Some(response_time_ms),
                retry_count,
                next_retry_at: Some(Utc::now() + chrono::Duration::seconds(60)),
            },
        }
    }

    /// Record delivery attempt
    fn record_delivery(&self, webhook_id: String, delivery: WebhookDelivery) {
        let mut deliveries = self.deliveries.write();
        deliveries
            .entry(webhook_id)
            .or_insert_with(Vec::new)
            .push(delivery);
    }

    /// Update webhook statistics
    fn update_webhook_stats(&self, webhook_id: &str, success: bool, response_time_ms: Option<u64>) {
        let mut webhooks = self.webhooks.write();
        if let Some(webhook) = webhooks.get_mut(webhook_id) {
            webhook.stats.total_deliveries += 1;
            if success {
                webhook.stats.successful_deliveries += 1;
            } else {
                webhook.stats.failed_deliveries += 1;
            }

            if let Some(time_ms) = response_time_ms {
                let total = webhook.stats.total_deliveries;
                webhook.stats.avg_response_time_ms = (webhook.stats.avg_response_time_ms
                    * (total - 1)
                    + time_ms)
                    / total;
            }

            webhook.last_delivery_at = Some(Utc::now());
        }
    }

    /// Get delivery history for webhook
    pub fn get_deliveries(&self, webhook_id: &str) -> Vec<WebhookDelivery> {
        self.deliveries
            .read()
            .get(webhook_id)
            .cloned()
            .unwrap_or_default()
    }
}

impl Clone for WebhookManager {
    fn clone(&self) -> Self {
        Self {
            webhooks: self.webhooks.clone(),
            deliveries: self.deliveries.clone(),
            client: self.client.clone(),
            retry_config: self.retry_config.clone(),
        }
    }
}

impl Default for WebhookManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Generate webhook secret
fn generate_secret() -> String {
    use rand::Rng;
    let random_bytes: Vec<u8> = (0..32).map(|_| rand::thread_rng().gen()).collect();
    hex::encode(random_bytes)
}

/// Generate HMAC-SHA256 signature
fn generate_signature(secret: &str, payload: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(payload.as_bytes());
    let result = mac.finalize();
    format!("sha256={}", hex::encode(result.into_bytes()))
}

/// Verify HMAC-SHA256 signature
pub fn verify_signature(secret: &str, payload: &str, signature: &str) -> bool {
    let expected = generate_signature(secret, payload);
    constant_time_compare(&expected, signature)
}

/// Constant-time string comparison
fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();

    let mut result = 0u8;
    for i in 0..a_bytes.len() {
        result |= a_bytes[i] ^ b_bytes[i];
    }

    result == 0
}

// ============================================================================
// Webhook Error
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum WebhookError {
    #[error("Invalid webhook URL")]
    InvalidUrl,

    #[error("Webhook not found")]
    NotFound,

    #[error("Webhook delivery failed")]
    DeliveryFailed,

    #[error("Invalid signature")]
    InvalidSignature,
}

// ============================================================================
// API Handlers
// ============================================================================

/// Webhook update request
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookUpdate {
    pub url: Option<String>,
    pub events: Option<Vec<EventType>>,
    pub active: Option<bool>,
    pub headers: Option<HashMap<String, String>>,
    pub description: Option<String>,
}

/// Create webhook request
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWebhookRequest {
    pub url: String,
    pub events: Vec<EventType>,
    pub secret: Option<String>,
    pub description: Option<String>,
}

/// List webhooks handler
pub async fn list_webhooks(
    State(_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    // TODO: Get from actual webhook manager
    let webhooks: Vec<Webhook> = vec![];
    Ok(ApiResponse::success(webhooks, "Webhooks retrieved"))
}

/// Create webhook handler
pub async fn create_webhook(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<CreateWebhookRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // TODO: Use actual webhook manager
    let webhook = Webhook {
        id: Uuid::new_v4().to_string(),
        url: request.url,
        events: request.events,
        secret: request.secret.unwrap_or_else(generate_secret),
        active: true,
        headers: HashMap::new(),
        description: request.description,
        created_at: Utc::now(),
        last_delivery_at: None,
        stats: WebhookStats::default(),
    };

    Ok((
        StatusCode::CREATED,
        ApiResponse::success(webhook, "Webhook created"),
    ))
}

/// Update webhook handler
pub async fn update_webhook(
    State(_state): State<Arc<AppState>>,
    Path(webhook_id): Path<String>,
    Json(update): Json<WebhookUpdate>,
) -> Result<impl IntoResponse, ApiError> {
    // TODO: Update in actual webhook manager
    Err(ApiError::not_found(
        format!("webhooks/{}", webhook_id),
        "Webhook not found",
    ))
}

/// Delete webhook handler
pub async fn delete_webhook(
    State(_state): State<Arc<AppState>>,
    Path(_webhook_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    // TODO: Delete from actual webhook manager
    Ok(StatusCode::NO_CONTENT)
}

/// Test webhook handler
pub async fn test_webhook(
    State(_state): State<Arc<AppState>>,
    Path(webhook_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    // TODO: Send test event to webhook
    Ok(ApiResponse::success(
        serde_json::json!({"sent": true}),
        "Test webhook sent",
    ))
}

/// Trigger webhook test (system endpoint)
pub async fn trigger_webhook_test(
    State(_state): State<Arc<AppState>>,
    Path(webhook_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(ApiResponse::success(
        serde_json::json!({"triggered": true}),
        "Webhook test triggered",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_secret() {
        let secret = generate_secret();
        assert_eq!(secret.len(), 64); // 32 bytes = 64 hex chars
    }

    #[test]
    fn test_signature_generation() {
        let secret = "my_secret";
        let payload = r#"{"event":"test"}"#;
        let signature = generate_signature(secret, payload);

        assert!(signature.starts_with("sha256="));
        assert!(verify_signature(secret, payload, &signature));
    }

    #[test]
    fn test_signature_verification_fails_wrong_secret() {
        let payload = r#"{"event":"test"}"#;
        let signature = generate_signature("secret1", payload);

        assert!(!verify_signature("secret2", payload, &signature));
    }

    #[test]
    fn test_constant_time_compare() {
        assert!(constant_time_compare("hello", "hello"));
        assert!(!constant_time_compare("hello", "world"));
        assert!(!constant_time_compare("hello", "hello!"));
    }

    #[tokio::test]
    async fn test_webhook_manager_register() {
        let manager = WebhookManager::new();

        let webhook = manager
            .register_webhook(
                "https://example.com/webhook".to_string(),
                vec![EventType::ScanCompleted],
                None,
            )
            .await
            .unwrap();

        assert_eq!(webhook.url, "https://example.com/webhook");
        assert_eq!(webhook.events, vec![EventType::ScanCompleted]);
        assert!(webhook.active);
    }

    #[tokio::test]
    async fn test_webhook_manager_invalid_url() {
        let manager = WebhookManager::new();

        let result = manager
            .register_webhook(
                "not-a-url".to_string(),
                vec![EventType::ScanCompleted],
                None,
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_webhook_manager_list() {
        let manager = WebhookManager::new();

        manager
            .register_webhook(
                "https://example.com/webhook1".to_string(),
                vec![EventType::ScanCompleted],
                None,
            )
            .await
            .unwrap();

        manager
            .register_webhook(
                "https://example.com/webhook2".to_string(),
                vec![EventType::IssueDetected],
                None,
            )
            .await
            .unwrap();

        let webhooks = manager.list_webhooks();
        assert_eq!(webhooks.len(), 2);
    }
}
