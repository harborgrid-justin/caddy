//! Review and rating system
//!
//! This module provides a comprehensive review system including ratings,
//! moderation, developer responses, and abuse reporting.

use super::{MarketplaceError, Result};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Review rating (1-5 stars)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReviewRating(u8);

impl ReviewRating {
    /// Create a new rating (1-5)
    pub fn new(rating: u8) -> Result<Self> {
        if rating >= 1 && rating <= 5 {
            Ok(Self(rating))
        } else {
            Err(MarketplaceError::ReviewError(
                "Rating must be between 1 and 5".to_string()
            ))
        }
    }

    /// Get rating value
    pub fn value(&self) -> u8 {
        self.0
    }

    /// Get rating as float
    pub fn as_float(&self) -> f32 {
        self.0 as f32
    }
}

impl Default for ReviewRating {
    fn default() -> Self {
        Self(5)
    }
}

/// Plugin review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    /// Review ID
    pub id: Uuid,

    /// Plugin ID
    pub plugin_id: Uuid,

    /// Reviewer user ID
    pub user_id: Uuid,

    /// Reviewer name
    pub reviewer_name: String,

    /// Rating (1-5 stars)
    pub rating: ReviewRating,

    /// Review title
    pub title: String,

    /// Review content
    pub content: String,

    /// Verified purchase
    pub verified_purchase: bool,

    /// Review creation date
    pub created_at: DateTime<Utc>,

    /// Last updated date
    pub updated_at: DateTime<Utc>,

    /// Helpful votes count
    pub helpful_count: u32,

    /// Not helpful votes count
    pub not_helpful_count: u32,

    /// Developer response
    pub developer_response: Option<DeveloperResponse>,

    /// Moderation status
    pub moderation_status: ModerationStatus,

    /// CADDY version used
    pub caddy_version: Option<String>,

    /// Plugin version reviewed
    pub plugin_version: String,
}

impl Review {
    /// Create a new review
    pub fn new(
        plugin_id: Uuid,
        user_id: Uuid,
        reviewer_name: String,
        rating: ReviewRating,
        title: String,
        content: String,
        plugin_version: String,
    ) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),
            plugin_id,
            user_id,
            reviewer_name,
            rating,
            title,
            content,
            verified_purchase: false,
            created_at: now,
            updated_at: now,
            helpful_count: 0,
            not_helpful_count: 0,
            developer_response: None,
            moderation_status: ModerationStatus::Pending,
            caddy_version: None,
            plugin_version,
        }
    }

    /// Update review content
    pub fn update(&mut self, title: String, content: String, rating: ReviewRating) {
        self.title = title;
        self.content = content;
        self.rating = rating;
        self.updated_at = Utc::now();
    }

    /// Add developer response
    pub fn add_developer_response(&mut self, response: DeveloperResponse) {
        self.developer_response = Some(response);
        self.updated_at = Utc::now();
    }

    /// Vote helpful
    pub fn vote_helpful(&mut self) {
        self.helpful_count += 1;
    }

    /// Vote not helpful
    pub fn vote_not_helpful(&mut self) {
        self.not_helpful_count += 1;
    }

    /// Get helpfulness score
    pub fn helpfulness_score(&self) -> f32 {
        let total = (self.helpful_count + self.not_helpful_count) as f32;
        if total == 0.0 {
            0.0
        } else {
            self.helpful_count as f32 / total
        }
    }

    /// Check if review is approved
    pub fn is_approved(&self) -> bool {
        self.moderation_status == ModerationStatus::Approved
    }
}

/// Developer response to review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperResponse {
    /// Response ID
    pub id: Uuid,

    /// Developer user ID
    pub developer_id: Uuid,

    /// Developer name
    pub developer_name: String,

    /// Response content
    pub content: String,

    /// Response creation date
    pub created_at: DateTime<Utc>,

    /// Last updated date
    pub updated_at: DateTime<Utc>,
}

impl DeveloperResponse {
    /// Create a new developer response
    pub fn new(developer_id: Uuid, developer_name: String, content: String) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),
            developer_id,
            developer_name,
            content,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update response
    pub fn update(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
    }
}

/// Moderation status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModerationStatus {
    /// Pending moderation
    Pending,

    /// Approved and visible
    Approved,

    /// Rejected (policy violation)
    Rejected,

    /// Flagged for review
    Flagged,

    /// Removed by moderator
    Removed,
}

