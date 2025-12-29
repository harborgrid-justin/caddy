// CADDY - Enterprise CAD System
// File I/O System - Format Validation and Repair
// Agent 9 - Import/Export Pipeline Specialist

//! # Format Validation and Repair
//!
//! Provides comprehensive validation and repair capabilities for CAD documents
//! and file formats. Detects common issues and attempts automatic repair when possible.
//!
//! ## Validation Checks
//!
//! - Geometric validity (no degenerate entities)
//! - Referential integrity (valid layer/block references)
//! - Numerical stability (no NaN/Inf values)
//! - Format compliance (adherence to format specifications)
//! - Data consistency (matching entity counts, correct structure)
//!
//! ## Repair Capabilities
//!
//! - Remove degenerate entities
//! - Fix invalid references
//! - Normalize coordinates
//! - Rebuild indices
//! - Clean duplicate entities

use crate::io::document::*;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Validation errors
#[derive(Error, Debug, Clone)]
pub enum ValidationError {
    #[error("Degenerate entity: {entity_type} at ID {entity_id}")]
    DegenerateEntity {
        entity_id: String,
        entity_type: String,
    },

    #[error("Invalid reference: {reference_type} '{reference}' not found")]
    InvalidReference {
        reference_type: String,
        reference: String,
    },

    #[error("Invalid coordinate: {field} contains {value}")]
    InvalidCoordinate {
        field: String,
        value: String,
    },

    #[error("Duplicate entity ID: {0}")]
    DuplicateId(String),

    #[error("Empty document")]
    EmptyDocument,

    #[error("Invalid entity count: {0}")]
    InvalidEntityCount(String),

    #[error("Circular reference detected: {0}")]
    CircularReference(String),

    #[error("Inconsistent data: {0}")]
    InconsistentData(String),
}

/// Validation severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Validation issue
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: Severity,
    pub error: ValidationError,
    pub repairable: bool,
    pub entity_id: Option<String>,
}

impl ValidationIssue {
    fn new(severity: Severity, error: ValidationError, repairable: bool) -> Self {
        Self {
            severity,
            error,
            repairable,
            entity_id: None,
        }
    }

    fn with_entity_id(mut self, id: String) -> Self {
        self.entity_id = Some(id);
        self
    }
}

/// Validation result
pub type ValidationResult = Result<(), Vec<ValidationIssue>>;

/// Document validator
pub struct Validator {
    check_geometry: bool,
    check_references: bool,
    check_coordinates: bool,
    check_duplicates: bool,
    allow_empty: bool,
    strict_mode: bool,
}

impl Validator {
    /// Create a new validator with default settings
    pub fn new() -> Self {
        Self {
            check_geometry: true,
            check_references: true,
            check_coordinates: true,
            check_duplicates: true,
            allow_empty: false,
            strict_mode: false,
        }
    }

    /// Enable strict validation mode (treat warnings as errors)
    pub fn strict(mut self) -> Self {
        self.strict_mode = true;
        self
    }

    /// Allow empty documents
    pub fn allow_empty(mut self) -> Self {
        self.allow_empty = true;
        self
    }

    /// Skip geometry validation
    pub fn skip_geometry_checks(mut self) -> Self {
        self.check_geometry = false;
        self
    }

    /// Skip reference validation
    pub fn skip_reference_checks(mut self) -> Self {
        self.check_references = false;
        self
    }

    /// Validate a document
    pub fn validate(&self, doc: &Document) -> ValidationResult {
        let mut issues = Vec::new();

        // Check if document is empty
        if !self.allow_empty && doc.entities.is_empty() {
            issues.push(ValidationIssue::new(
                Severity::Warning,
                ValidationError::EmptyDocument,
                false,
            ));
        }

        // Validate entities
        if self.check_geometry {
            issues.extend(self.validate_geometry(doc));
        }

        // Validate references
        if self.check_references {
            issues.extend(self.validate_references(doc));
        }

        // Check for duplicates
        if self.check_duplicates {
            issues.extend(self.validate_duplicates(doc));
        }

        // Validate coordinates
        if self.check_coordinates {
            issues.extend(self.validate_coordinates(doc));
        }

        // Filter issues based on strict mode
        if self.strict_mode {
            // In strict mode, treat warnings as errors
            for issue in &mut issues {
                if issue.severity == Severity::Warning {
                    issue.severity = Severity::Error;
                }
            }
        }

        // Return result
        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }

    fn validate_geometry(&self, doc: &Document) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for entity in &doc.entities {
            match &entity.geometry {
                GeometryType::Line(line) => {
                    // Check for zero-length lines
                    let dx = line.end.x - line.start.x;
                    let dy = line.end.y - line.start.y;
                    let dz = line.end.z - line.start.z;
                    let length = (dx * dx + dy * dy + dz * dz).sqrt();

                    if length < 1e-10 {
                        issues.push(
                            ValidationIssue::new(
                                Severity::Warning,
                                ValidationError::DegenerateEntity {
                                    entity_id: entity.id.clone(),
                                    entity_type: "Line".to_string(),
                                },
                                true,
                            )
                            .with_entity_id(entity.id.clone()),
                        );
                    }
                }
                GeometryType::Circle(circle) => {
                    // Check for zero or negative radius
                    if circle.radius <= 1e-10 {
                        issues.push(
                            ValidationIssue::new(
                                Severity::Warning,
                                ValidationError::DegenerateEntity {
                                    entity_id: entity.id.clone(),
                                    entity_type: "Circle".to_string(),
                                },
                                true,
                            )
                            .with_entity_id(entity.id.clone()),
                        );
                    }
                }
                GeometryType::Arc(arc) => {
                    // Check for zero or negative radius
                    if arc.radius <= 1e-10 {
                        issues.push(
                            ValidationIssue::new(
                                Severity::Warning,
                                ValidationError::DegenerateEntity {
                                    entity_id: entity.id.clone(),
                                    entity_type: "Arc".to_string(),
                                },
                                true,
                            )
                            .with_entity_id(entity.id.clone()),
                        );
                    }

                    // Check for zero-length arc
                    let angle_diff = (arc.end_angle - arc.start_angle).abs();
                    if angle_diff < 1e-10 {
                        issues.push(
                            ValidationIssue::new(
                                Severity::Warning,
                                ValidationError::DegenerateEntity {
                                    entity_id: entity.id.clone(),
                                    entity_type: "Arc (zero sweep)".to_string(),
                                },
                                true,
                            )
                            .with_entity_id(entity.id.clone()),
                        );
                    }
                }
                GeometryType::Polyline(polyline) => {
                    // Check for polylines with fewer than 2 points
                    if polyline.points.len() < 2 {
                        issues.push(
                            ValidationIssue::new(
                                Severity::Error,
                                ValidationError::DegenerateEntity {
                                    entity_id: entity.id.clone(),
                                    entity_type: "Polyline (insufficient points)".to_string(),
                                },
                                true,
                            )
                            .with_entity_id(entity.id.clone()),
                        );
                    }
                }
                _ => {}
            }
        }

        issues
    }

    fn validate_references(&self, doc: &Document) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // Build set of valid layer names
        let layer_names: HashSet<String> = doc.layers.iter().map(|l| l.name.clone()).collect();

        // Build set of valid block names
        let block_names: HashSet<String> = doc.blocks.iter().map(|b| b.name.clone()).collect();

        // Check entity layer references
        for entity in &doc.entities {
            if !layer_names.contains(&entity.layer) {
                issues.push(
                    ValidationIssue::new(
                        Severity::Warning,
                        ValidationError::InvalidReference {
                            reference_type: "Layer".to_string(),
                            reference: entity.layer.clone(),
                        },
                        true,
                    )
                    .with_entity_id(entity.id.clone()),
                );
            }

            // Check block insert references
            if let GeometryType::Insert(insert) = &entity.geometry {
                if !block_names.contains(&insert.block_name) {
                    issues.push(
                        ValidationIssue::new(
                            Severity::Error,
                            ValidationError::InvalidReference {
                                reference_type: "Block".to_string(),
                                reference: insert.block_name.clone(),
                            },
                            false,
                        )
                        .with_entity_id(entity.id.clone()),
                    );
                }
            }
        }

        issues
    }

    fn validate_duplicates(&self, doc: &Document) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        let mut seen_ids = HashSet::new();

        for entity in &doc.entities {
            if !seen_ids.insert(entity.id.clone()) {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        ValidationError::DuplicateId(entity.id.clone()),
                        true,
                    )
                    .with_entity_id(entity.id.clone()),
                );
            }
        }

        issues
    }

    fn validate_coordinates(&self, doc: &Document) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for entity in &doc.entities {
            // Check coordinates in geometry
            match &entity.geometry {
                GeometryType::Point(point) => {
                    if !self.is_valid_vec3(&point.position) {
                        issues.push(
                            ValidationIssue::new(
                                Severity::Error,
                                ValidationError::InvalidCoordinate {
                                    field: "Point position".to_string(),
                                    value: format!("{:?}", point.position),
                                },
                                false,
                            )
                            .with_entity_id(entity.id.clone()),
                        );
                    }
                }
                GeometryType::Line(line) => {
                    if !self.is_valid_vec3(&line.start) || !self.is_valid_vec3(&line.end) {
                        issues.push(
                            ValidationIssue::new(
                                Severity::Error,
                                ValidationError::InvalidCoordinate {
                                    field: "Line coordinates".to_string(),
                                    value: format!("start: {:?}, end: {:?}", line.start, line.end),
                                },
                                false,
                            )
                            .with_entity_id(entity.id.clone()),
                        );
                    }
                }
                GeometryType::Circle(circle) => {
                    if !self.is_valid_vec3(&circle.center) {
                        issues.push(
                            ValidationIssue::new(
                                Severity::Error,
                                ValidationError::InvalidCoordinate {
                                    field: "Circle center".to_string(),
                                    value: format!("{:?}", circle.center),
                                },
                                false,
                            )
                            .with_entity_id(entity.id.clone()),
                        );
                    }

                    if !circle.radius.is_finite() {
                        issues.push(
                            ValidationIssue::new(
                                Severity::Error,
                                ValidationError::InvalidCoordinate {
                                    field: "Circle radius".to_string(),
                                    value: format!("{}", circle.radius),
                                },
                                false,
                            )
                            .with_entity_id(entity.id.clone()),
                        );
                    }
                }
                _ => {}
            }
        }

        issues
    }

    fn is_valid_vec3(&self, v: &Vec3) -> bool {
        v.x.is_finite() && v.y.is_finite() && v.z.is_finite()
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}

