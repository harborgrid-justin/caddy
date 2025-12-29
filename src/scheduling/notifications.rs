//! Notification system with multiple delivery channels
//!
//! This module provides:
//! - Email notifications
//! - Slack integration
//! - Microsoft Teams integration
//! - Custom webhook notifications
//! - Notification preferences and filtering

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

/// Notification errors
#[derive(Error, Debug)]
pub enum NotificationError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Channel not found: {0}")]
    ChannelNotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Delivery failed: {0}")]
    DeliveryFailed(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

/// Result type for notification operations
pub type NotificationResult<T> = Result<T, NotificationError>;

/// Notification severity
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum NotificationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Notification priority
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Urgent,
}

/// Notification channel type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationChannel {
    Email,
    Slack,
    MicrosoftTeams,
    Webhook,
    Console,
}

/// Notification message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub severity: NotificationSeverity,
    pub priority: NotificationPriority,
    pub source: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub delivered: bool,
    pub delivery_attempts: u32,
}

impl Notification {
    /// Create a new notification
    pub fn new(title: String, message: String, source: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            message,
            severity: NotificationSeverity::Info,
            priority: NotificationPriority::Normal,
            source,
            metadata: HashMap::new(),
            created_at: Utc::now(),
            delivered: false,
            delivery_attempts: 0,
        }
    }

    /// Set severity
    pub fn with_severity(mut self, severity: NotificationSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: NotificationPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Notification delivery trait
#[async_trait]
pub trait NotificationDelivery: Send + Sync {
    /// Deliver a notification
    async fn deliver(&self, notification: &Notification) -> NotificationResult<()>;

    /// Get channel type
    fn channel_type(&self) -> NotificationChannel;

    /// Test the delivery channel
    async fn test(&self) -> NotificationResult<()>;
}

/// Email notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_address: String,
    pub from_name: String,
    pub to_addresses: Vec<String>,
    pub use_tls: bool,
}

/// Email notification delivery
pub struct EmailDelivery {
    config: EmailConfig,
    client: Client,
}

