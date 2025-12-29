//! AI-Powered Suggestions for Accessibility
//!
//! Provides intelligent suggestions for accessibility improvements including:
//! - Auto-fix generation for common issues
//! - Code completion for accessibility fixes
//! - Alternative text suggestions
//! - ARIA attribute recommendations
//! - Best practice suggestions

use crate::ai::{AIError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Suggestion engine for accessibility improvements
pub struct SuggestionEngine {
    config: SuggestionConfig,
    knowledge_base: KnowledgeBase,
}

/// Suggestion engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionConfig {
    /// Enable auto-fix generation
    pub enable_auto_fix: bool,
    /// Enable code completion
    pub enable_code_completion: bool,
    /// Enable alt-text suggestions
    pub enable_alt_text: bool,
    /// Enable ARIA suggestions
    pub enable_aria: bool,
    /// Enable best practice suggestions
    pub enable_best_practices: bool,
    /// Minimum confidence for suggestions
    pub min_confidence: f64,
    /// Maximum suggestions per issue
    pub max_suggestions_per_issue: usize,
}

impl Default for SuggestionConfig {
    fn default() -> Self {
        Self {
            enable_auto_fix: true,
            enable_code_completion: true,
            enable_alt_text: true,
            enable_aria: true,
            enable_best_practices: true,
            min_confidence: 0.7,
            max_suggestions_per_issue: 3,
        }
    }
}

/// Knowledge base for accessibility patterns and fixes
#[derive(Debug, Clone)]
struct KnowledgeBase {
    auto_fix_patterns: HashMap<String, FixPattern>,
    aria_patterns: HashMap<String, AriaPattern>,
    best_practices: Vec<BestPracticeRule>,
}

impl Default for KnowledgeBase {
    fn default() -> Self {
        Self {
            auto_fix_patterns: Self::init_auto_fix_patterns(),
            aria_patterns: Self::init_aria_patterns(),
            best_practices: Self::init_best_practices(),
        }
    }
}

impl KnowledgeBase {
    /// Initialize auto-fix patterns
    fn init_auto_fix_patterns() -> HashMap<String, FixPattern> {
        let mut patterns = HashMap::new();

        patterns.insert(
            "missing_alt_text".to_string(),
            FixPattern {
                issue_type: "missing_alt_text".to_string(),
                fix_template: r#"alt="{suggested_text}""#.to_string(),
                confidence: 0.9,
                requires_manual_review: false,
            },
        );

        patterns.insert(
            "missing_label".to_string(),
            FixPattern {
                issue_type: "missing_label".to_string(),
                fix_template: r#"<label for="{input_id}">{label_text}</label>"#.to_string(),
                confidence: 0.85,
                requires_manual_review: false,
            },
        );

        patterns.insert(
            "low_contrast".to_string(),
            FixPattern {
                issue_type: "low_contrast".to_string(),
                fix_template: r#"color: {suggested_color};"#.to_string(),
                confidence: 0.8,
                requires_manual_review: true,
            },
        );

        patterns.insert(
            "missing_lang".to_string(),
            FixPattern {
                issue_type: "missing_lang".to_string(),
                fix_template: r#"lang="en""#.to_string(),
                confidence: 0.7,
                requires_manual_review: true,
            },
        );

        patterns
    }

    /// Initialize ARIA patterns
    fn init_aria_patterns() -> HashMap<String, AriaPattern> {
        let mut patterns = HashMap::new();

        patterns.insert(
            "button".to_string(),
            AriaPattern {
                element_type: "button".to_string(),
                recommended_attributes: vec!["aria-label".to_string(), "aria-pressed".to_string()],
                required_attributes: vec![],
                conditional_attributes: vec![
                    ("icon_only".to_string(), vec!["aria-label".to_string()]),
                ],
            },
        );

        patterns.insert(
            "navigation".to_string(),
            AriaPattern {
                element_type: "nav".to_string(),
                recommended_attributes: vec!["aria-label".to_string()],
                required_attributes: vec![],
                conditional_attributes: vec![],
            },
        );

        patterns.insert(
            "dialog".to_string(),
            AriaPattern {
                element_type: "dialog".to_string(),
                recommended_attributes: vec!["aria-labelledby".to_string(), "aria-describedby".to_string()],
                required_attributes: vec!["role".to_string()],
                conditional_attributes: vec![],
            },
        );

        patterns
    }

