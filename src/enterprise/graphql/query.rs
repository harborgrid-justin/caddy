//! GraphQL Query Engine
//!
//! This module provides query parsing, validation, field resolution,
//! batched execution, and comprehensive error handling.

use super::schema::{
    Field, ObjectType, ResolverContext, Schema, SchemaError, TypeRef, Value,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

// ============================================================================
// Error Types
// ============================================================================

/// Query execution errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum QueryError {
    /// Syntax error in query
    #[error("Syntax error: {0}")]
    SyntaxError(String),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Field not found
    #[error("Field '{0}' not found on type '{1}'")]
    FieldNotFound(String, String),

    /// Missing required argument
    #[error("Missing required argument '{0}' on field '{1}'")]
    MissingArgument(String, String),

    /// Invalid argument type
    #[error("Invalid argument type for '{0}': expected {1}")]
    InvalidArgumentType(String, String),

    /// Execution error
    #[error("Execution error: {0}")]
    ExecutionError(String),

    /// Too many operations
    #[error("Query must contain exactly one operation")]
    TooManyOperations,

    /// No operation found
    #[error("No operation found in query")]
    NoOperation,
}

pub type QueryResult<T> = Result<T, QueryError>;

/// Error with location information and extensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLError {
    /// Error message
    pub message: String,
    /// Path to the field that caused the error
    pub path: Vec<String>,
    /// Location in the query document
    pub locations: Vec<Location>,
    /// Additional error data
    pub extensions: HashMap<String, Value>,
}

impl GraphQLError {
    /// Create a new GraphQL error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            path: Vec::new(),
            locations: Vec::new(),
            extensions: HashMap::new(),
        }
    }

    /// Add path segment
    pub fn with_path(mut self, path: Vec<String>) -> Self {
        self.path = path;
        self
    }

    /// Add location
    pub fn with_location(mut self, line: usize, column: usize) -> Self {
        self.locations.push(Location { line, column });
        self
    }

    /// Add extension data
    pub fn with_extension(mut self, key: impl Into<String>, value: Value) -> Self {
        self.extensions.insert(key.into(), value);
        self
    }
}

/// Location in query document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
}

// ============================================================================
// Query AST
// ============================================================================

/// Operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    /// Query operation
    Query,
    /// Mutation operation
    Mutation,
    /// Subscription operation
    Subscription,
}

/// Variable definition
#[derive(Debug, Clone)]
pub struct VariableDefinition {
    /// Variable name
    pub name: String,
    /// Variable type
    pub type_ref: TypeRef,
    /// Default value
    pub default_value: Option<Value>,
}

/// Selection in query
#[derive(Debug, Clone)]
pub enum Selection {
    /// Field selection
    Field(FieldSelection),
    /// Fragment spread
    FragmentSpread {
        /// Fragment name
        name: String,
    },
    /// Inline fragment
    InlineFragment {
        /// Type condition
        type_condition: Option<String>,
        /// Selections within fragment
        selections: Vec<Selection>,
    },
}

/// Field selection
#[derive(Debug, Clone)]
pub struct FieldSelection {
    /// Field alias
    pub alias: Option<String>,
    /// Field name
    pub name: String,
    /// Field arguments
    pub arguments: HashMap<String, Value>,
    /// Nested selections
    pub selections: Vec<Selection>,
}

impl FieldSelection {
    /// Get the response key (alias or name)
    pub fn response_key(&self) -> &str {
        self.alias.as_deref().unwrap_or(&self.name)
    }
}

/// Fragment definition
#[derive(Debug, Clone)]
pub struct FragmentDefinition {
    /// Fragment name
    pub name: String,
    /// Type condition
    pub type_condition: String,
    /// Fragment selections
    pub selections: Vec<Selection>,
}

/// GraphQL operation
#[derive(Debug, Clone)]
pub struct Operation {
    /// Operation type
    pub operation_type: OperationType,
    /// Operation name (optional)
    pub name: Option<String>,
    /// Variable definitions
    pub variables: Vec<VariableDefinition>,
    /// Selections
    pub selections: Vec<Selection>,
}

/// Parsed GraphQL document
#[derive(Debug, Clone)]
pub struct Document {
    /// Operations in the document
    pub operations: Vec<Operation>,
    /// Fragment definitions
    pub fragments: HashMap<String, FragmentDefinition>,
}

