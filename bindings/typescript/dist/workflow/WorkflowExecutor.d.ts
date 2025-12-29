import type { Workflow, WorkflowExecution, NodeExecution, ExecutionContext, ExecutionError, RetryPolicy } from './types';
export interface WorkflowExecutorProps {
    workflow: Workflow;
    onExecutionStart?: (execution: WorkflowExecution) => void;
    onExecutionUpdate?: (execution: WorkflowExecution) => void;
    onExecutionComplete?: (execution: WorkflowExecution) => void;
    onExecutionError?: (error: ExecutionError) => void;
    onNodeExecutionStart?: (nodeExecution: NodeExecution) => void;
    onNodeExecutionComplete?: (nodeExecution: NodeExecution) => void;
    retryPolicy?: RetryPolicy;
    context?: ExecutionContext;
}
export declare const useWorkflowExecutor: ({ workflow, onExecutionStart, onExecutionUpdate, onExecutionComplete, onExecutionError, onNodeExecutionStart, onNodeExecutionComplete, retryPolicy, context, }: WorkflowExecutorProps) => {
    execution: WorkflowExecution | null;
    isExecuting: boolean;
    executeWorkflow: () => Promise<void>;
    cancelExecution: () => void;
    pauseExecution: () => void;
    resumeExecution: () => void;
};
export declare const WorkflowExecutor: ({ workflow, onExecutionStart, onExecutionUpdate, onExecutionComplete, onExecutionError, onNodeExecutionStart, onNodeExecutionComplete, retryPolicy, context, }: WorkflowExecutorProps) => {
    execution: WorkflowExecution | null;
    isExecuting: boolean;
    executeWorkflow: () => Promise<void>;
    cancelExecution: () => void;
    pauseExecution: () => void;
    resumeExecution: () => void;
};
export default useWorkflowExecutor;
//# sourceMappingURL=WorkflowExecutor.d.ts.map