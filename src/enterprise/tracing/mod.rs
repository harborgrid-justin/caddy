//! # Distributed Tracing & Observability
//!
//! This module provides a comprehensive observability stack for CADDY including distributed
//! tracing, metrics collection, log correlation, sampling strategies, and performance profiling.
//!
//! ## Features
//!
//! - **Distributed Tracing**: W3C Trace Context compliant span management with parent-child
//!   relationships, baggage propagation, and context injection/extraction.
//!
//! - **Multi-Format Export**: Support for OpenTelemetry Protocol (OTLP), Jaeger, Zipkin,
//!   and console output for flexible backend integration.
//!
//! - **Metrics Collection**: Counter, Gauge, and Histogram metric types with Prometheus
//!   exposition format and StatsD protocol support.
//!
//! - **Log Correlation**: Automatic trace ID injection in logs, structured logging,
//!   log-to-span linking, and trace-aware log sampling.
//!
//! - **Intelligent Sampling**: Multiple sampling strategies including head-based, tail-based,
//!   rate-limited, priority-based, and adaptive sampling.
//!
//! - **Performance Profiling**: CPU profiling with stack traces, memory allocation tracking,
//!   flame graph generation, and continuous profiling support.
//!
//! ## Quick Start
//!
//! ### Basic Tracing
//!
//! ```rust
//! use caddy::enterprise::tracing::{
//!     span::{Span, SpanKind},
//!     exporter::{ConsoleExporter, TraceExporter},
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a span
//! let mut span = Span::new("process_drawing")
//!     .with_kind(SpanKind::Server);
//!
//! span.set_attribute("drawing_id", "12345");
//! span.set_attribute("user", "alice");
//!
//! // Add events
//! span.add_event("validation_started");
//! span.add_event("validation_completed");
//!
//! // End the span
//! span.end();
//!
//! // Export the span
//! let exporter = ConsoleExporter::new();
//! exporter.export(vec![span]).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Parent-Child Spans
//!
//! ```rust
//! use caddy::enterprise::tracing::span::Span;
//!
//! # fn example() {
//! let mut parent = Span::new("render_drawing");
//! parent.set_attribute("resolution", "1920x1080");
//!
//! let mut child = parent.child("render_layer");
//! child.set_attribute("layer", "geometry");
//! child.end();
//!
//! parent.end();
//! # }
//! ```
//!
//! ### Metrics Collection
//!
//! ```rust
//! use caddy::enterprise::tracing::metrics::{MetricRegistry, buckets};
//!
//! # fn example() {
//! let registry = MetricRegistry::new();
//!
//! // Counter for tracking requests
//! let requests = registry.counter("cad_requests_total", "Total CAD operations");
//! requests.inc();
//!
//! // Gauge for active users
//! let active_users = registry.gauge("cad_active_users", "Number of active users");
//! active_users.set(42.0);
//!
//! // Histogram for operation duration
//! let duration = registry.histogram(
//!     "cad_operation_duration_seconds",
//!     "CAD operation duration",
//!     buckets::DEFAULT.to_vec(),
//! );
//! duration.observe(0.123);
//!
//! // Export metrics in Prometheus format
//! let metrics = registry.prometheus_export();
//! println!("{}", metrics);
//! # }
//! ```
//!
//! ### Log Correlation
//!
//! ```rust
//! use caddy::enterprise::tracing::{
//!     span::Span,
//!     correlation::{LogCollector, LogLevel, SpanLogger},
//! };
//! use std::sync::Arc;
//!
//! # fn example() {
//! let collector = Arc::new(LogCollector::new());
//! let span = Span::new("process_file");
//! let logger = SpanLogger::new(span, collector.clone());
//!
//! logger.info("Processing started");
//! logger.debug("Validating input");
//! logger.info("Processing completed");
//!
//! // All logs are automatically correlated with the span
//! let logs = logger.logs();
//! assert_eq!(logs.len(), 3);
//! # }
//! ```
//!
//! ### Sampling
//!
//! ```rust
//! use caddy::enterprise::tracing::sampler::{
//!     Sampler, ProbabilitySampler, RateLimitingSampler,
//!     ParentBasedSampler, SamplingRule, RuleBasedSampler,
//! };
//!
//! # fn example() {
//! // Sample 10% of traces
//! let probability_sampler = ProbabilitySampler::new(0.1);
//!
//! // Limit to 100 traces per second
//! let rate_sampler = RateLimitingSampler::new(100);
//!
//! // Rule-based sampling
//! let rule_sampler = RuleBasedSampler::new(Box::new(ProbabilitySampler::new(0.01)));
//! rule_sampler.add_rule(
//!     SamplingRule::new("api-requests", 1.0)
//!         .with_span_pattern("api_.*")
//!         .with_priority(10)
//! );
//! # }
//! ```
//!
//! ### Performance Profiling
//!
//! ```rust
//! use caddy::enterprise::tracing::profiler::{
//!     CpuProfiler, MemoryProfiler, ContinuousProfiler, ProfileScope,
//! };
//! use std::sync::Arc;
//!
//! # fn example() {
//! // CPU profiling
//! let cpu_profiler = Arc::new(CpuProfiler::new());
//! cpu_profiler.start();
//!
//! {
//!     let _scope = ProfileScope::with_profiler("expensive_operation", cpu_profiler.clone());
//!     // ... do work ...
//! }
//!
//! cpu_profiler.stop();
//! let stats = cpu_profiler.stats();
//! let flame_graph = cpu_profiler.flame_graph_data();
//!
//! // Memory profiling
//! let mem_profiler = MemoryProfiler::new();
//! mem_profiler.start();
//! mem_profiler.record_allocation(1024, caddy::enterprise::tracing::profiler::AllocationType::Heap);
//! let mem_stats = mem_profiler.stats();
//! # }
//! ```
//!
//! ## OpenTelemetry Integration
//!
//! ### Exporting to OTLP
//!
//! ```rust
//! use caddy::enterprise::tracing::{
//!     span::Span,
//!     exporter::{OtlpExporter, TraceExporter},
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let exporter = OtlpExporter::new("http://localhost:4318")
//!     .with_header("Authorization", "Bearer token")
//!     .with_batch_size(100);
//!
//! let span = Span::new("operation");
//! exporter.export(vec![span]).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Exporting to Jaeger
//!
//! ```rust
//! use caddy::enterprise::tracing::{
//!     span::Span,
//!     exporter::{JaegerExporter, TraceExporter},
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let exporter = JaegerExporter::new("http://localhost:14268", "caddy");
//! let span = Span::new("operation");
//! exporter.export(vec![span]).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Exporting to Zipkin
//!
//! ```rust
//! use caddy::enterprise::tracing::{
//!     span::Span,
//!     exporter::{ZipkinExporter, TraceExporter},
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let exporter = ZipkinExporter::new("http://localhost:9411", "caddy");
//! let span = Span::new("operation");
//! exporter.export(vec![span]).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## W3C Trace Context
//!
//! The tracing system supports W3C Trace Context for distributed tracing across services:
//!
//! ```rust
//! use caddy::enterprise::tracing::span::SpanContext;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a span context
//! let context = SpanContext::new_root();
//!
//! // Convert to W3C traceparent header
//! let traceparent = context.to_traceparent();
//! // "00-<trace-id>-<span-id>-<flags>"
//!
//! // Parse from incoming header
//! let parsed = SpanContext::from_traceparent(&traceparent)?;
//! assert_eq!(parsed.trace_id, context.trace_id);
//! # Ok(())
//! # }
//! ```
//!
//! ## Baggage Propagation
//!
//! Baggage items are propagated across span boundaries for cross-cutting concerns:
//!
//! ```rust
//! use caddy::enterprise::tracing::span::Span;
//!
//! # fn example() {
//! let mut parent = Span::new("parent");
//! parent.set_baggage("user_id", "alice");
//! parent.set_baggage("tenant", "acme-corp");
//!
//! let child = parent.child("child");
//! assert_eq!(child.get_baggage("user_id"), Some(&"alice".to_string()));
//! assert_eq!(child.get_baggage("tenant"), Some(&"acme-corp".to_string()));
//! # }
//! ```
//!
//! ## Best Practices
//!
//! ### 1. Span Naming
//!
//! Use clear, descriptive span names that indicate the operation:
//!
//! ```rust
//! use caddy::enterprise::tracing::span::Span;
//!
//! # fn example() {
//! // Good
//! let span = Span::new("render_3d_model");
//! let span = Span::new("validate_dxf_file");
//! let span = Span::new("calculate_intersections");
//!
//! // Avoid
//! let span = Span::new("function1");
//! let span = Span::new("process");
//! # }
//! ```
//!
//! ### 2. Set Meaningful Attributes
//!
//! Add attributes that provide context for debugging and analysis:
//!
//! ```rust
//! use caddy::enterprise::tracing::span::Span;
//!
//! # fn example() {
//! let mut span = Span::new("load_drawing");
//! span.set_attribute("file_format", "dxf");
//! span.set_attribute("file_size_mb", 12.5);
//! span.set_attribute("layer_count", 42);
//! span.set_attribute("user_id", "alice");
//! # }
//! ```
//!
//! ### 3. Use Sampling Wisely
//!
//! In production, always use sampling to control overhead:
//!
//! ```rust
//! use caddy::enterprise::tracing::sampler::{
//!     RuleBasedSampler, SamplingRule, ProbabilitySampler,
//! };
//!
//! # fn example() {
//! let sampler = RuleBasedSampler::new(
//!     Box::new(ProbabilitySampler::new(0.01)) // 1% default
//! );
//!
//! // Sample all errors
//! sampler.add_rule(
//!     SamplingRule::new("errors", 1.0)
//!         .with_span_pattern(".*_error")
//!         .with_priority(100)
//! );
//!
//! // Sample all API requests at 10%
//! sampler.add_rule(
//!     SamplingRule::new("api", 0.1)
//!         .with_span_pattern("api_.*")
//!         .with_priority(50)
//! );
//! # }
//! ```
//!
//! ### 4. Correlate Logs with Traces
//!
//! Always use the log correlation features to connect logs with traces:
//!
//! ```rust
//! use caddy::enterprise::tracing::{
//!     span::Span,
//!     correlation::{LogCollector, SpanLogger},
//! };
//! use std::sync::Arc;
//!
//! # fn example() {
//! let collector = Arc::new(LogCollector::new());
//! let span = Span::new("operation");
//! let logger = SpanLogger::new(span, collector);
//!
//! logger.info("Starting operation");
//! // Work happens here
//! logger.info("Operation completed");
//! # }
//! ```
//!
//! ## Architecture
//!
//! The tracing module is organized into several submodules:
//!
//! ```text
//! tracing/
//! ├── span.rs          - Span context, IDs, and span management
//! ├── exporter.rs      - OTLP, Jaeger, Zipkin, Console exporters
//! ├── metrics.rs       - Counter, Gauge, Histogram metrics
//! ├── correlation.rs   - Log-to-trace correlation
//! ├── sampler.rs       - Sampling strategies
//! ├── profiler.rs      - CPU and memory profiling
//! └── mod.rs           - Module documentation (this file)
//! ```
//!
//! ## Performance Considerations
//!
//! - **Sampling**: Always use sampling in production to limit overhead
//! - **Batch Export**: Use batch exporters to reduce network calls
//! - **Async Export**: Exporters are async to avoid blocking application threads
//! - **Lock-Free**: Core data structures use lock-free operations where possible
//! - **Zero-Copy**: Span IDs and trace IDs are designed for zero-copy propagation
//!
//! ## Compliance
//!
//! This module implements:
//!
//! - **W3C Trace Context**: For distributed tracing across services
//! - **OpenTelemetry**: Compatible span format and OTLP export
//! - **Prometheus**: Standard exposition format for metrics
//! - **StatsD**: Wire protocol for metrics aggregation
//!
//! ## Version
//!
//! Current version: 0.2.0

