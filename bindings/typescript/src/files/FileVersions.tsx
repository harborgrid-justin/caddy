/**
 * CADDY v0.4.0 - File Versions Component
 * Version history, comparison, and restoration
 */

import React, { useState, useEffect, useCallback } from 'react';
import { FileItem, FileVersion } from './types';

interface FileVersionsProps {
  file: FileItem;
  tenantId: string;
  onClose: () => void;
  onVersionRestore?: (version: FileVersion) => void;
  className?: string;
}

export const FileVersions: React.FC<FileVersionsProps> = ({
  file,
  tenantId,
  onClose,
  onVersionRestore,
  className = '',
}) => {
  const [versions, setVersions] = useState<FileVersion[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedVersions, setSelectedVersions] = useState<[string, string] | null>(null);
  const [comparing, setComparing] = useState(false);
  const [comparisonData, setComparisonData] = useState<any>(null);

  useEffect(() => {
    loadVersions();
  }, [file.id]);

  const loadVersions = async () => {
    setLoading(true);
    setError(null);

    try {
      const response = await fetch(`/api/v1/tenants/${tenantId}/files/${file.id}/versions`, {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
        },
      });

      if (!response.ok) {
        throw new Error('Failed to load versions');
      }

      const data = await response.json();
      setVersions(data.versions);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load versions');
    } finally {
      setLoading(false);
    }
  };

  const restoreVersion = async (version: FileVersion) => {
    if (!confirm(`Restore version ${version.version}? This will create a new version with the content from version ${version.version}.`)) {
      return;
    }

    try {
      const response = await fetch(
        `/api/v1/tenants/${tenantId}/files/${file.id}/versions/${version.id}/restore`,
        {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${localStorage.getItem('token')}`,
          },
        }
      );

      if (!response.ok) {
        throw new Error('Failed to restore version');
      }

      await loadVersions();
      onVersionRestore?.(version);
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to restore version');
    }
  };

  const downloadVersion = (version: FileVersion) => {
    const token = localStorage.getItem('token');
    window.open(`${version.url}?token=${token}`, '_blank');
  };

  const deleteVersion = async (version: FileVersion) => {
    if (version.isCurrent) {
      alert('Cannot delete the current version');
      return;
    }

    if (!confirm(`Delete version ${version.version}? This action cannot be undone.`)) {
      return;
    }

    try {
      const response = await fetch(
        `/api/v1/tenants/${tenantId}/files/${file.id}/versions/${version.id}`,
        {
          method: 'DELETE',
          headers: {
            'Authorization': `Bearer ${localStorage.getItem('token')}`,
          },
        }
      );

      if (!response.ok) {
        throw new Error('Failed to delete version');
      }

      await loadVersions();
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to delete version');
    }
  };

  const compareVersions = async () => {
    if (!selectedVersions || selectedVersions.length !== 2) return;

    setComparing(true);
    try {
      const response = await fetch(
        `/api/v1/tenants/${tenantId}/files/${file.id}/versions/compare`,
        {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${localStorage.getItem('token')}`,
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            versionId1: selectedVersions[0],
            versionId2: selectedVersions[1],
          }),
        }
      );

      if (!response.ok) {
        throw new Error('Failed to compare versions');
      }

      const data = await response.json();
      setComparisonData(data);
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to compare versions');
    } finally {
      setComparing(false);
    }
  };

  const toggleVersionSelection = (versionId: string) => {
    if (!selectedVersions) {
      setSelectedVersions([versionId, '']);
    } else if (selectedVersions[0] === versionId) {
      setSelectedVersions(null);
    } else if (selectedVersions[1] === versionId) {
      setSelectedVersions([selectedVersions[0], '']);
    } else if (!selectedVersions[1]) {
      setSelectedVersions([selectedVersions[0], versionId]);
    } else {
      setSelectedVersions([versionId, '']);
    }
  };

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

  const getSizeChange = (current: number, previous: number): string => {
    const diff = current - previous;
    if (diff === 0) return 'No change';
    const sign = diff > 0 ? '+' : '';
    return `${sign}${formatBytes(Math.abs(diff))}`;
  };

  return (
    <div className={`file-versions-modal ${className}`}>
      <div className="versions-overlay" onClick={onClose}></div>
      <div className="versions-container">
        {/* Header */}
        <div className="versions-header">
          <h2>Version History - {file.name}</h2>
          <button onClick={onClose} className="btn-close">
            ✕
          </button>
        </div>

        {/* Toolbar */}
        {selectedVersions && selectedVersions[0] && selectedVersions[1] && (
          <div className="versions-toolbar">
            <button onClick={compareVersions} disabled={comparing} className="btn btn-primary">
              {comparing ? 'Comparing...' : 'Compare Selected Versions'}
            </button>
            <button onClick={() => setSelectedVersions(null)} className="btn">
              Clear Selection
            </button>
          </div>
        )}

        {/* Content */}
        <div className="versions-content">
          {loading ? (
            <div className="loading-state">Loading versions...</div>
          ) : error ? (
            <div className="error-state">{error}</div>
          ) : versions.length === 0 ? (
            <div className="empty-state">No version history available</div>
          ) : (
            <div className="versions-list">
              {versions.map((version, index) => {
                const previousVersion = versions[index + 1];
                const isSelected =
                  selectedVersions?.includes(version.id) || false;

                return (
                  <div
                    key={version.id}
                    className={`version-item ${version.isCurrent ? 'current' : ''} ${
                      isSelected ? 'selected' : ''
                    }`}
                  >
                    <div className="version-selector">
                      <input
                        type="checkbox"
                        checked={isSelected}
                        onChange={() => toggleVersionSelection(version.id)}
                      />
                    </div>

                    <div className="version-info">
                      <div className="version-header">
                        <div className="version-number">
                          Version {version.version}
                          {version.isCurrent && (
                            <span className="current-badge">Current</span>
                          )}
                        </div>
                        <div className="version-date">
                          {formatDate(version.createdAt)}
                        </div>
                      </div>

                      <div className="version-meta">
                        <div className="meta-item">
                          <strong>Size:</strong> {formatBytes(version.size)}
                          {previousVersion && (
                            <span className="size-change">
                              {' '}
                              ({getSizeChange(version.size, previousVersion.size)})
                            </span>
                          )}
                        </div>
                        <div className="meta-item">
                          <strong>Created by:</strong> {version.createdBy}
                        </div>
                        {version.checksum && (
                          <div className="meta-item">
                            <strong>Checksum:</strong>{' '}
                            <code className="checksum">{version.checksum.slice(0, 16)}...</code>
                          </div>
                        )}
                      </div>

                      {version.comment && (
                        <div className="version-comment">
                          <strong>Comment:</strong> {version.comment}
                        </div>
                      )}
                    </div>

                    <div className="version-actions">
                      <button
                        onClick={() => downloadVersion(version)}
                        className="btn btn-sm"
                        title="Download this version"
                      >
                        Download
                      </button>
                      {!version.isCurrent && (
                        <>
                          <button
                            onClick={() => restoreVersion(version)}
                            className="btn btn-sm btn-primary"
                            title="Restore this version"
                          >
                            Restore
                          </button>
                          <button
                            onClick={() => deleteVersion(version)}
                            className="btn btn-sm btn-danger"
                            title="Delete this version"
                          >
                            Delete
                          </button>
                        </>
                      )}
                    </div>
                  </div>
                );
              })}
            </div>
          )}

          {/* Comparison View */}
          {comparisonData && (
            <div className="comparison-panel">
              <div className="comparison-header">
                <h3>Version Comparison</h3>
                <button
                  onClick={() => setComparisonData(null)}
                  className="btn-close"
                >
                  ✕
                </button>
              </div>
              <div className="comparison-content">
                <div className="comparison-stats">
                  <div className="stat">
                    <strong>Size Difference:</strong>{' '}
                    {formatBytes(Math.abs(comparisonData.sizeDiff))}
                  </div>
                  <div className="stat">
                    <strong>Changes:</strong> {comparisonData.changes || 'N/A'}
                  </div>
                </div>
                {comparisonData.diff && (
                  <div className="comparison-diff">
                    <pre>{comparisonData.diff}</pre>
                  </div>
                )}
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="versions-footer">
          <div className="versions-summary">
            Total versions: {versions.length} • Current version: {file.version}
          </div>
        </div>
      </div>
    </div>
  );
};

export default FileVersions;
