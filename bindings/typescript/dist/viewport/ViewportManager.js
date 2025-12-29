import { EventEmitter } from 'events';
export var CameraMode;
(function (CameraMode) {
    CameraMode["Orthographic"] = "orthographic";
    CameraMode["Perspective"] = "perspective";
    CameraMode["Isometric"] = "isometric";
})(CameraMode || (CameraMode = {}));
export var ViewportLayout;
(function (ViewportLayout) {
    ViewportLayout["Single"] = "single";
    ViewportLayout["Horizontal"] = "horizontal";
    ViewportLayout["Vertical"] = "vertical";
    ViewportLayout["Grid2x2"] = "grid-2x2";
    ViewportLayout["Grid3x3"] = "grid-3x3";
    ViewportLayout["Custom"] = "custom";
})(ViewportLayout || (ViewportLayout = {}));
export const DEFAULT_VIEWPORT_CONFIG = {
    width: 1920,
    height: 1080,
    backgroundColor: [0.15, 0.15, 0.18, 1.0],
    msaaSamples: 4,
    vsync: true,
    maxFps: 60,
    enableCulling: true,
    enableOcclusion: true,
    enableLod: true,
    gridSize: 1.0,
    gridSubdivisions: 10,
    showAxis: true,
    showStats: false,
};
export class ViewportManager extends EventEmitter {
    constructor() {
        super();
        this.viewports = new Map();
        this.activeViewportId = null;
        this.layout = ViewportLayout.Single;
        this.containerSize = [1920, 1080];
        this.renderLoopId = null;
        this.renderCallbacks = new Map();
        this.lastFrameTime = performance.now();
    }
    addViewport(config) {
        const id = { value: this.viewports.size };
        const fullConfig = {
            ...DEFAULT_VIEWPORT_CONFIG,
            ...config,
        };
        const camera = {
            mode: CameraMode.Perspective,
            position: [0, 0, 10],
            target: [0, 0, 0],
            up: [0, 1, 0],
            zoom: 1.0,
            fov: Math.PI / 4,
            aspectRatio: fullConfig.width / fullConfig.height,
        };
        const state = {
            id,
            config: fullConfig,
            camera,
            active: this.viewports.size === 0,
            visible: true,
            position: [0, 0],
            size: [fullConfig.width, fullConfig.height],
        };
        this.viewports.set(id.value, state);
        if (this.activeViewportId === null) {
            this.activeViewportId = id;
        }
        this.updateLayout();
        this.emit('viewportAdded', id);
        return id;
    }
    removeViewport(id) {
        if (!this.viewports.has(id.value)) {
            return false;
        }
        this.viewports.delete(id.value);
        this.renderCallbacks.delete(id.value);
        if (this.activeViewportId?.value === id.value) {
            const firstId = this.viewports.keys().next().value;
            this.activeViewportId = firstId !== undefined ? { value: firstId } : null;
        }
        this.updateLayout();
        this.emit('viewportRemoved', id);
        return true;
    }
    getViewport(id) {
        return this.viewports.get(id.value) || null;
    }
    getAllViewports() {
        return Array.from(this.viewports.values());
    }
    setActiveViewport(id) {
        if (!this.viewports.has(id.value)) {
            return false;
        }
        this.viewports.forEach((viewport) => {
            viewport.active = false;
        });
        const viewport = this.viewports.get(id.value);
        viewport.active = true;
        this.activeViewportId = id;
        this.emit('viewportActivated', id);
        return true;
    }
    getActiveViewport() {
        if (this.activeViewportId === null) {
            return null;
        }
        return this.viewports.get(this.activeViewportId.value) || null;
    }
    setLayout(layout) {
        this.layout = layout;
        this.updateLayout();
        this.emit('layoutChanged', layout);
    }
    getLayout() {
        return this.layout;
    }
    updateLayout() {
        const [width, height] = this.containerSize;
        const viewportArray = Array.from(this.viewports.values());
        switch (this.layout) {
            case ViewportLayout.Single: {
                if (viewportArray.length > 0) {
                    const vp = viewportArray[0];
                    vp.position = [0, 0];
                    vp.size = [width, height];
                    vp.config.width = width;
                    vp.config.height = height;
                    vp.camera.aspectRatio = width / height;
                }
                break;
            }
            case ViewportLayout.Horizontal: {
                const vpWidth = width / viewportArray.length;
                viewportArray.forEach((vp, index) => {
                    vp.position = [index * vpWidth, 0];
                    vp.size = [vpWidth, height];
                    vp.config.width = vpWidth;
                    vp.config.height = height;
                    vp.camera.aspectRatio = vpWidth / height;
                });
                break;
            }
            case ViewportLayout.Vertical: {
                const vpHeight = height / viewportArray.length;
                viewportArray.forEach((vp, index) => {
                    vp.position = [0, index * vpHeight];
                    vp.size = [width, vpHeight];
                    vp.config.width = width;
                    vp.config.height = vpHeight;
                    vp.camera.aspectRatio = width / vpHeight;
                });
                break;
            }
            case ViewportLayout.Grid2x2: {
                const vpWidth = width / 2;
                const vpHeight = height / 2;
                viewportArray.forEach((vp, index) => {
                    const row = Math.floor(index / 2);
                    const col = index % 2;
                    vp.position = [col * vpWidth, row * vpHeight];
                    vp.size = [vpWidth, vpHeight];
                    vp.config.width = vpWidth;
                    vp.config.height = vpHeight;
                    vp.camera.aspectRatio = vpWidth / vpHeight;
                });
                break;
            }
            case ViewportLayout.Grid3x3: {
                const vpWidth = width / 3;
                const vpHeight = height / 3;
                viewportArray.forEach((vp, index) => {
                    const row = Math.floor(index / 3);
                    const col = index % 3;
                    vp.position = [col * vpWidth, row * vpHeight];
                    vp.size = [vpWidth, vpHeight];
                    vp.config.width = vpWidth;
                    vp.config.height = vpHeight;
                    vp.camera.aspectRatio = vpWidth / vpHeight;
                });
                break;
            }
            case ViewportLayout.Custom:
                break;
        }
    }
    resize(width, height) {
        this.containerSize = [width, height];
        this.updateLayout();
        this.viewports.forEach((vp) => {
            this.emit('viewportResized', vp.id, vp.config.width, vp.config.height);
        });
    }
    updateCamera(id, camera) {
        const viewport = this.viewports.get(id.value);
        if (!viewport) {
            return false;
        }
        viewport.camera = { ...viewport.camera, ...camera };
        this.emit('cameraChanged', id, viewport.camera);
        return true;
    }
    registerRenderCallback(id, callback) {
        this.renderCallbacks.set(id.value, callback);
    }
    startRenderLoop() {
        if (this.renderLoopId !== null) {
            return;
        }
        const renderFrame = (currentTime) => {
            const deltaTime = (currentTime - this.lastFrameTime) / 1000;
            this.lastFrameTime = currentTime;
            this.viewports.forEach((viewport) => {
                if (viewport.visible) {
                    const callback = this.renderCallbacks.get(viewport.id.value);
                    if (callback) {
                        callback(deltaTime);
                    }
                }
            });
            this.renderLoopId = requestAnimationFrame(renderFrame);
        };
        this.renderLoopId = requestAnimationFrame(renderFrame);
    }
    stopRenderLoop() {
        if (this.renderLoopId !== null) {
            cancelAnimationFrame(this.renderLoopId);
            this.renderLoopId = null;
        }
    }
    isRenderLoopRunning() {
        return this.renderLoopId !== null;
    }
    synchronizeCameras() {
        const active = this.getActiveViewport();
        if (!active) {
            return;
        }
        const { camera } = active;
        this.viewports.forEach((vp) => {
            if (vp.id.value !== active.id.value) {
                vp.camera = {
                    ...camera,
                    aspectRatio: vp.camera.aspectRatio,
                };
                this.emit('cameraChanged', vp.id, vp.camera);
            }
        });
    }
    getViewportCount() {
        return this.viewports.size;
    }
    clear() {
        this.stopRenderLoop();
        this.viewports.clear();
        this.renderCallbacks.clear();
        this.activeViewportId = null;
    }
    exportConfig() {
        return {
            layout: this.layout,
            containerSize: this.containerSize,
            viewports: Array.from(this.viewports.values()).map((vp) => ({
                id: vp.id,
                config: vp.config,
                camera: vp.camera,
                position: vp.position,
                size: vp.size,
            })),
        };
    }
    importConfig(config) {
        this.clear();
        if (config.layout) {
            this.layout = config.layout;
        }
        if (config.containerSize) {
            this.containerSize = config.containerSize;
        }
        if (config.viewports && Array.isArray(config.viewports)) {
            config.viewports.forEach((vpConfig) => {
                const id = this.addViewport(vpConfig.config);
                const viewport = this.viewports.get(id.value);
                if (viewport && vpConfig.camera) {
                    viewport.camera = vpConfig.camera;
                }
            });
        }
    }
    destroy() {
        this.clear();
        this.removeAllListeners();
    }
}
let globalViewportManager = null;
export function getViewportManager() {
    if (!globalViewportManager) {
        globalViewportManager = new ViewportManager();
    }
    return globalViewportManager;
}
export function resetViewportManager() {
    if (globalViewportManager) {
        globalViewportManager.destroy();
        globalViewportManager = null;
    }
}
//# sourceMappingURL=ViewportManager.js.map