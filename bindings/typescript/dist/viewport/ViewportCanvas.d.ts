import React from 'react';
import { ViewportId, ViewportConfig, RenderStatistics, CameraState } from './ViewportManager';
export declare enum RenderBackend {
    WebGL2 = "webgl2",
    WebGPU = "webgpu",
    Auto = "auto"
}
export interface ViewportCanvasProps {
    viewportId: ViewportId;
    config: ViewportConfig;
    camera: CameraState;
    backend?: RenderBackend;
    enablePointerEvents?: boolean;
    enableKeyboardEvents?: boolean;
    onRender?: (context: RenderContext) => void;
    onStatsUpdate?: (stats: RenderStatistics) => void;
    onError?: (error: Error) => void;
    onReady?: (canvas: HTMLCanvasElement) => void;
    className?: string;
    style?: React.CSSProperties;
    children?: React.ReactNode;
}
export interface RenderContext {
    canvas: HTMLCanvasElement;
    context: WebGL2RenderingContext | GPUCanvasContext;
    width: number;
    height: number;
    deltaTime: number;
    timestamp: number;
    frameCount: number;
}
export declare const ViewportCanvas: React.FC<ViewportCanvasProps>;
export default ViewportCanvas;
//# sourceMappingURL=ViewportCanvas.d.ts.map