import React, { useEffect, useRef, useState, useCallback, useMemo } from 'react';
export var RenderBackend;
(function (RenderBackend) {
    RenderBackend["WebGL2"] = "webgl2";
    RenderBackend["WebGPU"] = "webgpu";
    RenderBackend["Auto"] = "auto";
})(RenderBackend || (RenderBackend = {}));
const WEBGL_CONTEXT_ATTRIBUTES = {
    alpha: false,
    depth: true,
    stencil: false,
    antialias: true,
    premultipliedAlpha: false,
    preserveDrawingBuffer: false,
    powerPreference: 'high-performance',
    failIfMajorPerformanceCaveat: false,
};
export const ViewportCanvas = React.memo(({ viewportId, config, camera, backend = RenderBackend.Auto, enablePointerEvents = true, enableKeyboardEvents = false, onRender, onStatsUpdate, onError, onReady, className, style, children, }) => {
    const canvasRef = useRef(null);
    const renderContextRef = useRef(null);
    const animationFrameRef = useRef(null);
    const lastFrameTimeRef = useRef(0);
    const frameCountRef = useRef(0);
    const statsRef = useRef({
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
    const [error, setError] = useState(null);
    const initializeContext = useCallback(async () => {
        const canvas = canvasRef.current;
        if (!canvas)
            return;
        try {
            let context = null;
            if (backend === RenderBackend.WebGPU || backend === RenderBackend.Auto) {
                if ('gpu' in navigator) {
                    try {
                        const gpu = navigator.gpu;
                        const adapter = await gpu.requestAdapter();
                        if (adapter) {
                            const device = await adapter.requestDevice();
                            context = canvas.getContext('webgpu');
                            if (context) {
                                const format = gpu.getPreferredCanvasFormat();
                                context.configure({
                                    device,
                                    format,
                                    alphaMode: 'opaque',
                                });
                            }
                        }
                    }
                    catch (e) {
                        console.warn('WebGPU initialization failed, falling back to WebGL2:', e);
                    }
                }
            }
            if (!context && (backend === RenderBackend.WebGL2 || backend === RenderBackend.Auto)) {
                context = canvas.getContext('webgl2', WEBGL_CONTEXT_ATTRIBUTES);
                if (!context) {
                    throw new Error('WebGL2 is not supported in this browser');
                }
                context.enable(context.DEPTH_TEST);
                context.depthFunc(context.LEQUAL);
                context.enable(context.CULL_FACE);
                context.cullFace(context.BACK);
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
        }
        catch (err) {
            const error = err instanceof Error ? err : new Error(String(err));
            setError(error);
            if (onError) {
                onError(error);
            }
        }
    }, [backend, config.backgroundColor, onReady, onError]);
    const resizeCanvas = useCallback(() => {
        const canvas = canvasRef.current;
        if (!canvas)
            return;
        const displayWidth = config.width;
        const displayHeight = config.height;
        const needResize = canvas.width !== displayWidth || canvas.height !== displayHeight;
        if (needResize) {
            canvas.width = displayWidth;
            canvas.height = displayHeight;
            if (renderContextRef.current && 'viewport' in renderContextRef.current) {
                const gl = renderContextRef.current;
                gl.viewport(0, 0, displayWidth, displayHeight);
            }
        }
    }, [config.width, config.height]);
    const renderFrame = useCallback((timestamp) => {
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
        if ('clear' in context) {
            const gl = context;
            gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
        }
        const renderContext = {
            canvas,
            context,
            width: config.width,
            height: config.height,
            deltaTime,
            timestamp,
            frameCount: frameCountRef.current,
        };
        if (onRender) {
            try {
                onRender(renderContext);
            }
            catch (err) {
                console.error('Render callback error:', err);
                if (onError) {
                    const error = err instanceof Error ? err : new Error(String(err));
                    onError(error);
                }
            }
        }
        const stats = statsRef.current;
        stats.frameTimeMs = deltaTime * 1000;
        stats.fps = deltaTime > 0 ? 1 / deltaTime : 0;
        if (onStatsUpdate && frameCountRef.current % 60 === 0) {
            onStatsUpdate({ ...stats });
        }
        if (config.maxFps === 0 || stats.fps < config.maxFps) {
            animationFrameRef.current = requestAnimationFrame(renderFrame);
        }
        else {
            const delay = (1000 / config.maxFps) - stats.frameTimeMs;
            setTimeout(() => {
                animationFrameRef.current = requestAnimationFrame(renderFrame);
            }, Math.max(0, delay));
        }
    }, [isReady, config.width, config.height, config.maxFps, onRender, onStatsUpdate, onError]);
    const startRenderLoop = useCallback(() => {
        if (animationFrameRef.current === null && isReady) {
            lastFrameTimeRef.current = 0;
            animationFrameRef.current = requestAnimationFrame(renderFrame);
        }
    }, [isReady, renderFrame]);
    const stopRenderLoop = useCallback(() => {
        if (animationFrameRef.current !== null) {
            cancelAnimationFrame(animationFrameRef.current);
            animationFrameRef.current = null;
        }
    }, []);
    useEffect(() => {
        initializeContext();
        return () => {
            stopRenderLoop();
        };
    }, [initializeContext, stopRenderLoop]);
    useEffect(() => {
        if (isReady) {
            startRenderLoop();
        }
        return () => {
            stopRenderLoop();
        };
    }, [isReady, startRenderLoop, stopRenderLoop]);
    useEffect(() => {
        resizeCanvas();
    }, [resizeCanvas]);
    const handlePointerDown = useCallback((e) => {
        if (!enablePointerEvents)
            return;
        e.currentTarget.setPointerCapture(e.pointerId);
    }, [enablePointerEvents]);
    const handlePointerUp = useCallback((e) => {
        if (!enablePointerEvents)
            return;
        e.currentTarget.releasePointerCapture(e.pointerId);
    }, [enablePointerEvents]);
    const handlePointerMove = useCallback((e) => {
        if (!enablePointerEvents)
            return;
    }, [enablePointerEvents]);
    const handleWheel = useCallback((e) => {
        if (!enablePointerEvents)
            return;
        e.preventDefault();
    }, [enablePointerEvents]);
    const handleKeyDown = useCallback((e) => {
        if (!enableKeyboardEvents)
            return;
    }, [enableKeyboardEvents]);
    const canvasStyle = useMemo(() => ({
        display: 'block',
        width: '100%',
        height: '100%',
        touchAction: 'none',
        userSelect: 'none',
        outline: 'none',
        ...style,
    }), [style]);
    const containerStyle = useMemo(() => ({
        position: 'relative',
        width: config.width,
        height: config.height,
        overflow: 'hidden',
    }), [config.width, config.height]);
    if (error) {
        return (React.createElement("div", { style: containerStyle, className: className },
            React.createElement("div", { style: {
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
                } },
                React.createElement("div", null,
                    React.createElement("div", { style: { fontWeight: 'bold', marginBottom: '10px' } }, "Viewport Error"),
                    React.createElement("div", null, error.message)))));
    }
    return (React.createElement("div", { style: containerStyle, className: className },
        React.createElement("canvas", { ref: canvasRef, style: canvasStyle, tabIndex: enableKeyboardEvents ? 0 : -1, onPointerDown: handlePointerDown, onPointerUp: handlePointerUp, onPointerMove: handlePointerMove, onWheel: handleWheel, onKeyDown: handleKeyDown }),
        children && (React.createElement("div", { style: {
                position: 'absolute',
                top: 0,
                left: 0,
                width: '100%',
                height: '100%',
                pointerEvents: 'none',
            } }, children)),
        !isReady && (React.createElement("div", { style: {
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
            } }, "Initializing viewport..."))));
});
ViewportCanvas.displayName = 'ViewportCanvas';
export default ViewportCanvas;
//# sourceMappingURL=ViewportCanvas.js.map