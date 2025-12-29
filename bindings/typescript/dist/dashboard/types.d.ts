export type ThemeMode = 'light' | 'dark' | 'auto';
export type ChartType = 'line' | 'bar' | 'pie' | 'area' | 'scatter' | 'radar' | 'heatmap';
export type TimeRange = '1h' | '24h' | '7d' | '30d' | '90d' | '1y' | 'custom';
export type AggregationInterval = '1m' | '5m' | '15m' | '1h' | '6h' | '1d' | '1w' | '1mo';
export type ExportFormat = 'pdf' | 'excel' | 'csv' | 'json';
export type WidgetSize = 'small' | 'medium' | 'large' | 'xlarge' | 'full';
export type TrendDirection = 'up' | 'down' | 'neutral';
export type AlertSeverity = 'info' | 'warning' | 'error' | 'critical';
export interface DashboardConfig {
    id: string;
    title: string;
    description?: string;
    theme: ThemeMode;
    enableRealtime?: boolean;
    realtimeInterval?: number;
    wsUrl?: string;
    authToken?: string;
    defaultTimeRange?: TimeRange;
    refreshInterval?: number;
    autoRefresh?: boolean;
    accessibleMode?: boolean;
    colorPalette?: ColorPalette;
}
export interface ColorPalette {
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
}
export interface DashboardLayout {
    id: string;
    columns: {
        xs: number;
        sm: number;
        md: number;
        lg: number;
        xl: number;
    };
    gap: number;
    widgets: WidgetConfig[];
    className?: string;
}
export interface WidgetConfig {
    id: string;
    type: 'metric' | 'chart' | 'table' | 'feed' | 'custom';
    title: string;
    description?: string;
    size: WidgetSize;
    position: {
        row: number;
        col: number;
    };
    span: {
        rows: number;
        cols: number;
    };
    dataSource: DataSourceConfig;
    refreshInterval?: number;
    autoRefresh?: boolean;
    style?: React.CSSProperties;
    options?: Record<string, any>;
}
export interface DataSourceConfig {
    type: 'api' | 'websocket' | 'static';
    url?: string;
    method?: 'GET' | 'POST' | 'PUT' | 'DELETE';
    headers?: Record<string, string>;
    body?: any;
    channel?: string;
    data?: any;
    transform?: string;
}
export interface MetricData {
    id: string;
    name: string;
    value: number;
    formattedValue: string;
    previousValue?: number;
    change?: number;
    changePercent?: number;
    trend: TrendDirection;
    unit: string;
    category: string;
    icon?: string;
    color?: string;
    target?: number;
    progress?: number;
    history?: DataPoint[];
    lastUpdated: string;
    metadata?: Record<string, any>;
}
export interface ChartData {
    id: string;
    title: string;
    type: ChartType;
    datasets: ChartDataset[];
    labels?: string[];
    options: ChartOptions;
    lastUpdated: string;
}
export interface ChartDataset {
    label: string;
    data: (number | DataPoint)[];
    color?: string;
    backgroundColor?: string;
    borderColor?: string;
    borderWidth?: number;
    fill?: boolean;
    tension?: number;
    pointRadius?: number;
    type?: ChartType;
    hidden?: boolean;
}
export interface DataPoint {
    x: string | number;
    y: number;
    label?: string;
    metadata?: Record<string, any>;
}
export interface ChartOptions {
    responsive?: boolean;
    maintainAspectRatio?: boolean;
    aspectRatio?: number;
    showLegend?: boolean;
    legendPosition?: 'top' | 'bottom' | 'left' | 'right';
    showGrid?: boolean;
    showTooltips?: boolean;
    animated?: boolean;
    animationDuration?: number;
    xAxis?: AxisConfig;
    yAxis?: AxisConfig;
    colors?: string[];
    enableZoom?: boolean;
    enablePan?: boolean;
    stacked?: boolean;
}
export interface AxisConfig {
    label?: string;
    display?: boolean;
    type?: 'linear' | 'logarithmic' | 'time' | 'category';
    min?: number;
    max?: number;
    gridLines?: boolean;
    ticks?: {
        display?: boolean;
        fontSize?: number;
        fontColor?: string;
        format?: string;
    };
}
export interface DashboardFilters {
    timeRange: TimeRange;
    startDate?: string;
    endDate?: string;
    departments?: string[];
    regions?: string[];
    users?: string[];
    statuses?: string[];
    custom?: Record<string, any>;
}
export interface ExecutiveOverview {
    id: string;
    generatedAt: string;
    period: TimeRange;
    keyMetrics: MetricData[];
    revenue: RevenueData;
    performance: PerformanceData;
    initiatives: Initiative[];
    risks: RiskIndicator[];
    highlights: Highlight[];
    recommendations: Recommendation[];
}
export interface RevenueData {
    total: number;
    growth: number;
    bySegment: Record<string, number>;
    byRegion: Record<string, number>;
    forecast?: number[];
    target?: number;
}
export interface PerformanceData {
    score: number;
    trend: TrendDirection;
    efficiency: Record<string, number>;
    quality: Record<string, number>;
    satisfaction: number;
    engagement: number;
}
export interface Initiative {
    id: string;
    name: string;
    description: string;
    status: 'planning' | 'in-progress' | 'completed' | 'on-hold' | 'cancelled';
    progress: number;
    owner: string;
    startDate: string;
    targetDate: string;
    completedDate?: string;
    priority: 'low' | 'medium' | 'high' | 'critical';
    milestones: Milestone[];
}
export interface Milestone {
    id: string;
    name: string;
    targetDate: string;
    completed: boolean;
    completedDate?: string;
}
export interface RiskIndicator {
    id: string;
    category: string;
    description: string;
    severity: AlertSeverity;
    probability: number;
    impact: number;
    score: number;
    mitigated: boolean;
    mitigationPlan?: string;
    owner?: string;
}
export interface Highlight {
    id: string;
    type: 'achievement' | 'milestone' | 'alert' | 'insight';
    title: string;
    description: string;
    timestamp: string;
    metricId?: string;
    icon?: string;
    color?: string;
}
export interface Recommendation {
    id: string;
    title: string;
    description: string;
    priority: 'low' | 'medium' | 'high';
    impact: string;
    confidence: number;
    category: string;
    actions: string[];
}
export interface ActivityFeedItem {
    id: string;
    type: 'user' | 'system' | 'alert' | 'metric' | 'event';
    timestamp: string;
    user?: {
        id: string;
        name: string;
        avatar?: string;
    };
    title: string;
    description: string;
    severity?: AlertSeverity;
    resource?: {
        type: string;
        id: string;
        name: string;
    };
    icon?: string;
    color?: string;
    actionUrl?: string;
    actionLabel?: string;
    read?: boolean;
}
export interface ExportConfig {
    format: ExportFormat;
    includeCharts?: boolean;
    includeRawData?: boolean;
    dateRange?: {
        start: string;
        end: string;
    };
    metrics?: string[];
    charts?: string[];
    fileName?: string;
    orientation?: 'portrait' | 'landscape';
    paperSize?: 'letter' | 'a4' | 'legal';
    includeHeader?: boolean;
    includePageNumbers?: boolean;
    branding?: {
        logo?: string;
        companyName?: string;
        footer?: string;
    };
}
export interface DashboardResponse<T = any> {
    success: boolean;
    data?: T;
    error?: string;
    errorCode?: string;
    timestamp: string;
    requestId?: string;
    pagination?: {
        page: number;
        pageSize: number;
        totalPages: number;
        totalItems: number;
    };
}
export interface WebSocketMessage {
    type: 'metric-update' | 'chart-update' | 'activity' | 'alert' | 'ping' | 'pong';
    payload: any;
    timestamp: string;
    channel?: string;
}
export interface LoadingState {
    isLoading: boolean;
    progress?: number;
    message?: string;
}
export interface ErrorState {
    hasError: boolean;
    message?: string;
    code?: string;
    details?: any;
    retry?: () => void;
}
export interface AccessibilityConfig {
    highContrast?: boolean;
    reducedMotion?: boolean;
    screenReaderOptimized?: boolean;
    keyboardNavigation?: boolean;
    focusIndicators?: boolean;
    ariaLabels?: boolean;
    fontSizeMultiplier?: number;
}
//# sourceMappingURL=types.d.ts.map