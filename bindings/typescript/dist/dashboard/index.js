export * from './types';
export { DashboardLayout, GridItem, useDashboard, } from './DashboardLayout';
export { MetricCard, MetricsGrid, } from './DashboardMetrics';
export { Chart, } from './DashboardCharts';
export { Widget, WidgetGrid, } from './DashboardWidgets';
export { ExecutiveOverview, } from './ExecutiveOverview';
export { RealtimeFeed, } from './RealtimeFeed';
export { DashboardFiltersComponent as DashboardFilters, } from './DashboardFilters';
export { DashboardExport, } from './DashboardExport';
export const DASHBOARD_VERSION = '0.4.0';
export const DEFAULT_DASHBOARD_CONFIG = {
    id: 'default-dashboard',
    title: 'Enterprise Dashboard',
    description: 'Real-time enterprise metrics and analytics',
    theme: 'light',
    enableRealtime: true,
    realtimeInterval: 5000,
    defaultTimeRange: '24h',
    refreshInterval: 30,
    autoRefresh: true,
    accessibleMode: true,
};
export const DEFAULT_COLOR_PALETTE_LIGHT = {
    primary: '#1976d2',
    secondary: '#424242',
    success: '#4caf50',
    warning: '#ff9800',
    error: '#f44336',
    info: '#2196f3',
    background: '#f5f5f5',
    surface: '#ffffff',
    text: '#333333',
    textSecondary: '#666666',
    border: '#e0e0e0',
    divider: '#e0e0e0',
};
export const DEFAULT_COLOR_PALETTE_DARK = {
    primary: '#90caf9',
    secondary: '#ce93d8',
    success: '#81c784',
    warning: '#ffb74d',
    error: '#e57373',
    info: '#64b5f6',
    background: '#121212',
    surface: '#1e1e1e',
    text: '#ffffff',
    textSecondary: '#b0b0b0',
    border: '#333333',
    divider: '#333333',
};
export function createDashboardConfig(overrides = {}) {
    return {
        ...DEFAULT_DASHBOARD_CONFIG,
        ...overrides,
    };
}
export function formatMetricValue(value, format) {
    switch (format) {
        case 'currency':
            return new Intl.NumberFormat('en-US', {
                style: 'currency',
                currency: 'USD',
                minimumFractionDigits: 0,
                maximumFractionDigits: 2,
            }).format(value);
        case 'percentage':
            return `${value.toFixed(1)}%`;
        case 'bytes':
            const units = ['B', 'KB', 'MB', 'GB', 'TB'];
            let size = value;
            let unitIndex = 0;
            while (size >= 1024 && unitIndex < units.length - 1) {
                size /= 1024;
                unitIndex++;
            }
            return `${size.toFixed(2)} ${units[unitIndex]}`;
        case 'number':
        default:
            return new Intl.NumberFormat('en-US', {
                maximumFractionDigits: 2,
            }).format(value);
    }
}
export function calculateTrend(current, previous) {
    if (previous === 0) {
        return {
            direction: current > 0 ? 'up' : current < 0 ? 'down' : 'neutral',
            change: current,
            changePercent: 0,
        };
    }
    const change = current - previous;
    const changePercent = (change / Math.abs(previous)) * 100;
    let direction;
    if (Math.abs(changePercent) < 0.1) {
        direction = 'neutral';
    }
    else if (change > 0) {
        direction = 'up';
    }
    else {
        direction = 'down';
    }
    return { direction, change, changePercent };
}
export function getTimeRangeDates(timeRange) {
    const end = new Date();
    const start = new Date();
    switch (timeRange) {
        case '1h':
            start.setHours(start.getHours() - 1);
            break;
        case '24h':
            start.setHours(start.getHours() - 24);
            break;
        case '7d':
            start.setDate(start.getDate() - 7);
            break;
        case '30d':
            start.setDate(start.getDate() - 30);
            break;
        case '90d':
            start.setDate(start.getDate() - 90);
            break;
        case '1y':
            start.setFullYear(start.getFullYear() - 1);
            break;
        default:
            start.setHours(start.getHours() - 24);
    }
    return { start, end };
}
export function aggregateData(data, interval) {
    const grouped = new Map();
    data.forEach((point) => {
        const date = new Date(point.timestamp);
        let key;
        switch (interval) {
            case 'minute':
                key = `${date.getFullYear()}-${date.getMonth()}-${date.getDate()}-${date.getHours()}-${date.getMinutes()}`;
                break;
            case 'hour':
                key = `${date.getFullYear()}-${date.getMonth()}-${date.getDate()}-${date.getHours()}`;
                break;
            case 'day':
                key = `${date.getFullYear()}-${date.getMonth()}-${date.getDate()}`;
                break;
            case 'week':
                const weekStart = new Date(date);
                weekStart.setDate(date.getDate() - date.getDay());
                key = `${weekStart.getFullYear()}-${weekStart.getMonth()}-${weekStart.getDate()}`;
                break;
            case 'month':
                key = `${date.getFullYear()}-${date.getMonth()}`;
                break;
            default:
                key = point.timestamp;
        }
        const existing = grouped.get(key) || { sum: 0, count: 0 };
        grouped.set(key, {
            sum: existing.sum + point.value,
            count: existing.count + 1,
        });
    });
    return Array.from(grouped.entries()).map(([timestamp, { sum, count }]) => ({
        timestamp,
        value: sum / count,
        count,
    }));
}
export function generateMockMetricData(count = 10) {
    const categories = ['Performance', 'Revenue', 'Users', 'System', 'Quality'];
    const icons = ['📊', '💰', '👥', '⚙️', '✨'];
    return Array.from({ length: count }, (_, i) => ({
        id: `metric-${i}`,
        name: `Metric ${i + 1}`,
        value: Math.floor(Math.random() * 10000),
        formattedValue: formatMetricValue(Math.floor(Math.random() * 10000), 'number'),
        previousValue: Math.floor(Math.random() * 10000),
        change: Math.floor(Math.random() * 1000) - 500,
        changePercent: Math.random() * 40 - 20,
        trend: ['up', 'down', 'neutral'][Math.floor(Math.random() * 3)],
        unit: ['', '%', 'ms', 'req/s'][Math.floor(Math.random() * 4)],
        category: categories[i % categories.length],
        icon: icons[i % icons.length],
        progress: Math.floor(Math.random() * 100),
        lastUpdated: new Date().toISOString(),
    }));
}
export function generateMockChartData(type = 'line', points = 10) {
    const labels = Array.from({ length: points }, (_, i) => `Point ${i + 1}`);
    const datasets = [
        {
            label: 'Dataset 1',
            data: Array.from({ length: points }, () => Math.floor(Math.random() * 100)),
            color: '#1976d2',
            backgroundColor: '#1976d2',
            borderColor: '#1976d2',
        },
    ];
    if (type !== 'pie') {
        datasets.push({
            label: 'Dataset 2',
            data: Array.from({ length: points }, () => Math.floor(Math.random() * 100)),
            color: '#f44336',
            backgroundColor: '#f44336',
            borderColor: '#f44336',
        });
    }
    return {
        id: `chart-${Date.now()}`,
        title: `${type.charAt(0).toUpperCase() + type.slice(1)} Chart`,
        type,
        datasets,
        labels,
        options: {
            responsive: true,
            showLegend: true,
            showGrid: true,
            showTooltips: true,
            animated: true,
        },
        lastUpdated: new Date().toISOString(),
    };
}
export default {
    version: DASHBOARD_VERSION,
    defaultConfig: DEFAULT_DASHBOARD_CONFIG,
    colorPalettes: {
        light: DEFAULT_COLOR_PALETTE_LIGHT,
        dark: DEFAULT_COLOR_PALETTE_DARK,
    },
    utils: {
        createDashboardConfig,
        formatMetricValue,
        calculateTrend,
        getTimeRangeDates,
        aggregateData,
        generateMockMetricData,
        generateMockChartData,
    },
};
//# sourceMappingURL=index.js.map