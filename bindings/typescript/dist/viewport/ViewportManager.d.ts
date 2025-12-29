import { EventEmitter } from 'events';
export interface ViewportId {
    value: number;
}
export declare enum CameraMode {
    Orthographic = "orthographic",
    Perspective = "perspective",
    Isometric = "isometric"
}
export interface ViewportConfig {
    width: number;
    height: number;
    backgroundColor: [number, number, number, number];
    msaaSamples: number;
    vsync: boolean;
    maxFps: number;
    enableCulling: boolean;
    enableOcclusion: boolean;
    enableLod: boolean;
    gridSize: number;
    gridSubdivisions: number;
    showAxis: boolean;
    showStats: boolean;
}
export interface CameraState {
    mode: CameraMode;
    position: [number, number, number];
    target: [number, number, number];
    up: [number, number, number];
    zoom: number;
    fov?: number;
    aspectRatio: number;
}
export interface ViewportState {
    id: ViewportId;
    config: ViewportConfig;
    camera: CameraState;
    active: boolean;
    visible: boolean;
    position: [number, number];
    size: [number, number];
}
export declare enum ViewportLayout {
    Single = "single",
    Horizontal = "horizontal",
    Vertical = "vertical",
    Grid2x2 = "grid-2x2",
    Grid3x3 = "grid-3x3",
    Custom = "custom"
}
export interface RenderStatistics {
    fps: number;
    frameTimeMs: number;
    drawCalls: number;
    verticesRendered: number;
    trianglesRendered: number;
    objectsCulled: number;
    objectsRendered: number;
    gpuMemoryBytes: number;
    cpuTimeMs: number;
    gpuTimeMs: number;
}
export interface ViewportEvents {
    viewportAdded: (id: ViewportId) => void;
    viewportRemoved: (id: ViewportId) => void;
    viewportActivated: (id: ViewportId) => void;
    viewportResized: (id: ViewportId, width: number, height: number) => void;
    layoutChanged: (layout: ViewportLayout) => void;
    cameraChanged: (id: ViewportId, camera: CameraState) => void;
    renderComplete: (id: ViewportId, stats: RenderStatistics) => void;
}
export declare const DEFAULT_VIEWPORT_CONFIG: ViewportConfig;
export declare class ViewportManager extends EventEmitter {
    private viewports;
    private activeViewportId;
    private layout;
    private containerSize;
    private renderLoopId;
    private renderCallbacks;
    private lastFrameTime;
    constructor();
    addViewport(config?: Partial<ViewportConfig>): ViewportId;
    removeViewport(id: ViewportId): boolean;
    getViewport(id: ViewportId): ViewportState | null;
    getAllViewports(): ViewportState[];
    setActiveViewport(id: ViewportId): boolean;
    getActiveViewport(): ViewportState | null;
    setLayout(layout: ViewportLayout): void;
    getLayout(): ViewportLayout;
    private updateLayout;
    resize(width: number, height: number): void;
    updateCamera(id: ViewportId, camera: Partial<CameraState>): boolean;
    registerRenderCallback(id: ViewportId, callback: (deltaTime: number) => void): void;
    startRenderLoop(): void;
    stopRenderLoop(): void;
    isRenderLoopRunning(): boolean;
    synchronizeCameras(): void;
    getViewportCount(): number;
    clear(): void;
    exportConfig(): object;
    importConfig(config: any): void;
    destroy(): void;
}
export declare function getViewportManager(): ViewportManager;
export declare function resetViewportManager(): void;
//# sourceMappingURL=ViewportManager.d.ts.map