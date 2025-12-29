/**
 * Plugin Settings UI Component
 *
 * Configure individual plugin settings, permissions, and resource limits.
 */

import React, { useState } from 'react';
import { usePluginSettings, usePlugin } from './usePlugin';
import {
  PluginSettings as IPluginSettings,
  ResourceLimits,
  Permission,
} from './types';

export interface PluginSettingsProps {
  pluginId: string;
  className?: string;
  onClose?: () => void;
  onSaved?: () => void;
}

export const PluginSettings: React.FC<PluginSettingsProps> = ({
  pluginId,
  className = '',
  onClose,
  onSaved,
}) => {
  const { plugin } = usePlugin(pluginId);
  const { settings, loading, error, updateSettings } = usePluginSettings(pluginId);

  const [localSettings, setLocalSettings] = useState<IPluginSettings | null>(null);
  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);

  // Initialize local settings from loaded settings
  React.useEffect(() => {
    if (settings && !localSettings) {
      setLocalSettings(settings);
    }
  }, [settings]);

  const handleSave = async () => {
    if (!localSettings) return;

    setSaving(true);
    setSaveError(null);

    try {
      await updateSettings({
        enabled: localSettings.enabled,
        config: localSettings.config,
        autoStart: localSettings.autoStart,
        resourceLimits: localSettings.resourceLimits,
      });

      onSaved?.();
      onClose?.();
    } catch (err: any) {
      setSaveError(err.message);
    } finally {
      setSaving(false);
    }
  };

  const handleConfigChange = (key: string, value: any) => {
    if (!localSettings) return;

    setLocalSettings({
      ...localSettings,
      config: {
        ...localSettings.config,
        [key]: value,
      },
    });
  };

  const handleResourceLimitChange = (key: keyof ResourceLimits, value: number) => {
    if (!localSettings) return;

    setLocalSettings({
      ...localSettings,
      resourceLimits: {
        ...(localSettings.resourceLimits || getDefaultResourceLimits()),
        [key]: value,
      },
    });
  };

  if (loading || !localSettings) {
    return (
      <div className={`plugin-settings loading ${className}`}>
        <div className="loading-spinner">Loading settings...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={`plugin-settings error ${className}`}>
        <div className="error-message">
          <h3>Error loading settings</h3>
          <p>{error.message}</p>
        </div>
      </div>
    );
  }

  return (
    <div className={`plugin-settings ${className}`}>
      {/* Header */}
      <div className="settings-header">
        <h2>Plugin Settings</h2>
        {plugin && <h3>{plugin.manifest.name}</h3>}
        {onClose && (
          <button className="btn-close" onClick={onClose}>
            Close
          </button>
        )}
      </div>

      {/* Settings Form */}
      <div className="settings-body">
        {/* General Settings */}
        <section className="settings-section">
          <h4>General</h4>

          <label className="setting-item">
            <input
              type="checkbox"
              checked={localSettings.enabled}
              onChange={(e) =>
                setLocalSettings({ ...localSettings, enabled: e.target.checked })
              }
            />
            <span>Enable plugin</span>
          </label>

          <label className="setting-item">
            <input
              type="checkbox"
              checked={localSettings.autoStart}
              onChange={(e) =>
                setLocalSettings({ ...localSettings, autoStart: e.target.checked })
              }
            />
            <span>Auto-start when CADDY launches</span>
          </label>
        </section>

        {/* Permissions */}
        {plugin && (
          <section className="settings-section">
            <h4>Permissions</h4>
            <div className="permissions-list">
              {plugin.manifest.permissions.map((permission) => (
                <div key={permission} className="permission-item">
                  <span className="permission-icon">✓</span>
                  <span className="permission-name">{permission}</span>
                  <span className="permission-description">
                    {getPermissionDescription(permission)}
                  </span>
                </div>
              ))}
            </div>
          </section>
        )}

        {/* Resource Limits */}
        <section className="settings-section">
          <h4>Resource Limits</h4>

          <div className="setting-item">
            <label>Maximum Memory (MB)</label>
            <input
              type="number"
              value={Math.round(
                (localSettings.resourceLimits?.maxMemoryBytes || 128 * 1024 * 1024) /
                  (1024 * 1024)
              )}
              onChange={(e) =>
                handleResourceLimitChange(
                  'maxMemoryBytes',
                  Number(e.target.value) * 1024 * 1024
                )
              }
              min="16"
              max="2048"
            />
          </div>

          <div className="setting-item">
            <label>Maximum Execution Time (ms)</label>
            <input
              type="number"
              value={localSettings.resourceLimits?.maxExecutionTimeMs || 5000}
              onChange={(e) =>
                handleResourceLimitChange('maxExecutionTimeMs', Number(e.target.value))
              }
              min="100"
              max="60000"
            />
          </div>

          <div className="setting-item">
            <label>Maximum File Size (MB)</label>
            <input
              type="number"
              value={Math.round(
                (localSettings.resourceLimits?.maxFileSizeBytes || 10 * 1024 * 1024) /
                  (1024 * 1024)
              )}
              onChange={(e) =>
                handleResourceLimitChange(
                  'maxFileSizeBytes',
                  Number(e.target.value) * 1024 * 1024
                )
              }
              min="1"
              max="100"
            />
          </div>

          <div className="setting-item">
            <label>Max File Operations/Second</label>
            <input
              type="number"
              value={localSettings.resourceLimits?.maxFileOpsPerSecond || 100}
              onChange={(e) =>
                handleResourceLimitChange('maxFileOpsPerSecond', Number(e.target.value))
              }
              min="1"
              max="1000"
            />
          </div>

          <div className="setting-item">
            <label>Max Network Requests/Second</label>
            <input
              type="number"
              value={localSettings.resourceLimits?.maxNetworkRequestsPerSecond || 10}
              onChange={(e) =>
                handleResourceLimitChange(
                  'maxNetworkRequestsPerSecond',
                  Number(e.target.value)
                )
              }
              min="1"
              max="100"
            />
          </div>

          <div className="setting-item">
            <label>Maximum CPU Usage (%)</label>
            <input
              type="number"
              value={localSettings.resourceLimits?.maxCpuPercent || 50}
              onChange={(e) =>
                handleResourceLimitChange('maxCpuPercent', Number(e.target.value))
              }
              min="1"
              max="100"
            />
          </div>
        </section>

        {/* Plugin-Specific Configuration */}
        {Object.keys(localSettings.config).length > 0 && (
          <section className="settings-section">
            <h4>Plugin Configuration</h4>
            {Object.entries(localSettings.config).map(([key, value]) => (
              <div key={key} className="setting-item">
                <label>{formatConfigKey(key)}</label>
                <ConfigInput
                  value={value}
                  onChange={(newValue) => handleConfigChange(key, newValue)}
                />
              </div>
            ))}
          </section>
        )}

        {/* Plugin Info */}
        {plugin && (
          <section className="settings-section">
            <h4>Plugin Information</h4>
            <dl className="plugin-info-list">
              <dt>Version:</dt>
              <dd>{plugin.manifest.version}</dd>

              <dt>Author:</dt>
              <dd>{plugin.manifest.author}</dd>

              <dt>Type:</dt>
              <dd>{plugin.manifest.pluginType}</dd>

              {plugin.manifest.license && (
                <>
                  <dt>License:</dt>
                  <dd>{plugin.manifest.license}</dd>
                </>
              )}

              {plugin.manifest.website && (
                <>
                  <dt>Website:</dt>
                  <dd>
                    <a href={plugin.manifest.website} target="_blank" rel="noopener noreferrer">
                      {plugin.manifest.website}
                    </a>
                  </dd>
                </>
              )}

              {plugin.manifest.repository && (
                <>
                  <dt>Repository:</dt>
                  <dd>
                    <a href={plugin.manifest.repository} target="_blank" rel="noopener noreferrer">
                      {plugin.manifest.repository}
                    </a>
                  </dd>
                </>
              )}
            </dl>
          </section>
        )}
      </div>

      {/* Footer */}
      <div className="settings-footer">
        {saveError && (
          <div className="error-message">
            <p>{saveError}</p>
          </div>
        )}

        <div className="footer-actions">
          {onClose && (
            <button className="btn-secondary" onClick={onClose} disabled={saving}>
              Cancel
            </button>
          )}
          <button className="btn-primary" onClick={handleSave} disabled={saving}>
            {saving ? 'Saving...' : 'Save Settings'}
          </button>
        </div>
      </div>
    </div>
  );
};

