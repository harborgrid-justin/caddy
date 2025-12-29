/**
 * CADDY Enterprise Dashboard - TypeScript Type Definitions v0.4.0
 *
 * Comprehensive type definitions for the enterprise dashboard system including
 * metrics, KPIs, charts, widgets, and real-time data structures.
 */

/**
 * Dashboard theme mode
 */
export type ThemeMode = 'light' | 'dark' | 'auto';

/**
 * Chart type enumeration
 */
export type ChartType = 'line' | 'bar' | 'pie' | 'area' | 'scatter' | 'radar' | 'heatmap';

/**
 * Time range for data filtering
 */
export type TimeRange = '1h' | '24h' | '7d' | '30d' | '90d' | '1y' | 'custom';

/**
 * Data aggregation interval
 */
export type AggregationInterval = '1m' | '5m' | '15m' | '1h' | '6h' | '1d' | '1w' | '1mo';

/**
 * Export format types
 */
export type ExportFormat = 'pdf' | 'excel' | 'csv' | 'json';

/**
 * Widget size options
 */
export type WidgetSize = 'small' | 'medium' | 'large' | 'xlarge' | 'full';

/**
 * Metric trend direction
 */
export type TrendDirection = 'up' | 'down' | 'neutral';

/**
 * Alert severity levels
 */
export type AlertSeverity = 'info' | 'warning' | 'error' | 'critical';

/**
 * Dashboard configuration
 */
export interface DashboardConfig {
  /** Dashboard unique identifier */
  id: string;
  /** Dashboard title */
  title: string;
  /** Dashboard description */
  description?: string;
  /** Theme mode */
  theme: ThemeMode;
  /** Enable real-time updates */
  enableRealtime?: boolean;
  /** Real-time update interval in ms */
  realtimeInterval?: number;
  /** WebSocket URL for real-time data */
  wsUrl?: string;
  /** Authentication token */
  authToken?: string;
  /** Default time range */
  defaultTimeRange?: TimeRange;
  /** Default refresh interval in seconds */
  refreshInterval?: number;
  /** Enable auto-refresh */
  autoRefresh?: boolean;
  /** Accessible mode (WCAG AAA) */
  accessibleMode?: boolean;
  /** Custom color palette */
  colorPalette?: ColorPalette;
}

/**
 * Color palette for theming
 */
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

/**
 * Dashboard layout configuration
 */
export interface DashboardLayout {
  /** Layout ID */
  id: string;
  /** Grid columns (responsive) */
  columns: { xs: number; sm: number; md: number; lg: number; xl: number };
  /** Grid gap in pixels */
  gap: number;
  /** Widgets in this layout */
  widgets: WidgetConfig[];
  /** Custom CSS class */
  className?: string;
}

/**
 * Widget configuration
 */
export interface WidgetConfig {
  /** Widget unique identifier */
  id: string;
  /** Widget type */
  type: 'metric' | 'chart' | 'table' | 'feed' | 'custom';
  /** Widget title */
  title: string;
  /** Widget description */
  description?: string;
  /** Widget size */
  size: WidgetSize;
  /** Grid position */
  position: { row: number; col: number };
  /** Grid span */
  span: { rows: number; cols: number };
  /** Data source configuration */
  dataSource: DataSourceConfig;
  /** Refresh interval in seconds */
  refreshInterval?: number;
  /** Enable auto-refresh */
  autoRefresh?: boolean;
  /** Custom styling */
  style?: React.CSSProperties;
  /** Widget-specific options */
  options?: Record<string, any>;
}

/**
 * Data source configuration
 */
export interface DataSourceConfig {
  /** Data source type */
  type: 'api' | 'websocket' | 'static';
  /** API endpoint URL */
  url?: string;
  /** HTTP method */
  method?: 'GET' | 'POST' | 'PUT' | 'DELETE';
  /** Request headers */
  headers?: Record<string, string>;
  /** Request body */
  body?: any;
  /** WebSocket channel */
  channel?: string;
  /** Static data */
  data?: any;
  /** Data transformation function name */
  transform?: string;
}

