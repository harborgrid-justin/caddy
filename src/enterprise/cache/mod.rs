//! # Enterprise Distributed Cache System
//!
//! A comprehensive, high-performance distributed caching system for CADDY v0.2.0
//! providing multi-tier caching, sophisticated invalidation strategies, distributed
//! locking, and efficient serialization.
//!
//! ## Features
//!
//! ### Multi-Tier Caching (`tier`)
//!
//! Three-level caching hierarchy optimized for different access patterns:
//!
//! - **L1 (In-Memory)**: Ultra-fast local cache with LRU eviction
//!   - Typical latency: ~10-100 nanoseconds
//!   - Capacity: 1,000s of entries
//!   - Use for: Frequently accessed data, hot paths
//!
//! - **L2 (Shared Memory)**: Cross-process cache for local coordination
//!   - Typical latency: ~1-10 microseconds
//!   - Capacity: 10,000s of entries
//!   - Use for: Shared data between processes, warm data
//!
//! - **L3 (Distributed)**: Redis/Memcached cluster for global state
//!   - Typical latency: ~1-10 milliseconds
//!   - Capacity: Millions of entries
//!   - Use for: Cold data, cross-machine coordination
//!
//! The system automatically promotes hot data to faster tiers and demotes
//! cold data to slower tiers based on access patterns.
//!
//! ### Cache Strategies (`strategy`)
//!
//! Multiple strategies for different consistency and performance requirements:
//!
//! - **Write-Through**: Synchronous writes to cache and backing store
//!   - Guarantees consistency
//!   - Higher write latency
//!   - Use for: Critical data requiring strong consistency
//!
//! - **Write-Behind**: Asynchronous writes with eventual consistency
//!   - Lower write latency
//!   - Better write throughput
//!   - Use for: High-volume writes, analytics data
//!
//! - **Write-Around**: Bypass cache on writes, invalidate if needed
//!   - Prevents cache pollution
//!   - Use for: Large objects, infrequently read data
//!
//! - **Read-Through**: Automatic cache population on miss
//!   - Simplifies application code
//!   - Use for: Read-heavy workloads
//!
//! - **Cache-Aside**: Manual cache management
//!   - Maximum control
//!   - Use for: Complex caching logic
//!
//! - **Refresh-Ahead**: Proactive refresh before expiration
//!   - Prevents cache stampede
//!   - Use for: Expensive computations, hot keys
//!
//! ### Distributed Locking (`lock`)
//!
//! Coordination primitives for distributed systems:
//!
//! - **Distributed Mutex**: Exclusive locks with fencing tokens
//!   - Prevents split-brain scenarios
//!   - Automatic lease renewal
//!   - Deadlock detection
//!
//! - **Read-Write Locks**: Multiple readers, single writer
//!   - Fair scheduling
//!   - Prevents writer starvation
//!
//! - **Lock Leasing**: Time-bound locks with auto-expiration
//!   - Prevents indefinite blocking
//!   - Configurable lease duration
//!
//! ### Invalidation Protocols (`invalidation`)
//!
//! Sophisticated cache invalidation for maintaining consistency:
//!
//! - **Tag-Based**: Group related entries for batch invalidation
//!   ```rust
//!   // Invalidate all cache entries for a user
//!   cache.invalidate_tag("user:123")?;
//!   ```
//!
//! - **Pattern-Based**: Wildcard matching for bulk operations
//!   ```rust
//!   // Invalidate all session keys
//!   cache.invalidate_pattern("session:*")?;
//!   ```
//!
//! - **Cascade**: Automatic invalidation of dependent entries
//!   ```rust
//!   // Invalidating a user invalidates their sessions, preferences, etc.
//!   cache.invalidate_cascade(&user_id)?;
//!   ```
//!
//! - **Pub/Sub**: Distributed change notifications
//!   - Event-driven invalidation
//!   - Cross-instance coordination
//!
//! ### Serialization & Compression (`codec`)
//!
//! Efficient encoding/decoding with multiple formats:
//!
//! - **Binary Serialization**: Compact bincode format
//! - **Compression**: LZ4 (fast) or ZSTD (high ratio)
//! - **Schema Versioning**: Backward compatibility support
//! - **Checksum Validation**: Detect corruption
//!
//! ## Quick Start
//!
//! ```rust
//! use caddy::enterprise::cache::{
//!     tier::{MultiTierCache, TierConfig},
//!     strategy::{WriteThroughCache, InMemoryStore},
//!     lock::DistributedMutex,
//!     invalidation::TagInvalidator,
//! };
//!
//! // Create a multi-tier cache
//! let cache = MultiTierCache::<String, Vec<u8>>::new();
//!
//! // Insert data (starts at L3, promotes on access)
//! cache.insert("key1".to_string(), vec![1, 2, 3], None).await;
//!
//! // Access data (automatic promotion)
//! let value = cache.get(&"key1".to_string()).await;
//!
//! // Create a write-through cache with backing store
//! let store = InMemoryStore::new();
//! let wt_cache = WriteThroughCache::new(store);
//!
//! wt_cache.put(1, "value".to_string(), None).await?;
//! let value = wt_cache.get(&1).await?;
//!
//! // Use distributed locking
//! let mutex = DistributedMutex::new();
//! let owner = uuid::Uuid::new_v4();
//!
//! let token = mutex.lock(resource_id, owner, None).await?;
//! // ... critical section ...
//! mutex.unlock(&resource_id, owner, token).await?;
//!
//! // Tag-based invalidation
//! let invalidator = TagInvalidator::new();
//! let tags = std::collections::HashSet::from(["user:123".to_string()]);
//!
//! invalidator.insert(1, "data".to_string(), tags);
//! invalidator.invalidate_tag("user:123")?;
//! ```
//!
//! ## Performance Characteristics
//!
//! ### Latency
//!
//! | Operation | L1 | L2 | L3 |
//! |-----------|----|----|-----|
//! | Read (hit) | 10-100 ns | 1-10 μs | 1-10 ms |
//! | Write | 50-200 ns | 5-20 μs | 2-20 ms |
//! | Eviction | 100-500 ns | 10-50 μs | N/A |
//!
//! ### Throughput
//!
//! - L1: ~10M ops/sec (single-threaded)
//! - L2: ~1M ops/sec (multi-process)
//! - L3: ~100K ops/sec (distributed)
//!
//! ### Memory Efficiency
//!
//! - Metadata overhead: ~64 bytes per entry
//! - Compression ratio: 2-5x (ZSTD), 1.5-3x (LZ4)
//! - Index overhead: ~16 bytes per tag/pattern
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │          Application Layer                      │
//! └─────────────────────────────────────────────────┘
//!                       │
//!                       ▼
//! ┌─────────────────────────────────────────────────┐
//! │     L1: In-Memory Cache (LRU)                   │
//! │     - DashMap for concurrent access             │
//! │     - LRU eviction policy                       │
//! │     - Automatic promotion from L2               │
//! └─────────────────────────────────────────────────┘
//!                       │ (miss)
//!                       ▼
//! ┌─────────────────────────────────────────────────┐
//! │     L2: Shared Memory Cache                     │
//! │     - Cross-process coordination                │
//! │     - Promotion to L1 on hot access             │
//! │     - Demotion to L3 on cold access             │
//! └─────────────────────────────────────────────────┘
//!                       │ (miss)
//!                       ▼
//! ┌─────────────────────────────────────────────────┐
//! │     L3: Distributed Cache (Redis/Memcached)     │
//! │     - Cluster support                           │
//! │     - Persistence options                       │
//! │     - Global coordination                       │
//! └─────────────────────────────────────────────────┘
//!                       │ (miss)
//!                       ▼
//! ┌─────────────────────────────────────────────────┐
//! │     Backing Store (Database, File, etc.)        │
//! └─────────────────────────────────────────────────┘
//! ```
//!
//! ## Best Practices
//!
//! ### Choosing Cache Tiers
//!
//! 1. **L1 for**:
//!    - Current user's active data
//!    - Frequently accessed configuration
//!    - Hot computation results
//!
//! 2. **L2 for**:
//!    - Shared session data
//!    - Inter-process communication
//!    - Warm dataset
//!
//! 3. **L3 for**:
//!    - User profiles
//!    - Global configuration
//!    - Cold data with occasional access
//!
//! ### Choosing Cache Strategies
//!
//! 1. **Write-Through when**:
//!    - Consistency is critical
//!    - Write volume is moderate
//!    - Data must be durable immediately
//!
//! 2. **Write-Behind when**:
//!    - Write throughput is critical
//!    - Eventual consistency is acceptable
//!    - Analytics or logging data
//!
//! 3. **Refresh-Ahead when**:
//!    - Computing data is expensive
//!    - Cache stampede is a concern
//!    - Predictable access patterns
//!
//! ### Invalidation Strategies
//!
//! 1. **Use tags for**:
//!    - Related entities (user data, session data)
//!    - Feature flags
//!    - Multi-tenant data
//!
//! 2. **Use patterns for**:
//!    - Time-based data (sessions:2024-*)
//!    - Hierarchical data (user:123:*)
//!    - Bulk operations
//!
//! 3. **Use cascade for**:
//!    - Complex dependencies
//!    - Derived data
//!    - Computed views
//!
//! ## Configuration Guidelines
//!
//! ### Sizing
//!
//! ```rust
//! use caddy::enterprise::cache::tier::TierConfig;
//!
//! let config = TierConfig {
//!     l1_capacity: 1_000,        // Active working set
//!     l2_capacity: 10_000,       // 10x L1
//!     l3_capacity: 100_000,      // 100x L1
//!     promotion_threshold: 10,   // Promote after 10 hits
//!     demotion_threshold_secs: 300, // Demote after 5 min idle
//!     enable_auto_tiering: true,
//!     maintenance_interval_secs: 60,
//! };
//! ```
//!
//! ### Tuning
//!
//! - **High read/write ratio**: Increase L1 capacity, use refresh-ahead
//! - **High write volume**: Use write-behind, larger batches
//! - **Memory constrained**: Aggressive eviction, enable compression
//! - **Latency sensitive**: Larger L1, disable compression
//!
//! ## Monitoring
//!
//! ```rust
//! // Get cache statistics
//! let stats = cache.get_stats();
//! println!("L1 hits: {}", stats.get("l1_hits").unwrap_or(&0));
//! println!("Hit rate: {:.2}%", cache.hit_rate() * 100.0);
//!
//! // Get tier sizes
//! let (l1_size, l2_size, l3_size) = cache.tier_sizes();
//! println!("Tier sizes: L1={}, L2={}, L3={}", l1_size, l2_size, l3_size);
//! ```
//!
//! ## Thread Safety
//!
//! All cache implementations are thread-safe and use lock-free data structures
//! where possible (DashMap). Distributed locks provide cross-process and
//! cross-machine coordination.
//!
//! ## Async Support
//!
//! All I/O operations are async and built on tokio. This allows for efficient
//! handling of high-concurrency workloads without blocking threads.
//!
//! ## Error Handling
//!
//! All operations return `EnterpriseResult<T>` which wraps the standard
//! `EnterpriseError` type. Errors are categorized and include context for
//! debugging.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![allow(clippy::module_name_repetitions)]

