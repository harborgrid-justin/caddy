/**
 * CADDY Enterprise Plugin System
 *
 * Complete plugin architecture for extending CADDY functionality.
 * Includes SDK, UI components, hooks, and type definitions.
 *
 * @module @caddy/enterprise-sdk/plugins
 */

// Type definitions
export * from './types';

// Plugin SDK
export { PluginSDK, createPluginSDK, Plugin } from './PluginSDK';

// React Hooks
export {
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
} from './usePlugin';

// UI Components
export { PluginManager } from './PluginManager';
export { PluginMarketplace } from './PluginMarketplace';
export { PluginSettings } from './PluginSettings';
export { PluginHost, PluginContainer } from './PluginHost';

// Default export for convenience
export { PluginSDK as default } from './PluginSDK';
