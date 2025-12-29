//! # Accessibility Remediation Engine
//!
//! AI-powered automated remediation suggestions, one-click fixes, bulk remediation,
//! and priority-based remediation queuing for accessibility issues.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque, BTreeMap};
use thiserror::Error;
use chrono::{DateTime, Utc};

use super::rules::{RuleViolation, RuleSeverity};
use super::scanner::AccessibilityViolation;
use super::analyzer::{
    StructuralIssue, SemanticIssue, AriaIssue, IssueSeverity,
    AltTextIssue, LinkIssue, FormIssue, HeadingIssue, NavigationIssue,
};

/// Errors that can occur during remediation
#[derive(Error, Debug)]
pub enum RemediationError {
    #[error("Fix failed: {0}")]
    FixFailed(String),

    #[error("Invalid fix: {0}")]
    InvalidFix(String),

    #[error("Element not found: {0}")]
    ElementNotFound(String),

    #[error("Fix not applicable: {0}")]
    NotApplicable(String),

    #[error("AI service error: {0}")]
    AiServiceError(String),
}

/// Type of remediation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RemediationType {
    /// Automated fix that can be applied automatically
    Automatic,
    /// Semi-automatic fix that requires user confirmation
    SemiAutomatic,
    /// Manual fix that requires user intervention
    Manual,
    /// AI-suggested fix
    AiSuggested,
}

/// Priority of a fix
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum FixPriority {
    /// Critical - must be fixed immediately (blocks compliance)
    Critical = 4,
    /// High - should be fixed soon (major compliance issue)
    High = 3,
    /// Medium - should be addressed (minor compliance issue)
    Medium = 2,
    /// Low - nice to have (improvement)
    Low = 1,
    /// Info - informational only
    Info = 0,
}

impl From<RuleSeverity> for FixPriority {
    fn from(severity: RuleSeverity) -> Self {
        match severity {
            RuleSeverity::Critical => FixPriority::Critical,
            RuleSeverity::Major => FixPriority::High,
            RuleSeverity::Minor => FixPriority::Medium,
            RuleSeverity::Info => FixPriority::Info,
        }
    }
}

impl From<IssueSeverity> for FixPriority {
    fn from(severity: IssueSeverity) -> Self {
        match severity {
            IssueSeverity::Critical => FixPriority::Critical,
            IssueSeverity::High => FixPriority::High,
            IssueSeverity::Medium => FixPriority::Medium,
            IssueSeverity::Low => FixPriority::Low,
            IssueSeverity::Info => FixPriority::Info,
        }
    }
}

/// A remediation suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationSuggestion {
    /// Unique identifier for this suggestion
    pub id: String,

    /// Issue identifier this fixes
    pub issue_id: String,

    /// Human-readable title
    pub title: String,

    /// Detailed description
    pub description: String,

    /// Type of remediation
    pub remediation_type: RemediationType,

    /// Priority level
    pub priority: FixPriority,

    /// Element selector
    pub element_selector: String,

    /// Before code snippet
    pub before_code: String,

    /// After code snippet
    pub after_code: String,

    /// Confidence level (0-100)
    pub confidence: u8,

    /// Estimated effort (in minutes)
    pub estimated_effort: u32,

    /// Impact description
    pub impact: String,

    /// WCAG criteria this addresses
    pub wcag_criteria: Vec<String>,

    /// Additional context
    pub context: HashMap<String, String>,

    /// Whether this fix has been applied
    pub applied: bool,

    /// When this suggestion was created
    pub created_at: DateTime<Utc>,

    /// When this fix was applied (if applied)
    pub applied_at: Option<DateTime<Utc>>,
}

impl RemediationSuggestion {
    /// Create a new remediation suggestion
    pub fn new(
        issue_id: impl Into<String>,
        title: impl Into<String>,
        element_selector: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            issue_id: issue_id.into(),
            title: title.into(),
            description: String::new(),
            remediation_type: RemediationType::Manual,
            priority: FixPriority::Medium,
            element_selector: element_selector.into(),
            before_code: String::new(),
            after_code: String::new(),
            confidence: 75,
            estimated_effort: 5,
            impact: String::new(),
            wcag_criteria: Vec::new(),
            context: HashMap::new(),
            applied: false,
            created_at: Utc::now(),
            applied_at: None,
        }
    }

    /// Mark this suggestion as applied
    pub fn mark_applied(&mut self) {
        self.applied = true;
        self.applied_at = Some(Utc::now());
    }

    /// Check if this suggestion can be auto-applied
    pub fn can_auto_apply(&self) -> bool {
        self.remediation_type == RemediationType::Automatic && self.confidence >= 90
    }
}

