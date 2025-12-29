import { CacheConfig, CacheStats } from './types';
export declare class CacheManager {
    private cache;
    private config;
    private stats;
    private cleanupInterval?;
    constructor(config?: Partial<CacheConfig>);
    get<T>(key: string): Promise<T | null>;
    set<T>(key: string, value: T, ttl?: number): Promise<void>;
    has(key: string): boolean;
    delete(key: string): boolean;
    clear(): void;
    getOrSet<T>(key: string, factory: () => Promise<T>, ttl?: number): Promise<T>;
    invalidatePattern(pattern: string | RegExp): number;
    invalidateTag(tag: string): number;
    getStats(): CacheStats;
    resetStats(): void;
    keys(): string[];
    size(): number;
    warmUp<T>(data: Record<string, T>, ttl?: number): Promise<void>;
    export(): Record<string, any>;
    import(data: Record<string, any>, ttl?: number): Promise<void>;
    dispose(): void;
    private isExpired;
    private evictLRU;
    private estimateSize;
    private updateHitRate;
    private startCleanupTask;
    private cleanup;
}
export declare class CacheKeyBuilder {
    private parts;
    part(value: string | number): this;
    tag(tag: string): this;
    build(): string;
    static create(): CacheKeyBuilder;
}
export declare function cached(ttl?: number): (target: any, propertyKey: string, descriptor: PropertyDescriptor) => PropertyDescriptor;
export declare const globalCache: CacheManager;
//# sourceMappingURL=CacheManager.d.ts.map