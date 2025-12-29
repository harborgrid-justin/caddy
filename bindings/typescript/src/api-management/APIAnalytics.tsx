/**
 * CADDY API Analytics
 *
 * API usage analytics and metrics visualization with real-time monitoring.
 */

import React, { useState, useEffect } from 'react';
import {
  APIAnalytics as APIAnalyticsData,
  APIMetrics,
  EndpointMetrics,
  TimeSeriesData,
  HTTPMethod,
} from './types';

interface APIAnalyticsProps {
  projectId?: string;
  refreshInterval?: number;
}

export const APIAnalytics: React.FC<APIAnalyticsProps> = ({
  projectId = 'default',
  refreshInterval = 30000,
}) => {
  const [analytics, setAnalytics] = useState<APIAnalyticsData | null>(null);
  const [timeRange, setTimeRange] = useState<'1h' | '24h' | '7d' | '30d'>('24h');
  const [selectedMetric, setSelectedMetric] = useState<'requests' | 'errors' | 'latency'>('requests');
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    loadAnalytics();
    const interval = setInterval(loadAnalytics, refreshInterval);
    return () => clearInterval(interval);
  }, [projectId, timeRange]);

  const loadAnalytics = async () => {
    setIsLoading(true);
    try {
      // Simulate loading analytics data
      await new Promise((resolve) => setTimeout(resolve, 500));

      const now = Date.now();
      const period = {
        start: now - getTimeRangeMs(timeRange),
        end: now,
      };

      const mockAnalytics: APIAnalyticsData = {
        period,
        overall: {
          totalRequests: 125847,
          successRate: 99.2,
          averageResponseTime: 142,
          p50ResponseTime: 98,
          p95ResponseTime: 287,
          p99ResponseTime: 521,
          errorRate: 0.8,
          requestsPerSecond: 87.3,
          bandwidth: 1024 * 1024 * 45.6,
          timestamp: now,
        },
        byEndpoint: [
          {
            endpointId: '1',
            path: '/api/v1/users',
            method: 'GET',
            metrics: {
              totalRequests: 45234,
              successRate: 99.8,
              averageResponseTime: 98,
              p50ResponseTime: 87,
              p95ResponseTime: 156,
              p99ResponseTime: 234,
              errorRate: 0.2,
              requestsPerSecond: 31.4,
              bandwidth: 1024 * 1024 * 15.2,
              timestamp: now,
            },
            statusCodes: { 200: 45143, 404: 91 },
            errors: [],
            topUsers: [
              { userId: 'user1', count: 2341 },
              { userId: 'user2', count: 1876 },
            ],
          },
          {
            endpointId: '2',
            path: '/api/v1/products',
            method: 'GET',
            metrics: {
              totalRequests: 38721,
              successRate: 98.9,
              averageResponseTime: 203,
              p50ResponseTime: 176,
              p95ResponseTime: 421,
              p99ResponseTime: 687,
              errorRate: 1.1,
              requestsPerSecond: 26.9,
              bandwidth: 1024 * 1024 * 19.4,
              timestamp: now,
            },
            statusCodes: { 200: 38295, 400: 234, 500: 192 },
            errors: [
              {
                statusCode: 500,
                message: 'Database connection timeout',
                count: 192,
                lastOccurrence: now - 3600000,
              },
            ],
            topUsers: [
              { userId: 'user3', count: 1987 },
              { userId: 'user1', count: 1654 },
            ],
          },
        ],
        byUser: {},
        byRegion: {
          'us-east-1': {
            totalRequests: 67234,
            successRate: 99.5,
            averageResponseTime: 123,
            p50ResponseTime: 98,
            p95ResponseTime: 234,
            p99ResponseTime: 421,
            errorRate: 0.5,
            requestsPerSecond: 46.7,
            bandwidth: 1024 * 1024 * 24.3,
            timestamp: now,
          },
          'eu-west-1': {
            totalRequests: 58613,
            successRate: 98.8,
            averageResponseTime: 167,
            p50ResponseTime: 143,
            p95ResponseTime: 321,
            p99ResponseTime: 587,
            errorRate: 1.2,
            requestsPerSecond: 40.6,
            bandwidth: 1024 * 1024 * 21.3,
            timestamp: now,
          },
        },
        timeSeries: generateTimeSeries(period.start, period.end),
        topErrors: [
          {
            statusCode: 500,
            message: 'Database connection timeout',
            count: 192,
            lastOccurrence: now - 3600000,
          },
          {
            statusCode: 429,
            message: 'Rate limit exceeded',
            count: 87,
            lastOccurrence: now - 1800000,
          },
        ],
      };

      setAnalytics(mockAnalytics);
    } catch (error) {
      console.error('Failed to load analytics:', error);
    } finally {
      setIsLoading(false);
    }
  };

  if (isLoading || !analytics) {
    return (
      <div className="flex items-center justify-center h-screen bg-gray-50 dark:bg-gray-900">
        <div className="text-center">
          <div className="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
          <p className="mt-4 text-gray-600 dark:text-gray-400">Loading analytics...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">API Analytics</h1>

          <div className="flex items-center space-x-2 bg-white dark:bg-gray-800 rounded-lg p-1 border border-gray-200 dark:border-gray-700">
            {(['1h', '24h', '7d', '30d'] as const).map((range) => (
              <button
                key={range}
                onClick={() => setTimeRange(range)}
                className={`px-3 py-1 rounded ${
                  timeRange === range
                    ? 'bg-blue-600 text-white'
                    : 'text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700'
                }`}
              >
                {range}
              </button>
            ))}
          </div>
        </div>

        {/* Overview Stats */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          <MetricCard
            title="Total Requests"
            value={formatNumber(analytics.overall.totalRequests)}
            icon="📊"
            color="blue"
          />
          <MetricCard
            title="Success Rate"
            value={`${analytics.overall.successRate}%`}
            icon="✅"
            color="green"
          />
          <MetricCard
            title="Avg Response Time"
            value={`${analytics.overall.averageResponseTime}ms`}
            icon="⚡"
            color="yellow"
          />
          <MetricCard
            title="Requests/sec"
            value={analytics.overall.requestsPerSecond.toFixed(1)}
            icon="🚀"
            color="purple"
          />
        </div>

        {/* Time Series Chart */}
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6 mb-8">
          <div className="flex items-center justify-between mb-6">
            <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
              Request Volume
            </h2>
            <div className="flex items-center space-x-2">
              {(['requests', 'errors', 'latency'] as const).map((metric) => (
                <button
                  key={metric}
                  onClick={() => setSelectedMetric(metric)}
                  className={`px-3 py-1 rounded text-sm ${
                    selectedMetric === metric
                      ? 'bg-blue-600 text-white'
                      : 'text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700'
                  }`}
                >
                  {metric.charAt(0).toUpperCase() + metric.slice(1)}
                </button>
              ))}
            </div>
          </div>

          <TimeSeriesChart
            data={analytics.timeSeries[selectedMetric] || []}
            metric={selectedMetric}
          />
        </div>

        {/* Endpoint Performance */}
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 mb-8">
          <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
            <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
              Endpoint Performance
            </h2>
          </div>
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
              <thead className="bg-gray-50 dark:bg-gray-700">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    Endpoint
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    Requests
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    Success Rate
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    Avg Time
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    P95
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    Errors
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                {analytics.byEndpoint.map((endpoint) => (
                  <tr key={endpoint.endpointId} className="hover:bg-gray-50 dark:hover:bg-gray-700/50">
                    <td className="px-6 py-4">
                      <div className="flex items-center space-x-2">
                        <span className={`px-2 py-1 rounded text-xs font-semibold ${getMethodColor(endpoint.method)}`}>
                          {endpoint.method}
                        </span>
                        <code className="text-sm font-mono text-gray-900 dark:text-white">
                          {endpoint.path}
                        </code>
                      </div>
                    </td>
                    <td className="px-6 py-4 text-sm text-gray-900 dark:text-white">
                      {formatNumber(endpoint.metrics.totalRequests)}
                    </td>
                    <td className="px-6 py-4 text-sm">
                      <span className={getSuccessRateColor(endpoint.metrics.successRate)}>
                        {endpoint.metrics.successRate}%
                      </span>
                    </td>
                    <td className="px-6 py-4 text-sm text-gray-900 dark:text-white">
                      {endpoint.metrics.averageResponseTime}ms
                    </td>
                    <td className="px-6 py-4 text-sm text-gray-900 dark:text-white">
                      {endpoint.metrics.p95ResponseTime}ms
                    </td>
                    <td className="px-6 py-4 text-sm">
                      {endpoint.errors.length > 0 ? (
                        <span className="text-red-600 dark:text-red-400">
                          {endpoint.errors.reduce((sum, e) => sum + e.count, 0)}
                        </span>
                      ) : (
                        <span className="text-gray-400">0</span>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>

        {/* Regional Distribution and Top Errors */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* Regional Distribution */}
          <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
            <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
              <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
                Regional Distribution
              </h2>
            </div>
            <div className="p-6 space-y-4">
              {Object.entries(analytics.byRegion).map(([region, metrics]) => (
                <div key={region}>
                  <div className="flex items-center justify-between mb-2">
                    <span className="text-sm font-medium text-gray-900 dark:text-white">
                      {region}
                    </span>
                    <span className="text-sm text-gray-600 dark:text-gray-400">
                      {formatNumber(metrics.totalRequests)} requests
                    </span>
                  </div>
                  <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                    <div
                      className="bg-blue-600 h-2 rounded-full"
                      style={{
                        width: `${(metrics.totalRequests / analytics.overall.totalRequests) * 100}%`,
                      }}
                    />
                  </div>
                  <div className="flex items-center justify-between mt-1 text-xs text-gray-500 dark:text-gray-400">
                    <span>{metrics.averageResponseTime}ms avg</span>
                    <span>{metrics.successRate}% success</span>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Top Errors */}
          <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
            <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
              <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
                Top Errors
              </h2>
            </div>
            <div className="p-6 space-y-4">
              {analytics.topErrors.map((error, index) => (
                <div key={index} className="border border-red-200 dark:border-red-800 rounded-lg p-4">
                  <div className="flex items-start justify-between mb-2">
                    <div className="flex items-center space-x-2">
                      <span className="px-2 py-1 rounded text-xs font-semibold bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200">
                        {error.statusCode}
                      </span>
                      <span className="text-sm font-medium text-gray-900 dark:text-white">
                        {error.message}
                      </span>
                    </div>
                    <span className="text-sm text-red-600 dark:text-red-400 font-semibold">
                      {error.count}
                    </span>
                  </div>
                  <div className="text-xs text-gray-500 dark:text-gray-400">
                    Last occurred {formatTimeAgo(error.lastOccurrence)}
                  </div>
                </div>
              ))}

              {analytics.topErrors.length === 0 && (
                <div className="text-center py-8 text-gray-500 dark:text-gray-400">
                  No errors in this time period
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

// Helper Components

interface MetricCardProps {
  title: string;
  value: string;
  icon: string;
  color: string;
}

const MetricCard: React.FC<MetricCardProps> = ({ title, value, icon, color }) => {
  const colors: Record<string, string> = {
    blue: 'bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400',
    green: 'bg-green-50 dark:bg-green-900/20 text-green-600 dark:text-green-400',
    yellow: 'bg-yellow-50 dark:bg-yellow-900/20 text-yellow-600 dark:text-yellow-400',
    purple: 'bg-purple-50 dark:bg-purple-900/20 text-purple-600 dark:text-purple-400',
  };

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm text-gray-600 dark:text-gray-400">{title}</p>
          <p className="mt-1 text-2xl font-semibold text-gray-900 dark:text-white">{value}</p>
        </div>
        <div className={`p-3 rounded-lg ${colors[color] || colors.blue}`}>
          <span className="text-2xl">{icon}</span>
        </div>
      </div>
    </div>
  );
};

interface TimeSeriesChartProps {
  data: TimeSeriesData[];
  metric: string;
}

const TimeSeriesChart: React.FC<TimeSeriesChartProps> = ({ data, metric }) => {
  if (data.length === 0) {
    return (
      <div className="text-center py-12 text-gray-500 dark:text-gray-400">
        No data available
      </div>
    );
  }

  const maxValue = Math.max(...data.map((d) => d.value));
  const minValue = Math.min(...data.map((d) => d.value));
  const range = maxValue - minValue;

  return (
    <div className="h-64">
      <svg className="w-full h-full">
        <polyline
          points={data
            .map((point, index) => {
              const x = (index / (data.length - 1)) * 100;
              const y = 100 - ((point.value - minValue) / range) * 90;
              return `${x}%,${y}%`;
            })
            .join(' ')}
          fill="none"
          stroke="rgb(59, 130, 246)"
          strokeWidth="2"
          vectorEffect="non-scaling-stroke"
        />
      </svg>
    </div>
  );
};

// Helper Functions

function getTimeRangeMs(range: '1h' | '24h' | '7d' | '30d'): number {
  const ranges = {
    '1h': 3600000,
    '24h': 86400000,
    '7d': 604800000,
    '30d': 2592000000,
  };
  return ranges[range];
}

function formatNumber(num: number): string {
  if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`;
  if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
  return num.toString();
}

function formatTimeAgo(timestamp: number): string {
  const seconds = Math.floor((Date.now() - timestamp) / 1000);

  if (seconds < 60) return 'just now';
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
  return `${Math.floor(seconds / 86400)}d ago`;
}

function getMethodColor(method: HTTPMethod): string {
  const colors: Record<HTTPMethod, string> = {
    GET: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
    POST: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
    PUT: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200',
    PATCH: 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200',
    DELETE: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
    HEAD: 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200',
    OPTIONS: 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200',
  };
  return colors[method];
}

function getSuccessRateColor(rate: number): string {
  if (rate >= 99) return 'text-green-600 dark:text-green-400 font-semibold';
  if (rate >= 95) return 'text-yellow-600 dark:text-yellow-400 font-semibold';
  return 'text-red-600 dark:text-red-400 font-semibold';
}

function generateTimeSeries(start: number, end: number): Record<string, TimeSeriesData[]> {
  const points = 24;
  const interval = (end - start) / points;

  const requests: TimeSeriesData[] = [];
  const errors: TimeSeriesData[] = [];
  const latency: TimeSeriesData[] = [];

  for (let i = 0; i <= points; i++) {
    const timestamp = start + i * interval;
    requests.push({ timestamp, value: Math.random() * 100 + 50 });
    errors.push({ timestamp, value: Math.random() * 10 });
    latency.push({ timestamp, value: Math.random() * 200 + 100 });
  }

  return { requests, errors, latency };
}

export default APIAnalytics;
