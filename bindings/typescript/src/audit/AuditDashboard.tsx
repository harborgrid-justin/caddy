/**
 * Audit Dashboard Component
 * Main dashboard displaying comprehensive audit metrics and overview
 */

import React, { useState, useEffect, useMemo } from 'react';
import type {
  AuditMetrics,
  AuditEvent,
  AuditFilter,
  TimeRange,
  AuditSeverity,
  AuditEventType,
} from './types';

interface AuditDashboardProps {
  organizationId?: string;
  onNavigate?: (view: string) => void;
}

export const AuditDashboard: React.FC<AuditDashboardProps> = ({
  organizationId,
  onNavigate,
}) => {
  const [metrics, setMetrics] = useState<AuditMetrics | null>(null);
  const [recentEvents, setRecentEvents] = useState<AuditEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const [timeRange, setTimeRange] = useState<TimeRange>({
    start: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000), // Last 7 days
    end: new Date(),
  });

  useEffect(() => {
    loadDashboardData();
  }, [organizationId, timeRange]);

  const loadDashboardData = async () => {
    setLoading(true);
    try {
      const params = new URLSearchParams({
        start_date: timeRange.start.toISOString(),
        end_date: timeRange.end.toISOString(),
        ...(organizationId && { organization_id: organizationId }),
      });

      const [metricsRes, eventsRes] = await Promise.all([
        fetch(`/api/audit/metrics?${params}`),
        fetch(`/api/audit/events?${params}&limit=10`),
      ]);

      const metricsData = await metricsRes.json();
      const eventsData = await eventsRes.json();

      setMetrics(metricsData);
      setRecentEvents(eventsData.events || []);
    } catch (error) {
      console.error('Failed to load dashboard data:', error);
    } finally {
      setLoading(false);
    }
  };

  const severityDistribution = useMemo(() => {
    if (!metrics) return [];
    return Object.entries(metrics.events_by_severity).map(([severity, count]) => ({
      severity: severity as AuditSeverity,
      count,
      percentage: (count / metrics.total_events) * 100,
    }));
  }, [metrics]);

  const topEventTypes = useMemo(() => {
    if (!metrics) return [];
    return Object.entries(metrics.events_by_type)
      .sort(([, a], [, b]) => b - a)
      .slice(0, 5)
      .map(([type, count]) => ({
        type: type as AuditEventType,
        count,
      }));
  }, [metrics]);

  if (loading) {
    return (
      <div className="audit-dashboard loading">
        <div className="loading-spinner" />
        <p>Loading audit dashboard...</p>
      </div>
    );
  }

  return (
    <div className="audit-dashboard">
      {/* Header */}
      <header className="dashboard-header">
        <div>
          <h1>Audit Dashboard</h1>
          <p className="subtitle">
            Comprehensive audit trail and security monitoring
          </p>
        </div>
        <div className="dashboard-controls">
          <TimeRangeSelector value={timeRange} onChange={setTimeRange} />
          <button className="refresh-button" onClick={loadDashboardData}>
            Refresh
          </button>
        </div>
      </header>

      {/* Key Metrics */}
      {metrics && (
        <div className="metrics-grid">
          <MetricCard
            title="Total Events"
            value={formatNumber(metrics.total_events)}
            icon="📊"
            trend="neutral"
          />
          <MetricCard
            title="Anomalies Detected"
            value={metrics.anomalies_detected.toString()}
            icon="⚠️"
            trend={metrics.anomalies_detected > 0 ? 'danger' : 'success'}
            onClick={() => onNavigate?.('anomalies')}
          />
          <MetricCard
            title="Unique Users"
            value={metrics.unique_users.toString()}
            icon="👥"
            trend="neutral"
          />
          <MetricCard
            title="High Risk Events"
            value={metrics.high_risk_events.toString()}
            icon="🔥"
            trend={metrics.high_risk_events > 0 ? 'warning' : 'success'}
            onClick={() => onNavigate?.('high-risk')}
          />
          <MetricCard
            title="Failed Events"
            value={metrics.failed_events.toString()}
            icon="❌"
            trend={metrics.failed_events > 0 ? 'warning' : 'success'}
          />
          <MetricCard
            title="Unique Resources"
            value={metrics.unique_resources.toString()}
            icon="📦"
            trend="neutral"
          />
        </div>
      )}

      {/* Charts Row */}
      <div className="charts-section">
        {/* Event Timeline */}
        <div className="chart-container">
          <h3>Event Timeline</h3>
          {metrics && (
            <EventTimelineChart data={metrics.timeline} />
          )}
        </div>

        {/* Severity Distribution */}
        <div className="chart-container">
          <h3>Events by Severity</h3>
          <SeverityDistributionChart data={severityDistribution} />
        </div>
      </div>

      {/* Secondary Charts */}
      <div className="charts-section">
        {/* Top Event Types */}
        <div className="chart-container">
          <h3>Top Event Types</h3>
          <TopEventsChart data={topEventTypes} />
        </div>

        {/* Top Users */}
        <div className="chart-container">
          <h3>Most Active Users</h3>
          {metrics && <TopUsersTable users={metrics.top_users} />}
        </div>
      </div>

      {/* Recent Events */}
      <div className="recent-events-section">
        <div className="section-header">
          <h3>Recent Audit Events</h3>
          <button onClick={() => onNavigate?.('logs')}>
            View All Logs
          </button>
        </div>
        <RecentEventsTable events={recentEvents} />
      </div>

      {/* Quick Actions */}
      <div className="quick-actions">
        <QuickActionButton
          icon="🔍"
          title="Search Logs"
          description="Advanced audit log search"
          onClick={() => onNavigate?.('logs')}
        />
        <QuickActionButton
          icon="📋"
          title="Compliance Reports"
          description="Generate compliance reports"
          onClick={() => onNavigate?.('compliance')}
        />
        <QuickActionButton
          icon="📤"
          title="Export Logs"
          description="Export audit data"
          onClick={() => onNavigate?.('export')}
        />
        <QuickActionButton
          icon="🔔"
          title="Configure Alerts"
          description="Set up audit alerts"
          onClick={() => onNavigate?.('alerts')}
        />
      </div>
    </div>
  );
};

