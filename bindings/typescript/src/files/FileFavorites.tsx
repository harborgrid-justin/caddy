/**
 * CADDY v0.4.0 - File Favorites Component
 * Starred/favorite files quick access
 */

import React, { useState, useEffect, useCallback } from 'react';
import { FileItem, ViewMode, SortField, SortDirection } from './types';

interface FileFavoritesProps {
  tenantId: string;
  onFileSelect?: (file: FileItem) => void;
  onFileOpen?: (file: FileItem) => void;
  className?: string;
}

export const FileFavorites: React.FC<FileFavoritesProps> = ({
  tenantId,
  onFileSelect,
  onFileOpen,
  className = '',
}) => {
  const [files, setFiles] = useState<FileItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<ViewMode>('list');
  const [sortField, setSortField] = useState<SortField>('name');
  const [sortDirection, setSortDirection] = useState<SortDirection>('asc');
  const [selectedFiles, setSelectedFiles] = useState<string[]>([]);

  useEffect(() => {
    loadFavorites();
  }, [tenantId, sortField, sortDirection]);

  const loadFavorites = async () => {
    setLoading(true);
    setError(null);

    try {
      const response = await fetch(
        `/api/v1/tenants/${tenantId}/files/favorites?sort=${sortField}&direction=${sortDirection}`,
        {
          headers: {
            'Authorization': `Bearer ${localStorage.getItem('token')}`,
          },
        }
      );

      if (!response.ok) {
        throw new Error('Failed to load favorites');
      }

      const data = await response.json();
      setFiles(data.files);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load favorites');
    } finally {
      setLoading(false);
    }
  };

  const toggleStar = async (fileId: string) => {
    try {
      const file = files.find(f => f.id === fileId);
      if (!file) return;

      const response = await fetch(`/api/v1/tenants/${tenantId}/files/${fileId}/star`, {
        method: file.isStarred ? 'DELETE' : 'POST',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
        },
      });

      if (!response.ok) {
        throw new Error('Failed to toggle star');
      }

      await loadFavorites();
    } catch (err) {
      console.error('Failed to toggle star:', err);
    }
  };

  const handleFileClick = useCallback(
    (file: FileItem, multi: boolean = false) => {
      if (multi) {
        setSelectedFiles(prev =>
          prev.includes(file.id)
            ? prev.filter(id => id !== file.id)
            : [...prev, file.id]
        );
      } else {
        setSelectedFiles([file.id]);
      }
      onFileSelect?.(file);
    },
    [onFileSelect]
  );

  const handleFileDoubleClick = useCallback(
    (file: FileItem) => {
      onFileOpen?.(file);
    },
    [onFileOpen]
  );

  const handleSort = useCallback((field: SortField) => {
    setSortDirection(prev =>
      sortField === field && prev === 'asc' ? 'desc' : 'asc'
    );
    setSortField(field);
  }, [sortField]);

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

  const getFileIcon = (file: FileItem): string => {
    if (file.type === 'folder') return '📁';

    const ext = file.name.split('.').pop()?.toLowerCase() || '';
    const iconMap: Record<string, string> = {
      jpg: '🖼️', jpeg: '🖼️', png: '🖼️', gif: '🖼️',
      pdf: '📕', doc: '📘', docx: '📘', xls: '📗', xlsx: '📗',
      txt: '📄', md: '📝',
      zip: '📦', rar: '📦',
      mp4: '🎬', mov: '🎬',
      mp3: '🎵', wav: '🎵',
    };

    return iconMap[ext] || '📄';
  };

  return (
    <div className={`file-favorites ${className}`}>
      {/* Header */}
      <div className="favorites-header">
        <h2>Starred Files</h2>
        <div className="header-actions">
          <button
            onClick={() => setViewMode('grid')}
            className={`btn btn-sm ${viewMode === 'grid' ? 'active' : ''}`}
          >
            Grid
          </button>
          <button
            onClick={() => setViewMode('list')}
            className={`btn btn-sm ${viewMode === 'list' ? 'active' : ''}`}
          >
            List
          </button>
        </div>
      </div>

      {/* Content */}
      <div className="favorites-content">
        {loading ? (
          <div className="loading-state">Loading favorites...</div>
        ) : error ? (
          <div className="error-state">{error}</div>
        ) : files.length === 0 ? (
          <div className="empty-state">
            <div className="empty-icon">⭐</div>
            <h3>No starred files yet</h3>
            <p>Star files to quickly access them here</p>
          </div>
        ) : viewMode === 'grid' ? (
          <div className="favorites-grid">
            {files.map(file => (
              <div
                key={file.id}
                className={`file-grid-item ${
                  selectedFiles.includes(file.id) ? 'selected' : ''
                }`}
                onClick={(e) => handleFileClick(file, e.ctrlKey || e.metaKey)}
                onDoubleClick={() => handleFileDoubleClick(file)}
              >
                <div className="file-grid-thumbnail">
                  {file.thumbnail ? (
                    <img src={file.thumbnail} alt={file.name} />
                  ) : (
                    <span className="file-grid-icon">{getFileIcon(file)}</span>
                  )}
                  <button
                    className="star-button active"
                    onClick={(e) => {
                      e.stopPropagation();
                      toggleStar(file.id);
                    }}
                  >
                    ⭐
                  </button>
                </div>
                <div className="file-grid-name" title={file.name}>
                  {file.name}
                </div>
                <div className="file-grid-meta">
                  {file.type === 'file' && formatBytes(file.size)}
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="favorites-list">
            <div className="list-header">
              <div className="list-column column-name" onClick={() => handleSort('name')}>
                Name {sortField === 'name' && (sortDirection === 'asc' ? '▲' : '▼')}
              </div>
              <div className="list-column column-size" onClick={() => handleSort('size')}>
                Size {sortField === 'size' && (sortDirection === 'asc' ? '▲' : '▼')}
              </div>
              <div className="list-column column-modified" onClick={() => handleSort('modified')}>
                Modified {sortField === 'modified' && (sortDirection === 'asc' ? '▲' : '▼')}
              </div>
              <div className="list-column column-path">Path</div>
              <div className="list-column column-actions">Actions</div>
            </div>
            <div className="list-body">
              {files.map(file => (
                <div
                  key={file.id}
                  className={`list-row ${
                    selectedFiles.includes(file.id) ? 'selected' : ''
                  }`}
                  onClick={(e) => handleFileClick(file, e.ctrlKey || e.metaKey)}
                  onDoubleClick={() => handleFileDoubleClick(file)}
                >
                  <div className="list-column column-name">
                    <span className="file-icon">{getFileIcon(file)}</span>
                    <span className="file-name">{file.name}</span>
                  </div>
                  <div className="list-column column-size">
                    {file.type === 'file' ? formatBytes(file.size) : '—'}
                  </div>
                  <div className="list-column column-modified">
                    {formatDate(file.modifiedAt)}
                  </div>
                  <div className="list-column column-path" title={file.path}>
                    {file.path}
                  </div>
                  <div className="list-column column-actions">
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        toggleStar(file.id);
                      }}
                      className="btn btn-sm"
                      title="Remove from favorites"
                    >
                      Unstar
                    </button>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default FileFavorites;