/// Result of applying a fix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixResult {
    /// Whether the fix was successful
    pub success: bool,

    /// Suggestion that was applied
    pub suggestion: RemediationSuggestion,

    /// Modified HTML (if successful)
    pub modified_html: Option<String>,

    /// Error message (if failed)
    pub error: Option<String>,

    /// Additional notes
    pub notes: Vec<String>,
}

/// Main remediation engine
pub struct RemediationEngine {
    suggestions_cache: HashMap<String, RemediationSuggestion>,
    ai_engine: AiFixEngine,
    auto_fix_engine: AutoFixEngine,
}

impl RemediationEngine {
    /// Create a new remediation engine
    pub fn new() -> Self {
        Self {
            suggestions_cache: HashMap::new(),
            ai_engine: AiFixEngine::new(),
            auto_fix_engine: AutoFixEngine::new(),
        }
    }

    /// Generate remediation suggestions for violations
    pub fn generate_suggestions(
        &mut self,
        violations: &[AccessibilityViolation],
    ) -> Vec<RemediationSuggestion> {
        let mut suggestions = Vec::new();

        for violation in violations {
            let violation_suggestions = self.suggest_fixes_for_violation(violation);
            for suggestion in violation_suggestions {
                self.suggestions_cache.insert(suggestion.id.clone(), suggestion.clone());
                suggestions.push(suggestion);
            }
        }

        // Sort by priority
        suggestions.sort_by(|a, b| b.priority.cmp(&a.priority));

        suggestions
    }

    /// Suggest fixes for a specific violation
    fn suggest_fixes_for_violation(
        &self,
        violation: &AccessibilityViolation,
    ) -> Vec<RemediationSuggestion> {
        match violation.rule_id.as_str() {
            "image-alt" => self.suggest_image_alt_fixes(violation),
            "color-contrast" => self.suggest_contrast_fixes(violation),
            "form-label" => self.suggest_form_label_fixes(violation),
            "link-name" => self.suggest_link_name_fixes(violation),
            "heading-order" => self.suggest_heading_fixes(violation),
            "aria-navigation-label" => self.suggest_aria_label_fixes(violation),
            "main-landmark" => self.suggest_landmark_fixes(violation),
            "keyboard-accessible" => self.suggest_keyboard_fixes(violation),
            _ => self.suggest_generic_fixes(violation),
        }
    }

    fn suggest_image_alt_fixes(&self, violation: &AccessibilityViolation) -> Vec<RemediationSuggestion> {
        let mut suggestions = Vec::new();

        let mut suggestion = RemediationSuggestion::new(
            &violation.rule_id,
            "Add descriptive alt text",
            &violation.element,
        );

        suggestion.description = "Add an alt attribute with descriptive text that conveys the purpose and content of the image.".to_string();
        suggestion.remediation_type = RemediationType::SemiAutomatic;
        suggestion.priority = FixPriority::Critical;
        suggestion.before_code = format!("<img src=\"image.jpg\">",);
        suggestion.after_code = format!("<img src=\"image.jpg\" alt=\"Description of image\">");
        suggestion.confidence = 85;
        suggestion.estimated_effort = 2;
        suggestion.impact = "Screen reader users will be able to understand the image content".to_string();
        suggestion.wcag_criteria = vec!["1.1.1".to_string()];

        suggestions.push(suggestion);

        // Alternative: mark as decorative
        let mut decorative_suggestion = RemediationSuggestion::new(
            &violation.rule_id,
            "Mark image as decorative",
            &violation.element,
        );

        decorative_suggestion.description = "If the image is purely decorative, use an empty alt attribute.".to_string();
        decorative_suggestion.remediation_type = RemediationType::SemiAutomatic;
        decorative_suggestion.priority = FixPriority::High;
        decorative_suggestion.before_code = format!("<img src=\"decorative.jpg\">");
        decorative_suggestion.after_code = format!("<img src=\"decorative.jpg\" alt=\"\">");
        decorative_suggestion.confidence = 70;
        decorative_suggestion.estimated_effort = 1;
        decorative_suggestion.impact = "Screen readers will skip this decorative image".to_string();
        decorative_suggestion.wcag_criteria = vec!["1.1.1".to_string()];

        suggestions.push(decorative_suggestion);

        suggestions
    }

