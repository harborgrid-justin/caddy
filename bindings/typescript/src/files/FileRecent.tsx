/**
 * CADDY v0.4.0 - Recent Files Component
 * Recently accessed files with access history
 */

import React, { useState, useEffect, useCallback } from 'react';
import { RecentFile, FileItem, ViewMode } from './types';

interface FileRecentProps {
  tenantId: string;
  onFileSelect?: (file: FileItem) => void;
  onFileOpen?: (file: FileItem) => void;
  limit?: number;
  className?: string;
}

export const FileRecent: React.FC<FileRecentProps> = ({
  tenantId,
  onFileSelect,
  onFileOpen,
  limit = 50,
  className = '',
}) => {
  const [recentFiles, setRecentFiles] = useState<RecentFile[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<ViewMode>('list');
  const [filterType, setFilterType] = useState<'all' | 'view' | 'edit' | 'download'>('all');
  const [selectedFiles, setSelectedFiles] = useState<string[]>([]);

  useEffect(() => {
    loadRecentFiles();
  }, [tenantId, limit, filterType]);

  const loadRecentFiles = async () => {
    setLoading(true);
    setError(null);

    try {
      const response = await fetch(
        `/api/v1/tenants/${tenantId}/files/recent?limit=${limit}${
          filterType !== 'all' ? `&type=${filterType}` : ''
        }`,
        {
          headers: {
            'Authorization': `Bearer ${localStorage.getItem('token')}`,
          },
        }
      );

      if (!response.ok) {
        throw new Error('Failed to load recent files');
      }

      const data = await response.json();
      setRecentFiles(data.recent);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load recent files');
    } finally {
      setLoading(false);
    }
  };

  const clearHistory = async () => {
    if (!confirm('Clear all recent files history?')) {
      return;
    }

    try {
      const response = await fetch(`/api/v1/tenants/${tenantId}/files/recent`, {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
        },
      });

      if (!response.ok) {
        throw new Error('Failed to clear history');
      }

      await loadRecentFiles();
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to clear history');
    }
  };

  const removeFromRecent = async (fileId: string) => {
    try {
      const response = await fetch(
        `/api/v1/tenants/${tenantId}/files/${fileId}/recent`,
        {
          method: 'DELETE',
          headers: {
            'Authorization': `Bearer ${localStorage.getItem('token')}`,
          },
        }
      );

      if (!response.ok) {
        throw new Error('Failed to remove from recent');
      }

      await loadRecentFiles();
    } catch (err) {
      console.error('Failed to remove from recent:', err);
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
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;

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

  const getAccessTypeIcon = (type: RecentFile['accessType']): string => {
    const icons = {
      view: '👁️',
      edit: '✏️',
      download: '⬇️',
      share: '🔗',
    };
    return icons[type];
  };

  const getAccessTypeLabel = (type: RecentFile['accessType']): string => {
    const labels = {
      view: 'Viewed',
      edit: 'Edited',
      download: 'Downloaded',
      share: 'Shared',
    };
    return labels[type];
  };

  // Group files by date
  const groupedFiles = recentFiles.reduce((groups, recent) => {
    const date = new Date(recent.accessedAt);
    const today = new Date();
    const yesterday = new Date(today);
    yesterday.setDate(yesterday.getDate() - 1);

    let groupKey: string;
    if (date.toDateString() === today.toDateString()) {
      groupKey = 'Today';
    } else if (date.toDateString() === yesterday.toDateString()) {
      groupKey = 'Yesterday';
    } else if (date > new Date(today.getTime() - 7 * 86400000)) {
      groupKey = 'This Week';
    } else if (date > new Date(today.getTime() - 30 * 86400000)) {
      groupKey = 'This Month';
    } else {
      groupKey = 'Older';
    }

    if (!groups[groupKey]) {
      groups[groupKey] = [];
    }
    groups[groupKey].push(recent);
    return groups;
  }, {} as Record<string, RecentFile[]>);

  const groupOrder = ['Today', 'Yesterday', 'This Week', 'This Month', 'Older'];

  return (
    <div className={`file-recent ${className}`}>
      {/* Header */}
      <div className="recent-header">
        <h2>Recent Files</h2>
        <div className="header-actions">
          <select
            value={filterType}
            onChange={(e) => setFilterType(e.target.value as any)}
            className="form-select form-select-sm"
          >
            <option value="all">All Activity</option>
            <option value="view">Viewed</option>
            <option value="edit">Edited</option>
            <option value="download">Downloaded</option>
          </select>
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
          {recentFiles.length > 0 && (
            <button onClick={clearHistory} className="btn btn-sm btn-danger">
              Clear History
            </button>
          )}
        </div>
      </div>

      {/* Content */}
      <div className="recent-content">
        {loading ? (
          <div className="loading-state">Loading recent files...</div>
        ) : error ? (
          <div className="error-state">{error}</div>
        ) : recentFiles.length === 0 ? (
          <div className="empty-state">
            <div className="empty-icon">🕒</div>
            <h3>No recent files</h3>
            <p>Files you access will appear here</p>
          </div>
        ) : viewMode === 'grid' ? (
          <div className="recent-groups">
            {groupOrder.map(groupKey => {
              const group = groupedFiles[groupKey];
              if (!group || group.length === 0) return null;

              return (
                <div key={groupKey} className="recent-group">
                  <h3 className="group-title">{groupKey}</h3>
                  <div className="recent-grid">
                    {group.map(recent => (
                      <div
                        key={recent.file.id}
                        className={`file-grid-item ${
                          selectedFiles.includes(recent.file.id) ? 'selected' : ''
                        }`}
                        onClick={(e) =>
                          handleFileClick(recent.file, e.ctrlKey || e.metaKey)
                        }
                        onDoubleClick={() => handleFileDoubleClick(recent.file)}
                      >
                        <div className="file-grid-thumbnail">
                          {recent.file.thumbnail ? (
                            <img src={recent.file.thumbnail} alt={recent.file.name} />
                          ) : (
                            <span className="file-grid-icon">
                              {getFileIcon(recent.file)}
                            </span>
                          )}
                          <div className="access-badge" title={getAccessTypeLabel(recent.accessType)}>
                            {getAccessTypeIcon(recent.accessType)}
                          </div>
                        </div>
                        <div className="file-grid-name" title={recent.file.name}>
                          {recent.file.name}
                        </div>
                        <div className="file-grid-meta">
                          {formatDate(recent.accessedAt)}
                        </div>
                        <button
                          className="remove-button"
                          onClick={(e) => {
                            e.stopPropagation();
                            removeFromRecent(recent.file.id);
                          }}
                          title="Remove from recent"
                        >
                          ✕
                        </button>
                      </div>
                    ))}
                  </div>
                </div>
              );
            })}
          </div>
        ) : (
          <div className="recent-groups">
            {groupOrder.map(groupKey => {
              const group = groupedFiles[groupKey];
              if (!group || group.length === 0) return null;

              return (
                <div key={groupKey} className="recent-group">
                  <h3 className="group-title">{groupKey}</h3>
                  <div className="recent-list">
                    <div className="list-header">
                      <div className="list-column column-name">Name</div>
                      <div className="list-column column-activity">Activity</div>
                      <div className="list-column column-size">Size</div>
                      <div className="list-column column-accessed">Accessed</div>
                      <div className="list-column column-count">Times</div>
                      <div className="list-column column-actions">Actions</div>
                    </div>
                    <div className="list-body">
                      {group.map(recent => (
                        <div
                          key={recent.file.id}
                          className={`list-row ${
                            selectedFiles.includes(recent.file.id) ? 'selected' : ''
                          }`}
                          onClick={(e) =>
                            handleFileClick(recent.file, e.ctrlKey || e.metaKey)
                          }
                          onDoubleClick={() => handleFileDoubleClick(recent.file)}
                        >
                          <div className="list-column column-name">
                            <span className="file-icon">{getFileIcon(recent.file)}</span>
                            <span className="file-name">{recent.file.name}</span>
                            {recent.file.isStarred && <span className="star-icon">⭐</span>}
                          </div>
                          <div className="list-column column-activity">
                            <span
                              className="access-badge"
                              title={getAccessTypeLabel(recent.accessType)}
                            >
                              {getAccessTypeIcon(recent.accessType)}
                              {' '}
                              {getAccessTypeLabel(recent.accessType)}
                            </span>
                          </div>
                          <div className="list-column column-size">
                            {recent.file.type === 'file'
                              ? formatBytes(recent.file.size)
                              : '—'}
                          </div>
                          <div className="list-column column-accessed">
                            {formatDate(recent.accessedAt)}
                          </div>
                          <div className="list-column column-count">
                            {recent.accessCount}x
                          </div>
                          <div className="list-column column-actions">
                            <button
                              onClick={(e) => {
                                e.stopPropagation();
                                removeFromRecent(recent.file.id);
                              }}
                              className="btn btn-sm"
                            >
                              Remove
                            </button>
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
};

export default FileRecent;
