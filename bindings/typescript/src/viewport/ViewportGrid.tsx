/**
 * Viewport Grid - Infinite Grid with Snapping
 *
 * Enterprise-grade infinite grid system for CADDY v0.2.5
 * Provides visual reference grid with automatic snapping and dynamic LOD.
 *
 * @module ViewportGrid
 */

import React, { useMemo, useCallback, useEffect, useRef } from 'react';
import { CameraState, CameraMode } from './ViewportManager';

/**
 * Grid configuration
 */
export interface GridConfig {
  /** Primary grid size (spacing between major lines) */
  size: number;

  /** Number of subdivisions between major lines */
  subdivisions: number;

  /** Show grid */
  visible: boolean;

  /** Primary grid line color (RGBA) */
  primaryColor: [number, number, number, number];

  /** Secondary grid line color (RGBA) */
  secondaryColor: [number, number, number, number];

  /** Primary line width */
  primaryWidth: number;

  /** Secondary line width */
  secondaryWidth: number;

  /** Fade grid at distance */
  fadeAtDistance: boolean;

  /** Maximum fade distance */
  fadeDistance: number;

  /** Show axis lines */
  showAxis: boolean;

  /** X-axis color (RGBA) */
  xAxisColor: [number, number, number, number];

  /** Y-axis color (RGBA) */
  yAxisColor: [number, number, number, number];

  /** Z-axis color (RGBA) */
  zAxisColor: [number, number, number, number];

  /** Axis line width */
  axisWidth: number;

  /** Enable dynamic LOD for grid density */
  dynamicLOD: boolean;

  /** Grid plane (XY, XZ, or YZ) */
  plane: 'xy' | 'xz' | 'yz';
}

/**
 * Default grid configuration
 */
export const DEFAULT_GRID_CONFIG: GridConfig = {
  size: 1.0,
  subdivisions: 10,
  visible: true,
  primaryColor: [0.5, 0.5, 0.5, 0.5],
  secondaryColor: [0.3, 0.3, 0.3, 0.3],
  primaryWidth: 2.0,
  secondaryWidth: 1.0,
  fadeAtDistance: true,
  fadeDistance: 100.0,
  showAxis: true,
  xAxisColor: [1.0, 0.3, 0.3, 0.8],
  yAxisColor: [0.3, 1.0, 0.3, 0.8],
  zAxisColor: [0.3, 0.3, 1.0, 0.8],
  axisWidth: 3.0,
  dynamicLOD: true,
  plane: 'xz',
};

/**
 * Snapping configuration
 */
export interface SnapConfig {
  /** Enable snapping */
  enabled: boolean;

  /** Snap to grid */
  snapToGrid: boolean;

  /** Grid snap size */
  gridSnapSize: number;

  /** Snap to angles */
  snapToAngle: boolean;

  /** Angle snap increment (degrees) */
  angleSnapIncrement: number;

  /** Snap threshold (pixels) */
  snapThreshold: number;

  /** Show snap indicators */
  showSnapIndicators: boolean;

  /** Snap indicator color */
  snapIndicatorColor: [number, number, number, number];

  /** Snap indicator size */
  snapIndicatorSize: number;
}

/**
 * Default snapping configuration
 */
export const DEFAULT_SNAP_CONFIG: SnapConfig = {
  enabled: true,
  snapToGrid: true,
  gridSnapSize: 1.0,
  snapToAngle: true,
  angleSnapIncrement: 15.0,
  snapThreshold: 10.0,
  showSnapIndicators: true,
  snapIndicatorColor: [1.0, 1.0, 0.0, 0.8],
  snapIndicatorSize: 8.0,
};

/**
 * Viewport grid props
 */
export interface ViewportGridProps {
  /** Camera state */
  camera: CameraState;

  /** Canvas context */
  context: CanvasRenderingContext2D | null;

  /** Canvas width */
  width: number;

  /** Canvas height */
  height: number;

  /** Grid configuration */
  gridConfig?: Partial<GridConfig>;

  /** Snap configuration */
  snapConfig?: Partial<SnapConfig>;
}

/**
 * Calculate grid lines for rendering
 */