    fn suggest_contrast_fixes(&self, violation: &AccessibilityViolation) -> Vec<RemediationSuggestion> {
        let mut suggestions = Vec::new();

        // Extract colors from context
        let fg_color = violation.context.get("foreground").cloned().unwrap_or_else(|| "#000000".to_string());
        let bg_color = violation.context.get("background").cloned().unwrap_or_else(|| "#ffffff".to_string());

        // Suggest darkening text
        let mut darken_suggestion = RemediationSuggestion::new(
            &violation.rule_id,
            "Darken text color",
            &violation.element,
        );

        darken_suggestion.description = "Increase contrast by using a darker text color.".to_string();
        darken_suggestion.remediation_type = RemediationType::SemiAutomatic;
        darken_suggestion.priority = FixPriority::Critical;
        darken_suggestion.before_code = format!("color: {};", fg_color);
        darken_suggestion.after_code = "color: #000000; /* Adjusted for better contrast */".to_string();
        darken_suggestion.confidence = 90;
        darken_suggestion.estimated_effort = 1;
        darken_suggestion.impact = "Text will be readable by users with low vision".to_string();
        darken_suggestion.wcag_criteria = vec!["1.4.3".to_string()];

        suggestions.push(darken_suggestion);

        // Suggest lightening background
        let mut lighten_suggestion = RemediationSuggestion::new(
            &violation.rule_id,
            "Lighten background color",
            &violation.element,
        );

        lighten_suggestion.description = "Increase contrast by using a lighter background color.".to_string();
        lighten_suggestion.remediation_type = RemediationType::SemiAutomatic;
        lighten_suggestion.priority = FixPriority::Critical;
        lighten_suggestion.before_code = format!("background-color: {};", bg_color);
        lighten_suggestion.after_code = "background-color: #ffffff; /* Adjusted for better contrast */".to_string();
        lighten_suggestion.confidence = 90;
        lighten_suggestion.estimated_effort = 1;
        lighten_suggestion.impact = "Text will be readable by users with low vision".to_string();
        lighten_suggestion.wcag_criteria = vec!["1.4.3".to_string()];

        suggestions.push(lighten_suggestion);

        suggestions
    }

    fn suggest_form_label_fixes(&self, violation: &AccessibilityViolation) -> Vec<RemediationSuggestion> {
        let mut suggestions = Vec::new();

        // Suggest adding a label element
        let mut label_suggestion = RemediationSuggestion::new(
            &violation.rule_id,
            "Add associated label element",
            &violation.element,
        );

        label_suggestion.description = "Add a <label> element associated with this input.".to_string();
        label_suggestion.remediation_type = RemediationType::SemiAutomatic;
        label_suggestion.priority = FixPriority::Critical;
        label_suggestion.before_code = "<input type=\"text\" id=\"username\">".to_string();
        label_suggestion.after_code = "<label for=\"username\">Username:</label>\n<input type=\"text\" id=\"username\">".to_string();
        label_suggestion.confidence = 95;
        label_suggestion.estimated_effort = 2;
        label_suggestion.impact = "Screen reader users will know the purpose of this input field".to_string();
        label_suggestion.wcag_criteria = vec!["3.3.2".to_string(), "4.1.2".to_string()];

        suggestions.push(label_suggestion);

        // Alternative: aria-label
        let mut aria_suggestion = RemediationSuggestion::new(
            &violation.rule_id,
            "Add aria-label attribute",
            &violation.element,
        );

        aria_suggestion.description = "Add aria-label attribute to provide accessible name.".to_string();
        aria_suggestion.remediation_type = RemediationType::SemiAutomatic;
        aria_suggestion.priority = FixPriority::High;
        aria_suggestion.before_code = "<input type=\"text\">".to_string();
        aria_suggestion.after_code = "<input type=\"text\" aria-label=\"Field description\">".to_string();
        aria_suggestion.confidence = 85;
        aria_suggestion.estimated_effort = 1;
        aria_suggestion.impact = "Screen reader users will know the purpose of this input field".to_string();
        aria_suggestion.wcag_criteria = vec!["3.3.2".to_string(), "4.1.2".to_string()];

        suggestions.push(aria_suggestion);

        suggestions
    }

