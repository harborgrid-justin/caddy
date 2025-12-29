//! Trace sampling strategies
//!
//! This module provides head-based sampling, tail-based sampling, rate-limited
//! sampling, and priority-based sampling rules for trace collection.

use super::span::{Span, SpanContext, SpanStatus};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Sampling decision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SamplingDecision {
    /// Record and sample the span
    RecordAndSample,
    /// Record but don't sample
    RecordOnly,
    /// Drop the span
    Drop,
}

/// Trait for trace samplers
pub trait Sampler: Send + Sync {
    /// Make a sampling decision for a span
    fn should_sample(&self, context: &SpanContext, span_name: &str) -> SamplingDecision;

    /// Get sampler description
    fn description(&self) -> String;
}

// ============================================================================
// Always Sample (for development/debugging)
// ============================================================================

/// Always sample all traces
pub struct AlwaysSampler;

impl AlwaysSampler {
    /// Create a new always sampler
    pub fn new() -> Self {
        Self
    }
}

impl Default for AlwaysSampler {
    fn default() -> Self {
        Self::new()
    }
}

impl Sampler for AlwaysSampler {
    fn should_sample(&self, _context: &SpanContext, _span_name: &str) -> SamplingDecision {
        SamplingDecision::RecordAndSample
    }

    fn description(&self) -> String {
        "AlwaysSampler".to_string()
    }
}

// ============================================================================
// Never Sample
// ============================================================================

/// Never sample any traces
pub struct NeverSampler;

impl NeverSampler {
    /// Create a new never sampler
    pub fn new() -> Self {
        Self
    }
}

impl Default for NeverSampler {
    fn default() -> Self {
        Self::new()
    }
}

impl Sampler for NeverSampler {
    fn should_sample(&self, _context: &SpanContext, _span_name: &str) -> SamplingDecision {
        SamplingDecision::Drop
    }

    fn description(&self) -> String {
        "NeverSampler".to_string()
    }
}

// ============================================================================
// Probability Sampler (head-based)
// ============================================================================

/// Sample traces based on probability (0.0 to 1.0)
pub struct ProbabilitySampler {
    probability: f64,
}

impl ProbabilitySampler {
    /// Create a new probability sampler
    pub fn new(probability: f64) -> Self {
        Self {
            probability: probability.clamp(0.0, 1.0),
        }
    }
}

impl Sampler for ProbabilitySampler {
    fn should_sample(&self, _context: &SpanContext, _span_name: &str) -> SamplingDecision {
        if rand::random::<f64>() < self.probability {
            SamplingDecision::RecordAndSample
        } else {
            SamplingDecision::Drop
        }
    }

    fn description(&self) -> String {
        format!("ProbabilitySampler({})", self.probability)
    }
}

// ============================================================================
// Rate Limiting Sampler
// ============================================================================

/// Sample up to N traces per second
pub struct RateLimitingSampler {
    max_traces_per_second: u64,
    state: Arc<RwLock<RateLimitState>>,
}

struct RateLimitState {
    last_reset: Instant,
    count: u64,
}

impl RateLimitingSampler {
    /// Create a new rate limiting sampler
    pub fn new(max_traces_per_second: u64) -> Self {
        Self {
            max_traces_per_second,
            state: Arc::new(RwLock::new(RateLimitState {
                last_reset: Instant::now(),
                count: 0,
            })),
        }
    }
}

impl Sampler for RateLimitingSampler {
    fn should_sample(&self, _context: &SpanContext, _span_name: &str) -> SamplingDecision {
        let mut state = self.state.write();

        // Reset counter every second
        if state.last_reset.elapsed() >= Duration::from_secs(1) {
            state.last_reset = Instant::now();
            state.count = 0;
        }

        if state.count < self.max_traces_per_second {
            state.count += 1;
            SamplingDecision::RecordAndSample
        } else {
            SamplingDecision::Drop
        }
    }

    fn description(&self) -> String {
        format!("RateLimitingSampler({}/s)", self.max_traces_per_second)
    }
}

