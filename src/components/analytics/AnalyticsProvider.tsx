/**
 * Analytics Provider
 *
 * React context provider for analytics functionality.
 */

import React, { createContext, useState, useEffect, useCallback, ReactNode } from 'react';
import type {
  AnalyticsConfig,
  HealthStatus,
  UsageStats,
  ProfileReport,
  TimeRange,
  Metric,
} from './types';

interface AnalyticsContextValue {
  config: AnalyticsConfig | null;
  health: HealthStatus | null;
  enabled: boolean;
  loading: boolean;
  error: Error | null;

  // Configuration methods
  updateConfig: (config: Partial<AnalyticsConfig>) => Promise<void>;
  enableAnalytics: () => Promise<void>;
  disableAnalytics: () => Promise<void>;

  // Data fetching methods
  fetchHealth: () => Promise<void>;
  fetchMetrics: (metricName: string, timeRange: TimeRange) => Promise<any>;

  // Tracking methods
  recordMetric: (metric: Metric) => Promise<void>;
  clearData: () => Promise<void>;
}

export const AnalyticsContext = createContext<AnalyticsContextValue | null>(null);

interface AnalyticsProviderProps {
  children: ReactNode;
  initialConfig?: Partial<AnalyticsConfig>;
}

export function AnalyticsProvider({ children, initialConfig }: AnalyticsProviderProps) {
  const [config, setConfig] = useState<AnalyticsConfig | null>(null);
  const [health, setHealth] = useState<HealthStatus | null>(null);
  const [enabled, setEnabled] = useState(true);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  // Initialize analytics
  useEffect(() => {
    const initialize = async () => {
      try {
        setLoading(true);

        // Fetch current configuration
        const response = await fetch('/api/analytics/config');
        if (response.ok) {
          const currentConfig = await response.json();
          setConfig({ ...currentConfig, ...initialConfig });
          setEnabled(currentConfig.enabled);
        } else {
          // Use default configuration
          const defaultConfig: AnalyticsConfig = {
            enabled: true,
            collection_interval_secs: 10,
            aggregation_window_secs: 60,
            retention_days: 30,
            max_storage_bytes: 10 * 1024 * 1024 * 1024,
            enable_profiling: true,
            enable_usage_tracking: true,
            export_endpoints: [],
            storage_path: './analytics_data',
            ...initialConfig,
          };
          setConfig(defaultConfig);
          setEnabled(defaultConfig.enabled);
        }

        // Fetch initial health status
        await fetchHealth();

        setError(null);
      } catch (err) {
        setError(err as Error);
        console.error('Failed to initialize analytics:', err);
      } finally {
        setLoading(false);
      }
    };

    initialize();
  }, [initialConfig]);

  // Update configuration
  const updateConfig = useCallback(async (updates: Partial<AnalyticsConfig>) => {
    try {
      const response = await fetch('/api/analytics/config', {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(updates),
      });

      if (!response.ok) {
        throw new Error('Failed to update configuration');
      }

      const updatedConfig = await response.json();
      setConfig(updatedConfig);
      setEnabled(updatedConfig.enabled);
    } catch (err) {
      setError(err as Error);
      throw err;
    }
  }, []);

  // Enable analytics
  const enableAnalytics = useCallback(async () => {
    await updateConfig({ enabled: true });
  }, [updateConfig]);

  // Disable analytics
  const disableAnalytics = useCallback(async () => {
    await updateConfig({ enabled: false });
  }, [updateConfig]);

  // Fetch health status
  const fetchHealth = useCallback(async () => {
    try {
      const response = await fetch('/api/analytics/health');
      if (response.ok) {
        const healthData = await response.json();
        setHealth(healthData);
      }
    } catch (err) {
      console.error('Failed to fetch health status:', err);
    }
  }, []);

  // Fetch metrics
  const fetchMetrics = useCallback(
    async (metricName: string, timeRange: TimeRange) => {
      try {
        const response = await fetch(
          `/api/analytics/metrics/${metricName}?start=${timeRange.start.toISOString()}&end=${timeRange.end.toISOString()}`
        );

        if (!response.ok) {
          throw new Error('Failed to fetch metrics');
        }

        return await response.json();
      } catch (err) {
        setError(err as Error);
        throw err;
      }
    },
    []
  );

  // Record a metric
  const recordMetric = useCallback(async (metric: Metric) => {
    try {
      const response = await fetch('/api/analytics/metrics', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(metric),
      });

      if (!response.ok) {
        throw new Error('Failed to record metric');
      }
    } catch (err) {
      console.error('Failed to record metric:', err);
    }
  }, []);

  // Clear all analytics data
  const clearData = useCallback(async () => {
    try {
      const response = await fetch('/api/analytics/clear', {
        method: 'POST',
      });

      if (!response.ok) {
        throw new Error('Failed to clear analytics data');
      }

      // Refresh health status
      await fetchHealth();
    } catch (err) {
      setError(err as Error);
      throw err;
    }
  }, [fetchHealth]);

  // Periodic health status updates
  useEffect(() => {
    if (!enabled) return;

    const interval = setInterval(() => {
      fetchHealth();
    }, 10000); // Update every 10 seconds

    return () => clearInterval(interval);
  }, [enabled, fetchHealth]);

  const value: AnalyticsContextValue = {
    config,
    health,
    enabled,
    loading,
    error,
    updateConfig,
    enableAnalytics,
    disableAnalytics,
    fetchHealth,
    fetchMetrics,
    recordMetric,
    clearData,
  };

  return (
    <AnalyticsContext.Provider value={value}>
      {children}
    </AnalyticsContext.Provider>
  );
}

/**
 * Higher-order component to wrap components with analytics
 */
export function withAnalytics<P extends object>(
  Component: React.ComponentType<P>
): React.ComponentType<P> {
  return function WrappedComponent(props: P) {
    return (
      <AnalyticsProvider>
        <Component {...props} />
      </AnalyticsProvider>
    );
  };
}

/**
 * Utility function to format bytes
 */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 Bytes';

  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}

/**
 * Utility function to format duration
 */
export function formatDuration(seconds: number): string {
  if (seconds < 60) {
    return `${seconds}s`;
  } else if (seconds < 3600) {
    const minutes = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return secs > 0 ? `${minutes}m ${secs}s` : `${minutes}m`;
  } else if (seconds < 86400) {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return minutes > 0 ? `${hours}h ${minutes}m` : `${hours}h`;
  } else {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    return hours > 0 ? `${days}d ${hours}h` : `${days}d`;
  }
}

/**
 * Utility function to format numbers with K, M, B suffixes
 */
export function formatNumber(num: number): string {
  if (num >= 1000000000) {
    return `${(num / 1000000000).toFixed(1)}B`;
  } else if (num >= 1000000) {
    return `${(num / 1000000).toFixed(1)}M`;
  } else if (num >= 1000) {
    return `${(num / 1000).toFixed(1)}K`;
  }
  return num.toString();
}

/**
 * Utility function to calculate percentage change
 */
export function calculateChange(current: number, previous: number): number {
  if (previous === 0) return 0;
  return ((current - previous) / previous) * 100;
}

/**
 * Utility function to determine trend direction
 */
export function getTrend(change: number): 'up' | 'down' | 'stable' {
  if (Math.abs(change) < 0.1) return 'stable';
  return change > 0 ? 'up' : 'down';
}