    fn suggest_link_name_fixes(&self, violation: &AccessibilityViolation) -> Vec<RemediationSuggestion> {
        let mut suggestions = Vec::new();

        let mut suggestion = RemediationSuggestion::new(
            &violation.rule_id,
            "Use descriptive link text",
            &violation.element,
        );

        let current_text = violation.context.get("text").cloned().unwrap_or_else(|| "click here".to_string());

        suggestion.description = "Replace generic link text with descriptive text that indicates the link destination.".to_string();
        suggestion.remediation_type = RemediationType::Manual;
        suggestion.priority = FixPriority::Medium;
        suggestion.before_code = format!("<a href=\"page.html\">{}</a>", current_text);
        suggestion.after_code = "<a href=\"page.html\">Read the full accessibility guide</a>".to_string();
        suggestion.confidence = 80;
        suggestion.estimated_effort = 3;
        suggestion.impact = "Users will understand link purpose without context".to_string();
        suggestion.wcag_criteria = vec!["2.4.4".to_string()];

        suggestions.push(suggestion);

        suggestions
    }

    fn suggest_heading_fixes(&self, violation: &AccessibilityViolation) -> Vec<RemediationSuggestion> {
        let mut suggestions = Vec::new();

        let mut suggestion = RemediationSuggestion::new(
            &violation.rule_id,
            "Fix heading hierarchy",
            &violation.element,
        );

        suggestion.description = "Maintain sequential heading order without skipping levels.".to_string();
        suggestion.remediation_type = RemediationType::Manual;
        suggestion.priority = FixPriority::Medium;
        suggestion.before_code = "<h1>Title</h1>\n...\n<h4>Subsection</h4>".to_string();
        suggestion.after_code = "<h1>Title</h1>\n...\n<h2>Section</h2>\n<h3>Subsection</h3>".to_string();
        suggestion.confidence = 70;
        suggestion.estimated_effort = 5;
        suggestion.impact = "Screen reader users will understand document structure".to_string();
        suggestion.wcag_criteria = vec!["2.4.6".to_string()];

        suggestions.push(suggestion);

        suggestions
    }

    fn suggest_aria_label_fixes(&self, violation: &AccessibilityViolation) -> Vec<RemediationSuggestion> {
        let mut suggestions = Vec::new();

        let mut suggestion = RemediationSuggestion::new(
            &violation.rule_id,
            "Add aria-label to navigation",
            &violation.element,
        );

        suggestion.description = "Add aria-label to describe the purpose of this navigation landmark.".to_string();
        suggestion.remediation_type = RemediationType::SemiAutomatic;
        suggestion.priority = FixPriority::High;
        suggestion.before_code = "<nav>...</nav>".to_string();
        suggestion.after_code = "<nav aria-label=\"Main navigation\">...</nav>".to_string();
        suggestion.confidence = 90;
        suggestion.estimated_effort = 1;
        suggestion.impact = "Screen reader users can identify different navigation regions".to_string();
        suggestion.wcag_criteria = vec!["2.4.1".to_string()];

        suggestions.push(suggestion);

        suggestions
    }

    fn suggest_landmark_fixes(&self, violation: &AccessibilityViolation) -> Vec<RemediationSuggestion> {
        let mut suggestions = Vec::new();

        let mut suggestion = RemediationSuggestion::new(
            &violation.rule_id,
            "Add main landmark",
            &violation.element,
        );

        suggestion.description = "Add a <main> element to identify the primary content area.".to_string();
        suggestion.remediation_type = RemediationType::SemiAutomatic;
        suggestion.priority = FixPriority::High;
        suggestion.before_code = "<div class=\"content\">...</div>".to_string();
        suggestion.after_code = "<main>\n  <div class=\"content\">...</div>\n</main>".to_string();
        suggestion.confidence = 85;
        suggestion.estimated_effort = 2;
        suggestion.impact = "Screen reader users can skip directly to main content".to_string();
        suggestion.wcag_criteria = vec!["2.4.1".to_string()];

        suggestions.push(suggestion);

        suggestions
    }

