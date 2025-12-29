/**
 * CADDY v0.4.0 - Workflow Triggers Component
 * Event triggers configuration (schedule, webhook, data changes)
 */

import React, { useState, useCallback, useMemo } from 'react';
import type { WorkflowTrigger, TriggerType, TriggerConfig } from './types';

export interface WorkflowTriggersProps {
  triggers: WorkflowTrigger[];
  onTriggerAdd?: (trigger: Omit<WorkflowTrigger, 'id'>) => void;
  onTriggerUpdate?: (triggerId: string, updates: Partial<WorkflowTrigger>) => void;
  onTriggerDelete?: (triggerId: string) => void;
  onTriggerToggle?: (triggerId: string, enabled: boolean) => void;
  readOnly?: boolean;
}

const TRIGGER_TYPES: { value: TriggerType; label: string; icon: string; description: string }[] = [
  {
    value: 'schedule',
    label: 'Schedule',
    icon: '⏰',
    description: 'Run on a schedule using cron expressions',
  },
  {
    value: 'webhook',
    label: 'Webhook',
    icon: '🔗',
    description: 'Trigger via HTTP webhook endpoint',
  },
  {
    value: 'data-change',
    label: 'Data Change',
    icon: '💾',
    description: 'Trigger when data changes in database',
  },
  {
    value: 'manual',
    label: 'Manual',
    icon: '👆',
    description: 'Manual execution by user',
  },
  {
    value: 'event',
    label: 'Event',
    icon: '⚡',
    description: 'Trigger on specific events',
  },
];

const CRON_PRESETS = [
  { label: 'Every minute', value: '* * * * *' },
  { label: 'Every 5 minutes', value: '*/5 * * * *' },
  { label: 'Every hour', value: '0 * * * *' },
  { label: 'Every day at midnight', value: '0 0 * * *' },
  { label: 'Every day at 9 AM', value: '0 9 * * *' },
  { label: 'Every Monday at 9 AM', value: '0 9 * * 1' },
  { label: 'First day of month', value: '0 0 1 * *' },
];