/// Multi-tier caching with automatic promotion and demotion
///
/// Provides L1 (in-memory), L2 (shared memory), and L3 (distributed) caching
/// tiers with automatic data movement based on access patterns.
///
/// # Examples
///
/// ```rust
/// use caddy::enterprise::cache::tier::{MultiTierCache, TierConfig};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create cache with default configuration
/// let cache = MultiTierCache::<String, Vec<u8>>::new();
///
/// // Insert data
/// cache.insert("key1".to_string(), vec![1, 2, 3], None).await;
///
/// // Access data (automatic promotion)
/// let value = cache.get(&"key1".to_string()).await;
/// assert_eq!(value, Some(vec![1, 2, 3]));
///
/// // Insert hot data directly in L1
/// cache.insert_hot("hot_key".to_string(), vec![4, 5, 6], None).await;
/// # Ok(())
/// # }
/// ```
pub mod tier;

/// Cache strategies for different consistency requirements
///
/// Provides write-through, write-behind, write-around, read-through,
/// cache-aside, and refresh-ahead strategies.
///
/// # Examples
///
/// ```rust
/// use caddy::enterprise::cache::strategy::{WriteThroughCache, InMemoryStore};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a write-through cache
/// let store = InMemoryStore::new();
/// let cache = WriteThroughCache::new(store);
///
/// // Write to cache and store synchronously
/// cache.put(1, "value".to_string(), None).await?;
///
/// // Read from cache
/// let value = cache.get(&1).await?;
/// assert_eq!(value, Some("value".to_string()));
/// # Ok(())
/// # }
/// ```
pub mod strategy;

