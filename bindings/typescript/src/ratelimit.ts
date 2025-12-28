/**
 * Rate Limiting Client
 *
 * Provides TypeScript bindings for CADDY's rate limiting system
 * with multiple algorithms, distributed coordination, and quota management.
 */

import axios, { AxiosInstance } from 'axios';

/**
 * Rate limiting algorithm
 */
export enum RateLimitAlgorithm {
  /** Token bucket algorithm */
  TokenBucket = 'token_bucket',
  /** Leaky bucket algorithm */
  LeakyBucket = 'leaky_bucket',
  /** Sliding window algorithm */
  SlidingWindow = 'sliding_window',
  /** Generic Cell Rate Algorithm */
  GCRA = 'gcra',
}

/**
 * Throttling action
 */
export enum ThrottleAction {
  /** Reject the request */
  Reject = 'reject',
  /** Delay the request */
  Delay = 'delay',
  /** Degrade service quality */
  Degrade = 'degrade',
  /** Add to priority queue */
  Queue = 'queue',
}

/**
 * Rate limit configuration
 */
export interface RateLimitConfig {
  /** API base URL */
  apiUrl: string;
  /** Authentication token */
  token?: string;
  /** Rate limit algorithm */
  algorithm?: RateLimitAlgorithm;
  /** Default throttle action */
  defaultAction?: ThrottleAction;
}

/**
 * Rate limit result
 */
export interface RateLimitResult {
  /** Whether the request is allowed */
  allowed: boolean;
  /** Remaining quota */
  remaining: number;
  /** Total limit */
  limit: number;
  /** Time until reset (seconds) */
  resetIn: number;
  /** Retry after (seconds, if not allowed) */
  retryAfter?: number;
}

/**
 * Quota information
 */
export interface QuotaInfo {
  /** Quota identifier */
  id: string;
  /** Quota type (user, api_key, tenant, ip) */
  type: 'user' | 'api_key' | 'tenant' | 'ip';
  /** Requests per period */
  requestsPerPeriod: number;
  /** Period in seconds */
  periodSeconds: number;
  /** Current usage */
  currentUsage: number;
  /** Time until reset */
  resetAt: string;
}

/**
 * Rate limit violation
 */
export interface RateLimitViolation {
  /** Violation ID */
  id: string;
  /** Identifier that violated the limit */
  identifier: string;
  /** Violation type */
  type: string;
  /** Timestamp */
  timestamp: string;
  /** Number of excess requests */
  excessRequests: number;
}

/**
 * Rate limiting client
 */
export class RateLimitClient {
  private client: AxiosInstance;
  private config: Required<RateLimitConfig>;

  constructor(config: RateLimitConfig) {
    this.config = {
      apiUrl: config.apiUrl,
      token: config.token || '',
      algorithm: config.algorithm || RateLimitAlgorithm.TokenBucket,
      defaultAction: config.defaultAction || ThrottleAction.Reject,
    };

    this.client = axios.create({
      baseURL: `${this.config.apiUrl}/api/ratelimit`,
      headers: {
        'Authorization': `Bearer ${this.config.token}`,
        'Content-Type': 'application/json',
      },
      timeout: 10000,
    });
  }

  /**
   * Check if a request is allowed
   */
  async check(
    identifier: string,
    options?: {
      type?: 'user' | 'api_key' | 'tenant' | 'ip';
      cost?: number;
    }
  ): Promise<RateLimitResult> {
    const response = await this.client.post<RateLimitResult>('/check', {
      identifier,
      type: options?.type || 'user',
      cost: options?.cost || 1,
      algorithm: this.config.algorithm,
    });
    return response.data;
  }

  /**
   * Create a new quota
   */
  async createQuota(
    identifier: string,
    type: 'user' | 'api_key' | 'tenant' | 'ip',
    requestsPerPeriod: number,
    periodSeconds: number
  ): Promise<QuotaInfo> {
    const response = await this.client.post<QuotaInfo>('/quotas', {
      identifier,
      type,
      requestsPerPeriod,
      periodSeconds,
    });
    return response.data;
  }

  /**
   * Get quota information
   */
  async getQuota(quotaId: string): Promise<QuotaInfo> {
    const response = await this.client.get<QuotaInfo>(`/quotas/${quotaId}`);
    return response.data;
  }

  /**
   * Update quota
   */
  async updateQuota(
    quotaId: string,
    updates: {
      requestsPerPeriod?: number;
      periodSeconds?: number;
    }
  ): Promise<QuotaInfo> {
    const response = await this.client.patch<QuotaInfo>(
      `/quotas/${quotaId}`,
      updates
    );
    return response.data;
  }

  /**
   * Delete quota
   */
  async deleteQuota(quotaId: string): Promise<void> {
    await this.client.delete(`/quotas/${quotaId}`);
  }

  /**
   * Reset quota usage
   */
  async resetQuota(quotaId: string): Promise<void> {
    await this.client.post(`/quotas/${quotaId}/reset`);
  }

  /**
   * Get rate limit violations
   */
  async getViolations(
    options?: {
      identifier?: string;
      type?: string;
      since?: string;
      limit?: number;
    }
  ): Promise<RateLimitViolation[]> {
    const response = await this.client.get<RateLimitViolation[]>(
      '/violations',
      { params: options }
    );
    return response.data;
  }

  /**
   * Execute a rate-limited function
   */
  async rateLimited<T>(
    identifier: string,
    fn: () => Promise<T>,
    options?: {
      type?: 'user' | 'api_key' | 'tenant' | 'ip';
      cost?: number;
      retryOnLimit?: boolean;
      maxRetries?: number;
    }
  ): Promise<T> {
    const maxRetries = options?.maxRetries || 3;
    let attempt = 0;

    while (attempt <= maxRetries) {
      const result = await this.check(identifier, {
        type: options?.type,
        cost: options?.cost,
      });

      if (result.allowed) {
        return await fn();
      }

      if (!options?.retryOnLimit || attempt >= maxRetries) {
        throw new Error(
          `Rate limit exceeded. Retry after ${result.retryAfter} seconds.`
        );
      }

      // Wait before retrying
      if (result.retryAfter) {
        await new Promise(resolve => setTimeout(resolve, result.retryAfter * 1000));
      }

      attempt++;
    }

    throw new Error('Rate limit exceeded after maximum retries');
  }

  /**
   * Get rate limit statistics
   */
  async getStats(identifier: string): Promise<{
    totalRequests: number;
    allowedRequests: number;
    rejectedRequests: number;
    averageUsage: number;
  }> {
    const response = await this.client.get(`/stats/${identifier}`);
    return response.data;
  }
}
