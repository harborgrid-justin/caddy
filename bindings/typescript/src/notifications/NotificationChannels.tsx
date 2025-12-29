/**
 * CADDY v0.4.0 - Notification Channels Configuration
 * Multi-channel delivery configuration (Email, SMS, Push, Slack, Teams, Webhook)
 */

import React, { useState, useCallback, useEffect } from 'react';
import { NotificationChannel, NotificationChannelConfig } from './types';

interface NotificationChannelsProps {
  tenantId: string;
  apiUrl?: string;
}

export const NotificationChannels: React.FC<NotificationChannelsProps> = ({
  tenantId,
  apiUrl = '/api/notifications/channels'
}) => {
  const [configs, setConfigs] = useState<NotificationChannelConfig[]>([]);
  const [loading, setLoading] = useState(false);
  const [editingConfig, setEditingConfig] = useState<Partial<NotificationChannelConfig> | null>(null);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [showSecrets, setShowSecrets] = useState<Record<string, boolean>>({});

  const fetchConfigs = useCallback(async () => {
    setLoading(true);
    try {
      const response = await fetch(`${apiUrl}?tenantId=${tenantId}`, {
        credentials: 'include'
      });
      const data = await response.json();
      setConfigs(data.configs || []);
    } catch (err) {
      console.error('Error fetching channel configs:', err);
    } finally {
      setLoading(false);
    }
  }, [apiUrl, tenantId]);

  useEffect(() => {
    fetchConfigs();
  }, [fetchConfigs]);

  const handleEdit = useCallback((config: NotificationChannelConfig) => {
    setEditingConfig(config);
    setIsModalOpen(true);
  }, []);

  const handleCreate = useCallback((channel: NotificationChannel) => {
    setEditingConfig({
      channel,
      enabled: true,
      config: {},
      rateLimit: {
        maxPerMinute: 60,
        maxPerHour: 1000,
        maxPerDay: 10000
      },
      retryPolicy: {
        maxAttempts: 3,
        backoffMultiplier: 2,
        initialDelay: 1000,
        maxDelay: 60000
      }
    });
    setIsModalOpen(true);
  }, []);

  const handleSave = useCallback(async () => {
    if (!editingConfig) return;

    try {
      const method = editingConfig.id ? 'PUT' : 'POST';
      const url = editingConfig.id ? `${apiUrl}/${editingConfig.id}` : apiUrl;

      const response = await fetch(url, {
        method,
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ ...editingConfig, tenantId })
      });

      if (response.ok) {
        await fetchConfigs();
        setIsModalOpen(false);
        setEditingConfig(null);
      }
    } catch (err) {
      console.error('Error saving config:', err);
      alert('Failed to save configuration');
    }
  }, [editingConfig, apiUrl, tenantId, fetchConfigs]);

  const handleToggleEnabled = useCallback(async (config: NotificationChannelConfig) => {
    try {
      await fetch(`${apiUrl}/${config.id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ ...config, enabled: !config.enabled })
      });
      await fetchConfigs();
    } catch (err) {
      console.error('Error toggling config:', err);
    }
  }, [apiUrl, fetchConfigs]);

  const getChannelIcon = (channel: NotificationChannel): string => {
    const icons: Record<NotificationChannel, string> = {
      [NotificationChannel.IN_APP]: '🔔',
      [NotificationChannel.EMAIL]: '📧',
      [NotificationChannel.SMS]: '💬',
      [NotificationChannel.PUSH]: '📱',
      [NotificationChannel.SLACK]: '💼',
      [NotificationChannel.TEAMS]: '👥',
      [NotificationChannel.WEBHOOK]: '🔗'
    };
    return icons[channel];
  };

  const availableChannels = Object.values(NotificationChannel).filter(
    channel => !configs.some(c => c.channel === channel)
  );

  const renderConfigForm = () => {
    if (!editingConfig) return null;

    const channel = editingConfig.channel!;
    const config = editingConfig.config || {};

    return (
      <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
        {/* Email Configuration */}
        {channel === NotificationChannel.EMAIL && (
          <>
            <div>
              <label style={{ display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
                SMTP Host *
              </label>
              <input
                type="text"
                value={config.smtpHost || ''}
                onChange={(e) => setEditingConfig({
                  ...editingConfig,
                  config: { ...config, smtpHost: e.target.value }
                })}
                placeholder="smtp.gmail.com"
                style={{ width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' }}
              />
            </div>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px' }}>
              <div>
                <label style={{ display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
                  Port *
                </label>
                <input
                  type="number"
                  value={config.smtpPort || 587}
                  onChange={(e) => setEditingConfig({
                    ...editingConfig,
                    config: { ...config, smtpPort: parseInt(e.target.value) }
                  })}
                  style={{ width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' }}
                />
              </div>
              <div style={{ display: 'flex', alignItems: 'flex-end' }}>
                <label style={{ display: 'flex', alignItems: 'center', gap: '8px', padding: '8px 0' }}>
                  <input
                    type="checkbox"
                    checked={config.smtpSecure || false}
                    onChange={(e) => setEditingConfig({
                      ...editingConfig,
                      config: { ...config, smtpSecure: e.target.checked }
                    })}
                  />
                  <span style={{ fontSize: '13px', color: '#374151' }}>Use TLS/SSL</span>
                </label>
              </div>
            </div>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px' }}>
              <div>
                <label style={{ display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
                  Username *
                </label>
                <input
                  type="text"
                  value={config.smtpUser || ''}
                  onChange={(e) => setEditingConfig({
                    ...editingConfig,
                    config: { ...config, smtpUser: e.target.value }
                  })}
                  style={{ width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' }}
                />
              </div>
              <div>
                <label style={{ display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
                  Password *
                </label>
                <input
                  type={showSecrets[channel] ? 'text' : 'password'}
                  value={config.smtpPassword || ''}
                  onChange={(e) => setEditingConfig({
                    ...editingConfig,
                    config: { ...config, smtpPassword: e.target.value }
                  })}
                  style={{ width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' }}
                />
              </div>
            </div>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px' }}>
              <div>
                <label style={{ display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
                  From Email *
                </label>
                <input
                  type="email"
                  value={config.fromEmail || ''}
                  onChange={(e) => setEditingConfig({
                    ...editingConfig,
                    config: { ...config, fromEmail: e.target.value }
                  })}
                  placeholder="noreply@example.com"
                  style={{ width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' }}
                />
              </div>
              <div>
                <label style={{ display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
                  From Name
                </label>
                <input
                  type="text"
                  value={config.fromName || ''}
                  onChange={(e) => setEditingConfig({
                    ...editingConfig,
                    config: { ...config, fromName: e.target.value }
                  })}
                  placeholder="CADDY Notifications"
                  style={{ width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' }}
                />
              </div>
            </div>
          </>
        )}

        {/* SMS Configuration */}
        {channel === NotificationChannel.SMS && (
          <>
            <div>
              <label style={{ display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
                SMS Provider *
              </label>
              <select
                value={config.smsProvider || 'twilio'}
                onChange={(e) => setEditingConfig({
                  ...editingConfig,
                  config: { ...config, smsProvider: e.target.value as any }
                })}
                style={{ width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' }}
              >
                <option value="twilio">Twilio</option>
                <option value="nexmo">Nexmo</option>
                <option value="aws-sns">AWS SNS</option>
              </select>
            </div>
            <div>
              <label style={{ display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
                API Key *
              </label>
              <input
                type={showSecrets[channel] ? 'text' : 'password'}
                value={config.smsApiKey || ''}
                onChange={(e) => setEditingConfig({
                  ...editingConfig,
                  config: { ...config, smsApiKey: e.target.value }
                })}
                style={{ width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' }}
              />
            </div>
            <div>
              <label style={{ display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
                API Secret *
              </label>
              <input
                type={showSecrets[channel] ? 'text' : 'password'}
                value={config.smsApiSecret || ''}
                onChange={(e) => setEditingConfig({
                  ...editingConfig,
                  config: { ...config, smsApiSecret: e.target.value }
                })}
                style={{ width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' }}
              />
            </div>
            <div>
              <label style={{ display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
                From Number *
              </label>
              <input
                type="tel"
                value={config.smsFromNumber || ''}
                onChange={(e) => setEditingConfig({
                  ...editingConfig,
                  config: { ...config, smsFromNumber: e.target.value }
                })}
                placeholder="+1234567890"
                style={{ width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' }}
              />
            </div>
          </>
        )}

        {/* Push Configuration */}
        {channel === NotificationChannel.PUSH && (
          <>
            <div>
              <label style={{ display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
                Push Provider *
              </label>
              <select
                value={config.pushProvider || 'fcm'}
                onChange={(e) => setEditingConfig({
                  ...editingConfig,
                  config: { ...config, pushProvider: e.target.value as any }
                })}
                style={{ width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' }}
              >
                <option value="fcm">Firebase Cloud Messaging</option>
                <option value="apns">Apple Push Notification Service</option>
                <option value="onesignal">OneSignal</option>
              </select>
            </div>
            <div>
              <label style={{ display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
                API Key *
              </label>
              <input
                type={showSecrets[channel] ? 'text' : 'password'}
                value={config.pushApiKey || ''}
                onChange={(e) => setEditingConfig({
                  ...editingConfig,
                  config: { ...config, pushApiKey: e.target.value }
                })}
                style={{ width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' }}
              />
            </div>
            <div>
              <label style={{ display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
                App ID
              </label>
              <input
                type="text"
                value={config.pushAppId || ''}
                onChange={(e) => setEditingConfig({
                  ...editingConfig,
                  config: { ...config, pushAppId: e.target.value }
                })}
                style={{ width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' }}
              />
            </div>
          </>
        )}

        {/* Slack Configuration */}
        {channel === NotificationChannel.SLACK && (
          <>
            <div>
              <label style={{ display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
                Webhook URL *
              </label>
              <input
                type={showSecrets[channel] ? 'text' : 'password'}
                value={config.slackWebhookUrl || ''}
                onChange={(e) => setEditingConfig({
                  ...editingConfig,
                  config: { ...config, slackWebhookUrl: e.target.value }
                })}
                placeholder="https://hooks.slack.com/services/..."
                style={{ width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' }}
              />
            </div>
            <div>
              <label style={{ display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
                Bot Token (Optional)
              </label>
              <input
                type={showSecrets[channel] ? 'text' : 'password'}
                value={config.slackBotToken || ''}
                onChange={(e) => setEditingConfig({
                  ...editingConfig,
                  config: { ...config, slackBotToken: e.target.value }
                })}
                placeholder="xoxb-..."
                style={{ width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' }}
              />
            </div>
            <div>
              <label style={{ display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
                Default Channel
              </label>
              <input
                type="text"
                value={config.slackChannel || ''}
                onChange={(e) => setEditingConfig({
                  ...editingConfig,
                  config: { ...config, slackChannel: e.target.value }
                })}
                placeholder="#notifications"
                style={{ width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' }}
              />
            </div>
          </>
        )}

        {/* Teams Configuration */}
        {channel === NotificationChannel.TEAMS && (
          <div>
            <label style={{ display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
              Webhook URL *
            </label>
            <input
              type={showSecrets[channel] ? 'text' : 'password'}
              value={config.teamsWebhookUrl || ''}
              onChange={(e) => setEditingConfig({
                ...editingConfig,
                config: { ...config, teamsWebhookUrl: e.target.value }
              })}
              placeholder="https://outlook.office.com/webhook/..."
              style={{ width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' }}
            />
          </div>
        )}

        {/* Webhook Configuration */}
        {channel === NotificationChannel.WEBHOOK && (
          <>
            <div>
              <label style={{ display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
                Webhook URL *
              </label>
              <input
                type="text"
                value={config.webhookUrl || ''}
                onChange={(e) => setEditingConfig({
                  ...editingConfig,
                  config: { ...config, webhookUrl: e.target.value }
                })}
                placeholder="https://api.example.com/webhook"
                style={{ width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' }}
              />
            </div>
            <div>
              <label style={{ display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
                HTTP Method
              </label>
              <select
                value={config.webhookMethod || 'POST'}
                onChange={(e) => setEditingConfig({
                  ...editingConfig,
                  config: { ...config, webhookMethod: e.target.value as any }
                })}
                style={{ width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' }}
              >
                <option value="POST">POST</option>
                <option value="PUT">PUT</option>
                <option value="PATCH">PATCH</option>
              </select>
            </div>
          </>
        )}

        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <input
            type="checkbox"
            checked={showSecrets[channel] || false}
            onChange={(e) => setShowSecrets({ ...showSecrets, [channel]: e.target.checked })}
          />
          <span style={{ fontSize: '13px', color: '#6b7280' }}>Show sensitive values</span>
        </div>

        {/* Rate Limits */}
        <div style={{ paddingTop: '16px', borderTop: '1px solid #e5e7eb' }}>
          <h4 style={{ margin: '0 0 12px 0', fontSize: '14px', fontWeight: '600', color: '#111827' }}>
            Rate Limits
          </h4>
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: '12px' }}>
            <div>
              <label style={{ display: 'block', fontSize: '12px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
                Per Minute
              </label>
              <input
                type="number"
                value={editingConfig.rateLimit?.maxPerMinute || 60}
                onChange={(e) => setEditingConfig({
                  ...editingConfig,
                  rateLimit: { ...editingConfig.rateLimit!, maxPerMinute: parseInt(e.target.value) }
                })}
                style={{ width: '100%', padding: '6px 8px', fontSize: '13px', border: '1px solid #d1d5db', borderRadius: '4px' }}
              />
            </div>
            <div>
              <label style={{ display: 'block', fontSize: '12px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
                Per Hour
              </label>
              <input
                type="number"
                value={editingConfig.rateLimit?.maxPerHour || 1000}
                onChange={(e) => setEditingConfig({
                  ...editingConfig,
                  rateLimit: { ...editingConfig.rateLimit!, maxPerHour: parseInt(e.target.value) }
                })}
                style={{ width: '100%', padding: '6px 8px', fontSize: '13px', border: '1px solid #d1d5db', borderRadius: '4px' }}
              />
            </div>
            <div>
              <label style={{ display: 'block', fontSize: '12px', fontWeight: '500', color: '#374151', marginBottom: '4px' }}>
                Per Day
              </label>
              <input
                type="number"
                value={editingConfig.rateLimit?.maxPerDay || 10000}
                onChange={(e) => setEditingConfig({
                  ...editingConfig,
                  rateLimit: { ...editingConfig.rateLimit!, maxPerDay: parseInt(e.target.value) }
                })}
                style={{ width: '100%', padding: '6px 8px', fontSize: '13px', border: '1px solid #d1d5db', borderRadius: '4px' }}
              />
            </div>
          </div>
        </div>
      </div>
    );
  };

  return (
    <div style={{ padding: '24px', maxWidth: '1200px', margin: '0 auto' }}>
      <div style={{ marginBottom: '24px' }}>
        <h2 style={{ margin: '0 0 4px 0', fontSize: '20px', fontWeight: '600', color: '#111827' }}>
          Notification Channels
        </h2>
        <p style={{ margin: 0, fontSize: '14px', color: '#6b7280' }}>
          Configure multi-channel notification delivery
        </p>
      </div>

      {/* Configured Channels */}
      <div style={{ display: 'grid', gap: '16px', marginBottom: '32px' }}>
        {configs.map((config) => (
          <div
            key={config.id}
            style={{
              padding: '16px',
              border: '1px solid #e5e7eb',
              borderRadius: '8px',
              backgroundColor: config.enabled ? '#ffffff' : '#f9fafb'
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                <div style={{ fontSize: '32px' }}>{getChannelIcon(config.channel)}</div>
                <div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <h3 style={{ margin: 0, fontSize: '16px', fontWeight: '600', color: '#111827', textTransform: 'capitalize' }}>
                      {config.channel.replace('_', ' ')}
                    </h3>
                    <span
                      style={{
                        padding: '2px 8px',
                        fontSize: '11px',
                        fontWeight: '500',
                        borderRadius: '12px',
                        backgroundColor: config.enabled ? '#dcfce7' : '#fee2e2',
                        color: config.enabled ? '#166534' : '#991b1b'
                      }}
                    >
                      {config.enabled ? 'Enabled' : 'Disabled'}
                    </span>
                  </div>
                  <div style={{ marginTop: '4px', fontSize: '12px', color: '#6b7280' }}>
                    Rate limits: {config.rateLimit?.maxPerMinute}/min, {config.rateLimit?.maxPerHour}/hr, {config.rateLimit?.maxPerDay}/day
                  </div>
                </div>
              </div>
              <div style={{ display: 'flex', gap: '8px' }}>
                <button
                  onClick={() => handleToggleEnabled(config)}
                  style={{
                    padding: '6px 12px',
                    fontSize: '12px',
                    fontWeight: '500',
                    border: '1px solid #d1d5db',
                    borderRadius: '4px',
                    backgroundColor: '#ffffff',
                    color: '#374151',
                    cursor: 'pointer'
                  }}
                >
                  {config.enabled ? 'Disable' : 'Enable'}
                </button>
                <button
                  onClick={() => handleEdit(config)}
                  style={{
                    padding: '6px 12px',
                    fontSize: '12px',
                    fontWeight: '500',
                    border: '1px solid #d1d5db',
                    borderRadius: '4px',
                    backgroundColor: '#ffffff',
                    color: '#374151',
                    cursor: 'pointer'
                  }}
                >
                  Configure
                </button>
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Available Channels */}
      {availableChannels.length > 0 && (
        <div>
          <h3 style={{ margin: '0 0 12px 0', fontSize: '16px', fontWeight: '600', color: '#111827' }}>
            Add Channel
          </h3>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(150px, 1fr))', gap: '12px' }}>
            {availableChannels.map((channel) => (
              <button
                key={channel}
                onClick={() => handleCreate(channel)}
                style={{
                  padding: '16px',
                  border: '2px dashed #d1d5db',
                  borderRadius: '8px',
                  backgroundColor: '#ffffff',
                  cursor: 'pointer',
                  transition: 'all 0.2s',
                  textAlign: 'center'
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.borderColor = '#3b82f6';
                  e.currentTarget.style.backgroundColor = '#f0f9ff';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.borderColor = '#d1d5db';
                  e.currentTarget.style.backgroundColor = '#ffffff';
                }}
              >
                <div style={{ fontSize: '32px', marginBottom: '8px' }}>{getChannelIcon(channel)}</div>
                <div style={{ fontSize: '13px', fontWeight: '500', color: '#374151', textTransform: 'capitalize' }}>
                  {channel.replace('_', ' ')}
                </div>
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Edit Modal */}
      {isModalOpen && editingConfig && (
        <div
          style={{
            position: 'fixed',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            backgroundColor: 'rgba(0, 0, 0, 0.5)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 10000
          }}
          onClick={() => setIsModalOpen(false)}
        >
          <div
            onClick={(e) => e.stopPropagation()}
            style={{
              backgroundColor: '#ffffff',
              borderRadius: '8px',
              padding: '24px',
              maxWidth: '600px',
              width: '90%',
              maxHeight: '80vh',
              overflowY: 'auto',
              boxShadow: '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)'
            }}
          >
            <h3 style={{ margin: '0 0 24px 0', fontSize: '20px', fontWeight: '600', color: '#111827' }}>
              Configure {editingConfig.channel?.replace('_', ' ')} Channel
            </h3>

            {renderConfigForm()}

            <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end', marginTop: '24px' }}>
              <button
                onClick={() => setIsModalOpen(false)}
                style={{
                  padding: '10px 20px',
                  fontSize: '14px',
                  fontWeight: '500',
                  border: '1px solid #d1d5db',
                  borderRadius: '6px',
                  backgroundColor: '#ffffff',
                  color: '#374151',
                  cursor: 'pointer'
                }}
              >
                Cancel
              </button>
              <button
                onClick={handleSave}
                style={{
                  padding: '10px 20px',
                  fontSize: '14px',
                  fontWeight: '500',
                  border: 'none',
                  borderRadius: '6px',
                  backgroundColor: '#3b82f6',
                  color: '#ffffff',
                  cursor: 'pointer'
                }}
              >
                Save Configuration
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
