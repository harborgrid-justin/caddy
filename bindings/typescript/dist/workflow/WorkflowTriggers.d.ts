import React from 'react';
import type { WorkflowTrigger } from './types';
export interface WorkflowTriggersProps {
    triggers: WorkflowTrigger[];
    onTriggerAdd?: (trigger: Omit<WorkflowTrigger, 'id'>) => void;
    onTriggerUpdate?: (triggerId: string, updates: Partial<WorkflowTrigger>) => void;
    onTriggerDelete?: (triggerId: string) => void;
    onTriggerToggle?: (triggerId: string, enabled: boolean) => void;
    readOnly?: boolean;
}
export declare const WorkflowTriggers: React.FC<WorkflowTriggersProps>;
export default WorkflowTriggers;
//# sourceMappingURL=WorkflowTriggers.d.ts.map