/// Distributed locking mechanisms
///
/// Provides distributed mutex with fencing tokens, read-write locks,
/// lock leasing, and deadlock detection.
///
/// # Examples
///
/// ```rust
/// use caddy::enterprise::cache::lock::DistributedMutex;
/// use uuid::Uuid;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mutex = DistributedMutex::new();
/// let owner = Uuid::new_v4();
///
/// // Acquire lock
/// let token = mutex.lock(1, owner, None).await?;
///
/// // Critical section
/// // ...
///
/// // Release lock
/// mutex.unlock(&1, owner, token).await?;
/// # Ok(())
/// # }
/// ```
pub mod lock;

/// Cache invalidation protocols
///
/// Provides tag-based, pattern-based, cascade, and pub/sub invalidation
/// strategies for maintaining cache consistency.
///
/// # Examples
///
/// ```rust
/// use caddy::enterprise::cache::invalidation::TagInvalidator;
/// use std::collections::HashSet;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let invalidator = TagInvalidator::new();
///
/// // Insert with tags
/// let tags = HashSet::from(["user:123".to_string(), "active".to_string()]);
/// invalidator.insert(1, "data".to_string(), tags);
///
/// // Invalidate all entries for user
/// let count = invalidator.invalidate_tag("user:123")?;
/// println!("Invalidated {} entries", count);
/// # Ok(())
/// # }
/// ```
pub mod invalidation;

