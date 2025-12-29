import React from 'react';
import type { ConditionGroup } from './types';
export interface WorkflowConditionsProps {
    conditions?: ConditionGroup;
    onChange?: (conditions: ConditionGroup) => void;
    readOnly?: boolean;
}
export declare const WorkflowConditions: React.FC<WorkflowConditionsProps>;
export default WorkflowConditions;
//# sourceMappingURL=WorkflowConditions.d.ts.map