/**
 * KPI Metric data
 */
export interface MetricData {
  /** Metric unique identifier */
  id: string;
  /** Metric name */
  name: string;
  /** Metric value */
  value: number;
  /** Formatted value string */
  formattedValue: string;
  /** Previous value for comparison */
  previousValue?: number;
  /** Change amount */
  change?: number;
  /** Change percentage */
  changePercent?: number;
  /** Trend direction */
  trend: TrendDirection;
  /** Metric unit */
  unit: string;
  /** Metric category */
  category: string;
  /** Icon name */
  icon?: string;
  /** Color theme */
  color?: string;
  /** Target value */
  target?: number;
  /** Progress percentage (0-100) */
  progress?: number;
  /** Historical data points */
  history?: DataPoint[];
  /** Last updated timestamp */
  lastUpdated: string;
  /** Additional metadata */
  metadata?: Record<string, any>;
}

/**
 * Chart data structure
 */
export interface ChartData {
  /** Chart unique identifier */
  id: string;
  /** Chart title */
  title: string;
  /** Chart type */
  type: ChartType;
  /** Chart datasets */
  datasets: ChartDataset[];
  /** X-axis labels */
  labels?: string[];
  /** Chart options */
  options: ChartOptions;
  /** Last updated timestamp */
  lastUpdated: string;
}

/**
 * Chart dataset
 */
export interface ChartDataset {
  /** Dataset label */
  label: string;
  /** Data points */
  data: (number | DataPoint)[];
  /** Line/bar color */
  color?: string;
  /** Background color */
  backgroundColor?: string;
  /** Border color */
  borderColor?: string;
  /** Border width */
  borderWidth?: number;
  /** Fill area under line */
  fill?: boolean;
  /** Line tension (smoothness) */
  tension?: number;
  /** Point radius */
  pointRadius?: number;
  /** Dataset type override */
  type?: ChartType;
  /** Hidden by default */
  hidden?: boolean;
}

/**
 * Data point structure
 */
export interface DataPoint {
  /** X-axis value (usually timestamp) */
  x: string | number;
  /** Y-axis value */
  y: number;
  /** Optional label */
  label?: string;
  /** Additional metadata */
  metadata?: Record<string, any>;
}

/**
 * Chart options
 */
export interface ChartOptions {
  /** Responsive behavior */
  responsive?: boolean;
  /** Maintain aspect ratio */
  maintainAspectRatio?: boolean;
  /** Aspect ratio (width/height) */
  aspectRatio?: number;
  /** Show legend */
  showLegend?: boolean;
  /** Legend position */
  legendPosition?: 'top' | 'bottom' | 'left' | 'right';
  /** Show grid lines */
  showGrid?: boolean;
  /** Enable tooltips */
  showTooltips?: boolean;
  /** Enable animations */
  animated?: boolean;
  /** Animation duration in ms */
  animationDuration?: number;
  /** X-axis configuration */
  xAxis?: AxisConfig;
  /** Y-axis configuration */
  yAxis?: AxisConfig;
  /** Custom colors */
  colors?: string[];
  /** Enable zoom */
  enableZoom?: boolean;
  /** Enable pan */
  enablePan?: boolean;
  /** Stacked bars/areas */
  stacked?: boolean;
}

/**
 * Axis configuration
 */
export interface AxisConfig {
  /** Axis label */
  label?: string;
  /** Display axis */
  display?: boolean;
  /** Axis type */
  type?: 'linear' | 'logarithmic' | 'time' | 'category';
  /** Minimum value */
  min?: number;
  /** Maximum value */
  max?: number;
  /** Grid line display */
  gridLines?: boolean;
  /** Tick configuration */
  ticks?: {
    display?: boolean;
    fontSize?: number;
    fontColor?: string;
    format?: string;
  };
}

/**
 * Dashboard filter configuration
 */
