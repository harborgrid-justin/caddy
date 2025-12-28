//! Data collectors for various metric types
//!
//! This module provides collectors for system, application, user, and render metrics.

use super::metrics::{MetricRegistry, Labels, Counter, Gauge, Histogram};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;

/// Collector configuration
#[derive(Debug, Clone)]
pub struct CollectorConfig {
    /// Collection interval in seconds
    pub interval: u64,
    /// Enable system metrics
    pub system_metrics: bool,
    /// Enable application metrics
    pub app_metrics: bool,
    /// Enable user metrics
    pub user_metrics: bool,
    /// Enable render metrics
    pub render_metrics: bool,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            interval: 60,
            system_metrics: true,
            app_metrics: true,
            user_metrics: true,
            render_metrics: true,
        }
    }
}

/// System metrics collector
#[derive(Clone)]
pub struct SystemCollector {
    registry: MetricRegistry,
    running: Arc<AtomicBool>,
    cpu_usage: Gauge,
    memory_usage: Gauge,
    disk_usage: Gauge,
    network_rx: Counter,
    network_tx: Counter,
}

impl SystemCollector {
    /// Create a new system collector
    pub fn new(registry: &MetricRegistry) -> Self {
        let labels = Labels::new();

        Self {
            registry: registry.clone(),
            running: Arc::new(AtomicBool::new(false)),
            cpu_usage: registry.gauge("system_cpu_usage_percent", labels.clone()),
            memory_usage: registry.gauge("system_memory_usage_bytes", labels.clone()),
            disk_usage: registry.gauge("system_disk_usage_bytes", labels.clone()),
            network_rx: registry.counter("system_network_rx_bytes", labels.clone()),
            network_tx: registry.counter("system_network_tx_bytes", labels),
        }
    }

    /// Start collecting system metrics
    pub fn start(&self) {
        self.start_with_interval(Duration::from_secs(60))
    }

    /// Start collecting with custom interval
    pub fn start_with_interval(&self, interval: Duration) {
        if self.running.swap(true, Ordering::SeqCst) {
            return; // Already running
        }

        let collector = self.clone();
        thread::spawn(move || {
            while collector.running.load(Ordering::SeqCst) {
                collector.collect();
                thread::sleep(interval);
            }
        });
    }

    /// Stop collecting
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Collect current system metrics
    pub fn collect(&self) {
        // CPU usage (simulated - in production, use sysinfo crate)
        let cpu = self.get_cpu_usage();
        self.cpu_usage.set(cpu);

        // Memory usage
        let memory = self.get_memory_usage();
        self.memory_usage.set(memory);

        // Disk usage
        let disk = self.get_disk_usage();
        self.disk_usage.set(disk);

        // Network stats
        let (rx, tx) = self.get_network_stats();
        self.network_rx.add(rx);
        self.network_tx.add(tx);
    }

    fn get_cpu_usage(&self) -> f64 {
        // In production, use sysinfo::System or similar
        // This is a placeholder
        50.0
    }

    fn get_memory_usage(&self) -> f64 {
        // In production, use sysinfo::System
        1024.0 * 1024.0 * 512.0 // 512 MB
    }

    fn get_disk_usage(&self) -> f64 {
        // In production, use sysinfo::Disks
        1024.0 * 1024.0 * 1024.0 * 10.0 // 10 GB
    }

    fn get_network_stats(&self) -> (f64, f64) {
        // In production, use sysinfo::Networks
        (1024.0 * 100.0, 1024.0 * 50.0) // 100KB RX, 50KB TX
    }
}

/// Application metrics collector
#[derive(Clone)]
pub struct ApplicationCollector {
    registry: MetricRegistry,
    operations_total: Counter,
    operations_failed: Counter,
    operation_duration: Histogram,
    active_sessions: Gauge,
    cache_hits: Counter,
    cache_misses: Counter,
}

impl ApplicationCollector {
    /// Create a new application collector
    pub fn new(registry: &MetricRegistry) -> Self {
        let labels = Labels::new();

        Self {
            registry: registry.clone(),
            operations_total: registry.counter("app_operations_total", labels.clone()),
            operations_failed: registry.counter("app_operations_failed", labels.clone()),
            operation_duration: registry.histogram("app_operation_duration_seconds", labels.clone()),
            active_sessions: registry.gauge("app_active_sessions", labels.clone()),
            cache_hits: registry.counter("app_cache_hits", labels.clone()),
            cache_misses: registry.counter("app_cache_misses", labels),
        }
    }

    /// Record an operation
    pub fn record_operation(&self, duration: Duration, success: bool) {
        self.operations_total.inc();
        if !success {
            self.operations_failed.inc();
        }
        self.operation_duration.observe(duration.as_secs_f64());
    }

