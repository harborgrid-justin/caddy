import React, { useMemo, useCallback, useEffect, useRef } from 'react';
export const DEFAULT_GRID_CONFIG = {
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
export const DEFAULT_SNAP_CONFIG = {
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
function calculateGridLines(camera, config, viewportSize) {
    const [width, height] = viewportSize;
    const primaryLines = [];
    const secondaryLines = [];
    const [px, py, pz] = camera.position;
    const zoom = camera.zoom || 1.0;
    const scale = 50.0 / zoom;
    const centerX = width / 2;
    const centerY = height / 2;
    const rangeX = (width / scale) * 1.5;
    const rangeY = (height / scale) * 1.5;
    const startX = Math.floor((px - rangeX / 2) / config.size) * config.size;
    const endX = Math.ceil((px + rangeX / 2) / config.size) * config.size;
    const startY = Math.floor((pz - rangeY / 2) / config.size) * config.size;
    const endY = Math.ceil((pz + rangeY / 2) / config.size) * config.size;
    const secondarySize = config.size / config.subdivisions;
    for (let x = startX; x <= endX; x += secondarySize) {
        const isPrimary = Math.abs(x % config.size) < 0.0001;
        const screenX = centerX + (x - px) * scale;
        const line = {
            start: [screenX, 0],
            end: [screenX, height],
        };
        if (isPrimary) {
            primaryLines.push(line);
        }
        else {
            secondaryLines.push(line);
        }
    }
    for (let y = startY; y <= endY; y += secondarySize) {
        const isPrimary = Math.abs(y % config.size) < 0.0001;
        const screenY = centerY + (y - pz) * scale;
        const line = {
            start: [0, screenY],
            end: [width, screenY],
        };
        if (isPrimary) {
            primaryLines.push(line);
        }
        else {
            secondaryLines.push(line);
        }
    }
    const axisLines = {
        x: null,
        y: null,
        z: null,
    };
    if (config.showAxis) {
        const zeroY = centerY + (0 - pz) * scale;
        if (zeroY >= 0 && zeroY <= height) {
            axisLines.x = {
                start: [0, zeroY],
                end: [width, zeroY],
            };
        }
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
export function snapToGrid(point, snapConfig) {
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
export function snapAngle(angle, snapConfig) {
    if (!snapConfig.enabled || !snapConfig.snapToAngle) {
        return angle;
    }
    const increment = (snapConfig.angleSnapIncrement * Math.PI) / 180.0;
    return Math.round(angle / increment) * increment;
}
export function isNearSnapPoint(screenPoint, snapPoint, threshold) {
    const dx = screenPoint[0] - snapPoint[0];
    const dy = screenPoint[1] - snapPoint[1];
    const distance = Math.sqrt(dx * dx + dy * dy);
    return distance <= threshold;
}
export const ViewportGrid = React.memo(({ camera, context, width, height, gridConfig: gridConfigProp, snapConfig: snapConfigProp, }) => {
    const gridConfig = useMemo(() => ({ ...DEFAULT_GRID_CONFIG, ...gridConfigProp }), [gridConfigProp]);
    const snapConfig = useMemo(() => ({ ...DEFAULT_SNAP_CONFIG, ...snapConfigProp }), [snapConfigProp]);
    const gridLinesRef = useRef(null);
    const updateGridLines = useCallback(() => {
        if (!gridConfig.visible) {
            gridLinesRef.current = null;
            return;
        }
        gridLinesRef.current = calculateGridLines(camera, gridConfig, [width, height]);
    }, [camera, gridConfig, width, height]);
    const renderGrid = useCallback(() => {
        if (!context || !gridConfig.visible || !gridLinesRef.current) {
            return;
        }
        const { primaryLines, secondaryLines, axisLines } = gridLinesRef.current;
        context.save();
        context.strokeStyle = `rgba(${gridConfig.secondaryColor[0] * 255}, ${gridConfig.secondaryColor[1] * 255}, ${gridConfig.secondaryColor[2] * 255}, ${gridConfig.secondaryColor[3]})`;
        context.lineWidth = gridConfig.secondaryWidth;
        context.beginPath();
        for (const line of secondaryLines) {
            context.moveTo(line.start[0], line.start[1]);
            context.lineTo(line.end[0], line.end[1]);
        }
        context.stroke();
        context.strokeStyle = `rgba(${gridConfig.primaryColor[0] * 255}, ${gridConfig.primaryColor[1] * 255}, ${gridConfig.primaryColor[2] * 255}, ${gridConfig.primaryColor[3]})`;
        context.lineWidth = gridConfig.primaryWidth;
        context.beginPath();
        for (const line of primaryLines) {
            context.moveTo(line.start[0], line.start[1]);
            context.lineTo(line.end[0], line.end[1]);
        }
        context.stroke();
        if (gridConfig.showAxis) {
            if (axisLines.x) {
                context.strokeStyle = `rgba(${gridConfig.xAxisColor[0] * 255}, ${gridConfig.xAxisColor[1] * 255}, ${gridConfig.xAxisColor[2] * 255}, ${gridConfig.xAxisColor[3]})`;
                context.lineWidth = gridConfig.axisWidth;
                context.beginPath();
                context.moveTo(axisLines.x.start[0], axisLines.x.start[1]);
                context.lineTo(axisLines.x.end[0], axisLines.x.end[1]);
                context.stroke();
            }
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
    useEffect(() => {
        updateGridLines();
    }, [updateGridLines]);
    useEffect(() => {
        if (context && gridConfig.visible) {
            renderGrid();
        }
    }, [context, gridConfig.visible, renderGrid]);
    return null;
});
ViewportGrid.displayName = 'ViewportGrid';
export const GridHelpers = {
    worldToScreen(worldPos, camera, viewportSize) {
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
    screenToWorld(screenPos, camera, viewportSize) {
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
    getNearestGridPoint(screenPos, camera, viewportSize, gridSize) {
        const worldPos = GridHelpers.screenToWorld(screenPos, camera, viewportSize);
        return snapToGrid(worldPos, {
            ...DEFAULT_SNAP_CONFIG,
            gridSnapSize: gridSize,
        });
    },
    calculateGridLOD(camera) {
        const zoom = camera.zoom || 1.0;
        if (zoom < 0.1)
            return 3;
        if (zoom < 1.0)
            return 2;
        if (zoom < 10.0)
            return 1;
        return 0;
    },
};
export default ViewportGrid;
//# sourceMappingURL=ViewportGrid.js.map