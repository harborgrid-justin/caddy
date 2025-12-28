//! Dashboard configuration and management
//!
//! This module provides dashboard creation, widget types,
//! and real-time data updates.

use super::{MetricRegistry, Result, AnalyticsError};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Widget types for dashboards
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WidgetType {
    /// Line chart for time-series data
    LineChart,
    /// Bar chart for comparisons
    BarChart,
    /// Pie chart for proportions
    PieChart,
    /// Gauge for single values
    Gauge,
    /// Counter display
    Counter,
    /// Table for detailed data
    Table,
    /// Heatmap for density visualization
    Heatmap,
    /// Text display
    Text,
}

/// Widget refresh interval
#[derive(Debug, Clone, Copy)]
pub enum RefreshInterval {
    /// No automatic refresh
    None,
    /// Real-time updates (1 second)
    RealTime,
    /// Every 5 seconds
    FiveSeconds,
    /// Every 30 seconds
    ThirtySeconds,
    /// Every minute
    Minute,
    /// Every 5 minutes
    FiveMinutes,
    /// Custom interval in seconds
    Custom(u64),
}

impl RefreshInterval {
    /// Get interval in seconds
    pub fn as_secs(&self) -> Option<u64> {
        match self {
            Self::None => None,
            Self::RealTime => Some(1),
            Self::FiveSeconds => Some(5),
            Self::ThirtySeconds => Some(30),
            Self::Minute => Some(60),
            Self::FiveMinutes => Some(300),
            Self::Custom(secs) => Some(*secs),
        }
    }
}

/// Dashboard widget configuration
#[derive(Debug, Clone)]
pub struct Widget {
    /// Widget ID
    pub id: String,
    /// Widget title
    pub title: String,
    /// Widget type
    pub widget_type: WidgetType,
    /// Metrics to display
    pub metrics: Vec<String>,
    /// Refresh interval
    pub refresh_interval: RefreshInterval,
    /// Position (row, column)
    pub position: (usize, usize),
    /// Size (rows, columns)
    pub size: (usize, usize),
    /// Custom configuration
    pub config: HashMap<String, String>,
}

impl Widget {
    /// Create a new widget
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        widget_type: WidgetType,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            widget_type,
            metrics: Vec::new(),
            refresh_interval: RefreshInterval::FiveSeconds,
            position: (0, 0),
            size: (1, 1),
            config: HashMap::new(),
        }
    }

    /// Add a metric to track
    pub fn metric(mut self, metric: impl Into<String>) -> Self {
        self.metrics.push(metric.into());
        self
    }

    /// Set refresh interval
    pub fn refresh(mut self, interval: RefreshInterval) -> Self {
        self.refresh_interval = interval;
        self
    }

    /// Set position
    pub fn position(mut self, row: usize, col: usize) -> Self {
        self.position = (row, col);
        self
    }

    /// Set size
    pub fn size(mut self, rows: usize, cols: usize) -> Self {
        self.size = (rows, cols);
        self
    }

    /// Add custom configuration
    pub fn config(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.insert(key.into(), value.into());
        self
    }

    /// Get configuration value
    pub fn get_config(&self, key: &str) -> Option<&str> {
        self.config.get(key).map(|s| s.as_str())
    }
}

/// Dashboard layout
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashboardLayout {
    /// Grid layout
    Grid { rows: usize, cols: usize },
    /// Flexible flow layout
    Flow,
    /// Custom layout
    Custom,
}

/// Dashboard configuration
#[derive(Debug, Clone)]
pub struct DashboardConfig {
    /// Dashboard name
    pub name: String,
    /// Dashboard description
    pub description: String,
    /// Layout type
    pub layout: DashboardLayout,
    /// Time range (in seconds from now)
    pub time_range: u64,
    /// Auto-refresh enabled
    pub auto_refresh: bool,
    /// Theme (light/dark)
    pub theme: String,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            name: "Dashboard".to_string(),
            description: String::new(),
            layout: DashboardLayout::Grid { rows: 3, cols: 3 },
            time_range: 3600,
            auto_refresh: true,
            theme: "light".to_string(),
        }
    }
}

