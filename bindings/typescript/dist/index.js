export { CacheClient, CacheTier } from './cache';
export { TracingClient, TraceExporter } from './tracing';
export { TenantContext, TenantManager } from './tenant';
export { RateLimitClient } from './ratelimit';
export { RealtimeClient } from './realtime';
export class EnterpriseSDK {
    constructor(config) {
        this.config = {
            apiUrl: config.apiUrl,
            apiToken: config.apiToken || '',
            licenseKey: config.licenseKey || '',
            enableCache: config.enableCache ?? true,
            enableTracing: config.enableTracing ?? true,
            enableMultiTenant: config.enableMultiTenant ?? false,
            enableRateLimit: config.enableRateLimit ?? true,
            enableRealtime: config.enableRealtime ?? true,
            headers: config.headers || {},
            timeout: config.timeout || 30000,
        };
    }
    getConfig() {
        return { ...this.config };
    }
    updateConfig(updates) {
        this.config = { ...this.config, ...updates };
    }
    async validateLicense() {
        try {
            const response = await fetch(`${this.config.apiUrl}/api/license/validate`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${this.config.apiToken}`,
                    ...this.config.headers,
                },
                body: JSON.stringify({ licenseKey: this.config.licenseKey }),
            });
            const data = await response.json();
            return data.valid === true;
        }
        catch (error) {
            console.error('License validation failed:', error);
            return false;
        }
    }
    async getFeatureStatus() {
        try {
            const response = await fetch(`${this.config.apiUrl}/api/features/status`, {
                headers: {
                    'Authorization': `Bearer ${this.config.apiToken}`,
                    ...this.config.headers,
                },
            });
            return await response.json();
        }
        catch (error) {
            console.error('Failed to get feature status:', error);
            return {};
        }
    }
}
export const SDK_VERSION = '0.2.0';
export const COMPATIBLE_CADDY_VERSION = '0.2.0';
//# sourceMappingURL=index.js.map