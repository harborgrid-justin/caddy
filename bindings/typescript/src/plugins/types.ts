/**
 * Plugin Type Definitions for CADDY Enterprise SDK
 *
 * Comprehensive type definitions for the plugin system including
 * manifests, permissions, lifecycle states, and API interfaces.
 */

/**
 * Plugin manifest structure
 */
export interface PluginManifest {
  /** Unique plugin identifier */
  id: string;

  /** Plugin name */
  name: string;

  /** Plugin version (semver) */
  version: string;

  /** Plugin description */
  description: string;

  /** Plugin author */
  author: string;

  /** Required API version */
  apiVersion: string;

  /** Plugin entry point */
  entryPoint: string;

  /** Plugin type */
  pluginType: PluginType;

  /** Required permissions */
  permissions: string[];

  /** Plugin dependencies */
  dependencies?: PluginDependency[];

  /** Plugin capabilities */
  capabilities?: string[];

  /** Plugin icon (base64 or path) */
  icon?: string;

  /** Plugin website */
  website?: string;

  /** Plugin repository */
  repository?: string;

  /** Plugin license */
  license?: string;

  /** Minimum CADDY version */
  minCaddyVersion?: string;

  /** Maximum CADDY version */
  maxCaddyVersion?: string;
}

/**
 * Plugin types
 */
export enum PluginType {
  /** WebAssembly plugin (sandboxed) */
  Wasm = 'Wasm',
  /** Native plugin (higher performance, less secure) */
  Native = 'Native',
}

/**
 * Plugin dependency specification
 */
export interface PluginDependency {
  /** Dependency plugin ID */
  id: string;

  /** Version requirement (semver) */
  versionRequirement: string;

  /** Whether dependency is optional */
  optional?: boolean;
}

/**
 * Plugin lifecycle states
 */
export enum PluginState {
  Loading = 'Loading',
  Loaded = 'Loaded',
  Initializing = 'Initializing',
  Ready = 'Ready',
  Starting = 'Starting',
  Running = 'Running',
  Suspended = 'Suspended',
  Stopping = 'Stopping',
  Stopped = 'Stopped',
  Error = 'Error',
  Unloading = 'Unloading',
  Unloaded = 'Unloaded',
}

/**
 * Plugin permissions
 */
export enum Permission {
  // Geometry permissions
  GeometryRead = 'geometry:read',
  GeometryWrite = 'geometry:write',
  GeometryDelete = 'geometry:delete',

  // Rendering permissions
  RenderingRead = 'rendering:read',
  RenderingWrite = 'rendering:write',
  RenderingShaderAccess = 'rendering:shader',

  // UI permissions
  UIRead = 'ui:read',
  UIWrite = 'ui:write',
  UIMenuAccess = 'ui:menu',
  UIToolbarAccess = 'ui:toolbar',
  UIDialogAccess = 'ui:dialog',

  // File I/O permissions
  FileRead = 'file:read',
  FileWrite = 'file:write',
  FileDelete = 'file:delete',
  FileExecute = 'file:execute',

  // Command permissions
  CommandExecute = 'command:execute',
  CommandRegister = 'command:register',

  // Layer permissions
  LayerRead = 'layer:read',
  LayerWrite = 'layer:write',
  LayerDelete = 'layer:delete',

  // Network permissions
  NetworkHTTP = 'network:http',
  NetworkWebSocket = 'network:websocket',
  NetworkUnrestricted = 'network:unrestricted',

  // System permissions
  SystemClipboard = 'system:clipboard',
  SystemNotifications = 'system:notifications',

  // Database permissions
  DatabaseRead = 'database:read',
  DatabaseWrite = 'database:write',

  // Enterprise permissions
  EnterpriseAccess = 'enterprise:access',
}

/**
 * Plugin information
 */
export interface PluginInfo {
  /** Plugin manifest */
  manifest: PluginManifest;

  /** Current state */
  state: PluginState;

  /** When plugin was loaded */
  loadedAt: string;

  /** Whether plugin is enabled */
  enabled: boolean;

  /** Installation source */
  source: InstallationSource;

  /** Resource usage statistics */
  resourceUsage?: ResourceUsageStats;
}

/**
 * Installation sources
 */
export type InstallationSource =
  | { type: 'marketplace'; url: string }
  | { type: 'local' }
  | { type: 'git'; repo: string; commit: string }
  | { type: 'url'; url: string }
  | { type: 'builtin' };

/**
 * Resource usage statistics
 */
export interface ResourceUsageStats {
  /** Memory used in bytes */
  memoryUsedBytes: number;

  /** Memory limit in bytes */
  memoryLimitBytes: number;

  /** Execution time in milliseconds */
  executionTimeMs: number;

  /** Execution limit in milliseconds */
  executionLimitMs: number;

  /** File operations count */
  fileOpsCount: number;

  /** Network requests count */
  networkRequestsCount: number;
}

/**
 * Marketplace plugin listing
 */
export interface MarketplacePlugin {
  /** Plugin ID */
  id: string;

  /** Plugin name */
  name: string;

  /** Short description */
  description: string;

  /** Current version */
  version: string;

