//! Distributed tracing span management
//!
//! This module provides span context propagation, parent-child relationships,
//! baggage items, and timing tracking for distributed tracing.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Span context containing trace and span identifiers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpanContext {
    /// Unique trace identifier
    pub trace_id: TraceId,
    /// Unique span identifier
    pub span_id: SpanId,
    /// Parent span identifier (if any)
    pub parent_span_id: Option<SpanId>,
    /// Trace flags (sampled, debug, etc.)
    pub flags: TraceFlags,
    /// Trace state for vendor-specific data
    pub trace_state: TraceState,
}

impl SpanContext {
    /// Create a new root span context (no parent)
    pub fn new_root() -> Self {
        Self {
            trace_id: TraceId::new(),
            span_id: SpanId::new(),
            parent_span_id: None,
            flags: TraceFlags::default(),
            trace_state: TraceState::default(),
        }
    }

    /// Create a child span context from this context
    pub fn child(&self) -> Self {
        Self {
            trace_id: self.trace_id,
            span_id: SpanId::new(),
            parent_span_id: Some(self.span_id),
            flags: self.flags,
            trace_state: self.trace_state.clone(),
        }
    }

    /// Check if this span should be sampled
    pub fn is_sampled(&self) -> bool {
        self.flags.sampled
    }

    /// Set sampling decision
    pub fn set_sampled(&mut self, sampled: bool) {
        self.flags.sampled = sampled;
    }

    /// Check if this is a debug span
    pub fn is_debug(&self) -> bool {
        self.flags.debug
    }

    /// Convert to W3C trace-context format
    pub fn to_traceparent(&self) -> String {
        format!(
            "00-{}-{}-{:02x}",
            self.trace_id,
            self.span_id,
            self.flags.to_byte()
        )
    }

    /// Parse from W3C trace-context format
    pub fn from_traceparent(header: &str) -> Result<Self, TracingError> {
        let parts: Vec<&str> = header.split('-').collect();
        if parts.len() != 4 {
            return Err(TracingError::InvalidTraceContext(
                "Invalid traceparent format".to_string(),
            ));
        }

        let trace_id = TraceId::from_hex(parts[1])?;
        let span_id = SpanId::from_hex(parts[2])?;
        let flags_byte = u8::from_str_radix(parts[3], 16)
            .map_err(|_| TracingError::InvalidTraceContext("Invalid flags".to_string()))?;

        Ok(Self {
            trace_id,
            span_id,
            parent_span_id: None,
            flags: TraceFlags::from_byte(flags_byte),
            trace_state: TraceState::default(),
        })
    }
}

/// 128-bit trace identifier
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TraceId([u8; 16]);

impl TraceId {
    /// Generate a new random trace ID
    pub fn new() -> Self {
        Self(*Uuid::new_v4().as_bytes())
    }

