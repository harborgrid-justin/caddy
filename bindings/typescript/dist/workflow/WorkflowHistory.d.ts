import React from 'react';
import type { WorkflowExecution } from './types';
export interface WorkflowHistoryProps {
    executions: WorkflowExecution[];
    selectedExecutionId?: string;
    onExecutionSelect?: (executionId: string) => void;
    onExecutionRetry?: (executionId: string) => void;
    onExecutionDelete?: (executionId: string) => void;
    maxHeight?: string;
}
export declare const WorkflowHistory: React.FC<WorkflowHistoryProps>;
export default WorkflowHistory;
//# sourceMappingURL=WorkflowHistory.d.ts.map