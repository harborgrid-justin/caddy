//! Rate Limiting Algorithms
//!
//! This module provides various rate limiting algorithms including:
//! - Token bucket with burst support
//! - Leaky bucket for traffic smoothing
//! - Sliding window log for precise counting
//! - Sliding window counter for efficiency
//! - Generic Cell Rate Algorithm (GCRA) for network traffic

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Result type for rate limiting operations
pub type RateLimitResult<T> = Result<T, RateLimitError>;

/// Rate limiting errors
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    /// Rate limit exceeded
    #[error("Rate limit exceeded: retry after {0} seconds")]
    Exceeded(u64),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Rate limiting decision
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// Request allowed
    Allowed {
        /// Tokens/capacity remaining
        remaining: u64,
        /// Time until reset (seconds)
        reset_after: u64,
    },
    /// Request denied
    Denied {
        /// Time until retry allowed (seconds)
        retry_after: u64,
        /// Current limit
        limit: u64,
    },
}

impl Decision {
    /// Check if request is allowed
    pub fn is_allowed(&self) -> bool {
        matches!(self, Decision::Allowed { .. })
    }

    /// Get retry after duration if denied
    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            Decision::Denied { retry_after, .. } => Some(Duration::from_secs(*retry_after)),
            _ => None,
        }
    }

    /// Get remaining capacity
    pub fn remaining(&self) -> Option<u64> {
        match self {
            Decision::Allowed { remaining, .. } => Some(*remaining),
            _ => None,
        }
    }
}

// ============================================================================
// Token Bucket Algorithm
// ============================================================================

/// Token bucket rate limiter with burst support
///
/// The token bucket algorithm allows bursts of traffic up to the bucket capacity
/// while maintaining an average rate over time. Tokens are added at a constant rate
/// and consumed by requests.
#[derive(Debug)]
pub struct TokenBucket {
    /// Maximum number of tokens (burst capacity)
    capacity: u64,
    /// Current number of tokens (atomic for lock-free access)
    tokens: AtomicU64,
    /// Refill rate (tokens per second)
    refill_rate: f64,
    /// Last refill timestamp (nanoseconds)
    last_refill: AtomicU64,
}

impl TokenBucket {
    /// Create a new token bucket
    ///
    /// # Arguments
    /// * `capacity` - Maximum number of tokens (burst size)
    /// * `refill_rate` - Tokens added per second
    pub fn new(capacity: u64, refill_rate: f64) -> Self {
        let now = Self::now_nanos();
        Self {
            capacity,
            tokens: AtomicU64::new(capacity),
            refill_rate,
            last_refill: AtomicU64::new(now),
        }
    }

    /// Check if a request is allowed
    ///
    /// # Arguments
    /// * `tokens` - Number of tokens to consume (default: 1)
    pub fn check(&self, tokens: u64) -> Decision {
        // Refill tokens based on elapsed time
        self.refill();

        // Try to consume tokens
        loop {
            let current = self.tokens.load(Ordering::Acquire);

            if current >= tokens {
                // Try to consume tokens atomically
                if self.tokens.compare_exchange(
                    current,
                    current - tokens,
                    Ordering::Release,
                    Ordering::Acquire,
                ).is_ok() {
                    return Decision::Allowed {
                        remaining: current - tokens,
                        reset_after: self.time_until_refill(tokens),
                    };
                }
                // CAS failed, retry
            } else {
                // Not enough tokens
                let retry_after = self.time_until_refill(tokens - current);
                return Decision::Denied {
                    retry_after,
                    limit: self.capacity,
                };
            }
        }
    }

