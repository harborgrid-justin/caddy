/**
 * Notification Settings Component
 *
 * Provides a comprehensive UI for configuring notifications:
 * - Email, Slack, Teams, and webhook configuration
 * - Notification preferences and filters
 * - Quiet hours setup
 * - Test notification functionality
 * - Notification history
 */

import React, { useState, useEffect, useCallback } from 'react';
import {
  NotificationChannel,
  NotificationSeverity,
  NotificationPriority,
  NotificationPreferences,
  EmailConfig,
  SlackConfig,
  TeamsConfig,
  WebhookConfig,
  QuietHours,
  Notification,
} from './types';

interface NotificationSettingsProps {
  apiBaseUrl?: string;
  userId?: string;
  onSettingsSaved?: () => void;
}

export const NotificationSettings: React.FC<NotificationSettingsProps> = ({
  apiBaseUrl = '/api/notifications',
  userId = 'default-user',
  onSettingsSaved,
}) => {
  const [preferences, setPreferences] = useState<NotificationPreferences>({
    user_id: userId,
    enabled_channels: [NotificationChannel.Email],
    min_severity: NotificationSeverity.Warning,
    min_priority: NotificationPriority.Normal,
    source_filters: [],
  });

  const [emailConfig, setEmailConfig] = useState<EmailConfig>({
    smtp_host: '',
    smtp_port: 587,
    smtp_username: '',
    smtp_password: '',
    from_address: '',
    from_name: 'CADDY Monitoring',
    to_addresses: [],
    use_tls: true,
  });

  const [slackConfig, setSlackConfig] = useState<SlackConfig>({
    webhook_url: '',
    channel: '#alerts',
    username: 'CADDY Bot',
    icon_emoji: ':robot_face:',
  });

  const [teamsConfig, setTeamsConfig] = useState<TeamsConfig>({
    webhook_url: '',
  });

  const [webhookConfig, setWebhookConfig] = useState<WebhookConfig>({
    url: '',
    method: 'POST',
    headers: {},
    timeout_seconds: 30,
  });

  const [quietHours, setQuietHours] = useState<QuietHours>({
    start_hour: 22,
    end_hour: 8,
    timezone: 'UTC',
  });

  const [newEmailAddress, setNewEmailAddress] = useState('');
  const [newSourceFilter, setNewSourceFilter] = useState('');
  const [newHeaderKey, setNewHeaderKey] = useState('');
  const [newHeaderValue, setNewHeaderValue] = useState('');

  const [activeTab, setActiveTab] = useState<'preferences' | 'email' | 'slack' | 'teams' | 'webhook' | 'history'>('preferences');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [testResults, setTestResults] = useState<Record<string, string>>({});
  const [notificationHistory, setNotificationHistory] = useState<Notification[]>([]);

  // Load preferences
  const loadPreferences = useCallback(async () => {
    try {
      const response = await fetch(`${apiBaseUrl}/preferences/${userId}`);
      if (response.ok) {
        const data = await response.json();
        setPreferences(data);
      }
    } catch (err) {
      console.error('Error loading preferences:', err);
    }
  }, [apiBaseUrl, userId]);

  // Load channel configs
  const loadChannelConfigs = useCallback(async () => {
    try {
      const [emailRes, slackRes, teamsRes, webhookRes] = await Promise.all([
        fetch(`${apiBaseUrl}/channels/email/config`),
        fetch(`${apiBaseUrl}/channels/slack/config`),
        fetch(`${apiBaseUrl}/channels/teams/config`),
        fetch(`${apiBaseUrl}/channels/webhook/config`),
      ]);

      if (emailRes.ok) {
        const data = await emailRes.json();
        setEmailConfig(data);
      }
      if (slackRes.ok) {
        const data = await slackRes.json();
        setSlackConfig(data);
      }
      if (teamsRes.ok) {
        const data = await teamsRes.json();
        setTeamsConfig(data);
      }
      if (webhookRes.ok) {
        const data = await webhookRes.json();
        setWebhookConfig(data);
      }
    } catch (err) {
      console.error('Error loading channel configs:', err);
    }
  }, [apiBaseUrl]);

  // Load notification history
  const loadHistory = useCallback(async () => {
    try {
      const response = await fetch(`${apiBaseUrl}/history?limit=50`);
      if (response.ok) {
        const data = await response.json();
        setNotificationHistory(data);
      }
    } catch (err) {
      console.error('Error loading history:', err);
    }
  }, [apiBaseUrl]);

  useEffect(() => {
    loadPreferences();
    loadChannelConfigs();
    loadHistory();
  }, [loadPreferences, loadChannelConfigs, loadHistory]);

  // Save preferences
  const handleSavePreferences = async () => {
    setLoading(true);
    setError(null);
    setSuccessMessage(null);

    try {
      const response = await fetch(`${apiBaseUrl}/preferences`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(preferences),
      });

      if (!response.ok) throw new Error('Failed to save preferences');

      setSuccessMessage('Preferences saved successfully');
      if (onSettingsSaved) {
        onSettingsSaved();
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  // Save channel config
  const handleSaveChannelConfig = async (channel: string, config: any) => {
    setLoading(true);
    setError(null);
    setSuccessMessage(null);

    try {
      const response = await fetch(`${apiBaseUrl}/channels/${channel}/config`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config),
      });

      if (!response.ok) throw new Error(`Failed to save ${channel} configuration`);

      setSuccessMessage(`${channel.charAt(0).toUpperCase() + channel.slice(1)} configuration saved successfully`);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  // Test channel
  const handleTestChannel = async (channel: NotificationChannel) => {
    setTestResults({ ...testResults, [channel]: 'Testing...' });

    try {
      const response = await fetch(`${apiBaseUrl}/channels/${channel.toLowerCase()}/test`, {
        method: 'POST',
      });

      if (response.ok) {
        setTestResults({ ...testResults, [channel]: 'Test successful!' });
      } else {
        const errorData = await response.json();
        setTestResults({
          ...testResults,
          [channel]: `Test failed: ${errorData.message}`,
        });
      }
    } catch (err) {
      setTestResults({
        ...testResults,
        [channel]: `Test failed: ${err instanceof Error ? err.message : 'Unknown error'}`,
      });
    }
  };

  // Toggle channel
  const toggleChannel = (channel: NotificationChannel) => {
    const channels = preferences.enabled_channels.includes(channel)
      ? preferences.enabled_channels.filter((c) => c !== channel)
      : [...preferences.enabled_channels, channel];

    setPreferences({ ...preferences, enabled_channels: channels });
  };

  // Add email address
  const handleAddEmail = () => {
    if (newEmailAddress && !emailConfig.to_addresses.includes(newEmailAddress)) {
      setEmailConfig({
        ...emailConfig,
        to_addresses: [...emailConfig.to_addresses, newEmailAddress],
      });
      setNewEmailAddress('');
    }
  };

  // Remove email address
  const handleRemoveEmail = (email: string) => {
    setEmailConfig({
      ...emailConfig,
      to_addresses: emailConfig.to_addresses.filter((e) => e !== email),
    });
  };

  // Add source filter
  const handleAddSourceFilter = () => {
    if (newSourceFilter && !preferences.source_filters.includes(newSourceFilter)) {
      setPreferences({
        ...preferences,
        source_filters: [...preferences.source_filters, newSourceFilter],
      });
      setNewSourceFilter('');
    }
  };

  // Remove source filter
  const handleRemoveSourceFilter = (filter: string) => {
    setPreferences({
      ...preferences,
      source_filters: preferences.source_filters.filter((f) => f !== filter),
    });
  };

  // Add webhook header
  const handleAddWebhookHeader = () => {
    if (newHeaderKey && newHeaderValue) {
      setWebhookConfig({
        ...webhookConfig,
        headers: { ...webhookConfig.headers, [newHeaderKey]: newHeaderValue },
      });
      setNewHeaderKey('');
      setNewHeaderValue('');
    }
  };

  // Remove webhook header
  const handleRemoveWebhookHeader = (key: string) => {
    const { [key]: _, ...rest } = webhookConfig.headers;
    setWebhookConfig({ ...webhookConfig, headers: rest });
  };

  // Format date
  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleString();
  };

  // Get severity color
  const getSeverityColor = (severity: NotificationSeverity): string => {
    switch (severity) {
      case NotificationSeverity.Critical:
        return 'text-red-600 bg-red-100';
      case NotificationSeverity.Error:
        return 'text-orange-600 bg-orange-100';
      case NotificationSeverity.Warning:
        return 'text-yellow-600 bg-yellow-100';
      default:
        return 'text-blue-600 bg-blue-100';
    }
  };

  return (
    <div className="notification-settings p-6 bg-gray-50 min-h-screen">
      <div className="max-w-5xl mx-auto">
        {/* Header */}
        <div className="mb-6">
          <h1 className="text-3xl font-bold text-gray-900">Notification Settings</h1>
          <p className="mt-2 text-gray-600">
            Configure how and when you receive notifications
          </p>
        </div>

        {/* Messages */}
        {error && (
          <div className="mb-4 p-4 bg-red-50 border border-red-200 rounded-lg">
            <p className="text-red-800">{error}</p>
          </div>
        )}

        {successMessage && (
          <div className="mb-4 p-4 bg-green-50 border border-green-200 rounded-lg">
            <p className="text-green-800">{successMessage}</p>
          </div>
        )}

        {/* Tabs */}
        <div className="mb-6 border-b border-gray-200">
          <nav className="flex space-x-8">
            {['preferences', 'email', 'slack', 'teams', 'webhook', 'history'].map((tab) => (
              <button
                key={tab}
                onClick={() => setActiveTab(tab as any)}
                className={`py-4 px-1 border-b-2 font-medium text-sm transition ${
                  activeTab === tab
                    ? 'border-blue-600 text-blue-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                }`}
              >
                {tab.charAt(0).toUpperCase() + tab.slice(1)}
              </button>
            ))}
          </nav>
        </div>

        {/* Preferences Tab */}
        {activeTab === 'preferences' && (
          <div className="bg-white p-6 rounded-lg shadow">
            <h2 className="text-xl font-bold mb-4">Notification Preferences</h2>

            <div className="space-y-6">
              {/* Enabled Channels */}
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-3">
                  Enabled Notification Channels
                </label>
                <div className="space-y-2">
                  {Object.values(NotificationChannel).map((channel) => (
                    <div key={channel} className="flex items-center">
                      <input
                        type="checkbox"
                        id={`channel-${channel}`}
                        checked={preferences.enabled_channels.includes(channel)}
                        onChange={() => toggleChannel(channel)}
                        className="mr-3"
                      />
                      <label htmlFor={`channel-${channel}`} className="text-sm text-gray-700">
                        {channel}
                      </label>
                    </div>
                  ))}
                </div>
              </div>

              {/* Minimum Severity */}
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Minimum Severity
                </label>
                <select
                  value={preferences.min_severity}
                  onChange={(e) =>
                    setPreferences({
                      ...preferences,
                      min_severity: e.target.value as NotificationSeverity,
                    })
                  }
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                >
                  {Object.values(NotificationSeverity).map((severity) => (
                    <option key={severity} value={severity}>
                      {severity}
                    </option>
                  ))}
                </select>
                <p className="mt-1 text-xs text-gray-500">
                  Only receive notifications at or above this severity level
                </p>
              </div>

              {/* Minimum Priority */}
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Minimum Priority
                </label>
                <select
                  value={preferences.min_priority}
                  onChange={(e) =>
                    setPreferences({
                      ...preferences,
                      min_priority: e.target.value as NotificationPriority,
                    })
                  }
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                >
                  {Object.values(NotificationPriority).map((priority) => (
                    <option key={priority} value={priority}>
                      {priority}
                    </option>
                  ))}
                </select>
              </div>

              {/* Source Filters */}
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Source Filters
                </label>
                <div className="flex gap-2 mb-2">
                  <input
                    type="text"
                    value={newSourceFilter}
                    onChange={(e) => setNewSourceFilter(e.target.value)}
                    onKeyPress={(e) => e.key === 'Enter' && handleAddSourceFilter()}
                    placeholder="monitor-system"
                    className="flex-1 px-3 py-2 border border-gray-300 rounded-lg"
                  />
                  <button
                    onClick={handleAddSourceFilter}
                    className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
                  >
                    Add
                  </button>
                </div>
                <div className="flex flex-wrap gap-2">
                  {preferences.source_filters.map((filter) => (
                    <span
                      key={filter}
                      className="px-3 py-1 bg-gray-100 text-gray-700 rounded-full text-sm flex items-center gap-2"
                    >
                      {filter}
                      <button
                        onClick={() => handleRemoveSourceFilter(filter)}
                        className="text-red-600 hover:text-red-800"
                      >
                        ×
                      </button>
                    </span>
                  ))}
                </div>
                <p className="mt-1 text-xs text-gray-500">
                  Leave empty to receive notifications from all sources
                </p>
              </div>

              {/* Quiet Hours */}
              <div>
                <div className="flex items-center mb-3">
                  <input
                    type="checkbox"
                    id="enable-quiet-hours"
                    checked={preferences.quiet_hours !== undefined}
                    onChange={(e) =>
                      setPreferences({
                        ...preferences,
                        quiet_hours: e.target.checked ? quietHours : undefined,
                      })
                    }
                    className="mr-2"
                  />
                  <label htmlFor="enable-quiet-hours" className="text-sm font-medium text-gray-700">
                    Enable Quiet Hours
                  </label>
                </div>

                {preferences.quiet_hours && (
                  <div className="grid grid-cols-3 gap-4 ml-6">
                    <div>
                      <label className="block text-sm text-gray-700 mb-1">Start Hour</label>
                      <input
                        type="number"
                        min="0"
                        max="23"
                        value={quietHours.start_hour}
                        onChange={(e) =>
                          setQuietHours({
                            ...quietHours,
                            start_hour: parseInt(e.target.value),
                          })
                        }
                        className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                      />
                    </div>
                    <div>
                      <label className="block text-sm text-gray-700 mb-1">End Hour</label>
                      <input
                        type="number"
                        min="0"
                        max="23"
                        value={quietHours.end_hour}
                        onChange={(e) =>
                          setQuietHours({ ...quietHours, end_hour: parseInt(e.target.value) })
                        }
                        className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                      />
                    </div>
                    <div>
                      <label className="block text-sm text-gray-700 mb-1">Timezone</label>
                      <input
                        type="text"
                        value={quietHours.timezone}
                        onChange={(e) =>
                          setQuietHours({ ...quietHours, timezone: e.target.value })
                        }
                        className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                      />
                    </div>
                  </div>
                )}
              </div>

              <button
                onClick={handleSavePreferences}
                disabled={loading}
                className="w-full px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition disabled:opacity-50"
              >
                {loading ? 'Saving...' : 'Save Preferences'}
              </button>
            </div>
          </div>
        )}

        {/* Email Tab */}
        {activeTab === 'email' && (
          <div className="bg-white p-6 rounded-lg shadow">
            <h2 className="text-xl font-bold mb-4">Email Configuration</h2>

            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    SMTP Host
                  </label>
                  <input
                    type="text"
                    value={emailConfig.smtp_host}
                    onChange={(e) => setEmailConfig({ ...emailConfig, smtp_host: e.target.value })}
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                    placeholder="smtp.gmail.com"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    SMTP Port
                  </label>
                  <input
                    type="number"
                    value={emailConfig.smtp_port}
                    onChange={(e) =>
                      setEmailConfig({ ...emailConfig, smtp_port: parseInt(e.target.value) })
                    }
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Username
                  </label>
                  <input
                    type="text"
                    value={emailConfig.smtp_username}
                    onChange={(e) =>
                      setEmailConfig({ ...emailConfig, smtp_username: e.target.value })
                    }
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Password
                  </label>
                  <input
                    type="password"
                    value={emailConfig.smtp_password}
                    onChange={(e) =>
                      setEmailConfig({ ...emailConfig, smtp_password: e.target.value })
                    }
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    From Address
                  </label>
                  <input
                    type="email"
                    value={emailConfig.from_address}
                    onChange={(e) =>
                      setEmailConfig({ ...emailConfig, from_address: e.target.value })
                    }
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    From Name
                  </label>
                  <input
                    type="text"
                    value={emailConfig.from_name}
                    onChange={(e) => setEmailConfig({ ...emailConfig, from_name: e.target.value })}
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                  />
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Recipient Addresses
                </label>
                <div className="flex gap-2 mb-2">
                  <input
                    type="email"
                    value={newEmailAddress}
                    onChange={(e) => setNewEmailAddress(e.target.value)}
                    onKeyPress={(e) => e.key === 'Enter' && handleAddEmail()}
                    placeholder="admin@example.com"
                    className="flex-1 px-3 py-2 border border-gray-300 rounded-lg"
                  />
                  <button
                    onClick={handleAddEmail}
                    className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
                  >
                    Add
                  </button>
                </div>
                <div className="flex flex-wrap gap-2">
                  {emailConfig.to_addresses.map((email) => (
                    <span
                      key={email}
                      className="px-3 py-1 bg-gray-100 text-gray-700 rounded-full text-sm flex items-center gap-2"
                    >
                      {email}
                      <button
                        onClick={() => handleRemoveEmail(email)}
                        className="text-red-600 hover:text-red-800"
                      >
                        ×
                      </button>
                    </span>
                  ))}
                </div>
              </div>

              <div className="flex items-center">
                <input
                  type="checkbox"
                  id="use-tls"
                  checked={emailConfig.use_tls}
                  onChange={(e) => setEmailConfig({ ...emailConfig, use_tls: e.target.checked })}
                  className="mr-2"
                />
                <label htmlFor="use-tls" className="text-sm text-gray-700">
                  Use TLS
                </label>
              </div>

              <div className="flex gap-4">
                <button
                  onClick={() => handleSaveChannelConfig('email', emailConfig)}
                  disabled={loading}
                  className="flex-1 px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition disabled:opacity-50"
                >
                  {loading ? 'Saving...' : 'Save Configuration'}
                </button>
                <button
                  onClick={() => handleTestChannel(NotificationChannel.Email)}
                  className="px-6 py-3 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 transition"
                >
                  Test
                </button>
              </div>

              {testResults[NotificationChannel.Email] && (
                <p className="text-sm text-gray-600">{testResults[NotificationChannel.Email]}</p>
              )}
            </div>
          </div>
        )}

        {/* Slack Tab */}
        {activeTab === 'slack' && (
          <div className="bg-white p-6 rounded-lg shadow">
            <h2 className="text-xl font-bold mb-4">Slack Configuration</h2>

            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Webhook URL
                </label>
                <input
                  type="url"
                  value={slackConfig.webhook_url}
                  onChange={(e) => setSlackConfig({ ...slackConfig, webhook_url: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                  placeholder="https://hooks.slack.com/services/YOUR/WEBHOOK/URL"
                />
              </div>

              <div className="grid grid-cols-3 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Channel
                  </label>
                  <input
                    type="text"
                    value={slackConfig.channel}
                    onChange={(e) => setSlackConfig({ ...slackConfig, channel: e.target.value })}
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                    placeholder="#alerts"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Username
                  </label>
                  <input
                    type="text"
                    value={slackConfig.username}
                    onChange={(e) => setSlackConfig({ ...slackConfig, username: e.target.value })}
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                    placeholder="CADDY Bot"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Icon Emoji
                  </label>
                  <input
                    type="text"
                    value={slackConfig.icon_emoji}
                    onChange={(e) =>
                      setSlackConfig({ ...slackConfig, icon_emoji: e.target.value })
                    }
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                    placeholder=":robot_face:"
                  />
                </div>
              </div>

              <div className="flex gap-4">
                <button
                  onClick={() => handleSaveChannelConfig('slack', slackConfig)}
                  disabled={loading}
                  className="flex-1 px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition disabled:opacity-50"
                >
                  {loading ? 'Saving...' : 'Save Configuration'}
                </button>
                <button
                  onClick={() => handleTestChannel(NotificationChannel.Slack)}
                  className="px-6 py-3 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 transition"
                >
                  Test
                </button>
              </div>

              {testResults[NotificationChannel.Slack] && (
                <p className="text-sm text-gray-600">{testResults[NotificationChannel.Slack]}</p>
              )}
            </div>
          </div>
        )}

        {/* Teams Tab */}
        {activeTab === 'teams' && (
          <div className="bg-white p-6 rounded-lg shadow">
            <h2 className="text-xl font-bold mb-4">Microsoft Teams Configuration</h2>

            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Webhook URL
                </label>
                <input
                  type="url"
                  value={teamsConfig.webhook_url}
                  onChange={(e) => setTeamsConfig({ ...teamsConfig, webhook_url: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                  placeholder="https://outlook.office.com/webhook/..."
                />
                <p className="mt-1 text-xs text-gray-500">
                  Get this URL from Teams → Channel → Connectors → Incoming Webhook
                </p>
              </div>

              <div className="flex gap-4">
                <button
                  onClick={() => handleSaveChannelConfig('teams', teamsConfig)}
                  disabled={loading}
                  className="flex-1 px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition disabled:opacity-50"
                >
                  {loading ? 'Saving...' : 'Save Configuration'}
                </button>
                <button
                  onClick={() => handleTestChannel(NotificationChannel.MicrosoftTeams)}
                  className="px-6 py-3 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 transition"
                >
                  Test
                </button>
              </div>

              {testResults[NotificationChannel.MicrosoftTeams] && (
                <p className="text-sm text-gray-600">
                  {testResults[NotificationChannel.MicrosoftTeams]}
                </p>
              )}
            </div>
          </div>
        )}

        {/* Webhook Tab */}
        {activeTab === 'webhook' && (
          <div className="bg-white p-6 rounded-lg shadow">
            <h2 className="text-xl font-bold mb-4">Custom Webhook Configuration</h2>

            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">URL</label>
                <input
                  type="url"
                  value={webhookConfig.url}
                  onChange={(e) => setWebhookConfig({ ...webhookConfig, url: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                  placeholder="https://api.example.com/notifications"
                />
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    HTTP Method
                  </label>
                  <select
                    value={webhookConfig.method}
                    onChange={(e) => setWebhookConfig({ ...webhookConfig, method: e.target.value })}
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                  >
                    <option value="POST">POST</option>
                    <option value="PUT">PUT</option>
                  </select>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Timeout (seconds)
                  </label>
                  <input
                    type="number"
                    value={webhookConfig.timeout_seconds}
                    onChange={(e) =>
                      setWebhookConfig({
                        ...webhookConfig,
                        timeout_seconds: parseInt(e.target.value),
                      })
                    }
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                  />
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Custom Headers
                </label>
                <div className="flex gap-2 mb-2">
                  <input
                    type="text"
                    value={newHeaderKey}
                    onChange={(e) => setNewHeaderKey(e.target.value)}
                    placeholder="Header-Name"
                    className="flex-1 px-3 py-2 border border-gray-300 rounded-lg"
                  />
                  <input
                    type="text"
                    value={newHeaderValue}
                    onChange={(e) => setNewHeaderValue(e.target.value)}
                    placeholder="Header Value"
                    className="flex-1 px-3 py-2 border border-gray-300 rounded-lg"
                  />
                  <button
                    onClick={handleAddWebhookHeader}
                    className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
                  >
                    Add
                  </button>
                </div>
                <div className="space-y-1">
                  {Object.entries(webhookConfig.headers).map(([key, value]) => (
                    <div
                      key={key}
                      className="flex justify-between items-center p-2 bg-gray-50 rounded"
                    >
                      <span className="text-sm">
                        <strong>{key}:</strong> {value}
                      </span>
                      <button
                        onClick={() => handleRemoveWebhookHeader(key)}
                        className="text-red-600 hover:text-red-800"
                      >
                        ×
                      </button>
                    </div>
                  ))}
                </div>
              </div>

              <div className="flex gap-4">
                <button
                  onClick={() => handleSaveChannelConfig('webhook', webhookConfig)}
                  disabled={loading}
                  className="flex-1 px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition disabled:opacity-50"
                >
                  {loading ? 'Saving...' : 'Save Configuration'}
                </button>
                <button
                  onClick={() => handleTestChannel(NotificationChannel.Webhook)}
                  className="px-6 py-3 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 transition"
                >
                  Test
                </button>
              </div>

              {testResults[NotificationChannel.Webhook] && (
                <p className="text-sm text-gray-600">{testResults[NotificationChannel.Webhook]}</p>
              )}
            </div>
          </div>
        )}

        {/* History Tab */}
        {activeTab === 'history' && (
          <div className="bg-white rounded-lg shadow">
            <div className="p-6 border-b border-gray-200">
              <h2 className="text-xl font-bold">Notification History</h2>
              <p className="mt-1 text-sm text-gray-600">Recent notifications sent</p>
            </div>

            <div className="divide-y divide-gray-200">
              {notificationHistory.length === 0 ? (
                <div className="p-8 text-center text-gray-500">No notifications yet</div>
              ) : (
                notificationHistory.map((notification) => (
                  <div key={notification.id} className="p-4 hover:bg-gray-50">
                    <div className="flex justify-between items-start">
                      <div className="flex-1">
                        <div className="flex items-center gap-2 mb-1">
                          <span
                            className={`px-2 py-1 text-xs font-semibold rounded ${getSeverityColor(
                              notification.severity
                            )}`}
                          >
                            {notification.severity}
                          </span>
                          <h3 className="font-medium text-gray-900">{notification.title}</h3>
                        </div>
                        <p className="text-sm text-gray-700 mb-2">{notification.message}</p>
                        <div className="flex items-center gap-4 text-xs text-gray-500">
                          <span>Source: {notification.source}</span>
                          <span>{formatDate(notification.created_at)}</span>
                          <span>Attempts: {notification.delivery_attempts}</span>
                        </div>
                      </div>
                      <div className="text-right">
                        {notification.delivered ? (
                          <span className="text-green-600 text-sm">✓ Delivered</span>
                        ) : (
                          <span className="text-red-600 text-sm">✗ Failed</span>
                        )}
                      </div>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default NotificationSettings;
