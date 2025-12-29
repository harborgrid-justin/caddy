export class CacheManager {
    constructor(config = {}) {
        this.config = {
            enableL1: config.enableL1 ?? true,
            l1Capacity: config.l1Capacity ?? 1000,
            l1Ttl: config.l1Ttl ?? 300000,
            enableL2: config.enableL2 ?? false,
            l2Directory: config.l2Directory ?? './cache',
            l2MaxSize: config.l2MaxSize ?? 1024 * 1024 * 100,
            enableL3: config.enableL3 ?? false,
            l3RedisUrl: config.l3RedisUrl ?? '',
            enableCompression: config.enableCompression ?? false,
            compressionThreshold: config.compressionThreshold ?? 1024,
        };
        this.cache = new Map();
        this.stats = {
            l1Hits: 0,
            l1Misses: 0,
            l1Size: 0,
            l2Hits: 0,
            l2Misses: 0,
            l2Size: 0,
            l3Hits: 0,
            l3Misses: 0,
            totalHits: 0,
            totalMisses: 0,
            hitRate: 0,
            avgGetTimeUs: 0,
            avgSetTimeUs: 0,
        };
        this.startCleanupTask();
    }
    async get(key) {
        const startTime = performance.now();
        const entry = this.cache.get(key);
        if (!entry) {
            this.stats.l1Misses++;
            this.stats.totalMisses++;
            this.updateHitRate();
            return null;
        }
        if (this.isExpired(entry)) {
            this.cache.delete(key);
            this.stats.l1Misses++;
            this.stats.totalMisses++;
            this.updateHitRate();
            return null;
        }
        entry.accessCount++;
        entry.lastAccessed = Date.now();
        this.stats.l1Hits++;
        this.stats.totalHits++;
        this.updateHitRate();
        const elapsedUs = (performance.now() - startTime) * 1000;
        this.stats.avgGetTimeUs = (this.stats.avgGetTimeUs + elapsedUs) / 2;
        return entry.value;
    }
    async set(key, value, ttl) {
        const startTime = performance.now();
        if (this.cache.size >= this.config.l1Capacity) {
            this.evictLRU();
        }
        const size = this.estimateSize(value);
        const entry = {
            value,
            createdAt: Date.now(),
            ttl: ttl ?? this.config.l1Ttl,
            accessCount: 0,
            lastAccessed: Date.now(),
            size,
        };
        this.cache.set(key, entry);
        this.stats.l1Size = this.cache.size;
        const elapsedUs = (performance.now() - startTime) * 1000;
        this.stats.avgSetTimeUs = (this.stats.avgSetTimeUs + elapsedUs) / 2;
    }
    has(key) {
        const entry = this.cache.get(key);
        if (!entry)
            return false;
        if (this.isExpired(entry)) {
            this.cache.delete(key);
            return false;
        }
        return true;
    }
    delete(key) {
        return this.cache.delete(key);
    }
    clear() {
        this.cache.clear();
        this.stats.l1Size = 0;
    }
    async getOrSet(key, factory, ttl) {
        const cached = await this.get(key);
        if (cached !== null) {
            return cached;
        }
        const value = await factory();
        await this.set(key, value, ttl);
        return value;
    }
    invalidatePattern(pattern) {
        const regex = typeof pattern === 'string' ? new RegExp(pattern) : pattern;
        let count = 0;
        for (const key of this.cache.keys()) {
            if (regex.test(key)) {
                this.cache.delete(key);
                count++;
            }
        }
        this.stats.l1Size = this.cache.size;
        return count;
    }
    invalidateTag(tag) {
        const prefix = `${tag}:`;
        let count = 0;
        for (const key of this.cache.keys()) {
            if (key.startsWith(prefix)) {
                this.cache.delete(key);
                count++;
            }
        }
        this.stats.l1Size = this.cache.size;
        return count;
    }
    getStats() {
        return { ...this.stats };
    }
    resetStats() {
        this.stats = {
            l1Hits: 0,
            l1Misses: 0,
            l1Size: this.cache.size,
            l2Hits: 0,
            l2Misses: 0,
            l2Size: 0,
            l3Hits: 0,
            l3Misses: 0,
            totalHits: 0,
            totalMisses: 0,
            hitRate: 0,
            avgGetTimeUs: 0,
            avgSetTimeUs: 0,
        };
    }
    keys() {
        return Array.from(this.cache.keys());
    }
    size() {
        return this.cache.size;
    }
    async warmUp(data, ttl) {
        for (const [key, value] of Object.entries(data)) {
            await this.set(key, value, ttl);
        }
    }
    export() {
        const data = {};
        for (const [key, entry] of this.cache.entries()) {
            if (!this.isExpired(entry)) {
                data[key] = entry.value;
            }
        }
        return data;
    }
    async import(data, ttl) {
        for (const [key, value] of Object.entries(data)) {
            await this.set(key, value, ttl);
        }
    }
    dispose() {
        if (this.cleanupInterval) {
            clearInterval(this.cleanupInterval);
        }
        this.clear();
    }
    isExpired(entry) {
        return Date.now() - entry.createdAt > entry.ttl;
    }
    evictLRU() {
        let oldestKey = null;
        let oldestTime = Infinity;
        for (const [key, entry] of this.cache.entries()) {
            if (entry.lastAccessed < oldestTime) {
                oldestTime = entry.lastAccessed;
                oldestKey = key;
            }
        }
        if (oldestKey) {
            this.cache.delete(oldestKey);
        }
    }
    estimateSize(value) {
        const json = JSON.stringify(value);
        return new Blob([json]).size;
    }
    updateHitRate() {
        const total = this.stats.totalHits + this.stats.totalMisses;
        this.stats.hitRate = total > 0 ? this.stats.totalHits / total : 0;
    }
    startCleanupTask() {
        this.cleanupInterval = setInterval(() => {
            this.cleanup();
        }, 60000);
    }
    cleanup() {
        const toDelete = [];
        for (const [key, entry] of this.cache.entries()) {
            if (this.isExpired(entry)) {
                toDelete.push(key);
            }
        }
        for (const key of toDelete) {
            this.cache.delete(key);
        }
        if (toDelete.length > 0) {
            this.stats.l1Size = this.cache.size;
        }
    }
}
export class CacheKeyBuilder {
    constructor() {
        this.parts = [];
    }
    part(value) {
        this.parts.push(String(value));
        return this;
    }
    tag(tag) {
        this.parts.unshift(tag);
        return this;
    }
    build() {
        return this.parts.join(':');
    }
    static create() {
        return new CacheKeyBuilder();
    }
}
export function cached(ttl = 300000) {
    return function (target, propertyKey, descriptor) {
        const originalMethod = descriptor.value;
        const cache = new CacheManager();
        descriptor.value = async function (...args) {
            const key = `${propertyKey}:${JSON.stringify(args)}`;
            return cache.getOrSet(key, async () => originalMethod.apply(this, args), ttl);
        };
        return descriptor;
    };
}
export const globalCache = new CacheManager();
//# sourceMappingURL=CacheManager.js.map