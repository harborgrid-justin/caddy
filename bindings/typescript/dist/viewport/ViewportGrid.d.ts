import React from 'react';
import { CameraState } from './ViewportManager';
export interface GridConfig {
    size: number;
    subdivisions: number;
    visible: boolean;
    primaryColor: [number, number, number, number];
    secondaryColor: [number, number, number, number];
    primaryWidth: number;
    secondaryWidth: number;
    fadeAtDistance: boolean;
    fadeDistance: number;
    showAxis: boolean;
    xAxisColor: [number, number, number, number];
    yAxisColor: [number, number, number, number];
    zAxisColor: [number, number, number, number];
    axisWidth: number;
    dynamicLOD: boolean;
    plane: 'xy' | 'xz' | 'yz';
}
export declare const DEFAULT_GRID_CONFIG: GridConfig;
export interface SnapConfig {
    enabled: boolean;
    snapToGrid: boolean;
    gridSnapSize: number;
    snapToAngle: boolean;
    angleSnapIncrement: number;
    snapThreshold: number;
    showSnapIndicators: boolean;
    snapIndicatorColor: [number, number, number, number];
    snapIndicatorSize: number;
}
export declare const DEFAULT_SNAP_CONFIG: SnapConfig;
export interface ViewportGridProps {
    camera: CameraState;
    context: CanvasRenderingContext2D | null;
    width: number;
    height: number;
    gridConfig?: Partial<GridConfig>;
    snapConfig?: Partial<SnapConfig>;
}
export declare function snapToGrid(point: [number, number, number], snapConfig: SnapConfig): [number, number, number];
export declare function snapAngle(angle: number, snapConfig: SnapConfig): number;
export declare function isNearSnapPoint(screenPoint: [number, number], snapPoint: [number, number], threshold: number): boolean;
export declare const ViewportGrid: React.FC<ViewportGridProps>;
export declare const GridHelpers: {
    worldToScreen(worldPos: [number, number, number], camera: CameraState, viewportSize: [number, number]): [number, number];
    screenToWorld(screenPos: [number, number], camera: CameraState, viewportSize: [number, number]): [number, number, number];
    getNearestGridPoint(screenPos: [number, number], camera: CameraState, viewportSize: [number, number], gridSize: number): [number, number, number];
    calculateGridLOD(camera: CameraState): number;
};
export default ViewportGrid;
//# sourceMappingURL=ViewportGrid.d.ts.map