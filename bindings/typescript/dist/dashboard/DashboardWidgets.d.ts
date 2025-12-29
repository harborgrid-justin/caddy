import React from 'react';
import type { WidgetConfig, WidgetSize } from './types';
export interface WidgetProps {
    config: WidgetConfig;
    children?: React.ReactNode;
    onConfigChange?: (config: WidgetConfig) => void;
    onRemove?: (widgetId: string) => void;
    onResize?: (widgetId: string, size: WidgetSize) => void;
    editable?: boolean;
    className?: string;
}
export declare const Widget: React.FC<WidgetProps>;
export interface WidgetGridProps {
    widgets: WidgetConfig[];
    columns?: number;
    editable?: boolean;
    onWidgetChange?: (widgets: WidgetConfig[]) => void;
    className?: string;
}
export declare const WidgetGrid: React.FC<WidgetGridProps>;
export default Widget;
//# sourceMappingURL=DashboardWidgets.d.ts.map