//! GraphQL Federation Support
//!
//! Provides entity resolution, reference resolvers, schema stitching,
//! and gateway support for distributed GraphQL architectures.

use super::schema::{
    Directive, Field, FieldResolver, ObjectType, ResolverContext, Schema, TypeDefinition, TypeRef,
    Value,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

// ============================================================================
// Error Types
// ============================================================================

/// Federation errors
#[derive(Error, Debug, Clone)]
pub enum FederationError {
    /// Entity not found
    #[error("Entity not found: {0}")]
    EntityNotFound(String),

    /// Invalid entity representation
    #[error("Invalid entity representation: {0}")]
    InvalidRepresentation(String),

    /// Service not available
    #[error("Service '{0}' not available")]
    ServiceNotAvailable(String),

    /// Schema stitching error
    #[error("Schema stitching error: {0}")]
    StitchingError(String),

    /// Invalid federation directive
    #[error("Invalid federation directive: {0}")]
    InvalidDirective(String),

    /// Circular dependency
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),
}

pub type FederationResult<T> = Result<T, FederationError>;

// ============================================================================
// Federation Directives
// ============================================================================

/// Federation directive types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FederationDirective {
    /// Marks a type as an entity
    Key,
    /// Extends a type from another service
    Extends,
    /// Marks a field as external
    External,
    /// Requires fields from another service
    Requires,
    /// Provides fields to another service
    Provides,
}

impl FederationDirective {
    /// Get directive name
    pub fn name(&self) -> &str {
        match self {
            Self::Key => "key",
            Self::Extends => "extends",
            Self::External => "external",
            Self::Requires => "requires",
            Self::Provides => "provides",
        }
    }
}

// ============================================================================
// Entity Definition
// ============================================================================

/// Entity key fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityKey {
    /// Key fields (e.g., ["id"] or ["userId", "productId"])
    pub fields: Vec<String>,
}

impl EntityKey {
    /// Create a new entity key
    pub fn new(fields: Vec<String>) -> Self {
        Self { fields }
    }

    /// Create a single-field key
    pub fn single(field: impl Into<String>) -> Self {
        Self {
            fields: vec![field.into()],
        }
    }

    /// Create a composite key
    pub fn composite(fields: Vec<String>) -> Self {
        Self { fields }
    }
}

/// Entity representation (used for resolving entities)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRepresentation {
    /// Type name
    #[serde(rename = "__typename")]
    pub typename: String,
    /// Key fields and values
    #[serde(flatten)]
    pub fields: HashMap<String, Value>,
}

impl EntityRepresentation {
    /// Create a new entity representation
    pub fn new(typename: impl Into<String>) -> Self {
        Self {
            typename: typename.into(),
            fields: HashMap::new(),
        }
    }

    /// Add a field
    pub fn with_field(mut self, key: impl Into<String>, value: Value) -> Self {
        self.fields.insert(key.into(), value);
        self
    }

    /// Get field value
    pub fn get_field(&self, key: &str) -> Option<&Value> {
        self.fields.get(key)
    }
}

// ============================================================================
// Entity Resolver
// ============================================================================

/// Trait for resolving federated entities
#[async_trait]
pub trait EntityResolver: Send + Sync {
    /// Resolve entities by their representations
    async fn resolve_entities(
        &self,
        ctx: &ResolverContext,
        representations: Vec<EntityRepresentation>,
    ) -> FederationResult<Vec<Value>>;

    /// Get supported entity types
    fn supported_types(&self) -> Vec<String>;
}

/// Entity resolver registry
pub struct EntityResolverRegistry {
    /// Resolvers by type name
    resolvers: HashMap<String, Arc<dyn EntityResolver>>,
}

