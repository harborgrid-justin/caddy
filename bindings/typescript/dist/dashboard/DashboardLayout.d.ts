import React from 'react';
import type { DashboardConfig, DashboardLayout as LayoutConfig, WidgetConfig, ThemeMode, AccessibilityConfig, ErrorState } from './types';
interface DashboardContextValue {
    config: DashboardConfig;
    theme: ThemeMode;
    setTheme: (theme: ThemeMode) => void;
    filters: any;
    setFilters: (filters: any) => void;
    isLoading: boolean;
    error: ErrorState | null;
    refreshData: () => void;
    accessibility: AccessibilityConfig;
}
export declare const useDashboard: () => DashboardContextValue;
export interface DashboardLayoutProps {
    config: DashboardConfig;
    layout: LayoutConfig;
    children?: React.ReactNode;
    header?: React.ReactNode;
    sidebar?: React.ReactNode;
    footer?: React.ReactNode;
    onLayoutChange?: (layout: LayoutConfig) => void;
    onThemeChange?: (theme: ThemeMode) => void;
    className?: string;
    accessibility?: AccessibilityConfig;
}
export declare const DashboardLayout: React.FC<DashboardLayoutProps>;
export interface GridItemProps {
    widget: WidgetConfig;
    children: React.ReactNode;
    className?: string;
    style?: React.CSSProperties;
}
export declare const GridItem: React.FC<GridItemProps>;
export default DashboardLayout;
//# sourceMappingURL=DashboardLayout.d.ts.map