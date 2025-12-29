import React, { useState, useEffect, useCallback, useRef } from 'react';
const TIMEZONES = [
    'UTC',
    'America/New_York',
    'America/Chicago',
    'America/Denver',
    'America/Los_Angeles',
    'Europe/London',
    'Europe/Paris',
    'Asia/Tokyo',
    'Asia/Shanghai',
    'Australia/Sydney',
];
const LOCALES = [
    { code: 'en-US', name: 'English (US)' },
    { code: 'en-GB', name: 'English (UK)' },
    { code: 'es-ES', name: 'Spanish' },
    { code: 'fr-FR', name: 'French' },
    { code: 'de-DE', name: 'German' },
    { code: 'ja-JP', name: 'Japanese' },
    { code: 'zh-CN', name: 'Chinese (Simplified)' },
];
const DATE_FORMATS = [
    'MM/DD/YYYY',
    'DD/MM/YYYY',
    'YYYY-MM-DD',
    'MMM DD, YYYY',
    'DD MMM YYYY',
];
const CURRENCIES = [
    { code: 'USD', symbol: '$', name: 'US Dollar' },
    { code: 'EUR', symbol: '€', name: 'Euro' },
    { code: 'GBP', symbol: '£', name: 'British Pound' },
    { code: 'JPY', symbol: '¥', name: 'Japanese Yen' },
    { code: 'CNY', symbol: '¥', name: 'Chinese Yuan' },
];
const GeneralSettings = ({ onSave, onConfirm, addToast, addToHistory, }) => {
    const [settings, setSettings] = useState({
        id: 'general-1',
        version: 1,
        updatedAt: new Date(),
        updatedBy: 'current-user',
        appName: 'CADDY Enterprise',
        description: 'Enterprise-grade application platform',
        timezone: 'UTC',
        locale: 'en-US',
        dateFormat: 'MM/DD/YYYY',
        timeFormat: '12h',
        currency: 'USD',
        branding: {
            logo: '/assets/logo.svg',
            favicon: '/assets/favicon.ico',
            primaryColor: '#1976d2',
            secondaryColor: '#dc004e',
            accentColor: '#ff9800',
        },
        maintenance: {
            enabled: false,
            message: 'System maintenance in progress',
            allowedIPs: [],
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
    const saveTimeoutRef = useRef(undefined);
    const fileInputRef = useRef(null);
    const validate = useCallback((data) => {
        const errors = [];
        if (!data.appName || data.appName.trim().length === 0) {
            errors.push({ field: 'appName', message: 'App name is required' });
        }
        else if (data.appName.length > 100) {
            errors.push({ field: 'appName', message: 'App name must be 100 characters or less' });
        }
        if (data.description && data.description.length > 500) {
            errors.push({ field: 'description', message: 'Description must be 500 characters or less' });
        }
        if (!TIMEZONES.includes(data.timezone)) {
            errors.push({ field: 'timezone', message: 'Invalid timezone' });
        }
        if (!LOCALES.find((l) => l.code === data.locale)) {
            errors.push({ field: 'locale', message: 'Invalid locale' });
        }
        const colorRegex = /^#[0-9A-Fa-f]{6}$/;
        if (!colorRegex.test(data.branding.primaryColor)) {
            errors.push({ field: 'primaryColor', message: 'Invalid primary color format' });
        }
        if (!colorRegex.test(data.branding.secondaryColor)) {
            errors.push({ field: 'secondaryColor', message: 'Invalid secondary color format' });
        }
        if (!colorRegex.test(data.branding.accentColor)) {
            errors.push({ field: 'accentColor', message: 'Invalid accent color format' });
        }
        const ipRegex = /^(\d{1,3}\.){3}\d{1,3}(\/\d{1,2})?$/;
        data.maintenance.allowedIPs.forEach((ip, index) => {
            if (!ipRegex.test(ip)) {
                errors.push({
                    field: `allowedIPs.${index}`,
                    message: `Invalid IP address: ${ip}`,
                });
            }
        });
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
    useEffect(() => {
        const handleKeyDown = (e) => {
            if ((e.ctrlKey || e.metaKey) && e.key === 'z' && !e.shiftKey) {
                e.preventDefault();
                undo();
            }
            else if ((e.ctrlKey || e.metaKey) && (e.key === 'y' || (e.shiftKey && e.key === 'z'))) {
                e.preventDefault();
                redo();
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [undo, redo]);
    const handleSave = useCallback(async () => {
        const errors = validate(formState.values);
        if (errors.length > 0) {
            setFormState((prev) => ({ ...prev, errors, isValid: false }));
            addToast({ type: 'error', message: 'Please fix validation errors' });
            return;
        }
        setFormState((prev) => ({ ...prev, isSubmitting: true }));
        try {
            await onSave('general', formState.values);
            addToHistory({
                section: 'General Settings',
                action: 'update',
                changes: [],
                userId: 'current-user',
                userName: 'Current User',
            });
            setFormState((prev) => ({
                ...prev,
                isDirty: false,
                isSubmitting: false,
            }));
        }
        catch (error) {
            setFormState((prev) => ({ ...prev, isSubmitting: false }));
            addToast({
                type: 'error',
                message: error instanceof Error ? error.message : 'Save failed',
            });
        }
    }, [formState.values, validate, onSave, addToast, addToHistory]);
    const handleLogoUpload = useCallback((e) => {
        const file = e.target.files?.[0];
        if (!file)
            return;
        if (!file.type.startsWith('image/')) {
            addToast({ type: 'error', message: 'Please select an image file' });
            return;
        }
        if (file.size > 5 * 1024 * 1024) {
            addToast({ type: 'error', message: 'File size must be less than 5MB' });
            return;
        }
        const reader = new FileReader();
        reader.onload = (event) => {
            updateField('branding.logo', event.target?.result);
            addToast({ type: 'success', message: 'Logo uploaded successfully' });
        };
        reader.readAsDataURL(file);
    }, [updateField, addToast]);
    const handleReset = useCallback(() => {
        onConfirm({
            title: 'Reset to Defaults',
            message: 'Are you sure you want to reset all general settings to default values? This action cannot be undone.',
            severity: 'warning',
            confirmText: 'Reset',
            cancelText: 'Cancel',
            onConfirm: () => {
                const defaultSettings = {
                    id: 'general-1',
                    version: 1,
                    updatedAt: new Date(),
                    updatedBy: 'current-user',
                    appName: 'CADDY Enterprise',
                    description: 'Enterprise-grade application platform',
                    timezone: 'UTC',
                    locale: 'en-US',
                    dateFormat: 'MM/DD/YYYY',
                    timeFormat: '12h',
                    currency: 'USD',
                    branding: {
                        logo: '/assets/logo.svg',
                        favicon: '/assets/favicon.ico',
                        primaryColor: '#1976d2',
                        secondaryColor: '#dc004e',
                        accentColor: '#ff9800',
                    },
                    maintenance: {
                        enabled: false,
                        message: 'System maintenance in progress',
                        allowedIPs: [],
                    },
                };
                setFormState({
                    values: defaultSettings,
                    errors: [],
                    isDirty: true,
                    isSubmitting: false,
                    isValid: true,
                });
                addToast({ type: 'success', message: 'Settings reset to defaults' });
            },
            onCancel: () => { },
        });
    }, [onConfirm, addToast]);
    const getFieldError = (field) => {
        return formState.errors.find((e) => e.field === field)?.message;
    };
    return (React.createElement("div", { style: { maxWidth: '800px' } },
        React.createElement("div", { style: { marginBottom: '2rem' } },
            React.createElement("h2", { style: { fontSize: '1.5rem', marginBottom: '0.5rem' } }, "General Settings"),
            React.createElement("p", { style: { color: '#666', margin: 0 } }, "Configure application name, timezone, locale, and branding")),
        React.createElement("div", { style: {
                marginBottom: '1.5rem',
                display: 'flex',
                gap: '0.5rem',
                padding: '0.75rem',
                backgroundColor: '#f5f5f5',
                borderRadius: '4px',
            } },
            React.createElement("button", { onClick: undo, disabled: undoRedo.past.length === 0, "aria-label": "Undo (Ctrl+Z)", title: "Undo (Ctrl+Z)", style: {
                    padding: '0.5rem 1rem',
                    backgroundColor: '#fff',
                    border: '1px solid #e0e0e0',
                    borderRadius: '4px',
                    cursor: undoRedo.past.length === 0 ? 'not-allowed' : 'pointer',
                    opacity: undoRedo.past.length === 0 ? 0.5 : 1,
                } }, "\u21B6 Undo"),
            React.createElement("button", { onClick: redo, disabled: undoRedo.future.length === 0, "aria-label": "Redo (Ctrl+Y)", title: "Redo (Ctrl+Y)", style: {
                    padding: '0.5rem 1rem',
                    backgroundColor: '#fff',
                    border: '1px solid #e0e0e0',
                    borderRadius: '4px',
                    cursor: undoRedo.future.length === 0 ? 'not-allowed' : 'pointer',
                    opacity: undoRedo.future.length === 0 ? 0.5 : 1,
                } }, "\u21B7 Redo"),
            React.createElement("button", { onClick: handleReset, style: {
                    marginLeft: 'auto',
                    padding: '0.5rem 1rem',
                    backgroundColor: '#fff',
                    border: '1px solid #e0e0e0',
                    borderRadius: '4px',
                    cursor: 'pointer',
                } }, "Reset to Defaults")),
        React.createElement("section", { style: {
                backgroundColor: '#fff',
                borderRadius: '8px',
                padding: '1.5rem',
                marginBottom: '1.5rem',
                border: '1px solid #e0e0e0',
            } },
            React.createElement("h3", { style: { fontSize: '1.125rem', marginBottom: '1rem' } }, "Basic Information"),
            React.createElement("div", { style: { marginBottom: '1rem' } },
                React.createElement("label", { htmlFor: "appName", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } },
                    "Application Name ",
                    React.createElement("span", { style: { color: '#d32f2f' } }, "*")),
                React.createElement("input", { id: "appName", type: "text", value: formState.values.appName, onChange: (e) => updateField('appName', e.target.value), "aria-invalid": !!getFieldError('appName'), "aria-describedby": getFieldError('appName') ? 'appName-error' : undefined, style: {
                        width: '100%',
                        padding: '0.5rem',
                        border: `1px solid ${getFieldError('appName') ? '#d32f2f' : '#d0d0d0'}`,
                        borderRadius: '4px',
                        fontSize: '1rem',
                    } }),
                getFieldError('appName') && (React.createElement("div", { id: "appName-error", role: "alert", style: { color: '#d32f2f', fontSize: '0.875rem', marginTop: '0.25rem' } }, getFieldError('appName')))),
            React.createElement("div", { style: { marginBottom: '1rem' } },
                React.createElement("label", { htmlFor: "description", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Description"),
                React.createElement("textarea", { id: "description", value: formState.values.description, onChange: (e) => updateField('description', e.target.value), "aria-invalid": !!getFieldError('description'), "aria-describedby": getFieldError('description') ? 'description-error' : undefined, rows: 3, style: {
                        width: '100%',
                        padding: '0.5rem',
                        border: `1px solid ${getFieldError('description') ? '#d32f2f' : '#d0d0d0'}`,
                        borderRadius: '4px',
                        fontSize: '1rem',
                        fontFamily: 'inherit',
                    } }),
                getFieldError('description') && (React.createElement("div", { id: "description-error", role: "alert", style: { color: '#d32f2f', fontSize: '0.875rem', marginTop: '0.25rem' } }, getFieldError('description'))))),
        React.createElement("section", { style: {
                backgroundColor: '#fff',
                borderRadius: '8px',
                padding: '1.5rem',
                marginBottom: '1.5rem',
                border: '1px solid #e0e0e0',
            } },
            React.createElement("h3", { style: { fontSize: '1.125rem', marginBottom: '1rem' } }, "Regional Settings"),
            React.createElement("div", { style: { display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem' } },
                React.createElement("div", null,
                    React.createElement("label", { htmlFor: "timezone", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Timezone"),
                    React.createElement("select", { id: "timezone", value: formState.values.timezone, onChange: (e) => updateField('timezone', e.target.value), style: {
                            width: '100%',
                            padding: '0.5rem',
                            border: '1px solid #d0d0d0',
                            borderRadius: '4px',
                            fontSize: '1rem',
                        } }, TIMEZONES.map((tz) => (React.createElement("option", { key: tz, value: tz }, tz))))),
                React.createElement("div", null,
                    React.createElement("label", { htmlFor: "locale", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Locale"),
                    React.createElement("select", { id: "locale", value: formState.values.locale, onChange: (e) => updateField('locale', e.target.value), style: {
                            width: '100%',
                            padding: '0.5rem',
                            border: '1px solid #d0d0d0',
                            borderRadius: '4px',
                            fontSize: '1rem',
                        } }, LOCALES.map((locale) => (React.createElement("option", { key: locale.code, value: locale.code }, locale.name))))),
                React.createElement("div", null,
                    React.createElement("label", { htmlFor: "dateFormat", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Date Format"),
                    React.createElement("select", { id: "dateFormat", value: formState.values.dateFormat, onChange: (e) => updateField('dateFormat', e.target.value), style: {
                            width: '100%',
                            padding: '0.5rem',
                            border: '1px solid #d0d0d0',
                            borderRadius: '4px',
                            fontSize: '1rem',
                        } }, DATE_FORMATS.map((format) => (React.createElement("option", { key: format, value: format }, format))))),
                React.createElement("div", null,
                    React.createElement("label", { htmlFor: "timeFormat", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Time Format"),
                    React.createElement("select", { id: "timeFormat", value: formState.values.timeFormat, onChange: (e) => updateField('timeFormat', e.target.value), style: {
                            width: '100%',
                            padding: '0.5rem',
                            border: '1px solid #d0d0d0',
                            borderRadius: '4px',
                            fontSize: '1rem',
                        } },
                        React.createElement("option", { value: "12h" }, "12-hour"),
                        React.createElement("option", { value: "24h" }, "24-hour"))),
                React.createElement("div", null,
                    React.createElement("label", { htmlFor: "currency", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Currency"),
                    React.createElement("select", { id: "currency", value: formState.values.currency, onChange: (e) => updateField('currency', e.target.value), style: {
                            width: '100%',
                            padding: '0.5rem',
                            border: '1px solid #d0d0d0',
                            borderRadius: '4px',
                            fontSize: '1rem',
                        } }, CURRENCIES.map((currency) => (React.createElement("option", { key: currency.code, value: currency.code },
                        currency.symbol,
                        " ",
                        currency.name))))))),
        React.createElement("section", { style: {
                backgroundColor: '#fff',
                borderRadius: '8px',
                padding: '1.5rem',
                marginBottom: '1.5rem',
                border: '1px solid #e0e0e0',
            } },
            React.createElement("h3", { style: { fontSize: '1.125rem', marginBottom: '1rem' } }, "Branding"),
            React.createElement("div", { style: { marginBottom: '1rem' } },
                React.createElement("label", { style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Logo"),
                React.createElement("div", { style: { display: 'flex', alignItems: 'center', gap: '1rem' } },
                    formState.values.branding.logo && (React.createElement("img", { src: formState.values.branding.logo, alt: "Current logo", style: { width: '100px', height: '100px', objectFit: 'contain' } })),
                    React.createElement("input", { ref: fileInputRef, type: "file", accept: "image/*", onChange: handleLogoUpload, style: { display: 'none' }, "aria-label": "Upload logo" }),
                    React.createElement("button", { onClick: () => fileInputRef.current?.click(), style: {
                            padding: '0.5rem 1rem',
                            backgroundColor: '#1976d2',
                            color: '#fff',
                            border: 'none',
                            borderRadius: '4px',
                            cursor: 'pointer',
                        } }, "Upload Logo"))),
            React.createElement("div", { style: { display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: '1rem' } },
                React.createElement("div", null,
                    React.createElement("label", { htmlFor: "primaryColor", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Primary Color"),
                    React.createElement("div", { style: { display: 'flex', gap: '0.5rem' } },
                        React.createElement("input", { id: "primaryColor", type: "color", value: formState.values.branding.primaryColor, onChange: (e) => updateField('branding.primaryColor', e.target.value), style: { width: '50px', height: '38px', border: '1px solid #d0d0d0', cursor: 'pointer' } }),
                        React.createElement("input", { type: "text", value: formState.values.branding.primaryColor, onChange: (e) => updateField('branding.primaryColor', e.target.value), pattern: "^#[0-9A-Fa-f]{6}$", style: {
                                flex: 1,
                                padding: '0.5rem',
                                border: '1px solid #d0d0d0',
                                borderRadius: '4px',
                            } }))),
                React.createElement("div", null,
                    React.createElement("label", { htmlFor: "secondaryColor", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Secondary Color"),
                    React.createElement("div", { style: { display: 'flex', gap: '0.5rem' } },
                        React.createElement("input", { id: "secondaryColor", type: "color", value: formState.values.branding.secondaryColor, onChange: (e) => updateField('branding.secondaryColor', e.target.value), style: { width: '50px', height: '38px', border: '1px solid #d0d0d0', cursor: 'pointer' } }),
                        React.createElement("input", { type: "text", value: formState.values.branding.secondaryColor, onChange: (e) => updateField('branding.secondaryColor', e.target.value), pattern: "^#[0-9A-Fa-f]{6}$", style: {
                                flex: 1,
                                padding: '0.5rem',
                                border: '1px solid #d0d0d0',
                                borderRadius: '4px',
                            } }))),
                React.createElement("div", null,
                    React.createElement("label", { htmlFor: "accentColor", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Accent Color"),
                    React.createElement("div", { style: { display: 'flex', gap: '0.5rem' } },
                        React.createElement("input", { id: "accentColor", type: "color", value: formState.values.branding.accentColor, onChange: (e) => updateField('branding.accentColor', e.target.value), style: { width: '50px', height: '38px', border: '1px solid #d0d0d0', cursor: 'pointer' } }),
                        React.createElement("input", { type: "text", value: formState.values.branding.accentColor, onChange: (e) => updateField('branding.accentColor', e.target.value), pattern: "^#[0-9A-Fa-f]{6}$", style: {
                                flex: 1,
                                padding: '0.5rem',
                                border: '1px solid #d0d0d0',
                                borderRadius: '4px',
                            } }))))),
        React.createElement("section", { style: {
                backgroundColor: '#fff',
                borderRadius: '8px',
                padding: '1.5rem',
                border: '1px solid #e0e0e0',
            } },
            React.createElement("h3", { style: { fontSize: '1.125rem', marginBottom: '1rem' } }, "Maintenance Mode"),
            React.createElement("div", { style: { marginBottom: '1rem' } },
                React.createElement("label", { style: { display: 'flex', alignItems: 'center', cursor: 'pointer' } },
                    React.createElement("input", { type: "checkbox", checked: formState.values.maintenance.enabled, onChange: (e) => updateField('maintenance.enabled', e.target.checked), style: { marginRight: '0.5rem' } }),
                    React.createElement("span", null, "Enable Maintenance Mode"))),
            formState.values.maintenance.enabled && (React.createElement(React.Fragment, null,
                React.createElement("div", { style: { marginBottom: '1rem' } },
                    React.createElement("label", { htmlFor: "maintenanceMessage", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Maintenance Message"),
                    React.createElement("textarea", { id: "maintenanceMessage", value: formState.values.maintenance.message, onChange: (e) => updateField('maintenance.message', e.target.value), rows: 2, style: {
                            width: '100%',
                            padding: '0.5rem',
                            border: '1px solid #d0d0d0',
                            borderRadius: '4px',
                            fontSize: '1rem',
                            fontFamily: 'inherit',
                        } })),
                React.createElement("div", null,
                    React.createElement("label", { htmlFor: "allowedIPs", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Allowed IP Addresses (one per line)"),
                    React.createElement("textarea", { id: "allowedIPs", value: formState.values.maintenance.allowedIPs.join('\n'), onChange: (e) => updateField('maintenance.allowedIPs', e.target.value.split('\n').filter((ip) => ip.trim())), rows: 3, placeholder: "192.168.1.1\n10.0.0.0/8", style: {
                            width: '100%',
                            padding: '0.5rem',
                            border: '1px solid #d0d0d0',
                            borderRadius: '4px',
                            fontSize: '1rem',
                            fontFamily: 'monospace',
                        } })))))));
};
export default GeneralSettings;
//# sourceMappingURL=GeneralSettings.js.map