impl EntityResolverRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            resolvers: HashMap::new(),
        }
    }

    /// Register an entity resolver
    pub fn register(&mut self, typename: impl Into<String>, resolver: Arc<dyn EntityResolver>) {
        self.resolvers.insert(typename.into(), resolver);
    }

    /// Get resolver for a type
    pub fn get_resolver(&self, typename: &str) -> Option<&Arc<dyn EntityResolver>> {
        self.resolvers.get(typename)
    }

    /// Resolve multiple entities
    pub async fn resolve_entities(
        &self,
        ctx: &ResolverContext,
        representations: Vec<EntityRepresentation>,
    ) -> FederationResult<Vec<Value>> {
        let mut results = Vec::new();

        // Group representations by type
        let mut by_type: HashMap<String, Vec<EntityRepresentation>> = HashMap::new();
        for repr in representations {
            by_type.entry(repr.typename.clone()).or_default().push(repr);
        }

        // Resolve each type
        for (typename, reprs) in by_type {
            let resolver = self.get_resolver(&typename).ok_or_else(|| {
                FederationError::EntityNotFound(format!("No resolver for type '{}'", typename))
            })?;

            let entities = resolver.resolve_entities(ctx, reprs).await?;
            results.extend(entities);
        }

        Ok(results)
    }

    /// Get all supported types
    pub fn supported_types(&self) -> Vec<String> {
        self.resolvers.keys().cloned().collect()
    }
}

impl Default for EntityResolverRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Reference Resolver
// ============================================================================

/// Trait for resolving references to entities in other services
#[async_trait]
pub trait ReferenceResolver: Send + Sync {
    /// Resolve a reference
    async fn resolve_reference(
        &self,
        ctx: &ResolverContext,
        representation: &EntityRepresentation,
    ) -> FederationResult<Value>;
}

/// Simple reference resolver using a closure
pub struct SimpleReferenceResolver<F>
where
    F: Fn(&ResolverContext, &EntityRepresentation) -> FederationResult<Value> + Send + Sync,
{
    func: F,
}

impl<F> SimpleReferenceResolver<F>
where
    F: Fn(&ResolverContext, &EntityRepresentation) -> FederationResult<Value> + Send + Sync,
{
    /// Create a new simple reference resolver
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

#[async_trait]
impl<F> ReferenceResolver for SimpleReferenceResolver<F>
where
    F: Fn(&ResolverContext, &EntityRepresentation) -> FederationResult<Value> + Send + Sync,
{
    async fn resolve_reference(
        &self,
        ctx: &ResolverContext,
        representation: &EntityRepresentation,
    ) -> FederationResult<Value> {
        (self.func)(ctx, representation)
    }
}

// ============================================================================
// Service Definition
// ============================================================================

/// Federated service definition
#[derive(Debug, Clone)]
pub struct ServiceDefinition {
    /// Service name
    pub name: String,
    /// Service URL
    pub url: String,
    /// Service SDL (Schema Definition Language)
    pub sdl: String,
}

impl ServiceDefinition {
    /// Create a new service definition
    pub fn new(name: impl Into<String>, url: impl Into<String>, sdl: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            url: url.into(),
            sdl: sdl.into(),
        }
    }
}

// ============================================================================
// Schema Stitching
// ============================================================================

/// Schema stitcher for combining multiple federated schemas
pub struct SchemaStitcher {
    /// Services to stitch
    services: Vec<ServiceDefinition>,
    /// Entity resolver registry
    entity_registry: Arc<EntityResolverRegistry>,
}

impl SchemaStitcher {
    /// Create a new schema stitcher
    pub fn new(entity_registry: Arc<EntityResolverRegistry>) -> Self {
        Self {
            services: Vec::new(),
            entity_registry,
        }
    }

    /// Add a service
    pub fn add_service(&mut self, service: ServiceDefinition) {
        self.services.push(service);
    }

    /// Stitch schemas together
    pub fn stitch(&self) -> FederationResult<Schema> {
        let mut schema = Schema::new();

        // Add federation types
        self.add_federation_types(&mut schema)?;

        // Process each service
        for service in &self.services {
            self.process_service(&mut schema, service)?;
        }

        Ok(schema)
    }

    /// Add federation-specific types
    fn add_federation_types(&self, schema: &mut Schema) -> FederationResult<()> {
        // Add _Any scalar for entity representations
        schema
            .add_type(TypeDefinition::Scalar(super::schema::ScalarType {
                name: "_Any".to_string(),
                description: Some("Represents any entity type".to_string()),
            }))
            .map_err(|e| FederationError::StitchingError(e.to_string()))?;

        // Add _Service type
        let service_resolver = Arc::new(super::schema::FnResolver::new(
            |_ctx, _parent, _args| Ok(Value::Null),
        ));

        let service_type = ObjectType::new("_Service")
            .field(
                Field::new(
                    "sdl",
                    TypeRef::Named("String".to_string()),
                    service_resolver.clone(),
                )
                .description("Service SDL"),
            );

        schema
            .add_type(TypeDefinition::Object(service_type))
            .map_err(|e| FederationError::StitchingError(e.to_string()))?;

        Ok(())
    }

