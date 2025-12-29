/**
 * CADDY v0.4.0 - Workflow History Component
 * Execution history and logs viewer
 */

import React, { useMemo, useState, useCallback } from 'react';
import type {
  WorkflowExecution,
  NodeExecution,
  ExecutionStatus,
  ExecutionLog,
} from './types';

export interface WorkflowHistoryProps {
  executions: WorkflowExecution[];
  selectedExecutionId?: string;
  onExecutionSelect?: (executionId: string) => void;
  onExecutionRetry?: (executionId: string) => void;
  onExecutionDelete?: (executionId: string) => void;
  maxHeight?: string;
}

const STATUS_COLORS: Record<ExecutionStatus, string> = {
  idle: '#94a3b8',
  running: '#3b82f6',
  paused: '#f59e0b',
  completed: '#10b981',
  failed: '#ef4444',
  cancelled: '#64748b',
  retrying: '#f97316',
};

const LOG_LEVEL_COLORS: Record<string, string> = {
  debug: '#94a3b8',
  info: '#3b82f6',
  warning: '#f59e0b',
  error: '#ef4444',
};

export const WorkflowHistory: React.FC<WorkflowHistoryProps> = ({
  executions,
  selectedExecutionId,
  onExecutionSelect,
  onExecutionRetry,
  onExecutionDelete,
  maxHeight = '600px',
}) => {
  const [filterStatus, setFilterStatus] = useState<ExecutionStatus | 'all'>('all');
  const [searchTerm, setSearchTerm] = useState('');
  const [expandedExecutionId, setExpandedExecutionId] = useState<string | null>(null);
  const [expandedNodeId, setExpandedNodeId] = useState<string | null>(null);

  const filteredExecutions = useMemo(() => {
    return executions
      .filter((exec) => {
        const matchesStatus = filterStatus === 'all' || exec.status === filterStatus;
        const matchesSearch =
          searchTerm === '' ||
          exec.id.toLowerCase().includes(searchTerm.toLowerCase()) ||
          exec.workflowId.toLowerCase().includes(searchTerm.toLowerCase());
        return matchesStatus && matchesSearch;
      })
      .sort((a, b) => b.startTime.getTime() - a.startTime.getTime());
  }, [executions, filterStatus, searchTerm]);

  const selectedExecution = useMemo(() => {
    return executions.find((exec) => exec.id === selectedExecutionId);
  }, [executions, selectedExecutionId]);

  const formatDuration = useCallback((ms?: number) => {
    if (!ms) return 'N/A';
    if (ms < 1000) return `${ms}ms`;
    if (ms < 60000) return `${(ms / 1000).toFixed(2)}s`;
    return `${Math.floor(ms / 60000)}m ${Math.floor((ms % 60000) / 1000)}s`;
  }, []);

  const formatDate = useCallback((date: Date) => {
    return new Date(date).toLocaleString();
  }, []);

  const handleExecutionClick = useCallback(
    (executionId: string) => {
      if (onExecutionSelect) {
        onExecutionSelect(executionId);
      }
      setExpandedExecutionId(expandedExecutionId === executionId ? null : executionId);
    },
    [onExecutionSelect, expandedExecutionId]
  );

  const renderExecutionItem = useCallback(
    (execution: WorkflowExecution) => {
      const isExpanded = expandedExecutionId === execution.id;
      const isSelected = selectedExecutionId === execution.id;

      return (
        <div
          key={execution.id}
          style={{
            backgroundColor: isSelected ? '#eff6ff' : '#fff',
            border: `1px solid ${isSelected ? '#3b82f6' : '#e2e8f0'}`,
            borderRadius: '8px',
            marginBottom: '8px',
            overflow: 'hidden',
          }}
        >
          {/* Header */}
          <div
            onClick={() => handleExecutionClick(execution.id)}
            style={{
              padding: '12px 16px',
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              backgroundColor: isSelected ? '#eff6ff' : '#fff',
            }}
          >
            <div style={{ flex: 1 }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <div
                  style={{
                    width: '10px',
                    height: '10px',
                    borderRadius: '50%',
                    backgroundColor: STATUS_COLORS[execution.status],
                  }}
                />
                <span style={{ fontSize: '14px', fontWeight: 600, color: '#1e293b' }}>
                  {execution.id}
                </span>
                <span
                  style={{
                    padding: '2px 8px',
                    backgroundColor: `${STATUS_COLORS[execution.status]}20`,
                    color: STATUS_COLORS[execution.status],
                    borderRadius: '4px',
                    fontSize: '11px',
                    fontWeight: 600,
                    textTransform: 'uppercase',
                  }}
                >
                  {execution.status}
                </span>
              </div>
              <div style={{ fontSize: '12px', color: '#64748b', marginTop: '4px' }}>
                Started: {formatDate(execution.startTime)} • Duration:{' '}
                {formatDuration(execution.duration)}
              </div>
            </div>

            <div style={{ display: 'flex', gap: '8px' }}>
              {execution.status === 'failed' && onExecutionRetry && (
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onExecutionRetry(execution.id);
                  }}
                  style={{
                    padding: '4px 12px',
                    backgroundColor: '#3b82f6',
                    color: '#fff',
                    border: 'none',
                    borderRadius: '4px',
                    cursor: 'pointer',
                    fontSize: '12px',
                  }}
                >
                  Retry
                </button>
              )}
              {onExecutionDelete && (
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onExecutionDelete(execution.id);
                  }}
                  style={{
                    padding: '4px 12px',
                    backgroundColor: '#ef4444',
                    color: '#fff',
                    border: 'none',
                    borderRadius: '4px',
                    cursor: 'pointer',
                    fontSize: '12px',
                  }}
                >
                  Delete
                </button>
              )}
              <span style={{ fontSize: '16px', color: '#64748b' }}>
                {isExpanded ? '▼' : '▶'}
              </span>
            </div>
          </div>

          {/* Expanded Details */}
          {isExpanded && (
            <div style={{ padding: '16px', backgroundColor: '#f8fafc' }}>
              {/* Error Details */}
              {execution.error && (
                <div
                  style={{
                    padding: '12px',
                    backgroundColor: '#fef2f2',
                    border: '1px solid #fecaca',
                    borderRadius: '6px',
                    marginBottom: '16px',
                  }}
                >
                  <div style={{ fontWeight: 600, color: '#ef4444', marginBottom: '8px' }}>
                    Error: {execution.error.code}
                  </div>
                  <div style={{ fontSize: '13px', color: '#64748b' }}>
                    {execution.error.message}
                  </div>
                  {execution.error.stack && (
                    <pre
                      style={{
                        marginTop: '8px',
                        padding: '8px',
                        backgroundColor: '#fff',
                        border: '1px solid #e2e8f0',
                        borderRadius: '4px',
                        fontSize: '11px',
                        overflow: 'auto',
                        maxHeight: '200px',
                      }}
                    >
                      {execution.error.stack}
                    </pre>
                  )}
                </div>
              )}

              {/* Node Executions */}
              <div style={{ marginBottom: '16px' }}>
                <h4 style={{ fontSize: '13px', fontWeight: 600, marginBottom: '8px' }}>
                  Node Executions ({execution.nodeExecutions.length})
                </h4>
                {execution.nodeExecutions.map((nodeExec) =>
                  renderNodeExecution(nodeExec)
                )}
              </div>

              {/* Context */}
              <div>
                <h4 style={{ fontSize: '13px', fontWeight: 600, marginBottom: '8px' }}>
                  Context
                </h4>
                <pre
                  style={{
                    padding: '12px',
                    backgroundColor: '#fff',
                    border: '1px solid #e2e8f0',
                    borderRadius: '6px',
                    fontSize: '11px',
                    overflow: 'auto',
                    maxHeight: '200px',
                  }}
                >
                  {JSON.stringify(execution.context, null, 2)}
                </pre>
              </div>
            </div>
          )}
        </div>
      );
    },
    [
      expandedExecutionId,
      selectedExecutionId,
      handleExecutionClick,
      formatDate,
      formatDuration,
      onExecutionRetry,
      onExecutionDelete,
    ]
  );

  const renderNodeExecution = useCallback(
    (nodeExec: NodeExecution) => {
      const isExpanded = expandedNodeId === nodeExec.id;

      return (
        <div
          key={nodeExec.id}
          style={{
            backgroundColor: '#fff',
            border: '1px solid #e2e8f0',
            borderRadius: '6px',
            marginBottom: '8px',
          }}
        >
          <div
            onClick={() =>
              setExpandedNodeId(expandedNodeId === nodeExec.id ? null : nodeExec.id)
            }
            style={{
              padding: '8px 12px',
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
            }}
          >
            <div style={{ flex: 1 }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <div
                  style={{
                    width: '8px',
                    height: '8px',
                    borderRadius: '50%',
                    backgroundColor: STATUS_COLORS[nodeExec.status],
                  }}
                />
                <span style={{ fontSize: '13px', fontWeight: 500, color: '#1e293b' }}>
                  {nodeExec.nodeId}
                </span>
                {nodeExec.retryCount && nodeExec.retryCount > 0 && (
                  <span
                    style={{
                      padding: '1px 6px',
                      backgroundColor: '#fef3c7',
                      color: '#f59e0b',
                      borderRadius: '3px',
                      fontSize: '10px',
                      fontWeight: 600,
                    }}
                  >
                    Retry {nodeExec.retryCount}
                  </span>
                )}
              </div>
              <div style={{ fontSize: '11px', color: '#64748b', marginTop: '2px' }}>
                Duration: {formatDuration(nodeExec.duration)}
              </div>
            </div>
            <span style={{ fontSize: '14px', color: '#64748b' }}>
              {isExpanded ? '▼' : '▶'}
            </span>
          </div>

          {isExpanded && (
            <div style={{ padding: '12px', backgroundColor: '#f8fafc' }}>
              {/* Input */}
              <div style={{ marginBottom: '12px' }}>
                <div style={{ fontSize: '11px', fontWeight: 600, marginBottom: '4px' }}>
                  Input:
                </div>
                <pre
                  style={{
                    padding: '8px',
                    backgroundColor: '#fff',
                    border: '1px solid #e2e8f0',
                    borderRadius: '4px',
                    fontSize: '10px',
                    overflow: 'auto',
                    maxHeight: '150px',
                  }}
                >
                  {JSON.stringify(nodeExec.input, null, 2)}
                </pre>
              </div>

              {/* Output */}
              {nodeExec.output && (
                <div style={{ marginBottom: '12px' }}>
                  <div style={{ fontSize: '11px', fontWeight: 600, marginBottom: '4px' }}>
                    Output:
                  </div>
                  <pre
                    style={{
                      padding: '8px',
                      backgroundColor: '#fff',
                      border: '1px solid #e2e8f0',
                      borderRadius: '4px',
                      fontSize: '10px',
                      overflow: 'auto',
                      maxHeight: '150px',
                    }}
                  >
                    {JSON.stringify(nodeExec.output, null, 2)}
                  </pre>
                </div>
              )}

              {/* Error */}
              {nodeExec.error && (
                <div
                  style={{
                    padding: '8px',
                    backgroundColor: '#fef2f2',
                    border: '1px solid #fecaca',
                    borderRadius: '4px',
                    marginBottom: '12px',
                  }}
                >
                  <div style={{ fontSize: '11px', fontWeight: 600, color: '#ef4444' }}>
                    {nodeExec.error.code}: {nodeExec.error.message}
                  </div>
                </div>
              )}

              {/* Logs */}
              <div>
                <div style={{ fontSize: '11px', fontWeight: 600, marginBottom: '4px' }}>
                  Logs ({nodeExec.logs.length}):
                </div>
                <div
                  style={{
                    backgroundColor: '#fff',
                    border: '1px solid #e2e8f0',
                    borderRadius: '4px',
                    maxHeight: '150px',
                    overflow: 'auto',
                  }}
                >
                  {nodeExec.logs.map((log) => (
                    <div
                      key={log.id}
                      style={{
                        padding: '6px 8px',
                        borderBottom: '1px solid #f1f5f9',
                        fontSize: '10px',
                      }}
                    >
                      <span style={{ color: LOG_LEVEL_COLORS[log.level] }}>
                        [{log.level.toUpperCase()}]
                      </span>{' '}
                      <span style={{ color: '#94a3b8' }}>
                        {formatDate(log.timestamp)}
                      </span>{' '}
                      <span style={{ color: '#1e293b' }}>{log.message}</span>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}
        </div>
      );
    },
    [expandedNodeId, formatDuration, formatDate]
  );

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
        backgroundColor: '#fff',
        borderRadius: '8px',
        overflow: 'hidden',
      }}
    >
      {/* Header */}
      <div
        style={{
          padding: '16px',
          borderBottom: '1px solid #e2e8f0',
          backgroundColor: '#f8fafc',
        }}
      >
        <h2 style={{ fontSize: '18px', fontWeight: 600, marginBottom: '12px' }}>
          Execution History
        </h2>

        {/* Filters */}
        <div style={{ display: 'flex', gap: '12px', flexWrap: 'wrap' }}>
          <input
            type="text"
            placeholder="Search executions..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            style={{
              flex: 1,
              minWidth: '200px',
              padding: '8px 12px',
              border: '1px solid #e2e8f0',
              borderRadius: '6px',
              fontSize: '14px',
            }}
          />

          <select
            value={filterStatus}
            onChange={(e) => setFilterStatus(e.target.value as ExecutionStatus | 'all')}
            style={{
              padding: '8px 12px',
              border: '1px solid #e2e8f0',
              borderRadius: '6px',
              fontSize: '14px',
              cursor: 'pointer',
            }}
          >
            <option value="all">All Status</option>
            <option value="running">Running</option>
            <option value="completed">Completed</option>
            <option value="failed">Failed</option>
            <option value="cancelled">Cancelled</option>
            <option value="paused">Paused</option>
          </select>
        </div>
      </div>

      {/* Execution List */}
      <div
        style={{
          flex: 1,
          overflow: 'auto',
          padding: '16px',
          maxHeight,
        }}
      >
        {filteredExecutions.length > 0 ? (
          filteredExecutions.map((execution) => renderExecutionItem(execution))
        ) : (
          <div
            style={{
              textAlign: 'center',
              color: '#94a3b8',
              padding: '40px 20px',
            }}
          >
            No executions found
          </div>
        )}
      </div>
    </div>
  );
};

export default WorkflowHistory;
