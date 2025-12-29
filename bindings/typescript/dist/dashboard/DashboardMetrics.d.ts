import React from 'react';
import type { MetricData } from './types';
export interface MetricCardProps {
    metric: MetricData;
    showSparkline?: boolean;
    showComparison?: boolean;
    showProgress?: boolean;
    size?: 'small' | 'medium' | 'large';
    onClick?: (metric: MetricData) => void;
    className?: string;
    isLoading?: boolean;
}
export declare const MetricCard: React.FC<MetricCardProps>;
export interface MetricsGridProps {
    metrics: MetricData[];
    columns?: {
        xs: number;
        sm: number;
        md: number;
        lg: number;
        xl: number;
    };
    showSparklines?: boolean;
    showComparisons?: boolean;
    showProgress?: boolean;
    cardSize?: 'small' | 'medium' | 'large';
    onMetricClick?: (metric: MetricData) => void;
    refreshInterval?: number;
    onRefresh?: () => Promise<MetricData[]>;
    className?: string;
}
export declare const MetricsGrid: React.FC<MetricsGridProps>;
export default MetricsGrid;
//# sourceMappingURL=DashboardMetrics.d.ts.map