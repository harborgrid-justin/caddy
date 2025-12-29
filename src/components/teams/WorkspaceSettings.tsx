/**
 * Workspace Settings Component
 *
 * Configuration interface for workspace settings and preferences.
 */

import React, { useState, useEffect } from 'react';
import type {
  Workspace,
  WorkspaceSettings as Settings,
  WorkspaceVisibility,
  WorkingHours,
} from './types';

interface WorkspaceSettingsProps {
  workspaceId: string;
  onSave?: (settings: Settings) => void;
  onCancel?: () => void;
}

type SettingsTab = 'general' | 'members' | 'notifications' | 'integrations' | 'advanced';

export function WorkspaceSettings({
  workspaceId,
  onSave,
  onCancel,
}: WorkspaceSettingsProps) {
  const [activeTab, setActiveTab] = useState<SettingsTab>('general');
  const [workspace, setWorkspace] = useState<Workspace | null>(null);
  const [settings, setSettings] = useState<Settings | null>(null);
  const [hasChanges, setHasChanges] = useState(false);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    loadWorkspace();
  }, [workspaceId]);

  const loadWorkspace = async () => {
    setLoading(true);
    try {
      // Mock implementation
      const mockWorkspace: Workspace = {
        id: workspaceId,
        name: 'Engineering Team',
        slug: 'engineering',
        owner_id: 'user1',
        status: 'Active',
        visibility: 'Private',
        settings: {
          auto_assignment_enabled: true,
          require_member_approval: false,
          allow_guest_access: true,
          default_member_role: 'developer',
          activity_tracking_enabled: true,
          email_notifications_enabled: true,
          slack_integration_enabled: false,
          notification_settings: {},
          timezone: 'UTC',
          working_hours: {
            start_hour: 9,
            end_hour: 17,
            working_days: [1, 2, 3, 4, 5],
          },
        },
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
        metadata: {},
      };
      setWorkspace(mockWorkspace);
      setSettings(mockWorkspace.settings);
    } finally {
      setLoading(false);
    }
  };

  const updateSetting = <K extends keyof Settings>(
    key: K,
    value: Settings[K]
  ) => {
    if (!settings) return;
    setSettings({ ...settings, [key]: value });
    setHasChanges(true);
  };

  const handleSave = async () => {
    if (!settings) return;

    setSaving(true);
    try {
      // In production, would call API
      await new Promise((resolve) => setTimeout(resolve, 1000));
      onSave?.(settings);
      setHasChanges(false);
    } finally {
      setSaving(false);
    }
  };

  const handleCancel = () => {
    if (hasChanges) {
      if (confirm('You have unsaved changes. Discard them?')) {
        onCancel?.();
      }
    } else {
      onCancel?.();
    }
  };

  if (loading || !workspace || !settings) {
    return (
      <div className="workspace-settings loading">
        <div className="loading-spinner" />
        <p>Loading settings...</p>
      </div>
    );
  }

  return (
    <div className="workspace-settings">
      {/* Header */}
      <header className="settings-header">
        <div>
          <h1>Workspace Settings</h1>
          <p className="settings-subtitle">{workspace.name}</p>
        </div>
        <div className="settings-actions">
          <button
            className="btn-secondary"
            onClick={handleCancel}
            disabled={saving}
          >
            Cancel
          </button>
          <button
            className="btn-primary"
            onClick={handleSave}
            disabled={!hasChanges || saving}
          >
            {saving ? 'Saving...' : 'Save Changes'}
          </button>
        </div>
      </header>

      {/* Tabs */}
      <div className="settings-tabs">
        <button
          className={activeTab === 'general' ? 'active' : ''}
          onClick={() => setActiveTab('general')}
        >
          General
        </button>
        <button
          className={activeTab === 'members' ? 'active' : ''}
          onClick={() => setActiveTab('members')}
        >
          Members
        </button>
        <button
          className={activeTab === 'notifications' ? 'active' : ''}
          onClick={() => setActiveTab('notifications')}
        >
          Notifications
        </button>
        <button
          className={activeTab === 'integrations' ? 'active' : ''}
          onClick={() => setActiveTab('integrations')}
        >
          Integrations
        </button>
        <button
          className={activeTab === 'advanced' ? 'active' : ''}
          onClick={() => setActiveTab('advanced')}
        >
          Advanced
        </button>
      </div>

      {/* Tab Content */}
      <div className="settings-content">
        {activeTab === 'general' && (
          <GeneralSettings
            workspace={workspace}
            onUpdate={(field, value) => {
              if (field === 'name' || field === 'description' || field === 'visibility') {
                setWorkspace({ ...workspace, [field]: value });
                setHasChanges(true);
              }
            }}
          />
        )}

        {activeTab === 'members' && (
          <MemberSettings settings={settings} onUpdate={updateSetting} />
        )}

        {activeTab === 'notifications' && (
          <NotificationSettings settings={settings} onUpdate={updateSetting} />
        )}

        {activeTab === 'integrations' && (
          <IntegrationSettings settings={settings} onUpdate={updateSetting} />
        )}

        {activeTab === 'advanced' && (
          <AdvancedSettings
            workspace={workspace}
            settings={settings}
            onUpdate={updateSetting}
          />
        )}
      </div>
    </div>
  );
}