interface ConfigInputProps {
  value: any;
  onChange: (value: any) => void;
}

const ConfigInput: React.FC<ConfigInputProps> = ({ value, onChange }) => {
  if (typeof value === 'boolean') {
    return (
      <input
        type="checkbox"
        checked={value}
        onChange={(e) => onChange(e.target.checked)}
      />
    );
  }

  if (typeof value === 'number') {
    return (
      <input
        type="number"
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
      />
    );
  }

  if (typeof value === 'string') {
    return (
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
      />
    );
  }

  // For complex types, use textarea with JSON
  return (
    <textarea
      value={JSON.stringify(value, null, 2)}
      onChange={(e) => {
        try {
          onChange(JSON.parse(e.target.value));
        } catch {
          // Invalid JSON, ignore
        }
      }}
    />
  );
};

function getDefaultResourceLimits(): ResourceLimits {
  return {
    maxMemoryBytes: 128 * 1024 * 1024,
    maxExecutionTimeMs: 5000,
    maxFileSizeBytes: 10 * 1024 * 1024,
    maxFileOpsPerSecond: 100,
    maxNetworkRequestsPerSecond: 10,
    maxCpuPercent: 50,
  };
}

function formatConfigKey(key: string): string {
  return key
    .replace(/([A-Z])/g, ' $1')
    .replace(/^./, (str) => str.toUpperCase())
    .trim();
}

function getPermissionDescription(permission: string): string {
  const descriptions: Record<string, string> = {
    'geometry:read': 'Read geometry data',
    'geometry:write': 'Create and modify geometry',
    'geometry:delete': 'Delete geometry entities',
    'rendering:read': 'Read rendering state',
    'rendering:write': 'Modify rendering settings',
    'ui:read': 'Read UI state',
    'ui:write': 'Modify UI elements',
    'ui:menu': 'Add menu items',
    'ui:toolbar': 'Add toolbar buttons',
    'file:read': 'Read files from disk',
    'file:write': 'Write files to disk',
    'command:execute': 'Execute commands',
    'network:http': 'Make HTTP requests',
    'system:clipboard': 'Access clipboard',
    'system:notifications': 'Show notifications',
  };

  return descriptions[permission] || 'Unknown permission';
}

export default PluginSettings;
