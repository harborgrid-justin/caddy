import React from 'react';
import { ReportTemplate, ReportDefinition } from './types';
export interface ReportTemplatesProps {
    templates: ReportTemplate[];
    onSelectTemplate: (template: ReportTemplate) => void;
    onCreateFromTemplate?: (template: ReportTemplate) => Promise<ReportDefinition>;
    onSaveAsTemplate?: (definition: ReportDefinition, metadata: {
        name: string;
        description: string;
        category: string;
        tags: string[];
    }) => Promise<void>;
    showCreateButton?: boolean;
}
export declare const ReportTemplates: React.FC<ReportTemplatesProps>;
export default ReportTemplates;
//# sourceMappingURL=ReportTemplates.d.ts.map