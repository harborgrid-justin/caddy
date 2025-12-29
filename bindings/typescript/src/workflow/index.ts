/**
 * CADDY v0.4.0 - Workflow Engine Module
 * Visual workflow automation system with drag-and-drop builder
 *
 * @module workflow
 * @version 0.4.0
 */

import type { Workflow, ValidationResult, ValidationError, ValidationWarning } from './types';

// Main Components
export { default as WorkflowBuilder } from './WorkflowBuilder';
export type { WorkflowBuilderProps } from './WorkflowBuilder';

export { default as WorkflowCanvas } from './WorkflowCanvas';
export type { WorkflowCanvasProps } from './WorkflowCanvas';

export { default as WorkflowNode } from './WorkflowNode';
export type { WorkflowNodeProps } from './WorkflowNode';

export { default as WorkflowConnector } from './WorkflowConnector';
export type { WorkflowConnectorProps } from './WorkflowConnector';

export { default as WorkflowSidebar } from './WorkflowSidebar';
export type { WorkflowSidebarProps } from './WorkflowSidebar';

export { default as WorkflowExecutor } from './WorkflowExecutor';
export type { WorkflowExecutorProps } from './WorkflowExecutor';

export { default as WorkflowHistory } from './WorkflowHistory';
export type { WorkflowHistoryProps } from './WorkflowHistory';

export { default as WorkflowTemplates } from './WorkflowTemplates';
export type { WorkflowTemplatesProps } from './WorkflowTemplates';

export { default as WorkflowConditions } from './WorkflowConditions';
export type { WorkflowConditionsProps } from './WorkflowConditions';

export { default as WorkflowActions } from './WorkflowActions';
export type { WorkflowActionsProps } from './WorkflowActions';

export { default as WorkflowTriggers } from './WorkflowTriggers';
export type { WorkflowTriggersProps } from './WorkflowTriggers';

// Types
export type {
  // Core Types
  Workflow,
  WorkflowNode as WorkflowNodeType,
  WorkflowConnection,
  WorkflowVariable,
  WorkflowTrigger,
  WorkflowExecution,
  NodeExecution,

  // Configuration Types
  WorkflowSettings,
  WorkflowMetadata,
  NodeData,
  NodePort,
  NodeMetadata,
  TriggerConfig,
  NotificationSettings,

  // Execution Types
  ExecutionContext,
  ExecutionError,
  ExecutionLog,
  ExecutionMetadata,
  RetryPolicy,

  // Logic Types
  Condition,
  ConditionGroup,
  WorkflowAction,
  WorkflowTemplate,

  // UI Types
  Position,
  Size,
  CanvasState,
  HistoryState,
  UserCursor,
  ValidationResult,
  ValidationError,
  ValidationWarning,
  WorkflowPermissions,
  WorkflowStats,

  // Schema Types
  JSONSchema,

  // Enum Types
  NodeType,
  TriggerType,
  ActionType,
  ExecutionStatus,
  ConditionOperator,
} from './types';