    /// Create from hex string
    pub fn from_hex(hex: &str) -> Result<Self, TracingError> {
        if hex.len() != 32 {
            return Err(TracingError::InvalidTraceContext(
                "Trace ID must be 32 hex characters".to_string(),
            ));
        }

        let mut bytes = [0u8; 16];
        for i in 0..16 {
            bytes[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
                .map_err(|_| TracingError::InvalidTraceContext("Invalid hex".to_string()))?;
        }
        Ok(Self(bytes))
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

impl Default for TraceId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TraceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// 64-bit span identifier
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SpanId([u8; 8]);

impl SpanId {
    /// Generate a new random span ID
    pub fn new() -> Self {
        let uuid = Uuid::new_v4();
        let bytes = uuid.as_bytes();
        Self([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ])
    }

    /// Create from hex string
    pub fn from_hex(hex: &str) -> Result<Self, TracingError> {
        if hex.len() != 16 {
            return Err(TracingError::InvalidTraceContext(
                "Span ID must be 16 hex characters".to_string(),
            ));
        }

        let mut bytes = [0u8; 8];
        for i in 0..8 {
            bytes[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
                .map_err(|_| TracingError::InvalidTraceContext("Invalid hex".to_string()))?;
        }
        Ok(Self(bytes))
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

impl Default for SpanId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SpanId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Trace flags for sampling and debug
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TraceFlags {
    /// Whether this trace should be sampled
    pub sampled: bool,
    /// Whether this is a debug trace
    pub debug: bool,
}

impl TraceFlags {
    /// Convert to byte representation
    pub fn to_byte(&self) -> u8 {
        let mut byte = 0u8;
        if self.sampled {
            byte |= 0x01;
        }
        if self.debug {
            byte |= 0x02;
        }
        byte
    }

    /// Create from byte representation
    pub fn from_byte(byte: u8) -> Self {
        Self {
            sampled: (byte & 0x01) != 0,
            debug: (byte & 0x02) != 0,
        }
    }
}

impl Default for TraceFlags {
    fn default() -> Self {
        Self {
            sampled: true,
            debug: false,
        }
    }
}

/// Trace state for vendor-specific context propagation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TraceState {
    entries: HashMap<String, String>,
}

impl TraceState {
    /// Create a new empty trace state
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or update an entry
    pub fn insert(&mut self, key: String, value: String) {
        self.entries.insert(key, value);
    }

    /// Get an entry
    pub fn get(&self, key: &str) -> Option<&String> {
        self.entries.get(key)
    }

    /// Convert to W3C tracestate header format
    pub fn to_header(&self) -> String {
        self.entries
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(",")
    }

    /// Parse from W3C tracestate header format
    pub fn from_header(header: &str) -> Self {
        let mut entries = HashMap::new();
        for pair in header.split(',') {
            if let Some((k, v)) = pair.split_once('=') {
                entries.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
        Self { entries }
    }
}

/// Span representing a unit of work in a trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    /// Span context
    pub context: SpanContext,
    /// Span name/operation
    pub name: String,
    /// Span kind
    pub kind: SpanKind,
    /// Start time
    pub start_time: SystemTime,
    /// End time (None if still active)
    pub end_time: Option<SystemTime>,
    /// Span attributes (tags)
    pub attributes: HashMap<String, AttributeValue>,
    /// Span events
    pub events: Vec<SpanEvent>,
    /// Span status
    pub status: SpanStatus,
    /// Baggage items for cross-cutting concerns
    pub baggage: HashMap<String, String>,
}

impl Span {
    /// Create a new span with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            context: SpanContext::new_root(),
            name: name.into(),
            kind: SpanKind::Internal,
            start_time: SystemTime::now(),
            end_time: None,
            attributes: HashMap::new(),
            events: Vec::new(),
            status: SpanStatus::Unset,
            baggage: HashMap::new(),
        }
    }

    /// Create a child span
    pub fn child(&self, name: impl Into<String>) -> Self {
        Self {
            context: self.context.child(),
            name: name.into(),
            kind: SpanKind::Internal,
            start_time: SystemTime::now(),
            end_time: None,
            attributes: HashMap::new(),
            events: Vec::new(),
            status: SpanStatus::Unset,
            baggage: self.baggage.clone(),
        }
    }

    /// Set span kind
    pub fn with_kind(mut self, kind: SpanKind) -> Self {
        self.kind = kind;
        self
    }

    /// Add an attribute
    pub fn set_attribute(&mut self, key: impl Into<String>, value: impl Into<AttributeValue>) {
        self.attributes.insert(key.into(), value.into());
    }

    /// Add baggage item
    pub fn set_baggage(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.baggage.insert(key.into(), value.into());
    }

    /// Get baggage item
    pub fn get_baggage(&self, key: &str) -> Option<&String> {
        self.baggage.get(key)
    }

    /// Add an event
    pub fn add_event(&mut self, name: impl Into<String>) {
        self.events.push(SpanEvent {
            name: name.into(),
            timestamp: SystemTime::now(),
            attributes: HashMap::new(),
        });
    }

    /// Add an event with attributes
    pub fn add_event_with_attributes(
        &mut self,
        name: impl Into<String>,
        attributes: HashMap<String, AttributeValue>,
    ) {
        self.events.push(SpanEvent {
            name: name.into(),
            timestamp: SystemTime::now(),
            attributes,
        });
    }

    /// End the span
    pub fn end(&mut self) {
        if self.end_time.is_none() {
            self.end_time = Some(SystemTime::now());
        }
    }

    /// Set span status
    pub fn set_status(&mut self, status: SpanStatus) {
        self.status = status;
    }

    /// Record an error
    pub fn record_error(&mut self, error: &dyn std::error::Error) {
        self.set_status(SpanStatus::Error);
        self.set_attribute("error", true);
        self.set_attribute("error.message", error.to_string());
        self.set_attribute("error.type", std::any::type_name_of_val(error));
    }

    /// Get span duration
    pub fn duration(&self) -> Option<Duration> {
        self.end_time.and_then(|end| end.duration_since(self.start_time).ok())
    }

    /// Check if span is finished
    pub fn is_finished(&self) -> bool {
        self.end_time.is_some()
    }
}

/// Span kind indicating the role of the span
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SpanKind {
    /// Internal operation
    Internal,
    /// Server handling a request
    Server,
    /// Client making a request
    Client,
    /// Producer sending a message
    Producer,
    /// Consumer receiving a message
    Consumer,
}

/// Span status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SpanStatus {
    /// Status not set
    Unset,
    /// Operation completed successfully
    Ok,
    /// Operation failed
    Error,
}

/// Event occurring during a span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    /// Event name
    pub name: String,
    /// Event timestamp
    pub timestamp: SystemTime,
    /// Event attributes
    pub attributes: HashMap<String, AttributeValue>,
}

/// Attribute value supporting multiple types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    /// String value
    String(String),
    /// Integer value
    Int(i64),
    /// Float value
    Float(f64),
    /// Boolean value
    Bool(bool),
    /// Array of strings
    StringArray(Vec<String>),
    /// Array of integers
    IntArray(Vec<i64>),
    /// Array of floats
    FloatArray(Vec<f64>),
    /// Array of booleans
    BoolArray(Vec<bool>),
}

impl From<String> for AttributeValue {
    fn from(s: String) -> Self {
        AttributeValue::String(s)
    }
}

impl From<&str> for AttributeValue {
    fn from(s: &str) -> Self {
        AttributeValue::String(s.to_string())
    }
}

impl From<i64> for AttributeValue {
    fn from(i: i64) -> Self {
        AttributeValue::Int(i)
    }
}

impl From<f64> for AttributeValue {
    fn from(f: f64) -> Self {
        AttributeValue::Float(f)
    }
}

impl From<bool> for AttributeValue {
    fn from(b: bool) -> Self {
        AttributeValue::Bool(b)
    }
}

/// Span storage for active spans
pub struct SpanStore {
    spans: Arc<RwLock<HashMap<SpanId, Span>>>,
}

impl SpanStore {
    /// Create a new span store
    pub fn new() -> Self {
        Self {
            spans: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a span to the store
    pub fn add(&self, span: Span) {
        self.spans.write().insert(span.context.span_id, span);
    }

    /// Get a span by ID
    pub fn get(&self, span_id: &SpanId) -> Option<Span> {
        self.spans.read().get(span_id).cloned()
    }

    /// Remove a span by ID
    pub fn remove(&self, span_id: &SpanId) -> Option<Span> {
        self.spans.write().remove(span_id)
    }

    /// Get all spans
    pub fn all(&self) -> Vec<Span> {
        self.spans.read().values().cloned().collect()
    }

    /// Get finished spans and remove them from the store
    pub fn collect_finished(&self) -> Vec<Span> {
        let mut spans = self.spans.write();
        let finished: Vec<SpanId> = spans
            .iter()
            .filter(|(_, span)| span.is_finished())
            .map(|(id, _)| *id)
            .collect();

        finished.into_iter().filter_map(|id| spans.remove(&id)).collect()
    }

    /// Clear all spans
    pub fn clear(&self) {
        self.spans.write().clear();
    }
}

impl Default for SpanStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Tracing errors
#[derive(Debug, thiserror::Error)]
pub enum TracingError {
    /// Invalid trace context
    #[error("Invalid trace context: {0}")]
    InvalidTraceContext(String),

    /// Span not found
    #[error("Span not found: {0}")]
    SpanNotFound(String),

    /// Generic error
    #[error("Tracing error: {0}")]
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_id_generation() {
        let id1 = TraceId::new();
        let id2 = TraceId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_trace_id_hex_conversion() {
        let id = TraceId::new();
        let hex = id.to_hex();
        assert_eq!(hex.len(), 32);
        let parsed = TraceId::from_hex(&hex).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_span_id_generation() {
        let id1 = SpanId::new();
        let id2 = SpanId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_span_id_hex_conversion() {
        let id = SpanId::new();
        let hex = id.to_hex();
        assert_eq!(hex.len(), 16);
        let parsed = SpanId::from_hex(&hex).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_span_context_creation() {
        let ctx = SpanContext::new_root();
        assert!(ctx.parent_span_id.is_none());
        assert!(ctx.is_sampled());
    }

    #[test]
    fn test_span_context_child() {
        let parent = SpanContext::new_root();
        let child = parent.child();
        assert_eq!(child.trace_id, parent.trace_id);
        assert_ne!(child.span_id, parent.span_id);
        assert_eq!(child.parent_span_id, Some(parent.span_id));
    }

    #[test]
    fn test_traceparent_format() {
        let ctx = SpanContext::new_root();
        let header = ctx.to_traceparent();
        assert!(header.starts_with("00-"));

        let parsed = SpanContext::from_traceparent(&header).unwrap();
        assert_eq!(parsed.trace_id, ctx.trace_id);
        assert_eq!(parsed.span_id, ctx.span_id);
    }

    #[test]
    fn test_trace_flags() {
        let mut flags = TraceFlags::default();
        assert!(flags.sampled);
        assert!(!flags.debug);

        flags.debug = true;
        let byte = flags.to_byte();
        let parsed = TraceFlags::from_byte(byte);
        assert_eq!(flags, parsed);
    }

    #[test]
    fn test_trace_state() {
        let mut state = TraceState::new();
        state.insert("vendor".to_string(), "value".to_string());

        let header = state.to_header();
        assert_eq!(header, "vendor=value");

        let parsed = TraceState::from_header(&header);
        assert_eq!(parsed.get("vendor"), Some(&"value".to_string()));
    }

    #[test]
    fn test_span_creation() {
        let span = Span::new("test-operation");
        assert_eq!(span.name, "test-operation");
        assert!(!span.is_finished());
        assert_eq!(span.status, SpanStatus::Unset);
    }

    #[test]
    fn test_span_child() {
        let parent = Span::new("parent");
        let child = parent.child("child");

        assert_eq!(child.context.trace_id, parent.context.trace_id);
        assert_ne!(child.context.span_id, parent.context.span_id);
        assert_eq!(child.context.parent_span_id, Some(parent.context.span_id));
    }

    #[test]
    fn test_span_attributes() {
        let mut span = Span::new("test");
        span.set_attribute("key", "value");
        span.set_attribute("count", 42);
        span.set_attribute("enabled", true);

        assert_eq!(span.attributes.len(), 3);
    }

    #[test]
    fn test_span_baggage() {
        let mut span = Span::new("test");
        span.set_baggage("user_id", "12345");

        assert_eq!(span.get_baggage("user_id"), Some(&"12345".to_string()));

        let child = span.child("child-op");
        assert_eq!(child.get_baggage("user_id"), Some(&"12345".to_string()));
    }

    #[test]
    fn test_span_events() {
        let mut span = Span::new("test");
        span.add_event("cache_miss");
        span.add_event("db_query");

        assert_eq!(span.events.len(), 2);
        assert_eq!(span.events[0].name, "cache_miss");
    }

    #[test]
    fn test_span_end() {
        let mut span = Span::new("test");
        assert!(!span.is_finished());

        span.end();
        assert!(span.is_finished());
        assert!(span.duration().is_some());
    }

    #[test]
    fn test_span_store() {
        let store = SpanStore::new();
        let span = Span::new("test");
        let span_id = span.context.span_id;

        store.add(span);
        assert!(store.get(&span_id).is_some());

        let removed = store.remove(&span_id);
        assert!(removed.is_some());
        assert!(store.get(&span_id).is_none());
    }

    #[test]
    fn test_span_store_collect_finished() {
        let store = SpanStore::new();

        let mut span1 = Span::new("finished");
        span1.end();
        store.add(span1);

        let span2 = Span::new("active");
        store.add(span2);

        let finished = store.collect_finished();
        assert_eq!(finished.len(), 1);
        assert_eq!(finished[0].name, "finished");

        // Active span should still be in store
        assert_eq!(store.all().len(), 1);
    }

    #[test]
    fn test_attribute_value_conversions() {
        let _str_val: AttributeValue = "test".into();
        let _int_val: AttributeValue = 42i64.into();
        let _float_val: AttributeValue = 3.14f64.into();
        let _bool_val: AttributeValue = true.into();
    }
}
