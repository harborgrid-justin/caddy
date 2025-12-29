import React, { useState, useEffect, useCallback, useRef } from 'react';
const SecuritySettings = ({ onSave, onConfirm, addToast, addToHistory, }) => {
    const [settings, setSettings] = useState({
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
    const [formState, setFormState] = useState({
        values: settings,
        errors: [],
        isDirty: false,
        isSubmitting: false,
        isValid: true,
    });
    const [undoRedo, setUndoRedo] = useState({
        past: [],
        present: settings,
        future: [],
    });
    const [showSSOForm, setShowSSOForm] = useState(false);
    const [editingSSOId, setEditingSSOId] = useState(null);
    const saveTimeoutRef = useRef(undefined);
    const validate = useCallback((data) => {
        const errors = [];
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
        if (data.twoFactorAuth.enabled && data.twoFactorAuth.methods.length === 0) {
            errors.push({
                field: 'twoFactorAuth.methods',
                message: 'At least one 2FA method must be selected',
            });
        }
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
    const updateField = useCallback((field, value) => {
        setFormState((prev) => {
            const newValues = { ...prev.values };
            const keys = field.split('.');
            let current = newValues;
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
    }, [validate]);
    const undo = useCallback(() => {
        setUndoRedo((prev) => {
            if (prev.past.length === 0)
                return prev;
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
            if (prev.future.length === 0)
                return prev;
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
        }
        catch (error) {
            setFormState((prev) => ({ ...prev, isSubmitting: false }));
            addToast({
                type: 'error',
                message: error instanceof Error ? error.message : 'Save failed',
            });
        }
    }, [formState.values, validate, onSave, addToast, addToHistory]);
    const addSSOProvider = useCallback(() => {
        const newProvider = {
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
    const removeSSOProvider = useCallback((id) => {
        onConfirm({
            title: 'Remove SSO Provider',
            message: 'Are you sure you want to remove this SSO provider? Users relying on this provider will lose access.',
            severity: 'error',
            confirmText: 'Remove',
            cancelText: 'Cancel',
            onConfirm: () => {
                updateField('sso.providers', formState.values.sso.providers.filter((p) => p.id !== id));
                addToast({ type: 'success', message: 'SSO provider removed' });
            },
            onCancel: () => { },
        });
    }, [formState.values.sso.providers, updateField, onConfirm, addToast]);
    const getFieldError = (field) => {
        return formState.errors.find((e) => e.field === field)?.message;
    };
    return (React.createElement("div", { style: { maxWidth: '800px' } },
        React.createElement("div", { style: { marginBottom: '2rem' } },
            React.createElement("h2", { style: { fontSize: '1.5rem', marginBottom: '0.5rem' } }, "Security Settings"),
            React.createElement("p", { style: { color: '#666', margin: 0 } }, "Configure password policies, authentication, and security features")),
        React.createElement("div", { style: {
                marginBottom: '1.5rem',
                display: 'flex',
                gap: '0.5rem',
                padding: '0.75rem',
                backgroundColor: '#f5f5f5',
                borderRadius: '4px',
            } },
            React.createElement("button", { onClick: undo, disabled: undoRedo.past.length === 0, "aria-label": "Undo", style: {
                    padding: '0.5rem 1rem',
                    backgroundColor: '#fff',
                    border: '1px solid #e0e0e0',
                    borderRadius: '4px',
                    cursor: undoRedo.past.length === 0 ? 'not-allowed' : 'pointer',
                    opacity: undoRedo.past.length === 0 ? 0.5 : 1,
                } }, "\u21B6 Undo"),
            React.createElement("button", { onClick: redo, disabled: undoRedo.future.length === 0, "aria-label": "Redo", style: {
                    padding: '0.5rem 1rem',
                    backgroundColor: '#fff',
                    border: '1px solid #e0e0e0',
                    borderRadius: '4px',
                    cursor: undoRedo.future.length === 0 ? 'not-allowed' : 'pointer',
                    opacity: undoRedo.future.length === 0 ? 0.5 : 1,
                } }, "\u21B7 Redo")),
        React.createElement("section", { style: {
                backgroundColor: '#fff',
                borderRadius: '8px',
                padding: '1.5rem',
                marginBottom: '1.5rem',
                border: '1px solid #e0e0e0',
            } },
            React.createElement("h3", { style: { fontSize: '1.125rem', marginBottom: '1rem' } }, "Password Policy"),
            React.createElement("div", { style: { display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem', marginBottom: '1rem' } },
                React.createElement("div", null,
                    React.createElement("label", { htmlFor: "minLength", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Minimum Length"),
                    React.createElement("input", { id: "minLength", type: "number", min: "8", max: "128", value: formState.values.passwordPolicy.minLength, onChange: (e) => updateField('passwordPolicy.minLength', parseInt(e.target.value)), "aria-invalid": !!getFieldError('passwordPolicy.minLength'), style: {
                            width: '100%',
                            padding: '0.5rem',
                            border: `1px solid ${getFieldError('passwordPolicy.minLength') ? '#d32f2f' : '#d0d0d0'}`,
                            borderRadius: '4px',
                        } }),
                    getFieldError('passwordPolicy.minLength') && (React.createElement("div", { role: "alert", style: { color: '#d32f2f', fontSize: '0.875rem', marginTop: '0.25rem' } }, getFieldError('passwordPolicy.minLength')))),
                React.createElement("div", null,
                    React.createElement("label", { htmlFor: "maxLength", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Maximum Length"),
                    React.createElement("input", { id: "maxLength", type: "number", min: "8", max: "256", value: formState.values.passwordPolicy.maxLength, onChange: (e) => updateField('passwordPolicy.maxLength', parseInt(e.target.value)), style: {
                            width: '100%',
                            padding: '0.5rem',
                            border: '1px solid #d0d0d0',
                            borderRadius: '4px',
                        } }))),
            React.createElement("div", { style: { display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem', marginBottom: '1rem' } },
                React.createElement("label", { style: { display: 'flex', alignItems: 'center', cursor: 'pointer' } },
                    React.createElement("input", { type: "checkbox", checked: formState.values.passwordPolicy.requireUppercase, onChange: (e) => updateField('passwordPolicy.requireUppercase', e.target.checked), style: { marginRight: '0.5rem' } }),
                    React.createElement("span", null, "Require Uppercase")),
                React.createElement("label", { style: { display: 'flex', alignItems: 'center', cursor: 'pointer' } },
                    React.createElement("input", { type: "checkbox", checked: formState.values.passwordPolicy.requireLowercase, onChange: (e) => updateField('passwordPolicy.requireLowercase', e.target.checked), style: { marginRight: '0.5rem' } }),
                    React.createElement("span", null, "Require Lowercase")),
                React.createElement("label", { style: { display: 'flex', alignItems: 'center', cursor: 'pointer' } },
                    React.createElement("input", { type: "checkbox", checked: formState.values.passwordPolicy.requireNumbers, onChange: (e) => updateField('passwordPolicy.requireNumbers', e.target.checked), style: { marginRight: '0.5rem' } }),
                    React.createElement("span", null, "Require Numbers")),
                React.createElement("label", { style: { display: 'flex', alignItems: 'center', cursor: 'pointer' } },
                    React.createElement("input", { type: "checkbox", checked: formState.values.passwordPolicy.requireSpecialChars, onChange: (e) => updateField('passwordPolicy.requireSpecialChars', e.target.checked), style: { marginRight: '0.5rem' } }),
                    React.createElement("span", null, "Require Special Characters"))),
            React.createElement("div", { style: { display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem' } },
                React.createElement("div", null,
                    React.createElement("label", { htmlFor: "preventReuse", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Prevent Password Reuse (last N passwords)"),
                    React.createElement("input", { id: "preventReuse", type: "number", min: "0", max: "24", value: formState.values.passwordPolicy.preventReuse, onChange: (e) => updateField('passwordPolicy.preventReuse', parseInt(e.target.value)), style: {
                            width: '100%',
                            padding: '0.5rem',
                            border: '1px solid #d0d0d0',
                            borderRadius: '4px',
                        } })),
                React.createElement("div", null,
                    React.createElement("label", { htmlFor: "expirationDays", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Password Expiration (days, 0 = never)"),
                    React.createElement("input", { id: "expirationDays", type: "number", min: "0", max: "365", value: formState.values.passwordPolicy.expirationDays, onChange: (e) => updateField('passwordPolicy.expirationDays', parseInt(e.target.value)), style: {
                            width: '100%',
                            padding: '0.5rem',
                            border: '1px solid #d0d0d0',
                            borderRadius: '4px',
                        } })),
                React.createElement("div", null,
                    React.createElement("label", { htmlFor: "maxAttempts", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Max Login Attempts"),
                    React.createElement("input", { id: "maxAttempts", type: "number", min: "1", max: "10", value: formState.values.passwordPolicy.maxAttempts, onChange: (e) => updateField('passwordPolicy.maxAttempts', parseInt(e.target.value)), style: {
                            width: '100%',
                            padding: '0.5rem',
                            border: '1px solid #d0d0d0',
                            borderRadius: '4px',
                        } })),
                React.createElement("div", null,
                    React.createElement("label", { htmlFor: "lockoutDuration", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Lockout Duration (minutes)"),
                    React.createElement("input", { id: "lockoutDuration", type: "number", min: "1", max: "1440", value: formState.values.passwordPolicy.lockoutDuration, onChange: (e) => updateField('passwordPolicy.lockoutDuration', parseInt(e.target.value)), style: {
                            width: '100%',
                            padding: '0.5rem',
                            border: '1px solid #d0d0d0',
                            borderRadius: '4px',
                        } })))),
        React.createElement("section", { style: {
                backgroundColor: '#fff',
                borderRadius: '8px',
                padding: '1.5rem',
                marginBottom: '1.5rem',
                border: '1px solid #e0e0e0',
            } },
            React.createElement("h3", { style: { fontSize: '1.125rem', marginBottom: '1rem' } }, "Two-Factor Authentication"),
            React.createElement("div", { style: { marginBottom: '1rem' } },
                React.createElement("label", { style: { display: 'flex', alignItems: 'center', cursor: 'pointer', marginBottom: '0.5rem' } },
                    React.createElement("input", { type: "checkbox", checked: formState.values.twoFactorAuth.enabled, onChange: (e) => updateField('twoFactorAuth.enabled', e.target.checked), style: { marginRight: '0.5rem' } }),
                    React.createElement("span", null, "Enable Two-Factor Authentication")),
                React.createElement("label", { style: { display: 'flex', alignItems: 'center', cursor: 'pointer' } },
                    React.createElement("input", { type: "checkbox", checked: formState.values.twoFactorAuth.required, onChange: (e) => updateField('twoFactorAuth.required', e.target.checked), disabled: !formState.values.twoFactorAuth.enabled, style: { marginRight: '0.5rem' } }),
                    React.createElement("span", null, "Require for All Users"))),
            formState.values.twoFactorAuth.enabled && (React.createElement(React.Fragment, null,
                React.createElement("div", { style: { marginBottom: '1rem' } },
                    React.createElement("label", { style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Allowed Methods"),
                    React.createElement("div", { style: { display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '0.5rem' } }, ['totp', 'sms', 'email', 'hardware'].map((method) => (React.createElement("label", { key: method, style: { display: 'flex', alignItems: 'center', cursor: 'pointer' } },
                        React.createElement("input", { type: "checkbox", checked: formState.values.twoFactorAuth.methods.includes(method), onChange: (e) => {
                                const methods = e.target.checked
                                    ? [...formState.values.twoFactorAuth.methods, method]
                                    : formState.values.twoFactorAuth.methods.filter((m) => m !== method);
                                updateField('twoFactorAuth.methods', methods);
                            }, style: { marginRight: '0.5rem' } }),
                        React.createElement("span", { style: { textTransform: 'uppercase' } }, method))))),
                    getFieldError('twoFactorAuth.methods') && (React.createElement("div", { role: "alert", style: { color: '#d32f2f', fontSize: '0.875rem', marginTop: '0.25rem' } }, getFieldError('twoFactorAuth.methods')))),
                React.createElement("div", { style: { display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem' } },
                    React.createElement("div", null,
                        React.createElement("label", { htmlFor: "gracePeriod", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Grace Period (days)"),
                        React.createElement("input", { id: "gracePeriod", type: "number", min: "0", max: "30", value: formState.values.twoFactorAuth.gracePeriod, onChange: (e) => updateField('twoFactorAuth.gracePeriod', parseInt(e.target.value)), style: {
                                width: '100%',
                                padding: '0.5rem',
                                border: '1px solid #d0d0d0',
                                borderRadius: '4px',
                            } })),
                    React.createElement("div", null,
                        React.createElement("label", { htmlFor: "trustedDeviceDuration", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Trusted Device Duration (days)"),
                        React.createElement("input", { id: "trustedDeviceDuration", type: "number", min: "0", max: "90", value: formState.values.twoFactorAuth.trustedDeviceDuration, onChange: (e) => updateField('twoFactorAuth.trustedDeviceDuration', parseInt(e.target.value)), style: {
                                width: '100%',
                                padding: '0.5rem',
                                border: '1px solid #d0d0d0',
                                borderRadius: '4px',
                            } })))))),
        React.createElement("section", { style: {
                backgroundColor: '#fff',
                borderRadius: '8px',
                padding: '1.5rem',
                marginBottom: '1.5rem',
                border: '1px solid #e0e0e0',
            } },
            React.createElement("h3", { style: { fontSize: '1.125rem', marginBottom: '1rem' } }, "Session Management"),
            React.createElement("div", { style: { display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem', marginBottom: '1rem' } },
                React.createElement("div", null,
                    React.createElement("label", { htmlFor: "timeout", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Idle Timeout (minutes)"),
                    React.createElement("input", { id: "timeout", type: "number", min: "5", max: "1440", value: formState.values.sessionManagement.timeout, onChange: (e) => updateField('sessionManagement.timeout', parseInt(e.target.value)), "aria-invalid": !!getFieldError('sessionManagement.timeout'), style: {
                            width: '100%',
                            padding: '0.5rem',
                            border: `1px solid ${getFieldError('sessionManagement.timeout') ? '#d32f2f' : '#d0d0d0'}`,
                            borderRadius: '4px',
                        } }),
                    getFieldError('sessionManagement.timeout') && (React.createElement("div", { role: "alert", style: { color: '#d32f2f', fontSize: '0.875rem', marginTop: '0.25rem' } }, getFieldError('sessionManagement.timeout')))),
                React.createElement("div", null,
                    React.createElement("label", { htmlFor: "absoluteTimeout", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Absolute Timeout (minutes)"),
                    React.createElement("input", { id: "absoluteTimeout", type: "number", min: "30", max: "2880", value: formState.values.sessionManagement.absoluteTimeout, onChange: (e) => updateField('sessionManagement.absoluteTimeout', parseInt(e.target.value)), "aria-invalid": !!getFieldError('sessionManagement.absoluteTimeout'), style: {
                            width: '100%',
                            padding: '0.5rem',
                            border: `1px solid ${getFieldError('sessionManagement.absoluteTimeout') ? '#d32f2f' : '#d0d0d0'}`,
                            borderRadius: '4px',
                        } }),
                    getFieldError('sessionManagement.absoluteTimeout') && (React.createElement("div", { role: "alert", style: { color: '#d32f2f', fontSize: '0.875rem', marginTop: '0.25rem' } }, getFieldError('sessionManagement.absoluteTimeout')))),
                React.createElement("div", null,
                    React.createElement("label", { htmlFor: "maxConcurrentSessions", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Max Concurrent Sessions"),
                    React.createElement("input", { id: "maxConcurrentSessions", type: "number", min: "1", max: "10", value: formState.values.sessionManagement.maxConcurrentSessions, onChange: (e) => updateField('sessionManagement.maxConcurrentSessions', parseInt(e.target.value)), style: {
                            width: '100%',
                            padding: '0.5rem',
                            border: '1px solid #d0d0d0',
                            borderRadius: '4px',
                        } }))),
            React.createElement("div", { style: { display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '0.5rem' } },
                React.createElement("label", { style: { display: 'flex', alignItems: 'center', cursor: 'pointer' } },
                    React.createElement("input", { type: "checkbox", checked: formState.values.sessionManagement.extendOnActivity, onChange: (e) => updateField('sessionManagement.extendOnActivity', e.target.checked), style: { marginRight: '0.5rem' } }),
                    React.createElement("span", null, "Extend on Activity")),
                React.createElement("label", { style: { display: 'flex', alignItems: 'center', cursor: 'pointer' } },
                    React.createElement("input", { type: "checkbox", checked: formState.values.sessionManagement.enforceIPBinding, onChange: (e) => updateField('sessionManagement.enforceIPBinding', e.target.checked), style: { marginRight: '0.5rem' } }),
                    React.createElement("span", null, "Enforce IP Binding")),
                React.createElement("label", { style: { display: 'flex', alignItems: 'center', cursor: 'pointer' } },
                    React.createElement("input", { type: "checkbox", checked: formState.values.sessionManagement.secureOnly, onChange: (e) => updateField('sessionManagement.secureOnly', e.target.checked), style: { marginRight: '0.5rem' } }),
                    React.createElement("span", null, "Secure Cookies Only (HTTPS)")))),
        React.createElement("section", { style: {
                backgroundColor: '#fff',
                borderRadius: '8px',
                padding: '1.5rem',
                marginBottom: '1.5rem',
                border: '1px solid #e0e0e0',
            } },
            React.createElement("h3", { style: { fontSize: '1.125rem', marginBottom: '1rem' } }, "Single Sign-On (SSO)"),
            React.createElement("div", { style: { marginBottom: '1rem' } },
                React.createElement("label", { style: { display: 'flex', alignItems: 'center', cursor: 'pointer', marginBottom: '0.5rem' } },
                    React.createElement("input", { type: "checkbox", checked: formState.values.sso.enabled, onChange: (e) => updateField('sso.enabled', e.target.checked), style: { marginRight: '0.5rem' } }),
                    React.createElement("span", null, "Enable SSO")),
                formState.values.sso.enabled && (React.createElement(React.Fragment, null,
                    React.createElement("label", { style: { display: 'flex', alignItems: 'center', cursor: 'pointer', marginBottom: '0.5rem' } },
                        React.createElement("input", { type: "checkbox", checked: formState.values.sso.allowLocalAuth, onChange: (e) => updateField('sso.allowLocalAuth', e.target.checked), style: { marginRight: '0.5rem' } }),
                        React.createElement("span", null, "Allow Local Authentication")),
                    React.createElement("label", { style: { display: 'flex', alignItems: 'center', cursor: 'pointer' } },
                        React.createElement("input", { type: "checkbox", checked: formState.values.sso.autoProvision, onChange: (e) => updateField('sso.autoProvision', e.target.checked), style: { marginRight: '0.5rem' } }),
                        React.createElement("span", null, "Auto-Provision Users"))))),
            formState.values.sso.enabled && (React.createElement(React.Fragment, null,
                React.createElement("div", { style: { marginBottom: '1rem' } },
                    React.createElement("button", { onClick: addSSOProvider, style: {
                            padding: '0.5rem 1rem',
                            backgroundColor: '#1976d2',
                            color: '#fff',
                            border: 'none',
                            borderRadius: '4px',
                            cursor: 'pointer',
                        } }, "+ Add SSO Provider")),
                formState.values.sso.providers.length > 0 && (React.createElement("div", { style: { marginTop: '1rem' } }, formState.values.sso.providers.map((provider) => (React.createElement("div", { key: provider.id, style: {
                        padding: '1rem',
                        border: '1px solid #e0e0e0',
                        borderRadius: '4px',
                        marginBottom: '1rem',
                    } },
                    React.createElement("div", { style: { display: 'flex', justifyContent: 'space-between', alignItems: 'center' } },
                        React.createElement("div", null,
                            React.createElement("strong", null, provider.name || 'Unnamed Provider'),
                            React.createElement("span", { style: { marginLeft: '0.5rem', color: '#666', fontSize: '0.875rem' } },
                                "(",
                                provider.type.toUpperCase(),
                                ")")),
                        React.createElement("button", { onClick: () => removeSSOProvider(provider.id), style: {
                                padding: '0.25rem 0.75rem',
                                backgroundColor: '#d32f2f',
                                color: '#fff',
                                border: 'none',
                                borderRadius: '4px',
                                cursor: 'pointer',
                                fontSize: '0.875rem',
                            } }, "Remove")))))))))),
        React.createElement("section", { style: {
                backgroundColor: '#fff',
                borderRadius: '8px',
                padding: '1.5rem',
                border: '1px solid #e0e0e0',
            } },
            React.createElement("h3", { style: { fontSize: '1.125rem', marginBottom: '1rem' } }, "Audit Logging"),
            React.createElement("div", { style: { marginBottom: '1rem' } },
                React.createElement("label", { style: { display: 'flex', alignItems: 'center', cursor: 'pointer' } },
                    React.createElement("input", { type: "checkbox", checked: formState.values.auditLog.enabled, onChange: (e) => updateField('auditLog.enabled', e.target.checked), style: { marginRight: '0.5rem' } }),
                    React.createElement("span", null, "Enable Audit Logging"))),
            formState.values.auditLog.enabled && (React.createElement(React.Fragment, null,
                React.createElement("div", { style: { marginBottom: '1rem' } },
                    React.createElement("label", { htmlFor: "retentionDays", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Retention Period (days)"),
                    React.createElement("input", { id: "retentionDays", type: "number", min: "30", max: "2555", value: formState.values.auditLog.retentionDays, onChange: (e) => updateField('auditLog.retentionDays', parseInt(e.target.value)), style: {
                            width: '200px',
                            padding: '0.5rem',
                            border: '1px solid #d0d0d0',
                            borderRadius: '4px',
                        } })),
                React.createElement("div", { style: { display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '0.5rem', marginBottom: '1rem' } },
                    React.createElement("label", { style: { display: 'flex', alignItems: 'center', cursor: 'pointer' } },
                        React.createElement("input", { type: "checkbox", checked: formState.values.auditLog.logAuthEvents, onChange: (e) => updateField('auditLog.logAuthEvents', e.target.checked), style: { marginRight: '0.5rem' } }),
                        React.createElement("span", null, "Log Authentication Events")),
                    React.createElement("label", { style: { display: 'flex', alignItems: 'center', cursor: 'pointer' } },
                        React.createElement("input", { type: "checkbox", checked: formState.values.auditLog.logDataChanges, onChange: (e) => updateField('auditLog.logDataChanges', e.target.checked), style: { marginRight: '0.5rem' } }),
                        React.createElement("span", null, "Log Data Changes")),
                    React.createElement("label", { style: { display: 'flex', alignItems: 'center', cursor: 'pointer' } },
                        React.createElement("input", { type: "checkbox", checked: formState.values.auditLog.logApiCalls, onChange: (e) => updateField('auditLog.logApiCalls', e.target.checked), style: { marginRight: '0.5rem' } }),
                        React.createElement("span", null, "Log API Calls"))),
                React.createElement("div", null,
                    React.createElement("label", { htmlFor: "exportFormat", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Export Format"),
                    React.createElement("select", { id: "exportFormat", value: formState.values.auditLog.exportFormat, onChange: (e) => updateField('auditLog.exportFormat', e.target.value), style: {
                            padding: '0.5rem',
                            border: '1px solid #d0d0d0',
                            borderRadius: '4px',
                        } },
                        React.createElement("option", { value: "json" }, "JSON"),
                        React.createElement("option", { value: "csv" }, "CSV"),
                        React.createElement("option", { value: "syslog" }, "Syslog"))))))));
};
export default SecuritySettings;
//# sourceMappingURL=SecuritySettings.js.map