export interface DashboardFilters {
  /** Time range filter */
  timeRange: TimeRange;
  /** Custom start date */
  startDate?: string;
  /** Custom end date */
  endDate?: string;
  /** Department filter */
  departments?: string[];
  /** Region filter */
  regions?: string[];
  /** User filter */
  users?: string[];
  /** Status filter */
  statuses?: string[];
  /** Custom filters */
  custom?: Record<string, any>;
}

/**
 * Executive overview data
 */
export interface ExecutiveOverview {
  /** Overview ID */
  id: string;
  /** Generated timestamp */
  generatedAt: string;
  /** Time period */
  period: TimeRange;
  /** Key metrics summary */
  keyMetrics: MetricData[];
  /** Revenue data */
  revenue: RevenueData;
  /** Performance indicators */
  performance: PerformanceData;
  /** Strategic initiatives */
  initiatives: Initiative[];
  /** Risk indicators */
  risks: RiskIndicator[];
  /** Highlights */
  highlights: Highlight[];
  /** Recommendations */
  recommendations: Recommendation[];
}

/**
 * Revenue data structure
 */
export interface RevenueData {
  /** Total revenue */
  total: number;
  /** Revenue growth rate */
  growth: number;
  /** Revenue by segment */
  bySegment: Record<string, number>;
  /** Revenue by region */
  byRegion: Record<string, number>;
  /** Revenue forecast */
  forecast?: number[];
  /** Target revenue */
  target?: number;
}

/**
 * Performance data structure
 */
export interface PerformanceData {
  /** Overall score (0-100) */
  score: number;
  /** Performance trend */
  trend: TrendDirection;
  /** Efficiency metrics */
  efficiency: Record<string, number>;
  /** Quality metrics */
  quality: Record<string, number>;
  /** Customer satisfaction */
  satisfaction: number;
  /** Employee engagement */
  engagement: number;
}

/**
 * Strategic initiative
 */
export interface Initiative {
  /** Initiative ID */
  id: string;
  /** Initiative name */
  name: string;
  /** Initiative description */
  description: string;
  /** Current status */
  status: 'planning' | 'in-progress' | 'completed' | 'on-hold' | 'cancelled';
  /** Progress percentage (0-100) */
  progress: number;
  /** Owner */
  owner: string;
  /** Start date */
  startDate: string;
  /** Target completion date */
  targetDate: string;
  /** Actual completion date */
  completedDate?: string;
  /** Priority level */
  priority: 'low' | 'medium' | 'high' | 'critical';
  /** Key milestones */
  milestones: Milestone[];
}

/**
 * Milestone structure
 */
export interface Milestone {
  /** Milestone ID */
  id: string;
  /** Milestone name */
  name: string;
  /** Target date */
  targetDate: string;
  /** Completion status */
  completed: boolean;
  /** Completion date */
  completedDate?: string;
}

/**
 * Risk indicator
 */
export interface RiskIndicator {
  /** Risk ID */
  id: string;
  /** Risk category */
  category: string;
  /** Risk description */
  description: string;
  /** Severity level */
  severity: AlertSeverity;
  /** Probability (0-100) */
  probability: number;
  /** Impact (0-100) */
  impact: number;
  /** Risk score */
  score: number;
  /** Mitigation status */
  mitigated: boolean;
  /** Mitigation plan */
  mitigationPlan?: string;
  /** Owner */
  owner?: string;
}

/**
 * Highlight structure
 */
export interface Highlight {
  /** Highlight ID */
  id: string;
  /** Highlight type */
  type: 'achievement' | 'milestone' | 'alert' | 'insight';
  /** Title */
  title: string;
  /** Description */
  description: string;
  /** Timestamp */
  timestamp: string;
  /** Related metric ID */
  metricId?: string;
  /** Icon */
  icon?: string;
  /** Color */
  color?: string;
}

/**
 * Recommendation structure
 */
