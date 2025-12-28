//! Resource Isolation and Quotas
//!
//! Implements per-tenant resource limits including memory, CPU, storage, and network quotas.

use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use super::context::TenantId;

/// Resource isolation errors
#[derive(Error, Debug)]
pub enum IsolationError {
    #[error("Memory quota exceeded: {current}/{limit} bytes")]
    MemoryQuotaExceeded { current: u64, limit: u64 },

    #[error("CPU quota exceeded: {current}/{limit} milliseconds")]
    CpuQuotaExceeded { current: u64, limit: u64 },

    #[error("Storage quota exceeded: {current}/{limit} bytes")]
    StorageQuotaExceeded { current: u64, limit: u64 },

    #[error("Network bandwidth exceeded: {current}/{limit} bytes/sec")]
    BandwidthExceeded { current: u64, limit: u64 },

    #[error("Concurrent connections limit exceeded: {current}/{limit}")]
    ConnectionLimitExceeded { current: u32, limit: u32 },

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Tenant not found: {0}")]
    TenantNotFound(String),
}

pub type IsolationResult<T> = Result<T, IsolationError>;

/// Resource quotas for a tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuotas {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: u64,
    /// Maximum CPU time in milliseconds per minute
    pub max_cpu_time_ms: u64,
    /// Maximum storage in bytes
    pub max_storage_bytes: u64,
    /// Maximum network bandwidth in bytes per second
    pub max_bandwidth_bps: u64,
    /// Maximum concurrent connections
    pub max_connections: u32,
    /// Maximum API requests per minute
    pub max_requests_per_minute: u32,
}

impl Default for ResourceQuotas {
    fn default() -> Self {
        Self {
            max_memory_bytes: 1024 * 1024 * 1024,      // 1 GB
            max_cpu_time_ms: 60_000,                    // 1 minute per minute
            max_storage_bytes: 10 * 1024 * 1024 * 1024, // 10 GB
            max_bandwidth_bps: 10 * 1024 * 1024,        // 10 MB/s
            max_connections: 100,
            max_requests_per_minute: 1000,
        }
    }
}

impl ResourceQuotas {
    /// Create enterprise tier quotas
    pub fn enterprise() -> Self {
        Self {
            max_memory_bytes: 10 * 1024 * 1024 * 1024,   // 10 GB
            max_cpu_time_ms: 300_000,                     // 5 minutes per minute
            max_storage_bytes: 1024 * 1024 * 1024 * 1024, // 1 TB
            max_bandwidth_bps: 100 * 1024 * 1024,         // 100 MB/s
            max_connections: 1000,
            max_requests_per_minute: 10_000,
        }
    }

    /// Create basic tier quotas
    pub fn basic() -> Self {
        Self {
            max_memory_bytes: 256 * 1024 * 1024,    // 256 MB
            max_cpu_time_ms: 30_000,                 // 30 seconds per minute
            max_storage_bytes: 1024 * 1024 * 1024,   // 1 GB
            max_bandwidth_bps: 1024 * 1024,          // 1 MB/s
            max_connections: 10,
            max_requests_per_minute: 100,
        }
    }
}

/// Current resource usage tracking
#[derive(Debug, Default)]
struct ResourceUsage {
    /// Current memory usage in bytes
    memory_bytes: u64,
    /// CPU time used in current window
    cpu_time_ms: u64,
    /// Window start for CPU tracking
    cpu_window_start: Option<Instant>,
    /// Total storage used
    storage_bytes: u64,
    /// Network bytes transferred in current window
    network_bytes: u64,
    /// Window start for network tracking
    network_window_start: Option<Instant>,
    /// Current active connections
    active_connections: u32,
    /// Requests in current window
    requests_count: u32,
    /// Window start for request tracking
    requests_window_start: Option<Instant>,
}

impl ResourceUsage {
    fn new() -> Self {
        Self {
            cpu_window_start: Some(Instant::now()),
            network_window_start: Some(Instant::now()),
            requests_window_start: Some(Instant::now()),
            ..Default::default()
        }
    }

