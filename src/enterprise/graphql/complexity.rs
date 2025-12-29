//! GraphQL Query Complexity Analysis
//!
//! Provides query cost calculation, depth limiting, field cost annotations,
//! and query rejection policies to prevent resource exhaustion attacks.

use super::query::{Document, FieldSelection, Operation, Selection};
use super::schema::{Field, ObjectType, Schema, TypeRef};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

// ============================================================================
// Error Types
// ============================================================================

/// Complexity analysis errors
#[derive(Error, Debug, Clone)]
pub enum ComplexityError {
    /// Query exceeds maximum depth
    #[error("Query depth {0} exceeds maximum {1}")]
    MaxDepthExceeded(usize, usize),

    /// Query exceeds maximum complexity
    #[error("Query complexity {0} exceeds maximum {1}")]
    MaxComplexityExceeded(f64, f64),

    /// Field not found
    #[error("Field '{0}' not found on type '{1}'")]
    FieldNotFound(String, String),

    /// Invalid complexity configuration
    #[error("Invalid complexity configuration: {0}")]
    InvalidConfig(String),

    /// Type not found
    #[error("Type not found: {0}")]
    TypeNotFound(String),
}

pub type ComplexityResult<T> = Result<T, ComplexityError>;

// ============================================================================
// Complexity Configuration
// ============================================================================

/// Complexity analyzer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityConfig {
    /// Maximum query depth (0 = unlimited)
    pub max_depth: usize,
    /// Maximum query complexity score (0.0 = unlimited)
    pub max_complexity: f64,
    /// Default field cost
    pub default_field_cost: f64,
    /// Cost multiplier for lists
    pub list_multiplier: f64,
    /// Enable depth limiting
    pub enable_depth_limit: bool,
    /// Enable complexity limiting
    pub enable_complexity_limit: bool,
    /// Reject queries exceeding limits
    pub reject_on_limit: bool,
}

impl Default for ComplexityConfig {
    fn default() -> Self {
        Self {
            max_depth: 10,
            max_complexity: 1000.0,
            default_field_cost: 1.0,
            list_multiplier: 10.0,
            enable_depth_limit: true,
            enable_complexity_limit: true,
            reject_on_limit: true,
        }
    }
}

impl ComplexityConfig {
    /// Create permissive configuration (for development)
    pub fn permissive() -> Self {
        Self {
            max_depth: 50,
            max_complexity: 10000.0,
            enable_depth_limit: false,
            enable_complexity_limit: false,
            reject_on_limit: false,
            ..Default::default()
        }
    }

    /// Create strict configuration (for production)
    pub fn strict() -> Self {
        Self {
            max_depth: 5,
            max_complexity: 100.0,
            enable_depth_limit: true,
            enable_complexity_limit: true,
            reject_on_limit: true,
            ..Default::default()
        }
    }
}

// ============================================================================
// Field Cost Annotations
// ============================================================================

/// Field cost calculator
pub trait FieldCostCalculator: Send + Sync {
    /// Calculate cost for a field
    fn calculate_cost(&self, args: &HashMap<String, super::schema::Value>) -> f64;
}

/// Static field cost
#[derive(Debug, Clone)]
pub struct StaticCost {
    /// Fixed cost value
    cost: f64,
}

impl StaticCost {
    /// Create a new static cost
    pub fn new(cost: f64) -> Self {
        Self { cost }
    }
}

impl FieldCostCalculator for StaticCost {
    fn calculate_cost(&self, _args: &HashMap<String, super::schema::Value>) -> f64 {
        self.cost
    }
}

/// Dynamic cost based on arguments
#[derive(Debug, Clone)]
pub struct DynamicCost {
    /// Base cost
    base_cost: f64,
    /// Argument-based multipliers
    arg_multipliers: HashMap<String, f64>,
}

impl DynamicCost {
    /// Create a new dynamic cost
    pub fn new(base_cost: f64) -> Self {
        Self {
            base_cost,
            arg_multipliers: HashMap::new(),
        }
    }

    /// Add argument multiplier
    pub fn with_arg_multiplier(mut self, arg_name: impl Into<String>, multiplier: f64) -> Self {
        self.arg_multipliers.insert(arg_name.into(), multiplier);
        self
    }
}

impl FieldCostCalculator for DynamicCost {
    fn calculate_cost(&self, args: &HashMap<String, super::schema::Value>) -> f64 {
        let mut cost = self.base_cost;

        for (arg_name, multiplier) in &self.arg_multipliers {
            if let Some(value) = args.get(arg_name) {
                // Try to extract numeric value
                if let Some(int_val) = value.as_int() {
                    cost += int_val as f64 * multiplier;
                } else if let Some(float_val) = value.as_float() {
                    cost += float_val * multiplier;
                }
            }
        }

        cost
    }
}

