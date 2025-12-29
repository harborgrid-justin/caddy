/**
 * Enterprise Splitter Component
 * Features: Resizable split panes, horizontal/vertical, min/max sizes, keyboard support
 */

import React, { useState, useRef, useCallback, CSSProperties, ReactNode } from 'react';
import { useTheme } from './styles/theme';
import { transitionPresets } from './styles/animations';

export interface SplitterProps {
  /** First pane content */
  children: [ReactNode, ReactNode];
  /** Initial split position (0-1) */
  initialSplit?: number;
  /** Orientation */
  orientation?: 'horizontal' | 'vertical';
  /** Minimum size of first pane (px) */
  minSize?: number;
  /** Maximum size of first pane (px) */
  maxSize?: number;
  /** Splitter size (px) */
  splitterSize?: number;
  /** Disable resizing */
  disabled?: boolean;
  /** Split change handler */
  onSplitChange?: (split: number) => void;
}

export const Splitter: React.FC<SplitterProps> = ({
  children,
  initialSplit = 0.5,
  orientation = 'horizontal',
  minSize = 100,
  maxSize,
  splitterSize = 4,
  disabled = false,
  onSplitChange,
}) => {
  const { theme } = useTheme();
  const [split, setSplit] = useState(initialSplit);
  const [isDragging, setIsDragging] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (disabled) return;
    e.preventDefault();
    setIsDragging(true);
  }, [disabled]);

  const handleMouseMove = useCallback(
    (e: MouseEvent) => {
      if (!isDragging || !containerRef.current) return;

      const rect = containerRef.current.getBoundingClientRect();
      let newSplit: number;

      if (orientation === 'horizontal') {
        const x = e.clientX - rect.left;
        newSplit = x / rect.width;
      } else {
        const y = e.clientY - rect.top;
        newSplit = y / rect.height;
      }

      // Apply min/max constraints
      const containerSize = orientation === 'horizontal' ? rect.width : rect.height;
      const minSplit = minSize / containerSize;
      const maxSplit = maxSize ? maxSize / containerSize : 1 - minSize / containerSize;

      newSplit = Math.max(minSplit, Math.min(maxSplit, newSplit));

      setSplit(newSplit);
      onSplitChange?.(newSplit);
    },
    [isDragging, orientation, minSize, maxSize, onSplitChange]
  );

  const handleMouseUp = useCallback(() => {
    setIsDragging(false);
  }, []);

  // Mouse event handlers
  React.useEffect(() => {
    if (isDragging) {
      document.addEventListener('mousemove', handleMouseMove);
      document.addEventListener('mouseup', handleMouseUp);
      document.body.style.cursor = orientation === 'horizontal' ? 'col-resize' : 'row-resize';
      document.body.style.userSelect = 'none';

      return () => {
        document.removeEventListener('mousemove', handleMouseMove);
        document.removeEventListener('mouseup', handleMouseUp);
        document.body.style.cursor = '';
        document.body.style.userSelect = '';
      };
    }
  }, [isDragging, handleMouseMove, handleMouseUp, orientation]);

  // Keyboard support
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (disabled) return;

    const step = 0.05;
    let newSplit = split;

    switch (e.key) {
      case 'ArrowLeft':
      case 'ArrowUp':
        e.preventDefault();
        newSplit = Math.max(minSize / (containerRef.current?.getBoundingClientRect().width || 1), split - step);
        break;
      case 'ArrowRight':
      case 'ArrowDown':
        e.preventDefault();
        newSplit = Math.min(1 - minSize / (containerRef.current?.getBoundingClientRect().width || 1), split + step);
        break;
      case 'Home':
        e.preventDefault();
        newSplit = minSize / (containerRef.current?.getBoundingClientRect().width || 1);
        break;
      case 'End':
        e.preventDefault();
        newSplit = 1 - minSize / (containerRef.current?.getBoundingClientRect().width || 1);
        break;
      default:
        return;
    }

    setSplit(newSplit);
    onSplitChange?.(newSplit);
  };

  const containerStyles: CSSProperties = {
    display: 'flex',
    flexDirection: orientation === 'horizontal' ? 'row' : 'column',
    width: '100%',
    height: '100%',
    position: 'relative',
  };

  const pane1Styles: CSSProperties = {
    flexShrink: 0,
    flexBasis: orientation === 'horizontal' ? `${split * 100}%` : `${split * 100}%`,
    overflow: 'auto',
  };

  const pane2Styles: CSSProperties = {
    flex: 1,
    overflow: 'auto',
  };

  const splitterStyles: CSSProperties = {
    flexShrink: 0,
    backgroundColor: theme.colors.border.primary,
    cursor: disabled
      ? 'default'
      : orientation === 'horizontal'
      ? 'col-resize'
      : 'row-resize',
    position: 'relative',
    zIndex: 1,
    width: orientation === 'horizontal' ? `${splitterSize}px` : '100%',
    height: orientation === 'vertical' ? `${splitterSize}px` : '100%',
    transition: isDragging ? 'none' : transitionPresets.colors,
  };

  const handleStyles: CSSProperties = {
    position: 'absolute',
    top: '50%',
    left: '50%',
    transform: 'translate(-50%, -50%)',
    width: orientation === 'horizontal' ? '4px' : '40px',
    height: orientation === 'horizontal' ? '40px' : '4px',
    backgroundColor: theme.colors.border.secondary,
    borderRadius: theme.borderRadius.full,
    opacity: isDragging ? 1 : 0,
    transition: transitionPresets.opacity,
  };

  return (
    <div ref={containerRef} style={containerStyles}>
      <div style={pane1Styles}>{children[0]}</div>

      <div
        role="separator"
        aria-orientation={orientation}
        aria-valuenow={Math.round(split * 100)}
        aria-valuemin={0}
        aria-valuemax={100}
        tabIndex={disabled ? -1 : 0}
        style={splitterStyles}
        onMouseDown={handleMouseDown}
        onKeyDown={handleKeyDown}
        onMouseEnter={(e) => {
          if (!disabled) {
            e.currentTarget.style.backgroundColor = theme.colors.interactive.primary;
          }
        }}
        onMouseLeave={(e) => {
          if (!isDragging) {
            e.currentTarget.style.backgroundColor = theme.colors.border.primary;
          }
        }}
        onFocus={(e) => {
          e.currentTarget.style.outline = `2px solid ${theme.colors.border.focus}`;
          e.currentTarget.style.outlineOffset = '2px';
        }}
        onBlur={(e) => {
          e.currentTarget.style.outline = 'none';
        }}
      >
        <div style={handleStyles} />
      </div>

      <div style={pane2Styles}>{children[1]}</div>
    </div>
  );
};

Splitter.displayName = 'Splitter';