    /// Refill tokens based on elapsed time
    fn refill(&self) {
        let now = Self::now_nanos();
        let last = self.last_refill.load(Ordering::Acquire);

        let elapsed_nanos = now.saturating_sub(last);
        let elapsed_secs = elapsed_nanos as f64 / 1_000_000_000.0;
        let tokens_to_add = (elapsed_secs * self.refill_rate) as u64;

        if tokens_to_add > 0 {
            // Try to update last_refill timestamp
            if self.last_refill.compare_exchange(
                last,
                now,
                Ordering::Release,
                Ordering::Acquire,
            ).is_ok() {
                // Add tokens, capped at capacity
                loop {
                    let current = self.tokens.load(Ordering::Acquire);
                    let new_tokens = std::cmp::min(current + tokens_to_add, self.capacity);

                    if self.tokens.compare_exchange(
                        current,
                        new_tokens,
                        Ordering::Release,
                        Ordering::Acquire,
                    ).is_ok() {
                        break;
                    }
                }
            }
        }
    }

    /// Calculate time until enough tokens are available
    fn time_until_refill(&self, needed_tokens: u64) -> u64 {
        if needed_tokens == 0 || self.refill_rate == 0.0 {
            return 0;
        }
        ((needed_tokens as f64 / self.refill_rate).ceil() as u64).max(1)
    }

    /// Get current nanoseconds since UNIX epoch
    fn now_nanos() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }

    /// Get current token count
    pub fn available_tokens(&self) -> u64 {
        self.refill();
        self.tokens.load(Ordering::Acquire)
    }

    /// Reset the bucket to full capacity
    pub fn reset(&self) {
        self.tokens.store(self.capacity, Ordering::Release);
        self.last_refill.store(Self::now_nanos(), Ordering::Release);
    }
}

// ============================================================================
// Leaky Bucket Algorithm
// ============================================================================

/// Leaky bucket rate limiter for traffic smoothing
///
/// The leaky bucket algorithm enforces a constant rate by "leaking" requests
/// at a fixed rate. Unlike token bucket, it doesn't allow bursts.
#[derive(Debug)]
pub struct LeakyBucket {
    /// Maximum queue size
    capacity: u64,
    /// Leak rate (requests per second)
    leak_rate: f64,
    /// Request queue
    queue: RwLock<VecDeque<u64>>,
    /// Last leak timestamp (nanoseconds)
    last_leak: AtomicU64,
}

impl LeakyBucket {
    /// Create a new leaky bucket
    ///
    /// # Arguments
    /// * `capacity` - Maximum queue size
    /// * `leak_rate` - Requests processed per second
    pub fn new(capacity: u64, leak_rate: f64) -> Self {
        Self {
            capacity,
            leak_rate,
            queue: RwLock::new(VecDeque::new()),
            last_leak: AtomicU64::new(Self::now_nanos()),
        }
    }

    /// Check if a request is allowed
    pub fn check(&self) -> Decision {
        // Leak requests based on elapsed time
        self.leak();

        let mut queue = self.queue.write();

        if (queue.len() as u64) < self.capacity {
            // Add request to queue
            queue.push_back(Self::now_nanos());

            Decision::Allowed {
                remaining: self.capacity - queue.len() as u64,
                reset_after: self.time_until_leak(queue.len()),
            }
        } else {
            // Queue is full
            let retry_after = self.time_until_leak(1);
            Decision::Denied {
                retry_after,
                limit: self.capacity,
            }
        }
    }

    /// Leak requests based on elapsed time
    fn leak(&self) {
        let now = Self::now_nanos();
        let last = self.last_leak.swap(now, Ordering::AcqRel);

        let elapsed_nanos = now.saturating_sub(last);
        let elapsed_secs = elapsed_nanos as f64 / 1_000_000_000.0;
        let requests_to_leak = (elapsed_secs * self.leak_rate) as usize;

        if requests_to_leak > 0 {
            let mut queue = self.queue.write();
            for _ in 0..requests_to_leak.min(queue.len()) {
                queue.pop_front();
            }
        }
    }

    /// Calculate time until space available
    fn time_until_leak(&self, queue_size: usize) -> u64 {
        if queue_size == 0 || self.leak_rate == 0.0 {
            return 0;
        }
        ((queue_size as f64 / self.leak_rate).ceil() as u64).max(1)
    }

    /// Get current nanoseconds since UNIX epoch
    fn now_nanos() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }

    /// Get current queue size
    pub fn queue_size(&self) -> usize {
        self.leak();
        self.queue.read().len()
    }

    /// Reset the bucket
    pub fn reset(&self) {
        self.queue.write().clear();
        self.last_leak.store(Self::now_nanos(), Ordering::Release);
    }
}

