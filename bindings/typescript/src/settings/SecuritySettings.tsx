/**
 * CADDY v0.4.0 Enterprise Security Settings
 * Password policies, 2FA, SSO, session management
 */

import React, { useState, useEffect, useCallback, useRef } from 'react';
import {
  SecuritySettings as SecuritySettingsType,
  SSOProvider,
  FormState,
  ValidationError,
  UndoRedoState,
  ToastNotification,
  ConfirmationDialog,
  SettingsHistory,
} from './types';

interface SecuritySettingsProps {
  onSave: (section: string, data: SecuritySettingsType) => Promise<void>;
  onConfirm: (config: Omit<ConfirmationDialog, 'open'>) => void;
  addToast: (toast: Omit<ToastNotification, 'id'>) => void;
  addToHistory: (entry: Omit<SettingsHistory, 'id' | 'timestamp'>) => void;
}

const SecuritySettings: React.FC<SecuritySettingsProps> = ({
  onSave,
  onConfirm,
  addToast,
  addToHistory,
}) => {
  const [settings, setSettings] = useState<SecuritySettingsType>({
    id: 'security-1',
    version: 1,
    updatedAt: new Date(),
    updatedBy: 'current-user',
    passwordPolicy: {
      minLength: 12,
      maxLength: 128,
      requireUppercase: true,
      requireLowercase: true,
      requireNumbers: true,
      requireSpecialChars: true,
      preventReuse: 5,
      expirationDays: 90,
      maxAttempts: 5,
      lockoutDuration: 30,
    },
    twoFactorAuth: {
      enabled: true,
      required: false,
      methods: ['totp', 'sms'],
      gracePeriod: 7,
      trustedDeviceDuration: 30,
    },
    sso: {
      enabled: false,
      providers: [],
      allowLocalAuth: true,
      autoProvision: false,
      defaultRole: 'user',
    },
    sessionManagement: {
      timeout: 30,
      absoluteTimeout: 480,
      extendOnActivity: true,
      maxConcurrentSessions: 3,
      enforceIPBinding: false,
      secureOnly: true,
    },
    ipWhitelist: {
      enabled: false,
      allowedRanges: [],
      bypassRoles: ['admin'],
    },
    auditLog: {
      enabled: true,
      retentionDays: 365,
      logAuthEvents: true,
      logDataChanges: true,
      logApiCalls: true,
      exportFormat: 'json',
    },
  });

  const [formState, setFormState] = useState<FormState<SecuritySettingsType>>({
    values: settings,
    errors: [],
    isDirty: false,
    isSubmitting: false,
    isValid: true,
  });

  const [undoRedo, setUndoRedo] = useState<UndoRedoState<SecuritySettingsType>>({
    past: [],
    present: settings,
    future: [],
  });

  const [showSSOForm, setShowSSOForm] = useState(false);
  const [editingSSOId, setEditingSSOId] = useState<string | null>(null);
  const saveTimeoutRef = useRef<NodeJS.Timeout | undefined>(undefined);

  // Validation
  const validate = useCallback((data: SecuritySettingsType): ValidationError[] => {
    const errors: ValidationError[] = [];

    // Password policy validation
    if (data.passwordPolicy.minLength < 8) {
      errors.push({
        field: 'passwordPolicy.minLength',
        message: 'Minimum length must be at least 8 characters',
      });
    }
    if (data.passwordPolicy.minLength > data.passwordPolicy.maxLength) {
      errors.push({
        field: 'passwordPolicy.minLength',
        message: 'Minimum length cannot exceed maximum length',
      });
    }
    if (data.passwordPolicy.maxAttempts < 1) {
      errors.push({
        field: 'passwordPolicy.maxAttempts',
        message: 'Max attempts must be at least 1',
      });
    }

    // 2FA validation
    if (data.twoFactorAuth.enabled && data.twoFactorAuth.methods.length === 0) {
      errors.push({
        field: 'twoFactorAuth.methods',
        message: 'At least one 2FA method must be selected',
      });
    }

    // Session validation
    if (data.sessionManagement.timeout < 5) {
      errors.push({
        field: 'sessionManagement.timeout',
        message: 'Session timeout must be at least 5 minutes',
      });
    }
    if (data.sessionManagement.absoluteTimeout <= data.sessionManagement.timeout) {
      errors.push({
        field: 'sessionManagement.absoluteTimeout',
        message: 'Absolute timeout must be greater than session timeout',
      });
    }

    // SSO validation
    data.sso.providers.forEach((provider, index) => {
      if (!provider.name.trim()) {
        errors.push({
          field: `sso.providers.${index}.name`,
          message: 'Provider name is required',
        });
      }
      if (!provider.clientId.trim()) {
        errors.push({
          field: `sso.providers.${index}.clientId`,
          message: 'Client ID is required',
        });
      }
    });

    // IP whitelist validation
    if (data.ipWhitelist.enabled) {
      const ipRegex = /^(\d{1,3}\.){3}\d{1,3}(\/\d{1,2})?$/;
      data.ipWhitelist.allowedRanges.forEach((ip, index) => {
        if (!ipRegex.test(ip)) {
          errors.push({
            field: `ipWhitelist.allowedRanges.${index}`,
            message: `Invalid IP range: ${ip}`,
          });
        }
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
      await onSave('security', formState.values);
      addToHistory({
        section: 'Security Settings',
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

  // SSO Provider management
  const addSSOProvider = useCallback(() => {
    const newProvider: SSOProvider = {
      id: `sso-${Date.now()}`,
      type: 'oauth2',
      name: '',
      enabled: true,
      clientId: '',
      clientSecret: '',
      attributeMapping: {},
    };

    updateField('sso.providers', [...formState.values.sso.providers, newProvider]);
    setEditingSSOId(newProvider.id);
    setShowSSOForm(true);
  }, [formState.values.sso.providers, updateField]);

  const removeSSOProvider = useCallback(
    (id: string) => {
      onConfirm({
        title: 'Remove SSO Provider',
        message: 'Are you sure you want to remove this SSO provider? Users relying on this provider will lose access.',
        severity: 'error',
        confirmText: 'Remove',
        cancelText: 'Cancel',
        onConfirm: () => {
          updateField(
            'sso.providers',
            formState.values.sso.providers.filter((p) => p.id !== id)
          );
          addToast({ type: 'success', message: 'SSO provider removed' });
        },
        onCancel: () => {},
      });
    },
    [formState.values.sso.providers, updateField, onConfirm, addToast]
  );

  const getFieldError = (field: string): string | undefined => {
    return formState.errors.find((e) => e.field === field)?.message;
  };

  return (
    <div style={{ maxWidth: '800px' }}>
      <div style={{ marginBottom: '2rem' }}>
        <h2 style={{ fontSize: '1.5rem', marginBottom: '0.5rem' }}>Security Settings</h2>
        <p style={{ color: '#666', margin: 0 }}>
          Configure password policies, authentication, and security features
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

      {/* Password Policy */}
      <section
        style={{
          backgroundColor: '#fff',
          borderRadius: '8px',
          padding: '1.5rem',
          marginBottom: '1.5rem',
          border: '1px solid #e0e0e0',
        }}
      >
        <h3 style={{ fontSize: '1.125rem', marginBottom: '1rem' }}>Password Policy</h3>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem', marginBottom: '1rem' }}>
          <div>
            <label
              htmlFor="minLength"
              style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
            >
              Minimum Length
            </label>
            <input
              id="minLength"
              type="number"
              min="8"
              max="128"
              value={formState.values.passwordPolicy.minLength}
              onChange={(e) => updateField('passwordPolicy.minLength', parseInt(e.target.value))}
              aria-invalid={!!getFieldError('passwordPolicy.minLength')}
              style={{
                width: '100%',
                padding: '0.5rem',
                border: `1px solid ${getFieldError('passwordPolicy.minLength') ? '#d32f2f' : '#d0d0d0'}`,
                borderRadius: '4px',
              }}
            />
            {getFieldError('passwordPolicy.minLength') && (
              <div role="alert" style={{ color: '#d32f2f', fontSize: '0.875rem', marginTop: '0.25rem' }}>
                {getFieldError('passwordPolicy.minLength')}
              </div>
            )}
          </div>

          <div>
            <label
              htmlFor="maxLength"
              style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
            >
              Maximum Length
            </label>
            <input
              id="maxLength"
              type="number"
              min="8"
              max="256"
              value={formState.values.passwordPolicy.maxLength}
              onChange={(e) => updateField('passwordPolicy.maxLength', parseInt(e.target.value))}
              style={{
                width: '100%',
                padding: '0.5rem',
                border: '1px solid #d0d0d0',
                borderRadius: '4px',
              }}
            />
          </div>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem', marginBottom: '1rem' }}>
          <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={formState.values.passwordPolicy.requireUppercase}
              onChange={(e) => updateField('passwordPolicy.requireUppercase', e.target.checked)}
              style={{ marginRight: '0.5rem' }}
            />
            <span>Require Uppercase</span>
          </label>

          <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={formState.values.passwordPolicy.requireLowercase}
              onChange={(e) => updateField('passwordPolicy.requireLowercase', e.target.checked)}
              style={{ marginRight: '0.5rem' }}
            />
            <span>Require Lowercase</span>
          </label>

          <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={formState.values.passwordPolicy.requireNumbers}
              onChange={(e) => updateField('passwordPolicy.requireNumbers', e.target.checked)}
              style={{ marginRight: '0.5rem' }}
            />
            <span>Require Numbers</span>
          </label>

          <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={formState.values.passwordPolicy.requireSpecialChars}
              onChange={(e) => updateField('passwordPolicy.requireSpecialChars', e.target.checked)}
              style={{ marginRight: '0.5rem' }}
            />
            <span>Require Special Characters</span>
          </label>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem' }}>
          <div>
            <label
              htmlFor="preventReuse"
              style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
            >
              Prevent Password Reuse (last N passwords)
            </label>
            <input
              id="preventReuse"
              type="number"
              min="0"
              max="24"
              value={formState.values.passwordPolicy.preventReuse}
              onChange={(e) => updateField('passwordPolicy.preventReuse', parseInt(e.target.value))}
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
              htmlFor="expirationDays"
              style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
            >
              Password Expiration (days, 0 = never)
            </label>
            <input
              id="expirationDays"
              type="number"
              min="0"
              max="365"
              value={formState.values.passwordPolicy.expirationDays}
              onChange={(e) => updateField('passwordPolicy.expirationDays', parseInt(e.target.value))}
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
              htmlFor="maxAttempts"
              style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
            >
              Max Login Attempts
            </label>
            <input
              id="maxAttempts"
              type="number"
              min="1"
              max="10"
              value={formState.values.passwordPolicy.maxAttempts}
              onChange={(e) => updateField('passwordPolicy.maxAttempts', parseInt(e.target.value))}
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
              htmlFor="lockoutDuration"
              style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
            >
              Lockout Duration (minutes)
            </label>
            <input
              id="lockoutDuration"
              type="number"
              min="1"
              max="1440"
              value={formState.values.passwordPolicy.lockoutDuration}
              onChange={(e) => updateField('passwordPolicy.lockoutDuration', parseInt(e.target.value))}
              style={{
                width: '100%',
                padding: '0.5rem',
                border: '1px solid #d0d0d0',
                borderRadius: '4px',
              }}
            />
          </div>
        </div>
      </section>

      {/* Two-Factor Authentication */}
      <section
        style={{
          backgroundColor: '#fff',
          borderRadius: '8px',
          padding: '1.5rem',
          marginBottom: '1.5rem',
          border: '1px solid #e0e0e0',
        }}
      >
        <h3 style={{ fontSize: '1.125rem', marginBottom: '1rem' }}>Two-Factor Authentication</h3>

        <div style={{ marginBottom: '1rem' }}>
          <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer', marginBottom: '0.5rem' }}>
            <input
              type="checkbox"
              checked={formState.values.twoFactorAuth.enabled}
              onChange={(e) => updateField('twoFactorAuth.enabled', e.target.checked)}
              style={{ marginRight: '0.5rem' }}
            />
            <span>Enable Two-Factor Authentication</span>
          </label>

          <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={formState.values.twoFactorAuth.required}
              onChange={(e) => updateField('twoFactorAuth.required', e.target.checked)}
              disabled={!formState.values.twoFactorAuth.enabled}
              style={{ marginRight: '0.5rem' }}
            />
            <span>Require for All Users</span>
          </label>
        </div>

        {formState.values.twoFactorAuth.enabled && (
          <>
            <div style={{ marginBottom: '1rem' }}>
              <label style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}>
                Allowed Methods
              </label>
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '0.5rem' }}>
                {['totp', 'sms', 'email', 'hardware'].map((method) => (
                  <label key={method} style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
                    <input
                      type="checkbox"
                      checked={formState.values.twoFactorAuth.methods.includes(method as any)}
                      onChange={(e) => {
                        const methods = e.target.checked
                          ? [...formState.values.twoFactorAuth.methods, method as any]
                          : formState.values.twoFactorAuth.methods.filter((m) => m !== method);
                        updateField('twoFactorAuth.methods', methods);
                      }}
                      style={{ marginRight: '0.5rem' }}
                    />
                    <span style={{ textTransform: 'uppercase' }}>{method}</span>
                  </label>
                ))}
              </div>
              {getFieldError('twoFactorAuth.methods') && (
                <div role="alert" style={{ color: '#d32f2f', fontSize: '0.875rem', marginTop: '0.25rem' }}>
                  {getFieldError('twoFactorAuth.methods')}
                </div>
              )}
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem' }}>
              <div>
                <label
                  htmlFor="gracePeriod"
                  style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
                >
                  Grace Period (days)
                </label>
                <input
                  id="gracePeriod"
                  type="number"
                  min="0"
                  max="30"
                  value={formState.values.twoFactorAuth.gracePeriod}
                  onChange={(e) => updateField('twoFactorAuth.gracePeriod', parseInt(e.target.value))}
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
                  htmlFor="trustedDeviceDuration"
                  style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
                >
                  Trusted Device Duration (days)
                </label>
                <input
                  id="trustedDeviceDuration"
                  type="number"
                  min="0"
                  max="90"
                  value={formState.values.twoFactorAuth.trustedDeviceDuration}
                  onChange={(e) => updateField('twoFactorAuth.trustedDeviceDuration', parseInt(e.target.value))}
                  style={{
                    width: '100%',
                    padding: '0.5rem',
                    border: '1px solid #d0d0d0',
                    borderRadius: '4px',
                  }}
                />
              </div>
            </div>
          </>
        )}
      </section>

      {/* Session Management */}
      <section
        style={{
          backgroundColor: '#fff',
          borderRadius: '8px',
          padding: '1.5rem',
          marginBottom: '1.5rem',
          border: '1px solid #e0e0e0',
        }}
      >
        <h3 style={{ fontSize: '1.125rem', marginBottom: '1rem' }}>Session Management</h3>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem', marginBottom: '1rem' }}>
          <div>
            <label
              htmlFor="timeout"
              style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
            >
              Idle Timeout (minutes)
            </label>
            <input
              id="timeout"
              type="number"
              min="5"
              max="1440"
              value={formState.values.sessionManagement.timeout}
              onChange={(e) => updateField('sessionManagement.timeout', parseInt(e.target.value))}
              aria-invalid={!!getFieldError('sessionManagement.timeout')}
              style={{
                width: '100%',
                padding: '0.5rem',
                border: `1px solid ${getFieldError('sessionManagement.timeout') ? '#d32f2f' : '#d0d0d0'}`,
                borderRadius: '4px',
              }}
            />
            {getFieldError('sessionManagement.timeout') && (
              <div role="alert" style={{ color: '#d32f2f', fontSize: '0.875rem', marginTop: '0.25rem' }}>
                {getFieldError('sessionManagement.timeout')}
              </div>
            )}
          </div>

          <div>
            <label
              htmlFor="absoluteTimeout"
              style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
            >
              Absolute Timeout (minutes)
            </label>
            <input
              id="absoluteTimeout"
              type="number"
              min="30"
              max="2880"
              value={formState.values.sessionManagement.absoluteTimeout}
              onChange={(e) => updateField('sessionManagement.absoluteTimeout', parseInt(e.target.value))}
              aria-invalid={!!getFieldError('sessionManagement.absoluteTimeout')}
              style={{
                width: '100%',
                padding: '0.5rem',
                border: `1px solid ${getFieldError('sessionManagement.absoluteTimeout') ? '#d32f2f' : '#d0d0d0'}`,
                borderRadius: '4px',
              }}
            />
            {getFieldError('sessionManagement.absoluteTimeout') && (
              <div role="alert" style={{ color: '#d32f2f', fontSize: '0.875rem', marginTop: '0.25rem' }}>
                {getFieldError('sessionManagement.absoluteTimeout')}
              </div>
            )}
          </div>

          <div>
            <label
              htmlFor="maxConcurrentSessions"
              style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
            >
              Max Concurrent Sessions
            </label>
            <input
              id="maxConcurrentSessions"
              type="number"
              min="1"
              max="10"
              value={formState.values.sessionManagement.maxConcurrentSessions}
              onChange={(e) => updateField('sessionManagement.maxConcurrentSessions', parseInt(e.target.value))}
              style={{
                width: '100%',
                padding: '0.5rem',
                border: '1px solid #d0d0d0',
                borderRadius: '4px',
              }}
            />
          </div>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '0.5rem' }}>
          <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={formState.values.sessionManagement.extendOnActivity}
              onChange={(e) => updateField('sessionManagement.extendOnActivity', e.target.checked)}
              style={{ marginRight: '0.5rem' }}
            />
            <span>Extend on Activity</span>
          </label>

          <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={formState.values.sessionManagement.enforceIPBinding}
              onChange={(e) => updateField('sessionManagement.enforceIPBinding', e.target.checked)}
              style={{ marginRight: '0.5rem' }}
            />
            <span>Enforce IP Binding</span>
          </label>

          <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={formState.values.sessionManagement.secureOnly}
              onChange={(e) => updateField('sessionManagement.secureOnly', e.target.checked)}
              style={{ marginRight: '0.5rem' }}
            />
            <span>Secure Cookies Only (HTTPS)</span>
          </label>
        </div>
      </section>

      {/* SSO Configuration */}
      <section
        style={{
          backgroundColor: '#fff',
          borderRadius: '8px',
          padding: '1.5rem',
          marginBottom: '1.5rem',
          border: '1px solid #e0e0e0',
        }}
      >
        <h3 style={{ fontSize: '1.125rem', marginBottom: '1rem' }}>Single Sign-On (SSO)</h3>

        <div style={{ marginBottom: '1rem' }}>
          <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer', marginBottom: '0.5rem' }}>
            <input
              type="checkbox"
              checked={formState.values.sso.enabled}
              onChange={(e) => updateField('sso.enabled', e.target.checked)}
              style={{ marginRight: '0.5rem' }}
            />
            <span>Enable SSO</span>
          </label>

          {formState.values.sso.enabled && (
            <>
              <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer', marginBottom: '0.5rem' }}>
                <input
                  type="checkbox"
                  checked={formState.values.sso.allowLocalAuth}
                  onChange={(e) => updateField('sso.allowLocalAuth', e.target.checked)}
                  style={{ marginRight: '0.5rem' }}
                />
                <span>Allow Local Authentication</span>
              </label>

              <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
                <input
                  type="checkbox"
                  checked={formState.values.sso.autoProvision}
                  onChange={(e) => updateField('sso.autoProvision', e.target.checked)}
                  style={{ marginRight: '0.5rem' }}
                />
                <span>Auto-Provision Users</span>
              </label>
            </>
          )}
        </div>

        {formState.values.sso.enabled && (
          <>
            <div style={{ marginBottom: '1rem' }}>
              <button
                onClick={addSSOProvider}
                style={{
                  padding: '0.5rem 1rem',
                  backgroundColor: '#1976d2',
                  color: '#fff',
                  border: 'none',
                  borderRadius: '4px',
                  cursor: 'pointer',
                }}
              >
                + Add SSO Provider
              </button>
            </div>

            {formState.values.sso.providers.length > 0 && (
              <div style={{ marginTop: '1rem' }}>
                {formState.values.sso.providers.map((provider) => (
                  <div
                    key={provider.id}
                    style={{
                      padding: '1rem',
                      border: '1px solid #e0e0e0',
                      borderRadius: '4px',
                      marginBottom: '1rem',
                    }}
                  >
                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                      <div>
                        <strong>{provider.name || 'Unnamed Provider'}</strong>
                        <span style={{ marginLeft: '0.5rem', color: '#666', fontSize: '0.875rem' }}>
                          ({provider.type.toUpperCase()})
                        </span>
                      </div>
                      <button
                        onClick={() => removeSSOProvider(provider.id)}
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
                  </div>
                ))}
              </div>
            )}
          </>
        )}
      </section>

      {/* Audit Log */}
      <section
        style={{
          backgroundColor: '#fff',
          borderRadius: '8px',
          padding: '1.5rem',
          border: '1px solid #e0e0e0',
        }}
      >
        <h3 style={{ fontSize: '1.125rem', marginBottom: '1rem' }}>Audit Logging</h3>

        <div style={{ marginBottom: '1rem' }}>
          <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={formState.values.auditLog.enabled}
              onChange={(e) => updateField('auditLog.enabled', e.target.checked)}
              style={{ marginRight: '0.5rem' }}
            />
            <span>Enable Audit Logging</span>
          </label>
        </div>

        {formState.values.auditLog.enabled && (
          <>
            <div style={{ marginBottom: '1rem' }}>
              <label
                htmlFor="retentionDays"
                style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
              >
                Retention Period (days)
              </label>
              <input
                id="retentionDays"
                type="number"
                min="30"
                max="2555"
                value={formState.values.auditLog.retentionDays}
                onChange={(e) => updateField('auditLog.retentionDays', parseInt(e.target.value))}
                style={{
                  width: '200px',
                  padding: '0.5rem',
                  border: '1px solid #d0d0d0',
                  borderRadius: '4px',
                }}
              />
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '0.5rem', marginBottom: '1rem' }}>
              <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
                <input
                  type="checkbox"
                  checked={formState.values.auditLog.logAuthEvents}
                  onChange={(e) => updateField('auditLog.logAuthEvents', e.target.checked)}
                  style={{ marginRight: '0.5rem' }}
                />
                <span>Log Authentication Events</span>
              </label>

              <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
                <input
                  type="checkbox"
                  checked={formState.values.auditLog.logDataChanges}
                  onChange={(e) => updateField('auditLog.logDataChanges', e.target.checked)}
                  style={{ marginRight: '0.5rem' }}
                />
                <span>Log Data Changes</span>
              </label>

              <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
                <input
                  type="checkbox"
                  checked={formState.values.auditLog.logApiCalls}
                  onChange={(e) => updateField('auditLog.logApiCalls', e.target.checked)}
                  style={{ marginRight: '0.5rem' }}
                />
                <span>Log API Calls</span>
              </label>
            </div>

            <div>
              <label
                htmlFor="exportFormat"
                style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}
              >
                Export Format
              </label>
              <select
                id="exportFormat"
                value={formState.values.auditLog.exportFormat}
                onChange={(e) => updateField('auditLog.exportFormat', e.target.value as 'json' | 'csv' | 'syslog')}
                style={{
                  padding: '0.5rem',
                  border: '1px solid #d0d0d0',
                  borderRadius: '4px',
                }}
              >
                <option value="json">JSON</option>
                <option value="csv">CSV</option>
                <option value="syslog">Syslog</option>
              </select>
            </div>
          </>
        )}
      </section>
    </div>
  );
};

export default SecuritySettings;
