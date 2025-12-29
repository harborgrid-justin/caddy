//! # CADDY GraphQL Enterprise API - v0.2.0
//!
//! A complete, production-ready GraphQL infrastructure for enterprise CAD applications.
//!
//! ## Overview
//!
//! This module provides a comprehensive GraphQL implementation with enterprise-grade features:
//!
//! - **Schema Definition**: Full GraphQL type system with objects, interfaces, unions, enums, and input types
//! - **Query Execution**: Efficient query parsing, validation, and execution with batching support
//! - **DataLoader**: N+1 query prevention through intelligent batching and caching
//! - **Subscriptions**: Real-time updates via WebSocket with connection management
//! - **Complexity Analysis**: Query cost calculation and depth limiting for DoS prevention
//! - **Federation**: Distributed schema support with entity resolution and query planning
//! - **Persisted Queries**: APQ (Automatic Persisted Queries) for performance and security
//!
//! ## Quick Start
//!
//! ### Creating a Schema
//!
//! ```rust
//! use caddy::enterprise::graphql::{
//!     schema::{Schema, ObjectType, Field, TypeRef, FnResolver, Value},
//!     query::{QueryExecutor, ResolverContext},
//! };
//! use std::sync::Arc;
//! use std::collections::HashMap;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a schema
//! let mut schema = Schema::new();
//!
//! // Define a resolver
//! let resolver = Arc::new(FnResolver::new(|_ctx, _parent, _args| {
//!     Ok(Value::String("Hello, GraphQL!".to_string()))
//! }));
//!
//! // Create a query type
//! let query_type = ObjectType::new("Query")
//!     .field(Field::new(
//!         "hello",
//!         TypeRef::Named("String".to_string()),
//!         resolver,
//!     ).description("A simple hello query"));
//!
//! // Add to schema
//! schema.add_type(schema::TypeDefinition::Object(query_type))?;
//! schema.set_query_type("Query");
//!
//! // Execute queries
//! let executor = QueryExecutor::new(Arc::new(schema));
//! # Ok(())
//! # }
//! ```
//!
//! ### Using DataLoader
//!
//! ```rust
//! use caddy::enterprise::graphql::dataloader::{DataLoader, BatchLoadFn};
//! use async_trait::async_trait;
//! use std::collections::HashMap;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Define a batch loader
//! struct UserLoader;
//!
//! #[async_trait]
//! impl BatchLoadFn<i32, String> for UserLoader {
//!     async fn load(&self, keys: &[i32]) -> Result<HashMap<i32, String>, dataloader::DataLoaderError> {
//!         // Fetch users from database in a single query
//!         let mut users = HashMap::new();
//!         for id in keys {
//!             users.insert(*id, format!("User {}", id));
//!         }
//!         Ok(users)
//!     }
//! }
//!
//! // Create DataLoader
//! let loader = DataLoader::new(Arc::new(UserLoader));
//!
//! // Load users (will be batched automatically)
//! let user1 = loader.load(1).await?;
//! let user2 = loader.load(2).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Setting Up Subscriptions
//!
//! ```rust
//! use caddy::enterprise::graphql::subscription::{
//!     SubscriptionManager, EventBus, SubscriptionEvent,
//! };
//! use caddy::enterprise::graphql::schema::Value;
//!
//! # async fn example() {
//! // Create subscription manager
//! let manager = SubscriptionManager::new();
//!
//! // Publish events
//! let event = SubscriptionEvent::new(Value::String("New message".to_string()));
//! manager.event_bus().publish("messages", event).await;
//!
//! // Subscribe to events
//! let mut rx = manager.event_bus().subscribe("messages").await;
//! # }
//! ```
//!
//! ### Query Complexity Analysis
//!
//! ```rust
//! use caddy::enterprise::graphql::complexity::{
//!     ComplexityAnalyzer, ComplexityConfig,
//! };
//! use caddy::enterprise::graphql::query::QueryBuilder;
//! use std::sync::Arc;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let schema = Arc::new(caddy::enterprise::graphql::schema::Schema::new());
//! // Create analyzer with custom config
//! let config = ComplexityConfig {
//!     max_depth: 5,
//!     max_complexity: 100.0,
//!     ..Default::default()
//! };
//!
//! let analyzer = ComplexityAnalyzer::with_config(schema, config);
//!
//! // Analyze a query
//! let document = QueryBuilder::query().field("users").build();
//! let analysis = analyzer.analyze(&document)?;
//!
//! if !analysis.is_acceptable() {
//!     println!("Query rejected: {:?}", analysis.violations);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Federation Setup
//!
//! ```rust
//! use caddy::enterprise::graphql::federation::{
//!     FederationGateway, GatewayConfig, ServiceDefinition, EntityResolverRegistry,
//! };
//! use std::sync::Arc;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Define services
//! let user_service = ServiceDefinition::new(
//!     "users",
//!     "http://localhost:4001/graphql",
//!     "type User @key(fields: \"id\") { id: ID! name: String }",
//! );
//!
//! let post_service = ServiceDefinition::new(
//!     "posts",
//!     "http://localhost:4002/graphql",
//!     "type Post { id: ID! title: String author: User }",
//! );
//!
//! // Create gateway
//! let config = GatewayConfig {
//!     services: vec![user_service, post_service],
//!     ..Default::default()
//! };
//!
//! let registry = Arc::new(EntityResolverRegistry::new());
//! let gateway = FederationGateway::new(config, registry)?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Persisted Queries
//!
//! ```rust
//! use caddy::enterprise::graphql::persisted::{
//!     PersistedQueryManager, PersistedQueryConfig, QueryHash,
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create manager
//! let config = PersistedQueryConfig::production();
//! let manager = PersistedQueryManager::with_memory_storage(config);
//!
//! // Register a query
//! let query = "{ user(id: 1) { name email } }";
//! let hash = manager.register(query, Some("GetUser".to_string()), None).await?;
//!
//! // Retrieve by hash
//! let retrieved = manager.get(&hash).await?;
//! assert_eq!(retrieved, Some(query.to_string()));
//! # Ok(())
//! # }
//! ```
//!
//! ## Architecture
//!
//! ### Schema Layer
//!
//! The schema layer (`schema` module) provides the core GraphQL type system:
//!
//! - **Type Definitions**: Objects, Interfaces, Unions, Enums, Scalars, Input Objects
//! - **Field Resolvers**: Async trait-based resolver system with context propagation
//! - **Directives**: Built-in and custom directive support
//! - **Introspection**: Full schema introspection support
//!
//! ### Execution Layer
//!
//! The execution layer (`query` module) handles query processing:
//!
//! - **Parser**: Converts GraphQL query strings to AST
//! - **Validator**: Validates queries against the schema
//! - **Executor**: Resolves fields and executes queries
//! - **Batch Executor**: Efficiently executes multiple queries
//!
//! ### Performance Layer
//!
//! Performance optimizations:
//!
//! - **DataLoader** (`dataloader`): Batches and caches data fetching
//! - **Complexity Analysis** (`complexity`): Prevents expensive queries
//! - **Persisted Queries** (`persisted`): Reduces bandwidth and improves security
//!
//! ### Real-time Layer
//!
//! Real-time capabilities:
//!
//! - **Subscriptions** (`subscription`): WebSocket-based event streaming
//! - **Event Bus**: Topic-based pub/sub for efficient event distribution
//! - **Connection Management**: Handles WebSocket lifecycle and cleanup
//!
//! ### Distribution Layer
//!
//! Federation support:
//!
//! - **Entity Resolution** (`federation`): Resolves entities across services
//! - **Schema Stitching**: Combines schemas from multiple services
//! - **Query Planning**: Optimizes query execution across services
//! - **Gateway**: Single entry point for federated services
//!
//! ## Performance Considerations
//!
//! ### Batching
//!
//! Use DataLoaders to batch database queries and prevent N+1 problems:
//!
//! ```text
//! Without DataLoader:        With DataLoader:
//! Query users (1 query)      Query users (1 query)
//!   → Get posts (N queries)    → Get posts (1 query)
//! Total: N+1 queries         Total: 2 queries
//! ```
//!
//! ### Caching
//!
//! - Field-level caching within request context
//! - Persisted queries for repeated query strings
//! - Connection-level caching for subscriptions
//!
//! ### Complexity Limits
//!
//! Prevent DoS attacks with query complexity analysis:
//!
//! - Maximum query depth
//! - Maximum query complexity score
//! - Field-specific cost annotations
//! - Automatic query rejection
//!
//! ## Security
//!
//! ### Query Validation
//!
//! - Schema validation before execution
//! - Type checking for all fields and arguments
//! - Depth and complexity limiting
//!
//! ### Persisted Queries
//!
//! - Only allow pre-approved queries in production
//! - Prevent query injection attacks
//! - Reduce bandwidth usage
//!
//! ### Authentication & Authorization
//!
//! - Context-based authentication
//! - Field-level authorization via resolvers
//! - Integration with enterprise auth module
//!
//! ## Error Handling
//!
//! All operations return detailed error types:
//!
//! - `SchemaError`: Schema definition and validation errors
//! - `QueryError`: Query parsing and execution errors
//! - `DataLoaderError`: Data loading errors
//! - `SubscriptionError`: Subscription and WebSocket errors
//! - `ComplexityError`: Query complexity violations
//! - `FederationError`: Federation and stitching errors
//! - `PersistedQueryError`: Persisted query errors
//!
//! ## Testing
//!
//! All modules include comprehensive unit tests. Run with:
//!
//! ```bash
//! cargo test --package caddy --lib enterprise::graphql
//! ```
//!
//! ## Future Enhancements
//!
//! Planned features for future releases:
//!
//! - [ ] GraphQL Schema Definition Language (SDL) parser
//! - [ ] Code generation from schema
//! - [ ] Query result caching with Redis
//! - [ ] Distributed tracing integration
//! - [ ] Metrics and monitoring
//! - [ ] Rate limiting per user/IP
//! - [ ] File upload support (multipart)
//! - [ ] Defer and Stream directives
//!
//! ## Version History
//!
//! - **v0.2.0** (2025-12-28): Initial GraphQL implementation with complete feature set
//!
//! ## License
//!
//! Part of CADDY Enterprise Edition. See LICENSE-ENTERPRISE.txt for details.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![allow(clippy::module_name_repetitions)]