/// Abuse report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbuseReport {
    /// Report ID
    pub id: Uuid,

    /// Review being reported
    pub review_id: Uuid,

    /// Reporter user ID
    pub reporter_id: Uuid,

    /// Report reason
    pub reason: AbuseReason,

    /// Additional details
    pub details: Option<String>,

    /// Report creation date
    pub created_at: DateTime<Utc>,

    /// Report status
    pub status: ReportStatus,

    /// Moderator notes
    pub moderator_notes: Option<String>,

    /// Resolved by (moderator ID)
    pub resolved_by: Option<Uuid>,

    /// Resolution date
    pub resolved_at: Option<DateTime<Utc>>,
}

impl AbuseReport {
    /// Create a new abuse report
    pub fn new(review_id: Uuid, reporter_id: Uuid, reason: AbuseReason, details: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            review_id,
            reporter_id,
            reason,
            details,
            created_at: Utc::now(),
            status: ReportStatus::Open,
            moderator_notes: None,
            resolved_by: None,
            resolved_at: None,
        }
    }

    /// Resolve report
    pub fn resolve(&mut self, moderator_id: Uuid, notes: Option<String>) {
        self.status = ReportStatus::Resolved;
        self.moderator_notes = notes;
        self.resolved_by = Some(moderator_id);
        self.resolved_at = Some(Utc::now());
    }
}

/// Abuse report reason
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AbuseReason {
    /// Spam or advertising
    Spam,

    /// Offensive language
    OffensiveLanguage,

    /// Harassment
    Harassment,

    /// Fake review
    FakeReview,

    /// Off-topic
    OffTopic,

    /// Copyright violation
    Copyright,

    /// Other reason
    Other,
}

impl AbuseReason {
    /// Get display name
    pub fn display_name(&self) -> &str {
        match self {
            Self::Spam => "Spam or Advertising",
            Self::OffensiveLanguage => "Offensive Language",
            Self::Harassment => "Harassment",
            Self::FakeReview => "Fake Review",
            Self::OffTopic => "Off-Topic",
            Self::Copyright => "Copyright Violation",
            Self::Other => "Other",
        }
    }
}

/// Report status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReportStatus {
    /// Open and pending review
    Open,

    /// Under investigation
    UnderReview,

    /// Resolved
    Resolved,

    /// Dismissed (no action taken)
    Dismissed,
}

/// Review moderation
#[derive(Debug)]
pub struct ReviewModeration {
    /// Reviews pending moderation
    pending_queue: Arc<RwLock<Vec<Uuid>>>,

    /// All reviews (review_id -> review)
    reviews: Arc<RwLock<HashMap<Uuid, Review>>>,

    /// Moderation rules
    rules: Arc<RwLock<ModerationRules>>,
}

impl ReviewModeration {
    /// Create a new review moderation system
    pub fn new() -> Self {
        Self {
            pending_queue: Arc::new(RwLock::new(Vec::new())),
            reviews: Arc::new(RwLock::new(HashMap::new())),
            rules: Arc::new(RwLock::new(ModerationRules::default())),
        }
    }

    /// Submit review for moderation
    pub fn submit_review(&self, review: Review) -> Result<Uuid> {
        let review_id = review.id;

        // Auto-moderate based on rules
        let mut review = review;
        self.auto_moderate(&mut review);

        if review.moderation_status == ModerationStatus::Pending {
            self.pending_queue.write().push(review_id);
        }

        self.reviews.write().insert(review_id, review);

        Ok(review_id)
    }

    /// Auto-moderate review
    fn auto_moderate(&self, review: &mut Review) {
        let rules = self.rules.read();

        // Check for banned words
        let content_lower = format!("{} {}", review.title, review.content).to_lowercase();

        for banned_word in &rules.banned_words {
            if content_lower.contains(banned_word) {
                review.moderation_status = ModerationStatus::Flagged;
                return;
            }
        }

        // Check minimum content length
        if review.content.len() < rules.min_content_length {
            review.moderation_status = ModerationStatus::Flagged;
            return;
        }

        // Auto-approve if no issues
        if rules.auto_approve_verified_purchases && review.verified_purchase {
            review.moderation_status = ModerationStatus::Approved;
        } else if rules.auto_approve_enabled {
            review.moderation_status = ModerationStatus::Approved;
        }
    }