// ============================================================================
// Settings Tab Components
// ============================================================================

interface GeneralSettingsProps {
  workspace: Workspace;
  onUpdate: (field: string, value: any) => void;
}

function GeneralSettings({ workspace, onUpdate }: GeneralSettingsProps) {
  return (
    <div className="settings-section">
      <h2>General Settings</h2>

      <div className="form-group">
        <label htmlFor="workspace-name">Workspace Name</label>
        <input
          id="workspace-name"
          type="text"
          value={workspace.name}
          onChange={(e) => onUpdate('name', e.target.value)}
          placeholder="Enter workspace name"
        />
      </div>

      <div className="form-group">
        <label htmlFor="workspace-slug">Workspace Slug</label>
        <input
          id="workspace-slug"
          type="text"
          value={workspace.slug}
          readOnly
          disabled
        />
        <p className="form-help">
          The workspace slug cannot be changed after creation
        </p>
      </div>

      <div className="form-group">
        <label htmlFor="workspace-description">Description</label>
        <textarea
          id="workspace-description"
          value={workspace.description || ''}
          onChange={(e) => onUpdate('description', e.target.value)}
          placeholder="Describe this workspace"
          rows={3}
        />
      </div>

      <div className="form-group">
        <label htmlFor="workspace-visibility">Visibility</label>
        <select
          id="workspace-visibility"
          value={workspace.visibility}
          onChange={(e) => onUpdate('visibility', e.target.value as WorkspaceVisibility)}
        >
          <option value="Private">Private - Only members can see and access</option>
          <option value="Internal">
            Internal - Anyone can see, only members can access
          </option>
          <option value="Public">Public - Anyone can see and request access</option>
        </select>
      </div>

      <div className="form-group">
        <label htmlFor="workspace-timezone">Timezone</label>
        <select
          id="workspace-timezone"
          value={workspace.settings.timezone}
          onChange={(e) => onUpdate('timezone', e.target.value)}
        >
          <option value="UTC">UTC</option>
          <option value="America/New_York">Eastern Time</option>
          <option value="America/Chicago">Central Time</option>
          <option value="America/Denver">Mountain Time</option>
          <option value="America/Los_Angeles">Pacific Time</option>
          <option value="Europe/London">London</option>
          <option value="Europe/Paris">Paris</option>
          <option value="Asia/Tokyo">Tokyo</option>
          <option value="Asia/Singapore">Singapore</option>
        </select>
      </div>
    </div>
  );
}

