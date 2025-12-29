import React, { useEffect, useState, useCallback } from 'react';
import { AlertSeverity, ServiceStatus } from './types';
export const MonitoringDashboard = ({ config, onConfigChange, className = '' }) => {
    const [services, setServices] = useState([]);
    const [alerts, setAlerts] = useState([]);
    const [stats, setStats] = useState({
        totalServices: 0,
        healthyServices: 0,
        degradedServices: 0,
        downServices: 0,
        activeAlerts: 0,
        criticalAlerts: 0,
        averageResponseTime: 0,
        overallUptime: 100
    });
    const [timeRange, setTimeRange] = useState({
        from: new Date(Date.now() - 3600000),
        to: new Date(),
        quick: '1h'
    });
    const [isConnected, setIsConnected] = useState(false);
    const [lastUpdate, setLastUpdate] = useState(new Date());
    const [autoRefresh, setAutoRefresh] = useState(true);
    const [ws, setWs] = useState(null);
    useEffect(() => {
        const connectWebSocket = () => {
            const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
            const wsUrl = `${protocol}//${window.location.host}/api/monitoring/stream`;
            const socket = new WebSocket(wsUrl);
            socket.onopen = () => {
                console.log('[MonitoringDashboard] WebSocket connected');
                setIsConnected(true);
                socket.send(JSON.stringify({ type: 'subscribe', channels: ['metrics', 'alerts', 'health'] }));
            };
            socket.onmessage = (event) => {
                try {
                    const message = JSON.parse(event.data);
                    handleWebSocketMessage(message);
                }
                catch (error) {
                    console.error('[MonitoringDashboard] Failed to parse WebSocket message:', error);
                }
            };
            socket.onerror = (error) => {
                console.error('[MonitoringDashboard] WebSocket error:', error);
                setIsConnected(false);
            };
            socket.onclose = () => {
                console.log('[MonitoringDashboard] WebSocket disconnected');
                setIsConnected(false);
                setTimeout(connectWebSocket, 5000);
            };
            setWs(socket);
        };
        if (autoRefresh) {
            connectWebSocket();
        }
        return () => {
            if (ws) {
                ws.close();
            }
        };
    }, [autoRefresh]);
    const handleWebSocketMessage = useCallback((message) => {
        setLastUpdate(new Date());
        switch (message.type) {
            case 'health':
                setServices(prev => {
                    const updated = [...prev];
                    const index = updated.findIndex(s => s.id === message.data.id);
                    if (index >= 0) {
                        updated[index] = message.data;
                    }
                    else {
                        updated.push(message.data);
                    }
                    return updated;
                });
                break;
            case 'alert':
                setAlerts(prev => {
                    const updated = [...prev];
                    const index = updated.findIndex(a => a.id === message.data.id);
                    if (index >= 0) {
                        updated[index] = message.data;
                    }
                    else {
                        updated.unshift(message.data);
                    }
                    return updated.slice(0, 100);
                });
                break;
            default:
                break;
        }
    }, []);
    useEffect(() => {
        const newStats = {
            totalServices: services.length,
            healthyServices: services.filter(s => s.status === ServiceStatus.HEALTHY).length,
            degradedServices: services.filter(s => s.status === ServiceStatus.DEGRADED).length,
            downServices: services.filter(s => s.status === ServiceStatus.DOWN).length,
            activeAlerts: alerts.filter(a => a.state === 'active').length,
            criticalAlerts: alerts.filter(a => a.state === 'active' && a.severity === AlertSeverity.CRITICAL).length,
            averageResponseTime: services.length > 0
                ? services.reduce((sum, s) => sum + s.responseTime, 0) / services.length
                : 0,
            overallUptime: services.length > 0
                ? services.reduce((sum, s) => sum + s.uptime, 0) / services.length
                : 100
        };
        setStats(newStats);
    }, [services, alerts]);
    useEffect(() => {
        fetchDashboardData();
    }, [timeRange]);
    const fetchDashboardData = async () => {
        try {
            const [servicesRes, alertsRes] = await Promise.all([
                fetch('/api/monitoring/services'),
                fetch('/api/monitoring/alerts?state=active')
            ]);
            if (servicesRes.ok) {
                const servicesData = await servicesRes.json();
                setServices(servicesData);
            }
            if (alertsRes.ok) {
                const alertsData = await alertsRes.json();
                setAlerts(alertsData);
            }
        }
        catch (error) {
            console.error('[MonitoringDashboard] Failed to fetch dashboard data:', error);
        }
    };
    const handleTimeRangeChange = (range) => {
        setTimeRange(range);
    };
    const getStatusColor = (status) => {
        switch (status) {
            case ServiceStatus.HEALTHY:
                return '#10b981';
            case ServiceStatus.DEGRADED:
                return '#f59e0b';
            case ServiceStatus.DOWN:
                return '#ef4444';
            case ServiceStatus.MAINTENANCE:
                return '#3b82f6';
            default:
                return '#6b7280';
        }
    };
    const getSeverityColor = (severity) => {
        switch (severity) {
            case AlertSeverity.CRITICAL:
                return '#dc2626';
            case AlertSeverity.HIGH:
                return '#f59e0b';
            case AlertSeverity.MEDIUM:
                return '#3b82f6';
            case AlertSeverity.LOW:
                return '#6b7280';
            default:
                return '#6b7280';
        }
    };
    const formatUptime = (uptime) => {
        return `${uptime.toFixed(3)}%`;
    };
    const formatResponseTime = (ms) => {
        if (ms < 1000) {
            return `${ms.toFixed(0)}ms`;
        }
        return `${(ms / 1000).toFixed(2)}s`;
    };
    const quickTimeRanges = [
        { label: '5m', value: 5 * 60 * 1000 },
        { label: '15m', value: 15 * 60 * 1000 },
        { label: '1h', value: 60 * 60 * 1000 },
        { label: '6h', value: 6 * 60 * 60 * 1000 },
        { label: '24h', value: 24 * 60 * 60 * 1000 },
        { label: '7d', value: 7 * 24 * 60 * 60 * 1000 }
    ];
    return (React.createElement("div", { className: `monitoring-dashboard ${className}`, style: styles.container },
        React.createElement("div", { style: styles.header },
            React.createElement("div", { style: styles.headerLeft },
                React.createElement("h1", { style: styles.title }, "System Monitoring Dashboard"),
                React.createElement("div", { style: styles.connectionStatus },
                    React.createElement("div", { style: {
                            ...styles.statusIndicator,
                            backgroundColor: isConnected ? '#10b981' : '#ef4444'
                        } }),
                    React.createElement("span", { style: styles.statusText }, isConnected ? 'Connected' : 'Disconnected'),
                    React.createElement("span", { style: styles.lastUpdateText },
                        "Last update: ",
                        lastUpdate.toLocaleTimeString()))),
            React.createElement("div", { style: styles.headerRight },
                React.createElement("div", { style: styles.timeRangeSelector }, quickTimeRanges.map((range) => (React.createElement("button", { key: range.label, style: {
                        ...styles.timeRangeButton,
                        ...(timeRange.quick === range.label ? styles.timeRangeButtonActive : {})
                    }, onClick: () => handleTimeRangeChange({
                        from: new Date(Date.now() - range.value),
                        to: new Date(),
                        quick: range.label
                    }) }, range.label)))),
                React.createElement("button", { style: {
                        ...styles.refreshButton,
                        ...(autoRefresh ? styles.refreshButtonActive : {})
                    }, onClick: () => setAutoRefresh(!autoRefresh) }, autoRefresh ? '● Auto' : '○ Manual'))),
        React.createElement("div", { style: styles.statsGrid },
            React.createElement("div", { style: styles.statCard },
                React.createElement("div", { style: styles.statValue }, stats.totalServices),
                React.createElement("div", { style: styles.statLabel }, "Total Services")),
            React.createElement("div", { style: styles.statCard },
                React.createElement("div", { style: { ...styles.statValue, color: '#10b981' } }, stats.healthyServices),
                React.createElement("div", { style: styles.statLabel }, "Healthy")),
            React.createElement("div", { style: styles.statCard },
                React.createElement("div", { style: { ...styles.statValue, color: '#f59e0b' } }, stats.degradedServices),
                React.createElement("div", { style: styles.statLabel }, "Degraded")),
            React.createElement("div", { style: styles.statCard },
                React.createElement("div", { style: { ...styles.statValue, color: '#ef4444' } }, stats.downServices),
                React.createElement("div", { style: styles.statLabel }, "Down")),
            React.createElement("div", { style: styles.statCard },
                React.createElement("div", { style: { ...styles.statValue, color: stats.activeAlerts > 0 ? '#f59e0b' : '#10b981' } }, stats.activeAlerts),
                React.createElement("div", { style: styles.statLabel }, "Active Alerts")),
            React.createElement("div", { style: styles.statCard },
                React.createElement("div", { style: { ...styles.statValue, color: stats.criticalAlerts > 0 ? '#dc2626' : '#10b981' } }, stats.criticalAlerts),
                React.createElement("div", { style: styles.statLabel }, "Critical")),
            React.createElement("div", { style: styles.statCard },
                React.createElement("div", { style: styles.statValue }, formatResponseTime(stats.averageResponseTime)),
                React.createElement("div", { style: styles.statLabel }, "Avg Response")),
            React.createElement("div", { style: styles.statCard },
                React.createElement("div", { style: { ...styles.statValue, color: stats.overallUptime >= 99.9 ? '#10b981' : '#f59e0b' } }, formatUptime(stats.overallUptime)),
                React.createElement("div", { style: styles.statLabel }, "Overall Uptime"))),
        alerts.length > 0 && (React.createElement("div", { style: styles.section },
            React.createElement("h2", { style: styles.sectionTitle }, "Active Alerts"),
            React.createElement("div", { style: styles.alertsList }, alerts.slice(0, 5).map((alert) => (React.createElement("div", { key: alert.id, style: styles.alertItem },
                React.createElement("div", { style: {
                        ...styles.alertSeverity,
                        backgroundColor: getSeverityColor(alert.severity)
                    } }),
                React.createElement("div", { style: styles.alertContent },
                    React.createElement("div", { style: styles.alertHeader },
                        React.createElement("span", { style: styles.alertName }, alert.name),
                        React.createElement("span", { style: styles.alertService }, alert.service)),
                    React.createElement("div", { style: styles.alertMessage }, alert.message),
                    React.createElement("div", { style: styles.alertTime }, new Date(alert.triggeredAt).toLocaleString())))))))),
        React.createElement("div", { style: styles.section },
            React.createElement("h2", { style: styles.sectionTitle }, "Services Status"),
            React.createElement("div", { style: styles.servicesGrid }, services.map((service) => (React.createElement("div", { key: service.id, style: styles.serviceCard },
                React.createElement("div", { style: styles.serviceHeader },
                    React.createElement("div", { style: {
                            ...styles.serviceStatus,
                            backgroundColor: getStatusColor(service.status)
                        } }),
                    React.createElement("span", { style: styles.serviceName }, service.name)),
                React.createElement("div", { style: styles.serviceMetrics },
                    React.createElement("div", { style: styles.serviceMetric },
                        React.createElement("span", { style: styles.metricLabel }, "Uptime:"),
                        React.createElement("span", { style: styles.metricValue }, formatUptime(service.uptime))),
                    React.createElement("div", { style: styles.serviceMetric },
                        React.createElement("span", { style: styles.metricLabel }, "Response:"),
                        React.createElement("span", { style: styles.metricValue }, formatResponseTime(service.responseTime))),
                    React.createElement("div", { style: styles.serviceMetric },
                        React.createElement("span", { style: styles.metricLabel }, "CPU:"),
                        React.createElement("span", { style: styles.metricValue },
                            service.metrics.cpu.toFixed(1),
                            "%")),
                    React.createElement("div", { style: styles.serviceMetric },
                        React.createElement("span", { style: styles.metricLabel }, "Memory:"),
                        React.createElement("span", { style: styles.metricValue },
                            service.metrics.memory.toFixed(1),
                            "%"))))))))));
};
const styles = {
    container: {
        padding: '24px',
        backgroundColor: '#f9fafb',
        minHeight: '100vh',
        fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif'
    },
    header: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '24px',
        flexWrap: 'wrap',
        gap: '16px'
    },
    headerLeft: {
        flex: 1
    },
    headerRight: {
        display: 'flex',
        gap: '12px',
        alignItems: 'center'
    },
    title: {
        fontSize: '28px',
        fontWeight: 700,
        color: '#111827',
        margin: 0,
        marginBottom: '8px'
    },
    connectionStatus: {
        display: 'flex',
        alignItems: 'center',
        gap: '8px'
    },
    statusIndicator: {
        width: '8px',
        height: '8px',
        borderRadius: '50%'
    },
    statusText: {
        fontSize: '14px',
        color: '#6b7280',
        fontWeight: 500
    },
    lastUpdateText: {
        fontSize: '12px',
        color: '#9ca3af',
        marginLeft: '8px'
    },
    timeRangeSelector: {
        display: 'flex',
        gap: '4px',
        backgroundColor: '#fff',
        borderRadius: '8px',
        padding: '4px',
        border: '1px solid #e5e7eb'
    },
    timeRangeButton: {
        padding: '6px 12px',
        border: 'none',
        background: 'transparent',
        borderRadius: '6px',
        cursor: 'pointer',
        fontSize: '13px',
        fontWeight: 500,
        color: '#6b7280',
        transition: 'all 0.2s'
    },
    timeRangeButtonActive: {
        backgroundColor: '#3b82f6',
        color: '#fff'
    },
    refreshButton: {
        padding: '8px 16px',
        border: '1px solid #e5e7eb',
        background: '#fff',
        borderRadius: '8px',
        cursor: 'pointer',
        fontSize: '13px',
        fontWeight: 500,
        color: '#6b7280',
        transition: 'all 0.2s'
    },
    refreshButtonActive: {
        backgroundColor: '#10b981',
        color: '#fff',
        borderColor: '#10b981'
    },
    statsGrid: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))',
        gap: '16px',
        marginBottom: '24px'
    },
    statCard: {
        backgroundColor: '#fff',
        borderRadius: '12px',
        padding: '20px',
        border: '1px solid #e5e7eb',
        textAlign: 'center'
    },
    statValue: {
        fontSize: '32px',
        fontWeight: 700,
        color: '#111827',
        marginBottom: '4px'
    },
    statLabel: {
        fontSize: '13px',
        color: '#6b7280',
        fontWeight: 500
    },
    section: {
        marginBottom: '24px'
    },
    sectionTitle: {
        fontSize: '18px',
        fontWeight: 600,
        color: '#111827',
        marginBottom: '16px'
    },
    alertsList: {
        display: 'flex',
        flexDirection: 'column',
        gap: '12px'
    },
    alertItem: {
        display: 'flex',
        backgroundColor: '#fff',
        borderRadius: '8px',
        border: '1px solid #e5e7eb',
        overflow: 'hidden'
    },
    alertSeverity: {
        width: '4px',
        flexShrink: 0
    },
    alertContent: {
        padding: '16px',
        flex: 1
    },
    alertHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '8px'
    },
    alertName: {
        fontSize: '15px',
        fontWeight: 600,
        color: '#111827'
    },
    alertService: {
        fontSize: '13px',
        color: '#6b7280',
        backgroundColor: '#f3f4f6',
        padding: '2px 8px',
        borderRadius: '4px'
    },
    alertMessage: {
        fontSize: '14px',
        color: '#4b5563',
        marginBottom: '8px'
    },
    alertTime: {
        fontSize: '12px',
        color: '#9ca3af'
    },
    servicesGrid: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))',
        gap: '16px'
    },
    serviceCard: {
        backgroundColor: '#fff',
        borderRadius: '8px',
        padding: '16px',
        border: '1px solid #e5e7eb'
    },
    serviceHeader: {
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        marginBottom: '12px'
    },
    serviceStatus: {
        width: '10px',
        height: '10px',
        borderRadius: '50%'
    },
    serviceName: {
        fontSize: '15px',
        fontWeight: 600,
        color: '#111827'
    },
    serviceMetrics: {
        display: 'grid',
        gridTemplateColumns: '1fr 1fr',
        gap: '8px'
    },
    serviceMetric: {
        display: 'flex',
        justifyContent: 'space-between',
        fontSize: '13px'
    },
    metricLabel: {
        color: '#6b7280'
    },
    metricValue: {
        color: '#111827',
        fontWeight: 500
    }
};
export default MonitoringDashboard;
//# sourceMappingURL=MonitoringDashboard.js.map