// Time Range Selector Component
function TimeRangeSelector({
  value,
  onChange,
}: {
  value: TimeRange;
  onChange: (range: TimeRange) => void;
}) {
  const presets = [
    { label: 'Last 24 Hours', hours: 24 },
    { label: 'Last 7 Days', days: 7 },
    { label: 'Last 30 Days', days: 30 },
    { label: 'Last 90 Days', days: 90 },
  ];

  return (
    <div className="time-range-selector">
      <select
        onChange={(e) => {
          const preset = presets[parseInt(e.target.value)];
          if (preset) {
            const ms = (preset.hours || 0) * 60 * 60 * 1000 + (preset.days || 0) * 24 * 60 * 60 * 1000;
            onChange({
              start: new Date(Date.now() - ms),
              end: new Date(),
            });
          }
        }}
      >
        {presets.map((preset, index) => (
          <option key={index} value={index}>
            {preset.label}
          </option>
        ))}
      </select>
    </div>
  );
}

// Metric Card Component
function MetricCard({
  title,
  value,
  icon,
  trend,
  onClick,
}: {
  title: string;
  value: string;
  icon?: string;
  trend?: 'success' | 'warning' | 'danger' | 'neutral';
  onClick?: () => void;
}) {
  return (
    <div
      className={`metric-card ${trend ? `trend-${trend}` : ''} ${onClick ? 'clickable' : ''}`}
      onClick={onClick}
    >
      {icon && <div className="metric-icon">{icon}</div>}
      <div className="metric-content">
        <h4>{title}</h4>
        <div className="metric-value">{value}</div>
      </div>
    </div>
  );
}

// Event Timeline Chart
function EventTimelineChart({
  data,
}: {
  data: { timestamp: string; count: number; anomalies: number; high_risk: number }[];
}) {
  const maxCount = Math.max(...data.map((d) => d.count), 1);

  return (
    <div className="timeline-chart">
      {data.map((point, index) => (
        <div key={index} className="timeline-bar">
          <div
            className="bar"
            style={{ height: `${(point.count / maxCount) * 100}%` }}
            title={`${point.count} events`}
          />
          {point.anomalies > 0 && (
            <div className="anomaly-indicator" title={`${point.anomalies} anomalies`} />
          )}
          <div className="bar-label">
            {new Date(point.timestamp).toLocaleDateString('en-US', {
              month: 'short',
              day: 'numeric',
            })}
          </div>
        </div>
      ))}
    </div>
  );
}

