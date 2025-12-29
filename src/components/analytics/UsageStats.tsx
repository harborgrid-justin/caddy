/**
 * Usage Statistics Display
 *
 * Comprehensive usage analytics and user behavior tracking.
 */

import React, { useState, useMemo } from 'react';
import { useUsageStats } from './useAnalytics';
import type { TimeRange, EventType } from './types';
import { formatNumber, formatDuration } from './AnalyticsProvider';

interface UsageStatsProps {
  timeRange: TimeRange;
}

export function UsageStats({ timeRange }: UsageStatsProps) {
  const { stats, loading, error, refetch } = useUsageStats();
  const [selectedEventType, setSelectedEventType] = useState<EventType | null>(null);

  // Calculate trends
  const trends = useMemo(() => {
    if (!stats) return null;

    return {
      eventsGrowth: 0, // Would calculate from historical data
      usersGrowth: 0,
      errorTrend: stats.error_rate > 0.05 ? 'increasing' : 'stable',
    };
  }, [stats]);

  if (loading && !stats) {
    return (
      <div className="usage-stats loading">
        <div className="loading-spinner" />
        <p>Loading usage statistics...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="usage-stats error">
        <p>Error: {error.message}</p>
        <button onClick={refetch}>Retry</button>
      </div>
    );
  }

  if (!stats) {
    return (
      <div className="usage-stats empty">
        <p>No usage data available</p>
      </div>
    );
  }

  return (
    <div className="usage-stats">
      {/* Header */}
      <div className="stats-header">
        <h2>Usage Analytics</h2>
        <button onClick={refetch} className="refresh-btn">
          🔄 Refresh
        </button>
      </div>

      {/* Overview Cards */}
      <div className="overview-cards">
        <UsageCard
          title="Total Events"
          value={formatNumber(stats.total_events)}
          icon="📊"
          subtitle={`Since ${stats.first_event ? new Date(stats.first_event).toLocaleDateString() : 'N/A'}`}
        />
        <UsageCard
          title="Active Users"
          value={stats.active_users.toString()}
          icon="👥"
          trend={trends?.usersGrowth}
        />
        <UsageCard
          title="Active Sessions"
          value={stats.active_sessions.toString()}
          icon="🔗"
        />
        <UsageCard
          title="Error Rate"
          value={`${(stats.error_rate * 100).toFixed(2)}%`}
          icon="⚠️"
          alert={stats.error_rate > 0.05}
        />
      </div>

      {/* Session Statistics */}
      <div className="session-stats">
        <h3>Session Statistics</h3>
        <div className="stats-grid">
          <StatCard
            label="Total Session Duration"
            value={formatDuration(stats.total_session_duration_secs)}
          />
          <StatCard
            label="Average Session Duration"
            value={formatDuration(stats.avg_session_duration_secs)}
          />
          <StatCard
            label="Active Sessions"
            value={stats.active_sessions.toString()}
          />
        </div>
      </div>

      {/* Events by Type */}
      <div className="events-by-type">
        <h3>Events by Type</h3>
        <div className="event-types-grid">
          {Object.entries(stats.events_by_type)
            .sort(([, a], [, b]) => b - a)
            .map(([type, count]) => (
              <EventTypeCard
                key={type}
                type={type as EventType}
                count={count}
                percentage={(count / stats.total_events) * 100}
                selected={selectedEventType === type}
                onClick={() => setSelectedEventType(
                  selectedEventType === type ? null : (type as EventType)
                )}
              />
            ))}
        </div>
      </div>

      {/* Most Used Features */}
      {stats.most_used_features.length > 0 && (
        <div className="most-used-features">
          <h3>Most Used Features</h3>
          <div className="features-list">
            {stats.most_used_features.map(([feature, count], index) => (
              <FeatureBar
                key={feature}
                rank={index + 1}
                name={feature}
                count={count}
                maxCount={stats.most_used_features[0][1]}
              />
            ))}
          </div>
        </div>
      )}

      {/* Most Executed Commands */}
      {stats.most_executed_commands.length > 0 && (
        <div className="most-executed-commands">
          <h3>Most Executed Commands</h3>
          <div className="commands-list">
            {stats.most_executed_commands.map(([command, count], index) => (
              <CommandBar
                key={command}
                rank={index + 1}
                name={command}
                count={count}
                maxCount={stats.most_executed_commands[0][1]}
              />
            ))}
          </div>
        </div>
      )}

      {/* Usage Timeline */}
      <div className="usage-timeline">
        <h3>Usage Timeline</h3>
        <TimelineVisualization stats={stats} timeRange={timeRange} />
      </div>
    </div>
  );
}

// Usage Card Component
function UsageCard({
  title,
  value,
  icon,
  subtitle,
  trend,
  alert,
}: {
  title: string;
  value: string;
  icon: string;
  subtitle?: string;
  trend?: number;
  alert?: boolean;
}) {
  return (
    <div className={`usage-card ${alert ? 'alert' : ''}`}>
      <div className="card-icon">{icon}</div>
      <div className="card-content">
        <h4>{title}</h4>
        <div className="card-value">
          {value}
          {trend !== undefined && trend !== 0 && (
            <span className={`trend ${trend > 0 ? 'positive' : 'negative'}`}>
              {trend > 0 ? '↑' : '↓'} {Math.abs(trend).toFixed(1)}%
            </span>
          )}
        </div>
        {subtitle && <div className="card-subtitle">{subtitle}</div>}
      </div>
    </div>
  );
}