// ============================================================================
// Sliding Window Log Algorithm
// ============================================================================

/// Sliding window log rate limiter for precise counting
///
/// Maintains a log of request timestamps and counts requests within
/// the sliding time window. Most accurate but memory-intensive.
#[derive(Debug)]
pub struct SlidingWindowLog {
    /// Maximum requests in window
    limit: u64,
    /// Window duration
    window: Duration,
    /// Request timestamps (nanoseconds)
    log: RwLock<VecDeque<u64>>,
}

impl SlidingWindowLog {
    /// Create a new sliding window log
    ///
    /// # Arguments
    /// * `limit` - Maximum requests in window
    /// * `window` - Time window duration
    pub fn new(limit: u64, window: Duration) -> Self {
        Self {
            limit,
            window,
            log: RwLock::new(VecDeque::new()),
        }
    }

    /// Check if a request is allowed
    pub fn check(&self) -> Decision {
        let now = Self::now_nanos();
        let window_start = now - self.window.as_nanos() as u64;

        let mut log = self.log.write();

        // Remove expired entries
        while let Some(&timestamp) = log.front() {
            if timestamp < window_start {
                log.pop_front();
            } else {
                break;
            }
        }

        let count = log.len() as u64;

        if count < self.limit {
            // Allow request
            log.push_back(now);

            Decision::Allowed {
                remaining: self.limit - count - 1,
                reset_after: self.window.as_secs(),
            }
        } else {
            // Calculate retry time (when oldest entry expires)
            let retry_after = if let Some(&oldest) = log.front() {
                let expires_at = oldest + self.window.as_nanos() as u64;
                ((expires_at.saturating_sub(now)) / 1_000_000_000).max(1)
            } else {
                1
            };

            Decision::Denied {
                retry_after,
                limit: self.limit,
            }
        }
    }

    /// Get current nanoseconds since UNIX epoch
    fn now_nanos() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }

    /// Get current request count in window
    pub fn current_count(&self) -> u64 {
        let now = Self::now_nanos();
        let window_start = now - self.window.as_nanos() as u64;

        let log = self.log.read();
        log.iter().filter(|&&ts| ts >= window_start).count() as u64
    }

    /// Reset the log
    pub fn reset(&self) {
        self.log.write().clear();
    }
}

// ============================================================================
// Sliding Window Counter Algorithm
// ============================================================================

/// Sliding window counter rate limiter (memory-efficient approximation)
///
/// Divides time into fixed windows and interpolates between current and
/// previous window. Less accurate than log but much more memory-efficient.
#[derive(Debug)]
pub struct SlidingWindowCounter {
    /// Maximum requests in window
    limit: u64,
    /// Window duration
    window: Duration,
    /// Current window start (nanoseconds)
    current_window: AtomicU64,
    /// Current window count
    current_count: AtomicU64,
    /// Previous window count
    previous_count: AtomicU64,
}

impl SlidingWindowCounter {
    /// Create a new sliding window counter
    ///
    /// # Arguments
    /// * `limit` - Maximum requests in window
    /// * `window` - Time window duration
    pub fn new(limit: u64, window: Duration) -> Self {
        let now = Self::now_nanos();
        let window_nanos = window.as_nanos() as u64;
        let current_window = (now / window_nanos) * window_nanos;

        Self {
            limit,
            window,
            current_window: AtomicU64::new(current_window),
            current_count: AtomicU64::new(0),
            previous_count: AtomicU64::new(0),
        }
    }