    /// Initialize best practices
    fn init_best_practices() -> Vec<BestPracticeRule> {
        vec![
            BestPracticeRule {
                id: "bp_001".to_string(),
                title: "Use semantic HTML".to_string(),
                description: "Prefer semantic HTML elements over generic divs with ARIA roles".to_string(),
                category: "HTML Structure".to_string(),
                priority: 1,
            },
            BestPracticeRule {
                id: "bp_002".to_string(),
                title: "Provide text alternatives".to_string(),
                description: "All non-text content must have text alternatives".to_string(),
                category: "Content".to_string(),
                priority: 1,
            },
            BestPracticeRule {
                id: "bp_003".to_string(),
                title: "Ensure keyboard accessibility".to_string(),
                description: "All interactive elements must be keyboard accessible".to_string(),
                category: "Interaction".to_string(),
                priority: 1,
            },
            BestPracticeRule {
                id: "bp_004".to_string(),
                title: "Use sufficient color contrast".to_string(),
                description: "Text must have sufficient contrast against background (4.5:1 for normal text)".to_string(),
                category: "Visual Design".to_string(),
                priority: 2,
            },
        ]
    }
}

/// Fix pattern for auto-fix generation
#[derive(Debug, Clone)]
struct FixPattern {
    issue_type: String,
    fix_template: String,
    confidence: f64,
    requires_manual_review: bool,
}

/// ARIA pattern recommendation
#[derive(Debug, Clone)]
struct AriaPattern {
    element_type: String,
    recommended_attributes: Vec<String>,
    required_attributes: Vec<String>,
    conditional_attributes: Vec<(String, Vec<String>)>,
}

/// Best practice rule
#[derive(Debug, Clone)]
struct BestPracticeRule {
    id: String,
    title: String,
    description: String,
    category: String,
    priority: u8,
}

impl SuggestionEngine {
    /// Create a new suggestion engine
    pub fn new(config: SuggestionConfig) -> Self {
        Self {
            config,
            knowledge_base: KnowledgeBase::default(),
        }
    }

    /// Generate auto-fix for an issue
    pub fn generate_auto_fix(&self, issue: &AccessibilityIssue) -> Result<AutoFix> {
        if !self.config.enable_auto_fix {
            return Err(AIError::InvalidConfig("Auto-fix is disabled".to_string()));
        }

        let pattern = self.knowledge_base.auto_fix_patterns.get(&issue.issue_type)
            .ok_or_else(|| AIError::InferenceError(format!("No fix pattern for issue type: {}", issue.issue_type)))?;

        if pattern.confidence < self.config.min_confidence {
            return Err(AIError::InferenceError("Confidence below threshold".to_string()));
        }

        // Generate fix code based on pattern
        let fix_code = self.generate_fix_code(pattern, issue)?;

        Ok(AutoFix {
            issue_id: issue.id.clone(),
            fix_type: issue.issue_type.clone(),
            original_code: issue.code_snippet.clone(),
            fixed_code: fix_code.clone(),
            diff: self.generate_diff(&issue.code_snippet, &fix_code),
            confidence: SuggestionConfidence::from_score(pattern.confidence),
            requires_manual_review: pattern.requires_manual_review,
            explanation: self.generate_fix_explanation(&issue.issue_type),
            wcag_criteria: issue.wcag_criteria.clone(),
            applied: false,
            timestamp: Utc::now(),
        })
    }