/// Field cost registry
pub struct FieldCostRegistry {
    /// Field costs by type and field name
    costs: HashMap<String, HashMap<String, Arc<dyn FieldCostCalculator>>>,
    /// Default cost
    default_cost: f64,
}

impl FieldCostRegistry {
    /// Create a new field cost registry
    pub fn new(default_cost: f64) -> Self {
        Self {
            costs: HashMap::new(),
            default_cost,
        }
    }

    /// Register a field cost
    pub fn register_cost(
        &mut self,
        type_name: impl Into<String>,
        field_name: impl Into<String>,
        calculator: Arc<dyn FieldCostCalculator>,
    ) {
        let type_name = type_name.into();
        let field_name = field_name.into();

        self.costs
            .entry(type_name)
            .or_insert_with(HashMap::new)
            .insert(field_name, calculator);
    }

    /// Get field cost calculator
    pub fn get_calculator(
        &self,
        type_name: &str,
        field_name: &str,
    ) -> Option<&Arc<dyn FieldCostCalculator>> {
        self.costs.get(type_name).and_then(|fields| fields.get(field_name))
    }

    /// Get default cost
    pub fn default_cost(&self) -> f64 {
        self.default_cost
    }
}

impl Default for FieldCostRegistry {
    fn default() -> Self {
        Self::new(1.0)
    }
}

// ============================================================================
// Complexity Analysis Result
// ============================================================================

/// Query complexity analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityAnalysis {
    /// Total complexity score
    pub total_complexity: f64,
    /// Maximum depth reached
    pub max_depth: usize,
    /// Field count
    pub field_count: usize,
    /// List field count
    pub list_field_count: usize,
    /// Breakdown by type
    pub type_breakdown: HashMap<String, f64>,
    /// Whether query exceeds limits
    pub exceeds_limits: bool,
    /// Limit violations
    pub violations: Vec<String>,
}

impl ComplexityAnalysis {
    /// Create a new analysis result
    pub fn new() -> Self {
        Self {
            total_complexity: 0.0,
            max_depth: 0,
            field_count: 0,
            list_field_count: 0,
            type_breakdown: HashMap::new(),
            exceeds_limits: false,
            violations: Vec::new(),
        }
    }

    /// Check if query is acceptable
    pub fn is_acceptable(&self) -> bool {
        !self.exceeds_limits
    }

    /// Add a violation
    pub fn add_violation(&mut self, message: impl Into<String>) {
        self.violations.push(message.into());
        self.exceeds_limits = true;
    }
}

impl Default for ComplexityAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Complexity Analyzer
// ============================================================================

/// Query complexity analyzer
pub struct ComplexityAnalyzer {
    /// Schema
    schema: Arc<Schema>,
    /// Configuration
    config: ComplexityConfig,
    /// Field cost registry
    cost_registry: Arc<FieldCostRegistry>,
}

impl ComplexityAnalyzer {
    /// Create a new complexity analyzer
    pub fn new(schema: Arc<Schema>) -> Self {
        Self {
            schema,
            config: ComplexityConfig::default(),
            cost_registry: Arc::new(FieldCostRegistry::default()),
        }
    }

    /// Create analyzer with custom configuration
    pub fn with_config(schema: Arc<Schema>, config: ComplexityConfig) -> Self {
        Self {
            schema,
            config,
            cost_registry: Arc::new(FieldCostRegistry::default()),
        }
    }

    /// Set field cost registry
    pub fn with_cost_registry(mut self, registry: Arc<FieldCostRegistry>) -> Self {
        self.cost_registry = registry;
        self
    }

    /// Analyze a document
    pub fn analyze(&self, document: &Document) -> ComplexityResult<ComplexityAnalysis> {
        let mut analysis = ComplexityAnalysis::new();

        for operation in &document.operations {
            self.analyze_operation(operation, &mut analysis)?;
        }

        // Check limits
        self.check_limits(&mut analysis)?;

        Ok(analysis)
    }

    /// Analyze an operation
    fn analyze_operation(
        &self,
        operation: &Operation,
        analysis: &mut ComplexityAnalysis,
    ) -> ComplexityResult<()> {
        let root_type = match operation.operation_type {
            super::query::OperationType::Query => self.schema.query_type(),
            super::query::OperationType::Mutation => self.schema.mutation_type(),
            super::query::OperationType::Subscription => self.schema.subscription_type(),
        };

        if let Some(root_type) = root_type {
            self.analyze_selections(&operation.selections, root_type, 1, 1.0, analysis)?;
        }

        Ok(())
    }

