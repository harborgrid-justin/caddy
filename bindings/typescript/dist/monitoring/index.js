export * from './types';
export { MonitoringDashboard } from './MonitoringDashboard';
export { HealthChecks } from './HealthChecks';
export { PerformanceMetrics } from './PerformanceMetrics';
export { AlertManager } from './AlertManager';
export { AlertHistory } from './AlertHistory';
export { IncidentManager } from './IncidentManager';
export { UptimeDisplay } from './UptimeDisplay';
export { LogViewer } from './LogViewer';
export { ResourceUsage } from './ResourceUsage';
export { ServiceMap } from './ServiceMap';
export { MaintenanceMode } from './MaintenanceMode';
export { StatusPage } from './StatusPage';
export { default as MonitoringDashboardDefault } from './MonitoringDashboard';
export { default as HealthChecksDefault } from './HealthChecks';
export { default as PerformanceMetricsDefault } from './PerformanceMetrics';
export { default as AlertManagerDefault } from './AlertManager';
export { default as AlertHistoryDefault } from './AlertHistory';
export { default as IncidentManagerDefault } from './IncidentManager';
export { default as UptimeDisplayDefault } from './UptimeDisplay';
export { default as LogViewerDefault } from './LogViewer';
export { default as ResourceUsageDefault } from './ResourceUsage';
export { default as ServiceMapDefault } from './ServiceMap';
export { default as MaintenanceModeDefault } from './MaintenanceMode';
export { default as StatusPageDefault } from './StatusPage';
export const MONITORING_VERSION = '0.4.0';
export const MONITORING_FEATURES = {
    REAL_TIME_METRICS: true,
    WEBSOCKET_STREAMING: true,
    ALERT_MANAGEMENT: true,
    INCIDENT_TRACKING: true,
    ANOMALY_DETECTION: true,
    SLA_TRACKING: true,
    LOG_STREAMING: true,
    SERVICE_DEPENDENCIES: true,
    MAINTENANCE_WINDOWS: true,
    STATUS_PAGES: true,
    HEALTH_CHECKS: true,
    PERFORMANCE_MONITORING: true,
    RESOURCE_TRACKING: true
};
export const DEFAULT_MONITORING_CONFIG = {
    refreshInterval: 30000,
    metricsRetention: 2592000000,
    alertCooldown: 300,
    healthCheckInterval: 60,
    healthCheckTimeout: 10,
    logRetention: 604800000,
    maxLogLines: 10000,
    wsReconnectDelay: 5000,
    defaultTimeRange: '1h',
    slaTarget: 99.9,
    anomalyDetectionEnabled: true,
    anomalyConfidenceThreshold: 0.85
};
export const MONITORING_ENDPOINTS = {
    SERVICES: '/api/monitoring/services',
    HEALTH: '/api/monitoring/health',
    METRICS: '/api/monitoring/metrics',
    ALERTS: '/api/monitoring/alerts',
    ALERT_RULES: '/api/monitoring/alerts/rules',
    INCIDENTS: '/api/monitoring/incidents',
    UPTIME: '/api/monitoring/uptime',
    SLA: '/api/monitoring/sla',
    LOGS: '/api/monitoring/logs',
    RESOURCES: '/api/monitoring/resources',
    DEPENDENCIES: '/api/monitoring/dependencies',
    MAINTENANCE: '/api/monitoring/maintenance',
    STATUS_PAGE: '/api/monitoring/status-page',
    NOTIFICATIONS: '/api/monitoring/notifications',
    WEBSOCKET: '/api/monitoring/stream'
};
export const WS_MESSAGE_TYPES = {
    SUBSCRIBE: 'subscribe',
    UNSUBSCRIBE: 'unsubscribe',
    METRIC: 'metric',
    ALERT: 'alert',
    LOG: 'log',
    HEALTH: 'health',
    INCIDENT: 'incident',
    RESOURCE: 'resource',
    HEARTBEAT: 'heartbeat'
};
export const ALERT_SEVERITY_PRIORITY = [
    'critical',
    'high',
    'medium',
    'low',
    'info'
];
export const METRIC_AGGREGATIONS = {
    AVG: 'avg',
    MIN: 'min',
    MAX: 'max',
    SUM: 'sum',
    COUNT: 'count',
    P50: 'p50',
    P95: 'p95',
    P99: 'p99'
};
export const TIME_RANGES = {
    LAST_5_MINUTES: '5m',
    LAST_15_MINUTES: '15m',
    LAST_HOUR: '1h',
    LAST_6_HOURS: '6h',
    LAST_24_HOURS: '24h',
    LAST_7_DAYS: '7d',
    LAST_30_DAYS: '30d'
};
export function createMonitoringWebSocket(channels = [], onMessage, onError) {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}${MONITORING_ENDPOINTS.WEBSOCKET}`;
    const ws = new WebSocket(wsUrl);
    ws.onopen = () => {
        console.log('[Monitoring] WebSocket connected');
        if (channels.length > 0) {
            ws.send(JSON.stringify({
                type: WS_MESSAGE_TYPES.SUBSCRIBE,
                channels
            }));
        }
    };
    ws.onmessage = (event) => {
        try {
            const data = JSON.parse(event.data);
            if (onMessage) {
                onMessage(data);
            }
        }
        catch (error) {
            console.error('[Monitoring] Failed to parse WebSocket message:', error);
        }
    };
    ws.onerror = (error) => {
        console.error('[Monitoring] WebSocket error:', error);
        if (onError) {
            onError(error);
        }
    };
    ws.onclose = () => {
        console.log('[Monitoring] WebSocket disconnected');
    };
    return ws;
}
export function formatMetricValue(value, unit) {
    if (unit === '%') {
        return `${value.toFixed(2)}%`;
    }
    if (unit === 'ms') {
        if (value >= 1000) {
            return `${(value / 1000).toFixed(2)}s`;
        }
        return `${value.toFixed(0)}ms`;
    }
    if (unit === 'bytes') {
        const units = ['B', 'KB', 'MB', 'GB', 'TB'];
        let size = value;
        let unitIndex = 0;
        while (size >= 1024 && unitIndex < units.length - 1) {
            size /= 1024;
            unitIndex++;
        }
        return `${size.toFixed(2)} ${units[unitIndex]}`;
    }
    return `${value.toFixed(2)} ${unit}`;
}
export function calculateUptime(successfulChecks, totalChecks) {
    if (totalChecks === 0)
        return 100;
    return (successfulChecks / totalChecks) * 100;
}
export function calculateErrorBudget(target, current, totalTime) {
    const errorBudget = 100 - target;
    const actualError = 100 - current;
    const errorBudgetUsed = actualError;
    const errorBudgetRemaining = errorBudget - actualError;
    return {
        errorBudget,
        errorBudgetRemaining,
        errorBudgetUsed
    };
}
export function getAlertPriority(severity) {
    return ALERT_SEVERITY_PRIORITY.indexOf(severity);
}
export function isServiceHealthy(status, responseTime, errorRate) {
    return (status === 'healthy' &&
        responseTime < 1000 &&
        errorRate < 1);
}
export function formatDuration(ms) {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);
    if (days > 0) {
        return `${days}d ${hours % 24}h ${minutes % 60}m`;
    }
    if (hours > 0) {
        return `${hours}h ${minutes % 60}m`;
    }
    if (minutes > 0) {
        return `${minutes}m ${seconds % 60}s`;
    }
    return `${seconds}s`;
}
export function initMonitoring(config) {
    const finalConfig = { ...DEFAULT_MONITORING_CONFIG, ...config };
    console.log(`[Monitoring] CADDY v${MONITORING_VERSION} Monitoring Module Initialized`, finalConfig);
    return {
        config: finalConfig,
        version: MONITORING_VERSION,
        features: MONITORING_FEATURES
    };
}
//# sourceMappingURL=index.js.map