export interface Recommendation {
  /** Recommendation ID */
  id: string;
  /** Recommendation title */
  title: string;
  /** Detailed description */
  description: string;
  /** Priority level */
  priority: 'low' | 'medium' | 'high';
  /** Expected impact */
  impact: string;
  /** Confidence score (0-100) */
  confidence: number;
  /** Category */
  category: string;
  /** Action items */
  actions: string[];
}

/**
 * Real-time activity feed item
 */
export interface ActivityFeedItem {
  /** Activity ID */
  id: string;
  /** Activity type */
  type: 'user' | 'system' | 'alert' | 'metric' | 'event';
  /** Timestamp */
  timestamp: string;
  /** User who triggered the activity */
  user?: {
    id: string;
    name: string;
    avatar?: string;
  };
  /** Activity title */
  title: string;
  /** Activity description */
  description: string;
  /** Severity (for alerts) */
  severity?: AlertSeverity;
  /** Related resource */
  resource?: {
    type: string;
    id: string;
    name: string;
  };
  /** Icon */
  icon?: string;
  /** Color */
  color?: string;
  /** Action link */
  actionUrl?: string;
  /** Action label */
  actionLabel?: string;
  /** Read status */
  read?: boolean;
}

/**
 * Export configuration
 */
export interface ExportConfig {
  /** Export format */
  format: ExportFormat;
  /** Include charts as images */
  includeCharts?: boolean;
  /** Include raw data */
  includeRawData?: boolean;
  /** Date range */
  dateRange?: {
    start: string;
    end: string;
  };
  /** Selected metrics */
  metrics?: string[];
  /** Selected charts */
  charts?: string[];
  /** File name */
  fileName?: string;
  /** Page orientation (PDF) */
  orientation?: 'portrait' | 'landscape';
  /** Paper size (PDF) */
  paperSize?: 'letter' | 'a4' | 'legal';
  /** Include header/footer */
  includeHeader?: boolean;
  /** Include page numbers */
  includePageNumbers?: boolean;
  /** Custom branding */
  branding?: {
    logo?: string;
    companyName?: string;
    footer?: string;
  };
}

/**
 * Dashboard API response
 */
export interface DashboardResponse<T = any> {
  /** Response success status */
  success: boolean;
  /** Response data */
  data?: T;
  /** Error message */
  error?: string;
  /** Error code */
  errorCode?: string;
  /** Response timestamp */
  timestamp: string;
  /** Request ID for tracking */
  requestId?: string;
  /** Pagination metadata */
  pagination?: {
    page: number;
    pageSize: number;
    totalPages: number;
    totalItems: number;
  };
}

/**
 * WebSocket message structure
 */
export interface WebSocketMessage {
  /** Message type */
  type: 'metric-update' | 'chart-update' | 'activity' | 'alert' | 'ping' | 'pong';
  /** Message payload */
  payload: any;
  /** Message timestamp */
  timestamp: string;
  /** Channel/topic */
  channel?: string;
}

/**
 * Loading state
 */
export interface LoadingState {
  /** Is loading */
  isLoading: boolean;
  /** Loading progress (0-100) */
  progress?: number;
  /** Loading message */
  message?: string;
}

/**
 * Error state
 */
export interface ErrorState {
  /** Has error */
  hasError: boolean;
  /** Error message */
  message?: string;
  /** Error code */
  code?: string;
  /** Error details */
  details?: any;
  /** Retry function */
  retry?: () => void;
}

/**
 * Accessibility configuration
 */
export interface AccessibilityConfig {
  /** High contrast mode */
  highContrast?: boolean;
  /** Reduced motion */
  reducedMotion?: boolean;
  /** Screen reader optimized */
  screenReaderOptimized?: boolean;
  /** Keyboard navigation enhanced */
  keyboardNavigation?: boolean;
  /** Focus indicators */
  focusIndicators?: boolean;
  /** ARIA labels enabled */
  ariaLabels?: boolean;
  /** Font size multiplier */
  fontSizeMultiplier?: number;
}
