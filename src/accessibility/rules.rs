//! # Accessibility Rules Engine
//!
//! Comprehensive accessibility rules for WCAG 2.1, WCAG 2.2, Section 508, ADA,
//! and EN 301 549 compliance checking.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;
use once_cell::sync::Lazy;

/// Errors that can occur in the rules engine
#[derive(Error, Debug)]
pub enum RuleError {
    #[error("Rule not found: {0}")]
    RuleNotFound(String),

    #[error("Invalid rule configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Rule evaluation failed: {0}")]
    EvaluationError(String),
}

/// WCAG compliance level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WcagLevel {
    /// Level A - Minimum compliance
    A,
    /// Level AA - Mid-range compliance (recommended)
    AA,
    /// Level AAA - Highest compliance
    AAA,
}

/// Rule severity classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RuleSeverity {
    /// Critical - must be fixed immediately
    Critical,
    /// Major - should be fixed soon
    Major,
    /// Minor - should be addressed
    Minor,
    /// Info - informational only
    Info,
}

/// Rule category classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuleCategory {
    /// Perceivable - Information and UI must be presentable
    Perceivable,
    /// Operable - UI and navigation must be operable
    Operable,
    /// Understandable - Information and UI must be understandable
    Understandable,
    /// Robust - Content must be robust enough for assistive technologies
    Robust,
    /// Custom category
    Custom(String),
}

/// An accessibility rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityRule {
    /// Unique rule identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Detailed description
    pub description: String,

    /// Rule severity
    pub severity: RuleSeverity,

    /// Rule category
    pub category: RuleCategory,

    /// WCAG level (if applicable)
    pub wcag_level: Option<WcagLevel>,

    /// WCAG success criterion (e.g., "1.1.1")
    pub wcag_criterion: Option<String>,

    /// Related WCAG techniques
    pub wcag_techniques: Vec<String>,

    /// Section 508 reference
    pub section_508_ref: Option<String>,

    /// ADA reference
    pub ada_ref: Option<String>,

    /// EN 301 549 reference
    pub en_301_549_ref: Option<String>,

    /// Tags for filtering
    pub tags: HashSet<String>,

    /// Whether this rule is enabled by default
    pub enabled_by_default: bool,

    /// Custom properties
    pub properties: HashMap<String, String>,
}

impl AccessibilityRule {
    /// Create a new accessibility rule
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            severity: RuleSeverity::Major,
            category: RuleCategory::Perceivable,
            wcag_level: None,
            wcag_criterion: None,
            wcag_techniques: Vec::new(),
            section_508_ref: None,
            ada_ref: None,
            en_301_549_ref: None,
            tags: HashSet::new(),
            enabled_by_default: true,
            properties: HashMap::new(),
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set the severity
    pub fn with_severity(mut self, severity: RuleSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Set the category
    pub fn with_category(mut self, category: RuleCategory) -> Self {
        self.category = category;
        self
    }

    /// Set the WCAG level
    pub fn with_wcag_level(mut self, level: WcagLevel) -> Self {
        self.wcag_level = Some(level);
        self
    }

    /// Set the WCAG success criterion
    pub fn with_wcag_criterion(mut self, criterion: impl Into<String>) -> Self {
        self.wcag_criterion = Some(criterion.into());
        self
    }

    /// Add WCAG techniques
    pub fn with_wcag_techniques(mut self, techniques: Vec<impl Into<String>>) -> Self {
        self.wcag_techniques = techniques.into_iter().map(|t| t.into()).collect();
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.insert(tag.into());
        self
    }
}

/// A violation of an accessibility rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleViolation {
    /// Rule that was violated
    pub rule_id: String,

    /// Element that violated the rule
    pub element_selector: String,

    /// Additional context
    pub context: HashMap<String, String>,

    /// Suggested fixes
    pub fixes: Vec<String>,
}

/// WCAG 2.1 specific rule
#[derive(Debug, Clone)]
pub struct WcagRule {
    /// Base rule
    pub base: AccessibilityRule,

    /// WCAG 2.1 principle (1-4)
    pub principle: u8,

    /// WCAG 2.1 guideline
    pub guideline: String,
}