/**
 * Workflow Engine Features:
 *
 * 1. Visual Workflow Builder
 *    - Drag-and-drop interface
 *    - Node palette with categories
 *    - Visual connections between nodes
 *    - Zoom, pan, and minimap controls
 *
 * 2. Node Types
 *    - Triggers: schedule, webhook, data-change, manual, event
 *    - Actions: email, API, database, transform, script
 *    - Logic: conditions, loops, delays
 *    - Data: transform, aggregate
 *
 * 3. Execution Engine
 *    - Topological execution order
 *    - Parallel execution support
 *    - Error handling and retry logic
 *    - Real-time execution visualization
 *    - Execution history and logs
 *
 * 4. Conditional Logic
 *    - Visual condition builder
 *    - Multiple operators (equals, contains, etc.)
 *    - Nested condition groups
 *    - AND/OR logic
 *
 * 5. Workflow Templates
 *    - Pre-built templates
 *    - Template categories
 *    - Custom template creation
 *
 * 6. Real-time Collaboration
 *    - Collaborative editing
 *    - User cursors
 *    - Live updates
 *
 * 7. History and Versioning
 *    - Execution history
 *    - Undo/Redo support
 *    - Version control
 *
 * 8. Advanced Features
 *    - Variables and context
 *    - Workflow validation
 *    - Auto-save
 *    - Read-only mode
 *    - Keyboard shortcuts
 *
 * @example Basic Usage
 * ```tsx
 * import { WorkflowBuilder } from '@caddy/workflow';
 *
 * function MyWorkflowApp() {
 *   const [workflow, setWorkflow] = useState<Workflow>(initialWorkflow);
 *
 *   return (
 *     <WorkflowBuilder
 *       workflow={workflow}
 *       onWorkflowChange={setWorkflow}
 *       onWorkflowSave={handleSave}
 *       onWorkflowExecute={handleExecute}
 *       autoSave={true}
 *     />
 *   );
 * }
 * ```
 *
 * @example Custom Workflow Creation
 * ```tsx
 * import { Workflow, WorkflowNode, WorkflowConnection } from '@caddy/workflow';
 *
 * const myWorkflow: Workflow = {
 *   id: 'wf_1',
 *   name: 'My Workflow',
 *   description: 'Custom workflow',
 *   version: '1.0.0',
 *   nodes: [
 *     {
 *       id: 'node_1',
 *       type: 'trigger',
 *       label: 'Start',
 *       position: { x: 100, y: 100 },
 *       // ... other properties
 *     },
 *     {
 *       id: 'node_2',
 *       type: 'email',
 *       label: 'Send Email',
 *       position: { x: 400, y: 100 },
 *       // ... other properties
 *     },
 *   ],
 *   connections: [
 *     {
 *       id: 'conn_1',
 *       sourceNodeId: 'node_1',
 *       sourcePortId: 'out_1',
 *       targetNodeId: 'node_2',
 *       targetPortId: 'in_1',
 *     },
 *   ],
 *   // ... other properties
 * };
 * ```
 *
 * @example Using Workflow Executor
 * ```tsx
 * import { WorkflowExecutor } from '@caddy/workflow';
 *
 * function ExecuteWorkflow({ workflow }) {
 *   const executor = WorkflowExecutor({
 *     workflow,
 *     onExecutionStart: (execution) => console.log('Started', execution),
 *     onExecutionComplete: (execution) => console.log('Completed', execution),
 *     onExecutionError: (error) => console.error('Error', error),
 *   });
 *
 *   return (
 *     <button onClick={executor.executeWorkflow}>
 *       Execute Workflow
 *     </button>
 *   );
 * }
 * ```
 *
 * @example Workflow Templates
 * ```tsx
 * import { WorkflowTemplates } from '@caddy/workflow';
 *
 * function TemplatesView() {
 *   return (
 *     <WorkflowTemplates
 *       onTemplateSelect={(template) => {
 *         createWorkflowFromTemplate(template);
 *       }}
 *     />
 *   );
 * }
 * ```
 *
 * @example Conditional Logic
 * ```tsx
 * import { WorkflowConditions, ConditionGroup } from '@caddy/workflow';
 *
 * function ConditionBuilder() {
 *   const [conditions, setConditions] = useState<ConditionGroup>({
 *     id: 'root',
 *     conditions: [],
 *     logic: 'AND',
 *   });
 *
 *   return (
 *     <WorkflowConditions
 *       conditions={conditions}
 *       onChange={setConditions}
 *     />
 *   );
 * }
 * ```
 */

// Utility functions
export const createWorkflow = (
  name: string,
  description: string = ''
): Workflow => ({
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

export const validateWorkflow = (workflow: Workflow): ValidationResult => {
  const errors: ValidationError[] = [];
  const warnings: ValidationWarning[] = [];

  // Check for trigger nodes
  const triggerNodes = workflow.nodes.filter((n) => n.type === 'trigger');
  if (triggerNodes.length === 0) {
    errors.push({
      message: 'Workflow must have at least one trigger node',
      severity: 'error',
      code: 'NO_TRIGGER',
    });
  }

  // Check for disconnected nodes
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

export const cloneWorkflow = (workflow: Workflow): Workflow => ({
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

/**
 * Default export provides the main WorkflowBuilder component
 */
export { default } from './WorkflowBuilder';
