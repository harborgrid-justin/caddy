import React from 'react';
import { SelectField, Table } from './types';
export interface ReportFieldsProps {
    availableTables: Table[];
    selectedFields: SelectField[];
    onChange: (fields: SelectField[]) => void;
    readOnly?: boolean;
    showAggregations?: boolean;
    showCalculations?: boolean;
}
export declare const ReportFields: React.FC<ReportFieldsProps>;
export default ReportFields;
//# sourceMappingURL=ReportFields.d.ts.map