import React from 'react';
interface APIPortalProps {
    projectId?: string;
    onNavigate?: (view: PortalView) => void;
    config?: PortalConfig;
}
interface PortalConfig {
    title?: string;
    logo?: string;
    enableAnalytics?: boolean;
    enableTesting?: boolean;
    enableMocking?: boolean;
    showQuickStart?: boolean;
}
type PortalView = 'explorer' | 'documentation' | 'endpoints' | 'keys' | 'rate-limits' | 'analytics' | 'webhooks' | 'mocking' | 'versioning' | 'testing';
export declare const APIPortal: React.FC<APIPortalProps>;
export default APIPortal;
//# sourceMappingURL=APIPortal.d.ts.map