    /// Process a service schema
    fn process_service(
        &self,
        _schema: &mut Schema,
        _service: &ServiceDefinition,
    ) -> FederationResult<()> {
        // In a real implementation, this would:
        // 1. Parse the service SDL
        // 2. Extract types and their federation directives
        // 3. Merge types into the main schema
        // 4. Handle type extensions and conflicts
        Ok(())
    }
}

// ============================================================================
// Federation Gateway
// ============================================================================

/// Federation gateway configuration
#[derive(Debug, Clone)]
pub struct GatewayConfig {
    /// Services
    pub services: Vec<ServiceDefinition>,
    /// Enable query planning
    pub enable_query_planning: bool,
    /// Enable caching
    pub enable_caching: bool,
    /// Maximum query depth
    pub max_query_depth: usize,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            services: Vec::new(),
            enable_query_planning: true,
            enable_caching: true,
            max_query_depth: 10,
        }
    }
}

/// Query plan step
#[derive(Debug, Clone)]
pub enum QueryPlanStep {
    /// Fetch data from a service
    Fetch {
        /// Service name
        service: String,
        /// Query to execute
        query: String,
        /// Variables
        variables: HashMap<String, Value>,
    },
    /// Parallel execution
    Parallel {
        /// Steps to execute in parallel
        steps: Vec<QueryPlanStep>,
    },
    /// Sequential execution
    Sequence {
        /// Steps to execute sequentially
        steps: Vec<QueryPlanStep>,
    },
    /// Flatten nested results
    Flatten {
        /// Path to flatten
        path: Vec<String>,
    },
}

/// Query planner for federation
pub struct QueryPlanner {
    /// Services
    services: HashMap<String, ServiceDefinition>,
}

impl QueryPlanner {
    /// Create a new query planner
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    /// Add a service
    pub fn add_service(&mut self, service: ServiceDefinition) {
        self.services.insert(service.name.clone(), service);
    }

    /// Plan a query
    pub fn plan(&self, _query: &str) -> FederationResult<QueryPlanStep> {
        // Simplified planning: in production, this would analyze the query
        // and create an optimal execution plan
        Ok(QueryPlanStep::Sequence { steps: Vec::new() })
    }
}

impl Default for QueryPlanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Federation gateway
pub struct FederationGateway {
    /// Configuration
    config: GatewayConfig,
    /// Stitched schema
    schema: Arc<Schema>,
    /// Query planner
    planner: QueryPlanner,
    /// Entity resolver registry
    entity_registry: Arc<EntityResolverRegistry>,
}

impl FederationGateway {
    /// Create a new federation gateway
    pub fn new(config: GatewayConfig, entity_registry: Arc<EntityResolverRegistry>) -> FederationResult<Self> {
        let stitcher = SchemaStitcher::new(Arc::clone(&entity_registry));
        let schema = stitcher.stitch()?;

        let mut planner = QueryPlanner::new();
        for service in &config.services {
            planner.add_service(service.clone());
        }

        Ok(Self {
            config,
            schema: Arc::new(schema),
            planner,
            entity_registry,
        })
    }

    /// Get the gateway schema
    pub fn schema(&self) -> &Arc<Schema> {
        &self.schema
    }

    /// Execute a query across federated services
    pub async fn execute(
        &self,
        _query: &str,
        _variables: HashMap<String, Value>,
        _ctx: ResolverContext,
    ) -> FederationResult<Value> {
        // In production, this would:
        // 1. Plan the query
        // 2. Execute plan steps
        // 3. Merge results
        // 4. Return combined response
        Ok(Value::Null)
    }