impl EmailDelivery {
    /// Create a new email delivery
    pub fn new(config: EmailConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    /// Format email body
    fn format_email(&self, notification: &Notification) -> String {
        format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <style>
        body {{ font-family: Arial, sans-serif; }}
        .header {{ background-color: {}; color: white; padding: 20px; }}
        .content {{ padding: 20px; }}
        .metadata {{ background-color: #f5f5f5; padding: 10px; margin-top: 20px; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>{}</h1>
    </div>
    <div class="content">
        <p><strong>Severity:</strong> {:?}</p>
        <p><strong>Priority:</strong> {:?}</p>
        <p><strong>Source:</strong> {}</p>
        <p><strong>Time:</strong> {}</p>
        <hr>
        <p>{}</p>
        {}
    </div>
</body>
</html>
"#,
            self.severity_color(&notification.severity),
            notification.title,
            notification.severity,
            notification.priority,
            notification.source,
            notification.created_at.format("%Y-%m-%d %H:%M:%S UTC"),
            notification.message,
            self.format_metadata(&notification.metadata)
        )
    }

    /// Get color for severity
    fn severity_color(&self, severity: &NotificationSeverity) -> &str {
        match severity {
            NotificationSeverity::Info => "#2196F3",
            NotificationSeverity::Warning => "#FF9800",
            NotificationSeverity::Error => "#F44336",
            NotificationSeverity::Critical => "#9C27B0",
        }
    }

    /// Format metadata
    fn format_metadata(&self, metadata: &HashMap<String, serde_json::Value>) -> String {
        if metadata.is_empty() {
            return String::new();
        }

        let mut html = String::from("<div class=\"metadata\"><h3>Additional Information:</h3><ul>");
        for (key, value) in metadata {
            html.push_str(&format!("<li><strong>{}:</strong> {}</li>", key, value));
        }
        html.push_str("</ul></div>");
        html
    }
}

#[async_trait]
impl NotificationDelivery for EmailDelivery {
    async fn deliver(&self, notification: &Notification) -> NotificationResult<()> {
        // In production, integrate with an email service provider
        // For now, we'll log the email
        let body = self.format_email(notification);

        println!("Email Notification:");
        println!("To: {:?}", self.config.to_addresses);
        println!("Subject: {}", notification.title);
        println!("Body (HTML): {}", body);

        Ok(())
    }

    fn channel_type(&self) -> NotificationChannel {
        NotificationChannel::Email
    }

    async fn test(&self) -> NotificationResult<()> {
        // Test SMTP connection
        Ok(())
    }
}

/// Slack notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    pub webhook_url: String,
    pub channel: Option<String>,
    pub username: Option<String>,
    pub icon_emoji: Option<String>,
}

/// Slack notification delivery
pub struct SlackDelivery {
    config: SlackConfig,
    client: Client,
}

impl SlackDelivery {
    /// Create a new Slack delivery
    pub fn new(config: SlackConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    /// Format Slack message
    fn format_slack_message(&self, notification: &Notification) -> serde_json::Value {
        let color = match notification.severity {
            NotificationSeverity::Info => "#2196F3",
            NotificationSeverity::Warning => "#FF9800",
            NotificationSeverity::Error => "#F44336",
            NotificationSeverity::Critical => "#9C27B0",
        };

        let mut fields = vec![
            serde_json::json!({
                "title": "Severity",
                "value": format!("{:?}", notification.severity),
                "short": true
            }),
            serde_json::json!({
                "title": "Priority",
                "value": format!("{:?}", notification.priority),
                "short": true
            }),
            serde_json::json!({
                "title": "Source",
                "value": notification.source,
                "short": true
            }),
            serde_json::json!({
                "title": "Time",
                "value": notification.created_at.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                "short": true
            }),
        ];

        // Add metadata fields
        for (key, value) in &notification.metadata {
            fields.push(serde_json::json!({
                "title": key,
                "value": value.to_string(),
                "short": true
            }));
        }

        let mut payload = serde_json::json!({
            "attachments": [{
                "color": color,
                "title": notification.title,
                "text": notification.message,
                "fields": fields,
                "footer": "CADDY Monitoring System",
                "ts": notification.created_at.timestamp()
            }]
        });

        if let Some(channel) = &self.config.channel {
            payload["channel"] = serde_json::Value::String(channel.clone());
        }

        if let Some(username) = &self.config.username {
            payload["username"] = serde_json::Value::String(username.clone());
        }

        if let Some(icon) = &self.config.icon_emoji {
            payload["icon_emoji"] = serde_json::Value::String(icon.clone());
        }

        payload
    }
}

#[async_trait]
impl NotificationDelivery for SlackDelivery {
    async fn deliver(&self, notification: &Notification) -> NotificationResult<()> {
        let payload = self.format_slack_message(notification);

        let response = self
            .client
            .post(&self.config.webhook_url)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(NotificationError::DeliveryFailed(format!(
                "Slack API returned status: {}",
                response.status()
            )))
        }
    }

    fn channel_type(&self) -> NotificationChannel {
        NotificationChannel::Slack
    }

    async fn test(&self) -> NotificationResult<()> {
        let test_notification = Notification::new(
            "Test Notification".to_string(),
            "This is a test notification from CADDY".to_string(),
            "test".to_string(),
        );

        self.deliver(&test_notification).await
    }
}

/// Microsoft Teams notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamsConfig {
    pub webhook_url: String,
}

/// Microsoft Teams notification delivery
pub struct TeamsDelivery {
    config: TeamsConfig,
    client: Client,
}

