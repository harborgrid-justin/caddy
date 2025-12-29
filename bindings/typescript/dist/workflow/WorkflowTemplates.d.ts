import React from 'react';
import type { WorkflowTemplate, Workflow } from './types';
export interface WorkflowTemplatesProps {
    templates?: WorkflowTemplate[];
    onTemplateSelect?: (template: WorkflowTemplate) => void;
    onTemplateCreate?: (workflow: Workflow) => void;
    onTemplateDelete?: (templateId: string) => void;
}
export declare const WorkflowTemplates: React.FC<WorkflowTemplatesProps>;
export default WorkflowTemplates;
//# sourceMappingURL=WorkflowTemplates.d.ts.map