// Stat Card Component
function StatCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="stat-card">
      <div className="stat-label">{label}</div>
      <div className="stat-value">{value}</div>
    </div>
  );
}

// Event Type Card Component
function EventTypeCard({
  type,
  count,
  percentage,
  selected,
  onClick,
}: {
  type: EventType;
  count: number;
  percentage: number;
  selected: boolean;
  onClick: () => void;
}) {
  const icons: Record<EventType, string> = {
    [EventType.AppStart]: '🚀',
    [EventType.AppStop]: '⏹️',
    [EventType.FeatureUsed]: '✨',
    [EventType.CommandExecuted]: '⚡',
    [EventType.FileOpened]: '📂',
    [EventType.FileSaved]: '💾',
    [EventType.EntityCreated]: '➕',
    [EventType.EntityModified]: '✏️',
    [EventType.EntityDeleted]: '🗑️',
    [EventType.ToolActivated]: '🔧',
    [EventType.LayerCreated]: '📑',
    [EventType.ViewChanged]: '👁️',
    [EventType.RenderingModeChanged]: '🎨',
    [EventType.Export]: '📤',
    [EventType.Import]: '📥',
    [EventType.Error]: '❌',
    [EventType.Custom]: '🔹',
  };

  return (
    <div
      className={`event-type-card ${selected ? 'selected' : ''}`}
      onClick={onClick}
    >
      <div className="event-icon">{icons[type] || '🔹'}</div>
      <div className="event-info">
        <div className="event-type">{type}</div>
        <div className="event-count">{formatNumber(count)}</div>
        <div className="event-percentage">{percentage.toFixed(1)}%</div>
      </div>
      <div className="event-bar">
        <div
          className="event-bar-fill"
          style={{ width: `${percentage}%` }}
        />
      </div>
    </div>
  );
}

// Feature Bar Component
function FeatureBar({
  rank,
  name,
  count,
  maxCount,
}: {
  rank: number;
  name: string;
  count: number;
  maxCount: number;
}) {
  const percentage = (count / maxCount) * 100;

  return (
    <div className="feature-bar">
      <div className="feature-rank">#{rank}</div>
      <div className="feature-name">{name}</div>
      <div className="feature-bar-container">
        <div
          className="feature-bar-fill"
          style={{ width: `${percentage}%` }}
        >
          <span className="feature-count">{formatNumber(count)}</span>
        </div>
      </div>
    </div>
  );
}

// Command Bar Component
function CommandBar({
  rank,
  name,
  count,
  maxCount,
}: {
  rank: number;
  name: string;
  count: number;
  maxCount: number;
}) {
  const percentage = (count / maxCount) * 100;

  return (
    <div className="command-bar">
      <div className="command-rank">#{rank}</div>
      <div className="command-name">{name}</div>
      <div className="command-bar-container">
        <div
          className="command-bar-fill"
          style={{ width: `${percentage}%` }}
        >
          <span className="command-count">{formatNumber(count)}</span>
        </div>
      </div>
    </div>
  );
}

// Timeline Visualization Component
function TimelineVisualization({
  stats,
  timeRange,
}: {
  stats: any;
  timeRange: TimeRange;
}) {
  // This would be a more sophisticated chart in production
  // For now, showing a simple placeholder

  return (
    <div className="timeline-visualization">
      <div className="timeline-placeholder">
        <p>Timeline visualization showing activity over the selected time range</p>
        <div className="timeline-stats">
          <div>Start: {timeRange.start.toLocaleString()}</div>
          <div>End: {timeRange.end.toLocaleString()}</div>
          {stats.first_event && (
            <div>First Event: {new Date(stats.first_event).toLocaleString()}</div>
          )}
          {stats.last_event && (
            <div>Last Event: {new Date(stats.last_event).toLocaleString()}</div>
          )}
        </div>
      </div>
    </div>
  );
}

/**
 * User Engagement Score Component
 */
export function UserEngagementScore({ stats }: { stats: any }) {
  const score = useMemo(() => {
    if (!stats) return 0;

    // Calculate engagement score (0-100)
    const factors = {
      sessionDuration: Math.min((stats.avg_session_duration_secs / 3600) * 20, 30),
      eventFrequency: Math.min((stats.total_events / 1000) * 20, 30),
      featureUsage: Math.min(stats.most_used_features.length * 2, 20),
      activeUsers: Math.min(stats.active_users * 5, 20),
    };

    return Object.values(factors).reduce((sum, val) => sum + val, 0);
  }, [stats]);

  const getScoreColor = (score: number) => {
    if (score >= 80) return '#10b981'; // Green
    if (score >= 60) return '#3b82f6'; // Blue
    if (score >= 40) return '#f59e0b'; // Yellow
    return '#ef4444'; // Red
  };

  const getScoreLabel = (score: number) => {
    if (score >= 80) return 'Excellent';
    if (score >= 60) return 'Good';
    if (score >= 40) return 'Fair';
    return 'Poor';
  };

  return (
    <div className="user-engagement-score">
      <h3>User Engagement Score</h3>
      <div className="score-display">
        <div
          className="score-circle"
          style={{
            background: `conic-gradient(${getScoreColor(score)} ${score * 3.6}deg, #e5e7eb 0deg)`,
          }}
        >
          <div className="score-inner">
            <div className="score-value">{score.toFixed(0)}</div>
            <div className="score-label">{getScoreLabel(score)}</div>
          </div>
        </div>
      </div>
    </div>
  );
}
