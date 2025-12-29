import React from 'react';
import type { Workflow, WorkflowNode as WorkflowNodeType, Position, NodeExecution, UserCursor } from './types';
export interface WorkflowCanvasProps {
    workflow: Workflow;
    executions?: NodeExecution[];
    isExecuting?: boolean;
    selectedNodeIds?: string[];
    selectedConnectionIds?: string[];
    collaboratorCursors?: UserCursor[];
    onNodeSelect?: (nodeId: string, multiSelect: boolean) => void;
    onNodeUpdate?: (nodeId: string, updates: Partial<WorkflowNodeType>) => void;
    onNodeDelete?: (nodeId: string) => void;
    onNodeAdd?: (node: Partial<WorkflowNodeType>, position: Position) => void;
    onConnectionCreate?: (sourcePortId: string, targetPortId: string) => void;
    onConnectionDelete?: (connectionId: string) => void;
    onCanvasClick?: () => void;
    onCursorMove?: (position: Position) => void;
    readOnly?: boolean;
    showGrid?: boolean;
    showMinimap?: boolean;
}
export declare const WorkflowCanvas: React.FC<WorkflowCanvasProps>;
export default WorkflowCanvas;
//# sourceMappingURL=WorkflowCanvas.d.ts.map