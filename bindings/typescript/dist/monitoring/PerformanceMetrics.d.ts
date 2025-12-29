import React from 'react';
import { MetricType, TimeRange } from './types';
interface PerformanceMetricsProps {
    service?: string;
    metrics?: MetricType[];
    timeRange?: TimeRange;
    refreshInterval?: number;
    className?: string;
}
export declare const PerformanceMetrics: React.FC<PerformanceMetricsProps>;
export default PerformanceMetrics;
//# sourceMappingURL=PerformanceMetrics.d.ts.map