// ============================================================================
// Module Declarations
// ============================================================================

/// GraphQL schema definition and type system
///
/// Provides the core GraphQL type system including objects, interfaces, unions,
/// enums, scalars, and input types, along with field resolvers and directives.
pub mod schema;

/// GraphQL query engine
///
/// Query parsing, validation, field resolution, and execution with support
/// for batched operations and comprehensive error handling.
pub mod query;

/// DataLoader for batching and caching
///
/// Prevents N+1 query problems through intelligent batching and per-request
/// caching of data fetching operations.
pub mod dataloader;

/// GraphQL subscriptions
///
/// Real-time subscriptions over WebSocket with connection management,
/// event filtering, and pub/sub support.
pub mod subscription;

/// Query complexity analysis
///
/// Analyzes and limits query complexity to prevent resource exhaustion
/// attacks and ensure fair usage.
pub mod complexity;

/// GraphQL Federation
///
/// Support for distributed GraphQL architectures with entity resolution,
/// schema stitching, and gateway functionality.
pub mod federation;

/// Persisted queries
///
/// Query registration, hash-based lookup, and Automatic Persisted Queries
/// (APQ) for improved performance and security.
pub mod persisted;

// ============================================================================
// Re-exports for Convenience
// ============================================================================

// Schema types
pub use schema::{
    Directive, DirectiveDefinition, DirectiveLocation, EnumType, EnumValue, Field, FieldResolver,
    FnResolver, InputObjectType, InputValue, InterfaceType, ObjectType, ResolverContext,
    ScalarType, Schema, SchemaError, SchemaResult, TypeDefinition, TypeKind, TypeRef, UnionType,
    Value,
};

