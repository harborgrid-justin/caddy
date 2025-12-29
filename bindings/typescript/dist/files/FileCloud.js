import React, { useState, useEffect } from 'react';
export const FileCloud = ({ tenantId, onClose, className = '', }) => {
    const [configs, setConfigs] = useState([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const [activeTab, setActiveTab] = useState('providers');
    const [showAddProvider, setShowAddProvider] = useState(false);
    const [editingConfig, setEditingConfig] = useState(null);
    const [syncActivity, setSyncActivity] = useState([]);
    useEffect(() => {
        loadConfigs();
        loadSyncActivity();
    }, [tenantId]);
    const loadConfigs = async () => {
        setLoading(true);
        setError(null);
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/cloud/configs`, {
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                },
            });
            if (!response.ok) {
                throw new Error('Failed to load cloud configurations');
            }
            const data = await response.json();
            setConfigs(data.configs);
        }
        catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to load configurations');
        }
        finally {
            setLoading(false);
        }
    };
    const loadSyncActivity = async () => {
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/cloud/activity`, {
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                },
            });
            if (response.ok) {
                const data = await response.json();
                setSyncActivity(data.activity);
            }
        }
        catch (err) {
            console.error('Failed to load sync activity:', err);
        }
    };
    const saveConfig = async (config) => {
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/cloud/configs`, {
                method: config.provider ? 'PUT' : 'POST',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(config),
            });
            if (!response.ok) {
                throw new Error('Failed to save configuration');
            }
            await loadConfigs();
            setShowAddProvider(false);
            setEditingConfig(null);
        }
        catch (err) {
            alert(err instanceof Error ? err.message : 'Failed to save configuration');
        }
    };
    const deleteConfig = async (provider) => {
        if (!confirm(`Delete ${provider} configuration? This will stop all syncing with this provider.`)) {
            return;
        }
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/cloud/configs/${provider}`, {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                },
            });
            if (!response.ok) {
                throw new Error('Failed to delete configuration');
            }
            await loadConfigs();
        }
        catch (err) {
            alert(err instanceof Error ? err.message : 'Failed to delete configuration');
        }
    };
    const testConnection = async (config) => {
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/cloud/test`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ provider: config.provider }),
            });
            if (!response.ok) {
                throw new Error('Connection test failed');
            }
            alert('Connection successful!');
        }
        catch (err) {
            alert(err instanceof Error ? err.message : 'Connection test failed');
        }
    };
    const triggerSync = async (provider) => {
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/cloud/sync`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ provider }),
            });
            if (!response.ok) {
                throw new Error('Failed to trigger sync');
            }
            alert('Sync started');
            await loadSyncActivity();
        }
        catch (err) {
            alert(err instanceof Error ? err.message : 'Failed to trigger sync');
        }
    };
    const getProviderIcon = (provider) => {
        const icons = {
            s3: '☁️',
            gcs: '☁️',
            azure: '☁️',
            dropbox: '📦',
            onedrive: '📂',
        };
        return icons[provider];
    };
    const getProviderName = (provider) => {
        const names = {
            s3: 'Amazon S3',
            gcs: 'Google Cloud Storage',
            azure: 'Azure Blob Storage',
            dropbox: 'Dropbox',
            onedrive: 'OneDrive',
        };
        return names[provider];
    };
    return (React.createElement("div", { className: `file-cloud ${className}` },
        React.createElement("div", { className: "cloud-header" },
            React.createElement("h2", null, "Cloud Storage"),
            onClose && (React.createElement("button", { onClick: onClose, className: "btn-close" }, "\u2715"))),
        React.createElement("div", { className: "cloud-tabs" },
            React.createElement("button", { className: `tab ${activeTab === 'providers' ? 'active' : ''}`, onClick: () => setActiveTab('providers') }, "Providers"),
            React.createElement("button", { className: `tab ${activeTab === 'sync' ? 'active' : ''}`, onClick: () => setActiveTab('sync') }, "Sync Rules"),
            React.createElement("button", { className: `tab ${activeTab === 'activity' ? 'active' : ''}`, onClick: () => setActiveTab('activity') }, "Activity")),
        React.createElement("div", { className: "cloud-content" }, loading ? (React.createElement("div", { className: "loading-state" }, "Loading cloud configurations...")) : error ? (React.createElement("div", { className: "error-state" }, error)) : (React.createElement(React.Fragment, null,
            activeTab === 'providers' && (React.createElement("div", { className: "providers-tab" },
                React.createElement("div", { className: "tab-header" },
                    React.createElement("button", { onClick: () => setShowAddProvider(true), className: "btn btn-primary" }, "Add Cloud Provider")),
                configs.length === 0 ? (React.createElement("div", { className: "empty-state" }, "No cloud providers configured. Add one to enable cloud sync.")) : (React.createElement("div", { className: "providers-list" }, configs.map(config => (React.createElement("div", { key: config.provider, className: "provider-card" },
                    React.createElement("div", { className: "provider-header" },
                        React.createElement("div", { className: "provider-title" },
                            React.createElement("span", { className: "provider-icon" }, getProviderIcon(config.provider)),
                            React.createElement("span", { className: "provider-name" }, getProviderName(config.provider)),
                            React.createElement("span", { className: `status-badge ${config.enabled ? 'active' : 'inactive'}` }, config.enabled ? 'Active' : 'Inactive')),
                        React.createElement("div", { className: "provider-actions" },
                            React.createElement("button", { onClick: () => testConnection(config), className: "btn btn-sm" }, "Test"),
                            React.createElement("button", { onClick: () => triggerSync(config.provider), disabled: !config.enabled, className: "btn btn-sm btn-primary" }, "Sync Now"),
                            React.createElement("button", { onClick: () => setEditingConfig(config), className: "btn btn-sm" }, "Edit"),
                            React.createElement("button", { onClick: () => deleteConfig(config.provider), className: "btn btn-sm btn-danger" }, "Delete"))),
                    React.createElement("div", { className: "provider-details" },
                        React.createElement("div", { className: "detail-item" },
                            React.createElement("strong", null, "Auto Sync:"),
                            ' ',
                            config.settings.autoSync ? 'Enabled' : 'Disabled'),
                        config.settings.autoSync && (React.createElement("div", { className: "detail-item" },
                            React.createElement("strong", null, "Sync Interval:"),
                            ' ',
                            config.settings.syncInterval,
                            " minutes")),
                        React.createElement("div", { className: "detail-item" },
                            React.createElement("strong", null, "Conflict Resolution:"),
                            ' ',
                            config.settings.conflictResolution),
                        React.createElement("div", { className: "detail-item" },
                            React.createElement("strong", null, "Encryption:"),
                            ' ',
                            config.settings.encryption ? 'Enabled' : 'Disabled'),
                        React.createElement("div", { className: "detail-item" },
                            React.createElement("strong", null, "Sync Rules:"),
                            ' ',
                            config.syncRules.filter(r => r.enabled).length,
                            " active"))))))),
                (showAddProvider || editingConfig) && (React.createElement(CloudProviderDialog, { config: editingConfig, onSave: saveConfig, onCancel: () => {
                        setShowAddProvider(false);
                        setEditingConfig(null);
                    } })))),
            activeTab === 'sync' && (React.createElement("div", { className: "sync-tab" }, configs.length === 0 ? (React.createElement("div", { className: "empty-state" }, "Configure a cloud provider first to set up sync rules.")) : (configs.map(config => (React.createElement("div", { key: config.provider, className: "sync-rules-section" },
                React.createElement("h3", null,
                    getProviderName(config.provider),
                    " Sync Rules"),
                React.createElement("div", { className: "sync-rules-list" }, config.syncRules.length === 0 ? (React.createElement("div", { className: "empty-state" }, "No sync rules configured")) : (config.syncRules.map(rule => (React.createElement("div", { key: rule.id, className: "sync-rule-item" },
                    React.createElement("div", { className: "rule-pattern" },
                        React.createElement("code", null, rule.pattern)),
                    React.createElement("div", { className: "rule-action" }, rule.action),
                    React.createElement("div", { className: "rule-status" }, rule.enabled ? '✓ Enabled' : '✗ Disabled')))))))))))),
            activeTab === 'activity' && (React.createElement("div", { className: "activity-tab" }, syncActivity.length === 0 ? (React.createElement("div", { className: "empty-state" }, "No sync activity yet")) : (React.createElement("div", { className: "activity-list" }, syncActivity.map(activity => (React.createElement("div", { key: activity.id, className: `activity-item activity-${activity.status}` },
                React.createElement("div", { className: "activity-icon" }, getProviderIcon(activity.provider)),
                React.createElement("div", { className: "activity-details" },
                    React.createElement("div", { className: "activity-title" },
                        activity.action,
                        " \u2022 ",
                        activity.fileName),
                    React.createElement("div", { className: "activity-meta" },
                        getProviderName(activity.provider),
                        " \u2022",
                        ' ',
                        new Date(activity.timestamp).toLocaleString()),
                    activity.error && (React.createElement("div", { className: "activity-error" }, activity.error))),
                React.createElement("div", { className: `activity-status status-${activity.status}` }, activity.status)))))))))))));
};
const CloudProviderDialog = ({ config, onSave, onCancel, }) => {
    const [provider, setProvider] = useState(config?.provider || 's3');
    const [credentials, setCredentials] = useState(config?.credentials || {});
    const [settings, setSettings] = useState(config?.settings || {
        autoSync: false,
        syncInterval: 60,
        conflictResolution: 'newest',
        bandwidth: { upload: 0, download: 0 },
        encryption: false,
    });
    const handleSave = () => {
        onSave({
            provider,
            enabled: true,
            credentials,
            settings,
            syncRules: config?.syncRules || [],
        });
    };
    return (React.createElement("div", { className: "cloud-dialog-overlay" },
        React.createElement("div", { className: "cloud-dialog" },
            React.createElement("div", { className: "dialog-header" },
                React.createElement("h3", null,
                    config ? 'Edit' : 'Add',
                    " Cloud Provider"),
                React.createElement("button", { onClick: onCancel, className: "btn-close" }, "\u2715")),
            React.createElement("div", { className: "dialog-content" },
                !config && (React.createElement("div", { className: "form-group" },
                    React.createElement("label", null, "Provider"),
                    React.createElement("select", { value: provider, onChange: (e) => setProvider(e.target.value), className: "form-select" },
                        React.createElement("option", { value: "s3" }, "Amazon S3"),
                        React.createElement("option", { value: "gcs" }, "Google Cloud Storage"),
                        React.createElement("option", { value: "azure" }, "Azure Blob Storage"),
                        React.createElement("option", { value: "dropbox" }, "Dropbox"),
                        React.createElement("option", { value: "onedrive" }, "OneDrive")))),
                provider === 's3' && (React.createElement(React.Fragment, null,
                    React.createElement("div", { className: "form-group" },
                        React.createElement("label", null, "Access Key ID"),
                        React.createElement("input", { type: "text", value: credentials.accessKey || '', onChange: (e) => setCredentials({ ...credentials, accessKey: e.target.value }), className: "form-input" })),
                    React.createElement("div", { className: "form-group" },
                        React.createElement("label", null, "Secret Access Key"),
                        React.createElement("input", { type: "password", value: credentials.secretKey || '', onChange: (e) => setCredentials({ ...credentials, secretKey: e.target.value }), className: "form-input" })),
                    React.createElement("div", { className: "form-group" },
                        React.createElement("label", null, "Bucket Name"),
                        React.createElement("input", { type: "text", value: credentials.bucket || '', onChange: (e) => setCredentials({ ...credentials, bucket: e.target.value }), className: "form-input" })),
                    React.createElement("div", { className: "form-group" },
                        React.createElement("label", null, "Region"),
                        React.createElement("input", { type: "text", value: credentials.region || '', onChange: (e) => setCredentials({ ...credentials, region: e.target.value }), className: "form-input", placeholder: "us-east-1" })))),
                React.createElement("div", { className: "form-group" },
                    React.createElement("label", null,
                        React.createElement("input", { type: "checkbox", checked: settings.autoSync, onChange: (e) => setSettings({ ...settings, autoSync: e.target.checked }) }),
                        ' ',
                        "Enable Auto Sync")),
                settings.autoSync && (React.createElement("div", { className: "form-group" },
                    React.createElement("label", null, "Sync Interval (minutes)"),
                    React.createElement("input", { type: "number", value: settings.syncInterval, onChange: (e) => setSettings({ ...settings, syncInterval: parseInt(e.target.value) }), className: "form-input", min: "5" }))),
                React.createElement("div", { className: "form-group" },
                    React.createElement("label", null, "Conflict Resolution"),
                    React.createElement("select", { value: settings.conflictResolution, onChange: (e) => setSettings({
                            ...settings,
                            conflictResolution: e.target.value,
                        }), className: "form-select" },
                        React.createElement("option", { value: "local" }, "Local wins"),
                        React.createElement("option", { value: "remote" }, "Remote wins"),
                        React.createElement("option", { value: "newest" }, "Newest wins"),
                        React.createElement("option", { value: "manual" }, "Manual resolution"))),
                React.createElement("div", { className: "form-group" },
                    React.createElement("label", null,
                        React.createElement("input", { type: "checkbox", checked: settings.encryption, onChange: (e) => setSettings({ ...settings, encryption: e.target.checked }) }),
                        ' ',
                        "Enable Encryption"))),
            React.createElement("div", { className: "dialog-actions" },
                React.createElement("button", { onClick: onCancel, className: "btn" }, "Cancel"),
                React.createElement("button", { onClick: handleSave, className: "btn btn-primary" }, "Save")))));
};
export default FileCloud;
//# sourceMappingURL=FileCloud.js.map