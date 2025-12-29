/**
 * CADDY v0.4.0 - Report Scheduler Component
 * $650M Platform - Production Ready
 *
 * Advanced report scheduling with cron expressions, retry policies,
 * conditional execution, and notification management.
 */

import React, { useState, useCallback } from 'react';
import {
  ReportSchedule,
  ScheduleFrequency,
} from './types';

export interface ReportSchedulerProps {
  schedule?: ReportSchedule;
  onChange: (schedule: ReportSchedule) => void;
  readOnly?: boolean;
}

export const ReportScheduler: React.FC<ReportSchedulerProps> = ({
  schedule,
  onChange,
  readOnly = false,
}) => {
  const [currentSchedule, setCurrentSchedule] = useState<ReportSchedule>(
    schedule || createDefaultSchedule()
  );

  const updateSchedule = useCallback(
    (updates: Partial<ReportSchedule>) => {
      const updated = { ...currentSchedule, ...updates };
      setCurrentSchedule(updated);
      onChange(updated);
    },
    [currentSchedule, onChange]
  );

  const generateCronExpression = (frequency: ScheduleFrequency): string => {
    switch (frequency) {
      case 'hourly':
        return '0 * * * *';
      case 'daily':
        return '0 0 * * *';
      case 'weekly':
        return '0 0 * * 0';
      case 'monthly':
        return '0 0 1 * *';
      case 'quarterly':
        return '0 0 1 */3 *';
      case 'yearly':
        return '0 0 1 1 *';
      default:
        return '';
    }
  };

  const handleFrequencyChange = (frequency: ScheduleFrequency) => {
    const cronExpression =
      frequency === 'cron' ? currentSchedule.cronExpression || '' : generateCronExpression(frequency);

    updateSchedule({
      frequency,
      cronExpression: frequency === 'cron' ? cronExpression : undefined,
    });
  };

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h3 style={styles.title}>Schedule Configuration</h3>
        <label style={styles.toggleLabel}>
          <input
            type="checkbox"
            checked={currentSchedule.enabled}
            onChange={(e) => updateSchedule({ enabled: e.target.checked })}
            disabled={readOnly}
          />
          <span>Enabled</span>
        </label>
      </div>

      <div style={styles.content}>
        {/* Basic Schedule Settings */}
        <div style={styles.section}>
          <h4 style={styles.sectionTitle}>Frequency</h4>

          <div style={styles.formGroup}>
            <label style={styles.label}>Schedule Type</label>
            <select
              value={currentSchedule.frequency}
              onChange={(e) => handleFrequencyChange(e.target.value as ScheduleFrequency)}
              style={styles.select}
              disabled={readOnly || !currentSchedule.enabled}
            >
              <option value="once">Once</option>
              <option value="hourly">Hourly</option>
              <option value="daily">Daily</option>
              <option value="weekly">Weekly</option>
              <option value="monthly">Monthly</option>
              <option value="quarterly">Quarterly</option>
              <option value="yearly">Yearly</option>
              <option value="cron">Custom (Cron)</option>
            </select>
          </div>

          {currentSchedule.frequency === 'cron' && (
            <div style={styles.formGroup}>
              <label style={styles.label}>Cron Expression</label>
              <input
                type="text"
                value={currentSchedule.cronExpression || ''}
                onChange={(e) => updateSchedule({ cronExpression: e.target.value })}
                style={styles.input}
                placeholder="0 0 * * *"
                disabled={readOnly || !currentSchedule.enabled}
              />
              <div style={styles.helpText}>
                Format: minute hour day month weekday
                <br />
                Example: "0 9 * * 1-5" = 9 AM on weekdays
              </div>
            </div>
          )}

          <div style={styles.formGroup}>
            <label style={styles.label}>Start Date</label>
            <input
              type="datetime-local"
              value={formatDateTimeLocal(currentSchedule.startDate)}
              onChange={(e) => updateSchedule({ startDate: new Date(e.target.value) })}
              style={styles.input}
              disabled={readOnly || !currentSchedule.enabled}
            />
          </div>

          <div style={styles.formGroup}>
            <label style={styles.label}>End Date (Optional)</label>
            <input
              type="datetime-local"
              value={currentSchedule.endDate ? formatDateTimeLocal(currentSchedule.endDate) : ''}
              onChange={(e) =>
                updateSchedule({ endDate: e.target.value ? new Date(e.target.value) : undefined })
              }
              style={styles.input}
              disabled={readOnly || !currentSchedule.enabled}
            />
          </div>

          <div style={styles.formGroup}>
            <label style={styles.label}>Timezone</label>
            <select
              value={currentSchedule.timezone}
              onChange={(e) => updateSchedule({ timezone: e.target.value })}
              style={styles.select}
              disabled={readOnly || !currentSchedule.enabled}
            >
              <option value="UTC">UTC</option>
              <option value="America/New_York">America/New York (EST)</option>
              <option value="America/Chicago">America/Chicago (CST)</option>
              <option value="America/Denver">America/Denver (MST)</option>
              <option value="America/Los_Angeles">America/Los Angeles (PST)</option>
              <option value="Europe/London">Europe/London</option>
              <option value="Europe/Paris">Europe/Paris</option>
              <option value="Asia/Tokyo">Asia/Tokyo</option>
              <option value="Asia/Shanghai">Asia/Shanghai</option>
              <option value="Australia/Sydney">Australia/Sydney</option>
            </select>
          </div>
        </div>

        {/* Execution Conditions */}
        <div style={styles.section}>
          <h4 style={styles.sectionTitle}>Execution Conditions</h4>

          <div style={styles.formGroup}>
            <label style={styles.checkboxLabel}>
              <input
                type="checkbox"
                checked={currentSchedule.conditions?.dataAvailable ?? false}
                onChange={(e) =>
                  updateSchedule({
                    conditions: {
                      ...currentSchedule.conditions,
                      dataAvailable: e.target.checked,
                    },
                  })
                }
                disabled={readOnly || !currentSchedule.enabled}
              />
              <span>Wait for data availability</span>
            </label>
          </div>

          <div style={styles.formGroup}>
            <label style={styles.label}>Minimum Rows Required</label>
            <input
              type="number"
              value={currentSchedule.conditions?.minimumRows || 0}
              onChange={(e) =>
                updateSchedule({
                  conditions: {
                    ...currentSchedule.conditions,
                    minimumRows: Number(e.target.value),
                  },
                })
              }
              style={styles.input}
              min="0"
              disabled={readOnly || !currentSchedule.enabled}
            />
          </div>

          <div style={styles.formGroup}>
            <label style={styles.label}>Custom Condition (SQL/Expression)</label>
            <textarea
              value={currentSchedule.conditions?.customCondition || ''}
              onChange={(e) =>
                updateSchedule({
                  conditions: {
                    ...currentSchedule.conditions,
                    customCondition: e.target.value,
                  },
                })
              }
              style={styles.textarea}
              placeholder="e.g., SELECT COUNT(*) > 0 FROM table WHERE updated_at > NOW() - INTERVAL 1 DAY"
              rows={3}
              disabled={readOnly || !currentSchedule.enabled}
            />
          </div>
        </div>

        {/* Retry Policy */}
        <div style={styles.section}>
          <h4 style={styles.sectionTitle}>Retry Policy</h4>

          <div style={styles.formGroup}>
            <label style={styles.label}>Max Retry Attempts</label>
            <input
              type="number"
              value={currentSchedule.retryPolicy?.maxAttempts || 3}
              onChange={(e) =>
                updateSchedule({
                  retryPolicy: {
                    ...currentSchedule.retryPolicy!,
                    maxAttempts: Number(e.target.value),
                  },
                })
              }
              style={styles.input}
              min="0"
              max="10"
              disabled={readOnly || !currentSchedule.enabled}
            />
          </div>

          <div style={styles.formGroup}>
            <label style={styles.label}>Retry Interval (seconds)</label>
            <input
              type="number"
              value={currentSchedule.retryPolicy?.retryInterval || 60}
              onChange={(e) =>
                updateSchedule({
                  retryPolicy: {
                    ...currentSchedule.retryPolicy!,
                    retryInterval: Number(e.target.value),
                  },
                })
              }
              style={styles.input}
              min="1"
              disabled={readOnly || !currentSchedule.enabled}
            />
          </div>

          <div style={styles.formGroup}>
            <label style={styles.label}>Backoff Multiplier</label>
            <input
              type="number"
              value={currentSchedule.retryPolicy?.backoffMultiplier || 2}
              onChange={(e) =>
                updateSchedule({
                  retryPolicy: {
                    ...currentSchedule.retryPolicy!,
                    backoffMultiplier: Number(e.target.value),
                  },
                })
              }
              style={styles.input}
              min="1"
              step="0.1"
              disabled={readOnly || !currentSchedule.enabled}
            />
            <div style={styles.helpText}>
              Each retry will wait (interval × multiplier^attempt) seconds
            </div>
          </div>
        </div>

        {/* Notifications */}
        <div style={styles.section}>
          <h4 style={styles.sectionTitle}>Notifications</h4>

          <div style={styles.formGroup}>
            <label style={styles.checkboxLabel}>
              <input
                type="checkbox"
                checked={currentSchedule.notifications?.onSuccess ?? false}
                onChange={(e) =>
                  updateSchedule({
                    notifications: {
                      ...currentSchedule.notifications!,
                      onSuccess: e.target.checked,
                    },
                  })
                }
                disabled={readOnly || !currentSchedule.enabled}
              />
              <span>Notify on successful execution</span>
            </label>
          </div>

          <div style={styles.formGroup}>
            <label style={styles.checkboxLabel}>
              <input
                type="checkbox"
                checked={currentSchedule.notifications?.onFailure ?? true}
                onChange={(e) =>
                  updateSchedule({
                    notifications: {
                      ...currentSchedule.notifications!,
                      onFailure: e.target.checked,
                    },
                  })
                }
                disabled={readOnly || !currentSchedule.enabled}
              />
              <span>Notify on execution failure</span>
            </label>
          </div>

          <div style={styles.formGroup}>
            <label style={styles.label}>Recipients (comma-separated emails)</label>
            <input
              type="text"
              value={(currentSchedule.notifications?.recipients || []).join(', ')}
              onChange={(e) =>
                updateSchedule({
                  notifications: {
                    ...currentSchedule.notifications!,
                    recipients: e.target.value.split(',').map((r) => r.trim()).filter(Boolean),
                  },
                })
              }
              style={styles.input}
              placeholder="user1@example.com, user2@example.com"
              disabled={readOnly || !currentSchedule.enabled}
            />
          </div>
        </div>

        {/* Schedule Preview */}
        <div style={styles.section}>
          <h4 style={styles.sectionTitle}>Schedule Summary</h4>
          <div style={styles.summary}>
            <div style={styles.summaryItem}>
              <span style={styles.summaryLabel}>Status:</span>
              <span style={currentSchedule.enabled ? styles.statusEnabled : styles.statusDisabled}>
                {currentSchedule.enabled ? 'Enabled' : 'Disabled'}
              </span>
            </div>
            <div style={styles.summaryItem}>
              <span style={styles.summaryLabel}>Frequency:</span>
              <span>{currentSchedule.frequency}</span>
            </div>
            {currentSchedule.cronExpression && (
              <div style={styles.summaryItem}>
                <span style={styles.summaryLabel}>Cron:</span>
                <code style={styles.codeText}>{currentSchedule.cronExpression}</code>
              </div>
            )}
            <div style={styles.summaryItem}>
              <span style={styles.summaryLabel}>Next Run:</span>
              <span>{calculateNextRun(currentSchedule)}</span>
            </div>
            <div style={styles.summaryItem}>
              <span style={styles.summaryLabel}>Timezone:</span>
              <span>{currentSchedule.timezone}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

// Helper functions
function createDefaultSchedule(): ReportSchedule {
  return {
    id: generateId(),
    enabled: false,
    frequency: 'daily',
    startDate: new Date(),
    timezone: 'UTC',
    retryPolicy: {
      maxAttempts: 3,
      retryInterval: 60,
      backoffMultiplier: 2,
    },
    notifications: {
      onSuccess: false,
      onFailure: true,
      recipients: [],
    },
  };
}

function generateId(): string {
  return `schedule-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}

function formatDateTimeLocal(date: Date): string {
  const d = new Date(date);
  const offset = d.getTimezoneOffset();
  const adjusted = new Date(d.getTime() - offset * 60000);
  return adjusted.toISOString().slice(0, 16);
}

function calculateNextRun(schedule: ReportSchedule): string {
  if (!schedule.enabled) return 'N/A (Disabled)';

  const now = new Date();
  const start = new Date(schedule.startDate);

  if (start > now) {
    return start.toLocaleString();
  }

  // Simple calculation - in production, use a cron parser library
  switch (schedule.frequency) {
    case 'once':
      return start.toLocaleString();
    case 'hourly':
      return new Date(now.getFullYear(), now.getMonth(), now.getDate(), now.getHours() + 1).toLocaleString();
    case 'daily':
      return new Date(now.getFullYear(), now.getMonth(), now.getDate() + 1).toLocaleString();
    case 'weekly':
      return new Date(now.getFullYear(), now.getMonth(), now.getDate() + 7).toLocaleString();
    case 'monthly':
      return new Date(now.getFullYear(), now.getMonth() + 1, 1).toLocaleString();
    default:
      return 'Calculating...';
  }
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
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
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
  toggleLabel: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    fontSize: '13px',
    fontWeight: 500,
    color: '#475569',
    cursor: 'pointer',
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
  sectionTitle: {
    fontSize: '13px',
    fontWeight: 600,
    margin: '0 0 12px 0',
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
  helpText: {
    fontSize: '11px',
    color: '#64748b',
    marginTop: '4px',
  },
  summary: {
    backgroundColor: '#f8fafc',
    border: '1px solid #e2e8f0',
    borderRadius: '6px',
    padding: '12px',
  },
  summaryItem: {
    display: 'flex',
    justifyContent: 'space-between',
    padding: '6px 0',
    fontSize: '13px',
    borderBottom: '1px solid #e2e8f0',
  },
  summaryLabel: {
    fontWeight: 600,
    color: '#475569',
  },
  statusEnabled: {
    color: '#10b981',
    fontWeight: 600,
  },
  statusDisabled: {
    color: '#ef4444',
    fontWeight: 600,
  },
  codeText: {
    fontFamily: 'monospace',
    fontSize: '12px',
    backgroundColor: '#f1f5f9',
    padding: '2px 6px',
    borderRadius: '3px',
  },
};

export default ReportScheduler;
