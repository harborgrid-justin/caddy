import React from 'react';
import type { WorkflowNode, WorkflowVariable } from './types';
export interface WorkflowSidebarProps {
    selectedNode?: WorkflowNode;
    variables?: WorkflowVariable[];
    onNodeUpdate?: (nodeId: string, updates: Partial<WorkflowNode>) => void;
    onVariableCreate?: (variable: Omit<WorkflowVariable, 'id'>) => void;
    onVariableUpdate?: (variableId: string, updates: Partial<WorkflowVariable>) => void;
    onVariableDelete?: (variableId: string) => void;
    readOnly?: boolean;
}
export declare const WorkflowSidebar: React.FC<WorkflowSidebarProps>;
export default WorkflowSidebar;
//# sourceMappingURL=WorkflowSidebar.d.ts.map