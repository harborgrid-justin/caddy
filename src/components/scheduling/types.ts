/**
 * TypeScript types for CADDY Scheduling and Monitoring System
 * Mirrors the Rust backend types for type safety
 */

// ============================================================================
// Scheduler Types
// ============================================================================

export enum JobPriority {
  Low = 0,
  Normal = 1,
  High = 2,
  Critical = 3,
}

export enum JobStatus {
  Pending = 'Pending',
  Scheduled = 'Scheduled',
  Running = 'Running',
  Completed = 'Completed',
  Failed = 'Failed',
  Cancelled = 'Cancelled',
  Retrying = 'Retrying',
}

export type JobSchedule =
  | { type: 'Once'; timestamp: string }
  | { type: 'Cron'; expression: string }
  | { type: 'Interval'; duration: number; start?: string };

export interface Job {
  id: string;
  name: string;
  job_type: string;
  schedule: JobSchedule;
  priority: JobPriority;
  status: JobStatus;
  payload: any;
  max_retries: number;
  retry_count: number;
  timeout_seconds: number;
  created_at: string;
  updated_at: string;
  next_run?: string;
  last_run?: string;
  last_error?: string;
  tags: Record<string, string>;
}

// ============================================================================
// Queue Types
// ============================================================================

export interface JobProgress {
  job_id: string;
  current: number;
  total: number;
  percentage: number;
  message?: string;
  updated_at: string;
}

export interface QueuedJob {
  id: string;
  queue_name: string;
  job_type: string;
  priority: JobPriority;
  payload: any;
  dedup_key?: string;
  delay_until?: string;
  max_retries: number;
  retry_count: number;
  timeout_seconds: number;
  created_at: string;
  started_at?: string;
  completed_at?: string;
  error?: string;
  metadata: Record<string, string>;
}

export interface QueueStats {
  queue_name: string;
  pending_jobs: number;
  failed_jobs: number;
}

// ============================================================================
// Worker Types
// ============================================================================

export enum WorkerStatus {
  Idle = 'Idle',
  Busy = 'Busy',
  Unhealthy = 'Unhealthy',
  Shutdown = 'Shutdown',
}

export interface WorkerHealth {
  worker_id: string;
  status: WorkerStatus;
  tasks_completed: number;
  tasks_failed: number;
  last_heartbeat: string;
  uptime_seconds: number;
  memory_usage_mb: number;
  cpu_usage_percent: number;
}

export interface PoolStats {
  total_workers: number;
  active_workers: number;
  idle_workers: number;
  total_tasks_completed: number;
  total_tasks_failed: number;
}

// ============================================================================
// Monitor Types
// ============================================================================

export enum MonitorStatus {
  Up = 'Up',
  Down = 'Down',
  Degraded = 'Degraded',
  Unknown = 'Unknown',
}

export type CheckType =
  | {
      type: 'Http';
      url: string;
      method: string;
      expected_status: number;
      timeout_ms: number;
    }
  | {
      type: 'AccessibilityScan';
      url: string;
      standards: string[];
    }
  | {
      type: 'ContentChange';
      url: string;
      selector?: string;
      hash_algorithm: string;
    }
  | {
      type: 'Performance';
      url: string;
      max_load_time_ms: number;
      max_first_byte_ms: number;
    }
  | {
      type: 'Custom';
      check_type: string;
      config: any;
    };

export interface Monitor {
  id: string;
  name: string;
  check_type: CheckType;
  interval_seconds: number;
  enabled: boolean;
  alert_on_failure: boolean;
  alert_threshold: number;
  created_at: string;
  updated_at: string;
  tags: Record<string, string>;
}

export interface CheckResult {
  check_id: string;
  monitor_id: string;
  status: MonitorStatus;
  response_time_ms: number;
  timestamp: string;
  error?: string;
  metadata: Record<string, any>;
}

export interface UptimeStats {
  monitor_id: string;
  total_checks: number;
  successful_checks: number;
  failed_checks: number;
  uptime_percentage: number;
  average_response_time_ms: number;
  last_downtime?: string;
  last_check?: string;
}

export interface ChangeDetection {
  monitor_id: string;
  previous_hash: string;
  current_hash: string;
  changed: boolean;
  change_percentage: number;
  detected_at: string;
  details?: string;
}

export interface PerformanceMetrics {
  monitor_id: string;
  timestamp: string;
  dns_time_ms: number;
  connect_time_ms: number;
  first_byte_time_ms: number;
  download_time_ms: number;
  total_time_ms: number;
  content_size_bytes: number;
  http_status: number;
}

