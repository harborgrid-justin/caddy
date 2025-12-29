/**
 * CADDY v0.4.0 - Public Status Page Configuration
 * Configure and manage public-facing status pages
 * @module monitoring/StatusPage
 */

import React, { useEffect, useState } from 'react';
import { StatusPageConfig, StatusPageService, ServiceStatus, Incident, IncidentStatus } from './types';

interface StatusPageProps {
  configId?: string;
  preview?: boolean;
  className?: string;
}

export const StatusPage: React.FC<StatusPageProps> = ({
  configId,
  preview = false,
  className = ''
}) => {
  const [config, setConfig] = useState<StatusPageConfig | null>(null);
  const [services, setServices] = useState<Array<StatusPageService & { currentStatus: ServiceStatus }>>([]);
  const [incidents, setIncidents] = useState<Incident[]>([]);
  const [loading, setLoading] = useState(true);
  const [editMode, setEditMode] = useState(false);

  useEffect(() => {
    if (configId) {
      fetchStatusPageConfig();
      fetchServiceStatuses();
      fetchActiveIncidents();
    }
  }, [configId]);

  const fetchStatusPageConfig = async () => {
    try {
      setLoading(true);
      const response = await fetch(`/api/monitoring/status-page/${configId}`);
      if (!response.ok) throw new Error('Failed to fetch status page config');

      const data = await response.json();
      setConfig(data);
    } catch (error) {
      console.error('[StatusPage] Failed to fetch config:', error);
    } finally {
      setLoading(false);
    }
  };

  const fetchServiceStatuses = async () => {
    try {
      const response = await fetch('/api/monitoring/services');
      if (!response.ok) throw new Error('Failed to fetch service statuses');

      const data = await response.json();

      if (config) {
        const servicesWithStatus = config.services.map(svc => {
          const serviceData = data.find((d: any) => d.id === svc.id);
          return {
            ...svc,
            currentStatus: serviceData?.status || ServiceStatus.UNKNOWN
          };
        });
        setServices(servicesWithStatus);
      }
    } catch (error) {
      console.error('[StatusPage] Failed to fetch service statuses:', error);
    }
  };

  const fetchActiveIncidents = async () => {
    try {
      const response = await fetch('/api/monitoring/incidents?status=active,investigating,identified,monitoring');
      if (!response.ok) throw new Error('Failed to fetch incidents');

      const data = await response.json();
      setIncidents(data);
    } catch (error) {
      console.error('[StatusPage] Failed to fetch incidents:', error);
    }
  };

  const updateConfig = async (updates: Partial<StatusPageConfig>) => {
    try {
      const response = await fetch(`/api/monitoring/status-page/${configId}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(updates)
      });

      if (!response.ok) throw new Error('Failed to update config');

      const updatedConfig = await response.json();
      setConfig(updatedConfig);
    } catch (error) {
      console.error('[StatusPage] Failed to update config:', error);
      alert('Failed to update status page configuration');
    }
  };

  const getStatusColor = (status: ServiceStatus): string => {
    switch (status) {
      case ServiceStatus.HEALTHY:
        return config?.theme.primaryColor || '#10b981';
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
    const labels = {
      [ServiceStatus.HEALTHY]: 'Operational',
      [ServiceStatus.DEGRADED]: 'Degraded Performance',
      [ServiceStatus.DOWN]: 'Major Outage',
      [ServiceStatus.MAINTENANCE]: 'Maintenance',
      [ServiceStatus.UNKNOWN]: 'Unknown'
    };
    return labels[status];
  };

  const getOverallStatus = (): { status: ServiceStatus; message: string } => {
    const statuses = services.map(s => s.currentStatus);

    if (statuses.includes(ServiceStatus.DOWN)) {
      return {
        status: ServiceStatus.DOWN,
        message: 'Some systems are experiencing major outages'
      };
    }

    if (statuses.includes(ServiceStatus.DEGRADED)) {
      return {
        status: ServiceStatus.DEGRADED,
        message: 'Some systems are experiencing degraded performance'
      };
    }

    if (statuses.includes(ServiceStatus.MAINTENANCE)) {
      return {
        status: ServiceStatus.MAINTENANCE,
        message: 'Scheduled maintenance in progress'
      };
    }

    if (statuses.every(s => s === ServiceStatus.HEALTHY)) {
      return {
        status: ServiceStatus.HEALTHY,
        message: 'All systems operational'
      };
    }

    return {
      status: ServiceStatus.UNKNOWN,
      message: 'System status unknown'
    };
  };

  const groupServicesByGroup = () => {
    const grouped: Record<string, Array<StatusPageService & { currentStatus: ServiceStatus }>> = {
      '': [] // Default group
    };

    services.forEach(service => {
      const group = service.group || '';
      if (!grouped[group]) {
        grouped[group] = [];
      }
      grouped[group].push(service);
    });

    // Sort by displayOrder
    Object.keys(grouped).forEach(group => {
      grouped[group].sort((a, b) => a.displayOrder - b.displayOrder);
    });

    return grouped;
  };

  if (loading || !config) {
    return (
      <div style={styles.loading}>
        <div style={styles.spinner} />
        <p>Loading status page...</p>
      </div>
    );
  }

  const overall = getOverallStatus();
  const groupedServices = groupServicesByGroup();

  return (
    <div
      className={`status-page ${className}`}
      style={{
        ...styles.container,
        backgroundColor: config.theme.backgroundColor,
        color: config.theme.textColor
      }}
    >
      {/* Header */}
      <div style={styles.header}>
        {config.logo && (
          <img src={config.logo} alt={config.title} style={styles.logo} />
        )}
        <h1 style={{ ...styles.title, color: config.theme.textColor }}>{config.title}</h1>
        {config.description && (
          <p style={{ ...styles.description, color: config.theme.textColor }}>
            {config.description}
          </p>
        )}

        {!preview && (
          <div style={styles.headerActions}>
            <button
              style={styles.editButton}
              onClick={() => setEditMode(!editMode)}
            >
              {editMode ? 'View Mode' : 'Edit Mode'}
            </button>
          </div>
        )}
      </div>

      {/* Overall Status */}
      <div
        style={{
          ...styles.overallStatus,
          borderColor: getStatusColor(overall.status)
        }}
      >
        <div
          style={{
            ...styles.statusIndicator,
            backgroundColor: getStatusColor(overall.status)
          }}
        />
        <div style={styles.overallStatusContent}>
          <div style={{ ...styles.overallStatusLabel, color: config.theme.textColor }}>
            {overall.message}
          </div>
          <div style={styles.lastUpdated}>
            Last updated: {new Date().toLocaleString()}
          </div>
        </div>
      </div>

      {/* Active Incidents */}
      {config.showIncidents && incidents.length > 0 && (
        <div style={styles.section}>
          <h2 style={{ ...styles.sectionTitle, color: config.theme.textColor }}>
            Active Incidents
          </h2>
          {incidents.map(incident => (
            <div key={incident.id} style={styles.incidentCard}>
              <div style={styles.incidentHeader}>
                <h3 style={styles.incidentTitle}>{incident.title}</h3>
                <span style={styles.incidentStatus}>
                  {incident.status.replace('_', ' ').toUpperCase()}
                </span>
              </div>
              <p style={styles.incidentDescription}>{incident.description}</p>
              <div style={styles.incidentMeta}>
                <span>Started: {new Date(incident.startedAt).toLocaleString()}</span>
                {incident.affectedServices.length > 0 && (
                  <span>Affected: {incident.affectedServices.join(', ')}</span>
                )}
              </div>

              {incident.timeline.length > 0 && (
                <div style={styles.incidentTimeline}>
                  <h4 style={styles.timelineTitle}>Updates:</h4>
                  {incident.timeline.slice(-3).reverse().map(entry => (
                    <div key={entry.id} style={styles.timelineEntry}>
                      <div style={styles.timelineTime}>
                        {new Date(entry.timestamp).toLocaleString()}
                      </div>
                      <div style={styles.timelineMessage}>{entry.message}</div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          ))}
        </div>
      )}

      {/* Services Status */}
      <div style={styles.section}>
        <h2 style={{ ...styles.sectionTitle, color: config.theme.textColor }}>
          System Status
        </h2>

        {Object.entries(groupedServices).map(([group, groupServices]) => (
          <div key={group} style={styles.serviceGroup}>
            {group && (
              <h3 style={{ ...styles.groupTitle, color: config.theme.textColor }}>
                {group}
              </h3>
            )}

            {groupServices.map(service => (
              <div key={service.id} style={styles.serviceRow}>
                <div style={styles.serviceInfo}>
                  <div style={styles.serviceName}>{service.name}</div>
                  {service.description && (
                    <div style={styles.serviceDescription}>{service.description}</div>
                  )}
                </div>

                <div style={styles.serviceStatus}>
                  <div
                    style={{
                      ...styles.statusDot,
                      backgroundColor: getStatusColor(service.currentStatus)
                    }}
                  />
                  <span style={styles.statusText}>
                    {getStatusLabel(service.currentStatus)}
                  </span>
                </div>
              </div>
            ))}
          </div>
        ))}
      </div>

      {/* Metrics (if enabled) */}
      {config.showMetrics && (
        <div style={styles.section}>
          <h2 style={{ ...styles.sectionTitle, color: config.theme.textColor }}>
            Performance Metrics
          </h2>
          <div style={styles.metricsGrid}>
            <div style={styles.metricCard}>
              <div style={styles.metricValue}>99.98%</div>
              <div style={styles.metricLabel}>Uptime (30d)</div>
            </div>
            <div style={styles.metricCard}>
              <div style={styles.metricValue}>124ms</div>
              <div style={styles.metricLabel}>Avg Response</div>
            </div>
            <div style={styles.metricCard}>
              <div style={styles.metricValue}>0.01%</div>
              <div style={styles.metricLabel}>Error Rate</div>
            </div>
          </div>
        </div>
      )}

      {/* Footer */}
      <div style={styles.footer}>
        <p style={styles.footerText}>
          Powered by CADDY v0.4.0
          {config.customDomain && ` • ${config.customDomain}`}
        </p>
      </div>

      {/* Edit Panel */}
      {editMode && (
        <div style={styles.editPanel}>
          <h3 style={styles.editPanelTitle}>Configuration</h3>

          <div style={styles.formGroup}>
            <label style={styles.label}>Title</label>
            <input
              type="text"
              value={config.title}
              onChange={(e) => updateConfig({ title: e.target.value })}
              style={styles.input}
            />
          </div>

          <div style={styles.formGroup}>
            <label style={styles.label}>Description</label>
            <textarea
              value={config.description}
              onChange={(e) => updateConfig({ description: e.target.value })}
              style={{ ...styles.input, minHeight: '60px' }}
            />
          </div>

          <div style={styles.formGroup}>
            <label style={styles.label}>Public URL</label>
            <input
              type="text"
              value={config.publicUrl}
              onChange={(e) => updateConfig({ publicUrl: e.target.value })}
              style={styles.input}
            />
          </div>

          <div style={styles.formGroup}>
            <label style={styles.checkbox}>
              <input
                type="checkbox"
                checked={config.showMetrics}
                onChange={(e) => updateConfig({ showMetrics: e.target.checked })}
              />
              Show Performance Metrics
            </label>
          </div>

          <div style={styles.formGroup}>
            <label style={styles.checkbox}>
              <input
                type="checkbox"
                checked={config.showIncidents}
                onChange={(e) => updateConfig({ showIncidents: e.target.checked })}
              />
              Show Active Incidents
            </label>
          </div>

          <div style={styles.themeSection}>
            <h4 style={styles.themeSectionTitle}>Theme</h4>
            <div style={styles.colorInputs}>
              <div style={styles.colorGroup}>
                <label style={styles.label}>Primary Color</label>
                <input
                  type="color"
                  value={config.theme.primaryColor}
                  onChange={(e) => updateConfig({
                    theme: { ...config.theme, primaryColor: e.target.value }
                  })}
                  style={styles.colorInput}
                />
              </div>
              <div style={styles.colorGroup}>
                <label style={styles.label}>Background</label>
                <input
                  type="color"
                  value={config.theme.backgroundColor}
                  onChange={(e) => updateConfig({
                    theme: { ...config.theme, backgroundColor: e.target.value }
                  })}
                  style={styles.colorInput}
                />
              </div>
              <div style={styles.colorGroup}>
                <label style={styles.label}>Text Color</label>
                <input
                  type="color"
                  value={config.theme.textColor}
                  onChange={(e) => updateConfig({
                    theme: { ...config.theme, textColor: e.target.value }
                  })}
                  style={styles.colorInput}
                />
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
    maxWidth: '900px',
    margin: '0 auto',
    padding: '40px 20px',
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
    minHeight: '100vh'
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
    textAlign: 'center',
    marginBottom: '40px'
  },
  logo: {
    maxHeight: '60px',
    marginBottom: '20px'
  },
  title: {
    fontSize: '36px',
    fontWeight: 700,
    margin: 0,
    marginBottom: '12px'
  },
  description: {
    fontSize: '16px',
    margin: 0,
    opacity: 0.8
  },
  headerActions: {
    marginTop: '20px'
  },
  editButton: {
    padding: '8px 16px',
    backgroundColor: '#3b82f6',
    color: '#fff',
    border: 'none',
    borderRadius: '6px',
    fontSize: '14px',
    fontWeight: 500,
    cursor: 'pointer'
  },
  overallStatus: {
    display: 'flex',
    alignItems: 'center',
    gap: '16px',
    padding: '24px',
    backgroundColor: 'rgba(255, 255, 255, 0.5)',
    borderRadius: '12px',
    marginBottom: '32px',
    borderLeft: '4px solid'
  },
  statusIndicator: {
    width: '16px',
    height: '16px',
    borderRadius: '50%'
  },
  overallStatusContent: {
    flex: 1
  },
  overallStatusLabel: {
    fontSize: '20px',
    fontWeight: 600,
    marginBottom: '4px'
  },
  lastUpdated: {
    fontSize: '13px',
    opacity: 0.7
  },
  section: {
    marginBottom: '40px'
  },
  sectionTitle: {
    fontSize: '24px',
    fontWeight: 600,
    marginBottom: '20px'
  },
  incidentCard: {
    backgroundColor: 'rgba(239, 68, 68, 0.1)',
    border: '1px solid #fecaca',
    borderRadius: '8px',
    padding: '20px',
    marginBottom: '16px'
  },
  incidentHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    marginBottom: '12px'
  },
  incidentTitle: {
    fontSize: '18px',
    fontWeight: 600,
    margin: 0,
    color: '#991b1b'
  },
  incidentStatus: {
    fontSize: '11px',
    fontWeight: 600,
    padding: '4px 8px',
    backgroundColor: '#fee2e2',
    color: '#991b1b',
    borderRadius: '10px'
  },
  incidentDescription: {
    fontSize: '14px',
    marginBottom: '12px',
    color: '#7f1d1d'
  },
  incidentMeta: {
    fontSize: '12px',
    color: '#991b1b',
    display: 'flex',
    gap: '16px',
    marginBottom: '12px'
  },
  incidentTimeline: {
    paddingTop: '12px',
    borderTop: '1px solid #fecaca'
  },
  timelineTitle: {
    fontSize: '14px',
    fontWeight: 600,
    marginBottom: '8px',
    color: '#991b1b'
  },
  timelineEntry: {
    padding: '8px 0',
    fontSize: '13px',
    color: '#7f1d1d'
  },
  timelineTime: {
    fontWeight: 600,
    marginBottom: '2px'
  },
  timelineMessage: {},
  serviceGroup: {
    marginBottom: '24px'
  },
  groupTitle: {
    fontSize: '16px',
    fontWeight: 600,
    marginBottom: '12px',
    opacity: 0.8
  },
  serviceRow: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '16px',
    backgroundColor: 'rgba(255, 255, 255, 0.3)',
    borderRadius: '8px',
    marginBottom: '8px'
  },
  serviceInfo: {
    flex: 1
  },
  serviceName: {
    fontSize: '15px',
    fontWeight: 500,
    marginBottom: '4px'
  },
  serviceDescription: {
    fontSize: '13px',
    opacity: 0.7
  },
  serviceStatus: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px'
  },
  statusDot: {
    width: '10px',
    height: '10px',
    borderRadius: '50%'
  },
  statusText: {
    fontSize: '14px',
    fontWeight: 500
  },
  metricsGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
    gap: '16px'
  },
  metricCard: {
    backgroundColor: 'rgba(255, 255, 255, 0.3)',
    borderRadius: '8px',
    padding: '20px',
    textAlign: 'center'
  },
  metricValue: {
    fontSize: '32px',
    fontWeight: 700,
    marginBottom: '8px'
  },
  metricLabel: {
    fontSize: '14px',
    opacity: 0.7
  },
  footer: {
    textAlign: 'center',
    padding: '20px 0',
    borderTop: '1px solid rgba(0, 0, 0, 0.1)',
    marginTop: '40px'
  },
  footerText: {
    fontSize: '13px',
    opacity: 0.6,
    margin: 0
  },
  editPanel: {
    position: 'fixed',
    right: 0,
    top: 0,
    bottom: 0,
    width: '350px',
    backgroundColor: '#fff',
    boxShadow: '-2px 0 8px rgba(0, 0, 0, 0.1)',
    padding: '24px',
    overflowY: 'auto',
    zIndex: 1000
  },
  editPanelTitle: {
    fontSize: '18px',
    fontWeight: 600,
    marginBottom: '20px',
    color: '#111827'
  },
  formGroup: {
    marginBottom: '16px'
  },
  label: {
    display: 'block',
    fontSize: '13px',
    fontWeight: 500,
    color: '#374151',
    marginBottom: '6px'
  },
  input: {
    width: '100%',
    padding: '8px 12px',
    border: '1px solid #d1d5db',
    borderRadius: '6px',
    fontSize: '14px',
    outline: 'none',
    boxSizing: 'border-box'
  },
  checkbox: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    fontSize: '14px',
    color: '#374151',
    cursor: 'pointer'
  },
  themeSection: {
    paddingTop: '16px',
    borderTop: '1px solid #e5e7eb',
    marginTop: '16px'
  },
  themeSectionTitle: {
    fontSize: '14px',
    fontWeight: 600,
    color: '#111827',
    marginBottom: '12px'
  },
  colorInputs: {
    display: 'flex',
    flexDirection: 'column',
    gap: '12px'
  },
  colorGroup: {},
  colorInput: {
    width: '100%',
    height: '40px',
    border: '1px solid #d1d5db',
    borderRadius: '6px',
    cursor: 'pointer'
  }
};

export default StatusPage;
