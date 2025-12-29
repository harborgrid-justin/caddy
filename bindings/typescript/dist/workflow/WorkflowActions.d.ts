import React from 'react';
import type { WorkflowAction } from './types';
export interface WorkflowActionsProps {
    actions?: WorkflowAction[];
    selectedAction?: WorkflowAction;
    onActionSelect?: (action: WorkflowAction) => void;
    onActionCreate?: (action: Omit<WorkflowAction, 'id'>) => void;
    onActionUpdate?: (actionId: string, updates: Partial<WorkflowAction>) => void;
    onActionDelete?: (actionId: string) => void;
    readOnly?: boolean;
}
export declare const WorkflowActions: React.FC<WorkflowActionsProps>;
export default WorkflowActions;
//# sourceMappingURL=WorkflowActions.d.ts.map