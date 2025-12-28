//! Type-Safe Query Builder
//!
//! Provides a fluent, type-safe interface for building SQL queries with support
//! for complex conditions, joins, pagination, and optimization hints.

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Query builder errors
#[derive(Debug, Error)]
pub enum QueryError {
    #[error("Invalid query: {0}")]
    Invalid(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    #[error("SQL syntax error: {0}")]
    SyntaxError(String),

    #[error("Unsupported operation: {0}")]
    Unsupported(String),
}

/// SQL comparison operators
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Operator {
    Equals,
    NotEquals,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Like,
    ILike,
    In,
    NotIn,
    IsNull,
    IsNotNull,
    Between,
}

impl Operator {
    pub fn to_sql(&self) -> &'static str {
        match self {
            Operator::Equals => "=",
            Operator::NotEquals => "!=",
            Operator::GreaterThan => ">",
            Operator::GreaterThanOrEqual => ">=",
            Operator::LessThan => "<",
            Operator::LessThanOrEqual => "<=",
            Operator::Like => "LIKE",
            Operator::ILike => "ILIKE",
            Operator::In => "IN",
            Operator::NotIn => "NOT IN",
            Operator::IsNull => "IS NULL",
            Operator::IsNotNull => "IS NOT NULL",
            Operator::Between => "BETWEEN",
        }
    }
}

/// Query value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "NULL"),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Integer(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "'{}'", s.replace('\'', "''")),
            Value::Array(arr) => {
                write!(f, "(")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, ")")
            }
        }
    }
}

/// WHERE clause condition
#[derive(Debug, Clone)]
pub struct WhereClause {
    pub column: String,
    pub operator: Operator,
    pub value: Option<Value>,
    pub value2: Option<Value>, // For BETWEEN operator
}

impl WhereClause {
    /// Create a new WHERE clause
    pub fn new(column: impl Into<String>, operator: Operator, value: Value) -> Self {
        Self {
            column: column.into(),
            operator,
            value: Some(value),
            value2: None,
        }
    }

    /// Create BETWEEN clause
    pub fn between(column: impl Into<String>, start: Value, end: Value) -> Self {
        Self {
            column: column.into(),
            operator: Operator::Between,
            value: Some(start),
            value2: Some(end),
        }
    }

    /// Create IS NULL clause
    pub fn is_null(column: impl Into<String>) -> Self {
        Self {
            column: column.into(),
            operator: Operator::IsNull,
            value: None,
            value2: None,
        }
    }

    /// Create IS NOT NULL clause
    pub fn is_not_null(column: impl Into<String>) -> Self {
        Self {
            column: column.into(),
            operator: Operator::IsNotNull,
            value: None,
            value2: None,
        }
    }

    /// Convert to SQL string
    pub fn to_sql(&self) -> String {
        match self.operator {
            Operator::IsNull | Operator::IsNotNull => {
                format!("{} {}", self.column, self.operator.to_sql())
            }
            Operator::Between => {
                if let (Some(v1), Some(v2)) = (&self.value, &self.value2) {
                    format!("{} BETWEEN {} AND {}", self.column, v1, v2)
                } else {
                    format!("{} {}", self.column, self.operator.to_sql())
                }
            }
            _ => {
                if let Some(v) = &self.value {
                    format!("{} {} {}", self.column, self.operator.to_sql(), v)
                } else {
                    format!("{} {}", self.column, self.operator.to_sql())
                }
            }
        }
    }
}

/// Logical operator for combining WHERE clauses
#[derive(Debug, Clone, Copy)]
pub enum LogicalOperator {
    And,
    Or,
}

impl LogicalOperator {
    pub fn to_sql(&self) -> &'static str {
        match self {
            LogicalOperator::And => "AND",
            LogicalOperator::Or => "OR",
        }
    }
}

/// Join type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
    Cross,
}

impl JoinType {
    pub fn to_sql(&self) -> &'static str {
        match self {
            JoinType::Inner => "INNER JOIN",
            JoinType::Left => "LEFT JOIN",
            JoinType::Right => "RIGHT JOIN",
            JoinType::Full => "FULL OUTER JOIN",
            JoinType::Cross => "CROSS JOIN",
        }
    }
}