impl WcagRule {
    /// Create a new WCAG rule
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        level: WcagLevel,
        criterion: impl Into<String>,
    ) -> Self {
        let criterion_str = criterion.into();
        let principle = criterion_str
            .split('.')
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);

        Self {
            base: AccessibilityRule::new(id, name)
                .with_wcag_level(level)
                .with_wcag_criterion(&criterion_str),
            principle,
            guideline: criterion_str,
        }
    }
}

/// Section 508 specific rule
#[derive(Debug, Clone)]
pub struct Section508Rule {
    /// Base rule
    pub base: AccessibilityRule,

    /// Section 508 section reference
    pub section: String,
}

impl Section508Rule {
    /// Create a new Section 508 rule
    pub fn new(id: impl Into<String>, name: impl Into<String>, section: impl Into<String>) -> Self {
        let mut base = AccessibilityRule::new(id, name);
        base.section_508_ref = Some(section.into());

        Self {
            base,
            section: section.into(),
        }
    }
}

/// ADA specific rule
#[derive(Debug, Clone)]
pub struct AdaRule {
    /// Base rule
    pub base: AccessibilityRule,

    /// ADA title reference
    pub title: String,
}

impl AdaRule {
    /// Create a new ADA rule
    pub fn new(id: impl Into<String>, name: impl Into<String>, title: impl Into<String>) -> Self {
        let mut base = AccessibilityRule::new(id, name);
        base.ada_ref = Some(title.into());

        Self {
            base,
            title: title.into(),
        }
    }
}

/// EN 301 549 European standard rule
#[derive(Debug, Clone)]
pub struct En301549Rule {
    /// Base rule
    pub base: AccessibilityRule,

    /// EN 301 549 clause reference
    pub clause: String,
}

impl En301549Rule {
    /// Create a new EN 301 549 rule
    pub fn new(id: impl Into<String>, name: impl Into<String>, clause: impl Into<String>) -> Self {
        let mut base = AccessibilityRule::new(id, name);
        base.en_301_549_ref = Some(clause.into());

        Self {
            base,
            clause: clause.into(),
        }
    }
}

/// Collection of accessibility rules
#[derive(Debug, Clone)]
pub struct RuleSet {
    rules: HashMap<String, AccessibilityRule>,
    rules_by_category: HashMap<RuleCategory, Vec<String>>,
    rules_by_severity: HashMap<RuleSeverity, Vec<String>>,
    rules_by_wcag_level: HashMap<WcagLevel, Vec<String>>,
}

