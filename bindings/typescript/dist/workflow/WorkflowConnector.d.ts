import React from 'react';
import type { WorkflowConnection, Position } from './types';
export interface WorkflowConnectorProps {
    connection: WorkflowConnection;
    sourcePosition: Position;
    targetPosition: Position;
    isSelected?: boolean;
    isAnimated?: boolean;
    isExecuting?: boolean;
    color?: string;
    onClick?: (connectionId: string) => void;
    onDelete?: (connectionId: string) => void;
    readOnly?: boolean;
    zoom?: number;
}
export declare const WorkflowConnector: React.FC<WorkflowConnectorProps>;
export default WorkflowConnector;
//# sourceMappingURL=WorkflowConnector.d.ts.map