    /// Check if a request is allowed
    pub fn check(&self) -> Decision {
        let now = Self::now_nanos();
        let window_nanos = self.window.as_nanos() as u64;
        let window_start = (now / window_nanos) * window_nanos;

        // Check if we need to roll to new window
        let current_window = self.current_window.load(Ordering::Acquire);
        if window_start > current_window {
            // Roll to new window
            if self.current_window.compare_exchange(
                current_window,
                window_start,
                Ordering::Release,
                Ordering::Acquire,
            ).is_ok() {
                // Move current to previous
                let count = self.current_count.swap(0, Ordering::AcqRel);
                self.previous_count.store(count, Ordering::Release);
            }
        }

        // Calculate weighted count
        let elapsed_in_window = now - self.current_window.load(Ordering::Acquire);
        let window_progress = elapsed_in_window as f64 / window_nanos as f64;

        let current = self.current_count.load(Ordering::Acquire) as f64;
        let previous = self.previous_count.load(Ordering::Acquire) as f64;
        let weighted_count = (previous * (1.0 - window_progress) + current).ceil() as u64;

        if weighted_count < self.limit {
            // Allow request
            self.current_count.fetch_add(1, Ordering::AcqRel);

            Decision::Allowed {
                remaining: self.limit.saturating_sub(weighted_count + 1),
                reset_after: ((window_nanos - elapsed_in_window) / 1_000_000_000).max(1),
            }
        } else {
            // Deny request
            let retry_after = ((window_nanos - elapsed_in_window) / 1_000_000_000).max(1);
            Decision::Denied {
                retry_after,
                limit: self.limit,
            }
        }
    }

    /// Get current nanoseconds since UNIX epoch
    fn now_nanos() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }

    /// Get current weighted count
    pub fn current_count(&self) -> u64 {
        let now = Self::now_nanos();
        let window_nanos = self.window.as_nanos() as u64;
        let elapsed_in_window = now - self.current_window.load(Ordering::Acquire);
        let window_progress = elapsed_in_window as f64 / window_nanos as f64;

        let current = self.current_count.load(Ordering::Acquire) as f64;
        let previous = self.previous_count.load(Ordering::Acquire) as f64;
        (previous * (1.0 - window_progress) + current).ceil() as u64
    }

    /// Reset the counter
    pub fn reset(&self) {
        self.current_count.store(0, Ordering::Release);
        self.previous_count.store(0, Ordering::Release);

        let now = Self::now_nanos();
        let window_nanos = self.window.as_nanos() as u64;
        let window_start = (now / window_nanos) * window_nanos;
        self.current_window.store(window_start, Ordering::Release);
    }
}

// ============================================================================
// Generic Cell Rate Algorithm (GCRA)
// ============================================================================

/// Generic Cell Rate Algorithm (GCRA) for network traffic shaping
///
/// Also known as Virtual Scheduling Algorithm. Used in ATM networks
/// and provides smooth rate limiting with precise timing.
#[derive(Debug)]
pub struct GCRA {
    /// Time between requests (nanoseconds)
    interval: u64,
    /// Maximum burst size (nanoseconds)
    burst: u64,
    /// Theoretical arrival time (nanoseconds)
    tat: AtomicU64,
}

impl GCRA {
    /// Create a new GCRA limiter
    ///
    /// # Arguments
    /// * `rate` - Maximum requests per second
    /// * `burst` - Maximum burst size (requests)
    pub fn new(rate: f64, burst: u64) -> Self {
        let interval = (1_000_000_000.0 / rate) as u64;
        let burst_nanos = interval * burst;

        Self {
            interval,
            burst: burst_nanos,
            tat: AtomicU64::new(0),
        }
    }

    /// Check if a request is allowed
    pub fn check(&self) -> Decision {
        let now = Self::now_nanos();

        loop {
            let tat = self.tat.load(Ordering::Acquire);
            let allow_at = tat.saturating_sub(self.burst);

            if now >= allow_at {
                // Request is allowed
                let new_tat = std::cmp::max(tat, now) + self.interval;

                if self.tat.compare_exchange(
                    tat,
                    new_tat,
                    Ordering::Release,
                    Ordering::Acquire,
                ).is_ok() {
                    let remaining = self.burst.saturating_sub(new_tat.saturating_sub(now)) / self.interval;

                    return Decision::Allowed {
                        remaining,
                        reset_after: (self.burst / 1_000_000_000).max(1),
                    };
                }
                // CAS failed, retry
            } else {
                // Request is denied
                let retry_after = ((allow_at - now) / 1_000_000_000).max(1);
                let limit = self.burst / self.interval;

                return Decision::Denied {
                    retry_after,
                    limit,
                };
            }
        }
    }

