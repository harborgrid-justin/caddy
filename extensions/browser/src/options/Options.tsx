/**
 * CADDY v0.3.0 - Options Page
 * Extension settings and configuration interface
 */

import React, { useState, useEffect } from 'react';
import type { UserSettings, WCAGLevel, SeverityLevel } from '../shared/types';
import { api } from '../shared/api';

// ============================================================================
// Options Component
// ============================================================================

export const Options: React.FC = () => {
  const [settings, setSettings] = useState<UserSettings | null>(null);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const [activeTab, setActiveTab] = useState<'general' | 'scanning' | 'notifications' | 'account'>('general');

  // --------------------------------------------------------------------------
  // Effects
  // --------------------------------------------------------------------------

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const response = await chrome.runtime.sendMessage({
        type: 'GET_SETTINGS',
        timestamp: Date.now(),
      });
      setSettings(response);
    } catch (error) {
      console.error('[CADDY Options] Error loading settings:', error);
    }
  };

  // --------------------------------------------------------------------------
  // Handlers
  // --------------------------------------------------------------------------

  const handleSave = async () => {
    if (!settings) return;

    setSaving(true);
    setSaved(false);

    try {
      await chrome.runtime.sendMessage({
        type: 'UPDATE_SETTINGS',
        payload: settings,
        timestamp: Date.now(),
      });

      setSaved(true);
      setTimeout(() => setSaved(false), 3000);
    } catch (error) {
      console.error('[CADDY Options] Error saving settings:', error);
    } finally {
      setSaving(false);
    }
  };

  const updateSettings = (updates: Partial<UserSettings>) => {
    setSettings((prev) => (prev ? { ...prev, ...updates } : null));
  };

  const handleAuthenticate = async (apiKey: string) => {
    try {
      await chrome.runtime.sendMessage({
        type: 'AUTHENTICATE',
        payload: { apiKey },
        timestamp: Date.now(),
      });

      alert('Authentication successful!');
      loadSettings();
    } catch (error) {
      console.error('[CADDY Options] Authentication error:', error);
      alert('Authentication failed. Please check your API key.');
    }
  };

  // --------------------------------------------------------------------------
  // Render
  // --------------------------------------------------------------------------

  if (!settings) {
    return (
      <div className="options-container">
        <div className="loading">Loading settings...</div>
      </div>
    );
  }

  return (
    <div className="options-container">
      <header className="options-header">
        <div className="header-content">
          <img src="/icons/icon-48.png" alt="CADDY" className="logo" />
          <div>
            <h1>CADDY Settings</h1>
            <p className="subtitle">Configure your accessibility scanner</p>
          </div>
        </div>
      </header>

      <div className="options-main">
        <nav className="options-nav">
          <button
            className={`nav-button ${activeTab === 'general' ? 'active' : ''}`}
            onClick={() => setActiveTab('general')}
          >
            <GeneralIcon />
            <span>General</span>
          </button>
          <button
            className={`nav-button ${activeTab === 'scanning' ? 'active' : ''}`}
            onClick={() => setActiveTab('scanning')}
          >
            <ScanIcon />
            <span>Scanning</span>
          </button>
          <button
            className={`nav-button ${activeTab === 'notifications' ? 'active' : ''}`}
            onClick={() => setActiveTab('notifications')}
          >
            <NotificationIcon />
            <span>Notifications</span>
          </button>
          <button
            className={`nav-button ${activeTab === 'account' ? 'active' : ''}`}
            onClick={() => setActiveTab('account')}
          >
            <AccountIcon />
            <span>Account</span>
          </button>
        </nav>

        <div className="options-content">
          {activeTab === 'general' && (
            <GeneralSettings settings={settings} updateSettings={updateSettings} />
          )}
          {activeTab === 'scanning' && (
            <ScanningSettings settings={settings} updateSettings={updateSettings} />
          )}
          {activeTab === 'notifications' && (
            <NotificationSettings settings={settings} updateSettings={updateSettings} />
          )}
          {activeTab === 'account' && (
            <AccountSettings
              settings={settings}
              updateSettings={updateSettings}
              onAuthenticate={handleAuthenticate}
            />
          )}

          <div className="options-footer">
            <button
              className="btn btn-primary"
              onClick={handleSave}
              disabled={saving}
            >
              {saving ? 'Saving...' : saved ? 'Saved!' : 'Save Settings'}
            </button>
            {saved && <span className="save-indicator">✓ Settings saved</span>}
          </div>
        </div>
      </div>
    </div>
  );
};

// ============================================================================
// Settings Sections
// ============================================================================