const generateId = () => `trigger_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

export const WorkflowTriggers: React.FC<WorkflowTriggersProps> = ({
  triggers,
  onTriggerAdd,
  onTriggerUpdate,
  onTriggerDelete,
  onTriggerToggle,
  readOnly = false,
}) => {
  const [isAdding, setIsAdding] = useState(false);
  const [selectedType, setSelectedType] = useState<TriggerType>('schedule');
  const [editingTriggerId, setEditingTriggerId] = useState<string | null>(null);
  const [newTriggerConfig, setNewTriggerConfig] = useState<TriggerConfig>({});

  const handleAddTrigger = useCallback(() => {
    if (onTriggerAdd) {
      onTriggerAdd({
        type: selectedType,
        config: newTriggerConfig,
        enabled: true,
      });
      setIsAdding(false);
      setNewTriggerConfig({});
    }
  }, [selectedType, newTriggerConfig, onTriggerAdd]);

  const handleUpdateConfig = useCallback(
    (triggerId: string, key: string, value: unknown) => {
      if (onTriggerUpdate && !readOnly) {
        const trigger = triggers.find((t) => t.id === triggerId);
        if (trigger) {
          onTriggerUpdate(triggerId, {
            config: {
              ...trigger.config,
              [key]: value,
            },
          });
        }
      }
    },
    [triggers, onTriggerUpdate, readOnly]
  );

  const renderTriggerConfig = useCallback(
    (trigger: WorkflowTrigger, isNew: boolean = false) => {
      const config = isNew ? newTriggerConfig : trigger.config;
      const updateFn = isNew
        ? (key: string, value: unknown) =>
            setNewTriggerConfig((prev) => ({ ...prev, [key]: value }))
        : (key: string, value: unknown) => handleUpdateConfig(trigger.id, key, value);

      switch (isNew ? selectedType : trigger.type) {
        case 'schedule':
          return (
            <>
              <div style={{ marginBottom: '12px' }}>
                <label
                  style={{
                    display: 'block',
                    fontSize: '12px',
                    fontWeight: 600,
                    color: '#64748b',
                    marginBottom: '6px',
                  }}
                >
                  Cron Expression
                </label>
                <input
                  type="text"
                  value={(config.schedule as string) || ''}
                  onChange={(e) => updateFn('schedule', e.target.value)}
                  disabled={readOnly}
                  placeholder="* * * * *"
                  style={{
                    width: '100%',
                    padding: '8px 12px',
                    border: '1px solid #e2e8f0',
                    borderRadius: '4px',
                    fontSize: '13px',
                    fontFamily: 'monospace',
                  }}
                />
              </div>
              <div style={{ marginBottom: '12px' }}>
                <label
                  style={{
                    display: 'block',
                    fontSize: '12px',
                    fontWeight: 600,
                    color: '#64748b',
                    marginBottom: '6px',
                  }}
                >
                  Presets
                </label>
                <select
                  onChange={(e) => updateFn('schedule', e.target.value)}
                  disabled={readOnly}
                  value=""
                  style={{
                    width: '100%',
                    padding: '8px 12px',
                    border: '1px solid #e2e8f0',
                    borderRadius: '4px',
                    fontSize: '13px',
                    cursor: readOnly ? 'default' : 'pointer',
                  }}
                >
                  <option value="">Select a preset...</option>
                  {CRON_PRESETS.map((preset) => (
                    <option key={preset.value} value={preset.value}>
                      {preset.label} ({preset.value})
                    </option>
                  ))}
                </select>
              </div>
            </>
          );

        case 'webhook':
          return (
            <>
              <div style={{ marginBottom: '12px' }}>
                <label
                  style={{
                    display: 'block',
                    fontSize: '12px',
                    fontWeight: 600,
                    color: '#64748b',
                    marginBottom: '6px',
                  }}
                >
                  Webhook URL
                </label>
                <input
                  type="text"
                  value={(config.webhookUrl as string) || ''}
                  readOnly
                  placeholder="Generated automatically"
                  style={{
                    width: '100%',
                    padding: '8px 12px',
                    border: '1px solid #e2e8f0',
                    borderRadius: '4px',
                    fontSize: '13px',
                    backgroundColor: '#f8fafc',
                  }}
                />
                <p
                  style={{
                    fontSize: '11px',
                    color: '#64748b',
                    marginTop: '4px',
                  }}
                >
                  POST requests to this URL will trigger the workflow
                </p>
              </div>
              <div style={{ marginBottom: '12px' }}>
                <label style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                  <input
                    type="checkbox"
                    checked={(config.requireAuth as boolean) || false}
                    onChange={(e) => updateFn('requireAuth', e.target.checked)}
                    disabled={readOnly}
                    style={{ cursor: readOnly ? 'default' : 'pointer' }}
                  />
                  <span style={{ fontSize: '13px', color: '#1e293b' }}>
                    Require authentication
                  </span>
                </label>
              </div>
            </>
          );

        case 'data-change':
          return (
            <>
              <div style={{ marginBottom: '12px' }}>
                <label
                  style={{
                    display: 'block',
                    fontSize: '12px',
                    fontWeight: 600,
                    color: '#64748b',
                    marginBottom: '6px',
                  }}
                >
                  Table/Collection
                </label>
                <input
                  type="text"
                  value={(config.table as string) || ''}
                  onChange={(e) => updateFn('table', e.target.value)}
                  disabled={readOnly}
                  placeholder="users"
                  style={{
                    width: '100%',
                    padding: '8px 12px',
                    border: '1px solid #e2e8f0',
                    borderRadius: '4px',
                    fontSize: '13px',
                  }}
                />
              </div>
              <div style={{ marginBottom: '12px' }}>
                <label
                  style={{
                    display: 'block',
                    fontSize: '12px',
                    fontWeight: 600,
                    color: '#64748b',
                    marginBottom: '6px',
                  }}
                >
                  Operation
                </label>
                <select
                  value={(config.operation as string) || 'any'}
                  onChange={(e) => updateFn('operation', e.target.value)}
                  disabled={readOnly}
                  style={{
                    width: '100%',
                    padding: '8px 12px',
                    border: '1px solid #e2e8f0',
                    borderRadius: '4px',
                    fontSize: '13px',
                    cursor: readOnly ? 'default' : 'pointer',
                  }}
                >
                  <option value="any">Any change</option>
                  <option value="insert">Insert</option>
                  <option value="update">Update</option>
                  <option value="delete">Delete</option>
                </select>
              </div>
              <div style={{ marginBottom: '12px' }}>
                <label
                  style={{
                    display: 'block',
                    fontSize: '12px',
                    fontWeight: 600,
                    color: '#64748b',
                    marginBottom: '6px',
                  }}
                >
                  Filter (SQL WHERE clause)
                </label>
                <input
                  type="text"
                  value={(config.filter as string) || ''}
                  onChange={(e) => updateFn('filter', e.target.value)}
                  disabled={readOnly}
                  placeholder="status = 'active'"
                  style={{
                    width: '100%',
                    padding: '8px 12px',
                    border: '1px solid #e2e8f0',
                    borderRadius: '4px',
                    fontSize: '13px',
                    fontFamily: 'monospace',
                  }}
                />
              </div>
            </>
          );

        case 'event':
          return (
            <>
              <div style={{ marginBottom: '12px' }}>
                <label
                  style={{
                    display: 'block',
                    fontSize: '12px',
                    fontWeight: 600,
                    color: '#64748b',
                    marginBottom: '6px',
                  }}
                >
                  Event Name
                </label>
                <input
                  type="text"
                  value={(config.event as string) || ''}
                  onChange={(e) => updateFn('event', e.target.value)}
                  disabled={readOnly}
                  placeholder="user.created"
                  style={{
                    width: '100%',
                    padding: '8px 12px',
                    border: '1px solid #e2e8f0',
                    borderRadius: '4px',
                    fontSize: '13px',
                  }}
                />
              </div>
              <div style={{ marginBottom: '12px' }}>
                <label
                  style={{
                    display: 'block',
                    fontSize: '12px',
                    fontWeight: 600,
                    color: '#64748b',
                    marginBottom: '6px',
                  }}
                >
                  Event Source
                </label>
                <input
                  type="text"
                  value={(config.source as string) || ''}
                  onChange={(e) => updateFn('source', e.target.value)}
                  disabled={readOnly}
                  placeholder="Optional event source"
                  style={{
                    width: '100%',
                    padding: '8px 12px',
                    border: '1px solid #e2e8f0',
                    borderRadius: '4px',
                    fontSize: '13px',
                  }}
                />
              </div>
            </>
          );

        case 'manual':
          return (
            <div
              style={{
                padding: '16px',
                backgroundColor: '#f8fafc',
                borderRadius: '6px',
                textAlign: 'center',
                color: '#64748b',
                fontSize: '13px',
              }}
            >
              Manual triggers require no configuration. The workflow can be started manually
              from the UI or via API.
            </div>
          );

        default:
          return null;
      }
    },
    [
      newTriggerConfig,
      selectedType,
      handleUpdateConfig,
      readOnly,
    ]
  );

  const containerStyle: React.CSSProperties = {
    display: 'flex',
    flexDirection: 'column',
    height: '100%',
    backgroundColor: '#fff',
    borderRadius: '8px',
    overflow: 'hidden',
    border: '1px solid #e2e8f0',
  };

  return (
    <div style={containerStyle}>
      {/* Header */}
      <div
        style={{
          padding: '16px',
          borderBottom: '1px solid #e2e8f0',
          backgroundColor: '#f8fafc',
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
        }}
      >
        <h3 style={{ fontSize: '16px', fontWeight: 600 }}>Workflow Triggers</h3>
        {!readOnly && !isAdding && (
          <button
            onClick={() => setIsAdding(true)}
            style={{
              padding: '8px 16px',
              backgroundColor: '#3b82f6',
              color: '#fff',
              border: 'none',
              borderRadius: '6px',
              cursor: 'pointer',
              fontSize: '13px',
              fontWeight: 500,
            }}
          >
            + Add Trigger
          </button>
        )}
      </div>

      {/* Content */}
      <div style={{ flex: 1, overflow: 'auto', padding: '16px' }}>
        {/* Add Trigger Form */}
        {isAdding && (
          <div
            style={{
              padding: '16px',
              backgroundColor: '#f8fafc',
              border: '2px dashed #3b82f6',
              borderRadius: '8px',
              marginBottom: '16px',
            }}
          >
            <h4 style={{ fontSize: '14px', fontWeight: 600, marginBottom: '12px' }}>
              Add New Trigger
            </h4>

            {/* Trigger Type Selection */}
            <div style={{ marginBottom: '16px' }}>
              <label
                style={{
                  display: 'block',
                  fontSize: '12px',
                  fontWeight: 600,
                  color: '#64748b',
                  marginBottom: '8px',
                }}
              >
                Trigger Type
              </label>
              <div
                style={{
                  display: 'grid',
                  gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))',
                  gap: '8px',
                }}
              >
                {TRIGGER_TYPES.map((type) => (
                  <button
                    key={type.value}
                    onClick={() => setSelectedType(type.value)}
                    style={{
                      padding: '12px',
                      backgroundColor:
                        selectedType === type.value ? '#eff6ff' : '#fff',
                      border: `2px solid ${
                        selectedType === type.value ? '#3b82f6' : '#e2e8f0'
                      }`,
                      borderRadius: '6px',
                      cursor: 'pointer',
                      textAlign: 'left',
                      transition: 'all 0.2s ease',
                    }}
                  >
                    <div style={{ fontSize: '20px', marginBottom: '4px' }}>
                      {type.icon}
                    </div>
                    <div style={{ fontSize: '13px', fontWeight: 600, color: '#1e293b' }}>
                      {type.label}
                    </div>
                  </button>
                ))}
              </div>
            </div>

            {/* Trigger Configuration */}
            {renderTriggerConfig({ id: '', type: selectedType, config: newTriggerConfig, enabled: true }, true)}

            {/* Actions */}
            <div style={{ display: 'flex', gap: '8px', justifyContent: 'flex-end' }}>
              <button
                onClick={() => {
                  setIsAdding(false);
                  setNewTriggerConfig({});
                }}
                style={{
                  padding: '8px 16px',
                  backgroundColor: '#fff',
                  color: '#64748b',
                  border: '1px solid #e2e8f0',
                  borderRadius: '6px',
                  cursor: 'pointer',
                  fontSize: '13px',
                }}
              >
                Cancel
              </button>
              <button
                onClick={handleAddTrigger}
                style={{
                  padding: '8px 16px',
                  backgroundColor: '#3b82f6',
                  color: '#fff',
                  border: 'none',
                  borderRadius: '6px',
                  cursor: 'pointer',
                  fontSize: '13px',
                  fontWeight: 500,
                }}
              >
                Add Trigger
              </button>
            </div>
          </div>
        )}

        {/* Existing Triggers */}
        {triggers.length > 0 ? (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
            {triggers.map((trigger) => {
              const typeInfo = TRIGGER_TYPES.find((t) => t.value === trigger.type);
              const isEditing = editingTriggerId === trigger.id;

              return (
                <div
                  key={trigger.id}
                  style={{
                    padding: '16px',
                    backgroundColor: trigger.enabled ? '#fff' : '#f8fafc',
                    border: '1px solid #e2e8f0',
                    borderRadius: '8px',
                  }}
                >
                  {/* Header */}
                  <div
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'space-between',
                      marginBottom: isEditing ? '12px' : '0',
                    }}
                  >
                    <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                      <span style={{ fontSize: '24px' }}>{typeInfo?.icon}</span>
                      <div>
                        <div style={{ fontSize: '14px', fontWeight: 600, color: '#1e293b' }}>
                          {typeInfo?.label}
                        </div>
                        <div style={{ fontSize: '12px', color: '#64748b' }}>
                          {typeInfo?.description}
                        </div>
                      </div>
                    </div>

                    {!readOnly && (
                      <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                        <label
                          style={{
                            display: 'flex',
                            alignItems: 'center',
                            gap: '6px',
                            cursor: 'pointer',
                          }}
                        >
                          <input
                            type="checkbox"
                            checked={trigger.enabled}
                            onChange={(e) =>
                              onTriggerToggle &&
                              onTriggerToggle(trigger.id, e.target.checked)
                            }
                            style={{ cursor: 'pointer' }}
                          />
                          <span style={{ fontSize: '12px', color: '#64748b' }}>
                            Enabled
                          </span>
                        </label>
                        <button
                          onClick={() =>
                            setEditingTriggerId(isEditing ? null : trigger.id)
                          }
                          style={{
                            padding: '4px 12px',
                            backgroundColor: '#fff',
                            color: '#64748b',
                            border: '1px solid #e2e8f0',
                            borderRadius: '4px',
                            cursor: 'pointer',
                            fontSize: '12px',
                          }}
                        >
                          {isEditing ? 'Done' : 'Edit'}
                        </button>
                        <button
                          onClick={() => onTriggerDelete && onTriggerDelete(trigger.id)}
                          style={{
                            padding: '4px 12px',
                            backgroundColor: '#ef4444',
                            color: '#fff',
                            border: 'none',
                            borderRadius: '4px',
                            cursor: 'pointer',
                            fontSize: '12px',
                          }}
                        >
                          Delete
                        </button>
                      </div>
                    )}
                  </div>

                  {/* Configuration */}
                  {isEditing && <div style={{ marginTop: '12px' }}>{renderTriggerConfig(trigger)}</div>}
                </div>
              );
            })}
          </div>
        ) : (
          !isAdding && (
            <div
              style={{
                textAlign: 'center',
                color: '#94a3b8',
                padding: '40px 20px',
              }}
            >
              No triggers configured. Add a trigger to start automating your workflow.
            </div>
          )
        )}
      </div>
    </div>
  );
};

export default WorkflowTriggers;
