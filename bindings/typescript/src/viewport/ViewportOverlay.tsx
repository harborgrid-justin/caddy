/**
 * Viewport Overlay - Measurement Overlays and Annotations
 *
 * Enterprise-grade overlay system for CADDY v0.2.5
 * Provides measurements, annotations, dimensions, and visual indicators.
 *
 * @module ViewportOverlay
 */

import React, { useMemo, useCallback, useState, useEffect } from 'react';
import { CameraState, RenderStatistics } from './ViewportManager';

/**
 * Overlay element types
 */
export enum OverlayType {
  Text = 'text',
  Measurement = 'measurement',
  Dimension = 'dimension',
  Annotation = 'annotation',
  Cursor = 'cursor',
  Icon = 'icon',
  Line = 'line',
  Rectangle = 'rectangle',
  Circle = 'circle',
}

/**
 * Position anchor for overlay elements
 */
export enum AnchorPosition {
  TopLeft = 'top-left',
  TopCenter = 'top-center',
  TopRight = 'top-right',
  MiddleLeft = 'middle-left',
  Center = 'center',
  MiddleRight = 'middle-right',
  BottomLeft = 'bottom-left',
  BottomCenter = 'bottom-center',
  BottomRight = 'bottom-right',
}

/**
 * Base overlay element
 */
export interface OverlayElement {
  id: string;
  type: OverlayType;
  visible: boolean;
  position: [number, number]; // Screen coordinates
  anchor?: AnchorPosition;
  opacity?: number;
  zIndex?: number;
}

/**
 * Text overlay element
 */
export interface TextOverlay extends OverlayElement {
  type: OverlayType.Text;
  text: string;
  fontSize?: number;
  fontFamily?: string;
  color?: string;
  backgroundColor?: string;
  padding?: number;
  borderRadius?: number;
}

/**
 * Measurement overlay element
 */
export interface MeasurementOverlay extends OverlayElement {
  type: OverlayType.Measurement;
  startPoint: [number, number, number]; // World coordinates
  endPoint: [number, number, number]; // World coordinates
  value: number;
  unit: string;
  label?: string;
  color?: string;
  lineWidth?: number;
  showEndpoints?: boolean;
}

/**
 * Dimension overlay element
 */
export interface DimensionOverlay extends OverlayElement {
  type: OverlayType.Dimension;
  points: Array<[number, number, number]>; // World coordinates
  value: number;
  unit: string;
  label?: string;
  color?: string;
  offset?: number; // Offset from geometry
}

/**
 * Annotation overlay element
 */
export interface AnnotationOverlay extends OverlayElement {
  type: OverlayType.Annotation;
  worldPosition: [number, number, number];
  title: string;
  description?: string;
  icon?: string;
  color?: string;
  size?: number;
}

/**
 * Cursor overlay element
 */
export interface CursorOverlay extends OverlayElement {
  type: OverlayType.Cursor;
  cursorType: 'crosshair' | 'circle' | 'square' | 'custom';
  size?: number;
  color?: string;
}

/**
 * Line overlay element
 */
export interface LineOverlay extends OverlayElement {
  type: OverlayType.Line;
  endPosition: [number, number];
  color?: string;
  width?: number;
  style?: 'solid' | 'dashed' | 'dotted';
}

/**
 * Rectangle overlay element
 */
export interface RectangleOverlay extends OverlayElement {
  type: OverlayType.Rectangle;
  width: number;
  height: number;
  fillColor?: string;
  strokeColor?: string;
  strokeWidth?: number;
}

/**
 * Circle overlay element
 */
export interface CircleOverlay extends OverlayElement {
  type: OverlayType.Circle;
  radius: number;
  fillColor?: string;
  strokeColor?: string;
  strokeWidth?: number;
}

/**
 * Union type for all overlay elements
 */
export type AnyOverlay =
  | TextOverlay
  | MeasurementOverlay
  | DimensionOverlay
  | AnnotationOverlay
  | CursorOverlay
  | LineOverlay
  | RectangleOverlay
  | CircleOverlay;

/**
 * Viewport overlay props
 */
export interface ViewportOverlayProps {
  /** Camera state for world-to-screen conversion */
  camera: CameraState;

  /** Viewport width */
  width: number;

  /** Viewport height */
  height: number;

  /** Overlay elements to render */
  overlays?: AnyOverlay[];

