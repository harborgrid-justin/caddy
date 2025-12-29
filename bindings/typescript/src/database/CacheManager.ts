/**
 * CADDY Database - Client-Side Cache Manager
 *
 * Provides intelligent client-side caching with LRU eviction,
 * TTL support, and cache invalidation strategies.
 */

import { CacheConfig, CacheStats } from './types';

/**
 * Cache entry
 */
interface CacheEntry<T> {
  /** Cached value */
  value: T;

  /** Creation timestamp */
  createdAt: number;

  /** Time-to-live in milliseconds */
  ttl: number;

  /** Access count */
  accessCount: number;

  /** Last access timestamp */
  lastAccessed: number;

  /** Entry size in bytes (approximate) */
  size: number;
}

/**
 * Cache manager for client-side caching
 */
export class CacheManager {
  private cache: Map<string, CacheEntry<any>>;
  private config: Required<CacheConfig>;
  private stats: CacheStats;
  private cleanupInterval?: NodeJS.Timeout;

  constructor(config: Partial<CacheConfig> = {}) {
    this.config = {
      enableL1: config.enableL1 ?? true,
      l1Capacity: config.l1Capacity ?? 1000,
      l1Ttl: config.l1Ttl ?? 300000, // 5 minutes
      enableL2: config.enableL2 ?? false,
      l2Directory: config.l2Directory ?? './cache',
      l2MaxSize: config.l2MaxSize ?? 1024 * 1024 * 100, // 100MB
      enableL3: config.enableL3 ?? false,
      l3RedisUrl: config.l3RedisUrl ?? undefined,
      enableCompression: config.enableCompression ?? false,
      compressionThreshold: config.compressionThreshold ?? 1024, // 1KB
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

    // Start cleanup task
    this.startCleanupTask();
  }

  /**
   * Get a value from cache
   */
  async get<T>(key: string): Promise<T | null> {
    const startTime = performance.now();

    const entry = this.cache.get(key);

    if (!entry) {
      this.stats.l1Misses++;
      this.stats.totalMisses++;
      this.updateHitRate();
      return null;
    }

    // Check if expired
    if (this.isExpired(entry)) {
      this.cache.delete(key);
      this.stats.l1Misses++;
      this.stats.totalMisses++;
      this.updateHitRate();
      return null;
    }

    // Update access metadata
    entry.accessCount++;
    entry.lastAccessed = Date.now();

    this.stats.l1Hits++;
    this.stats.totalHits++;
    this.updateHitRate();

    const elapsedUs = (performance.now() - startTime) * 1000;
    this.stats.avgGetTimeUs = (this.stats.avgGetTimeUs + elapsedUs) / 2;

    return entry.value as T;
  }

  /**
   * Set a value in cache
   */
  async set<T>(key: string, value: T, ttl?: number): Promise<void> {
    const startTime = performance.now();

    // Enforce capacity limit
    if (this.cache.size >= this.config.l1Capacity) {
      this.evictLRU();
    }

    const size = this.estimateSize(value);

    const entry: CacheEntry<T> = {
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

  /**
   * Check if a key exists in cache
   */
  has(key: string): boolean {
    const entry = this.cache.get(key);
    if (!entry) return false;
    if (this.isExpired(entry)) {
      this.cache.delete(key);
      return false;
    }
    return true;
  }

  /**
   * Delete a key from cache
   */
  delete(key: string): boolean {
    return this.cache.delete(key);
  }

  /**
   * Clear all cache entries
   */
  clear(): void {
    this.cache.clear();
    this.stats.l1Size = 0;
  }

  /**
   * Get or set a value (with factory function)
   */
  async getOrSet<T>(
    key: string,
    factory: () => Promise<T>,
    ttl?: number
  ): Promise<T> {
    const cached = await this.get<T>(key);
    if (cached !== null) {
      return cached;
    }

    const value = await factory();
    await this.set(key, value, ttl);
    return value;
  }

  /**
   * Invalidate cache entries by pattern
   */
  invalidatePattern(pattern: string | RegExp): number {
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

  /**
   * Invalidate cache entries by tag
   */
  invalidateTag(tag: string): number {
    // Tags are encoded in keys as "tag:value:rest"
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

  /**
   * Get cache statistics
   */
  getStats(): CacheStats {
    return { ...this.stats };
  }

  /**
   * Reset statistics
   */
  resetStats(): void {
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

  /**
   * Get all cache keys
   */
  keys(): string[] {
    return Array.from(this.cache.keys());
  }

  /**
   * Get cache size
   */
  size(): number {
    return this.cache.size;
  }

  /**
   * Warm up cache with data
   */
  async warmUp<T>(data: Record<string, T>, ttl?: number): Promise<void> {
    for (const [key, value] of Object.entries(data)) {
      await this.set(key, value, ttl);
    }
  }

  /**
   * Export cache data
   */
  export(): Record<string, any> {
    const data: Record<string, any> = {};

    for (const [key, entry] of this.cache.entries()) {
      if (!this.isExpired(entry)) {
        data[key] = entry.value;
      }
    }

    return data;
  }

  /**
   * Import cache data
   */
  async import(data: Record<string, any>, ttl?: number): Promise<void> {
    for (const [key, value] of Object.entries(data)) {
      await this.set(key, value, ttl);
    }
  }

  /**
   * Dispose the cache manager
   */
  dispose(): void {
    if (this.cleanupInterval) {
      clearInterval(this.cleanupInterval);
    }
    this.clear();
  }

  // Private methods

  /**
   * Check if an entry is expired
   */
  private isExpired(entry: CacheEntry<any>): boolean {
    return Date.now() - entry.createdAt > entry.ttl;
  }

  /**
   * Evict least recently used entry
   */
  private evictLRU(): void {
    let oldestKey: string | null = null;
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

  /**
   * Estimate size of a value in bytes
   */
  private estimateSize(value: any): number {
    const json = JSON.stringify(value);
    return new Blob([json]).size;
  }

  /**
   * Update hit rate
   */
  private updateHitRate(): void {
    const total = this.stats.totalHits + this.stats.totalMisses;
    this.stats.hitRate = total > 0 ? this.stats.totalHits / total : 0;
  }

  /**
   * Start background cleanup task
   */
  private startCleanupTask(): void {
    // Run cleanup every minute
    this.cleanupInterval = setInterval(() => {
      this.cleanup();
    }, 60000);
  }

  /**
   * Cleanup expired entries
   */
  private cleanup(): void {
    const toDelete: string[] = [];

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

/**
 * Cache key builder utility
 */
export class CacheKeyBuilder {
  private parts: string[] = [];

  /**
   * Add a part to the cache key
   */
  part(value: string | number): this {
    this.parts.push(String(value));
    return this;
  }

  /**
   * Add a tag to the cache key
   */
  tag(tag: string): this {
    this.parts.unshift(tag);
    return this;
  }

  /**
   * Build the cache key
   */
  build(): string {
    return this.parts.join(':');
  }

  /**
   * Static factory method
   */
  static create(): CacheKeyBuilder {
    return new CacheKeyBuilder();
  }
}

/**
 * Decorator for caching function results
 */
export function cached(ttl: number = 300000) {
  return function (
    target: any,
    propertyKey: string,
    descriptor: PropertyDescriptor
  ) {
    const originalMethod = descriptor.value;
    const cache = new CacheManager();

    descriptor.value = async function (...args: any[]) {
      const key = `${propertyKey}:${JSON.stringify(args)}`;

      return cache.getOrSet(
        key,
        async () => originalMethod.apply(this, args),
        ttl
      );
    };

    return descriptor;
  };
}

/**
 * Global cache instance
 */
export const globalCache = new CacheManager();
