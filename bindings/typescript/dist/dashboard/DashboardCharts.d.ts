import React from 'react';
import type { ChartData, ChartType, DataPoint } from './types';
export interface ChartProps {
    data: ChartData;
    type?: ChartType;
    height?: number;
    width?: string | number;
    interactive?: boolean;
    onClick?: (dataPoint: DataPoint, datasetIndex: number) => void;
    className?: string;
    isLoading?: boolean;
}
export declare const Chart: React.FC<ChartProps>;
export default Chart;
//# sourceMappingURL=DashboardCharts.d.ts.map