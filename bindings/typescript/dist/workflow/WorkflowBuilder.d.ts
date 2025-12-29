import React from 'react';
import type { Workflow, WorkflowExecution, Position, UserCursor } from './types';
export interface WorkflowBuilderProps {
    workflow?: Workflow;
    executions?: WorkflowExecution[];
    collaboratorCursors?: UserCursor[];
    onWorkflowChange?: (workflow: Workflow) => void;
    onWorkflowSave?: (workflow: Workflow) => void;
    onWorkflowExecute?: (workflow: Workflow) => void;
    onCursorMove?: (position: Position) => void;
    readOnly?: boolean;
    showTemplates?: boolean;
    showHistory?: boolean;
    autoSave?: boolean;
    autoSaveInterval?: number;
}
export declare const WorkflowBuilder: React.FC<WorkflowBuilderProps>;
export default WorkflowBuilder;
//# sourceMappingURL=WorkflowBuilder.d.ts.map