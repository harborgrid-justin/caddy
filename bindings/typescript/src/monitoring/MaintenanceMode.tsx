/**
 * CADDY v0.4.0 - Maintenance Mode Manager
 * Schedule and manage maintenance windows
 * @module monitoring/MaintenanceMode
 */

import React, { useEffect, useState } from 'react';
import { MaintenanceWindow } from './types';

interface MaintenanceModeProps {
  className?: string;
}

export const MaintenanceMode: React.FC<MaintenanceModeProps> = ({
  className = ''
}) => {
  const [windows, setWindows] = useState<MaintenanceWindow[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [formData, setFormData] = useState<Partial<MaintenanceWindow>>({
    title: '',
    description: '',
    services: [],
    startTime: new Date(),
    endTime: new Date(Date.now() + 3600000),
    impactLevel: 'minor',
    notifyUsers: true
  });

  useEffect(() => {
    fetchMaintenanceWindows();
  }, []);

  const fetchMaintenanceWindows = async () => {
    try {
      setLoading(true);
      const response = await fetch('/api/monitoring/maintenance');
      if (!response.ok) throw new Error('Failed to fetch maintenance windows');

      const data = await response.json();
      setWindows(data);
    } catch (error) {
      console.error('[MaintenanceMode] Failed to fetch windows:', error);
    } finally {
      setLoading(false);
    }
  };

  const createWindow = async () => {
    try {
      const response = await fetch('/api/monitoring/maintenance', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          ...formData,
          status: 'scheduled',
          createdBy: 'current-user'
        })
      });

      if (!response.ok) throw new Error('Failed to create maintenance window');

      const newWindow = await response.json();
      setWindows(prev => [...prev, newWindow]);
      setShowCreateModal(false);
      resetForm();
    } catch (error) {
      console.error('[MaintenanceMode] Failed to create window:', error);
      alert('Failed to create maintenance window');
    }
  };

  const updateWindow = async (id: string, updates: Partial<MaintenanceWindow>) => {
    try {
      const response = await fetch(`/api/monitoring/maintenance/${id}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(updates)
      });

      if (!response.ok) throw new Error('Failed to update maintenance window');

      const updatedWindow = await response.json();
      setWindows(prev => prev.map(w => w.id === id ? updatedWindow : w));
    } catch (error) {
      console.error('[MaintenanceMode] Failed to update window:', error);
      alert('Failed to update maintenance window');
    }
  };

  const cancelWindow = async (id: string) => {
    if (!confirm('Are you sure you want to cancel this maintenance window?')) return;

    await updateWindow(id, { status: 'cancelled' });
  };

  const startMaintenance = async (id: string) => {
    if (!confirm('Start this maintenance window now?')) return;

    await updateWindow(id, { status: 'active' });
  };

  const completeMaintenance = async (id: string) => {
    if (!confirm('Mark this maintenance window as completed?')) return;

    await updateWindow(id, { status: 'completed' });
  };

  const resetForm = () => {
    setFormData({
      title: '',
      description: '',
      services: [],
      startTime: new Date(),
      endTime: new Date(Date.now() + 3600000),
      impactLevel: 'minor',
      notifyUsers: true
    });
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!formData.title || !formData.description || !formData.services?.length) {
      alert('Please fill in all required fields');
      return;
    }
    createWindow();
  };

  const getStatusColor = (status: MaintenanceWindow['status']): string => {
    switch (status) {
      case 'scheduled':
        return '#3b82f6';
      case 'active':
        return '#f59e0b';
      case 'completed':
        return '#10b981';
      case 'cancelled':
        return '#6b7280';
      default:
        return '#9ca3af';
    }
  };

  const getImpactColor = (impact: MaintenanceWindow['impactLevel']): string => {
    switch (impact) {
      case 'none':
        return '#10b981';
      case 'minor':
        return '#3b82f6';
      case 'major':
        return '#f59e0b';
      case 'full':
        return '#ef4444';
      default:
        return '#6b7280';
    }
  };

  const formatDuration = (start: Date, end: Date): string => {
    const ms = new Date(end).getTime() - new Date(start).getTime();
    const hours = Math.floor(ms / 3600000);
    const minutes = Math.floor((ms % 3600000) / 60000);

    if (hours > 0) {
      return `${hours}h ${minutes}m`;
    }
    return `${minutes}m`;
  };

  const isUpcoming = (window: MaintenanceWindow): boolean => {
    return window.status === 'scheduled' && new Date(window.startTime) > new Date();
  };

  const isActive = (window: MaintenanceWindow): boolean => {
    return window.status === 'active';
  };

  const upcomingWindows = windows.filter(isUpcoming).sort((a, b) =>
    new Date(a.startTime).getTime() - new Date(b.startTime).getTime()
  );

  const activeWindows = windows.filter(isActive);

  const pastWindows = windows.filter(w => w.status === 'completed' || w.status === 'cancelled').sort((a, b) =>
    new Date(b.startTime).getTime() - new Date(a.startTime).getTime()
  );

  if (loading) {
    return (
      <div style={styles.loading}>
        <div style={styles.spinner} />
        <p>Loading maintenance windows...</p>
      </div>
    );
  }

  return (
    <div className={`maintenance-mode ${className}`} style={styles.container}>
      {/* Header */}
      <div style={styles.header}>
        <div>
          <h2 style={styles.title}>Maintenance Management</h2>
          <p style={styles.subtitle}>Schedule and manage maintenance windows</p>
        </div>
        <button
          style={styles.createButton}
          onClick={() => {
            resetForm();
            setShowCreateModal(true);
          }}
        >
          + Schedule Maintenance
        </button>
      </div>

      {/* Stats */}
      <div style={styles.stats}>
        <div style={styles.statCard}>
          <div style={{ ...styles.statValue, color: '#f59e0b' }}>
            {activeWindows.length}
          </div>
          <div style={styles.statLabel}>Active</div>
        </div>
        <div style={styles.statCard}>
          <div style={{ ...styles.statValue, color: '#3b82f6' }}>
            {upcomingWindows.length}
          </div>
          <div style={styles.statLabel}>Upcoming</div>
        </div>
        <div style={styles.statCard}>
          <div style={styles.statValue}>{windows.length}</div>
          <div style={styles.statLabel}>Total</div>
        </div>
      </div>

      {/* Active Maintenance */}
      {activeWindows.length > 0 && (
        <div style={styles.section}>
          <h3 style={styles.sectionTitle}>🔧 Active Maintenance</h3>
          {activeWindows.map(window => (
            <div key={window.id} style={{ ...styles.windowCard, borderColor: '#f59e0b', borderWidth: '2px' }}>
              <div style={styles.windowHeader}>
                <div>
                  <h4 style={styles.windowTitle}>{window.title}</h4>
                  <p style={styles.windowDescription}>{window.description}</p>
                </div>
                <div style={styles.windowActions}>
                  <button
                    style={styles.actionButton}
                    onClick={() => completeMaintenance(window.id)}
                  >
                    Complete
                  </button>
                </div>
              </div>

              <div style={styles.windowDetails}>
                <div style={styles.detailItem}>
                  <strong>Started:</strong> {new Date(window.startTime).toLocaleString()}
                </div>
                <div style={styles.detailItem}>
                  <strong>Expected End:</strong> {new Date(window.endTime).toLocaleString()}
                </div>
                <div style={styles.detailItem}>
                  <strong>Duration:</strong> {formatDuration(window.startTime, window.endTime)}
                </div>
                <div style={styles.detailItem}>
                  <strong>Impact:</strong>
                  <span
                    style={{
                      ...styles.impactBadge,
                      backgroundColor: `${getImpactColor(window.impactLevel)}20`,
                      color: getImpactColor(window.impactLevel)
                    }}
                  >
                    {window.impactLevel.toUpperCase()}
                  </span>
                </div>
              </div>

              {window.services.length > 0 && (
                <div style={styles.services}>
                  <strong>Affected Services:</strong>
                  <div style={styles.servicesList}>
                    {window.services.map((svc, idx) => (
                      <span key={idx} style={styles.serviceTag}>{svc}</span>
                    ))}
                  </div>
                </div>
              )}
            </div>
          ))}
        </div>
      )}

      {/* Upcoming Maintenance */}
      {upcomingWindows.length > 0 && (
        <div style={styles.section}>
          <h3 style={styles.sectionTitle}>📅 Upcoming Maintenance</h3>
          <div style={styles.windowsList}>
            {upcomingWindows.map(window => (
              <div key={window.id} style={styles.windowCard}>
                <div style={styles.windowHeader}>
                  <div>
                    <h4 style={styles.windowTitle}>{window.title}</h4>
                    <p style={styles.windowDescription}>{window.description}</p>
                  </div>
                  <div style={styles.windowActions}>
                    <button
                      style={styles.actionButton}
                      onClick={() => startMaintenance(window.id)}
                    >
                      Start Now
                    </button>
                    <button
                      style={{ ...styles.actionButton, ...styles.cancelButton }}
                      onClick={() => cancelWindow(window.id)}
                    >
                      Cancel
                    </button>
                  </div>
                </div>

                <div style={styles.windowDetails}>
                  <div style={styles.detailItem}>
                    <strong>Starts:</strong> {new Date(window.startTime).toLocaleString()}
                  </div>
                  <div style={styles.detailItem}>
                    <strong>Ends:</strong> {new Date(window.endTime).toLocaleString()}
                  </div>
                  <div style={styles.detailItem}>
                    <strong>Duration:</strong> {formatDuration(window.startTime, window.endTime)}
                  </div>
                  <div style={styles.detailItem}>
                    <strong>Impact:</strong>
                    <span
                      style={{
                        ...styles.impactBadge,
                        backgroundColor: `${getImpactColor(window.impactLevel)}20`,
                        color: getImpactColor(window.impactLevel)
                      }}
                    >
                      {window.impactLevel.toUpperCase()}
                    </span>
                  </div>
                  <div style={styles.detailItem}>
                    <strong>Notify Users:</strong> {window.notifyUsers ? 'Yes' : 'No'}
                  </div>
                </div>

                {window.services.length > 0 && (
                  <div style={styles.services}>
                    <strong>Affected Services:</strong>
                    <div style={styles.servicesList}>
                      {window.services.map((svc, idx) => (
                        <span key={idx} style={styles.serviceTag}>{svc}</span>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Past Maintenance */}
      {pastWindows.length > 0 && (
        <div style={styles.section}>
          <h3 style={styles.sectionTitle}>📜 Past Maintenance</h3>
          <div style={styles.windowsList}>
            {pastWindows.slice(0, 5).map(window => (
              <div key={window.id} style={styles.windowCard}>
                <div style={styles.windowHeader}>
                  <div>
                    <h4 style={styles.windowTitle}>
                      {window.title}
                      <span
                        style={{
                          ...styles.statusBadge,
                          backgroundColor: `${getStatusColor(window.status)}20`,
                          color: getStatusColor(window.status)
                        }}
                      >
                        {window.status.toUpperCase()}
                      </span>
                    </h4>
                    <p style={styles.windowDescription}>{window.description}</p>
                  </div>
                </div>

                <div style={styles.windowDetails}>
                  <div style={styles.detailItem}>
                    <strong>Date:</strong> {new Date(window.startTime).toLocaleString()}
                  </div>
                  <div style={styles.detailItem}>
                    <strong>Duration:</strong> {formatDuration(window.startTime, window.endTime)}
                  </div>
                  <div style={styles.detailItem}>
                    <strong>Services:</strong> {window.services.join(', ')}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Create Modal */}
      {showCreateModal && (
        <div style={styles.modal} onClick={() => setShowCreateModal(false)}>
          <div style={styles.modalContent} onClick={(e) => e.stopPropagation()}>
            <div style={styles.modalHeader}>
              <h3>Schedule Maintenance Window</h3>
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
                  placeholder="e.g., Database Migration"
                  required
                />
              </div>

              <div style={styles.formGroup}>
                <label style={styles.label}>Description *</label>
                <textarea
                  value={formData.description}
                  onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                  style={{ ...styles.input, minHeight: '80px' }}
                  placeholder="Describe the maintenance work..."
                  required
                />
              </div>

              <div style={styles.formRow}>
                <div style={styles.formGroup}>
                  <label style={styles.label}>Start Time *</label>
                  <input
                    type="datetime-local"
                    value={new Date(formData.startTime!).toISOString().slice(0, 16)}
                    onChange={(e) => setFormData({ ...formData, startTime: new Date(e.target.value) })}
                    style={styles.input}
                    required
                  />
                </div>

                <div style={styles.formGroup}>
                  <label style={styles.label}>End Time *</label>
                  <input
                    type="datetime-local"
                    value={new Date(formData.endTime!).toISOString().slice(0, 16)}
                    onChange={(e) => setFormData({ ...formData, endTime: new Date(e.target.value) })}
                    style={styles.input}
                    required
                  />
                </div>
              </div>

              <div style={styles.formGroup}>
                <label style={styles.label}>Impact Level *</label>
                <select
                  value={formData.impactLevel}
                  onChange={(e) => setFormData({ ...formData, impactLevel: e.target.value as any })}
                  style={styles.select}
                >
                  <option value="none">None - No user impact</option>
                  <option value="minor">Minor - Limited functionality</option>
                  <option value="major">Major - Significant degradation</option>
                  <option value="full">Full - Complete outage</option>
                </select>
              </div>

              <div style={styles.formGroup}>
                <label style={styles.label}>Affected Services *</label>
                <input
                  type="text"
                  value={formData.services?.join(', ')}
                  onChange={(e) => setFormData({
                    ...formData,
                    services: e.target.value.split(',').map(s => s.trim()).filter(Boolean)
                  })}
                  style={styles.input}
                  placeholder="service1, service2, service3"
                  required
                />
              </div>

              <div style={styles.formGroup}>
                <label style={styles.checkbox}>
                  <input
                    type="checkbox"
                    checked={formData.notifyUsers}
                    onChange={(e) => setFormData({ ...formData, notifyUsers: e.target.checked })}
                  />
                  Notify users about this maintenance
                </label>
              </div>

              <div style={styles.formActions}>
                <button
                  type="button"
                  style={styles.cancelFormButton}
                  onClick={() => setShowCreateModal(false)}
                >
                  Cancel
                </button>
                <button type="submit" style={styles.submitButton}>
                  Schedule Maintenance
                </button>
              </div>
            </form>
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
    gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))',
    gap: '16px',
    marginBottom: '32px'
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
  section: {
    marginBottom: '32px'
  },
  sectionTitle: {
    fontSize: '18px',
    fontWeight: 600,
    color: '#111827',
    marginBottom: '16px'
  },
  windowsList: {
    display: 'flex',
    flexDirection: 'column',
    gap: '12px'
  },
  windowCard: {
    backgroundColor: '#fff',
    border: '1px solid #e5e7eb',
    borderRadius: '8px',
    padding: '20px'
  },
  windowHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    marginBottom: '16px'
  },
  windowTitle: {
    fontSize: '16px',
    fontWeight: 600,
    color: '#111827',
    margin: 0,
    marginBottom: '4px',
    display: 'flex',
    alignItems: 'center',
    gap: '8px'
  },
  windowDescription: {
    fontSize: '14px',
    color: '#6b7280',
    margin: 0
  },
  windowActions: {
    display: 'flex',
    gap: '8px'
  },
  actionButton: {
    padding: '6px 12px',
    backgroundColor: '#3b82f6',
    color: '#fff',
    border: 'none',
    borderRadius: '6px',
    fontSize: '13px',
    fontWeight: 500,
    cursor: 'pointer'
  },
  cancelButton: {
    backgroundColor: '#fff',
    color: '#ef4444',
    border: '1px solid #ef4444'
  },
  windowDetails: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
    gap: '12px',
    marginBottom: '12px'
  },
  detailItem: {
    fontSize: '13px',
    color: '#4b5563',
    display: 'flex',
    alignItems: 'center',
    gap: '8px'
  },
  statusBadge: {
    fontSize: '10px',
    fontWeight: 600,
    padding: '3px 8px',
    borderRadius: '10px',
    marginLeft: '8px'
  },
  impactBadge: {
    fontSize: '11px',
    fontWeight: 600,
    padding: '3px 8px',
    borderRadius: '10px'
  },
  services: {
    paddingTop: '12px',
    borderTop: '1px solid #e5e7eb'
  },
  servicesList: {
    display: 'flex',
    flexWrap: 'wrap',
    gap: '6px',
    marginTop: '8px'
  },
  serviceTag: {
    fontSize: '12px',
    padding: '4px 8px',
    backgroundColor: '#f3f4f6',
    borderRadius: '4px',
    color: '#374151'
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
  checkbox: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    fontSize: '14px',
    color: '#374151',
    cursor: 'pointer'
  },
  formActions: {
    display: 'flex',
    justifyContent: 'flex-end',
    gap: '12px',
    marginTop: '24px',
    paddingTop: '20px',
    borderTop: '1px solid #e5e7eb'
  },
  cancelFormButton: {
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
  }
};

export default MaintenanceMode;