    /// Reset time-based windows if expired
    fn reset_windows(&mut self) {
        let now = Instant::now();

        // Reset CPU window (1 minute)
        if let Some(start) = self.cpu_window_start {
            if now.duration_since(start) >= Duration::from_secs(60) {
                self.cpu_time_ms = 0;
                self.cpu_window_start = Some(now);
            }
        }

        // Reset network window (1 second)
        if let Some(start) = self.network_window_start {
            if now.duration_since(start) >= Duration::from_secs(1) {
                self.network_bytes = 0;
                self.network_window_start = Some(now);
            }
        }

        // Reset requests window (1 minute)
        if let Some(start) = self.requests_window_start {
            if now.duration_since(start) >= Duration::from_secs(60) {
                self.requests_count = 0;
                self.requests_window_start = Some(now);
            }
        }
    }
}

/// Tenant resource isolation manager
pub struct TenantIsolationManager {
    quotas: DashMap<TenantId, ResourceQuotas>,
    usage: DashMap<TenantId, Arc<RwLock<ResourceUsage>>>,
}

impl TenantIsolationManager {
    /// Create a new isolation manager
    pub fn new() -> Self {
        Self {
            quotas: DashMap::new(),
            usage: DashMap::new(),
        }
    }

    /// Set quotas for a tenant
    pub fn set_quotas(&self, tenant_id: TenantId, quotas: ResourceQuotas) {
        self.quotas.insert(tenant_id.clone(), quotas);
        self.usage.insert(tenant_id, Arc::new(RwLock::new(ResourceUsage::new())));
    }

    /// Get quotas for a tenant
    pub fn get_quotas(&self, tenant_id: &TenantId) -> Option<ResourceQuotas> {
        self.quotas.get(tenant_id).map(|q| q.clone())
    }

    /// Allocate memory for a tenant
    pub fn allocate_memory(&self, tenant_id: &TenantId, bytes: u64) -> IsolationResult<()> {
        let quotas = self.quotas.get(tenant_id)
            .ok_or_else(|| IsolationError::TenantNotFound(tenant_id.to_string()))?;

        let usage = self.usage.get(tenant_id)
            .ok_or_else(|| IsolationError::TenantNotFound(tenant_id.to_string()))?;

        let mut usage = usage.write();
        let new_usage = usage.memory_bytes + bytes;

        if new_usage > quotas.max_memory_bytes {
            return Err(IsolationError::MemoryQuotaExceeded {
                current: new_usage,
                limit: quotas.max_memory_bytes,
            });
        }

        usage.memory_bytes = new_usage;
        Ok(())
    }

    /// Deallocate memory for a tenant
    pub fn deallocate_memory(&self, tenant_id: &TenantId, bytes: u64) {
        if let Some(usage) = self.usage.get(tenant_id) {
            let mut usage = usage.write();
            usage.memory_bytes = usage.memory_bytes.saturating_sub(bytes);
        }
    }

    /// Record CPU time usage
    pub fn record_cpu_time(&self, tenant_id: &TenantId, milliseconds: u64) -> IsolationResult<()> {
        let quotas = self.quotas.get(tenant_id)
            .ok_or_else(|| IsolationError::TenantNotFound(tenant_id.to_string()))?;

        let usage = self.usage.get(tenant_id)
            .ok_or_else(|| IsolationError::TenantNotFound(tenant_id.to_string()))?;

        let mut usage = usage.write();
        usage.reset_windows();

        let new_usage = usage.cpu_time_ms + milliseconds;

        if new_usage > quotas.max_cpu_time_ms {
            return Err(IsolationError::CpuQuotaExceeded {
                current: new_usage,
                limit: quotas.max_cpu_time_ms,
            });
        }

        usage.cpu_time_ms = new_usage;
        Ok(())
    }

    /// Update storage usage
    pub fn update_storage(&self, tenant_id: &TenantId, bytes: i64) -> IsolationResult<()> {
        let quotas = self.quotas.get(tenant_id)
            .ok_or_else(|| IsolationError::TenantNotFound(tenant_id.to_string()))?;

        let usage = self.usage.get(tenant_id)
            .ok_or_else(|| IsolationError::TenantNotFound(tenant_id.to_string()))?;

        let mut usage = usage.write();
        let new_usage = if bytes >= 0 {
            usage.storage_bytes + bytes as u64
        } else {
            usage.storage_bytes.saturating_sub((-bytes) as u64)
        };

        if new_usage > quotas.max_storage_bytes {
            return Err(IsolationError::StorageQuotaExceeded {
                current: new_usage,
                limit: quotas.max_storage_bytes,
            });
        }

        usage.storage_bytes = new_usage;
        Ok(())
    }