// ============================================================================
// Query Parser
// ============================================================================

/// Simple GraphQL query parser
pub struct QueryParser {
    /// Input query string
    query: String,
}

impl QueryParser {
    /// Create a new query parser
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
        }
    }

    /// Parse the query into a document
    pub fn parse(&self) -> QueryResult<Document> {
        // This is a simplified parser for demonstration
        // In production, use a proper GraphQL parser library

        let query = self.query.trim();

        // Simple heuristic parsing
        if query.is_empty() {
            return Err(QueryError::NoOperation);
        }

        // For this example, create a simple query operation
        let operation = Operation {
            operation_type: if query.starts_with("mutation") {
                OperationType::Mutation
            } else if query.starts_with("subscription") {
                OperationType::Subscription
            } else {
                OperationType::Query
            },
            name: None,
            variables: Vec::new(),
            selections: self.parse_selections(query)?,
        };

        Ok(Document {
            operations: vec![operation],
            fragments: HashMap::new(),
        })
    }

    /// Parse selections (simplified)
    fn parse_selections(&self, _query: &str) -> QueryResult<Vec<Selection>> {
        // Simplified: In production, use a proper parser
        Ok(Vec::new())
    }
}

// ============================================================================
// Query Validator
// ============================================================================

/// Query validator
pub struct QueryValidator<'a> {
    /// Schema to validate against
    schema: &'a Schema,
}

impl<'a> QueryValidator<'a> {
    /// Create a new validator
    pub fn new(schema: &'a Schema) -> Self {
        Self { schema }
    }

    /// Validate a document
    pub fn validate(&self, document: &Document) -> QueryResult<()> {
        // Validate operations
        for operation in &document.operations {
            self.validate_operation(operation)?;
        }

        // Validate fragments
        for fragment in document.fragments.values() {
            self.validate_fragment(fragment)?;
        }

        Ok(())
    }

    /// Validate an operation
    fn validate_operation(&self, operation: &Operation) -> QueryResult<()> {
        let root_type = match operation.operation_type {
            OperationType::Query => self.schema.query_type(),
            OperationType::Mutation => self.schema.mutation_type(),
            OperationType::Subscription => self.schema.subscription_type(),
        };

        if let Some(root_type) = root_type {
            self.validate_selections(&operation.selections, root_type)?;
        } else {
            return Err(QueryError::ValidationError(format!(
                "{:?} root type not defined in schema",
                operation.operation_type
            )));
        }

        Ok(())
    }

    /// Validate selections against a type
    fn validate_selections(
        &self,
        selections: &[Selection],
        parent_type: &ObjectType,
    ) -> QueryResult<()> {
        for selection in selections {
            match selection {
                Selection::Field(field) => {
                    self.validate_field_selection(field, parent_type)?;
                }
                Selection::FragmentSpread { .. } => {
                    // Validate fragment spread
                }
                Selection::InlineFragment { .. } => {
                    // Validate inline fragment
                }
            }
        }
        Ok(())
    }

    /// Validate a field selection
    fn validate_field_selection(
        &self,
        field_sel: &FieldSelection,
        parent_type: &ObjectType,
    ) -> QueryResult<()> {
        // Check if field exists
        let _field = parent_type
            .get_field(&field_sel.name)
            .ok_or_else(|| {
                QueryError::FieldNotFound(field_sel.name.clone(), parent_type.name.clone())
            })?;

        // Validate arguments
        // Validate nested selections
        Ok(())
    }

    /// Validate a fragment
    fn validate_fragment(&self, _fragment: &FragmentDefinition) -> QueryResult<()> {
        // Validate fragment type exists
        // Validate fragment selections
        Ok(())
    }
}

// ============================================================================
// Query Executor
// ============================================================================

/// Execution context
pub struct ExecutionContext {
    /// Schema
    pub schema: Arc<Schema>,
    /// Resolver context
    pub resolver_context: ResolverContext,
    /// Variables
    pub variables: HashMap<String, Value>,
    /// Fragments
    pub fragments: HashMap<String, FragmentDefinition>,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new(
        schema: Arc<Schema>,
        resolver_context: ResolverContext,
        variables: HashMap<String, Value>,
    ) -> Self {
        Self {
            schema,
            resolver_context,
            variables,
            fragments: HashMap::new(),
        }
    }
}

