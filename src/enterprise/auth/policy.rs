//! Policy engine for complex authorization rules and ABAC.
//!
//! This module provides:
//! - Attribute-based access control (ABAC)
//! - Policy engine for complex authorization rules
//! - Policy evaluation with caching
//! - Context-aware access control

use super::permission::Permission;
use super::role::RoleManager;
use super::user::User;
use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during policy operations
#[derive(Error, Debug)]
pub enum PolicyError {
    #[error("Policy not found: {0}")]
    NotFound(String),

    #[error("Invalid policy: {0}")]
    Invalid(String),

    #[error("Policy evaluation error: {0}")]
    EvaluationError(String),

    #[error("Access denied by policy: {0}")]
    AccessDenied(String),

    #[error("Condition evaluation failed: {0}")]
    ConditionFailed(String),
}

/// Result type for policy operations
pub type PolicyResult<T> = Result<T, PolicyError>;

/// Policy effect - allow or deny
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    Allow,
    Deny,
}

/// Comparison operators for conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Contains,
    NotContains,
    In,
    NotIn,
}

impl Operator {
    /// Evaluate the operator on two string values
    pub fn evaluate(&self, left: &str, right: &str) -> bool {
        match self {
            Operator::Equals => left == right,
            Operator::NotEquals => left != right,
            Operator::GreaterThan => left > right,
            Operator::LessThan => left < right,
            Operator::GreaterThanOrEqual => left >= right,
            Operator::LessThanOrEqual => left <= right,
            Operator::Contains => left.contains(right),
            Operator::NotContains => !left.contains(right),
            Operator::In => right.split(',').any(|v| v.trim() == left),
            Operator::NotIn => !right.split(',').any(|v| v.trim() == left),
        }
    }
}

/// A condition in a policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    /// Attribute name (e.g., "user.department", "resource.owner")
    pub attribute: String,

    /// Comparison operator
    pub operator: Operator,

    /// Value to compare against
    pub value: String,
}

impl Condition {
    /// Create a new condition
    pub fn new(attribute: String, operator: Operator, value: String) -> Self {
        Self {
            attribute,
            operator,
            value,
        }
    }

    /// Evaluate the condition against a context
    pub fn evaluate(&self, context: &PolicyContext) -> bool {
        if let Some(actual_value) = context.get_attribute(&self.attribute) {
            self.operator.evaluate(actual_value, &self.value)
        } else {
            false
        }
    }
}

/// A policy statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statement {
    /// Statement ID
    pub id: String,

    /// Effect of the statement
    pub effect: Effect,

    /// Permissions this statement applies to
    pub permissions: Vec<Permission>,

    /// Resources this statement applies to (e.g., ["drawing:*", "project:123"])
    pub resources: Vec<String>,

    /// Conditions that must be met
    pub conditions: Vec<Condition>,
}

impl Statement {
    /// Create a new statement
    pub fn new(id: String, effect: Effect) -> Self {
        Self {
            id,
            effect,
            permissions: Vec::new(),
            resources: Vec::new(),
            conditions: Vec::new(),
        }
    }

    /// Add a permission to the statement
    pub fn add_permission(mut self, permission: Permission) -> Self {
        self.permissions.push(permission);
        self
    }

    /// Add multiple permissions to the statement
    pub fn add_permissions(mut self, permissions: Vec<Permission>) -> Self {
        self.permissions.extend(permissions);
        self
    }

    /// Add a resource pattern
    pub fn add_resource(mut self, resource: String) -> Self {
        self.resources.push(resource);
        self
    }

    /// Add a condition
    pub fn add_condition(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Check if this statement applies to a permission and resource
    pub fn applies_to(&self, permission: &Permission, resource: &str) -> bool {
        let permission_matches = self.permissions.is_empty()
            || self.permissions.contains(permission)
            || self.permissions.contains(&Permission::SuperAdmin);

        let resource_matches = self.resources.is_empty()
            || self.resources.iter().any(|pattern| {
                if pattern.ends_with('*') {
                    let prefix = &pattern[..pattern.len() - 1];
                    resource.starts_with(prefix)
                } else {
                    pattern == resource
                }
            });

        permission_matches && resource_matches
    }

    /// Evaluate the statement conditions
    pub fn evaluate_conditions(&self, context: &PolicyContext) -> bool {
        if self.conditions.is_empty() {
            return true;
        }

        self.conditions.iter().all(|c| c.evaluate(context))
    }

    /// Evaluate the statement
    pub fn evaluate(&self, permission: &Permission, resource: &str, context: &PolicyContext) -> Option<Effect> {
        if self.applies_to(permission, resource) && self.evaluate_conditions(context) {
            Some(self.effect)
        } else {
            None
        }
    }
}

/// A complete policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    /// Policy ID
    pub id: String,

