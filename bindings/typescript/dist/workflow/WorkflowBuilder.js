import React, { useState, useCallback, useEffect, useRef } from 'react';
import { DndProvider } from 'react-dnd';
import { HTML5Backend } from 'react-dnd-html5-backend';
import WorkflowCanvas from './WorkflowCanvas';
import WorkflowSidebar from './WorkflowSidebar';
import WorkflowHistory from './WorkflowHistory';
import WorkflowTemplates from './WorkflowTemplates';
const generateId = () => `${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
const DEFAULT_WORKFLOW = {
    id: generateId(),
    name: 'New Workflow',
    description: 'Workflow description',
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
};
export const WorkflowBuilder = ({ workflow: initialWorkflow, executions = [], collaboratorCursors = [], onWorkflowChange, onWorkflowSave, onWorkflowExecute, onCursorMove, readOnly = false, showTemplates = false, showHistory = false, autoSave = false, autoSaveInterval = 30000, }) => {
    const [workflow, setWorkflow] = useState(initialWorkflow || DEFAULT_WORKFLOW);
    const [history, setHistory] = useState({
        past: [],
        present: workflow,
        future: [],
    });
    const [selectedNodeIds, setSelectedNodeIds] = useState([]);
    const [selectedConnectionIds, setSelectedConnectionIds] = useState([]);
    const [isExecuting, setIsExecuting] = useState(false);
    const [currentExecution, setCurrentExecution] = useState(null);
    const [validationResult, setValidationResult] = useState({
        valid: true,
        errors: [],
        warnings: [],
    });
    const [activePanel, setActivePanel] = useState('sidebar');
    const autoSaveTimerRef = useRef(null);
    const workflowRef = useRef(workflow);
    useEffect(() => {
        workflowRef.current = workflow;
    }, [workflow]);
    useEffect(() => {
        if (autoSave && onWorkflowSave && !readOnly) {
            if (autoSaveTimerRef.current) {
                clearTimeout(autoSaveTimerRef.current);
            }
            autoSaveTimerRef.current = setTimeout(() => {
                onWorkflowSave(workflowRef.current);
            }, autoSaveInterval);
        }
        return () => {
            if (autoSaveTimerRef.current) {
                clearTimeout(autoSaveTimerRef.current);
            }
        };
    }, [workflow, autoSave, autoSaveInterval, onWorkflowSave, readOnly]);
    const updateHistory = useCallback((newWorkflow) => {
        setHistory((prev) => ({
            past: [...prev.past, prev.present],
            present: newWorkflow,
            future: [],
        }));
    }, []);
    const undo = useCallback(() => {
        if (history.past.length === 0)
            return;
        const previous = history.past[history.past.length - 1];
        const newPast = history.past.slice(0, history.past.length - 1);
        setHistory({
            past: newPast,
            present: previous,
            future: [history.present, ...history.future],
        });
        setWorkflow(previous);
    }, [history]);
    const redo = useCallback(() => {
        if (history.future.length === 0)
            return;
        const next = history.future[0];
        const newFuture = history.future.slice(1);
        setHistory({
            past: [...history.past, history.present],
            present: next,
            future: newFuture,
        });
        setWorkflow(next);
    }, [history]);
    useEffect(() => {
        const handleKeyDown = (e) => {
            if ((e.ctrlKey || e.metaKey) && e.key === 'z' && !e.shiftKey) {
                e.preventDefault();
                undo();
            }
            else if ((e.ctrlKey || e.metaKey) && (e.key === 'y' || (e.key === 'z' && e.shiftKey))) {
                e.preventDefault();
                redo();
            }
            else if ((e.ctrlKey || e.metaKey) && e.key === 's') {
                e.preventDefault();
                if (onWorkflowSave && !readOnly) {
                    onWorkflowSave(workflow);
                }
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [undo, redo, workflow, onWorkflowSave, readOnly]);
    const validateWorkflow = useCallback((wf) => {
        const errors = [];
        const warnings = [];
        const triggerNodes = wf.nodes.filter((n) => n.type === 'trigger');
        if (triggerNodes.length === 0) {
            errors.push({
                message: 'Workflow must have at least one trigger node',
                severity: 'error',
                code: 'NO_TRIGGER',
            });
        }
        wf.nodes.forEach((node) => {
            const hasIncoming = wf.connections.some((c) => c.targetNodeId === node.id);
            const hasOutgoing = wf.connections.some((c) => c.sourceNodeId === node.id);
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
        const visited = new Set();
        const recursionStack = new Set();
        const hasCycle = (nodeId) => {
            visited.add(nodeId);
            recursionStack.add(nodeId);
            const outgoing = wf.connections.filter((c) => c.sourceNodeId === nodeId);
            for (const conn of outgoing) {
                if (!visited.has(conn.targetNodeId)) {
                    if (hasCycle(conn.targetNodeId))
                        return true;
                }
                else if (recursionStack.has(conn.targetNodeId)) {
                    return true;
                }
            }
            recursionStack.delete(nodeId);
            return false;
        };
        for (const node of wf.nodes) {
            if (!visited.has(node.id) && hasCycle(node.id)) {
                errors.push({
                    message: 'Workflow contains circular dependencies',
                    severity: 'error',
                    code: 'CIRCULAR_DEPENDENCY',
                });
                break;
            }
        }
        return {
            valid: errors.length === 0,
            errors,
            warnings,
        };
    }, []);
    const handleNodeAdd = useCallback((nodeData, position) => {
        if (readOnly)
            return;
        const newNode = {
            id: `node_${generateId()}`,
            type: nodeData.type || 'action',
            label: nodeData.label || 'New Node',
            position,
            data: nodeData.data || {},
            inputs: nodeData.inputs || [
                {
                    id: `port_${generateId()}`,
                    nodeId: '',
                    type: 'input',
                    label: 'Input',
                },
            ],
            outputs: nodeData.outputs || [
                {
                    id: `port_${generateId()}`,
                    nodeId: '',
                    type: 'output',
                    label: 'Output',
                },
            ],
            config: nodeData.config || {},
            metadata: {
                createdAt: new Date(),
                updatedAt: new Date(),
                createdBy: 'user',
                version: 1,
            },
        };
        newNode.inputs.forEach((port) => (port.nodeId = newNode.id));
        newNode.outputs.forEach((port) => (port.nodeId = newNode.id));
        const newWorkflow = {
            ...workflow,
            nodes: [...workflow.nodes, newNode],
            metadata: {
                ...workflow.metadata,
                updatedAt: new Date(),
                lastModifiedBy: 'user',
            },
        };
        setWorkflow(newWorkflow);
        updateHistory(newWorkflow);
        if (onWorkflowChange) {
            onWorkflowChange(newWorkflow);
        }
    }, [workflow, readOnly, updateHistory, onWorkflowChange]);
    const handleNodeUpdate = useCallback((nodeId, updates) => {
        if (readOnly)
            return;
        const newWorkflow = {
            ...workflow,
            nodes: workflow.nodes.map((node) => node.id === nodeId
                ? {
                    ...node,
                    ...updates,
                    metadata: {
                        ...node.metadata,
                        updatedAt: new Date(),
                    },
                }
                : node),
            metadata: {
                ...workflow.metadata,
                updatedAt: new Date(),
                lastModifiedBy: 'user',
            },
        };
        setWorkflow(newWorkflow);
        updateHistory(newWorkflow);
        if (onWorkflowChange) {
            onWorkflowChange(newWorkflow);
        }
    }, [workflow, readOnly, updateHistory, onWorkflowChange]);
    const handleNodeDelete = useCallback((nodeId) => {
        if (readOnly)
            return;
        const newWorkflow = {
            ...workflow,
            nodes: workflow.nodes.filter((node) => node.id !== nodeId),
            connections: workflow.connections.filter((conn) => conn.sourceNodeId !== nodeId && conn.targetNodeId !== nodeId),
            metadata: {
                ...workflow.metadata,
                updatedAt: new Date(),
                lastModifiedBy: 'user',
            },
        };
        setWorkflow(newWorkflow);
        updateHistory(newWorkflow);
        setSelectedNodeIds((prev) => prev.filter((id) => id !== nodeId));
        if (onWorkflowChange) {
            onWorkflowChange(newWorkflow);
        }
    }, [workflow, readOnly, updateHistory, onWorkflowChange]);
    const handleNodeSelect = useCallback((nodeId, multiSelect) => {
        if (multiSelect) {
            setSelectedNodeIds((prev) => prev.includes(nodeId) ? prev.filter((id) => id !== nodeId) : [...prev, nodeId]);
        }
        else {
            setSelectedNodeIds([nodeId]);
        }
        setSelectedConnectionIds([]);
    }, []);
    const handleConnectionCreate = useCallback((sourcePortId, targetPortId) => {
        if (readOnly)
            return;
        const newConnection = {
            id: `conn_${generateId()}`,
            sourceNodeId: '',
            sourcePortId,
            targetNodeId: '',
            targetPortId,
            metadata: {
                createdAt: new Date(),
                updatedAt: new Date(),
            },
        };
        for (const node of workflow.nodes) {
            const sourcePort = node.outputs.find((p) => p.id === sourcePortId);
            if (sourcePort) {
                newConnection.sourceNodeId = node.id;
            }
            const targetPort = node.inputs.find((p) => p.id === targetPortId);
            if (targetPort) {
                newConnection.targetNodeId = node.id;
            }
        }
        if (!newConnection.sourceNodeId || !newConnection.targetNodeId) {
            return;
        }
        const newWorkflow = {
            ...workflow,
            connections: [...workflow.connections, newConnection],
            metadata: {
                ...workflow.metadata,
                updatedAt: new Date(),
                lastModifiedBy: 'user',
            },
        };
        setWorkflow(newWorkflow);
        updateHistory(newWorkflow);
        if (onWorkflowChange) {
            onWorkflowChange(newWorkflow);
        }
    }, [workflow, readOnly, updateHistory, onWorkflowChange]);
    const handleConnectionDelete = useCallback((connectionId) => {
        if (readOnly)
            return;
        const newWorkflow = {
            ...workflow,
            connections: workflow.connections.filter((conn) => conn.id !== connectionId),
            metadata: {
                ...workflow.metadata,
                updatedAt: new Date(),
                lastModifiedBy: 'user',
            },
        };
        setWorkflow(newWorkflow);
        updateHistory(newWorkflow);
        setSelectedConnectionIds((prev) => prev.filter((id) => id !== connectionId));
        if (onWorkflowChange) {
            onWorkflowChange(newWorkflow);
        }
    }, [workflow, readOnly, updateHistory, onWorkflowChange]);
    const handleVariableCreate = useCallback((variable) => {
        if (readOnly)
            return;
        const newVariable = {
            ...variable,
            id: `var_${generateId()}`,
        };
        const newWorkflow = {
            ...workflow,
            variables: [...workflow.variables, newVariable],
        };
        setWorkflow(newWorkflow);
        updateHistory(newWorkflow);
        if (onWorkflowChange) {
            onWorkflowChange(newWorkflow);
        }
    }, [workflow, readOnly, updateHistory, onWorkflowChange]);
    const handleVariableUpdate = useCallback((variableId, updates) => {
        if (readOnly)
            return;
        const newWorkflow = {
            ...workflow,
            variables: workflow.variables.map((v) => v.id === variableId ? { ...v, ...updates } : v),
        };
        setWorkflow(newWorkflow);
        updateHistory(newWorkflow);
        if (onWorkflowChange) {
            onWorkflowChange(newWorkflow);
        }
    }, [workflow, readOnly, updateHistory, onWorkflowChange]);
    const handleVariableDelete = useCallback((variableId) => {
        if (readOnly)
            return;
        const newWorkflow = {
            ...workflow,
            variables: workflow.variables.filter((v) => v.id !== variableId),
        };
        setWorkflow(newWorkflow);
        updateHistory(newWorkflow);
        if (onWorkflowChange) {
            onWorkflowChange(newWorkflow);
        }
    }, [workflow, readOnly, updateHistory, onWorkflowChange]);
    const handleExecute = useCallback(() => {
        const validation = validateWorkflow(workflow);
        setValidationResult(validation);
        if (!validation.valid) {
            alert(`Cannot execute workflow:\n${validation.errors.map((e) => e.message).join('\n')}`);
            return;
        }
        if (onWorkflowExecute) {
            onWorkflowExecute(workflow);
        }
        setIsExecuting(true);
    }, [workflow, validateWorkflow, onWorkflowExecute]);
    const selectedNode = workflow.nodes.find((n) => selectedNodeIds.includes(n.id));
    return (React.createElement(DndProvider, { backend: HTML5Backend },
        React.createElement("div", { style: {
                display: 'flex',
                flexDirection: 'column',
                height: '100vh',
                backgroundColor: '#f8fafc',
            } },
            React.createElement("div", { style: {
                    height: '60px',
                    backgroundColor: '#fff',
                    borderBottom: '1px solid #e2e8f0',
                    display: 'flex',
                    alignItems: 'center',
                    padding: '0 20px',
                    gap: '12px',
                } },
                React.createElement("div", { style: { flex: 1 } },
                    React.createElement("input", { type: "text", value: workflow.name, onChange: (e) => !readOnly &&
                            setWorkflow({ ...workflow, name: e.target.value }), disabled: readOnly, style: {
                            fontSize: '18px',
                            fontWeight: 600,
                            border: 'none',
                            outline: 'none',
                            backgroundColor: 'transparent',
                            color: '#1e293b',
                        } })),
                React.createElement("button", { onClick: undo, disabled: history.past.length === 0 || readOnly, style: {
                        padding: '8px 16px',
                        backgroundColor: '#fff',
                        border: '1px solid #e2e8f0',
                        borderRadius: '6px',
                        cursor: history.past.length === 0 || readOnly ? 'not-allowed' : 'pointer',
                        opacity: history.past.length === 0 || readOnly ? 0.5 : 1,
                    }, title: "Undo (Ctrl+Z)" }, "\u21B6 Undo"),
                React.createElement("button", { onClick: redo, disabled: history.future.length === 0 || readOnly, style: {
                        padding: '8px 16px',
                        backgroundColor: '#fff',
                        border: '1px solid #e2e8f0',
                        borderRadius: '6px',
                        cursor: history.future.length === 0 || readOnly ? 'not-allowed' : 'pointer',
                        opacity: history.future.length === 0 || readOnly ? 0.5 : 1,
                    }, title: "Redo (Ctrl+Y)" }, "\u21B7 Redo"),
                React.createElement("button", { onClick: () => setActivePanel('templates'), style: {
                        padding: '8px 16px',
                        backgroundColor: activePanel === 'templates' ? '#eff6ff' : '#fff',
                        border: `1px solid ${activePanel === 'templates' ? '#3b82f6' : '#e2e8f0'}`,
                        borderRadius: '6px',
                        cursor: 'pointer',
                    } }, "Templates"),
                React.createElement("button", { onClick: () => setActivePanel('history'), style: {
                        padding: '8px 16px',
                        backgroundColor: activePanel === 'history' ? '#eff6ff' : '#fff',
                        border: `1px solid ${activePanel === 'history' ? '#3b82f6' : '#e2e8f0'}`,
                        borderRadius: '6px',
                        cursor: 'pointer',
                    } },
                    "History (",
                    executions.length,
                    ")"),
                !readOnly && onWorkflowSave && (React.createElement("button", { onClick: () => onWorkflowSave(workflow), style: {
                        padding: '8px 16px',
                        backgroundColor: '#10b981',
                        color: '#fff',
                        border: 'none',
                        borderRadius: '6px',
                        cursor: 'pointer',
                        fontWeight: 500,
                    } }, "Save")),
                React.createElement("button", { onClick: handleExecute, disabled: isExecuting || readOnly, style: {
                        padding: '8px 16px',
                        backgroundColor: isExecuting ? '#94a3b8' : '#3b82f6',
                        color: '#fff',
                        border: 'none',
                        borderRadius: '6px',
                        cursor: isExecuting || readOnly ? 'not-allowed' : 'pointer',
                        fontWeight: 500,
                    } }, isExecuting ? 'Executing...' : 'Execute')),
            React.createElement("div", { style: { flex: 1, display: 'flex', overflow: 'hidden' } },
                React.createElement("div", { style: { flex: 1 } },
                    React.createElement(WorkflowCanvas, { workflow: workflow, executions: currentExecution?.nodeExecutions, isExecuting: isExecuting, selectedNodeIds: selectedNodeIds, selectedConnectionIds: selectedConnectionIds, collaboratorCursors: collaboratorCursors, onNodeSelect: handleNodeSelect, onNodeUpdate: handleNodeUpdate, onNodeDelete: handleNodeDelete, onNodeAdd: handleNodeAdd, onConnectionCreate: handleConnectionCreate, onConnectionDelete: handleConnectionDelete, onCanvasClick: () => {
                            setSelectedNodeIds([]);
                            setSelectedConnectionIds([]);
                        }, onCursorMove: onCursorMove, readOnly: readOnly, showGrid: true, showMinimap: true })),
                React.createElement("div", { style: { width: '360px' } },
                    activePanel === 'sidebar' && (React.createElement(WorkflowSidebar, { selectedNode: selectedNode, variables: workflow.variables, onNodeUpdate: handleNodeUpdate, onVariableCreate: handleVariableCreate, onVariableUpdate: handleVariableUpdate, onVariableDelete: handleVariableDelete, readOnly: readOnly })),
                    activePanel === 'history' && (React.createElement(WorkflowHistory, { executions: executions, selectedExecutionId: currentExecution?.id, onExecutionSelect: (id) => setCurrentExecution(executions.find((e) => e.id === id) || null) })),
                    activePanel === 'templates' && (React.createElement(WorkflowTemplates, { onTemplateSelect: (template) => {
                            if (template.workflow && !readOnly) {
                                const newWorkflow = {
                                    ...template.workflow,
                                    id: generateId(),
                                    name: template.name,
                                    metadata: {
                                        ...template.workflow.metadata,
                                        createdAt: new Date(),
                                        updatedAt: new Date(),
                                        status: 'draft',
                                    },
                                };
                                setWorkflow(newWorkflow);
                                updateHistory(newWorkflow);
                                setActivePanel('sidebar');
                            }
                        } })))),
            validationResult.errors.length > 0 && (React.createElement("div", { style: {
                    position: 'fixed',
                    bottom: '20px',
                    right: '380px',
                    backgroundColor: '#fef2f2',
                    border: '1px solid #fecaca',
                    borderRadius: '8px',
                    padding: '12px 16px',
                    maxWidth: '400px',
                    boxShadow: '0 4px 6px -1px rgba(0, 0, 0, 0.1)',
                } },
                React.createElement("div", { style: { fontWeight: 600, color: '#ef4444', marginBottom: '8px' } }, "Validation Errors"),
                validationResult.errors.map((error, index) => (React.createElement("div", { key: index, style: { fontSize: '13px', color: '#64748b', marginBottom: '4px' } },
                    "\u2022 ",
                    error.message))))))));
};
export default WorkflowBuilder;
//# sourceMappingURL=WorkflowBuilder.js.map