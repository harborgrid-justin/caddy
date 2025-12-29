/**
 * CADDY v0.4.0 - Health Checks Grid
 * Service health status monitoring and configuration
 * @module monitoring/HealthChecks
 */

import React, { useEffect, useState, useCallback } from 'react';
import {
  ServiceHealth,
  ServiceStatus,
  HealthCheckConfig,
  WebSocketMessage
} from './types';

interface HealthChecksProps {
  services?: string[];
  autoRefresh?: boolean;
  refreshInterval?: number;
  onServiceClick?: (service: ServiceHealth) => void;
  className?: string;
}

interface HealthCheckResult {
  serviceId: string;
  success: boolean;
  responseTime: number;
  statusCode?: number;
  error?: string;
  timestamp: Date;
}

export const HealthChecks: React.FC<HealthChecksProps> = ({
  services: serviceFilter,
  autoRefresh = true,
  refreshInterval = 30000,
  onServiceClick,
  className = ''
}) => {
  const [services, setServices] = useState<ServiceHealth[]>([]);
  const [configs, setConfigs] = useState<HealthCheckConfig[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedService, setSelectedService] = useState<ServiceHealth | null>(null);
  const [showConfig, setShowConfig] = useState(false);
  const [filter, setFilter] = useState<ServiceStatus | 'all'>('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [ws, setWs] = useState<WebSocket | null>(null);

  // WebSocket connection for real-time health updates
  useEffect(() => {
    if (!autoRefresh) return;

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/api/monitoring/health/stream`;
    const socket = new WebSocket(wsUrl);

    socket.onopen = () => {
      console.log('[HealthChecks] WebSocket connected');
      socket.send(JSON.stringify({ type: 'subscribe', services: serviceFilter || [] }));
    };

    socket.onmessage = (event) => {
      try {
        const message: WebSocketMessage = JSON.parse(event.data);
        if (message.type === 'health') {
          updateServiceHealth(message.data);
        }
      } catch (err) {
        console.error('[HealthChecks] Failed to parse WebSocket message:', err);
      }
    };

    socket.onerror = (err) => {
      console.error('[HealthChecks] WebSocket error:', err);
    };

    socket.onclose = () => {
      console.log('[HealthChecks] WebSocket disconnected');
      setTimeout(() => {
        // Reconnect
      }, 5000);
    };

    setWs(socket);

    return () => {
      socket.close();
    };
  }, [autoRefresh, serviceFilter]);

  // Fetch health data
  useEffect(() => {
    fetchHealthData();
  }, [serviceFilter]);

  // Auto-refresh polling (fallback when WebSocket is not available)
  useEffect(() => {
    if (!autoRefresh || ws) return;

    const interval = setInterval(() => {
      fetchHealthData();
    }, refreshInterval);

    return () => clearInterval(interval);
  }, [autoRefresh, refreshInterval, ws]);

  const fetchHealthData = async () => {
    try {
      setLoading(true);
      setError(null);

      const queryParams = serviceFilter
        ? `?services=${serviceFilter.join(',')}`
        : '';

      const [servicesRes, configsRes] = await Promise.all([
        fetch(`/api/monitoring/health${queryParams}`),
        fetch(`/api/monitoring/health/configs${queryParams}`)
      ]);

      if (!servicesRes.ok || !configsRes.ok) {
        throw new Error('Failed to fetch health data');
      }

      const [servicesData, configsData] = await Promise.all([
        servicesRes.json(),
        configsRes.json()
      ]);

      setServices(servicesData);
      setConfigs(configsData);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
      console.error('[HealthChecks] Error fetching health data:', err);
    } finally {
      setLoading(false);
    }
  };

  const updateServiceHealth = useCallback((updatedService: ServiceHealth) => {
    setServices(prev => {
      const index = prev.findIndex(s => s.id === updatedService.id);
      if (index >= 0) {
        const updated = [...prev];
        updated[index] = updatedService;
        return updated;
      }
      return [...prev, updatedService];
    });
  }, []);

  const runHealthCheck = async (serviceId: string) => {
    try {
      const response = await fetch(`/api/monitoring/health/${serviceId}/check`, {
        method: 'POST'
      });

      if (!response.ok) {
        throw new Error('Health check failed');
      }

      const result: HealthCheckResult = await response.json();

      // Update service health based on result
      setServices(prev => prev.map(s => {
        if (s.id === serviceId) {
          return {
            ...s,
            status: result.success ? ServiceStatus.HEALTHY : ServiceStatus.DOWN,
            responseTime: result.responseTime,
            lastCheck: result.timestamp
          };
        }
        return s;
      }));
    } catch (err) {
      console.error(`[HealthChecks] Failed to run health check for ${serviceId}:`, err);
    }
  };

  const handleServiceClick = (service: ServiceHealth) => {
    setSelectedService(service);
    if (onServiceClick) {
      onServiceClick(service);
    }
  };

  const getStatusIcon = (status: ServiceStatus): string => {
    switch (status) {
      case ServiceStatus.HEALTHY:
        return '✓';
      case ServiceStatus.DEGRADED:
        return '⚠';
      case ServiceStatus.DOWN:
        return '✗';
      case ServiceStatus.MAINTENANCE:
        return '🔧';
      default:
        return '?';
    }
  };

  const getStatusColor = (status: ServiceStatus): string => {
    switch (status) {
      case ServiceStatus.HEALTHY:
        return '#10b981';
      case ServiceStatus.DEGRADED:
        return '#f59e0b';
      case ServiceStatus.DOWN:
        return '#ef4444';
      case ServiceStatus.MAINTENANCE:
        return '#3b82f6';
      default:
        return '#6b7280';
    }
  };

  const getStatusLabel = (status: ServiceStatus): string => {
    return status.charAt(0).toUpperCase() + status.slice(1);
  };

  const filteredServices = services.filter(service => {
    const matchesFilter = filter === 'all' || service.status === filter;
    const matchesSearch = service.name.toLowerCase().includes(searchQuery.toLowerCase());
    return matchesFilter && matchesSearch;
  });

  const statusCounts = {
    all: services.length,
    healthy: services.filter(s => s.status === ServiceStatus.HEALTHY).length,
    degraded: services.filter(s => s.status === ServiceStatus.DEGRADED).length,
    down: services.filter(s => s.status === ServiceStatus.DOWN).length,
    maintenance: services.filter(s => s.status === ServiceStatus.MAINTENANCE).length
  };

  if (loading && services.length === 0) {
    return (
      <div style={styles.loading}>
        <div style={styles.spinner} />
        <p>Loading health checks...</p>
      </div>
    );
  }

  return (
    <div className={`health-checks ${className}`} style={styles.container}>
      {/* Header */}
      <div style={styles.header}>
        <h2 style={styles.title}>Service Health Checks</h2>
        <div style={styles.headerActions}>
          <button
            style={styles.button}
            onClick={() => fetchHealthData()}
          >
            Refresh All
          </button>
          <button
            style={styles.button}
            onClick={() => setShowConfig(!showConfig)}
          >
            {showConfig ? 'Hide' : 'Show'} Config
          </button>
        </div>
      </div>

      {/* Error Display */}
      {error && (
        <div style={styles.error}>
          <span>⚠ {error}</span>
          <button onClick={() => setError(null)} style={styles.errorClose}>×</button>
        </div>
      )}

      {/* Filters */}
      <div style={styles.filters}>
        <div style={styles.statusFilters}>
          {(['all', ServiceStatus.HEALTHY, ServiceStatus.DEGRADED, ServiceStatus.DOWN, ServiceStatus.MAINTENANCE] as const).map(
            (status) => (
              <button
                key={status}
                style={{
                  ...styles.filterButton,
                  ...(filter === status ? styles.filterButtonActive : {})
                }}
                onClick={() => setFilter(status)}
              >
                {status === 'all' ? 'All' : getStatusLabel(status)}
                <span style={styles.filterCount}>
                  {status === 'all' ? statusCounts.all : statusCounts[status as keyof typeof statusCounts]}
                </span>
              </button>
            )
          )}
        </div>

        <input
          type="text"
          placeholder="Search services..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          style={styles.searchInput}
        />
      </div>

      {/* Health Checks Grid */}
      <div style={styles.grid}>
        {filteredServices.map((service) => {
          const config = configs.find(c => c.service === service.id);

          return (
            <div
              key={service.id}
              style={styles.card}
              onClick={() => handleServiceClick(service)}
            >
              {/* Status Indicator */}
              <div
                style={{
                  ...styles.statusBar,
                  backgroundColor: getStatusColor(service.status)
                }}
              />

              {/* Card Content */}
              <div style={styles.cardContent}>
                <div style={styles.cardHeader}>
                  <div style={styles.cardTitle}>
                    <span
                      style={{
                        ...styles.statusIcon,
                        color: getStatusColor(service.status)
                      }}
                    >
                      {getStatusIcon(service.status)}
                    </span>
                    <span style={styles.serviceName}>{service.name}</span>
                  </div>
                  <button
                    style={styles.checkButton}
                    onClick={(e) => {
                      e.stopPropagation();
                      runHealthCheck(service.id);
                    }}
                  >
                    Check Now
                  </button>
                </div>

                {service.message && (
                  <div style={styles.message}>{service.message}</div>
                )}

                <div style={styles.metrics}>
                  <div style={styles.metric}>
                    <span style={styles.metricLabel}>Response Time:</span>
                    <span style={styles.metricValue}>
                      {service.responseTime < 1000
                        ? `${service.responseTime.toFixed(0)}ms`
                        : `${(service.responseTime / 1000).toFixed(2)}s`}
                    </span>
                  </div>

                  <div style={styles.metric}>
                    <span style={styles.metricLabel}>Uptime:</span>
                    <span style={styles.metricValue}>
                      {service.uptime.toFixed(3)}%
                    </span>
                  </div>

                  <div style={styles.metric}>
                    <span style={styles.metricLabel}>Last Check:</span>
                    <span style={styles.metricValue}>
                      {new Date(service.lastCheck).toLocaleTimeString()}
                    </span>
                  </div>

                  {config && (
                    <div style={styles.metric}>
                      <span style={styles.metricLabel}>Interval:</span>
                      <span style={styles.metricValue}>{config.interval}s</span>
                    </div>
                  )}
                </div>

                {/* Dependencies */}
                {service.dependencies.length > 0 && (
                  <div style={styles.dependencies}>
                    <span style={styles.dependenciesLabel}>Dependencies:</span>
                    <div style={styles.dependenciesList}>
                      {service.dependencies.map((dep, idx) => (
                        <span key={idx} style={styles.dependency}>
                          {dep}
                        </span>
                      ))}
                    </div>
                  </div>
                )}

                {/* Config Details (if visible) */}
                {showConfig && config && (
                  <div style={styles.config}>
                    <div style={styles.configTitle}>Configuration:</div>
                    <div style={styles.configDetails}>
                      <div>Type: {config.type}</div>
                      <div>Endpoint: {config.endpoint}</div>
                      <div>Timeout: {config.timeout}s</div>
                      <div>Retries: {config.retries}</div>
                    </div>
                  </div>
                )}
              </div>
            </div>
          );
        })}
      </div>

      {/* Empty State */}
      {filteredServices.length === 0 && (
        <div style={styles.emptyState}>
          <p>No services found matching your criteria</p>
        </div>
      )}

      {/* Selected Service Details Modal */}
      {selectedService && (
        <div style={styles.modal} onClick={() => setSelectedService(null)}>
          <div style={styles.modalContent} onClick={(e) => e.stopPropagation()}>
            <div style={styles.modalHeader}>
              <h3>{selectedService.name}</h3>
              <button
                style={styles.modalClose}
                onClick={() => setSelectedService(null)}
              >
                ×
              </button>
            </div>
            <div style={styles.modalBody}>
              <div style={styles.detailRow}>
                <strong>Status:</strong> {getStatusLabel(selectedService.status)}
              </div>
              <div style={styles.detailRow}>
                <strong>Response Time:</strong> {selectedService.responseTime}ms
              </div>
              <div style={styles.detailRow}>
                <strong>Uptime:</strong> {selectedService.uptime.toFixed(3)}%
              </div>
              <div style={styles.detailRow}>
                <strong>CPU:</strong> {selectedService.metrics.cpu.toFixed(1)}%
              </div>
              <div style={styles.detailRow}>
                <strong>Memory:</strong> {selectedService.metrics.memory.toFixed(1)}%
              </div>
              <div style={styles.detailRow}>
                <strong>Disk:</strong> {selectedService.metrics.disk.toFixed(1)}%
              </div>
              <div style={styles.detailRow}>
                <strong>Request Rate:</strong> {selectedService.metrics.requestRate}/s
              </div>
              <div style={styles.detailRow}>
                <strong>Error Rate:</strong> {selectedService.metrics.errorRate.toFixed(2)}%
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: {
    padding: '24px',
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif'
  },
  loading: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '48px',
    color: '#6b7280'
  },
  spinner: {
    width: '40px',
    height: '40px',
    border: '4px solid #e5e7eb',
    borderTopColor: '#3b82f6',
    borderRadius: '50%',
    animation: 'spin 1s linear infinite'
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '24px'
  },
  title: {
    fontSize: '24px',
    fontWeight: 700,
    color: '#111827',
    margin: 0
  },
  headerActions: {
    display: 'flex',
    gap: '12px'
  },
  button: {
    padding: '8px 16px',
    backgroundColor: '#3b82f6',
    color: '#fff',
    border: 'none',
    borderRadius: '6px',
    fontSize: '14px',
    fontWeight: 500,
    cursor: 'pointer',
    transition: 'background-color 0.2s'
  },
  error: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '12px 16px',
    backgroundColor: '#fef2f2',
    border: '1px solid #fecaca',
    borderRadius: '8px',
    color: '#991b1b',
    marginBottom: '16px'
  },
  errorClose: {
    background: 'none',
    border: 'none',
    fontSize: '24px',
    cursor: 'pointer',
    color: '#991b1b'
  },
  filters: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '24px',
    gap: '16px',
    flexWrap: 'wrap'
  },
  statusFilters: {
    display: 'flex',
    gap: '8px',
    flexWrap: 'wrap'
  },
  filterButton: {
    padding: '6px 12px',
    backgroundColor: '#fff',
    border: '1px solid #e5e7eb',
    borderRadius: '6px',
    fontSize: '13px',
    fontWeight: 500,
    color: '#6b7280',
    cursor: 'pointer',
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    transition: 'all 0.2s'
  },
  filterButtonActive: {
    backgroundColor: '#3b82f6',
    color: '#fff',
    borderColor: '#3b82f6'
  },
  filterCount: {
    fontSize: '11px',
    padding: '2px 6px',
    backgroundColor: 'rgba(0, 0, 0, 0.1)',
    borderRadius: '10px'
  },
  searchInput: {
    padding: '8px 12px',
    border: '1px solid #e5e7eb',
    borderRadius: '6px',
    fontSize: '14px',
    minWidth: '200px',
    outline: 'none'
  },
  grid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(350px, 1fr))',
    gap: '16px'
  },
  card: {
    backgroundColor: '#fff',
    borderRadius: '8px',
    border: '1px solid #e5e7eb',
    overflow: 'hidden',
    cursor: 'pointer',
    transition: 'box-shadow 0.2s',
    position: 'relative'
  },
  statusBar: {
    height: '4px'
  },
  cardContent: {
    padding: '16px'
  },
  cardHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '12px'
  },
  cardTitle: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px'
  },
  statusIcon: {
    fontSize: '18px',
    fontWeight: 'bold'
  },
  serviceName: {
    fontSize: '16px',
    fontWeight: 600,
    color: '#111827'
  },
  checkButton: {
    padding: '4px 12px',
    backgroundColor: '#f3f4f6',
    border: '1px solid #e5e7eb',
    borderRadius: '4px',
    fontSize: '12px',
    fontWeight: 500,
    cursor: 'pointer',
    color: '#374151'
  },
  message: {
    fontSize: '13px',
    color: '#6b7280',
    marginBottom: '12px',
    fontStyle: 'italic'
  },
  metrics: {
    display: 'grid',
    gridTemplateColumns: '1fr 1fr',
    gap: '8px',
    marginBottom: '12px'
  },
  metric: {
    fontSize: '13px',
    display: 'flex',
    justifyContent: 'space-between'
  },
  metricLabel: {
    color: '#6b7280'
  },
  metricValue: {
    color: '#111827',
    fontWeight: 500
  },
  dependencies: {
    marginTop: '12px',
    paddingTop: '12px',
    borderTop: '1px solid #e5e7eb'
  },
  dependenciesLabel: {
    fontSize: '12px',
    color: '#6b7280',
    fontWeight: 500,
    display: 'block',
    marginBottom: '6px'
  },
  dependenciesList: {
    display: 'flex',
    flexWrap: 'wrap',
    gap: '4px'
  },
  dependency: {
    fontSize: '11px',
    padding: '2px 8px',
    backgroundColor: '#f3f4f6',
    borderRadius: '4px',
    color: '#374151'
  },
  config: {
    marginTop: '12px',
    paddingTop: '12px',
    borderTop: '1px solid #e5e7eb'
  },
  configTitle: {
    fontSize: '12px',
    fontWeight: 600,
    color: '#111827',
    marginBottom: '8px'
  },
  configDetails: {
    fontSize: '12px',
    color: '#6b7280',
    display: 'grid',
    gap: '4px'
  },
  emptyState: {
    textAlign: 'center',
    padding: '48px',
    color: '#6b7280'
  },
  modal: {
    position: 'fixed',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 1000
  },
  modalContent: {
    backgroundColor: '#fff',
    borderRadius: '12px',
    maxWidth: '500px',
    width: '90%',
    maxHeight: '80vh',
    overflow: 'auto'
  },
  modalHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '20px',
    borderBottom: '1px solid #e5e7eb'
  },
  modalClose: {
    background: 'none',
    border: 'none',
    fontSize: '32px',
    cursor: 'pointer',
    color: '#6b7280'
  },
  modalBody: {
    padding: '20px'
  },
  detailRow: {
    padding: '8px 0',
    borderBottom: '1px solid #f3f4f6',
    fontSize: '14px'
  }
};

export default HealthChecks;
