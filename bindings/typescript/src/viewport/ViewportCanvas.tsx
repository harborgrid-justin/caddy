/**
 * Viewport Canvas Component - Hardware-Accelerated React Component
 *
 * Enterprise-grade WebGL/WebGPU canvas component for CADDY v0.2.5
 * Provides hardware-accelerated rendering with React integration.
 *
 * @module ViewportCanvas
 */

import React, { useEffect, useRef, useState, useCallback, useMemo } from 'react';
import { ViewportId, ViewportConfig, RenderStatistics, CameraState } from './ViewportManager';

/**
 * Canvas rendering backend
 */
export enum RenderBackend {
  WebGL2 = 'webgl2',
  WebGPU = 'webgpu',
  Auto = 'auto',
}

/**
 * Viewport canvas props
 */
export interface ViewportCanvasProps {
  /** Viewport identifier */
  viewportId: ViewportId;

  /** Viewport configuration */
  config: ViewportConfig;

  /** Camera state */
  camera: CameraState;

  /** Rendering backend preference */
  backend?: RenderBackend;

  /** Enable pointer events */
  enablePointerEvents?: boolean;

  /** Enable keyboard events */
  enableKeyboardEvents?: boolean;

  /** Custom render callback */
  onRender?: (context: RenderContext) => void;

  /** Statistics update callback */
  onStatsUpdate?: (stats: RenderStatistics) => void;

  /** Error callback */
  onError?: (error: Error) => void;

  /** Canvas ready callback */
  onReady?: (canvas: HTMLCanvasElement) => void;

  /** Custom CSS class */
  className?: string;

  /** Custom styles */
  style?: React.CSSProperties;

  /** Children elements (overlays) */
  children?: React.ReactNode;
}

/**
 * Render context passed to custom render callbacks
 */
export interface RenderContext {
  canvas: HTMLCanvasElement;
  context: WebGL2RenderingContext | GPUCanvasContext;
  width: number;
  height: number;
  deltaTime: number;
  timestamp: number;
  frameCount: number;
}

/**
 * WebGL context attributes
 */
const WEBGL_CONTEXT_ATTRIBUTES: WebGLContextAttributes = {
  alpha: false,
  depth: true,
  stencil: false,
  antialias: true,
  premultipliedAlpha: false,
  preserveDrawingBuffer: false,
  powerPreference: 'high-performance',
  failIfMajorPerformanceCaveat: false,
};

/**
 * Viewport Canvas Component
 *
 * Hardware-accelerated canvas component with WebGL2/WebGPU support.
 */
