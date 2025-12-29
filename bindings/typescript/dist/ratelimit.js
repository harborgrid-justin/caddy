import axios from 'axios';
export var RateLimitAlgorithm;
(function (RateLimitAlgorithm) {
    RateLimitAlgorithm["TokenBucket"] = "token_bucket";
    RateLimitAlgorithm["LeakyBucket"] = "leaky_bucket";
    RateLimitAlgorithm["SlidingWindow"] = "sliding_window";
    RateLimitAlgorithm["GCRA"] = "gcra";
})(RateLimitAlgorithm || (RateLimitAlgorithm = {}));
export var ThrottleAction;
(function (ThrottleAction) {
    ThrottleAction["Reject"] = "reject";
    ThrottleAction["Delay"] = "delay";
    ThrottleAction["Degrade"] = "degrade";
    ThrottleAction["Queue"] = "queue";
})(ThrottleAction || (ThrottleAction = {}));
export class RateLimitClient {
    constructor(config) {
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
    async check(identifier, options) {
        const response = await this.client.post('/check', {
            identifier,
            type: options?.type || 'user',
            cost: options?.cost || 1,
            algorithm: this.config.algorithm,
        });
        return response.data;
    }
    async createQuota(identifier, type, requestsPerPeriod, periodSeconds) {
        const response = await this.client.post('/quotas', {
            identifier,
            type,
            requestsPerPeriod,
            periodSeconds,
        });
        return response.data;
    }
    async getQuota(quotaId) {
        const response = await this.client.get(`/quotas/${quotaId}`);
        return response.data;
    }
    async updateQuota(quotaId, updates) {
        const response = await this.client.patch(`/quotas/${quotaId}`, updates);
        return response.data;
    }
    async deleteQuota(quotaId) {
        await this.client.delete(`/quotas/${quotaId}`);
    }
    async resetQuota(quotaId) {
        await this.client.post(`/quotas/${quotaId}/reset`);
    }
    async getViolations(options) {
        const response = await this.client.get('/violations', { params: options });
        return response.data;
    }
    async rateLimited(identifier, fn, options) {
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
                throw new Error(`Rate limit exceeded. Retry after ${result.retryAfter} seconds.`);
            }
            if (result.retryAfter) {
                await new Promise(resolve => setTimeout(resolve, result.retryAfter * 1000));
            }
            attempt++;
        }
        throw new Error('Rate limit exceeded after maximum retries');
    }
    async getStats(identifier) {
        const response = await this.client.get(`/stats/${identifier}`);
        return response.data;
    }
}
//# sourceMappingURL=ratelimit.js.map