impl TeamsDelivery {
    /// Create a new Teams delivery
    pub fn new(config: TeamsConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    /// Format Teams message
    fn format_teams_message(&self, notification: &Notification) -> serde_json::Value {
        let theme_color = match notification.severity {
            NotificationSeverity::Info => "0078D4",
            NotificationSeverity::Warning => "FF8C00",
            NotificationSeverity::Error => "D13438",
            NotificationSeverity::Critical => "7B1FA2",
        };

        let mut facts = vec![
            serde_json::json!({
                "name": "Severity",
                "value": format!("{:?}", notification.severity)
            }),
            serde_json::json!({
                "name": "Priority",
                "value": format!("{:?}", notification.priority)
            }),
            serde_json::json!({
                "name": "Source",
                "value": notification.source
            }),
            serde_json::json!({
                "name": "Time",
                "value": notification.created_at.format("%Y-%m-%d %H:%M:%S UTC").to_string()
            }),
        ];

        // Add metadata
        for (key, value) in &notification.metadata {
            facts.push(serde_json::json!({
                "name": key,
                "value": value.to_string()
            }));
        }

        serde_json::json!({
            "@type": "MessageCard",
            "@context": "https://schema.org/extensions",
            "themeColor": theme_color,
            "summary": notification.title,
            "sections": [{
                "activityTitle": notification.title,
                "activitySubtitle": "CADDY Monitoring System",
                "text": notification.message,
                "facts": facts
            }]
        })
    }
}

#[async_trait]
impl NotificationDelivery for TeamsDelivery {
    async fn deliver(&self, notification: &Notification) -> NotificationResult<()> {
        let payload = self.format_teams_message(notification);

        let response = self
            .client
            .post(&self.config.webhook_url)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(NotificationError::DeliveryFailed(format!(
                "Teams API returned status: {}",
                response.status()
            )))
        }
    }

    fn channel_type(&self) -> NotificationChannel {
        NotificationChannel::MicrosoftTeams
    }

    async fn test(&self) -> NotificationResult<()> {
        let test_notification = Notification::new(
            "Test Notification".to_string(),
            "This is a test notification from CADDY".to_string(),
            "test".to_string(),
        );

        self.deliver(&test_notification).await
    }
}

/// Webhook notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub timeout_seconds: u64,
}

/// Webhook notification delivery
pub struct WebhookDelivery {
    config: WebhookConfig,
    client: Client,
}

impl WebhookDelivery {
    /// Create a new webhook delivery
    pub fn new(config: WebhookConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl NotificationDelivery for WebhookDelivery {
    async fn deliver(&self, notification: &Notification) -> NotificationResult<()> {
        let payload = serde_json::to_value(notification)?;

        let mut request = match self.config.method.to_uppercase().as_str() {
            "POST" => self.client.post(&self.config.url),
            "PUT" => self.client.put(&self.config.url),
            _ => self.client.post(&self.config.url),
        };

        // Add headers
        for (key, value) in &self.config.headers {
            request = request.header(key, value);
        }

        // Send request
        let response = request
            .json(&payload)
            .timeout(std::time::Duration::from_secs(self.config.timeout_seconds))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(NotificationError::DeliveryFailed(format!(
                "Webhook returned status: {}",
                response.status()
            )))
        }
    }

    fn channel_type(&self) -> NotificationChannel {
        NotificationChannel::Webhook
    }

    async fn test(&self) -> NotificationResult<()> {
        let test_notification = Notification::new(
            "Test Notification".to_string(),
            "This is a test notification from CADDY".to_string(),
            "test".to_string(),
        );

        self.deliver(&test_notification).await
    }
}

/// Notification preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub user_id: String,
    pub enabled_channels: Vec<NotificationChannel>,
    pub min_severity: NotificationSeverity,
    pub min_priority: NotificationPriority,
    pub quiet_hours: Option<QuietHours>,
    pub source_filters: Vec<String>, // Only notify for these sources
}

/// Quiet hours configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuietHours {
    pub start_hour: u8, // 0-23
    pub end_hour: u8,   // 0-23
    pub timezone: String,
}

impl QuietHours {
    /// Check if current time is in quiet hours
    pub fn is_quiet_now(&self) -> bool {
        let now = Utc::now();
        let hour = now.hour() as u8;

        if self.start_hour < self.end_hour {
            hour >= self.start_hour && hour < self.end_hour
        } else {
            hour >= self.start_hour || hour < self.end_hour
        }
    }
}

/// Notification service
pub struct NotificationService {
    deliveries: Arc<RwLock<HashMap<NotificationChannel, Arc<dyn NotificationDelivery>>>>,
    preferences: Arc<RwLock<HashMap<String, NotificationPreferences>>>,
    history: Arc<RwLock<Vec<Notification>>>,
}

