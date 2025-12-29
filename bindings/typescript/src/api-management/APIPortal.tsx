/**
 * CADDY API Portal Dashboard
 *
 * Main API management portal dashboard with overview, quick actions,
 * and navigation to all API management features.
 */

import React, { useState, useEffect } from 'react';
import {
  APIEndpoint,
  APIMetrics,
  APIVersion,
  APIKey,
  RateLimitStatus,
  HTTPMethod,
} from './types';

interface APIPortalProps {
  projectId?: string;
  onNavigate?: (view: PortalView) => void;
  config?: PortalConfig;
}

interface PortalConfig {
  title?: string;
  logo?: string;
  enableAnalytics?: boolean;
  enableTesting?: boolean;
  enableMocking?: boolean;
  showQuickStart?: boolean;
}

type PortalView =
  | 'explorer'
  | 'documentation'
  | 'endpoints'
  | 'keys'
  | 'rate-limits'
  | 'analytics'
  | 'webhooks'
  | 'mocking'
  | 'versioning'
  | 'testing';

interface DashboardStats {
  totalEndpoints: number;
  activeKeys: number;
  todayRequests: number;
  avgResponseTime: number;
  successRate: number;
  activeWebhooks: number;
}

export const APIPortal: React.FC<APIPortalProps> = ({
  projectId = 'default',
  onNavigate,
  config = {},
}) => {
  const [activeView, setActiveView] = useState<PortalView>('explorer');
  const [stats, setStats] = useState<DashboardStats>({
    totalEndpoints: 0,
    activeKeys: 0,
    todayRequests: 0,
    avgResponseTime: 0,
    successRate: 0,
    activeWebhooks: 0,
  });
  const [recentActivity, setRecentActivity] = useState<ActivityItem[]>([]);
  const [popularEndpoints, setPopularEndpoints] = useState<PopularEndpoint[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState('');

  useEffect(() => {
    loadDashboardData();
  }, [projectId]);

  const loadDashboardData = async () => {
    setIsLoading(true);
    try {
      // Simulate loading dashboard data
      await new Promise((resolve) => setTimeout(resolve, 500));

      setStats({
        totalEndpoints: 47,
        activeKeys: 12,
        todayRequests: 15847,
        avgResponseTime: 142,
        successRate: 99.2,
        activeWebhooks: 5,
      });

      setRecentActivity([
        {
          id: '1',
          type: 'endpoint_created',
          message: 'New endpoint created: POST /api/v1/users',
          timestamp: Date.now() - 300000,
          user: 'admin@example.com',
        },
        {
          id: '2',
          type: 'key_generated',
          message: 'API key generated for production',
          timestamp: Date.now() - 600000,
          user: 'developer@example.com',
        },
        {
          id: '3',
          type: 'rate_limit_hit',
          message: 'Rate limit reached for /api/v1/search',
          timestamp: Date.now() - 900000,
          user: 'system',
        },
      ]);

      setPopularEndpoints([
        { path: '/api/v1/users', method: 'GET', requests: 5234, avgTime: 98 },
        { path: '/api/v1/auth/login', method: 'POST', requests: 3421, avgTime: 156 },
        { path: '/api/v1/products', method: 'GET', requests: 2987, avgTime: 203 },
        { path: '/api/v1/orders', method: 'POST', requests: 1876, avgTime: 287 },
        { path: '/api/v1/search', method: 'GET', requests: 1654, avgTime: 421 },
      ]);
    } catch (error) {
      console.error('Failed to load dashboard data:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleNavigate = (view: PortalView) => {
    setActiveView(view);
    onNavigate?.(view);
  };

  const formatNumber = (num: number): string => {
    if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`;
    if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
    return num.toString();
  };

  const formatDuration = (ms: number): string => {
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(2)}s`;
  };

  const getActivityIcon = (type: string): string => {
    const icons: Record<string, string> = {
      endpoint_created: '➕',
      endpoint_updated: '✏️',
      key_generated: '🔑',
      rate_limit_hit: '⚠️',
      webhook_triggered: '🔔',
      test_passed: '✅',
      test_failed: '❌',
    };
    return icons[type] || '📌';
  };

  const getMethodColor = (method: HTTPMethod): string => {
    const colors: Record<HTTPMethod, string> = {
      GET: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
      POST: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
      PUT: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200',
      PATCH: 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200',
      DELETE: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
      HEAD: 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200',
      OPTIONS: 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200',
    };
    return colors[method] || 'bg-gray-100 text-gray-800';
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-screen bg-gray-50 dark:bg-gray-900">
        <div className="text-center">
          <div className="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
          <p className="mt-4 text-gray-600 dark:text-gray-400">Loading API Portal...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      {/* Header */}
      <header className="bg-white dark:bg-gray-800 shadow-sm border-b border-gray-200 dark:border-gray-700">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-4">
              {config.logo && <img src={config.logo} alt="Logo" className="h-8 w-8" />}
              <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
                {config.title || 'CADDY API Portal'}
              </h1>
            </div>
            <div className="flex items-center space-x-4">
              <div className="relative">
                <input
                  type="text"
                  placeholder="Search endpoints, docs..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="w-64 px-4 py-2 pl-10 border border-gray-300 dark:border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                />
                <span className="absolute left-3 top-2.5 text-gray-400">🔍</span>
              </div>
              <button className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors">
                Generate API Key
              </button>
            </div>
          </div>
        </div>
      </header>

      {/* Navigation */}
      <nav className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex space-x-8 overflow-x-auto">
            {[
              { id: 'explorer', label: 'API Explorer', icon: '🔍' },
              { id: 'documentation', label: 'Documentation', icon: '📚' },
              { id: 'endpoints', label: 'Endpoints', icon: '🔗' },
              { id: 'keys', label: 'API Keys', icon: '🔑' },
              { id: 'rate-limits', label: 'Rate Limits', icon: '⚡' },
              { id: 'analytics', label: 'Analytics', icon: '📊' },
              { id: 'webhooks', label: 'Webhooks', icon: '🔔' },
              { id: 'mocking', label: 'Mock Server', icon: '🎭' },
              { id: 'versioning', label: 'Versions', icon: '📝' },
              { id: 'testing', label: 'Testing', icon: '🧪' },
            ].map((item) => (
              <button
                key={item.id}
                onClick={() => handleNavigate(item.id as PortalView)}
                className={`flex items-center space-x-2 px-4 py-3 border-b-2 transition-colors whitespace-nowrap ${
                  activeView === item.id
                    ? 'border-blue-600 text-blue-600 dark:text-blue-400'
                    : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'
                }`}
              >
                <span>{item.icon}</span>
                <span className="font-medium">{item.label}</span>
              </button>
            ))}
          </div>
        </div>
      </nav>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Stats Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6 gap-6 mb-8">
          <StatCard
            title="Endpoints"
            value={stats.totalEndpoints.toString()}
            icon="🔗"
            color="blue"
          />
          <StatCard
            title="Active Keys"
            value={stats.activeKeys.toString()}
            icon="🔑"
            color="green"
          />
          <StatCard
            title="Requests Today"
            value={formatNumber(stats.todayRequests)}
            icon="📊"
            color="purple"
          />
          <StatCard
            title="Avg Response"
            value={`${stats.avgResponseTime}ms`}
            icon="⚡"
            color="yellow"
          />
          <StatCard
            title="Success Rate"
            value={`${stats.successRate}%`}
            icon="✅"
            color="green"
          />
          <StatCard
            title="Webhooks"
            value={stats.activeWebhooks.toString()}
            icon="🔔"
            color="indigo"
          />
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          {/* Popular Endpoints */}
          <div className="lg:col-span-2 bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
            <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
              <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
                Popular Endpoints
              </h2>
            </div>
            <div className="divide-y divide-gray-200 dark:divide-gray-700">
              {popularEndpoints.map((endpoint, index) => (
                <div key={index} className="px-6 py-4 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-3 flex-1">
                      <span className={`px-2 py-1 rounded text-xs font-semibold ${getMethodColor(endpoint.method)}`}>
                        {endpoint.method}
                      </span>
                      <code className="text-sm text-gray-900 dark:text-gray-100 font-mono">
                        {endpoint.path}
                      </code>
                    </div>
                    <div className="flex items-center space-x-6 text-sm">
                      <div className="text-right">
                        <div className="text-gray-900 dark:text-white font-medium">
                          {formatNumber(endpoint.requests)}
                        </div>
                        <div className="text-gray-500 dark:text-gray-400 text-xs">requests</div>
                      </div>
                      <div className="text-right">
                        <div className="text-gray-900 dark:text-white font-medium">
                          {endpoint.avgTime}ms
                        </div>
                        <div className="text-gray-500 dark:text-gray-400 text-xs">avg time</div>
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Recent Activity */}
          <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
            <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
              <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
                Recent Activity
              </h2>
            </div>
            <div className="divide-y divide-gray-200 dark:divide-gray-700">
              {recentActivity.map((activity) => (
                <div key={activity.id} className="px-6 py-4">
                  <div className="flex items-start space-x-3">
                    <span className="text-2xl">{getActivityIcon(activity.type)}</span>
                    <div className="flex-1 min-w-0">
                      <p className="text-sm text-gray-900 dark:text-gray-100">
                        {activity.message}
                      </p>
                      <div className="mt-1 flex items-center space-x-2 text-xs text-gray-500 dark:text-gray-400">
                        <span>{activity.user}</span>
                        <span>•</span>
                        <span>{formatTimeAgo(activity.timestamp)}</span>
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Quick Start Guide */}
        {config.showQuickStart !== false && (
          <div className="mt-8 bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20 rounded-lg border border-blue-200 dark:border-blue-800 p-6">
            <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
              Quick Start Guide
            </h2>
            <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
              <QuickStartCard
                step={1}
                title="Explore APIs"
                description="Browse and test endpoints"
                action="Open Explorer"
                onClick={() => handleNavigate('explorer')}
              />
              <QuickStartCard
                step={2}
                title="Generate Key"
                description="Create an API key"
                action="Create Key"
                onClick={() => handleNavigate('keys')}
              />
              <QuickStartCard
                step={3}
                title="View Docs"
                description="Read API documentation"
                action="View Docs"
                onClick={() => handleNavigate('documentation')}
              />
              <QuickStartCard
                step={4}
                title="Test APIs"
                description="Run automated tests"
                action="Start Testing"
                onClick={() => handleNavigate('testing')}
              />
            </div>
          </div>
        )}
      </main>
    </div>
  );
};

// Helper Components

interface StatCardProps {
  title: string;
  value: string;
  icon: string;
  color: string;
}

const StatCard: React.FC<StatCardProps> = ({ title, value, icon, color }) => {
  const colors: Record<string, string> = {
    blue: 'bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400',
    green: 'bg-green-50 dark:bg-green-900/20 text-green-600 dark:text-green-400',
    purple: 'bg-purple-50 dark:bg-purple-900/20 text-purple-600 dark:text-purple-400',
    yellow: 'bg-yellow-50 dark:bg-yellow-900/20 text-yellow-600 dark:text-yellow-400',
    indigo: 'bg-indigo-50 dark:bg-indigo-900/20 text-indigo-600 dark:text-indigo-400',
  };

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-4">
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

interface QuickStartCardProps {
  step: number;
  title: string;
  description: string;
  action: string;
  onClick: () => void;
}

const QuickStartCard: React.FC<QuickStartCardProps> = ({
  step,
  title,
  description,
  action,
  onClick,
}) => {
  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
      <div className="flex items-center space-x-3 mb-2">
        <div className="flex items-center justify-center w-6 h-6 rounded-full bg-blue-600 text-white text-xs font-bold">
          {step}
        </div>
        <h3 className="font-semibold text-gray-900 dark:text-white">{title}</h3>
      </div>
      <p className="text-sm text-gray-600 dark:text-gray-400 mb-3">{description}</p>
      <button
        onClick={onClick}
        className="w-full px-3 py-2 text-sm bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 rounded hover:bg-blue-100 dark:hover:bg-blue-900/30 transition-colors"
      >
        {action}
      </button>
    </div>
  );
};

// Helper Interfaces

interface ActivityItem {
  id: string;
  type: string;
  message: string;
  timestamp: number;
  user: string;
}

interface PopularEndpoint {
  path: string;
  method: HTTPMethod;
  requests: number;
  avgTime: number;
}

// Helper Functions

function formatTimeAgo(timestamp: number): string {
  const seconds = Math.floor((Date.now() - timestamp) / 1000);

  if (seconds < 60) return 'just now';
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
  return `${Math.floor(seconds / 86400)}d ago`;
}

export default APIPortal;
