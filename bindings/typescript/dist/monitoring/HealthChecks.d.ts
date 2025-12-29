import React from 'react';
import { ServiceHealth } from './types';
interface HealthChecksProps {
    services?: string[];
    autoRefresh?: boolean;
    refreshInterval?: number;
    onServiceClick?: (service: ServiceHealth) => void;
    className?: string;
}
export declare const HealthChecks: React.FC<HealthChecksProps>;
export default HealthChecks;
//# sourceMappingURL=HealthChecks.d.ts.map