impl RuleSet {
    /// Create a new empty rule set
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            rules_by_category: HashMap::new(),
            rules_by_severity: HashMap::new(),
            rules_by_wcag_level: HashMap::new(),
        }
    }

    /// Add a rule to the set
    pub fn add_rule(&mut self, rule: AccessibilityRule) {
        let id = rule.id.clone();

        // Index by category
        self.rules_by_category
            .entry(rule.category.clone())
            .or_insert_with(Vec::new)
            .push(id.clone());

        // Index by severity
        self.rules_by_severity
            .entry(rule.severity)
            .or_insert_with(Vec::new)
            .push(id.clone());

        // Index by WCAG level
        if let Some(level) = rule.wcag_level {
            self.rules_by_wcag_level
                .entry(level)
                .or_insert_with(Vec::new)
                .push(id.clone());
        }

        self.rules.insert(id, rule);
    }

    /// Get a rule by ID
    pub fn get_rule(&self, id: &str) -> Option<&AccessibilityRule> {
        self.rules.get(id)
    }

    /// Get all rules
    pub fn all_rules(&self) -> impl Iterator<Item = &AccessibilityRule> {
        self.rules.values()
    }

    /// Get rules by category
    pub fn rules_by_category(&self, category: &RuleCategory) -> Vec<&AccessibilityRule> {
        self.rules_by_category
            .get(category)
            .map(|ids| ids.iter().filter_map(|id| self.rules.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get rules by severity
    pub fn rules_by_severity(&self, severity: RuleSeverity) -> Vec<&AccessibilityRule> {
        self.rules_by_severity
            .get(&severity)
            .map(|ids| ids.iter().filter_map(|id| self.rules.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get rules by WCAG level
    pub fn rules_by_wcag_level(&self, level: WcagLevel) -> Vec<&AccessibilityRule> {
        self.rules_by_wcag_level
            .get(&level)
            .map(|ids| ids.iter().filter_map(|id| self.rules.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get enabled rules
    pub fn enabled_rules(&self) -> impl Iterator<Item = &AccessibilityRule> {
        self.rules.values().filter(|r| r.enabled_by_default)
    }
}

impl Default for RuleSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Main rules engine
pub struct RuleEngine {
    rule_sets: HashMap<String, RuleSet>,
    active_standards: HashSet<String>,
}

impl RuleEngine {
    /// Create a new rules engine
    pub fn new() -> Self {
        let mut engine = Self {
            rule_sets: HashMap::new(),
            active_standards: HashSet::new(),
        };

        // Load default rule sets
        engine.load_wcag21_rules();
        engine.load_wcag22_rules();
        engine.load_section508_rules();
        engine.load_ada_rules();
        engine.load_en301549_rules();

        engine
    }

    /// Load WCAG 2.1 rules
    fn load_wcag21_rules(&mut self) {
        let mut ruleset = RuleSet::new();

        // Principle 1: Perceivable
        ruleset.add_rule(
            AccessibilityRule::new("wcag21-1.1.1", "Non-text Content")
                .with_description("All non-text content has a text alternative")
                .with_wcag_level(WcagLevel::A)
                .with_wcag_criterion("1.1.1")
                .with_wcag_techniques(vec!["G94", "H37", "ARIA6"])
                .with_category(RuleCategory::Perceivable)
                .with_severity(RuleSeverity::Critical)
                .with_tag("images")
                .with_tag("alt-text"),
        );

        ruleset.add_rule(
            AccessibilityRule::new("wcag21-1.4.3", "Contrast (Minimum)")
                .with_description("Text has a contrast ratio of at least 4.5:1")
                .with_wcag_level(WcagLevel::AA)
                .with_wcag_criterion("1.4.3")
                .with_wcag_techniques(vec!["G18", "G145"])
                .with_category(RuleCategory::Perceivable)
                .with_severity(RuleSeverity::Critical)
                .with_tag("color")
                .with_tag("contrast"),
        );

        ruleset.add_rule(
            AccessibilityRule::new("wcag21-1.4.6", "Contrast (Enhanced)")
                .with_description("Text has a contrast ratio of at least 7:1")
                .with_wcag_level(WcagLevel::AAA)
                .with_wcag_criterion("1.4.6")
                .with_wcag_techniques(vec!["G17", "G18"])
                .with_category(RuleCategory::Perceivable)
                .with_severity(RuleSeverity::Major)
                .with_tag("color")
                .with_tag("contrast"),
        );

        // Principle 2: Operable
        ruleset.add_rule(
            AccessibilityRule::new("wcag21-2.1.1", "Keyboard")
                .with_description("All functionality is operable through keyboard")
                .with_wcag_level(WcagLevel::A)
                .with_wcag_criterion("2.1.1")
                .with_wcag_techniques(vec!["G202", "H91"])
                .with_category(RuleCategory::Operable)
                .with_severity(RuleSeverity::Critical)
                .with_tag("keyboard"),
        );

        ruleset.add_rule(
            AccessibilityRule::new("wcag21-2.1.2", "No Keyboard Trap")
                .with_description("Keyboard focus can be moved away from component")
                .with_wcag_level(WcagLevel::A)
                .with_wcag_criterion("2.1.2")
                .with_wcag_techniques(vec!["G21"])
                .with_category(RuleCategory::Operable)
                .with_severity(RuleSeverity::Critical)
                .with_tag("keyboard")
                .with_tag("focus"),
        );

        ruleset.add_rule(
            AccessibilityRule::new("wcag21-2.4.1", "Bypass Blocks")
                .with_description("Mechanism to bypass repeated blocks of content")
                .with_wcag_level(WcagLevel::A)
                .with_wcag_criterion("2.4.1")
                .with_wcag_techniques(vec!["G1", "G123", "G124"])
                .with_category(RuleCategory::Operable)
                .with_severity(RuleSeverity::Major)
                .with_tag("navigation")
                .with_tag("landmarks"),
        );

        ruleset.add_rule(
            AccessibilityRule::new("wcag21-2.4.2", "Page Titled")
                .with_description("Web pages have titles that describe topic or purpose")
                .with_wcag_level(WcagLevel::A)
                .with_wcag_criterion("2.4.2")
                .with_wcag_techniques(vec!["G88", "H25"])
                .with_category(RuleCategory::Operable)
                .with_severity(RuleSeverity::Major)
                .with_tag("navigation")
                .with_tag("title"),
        );

        ruleset.add_rule(
            AccessibilityRule::new("wcag21-2.4.3", "Focus Order")
                .with_description("Focusable components receive focus in meaningful order")
                .with_wcag_level(WcagLevel::A)
                .with_wcag_criterion("2.4.3")
                .with_wcag_techniques(vec!["G59", "H4"])
                .with_category(RuleCategory::Operable)
                .with_severity(RuleSeverity::Major)
                .with_tag("keyboard")
                .with_tag("focus"),
        );

        ruleset.add_rule(
            AccessibilityRule::new("wcag21-2.4.4", "Link Purpose")
                .with_description("Purpose of each link can be determined from link text")
                .with_wcag_level(WcagLevel::A)
                .with_wcag_criterion("2.4.4")
                .with_wcag_techniques(vec!["G91", "H30", "H24"])
                .with_category(RuleCategory::Operable)
                .with_severity(RuleSeverity::Minor)
                .with_tag("links")
                .with_tag("navigation"),
        );

        ruleset.add_rule(
            AccessibilityRule::new("wcag21-2.4.6", "Headings and Labels")
                .with_description("Headings and labels describe topic or purpose")
                .with_wcag_level(WcagLevel::AA)
                .with_wcag_criterion("2.4.6")
                .with_wcag_techniques(vec!["G130", "G131"])
                .with_category(RuleCategory::Operable)
                .with_severity(RuleSeverity::Minor)
                .with_tag("headings")
                .with_tag("labels"),
        );

        ruleset.add_rule(
            AccessibilityRule::new("wcag21-2.4.7", "Focus Visible")
                .with_description("Keyboard focus indicator is visible")
                .with_wcag_level(WcagLevel::AA)
                .with_wcag_criterion("2.4.7")
                .with_wcag_techniques(vec!["G149", "G165"])
                .with_category(RuleCategory::Operable)
                .with_severity(RuleSeverity::Major)
                .with_tag("focus")
                .with_tag("keyboard"),
        );

        // Principle 3: Understandable
        ruleset.add_rule(
            AccessibilityRule::new("wcag21-3.1.1", "Language of Page")
                .with_description("Default human language of page can be determined")
                .with_wcag_level(WcagLevel::A)
                .with_wcag_criterion("3.1.1")
                .with_wcag_techniques(vec!["H57"])
                .with_category(RuleCategory::Understandable)
                .with_severity(RuleSeverity::Major)
                .with_tag("language"),
        );

        ruleset.add_rule(
            AccessibilityRule::new("wcag21-3.2.1", "On Focus")
                .with_description("No context change when component receives focus")
                .with_wcag_level(WcagLevel::A)
                .with_wcag_criterion("3.2.1")
                .with_wcag_techniques(vec!["G107"])
                .with_category(RuleCategory::Understandable)
                .with_severity(RuleSeverity::Major)
                .with_tag("focus")
                .with_tag("behavior"),
        );

        ruleset.add_rule(
            AccessibilityRule::new("wcag21-3.3.1", "Error Identification")
                .with_description("Input errors are identified and described to user")
                .with_wcag_level(WcagLevel::A)
                .with_wcag_criterion("3.3.1")
                .with_wcag_techniques(vec!["G83", "G84", "G85"])
                .with_category(RuleCategory::Understandable)
                .with_severity(RuleSeverity::Critical)
                .with_tag("forms")
                .with_tag("errors"),
        );

        ruleset.add_rule(
            AccessibilityRule::new("wcag21-3.3.2", "Labels or Instructions")
                .with_description("Labels or instructions provided for user input")
                .with_wcag_level(WcagLevel::A)
                .with_wcag_criterion("3.3.2")
                .with_wcag_techniques(vec!["G131", "G89", "G184"])
                .with_category(RuleCategory::Understandable)
                .with_severity(RuleSeverity::Critical)
                .with_tag("forms")
                .with_tag("labels"),
        );

        // Principle 4: Robust
        ruleset.add_rule(
            AccessibilityRule::new("wcag21-4.1.1", "Parsing")
                .with_description("Content is well-formed with complete start and end tags")
                .with_wcag_level(WcagLevel::A)
                .with_wcag_criterion("4.1.1")
                .with_wcag_techniques(vec!["G134", "G192", "H74", "H93"])
                .with_category(RuleCategory::Robust)
                .with_severity(RuleSeverity::Major)
                .with_tag("html")
                .with_tag("validation"),
        );

        ruleset.add_rule(
            AccessibilityRule::new("wcag21-4.1.2", "Name, Role, Value")
                .with_description("Name and role can be determined; states can be set")
                .with_wcag_level(WcagLevel::A)
                .with_wcag_criterion("4.1.2")
                .with_wcag_techniques(vec!["G108", "H91", "ARIA14", "ARIA16"])
                .with_category(RuleCategory::Robust)
                .with_severity(RuleSeverity::Critical)
                .with_tag("aria")
                .with_tag("semantics"),
        );

        self.rule_sets.insert("WCAG21".to_string(), ruleset);
    }

    /// Load WCAG 2.2 rules (includes 2.1 + new criteria)
    fn load_wcag22_rules(&mut self) {
        let mut ruleset = RuleSet::new();

        // Include all WCAG 2.1 rules
        if let Some(wcag21) = self.rule_sets.get("WCAG21") {
            for rule in wcag21.all_rules() {
                ruleset.add_rule(rule.clone());
            }
        }

        // Add WCAG 2.2 specific rules
        ruleset.add_rule(
            AccessibilityRule::new("wcag22-2.4.11", "Focus Not Obscured (Minimum)")
                .with_description("Focused element is not entirely hidden by author-created content")
                .with_wcag_level(WcagLevel::AA)
                .with_wcag_criterion("2.4.11")
                .with_category(RuleCategory::Operable)
                .with_severity(RuleSeverity::Major)
                .with_tag("focus")
                .with_tag("visibility"),
        );

        ruleset.add_rule(
            AccessibilityRule::new("wcag22-2.4.12", "Focus Not Obscured (Enhanced)")
                .with_description("No part of focused element is hidden by author-created content")
                .with_wcag_level(WcagLevel::AAA)
                .with_wcag_criterion("2.4.12")
                .with_category(RuleCategory::Operable)
                .with_severity(RuleSeverity::Minor)
                .with_tag("focus")
                .with_tag("visibility"),
        );

        ruleset.add_rule(
            AccessibilityRule::new("wcag22-2.4.13", "Focus Appearance")
                .with_description("Keyboard focus indicator meets minimum size and contrast")
                .with_wcag_level(WcagLevel::AAA)
                .with_wcag_criterion("2.4.13")
                .with_category(RuleCategory::Operable)
                .with_severity(RuleSeverity::Minor)
                .with_tag("focus")
                .with_tag("contrast"),
        );

        ruleset.add_rule(
            AccessibilityRule::new("wcag22-2.5.7", "Dragging Movements")
                .with_description("Dragging functionality can be achieved by single pointer")
                .with_wcag_level(WcagLevel::AA)
                .with_wcag_criterion("2.5.7")
                .with_category(RuleCategory::Operable)
                .with_severity(RuleSeverity::Major)
                .with_tag("interaction")
                .with_tag("pointer"),
        );

        ruleset.add_rule(
            AccessibilityRule::new("wcag22-2.5.8", "Target Size (Minimum)")
                .with_description("Target size is at least 24x24 CSS pixels")
                .with_wcag_level(WcagLevel::AA)
                .with_wcag_criterion("2.5.8")
                .with_category(RuleCategory::Operable)
                .with_severity(RuleSeverity::Major)
                .with_tag("interaction")
                .with_tag("touch"),
        );

        ruleset.add_rule(
            AccessibilityRule::new("wcag22-3.2.6", "Consistent Help")
                .with_description("Help mechanisms appear in consistent order")
                .with_wcag_level(WcagLevel::A)
                .with_wcag_criterion("3.2.6")
                .with_category(RuleCategory::Understandable)
                .with_severity(RuleSeverity::Minor)
                .with_tag("help")
                .with_tag("consistency"),
        );

        ruleset.add_rule(
            AccessibilityRule::new("wcag22-3.3.7", "Redundant Entry")
                .with_description("Information previously entered is auto-populated or available")
                .with_wcag_level(WcagLevel::A)
                .with_wcag_criterion("3.3.7")
                .with_category(RuleCategory::Understandable)
                .with_severity(RuleSeverity::Minor)
                .with_tag("forms")
                .with_tag("input"),
        );

        ruleset.add_rule(
            AccessibilityRule::new("wcag22-3.3.8", "Accessible Authentication (Minimum)")
                .with_description("Cognitive function test not required for authentication")
                .with_wcag_level(WcagLevel::AA)
                .with_wcag_criterion("3.3.8")
                .with_category(RuleCategory::Understandable)
                .with_severity(RuleSeverity::Major)
                .with_tag("authentication")
                .with_tag("cognitive"),
        );

        self.rule_sets.insert("WCAG22".to_string(), ruleset);
    }

    /// Load Section 508 rules
    fn load_section508_rules(&mut self) {
        let mut ruleset = RuleSet::new();

        ruleset.add_rule(
            AccessibilityRule::new("508-1194.22-a", "Text Equivalents")
                .with_description("Text equivalent for every non-text element")
                .with_severity(RuleSeverity::Critical)
                .with_category(RuleCategory::Perceivable)
                .with_tag("section508")
                .with_tag("images"),
        );

        ruleset.add_rule(
            AccessibilityRule::new("508-1194.22-b", "Multimedia Alternatives")
                .with_description("Equivalent alternatives for multimedia presentations")
                .with_severity(RuleSeverity::Critical)
                .with_category(RuleCategory::Perceivable)
                .with_tag("section508")
                .with_tag("multimedia"),
        );

        ruleset.add_rule(
            AccessibilityRule::new("508-1194.22-c", "Color Information")
                .with_description("Information not conveyed by color alone")
                .with_severity(RuleSeverity::Major)
                .with_category(RuleCategory::Perceivable)
                .with_tag("section508")
                .with_tag("color"),
        );

        ruleset.add_rule(
            AccessibilityRule::new("508-1194.22-d", "Readable Without Stylesheet")
                .with_description("Documents readable without stylesheets")
                .with_severity(RuleSeverity::Minor)
                .with_category(RuleCategory::Perceivable)
                .with_tag("section508")
                .with_tag("css"),
        );

        self.rule_sets.insert("Section508".to_string(), ruleset);
    }

    /// Load ADA rules
    fn load_ada_rules(&mut self) {
        let mut ruleset = RuleSet::new();

        ruleset.add_rule(
            AccessibilityRule::new("ada-title2", "Public Accommodations")
                .with_description("Website must be accessible as public accommodation")
                .with_severity(RuleSeverity::Critical)
                .with_category(RuleCategory::Perceivable)
                .with_tag("ada")
                .with_tag("compliance"),
        );

        ruleset.add_rule(
            AccessibilityRule::new("ada-effective-communication", "Effective Communication")
                .with_description("Effective communication with people with disabilities")
                .with_severity(RuleSeverity::Critical)
                .with_category(RuleCategory::Understandable)
                .with_tag("ada")
                .with_tag("communication"),
        );

        self.rule_sets.insert("ADA".to_string(), ruleset);
    }

    /// Load EN 301 549 rules
    fn load_en301549_rules(&mut self) {
        let mut ruleset = RuleSet::new();

        ruleset.add_rule(
            AccessibilityRule::new("en301549-9.1.1.1", "Non-text Content")
                .with_description("Text alternatives for non-text content (EN 301 549)")
                .with_severity(RuleSeverity::Critical)
                .with_category(RuleCategory::Perceivable)
                .with_tag("en301549")
                .with_tag("images"),
        );

        ruleset.add_rule(
            AccessibilityRule::new("en301549-9.2.1.1", "Keyboard")
                .with_description("Keyboard accessibility (EN 301 549)")
                .with_severity(RuleSeverity::Critical)
                .with_category(RuleCategory::Operable)
                .with_tag("en301549")
                .with_tag("keyboard"),
        );

        self.rule_sets.insert("EN301549".to_string(), ruleset);
    }

    /// Get rules for specific standards
    pub fn get_rules_for_standards(&self, standards: &[String]) -> RuleSet {
        let mut combined = RuleSet::new();

        for standard in standards {
            if let Some(ruleset) = self.rule_sets.get(standard) {
                for rule in ruleset.all_rules() {
                    combined.add_rule(rule.clone());
                }
            }
        }

        combined
    }

    /// Get a specific rule
    pub fn get_rule(&self, standard: &str, rule_id: &str) -> Option<&AccessibilityRule> {
        self.rule_sets.get(standard)?.get_rule(rule_id)
    }

    /// Activate standards
    pub fn activate_standards(&mut self, standards: Vec<String>) {
        self.active_standards = standards.into_iter().collect();
    }

    /// Get active rules
    pub fn active_rules(&self) -> RuleSet {
        self.get_rules_for_standards(&self.active_standards.iter().cloned().collect::<Vec<_>>())
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Global rule engine instance
pub static RULE_ENGINE: Lazy<RuleEngine> = Lazy::new(RuleEngine::new);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_creation() {
        let rule = AccessibilityRule::new("test-1", "Test Rule")
            .with_description("A test rule")
            .with_severity(RuleSeverity::Critical)
            .with_wcag_level(WcagLevel::AA)
            .with_wcag_criterion("1.1.1")
            .with_tag("test");

        assert_eq!(rule.id, "test-1");
        assert_eq!(rule.name, "Test Rule");
        assert_eq!(rule.severity, RuleSeverity::Critical);
        assert_eq!(rule.wcag_level, Some(WcagLevel::AA));
        assert!(rule.tags.contains("test"));
    }

    #[test]
    fn test_ruleset_indexing() {
        let mut ruleset = RuleSet::new();

        ruleset.add_rule(
            AccessibilityRule::new("test-1", "Test 1")
                .with_severity(RuleSeverity::Critical)
                .with_category(RuleCategory::Perceivable),
        );

        ruleset.add_rule(
            AccessibilityRule::new("test-2", "Test 2")
                .with_severity(RuleSeverity::Minor)
                .with_category(RuleCategory::Operable),
        );

        assert_eq!(ruleset.all_rules().count(), 2);
        assert_eq!(ruleset.rules_by_severity(RuleSeverity::Critical).len(), 1);
        assert_eq!(
            ruleset
                .rules_by_category(&RuleCategory::Perceivable)
                .len(),
            1
        );
    }

    #[test]
    fn test_rule_engine_initialization() {
        let engine = RuleEngine::new();

        assert!(engine.rule_sets.contains_key("WCAG21"));
        assert!(engine.rule_sets.contains_key("WCAG22"));
        assert!(engine.rule_sets.contains_key("Section508"));
        assert!(engine.rule_sets.contains_key("ADA"));
        assert!(engine.rule_sets.contains_key("EN301549"));
    }

    #[test]
    fn test_wcag_levels() {
        let engine = RuleEngine::new();
        let wcag21 = engine.rule_sets.get("WCAG21").unwrap();

        let level_a_rules = wcag21.rules_by_wcag_level(WcagLevel::A);
        let level_aa_rules = wcag21.rules_by_wcag_level(WcagLevel::AA);
        let level_aaa_rules = wcag21.rules_by_wcag_level(WcagLevel::AAA);

        assert!(!level_a_rules.is_empty());
        assert!(!level_aa_rules.is_empty());
        assert!(!level_aaa_rules.is_empty());
    }
}
