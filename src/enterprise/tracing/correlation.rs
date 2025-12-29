//! Log-to-trace correlation
//!
//! This module provides trace ID injection in logs, log-to-span linking,
//! structured logging integration, and log sampling based on trace context.

use super::span::{Span, SpanContext, TraceId, SpanId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use parking_lot::RwLock;

/// Log level enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Trace level
    Trace,
    /// Debug level
    Debug,
    /// Info level
    Info,
    /// Warn level
    Warn,
    /// Error level
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

impl From<&str> for LogLevel {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "TRACE" => LogLevel::Trace,
            "DEBUG" => LogLevel::Debug,
            "INFO" => LogLevel::Info,
            "WARN" => LogLevel::Warn,
            "ERROR" => LogLevel::Error,
            _ => LogLevel::Info,
        }
    }
}

/// Correlated log entry with trace context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelatedLog {
    /// Log timestamp
    pub timestamp: SystemTime,
    /// Log level
    pub level: LogLevel,
    /// Log message
    pub message: String,
    /// Trace ID (if in trace context)
    pub trace_id: Option<TraceId>,
    /// Span ID (if in span context)
    pub span_id: Option<SpanId>,
    /// Structured fields
    pub fields: HashMap<String, String>,
    /// Source file
    pub file: Option<String>,
    /// Source line
    pub line: Option<u32>,
    /// Module path
    pub module: Option<String>,
}

impl CorrelatedLog {
    /// Create a new correlated log entry
    pub fn new(level: LogLevel, message: impl Into<String>) -> Self {
        Self {
            timestamp: SystemTime::now(),
            level,
            message: message.into(),
            trace_id: None,
            span_id: None,
            fields: HashMap::new(),
            file: None,
            line: None,
            module: None,
        }
    }

    /// Add trace context
    pub fn with_context(mut self, context: &SpanContext) -> Self {
        self.trace_id = Some(context.trace_id);
        self.span_id = Some(context.span_id);
        self
    }

    /// Add a field
    pub fn with_field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }

    /// Add source location
    pub fn with_location(mut self, file: &str, line: u32, module: &str) -> Self {
        self.file = Some(file.to_string());
        self.line = Some(line);
        self.module = Some(module.to_string());
        self
    }

    /// Format as JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Format as human-readable string
    pub fn to_string_pretty(&self) -> String {
        let mut parts = vec![
            format!("[{}]", self.level),
            format!("{:?}", self.timestamp),
        ];

        if let Some(trace_id) = &self.trace_id {
            parts.push(format!("trace_id={}", trace_id));
        }

        if let Some(span_id) = &self.span_id {
            parts.push(format!("span_id={}", span_id));
        }

        parts.push(self.message.clone());

        if !self.fields.is_empty() {
            let fields_str = self.fields
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(" ");
            parts.push(fields_str);
        }

        parts.join(" ")
    }
}

/// Log collector that correlates logs with traces
pub struct LogCollector {
    logs: Arc<RwLock<Vec<CorrelatedLog>>>,
    current_context: Arc<RwLock<Option<SpanContext>>>,
    max_logs: usize,
}

impl LogCollector {
    /// Create a new log collector
    pub fn new() -> Self {
        Self {
            logs: Arc::new(RwLock::new(Vec::new())),
            current_context: Arc::new(RwLock::new(None)),
            max_logs: 10000,
        }
    }

    /// Set maximum number of logs to retain
    pub fn with_max_logs(mut self, max: usize) -> Self {
        self.max_logs = max;
        self
    }

    /// Set current trace context
    pub fn set_context(&self, context: Option<SpanContext>) {
        *self.current_context.write() = context;
    }

    /// Get current trace context
    pub fn get_context(&self) -> Option<SpanContext> {
        self.current_context.read().clone()
    }

    /// Log a message with current context
    pub fn log(&self, level: LogLevel, message: impl Into<String>) {
        let mut log = CorrelatedLog::new(level, message);

        if let Some(context) = self.get_context() {
            log = log.with_context(&context);
        }

        self.add_log(log);
    }

    /// Log with additional fields
    pub fn log_with_fields(
        &self,
        level: LogLevel,
        message: impl Into<String>,
        fields: HashMap<String, String>,
    ) {
        let mut log = CorrelatedLog::new(level, message);

        if let Some(context) = self.get_context() {
            log = log.with_context(&context);
        }

        for (key, value) in fields {
            log = log.with_field(key, value);
        }

        self.add_log(log);
    }

