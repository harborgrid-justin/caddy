/**
 * Viewport Manager - Multi-Viewport Orchestration
 *
 * Enterprise-grade viewport management system for CADDY v0.2.5
 * Handles multiple viewports, layouts, and coordination between views.
 *
 * @module ViewportManager
 */

import { EventEmitter } from 'events';

/**
 * Viewport identification
 */
export interface ViewportId {
  value: number;
}

/**
 * Camera mode enumeration
 */
export enum CameraMode {
  Orthographic = 'orthographic',
  Perspective = 'perspective',
  Isometric = 'isometric',
}

/**
 * Viewport configuration
 */
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

/**
 * Camera state
 */
export interface CameraState {
  mode: CameraMode;
  position: [number, number, number];
  target: [number, number, number];
  up: [number, number, number];
  zoom: number;
  fov?: number;
  aspectRatio: number;
}

/**
 * Viewport state
 */
export interface ViewportState {
  id: ViewportId;
  config: ViewportConfig;
  camera: CameraState;
  active: boolean;
  visible: boolean;
  position: [number, number];
  size: [number, number];
}

/**
 * Viewport layout types
 */
export enum ViewportLayout {
  Single = 'single',
  Horizontal = 'horizontal',
  Vertical = 'vertical',
  Grid2x2 = 'grid-2x2',
  Grid3x3 = 'grid-3x3',
  Custom = 'custom',
}

/**
 * Render statistics
 */
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

/**
 * Viewport events
 */
export interface ViewportEvents {
  viewportAdded: (id: ViewportId) => void;
  viewportRemoved: (id: ViewportId) => void;
  viewportActivated: (id: ViewportId) => void;
  viewportResized: (id: ViewportId, width: number, height: number) => void;
  layoutChanged: (layout: ViewportLayout) => void;
  cameraChanged: (id: ViewportId, camera: CameraState) => void;
  renderComplete: (id: ViewportId, stats: RenderStatistics) => void;
}

/**
 * Default viewport configuration
 */
