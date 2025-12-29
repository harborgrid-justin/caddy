import React from 'react';
import { ChartConfig, SelectField } from './types';
export interface ReportChartsProps {
    config: ChartConfig;
    availableFields: SelectField[];
    onChange: (config: ChartConfig) => void;
    readOnly?: boolean;
    showPreview?: boolean;
}
export declare const ReportCharts: React.FC<ReportChartsProps>;
export default ReportCharts;
//# sourceMappingURL=ReportCharts.d.ts.map