  /** Show FPS counter */
  showFPS?: boolean;

  /** Show statistics panel */
  showStats?: boolean;

  /** Render statistics */
  stats?: RenderStatistics;

  /** Show coordinate display */
  showCoordinates?: boolean;

  /** Current cursor world position */
  cursorPosition?: [number, number, number];

  /** Custom CSS class */
  className?: string;

  /** Custom styles */
  style?: React.CSSProperties;
}

/**
 * Render text overlay
 */
const RenderTextOverlay: React.FC<{ overlay: TextOverlay }> = ({ overlay }) => {
  const style: React.CSSProperties = {
    position: 'absolute',
    left: overlay.position[0],
    top: overlay.position[1],
    fontSize: overlay.fontSize || 14,
    fontFamily: overlay.fontFamily || 'sans-serif',
    color: overlay.color || '#ffffff',
    backgroundColor: overlay.backgroundColor || 'rgba(0, 0, 0, 0.7)',
    padding: overlay.padding || 8,
    borderRadius: overlay.borderRadius || 4,
    opacity: overlay.opacity || 1,
    zIndex: overlay.zIndex || 100,
    pointerEvents: 'none',
    whiteSpace: 'nowrap',
    userSelect: 'none',
  };

  return <div style={style}>{overlay.text}</div>;
};

/**
 * Render measurement overlay
 */
const RenderMeasurementOverlay: React.FC<{
  overlay: MeasurementOverlay;
  camera: CameraState;
  viewportSize: [number, number];
}> = ({ overlay, camera, viewportSize }) => {
  // Convert world coordinates to screen coordinates
  const startScreen = worldToScreen(overlay.startPoint, camera, viewportSize);
  const endScreen = worldToScreen(overlay.endPoint, camera, viewportSize);

  if (!startScreen || !endScreen) {
    return null;
  }

  const color = overlay.color || '#00ff00';
  const lineWidth = overlay.lineWidth || 2;
  const showEndpoints = overlay.showEndpoints !== false;

  const midX = (startScreen[0] + endScreen[0]) / 2;
  const midY = (startScreen[1] + endScreen[1]) / 2;

  return (
    <>
      <svg
        style={{
          position: 'absolute',
          top: 0,
          left: 0,
          width: '100%',
          height: '100%',
          pointerEvents: 'none',
          zIndex: overlay.zIndex || 100,
        }}
      >
        <line
          x1={startScreen[0]}
          y1={startScreen[1]}
          x2={endScreen[0]}
          y2={endScreen[1]}
          stroke={color}
          strokeWidth={lineWidth}
        />
        {showEndpoints && (
          <>
            <circle cx={startScreen[0]} cy={startScreen[1]} r={4} fill={color} />
            <circle cx={endScreen[0]} cy={endScreen[1]} r={4} fill={color} />
          </>
        )}
      </svg>
      <div
        style={{
          position: 'absolute',
          left: midX,
          top: midY,
          transform: 'translate(-50%, -50%)',
          backgroundColor: 'rgba(0, 0, 0, 0.8)',
          color: '#ffffff',
          padding: '4px 8px',
          borderRadius: 4,
          fontSize: 12,
          fontFamily: 'monospace',
          pointerEvents: 'none',
          zIndex: overlay.zIndex || 101,
        }}
      >
        {overlay.label && <div>{overlay.label}</div>}
        <div>
          {overlay.value.toFixed(2)} {overlay.unit}
        </div>
      </div>
    </>
  );
};

/**
 * Render annotation overlay
 */
const RenderAnnotationOverlay: React.FC<{
  overlay: AnnotationOverlay;
  camera: CameraState;
  viewportSize: [number, number];
}> = ({ overlay, camera, viewportSize }) => {
  const screenPos = worldToScreen(overlay.worldPosition, camera, viewportSize);

  if (!screenPos) {
    return null;
  }

  const color = overlay.color || '#ffaa00';
  const size = overlay.size || 24;

  return (
    <div
      style={{
        position: 'absolute',
        left: screenPos[0],
        top: screenPos[1],
        transform: 'translate(-50%, -100%)',
        zIndex: overlay.zIndex || 100,
        pointerEvents: 'auto',
        cursor: 'pointer',
      }}
    >
      <div
        style={{
          width: size,
          height: size,
          borderRadius: '50%',
          backgroundColor: color,
          border: '2px solid white',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          color: '#ffffff',
          fontSize: size * 0.6,
          fontWeight: 'bold',
          marginBottom: 4,
        }}
      >
        {overlay.icon || 'i'}
      </div>
      <div
        style={{
          backgroundColor: 'rgba(0, 0, 0, 0.9)',
          color: '#ffffff',
          padding: '8px 12px',
          borderRadius: 4,
          fontSize: 12,
          fontFamily: 'sans-serif',
          minWidth: 150,
          maxWidth: 300,
        }}
      >
        <div style={{ fontWeight: 'bold', marginBottom: 4 }}>{overlay.title}</div>
        {overlay.description && <div style={{ fontSize: 11 }}>{overlay.description}</div>}
      </div>
    </div>
  );
};

