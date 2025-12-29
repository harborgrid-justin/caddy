//! # Query Optimizer for CAD Data Patterns
//!
//! Provides intelligent query optimization specifically designed for CAD workloads:
//! - Spatial query optimization using indexes
//! - Batch query optimization for bulk operations
//! - Query plan caching and analysis
//! - Statistics-based optimization hints

use crate::database::Result;
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Query optimization hint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OptimizationHint {
    /// Use spatial index for this query
    UseSpatialIndex,

    /// Use covering index (all columns in index)
    UseCoveringIndex(String),

    /// Force index usage
    ForceIndex(String),

    /// Prefer index scan over table scan
    PreferIndexScan,

    /// Use batch processing for this query
    UseBatch(usize),

    /// Cache this query result
    CacheResult(Duration),

    /// Parallelize this query across shards
    Parallelize,

    /// Read from replica instead of primary
    ReadFromReplica,
}

/// Query execution plan
#[derive(Debug, Clone)]
pub struct QueryPlan {
    /// Original SQL query
    pub query: String,

    /// Optimized SQL query
    pub optimized_query: String,

    /// Estimated cost (lower is better)
    pub estimated_cost: f64,

    /// Estimated number of rows
    pub estimated_rows: u64,

    /// Optimization hints applied
    pub hints: Vec<OptimizationHint>,

    /// Index suggestions
    pub index_suggestions: Vec<String>,

    /// Whether this query can use spatial index
    pub uses_spatial_index: bool,

    /// Whether this query can be cached
    pub cacheable: bool,

    /// Execution strategy
    pub strategy: ExecutionStrategy,
}

/// Query execution strategy
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionStrategy {
    /// Direct execution (default)
    Direct,

    /// Batch execution
    Batch { batch_size: usize },

    /// Parallel execution across shards
    Parallel { shard_count: usize },

    /// Use materialized view
    MaterializedView { view_name: String },

    /// Use cached result
    Cached { ttl: Duration },
}

/// Query statistics for optimization
#[derive(Debug, Clone, Default)]
pub struct QueryStats {
    /// Number of times this query was executed
    pub execution_count: u64,

    /// Average execution time
    pub avg_execution_time: Duration,

    /// Total execution time
    pub total_execution_time: Duration,

    /// Last execution time
    pub last_execution: Option<Instant>,

    /// Number of rows returned (average)
    pub avg_rows_returned: f64,

    /// Cache hit rate
    pub cache_hit_rate: f64,
}

/// Query optimizer
pub struct QueryOptimizer {
    /// Query plan cache (query hash -> plan)
    plan_cache: Arc<DashMap<u64, QueryPlan>>,

    /// Query statistics (query hash -> stats)
    query_stats: Arc<DashMap<u64, RwLock<QueryStats>>>,

    /// Table statistics (table name -> row count, etc.)
    table_stats: Arc<RwLock<HashMap<String, TableStats>>>,

    /// Configuration
    config: OptimizerConfig,
}

/// Table statistics
#[derive(Debug, Clone)]
pub struct TableStats {
    /// Total number of rows
    pub row_count: u64,

    /// Average row size in bytes
    pub avg_row_size: u64,

    /// Available indexes
    pub indexes: Vec<IndexInfo>,

    /// Data distribution statistics
    pub cardinality: HashMap<String, u64>,

    /// Last updated
    pub last_updated: Instant,
}

/// Index information
#[derive(Debug, Clone)]
pub struct IndexInfo {
    /// Index name
    pub name: String,

    /// Indexed columns
    pub columns: Vec<String>,

    /// Index type (btree, spatial, etc.)
    pub index_type: String,

    /// Whether this is a unique index
    pub is_unique: bool,

    /// Estimated selectivity (0.0 - 1.0)
    pub selectivity: f64,
}

/// Optimizer configuration
#[derive(Debug, Clone)]
pub struct OptimizerConfig {
    /// Enable query plan caching
    pub enable_plan_caching: bool,

    /// Maximum number of cached plans
    pub max_cached_plans: usize,

    /// Enable statistics collection
    pub enable_statistics: bool,

    /// Statistics update interval
    pub stats_update_interval: Duration,

    /// Batch size threshold for batch execution
    pub batch_threshold: usize,

    /// Enable spatial optimization
    pub enable_spatial_optimization: bool,

    /// Enable query result caching
    pub enable_result_caching: bool,

    /// Default cache TTL
    pub default_cache_ttl: Duration,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            enable_plan_caching: true,
            max_cached_plans: 10000,
            enable_statistics: true,
            stats_update_interval: Duration::from_secs(300),
            batch_threshold: 1000,
            enable_spatial_optimization: true,
            enable_result_caching: true,
            default_cache_ttl: Duration::from_secs(300),
        }
    }
}