    /// Record network transfer
    pub fn record_network_transfer(&self, tenant_id: &TenantId, bytes: u64) -> IsolationResult<()> {
        let quotas = self.quotas.get(tenant_id)
            .ok_or_else(|| IsolationError::TenantNotFound(tenant_id.to_string()))?;

        let usage = self.usage.get(tenant_id)
            .ok_or_else(|| IsolationError::TenantNotFound(tenant_id.to_string()))?;

        let mut usage = usage.write();
        usage.reset_windows();

        let new_usage = usage.network_bytes + bytes;

        if new_usage > quotas.max_bandwidth_bps {
            return Err(IsolationError::BandwidthExceeded {
                current: new_usage,
                limit: quotas.max_bandwidth_bps,
            });
        }

        usage.network_bytes = new_usage;
        Ok(())
    }

    /// Acquire a connection slot
    pub fn acquire_connection(&self, tenant_id: &TenantId) -> IsolationResult<ConnectionGuard> {
        let quotas = self.quotas.get(tenant_id)
            .ok_or_else(|| IsolationError::TenantNotFound(tenant_id.to_string()))?;

        let usage = self.usage.get(tenant_id)
            .ok_or_else(|| IsolationError::TenantNotFound(tenant_id.to_string()))?;

        let mut usage_lock = usage.write();

        if usage_lock.active_connections >= quotas.max_connections {
            return Err(IsolationError::ConnectionLimitExceeded {
                current: usage_lock.active_connections,
                limit: quotas.max_connections,
            });
        }

        usage_lock.active_connections += 1;
        drop(usage_lock);

        Ok(ConnectionGuard {
            tenant_id: tenant_id.clone(),
            usage: Arc::clone(&usage),
        })
    }

    /// Check and increment request counter
    pub fn check_rate_limit(&self, tenant_id: &TenantId) -> IsolationResult<()> {
        let quotas = self.quotas.get(tenant_id)
            .ok_or_else(|| IsolationError::TenantNotFound(tenant_id.to_string()))?;

        let usage = self.usage.get(tenant_id)
            .ok_or_else(|| IsolationError::TenantNotFound(tenant_id.to_string()))?;

        let mut usage = usage.write();
        usage.reset_windows();

        if usage.requests_count >= quotas.max_requests_per_minute {
            return Err(IsolationError::RateLimitExceeded(
                format!("Limit: {} requests per minute", quotas.max_requests_per_minute)
            ));
        }

        usage.requests_count += 1;
        Ok(())
    }

    /// Get current resource usage for a tenant
    pub fn get_usage(&self, tenant_id: &TenantId) -> Option<ResourceUsageSnapshot> {
        self.usage.get(tenant_id).map(|usage| {
            let usage = usage.read();
            ResourceUsageSnapshot {
                memory_bytes: usage.memory_bytes,
                cpu_time_ms: usage.cpu_time_ms,
                storage_bytes: usage.storage_bytes,
                network_bytes: usage.network_bytes,
                active_connections: usage.active_connections,
                requests_count: usage.requests_count,
            }
        })
    }

    /// Remove tenant from tracking
    pub fn remove_tenant(&self, tenant_id: &TenantId) {
        self.quotas.remove(tenant_id);
        self.usage.remove(tenant_id);
    }
}

impl Default for TenantIsolationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of current resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageSnapshot {
    pub memory_bytes: u64,
    pub cpu_time_ms: u64,
    pub storage_bytes: u64,
    pub network_bytes: u64,
    pub active_connections: u32,
    pub requests_count: u32,
}

/// RAII guard for connection slots
pub struct ConnectionGuard {
    tenant_id: TenantId,
    usage: Arc<RwLock<ResourceUsage>>,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        let mut usage = self.usage.write();
        usage.active_connections = usage.active_connections.saturating_sub(1);
    }
}

