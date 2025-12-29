//! GraphQL Schema Definition and Type System
//!
//! This module provides a complete GraphQL schema definition system including:
//! - Type system (Object, Input, Enum, Interface, Union)
//! - Field definitions with resolvers
//! - Directive support
//! - Schema introspection

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use thiserror::Error;

// ============================================================================
// Error Types
// ============================================================================

/// Schema-related errors
#[derive(Error, Debug)]
pub enum SchemaError {
    /// Type not found in schema
    #[error("Type not found: {0}")]
    TypeNotFound(String),

    /// Field not found on type
    #[error("Field '{0}' not found on type '{1}'")]
    FieldNotFound(String, String),

    /// Invalid type definition
    #[error("Invalid type definition: {0}")]
    InvalidType(String),

    /// Circular type reference
    #[error("Circular type reference detected: {0}")]
    CircularReference(String),

    /// Resolver error
    #[error("Resolver error: {0}")]
    ResolverError(String),

    /// Directive error
    #[error("Directive error: {0}")]
    DirectiveError(String),
}

pub type SchemaResult<T> = Result<T, SchemaError>;

// ============================================================================
// GraphQL Value Types
// ============================================================================

/// GraphQL value representation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Value {
    /// Null value
    Null,
    /// Integer value
    Int(i64),
    /// Float value
    Float(f64),
    /// String value
    String(String),
    /// Boolean value
    Boolean(bool),
    /// Enum value
    Enum(String),
    /// List of values
    List(Vec<Value>),
    /// Object with field-value pairs
    Object(HashMap<String, Value>),
}

impl Value {
    /// Check if value is null
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Convert to string if possible
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// Convert to integer if possible
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// Convert to float if possible
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Int(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// Convert to boolean if possible
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Enum(e) => write!(f, "{}", e),
            Value::List(l) => {
                write!(f, "[")?;
                for (i, v) in l.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Object(o) => {
                write!(f, "{{")?;
                for (i, (k, v)) in o.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
        }
    }
}

// ============================================================================
// Type System
// ============================================================================

/// GraphQL type kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeKind {
    /// Scalar type
    Scalar,
    /// Object type
    Object,
    /// Interface type
    Interface,
    /// Union type
    Union,
    /// Enum type
    Enum,
    /// Input object type
    InputObject,
    /// List type
    List,
    /// Non-null type
    NonNull,
}

/// Type reference (can be nullable, list, etc.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeRef {
    /// Named type (nullable by default)
    Named(String),
    /// Non-null type
    NonNull(Box<TypeRef>),
    /// List type
    List(Box<TypeRef>),
}

impl TypeRef {
    /// Create a non-null version of this type
    pub fn non_null(self) -> Self {
        TypeRef::NonNull(Box::new(self))
    }

    /// Create a list version of this type
    pub fn list(self) -> Self {
        TypeRef::List(Box::new(self))
    }

    /// Get the base type name
    pub fn base_type(&self) -> &str {
        match self {
            TypeRef::Named(name) => name,
            TypeRef::NonNull(inner) | TypeRef::List(inner) => inner.base_type(),
        }
    }

    /// Check if type is nullable
    pub fn is_nullable(&self) -> bool {
        !matches!(self, TypeRef::NonNull(_))
    }
}

impl fmt::Display for TypeRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeRef::Named(name) => write!(f, "{}", name),
            TypeRef::NonNull(inner) => write!(f, "{}!", inner),
            TypeRef::List(inner) => write!(f, "[{}]", inner),
        }
    }
}

// ============================================================================
// Field Definitions
// ============================================================================

/// GraphQL field argument
#[derive(Debug, Clone)]
pub struct InputValue {
    /// Argument name
    pub name: String,
    /// Argument description
    pub description: Option<String>,
    /// Argument type
    pub type_ref: TypeRef,
    /// Default value
    pub default_value: Option<Value>,
    /// Directives applied to this argument
    pub directives: Vec<Directive>,
}

impl InputValue {
    /// Create a new input value
    pub fn new(name: impl Into<String>, type_ref: TypeRef) -> Self {
        Self {
            name: name.into(),
            description: None,
            type_ref,
            default_value: None,
            directives: Vec::new(),
        }
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set default value
    pub fn default_value(mut self, value: Value) -> Self {
        self.default_value = Some(value);
        self
    }
}

/// Resolver context containing request-scoped data
#[derive(Debug, Clone)]
pub struct ResolverContext {
    /// User ID if authenticated
    pub user_id: Option<String>,
    /// Request ID for tracing
    pub request_id: String,
    /// Custom context data
    pub data: HashMap<String, Value>,
}

impl ResolverContext {
    /// Create a new resolver context
    pub fn new(request_id: impl Into<String>) -> Self {
        Self {
            user_id: None,
            request_id: request_id.into(),
            data: HashMap::new(),
        }
    }

