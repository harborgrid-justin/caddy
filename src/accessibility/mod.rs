//! # Accessibility Module
//!
//! Enterprise-grade accessibility scanning and remediation engine for CADDY v0.3.0.
//!
//! This module provides comprehensive WCAG 2.1/2.2, Section 508, ADA, and EN 301 549
//! compliance checking with automated remediation suggestions.
//!
//! ## Features
//!
//! - **WCAG 2.1/2.2 Compliance**: Full support for Levels A, AA, and AAA
//! - **Color Contrast Analysis**: AA/AAA compliance checking
//! - **Keyboard Navigation**: Complete keyboard accessibility validation
//! - **Screen Reader Support**: ARIA and semantic HTML validation
//! - **Automated Remediation**: AI-powered fix recommendations
//! - **Multi-Standard Support**: WCAG, Section 508, ADA, EN 301 549
//!
//! ## Example
//!
//! ```rust,no_run
//! use caddy::accessibility::{Scanner, ScanConfig, ComplianceLevel};
//!
//! let config = ScanConfig::new()
//!     .with_level(ComplianceLevel::AA)
//!     .with_standards(vec!["WCAG21", "Section508"]);
//!
//! let scanner = Scanner::new(config);
//! let results = scanner.scan_document("<html>...</html>")?;
//!
//! for violation in results.violations() {
//!     println!("Violation: {:?}", violation);
//!     println!("Remediation: {:?}", violation.suggested_fixes());
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod scanner;
pub mod rules;
pub mod analyzer;
pub mod remediation;

// Re-export commonly used types
pub use scanner::{
    AccessibilityScanner, ScanConfig, ScanResult, ScanStatus,
    ComplianceLevel, Scanner, ColorContrastChecker, KeyboardNavigationValidator,
};

pub use rules::{
    AccessibilityRule, RuleEngine, RuleSet, RuleSeverity, RuleCategory,
    WcagRule, Section508Rule, AdaRule, En301549Rule,
    WcagLevel, RuleViolation,
};

pub use analyzer::{
    AccessibilityAnalyzer, DomAnalyzer, SemanticAnalyzer, AriaAnalyzer,
    AnalysisResult, StructuralIssue, SemanticIssue, AriaIssue,
};

pub use remediation::{
    RemediationEngine, RemediationSuggestion, RemediationType,
    FixPriority, AutoFixEngine, BulkRemediationQueue,
};

/// Version of the accessibility engine
pub const ACCESSIBILITY_ENGINE_VERSION: &str = "0.3.0";

/// Supported WCAG versions
pub const SUPPORTED_WCAG_VERSIONS: &[&str] = &["2.1", "2.2"];

/// Supported standards
pub const SUPPORTED_STANDARDS: &[&str] = &["WCAG21", "WCAG22", "Section508", "ADA", "EN301549"];
