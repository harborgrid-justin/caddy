/**
 * CADDY v0.4.0 - Central Type Definitions
 *
 * Shared type definitions used across all modules in the CADDY platform.
 * This module provides common types, interfaces, and utilities that ensure
 * type consistency throughout the entire application.
 */

// ============================================================================
// Common API Types
// ============================================================================

/**
 * Standard API response wrapper
 */
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: ApiError;
  metadata?: ResponseMetadata;
}

/**
 * API error structure
 */
export interface ApiError {
  code: string;
  message: string;
  details?: any;
  statusCode: number;
  timestamp?: string;
}

/**
 * Response metadata
 */
export interface ResponseMetadata {
  timestamp: string;
  requestId: string;
  version: string;
  processingTime?: number;
}

// ============================================================================
// Pagination Types
// ============================================================================

/**
 * Pagination parameters
 */
export interface PaginationParams {
  page: number;
  pageSize: number;
}

/**
 * Cursor-based pagination parameters
 */
export interface CursorPaginationParams {
  cursor?: string;
  pageSize: number;
}

/**
 * Paginated response
 */
export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
  hasMore: boolean;
}

/**
 * Cursor-based paginated response
 */
export interface CursorPaginatedResponse<T> {
  items: T[];
  nextCursor?: string;
  previousCursor?: string;
  hasMore: boolean;
  total?: number;
}

// ============================================================================
// Sorting & Filtering Types
// ============================================================================

/**
 * Sort direction
 */
export type SortDirection = 'asc' | 'desc';

/**
 * Sort parameters
 */
export interface SortParams {
  sortBy: string;
  sortOrder: SortDirection;
}

/**
 * Filter parameters
 */
export interface FilterParams {
  [key: string]: any;
}

/**
 * Search parameters
 */
export interface SearchParams {
  search: string;
  searchFields?: string[];
}

// ============================================================================
// Time & Date Types
// ============================================================================

/**
 * Time range
 */
export interface TimeRange {
  start: Date | string;
  end: Date | string;
}

/**
 * Date range
 */
export interface DateRange {
  startDate: Date | string;
  endDate: Date | string;
}

/**
 * Time range preset
 */
export type TimeRangePreset =
  | '1h'
  | '24h'
  | '7d'
  | '30d'
  | '90d'
  | '1y'
  | 'custom';

// ============================================================================
// User Context Types
// ============================================================================

/**
 * User context for authenticated requests
 */
export interface UserContext {
  userId: string;
  tenantId: string;
  username: string;
  email: string;
  roles: string[];
  permissions: Permission[];
  sessionId: string;
  metadata?: Record<string, any>;
}

/**
 * Permission definition
 */
export interface Permission {
  id: string;
  resource: string;
  action: PermissionAction;
  scope: PermissionScope;
  conditions?: PermissionCondition[];
  effect: 'allow' | 'deny';
}

/**
 * Permission action types
 */
export type PermissionAction =
  | 'create'
  | 'read'
  | 'update'
  | 'delete'
  | 'list'
  | 'execute'
  | 'manage'
  | 'approve'
  | 'publish'
  | '*';

/**
 * Permission scope
 */
export type PermissionScope =
  | 'own'
  | 'team'
  | 'department'
  | 'organization'
  | 'tenant'
  | 'global';

/**
 * Permission condition
 */
export interface PermissionCondition {
  field: string;
  operator: 'eq' | 'ne' | 'gt' | 'gte' | 'lt' | 'lte' | 'in' | 'nin' | 'contains';
  value: any;
}

// ============================================================================
// Audit & Logging Types
// ============================================================================

/**
 * Audit entry for tracking changes
 */
export interface AuditEntry {
  id: string;
  userId: string;
  tenantId: string;
  action: string;
  resource: string;
  resourceId?: string;
  changes?: ChangeRecord[];
  metadata?: AuditMetadata;
  timestamp: string;
  ipAddress: string;
  userAgent: string;
}

