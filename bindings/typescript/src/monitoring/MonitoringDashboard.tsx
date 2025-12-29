/**
 * CADDY v0.4.0 - System Monitoring Dashboard
 * Real-time monitoring overview with customizable widgets
 * @module monitoring/MonitoringDashboard
 */

import React, { useEffect, useState, useCallback, useMemo } from 'react';
import {
  ServiceHealth,
  Alert,
  AlertSeverity,
  MonitoringDashboardConfig,
  TimeRange,
  WebSocketMessage,
  ServiceStatus
} from './types';

interface MonitoringDashboardProps {
  config?: MonitoringDashboardConfig;
  onConfigChange?: (config: MonitoringDashboardConfig) => void;
  className?: string;
}

interface DashboardStats {
  totalServices: number;
  healthyServices: number;
  degradedServices: number;
  downServices: number;
  activeAlerts: number;
  criticalAlerts: number;
  averageResponseTime: number;
  overallUptime: number;
}

export const MonitoringDashboard: React.FC<MonitoringDashboardProps> = ({
  config,
  onConfigChange,
  className = ''
}) => {
  const [services, setServices] = useState<ServiceHealth[]>([]);
  const [alerts, setAlerts] = useState<Alert[]>([]);
  const [stats, setStats] = useState<DashboardStats>({
    totalServices: 0,
    healthyServices: 0,
    degradedServices: 0,
    downServices: 0,
    activeAlerts: 0,
    criticalAlerts: 0,
    averageResponseTime: 0,
    overallUptime: 100
  });
  const [timeRange, setTimeRange] = useState<TimeRange>({
    from: new Date(Date.now() - 3600000),
    to: new Date(),
    quick: '1h'
  });
  const [isConnected, setIsConnected] = useState(false);
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());
  const [autoRefresh, setAutoRefresh] = useState(true);
  const [ws, setWs] = useState<WebSocket | null>(null);

  // WebSocket connection for real-time updates
  useEffect(() => {
    const connectWebSocket = () => {
      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      const wsUrl = `${protocol}//${window.location.host}/api/monitoring/stream`;
      const socket = new WebSocket(wsUrl);

      socket.onopen = () => {
        console.log('[MonitoringDashboard] WebSocket connected');
        setIsConnected(true);
        socket.send(JSON.stringify({ type: 'subscribe', channels: ['metrics', 'alerts', 'health'] }));
      };

      socket.onmessage = (event) => {
        try {
          const message: WebSocketMessage = JSON.parse(event.data);
          handleWebSocketMessage(message);
        } catch (error) {
          console.error('[MonitoringDashboard] Failed to parse WebSocket message:', error);
        }
      };

      socket.onerror = (error) => {
        console.error('[MonitoringDashboard] WebSocket error:', error);
        setIsConnected(false);
      };

      socket.onclose = () => {
        console.log('[MonitoringDashboard] WebSocket disconnected');
        setIsConnected(false);
        // Attempt to reconnect after 5 seconds
        setTimeout(connectWebSocket, 5000);
      };

      setWs(socket);
    };

    if (autoRefresh) {
      connectWebSocket();
    }

    return () => {
      if (ws) {
        ws.close();
      }
    };
  }, [autoRefresh]);

  // Handle WebSocket messages
  const handleWebSocketMessage = useCallback((message: WebSocketMessage) => {
    setLastUpdate(new Date());

    switch (message.type) {
      case 'health':
        setServices(prev => {
          const updated = [...prev];
          const index = updated.findIndex(s => s.id === message.data.id);
          if (index >= 0) {
            updated[index] = message.data;
          } else {
            updated.push(message.data);
          }
          return updated;
        });
        break;

      case 'alert':
        setAlerts(prev => {
          const updated = [...prev];
          const index = updated.findIndex(a => a.id === message.data.id);
          if (index >= 0) {
            updated[index] = message.data;
          } else {
            updated.unshift(message.data);
          }
          return updated.slice(0, 100); // Keep last 100 alerts
        });
        break;

      default:
        break;
    }
  }, []);

  // Calculate dashboard statistics
  useEffect(() => {
    const newStats: DashboardStats = {
      totalServices: services.length,
      healthyServices: services.filter(s => s.status === ServiceStatus.HEALTHY).length,
      degradedServices: services.filter(s => s.status === ServiceStatus.DEGRADED).length,
      downServices: services.filter(s => s.status === ServiceStatus.DOWN).length,
      activeAlerts: alerts.filter(a => a.state === 'active').length,
      criticalAlerts: alerts.filter(a => a.state === 'active' && a.severity === AlertSeverity.CRITICAL).length,
      averageResponseTime: services.length > 0
        ? services.reduce((sum, s) => sum + s.responseTime, 0) / services.length
        : 0,
      overallUptime: services.length > 0
        ? services.reduce((sum, s) => sum + s.uptime, 0) / services.length
        : 100
    };

    setStats(newStats);
  }, [services, alerts]);

  // Fetch initial data
  useEffect(() => {
    fetchDashboardData();
  }, [timeRange]);

  const fetchDashboardData = async () => {
    try {
      const [servicesRes, alertsRes] = await Promise.all([
        fetch('/api/monitoring/services'),
        fetch('/api/monitoring/alerts?state=active')
      ]);

      if (servicesRes.ok) {
        const servicesData = await servicesRes.json();
        setServices(servicesData);
      }

      if (alertsRes.ok) {
        const alertsData = await alertsRes.json();
        setAlerts(alertsData);
      }
    } catch (error) {
      console.error('[MonitoringDashboard] Failed to fetch dashboard data:', error);
    }
  };

  const handleTimeRangeChange = (range: TimeRange) => {
    setTimeRange(range);
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

  const getSeverityColor = (severity: AlertSeverity): string => {
    switch (severity) {
      case AlertSeverity.CRITICAL:
        return '#dc2626';
      case AlertSeverity.HIGH:
        return '#f59e0b';
      case AlertSeverity.MEDIUM:
        return '#3b82f6';
      case AlertSeverity.LOW:
        return '#6b7280';
      default:
        return '#6b7280';
    }
  };

  const formatUptime = (uptime: number): string => {
    return `${uptime.toFixed(3)}%`;
  };

  const formatResponseTime = (ms: number): string => {
    if (ms < 1000) {
      return `${ms.toFixed(0)}ms`;
    }
    return `${(ms / 1000).toFixed(2)}s`;
  };

  const quickTimeRanges = [
    { label: '5m', value: 5 * 60 * 1000 },
    { label: '15m', value: 15 * 60 * 1000 },
    { label: '1h', value: 60 * 60 * 1000 },
    { label: '6h', value: 6 * 60 * 60 * 1000 },
    { label: '24h', value: 24 * 60 * 60 * 1000 },
    { label: '7d', value: 7 * 24 * 60 * 60 * 1000 }
  ];

  return (
    <div className={`monitoring-dashboard ${className}`} style={styles.container}>
      {/* Header */}
      <div style={styles.header}>
        <div style={styles.headerLeft}>
          <h1 style={styles.title}>System Monitoring Dashboard</h1>
          <div style={styles.connectionStatus}>
            <div
              style={{
                ...styles.statusIndicator,
                backgroundColor: isConnected ? '#10b981' : '#ef4444'
              }}
            />
            <span style={styles.statusText}>
              {isConnected ? 'Connected' : 'Disconnected'}
            </span>
            <span style={styles.lastUpdateText}>
              Last update: {lastUpdate.toLocaleTimeString()}
            </span>
          </div>
        </div>

        <div style={styles.headerRight}>
          {/* Time Range Selector */}
          <div style={styles.timeRangeSelector}>
            {quickTimeRanges.map((range) => (
              <button
                key={range.label}
                style={{
                  ...styles.timeRangeButton,
                  ...(timeRange.quick === range.label ? styles.timeRangeButtonActive : {})
                }}
                onClick={() => handleTimeRangeChange({
                  from: new Date(Date.now() - range.value),
                  to: new Date(),
                  quick: range.label as any
                })}
              >
                {range.label}
              </button>
            ))}
          </div>

          {/* Auto-refresh Toggle */}
          <button
            style={{
              ...styles.refreshButton,
              ...(autoRefresh ? styles.refreshButtonActive : {})
            }}
            onClick={() => setAutoRefresh(!autoRefresh)}
          >
            {autoRefresh ? '● Auto' : '○ Manual'}
          </button>
        </div>
      </div>

      {/* Stats Overview */}
      <div style={styles.statsGrid}>
        <div style={styles.statCard}>
          <div style={styles.statValue}>{stats.totalServices}</div>
          <div style={styles.statLabel}>Total Services</div>
        </div>

        <div style={styles.statCard}>
          <div style={{ ...styles.statValue, color: '#10b981' }}>
            {stats.healthyServices}
          </div>
          <div style={styles.statLabel}>Healthy</div>
        </div>

        <div style={styles.statCard}>
          <div style={{ ...styles.statValue, color: '#f59e0b' }}>
            {stats.degradedServices}
          </div>
          <div style={styles.statLabel}>Degraded</div>
        </div>

        <div style={styles.statCard}>
          <div style={{ ...styles.statValue, color: '#ef4444' }}>
            {stats.downServices}
          </div>
          <div style={styles.statLabel}>Down</div>
        </div>

        <div style={styles.statCard}>
          <div style={{ ...styles.statValue, color: stats.activeAlerts > 0 ? '#f59e0b' : '#10b981' }}>
            {stats.activeAlerts}
          </div>
          <div style={styles.statLabel}>Active Alerts</div>
        </div>

        <div style={styles.statCard}>
          <div style={{ ...styles.statValue, color: stats.criticalAlerts > 0 ? '#dc2626' : '#10b981' }}>
            {stats.criticalAlerts}
          </div>
          <div style={styles.statLabel}>Critical</div>
        </div>

        <div style={styles.statCard}>
          <div style={styles.statValue}>
            {formatResponseTime(stats.averageResponseTime)}
          </div>
          <div style={styles.statLabel}>Avg Response</div>
        </div>

        <div style={styles.statCard}>
          <div style={{ ...styles.statValue, color: stats.overallUptime >= 99.9 ? '#10b981' : '#f59e0b' }}>
            {formatUptime(stats.overallUptime)}
          </div>
          <div style={styles.statLabel}>Overall Uptime</div>
        </div>
      </div>

      {/* Active Alerts */}
      {alerts.length > 0 && (
        <div style={styles.section}>
          <h2 style={styles.sectionTitle}>Active Alerts</h2>
          <div style={styles.alertsList}>
            {alerts.slice(0, 5).map((alert) => (
              <div key={alert.id} style={styles.alertItem}>
                <div
                  style={{
                    ...styles.alertSeverity,
                    backgroundColor: getSeverityColor(alert.severity)
                  }}
                />
                <div style={styles.alertContent}>
                  <div style={styles.alertHeader}>
                    <span style={styles.alertName}>{alert.name}</span>
                    <span style={styles.alertService}>{alert.service}</span>
                  </div>
                  <div style={styles.alertMessage}>{alert.message}</div>
                  <div style={styles.alertTime}>
                    {new Date(alert.triggeredAt).toLocaleString()}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Services Overview */}
      <div style={styles.section}>
        <h2 style={styles.sectionTitle}>Services Status</h2>
        <div style={styles.servicesGrid}>
          {services.map((service) => (
            <div key={service.id} style={styles.serviceCard}>
              <div style={styles.serviceHeader}>
                <div
                  style={{
                    ...styles.serviceStatus,
                    backgroundColor: getStatusColor(service.status)
                  }}
                />
                <span style={styles.serviceName}>{service.name}</span>
              </div>
              <div style={styles.serviceMetrics}>
                <div style={styles.serviceMetric}>
                  <span style={styles.metricLabel}>Uptime:</span>
                  <span style={styles.metricValue}>{formatUptime(service.uptime)}</span>
                </div>
                <div style={styles.serviceMetric}>
                  <span style={styles.metricLabel}>Response:</span>
                  <span style={styles.metricValue}>{formatResponseTime(service.responseTime)}</span>
                </div>
                <div style={styles.serviceMetric}>
                  <span style={styles.metricLabel}>CPU:</span>
                  <span style={styles.metricValue}>{service.metrics.cpu.toFixed(1)}%</span>
                </div>
                <div style={styles.serviceMetric}>
                  <span style={styles.metricLabel}>Memory:</span>
                  <span style={styles.metricValue}>{service.metrics.memory.toFixed(1)}%</span>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: {
    padding: '24px',
    backgroundColor: '#f9fafb',
    minHeight: '100vh',
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif'
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '24px',
    flexWrap: 'wrap',
    gap: '16px'
  },
  headerLeft: {
    flex: 1
  },
  headerRight: {
    display: 'flex',
    gap: '12px',
    alignItems: 'center'
  },
  title: {
    fontSize: '28px',
    fontWeight: 700,
    color: '#111827',
    margin: 0,
    marginBottom: '8px'
  },
  connectionStatus: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px'
  },
  statusIndicator: {
    width: '8px',
    height: '8px',
    borderRadius: '50%'
  },
  statusText: {
    fontSize: '14px',
    color: '#6b7280',
    fontWeight: 500
  },
  lastUpdateText: {
    fontSize: '12px',
    color: '#9ca3af',
    marginLeft: '8px'
  },
  timeRangeSelector: {
    display: 'flex',
    gap: '4px',
    backgroundColor: '#fff',
    borderRadius: '8px',
    padding: '4px',
    border: '1px solid #e5e7eb'
  },
  timeRangeButton: {
    padding: '6px 12px',
    border: 'none',
    background: 'transparent',
    borderRadius: '6px',
    cursor: 'pointer',
    fontSize: '13px',
    fontWeight: 500,
    color: '#6b7280',
    transition: 'all 0.2s'
  },
  timeRangeButtonActive: {
    backgroundColor: '#3b82f6',
    color: '#fff'
  },
  refreshButton: {
    padding: '8px 16px',
    border: '1px solid #e5e7eb',
    background: '#fff',
    borderRadius: '8px',
    cursor: 'pointer',
    fontSize: '13px',
    fontWeight: 500,
    color: '#6b7280',
    transition: 'all 0.2s'
  },
  refreshButtonActive: {
    backgroundColor: '#10b981',
    color: '#fff',
    borderColor: '#10b981'
  },
  statsGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))',
    gap: '16px',
    marginBottom: '24px'
  },
  statCard: {
    backgroundColor: '#fff',
    borderRadius: '12px',
    padding: '20px',
    border: '1px solid #e5e7eb',
    textAlign: 'center'
  },
  statValue: {
    fontSize: '32px',
    fontWeight: 700,
    color: '#111827',
    marginBottom: '4px'
  },
  statLabel: {
    fontSize: '13px',
    color: '#6b7280',
    fontWeight: 500
  },
  section: {
    marginBottom: '24px'
  },
  sectionTitle: {
    fontSize: '18px',
    fontWeight: 600,
    color: '#111827',
    marginBottom: '16px'
  },
  alertsList: {
    display: 'flex',
    flexDirection: 'column',
    gap: '12px'
  },
  alertItem: {
    display: 'flex',
    backgroundColor: '#fff',
    borderRadius: '8px',
    border: '1px solid #e5e7eb',
    overflow: 'hidden'
  },
  alertSeverity: {
    width: '4px',
    flexShrink: 0
  },
  alertContent: {
    padding: '16px',
    flex: 1
  },
  alertHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '8px'
  },
  alertName: {
    fontSize: '15px',
    fontWeight: 600,
    color: '#111827'
  },
  alertService: {
    fontSize: '13px',
    color: '#6b7280',
    backgroundColor: '#f3f4f6',
    padding: '2px 8px',
    borderRadius: '4px'
  },
  alertMessage: {
    fontSize: '14px',
    color: '#4b5563',
    marginBottom: '8px'
  },
  alertTime: {
    fontSize: '12px',
    color: '#9ca3af'
  },
  servicesGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))',
    gap: '16px'
  },
  serviceCard: {
    backgroundColor: '#fff',
    borderRadius: '8px',
    padding: '16px',
    border: '1px solid #e5e7eb'
  },
  serviceHeader: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    marginBottom: '12px'
  },
  serviceStatus: {
    width: '10px',
    height: '10px',
    borderRadius: '50%'
  },
  serviceName: {
    fontSize: '15px',
    fontWeight: 600,
    color: '#111827'
  },
  serviceMetrics: {
    display: 'grid',
    gridTemplateColumns: '1fr 1fr',
    gap: '8px'
  },
  serviceMetric: {
    display: 'flex',
    justifyContent: 'space-between',
    fontSize: '13px'
  },
  metricLabel: {
    color: '#6b7280'
  },
  metricValue: {
    color: '#111827',
    fontWeight: 500
  }
};

export default MonitoringDashboard;