    fn suggest_keyboard_fixes(&self, violation: &AccessibilityViolation) -> Vec<RemediationSuggestion> {
        let mut suggestions = Vec::new();

        let mut suggestion = RemediationSuggestion::new(
            &violation.rule_id,
            "Make element keyboard accessible",
            &violation.element,
        );

        suggestion.description = "Add keyboard support to interactive element.".to_string();
        suggestion.remediation_type = RemediationType::Manual;
        suggestion.priority = FixPriority::Critical;
        suggestion.before_code = "<div onclick=\"doSomething()\">Click me</div>".to_string();
        suggestion.after_code = "<button type=\"button\" onclick=\"doSomething()\">Click me</button>".to_string();
        suggestion.confidence = 95;
        suggestion.estimated_effort = 3;
        suggestion.impact = "Keyboard users will be able to interact with this element".to_string();
        suggestion.wcag_criteria = vec!["2.1.1".to_string(), "4.1.2".to_string()];

        suggestions.push(suggestion);

        suggestions
    }

    fn suggest_generic_fixes(&self, violation: &AccessibilityViolation) -> Vec<RemediationSuggestion> {
        let mut suggestions = Vec::new();

        for (i, fix) in violation.suggested_fixes.iter().enumerate() {
            let mut suggestion = RemediationSuggestion::new(
                &violation.rule_id,
                fix,
                &violation.element,
            );

            suggestion.description = fix.clone();
            suggestion.remediation_type = RemediationType::Manual;
            suggestion.priority = match violation.severity {
                super::scanner::ViolationSeverity::Critical => FixPriority::Critical,
                super::scanner::ViolationSeverity::Major => FixPriority::High,
                super::scanner::ViolationSeverity::Minor => FixPriority::Medium,
                super::scanner::ViolationSeverity::Info => FixPriority::Info,
            };
            suggestion.confidence = 70 - (i as u8 * 10); // Decrease confidence for later suggestions
            suggestion.estimated_effort = 5;
            suggestion.impact = violation.impact.clone();
            suggestion.wcag_criteria = vec![violation.wcag_criterion.clone()];

            suggestions.push(suggestion);
        }

        suggestions
    }

    /// Apply a fix
    pub fn apply_fix(
        &mut self,
        suggestion_id: &str,
        html: &str,
    ) -> Result<FixResult, RemediationError> {
        let suggestion = self
            .suggestions_cache
            .get_mut(suggestion_id)
            .ok_or_else(|| RemediationError::ElementNotFound(suggestion_id.to_string()))?;

        // For automatic fixes, apply directly
        if suggestion.can_auto_apply() {
            match self.auto_fix_engine.apply_fix(suggestion, html) {
                Ok(modified_html) => {
                    suggestion.mark_applied();
                    Ok(FixResult {
                        success: true,
                        suggestion: suggestion.clone(),
                        modified_html: Some(modified_html),
                        error: None,
                        notes: vec!["Fix applied successfully".to_string()],
                    })
                }
                Err(e) => Ok(FixResult {
                    success: false,
                    suggestion: suggestion.clone(),
                    modified_html: None,
                    error: Some(e.to_string()),
                    notes: Vec::new(),
                }),
            }
        } else {
            // For manual fixes, return the suggestion without applying
            Ok(FixResult {
                success: false,
                suggestion: suggestion.clone(),
                modified_html: None,
                error: Some("Manual intervention required".to_string()),
                notes: vec![
                    "This fix requires manual intervention".to_string(),
                    format!("Apply the following change: {}", suggestion.after_code),
                ],
            })
        }
    }

    /// Get AI-powered suggestions
    pub fn get_ai_suggestions(
        &self,
        violation: &AccessibilityViolation,
        context: &str,
    ) -> Result<Vec<RemediationSuggestion>, RemediationError> {
        self.ai_engine.generate_suggestions(violation, context)
    }
}

impl Default for RemediationEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// AI-powered fix engine
pub struct AiFixEngine {
    model_endpoint: Option<String>,
}

impl AiFixEngine {
    /// Create a new AI fix engine
    pub fn new() -> Self {
        Self {
            model_endpoint: None,
        }
    }

    /// Generate AI-powered suggestions
    pub fn generate_suggestions(
        &self,
        violation: &AccessibilityViolation,
        _context: &str,
    ) -> Result<Vec<RemediationSuggestion>, RemediationError> {
        // In production, this would call an AI service
        // For now, return enhanced suggestions based on rule patterns

        let mut suggestions = Vec::new();

        let mut suggestion = RemediationSuggestion::new(
            &violation.rule_id,
            "AI-suggested fix",
            &violation.element,
        );

        suggestion.remediation_type = RemediationType::AiSuggested;
        suggestion.description = format!(
            "AI-generated fix for: {}",
            violation.description
        );
        suggestion.confidence = 75;
        suggestion.impact = violation.impact.clone();

        suggestions.push(suggestion);

        Ok(suggestions)
    }