/**
 * Convert world coordinates to screen coordinates (simplified)
 */
function worldToScreen(
  worldPos: [number, number, number],
  camera: CameraState,
  viewportSize: [number, number]
): [number, number] | null {
  const [width, height] = viewportSize;
  const [wx, wy, wz] = worldPos;
  const [cx, cy, cz] = camera.position;
  const zoom = camera.zoom || 1.0;

  // Simple orthographic projection
  const scale = 50.0 / zoom;
  const centerX = width / 2;
  const centerY = height / 2;

  const screenX = centerX + (wx - cx) * scale;
  const screenY = centerY + (wz - cz) * scale;

  // Check if in viewport
  if (screenX < 0 || screenX > width || screenY < 0 || screenY > height) {
    return null;
  }

  return [screenX, screenY];
}

/**
 * Statistics Panel Component
 */
const StatsPanel: React.FC<{ stats: RenderStatistics; position?: AnchorPosition }> = ({
  stats,
  position = AnchorPosition.TopLeft,
}) => {
  const positionStyle = useMemo<React.CSSProperties>(() => {
    const base: React.CSSProperties = {
      position: 'absolute',
      backgroundColor: 'rgba(0, 0, 0, 0.8)',
      color: '#00ff00',
      padding: 12,
      borderRadius: 4,
      fontSize: 11,
      fontFamily: 'monospace',
      zIndex: 1000,
      pointerEvents: 'none',
      userSelect: 'none',
    };

    switch (position) {
      case AnchorPosition.TopLeft:
        return { ...base, top: 10, left: 10 };
      case AnchorPosition.TopRight:
        return { ...base, top: 10, right: 10 };
      case AnchorPosition.BottomLeft:
        return { ...base, bottom: 10, left: 10 };
      case AnchorPosition.BottomRight:
        return { ...base, bottom: 10, right: 10 };
      default:
        return { ...base, top: 10, left: 10 };
    }
  }, [position]);

  return (
    <div style={positionStyle}>
      <div style={{ marginBottom: 8, fontWeight: 'bold', color: '#ffffff' }}>
        Performance Stats
      </div>
      <div>FPS: {stats.fps.toFixed(1)}</div>
      <div>Frame Time: {stats.frameTimeMs.toFixed(2)}ms</div>
      <div>Draw Calls: {stats.drawCalls}</div>
      <div>Vertices: {stats.verticesRendered.toLocaleString()}</div>
      <div>Triangles: {stats.trianglesRendered.toLocaleString()}</div>
      <div>Culled: {stats.objectsCulled}</div>
      <div>Rendered: {stats.objectsRendered}</div>
      <div style={{ marginTop: 8 }}>
        GPU Memory: {(stats.gpuMemoryBytes / 1024 / 1024).toFixed(2)}MB
      </div>
      <div>CPU Time: {stats.cpuTimeMs.toFixed(2)}ms</div>
      <div>GPU Time: {stats.gpuTimeMs.toFixed(2)}ms</div>
    </div>
  );
};

/**
 * FPS Counter Component
 */