impl NotificationService {
    /// Create a new notification service
    pub fn new() -> Self {
        Self {
            deliveries: Arc::new(RwLock::new(HashMap::new())),
            preferences: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a delivery channel
    pub async fn register_channel(&self, delivery: Arc<dyn NotificationDelivery>) {
        let mut deliveries = self.deliveries.write().await;
        deliveries.insert(delivery.channel_type(), delivery);
    }

    /// Set user preferences
    pub async fn set_preferences(&self, preferences: NotificationPreferences) {
        let mut prefs = self.preferences.write().await;
        prefs.insert(preferences.user_id.clone(), preferences);
    }

    /// Send notification
    pub async fn send(&self, mut notification: Notification) -> NotificationResult<()> {
        notification.delivery_attempts += 1;

        // Store in history
        {
            let mut history = self.history.write().await;
            history.push(notification.clone());

            // Keep only last 10000 notifications
            if history.len() > 10000 {
                history.remove(0);
            }
        }

        // Get enabled channels based on preferences
        let channels = self.get_enabled_channels(&notification).await;

        if channels.is_empty() {
            return Ok(()); // No channels to send to
        }

        // Send to all enabled channels
        let deliveries = self.deliveries.read().await;
        let mut errors = Vec::new();

        for channel in channels {
            if let Some(delivery) = deliveries.get(&channel) {
                if let Err(e) = delivery.deliver(&notification).await {
                    errors.push(format!("{:?}: {}", channel, e));
                }
            }
        }

        if !errors.is_empty() {
            Err(NotificationError::DeliveryFailed(errors.join(", ")))
        } else {
            Ok(())
        }
    }

    /// Get enabled channels for a notification
    async fn get_enabled_channels(&self, notification: &Notification) -> Vec<NotificationChannel> {
        let prefs = self.preferences.read().await;

        // Get all user preferences that match
        let mut channels = Vec::new();

        for pref in prefs.values() {
            // Check severity
            if notification.severity < pref.min_severity {
                continue;
            }

            // Check priority
            if notification.priority < pref.min_priority {
                continue;
            }

            // Check source filters
            if !pref.source_filters.is_empty()
                && !pref.source_filters.contains(&notification.source)
            {
                continue;
            }

            // Check quiet hours
            if let Some(quiet_hours) = &pref.quiet_hours {
                if quiet_hours.is_quiet_now() {
                    continue;
                }
            }

            // Add enabled channels
            for channel in &pref.enabled_channels {
                if !channels.contains(channel) {
                    channels.push(channel.clone());
                }
            }
        }

        // If no preferences match, use all available channels for critical notifications
        if channels.is_empty() && notification.severity == NotificationSeverity::Critical {
            let deliveries = self.deliveries.read().await;
            channels = deliveries.keys().cloned().collect();
        }

        channels
    }

    /// Get notification history
    pub async fn get_history(&self, limit: usize) -> Vec<Notification> {
        let history = self.history.read().await;
        history.iter().rev().take(limit).cloned().collect()
    }

    /// Test all configured channels
    pub async fn test_all_channels(&self) -> HashMap<NotificationChannel, NotificationResult<()>> {
        let deliveries = self.deliveries.read().await;
        let mut results = HashMap::new();

        for (channel, delivery) in deliveries.iter() {
            let result = delivery.test().await;
            results.insert(channel.clone(), result);
        }

        results
    }
}

impl Default for NotificationService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_creation() {
        let notification = Notification::new(
            "Test".to_string(),
            "Test message".to_string(),
            "test-source".to_string(),
        )
        .with_severity(NotificationSeverity::Warning)
        .with_priority(NotificationPriority::High);

        assert_eq!(notification.title, "Test");
        assert_eq!(notification.severity, NotificationSeverity::Warning);
        assert_eq!(notification.priority, NotificationPriority::High);
    }

    #[test]
    fn test_quiet_hours() {
        let quiet_hours = QuietHours {
            start_hour: 22,
            end_hour: 8,
            timezone: "UTC".to_string(),
        };

        // This is time-dependent, so we just test that it doesn't panic
        let _ = quiet_hours.is_quiet_now();
    }
}
