/**
 * Plugin Manager UI Component
 *
 * Main plugin management interface for viewing, enabling/disabling,
 * and managing installed plugins.
 */

import React, { useState, useMemo } from 'react';
import {
  usePlugins,
  usePluginUpdates,
  usePluginStats,
} from './usePlugin';
import { PluginInfo, PluginState } from './types';

export interface PluginManagerProps {
  className?: string;
  onOpenSettings?: (pluginId: string) => void;
  onOpenMarketplace?: () => void;
}

export const PluginManager: React.FC<PluginManagerProps> = ({
  className = '',
  onOpenSettings,
  onOpenMarketplace,
}) => {
  const { plugins, loading, error, refresh } = usePlugins();
  const { updates, hasUpdates } = usePluginUpdates();
  const { stats } = usePluginStats();

  const [searchQuery, setSearchQuery] = useState('');
  const [filterState, setFilterState] = useState<'all' | 'enabled' | 'disabled'>('all');
  const [sortBy, setSortBy] = useState<'name' | 'author' | 'state'>('name');

  // Filter and sort plugins
  const filteredPlugins = useMemo(() => {
    let result = [...plugins];

    // Apply search filter
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      result = result.filter(
        (plugin) =>
          plugin.manifest.name.toLowerCase().includes(query) ||
          plugin.manifest.description.toLowerCase().includes(query) ||
          plugin.manifest.author.toLowerCase().includes(query)
      );
    }

    // Apply state filter
    if (filterState === 'enabled') {
      result = result.filter((plugin) => plugin.enabled);
    } else if (filterState === 'disabled') {
      result = result.filter((plugin) => !plugin.enabled);
    }

    // Sort
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
    return (
      <div className={`plugin-manager loading ${className}`}>
        <div className="loading-spinner">Loading plugins...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={`plugin-manager error ${className}`}>
        <div className="error-message">
          <h3>Error loading plugins</h3>
          <p>{error.message}</p>
          <button onClick={refresh}>Retry</button>
        </div>
      </div>
    );
  }

  return (
    <div className={`plugin-manager ${className}`}>
      {/* Header */}
      <div className="plugin-manager-header">
        <h1>Plugin Manager</h1>

        <div className="header-actions">
          <button className="btn-primary" onClick={onOpenMarketplace}>
            Browse Marketplace
          </button>
          <button className="btn-secondary" onClick={refresh}>
            Refresh
          </button>
        </div>
      </div>

      {/* Statistics */}
      {stats && (
        <div className="plugin-stats">
          <div className="stat-card">
            <div className="stat-value">{stats.loadedPlugins}</div>
            <div className="stat-label">Loaded</div>
          </div>
          <div className="stat-card">
            <div className="stat-value">{stats.runningPlugins}</div>
            <div className="stat-label">Running</div>
          </div>
          <div className="stat-card">
            <div className="stat-value">{stats.enabledPlugins}</div>
            <div className="stat-label">Enabled</div>
          </div>
          {hasUpdates && (
            <div className="stat-card highlight">
              <div className="stat-value">{updates.length}</div>
              <div className="stat-label">Updates Available</div>
            </div>
          )}
        </div>
      )}

      {/* Filters and Search */}
      <div className="plugin-filters">
        <input
          type="search"
          className="search-input"
          placeholder="Search plugins..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
        />

        <select
          className="filter-select"
          value={filterState}
          onChange={(e) => setFilterState(e.target.value as any)}
        >
          <option value="all">All Plugins</option>
          <option value="enabled">Enabled Only</option>
          <option value="disabled">Disabled Only</option>
        </select>

        <select
          className="sort-select"
          value={sortBy}
          onChange={(e) => setSortBy(e.target.value as any)}
        >
          <option value="name">Sort by Name</option>
          <option value="author">Sort by Author</option>
          <option value="state">Sort by State</option>
        </select>
      </div>

      {/* Plugin List */}
      <div className="plugin-list">
        {filteredPlugins.length === 0 ? (
          <div className="empty-state">
            <p>No plugins found.</p>
            {onOpenMarketplace && (
              <button onClick={onOpenMarketplace}>
                Browse Marketplace
              </button>
            )}
          </div>
        ) : (
          filteredPlugins.map((plugin) => (
            <PluginCard
              key={plugin.manifest.id}
              plugin={plugin}
              hasUpdate={updates.some((u) => u.pluginId === plugin.manifest.id)}
              onOpenSettings={onOpenSettings}
              onRefresh={refresh}
            />
          ))
        )}
      </div>
    </div>
  );
};

interface PluginCardProps {
  plugin: PluginInfo;
  hasUpdate: boolean;
  onOpenSettings?: (pluginId: string) => void;
  onRefresh: () => void;
}

const PluginCard: React.FC<PluginCardProps> = ({
  plugin,
  hasUpdate,
  onOpenSettings,
  onRefresh,
}) => {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleToggleEnable = async () => {
    setLoading(true);
    setError(null);

    try {
      const manager = (window as any).__caddyPluginManager;
      if (plugin.enabled) {
        await manager.disablePlugin(plugin.manifest.id);
      } else {
        await manager.enablePlugin(plugin.manifest.id);
      }
      onRefresh();
    } catch (err: any) {
      setError(err.message);
    } finally {
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
      const manager = (window as any).__caddyPluginManager;
      await manager.uninstallPlugin(plugin.manifest.id);
      onRefresh();
    } catch (err: any) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const getStateColor = (state: PluginState): string => {
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

  return (
    <div className={`plugin-card ${plugin.enabled ? 'enabled' : 'disabled'}`}>
      <div className="plugin-card-header">
        <div className="plugin-icon">
          {plugin.manifest.icon ? (
            <img src={plugin.manifest.icon} alt={plugin.manifest.name} />
          ) : (
            <div className="default-icon">{plugin.manifest.name[0]}</div>
          )}
        </div>

        <div className="plugin-info">
          <h3>{plugin.manifest.name}</h3>
          <p className="plugin-author">by {plugin.manifest.author}</p>
          <p className="plugin-description">{plugin.manifest.description}</p>
        </div>

        <div className="plugin-status">
          <span className={`status-badge status-${getStateColor(plugin.state)}`}>
            {plugin.state}
          </span>
          {hasUpdate && <span className="update-badge">Update Available</span>}
        </div>
      </div>

      <div className="plugin-card-footer">
        <div className="plugin-meta">
          <span>v{plugin.manifest.version}</span>
          <span>{plugin.manifest.pluginType}</span>
          {plugin.manifest.license && <span>{plugin.manifest.license}</span>}
        </div>

        <div className="plugin-actions">
          <button
            className="btn-toggle"
            onClick={handleToggleEnable}
            disabled={loading}
          >
            {plugin.enabled ? 'Disable' : 'Enable'}
          </button>

          {onOpenSettings && (
            <button
              className="btn-secondary"
              onClick={() => onOpenSettings(plugin.manifest.id)}
              disabled={loading}
            >
              Settings
            </button>
          )}

          <button
            className="btn-danger"
            onClick={handleUninstall}
            disabled={loading}
          >
            Uninstall
          </button>
        </div>
      </div>

      {error && (
        <div className="plugin-error">
          <p>{error}</p>
        </div>
      )}

      {loading && <div className="plugin-loading-overlay">Processing...</div>}
    </div>
  );
};

export default PluginManager;
