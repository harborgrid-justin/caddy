/**
 * CADDY v0.4.0 Enterprise Integration Settings
 * Third-party API configurations and webhooks
 */

import React, { useState, useEffect, useCallback, useRef } from 'react';
import {
  IntegrationSettings as IntegrationSettingsType,
  Integration,
  Webhook,
  FormState,
  ValidationError,
  UndoRedoState,
  ToastNotification,
  ConfirmationDialog,
  SettingsHistory,
} from './types';

interface IntegrationSettingsProps {
  onSave: (section: string, data: IntegrationSettingsType) => Promise<void>;
  onConfirm: (config: Omit<ConfirmationDialog, 'open'>) => void;
  addToast: (toast: Omit<ToastNotification, 'id'>) => void;
  addToHistory: (entry: Omit<SettingsHistory, 'id' | 'timestamp'>) => void;
}

const AVAILABLE_INTEGRATIONS = [
  { id: 'slack', name: 'Slack', icon: '💬', type: 'communication' },
  { id: 'github', name: 'GitHub', icon: '🐙', type: 'development' },
  { id: 'jira', name: 'Jira', icon: '📋', type: 'project-management' },
  { id: 'salesforce', name: 'Salesforce', icon: '☁️', type: 'crm' },
  { id: 'stripe', name: 'Stripe', icon: '💳', type: 'payment' },
  { id: 'google-workspace', name: 'Google Workspace', icon: '📧', type: 'productivity' },
];

