import { PluginSDK } from './PluginSDK';
import { PluginInfo, PluginState, PluginEvent, PluginEventType, PluginSystemStats, SearchFilters, SearchResults, PluginUpdate, PluginSettings } from './types';
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
declare global {
    interface Window {
        __caddyPluginManager?: PluginManagerAPI;
    }
}
export declare function usePlugins(): {
    plugins: PluginInfo[];
    loading: boolean;
    error: Error | null;
    refresh: () => Promise<void>;
};
export declare function usePlugin(pluginId: string): {
    plugin: PluginInfo | null;
    loading: boolean;
    error: Error | null;
    refresh: () => Promise<void>;
    load: () => Promise<void>;
    unload: () => Promise<void>;
    enable: () => Promise<void>;
    disable: () => Promise<void>;
    uninstall: () => Promise<void>;
};
export declare function useMarketplace(initialFilters?: SearchFilters): {
    results: SearchResults | null;
    loading: boolean;
    error: Error | null;
    search: (filters: SearchFilters) => Promise<void>;
    install: (pluginId: string) => Promise<void>;
};
export declare function usePluginUpdates(): {
    updates: PluginUpdate[];
    loading: boolean;
    error: Error | null;
    checkUpdates: () => Promise<void>;
    updatePlugin: (pluginId: string) => Promise<void>;
    updateAll: () => Promise<void>;
    hasUpdates: boolean;
};
export declare function usePluginStats(): {
    stats: PluginSystemStats | null;
    loading: boolean;
    error: Error | null;
    refresh: () => Promise<void>;
};
export declare function usePluginSettings(pluginId: string): {
    settings: PluginSettings | null;
    loading: boolean;
    error: Error | null;
    refresh: () => Promise<void>;
    updateSettings: (updates: Partial<PluginSettings>) => Promise<void>;
};
export declare function usePluginEvents(pluginId: string | null, eventTypes?: PluginEventType[]): {
    events: PluginEvent[];
    clearEvents: () => void;
    latestEvent: PluginEvent;
};
export declare function usePluginSDK(pluginId: string): {
    sdk: PluginSDK | null;
    connected: boolean;
    error: Error | null;
};
export declare function useFilteredPlugins(filterFn: (plugin: PluginInfo) => boolean): {
    plugins: PluginInfo[];
    loading: boolean;
    error: Error | null;
    refresh: () => Promise<void>;
};
export declare function usePluginsByState(state: PluginState): {
    plugins: PluginInfo[];
    loading: boolean;
    error: Error | null;
    refresh: () => Promise<void>;
};
export declare function useEnabledPlugins(): {
    plugins: PluginInfo[];
    loading: boolean;
    error: Error | null;
    refresh: () => Promise<void>;
};
export declare function useRunningPlugins(): {
    plugins: PluginInfo[];
    loading: boolean;
    error: Error | null;
    refresh: () => Promise<void>;
};
declare const _default: {
    usePlugins: typeof usePlugins;
    usePlugin: typeof usePlugin;
    useMarketplace: typeof useMarketplace;
    usePluginUpdates: typeof usePluginUpdates;
    usePluginStats: typeof usePluginStats;
    usePluginSettings: typeof usePluginSettings;
    usePluginEvents: typeof usePluginEvents;
    usePluginSDK: typeof usePluginSDK;
    useFilteredPlugins: typeof useFilteredPlugins;
    usePluginsByState: typeof usePluginsByState;
    useEnabledPlugins: typeof useEnabledPlugins;
    useRunningPlugins: typeof useRunningPlugins;
};
export default _default;
//# sourceMappingURL=usePlugin.d.ts.map