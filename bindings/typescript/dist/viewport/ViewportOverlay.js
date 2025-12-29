import React, { useMemo } from 'react';
export var OverlayType;
(function (OverlayType) {
    OverlayType["Text"] = "text";
    OverlayType["Measurement"] = "measurement";
    OverlayType["Dimension"] = "dimension";
    OverlayType["Annotation"] = "annotation";
    OverlayType["Cursor"] = "cursor";
    OverlayType["Icon"] = "icon";
    OverlayType["Line"] = "line";
    OverlayType["Rectangle"] = "rectangle";
    OverlayType["Circle"] = "circle";
})(OverlayType || (OverlayType = {}));
export var AnchorPosition;
(function (AnchorPosition) {
    AnchorPosition["TopLeft"] = "top-left";
    AnchorPosition["TopCenter"] = "top-center";
    AnchorPosition["TopRight"] = "top-right";
    AnchorPosition["MiddleLeft"] = "middle-left";
    AnchorPosition["Center"] = "center";
    AnchorPosition["MiddleRight"] = "middle-right";
    AnchorPosition["BottomLeft"] = "bottom-left";
    AnchorPosition["BottomCenter"] = "bottom-center";
    AnchorPosition["BottomRight"] = "bottom-right";
})(AnchorPosition || (AnchorPosition = {}));
const RenderTextOverlay = ({ overlay }) => {
    const style = {
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
    return React.createElement("div", { style: style }, overlay.text);
};
const RenderMeasurementOverlay = ({ overlay, camera, viewportSize }) => {
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
    return (React.createElement(React.Fragment, null,
        React.createElement("svg", { style: {
                position: 'absolute',
                top: 0,
                left: 0,
                width: '100%',
                height: '100%',
                pointerEvents: 'none',
                zIndex: overlay.zIndex || 100,
            } },
            React.createElement("line", { x1: startScreen[0], y1: startScreen[1], x2: endScreen[0], y2: endScreen[1], stroke: color, strokeWidth: lineWidth }),
            showEndpoints && (React.createElement(React.Fragment, null,
                React.createElement("circle", { cx: startScreen[0], cy: startScreen[1], r: 4, fill: color }),
                React.createElement("circle", { cx: endScreen[0], cy: endScreen[1], r: 4, fill: color })))),
        React.createElement("div", { style: {
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
            } },
            overlay.label && React.createElement("div", null, overlay.label),
            React.createElement("div", null,
                overlay.value.toFixed(2),
                " ",
                overlay.unit))));
};
const RenderAnnotationOverlay = ({ overlay, camera, viewportSize }) => {
    const screenPos = worldToScreen(overlay.worldPosition, camera, viewportSize);
    if (!screenPos) {
        return null;
    }
    const color = overlay.color || '#ffaa00';
    const size = overlay.size || 24;
    return (React.createElement("div", { style: {
            position: 'absolute',
            left: screenPos[0],
            top: screenPos[1],
            transform: 'translate(-50%, -100%)',
            zIndex: overlay.zIndex || 100,
            pointerEvents: 'auto',
            cursor: 'pointer',
        } },
        React.createElement("div", { style: {
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
            } }, overlay.icon || 'i'),
        React.createElement("div", { style: {
                backgroundColor: 'rgba(0, 0, 0, 0.9)',
                color: '#ffffff',
                padding: '8px 12px',
                borderRadius: 4,
                fontSize: 12,
                fontFamily: 'sans-serif',
                minWidth: 150,
                maxWidth: 300,
            } },
            React.createElement("div", { style: { fontWeight: 'bold', marginBottom: 4 } }, overlay.title),
            overlay.description && React.createElement("div", { style: { fontSize: 11 } }, overlay.description))));
};
function worldToScreen(worldPos, camera, viewportSize) {
    const [width, height] = viewportSize;
    const [wx, wy, wz] = worldPos;
    const [cx, cy, cz] = camera.position;
    const zoom = camera.zoom || 1.0;
    const scale = 50.0 / zoom;
    const centerX = width / 2;
    const centerY = height / 2;
    const screenX = centerX + (wx - cx) * scale;
    const screenY = centerY + (wz - cz) * scale;
    if (screenX < 0 || screenX > width || screenY < 0 || screenY > height) {
        return null;
    }
    return [screenX, screenY];
}
const StatsPanel = ({ stats, position = AnchorPosition.TopLeft, }) => {
    const positionStyle = useMemo(() => {
        const base = {
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
    return (React.createElement("div", { style: positionStyle },
        React.createElement("div", { style: { marginBottom: 8, fontWeight: 'bold', color: '#ffffff' } }, "Performance Stats"),
        React.createElement("div", null,
            "FPS: ",
            stats.fps.toFixed(1)),
        React.createElement("div", null,
            "Frame Time: ",
            stats.frameTimeMs.toFixed(2),
            "ms"),
        React.createElement("div", null,
            "Draw Calls: ",
            stats.drawCalls),
        React.createElement("div", null,
            "Vertices: ",
            stats.verticesRendered.toLocaleString()),
        React.createElement("div", null,
            "Triangles: ",
            stats.trianglesRendered.toLocaleString()),
        React.createElement("div", null,
            "Culled: ",
            stats.objectsCulled),
        React.createElement("div", null,
            "Rendered: ",
            stats.objectsRendered),
        React.createElement("div", { style: { marginTop: 8 } },
            "GPU Memory: ",
            (stats.gpuMemoryBytes / 1024 / 1024).toFixed(2),
            "MB"),
        React.createElement("div", null,
            "CPU Time: ",
            stats.cpuTimeMs.toFixed(2),
            "ms"),
        React.createElement("div", null,
            "GPU Time: ",
            stats.gpuTimeMs.toFixed(2),
            "ms")));
};
const FPSCounter = ({ fps, position = AnchorPosition.TopRight, }) => {
    const positionStyle = useMemo(() => {
        const base = {
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
    return React.createElement("div", { style: positionStyle },
        fps.toFixed(1),
        " FPS");
};
const CoordinateDisplay = ({ position, anchorPosition = AnchorPosition.BottomLeft }) => {
    if (!position) {
        return null;
    }
    const [x, y, z] = position;
    const positionStyle = useMemo(() => {
        const base = {
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
    return (React.createElement("div", { style: positionStyle },
        "X: ",
        x.toFixed(2),
        " Y: ",
        y.toFixed(2),
        " Z: ",
        z.toFixed(2)));
};
export const ViewportOverlay = React.memo(({ camera, width, height, overlays = [], showFPS = false, showStats = false, stats, showCoordinates = false, cursorPosition, className, style, }) => {
    const containerStyle = useMemo(() => ({
        position: 'absolute',
        top: 0,
        left: 0,
        width: '100%',
        height: '100%',
        pointerEvents: 'none',
        ...style,
    }), [style]);
    const viewportSize = [width, height];
    return (React.createElement("div", { style: containerStyle, className: className },
        overlays
            .filter((overlay) => overlay.visible)
            .sort((a, b) => (a.zIndex || 100) - (b.zIndex || 100))
            .map((overlay) => {
            switch (overlay.type) {
                case OverlayType.Text:
                    return React.createElement(RenderTextOverlay, { key: overlay.id, overlay: overlay });
                case OverlayType.Measurement:
                    return (React.createElement(RenderMeasurementOverlay, { key: overlay.id, overlay: overlay, camera: camera, viewportSize: viewportSize }));
                case OverlayType.Annotation:
                    return (React.createElement(RenderAnnotationOverlay, { key: overlay.id, overlay: overlay, camera: camera, viewportSize: viewportSize }));
                default:
                    return null;
            }
        }),
        showFPS && stats && React.createElement(FPSCounter, { fps: stats.fps }),
        showStats && stats && React.createElement(StatsPanel, { stats: stats }),
        showCoordinates && React.createElement(CoordinateDisplay, { position: cursorPosition })));
});
ViewportOverlay.displayName = 'ViewportOverlay';
export default ViewportOverlay;
//# sourceMappingURL=ViewportOverlay.js.map