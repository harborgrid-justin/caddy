/**
 * Analytics Dashboard
 *
 * Main dashboard component displaying comprehensive analytics overview.
 */

import React, { useState, useMemo } from 'react';
import { useAnalytics, useHealthStatus, useUsageStats, usePerformanceProfiles } from './useAnalytics';
import { formatBytes, formatDuration, formatNumber } from './AnalyticsProvider';
import { MetricsChart } from './MetricsChart';
import { PerformancePanel } from './PerformancePanel';
import { UsageStats } from './UsageStats';
import { ReportBuilder } from './ReportBuilder';
import { ExportPanel } from './ExportPanel';
import type { TimeRange } from './types';

type TabType = 'overview' | 'metrics' | 'performance' | 'usage' | 'reports' | 'export';

export function Dashboard() {
  const { config, enabled, loading: analyticsLoading, error } = useAnalytics();
  const { health, loading: healthLoading } = useHealthStatus();
  const { stats, loading: statsLoading } = useUsageStats();
  const { profiles, loading: profilesLoading } = usePerformanceProfiles(5);

  const [activeTab, setActiveTab] = useState<TabType>('overview');
  const [timeRange, setTimeRange] = useState<TimeRange>({
    start: new Date(Date.now() - 24 * 60 * 60 * 1000), // Last 24 hours
    end: new Date(),
  });

  // Calculate key metrics
  const keyMetrics = useMemo(() => {
    if (!health || !stats) return null;

    return {
      totalEvents: stats.total_events,
      activeUsers: stats.active_users,
      storageSize: health.storage_size_bytes,
      uptime: health.uptime_seconds,
      errorRate: stats.error_rate,
      avgOperationTime: profiles.length > 0
        ? profiles.reduce((sum, p) => sum + p.avg_duration_ms, 0) / profiles.length
        : 0,
    };
  }, [health, stats, profiles]);

  if (analyticsLoading || !config) {
    return (
      <div className="analytics-dashboard loading">
        <div className="loading-spinner" />
        <p>Loading analytics...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="analytics-dashboard error">
        <h2>Error Loading Analytics</h2>
        <p>{error.message}</p>
      </div>
    );
  }

  if (!enabled) {
    return (
      <div className="analytics-dashboard disabled">
        <h2>Analytics Disabled</h2>
        <p>Analytics collection is currently disabled. Enable it in settings to view data.</p>
      </div>
    );
  }

  return (
    <div className="analytics-dashboard">
      {/* Header */}
      <header className="dashboard-header">
        <h1>Analytics Dashboard</h1>
        <div className="dashboard-controls">
          <TimeRangeSelector value={timeRange} onChange={setTimeRange} />
          <button className="refresh-button" onClick={() => window.location.reload()}>
            Refresh
          </button>
        </div>
      </header>

      {/* Status Bar */}
      {health && (
        <div className="status-bar">
          <StatusIndicator
            label="System Status"
            value="Operational"
            status="success"
          />
          <StatusIndicator
            label="Uptime"
            value={formatDuration(health.uptime_seconds)}
            status="info"
          />
          <StatusIndicator
            label="Storage"
            value={formatBytes(health.storage_size_bytes)}
            status="info"
          />
          <StatusIndicator
            label="Active Profiles"
            value={health.active_profiles.toString()}
            status="info"
          />
        </div>
      )}

      {/* Navigation Tabs */}
      <nav className="dashboard-tabs">
        <TabButton
          active={activeTab === 'overview'}
          onClick={() => setActiveTab('overview')}
        >
          Overview
        </TabButton>
        <TabButton
          active={activeTab === 'metrics'}
          onClick={() => setActiveTab('metrics')}
        >
          Metrics
        </TabButton>
        <TabButton
          active={activeTab === 'performance'}
          onClick={() => setActiveTab('performance')}
        >
          Performance
        </TabButton>
        <TabButton
          active={activeTab === 'usage'}
          onClick={() => setActiveTab('usage')}
        >
          Usage
        </TabButton>
        <TabButton
          active={activeTab === 'reports'}
          onClick={() => setActiveTab('reports')}
        >
          Reports
        </TabButton>
        <TabButton
          active={activeTab === 'export'}
          onClick={() => setActiveTab('export')}
        >
          Export
        </TabButton>
      </nav>

      {/* Main Content */}
      <main className="dashboard-content">
        {activeTab === 'overview' && (
          <OverviewTab
            keyMetrics={keyMetrics}
            health={health}
            stats={stats}
            profiles={profiles}
            timeRange={timeRange}
          />
        )}
        {activeTab === 'metrics' && (
          <MetricsTab timeRange={timeRange} />
        )}
        {activeTab === 'performance' && (
          <PerformancePanel timeRange={timeRange} />
        )}
        {activeTab === 'usage' && (
          <UsageStats timeRange={timeRange} />
        )}
        {activeTab === 'reports' && (
          <ReportBuilder />
        )}
        {activeTab === 'export' && (
          <ExportPanel />
        )}
      </main>
    </div>
  );
}

