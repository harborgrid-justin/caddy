export { CacheClient, CacheConfig, CacheEntry, CacheTier } from './cache';
export { TracingClient, TracingConfig, Span, SpanContext, TraceExporter } from './tracing';
export { TenantContext, TenantConfig, Tenant, TenantManager } from './tenant';
export { RateLimitClient, RateLimitConfig, RateLimitResult, QuotaInfo } from './ratelimit';
export { RealtimeClient, RealtimeConfig, CollaborationSession, DocumentUpdate } from './realtime';
export * from './dashboard';
export * from './users';
export * from './workflow';
export * from './files';
export * from './api-management';
export * from './monitoring';
export * from './settings';
export * from './reporting';
export * from './notifications';
export * from './audit';
export * from './types';
export interface EnterpriseConfig {
    apiUrl: string;
    apiToken?: string;
    licenseKey?: string;
    enableCache?: boolean;
    enableTracing?: boolean;
    enableMultiTenant?: boolean;
    enableRateLimit?: boolean;
    enableRealtime?: boolean;
    enableDashboard?: boolean;
    enableUserManagement?: boolean;
    enableWorkflow?: boolean;
    enableFileManagement?: boolean;
    enableAPIManagement?: boolean;
    enableMonitoring?: boolean;
    headers?: Record<string, string>;
    timeout?: number;
}
export declare class EnterpriseSDK {
    private config;
    constructor(config: EnterpriseConfig);
    getConfig(): Readonly<Required<EnterpriseConfig>>;
    updateConfig(updates: Partial<EnterpriseConfig>): void;
    validateLicense(): Promise<boolean>;
    getFeatureStatus(): Promise<Record<string, boolean>>;
    getSDKInfo(): {
        version: string;
        compatibleCaddyVersion: string;
        enabledFeatures: {
            cache: boolean;
            tracing: boolean;
            multiTenant: boolean;
            rateLimit: boolean;
            realtime: boolean;
            dashboard: boolean;
            userManagement: boolean;
            workflow: boolean;
            fileManagement: boolean;
            apiManagement: boolean;
            monitoring: boolean;
        };
    };
}
export declare const SDK_VERSION = "0.4.0";
export declare const COMPATIBLE_CADDY_VERSION = "0.4.0";
export declare const PLATFORM_INFO: {
    name: string;
    version: string;
    value: string;
    codeName: string;
    releaseDate: string;
    modules: string[];
};
//# sourceMappingURL=index.d.ts.map