/**
 * CADDY Enterprise UI Component Library
 * v0.2.5
 *
 * Complete enterprise-grade React component library with:
 * - Full TypeScript support
 * - WCAG 2.1 AA accessibility compliance
 * - Dark/Light theme support
 * - Performance optimizations (virtualization, memoization)
 * - CAD-specific components
 */

// Core Components
export { Button } from './Button';
export type { ButtonProps, ButtonVariant, ButtonSize } from './Button';

export { Input, InputMasks } from './Input';
export type { InputProps } from './Input';

export { Select } from './Select';
export type { SelectProps, SelectOption } from './Select';

export { Modal } from './Modal';
export type { ModalProps, ModalSize } from './Modal';

export { Tooltip } from './Tooltip';
export type { TooltipProps, TooltipPlacement } from './Tooltip';

export { Tree } from './Tree';
export type { TreeProps, TreeNode } from './Tree';

export { Table } from './Table';
export type { TableProps, Column, SortDirection } from './Table';

export { Tabs } from './Tabs';
export type { TabsProps, Tab } from './Tabs';

export { ContextMenu } from './ContextMenu';
export type { ContextMenuProps, ContextMenuItem } from './ContextMenu';

export { Splitter } from './Splitter';
export type { SplitterProps } from './Splitter';

export { ColorPicker } from './ColorPicker';
export type { ColorPickerProps } from './ColorPicker';

// CAD-Specific Components
export { PropertyPanel } from './PropertyPanel';
export type {
  PropertyPanelProps,
  Property,
  PropertyCategory,
  PropertyValue,
} from './PropertyPanel';

export { Toolbar } from './Toolbar';
export type { ToolbarProps, ToolbarItem, ToolbarGroup } from './Toolbar';

export {
  StatusBar,
  CoordinateDisplay,
  ZoomLevel,
  SelectionCount,
  NotificationBadge,
  LoadingIndicator,
} from './StatusBar';
export type {
  StatusBarProps,
  StatusBarItem,
  CoordinateDisplayProps,
  ZoomLevelProps,
  SelectionCountProps,
  NotificationBadgeProps,
  LoadingIndicatorProps,
} from './StatusBar';

// Theme System
export { ThemeProvider, useTheme, themes, createStyles, getContrastText } from './styles/theme';
export type { Theme, ThemeMode, ThemeProviderProps } from './styles/theme';

// Design Tokens
export {
  colors,
  spacing,
  typography,
  borderRadius,
  shadows,
  zIndex,
  transitions,
  breakpoints,
} from './styles/tokens';
export type {
  ColorPalette,
  Spacing,
  Typography,
  BorderRadius,
  Shadows,
  ZIndex,
  Transitions,
  Breakpoints,
} from './styles/tokens';

// Animations
export {
  animationPresets,
  transitionPresets,
  springPresets,
  injectKeyframes,
  prefersReducedMotion,
  getAnimationDuration,
  getAnimation,
} from './styles/animations';
export type { SpringConfig } from './styles/animations';
