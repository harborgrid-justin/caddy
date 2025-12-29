/**
 * CompressionStats - Real-time compression statistics display
 *
 * React component for visualizing compression performance metrics
 */

import React, { useState, useEffect } from 'react';
import {
  CompressionStats as ICompressionStats,
  CompressionPerformanceMetrics,
  CompressionAlgorithm,
  calculateCompressionPercentage,
  calculateThroughput,
  formatFileSize,
  formatDuration,
} from './types';
import { compressionService } from './CompressionService';

export interface CompressionStatsProps {
  /** Statistics to display (if null, shows overall metrics) */
  stats?: ICompressionStats | null;
  /** Show detailed metrics */
  showDetailed?: boolean;
  /** Auto-refresh interval in ms (0 to disable) */
  refreshInterval?: number;
  /** Custom CSS class */
  className?: string;
}

/**
 * CompressionStats component
 */
export const CompressionStats: React.FC<CompressionStatsProps> = ({
  stats = null,
  showDetailed = true,
  refreshInterval = 0,
  className = '',
}) => {
  const [metrics, setMetrics] = useState<CompressionPerformanceMetrics>(
    compressionService.getMetrics()
  );

  // Auto-refresh metrics
  useEffect(() => {
    if (refreshInterval > 0) {
      const interval = setInterval(() => {
        setMetrics(compressionService.getMetrics());
      }, refreshInterval);

      return () => clearInterval(interval);
    }
    return undefined;
  }, [refreshInterval]);

  const handleRefresh = () => {
    setMetrics(compressionService.getMetrics());
  };

  const handleReset = () => {
    compressionService.resetMetrics();
    setMetrics(compressionService.getMetrics());
  };

  // Show single operation stats
  if (stats) {
    return <SingleOperationStats stats={stats} className={className} />;
  }

  // Show overall performance metrics
  return (
    <div className={`compression-stats ${className}`}>
      <div className="stats-header">
        <h3 className="stats-title">Compression Performance</h3>
        <div className="stats-actions">
          <button onClick={handleRefresh} className="stats-button" title="Refresh">
            ↻
          </button>
          <button onClick={handleReset} className="stats-button" title="Reset">
            ✕
          </button>
        </div>
      </div>

      {/* Summary Cards */}
      <div className="stats-grid">
        <StatCard
          label="Files Compressed"
          value={metrics.totalFilesCompressed.toLocaleString()}
          icon="📦"
        />
        <StatCard
          label="Files Decompressed"
          value={metrics.totalFilesDecompressed.toLocaleString()}
          icon="📂"
        />
        <StatCard
          label="Total Compressed"
          value={formatFileSize(metrics.totalBytesCompressed)}
          icon="💾"
        />
        <StatCard
          label="Average Ratio"
          value={`${((1 - metrics.averageCompressionRatio) * 100).toFixed(1)}%`}
          icon="📊"
        />
      </div>

      {/* Detailed Stats */}
      {showDetailed && (
        <>
          {/* Performance Metrics */}
          <div className="stats-section">
            <h4 className="section-title">Performance Metrics</h4>
            <div className="metrics-list">
              <MetricRow
                label="Average Compression Speed"
                value={`${metrics.averageCompressionSpeed.toFixed(2)} MB/s`}
              />
              <MetricRow
                label="Average Decompression Speed"
                value={`${metrics.averageDecompressionSpeed.toFixed(2)} MB/s`}
              />
              <MetricRow
                label="Total Data Processed"
                value={formatFileSize(
                  metrics.totalBytesCompressed + metrics.totalBytesDecompressed
                )}
              />
            </div>
          </div>

          {/* Algorithm Usage */}
          {Object.keys(metrics.algorithmUsage).length > 0 && (
            <div className="stats-section">
              <h4 className="section-title">Algorithm Usage</h4>
              <div className="algorithm-chart">
                {Object.entries(metrics.algorithmUsage)
                  .sort((a, b) => b[1] - a[1])
                  .map(([algorithm, count]) => (
                    <AlgorithmBar
                      key={algorithm}
                      algorithm={algorithm as CompressionAlgorithm}
                      count={count}
                      total={metrics.totalFilesCompressed}
                    />
                  ))}
              </div>
            </div>
          )}
        </>
      )}

      <style>{compressionStatsStyles}</style>
    </div>
  );
};

/**
 * Single operation statistics display
 */
const SingleOperationStats: React.FC<{
  stats: ICompressionStats;
  className?: string;
}> = ({ stats, className = '' }) => {
  const compressionPercentage = calculateCompressionPercentage(stats);
  const throughput = calculateThroughput(stats);

  return (
    <div className={`single-stats ${className}`}>
      <div className="stats-header">
        <h3 className="stats-title">Compression Results</h3>
        <span className="stats-algorithm">{stats.algorithm}</span>
      </div>

      <div className="stats-grid">
        <StatCard
          label="Original Size"
          value={formatFileSize(stats.originalSize)}
          icon="📄"
        />
        <StatCard
          label="Compressed Size"
          value={formatFileSize(stats.compressedSize)}
          icon="📦"
        />
        <StatCard
          label="Compression"
          value={`${compressionPercentage.toFixed(1)}%`}
          icon="📊"
          highlight={compressionPercentage > 50}
        />
        <StatCard
          label="Time"
          value={formatDuration(stats.compressionTimeMs)}
          icon="⏱️"
        />
      </div>

      <div className="stats-section">
        <h4 className="section-title">Performance</h4>
        <div className="metrics-list">
          <MetricRow label="Compression Speed" value={`${throughput.toFixed(2)} MB/s`} />
          {stats.decompressionTimeMs && (
            <MetricRow
              label="Decompression Time"
              value={formatDuration(stats.decompressionTimeMs)}
            />
          )}
          <MetricRow
            label="Compression Ratio"
            value={`${stats.ratio.toFixed(3)} : 1`}
          />
        </div>
      </div>

      {/* Metadata */}
      {Object.keys(stats.metadata).length > 0 && (
        <div className="stats-section">
          <h4 className="section-title">Details</h4>
          <div className="metadata-list">
            {Object.entries(stats.metadata).map(([key, value]) => (
              <MetricRow key={key} label={key} value={value} />
            ))}
          </div>
        </div>
      )}

      <style>{compressionStatsStyles}</style>
    </div>
  );
};

