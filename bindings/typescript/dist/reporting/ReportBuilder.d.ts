import React from 'react';
import { ReportDefinition, ValidationResult, DataSource } from './types';
export interface ReportBuilderProps {
    initialDefinition?: Partial<ReportDefinition>;
    dataSources: DataSource[];
    onSave: (definition: ReportDefinition) => Promise<void>;
    onCancel?: () => void;
    onValidate?: (definition: ReportDefinition) => Promise<ValidationResult>;
    readOnly?: boolean;
}
export declare const ReportBuilder: React.FC<ReportBuilderProps>;
export default ReportBuilder;
//# sourceMappingURL=ReportBuilder.d.ts.map