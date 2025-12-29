export interface TenantConfig {
    apiUrl: string;
    token?: string;
    defaultTenantId?: string;
}
export interface Tenant {
    id: string;
    name: string;
    status: 'active' | 'suspended' | 'inactive';
    quotas: TenantQuotas;
    settings: Record<string, any>;
    createdAt: string;
    updatedAt: string;
}
export interface TenantQuotas {
    maxStorage: number;
    maxApiRequests: number;
    maxConcurrentUsers: number;
    maxProjects: number;
}
export interface TenantUsage {
    storageUsed: number;
    apiRequestsThisHour: number;
    concurrentUsers: number;
    projectCount: number;
}
export declare class TenantContext {
    private tenantId;
    private metadata;
    setTenantId(tenantId: string): void;
    getTenantId(): string | null;
    clear(): void;
    setMetadata(key: string, value: any): void;
    getMetadata(key: string): any;
    getAllMetadata(): Record<string, any>;
}
export declare class TenantManager {
    private client;
    private config;
    private context;
    constructor(config: TenantConfig);
    getContext(): TenantContext;
    createTenant(name: string, quotas?: Partial<TenantQuotas>, settings?: Record<string, any>): Promise<Tenant>;
    getTenant(tenantId: string): Promise<Tenant>;
    updateTenant(tenantId: string, updates: {
        name?: string;
        status?: 'active' | 'suspended' | 'inactive';
        quotas?: Partial<TenantQuotas>;
        settings?: Record<string, any>;
    }): Promise<Tenant>;
    deleteTenant(tenantId: string): Promise<void>;
    listTenants(options?: {
        status?: 'active' | 'suspended' | 'inactive';
        limit?: number;
        offset?: number;
    }): Promise<Tenant[]>;
    getTenantUsage(tenantId: string): Promise<TenantUsage>;
    checkQuotas(tenantId: string): Promise<{
        withinLimits: boolean;
        violations: string[];
    }>;
    withTenant<T>(tenantId: string, fn: () => Promise<T>): Promise<T>;
}
//# sourceMappingURL=tenant.d.ts.map