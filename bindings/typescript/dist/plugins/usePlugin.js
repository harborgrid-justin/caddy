import { useState, useEffect, useCallback, useRef, useMemo } from 'react';
import { PluginSDK } from './PluginSDK';
import { PluginState, } from './types';
function getPluginManager() {
    if (!window.__caddyPluginManager) {
        throw new Error('Plugin manager not available');
    }
    return window.__caddyPluginManager;
}
export function usePlugins() {
    const [plugins, setPlugins] = useState([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const refresh = useCallback(async () => {
        try {
            setLoading(true);
            setError(null);
            const manager = getPluginManager();
            const list = await manager.listPlugins();
            setPlugins(list);
        }
        catch (err) {
            setError(err);
        }
        finally {
            setLoading(false);
        }
    }, []);
    useEffect(() => {
        refresh();
    }, [refresh]);
    return {
        plugins,
        loading,
        error,
        refresh,
    };
}
export function usePlugin(pluginId) {
    const [plugin, setPlugin] = useState(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const refresh = useCallback(async () => {
        try {
            setLoading(true);
            setError(null);
            const manager = getPluginManager();
            const info = await manager.getPlugin(pluginId);
            setPlugin(info);
        }
        catch (err) {
            setError(err);
        }
        finally {
            setLoading(false);
        }
    }, [pluginId]);
    useEffect(() => {
        refresh();
    }, [refresh]);
    const load = useCallback(async () => {
        const manager = getPluginManager();
        await manager.loadPlugin(pluginId);
        await refresh();
    }, [pluginId, refresh]);
    const unload = useCallback(async () => {
        const manager = getPluginManager();
        await manager.unloadPlugin(pluginId);
        await refresh();
    }, [pluginId, refresh]);
    const enable = useCallback(async () => {
        const manager = getPluginManager();
        await manager.enablePlugin(pluginId);
        await refresh();
    }, [pluginId, refresh]);
    const disable = useCallback(async () => {
        const manager = getPluginManager();
        await manager.disablePlugin(pluginId);
        await refresh();
    }, [pluginId, refresh]);
    const uninstall = useCallback(async () => {
        const manager = getPluginManager();
        await manager.uninstallPlugin(pluginId);
        await refresh();
    }, [pluginId, refresh]);
    return {
        plugin,
        loading,
        error,
        refresh,
        load,
        unload,
        enable,
        disable,
        uninstall,
    };
}
export function useMarketplace(initialFilters) {
    const [results, setResults] = useState(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState(null);
    const search = useCallback(async (filters) => {
        try {
            setLoading(true);
            setError(null);
            const manager = getPluginManager();
            const searchResults = await manager.searchMarketplace(filters);
            setResults(searchResults);
        }
        catch (err) {
            setError(err);
        }
        finally {
            setLoading(false);
        }
    }, []);
    useEffect(() => {
        if (initialFilters) {
            search(initialFilters);
        }
    }, []);
    const install = useCallback(async (pluginId) => {
        const manager = getPluginManager();
        await manager.installPlugin(pluginId);
    }, []);
    return {
        results,
        loading,
        error,
        search,
        install,
    };
}
export function usePluginUpdates() {
    const [updates, setUpdates] = useState([]);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState(null);
    const checkUpdates = useCallback(async () => {
        try {
            setLoading(true);
            setError(null);
            const manager = getPluginManager();
            const availableUpdates = await manager.checkUpdates();
            setUpdates(availableUpdates);
        }
        catch (err) {
            setError(err);
        }
        finally {
            setLoading(false);
        }
    }, []);
    const updatePlugin = useCallback(async (pluginId) => {
        const manager = getPluginManager();
        await manager.updatePlugin(pluginId);
        await checkUpdates();
    }, [checkUpdates]);
    const updateAll = useCallback(async () => {
        const manager = getPluginManager();
        for (const update of updates) {
            try {
                await manager.updatePlugin(update.pluginId);
            }
            catch (err) {
                console.error(`Failed to update ${update.pluginId}:`, err);
            }
        }
        await checkUpdates();
    }, [updates, checkUpdates]);
    useEffect(() => {
        checkUpdates();
    }, [checkUpdates]);
    return {
        updates,
        loading,
        error,
        checkUpdates,
        updatePlugin,
        updateAll,
        hasUpdates: updates.length > 0,
    };
}
export function usePluginStats() {
    const [stats, setStats] = useState(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const refresh = useCallback(async () => {
        try {
            setLoading(true);
            setError(null);
            const manager = getPluginManager();
            const systemStats = await manager.getStats();
            setStats(systemStats);
        }
        catch (err) {
            setError(err);
        }
        finally {
            setLoading(false);
        }
    }, []);
    useEffect(() => {
        refresh();
        const interval = setInterval(refresh, 5000);
        return () => clearInterval(interval);
    }, [refresh]);
    return {
        stats,
        loading,
        error,
        refresh,
    };
}
export function usePluginSettings(pluginId) {
    const [settings, setSettings] = useState(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const refresh = useCallback(async () => {
        try {
            setLoading(true);
            setError(null);
            const manager = getPluginManager();
            const pluginSettings = await manager.getPluginSettings(pluginId);
            setSettings(pluginSettings);
        }
        catch (err) {
            setError(err);
        }
        finally {
            setLoading(false);
        }
    }, [pluginId]);
    useEffect(() => {
        refresh();
    }, [refresh]);
    const updateSettings = useCallback(async (updates) => {
        const manager = getPluginManager();
        await manager.updatePluginSettings(pluginId, updates);
        await refresh();
    }, [pluginId, refresh]);
    return {
        settings,
        loading,
        error,
        refresh,
        updateSettings,
    };
}
export function usePluginEvents(pluginId, eventTypes) {
    const [events, setEvents] = useState([]);
    const eventListenerRef = useRef(null);
    useEffect(() => {
        const handleEvent = (event) => {
            const pluginEvent = event.detail;
            if (pluginId && pluginEvent.pluginId !== pluginId) {
                return;
            }
            if (eventTypes && !eventTypes.includes(pluginEvent.type)) {
                return;
            }
            setEvents((prev) => [...prev, pluginEvent]);
        };
        eventListenerRef.current = handleEvent;
        window.addEventListener('caddyPluginEvent', handleEvent);
        return () => {
            if (eventListenerRef.current) {
                window.removeEventListener('caddyPluginEvent', eventListenerRef.current);
            }
        };
    }, [pluginId, eventTypes]);
    const clearEvents = useCallback(() => {
        setEvents([]);
    }, []);
    return {
        events,
        clearEvents,
        latestEvent: events[events.length - 1] || null,
    };
}
export function usePluginSDK(pluginId) {
    const [sdk, setSdk] = useState(null);
    const [connected, setConnected] = useState(false);
    const [error, setError] = useState(null);
    useEffect(() => {
        const manager = getPluginManager();
        manager.getPlugin(pluginId).then((plugin) => {
            if (!plugin) {
                setError(new Error(`Plugin ${pluginId} not found`));
                return;
            }
            const context = {
                pluginId: plugin.manifest.id,
                apiVersion: plugin.manifest.apiVersion,
                permissions: plugin.manifest.permissions,
                config: {},
            };
            const pluginSDK = new PluginSDK(context);
            pluginSDK.on('connected', () => setConnected(true));
            pluginSDK.on('disconnected', () => setConnected(false));
            pluginSDK.on('error', (err) => setError(err));
            pluginSDK.initialize().catch(setError);
            setSdk(pluginSDK);
            return () => {
                pluginSDK.disconnect();
            };
        });
    }, [pluginId]);
    return {
        sdk,
        connected,
        error,
    };
}
export function useFilteredPlugins(filterFn) {
    const { plugins, loading, error, refresh } = usePlugins();
    const filteredPlugins = useMemo(() => plugins.filter(filterFn), [plugins, filterFn]);
    return {
        plugins: filteredPlugins,
        loading,
        error,
        refresh,
    };
}
export function usePluginsByState(state) {
    return useFilteredPlugins(useCallback((plugin) => plugin.state === state, [state]));
}
export function useEnabledPlugins() {
    return useFilteredPlugins(useCallback((plugin) => plugin.enabled, []));
}
export function useRunningPlugins() {
    return usePluginsByState(PluginState.Running);
}
export default {
    usePlugins,
    usePlugin,
    useMarketplace,
    usePluginUpdates,
    usePluginStats,
    usePluginSettings,
    usePluginEvents,
    usePluginSDK,
    useFilteredPlugins,
    usePluginsByState,
    useEnabledPlugins,
    useRunningPlugins,
};
//# sourceMappingURL=usePlugin.js.map