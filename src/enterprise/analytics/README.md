# CADDY Enterprise Performance Analytics System

## Overview

The CADDY Enterprise Performance Analytics system provides comprehensive performance monitoring, metrics collection, and analytics capabilities for enterprise CAD operations. This production-ready system includes thread-safe metrics collection, real-time aggregation, time-series storage, configurable dashboards, alerting, and comprehensive reporting.

## Features

### Core Capabilities

- **Thread-Safe Metrics Collection**: Lock-free counters, gauges, histograms, and summaries
- **Real-Time Aggregation**: Statistical analysis, percentiles, anomaly detection, and trend analysis
- **Time-Series Storage**: Multiple backend support (memory, file-based) with retention policies
- **Configurable Dashboards**: Pre-built and custom dashboard templates with real-time updates
- **Intelligent Alerting**: Rule-based alerts with simple and composite conditions
- **Comprehensive Reporting**: Scheduled and ad-hoc reports in multiple formats (HTML, CSV, JSON, Markdown)

### Metric Types

1. **Counter** - Monotonically increasing values (e.g., request counts, errors)
2. **Gauge** - Values that can increase or decrease (e.g., CPU usage, memory)
3. **Histogram** - Distribution tracking with buckets (e.g., latency distributions)
4. **Summary** - Quantile calculations (e.g., p50, p90, p95, p99)

### Collectors

- **SystemCollector**: CPU, memory, disk, network metrics
- **ApplicationCollector**: Operations, latency, errors, cache metrics
- **UserCollector**: Sessions, actions, feature usage, errors
- **RenderCollector**: FPS, draw calls, GPU usage, frame time

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     MetricRegistry                          │
│  (Central registry for all metrics)                         │
└────────────────────┬────────────────────────────────────────┘
                     │
         ┌───────────┼───────────┬──────────────┐
         │           │           │              │
    ┌────▼────┐ ┌───▼────┐ ┌───▼────┐   ┌─────▼──────┐
    │ Counter │ │ Gauge  │ │Histogram│   │  Summary   │
    └────┬────┘ └───┬────┘ └───┬────┘   └─────┬──────┘
         │          │          │              │
         └──────────┴──────────┴──────────────┘
                     │
         ┌───────────┴───────────┐
         │                       │
    ┌────▼────────┐      ┌──────▼──────┐
    │ Aggregator  │      │  Storage    │
    │  (Stats)    │      │ (Time-series)│
    └────┬────────┘      └──────┬──────┘
         │                      │
         └──────────┬───────────┘
                    │
         ┌──────────┴──────────┐
         │                     │
    ┌────▼────────┐    ┌──────▼──────┐
    │ Dashboard   │    │  Alerting   │
    └─────────────┘    └──────┬──────┘
                              │
                       ┌──────▼──────┐
                       │  Reporting  │
                       └─────────────┘
```

## Module Structure

```
analytics/
├── mod.rs           - Module exports and common types
├── metrics.rs       - Core metric types (Counter, Gauge, Histogram, Summary)
├── collector.rs     - Data collectors (System, App, User, Render)
├── aggregator.rs    - Statistical aggregation and analysis
├── storage.rs       - Time-series storage with retention policies
├── dashboard.rs     - Dashboard configuration and widgets
├── alerting.rs      - Alert rules and notification system
└── reporting.rs     - Report generation and export
```

## Quick Start

### Basic Metrics Collection

```rust
use caddy::enterprise::analytics::{MetricRegistry, Labels};

// Create a registry
let registry = MetricRegistry::new();

// Counter
let requests = registry.counter("http_requests_total", Labels::new());
requests.inc();

// Gauge
let cpu = registry.gauge("cpu_usage", Labels::new());
cpu.set(45.5);

// Histogram
let latency = registry.histogram("request_latency", Labels::new());
latency.observe(0.125);

// Summary
let response = registry.summary("response_time", Labels::new());
response.observe(100.0);
```

### System Monitoring

```rust
use caddy::enterprise::analytics::{MetricRegistry, SystemCollector};

let registry = MetricRegistry::new();
let collector = SystemCollector::new(&registry);
collector.start(); // Start background collection

// Metrics are automatically collected and updated
```

### Data Aggregation

```rust
use caddy::enterprise::analytics::{Aggregator, AggregationConfig, TimeWindow};

let config = AggregationConfig {
    window: TimeWindow::Hour,
    max_points: 1000,
    downsample: true,
    downsample_interval: 60,
};

let mut agg = Aggregator::new(config);
agg.add_value(42.5);

// Get statistics
let mean = agg.mean(None);
let median = agg.median(None);
let std_dev = agg.std_dev(None);
let percentiles = agg.percentiles(None);

// Detect anomalies
let anomalies = agg.detect_anomalies(None, 3.0);

// Analyze trends
let trend = agg.trend(None);
```

### Storage and Retention

```rust
use caddy::enterprise::analytics::{MetricStorage, RetentionPolicy, ExportFormat};

// Create storage
let mut storage = MetricStorage::new_file("/var/metrics")?;

// Write metrics
storage.write("cpu_usage", 45.5)?;

// Configure retention
let policies = vec![
    RetentionPolicy::new("raw", 3600, 1),      // 1 hour
    RetentionPolicy::new("1min", 86400, 60),   // 1 day
];
storage.set_policies(policies);

