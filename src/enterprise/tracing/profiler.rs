//! Performance profiling
//!
//! This module provides CPU profiling hooks, memory allocation tracking,
//! flame graph generation support, and continuous profiling capabilities.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// CPU profiling sample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuSample {
    /// Timestamp
    pub timestamp: SystemTime,
    /// Thread ID
    pub thread_id: u64,
    /// Stack trace
    pub stack: Vec<StackFrame>,
    /// CPU time consumed
    pub cpu_time: Duration,
}

/// Stack frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    /// Function name
    pub function: String,
    /// File name
    pub file: Option<String>,
    /// Line number
    pub line: Option<u32>,
    /// Module name
    pub module: Option<String>,
}

impl StackFrame {
    /// Create a new stack frame
    pub fn new(function: impl Into<String>) -> Self {
        Self {
            function: function.into(),
            file: None,
            line: None,
            module: None,
        }
    }

    /// With file and line
    pub fn with_location(mut self, file: impl Into<String>, line: u32) -> Self {
        self.file = Some(file.into());
        self.line = Some(line);
        self
    }

    /// With module
    pub fn with_module(mut self, module: impl Into<String>) -> Self {
        self.module = Some(module.into());
        self
    }
}

/// CPU profiler
pub struct CpuProfiler {
    samples: Arc<RwLock<Vec<CpuSample>>>,
    enabled: Arc<RwLock<bool>>,
    sample_interval: Duration,
}

impl CpuProfiler {
    /// Create a new CPU profiler
    pub fn new() -> Self {
        Self {
            samples: Arc::new(RwLock::new(Vec::new())),
            enabled: Arc::new(RwLock::new(false)),
            sample_interval: Duration::from_millis(10),
        }
    }

    /// Set sampling interval
    pub fn with_sample_interval(mut self, interval: Duration) -> Self {
        self.sample_interval = interval;
        self
    }

    /// Start profiling
    pub fn start(&self) {
        *self.enabled.write() = true;
        self.samples.write().clear();

        // In a real implementation, this would start a background thread
        // that periodically samples the call stack
        log::info!("CPU profiler started");
    }

    /// Stop profiling
    pub fn stop(&self) {
        *self.enabled.write() = false;
        log::info!("CPU profiler stopped");
    }

    /// Check if profiler is running
    pub fn is_running(&self) -> bool {
        *self.enabled.read()
    }

    /// Add a sample manually (for testing or integration)
    pub fn add_sample(&self, sample: CpuSample) {
        if *self.enabled.read() {
            self.samples.write().push(sample);
        }
    }

    /// Get all samples
    pub fn samples(&self) -> Vec<CpuSample> {
        self.samples.read().clone()
    }

    /// Get samples for a specific thread
    pub fn samples_for_thread(&self, thread_id: u64) -> Vec<CpuSample> {
        self.samples
            .read()
            .iter()
            .filter(|s| s.thread_id == thread_id)
            .cloned()
            .collect()
    }

    /// Generate flame graph data
    pub fn flame_graph_data(&self) -> FlameGraphData {
        let samples = self.samples.read();
        let mut data = FlameGraphData::new();

        for sample in samples.iter() {
            let stack: Vec<String> = sample.stack
                .iter()
                .map(|f| f.function.clone())
                .collect();

            data.add_sample(stack, 1);
        }

        data
    }

    /// Clear all samples
    pub fn clear(&self) {
        self.samples.write().clear();
    }

    /// Get profiling statistics
    pub fn stats(&self) -> ProfileStats {
        let samples = self.samples.read();
        let total_samples = samples.len();

        let mut function_counts: HashMap<String, usize> = HashMap::new();
        for sample in samples.iter() {
            for frame in &sample.stack {
                *function_counts.entry(frame.function.clone()).or_insert(0) += 1;
            }
        }

        let mut hot_functions: Vec<(String, usize)> = function_counts.into_iter().collect();
        hot_functions.sort_by(|a, b| b.1.cmp(&a.1));
        hot_functions.truncate(10);

        ProfileStats {
            total_samples,
            unique_threads: samples.iter().map(|s| s.thread_id).collect::<std::collections::HashSet<_>>().len(),
            hot_functions,
        }
    }
}

