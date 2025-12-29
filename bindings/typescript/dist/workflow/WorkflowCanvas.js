import React, { useCallback, useRef, useState, useEffect } from 'react';
import { useDrop } from 'react-dnd';
import WorkflowNode from './WorkflowNode';
import WorkflowConnector from './WorkflowConnector';
export const WorkflowCanvas = ({ workflow, executions = [], isExecuting = false, selectedNodeIds = [], selectedConnectionIds = [], collaboratorCursors = [], onNodeSelect, onNodeUpdate, onNodeDelete, onNodeAdd, onConnectionCreate, onConnectionDelete, onCanvasClick, onCursorMove, readOnly = false, showGrid = true, showMinimap = true, }) => {
    const canvasRef = useRef(null);
    const svgRef = useRef(null);
    const [canvasState, setCanvasState] = useState({
        zoom: 1,
        offset: { x: 0, y: 0 },
        selectedNodes: [],
        selectedConnections: [],
        isDragging: false,
        isPanning: false,
    });
    const [isPanning, setIsPanning] = useState(false);
    const [panStart, setPanStart] = useState({ x: 0, y: 0 });
    const [connectingPort, setConnectingPort] = useState(null);
    const [tempConnectionEnd, setTempConnectionEnd] = useState(null);
    const [{ isOver }, dropRef] = useDrop({
        accept: ['workflow-node-palette', 'workflow-node'],
        drop: (item, monitor) => {
            if (!readOnly && onNodeAdd) {
                const offset = monitor.getClientOffset();
                if (offset && canvasRef.current) {
                    const rect = canvasRef.current.getBoundingClientRect();
                    const position = {
                        x: (offset.x - rect.left - canvasState.offset.x) / canvasState.zoom,
                        y: (offset.y - rect.top - canvasState.offset.y) / canvasState.zoom,
                    };
                    onNodeAdd(item.nodeData, position);
                }
            }
        },
        collect: (monitor) => ({
            isOver: monitor.isOver(),
        }),
    });
    const handleWheel = useCallback((e) => {
        if (e.ctrlKey || e.metaKey) {
            e.preventDefault();
            const delta = e.deltaY > 0 ? 0.9 : 1.1;
            setCanvasState((prev) => ({
                ...prev,
                zoom: Math.max(0.1, Math.min(3, prev.zoom * delta)),
            }));
        }
    }, []);
    const handleMouseDown = useCallback((e) => {
        if (e.button === 1 || (e.button === 0 && e.spaceKey)) {
            setIsPanning(true);
            setPanStart({ x: e.clientX - canvasState.offset.x, y: e.clientY - canvasState.offset.y });
        }
        else if (e.button === 0 && e.target === e.currentTarget) {
            if (onCanvasClick) {
                onCanvasClick();
            }
        }
    }, [canvasState.offset, onCanvasClick]);
    const handleMouseMove = useCallback((e) => {
        if (isPanning) {
            setCanvasState((prev) => ({
                ...prev,
                offset: {
                    x: e.clientX - panStart.x,
                    y: e.clientY - panStart.y,
                },
            }));
        }
        if (connectingPort && canvasRef.current) {
            const rect = canvasRef.current.getBoundingClientRect();
            setTempConnectionEnd({
                x: (e.clientX - rect.left - canvasState.offset.x) / canvasState.zoom,
                y: (e.clientY - rect.top - canvasState.offset.y) / canvasState.zoom,
            });
        }
        if (onCursorMove && canvasRef.current) {
            const rect = canvasRef.current.getBoundingClientRect();
            onCursorMove({
                x: (e.clientX - rect.left - canvasState.offset.x) / canvasState.zoom,
                y: (e.clientY - rect.top - canvasState.offset.y) / canvasState.zoom,
            });
        }
    }, [isPanning, panStart, connectingPort, canvasState.offset, canvasState.zoom, onCursorMove]);
    const handleMouseUp = useCallback(() => {
        setIsPanning(false);
        if (connectingPort) {
            setConnectingPort(null);
            setTempConnectionEnd(null);
        }
    }, [connectingPort]);
    const handleKeyDown = useCallback((e) => {
        if (e.key === 'Delete' || e.key === 'Backspace') {
            selectedNodeIds.forEach((nodeId) => {
                if (onNodeDelete && !readOnly) {
                    onNodeDelete(nodeId);
                }
            });
            selectedConnectionIds.forEach((connId) => {
                if (onConnectionDelete && !readOnly) {
                    onConnectionDelete(connId);
                }
            });
        }
        else if (e.key === '0' && (e.ctrlKey || e.metaKey)) {
            e.preventDefault();
            setCanvasState((prev) => ({ ...prev, zoom: 1, offset: { x: 0, y: 0 } }));
        }
        else if (e.key === '=' && (e.ctrlKey || e.metaKey)) {
            e.preventDefault();
            setCanvasState((prev) => ({ ...prev, zoom: Math.min(3, prev.zoom * 1.1) }));
        }
        else if (e.key === '-' && (e.ctrlKey || e.metaKey)) {
            e.preventDefault();
            setCanvasState((prev) => ({ ...prev, zoom: Math.max(0.1, prev.zoom * 0.9) }));
        }
    }, [selectedNodeIds, selectedConnectionIds, onNodeDelete, onConnectionDelete, readOnly]);
    const getNodeExecution = useCallback((nodeId) => {
        return executions.find((exec) => exec.nodeId === nodeId);
    }, [executions]);
    const getPortPosition = useCallback((nodeId, portId) => {
        const node = workflow.nodes.find((n) => n.id === nodeId);
        if (!node)
            return { x: 0, y: 0 };
        const portIndex = [...node.inputs, ...node.outputs].findIndex((p) => p.id === portId);
        const isInput = node.inputs.some((p) => p.id === portId);
        const portCount = isInput ? node.inputs.length : node.outputs.length;
        const index = isInput ? node.inputs.findIndex((p) => p.id === portId) : node.outputs.findIndex((p) => p.id === portId);
        return {
            x: node.position.x + (isInput ? 0 : 200),
            y: node.position.y + ((index + 1) * (80 / (portCount + 1))),
        };
    }, [workflow.nodes]);
    const handlePortConnect = useCallback((portId, portType) => {
        const port = workflow.nodes
            .flatMap((n) => [...n.inputs, ...n.outputs])
            .find((p) => p.id === portId);
        if (!port)
            return;
        const node = workflow.nodes.find((n) => n.id === port.nodeId);
        if (!node)
            return;
        setConnectingPort({
            portId,
            portType,
            position: getPortPosition(node.id, portId),
        });
    }, [workflow.nodes, getPortPosition]);
    useEffect(() => {
        const canvas = canvasRef.current;
        if (canvas) {
            canvas.addEventListener('wheel', handleWheel, { passive: false });
            window.addEventListener('keydown', handleKeyDown);
            return () => {
                canvas.removeEventListener('wheel', handleWheel);
                window.removeEventListener('keydown', handleKeyDown);
            };
        }
        return undefined;
    }, [handleWheel, handleKeyDown]);
    const canvasStyle = {
        position: 'relative',
        width: '100%',
        height: '100%',
        overflow: 'hidden',
        backgroundColor: '#f8fafc',
        backgroundImage: showGrid
            ? `
        linear-gradient(#e2e8f0 1px, transparent 1px),
        linear-gradient(90deg, #e2e8f0 1px, transparent 1px)
      `
            : undefined,
        backgroundSize: showGrid ? `${20 * canvasState.zoom}px ${20 * canvasState.zoom}px` : undefined,
        backgroundPosition: showGrid ? `${canvasState.offset.x}px ${canvasState.offset.y}px` : undefined,
        cursor: isPanning ? 'grabbing' : 'grab',
    };
    const workspaceStyle = {
        position: 'absolute',
        transform: `translate(${canvasState.offset.x}px, ${canvasState.offset.y}px) scale(${canvasState.zoom})`,
        transformOrigin: '0 0',
        width: '100%',
        height: '100%',
    };
    return (React.createElement("div", { ref: (el) => {
            canvasRef.current = el;
            if (!readOnly)
                dropRef(el);
        }, style: canvasStyle, onMouseDown: handleMouseDown, onMouseMove: handleMouseMove, onMouseUp: handleMouseUp, className: "workflow-canvas" },
        React.createElement("div", { style: workspaceStyle },
            React.createElement("svg", { ref: svgRef, style: {
                    position: 'absolute',
                    top: 0,
                    left: 0,
                    width: '100%',
                    height: '100%',
                    pointerEvents: 'none',
                    overflow: 'visible',
                } },
                React.createElement("g", { style: { pointerEvents: 'auto' } },
                    workflow.connections.map((connection) => {
                        const sourcePos = getPortPosition(connection.sourceNodeId, connection.sourcePortId);
                        const targetPos = getPortPosition(connection.targetNodeId, connection.targetPortId);
                        const isExecuting = executions.some((exec) => exec.nodeId === connection.targetNodeId &&
                            exec.status === 'running');
                        return (React.createElement(WorkflowConnector, { key: connection.id, connection: connection, sourcePosition: sourcePos, targetPosition: targetPos, isSelected: selectedConnectionIds.includes(connection.id), isExecuting: isExecuting, isAnimated: isExecuting, onDelete: onConnectionDelete, readOnly: readOnly, zoom: canvasState.zoom }));
                    }),
                    connectingPort && tempConnectionEnd && (React.createElement("path", { d: `M ${connectingPort.position.x} ${connectingPort.position.y} L ${tempConnectionEnd.x} ${tempConnectionEnd.y}`, stroke: "#3b82f6", strokeWidth: "2", strokeDasharray: "5,5", fill: "none" })))),
            workflow.nodes.map((node) => {
                const execution = getNodeExecution(node.id);
                return (React.createElement(WorkflowNode, { key: node.id, node: node, isSelected: selectedNodeIds.includes(node.id), isExecuting: isExecuting && !!execution, executionStatus: execution?.status, executionProgress: execution ? ((execution.duration || 0) / 1000) * 10 : 0, onSelect: onNodeSelect, onUpdate: onNodeUpdate, onDelete: onNodeDelete, onPortConnect: handlePortConnect, readOnly: readOnly, zoom: canvasState.zoom }));
            }),
            collaboratorCursors.map((cursor) => (React.createElement("div", { key: cursor.userId, style: {
                    position: 'absolute',
                    left: cursor.position.x,
                    top: cursor.position.y,
                    pointerEvents: 'none',
                    zIndex: 1000,
                } },
                React.createElement("svg", { width: "20", height: "20", viewBox: "0 0 20 20" },
                    React.createElement("path", { d: "M0 0 L0 16 L6 12 L10 20 L12 19 L8 11 L16 10 Z", fill: cursor.color, stroke: "#fff", strokeWidth: "1" })),
                React.createElement("div", { style: {
                        marginTop: '4px',
                        padding: '2px 6px',
                        backgroundColor: cursor.color,
                        color: '#fff',
                        borderRadius: '3px',
                        fontSize: '11px',
                        whiteSpace: 'nowrap',
                    } }, cursor.userName))))),
        React.createElement("div", { style: {
                position: 'absolute',
                bottom: '20px',
                right: '20px',
                display: 'flex',
                flexDirection: 'column',
                gap: '8px',
                backgroundColor: '#fff',
                borderRadius: '8px',
                padding: '8px',
                boxShadow: '0 4px 6px -1px rgba(0, 0, 0, 0.1)',
            } },
            React.createElement("button", { onClick: () => setCanvasState((prev) => ({ ...prev, zoom: Math.min(3, prev.zoom * 1.1) })), style: {
                    padding: '8px 12px',
                    backgroundColor: '#fff',
                    border: '1px solid #e2e8f0',
                    borderRadius: '4px',
                    cursor: 'pointer',
                    fontSize: '16px',
                }, title: "Zoom In" }, "+"),
            React.createElement("div", { style: { textAlign: 'center', fontSize: '12px', color: '#64748b' } },
                Math.round(canvasState.zoom * 100),
                "%"),
            React.createElement("button", { onClick: () => setCanvasState((prev) => ({ ...prev, zoom: Math.max(0.1, prev.zoom * 0.9) })), style: {
                    padding: '8px 12px',
                    backgroundColor: '#fff',
                    border: '1px solid #e2e8f0',
                    borderRadius: '4px',
                    cursor: 'pointer',
                    fontSize: '16px',
                }, title: "Zoom Out" }, "-"),
            React.createElement("button", { onClick: () => setCanvasState((prev) => ({ ...prev, zoom: 1, offset: { x: 0, y: 0 } })), style: {
                    padding: '8px 12px',
                    backgroundColor: '#fff',
                    border: '1px solid #e2e8f0',
                    borderRadius: '4px',
                    cursor: 'pointer',
                    fontSize: '12px',
                }, title: "Reset View" }, "\u27F2")),
        isOver && !readOnly && (React.createElement("div", { style: {
                position: 'absolute',
                top: 0,
                left: 0,
                right: 0,
                bottom: 0,
                backgroundColor: 'rgba(59, 130, 246, 0.1)',
                border: '2px dashed #3b82f6',
                pointerEvents: 'none',
            } }))));
};
export default WorkflowCanvas;
//# sourceMappingURL=WorkflowCanvas.js.map