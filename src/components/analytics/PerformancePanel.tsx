/**
 * Performance Panel
 *
 * Performance profiling visualization and analysis.
 */

import React, { useState, useMemo } from 'react';
import { usePerformanceProfiles } from './useAnalytics';
import type { TimeRange, ProfileReport, FlameNode } from './types';

interface PerformancePanelProps {
  timeRange: TimeRange;
}

export function PerformancePanel({ timeRange }: PerformancePanelProps) {
  const { profiles, loading, error, refetch } = usePerformanceProfiles(50);
  const [sortBy, setSortBy] = useState<'avg' | 'total' | 'calls' | 'errors'>('avg');
  const [filterText, setFilterText] = useState('');
  const [selectedProfile, setSelectedProfile] = useState<ProfileReport | null>(null);

  // Sort and filter profiles
  const filteredProfiles = useMemo(() => {
    if (!profiles) return [];

    let filtered = profiles;

    // Apply filter
    if (filterText) {
      filtered = filtered.filter((p) =>
        p.operation_name.toLowerCase().includes(filterText.toLowerCase())
      );
    }

    // Sort
    const sorted = [...filtered].sort((a, b) => {
      switch (sortBy) {
        case 'avg':
          return b.avg_duration_ms - a.avg_duration_ms;
        case 'total':
          return b.total_duration_ms - a.total_duration_ms;
        case 'calls':
          return b.total_calls - a.total_calls;
        case 'errors':
          return b.error_rate - a.error_rate;
        default:
          return 0;
      }
    });

    return sorted;
  }, [profiles, sortBy, filterText]);

  // Calculate summary statistics
  const summary = useMemo(() => {
    if (!profiles || profiles.length === 0) {
      return {
        totalOperations: 0,
        totalCalls: 0,
        avgDuration: 0,
        slowestOperation: null,
        operationsWithErrors: 0,
      };
    }

    const totalOperations = profiles.length;
    const totalCalls = profiles.reduce((sum, p) => sum + p.total_calls, 0);
    const avgDuration =
      profiles.reduce((sum, p) => sum + p.avg_duration_ms, 0) / profiles.length;
    const slowestOperation = [...profiles].sort(
      (a, b) => b.avg_duration_ms - a.avg_duration_ms
    )[0];
    const operationsWithErrors = profiles.filter((p) => p.error_rate > 0).length;

    return {
      totalOperations,
      totalCalls,
      avgDuration,
      slowestOperation,
      operationsWithErrors,
    };
  }, [profiles]);

  if (loading && !profiles) {
    return (
      <div className="performance-panel loading">
        <div className="loading-spinner" />
        <p>Loading performance data...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="performance-panel error">
        <p>Error: {error.message}</p>
        <button onClick={refetch}>Retry</button>
      </div>
    );
  }

  return (
    <div className="performance-panel">
      {/* Header */}
      <div className="panel-header">
        <h2>Performance Profiling</h2>
        <button onClick={refetch} className="refresh-btn">
          🔄 Refresh
        </button>
      </div>

      {/* Summary Cards */}
      <div className="summary-cards">
        <SummaryCard
          title="Total Operations"
          value={summary.totalOperations.toString()}
          icon="⚙️"
        />
        <SummaryCard
          title="Total Calls"
          value={summary.totalCalls.toLocaleString()}
          icon="📞"
        />
        <SummaryCard
          title="Avg Duration"
          value={`${summary.avgDuration.toFixed(2)}ms`}
          icon="⏱️"
        />
        <SummaryCard
          title="Operations with Errors"
          value={summary.operationsWithErrors.toString()}
          icon="⚠️"
          alert={summary.operationsWithErrors > 0}
        />
      </div>

      {/* Slowest Operation Highlight */}
      {summary.slowestOperation && (
        <div className="slowest-operation-alert">
          <h3>⚡ Slowest Operation</h3>
          <p>
            <strong>{summary.slowestOperation.operation_name}</strong> averages{' '}
            <strong>{summary.slowestOperation.avg_duration_ms.toFixed(2)}ms</strong>
          </p>
        </div>
      )}

      {/* Controls */}
      <div className="panel-controls">
        <input
          type="text"
          placeholder="Filter operations..."
          value={filterText}
          onChange={(e) => setFilterText(e.target.value)}
          className="filter-input"
        />
        <select
          value={sortBy}
          onChange={(e) => setSortBy(e.target.value as any)}
          className="sort-select"
        >
          <option value="avg">Sort by Avg Duration</option>
          <option value="total">Sort by Total Duration</option>
          <option value="calls">Sort by Call Count</option>
          <option value="errors">Sort by Error Rate</option>
        </select>
      </div>

      {/* Performance Table */}
      <div className="performance-table-container">
        <table className="performance-table">
          <thead>
            <tr>
              <th>Operation</th>
              <th>Total Calls</th>
              <th>Avg Duration</th>
              <th>Min Duration</th>
              <th>Max Duration</th>
              <th>Total Duration</th>
              <th>Error Rate</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {filteredProfiles.map((profile) => (
              <tr
                key={profile.operation_name}
                className={selectedProfile?.operation_name === profile.operation_name ? 'selected' : ''}
              >
                <td className="operation-name">{profile.operation_name}</td>
                <td>{profile.total_calls.toLocaleString()}</td>
                <td>
                  <DurationBadge duration={profile.avg_duration_ms} />
                </td>
                <td>{profile.min_duration_ms.toFixed(2)}ms</td>
                <td>{profile.max_duration_ms.toFixed(2)}ms</td>
                <td>{profile.total_duration_ms.toFixed(2)}ms</td>
                <td>
                  <ErrorRateBadge rate={profile.error_rate} />
                </td>
                <td>
                  <button
                    onClick={() => setSelectedProfile(profile)}
                    className="details-btn"
                  >
                    Details
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Selected Profile Details */}
      {selectedProfile && (
        <ProfileDetails
          profile={selectedProfile}
          onClose={() => setSelectedProfile(null)}
        />
      )}

      {/* Performance Visualization */}
      <div className="performance-visualization">
        <h3>Duration Distribution</h3>
        <DurationHistogram profiles={filteredProfiles} />
      </div>
    </div>
  );
}

// Summary Card Component
function SummaryCard({
  title,
  value,
  icon,
  alert,
}: {
  title: string;
  value: string;
  icon: string;
  alert?: boolean;
}) {
  return (
    <div className={`summary-card ${alert ? 'alert' : ''}`}>
      <div className="card-icon">{icon}</div>
      <div className="card-content">
        <h4>{title}</h4>
        <div className="card-value">{value}</div>
      </div>
    </div>
  );
}

// Duration Badge Component
function DurationBadge({ duration }: { duration: number }) {
  let className = 'duration-badge';
  if (duration < 10) className += ' fast';
  else if (duration < 100) className += ' medium';
  else if (duration < 1000) className += ' slow';
  else className += ' very-slow';

  return <span className={className}>{duration.toFixed(2)}ms</span>;
}

// Error Rate Badge Component
function ErrorRateBadge({ rate }: { rate: number }) {
  const percentage = (rate * 100).toFixed(2);
  let className = 'error-rate-badge';
  if (rate === 0) className += ' success';
  else if (rate < 0.01) className += ' warning';
  else className += ' danger';

  return <span className={className}>{percentage}%</span>;
}

// Profile Details Modal
function ProfileDetails({
  profile,
  onClose,
}: {
  profile: ProfileReport;
  onClose: () => void;
}) {
  return (
    <div className="profile-details-modal">
      <div className="modal-overlay" onClick={onClose} />
      <div className="modal-content">
        <div className="modal-header">
          <h3>{profile.operation_name}</h3>
          <button onClick={onClose} className="close-btn">
            ✕
          </button>
        </div>

        <div className="modal-body">
          {/* Statistics Grid */}
          <div className="stats-grid">
            <StatItem label="Total Calls" value={profile.total_calls.toLocaleString()} />
            <StatItem
              label="Total Duration"
              value={`${profile.total_duration_ms.toFixed(2)}ms`}
            />
            <StatItem
              label="Average Duration"
              value={`${profile.avg_duration_ms.toFixed(2)}ms`}
            />
            <StatItem
              label="Min Duration"
              value={`${profile.min_duration_ms.toFixed(2)}ms`}
            />
            <StatItem
              label="Max Duration"
              value={`${profile.max_duration_ms.toFixed(2)}ms`}
            />
            <StatItem
              label="Error Rate"
              value={`${(profile.error_rate * 100).toFixed(2)}%`}
            />
          </div>

          {/* Recent Spans */}
          {profile.recent_spans && profile.recent_spans.length > 0 && (
            <div className="recent-spans">
              <h4>Recent Executions</h4>
              <table>
                <thead>
                  <tr>
                    <th>Time</th>
                    <th>Duration</th>
                    <th>Status</th>
                  </tr>
                </thead>
                <tbody>
                  {profile.recent_spans.slice(0, 10).map((span) => (
                    <tr key={span.id}>
                      <td>{new Date(span.start_time).toLocaleTimeString()}</td>
                      <td>
                        {span.duration_us
                          ? `${(span.duration_us / 1000).toFixed(2)}ms`
                          : 'N/A'}
                      </td>
                      <td>
                        {span.error ? (
                          <span className="status-error">Error</span>
                        ) : (
                          <span className="status-success">Success</span>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

// Statistics Item Component
function StatItem({ label, value }: { label: string; value: string }) {
  return (
    <div className="stat-item">
      <div className="stat-label">{label}</div>
      <div className="stat-value">{value}</div>
    </div>
  );
}

// Duration Histogram Component
function DurationHistogram({ profiles }: { profiles: ProfileReport[] }) {
  const buckets = useMemo(() => {
    if (profiles.length === 0) return [];

    // Create logarithmic buckets
    const bucketRanges = [
      { min: 0, max: 10, label: '0-10ms' },
      { min: 10, max: 100, label: '10-100ms' },
      { min: 100, max: 1000, label: '100ms-1s' },
      { min: 1000, max: 10000, label: '1s-10s' },
      { min: 10000, max: Infinity, label: '>10s' },
    ];

    return bucketRanges.map((range) => {
      const count = profiles.filter(
        (p) => p.avg_duration_ms >= range.min && p.avg_duration_ms < range.max
      ).length;

      return {
        label: range.label,
        count,
        percentage: (count / profiles.length) * 100,
      };
    });
  }, [profiles]);

  const maxCount = Math.max(...buckets.map((b) => b.count), 1);

  return (
    <div className="duration-histogram">
      {buckets.map((bucket) => (
        <div key={bucket.label} className="histogram-bar">
          <div className="bar-label">{bucket.label}</div>
          <div className="bar-container">
            <div
              className="bar-fill"
              style={{ width: `${(bucket.count / maxCount) * 100}%` }}
            />
          </div>
          <div className="bar-value">
            {bucket.count} ({bucket.percentage.toFixed(1)}%)
          </div>
        </div>
      ))}
    </div>
  );
}