impl Default for CpuProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Profiling statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileStats {
    /// Total number of samples
    pub total_samples: usize,
    /// Number of unique threads sampled
    pub unique_threads: usize,
    /// Hot functions (top 10 by sample count)
    pub hot_functions: Vec<(String, usize)>,
}

/// Flame graph data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlameGraphData {
    /// Root nodes
    pub roots: Vec<FlameGraphNode>,
}

impl FlameGraphData {
    /// Create new flame graph data
    pub fn new() -> Self {
        Self {
            roots: Vec::new(),
        }
    }

    /// Add a sample stack
    pub fn add_sample(&mut self, stack: Vec<String>, count: usize) {
        if stack.is_empty() {
            return;
        }

        let root_name = &stack[0];
        let root = self.roots.iter_mut()
            .find(|n| n.name == *root_name);

        match root {
            Some(node) => {
                node.value += count;
                if stack.len() > 1 {
                    node.add_child_stack(&stack[1..], count);
                }
            }
            None => {
                let mut node = FlameGraphNode::new(root_name.clone(), count);
                if stack.len() > 1 {
                    node.add_child_stack(&stack[1..], count);
                }
                self.roots.push(node);
            }
        }
    }

    /// Export to JSON format for visualization
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}

impl Default for FlameGraphData {
    fn default() -> Self {
        Self::new()
    }
}

/// Flame graph node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlameGraphNode {
    /// Node name (function name)
    pub name: String,
    /// Sample count
    pub value: usize,
    /// Child nodes
    pub children: Vec<FlameGraphNode>,
}

impl FlameGraphNode {
    /// Create a new flame graph node
    pub fn new(name: String, value: usize) -> Self {
        Self {
            name,
            value,
            children: Vec::new(),
        }
    }

    fn add_child_stack(&mut self, stack: &[String], count: usize) {
        if stack.is_empty() {
            return;
        }

        let child_name = &stack[0];
        let child = self.children.iter_mut()
            .find(|n| n.name == *child_name);

        match child {
            Some(node) => {
                node.value += count;
                if stack.len() > 1 {
                    node.add_child_stack(&stack[1..], count);
                }
            }
            None => {
                let mut node = FlameGraphNode::new(child_name.clone(), count);
                if stack.len() > 1 {
                    node.add_child_stack(&stack[1..], count);
                }
                self.children.push(node);
            }
        }
    }
}

// ============================================================================
// Memory Profiling
// ============================================================================

/// Memory allocation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationEvent {
    /// Timestamp
    pub timestamp: SystemTime,
    /// Allocation size in bytes
    pub size: usize,
    /// Allocation type
    pub allocation_type: AllocationType,
    /// Stack trace
    pub stack: Vec<StackFrame>,
}

/// Allocation type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AllocationType {
    /// Heap allocation
    Heap,
    /// Stack allocation
    Stack,
    /// Mmap allocation
    Mmap,
}

/// Memory profiler
pub struct MemoryProfiler {
    allocations: Arc<RwLock<Vec<AllocationEvent>>>,
    enabled: Arc<RwLock<bool>>,
    total_allocated: Arc<RwLock<usize>>,
    peak_memory: Arc<RwLock<usize>>,
}

impl MemoryProfiler {
    /// Create a new memory profiler
    pub fn new() -> Self {
        Self {
            allocations: Arc::new(RwLock::new(Vec::new())),
            enabled: Arc::new(RwLock::new(false)),
            total_allocated: Arc::new(RwLock::new(0)),
            peak_memory: Arc::new(RwLock::new(0)),
        }
    }

    /// Start memory profiling
    pub fn start(&self) {
        *self.enabled.write() = true;
        self.allocations.write().clear();
        *self.total_allocated.write() = 0;
        *self.peak_memory.write() = 0;
        log::info!("Memory profiler started");
    }