    /// Add a log entry
    pub fn add_log(&self, log: CorrelatedLog) {
        let mut logs = self.logs.write();
        logs.push(log);

        // Trim if exceeds max
        if logs.len() > self.max_logs {
            let len = logs.len();
            logs.drain(0..len - self.max_logs);
        }
    }

    /// Get all logs
    pub fn get_logs(&self) -> Vec<CorrelatedLog> {
        self.logs.read().clone()
    }

    /// Get logs for a specific trace
    pub fn get_logs_for_trace(&self, trace_id: &TraceId) -> Vec<CorrelatedLog> {
        self.logs
            .read()
            .iter()
            .filter(|log| log.trace_id.as_ref() == Some(trace_id))
            .cloned()
            .collect()
    }

    /// Get logs for a specific span
    pub fn get_logs_for_span(&self, span_id: &SpanId) -> Vec<CorrelatedLog> {
        self.logs
            .read()
            .iter()
            .filter(|log| log.span_id.as_ref() == Some(span_id))
            .cloned()
            .collect()
    }

    /// Get logs by level
    pub fn get_logs_by_level(&self, level: LogLevel) -> Vec<CorrelatedLog> {
        self.logs
            .read()
            .iter()
            .filter(|log| log.level == level)
            .cloned()
            .collect()
    }

    /// Clear all logs
    pub fn clear(&self) {
        self.logs.write().clear();
    }
}

impl Default for LogCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Span logger that attaches logs to spans
pub struct SpanLogger {
    span: Span,
    collector: Arc<LogCollector>,
}

impl SpanLogger {
    /// Create a new span logger
    pub fn new(span: Span, collector: Arc<LogCollector>) -> Self {
        collector.set_context(Some(span.context.clone()));
        Self { span, collector }
    }

    /// Log a trace message
    pub fn trace(&self, message: impl Into<String>) {
        self.collector.log(LogLevel::Trace, message);
    }

    /// Log a debug message
    pub fn debug(&self, message: impl Into<String>) {
        self.collector.log(LogLevel::Debug, message);
    }

    /// Log an info message
    pub fn info(&self, message: impl Into<String>) {
        self.collector.log(LogLevel::Info, message);
    }

    /// Log a warning message
    pub fn warn(&self, message: impl Into<String>) {
        self.collector.log(LogLevel::Warn, message);
    }

    /// Log an error message
    pub fn error(&self, message: impl Into<String>) {
        self.collector.log(LogLevel::Error, message);
    }

    /// Get the span
    pub fn span(&self) -> &Span {
        &self.span
    }

    /// Get logs for this span
    pub fn logs(&self) -> Vec<CorrelatedLog> {
        self.collector.get_logs_for_span(&self.span.context.span_id)
    }
}

/// Log sampler that samples logs based on trace context
pub struct LogSampler {
    /// Sample all logs in sampled traces
    sample_traced: bool,
    /// Sample rate for non-traced logs (0.0 to 1.0)
    sample_rate: f64,
    /// Always sample logs at or above this level
    always_sample_level: Option<LogLevel>,
}

impl LogSampler {
    /// Create a new log sampler
    pub fn new() -> Self {
        Self {
            sample_traced: true,
            sample_rate: 1.0,
            always_sample_level: Some(LogLevel::Error),
        }
    }

    /// Set whether to sample all logs in traced requests
    pub fn with_sample_traced(mut self, sample: bool) -> Self {
        self.sample_traced = sample;
        self
    }

    /// Set sample rate for non-traced logs
    pub fn with_sample_rate(mut self, rate: f64) -> Self {
        self.sample_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Set minimum level to always sample
    pub fn with_always_sample_level(mut self, level: Option<LogLevel>) -> Self {
        self.always_sample_level = level;
        self
    }

    /// Check if a log should be sampled
    pub fn should_sample(&self, log: &CorrelatedLog) -> bool {
        // Always sample if at or above the threshold level
        if let Some(threshold) = self.always_sample_level {
            if log.level >= threshold {
                return true;
            }
        }

        // Sample all logs in traced requests
        if self.sample_traced && log.trace_id.is_some() {
            return true;
        }

        // Sample based on rate for non-traced logs
        if log.trace_id.is_none() {
            return rand::random::<f64>() < self.sample_rate;
        }

        false
    }

    /// Filter logs based on sampling
    pub fn filter_logs(&self, logs: Vec<CorrelatedLog>) -> Vec<CorrelatedLog> {
        logs.into_iter()
            .filter(|log| self.should_sample(log))
            .collect()
    }
}

impl Default for LogSampler {
    fn default() -> Self {
        Self::new()
    }
}

/// Structured logging field builder
pub struct FieldBuilder {
    fields: HashMap<String, String>,
}

impl FieldBuilder {
    /// Create a new field builder
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    /// Add a string field
    pub fn str(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }

    /// Add an integer field
    pub fn int(mut self, key: impl Into<String>, value: i64) -> Self {
        self.fields.insert(key.into(), value.to_string());
        self
    }

    /// Add a float field
    pub fn float(mut self, key: impl Into<String>, value: f64) -> Self {
        self.fields.insert(key.into(), value.to_string());
        self
    }

    /// Add a boolean field
    pub fn bool(mut self, key: impl Into<String>, value: bool) -> Self {
        self.fields.insert(key.into(), value.to_string());
        self
    }

    /// Build the fields
    pub fn build(self) -> HashMap<String, String> {
        self.fields
    }
}

impl Default for FieldBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro for creating structured log fields
#[macro_export]
macro_rules! log_fields {
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut fields = std::collections::HashMap::new();
        $(
            fields.insert($key.to_string(), $value.to_string());
        )*
        fields
    }};
}

/// Context propagation for async tasks
pub struct ContextGuard {
    collector: Arc<LogCollector>,
    previous_context: Option<SpanContext>,
}

impl ContextGuard {
    /// Create a new context guard
    pub fn new(collector: Arc<LogCollector>, context: SpanContext) -> Self {
        let previous = collector.get_context();
        collector.set_context(Some(context));
        Self {
            collector,
            previous_context: previous,
        }
    }
}

