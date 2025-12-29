//! Trace export formats and protocols
//!
//! This module provides exporters for multiple tracing backends including
//! OpenTelemetry Protocol (OTLP), Jaeger, Zipkin, and console output.

use super::span::{Span, SpanKind, SpanStatus, AttributeValue};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::UNIX_EPOCH;
use tokio::sync::mpsc;

/// Trait for trace exporters
#[async_trait]
pub trait TraceExporter: Send + Sync {
    /// Export a batch of spans
    async fn export(&self, spans: Vec<Span>) -> Result<(), ExportError>;

    /// Flush any buffered spans
    async fn flush(&self) -> Result<(), ExportError>;

    /// Shutdown the exporter
    async fn shutdown(&self) -> Result<(), ExportError>;
}

/// Export errors
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    Config(String),

    /// Export failed
    #[error("Export failed: {0}")]
    Failed(String),

    /// Generic error
    #[error("Export error: {0}")]
    Other(String),
}

// ============================================================================
// OTLP Exporter (OpenTelemetry Protocol)
// ============================================================================

/// OpenTelemetry Protocol (OTLP) exporter
pub struct OtlpExporter {
    endpoint: String,
    headers: HashMap<String, String>,
    client: reqwest::Client,
    batch_size: usize,
}

impl OtlpExporter {
    /// Create a new OTLP exporter
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            headers: HashMap::new(),
            client: reqwest::Client::new(),
            batch_size: 100,
        }
    }

    /// Add a header (e.g., for authentication)
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set batch size
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Convert spans to OTLP format
    fn to_otlp_format(&self, spans: &[Span]) -> OtlpTraceData {
        let resource_spans = vec![OtlpResourceSpans {
            resource: OtlpResource {
                attributes: vec![
                    OtlpKeyValue {
                        key: "service.name".to_string(),
                        value: OtlpAnyValue::StringValue("caddy".to_string()),
                    },
                ],
            },
            scope_spans: vec![OtlpScopeSpans {
                scope: OtlpInstrumentationScope {
                    name: "caddy-tracing".to_string(),
                    version: "0.2.0".to_string(),
                },
                spans: spans.iter().map(|s| self.span_to_otlp(s)).collect(),
            }],
        }];

        OtlpTraceData { resource_spans }
    }

    fn span_to_otlp(&self, span: &Span) -> OtlpSpan {
        OtlpSpan {
            trace_id: span.context.trace_id.to_hex(),
            span_id: span.context.span_id.to_hex(),
            parent_span_id: span.context.parent_span_id.map(|id| id.to_hex()),
            name: span.name.clone(),
            kind: match span.kind {
                SpanKind::Internal => 1,
                SpanKind::Server => 2,
                SpanKind::Client => 3,
                SpanKind::Producer => 4,
                SpanKind::Consumer => 5,
            },
            start_time_unix_nano: span.start_time
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
            end_time_unix_nano: span.end_time
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0),
            attributes: span.attributes.iter().map(|(k, v)| {
                OtlpKeyValue {
                    key: k.clone(),
                    value: attribute_to_otlp(v),
                }
            }).collect(),
            events: span.events.iter().map(|e| {
                OtlpEvent {
                    time_unix_nano: e.timestamp
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos() as u64,
                    name: e.name.clone(),
                    attributes: e.attributes.iter().map(|(k, v)| {
                        OtlpKeyValue {
                            key: k.clone(),
                            value: attribute_to_otlp(v),
                        }
                    }).collect(),
                }
            }).collect(),
            status: OtlpStatus {
                code: match span.status {
                    SpanStatus::Unset => 0,
                    SpanStatus::Ok => 1,
                    SpanStatus::Error => 2,
                },
                message: String::new(),
            },
        }
    }
}

#[async_trait]
impl TraceExporter for OtlpExporter {
    async fn export(&self, spans: Vec<Span>) -> Result<(), ExportError> {
        if spans.is_empty() {
            return Ok(());
        }

        let data = self.to_otlp_format(&spans);
        let json = serde_json::to_string(&data)
            .map_err(|e| ExportError::Serialization(e.to_string()))?;

        let mut request = self.client
            .post(&format!("{}/v1/traces", self.endpoint))
            .header("Content-Type", "application/json");

        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        request
            .body(json)
            .send()
            .await
            .map_err(|e| ExportError::Network(e.to_string()))?;

        Ok(())
    }