    /// Set user ID
    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Insert custom data
    pub fn insert(&mut self, key: impl Into<String>, value: Value) {
        self.data.insert(key.into(), value);
    }

    /// Get custom data
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }
}

/// Field resolver trait
#[async_trait]
pub trait FieldResolver: Send + Sync {
    /// Resolve field value
    async fn resolve(
        &self,
        ctx: &ResolverContext,
        parent: &Value,
        args: &HashMap<String, Value>,
    ) -> SchemaResult<Value>;
}

/// Function-based resolver
pub struct FnResolver<F>
where
    F: Fn(&ResolverContext, &Value, &HashMap<String, Value>) -> SchemaResult<Value>
        + Send
        + Sync,
{
    func: F,
}

impl<F> FnResolver<F>
where
    F: Fn(&ResolverContext, &Value, &HashMap<String, Value>) -> SchemaResult<Value>
        + Send
        + Sync,
{
    /// Create a new function resolver
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

#[async_trait]
impl<F> FieldResolver for FnResolver<F>
where
    F: Fn(&ResolverContext, &Value, &HashMap<String, Value>) -> SchemaResult<Value>
        + Send
        + Sync,
{
    async fn resolve(
        &self,
        ctx: &ResolverContext,
        parent: &Value,
        args: &HashMap<String, Value>,
    ) -> SchemaResult<Value> {
        (self.func)(ctx, parent, args)
    }
}

/// GraphQL field definition
#[derive(Clone)]
pub struct Field {
    /// Field name
    pub name: String,
    /// Field description
    pub description: Option<String>,
    /// Field type
    pub type_ref: TypeRef,
    /// Field arguments
    pub args: Vec<InputValue>,
    /// Field resolver
    pub resolver: Arc<dyn FieldResolver>,
    /// Directives applied to this field
    pub directives: Vec<Directive>,
    /// Whether field is deprecated
    pub deprecated: Option<String>,
}

impl Field {
    /// Create a new field
    pub fn new(
        name: impl Into<String>,
        type_ref: TypeRef,
        resolver: Arc<dyn FieldResolver>,
    ) -> Self {
        Self {
            name: name.into(),
            description: None,
            type_ref,
            args: Vec::new(),
            resolver,
            directives: Vec::new(),
            deprecated: None,
        }
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add argument
    pub fn arg(mut self, arg: InputValue) -> Self {
        self.args.push(arg);
        self
    }

    /// Mark as deprecated
    pub fn deprecated(mut self, reason: impl Into<String>) -> Self {
        self.deprecated = Some(reason.into());
        self
    }
}

impl fmt::Debug for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Field")
            .field("name", &self.name)
            .field("type_ref", &self.type_ref)
            .field("args", &self.args)
            .finish()
    }
}

// ============================================================================
// Type Definitions
// ============================================================================

/// Scalar type definition
#[derive(Debug, Clone)]
pub struct ScalarType {
    /// Scalar name
    pub name: String,
    /// Scalar description
    pub description: Option<String>,
}

/// Object type definition
#[derive(Clone)]
pub struct ObjectType {
    /// Object name
    pub name: String,
    /// Object description
    pub description: Option<String>,
    /// Fields on this object
    pub fields: Vec<Field>,
    /// Interfaces this object implements
    pub interfaces: Vec<String>,
}

impl ObjectType {
    /// Create a new object type
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            fields: Vec::new(),
            interfaces: Vec::new(),
        }
    }

    /// Add a field
    pub fn field(mut self, field: Field) -> Self {
        self.fields.push(field);
        self
    }

    /// Implement an interface
    pub fn implements(mut self, interface: impl Into<String>) -> Self {
        self.interfaces.push(interface.into());
        self
    }

    /// Get field by name
    pub fn get_field(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|f| f.name == name)
    }
}

impl fmt::Debug for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ObjectType")
            .field("name", &self.name)
            .field("fields", &self.fields)
            .field("interfaces", &self.interfaces)
            .finish()
    }
}

/// Interface type definition
#[derive(Clone)]
pub struct InterfaceType {
    /// Interface name
    pub name: String,
    /// Interface description
    pub description: Option<String>,
    /// Fields that implementers must have
    pub fields: Vec<Field>,
}

impl fmt::Debug for InterfaceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InterfaceType")
            .field("name", &self.name)
            .field("fields", &self.fields)
            .finish()
    }
}