#![warn(missing_docs)]
#![warn(clippy::all)]
#![allow(clippy::module_name_repetitions)]

// ============================================================================
// Module Declarations
// ============================================================================

/// Span management and context propagation
pub mod span;

/// Trace exporters for various backends
pub mod exporter;

/// Metrics collection and exposition
pub mod metrics;

/// Log-to-trace correlation
pub mod correlation;

/// Sampling strategies
pub mod sampler;

/// Performance profiling
pub mod profiler;

// ============================================================================
// Re-exports for convenience
// ============================================================================

pub use span::{
    Span, SpanContext, SpanId, SpanKind, SpanStatus, TraceId, TraceFlags,
    TraceState, SpanStore, AttributeValue,
};

pub use exporter::{
    TraceExporter, OtlpExporter, JaegerExporter, ZipkinExporter,
    ConsoleExporter, MultiExporter, BatchExporter, ExportError,
};

pub use metrics::{
    MetricRegistry, Counter, Gauge, Histogram, HistogramStats,
    Metric, Labels, StatsdClient, MetricError, buckets,
};

pub use correlation::{
    LogCollector, LogLevel, CorrelatedLog, SpanLogger,
    LogSampler, FieldBuilder, ContextGuard,
};

pub use sampler::{
    Sampler, SamplingDecision, SamplingRule,
    AlwaysSampler, NeverSampler, ProbabilitySampler,
    RateLimitingSampler, ParentBasedSampler, RuleBasedSampler,
    TailBasedSampler, TailSamplingCriteria, CompositeSampler,
    CompositeMode, AdaptiveSampler,
};