    /// Analyze selections
    fn analyze_selections(
        &self,
        selections: &[Selection],
        parent_type: &ObjectType,
        depth: usize,
        multiplier: f64,
        analysis: &mut ComplexityAnalysis,
    ) -> ComplexityResult<()> {
        // Update max depth
        if depth > analysis.max_depth {
            analysis.max_depth = depth;
        }

        for selection in selections {
            match selection {
                Selection::Field(field) => {
                    self.analyze_field(field, parent_type, depth, multiplier, analysis)?;
                }
                Selection::FragmentSpread { .. } => {
                    // Fragment spreads would be analyzed similarly
                }
                Selection::InlineFragment { selections, .. } => {
                    self.analyze_selections(selections, parent_type, depth, multiplier, analysis)?;
                }
            }
        }

        Ok(())
    }

    /// Analyze a field
    fn analyze_field(
        &self,
        field_sel: &FieldSelection,
        parent_type: &ObjectType,
        depth: usize,
        multiplier: f64,
        analysis: &mut ComplexityAnalysis,
    ) -> ComplexityResult<()> {
        let field = parent_type.get_field(&field_sel.name).ok_or_else(|| {
            ComplexityError::FieldNotFound(field_sel.name.clone(), parent_type.name.clone())
        })?;

        // Calculate field cost
        let field_cost = self.calculate_field_cost(parent_type, field, field_sel);
        let total_cost = field_cost * multiplier;

        // Update analysis
        analysis.total_complexity += total_cost;
        analysis.field_count += 1;

        *analysis
            .type_breakdown
            .entry(parent_type.name.clone())
            .or_insert(0.0) += total_cost;

        // Check if field is a list
        let (is_list, inner_multiplier) = self.check_list_type(&field.type_ref);
        if is_list {
            analysis.list_field_count += 1;
        }

        // Analyze nested selections
        if !field_sel.selections.is_empty() {
            if let Some(nested_type) = self.get_object_type(&field.type_ref) {
                let nested_multiplier = if is_list {
                    multiplier * inner_multiplier
                } else {
                    multiplier
                };

                self.analyze_selections(
                    &field_sel.selections,
                    nested_type,
                    depth + 1,
                    nested_multiplier,
                    analysis,
                )?;
            }
        }

        Ok(())
    }

    /// Calculate field cost
    fn calculate_field_cost(
        &self,
        parent_type: &ObjectType,
        field: &Field,
        field_sel: &FieldSelection,
    ) -> f64 {
        // Check if we have a custom cost calculator
        if let Some(calculator) = self.cost_registry.get_calculator(&parent_type.name, &field.name) {
            return calculator.calculate_cost(&field_sel.arguments);
        }

        // Use default cost
        self.config.default_field_cost
    }

    /// Check if type is a list and get multiplier
    fn check_list_type(&self, type_ref: &TypeRef) -> (bool, f64) {
        match type_ref {
            TypeRef::List(_) => (true, self.config.list_multiplier),
            TypeRef::NonNull(inner) => self.check_list_type(inner),
            TypeRef::Named(_) => (false, 1.0),
        }
    }

    /// Get object type from type reference
    fn get_object_type(&self, type_ref: &TypeRef) -> Option<&ObjectType> {
        let type_name = type_ref.base_type();
        self.schema.get_type(type_name).and_then(|t| match t {
            super::schema::TypeDefinition::Object(obj) => Some(obj),
            _ => None,
        })
    }

    /// Check if analysis exceeds limits
    fn check_limits(&self, analysis: &mut ComplexityAnalysis) -> ComplexityResult<()> {
        let mut has_violations = false;

        // Check depth limit
        if self.config.enable_depth_limit && self.config.max_depth > 0 {
            if analysis.max_depth > self.config.max_depth {
                let msg = format!(
                    "Query depth {} exceeds maximum {}",
                    analysis.max_depth, self.config.max_depth
                );
                analysis.add_violation(msg.clone());
                has_violations = true;

                if self.config.reject_on_limit {
                    return Err(ComplexityError::MaxDepthExceeded(
                        analysis.max_depth,
                        self.config.max_depth,
                    ));
                }
            }
        }

        // Check complexity limit
        if self.config.enable_complexity_limit && self.config.max_complexity > 0.0 {
            if analysis.total_complexity > self.config.max_complexity {
                let msg = format!(
                    "Query complexity {:.2} exceeds maximum {:.2}",
                    analysis.total_complexity, self.config.max_complexity
                );
                analysis.add_violation(msg.clone());
                has_violations = true;

                if self.config.reject_on_limit {
                    return Err(ComplexityError::MaxComplexityExceeded(
                        analysis.total_complexity,
                        self.config.max_complexity,
                    ));
                }
            }
        }

        if has_violations {
            analysis.exceeds_limits = true;
        }

        Ok(())
    }
}

