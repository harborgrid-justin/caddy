/**
 * CADDY v0.4.0 - Workflow Connector Component
 * Connection lines between workflow nodes with animations
 */

import React, { useMemo, useCallback } from 'react';
import type { WorkflowConnection, Position } from './types';

export interface WorkflowConnectorProps {
  connection: WorkflowConnection;
  sourcePosition: Position;
  targetPosition: Position;
  isSelected?: boolean;
  isAnimated?: boolean;
  isExecuting?: boolean;
  color?: string;
  onClick?: (connectionId: string) => void;
  onDelete?: (connectionId: string) => void;
  readOnly?: boolean;
  zoom?: number;
}

export const WorkflowConnector: React.FC<WorkflowConnectorProps> = ({
  connection,
  sourcePosition,
  targetPosition,
  isSelected = false,
  isAnimated = false,
  isExecuting = false,
  color = '#64748b',
  onClick,
  onDelete,
  readOnly = false,
  zoom = 1,
}) => {
  const handleClick = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      if (onClick) {
        onClick(connection.id);
      }
    },
    [connection.id, onClick]
  );

  const handleDelete = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      if (onDelete && !readOnly) {
        onDelete(connection.id);
      }
    },
    [connection.id, onDelete, readOnly]
  );

  const pathData = useMemo(() => {
    const dx = targetPosition.x - sourcePosition.x;
    const dy = targetPosition.y - sourcePosition.y;

    // Control point offset for smooth curves
    const controlPointOffset = Math.min(Math.abs(dx) / 2, 150);

    // Bezier curve path
    const path = `
      M ${sourcePosition.x} ${sourcePosition.y}
      C ${sourcePosition.x + controlPointOffset} ${sourcePosition.y},
        ${targetPosition.x - controlPointOffset} ${targetPosition.y},
        ${targetPosition.x} ${targetPosition.y}
    `;

    return path.trim();
  }, [sourcePosition, targetPosition]);

  const midPoint = useMemo(() => {
    // Calculate approximate midpoint on the bezier curve
    const t = 0.5;
    const dx = targetPosition.x - sourcePosition.x;
    const controlPointOffset = Math.min(Math.abs(dx) / 2, 150);

    const p0 = sourcePosition;
    const p1 = { x: sourcePosition.x + controlPointOffset, y: sourcePosition.y };
    const p2 = { x: targetPosition.x - controlPointOffset, y: targetPosition.y };
    const p3 = targetPosition;

    const x = Math.pow(1 - t, 3) * p0.x +
              3 * Math.pow(1 - t, 2) * t * p1.x +
              3 * (1 - t) * Math.pow(t, 2) * p2.x +
              Math.pow(t, 3) * p3.x;

    const y = Math.pow(1 - t, 3) * p0.y +
              3 * Math.pow(1 - t, 2) * t * p1.y +
              3 * (1 - t) * Math.pow(t, 2) * p2.y +
              Math.pow(t, 3) * p3.y;

    return { x, y };
  }, [sourcePosition, targetPosition]);

  const strokeColor = isSelected ? '#3b82f6' : isExecuting ? '#10b981' : color;
  const strokeWidth = isSelected ? 3 : 2;

  return (
    <g className="workflow-connector" data-connection-id={connection.id}>
      {/* Invisible wider path for easier clicking */}
      <path
        d={pathData}
        fill="none"
        stroke="transparent"
        strokeWidth={20}
        onClick={handleClick}
        style={{ cursor: readOnly ? 'default' : 'pointer' }}
      />

      {/* Visible path */}
      <path
        d={pathData}
        fill="none"
        stroke={strokeColor}
        strokeWidth={strokeWidth}
        strokeLinecap="round"
        strokeDasharray={isAnimated ? '5,5' : undefined}
        style={{
          transition: 'all 0.2s ease',
          filter: isSelected ? 'drop-shadow(0 0 3px rgba(59, 130, 246, 0.5))' : undefined,
        }}
      >
        {isAnimated && (
          <animate
            attributeName="stroke-dashoffset"
            from="0"
            to="10"
            dur="0.5s"
            repeatCount="indefinite"
          />
        )}
      </path>

      {/* Execution flow animation */}
      {isExecuting && (
        <circle r="4" fill="#10b981">
          <animateMotion dur="1.5s" repeatCount="indefinite" path={pathData} />
        </circle>
      )}

      {/* Arrow head */}
      <defs>
        <marker
          id={`arrowhead-${connection.id}`}
          markerWidth="10"
          markerHeight="10"
          refX="9"
          refY="3"
          orient="auto"
          markerUnits="strokeWidth"
        >
          <path d="M0,0 L0,6 L9,3 z" fill={strokeColor} />
        </marker>
      </defs>
      <path
        d={pathData}
        fill="none"
        stroke={strokeColor}
        strokeWidth={strokeWidth}
        markerEnd={`url(#arrowhead-${connection.id})`}
        pointerEvents="none"
      />

      {/* Label */}
      {connection.label && (
        <g transform={`translate(${midPoint.x}, ${midPoint.y})`}>
          <rect
            x="-30"
            y="-10"
            width="60"
            height="20"
            fill="#fff"
            stroke={strokeColor}
            strokeWidth="1"
            rx="3"
          />
          <text
            x="0"
            y="5"
            textAnchor="middle"
            fontSize="11"
            fill="#64748b"
            fontWeight="500"
          >
            {connection.label}
          </text>
        </g>
      )}

      {/* Delete button */}
      {isSelected && !readOnly && (
        <g
          transform={`translate(${midPoint.x}, ${midPoint.y})`}
          onClick={handleDelete}
          style={{ cursor: 'pointer' }}
        >
          <circle
            cx="20"
            cy="-15"
            r="10"
            fill="#ef4444"
            stroke="#fff"
            strokeWidth="2"
          />
          <text
            x="20"
            y="-10"
            textAnchor="middle"
            fontSize="14"
            fill="#fff"
            fontWeight="bold"
          >
            ×
          </text>
        </g>
      )}
    </g>
  );
};

export default WorkflowConnector;