/**
 * Change record for audit trail
 */
export interface ChangeRecord {
  field: string;
  oldValue?: any;
  newValue?: any;
}

/**
 * Audit metadata
 */
export interface AuditMetadata {
  requestId?: string;
  sessionId?: string;
  location?: string;
  deviceId?: string;
  source?: string;
  reason?: string;
}

// ============================================================================
// Settings Types
// ============================================================================

/**
 * Setting definition
 */
export interface Setting {
  key: string;
  value: any;
  type: 'string' | 'number' | 'boolean' | 'object' | 'array';
  encrypted: boolean;
  scope: 'system' | 'tenant' | 'user';
  category?: string;
  description?: string;
  defaultValue?: any;
  validation?: SettingValidation;
}

/**
 * Setting validation rules
 */
export interface SettingValidation {
  required?: boolean;
  min?: number;
  max?: number;
  pattern?: string;
  enum?: any[];
  custom?: (value: any) => boolean | string;
}

// ============================================================================
// Metrics Types
// ============================================================================

/**
 * Metric data point
 */
export interface Metric {
  id: string;
  name: string;
  value: number;
  unit?: string;
  timestamp: string;
  tags?: Record<string, string>;
  metadata?: Record<string, any>;
}

/**
 * Metric with trend information
 */
export interface TrendMetric extends Metric {
  previousValue?: number;
  change?: number;
  changePercent?: number;
  trend: 'up' | 'down' | 'neutral';
}

/**
 * Time series data point
 */
export interface TimeSeriesDataPoint {
  timestamp: string;
  value: number;
  label?: string;
}

// ============================================================================
// Notification Types
// ============================================================================

/**
 * Notification
 */
export interface Notification {
  id: string;
  tenantId: string;
  userId: string;
  type: NotificationType;
  priority: NotificationPriority;
  title: string;
  message: string;
  data?: Record<string, any>;
  read: boolean;
  actionUrl?: string;
  actionLabel?: string;
  createdAt: string;
  readAt?: string;
  expiresAt?: string;
}

/**
 * Notification type
 */
export type NotificationType =
  | 'info'
  | 'success'
  | 'warning'
  | 'error'
  | 'alert'
  | 'system';

/**
 * Notification priority
 */
export type NotificationPriority = 'low' | 'medium' | 'high' | 'urgent';

// ============================================================================
// File Types
// ============================================================================

/**
 * File metadata
 */
export interface FileMetadata {
  id: string;
  tenantId: string;
  name: string;
  path: string;
  size: number;
  mimeType: string;
  extension: string;
  checksum: string;
  url?: string;
  thumbnailUrl?: string;
  ownerId: string;
  folderId?: string;
  tags?: string[];
  metadata?: Record<string, any>;
  createdAt: string;
  updatedAt: string;
  deletedAt?: string;
}

/**
 * File upload progress
 */
export interface FileUploadProgress {
  fileId: string;
  filename: string;
  size: number;
  uploaded: number;
  progress: number;
  status: 'pending' | 'uploading' | 'processing' | 'completed' | 'failed';
  error?: string;
}

// ============================================================================
// Status Types
// ============================================================================

/**
 * Generic status type
 */
export type Status =
  | 'active'
  | 'inactive'
  | 'pending'
  | 'suspended'
  | 'archived'
  | 'deleted';

/**
 * Health status
 */
export type HealthStatus = 'healthy' | 'degraded' | 'unhealthy' | 'unknown';

/**
 * Execution status
 */
export type ExecutionStatus =
  | 'idle'
  | 'running'
  | 'paused'
  | 'completed'
  | 'failed'
  | 'cancelled'
  | 'retrying';

// ============================================================================
// Theme Types
// ============================================================================

/**
 * Theme mode
 */
export type ThemeMode = 'light' | 'dark' | 'auto';

/**
 * Color palette
 */