// ============================================================================
// Query Cost Estimator
// ============================================================================

/// Estimates query execution cost
pub struct QueryCostEstimator {
    /// Analyzer
    analyzer: ComplexityAnalyzer,
}

impl QueryCostEstimator {
    /// Create a new cost estimator
    pub fn new(analyzer: ComplexityAnalyzer) -> Self {
        Self { analyzer }
    }

    /// Estimate query cost
    pub fn estimate(&self, document: &Document) -> ComplexityResult<QueryCostEstimate> {
        let analysis = self.analyzer.analyze(document)?;

        Ok(QueryCostEstimate {
            complexity: analysis.total_complexity,
            estimated_time_ms: self.estimate_execution_time(&analysis),
            estimated_memory_kb: self.estimate_memory_usage(&analysis),
            field_count: analysis.field_count,
            max_depth: analysis.max_depth,
        })
    }

    /// Estimate execution time in milliseconds
    fn estimate_execution_time(&self, analysis: &ComplexityAnalysis) -> f64 {
        // Simple heuristic: 1 complexity unit = 1ms
        // In production, this would be calibrated based on actual measurements
        analysis.total_complexity
    }

    /// Estimate memory usage in KB
    fn estimate_memory_usage(&self, analysis: &ComplexityAnalysis) -> f64 {
        // Simple heuristic: each field takes ~1KB
        analysis.field_count as f64
    }
}

/// Query cost estimate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryCostEstimate {
    /// Complexity score
    pub complexity: f64,
    /// Estimated execution time in milliseconds
    pub estimated_time_ms: f64,
    /// Estimated memory usage in KB
    pub estimated_memory_kb: f64,
    /// Field count
    pub field_count: usize,
    /// Maximum depth
    pub max_depth: usize,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enterprise::graphql::query::QueryBuilder;
    use crate::enterprise::graphql::schema::{FnResolver, Schema};

    #[test]
    fn test_complexity_config() {
        let config = ComplexityConfig::default();
        assert_eq!(config.max_depth, 10);
        assert!(config.enable_depth_limit);

        let permissive = ComplexityConfig::permissive();
        assert!(!permissive.enable_depth_limit);

        let strict = ComplexityConfig::strict();
        assert_eq!(strict.max_depth, 5);
    }

    #[test]
    fn test_static_cost() {
        let cost = StaticCost::new(5.0);
        assert_eq!(cost.calculate_cost(&HashMap::new()), 5.0);
    }

    #[test]
    fn test_dynamic_cost() {
        use super::super::schema::Value;

        let cost = DynamicCost::new(1.0).with_arg_multiplier("limit", 0.1);

        let mut args = HashMap::new();
        args.insert("limit".to_string(), Value::Int(100));

        assert_eq!(cost.calculate_cost(&args), 11.0); // 1.0 + (100 * 0.1)
    }

    #[test]
    fn test_field_cost_registry() {
        let mut registry = FieldCostRegistry::new(1.0);
        let calculator = Arc::new(StaticCost::new(10.0));

        registry.register_cost("Query", "expensiveField", calculator);

        assert!(registry.get_calculator("Query", "expensiveField").is_some());
        assert_eq!(registry.default_cost(), 1.0);
    }

    #[tokio::test]
    async fn test_complexity_analysis() {
        let mut schema = Schema::new();

        let resolver = Arc::new(FnResolver::new(|_ctx, _parent, _args| {
            Ok(super::super::schema::Value::String("test".to_string()))
        }));

        let query_type = super::super::schema::ObjectType::new("Query").field(
            super::super::schema::Field::new(
                "hello",
                TypeRef::Named("String".to_string()),
                resolver,
            ),
        );

        schema
            .add_type(super::super::schema::TypeDefinition::Object(query_type))
            .unwrap();
        schema.set_query_type("Query");

        let analyzer = ComplexityAnalyzer::new(Arc::new(schema));
        let _doc = QueryBuilder::query().field("hello").build();

        let analysis = analyzer.analyze(&doc).unwrap();
        assert!(analysis.total_complexity > 0.0);
        assert!(analysis.field_count > 0);
    }

    #[test]
    fn test_complexity_analysis_creation() {
        let mut analysis = ComplexityAnalysis::new();
        assert_eq!(analysis.total_complexity, 0.0);
        assert!(analysis.is_acceptable());

        analysis.add_violation("Test violation");
        assert!(!analysis.is_acceptable());
        assert_eq!(analysis.violations.len(), 1);
    }
}