/// Join clause
#[derive(Debug, Clone)]
pub struct Join {
    pub join_type: JoinType,
    pub table: String,
    pub alias: Option<String>,
    pub on_condition: String,
}

/// Order direction
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OrderDirection {
    Asc,
    Desc,
}

impl OrderDirection {
    pub fn to_sql(&self) -> &'static str {
        match self {
            OrderDirection::Asc => "ASC",
            OrderDirection::Desc => "DESC",
        }
    }
}

/// ORDER BY clause
#[derive(Debug, Clone)]
pub struct OrderBy {
    pub column: String,
    pub direction: OrderDirection,
    pub nulls_first: Option<bool>,
}

impl OrderBy {
    pub fn to_sql(&self) -> String {
        let mut sql = format!("{} {}", self.column, self.direction.to_sql());

        if let Some(nulls_first) = self.nulls_first {
            sql.push_str(if nulls_first {
                " NULLS FIRST"
            } else {
                " NULLS LAST"
            });
        }

        sql
    }
}

/// Aggregate function
#[derive(Debug, Clone)]
pub enum Aggregate {
    Count { column: Option<String> },
    Sum { column: String },
    Avg { column: String },
    Min { column: String },
    Max { column: String },
}

impl Aggregate {
    pub fn to_sql(&self) -> String {
        match self {
            Aggregate::Count { column } => {
                if let Some(col) = column {
                    format!("COUNT({})", col)
                } else {
                    "COUNT(*)".to_string()
                }
            }
            Aggregate::Sum { column } => format!("SUM({})", column),
            Aggregate::Avg { column } => format!("AVG({})", column),
            Aggregate::Min { column } => format!("MIN({})", column),
            Aggregate::Max { column } => format!("MAX({})", column),
        }
    }
}

/// Query optimization hints
#[derive(Debug, Clone)]
pub enum QueryHint {
    /// Force index usage
    UseIndex { index_name: String },
    /// Force sequential scan
    SeqScan,
    /// Force index scan
    IndexScan,
    /// Set work memory for this query
    WorkMem { size_mb: u32 },
}

/// SQL Query representation
#[derive(Debug, Clone)]
pub struct Query {
    pub select_columns: Vec<String>,
    pub from_table: String,
    pub table_alias: Option<String>,
    pub joins: Vec<Join>,
    pub where_clauses: Vec<(LogicalOperator, WhereClause)>,
    pub group_by: Vec<String>,
    pub having_clauses: Vec<(LogicalOperator, WhereClause)>,
    pub order_by: Vec<OrderBy>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub aggregates: Vec<Aggregate>,
    pub hints: Vec<QueryHint>,
    pub distinct: bool,
    pub for_update: bool,
}

impl Query {
    /// Build SQL string
    pub fn to_sql(&self) -> Result<String, QueryError> {
        let mut sql = String::new();

        // SELECT clause
        sql.push_str("SELECT ");

        if self.distinct {
            sql.push_str("DISTINCT ");
        }

        if self.select_columns.is_empty() && self.aggregates.is_empty() {
            sql.push('*');
        } else {
            let mut columns = self.select_columns.clone();
            for agg in &self.aggregates {
                columns.push(agg.to_sql());
            }
            sql.push_str(&columns.join(", "));
        }

        // FROM clause
        sql.push_str(" FROM ");
        sql.push_str(&self.from_table);

        if let Some(alias) = &self.table_alias {
            sql.push_str(" AS ");
            sql.push_str(alias);
        }

        // JOIN clauses
        for join in &self.joins {
            sql.push(' ');
            sql.push_str(join.join_type.to_sql());
            sql.push(' ');
            sql.push_str(&join.table);

            if let Some(alias) = &join.alias {
                sql.push_str(" AS ");
                sql.push_str(alias);
            }

            if join.join_type != JoinType::Cross {
                sql.push_str(" ON ");
                sql.push_str(&join.on_condition);
            }
        }

        // WHERE clause
        if !self.where_clauses.is_empty() {
            sql.push_str(" WHERE ");

            for (i, (logical_op, clause)) in self.where_clauses.iter().enumerate() {
                if i > 0 {
                    sql.push(' ');
                    sql.push_str(logical_op.to_sql());
                    sql.push(' ');
                }
                sql.push_str(&clause.to_sql());
            }
        }

        // GROUP BY clause
        if !self.group_by.is_empty() {
            sql.push_str(" GROUP BY ");
            sql.push_str(&self.group_by.join(", "));
        }

        // HAVING clause
        if !self.having_clauses.is_empty() {
            sql.push_str(" HAVING ");

            for (i, (logical_op, clause)) in self.having_clauses.iter().enumerate() {
                if i > 0 {
                    sql.push(' ');
                    sql.push_str(logical_op.to_sql());
                    sql.push(' ');
                }
                sql.push_str(&clause.to_sql());
            }
        }

        // ORDER BY clause
        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            let order_clauses: Vec<String> = self.order_by.iter().map(|o| o.to_sql()).collect();
            sql.push_str(&order_clauses.join(", "));
        }

