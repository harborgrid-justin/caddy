/**
 * CADDY Enterprise Edition - TypeScript SDK v0.2.0
 *
 * Comprehensive TypeScript bindings for CADDY enterprise features including
 * distributed caching, tracing, multi-tenancy, rate limiting, and real-time collaboration.
 */

export { CacheClient, CacheConfig, CacheEntry, CacheTier } from './cache';
export { TracingClient, TracingConfig, Span, SpanContext, TraceExporter } from './tracing';
export { TenantContext, TenantConfig, Tenant, TenantManager } from './tenant';
export { RateLimitClient, RateLimitConfig, RateLimitResult, QuotaInfo } from './ratelimit';
export { RealtimeClient, RealtimeConfig, CollaborationSession, DocumentUpdate } from './realtime';

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
}

/**
 * SDK version information
 */
export const SDK_VERSION = '0.2.0';
export const COMPATIBLE_CADDY_VERSION = '0.2.0';