// ============================================================================
// Parent-based Sampler
// ============================================================================

/// Sample based on parent span's sampling decision
pub struct ParentBasedSampler {
    root_sampler: Box<dyn Sampler>,
}

impl ParentBasedSampler {
    /// Create a new parent-based sampler
    pub fn new(root_sampler: Box<dyn Sampler>) -> Self {
        Self { root_sampler }
    }
}

impl Sampler for ParentBasedSampler {
    fn should_sample(&self, context: &SpanContext, span_name: &str) -> SamplingDecision {
        if context.parent_span_id.is_some() {
            // Use parent's decision
            if context.is_sampled() {
                SamplingDecision::RecordAndSample
            } else {
                SamplingDecision::Drop
            }
        } else {
            // Root span - use root sampler
            self.root_sampler.should_sample(context, span_name)
        }
    }

    fn description(&self) -> String {
        format!("ParentBasedSampler(root: {})", self.root_sampler.description())
    }
}

// ============================================================================
// Rule-based Sampler (priority-based)
// ============================================================================

/// Sampling rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingRule {
    /// Rule name
    pub name: String,
    /// Span name pattern (regex)
    pub span_name_pattern: Option<String>,
    /// Attribute matchers
    pub attribute_matchers: HashMap<String, String>,
    /// Sampling probability for matching spans
    pub probability: f64,
    /// Priority (higher = evaluated first)
    pub priority: i32,
}

impl SamplingRule {
    /// Create a new sampling rule
    pub fn new(name: impl Into<String>, probability: f64) -> Self {
        Self {
            name: name.into(),
            span_name_pattern: None,
            attribute_matchers: HashMap::new(),
            probability: probability.clamp(0.0, 1.0),
            priority: 0,
        }
    }

    /// Set span name pattern
    pub fn with_span_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.span_name_pattern = Some(pattern.into());
        self
    }

    /// Add attribute matcher
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attribute_matchers.insert(key.into(), value.into());
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Check if span matches this rule
    pub fn matches(&self, span_name: &str, _attributes: &HashMap<String, String>) -> bool {
        // Check span name pattern
        if let Some(pattern) = &self.span_name_pattern {
            if let Ok(re) = regex::Regex::new(pattern) {
                if !re.is_match(span_name) {
                    return false;
                }
            }
        }

        // For now, just match on span name
        // In a real implementation, would also check attributes
        true
    }
}

/// Rule-based sampler with priority rules
pub struct RuleBasedSampler {
    rules: Arc<RwLock<Vec<SamplingRule>>>,
    default_sampler: Box<dyn Sampler>,
}

impl RuleBasedSampler {
    /// Create a new rule-based sampler
    pub fn new(default_sampler: Box<dyn Sampler>) -> Self {
        Self {
            rules: Arc::new(RwLock::new(Vec::new())),
            default_sampler,
        }
    }

    /// Add a sampling rule
    pub fn add_rule(&self, rule: SamplingRule) {
        let mut rules = self.rules.write();
        rules.push(rule);
        // Sort by priority (highest first)
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Get all rules
    pub fn rules(&self) -> Vec<SamplingRule> {
        self.rules.read().clone()
    }
}

impl Sampler for RuleBasedSampler {
    fn should_sample(&self, context: &SpanContext, span_name: &str) -> SamplingDecision {
        let rules = self.rules.read();
        let attributes = HashMap::new(); // Would get from context in real impl

        for rule in rules.iter() {
            if rule.matches(span_name, &attributes) {
                return if rand::random::<f64>() < rule.probability {
                    SamplingDecision::RecordAndSample
                } else {
                    SamplingDecision::Drop
                };
            }
        }

        // No rule matched, use default sampler
        self.default_sampler.should_sample(context, span_name)
    }