/// Serialization and compression codecs
///
/// Provides binary serialization, compression (LZ4, ZSTD), schema versioning,
/// and checksum validation.
///
/// # Examples
///
/// ```rust
/// use caddy::enterprise::cache::codec::{BincodeCodec, CodecConfig, CompressionAlgorithm};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, PartialEq, Debug)]
/// struct Data {
///     id: u64,
///     name: String,
/// }
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create codec with compression
/// let mut config = CodecConfig::default();
/// config.compression = CompressionAlgorithm::Lz4;
///
/// let codec = BincodeCodec::<Data>::with_config(config);
///
/// // Encode
/// let data = Data { id: 1, name: "test".to_string() };
/// let encoded = codec.encode(&data)?;
///
/// println!("Compression ratio: {:.2}", encoded.compression_ratio());
///
/// // Decode
/// let decoded = codec.decode(&encoded)?;
/// assert_eq!(data, decoded);
/// # Ok(())
/// # }
/// ```
pub mod codec;

// Re-export commonly used types for convenience
pub use tier::{CacheTier, MultiTierCache, TierConfig};
pub use strategy::{
    BackingStore, InMemoryStore, ReadThroughCache, RefreshAheadCache,
    StrategyConfig, StrategyType, WriteBehindCache, WriteThroughCache,
};
pub use lock::{
    DeadlockDetector, DistributedMutex, DistributedRwLock, FencingToken,
    LockConfig, LockMode, LockStatus,
};
pub use invalidation::{
    CascadeInvalidator, InvalidationEvent, InvalidationReason,
    PatternInvalidator, PubSubInvalidator, TagInvalidator,
};
pub use codec::{
    BincodeCodec, CodecConfig, CompressionAlgorithm, EncodedData,
    TrackedCodec, VersionedCodec,
};

/// Cache module version
pub const CACHE_VERSION: &str = "0.2.0";

/// Cache module build date
pub const CACHE_BUILD_DATE: &str = "2025-12-28";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constants() {
        assert_eq!(CACHE_VERSION, "0.2.0");
        assert!(!CACHE_BUILD_DATE.is_empty());
    }
}