    /// Approve review
    pub fn approve_review(&self, review_id: Uuid) -> Result<()> {
        let mut reviews = self.reviews.write();

        if let Some(review) = reviews.get_mut(&review_id) {
            review.moderation_status = ModerationStatus::Approved;
            self.pending_queue.write().retain(|&id| id != review_id);
            Ok(())
        } else {
            Err(MarketplaceError::ReviewError("Review not found".to_string()))
        }
    }

    /// Reject review
    pub fn reject_review(&self, review_id: Uuid) -> Result<()> {
        let mut reviews = self.reviews.write();

        if let Some(review) = reviews.get_mut(&review_id) {
            review.moderation_status = ModerationStatus::Rejected;
            self.pending_queue.write().retain(|&id| id != review_id);
            Ok(())
        } else {
            Err(MarketplaceError::ReviewError("Review not found".to_string()))
        }
    }

    /// Get pending reviews
    pub fn get_pending(&self) -> Vec<Review> {
        let pending_queue = self.pending_queue.read();
        let reviews = self.reviews.read();

        pending_queue.iter()
            .filter_map(|id| reviews.get(id).cloned())
            .collect()
    }
}

impl Default for ReviewModeration {
    fn default() -> Self {
        Self::new()
    }
}

/// Moderation rules
#[derive(Debug, Clone)]
struct ModerationRules {
    /// Minimum content length
    min_content_length: usize,

    /// Banned words
    banned_words: Vec<String>,

    /// Auto-approve enabled
    auto_approve_enabled: bool,

    /// Auto-approve verified purchases
    auto_approve_verified_purchases: bool,
}

impl Default for ModerationRules {
    fn default() -> Self {
        Self {
            min_content_length: 10,
            banned_words: vec![
                "spam".to_string(),
                "viagra".to_string(),
            ],
            auto_approve_enabled: false,
            auto_approve_verified_purchases: true,
        }
    }
}

/// Review system
#[derive(Debug)]
pub struct ReviewSystem {
    /// All reviews indexed by plugin (plugin_id -> review_ids)
    plugin_reviews: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,

    /// All reviews indexed by user (user_id -> review_ids)
    user_reviews: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,

    /// Review storage (review_id -> review)
    reviews: Arc<RwLock<HashMap<Uuid, Review>>>,

    /// Abuse reports (report_id -> report)
    abuse_reports: Arc<RwLock<HashMap<Uuid, AbuseReport>>>,

    /// Review moderation
    moderation: ReviewModeration,
}

impl ReviewSystem {
    /// Create a new review system
    pub fn new() -> Self {
        Self {
            plugin_reviews: Arc::new(RwLock::new(HashMap::new())),
            user_reviews: Arc::new(RwLock::new(HashMap::new())),
            reviews: Arc::new(RwLock::new(HashMap::new())),
            abuse_reports: Arc::new(RwLock::new(HashMap::new())),
            moderation: ReviewModeration::new(),
        }
    }

    /// Submit a review
    pub fn submit_review(&self, review: Review) -> Result<Uuid> {
        let plugin_id = review.plugin_id;
        let user_id = review.user_id;
        let review_id = review.id;

        // Check if user already reviewed this plugin
        let user_reviews = self.user_reviews.read();
        if let Some(reviews) = user_reviews.get(&user_id) {
            let existing_reviews = self.reviews.read();
            for &existing_id in reviews {
                if let Some(existing) = existing_reviews.get(&existing_id) {
                    if existing.plugin_id == plugin_id {
                        return Err(MarketplaceError::ReviewError(
                            "User already reviewed this plugin".to_string()
                        ));
                    }
                }
            }
        }
        drop(user_reviews);

        // Submit for moderation
        self.moderation.submit_review(review.clone())?;

        // Index review
        self.plugin_reviews.write()
            .entry(plugin_id)
            .or_insert_with(Vec::new)
            .push(review_id);

        self.user_reviews.write()
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(review_id);

        self.reviews.write().insert(review_id, review);

        Ok(review_id)
    }

    /// Update a review
    pub fn update_review(
        &self,
        review_id: Uuid,
        title: String,
        content: String,
        rating: ReviewRating,
    ) -> Result<()> {
        let mut reviews = self.reviews.write();

        if let Some(review) = reviews.get_mut(&review_id) {
            review.update(title, content, rating);
            Ok(())
        } else {
            Err(MarketplaceError::ReviewError("Review not found".to_string()))
        }
    }

