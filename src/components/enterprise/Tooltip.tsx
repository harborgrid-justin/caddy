/**
 * Enterprise Tooltip Component
 * Features: Smart positioning, auto-flip, delay, accessibility
 */

import React, { useState, useRef, useEffect, ReactNode, CSSProperties, cloneElement, isValidElement } from 'react';
import { createPortal } from 'react-dom';
import { useTheme } from './styles/theme';
import { animationPresets } from './styles/animations';

export type TooltipPlacement = 'top' | 'bottom' | 'left' | 'right';

export interface TooltipProps {
  /** Tooltip content */
  content: ReactNode;
  /** Placement preference */
  placement?: TooltipPlacement;
  /** Delay before showing (ms) */
  delay?: number;
  /** Disable tooltip */
  disabled?: boolean;
  /** Children (trigger element) */
  children: React.ReactElement;
  /** Max width of tooltip */
  maxWidth?: number;
}

export const Tooltip: React.FC<TooltipProps> = ({
  content,
  placement = 'top',
  delay = 300,
  disabled = false,
  children,
  maxWidth = 300,
}) => {
  const { theme } = useTheme();
  const [isVisible, setIsVisible] = useState(false);
  const [position, setPosition] = useState({ top: 0, left: 0 });
  const [actualPlacement, setActualPlacement] = useState(placement);
  const triggerRef = useRef<HTMLElement>(null);
  const tooltipRef = useRef<HTMLDivElement>(null);
  const timeoutRef = useRef<number>();

  const calculatePosition = () => {
    if (!triggerRef.current || !tooltipRef.current) return;

    const triggerRect = triggerRef.current.getBoundingClientRect();
    const tooltipRect = tooltipRef.current.getBoundingClientRect();
    const spacing = 8;
    const viewport = {
      width: window.innerWidth,
      height: window.innerHeight,
    };

    let top = 0;
    let left = 0;
    let finalPlacement = placement;

    // Calculate position based on placement
    const positions = {
      top: {
        top: triggerRect.top - tooltipRect.height - spacing,
        left: triggerRect.left + triggerRect.width / 2 - tooltipRect.width / 2,
      },
      bottom: {
        top: triggerRect.bottom + spacing,
        left: triggerRect.left + triggerRect.width / 2 - tooltipRect.width / 2,
      },
      left: {
        top: triggerRect.top + triggerRect.height / 2 - tooltipRect.height / 2,
        left: triggerRect.left - tooltipRect.width - spacing,
      },
      right: {
        top: triggerRect.top + triggerRect.height / 2 - tooltipRect.height / 2,
        left: triggerRect.right + spacing,
      },
    };

    let pos = positions[placement];

    // Check if tooltip fits in viewport, flip if necessary
    if (placement === 'top' && pos.top < 0) {
      finalPlacement = 'bottom';
      pos = positions.bottom;
    } else if (placement === 'bottom' && pos.top + tooltipRect.height > viewport.height) {
      finalPlacement = 'top';
      pos = positions.top;
    } else if (placement === 'left' && pos.left < 0) {
      finalPlacement = 'right';
      pos = positions.right;
    } else if (placement === 'right' && pos.left + tooltipRect.width > viewport.width) {
      finalPlacement = 'left';
      pos = positions.left;
    }

    top = pos.top;
    left = pos.left;

    // Ensure tooltip stays within viewport bounds
    if (left < spacing) {
      left = spacing;
    } else if (left + tooltipRect.width > viewport.width - spacing) {
      left = viewport.width - tooltipRect.width - spacing;
    }

    if (top < spacing) {
      top = spacing;
    } else if (top + tooltipRect.height > viewport.height - spacing) {
      top = viewport.height - tooltipRect.height - spacing;
    }

    setPosition({ top, left });
    setActualPlacement(finalPlacement);
  };

  const showTooltip = () => {
    if (disabled) return;

    timeoutRef.current = window.setTimeout(() => {
      setIsVisible(true);
    }, delay);
  };

  const hideTooltip = () => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }
    setIsVisible(false);
  };

  useEffect(() => {
    if (isVisible) {
      calculatePosition();

      // Recalculate on scroll/resize
      const handleUpdate = () => calculatePosition();
      window.addEventListener('scroll', handleUpdate, true);
      window.addEventListener('resize', handleUpdate);

      return () => {
        window.removeEventListener('scroll', handleUpdate, true);
        window.removeEventListener('resize', handleUpdate);
      };
    }
  }, [isVisible]);

  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, []);

  const tooltipStyles: CSSProperties = {
    position: 'fixed',
    top: `${position.top}px`,
    left: `${position.left}px`,
    backgroundColor: theme.colors.background.tertiary,
    color: theme.colors.text.primary,
    padding: `${theme.spacing[2]} ${theme.spacing[3]}`,
    borderRadius: theme.borderRadius.md,
    fontSize: theme.typography.fontSize.sm,
    lineHeight: theme.typography.lineHeight.normal,
    maxWidth: `${maxWidth}px`,
    boxShadow: theme.shadows.lg,
    zIndex: theme.zIndex.tooltip,
    pointerEvents: 'none',
    animation: animationPresets.fadeIn,
    wordWrap: 'break-word',
  };

  const arrowSize = 6;
  const arrowStyles: CSSProperties = {
    position: 'absolute',
    width: 0,
    height: 0,
    borderStyle: 'solid',
  };

  const getArrowStyles = (): CSSProperties => {
    const baseArrow = { ...arrowStyles };

    switch (actualPlacement) {
      case 'top':
        return {
          ...baseArrow,
          bottom: `-${arrowSize}px`,
          left: '50%',
          transform: 'translateX(-50%)',
          borderWidth: `${arrowSize}px ${arrowSize}px 0 ${arrowSize}px`,
          borderColor: `${theme.colors.background.tertiary} transparent transparent transparent`,
        };
      case 'bottom':
        return {
          ...baseArrow,
          top: `-${arrowSize}px`,
          left: '50%',
          transform: 'translateX(-50%)',
          borderWidth: `0 ${arrowSize}px ${arrowSize}px ${arrowSize}px`,
          borderColor: `transparent transparent ${theme.colors.background.tertiary} transparent`,
        };
      case 'left':
        return {
          ...baseArrow,
          right: `-${arrowSize}px`,
          top: '50%',
          transform: 'translateY(-50%)',
          borderWidth: `${arrowSize}px 0 ${arrowSize}px ${arrowSize}px`,
          borderColor: `transparent transparent transparent ${theme.colors.background.tertiary}`,
        };
      case 'right':
        return {
          ...baseArrow,
          left: `-${arrowSize}px`,
          top: '50%',
          transform: 'translateY(-50%)',
          borderWidth: `${arrowSize}px ${arrowSize}px ${arrowSize}px 0`,
          borderColor: `transparent ${theme.colors.background.tertiary} transparent transparent`,
        };
      default:
        return baseArrow;
    }
  };

  // Clone child and attach event handlers
  const trigger = isValidElement(children)
    ? cloneElement(children, {
        ref: triggerRef,
        onMouseEnter: (e: React.MouseEvent) => {
          showTooltip();
          children.props.onMouseEnter?.(e);
        },
        onMouseLeave: (e: React.MouseEvent) => {
          hideTooltip();
          children.props.onMouseLeave?.(e);
        },
        onFocus: (e: React.FocusEvent) => {
          showTooltip();
          children.props.onFocus?.(e);
        },
        onBlur: (e: React.FocusEvent) => {
          hideTooltip();
          children.props.onBlur?.(e);
        },
        'aria-describedby': isVisible ? 'tooltip' : undefined,
      } as any)
    : children;

  return (
    <>
      {trigger}
      {isVisible &&
        createPortal(
          <div ref={tooltipRef} style={tooltipStyles} role="tooltip" id="tooltip">
            {content}
            <div style={getArrowStyles()} />
          </div>,
          document.body
        )}
    </>
  );
};

Tooltip.displayName = 'Tooltip';