/// Union type definition
#[derive(Debug, Clone)]
pub struct UnionType {
    /// Union name
    pub name: String,
    /// Union description
    pub description: Option<String>,
    /// Possible types in this union
    pub types: Vec<String>,
}

/// Enum value definition
#[derive(Debug, Clone)]
pub struct EnumValue {
    /// Value name
    pub name: String,
    /// Value description
    pub description: Option<String>,
    /// Deprecation reason
    pub deprecated: Option<String>,
}

/// Enum type definition
#[derive(Debug, Clone)]
pub struct EnumType {
    /// Enum name
    pub name: String,
    /// Enum description
    pub description: Option<String>,
    /// Possible values
    pub values: Vec<EnumValue>,
}

/// Input object field
#[derive(Debug, Clone)]
pub struct InputField {
    /// Field name
    pub name: String,
    /// Field description
    pub description: Option<String>,
    /// Field type
    pub type_ref: TypeRef,
    /// Default value
    pub default_value: Option<Value>,
}

/// Input object type definition
#[derive(Debug, Clone)]
pub struct InputObjectType {
    /// Input object name
    pub name: String,
    /// Input object description
    pub description: Option<String>,
    /// Input fields
    pub fields: Vec<InputField>,
}

// ============================================================================
// Directives
// ============================================================================

/// Directive location (where it can be applied)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectiveLocation {
    /// Query operation
    Query,
    /// Mutation operation
    Mutation,
    /// Subscription operation
    Subscription,
    /// Field
    Field,
    /// Fragment definition
    FragmentDefinition,
    /// Fragment spread
    FragmentSpread,
    /// Inline fragment
    InlineFragment,
    /// Schema definition
    Schema,
    /// Scalar type
    Scalar,
    /// Object type
    Object,
    /// Field definition
    FieldDefinition,
    /// Argument definition
    ArgumentDefinition,
    /// Interface type
    Interface,
    /// Union type
    Union,
    /// Enum type
    Enum,
    /// Enum value
    EnumValue,
    /// Input object type
    InputObject,
    /// Input field definition
    InputFieldDefinition,
}

/// Directive definition
#[derive(Debug, Clone)]
pub struct DirectiveDefinition {
    /// Directive name
    pub name: String,
    /// Directive description
    pub description: Option<String>,
    /// Locations where directive can be applied
    pub locations: Vec<DirectiveLocation>,
    /// Directive arguments
    pub args: Vec<InputValue>,
    /// Whether directive is repeatable
    pub is_repeatable: bool,
}

/// Applied directive instance
#[derive(Debug, Clone)]
pub struct Directive {
    /// Directive name
    pub name: String,
    /// Directive arguments
    pub args: HashMap<String, Value>,
}

// ============================================================================
// Schema
// ============================================================================

/// Type definition enum
#[derive(Clone)]
pub enum TypeDefinition {
    /// Scalar type
    Scalar(ScalarType),
    /// Object type
    Object(ObjectType),
    /// Interface type
    Interface(InterfaceType),
    /// Union type
    Union(UnionType),
    /// Enum type
    Enum(EnumType),
    /// Input object type
    InputObject(InputObjectType),
}

impl TypeDefinition {
    /// Get type name
    pub fn name(&self) -> &str {
        match self {
            TypeDefinition::Scalar(t) => &t.name,
            TypeDefinition::Object(t) => &t.name,
            TypeDefinition::Interface(t) => &t.name,
            TypeDefinition::Union(t) => &t.name,
            TypeDefinition::Enum(t) => &t.name,
            TypeDefinition::InputObject(t) => &t.name,
        }
    }

    /// Get type kind
    pub fn kind(&self) -> TypeKind {
        match self {
            TypeDefinition::Scalar(_) => TypeKind::Scalar,
            TypeDefinition::Object(_) => TypeKind::Object,
            TypeDefinition::Interface(_) => TypeKind::Interface,
            TypeDefinition::Union(_) => TypeKind::Union,
            TypeDefinition::Enum(_) => TypeKind::Enum,
            TypeDefinition::InputObject(_) => TypeKind::InputObject,
        }
    }
}

impl fmt::Debug for TypeDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeDefinition::Scalar(t) => write!(f, "Scalar({})", t.name),
            TypeDefinition::Object(t) => write!(f, "Object({})", t.name),
            TypeDefinition::Interface(t) => write!(f, "Interface({})", t.name),
            TypeDefinition::Union(t) => write!(f, "Union({})", t.name),
            TypeDefinition::Enum(t) => write!(f, "Enum({})", t.name),
            TypeDefinition::InputObject(t) => write!(f, "InputObject({})", t.name),
        }
    }
}

