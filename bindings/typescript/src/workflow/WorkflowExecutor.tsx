/**
 * CADDY v0.4.0 - Workflow Executor Component
 * Workflow execution engine with retry logic and error handling
 */

import React, { useCallback, useEffect, useState, useRef } from 'react';
import type {
  Workflow,
  WorkflowExecution,
  NodeExecution,
  ExecutionContext,
  ExecutionError,
  ExecutionStatus,
  RetryPolicy,
} from './types';

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

const DEFAULT_RETRY_POLICY: RetryPolicy = {
  maxRetries: 3,
  initialDelay: 1000,
  maxDelay: 30000,
  backoffMultiplier: 2,
  retryableErrors: ['TIMEOUT', 'NETWORK_ERROR', 'SERVICE_UNAVAILABLE'],
};

export const useWorkflowExecutor = ({
  workflow,
  onExecutionStart,
  onExecutionUpdate,
  onExecutionComplete,
  onExecutionError,
  onNodeExecutionStart,
  onNodeExecutionComplete,
  retryPolicy = DEFAULT_RETRY_POLICY,
  context = { variables: {} },
}: WorkflowExecutorProps) => {
  const [execution, setExecution] = useState<WorkflowExecution | null>(null);
  const [isExecuting, setIsExecuting] = useState(false);
  const executionRef = useRef<WorkflowExecution | null>(null);
  const abortControllerRef = useRef<AbortController | null>(null);

  const generateExecutionId = useCallback(() => {
    return `exec_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }, []);

  const generateNodeExecutionId = useCallback(() => {
    return `node_exec_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }, []);

  const createExecution = useCallback(
    (triggeredBy: 'manual' | 'schedule' | 'webhook' | 'event' = 'manual'): WorkflowExecution => {
      return {
        id: generateExecutionId(),
        workflowId: workflow.id,
        status: 'running',
        startTime: new Date(),
        nodeExecutions: [],
        context,
        metadata: {
          triggeredBy,
          environment: 'production',
        },
      };
    },
    [workflow.id, context, generateExecutionId]
  );

  const executeNode = useCallback(
    async (
      nodeId: string,
      input: unknown,
      retryCount: number = 0
    ): Promise<{ output: unknown; logs: any[] }> => {
      const node = workflow.nodes.find((n) => n.id === nodeId);
      if (!node) {
        throw new Error(`Node ${nodeId} not found`);
      }

      const nodeExecution: NodeExecution = {
        id: generateNodeExecutionId(),
        nodeId,
        status: 'running',
        startTime: new Date(),
        input,
        retryCount,
        logs: [],
      };

      if (onNodeExecutionStart) {
        onNodeExecutionStart(nodeExecution);
      }

      try {
        // Simulate node execution based on type
        let output: unknown;
        const logs: any[] = [];

        switch (node.type) {
          case 'trigger':
            output = input || context.trigger?.data;
            logs.push({
              id: `log_${Date.now()}`,
              timestamp: new Date(),
              level: 'info' as const,
              message: 'Workflow triggered',
              data: output,
            });
            break;

          case 'action':
            await new Promise((resolve) => setTimeout(resolve, 500));
            output = { success: true, data: input };
            logs.push({
              id: `log_${Date.now()}`,
              timestamp: new Date(),
              level: 'info' as const,
              message: 'Action executed successfully',
              data: output,
            });
            break;

          case 'email':
            await new Promise((resolve) => setTimeout(resolve, 1000));
            output = {
              sent: true,
              to: node.config.to,
              subject: node.config.subject,
              timestamp: new Date(),
            };
            logs.push({
              id: `log_${Date.now()}`,
              timestamp: new Date(),
              level: 'info' as const,
              message: `Email sent to ${node.config.to}`,
              data: output,
            });
            break;

          case 'api':
            await new Promise((resolve) => setTimeout(resolve, 800));
            output = {
              status: 200,
              data: { result: 'API call successful' },
            };
            logs.push({
              id: `log_${Date.now()}`,
              timestamp: new Date(),
              level: 'info' as const,
              message: 'API request completed',
              data: output,
            });
            break;

          case 'database':
            await new Promise((resolve) => setTimeout(resolve, 600));
            output = {
              rows: [],
              count: 0,
            };
            logs.push({
              id: `log_${Date.now()}`,
              timestamp: new Date(),
              level: 'info' as const,
              message: 'Database query executed',
              data: output,
            });
            break;

          case 'condition':
            const conditions = node.config.conditions as any[];
            const result = conditions?.length ? Math.random() > 0.5 : true;
            output = { conditionMet: result, branch: result ? 'true' : 'false' };
            logs.push({
              id: `log_${Date.now()}`,
              timestamp: new Date(),
              level: 'info' as const,
              message: `Condition evaluated to ${result}`,
              data: output,
            });
            break;

          case 'loop':
            const items = (input as any)?.items || [];
            output = { iterations: items.length, results: items };
            logs.push({
              id: `log_${Date.now()}`,
              timestamp: new Date(),
              level: 'info' as const,
              message: `Loop executed ${items.length} times`,
              data: output,
            });
            break;

          case 'delay':
            const delay = (node.config.delay as number) || 1;
            await new Promise((resolve) => setTimeout(resolve, delay * 1000));
            output = { delayed: true, duration: delay };
            logs.push({
              id: `log_${Date.now()}`,
              timestamp: new Date(),
              level: 'info' as const,
              message: `Delayed for ${delay} seconds`,
              data: output,
            });
            break;

          case 'transform':
            output = {
              transformed: true,
              original: input,
              result: input,
            };
            logs.push({
              id: `log_${Date.now()}`,
              timestamp: new Date(),
              level: 'info' as const,
              message: 'Data transformed',
              data: output,
            });
            break;

          case 'script':
            await new Promise((resolve) => setTimeout(resolve, 400));
            output = { executed: true, result: 'Script executed' };
            logs.push({
              id: `log_${Date.now()}`,
              timestamp: new Date(),
              level: 'info' as const,
              message: 'Script executed',
              data: output,
            });
            break;

          default:
            output = { success: true, input };
            logs.push({
              id: `log_${Date.now()}`,
              timestamp: new Date(),
              level: 'warning' as const,
              message: `Unknown node type: ${node.type}`,
            });
        }

        nodeExecution.status = 'completed';
        nodeExecution.endTime = new Date();
        nodeExecution.duration =
          nodeExecution.endTime.getTime() - nodeExecution.startTime.getTime();
        nodeExecution.output = output;
        nodeExecution.logs = logs;

        if (onNodeExecutionComplete) {
          onNodeExecutionComplete(nodeExecution);
        }

        return { output, logs };
      } catch (error: any) {
        const executionError: ExecutionError = {
          code: error.code || 'EXECUTION_ERROR',
          message: error.message,
          stack: error.stack,
          nodeId,
          timestamp: new Date(),
          recoverable: retryPolicy.retryableErrors?.includes(error.code) || false,
        };

        // Retry logic
        if (
          executionError.recoverable &&
          retryCount < retryPolicy.maxRetries
        ) {
          const delay = Math.min(
            retryPolicy.initialDelay * Math.pow(retryPolicy.backoffMultiplier, retryCount),
            retryPolicy.maxDelay
          );

          nodeExecution.logs.push({
            id: `log_${Date.now()}`,
            timestamp: new Date(),
            level: 'warning' as const,
            message: `Retrying in ${delay}ms (attempt ${retryCount + 1}/${retryPolicy.maxRetries})`,
            data: executionError,
          });

          nodeExecution.status = 'retrying';
          if (onNodeExecutionComplete) {
            onNodeExecutionComplete(nodeExecution);
          }

          await new Promise((resolve) => setTimeout(resolve, delay));
          return executeNode(nodeId, input, retryCount + 1);
        }

        nodeExecution.status = 'failed';
        nodeExecution.endTime = new Date();
        nodeExecution.duration =
          nodeExecution.endTime.getTime() - nodeExecution.startTime.getTime();
        nodeExecution.error = executionError;

        if (onNodeExecutionComplete) {
          onNodeExecutionComplete(nodeExecution);
        }

        throw executionError;
      }
    },
    [
      workflow.nodes,
      context,
      retryPolicy,
      onNodeExecutionStart,
      onNodeExecutionComplete,
      generateNodeExecutionId,
    ]
  );

  const executeWorkflow = useCallback(async () => {
    if (isExecuting) {
      return;
    }

    setIsExecuting(true);
    abortControllerRef.current = new AbortController();

    const newExecution = createExecution('manual');
    setExecution(newExecution);
    executionRef.current = newExecution;

    if (onExecutionStart) {
      onExecutionStart(newExecution);
    }

    try {
      // Find trigger nodes
      const triggerNodes = workflow.nodes.filter((n) => n.type === 'trigger');
      if (triggerNodes.length === 0) {
        throw new Error('No trigger node found');
      }

      // Build execution graph
      const executionOrder = topologicalSort(workflow.nodes, workflow.connections);

      // Execute nodes in order
      const outputs: Record<string, unknown> = {};

      for (const nodeId of executionOrder) {
        if (abortControllerRef.current?.signal.aborted) {
          newExecution.status = 'cancelled';
          break;
        }

        // Get input from connected nodes
        const incomingConnections = workflow.connections.filter(
          (c) => c.targetNodeId === nodeId
        );
        const input = incomingConnections.reduce((acc, conn) => {
          return { ...acc, [conn.sourceNodeId]: outputs[conn.sourceNodeId] };
        }, {});

        const { output, logs } = await executeNode(nodeId, input);
        outputs[nodeId] = output;

        // Update execution
        newExecution.nodeExecutions.push({
          id: generateNodeExecutionId(),
          nodeId,
          status: 'completed',
          startTime: new Date(),
          endTime: new Date(),
          duration: 0,
          input,
          output,
          logs,
        });

        if (onExecutionUpdate) {
          onExecutionUpdate(newExecution);
        }
      }

      if (newExecution.status !== 'cancelled') {
        newExecution.status = 'completed';
      }
      newExecution.endTime = new Date();
      newExecution.duration =
        newExecution.endTime.getTime() - newExecution.startTime.getTime();

      setExecution(newExecution);
      executionRef.current = newExecution;

      if (onExecutionComplete) {
        onExecutionComplete(newExecution);
      }
    } catch (error: any) {
      const executionError: ExecutionError = {
        code: error.code || 'WORKFLOW_ERROR',
        message: error.message,
        stack: error.stack,
        timestamp: new Date(),
        recoverable: false,
      };

      newExecution.status = 'failed';
      newExecution.endTime = new Date();
      newExecution.duration =
        newExecution.endTime.getTime() - newExecution.startTime.getTime();
      newExecution.error = executionError;

      setExecution(newExecution);
      executionRef.current = newExecution;

      if (onExecutionError) {
        onExecutionError(executionError);
      }

      if (onExecutionComplete) {
        onExecutionComplete(newExecution);
      }
    } finally {
      setIsExecuting(false);
      abortControllerRef.current = null;
    }
  }, [
    isExecuting,
    workflow,
    createExecution,
    executeNode,
    onExecutionStart,
    onExecutionUpdate,
    onExecutionComplete,
    onExecutionError,
    generateNodeExecutionId,
  ]);

  const cancelExecution = useCallback(() => {
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }
  }, []);

  const pauseExecution = useCallback(() => {
    if (execution && isExecuting) {
      setExecution({ ...execution, status: 'paused' });
    }
  }, [execution, isExecuting]);

  const resumeExecution = useCallback(() => {
    if (execution && execution.status === 'paused') {
      setExecution({ ...execution, status: 'running' });
    }
  }, [execution]);

  // Topological sort for execution order
  const topologicalSort = (nodes: any[], connections: any[]): string[] => {
    const sorted: string[] = [];
    const visited = new Set<string>();
    const temp = new Set<string>();

    const adjacencyList: Record<string, string[]> = {};
    nodes.forEach((node) => {
      adjacencyList[node.id] = [];
    });
    connections.forEach((conn) => {
      if (!adjacencyList[conn.sourceNodeId]) {
        adjacencyList[conn.sourceNodeId] = [];
      }
      adjacencyList[conn.sourceNodeId].push(conn.targetNodeId);
    });

    const visit = (nodeId: string) => {
      if (temp.has(nodeId)) {
        throw new Error('Circular dependency detected');
      }
      if (!visited.has(nodeId)) {
        temp.add(nodeId);
        adjacencyList[nodeId]?.forEach((neighbor) => visit(neighbor));
        temp.delete(nodeId);
        visited.add(nodeId);
        sorted.push(nodeId);
      }
    };

    nodes.forEach((node) => {
      if (!visited.has(node.id)) {
        visit(node.id);
      }
    });

    return sorted.reverse();
  };

  return {
    execution,
    isExecuting,
    executeWorkflow,
    cancelExecution,
    pauseExecution,
    resumeExecution,
  };
};

// Legacy export for backwards compatibility
export const WorkflowExecutor = useWorkflowExecutor;
export default useWorkflowExecutor;
