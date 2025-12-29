/**
 * CADDY v0.4.0 - Cloud Storage Integration
 * Support for S3, Google Cloud Storage, Azure Blob, and more
 */

import React, { useState, useEffect, useCallback } from 'react';
import {
  CloudConfig,
  CloudProvider,
  CloudCredentials,
  CloudSettings,
  SyncRule,
  FileItem,
  CloudSyncInfo,
} from './types';

interface FileCloudProps {
  tenantId: string;
  onClose?: () => void;
  className?: string;
}

export const FileCloud: React.FC<FileCloudProps> = ({
  tenantId,
  onClose,
  className = '',
}) => {
  const [configs, setConfigs] = useState<CloudConfig[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'providers' | 'sync' | 'activity'>('providers');
  const [showAddProvider, setShowAddProvider] = useState(false);
  const [editingConfig, setEditingConfig] = useState<CloudConfig | null>(null);
  const [syncActivity, setSyncActivity] = useState<Array<{
    id: string;
    provider: CloudProvider;
    action: string;
    fileName: string;
    status: 'success' | 'error' | 'pending';
    timestamp: Date;
    error?: string;
  }>>([]);

  useEffect(() => {
    loadConfigs();
    loadSyncActivity();
  }, [tenantId]);

  const loadConfigs = async () => {
    setLoading(true);
    setError(null);

    try {
      const response = await fetch(`/api/v1/tenants/${tenantId}/files/cloud/configs`, {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
        },
      });

      if (!response.ok) {
        throw new Error('Failed to load cloud configurations');
      }

      const data = await response.json();
      setConfigs(data.configs);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load configurations');
    } finally {
      setLoading(false);
    }
  };

  const loadSyncActivity = async () => {
    try {
      const response = await fetch(`/api/v1/tenants/${tenantId}/files/cloud/activity`, {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
        },
      });

      if (response.ok) {
        const data = await response.json();
        setSyncActivity(data.activity);
      }
    } catch (err) {
      console.error('Failed to load sync activity:', err);
    }
  };

  const saveConfig = async (config: Partial<CloudConfig>) => {
    try {
      const response = await fetch(`/api/v1/tenants/${tenantId}/files/cloud/configs`, {
        method: config.provider ? 'PUT' : 'POST',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(config),
      });

      if (!response.ok) {
        throw new Error('Failed to save configuration');
      }

      await loadConfigs();
      setShowAddProvider(false);
      setEditingConfig(null);
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to save configuration');
    }
  };

  const deleteConfig = async (provider: CloudProvider) => {
    if (!confirm(`Delete ${provider} configuration? This will stop all syncing with this provider.`)) {
      return;
    }

    try {
      const response = await fetch(
        `/api/v1/tenants/${tenantId}/files/cloud/configs/${provider}`,
        {
          method: 'DELETE',
          headers: {
            'Authorization': `Bearer ${localStorage.getItem('token')}`,
          },
        }
      );

      if (!response.ok) {
        throw new Error('Failed to delete configuration');
      }

      await loadConfigs();
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to delete configuration');
    }
  };

  const testConnection = async (config: CloudConfig) => {
    try {
      const response = await fetch(
        `/api/v1/tenants/${tenantId}/files/cloud/test`,
        {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${localStorage.getItem('token')}`,
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ provider: config.provider }),
        }
      );

      if (!response.ok) {
        throw new Error('Connection test failed');
      }

      alert('Connection successful!');
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Connection test failed');
    }
  };

  const triggerSync = async (provider: CloudProvider) => {
    try {
      const response = await fetch(
        `/api/v1/tenants/${tenantId}/files/cloud/sync`,
        {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${localStorage.getItem('token')}`,
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ provider }),
        }
      );

      if (!response.ok) {
        throw new Error('Failed to trigger sync');
      }

      alert('Sync started');
      await loadSyncActivity();
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to trigger sync');
    }
  };

  const getProviderIcon = (provider: CloudProvider): string => {
    const icons: Record<CloudProvider, string> = {
      s3: '☁️',
      gcs: '☁️',
      azure: '☁️',
      dropbox: '📦',
      onedrive: '📂',
    };
    return icons[provider];
  };

  const getProviderName = (provider: CloudProvider): string => {
    const names: Record<CloudProvider, string> = {
      s3: 'Amazon S3',
      gcs: 'Google Cloud Storage',
      azure: 'Azure Blob Storage',
      dropbox: 'Dropbox',
      onedrive: 'OneDrive',
    };
    return names[provider];
  };

  return (
    <div className={`file-cloud ${className}`}>
      {/* Header */}
      <div className="cloud-header">
        <h2>Cloud Storage</h2>
        {onClose && (
          <button onClick={onClose} className="btn-close">
            ✕
          </button>
        )}
      </div>

      {/* Tabs */}
      <div className="cloud-tabs">
        <button
          className={`tab ${activeTab === 'providers' ? 'active' : ''}`}
          onClick={() => setActiveTab('providers')}
        >
          Providers
        </button>
        <button
          className={`tab ${activeTab === 'sync' ? 'active' : ''}`}
          onClick={() => setActiveTab('sync')}
        >
          Sync Rules
        </button>
        <button
          className={`tab ${activeTab === 'activity' ? 'active' : ''}`}
          onClick={() => setActiveTab('activity')}
        >
          Activity
        </button>
      </div>

      {/* Content */}
      <div className="cloud-content">
        {loading ? (
          <div className="loading-state">Loading cloud configurations...</div>
        ) : error ? (
          <div className="error-state">{error}</div>
        ) : (
          <>
            {/* Providers Tab */}
            {activeTab === 'providers' && (
              <div className="providers-tab">
                <div className="tab-header">
                  <button
                    onClick={() => setShowAddProvider(true)}
                    className="btn btn-primary"
                  >
                    Add Cloud Provider
                  </button>
                </div>

                {configs.length === 0 ? (
                  <div className="empty-state">
                    No cloud providers configured. Add one to enable cloud sync.
                  </div>
                ) : (
                  <div className="providers-list">
                    {configs.map(config => (
                      <div key={config.provider} className="provider-card">
                        <div className="provider-header">
                          <div className="provider-title">
                            <span className="provider-icon">
                              {getProviderIcon(config.provider)}
                            </span>
                            <span className="provider-name">
                              {getProviderName(config.provider)}
                            </span>
                            <span className={`status-badge ${config.enabled ? 'active' : 'inactive'}`}>
                              {config.enabled ? 'Active' : 'Inactive'}
                            </span>
                          </div>
                          <div className="provider-actions">
                            <button
                              onClick={() => testConnection(config)}
                              className="btn btn-sm"
                            >
                              Test
                            </button>
                            <button
                              onClick={() => triggerSync(config.provider)}
                              disabled={!config.enabled}
                              className="btn btn-sm btn-primary"
                            >
                              Sync Now
                            </button>
                            <button
                              onClick={() => setEditingConfig(config)}
                              className="btn btn-sm"
                            >
                              Edit
                            </button>
                            <button
                              onClick={() => deleteConfig(config.provider)}
                              className="btn btn-sm btn-danger"
                            >
                              Delete
                            </button>
                          </div>
                        </div>

                        <div className="provider-details">
                          <div className="detail-item">
                            <strong>Auto Sync:</strong>{' '}
                            {config.settings.autoSync ? 'Enabled' : 'Disabled'}
                          </div>
                          {config.settings.autoSync && (
                            <div className="detail-item">
                              <strong>Sync Interval:</strong>{' '}
                              {config.settings.syncInterval} minutes
                            </div>
                          )}
                          <div className="detail-item">
                            <strong>Conflict Resolution:</strong>{' '}
                            {config.settings.conflictResolution}
                          </div>
                          <div className="detail-item">
                            <strong>Encryption:</strong>{' '}
                            {config.settings.encryption ? 'Enabled' : 'Disabled'}
                          </div>
                          <div className="detail-item">
                            <strong>Sync Rules:</strong>{' '}
                            {config.syncRules.filter(r => r.enabled).length} active
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                )}

                {/* Add/Edit Provider Dialog */}
                {(showAddProvider || editingConfig) && (
                  <CloudProviderDialog
                    config={editingConfig}
                    onSave={saveConfig}
                    onCancel={() => {
                      setShowAddProvider(false);
                      setEditingConfig(null);
                    }}
                  />
                )}
              </div>
            )}

            {/* Sync Rules Tab */}
            {activeTab === 'sync' && (
              <div className="sync-tab">
                {configs.length === 0 ? (
                  <div className="empty-state">
                    Configure a cloud provider first to set up sync rules.
                  </div>
                ) : (
                  configs.map(config => (
                    <div key={config.provider} className="sync-rules-section">
                      <h3>{getProviderName(config.provider)} Sync Rules</h3>
                      <div className="sync-rules-list">
                        {config.syncRules.length === 0 ? (
                          <div className="empty-state">No sync rules configured</div>
                        ) : (
                          config.syncRules.map(rule => (
                            <div key={rule.id} className="sync-rule-item">
                              <div className="rule-pattern">
                                <code>{rule.pattern}</code>
                              </div>
                              <div className="rule-action">
                                {rule.action}
                              </div>
                              <div className="rule-status">
                                {rule.enabled ? '✓ Enabled' : '✗ Disabled'}
                              </div>
                            </div>
                          ))
                        )}
                      </div>
                    </div>
                  ))
                )}
              </div>
            )}

            {/* Activity Tab */}
            {activeTab === 'activity' && (
              <div className="activity-tab">
                {syncActivity.length === 0 ? (
                  <div className="empty-state">No sync activity yet</div>
                ) : (
                  <div className="activity-list">
                    {syncActivity.map(activity => (
                      <div key={activity.id} className={`activity-item activity-${activity.status}`}>
                        <div className="activity-icon">
                          {getProviderIcon(activity.provider)}
                        </div>
                        <div className="activity-details">
                          <div className="activity-title">
                            {activity.action} • {activity.fileName}
                          </div>
                          <div className="activity-meta">
                            {getProviderName(activity.provider)} •{' '}
                            {new Date(activity.timestamp).toLocaleString()}
                          </div>
                          {activity.error && (
                            <div className="activity-error">{activity.error}</div>
                          )}
                        </div>
                        <div className={`activity-status status-${activity.status}`}>
                          {activity.status}
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
};

// Cloud Provider Configuration Dialog
interface CloudProviderDialogProps {
  config: CloudConfig | null;
  onSave: (config: Partial<CloudConfig>) => void;
  onCancel: () => void;
}

const CloudProviderDialog: React.FC<CloudProviderDialogProps> = ({
  config,
  onSave,
  onCancel,
}) => {
  const [provider, setProvider] = useState<CloudProvider>(config?.provider || 's3');
  const [credentials, setCredentials] = useState<CloudCredentials>(
    config?.credentials || {}
  );
  const [settings, setSettings] = useState<CloudSettings>(
    config?.settings || {
      autoSync: false,
      syncInterval: 60,
      conflictResolution: 'newest',
      bandwidth: { upload: 0, download: 0 },
      encryption: false,
    }
  );

  const handleSave = () => {
    onSave({
      provider,
      enabled: true,
      credentials,
      settings,
      syncRules: config?.syncRules || [],
    });
  };

  return (
    <div className="cloud-dialog-overlay">
      <div className="cloud-dialog">
        <div className="dialog-header">
          <h3>{config ? 'Edit' : 'Add'} Cloud Provider</h3>
          <button onClick={onCancel} className="btn-close">
            ✕
          </button>
        </div>

        <div className="dialog-content">
          {!config && (
            <div className="form-group">
              <label>Provider</label>
              <select
                value={provider}
                onChange={(e) => setProvider(e.target.value as CloudProvider)}
                className="form-select"
              >
                <option value="s3">Amazon S3</option>
                <option value="gcs">Google Cloud Storage</option>
                <option value="azure">Azure Blob Storage</option>
                <option value="dropbox">Dropbox</option>
                <option value="onedrive">OneDrive</option>
              </select>
            </div>
          )}

          {provider === 's3' && (
            <>
              <div className="form-group">
                <label>Access Key ID</label>
                <input
                  type="text"
                  value={credentials.accessKey || ''}
                  onChange={(e) =>
                    setCredentials({ ...credentials, accessKey: e.target.value })
                  }
                  className="form-input"
                />
              </div>
              <div className="form-group">
                <label>Secret Access Key</label>
                <input
                  type="password"
                  value={credentials.secretKey || ''}
                  onChange={(e) =>
                    setCredentials({ ...credentials, secretKey: e.target.value })
                  }
                  className="form-input"
                />
              </div>
              <div className="form-group">
                <label>Bucket Name</label>
                <input
                  type="text"
                  value={credentials.bucket || ''}
                  onChange={(e) =>
                    setCredentials({ ...credentials, bucket: e.target.value })
                  }
                  className="form-input"
                />
              </div>
              <div className="form-group">
                <label>Region</label>
                <input
                  type="text"
                  value={credentials.region || ''}
                  onChange={(e) =>
                    setCredentials({ ...credentials, region: e.target.value })
                  }
                  className="form-input"
                  placeholder="us-east-1"
                />
              </div>
            </>
          )}

          <div className="form-group">
            <label>
              <input
                type="checkbox"
                checked={settings.autoSync}
                onChange={(e) =>
                  setSettings({ ...settings, autoSync: e.target.checked })
                }
              />
              {' '}Enable Auto Sync
            </label>
          </div>

          {settings.autoSync && (
            <div className="form-group">
              <label>Sync Interval (minutes)</label>
              <input
                type="number"
                value={settings.syncInterval}
                onChange={(e) =>
                  setSettings({ ...settings, syncInterval: parseInt(e.target.value) })
                }
                className="form-input"
                min="5"
              />
            </div>
          )}

          <div className="form-group">
            <label>Conflict Resolution</label>
            <select
              value={settings.conflictResolution}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  conflictResolution: e.target.value as any,
                })
              }
              className="form-select"
            >
              <option value="local">Local wins</option>
              <option value="remote">Remote wins</option>
              <option value="newest">Newest wins</option>
              <option value="manual">Manual resolution</option>
            </select>
          </div>

          <div className="form-group">
            <label>
              <input
                type="checkbox"
                checked={settings.encryption}
                onChange={(e) =>
                  setSettings({ ...settings, encryption: e.target.checked })
                }
              />
              {' '}Enable Encryption
            </label>
          </div>
        </div>

        <div className="dialog-actions">
          <button onClick={onCancel} className="btn">
            Cancel
          </button>
          <button onClick={handleSave} className="btn btn-primary">
            Save
          </button>
        </div>
      </div>
    </div>
  );
};

export default FileCloud;