const IntegrationSettings: React.FC<IntegrationSettingsProps> = ({
  onSave,
  onConfirm,
  addToast,
  addToHistory,
}) => {
  const [settings, setSettings] = useState<IntegrationSettingsType>({
    id: 'integration-1',
    version: 1,
    updatedAt: new Date(),
    updatedBy: 'current-user',
    integrations: [],
    webhooks: [],
    apiLimits: {
      rateLimitPerMinute: 60,
      rateLimitPerHour: 1000,
      maxPayloadSize: 10485760, // 10MB
      allowedOrigins: ['*'],
      requireApiKey: true,
    },
  });

  const [formState, setFormState] = useState<FormState<IntegrationSettingsType>>({
    values: settings,
    errors: [],
    isDirty: false,
    isSubmitting: false,
    isValid: true,
  });

  const [undoRedo, setUndoRedo] = useState<UndoRedoState<IntegrationSettingsType>>({
    past: [],
    present: settings,
    future: [],
  });

  const [testingIntegration, setTestingIntegration] = useState<string | null>(null);
  const saveTimeoutRef = useRef<NodeJS.Timeout | undefined>(undefined);

  // Validation
  const validate = useCallback((data: IntegrationSettingsType): ValidationError[] => {
    const errors: ValidationError[] = [];

    // Validate webhooks
    data.webhooks.forEach((webhook, index) => {
      if (!webhook.name.trim()) {
        errors.push({ field: `webhooks.${index}.name`, message: 'Webhook name is required' });
      }
      try {
        new URL(webhook.url);
      } catch {
        errors.push({ field: `webhooks.${index}.url`, message: 'Invalid webhook URL' });
      }
      if (webhook.retryCount < 0 || webhook.retryCount > 10) {
        errors.push({ field: `webhooks.${index}.retryCount`, message: 'Retry count must be between 0 and 10' });
      }
    });

    // Validate API limits
    if (data.apiLimits.rateLimitPerMinute < 1) {
      errors.push({ field: 'apiLimits.rateLimitPerMinute', message: 'Rate limit must be at least 1' });
    }
    if (data.apiLimits.maxPayloadSize < 1024) {
      errors.push({ field: 'apiLimits.maxPayloadSize', message: 'Payload size must be at least 1KB' });
    }

    return errors;
  }, []);

  // Debounced auto-save
  useEffect(() => {
    if (formState.isDirty && formState.isValid) {
      if (saveTimeoutRef.current) {
        clearTimeout(saveTimeoutRef.current);
      }

      saveTimeoutRef.current = setTimeout(() => {
        handleSave();
      }, 2000);
    }

    return () => {
      if (saveTimeoutRef.current) {
        clearTimeout(saveTimeoutRef.current);
      }
    };
  }, [formState.values, formState.isDirty]);

  // Update field
  const updateField = useCallback(
    (field: string, value: unknown) => {
      setFormState((prev) => {
        const newValues = { ...prev.values };
        const keys = field.split('.');
        let current: any = newValues;

        for (let i = 0; i < keys.length - 1; i++) {
          current = current[keys[i]];
        }
        current[keys[keys.length - 1]] = value;

        const errors = validate(newValues);

        setUndoRedo((undoPrev) => ({
          past: [...undoPrev.past, undoPrev.present],
          present: newValues,
          future: [],
        }));

        return {
          values: newValues,
          errors,
          isDirty: true,
          isSubmitting: false,
          isValid: errors.length === 0,
        };
      });
    },
    [validate]
  );

  // Undo/Redo
  const undo = useCallback(() => {
    setUndoRedo((prev) => {
      if (prev.past.length === 0) return prev;
      const previous = prev.past[prev.past.length - 1];
      const newPast = prev.past.slice(0, prev.past.length - 1);
      setFormState((formPrev) => ({
        ...formPrev,
        values: previous,
        errors: validate(previous),
        isDirty: true,
      }));
      return { past: newPast, present: previous, future: [prev.present, ...prev.future] };
    });
    addToast({ type: 'info', message: 'Changes undone', duration: 2000 });
  }, [validate, addToast]);

  const redo = useCallback(() => {
    setUndoRedo((prev) => {
      if (prev.future.length === 0) return prev;
      const next = prev.future[0];
      const newFuture = prev.future.slice(1);
      setFormState((formPrev) => ({
        ...formPrev,
        values: next,
        errors: validate(next),
        isDirty: true,
      }));
      return { past: [...prev.past, prev.present], present: next, future: newFuture };
    });
    addToast({ type: 'info', message: 'Changes redone', duration: 2000 });
  }, [validate, addToast]);

  // Save handler
  const handleSave = useCallback(async () => {
    const errors = validate(formState.values);
    if (errors.length > 0) {
      setFormState((prev) => ({ ...prev, errors, isValid: false }));
      addToast({ type: 'error', message: 'Please fix validation errors' });
      return;
    }

    setFormState((prev) => ({ ...prev, isSubmitting: true }));

    try {
      await onSave('integrations', formState.values);
      addToHistory({
        section: 'Integration Settings',
        action: 'update',
        changes: [],
        userId: 'current-user',
        userName: 'Current User',
      });
      setFormState((prev) => ({ ...prev, isDirty: false, isSubmitting: false }));
    } catch (error) {
      setFormState((prev) => ({ ...prev, isSubmitting: false }));
      addToast({
        type: 'error',
        message: error instanceof Error ? error.message : 'Save failed',
      });
    }
  }, [formState.values, validate, onSave, addToast, addToHistory]);

  // Add integration
  const addIntegration = useCallback(
    (integrationId: string) => {
      const template = AVAILABLE_INTEGRATIONS.find((i) => i.id === integrationId);
      if (!template) return;

      const newIntegration: Integration = {
        id: `${integrationId}-${Date.now()}`,
        name: template.name,
        type: template.type,
        enabled: false,
        config: {},
        credentials: {},
        status: 'disconnected',
      };

      updateField('integrations', [...formState.values.integrations, newIntegration]);
      addToast({ type: 'success', message: `${template.name} integration added` });
    },
    [formState.values.integrations, updateField, addToast]
  );

  // Remove integration
  const removeIntegration = useCallback(
    (id: string) => {
      onConfirm({
        title: 'Remove Integration',
        message: 'Are you sure you want to remove this integration? This action cannot be undone.',
        severity: 'error',
        confirmText: 'Remove',
        cancelText: 'Cancel',
        onConfirm: () => {
          updateField(
            'integrations',
            formState.values.integrations.filter((i) => i.id !== id)
          );
          addToast({ type: 'success', message: 'Integration removed' });
        },
        onCancel: () => {},
      });
    },
    [formState.values.integrations, updateField, onConfirm, addToast]
  );

  // Test integration
  const testIntegration = useCallback(
    async (id: string) => {
      setTestingIntegration(id);

      try {
        await new Promise((resolve) => setTimeout(resolve, 1500));
        const integration = formState.values.integrations.find((i) => i.id === id);
        addToast({
          type: 'success',
          message: `${integration?.name} connection successful`,
        });
      } catch (error) {
        addToast({
          type: 'error',
          message: 'Connection test failed',
        });
      } finally {
        setTestingIntegration(null);
      }
    },
    [formState.values.integrations, addToast]
  );

  // Add webhook
  const addWebhook = useCallback(() => {
    const newWebhook: Webhook = {
      id: `webhook-${Date.now()}`,
      name: 'New Webhook',
      url: '',
      events: [],
      enabled: true,
      retryCount: 3,
      timeout: 30000,
    };

    updateField('webhooks', [...formState.values.webhooks, newWebhook]);
  }, [formState.values.webhooks, updateField]);

  // Remove webhook
  const removeWebhook = useCallback(
    (id: string) => {
      onConfirm({
        title: 'Remove Webhook',
        message: 'Are you sure you want to remove this webhook?',
        severity: 'warning',
        confirmText: 'Remove',
        cancelText: 'Cancel',
        onConfirm: () => {
          updateField(
            'webhooks',
            formState.values.webhooks.filter((w) => w.id !== id)
          );
          addToast({ type: 'success', message: 'Webhook removed' });
        },
        onCancel: () => {},
      });
    },
    [formState.values.webhooks, updateField, onConfirm, addToast]
  );

  const getFieldError = (field: string): string | undefined => {
    return formState.errors.find((e) => e.field === field)?.message;
  };

  return (
    <div style={{ maxWidth: '800px' }}>
      <div style={{ marginBottom: '2rem' }}>
        <h2 style={{ fontSize: '1.5rem', marginBottom: '0.5rem' }}>Integration Settings</h2>
        <p style={{ color: '#666', margin: 0 }}>
          Configure third-party integrations, webhooks, and API settings
        </p>
      </div>

      {/* Undo/Redo Controls */}
      <div
        style={{
          marginBottom: '1.5rem',
          display: 'flex',
          gap: '0.5rem',
          padding: '0.75rem',
          backgroundColor: '#f5f5f5',
          borderRadius: '4px',
        }}
      >
        <button
          onClick={undo}
          disabled={undoRedo.past.length === 0}
          aria-label="Undo"
          style={{
            padding: '0.5rem 1rem',
            backgroundColor: '#fff',
            border: '1px solid #e0e0e0',
            borderRadius: '4px',
            cursor: undoRedo.past.length === 0 ? 'not-allowed' : 'pointer',
            opacity: undoRedo.past.length === 0 ? 0.5 : 1,
          }}
        >
          ↶ Undo
        </button>
        <button
          onClick={redo}
          disabled={undoRedo.future.length === 0}
          aria-label="Redo"
          style={{
            padding: '0.5rem 1rem',
            backgroundColor: '#fff',
            border: '1px solid #e0e0e0',
            borderRadius: '4px',
            cursor: undoRedo.future.length === 0 ? 'not-allowed' : 'pointer',
            opacity: undoRedo.future.length === 0 ? 0.5 : 1,
          }}
        >
          ↷ Redo
        </button>
      </div>

      {/* Available Integrations */}
      <section
        style={{
          backgroundColor: '#fff',
          borderRadius: '8px',
          padding: '1.5rem',
          marginBottom: '1.5rem',
          border: '1px solid #e0e0e0',
        }}
      >
        <h3 style={{ fontSize: '1.125rem', marginBottom: '1rem' }}>Available Integrations</h3>

        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))', gap: '1rem' }}>
          {AVAILABLE_INTEGRATIONS.map((integration) => {
            const isAdded = formState.values.integrations.some((i) => i.type === integration.type);
            return (
              <button
                key={integration.id}
                onClick={() => !isAdded && addIntegration(integration.id)}
                disabled={isAdded}
                style={{
                  padding: '1rem',
                  backgroundColor: isAdded ? '#f5f5f5' : '#fff',
                  border: '1px solid #e0e0e0',
                  borderRadius: '4px',
                  cursor: isAdded ? 'not-allowed' : 'pointer',
                  textAlign: 'center',
                  opacity: isAdded ? 0.6 : 1,
                }}
              >
                <div style={{ fontSize: '2rem', marginBottom: '0.5rem' }} aria-hidden="true">
                  {integration.icon}
                </div>
                <div style={{ fontWeight: 500 }}>{integration.name}</div>
                <div style={{ fontSize: '0.75rem', color: '#666', marginTop: '0.25rem' }}>
                  {isAdded ? 'Added' : 'Click to add'}
                </div>
              </button>
            );
          })}
        </div>
      </section>

      {/* Active Integrations */}
      {formState.values.integrations.length > 0 && (
        <section
          style={{
            backgroundColor: '#fff',
            borderRadius: '8px',
            padding: '1.5rem',
            marginBottom: '1.5rem',
            border: '1px solid #e0e0e0',
          }}
        >
          <h3 style={{ fontSize: '1.125rem', marginBottom: '1rem' }}>Active Integrations</h3>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
            {formState.values.integrations.map((integration, index) => (
              <div
                key={integration.id}
                style={{
                  padding: '1rem',
                  border: '1px solid #e0e0e0',
                  borderRadius: '4px',
                }}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'start', marginBottom: '1rem' }}>
                  <div>
                    <div style={{ fontWeight: 600, fontSize: '1.125rem', marginBottom: '0.25rem' }}>
                      {integration.name}
                    </div>
                    <div style={{ fontSize: '0.875rem', color: '#666' }}>
                      Status:{' '}
                      <span
                        style={{
                          color:
                            integration.status === 'connected'
                              ? '#4caf50'
                              : integration.status === 'error'
                              ? '#d32f2f'
                              : '#999',
                        }}
                      >
                        {integration.status}
                      </span>
                    </div>
                  </div>
                  <div style={{ display: 'flex', gap: '0.5rem' }}>
                    <button
                      onClick={() => testIntegration(integration.id)}
                      disabled={testingIntegration === integration.id}
                      style={{
                        padding: '0.5rem 1rem',
                        backgroundColor: '#1976d2',
                        color: '#fff',
                        border: 'none',
                        borderRadius: '4px',
                        cursor: testingIntegration === integration.id ? 'not-allowed' : 'pointer',
                        fontSize: '0.875rem',
                      }}
                    >
                      {testingIntegration === integration.id ? 'Testing...' : 'Test'}
                    </button>
                    <button
                      onClick={() => removeIntegration(integration.id)}
                      style={{
                        padding: '0.5rem 1rem',
                        backgroundColor: '#d32f2f',
                        color: '#fff',
                        border: 'none',
                        borderRadius: '4px',
                        cursor: 'pointer',
                        fontSize: '0.875rem',
                      }}
                    >
                      Remove
                    </button>
                  </div>
                </div>

                <div style={{ marginBottom: '0.5rem' }}>
                  <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
                    <input
                      type="checkbox"
                      checked={integration.enabled}
                      onChange={(e) =>
                        updateField(`integrations.${index}.enabled`, e.target.checked)
                      }
                      style={{ marginRight: '0.5rem' }}
                    />
                    <span>Enable Integration</span>
                  </label>
                </div>

                {integration.lastSync && (
                  <div style={{ fontSize: '0.875rem', color: '#666' }}>
                    Last synced: {new Date(integration.lastSync).toLocaleString()}
                  </div>
                )}
              </div>
            ))}
          </div>
        </section>
      )}

      {/* Webhooks */}
      <section
        style={{
          backgroundColor: '#fff',
          borderRadius: '8px',
          padding: '1.5rem',
          marginBottom: '1.5rem',
          border: '1px solid #e0e0e0',
        }}
      >
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
          <h3 style={{ fontSize: '1.125rem', margin: 0 }}>Webhooks</h3>
          <button
            onClick={addWebhook}
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: '#1976d2',
              color: '#fff',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
            }}
          >
            + Add Webhook
          </button>
        </div>

        {formState.values.webhooks.length === 0 ? (
          <p style={{ color: '#666', fontSize: '0.875rem' }}>
            No webhooks configured. Click "Add Webhook" to create one.
          </p>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
            {formState.values.webhooks.map((webhook, index) => (
              <div
                key={webhook.id}
                style={{
                  padding: '1rem',
                  border: '1px solid #e0e0e0',
                  borderRadius: '4px',
                }}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'start', marginBottom: '1rem' }}>
                  <div style={{ flex: 1 }}>
                    <input
                      type="text"
                      value={webhook.name}
                      onChange={(e) => updateField(`webhooks.${index}.name`, e.target.value)}
                      placeholder="Webhook name"
                      style={{
                        width: '100%',
                        padding: '0.5rem',
                        border: '1px solid #d0d0d0',
                        borderRadius: '4px',
                        fontWeight: 600,
                        marginBottom: '0.5rem',
                      }}
                    />
                    <input
                      type="url"
                      value={webhook.url}
                      onChange={(e) => updateField(`webhooks.${index}.url`, e.target.value)}
                      placeholder="https://example.com/webhook"
                      aria-invalid={!!getFieldError(`webhooks.${index}.url`)}
                      style={{
                        width: '100%',
                        padding: '0.5rem',
                        border: `1px solid ${getFieldError(`webhooks.${index}.url`) ? '#d32f2f' : '#d0d0d0'}`,
                        borderRadius: '4px',
                        fontFamily: 'monospace',
                      }}
                    />
                    {getFieldError(`webhooks.${index}.url`) && (
                      <div role="alert" style={{ color: '#d32f2f', fontSize: '0.875rem', marginTop: '0.25rem' }}>
                        {getFieldError(`webhooks.${index}.url`)}
                      </div>
                    )}
                  </div>
                  <button
                    onClick={() => removeWebhook(webhook.id)}
                    style={{
                      marginLeft: '1rem',
                      padding: '0.5rem 1rem',
                      backgroundColor: '#d32f2f',
                      color: '#fff',
                      border: 'none',
                      borderRadius: '4px',
                      cursor: 'pointer',
                      fontSize: '0.875rem',
                    }}
                  >
                    Remove
                  </button>
                </div>

                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem' }}>
                  <div>
                    <label
                      htmlFor={`webhook-${index}-retry`}
                      style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500, fontSize: '0.875rem' }}
                    >
                      Retry Count
                    </label>
                    <input
                      id={`webhook-${index}-retry`}
                      type="number"
                      min="0"
                      max="10"
                      value={webhook.retryCount}
                      onChange={(e) => updateField(`webhooks.${index}.retryCount`, parseInt(e.target.value))}
                      style={{
                        width: '100%',
                        padding: '0.5rem',
                        border: '1px solid #d0d0d0',
                        borderRadius: '4px',
                      }}
                    />
                  </div>

                  <div>
                    <label
                      htmlFor={`webhook-${index}-timeout`}
                      style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500, fontSize: '0.875rem' }}
                    >
                      Timeout (ms)
                    </label>
                    <input
                      id={`webhook-${index}-timeout`}
                      type="number"
                      min="1000"
                      max="60000"
                      step="1000"
                      value={webhook.timeout}
                      onChange={(e) => updateField(`webhooks.${index}.timeout`, parseInt(e.target.value))}
                      style={{
                        width: '100%',
                        padding: '0.5rem',
                        border: '1px solid #d0d0d0',
                        borderRadius: '4px',
                      }}
                    />
                  </div>
                </div>

                <div style={{ marginTop: '0.5rem' }}>
                  <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
                    <input
                      type="checkbox"
                      checked={webhook.enabled}
                      onChange={(e) => updateField(`webhooks.${index}.enabled`, e.target.checked)}
                      style={{ marginRight: '0.5rem' }}
                    />
                    <span style={{ fontSize: '0.875rem' }}>Enable Webhook</span>
                  </label>
                </div>
              </div>
            ))}
          </div>
        )}
      </section>

      {/* API Limits */}
      <section
        style={{
          backgroundColor: '#fff',
          borderRadius: '8px',
          padding: '1.5rem',
          border: '1px solid #e0e0e0',
        }}
      >
        <h3 style={{ fontSize: '1.125rem', marginBottom: '1rem' }}>API Rate Limits</h3>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem', marginBottom: '1rem' }}>
          <div>
            <label
              htmlFor="rateLimitPerMinute"
              style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
            >
              Requests per Minute
            </label>
            <input
              id="rateLimitPerMinute"
              type="number"
              min="1"
              max="10000"
              value={formState.values.apiLimits.rateLimitPerMinute}
              onChange={(e) => updateField('apiLimits.rateLimitPerMinute', parseInt(e.target.value))}
              style={{
                width: '100%',
                padding: '0.5rem',
                border: '1px solid #d0d0d0',
                borderRadius: '4px',
              }}
            />
          </div>

          <div>
            <label
              htmlFor="rateLimitPerHour"
              style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
            >
              Requests per Hour
            </label>
            <input
              id="rateLimitPerHour"
              type="number"
              min="1"
              max="100000"
              value={formState.values.apiLimits.rateLimitPerHour}
              onChange={(e) => updateField('apiLimits.rateLimitPerHour', parseInt(e.target.value))}
              style={{
                width: '100%',
                padding: '0.5rem',
                border: '1px solid #d0d0d0',
                borderRadius: '4px',
              }}
            />
          </div>

          <div>
            <label
              htmlFor="maxPayloadSize"
              style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
            >
              Max Payload Size (bytes)
            </label>
            <input
              id="maxPayloadSize"
              type="number"
              min="1024"
              max="104857600"
              value={formState.values.apiLimits.maxPayloadSize}
              onChange={(e) => updateField('apiLimits.maxPayloadSize', parseInt(e.target.value))}
              style={{
                width: '100%',
                padding: '0.5rem',
                border: '1px solid #d0d0d0',
                borderRadius: '4px',
              }}
            />
            <div style={{ fontSize: '0.75rem', color: '#666', marginTop: '0.25rem' }}>
              {(formState.values.apiLimits.maxPayloadSize / 1024 / 1024).toFixed(2)} MB
            </div>
          </div>
        </div>

        <div>
          <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={formState.values.apiLimits.requireApiKey}
              onChange={(e) => updateField('apiLimits.requireApiKey', e.target.checked)}
              style={{ marginRight: '0.5rem' }}
            />
            <span>Require API Key for All Requests</span>
          </label>
        </div>
      </section>
    </div>
  );
};

export default IntegrationSettings;