/**
 * Stat card component
 */
const StatCard: React.FC<{
  label: string;
  value: string;
  icon?: string;
  highlight?: boolean;
}> = ({ label, value, icon, highlight = false }) => (
  <div className={`stat-card ${highlight ? 'stat-card-highlight' : ''}`}>
    {icon && <div className="stat-icon">{icon}</div>}
    <div className="stat-content">
      <div className="stat-label">{label}</div>
      <div className="stat-value">{value}</div>
    </div>
  </div>
);

/**
 * Metric row component
 */
const MetricRow: React.FC<{
  label: string;
  value: string;
}> = ({ label, value }) => (
  <div className="metric-row">
    <span className="metric-label">{label}</span>
    <span className="metric-value">{value}</span>
  </div>
);

/**
 * Algorithm bar component
 */
const AlgorithmBar: React.FC<{
  algorithm: CompressionAlgorithm;
  count: number;
  total: number;
}> = ({ algorithm, count, total }) => {
  const percentage = total > 0 ? (count / total) * 100 : 0;

  return (
    <div className="algorithm-bar">
      <div className="algorithm-info">
        <span className="algorithm-name">{algorithm}</span>
        <span className="algorithm-count">
          {count} ({percentage.toFixed(1)}%)
        </span>
      </div>
      <div className="algorithm-progress">
        <div
          className="algorithm-progress-bar"
          style={{ width: `${percentage}%` }}
        />
      </div>
    </div>
  );
};

/**
 * Styles for compression stats components
 */
const compressionStatsStyles = `
  .compression-stats,
  .single-stats {
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
    background: #ffffff;
    border-radius: 8px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  .stats-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }

  .stats-title {
    margin: 0;
    font-size: 24px;
    font-weight: 600;
    color: #1a1a1a;
  }

  .stats-algorithm {
    padding: 4px 12px;
    background: #f3f4f6;
    border-radius: 4px;
    font-size: 14px;
    font-weight: 500;
    color: #374151;
  }

  .stats-actions {
    display: flex;
    gap: 8px;
  }

  .stats-button {
    width: 32px;
    height: 32px;
    padding: 0;
    background: #f3f4f6;
    border: none;
    border-radius: 4px;
    font-size: 16px;
    cursor: pointer;
    transition: background-color 0.2s;
  }

  .stats-button:hover {
    background: #e5e7eb;
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 16px;
    margin-bottom: 24px;
  }

  .stat-card {
    display: flex;
    gap: 12px;
    padding: 16px;
    background: #f9fafb;
    border-radius: 6px;
    border: 1px solid #e5e7eb;
    transition: transform 0.2s, box-shadow 0.2s;
  }

  .stat-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .stat-card-highlight {
    background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
    border-color: #2563eb;
  }

  .stat-card-highlight .stat-label,
  .stat-card-highlight .stat-value {
    color: #ffffff;
  }

  .stat-icon {
    font-size: 24px;
    line-height: 1;
  }

  .stat-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .stat-label {
    font-size: 13px;
    color: #6b7280;
    font-weight: 500;
  }

  .stat-value {
    font-size: 20px;
    font-weight: 600;
    color: #1a1a1a;
  }

  .stats-section {
    margin-top: 24px;
  }

  .section-title {
    margin: 0 0 12px 0;
    font-size: 16px;
    font-weight: 600;
    color: #374151;
  }

  .metrics-list,
  .metadata-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px;
    background: #f9fafb;
    border-radius: 6px;
  }

  .metric-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 0;
    border-bottom: 1px solid #e5e7eb;
  }

  .metric-row:last-child {
    border-bottom: none;
  }

  .metric-label {
    font-size: 14px;
    color: #6b7280;
  }

  .metric-value {
    font-size: 14px;
    font-weight: 600;
    color: #1a1a1a;
  }

  .algorithm-chart {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .algorithm-bar {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .algorithm-info {
    display: flex;
    justify-content: space-between;
    font-size: 14px;
  }

  .algorithm-name {
    font-weight: 500;
    color: #374151;
  }

  .algorithm-count {
    color: #6b7280;
  }

  .algorithm-progress {
    height: 8px;
    background: #e5e7eb;
    border-radius: 4px;
    overflow: hidden;
  }

  .algorithm-progress-bar {
    height: 100%;
    background: linear-gradient(90deg, #3b82f6 0%, #2563eb 100%);
    border-radius: 4px;
    transition: width 0.3s ease;
  }

  @media (max-width: 640px) {
    .compression-stats,
    .single-stats {
      padding: 16px;
    }

    .stats-title {
      font-size: 20px;
    }

    .stats-grid {
      grid-template-columns: 1fr;
    }
  }
`;

export default CompressionStats;