const GeneralSettings: React.FC<{
  settings: UserSettings;
  updateSettings: (updates: Partial<UserSettings>) => void;
}> = ({ settings, updateSettings }) => (
  <div className="settings-section">
    <h2>General Settings</h2>

    <div className="setting-group">
      <label className="setting-label">
        <span className="label-text">Theme</span>
        <select
          className="setting-select"
          value={settings.theme}
          onChange={(e) =>
            updateSettings({ theme: e.target.value as 'light' | 'dark' | 'auto' })
          }
        >
          <option value="auto">Auto (System)</option>
          <option value="light">Light</option>
          <option value="dark">Dark</option>
        </select>
      </label>
      <p className="setting-description">Choose the extension's color theme</p>
    </div>

    <div className="setting-group">
      <label className="setting-label">
        <span className="label-text">API Endpoint</span>
        <input
          type="url"
          className="setting-input"
          value={settings.apiEndpoint}
          onChange={(e) => updateSettings({ apiEndpoint: e.target.value })}
          placeholder="https://api.caddy.dev/v1"
        />
      </label>
      <p className="setting-description">
        Custom API endpoint for enterprise deployments
      </p>
    </div>

    <div className="setting-group">
      <label className="setting-checkbox">
        <input
          type="checkbox"
          checked={settings.syncEnabled}
          onChange={(e) => updateSettings({ syncEnabled: e.target.checked })}
        />
        <span className="checkbox-label">Enable cloud sync</span>
      </label>
      <p className="setting-description">
        Sync scan results and settings across devices (requires account)
      </p>
    </div>
  </div>
);

const ScanningSettings: React.FC<{
  settings: UserSettings;
  updateSettings: (updates: Partial<UserSettings>) => void;
}> = ({ settings, updateSettings }) => {
  const updateScanning = (updates: Partial<typeof settings.scanning>) => {
    updateSettings({ scanning: { ...settings.scanning, ...updates } });
  };

  return (
    <div className="settings-section">
      <h2>Scanning Settings</h2>

      <div className="setting-group">
        <label className="setting-label">
          <span className="label-text">WCAG Level</span>
          <select
            className="setting-select"
            value={settings.scanning.wcagLevel}
            onChange={(e) =>
              updateScanning({ wcagLevel: e.target.value as WCAGLevel })
            }
          >
            <option value="A">Level A</option>
            <option value="AA">Level AA (Recommended)</option>
            <option value="AAA">Level AAA</option>
          </select>
        </label>
        <p className="setting-description">
          WCAG conformance level to check against
        </p>
      </div>

      <div className="setting-group">
        <label className="setting-checkbox">
          <input
            type="checkbox"
            checked={settings.scanning.autoScan}
            onChange={(e) => updateScanning({ autoScan: e.target.checked })}
          />
          <span className="checkbox-label">Enable auto-scan</span>
        </label>
        <p className="setting-description">
          Automatically scan pages when you visit them
        </p>
      </div>

      <div className="setting-group">
        <label className="setting-checkbox">
          <input
            type="checkbox"
            checked={settings.scanning.scanOnLoad}
            onChange={(e) => updateScanning({ scanOnLoad: e.target.checked })}
          />
          <span className="checkbox-label">Scan on page load</span>
        </label>
        <p className="setting-description">
          Run a scan when the page finishes loading
        </p>
      </div>

      <div className="setting-group">
        <label className="setting-checkbox">
          <input
            type="checkbox"
            checked={settings.scanning.scanFrames}
            onChange={(e) => updateScanning({ scanFrames: e.target.checked })}
          />
          <span className="checkbox-label">Scan iframes</span>
        </label>
        <p className="setting-description">
          Include iframe content in accessibility scans
        </p>
      </div>

      <div className="setting-group">
        <label className="setting-label">
          <span className="label-text">Max Issues</span>
          <input
            type="number"
            className="setting-input"
            value={settings.scanning.maxIssues}
            onChange={(e) =>
              updateScanning({ maxIssues: parseInt(e.target.value) })
            }
            min="100"
            max="10000"
            step="100"
          />
        </label>
        <p className="setting-description">
          Maximum number of issues to report per scan
        </p>
      </div>
    </div>
  );
};