// Severity Distribution Chart
function SeverityDistributionChart({
  data,
}: {
  data: { severity: AuditSeverity; count: number; percentage: number }[];
}) {
  const severityColors: Record<AuditSeverity, string> = {
    low: '#10b981',
    medium: '#f59e0b',
    high: '#f97316',
    critical: '#ef4444',
  };

  return (
    <div className="severity-chart">
      <div className="severity-bars">
        {data.map((item) => (
          <div key={item.severity} className="severity-item">
            <div className="severity-label">
              <span className="severity-name">{item.severity}</span>
              <span className="severity-count">{item.count}</span>
            </div>
            <div className="severity-bar-container">
              <div
                className="severity-bar"
                style={{
                  width: `${item.percentage}%`,
                  backgroundColor: severityColors[item.severity],
                }}
              />
            </div>
            <span className="severity-percentage">
              {item.percentage.toFixed(1)}%
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}

// Top Events Chart
function TopEventsChart({
  data,
}: {
  data: { type: AuditEventType; count: number }[];
}) {
  const maxCount = Math.max(...data.map((d) => d.count), 1);

  return (
    <div className="top-events-chart">
      {data.map((item) => (
        <div key={item.type} className="event-item">
          <div className="event-label">{formatEventType(item.type)}</div>
          <div className="event-bar-container">
            <div
              className="event-bar"
              style={{ width: `${(item.count / maxCount) * 100}%` }}
            />
          </div>
          <div className="event-count">{item.count}</div>
        </div>
      ))}
    </div>
  );
}

// Top Users Table
function TopUsersTable({
  users,
}: {
  users: { user_id: string; user_email: string; event_count: number; risk_score: number }[];
}) {
  return (
    <table className="top-users-table">
      <thead>
        <tr>
          <th>User</th>
          <th>Events</th>
          <th>Risk Score</th>
        </tr>
      </thead>
      <tbody>
        {users.map((user) => (
          <tr key={user.user_id}>
            <td>{user.user_email}</td>
            <td>{user.event_count}</td>
            <td>
              <RiskScoreBadge score={user.risk_score} />
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

// Recent Events Table
function RecentEventsTable({ events }: { events: AuditEvent[] }) {
  return (
    <table className="recent-events-table">
      <thead>
        <tr>
          <th>Time</th>
          <th>Event Type</th>
          <th>User</th>
          <th>Resource</th>
          <th>Status</th>
          <th>Severity</th>
        </tr>
      </thead>
      <tbody>
        {events.map((event) => (
          <tr key={event.id}>
            <td>{formatTimestamp(event.timestamp)}</td>
            <td>{formatEventType(event.event_type)}</td>
            <td>{event.user_email || 'System'}</td>
            <td>
              {event.resource_name || event.resource_id || '-'}
            </td>
            <td>
              <StatusBadge status={event.status} />
            </td>
            <td>
              <SeverityBadge severity={event.severity} />
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

// Quick Action Button
function QuickActionButton({
  icon,
  title,
  description,
  onClick,
}: {
  icon: string;
  title: string;
  description: string;
  onClick: () => void;
}) {
  return (
    <button className="quick-action-button" onClick={onClick}>
      <div className="action-icon">{icon}</div>
      <div className="action-content">
        <div className="action-title">{title}</div>
        <div className="action-description">{description}</div>
      </div>
    </button>
  );
}

// Utility Components
function StatusBadge({ status }: { status: string }) {
  const colors: Record<string, string> = {
    success: 'green',
    failure: 'red',
    pending: 'yellow',
    blocked: 'gray',
  };

  return (
    <span className={`badge badge-${colors[status] || 'gray'}`}>
      {status}
    </span>
  );
}

function SeverityBadge({ severity }: { severity: AuditSeverity }) {
  const colors: Record<AuditSeverity, string> = {
    low: 'green',
    medium: 'yellow',
    high: 'orange',
    critical: 'red',
  };

  return (
    <span className={`badge badge-${colors[severity]}`}>
      {severity}
    </span>
  );
}

function RiskScoreBadge({ score }: { score: number }) {
  const getColor = (score: number) => {
    if (score >= 80) return 'red';
    if (score >= 60) return 'orange';
    if (score >= 40) return 'yellow';
    return 'green';
  };

  return (
    <span className={`badge badge-${getColor(score)}`}>
      {score.toFixed(0)}
    </span>
  );
}

// Utility Functions
function formatNumber(num: number): string {
  return new Intl.NumberFormat('en-US').format(num);
}

function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp);
  return new Intl.DateTimeFormat('en-US', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date);
}

function formatEventType(eventType: AuditEventType): string {
  return eventType
    .split('.')
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(' ');
}
