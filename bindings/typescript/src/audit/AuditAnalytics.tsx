/**
 * Audit Analytics Component
 * Audit trend analysis and advanced analytics
 */

import React, { useState, useEffect, useMemo } from 'react';
import type { AuditAnalytics as AuditAnalyticsData, TimeRange } from './types';

interface AuditAnalyticsProps {
  organizationId?: string;
}

export const AuditAnalytics: React.FC<AuditAnalyticsProps> = ({ organizationId }) => {
  const [analytics, setAnalytics] = useState<AuditAnalyticsData | null>(null);
  const [loading, setLoading] = useState(true);
  const [timeRange, setTimeRange] = useState<TimeRange>({
    start: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000), // Last 30 days
    end: new Date(),
  });
  const [selectedTab, setSelectedTab] = useState<'trends' | 'users' | 'resources' | 'security'>(
    'trends'
  );

  useEffect(() => {
    loadAnalytics();
  }, [timeRange, organizationId]);

  const loadAnalytics = async () => {
    setLoading(true);
    try {
      const params = new URLSearchParams({
        start_date: timeRange.start.toISOString(),
        end_date: timeRange.end.toISOString(),
        ...(organizationId && { organization_id: organizationId }),
      });

      const response = await fetch(`/api/audit/analytics?${params}`);
      const data = await response.json();
      setAnalytics(data);
    } catch (error) {
      console.error('Failed to load analytics:', error);
    } finally {
      setLoading(false);
    }
  };

  const totalEvents = useMemo(() => {
    if (!analytics) return 0;
    return analytics.event_trends.reduce((sum, trend) => sum + trend.total, 0);
  }, [analytics]);

  const avgEventsPerDay = useMemo(() => {
    if (!analytics || analytics.event_trends.length === 0) return 0;
    return totalEvents / analytics.event_trends.length;
  }, [analytics, totalEvents]);

  if (loading) {
    return (
      <div className="audit-analytics loading">
        <div className="loading-spinner" />
        <p>Loading analytics...</p>
      </div>
    );
  }

  if (!analytics) {
    return (
      <div className="audit-analytics error">
        <h2>Failed to Load Analytics</h2>
        <p>Unable to load analytics data. Please try again.</p>
      </div>
    );
  }

  return (
    <div className="audit-analytics">
      {/* Header */}
      <div className="analytics-header">
        <div>
          <h2>Audit Analytics</h2>
          <p className="subtitle">
            Advanced analytics and trend analysis for audit events
          </p>
        </div>
        <TimeRangeSelector value={timeRange} onChange={setTimeRange} />
      </div>

      {/* Summary Stats */}
      <div className="summary-stats">
        <StatCard
          title="Total Events"
          value={formatNumber(totalEvents)}
          icon="📊"
          trend="neutral"
        />
        <StatCard
          title="Avg Events/Day"
          value={formatNumber(Math.round(avgEventsPerDay))}
          icon="📈"
          trend="neutral"
        />
        <StatCard
          title="Total Anomalies"
          value={analytics.security_insights.total_anomalies.toString()}
          icon="⚠️"
          trend={analytics.security_insights.total_anomalies > 0 ? 'danger' : 'success'}
        />
        <StatCard
          title="High Risk Events"
          value={analytics.security_insights.high_risk_events.toString()}
          icon="🔥"
          trend={analytics.security_insights.high_risk_events > 0 ? 'warning' : 'success'}
        />
      </div>

      {/* Tab Navigation */}
      <div className="analytics-tabs">
        <button
          className={`tab ${selectedTab === 'trends' ? 'active' : ''}`}
          onClick={() => setSelectedTab('trends')}
        >
          Event Trends
        </button>
        <button
          className={`tab ${selectedTab === 'users' ? 'active' : ''}`}
          onClick={() => setSelectedTab('users')}
        >
          User Behavior
        </button>
        <button
          className={`tab ${selectedTab === 'resources' ? 'active' : ''}`}
          onClick={() => setSelectedTab('resources')}
        >
          Resource Usage
        </button>
        <button
          className={`tab ${selectedTab === 'security' ? 'active' : ''}`}
          onClick={() => setSelectedTab('security')}
        >
          Security Insights
        </button>
      </div>

      {/* Tab Content */}
      <div className="analytics-content">
        {selectedTab === 'trends' && <EventTrendsTab analytics={analytics} />}
        {selectedTab === 'users' && <UserBehaviorTab analytics={analytics} />}
        {selectedTab === 'resources' && <ResourceUsageTab analytics={analytics} />}
        {selectedTab === 'security' && <SecurityInsightsTab analytics={analytics} />}
      </div>
    </div>
  );
};