/// Query executor
pub struct QueryExecutor {
    /// Schema
    schema: Arc<Schema>,
}

impl QueryExecutor {
    /// Create a new query executor
    pub fn new(schema: Arc<Schema>) -> Self {
        Self { schema }
    }

    /// Execute a query
    pub async fn execute(
        &self,
        document: &Document,
        variables: HashMap<String, Value>,
        resolver_context: ResolverContext,
    ) -> ExecutionResult {
        // Ensure exactly one operation
        if document.operations.is_empty() {
            return ExecutionResult {
                data: None,
                errors: vec![GraphQLError::new("No operation in query")],
            };
        }

        if document.operations.len() > 1 {
            return ExecutionResult {
                data: None,
                errors: vec![GraphQLError::new("Multiple operations not supported")],
            };
        }

        let operation = &document.operations[0];

        // Create execution context
        let mut ctx = ExecutionContext::new(
            Arc::clone(&self.schema),
            resolver_context,
            variables,
        );
        ctx.fragments = document.fragments.clone();

        // Execute operation
        match self.execute_operation(&ctx, operation).await {
            Ok(data) => ExecutionResult {
                data: Some(data),
                errors: Vec::new(),
            },
            Err(e) => ExecutionResult {
                data: None,
                errors: vec![GraphQLError::new(e.to_string())],
            },
        }
    }

    /// Execute an operation
    async fn execute_operation(
        &self,
        ctx: &ExecutionContext,
        operation: &Operation,
    ) -> Result<Value, SchemaError> {
        let root_type = match operation.operation_type {
            OperationType::Query => self.schema.query_type(),
            OperationType::Mutation => self.schema.mutation_type(),
            OperationType::Subscription => self.schema.subscription_type(),
        };

        if let Some(root_type) = root_type {
            self.execute_selections(ctx, &operation.selections, root_type, &Value::Null)
                .await
        } else {
            Err(SchemaError::InvalidType(format!(
                "{:?} root type not defined",
                operation.operation_type
            )))
        }
    }

    /// Execute selections
    fn execute_selections<'a>(
        &'a self,
        ctx: &'a ExecutionContext,
        selections: &'a [Selection],
        parent_type: &'a ObjectType,
        parent_value: &'a Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value, SchemaError>> + Send + 'a>> {
        Box::pin(async move {
            let mut result = HashMap::new();

            for selection in selections {
                match selection {
                    Selection::Field(field) => {
                        let value = self
                            .execute_field(ctx, field, parent_type, parent_value)
                            .await?;
                        result.insert(field.response_key().to_string(), value);
                    }
                    Selection::FragmentSpread { name } => {
                        if let Some(fragment) = ctx.fragments.get(name) {
                            let fragment_value = self
                                .execute_selections(
                                    ctx,
                                    &fragment.selections,
                                    parent_type,
                                    parent_value,
                                )
                                .await?;
                            if let Value::Object(obj) = fragment_value {
                                result.extend(obj);
                            }
                        }
                    }
                    Selection::InlineFragment {
                        type_condition: _,
                        selections: inline_selections,
                    } => {
                        let fragment_value = self
                            .execute_selections(ctx, inline_selections, parent_type, parent_value)
                            .await?;
                        if let Value::Object(obj) = fragment_value {
                            result.extend(obj);
                        }
                    }
                }
            }

            Ok(Value::Object(result))
        })
    }

    /// Execute a field
    async fn execute_field(
        &self,
        ctx: &ExecutionContext,
        field_sel: &FieldSelection,
        parent_type: &ObjectType,
        parent_value: &Value,
    ) -> Result<Value, SchemaError> {
        let field = parent_type
            .get_field(&field_sel.name)
            .ok_or_else(|| {
                SchemaError::FieldNotFound(field_sel.name.clone(), parent_type.name.clone())
            })?;

        // Resolve field value
        let value = field
            .resolver
            .resolve(&ctx.resolver_context, parent_value, &field_sel.arguments)
            .await?;

        // Execute nested selections if any
        if !field_sel.selections.is_empty() {
            if let Some(nested_type) = self.get_object_type(&field.type_ref) {
                return self
                    .execute_selections(ctx, &field_sel.selections, nested_type, &value)
                    .await;
            }
        }

        Ok(value)
    }

