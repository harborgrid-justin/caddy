/**
 * CADDY v0.4.0 Enterprise Notification Settings
 * Email, SMS, push, in-app notification preferences
 */

import React, { useState, useEffect, useCallback, useRef } from 'react';
import {
  NotificationSettings as NotificationSettingsType,
  NotificationChannel,
  FormState,
  ValidationError,
  UndoRedoState,
  ToastNotification,
  ConfirmationDialog,
  SettingsHistory,
} from './types';

interface NotificationSettingsProps {
  onSave: (section: string, data: NotificationSettingsType) => Promise<void>;
  onConfirm: (config: Omit<ConfirmationDialog, 'open'>) => void;
  addToast: (toast: Omit<ToastNotification, 'id'>) => void;
  addToHistory: (entry: Omit<SettingsHistory, 'id' | 'timestamp'>) => void;
}

const NotificationSettings: React.FC<NotificationSettingsProps> = ({
  onSave,
  onConfirm,
  addToast,
  addToHistory,
}) => {
  const [settings, setSettings] = useState<NotificationSettingsType>({
    id: 'notification-1',
    version: 1,
    updatedAt: new Date(),
    updatedBy: 'current-user',
    email: {
      enabled: true,
      provider: 'smtp',
      host: 'smtp.example.com',
      port: 587,
      username: '',
      password: '',
      fromAddress: 'noreply@example.com',
      fromName: 'CADDY Platform',
      useTLS: true,
      templates: {},
    },
    sms: {
      enabled: false,
      provider: 'twilio',
      fromNumber: '',
      templates: {},
    },
    push: {
      enabled: false,
      provider: 'fcm',
    },
    inApp: {
      enabled: true,
      soundEnabled: true,
      desktopEnabled: true,
      retentionDays: 30,
      maxNotifications: 100,
    },
    channels: [],
  });

  const [formState, setFormState] = useState<FormState<NotificationSettingsType>>({
    values: settings,
    errors: [],
    isDirty: false,
    isSubmitting: false,
    isValid: true,
  });

  const [undoRedo, setUndoRedo] = useState<UndoRedoState<NotificationSettingsType>>({
    past: [],
    present: settings,
    future: [],
  });

  const [testingChannel, setTestingChannel] = useState<string | null>(null);
  const saveTimeoutRef = useRef<NodeJS.Timeout | undefined>(undefined);

  // Validation
  const validate = useCallback((data: NotificationSettingsType): ValidationError[] => {
    const errors: ValidationError[] = [];

    // Email validation
    if (data.email.enabled) {
      const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
      if (!emailRegex.test(data.email.fromAddress)) {
        errors.push({
          field: 'email.fromAddress',
          message: 'Invalid email address',
        });
      }

      if (data.email.provider === 'smtp') {
        if (!data.email.host) {
          errors.push({ field: 'email.host', message: 'SMTP host is required' });
        }
        if (!data.email.port || data.email.port < 1 || data.email.port > 65535) {
          errors.push({ field: 'email.port', message: 'Invalid port number' });
        }
      }
    }

    // SMS validation
    if (data.sms.enabled) {
      const phoneRegex = /^\+?[1-9]\d{1,14}$/;
      if (!phoneRegex.test(data.sms.fromNumber.replace(/\s/g, ''))) {
        errors.push({
          field: 'sms.fromNumber',
          message: 'Invalid phone number format',
        });
      }
    }

    // In-app validation
    if (data.inApp.retentionDays < 1 || data.inApp.retentionDays > 365) {
      errors.push({
        field: 'inApp.retentionDays',
        message: 'Retention days must be between 1 and 365',
      });
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
      return {
        past: newPast,
        present: previous,
        future: [prev.present, ...prev.future],
      };
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
      return {
        past: [...prev.past, prev.present],
        present: next,
        future: newFuture,
      };
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
      await onSave('notifications', formState.values);
      addToHistory({
        section: 'Notification Settings',
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

  // Test notification
  const testNotification = useCallback(
    async (type: 'email' | 'sms' | 'push') => {
      setTestingChannel(type);

      try {
        // Simulate test notification
        await new Promise((resolve) => setTimeout(resolve, 1500));
        addToast({
          type: 'success',
          message: `Test ${type} notification sent successfully`,
        });
      } catch (error) {
        addToast({
          type: 'error',
          message: `Failed to send test ${type} notification`,
        });
      } finally {
        setTestingChannel(null);
      }
    },
    [addToast]
  );

  // Add notification channel
  const addChannel = useCallback(() => {
    const newChannel: NotificationChannel = {
      id: `channel-${Date.now()}`,
      name: 'New Channel',
      type: 'email',
      events: [],
      enabled: true,
    };

    updateField('channels', [...formState.values.channels, newChannel]);
  }, [formState.values.channels, updateField]);

  // Remove notification channel
  const removeChannel = useCallback(
    (id: string) => {
      onConfirm({
        title: 'Remove Notification Channel',
        message: 'Are you sure you want to remove this notification channel?',
        severity: 'warning',
        confirmText: 'Remove',
        cancelText: 'Cancel',
        onConfirm: () => {
          updateField(
            'channels',
            formState.values.channels.filter((c) => c.id !== id)
          );
          addToast({ type: 'success', message: 'Channel removed' });
        },
        onCancel: () => {},
      });
    },
    [formState.values.channels, updateField, onConfirm, addToast]
  );

  const getFieldError = (field: string): string | undefined => {
    return formState.errors.find((e) => e.field === field)?.message;
  };

  return (
    <div style={{ maxWidth: '800px' }}>
      <div style={{ marginBottom: '2rem' }}>
        <h2 style={{ fontSize: '1.5rem', marginBottom: '0.5rem' }}>Notification Settings</h2>
        <p style={{ color: '#666', margin: 0 }}>
          Configure email, SMS, push, and in-app notifications
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

      {/* Email Notifications */}
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
          <h3 style={{ fontSize: '1.125rem', margin: 0 }}>Email Notifications</h3>
          <button
            onClick={() => testNotification('email')}
            disabled={!formState.values.email.enabled || testingChannel === 'email'}
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: '#1976d2',
              color: '#fff',
              border: 'none',
              borderRadius: '4px',
              cursor: !formState.values.email.enabled || testingChannel === 'email' ? 'not-allowed' : 'pointer',
              opacity: !formState.values.email.enabled || testingChannel === 'email' ? 0.5 : 1,
            }}
          >
            {testingChannel === 'email' ? 'Testing...' : 'Test Email'}
          </button>
        </div>

        <div style={{ marginBottom: '1rem' }}>
          <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={formState.values.email.enabled}
              onChange={(e) => updateField('email.enabled', e.target.checked)}
              style={{ marginRight: '0.5rem' }}
            />
            <span>Enable Email Notifications</span>
          </label>
        </div>

        {formState.values.email.enabled && (
          <>
            <div style={{ marginBottom: '1rem' }}>
              <label
                htmlFor="emailProvider"
                style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
              >
                Email Provider
              </label>
              <select
                id="emailProvider"
                value={formState.values.email.provider}
                onChange={(e) => updateField('email.provider', e.target.value)}
                style={{
                  width: '100%',
                  padding: '0.5rem',
                  border: '1px solid #d0d0d0',
                  borderRadius: '4px',
                }}
              >
                <option value="smtp">SMTP</option>
                <option value="sendgrid">SendGrid</option>
                <option value="ses">Amazon SES</option>
                <option value="mailgun">Mailgun</option>
              </select>
            </div>

            {formState.values.email.provider === 'smtp' && (
              <div style={{ display: 'grid', gridTemplateColumns: '2fr 1fr', gap: '1rem', marginBottom: '1rem' }}>
                <div>
                  <label
                    htmlFor="emailHost"
                    style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
                  >
                    SMTP Host
                  </label>
                  <input
                    id="emailHost"
                    type="text"
                    value={formState.values.email.host || ''}
                    onChange={(e) => updateField('email.host', e.target.value)}
                    aria-invalid={!!getFieldError('email.host')}
                    style={{
                      width: '100%',
                      padding: '0.5rem',
                      border: `1px solid ${getFieldError('email.host') ? '#d32f2f' : '#d0d0d0'}`,
                      borderRadius: '4px',
                    }}
                  />
                  {getFieldError('email.host') && (
                    <div role="alert" style={{ color: '#d32f2f', fontSize: '0.875rem', marginTop: '0.25rem' }}>
                      {getFieldError('email.host')}
                    </div>
                  )}
                </div>

                <div>
                  <label
                    htmlFor="emailPort"
                    style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
                  >
                    Port
                  </label>
                  <input
                    id="emailPort"
                    type="number"
                    min="1"
                    max="65535"
                    value={formState.values.email.port || 587}
                    onChange={(e) => updateField('email.port', parseInt(e.target.value))}
                    aria-invalid={!!getFieldError('email.port')}
                    style={{
                      width: '100%',
                      padding: '0.5rem',
                      border: `1px solid ${getFieldError('email.port') ? '#d32f2f' : '#d0d0d0'}`,
                      borderRadius: '4px',
                    }}
                  />
                  {getFieldError('email.port') && (
                    <div role="alert" style={{ color: '#d32f2f', fontSize: '0.875rem', marginTop: '0.25rem' }}>
                      {getFieldError('email.port')}
                    </div>
                  )}
                </div>

                <div>
                  <label
                    htmlFor="emailUsername"
                    style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
                  >
                    Username
                  </label>
                  <input
                    id="emailUsername"
                    type="text"
                    value={formState.values.email.username || ''}
                    onChange={(e) => updateField('email.username', e.target.value)}
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
                    htmlFor="emailPassword"
                    style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
                  >
                    Password
                  </label>
                  <input
                    id="emailPassword"
                    type="password"
                    value={formState.values.email.password || ''}
                    onChange={(e) => updateField('email.password', e.target.value)}
                    autoComplete="new-password"
                    style={{
                      width: '100%',
                      padding: '0.5rem',
                      border: '1px solid #d0d0d0',
                      borderRadius: '4px',
                    }}
                  />
                </div>
              </div>
            )}

            {formState.values.email.provider !== 'smtp' && (
              <div style={{ marginBottom: '1rem' }}>
                <label
                  htmlFor="emailApiKey"
                  style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
                >
                  API Key
                </label>
                <input
                  id="emailApiKey"
                  type="password"
                  value={formState.values.email.apiKey || ''}
                  onChange={(e) => updateField('email.apiKey', e.target.value)}
                  autoComplete="new-password"
                  style={{
                    width: '100%',
                    padding: '0.5rem',
                    border: '1px solid #d0d0d0',
                    borderRadius: '4px',
                  }}
                />
              </div>
            )}

            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem', marginBottom: '1rem' }}>
              <div>
                <label
                  htmlFor="fromAddress"
                  style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
                >
                  From Address
                </label>
                <input
                  id="fromAddress"
                  type="email"
                  value={formState.values.email.fromAddress}
                  onChange={(e) => updateField('email.fromAddress', e.target.value)}
                  aria-invalid={!!getFieldError('email.fromAddress')}
                  style={{
                    width: '100%',
                    padding: '0.5rem',
                    border: `1px solid ${getFieldError('email.fromAddress') ? '#d32f2f' : '#d0d0d0'}`,
                    borderRadius: '4px',
                  }}
                />
                {getFieldError('email.fromAddress') && (
                  <div role="alert" style={{ color: '#d32f2f', fontSize: '0.875rem', marginTop: '0.25rem' }}>
                    {getFieldError('email.fromAddress')}
                  </div>
                )}
              </div>

              <div>
                <label
                  htmlFor="fromName"
                  style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
                >
                  From Name
                </label>
                <input
                  id="fromName"
                  type="text"
                  value={formState.values.email.fromName}
                  onChange={(e) => updateField('email.fromName', e.target.value)}
                  style={{
                    width: '100%',
                    padding: '0.5rem',
                    border: '1px solid #d0d0d0',
                    borderRadius: '4px',
                  }}
                />
              </div>
            </div>

            <div>
              <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
                <input
                  type="checkbox"
                  checked={formState.values.email.useTLS}
                  onChange={(e) => updateField('email.useTLS', e.target.checked)}
                  style={{ marginRight: '0.5rem' }}
                />
                <span>Use TLS/SSL</span>
              </label>
            </div>
          </>
        )}
      </section>

      {/* SMS Notifications */}
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
          <h3 style={{ fontSize: '1.125rem', margin: 0 }}>SMS Notifications</h3>
          <button
            onClick={() => testNotification('sms')}
            disabled={!formState.values.sms.enabled || testingChannel === 'sms'}
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: '#1976d2',
              color: '#fff',
              border: 'none',
              borderRadius: '4px',
              cursor: !formState.values.sms.enabled || testingChannel === 'sms' ? 'not-allowed' : 'pointer',
              opacity: !formState.values.sms.enabled || testingChannel === 'sms' ? 0.5 : 1,
            }}
          >
            {testingChannel === 'sms' ? 'Testing...' : 'Test SMS'}
          </button>
        </div>

        <div style={{ marginBottom: '1rem' }}>
          <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={formState.values.sms.enabled}
              onChange={(e) => updateField('sms.enabled', e.target.checked)}
              style={{ marginRight: '0.5rem' }}
            />
            <span>Enable SMS Notifications</span>
          </label>
        </div>

        {formState.values.sms.enabled && (
          <>
            <div style={{ marginBottom: '1rem' }}>
              <label
                htmlFor="smsProvider"
                style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
              >
                SMS Provider
              </label>
              <select
                id="smsProvider"
                value={formState.values.sms.provider}
                onChange={(e) => updateField('sms.provider', e.target.value)}
                style={{
                  width: '100%',
                  padding: '0.5rem',
                  border: '1px solid #d0d0d0',
                  borderRadius: '4px',
                }}
              >
                <option value="twilio">Twilio</option>
                <option value="sns">Amazon SNS</option>
                <option value="nexmo">Nexmo</option>
              </select>
            </div>

            <div style={{ marginBottom: '1rem' }}>
              <label
                htmlFor="fromNumber"
                style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
              >
                From Phone Number
              </label>
              <input
                id="fromNumber"
                type="tel"
                value={formState.values.sms.fromNumber}
                onChange={(e) => updateField('sms.fromNumber', e.target.value)}
                placeholder="+1234567890"
                aria-invalid={!!getFieldError('sms.fromNumber')}
                style={{
                  width: '100%',
                  padding: '0.5rem',
                  border: `1px solid ${getFieldError('sms.fromNumber') ? '#d32f2f' : '#d0d0d0'}`,
                  borderRadius: '4px',
                }}
              />
              {getFieldError('sms.fromNumber') && (
                <div role="alert" style={{ color: '#d32f2f', fontSize: '0.875rem', marginTop: '0.25rem' }}>
                  {getFieldError('sms.fromNumber')}
                </div>
              )}
            </div>
          </>
        )}
      </section>

      {/* Push Notifications */}
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
          <h3 style={{ fontSize: '1.125rem', margin: 0 }}>Push Notifications</h3>
          <button
            onClick={() => testNotification('push')}
            disabled={!formState.values.push.enabled || testingChannel === 'push'}
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: '#1976d2',
              color: '#fff',
              border: 'none',
              borderRadius: '4px',
              cursor: !formState.values.push.enabled || testingChannel === 'push' ? 'not-allowed' : 'pointer',
              opacity: !formState.values.push.enabled || testingChannel === 'push' ? 0.5 : 1,
            }}
          >
            {testingChannel === 'push' ? 'Testing...' : 'Test Push'}
          </button>
        </div>

        <div style={{ marginBottom: '1rem' }}>
          <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={formState.values.push.enabled}
              onChange={(e) => updateField('push.enabled', e.target.checked)}
              style={{ marginRight: '0.5rem' }}
            />
            <span>Enable Push Notifications</span>
          </label>
        </div>

        {formState.values.push.enabled && (
          <div style={{ marginBottom: '1rem' }}>
            <label
              htmlFor="pushProvider"
              style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
            >
              Push Provider
            </label>
            <select
              id="pushProvider"
              value={formState.values.push.provider}
              onChange={(e) => updateField('push.provider', e.target.value)}
              style={{
                width: '100%',
                padding: '0.5rem',
                border: '1px solid #d0d0d0',
                borderRadius: '4px',
              }}
            >
              <option value="fcm">Firebase Cloud Messaging</option>
              <option value="apns">Apple Push Notification Service</option>
              <option value="onesignal">OneSignal</option>
            </select>
          </div>
        )}
      </section>

      {/* In-App Notifications */}
      <section
        style={{
          backgroundColor: '#fff',
          borderRadius: '8px',
          padding: '1.5rem',
          marginBottom: '1.5rem',
          border: '1px solid #e0e0e0',
        }}
      >
        <h3 style={{ fontSize: '1.125rem', marginBottom: '1rem' }}>In-App Notifications</h3>

        <div style={{ marginBottom: '1rem' }}>
          <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer', marginBottom: '0.5rem' }}>
            <input
              type="checkbox"
              checked={formState.values.inApp.enabled}
              onChange={(e) => updateField('inApp.enabled', e.target.checked)}
              style={{ marginRight: '0.5rem' }}
            />
            <span>Enable In-App Notifications</span>
          </label>

          <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer', marginBottom: '0.5rem' }}>
            <input
              type="checkbox"
              checked={formState.values.inApp.soundEnabled}
              onChange={(e) => updateField('inApp.soundEnabled', e.target.checked)}
              disabled={!formState.values.inApp.enabled}
              style={{ marginRight: '0.5rem' }}
            />
            <span>Enable Sound</span>
          </label>

          <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={formState.values.inApp.desktopEnabled}
              onChange={(e) => updateField('inApp.desktopEnabled', e.target.checked)}
              disabled={!formState.values.inApp.enabled}
              style={{ marginRight: '0.5rem' }}
            />
            <span>Enable Desktop Notifications</span>
          </label>
        </div>

        {formState.values.inApp.enabled && (
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem' }}>
            <div>
              <label
                htmlFor="retentionDays"
                style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
              >
                Retention Period (days)
              </label>
              <input
                id="retentionDays"
                type="number"
                min="1"
                max="365"
                value={formState.values.inApp.retentionDays}
                onChange={(e) => updateField('inApp.retentionDays', parseInt(e.target.value))}
                aria-invalid={!!getFieldError('inApp.retentionDays')}
                style={{
                  width: '100%',
                  padding: '0.5rem',
                  border: `1px solid ${getFieldError('inApp.retentionDays') ? '#d32f2f' : '#d0d0d0'}`,
                  borderRadius: '4px',
                }}
              />
              {getFieldError('inApp.retentionDays') && (
                <div role="alert" style={{ color: '#d32f2f', fontSize: '0.875rem', marginTop: '0.25rem' }}>
                  {getFieldError('inApp.retentionDays')}
                </div>
              )}
            </div>

            <div>
              <label
                htmlFor="maxNotifications"
                style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
              >
                Max Notifications
              </label>
              <input
                id="maxNotifications"
                type="number"
                min="10"
                max="1000"
                value={formState.values.inApp.maxNotifications}
                onChange={(e) => updateField('inApp.maxNotifications', parseInt(e.target.value))}
                style={{
                  width: '100%',
                  padding: '0.5rem',
                  border: '1px solid #d0d0d0',
                  borderRadius: '4px',
                }}
              />
            </div>
          </div>
        )}
      </section>

      {/* Notification Channels */}
      <section
        style={{
          backgroundColor: '#fff',
          borderRadius: '8px',
          padding: '1.5rem',
          border: '1px solid #e0e0e0',
        }}
      >
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
          <h3 style={{ fontSize: '1.125rem', margin: 0 }}>Notification Channels</h3>
          <button
            onClick={addChannel}
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: '#1976d2',
              color: '#fff',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
            }}
          >
            + Add Channel
          </button>
        </div>

        {formState.values.channels.length === 0 ? (
          <p style={{ color: '#666', fontSize: '0.875rem' }}>
            No notification channels configured. Click "Add Channel" to create one.
          </p>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
            {formState.values.channels.map((channel) => (
              <div
                key={channel.id}
                style={{
                  padding: '1rem',
                  border: '1px solid #e0e0e0',
                  borderRadius: '4px',
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                }}
              >
                <div>
                  <div style={{ fontWeight: 600, marginBottom: '0.25rem' }}>{channel.name}</div>
                  <div style={{ fontSize: '0.875rem', color: '#666' }}>
                    Type: {channel.type} • Events: {channel.events.length}
                  </div>
                </div>
                <button
                  onClick={() => removeChannel(channel.id)}
                  style={{
                    padding: '0.25rem 0.75rem',
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
            ))}
          </div>
        )}
      </section>
    </div>
  );
};

export default NotificationSettings;