    /// Generate fix code from pattern
    fn generate_fix_code(&self, pattern: &FixPattern, issue: &AccessibilityIssue) -> Result<String> {
        let mut code = issue.code_snippet.clone();

        match pattern.issue_type.as_str() {
            "missing_alt_text" => {
                // Extract image element and add alt text
                if code.contains("<img") && !code.contains("alt=") {
                    let alt_text = issue.context.get("suggested_alt")
                        .map(|s| s.as_str())
                        .unwrap_or("Image description");
                    code = code.replace(">", &format!(r#" alt="{}">"#, alt_text));
                }
            }
            "missing_label" => {
                // Add label for input
                if code.contains("<input") {
                    let input_id = issue.context.get("input_id")
                        .map(|s| s.as_str())
                        .unwrap_or("input");
                    let label_text = issue.context.get("label_text")
                        .map(|s| s.as_str())
                        .unwrap_or("Label");
                    code = format!(r#"<label for="{}">{}</label>{}"#, input_id, label_text, code);
                }
            }
            "low_contrast" => {
                // Suggest better color
                if let Some(new_color) = issue.context.get("suggested_color") {
                    if let Some(old_color) = issue.context.get("current_color") {
                        code = code.replace(old_color, new_color);
                    }
                }
            }
            "missing_lang" => {
                // Add lang attribute to html tag
                if code.contains("<html") && !code.contains("lang=") {
                    code = code.replace("<html", r#"<html lang="en""#);
                }
            }
            _ => {}
        }

        Ok(code)
    }

    /// Generate diff between original and fixed code
    fn generate_diff(&self, original: &str, fixed: &str) -> String {
        if original == fixed {
            return "No changes".to_string();
        }

        // Simple diff - in production would use proper diff algorithm
        format!("- {}\n+ {}", original, fixed)
    }

    /// Generate explanation for fix
    fn generate_fix_explanation(&self, issue_type: &str) -> String {
        match issue_type {
            "missing_alt_text" => "Added alt text to image for screen reader users".to_string(),
            "missing_label" => "Added label to form input for better accessibility".to_string(),
            "low_contrast" => "Increased color contrast to meet WCAG AA standards".to_string(),
            "missing_lang" => "Added language attribute to HTML element".to_string(),
            _ => "Applied accessibility fix".to_string(),
        }
    }

    /// Generate code completion suggestions
    pub fn generate_code_completion(&self, context: &CodeContext) -> Result<Vec<CodeCompletion>> {
        if !self.config.enable_code_completion {
            return Err(AIError::InvalidConfig("Code completion is disabled".to_string()));
        }

        let mut completions = Vec::new();

        // Detect incomplete accessibility attributes
        if context.element_type == "img" && !context.has_attribute("alt") {
            completions.push(CodeCompletion {
                completion_text: r#"alt="Image description""#.to_string(),
                display_text: "alt (required)".to_string(),
                description: "Add alternative text for image".to_string(),
                confidence: SuggestionConfidence::High,
                category: "Required Attribute".to_string(),
            });
        }

        if context.element_type == "input" && !context.has_attribute("aria-label") && !context.has_attribute("id") {
            completions.push(CodeCompletion {
                completion_text: r#"aria-label="Input label""#.to_string(),
                display_text: "aria-label".to_string(),
                description: "Add accessible label for input".to_string(),
                confidence: SuggestionConfidence::Medium,
                category: "Recommended Attribute".to_string(),
            });
        }

        Ok(completions.into_iter()
            .take(self.config.max_suggestions_per_issue)
            .collect())
    }

    /// Generate alt-text suggestions
    pub fn suggest_alt_text(&self, image_context: &ImageContext) -> Result<Vec<AltTextSuggestion>> {
        if !self.config.enable_alt_text {
            return Err(AIError::InvalidConfig("Alt-text suggestions disabled".to_string()));
        }

        let mut suggestions = Vec::new();

        // Generate suggestions based on context
        if let Some(filename) = &image_context.filename {
            // Extract information from filename
            let cleaned_name = filename
                .trim_end_matches(".jpg")
                .trim_end_matches(".png")
                .trim_end_matches(".gif")
                .replace('-', " ")
                .replace('_', " ");

            suggestions.push(AltTextSuggestion {
                suggested_text: cleaned_name.clone(),
                confidence: SuggestionConfidence::Medium,
                reasoning: "Based on image filename".to_string(),
                is_decorative: false,
            });
        }

        // If image is likely decorative
        if image_context.is_likely_decorative() {
            suggestions.push(AltTextSuggestion {
                suggested_text: String::new(),
                confidence: SuggestionConfidence::Medium,
                reasoning: "Image appears decorative (use empty alt text)".to_string(),
                is_decorative: true,
            });
        }

        // Context-based suggestions
        if let Some(nearby_text) = &image_context.nearby_text {
            if !nearby_text.is_empty() {
                suggestions.push(AltTextSuggestion {
                    suggested_text: nearby_text.clone(),
                    confidence: SuggestionConfidence::Low,
                    reasoning: "Based on nearby text content".to_string(),
                    is_decorative: false,
                });
            }
        }

        Ok(suggestions.into_iter()
            .take(self.config.max_suggestions_per_issue)
            .collect())
    }

    /// Generate ARIA attribute recommendations
    pub fn recommend_aria_attributes(&self, element: &ElementContext) -> Result<Vec<ARIASuggestion>> {
        if !self.config.enable_aria {
            return Err(AIError::InvalidConfig("ARIA suggestions disabled".to_string()));
        }

        let mut suggestions = Vec::new();

        if let Some(pattern) = self.knowledge_base.aria_patterns.get(&element.element_type) {
            // Check required attributes
            for attr in &pattern.required_attributes {
                if !element.has_attribute(attr) {
                    suggestions.push(ARIASuggestion {
                        attribute_name: attr.clone(),
                        suggested_value: self.suggest_aria_value(attr, element),
                        reason: format!("Required ARIA attribute for {}", element.element_type),
                        confidence: SuggestionConfidence::High,
                        is_required: true,
                        wcag_criteria: vec!["4.1.2".to_string()],
                    });
                }
            }

            // Check recommended attributes
            for attr in &pattern.recommended_attributes {
                if !element.has_attribute(attr) {
                    suggestions.push(ARIASuggestion {
                        attribute_name: attr.clone(),
                        suggested_value: self.suggest_aria_value(attr, element),
                        reason: format!("Recommended for better accessibility"),
                        confidence: SuggestionConfidence::Medium,
                        is_required: false,
                        wcag_criteria: vec!["4.1.2".to_string()],
                    });
                }
            }
        }

        Ok(suggestions.into_iter()
            .take(self.config.max_suggestions_per_issue)
            .collect())
    }

    /// Suggest ARIA attribute value
    fn suggest_aria_value(&self, attribute: &str, element: &ElementContext) -> String {
        match attribute {
            "aria-label" => element.element_type.clone(),
            "aria-labelledby" => "heading-id".to_string(),
            "aria-describedby" => "description-id".to_string(),
            "role" => element.element_type.clone(),
            "aria-pressed" => "false".to_string(),
            _ => String::new(),
        }
    }

    /// Generate best practice suggestions
    pub fn suggest_best_practices(&self, code_context: &CodeContext) -> Result<Vec<BestPracticeSuggestion>> {
        if !self.config.enable_best_practices {
            return Err(AIError::InvalidConfig("Best practice suggestions disabled".to_string()));
        }

        let mut suggestions = Vec::new();

        // Check for semantic HTML usage
        if code_context.element_type == "div" && code_context.has_attribute("role") {
            if let Some(role) = code_context.get_attribute("role") {
                let semantic_element = match role.as_str() {
                    "navigation" => Some("nav"),
                    "main" => Some("main"),
                    "button" => Some("button"),
                    "article" => Some("article"),
                    _ => None,
                };

                if let Some(elem) = semantic_element {
                    suggestions.push(BestPracticeSuggestion {
                        title: "Use semantic HTML".to_string(),
                        description: format!("Replace <div role=\"{}\"> with <{}>", role, elem),
                        priority: 1,
                        category: "HTML Structure".to_string(),
                        example_code: Some(format!("<{}>...</{}>", elem, elem)),
                        wcag_reference: vec!["1.3.1".to_string()],
                    });
                }
            }
        }

        // Check for heading hierarchy
        if code_context.element_type.starts_with('h') {
            suggestions.push(BestPracticeSuggestion {
                title: "Maintain heading hierarchy".to_string(),
                description: "Ensure headings follow logical order (h1 > h2 > h3)".to_string(),
                priority: 2,
                category: "HTML Structure".to_string(),
                example_code: None,
                wcag_reference: vec!["1.3.1".to_string(), "2.4.6".to_string()],
            });
        }

        Ok(suggestions.into_iter()
            .take(self.config.max_suggestions_per_issue)
            .collect())
    }
}

/// Auto-fix for accessibility issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoFix {
    pub issue_id: String,
    pub fix_type: String,
    pub original_code: String,
    pub fixed_code: String,
    pub diff: String,
    pub confidence: SuggestionConfidence,
    pub requires_manual_review: bool,
    pub explanation: String,
    pub wcag_criteria: Vec<String>,
    pub applied: bool,
    pub timestamp: DateTime<Utc>,
}

/// Code completion suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeCompletion {
    pub completion_text: String,
    pub display_text: String,
    pub description: String,
    pub confidence: SuggestionConfidence,
    pub category: String,
}

/// Alt-text suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AltTextSuggestion {
    pub suggested_text: String,
    pub confidence: SuggestionConfidence,
    pub reasoning: String,
    pub is_decorative: bool,
}

/// ARIA attribute suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ARIASuggestion {
    pub attribute_name: String,
    pub suggested_value: String,
    pub reason: String,
    pub confidence: SuggestionConfidence,
    pub is_required: bool,
    pub wcag_criteria: Vec<String>,
}

