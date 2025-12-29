export declare enum CacheTier {
    L1 = "L1",
    L2 = "L2",
    L3 = "L3"
}
export interface CacheConfig {
    apiUrl: string;
    token?: string;
    defaultTtl?: number;
    enableCompression?: boolean;
    preferredTier?: CacheTier;
}
export interface CacheEntry<T = any> {
    key: string;
    value: T;
    tags?: string[];
    ttl?: number;
    tier?: CacheTier;
    createdAt?: string;
    expiresAt?: string;
}
export interface CacheStats {
    totalEntries: number;
    hitRate: number;
    missRate: number;
    memoryUsage: number;
    evictions: number;
}
export declare class CacheClient {
    private client;
    private config;
    constructor(config: CacheConfig);
    get<T = any>(key: string): Promise<T | null>;
    set<T = any>(key: string, value: T, options?: {
        ttl?: number;
        tags?: string[];
        tier?: CacheTier;
    }): Promise<void>;
    delete(key: string): Promise<boolean>;
    exists(key: string): Promise<boolean>;
    invalidateByTag(tag: string): Promise<number>;
    invalidateByPattern(pattern: string): Promise<number>;
    clear(): Promise<void>;
    getStats(): Promise<CacheStats>;
    lock(key: string, ttl?: number): Promise<string>;
    unlock(lockId: string): Promise<boolean>;
}
//# sourceMappingURL=cache.d.ts.map