    /// Record operation with labels
    pub fn record_operation_labeled(
        &self,
        operation_type: &str,
        duration: Duration,
        success: bool,
    ) {
        let mut labels = Labels::new();
        labels.add("operation", operation_type);
        labels.add("success", success.to_string());

        let counter = self.registry.counter("app_operations_by_type", labels.clone());
        counter.inc();

        let histogram = self.registry.histogram("app_operation_duration_by_type", labels);
        histogram.observe(duration.as_secs_f64());
    }

    /// Update active sessions
    pub fn set_active_sessions(&self, count: f64) {
        self.active_sessions.set(count);
    }

    /// Record cache hit
    pub fn record_cache_hit(&self) {
        self.cache_hits.inc();
    }

    /// Record cache miss
    pub fn record_cache_miss(&self) {
        self.cache_misses.inc();
    }

    /// Get cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits.get();
        let misses = self.cache_misses.get();
        let total = hits + misses;
        if total > 0.0 {
            hits / total
        } else {
            0.0
        }
    }
}

/// User metrics collector
#[derive(Clone)]
pub struct UserCollector {
    registry: MetricRegistry,
    user_sessions: Counter,
    user_actions: Counter,
    features_used: Counter,
    error_encountered: Counter,
}

impl UserCollector {
    /// Create a new user collector
    pub fn new(registry: &MetricRegistry) -> Self {
        let labels = Labels::new();

        Self {
            registry: registry.clone(),
            user_sessions: registry.counter("user_sessions_total", labels.clone()),
            user_actions: registry.counter("user_actions_total", labels.clone()),
            features_used: registry.counter("user_features_used_total", labels.clone()),
            error_encountered: registry.counter("user_errors_total", labels),
        }
    }

    /// Record a new user session
    pub fn record_session(&self, user_id: &str) {
        let mut labels = Labels::new();
        labels.add("user_id", user_id);

        let counter = self.registry.counter("user_sessions", labels);
        counter.inc();

        self.user_sessions.inc();
    }

    /// Record a user action
    pub fn record_action(&self, action: &str, user_id: &str) {
        let mut labels = Labels::new();
        labels.add("action", action);
        labels.add("user_id", user_id);

        let counter = self.registry.counter("user_actions", labels);
        counter.inc();

        self.user_actions.inc();
    }

    /// Record feature usage
    pub fn record_feature_use(&self, feature: &str, user_id: &str) {
        let mut labels = Labels::new();
        labels.add("feature", feature);
        labels.add("user_id", user_id);

        let counter = self.registry.counter("feature_usage", labels);
        counter.inc();

        self.features_used.inc();
    }

    /// Record user error
    pub fn record_error(&self, error_type: &str, user_id: &str) {
        let mut labels = Labels::new();
        labels.add("error_type", error_type);
        labels.add("user_id", user_id);

        let counter = self.registry.counter("user_errors", labels);
        counter.inc();

        self.error_encountered.inc();
    }

    /// Get total sessions
    pub fn total_sessions(&self) -> f64 {
        self.user_sessions.get()
    }

    /// Get total actions
    pub fn total_actions(&self) -> f64 {
        self.user_actions.get()
    }
}

/// Render metrics collector
#[derive(Clone)]
pub struct RenderCollector {
    registry: MetricRegistry,
    running: Arc<AtomicBool>,
    fps: Gauge,
    draw_calls: Counter,
    triangles_rendered: Counter,
    gpu_memory_usage: Gauge,
    frame_time: Histogram,
}

impl RenderCollector {
    /// Create a new render collector
    pub fn new(registry: &MetricRegistry) -> Self {
        let labels = Labels::new();

        Self {
            registry: registry.clone(),
            running: Arc::new(AtomicBool::new(false)),
            fps: registry.gauge("render_fps", labels.clone()),
            draw_calls: registry.counter("render_draw_calls_total", labels.clone()),
            triangles_rendered: registry.counter("render_triangles_total", labels.clone()),
            gpu_memory_usage: registry.gauge("render_gpu_memory_bytes", labels.clone()),
            frame_time: registry.histogram("render_frame_time_seconds", labels),
        }
    }

    /// Start collecting render metrics
    pub fn start(&self) {
        self.start_with_interval(Duration::from_secs(1))
    }

    /// Start collecting with custom interval
    pub fn start_with_interval(&self, interval: Duration) {
        if self.running.swap(true, Ordering::SeqCst) {
            return;
        }

        let collector = self.clone();
        thread::spawn(move || {
            while collector.running.load(Ordering::SeqCst) {
                collector.collect();
                thread::sleep(interval);
            }
        });
    }