impl DashboardConfig {
    /// Create a new dashboard configuration
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set layout
    pub fn layout(mut self, layout: DashboardLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Set time range
    pub fn time_range(mut self, seconds: u64) -> Self {
        self.time_range = seconds;
        self
    }

    /// Set auto-refresh
    pub fn auto_refresh(mut self, enabled: bool) -> Self {
        self.auto_refresh = enabled;
        self
    }

    /// Set theme
    pub fn theme(mut self, theme: impl Into<String>) -> Self {
        self.theme = theme.into();
        self
    }
}

/// Dashboard with widgets and configuration
pub struct Dashboard {
    config: DashboardConfig,
    widgets: Arc<RwLock<Vec<Widget>>>,
    registry: Option<MetricRegistry>,
}

impl Dashboard {
    /// Create a new dashboard
    pub fn new(config: DashboardConfig) -> Self {
        Self {
            config,
            widgets: Arc::new(RwLock::new(Vec::new())),
            registry: None,
        }
    }

    /// Create a dashboard with metric registry
    pub fn with_registry(config: DashboardConfig, registry: MetricRegistry) -> Self {
        Self {
            config,
            widgets: Arc::new(RwLock::new(Vec::new())),
            registry: Some(registry),
        }
    }

    /// Add a widget to the dashboard
    pub fn add_widget(&self, widget: Widget) -> Result<()> {
        let mut widgets = self.widgets.write().unwrap();

        // Check for duplicate IDs
        if widgets.iter().any(|w| w.id == widget.id) {
            return Err(AnalyticsError::InvalidConfig(format!(
                "Widget with ID '{}' already exists",
                widget.id
            )));
        }

        widgets.push(widget);
        Ok(())
    }

    /// Remove a widget by ID
    pub fn remove_widget(&self, id: &str) -> Result<()> {
        let mut widgets = self.widgets.write().unwrap();
        let original_len = widgets.len();
        widgets.retain(|w| w.id != id);

        if widgets.len() == original_len {
            Err(AnalyticsError::InvalidConfig(format!(
                "Widget '{}' not found",
                id
            )))
        } else {
            Ok(())
        }
    }

    /// Get a widget by ID
    pub fn get_widget(&self, id: &str) -> Option<Widget> {
        let widgets = self.widgets.read().unwrap();
        widgets.iter().find(|w| w.id == id).cloned()
    }

    /// Get all widgets
    pub fn widgets(&self) -> Vec<Widget> {
        self.widgets.read().unwrap().clone()
    }

    /// Get dashboard configuration
    pub fn config(&self) -> &DashboardConfig {
        &self.config
    }

    /// Update widget configuration
    pub fn update_widget<F>(&self, id: &str, update: F) -> Result<()>
    where
        F: FnOnce(&mut Widget),
    {
        let mut widgets = self.widgets.write().unwrap();
        let widget = widgets
            .iter_mut()
            .find(|w| w.id == id)
            .ok_or_else(|| {
                AnalyticsError::InvalidConfig(format!("Widget '{}' not found", id))
            })?;

        update(widget);
        Ok(())
    }

    /// Get widget data from registry
    pub fn get_widget_data(&self, widget_id: &str) -> Result<WidgetData> {
        let widget = self.get_widget(widget_id).ok_or_else(|| {
            AnalyticsError::InvalidConfig(format!("Widget '{}' not found", widget_id))
        })?;

        let registry = self.registry.as_ref().ok_or_else(|| {
            AnalyticsError::InvalidConfig("No metric registry configured".to_string())
        })?;

        let mut data = WidgetData {
            widget_id: widget.id.clone(),
            title: widget.title.clone(),
            widget_type: widget.widget_type.clone(),
            values: HashMap::new(),
        };

        // Collect metric values
        for metric_name in &widget.metrics {
            if let Some(metric) = registry.get(metric_name) {
                let value = match metric {
                    super::metrics::Metric::Counter(c) => c.get(),
                    super::metrics::Metric::Gauge(g) => g.get(),
                    super::metrics::Metric::Histogram(h) => h.mean(),
                    super::metrics::Metric::Summary(s) => s.mean(),
                };
                data.values.insert(metric_name.clone(), value);
            }
        }

        Ok(data)
    }