interface MemberSettingsProps {
  settings: Settings;
  onUpdate: <K extends keyof Settings>(key: K, value: Settings[K]) => void;
}

function MemberSettings({ settings, onUpdate }: MemberSettingsProps) {
  return (
    <div className="settings-section">
      <h2>Member Settings</h2>

      <div className="form-group">
        <label className="checkbox-label">
          <input
            type="checkbox"
            checked={settings.require_member_approval}
            onChange={(e) => onUpdate('require_member_approval', e.target.checked)}
          />
          <span>Require approval for new members</span>
        </label>
        <p className="form-help">
          When enabled, workspace owners must approve member invitations
        </p>
      </div>

      <div className="form-group">
        <label className="checkbox-label">
          <input
            type="checkbox"
            checked={settings.allow_guest_access}
            onChange={(e) => onUpdate('allow_guest_access', e.target.checked)}
          />
          <span>Allow guest access</span>
        </label>
        <p className="form-help">
          Enable temporary guest access for external collaborators
        </p>
      </div>

      <div className="form-group">
        <label htmlFor="default-role">Default Member Role</label>
        <select
          id="default-role"
          value={settings.default_member_role}
          onChange={(e) => onUpdate('default_member_role', e.target.value)}
        >
          <option value="viewer">Viewer</option>
          <option value="developer">Developer</option>
          <option value="designer">Designer</option>
          <option value="reviewer">Reviewer</option>
        </select>
        <p className="form-help">
          Role automatically assigned to new members
        </p>
      </div>

      <div className="form-group">
        <label htmlFor="max-members">Maximum Members</label>
        <input
          id="max-members"
          type="number"
          value={settings.max_members || ''}
          onChange={(e) =>
            onUpdate(
              'max_members',
              e.target.value ? parseInt(e.target.value) : undefined
            )
          }
          placeholder="Unlimited"
          min="1"
        />
        <p className="form-help">
          Leave empty for unlimited members
        </p>
      </div>

      <div className="form-group">
        <h3>Working Hours</h3>
        <div className="working-hours-grid">
          <div>
            <label htmlFor="start-hour">Start Hour</label>
            <input
              id="start-hour"
              type="number"
              min="0"
              max="23"
              value={settings.working_hours.start_hour}
              onChange={(e) =>
                onUpdate('working_hours', {
                  ...settings.working_hours,
                  start_hour: parseInt(e.target.value),
                })
              }
            />
          </div>
          <div>
            <label htmlFor="end-hour">End Hour</label>
            <input
              id="end-hour"
              type="number"
              min="0"
              max="23"
              value={settings.working_hours.end_hour}
              onChange={(e) =>
                onUpdate('working_hours', {
                  ...settings.working_hours,
                  end_hour: parseInt(e.target.value),
                })
              }
            />
          </div>
        </div>
      </div>
    </div>
  );
}

function NotificationSettings({ settings, onUpdate }: MemberSettingsProps) {
  return (
    <div className="settings-section">
      <h2>Notification Settings</h2>

      <div className="form-group">
        <label className="checkbox-label">
          <input
            type="checkbox"
            checked={settings.email_notifications_enabled}
            onChange={(e) => onUpdate('email_notifications_enabled', e.target.checked)}
          />
          <span>Enable email notifications</span>
        </label>
      </div>

      <div className="form-group">
        <label className="checkbox-label">
          <input
            type="checkbox"
            checked={settings.activity_tracking_enabled}
            onChange={(e) => onUpdate('activity_tracking_enabled', e.target.checked)}
          />
          <span>Enable activity tracking</span>
        </label>
        <p className="form-help">
          Track and display member activity in the workspace
        </p>
      </div>

      <div className="notification-types">
        <h3>Email Notification Types</h3>
        <div className="checkbox-group">
          {[
            { key: 'member_joined', label: 'New member joined' },
            { key: 'issue_assigned', label: 'Issue assigned to me' },
            { key: 'comment_mention', label: 'Mentioned in comment' },
            { key: 'assignment_due', label: 'Assignment due soon' },
            { key: 'weekly_summary', label: 'Weekly activity summary' },
          ].map(({ key, label }) => (
            <label key={key} className="checkbox-label">
              <input
                type="checkbox"
                checked={settings.notification_settings[key] !== false}
                onChange={(e) =>
                  onUpdate('notification_settings', {
                    ...settings.notification_settings,
                    [key]: e.target.checked,
                  })
                }
              />
              <span>{label}</span>
            </label>
          ))}
        </div>
      </div>
    </div>
  );
}