const FPSCounter: React.FC<{ fps: number; position?: AnchorPosition }> = ({
  fps,
  position = AnchorPosition.TopRight,
}) => {
  const positionStyle = useMemo<React.CSSProperties>(() => {
    const base: React.CSSProperties = {
      position: 'absolute',
      backgroundColor: 'rgba(0, 0, 0, 0.8)',
      color: fps > 55 ? '#00ff00' : fps > 30 ? '#ffaa00' : '#ff0000',
      padding: '6px 12px',
      borderRadius: 4,
      fontSize: 14,
      fontFamily: 'monospace',
      fontWeight: 'bold',
      zIndex: 1000,
      pointerEvents: 'none',
      userSelect: 'none',
    };

    switch (position) {
      case AnchorPosition.TopLeft:
        return { ...base, top: 10, left: 10 };
      case AnchorPosition.TopRight:
        return { ...base, top: 10, right: 10 };
      case AnchorPosition.BottomLeft:
        return { ...base, bottom: 10, left: 10 };
      case AnchorPosition.BottomRight:
        return { ...base, bottom: 10, right: 10 };
      default:
        return { ...base, top: 10, right: 10 };
    }
  }, [fps, position]);

  return <div style={positionStyle}>{fps.toFixed(1)} FPS</div>;
};

/**
 * Coordinate Display Component
 */
const CoordinateDisplay: React.FC<{
  position: [number, number, number] | undefined;
  anchorPosition?: AnchorPosition;
}> = ({ position, anchorPosition = AnchorPosition.BottomLeft }) => {
  if (!position) {
    return null;
  }

  const [x, y, z] = position;

  const positionStyle = useMemo<React.CSSProperties>(() => {
    const base: React.CSSProperties = {
      position: 'absolute',
      backgroundColor: 'rgba(0, 0, 0, 0.8)',
      color: '#ffffff',
      padding: '6px 12px',
      borderRadius: 4,
      fontSize: 12,
      fontFamily: 'monospace',
      zIndex: 1000,
      pointerEvents: 'none',
      userSelect: 'none',
    };

    switch (anchorPosition) {
      case AnchorPosition.TopLeft:
        return { ...base, top: 10, left: 10 };
      case AnchorPosition.TopRight:
        return { ...base, top: 10, right: 10 };
      case AnchorPosition.BottomLeft:
        return { ...base, bottom: 10, left: 10 };
      case AnchorPosition.BottomRight:
        return { ...base, bottom: 10, right: 10 };
      default:
        return { ...base, bottom: 10, left: 10 };
    }
  }, [anchorPosition]);

  return (
    <div style={positionStyle}>
      X: {x.toFixed(2)} Y: {y.toFixed(2)} Z: {z.toFixed(2)}
    </div>
  );
};

/**
 * Viewport Overlay Component
 *
 * Renders measurements, annotations, and visual indicators over the viewport.
 */
export const ViewportOverlay: React.FC<ViewportOverlayProps> = React.memo(({
  camera,
  width,
  height,
  overlays = [],
  showFPS = false,
  showStats = false,
  stats,
  showCoordinates = false,
  cursorPosition,
  className,
  style,
}) => {
  const containerStyle = useMemo<React.CSSProperties>(
    () => ({
      position: 'absolute',
      top: 0,
      left: 0,
      width: '100%',
      height: '100%',
      pointerEvents: 'none',
      ...style,
    }),
    [style]
  );

  const viewportSize: [number, number] = [width, height];

  return (
    <div style={containerStyle} className={className}>
      {/* Render overlay elements */}
      {overlays
        .filter((overlay) => overlay.visible)
        .sort((a, b) => (a.zIndex || 100) - (b.zIndex || 100))
        .map((overlay) => {
          switch (overlay.type) {
            case OverlayType.Text:
              return <RenderTextOverlay key={overlay.id} overlay={overlay as TextOverlay} />;

            case OverlayType.Measurement:
              return (
                <RenderMeasurementOverlay
                  key={overlay.id}
                  overlay={overlay as MeasurementOverlay}
                  camera={camera}
                  viewportSize={viewportSize}
                />
              );

            case OverlayType.Annotation:
              return (
                <RenderAnnotationOverlay
                  key={overlay.id}
                  overlay={overlay as AnnotationOverlay}
                  camera={camera}
                  viewportSize={viewportSize}
                />
              );

            default:
              return null;
          }
        })}

      {/* FPS Counter */}
      {showFPS && stats && <FPSCounter fps={stats.fps} />}

      {/* Statistics Panel */}
      {showStats && stats && <StatsPanel stats={stats} />}

      {/* Coordinate Display */}
      {showCoordinates && <CoordinateDisplay position={cursorPosition} />}
    </div>
  );
});

ViewportOverlay.displayName = 'ViewportOverlay';

export default ViewportOverlay;
