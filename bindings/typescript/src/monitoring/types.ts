/**
 * CADDY v0.4.0 - System Monitoring Types
 * Enterprise-grade monitoring and alerting type definitions
 * @module monitoring/types
 */

export enum ServiceStatus {
  HEALTHY = 'healthy',
  DEGRADED = 'degraded',
  DOWN = 'down',
  MAINTENANCE = 'maintenance',
  UNKNOWN = 'unknown'
}

export enum AlertSeverity {
  CRITICAL = 'critical',
  HIGH = 'high',
  MEDIUM = 'medium',
  LOW = 'low',
  INFO = 'info'
}

export enum AlertState {
  ACTIVE = 'active',
  ACKNOWLEDGED = 'acknowledged',
  RESOLVED = 'resolved',
  SILENCED = 'silenced'
}

export enum IncidentStatus {
  INVESTIGATING = 'investigating',
  IDENTIFIED = 'identified',
  MONITORING = 'monitoring',
  RESOLVED = 'resolved'
}

export enum MetricType {
  CPU = 'cpu',
  MEMORY = 'memory',
  DISK = 'disk',
  NETWORK = 'network',
  LATENCY = 'latency',
  THROUGHPUT = 'throughput',
  ERROR_RATE = 'error_rate',
  CUSTOM = 'custom'
}

export interface ServiceHealth {
  id: string;
  name: string;
  status: ServiceStatus;
  responseTime: number;
  uptime: number;
  lastCheck: Date;
  message?: string;
  dependencies: string[];
  metrics: ServiceMetrics;
}

export interface ServiceMetrics {
  cpu: number;
  memory: number;
  disk: number;
  network: NetworkMetrics;
  requestRate: number;
  errorRate: number;
  latencyP50: number;
  latencyP95: number;
  latencyP99: number;
}

export interface NetworkMetrics {
  bytesIn: number;
  bytesOut: number;
  packetsIn: number;
  packetsOut: number;
  errorsIn: number;
  errorsOut: number;
}

export interface PerformanceMetric {
  timestamp: Date;
  type: MetricType;
  value: number;
  unit: string;
  service?: string;
  tags?: Record<string, string>;
}

export interface Alert {
  id: string;
  name: string;
  severity: AlertSeverity;
  state: AlertState;
  message: string;
  service: string;
  threshold: AlertThreshold;
  triggeredAt: Date;
  acknowledgedAt?: Date;
  resolvedAt?: Date;
  acknowledgedBy?: string;
  resolvedBy?: string;
  metadata: Record<string, any>;
  silencedUntil?: Date;
}

export interface AlertThreshold {
  metric: MetricType;
  operator: 'gt' | 'gte' | 'lt' | 'lte' | 'eq' | 'neq';
  value: number;
  duration: number; // seconds
  evaluationWindow: number; // seconds
}

export interface AlertRule {
  id: string;
  name: string;
  description: string;
  enabled: boolean;
  severity: AlertSeverity;
  service: string;
  threshold: AlertThreshold;
  notificationChannels: string[];
  cooldown: number; // seconds
  createdAt: Date;
  updatedAt: Date;
  createdBy: string;
}

export interface Incident {
  id: string;
  title: string;
  description: string;
  status: IncidentStatus;
  severity: AlertSeverity;
  affectedServices: string[];
  startedAt: Date;
  identifiedAt?: Date;
  resolvedAt?: Date;
  rootCause?: string;
  resolution?: string;
  timeline: IncidentTimelineEntry[];
  assignedTo?: string;
  createdBy: string;
  impactedUsers?: number;
  metadata: Record<string, any>;
}

export interface IncidentTimelineEntry {
  id: string;
  timestamp: Date;
  type: 'update' | 'status_change' | 'comment' | 'action';
  message: string;
  user: string;
  metadata?: Record<string, any>;
}

export interface UptimeRecord {
  service: string;
  period: 'hour' | 'day' | 'week' | 'month' | 'year';
  uptime: number; // percentage
  totalChecks: number;
  successfulChecks: number;
  failedChecks: number;
  averageResponseTime: number;
  startTime: Date;
  endTime: Date;
}

