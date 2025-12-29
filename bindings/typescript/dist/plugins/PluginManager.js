import React, { useState, useMemo } from 'react';
import { usePlugins, usePluginUpdates, usePluginStats, } from './usePlugin';
import { PluginState } from './types';
export const PluginManager = ({ className = '', onOpenSettings, onOpenMarketplace, }) => {
    const { plugins, loading, error, refresh } = usePlugins();
    const { updates, hasUpdates } = usePluginUpdates();
    const { stats } = usePluginStats();
    const [searchQuery, setSearchQuery] = useState('');
    const [filterState, setFilterState] = useState('all');
    const [sortBy, setSortBy] = useState('name');
    const filteredPlugins = useMemo(() => {
        let result = [...plugins];
        if (searchQuery) {
            const query = searchQuery.toLowerCase();
            result = result.filter((plugin) => plugin.manifest.name.toLowerCase().includes(query) ||
                plugin.manifest.description.toLowerCase().includes(query) ||
                plugin.manifest.author.toLowerCase().includes(query));
        }
        if (filterState === 'enabled') {
            result = result.filter((plugin) => plugin.enabled);
        }
        else if (filterState === 'disabled') {
            result = result.filter((plugin) => !plugin.enabled);
        }
        result.sort((a, b) => {
            switch (sortBy) {
                case 'name':
                    return a.manifest.name.localeCompare(b.manifest.name);
                case 'author':
                    return a.manifest.author.localeCompare(b.manifest.author);
                case 'state':
                    return a.state.localeCompare(b.state);
                default:
                    return 0;
            }
        });
        return result;
    }, [plugins, searchQuery, filterState, sortBy]);
    if (loading && plugins.length === 0) {
        return (React.createElement("div", { className: `plugin-manager loading ${className}` },
            React.createElement("div", { className: "loading-spinner" }, "Loading plugins...")));
    }
    if (error) {
        return (React.createElement("div", { className: `plugin-manager error ${className}` },
            React.createElement("div", { className: "error-message" },
                React.createElement("h3", null, "Error loading plugins"),
                React.createElement("p", null, error.message),
                React.createElement("button", { onClick: refresh }, "Retry"))));
    }
    return (React.createElement("div", { className: `plugin-manager ${className}` },
        React.createElement("div", { className: "plugin-manager-header" },
            React.createElement("h1", null, "Plugin Manager"),
            React.createElement("div", { className: "header-actions" },
                React.createElement("button", { className: "btn-primary", onClick: onOpenMarketplace }, "Browse Marketplace"),
                React.createElement("button", { className: "btn-secondary", onClick: refresh }, "Refresh"))),
        stats && (React.createElement("div", { className: "plugin-stats" },
            React.createElement("div", { className: "stat-card" },
                React.createElement("div", { className: "stat-value" }, stats.loadedPlugins),
                React.createElement("div", { className: "stat-label" }, "Loaded")),
            React.createElement("div", { className: "stat-card" },
                React.createElement("div", { className: "stat-value" }, stats.runningPlugins),
                React.createElement("div", { className: "stat-label" }, "Running")),
            React.createElement("div", { className: "stat-card" },
                React.createElement("div", { className: "stat-value" }, stats.enabledPlugins),
                React.createElement("div", { className: "stat-label" }, "Enabled")),
            hasUpdates && (React.createElement("div", { className: "stat-card highlight" },
                React.createElement("div", { className: "stat-value" }, updates.length),
                React.createElement("div", { className: "stat-label" }, "Updates Available"))))),
        React.createElement("div", { className: "plugin-filters" },
            React.createElement("input", { type: "search", className: "search-input", placeholder: "Search plugins...", value: searchQuery, onChange: (e) => setSearchQuery(e.target.value) }),
            React.createElement("select", { className: "filter-select", value: filterState, onChange: (e) => setFilterState(e.target.value) },
                React.createElement("option", { value: "all" }, "All Plugins"),
                React.createElement("option", { value: "enabled" }, "Enabled Only"),
                React.createElement("option", { value: "disabled" }, "Disabled Only")),
            React.createElement("select", { className: "sort-select", value: sortBy, onChange: (e) => setSortBy(e.target.value) },
                React.createElement("option", { value: "name" }, "Sort by Name"),
                React.createElement("option", { value: "author" }, "Sort by Author"),
                React.createElement("option", { value: "state" }, "Sort by State"))),
        React.createElement("div", { className: "plugin-list" }, filteredPlugins.length === 0 ? (React.createElement("div", { className: "empty-state" },
            React.createElement("p", null, "No plugins found."),
            onOpenMarketplace && (React.createElement("button", { onClick: onOpenMarketplace }, "Browse Marketplace")))) : (filteredPlugins.map((plugin) => (React.createElement(PluginCard, { key: plugin.manifest.id, plugin: plugin, hasUpdate: updates.some((u) => u.pluginId === plugin.manifest.id), onOpenSettings: onOpenSettings, onRefresh: refresh })))))));
};
const PluginCard = ({ plugin, hasUpdate, onOpenSettings, onRefresh, }) => {
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState(null);
    const handleToggleEnable = async () => {
        setLoading(true);
        setError(null);
        try {
            const manager = window.__caddyPluginManager;
            if (plugin.enabled) {
                await manager.disablePlugin(plugin.manifest.id);
            }
            else {
                await manager.enablePlugin(plugin.manifest.id);
            }
            onRefresh();
        }
        catch (err) {
            setError(err.message);
        }
        finally {
            setLoading(false);
        }
    };
    const handleUninstall = async () => {
        if (!confirm(`Are you sure you want to uninstall ${plugin.manifest.name}?`)) {
            return;
        }
        setLoading(true);
        setError(null);
        try {
            const manager = window.__caddyPluginManager;
            await manager.uninstallPlugin(plugin.manifest.id);
            onRefresh();
        }
        catch (err) {
            setError(err.message);
        }
        finally {
            setLoading(false);
        }
    };
    const getStateColor = (state) => {
        switch (state) {
            case PluginState.Running:
                return 'green';
            case PluginState.Error:
                return 'red';
            case PluginState.Stopped:
                return 'gray';
            default:
                return 'blue';
        }
    };
    return (React.createElement("div", { className: `plugin-card ${plugin.enabled ? 'enabled' : 'disabled'}` },
        React.createElement("div", { className: "plugin-card-header" },
            React.createElement("div", { className: "plugin-icon" }, plugin.manifest.icon ? (React.createElement("img", { src: plugin.manifest.icon, alt: plugin.manifest.name })) : (React.createElement("div", { className: "default-icon" }, plugin.manifest.name[0]))),
            React.createElement("div", { className: "plugin-info" },
                React.createElement("h3", null, plugin.manifest.name),
                React.createElement("p", { className: "plugin-author" },
                    "by ",
                    plugin.manifest.author),
                React.createElement("p", { className: "plugin-description" }, plugin.manifest.description)),
            React.createElement("div", { className: "plugin-status" },
                React.createElement("span", { className: `status-badge status-${getStateColor(plugin.state)}` }, plugin.state),
                hasUpdate && React.createElement("span", { className: "update-badge" }, "Update Available"))),
        React.createElement("div", { className: "plugin-card-footer" },
            React.createElement("div", { className: "plugin-meta" },
                React.createElement("span", null,
                    "v",
                    plugin.manifest.version),
                React.createElement("span", null, plugin.manifest.pluginType),
                plugin.manifest.license && React.createElement("span", null, plugin.manifest.license)),
            React.createElement("div", { className: "plugin-actions" },
                React.createElement("button", { className: "btn-toggle", onClick: handleToggleEnable, disabled: loading }, plugin.enabled ? 'Disable' : 'Enable'),
                onOpenSettings && (React.createElement("button", { className: "btn-secondary", onClick: () => onOpenSettings(plugin.manifest.id), disabled: loading }, "Settings")),
                React.createElement("button", { className: "btn-danger", onClick: handleUninstall, disabled: loading }, "Uninstall"))),
        error && (React.createElement("div", { className: "plugin-error" },
            React.createElement("p", null, error))),
        loading && React.createElement("div", { className: "plugin-loading-overlay" }, "Processing...")));
};
export default PluginManager;
//# sourceMappingURL=PluginManager.js.map