export interface ColorPalette {
  primary: string;
  secondary: string;
  success: string;
  warning: string;
  error: string;
  info: string;
  background: string;
  surface: string;
  text: string;
  textSecondary: string;
  border: string;
  divider: string;
}

// ============================================================================
// Event Types
// ============================================================================

/**
 * System event
 */
export interface SystemEvent {
  id: string;
  type: string;
  source: string;
  tenantId?: string;
  userId?: string;
  data: any;
  timestamp: string;
  metadata?: Record<string, any>;
}

/**
 * WebSocket event
 */
export interface WebSocketEvent {
  type: string;
  channel?: string;
  data: any;
  timestamp: string;
}

// ============================================================================
// Validation Types
// ============================================================================

/**
 * Validation result
 */
export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
  warnings?: ValidationWarning[];
}

/**
 * Validation error
 */
export interface ValidationError {
  field?: string;
  message: string;
  code: string;
  severity: 'error';
}

/**
 * Validation warning
 */
export interface ValidationWarning {
  field?: string;
  message: string;
  code: string;
  severity: 'warning';
}

// ============================================================================
// Chart & Visualization Types
// ============================================================================

/**
 * Chart type
 */
export type ChartType = 'line' | 'bar' | 'pie' | 'area' | 'donut' | 'scatter' | 'radar';

/**
 * Chart dataset
 */
export interface ChartDataset {
  label: string;
  data: number[];
  color?: string;
  backgroundColor?: string | string[];
  borderColor?: string;
  borderWidth?: number;
}

/**
 * Chart options
 */
export interface ChartOptions {
  responsive?: boolean;
  maintainAspectRatio?: boolean;
  showLegend?: boolean;
  showGrid?: boolean;
  showTooltips?: boolean;
  animated?: boolean;
  stacked?: boolean;
  aspectRatio?: number;
}

// ============================================================================
// Tenant Types
// ============================================================================

/**
 * Tenant information
 */
export interface Tenant {
  id: string;
  name: string;
  displayName: string;
  domain?: string;
  status: Status;
  plan: SubscriptionPlan;
  settings: TenantSettings;
  metadata?: Record<string, any>;
  createdAt: string;
  updatedAt: string;
}

/**
 * Subscription plan
 */
export type SubscriptionPlan = 'free' | 'starter' | 'professional' | 'enterprise';

/**
 * Tenant settings
 */
export interface TenantSettings {
  maxUsers?: number;
  maxStorage?: number;
  features: string[];
  customDomain?: boolean;
  ssoEnabled?: boolean;
  auditLogRetentionDays?: number;
}

// ============================================================================
// Address & Location Types
// ============================================================================

/**
 * Address
 */
export interface Address {
  street1: string;
  street2?: string;
  city: string;
  state?: string;
  postalCode: string;
  country: string;
}

/**
 * Geographic location
 */
export interface GeoLocation {
  latitude: number;
  longitude: number;
  accuracy?: number;
}

/**
 * Location information
 */
export interface Location {
  country?: string;
  region?: string;
  city?: string;
  timezone?: string;
  coordinates?: GeoLocation;
}

// ============================================================================
// Configuration Types
// ============================================================================

/**
 * Feature flag
 */
export interface FeatureFlag {
  key: string;
  enabled: boolean;
  description?: string;
  rolloutPercentage?: number;
  conditions?: FeatureFlagCondition[];
}

/**
 * Feature flag condition
 */
export interface FeatureFlagCondition {
  type: 'user' | 'tenant' | 'role' | 'custom';
  operator: 'equals' | 'contains' | 'in';
  value: any;
}

// ============================================================================
// Utility Types
// ============================================================================

/**
 * Deep partial - makes all properties and nested properties optional
 */
export type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};

/**
 * Deep readonly - makes all properties and nested properties readonly
 */
export type DeepReadonly<T> = {
  readonly [P in keyof T]: T[P] extends object ? DeepReadonly<T[P]> : T[P];
};

/**
 * Nullable type
 */
