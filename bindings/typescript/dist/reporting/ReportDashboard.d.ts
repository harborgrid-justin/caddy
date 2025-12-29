import React from 'react';
import { ReportDefinition, ReportExecution } from './types';
export interface ReportDashboardProps {
    reports: ReportDefinition[];
    executions?: ReportExecution[];
    onCreateReport?: () => void;
    onEditReport?: (report: ReportDefinition) => void;
    onDeleteReport?: (reportId: string) => void;
    onDuplicateReport?: (report: ReportDefinition) => void;
    onExecuteReport?: (reportId: string) => void;
    onScheduleReport?: (reportId: string) => void;
    onViewReport?: (reportId: string) => void;
    onExportReport?: (reportId: string) => void;
}
export declare const ReportDashboard: React.FC<ReportDashboardProps>;
export default ReportDashboard;
//# sourceMappingURL=ReportDashboard.d.ts.map