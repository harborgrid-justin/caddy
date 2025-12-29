/**
 * CADDY v0.4.0 - Workflow Canvas Component
 * Main canvas with zoom, pan, minimap, and grid
 */

import React, { useCallback, useRef, useState, useEffect, useMemo } from 'react';
import { useDrop } from 'react-dnd';
import WorkflowNode from './WorkflowNode';
import WorkflowConnector from './WorkflowConnector';
import type {
  Workflow,
  WorkflowNode as WorkflowNodeType,
  WorkflowConnection,
  Position,
  CanvasState,
  NodeExecution,
  UserCursor,
} from './types';

export interface WorkflowCanvasProps {
  workflow: Workflow;
  executions?: NodeExecution[];
  isExecuting?: boolean;
  selectedNodeIds?: string[];
  selectedConnectionIds?: string[];
  collaboratorCursors?: UserCursor[];
  onNodeSelect?: (nodeId: string, multiSelect: boolean) => void;
  onNodeUpdate?: (nodeId: string, updates: Partial<WorkflowNodeType>) => void;
  onNodeDelete?: (nodeId: string) => void;
  onNodeAdd?: (node: Partial<WorkflowNodeType>, position: Position) => void;
  onConnectionCreate?: (sourcePortId: string, targetPortId: string) => void;
  onConnectionDelete?: (connectionId: string) => void;
  onCanvasClick?: () => void;
  onCursorMove?: (position: Position) => void;
  readOnly?: boolean;
  showGrid?: boolean;
  showMinimap?: boolean;
}

