import React from 'react';
import { ReportSchedule } from './types';
export interface ReportSchedulerProps {
    schedule?: ReportSchedule;
    onChange: (schedule: ReportSchedule) => void;
    readOnly?: boolean;
}
export declare const ReportScheduler: React.FC<ReportSchedulerProps>;
export default ReportScheduler;
//# sourceMappingURL=ReportScheduler.d.ts.map