    /// Configure AI model endpoint
    pub fn configure_endpoint(&mut self, endpoint: String) {
        self.model_endpoint = Some(endpoint);
    }
}

impl Default for AiFixEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Automated fix engine
pub struct AutoFixEngine;

impl AutoFixEngine {
    /// Create a new auto-fix engine
    pub fn new() -> Self {
        Self
    }

    /// Apply an automatic fix
    pub fn apply_fix(
        &self,
        suggestion: &RemediationSuggestion,
        html: &str,
    ) -> Result<String, RemediationError> {
        // In production, use a proper HTML parser and DOM manipulation
        // For now, perform simple string replacement

        if suggestion.before_code.is_empty() {
            return Err(RemediationError::InvalidFix(
                "No before code specified".to_string(),
            ));
        }

        let modified = html.replace(&suggestion.before_code, &suggestion.after_code);

        if modified == html {
            return Err(RemediationError::FixFailed(
                "Pattern not found in HTML".to_string(),
            ));
        }

        Ok(modified)
    }

    /// Check if a fix can be applied automatically
    pub fn can_apply(&self, suggestion: &RemediationSuggestion) -> bool {
        suggestion.remediation_type == RemediationType::Automatic
            && !suggestion.before_code.is_empty()
            && !suggestion.after_code.is_empty()
    }
}

impl Default for AutoFixEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Bulk remediation queue with priority ordering
pub struct BulkRemediationQueue {
    queue: BTreeMap<FixPriority, VecDeque<RemediationSuggestion>>,
    applied_fixes: Vec<RemediationSuggestion>,
}

impl BulkRemediationQueue {
    /// Create a new bulk remediation queue
    pub fn new() -> Self {
        Self {
            queue: BTreeMap::new(),
            applied_fixes: Vec::new(),
        }
    }

    /// Add a suggestion to the queue
    pub fn add(&mut self, suggestion: RemediationSuggestion) {
        self.queue
            .entry(suggestion.priority)
            .or_insert_with(VecDeque::new)
            .push_back(suggestion);
    }

    /// Add multiple suggestions
    pub fn add_batch(&mut self, suggestions: Vec<RemediationSuggestion>) {
        for suggestion in suggestions {
            self.add(suggestion);
        }
    }

    /// Get the next suggestion to process (highest priority)
    pub fn next(&mut self) -> Option<RemediationSuggestion> {
        // Iterate in reverse order (highest priority first)
        for (_, queue) in self.queue.iter_mut().rev() {
            if let Some(suggestion) = queue.pop_front() {
                return Some(suggestion);
            }
        }
        None
    }

    /// Get all suggestions for a specific priority
    pub fn get_by_priority(&self, priority: FixPriority) -> Vec<&RemediationSuggestion> {
        self.queue
            .get(&priority)
            .map(|q| q.iter().collect())
            .unwrap_or_default()
    }

    /// Get total count of pending suggestions
    pub fn pending_count(&self) -> usize {
        self.queue.values().map(|q| q.len()).sum()
    }

    /// Get count by priority
    pub fn count_by_priority(&self, priority: FixPriority) -> usize {
        self.queue.get(&priority).map(|q| q.len()).unwrap_or(0)
    }

    /// Mark a suggestion as applied
    pub fn mark_applied(&mut self, mut suggestion: RemediationSuggestion) {
        suggestion.mark_applied();
        self.applied_fixes.push(suggestion);
    }

    /// Get statistics
    pub fn statistics(&self) -> QueueStatistics {
        QueueStatistics {
            total_pending: self.pending_count(),
            critical_count: self.count_by_priority(FixPriority::Critical),
            high_count: self.count_by_priority(FixPriority::High),
            medium_count: self.count_by_priority(FixPriority::Medium),
            low_count: self.count_by_priority(FixPriority::Low),
            info_count: self.count_by_priority(FixPriority::Info),
            applied_count: self.applied_fixes.len(),
        }
    }

    /// Get applied fixes
    pub fn applied_fixes(&self) -> &[RemediationSuggestion] {
        &self.applied_fixes
    }