    async fn flush(&self) -> Result<(), ExportError> {
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), ExportError> {
        Ok(())
    }
}

// OTLP data structures
#[derive(Debug, Serialize, Deserialize)]
struct OtlpTraceData {
    resource_spans: Vec<OtlpResourceSpans>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OtlpResourceSpans {
    resource: OtlpResource,
    scope_spans: Vec<OtlpScopeSpans>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OtlpResource {
    attributes: Vec<OtlpKeyValue>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OtlpScopeSpans {
    scope: OtlpInstrumentationScope,
    spans: Vec<OtlpSpan>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OtlpInstrumentationScope {
    name: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OtlpSpan {
    trace_id: String,
    span_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_span_id: Option<String>,
    name: String,
    kind: i32,
    start_time_unix_nano: u64,
    end_time_unix_nano: u64,
    attributes: Vec<OtlpKeyValue>,
    events: Vec<OtlpEvent>,
    status: OtlpStatus,
}

#[derive(Debug, Serialize, Deserialize)]
struct OtlpKeyValue {
    key: String,
    value: OtlpAnyValue,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum OtlpAnyValue {
    StringValue(String),
    IntValue(i64),
    DoubleValue(f64),
    BoolValue(bool),
}

#[derive(Debug, Serialize, Deserialize)]
struct OtlpEvent {
    time_unix_nano: u64,
    name: String,
    attributes: Vec<OtlpKeyValue>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OtlpStatus {
    code: i32,
    message: String,
}

fn attribute_to_otlp(attr: &AttributeValue) -> OtlpAnyValue {
    match attr {
        AttributeValue::String(s) => OtlpAnyValue::StringValue(s.clone()),
        AttributeValue::Int(i) => OtlpAnyValue::IntValue(*i),
        AttributeValue::Float(f) => OtlpAnyValue::DoubleValue(*f),
        AttributeValue::Bool(b) => OtlpAnyValue::BoolValue(*b),
        _ => OtlpAnyValue::StringValue(format!("{:?}", attr)),
    }
}

// ============================================================================
// Jaeger Exporter
// ============================================================================

/// Jaeger exporter (Thrift over HTTP)
pub struct JaegerExporter {
    endpoint: String,
    service_name: String,
    client: reqwest::Client,
}

impl JaegerExporter {
    /// Create a new Jaeger exporter
    pub fn new(endpoint: impl Into<String>, service_name: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            service_name: service_name.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Convert spans to Jaeger format
    fn to_jaeger_format(&self, spans: &[Span]) -> JaegerBatch {
        JaegerBatch {
            process: JaegerProcess {
                service_name: self.service_name.clone(),
                tags: vec![],
            },
            spans: spans.iter().map(|s| self.span_to_jaeger(s)).collect(),
        }
    }

    fn span_to_jaeger(&self, span: &Span) -> JaegerSpan {
        JaegerSpan {
            trace_id: span.context.trace_id.to_hex(),
            span_id: span.context.span_id.to_hex(),
            parent_span_id: span.context.parent_span_id.map(|id| id.to_hex()).unwrap_or_default(),
            operation_name: span.name.clone(),
            start_time: span.start_time
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_micros() as i64,
            duration: span.duration()
                .map(|d| d.as_micros() as i64)
                .unwrap_or(0),
            tags: span.attributes.iter().map(|(k, v)| {
                JaegerTag {
                    key: k.clone(),
                    value: format!("{:?}", v),
                    tag_type: "string".to_string(),
                }
            }).collect(),
            logs: span.events.iter().map(|e| {
                JaegerLog {
                    timestamp: e.timestamp
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_micros() as i64,
                    fields: vec![
                        JaegerTag {
                            key: "event".to_string(),
                            value: e.name.clone(),
                            tag_type: "string".to_string(),
                        },
                    ],
                }
            }).collect(),
        }
    }
}

#[async_trait]
impl TraceExporter for JaegerExporter {
    async fn export(&self, spans: Vec<Span>) -> Result<(), ExportError> {
        if spans.is_empty() {
            return Ok(());
        }

        let batch = self.to_jaeger_format(&spans);
        let json = serde_json::to_string(&batch)
            .map_err(|e| ExportError::Serialization(e.to_string()))?;

        self.client
            .post(&format!("{}/api/traces", self.endpoint))
            .header("Content-Type", "application/json")
            .body(json)
            .send()
            .await
            .map_err(|e| ExportError::Network(e.to_string()))?;

        Ok(())
    }

    async fn flush(&self) -> Result<(), ExportError> {
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), ExportError> {
        Ok(())
    }
}

// Jaeger data structures
#[derive(Debug, Serialize, Deserialize)]
struct JaegerBatch {
    process: JaegerProcess,
    spans: Vec<JaegerSpan>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JaegerProcess {
    #[serde(rename = "serviceName")]
    service_name: String,
    tags: Vec<JaegerTag>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JaegerSpan {
    #[serde(rename = "traceID")]
    trace_id: String,
    #[serde(rename = "spanID")]
    span_id: String,
    #[serde(rename = "parentSpanID")]
    parent_span_id: String,
    #[serde(rename = "operationName")]
    operation_name: String,
    #[serde(rename = "startTime")]
    start_time: i64,
    duration: i64,
    tags: Vec<JaegerTag>,
    logs: Vec<JaegerLog>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JaegerTag {
    key: String,
    #[serde(rename = "type")]
    tag_type: String,
    value: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JaegerLog {
    timestamp: i64,
    fields: Vec<JaegerTag>,
}

// ============================================================================
// Zipkin Exporter
// ============================================================================

/// Zipkin exporter (JSON v2 format)
pub struct ZipkinExporter {
    endpoint: String,
    service_name: String,
    client: reqwest::Client,
}

impl ZipkinExporter {
    /// Create a new Zipkin exporter
    pub fn new(endpoint: impl Into<String>, service_name: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            service_name: service_name.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Convert span to Zipkin format
    fn span_to_zipkin(&self, span: &Span) -> ZipkinSpan {
        ZipkinSpan {
            trace_id: span.context.trace_id.to_hex(),
            id: span.context.span_id.to_hex(),
            parent_id: span.context.parent_span_id.map(|id| id.to_hex()),
            name: span.name.clone(),
            kind: match span.kind {
                SpanKind::Client => Some("CLIENT".to_string()),
                SpanKind::Server => Some("SERVER".to_string()),
                SpanKind::Producer => Some("PRODUCER".to_string()),
                SpanKind::Consumer => Some("CONSUMER".to_string()),
                SpanKind::Internal => None,
            },
            timestamp: span.start_time
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_micros() as i64,
            duration: span.duration()
                .map(|d| d.as_micros() as i64),
            local_endpoint: ZipkinEndpoint {
                service_name: self.service_name.clone(),
            },
            tags: span.attributes.iter()
                .map(|(k, v)| (k.clone(), format!("{:?}", v)))
                .collect(),
            annotations: span.events.iter().map(|e| {
                ZipkinAnnotation {
                    timestamp: e.timestamp
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_micros() as i64,
                    value: e.name.clone(),
                }
            }).collect(),
        }
    }
}

#[async_trait]
impl TraceExporter for ZipkinExporter {
    async fn export(&self, spans: Vec<Span>) -> Result<(), ExportError> {
        if spans.is_empty() {
            return Ok(());
        }

        let zipkin_spans: Vec<ZipkinSpan> = spans.iter()
            .map(|s| self.span_to_zipkin(s))
            .collect();

        let json = serde_json::to_string(&zipkin_spans)
            .map_err(|e| ExportError::Serialization(e.to_string()))?;

        self.client
            .post(&format!("{}/api/v2/spans", self.endpoint))
            .header("Content-Type", "application/json")
            .body(json)
            .send()
            .await
            .map_err(|e| ExportError::Network(e.to_string()))?;

        Ok(())
    }

    async fn flush(&self) -> Result<(), ExportError> {
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), ExportError> {
        Ok(())
    }
}

// Zipkin data structures
#[derive(Debug, Serialize, Deserialize)]
struct ZipkinSpan {
    #[serde(rename = "traceId")]
    trace_id: String,
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "parentId")]
    parent_id: Option<String>,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<String>,
    timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration: Option<i64>,
    #[serde(rename = "localEndpoint")]
    local_endpoint: ZipkinEndpoint,
    tags: HashMap<String, String>,
    annotations: Vec<ZipkinAnnotation>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ZipkinEndpoint {
    #[serde(rename = "serviceName")]
    service_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ZipkinAnnotation {
    timestamp: i64,
    value: String,
}

// ============================================================================
// Console Exporter (for debugging)
// ============================================================================

/// Console exporter for debugging
pub struct ConsoleExporter {
    pretty_print: bool,
}

impl ConsoleExporter {
    /// Create a new console exporter
    pub fn new() -> Self {
        Self {
            pretty_print: true,
        }
    }

    /// Enable/disable pretty printing
    pub fn with_pretty_print(mut self, enabled: bool) -> Self {
        self.pretty_print = enabled;
        self
    }

    fn format_span(&self, span: &Span) -> String {
        let duration = span.duration()
            .map(|d| format!("{}ms", d.as_millis()))
            .unwrap_or_else(|| "active".to_string());

        let status = match span.status {
            SpanStatus::Ok => "✓",
            SpanStatus::Error => "✗",
            SpanStatus::Unset => "·",
        };

        format!(
            "[{}] {} {} (trace: {}, span: {}) [{}]",
            status,
            span.name,
            duration,
            span.context.trace_id,
            span.context.span_id,
            match span.kind {
                SpanKind::Internal => "internal",
                SpanKind::Server => "server",
                SpanKind::Client => "client",
                SpanKind::Producer => "producer",
                SpanKind::Consumer => "consumer",
            }
        )
    }
}

impl Default for ConsoleExporter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TraceExporter for ConsoleExporter {
    async fn export(&self, spans: Vec<Span>) -> Result<(), ExportError> {
        for span in spans {
            if self.pretty_print {
                println!("{}", self.format_span(&span));
                if !span.attributes.is_empty() {
                    println!("  Attributes:");
                    for (key, value) in &span.attributes {
                        println!("    {}: {:?}", key, value);
                    }
                }
                if !span.events.is_empty() {
                    println!("  Events:");
                    for event in &span.events {
                        println!("    - {}", event.name);
                    }
                }
            } else {
                let json = serde_json::to_string(&span)
                    .map_err(|e| ExportError::Serialization(e.to_string()))?;
                println!("{}", json);
            }
        }
        Ok(())
    }

    async fn flush(&self) -> Result<(), ExportError> {
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), ExportError> {
        Ok(())
    }
}

// ============================================================================
// Multi Exporter (fan-out to multiple backends)
// ============================================================================

/// Multi exporter that sends spans to multiple backends
pub struct MultiExporter {
    exporters: Vec<Box<dyn TraceExporter>>,
}

impl MultiExporter {
    /// Create a new multi exporter
    pub fn new() -> Self {
        Self {
            exporters: Vec::new(),
        }
    }

    /// Add an exporter
    pub fn add_exporter(mut self, exporter: Box<dyn TraceExporter>) -> Self {
        self.exporters.push(exporter);
        self
    }
}

impl Default for MultiExporter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TraceExporter for MultiExporter {
    async fn export(&self, spans: Vec<Span>) -> Result<(), ExportError> {
        let mut errors = Vec::new();

        for exporter in &self.exporters {
            if let Err(e) = exporter.export(spans.clone()).await {
                errors.push(e.to_string());
            }
        }

        if !errors.is_empty() {
            return Err(ExportError::Failed(errors.join("; ")));
        }

        Ok(())
    }

    async fn flush(&self) -> Result<(), ExportError> {
        for exporter in &self.exporters {
            exporter.flush().await?;
        }
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), ExportError> {
        for exporter in &self.exporters {
            exporter.shutdown().await?;
        }
        Ok(())
    }
}

// ============================================================================
// Batch Exporter (batches spans before exporting)
// ============================================================================

/// Batch exporter that accumulates spans before exporting
pub struct BatchExporter {
    inner: Box<dyn TraceExporter>,
    tx: mpsc::UnboundedSender<Span>,
}

impl BatchExporter {
    /// Create a new batch exporter
    pub fn new(
        exporter: Box<dyn TraceExporter>,
        max_batch_size: usize,
        max_delay: std::time::Duration,
    ) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<Span>();
        let exporter_clone = exporter;

        tokio::spawn(async move {
            let mut batch = Vec::new();
            let mut deadline = tokio::time::Instant::now() + max_delay;

            loop {
                tokio::select! {
                    Some(span) = rx.recv() => {
                        batch.push(span);

                        if batch.len() >= max_batch_size {
                            if let Err(e) = exporter_clone.export(batch.clone()).await {
                                eprintln!("Export error: {}", e);
                            }
                            batch.clear();
                            deadline = tokio::time::Instant::now() + max_delay;
                        }
                    }
                    _ = tokio::time::sleep_until(deadline) => {
                        if !batch.is_empty() {
                            if let Err(e) = exporter_clone.export(batch.clone()).await {
                                eprintln!("Export error: {}", e);
                            }
                            batch.clear();
                        }
                        deadline = tokio::time::Instant::now() + max_delay;
                    }
                }
            }
        });

        Self {
            inner: Box::new(ConsoleExporter::new()), // Placeholder
            tx,
        }
    }
}

#[async_trait]
impl TraceExporter for BatchExporter {
    async fn export(&self, spans: Vec<Span>) -> Result<(), ExportError> {
        for span in spans {
            self.tx.send(span)
                .map_err(|e| ExportError::Failed(e.to_string()))?;
        }
        Ok(())
    }

    async fn flush(&self) -> Result<(), ExportError> {
        // Wait a bit to allow pending spans to be exported
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), ExportError> {
        self.flush().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enterprise::tracing::span::Span;

    #[tokio::test]
    async fn test_console_exporter() {
        let exporter = ConsoleExporter::new();
        let span = Span::new("test-operation");

        let result = exporter.export(vec![span]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_console_exporter_with_attributes() {
        let exporter = ConsoleExporter::new();
        let mut span = Span::new("test-operation");
        span.set_attribute("user_id", "12345");
        span.set_attribute("status", "success");

        let result = exporter.export(vec![span]).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_otlp_exporter_creation() {
        let exporter = OtlpExporter::new("http://localhost:4318")
            .with_header("Authorization", "Bearer token")
            .with_batch_size(50);

        assert_eq!(exporter.endpoint, "http://localhost:4318");
        assert_eq!(exporter.batch_size, 50);
    }

    #[test]
    fn test_jaeger_exporter_creation() {
        let exporter = JaegerExporter::new("http://localhost:14268", "caddy");
        assert_eq!(exporter.service_name, "caddy");
    }

    #[test]
    fn test_zipkin_exporter_creation() {
        let exporter = ZipkinExporter::new("http://localhost:9411", "caddy");
        assert_eq!(exporter.service_name, "caddy");
    }

    #[tokio::test]
    async fn test_multi_exporter() {
        let console = Box::new(ConsoleExporter::new());
        let multi = MultiExporter::new().add_exporter(console);

        let span = Span::new("test");
        let result = multi.export(vec![span]).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_otlp_span_conversion() {
        let exporter = OtlpExporter::new("http://localhost:4318");
        let mut span = Span::new("test-operation");
        span.set_attribute("key", "value");

        let otlp_span = exporter.span_to_otlp(&span);
        assert_eq!(otlp_span.name, "test-operation");
        assert_eq!(otlp_span.attributes.len(), 1);
    }

    #[test]
    fn test_jaeger_span_conversion() {
        let exporter = JaegerExporter::new("http://localhost:14268", "test-service");
        let span = Span::new("test-operation");

        let jaeger_span = exporter.span_to_jaeger(&span);
        assert_eq!(jaeger_span.operation_name, "test-operation");
    }

    #[test]
    fn test_zipkin_span_conversion() {
        let exporter = ZipkinExporter::new("http://localhost:9411", "test-service");
        let span = Span::new("test-operation");

        let zipkin_span = exporter.span_to_zipkin(&span);
        assert_eq!(zipkin_span.name, "test-operation");
        assert_eq!(zipkin_span.local_endpoint.service_name, "test-service");
    }

    #[test]
    fn test_attribute_to_otlp() {
        let str_val = AttributeValue::String("test".to_string());
        let otlp_val = attribute_to_otlp(&str_val);
        assert!(matches!(otlp_val, OtlpAnyValue::StringValue(_)));

        let int_val = AttributeValue::Int(42);
        let otlp_val = attribute_to_otlp(&int_val);
        assert!(matches!(otlp_val, OtlpAnyValue::IntValue(42)));
    }
}