/// Memory allocation guard with automatic cleanup
pub struct MemoryGuard<'a> {
    manager: &'a TenantIsolationManager,
    tenant_id: TenantId,
    bytes: u64,
}

impl<'a> MemoryGuard<'a> {
    /// Create a new memory guard
    pub fn new(
        manager: &'a TenantIsolationManager,
        tenant_id: TenantId,
        bytes: u64,
    ) -> IsolationResult<Self> {
        manager.allocate_memory(&tenant_id, bytes)?;
        Ok(Self {
            manager,
            tenant_id,
            bytes,
        })
    }
}

impl<'a> Drop for MemoryGuard<'a> {
    fn drop(&mut self) {
        self.manager.deallocate_memory(&self.tenant_id, self.bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_quota_enforcement() {
        let manager = TenantIsolationManager::new();
        let tenant_id = TenantId::new_org(Uuid::new_v4());

        let quotas = ResourceQuotas {
            max_memory_bytes: 1000,
            ..Default::default()
        };

        manager.set_quotas(tenant_id.clone(), quotas);

        assert!(manager.allocate_memory(&tenant_id, 500).is_ok());
        assert!(manager.allocate_memory(&tenant_id, 400).is_ok());
        assert!(manager.allocate_memory(&tenant_id, 200).is_err()); // Should exceed

        manager.deallocate_memory(&tenant_id, 500);
        assert!(manager.allocate_memory(&tenant_id, 200).is_ok()); // Should work now
    }

    #[test]
    fn test_connection_limits() {
        let manager = TenantIsolationManager::new();
        let tenant_id = TenantId::new_org(Uuid::new_v4());

        let quotas = ResourceQuotas {
            max_connections: 2,
            ..Default::default()
        };

        manager.set_quotas(tenant_id.clone(), quotas);

        let _conn1 = manager.acquire_connection(&tenant_id).unwrap();
        let _conn2 = manager.acquire_connection(&tenant_id).unwrap();
        assert!(manager.acquire_connection(&tenant_id).is_err());

        drop(_conn1);
        assert!(manager.acquire_connection(&tenant_id).is_ok());
    }

    #[test]
    fn test_storage_quota() {
        let manager = TenantIsolationManager::new();
        let tenant_id = TenantId::new_org(Uuid::new_v4());

        let quotas = ResourceQuotas {
            max_storage_bytes: 1000,
            ..Default::default()
        };

        manager.set_quotas(tenant_id.clone(), quotas);

        assert!(manager.update_storage(&tenant_id, 500).is_ok());
        assert!(manager.update_storage(&tenant_id, 400).is_ok());
        assert!(manager.update_storage(&tenant_id, 200).is_err());

        // Test deallocation
        assert!(manager.update_storage(&tenant_id, -500).is_ok());
        assert!(manager.update_storage(&tenant_id, 200).is_ok());
    }

    #[test]
    fn test_rate_limiting() {
        let manager = TenantIsolationManager::new();
        let tenant_id = TenantId::new_org(Uuid::new_v4());

        let quotas = ResourceQuotas {
            max_requests_per_minute: 3,
            ..Default::default()
        };

        manager.set_quotas(tenant_id.clone(), quotas);

        assert!(manager.check_rate_limit(&tenant_id).is_ok());
        assert!(manager.check_rate_limit(&tenant_id).is_ok());
        assert!(manager.check_rate_limit(&tenant_id).is_ok());
        assert!(manager.check_rate_limit(&tenant_id).is_err());
    }

    #[test]
    fn test_memory_guard() {
        let manager = TenantIsolationManager::new();
        let tenant_id = TenantId::new_org(Uuid::new_v4());

        let quotas = ResourceQuotas {
            max_memory_bytes: 1000,
            ..Default::default()
        };

        manager.set_quotas(tenant_id.clone(), quotas);

        {
            let _guard = MemoryGuard::new(&manager, tenant_id.clone(), 500).unwrap();
            let usage = manager.get_usage(&tenant_id).unwrap();
            assert_eq!(usage.memory_bytes, 500);
        }

        let usage = manager.get_usage(&tenant_id).unwrap();
        assert_eq!(usage.memory_bytes, 0);
    }
}
