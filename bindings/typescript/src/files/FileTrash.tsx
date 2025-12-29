/**
 * CADDY v0.4.0 - File Trash Component
 * Recycle bin with restore and permanent deletion
 */

import React, { useState, useEffect, useCallback } from 'react';
import { FileItem, ViewMode, SortField, SortDirection } from './types';

interface FileTrashProps {
  tenantId: string;
  onFileRestore?: (files: FileItem[]) => void;
  className?: string;
}

export const FileTrash: React.FC<FileTrashProps> = ({
  tenantId,
  onFileRestore,
  className = '',
}) => {
  const [files, setFiles] = useState<FileItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<ViewMode>('list');
  const [sortField, setSortField] = useState<SortField>('modified');
  const [sortDirection, setSortDirection] = useState<SortDirection>('desc');
  const [selectedFiles, setSelectedFiles] = useState<string[]>([]);
  const [storageUsed, setStorageUsed] = useState(0);
  const [retentionDays, setRetentionDays] = useState(30);

  useEffect(() => {
    loadTrash();
  }, [tenantId, sortField, sortDirection]);

  const loadTrash = async () => {
    setLoading(true);
    setError(null);

    try {
      const response = await fetch(
        `/api/v1/tenants/${tenantId}/files/trash?sort=${sortField}&direction=${sortDirection}`,
        {
          headers: {
            'Authorization': `Bearer ${localStorage.getItem('token')}`,
          },
        }
      );

      if (!response.ok) {
        throw new Error('Failed to load trash');
      }

      const data = await response.json();
      setFiles(data.files);
      setStorageUsed(data.storageUsed || 0);
      setRetentionDays(data.retentionDays || 30);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load trash');
    } finally {
      setLoading(false);
    }
  };

  const restoreFiles = async (fileIds: string[]) => {
    try {
      const response = await fetch(`/api/v1/tenants/${tenantId}/files/trash/restore`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ fileIds }),
      });

      if (!response.ok) {
        throw new Error('Failed to restore files');
      }

      const result = await response.json();
      await loadTrash();
      setSelectedFiles([]);
      onFileRestore?.(result.restored);
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to restore files');
    }
  };

  const permanentlyDelete = async (fileIds: string[]) => {
    if (!confirm(
      `Permanently delete ${fileIds.length} item${fileIds.length !== 1 ? 's' : ''}? This action cannot be undone.`
    )) {
      return;
    }

    try {
      const response = await fetch(`/api/v1/tenants/${tenantId}/files/trash/permanent`, {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ fileIds }),
      });

      if (!response.ok) {
        throw new Error('Failed to delete files');
      }

      await loadTrash();
      setSelectedFiles([]);
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to delete files');
    }
  };

  const emptyTrash = async () => {
    if (!confirm(
      'Empty trash? This will permanently delete all items in trash. This action cannot be undone.'
    )) {
      return;
    }

    try {
      const response = await fetch(`/api/v1/tenants/${tenantId}/files/trash/empty`, {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
        },
      });

      if (!response.ok) {
        throw new Error('Failed to empty trash');
      }

      await loadTrash();
      setSelectedFiles([]);
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to empty trash');
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
    },
    []
  );

  const selectAll = useCallback(() => {
    setSelectedFiles(files.map(f => f.id));
  }, [files]);

  const clearSelection = useCallback(() => {
    setSelectedFiles([]);
  }, []);

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
    return d.toLocaleString();
  };

  const getDaysUntilDeletion = (trashedAt: Date): number => {
    const trashed = new Date(trashedAt);
    const deleteDate = new Date(trashed.getTime() + retentionDays * 86400000);
    const now = new Date();
    const daysRemaining = Math.ceil((deleteDate.getTime() - now.getTime()) / 86400000);
    return Math.max(0, daysRemaining);
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
    <div className={`file-trash ${className}`}>
      {/* Header */}
      <div className="trash-header">
        <div className="header-left">
          <h2>Trash</h2>
          <div className="trash-info">
            {files.length} item{files.length !== 1 ? 's' : ''} • {formatBytes(storageUsed)} used
          </div>
        </div>
        <div className="header-right">
          {files.length > 0 && (
            <button onClick={emptyTrash} className="btn btn-danger">
              Empty Trash
            </button>
          )}
        </div>
      </div>

      {/* Toolbar */}
      {files.length > 0 && (
        <div className="trash-toolbar">
          <div className="toolbar-left">
            {selectedFiles.length > 0 ? (
              <>
                <span className="selection-count">
                  {selectedFiles.length} selected
                </span>
                <button
                  onClick={() => restoreFiles(selectedFiles)}
                  className="btn btn-primary"
                >
                  Restore
                </button>
                <button
                  onClick={() => permanentlyDelete(selectedFiles)}
                  className="btn btn-danger"
                >
                  Delete Forever
                </button>
                <button onClick={clearSelection} className="btn">
                  Clear Selection
                </button>
              </>
            ) : (
              <button onClick={selectAll} className="btn">
                Select All
              </button>
            )}
          </div>
          <div className="toolbar-right">
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
      )}

      {/* Info Banner */}
      {files.length > 0 && (
        <div className="trash-banner">
          <div className="banner-icon">ℹ️</div>
          <div className="banner-text">
            Items in trash will be permanently deleted after {retentionDays} days
          </div>
        </div>
      )}

      {/* Content */}
      <div className="trash-content">
        {loading ? (
          <div className="loading-state">Loading trash...</div>
        ) : error ? (
          <div className="error-state">{error}</div>
        ) : files.length === 0 ? (
          <div className="empty-state">
            <div className="empty-icon">🗑️</div>
            <h3>Trash is empty</h3>
            <p>Deleted files will appear here</p>
          </div>
        ) : viewMode === 'grid' ? (
          <div className="trash-grid">
            {files.map(file => {
              const daysLeft = file.trashedAt ? getDaysUntilDeletion(file.trashedAt) : 0;

              return (
                <div
                  key={file.id}
                  className={`file-grid-item ${
                    selectedFiles.includes(file.id) ? 'selected' : ''
                  } ${daysLeft <= 7 ? 'expiring-soon' : ''}`}
                  onClick={(e) => handleFileClick(file, e.ctrlKey || e.metaKey)}
                >
                  <div className="file-grid-thumbnail">
                    {file.thumbnail ? (
                      <img src={file.thumbnail} alt={file.name} />
                    ) : (
                      <span className="file-grid-icon">{getFileIcon(file)}</span>
                    )}
                    {daysLeft <= 7 && (
                      <div className="expiry-badge" title={`${daysLeft} days remaining`}>
                        {daysLeft}d
                      </div>
                    )}
                  </div>
                  <div className="file-grid-name" title={file.name}>
                    {file.name}
                  </div>
                  <div className="file-grid-meta">
                    Deleted {file.trashedAt && formatDate(file.trashedAt).split(',')[0]}
                  </div>
                  <div className="file-grid-actions">
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        restoreFiles([file.id]);
                      }}
                      className="btn btn-sm btn-primary"
                    >
                      Restore
                    </button>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        permanentlyDelete([file.id]);
                      }}
                      className="btn btn-sm btn-danger"
                    >
                      Delete
                    </button>
                  </div>
                </div>
              );
            })}
          </div>
        ) : (
          <div className="trash-list">
            <div className="list-header">
              <div className="list-column column-checkbox">
                <input
                  type="checkbox"
                  checked={selectedFiles.length === files.length}
                  onChange={(e) => e.target.checked ? selectAll() : clearSelection()}
                />
              </div>
              <div className="list-column column-name" onClick={() => handleSort('name')}>
                Name {sortField === 'name' && (sortDirection === 'asc' ? '▲' : '▼')}
              </div>
              <div className="list-column column-path">Original Location</div>
              <div className="list-column column-size" onClick={() => handleSort('size')}>
                Size {sortField === 'size' && (sortDirection === 'asc' ? '▲' : '▼')}
              </div>
              <div className="list-column column-deleted">Deleted</div>
              <div className="list-column column-expires">Expires In</div>
              <div className="list-column column-actions">Actions</div>
            </div>
            <div className="list-body">
              {files.map(file => {
                const daysLeft = file.trashedAt ? getDaysUntilDeletion(file.trashedAt) : 0;

                return (
                  <div
                    key={file.id}
                    className={`list-row ${
                      selectedFiles.includes(file.id) ? 'selected' : ''
                    } ${daysLeft <= 7 ? 'expiring-soon' : ''}`}
                    onClick={(e) => handleFileClick(file, e.ctrlKey || e.metaKey)}
                  >
                    <div className="list-column column-checkbox">
                      <input
                        type="checkbox"
                        checked={selectedFiles.includes(file.id)}
                        onChange={(e) => {
                          e.stopPropagation();
                          handleFileClick(file, true);
                        }}
                      />
                    </div>
                    <div className="list-column column-name">
                      <span className="file-icon">{getFileIcon(file)}</span>
                      <span className="file-name">{file.name}</span>
                    </div>
                    <div className="list-column column-path" title={file.path}>
                      {file.path}
                    </div>
                    <div className="list-column column-size">
                      {file.type === 'file' ? formatBytes(file.size) : '—'}
                    </div>
                    <div className="list-column column-deleted">
                      {file.trashedAt && formatDate(file.trashedAt)}
                    </div>
                    <div className="list-column column-expires">
                      <span className={daysLeft <= 7 ? 'text-danger' : ''}>
                        {daysLeft} day{daysLeft !== 1 ? 's' : ''}
                      </span>
                    </div>
                    <div className="list-column column-actions">
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          restoreFiles([file.id]);
                        }}
                        className="btn btn-sm btn-primary"
                      >
                        Restore
                      </button>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          permanentlyDelete([file.id]);
                        }}
                        className="btn btn-sm btn-danger"
                      >
                        Delete
                      </button>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default FileTrash;
