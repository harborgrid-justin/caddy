/**
 * CADDY v0.4.0 - File Storage Component
 * Storage quota, usage tracking, and analytics
 */

import React, { useState, useEffect, useCallback } from 'react';
import { StorageQuota, StorageBreakdown } from './types';

interface FileStorageProps {
  tenantId: string;
  onUpgrade?: () => void;
  className?: string;
}

export const FileStorage: React.FC<FileStorageProps> = ({
  tenantId,
  onUpgrade,
  className = '',
}) => {
  const [quota, setQuota] = useState<StorageQuota | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<'overview' | 'breakdown' | 'limits'>('overview');

  useEffect(() => {
    loadQuota();
  }, [tenantId]);

  const loadQuota = async () => {
    setLoading(true);
    setError(null);

    try {
      const response = await fetch(`/api/v1/tenants/${tenantId}/files/storage/quota`, {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
        },
      });

      if (!response.ok) {
        throw new Error('Failed to load storage quota');
      }

      const data = await response.json();
      setQuota(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load storage quota');
    } finally {
      setLoading(false);
    }
  };

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  };

  const getStorageColor = (percentage: number): string => {
    if (percentage >= 90) return '#ef4444';
    if (percentage >= 75) return '#f59e0b';
    if (percentage >= 50) return '#3b82f6';
    return '#10b981';
  };

  const calculatePercentage = (value: number, total: number): number => {
    return total > 0 ? (value / total) * 100 : 0;
  };

  if (loading) {
    return (
      <div className={`file-storage ${className}`}>
        <div className="loading-state">Loading storage information...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={`file-storage ${className}`}>
        <div className="error-state">{error}</div>
      </div>
    );
  }

  if (!quota) {
    return null;
  }

  return (
    <div className={`file-storage ${className}`}>
      {/* Header */}
      <div className="storage-header">
        <h2>Storage</h2>
        <div className="storage-tabs">
          <button
            className={`tab ${viewMode === 'overview' ? 'active' : ''}`}
            onClick={() => setViewMode('overview')}
          >
            Overview
          </button>
          <button
            className={`tab ${viewMode === 'breakdown' ? 'active' : ''}`}
            onClick={() => setViewMode('breakdown')}
          >
            Breakdown
          </button>
          <button
            className={`tab ${viewMode === 'limits' ? 'active' : ''}`}
            onClick={() => setViewMode('limits')}
          >
            Limits
          </button>
        </div>
      </div>

      {/* Overview */}
      {viewMode === 'overview' && (
        <div className="storage-overview">
          {/* Usage Summary */}
          <div className="usage-summary">
            <div className="usage-circle">
              <svg viewBox="0 0 200 200" className="circle-chart">
                <circle
                  cx="100"
                  cy="100"
                  r="80"
                  fill="none"
                  stroke="#e5e7eb"
                  strokeWidth="20"
                />
                <circle
                  cx="100"
                  cy="100"
                  r="80"
                  fill="none"
                  stroke={getStorageColor(quota.percentage)}
                  strokeWidth="20"
                  strokeDasharray={`${quota.percentage * 5.03} 503`}
                  strokeLinecap="round"
                  transform="rotate(-90 100 100)"
                />
                <text
                  x="100"
                  y="100"
                  textAnchor="middle"
                  dy="0.3em"
                  className="percentage-text"
                >
                  {Math.round(quota.percentage)}%
                </text>
              </svg>
            </div>
            <div className="usage-details">
              <div className="usage-label">Storage Used</div>
              <div className="usage-value">
                {formatBytes(quota.used)} of {formatBytes(quota.total)}
              </div>
              <div className="usage-available">
                {formatBytes(quota.available)} available
              </div>
            </div>
          </div>

          {/* Quick Stats */}
          <div className="quick-stats">
            <div className="stat-card">
              <div className="stat-icon">📄</div>
              <div className="stat-label">Documents</div>
              <div className="stat-value">{formatBytes(quota.breakdown.documents)}</div>
            </div>
            <div className="stat-card">
              <div className="stat-icon">🖼️</div>
              <div className="stat-label">Images</div>
              <div className="stat-value">{formatBytes(quota.breakdown.images)}</div>
            </div>
            <div className="stat-card">
              <div className="stat-icon">🎬</div>
              <div className="stat-label">Videos</div>
              <div className="stat-value">{formatBytes(quota.breakdown.videos)}</div>
            </div>
            <div className="stat-card">
              <div className="stat-icon">🗑️</div>
              <div className="stat-label">Trash</div>
              <div className="stat-value">{formatBytes(quota.breakdown.trash)}</div>
            </div>
          </div>

          {/* Warnings */}
          {quota.percentage >= 90 && (
            <div className="storage-warning storage-critical">
              <strong>Storage Almost Full!</strong>
              <p>
                You're using {quota.percentage.toFixed(1)}% of your storage.
                {onUpgrade && ' Consider upgrading your plan for more storage.'}
              </p>
              {onUpgrade && (
                <button onClick={onUpgrade} className="btn btn-primary">
                  Upgrade Plan
                </button>
              )}
            </div>
          )}
          {quota.percentage >= 75 && quota.percentage < 90 && (
            <div className="storage-warning storage-warning-level">
              <strong>Storage Running Low</strong>
              <p>
                You're using {quota.percentage.toFixed(1)}% of your storage.
                Consider cleaning up old files or upgrading your plan.
              </p>
            </div>
          )}

          {/* Plan Info */}
          <div className="plan-info">
            <div className="plan-badge">{quota.plan}</div>
            <div className="plan-details">
              You're on the <strong>{quota.plan}</strong> plan
            </div>
            {onUpgrade && (
              <button onClick={onUpgrade} className="btn btn-sm">
                Change Plan
              </button>
            )}
          </div>
        </div>
      )}

      {/* Breakdown */}
      {viewMode === 'breakdown' && (
        <div className="storage-breakdown">
          <div className="breakdown-chart">
            <div className="breakdown-bar">
              {Object.entries(quota.breakdown).map(([type, size]) => {
                const percentage = calculatePercentage(size, quota.used);
                if (percentage < 0.1) return null;

                const colors: Record<string, string> = {
                  documents: '#3b82f6',
                  images: '#8b5cf6',
                  videos: '#ec4899',
                  audio: '#10b981',
                  archives: '#f59e0b',
                  other: '#6b7280',
                  trash: '#ef4444',
                };

                return (
                  <div
                    key={type}
                    className="breakdown-segment"
                    style={{
                      width: `${percentage}%`,
                      backgroundColor: colors[type] || '#6b7280',
                    }}
                    title={`${type}: ${formatBytes(size)} (${percentage.toFixed(1)}%)`}
                  />
                );
              })}
            </div>
          </div>

          <div className="breakdown-list">
            {Object.entries(quota.breakdown)
              .sort(([, a], [, b]) => b - a)
              .map(([type, size]) => {
                const percentage = calculatePercentage(size, quota.used);
                const icons: Record<string, string> = {
                  documents: '📄',
                  images: '🖼️',
                  videos: '🎬',
                  audio: '🎵',
                  archives: '📦',
                  other: '📁',
                  trash: '🗑️',
                };

                return (
                  <div key={type} className="breakdown-item">
                    <div className="breakdown-type">
                      <span className="type-icon">{icons[type] || '📁'}</span>
                      <span className="type-name">
                        {type.charAt(0).toUpperCase() + type.slice(1)}
                      </span>
                    </div>
                    <div className="breakdown-size">{formatBytes(size)}</div>
                    <div className="breakdown-percentage">
                      {percentage.toFixed(1)}%
                    </div>
                  </div>
                );
              })}
          </div>
        </div>
      )}

      {/* Limits */}
      {viewMode === 'limits' && (
        <div className="storage-limits">
          <div className="limits-list">
            <div className="limit-item">
              <div className="limit-label">
                <strong>Maximum File Size</strong>
                <p>The largest file you can upload</p>
              </div>
              <div className="limit-value">{formatBytes(quota.limits.maxFileSize)}</div>
            </div>

            <div className="limit-item">
              <div className="limit-label">
                <strong>Total Storage</strong>
                <p>Total storage space available</p>
              </div>
              <div className="limit-value">{formatBytes(quota.limits.maxTotalStorage)}</div>
            </div>

            <div className="limit-item">
              <div className="limit-label">
                <strong>Files Per Folder</strong>
                <p>Maximum number of items in a single folder</p>
              </div>
              <div className="limit-value">
                {quota.limits.maxFilesPerFolder.toLocaleString()}
              </div>
            </div>

            <div className="limit-item">
              <div className="limit-label">
                <strong>Version History</strong>
                <p>Maximum versions kept per file</p>
              </div>
              <div className="limit-value">
                {quota.limits.maxVersionsPerFile}
              </div>
            </div>

            <div className="limit-item">
              <div className="limit-label">
                <strong>Trash Retention</strong>
                <p>Files in trash are kept for</p>
              </div>
              <div className="limit-value">
                {quota.limits.retentionDays} days
              </div>
            </div>

            {quota.limits.allowedFileTypes.length > 0 && (
              <div className="limit-item">
                <div className="limit-label">
                  <strong>Allowed File Types</strong>
                  <p>File types you can upload</p>
                </div>
                <div className="limit-value">
                  <div className="file-types">
                    {quota.limits.allowedFileTypes.slice(0, 10).map(type => (
                      <span key={type} className="file-type-badge">
                        {type}
                      </span>
                    ))}
                    {quota.limits.allowedFileTypes.length > 10 && (
                      <span className="file-type-badge">
                        +{quota.limits.allowedFileTypes.length - 10} more
                      </span>
                    )}
                  </div>
                </div>
              </div>
            )}
          </div>

          {onUpgrade && (
            <div className="limits-upgrade">
              <h3>Need Higher Limits?</h3>
              <p>
                Upgrade your plan to get higher storage limits, larger file sizes,
                and more features.
              </p>
              <button onClick={onUpgrade} className="btn btn-primary">
                View Plans
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  );
};

export default FileStorage;
