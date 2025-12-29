/**
 * Plugin Marketplace UI Component
 *
 * Browse, search, and install plugins from the CADDY marketplace.
 */

import React, { useState, useEffect } from 'react';
import { useMarketplace } from './usePlugin';
import {
  MarketplacePlugin,
  SearchFilters,
  SortBy,
  Category,
} from './types';

export interface PluginMarketplaceProps {
  className?: string;
  onClose?: () => void;
  onPluginInstalled?: (pluginId: string) => void;
}

export const PluginMarketplace: React.FC<PluginMarketplaceProps> = ({
  className = '',
  onClose,
  onPluginInstalled,
}) => {
  const { results, loading, error, search, install } = useMarketplace();

  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string>('');
  const [sortBy, setSortBy] = useState<SortBy>(SortBy.Relevance);
  const [verifiedOnly, setVerifiedOnly] = useState(false);
  const [minRating, setMinRating] = useState<number | undefined>(undefined);
  const [currentPage, setCurrentPage] = useState(0);
  const [categories, setCategories] = useState<Category[]>([]);

  const perPage = 12;

  // Load categories on mount
  useEffect(() => {
    loadCategories();
  }, []);

  // Perform search when filters change
  useEffect(() => {
    performSearch();
  }, [searchQuery, selectedCategory, sortBy, verifiedOnly, minRating, currentPage]);

  const loadCategories = async () => {
    try {
      const manager = (window as any).__caddyPluginManager;
      // In a real implementation, this would fetch from the API
      const mockCategories: Category[] = [
        { id: 'modeling', name: 'Modeling', description: 'Tools for 3D modeling', pluginCount: 15 },
        { id: 'rendering', name: 'Rendering', description: 'Rendering and visualization', pluginCount: 8 },
        { id: 'import-export', name: 'Import/Export', description: 'File format converters', pluginCount: 12 },
        { id: 'automation', name: 'Automation', description: 'Workflow automation', pluginCount: 6 },
        { id: 'analysis', name: 'Analysis', description: 'Engineering analysis tools', pluginCount: 10 },
      ];
      setCategories(mockCategories);
    } catch (err) {
      console.error('Failed to load categories:', err);
    }
  };

  const performSearch = async () => {
    const filters: SearchFilters = {
      query: searchQuery || undefined,
      category: selectedCategory || undefined,
      sortBy,
      verifiedOnly,
      minRating,
      page: currentPage,
      perPage,
    };

    await search(filters);
  };

  const handleInstall = async (plugin: MarketplacePlugin) => {
    try {
      await install(plugin.id);
      onPluginInstalled?.(plugin.id);
      alert(`Successfully installed ${plugin.name}!`);
    } catch (err: any) {
      alert(`Failed to install plugin: ${err.message}`);
    }
  };

  const handlePageChange = (newPage: number) => {
    setCurrentPage(newPage);
  };

  return (
    <div className={`plugin-marketplace ${className}`}>
      {/* Header */}
      <div className="marketplace-header">
        <h1>Plugin Marketplace</h1>
        {onClose && (
          <button className="btn-close" onClick={onClose}>
            Close
          </button>
        )}
      </div>

      {/* Sidebar */}
      <div className="marketplace-layout">
        <aside className="marketplace-sidebar">
          <div className="sidebar-section">
            <h3>Categories</h3>
            <ul className="category-list">
              <li>
                <button
                  className={selectedCategory === '' ? 'active' : ''}
                  onClick={() => setSelectedCategory('')}
                >
                  All Categories
                </button>
              </li>
              {categories.map((category) => (
                <li key={category.id}>
                  <button
                    className={selectedCategory === category.id ? 'active' : ''}
                    onClick={() => setSelectedCategory(category.id)}
                  >
                    {category.name}
                    <span className="count">({category.pluginCount})</span>
                  </button>
                </li>
              ))}
            </ul>
          </div>

          <div className="sidebar-section">
            <h3>Filters</h3>

            <label className="filter-option">
              <input
                type="checkbox"
                checked={verifiedOnly}
                onChange={(e) => setVerifiedOnly(e.target.checked)}
              />
              <span>Verified plugins only</span>
            </label>

            <div className="filter-option">
              <label>Minimum rating:</label>
              <select
                value={minRating || ''}
                onChange={(e) => setMinRating(e.target.value ? Number(e.target.value) : undefined)}
              >
                <option value="">Any</option>
                <option value="4">4+ Stars</option>
                <option value="3">3+ Stars</option>
                <option value="2">2+ Stars</option>
              </select>
            </div>
          </div>
        </aside>

        {/* Main Content */}
        <div className="marketplace-content">
          {/* Search and Sort */}
          <div className="marketplace-controls">
            <input
              type="search"
              className="search-input"
              placeholder="Search plugins..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />

            <select
              className="sort-select"
              value={sortBy}
              onChange={(e) => setSortBy(e.target.value as SortBy)}
            >
              <option value={SortBy.Relevance}>Most Relevant</option>
              <option value={SortBy.Downloads}>Most Downloads</option>
              <option value={SortBy.Rating}>Highest Rated</option>
              <option value={SortBy.Updated}>Recently Updated</option>
              <option value={SortBy.Name}>Name (A-Z)</option>
            </select>
          </div>

          {/* Results */}
          {loading && (
            <div className="loading-spinner">
              Searching marketplace...
            </div>
          )}

          {error && (
            <div className="error-message">
              <p>Failed to load marketplace: {error.message}</p>
              <button onClick={performSearch}>Retry</button>
            </div>
          )}

          {results && !loading && (
            <>
              <div className="results-info">
                <p>
                  Showing {results.plugins.length} of {results.totalCount} plugins
                  {selectedCategory && ` in ${categories.find((c) => c.id === selectedCategory)?.name}`}
                </p>
              </div>

              <div className="plugin-grid">
                {results.plugins.map((plugin) => (
                  <MarketplacePluginCard
                    key={plugin.id}
                    plugin={plugin}
                    onInstall={handleInstall}
                  />
                ))}
              </div>

              {results.totalPages > 1 && (
                <Pagination
                  currentPage={results.page}
                  totalPages={results.totalPages}
                  onPageChange={handlePageChange}
                />
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
};

interface MarketplacePluginCardProps {
  plugin: MarketplacePlugin;
  onInstall: (plugin: MarketplacePlugin) => void;
}

const MarketplacePluginCard: React.FC<MarketplacePluginCardProps> = ({
  plugin,
  onInstall,
}) => {
  const [installing, setInstalling] = useState(false);

  const handleInstall = async () => {
    setInstalling(true);
    try {
      await onInstall(plugin);
    } finally {
      setInstalling(false);
    }
  };

  const formatDownloads = (downloads: number): string => {
    if (downloads >= 1000000) {
      return `${(downloads / 1000000).toFixed(1)}M`;
    } else if (downloads >= 1000) {
      return `${(downloads / 1000).toFixed(1)}K`;
    }
    return downloads.toString();
  };

  const formatSize = (bytes: number): string => {
    if (bytes >= 1024 * 1024) {
      return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    } else if (bytes >= 1024) {
      return `${(bytes / 1024).toFixed(1)} KB`;
    }
    return `${bytes} B`;
  };

  return (
    <div className="marketplace-plugin-card">
      <div className="plugin-card-header">
        {plugin.iconUrl ? (
          <img src={plugin.iconUrl} alt={plugin.name} className="plugin-icon" />
        ) : (
          <div className="plugin-icon-placeholder">{plugin.name[0]}</div>
        )}

        {plugin.verified && (
          <div className="verified-badge" title="Verified Plugin">
            ✓
          </div>
        )}
      </div>

      <div className="plugin-card-body">
        <h3>{plugin.name}</h3>
        <p className="plugin-author">by {plugin.author.name}</p>
        <p className="plugin-description">{plugin.description}</p>

        <div className="plugin-tags">
          {plugin.categories.slice(0, 3).map((category) => (
            <span key={category} className="tag">
              {category}
            </span>
          ))}
        </div>

        <div className="plugin-stats">
          <div className="stat">
            <span className="rating">
              {'★'.repeat(Math.round(plugin.rating))}
              {'☆'.repeat(5 - Math.round(plugin.rating))}
            </span>
            <span className="rating-count">({plugin.ratingCount})</span>
          </div>
          <div className="stat">
            <span>↓ {formatDownloads(plugin.downloads)}</span>
          </div>
        </div>
      </div>

      <div className="plugin-card-footer">
        <div className="plugin-meta">
          <span>v{plugin.version}</span>
          <span>{formatSize(plugin.sizeBytes)}</span>
          <span>{plugin.license}</span>
        </div>

        <button
          className="btn-install"
          onClick={handleInstall}
          disabled={installing}
        >
          {installing ? 'Installing...' : 'Install'}
        </button>
      </div>
    </div>
  );
};

interface PaginationProps {
  currentPage: number;
  totalPages: number;
  onPageChange: (page: number) => void;
}

const Pagination: React.FC<PaginationProps> = ({
  currentPage,
  totalPages,
  onPageChange,
}) => {
  const pages = [];
  const maxVisible = 7;

  let startPage = Math.max(0, currentPage - Math.floor(maxVisible / 2));
  let endPage = Math.min(totalPages - 1, startPage + maxVisible - 1);

  if (endPage - startPage < maxVisible - 1) {
    startPage = Math.max(0, endPage - maxVisible + 1);
  }

  for (let i = startPage; i <= endPage; i++) {
    pages.push(i);
  }

  return (
    <div className="pagination">
      <button
        className="page-btn"
        disabled={currentPage === 0}
        onClick={() => onPageChange(currentPage - 1)}
      >
        Previous
      </button>

      {startPage > 0 && (
        <>
          <button className="page-btn" onClick={() => onPageChange(0)}>
            1
          </button>
          {startPage > 1 && <span className="page-ellipsis">...</span>}
        </>
      )}

      {pages.map((page) => (
        <button
          key={page}
          className={`page-btn ${page === currentPage ? 'active' : ''}`}
          onClick={() => onPageChange(page)}
        >
          {page + 1}
        </button>
      ))}

      {endPage < totalPages - 1 && (
        <>
          {endPage < totalPages - 2 && <span className="page-ellipsis">...</span>}
          <button className="page-btn" onClick={() => onPageChange(totalPages - 1)}>
            {totalPages}
          </button>
        </>
      )}

      <button
        className="page-btn"
        disabled={currentPage === totalPages - 1}
        onClick={() => onPageChange(currentPage + 1)}
      >
        Next
      </button>
    </div>
  );
};

export default PluginMarketplace;
