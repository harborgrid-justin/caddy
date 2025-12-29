/**
 * Monitoring Dashboard Component
 *
 * Provides real-time monitoring dashboard with:
 * - Live monitor status display
 * - Uptime statistics and graphs
 * - Performance metrics visualization
 * - Change detection alerts
 * - Monitor management (create, edit, delete)
 */

import React, { useState, useEffect, useCallback } from 'react';
import {
  Monitor,
  MonitorStatus,
  CheckType,
  CheckResult,
  UptimeStats,
  PerformanceMetrics,
  ChangeDetection,
  MonitorAlert,
  AlertSeverity,
  MonitorFormState,
  CreateMonitorRequest,
  DashboardFilters,
} from './types';

interface MonitoringDashboardProps {
  apiBaseUrl?: string;
  refreshInterval?: number; // milliseconds
  onMonitorCreated?: (monitor: Monitor) => void;
  onAlertTriggered?: (alert: MonitorAlert) => void;
}

export const MonitoringDashboard: React.FC<MonitoringDashboardProps> = ({
  apiBaseUrl = '/api/monitoring',
  refreshInterval = 10000,
  onMonitorCreated,
  onAlertTriggered,
}) => {
  const [monitors, setMonitors] = useState<Monitor[]>([]);
  const [selectedMonitor, setSelectedMonitor] = useState<Monitor | null>(null);
  const [uptimeStats, setUptimeStats] = useState<Map<string, UptimeStats>>(new Map());
  const [recentResults, setRecentResults] = useState<Map<string, CheckResult[]>>(new Map());
  const [performanceData, setPerformanceData] = useState<Map<string, PerformanceMetrics[]>>(
    new Map()
  );
  const [changeHistory, setChangeHistory] = useState<Map<string, ChangeDetection[]>>(
    new Map()
  );
  const [alerts, setAlerts] = useState<MonitorAlert[]>([]);
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [filters, setFilters] = useState<DashboardFilters>({
    status: undefined,
    searchQuery: '',
  });

  const [formState, setFormState] = useState<MonitorFormState>({
    name: '',
    check_type: 'http',
    url: '',
    http_method: 'GET',
    http_expected_status: 200,
    http_timeout_ms: 5000,
    interval_seconds: 60,
    alert_on_failure: true,
    alert_threshold: 3,
    tags: [],
  });

  // Fetch monitors
  const fetchMonitors = useCallback(async () => {
    try {
      const response = await fetch(`${apiBaseUrl}/monitors`);
      if (!response.ok) throw new Error('Failed to fetch monitors');
      const data = await response.json();
      setMonitors(data);
    } catch (err) {
      console.error('Error fetching monitors:', err);
      setError(err instanceof Error ? err.message : 'Unknown error');
    }
  }, [apiBaseUrl]);

  // Fetch uptime stats
  const fetchUptimeStats = useCallback(
    async (monitorId: string) => {
      try {
        const response = await fetch(`${apiBaseUrl}/monitors/${monitorId}/uptime`);
        if (!response.ok) return;
        const data: UptimeStats = await response.json();
        setUptimeStats((prev) => new Map(prev).set(monitorId, data));
      } catch (err) {
        console.error('Error fetching uptime stats:', err);
      }
    },
    [apiBaseUrl]
  );

  // Fetch recent results
  const fetchRecentResults = useCallback(
    async (monitorId: string) => {
      try {
        const response = await fetch(`${apiBaseUrl}/monitors/${monitorId}/results?limit=50`);
        if (!response.ok) return;
        const data: CheckResult[] = await response.json();
        setRecentResults((prev) => new Map(prev).set(monitorId, data));
      } catch (err) {
        console.error('Error fetching recent results:', err);
      }
    },
    [apiBaseUrl]
  );

  // Fetch performance data
  const fetchPerformanceData = useCallback(
    async (monitorId: string) => {
      try {
        const response = await fetch(
          `${apiBaseUrl}/monitors/${monitorId}/performance?limit=100`
        );
        if (!response.ok) return;
        const data: PerformanceMetrics[] = await response.json();
        setPerformanceData((prev) => new Map(prev).set(monitorId, data));
      } catch (err) {
        console.error('Error fetching performance data:', err);
      }
    },
    [apiBaseUrl]
  );

  // Fetch change history
  const fetchChangeHistory = useCallback(
    async (monitorId: string) => {
      try {
        const response = await fetch(`${apiBaseUrl}/monitors/${monitorId}/changes`);
        if (!response.ok) return;
        const data: ChangeDetection[] = await response.json();
        setChangeHistory((prev) => new Map(prev).set(monitorId, data));
      } catch (err) {
        console.error('Error fetching change history:', err);
      }
    },
    [apiBaseUrl]
  );

  // Fetch alerts
  const fetchAlerts = useCallback(async () => {
    try {
      const response = await fetch(`${apiBaseUrl}/alerts?active=true`);
      if (!response.ok) return;
      const data: MonitorAlert[] = await response.json();
      setAlerts(data);

      // Trigger callback for new alerts
      if (onAlertTriggered) {
        data.forEach((alert) => {
          if (!alert.acknowledged) {
            onAlertTriggered(alert);
          }
        });
      }
    } catch (err) {
      console.error('Error fetching alerts:', err);
    }
  }, [apiBaseUrl, onAlertTriggered]);

  // Initial load
  useEffect(() => {
    fetchMonitors();
    fetchAlerts();
  }, [fetchMonitors, fetchAlerts]);

  // Periodic refresh
  useEffect(() => {
    const interval = setInterval(() => {
      fetchMonitors();
      fetchAlerts();

      // Fetch stats for all monitors
      monitors.forEach((monitor) => {
        fetchUptimeStats(monitor.id);
        fetchRecentResults(monitor.id);
      });
    }, refreshInterval);

    return () => clearInterval(interval);
  }, [monitors, refreshInterval, fetchMonitors, fetchAlerts, fetchUptimeStats, fetchRecentResults]);

  // Fetch details when monitor is selected
  useEffect(() => {
    if (selectedMonitor) {
      fetchUptimeStats(selectedMonitor.id);
      fetchRecentResults(selectedMonitor.id);
      fetchPerformanceData(selectedMonitor.id);
      fetchChangeHistory(selectedMonitor.id);
    }
  }, [selectedMonitor, fetchUptimeStats, fetchRecentResults, fetchPerformanceData, fetchChangeHistory]);

  // Create monitor
  const handleCreateMonitor = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      let checkType: CheckType;
      switch (formState.check_type) {
        case 'http':
          checkType = {
            type: 'Http',
            url: formState.url,
            method: formState.http_method || 'GET',
            expected_status: formState.http_expected_status || 200,
            timeout_ms: formState.http_timeout_ms || 5000,
          };
          break;
        case 'accessibility':
          checkType = {
            type: 'AccessibilityScan',
            url: formState.url,
            standards: formState.accessibility_standards || ['WCAG2.1-AA'],
          };
          break;
        case 'content_change':
          checkType = {
            type: 'ContentChange',
            url: formState.url,
            selector: formState.content_selector,
            hash_algorithm: formState.content_hash_algorithm || 'sha256',
          };
          break;
        case 'performance':
          checkType = {
            type: 'Performance',
            url: formState.url,
            max_load_time_ms: formState.perf_max_load_time || 3000,
            max_first_byte_ms: formState.perf_max_first_byte || 1000,
          };
          break;
        default:
          throw new Error('Invalid check type');
      }

      const tags = Object.fromEntries(formState.tags.map((t) => [t.key, t.value]));

      const request: CreateMonitorRequest = {
        name: formState.name,
        check_type: checkType,
        interval_seconds: formState.interval_seconds,
        alert_on_failure: formState.alert_on_failure,
        alert_threshold: formState.alert_threshold,
        tags,
      };

      const response = await fetch(`${apiBaseUrl}/monitors`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(request),
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.message || 'Failed to create monitor');
      }

      const newMonitor: Monitor = await response.json();
      setMonitors([...monitors, newMonitor]);
      setShowCreateForm(false);
      resetForm();

      if (onMonitorCreated) {
        onMonitorCreated(newMonitor);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  // Delete monitor
  const handleDeleteMonitor = async (monitorId: string) => {
    if (!confirm('Are you sure you want to delete this monitor?')) return;

    try {
      const response = await fetch(`${apiBaseUrl}/monitors/${monitorId}`, {
        method: 'DELETE',
      });

      if (!response.ok) throw new Error('Failed to delete monitor');

      setMonitors(monitors.filter((m) => m.id !== monitorId));
      if (selectedMonitor?.id === monitorId) {
        setSelectedMonitor(null);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    }
  };

  // Acknowledge alert
  const handleAcknowledgeAlert = async (alertId: string) => {
    try {
      const response = await fetch(`${apiBaseUrl}/alerts/${alertId}/acknowledge`, {
        method: 'POST',
      });

      if (!response.ok) throw new Error('Failed to acknowledge alert');

      setAlerts(
        alerts.map((a) => (a.id === alertId ? { ...a, acknowledged: true } : a))
      );
    } catch (err) {
      console.error('Error acknowledging alert:', err);
    }
  };

  // Reset form
  const resetForm = () => {
    setFormState({
      name: '',
      check_type: 'http',
      url: '',
      http_method: 'GET',
      http_expected_status: 200,
      http_timeout_ms: 5000,
      interval_seconds: 60,
      alert_on_failure: true,
      alert_threshold: 3,
      tags: [],
    });
  };

  // Format date
  const formatDate = (dateStr?: string) => {
    if (!dateStr) return 'N/A';
    return new Date(dateStr).toLocaleString();
  };

  // Get status color
  const getStatusColor = (status: MonitorStatus): string => {
    switch (status) {
      case MonitorStatus.Up:
        return 'text-green-600 bg-green-100';
      case MonitorStatus.Down:
        return 'text-red-600 bg-red-100';
      case MonitorStatus.Degraded:
        return 'text-yellow-600 bg-yellow-100';
      default:
        return 'text-gray-600 bg-gray-100';
    }
  };

  // Get alert severity color
  const getSeverityColor = (severity: AlertSeverity): string => {
    switch (severity) {
      case AlertSeverity.Critical:
        return 'bg-red-600';
      case AlertSeverity.Error:
        return 'bg-orange-600';
      case AlertSeverity.Warning:
        return 'bg-yellow-600';
      default:
        return 'bg-blue-600';
    }
  };

  // Calculate overall stats
  const totalMonitors = monitors.length;
  const activeMonitors = monitors.filter((m) => m.enabled).length;
  const upMonitors = Array.from(uptimeStats.values()).filter(
    (s) => s.last_check && s.uptime_percentage > 99
  ).length;
  const downMonitors = monitors.length - upMonitors;
  const activeAlerts = alerts.filter((a) => !a.acknowledged && !a.resolved).length;

  // Filter monitors
  const filteredMonitors = monitors.filter((monitor) => {
    if (filters.searchQuery) {
      const query = filters.searchQuery.toLowerCase();
      if (!monitor.name.toLowerCase().includes(query)) {
        return false;
      }
    }
    return true;
  });

  return (
    <div className="monitoring-dashboard p-6 bg-gray-50 min-h-screen">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="mb-6">
          <h1 className="text-3xl font-bold text-gray-900">Monitoring Dashboard</h1>
          <p className="mt-2 text-gray-600">
            Real-time monitoring, uptime tracking, and performance metrics
          </p>
        </div>

        {/* Error Display */}
        {error && (
          <div className="mb-4 p-4 bg-red-50 border border-red-200 rounded-lg">
            <p className="text-red-800">{error}</p>
          </div>
        )}

        {/* Alerts Banner */}
        {activeAlerts > 0 && (
          <div className="mb-6 p-4 bg-red-50 border-l-4 border-red-600 rounded-lg">
            <div className="flex items-center justify-between">
              <div>
                <h3 className="text-lg font-semibold text-red-800">
                  {activeAlerts} Active Alert{activeAlerts !== 1 ? 's' : ''}
                </h3>
                <p className="text-sm text-red-700">
                  Monitors require immediate attention
                </p>
              </div>
              <button
                onClick={() => {
                  const alertsSection = document.getElementById('alerts-section');
                  alertsSection?.scrollIntoView({ behavior: 'smooth' });
                }}
                className="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700"
              >
                View Alerts
              </button>
            </div>
          </div>
        )}

        {/* Overview Stats */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
          <div className="bg-white p-4 rounded-lg shadow">
            <h3 className="text-sm font-medium text-gray-500 uppercase">Total Monitors</h3>
            <p className="mt-2 text-3xl font-bold text-gray-900">{totalMonitors}</p>
            <p className="text-xs text-gray-500 mt-1">{activeMonitors} active</p>
          </div>

          <div className="bg-white p-4 rounded-lg shadow">
            <h3 className="text-sm font-medium text-gray-500 uppercase">Online</h3>
            <p className="mt-2 text-3xl font-bold text-green-600">{upMonitors}</p>
            <p className="text-xs text-gray-500 mt-1">
              {totalMonitors > 0
                ? ((upMonitors / totalMonitors) * 100).toFixed(1)
                : 0}
              % uptime
            </p>
          </div>

          <div className="bg-white p-4 rounded-lg shadow">
            <h3 className="text-sm font-medium text-gray-500 uppercase">Offline</h3>
            <p className="mt-2 text-3xl font-bold text-red-600">{downMonitors}</p>
            <p className="text-xs text-gray-500 mt-1">
              Requires attention
            </p>
          </div>

          <div className="bg-white p-4 rounded-lg shadow">
            <h3 className="text-sm font-medium text-gray-500 uppercase">Active Alerts</h3>
            <p className="mt-2 text-3xl font-bold text-orange-600">{activeAlerts}</p>
            <p className="text-xs text-gray-500 mt-1">Unacknowledged</p>
          </div>
        </div>

        {/* Controls */}
        <div className="mb-6 flex justify-between items-center">
          <div className="flex gap-2">
            <button
              onClick={() => setShowCreateForm(!showCreateForm)}
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition"
            >
              {showCreateForm ? 'Cancel' : 'Add Monitor'}
            </button>
            <button
              onClick={fetchMonitors}
              disabled={loading}
              className="px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 transition disabled:opacity-50"
            >
              Refresh
            </button>
          </div>

          <div className="flex gap-2">
            <input
              type="text"
              placeholder="Search monitors..."
              value={filters.searchQuery}
              onChange={(e) => setFilters({ ...filters, searchQuery: e.target.value })}
              className="px-3 py-2 border border-gray-300 rounded-lg"
            />
          </div>
        </div>

        {/* Create Form */}
        {showCreateForm && (
          <div className="mb-6 bg-white p-6 rounded-lg shadow">
            <h2 className="text-xl font-bold mb-4">Create New Monitor</h2>
            <form onSubmit={handleCreateMonitor} className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Monitor Name
                  </label>
                  <input
                    type="text"
                    required
                    value={formState.name}
                    onChange={(e) => setFormState({ ...formState, name: e.target.value })}
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                    placeholder="Production Website"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Check Type
                  </label>
                  <select
                    value={formState.check_type}
                    onChange={(e) =>
                      setFormState({
                        ...formState,
                        check_type: e.target.value as any,
                      })
                    }
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                  >
                    <option value="http">HTTP Check</option>
                    <option value="accessibility">Accessibility Scan</option>
                    <option value="content_change">Content Change Detection</option>
                    <option value="performance">Performance Test</option>
                  </select>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    URL
                  </label>
                  <input
                    type="url"
                    required
                    value={formState.url}
                    onChange={(e) => setFormState({ ...formState, url: e.target.value })}
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                    placeholder="https://example.com"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Check Interval (seconds)
                  </label>
                  <input
                    type="number"
                    required
                    value={formState.interval_seconds}
                    onChange={(e) =>
                      setFormState({
                        ...formState,
                        interval_seconds: parseInt(e.target.value),
                      })
                    }
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                    min="10"
                  />
                </div>

                {formState.check_type === 'http' && (
                  <>
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">
                        HTTP Method
                      </label>
                      <select
                        value={formState.http_method}
                        onChange={(e) =>
                          setFormState({ ...formState, http_method: e.target.value })
                        }
                        className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                      >
                        <option value="GET">GET</option>
                        <option value="POST">POST</option>
                        <option value="HEAD">HEAD</option>
                      </select>
                    </div>

                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">
                        Expected Status
                      </label>
                      <input
                        type="number"
                        value={formState.http_expected_status}
                        onChange={(e) =>
                          setFormState({
                            ...formState,
                            http_expected_status: parseInt(e.target.value),
                          })
                        }
                        className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                      />
                    </div>
                  </>
                )}

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Alert Threshold (consecutive failures)
                  </label>
                  <input
                    type="number"
                    value={formState.alert_threshold}
                    onChange={(e) =>
                      setFormState({
                        ...formState,
                        alert_threshold: parseInt(e.target.value),
                      })
                    }
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                    min="1"
                  />
                </div>

                <div className="flex items-center">
                  <input
                    type="checkbox"
                    id="alert_on_failure"
                    checked={formState.alert_on_failure}
                    onChange={(e) =>
                      setFormState({ ...formState, alert_on_failure: e.target.checked })
                    }
                    className="mr-2"
                  />
                  <label htmlFor="alert_on_failure" className="text-sm text-gray-700">
                    Send alerts on failure
                  </label>
                </div>
              </div>

              <div className="flex gap-4">
                <button
                  type="submit"
                  disabled={loading}
                  className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition disabled:opacity-50"
                >
                  {loading ? 'Creating...' : 'Create Monitor'}
                </button>
                <button
                  type="button"
                  onClick={() => setShowCreateForm(false)}
                  className="px-6 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 transition"
                >
                  Cancel
                </button>
              </div>
            </form>
          </div>
        )}

        {/* Monitors Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 mb-6">
          {filteredMonitors.map((monitor) => {
            const stats = uptimeStats.get(monitor.id);
            const latestResults = recentResults.get(monitor.id) || [];
            const latestResult = latestResults[0];

            return (
              <div
                key={monitor.id}
                className="bg-white p-4 rounded-lg shadow hover:shadow-lg transition cursor-pointer"
                onClick={() => setSelectedMonitor(monitor)}
              >
                <div className="flex justify-between items-start mb-3">
                  <div>
                    <h3 className="font-semibold text-gray-900">{monitor.name}</h3>
                    <p className="text-xs text-gray-500 mt-1">
                      {monitor.check_type.type} • Every {monitor.interval_seconds}s
                    </p>
                  </div>
                  {latestResult && (
                    <span
                      className={`px-2 py-1 text-xs font-semibold rounded-full ${getStatusColor(
                        latestResult.status
                      )}`}
                    >
                      {latestResult.status}
                    </span>
                  )}
                </div>

                {stats && (
                  <div className="space-y-2">
                    <div className="flex justify-between text-sm">
                      <span className="text-gray-600">Uptime</span>
                      <span className="font-semibold text-gray-900">
                        {stats.uptime_percentage.toFixed(2)}%
                      </span>
                    </div>

                    <div className="w-full bg-gray-200 rounded-full h-2">
                      <div
                        className={`h-2 rounded-full ${
                          stats.uptime_percentage >= 99
                            ? 'bg-green-600'
                            : stats.uptime_percentage >= 95
                            ? 'bg-yellow-600'
                            : 'bg-red-600'
                        }`}
                        style={{ width: `${stats.uptime_percentage}%` }}
                      />
                    </div>

                    <div className="flex justify-between text-xs text-gray-500">
                      <span>Avg: {stats.average_response_time_ms.toFixed(0)}ms</span>
                      <span>
                        {stats.successful_checks}/{stats.total_checks} checks
                      </span>
                    </div>
                  </div>
                )}

                {!monitor.enabled && (
                  <div className="mt-3 text-xs text-gray-500">
                    Monitor disabled
                  </div>
                )}
              </div>
            );
          })}

          {filteredMonitors.length === 0 && (
            <div className="col-span-full text-center py-12 text-gray-500">
              No monitors found
            </div>
          )}
        </div>

        {/* Active Alerts */}
        {alerts.length > 0 && (
          <div id="alerts-section" className="mb-6">
            <h2 className="text-xl font-bold mb-4">Active Alerts</h2>
            <div className="space-y-3">
              {alerts.map((alert) => (
                <div
                  key={alert.id}
                  className={`p-4 rounded-lg border-l-4 ${
                    alert.acknowledged ? 'bg-gray-50 border-gray-400' : 'bg-white border-red-600'
                  } shadow`}
                >
                  <div className="flex justify-between items-start">
                    <div className="flex-1">
                      <div className="flex items-center gap-2">
                        <span
                          className={`px-2 py-1 text-xs font-semibold rounded text-white ${getSeverityColor(
                            alert.severity
                          )}`}
                        >
                          {alert.severity}
                        </span>
                        <h3 className="font-semibold text-gray-900">{alert.monitor_name}</h3>
                      </div>
                      <p className="mt-2 text-sm text-gray-700">{alert.message}</p>
                      <p className="mt-1 text-xs text-gray-500">
                        {formatDate(alert.created_at)} • {alert.consecutive_failures} consecutive
                        failures
                      </p>
                    </div>
                    {!alert.acknowledged && (
                      <button
                        onClick={() => handleAcknowledgeAlert(alert.id)}
                        className="ml-4 px-3 py-1 text-sm bg-blue-600 text-white rounded hover:bg-blue-700"
                      >
                        Acknowledge
                      </button>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Monitor Details Modal */}
        {selectedMonitor && (
          <div
            className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4"
            onClick={() => setSelectedMonitor(null)}
          >
            <div
              className="bg-white rounded-lg p-6 max-w-4xl w-full max-h-[90vh] overflow-y-auto"
              onClick={(e) => e.stopPropagation()}
            >
              <div className="flex justify-between items-start mb-6">
                <div>
                  <h2 className="text-2xl font-bold">{selectedMonitor.name}</h2>
                  <p className="text-sm text-gray-600 mt-1">
                    ID: {selectedMonitor.id.slice(0, 8)}
                  </p>
                </div>
                <div className="flex gap-2">
                  <button
                    onClick={() => handleDeleteMonitor(selectedMonitor.id)}
                    className="px-3 py-1 text-sm bg-red-600 text-white rounded hover:bg-red-700"
                  >
                    Delete
                  </button>
                  <button
                    onClick={() => setSelectedMonitor(null)}
                    className="px-3 py-1 text-sm bg-gray-200 text-gray-700 rounded hover:bg-gray-300"
                  >
                    Close
                  </button>
                </div>
              </div>

              {/* Uptime Stats */}
              {uptimeStats.has(selectedMonitor.id) && (
                <div className="mb-6 p-4 bg-gray-50 rounded-lg">
                  <h3 className="font-semibold mb-3">Uptime Statistics</h3>
                  <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                    {Object.entries(uptimeStats.get(selectedMonitor.id)!).map(([key, value]) => {
                      if (key === 'monitor_id') return null;
                      return (
                        <div key={key}>
                          <p className="text-xs text-gray-600">{key.replace(/_/g, ' ')}</p>
                          <p className="text-lg font-semibold">
                            {typeof value === 'number' ? value.toFixed(2) : String(value)}
                          </p>
                        </div>
                      );
                    })}
                  </div>
                </div>
              )}

              {/* Recent Results */}
              {recentResults.has(selectedMonitor.id) && (
                <div className="mb-6">
                  <h3 className="font-semibold mb-3">Recent Check Results</h3>
                  <div className="space-y-2 max-h-64 overflow-y-auto">
                    {recentResults.get(selectedMonitor.id)!.slice(0, 10).map((result) => (
                      <div key={result.check_id} className="flex justify-between items-center text-sm p-2 bg-gray-50 rounded">
                        <span
                          className={`px-2 py-1 text-xs font-semibold rounded ${getStatusColor(
                            result.status
                          )}`}
                        >
                          {result.status}
                        </span>
                        <span className="text-gray-600">{result.response_time_ms}ms</span>
                        <span className="text-gray-500">{formatDate(result.timestamp)}</span>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default MonitoringDashboard;
