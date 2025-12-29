/**
 * Enterprise Status Bar Component
 * Features: Application status bar with indicators, notifications, and contextual information
 */

import React, { ReactNode, CSSProperties } from 'react';
import { useTheme } from './styles/theme';
import { transitionPresets } from './styles/animations';

export interface StatusBarItem {
  id: string;
  content: ReactNode;
  icon?: ReactNode;
  onClick?: () => void;
  tooltip?: string;
  align?: 'left' | 'center' | 'right';
  priority?: number; // Higher priority items are kept visible on smaller screens
}

export interface StatusBarProps {
  /** Status bar items */
  items: StatusBarItem[];
  /** Height of status bar */
  height?: number;
  /** Show separator between items */
  showSeparators?: boolean;
}

export const StatusBar: React.FC<StatusBarProps> = ({
  items,
  height = 28,
  showSeparators = true,
}) => {
  const { theme } = useTheme();

  // Group items by alignment
  const leftItems = items.filter((item) => !item.align || item.align === 'left');
  const centerItems = items.filter((item) => item.align === 'center');
  const rightItems = items.filter((item) => item.align === 'right');

  const containerStyles: CSSProperties = {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    height: `${height}px`,
    backgroundColor: theme.colors.background.secondary,
    borderTop: `1px solid ${theme.colors.border.primary}`,
    fontSize: theme.typography.fontSize.xs,
    color: theme.colors.text.secondary,
    padding: `0 ${theme.spacing[2]}`,
    userSelect: 'none',
    flexShrink: 0,
  };

  const sectionStyles: CSSProperties = {
    display: 'flex',
    alignItems: 'center',
    gap: theme.spacing[1],
    flex: 1,
  };

  const itemStyles = (item: StatusBarItem): CSSProperties => ({
    display: 'flex',
    alignItems: 'center',
    gap: theme.spacing[1],
    padding: `0 ${theme.spacing[2]}`,
    height: '100%',
    cursor: item.onClick ? 'pointer' : 'default',
    transition: transitionPresets.colors,
    position: 'relative',
  });

  const separatorStyles: CSSProperties = {
    width: '1px',
    height: '16px',
    backgroundColor: theme.colors.border.secondary,
    margin: `0 ${theme.spacing[1]}`,
  };

  const renderItem = (item: StatusBarItem, index: number, total: number) => (
    <React.Fragment key={item.id}>
      <div
        style={itemStyles(item)}
        onClick={item.onClick}
        role={item.onClick ? 'button' : undefined}
        aria-label={item.tooltip}
        tabIndex={item.onClick ? 0 : undefined}
        onMouseEnter={(e) => {
          if (item.onClick) {
            e.currentTarget.style.backgroundColor = theme.colors.background.tertiary;
          }
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.backgroundColor = 'transparent';
        }}
        onKeyDown={(e) => {
          if (item.onClick && (e.key === 'Enter' || e.key === ' ')) {
            e.preventDefault();
            item.onClick();
          }
        }}
      >
        {item.icon && (
          <span style={{ display: 'flex', alignItems: 'center', width: '14px', height: '14px' }}>
            {item.icon}
          </span>
        )}
        <span>{item.content}</span>
      </div>

      {showSeparators && index < total - 1 && <div style={separatorStyles} />}
    </React.Fragment>
  );

  return (
    <div style={containerStyles}>
      {/* Left section */}
      <div style={{ ...sectionStyles, justifyContent: 'flex-start' }}>
        {leftItems.map((item, index) => renderItem(item, index, leftItems.length))}
      </div>

      {/* Center section */}
      {centerItems.length > 0 && (
        <div style={{ ...sectionStyles, justifyContent: 'center' }}>
          {centerItems.map((item, index) => renderItem(item, index, centerItems.length))}
        </div>
      )}

      {/* Right section */}
      <div style={{ ...sectionStyles, justifyContent: 'flex-end' }}>
        {rightItems.map((item, index) => renderItem(item, index, rightItems.length))}
      </div>
    </div>
  );
};

StatusBar.displayName = 'StatusBar';

// Common status bar item components for convenience

export interface CoordinateDisplayProps {
  x: number;
  y: number;
  z?: number;
  unit?: string;
}

export const CoordinateDisplay: React.FC<CoordinateDisplayProps> = ({ x, y, z, unit = 'mm' }) => {
  const formatCoord = (value: number) => value.toFixed(2);

  return (
    <>
      X: {formatCoord(x)} {unit} | Y: {formatCoord(y)} {unit}
      {z !== undefined && ` | Z: ${formatCoord(z)} ${unit}`}
    </>
  );
};

export interface ZoomLevelProps {
  zoom: number;
  onZoomChange?: (zoom: number) => void;
}

export const ZoomLevel: React.FC<ZoomLevelProps> = ({ zoom, onZoomChange }) => {
  const handleClick = () => {
    if (onZoomChange) {
      onZoomChange(100); // Reset to 100%
    }
  };

  return (
    <span onClick={handleClick} style={{ cursor: onZoomChange ? 'pointer' : 'default' }}>
      {zoom.toFixed(0)}%
    </span>
  );
};

export interface SelectionCountProps {
  count: number;
  type?: string;
}

export const SelectionCount: React.FC<SelectionCountProps> = ({ count, type = 'objects' }) => {
  if (count === 0) return <>No selection</>;

  return (
    <>
      {count} {type}
      {count !== 1 ? 's' : ''} selected
    </>
  );
};

export interface NotificationBadgeProps {
  count: number;
  severity?: 'info' | 'warning' | 'error';
}

export const NotificationBadge: React.FC<NotificationBadgeProps> = ({ count, severity = 'info' }) => {
  const { theme } = useTheme();

  if (count === 0) return null;

  const colors = {
    info: theme.colors.status.info,
    warning: theme.colors.status.warning,
    error: theme.colors.status.error,
  };

  const badgeStyles: CSSProperties = {
    display: 'inline-flex',
    alignItems: 'center',
    justifyContent: 'center',
    minWidth: '18px',
    height: '18px',
    padding: `0 ${theme.spacing[1]}`,
    backgroundColor: colors[severity],
    color: theme.colors.text.inverse,
    borderRadius: theme.borderRadius.full,
    fontSize: theme.typography.fontSize.xs,
    fontWeight: theme.typography.fontWeight.bold,
  };

  return <span style={badgeStyles}>{count > 99 ? '99+' : count}</span>;
};

export interface LoadingIndicatorProps {
  text?: string;
}

export const LoadingIndicator: React.FC<LoadingIndicatorProps> = ({ text = 'Loading...' }) => {
  const { theme } = useTheme();

  const spinnerStyles: CSSProperties = {
    width: '12px',
    height: '12px',
    border: `2px solid ${theme.colors.border.secondary}`,
    borderTopColor: theme.colors.interactive.primary,
    borderRadius: '50%',
    animation: 'spin 1s linear infinite',
  };

  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: theme.spacing[2] }}>
      <div style={spinnerStyles} />
      <span>{text}</span>
    </div>
  );
};
