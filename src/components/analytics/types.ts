/**
 * Analytics Type Definitions
 *
 * TypeScript type definitions for the analytics dashboard.
 */

export enum MetricType {
  Counter = 'Counter',
  Gauge = 'Gauge',
  Histogram = 'Histogram',
  Summary = 'Summary',
}

export enum EventType {
  AppStart = 'AppStart',
  AppStop = 'AppStop',
  FeatureUsed = 'FeatureUsed',
  CommandExecuted = 'CommandExecuted',
  FileOpened = 'FileOpened',
  FileSaved = 'FileSaved',
  EntityCreated = 'EntityCreated',
  EntityModified = 'EntityModified',
  EntityDeleted = 'EntityDeleted',
  ToolActivated = 'ToolActivated',
  LayerCreated = 'LayerCreated',
  ViewChanged = 'ViewChanged',
  RenderingModeChanged = 'RenderingModeChanged',
  Export = 'Export',
  Import = 'Import',
  Error = 'Error',
  Custom = 'Custom',
}

export enum AggregationWindow {
  Second = 'Second',
  Minute = 'Minute',
  FiveMinute = 'FiveMinute',
  FifteenMinute = 'FifteenMinute',
  Hour = 'Hour',
  Day = 'Day',
  Week = 'Week',
}

export enum ExportFormat {
  Prometheus = 'Prometheus',
  OpenTelemetry = 'OpenTelemetry',
  Json = 'Json',
  Csv = 'Csv',
  Binary = 'Binary',
}

export enum ReportFormat {
  Html = 'Html',
  Pdf = 'Pdf',
  Markdown = 'Markdown',
  Json = 'Json',
  Csv = 'Csv',
  Text = 'Text',
}

export enum ReportType {
  Usage = 'Usage',
  Performance = 'Performance',
  Errors = 'Errors',
  Custom = 'Custom',
  ExecutiveSummary = 'ExecutiveSummary',
  DetailedAnalytics = 'DetailedAnalytics',
}

export interface MetricValue {
  type: 'Int' | 'Float' | 'Bool' | 'Histogram' | 'Summary';
  value: number | boolean | {
    buckets?: number[];
    counts?: number[];
    count?: number;
    sum?: number;
    min?: number;
    max?: number;
    percentiles?: Record<string, number>;
  };
}

export interface Metric {
  name: string;
  metric_type: MetricType;
  value: MetricValue;
  labels: Record<string, string>;
  timestamp: string;
  description?: string;
}

export interface TimeSeriesPoint {
  timestamp: string;
  value: number;
  labels: Record<string, string>;
  metadata?: Record<string, any>;
}

export interface AggregationStats {
  count: number;
  sum: number;
  min: number;
  max: number;
  avg: number;
  median: number;
  p95: number;
  p99: number;
  stddev: number;
  first: number;
  last: number;
}

export interface AggregatedMetric {
  name: string;
  window: AggregationWindow;
  timestamp: string;
  stats: AggregationStats;
  labels: Record<string, string>;
}

export interface UsageEvent {
  id: string;
  event_type: EventType;
  name: string;
  timestamp: string;
  user_id?: string;
  session_id: string;
  properties: Record<string, any>;
  duration_ms?: number;
}

export interface UsageStats {
  total_events: number;
  events_by_type: Record<EventType, number>;
  active_users: number;
  active_sessions: number;
  total_session_duration_secs: number;
  avg_session_duration_secs: number;
  most_used_features: [string, number][];
  most_executed_commands: [string, number][];
  error_rate: number;
  first_event?: string;
  last_event?: string;
}

export interface ProfileSpan {
  id: string;
  parent_id?: string;
  name: string;
  start_time: string;
  end_time?: string;
  duration_us?: number;
  tags: Record<string, string>;
  metrics: Record<string, number>;
  error?: string;
}

export interface ProfileReport {
  operation_name: string;
  total_calls: number;
  total_duration_ms: number;
  avg_duration_ms: number;
  min_duration_ms: number;
  max_duration_ms: number;
  error_rate: number;
  recent_spans: ProfileSpan[];
}

export interface FlameNode {
  name: string;
  duration_ms: number;
  children: FlameNode[];
}

export interface FlameGraph {
  roots: FlameNode[];
}

export interface HealthStatus {
  metrics_collected: number;
  storage_size_bytes: number;
  uptime_seconds: number;
  active_profiles: number;
  last_export?: string;
}

export interface ReportSection {
  title: string;
  content: string;
  data?: any;
  section_type: 'Text' | 'Chart' | 'Table' | 'KeyValue' | 'List';
  subsections: ReportSection[];
}

export interface Report {
  id: string;
  report_type: ReportType;
  title: string;
  description?: string;
  generated_at: string;
  time_range?: [string, string];
  sections: ReportSection[];
  metadata: Record<string, string>;
}

export interface ExportEndpoint {
  name: string;
  url: string;
  format: ExportFormat;
  interval_secs: number;
  auth_token?: string;
}

export interface AnalyticsConfig {
  enabled: boolean;
  collection_interval_secs: number;
  aggregation_window_secs: number;
  retention_days: number;
  max_storage_bytes: number;
  enable_profiling: boolean;
  enable_usage_tracking: boolean;
  export_endpoints: ExportEndpoint[];
  storage_path: string;
}

export interface TimeRange {
  start: Date;
  end: Date;
}

export interface ChartDataPoint {
  x: number | Date;
  y: number;
  label?: string;
}

export interface ChartSeries {
  name: string;
  data: ChartDataPoint[];
  color?: string;
}

export interface DashboardMetric {
  id: string;
  name: string;
  value: number;
  unit?: string;
  trend?: 'up' | 'down' | 'stable';
  change?: number;
  chartData?: ChartSeries[];
}

export interface FilterOptions {
  timeRange: TimeRange;
  metricNames?: string[];
  labels?: Record<string, string>;
  eventTypes?: EventType[];
  aggregationWindow?: AggregationWindow;
}
