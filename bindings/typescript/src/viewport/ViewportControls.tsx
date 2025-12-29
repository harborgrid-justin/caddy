/**
 * Viewport Controls - Pan, Zoom, Rotate, Orbit Controls
 *
 * Enterprise-grade viewport interaction controls for CADDY v0.2.5
 * Provides smooth, responsive camera manipulation with customizable behavior.
 *
 * @module ViewportControls
 */

import React, { useRef, useCallback, useEffect, useState } from 'react';
import { CameraState, CameraMode, ViewportId } from './ViewportManager';

/**
 * Control mode
 */
export enum ControlMode {
  Pan = 'pan',
  Rotate = 'rotate',
  Orbit = 'orbit',
  Zoom = 'zoom',
  None = 'none',
}

/**
 * Mouse button enumeration
 */
export enum MouseButton {
  Left = 0,
  Middle = 1,
  Right = 2,
}

/**
 * Viewport controls configuration
 */
export interface ViewportControlsConfig {
  /** Enable pan with middle mouse or Shift+Left */
  enablePan: boolean;

  /** Enable rotation with right mouse or Alt+Left */
  enableRotate: boolean;

  /** Enable orbit mode */
  enableOrbit: boolean;

  /** Enable zoom with mouse wheel */
  enableZoom: boolean;

  /** Pan speed multiplier */
  panSpeed: number;

  /** Rotate speed multiplier */
  rotateSpeed: number;

  /** Zoom speed multiplier */
  zoomSpeed: number;

  /** Damping/inertia factor (0 = no damping, 1 = instant stop) */
  dampingFactor: number;

  /** Enable damping */
  enableDamping: boolean;

  /** Minimum zoom distance */
  minZoom: number;

  /** Maximum zoom distance */
  maxZoom: number;

  /** Auto-rotate speed (0 = disabled) */
  autoRotateSpeed: number;

  /** Enable touch controls */
  enableTouch: boolean;

  /** Touch zoom sensitivity */
  touchZoomSpeed: number;

  /** Touch rotate sensitivity */
  touchRotateSpeed: number;
}

/**
 * Default controls configuration
 */