// Event Trends Tab
function EventTrendsTab({ analytics }: { analytics: AuditAnalyticsData }) {
  // Transform event_trends data for timeline chart
  const timelineData = analytics.event_trends.map(trend => ({
    date: trend.date,
    total: trend.total,
    anomalies: trend.by_severity.high + trend.by_severity.critical,
    high_risk: trend.by_severity.critical,
  }));

  return (
    <div className="trends-tab">
      {/* Timeline Chart */}
      <div className="chart-section">
        <h3>Event Timeline</h3>
        <EventTimelineChart data={timelineData} />
      </div>

      {/* Severity Distribution */}
      <div className="charts-row">
        <div className="chart-section">
          <h3>Events by Severity</h3>
          <SeverityTrendsChart data={analytics.event_trends} />
        </div>

        <div className="chart-section">
          <h3>Event Status Distribution</h3>
          <StatusTrendsChart data={analytics.event_trends} />
        </div>
      </div>

      {/* Trends Summary */}
      <div className="trends-summary">
        <h3>Key Trends</h3>
        <div className="trends-grid">
          <TrendCard
            title="Peak Activity"
            value={findPeakActivity(analytics.event_trends)}
            icon="📈"
          />
          <TrendCard
            title="Anomaly Rate"
            value={calculateAnomalyRate(timelineData)}
            icon="⚠️"
          />
          <TrendCard
            title="High Risk Rate"
            value={calculateHighRiskRate(timelineData)}
            icon="🔥"
          />
        </div>
      </div>
    </div>
  );
}