/// Best practice suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BestPracticeSuggestion {
    pub title: String,
    pub description: String,
    pub priority: u8,
    pub category: String,
    pub example_code: Option<String>,
    pub wcag_reference: Vec<String>,
}

/// Suggestion confidence level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionConfidence {
    High,
    Medium,
    Low,
}

impl SuggestionConfidence {
    fn from_score(score: f64) -> Self {
        if score >= 0.8 {
            Self::High
        } else if score >= 0.6 {
            Self::Medium
        } else {
            Self::Low
        }
    }
}

/// Accessibility issue data
#[derive(Debug, Clone)]
pub struct AccessibilityIssue {
    pub id: String,
    pub issue_type: String,
    pub code_snippet: String,
    pub wcag_criteria: Vec<String>,
    pub context: HashMap<String, String>,
}

/// Code context for suggestions
#[derive(Debug, Clone)]
pub struct CodeContext {
    pub element_type: String,
    pub attributes: HashMap<String, String>,
    pub parent_context: Option<String>,
}

impl CodeContext {
    pub fn has_attribute(&self, attr: &str) -> bool {
        self.attributes.contains_key(attr)
    }

    pub fn get_attribute(&self, attr: &str) -> Option<String> {
        self.attributes.get(attr).cloned()
    }
}