    /// Stop collecting
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Collect current render metrics
    pub fn collect(&self) {
        let fps = self.get_current_fps();
        self.fps.set(fps);

        let gpu_mem = self.get_gpu_memory();
        self.gpu_memory_usage.set(gpu_mem);
    }

    /// Record a frame
    pub fn record_frame(&self, duration: Duration, draw_calls: u64, triangles: u64) {
        self.frame_time.observe(duration.as_secs_f64());
        self.draw_calls.add(draw_calls as f64);
        self.triangles_rendered.add(triangles as f64);
    }

    /// Update FPS
    pub fn update_fps(&self, fps: f64) {
        self.fps.set(fps);
    }

    fn get_current_fps(&self) -> f64 {
        // In production, calculate from actual frame times
        60.0
    }

    fn get_gpu_memory(&self) -> f64 {
        // In production, query actual GPU memory usage
        1024.0 * 1024.0 * 256.0 // 256 MB
    }
}

/// Combined collector manager
pub struct CollectorManager {
    system: Option<SystemCollector>,
    application: ApplicationCollector,
    user: UserCollector,
    render: Option<RenderCollector>,
    config: CollectorConfig,
}

impl CollectorManager {
    /// Create a new collector manager
    pub fn new(registry: &MetricRegistry, config: CollectorConfig) -> Self {
        let system = if config.system_metrics {
            Some(SystemCollector::new(registry))
        } else {
            None
        };

        let render = if config.render_metrics {
            Some(RenderCollector::new(registry))
        } else {
            None
        };

        Self {
            system,
            application: ApplicationCollector::new(registry),
            user: UserCollector::new(registry),
            render,
            config,
        }
    }

    /// Start all enabled collectors
    pub fn start(&self) {
        let interval = Duration::from_secs(self.config.interval);

        if let Some(ref system) = self.system {
            system.start_with_interval(interval);
        }

        if let Some(ref render) = self.render {
            render.start_with_interval(Duration::from_secs(1));
        }
    }

    /// Stop all collectors
    pub fn stop(&self) {
        if let Some(ref system) = self.system {
            system.stop();
        }

        if let Some(ref render) = self.render {
            render.stop();
        }
    }

    /// Get system collector
    pub fn system(&self) -> Option<&SystemCollector> {
        self.system.as_ref()
    }

    /// Get application collector
    pub fn application(&self) -> &ApplicationCollector {
        &self.application
    }

    /// Get user collector
    pub fn user(&self) -> &UserCollector {
        &self.user
    }

    /// Get render collector
    pub fn render(&self) -> Option<&RenderCollector> {
        self.render.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_collector() {
        let registry = MetricRegistry::new();
        let collector = SystemCollector::new(&registry);

        collector.collect();

        assert!(collector.cpu_usage.get() > 0.0);
        assert!(collector.memory_usage.get() > 0.0);
    }

    #[test]
    fn test_application_collector() {
        let registry = MetricRegistry::new();
        let collector = ApplicationCollector::new(&registry);

        collector.record_operation(Duration::from_millis(100), true);
        assert_eq!(collector.operations_total.get(), 1.0);

        collector.record_operation(Duration::from_millis(200), false);
        assert_eq!(collector.operations_total.get(), 2.0);
        assert_eq!(collector.operations_failed.get(), 1.0);
    }

    #[test]
    fn test_user_collector() {
        let registry = MetricRegistry::new();
        let collector = UserCollector::new(&registry);

        collector.record_session("user1");
        assert_eq!(collector.total_sessions(), 1.0);

        collector.record_action("draw_line", "user1");
        assert_eq!(collector.total_actions(), 1.0);

        collector.record_feature_use("3d_view", "user1");
        assert_eq!(collector.features_used.get(), 1.0);
    }

    #[test]
    fn test_render_collector() {
        let registry = MetricRegistry::new();
        let collector = RenderCollector::new(&registry);

        collector.record_frame(Duration::from_millis(16), 100, 5000);
        assert_eq!(collector.draw_calls.get(), 100.0);
        assert_eq!(collector.triangles_rendered.get(), 5000.0);

        collector.update_fps(60.0);
        assert_eq!(collector.fps.get(), 60.0);
    }

    #[test]
    fn test_cache_hit_rate() {
        let registry = MetricRegistry::new();
        let collector = ApplicationCollector::new(&registry);

        collector.record_cache_hit();
        collector.record_cache_hit();
        collector.record_cache_hit();
        collector.record_cache_miss();

        assert_eq!(collector.cache_hit_rate(), 0.75);
    }
}
