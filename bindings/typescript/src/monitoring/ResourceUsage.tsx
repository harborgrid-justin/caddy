/**
 * CADDY v0.4.0 - Resource Usage Charts
 * Real-time resource utilization visualization
 * @module monitoring/ResourceUsage
 */

import React, { useEffect, useState } from 'react';
import { ResourceUsage as ResourceUsageType, WebSocketMessage } from './types';

interface ResourceUsageProps {
  service?: string;
  className?: string;
}

export const ResourceUsage: React.FC<ResourceUsageProps> = ({
  service,
  className = ''
}) => {
  const [resources, setResources] = useState<ResourceUsageType[]>([]);
  const [selectedService, setSelectedService] = useState<string | null>(service || null);
  const [ws, setWs] = useState<WebSocket | null>(null);
  const [isConnected, setIsConnected] = useState(false);

  useEffect(() => {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/api/monitoring/resources/stream`;
    const socket = new WebSocket(wsUrl);

    socket.onopen = () => {
      console.log('[ResourceUsage] WebSocket connected');
      setIsConnected(true);
      socket.send(JSON.stringify({
        type: 'subscribe',
        service: selectedService
      }));
    };

    socket.onmessage = (event) => {
      try {
        const message: WebSocketMessage = JSON.parse(event.data);
        if (message.type === 'metric') {
          updateResourceData(message.data);
        }
      } catch (error) {
        console.error('[ResourceUsage] Failed to parse WebSocket message:', error);
      }
    };

    socket.onerror = (error) => {
      console.error('[ResourceUsage] WebSocket error:', error);
      setIsConnected(false);
    };

    socket.onclose = () => {
      console.log('[ResourceUsage] WebSocket disconnected');
      setIsConnected(false);
    };

    setWs(socket);

    return () => {
      socket.close();
    };
  }, [selectedService]);

  useEffect(() => {
    fetchResourceData();
  }, [selectedService]);

  const fetchResourceData = async () => {
    try {
      const params = selectedService ? `?service=${selectedService}` : '';
      const response = await fetch(`/api/monitoring/resources${params}`);
      if (!response.ok) throw new Error('Failed to fetch resource data');

      const data = await response.json();
      setResources(data);
    } catch (error) {
      console.error('[ResourceUsage] Failed to fetch resources:', error);
    }
  };

  const updateResourceData = (newData: ResourceUsageType) => {
    setResources(prev => {
      const updated = [...prev];
      const index = updated.findIndex(r => r.service === newData.service);

      if (index >= 0) {
        updated[index] = newData;
      } else {
        updated.push(newData);
      }

      return updated;
    });
  };

  const getUsageColor = (percentage: number): string => {
    if (percentage >= 90) return '#ef4444';
    if (percentage >= 75) return '#f59e0b';
    if (percentage >= 50) return '#3b82f6';
    return '#10b981';
  };

  const formatBytes = (bytes: number): string => {
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let size = bytes;
    let unitIndex = 0;

    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }

    return `${size.toFixed(2)} ${units[unitIndex]}`;
  };

  const formatBytesPerSec = (bytes: number): string => {
    return `${formatBytes(bytes)}/s`;
  };

  const uniqueServices = Array.from(new Set(resources.map(r => r.service)));

  return (
    <div className={`resource-usage ${className}`} style={styles.container}>
      {/* Header */}
      <div style={styles.header}>
        <h2 style={styles.title}>Resource Usage</h2>
        <div style={styles.connectionStatus}>
          <div
            style={{
              ...styles.statusDot,
              backgroundColor: isConnected ? '#10b981' : '#ef4444'
            }}
          />
          <span style={styles.statusText}>
            {isConnected ? 'Live' : 'Disconnected'}
          </span>
        </div>
      </div>

      {/* Service Filter */}
      {uniqueServices.length > 1 && (
        <div style={styles.serviceFilter}>
          <button
            style={{
              ...styles.filterButton,
              ...(selectedService === null ? styles.filterButtonActive : {})
            }}
            onClick={() => setSelectedService(null)}
          >
            All Services
          </button>
          {uniqueServices.map(svc => (
            <button
              key={svc}
              style={{
                ...styles.filterButton,
                ...(selectedService === svc ? styles.filterButtonActive : {})
              }}
              onClick={() => setSelectedService(svc)}
            >
              {svc}
            </button>
          ))}
        </div>
      )}

      {/* Resource Cards */}
      <div style={styles.resourceGrid}>
        {resources
          .filter(r => !selectedService || r.service === selectedService)
          .map((resource, index) => (
            <div key={index} style={styles.resourceCard}>
              <div style={styles.resourceHeader}>
                <h3 style={styles.resourceService}>{resource.service}</h3>
                <span style={styles.resourceTimestamp}>
                  {new Date(resource.timestamp).toLocaleTimeString()}
                </span>
              </div>

              {/* CPU Usage */}
              <div style={styles.metricSection}>
                <div style={styles.metricHeader}>
                  <span style={styles.metricLabel}>CPU</span>
                  <span
                    style={{
                      ...styles.metricValue,
                      color: getUsageColor(resource.cpu.percentage)
                    }}
                  >
                    {resource.cpu.percentage.toFixed(1)}%
                  </span>
                </div>
                <div style={styles.progressBar}>
                  <div
                    style={{
                      ...styles.progressFill,
                      width: `${Math.min(resource.cpu.percentage, 100)}%`,
                      backgroundColor: getUsageColor(resource.cpu.percentage)
                    }}
                  />
                </div>
                <div style={styles.metricDetails}>
                  {resource.cpu.used.toFixed(2)} / {resource.cpu.total.toFixed(2)} cores
                </div>
              </div>

              {/* Memory Usage */}
              <div style={styles.metricSection}>
                <div style={styles.metricHeader}>
                  <span style={styles.metricLabel}>Memory</span>
                  <span
                    style={{
                      ...styles.metricValue,
                      color: getUsageColor(resource.memory.percentage)
                    }}
                  >
                    {resource.memory.percentage.toFixed(1)}%
                  </span>
                </div>
                <div style={styles.progressBar}>
                  <div
                    style={{
                      ...styles.progressFill,
                      width: `${Math.min(resource.memory.percentage, 100)}%`,
                      backgroundColor: getUsageColor(resource.memory.percentage)
                    }}
                  />
                </div>
                <div style={styles.metricDetails}>
                  {formatBytes(resource.memory.used)} / {formatBytes(resource.memory.total)}
                </div>
              </div>

              {/* Disk Usage */}
              <div style={styles.metricSection}>
                <div style={styles.metricHeader}>
                  <span style={styles.metricLabel}>Disk</span>
                  <span
                    style={{
                      ...styles.metricValue,
                      color: getUsageColor(resource.disk.percentage)
                    }}
                  >
                    {resource.disk.percentage.toFixed(1)}%
                  </span>
                </div>
                <div style={styles.progressBar}>
                  <div
                    style={{
                      ...styles.progressFill,
                      width: `${Math.min(resource.disk.percentage, 100)}%`,
                      backgroundColor: getUsageColor(resource.disk.percentage)
                    }}
                  />
                </div>
                <div style={styles.metricDetails}>
                  {formatBytes(resource.disk.used)} / {formatBytes(resource.disk.total)}
                </div>
              </div>

              {/* Network Usage */}
              <div style={styles.metricSection}>
                <div style={styles.metricHeader}>
                  <span style={styles.metricLabel}>Network</span>
                </div>
                <div style={styles.networkStats}>
                  <div style={styles.networkStat}>
                    <span style={styles.networkLabel}>↓ In:</span>
                    <span style={styles.networkValue}>
                      {formatBytesPerSec(resource.network.bytesIn)}
                    </span>
                  </div>
                  <div style={styles.networkStat}>
                    <span style={styles.networkLabel}>↑ Out:</span>
                    <span style={styles.networkValue}>
                      {formatBytesPerSec(resource.network.bytesOut)}
                    </span>
                  </div>
                  <div style={styles.networkStat}>
                    <span style={styles.networkLabel}>Packets In:</span>
                    <span style={styles.networkValue}>
                      {resource.network.packetsIn.toLocaleString()}/s
                    </span>
                  </div>
                  <div style={styles.networkStat}>
                    <span style={styles.networkLabel}>Packets Out:</span>
                    <span style={styles.networkValue}>
                      {resource.network.packetsOut.toLocaleString()}/s
                    </span>
                  </div>
                  {(resource.network.errorsIn > 0 || resource.network.errorsOut > 0) && (
                    <div style={styles.networkErrors}>
                      <span style={{ color: '#ef4444' }}>
                        ⚠ Errors: {resource.network.errorsIn + resource.network.errorsOut}
                      </span>
                    </div>
                  )}
                </div>
              </div>
            </div>
          ))}
      </div>

      {/* Summary Stats */}
      {resources.length > 1 && (
        <div style={styles.summary}>
          <h3 style={styles.summaryTitle}>Overall Summary</h3>
          <div style={styles.summaryGrid}>
            <div style={styles.summaryCard}>
              <div style={styles.summaryLabel}>Avg CPU</div>
              <div
                style={{
                  ...styles.summaryValue,
                  color: getUsageColor(
                    resources.reduce((sum, r) => sum + r.cpu.percentage, 0) / resources.length
                  )
                }}
              >
                {(resources.reduce((sum, r) => sum + r.cpu.percentage, 0) / resources.length).toFixed(1)}%
              </div>
            </div>

            <div style={styles.summaryCard}>
              <div style={styles.summaryLabel}>Avg Memory</div>
              <div
                style={{
                  ...styles.summaryValue,
                  color: getUsageColor(
                    resources.reduce((sum, r) => sum + r.memory.percentage, 0) / resources.length
                  )
                }}
              >
                {(resources.reduce((sum, r) => sum + r.memory.percentage, 0) / resources.length).toFixed(1)}%
              </div>
            </div>

            <div style={styles.summaryCard}>
              <div style={styles.summaryLabel}>Avg Disk</div>
              <div
                style={{
                  ...styles.summaryValue,
                  color: getUsageColor(
                    resources.reduce((sum, r) => sum + r.disk.percentage, 0) / resources.length
                  )
                }}
              >
                {(resources.reduce((sum, r) => sum + r.disk.percentage, 0) / resources.length).toFixed(1)}%
              </div>
            </div>

            <div style={styles.summaryCard}>
              <div style={styles.summaryLabel}>Total Network</div>
              <div style={styles.summaryValue}>
                ↓ {formatBytesPerSec(resources.reduce((sum, r) => sum + r.network.bytesIn, 0))}
                <br />
                ↑ {formatBytesPerSec(resources.reduce((sum, r) => sum + r.network.bytesOut, 0))}
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
  connectionStatus: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px'
  },
  statusDot: {
    width: '8px',
    height: '8px',
    borderRadius: '50%'
  },
  statusText: {
    fontSize: '13px',
    color: '#6b7280',
    fontWeight: 500
  },
  serviceFilter: {
    display: 'flex',
    gap: '8px',
    marginBottom: '24px',
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
    transition: 'all 0.2s'
  },
  filterButtonActive: {
    backgroundColor: '#3b82f6',
    color: '#fff',
    borderColor: '#3b82f6'
  },
  resourceGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(400px, 1fr))',
    gap: '20px',
    marginBottom: '32px'
  },
  resourceCard: {
    backgroundColor: '#fff',
    border: '1px solid #e5e7eb',
    borderRadius: '12px',
    padding: '24px'
  },
  resourceHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '20px',
    paddingBottom: '12px',
    borderBottom: '2px solid #e5e7eb'
  },
  resourceService: {
    fontSize: '18px',
    fontWeight: 600,
    color: '#111827',
    margin: 0
  },
  resourceTimestamp: {
    fontSize: '12px',
    color: '#9ca3af'
  },
  metricSection: {
    marginBottom: '20px'
  },
  metricHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '8px'
  },
  metricLabel: {
    fontSize: '14px',
    fontWeight: 600,
    color: '#374151'
  },
  metricValue: {
    fontSize: '20px',
    fontWeight: 700
  },
  progressBar: {
    height: '10px',
    backgroundColor: '#f3f4f6',
    borderRadius: '5px',
    overflow: 'hidden',
    marginBottom: '6px'
  },
  progressFill: {
    height: '100%',
    borderRadius: '5px',
    transition: 'width 0.3s ease, background-color 0.3s ease'
  },
  metricDetails: {
    fontSize: '12px',
    color: '#6b7280'
  },
  networkStats: {
    display: 'grid',
    gridTemplateColumns: '1fr 1fr',
    gap: '8px',
    marginTop: '8px'
  },
  networkStat: {
    display: 'flex',
    justifyContent: 'space-between',
    fontSize: '13px',
    padding: '6px 0'
  },
  networkLabel: {
    color: '#6b7280'
  },
  networkValue: {
    color: '#111827',
    fontWeight: 500
  },
  networkErrors: {
    gridColumn: '1 / -1',
    fontSize: '13px',
    fontWeight: 600,
    padding: '6px 12px',
    backgroundColor: '#fef2f2',
    borderRadius: '6px',
    marginTop: '4px'
  },
  summary: {
    backgroundColor: '#fff',
    border: '1px solid #e5e7eb',
    borderRadius: '12px',
    padding: '24px'
  },
  summaryTitle: {
    fontSize: '18px',
    fontWeight: 600,
    color: '#111827',
    marginBottom: '16px'
  },
  summaryGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
    gap: '16px'
  },
  summaryCard: {
    padding: '16px',
    backgroundColor: '#f9fafb',
    borderRadius: '8px',
    textAlign: 'center'
  },
  summaryLabel: {
    fontSize: '13px',
    color: '#6b7280',
    fontWeight: 500,
    marginBottom: '8px'
  },
  summaryValue: {
    fontSize: '24px',
    fontWeight: 700,
    color: '#111827'
  }
};

export default ResourceUsage;
