/**
 * Export Panel
 *
 * Data export configuration and management for analytics.
 */

import React, { useState, useEffect } from 'react';
import { useAnalytics } from './useAnalytics';
import type { ExportFormat, ExportEndpoint, TimeRange } from './types';

export function ExportPanel() {
  const { config, updateConfig } = useAnalytics();
  const [endpoints, setEndpoints] = useState<ExportEndpoint[]>([]);
  const [editingEndpoint, setEditingEndpoint] = useState<ExportEndpoint | null>(null);
  const [showAddModal, setShowAddModal] = useState(false);
  const [exportHistory, setExportHistory] = useState<any[]>([]);

  // Load endpoints from config
  useEffect(() => {
    if (config?.export_endpoints) {
      setEndpoints(config.export_endpoints);
    }
  }, [config]);

  // Handle endpoint creation/update
  const handleSaveEndpoint = async (endpoint: ExportEndpoint) => {
    const updatedEndpoints = editingEndpoint
      ? endpoints.map((e) => (e.name === editingEndpoint.name ? endpoint : e))
      : [...endpoints, endpoint];

    setEndpoints(updatedEndpoints);

    try {
      await updateConfig({ export_endpoints: updatedEndpoints });
      setEditingEndpoint(null);
      setShowAddModal(false);
    } catch (err) {
      console.error('Failed to update export endpoints:', err);
    }
  };

  // Handle endpoint deletion
  const handleDeleteEndpoint = async (name: string) => {
    const updatedEndpoints = endpoints.filter((e) => e.name !== name);
    setEndpoints(updatedEndpoints);

    try {
      await updateConfig({ export_endpoints: updatedEndpoints });
    } catch (err) {
      console.error('Failed to delete export endpoint:', err);
    }
  };

  // Trigger manual export
  const handleManualExport = async (endpoint: ExportEndpoint) => {
    try {
      const response = await fetch('/api/analytics/export', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ endpoint }),
      });

      if (response.ok) {
        alert('Export successful!');
        // Refresh export history
        fetchExportHistory();
      } else {
        alert('Export failed');
      }
    } catch (err) {
      console.error('Export failed:', err);
    }
  };

  // Fetch export history
  const fetchExportHistory = async () => {
    try {
      const response = await fetch('/api/analytics/export/history');
      if (response.ok) {
        const history = await response.json();
        setExportHistory(history);
      }
    } catch (err) {
      console.error('Failed to fetch export history:', err);
    }
  };

  useEffect(() => {
    fetchExportHistory();
  }, []);

  return (
    <div className="export-panel">
      {/* Header */}
      <div className="panel-header">
        <h2>Data Export</h2>
        <p>Configure automatic exports to external systems and services</p>
      </div>

      {/* Quick Export Section */}
      <div className="quick-export">
        <h3>Quick Export</h3>
        <p>Export current analytics data in various formats</p>
        <div className="export-buttons">
          <QuickExportButton format={ExportFormat.Json} label="JSON" />
          <QuickExportButton format={ExportFormat.Csv} label="CSV" />
          <QuickExportButton format={ExportFormat.Prometheus} label="Prometheus" />
          <QuickExportButton format={ExportFormat.OpenTelemetry} label="OTLP" />
        </div>
      </div>

      {/* Configured Endpoints */}
      <div className="configured-endpoints">
        <div className="section-header">
          <h3>Export Endpoints</h3>
          <button
            onClick={() => setShowAddModal(true)}
            className="btn btn-primary"
          >
            + Add Endpoint
          </button>
        </div>

        {endpoints.length === 0 ? (
          <div className="empty-state">
            <p>No export endpoints configured</p>
            <p className="hint">
              Add an endpoint to automatically export metrics to external services
            </p>
          </div>
        ) : (
          <div className="endpoints-list">
            {endpoints.map((endpoint) => (
              <EndpointCard
                key={endpoint.name}
                endpoint={endpoint}
                onEdit={() => {
                  setEditingEndpoint(endpoint);
                  setShowAddModal(true);
                }}
                onDelete={() => handleDeleteEndpoint(endpoint.name)}
                onExport={() => handleManualExport(endpoint)}
              />
            ))}
          </div>
        )}
      </div>

      {/* Export History */}
      <div className="export-history">
        <h3>Export History</h3>
        {exportHistory.length === 0 ? (
          <p>No export history available</p>
        ) : (
          <table className="history-table">
            <thead>
              <tr>
                <th>Timestamp</th>
                <th>Endpoint</th>
                <th>Format</th>
                <th>Status</th>
                <th>Records</th>
              </tr>
            </thead>
            <tbody>
              {exportHistory.map((item, index) => (
                <tr key={index}>
                  <td>{new Date(item.timestamp).toLocaleString()}</td>
                  <td>{item.endpoint}</td>
                  <td>{item.format}</td>
                  <td>
                    <span className={`status-badge ${item.status}`}>
                      {item.status}
                    </span>
                  </td>
                  <td>{item.records.toLocaleString()}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {/* Add/Edit Endpoint Modal */}
      {showAddModal && (
        <EndpointModal
          endpoint={editingEndpoint}
          onSave={handleSaveEndpoint}
          onClose={() => {
            setShowAddModal(false);
            setEditingEndpoint(null);
          }}
        />
      )}
    </div>
  );
}

// Quick Export Button Component
function QuickExportButton({ format, label }: { format: ExportFormat; label: string }) {
  const [exporting, setExporting] = useState(false);

  const handleExport = async () => {
    setExporting(true);
    try {
      const timeRange: TimeRange = {
        start: new Date(Date.now() - 24 * 60 * 60 * 1000),
        end: new Date(),
      };

      const response = await fetch('/api/analytics/export/quick', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          format,
          timeRange,
        }),
      });

      if (response.ok) {
        const blob = await response.blob();
        const url = window.URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `analytics_export_${Date.now()}.${getFormatExtension(format)}`;
        document.body.appendChild(a);
        a.click();
        window.URL.revokeObjectURL(url);
        document.body.removeChild(a);
      } else {
        alert('Export failed');
      }
    } catch (err) {
      console.error('Export failed:', err);
    } finally {
      setExporting(false);
    }
  };

  return (
    <button
      onClick={handleExport}
      disabled={exporting}
      className="quick-export-btn"
    >
      {exporting ? 'Exporting...' : `Export ${label}`}
    </button>
  );
}

// Endpoint Card Component
function EndpointCard({
  endpoint,
  onEdit,
  onDelete,
  onExport,
}: {
  endpoint: ExportEndpoint;
  onEdit: () => void;
  onDelete: () => void;
  onExport: () => void;
}) {
  const formatIcons: Record<ExportFormat, string> = {
    [ExportFormat.Prometheus]: '📊',
    [ExportFormat.OpenTelemetry]: '🔭',
    [ExportFormat.Json]: '📄',
    [ExportFormat.Csv]: '📑',
    [ExportFormat.Binary]: '💾',
  };

  return (
    <div className="endpoint-card">
      <div className="endpoint-icon">{formatIcons[endpoint.format]}</div>
      <div className="endpoint-info">
        <h4>{endpoint.name}</h4>
        <p className="endpoint-url">{endpoint.url}</p>
        <div className="endpoint-meta">
          <span className="format-badge">{endpoint.format}</span>
          <span className="interval">Every {endpoint.interval_secs}s</span>
        </div>
      </div>
      <div className="endpoint-actions">
        <button onClick={onExport} className="btn-icon" title="Export Now">
          🚀
        </button>
        <button onClick={onEdit} className="btn-icon" title="Edit">
          ✏️
        </button>
        <button onClick={onDelete} className="btn-icon danger" title="Delete">
          🗑️
        </button>
      </div>
    </div>
  );
}

// Endpoint Modal Component
function EndpointModal({
  endpoint,
  onSave,
  onClose,
}: {
  endpoint: ExportEndpoint | null;
  onSave: (endpoint: ExportEndpoint) => void;
  onClose: () => void;
}) {
  const [formData, setFormData] = useState<ExportEndpoint>(
    endpoint || {
      name: '',
      url: '',
      format: ExportFormat.Json,
      interval_secs: 300,
      auth_token: undefined,
    }
  );

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSave(formData);
  };

  return (
    <div className="endpoint-modal">
      <div className="modal-overlay" onClick={onClose} />
      <div className="modal-content">
        <div className="modal-header">
          <h3>{endpoint ? 'Edit Endpoint' : 'Add Export Endpoint'}</h3>
          <button onClick={onClose} className="close-btn">
            ✕
          </button>
        </div>

        <form onSubmit={handleSubmit} className="modal-body">
          <div className="form-group">
            <label>Endpoint Name *</label>
            <input
              type="text"
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              placeholder="e.g., Production Prometheus"
              required
            />
          </div>

          <div className="form-group">
            <label>Endpoint URL *</label>
            <input
              type="url"
              value={formData.url}
              onChange={(e) => setFormData({ ...formData, url: e.target.value })}
              placeholder="https://prometheus.example.com/api/v1/push"
              required
            />
          </div>

          <div className="form-group">
            <label>Export Format *</label>
            <select
              value={formData.format}
              onChange={(e) =>
                setFormData({ ...formData, format: e.target.value as ExportFormat })
              }
            >
              <option value={ExportFormat.Prometheus}>Prometheus</option>
              <option value={ExportFormat.OpenTelemetry}>OpenTelemetry (OTLP)</option>
              <option value={ExportFormat.Json}>JSON</option>
              <option value={ExportFormat.Csv}>CSV</option>
              <option value={ExportFormat.Binary}>Binary</option>
            </select>
          </div>

          <div className="form-group">
            <label>Export Interval (seconds) *</label>
            <input
              type="number"
              value={formData.interval_secs}
              onChange={(e) =>
                setFormData({ ...formData, interval_secs: parseInt(e.target.value) })
              }
              min="1"
              required
            />
          </div>

          <div className="form-group">
            <label>Authentication Token (Optional)</label>
            <input
              type="password"
              value={formData.auth_token || ''}
              onChange={(e) =>
                setFormData({ ...formData, auth_token: e.target.value || undefined })
              }
              placeholder="Bearer token for authentication"
            />
          </div>

          <div className="modal-actions">
            <button type="button" onClick={onClose} className="btn btn-secondary">
              Cancel
            </button>
            <button type="submit" className="btn btn-primary">
              {endpoint ? 'Update' : 'Add'} Endpoint
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

// Helper function
function getFormatExtension(format: ExportFormat): string {
  const extensions: Record<ExportFormat, string> = {
    [ExportFormat.Prometheus]: 'prom',
    [ExportFormat.OpenTelemetry]: 'otlp',
    [ExportFormat.Json]: 'json',
    [ExportFormat.Csv]: 'csv',
    [ExportFormat.Binary]: 'bin',
  };
  return extensions[format];
}

/**
 * Export Templates Component
 */
export function ExportTemplates() {
  const templates = [
    {
      id: 'prometheus-pushgateway',
      name: 'Prometheus Pushgateway',
      format: ExportFormat.Prometheus,
      urlTemplate: 'http://localhost:9091/metrics/job/{job_name}',
    },
    {
      id: 'otlp-collector',
      name: 'OpenTelemetry Collector',
      format: ExportFormat.OpenTelemetry,
      urlTemplate: 'http://localhost:4318/v1/metrics',
    },
    {
      id: 's3-bucket',
      name: 'AWS S3 Bucket',
      format: ExportFormat.Json,
      urlTemplate: 'https://s3.amazonaws.com/{bucket}/analytics/',
    },
  ];

  return (
    <div className="export-templates">
      <h3>Export Templates</h3>
      <p>Pre-configured templates for common export destinations</p>
      <div className="templates-grid">
        {templates.map((template) => (
          <div key={template.id} className="template-card">
            <h4>{template.name}</h4>
            <p>{template.format}</p>
            <code>{template.urlTemplate}</code>
            <button className="btn btn-sm">Use Template</button>
          </div>
        ))}
      </div>
    </div>
  );
}
