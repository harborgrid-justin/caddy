/**
 * CADDY Enterprise Full-Stack Platform - TypeScript SDK v0.4.0
 *
 * Comprehensive TypeScript bindings for CADDY enterprise features including:
 * - Dashboard analytics and visualization
 * - User management with RBAC
 * - Workflow automation engine
 * - File management and storage
 * - API management portal
 * - Monitoring and observability
 * - Settings and configuration
 * - Reporting and analytics
 * - Notifications system
 * - Audit logging
 * - Distributed caching
 * - Distributed tracing
 * - Multi-tenancy
 * - Rate limiting
 * - Real-time collaboration
 */

// ============================================================================
// Core Enterprise Features (v0.2.0 - v0.3.0)
// ============================================================================

export { CacheClient, CacheConfig, CacheEntry, CacheTier } from './cache';
export { TracingClient, TracingConfig, Span, SpanContext, TraceExporter } from './tracing';
export { TenantContext, TenantConfig, Tenant, TenantManager } from './tenant';
export { RateLimitClient, RateLimitConfig, RateLimitResult, QuotaInfo } from './ratelimit';
export { RealtimeClient, RealtimeConfig, CollaborationSession, DocumentUpdate } from './realtime';

// ============================================================================
// New Full-Stack Modules (v0.4.0)
// ============================================================================

// Dashboard Module
export * from './dashboard';

// User Management Module
export * from './users';

// Workflow Engine Module
export * from './workflow';

// File Management Module
export * from './files';

// API Management Module
export * from './api-management';

// Monitoring Module
export * from './monitoring';

// Settings Module
export * from './settings';

// Reporting Module
export * from './reporting';

// Notifications Module
export * from './notifications';

// Audit Module
export * from './audit';

// ============================================================================
// Shared Types
// ============================================================================

export * from './types';

// ============================================================================
// Enterprise SDK Configuration
// ============================================================================

/**
 * Enterprise SDK configuration
 */
export interface EnterpriseConfig {
  /** Base URL for CADDY enterprise API */
  apiUrl: string;

  /** API authentication token */
  apiToken?: string;

  /** Enterprise license key */
  licenseKey?: string;

  /** Enable distributed caching */
  enableCache?: boolean;

  /** Enable distributed tracing */
  enableTracing?: boolean;

  /** Enable multi-tenancy */
  enableMultiTenant?: boolean;

  /** Enable rate limiting */
  enableRateLimit?: boolean;

  /** Enable real-time collaboration */
  enableRealtime?: boolean;

  /** Enable dashboard */
  enableDashboard?: boolean;

  /** Enable user management */
  enableUserManagement?: boolean;

  /** Enable workflow engine */
  enableWorkflow?: boolean;

  /** Enable file management */
  enableFileManagement?: boolean;

  /** Enable API management */
  enableAPIManagement?: boolean;

  /** Enable monitoring */
  enableMonitoring?: boolean;

  /** Custom headers for API requests */
  headers?: Record<string, string>;

  /** Request timeout in milliseconds */
  timeout?: number;
}

/**
 * Main enterprise SDK client
 */
export class EnterpriseSDK {
  private config: Required<EnterpriseConfig>;

  constructor(config: EnterpriseConfig) {
    this.config = {
      apiUrl: config.apiUrl,
      apiToken: config.apiToken || '',
      licenseKey: config.licenseKey || '',
      enableCache: config.enableCache ?? true,
      enableTracing: config.enableTracing ?? true,
      enableMultiTenant: config.enableMultiTenant ?? false,
      enableRateLimit: config.enableRateLimit ?? true,
      enableRealtime: config.enableRealtime ?? true,
      enableDashboard: config.enableDashboard ?? true,
      enableUserManagement: config.enableUserManagement ?? true,
      enableWorkflow: config.enableWorkflow ?? true,
      enableFileManagement: config.enableFileManagement ?? true,
      enableAPIManagement: config.enableAPIManagement ?? true,
      enableMonitoring: config.enableMonitoring ?? true,
      headers: config.headers || {},
      timeout: config.timeout || 30000,
    };
  }

  /**
   * Get the current configuration
   */
  getConfig(): Readonly<Required<EnterpriseConfig>> {
    return { ...this.config };
  }

  /**
   * Update SDK configuration
   */
  updateConfig(updates: Partial<EnterpriseConfig>): void {
    this.config = { ...this.config, ...updates };
  }

  /**
   * Validate enterprise license
   */
  async validateLicense(): Promise<boolean> {
    try {
      const response = await fetch(`${this.config.apiUrl}/api/license/validate`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${this.config.apiToken}`,
          ...this.config.headers,
        },
        body: JSON.stringify({ licenseKey: this.config.licenseKey }),
      });

      const data = await response.json();
      return data.valid === true;
    } catch (error) {
      console.error('License validation failed:', error);
      return false;
    }
  }

  /**
   * Get enterprise feature status
   */
  async getFeatureStatus(): Promise<Record<string, boolean>> {
    try {
      const response = await fetch(`${this.config.apiUrl}/api/features/status`, {
        headers: {
          'Authorization': `Bearer ${this.config.apiToken}`,
          ...this.config.headers,
        },
      });

      return await response.json();
    } catch (error) {
      console.error('Failed to get feature status:', error);
      return {};
    }
  }

  /**
   * Get SDK information
   */
  getSDKInfo() {
    return {
      version: SDK_VERSION,
      compatibleCaddyVersion: COMPATIBLE_CADDY_VERSION,
      enabledFeatures: {
        cache: this.config.enableCache,
        tracing: this.config.enableTracing,
        multiTenant: this.config.enableMultiTenant,
        rateLimit: this.config.enableRateLimit,
        realtime: this.config.enableRealtime,
        dashboard: this.config.enableDashboard,
        userManagement: this.config.enableUserManagement,
        workflow: this.config.enableWorkflow,
        fileManagement: this.config.enableFileManagement,
        apiManagement: this.config.enableAPIManagement,
        monitoring: this.config.enableMonitoring,
      },
    };
  }
}

/**
 * SDK version information
 */
export const SDK_VERSION = '0.4.0';
export const COMPATIBLE_CADDY_VERSION = '0.4.0';

/**
 * Platform information
 */
export const PLATFORM_INFO = {
  name: 'CADDY Enterprise Full-Stack Platform',
  version: '0.4.0',
  value: '$650M',
  codeName: 'Quantum',
  releaseDate: '2025-12-29',
  modules: [
    'Dashboard',
    'Users',
    'Workflow',
    'Files',
    'API Management',
    'Monitoring',
    'Settings',
    'Reporting',
    'Notifications',
    'Audit',
  ],
};
