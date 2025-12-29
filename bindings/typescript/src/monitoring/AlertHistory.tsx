/**
 * CADDY v0.4.0 - Alert History Timeline
 * Historical alert tracking and analysis
 * @module monitoring/AlertHistory
 */

import React, { useEffect, useState, useCallback } from 'react';
import {
  Alert,
  AlertSeverity,
  AlertState,
  TimeRange
} from './types';

interface AlertHistoryProps {
  service?: string;
  timeRange?: TimeRange;
  severities?: AlertSeverity[];
  className?: string;
}

interface AlertStats {
  total: number;
  bySeverity: Record<AlertSeverity, number>;
  byState: Record<AlertState, number>;
  averageResolutionTime: number;
  mostCommonAlert: string;
}

export const AlertHistory: React.FC<AlertHistoryProps> = ({
  service,
  timeRange = { from: new Date(Date.now() - 86400000), to: new Date(), quick: '24h' },
  severities,
  className = ''
}) => {
  const [alerts, setAlerts] = useState<Alert[]>([]);
  const [stats, setStats] = useState<AlertStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [filterState, setFilterState] = useState<AlertState | 'all'>('all');
  const [filterSeverity, setFilterSeverity] = useState<AlertSeverity | 'all'>('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedAlert, setSelectedAlert] = useState<Alert | null>(null);
  const [viewMode, setViewMode] = useState<'timeline' | 'list'>('timeline');

  useEffect(() => {
    fetchAlertHistory();
  }, [service, timeRange, severities]);

  useEffect(() => {
    if (alerts.length > 0) {
      calculateStats();
    }
  }, [alerts]);

  const fetchAlertHistory = async () => {
    try {
      setLoading(true);

      const params = new URLSearchParams({
        from: timeRange.from.toISOString(),
        to: timeRange.to.toISOString(),
        ...(service && { service }),
        ...(severities && { severities: severities.join(',') })
      });

      const response = await fetch(`/api/monitoring/alerts/history?${params}`);
      if (!response.ok) throw new Error('Failed to fetch alert history');

      const data = await response.json();
      setAlerts(data);
    } catch (error) {
      console.error('[AlertHistory] Failed to fetch alerts:', error);
    } finally {
      setLoading(false);
    }
  };

  const calculateStats = () => {
    const bySeverity = alerts.reduce((acc, alert) => {
      acc[alert.severity] = (acc[alert.severity] || 0) + 1;
      return acc;
    }, {} as Record<AlertSeverity, number>);

    const byState = alerts.reduce((acc, alert) => {
      acc[alert.state] = (acc[alert.state] || 0) + 1;
      return acc;
    }, {} as Record<AlertState, number>);

    const resolvedAlerts = alerts.filter(a => a.resolvedAt);
    const totalResolutionTime = resolvedAlerts.reduce((sum, alert) => {
      if (alert.resolvedAt) {
        return sum + (new Date(alert.resolvedAt).getTime() - new Date(alert.triggeredAt).getTime());
      }
      return sum;
    }, 0);

    const averageResolutionTime = resolvedAlerts.length > 0
      ? totalResolutionTime / resolvedAlerts.length
      : 0;

    const alertCounts = alerts.reduce((acc, alert) => {
      acc[alert.name] = (acc[alert.name] || 0) + 1;
      return acc;
    }, {} as Record<string, number>);

    const mostCommonAlert = Object.entries(alertCounts).sort((a, b) => b[1] - a[1])[0]?.[0] || '';

    setStats({
      total: alerts.length,
      bySeverity,
      byState,
      averageResolutionTime,
      mostCommonAlert
    });
  };

  const acknowledgeAlert = async (alertId: string) => {
    try {
      const response = await fetch(`/api/monitoring/alerts/${alertId}/acknowledge`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ acknowledgedBy: 'current-user' })
      });

      if (!response.ok) throw new Error('Failed to acknowledge alert');

      const updatedAlert = await response.json();
      setAlerts(prev => prev.map(a => a.id === alertId ? updatedAlert : a));
    } catch (error) {
      console.error('[AlertHistory] Failed to acknowledge alert:', error);
    }
  };

  const resolveAlert = async (alertId: string) => {
    try {
      const response = await fetch(`/api/monitoring/alerts/${alertId}/resolve`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ resolvedBy: 'current-user' })
      });

      if (!response.ok) throw new Error('Failed to resolve alert');

      const updatedAlert = await response.json();
      setAlerts(prev => prev.map(a => a.id === alertId ? updatedAlert : a));
    } catch (error) {
      console.error('[AlertHistory] Failed to resolve alert:', error);
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
        return '#9ca3af';
    }
  };

  const getStateIcon = (state: AlertState): string => {
    switch (state) {
      case AlertState.ACTIVE:
        return '🔴';
      case AlertState.ACKNOWLEDGED:
        return '🟡';
      case AlertState.RESOLVED:
        return '🟢';
      case AlertState.SILENCED:
        return '🔇';
      default:
        return '⚪';
    }
  };

  const formatDuration = (ms: number): string => {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (days > 0) return `${days}d ${hours % 24}h`;
    if (hours > 0) return `${hours}h ${minutes % 60}m`;
    if (minutes > 0) return `${minutes}m ${seconds % 60}s`;
    return `${seconds}s`;
  };

  const filteredAlerts = alerts.filter(alert => {
    const matchesState = filterState === 'all' || alert.state === filterState;
    const matchesSeverity = filterSeverity === 'all' || alert.severity === filterSeverity;
    const matchesSearch =
      alert.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      alert.message.toLowerCase().includes(searchQuery.toLowerCase()) ||
      alert.service.toLowerCase().includes(searchQuery.toLowerCase());

    return matchesState && matchesSeverity && matchesSearch;
  });

  const groupAlertsByDate = (alerts: Alert[]): Record<string, Alert[]> => {
    return alerts.reduce((acc, alert) => {
      const date = new Date(alert.triggeredAt).toLocaleDateString();
      if (!acc[date]) {
        acc[date] = [];
      }
      acc[date].push(alert);
      return acc;
    }, {} as Record<string, Alert[]>);
  };

  const groupedAlerts = groupAlertsByDate(filteredAlerts);

  if (loading) {
    return (
      <div style={styles.loading}>
        <div style={styles.spinner} />
        <p>Loading alert history...</p>
      </div>
    );
  }

  return (
    <div className={`alert-history ${className}`} style={styles.container}>
      {/* Header */}
      <div style={styles.header}>
        <h2 style={styles.title}>Alert History</h2>
        <div style={styles.viewToggle}>
          <button
            style={{
              ...styles.viewButton,
              ...(viewMode === 'timeline' ? styles.viewButtonActive : {})
            }}
            onClick={() => setViewMode('timeline')}
          >
            Timeline
          </button>
          <button
            style={{
              ...styles.viewButton,
              ...(viewMode === 'list' ? styles.viewButtonActive : {})
            }}
            onClick={() => setViewMode('list')}
          >
            List
          </button>
        </div>
      </div>

      {/* Stats */}
      {stats && (
        <div style={styles.stats}>
          <div style={styles.statCard}>
            <div style={styles.statValue}>{stats.total}</div>
            <div style={styles.statLabel}>Total Alerts</div>
          </div>
          <div style={styles.statCard}>
            <div style={{ ...styles.statValue, color: '#dc2626' }}>
              {stats.bySeverity[AlertSeverity.CRITICAL] || 0}
            </div>
            <div style={styles.statLabel}>Critical</div>
          </div>
          <div style={styles.statCard}>
            <div style={{ ...styles.statValue, color: '#10b981' }}>
              {stats.byState[AlertState.RESOLVED] || 0}
            </div>
            <div style={styles.statLabel}>Resolved</div>
          </div>
          <div style={styles.statCard}>
            <div style={styles.statValue}>
              {formatDuration(stats.averageResolutionTime)}
            </div>
            <div style={styles.statLabel}>Avg Resolution</div>
          </div>
        </div>
      )}

      {/* Filters */}
      <div style={styles.filters}>
        <input
          type="text"
          placeholder="Search alerts..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          style={styles.searchInput}
        />

        <select
          value={filterSeverity}
          onChange={(e) => setFilterSeverity(e.target.value as any)}
          style={styles.select}
        >
          <option value="all">All Severities</option>
          {Object.values(AlertSeverity).map(sev => (
            <option key={sev} value={sev}>{sev.toUpperCase()}</option>
          ))}
        </select>

        <select
          value={filterState}
          onChange={(e) => setFilterState(e.target.value as any)}
          style={styles.select}
        >
          <option value="all">All States</option>
          {Object.values(AlertState).map(state => (
            <option key={state} value={state}>{state.toUpperCase()}</option>
          ))}
        </select>

        <button style={styles.refreshButton} onClick={fetchAlertHistory}>
          Refresh
        </button>
      </div>

      {/* Alert List */}
      {filteredAlerts.length === 0 ? (
        <div style={styles.emptyState}>
          <p>No alerts found matching your criteria</p>
        </div>
      ) : viewMode === 'timeline' ? (
        <div style={styles.timeline}>
          {Object.entries(groupedAlerts).map(([date, dateAlerts]) => (
            <div key={date} style={styles.timelineGroup}>
              <div style={styles.timelineDate}>{date}</div>
              {dateAlerts.map(alert => (
                <div
                  key={alert.id}
                  style={styles.timelineItem}
                  onClick={() => setSelectedAlert(alert)}
                >
                  <div
                    style={{
                      ...styles.timelineDot,
                      backgroundColor: getSeverityColor(alert.severity)
                    }}
                  />
                  <div style={styles.timelineContent}>
                    <div style={styles.timelineHeader}>
                      <span style={styles.timelineTime}>
                        {new Date(alert.triggeredAt).toLocaleTimeString()}
                      </span>
                      <span style={styles.timelineState}>
                        {getStateIcon(alert.state)} {alert.state}
                      </span>
                    </div>
                    <div style={styles.timelineTitle}>{alert.name}</div>
                    <div style={styles.timelineMessage}>{alert.message}</div>
                    <div style={styles.timelineFooter}>
                      <span style={styles.timelineService}>{alert.service}</span>
                      {alert.resolvedAt && (
                        <span style={styles.timelineDuration}>
                          Resolved in {formatDuration(
                            new Date(alert.resolvedAt).getTime() - new Date(alert.triggeredAt).getTime()
                          )}
                        </span>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          ))}
        </div>
      ) : (
        <div style={styles.list}>
          {filteredAlerts.map(alert => (
            <div
              key={alert.id}
              style={styles.listItem}
              onClick={() => setSelectedAlert(alert)}
            >
              <div
                style={{
                  ...styles.listIndicator,
                  backgroundColor: getSeverityColor(alert.severity)
                }}
              />
              <div style={styles.listContent}>
                <div style={styles.listHeader}>
                  <div>
                    <span style={styles.listTitle}>{alert.name}</span>
                    <span style={styles.listState}>
                      {getStateIcon(alert.state)} {alert.state}
                    </span>
                  </div>
                  <div style={styles.listActions}>
                    {alert.state === AlertState.ACTIVE && (
                      <>
                        <button
                          style={styles.actionButton}
                          onClick={(e) => {
                            e.stopPropagation();
                            acknowledgeAlert(alert.id);
                          }}
                        >
                          Acknowledge
                        </button>
                        <button
                          style={styles.actionButton}
                          onClick={(e) => {
                            e.stopPropagation();
                            resolveAlert(alert.id);
                          }}
                        >
                          Resolve
                        </button>
                      </>
                    )}
                  </div>
                </div>
                <div style={styles.listMessage}>{alert.message}</div>
                <div style={styles.listFooter}>
                  <span>{alert.service}</span>
                  <span>{new Date(alert.triggeredAt).toLocaleString()}</span>
                  {alert.resolvedAt && (
                    <span style={styles.resolved}>
                      Resolved: {new Date(alert.resolvedAt).toLocaleString()}
                    </span>
                  )}
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Alert Details Modal */}
      {selectedAlert && (
        <div style={styles.modal} onClick={() => setSelectedAlert(null)}>
          <div style={styles.modalContent} onClick={(e) => e.stopPropagation()}>
            <div style={styles.modalHeader}>
              <h3>{selectedAlert.name}</h3>
              <button
                style={styles.modalClose}
                onClick={() => setSelectedAlert(null)}
              >
                ×
              </button>
            </div>
            <div style={styles.modalBody}>
              <div style={styles.detailRow}>
                <strong>Severity:</strong>
                <span
                  style={{
                    color: getSeverityColor(selectedAlert.severity),
                    fontWeight: 600
                  }}
                >
                  {selectedAlert.severity.toUpperCase()}
                </span>
              </div>
              <div style={styles.detailRow}>
                <strong>State:</strong> {getStateIcon(selectedAlert.state)} {selectedAlert.state}
              </div>
              <div style={styles.detailRow}>
                <strong>Service:</strong> {selectedAlert.service}
              </div>
              <div style={styles.detailRow}>
                <strong>Message:</strong> {selectedAlert.message}
              </div>
              <div style={styles.detailRow}>
                <strong>Triggered:</strong> {new Date(selectedAlert.triggeredAt).toLocaleString()}
              </div>
              {selectedAlert.acknowledgedAt && (
                <div style={styles.detailRow}>
                  <strong>Acknowledged:</strong> {new Date(selectedAlert.acknowledgedAt).toLocaleString()}
                  {selectedAlert.acknowledgedBy && ` by ${selectedAlert.acknowledgedBy}`}
                </div>
              )}
              {selectedAlert.resolvedAt && (
                <div style={styles.detailRow}>
                  <strong>Resolved:</strong> {new Date(selectedAlert.resolvedAt).toLocaleString()}
                  {selectedAlert.resolvedBy && ` by ${selectedAlert.resolvedBy}`}
                </div>
              )}
              {selectedAlert.threshold && (
                <div style={styles.detailRow}>
                  <strong>Threshold:</strong> {selectedAlert.threshold.metric} {selectedAlert.threshold.operator} {selectedAlert.threshold.value}
                </div>
              )}
              {Object.keys(selectedAlert.metadata).length > 0 && (
                <div style={styles.detailRow}>
                  <strong>Metadata:</strong>
                  <pre style={styles.metadata}>
                    {JSON.stringify(selectedAlert.metadata, null, 2)}
                  </pre>
                </div>
              )}
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
  viewToggle: {
    display: 'flex',
    gap: '4px',
    backgroundColor: '#f3f4f6',
    borderRadius: '8px',
    padding: '4px'
  },
  viewButton: {
    padding: '6px 16px',
    border: 'none',
    background: 'transparent',
    borderRadius: '6px',
    fontSize: '14px',
    fontWeight: 500,
    color: '#6b7280',
    cursor: 'pointer',
    transition: 'all 0.2s'
  },
  viewButtonActive: {
    backgroundColor: '#fff',
    color: '#111827'
  },
  stats: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))',
    gap: '16px',
    marginBottom: '24px'
  },
  statCard: {
    backgroundColor: '#fff',
    border: '1px solid #e5e7eb',
    borderRadius: '8px',
    padding: '20px',
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
  filters: {
    display: 'flex',
    gap: '12px',
    marginBottom: '24px',
    flexWrap: 'wrap'
  },
  searchInput: {
    flex: 1,
    minWidth: '200px',
    padding: '8px 12px',
    border: '1px solid #e5e7eb',
    borderRadius: '6px',
    fontSize: '14px',
    outline: 'none'
  },
  select: {
    padding: '8px 12px',
    border: '1px solid #e5e7eb',
    borderRadius: '6px',
    fontSize: '14px',
    outline: 'none',
    backgroundColor: '#fff'
  },
  refreshButton: {
    padding: '8px 16px',
    backgroundColor: '#3b82f6',
    color: '#fff',
    border: 'none',
    borderRadius: '6px',
    fontSize: '14px',
    fontWeight: 500,
    cursor: 'pointer'
  },
  emptyState: {
    textAlign: 'center',
    padding: '48px',
    backgroundColor: '#fff',
    border: '1px solid #e5e7eb',
    borderRadius: '8px',
    color: '#6b7280'
  },
  timeline: {
    display: 'flex',
    flexDirection: 'column',
    gap: '24px'
  },
  timelineGroup: {},
  timelineDate: {
    fontSize: '14px',
    fontWeight: 600,
    color: '#111827',
    marginBottom: '12px',
    padding: '8px 12px',
    backgroundColor: '#f3f4f6',
    borderRadius: '6px',
    display: 'inline-block'
  },
  timelineItem: {
    display: 'flex',
    gap: '16px',
    marginBottom: '12px',
    marginLeft: '20px',
    cursor: 'pointer',
    position: 'relative'
  },
  timelineDot: {
    width: '12px',
    height: '12px',
    borderRadius: '50%',
    marginTop: '6px',
    flexShrink: 0,
    position: 'relative',
    zIndex: 1
  },
  timelineContent: {
    flex: 1,
    backgroundColor: '#fff',
    border: '1px solid #e5e7eb',
    borderRadius: '8px',
    padding: '16px'
  },
  timelineHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '8px'
  },
  timelineTime: {
    fontSize: '12px',
    color: '#6b7280',
    fontWeight: 500
  },
  timelineState: {
    fontSize: '12px',
    color: '#6b7280',
    textTransform: 'uppercase'
  },
  timelineTitle: {
    fontSize: '16px',
    fontWeight: 600,
    color: '#111827',
    marginBottom: '4px'
  },
  timelineMessage: {
    fontSize: '14px',
    color: '#4b5563',
    marginBottom: '8px'
  },
  timelineFooter: {
    display: 'flex',
    justifyContent: 'space-between',
    fontSize: '12px',
    color: '#6b7280'
  },
  timelineService: {
    backgroundColor: '#f3f4f6',
    padding: '2px 8px',
    borderRadius: '4px'
  },
  timelineDuration: {},
  list: {
    display: 'flex',
    flexDirection: 'column',
    gap: '12px'
  },
  listItem: {
    display: 'flex',
    backgroundColor: '#fff',
    border: '1px solid #e5e7eb',
    borderRadius: '8px',
    overflow: 'hidden',
    cursor: 'pointer',
    transition: 'box-shadow 0.2s'
  },
  listIndicator: {
    width: '4px',
    flexShrink: 0
  },
  listContent: {
    flex: 1,
    padding: '16px'
  },
  listHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '8px'
  },
  listTitle: {
    fontSize: '16px',
    fontWeight: 600,
    color: '#111827',
    marginRight: '12px'
  },
  listState: {
    fontSize: '12px',
    color: '#6b7280',
    textTransform: 'uppercase'
  },
  listActions: {
    display: 'flex',
    gap: '8px'
  },
  actionButton: {
    padding: '4px 12px',
    backgroundColor: '#f3f4f6',
    border: '1px solid #e5e7eb',
    borderRadius: '4px',
    fontSize: '12px',
    fontWeight: 500,
    cursor: 'pointer',
    color: '#374151'
  },
  listMessage: {
    fontSize: '14px',
    color: '#4b5563',
    marginBottom: '8px'
  },
  listFooter: {
    display: 'flex',
    gap: '16px',
    fontSize: '12px',
    color: '#6b7280'
  },
  resolved: {
    color: '#10b981'
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
    maxWidth: '600px',
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
    padding: '12px 0',
    borderBottom: '1px solid #f3f4f6',
    fontSize: '14px',
    display: 'flex',
    flexDirection: 'column',
    gap: '4px'
  },
  metadata: {
    backgroundColor: '#f3f4f6',
    padding: '12px',
    borderRadius: '6px',
    fontSize: '12px',
    overflow: 'auto',
    marginTop: '8px'
  }
};

export default AlertHistory;
