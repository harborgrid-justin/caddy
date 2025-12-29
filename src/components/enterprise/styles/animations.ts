/**
 * Shared Animation Definitions for CADDY Enterprise UI
 * Provides consistent, performant animations across all components
 */

import { keyframes } from 'react';
import { transitions } from './tokens';

// Keyframe animations
export const fadeIn = `
  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
`;

export const fadeOut = `
  @keyframes fadeOut {
    from {
      opacity: 1;
    }
    to {
      opacity: 0;
    }
  }
`;

export const slideInUp = `
  @keyframes slideInUp {
    from {
      transform: translateY(20px);
      opacity: 0;
    }
    to {
      transform: translateY(0);
      opacity: 1;
    }
  }
`;

export const slideInDown = `
  @keyframes slideInDown {
    from {
      transform: translateY(-20px);
      opacity: 0;
    }
    to {
      transform: translateY(0);
      opacity: 1;
    }
  }
`;

export const slideInLeft = `
  @keyframes slideInLeft {
    from {
      transform: translateX(-20px);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }
`;

export const slideInRight = `
  @keyframes slideInRight {
    from {
      transform: translateX(20px);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }
`;

export const scaleIn = `
  @keyframes scaleIn {
    from {
      transform: scale(0.9);
      opacity: 0;
    }
    to {
      transform: scale(1);
      opacity: 1;
    }
  }
`;

export const scaleOut = `
  @keyframes scaleOut {
    from {
      transform: scale(1);
      opacity: 1;
    }
    to {
      transform: scale(0.9);
      opacity: 0;
    }
  }
`;

export const spin = `
  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
`;

export const pulse = `
  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }
`;

export const bounce = `
  @keyframes bounce {
    0%, 100% {
      transform: translateY(0);
    }
    50% {
      transform: translateY(-10px);
    }
  }
`;

export const shake = `
  @keyframes shake {
    0%, 100% {
      transform: translateX(0);
    }
    10%, 30%, 50%, 70%, 90% {
      transform: translateX(-4px);
    }
    20%, 40%, 60%, 80% {
      transform: translateX(4px);
    }
  }
`;

export const ripple = `
  @keyframes ripple {
    0% {
      transform: scale(0);
      opacity: 0.5;
    }
    100% {
      transform: scale(4);
      opacity: 0;
    }
  }
`;

// Animation presets for common use cases
export const animationPresets = {
  fadeIn: `fadeIn ${transitions.duration.normal} ${transitions.easing.easeOut}`,
  fadeOut: `fadeOut ${transitions.duration.normal} ${transitions.easing.easeIn}`,
  slideInUp: `slideInUp ${transitions.duration.normal} ${transitions.easing.easeOut}`,
  slideInDown: `slideInDown ${transitions.duration.normal} ${transitions.easing.easeOut}`,
  slideInLeft: `slideInLeft ${transitions.duration.normal} ${transitions.easing.easeOut}`,
  slideInRight: `slideInRight ${transitions.duration.normal} ${transitions.easing.easeOut}`,
  scaleIn: `scaleIn ${transitions.duration.fast} ${transitions.easing.easeOut}`,
  scaleOut: `scaleOut ${transitions.duration.fast} ${transitions.easing.easeIn}`,
  spin: `spin 1s ${transitions.easing.linear} infinite`,
  pulse: `pulse 2s ${transitions.easing.easeInOut} infinite`,
  bounce: `bounce 1s ${transitions.easing.easeInOut} infinite`,
  shake: `shake 0.5s ${transitions.easing.easeInOut}`,
  ripple: `ripple 0.6s ${transitions.easing.easeOut}`,
} as const;

// Transition utilities
export const transitionPresets = {
  fast: `all ${transitions.duration.fast} ${transitions.easing.easeInOut}`,
  normal: `all ${transitions.duration.normal} ${transitions.easing.easeInOut}`,
  slow: `all ${transitions.duration.slow} ${transitions.easing.easeInOut}`,
  colors: `background-color ${transitions.duration.fast} ${transitions.easing.easeInOut}, color ${transitions.duration.fast} ${transitions.easing.easeInOut}, border-color ${transitions.duration.fast} ${transitions.easing.easeInOut}`,
  transform: `transform ${transitions.duration.normal} ${transitions.easing.easeOut}`,
  opacity: `opacity ${transitions.duration.fast} ${transitions.easing.easeInOut}`,
} as const;

// Export all keyframes as a single string for injection
export const allKeyframes = `
  ${fadeIn}
  ${fadeOut}
  ${slideInUp}
  ${slideInDown}
  ${slideInLeft}
  ${slideInRight}
  ${scaleIn}
  ${scaleOut}
  ${spin}
  ${pulse}
  ${bounce}
  ${shake}
  ${ripple}
`;

// Helper function to inject keyframes into document
let keyframesInjected = false;

export function injectKeyframes(): void {
  if (keyframesInjected || typeof document === 'undefined') {
    return;
  }

  const style = document.createElement('style');
  style.textContent = allKeyframes;
  document.head.appendChild(style);
  keyframesInjected = true;
}

// Spring animation configuration for smooth, natural motion
export interface SpringConfig {
  tension: number;
  friction: number;
  mass: number;
}

export const springPresets = {
  default: { tension: 170, friction: 26, mass: 1 },
  gentle: { tension: 120, friction: 14, mass: 1 },
  wobbly: { tension: 180, friction: 12, mass: 1 },
  stiff: { tension: 210, friction: 20, mass: 1 },
  slow: { tension: 280, friction: 60, mass: 1 },
  molasses: { tension: 280, friction: 120, mass: 1 },
} as const;

// Reduced motion support for accessibility
export function prefersReducedMotion(): boolean {
  if (typeof window === 'undefined') return false;
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

export function getAnimationDuration(duration: string): string {
  return prefersReducedMotion() ? '0.01ms' : duration;
}

export function getAnimation(animation: string): string {
  return prefersReducedMotion() ? 'none' : animation;
}