/// GraphQL Schema
pub struct Schema {
    /// Type definitions
    types: HashMap<String, TypeDefinition>,
    /// Directive definitions
    directives: HashMap<String, DirectiveDefinition>,
    /// Query root type name
    query_type: Option<String>,
    /// Mutation root type name
    mutation_type: Option<String>,
    /// Subscription root type name
    subscription_type: Option<String>,
}

impl Schema {
    /// Create a new schema
    pub fn new() -> Self {
        let mut schema = Self {
            types: HashMap::new(),
            directives: HashMap::new(),
            query_type: None,
            mutation_type: None,
            subscription_type: None,
        };

        // Add built-in scalar types
        schema.add_builtin_scalars();
        schema.add_builtin_directives();

        schema
    }

    /// Add built-in scalar types
    fn add_builtin_scalars(&mut self) {
        for name in &["Int", "Float", "String", "Boolean", "ID"] {
            self.types.insert(
                name.to_string(),
                TypeDefinition::Scalar(ScalarType {
                    name: name.to_string(),
                    description: Some(format!("Built-in {} scalar type", name)),
                }),
            );
        }
    }

    /// Add built-in directives
    fn add_builtin_directives(&mut self) {
        // @skip directive
        self.directives.insert(
            "skip".to_string(),
            DirectiveDefinition {
                name: "skip".to_string(),
                description: Some("Skips field if condition is true".to_string()),
                locations: vec![
                    DirectiveLocation::Field,
                    DirectiveLocation::FragmentSpread,
                    DirectiveLocation::InlineFragment,
                ],
                args: vec![InputValue::new("if", TypeRef::Named("Boolean".to_string()).non_null())],
                is_repeatable: false,
            },
        );

        // @include directive
        self.directives.insert(
            "include".to_string(),
            DirectiveDefinition {
                name: "include".to_string(),
                description: Some("Includes field if condition is true".to_string()),
                locations: vec![
                    DirectiveLocation::Field,
                    DirectiveLocation::FragmentSpread,
                    DirectiveLocation::InlineFragment,
                ],
                args: vec![InputValue::new("if", TypeRef::Named("Boolean".to_string()).non_null())],
                is_repeatable: false,
            },
        );

        // @deprecated directive
        self.directives.insert(
            "deprecated".to_string(),
            DirectiveDefinition {
                name: "deprecated".to_string(),
                description: Some("Marks field as deprecated".to_string()),
                locations: vec![DirectiveLocation::FieldDefinition, DirectiveLocation::EnumValue],
                args: vec![InputValue::new(
                    "reason",
                    TypeRef::Named("String".to_string()),
                )
                .default_value(Value::String("No longer supported".to_string()))],
                is_repeatable: false,
            },
        );
    }

    /// Add a type to the schema
    pub fn add_type(&mut self, type_def: TypeDefinition) -> SchemaResult<()> {
        let name = type_def.name().to_string();
        if self.types.contains_key(&name) {
            return Err(SchemaError::InvalidType(format!(
                "Type '{}' already exists",
                name
            )));
        }
        self.types.insert(name, type_def);
        Ok(())
    }

    /// Get a type by name
    pub fn get_type(&self, name: &str) -> Option<&TypeDefinition> {
        self.types.get(name)
    }

    /// Set query root type
    pub fn set_query_type(&mut self, name: impl Into<String>) {
        self.query_type = Some(name.into());
    }

    /// Set mutation root type
    pub fn set_mutation_type(&mut self, name: impl Into<String>) {
        self.mutation_type = Some(name.into());
    }

    /// Set subscription root type
    pub fn set_subscription_type(&mut self, name: impl Into<String>) {
        self.subscription_type = Some(name.into());
    }

    /// Get query root type
    pub fn query_type(&self) -> Option<&ObjectType> {
        self.query_type.as_ref().and_then(|name| {
            self.get_type(name).and_then(|t| match t {
                TypeDefinition::Object(obj) => Some(obj),
                _ => None,
            })
        })
    }

    /// Get mutation root type
    pub fn mutation_type(&self) -> Option<&ObjectType> {
        self.mutation_type.as_ref().and_then(|name| {
            self.get_type(name).and_then(|t| match t {
                TypeDefinition::Object(obj) => Some(obj),
                _ => None,
            })
        })
    }