    /// Clear the queue
    pub fn clear(&mut self) {
        self.queue.clear();
        self.applied_fixes.clear();
    }

    /// Get all automatic fixes (for bulk auto-remediation)
    pub fn automatic_fixes(&self) -> Vec<RemediationSuggestion> {
        let mut fixes = Vec::new();

        for queue in self.queue.values() {
            for suggestion in queue {
                if suggestion.can_auto_apply() {
                    fixes.push(suggestion.clone());
                }
            }
        }

        // Sort by priority (highest first)
        fixes.sort_by(|a, b| b.priority.cmp(&a.priority));

        fixes
    }
}

impl Default for BulkRemediationQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Queue statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStatistics {
    pub total_pending: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub info_count: usize,
    pub applied_count: usize,
}

impl QueueStatistics {
    /// Get compliance urgency score (0-100)
    pub fn urgency_score(&self) -> u8 {
        let weighted_score = (self.critical_count * 10) + (self.high_count * 5) + (self.medium_count * 2);
        (weighted_score.min(100)) as u8
    }

    /// Get estimated total effort (in minutes)
    pub fn estimated_total_effort(&self) -> u32 {
        // Rough estimates: Critical=30min, High=15min, Medium=10min, Low=5min, Info=2min
        (self.critical_count * 30
            + self.high_count * 15
            + self.medium_count * 10
            + self.low_count * 5
            + self.info_count * 2) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remediation_suggestion_creation() {
        let suggestion = RemediationSuggestion::new("test-issue", "Test Fix", "div.test");

        assert_eq!(suggestion.issue_id, "test-issue");
        assert_eq!(suggestion.title, "Test Fix");
        assert_eq!(suggestion.element_selector, "div.test");
        assert!(!suggestion.applied);
    }

    #[test]
    fn test_fix_priority_ordering() {
        assert!(FixPriority::Critical > FixPriority::High);
        assert!(FixPriority::High > FixPriority::Medium);
        assert!(FixPriority::Medium > FixPriority::Low);
        assert!(FixPriority::Low > FixPriority::Info);
    }

    #[test]
    fn test_bulk_queue_priority() {
        let mut queue = BulkRemediationQueue::new();

        let low_priority = RemediationSuggestion::new("1", "Low", "div")
            .into_builder()
            .with_priority(FixPriority::Low)
            .build();

        let critical_priority = RemediationSuggestion::new("2", "Critical", "div")
            .into_builder()
            .with_priority(FixPriority::Critical)
            .build();

        queue.add(low_priority);
        queue.add(critical_priority);

        assert_eq!(queue.pending_count(), 2);

        // Should get critical first
        let next = queue.next().unwrap();
        assert_eq!(next.title, "Critical");
    }

    #[test]
    fn test_queue_statistics() {
        let mut queue = BulkRemediationQueue::new();

        let mut critical = RemediationSuggestion::new("1", "Critical", "div");
        critical.priority = FixPriority::Critical;

        let mut high = RemediationSuggestion::new("2", "High", "div");
        high.priority = FixPriority::High;

        queue.add(critical);
        queue.add(high);

        let stats = queue.statistics();
        assert_eq!(stats.total_pending, 2);
        assert_eq!(stats.critical_count, 1);
        assert_eq!(stats.high_count, 1);
    }

    #[test]
    fn test_auto_fix_capability() {
        let mut suggestion = RemediationSuggestion::new("test", "Fix", "div");
        suggestion.remediation_type = RemediationType::Automatic;
        suggestion.confidence = 95;

        assert!(suggestion.can_auto_apply());

        suggestion.confidence = 85;
        assert!(!suggestion.can_auto_apply());
    }
}

// Helper trait for builder pattern (used in tests)
trait SuggestionBuilder {
    fn into_builder(self) -> SuggestionBuilderImpl;
}

impl SuggestionBuilder for RemediationSuggestion {
    fn into_builder(self) -> SuggestionBuilderImpl {
        SuggestionBuilderImpl { suggestion: self }
    }
}

struct SuggestionBuilderImpl {
    suggestion: RemediationSuggestion,
}

impl SuggestionBuilderImpl {
    fn with_priority(mut self, priority: FixPriority) -> Self {
        self.suggestion.priority = priority;
        self
    }

    fn build(self) -> RemediationSuggestion {
        self.suggestion
    }
}
