import React, { useState } from 'react';
import { usePluginSettings, usePlugin } from './usePlugin';
export const PluginSettings = ({ pluginId, className = '', onClose, onSaved, }) => {
    const { plugin } = usePlugin(pluginId);
    const { settings, loading, error, updateSettings } = usePluginSettings(pluginId);
    const [localSettings, setLocalSettings] = useState(null);
    const [saving, setSaving] = useState(false);
    const [saveError, setSaveError] = useState(null);
    React.useEffect(() => {
        if (settings && !localSettings) {
            setLocalSettings(settings);
        }
    }, [settings]);
    const handleSave = async () => {
        if (!localSettings)
            return;
        setSaving(true);
        setSaveError(null);
        try {
            await updateSettings({
                enabled: localSettings.enabled,
                config: localSettings.config,
                autoStart: localSettings.autoStart,
                resourceLimits: localSettings.resourceLimits,
            });
            onSaved?.();
            onClose?.();
        }
        catch (err) {
            setSaveError(err.message);
        }
        finally {
            setSaving(false);
        }
    };
    const handleConfigChange = (key, value) => {
        if (!localSettings)
            return;
        setLocalSettings({
            ...localSettings,
            config: {
                ...localSettings.config,
                [key]: value,
            },
        });
    };
    const handleResourceLimitChange = (key, value) => {
        if (!localSettings)
            return;
        setLocalSettings({
            ...localSettings,
            resourceLimits: {
                ...(localSettings.resourceLimits || getDefaultResourceLimits()),
                [key]: value,
            },
        });
    };
    if (loading || !localSettings) {
        return (React.createElement("div", { className: `plugin-settings loading ${className}` },
            React.createElement("div", { className: "loading-spinner" }, "Loading settings...")));
    }
    if (error) {
        return (React.createElement("div", { className: `plugin-settings error ${className}` },
            React.createElement("div", { className: "error-message" },
                React.createElement("h3", null, "Error loading settings"),
                React.createElement("p", null, error.message))));
    }
    return (React.createElement("div", { className: `plugin-settings ${className}` },
        React.createElement("div", { className: "settings-header" },
            React.createElement("h2", null, "Plugin Settings"),
            plugin && React.createElement("h3", null, plugin.manifest.name),
            onClose && (React.createElement("button", { className: "btn-close", onClick: onClose }, "Close"))),
        React.createElement("div", { className: "settings-body" },
            React.createElement("section", { className: "settings-section" },
                React.createElement("h4", null, "General"),
                React.createElement("label", { className: "setting-item" },
                    React.createElement("input", { type: "checkbox", checked: localSettings.enabled, onChange: (e) => setLocalSettings({ ...localSettings, enabled: e.target.checked }) }),
                    React.createElement("span", null, "Enable plugin")),
                React.createElement("label", { className: "setting-item" },
                    React.createElement("input", { type: "checkbox", checked: localSettings.autoStart, onChange: (e) => setLocalSettings({ ...localSettings, autoStart: e.target.checked }) }),
                    React.createElement("span", null, "Auto-start when CADDY launches"))),
            plugin && (React.createElement("section", { className: "settings-section" },
                React.createElement("h4", null, "Permissions"),
                React.createElement("div", { className: "permissions-list" }, plugin.manifest.permissions.map((permission) => (React.createElement("div", { key: permission, className: "permission-item" },
                    React.createElement("span", { className: "permission-icon" }, "\u2713"),
                    React.createElement("span", { className: "permission-name" }, permission),
                    React.createElement("span", { className: "permission-description" }, getPermissionDescription(permission)))))))),
            React.createElement("section", { className: "settings-section" },
                React.createElement("h4", null, "Resource Limits"),
                React.createElement("div", { className: "setting-item" },
                    React.createElement("label", null, "Maximum Memory (MB)"),
                    React.createElement("input", { type: "number", value: Math.round((localSettings.resourceLimits?.maxMemoryBytes || 128 * 1024 * 1024) /
                            (1024 * 1024)), onChange: (e) => handleResourceLimitChange('maxMemoryBytes', Number(e.target.value) * 1024 * 1024), min: "16", max: "2048" })),
                React.createElement("div", { className: "setting-item" },
                    React.createElement("label", null, "Maximum Execution Time (ms)"),
                    React.createElement("input", { type: "number", value: localSettings.resourceLimits?.maxExecutionTimeMs || 5000, onChange: (e) => handleResourceLimitChange('maxExecutionTimeMs', Number(e.target.value)), min: "100", max: "60000" })),
                React.createElement("div", { className: "setting-item" },
                    React.createElement("label", null, "Maximum File Size (MB)"),
                    React.createElement("input", { type: "number", value: Math.round((localSettings.resourceLimits?.maxFileSizeBytes || 10 * 1024 * 1024) /
                            (1024 * 1024)), onChange: (e) => handleResourceLimitChange('maxFileSizeBytes', Number(e.target.value) * 1024 * 1024), min: "1", max: "100" })),
                React.createElement("div", { className: "setting-item" },
                    React.createElement("label", null, "Max File Operations/Second"),
                    React.createElement("input", { type: "number", value: localSettings.resourceLimits?.maxFileOpsPerSecond || 100, onChange: (e) => handleResourceLimitChange('maxFileOpsPerSecond', Number(e.target.value)), min: "1", max: "1000" })),
                React.createElement("div", { className: "setting-item" },
                    React.createElement("label", null, "Max Network Requests/Second"),
                    React.createElement("input", { type: "number", value: localSettings.resourceLimits?.maxNetworkRequestsPerSecond || 10, onChange: (e) => handleResourceLimitChange('maxNetworkRequestsPerSecond', Number(e.target.value)), min: "1", max: "100" })),
                React.createElement("div", { className: "setting-item" },
                    React.createElement("label", null, "Maximum CPU Usage (%)"),
                    React.createElement("input", { type: "number", value: localSettings.resourceLimits?.maxCpuPercent || 50, onChange: (e) => handleResourceLimitChange('maxCpuPercent', Number(e.target.value)), min: "1", max: "100" }))),
            Object.keys(localSettings.config).length > 0 && (React.createElement("section", { className: "settings-section" },
                React.createElement("h4", null, "Plugin Configuration"),
                Object.entries(localSettings.config).map(([key, value]) => (React.createElement("div", { key: key, className: "setting-item" },
                    React.createElement("label", null, formatConfigKey(key)),
                    React.createElement(ConfigInput, { value: value, onChange: (newValue) => handleConfigChange(key, newValue) })))))),
            plugin && (React.createElement("section", { className: "settings-section" },
                React.createElement("h4", null, "Plugin Information"),
                React.createElement("dl", { className: "plugin-info-list" },
                    React.createElement("dt", null, "Version:"),
                    React.createElement("dd", null, plugin.manifest.version),
                    React.createElement("dt", null, "Author:"),
                    React.createElement("dd", null, plugin.manifest.author),
                    React.createElement("dt", null, "Type:"),
                    React.createElement("dd", null, plugin.manifest.pluginType),
                    plugin.manifest.license && (React.createElement(React.Fragment, null,
                        React.createElement("dt", null, "License:"),
                        React.createElement("dd", null, plugin.manifest.license))),
                    plugin.manifest.website && (React.createElement(React.Fragment, null,
                        React.createElement("dt", null, "Website:"),
                        React.createElement("dd", null,
                            React.createElement("a", { href: plugin.manifest.website, target: "_blank", rel: "noopener noreferrer" }, plugin.manifest.website)))),
                    plugin.manifest.repository && (React.createElement(React.Fragment, null,
                        React.createElement("dt", null, "Repository:"),
                        React.createElement("dd", null,
                            React.createElement("a", { href: plugin.manifest.repository, target: "_blank", rel: "noopener noreferrer" }, plugin.manifest.repository)))))))),
        React.createElement("div", { className: "settings-footer" },
            saveError && (React.createElement("div", { className: "error-message" },
                React.createElement("p", null, saveError))),
            React.createElement("div", { className: "footer-actions" },
                onClose && (React.createElement("button", { className: "btn-secondary", onClick: onClose, disabled: saving }, "Cancel")),
                React.createElement("button", { className: "btn-primary", onClick: handleSave, disabled: saving }, saving ? 'Saving...' : 'Save Settings')))));
};
const ConfigInput = ({ value, onChange }) => {
    if (typeof value === 'boolean') {
        return (React.createElement("input", { type: "checkbox", checked: value, onChange: (e) => onChange(e.target.checked) }));
    }
    if (typeof value === 'number') {
        return (React.createElement("input", { type: "number", value: value, onChange: (e) => onChange(Number(e.target.value)) }));
    }
    if (typeof value === 'string') {
        return (React.createElement("input", { type: "text", value: value, onChange: (e) => onChange(e.target.value) }));
    }
    return (React.createElement("textarea", { value: JSON.stringify(value, null, 2), onChange: (e) => {
            try {
                onChange(JSON.parse(e.target.value));
            }
            catch {
            }
        } }));
};
function getDefaultResourceLimits() {
    return {
        maxMemoryBytes: 128 * 1024 * 1024,
        maxExecutionTimeMs: 5000,
        maxFileSizeBytes: 10 * 1024 * 1024,
        maxFileOpsPerSecond: 100,
        maxNetworkRequestsPerSecond: 10,
        maxCpuPercent: 50,
    };
}
function formatConfigKey(key) {
    return key
        .replace(/([A-Z])/g, ' $1')
        .replace(/^./, (str) => str.toUpperCase())
        .trim();
}
function getPermissionDescription(permission) {
    const descriptions = {
        'geometry:read': 'Read geometry data',
        'geometry:write': 'Create and modify geometry',
        'geometry:delete': 'Delete geometry entities',
        'rendering:read': 'Read rendering state',
        'rendering:write': 'Modify rendering settings',
        'ui:read': 'Read UI state',
        'ui:write': 'Modify UI elements',
        'ui:menu': 'Add menu items',
        'ui:toolbar': 'Add toolbar buttons',
        'file:read': 'Read files from disk',
        'file:write': 'Write files to disk',
        'command:execute': 'Execute commands',
        'network:http': 'Make HTTP requests',
        'system:clipboard': 'Access clipboard',
        'system:notifications': 'Show notifications',
    };
    return descriptions[permission] || 'Unknown permission';
}
export default PluginSettings;
//# sourceMappingURL=PluginSettings.js.map