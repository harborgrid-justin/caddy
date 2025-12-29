export declare enum RateLimitAlgorithm {
    TokenBucket = "token_bucket",
    LeakyBucket = "leaky_bucket",
    SlidingWindow = "sliding_window",
    GCRA = "gcra"
}
export declare enum ThrottleAction {
    Reject = "reject",
    Delay = "delay",
    Degrade = "degrade",
    Queue = "queue"
}
export interface RateLimitConfig {
    apiUrl: string;
    token?: string;
    algorithm?: RateLimitAlgorithm;
    defaultAction?: ThrottleAction;
}
export interface RateLimitResult {
    allowed: boolean;
    remaining: number;
    limit: number;
    resetIn: number;
    retryAfter?: number;
}
export interface QuotaInfo {
    id: string;
    type: 'user' | 'api_key' | 'tenant' | 'ip';
    requestsPerPeriod: number;
    periodSeconds: number;
    currentUsage: number;
    resetAt: string;
}
export interface RateLimitViolation {
    id: string;
    identifier: string;
    type: string;
    timestamp: string;
    excessRequests: number;
}
export declare class RateLimitClient {
    private client;
    private config;
    constructor(config: RateLimitConfig);
    check(identifier: string, options?: {
        type?: 'user' | 'api_key' | 'tenant' | 'ip';
        cost?: number;
    }): Promise<RateLimitResult>;
    createQuota(identifier: string, type: 'user' | 'api_key' | 'tenant' | 'ip', requestsPerPeriod: number, periodSeconds: number): Promise<QuotaInfo>;
    getQuota(quotaId: string): Promise<QuotaInfo>;
    updateQuota(quotaId: string, updates: {
        requestsPerPeriod?: number;
        periodSeconds?: number;
    }): Promise<QuotaInfo>;
    deleteQuota(quotaId: string): Promise<void>;
    resetQuota(quotaId: string): Promise<void>;
    getViolations(options?: {
        identifier?: string;
        type?: string;
        since?: string;
        limit?: number;
    }): Promise<RateLimitViolation[]>;
    rateLimited<T>(identifier: string, fn: () => Promise<T>, options?: {
        type?: 'user' | 'api_key' | 'tenant' | 'ip';
        cost?: number;
        retryOnLimit?: boolean;
        maxRetries?: number;
    }): Promise<T>;
    getStats(identifier: string): Promise<{
        totalRequests: number;
        allowedRequests: number;
        rejectedRequests: number;
        averageUsage: number;
    }>;
}
//# sourceMappingURL=ratelimit.d.ts.map