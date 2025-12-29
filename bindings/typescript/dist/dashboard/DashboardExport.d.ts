import React from 'react';
import type { ExportFormat, DashboardConfig } from './types';
export interface DashboardExportProps {
    dashboardConfig: DashboardConfig;
    data?: any;
    availableMetrics?: Array<{
        id: string;
        name: string;
    }>;
    availableCharts?: Array<{
        id: string;
        name: string;
    }>;
    onExportComplete?: (format: ExportFormat, blob: Blob) => void;
    onExportError?: (error: Error) => void;
    className?: string;
}
export declare const DashboardExport: React.FC<DashboardExportProps>;
export default DashboardExport;
//# sourceMappingURL=DashboardExport.d.ts.map