const NotificationSettings: React.FC<{
  settings: UserSettings;
  updateSettings: (updates: Partial<UserSettings>) => void;
}> = ({ settings, updateSettings }) => {
  const updateNotifications = (
    updates: Partial<typeof settings.notifications>
  ) => {
    updateSettings({ notifications: { ...settings.notifications, ...updates } });
  };

  const toggleSeverity = (severity: SeverityLevel) => {
    const current = settings.notifications.severity;
    const updated = current.includes(severity)
      ? current.filter((s) => s !== severity)
      : [...current, severity];
    updateNotifications({ severity: updated });
  };

  return (
    <div className="settings-section">
      <h2>Notification Settings</h2>

      <div className="setting-group">
        <label className="setting-checkbox">
          <input
            type="checkbox"
            checked={settings.notifications.enabled}
            onChange={(e) => updateNotifications({ enabled: e.target.checked })}
          />
          <span className="checkbox-label">Enable notifications</span>
        </label>
        <p className="setting-description">
          Show desktop notifications for accessibility issues
        </p>
      </div>

      <div className="setting-group">
        <label className="setting-checkbox">
          <input
            type="checkbox"
            checked={settings.notifications.onNewIssues}
            onChange={(e) =>
              updateNotifications({ onNewIssues: e.target.checked })
            }
            disabled={!settings.notifications.enabled}
          />
          <span className="checkbox-label">Notify on new issues</span>
        </label>
        <p className="setting-description">
          Get notified when new issues are detected
        </p>
      </div>

      <div className="setting-group">
        <label className="setting-checkbox">
          <input
            type="checkbox"
            checked={settings.notifications.onScanComplete}
            onChange={(e) =>
              updateNotifications({ onScanComplete: e.target.checked })
            }
            disabled={!settings.notifications.enabled}
          />
          <span className="checkbox-label">Notify on scan complete</span>
        </label>
        <p className="setting-description">Get notified when scans finish</p>
      </div>

      <div className="setting-group">
        <label className="setting-label">
          <span className="label-text">Severity Filter</span>
        </label>
        <p className="setting-description">
          Only notify for these severity levels:
        </p>
        <div className="checkbox-group">
          {(['critical', 'serious', 'moderate', 'minor'] as SeverityLevel[]).map(
            (severity) => (
              <label key={severity} className="setting-checkbox">
                <input
                  type="checkbox"
                  checked={settings.notifications.severity.includes(severity)}
                  onChange={() => toggleSeverity(severity)}
                  disabled={!settings.notifications.enabled}
                />
                <span className={`checkbox-label severity-${severity}`}>
                  {severity.charAt(0).toUpperCase() + severity.slice(1)}
                </span>
              </label>
            )
          )}
        </div>
      </div>
    </div>
  );
};

const AccountSettings: React.FC<{
  settings: UserSettings;
  updateSettings: (updates: Partial<UserSettings>) => void;
  onAuthenticate: (apiKey: string) => void;
}> = ({ settings, updateSettings, onAuthenticate }) => {
  const [apiKey, setApiKey] = useState('');
  const [authenticated, setAuthenticated] = useState(false);

  useEffect(() => {
    checkAuthentication();
  }, []);

  const checkAuthentication = async () => {
    const result = await chrome.storage.local.get(['auth']);
    setAuthenticated(!!result.auth?.token);
  };

  const handleConnect = () => {
    if (apiKey.trim()) {
      onAuthenticate(apiKey.trim());
      setApiKey('');
    }
  };

  const handleDisconnect = async () => {
    await chrome.storage.local.remove(['auth']);
    setAuthenticated(false);
  };

  return (
    <div className="settings-section">
      <h2>Account Settings</h2>

      {authenticated ? (
        <div className="account-connected">
          <div className="status-card success">
            <CheckIcon />
            <div>
              <h3>Account Connected</h3>
              <p>You're connected to CADDY Pro</p>
            </div>
          </div>

          <div className="account-features">
            <h3>Pro Features</h3>
            <ul className="feature-list">
              <li>
                <CheckIcon />
                <span>Cloud sync across devices</span>
              </li>
              <li>
                <CheckIcon />
                <span>Advanced reporting</span>
              </li>
              <li>
                <CheckIcon />
                <span>Custom rules engine</span>
              </li>
              <li>
                <CheckIcon />
                <span>Priority support</span>
              </li>
            </ul>
          </div>

          <button className="btn btn-secondary" onClick={handleDisconnect}>
            Disconnect Account
          </button>
        </div>
      ) : (
        <div className="account-disconnected">
          <div className="status-card">
            <InfoIcon />
            <div>
              <h3>Connect Your Account</h3>
              <p>Get access to Pro features with a CADDY account</p>
            </div>
          </div>

          <div className="setting-group">
            <label className="setting-label">
              <span className="label-text">API Key</span>
              <input
                type="password"
                className="setting-input"
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                placeholder="Enter your API key"
              />
            </label>
            <p className="setting-description">
              Get your API key from{' '}
              <a
                href="https://caddy.dev/account"
                target="_blank"
                rel="noopener noreferrer"
              >
                caddy.dev/account
              </a>
            </p>
          </div>

          <button
            className="btn btn-primary"
            onClick={handleConnect}
            disabled={!apiKey.trim()}
          >
            Connect Account
          </button>
        </div>
      )}

      <div className="account-info">
        <h3>About CADDY Pro</h3>
        <p>
          CADDY Pro provides advanced accessibility scanning features for
          professional developers and teams. Learn more at{' '}
          <a
            href="https://caddy.dev/pro"
            target="_blank"
            rel="noopener noreferrer"
          >
            caddy.dev/pro
          </a>
        </p>
      </div>
    </div>
  );
};