export type Nullable<T> = T | null;

/**
 * Optional type
 */
export type Optional<T> = T | undefined;

/**
 * ID type
 */
export type ID = string | number;

/**
 * Timestamp type (ISO 8601 string)
 */
export type Timestamp = string;

/**
 * URL type
 */
export type URL = string;

/**
 * Email type
 */
export type Email = string;

/**
 * UUID type
 */
export type UUID = string;

/**
 * JSON value type
 */
export type JSONValue =
  | string
  | number
  | boolean
  | null
  | JSONValue[]
  | { [key: string]: JSONValue };

/**
 * JSON object type
 */
export type JSONObject = { [key: string]: JSONValue };

/**
 * Callback function type
 */
export type Callback<T = void> = (result: T) => void;

/**
 * Error callback function type
 */
export type ErrorCallback = (error: Error | ApiError) => void;

/**
 * Event handler type
 */
export type EventHandler<T = any> = (event: T) => void;

/**
 * Async function type
 */
export type AsyncFunction<TArgs extends any[] = any[], TReturn = any> = (
  ...args: TArgs
) => Promise<TReturn>;

// ============================================================================
// HTTP Types
// ============================================================================

/**
 * HTTP method
 */
export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE' | 'HEAD' | 'OPTIONS';

/**
 * HTTP status code
 */
export type HttpStatusCode =
  | 200 // OK
  | 201 // Created
  | 204 // No Content
  | 400 // Bad Request
  | 401 // Unauthorized
  | 403 // Forbidden
  | 404 // Not Found
  | 409 // Conflict
  | 422 // Unprocessable Entity
  | 429 // Too Many Requests
  | 500 // Internal Server Error
  | 502 // Bad Gateway
  | 503; // Service Unavailable

/**
 * HTTP headers
 */
export type HttpHeaders = Record<string, string>;

/**
 * Request configuration
 */
export interface RequestConfig {
  method?: HttpMethod;
  headers?: HttpHeaders;
  body?: any;
  timeout?: number;
  retries?: number;
  cache?: boolean;
}

// ============================================================================
// Queue & Job Types
// ============================================================================

/**
 * Job status
 */
export type JobStatus =
  | 'pending'
  | 'queued'
  | 'running'
  | 'completed'
  | 'failed'
  | 'cancelled'
  | 'retrying';

/**
 * Job priority
 */
export type JobPriority = 'low' | 'medium' | 'high' | 'critical';

/**
 * Job definition
 */
export interface Job {
  id: string;
  type: string;
  status: JobStatus;
  priority: JobPriority;
  data: any;
  result?: any;
  error?: string;
  attempts: number;
  maxAttempts: number;
  createdAt: string;
  startedAt?: string;
  completedAt?: string;
  failedAt?: string;
}

// ============================================================================
// Export All
// ============================================================================

export default {
  // API Types
  ApiResponse,
  ApiError,
  ResponseMetadata,

  // Pagination Types
  PaginationParams,
  CursorPaginationParams,
  PaginatedResponse,
  CursorPaginatedResponse,

  // Sorting & Filtering Types
  SortParams,
  FilterParams,
  SearchParams,

  // Time & Date Types
  TimeRange,
  DateRange,

  // User Context Types
  UserContext,
  Permission,

  // Audit & Logging Types
  AuditEntry,
  ChangeRecord,

  // Settings Types
  Setting,
  SettingValidation,

  // Metrics Types
  Metric,
  TrendMetric,
  TimeSeriesDataPoint,

  // Notification Types
  Notification,

  // File Types
  FileMetadata,
  FileUploadProgress,

  // Chart Types
  ChartDataset,
  ChartOptions,

  // Tenant Types
  Tenant,
  TenantSettings,

  // Location Types
  Address,
  GeoLocation,
  Location,

  // Configuration Types
  FeatureFlag,
  FeatureFlagCondition,

  // HTTP Types
  RequestConfig,

  // Job Types
  Job,
};