pub use profiler::{
    CpuProfiler, MemoryProfiler, ContinuousProfiler,
    ProfileScope, ProfileStats, MemoryStats, ProfilingReport,
    FlameGraphData, FlameGraphNode, StackFrame,
    CpuSample, AllocationEvent, AllocationType,
};

// ============================================================================
// Module version and metadata
// ============================================================================

/// Tracing module version
pub const VERSION: &str = "0.2.0";

/// Build timestamp
pub const BUILD_DATE: &str = "2025-12-28";

/// Check if tracing is enabled (always true in this implementation)
pub fn is_enabled() -> bool {
    true
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_version() {
        assert_eq!(VERSION, "0.2.0");
    }

    #[test]
    fn test_is_enabled() {
        assert!(is_enabled());
    }

    #[tokio::test]
    async fn test_end_to_end_tracing() {
        // Create a span
        let mut span = Span::new("test_operation");
        span.set_attribute("test_id", "12345");

        // Add child span
        let mut child = span.child("child_operation");
        child.set_attribute("child_attr", "value");
        child.end();

        span.end();

        // Export
        let exporter = ConsoleExporter::new();
        let result = exporter.export(vec![span, child]).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_metrics_end_to_end() {
        let registry = MetricRegistry::new();

        let counter = registry.counter("test_counter", "Test counter");
        counter.inc();

        let gauge = registry.gauge("test_gauge", "Test gauge");
        gauge.set(42.0);

        let histogram = registry.histogram("test_histogram", "Test histogram", vec![1.0, 5.0, 10.0]);
        histogram.observe(3.5);

        let prometheus = registry.prometheus_export();
        assert!(prometheus.contains("test_counter"));
        assert!(prometheus.contains("test_gauge"));
        assert!(prometheus.contains("test_histogram"));
    }

    #[test]
    fn test_log_correlation_end_to_end() {
        use std::sync::Arc;

        let collector = Arc::new(LogCollector::new());
        let span = Span::new("test_span");
        let logger = SpanLogger::new(span.clone(), collector.clone());

        logger.info("Test message");
        logger.error("Error message");

        let logs = logger.logs();
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].trace_id, Some(span.context.trace_id));
    }

    #[test]
    fn test_sampling_end_to_end() {
        let sampler = ProbabilitySampler::new(1.0);
        let context = SpanContext::new_root();

        let decision = sampler.should_sample(&context, "test");
        assert_eq!(decision, SamplingDecision::RecordAndSample);
    }

    #[test]
    fn test_profiling_end_to_end() {
        use std::sync::Arc;

        let profiler = Arc::new(CpuProfiler::new());
        profiler.start();

        {
            let _scope = ProfileScope::with_profiler("test", profiler.clone());
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        profiler.stop();
        let stats = profiler.stats();
        assert_eq!(stats.total_samples, 1);
    }
}
