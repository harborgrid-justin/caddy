export { default as WorkflowBuilder } from './WorkflowBuilder';
export { default as WorkflowCanvas } from './WorkflowCanvas';
export { default as WorkflowNode } from './WorkflowNode';
export { default as WorkflowConnector } from './WorkflowConnector';
export { default as WorkflowSidebar } from './WorkflowSidebar';
export { default as WorkflowExecutor } from './WorkflowExecutor';
export { default as WorkflowHistory } from './WorkflowHistory';
export { default as WorkflowTemplates } from './WorkflowTemplates';
export { default as WorkflowConditions } from './WorkflowConditions';
export { default as WorkflowActions } from './WorkflowActions';
export { default as WorkflowTriggers } from './WorkflowTriggers';
export const createWorkflow = (name, description = '') => ({
    id: `wf_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
    name,
    description,
    version: '1.0.0',
    nodes: [],
    connections: [],
    variables: [],
    triggers: [],
    settings: {
        timeout: 300000,
        maxRetries: 3,
        retryDelay: 1000,
        errorHandling: 'stop',
    },
    metadata: {
        createdAt: new Date(),
        updatedAt: new Date(),
        createdBy: 'user',
        lastModifiedBy: 'user',
        version: 1,
        status: 'draft',
    },
});
export const validateWorkflow = (workflow) => {
    const errors = [];
    const warnings = [];
    const triggerNodes = workflow.nodes.filter((n) => n.type === 'trigger');
    if (triggerNodes.length === 0) {
        errors.push({
            message: 'Workflow must have at least one trigger node',
            severity: 'error',
            code: 'NO_TRIGGER',
        });
    }
    workflow.nodes.forEach((node) => {
        const hasIncoming = workflow.connections.some((c) => c.targetNodeId === node.id);
        const hasOutgoing = workflow.connections.some((c) => c.sourceNodeId === node.id);
        if (!hasIncoming && node.type !== 'trigger') {
            warnings.push({
                nodeId: node.id,
                message: `Node "${node.label}" has no incoming connections`,
                severity: 'warning',
                code: 'NO_INCOMING',
            });
        }
        if (!hasOutgoing && node.outputs.length > 0) {
            warnings.push({
                nodeId: node.id,
                message: `Node "${node.label}" has no outgoing connections`,
                severity: 'warning',
                code: 'NO_OUTGOING',
            });
        }
    });
    return {
        valid: errors.length === 0,
        errors,
        warnings,
    };
};
export const cloneWorkflow = (workflow) => ({
    ...workflow,
    id: `wf_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
    name: `${workflow.name} (Copy)`,
    metadata: {
        ...workflow.metadata,
        createdAt: new Date(),
        updatedAt: new Date(),
        version: 1,
        status: 'draft',
    },
});
export { default } from './WorkflowBuilder';
//# sourceMappingURL=index.js.map