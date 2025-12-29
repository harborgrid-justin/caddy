import React from 'react';
import type { ExecutiveOverview as ExecutiveOverviewData, TimeRange } from './types';
export interface ExecutiveOverviewProps {
    data: ExecutiveOverviewData;
    showPeriodSelector?: boolean;
    onPeriodChange?: (period: TimeRange) => void;
    showPrintButton?: boolean;
    className?: string;
}
export declare const ExecutiveOverview: React.FC<ExecutiveOverviewProps>;
export default ExecutiveOverview;
//# sourceMappingURL=ExecutiveOverview.d.ts.map