// Apply retention
storage.apply_retention()?;

// Export data
let csv = storage.export("cpu_usage", ExportFormat::Csv)?;
```

### Dashboards

```rust
use caddy::enterprise::analytics::{
    Dashboard, DashboardConfig, DashboardLayout, Widget, WidgetType
};

let config = DashboardConfig::new("System Monitor")
    .layout(DashboardLayout::Grid { rows: 2, cols: 2 })
    .time_range(3600);

let dashboard = Dashboard::new(config);

let widget = Widget::new("cpu", "CPU Usage", WidgetType::LineChart)
    .metric("system_cpu_usage_percent")
    .position(0, 0);

dashboard.add_widget(widget)?;

// Or use templates
let system_dash = DashboardTemplates::system_overview();
```

### Alerting

```rust
use caddy::enterprise::analytics::{
    AlertManager, AlertRule, AlertCondition, AlertSeverity, Comparator
};

let manager = AlertManager::new(registry);

let condition = AlertCondition::threshold(
    "cpu_usage",
    Comparator::GreaterThan,
    80.0
);

let rule = AlertRule::new(
    "cpu_high",
    "High CPU Usage",
    condition,
    AlertSeverity::Warning
)
.channel("email");

manager.add_rule(rule)?;

// Evaluate rules
let new_alerts = manager.evaluate();
```

### Reporting

```rust
use caddy::enterprise::analytics::{
    Reporter, ReportConfig, ReportFormat, ReportSchedule, ReportSection, TimeWindow
};

let mut reporter = Reporter::new(registry);

let config = ReportConfig::new("Daily Report", ReportFormat::Html)
    .schedule(ReportSchedule::Daily { hour: 9 })
    .section(ReportSection::Summary {
        metrics: vec!["cpu_usage".to_string()],
        window: TimeWindow::Day,
    })
    .recipient("admin@example.com");

let report = reporter.generate(config)?;
report.save("/var/reports/daily.html")?;
```

## Performance Considerations

### Thread Safety

All metric types use `Arc<RwLock<T>>` for thread-safe access:
- Multiple readers can access concurrently
- Writers get exclusive access
- No data races or undefined behavior

### Memory Usage

- Histograms: Fixed bucket count (configurable)
- Summaries: Configurable max values (default: 1000)
- Aggregators: Auto-downsampling when exceeding max points
- Storage: Automatic retention policy application

### CPU Usage

- Lock-free increments where possible
- Background collection threads for system metrics
- Efficient data structures (VecDeque, HashMap)
- Minimal overhead per metric operation

## Configuration Examples

### Complete System Setup

```rust
use caddy::enterprise::analytics::*;

// Create registry
let registry = MetricRegistry::new();

// Setup collectors
let collector_config = CollectorConfig::default();
let manager = CollectorManager::new(&registry, collector_config);
manager.start();

// Setup storage
let mut storage = MetricStorage::new_file("/var/metrics")?;
storage.set_policies(RetentionPolicy::defaults());

// Setup alerting
let alert_manager = AlertManager::new(registry.clone());

// Add alert rules
let cpu_alert = AlertRule::new(
    "cpu_critical",
    "Critical CPU",
    AlertCondition::threshold("cpu_usage", Comparator::GreaterThan, 90.0),
    AlertSeverity::Critical
);
alert_manager.add_rule(cpu_alert)?;

// Setup reporting
let mut reporter = Reporter::new(registry.clone());
let report_config = ReportTemplates::daily_system_performance();
```

## Best Practices

1. **Metric Naming**: Use consistent naming (e.g., `subsystem_metric_unit`)
2. **Labels**: Keep cardinality low (avoid user IDs, use user types)
3. **Sampling**: Use histograms/summaries for high-frequency events
4. **Retention**: Balance storage cost vs data granularity
5. **Alerting**: Set appropriate thresholds with hysteresis
6. **Dashboards**: Group related metrics, limit widget count
7. **Reporting**: Schedule during off-peak hours

## Testing

Run the test suite:

```bash
cargo test --lib --package caddy --features enterprise-analytics
```

Run specific module tests:

```bash
cargo test --lib analytics::metrics
cargo test --lib analytics::aggregator
cargo test --lib analytics::storage
```

## Examples

See `/examples/enterprise_analytics_demo.rs` for a comprehensive demonstration of all features.

Run the demo:

```bash
cargo run --example enterprise_analytics_demo
```

## API Documentation

Generate and view the API documentation:

```bash
cargo doc --open --features enterprise-analytics
```

## Troubleshooting

### High Memory Usage

- Reduce `max_values` in Summary metrics
- Decrease `max_points` in Aggregators
- Apply more aggressive retention policies

### Slow Performance

- Reduce metric cardinality (fewer unique label combinations)
- Increase collection intervals
- Use memory storage instead of file storage

### Missing Data

- Check retention policies
- Verify collector is running
- Check storage permissions

## License

CADDY Enterprise Edition - Commercial License Required

For licensing information, contact: enterprise@caddy-cad.com

## Support

- Documentation: https://docs.caddy-cad.com/analytics
- Email: support@caddy-cad.com
- Enterprise Support: enterprise@caddy-cad.com
