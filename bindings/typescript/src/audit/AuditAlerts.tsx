/**
 * Audit Alerts Component
 * Alert configuration for suspicious activity and security events
 */

import React, { useState, useEffect } from 'react';
import type { AuditAlert, AuditEventType, AuditSeverity } from './types';

interface AuditAlertsProps {
  organizationId?: string;
}

export const AuditAlerts: React.FC<AuditAlertsProps> = ({ organizationId }) => {
  const [alerts, setAlerts] = useState<AuditAlert[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [editingAlert, setEditingAlert] = useState<AuditAlert | null>(null);

  useEffect(() => {
    loadAlerts();
  }, [organizationId]);

  const loadAlerts = async () => {
    setLoading(true);
    try {
      const params = new URLSearchParams(
        organizationId ? { organization_id: organizationId } : {}
      );
      const response = await fetch(`/api/audit/alerts?${params}`);
      const data = await response.json();
      setAlerts(data.alerts || []);
    } catch (error) {
      console.error('Failed to load alerts:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleToggleAlert = async (alert: AuditAlert) => {
    try {
      const response = await fetch(`/api/audit/alerts/${alert.id}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ enabled: !alert.enabled }),
      });
      const updated = await response.json();
      setAlerts((prev) => prev.map((a) => (a.id === alert.id ? updated : a)));
    } catch (error) {
      console.error('Failed to toggle alert:', error);
    }
  };

  const handleDeleteAlert = async (alertId: string) => {
    if (!confirm('Are you sure you want to delete this alert?')) return;

    try {
      await fetch(`/api/audit/alerts/${alertId}`, {
        method: 'DELETE',
      });
      setAlerts((prev) => prev.filter((a) => a.id !== alertId));
    } catch (error) {
      console.error('Failed to delete alert:', error);
    }
  };

  const handleEditAlert = (alert: AuditAlert) => {
    setEditingAlert(alert);
    setShowCreateModal(true);
  };

  if (loading) {
    return (
      <div className="audit-alerts loading">
        <div className="loading-spinner" />
        <p>Loading audit alerts...</p>
      </div>
    );
  }

  return (
    <div className="audit-alerts">
      {/* Header */}
      <div className="alerts-header">
        <div>
          <h2>Audit Alerts</h2>
          <p className="subtitle">
            Configure alerts for suspicious activity and security events
          </p>
        </div>
        <button
          className="btn btn-primary"
          onClick={() => {
            setEditingAlert(null);
            setShowCreateModal(true);
          }}
        >
          Create Alert
        </button>
      </div>

      {/* Alert Templates */}
      <div className="alert-templates">
        <h3>Quick Templates</h3>
        <div className="templates-grid">
          <AlertTemplate
            title="Failed Login Attempts"
            description="Alert on multiple failed login attempts from same IP"
            icon="🔒"
            onClick={() => {
              /* Create from template */
            }}
          />
          <AlertTemplate
            title="Data Exfiltration"
            description="Alert on unusual data export patterns"
            icon="📤"
            onClick={() => {
              /* Create from template */
            }}
          />
          <AlertTemplate
            title="Privilege Escalation"
            description="Alert on role or permission changes"
            icon="⬆️"
            onClick={() => {
              /* Create from template */
            }}
          />
          <AlertTemplate
            title="Anomalous Behavior"
            description="Alert on detected anomalies with high confidence"
            icon="⚠️"
            onClick={() => {
              /* Create from template */
            }}
          />
        </div>
      </div>

      {/* Active Alerts */}
      <div className="active-alerts">
        <h3>Configured Alerts</h3>
        {alerts.length === 0 ? (
          <div className="empty-state">
            <h4>No alerts configured</h4>
            <p>Create your first alert to get notified of security events</p>
            <button
              className="btn btn-primary"
              onClick={() => setShowCreateModal(true)}
            >
              Create Alert
            </button>
          </div>
        ) : (
          <div className="alerts-list">
            {alerts.map((alert) => (
              <AlertCard
                key={alert.id}
                alert={alert}
                onToggle={() => handleToggleAlert(alert)}
                onEdit={() => handleEditAlert(alert)}
                onDelete={() => handleDeleteAlert(alert.id)}
              />
            ))}
          </div>
        )}
      </div>

      {/* Create/Edit Modal */}
      {showCreateModal && (
        <AlertModal
          alert={editingAlert}
          onClose={() => {
            setShowCreateModal(false);
            setEditingAlert(null);
          }}
          onSave={(savedAlert) => {
            if (editingAlert) {
              setAlerts((prev) =>
                prev.map((a) => (a.id === savedAlert.id ? savedAlert : a))
              );
            } else {
              setAlerts((prev) => [savedAlert, ...prev]);
            }
            setShowCreateModal(false);
            setEditingAlert(null);
          }}
        />
      )}
    </div>
  );
};

// Alert Template Component
function AlertTemplate({
  title,
  description,
  icon,
  onClick,
}: {
  title: string;
  description: string;
  icon: string;
  onClick: () => void;
}) {
  return (
    <button className="alert-template" onClick={onClick}>
      <div className="template-icon">{icon}</div>
      <div className="template-content">
        <div className="template-title">{title}</div>
        <div className="template-description">{description}</div>
      </div>
    </button>
  );
}

// Alert Card Component
function AlertCard({
  alert,
  onToggle,
  onEdit,
  onDelete,
}: {
  alert: AuditAlert;
  onToggle: () => void;
  onEdit: () => void;
  onDelete: () => void;
}) {
  return (
    <div className={`alert-card ${alert.enabled ? 'enabled' : 'disabled'}`}>
      <div className="alert-header">
        <div className="alert-title-section">
          <h4>{alert.name}</h4>
          <p className="alert-description">{alert.description}</p>
        </div>
        <div className="alert-toggle">
          <label className="toggle-switch">
            <input type="checkbox" checked={alert.enabled} onChange={onToggle} />
            <span className="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div className="alert-conditions">
        <h5>Trigger Conditions</h5>
        <div className="conditions-grid">
          {alert.conditions.event_types && (
            <div className="condition-item">
              <label>Event Types:</label>
              <div className="condition-values">
                {alert.conditions.event_types.map((type) => (
                  <span key={type} className="condition-tag">
                    {type}
                  </span>
                ))}
              </div>
            </div>
          )}
          {alert.conditions.severities && (
            <div className="condition-item">
              <label>Severities:</label>
              <div className="condition-values">
                {alert.conditions.severities.map((severity) => (
                  <span key={severity} className={`severity-tag severity-${severity}`}>
                    {severity.toUpperCase()}
                  </span>
                ))}
              </div>
            </div>
          )}
          {alert.conditions.min_risk_score !== undefined && (
            <div className="condition-item">
              <label>Minimum Risk Score:</label>
              <span className="condition-value">
                {alert.conditions.min_risk_score}
              </span>
            </div>
          )}
          {alert.conditions.threshold && (
            <div className="condition-item">
              <label>Threshold:</label>
              <span className="condition-value">
                {alert.conditions.threshold.count} events in{' '}
                {alert.conditions.threshold.window_seconds}s
              </span>
            </div>
          )}
        </div>
      </div>

      <div className="alert-notifications">
        <h5>Notifications</h5>
        <div className="notification-channels">
          {alert.notification_channels.map((channel) => (
            <span key={channel} className="channel-badge">
              {channel === 'email' && '📧'}
              {channel === 'slack' && '💬'}
              {channel === 'webhook' && '🔗'}
              {channel === 'sms' && '📱'}
              {channel}
            </span>
          ))}
        </div>
        <div className="notification-recipients">
          {alert.notification_recipients.join(', ')}
        </div>
      </div>

      <div className="alert-stats">
        <div className="stat-item">
          <label>Cooldown:</label>
          <span>{alert.cooldown_minutes} minutes</span>
        </div>
        <div className="stat-item">
          <label>Triggered:</label>
          <span>{alert.trigger_count} times</span>
        </div>
        {alert.last_triggered && (
          <div className="stat-item">
            <label>Last Triggered:</label>
            <span>{new Date(alert.last_triggered).toLocaleString()}</span>
          </div>
        )}
      </div>

      <div className="alert-actions">
        <button className="btn btn-sm btn-secondary" onClick={onEdit}>
          Edit
        </button>
        <button className="btn btn-sm btn-danger" onClick={onDelete}>
          Delete
        </button>
      </div>
    </div>
  );
}

// Alert Modal Component
function AlertModal({
  alert,
  onClose,
  onSave,
}: {
  alert: AuditAlert | null;
  onClose: () => void;
  onSave: (alert: AuditAlert) => void;
}) {
  const [name, setName] = useState(alert?.name || '');
  const [description, setDescription] = useState(alert?.description || '');
  const [eventTypes, setEventTypes] = useState<AuditEventType[]>(
    alert?.conditions.event_types || []
  );
  const [severities, setSeverities] = useState<AuditSeverity[]>(
    alert?.conditions.severities || []
  );
  const [minRiskScore, setMinRiskScore] = useState(
    alert?.conditions.min_risk_score || 70
  );
  const [thresholdCount, setThresholdCount] = useState(
    alert?.conditions.threshold?.count || 5
  );
  const [thresholdWindow, setThresholdWindow] = useState(
    alert?.conditions.threshold?.window_seconds || 300
  );
  const [channels, setChannels] = useState<('email' | 'slack' | 'webhook' | 'sms')[]>(
    alert?.notification_channels || ['email']
  );
  const [recipients, setRecipients] = useState(
    alert?.notification_recipients.join(', ') || ''
  );
  const [cooldown, setCooldown] = useState(alert?.cooldown_minutes || 60);
  const [errors, setErrors] = useState<Record<string, string>>({});

  const validate = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (!name.trim()) {
      newErrors.name = 'Alert name is required';
    }
    if (!recipients.trim()) {
      newErrors.recipients = 'At least one recipient is required';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSave = async () => {
    if (!validate()) return;

    const alertData: AuditAlert = {
      id: alert?.id || crypto.randomUUID(),
      name,
      description,
      enabled: alert?.enabled ?? true,
      conditions: {
        event_types: eventTypes.length > 0 ? eventTypes : undefined,
        severities: severities.length > 0 ? severities : undefined,
        min_risk_score: minRiskScore,
        threshold: {
          count: thresholdCount,
          window_seconds: thresholdWindow,
        },
      },
      notification_channels: channels,
      notification_recipients: recipients.split(',').map((r) => r.trim()),
      cooldown_minutes: cooldown,
      created_by: alert?.created_by || 'current_user',
      created_at: alert?.created_at || new Date().toISOString(),
      updated_at: new Date().toISOString(),
      last_triggered: alert?.last_triggered,
      trigger_count: alert?.trigger_count || 0,
    };

    try {
      const response = await fetch(
        alert ? `/api/audit/alerts/${alert.id}` : '/api/audit/alerts',
        {
          method: alert ? 'PUT' : 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(alertData),
        }
      );

      if (response.ok) {
        const savedAlert = await response.json();
        onSave(savedAlert);
      }
    } catch (error) {
      console.error('Failed to save alert:', error);
    }
  };

  const toggleSeverity = (severity: AuditSeverity) => {
    setSeverities((prev) =>
      prev.includes(severity)
        ? prev.filter((s) => s !== severity)
        : [...prev, severity]
    );
  };

  const toggleChannel = (channel: 'email' | 'slack' | 'webhook' | 'sms') => {
    setChannels((prev) =>
      prev.includes(channel) ? prev.filter((c) => c !== channel) : [...prev, channel]
    );
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal alert-modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>{alert ? 'Edit Alert' : 'Create Alert'}</h2>
          <button className="modal-close" onClick={onClose}>
            ×
          </button>
        </div>

        <div className="modal-content">
          <div className="form-section">
            <label>Alert Name *</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g., Suspicious Login Activity"
              className={errors.name ? 'error' : ''}
            />
            {errors.name && <span className="error-message">{errors.name}</span>}
          </div>

          <div className="form-section">
            <label>Description</label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Describe what this alert monitors"
              rows={3}
            />
          </div>

          <div className="form-section">
            <label>Severity Levels</label>
            <div className="severity-filters">
              {(['low', 'medium', 'high', 'critical'] as AuditSeverity[]).map(
                (severity) => (
                  <button
                    key={severity}
                    className={`severity-chip severity-${severity} ${
                      severities.includes(severity) ? 'active' : ''
                    }`}
                    onClick={() => toggleSeverity(severity)}
                  >
                    {severity.toUpperCase()}
                  </button>
                )
              )}
            </div>
          </div>

          <div className="form-section">
            <label>Minimum Risk Score: {minRiskScore}</label>
            <input
              type="range"
              min="0"
              max="100"
              step="10"
              value={minRiskScore}
              onChange={(e) => setMinRiskScore(parseInt(e.target.value))}
              className="risk-slider"
            />
          </div>

          <div className="form-section">
            <label>Threshold</label>
            <div className="form-row">
              <input
                type="number"
                value={thresholdCount}
                onChange={(e) => setThresholdCount(parseInt(e.target.value))}
                placeholder="Count"
                min="1"
              />
              <span>events in</span>
              <input
                type="number"
                value={thresholdWindow}
                onChange={(e) => setThresholdWindow(parseInt(e.target.value))}
                placeholder="Seconds"
                min="1"
              />
              <span>seconds</span>
            </div>
          </div>

          <div className="form-section">
            <label>Notification Channels</label>
            <div className="channel-options">
              {(['email', 'slack', 'webhook', 'sms'] as const).map((channel) => (
                <button
                  key={channel}
                  className={`channel-chip ${channels.includes(channel) ? 'active' : ''}`}
                  onClick={() => toggleChannel(channel)}
                >
                  {channel === 'email' && '📧 '}
                  {channel === 'slack' && '💬 '}
                  {channel === 'webhook' && '🔗 '}
                  {channel === 'sms' && '📱 '}
                  {channel}
                </button>
              ))}
            </div>
          </div>

          <div className="form-section">
            <label>Recipients (comma-separated) *</label>
            <input
              type="text"
              value={recipients}
              onChange={(e) => setRecipients(e.target.value)}
              placeholder="user@example.com, admin@example.com"
              className={errors.recipients ? 'error' : ''}
            />
            {errors.recipients && (
              <span className="error-message">{errors.recipients}</span>
            )}
          </div>

          <div className="form-section">
            <label>Cooldown Period (minutes)</label>
            <input
              type="number"
              value={cooldown}
              onChange={(e) => setCooldown(parseInt(e.target.value))}
              min="1"
              placeholder="60"
            />
            <small>Minimum time between alert notifications</small>
          </div>
        </div>

        <div className="modal-footer">
          <button className="btn btn-secondary" onClick={onClose}>
            Cancel
          </button>
          <button className="btn btn-primary" onClick={handleSave}>
            {alert ? 'Save Changes' : 'Create Alert'}
          </button>
        </div>
      </div>
    </div>
  );
}