    /// Stop memory profiling
    pub fn stop(&self) {
        *self.enabled.write() = false;
        log::info!("Memory profiler stopped");
    }

    /// Check if profiler is running
    pub fn is_running(&self) -> bool {
        *self.enabled.read()
    }

    /// Record an allocation
    pub fn record_allocation(&self, size: usize, allocation_type: AllocationType) {
        if !*self.enabled.read() {
            return;
        }

        let event = AllocationEvent {
            timestamp: SystemTime::now(),
            size,
            allocation_type,
            stack: Vec::new(), // In real impl, would capture stack trace
        };

        self.allocations.write().push(event);

        let mut total = self.total_allocated.write();
        *total += size;

        let mut peak = self.peak_memory.write();
        if *total > *peak {
            *peak = *total;
        }
    }

    /// Get total bytes allocated
    pub fn total_allocated(&self) -> usize {
        *self.total_allocated.read()
    }

    /// Get peak memory usage
    pub fn peak_memory(&self) -> usize {
        *self.peak_memory.read()
    }

    /// Get all allocation events
    pub fn allocations(&self) -> Vec<AllocationEvent> {
        self.allocations.read().clone()
    }

    /// Get memory statistics
    pub fn stats(&self) -> MemoryStats {
        let allocations = self.allocations.read();
        let total_allocations = allocations.len();

        let mut by_type: HashMap<AllocationType, (usize, usize)> = HashMap::new();
        for alloc in allocations.iter() {
            let entry = by_type.entry(alloc.allocation_type).or_insert((0, 0));
            entry.0 += 1; // count
            entry.1 += alloc.size; // total size
        }

        MemoryStats {
            total_allocations,
            total_bytes: *self.total_allocated.read(),
            peak_bytes: *self.peak_memory.read(),
            heap_allocations: by_type.get(&AllocationType::Heap).map(|e| e.0).unwrap_or(0),
            heap_bytes: by_type.get(&AllocationType::Heap).map(|e| e.1).unwrap_or(0),
        }
    }

    /// Clear all data
    pub fn clear(&self) {
        self.allocations.write().clear();
        *self.total_allocated.write() = 0;
        *self.peak_memory.write() = 0;
    }
}

impl Default for MemoryProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory profiling statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Total number of allocations
    pub total_allocations: usize,
    /// Total bytes allocated
    pub total_bytes: usize,
    /// Peak memory usage in bytes
    pub peak_bytes: usize,
    /// Number of heap allocations
    pub heap_allocations: usize,
    /// Total heap bytes
    pub heap_bytes: usize,
}

// ============================================================================
// Continuous Profiling
// ============================================================================

/// Continuous profiler that runs in the background
pub struct ContinuousProfiler {
    cpu_profiler: Arc<CpuProfiler>,
    memory_profiler: Arc<MemoryProfiler>,
    running: Arc<RwLock<bool>>,
}