    /// Add developer response
    pub fn add_developer_response(&self, review_id: Uuid, response: DeveloperResponse) -> Result<()> {
        let mut reviews = self.reviews.write();

        if let Some(review) = reviews.get_mut(&review_id) {
            review.add_developer_response(response);
            Ok(())
        } else {
            Err(MarketplaceError::ReviewError("Review not found".to_string()))
        }
    }

    /// Get reviews for plugin
    pub fn get_plugin_reviews(&self, plugin_id: Uuid) -> Vec<Review> {
        let plugin_reviews = self.plugin_reviews.read();
        let reviews = self.reviews.read();

        if let Some(review_ids) = plugin_reviews.get(&plugin_id) {
            review_ids.iter()
                .filter_map(|id| reviews.get(id))
                .filter(|r| r.is_approved())
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get reviews by user
    pub fn get_user_reviews(&self, user_id: Uuid) -> Vec<Review> {
        let user_reviews = self.user_reviews.read();
        let reviews = self.reviews.read();

        if let Some(review_ids) = user_reviews.get(&user_id) {
            review_ids.iter()
                .filter_map(|id| reviews.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Report abuse
    pub fn report_abuse(&self, report: AbuseReport) -> Result<Uuid> {
        let report_id = report.id;
        self.abuse_reports.write().insert(report_id, report);
        Ok(report_id)
    }

    /// Get abuse reports
    pub fn get_abuse_reports(&self, status: Option<ReportStatus>) -> Vec<AbuseReport> {
        let reports = self.abuse_reports.read();

        if let Some(status) = status {
            reports.values()
                .filter(|r| r.status == status)
                .cloned()
                .collect()
        } else {
            reports.values().cloned().collect()
        }
    }

    /// Calculate average rating for plugin
    pub fn calculate_average_rating(&self, plugin_id: Uuid) -> (f32, u32) {
        let reviews = self.get_plugin_reviews(plugin_id);

        if reviews.is_empty() {
            return (0.0, 0);
        }

        let total: f32 = reviews.iter().map(|r| r.rating.as_float()).sum();
        let count = reviews.len() as u32;
        let average = total / count as f32;

        (average, count)
    }
}

impl Default for ReviewSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_rating() {
        assert!(ReviewRating::new(0).is_err());
        assert!(ReviewRating::new(1).is_ok());
        assert!(ReviewRating::new(5).is_ok());
        assert!(ReviewRating::new(6).is_err());
    }

    #[test]
    fn test_review_creation() {
        let review = Review::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Reviewer".to_string(),
            ReviewRating::new(5).unwrap(),
            "Great!".to_string(),
            "This plugin is awesome!".to_string(),
            "1.0.0".to_string(),
        );

        assert_eq!(review.rating.value(), 5);
        assert_eq!(review.helpful_count, 0);
    }

    #[test]
    fn test_helpfulness_score() {
        let mut review = Review::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Reviewer".to_string(),
            ReviewRating::new(5).unwrap(),
            "Great!".to_string(),
            "This plugin is awesome!".to_string(),
            "1.0.0".to_string(),
        );

        assert_eq!(review.helpfulness_score(), 0.0);

        review.vote_helpful();
        review.vote_helpful();
        review.vote_not_helpful();

        assert!((review.helpfulness_score() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_review_system() -> Result<()> {
        let system = ReviewSystem::new();

        let plugin_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let review = Review::new(
            plugin_id,
            user_id,
            "Reviewer".to_string(),
            ReviewRating::new(5).unwrap(),
            "Great plugin!".to_string(),
            "This plugin works really well!".to_string(),
            "1.0.0".to_string(),
        );

        let review_id = system.submit_review(review)?;

        // Approve the review
        system.moderation.approve_review(review_id)?;

        let reviews = system.get_plugin_reviews(plugin_id);
        assert_eq!(reviews.len(), 1);

        // Test duplicate review prevention
        let duplicate = Review::new(
            plugin_id,
            user_id,
            "Reviewer".to_string(),
            ReviewRating::new(4).unwrap(),
            "Updated".to_string(),
            "Updated review".to_string(),
            "1.0.1".to_string(),
        );

        assert!(system.submit_review(duplicate).is_err());

        Ok(())
    }
}