    fn description(&self) -> String {
        format!("RuleBasedSampler({} rules)", self.rules.read().len())
    }
}

// ============================================================================
// Tail-based Sampler
// ============================================================================

/// Tail-based sampling decision criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TailSamplingCriteria {
    /// Sample if span has error status
    HasError,
    /// Sample if duration exceeds threshold (in seconds)
    DurationAbove(f64),
    /// Sample if duration is below threshold (in seconds)
    DurationBelow(f64),
    /// Sample if span has specific attribute
    HasAttribute(String, String),
    /// Always sample
    Always,
}

/// Tail-based sampler (samples after span completes)
pub struct TailBasedSampler {
    criteria: Vec<TailSamplingCriteria>,
    fallback_probability: f64,
}

impl TailBasedSampler {
    /// Create a new tail-based sampler
    pub fn new() -> Self {
        Self {
            criteria: Vec::new(),
            fallback_probability: 0.1,
        }
    }

    /// Add sampling criteria
    pub fn with_criteria(mut self, criteria: TailSamplingCriteria) -> Self {
        self.criteria.push(criteria);
        self
    }

    /// Set fallback probability
    pub fn with_fallback_probability(mut self, probability: f64) -> Self {
        self.fallback_probability = probability.clamp(0.0, 1.0);
        self
    }

    /// Evaluate if a completed span should be sampled
    pub fn should_sample_span(&self, span: &Span) -> bool {
        for criterion in &self.criteria {
            match criterion {
                TailSamplingCriteria::HasError => {
                    if span.status == SpanStatus::Error {
                        return true;
                    }
                }
                TailSamplingCriteria::DurationAbove(threshold) => {
                    if let Some(duration) = span.duration() {
                        if duration.as_secs_f64() > *threshold {
                            return true;
                        }
                    }
                }
                TailSamplingCriteria::DurationBelow(threshold) => {
                    if let Some(duration) = span.duration() {
                        if duration.as_secs_f64() < *threshold {
                            return true;
                        }
                    }
                }
                TailSamplingCriteria::HasAttribute(key, value) => {
                    if let Some(attr_value) = span.attributes.get(key) {
                        if format!("{:?}", attr_value).contains(value) {
                            return true;
                        }
                    }
                }
                TailSamplingCriteria::Always => {
                    return true;
                }
            }
        }

        // Fallback to probability-based sampling
        rand::random::<f64>() < self.fallback_probability
    }
}

impl Default for TailBasedSampler {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Composite Sampler
// ============================================================================

/// Composite sampler that combines multiple samplers
pub struct CompositeSampler {
    samplers: Vec<Box<dyn Sampler>>,
    mode: CompositeMode,
}

/// Composite mode
#[derive(Debug, Clone, Copy)]
pub enum CompositeMode {
    /// Sample if ALL samplers agree to sample
    And,
    /// Sample if ANY sampler agrees to sample
    Or,
}

impl CompositeSampler {
    /// Create a new composite sampler
    pub fn new(mode: CompositeMode) -> Self {
        Self {
            samplers: Vec::new(),
            mode,
        }
    }

    /// Add a sampler
    pub fn add_sampler(mut self, sampler: Box<dyn Sampler>) -> Self {
        self.samplers.push(sampler);
        self
    }
}

impl Sampler for CompositeSampler {
    fn should_sample(&self, context: &SpanContext, span_name: &str) -> SamplingDecision {
        if self.samplers.is_empty() {
            return SamplingDecision::Drop;
        }

        let decisions: Vec<SamplingDecision> = self.samplers
            .iter()
            .map(|s| s.should_sample(context, span_name))
            .collect();

        match self.mode {
            CompositeMode::And => {
                if decisions.iter().all(|d| matches!(d, SamplingDecision::RecordAndSample)) {
                    SamplingDecision::RecordAndSample
                } else {
                    SamplingDecision::Drop
                }
            }
            CompositeMode::Or => {
                if decisions.iter().any(|d| matches!(d, SamplingDecision::RecordAndSample)) {
                    SamplingDecision::RecordAndSample
                } else {
                    SamplingDecision::Drop
                }
            }
        }
    }