    /// Resolve entities
    pub async fn resolve_entities(
        &self,
        ctx: &ResolverContext,
        representations: Vec<EntityRepresentation>,
    ) -> FederationResult<Vec<Value>> {
        self.entity_registry.resolve_entities(ctx, representations).await
    }
}

// ============================================================================
// Federated Query Executor
// ============================================================================

/// Executor for federated queries
pub struct FederatedQueryExecutor {
    /// Gateway
    gateway: Arc<FederationGateway>,
}

impl FederatedQueryExecutor {
    /// Create a new federated query executor
    pub fn new(gateway: Arc<FederationGateway>) -> Self {
        Self { gateway }
    }

    /// Execute a federated query
    pub async fn execute(
        &self,
        query: &str,
        variables: HashMap<String, Value>,
        ctx: ResolverContext,
    ) -> FederationResult<Value> {
        self.gateway.execute(query, variables, ctx).await
    }

    /// Execute a query plan
    async fn execute_plan(&self, _plan: &QueryPlanStep) -> FederationResult<Value> {
        // Execute the query plan
        Ok(Value::Null)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    struct TestEntityResolver;

    #[async_trait]
    impl EntityResolver for TestEntityResolver {
        async fn resolve_entities(
            &self,
            _ctx: &ResolverContext,
            representations: Vec<EntityRepresentation>,
        ) -> FederationResult<Vec<Value>> {
            let mut results = Vec::new();
            for repr in representations {
                let mut obj = HashMap::new();
                obj.insert("__typename".to_string(), Value::String(repr.typename.clone()));
                for (k, v) in repr.fields {
                    obj.insert(k, v);
                }
                results.push(Value::Object(obj));
            }
            Ok(results)
        }

        fn supported_types(&self) -> Vec<String> {
            vec!["User".to_string()]
        }
    }

    #[test]
    fn test_entity_key() {
        let key = EntityKey::single("id");
        assert_eq!(key.fields.len(), 1);
        assert_eq!(key.fields[0], "id");

        let composite = EntityKey::composite(vec!["userId".to_string(), "productId".to_string()]);
        assert_eq!(composite.fields.len(), 2);
    }

    #[test]
    fn test_entity_representation() {
        let repr = EntityRepresentation::new("User")
            .with_field("id", Value::String("123".to_string()));

        assert_eq!(repr.typename, "User");
        assert!(repr.get_field("id").is_some());
    }

    #[test]
    fn test_entity_resolver_registry() {
        let mut registry = EntityResolverRegistry::new();
        let resolver = Arc::new(TestEntityResolver);

        registry.register("User", resolver);
        assert!(registry.get_resolver("User").is_some());
        assert!(registry.supported_types().contains(&"User".to_string()));
    }

    #[tokio::test]
    async fn test_entity_resolution() {
        let mut registry = EntityResolverRegistry::new();
        registry.register("User", Arc::new(TestEntityResolver));

        let repr = EntityRepresentation::new("User")
            .with_field("id", Value::String("123".to_string()));

        let ctx = ResolverContext::new("test-req");
        let results = registry.resolve_entities(&ctx, vec![repr]).await.unwrap();

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_service_definition() {
        let service = ServiceDefinition::new(
            "users",
            "http://localhost:4001",
            "type User { id: ID! }",
        );

        assert_eq!(service.name, "users");
        assert_eq!(service.url, "http://localhost:4001");
    }

    #[test]
    fn test_query_planner() {
        let mut planner = QueryPlanner::new();
        let service = ServiceDefinition::new(
            "users",
            "http://localhost:4001",
            "type User { id: ID! }",
        );

        planner.add_service(service);
        let plan = planner.plan("{ user(id: 1) { id } }").unwrap();

        match plan {
            QueryPlanStep::Sequence { .. } => (),
            _ => panic!("Expected sequence plan"),
        }
    }

    #[test]
    fn test_gateway_config() {
        let config = GatewayConfig::default();
        assert!(config.enable_query_planning);
        assert!(config.enable_caching);
        assert_eq!(config.max_query_depth, 10);
    }

    #[test]
    fn test_federation_directive() {
        let key = FederationDirective::Key;
        assert_eq!(key.name(), "key");

        let extends = FederationDirective::Extends;
        assert_eq!(extends.name(), "extends");
    }
}