    /// Export dashboard configuration as JSON
    pub fn export_config(&self) -> String {
        let widgets = self.widgets.read().unwrap();
        format!(
            r#"{{
  "name": "{}",
  "description": "{}",
  "time_range": {},
  "auto_refresh": {},
  "widget_count": {}
}}"#,
            self.config.name,
            self.config.description,
            self.config.time_range,
            self.config.auto_refresh,
            widgets.len()
        )
    }
}

/// Widget data for rendering
#[derive(Debug, Clone)]
pub struct WidgetData {
    pub widget_id: String,
    pub title: String,
    pub widget_type: WidgetType,
    pub values: HashMap<String, f64>,
}

/// Pre-configured dashboard templates
pub struct DashboardTemplates;

impl DashboardTemplates {
    /// Create a system overview dashboard
    pub fn system_overview() -> Dashboard {
        let config = DashboardConfig::new("System Overview")
            .description("System performance metrics")
            .layout(DashboardLayout::Grid { rows: 2, cols: 3 })
            .time_range(3600);

        let dashboard = Dashboard::new(config);

        // CPU widget
        let cpu_widget = Widget::new("cpu", "CPU Usage", WidgetType::LineChart)
            .metric("system_cpu_usage_percent")
            .position(0, 0)
            .size(1, 1)
            .refresh(RefreshInterval::FiveSeconds);
        dashboard.add_widget(cpu_widget).ok();

        // Memory widget
        let mem_widget = Widget::new("memory", "Memory Usage", WidgetType::LineChart)
            .metric("system_memory_usage_bytes")
            .position(0, 1)
            .size(1, 1)
            .refresh(RefreshInterval::FiveSeconds);
        dashboard.add_widget(mem_widget).ok();

        // Disk widget
        let disk_widget = Widget::new("disk", "Disk Usage", WidgetType::Gauge)
            .metric("system_disk_usage_bytes")
            .position(0, 2)
            .size(1, 1)
            .refresh(RefreshInterval::Minute);
        dashboard.add_widget(disk_widget).ok();

        // Network RX
        let net_rx_widget = Widget::new("network_rx", "Network RX", WidgetType::LineChart)
            .metric("system_network_rx_bytes")
            .position(1, 0)
            .size(1, 1)
            .refresh(RefreshInterval::FiveSeconds);
        dashboard.add_widget(net_rx_widget).ok();

        // Network TX
        let net_tx_widget = Widget::new("network_tx", "Network TX", WidgetType::LineChart)
            .metric("system_network_tx_bytes")
            .position(1, 1)
            .size(1, 1)
            .refresh(RefreshInterval::FiveSeconds);
        dashboard.add_widget(net_tx_widget).ok();

        dashboard
    }

    /// Create an application performance dashboard
    pub fn application_performance() -> Dashboard {
        let config = DashboardConfig::new("Application Performance")
            .description("Application metrics and performance")
            .layout(DashboardLayout::Grid { rows: 2, cols: 2 })
            .time_range(3600);

        let dashboard = Dashboard::new(config);

        // Operations counter
        let ops_widget = Widget::new("operations", "Total Operations", WidgetType::Counter)
            .metric("app_operations_total")
            .position(0, 0)
            .size(1, 1)
            .refresh(RefreshInterval::RealTime);
        dashboard.add_widget(ops_widget).ok();

        // Error rate
        let errors_widget = Widget::new("errors", "Failed Operations", WidgetType::Counter)
            .metric("app_operations_failed")
            .position(0, 1)
            .size(1, 1)
            .refresh(RefreshInterval::RealTime);
        dashboard.add_widget(errors_widget).ok();

        // Operation duration
        let duration_widget = Widget::new("duration", "Operation Duration", WidgetType::LineChart)
            .metric("app_operation_duration_seconds")
            .position(1, 0)
            .size(1, 2)
            .refresh(RefreshInterval::FiveSeconds);
        dashboard.add_widget(duration_widget).ok();

        dashboard
    }