    fn description(&self) -> String {
        format!("CompositeSampler({:?}, {} samplers)", self.mode, self.samplers.len())
    }
}

// ============================================================================
// Adaptive Sampler
// ============================================================================

/// Adaptive sampler that adjusts sampling rate based on throughput
pub struct AdaptiveSampler {
    state: Arc<RwLock<AdaptiveState>>,
    min_probability: f64,
    max_probability: f64,
    target_traces_per_second: u64,
}

struct AdaptiveState {
    current_probability: f64,
    last_adjustment: Instant,
    trace_count: u64,
}

impl AdaptiveSampler {
    /// Create a new adaptive sampler
    pub fn new(target_traces_per_second: u64) -> Self {
        Self {
            state: Arc::new(RwLock::new(AdaptiveState {
                current_probability: 0.1,
                last_adjustment: Instant::now(),
                trace_count: 0,
            })),
            min_probability: 0.001,
            max_probability: 1.0,
            target_traces_per_second,
        }
    }

    /// Set probability bounds
    pub fn with_bounds(mut self, min: f64, max: f64) -> Self {
        self.min_probability = min.clamp(0.0, 1.0);
        self.max_probability = max.clamp(0.0, 1.0);
        self
    }

    fn adjust_probability(&self) {
        let mut state = self.state.write();

        if state.last_adjustment.elapsed() >= Duration::from_secs(1) {
            let actual_rate = state.trace_count;
            let target = self.target_traces_per_second;

            // Adjust probability based on actual vs target rate
            if actual_rate > target {
                // Too many traces, decrease probability
                state.current_probability *= 0.9;
            } else if actual_rate < target {
                // Too few traces, increase probability
                state.current_probability *= 1.1;
            }

            // Clamp to bounds
            state.current_probability = state.current_probability
                .clamp(self.min_probability, self.max_probability);

            state.trace_count = 0;
            state.last_adjustment = Instant::now();
        }
    }
}

impl Sampler for AdaptiveSampler {
    fn should_sample(&self, _context: &SpanContext, _span_name: &str) -> SamplingDecision {
        self.adjust_probability();

        let mut state = self.state.write();
        state.trace_count += 1;

        let probability = state.current_probability;
        drop(state);

        if rand::random::<f64>() < probability {
            SamplingDecision::RecordAndSample
        } else {
            SamplingDecision::Drop
        }
    }

