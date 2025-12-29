import React from 'react';
import type { WorkflowNode as WorkflowNodeType, ExecutionStatus } from './types';
export interface WorkflowNodeProps {
    node: WorkflowNodeType;
    isSelected?: boolean;
    isExecuting?: boolean;
    executionStatus?: ExecutionStatus;
    executionProgress?: number;
    onSelect?: (nodeId: string, multiSelect: boolean) => void;
    onUpdate?: (nodeId: string, updates: Partial<WorkflowNodeType>) => void;
    onDelete?: (nodeId: string) => void;
    onPortConnect?: (portId: string, portType: 'input' | 'output') => void;
    onPortDisconnect?: (portId: string) => void;
    readOnly?: boolean;
    showPorts?: boolean;
    zoom?: number;
}
export declare const WorkflowNode: React.FC<WorkflowNodeProps>;
export default WorkflowNode;
//# sourceMappingURL=WorkflowNode.d.ts.map