    /// Policy name
    pub name: String,

    /// Policy description
    pub description: String,

    /// Policy statements
    pub statements: Vec<Statement>,

    /// Policy priority (higher priority policies are evaluated first)
    pub priority: i32,

    /// Whether this policy is active
    pub active: bool,

    /// Policy metadata
    pub metadata: HashMap<String, String>,
}

impl Policy {
    /// Create a new policy
    pub fn new(id: String, name: String, description: String) -> Self {
        Self {
            id,
            name,
            description,
            statements: Vec::new(),
            priority: 0,
            active: true,
            metadata: HashMap::new(),
        }
    }

    /// Add a statement to the policy
    pub fn add_statement(mut self, statement: Statement) -> Self {
        self.statements.push(statement);
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Evaluate the policy
    pub fn evaluate(&self, permission: &Permission, resource: &str, context: &PolicyContext) -> Option<Effect> {
        if !self.active {
            return None;
        }

        // Evaluate all statements
        for statement in &self.statements {
            if let Some(effect) = statement.evaluate(permission, resource, context) {
                return Some(effect);
            }
        }

        None
    }
}

/// Context for policy evaluation
#[derive(Debug, Clone)]
pub struct PolicyContext {
    /// User attributes
    user_attributes: HashMap<String, String>,

    /// Resource attributes
    resource_attributes: HashMap<String, String>,

    /// Environment attributes
    environment_attributes: HashMap<String, String>,

    /// Custom attributes
    custom_attributes: HashMap<String, String>,
}

impl PolicyContext {
    /// Create a new policy context
    pub fn new() -> Self {
        Self {
            user_attributes: HashMap::new(),
            resource_attributes: HashMap::new(),
            environment_attributes: HashMap::new(),
            custom_attributes: HashMap::new(),
        }
    }

    /// Create context from user
    pub fn from_user(user: &User) -> Self {
        let mut context = Self::new();

        context.set_user_attribute("id".to_string(), user.id.clone());
        context.set_user_attribute("username".to_string(), user.username.clone());
        context.set_user_attribute("email".to_string(), user.email.clone());
        context.set_user_attribute("status".to_string(), format!("{:?}", user.status));

        for (key, value) in &user.metadata {
            context.set_user_attribute(key.clone(), value.clone());
        }

        // Add environment attributes
        let now = Utc::now();
        context.set_environment_attribute("time".to_string(), now.to_rfc3339());
        context.set_environment_attribute("hour".to_string(), now.hour().to_string());
        context.set_environment_attribute("day_of_week".to_string(), now.weekday().to_string());

        context
    }

    /// Set a user attribute
    pub fn set_user_attribute(&mut self, key: String, value: String) {
        self.user_attributes.insert(key, value);
    }

    /// Set a resource attribute
    pub fn set_resource_attribute(&mut self, key: String, value: String) {
        self.resource_attributes.insert(key, value);
    }

    /// Set an environment attribute
    pub fn set_environment_attribute(&mut self, key: String, value: String) {
        self.environment_attributes.insert(key, value);
    }

    /// Set a custom attribute
    pub fn set_custom_attribute(&mut self, key: String, value: String) {
        self.custom_attributes.insert(key, value);
    }