    /// Get object type from type reference
    fn get_object_type(&self, type_ref: &TypeRef) -> Option<&ObjectType> {
        let type_name = type_ref.base_type();
        self.schema.get_type(type_name).and_then(|t| match t {
            super::schema::TypeDefinition::Object(obj) => Some(obj),
            _ => None,
        })
    }
}

// ============================================================================
// Batch Execution
// ============================================================================

/// Batch executor for executing multiple queries efficiently
pub struct BatchExecutor {
    /// Executor
    executor: Arc<QueryExecutor>,
}

impl BatchExecutor {
    /// Create a new batch executor
    pub fn new(executor: Arc<QueryExecutor>) -> Self {
        Self { executor }
    }

    /// Execute multiple queries in batch
    pub async fn execute_batch(
        &self,
        queries: Vec<(Document, HashMap<String, Value>, ResolverContext)>,
    ) -> Vec<ExecutionResult> {
        let mut results = Vec::with_capacity(queries.len());

        for (document, variables, context) in queries {
            let result = self.executor.execute(&document, variables, context).await;
            results.push(result);
        }

        results
    }
}

// ============================================================================
// Execution Result
// ============================================================================

/// Query execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Result data (null if errors)
    pub data: Option<Value>,
    /// Errors that occurred during execution
    pub errors: Vec<GraphQLError>,
}

impl ExecutionResult {
    /// Check if execution was successful
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }

    /// Check if execution had errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

// ============================================================================
// Query Builder (Convenience API)
// ============================================================================

/// Query builder for programmatic query construction
pub struct QueryBuilder {
    /// Operation type
    operation_type: OperationType,
    /// Field selections
    selections: Vec<Selection>,
}

impl QueryBuilder {
    /// Create a new query builder
    pub fn query() -> Self {
        Self {
            operation_type: OperationType::Query,
            selections: Vec::new(),
        }
    }

    /// Create a mutation builder
    pub fn mutation() -> Self {
        Self {
            operation_type: OperationType::Mutation,
            selections: Vec::new(),
        }
    }

    /// Add a field selection
    pub fn field(mut self, name: impl Into<String>) -> Self {
        self.selections.push(Selection::Field(FieldSelection {
            alias: None,
            name: name.into(),
            arguments: HashMap::new(),
            selections: Vec::new(),
        }));
        self
    }

    /// Build the document
    pub fn build(self) -> Document {
        Document {
            operations: vec![Operation {
                operation_type: self.operation_type,
                name: None,
                variables: Vec::new(),
                selections: self.selections,
            }],
            fragments: HashMap::new(),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enterprise::graphql::schema::{FnResolver, Schema};

    #[test]
    fn test_query_parser() {
        let parser = QueryParser::new("query { hello }");
        let doc = parser.parse().unwrap();
        assert_eq!(doc.operations.len(), 1);
    }

    #[test]
    fn test_graphql_error() {
        let err = GraphQLError::new("Test error")
            .with_path(vec!["user".to_string(), "name".to_string()])
            .with_location(1, 5);

        assert_eq!(err.message, "Test error");
        assert_eq!(err.path.len(), 2);
        assert_eq!(err.locations.len(), 1);
    }

    #[test]
    fn test_query_builder() {
        let doc = QueryBuilder::query()
            .field("hello")
            .field("world")
            .build();

        assert_eq!(doc.operations.len(), 1);
        assert_eq!(doc.operations[0].selections.len(), 2);
    }

    #[tokio::test]
    async fn test_query_execution() {
        use super::super::schema::ObjectType;

        let mut schema = Schema::new();

        let resolver = Arc::new(FnResolver::new(|_ctx, _parent, _args| {
            Ok(Value::String("Hello, World!".to_string()))
        }));

        let query_type = ObjectType::new("Query").field(
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

        let executor = QueryExecutor::new(Arc::new(schema));
        let doc = QueryBuilder::query().field("hello").build();
        let ctx = ResolverContext::new("test-req");

        let result = executor.execute(&doc, HashMap::new(), ctx).await;
        assert!(result.is_success());
    }
}
