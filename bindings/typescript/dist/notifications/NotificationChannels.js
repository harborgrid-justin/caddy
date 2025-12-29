import React, { useState, useCallback, useEffect } from 'react';
import { NotificationChannel } from './types';
export const NotificationChannels = ({ tenantId, apiUrl = '/api/notifications/channels' }) => {
    const [configs, setConfigs] = useState([]);
    const [loading, setLoading] = useState(false);
    const [editingConfig, setEditingConfig] = useState(null);
    const [isModalOpen, setIsModalOpen] = useState(false);
    const [showSecrets, setShowSecrets] = useState({});
    const fetchConfigs = useCallback(async () => {
        setLoading(true);
        try {
            const response = await fetch(`${apiUrl}?tenantId=${tenantId}`, {
                credentials: 'include'
            });
            const data = await response.json();
            setConfigs(data.configs || []);
        }
        catch (err) {
            console.error('Error fetching channel configs:', err);
        }
        finally {
            setLoading(false);
        }
    }, [apiUrl, tenantId]);
    useEffect(() => {
        fetchConfigs();
    }, [fetchConfigs]);
    const handleEdit = useCallback((config) => {
        setEditingConfig(config);
        setIsModalOpen(true);
    }, []);
    const handleCreate = useCallback((channel) => {
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
        if (!editingConfig)
            return;
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
        }
        catch (err) {
            console.error('Error saving config:', err);
            alert('Failed to save configuration');
        }
    }, [editingConfig, apiUrl, tenantId, fetchConfigs]);
    const handleToggleEnabled = useCallback(async (config) => {
        try {
            await fetch(`${apiUrl}/${config.id}`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                credentials: 'include',
                body: JSON.stringify({ ...config, enabled: !config.enabled })
            });
            await fetchConfigs();
        }
        catch (err) {
            console.error('Error toggling config:', err);
        }
    }, [apiUrl, fetchConfigs]);
    const getChannelIcon = (channel) => {
        const icons = {
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
    const availableChannels = Object.values(NotificationChannel).filter(channel => !configs.some(c => c.channel === channel));
    const renderConfigForm = () => {
        if (!editingConfig)
            return null;
        const channel = editingConfig.channel;
        const config = editingConfig.config || {};
        return (React.createElement("div", { style: { display: 'flex', flexDirection: 'column', gap: '16px' } },
            channel === NotificationChannel.EMAIL && (React.createElement(React.Fragment, null,
                React.createElement("div", null,
                    React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "SMTP Host *"),
                    React.createElement("input", { type: "text", value: config.smtpHost || '', onChange: (e) => setEditingConfig({
                            ...editingConfig,
                            config: { ...config, smtpHost: e.target.value }
                        }), placeholder: "smtp.gmail.com", style: { width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' } })),
                React.createElement("div", { style: { display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px' } },
                    React.createElement("div", null,
                        React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "Port *"),
                        React.createElement("input", { type: "number", value: config.smtpPort || 587, onChange: (e) => setEditingConfig({
                                ...editingConfig,
                                config: { ...config, smtpPort: parseInt(e.target.value) }
                            }), style: { width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' } })),
                    React.createElement("div", { style: { display: 'flex', alignItems: 'flex-end' } },
                        React.createElement("label", { style: { display: 'flex', alignItems: 'center', gap: '8px', padding: '8px 0' } },
                            React.createElement("input", { type: "checkbox", checked: config.smtpSecure || false, onChange: (e) => setEditingConfig({
                                    ...editingConfig,
                                    config: { ...config, smtpSecure: e.target.checked }
                                }) }),
                            React.createElement("span", { style: { fontSize: '13px', color: '#374151' } }, "Use TLS/SSL")))),
                React.createElement("div", { style: { display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px' } },
                    React.createElement("div", null,
                        React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "Username *"),
                        React.createElement("input", { type: "text", value: config.smtpUser || '', onChange: (e) => setEditingConfig({
                                ...editingConfig,
                                config: { ...config, smtpUser: e.target.value }
                            }), style: { width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' } })),
                    React.createElement("div", null,
                        React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "Password *"),
                        React.createElement("input", { type: showSecrets[channel] ? 'text' : 'password', value: config.smtpPassword || '', onChange: (e) => setEditingConfig({
                                ...editingConfig,
                                config: { ...config, smtpPassword: e.target.value }
                            }), style: { width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' } }))),
                React.createElement("div", { style: { display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px' } },
                    React.createElement("div", null,
                        React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "From Email *"),
                        React.createElement("input", { type: "email", value: config.fromEmail || '', onChange: (e) => setEditingConfig({
                                ...editingConfig,
                                config: { ...config, fromEmail: e.target.value }
                            }), placeholder: "noreply@example.com", style: { width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' } })),
                    React.createElement("div", null,
                        React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "From Name"),
                        React.createElement("input", { type: "text", value: config.fromName || '', onChange: (e) => setEditingConfig({
                                ...editingConfig,
                                config: { ...config, fromName: e.target.value }
                            }), placeholder: "CADDY Notifications", style: { width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' } }))))),
            channel === NotificationChannel.SMS && (React.createElement(React.Fragment, null,
                React.createElement("div", null,
                    React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "SMS Provider *"),
                    React.createElement("select", { value: config.smsProvider || 'twilio', onChange: (e) => setEditingConfig({
                            ...editingConfig,
                            config: { ...config, smsProvider: e.target.value }
                        }), style: { width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' } },
                        React.createElement("option", { value: "twilio" }, "Twilio"),
                        React.createElement("option", { value: "nexmo" }, "Nexmo"),
                        React.createElement("option", { value: "aws-sns" }, "AWS SNS"))),
                React.createElement("div", null,
                    React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "API Key *"),
                    React.createElement("input", { type: showSecrets[channel] ? 'text' : 'password', value: config.smsApiKey || '', onChange: (e) => setEditingConfig({
                            ...editingConfig,
                            config: { ...config, smsApiKey: e.target.value }
                        }), style: { width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' } })),
                React.createElement("div", null,
                    React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "API Secret *"),
                    React.createElement("input", { type: showSecrets[channel] ? 'text' : 'password', value: config.smsApiSecret || '', onChange: (e) => setEditingConfig({
                            ...editingConfig,
                            config: { ...config, smsApiSecret: e.target.value }
                        }), style: { width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' } })),
                React.createElement("div", null,
                    React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "From Number *"),
                    React.createElement("input", { type: "tel", value: config.smsFromNumber || '', onChange: (e) => setEditingConfig({
                            ...editingConfig,
                            config: { ...config, smsFromNumber: e.target.value }
                        }), placeholder: "+1234567890", style: { width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' } })))),
            channel === NotificationChannel.PUSH && (React.createElement(React.Fragment, null,
                React.createElement("div", null,
                    React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "Push Provider *"),
                    React.createElement("select", { value: config.pushProvider || 'fcm', onChange: (e) => setEditingConfig({
                            ...editingConfig,
                            config: { ...config, pushProvider: e.target.value }
                        }), style: { width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' } },
                        React.createElement("option", { value: "fcm" }, "Firebase Cloud Messaging"),
                        React.createElement("option", { value: "apns" }, "Apple Push Notification Service"),
                        React.createElement("option", { value: "onesignal" }, "OneSignal"))),
                React.createElement("div", null,
                    React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "API Key *"),
                    React.createElement("input", { type: showSecrets[channel] ? 'text' : 'password', value: config.pushApiKey || '', onChange: (e) => setEditingConfig({
                            ...editingConfig,
                            config: { ...config, pushApiKey: e.target.value }
                        }), style: { width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' } })),
                React.createElement("div", null,
                    React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "App ID"),
                    React.createElement("input", { type: "text", value: config.pushAppId || '', onChange: (e) => setEditingConfig({
                            ...editingConfig,
                            config: { ...config, pushAppId: e.target.value }
                        }), style: { width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' } })))),
            channel === NotificationChannel.SLACK && (React.createElement(React.Fragment, null,
                React.createElement("div", null,
                    React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "Webhook URL *"),
                    React.createElement("input", { type: showSecrets[channel] ? 'text' : 'password', value: config.slackWebhookUrl || '', onChange: (e) => setEditingConfig({
                            ...editingConfig,
                            config: { ...config, slackWebhookUrl: e.target.value }
                        }), placeholder: "https://hooks.slack.com/services/...", style: { width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' } })),
                React.createElement("div", null,
                    React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "Bot Token (Optional)"),
                    React.createElement("input", { type: showSecrets[channel] ? 'text' : 'password', value: config.slackBotToken || '', onChange: (e) => setEditingConfig({
                            ...editingConfig,
                            config: { ...config, slackBotToken: e.target.value }
                        }), placeholder: "xoxb-...", style: { width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' } })),
                React.createElement("div", null,
                    React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "Default Channel"),
                    React.createElement("input", { type: "text", value: config.slackChannel || '', onChange: (e) => setEditingConfig({
                            ...editingConfig,
                            config: { ...config, slackChannel: e.target.value }
                        }), placeholder: "#notifications", style: { width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' } })))),
            channel === NotificationChannel.TEAMS && (React.createElement("div", null,
                React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "Webhook URL *"),
                React.createElement("input", { type: showSecrets[channel] ? 'text' : 'password', value: config.teamsWebhookUrl || '', onChange: (e) => setEditingConfig({
                        ...editingConfig,
                        config: { ...config, teamsWebhookUrl: e.target.value }
                    }), placeholder: "https://outlook.office.com/webhook/...", style: { width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' } }))),
            channel === NotificationChannel.WEBHOOK && (React.createElement(React.Fragment, null,
                React.createElement("div", null,
                    React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "Webhook URL *"),
                    React.createElement("input", { type: "text", value: config.webhookUrl || '', onChange: (e) => setEditingConfig({
                            ...editingConfig,
                            config: { ...config, webhookUrl: e.target.value }
                        }), placeholder: "https://api.example.com/webhook", style: { width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' } })),
                React.createElement("div", null,
                    React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "HTTP Method"),
                    React.createElement("select", { value: config.webhookMethod || 'POST', onChange: (e) => setEditingConfig({
                            ...editingConfig,
                            config: { ...config, webhookMethod: e.target.value }
                        }), style: { width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' } },
                        React.createElement("option", { value: "POST" }, "POST"),
                        React.createElement("option", { value: "PUT" }, "PUT"),
                        React.createElement("option", { value: "PATCH" }, "PATCH"))))),
            React.createElement("div", { style: { display: 'flex', alignItems: 'center', gap: '8px' } },
                React.createElement("input", { type: "checkbox", checked: showSecrets[channel] || false, onChange: (e) => setShowSecrets({ ...showSecrets, [channel]: e.target.checked }) }),
                React.createElement("span", { style: { fontSize: '13px', color: '#6b7280' } }, "Show sensitive values")),
            React.createElement("div", { style: { paddingTop: '16px', borderTop: '1px solid #e5e7eb' } },
                React.createElement("h4", { style: { margin: '0 0 12px 0', fontSize: '14px', fontWeight: '600', color: '#111827' } }, "Rate Limits"),
                React.createElement("div", { style: { display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: '12px' } },
                    React.createElement("div", null,
                        React.createElement("label", { style: { display: 'block', fontSize: '12px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "Per Minute"),
                        React.createElement("input", { type: "number", value: editingConfig.rateLimit?.maxPerMinute || 60, onChange: (e) => setEditingConfig({
                                ...editingConfig,
                                rateLimit: { ...editingConfig.rateLimit, maxPerMinute: parseInt(e.target.value) }
                            }), style: { width: '100%', padding: '6px 8px', fontSize: '13px', border: '1px solid #d1d5db', borderRadius: '4px' } })),
                    React.createElement("div", null,
                        React.createElement("label", { style: { display: 'block', fontSize: '12px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "Per Hour"),
                        React.createElement("input", { type: "number", value: editingConfig.rateLimit?.maxPerHour || 1000, onChange: (e) => setEditingConfig({
                                ...editingConfig,
                                rateLimit: { ...editingConfig.rateLimit, maxPerHour: parseInt(e.target.value) }
                            }), style: { width: '100%', padding: '6px 8px', fontSize: '13px', border: '1px solid #d1d5db', borderRadius: '4px' } })),
                    React.createElement("div", null,
                        React.createElement("label", { style: { display: 'block', fontSize: '12px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "Per Day"),
                        React.createElement("input", { type: "number", value: editingConfig.rateLimit?.maxPerDay || 10000, onChange: (e) => setEditingConfig({
                                ...editingConfig,
                                rateLimit: { ...editingConfig.rateLimit, maxPerDay: parseInt(e.target.value) }
                            }), style: { width: '100%', padding: '6px 8px', fontSize: '13px', border: '1px solid #d1d5db', borderRadius: '4px' } }))))));
    };
    return (React.createElement("div", { style: { padding: '24px', maxWidth: '1200px', margin: '0 auto' } },
        React.createElement("div", { style: { marginBottom: '24px' } },
            React.createElement("h2", { style: { margin: '0 0 4px 0', fontSize: '20px', fontWeight: '600', color: '#111827' } }, "Notification Channels"),
            React.createElement("p", { style: { margin: 0, fontSize: '14px', color: '#6b7280' } }, "Configure multi-channel notification delivery")),
        React.createElement("div", { style: { display: 'grid', gap: '16px', marginBottom: '32px' } }, configs.map((config) => (React.createElement("div", { key: config.id, style: {
                padding: '16px',
                border: '1px solid #e5e7eb',
                borderRadius: '8px',
                backgroundColor: config.enabled ? '#ffffff' : '#f9fafb'
            } },
            React.createElement("div", { style: { display: 'flex', alignItems: 'center', justifyContent: 'space-between' } },
                React.createElement("div", { style: { display: 'flex', alignItems: 'center', gap: '12px' } },
                    React.createElement("div", { style: { fontSize: '32px' } }, getChannelIcon(config.channel)),
                    React.createElement("div", null,
                        React.createElement("div", { style: { display: 'flex', alignItems: 'center', gap: '8px' } },
                            React.createElement("h3", { style: { margin: 0, fontSize: '16px', fontWeight: '600', color: '#111827', textTransform: 'capitalize' } }, config.channel.replace('_', ' ')),
                            React.createElement("span", { style: {
                                    padding: '2px 8px',
                                    fontSize: '11px',
                                    fontWeight: '500',
                                    borderRadius: '12px',
                                    backgroundColor: config.enabled ? '#dcfce7' : '#fee2e2',
                                    color: config.enabled ? '#166534' : '#991b1b'
                                } }, config.enabled ? 'Enabled' : 'Disabled')),
                        React.createElement("div", { style: { marginTop: '4px', fontSize: '12px', color: '#6b7280' } },
                            "Rate limits: ",
                            config.rateLimit?.maxPerMinute,
                            "/min, ",
                            config.rateLimit?.maxPerHour,
                            "/hr, ",
                            config.rateLimit?.maxPerDay,
                            "/day"))),
                React.createElement("div", { style: { display: 'flex', gap: '8px' } },
                    React.createElement("button", { onClick: () => handleToggleEnabled(config), style: {
                            padding: '6px 12px',
                            fontSize: '12px',
                            fontWeight: '500',
                            border: '1px solid #d1d5db',
                            borderRadius: '4px',
                            backgroundColor: '#ffffff',
                            color: '#374151',
                            cursor: 'pointer'
                        } }, config.enabled ? 'Disable' : 'Enable'),
                    React.createElement("button", { onClick: () => handleEdit(config), style: {
                            padding: '6px 12px',
                            fontSize: '12px',
                            fontWeight: '500',
                            border: '1px solid #d1d5db',
                            borderRadius: '4px',
                            backgroundColor: '#ffffff',
                            color: '#374151',
                            cursor: 'pointer'
                        } }, "Configure"))))))),
        availableChannels.length > 0 && (React.createElement("div", null,
            React.createElement("h3", { style: { margin: '0 0 12px 0', fontSize: '16px', fontWeight: '600', color: '#111827' } }, "Add Channel"),
            React.createElement("div", { style: { display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(150px, 1fr))', gap: '12px' } }, availableChannels.map((channel) => (React.createElement("button", { key: channel, onClick: () => handleCreate(channel), style: {
                    padding: '16px',
                    border: '2px dashed #d1d5db',
                    borderRadius: '8px',
                    backgroundColor: '#ffffff',
                    cursor: 'pointer',
                    transition: 'all 0.2s',
                    textAlign: 'center'
                }, onMouseEnter: (e) => {
                    e.currentTarget.style.borderColor = '#3b82f6';
                    e.currentTarget.style.backgroundColor = '#f0f9ff';
                }, onMouseLeave: (e) => {
                    e.currentTarget.style.borderColor = '#d1d5db';
                    e.currentTarget.style.backgroundColor = '#ffffff';
                } },
                React.createElement("div", { style: { fontSize: '32px', marginBottom: '8px' } }, getChannelIcon(channel)),
                React.createElement("div", { style: { fontSize: '13px', fontWeight: '500', color: '#374151', textTransform: 'capitalize' } }, channel.replace('_', ' ')))))))),
        isModalOpen && editingConfig && (React.createElement("div", { style: {
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
            }, onClick: () => setIsModalOpen(false) },
            React.createElement("div", { onClick: (e) => e.stopPropagation(), style: {
                    backgroundColor: '#ffffff',
                    borderRadius: '8px',
                    padding: '24px',
                    maxWidth: '600px',
                    width: '90%',
                    maxHeight: '80vh',
                    overflowY: 'auto',
                    boxShadow: '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)'
                } },
                React.createElement("h3", { style: { margin: '0 0 24px 0', fontSize: '20px', fontWeight: '600', color: '#111827' } },
                    "Configure ",
                    editingConfig.channel?.replace('_', ' '),
                    " Channel"),
                renderConfigForm(),
                React.createElement("div", { style: { display: 'flex', gap: '12px', justifyContent: 'flex-end', marginTop: '24px' } },
                    React.createElement("button", { onClick: () => setIsModalOpen(false), style: {
                            padding: '10px 20px',
                            fontSize: '14px',
                            fontWeight: '500',
                            border: '1px solid #d1d5db',
                            borderRadius: '6px',
                            backgroundColor: '#ffffff',
                            color: '#374151',
                            cursor: 'pointer'
                        } }, "Cancel"),
                    React.createElement("button", { onClick: handleSave, style: {
                            padding: '10px 20px',
                            fontSize: '14px',
                            fontWeight: '500',
                            border: 'none',
                            borderRadius: '6px',
                            backgroundColor: '#3b82f6',
                            color: '#ffffff',
                            cursor: 'pointer'
                        } }, "Save Configuration")))))));
};
//# sourceMappingURL=NotificationChannels.js.map