export interface SLATarget {
  id: string;
  service: string;
  metric: MetricType;
  target: number; // percentage or value
  period: 'day' | 'week' | 'month' | 'quarter' | 'year';
  current: number;
  status: 'met' | 'at_risk' | 'breached';
  errorBudget: number;
  errorBudgetRemaining: number;
}

export interface LogEntry {
  id: string;
  timestamp: Date;
  level: 'debug' | 'info' | 'warn' | 'error' | 'fatal';
  service: string;
  message: string;
  context?: Record<string, any>;
  traceId?: string;
  spanId?: string;
  stack?: string;
}

export interface LogFilter {
  services?: string[];
  levels?: LogEntry['level'][];
  search?: string;
  startTime?: Date;
  endTime?: Date;
  traceId?: string;
  limit?: number;
}

export interface ResourceUsage {
  timestamp: Date;
  service: string;
  cpu: ResourceMetric;
  memory: ResourceMetric;
  disk: ResourceMetric;
  network: NetworkMetrics;
}

export interface ResourceMetric {
  used: number;
  total: number;
  percentage: number;
  unit: string;
}

export interface ServiceDependency {
  id: string;
  name: string;
  type: 'internal' | 'external' | 'database' | 'cache' | 'queue';
  status: ServiceStatus;
  dependencies: ServiceDependency[];
  healthEndpoint?: string;
  criticalPath: boolean;
}

export interface MaintenanceWindow {
  id: string;
  title: string;
  description: string;
  services: string[];
  startTime: Date;
  endTime: Date;
  status: 'scheduled' | 'active' | 'completed' | 'cancelled';
  impactLevel: 'none' | 'minor' | 'major' | 'full';
  createdBy: string;
  notifyUsers: boolean;
  metadata?: Record<string, any>;
}

export interface StatusPageConfig {
  id: string;
  title: string;
  description: string;
  logo?: string;
  publicUrl: string;
  services: StatusPageService[];
  showMetrics: boolean;
  showIncidents: boolean;
  customDomain?: string;
  theme: {
    primaryColor: string;
    backgroundColor: string;
    textColor: string;
  };
}

export interface StatusPageService {
  id: string;
  name: string;
  description?: string;
  group?: string;
  displayOrder: number;
  showUptime: boolean;
  showMetrics: boolean;
}

export interface AnomalyDetection {
  id: string;
  service: string;
  metric: MetricType;
  timestamp: Date;
  expectedValue: number;
  actualValue: number;
  deviation: number; // percentage
  confidence: number; // 0-1
  severity: AlertSeverity;
  model: 'statistical' | 'ml' | 'seasonal';
}

export interface MonitoringDashboardConfig {
  id: string;
  name: string;
  layout: DashboardWidget[];
  refreshInterval: number; // seconds
  timeRange: TimeRange;
  filters: {
    services?: string[];
    tags?: Record<string, string>;
  };
  createdBy: string;
  shared: boolean;
}

export interface DashboardWidget {
  id: string;
  type: 'chart' | 'stat' | 'table' | 'log' | 'alert' | 'service_map';
  title: string;
  position: { x: number; y: number; w: number; h: number };
  config: Record<string, any>;
}

export interface TimeRange {
  from: Date;
  to: Date;
  quick?: '5m' | '15m' | '1h' | '6h' | '24h' | '7d' | '30d';
}

export interface WebSocketMessage {
  type: 'metric' | 'alert' | 'log' | 'health' | 'incident';
  data: any;
  timestamp: Date;
  service?: string;
}

export interface NotificationChannel {
  id: string;
  name: string;
  type: 'email' | 'slack' | 'pagerduty' | 'webhook' | 'sms';
  enabled: boolean;
  config: Record<string, any>;
  services: string[];
  severities: AlertSeverity[];
}

export interface HealthCheckConfig {
  id: string;
  service: string;
  type: 'http' | 'tcp' | 'grpc' | 'script';
  endpoint: string;
  interval: number; // seconds
  timeout: number; // seconds
  retries: number;
  expectedStatus?: number;
  expectedBody?: string;
  headers?: Record<string, string>;
  metadata?: Record<string, any>;
}
