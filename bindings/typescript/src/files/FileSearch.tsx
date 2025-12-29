/**
 * CADDY v0.4.0 - File Search Component
 * Full-text search with advanced filters and facets
 */

import React, { useState, useCallback, useEffect, useRef } from 'react';
import {
  SearchQuery,
  SearchFilters,
  SearchResult,
  FileItem,
  FileType,
  SortField,
  SortDirection,
} from './types';

interface FileSearchProps {
  tenantId: string;
  onFileSelect?: (file: FileItem) => void;
  onFileOpen?: (file: FileItem) => void;
  initialQuery?: string;
  className?: string;
}

export const FileSearch: React.FC<FileSearchProps> = ({
  tenantId,
  onFileSelect,
  onFileOpen,
  initialQuery = '',
  className = '',
}) => {
  const [query, setQuery] = useState(initialQuery);
  const [filters, setFilters] = useState<SearchFilters>({});
  const [sortField, setSortField] = useState<SortField>('modified');
  const [sortDirection, setSortDirection] = useState<SortDirection>('desc');
  const [page, setPage] = useState(1);
  const [limit] = useState(20);
  const [result, setResult] = useState<SearchResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showFilters, setShowFilters] = useState(false);
  const searchTimeoutRef = useRef<NodeJS.Timeout | undefined>(undefined);

  // Debounced search
  useEffect(() => {
    if (searchTimeoutRef.current) {
      clearTimeout(searchTimeoutRef.current);
    }

    if (query.trim().length >= 2 || Object.keys(filters).length > 0) {
      searchTimeoutRef.current = setTimeout(() => {
        performSearch();
      }, 300);
    } else {
      setResult(null);
    }

    return () => {
      if (searchTimeoutRef.current) {
        clearTimeout(searchTimeoutRef.current);
      }
    };
  }, [query, filters, sortField, sortDirection, page]);

  const performSearch = async () => {
    setLoading(true);
    setError(null);

    try {
      const searchQuery: SearchQuery = {
        query: query.trim(),
        filters,
        sort: {
          field: sortField,
          direction: sortDirection,
        },
        pagination: {
          page,
          limit,
        },
      };

      const response = await fetch(`/api/v1/tenants/${tenantId}/files/search`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(searchQuery),
      });

      if (!response.ok) {
        throw new Error('Search failed');
      }

      const data: SearchResult = await response.json();
      setResult(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Search failed');
    } finally {
      setLoading(false);
    }
  };

  const updateFilter = useCallback(<K extends keyof SearchFilters>(
    key: K,
    value: SearchFilters[K]
  ) => {
    setFilters(prev => {
      if (value === undefined || value === null || value === '' || (Array.isArray(value) && value.length === 0)) {
        const { [key]: removed, ...rest } = prev;
        return rest;
      }
      return { ...prev, [key]: value };
    });
    setPage(1);
  }, []);

  const clearFilters = useCallback(() => {
    setFilters({});
    setPage(1);
  }, []);

  const handleSort = useCallback((field: SortField) => {
    setSortDirection(prev =>
      sortField === field && prev === 'asc' ? 'desc' : 'asc'
    );
    setSortField(field);
    setPage(1);
  }, [sortField]);

  const handlePageChange = useCallback((newPage: number) => {
    setPage(newPage);
    window.scrollTo(0, 0);
  }, []);

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
  };

  const formatDate = (date: Date): string => {
    const d = new Date(date);
    const now = new Date();
    const diffMs = now.getTime() - d.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays === 0) return 'Today';
    if (diffDays === 1) return 'Yesterday';
    if (diffDays < 7) return `${diffDays} days ago`;

    return d.toLocaleDateString();
  };

  const highlightText = (text: string, fileId: string): React.ReactNode => {
    if (!result?.highlights?.[fileId]) return text;

    const highlights = result.highlights[fileId];
    if (!highlights || highlights.length === 0) return text;

    // Simple highlighting - in production, use a proper highlighting library
    let highlightedText = text;
    highlights.forEach(highlight => {
      const regex = new RegExp(`(${highlight})`, 'gi');
      highlightedText = highlightedText.replace(
        regex,
        '<mark>$1</mark>'
      );
    });

    return <span dangerouslySetInnerHTML={{ __html: highlightedText }} />;
  };

  const activeFilterCount = Object.keys(filters).length;

  return (
    <div className={`file-search ${className}`}>
      {/* Search Bar */}
      <div className="search-header">
        <div className="search-input-container">
          <input
            type="text"
            placeholder="Search files and folders..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            className="search-input"
            autoFocus
          />
          {loading && <div className="search-spinner">⏳</div>}
        </div>
        <button
          onClick={() => setShowFilters(!showFilters)}
          className={`btn ${activeFilterCount > 0 ? 'btn-primary' : ''}`}
        >
          Filters {activeFilterCount > 0 && `(${activeFilterCount})`}
        </button>
      </div>

      {/* Filters Panel */}
      {showFilters && (
        <div className="search-filters">
          <div className="filters-grid">
            {/* File Type */}
            <div className="filter-group">
              <label>Type</label>
              <select
                value={filters.type || ''}
                onChange={(e) => updateFilter('type', e.target.value as FileType || undefined)}
                className="form-select"
              >
                <option value="">All</option>
                <option value="file">Files</option>
                <option value="folder">Folders</option>
              </select>
            </div>

            {/* MIME Types */}
            <div className="filter-group">
              <label>File Format</label>
              <select
                value={filters.mimeTypes?.[0] || ''}
                onChange={(e) =>
                  updateFilter('mimeTypes', e.target.value ? [e.target.value] : undefined)
                }
                className="form-select"
              >
                <option value="">All formats</option>
                <optgroup label="Documents">
                  <option value="application/pdf">PDF</option>
                  <option value="application/msword">Word</option>
                  <option value="application/vnd.ms-excel">Excel</option>
                  <option value="text/plain">Text</option>
                </optgroup>
                <optgroup label="Images">
                  <option value="image/jpeg">JPEG</option>
                  <option value="image/png">PNG</option>
                  <option value="image/gif">GIF</option>
                </optgroup>
                <optgroup label="Media">
                  <option value="video/mp4">Video (MP4)</option>
                  <option value="audio/mpeg">Audio (MP3)</option>
                </optgroup>
              </select>
            </div>

            {/* Size Range */}
            <div className="filter-group">
              <label>Min Size</label>
              <select
                value={filters.minSize || ''}
                onChange={(e) =>
                  updateFilter('minSize', e.target.value ? parseInt(e.target.value) : undefined)
                }
                className="form-select"
              >
                <option value="">No minimum</option>
                <option value={1024}>1 KB</option>
                <option value={1024 * 100}>100 KB</option>
                <option value={1024 * 1024}>1 MB</option>
                <option value={1024 * 1024 * 10}>10 MB</option>
                <option value={1024 * 1024 * 100}>100 MB</option>
              </select>
            </div>

            <div className="filter-group">
              <label>Max Size</label>
              <select
                value={filters.maxSize || ''}
                onChange={(e) =>
                  updateFilter('maxSize', e.target.value ? parseInt(e.target.value) : undefined)
                }
                className="form-select"
              >
                <option value="">No maximum</option>
                <option value={1024 * 1024}>1 MB</option>
                <option value={1024 * 1024 * 10}>10 MB</option>
                <option value={1024 * 1024 * 100}>100 MB</option>
                <option value={1024 * 1024 * 1024}>1 GB</option>
              </select>
            </div>

            {/* Date Range */}
            <div className="filter-group">
              <label>Modified After</label>
              <input
                type="date"
                value={filters.modifiedAfter ? new Date(filters.modifiedAfter).toISOString().split('T')[0] : ''}
                onChange={(e) =>
                  updateFilter('modifiedAfter', e.target.value ? new Date(e.target.value) : undefined)
                }
                className="form-input"
              />
            </div>

            <div className="filter-group">
              <label>Modified Before</label>
              <input
                type="date"
                value={filters.modifiedBefore ? new Date(filters.modifiedBefore).toISOString().split('T')[0] : ''}
                onChange={(e) =>
                  updateFilter('modifiedBefore', e.target.value ? new Date(e.target.value) : undefined)
                }
                className="form-input"
              />
            </div>

            {/* Starred/Trashed */}
            <div className="filter-group">
              <label>
                <input
                  type="checkbox"
                  checked={filters.isStarred || false}
                  onChange={(e) => updateFilter('isStarred', e.target.checked || undefined)}
                />
                {' '}Starred only
              </label>
            </div>

            <div className="filter-group">
              <label>
                <input
                  type="checkbox"
                  checked={filters.isTrashed || false}
                  onChange={(e) => updateFilter('isTrashed', e.target.checked || undefined)}
                />
                {' '}Include trash
              </label>
            </div>
          </div>

          <div className="filter-actions">
            <button onClick={clearFilters} className="btn">
              Clear Filters
            </button>
          </div>
        </div>
      )}

      {/* Results */}
      <div className="search-results">
        {error ? (
          <div className="error-state">{error}</div>
        ) : !result && query.trim().length < 2 && activeFilterCount === 0 ? (
          <div className="empty-state">
            Enter a search query or apply filters to find files
          </div>
        ) : !result ? (
          <div className="empty-state">Type to search...</div>
        ) : result.items.length === 0 ? (
          <div className="empty-state">
            No results found for "{query}"
            {activeFilterCount > 0 && ' with the applied filters'}
          </div>
        ) : (
          <>
            {/* Results Header */}
            <div className="results-header">
              <div className="results-info">
                Found {result.total.toLocaleString()} result{result.total !== 1 ? 's' : ''}
                {result.took && ` in ${result.took}ms`}
              </div>
              <div className="results-sort">
                <label>Sort by:</label>
                <select
                  value={sortField}
                  onChange={(e) => handleSort(e.target.value as SortField)}
                  className="form-select form-select-sm"
                >
                  <option value="name">Name</option>
                  <option value="modified">Modified</option>
                  <option value="created">Created</option>
                  <option value="size">Size</option>
                  <option value="type">Type</option>
                </select>
                <button
                  onClick={() => setSortDirection(prev => prev === 'asc' ? 'desc' : 'asc')}
                  className="btn btn-sm"
                >
                  {sortDirection === 'asc' ? '↑' : '↓'}
                </button>
              </div>
            </div>

            {/* Results List */}
            <div className="results-list">
              {result.items.map(file => (
                <div
                  key={file.id}
                  className="result-item"
                  onClick={() => onFileSelect?.(file)}
                  onDoubleClick={() => onFileOpen?.(file)}
                >
                  <div className="result-icon">
                    {file.type === 'folder' ? '📁' : '📄'}
                  </div>
                  <div className="result-info">
                    <div className="result-name">
                      {highlightText(file.name, file.id)}
                      {file.isStarred && <span className="star-icon">⭐</span>}
                    </div>
                    <div className="result-path">{file.path}</div>
                    <div className="result-meta">
                      {file.type === 'file' && (
                        <>
                          <span>{formatBytes(file.size)}</span>
                          <span>•</span>
                        </>
                      )}
                      <span>{formatDate(file.modifiedAt)}</span>
                      <span>•</span>
                      <span>Modified by {file.modifiedBy}</span>
                    </div>
                  </div>
                </div>
              ))}
            </div>

            {/* Pagination */}
            {result.totalPages > 1 && (
              <div className="results-pagination">
                <button
                  onClick={() => handlePageChange(page - 1)}
                  disabled={page === 1}
                  className="btn btn-sm"
                >
                  Previous
                </button>
                <div className="pagination-info">
                  Page {page} of {result.totalPages}
                </div>
                <button
                  onClick={() => handlePageChange(page + 1)}
                  disabled={page === result.totalPages}
                  className="btn btn-sm"
                >
                  Next
                </button>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
};

export default FileSearch;
