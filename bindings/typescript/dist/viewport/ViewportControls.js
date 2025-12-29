import { useRef, useCallback, useEffect, useState } from 'react';
import { CameraMode } from './ViewportManager';
export var ControlMode;
(function (ControlMode) {
    ControlMode["Pan"] = "pan";
    ControlMode["Rotate"] = "rotate";
    ControlMode["Orbit"] = "orbit";
    ControlMode["Zoom"] = "zoom";
    ControlMode["None"] = "none";
})(ControlMode || (ControlMode = {}));
export var MouseButton;
(function (MouseButton) {
    MouseButton[MouseButton["Left"] = 0] = "Left";
    MouseButton[MouseButton["Middle"] = 1] = "Middle";
    MouseButton[MouseButton["Right"] = 2] = "Right";
})(MouseButton || (MouseButton = {}));
export const DEFAULT_CONTROLS_CONFIG = {
    enablePan: true,
    enableRotate: true,
    enableOrbit: true,
    enableZoom: true,
    panSpeed: 1.0,
    rotateSpeed: 1.0,
    zoomSpeed: 1.0,
    dampingFactor: 0.05,
    enableDamping: true,
    minZoom: 0.1,
    maxZoom: 1000.0,
    autoRotateSpeed: 0.0,
    enableTouch: true,
    touchZoomSpeed: 0.5,
    touchRotateSpeed: 0.5,
};
export function useViewportControls({ viewportId, camera, canvas, config: configProp, onCameraUpdate, onControlModeChange, enabled = true, }) {
    const config = { ...DEFAULT_CONTROLS_CONFIG, ...configProp };
    const [controlMode, setControlMode] = useState(ControlMode.None);
    const [isInteracting, setIsInteracting] = useState(false);
    const pointers = useRef(new Map());
    const lastTouchDistance = useRef(0);
    const velocity = useRef({ x: 0, y: 0, zoom: 0 });
    const animationFrameId = useRef(null);
    const getControlMode = useCallback((button, shiftKey, altKey, ctrlKey) => {
        if (!enabled)
            return ControlMode.None;
        if (button === MouseButton.Middle && config.enablePan) {
            return ControlMode.Pan;
        }
        if (button === MouseButton.Left && shiftKey && config.enablePan) {
            return ControlMode.Pan;
        }
        if (button === MouseButton.Left && altKey && config.enableRotate) {
            return ControlMode.Rotate;
        }
        if (button === MouseButton.Right && config.enableOrbit) {
            return camera.mode === CameraMode.Perspective
                ? ControlMode.Orbit
                : ControlMode.Rotate;
        }
        if (button === MouseButton.Left && !shiftKey && !altKey && !ctrlKey) {
            if (camera.mode === CameraMode.Perspective && config.enableOrbit) {
                return ControlMode.Orbit;
            }
        }
        return ControlMode.None;
    }, [enabled, config, camera.mode]);
    const updatePan = useCallback((deltaX, deltaY) => {
        if (!config.enablePan)
            return;
        const panSpeed = config.panSpeed * 0.002;
        const [px, py, pz] = camera.position;
        const [tx, ty, tz] = camera.target;
        const [ux, uy, uz] = camera.up;
        const fx = tx - px;
        const fy = ty - py;
        const fz = tz - pz;
        const flen = Math.sqrt(fx * fx + fy * fy + fz * fz);
        const fnx = fx / flen;
        const fny = fy / flen;
        const fnz = fz / flen;
        const rx = fny * uz - fnz * uy;
        const ry = fnz * ux - fnx * uz;
        const rz = fnx * uy - fny * ux;
        const rlen = Math.sqrt(rx * rx + ry * ry + rz * rz);
        const rnx = rx / rlen;
        const rny = ry / rlen;
        const rnz = rz / rlen;
        const tux = rny * fnz - rnz * fny;
        const tuy = rnz * fnx - rnx * fnz;
        const tuz = rnx * fny - rny * fnx;
        const distance = flen;
        const factor = panSpeed * distance;
        const offsetX = (rnx * deltaX + tux * deltaY) * factor;
        const offsetY = (rny * deltaX + tuy * deltaY) * factor;
        const offsetZ = (rnz * deltaX + tuz * deltaY) * factor;
        onCameraUpdate({
            position: [px + offsetX, py + offsetY, pz + offsetZ],
            target: [tx + offsetX, ty + offsetY, tz + offsetZ],
        });
    }, [config, camera, onCameraUpdate]);
    const updateRotate = useCallback((deltaX, deltaY) => {
        if (!config.enableRotate)
            return;
        const rotateSpeed = config.rotateSpeed * 0.005;
        const [px, py, pz] = camera.position;
        const [tx, ty, tz] = camera.target;
        let dx = tx - px;
        let dy = ty - py;
        let dz = tz - pz;
        const distance = Math.sqrt(dx * dx + dy * dy + dz * dz);
        dx /= distance;
        dy /= distance;
        dz /= distance;
        const yaw = -deltaX * rotateSpeed;
        const cosYaw = Math.cos(yaw);
        const sinYaw = Math.sin(yaw);
        const newDx = dx * cosYaw - dz * sinYaw;
        const newDz = dx * sinYaw + dz * cosYaw;
        dx = newDx;
        dz = newDz;
        const pitch = -deltaY * rotateSpeed;
        const cosPitch = Math.cos(pitch);
        const sinPitch = Math.sin(pitch);
        const newDy = dy * cosPitch - dz * sinPitch;
        const newDz2 = dy * sinPitch + dz * cosPitch;
        dy = newDy;
        dz = newDz2;
        onCameraUpdate({
            target: [
                px + dx * distance,
                py + dy * distance,
                pz + dz * distance,
            ],
        });
    }, [config, camera, onCameraUpdate]);
    const updateOrbit = useCallback((deltaX, deltaY) => {
        if (!config.enableOrbit)
            return;
        const rotateSpeed = config.rotateSpeed * 0.005;
        const [px, py, pz] = camera.position;
        const [tx, ty, tz] = camera.target;
        let dx = px - tx;
        let dy = py - ty;
        let dz = pz - tz;
        const distance = Math.sqrt(dx * dx + dy * dy + dz * dz);
        const radius = distance;
        let theta = Math.atan2(dz, dx);
        let phi = Math.acos(dy / radius);
        theta -= deltaX * rotateSpeed;
        phi -= deltaY * rotateSpeed;
        const epsilon = 0.01;
        phi = Math.max(epsilon, Math.min(Math.PI - epsilon, phi));
        const newDx = radius * Math.sin(phi) * Math.cos(theta);
        const newDy = radius * Math.cos(phi);
        const newDz = radius * Math.sin(phi) * Math.sin(theta);
        onCameraUpdate({
            position: [tx + newDx, ty + newDy, tz + newDz],
        });
    }, [config, camera, onCameraUpdate]);
    const updateZoom = useCallback((delta) => {
        if (!config.enableZoom)
            return;
        const zoomSpeed = config.zoomSpeed;
        if (camera.mode === CameraMode.Perspective) {
            const [px, py, pz] = camera.position;
            const [tx, ty, tz] = camera.target;
            let dx = tx - px;
            let dy = ty - py;
            let dz = tz - pz;
            const distance = Math.sqrt(dx * dx + dy * dy + dz * dz);
            dx /= distance;
            dy /= distance;
            dz /= distance;
            const newDistance = Math.max(config.minZoom, Math.min(config.maxZoom, distance - delta * zoomSpeed));
            onCameraUpdate({
                position: [
                    tx - dx * newDistance,
                    ty - dy * newDistance,
                    tz - dz * newDistance,
                ],
            });
        }
        else {
            const currentZoom = camera.zoom || 1.0;
            const newZoom = Math.max(config.minZoom, Math.min(config.maxZoom, currentZoom + delta * zoomSpeed * 0.1));
            onCameraUpdate({
                zoom: newZoom,
            });
        }
    }, [config, camera, onCameraUpdate]);
    const handlePointerDown = useCallback((e) => {
        if (!enabled || !canvas)
            return;
        e.preventDefault();
        const mode = getControlMode(e.button, e.shiftKey, e.altKey, e.ctrlKey);
        if (mode !== ControlMode.None) {
            const rect = canvas.getBoundingClientRect();
            pointers.current.set(e.pointerId, {
                id: e.pointerId,
                x: e.clientX - rect.left,
                y: e.clientY - rect.top,
                deltaX: 0,
                deltaY: 0,
                button: e.button,
            });
            setControlMode(mode);
            setIsInteracting(true);
            if (onControlModeChange) {
                onControlModeChange(mode);
            }
            canvas.setPointerCapture(e.pointerId);
        }
    }, [enabled, canvas, getControlMode, onControlModeChange]);
    const handlePointerMove = useCallback((e) => {
        if (!enabled || !canvas)
            return;
        const pointer = pointers.current.get(e.pointerId);
        if (!pointer)
            return;
        e.preventDefault();
        const rect = canvas.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;
        const deltaX = x - pointer.x;
        const deltaY = y - pointer.y;
        pointer.deltaX = deltaX;
        pointer.deltaY = deltaY;
        pointer.x = x;
        pointer.y = y;
        switch (controlMode) {
            case ControlMode.Pan:
                updatePan(deltaX, -deltaY);
                break;
            case ControlMode.Rotate:
                updateRotate(deltaX, deltaY);
                break;
            case ControlMode.Orbit:
                updateOrbit(deltaX, deltaY);
                break;
        }
        if (config.enableDamping) {
            velocity.current.x = deltaX;
            velocity.current.y = deltaY;
        }
    }, [enabled, canvas, controlMode, config.enableDamping, updatePan, updateRotate, updateOrbit]);
    const handlePointerUp = useCallback((e) => {
        if (!enabled || !canvas)
            return;
        pointers.current.delete(e.pointerId);
        if (pointers.current.size === 0) {
            setControlMode(ControlMode.None);
            setIsInteracting(false);
            if (onControlModeChange) {
                onControlModeChange(ControlMode.None);
            }
        }
        canvas.releasePointerCapture(e.pointerId);
    }, [enabled, canvas, onControlModeChange]);
    const handleWheel = useCallback((e) => {
        if (!enabled || !config.enableZoom)
            return;
        e.preventDefault();
        const delta = e.deltaY * 0.01;
        updateZoom(delta);
        if (config.enableDamping) {
            velocity.current.zoom = delta;
        }
    }, [enabled, config.enableZoom, config.enableDamping, updateZoom]);
    const handleTouchMove = useCallback((e) => {
        if (!enabled || !config.enableTouch || e.touches.length !== 2)
            return;
        e.preventDefault();
        const touch1 = e.touches[0];
        const touch2 = e.touches[1];
        const distance = Math.sqrt(Math.pow(touch2.clientX - touch1.clientX, 2) +
            Math.pow(touch2.clientY - touch1.clientY, 2));
        if (lastTouchDistance.current > 0) {
            const delta = (lastTouchDistance.current - distance) * config.touchZoomSpeed;
            updateZoom(delta);
        }
        lastTouchDistance.current = distance;
    }, [enabled, config.enableTouch, config.touchZoomSpeed, updateZoom]);
    const handleTouchEnd = useCallback(() => {
        lastTouchDistance.current = 0;
    }, []);
    const applyDamping = useCallback(() => {
        if (!config.enableDamping || !isInteracting) {
            const vx = velocity.current.x;
            const vy = velocity.current.y;
            const vz = velocity.current.zoom;
            if (Math.abs(vx) > 0.01 || Math.abs(vy) > 0.01 || Math.abs(vz) > 0.01) {
                if (Math.abs(vx) > 0.01 || Math.abs(vy) > 0.01) {
                    switch (controlMode) {
                        case ControlMode.Pan:
                            updatePan(vx, -vy);
                            break;
                        case ControlMode.Rotate:
                            updateRotate(vx, vy);
                            break;
                        case ControlMode.Orbit:
                            updateOrbit(vx, vy);
                            break;
                    }
                }
                if (Math.abs(vz) > 0.01) {
                    updateZoom(vz);
                }
                velocity.current.x *= 1 - config.dampingFactor;
                velocity.current.y *= 1 - config.dampingFactor;
                velocity.current.zoom *= 1 - config.dampingFactor;
            }
        }
        animationFrameId.current = requestAnimationFrame(applyDamping);
    }, [config, isInteracting, controlMode, updatePan, updateRotate, updateOrbit, updateZoom]);
    useEffect(() => {
        if (!canvas || !enabled)
            return;
        canvas.addEventListener('pointerdown', handlePointerDown);
        canvas.addEventListener('pointermove', handlePointerMove);
        canvas.addEventListener('pointerup', handlePointerUp);
        canvas.addEventListener('wheel', handleWheel, { passive: false });
        canvas.addEventListener('touchmove', handleTouchMove, { passive: false });
        canvas.addEventListener('touchend', handleTouchEnd);
        return () => {
            canvas.removeEventListener('pointerdown', handlePointerDown);
            canvas.removeEventListener('pointermove', handlePointerMove);
            canvas.removeEventListener('pointerup', handlePointerUp);
            canvas.removeEventListener('wheel', handleWheel);
            canvas.removeEventListener('touchmove', handleTouchMove);
            canvas.removeEventListener('touchend', handleTouchEnd);
        };
    }, [canvas, enabled, handlePointerDown, handlePointerMove, handlePointerUp, handleWheel, handleTouchMove, handleTouchEnd]);
    useEffect(() => {
        if (config.enableDamping) {
            animationFrameId.current = requestAnimationFrame(applyDamping);
            return () => {
                if (animationFrameId.current !== null) {
                    cancelAnimationFrame(animationFrameId.current);
                }
            };
        }
        return undefined;
    }, [config.enableDamping, applyDamping]);
    return {
        controlMode,
        isInteracting,
    };
}
export const ViewportControls = (props) => {
    useViewportControls(props);
    return null;
};
export default ViewportControls;
//# sourceMappingURL=ViewportControls.js.map