  /** Author information */
  author: MarketplaceAuthor;

  /** Plugin icon URL */
  iconUrl?: string;

  /** Download count */
  downloads: number;

  /** Average rating (0-5) */
  rating: number;

  /** Number of ratings */
  ratingCount: number;

  /** Categories/tags */
  categories: string[];

  /** Last updated timestamp */
  updatedAt: string;

  /** Plugin size in bytes */
  sizeBytes: number;

  /** Download URL */
  downloadUrl: string;

  /** Manifest URL */
  manifestUrl: string;

  /** Is verified plugin */
  verified: boolean;

  /** License type */
  license: string;

  /** Minimum CADDY version required */
  minCaddyVersion: string;
}

/**
 * Marketplace author
 */
export interface MarketplaceAuthor {
  id: string;
  name: string;
  email?: string;
  website?: string;
  verified: boolean;
}

/**
 * Marketplace search filters
 */
export interface SearchFilters {
  /** Search query */
  query?: string;

  /** Category filter */
  category?: string;

  /** Minimum rating */
  minRating?: number;

  /** Only verified plugins */
  verifiedOnly?: boolean;

  /** Sort by field */
  sortBy?: SortBy;

  /** Page number (0-indexed) */
  page?: number;

  /** Results per page */
  perPage?: number;
}

/**
 * Sort options
 */
export enum SortBy {
  Relevance = 'Relevance',
  Downloads = 'Downloads',
  Rating = 'Rating',
  Updated = 'Updated',
  Name = 'Name',
}

/**
 * Search results
 */
export interface SearchResults {
  plugins: MarketplacePlugin[];
  totalCount: number;
  page: number;
  perPage: number;
  totalPages: number;
}

/**
 * Plugin update information
 */
export interface PluginUpdate {
  pluginId: string;
  currentVersion: string;
  latestVersion: string;
  changelogUrl?: string;
}

/**
 * Plugin category
 */
export interface Category {
  id: string;
  name: string;
  description: string;
  icon?: string;
  pluginCount: number;
}

/**
 * Plugin configuration
 */
export interface PluginConfig {
  /** Plugin-specific configuration */
  [key: string]: any;
}

/**
 * Plugin settings
 */
export interface PluginSettings {
  /** Plugin ID */
  pluginId: string;

  /** Whether plugin is enabled */
  enabled: boolean;

  /** Plugin configuration */
  config: PluginConfig;

  /** Auto-start plugin */
  autoStart: boolean;

  /** Resource limits */
  resourceLimits?: ResourceLimits;
}

/**
 * Resource limits
 */
export interface ResourceLimits {
  /** Maximum memory in bytes */
  maxMemoryBytes: number;

  /** Maximum execution time in milliseconds */
  maxExecutionTimeMs: number;

  /** Maximum file size in bytes */
  maxFileSizeBytes: number;

  /** Maximum file operations per second */
  maxFileOpsPerSecond: number;

  /** Maximum network requests per second */
  maxNetworkRequestsPerSecond: number;

  /** Maximum CPU percentage */
  maxCpuPercent: number;
}

/**
 * Plugin event types
 */
export enum PluginEventType {
  Loaded = 'loaded',
  Unloaded = 'unloaded',
  Reloaded = 'reloaded',
  StateChanged = 'stateChanged',
  Error = 'error',
  ConfigChanged = 'configChanged',
}

/**
 * Plugin event
 */
export interface PluginEvent {
  type: PluginEventType;
  pluginId: string;
  timestamp: string;
  data?: any;
}

/**
 * Plugin API context
 */
export interface PluginApiContext {
  /** Plugin ID */
  pluginId: string;

  /** API version */
  apiVersion: string;

  /** Granted permissions */
  permissions: string[];

  /** Plugin configuration */
  config: PluginConfig;
}

/**
 * Plugin system statistics
 */
export interface PluginSystemStats {
  /** Number of loaded plugins */
  loadedPlugins: number;

  /** Number of registered plugins */
  registeredPlugins: number;

  /** Number of enabled plugins */
  enabledPlugins: number;

  /** Number of running plugins */
  runningPlugins: number;

  /** Total downloads */
  totalDownloads: number;
}

/**
 * Plugin error
 */
export interface PluginError {
  code: string;
  message: string;
  pluginId?: string;
  details?: any;
}

/**
 * Notification levels
 */
export enum NotificationLevel {
  Info = 'Info',
  Warning = 'Warning',
  Error = 'Error',
  Success = 'Success',
}

/**
 * Dialog types
 */
export enum DialogType {
  Info = 'Info',
  Warning = 'Warning',
  Error = 'Error',
  Question = 'Question',
}

/**
 * Dialog configuration
 */
export interface DialogConfig {
  title: string;
  message: string;
  dialogType: DialogType;
  buttons: string[];
}

/**
 * HTTP request configuration
 */
export interface HttpRequestConfig {
  url: string;
  method: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
  headers?: Record<string, string>;
  body?: any;
  timeoutMs?: number;
}

/**
 * HTTP response
 */
export interface HttpResponse {
  status: number;
  headers: Record<string, string>;
  body: any;
}