// Overview Tab Component
function OverviewTab({ keyMetrics, health, stats, profiles, timeRange }: any) {
  return (
    <div className="overview-tab">
      {/* Key Metrics Grid */}
      {keyMetrics && (
        <div className="metrics-grid">
          <MetricCard
            title="Total Events"
            value={formatNumber(keyMetrics.totalEvents)}
            icon="📊"
          />
          <MetricCard
            title="Active Users"
            value={keyMetrics.activeUsers.toString()}
            icon="👥"
          />
          <MetricCard
            title="Error Rate"
            value={`${(keyMetrics.errorRate * 100).toFixed(2)}%`}
            icon="⚠️"
            trend={keyMetrics.errorRate > 0.05 ? 'danger' : 'success'}
          />
          <MetricCard
            title="Avg Response Time"
            value={`${keyMetrics.avgOperationTime.toFixed(2)}ms`}
            icon="⚡"
          />
        </div>
      )}

      {/* Charts Row */}
      <div className="charts-row">
        <div className="chart-container">
          <h3>System Metrics</h3>
          <MetricsChart
            metricName="system.cpu.usage_percent"
            timeRange={timeRange}
            title="CPU Usage"
            yAxisLabel="Usage %"
          />
        </div>
        <div className="chart-container">
          <h3>Request Rate</h3>
          <MetricsChart
            metricName="requests.total"
            timeRange={timeRange}
            title="Requests per Second"
            yAxisLabel="Requests"
          />
        </div>
      </div>

      {/* Quick Stats */}
      {stats && (
        <div className="quick-stats">
          <h3>Usage Summary</h3>
          <div className="stats-list">
            <StatItem label="Active Sessions" value={stats.active_sessions} />
            <StatItem
              label="Avg Session Duration"
              value={formatDuration(stats.avg_session_duration_secs)}
            />
            <StatItem label="Total Events" value={formatNumber(stats.total_events)} />
          </div>
        </div>
      )}

      {/* Recent Performance */}
      {profiles && profiles.length > 0 && (
        <div className="recent-performance">
          <h3>Top 5 Operations by Duration</h3>
          <table className="performance-table">
            <thead>
              <tr>
                <th>Operation</th>
                <th>Calls</th>
                <th>Avg Duration</th>
                <th>Error Rate</th>
              </tr>
            </thead>
            <tbody>
              {profiles.map((profile) => (
                <tr key={profile.operation_name}>
                  <td>{profile.operation_name}</td>
                  <td>{formatNumber(profile.total_calls)}</td>
                  <td>{profile.avg_duration_ms.toFixed(2)}ms</td>
                  <td className={profile.error_rate > 0.01 ? 'error-rate-high' : ''}>
                    {(profile.error_rate * 100).toFixed(2)}%
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

// Metrics Tab Component
function MetricsTab({ timeRange }: { timeRange: TimeRange }) {
  const commonMetrics = [
    'system.cpu.usage_percent',
    'system.memory.used_bytes',
    'cad.entities.total',
    'cad.viewport.fps',
    'commands.total',
  ];

  return (
    <div className="metrics-tab">
      <h2>System Metrics</h2>
      <div className="metrics-charts-grid">
        {commonMetrics.map((metricName) => (
          <div key={metricName} className="metric-chart-card">
            <MetricsChart
              metricName={metricName}
              timeRange={timeRange}
              title={metricName.replace(/\./g, ' ').replace(/_/g, ' ')}
              height={300}
            />
          </div>
        ))}
      </div>
    </div>
  );
}

// UI Components
function TimeRangeSelector({ value, onChange }: {
  value: TimeRange;
  onChange: (range: TimeRange) => void;
}) {
  const presets = [
    { label: 'Last Hour', hours: 1 },
    { label: 'Last 6 Hours', hours: 6 },
    { label: 'Last 24 Hours', hours: 24 },
    { label: 'Last 7 Days', hours: 168 },
    { label: 'Last 30 Days', hours: 720 },
  ];

  return (
    <div className="time-range-selector">
      <select
        onChange={(e) => {
          const hours = parseInt(e.target.value);
          onChange({
            start: new Date(Date.now() - hours * 60 * 60 * 1000),
            end: new Date(),
          });
        }}
      >
        {presets.map((preset) => (
          <option key={preset.hours} value={preset.hours}>
            {preset.label}
          </option>
        ))}
      </select>
    </div>
  );
}

function TabButton({ active, onClick, children }: {
  active: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      className={`tab-button ${active ? 'active' : ''}`}
      onClick={onClick}
    >
      {children}
    </button>
  );
}

function StatusIndicator({ label, value, status }: {
  label: string;
  value: string;
  status: 'success' | 'warning' | 'danger' | 'info';
}) {
  return (
    <div className={`status-indicator status-${status}`}>
      <span className="status-label">{label}</span>
      <span className="status-value">{value}</span>
    </div>
  );
}

function MetricCard({ title, value, icon, trend }: {
  title: string;
  value: string;
  icon?: string;
  trend?: 'success' | 'warning' | 'danger';
}) {
  return (
    <div className={`metric-card ${trend ? `trend-${trend}` : ''}`}>
      {icon && <div className="metric-icon">{icon}</div>}
      <div className="metric-content">
        <h4>{title}</h4>
        <div className="metric-value">{value}</div>
      </div>
    </div>
  );
}

function StatItem({ label, value }: { label: string; value: string | number }) {
  return (
    <div className="stat-item">
      <span className="stat-label">{label}:</span>
      <span className="stat-value">{value}</span>
    </div>
  );
}
