import React from 'react';
import type { DashboardFilters } from './types';
export interface DashboardFiltersProps {
    filters: DashboardFilters;
    onChange: (filters: DashboardFilters) => void;
    departments?: string[];
    regions?: string[];
    users?: string[];
    statuses?: string[];
    showDateRange?: boolean;
    showDepartments?: boolean;
    showRegions?: boolean;
    showUsers?: boolean;
    showStatuses?: boolean;
    enableSavedFilters?: boolean;
    className?: string;
}
export declare const DashboardFiltersComponent: React.FC<DashboardFiltersProps>;
export default DashboardFiltersComponent;
//# sourceMappingURL=DashboardFilters.d.ts.map