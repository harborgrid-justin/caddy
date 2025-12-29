/**
 * Cursor Overlay - Real-time Collaborator Cursors
 *
 * Displays cursors and selections of other users in real-time,
 * with smooth animations and user identification labels.
 */

import React, { useEffect, useState, useRef, useCallback } from 'react';
import { usePresence, useThrottledCursor } from './useCollaboration';
import type { UserPresence, CursorPosition } from './useCollaboration';

/**
 * Cursor data with animation state
 */
interface AnimatedCursor extends UserPresence {
  displayPosition: CursorPosition;
  velocity: { x: number; y: number };
}

/**
 * Cursor Overlay Props
 */
export interface CursorOverlayProps {
  className?: string;
  style?: React.CSSProperties;
  viewportId?: string;
  showLabels?: boolean;
  showSelections?: boolean;
  animationDuration?: number;
  cursorSize?: number;
  labelOffset?: number;
}

/**
 * Individual Cursor Component
 */
function CollaboratorCursor({
  user,
  position,
  showLabel,
  size,
  labelOffset,
}: {
  user: UserPresence;
  position: CursorPosition;
  showLabel: boolean;
  size: number;
  labelOffset: number;
}) {
  const [isHovered, setIsHovered] = useState(false);

  const cursorStyle: React.CSSProperties = {
    position: 'absolute',
    left: position.x,
    top: position.y,
    pointerEvents: 'none',
    transform: 'translate(-2px, -2px)',
    transition: 'transform 0.15s ease-out',
    zIndex: 1000,
  };

  const labelStyle: React.CSSProperties = {
    position: 'absolute',
    left: size + labelOffset,
    top: size + labelOffset,
    backgroundColor: user.user.color,
    color: '#fff',
    padding: '4px 8px',
    borderRadius: '4px',
    fontSize: '12px',
    fontWeight: '500',
    whiteSpace: 'nowrap',
    boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
    transform: isHovered ? 'scale(1.05)' : 'scale(1)',
    transition: 'transform 0.2s ease',
  };

  // SVG cursor icon
  const CursorSVG = () => (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      style={{ filter: 'drop-shadow(0 1px 2px rgba(0,0,0,0.3))' }}
    >
      <path
        d="M5.5 3.5L16.5 14.5L11 15L8.5 20.5L5.5 3.5Z"
        fill={user.user.color}
        stroke="#fff"
        strokeWidth="1.5"
        strokeLinejoin="round"
      />
    </svg>
  );

  return (
    <div
      style={cursorStyle}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      <CursorSVG />
      {(showLabel || isHovered) && <div style={labelStyle}>{user.user.name}</div>}
    </div>
  );
}

/**
 * Selection Highlight Component
 */
function SelectionHighlight({
  entityIds,
  color,
  opacity = 0.2,
}: {
  entityIds: string[];
  color: string;
  opacity?: number;
}) {
  // In a real implementation, this would query the actual positions
  // of the selected entities and draw highlights around them
  // For now, we'll return null as this is viewport-specific

  return null; // Placeholder - requires integration with CAD rendering engine
}

/**
 * Main Cursor Overlay Component
 */
