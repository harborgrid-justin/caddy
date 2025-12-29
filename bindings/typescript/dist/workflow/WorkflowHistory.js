import React, { useMemo, useState, useCallback } from 'react';
const STATUS_COLORS = {
    idle: '#94a3b8',
    running: '#3b82f6',
    paused: '#f59e0b',
    completed: '#10b981',
    failed: '#ef4444',
    cancelled: '#64748b',
    retrying: '#f97316',
};
const LOG_LEVEL_COLORS = {
    debug: '#94a3b8',
    info: '#3b82f6',
    warning: '#f59e0b',
    error: '#ef4444',
};
export const WorkflowHistory = ({ executions, selectedExecutionId, onExecutionSelect, onExecutionRetry, onExecutionDelete, maxHeight = '600px', }) => {
    const [filterStatus, setFilterStatus] = useState('all');
    const [searchTerm, setSearchTerm] = useState('');
    const [expandedExecutionId, setExpandedExecutionId] = useState(null);
    const [expandedNodeId, setExpandedNodeId] = useState(null);
    const filteredExecutions = useMemo(() => {
        return executions
            .filter((exec) => {
            const matchesStatus = filterStatus === 'all' || exec.status === filterStatus;
            const matchesSearch = searchTerm === '' ||
                exec.id.toLowerCase().includes(searchTerm.toLowerCase()) ||
                exec.workflowId.toLowerCase().includes(searchTerm.toLowerCase());
            return matchesStatus && matchesSearch;
        })
            .sort((a, b) => b.startTime.getTime() - a.startTime.getTime());
    }, [executions, filterStatus, searchTerm]);
    const selectedExecution = useMemo(() => {
        return executions.find((exec) => exec.id === selectedExecutionId);
    }, [executions, selectedExecutionId]);
    const formatDuration = useCallback((ms) => {
        if (!ms)
            return 'N/A';
        if (ms < 1000)
            return `${ms}ms`;
        if (ms < 60000)
            return `${(ms / 1000).toFixed(2)}s`;
        return `${Math.floor(ms / 60000)}m ${Math.floor((ms % 60000) / 1000)}s`;
    }, []);
    const formatDate = useCallback((date) => {
        return new Date(date).toLocaleString();
    }, []);
    const handleExecutionClick = useCallback((executionId) => {
        if (onExecutionSelect) {
            onExecutionSelect(executionId);
        }
        setExpandedExecutionId(expandedExecutionId === executionId ? null : executionId);
    }, [onExecutionSelect, expandedExecutionId]);
    const renderExecutionItem = useCallback((execution) => {
        const isExpanded = expandedExecutionId === execution.id;
        const isSelected = selectedExecutionId === execution.id;
        return (React.createElement("div", { key: execution.id, style: {
                backgroundColor: isSelected ? '#eff6ff' : '#fff',
                border: `1px solid ${isSelected ? '#3b82f6' : '#e2e8f0'}`,
                borderRadius: '8px',
                marginBottom: '8px',
                overflow: 'hidden',
            } },
            React.createElement("div", { onClick: () => handleExecutionClick(execution.id), style: {
                    padding: '12px 16px',
                    cursor: 'pointer',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    backgroundColor: isSelected ? '#eff6ff' : '#fff',
                } },
                React.createElement("div", { style: { flex: 1 } },
                    React.createElement("div", { style: { display: 'flex', alignItems: 'center', gap: '8px' } },
                        React.createElement("div", { style: {
                                width: '10px',
                                height: '10px',
                                borderRadius: '50%',
                                backgroundColor: STATUS_COLORS[execution.status],
                            } }),
                        React.createElement("span", { style: { fontSize: '14px', fontWeight: 600, color: '#1e293b' } }, execution.id),
                        React.createElement("span", { style: {
                                padding: '2px 8px',
                                backgroundColor: `${STATUS_COLORS[execution.status]}20`,
                                color: STATUS_COLORS[execution.status],
                                borderRadius: '4px',
                                fontSize: '11px',
                                fontWeight: 600,
                                textTransform: 'uppercase',
                            } }, execution.status)),
                    React.createElement("div", { style: { fontSize: '12px', color: '#64748b', marginTop: '4px' } },
                        "Started: ",
                        formatDate(execution.startTime),
                        " \u2022 Duration:",
                        ' ',
                        formatDuration(execution.duration))),
                React.createElement("div", { style: { display: 'flex', gap: '8px' } },
                    execution.status === 'failed' && onExecutionRetry && (React.createElement("button", { onClick: (e) => {
                            e.stopPropagation();
                            onExecutionRetry(execution.id);
                        }, style: {
                            padding: '4px 12px',
                            backgroundColor: '#3b82f6',
                            color: '#fff',
                            border: 'none',
                            borderRadius: '4px',
                            cursor: 'pointer',
                            fontSize: '12px',
                        } }, "Retry")),
                    onExecutionDelete && (React.createElement("button", { onClick: (e) => {
                            e.stopPropagation();
                            onExecutionDelete(execution.id);
                        }, style: {
                            padding: '4px 12px',
                            backgroundColor: '#ef4444',
                            color: '#fff',
                            border: 'none',
                            borderRadius: '4px',
                            cursor: 'pointer',
                            fontSize: '12px',
                        } }, "Delete")),
                    React.createElement("span", { style: { fontSize: '16px', color: '#64748b' } }, isExpanded ? '▼' : '▶'))),
            isExpanded && (React.createElement("div", { style: { padding: '16px', backgroundColor: '#f8fafc' } },
                execution.error && (React.createElement("div", { style: {
                        padding: '12px',
                        backgroundColor: '#fef2f2',
                        border: '1px solid #fecaca',
                        borderRadius: '6px',
                        marginBottom: '16px',
                    } },
                    React.createElement("div", { style: { fontWeight: 600, color: '#ef4444', marginBottom: '8px' } },
                        "Error: ",
                        execution.error.code),
                    React.createElement("div", { style: { fontSize: '13px', color: '#64748b' } }, execution.error.message),
                    execution.error.stack && (React.createElement("pre", { style: {
                            marginTop: '8px',
                            padding: '8px',
                            backgroundColor: '#fff',
                            border: '1px solid #e2e8f0',
                            borderRadius: '4px',
                            fontSize: '11px',
                            overflow: 'auto',
                            maxHeight: '200px',
                        } }, execution.error.stack)))),
                React.createElement("div", { style: { marginBottom: '16px' } },
                    React.createElement("h4", { style: { fontSize: '13px', fontWeight: 600, marginBottom: '8px' } },
                        "Node Executions (",
                        execution.nodeExecutions.length,
                        ")"),
                    execution.nodeExecutions.map((nodeExec) => renderNodeExecution(nodeExec))),
                React.createElement("div", null,
                    React.createElement("h4", { style: { fontSize: '13px', fontWeight: 600, marginBottom: '8px' } }, "Context"),
                    React.createElement("pre", { style: {
                            padding: '12px',
                            backgroundColor: '#fff',
                            border: '1px solid #e2e8f0',
                            borderRadius: '6px',
                            fontSize: '11px',
                            overflow: 'auto',
                            maxHeight: '200px',
                        } }, JSON.stringify(execution.context, null, 2)))))));
    }, [
        expandedExecutionId,
        selectedExecutionId,
        handleExecutionClick,
        formatDate,
        formatDuration,
        onExecutionRetry,
        onExecutionDelete,
    ]);
    const renderNodeExecution = useCallback((nodeExec) => {
        const isExpanded = expandedNodeId === nodeExec.id;
        return (React.createElement("div", { key: nodeExec.id, style: {
                backgroundColor: '#fff',
                border: '1px solid #e2e8f0',
                borderRadius: '6px',
                marginBottom: '8px',
            } },
            React.createElement("div", { onClick: () => setExpandedNodeId(expandedNodeId === nodeExec.id ? null : nodeExec.id), style: {
                    padding: '8px 12px',
                    cursor: 'pointer',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                } },
                React.createElement("div", { style: { flex: 1 } },
                    React.createElement("div", { style: { display: 'flex', alignItems: 'center', gap: '8px' } },
                        React.createElement("div", { style: {
                                width: '8px',
                                height: '8px',
                                borderRadius: '50%',
                                backgroundColor: STATUS_COLORS[nodeExec.status],
                            } }),
                        React.createElement("span", { style: { fontSize: '13px', fontWeight: 500, color: '#1e293b' } }, nodeExec.nodeId),
                        nodeExec.retryCount && nodeExec.retryCount > 0 && (React.createElement("span", { style: {
                                padding: '1px 6px',
                                backgroundColor: '#fef3c7',
                                color: '#f59e0b',
                                borderRadius: '3px',
                                fontSize: '10px',
                                fontWeight: 600,
                            } },
                            "Retry ",
                            nodeExec.retryCount))),
                    React.createElement("div", { style: { fontSize: '11px', color: '#64748b', marginTop: '2px' } },
                        "Duration: ",
                        formatDuration(nodeExec.duration))),
                React.createElement("span", { style: { fontSize: '14px', color: '#64748b' } }, isExpanded ? '▼' : '▶')),
            isExpanded && (React.createElement("div", { style: { padding: '12px', backgroundColor: '#f8fafc' } },
                React.createElement("div", { style: { marginBottom: '12px' } },
                    React.createElement("div", { style: { fontSize: '11px', fontWeight: 600, marginBottom: '4px' } }, "Input:"),
                    React.createElement("pre", { style: {
                            padding: '8px',
                            backgroundColor: '#fff',
                            border: '1px solid #e2e8f0',
                            borderRadius: '4px',
                            fontSize: '10px',
                            overflow: 'auto',
                            maxHeight: '150px',
                        } }, JSON.stringify(nodeExec.input, null, 2))),
                nodeExec.output && (React.createElement("div", { style: { marginBottom: '12px' } },
                    React.createElement("div", { style: { fontSize: '11px', fontWeight: 600, marginBottom: '4px' } }, "Output:"),
                    React.createElement("pre", { style: {
                            padding: '8px',
                            backgroundColor: '#fff',
                            border: '1px solid #e2e8f0',
                            borderRadius: '4px',
                            fontSize: '10px',
                            overflow: 'auto',
                            maxHeight: '150px',
                        } }, JSON.stringify(nodeExec.output, null, 2)))),
                nodeExec.error && (React.createElement("div", { style: {
                        padding: '8px',
                        backgroundColor: '#fef2f2',
                        border: '1px solid #fecaca',
                        borderRadius: '4px',
                        marginBottom: '12px',
                    } },
                    React.createElement("div", { style: { fontSize: '11px', fontWeight: 600, color: '#ef4444' } },
                        nodeExec.error.code,
                        ": ",
                        nodeExec.error.message))),
                React.createElement("div", null,
                    React.createElement("div", { style: { fontSize: '11px', fontWeight: 600, marginBottom: '4px' } },
                        "Logs (",
                        nodeExec.logs.length,
                        "):"),
                    React.createElement("div", { style: {
                            backgroundColor: '#fff',
                            border: '1px solid #e2e8f0',
                            borderRadius: '4px',
                            maxHeight: '150px',
                            overflow: 'auto',
                        } }, nodeExec.logs.map((log) => (React.createElement("div", { key: log.id, style: {
                            padding: '6px 8px',
                            borderBottom: '1px solid #f1f5f9',
                            fontSize: '10px',
                        } },
                        React.createElement("span", { style: { color: LOG_LEVEL_COLORS[log.level] } },
                            "[",
                            log.level.toUpperCase(),
                            "]"),
                        ' ',
                        React.createElement("span", { style: { color: '#94a3b8' } }, formatDate(log.timestamp)),
                        ' ',
                        React.createElement("span", { style: { color: '#1e293b' } }, log.message))))))))));
    }, [expandedNodeId, formatDuration, formatDate]);
    return (React.createElement("div", { style: {
            display: 'flex',
            flexDirection: 'column',
            height: '100%',
            backgroundColor: '#fff',
            borderRadius: '8px',
            overflow: 'hidden',
        } },
        React.createElement("div", { style: {
                padding: '16px',
                borderBottom: '1px solid #e2e8f0',
                backgroundColor: '#f8fafc',
            } },
            React.createElement("h2", { style: { fontSize: '18px', fontWeight: 600, marginBottom: '12px' } }, "Execution History"),
            React.createElement("div", { style: { display: 'flex', gap: '12px', flexWrap: 'wrap' } },
                React.createElement("input", { type: "text", placeholder: "Search executions...", value: searchTerm, onChange: (e) => setSearchTerm(e.target.value), style: {
                        flex: 1,
                        minWidth: '200px',
                        padding: '8px 12px',
                        border: '1px solid #e2e8f0',
                        borderRadius: '6px',
                        fontSize: '14px',
                    } }),
                React.createElement("select", { value: filterStatus, onChange: (e) => setFilterStatus(e.target.value), style: {
                        padding: '8px 12px',
                        border: '1px solid #e2e8f0',
                        borderRadius: '6px',
                        fontSize: '14px',
                        cursor: 'pointer',
                    } },
                    React.createElement("option", { value: "all" }, "All Status"),
                    React.createElement("option", { value: "running" }, "Running"),
                    React.createElement("option", { value: "completed" }, "Completed"),
                    React.createElement("option", { value: "failed" }, "Failed"),
                    React.createElement("option", { value: "cancelled" }, "Cancelled"),
                    React.createElement("option", { value: "paused" }, "Paused")))),
        React.createElement("div", { style: {
                flex: 1,
                overflow: 'auto',
                padding: '16px',
                maxHeight,
            } }, filteredExecutions.length > 0 ? (filteredExecutions.map((execution) => renderExecutionItem(execution))) : (React.createElement("div", { style: {
                textAlign: 'center',
                color: '#94a3b8',
                padding: '40px 20px',
            } }, "No executions found")))));
};
export default WorkflowHistory;
//# sourceMappingURL=WorkflowHistory.js.map