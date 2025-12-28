use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum NotificationError {
    #[error("Notification not found: {0}")]
    NotFound(Uuid),
    #[error("Template not found: {0}")]
    TemplateNotFound(String),
    #[error("Invalid template: {0}")]
    InvalidTemplate(String),
    #[error("Delivery failed: {0}")]
    DeliveryFailed(String),
    #[error("Invalid channel: {0}")]
    InvalidChannel(String),
    #[error("Recipient validation failed: {0}")]
    InvalidRecipient(String),
}

pub type NotificationResult<T> = Result<T, NotificationError>;

/// Notification channel types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationChannel {
    /// Email notification
    Email,
    /// In-app notification
    InApp,
    /// Webhook/HTTP callback
    Webhook,
    /// SMS notification
    Sms,
    /// Push notification (mobile/desktop)
    Push,
    /// Slack integration
    Slack,
    /// Microsoft Teams integration
    Teams,
}

impl NotificationChannel {
    /// Get channel name as string
    pub fn as_str(&self) -> &str {
        match self {
            Self::Email => "email",
            Self::InApp => "in_app",
            Self::Webhook => "webhook",
            Self::Sms => "sms",
            Self::Push => "push",
            Self::Slack => "slack",
            Self::Teams => "teams",
        }
    }
}

/// Notification priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Urgent,
}

/// Notification status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationStatus {
    /// Pending delivery
    Pending,
    /// Currently being sent
    Sending,
    /// Successfully delivered
    Delivered,
    /// Delivery failed
    Failed,
    /// Cancelled before delivery
    Cancelled,
    /// Read by recipient
    Read,
}

/// Notification template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationTemplate {
    /// Template ID
    pub id: Uuid,
    /// Template name/identifier
    pub name: String,
    /// Template description
    pub description: String,
    /// Category for organization
    pub category: String,
    /// Subject template (for email, etc.)
    pub subject_template: String,
    /// Body template
    pub body_template: String,
    /// Supported channels
    pub channels: Vec<NotificationChannel>,
    /// Default priority
    pub default_priority: NotificationPriority,
    /// Template variables
    pub variables: Vec<String>,
    /// Is this template active
    pub is_active: bool,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Created by user
    pub created_by: Uuid,
}

impl NotificationTemplate {
    /// Create a new notification template
    pub fn new(name: String, description: String, category: String, created_by: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            category,
            subject_template: String::new(),
            body_template: String::new(),
            channels: vec![NotificationChannel::Email, NotificationChannel::InApp],
            default_priority: NotificationPriority::Normal,
            variables: Vec::new(),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by,
        }
    }

    /// Render template with variables
    pub fn render(&self, variables: &HashMap<String, String>) -> NotificationResult<RenderedTemplate> {
        let mut subject = self.subject_template.clone();
        let mut body = self.body_template.clone();

        // Simple variable substitution - in production would use proper template engine
        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            subject = subject.replace(&placeholder, value);
            body = body.replace(&placeholder, value);
        }

        Ok(RenderedTemplate {
            subject,
            body,
            variables: variables.clone(),
        })
    }

    /// Validate template has all required variables
    pub fn validate_variables(&self, variables: &HashMap<String, String>) -> NotificationResult<()> {
        for required_var in &self.variables {
            if !variables.contains_key(required_var) {
                return Err(NotificationError::InvalidTemplate(
                    format!("Missing required variable: {}", required_var),
                ));
            }
        }
        Ok(())
    }
}

/// Rendered template result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedTemplate {
    /// Rendered subject
    pub subject: String,
    /// Rendered body
    pub body: String,
    /// Variables used
    pub variables: HashMap<String, String>,
}