    /// Get subscription root type
    pub fn subscription_type(&self) -> Option<&ObjectType> {
        self.subscription_type.as_ref().and_then(|name| {
            self.get_type(name).and_then(|t| match t {
                TypeDefinition::Object(obj) => Some(obj),
                _ => None,
            })
        })
    }

    /// Validate schema
    pub fn validate(&self) -> SchemaResult<()> {
        // Check that query type exists
        if self.query_type.is_none() {
            return Err(SchemaError::InvalidType(
                "Schema must have a query type".to_string(),
            ));
        }

        // Validate all type references
        for type_def in self.types.values() {
            self.validate_type(type_def)?;
        }

        Ok(())
    }

    /// Validate a single type
    fn validate_type(&self, type_def: &TypeDefinition) -> SchemaResult<()> {
        match type_def {
            TypeDefinition::Object(obj) => {
                for field in &obj.fields {
                    self.validate_type_ref(&field.type_ref)?;
                    for arg in &field.args {
                        self.validate_type_ref(&arg.type_ref)?;
                    }
                }
            }
            TypeDefinition::Interface(iface) => {
                for field in &iface.fields {
                    self.validate_type_ref(&field.type_ref)?;
                }
            }
            TypeDefinition::InputObject(input) => {
                for field in &input.fields {
                    self.validate_type_ref(&field.type_ref)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Validate a type reference exists
    fn validate_type_ref(&self, type_ref: &TypeRef) -> SchemaResult<()> {
        let base_type = type_ref.base_type();
        if !self.types.contains_key(base_type) {
            return Err(SchemaError::TypeNotFound(base_type.to_string()));
        }
        Ok(())
    }

    /// Get all type names
    pub fn type_names(&self) -> Vec<&str> {
        self.types.keys().map(String::as_str).collect()
    }
}

impl Default for Schema {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Schema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Schema")
            .field("types", &self.types.keys())
            .field("query_type", &self.query_type)
            .field("mutation_type", &self.mutation_type)
            .field("subscription_type", &self.subscription_type)
            .finish()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_display() {
        assert_eq!(Value::Null.to_string(), "null");
        assert_eq!(Value::Int(42).to_string(), "42");
        assert_eq!(Value::Float(3.14).to_string(), "3.14");
        assert_eq!(Value::String("hello".to_string()).to_string(), "\"hello\"");
        assert_eq!(Value::Boolean(true).to_string(), "true");
    }

    #[test]
    fn test_type_ref() {
        let t = TypeRef::Named("String".to_string());
        assert_eq!(t.to_string(), "String");
        assert!(t.is_nullable());

        let t = t.non_null();
        assert_eq!(t.to_string(), "String!");
        assert!(!t.is_nullable());

        let t = TypeRef::Named("Int".to_string()).list().non_null();
        assert_eq!(t.to_string(), "[Int]!");
    }

    #[test]
    fn test_schema_creation() {
        let schema = Schema::new();
        assert!(schema.get_type("String").is_some());
        assert!(schema.get_type("Int").is_some());
        assert!(schema.get_type("Boolean").is_some());
    }

    #[test]
    fn test_object_type() {
        let resolver = Arc::new(FnResolver::new(|_ctx, _parent, _args| {
            Ok(Value::String("test".to_string()))
        }));

        let obj = ObjectType::new("User")
            .field(
                Field::new("id", TypeRef::Named("ID".to_string()).non_null(), resolver.clone())
                    .description("User ID"),
            )
            .field(
                Field::new("name", TypeRef::Named("String".to_string()), resolver)
                    .description("User name"),
            );

        assert_eq!(obj.name, "User");
        assert_eq!(obj.fields.len(), 2);
        assert!(obj.get_field("id").is_some());
        assert!(obj.get_field("name").is_some());
    }

    #[test]
    fn test_resolver_context() {
        let mut ctx = ResolverContext::new("req-123");
        assert_eq!(ctx.request_id, "req-123");
        assert!(ctx.user_id.is_none());

        ctx = ctx.with_user("user-456");
        assert_eq!(ctx.user_id, Some("user-456".to_string()));

        ctx.insert("key", Value::String("value".to_string()));
        assert!(ctx.get("key").is_some());
    }

    #[test]
    fn test_schema_validation() {
        let mut schema = Schema::new();

        let resolver = Arc::new(FnResolver::new(|_ctx, _parent, _args| {
            Ok(Value::Null)
        }));

        let query = ObjectType::new("Query").field(Field::new(
            "hello",
            TypeRef::Named("String".to_string()),
            resolver,
        ));

        schema.add_type(TypeDefinition::Object(query)).unwrap();
        schema.set_query_type("Query");

        assert!(schema.validate().is_ok());
    }
}