    /// Get current nanoseconds since UNIX epoch
    fn now_nanos() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }

    /// Reset the algorithm
    pub fn reset(&self) {
        self.tat.store(0, Ordering::Release);
    }
}

// ============================================================================
// Algorithm Configuration
// ============================================================================

/// Rate limiting algorithm type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlgorithmType {
    /// Token bucket
    TokenBucket,
    /// Leaky bucket
    LeakyBucket,
    /// Sliding window log
    SlidingWindowLog,
    /// Sliding window counter
    SlidingWindowCounter,
    /// GCRA
    GCRA,
}

/// Rate limiting algorithm configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmConfig {
    /// Algorithm type
    pub algorithm: AlgorithmType,
    /// Rate limit (requests per second)
    pub rate: f64,
    /// Burst/capacity size
    pub burst: u64,
    /// Window duration (for sliding window algorithms)
    #[serde(default)]
    pub window_secs: u64,
}

impl Default for AlgorithmConfig {
    fn default() -> Self {
        Self {
            algorithm: AlgorithmType::TokenBucket,
            rate: 100.0,
            burst: 100,
            window_secs: 60,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_token_bucket_basic() {
        let bucket = TokenBucket::new(10, 1.0);

        // First 10 requests should succeed
        for _ in 0..10 {
            assert!(bucket.check(1).is_allowed());
        }

        // 11th request should fail
        assert!(!bucket.check(1).is_allowed());
    }

    #[test]
    fn test_token_bucket_refill() {
        let bucket = TokenBucket::new(1, 10.0); // 10 tokens/sec

        assert!(bucket.check(1).is_allowed());
        assert!(!bucket.check(1).is_allowed());

        // Wait for refill
        thread::sleep(Duration::from_millis(150));

        assert!(bucket.check(1).is_allowed());
    }

    #[test]
    fn test_leaky_bucket_basic() {
        let bucket = LeakyBucket::new(5, 1.0);

        // Fill the bucket
        for _ in 0..5 {
            assert!(bucket.check().is_allowed());
        }

        // Should be full
        assert!(!bucket.check().is_allowed());
    }

    #[test]
    fn test_sliding_window_log() {
        let window = SlidingWindowLog::new(3, Duration::from_secs(1));

        assert!(window.check().is_allowed());
        assert!(window.check().is_allowed());
        assert!(window.check().is_allowed());
        assert!(!window.check().is_allowed());

        assert_eq!(window.current_count(), 3);
    }

    #[test]
    fn test_sliding_window_counter() {
        let counter = SlidingWindowCounter::new(5, Duration::from_secs(1));

        for _ in 0..5 {
            assert!(counter.check().is_allowed());
        }

        assert!(!counter.check().is_allowed());
    }

    #[test]
    fn test_gcra_basic() {
        let gcra = GCRA::new(10.0, 5);

        // Should allow burst
        for _ in 0..5 {
            assert!(gcra.check().is_allowed());
        }

        // Should deny after burst
        assert!(!gcra.check().is_allowed());
    }

    #[test]
    fn test_decision_methods() {
        let allowed = Decision::Allowed {
            remaining: 10,
            reset_after: 60,
        };
        assert!(allowed.is_allowed());
        assert_eq!(allowed.remaining(), Some(10));
        assert_eq!(allowed.retry_after(), None);

        let denied = Decision::Denied {
            retry_after: 30,
            limit: 100,
        };
        assert!(!denied.is_allowed());
        assert_eq!(denied.remaining(), None);
        assert_eq!(denied.retry_after(), Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_token_bucket_reset() {
        let bucket = TokenBucket::new(5, 1.0);

        // Consume all tokens
        for _ in 0..5 {
            bucket.check(1);
        }
        assert!(!bucket.check(1).is_allowed());

        // Reset
        bucket.reset();
        assert!(bucket.check(1).is_allowed());
    }
}