export const WorkflowCanvas: React.FC<WorkflowCanvasProps> = ({
  workflow,
  executions = [],
  isExecuting = false,
  selectedNodeIds = [],
  selectedConnectionIds = [],
  collaboratorCursors = [],
  onNodeSelect,
  onNodeUpdate,
  onNodeDelete,
  onNodeAdd,
  onConnectionCreate,
  onConnectionDelete,
  onCanvasClick,
  onCursorMove,
  readOnly = false,
  showGrid = true,
  showMinimap = true,
}) => {
  const canvasRef = useRef<HTMLDivElement>(null);
  const svgRef = useRef<SVGSVGElement>(null);
  const [canvasState, setCanvasState] = useState<CanvasState>({
    zoom: 1,
    offset: { x: 0, y: 0 },
    selectedNodes: [],
    selectedConnections: [],
    isDragging: false,
    isPanning: false,
  });

  const [isPanning, setIsPanning] = useState(false);
  const [panStart, setPanStart] = useState<Position>({ x: 0, y: 0 });
  const [connectingPort, setConnectingPort] = useState<{
    portId: string;
    portType: 'input' | 'output';
    position: Position;
  } | null>(null);
  const [tempConnectionEnd, setTempConnectionEnd] = useState<Position | null>(null);

  const [{ isOver }, dropRef] = useDrop({
    accept: ['workflow-node-palette', 'workflow-node'],
    drop: (item: any, monitor) => {
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

  // Zoom handling
  const handleWheel = useCallback(
    (e: WheelEvent) => {
      if (e.ctrlKey || e.metaKey) {
        e.preventDefault();
        const delta = e.deltaY > 0 ? 0.9 : 1.1;
        setCanvasState((prev) => ({
          ...prev,
          zoom: Math.max(0.1, Math.min(3, prev.zoom * delta)),
        }));
      }
    },
    []
  );

  // Pan handling
  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      if (e.button === 1 || (e.button === 0 && e.spaceKey)) {
        // Middle mouse button or space + left click
        setIsPanning(true);
        setPanStart({ x: e.clientX - canvasState.offset.x, y: e.clientY - canvasState.offset.y });
      } else if (e.button === 0 && e.target === e.currentTarget) {
        // Left click on canvas background
        if (onCanvasClick) {
          onCanvasClick();
        }
      }
    },
    [canvasState.offset, onCanvasClick]
  );

  const handleMouseMove = useCallback(
    (e: React.MouseEvent) => {
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

      // Send cursor position to collaborators
      if (onCursorMove && canvasRef.current) {
        const rect = canvasRef.current.getBoundingClientRect();
        onCursorMove({
          x: (e.clientX - rect.left - canvasState.offset.x) / canvasState.zoom,
          y: (e.clientY - rect.top - canvasState.offset.y) / canvasState.zoom,
        });
      }
    },
    [isPanning, panStart, connectingPort, canvasState.offset, canvasState.zoom, onCursorMove]
  );

  const handleMouseUp = useCallback(() => {
    setIsPanning(false);
    if (connectingPort) {
      setConnectingPort(null);
      setTempConnectionEnd(null);
    }
  }, [connectingPort]);

  // Keyboard shortcuts
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
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
      } else if (e.key === '0' && (e.ctrlKey || e.metaKey)) {
        // Reset zoom
        e.preventDefault();
        setCanvasState((prev) => ({ ...prev, zoom: 1, offset: { x: 0, y: 0 } }));
      } else if (e.key === '=' && (e.ctrlKey || e.metaKey)) {
        // Zoom in
        e.preventDefault();
        setCanvasState((prev) => ({ ...prev, zoom: Math.min(3, prev.zoom * 1.1) }));
      } else if (e.key === '-' && (e.ctrlKey || e.metaKey)) {
        // Zoom out
        e.preventDefault();
        setCanvasState((prev) => ({ ...prev, zoom: Math.max(0.1, prev.zoom * 0.9) }));
      }
    },
    [selectedNodeIds, selectedConnectionIds, onNodeDelete, onConnectionDelete, readOnly]
  );

  // Get node execution status
  const getNodeExecution = useCallback(
    (nodeId: string): NodeExecution | undefined => {
      return executions.find((exec) => exec.nodeId === nodeId);
    },
    [executions]
  );

  // Get port position for connections
  const getPortPosition = useCallback(
    (nodeId: string, portId: string): Position => {
      const node = workflow.nodes.find((n) => n.id === nodeId);
      if (!node) return { x: 0, y: 0 };

      const portIndex = [...node.inputs, ...node.outputs].findIndex((p) => p.id === portId);
      const isInput = node.inputs.some((p) => p.id === portId);
      const portCount = isInput ? node.inputs.length : node.outputs.length;
      const index = isInput ? node.inputs.findIndex((p) => p.id === portId) : node.outputs.findIndex((p) => p.id === portId);

      return {
        x: node.position.x + (isInput ? 0 : 200),
        y: node.position.y + ((index + 1) * (80 / (portCount + 1))),
      };
    },
    [workflow.nodes]
  );

  // Handle port connection
  const handlePortConnect = useCallback(
    (portId: string, portType: 'input' | 'output') => {
      const port = workflow.nodes
        .flatMap((n) => [...n.inputs, ...n.outputs])
        .find((p) => p.id === portId);

      if (!port) return;

      const node = workflow.nodes.find((n) => n.id === port.nodeId);
      if (!node) return;

      setConnectingPort({
        portId,
        portType,
        position: getPortPosition(node.id, portId),
      });
    },
    [workflow.nodes, getPortPosition]
  );

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

  const canvasStyle: React.CSSProperties = {
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

  const workspaceStyle: React.CSSProperties = {
    position: 'absolute',
    transform: `translate(${canvasState.offset.x}px, ${canvasState.offset.y}px) scale(${canvasState.zoom})`,
    transformOrigin: '0 0',
    width: '100%',
    height: '100%',
  };

  return (
    <div
      ref={(el) => {
        canvasRef.current = el;
        if (!readOnly) dropRef(el);
      }}
      style={canvasStyle}
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      className="workflow-canvas"
    >
      {/* Workspace */}
      <div style={workspaceStyle}>
        {/* SVG Layer for Connections */}
        <svg
          ref={svgRef}
          style={{
            position: 'absolute',
            top: 0,
            left: 0,
            width: '100%',
            height: '100%',
            pointerEvents: 'none',
            overflow: 'visible',
          }}
        >
          <g style={{ pointerEvents: 'auto' }}>
            {/* Render connections */}
            {workflow.connections.map((connection) => {
              const sourcePos = getPortPosition(connection.sourceNodeId, connection.sourcePortId);
              const targetPos = getPortPosition(connection.targetNodeId, connection.targetPortId);
              const isExecuting = executions.some(
                (exec) =>
                  exec.nodeId === connection.targetNodeId &&
                  exec.status === 'running'
              );

              return (
                <WorkflowConnector
                  key={connection.id}
                  connection={connection}
                  sourcePosition={sourcePos}
                  targetPosition={targetPos}
                  isSelected={selectedConnectionIds.includes(connection.id)}
                  isExecuting={isExecuting}
                  isAnimated={isExecuting}
                  onDelete={onConnectionDelete}
                  readOnly={readOnly}
                  zoom={canvasState.zoom}
                />
              );
            })}

            {/* Temporary connection while dragging */}
            {connectingPort && tempConnectionEnd && (
              <path
                d={`M ${connectingPort.position.x} ${connectingPort.position.y} L ${tempConnectionEnd.x} ${tempConnectionEnd.y}`}
                stroke="#3b82f6"
                strokeWidth="2"
                strokeDasharray="5,5"
                fill="none"
              />
            )}
          </g>
        </svg>

        {/* Nodes Layer */}
        {workflow.nodes.map((node) => {
          const execution = getNodeExecution(node.id);
          return (
            <WorkflowNode
              key={node.id}
              node={node}
              isSelected={selectedNodeIds.includes(node.id)}
              isExecuting={isExecuting && !!execution}
              executionStatus={execution?.status}
              executionProgress={execution ? ((execution.duration || 0) / 1000) * 10 : 0}
              onSelect={onNodeSelect}
              onUpdate={onNodeUpdate}
              onDelete={onNodeDelete}
              onPortConnect={handlePortConnect}
              readOnly={readOnly}
              zoom={canvasState.zoom}
            />
          );
        })}

        {/* Collaborator Cursors */}
        {collaboratorCursors.map((cursor) => (
          <div
            key={cursor.userId}
            style={{
              position: 'absolute',
              left: cursor.position.x,
              top: cursor.position.y,
              pointerEvents: 'none',
              zIndex: 1000,
            }}
          >
            <svg width="20" height="20" viewBox="0 0 20 20">
              <path
                d="M0 0 L0 16 L6 12 L10 20 L12 19 L8 11 L16 10 Z"
                fill={cursor.color}
                stroke="#fff"
                strokeWidth="1"
              />
            </svg>
            <div
              style={{
                marginTop: '4px',
                padding: '2px 6px',
                backgroundColor: cursor.color,
                color: '#fff',
                borderRadius: '3px',
                fontSize: '11px',
                whiteSpace: 'nowrap',
              }}
            >
              {cursor.userName}
            </div>
          </div>
        ))}
      </div>

      {/* Controls */}
      <div
        style={{
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
        }}
      >
        <button
          onClick={() => setCanvasState((prev) => ({ ...prev, zoom: Math.min(3, prev.zoom * 1.1) }))}
          style={{
            padding: '8px 12px',
            backgroundColor: '#fff',
            border: '1px solid #e2e8f0',
            borderRadius: '4px',
            cursor: 'pointer',
            fontSize: '16px',
          }}
          title="Zoom In"
        >
          +
        </button>
        <div style={{ textAlign: 'center', fontSize: '12px', color: '#64748b' }}>
          {Math.round(canvasState.zoom * 100)}%
        </div>
        <button
          onClick={() => setCanvasState((prev) => ({ ...prev, zoom: Math.max(0.1, prev.zoom * 0.9) }))}
          style={{
            padding: '8px 12px',
            backgroundColor: '#fff',
            border: '1px solid #e2e8f0',
            borderRadius: '4px',
            cursor: 'pointer',
            fontSize: '16px',
          }}
          title="Zoom Out"
        >
          -
        </button>
        <button
          onClick={() => setCanvasState((prev) => ({ ...prev, zoom: 1, offset: { x: 0, y: 0 } }))}
          style={{
            padding: '8px 12px',
            backgroundColor: '#fff',
            border: '1px solid #e2e8f0',
            borderRadius: '4px',
            cursor: 'pointer',
            fontSize: '12px',
          }}
          title="Reset View"
        >
          ⟲
        </button>
      </div>

      {/* Drop indicator */}
      {isOver && !readOnly && (
        <div
          style={{
            position: 'absolute',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            backgroundColor: 'rgba(59, 130, 246, 0.1)',
            border: '2px dashed #3b82f6',
            pointerEvents: 'none',
          }}
        />
      )}
    </div>
  );
};

export default WorkflowCanvas;