// User Behavior Tab
function UserBehaviorTab({ analytics }: { analytics: AuditAnalyticsData }) {
  const [sortBy, setSortBy] = useState<'events' | 'risk' | 'anomalies'>('events');

  const sortedUsers = useMemo(() => {
    const users = [...analytics.user_analytics];
    switch (sortBy) {
      case 'risk':
        return users.sort((a, b) => b.risk_score - a.risk_score);
      case 'anomalies':
        return users.sort((a, b) => b.anomaly_count - a.anomaly_count);
      default:
        return users.sort((a, b) => b.total_events - a.total_events);
    }
  }, [analytics.user_analytics, sortBy]);

  return (
    <div className="user-behavior-tab">
      <div className="tab-controls">
        <label>Sort by:</label>
        <select value={sortBy} onChange={(e) => setSortBy(e.target.value as any)}>
          <option value="events">Total Events</option>
          <option value="risk">Risk Score</option>
          <option value="anomalies">Anomalies</option>
        </select>
      </div>

      <table className="user-analytics-table">
        <thead>
          <tr>
            <th>User</th>
            <th>Total Events</th>
            <th>Logins</th>
            <th>Failed Logins</th>
            <th>Data Accessed</th>
            <th>Data Modified</th>
            <th>Anomalies</th>
            <th>Risk Score</th>
            <th>Last Activity</th>
          </tr>
        </thead>
        <tbody>
          {sortedUsers.map((user) => (
            <tr key={user.user_id}>
              <td>
                <div className="user-info">
                  <div className="user-email">{user.user_email}</div>
                  <div className="user-id">{user.user_id}</div>
                </div>
              </td>
              <td>{formatNumber(user.total_events)}</td>
              <td>{user.login_count}</td>
              <td>
                <span
                  className={`failed-logins ${
                    user.failed_login_count > 3 ? 'high' : ''
                  }`}
                >
                  {user.failed_login_count}
                </span>
              </td>
              <td>{formatNumber(user.data_accessed_count)}</td>
              <td>{formatNumber(user.data_modified_count)}</td>
              <td>
                <span className={`anomaly-count ${user.anomaly_count > 0 ? 'has-anomalies' : ''}`}>
                  {user.anomaly_count}
                </span>
              </td>
              <td>
                <RiskScoreBadge score={user.risk_score} />
              </td>
              <td>{formatTimestamp(user.last_activity)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

// Resource Usage Tab
function ResourceUsageTab({ analytics }: { analytics: AuditAnalyticsData }) {
  return (
    <div className="resource-usage-tab">
      <table className="resource-analytics-table">
        <thead>
          <tr>
            <th>Resource Type</th>
            <th>Resource ID</th>
            <th>Total Accesses</th>
            <th>Unique Users</th>
            <th>Modifications</th>
            <th>Last Accessed</th>
          </tr>
        </thead>
        <tbody>
          {analytics.resource_analytics.map((resource, index) => (
            <tr key={index}>
              <td>
                <span className="resource-type-badge">{resource.resource_type}</span>
              </td>
              <td className="resource-id">{resource.resource_id}</td>
              <td>{formatNumber(resource.total_accesses)}</td>
              <td>{resource.unique_users}</td>
              <td>{resource.modifications}</td>
              <td>{formatTimestamp(resource.last_accessed)}</td>
            </tr>
          ))}
        </tbody>
      </table>

      {/* Resource Summary */}
      <div className="resource-summary">
        <h3>Resource Summary</h3>
        <div className="summary-grid">
          <div className="summary-card">
            <div className="summary-value">
              {analytics.resource_analytics.length}
            </div>
            <div className="summary-label">Total Resources</div>
          </div>
          <div className="summary-card">
            <div className="summary-value">
              {analytics.resource_analytics.reduce(
                (sum, r) => sum + r.total_accesses,
                0
              )}
            </div>
            <div className="summary-label">Total Accesses</div>
          </div>
          <div className="summary-card">
            <div className="summary-value">
              {analytics.resource_analytics.reduce((sum, r) => sum + r.modifications, 0)}
            </div>
            <div className="summary-label">Total Modifications</div>
          </div>
        </div>
      </div>
    </div>
  );
}

// Security Insights Tab
function SecurityInsightsTab({ analytics }: { analytics: AuditAnalyticsData }) {
  return (
    <div className="security-insights-tab">
      {/* Security Metrics */}
      <div className="security-metrics">
        <MetricCard
          title="Total Anomalies"
          value={analytics.security_insights.total_anomalies}
          color="orange"
        />
        <MetricCard
          title="Breach Attempts"
          value={analytics.security_insights.breach_attempts}
          color="red"
        />
        <MetricCard
          title="Unauthorized Access"
          value={analytics.security_insights.unauthorized_access_attempts}
          color="red"
        />
        <MetricCard
          title="Suspicious Activities"
          value={analytics.security_insights.suspicious_activities}
          color="yellow"
        />
      </div>

      {/* Security Patterns */}
      <div className="security-patterns">
        <h3>Security Patterns</h3>
        {analytics.security_insights.patterns.length === 0 ? (
          <div className="empty-state">
            <p>No security patterns detected</p>
          </div>
        ) : (
          analytics.security_insights.patterns.map((pattern, index) => (
            <div key={index} className={`pattern-card risk-${pattern.risk_level}`}>
              <div className="pattern-header">
                <h4>{pattern.type}</h4>
                <span className={`risk-badge risk-${pattern.risk_level}`}>
                  {pattern.risk_level.toUpperCase()}
                </span>
              </div>
              <p className="pattern-description">{pattern.description}</p>
              <div className="pattern-stats">
                <span>
                  <strong>Occurrences:</strong> {pattern.occurrence_count}
                </span>
                <span>
                  <strong>Affected Users:</strong> {pattern.affected_users.length}
                </span>
              </div>
              <div className="pattern-recommendations">
                <strong>Recommendations:</strong>
                <ul>
                  {pattern.recommendations.map((rec, idx) => (
                    <li key={idx}>{rec}</li>
                  ))}
                </ul>
              </div>
            </div>
          ))
        )}
      </div>

      {/* Compliance Insights */}
      <div className="compliance-insights">
        <h3>Compliance Overview</h3>
        <div className="compliance-grid">
          {analytics.compliance_insights.map((insight) => (
            <div key={insight.framework} className="compliance-card">
              <div className="compliance-header">
                <h4>{insight.framework}</h4>
                <div className="compliance-percentage">
                  {insight.compliant_percentage.toFixed(1)}%
                </div>
              </div>
              <div className="compliance-stats">
                <div className="stat">
                  <label>Violations:</label>
                  <span className={insight.violations > 0 ? 'has-violations' : ''}>
                    {insight.violations}
                  </span>
                </div>
                {insight.at_risk_requirements.length > 0 && (
                  <div className="at-risk">
                    <label>At Risk:</label>
                    <span>{insight.at_risk_requirements.length} requirements</span>
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

// Chart Components
function EventTimelineChart({
  data,
}: {
  data: { date: string; total: number; anomalies: number; high_risk: number }[];
}) {
  const maxCount = Math.max(...data.map((d) => d.total), 1);

  return (
    <div className="timeline-chart">
      {data.map((point, index) => (
        <div key={index} className="timeline-bar-group">
          <div className="bar-stack">
            <div
              className="bar total-bar"
              style={{ height: `${(point.total / maxCount) * 100}%` }}
              title={`${point.total} events`}
            />
            {point.anomalies > 0 && (
              <div className="anomaly-overlay" title={`${point.anomalies} anomalies`} />
            )}
          </div>
          <div className="bar-label">
            {new Date(point.date).toLocaleDateString('en-US', {
              month: 'short',
              day: 'numeric',
            })}
          </div>
        </div>
      ))}
    </div>
  );
}

function SeverityTrendsChart({
  data,
}: {
  data: { date: string; by_severity: Record<string, number> }[];
}) {
  const severityColors = {
    low: '#10b981',
    medium: '#f59e0b',
    high: '#f97316',
    critical: '#ef4444',
  };

  return (
    <div className="severity-trends-chart">
      {data.map((point, index) => (
        <div key={index} className="severity-bar-group">
          <div className="stacked-bar">
            {Object.entries(severityColors).map(([severity, color]) => {
              const count = point.by_severity[severity] || 0;
              return (
                <div
                  key={severity}
                  className="severity-segment"
                  style={{
                    height: `${count * 5}px`,
                    backgroundColor: color,
                  }}
                  title={`${severity}: ${count}`}
                />
              );
            })}
          </div>
        </div>
      ))}
    </div>
  );
}

function StatusTrendsChart({
  data,
}: {
  data: { date: string; by_status: Record<string, number> }[];
}) {
  const total = data.reduce((sum, d) => {
    return sum + Object.values(d.by_status).reduce((a, b) => a + b, 0);
  }, 0);

  const statusCounts = data.reduce(
    (acc, d) => {
      Object.entries(d.by_status).forEach(([status, count]) => {
        acc[status] = (acc[status] || 0) + count;
      });
      return acc;
    },
    {} as Record<string, number>
  );

  return (
    <div className="status-pie-chart">
      {Object.entries(statusCounts).map(([status, count]) => (
        <div key={status} className="status-segment">
          <div className={`status-color status-${status}`} />
          <span className="status-label">{status}</span>
          <span className="status-value">
            {count} ({((count / total) * 100).toFixed(1)}%)
          </span>
        </div>
      ))}
    </div>
  );
}

// Utility Components
function TimeRangeSelector({
  value,
  onChange,
}: {
  value: TimeRange;
  onChange: (range: TimeRange) => void;
}) {
  const presets = [
    { label: 'Last 7 Days', days: 7 },
    { label: 'Last 30 Days', days: 30 },
    { label: 'Last 90 Days', days: 90 },
    { label: 'Last Year', days: 365 },
  ];

  return (
    <select
      onChange={(e) => {
        const days = parseInt(e.target.value);
        onChange({
          start: new Date(Date.now() - days * 24 * 60 * 60 * 1000),
          end: new Date(),
        });
      }}
      className="time-range-select"
    >
      {presets.map((preset) => (
        <option key={preset.days} value={preset.days}>
          {preset.label}
        </option>
      ))}
    </select>
  );
}

function StatCard({
  title,
  value,
  icon,
  trend,
}: {
  title: string;
  value: string;
  icon: string;
  trend: 'success' | 'warning' | 'danger' | 'neutral';
}) {
  return (
    <div className={`stat-card trend-${trend}`}>
      <div className="stat-icon">{icon}</div>
      <div className="stat-content">
        <div className="stat-value">{value}</div>
        <div className="stat-title">{title}</div>
      </div>
    </div>
  );
}

function TrendCard({ title, value, icon }: { title: string; value: string; icon: string }) {
  return (
    <div className="trend-card">
      <div className="trend-icon">{icon}</div>
      <div className="trend-content">
        <div className="trend-title">{title}</div>
        <div className="trend-value">{value}</div>
      </div>
    </div>
  );
}

function MetricCard({
  title,
  value,
  color,
}: {
  title: string;
  value: number;
  color: string;
}) {
  return (
    <div className={`metric-card metric-${color}`}>
      <div className="metric-value">{value}</div>
      <div className="metric-title">{title}</div>
    </div>
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
    <span className={`risk-badge risk-${getColor(score)}`}>
      {score.toFixed(0)}
    </span>
  );
}

// Utility Functions
function formatNumber(num: number): string {
  return new Intl.NumberFormat('en-US').format(num);
}

function formatTimestamp(timestamp: string): string {
  return new Date(timestamp).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

function findPeakActivity(
  trends: { date: string; total: number }[]
): string {
  if (trends.length === 0) return 'N/A';
  const peak = trends.reduce((max, trend) =>
    trend.total > max.total ? trend : max
  );
  return `${new Date(peak.date).toLocaleDateString()} (${peak.total} events)`;
}

function calculateAnomalyRate(
  trends: { total: number; anomalies: number }[]
): string {
  const total = trends.reduce((sum, t) => sum + t.total, 0);
  const anomalies = trends.reduce((sum, t) => sum + t.anomalies, 0);
  if (total === 0) return '0%';
  return `${((anomalies / total) * 100).toFixed(2)}%`;
}

function calculateHighRiskRate(
  trends: { total: number; high_risk: number }[]
): string {
  const total = trends.reduce((sum, t) => sum + t.total, 0);
  const highRisk = trends.reduce((sum, t) => sum + t.high_risk, 0);
  if (total === 0) return '0%';
  return `${((highRisk / total) * 100).toFixed(2)}%`;
}
