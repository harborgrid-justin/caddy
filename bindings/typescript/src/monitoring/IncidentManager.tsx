/**
 * CADDY v0.4.0 - Incident Manager
 * Incident tracking, resolution, and post-mortem management
 * @module monitoring/IncidentManager
 */

import React, { useEffect, useState } from 'react';
import {
  Incident,
  IncidentStatus,
  IncidentTimelineEntry,
  AlertSeverity
} from './types';

interface IncidentManagerProps {
  service?: string;
  onIncidentClick?: (incident: Incident) => void;
  className?: string;
}

export const IncidentManager: React.FC<IncidentManagerProps> = ({
  service,
  onIncidentClick,
  className = ''
}) => {
  const [incidents, setIncidents] = useState<Incident[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedIncident, setSelectedIncident] = useState<Incident | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [newTimelineEntry, setNewTimelineEntry] = useState('');
  const [filterStatus, setFilterStatus] = useState<IncidentStatus | 'all'>('all');
  const [formData, setFormData] = useState<Partial<Incident>>({
    title: '',
    description: '',
    status: IncidentStatus.INVESTIGATING,
    severity: AlertSeverity.MEDIUM,
    affectedServices: [],
    timeline: []
  });

  useEffect(() => {
    fetchIncidents();
  }, [service]);

  const fetchIncidents = async () => {
    try {
      setLoading(true);
      const params = service ? `?service=${service}` : '';
      const response = await fetch(`/api/monitoring/incidents${params}`);

      if (!response.ok) throw new Error('Failed to fetch incidents');

      const data = await response.json();
      setIncidents(data);
    } catch (error) {
      console.error('[IncidentManager] Failed to fetch incidents:', error);
    } finally {
      setLoading(false);
    }
  };

  const createIncident = async () => {
    try {
      const response = await fetch('/api/monitoring/incidents', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          ...formData,
          startedAt: new Date(),
          createdBy: 'current-user',
          timeline: [{
            id: `entry-${Date.now()}`,
            timestamp: new Date(),
            type: 'update',
            message: 'Incident created',
            user: 'current-user'
          }],
          metadata: {}
        })
      });

      if (!response.ok) throw new Error('Failed to create incident');

      const newIncident = await response.json();
      setIncidents(prev => [newIncident, ...prev]);
      setShowCreateModal(false);
      resetForm();
    } catch (error) {
      console.error('[IncidentManager] Failed to create incident:', error);
      alert('Failed to create incident');
    }
  };

  const updateIncidentStatus = async (incidentId: string, status: IncidentStatus) => {
    try {
      const response = await fetch(`/api/monitoring/incidents/${incidentId}/status`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          status,
          ...(status === IncidentStatus.IDENTIFIED && { identifiedAt: new Date() }),
          ...(status === IncidentStatus.RESOLVED && { resolvedAt: new Date() })
        })
      });

      if (!response.ok) throw new Error('Failed to update status');

      const updatedIncident = await response.json();
      setIncidents(prev => prev.map(i => i.id === incidentId ? updatedIncident : i));
      if (selectedIncident?.id === incidentId) {
        setSelectedIncident(updatedIncident);
      }
    } catch (error) {
      console.error('[IncidentManager] Failed to update status:', error);
    }
  };

  const addTimelineEntry = async (incidentId: string, message: string, type: IncidentTimelineEntry['type'] = 'update') => {
    try {
      const response = await fetch(`/api/monitoring/incidents/${incidentId}/timeline`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          type,
          message,
          user: 'current-user'
        })
      });

      if (!response.ok) throw new Error('Failed to add timeline entry');

      const updatedIncident = await response.json();
      setIncidents(prev => prev.map(i => i.id === incidentId ? updatedIncident : i));
      if (selectedIncident?.id === incidentId) {
        setSelectedIncident(updatedIncident);
      }
      setNewTimelineEntry('');
    } catch (error) {
      console.error('[IncidentManager] Failed to add timeline entry:', error);
    }
  };

  const updateIncidentResolution = async (incidentId: string, rootCause: string, resolution: string) => {
    try {
      const response = await fetch(`/api/monitoring/incidents/${incidentId}/resolution`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ rootCause, resolution })
      });

      if (!response.ok) throw new Error('Failed to update resolution');

      const updatedIncident = await response.json();
      setIncidents(prev => prev.map(i => i.id === incidentId ? updatedIncident : i));
      if (selectedIncident?.id === incidentId) {
        setSelectedIncident(updatedIncident);
      }
    } catch (error) {
      console.error('[IncidentManager] Failed to update resolution:', error);
    }
  };

  const resetForm = () => {
    setFormData({
      title: '',
      description: '',
      status: IncidentStatus.INVESTIGATING,
      severity: AlertSeverity.MEDIUM,
      affectedServices: [],
      timeline: []
    });
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!formData.title || !formData.description) {
      alert('Please fill in all required fields');
      return;
    }
    createIncident();
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

  const getStatusColor = (status: IncidentStatus): string => {
    switch (status) {
      case IncidentStatus.INVESTIGATING:
        return '#ef4444';
      case IncidentStatus.IDENTIFIED:
        return '#f59e0b';
      case IncidentStatus.MONITORING:
        return '#3b82f6';
      case IncidentStatus.RESOLVED:
        return '#10b981';
      default:
        return '#6b7280';
    }
  };

  const getTimelineIcon = (type: IncidentTimelineEntry['type']): string => {
    switch (type) {
      case 'update':
        return '📝';
      case 'status_change':
        return '🔄';
      case 'comment':
        return '💬';
      case 'action':
        return '⚡';
      default:
        return '•';
    }
  };

  const formatDuration = (start: Date, end?: Date): string => {
    const endTime = end ? new Date(end).getTime() : Date.now();
    const startTime = new Date(start).getTime();
    const ms = endTime - startTime;

    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (days > 0) return `${days}d ${hours % 24}h`;
    if (hours > 0) return `${hours}h ${minutes % 60}m`;
    if (minutes > 0) return `${minutes}m`;
    return `${seconds}s`;
  };

  const filteredIncidents = incidents.filter(
    incident => filterStatus === 'all' || incident.status === filterStatus
  );

  const stats = {
    total: incidents.length,
    investigating: incidents.filter(i => i.status === IncidentStatus.INVESTIGATING).length,
    identified: incidents.filter(i => i.status === IncidentStatus.IDENTIFIED).length,
    monitoring: incidents.filter(i => i.status === IncidentStatus.MONITORING).length,
    resolved: incidents.filter(i => i.status === IncidentStatus.RESOLVED).length
  };

  if (loading) {
    return (
      <div style={styles.loading}>
        <div style={styles.spinner} />
        <p>Loading incidents...</p>
      </div>
    );
  }

  return (
    <div className={`incident-manager ${className}`} style={styles.container}>
      {/* Header */}
      <div style={styles.header}>
        <div>
          <h2 style={styles.title}>Incident Management</h2>
          <p style={styles.subtitle}>Track and resolve system incidents</p>
        </div>
        <button
          style={styles.createButton}
          onClick={() => {
            resetForm();
            setShowCreateModal(true);
          }}
        >
          + Create Incident
        </button>
      </div>

      {/* Stats */}
      <div style={styles.stats}>
        <div style={styles.statCard}>
          <div style={styles.statValue}>{stats.total}</div>
          <div style={styles.statLabel}>Total</div>
        </div>
        <div style={styles.statCard}>
          <div style={{ ...styles.statValue, color: '#ef4444' }}>
            {stats.investigating}
          </div>
          <div style={styles.statLabel}>Investigating</div>
        </div>
        <div style={styles.statCard}>
          <div style={{ ...styles.statValue, color: '#f59e0b' }}>
            {stats.identified}
          </div>
          <div style={styles.statLabel}>Identified</div>
        </div>
        <div style={styles.statCard}>
          <div style={{ ...styles.statValue, color: '#3b82f6' }}>
            {stats.monitoring}
          </div>
          <div style={styles.statLabel}>Monitoring</div>
        </div>
        <div style={styles.statCard}>
          <div style={{ ...styles.statValue, color: '#10b981' }}>
            {stats.resolved}
          </div>
          <div style={styles.statLabel}>Resolved</div>
        </div>
      </div>

      {/* Status Filter */}
      <div style={styles.filters}>
        {(['all', ...Object.values(IncidentStatus)] as const).map(status => (
          <button
            key={status}
            style={{
              ...styles.filterButton,
              ...(filterStatus === status ? styles.filterButtonActive : {})
            }}
            onClick={() => setFilterStatus(status)}
          >
            {status === 'all' ? 'All Incidents' : status.replace('_', ' ').toUpperCase()}
          </button>
        ))}
      </div>

      {/* Incidents List */}
      {filteredIncidents.length === 0 ? (
        <div style={styles.emptyState}>
          <p>No incidents found</p>
        </div>
      ) : (
        <div style={styles.incidentsList}>
          {filteredIncidents.map(incident => (
            <div
              key={incident.id}
              style={styles.incidentCard}
              onClick={() => {
                setSelectedIncident(incident);
                if (onIncidentClick) onIncidentClick(incident);
              }}
            >
              <div
                style={{
                  ...styles.incidentIndicator,
                  backgroundColor: getSeverityColor(incident.severity)
                }}
              />
              <div style={styles.incidentContent}>
                <div style={styles.incidentHeader}>
                  <div style={styles.incidentTitle}>
                    <span>{incident.title}</span>
                    <span
                      style={{
                        ...styles.statusBadge,
                        backgroundColor: `${getStatusColor(incident.status)}20`,
                        color: getStatusColor(incident.status)
                      }}
                    >
                      {incident.status.replace('_', ' ')}
                    </span>
                  </div>
                  <div style={styles.incidentMeta}>
                    <span style={styles.duration}>
                      {formatDuration(incident.startedAt, incident.resolvedAt)}
                    </span>
                  </div>
                </div>

                <p style={styles.incidentDescription}>{incident.description}</p>

                <div style={styles.incidentFooter}>
                  <div style={styles.services}>
                    {incident.affectedServices.slice(0, 3).map((svc, idx) => (
                      <span key={idx} style={styles.serviceTag}>
                        {svc}
                      </span>
                    ))}
                    {incident.affectedServices.length > 3 && (
                      <span style={styles.serviceTag}>
                        +{incident.affectedServices.length - 3} more
                      </span>
                    )}
                  </div>
                  <div style={styles.timestamps}>
                    <span>Started: {new Date(incident.startedAt).toLocaleString()}</span>
                    {incident.impactedUsers && (
                      <span>{incident.impactedUsers.toLocaleString()} users impacted</span>
                    )}
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Create Incident Modal */}
      {showCreateModal && (
        <div style={styles.modal} onClick={() => setShowCreateModal(false)}>
          <div style={styles.modalContent} onClick={(e) => e.stopPropagation()}>
            <div style={styles.modalHeader}>
              <h3>Create New Incident</h3>
              <button
                style={styles.modalClose}
                onClick={() => setShowCreateModal(false)}
              >
                ×
              </button>
            </div>

            <form onSubmit={handleSubmit} style={styles.form}>
              <div style={styles.formGroup}>
                <label style={styles.label}>Title *</label>
                <input
                  type="text"
                  value={formData.title}
                  onChange={(e) => setFormData({ ...formData, title: e.target.value })}
                  style={styles.input}
                  placeholder="Brief incident title"
                  required
                />
              </div>

              <div style={styles.formGroup}>
                <label style={styles.label}>Description *</label>
                <textarea
                  value={formData.description}
                  onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                  style={{ ...styles.input, minHeight: '100px' }}
                  placeholder="Detailed incident description"
                  required
                />
              </div>

              <div style={styles.formRow}>
                <div style={styles.formGroup}>
                  <label style={styles.label}>Severity *</label>
                  <select
                    value={formData.severity}
                    onChange={(e) => setFormData({ ...formData, severity: e.target.value as AlertSeverity })}
                    style={styles.select}
                  >
                    {Object.values(AlertSeverity).map(sev => (
                      <option key={sev} value={sev}>{sev.toUpperCase()}</option>
                    ))}
                  </select>
                </div>

                <div style={styles.formGroup}>
                  <label style={styles.label}>Status *</label>
                  <select
                    value={formData.status}
                    onChange={(e) => setFormData({ ...formData, status: e.target.value as IncidentStatus })}
                    style={styles.select}
                  >
                    {Object.values(IncidentStatus).map(status => (
                      <option key={status} value={status}>
                        {status.replace('_', ' ').toUpperCase()}
                      </option>
                    ))}
                  </select>
                </div>
              </div>

              <div style={styles.formGroup}>
                <label style={styles.label}>Affected Services</label>
                <input
                  type="text"
                  value={formData.affectedServices?.join(', ')}
                  onChange={(e) => setFormData({
                    ...formData,
                    affectedServices: e.target.value.split(',').map(s => s.trim()).filter(Boolean)
                  })}
                  style={styles.input}
                  placeholder="service1, service2, service3"
                />
              </div>

              <div style={styles.formActions}>
                <button
                  type="button"
                  style={styles.cancelButton}
                  onClick={() => setShowCreateModal(false)}
                >
                  Cancel
                </button>
                <button type="submit" style={styles.submitButton}>
                  Create Incident
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Incident Details Modal */}
      {selectedIncident && !showCreateModal && (
        <div style={styles.modal} onClick={() => setSelectedIncident(null)}>
          <div
            style={{ ...styles.modalContent, maxWidth: '800px' }}
            onClick={(e) => e.stopPropagation()}
          >
            <div style={styles.modalHeader}>
              <div>
                <h3>{selectedIncident.title}</h3>
                <div style={styles.modalSubtitle}>
                  <span
                    style={{
                      ...styles.statusBadge,
                      backgroundColor: `${getStatusColor(selectedIncident.status)}20`,
                      color: getStatusColor(selectedIncident.status)
                    }}
                  >
                    {selectedIncident.status.replace('_', ' ')}
                  </span>
                  <span
                    style={{
                      ...styles.statusBadge,
                      backgroundColor: `${getSeverityColor(selectedIncident.severity)}20`,
                      color: getSeverityColor(selectedIncident.severity)
                    }}
                  >
                    {selectedIncident.severity.toUpperCase()}
                  </span>
                </div>
              </div>
              <button
                style={styles.modalClose}
                onClick={() => setSelectedIncident(null)}
              >
                ×
              </button>
            </div>

            <div style={styles.modalBody}>
              {/* Status Actions */}
              <div style={styles.statusActions}>
                {Object.values(IncidentStatus).map(status => (
                  <button
                    key={status}
                    style={{
                      ...styles.statusButton,
                      ...(selectedIncident.status === status ? {
                        backgroundColor: getStatusColor(status),
                        color: '#fff'
                      } : {})
                    }}
                    onClick={() => updateIncidentStatus(selectedIncident.id, status)}
                  >
                    {status.replace('_', ' ')}
                  </button>
                ))}
              </div>

              {/* Details */}
              <div style={styles.detailSection}>
                <h4 style={styles.sectionTitle}>Details</h4>
                <p>{selectedIncident.description}</p>
                <div style={styles.detailGrid}>
                  <div style={styles.detailItem}>
                    <strong>Started:</strong> {new Date(selectedIncident.startedAt).toLocaleString()}
                  </div>
                  {selectedIncident.identifiedAt && (
                    <div style={styles.detailItem}>
                      <strong>Identified:</strong> {new Date(selectedIncident.identifiedAt).toLocaleString()}
                    </div>
                  )}
                  {selectedIncident.resolvedAt && (
                    <div style={styles.detailItem}>
                      <strong>Resolved:</strong> {new Date(selectedIncident.resolvedAt).toLocaleString()}
                    </div>
                  )}
                  <div style={styles.detailItem}>
                    <strong>Duration:</strong> {formatDuration(selectedIncident.startedAt, selectedIncident.resolvedAt)}
                  </div>
                  {selectedIncident.assignedTo && (
                    <div style={styles.detailItem}>
                      <strong>Assigned:</strong> {selectedIncident.assignedTo}
                    </div>
                  )}
                  {selectedIncident.impactedUsers && (
                    <div style={styles.detailItem}>
                      <strong>Impacted Users:</strong> {selectedIncident.impactedUsers.toLocaleString()}
                    </div>
                  )}
                </div>

                {selectedIncident.affectedServices.length > 0 && (
                  <div style={styles.detailItem}>
                    <strong>Affected Services:</strong>
                    <div style={styles.services}>
                      {selectedIncident.affectedServices.map((svc, idx) => (
                        <span key={idx} style={styles.serviceTag}>{svc}</span>
                      ))}
                    </div>
                  </div>
                )}
              </div>

              {/* Timeline */}
              <div style={styles.detailSection}>
                <h4 style={styles.sectionTitle}>Timeline</h4>
                <div style={styles.timeline}>
                  {selectedIncident.timeline.map(entry => (
                    <div key={entry.id} style={styles.timelineEntry}>
                      <div style={styles.timelineIcon}>{getTimelineIcon(entry.type)}</div>
                      <div style={styles.timelineEntryContent}>
                        <div style={styles.timelineEntryHeader}>
                          <span style={styles.timelineEntryUser}>{entry.user}</span>
                          <span style={styles.timelineEntryTime}>
                            {new Date(entry.timestamp).toLocaleString()}
                          </span>
                        </div>
                        <div style={styles.timelineEntryMessage}>{entry.message}</div>
                      </div>
                    </div>
                  ))}
                </div>

                {/* Add Timeline Entry */}
                <div style={styles.addEntry}>
                  <input
                    type="text"
                    value={newTimelineEntry}
                    onChange={(e) => setNewTimelineEntry(e.target.value)}
                    placeholder="Add timeline entry..."
                    style={styles.input}
                    onKeyPress={(e) => {
                      if (e.key === 'Enter' && newTimelineEntry.trim()) {
                        addTimelineEntry(selectedIncident.id, newTimelineEntry);
                      }
                    }}
                  />
                  <button
                    style={styles.addButton}
                    onClick={() => {
                      if (newTimelineEntry.trim()) {
                        addTimelineEntry(selectedIncident.id, newTimelineEntry);
                      }
                    }}
                  >
                    Add
                  </button>
                </div>
              </div>

              {/* Resolution */}
              {(selectedIncident.rootCause || selectedIncident.resolution || selectedIncident.status === IncidentStatus.RESOLVED) && (
                <div style={styles.detailSection}>
                  <h4 style={styles.sectionTitle}>Resolution</h4>
                  {selectedIncident.rootCause && (
                    <div style={styles.resolutionItem}>
                      <strong>Root Cause:</strong>
                      <p>{selectedIncident.rootCause}</p>
                    </div>
                  )}
                  {selectedIncident.resolution && (
                    <div style={styles.resolutionItem}>
                      <strong>Resolution:</strong>
                      <p>{selectedIncident.resolution}</p>
                    </div>
                  )}
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
    alignItems: 'flex-start',
    marginBottom: '24px'
  },
  title: {
    fontSize: '24px',
    fontWeight: 700,
    color: '#111827',
    margin: 0,
    marginBottom: '4px'
  },
  subtitle: {
    fontSize: '14px',
    color: '#6b7280',
    margin: 0
  },
  createButton: {
    padding: '10px 20px',
    backgroundColor: '#3b82f6',
    color: '#fff',
    border: 'none',
    borderRadius: '8px',
    fontSize: '14px',
    fontWeight: 600,
    cursor: 'pointer'
  },
  stats: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(120px, 1fr))',
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
    gap: '8px',
    marginBottom: '24px',
    flexWrap: 'wrap'
  },
  filterButton: {
    padding: '8px 16px',
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
  emptyState: {
    textAlign: 'center',
    padding: '48px',
    backgroundColor: '#fff',
    border: '1px solid #e5e7eb',
    borderRadius: '8px',
    color: '#6b7280'
  },
  incidentsList: {
    display: 'flex',
    flexDirection: 'column',
    gap: '12px'
  },
  incidentCard: {
    display: 'flex',
    backgroundColor: '#fff',
    border: '1px solid #e5e7eb',
    borderRadius: '8px',
    overflow: 'hidden',
    cursor: 'pointer',
    transition: 'box-shadow 0.2s'
  },
  incidentIndicator: {
    width: '4px',
    flexShrink: 0
  },
  incidentContent: {
    flex: 1,
    padding: '20px'
  },
  incidentHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    marginBottom: '12px'
  },
  incidentTitle: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
    fontSize: '18px',
    fontWeight: 600,
    color: '#111827'
  },
  statusBadge: {
    fontSize: '11px',
    fontWeight: 600,
    padding: '4px 10px',
    borderRadius: '12px',
    textTransform: 'uppercase'
  },
  incidentMeta: {
    fontSize: '13px',
    color: '#6b7280'
  },
  duration: {
    fontWeight: 500
  },
  incidentDescription: {
    fontSize: '14px',
    color: '#4b5563',
    marginBottom: '16px',
    lineHeight: 1.5
  },
  incidentFooter: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    gap: '16px'
  },
  services: {
    display: 'flex',
    flexWrap: 'wrap',
    gap: '6px'
  },
  serviceTag: {
    fontSize: '11px',
    padding: '4px 8px',
    backgroundColor: '#f3f4f6',
    borderRadius: '4px',
    color: '#374151'
  },
  timestamps: {
    fontSize: '12px',
    color: '#6b7280',
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'flex-end',
    gap: '4px'
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
    maxHeight: '90vh',
    overflow: 'auto'
  },
  modalHeader: {
    padding: '20px',
    borderBottom: '1px solid #e5e7eb',
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'flex-start'
  },
  modalSubtitle: {
    display: 'flex',
    gap: '8px',
    marginTop: '8px'
  },
  modalClose: {
    background: 'none',
    border: 'none',
    fontSize: '32px',
    cursor: 'pointer',
    color: '#6b7280',
    lineHeight: 1
  },
  modalBody: {
    padding: '20px'
  },
  form: {
    padding: '20px'
  },
  formGroup: {
    marginBottom: '16px'
  },
  formRow: {
    display: 'grid',
    gridTemplateColumns: '1fr 1fr',
    gap: '16px'
  },
  label: {
    display: 'block',
    fontSize: '14px',
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
  select: {
    width: '100%',
    padding: '8px 12px',
    border: '1px solid #d1d5db',
    borderRadius: '6px',
    fontSize: '14px',
    outline: 'none',
    boxSizing: 'border-box',
    backgroundColor: '#fff'
  },
  formActions: {
    display: 'flex',
    justifyContent: 'flex-end',
    gap: '12px',
    marginTop: '24px',
    paddingTop: '20px',
    borderTop: '1px solid #e5e7eb'
  },
  cancelButton: {
    padding: '8px 16px',
    backgroundColor: '#fff',
    color: '#374151',
    border: '1px solid #d1d5db',
    borderRadius: '6px',
    fontSize: '14px',
    fontWeight: 500,
    cursor: 'pointer'
  },
  submitButton: {
    padding: '8px 16px',
    backgroundColor: '#3b82f6',
    color: '#fff',
    border: 'none',
    borderRadius: '6px',
    fontSize: '14px',
    fontWeight: 500,
    cursor: 'pointer'
  },
  statusActions: {
    display: 'flex',
    gap: '8px',
    marginBottom: '24px',
    flexWrap: 'wrap'
  },
  statusButton: {
    padding: '6px 12px',
    border: '1px solid #e5e7eb',
    borderRadius: '6px',
    fontSize: '13px',
    fontWeight: 500,
    cursor: 'pointer',
    backgroundColor: '#fff',
    color: '#374151',
    textTransform: 'capitalize'
  },
  detailSection: {
    marginBottom: '24px'
  },
  sectionTitle: {
    fontSize: '16px',
    fontWeight: 600,
    color: '#111827',
    marginBottom: '12px'
  },
  detailGrid: {
    display: 'grid',
    gridTemplateColumns: '1fr 1fr',
    gap: '12px',
    marginTop: '12px'
  },
  detailItem: {
    fontSize: '14px',
    color: '#4b5563'
  },
  timeline: {
    display: 'flex',
    flexDirection: 'column',
    gap: '12px',
    marginBottom: '16px'
  },
  timelineEntry: {
    display: 'flex',
    gap: '12px',
    padding: '12px',
    backgroundColor: '#f9fafb',
    borderRadius: '6px'
  },
  timelineIcon: {
    fontSize: '18px'
  },
  timelineEntryContent: {
    flex: 1
  },
  timelineEntryHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    marginBottom: '4px'
  },
  timelineEntryUser: {
    fontSize: '13px',
    fontWeight: 600,
    color: '#111827'
  },
  timelineEntryTime: {
    fontSize: '12px',
    color: '#6b7280'
  },
  timelineEntryMessage: {
    fontSize: '14px',
    color: '#4b5563'
  },
  addEntry: {
    display: 'flex',
    gap: '8px'
  },
  addButton: {
    padding: '8px 16px',
    backgroundColor: '#3b82f6',
    color: '#fff',
    border: 'none',
    borderRadius: '6px',
    fontSize: '14px',
    fontWeight: 500,
    cursor: 'pointer',
    whiteSpace: 'nowrap'
  },
  resolutionItem: {
    marginBottom: '12px'
  }
};

export default IncidentManager;