/// Notification recipient
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRecipient {
    /// User ID
    pub user_id: Option<Uuid>,
    /// Email address (if not using user_id)
    pub email: Option<String>,
    /// Phone number (for SMS)
    pub phone: Option<String>,
    /// Webhook URL
    pub webhook_url: Option<String>,
    /// Display name
    pub display_name: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl NotificationRecipient {
    /// Create recipient from user ID
    pub fn from_user(user_id: Uuid) -> Self {
        Self {
            user_id: Some(user_id),
            email: None,
            phone: None,
            webhook_url: None,
            display_name: None,
            metadata: HashMap::new(),
        }
    }

    /// Create recipient from email
    pub fn from_email(email: String) -> Self {
        Self {
            user_id: None,
            email: Some(email),
            phone: None,
            webhook_url: None,
            display_name: None,
            metadata: HashMap::new(),
        }
    }

    /// Create recipient from webhook URL
    pub fn from_webhook(url: String) -> Self {
        Self {
            user_id: None,
            email: None,
            phone: None,
            webhook_url: Some(url),
            display_name: None,
            metadata: HashMap::new(),
        }
    }

    /// Validate recipient has necessary information for channel
    pub fn validate_for_channel(&self, channel: NotificationChannel) -> NotificationResult<()> {
        match channel {
            NotificationChannel::Email => {
                if self.email.is_none() && self.user_id.is_none() {
                    return Err(NotificationError::InvalidRecipient(
                        "Email or user_id required for email channel".to_string(),
                    ));
                }
            }
            NotificationChannel::Sms => {
                if self.phone.is_none() && self.user_id.is_none() {
                    return Err(NotificationError::InvalidRecipient(
                        "Phone number or user_id required for SMS channel".to_string(),
                    ));
                }
            }
            NotificationChannel::Webhook => {
                if self.webhook_url.is_none() {
                    return Err(NotificationError::InvalidRecipient(
                        "Webhook URL required for webhook channel".to_string(),
                    ));
                }
            }
            NotificationChannel::InApp => {
                if self.user_id.is_none() {
                    return Err(NotificationError::InvalidRecipient(
                        "User ID required for in-app channel".to_string(),
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    }
}

/// Notification message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Notification ID
    pub id: Uuid,
    /// Workflow ID (if from workflow)
    pub workflow_id: Option<Uuid>,
    /// Template used
    pub template_name: Option<String>,
    /// Subject/title
    pub subject: String,
    /// Message body
    pub body: String,
    /// Recipients
    pub recipients: Vec<NotificationRecipient>,
    /// Delivery channels
    pub channels: Vec<NotificationChannel>,
    /// Priority
    pub priority: NotificationPriority,
    /// Current status
    pub status: NotificationStatus,
    /// Sender user ID
    pub sender_id: Option<Uuid>,
    /// Scheduled send time (None = send immediately)
    pub scheduled_at: Option<DateTime<Utc>>,
    /// Additional data/context
    pub data: HashMap<String, serde_json::Value>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Sent timestamp
    pub sent_at: Option<DateTime<Utc>>,
    /// Delivery attempts
    pub delivery_attempts: Vec<DeliveryAttempt>,
    /// Read timestamp (for in-app notifications)
    pub read_at: Option<DateTime<Utc>>,
}

impl Notification {
    /// Create a new notification
    pub fn new(subject: String, body: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            workflow_id: None,
            template_name: None,
            subject,
            body,
            recipients: Vec::new(),
            channels: vec![NotificationChannel::InApp],
            priority: NotificationPriority::Normal,
            status: NotificationStatus::Pending,
            sender_id: None,
            scheduled_at: None,
            data: HashMap::new(),
            created_at: Utc::now(),
            sent_at: None,
            delivery_attempts: Vec::new(),
            read_at: None,
        }
    }

    /// Create from template
    pub fn from_template(
        template: &NotificationTemplate,
        variables: HashMap<String, String>,
    ) -> NotificationResult<Self> {
        template.validate_variables(&variables)?;
        let rendered = template.render(&variables)?;

        let mut notification = Self::new(rendered.subject, rendered.body);
        notification.template_name = Some(template.name.clone());
        notification.channels = template.channels.clone();
        notification.priority = template.default_priority;

        Ok(notification)
    }

    /// Add a recipient
    pub fn add_recipient(&mut self, recipient: NotificationRecipient) {
        self.recipients.push(recipient);
    }

    /// Set workflow context
    pub fn with_workflow(mut self, workflow_id: Uuid) -> Self {
        self.workflow_id = Some(workflow_id);
        self
    }

    /// Set sender
    pub fn with_sender(mut self, sender_id: Uuid) -> Self {
        self.sender_id = Some(sender_id);
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: NotificationPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set channels
    pub fn with_channels(mut self, channels: Vec<NotificationChannel>) -> Self {
        self.channels = channels;
        self
    }

    /// Schedule for later delivery
    pub fn schedule(mut self, scheduled_at: DateTime<Utc>) -> Self {
        self.scheduled_at = Some(scheduled_at);
        self
    }

    /// Check if notification should be sent now
    pub fn should_send_now(&self) -> bool {
        match self.scheduled_at {
            Some(scheduled_time) => Utc::now() >= scheduled_time,
            None => true,
        }
    }

    /// Mark as read
    pub fn mark_read(&mut self) {
        if self.read_at.is_none() {
            self.read_at = Some(Utc::now());
            self.status = NotificationStatus::Read;
        }
    }

    /// Record delivery attempt
    pub fn record_attempt(&mut self, channel: NotificationChannel, success: bool, error: Option<String>) {
        let attempt = DeliveryAttempt {
            id: Uuid::new_v4(),
            channel,
            attempted_at: Utc::now(),
            success,
            error_message: error,
        };
        self.delivery_attempts.push(attempt);

        if success {
            self.status = NotificationStatus::Delivered;
            self.sent_at = Some(Utc::now());
        } else if self.delivery_attempts.iter().all(|a| !a.success) {
            self.status = NotificationStatus::Failed;
        }
    }
}

/// Delivery attempt record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryAttempt {
    /// Attempt ID
    pub id: Uuid,
    /// Channel used
    pub channel: NotificationChannel,
    /// Attempt timestamp
    pub attempted_at: DateTime<Utc>,
    /// Success status
    pub success: bool,
    /// Error message if failed
    pub error_message: Option<String>,
}

/// User notification preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    /// User ID
    pub user_id: Uuid,
    /// Enabled channels
    pub enabled_channels: HashMap<NotificationChannel, bool>,
    /// Notification category preferences
    pub category_preferences: HashMap<String, ChannelPreferences>,
    /// Quiet hours (no notifications during this time)
    pub quiet_hours: Option<QuietHours>,
    /// Digest settings (batch notifications)
    pub digest_settings: Option<DigestSettings>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

impl NotificationPreferences {
    /// Create default preferences for user
    pub fn default_for_user(user_id: Uuid) -> Self {
        let mut enabled_channels = HashMap::new();
        enabled_channels.insert(NotificationChannel::Email, true);
        enabled_channels.insert(NotificationChannel::InApp, true);
        enabled_channels.insert(NotificationChannel::Push, false);
        enabled_channels.insert(NotificationChannel::Sms, false);
        enabled_channels.insert(NotificationChannel::Webhook, false);
        enabled_channels.insert(NotificationChannel::Slack, false);
        enabled_channels.insert(NotificationChannel::Teams, false);

        Self {
            user_id,
            enabled_channels,
            category_preferences: HashMap::new(),
            quiet_hours: None,
            digest_settings: None,
            updated_at: Utc::now(),
        }
    }

    /// Check if channel is enabled for user
    pub fn is_channel_enabled(&self, channel: NotificationChannel) -> bool {
        self.enabled_channels.get(&channel).copied().unwrap_or(false)
    }

    /// Check if notification should be sent based on preferences
    pub fn should_send(&self, notification: &Notification, channel: NotificationChannel) -> bool {
        // Check if channel is enabled
        if !self.is_channel_enabled(channel) {
            return false;
        }

        // Check quiet hours
        if let Some(quiet_hours) = &self.quiet_hours {
            if quiet_hours.is_quiet_time() && notification.priority != NotificationPriority::Urgent {
                return false;
            }
        }

        true
    }
}

/// Channel preferences for a category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelPreferences {
    /// Enabled channels for this category
    pub channels: Vec<NotificationChannel>,
    /// Minimum priority to send
    pub min_priority: NotificationPriority,
}

/// Quiet hours configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuietHours {
    /// Start hour (0-23)
    pub start_hour: u32,
    /// End hour (0-23)
    pub end_hour: u32,
    /// Days of week (0 = Sunday, 6 = Saturday)
    pub days: Vec<u32>,
}

impl QuietHours {
    /// Check if current time is during quiet hours
    pub fn is_quiet_time(&self) -> bool {
        let now = Utc::now();
        let hour = now.hour();
        let weekday = now.weekday().num_days_from_sunday();

        // Check if current day is in quiet days
        if !self.days.contains(&weekday) {
            return false;
        }

        // Check if current hour is in quiet hours
        if self.start_hour <= self.end_hour {
            hour >= self.start_hour && hour < self.end_hour
        } else {
            // Wraps around midnight
            hour >= self.start_hour || hour < self.end_hour
        }
    }
}

/// Digest settings for batching notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestSettings {
    /// Whether digest is enabled
    pub enabled: bool,
    /// Digest frequency in hours
    pub frequency_hours: u32,
    /// Categories to include in digest
    pub categories: Vec<String>,
    /// Preferred delivery time (hour of day, 0-23)
    pub preferred_hour: Option<u32>,
}

/// Notification delivery tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryTracking {
    /// Notification ID
    pub notification_id: Uuid,
    /// Recipient
    pub recipient: NotificationRecipient,
    /// Channel
    pub channel: NotificationChannel,
    /// Delivery status
    pub status: NotificationStatus,
    /// Delivered at
    pub delivered_at: Option<DateTime<Utc>>,
    /// Opened/read at
    pub opened_at: Option<DateTime<Utc>>,
    /// Clicked at (for email/web notifications)
    pub clicked_at: Option<DateTime<Utc>>,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

impl DeliveryTracking {
    /// Create new delivery tracking
    pub fn new(
        notification_id: Uuid,
        recipient: NotificationRecipient,
        channel: NotificationChannel,
    ) -> Self {
        Self {
            notification_id,
            recipient,
            channel,
            status: NotificationStatus::Pending,
            delivered_at: None,
            opened_at: None,
            clicked_at: None,
            error_message: None,
            metadata: HashMap::new(),
        }
    }

    /// Mark as delivered
    pub fn mark_delivered(&mut self) {
        self.status = NotificationStatus::Delivered;
        self.delivered_at = Some(Utc::now());
    }

    /// Mark as opened
    pub fn mark_opened(&mut self) {
        self.status = NotificationStatus::Read;
        self.opened_at = Some(Utc::now());
    }

    /// Mark as clicked
    pub fn mark_clicked(&mut self) {
        self.clicked_at = Some(Utc::now());
    }

    /// Mark as failed
    pub fn mark_failed(&mut self, error: String) {
        self.status = NotificationStatus::Failed;
        self.error_message = Some(error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_template() {
        let user_id = Uuid::new_v4();
        let mut template = NotificationTemplate::new(
            "approval_request".to_string(),
            "Approval Request Template".to_string(),
            "workflow".to_string(),
            user_id,
        );

        template.subject_template = "Approval needed for {{entity_name}}".to_string();
        template.body_template = "Please review and approve {{entity_name}} by {{deadline}}".to_string();
        template.variables = vec!["entity_name".to_string(), "deadline".to_string()];

        let mut vars = HashMap::new();
        vars.insert("entity_name".to_string(), "Drawing-123".to_string());
        vars.insert("deadline".to_string(), "2024-12-31".to_string());

        let rendered = template.render(&vars).unwrap();
        assert!(rendered.subject.contains("Drawing-123"));
        assert!(rendered.body.contains("2024-12-31"));
    }

    #[test]
    fn test_notification_creation() {
        let notification = Notification::new(
            "Test Subject".to_string(),
            "Test Body".to_string(),
        );

        assert_eq!(notification.status, NotificationStatus::Pending);
        assert_eq!(notification.priority, NotificationPriority::Normal);
    }

    #[test]
    fn test_notification_preferences() {
        let user_id = Uuid::new_v4();
        let prefs = NotificationPreferences::default_for_user(user_id);

        assert!(prefs.is_channel_enabled(NotificationChannel::Email));
        assert!(prefs.is_channel_enabled(NotificationChannel::InApp));
        assert!(!prefs.is_channel_enabled(NotificationChannel::Sms));
    }

    #[test]
    fn test_recipient_validation() {
        let recipient = NotificationRecipient::from_email("test@example.com".to_string());
        assert!(recipient.validate_for_channel(NotificationChannel::Email).is_ok());

        let webhook_recipient = NotificationRecipient::from_webhook("https://example.com/hook".to_string());
        assert!(webhook_recipient.validate_for_channel(NotificationChannel::Webhook).is_ok());
    }
}
