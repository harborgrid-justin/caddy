export { CacheClient, CacheConfig, CacheEntry, CacheTier } from './cache';
export { TracingClient, TracingConfig, Span, SpanContext, TraceExporter } from './tracing';
export { TenantContext, TenantConfig, Tenant, TenantManager } from './tenant';
export { RateLimitClient, RateLimitConfig, RateLimitResult, QuotaInfo } from './ratelimit';
export { RealtimeClient, RealtimeConfig, CollaborationSession, DocumentUpdate } from './realtime';
export interface EnterpriseConfig {
    apiUrl: string;
    apiToken?: string;
    licenseKey?: string;
    enableCache?: boolean;
    enableTracing?: boolean;
    enableMultiTenant?: boolean;
    enableRateLimit?: boolean;
    enableRealtime?: boolean;
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
}
export declare const SDK_VERSION = "0.2.0";
export declare const COMPATIBLE_CADDY_VERSION = "0.2.0";
//# sourceMappingURL=index.d.ts.map