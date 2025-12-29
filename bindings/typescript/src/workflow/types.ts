/**
 * CADDY v0.4.0 - Workflow Engine Types
 * Enterprise-grade workflow automation type definitions
 */

export type NodeType =
  | 'trigger'
  | 'action'
  | 'condition'
  | 'loop'
  | 'delay'
  | 'transform'
  | 'api'
  | 'email'
  | 'webhook'
  | 'database'
  | 'script';

export type TriggerType =
  | 'schedule'
  | 'webhook'
  | 'data-change'
  | 'manual'
  | 'event';

export type ActionType =
  | 'email'
  | 'http-request'
  | 'database-query'
  | 'transform-data'
  | 'send-notification'
  | 'create-record'
  | 'update-record'
  | 'delete-record';

export type ExecutionStatus =
  | 'idle'
  | 'running'
  | 'paused'
  | 'completed'
  | 'failed'
  | 'cancelled'
  | 'retrying';

export type ConditionOperator =
  | 'equals'
  | 'not-equals'
  | 'greater-than'
  | 'less-than'
  | 'contains'
  | 'not-contains'
  | 'starts-with'
  | 'ends-with'
  | 'matches-regex'
  | 'is-empty'
  | 'is-not-empty';

export interface Position {
  x: number;
  y: number;
}

export interface Size {
  width: number;
  height: number;
}

export interface WorkflowNode {
  id: string;
  type: NodeType;
  label: string;
  position: Position;
  data: NodeData;
  inputs: NodePort[];
  outputs: NodePort[];
  config: Record<string, unknown>;
  metadata: NodeMetadata;
}

export interface NodeData {
  [key: string]: unknown;
  description?: string;
  icon?: string;
  color?: string;
}

export interface NodePort {
  id: string;
  nodeId: string;
  type: 'input' | 'output';
  label: string;
  dataType?: string;
  required?: boolean;
  multiple?: boolean;
}

export interface NodeMetadata {
  createdAt: Date;
  updatedAt: Date;
  createdBy: string;
  version: number;
  tags?: string[];
  notes?: string;
}

export interface WorkflowConnection {
  id: string;
  sourceNodeId: string;
  sourcePortId: string;
  targetNodeId: string;
  targetPortId: string;
  label?: string;
  metadata?: {
    createdAt: Date;
    updatedAt: Date;
  };
}

export interface Workflow {
  id: string;
  name: string;
  description: string;
  version: string;
  nodes: WorkflowNode[];
  connections: WorkflowConnection[];
  variables: WorkflowVariable[];
  triggers: WorkflowTrigger[];
  settings: WorkflowSettings;
  metadata: WorkflowMetadata;
}

export interface WorkflowVariable {
  id: string;
  name: string;
  type: 'string' | 'number' | 'boolean' | 'object' | 'array';
  value: unknown;
  scope: 'global' | 'local';
  encrypted?: boolean;
}

export interface WorkflowTrigger {
  id: string;
  type: TriggerType;
  config: TriggerConfig;
  enabled: boolean;
}

export interface TriggerConfig {
  [key: string]: unknown;
  schedule?: string; // cron expression
  webhookUrl?: string;
  event?: string;
}

export interface WorkflowSettings {
  timeout?: number;
  maxRetries?: number;
  retryDelay?: number;
  concurrency?: number;
  errorHandling?: 'stop' | 'continue' | 'retry';
  notifications?: NotificationSettings;
}

export interface NotificationSettings {
  onSuccess?: boolean;
  onFailure?: boolean;
  channels?: string[];
  recipients?: string[];
}

export interface WorkflowMetadata {
  createdAt: Date;
  updatedAt: Date;
  createdBy: string;
  lastModifiedBy: string;
  version: number;
  status: 'draft' | 'published' | 'archived';
  tags?: string[];
  category?: string;
  isTemplate?: boolean;
}

export interface WorkflowExecution {
  id: string;
  workflowId: string;
  status: ExecutionStatus;
  startTime: Date;
  endTime?: Date;
  duration?: number;
  nodeExecutions: NodeExecution[];
  context: ExecutionContext;
  error?: ExecutionError;
  metadata: ExecutionMetadata;
}

export interface NodeExecution {
  id: string;
  nodeId: string;
  status: ExecutionStatus;
  startTime: Date;
  endTime?: Date;
  duration?: number;
  input?: unknown;
  output?: unknown;
  error?: ExecutionError;
  retryCount?: number;
  logs: ExecutionLog[];
}

export interface ExecutionContext {
  variables: Record<string, unknown>;
  user?: {
    id: string;
    email: string;
    name: string;
  };
  trigger?: {
    type: string;
    data: unknown;
  };
  metadata?: Record<string, unknown>;
}

export interface ExecutionError {
  code: string;
  message: string;
  stack?: string;
  nodeId?: string;
  timestamp: Date;
  recoverable: boolean;
}

export interface ExecutionLog {
  id: string;
  timestamp: Date;
  level: 'debug' | 'info' | 'warning' | 'error';
  message: string;
  data?: unknown;
}

export interface ExecutionMetadata {
  triggeredBy: 'manual' | 'schedule' | 'webhook' | 'event';
  environment: 'development' | 'staging' | 'production';
  region?: string;
  tenantId?: string;
}

export interface Condition {
  id: string;
  field: string;
  operator: ConditionOperator;
  value: unknown;
  logic?: 'AND' | 'OR';
}

export interface ConditionGroup {
  id: string;
  conditions: (Condition | ConditionGroup)[];
  logic: 'AND' | 'OR';
}

export interface WorkflowTemplate {
  id: string;
  name: string;
  description: string;
  category: string;
  icon?: string;
  workflow: Partial<Workflow>;
  popularity?: number;
  tags?: string[];
}

export interface CanvasState {
  zoom: number;
  offset: Position;
  selectedNodes: string[];
  selectedConnections: string[];
  isDragging: boolean;
  isPanning: boolean;
}

export interface HistoryState {
  past: Workflow[];
  present: Workflow;
  future: Workflow[];
}

export interface CollaborationEvent {
  type: 'node-added' | 'node-updated' | 'node-deleted' | 'connection-added' | 'connection-deleted' | 'cursor-moved';
  userId: string;
  userName: string;
  timestamp: Date;
  data: unknown;
}

export interface UserCursor {
  userId: string;
  userName: string;
  color: string;
  position: Position;
  lastUpdate: Date;
}

export interface WorkflowStats {
  totalExecutions: number;
  successfulExecutions: number;
  failedExecutions: number;
  averageDuration: number;
  lastExecuted?: Date;
}

export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
}

export interface ValidationError {
  nodeId?: string;
  connectionId?: string;
  message: string;
  severity: 'error';
  code: string;
}

export interface ValidationWarning {
  nodeId?: string;
  connectionId?: string;
  message: string;
  severity: 'warning';
  code: string;
}

export interface WorkflowAction {
  id: string;
  type: ActionType;
  name: string;
  description: string;
  icon?: string;
  config: Record<string, unknown>;
  inputSchema?: JSONSchema;
  outputSchema?: JSONSchema;
}

export interface JSONSchema {
  type: string;
  properties?: Record<string, unknown>;
  required?: string[];
  [key: string]: unknown;
}

export interface RetryPolicy {
  maxRetries: number;
  initialDelay: number;
  maxDelay: number;
  backoffMultiplier: number;
  retryableErrors?: string[];
}

export interface WorkflowPermissions {
  canView: boolean;
  canEdit: boolean;
  canExecute: boolean;
  canDelete: boolean;
  canShare: boolean;
}
