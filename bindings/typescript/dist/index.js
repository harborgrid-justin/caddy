export { CacheClient, CacheTier } from './cache';
export { TracingClient, TraceExporter } from './tracing';
export { TenantContext, TenantManager } from './tenant';
export { RateLimitClient } from './ratelimit';
export { RealtimeClient } from './realtime';
export * from './dashboard';
export * from './users';
export * from './workflow';
export * from './files';
export * from './api-management';
export * from './monitoring';
export * from './settings';
export * from './reporting';
export * from './notifications';
export * from './audit';
export * from './types';
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
            enableDashboard: config.enableDashboard ?? true,
            enableUserManagement: config.enableUserManagement ?? true,
            enableWorkflow: config.enableWorkflow ?? true,
            enableFileManagement: config.enableFileManagement ?? true,
            enableAPIManagement: config.enableAPIManagement ?? true,
            enableMonitoring: config.enableMonitoring ?? true,
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
    getSDKInfo() {
        return {
            version: SDK_VERSION,
            compatibleCaddyVersion: COMPATIBLE_CADDY_VERSION,
            enabledFeatures: {
                cache: this.config.enableCache,
                tracing: this.config.enableTracing,
                multiTenant: this.config.enableMultiTenant,
                rateLimit: this.config.enableRateLimit,
                realtime: this.config.enableRealtime,
                dashboard: this.config.enableDashboard,
                userManagement: this.config.enableUserManagement,
                workflow: this.config.enableWorkflow,
                fileManagement: this.config.enableFileManagement,
                apiManagement: this.config.enableAPIManagement,
                monitoring: this.config.enableMonitoring,
            },
        };
    }
}
export const SDK_VERSION = '0.4.0';
export const COMPATIBLE_CADDY_VERSION = '0.4.0';
export const PLATFORM_INFO = {
    name: 'CADDY Enterprise Full-Stack Platform',
    version: '0.4.0',
    value: '$650M',
    codeName: 'Quantum',
    releaseDate: '2025-12-29',
    modules: [
        'Dashboard',
        'Users',
        'Workflow',
        'Files',
        'API Management',
        'Monitoring',
        'Settings',
        'Reporting',
        'Notifications',
        'Audit',
    ],
};
//# sourceMappingURL=index.js.map