export const ViewportCanvas: React.FC<ViewportCanvasProps> = React.memo(({
  viewportId,
  config,
  camera,
  backend = RenderBackend.Auto,
  enablePointerEvents = true,
  enableKeyboardEvents = false,
  onRender,
  onStatsUpdate,
  onError,
  onReady,
  className,
  style,
  children,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const renderContextRef = useRef<WebGL2RenderingContext | GPUCanvasContext | null>(null);
  const animationFrameRef = useRef<number | null>(null);
  const lastFrameTimeRef = useRef<number>(0);
  const frameCountRef = useRef<number>(0);
  const statsRef = useRef<RenderStatistics>({
    fps: 0,
    frameTimeMs: 0,
    drawCalls: 0,
    verticesRendered: 0,
    trianglesRendered: 0,
    objectsCulled: 0,
    objectsRendered: 0,
    gpuMemoryBytes: 0,
    cpuTimeMs: 0,
    gpuTimeMs: 0,
  });

  const [isReady, setIsReady] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  /**
   * Initialize rendering context
   */
  const initializeContext = useCallback(async () => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    try {
      let context: WebGL2RenderingContext | GPUCanvasContext | null = null;

      // Try WebGPU first if available and requested
      if (backend === RenderBackend.WebGPU || backend === RenderBackend.Auto) {
        if ('gpu' in navigator) {
          try {
            const gpu = (navigator as any).gpu;
            const adapter = await gpu.requestAdapter();
            if (adapter) {
              const device = await adapter.requestDevice();
              context = canvas.getContext('webgpu') as GPUCanvasContext;

              if (context) {
                // Configure WebGPU context
                const format = gpu.getPreferredCanvasFormat();
                (context as any).configure({
                  device,
                  format,
                  alphaMode: 'opaque',
                });
              }
            }
          } catch (e) {
            console.warn('WebGPU initialization failed, falling back to WebGL2:', e);
          }
        }
      }

      // Fall back to WebGL2
      if (!context && (backend === RenderBackend.WebGL2 || backend === RenderBackend.Auto)) {
        context = canvas.getContext('webgl2', WEBGL_CONTEXT_ATTRIBUTES) as WebGL2RenderingContext;

        if (!context) {
          throw new Error('WebGL2 is not supported in this browser');
        }

        // Set up WebGL state
        context.enable(context.DEPTH_TEST);
        context.depthFunc(context.LEQUAL);
        context.enable(context.CULL_FACE);
        context.cullFace(context.BACK);

        // Set clear color
        const [r, g, b, a] = config.backgroundColor;
        context.clearColor(r, g, b, a);
      }

      if (!context) {
        throw new Error('Failed to initialize rendering context');
      }

      renderContextRef.current = context;
      setIsReady(true);

      if (onReady) {
        onReady(canvas);
      }
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      if (onError) {
        onError(error);
      }
    }
  }, [backend, config.backgroundColor, onReady, onError]);

  /**
   * Resize canvas to match display size
   */
  const resizeCanvas = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const displayWidth = config.width;
    const displayHeight = config.height;

    const needResize =
      canvas.width !== displayWidth || canvas.height !== displayHeight;

    if (needResize) {
      canvas.width = displayWidth;
      canvas.height = displayHeight;

      // Update WebGL viewport
      if (renderContextRef.current && 'viewport' in renderContextRef.current) {
        const gl = renderContextRef.current as WebGL2RenderingContext;
        gl.viewport(0, 0, displayWidth, displayHeight);
      }
    }
  }, [config.width, config.height]);

  /**
   * Render frame
   */
  const renderFrame = useCallback((timestamp: number) => {
    const canvas = canvasRef.current;
    const context = renderContextRef.current;

    if (!canvas || !context || !isReady) {
      return;
    }

    const deltaTime = lastFrameTimeRef.current
      ? (timestamp - lastFrameTimeRef.current) / 1000
      : 0;
    lastFrameTimeRef.current = timestamp;
    frameCountRef.current++;

    // Clear canvas
    if ('clear' in context) {
      const gl = context as WebGL2RenderingContext;
      gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
    }

    // Create render context
    const renderContext: RenderContext = {
      canvas,
      context,
      width: config.width,
      height: config.height,
      deltaTime,
      timestamp,
      frameCount: frameCountRef.current,
    };

    // Call custom render callback
    if (onRender) {
      try {
        onRender(renderContext);
      } catch (err) {
        console.error('Render callback error:', err);
        if (onError) {
          const error = err instanceof Error ? err : new Error(String(err));
          onError(error);
        }
      }
    }

    // Update statistics
    const stats = statsRef.current;
    stats.frameTimeMs = deltaTime * 1000;
    stats.fps = deltaTime > 0 ? 1 / deltaTime : 0;

    if (onStatsUpdate && frameCountRef.current % 60 === 0) {
      onStatsUpdate({ ...stats });
    }

    // Continue render loop if not limited by FPS
    if (config.maxFps === 0 || stats.fps < config.maxFps) {
      animationFrameRef.current = requestAnimationFrame(renderFrame);
    } else {
      // Schedule next frame based on max FPS
      const delay = (1000 / config.maxFps) - stats.frameTimeMs;
      setTimeout(() => {
        animationFrameRef.current = requestAnimationFrame(renderFrame);
      }, Math.max(0, delay));
    }
  }, [isReady, config.width, config.height, config.maxFps, onRender, onStatsUpdate, onError]);

  /**
   * Start render loop
   */
  const startRenderLoop = useCallback(() => {
    if (animationFrameRef.current === null && isReady) {
      lastFrameTimeRef.current = 0;
      animationFrameRef.current = requestAnimationFrame(renderFrame);
    }
  }, [isReady, renderFrame]);

  /**
   * Stop render loop
   */
  const stopRenderLoop = useCallback(() => {
    if (animationFrameRef.current !== null) {
      cancelAnimationFrame(animationFrameRef.current);
      animationFrameRef.current = null;
    }
  }, []);

  /**
   * Initialize on mount
   */
  useEffect(() => {
    initializeContext();

    return () => {
      stopRenderLoop();
    };
  }, [initializeContext, stopRenderLoop]);

  /**
   * Start render loop when ready
   */
  useEffect(() => {
    if (isReady) {
      startRenderLoop();
    }

    return () => {
      stopRenderLoop();
    };
  }, [isReady, startRenderLoop, stopRenderLoop]);

  /**
   * Handle resize
   */
  useEffect(() => {
    resizeCanvas();
  }, [resizeCanvas]);

  /**
   * Pointer event handlers
   */
  const handlePointerDown = useCallback((e: React.PointerEvent<HTMLCanvasElement>) => {
    if (!enablePointerEvents) return;
    e.currentTarget.setPointerCapture(e.pointerId);
  }, [enablePointerEvents]);

  const handlePointerUp = useCallback((e: React.PointerEvent<HTMLCanvasElement>) => {
    if (!enablePointerEvents) return;
    e.currentTarget.releasePointerCapture(e.pointerId);
  }, [enablePointerEvents]);

  const handlePointerMove = useCallback((e: React.PointerEvent<HTMLCanvasElement>) => {
    if (!enablePointerEvents) return;
    // Pointer move handling will be implemented in ViewportControls
  }, [enablePointerEvents]);

  const handleWheel = useCallback((e: React.WheelEvent<HTMLCanvasElement>) => {
    if (!enablePointerEvents) return;
    e.preventDefault();
    // Wheel handling will be implemented in ViewportControls
  }, [enablePointerEvents]);

  /**
   * Keyboard event handlers
   */
  const handleKeyDown = useCallback((e: React.KeyboardEvent<HTMLCanvasElement>) => {
    if (!enableKeyboardEvents) return;
    // Keyboard handling will be implemented in ViewportControls
  }, [enableKeyboardEvents]);

  /**
   * Canvas styles
   */
  const canvasStyle = useMemo<React.CSSProperties>(() => ({
    display: 'block',
    width: '100%',
    height: '100%',
    touchAction: 'none',
    userSelect: 'none',
    outline: 'none',
    ...style,
  }), [style]);

  /**
   * Container styles
   */
  const containerStyle = useMemo<React.CSSProperties>(() => ({
    position: 'relative',
    width: config.width,
    height: config.height,
    overflow: 'hidden',
  }), [config.width, config.height]);

  if (error) {
    return (
      <div style={containerStyle} className={className}>
        <div style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          width: '100%',
          height: '100%',
          backgroundColor: '#1a1a1a',
          color: '#ff4444',
          fontSize: '14px',
          fontFamily: 'monospace',
          padding: '20px',
          textAlign: 'center',
        }}>
          <div>
            <div style={{ fontWeight: 'bold', marginBottom: '10px' }}>
              Viewport Error
            </div>
            <div>{error.message}</div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div style={containerStyle} className={className}>
      <canvas
        ref={canvasRef}
        style={canvasStyle}
        tabIndex={enableKeyboardEvents ? 0 : -1}
        onPointerDown={handlePointerDown}
        onPointerUp={handlePointerUp}
        onPointerMove={handlePointerMove}
        onWheel={handleWheel}
        onKeyDown={handleKeyDown}
      />
      {children && (
        <div style={{
          position: 'absolute',
          top: 0,
          left: 0,
          width: '100%',
          height: '100%',
          pointerEvents: 'none',
        }}>
          {children}
        </div>
      )}
      {!isReady && (
        <div style={{
          position: 'absolute',
          top: 0,
          left: 0,
          width: '100%',
          height: '100%',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          backgroundColor: 'rgba(0, 0, 0, 0.8)',
          color: '#ffffff',
          fontSize: '14px',
          fontFamily: 'sans-serif',
        }}>
          Initializing viewport...
        </div>
      )}
    </div>
  );
});

ViewportCanvas.displayName = 'ViewportCanvas';

export default ViewportCanvas;
