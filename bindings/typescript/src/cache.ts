/**
 * Distributed Cache Client
 *
 * Provides TypeScript bindings for CADDY's distributed caching system
 * with multi-tier support (L1/L2/L3), tag-based invalidation, and
 * distributed locking.
 */

import axios, { AxiosInstance } from 'axios';

/**
 * Cache tier levels
 */
export enum CacheTier {
  /** In-memory cache (fastest) */
  L1 = 'L1',
  /** Local disk cache */
  L2 = 'L2',
  /** Distributed cache (Redis) */
  L3 = 'L3',
}

/**
 * Cache configuration
 */
export interface CacheConfig {
  /** API base URL */
  apiUrl: string;
  /** Authentication token */
  token?: string;
  /** Default TTL in seconds */
  defaultTtl?: number;
  /** Enable compression */
  enableCompression?: boolean;
  /** Preferred cache tier */
  preferredTier?: CacheTier;
}

/**
 * Cache entry metadata
 */
export interface CacheEntry<T = any> {
  /** Entry key */
  key: string;
  /** Entry value */
  value: T;
  /** Tags for invalidation */
  tags?: string[];
  /** Time to live in seconds */
  ttl?: number;
  /** Cache tier */
  tier?: CacheTier;
  /** Creation timestamp */
  createdAt?: string;
  /** Expiration timestamp */
  expiresAt?: string;
}

/**
 * Cache statistics
 */
export interface CacheStats {
  /** Total number of entries */
  totalEntries: number;
  /** Hit rate percentage */
  hitRate: number;
  /** Miss rate percentage */
  missRate: number;
  /** Memory usage in bytes */
  memoryUsage: number;
  /** Eviction count */
  evictions: number;
}

/**
 * Distributed cache client
 */
export class CacheClient {
  private client: AxiosInstance;
  private config: Required<CacheConfig>;

  constructor(config: CacheConfig) {
    this.config = {
      apiUrl: config.apiUrl,
      token: config.token || '',
      defaultTtl: config.defaultTtl || 3600,
      enableCompression: config.enableCompression ?? true,
      preferredTier: config.preferredTier || CacheTier.L1,
    };

    this.client = axios.create({
      baseURL: `${this.config.apiUrl}/api/cache`,
      headers: {
        'Authorization': `Bearer ${this.config.token}`,
        'Content-Type': 'application/json',
      },
      timeout: 10000,
    });
  }

  /**
   * Get a value from the cache
   */
  async get<T = any>(key: string): Promise<T | null> {
    try {
      const response = await this.client.get<CacheEntry<T>>(`/entries/${key}`);
      return response.data.value;
    } catch (error: any) {
      if (error.response?.status === 404) {
        return null;
      }
      throw new Error(`Cache get failed: ${error.message}`);
    }
  }

  /**
   * Set a value in the cache
   */
  async set<T = any>(
    key: string,
    value: T,
    options?: {
      ttl?: number;
      tags?: string[];
      tier?: CacheTier;
    }
  ): Promise<void> {
    const entry: CacheEntry<T> = {
      key,
      value,
      tags: options?.tags,
      ttl: options?.ttl || this.config.defaultTtl,
      tier: options?.tier || this.config.preferredTier,
    };

    await this.client.put(`/entries/${key}`, entry);
  }

  /**
   * Delete a value from the cache
   */
  async delete(key: string): Promise<boolean> {
    try {
      await this.client.delete(`/entries/${key}`);
      return true;
    } catch (error: any) {
      if (error.response?.status === 404) {
        return false;
      }
      throw new Error(`Cache delete failed: ${error.message}`);
    }
  }

  /**
   * Check if a key exists in the cache
   */
  async exists(key: string): Promise<boolean> {
    try {
      await this.client.head(`/entries/${key}`);
      return true;
    } catch (error) {
      return false;
    }
  }

  /**
   * Invalidate cache entries by tag
   */
  async invalidateByTag(tag: string): Promise<number> {
    const response = await this.client.post<{ count: number }>(
      '/invalidate/tag',
      { tag }
    );
    return response.data.count;
  }

  /**
   * Invalidate cache entries by pattern
   */
  async invalidateByPattern(pattern: string): Promise<number> {
    const response = await this.client.post<{ count: number }>(
      '/invalidate/pattern',
      { pattern }
    );
    return response.data.count;
  }

  /**
   * Clear all cache entries
   */
  async clear(): Promise<void> {
    await this.client.post('/clear');
  }

  /**
   * Get cache statistics
   */
  async getStats(): Promise<CacheStats> {
    const response = await this.client.get<CacheStats>('/stats');
    return response.data;
  }

  /**
   * Acquire a distributed lock
   */
  async lock(key: string, ttl: number = 30): Promise<string> {
    const response = await this.client.post<{ lockId: string }>(
      '/locks',
      { key, ttl }
    );
    return response.data.lockId;
  }

  /**
   * Release a distributed lock
   */
  async unlock(lockId: string): Promise<boolean> {
    try {
      await this.client.delete(`/locks/${lockId}`);
      return true;
    } catch (error) {
      return false;
    }
  }
}