impl Drop for ContextGuard {
    fn drop(&mut self) {
        self.collector.set_context(self.previous_context.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Info.to_string(), "INFO");
        assert_eq!(LogLevel::Error.to_string(), "ERROR");
    }

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from("info"), LogLevel::Info);
        assert_eq!(LogLevel::from("ERROR"), LogLevel::Error);
        assert_eq!(LogLevel::from("unknown"), LogLevel::Info);
    }

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Trace < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Error);
    }

    #[test]
    fn test_correlated_log_creation() {
        let log = CorrelatedLog::new(LogLevel::Info, "test message");
        assert_eq!(log.level, LogLevel::Info);
        assert_eq!(log.message, "test message");
        assert!(log.trace_id.is_none());
        assert!(log.span_id.is_none());
    }

    #[test]
    fn test_correlated_log_with_context() {
        let _context = SpanContext::new_root();
        let log = CorrelatedLog::new(LogLevel::Info, "test")
            .with_context(&context);

        assert_eq!(log.trace_id, Some(context.trace_id));
        assert_eq!(log.span_id, Some(context.span_id));
    }

    #[test]
    fn test_correlated_log_with_fields() {
        let log = CorrelatedLog::new(LogLevel::Info, "test")
            .with_field("user_id", "12345")
            .with_field("request_id", "abc");

        assert_eq!(log.fields.len(), 2);
        assert_eq!(log.fields.get("user_id"), Some(&"12345".to_string()));
    }

    #[test]
    fn test_correlated_log_with_location() {
        let log = CorrelatedLog::new(LogLevel::Info, "test")
            .with_location("main.rs", 42, "caddy::main");

        assert_eq!(log.file, Some("main.rs".to_string()));
        assert_eq!(log.line, Some(42));
        assert_eq!(log.module, Some("caddy::main".to_string()));
    }

    #[test]
    fn test_correlated_log_json() {
        let log = CorrelatedLog::new(LogLevel::Info, "test");
        let json = log.to_json();
        assert!(json.contains("\"message\":\"test\""));
    }

    #[test]
    fn test_log_collector_creation() {
        let collector = LogCollector::new();
        assert_eq!(collector.get_logs().len(), 0);
        assert!(collector.get_context().is_none());
    }

    #[test]
    fn test_log_collector_set_context() {
        let collector = LogCollector::new();
        let _context = SpanContext::new_root();

        collector.set_context(Some(context.clone()));
        assert_eq!(collector.get_context(), Some(context));
    }

    #[test]
    fn test_log_collector_log() {
        let collector = LogCollector::new();
        let _context = SpanContext::new_root();

        collector.set_context(Some(context.clone()));
        collector.log(LogLevel::Info, "test message");

        let logs = collector.get_logs();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].message, "test message");
        assert_eq!(logs[0].trace_id, Some(context.trace_id));
    }

    #[test]
    fn test_log_collector_log_with_fields() {
        let collector = LogCollector::new();
        let fields = log_fields! {
            "key1" => "value1",
            "key2" => "value2",
        };

        collector.log_with_fields(LogLevel::Info, "test", fields);

        let logs = collector.get_logs();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].fields.len(), 2);
    }

    #[test]
    fn test_log_collector_get_logs_for_trace() {
        let collector = LogCollector::new();
        let context1 = SpanContext::new_root();
        let context2 = SpanContext::new_root();

        collector.set_context(Some(context1.clone()));
        collector.log(LogLevel::Info, "message 1");

        collector.set_context(Some(context2.clone()));
        collector.log(LogLevel::Info, "message 2");

        let logs = collector.get_logs_for_trace(&context1.trace_id);
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].message, "message 1");
    }

    #[test]
    fn test_log_collector_get_logs_by_level() {
        let collector = LogCollector::new();
        collector.log(LogLevel::Info, "info message");
        collector.log(LogLevel::Error, "error message");
        collector.log(LogLevel::Info, "another info");

        let info_logs = collector.get_logs_by_level(LogLevel::Info);
        assert_eq!(info_logs.len(), 2);

        let error_logs = collector.get_logs_by_level(LogLevel::Error);
        assert_eq!(error_logs.len(), 1);
    }

    #[test]
    fn test_log_collector_max_logs() {
        let collector = LogCollector::new().with_max_logs(5);

        for i in 0..10 {
            collector.log(LogLevel::Info, format!("message {}", i));
        }

        let logs = collector.get_logs();
        assert_eq!(logs.len(), 5);
    }

    #[test]
    fn test_span_logger() {
        let span = Span::new("test-span");
        let collector = Arc::new(LogCollector::new());
        let logger = SpanLogger::new(span.clone(), collector.clone());

        logger.info("test message");

        let logs = logger.logs();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].span_id, Some(span.context.span_id));
    }

    #[test]
    fn test_log_sampler_creation() {
        let sampler = LogSampler::new();
        assert!(sampler.sample_traced);
        assert_eq!(sampler.sample_rate, 1.0);
    }

    #[test]
    fn test_log_sampler_always_sample_level() {
        let sampler = LogSampler::new()
            .with_sample_rate(0.0)
            .with_always_sample_level(Some(LogLevel::Error));

        let error_log = CorrelatedLog::new(LogLevel::Error, "error");
        let info_log = CorrelatedLog::new(LogLevel::Info, "info");

        assert!(sampler.should_sample(&error_log));
        assert!(!sampler.should_sample(&info_log));
    }

    #[test]
    fn test_log_sampler_sample_traced() {
        let sampler = LogSampler::new()
            .with_sample_traced(true)
            .with_sample_rate(0.0);

        let _context = SpanContext::new_root();
        let traced_log = CorrelatedLog::new(LogLevel::Info, "traced")
            .with_context(&context);
        let untraced_log = CorrelatedLog::new(LogLevel::Info, "untraced");

        assert!(sampler.should_sample(&traced_log));
        assert!(!sampler.should_sample(&untraced_log));
    }

    #[test]
    fn test_field_builder() {
        let fields = FieldBuilder::new()
            .str("name", "test")
            .int("count", 42)
            .float("ratio", 3.14)
            .bool("enabled", true)
            .build();

        assert_eq!(fields.len(), 4);
        assert_eq!(fields.get("name"), Some(&"test".to_string()));
        assert_eq!(fields.get("count"), Some(&"42".to_string()));
    }

    #[test]
    fn test_context_guard() {
        let collector = Arc::new(LogCollector::new());
        let context1 = SpanContext::new_root();
        let context2 = SpanContext::new_root();

        collector.set_context(Some(context1.clone()));

        {
            let _guard = ContextGuard::new(collector.clone(), context2.clone());
            assert_eq!(collector.get_context(), Some(context2));
        }

        assert_eq!(collector.get_context(), Some(context1));
    }
}
