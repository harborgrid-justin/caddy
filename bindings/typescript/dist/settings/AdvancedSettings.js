import React, { useState, useCallback } from 'react';
const AdvancedSettings = ({ onSave, onConfirm, addToast, addToHistory, }) => {
    const [settings, setSettings] = useState({
        id: 'advanced-1',
        version: 1,
        updatedAt: new Date(),
        updatedBy: 'current-user',
        developerMode: false,
        apiKeys: [
            {
                id: 'key-1',
                name: 'Production API Key',
                key: 'sk_prod_AbCdEfGhIjKlMnOpQrStUvWxYz123456',
                prefix: 'sk_prod',
                permissions: ['read', 'write'],
                rateLimit: 1000,
                createdAt: new Date('2024-01-01'),
                lastUsed: new Date('2025-01-28'),
                status: 'active',
            },
        ],
        webhooks: [],
        customDomains: [],
        cors: {
            enabled: true,
            allowedOrigins: ['https://app.example.com'],
            allowedMethods: ['GET', 'POST', 'PUT', 'DELETE'],
            allowedHeaders: ['Content-Type', 'Authorization'],
            exposedHeaders: ['X-Request-ID'],
            maxAge: 86400,
            allowCredentials: true,
        },
        logging: {
            level: 'info',
            destinations: [
                { type: 'console', enabled: true, config: {} },
            ],
            retention: 30,
            maskSensitiveData: true,
            includeStackTrace: true,
        },
        performance: {
            cacheEnabled: true,
            cacheTTL: 3600,
            compressionEnabled: true,
            compressionLevel: 6,
            minifyAssets: true,
            cdnEnabled: false,
        },
    });
    const [showKeyModal, setShowKeyModal] = useState(false);
    const [newKeyName, setNewKeyName] = useState('');
    const [showNewKey, setShowNewKey] = useState(null);
    const generateAPIKey = useCallback(() => {
        if (!newKeyName.trim()) {
            addToast({ type: 'error', message: 'Please enter a key name' });
            return;
        }
        const prefix = 'sk_prod';
        const randomPart = Array.from({ length: 32 }, () => 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789'.charAt(Math.floor(Math.random() * 62))).join('');
        const fullKey = `${prefix}_${randomPart}`;
        const newKey = {
            id: `key-${Date.now()}`,
            name: newKeyName,
            key: fullKey,
            prefix,
            permissions: ['read'],
            rateLimit: 100,
            createdAt: new Date(),
            status: 'active',
        };
        setSettings((prev) => ({
            ...prev,
            apiKeys: [...prev.apiKeys, newKey],
        }));
        setShowNewKey(fullKey);
        addToast({
            type: 'success',
            message: 'API key generated successfully',
        });
        addToHistory({
            section: 'Advanced Settings',
            action: 'create',
            changes: [{ field: 'apiKey', oldValue: null, newValue: newKeyName }],
            userId: 'current-user',
            userName: 'Current User',
        });
        setNewKeyName('');
    }, [newKeyName, addToast, addToHistory]);
    const revokeAPIKey = useCallback((id) => {
        const key = settings.apiKeys.find((k) => k.id === id);
        if (!key)
            return;
        onConfirm({
            title: 'Revoke API Key',
            message: `Are you sure you want to revoke "${key.name}"? Applications using this key will no longer be able to access the API.`,
            severity: 'error',
            confirmText: 'Revoke',
            cancelText: 'Cancel',
            onConfirm: () => {
                setSettings((prev) => ({
                    ...prev,
                    apiKeys: prev.apiKeys.map((k) => k.id === id ? { ...k, status: 'revoked' } : k),
                }));
                addToast({ type: 'success', message: 'API key revoked' });
                addToHistory({
                    section: 'Advanced Settings',
                    action: 'update',
                    changes: [{ field: 'apiKey.status', oldValue: 'active', newValue: 'revoked' }],
                    userId: 'current-user',
                    userName: 'Current User',
                });
            },
            onCancel: () => { },
        });
    }, [settings.apiKeys, onConfirm, addToast, addToHistory]);
    const addCustomDomain = useCallback(() => {
        const newDomain = {
            id: `domain-${Date.now()}`,
            domain: '',
            status: 'pending',
            sslEnabled: true,
            dnsRecords: [
                { type: 'CNAME', name: 'www', value: 'app.caddy.io', verified: false },
                { type: 'TXT', name: '_caddy', value: 'verification-token-here', verified: false },
            ],
        };
        setSettings((prev) => ({
            ...prev,
            customDomains: [...prev.customDomains, newDomain],
        }));
    }, []);
    const removeCustomDomain = useCallback((id) => {
        onConfirm({
            title: 'Remove Custom Domain',
            message: 'Are you sure you want to remove this custom domain?',
            severity: 'warning',
            confirmText: 'Remove',
            cancelText: 'Cancel',
            onConfirm: () => {
                setSettings((prev) => ({
                    ...prev,
                    customDomains: prev.customDomains.filter((d) => d.id !== id),
                }));
                addToast({ type: 'success', message: 'Custom domain removed' });
            },
            onCancel: () => { },
        });
    }, [onConfirm, addToast]);
    const updateSetting = useCallback((field, value) => {
        setSettings((prev) => {
            const newSettings = { ...prev };
            const keys = field.split('.');
            let current = newSettings;
            for (let i = 0; i < keys.length - 1; i++) {
                current = current[keys[i]];
            }
            current[keys[keys.length - 1]] = value;
            return newSettings;
        });
    }, []);
    return (React.createElement("div", { style: { maxWidth: '800px' } },
        React.createElement("div", { style: { marginBottom: '2rem' } },
            React.createElement("h2", { style: { fontSize: '1.5rem', marginBottom: '0.5rem' } }, "Advanced Settings"),
            React.createElement("p", { style: { color: '#666', margin: 0 } }, "Developer options, API keys, and advanced configuration")),
        React.createElement("section", { style: {
                backgroundColor: '#fff',
                borderRadius: '8px',
                padding: '1.5rem',
                marginBottom: '1.5rem',
                border: '1px solid #e0e0e0',
            } },
            React.createElement("h3", { style: { fontSize: '1.125rem', marginBottom: '1rem' } }, "Developer Mode"),
            React.createElement("label", { style: { display: 'flex', alignItems: 'center', cursor: 'pointer' } },
                React.createElement("input", { type: "checkbox", checked: settings.developerMode, onChange: (e) => updateSetting('developerMode', e.target.checked), style: { marginRight: '0.5rem' } }),
                React.createElement("span", null, "Enable Developer Mode")),
            React.createElement("p", { style: { margin: '0.5rem 0 0 1.5rem', fontSize: '0.875rem', color: '#666' } }, "Enables additional debugging information, detailed error messages, and developer tools")),
        React.createElement("section", { style: {
                backgroundColor: '#fff',
                borderRadius: '8px',
                padding: '1.5rem',
                marginBottom: '1.5rem',
                border: '1px solid #e0e0e0',
            } },
            React.createElement("div", { style: { display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' } },
                React.createElement("h3", { style: { fontSize: '1.125rem', margin: 0 } }, "API Keys"),
                React.createElement("button", { onClick: () => setShowKeyModal(true), style: {
                        padding: '0.5rem 1rem',
                        backgroundColor: '#1976d2',
                        color: '#fff',
                        border: 'none',
                        borderRadius: '4px',
                        cursor: 'pointer',
                    } }, "+ Generate API Key")),
            settings.apiKeys.length === 0 ? (React.createElement("p", { style: { color: '#666', fontSize: '0.875rem' } }, "No API keys generated. Click \"Generate API Key\" to create one.")) : (React.createElement("div", { style: { display: 'flex', flexDirection: 'column', gap: '1rem' } }, settings.apiKeys.map((key) => (React.createElement("div", { key: key.id, style: {
                    padding: '1rem',
                    border: '1px solid #e0e0e0',
                    borderRadius: '4px',
                    opacity: key.status === 'revoked' ? 0.6 : 1,
                } },
                React.createElement("div", { style: { display: 'flex', justifyContent: 'space-between', alignItems: 'start', marginBottom: '0.5rem' } },
                    React.createElement("div", null,
                        React.createElement("div", { style: { fontWeight: 600, marginBottom: '0.25rem' } },
                            key.name,
                            key.status === 'revoked' && (React.createElement("span", { style: {
                                    marginLeft: '0.5rem',
                                    padding: '0.125rem 0.5rem',
                                    backgroundColor: '#ffebee',
                                    color: '#c62828',
                                    fontSize: '0.75rem',
                                    borderRadius: '4px',
                                } }, "Revoked"))),
                        React.createElement("code", { style: {
                                fontSize: '0.875rem',
                                padding: '0.25rem 0.5rem',
                                backgroundColor: '#f5f5f5',
                                borderRadius: '4px',
                                fontFamily: 'monospace',
                            } },
                            key.key.substring(0, 20),
                            "...",
                            key.key.substring(key.key.length - 4))),
                    key.status === 'active' && (React.createElement("button", { onClick: () => revokeAPIKey(key.id), style: {
                            padding: '0.25rem 0.75rem',
                            backgroundColor: '#fff',
                            color: '#d32f2f',
                            border: '1px solid #d32f2f',
                            borderRadius: '4px',
                            cursor: 'pointer',
                            fontSize: '0.875rem',
                        } }, "Revoke"))),
                React.createElement("div", { style: { fontSize: '0.875rem', color: '#666', marginTop: '0.5rem' } },
                    "Created: ",
                    key.createdAt.toLocaleDateString(),
                    key.lastUsed && ` • Last used: ${key.lastUsed.toLocaleDateString()}`,
                    ' • Rate limit: ',
                    key.rateLimit,
                    " req/min"))))))),
        React.createElement("section", { style: {
                backgroundColor: '#fff',
                borderRadius: '8px',
                padding: '1.5rem',
                marginBottom: '1.5rem',
                border: '1px solid #e0e0e0',
            } },
            React.createElement("h3", { style: { fontSize: '1.125rem', marginBottom: '1rem' } }, "CORS Configuration"),
            React.createElement("div", { style: { marginBottom: '1rem' } },
                React.createElement("label", { style: { display: 'flex', alignItems: 'center', cursor: 'pointer', marginBottom: '0.5rem' } },
                    React.createElement("input", { type: "checkbox", checked: settings.cors.enabled, onChange: (e) => updateSetting('cors.enabled', e.target.checked), style: { marginRight: '0.5rem' } }),
                    React.createElement("span", null, "Enable CORS")),
                React.createElement("label", { style: { display: 'flex', alignItems: 'center', cursor: 'pointer' } },
                    React.createElement("input", { type: "checkbox", checked: settings.cors.allowCredentials, onChange: (e) => updateSetting('cors.allowCredentials', e.target.checked), disabled: !settings.cors.enabled, style: { marginRight: '0.5rem' } }),
                    React.createElement("span", null, "Allow Credentials"))),
            settings.cors.enabled && (React.createElement(React.Fragment, null,
                React.createElement("div", { style: { marginBottom: '1rem' } },
                    React.createElement("label", { htmlFor: "allowedOrigins", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Allowed Origins (one per line)"),
                    React.createElement("textarea", { id: "allowedOrigins", value: settings.cors.allowedOrigins.join('\n'), onChange: (e) => updateSetting('cors.allowedOrigins', e.target.value.split('\n').filter((o) => o.trim())), rows: 4, placeholder: "https://example.com\nhttps://app.example.com", style: {
                            width: '100%',
                            padding: '0.5rem',
                            border: '1px solid #d0d0d0',
                            borderRadius: '4px',
                            fontFamily: 'monospace',
                            fontSize: '0.875rem',
                        } })),
                React.createElement("div", { style: { marginBottom: '1rem' } },
                    React.createElement("label", { htmlFor: "allowedMethods", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Allowed Methods"),
                    React.createElement("div", { style: { display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '0.5rem' } }, ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'OPTIONS'].map((method) => (React.createElement("label", { key: method, style: { display: 'flex', alignItems: 'center', cursor: 'pointer' } },
                        React.createElement("input", { type: "checkbox", checked: settings.cors.allowedMethods.includes(method), onChange: (e) => {
                                const methods = e.target.checked
                                    ? [...settings.cors.allowedMethods, method]
                                    : settings.cors.allowedMethods.filter((m) => m !== method);
                                updateSetting('cors.allowedMethods', methods);
                            }, style: { marginRight: '0.5rem' } }),
                        React.createElement("span", { style: { fontSize: '0.875rem' } }, method)))))),
                React.createElement("div", null,
                    React.createElement("label", { htmlFor: "maxAge", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Max Age (seconds)"),
                    React.createElement("input", { id: "maxAge", type: "number", min: "0", max: "86400", value: settings.cors.maxAge, onChange: (e) => updateSetting('cors.maxAge', parseInt(e.target.value)), style: {
                            width: '200px',
                            padding: '0.5rem',
                            border: '1px solid #d0d0d0',
                            borderRadius: '4px',
                        } }))))),
        React.createElement("section", { style: {
                backgroundColor: '#fff',
                borderRadius: '8px',
                padding: '1.5rem',
                marginBottom: '1.5rem',
                border: '1px solid #e0e0e0',
            } },
            React.createElement("h3", { style: { fontSize: '1.125rem', marginBottom: '1rem' } }, "Logging"),
            React.createElement("div", { style: { marginBottom: '1rem' } },
                React.createElement("label", { htmlFor: "logLevel", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Log Level"),
                React.createElement("select", { id: "logLevel", value: settings.logging.level, onChange: (e) => updateSetting('logging.level', e.target.value), style: {
                        padding: '0.5rem',
                        border: '1px solid #d0d0d0',
                        borderRadius: '4px',
                    } },
                    React.createElement("option", { value: "debug" }, "Debug"),
                    React.createElement("option", { value: "info" }, "Info"),
                    React.createElement("option", { value: "warn" }, "Warning"),
                    React.createElement("option", { value: "error" }, "Error"))),
            React.createElement("div", { style: { marginBottom: '1rem' } },
                React.createElement("label", { htmlFor: "retention", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Log Retention (days)"),
                React.createElement("input", { id: "retention", type: "number", min: "1", max: "365", value: settings.logging.retention, onChange: (e) => updateSetting('logging.retention', parseInt(e.target.value)), style: {
                        width: '200px',
                        padding: '0.5rem',
                        border: '1px solid #d0d0d0',
                        borderRadius: '4px',
                    } })),
            React.createElement("div", { style: { display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '0.5rem' } },
                React.createElement("label", { style: { display: 'flex', alignItems: 'center', cursor: 'pointer' } },
                    React.createElement("input", { type: "checkbox", checked: settings.logging.maskSensitiveData, onChange: (e) => updateSetting('logging.maskSensitiveData', e.target.checked), style: { marginRight: '0.5rem' } }),
                    React.createElement("span", null, "Mask Sensitive Data")),
                React.createElement("label", { style: { display: 'flex', alignItems: 'center', cursor: 'pointer' } },
                    React.createElement("input", { type: "checkbox", checked: settings.logging.includeStackTrace, onChange: (e) => updateSetting('logging.includeStackTrace', e.target.checked), style: { marginRight: '0.5rem' } }),
                    React.createElement("span", null, "Include Stack Traces")))),
        React.createElement("section", { style: {
                backgroundColor: '#fff',
                borderRadius: '8px',
                padding: '1.5rem',
                border: '1px solid #e0e0e0',
            } },
            React.createElement("h3", { style: { fontSize: '1.125rem', marginBottom: '1rem' } }, "Performance"),
            React.createElement("div", { style: { display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem', marginBottom: '1rem' } },
                React.createElement("label", { style: { display: 'flex', alignItems: 'center', cursor: 'pointer' } },
                    React.createElement("input", { type: "checkbox", checked: settings.performance.cacheEnabled, onChange: (e) => updateSetting('performance.cacheEnabled', e.target.checked), style: { marginRight: '0.5rem' } }),
                    React.createElement("span", null, "Enable Caching")),
                React.createElement("label", { style: { display: 'flex', alignItems: 'center', cursor: 'pointer' } },
                    React.createElement("input", { type: "checkbox", checked: settings.performance.compressionEnabled, onChange: (e) => updateSetting('performance.compressionEnabled', e.target.checked), style: { marginRight: '0.5rem' } }),
                    React.createElement("span", null, "Enable Compression")),
                React.createElement("label", { style: { display: 'flex', alignItems: 'center', cursor: 'pointer' } },
                    React.createElement("input", { type: "checkbox", checked: settings.performance.minifyAssets, onChange: (e) => updateSetting('performance.minifyAssets', e.target.checked), style: { marginRight: '0.5rem' } }),
                    React.createElement("span", null, "Minify Assets")),
                React.createElement("label", { style: { display: 'flex', alignItems: 'center', cursor: 'pointer' } },
                    React.createElement("input", { type: "checkbox", checked: settings.performance.cdnEnabled, onChange: (e) => updateSetting('performance.cdnEnabled', e.target.checked), style: { marginRight: '0.5rem' } }),
                    React.createElement("span", null, "Enable CDN"))),
            settings.performance.cacheEnabled && (React.createElement("div", { style: { marginBottom: '1rem' } },
                React.createElement("label", { htmlFor: "cacheTTL", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Cache TTL (seconds)"),
                React.createElement("input", { id: "cacheTTL", type: "number", min: "60", max: "86400", value: settings.performance.cacheTTL, onChange: (e) => updateSetting('performance.cacheTTL', parseInt(e.target.value)), style: {
                        width: '200px',
                        padding: '0.5rem',
                        border: '1px solid #d0d0d0',
                        borderRadius: '4px',
                    } }))),
            settings.performance.compressionEnabled && (React.createElement("div", null,
                React.createElement("label", { htmlFor: "compressionLevel", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Compression Level (1-9)"),
                React.createElement("input", { id: "compressionLevel", type: "range", min: "1", max: "9", value: settings.performance.compressionLevel, onChange: (e) => updateSetting('performance.compressionLevel', parseInt(e.target.value)), style: {
                        width: '200px',
                    } }),
                React.createElement("span", { style: { marginLeft: '1rem' } }, settings.performance.compressionLevel)))),
        showKeyModal && (React.createElement("div", { style: {
                position: 'fixed',
                top: 0,
                left: 0,
                right: 0,
                bottom: 0,
                backgroundColor: 'rgba(0, 0, 0, 0.5)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                zIndex: 1000,
            }, onClick: () => setShowKeyModal(false) },
            React.createElement("div", { onClick: (e) => e.stopPropagation(), style: {
                    backgroundColor: '#fff',
                    borderRadius: '8px',
                    padding: '2rem',
                    maxWidth: '500px',
                    width: '90%',
                } },
                React.createElement("h2", { style: { marginTop: 0 } }, "Generate API Key"),
                React.createElement("div", { style: { marginBottom: '1.5rem' } },
                    React.createElement("label", { htmlFor: "keyName", style: { display: 'block', marginBottom: '0.5rem', fontWeight: 500 } }, "Key Name"),
                    React.createElement("input", { id: "keyName", type: "text", value: newKeyName, onChange: (e) => setNewKeyName(e.target.value), placeholder: "e.g., Production API Key", style: {
                            width: '100%',
                            padding: '0.5rem',
                            border: '1px solid #d0d0d0',
                            borderRadius: '4px',
                        } })),
                showNewKey && (React.createElement("div", { style: {
                        marginBottom: '1.5rem',
                        padding: '1rem',
                        backgroundColor: '#fff3e0',
                        borderRadius: '4px',
                    } },
                    React.createElement("div", { style: { fontWeight: 600, marginBottom: '0.5rem', color: '#e65100' } }, "\u26A0\uFE0F Save this key securely"),
                    React.createElement("p", { style: { margin: '0 0 0.5rem 0', fontSize: '0.875rem' } }, "This key will only be shown once. Make sure to copy it now."),
                    React.createElement("code", { style: {
                            display: 'block',
                            padding: '0.5rem',
                            backgroundColor: '#fff',
                            borderRadius: '4px',
                            fontSize: '0.875rem',
                            wordBreak: 'break-all',
                            fontFamily: 'monospace',
                        } }, showNewKey))),
                React.createElement("div", { style: { display: 'flex', gap: '1rem', justifyContent: 'flex-end' } },
                    React.createElement("button", { onClick: () => {
                            setShowKeyModal(false);
                            setShowNewKey(null);
                        }, style: {
                            padding: '0.5rem 1.5rem',
                            backgroundColor: '#f5f5f5',
                            border: '1px solid #e0e0e0',
                            borderRadius: '4px',
                            cursor: 'pointer',
                        } }, showNewKey ? 'Close' : 'Cancel'),
                    !showNewKey && (React.createElement("button", { onClick: generateAPIKey, style: {
                            padding: '0.5rem 1.5rem',
                            backgroundColor: '#1976d2',
                            color: '#fff',
                            border: 'none',
                            borderRadius: '4px',
                            cursor: 'pointer',
                        } }, "Generate"))))))));
};
export default AdvancedSettings;
//# sourceMappingURL=AdvancedSettings.js.map