// Query types
pub use query::{
    Document, ExecutionResult, FieldSelection, FragmentDefinition, GraphQLError, Location,
    Operation, OperationType, QueryBuilder, QueryError, QueryExecutor, QueryParser,
    QueryResult, QueryValidator, Selection, VariableDefinition,
};

// DataLoader types
pub use dataloader::{
    BatchLoadFn, CacheStats, DataLoader, DataLoaderConfig, DataLoaderError, DataLoaderResult,
    SimpleBatchLoader,
};

// Subscription types
pub use subscription::{
    ConnectionManager, ConnectionState, EventBus, SubscriptionError, SubscriptionEvent,
    SubscriptionFilter, SubscriptionManager, SubscriptionPayload, SubscriptionResult,
    SubscriptionSource, SubscriptionStream, WsConnection, WsMessage,
};

// Complexity types
pub use complexity::{
    ComplexityAnalysis, ComplexityAnalyzer, ComplexityConfig, ComplexityError, ComplexityResult,
    DynamicCost, FieldCostCalculator, FieldCostRegistry, QueryCostEstimate, QueryCostEstimator,
    StaticCost,
};

// Federation types
pub use federation::{
    EntityKey, EntityRepresentation, EntityResolver, EntityResolverRegistry, FederatedQueryExecutor,
    FederationDirective, FederationError, FederationGateway, FederationResult, QueryPlanStep,
    QueryPlanner, ReferenceResolver, SchemaStitcher, ServiceDefinition, SimpleReferenceResolver,
};