/// Image context for alt-text suggestions
#[derive(Debug, Clone)]
pub struct ImageContext {
    pub filename: Option<String>,
    pub dimensions: Option<(u32, u32)>,
    pub nearby_text: Option<String>,
    pub is_in_link: bool,
    pub css_classes: Vec<String>,
}

impl ImageContext {
    fn is_likely_decorative(&self) -> bool {
        // Heuristics for decorative images
        if let Some(classes) = self.css_classes.iter().find(|c| c.contains("icon") || c.contains("decoration")) {
            return true;
        }

        if let Some((width, height)) = self.dimensions {
            if width < 10 || height < 10 {
                return true; // Likely a spacer or icon
            }
        }

        false
    }
}

/// Element context for ARIA suggestions
#[derive(Debug, Clone)]
pub struct ElementContext {
    pub element_type: String,
    pub attributes: HashMap<String, String>,
    pub children_count: usize,
}

impl ElementContext {
    pub fn has_attribute(&self, attr: &str) -> bool {
        self.attributes.contains_key(attr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_fix_generation() {
        let engine = SuggestionEngine::new(SuggestionConfig::default());

        let issue = AccessibilityIssue {
            id: "issue-1".to_string(),
            issue_type: "missing_alt_text".to_string(),
            code_snippet: "<img src=\"test.jpg\">".to_string(),
            wcag_criteria: vec!["1.1.1".to_string()],
            context: HashMap::new(),
        };

        let result = engine.generate_auto_fix(&issue);
        assert!(result.is_ok());

        let fix = result.unwrap();
        assert!(fix.fixed_code.contains("alt="));
    }

    #[test]
    fn test_confidence_from_score() {
        assert!(matches!(SuggestionConfidence::from_score(0.9), SuggestionConfidence::High));
        assert!(matches!(SuggestionConfidence::from_score(0.7), SuggestionConfidence::Medium));
        assert!(matches!(SuggestionConfidence::from_score(0.5), SuggestionConfidence::Low));
    }
}
