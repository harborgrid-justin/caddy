import axios from 'axios';
export class TenantContext {
    constructor() {
        this.tenantId = null;
        this.metadata = new Map();
    }
    setTenantId(tenantId) {
        this.tenantId = tenantId;
    }
    getTenantId() {
        return this.tenantId;
    }
    clear() {
        this.tenantId = null;
        this.metadata.clear();
    }
    setMetadata(key, value) {
        this.metadata.set(key, value);
    }
    getMetadata(key) {
        return this.metadata.get(key);
    }
    getAllMetadata() {
        return Object.fromEntries(this.metadata);
    }
}
export class TenantManager {
    constructor(config) {
        this.config = {
            apiUrl: config.apiUrl,
            token: config.token || '',
            defaultTenantId: config.defaultTenantId || '',
        };
        this.client = axios.create({
            baseURL: `${this.config.apiUrl}/api/tenants`,
            headers: {
                'Authorization': `Bearer ${this.config.token}`,
                'Content-Type': 'application/json',
            },
            timeout: 10000,
        });
        this.context = new TenantContext();
        if (this.config.defaultTenantId) {
            this.context.setTenantId(this.config.defaultTenantId);
        }
    }
    getContext() {
        return this.context;
    }
    async createTenant(name, quotas, settings) {
        const response = await this.client.post('/', {
            name,
            quotas,
            settings: settings || {},
        });
        return response.data;
    }
    async getTenant(tenantId) {
        const response = await this.client.get(`/${tenantId}`);
        return response.data;
    }
    async updateTenant(tenantId, updates) {
        const response = await this.client.patch(`/${tenantId}`, updates);
        return response.data;
    }
    async deleteTenant(tenantId) {
        await this.client.delete(`/${tenantId}`);
    }
    async listTenants(options) {
        const response = await this.client.get('/', { params: options });
        return response.data;
    }
    async getTenantUsage(tenantId) {
        const response = await this.client.get(`/${tenantId}/usage`);
        return response.data;
    }
    async checkQuotas(tenantId) {
        const response = await this.client.get(`/${tenantId}/quotas/check`);
        return response.data;
    }
    async withTenant(tenantId, fn) {
        const previousTenantId = this.context.getTenantId();
        this.context.setTenantId(tenantId);
        try {
            return await fn();
        }
        finally {
            if (previousTenantId) {
                this.context.setTenantId(previousTenantId);
            }
            else {
                this.context.clear();
            }
        }
    }
}
//# sourceMappingURL=tenant.js.map