    fn description(&self) -> String {
        let state = self.state.read();
        format!("AdaptiveSampler(p={:.4}, target={}/s)",
                state.current_probability,
                self.target_traces_per_second)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_always_sampler() {
        let sampler = AlwaysSampler::new();
        let _context = SpanContext::new_root();

        let decision = sampler.should_sample(&context, "test");
        assert_eq!(decision, SamplingDecision::RecordAndSample);
    }

    #[test]
    fn test_never_sampler() {
        let sampler = NeverSampler::new();
        let _context = SpanContext::new_root();

        let decision = sampler.should_sample(&context, "test");
        assert_eq!(decision, SamplingDecision::Drop);
    }

    #[test]
    fn test_probability_sampler_always() {
        let sampler = ProbabilitySampler::new(1.0);
        let _context = SpanContext::new_root();

        let decision = sampler.should_sample(&context, "test");
        assert_eq!(decision, SamplingDecision::RecordAndSample);
    }

    #[test]
    fn test_probability_sampler_never() {
        let sampler = ProbabilitySampler::new(0.0);
        let _context = SpanContext::new_root();

        let decision = sampler.should_sample(&context, "test");
        assert_eq!(decision, SamplingDecision::Drop);
    }

    #[test]
    fn test_rate_limiting_sampler() {
        let sampler = RateLimitingSampler::new(5);
        let _context = SpanContext::new_root();

        let mut sampled = 0;
        for _ in 0..10 {
            if sampler.should_sample(&context, "test") == SamplingDecision::RecordAndSample {
                sampled += 1;
            }
        }

        assert_eq!(sampled, 5);
    }

    #[test]
    fn test_parent_based_sampler_root() {
        let root_sampler = Box::new(AlwaysSampler::new());
        let sampler = ParentBasedSampler::new(root_sampler);

        let root_context = SpanContext::new_root();
        let decision = sampler.should_sample(&root_context, "test");
        assert_eq!(decision, SamplingDecision::RecordAndSample);
    }

    #[test]
    fn test_parent_based_sampler_child() {
        let root_sampler = Box::new(NeverSampler::new());
        let sampler = ParentBasedSampler::new(root_sampler);

        let mut parent_context = SpanContext::new_root();
        parent_context.set_sampled(true);
        let child_context = parent_context.child();

        let decision = sampler.should_sample(&child_context, "test");
        assert_eq!(decision, SamplingDecision::RecordAndSample);
    }

    #[test]
    fn test_sampling_rule() {
        let rule = SamplingRule::new("test-rule", 1.0)
            .with_span_pattern("api_.*")
            .with_priority(10);

        assert_eq!(rule.name, "test-rule");
        assert_eq!(rule.probability, 1.0);
        assert_eq!(rule.priority, 10);
    }

    #[test]
    fn test_rule_based_sampler() {
        let default_sampler = Box::new(NeverSampler::new());
        let sampler = RuleBasedSampler::new(default_sampler);

        let rule = SamplingRule::new("api-rule", 1.0)
            .with_span_pattern("api_.*");
        sampler.add_rule(rule);

        let _context = SpanContext::new_root();
        let decision = sampler.should_sample(&context, "api_request");
        assert_eq!(decision, SamplingDecision::RecordAndSample);
    }

    #[test]
    fn test_tail_based_sampler_error() {
        let sampler = TailBasedSampler::new()
            .with_criteria(TailSamplingCriteria::HasError);

        let mut span = Span::new("test");
        span.set_status(SpanStatus::Error);

        assert!(sampler.should_sample_span(&span));
    }

    #[test]
    fn test_tail_based_sampler_duration() {
        let sampler = TailBasedSampler::new()
            .with_criteria(TailSamplingCriteria::DurationAbove(0.0));

        let mut span = Span::new("test");
        std::thread::sleep(std::time::Duration::from_millis(10));
        span.end();

        assert!(sampler.should_sample_span(&span));
    }

    #[test]
    fn test_composite_sampler_and() {
        let sampler = CompositeSampler::new(CompositeMode::And)
            .add_sampler(Box::new(AlwaysSampler::new()))
            .add_sampler(Box::new(AlwaysSampler::new()));

        let _context = SpanContext::new_root();
        let decision = sampler.should_sample(&context, "test");
        assert_eq!(decision, SamplingDecision::RecordAndSample);
    }

    #[test]
    fn test_composite_sampler_or() {
        let sampler = CompositeSampler::new(CompositeMode::Or)
            .add_sampler(Box::new(AlwaysSampler::new()))
            .add_sampler(Box::new(NeverSampler::new()));

        let _context = SpanContext::new_root();
        let decision = sampler.should_sample(&context, "test");
        assert_eq!(decision, SamplingDecision::RecordAndSample);
    }

    #[test]
    fn test_adaptive_sampler() {
        let sampler = AdaptiveSampler::new(100)
            .with_bounds(0.01, 1.0);

        let _context = SpanContext::new_root();

        // Just test that it runs without crashing
        for _ in 0..10 {
            sampler.should_sample(&context, "test");
        }
    }

    #[test]
    fn test_sampler_descriptions() {
        assert_eq!(AlwaysSampler::new().description(), "AlwaysSampler");
        assert_eq!(NeverSampler::new().description(), "NeverSampler");
        assert!(ProbabilitySampler::new(0.5).description().contains("0.5"));
    }

    #[test]
    fn test_tail_sampling_criteria() {
        let criteria = TailSamplingCriteria::DurationAbove(1.0);
        assert!(matches!(criteria, TailSamplingCriteria::DurationAbove(1.0)));
    }
}