function calculateGridLines(
  camera: CameraState,
  config: GridConfig,
  viewportSize: [number, number]
): {
  primaryLines: Array<{ start: [number, number]; end: [number, number] }>;
  secondaryLines: Array<{ start: [number, number]; end: [number, number] }>;
  axisLines: {
    x: { start: [number, number]; end: [number, number] } | null;
    y: { start: [number, number]; end: [number, number] } | null;
    z: { start: [number, number]; end: [number, number] } | null;
  };
} {
  const [width, height] = viewportSize;
  const primaryLines: Array<{ start: [number, number]; end: [number, number] }> = [];
  const secondaryLines: Array<{ start: [number, number]; end: [number, number] }> = [];

  // Calculate visible area based on camera
  const [px, py, pz] = camera.position;
  const zoom = camera.zoom || 1.0;

  // Simple orthographic projection for grid
  const scale = 50.0 / zoom; // Pixels per unit
  const centerX = width / 2;
  const centerY = height / 2;

  // Calculate visible range
  const rangeX = (width / scale) * 1.5;
  const rangeY = (height / scale) * 1.5;

  const startX = Math.floor((px - rangeX / 2) / config.size) * config.size;
  const endX = Math.ceil((px + rangeX / 2) / config.size) * config.size;
  const startY = Math.floor((pz - rangeY / 2) / config.size) * config.size;
  const endY = Math.ceil((pz + rangeY / 2) / config.size) * config.size;

  // Generate grid lines
  const secondarySize = config.size / config.subdivisions;

  // Vertical lines
  for (let x = startX; x <= endX; x += secondarySize) {
    const isPrimary = Math.abs(x % config.size) < 0.0001;
    const screenX = centerX + (x - px) * scale;

    const line = {
      start: [screenX, 0] as [number, number],
      end: [screenX, height] as [number, number],
    };

    if (isPrimary) {
      primaryLines.push(line);
    } else {
      secondaryLines.push(line);
    }
  }

  // Horizontal lines
  for (let y = startY; y <= endY; y += secondarySize) {
    const isPrimary = Math.abs(y % config.size) < 0.0001;
    const screenY = centerY + (y - pz) * scale;

    const line = {
      start: [0, screenY] as [number, number],
      end: [width, screenY] as [number, number],
    };

    if (isPrimary) {
      primaryLines.push(line);
    } else {
      secondaryLines.push(line);
    }
  }

  // Axis lines
  const axisLines = {
    x: null as { start: [number, number]; end: [number, number] } | null,
    y: null as { start: [number, number]; end: [number, number] } | null,
    z: null as { start: [number, number]; end: [number, number] } | null,
  };

  if (config.showAxis) {
    // X-axis (horizontal at z=0)
    const zeroY = centerY + (0 - pz) * scale;
    if (zeroY >= 0 && zeroY <= height) {
      axisLines.x = {
        start: [0, zeroY],
        end: [width, zeroY],
      };
    }

    // Z-axis (vertical at x=0)
    const zeroX = centerX + (0 - px) * scale;
    if (zeroX >= 0 && zeroX <= width) {
      axisLines.z = {
        start: [zeroX, 0],
        end: [zeroX, height],
      };
    }
  }

  return { primaryLines, secondaryLines, axisLines };
}

/**
 * Snap point to grid
 */
export function snapToGrid(
  point: [number, number, number],
  snapConfig: SnapConfig
): [number, number, number] {
  if (!snapConfig.enabled || !snapConfig.snapToGrid) {
    return point;
  }

  const size = snapConfig.gridSnapSize;
  return [
    Math.round(point[0] / size) * size,
    Math.round(point[1] / size) * size,
    Math.round(point[2] / size) * size,
  ];
}

/**
 * Snap angle to increment
 */
export function snapAngle(angle: number, snapConfig: SnapConfig): number {
  if (!snapConfig.enabled || !snapConfig.snapToAngle) {
    return angle;
  }

  const increment = (snapConfig.angleSnapIncrement * Math.PI) / 180.0;
  return Math.round(angle / increment) * increment;
}

/**
 * Check if point is near snap point
 */
export function isNearSnapPoint(
  screenPoint: [number, number],
  snapPoint: [number, number],
  threshold: number
): boolean {
  const dx = screenPoint[0] - snapPoint[0];
  const dy = screenPoint[1] - snapPoint[1];
  const distance = Math.sqrt(dx * dx + dy * dy);
  return distance <= threshold;
}

/**
 * Viewport Grid Component
 *
 * Renders an infinite grid with snapping functionality.
 */
