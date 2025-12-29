/**
 * CADDY v0.4.0 - Report Distribution Component
 * $650M Platform - Production Ready
 *
 * Multi-channel report distribution with support for email, Slack, Teams,
 * webhooks, and cloud storage with encryption and compression options.
 */

import React, { useState, useCallback } from 'react';
import {
  ReportDistribution,
  DistributionConfig,
  DistributionChannel,
  EmailConfig,
  SlackConfig,
  TeamsConfig,
  WebhookConfig,
  StorageConfig,
  ExportFormat,
} from './types';

export interface ReportDistributionProps {
  distribution?: ReportDistribution;
  onChange: (distribution: ReportDistribution) => void;
  readOnly?: boolean;
}

export const ReportDistributionComponent: React.FC<ReportDistributionProps> = ({
  distribution,
  onChange,
  readOnly = false,
}) => {
  const [currentDistribution, setCurrentDistribution] = useState<ReportDistribution>(
    distribution || createDefaultDistribution()
  );

  const [selectedChannel, setSelectedChannel] = useState<DistributionChannel | null>(null);

  const updateDistribution = useCallback(
    (updates: Partial<ReportDistribution>) => {
      const updated = { ...currentDistribution, ...updates };
      setCurrentDistribution(updated);
      onChange(updated);
    },
    [currentDistribution, onChange]
  );

  const addChannel = useCallback(
    (type: DistributionChannel) => {
      const newChannel: DistributionConfig = {
        type,
        enabled: true,
        config: getDefaultChannelConfig(type),
      };

      updateDistribution({
        channels: [...currentDistribution.channels, newChannel],
      });

      setSelectedChannel(type);
    },
    [currentDistribution.channels, updateDistribution]
  );

  const updateChannel = useCallback(
    (index: number, updates: Partial<DistributionConfig>) => {
      const updatedChannels = currentDistribution.channels.map((channel, i) =>
        i === index ? { ...channel, ...updates } : channel
      );

      updateDistribution({ channels: updatedChannels });
    },
    [currentDistribution.channels, updateDistribution]
  );

  const removeChannel = useCallback(
    (index: number) => {
      const updatedChannels = currentDistribution.channels.filter((_, i) => i !== index);
      updateDistribution({ channels: updatedChannels });
      setSelectedChannel(null);
    },
    [currentDistribution.channels, updateDistribution]
  );

  const renderChannelConfig = (channel: DistributionConfig, index: number) => {
    switch (channel.type) {
      case 'email':
        return renderEmailConfig(channel.config as EmailConfig, index);
      case 'slack':
        return renderSlackConfig(channel.config as SlackConfig, index);
      case 'teams':
        return renderTeamsConfig(channel.config as TeamsConfig, index);
      case 'webhook':
        return renderWebhookConfig(channel.config as WebhookConfig, index);
      case 's3':
      case 'ftp':
      case 'sftp':
        return renderStorageConfig(channel.config as StorageConfig, index);
      default:
        return <div>Unknown channel type</div>;
    }
  };

  const renderEmailConfig = (config: EmailConfig, index: number) => (
    <div style={styles.configPanel}>
      <div style={styles.formGroup}>
        <label style={styles.label}>From Address</label>
        <input
          type="email"
          value={config.from}
          onChange={(e) =>
            updateChannel(index, {
              config: { ...config, from: e.target.value },
            })
          }
          style={styles.input}
          disabled={readOnly}
        />
      </div>

      <div style={styles.formGroup}>
        <label style={styles.label}>To Addresses (comma-separated)</label>
        <input
          type="text"
          value={config.to.join(', ')}
          onChange={(e) =>
            updateChannel(index, {
              config: {
                ...config,
                to: e.target.value.split(',').map((addr) => addr.trim()).filter(Boolean),
              },
            })
          }
          style={styles.input}
          disabled={readOnly}
        />
      </div>

      <div style={styles.formGroup}>
        <label style={styles.label}>CC (optional)</label>
        <input
          type="text"
          value={(config.cc || []).join(', ')}
          onChange={(e) =>
            updateChannel(index, {
              config: {
                ...config,
                cc: e.target.value.split(',').map((addr) => addr.trim()).filter(Boolean),
              },
            })
          }
          style={styles.input}
          disabled={readOnly}
        />
      </div>

      <div style={styles.formGroup}>
        <label style={styles.label}>Subject</label>
        <input
          type="text"
          value={config.subject}
          onChange={(e) =>
            updateChannel(index, {
              config: { ...config, subject: e.target.value },
            })
          }
          style={styles.input}
          placeholder="Report: {{reportName}} - {{date}}"
          disabled={readOnly}
        />
      </div>

      <div style={styles.formGroup}>
        <label style={styles.label}>Body</label>
        <textarea
          value={config.body}
          onChange={(e) =>
            updateChannel(index, {
              config: { ...config, body: e.target.value },
            })
          }
          style={styles.textarea}
          rows={4}
          disabled={readOnly}
        />
      </div>

      <div style={styles.formGroup}>
        <label style={styles.label}>Body Format</label>
        <select
          value={config.bodyFormat}
          onChange={(e) =>
            updateChannel(index, {
              config: { ...config, bodyFormat: e.target.value as 'text' | 'html' },
            })
          }
          style={styles.select}
          disabled={readOnly}
        >
          <option value="text">Plain Text</option>
          <option value="html">HTML</option>
        </select>
      </div>

      <div style={styles.formGroup}>
        <label style={styles.checkboxLabel}>
          <input
            type="checkbox"
            checked={config.attachReport}
            onChange={(e) =>
              updateChannel(index, {
                config: { ...config, attachReport: e.target.checked },
              })
            }
            disabled={readOnly}
          />
          <span>Attach report file</span>
        </label>
      </div>
    </div>
  );

  const renderSlackConfig = (config: SlackConfig, index: number) => (
    <div style={styles.configPanel}>
      <div style={styles.formGroup}>
        <label style={styles.label}>Webhook URL</label>
        <input
          type="url"
          value={config.webhookUrl}
          onChange={(e) =>
            updateChannel(index, {
              config: { ...config, webhookUrl: e.target.value },
            })
          }
          style={styles.input}
          placeholder="https://hooks.slack.com/services/..."
          disabled={readOnly}
        />
      </div>

      <div style={styles.formGroup}>
        <label style={styles.label}>Channel</label>
        <input
          type="text"
          value={config.channel}
          onChange={(e) =>
            updateChannel(index, {
              config: { ...config, channel: e.target.value },
            })
          }
          style={styles.input}
          placeholder="#reports"
          disabled={readOnly}
        />
      </div>

      <div style={styles.formGroup}>
        <label style={styles.label}>Username (optional)</label>
        <input
          type="text"
          value={config.username || ''}
          onChange={(e) =>
            updateChannel(index, {
              config: { ...config, username: e.target.value },
            })
          }
          style={styles.input}
          placeholder="Report Bot"
          disabled={readOnly}
        />
      </div>

      <div style={styles.formGroup}>
        <label style={styles.label}>Message</label>
        <textarea
          value={config.message}
          onChange={(e) =>
            updateChannel(index, {
              config: { ...config, message: e.target.value },
            })
          }
          style={styles.textarea}
          rows={4}
          disabled={readOnly}
        />
      </div>

      <div style={styles.formGroup}>
        <label style={styles.checkboxLabel}>
          <input
            type="checkbox"
            checked={config.attachReport}
            onChange={(e) =>
              updateChannel(index, {
                config: { ...config, attachReport: e.target.checked },
              })
            }
            disabled={readOnly}
          />
          <span>Attach report file</span>
        </label>
      </div>
    </div>
  );

  const renderTeamsConfig = (config: TeamsConfig, index: number) => (
    <div style={styles.configPanel}>
      <div style={styles.formGroup}>
        <label style={styles.label}>Webhook URL</label>
        <input
          type="url"
          value={config.webhookUrl}
          onChange={(e) =>
            updateChannel(index, {
              config: { ...config, webhookUrl: e.target.value },
            })
          }
          style={styles.input}
          disabled={readOnly}
        />
      </div>

      <div style={styles.formGroup}>
        <label style={styles.label}>Title</label>
        <input
          type="text"
          value={config.title}
          onChange={(e) =>
            updateChannel(index, {
              config: { ...config, title: e.target.value },
            })
          }
          style={styles.input}
          disabled={readOnly}
        />
      </div>

      <div style={styles.formGroup}>
        <label style={styles.label}>Message</label>
        <textarea
          value={config.message}
          onChange={(e) =>
            updateChannel(index, {
              config: { ...config, message: e.target.value },
            })
          }
          style={styles.textarea}
          rows={4}
          disabled={readOnly}
        />
      </div>

      <div style={styles.formGroup}>
        <label style={styles.label}>Theme Color</label>
        <input
          type="color"
          value={config.themeColor || '#0078D4'}
          onChange={(e) =>
            updateChannel(index, {
              config: { ...config, themeColor: e.target.value },
            })
          }
          style={styles.colorInput}
          disabled={readOnly}
        />
      </div>

      <div style={styles.formGroup}>
        <label style={styles.checkboxLabel}>
          <input
            type="checkbox"
            checked={config.attachReport}
            onChange={(e) =>
              updateChannel(index, {
                config: { ...config, attachReport: e.target.checked },
              })
            }
            disabled={readOnly}
          />
          <span>Attach report file</span>
        </label>
      </div>
    </div>
  );

  const renderWebhookConfig = (config: WebhookConfig, index: number) => (
    <div style={styles.configPanel}>
      <div style={styles.formGroup}>
        <label style={styles.label}>URL</label>
        <input
          type="url"
          value={config.url}
          onChange={(e) =>
            updateChannel(index, {
              config: { ...config, url: e.target.value },
            })
          }
          style={styles.input}
          disabled={readOnly}
        />
      </div>

      <div style={styles.formGroup}>
        <label style={styles.label}>HTTP Method</label>
        <select
          value={config.method}
          onChange={(e) =>
            updateChannel(index, {
              config: { ...config, method: e.target.value as 'GET' | 'POST' | 'PUT' },
            })
          }
          style={styles.select}
          disabled={readOnly}
        >
          <option value="GET">GET</option>
          <option value="POST">POST</option>
          <option value="PUT">PUT</option>
        </select>
      </div>

      <div style={styles.formGroup}>
        <label style={styles.label}>Headers (JSON)</label>
        <textarea
          value={JSON.stringify(config.headers || {}, null, 2)}
          onChange={(e) => {
            try {
              const headers = JSON.parse(e.target.value);
              updateChannel(index, {
                config: { ...config, headers },
              });
            } catch (err) {
              // Invalid JSON, ignore
            }
          }}
          style={styles.textarea}
          rows={4}
          disabled={readOnly}
        />
      </div>

      <div style={styles.formGroup}>
        <label style={styles.checkboxLabel}>
          <input
            type="checkbox"
            checked={config.includeReportData}
            onChange={(e) =>
              updateChannel(index, {
                config: { ...config, includeReportData: e.target.checked },
              })
            }
            disabled={readOnly}
          />
          <span>Include report data in payload</span>
        </label>
      </div>
    </div>
  );

  const renderStorageConfig = (config: StorageConfig, index: number) => (
    <div style={styles.configPanel}>
      {config.type === 's3' && (
        <div style={styles.formGroup}>
          <label style={styles.label}>S3 Bucket</label>
          <input
            type="text"
            value={config.bucket || ''}
            onChange={(e) =>
              updateChannel(index, {
                config: { ...config, bucket: e.target.value },
              })
            }
            style={styles.input}
            disabled={readOnly}
          />
        </div>
      )}

      <div style={styles.formGroup}>
        <label style={styles.label}>Path</label>
        <input
          type="text"
          value={config.path}
          onChange={(e) =>
            updateChannel(index, {
              config: { ...config, path: e.target.value },
            })
          }
          style={styles.input}
          placeholder="/reports/{{reportName}}/{{date}}.pdf"
          disabled={readOnly}
        />
      </div>

      <div style={styles.formGroup}>
        <label style={styles.label}>Credentials (JSON)</label>
        <textarea
          value={JSON.stringify(config.credentials || {}, null, 2)}
          onChange={(e) => {
            try {
              const credentials = JSON.parse(e.target.value);
              updateChannel(index, {
                config: { ...config, credentials },
              });
            } catch (err) {
              // Invalid JSON, ignore
            }
          }}
          style={styles.textarea}
          rows={4}
          disabled={readOnly}
        />
      </div>

      {config.type === 's3' && (
        <div style={styles.formGroup}>
          <label style={styles.checkboxLabel}>
            <input
              type="checkbox"
              checked={config.publicAccess ?? false}
              onChange={(e) =>
                updateChannel(index, {
                  config: { ...config, publicAccess: e.target.checked },
                })
              }
              disabled={readOnly}
            />
            <span>Public access</span>
          </label>
        </div>
      )}
    </div>
  );

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h3 style={styles.title}>Distribution Settings</h3>
      </div>

      <div style={styles.content}>
        {/* Global Settings */}
        <div style={styles.section}>
          <h4 style={styles.sectionTitle}>Global Settings</h4>

          <div style={styles.formGroup}>
            <label style={styles.label}>Attachment Format</label>
            <select
              value={currentDistribution.attachmentFormat || 'pdf'}
              onChange={(e) =>
                updateDistribution({ attachmentFormat: e.target.value as ExportFormat })
              }
              style={styles.select}
              disabled={readOnly}
            >
              <option value="pdf">PDF</option>
              <option value="excel">Excel</option>
              <option value="csv">CSV</option>
              <option value="powerpoint">PowerPoint</option>
            </select>
          </div>

          <div style={styles.formGroup}>
            <label style={styles.checkboxLabel}>
              <input
                type="checkbox"
                checked={currentDistribution.compression ?? false}
                onChange={(e) => updateDistribution({ compression: e.target.checked })}
                disabled={readOnly}
              />
              <span>Compress attachments (ZIP)</span>
            </label>
          </div>

          <div style={styles.formGroup}>
            <label style={styles.checkboxLabel}>
              <input
                type="checkbox"
                checked={currentDistribution.encryption?.enabled ?? false}
                onChange={(e) =>
                  updateDistribution({
                    encryption: {
                      ...currentDistribution.encryption!,
                      enabled: e.target.checked,
                      algorithm: 'aes-256',
                    },
                  })
                }
                disabled={readOnly}
              />
              <span>Encrypt attachments</span>
            </label>
          </div>

          {currentDistribution.encryption?.enabled && (
            <>
              <div style={styles.formGroup}>
                <label style={styles.label}>Encryption Algorithm</label>
                <select
                  value={currentDistribution.encryption.algorithm}
                  onChange={(e) =>
                    updateDistribution({
                      encryption: {
                        ...currentDistribution.encryption!,
                        algorithm: e.target.value as 'aes-256' | 'rsa-2048',
                      },
                    })
                  }
                  style={styles.select}
                  disabled={readOnly}
                >
                  <option value="aes-256">AES-256</option>
                  <option value="rsa-2048">RSA-2048</option>
                </select>
              </div>

              <div style={styles.formGroup}>
                <label style={styles.label}>Encryption Password</label>
                <input
                  type="password"
                  value={currentDistribution.encryption.password || ''}
                  onChange={(e) =>
                    updateDistribution({
                      encryption: {
                        ...currentDistribution.encryption!,
                        password: e.target.value,
                      },
                    })
                  }
                  style={styles.input}
                  disabled={readOnly}
                />
              </div>
            </>
          )}
        </div>

        {/* Distribution Channels */}
        <div style={styles.section}>
          <div style={styles.sectionHeader}>
            <h4 style={styles.sectionTitle}>Distribution Channels</h4>
            {!readOnly && (
              <div style={styles.addChannelDropdown}>
                <button style={styles.addButton}>+ Add Channel</button>
                <div style={styles.channelMenu}>
                  <button onClick={() => addChannel('email')}>📧 Email</button>
                  <button onClick={() => addChannel('slack')}>💬 Slack</button>
                  <button onClick={() => addChannel('teams')}>👥 Teams</button>
                  <button onClick={() => addChannel('webhook')}>🔗 Webhook</button>
                  <button onClick={() => addChannel('s3')}>☁️ S3</button>
                  <button onClick={() => addChannel('ftp')}>📁 FTP</button>
                  <button onClick={() => addChannel('sftp')}>🔐 SFTP</button>
                </div>
              </div>
            )}
          </div>

          <div style={styles.channelsList}>
            {currentDistribution.channels.map((channel, index) => (
              <div key={index} style={styles.channelItem}>
                <div style={styles.channelHeader}>
                  <div style={styles.channelHeaderLeft}>
                    <span style={styles.channelIcon}>{getChannelIcon(channel.type)}</span>
                    <span style={styles.channelName}>{channel.type.toUpperCase()}</span>
                    <label style={styles.channelToggle}>
                      <input
                        type="checkbox"
                        checked={channel.enabled}
                        onChange={(e) =>
                          updateChannel(index, { enabled: e.target.checked })
                        }
                        disabled={readOnly}
                      />
                      <span>Enabled</span>
                    </label>
                  </div>
                  {!readOnly && (
                    <button
                      onClick={() => removeChannel(index)}
                      style={styles.removeButton}
                    >
                      ✕
                    </button>
                  )}
                </div>

                {channel.enabled && renderChannelConfig(channel, index)}
              </div>
            ))}

            {currentDistribution.channels.length === 0 && (
              <div style={styles.emptyState}>
                <div style={styles.emptyStateIcon}>📬</div>
                <div style={styles.emptyStateText}>No distribution channels configured</div>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

// Helper functions
function createDefaultDistribution(): ReportDistribution {
  return {
    channels: [],
    attachmentFormat: 'pdf',
    compression: false,
    encryption: {
      enabled: false,
      algorithm: 'aes-256',
    },
  };
}

function getDefaultChannelConfig(type: DistributionChannel): any {
  switch (type) {
    case 'email':
      return {
        from: '',
        to: [],
        subject: 'Report: {{reportName}} - {{date}}',
        body: 'Please find the attached report.',
        bodyFormat: 'text',
        attachReport: true,
      } as EmailConfig;

    case 'slack':
      return {
        webhookUrl: '',
        channel: '#reports',
        message: 'New report available: {{reportName}}',
        attachReport: true,
      } as SlackConfig;

    case 'teams':
      return {
        webhookUrl: '',
        title: 'Report: {{reportName}}',
        message: 'A new report is ready for review.',
        themeColor: '#0078D4',
        attachReport: true,
      } as TeamsConfig;

    case 'webhook':
      return {
        url: '',
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        includeReportData: true,
      } as WebhookConfig;

    case 's3':
    case 'ftp':
    case 'sftp':
      return {
        type,
        path: '/reports/{{reportName}}/{{date}}',
        credentials: {},
      } as StorageConfig;

    default:
      return {};
  }
}

function getChannelIcon(type: DistributionChannel): string {
  const icons: Record<DistributionChannel, string> = {
    email: '📧',
    slack: '💬',
    teams: '👥',
    webhook: '🔗',
    s3: '☁️',
    ftp: '📁',
    sftp: '🔐',
  };
  return icons[type] || '📤';
}

// Styles
const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    height: '100%',
    backgroundColor: '#ffffff',
    border: '1px solid #e2e8f0',
    borderRadius: '8px',
    fontFamily: 'Inter, system-ui, sans-serif',
    overflow: 'hidden',
  },
  header: {
    padding: '12px 16px',
    borderBottom: '1px solid #e2e8f0',
    backgroundColor: '#f8fafc',
  },
  title: {
    fontSize: '14px',
    fontWeight: 600,
    margin: 0,
    color: '#1e293b',
  },
  content: {
    flex: 1,
    overflow: 'auto',
    padding: '16px',
  },
  section: {
    marginBottom: '24px',
    paddingBottom: '24px',
    borderBottom: '1px solid #e2e8f0',
  },
  sectionHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '12px',
  },
  sectionTitle: {
    fontSize: '13px',
    fontWeight: 600,
    margin: 0,
    color: '#1e293b',
  },
  formGroup: {
    marginBottom: '12px',
  },
  label: {
    display: 'block',
    fontSize: '12px',
    fontWeight: 500,
    color: '#475569',
    marginBottom: '4px',
  },
  input: {
    width: '100%',
    padding: '6px 8px',
    border: '1px solid #e2e8f0',
    borderRadius: '4px',
    fontSize: '13px',
  },
  select: {
    width: '100%',
    padding: '6px 8px',
    border: '1px solid #e2e8f0',
    borderRadius: '4px',
    fontSize: '13px',
    cursor: 'pointer',
  },
  textarea: {
    width: '100%',
    padding: '6px 8px',
    border: '1px solid #e2e8f0',
    borderRadius: '4px',
    fontSize: '12px',
    fontFamily: 'monospace',
    resize: 'vertical',
  },
  checkboxLabel: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    fontSize: '13px',
    color: '#475569',
    cursor: 'pointer',
  },
  colorInput: {
    width: '60px',
    height: '40px',
    border: '1px solid #e2e8f0',
    borderRadius: '4px',
    cursor: 'pointer',
  },
  addChannelDropdown: {
    position: 'relative',
  },
  addButton: {
    padding: '6px 12px',
    border: 'none',
    borderRadius: '6px',
    backgroundColor: '#2563eb',
    color: '#ffffff',
    cursor: 'pointer',
    fontSize: '13px',
    fontWeight: 500,
  },
  channelMenu: {
    display: 'none',
    position: 'absolute',
    top: '100%',
    right: 0,
    marginTop: '4px',
    backgroundColor: '#ffffff',
    border: '1px solid #e2e8f0',
    borderRadius: '6px',
    boxShadow: '0 4px 6px rgba(0, 0, 0, 0.1)',
    zIndex: 10,
  },
  channelsList: {
    display: 'flex',
    flexDirection: 'column',
    gap: '12px',
  },
  channelItem: {
    border: '1px solid #e2e8f0',
    borderRadius: '8px',
    overflow: 'hidden',
  },
  channelHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '12px',
    backgroundColor: '#f8fafc',
    borderBottom: '1px solid #e2e8f0',
  },
  channelHeaderLeft: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
  },
  channelIcon: {
    fontSize: '20px',
  },
  channelName: {
    fontSize: '13px',
    fontWeight: 600,
    color: '#1e293b',
  },
  channelToggle: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    fontSize: '12px',
    color: '#64748b',
    cursor: 'pointer',
  },
  removeButton: {
    border: 'none',
    background: 'none',
    color: '#ef4444',
    cursor: 'pointer',
    fontSize: '18px',
  },
  configPanel: {
    padding: '16px',
  },
  emptyState: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '48px',
    gap: '12px',
  },
  emptyStateIcon: {
    fontSize: '32px',
  },
  emptyStateText: {
    fontSize: '13px',
    color: '#64748b',
  },
};

export default ReportDistributionComponent;