export const DEFAULT_CONTROLS_CONFIG: ViewportControlsConfig = {
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

/**
 * Viewport controls props
 */
export interface ViewportControlsProps {
  /** Viewport identifier */
  viewportId: ViewportId;

  /** Camera state */
  camera: CameraState;

  /** Canvas element to attach controls to */
  canvas: HTMLCanvasElement | null;

  /** Configuration */
  config?: Partial<ViewportControlsConfig>;

  /** Camera update callback */
  onCameraUpdate: (camera: Partial<CameraState>) => void;

  /** Control mode change callback */
  onControlModeChange?: (mode: ControlMode) => void;

  /** Enable controls */
  enabled?: boolean;
}

/**
 * Pointer state for tracking interactions
 */
interface PointerState {
  id: number;
  x: number;
  y: number;
  deltaX: number;
  deltaY: number;
  button: number;
}

/**
 * Viewport Controls Hook
 *
 * Provides camera manipulation through mouse, keyboard, and touch inputs.
 */
export function useViewportControls({
  viewportId,
  camera,
  canvas,
  config: configProp,
  onCameraUpdate,
  onControlModeChange,
  enabled = true,
}: ViewportControlsProps): {
  controlMode: ControlMode;
  isInteracting: boolean;
} {
  const config = { ...DEFAULT_CONTROLS_CONFIG, ...configProp };

  const [controlMode, setControlMode] = useState<ControlMode>(ControlMode.None);
  const [isInteracting, setIsInteracting] = useState(false);

  const pointers = useRef<Map<number, PointerState>>(new Map());
  const lastTouchDistance = useRef<number>(0);
  const velocity = useRef({ x: 0, y: 0, zoom: 0 });
  const animationFrameId = useRef<number | null>(null);

  /**
   * Get current control mode based on input state
   */
  const getControlMode = useCallback((
    button: number,
    shiftKey: boolean,
    altKey: boolean,
    ctrlKey: boolean,
  ): ControlMode => {
    if (!enabled) return ControlMode.None;

    // Middle mouse = Pan
    if (button === MouseButton.Middle && config.enablePan) {
      return ControlMode.Pan;
    }

    // Shift + Left = Pan
    if (button === MouseButton.Left && shiftKey && config.enablePan) {
      return ControlMode.Pan;
    }

    // Alt/Option + Left = Rotate
    if (button === MouseButton.Left && altKey && config.enableRotate) {
      return ControlMode.Rotate;
    }

    // Right mouse = Orbit (or Rotate for orthographic)
    if (button === MouseButton.Right && config.enableOrbit) {
      return camera.mode === CameraMode.Perspective
        ? ControlMode.Orbit
        : ControlMode.Rotate;
    }

    // Left mouse default = Orbit (perspective) or None
    if (button === MouseButton.Left && !shiftKey && !altKey && !ctrlKey) {
      if (camera.mode === CameraMode.Perspective && config.enableOrbit) {
        return ControlMode.Orbit;
      }
    }

    return ControlMode.None;
  }, [enabled, config, camera.mode]);

  /**
   * Update camera position (pan)
   */
  const updatePan = useCallback((deltaX: number, deltaY: number) => {
    if (!config.enablePan) return;

    const panSpeed = config.panSpeed * 0.002;

    // Calculate pan vector based on camera orientation
    const [px, py, pz] = camera.position;
    const [tx, ty, tz] = camera.target;
    const [ux, uy, uz] = camera.up;

    // Forward vector
    const fx = tx - px;
    const fy = ty - py;
    const fz = tz - pz;
    const flen = Math.sqrt(fx * fx + fy * fy + fz * fz);
    const fnx = fx / flen;
    const fny = fy / flen;
    const fnz = fz / flen;

    // Right vector (cross product of forward and up)
    const rx = fny * uz - fnz * uy;
    const ry = fnz * ux - fnx * uz;
    const rz = fnx * uy - fny * ux;
    const rlen = Math.sqrt(rx * rx + ry * ry + rz * rz);
    const rnx = rx / rlen;
    const rny = ry / rlen;
    const rnz = rz / rlen;

    // True up vector (cross product of right and forward)
    const tux = rny * fnz - rnz * fny;
    const tuy = rnz * fnx - rnx * fnz;
    const tuz = rnx * fny - rny * fnx;

    // Apply pan
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

  /**
   * Update camera rotation (rotate around position)
   */
  const updateRotate = useCallback((deltaX: number, deltaY: number) => {
    if (!config.enableRotate) return;

    const rotateSpeed = config.rotateSpeed * 0.005;

    const [px, py, pz] = camera.position;
    const [tx, ty, tz] = camera.target;

    // Calculate direction from position to target
    let dx = tx - px;
    let dy = ty - py;
    let dz = tz - pz;
    const distance = Math.sqrt(dx * dx + dy * dy + dz * dz);

    // Normalize
    dx /= distance;
    dy /= distance;
    dz /= distance;

    // Apply yaw (horizontal rotation)
    const yaw = -deltaX * rotateSpeed;
    const cosYaw = Math.cos(yaw);
    const sinYaw = Math.sin(yaw);

    const newDx = dx * cosYaw - dz * sinYaw;
    const newDz = dx * sinYaw + dz * cosYaw;

    dx = newDx;
    dz = newDz;

    // Apply pitch (vertical rotation) - clamped to avoid gimbal lock
    const pitch = -deltaY * rotateSpeed;
    const cosPitch = Math.cos(pitch);
    const sinPitch = Math.sin(pitch);

    const newDy = dy * cosPitch - dz * sinPitch;
    const newDz2 = dy * sinPitch + dz * cosPitch;

    dy = newDy;
    dz = newDz2;

    // Update target
    onCameraUpdate({
      target: [
        px + dx * distance,
        py + dy * distance,
        pz + dz * distance,
      ],
    });
  }, [config, camera, onCameraUpdate]);

  /**
   * Update camera orbit (rotate around target)
   */
  const updateOrbit = useCallback((deltaX: number, deltaY: number) => {
    if (!config.enableOrbit) return;

    const rotateSpeed = config.rotateSpeed * 0.005;

    const [px, py, pz] = camera.position;
    const [tx, ty, tz] = camera.target;

    // Calculate offset from target
    let dx = px - tx;
    let dy = py - ty;
    let dz = pz - tz;
    const distance = Math.sqrt(dx * dx + dy * dy + dz * dz);

    // Convert to spherical coordinates
    const radius = distance;
    let theta = Math.atan2(dz, dx); // Azimuth angle
    let phi = Math.acos(dy / radius); // Polar angle

    // Apply rotation
    theta -= deltaX * rotateSpeed;
    phi -= deltaY * rotateSpeed;

    // Clamp phi to avoid gimbal lock
    const epsilon = 0.01;
    phi = Math.max(epsilon, Math.min(Math.PI - epsilon, phi));

    // Convert back to Cartesian
    const newDx = radius * Math.sin(phi) * Math.cos(theta);
    const newDy = radius * Math.cos(phi);
    const newDz = radius * Math.sin(phi) * Math.sin(theta);

    onCameraUpdate({
      position: [tx + newDx, ty + newDy, tz + newDz],
    });
  }, [config, camera, onCameraUpdate]);

  /**
   * Update camera zoom
   */
  const updateZoom = useCallback((delta: number) => {
    if (!config.enableZoom) return;

    const zoomSpeed = config.zoomSpeed;

    if (camera.mode === CameraMode.Perspective) {
      // Move camera closer/further from target
      const [px, py, pz] = camera.position;
      const [tx, ty, tz] = camera.target;

      let dx = tx - px;
      let dy = ty - py;
      let dz = tz - pz;
      const distance = Math.sqrt(dx * dx + dy * dy + dz * dz);

      // Normalize direction
      dx /= distance;
      dy /= distance;
      dz /= distance;

      // Calculate new distance
      const newDistance = Math.max(
        config.minZoom,
        Math.min(config.maxZoom, distance - delta * zoomSpeed)
      );

      onCameraUpdate({
        position: [
          tx - dx * newDistance,
          ty - dy * newDistance,
          tz - dz * newDistance,
        ],
      });
    } else {
      // Update zoom factor for orthographic/isometric
      const currentZoom = camera.zoom || 1.0;
      const newZoom = Math.max(
        config.minZoom,
        Math.min(config.maxZoom, currentZoom + delta * zoomSpeed * 0.1)
      );

      onCameraUpdate({
        zoom: newZoom,
      });
    }
  }, [config, camera, onCameraUpdate]);

  /**
   * Handle pointer down
   */
  const handlePointerDown = useCallback((e: PointerEvent) => {
    if (!enabled || !canvas) return;

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

  /**
   * Handle pointer move
   */
  const handlePointerMove = useCallback((e: PointerEvent) => {
    if (!enabled || !canvas) return;

    const pointer = pointers.current.get(e.pointerId);
    if (!pointer) return;

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

    // Apply camera updates based on control mode
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

    // Update velocity for damping
    if (config.enableDamping) {
      velocity.current.x = deltaX;
      velocity.current.y = deltaY;
    }
  }, [enabled, canvas, controlMode, config.enableDamping, updatePan, updateRotate, updateOrbit]);

  /**
   * Handle pointer up
   */
  const handlePointerUp = useCallback((e: PointerEvent) => {
    if (!enabled || !canvas) return;

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

  /**
   * Handle wheel
   */
  const handleWheel = useCallback((e: WheelEvent) => {
    if (!enabled || !config.enableZoom) return;

    e.preventDefault();

    const delta = e.deltaY * 0.01;
    updateZoom(delta);

    // Update velocity for damping
    if (config.enableDamping) {
      velocity.current.zoom = delta;
    }
  }, [enabled, config.enableZoom, config.enableDamping, updateZoom]);

  /**
   * Handle touch for pinch-to-zoom
   */
  const handleTouchMove = useCallback((e: TouchEvent) => {
    if (!enabled || !config.enableTouch || e.touches.length !== 2) return;

    e.preventDefault();

    const touch1 = e.touches[0];
    const touch2 = e.touches[1];

    const distance = Math.sqrt(
      Math.pow(touch2.clientX - touch1.clientX, 2) +
      Math.pow(touch2.clientY - touch1.clientY, 2)
    );

    if (lastTouchDistance.current > 0) {
      const delta = (lastTouchDistance.current - distance) * config.touchZoomSpeed;
      updateZoom(delta);
    }

    lastTouchDistance.current = distance;
  }, [enabled, config.enableTouch, config.touchZoomSpeed, updateZoom]);

  const handleTouchEnd = useCallback(() => {
    lastTouchDistance.current = 0;
  }, []);

  /**
   * Apply damping
   */
  const applyDamping = useCallback(() => {
    if (!config.enableDamping || !isInteracting) {
      const vx = velocity.current.x;
      const vy = velocity.current.y;
      const vz = velocity.current.zoom;

      if (Math.abs(vx) > 0.01 || Math.abs(vy) > 0.01 || Math.abs(vz) > 0.01) {
        // Apply velocity
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

        // Dampen velocity
        velocity.current.x *= 1 - config.dampingFactor;
        velocity.current.y *= 1 - config.dampingFactor;
        velocity.current.zoom *= 1 - config.dampingFactor;
      }
    }

    animationFrameId.current = requestAnimationFrame(applyDamping);
  }, [config, isInteracting, controlMode, updatePan, updateRotate, updateOrbit, updateZoom]);

  /**
   * Attach event listeners
   */
  useEffect(() => {
    if (!canvas || !enabled) return;

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

  /**
   * Start damping animation loop
   */
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

/**
 * Viewport Controls Component
 *
 * React component wrapper for viewport controls.
 */
export const ViewportControls: React.FC<ViewportControlsProps> = (props) => {
  useViewportControls(props);
  return null;
};

export default ViewportControls;