export const ViewportGrid: React.FC<ViewportGridProps> = React.memo(({
  camera,
  context,
  width,
  height,
  gridConfig: gridConfigProp,
  snapConfig: snapConfigProp,
}) => {
  const gridConfig = useMemo(
    () => ({ ...DEFAULT_GRID_CONFIG, ...gridConfigProp }),
    [gridConfigProp]
  );

  const snapConfig = useMemo(
    () => ({ ...DEFAULT_SNAP_CONFIG, ...snapConfigProp }),
    [snapConfigProp]
  );

  const gridLinesRef = useRef<ReturnType<typeof calculateGridLines> | null>(null);

  /**
   * Calculate grid lines
   */
  const updateGridLines = useCallback(() => {
    if (!gridConfig.visible) {
      gridLinesRef.current = null;
      return;
    }

    gridLinesRef.current = calculateGridLines(camera, gridConfig, [width, height]);
  }, [camera, gridConfig, width, height]);

  /**
   * Render grid to canvas
   */
  const renderGrid = useCallback(() => {
    if (!context || !gridConfig.visible || !gridLinesRef.current) {
      return;
    }

    const { primaryLines, secondaryLines, axisLines } = gridLinesRef.current;

    context.save();

    // Draw secondary lines
    context.strokeStyle = `rgba(${gridConfig.secondaryColor[0] * 255}, ${gridConfig.secondaryColor[1] * 255}, ${gridConfig.secondaryColor[2] * 255}, ${gridConfig.secondaryColor[3]})`;
    context.lineWidth = gridConfig.secondaryWidth;
    context.beginPath();
    for (const line of secondaryLines) {
      context.moveTo(line.start[0], line.start[1]);
      context.lineTo(line.end[0], line.end[1]);
    }
    context.stroke();

    // Draw primary lines
    context.strokeStyle = `rgba(${gridConfig.primaryColor[0] * 255}, ${gridConfig.primaryColor[1] * 255}, ${gridConfig.primaryColor[2] * 255}, ${gridConfig.primaryColor[3]})`;
    context.lineWidth = gridConfig.primaryWidth;
    context.beginPath();
    for (const line of primaryLines) {
      context.moveTo(line.start[0], line.start[1]);
      context.lineTo(line.end[0], line.end[1]);
    }
    context.stroke();

    // Draw axis lines
    if (gridConfig.showAxis) {
      // X-axis
      if (axisLines.x) {
        context.strokeStyle = `rgba(${gridConfig.xAxisColor[0] * 255}, ${gridConfig.xAxisColor[1] * 255}, ${gridConfig.xAxisColor[2] * 255}, ${gridConfig.xAxisColor[3]})`;
        context.lineWidth = gridConfig.axisWidth;
        context.beginPath();
        context.moveTo(axisLines.x.start[0], axisLines.x.start[1]);
        context.lineTo(axisLines.x.end[0], axisLines.x.end[1]);
        context.stroke();
      }

      // Z-axis (rendered as Y in 2D)
      if (axisLines.z) {
        context.strokeStyle = `rgba(${gridConfig.zAxisColor[0] * 255}, ${gridConfig.zAxisColor[1] * 255}, ${gridConfig.zAxisColor[2] * 255}, ${gridConfig.zAxisColor[3]})`;
        context.lineWidth = gridConfig.axisWidth;
        context.beginPath();
        context.moveTo(axisLines.z.start[0], axisLines.z.start[1]);
        context.lineTo(axisLines.z.end[0], axisLines.z.end[1]);
        context.stroke();
      }
    }

    context.restore();
  }, [context, gridConfig]);

  /**
   * Update grid when camera or config changes
   */
  useEffect(() => {
    updateGridLines();
  }, [updateGridLines]);

  /**
   * Render grid when context is available
   */
  useEffect(() => {
    if (context && gridConfig.visible) {
      renderGrid();
    }
  }, [context, gridConfig.visible, renderGrid]);

  return null;
});

ViewportGrid.displayName = 'ViewportGrid';

/**
 * Grid Helper Functions
 */
export const GridHelpers = {
  /**
   * Convert world coordinates to screen coordinates
   */
  worldToScreen(
    worldPos: [number, number, number],
    camera: CameraState,
    viewportSize: [number, number]
  ): [number, number] {
    const [width, height] = viewportSize;
    const [wx, wy, wz] = worldPos;
    const [cx, cy, cz] = camera.position;
    const zoom = camera.zoom || 1.0;

    const scale = 50.0 / zoom;
    const centerX = width / 2;
    const centerY = height / 2;

    const screenX = centerX + (wx - cx) * scale;
    const screenY = centerY + (wz - cz) * scale;

    return [screenX, screenY];
  },

  /**
   * Convert screen coordinates to world coordinates
   */
  screenToWorld(
    screenPos: [number, number],
    camera: CameraState,
    viewportSize: [number, number]
  ): [number, number, number] {
    const [width, height] = viewportSize;
    const [sx, sy] = screenPos;
    const [cx, cy, cz] = camera.position;
    const zoom = camera.zoom || 1.0;

    const scale = 50.0 / zoom;
    const centerX = width / 2;
    const centerY = height / 2;

    const worldX = cx + (sx - centerX) / scale;
    const worldZ = cz + (sy - centerY) / scale;

    return [worldX, 0, worldZ];
  },

  /**
   * Get nearest grid point to screen position
   */
  getNearestGridPoint(
    screenPos: [number, number],
    camera: CameraState,
    viewportSize: [number, number],
    gridSize: number
  ): [number, number, number] {
    const worldPos = GridHelpers.screenToWorld(screenPos, camera, viewportSize);
    return snapToGrid(worldPos, {
      ...DEFAULT_SNAP_CONFIG,
      gridSnapSize: gridSize,
    });
  },

  /**
   * Calculate grid level of detail based on zoom
   */
  calculateGridLOD(camera: CameraState): number {
    const zoom = camera.zoom || 1.0;

    if (zoom < 0.1) return 3; // Very zoomed out - show only major grid
    if (zoom < 1.0) return 2; // Zoomed out - show primary grid
    if (zoom < 10.0) return 1; // Normal - show subdivisions
    return 0; // Zoomed in - show all detail
  },
};

export default ViewportGrid;
