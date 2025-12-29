import React from 'react';
import { FilterGroup, Table } from './types';
export interface ReportFiltersProps {
    availableTables: Table[];
    filterGroup: FilterGroup;
    onChange: (filterGroup: FilterGroup) => void;
    readOnly?: boolean;
}
export declare const ReportFilters: React.FC<ReportFiltersProps>;
export default ReportFilters;
//# sourceMappingURL=ReportFilters.d.ts.map