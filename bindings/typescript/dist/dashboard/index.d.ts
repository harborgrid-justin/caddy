export * from './types';
export { DashboardLayout, GridItem, useDashboard, type DashboardLayoutProps, type GridItemProps, } from './DashboardLayout';
export { MetricCard, MetricsGrid, type MetricCardProps, type MetricsGridProps, } from './DashboardMetrics';
export { Chart, type ChartProps, } from './DashboardCharts';
export { Widget, WidgetGrid, type WidgetProps, type WidgetGridProps, } from './DashboardWidgets';
export { ExecutiveOverview, type ExecutiveOverviewProps, } from './ExecutiveOverview';
export { RealtimeFeed, type RealtimeFeedProps, } from './RealtimeFeed';
export { DashboardFiltersComponent as DashboardFilters, type DashboardFiltersProps, } from './DashboardFilters';
export { DashboardExport, type DashboardExportProps, } from './DashboardExport';
export declare const DASHBOARD_VERSION = "0.4.0";
export declare const DEFAULT_DASHBOARD_CONFIG: {
    id: string;
    title: string;
    description: string;
    theme: "light";
    enableRealtime: boolean;
    realtimeInterval: number;
    defaultTimeRange: "24h";
    refreshInterval: number;
    autoRefresh: boolean;
    accessibleMode: boolean;
};
export declare const DEFAULT_COLOR_PALETTE_LIGHT: {
    primary: string;
    secondary: string;
    success: string;
    warning: string;
    error: string;
    info: string;
    background: string;
    surface: string;
    text: string;
    textSecondary: string;
    border: string;
    divider: string;
};
export declare const DEFAULT_COLOR_PALETTE_DARK: {
    primary: string;
    secondary: string;
    success: string;
    warning: string;
    error: string;
    info: string;
    background: string;
    surface: string;
    text: string;
    textSecondary: string;
    border: string;
    divider: string;
};
export declare function createDashboardConfig(overrides?: Partial<typeof DEFAULT_DASHBOARD_CONFIG>): {
    id: string;
    title: string;
    description: string;
    theme: "light";
    enableRealtime: boolean;
    realtimeInterval: number;
    defaultTimeRange: "24h";
    refreshInterval: number;
    autoRefresh: boolean;
    accessibleMode: boolean;
};
export declare function formatMetricValue(value: number, format?: 'number' | 'currency' | 'percentage' | 'bytes'): string;
export declare function calculateTrend(current: number, previous: number): {
    direction: 'up' | 'down' | 'neutral';
    change: number;
    changePercent: number;
};
export declare function getTimeRangeDates(timeRange: string): {
    start: Date;
    end: Date;
};
export declare function aggregateData(data: Array<{
    timestamp: string;
    value: number;
}>, interval: 'minute' | 'hour' | 'day' | 'week' | 'month'): Array<{
    timestamp: string;
    value: number;
    count: number;
}>;
export declare function generateMockMetricData(count?: number): {
    id: string;
    name: string;
    value: number;
    formattedValue: string;
    previousValue: number;
    change: number;
    changePercent: number;
    trend: any;
    unit: string;
    category: string;
    icon: string;
    progress: number;
    lastUpdated: string;
}[];
export declare function generateMockChartData(type?: 'line' | 'bar' | 'pie', points?: number): {
    id: string;
    title: string;
    type: "line" | "bar" | "pie";
    datasets: {
        label: string;
        data: number[];
        color: string;
        backgroundColor: string;
        borderColor: string;
    }[];
    labels: string[];
    options: {
        responsive: boolean;
        showLegend: boolean;
        showGrid: boolean;
        showTooltips: boolean;
        animated: boolean;
    };
    lastUpdated: string;
};
declare const _default: {
    version: string;
    defaultConfig: {
        id: string;
        title: string;
        description: string;
        theme: "light";
        enableRealtime: boolean;
        realtimeInterval: number;
        defaultTimeRange: "24h";
        refreshInterval: number;
        autoRefresh: boolean;
        accessibleMode: boolean;
    };
    colorPalettes: {
        light: {
            primary: string;
            secondary: string;
            success: string;
            warning: string;
            error: string;
            info: string;
            background: string;
            surface: string;
            text: string;
            textSecondary: string;
            border: string;
            divider: string;
        };
        dark: {
            primary: string;
            secondary: string;
            success: string;
            warning: string;
            error: string;
            info: string;
            background: string;
            surface: string;
            text: string;
            textSecondary: string;
            border: string;
            divider: string;
        };
    };
    utils: {
        createDashboardConfig: typeof createDashboardConfig;
        formatMetricValue: typeof formatMetricValue;
        calculateTrend: typeof calculateTrend;
        getTimeRangeDates: typeof getTimeRangeDates;
        aggregateData: typeof aggregateData;
        generateMockMetricData: typeof generateMockMetricData;
        generateMockChartData: typeof generateMockChartData;
    };
};
export default _default;
//# sourceMappingURL=index.d.ts.map