impl ContinuousProfiler {
    /// Create a new continuous profiler
    pub fn new() -> Self {
        Self {
            cpu_profiler: Arc::new(CpuProfiler::new()),
            memory_profiler: Arc::new(MemoryProfiler::new()),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start continuous profiling
    pub fn start(&self) {
        *self.running.write() = true;
        self.cpu_profiler.start();
        self.memory_profiler.start();

        log::info!("Continuous profiler started");
    }

    /// Stop continuous profiling
    pub fn stop(&self) {
        *self.running.write() = false;
        self.cpu_profiler.stop();
        self.memory_profiler.stop();

        log::info!("Continuous profiler stopped");
    }

    /// Check if profiler is running
    pub fn is_running(&self) -> bool {
        *self.running.read()
    }

    /// Get CPU profiler
    pub fn cpu_profiler(&self) -> Arc<CpuProfiler> {
        self.cpu_profiler.clone()
    }

    /// Get memory profiler
    pub fn memory_profiler(&self) -> Arc<MemoryProfiler> {
        self.memory_profiler.clone()
    }

    /// Generate profiling report
    pub fn generate_report(&self) -> ProfilingReport {
        ProfilingReport {
            cpu_stats: self.cpu_profiler.stats(),
            memory_stats: self.memory_profiler.stats(),
            timestamp: SystemTime::now(),
        }
    }
}

impl Default for ContinuousProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Profiling report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingReport {
    /// CPU profiling statistics
    pub cpu_stats: ProfileStats,
    /// Memory profiling statistics
    pub memory_stats: MemoryStats,
    /// Report timestamp
    pub timestamp: SystemTime,
}

impl ProfilingReport {
    /// Convert to JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}

// ============================================================================
// Profile Scope (RAII-style profiling)
// ============================================================================

/// RAII-style profiling scope
pub struct ProfileScope {
    name: String,
    start: Instant,
    profiler: Option<Arc<CpuProfiler>>,
}

impl ProfileScope {
    /// Create a new profile scope
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            profiler: None,
        }
    }

    /// Create with profiler
    pub fn with_profiler(name: impl Into<String>, profiler: Arc<CpuProfiler>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            profiler: Some(profiler),
        }
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

impl Drop for ProfileScope {
    fn drop(&mut self) {
        let duration = self.start.elapsed();

        if let Some(profiler) = &self.profiler {
            // Record the sample
            // Use hash of thread ID since as_u64() is unstable
            let thread_id = {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                std::thread::current().id().hash(&mut hasher);
                hasher.finish()
            };

            let sample = CpuSample {
                timestamp: SystemTime::now(),
                thread_id,
                stack: vec![StackFrame::new(self.name.clone())],
                cpu_time: duration,
            };
            profiler.add_sample(sample);
        }

        log::debug!("{} took {:?}", self.name, duration);
    }
}

/// Macro for easy profiling scopes
#[macro_export]
macro_rules! profile_scope {
    ($name:expr) => {
        let _profile_scope = $crate::enterprise::tracing::profiler::ProfileScope::new($name);
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_frame_creation() {
        let frame = StackFrame::new("test_function")
            .with_location("test.rs", 42)
            .with_module("caddy::test");

        assert_eq!(frame.function, "test_function");
        assert_eq!(frame.file, Some("test.rs".to_string()));
        assert_eq!(frame.line, Some(42));
        assert_eq!(frame.module, Some("caddy::test".to_string()));
    }

    #[test]
    fn test_cpu_profiler_creation() {
        let profiler = CpuProfiler::new();
        assert!(!profiler.is_running());
        assert_eq!(profiler.samples().len(), 0);
    }

    #[test]
    fn test_cpu_profiler_start_stop() {
        let profiler = CpuProfiler::new();

        profiler.start();
        assert!(profiler.is_running());

        profiler.stop();
        assert!(!profiler.is_running());
    }

    #[test]
    fn test_cpu_profiler_add_sample() {
        let profiler = CpuProfiler::new();
        profiler.start();

        let sample = CpuSample {
            timestamp: SystemTime::now(),
            thread_id: 1,
            stack: vec![StackFrame::new("test")],
            cpu_time: Duration::from_millis(10),
        };

        profiler.add_sample(sample);
        assert_eq!(profiler.samples().len(), 1);
    }

    #[test]
    fn test_cpu_profiler_stats() {
        let profiler = CpuProfiler::new();
        profiler.start();

        let sample = CpuSample {
            timestamp: SystemTime::now(),
            thread_id: 1,
            stack: vec![
                StackFrame::new("func_a"),
                StackFrame::new("func_b"),
            ],
            cpu_time: Duration::from_millis(10),
        };

        profiler.add_sample(sample);

        let stats = profiler.stats();
        assert_eq!(stats.total_samples, 1);
        assert_eq!(stats.unique_threads, 1);
    }

    #[test]
    fn test_flame_graph_data() {
        let mut data = FlameGraphData::new();

        data.add_sample(vec!["main".to_string(), "foo".to_string()], 1);
        data.add_sample(vec!["main".to_string(), "bar".to_string()], 1);
        data.add_sample(vec!["main".to_string(), "foo".to_string()], 1);

        assert_eq!(data.roots.len(), 1);
        assert_eq!(data.roots[0].name, "main");
        assert_eq!(data.roots[0].value, 3);
        assert_eq!(data.roots[0].children.len(), 2);
    }

    #[test]
    fn test_memory_profiler_creation() {
        let profiler = MemoryProfiler::new();
        assert!(!profiler.is_running());
        assert_eq!(profiler.total_allocated(), 0);
    }

    #[test]
    fn test_memory_profiler_start_stop() {
        let profiler = MemoryProfiler::new();

        profiler.start();
        assert!(profiler.is_running());

        profiler.stop();
        assert!(!profiler.is_running());
    }

    #[test]
    fn test_memory_profiler_record_allocation() {
        let profiler = MemoryProfiler::new();
        profiler.start();

        profiler.record_allocation(1024, AllocationType::Heap);
        profiler.record_allocation(2048, AllocationType::Heap);

        assert_eq!(profiler.total_allocated(), 3072);
        assert_eq!(profiler.peak_memory(), 3072);
    }

    #[test]
    fn test_memory_profiler_stats() {
        let profiler = MemoryProfiler::new();
        profiler.start();

        profiler.record_allocation(1024, AllocationType::Heap);

        let stats = profiler.stats();
        assert_eq!(stats.total_allocations, 1);
        assert_eq!(stats.total_bytes, 1024);
        assert_eq!(stats.heap_allocations, 1);
        assert_eq!(stats.heap_bytes, 1024);
    }

    #[test]
    fn test_continuous_profiler() {
        let profiler = ContinuousProfiler::new();
        assert!(!profiler.is_running());

        profiler.start();
        assert!(profiler.is_running());

        profiler.stop();
        assert!(!profiler.is_running());
    }

    #[test]
    fn test_continuous_profiler_report() {
        let profiler = ContinuousProfiler::new();
        profiler.start();

        profiler.cpu_profiler().add_sample(CpuSample {
            timestamp: SystemTime::now(),
            thread_id: 1,
            stack: vec![StackFrame::new("test")],
            cpu_time: Duration::from_millis(10),
        });

        profiler.memory_profiler().record_allocation(1024, AllocationType::Heap);

        let report = profiler.generate_report();
        assert_eq!(report.cpu_stats.total_samples, 1);
        assert_eq!(report.memory_stats.total_bytes, 1024);
    }

    #[test]
    fn test_profile_scope() {
        let scope = ProfileScope::new("test_operation");
        std::thread::sleep(Duration::from_millis(10));
        assert!(scope.elapsed() >= Duration::from_millis(10));
    }

    #[test]
    fn test_profile_scope_with_profiler() {
        let profiler = Arc::new(CpuProfiler::new());
        profiler.start();

        {
            let _scope = ProfileScope::with_profiler("test", profiler.clone());
            std::thread::sleep(Duration::from_millis(10));
        }

        assert_eq!(profiler.samples().len(), 1);
    }

    #[test]
    fn test_allocation_type() {
        assert_eq!(AllocationType::Heap, AllocationType::Heap);
        assert_ne!(AllocationType::Heap, AllocationType::Stack);
    }

    #[test]
    fn test_flame_graph_json() {
        let data = FlameGraphData::new();
        let json = data.to_json();
        assert!(json.contains("roots"));
    }

    #[test]
    fn test_profiling_report_json() {
        let report = ProfilingReport {
            cpu_stats: ProfileStats {
                total_samples: 10,
                unique_threads: 2,
                hot_functions: vec![],
            },
            memory_stats: MemoryStats {
                total_allocations: 5,
                total_bytes: 1024,
                peak_bytes: 2048,
                heap_allocations: 5,
                heap_bytes: 1024,
            },
            timestamp: SystemTime::now(),
        };

        let json = report.to_json();
        assert!(json.contains("cpu_stats"));
        assert!(json.contains("memory_stats"));
    }
}