    /// Get an attribute value
    pub fn get_attribute(&self, key: &str) -> Option<&String> {
        if let Some(value) = key.strip_prefix("user.") {
            self.user_attributes.get(value)
        } else if let Some(value) = key.strip_prefix("resource.") {
            self.resource_attributes.get(value)
        } else if let Some(value) = key.strip_prefix("env.") {
            self.environment_attributes.get(value)
        } else {
            self.custom_attributes.get(key)
        }
    }
}

impl Default for PolicyContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Policy engine for evaluating policies
pub struct PolicyEngine {
    policies: HashMap<String, Policy>,
    cache: HashMap<String, (Effect, DateTime<Utc>)>,
    cache_ttl_seconds: i64,
}

impl PolicyEngine {
    /// Create a new policy engine
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
            cache: HashMap::new(),
            cache_ttl_seconds: 300, // 5 minutes
        }
    }

    /// Create a policy engine with custom cache TTL
    pub fn with_cache_ttl(cache_ttl_seconds: i64) -> Self {
        Self {
            policies: HashMap::new(),
            cache: HashMap::new(),
            cache_ttl_seconds,
        }
    }

    /// Add a policy
    pub fn add_policy(&mut self, policy: Policy) -> PolicyResult<()> {
        self.policies.insert(policy.id.clone(), policy);
        self.clear_cache();
        Ok(())
    }

    /// Remove a policy
    pub fn remove_policy(&mut self, id: &str) -> PolicyResult<()> {
        if self.policies.remove(id).is_none() {
            return Err(PolicyError::NotFound(id.to_string()));
        }
        self.clear_cache();
        Ok(())
    }

    /// Get a policy
    pub fn get_policy(&self, id: &str) -> PolicyResult<&Policy> {
        self.policies
            .get(id)
            .ok_or_else(|| PolicyError::NotFound(id.to_string()))
    }

    /// List all policies
    pub fn list_policies(&self) -> Vec<&Policy> {
        let mut policies: Vec<&Policy> = self.policies.values().collect();
        policies.sort_by(|a, b| b.priority.cmp(&a.priority));
        policies
    }

    /// Evaluate policies for a permission and resource
    pub fn evaluate(
        &mut self,
        permission: &Permission,
        resource: &str,
        context: &PolicyContext,
    ) -> PolicyResult<bool> {
        // Check cache first
        let cache_key = format!("{:?}:{}:{:?}", permission, resource, context.user_attributes);

        if let Some((effect, timestamp)) = self.cache.get(&cache_key) {
            if (Utc::now() - *timestamp).num_seconds() < self.cache_ttl_seconds {
                return Ok(*effect == Effect::Allow);
            }
        }

        // Evaluate policies in priority order
        let mut policies: Vec<&Policy> = self.policies.values().collect();
        policies.sort_by(|a, b| b.priority.cmp(&a.priority));

        let mut final_effect = Effect::Deny; // Default deny

        for policy in policies {
            if let Some(effect) = policy.evaluate(permission, resource, context) {
                match effect {
                    Effect::Deny => {
                        // Explicit deny always wins
                        final_effect = Effect::Deny;
                        break;
                    }
                    Effect::Allow => {
                        final_effect = Effect::Allow;
                    }
                }
            }
        }

        // Cache the result
        self.cache.insert(cache_key, (final_effect, Utc::now()));

        Ok(final_effect == Effect::Allow)
    }

    /// Check if a user has permission with policy evaluation
    pub fn check_permission(
        &mut self,
        user: &User,
        permission: &Permission,
        resource: &str,
        role_manager: &RoleManager,
    ) -> bool {
        // First check direct permissions from roles
        if user.has_permission(permission, role_manager) {
            // Then evaluate policies
            let mut context = PolicyContext::from_user(user);
            context.set_resource_attribute("id".to_string(), resource.to_string());

            return self.evaluate(permission, resource, &context).unwrap_or(false);
        }

        false
    }

    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Remove expired cache entries
    pub fn cleanup_cache(&mut self) {
        let now = Utc::now();
        self.cache.retain(|_, (_, timestamp)| {
            (now - *timestamp).num_seconds() < self.cache_ttl_seconds
        });
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to create common policy patterns
pub struct PolicyBuilder;

impl PolicyBuilder {
    /// Create a time-based access policy
    pub fn time_based_access(
        id: String,
        permission: Permission,
        resource: String,
        start_hour: u32,
        end_hour: u32,
    ) -> Policy {
        let mut policy = Policy::new(
            id,
            "Time-based Access".to_string(),
            format!("Allow access between {} and {}", start_hour, end_hour),
        );

        let statement = Statement::new("time_restriction".to_string(), Effect::Allow)
            .add_permission(permission)
            .add_resource(resource)
            .add_condition(Condition::new(
                "env.hour".to_string(),
                Operator::GreaterThanOrEqual,
                start_hour.to_string(),
            ))
            .add_condition(Condition::new(
                "env.hour".to_string(),
                Operator::LessThan,
                end_hour.to_string(),
            ));

        policy.add_statement(statement)
    }

    /// Create a resource owner policy
    pub fn resource_owner_access(id: String, permission: Permission, resource_pattern: String) -> Policy {
        let mut policy = Policy::new(
            id,
            "Resource Owner Access".to_string(),
            "Allow access to own resources".to_string(),
        );

        let statement = Statement::new("owner_access".to_string(), Effect::Allow)
            .add_permission(permission)
            .add_resource(resource_pattern)
            .add_condition(Condition::new(
                "user.id".to_string(),
                Operator::Equals,
                "{{resource.owner}}".to_string(),
            ));

        policy.add_statement(statement)
    }

    /// Create a department-based policy
    pub fn department_based_access(
        id: String,
        permissions: Vec<Permission>,
        resource_pattern: String,
        department: String,
    ) -> Policy {
        let mut policy = Policy::new(
            id,
            "Department-based Access".to_string(),
            format!("Allow access for {} department", department),
        );

        let statement = Statement::new("dept_access".to_string(), Effect::Allow)
            .add_permissions(permissions)
            .add_resource(resource_pattern)
            .add_condition(Condition::new(
                "user.department".to_string(),
                Operator::Equals,
                department,
            ));

        policy.add_statement(statement)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operator_evaluation() {
        assert!(Operator::Equals.evaluate("test", "test"));
        assert!(!Operator::Equals.evaluate("test", "other"));

        assert!(Operator::GreaterThan.evaluate("b", "a"));
        assert!(!Operator::GreaterThan.evaluate("a", "b"));

        assert!(Operator::Contains.evaluate("hello world", "world"));
        assert!(!Operator::Contains.evaluate("hello", "world"));

        assert!(Operator::In.evaluate("a", "a,b,c"));
        assert!(!Operator::In.evaluate("d", "a,b,c"));
    }

    #[test]
    fn test_condition_evaluation() {
        let mut context = PolicyContext::new();
        context.set_user_attribute("department".to_string(), "engineering".to_string());

        let condition = Condition::new(
            "user.department".to_string(),
            Operator::Equals,
            "engineering".to_string(),
        );

        assert!(condition.evaluate(&context));

        let condition2 = Condition::new(
            "user.department".to_string(),
            Operator::Equals,
            "sales".to_string(),
        );

        assert!(!condition2.evaluate(&context));
    }

    #[test]
    fn test_statement_applies_to() {
        let statement = Statement::new("test".to_string(), Effect::Allow)
            .add_permission(Permission::DrawingCreate)
            .add_resource("drawing:*".to_string());

        assert!(statement.applies_to(&Permission::DrawingCreate, "drawing:123"));
        assert!(statement.applies_to(&Permission::DrawingCreate, "drawing:456"));
        assert!(!statement.applies_to(&Permission::DrawingCreate, "project:123"));
        assert!(!statement.applies_to(&Permission::DrawingDelete, "drawing:123"));
    }

    #[test]
    fn test_policy_evaluation() {
        let mut context = PolicyContext::new();
        context.set_user_attribute("department".to_string(), "engineering".to_string());

        let statement = Statement::new("eng_access".to_string(), Effect::Allow)
            .add_permission(Permission::DrawingCreate)
            .add_resource("drawing:*".to_string())
            .add_condition(Condition::new(
                "user.department".to_string(),
                Operator::Equals,
                "engineering".to_string(),
            ));

        let policy = Policy::new(
            "eng_policy".to_string(),
            "Engineering Access".to_string(),
            "Allow engineering to create drawings".to_string(),
        )
        .add_statement(statement);

        let result = policy.evaluate(&Permission::DrawingCreate, "drawing:123", &context);
        assert_eq!(result, Some(Effect::Allow));
    }

    #[test]
    fn test_policy_engine() {
        let mut engine = PolicyEngine::new();

        let statement = Statement::new("allow_all".to_string(), Effect::Allow)
            .add_permission(Permission::DrawingRead);

        let policy = Policy::new(
            "read_policy".to_string(),
            "Read Policy".to_string(),
            "Allow all to read".to_string(),
        )
        .add_statement(statement);

        engine.add_policy(policy).unwrap();

        let context = PolicyContext::new();
        let result = engine.evaluate(&Permission::DrawingRead, "drawing:123", &context);

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_policy_builder() {
        let policy = PolicyBuilder::time_based_access(
            "work_hours".to_string(),
            Permission::DrawingCreate,
            "drawing:*".to_string(),
            9,
            17,
        );

        assert_eq!(policy.statements.len(), 1);
        assert_eq!(policy.statements[0].conditions.len(), 2);
    }
}
