/**
 * Analytics Hooks
 *
 * React hooks for accessing analytics data and functionality.
 */

import { useState, useEffect, useCallback, useContext } from 'react';
import { AnalyticsContext } from './AnalyticsProvider';
import type {
  TimeSeriesPoint,
  AggregatedMetric,
  UsageStats,
  ProfileReport,
  HealthStatus,
  Report,
  TimeRange,
  FilterOptions,
  AggregationWindow,
  EventType,
  UsageEvent,
} from './types';

/**
 * Main analytics hook
 */
export function useAnalytics() {
  const context = useContext(AnalyticsContext);

  if (!context) {
    throw new Error('useAnalytics must be used within an AnalyticsProvider');
  }

  return context;
}

/**
 * Hook for fetching time-series data
 */
export function useTimeSeriesData(
  metricName: string,
  timeRange: TimeRange,
  refreshInterval?: number
) {
  const [data, setData] = useState<TimeSeriesPoint[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchData = useCallback(async () => {
    try {
      setLoading(true);
      // In a real implementation, this would call a Rust backend via Tauri or API
      const response = await fetch(
        `/api/analytics/metrics/${metricName}?start=${timeRange.start.toISOString()}&end=${timeRange.end.toISOString()}`
      );

      if (!response.ok) {
        throw new Error('Failed to fetch time series data');
      }

      const result = await response.json();
      setData(result);
      setError(null);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  }, [metricName, timeRange]);

  useEffect(() => {
    fetchData();

    if (refreshInterval) {
      const interval = setInterval(fetchData, refreshInterval);
      return () => clearInterval(interval);
    }
  }, [fetchData, refreshInterval]);

  return { data, loading, error, refetch: fetchData };
}

/**
 * Hook for fetching aggregated metrics
 */
export function useAggregatedMetrics(
  metricName: string,
  window: AggregationWindow,
  timeRange: TimeRange
) {
  const [data, setData] = useState<AggregatedMetric[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setLoading(true);
        const response = await fetch(
          `/api/analytics/aggregated/${metricName}?window=${window}&start=${timeRange.start.toISOString()}&end=${timeRange.end.toISOString()}`
        );

        if (!response.ok) {
          throw new Error('Failed to fetch aggregated metrics');
        }

        const result = await response.json();
        setData(result);
        setError(null);
      } catch (err) {
        setError(err as Error);
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, [metricName, window, timeRange]);

  return { data, loading, error };
}

/**
 * Hook for usage statistics
 */
export function useUsageStats(refreshInterval: number = 5000) {
  const [stats, setStats] = useState<UsageStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchStats = useCallback(async () => {
    try {
      setLoading(true);
      const response = await fetch('/api/analytics/usage/stats');

      if (!response.ok) {
        throw new Error('Failed to fetch usage stats');
      }

      const result = await response.json();
      setStats(result);
      setError(null);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchStats();

    const interval = setInterval(fetchStats, refreshInterval);
    return () => clearInterval(interval);
  }, [fetchStats, refreshInterval]);

  return { stats, loading, error, refetch: fetchStats };
}

/**
 * Hook for performance profiles
 */
export function usePerformanceProfiles(limit: number = 10) {
  const [profiles, setProfiles] = useState<ProfileReport[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchProfiles = useCallback(async () => {
    try {
      setLoading(true);
      const response = await fetch(`/api/analytics/performance/profiles?limit=${limit}`);

      if (!response.ok) {
        throw new Error('Failed to fetch performance profiles');
      }

      const result = await response.json();
      setProfiles(result);
      setError(null);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  }, [limit]);

  useEffect(() => {
    fetchProfiles();
  }, [fetchProfiles]);

  return { profiles, loading, error, refetch: fetchProfiles };
}

/**
 * Hook for system health status
 */
export function useHealthStatus(refreshInterval: number = 10000) {
  const [health, setHealth] = useState<HealthStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchHealth = useCallback(async () => {
    try {
      const response = await fetch('/api/analytics/health');

      if (!response.ok) {
        throw new Error('Failed to fetch health status');
      }

      const result = await response.json();
      setHealth(result);
      setError(null);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchHealth();

    const interval = setInterval(fetchHealth, refreshInterval);
    return () => clearInterval(interval);
  }, [fetchHealth, refreshInterval]);

  return { health, loading, error, refetch: fetchHealth };
}

/**
 * Hook for tracking events
 */
export function useEventTracking() {
  const trackEvent = useCallback(async (event: Omit<UsageEvent, 'id' | 'timestamp'>) => {
    try {
      const response = await fetch('/api/analytics/usage/track', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(event),
      });

      if (!response.ok) {
        throw new Error('Failed to track event');
      }
    } catch (err) {
      console.error('Event tracking failed:', err);
    }
  }, []);

  const trackFeature = useCallback(
    (featureName: string, properties?: Record<string, any>) => {
      trackEvent({
        event_type: EventType.FeatureUsed,
        name: featureName,
        session_id: '', // Will be filled by backend
        properties: properties || {},
      });
    },
    [trackEvent]
  );

  const trackCommand = useCallback(
    (commandName: string, duration_ms: number, success: boolean) => {
      trackEvent({
        event_type: EventType.CommandExecuted,
        name: commandName,
        session_id: '',
        properties: { success },
        duration_ms,
      });
    },
    [trackEvent]
  );

  const trackError = useCallback(
    (errorMessage: string, context?: Record<string, any>) => {
      trackEvent({
        event_type: EventType.Error,
        name: 'error',
        session_id: '',
        properties: {
          message: errorMessage,
          ...context,
        },
      });
    },
    [trackEvent]
  );

  return { trackEvent, trackFeature, trackCommand, trackError };
}

/**
 * Hook for generating reports
 */
export function useReportGenerator() {
  const [generating, setGenerating] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const generateReport = useCallback(
    async (
      reportType: string,
      format: string,
      options: Record<string, any> = {}
    ): Promise<Report | null> => {
      try {
        setGenerating(true);
        setError(null);

        const response = await fetch('/api/analytics/reports/generate', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            report_type: reportType,
            format,
            options,
          }),
        });

        if (!response.ok) {
          throw new Error('Failed to generate report');
        }

        const report = await response.json();
        return report;
      } catch (err) {
        setError(err as Error);
        return null;
      } finally {
        setGenerating(false);
      }
    },
    []
  );

  const downloadReport = useCallback(
    async (report: Report, format: string) => {
      try {
        const response = await fetch('/api/analytics/reports/render', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            report,
            format,
          }),
        });

        if (!response.ok) {
          throw new Error('Failed to render report');
        }

        const blob = await response.blob();
        const url = window.URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `${report.title.replace(/\s+/g, '_')}.${format}`;
        document.body.appendChild(a);
        a.click();
        window.URL.revokeObjectURL(url);
        document.body.removeChild(a);
      } catch (err) {
        setError(err as Error);
      }
    },
    []
  );

  return { generateReport, downloadReport, generating, error };
}