export function CursorOverlay({
  className = '',
  style = {},
  viewportId,
  showLabels = true,
  showSelections = true,
  animationDuration = 150,
  cursorSize = 24,
  labelOffset = 4,
}: CursorOverlayProps) {
  const { otherUsers } = usePresence();
  const [animatedCursors, setAnimatedCursors] = useState<AnimatedCursor[]>([]);
  const animationFrameRef = useRef<number | null>(null);
  const lastUpdateRef = useRef<number>(Date.now());

  /**
   * Interpolate cursor positions for smooth animation
   */
  const interpolatePosition = useCallback(
    (current: CursorPosition, target: CursorPosition, deltaTime: number): CursorPosition => {
      const factor = Math.min(1, (deltaTime * 60) / animationDuration);

      return {
        x: current.x + (target.x - current.x) * factor,
        y: current.y + (target.y - current.y) * factor,
        z: target.z,
        viewportId: target.viewportId,
      };
    },
    [animationDuration]
  );

  /**
   * Animation loop for smooth cursor movement
   */
  const animate = useCallback(() => {
    const now = Date.now();
    const deltaTime = now - lastUpdateRef.current;
    lastUpdateRef.current = now;

    setAnimatedCursors(prev => {
      return prev.map(cursor => {
        const targetUser = otherUsers.find(u => u.userId === cursor.userId);

        if (!targetUser || !targetUser.cursor) {
          return cursor;
        }

        // Filter by viewport if specified
        if (viewportId && targetUser.cursor.viewportId !== viewportId) {
          return cursor;
        }

        const newPosition = interpolatePosition(
          cursor.displayPosition,
          targetUser.cursor,
          deltaTime
        );

        return {
          ...cursor,
          displayPosition: newPosition,
          velocity: {
            x: (newPosition.x - cursor.displayPosition.x) / deltaTime,
            y: (newPosition.y - cursor.displayPosition.y) / deltaTime,
          },
        };
      });
    });

    animationFrameRef.current = requestAnimationFrame(animate);
  }, [otherUsers, viewportId, interpolatePosition]);

  /**
   * Update cursor list when users change
   */
  useEffect(() => {
    setAnimatedCursors(prev => {
      const newCursors: AnimatedCursor[] = [];
      const existingMap = new Map(prev.map(c => [c.userId, c]));

      otherUsers.forEach(user => {
        if (!user.cursor) return;

        // Filter by viewport if specified
        if (viewportId && user.cursor.viewportId !== viewportId) {
          return;
        }

        const existing = existingMap.get(user.userId);

        if (existing) {
          newCursors.push({
            ...user,
            displayPosition: existing.displayPosition,
            velocity: existing.velocity,
          });
        } else {
          // New cursor - initialize at target position
          newCursors.push({
            ...user,
            displayPosition: user.cursor,
            velocity: { x: 0, y: 0 },
          });
        }
      });

      return newCursors;
    });
  }, [otherUsers, viewportId]);

  /**
   * Start/stop animation loop
   */
  useEffect(() => {
    if (animatedCursors.length > 0) {
      animationFrameRef.current = requestAnimationFrame(animate);
    }

    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, [animate, animatedCursors.length]);

  return (
    <div
      className={className}
      style={{
        position: 'absolute',
        top: 0,
        left: 0,
        width: '100%',
        height: '100%',
        pointerEvents: 'none',
        zIndex: 999,
        ...style,
      }}
    >
      {/* Render cursors */}
      {animatedCursors.map(cursor => (
        <CollaboratorCursor
          key={cursor.userId}
          user={cursor}
          position={cursor.displayPosition}
          showLabel={showLabels}
          size={cursorSize}
          labelOffset={labelOffset}
        />
      ))}

      {/* Render selections */}
      {showSelections &&
        otherUsers.map(user => {
          if (!user.selection || user.selection.length === 0) return null;

          return (
            <SelectionHighlight
              key={`selection-${user.userId}`}
              entityIds={user.selection}
              color={user.user.color}
            />
          );
        })}
    </div>
  );
}

/**
 * Hook for tracking local cursor and sending updates
 */
export function useLocalCursor(containerRef: React.RefObject<HTMLElement>, throttleDelay = 50) {
  const throttledSetCursor = useThrottledCursor(throttleDelay);
  const [isInViewport, setIsInViewport] = useState(false);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const handleMouseMove = (e: MouseEvent) => {
      const rect = container.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;

      throttledSetCursor({ x, y });
    };

    const handleMouseEnter = () => {
      setIsInViewport(true);
    };

    const handleMouseLeave = () => {
      setIsInViewport(false);
      throttledSetCursor(null);
    };

    container.addEventListener('mousemove', handleMouseMove);
    container.addEventListener('mouseenter', handleMouseEnter);
    container.addEventListener('mouseleave', handleMouseLeave);

    return () => {
      container.removeEventListener('mousemove', handleMouseMove);
      container.removeEventListener('mouseenter', handleMouseEnter);
      container.removeEventListener('mouseleave', handleMouseLeave);
    };
  }, [containerRef, throttledSetCursor]);

  return { isInViewport };
}

/**
 * Mini Cursor Map Component - Shows overview of all cursor positions
 */
export function CursorMiniMap({
  width = 200,
  height = 150,
  backgroundColor = '#f8fafc',
}: {
  width?: number;
  height?: number;
  backgroundColor?: string;
}) {
  const { otherUsers } = usePresence();

  return (
    <div
      style={{
        width,
        height,
        backgroundColor,
        border: '1px solid #e2e8f0',
        borderRadius: '6px',
        position: 'relative',
        overflow: 'hidden',
      }}
    >
      <div
        style={{
          padding: '4px 8px',
          fontSize: '11px',
          fontWeight: '600',
          color: '#64748b',
          borderBottom: '1px solid #e2e8f0',
        }}
      >
        Cursor Map
      </div>
      <div style={{ position: 'relative', width: '100%', height: 'calc(100% - 28px)' }}>
        {otherUsers.map(user => {
          if (!user.cursor) return null;

          // Normalize cursor position to mini map (simplified)
          const x = (user.cursor.x % width) || 0;
          const y = (user.cursor.y % height) || 0;

          return (
            <div
              key={user.userId}
              style={{
                position: 'absolute',
                left: x,
                top: y,
                width: '8px',
                height: '8px',
                borderRadius: '50%',
                backgroundColor: user.user.color,
                border: '1px solid #fff',
                transform: 'translate(-50%, -50%)',
                boxShadow: '0 1px 2px rgba(0,0,0,0.2)',
              }}
              title={user.user.name}
            />
          );
        })}
      </div>
    </div>
  );
}

export default CursorOverlay;