    /// Create a render performance dashboard
    pub fn render_performance() -> Dashboard {
        let config = DashboardConfig::new("Render Performance")
            .description("Rendering and GPU metrics")
            .layout(DashboardLayout::Grid { rows: 2, cols: 2 })
            .time_range(600);

        let dashboard = Dashboard::new(config);

        // FPS gauge
        let fps_widget = Widget::new("fps", "FPS", WidgetType::Gauge)
            .metric("render_fps")
            .position(0, 0)
            .size(1, 1)
            .refresh(RefreshInterval::RealTime);
        dashboard.add_widget(fps_widget).ok();

        // Draw calls
        let draw_widget = Widget::new("draw_calls", "Draw Calls", WidgetType::LineChart)
            .metric("render_draw_calls_total")
            .position(0, 1)
            .size(1, 1)
            .refresh(RefreshInterval::RealTime);
        dashboard.add_widget(draw_widget).ok();

        // GPU memory
        let gpu_widget = Widget::new("gpu_memory", "GPU Memory", WidgetType::Gauge)
            .metric("render_gpu_memory_bytes")
            .position(1, 0)
            .size(1, 1)
            .refresh(RefreshInterval::FiveSeconds);
        dashboard.add_widget(gpu_widget).ok();

        // Frame time
        let frame_widget = Widget::new("frame_time", "Frame Time", WidgetType::LineChart)
            .metric("render_frame_time_seconds")
            .position(1, 1)
            .size(1, 1)
            .refresh(RefreshInterval::RealTime);
        dashboard.add_widget(frame_widget).ok();

        dashboard
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_builder() {
        let widget = Widget::new("test", "Test Widget", WidgetType::LineChart)
            .metric("cpu")
            .metric("memory")
            .position(1, 2)
            .size(2, 3)
            .refresh(RefreshInterval::FiveSeconds)
            .config("color", "blue");

        assert_eq!(widget.id, "test");
        assert_eq!(widget.metrics.len(), 2);
        assert_eq!(widget.position, (1, 2));
        assert_eq!(widget.size, (2, 3));
        assert_eq!(widget.get_config("color"), Some("blue"));
    }

    #[test]
    fn test_dashboard_widgets() {
        let config = DashboardConfig::new("Test");
        let dashboard = Dashboard::new(config);

        let widget = Widget::new("w1", "Widget 1", WidgetType::Gauge);
        dashboard.add_widget(widget).unwrap();

        assert_eq!(dashboard.widgets().len(), 1);

        dashboard.remove_widget("w1").unwrap();
        assert_eq!(dashboard.widgets().len(), 0);
    }

    #[test]
    fn test_dashboard_templates() {
        let system_dash = DashboardTemplates::system_overview();
        assert!(!system_dash.widgets().is_empty());

        let app_dash = DashboardTemplates::application_performance();
        assert!(!app_dash.widgets().is_empty());

        let render_dash = DashboardTemplates::render_performance();
        assert!(!render_dash.widgets().is_empty());
    }

    #[test]
    fn test_refresh_interval() {
        assert_eq!(RefreshInterval::RealTime.as_secs(), Some(1));
        assert_eq!(RefreshInterval::FiveSeconds.as_secs(), Some(5));
        assert_eq!(RefreshInterval::Minute.as_secs(), Some(60));
        assert_eq!(RefreshInterval::None.as_secs(), None);
    }
}