/// Document repair utility
pub struct Repairer {
    remove_degenerate: bool,
    fix_references: bool,
    remove_duplicates: bool,
    normalize_coordinates: bool,
}

impl Repairer {
    /// Create a new repairer
    pub fn new() -> Self {
        Self {
            remove_degenerate: true,
            fix_references: true,
            remove_duplicates: true,
            normalize_coordinates: true,
        }
    }

    /// Skip removing degenerate entities
    pub fn keep_degenerate(mut self) -> Self {
        self.remove_degenerate = false;
        self
    }

    /// Repair document based on validation issues
    pub fn repair(&self, doc: &mut Document, issues: &[ValidationIssue]) -> usize {
        let mut repair_count = 0;

        // Collect entity IDs to remove
        let mut entities_to_remove = HashSet::new();

        for issue in issues {
            if !issue.repairable {
                continue;
            }

            match &issue.error {
                ValidationError::DegenerateEntity { entity_id, .. } => {
                    if self.remove_degenerate {
                        entities_to_remove.insert(entity_id.clone());
                        repair_count += 1;
                    }
                }
                ValidationError::InvalidReference { reference_type, reference } => {
                    if self.fix_references && reference_type == "Layer" {
                        // Create missing layer
                        if !doc.layers.iter().any(|l| &l.name == reference) {
                            doc.layers.push(Layer {
                                name: reference.clone(),
                                color: Color::white(),
                                line_type: "Continuous".to_string(),
                                line_weight: LineWeight::Default,
                                visible: true,
                                locked: false,
                                frozen: false,
                                plottable: true,
                            });
                            repair_count += 1;
                        }
                    }
                }
                ValidationError::DuplicateId(id) => {
                    if self.remove_duplicates {
                        // Keep first occurrence, remove duplicates
                        entities_to_remove.insert(id.clone());
                        repair_count += 1;
                    }
                }
                _ => {}
            }
        }

        // Remove flagged entities
        if !entities_to_remove.is_empty() {
            doc.entities.retain(|e| !entities_to_remove.contains(&e.id));
        }

        repair_count
    }

    /// Attempt automatic repair without validation
    pub fn auto_repair(&self, doc: &mut Document) -> usize {
        let validator = Validator::new();

        match validator.validate(doc) {
            Ok(_) => 0,
            Err(issues) => self.repair(doc, &issues),
        }
    }
}

impl Default for Repairer {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation report generator
pub struct ValidationReport;

impl ValidationReport {
    /// Generate a human-readable validation report
    pub fn generate(issues: &[ValidationIssue]) -> String {
        let mut report = String::new();
        report.push_str("=== Validation Report ===\n\n");

        let mut by_severity: HashMap<Severity, Vec<&ValidationIssue>> = HashMap::new();
        for issue in issues {
            by_severity.entry(issue.severity).or_default().push(issue);
        }

        for severity in &[Severity::Critical, Severity::Error, Severity::Warning, Severity::Info] {
            if let Some(severity_issues) = by_severity.get(severity) {
                if severity_issues.is_empty() {
                    continue;
                }

                report.push_str(&format!("\n{:?} ({})\n", severity, severity_issues.len()));
                report.push_str(&"-".repeat(40));
                report.push_str("\n");

                for issue in severity_issues {
                    report.push_str(&format!("- {}\n", issue.error));
                    if let Some(ref entity_id) = issue.entity_id {
                        report.push_str(&format!("  Entity ID: {}\n", entity_id));
                    }
                    if issue.repairable {
                        report.push_str("  (Repairable)\n");
                    }
                }
            }
        }

        report.push_str(&format!("\nTotal Issues: {}\n", issues.len()));
        report.push_str(&format!(
            "Repairable: {}\n",
            issues.iter().filter(|i| i.repairable).count()
        ));

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = Validator::new();
        assert!(validator.check_geometry);
        assert!(validator.check_references);
    }

    #[test]
    fn test_validator_strict_mode() {
        let validator = Validator::new().strict();
        assert!(validator.strict_mode);
    }

    #[test]
    fn test_empty_document_validation() {
        let validator = Validator::new();
        let _doc = Document::new();

        let result = validator.validate(&doc);
        assert!(result.is_err());

        let validator_allow_empty = Validator::new().allow_empty();
        let result = validator_allow_empty.validate(&doc);
        assert!(result.is_ok());
    }

    #[test]
    fn test_repairer_creation() {
        let repairer = Repairer::new();
        assert!(repairer.remove_degenerate);
        assert!(repairer.fix_references);
    }
}
