import React from 'react';
import { ExportConfig, ReportData } from './types';
export interface ReportExportProps {
    reportData?: ReportData;
    onExport: (config: ExportConfig) => Promise<void>;
    showPreview?: boolean;
}
export declare const ReportExport: React.FC<ReportExportProps>;
export default ReportExport;
//# sourceMappingURL=ReportExport.d.ts.map