/**
 * Hook for filtering and searching metrics
 */
export function useMetricFilter() {
  const [filters, setFilters] = useState<FilterOptions>({
    timeRange: {
      start: new Date(Date.now() - 24 * 60 * 60 * 1000), // Last 24 hours
      end: new Date(),
    },
  });

  const updateTimeRange = useCallback((start: Date, end: Date) => {
    setFilters((prev) => ({
      ...prev,
      timeRange: { start, end },
    }));
  }, []);

  const updateMetricNames = useCallback((names: string[]) => {
    setFilters((prev) => ({
      ...prev,
      metricNames: names,
    }));
  }, []);

  const updateLabels = useCallback((labels: Record<string, string>) => {
    setFilters((prev) => ({
      ...prev,
      labels,
    }));
  }, []);

  const updateEventTypes = useCallback((types: EventType[]) => {
    setFilters((prev) => ({
      ...prev,
      eventTypes: types,
    }));
  }, []);

  const updateAggregationWindow = useCallback((window: AggregationWindow) => {
    setFilters((prev) => ({
      ...prev,
      aggregationWindow: window,
    }));
  }, []);

  const resetFilters = useCallback(() => {
    setFilters({
      timeRange: {
        start: new Date(Date.now() - 24 * 60 * 60 * 1000),
        end: new Date(),
      },
    });
  }, []);

  return {
    filters,
    updateTimeRange,
    updateMetricNames,
    updateLabels,
    updateEventTypes,
    updateAggregationWindow,
    resetFilters,
  };
}

/**
 * Hook for real-time metrics streaming
 */
export function useMetricStream(metricNames: string[]) {
  const [metrics, setMetrics] = useState<Map<string, TimeSeriesPoint[]>>(new Map());
  const [connected, setConnected] = useState(false);

  useEffect(() => {
    // In a real implementation, this would use WebSocket or Server-Sent Events
    const ws = new WebSocket('ws://localhost:8080/analytics/stream');

    ws.onopen = () => {
      setConnected(true);
      // Subscribe to metrics
      ws.send(JSON.stringify({ type: 'subscribe', metrics: metricNames }));
    };

    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      if (data.type === 'metric') {
        setMetrics((prev) => {
          const updated = new Map(prev);
          const points = updated.get(data.metric_name) || [];
          points.push(data.point);
          // Keep only last 1000 points
          if (points.length > 1000) {
            points.shift();
          }
          updated.set(data.metric_name, points);
          return updated;
        });
      }
    };

    ws.onerror = () => {
      setConnected(false);
    };

    ws.onclose = () => {
      setConnected(false);
    };

    return () => {
      ws.close();
    };
  }, [metricNames]);

  return { metrics, connected };
}