export const DEFAULT_VIEWPORT_CONFIG: ViewportConfig = {
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

/**
 * Viewport Manager Class
 *
 * Orchestrates multiple viewports, managing their lifecycle, layout, and synchronization.
 */
export class ViewportManager extends EventEmitter {
  private viewports: Map<number, ViewportState>;
  private activeViewportId: ViewportId | null;
  private layout: ViewportLayout;
  private containerSize: [number, number];
  private renderLoopId: number | null;
  private renderCallbacks: Map<number, (deltaTime: number) => void>;
  private lastFrameTime: number;

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

  /**
   * Add a new viewport
   */
  public addViewport(config?: Partial<ViewportConfig>): ViewportId {
    const id: ViewportId = { value: this.viewports.size };

    const fullConfig: ViewportConfig = {
      ...DEFAULT_VIEWPORT_CONFIG,
      ...config,
    };

    const camera: CameraState = {
      mode: CameraMode.Perspective,
      position: [0, 0, 10],
      target: [0, 0, 0],
      up: [0, 1, 0],
      zoom: 1.0,
      fov: Math.PI / 4,
      aspectRatio: fullConfig.width / fullConfig.height,
    };

    const state: ViewportState = {
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

  /**
   * Remove a viewport
   */
  public removeViewport(id: ViewportId): boolean {
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

  /**
   * Get viewport state
   */
  public getViewport(id: ViewportId): ViewportState | null {
    return this.viewports.get(id.value) || null;
  }

  /**
   * Get all viewports
   */
  public getAllViewports(): ViewportState[] {
    return Array.from(this.viewports.values());
  }

  /**
   * Set active viewport
   */
  public setActiveViewport(id: ViewportId): boolean {
    if (!this.viewports.has(id.value)) {
      return false;
    }

    // Deactivate all viewports
    this.viewports.forEach((viewport) => {
      viewport.active = false;
    });

    // Activate the specified viewport
    const viewport = this.viewports.get(id.value)!;
    viewport.active = true;
    this.activeViewportId = id;

    this.emit('viewportActivated', id);

    return true;
  }

  /**
   * Get active viewport
   */
  public getActiveViewport(): ViewportState | null {
    if (this.activeViewportId === null) {
      return null;
    }
    return this.viewports.get(this.activeViewportId.value) || null;
  }

  /**
   * Set viewport layout
   */
  public setLayout(layout: ViewportLayout): void {
    this.layout = layout;
    this.updateLayout();
    this.emit('layoutChanged', layout);
  }

  /**
   * Get current layout
   */
  public getLayout(): ViewportLayout {
    return this.layout;
  }

  /**
   * Update layout calculations
   */
  private updateLayout(): void {
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
        // Custom layout handled externally
        break;
    }
  }

  /**
   * Resize container
   */
  public resize(width: number, height: number): void {
    this.containerSize = [width, height];
    this.updateLayout();

    this.viewports.forEach((vp) => {
      this.emit('viewportResized', vp.id, vp.config.width, vp.config.height);
    });
  }

  /**
   * Update camera for a viewport
   */
  public updateCamera(id: ViewportId, camera: Partial<CameraState>): boolean {
    const viewport = this.viewports.get(id.value);
    if (!viewport) {
      return false;
    }

    viewport.camera = { ...viewport.camera, ...camera };
    this.emit('cameraChanged', id, viewport.camera);

    return true;
  }

  /**
   * Register render callback for a viewport
   */
  public registerRenderCallback(id: ViewportId, callback: (deltaTime: number) => void): void {
    this.renderCallbacks.set(id.value, callback);
  }

  /**
   * Start render loop
   */
  public startRenderLoop(): void {
    if (this.renderLoopId !== null) {
      return;
    }

    const renderFrame = (currentTime: number) => {
      const deltaTime = (currentTime - this.lastFrameTime) / 1000;
      this.lastFrameTime = currentTime;

      // Render all visible viewports
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

  /**
   * Stop render loop
   */
  public stopRenderLoop(): void {
    if (this.renderLoopId !== null) {
      cancelAnimationFrame(this.renderLoopId);
      this.renderLoopId = null;
    }
  }

  /**
   * Check if render loop is running
   */
  public isRenderLoopRunning(): boolean {
    return this.renderLoopId !== null;
  }

  /**
   * Synchronize camera across all viewports
   */
  public synchronizeCameras(): void {
    const active = this.getActiveViewport();
    if (!active) {
      return;
    }

    const { camera } = active;

    this.viewports.forEach((vp) => {
      if (vp.id.value !== active.id.value) {
        vp.camera = {
          ...camera,
          aspectRatio: vp.camera.aspectRatio, // Keep individual aspect ratios
        };
        this.emit('cameraChanged', vp.id, vp.camera);
      }
    });
  }

  /**
   * Get viewport count
   */
  public getViewportCount(): number {
    return this.viewports.size;
  }

  /**
   * Clear all viewports
   */
  public clear(): void {
    this.stopRenderLoop();
    this.viewports.clear();
    this.renderCallbacks.clear();
    this.activeViewportId = null;
  }

  /**
   * Export viewport configuration
   */
  public exportConfig(): object {
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

  /**
   * Import viewport configuration
   */
  public importConfig(config: any): void {
    this.clear();

    if (config.layout) {
      this.layout = config.layout;
    }

    if (config.containerSize) {
      this.containerSize = config.containerSize;
    }

    if (config.viewports && Array.isArray(config.viewports)) {
      config.viewports.forEach((vpConfig: any) => {
        const id = this.addViewport(vpConfig.config);
        const viewport = this.viewports.get(id.value);
        if (viewport && vpConfig.camera) {
          viewport.camera = vpConfig.camera;
        }
      });
    }
  }

  /**
   * Destroy the manager
   */
  public destroy(): void {
    this.clear();
    this.removeAllListeners();
  }
}

/**
 * Create a singleton viewport manager instance
 */
let globalViewportManager: ViewportManager | null = null;

export function getViewportManager(): ViewportManager {
  if (!globalViewportManager) {
    globalViewportManager = new ViewportManager();
  }
  return globalViewportManager;
}

export function resetViewportManager(): void {
  if (globalViewportManager) {
    globalViewportManager.destroy();
    globalViewportManager = null;
  }
}
