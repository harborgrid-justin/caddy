import React from 'react';
import { ReportDefinition, ReportData, Filter } from './types';
export interface ReportViewerProps {
    reportId: string;
    definition?: ReportDefinition;
    initialParameters?: Record<string, any>;
    onExecute?: (reportId: string, parameters: Record<string, any>) => Promise<ReportData>;
    onDrillDown?: (targetReportId: string, filters: Filter[]) => void;
    onExport?: (format: string) => void;
    autoRefresh?: boolean;
    refreshInterval?: number;
    showToolbar?: boolean;
    showParameters?: boolean;
    interactive?: boolean;
}
export declare const ReportViewer: React.FC<ReportViewerProps>;
export default ReportViewer;
//# sourceMappingURL=ReportViewer.d.ts.map