impl QueryOptimizer {
    /// Create a new query optimizer
    pub fn new() -> Self {
        Self::with_config(OptimizerConfig::default())
    }

    /// Create a new query optimizer with configuration
    pub fn with_config(config: OptimizerConfig) -> Self {
        Self {
            plan_cache: Arc::new(DashMap::new()),
            query_stats: Arc::new(DashMap::new()),
            table_stats: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Optimize a query
    pub fn optimize(&self, query: &str, hints: Vec<OptimizationHint>) -> Result<QueryPlan> {
        let query_hash = self.hash_query(query);

        // Check cache first
        if self.config.enable_plan_caching {
            if let Some(cached_plan) = self.plan_cache.get(&query_hash) {
                return Ok(cached_plan.clone());
            }
        }

        // Analyze query
        let plan = self.analyze_query(query, hints)?;

        // Cache the plan
        if self.config.enable_plan_caching {
            self.plan_cache.insert(query_hash, plan.clone());

            // Evict old plans if cache is full
            if self.plan_cache.len() > self.config.max_cached_plans {
                // Simple LRU: remove first entry
                if let Some(entry) = self.plan_cache.iter().next() {
                    let key = *entry.key();
                    drop(entry);
                    self.plan_cache.remove(&key);
                }
            }
        }

        Ok(plan)
    }

    /// Analyze a query and create an execution plan
    fn analyze_query(&self, query: &str, hints: Vec<OptimizationHint>) -> Result<QueryPlan> {
        let mut optimized_query = query.to_string();
        let mut estimated_cost = 100.0; // Base cost
        let mut uses_spatial_index = false;
        let mut cacheable = true;
        let mut strategy = ExecutionStrategy::Direct;
        let mut index_suggestions = Vec::new();

        // Apply optimization hints
        for hint in &hints {
            match hint {
                OptimizationHint::UseSpatialIndex => {
                    uses_spatial_index = true;
                    estimated_cost *= 0.3; // Spatial index significantly reduces cost
                    index_suggestions.push("spatial_index".to_string());
                }
                OptimizationHint::UseCoveringIndex(index_name) => {
                    estimated_cost *= 0.5;
                    optimized_query = format!("{} -- USE INDEX ({})", optimized_query, index_name);
                }
                OptimizationHint::ForceIndex(index_name) => {
                    estimated_cost *= 0.6;
                    optimized_query = format!("{} -- FORCE INDEX ({})", optimized_query, index_name);
                }
                OptimizationHint::PreferIndexScan => {
                    estimated_cost *= 0.7;
                }
                OptimizationHint::UseBatch(size) => {
                    strategy = ExecutionStrategy::Batch { batch_size: *size };
                    estimated_cost *= 0.8;
                }
                OptimizationHint::CacheResult(ttl) => {
                    strategy = ExecutionStrategy::Cached { ttl: *ttl };
                    cacheable = true;
                }
                OptimizationHint::Parallelize => {
                    strategy = ExecutionStrategy::Parallel { shard_count: 4 };
                    estimated_cost *= 0.4;
                }
                OptimizationHint::ReadFromReplica => {
                    estimated_cost *= 0.9;
                }
            }
        }

        // Detect CAD-specific patterns
        if self.is_spatial_query(query) && self.config.enable_spatial_optimization {
            uses_spatial_index = true;
            estimated_cost *= 0.4;
            if !hints.contains(&OptimizationHint::UseSpatialIndex) {
                index_suggestions.push("Consider adding a spatial index".to_string());
            }
        }

        // Detect batch operations
        if query.to_uppercase().contains("INSERT") && query.contains("VALUES") {
            let value_count = query.matches("VALUES").count();
            if value_count > self.config.batch_threshold {
                strategy = ExecutionStrategy::Batch {
                    batch_size: self.config.batch_threshold,
                };
            }
        }

        // Detect cacheable queries (SELECT without volatile functions)
        if query.to_uppercase().trim().starts_with("SELECT") {
            cacheable = !query.to_uppercase().contains("RANDOM")
                && !query.to_uppercase().contains("NOW()")
                && !query.to_uppercase().contains("CURRENT_TIMESTAMP");
        } else {
            cacheable = false;
        }

        Ok(QueryPlan {
            query: query.to_string(),
            optimized_query,
            estimated_cost,
            estimated_rows: 1000, // Default estimate
            hints,
            index_suggestions,
            uses_spatial_index,
            cacheable,
            strategy,
        })
    }

    /// Check if a query is a spatial query
    fn is_spatial_query(&self, query: &str) -> bool {
        let upper = query.to_uppercase();
        upper.contains("ST_") // PostGIS functions
            || upper.contains("GEOMETRY")
            || upper.contains("POINT")
            || upper.contains("LINESTRING")
            || upper.contains("POLYGON")
            || upper.contains("BBOX")
            || upper.contains("INTERSECT")
            || upper.contains("WITHIN")
            || upper.contains("DISTANCE")
    }

    /// Hash a query for caching
    fn hash_query(&self, query: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        query.hash(&mut hasher);
        hasher.finish()
    }

    /// Record query execution statistics
    pub fn record_execution(
        &self,
        query: &str,
        execution_time: Duration,
        rows_returned: u64,
        cache_hit: bool,
    ) {
        if !self.config.enable_statistics {
            return;
        }

        let query_hash = self.hash_query(query);

        self.query_stats
            .entry(query_hash)
            .or_insert_with(|| RwLock::new(QueryStats::default()))
            .write()
            .update(execution_time, rows_returned, cache_hit);
    }

    /// Update table statistics
    pub fn update_table_stats(&self, table_name: String, stats: TableStats) {
        self.table_stats.write().insert(table_name, stats);
    }

    /// Get query statistics
    pub fn get_query_stats(&self, query: &str) -> Option<QueryStats> {
        let query_hash = self.hash_query(query);
        self.query_stats
            .get(&query_hash)
            .map(|stats| stats.read().clone())
    }

    /// Clear the plan cache
    pub fn clear_cache(&self) {
        self.plan_cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            total_plans: self.plan_cache.len(),
            max_plans: self.config.max_cached_plans,
            hit_rate: self.calculate_cache_hit_rate(),
        }
    }

    /// Calculate cache hit rate
    fn calculate_cache_hit_rate(&self) -> f64 {
        let total_executions: u64 = self
            .query_stats
            .iter()
            .map(|entry| entry.value().read().execution_count)
            .sum();

        let total_queries = self.query_stats.len() as u64;

        if total_queries == 0 {
            return 0.0;
        }

        (total_executions as f64 - total_queries as f64) / total_executions as f64
    }
}

impl Default for QueryOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryStats {
    /// Update statistics with new execution data
    fn update(&mut self, execution_time: Duration, rows_returned: u64, cache_hit: bool) {
        self.execution_count += 1;
        self.total_execution_time += execution_time;
        self.avg_execution_time = self.total_execution_time / self.execution_count as u32;
        self.last_execution = Some(Instant::now());

        // Update average rows returned (exponential moving average)
        if self.execution_count == 1 {
            self.avg_rows_returned = rows_returned as f64;
        } else {
            self.avg_rows_returned = self.avg_rows_returned * 0.9 + rows_returned as f64 * 0.1;
        }

        // Update cache hit rate
        if cache_hit {
            let old_hits = self.cache_hit_rate * (self.execution_count - 1) as f64;
            self.cache_hit_rate = (old_hits + 1.0) / self.execution_count as f64;
        } else {
            let old_hits = self.cache_hit_rate * (self.execution_count - 1) as f64;
            self.cache_hit_rate = old_hits / self.execution_count as f64;
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Total number of cached plans
    pub total_plans: usize,

    /// Maximum number of plans
    pub max_plans: usize,

    /// Cache hit rate
    pub hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimizer_creation() {
        let optimizer = QueryOptimizer::new();
        assert!(optimizer.plan_cache.is_empty());
    }

    #[test]
    fn test_spatial_query_detection() {
        let optimizer = QueryOptimizer::new();
        assert!(optimizer.is_spatial_query("SELECT * FROM entities WHERE ST_Contains(geometry, point)"));
        assert!(!optimizer.is_spatial_query("SELECT * FROM entities WHERE id = 1"));
    }

    #[test]
    fn test_query_optimization() {
        let optimizer = QueryOptimizer::new();
        let plan = optimizer.optimize(
            "SELECT * FROM entities WHERE layer_id = 1",
            vec![OptimizationHint::PreferIndexScan],
        );
        assert!(plan.is_ok());

        let plan = plan.unwrap();
        assert_eq!(plan.hints.len(), 1);
    }

    #[test]
    fn test_query_stats() {
        let optimizer = QueryOptimizer::new();
        let query = "SELECT * FROM test";

        optimizer.record_execution(query, Duration::from_millis(10), 100, false);
        optimizer.record_execution(query, Duration::from_millis(20), 200, true);

        let stats = optimizer.get_query_stats(query).unwrap();
        assert_eq!(stats.execution_count, 2);
        assert!(stats.avg_execution_time.as_millis() > 0);
        assert!(stats.cache_hit_rate > 0.0);
    }
}
