/**
 * Multi-Tenant Context Manager
 *
 * Provides TypeScript bindings for CADDY's multi-tenancy system
 * with tenant isolation, resource quotas, and billing integration.
 */

import axios, { AxiosInstance } from 'axios';

/**
 * Tenant configuration
 */
export interface TenantConfig {
  /** API base URL */
  apiUrl: string;
  /** Authentication token */
  token?: string;
  /** Default tenant ID */
  defaultTenantId?: string;
}

/**
 * Tenant information
 */
export interface Tenant {
  /** Tenant ID */
  id: string;
  /** Tenant name */
  name: string;
  /** Tenant status */
  status: 'active' | 'suspended' | 'inactive';
  /** Resource quotas */
  quotas: TenantQuotas;
  /** Custom settings */
  settings: Record<string, any>;
  /** Creation timestamp */
  createdAt: string;
  /** Last updated timestamp */
  updatedAt: string;
}

/**
 * Tenant resource quotas
 */
export interface TenantQuotas {
  /** Maximum storage in bytes */
  maxStorage: number;
  /** Maximum API requests per hour */
  maxApiRequests: number;
  /** Maximum concurrent users */
  maxConcurrentUsers: number;
  /** Maximum projects */
  maxProjects: number;
}

/**
 * Tenant usage statistics
 */
export interface TenantUsage {
  /** Current storage used in bytes */
  storageUsed: number;
  /** API requests in current hour */
  apiRequestsThisHour: number;
  /** Current concurrent users */
  concurrentUsers: number;
  /** Total projects */
  projectCount: number;
}

/**
 * Tenant context
 */
export class TenantContext {
  private tenantId: string | null = null;
  private metadata: Map<string, any> = new Map();

  /**
   * Set the current tenant ID
   */
  setTenantId(tenantId: string): void {
    this.tenantId = tenantId;
  }

  /**
   * Get the current tenant ID
   */
  getTenantId(): string | null {
    return this.tenantId;
  }

  /**
   * Clear the tenant context
   */
  clear(): void {
    this.tenantId = null;
    this.metadata.clear();
  }

  /**
   * Set context metadata
   */
  setMetadata(key: string, value: any): void {
    this.metadata.set(key, value);
  }

  /**
   * Get context metadata
   */
  getMetadata(key: string): any {
    return this.metadata.get(key);
  }

  /**
   * Get all metadata
   */
  getAllMetadata(): Record<string, any> {
    return Object.fromEntries(this.metadata);
  }
}

/**
 * Tenant manager
 */
export class TenantManager {
  private client: AxiosInstance;
  private config: Required<TenantConfig>;
  private context: TenantContext;

  constructor(config: TenantConfig) {
    this.config = {
      apiUrl: config.apiUrl,
      token: config.token || '',
      defaultTenantId: config.defaultTenantId || '',
    };

    this.client = axios.create({
      baseURL: `${this.config.apiUrl}/api/tenants`,
      headers: {
        'Authorization': `Bearer ${this.config.token}`,
        'Content-Type': 'application/json',
      },
      timeout: 10000,
    });

    this.context = new TenantContext();
    if (this.config.defaultTenantId) {
      this.context.setTenantId(this.config.defaultTenantId);
    }
  }

  /**
   * Get the tenant context
   */
  getContext(): TenantContext {
    return this.context;
  }

  /**
   * Create a new tenant
   */
  async createTenant(
    name: string,
    quotas?: Partial<TenantQuotas>,
    settings?: Record<string, any>
  ): Promise<Tenant> {
    const response = await this.client.post<Tenant>('/', {
      name,
      quotas,
      settings: settings || {},
    });
    return response.data;
  }

  /**
   * Get tenant by ID
   */
  async getTenant(tenantId: string): Promise<Tenant> {
    const response = await this.client.get<Tenant>(`/${tenantId}`);
    return response.data;
  }

  /**
   * Update tenant
   */
  async updateTenant(
    tenantId: string,
    updates: {
      name?: string;
      status?: 'active' | 'suspended' | 'inactive';
      quotas?: Partial<TenantQuotas>;
      settings?: Record<string, any>;
    }
  ): Promise<Tenant> {
    const response = await this.client.patch<Tenant>(`/${tenantId}`, updates);
    return response.data;
  }

  /**
   * Delete tenant
   */
  async deleteTenant(tenantId: string): Promise<void> {
    await this.client.delete(`/${tenantId}`);
  }

  /**
   * List all tenants
   */
  async listTenants(
    options?: {
      status?: 'active' | 'suspended' | 'inactive';
      limit?: number;
      offset?: number;
    }
  ): Promise<Tenant[]> {
    const response = await this.client.get<Tenant[]>('/', { params: options });
    return response.data;
  }

  /**
   * Get tenant usage statistics
   */
  async getTenantUsage(tenantId: string): Promise<TenantUsage> {
    const response = await this.client.get<TenantUsage>(`/${tenantId}/usage`);
    return response.data;
  }

  /**
   * Check if tenant has exceeded quotas
   */
  async checkQuotas(tenantId: string): Promise<{
    withinLimits: boolean;
    violations: string[];
  }> {
    const response = await this.client.get<{
      withinLimits: boolean;
      violations: string[];
    }>(`/${tenantId}/quotas/check`);
    return response.data;
  }

  /**
   * Execute a function within a tenant context
   */
  async withTenant<T>(tenantId: string, fn: () => Promise<T>): Promise<T> {
    const previousTenantId = this.context.getTenantId();
    this.context.setTenantId(tenantId);

    try {
      return await fn();
    } finally {
      if (previousTenantId) {
        this.context.setTenantId(previousTenantId);
      } else {
        this.context.clear();
      }
    }
  }
}
