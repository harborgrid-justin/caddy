import axios from 'axios';
export var CacheTier;
(function (CacheTier) {
    CacheTier["L1"] = "L1";
    CacheTier["L2"] = "L2";
    CacheTier["L3"] = "L3";
})(CacheTier || (CacheTier = {}));
export class CacheClient {
    constructor(config) {
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
    async get(key) {
        try {
            const response = await this.client.get(`/entries/${key}`);
            return response.data.value;
        }
        catch (error) {
            if (error.response?.status === 404) {
                return null;
            }
            throw new Error(`Cache get failed: ${error.message}`);
        }
    }
    async set(key, value, options) {
        const entry = {
            key,
            value,
            tags: options?.tags,
            ttl: options?.ttl || this.config.defaultTtl,
            tier: options?.tier || this.config.preferredTier,
        };
        await this.client.put(`/entries/${key}`, entry);
    }
    async delete(key) {
        try {
            await this.client.delete(`/entries/${key}`);
            return true;
        }
        catch (error) {
            if (error.response?.status === 404) {
                return false;
            }
            throw new Error(`Cache delete failed: ${error.message}`);
        }
    }
    async exists(key) {
        try {
            await this.client.head(`/entries/${key}`);
            return true;
        }
        catch (error) {
            return false;
        }
    }
    async invalidateByTag(tag) {
        const response = await this.client.post('/invalidate/tag', { tag });
        return response.data.count;
    }
    async invalidateByPattern(pattern) {
        const response = await this.client.post('/invalidate/pattern', { pattern });
        return response.data.count;
    }
    async clear() {
        await this.client.post('/clear');
    }
    async getStats() {
        const response = await this.client.get('/stats');
        return response.data;
    }
    async lock(key, ttl = 30) {
        const response = await this.client.post('/locks', { key, ttl });
        return response.data.lockId;
    }
    async unlock(lockId) {
        try {
            await this.client.delete(`/locks/${lockId}`);
            return true;
        }
        catch (error) {
            return false;
        }
    }
}
//# sourceMappingURL=cache.js.map