        // LIMIT clause
        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        // OFFSET clause
        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        // FOR UPDATE clause
        if self.for_update {
            sql.push_str(" FOR UPDATE");
        }

        Ok(sql)
    }
}

/// Query builder for fluent query construction
pub struct QueryBuilder {
    query: Query,
}

impl QueryBuilder {
    /// Create a new query builder
    pub fn new() -> Self {
        Self {
            query: Query {
                select_columns: Vec::new(),
                from_table: String::new(),
                table_alias: None,
                joins: Vec::new(),
                where_clauses: Vec::new(),
                group_by: Vec::new(),
                having_clauses: Vec::new(),
                order_by: Vec::new(),
                limit: None,
                offset: None,
                aggregates: Vec::new(),
                hints: Vec::new(),
                distinct: false,
                for_update: false,
            },
        }
    }

    /// Select specific columns
    pub fn select(mut self, columns: &[&str]) -> Self {
        self.query.select_columns = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Select all columns
    pub fn select_all(self) -> Self {
        self
    }

    /// Select with DISTINCT
    pub fn distinct(mut self) -> Self {
        self.query.distinct = true;
        self
    }

    /// FROM clause
    pub fn from(mut self, table: impl Into<String>) -> Self {
        self.query.from_table = table.into();
        self
    }

    /// Table alias
    pub fn alias(mut self, alias: impl Into<String>) -> Self {
        self.query.table_alias = Some(alias.into());
        self
    }

    /// Add a JOIN
    pub fn join(
        mut self,
        join_type: JoinType,
        table: impl Into<String>,
        on_condition: impl Into<String>,
    ) -> Self {
        self.query.joins.push(Join {
            join_type,
            table: table.into(),
            alias: None,
            on_condition: on_condition.into(),
        });
        self
    }

    /// Add INNER JOIN
    pub fn inner_join(
        self,
        table: impl Into<String>,
        on_condition: impl Into<String>,
    ) -> Self {
        self.join(JoinType::Inner, table, on_condition)
    }

    /// Add LEFT JOIN
    pub fn left_join(self, table: impl Into<String>, on_condition: impl Into<String>) -> Self {
        self.join(JoinType::Left, table, on_condition)
    }

    /// Add WHERE clause with AND
    pub fn where_and(mut self, clause: WhereClause) -> Self {
        self.query
            .where_clauses
            .push((LogicalOperator::And, clause));
        self
    }

    /// Add WHERE clause with OR
    pub fn where_or(mut self, clause: WhereClause) -> Self {
        self.query
            .where_clauses
            .push((LogicalOperator::Or, clause));
        self
    }

    /// Add WHERE equals condition
    pub fn where_eq(self, column: impl Into<String>, value: Value) -> Self {
        self.where_and(WhereClause::new(column, Operator::Equals, value))
    }

    /// Add WHERE IN condition
    pub fn where_in(self, column: impl Into<String>, values: Vec<Value>) -> Self {
        self.where_and(WhereClause::new(
            column,
            Operator::In,
            Value::Array(values),
        ))
    }

    /// Add GROUP BY
    pub fn group_by(mut self, columns: &[&str]) -> Self {
        self.query.group_by = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add HAVING clause
    pub fn having(mut self, clause: WhereClause) -> Self {
        self.query
            .having_clauses
            .push((LogicalOperator::And, clause));
        self
    }

    /// Add ORDER BY
    pub fn order_by(mut self, column: impl Into<String>, direction: OrderDirection) -> Self {
        self.query.order_by.push(OrderBy {
            column: column.into(),
            direction,
            nulls_first: None,
        });
        self
    }

    /// Add ORDER BY ASC
    pub fn order_asc(self, column: impl Into<String>) -> Self {
        self.order_by(column, OrderDirection::Asc)
    }

    /// Add ORDER BY DESC
    pub fn order_desc(self, column: impl Into<String>) -> Self {
        self.order_by(column, OrderDirection::Desc)
    }

    /// Set LIMIT
    pub fn limit(mut self, limit: u64) -> Self {
        self.query.limit = Some(limit);
        self
    }

    /// Set OFFSET
    pub fn offset(mut self, offset: u64) -> Self {
        self.query.offset = Some(offset);
        self
    }

    /// Add pagination
    pub fn paginate(self, page: u64, page_size: u64) -> Self {
        let offset = (page - 1) * page_size;
        self.limit(page_size).offset(offset)
    }

    /// Add COUNT aggregate
    pub fn count(mut self, column: Option<String>) -> Self {
        self.query.aggregates.push(Aggregate::Count { column });
        self
    }

    /// Add SUM aggregate
    pub fn sum(mut self, column: impl Into<String>) -> Self {
        self.query.aggregates.push(Aggregate::Sum {
            column: column.into(),
        });
        self
    }

    /// Add FOR UPDATE
    pub fn for_update(mut self) -> Self {
        self.query.for_update = true;
        self
    }

    /// Add query hint
    pub fn hint(mut self, hint: QueryHint) -> Self {
        self.query.hints.push(hint);
        self
    }

    /// Build the final query
    pub fn build(self) -> Result<Query, QueryError> {
        if self.query.from_table.is_empty() {
            return Err(QueryError::MissingField("FROM table".to_string()));
        }

        Ok(self.query)
    }

    /// Build and generate SQL
    pub fn to_sql(self) -> Result<String, QueryError> {
        let query = self.build()?;
        query.to_sql()
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_select() {
        let sql = QueryBuilder::new()
            .select(&["id", "name", "email"])
            .from("users")
            .to_sql()
            .unwrap();

        assert!(sql.contains("SELECT id, name, email"));
        assert!(sql.contains("FROM users"));
    }

    #[test]
    fn test_where_clause() {
        let sql = QueryBuilder::new()
            .select_all()
            .from("users")
            .where_eq("id", Value::Integer(1))
            .to_sql()
            .unwrap();

        assert!(sql.contains("WHERE id = 1"));
    }

    #[test]
    fn test_pagination() {
        let sql = QueryBuilder::new()
            .select_all()
            .from("users")
            .paginate(2, 10)
            .to_sql()
            .unwrap();

        assert!(sql.contains("LIMIT 10"));
        assert!(sql.contains("OFFSET 10"));
    }

    #[test]
    fn test_join() {
        let sql = QueryBuilder::new()
            .select(&["users.name", "projects.title"])
            .from("users")
            .inner_join("projects", "users.id = projects.owner_id")
            .to_sql()
            .unwrap();

        assert!(sql.contains("INNER JOIN projects"));
        assert!(sql.contains("ON users.id = projects.owner_id"));
    }

    #[test]
    fn test_order_by() {
        let sql = QueryBuilder::new()
            .select_all()
            .from("users")
            .order_desc("created_at")
            .to_sql()
            .unwrap();

        assert!(sql.contains("ORDER BY created_at DESC"));
    }
}