export enum AlertSeverity {
  Info = 'Info',
  Warning = 'Warning',
  Error = 'Error',
  Critical = 'Critical',
}

export interface MonitorAlert {
  id: string;
  monitor_id: string;
  monitor_name: string;
  severity: AlertSeverity;
  message: string;
  consecutive_failures: number;
  created_at: string;
  acknowledged: boolean;
  resolved: boolean;
}

// ============================================================================
// Notification Types
// ============================================================================

export enum NotificationSeverity {
  Info = 'Info',
  Warning = 'Warning',
  Error = 'Error',
  Critical = 'Critical',
}

export enum NotificationPriority {
  Low = 'Low',
  Normal = 'Normal',
  High = 'High',
  Urgent = 'Urgent',
}

export enum NotificationChannel {
  Email = 'Email',
  Slack = 'Slack',
  MicrosoftTeams = 'MicrosoftTeams',
  Webhook = 'Webhook',
  Console = 'Console',
}

export interface Notification {
  id: string;
  title: string;
  message: string;
  severity: NotificationSeverity;
  priority: NotificationPriority;
  source: string;
  metadata: Record<string, any>;
  created_at: string;
  delivered: boolean;
  delivery_attempts: number;
}

export interface QuietHours {
  start_hour: number;
  end_hour: number;
  timezone: string;
}

export interface NotificationPreferences {
  user_id: string;
  enabled_channels: NotificationChannel[];
  min_severity: NotificationSeverity;
  min_priority: NotificationPriority;
  quiet_hours?: QuietHours;
  source_filters: string[];
}

export interface EmailConfig {
  smtp_host: string;
  smtp_port: number;
  smtp_username: string;
  smtp_password: string;
  from_address: string;
  from_name: string;
  to_addresses: string[];
  use_tls: boolean;
}

export interface SlackConfig {
  webhook_url: string;
  channel?: string;
  username?: string;
  icon_emoji?: string;
}

export interface TeamsConfig {
  webhook_url: string;
}

export interface WebhookConfig {
  url: string;
  method: string;
  headers: Record<string, string>;
  timeout_seconds: number;
}

// ============================================================================
// API Request/Response Types
// ============================================================================

export interface CreateJobRequest {
  name: string;
  job_type: string;
  schedule: JobSchedule;
  priority?: JobPriority;
  payload?: any;
  max_retries?: number;
  timeout_seconds?: number;
  tags?: Record<string, string>;
}

export interface CreateMonitorRequest {
  name: string;
  check_type: CheckType;
  interval_seconds: number;
  alert_on_failure?: boolean;
  alert_threshold?: number;
  tags?: Record<string, string>;
}

export interface UpdateMonitorRequest {
  name?: string;
  check_type?: CheckType;
  interval_seconds?: number;
  enabled?: boolean;
  alert_on_failure?: boolean;
  alert_threshold?: number;
  tags?: Record<string, string>;
}

export interface ScaleWorkerPoolRequest {
  target_workers: number;
  queue_names: string[];
}

// ============================================================================
// UI State Types
// ============================================================================

export interface ScheduleFormState {
  name: string;
  job_type: string;
  schedule_type: 'once' | 'cron' | 'interval';
  once_timestamp?: string;
  cron_expression?: string;
  interval_duration?: number;
  interval_start?: string;
  priority: JobPriority;
  payload: string; // JSON string
  max_retries: number;
  timeout_seconds: number;
  tags: Array<{ key: string; value: string }>;
}

export interface MonitorFormState {
  name: string;
  check_type: 'http' | 'accessibility' | 'content_change' | 'performance' | 'custom';
  url: string;
  http_method?: string;
  http_expected_status?: number;
  http_timeout_ms?: number;
  accessibility_standards?: string[];
  content_selector?: string;
  content_hash_algorithm?: string;
  perf_max_load_time?: number;
  perf_max_first_byte?: number;
  custom_type?: string;
  custom_config?: string; // JSON string
  interval_seconds: number;
  alert_on_failure: boolean;
  alert_threshold: number;
  tags: Array<{ key: string; value: string }>;
}

export interface DashboardFilters {
  status?: MonitorStatus[];
  timeRange?: {
    start: string;
    end: string;
  };
  searchQuery?: string;
}

export interface ChartDataPoint {
  timestamp: string;
  value: number;
  label?: string;
}

export interface TimeSeriesData {
  labels: string[];
  datasets: Array<{
    label: string;
    data: number[];
    borderColor?: string;
    backgroundColor?: string;
  }>;
}
