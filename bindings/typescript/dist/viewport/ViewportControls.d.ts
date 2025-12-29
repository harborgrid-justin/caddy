import React from 'react';
import { CameraState, ViewportId } from './ViewportManager';
export declare enum ControlMode {
    Pan = "pan",
    Rotate = "rotate",
    Orbit = "orbit",
    Zoom = "zoom",
    None = "none"
}
export declare enum MouseButton {
    Left = 0,
    Middle = 1,
    Right = 2
}
export interface ViewportControlsConfig {
    enablePan: boolean;
    enableRotate: boolean;
    enableOrbit: boolean;
    enableZoom: boolean;
    panSpeed: number;
    rotateSpeed: number;
    zoomSpeed: number;
    dampingFactor: number;
    enableDamping: boolean;
    minZoom: number;
    maxZoom: number;
    autoRotateSpeed: number;
    enableTouch: boolean;
    touchZoomSpeed: number;
    touchRotateSpeed: number;
}
export declare const DEFAULT_CONTROLS_CONFIG: ViewportControlsConfig;
export interface ViewportControlsProps {
    viewportId: ViewportId;
    camera: CameraState;
    canvas: HTMLCanvasElement | null;
    config?: Partial<ViewportControlsConfig>;
    onCameraUpdate: (camera: Partial<CameraState>) => void;
    onControlModeChange?: (mode: ControlMode) => void;
    enabled?: boolean;
}
export declare function useViewportControls({ viewportId, camera, canvas, config: configProp, onCameraUpdate, onControlModeChange, enabled, }: ViewportControlsProps): {
    controlMode: ControlMode;
    isInteracting: boolean;
};
export declare const ViewportControls: React.FC<ViewportControlsProps>;
export default ViewportControls;
//# sourceMappingURL=ViewportControls.d.ts.map