// ============================================================================
// Icons
// ============================================================================

const GeneralIcon = () => (
  <svg width="20" height="20" fill="currentColor" viewBox="0 0 16 16">
    <path d="M9.405 1.05c-.413-1.4-2.397-1.4-2.81 0l-.1.34a1.464 1.464 0 0 1-2.105.872l-.31-.17c-1.283-.698-2.686.705-1.987 1.987l.169.311c.446.82.023 1.841-.872 2.105l-.34.1c-1.4.413-1.4 2.397 0 2.81l.34.1a1.464 1.464 0 0 1 .872 2.105l-.17.31c-.698 1.283.705 2.686 1.987 1.987l.311-.169a1.464 1.464 0 0 1 2.105.872l.1.34c.413 1.4 2.397 1.4 2.81 0l.1-.34a1.464 1.464 0 0 1 2.105-.872l.31.17c1.283.698 2.686-.705 1.987-1.987l-.169-.311a1.464 1.464 0 0 1 .872-2.105l.34-.1c1.4-.413 1.4-2.397 0-2.81l-.34-.1a1.464 1.464 0 0 1-.872-2.105l.17-.31c.698-1.283-.705-2.686-1.987-1.987l-.311.169a1.464 1.464 0 0 1-2.105-.872l-.1-.34zM8 10.93a2.929 2.929 0 1 1 0-5.86 2.929 2.929 0 0 1 0 5.858z" />
  </svg>
);

const ScanIcon = () => (
  <svg width="20" height="20" fill="currentColor" viewBox="0 0 16 16">
    <path d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
  </svg>
);

const NotificationIcon = () => (
  <svg width="20" height="20" fill="currentColor" viewBox="0 0 16 16">
    <path d="M8 16a2 2 0 0 0 2-2H6a2 2 0 0 0 2 2zM8 1.918l-.797.161A4.002 4.002 0 0 0 4 6c0 .628-.134 2.197-.459 3.742-.16.767-.376 1.566-.663 2.258h10.244c-.287-.692-.502-1.49-.663-2.258C12.134 8.197 12 6.628 12 6a4.002 4.002 0 0 0-3.203-3.92L8 1.917zM14.22 12c.223.447.481.801.78 1H1c.299-.199.557-.553.78-1C2.68 10.2 3 6.88 3 6c0-2.42 1.72-4.44 4.005-4.901a1 1 0 1 1 1.99 0A5.002 5.002 0 0 1 13 6c0 .88.32 4.2 1.22 6z" />
  </svg>
);

const AccountIcon = () => (
  <svg width="20" height="20" fill="currentColor" viewBox="0 0 16 16">
    <path d="M11 6a3 3 0 1 1-6 0 3 3 0 0 1 6 0z" />
    <path d="M0 8a8 8 0 1 1 16 0A8 8 0 0 1 0 8zm8-7a7 7 0 0 0-5.468 11.37C3.242 11.226 4.805 10 8 10s4.757 1.225 5.468 2.37A7 7 0 0 0 8 1z" />
  </svg>
);

const CheckIcon = () => (
  <svg width="16" height="16" fill="currentColor" viewBox="0 0 16 16">
    <path d="M10.97 4.97a.75.75 0 0 1 1.07 1.05l-3.99 4.99a.75.75 0 0 1-1.08.02L4.324 8.384a.75.75 0 1 1 1.06-1.06l2.094 2.093 3.473-4.425a.267.267 0 0 1 .02-.022z" />
  </svg>
);

const InfoIcon = () => (
  <svg width="16" height="16" fill="currentColor" viewBox="0 0 16 16">
    <path d="M8 15A7 7 0 1 1 8 1a7 7 0 0 1 0 14zm0 1A8 8 0 1 0 8 0a8 8 0 0 0 0 16z" />
    <path d="m8.93 6.588-2.29.287-.082.38.45.083c.294.07.352.176.288.469l-.738 3.468c-.194.897.105 1.319.808 1.319.545 0 1.178-.252 1.465-.598l.088-.416c-.2.176-.492.246-.686.246-.275 0-.375-.193-.304-.533L8.93 6.588zM9 4.5a1 1 0 1 1-2 0 1 1 0 0 1 2 0z" />
  </svg>
);

export default Options;
