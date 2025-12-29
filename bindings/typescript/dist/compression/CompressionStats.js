import React, { useState, useEffect } from 'react';
import { calculateCompressionPercentage, calculateThroughput, formatFileSize, formatDuration, } from './types';
import { compressionService } from './CompressionService';
export const CompressionStats = ({ stats = null, showDetailed = true, refreshInterval = 0, className = '', }) => {
    const [metrics, setMetrics] = useState(compressionService.getMetrics());
    useEffect(() => {
        if (refreshInterval > 0) {
            const interval = setInterval(() => {
                setMetrics(compressionService.getMetrics());
            }, refreshInterval);
            return () => clearInterval(interval);
        }
    }, [refreshInterval]);
    const handleRefresh = () => {
        setMetrics(compressionService.getMetrics());
    };
    const handleReset = () => {
        compressionService.resetMetrics();
        setMetrics(compressionService.getMetrics());
    };
    if (stats) {
        return React.createElement(SingleOperationStats, { stats: stats, className: className });
    }
    return (React.createElement("div", { className: `compression-stats ${className}` },
        React.createElement("div", { className: "stats-header" },
            React.createElement("h3", { className: "stats-title" }, "Compression Performance"),
            React.createElement("div", { className: "stats-actions" },
                React.createElement("button", { onClick: handleRefresh, className: "stats-button", title: "Refresh" }, "\u21BB"),
                React.createElement("button", { onClick: handleReset, className: "stats-button", title: "Reset" }, "\u2715"))),
        React.createElement("div", { className: "stats-grid" },
            React.createElement(StatCard, { label: "Files Compressed", value: metrics.totalFilesCompressed.toLocaleString(), icon: "\uD83D\uDCE6" }),
            React.createElement(StatCard, { label: "Files Decompressed", value: metrics.totalFilesDecompressed.toLocaleString(), icon: "\uD83D\uDCC2" }),
            React.createElement(StatCard, { label: "Total Compressed", value: formatFileSize(metrics.totalBytesCompressed), icon: "\uD83D\uDCBE" }),
            React.createElement(StatCard, { label: "Average Ratio", value: `${((1 - metrics.averageCompressionRatio) * 100).toFixed(1)}%`, icon: "\uD83D\uDCCA" })),
        showDetailed && (React.createElement(React.Fragment, null,
            React.createElement("div", { className: "stats-section" },
                React.createElement("h4", { className: "section-title" }, "Performance Metrics"),
                React.createElement("div", { className: "metrics-list" },
                    React.createElement(MetricRow, { label: "Average Compression Speed", value: `${metrics.averageCompressionSpeed.toFixed(2)} MB/s` }),
                    React.createElement(MetricRow, { label: "Average Decompression Speed", value: `${metrics.averageDecompressionSpeed.toFixed(2)} MB/s` }),
                    React.createElement(MetricRow, { label: "Total Data Processed", value: formatFileSize(metrics.totalBytesCompressed + metrics.totalBytesDecompressed) }))),
            Object.keys(metrics.algorithmUsage).length > 0 && (React.createElement("div", { className: "stats-section" },
                React.createElement("h4", { className: "section-title" }, "Algorithm Usage"),
                React.createElement("div", { className: "algorithm-chart" }, Object.entries(metrics.algorithmUsage)
                    .sort((a, b) => b[1] - a[1])
                    .map(([algorithm, count]) => (React.createElement(AlgorithmBar, { key: algorithm, algorithm: algorithm, count: count, total: metrics.totalFilesCompressed })))))))),
        React.createElement("style", null, compressionStatsStyles)));
};
const SingleOperationStats = ({ stats, className = '' }) => {
    const compressionPercentage = calculateCompressionPercentage(stats);
    const throughput = calculateThroughput(stats);
    return (React.createElement("div", { className: `single-stats ${className}` },
        React.createElement("div", { className: "stats-header" },
            React.createElement("h3", { className: "stats-title" }, "Compression Results"),
            React.createElement("span", { className: "stats-algorithm" }, stats.algorithm)),
        React.createElement("div", { className: "stats-grid" },
            React.createElement(StatCard, { label: "Original Size", value: formatFileSize(stats.originalSize), icon: "\uD83D\uDCC4" }),
            React.createElement(StatCard, { label: "Compressed Size", value: formatFileSize(stats.compressedSize), icon: "\uD83D\uDCE6" }),
            React.createElement(StatCard, { label: "Compression", value: `${compressionPercentage.toFixed(1)}%`, icon: "\uD83D\uDCCA", highlight: compressionPercentage > 50 }),
            React.createElement(StatCard, { label: "Time", value: formatDuration(stats.compressionTimeMs), icon: "\u23F1\uFE0F" })),
        React.createElement("div", { className: "stats-section" },
            React.createElement("h4", { className: "section-title" }, "Performance"),
            React.createElement("div", { className: "metrics-list" },
                React.createElement(MetricRow, { label: "Compression Speed", value: `${throughput.toFixed(2)} MB/s` }),
                stats.decompressionTimeMs && (React.createElement(MetricRow, { label: "Decompression Time", value: formatDuration(stats.decompressionTimeMs) })),
                React.createElement(MetricRow, { label: "Compression Ratio", value: `${stats.ratio.toFixed(3)} : 1` }))),
        Object.keys(stats.metadata).length > 0 && (React.createElement("div", { className: "stats-section" },
            React.createElement("h4", { className: "section-title" }, "Details"),
            React.createElement("div", { className: "metadata-list" }, Object.entries(stats.metadata).map(([key, value]) => (React.createElement(MetricRow, { key: key, label: key, value: value })))))),
        React.createElement("style", null, compressionStatsStyles)));
};
const StatCard = ({ label, value, icon, highlight = false }) => (React.createElement("div", { className: `stat-card ${highlight ? 'stat-card-highlight' : ''}` },
    icon && React.createElement("div", { className: "stat-icon" }, icon),
    React.createElement("div", { className: "stat-content" },
        React.createElement("div", { className: "stat-label" }, label),
        React.createElement("div", { className: "stat-value" }, value))));
const MetricRow = ({ label, value }) => (React.createElement("div", { className: "metric-row" },
    React.createElement("span", { className: "metric-label" }, label),
    React.createElement("span", { className: "metric-value" }, value)));
const AlgorithmBar = ({ algorithm, count, total }) => {
    const percentage = total > 0 ? (count / total) * 100 : 0;
    return (React.createElement("div", { className: "algorithm-bar" },
        React.createElement("div", { className: "algorithm-info" },
            React.createElement("span", { className: "algorithm-name" }, algorithm),
            React.createElement("span", { className: "algorithm-count" },
                count,
                " (",
                percentage.toFixed(1),
                "%)")),
        React.createElement("div", { className: "algorithm-progress" },
            React.createElement("div", { className: "algorithm-progress-bar", style: { width: `${percentage}%` } }))));
};
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
//# sourceMappingURL=CompressionStats.js.map