import React, { useCallback, useMemo, useRef, useState } from 'react';
import { useDrag } from 'react-dnd';
const NODE_COLORS = {
    trigger: '#10b981',
    action: '#3b82f6',
    condition: '#f59e0b',
    loop: '#8b5cf6',
    delay: '#6366f1',
    transform: '#06b6d4',
    api: '#ec4899',
    email: '#ef4444',
    webhook: '#14b8a6',
    database: '#84cc16',
    script: '#64748b',
};
const STATUS_COLORS = {
    idle: '#94a3b8',
    running: '#3b82f6',
    paused: '#f59e0b',
    completed: '#10b981',
    failed: '#ef4444',
    cancelled: '#64748b',
    retrying: '#f97316',
};
export const WorkflowNode = ({ node, isSelected = false, isExecuting = false, executionStatus = 'idle', executionProgress = 0, onSelect, onUpdate, onDelete, onPortConnect, onPortDisconnect, readOnly = false, showPorts = true, zoom = 1, }) => {
    const nodeRef = useRef(null);
    const [isHovered, setIsHovered] = useState(false);
    const [isDraggingPort, setIsDraggingPort] = useState(false);
    const [{ isDragging }, dragRef] = useDrag({
        type: 'workflow-node',
        item: { id: node.id, type: 'node' },
        collect: (monitor) => ({
            isDragging: monitor.isDragging(),
        }),
        canDrag: !readOnly,
    });
    const nodeColor = useMemo(() => {
        return node.data.color || NODE_COLORS[node.type] || '#64748b';
    }, [node.type, node.data.color]);
    const statusColor = useMemo(() => {
        return STATUS_COLORS[executionStatus];
    }, [executionStatus]);
    const handleClick = useCallback((e) => {
        e.stopPropagation();
        if (onSelect) {
            onSelect(node.id, e.shiftKey || e.metaKey || e.ctrlKey);
        }
    }, [node.id, onSelect]);
    const handleDoubleClick = useCallback((e) => {
        e.stopPropagation();
    }, []);
    const handleDelete = useCallback((e) => {
        e.stopPropagation();
        if (onDelete && !readOnly) {
            onDelete(node.id);
        }
    }, [node.id, onDelete, readOnly]);
    const handlePortMouseDown = useCallback((port, e) => {
        e.stopPropagation();
        if (onPortConnect && !readOnly) {
            setIsDraggingPort(true);
            onPortConnect(port.id, port.type);
        }
    }, [onPortConnect, readOnly]);
    const handlePortMouseUp = useCallback(() => {
        setIsDraggingPort(false);
    }, []);
    const renderPort = useCallback((port, index) => {
        const isInput = port.type === 'input';
        const portStyle = {
            position: 'absolute',
            width: '12px',
            height: '12px',
            borderRadius: '50%',
            backgroundColor: port.required ? '#ef4444' : '#64748b',
            border: '2px solid #fff',
            cursor: readOnly ? 'default' : 'crosshair',
            top: isInput ? `${(index + 1) * (100 / (node.inputs.length + 1))}%` : `${(index + 1) * (100 / (node.outputs.length + 1))}%`,
            left: isInput ? '-6px' : undefined,
            right: !isInput ? '-6px' : undefined,
            transform: 'translateY(-50%)',
            zIndex: 10,
            transition: 'all 0.2s ease',
        };
        return (React.createElement("div", { key: port.id, className: "workflow-node-port", style: portStyle, onMouseDown: (e) => handlePortMouseDown(port, e), onMouseUp: handlePortMouseUp, title: `${port.label}${port.required ? ' (Required)' : ''}` }));
    }, [node.inputs.length, node.outputs.length, handlePortMouseDown, handlePortMouseUp, readOnly]);
    const nodeStyle = {
        position: 'absolute',
        left: `${node.position.x}px`,
        top: `${node.position.y}px`,
        width: '200px',
        minHeight: '80px',
        backgroundColor: '#fff',
        borderRadius: '8px',
        border: `2px solid ${isSelected ? nodeColor : '#e2e8f0'}`,
        boxShadow: isSelected
            ? `0 0 0 3px ${nodeColor}33, 0 10px 25px -5px rgba(0, 0, 0, 0.2)`
            : isHovered
                ? '0 10px 25px -5px rgba(0, 0, 0, 0.15)'
                : '0 4px 6px -1px rgba(0, 0, 0, 0.1)',
        cursor: readOnly ? 'default' : 'move',
        opacity: isDragging ? 0.5 : 1,
        transition: 'all 0.2s ease',
        transform: `scale(${zoom})`,
        transformOrigin: 'top left',
        zIndex: isSelected ? 100 : isDragging ? 99 : 10,
    };
    const headerStyle = {
        backgroundColor: nodeColor,
        color: '#fff',
        padding: '8px 12px',
        borderTopLeftRadius: '6px',
        borderTopRightRadius: '6px',
        fontSize: '14px',
        fontWeight: 600,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
    };
    const bodyStyle = {
        padding: '12px',
        fontSize: '12px',
        color: '#64748b',
    };
    const statusBadgeStyle = {
        position: 'absolute',
        top: '-8px',
        right: '-8px',
        width: '24px',
        height: '24px',
        borderRadius: '50%',
        backgroundColor: statusColor,
        border: '2px solid #fff',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        fontSize: '10px',
        color: '#fff',
        fontWeight: 'bold',
        zIndex: 20,
    };
    const progressBarStyle = {
        position: 'absolute',
        bottom: 0,
        left: 0,
        height: '3px',
        width: `${executionProgress}%`,
        backgroundColor: statusColor,
        transition: 'width 0.3s ease',
        borderBottomLeftRadius: '6px',
        borderBottomRightRadius: executionProgress === 100 ? '6px' : '0',
    };
    return (React.createElement("div", { ref: (el) => {
            nodeRef.current = el;
            if (!readOnly)
                dragRef(el);
        }, style: nodeStyle, onClick: handleClick, onDoubleClick: handleDoubleClick, onMouseEnter: () => setIsHovered(true), onMouseLeave: () => setIsHovered(false), className: "workflow-node", "data-node-id": node.id },
        isExecuting && (React.createElement("div", { style: statusBadgeStyle }, executionStatus === 'running' ? '⚡' : executionStatus === 'completed' ? '✓' : executionStatus === 'failed' ? '✗' : '⏸')),
        React.createElement("div", { style: headerStyle },
            React.createElement("span", null, node.label),
            !readOnly && isHovered && (React.createElement("button", { onClick: handleDelete, style: {
                    background: 'transparent',
                    border: 'none',
                    color: '#fff',
                    cursor: 'pointer',
                    fontSize: '16px',
                    padding: '0 4px',
                }, title: "Delete node" }, "\u00D7"))),
        React.createElement("div", { style: bodyStyle },
            node.data.description && React.createElement("div", { style: { marginBottom: '4px' } }, node.data.description),
            React.createElement("div", { style: { fontSize: '11px', color: '#94a3b8' } }, node.type.charAt(0).toUpperCase() + node.type.slice(1))),
        showPorts && node.inputs.map((port, index) => renderPort(port, index)),
        showPorts && node.outputs.map((port, index) => renderPort(port, index)),
        isExecuting && executionProgress > 0 && React.createElement("div", { style: progressBarStyle })));
};
export default WorkflowNode;
//# sourceMappingURL=WorkflowNode.js.map