function IntegrationSettings({ settings, onUpdate }: MemberSettingsProps) {
  return (
    <div className="settings-section">
      <h2>Integrations</h2>

      <div className="integration-card">
        <div className="integration-header">
          <h3>Slack Integration</h3>
          <label className="toggle-switch">
            <input
              type="checkbox"
              checked={settings.slack_integration_enabled}
              onChange={(e) => onUpdate('slack_integration_enabled', e.target.checked)}
            />
            <span className="toggle-slider"></span>
          </label>
        </div>
        <p>
          Connect your workspace to Slack for real-time notifications and updates
        </p>
        {settings.slack_integration_enabled && (
          <div className="integration-config">
            <button className="btn-secondary">Configure Slack</button>
          </div>
        )}
      </div>

      <div className="integration-card">
        <div className="integration-header">
          <h3>GitHub Integration</h3>
          <label className="toggle-switch">
            <input type="checkbox" disabled />
            <span className="toggle-slider"></span>
          </label>
        </div>
        <p>Link issues to GitHub pull requests and commits (Coming soon)</p>
      </div>

      <div className="integration-card">
        <div className="integration-header">
          <h3>Jira Integration</h3>
          <label className="toggle-switch">
            <input type="checkbox" disabled />
            <span className="toggle-slider"></span>
          </label>
        </div>
        <p>Synchronize issues with Jira (Coming soon)</p>
      </div>
    </div>
  );
}

interface AdvancedSettingsProps {
  workspace: Workspace;
  settings: Settings;
  onUpdate: <K extends keyof Settings>(key: K, value: Settings[K]) => void;
}

function AdvancedSettings({ workspace, settings, onUpdate }: AdvancedSettingsProps) {
  const [showDangerZone, setShowDangerZone] = useState(false);

  return (
    <div className="settings-section">
      <h2>Advanced Settings</h2>

      <div className="form-group">
        <label className="checkbox-label">
          <input
            type="checkbox"
            checked={settings.auto_assignment_enabled}
            onChange={(e) => onUpdate('auto_assignment_enabled', e.target.checked)}
          />
          <span>Enable auto-assignment</span>
        </label>
        <p className="form-help">
          Automatically assign issues based on workload and availability
        </p>
      </div>

      <div className="form-group">
        <h3>Workspace Data</h3>
        <button className="btn-secondary">Export Workspace Data</button>
        <p className="form-help">
          Download all workspace data as JSON
        </p>
      </div>

      <div className="danger-zone">
        <h3
          className="danger-zone-toggle"
          onClick={() => setShowDangerZone(!showDangerZone)}
        >
          Danger Zone {showDangerZone ? '▼' : '▶'}
        </h3>

        {showDangerZone && (
          <div className="danger-zone-content">
            <div className="danger-action">
              <div>
                <h4>Archive Workspace</h4>
                <p>Archive this workspace and make it read-only</p>
              </div>
              <button className="btn-danger">Archive</button>
            </div>

            <div className="danger-action">
              <div>
                <h4>Delete Workspace</h4>
                <p>
                  Permanently delete this workspace and all its data. This action
                  cannot be undone.
                </p>
              </div>
              <button className="btn-danger">Delete</button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
