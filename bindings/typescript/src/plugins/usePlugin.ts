/**
 * React Hooks for Plugin Integration
 *
 * Custom React hooks for managing plugins, accessing plugin APIs,
 * and subscribing to plugin events.
 */

import { useState, useEffect, useCallback, useRef, useMemo } from 'react';
import { PluginSDK } from './PluginSDK';
import {
  PluginInfo,
  PluginState,
  PluginEvent,
  PluginEventType,
  PluginSystemStats,
  SearchFilters,
  SearchResults,
  MarketplacePlugin,
  PluginUpdate,
  PluginSettings,
} from './types';

/**
 * Plugin manager API interface
 */
interface PluginManagerAPI {
  listPlugins: () => Promise<PluginInfo[]>;
  getPlugin: (pluginId: string) => Promise<PluginInfo | null>;
  loadPlugin: (pluginId: string) => Promise<void>;
  unloadPlugin: (pluginId: string) => Promise<void>;
  enablePlugin: (pluginId: string) => Promise<void>;
  disablePlugin: (pluginId: string) => Promise<void>;
  installPlugin: (pluginId: string) => Promise<void>;
  uninstallPlugin: (pluginId: string) => Promise<void>;
  searchMarketplace: (filters: SearchFilters) => Promise<SearchResults>;
  checkUpdates: () => Promise<PluginUpdate[]>;
  updatePlugin: (pluginId: string) => Promise<void>;
  getStats: () => Promise<PluginSystemStats>;
  getPluginSettings: (pluginId: string) => Promise<PluginSettings>;
  updatePluginSettings: (pluginId: string, settings: Partial<PluginSettings>) => Promise<void>;
}

/**
 * Global plugin manager instance (injected by host application)
 */
declare global {
  interface Window {
    __caddyPluginManager?: PluginManagerAPI;
  }
}

/**
 * Get plugin manager API
 */
function getPluginManager(): PluginManagerAPI {
  if (!window.__caddyPluginManager) {
    throw new Error('Plugin manager not available');
  }
  return window.__caddyPluginManager;
}

/**
 * Hook to access list of plugins
 */
export function usePlugins() {
  const [plugins, setPlugins] = useState<PluginInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const refresh = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const manager = getPluginManager();
      const list = await manager.listPlugins();
      setPlugins(list);
    } catch (err) {
      setError(err as Error);
    } finally {
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

/**
 * Hook to access a specific plugin
 */
export function usePlugin(pluginId: string) {
  const [plugin, setPlugin] = useState<PluginInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const refresh = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const manager = getPluginManager();
      const info = await manager.getPlugin(pluginId);
      setPlugin(info);
    } catch (err) {
      setError(err as Error);
    } finally {
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

/**
 * Hook to search marketplace
 */
export function useMarketplace(initialFilters?: SearchFilters) {
  const [results, setResults] = useState<SearchResults | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const search = useCallback(async (filters: SearchFilters) => {
    try {
      setLoading(true);
      setError(null);
      const manager = getPluginManager();
      const searchResults = await manager.searchMarketplace(filters);
      setResults(searchResults);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (initialFilters) {
      search(initialFilters);
    }
  }, []);

  const install = useCallback(async (pluginId: string) => {
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

/**
 * Hook to check for plugin updates
 */
export function usePluginUpdates() {
  const [updates, setUpdates] = useState<PluginUpdate[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const checkUpdates = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const manager = getPluginManager();
      const availableUpdates = await manager.checkUpdates();
      setUpdates(availableUpdates);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  }, []);

  const updatePlugin = useCallback(async (pluginId: string) => {
    const manager = getPluginManager();
    await manager.updatePlugin(pluginId);
    await checkUpdates();
  }, [checkUpdates]);

  const updateAll = useCallback(async () => {
    const manager = getPluginManager();
    for (const update of updates) {
      try {
        await manager.updatePlugin(update.pluginId);
      } catch (err) {
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

/**
 * Hook to access plugin statistics
 */
export function usePluginStats() {
  const [stats, setStats] = useState<PluginSystemStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const refresh = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const manager = getPluginManager();
      const systemStats = await manager.getStats();
      setStats(systemStats);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
    // Refresh every 5 seconds
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

/**
 * Hook to manage plugin settings
 */
export function usePluginSettings(pluginId: string) {
  const [settings, setSettings] = useState<PluginSettings | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const refresh = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const manager = getPluginManager();
      const pluginSettings = await manager.getPluginSettings(pluginId);
      setSettings(pluginSettings);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  }, [pluginId]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const updateSettings = useCallback(async (updates: Partial<PluginSettings>) => {
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

/**
 * Hook to subscribe to plugin events
 */
export function usePluginEvents(
  pluginId: string | null,
  eventTypes?: PluginEventType[]
) {
  const [events, setEvents] = useState<PluginEvent[]>([]);
  const eventListenerRef = useRef<((event: CustomEvent) => void) | null>(null);

  useEffect(() => {
    const handleEvent = (event: CustomEvent<PluginEvent>) => {
      const pluginEvent = event.detail;

      // Filter by plugin ID if specified
      if (pluginId && pluginEvent.pluginId !== pluginId) {
        return;
      }

      // Filter by event types if specified
      if (eventTypes && !eventTypes.includes(pluginEvent.type)) {
        return;
      }

      setEvents((prev) => [...prev, pluginEvent]);
    };

    eventListenerRef.current = handleEvent;
    window.addEventListener('caddyPluginEvent', handleEvent as EventListener);

    return () => {
      if (eventListenerRef.current) {
        window.removeEventListener('caddyPluginEvent', eventListenerRef.current as EventListener);
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

/**
 * Hook to create and manage a plugin SDK instance
 */
export function usePluginSDK(pluginId: string) {
  const [sdk, setSdk] = useState<PluginSDK | null>(null);
  const [connected, setConnected] = useState(false);
  const [error, setError] = useState<Error | null>(null);

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

/**
 * Hook to filter plugins
 */
export function useFilteredPlugins(
  filterFn: (plugin: PluginInfo) => boolean
) {
  const { plugins, loading, error, refresh } = usePlugins();

  const filteredPlugins = useMemo(
    () => plugins.filter(filterFn),
    [plugins, filterFn]
  );

  return {
    plugins: filteredPlugins,
    loading,
    error,
    refresh,
  };
}

/**
 * Hook to get plugins by state
 */
export function usePluginsByState(state: PluginState) {
  return useFilteredPlugins(
    useCallback((plugin) => plugin.state === state, [state])
  );
}

/**
 * Hook to get enabled plugins
 */
export function useEnabledPlugins() {
  return useFilteredPlugins(
    useCallback((plugin) => plugin.enabled, [])
  );
}

/**
 * Hook to get running plugins
 */
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
