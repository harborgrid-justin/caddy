/**
 * Analytics Module Exports
 *
 * Central export point for all analytics components.
 */

// Context and Hooks
export { AnalyticsProvider, withAnalytics, formatBytes, formatDuration, formatNumber, calculateChange, getTrend } from './AnalyticsProvider';
export {
  useAnalytics,
  useTimeSeriesData,
  useAggregatedMetrics,
  useUsageStats,
  usePerformanceProfiles,
  useHealthStatus,
  useEventTracking,
  useReportGenerator,
  useMetricFilter,
  useMetricStream,
} from './useAnalytics';

// Components
export { Dashboard } from './Dashboard';
export { MetricsChart, MultiSeriesChart } from './MetricsChart';
export { PerformancePanel } from './PerformancePanel';
export { UsageStats, UserEngagementScore } from './UsageStats';
export { ReportBuilder, ScheduledReports } from './ReportBuilder';
export { ExportPanel, ExportTemplates } from './ExportPanel';

// Types
export type {
  MetricType,
  EventType,
  AggregationWindow,
  ExportFormat,
  ReportFormat,
  ReportType,
  MetricValue,
  Metric,
  TimeSeriesPoint,
  AggregationStats,
  AggregatedMetric,
  UsageEvent,
  UsageStats,
  ProfileSpan,
  ProfileReport,
  FlameNode,
  FlameGraph,
  HealthStatus,
  ReportSection,
  Report,
  ExportEndpoint,
  AnalyticsConfig,
  TimeRange,
  ChartDataPoint,
  ChartSeries,
  DashboardMetric,
  FilterOptions,
} from './types';

export {
  MetricType,
  EventType,
  AggregationWindow,
  ExportFormat,
  ReportFormat,
  ReportType,
} from './types';