// Persisted query types
pub use persisted::{
    APQExtension, InMemoryStorage, PersistedQuery, PersistedQueryConfig, PersistedQueryError,
    PersistedQueryManager, PersistedQueryResult, QueryHash, QueryStorage, StorageStats,
};

// ============================================================================
// GraphQL Module Version
// ============================================================================

/// GraphQL module version
pub const GRAPHQL_VERSION: &str = "0.2.0";

/// GraphQL module build date
pub const GRAPHQL_BUILD_DATE: &str = "2025-12-28";

// ============================================================================
// Utility Functions
// ============================================================================

/// Create a simple GraphQL server setup
///
/// This is a convenience function that creates a basic GraphQL server
/// with sensible defaults for common use cases.
///
/// # Example
///
/// ```rust,no_run
/// use caddy::enterprise::graphql;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let (schema, executor) = graphql::create_simple_server()?;
/// // Use schema and executor for your GraphQL API
/// # Ok(())
/// # }
/// ```
pub fn create_simple_server() -> SchemaResult<(std::sync::Arc<Schema>, QueryExecutor)> {
    let schema = std::sync::Arc::new(Schema::new());
    let executor = QueryExecutor::new(std::sync::Arc::clone(&schema));
    Ok((schema, executor))
}

// ============================================================================
// Integration Helpers
// ============================================================================

/// GraphQL server builder for easy configuration
///
/// Provides a fluent API for configuring a complete GraphQL server
/// with all enterprise features enabled.
pub struct GraphQLServerBuilder {
    schema: Schema,
    complexity_config: Option<ComplexityConfig>,
    persisted_query_config: Option<PersistedQueryConfig>,
    enable_subscriptions: bool,
    enable_federation: bool,
}

impl GraphQLServerBuilder {
    /// Create a new server builder
    pub fn new() -> Self {
        Self {
            schema: Schema::new(),
            complexity_config: None,
            persisted_query_config: None,
            enable_subscriptions: false,
            enable_federation: false,
        }
    }

    /// Set the schema
    pub fn schema(mut self, schema: Schema) -> Self {
        self.schema = schema;
        self
    }

    /// Enable complexity analysis
    pub fn with_complexity_analysis(mut self, config: ComplexityConfig) -> Self {
        self.complexity_config = Some(config);
        self
    }

    /// Enable persisted queries
    pub fn with_persisted_queries(mut self, config: PersistedQueryConfig) -> Self {
        self.persisted_query_config = Some(config);
        self
    }

    /// Enable subscriptions
    pub fn with_subscriptions(mut self) -> Self {
        self.enable_subscriptions = true;
        self
    }

    /// Enable federation
    pub fn with_federation(mut self) -> Self {
        self.enable_federation = true;
        self
    }

    /// Build the server components
    pub fn build(self) -> SchemaResult<GraphQLServer> {
        let schema = std::sync::Arc::new(self.schema);
        let executor = QueryExecutor::new(std::sync::Arc::clone(&schema));

        let complexity_analyzer = self.complexity_config.map(|config| {
            ComplexityAnalyzer::with_config(std::sync::Arc::clone(&schema), config)
        });

        let persisted_query_manager = self.persisted_query_config.map(|config| {
            PersistedQueryManager::with_memory_storage(config)
        });

        let subscription_manager = if self.enable_subscriptions {
            Some(SubscriptionManager::new())
        } else {
            None
        };

        Ok(GraphQLServer {
            schema,
            executor,
            complexity_analyzer,
            persisted_query_manager,
            subscription_manager,
        })
    }
}

impl Default for GraphQLServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete GraphQL server with all components
pub struct GraphQLServer {
    /// Schema
    pub schema: std::sync::Arc<Schema>,
    /// Query executor
    pub executor: QueryExecutor,
    /// Complexity analyzer (optional)
    pub complexity_analyzer: Option<ComplexityAnalyzer>,
    /// Persisted query manager (optional)
    pub persisted_query_manager: Option<PersistedQueryManager>,
    /// Subscription manager (optional)
    pub subscription_manager: Option<SubscriptionManager>,
}

impl GraphQLServer {
    /// Create a new server with default configuration
    pub fn new(schema: Schema) -> SchemaResult<Self> {
        GraphQLServerBuilder::new().schema(schema).build()
    }

    /// Execute a query
    pub async fn execute(
        &self,
        query: &str,
        variables: std::collections::HashMap<String, Value>,
        context: ResolverContext,
    ) -> ExecutionResult {
        // Parse query
        let parser = QueryParser::new(query);
        let document = match parser.parse() {
            Ok(doc) => doc,
            Err(e) => {
                return ExecutionResult {
                    data: None,
                    errors: vec![GraphQLError::new(e.to_string())],
                }
            }
        };

        // Check complexity if enabled
        if let Some(analyzer) = &self.complexity_analyzer {
            if let Err(e) = analyzer.analyze(&document) {
                return ExecutionResult {
                    data: None,
                    errors: vec![GraphQLError::new(format!("Complexity error: {}", e))],
                };
            }
        }

        // Execute query
        self.executor.execute(&document, variables, context).await
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constants() {
        assert_eq!(GRAPHQL_VERSION, "0.2.0");
        assert!(!GRAPHQL_BUILD_DATE.is_empty());
    }

    #[test]
    fn test_simple_server_creation() {
        let result = create_simple_server();
        assert!(result.is_ok());
    }

    #[test]
    fn test_server_builder() {
        let builder = GraphQLServerBuilder::new()
            .with_complexity_analysis(ComplexityConfig::default())
            .with_persisted_queries(PersistedQueryConfig::default())
            .with_subscriptions();

        let server = builder.build();
        assert!(server.is_ok());

        let server = server.unwrap();
        assert!(server.complexity_analyzer.is_some());
        assert!(server.persisted_query_manager.is_some());
        assert!(server.subscription_manager.is_some());
    }

    #[tokio::test]
    async fn test_graphql_server() {
        use std::sync::Arc;

        let mut schema = Schema::new();

        let resolver = Arc::new(FnResolver::new(|_ctx, _parent, _args| {
            Ok(Value::String("test".to_string()))
        }));

        let query_type = ObjectType::new("Query").field(Field::new(
            "test",
            TypeRef::Named("String".to_string()),
            resolver,
        ));

        schema
            .add_type(TypeDefinition::Object(query_type))
            .unwrap();
        schema.set_query_type("Query");

        let server = GraphQLServer::new(schema).unwrap();
        let ctx = ResolverContext::new("test-req");

        let result = server
            .execute("query { test }", std::collections::HashMap::new(), ctx)
            .await;

        assert!(result.is_success());
    }
}
