import React, { useEffect, useState, useCallback } from 'react';
import { ServiceStatus } from './types';
export const HealthChecks = ({ services: serviceFilter, autoRefresh = true, refreshInterval = 30000, onServiceClick, className = '' }) => {
    const [services, setServices] = useState([]);
    const [configs, setConfigs] = useState([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const [selectedService, setSelectedService] = useState(null);
    const [showConfig, setShowConfig] = useState(false);
    const [filter, setFilter] = useState('all');
    const [searchQuery, setSearchQuery] = useState('');
    const [ws, setWs] = useState(null);
    useEffect(() => {
        if (!autoRefresh)
            return;
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/api/monitoring/health/stream`;
        const socket = new WebSocket(wsUrl);
        socket.onopen = () => {
            console.log('[HealthChecks] WebSocket connected');
            socket.send(JSON.stringify({ type: 'subscribe', services: serviceFilter || [] }));
        };
        socket.onmessage = (event) => {
            try {
                const message = JSON.parse(event.data);
                if (message.type === 'health') {
                    updateServiceHealth(message.data);
                }
            }
            catch (err) {
                console.error('[HealthChecks] Failed to parse WebSocket message:', err);
            }
        };
        socket.onerror = (err) => {
            console.error('[HealthChecks] WebSocket error:', err);
        };
        socket.onclose = () => {
            console.log('[HealthChecks] WebSocket disconnected');
            setTimeout(() => {
            }, 5000);
        };
        setWs(socket);
        return () => {
            socket.close();
        };
    }, [autoRefresh, serviceFilter]);
    useEffect(() => {
        fetchHealthData();
    }, [serviceFilter]);
    useEffect(() => {
        if (!autoRefresh || ws)
            return;
        const interval = setInterval(() => {
            fetchHealthData();
        }, refreshInterval);
        return () => clearInterval(interval);
    }, [autoRefresh, refreshInterval, ws]);
    const fetchHealthData = async () => {
        try {
            setLoading(true);
            setError(null);
            const queryParams = serviceFilter
                ? `?services=${serviceFilter.join(',')}`
                : '';
            const [servicesRes, configsRes] = await Promise.all([
                fetch(`/api/monitoring/health${queryParams}`),
                fetch(`/api/monitoring/health/configs${queryParams}`)
            ]);
            if (!servicesRes.ok || !configsRes.ok) {
                throw new Error('Failed to fetch health data');
            }
            const [servicesData, configsData] = await Promise.all([
                servicesRes.json(),
                configsRes.json()
            ]);
            setServices(servicesData);
            setConfigs(configsData);
        }
        catch (err) {
            setError(err instanceof Error ? err.message : 'Unknown error');
            console.error('[HealthChecks] Error fetching health data:', err);
        }
        finally {
            setLoading(false);
        }
    };
    const updateServiceHealth = useCallback((updatedService) => {
        setServices(prev => {
            const index = prev.findIndex(s => s.id === updatedService.id);
            if (index >= 0) {
                const updated = [...prev];
                updated[index] = updatedService;
                return updated;
            }
            return [...prev, updatedService];
        });
    }, []);
    const runHealthCheck = async (serviceId) => {
        try {
            const response = await fetch(`/api/monitoring/health/${serviceId}/check`, {
                method: 'POST'
            });
            if (!response.ok) {
                throw new Error('Health check failed');
            }
            const result = await response.json();
            setServices(prev => prev.map(s => {
                if (s.id === serviceId) {
                    return {
                        ...s,
                        status: result.success ? ServiceStatus.HEALTHY : ServiceStatus.DOWN,
                        responseTime: result.responseTime,
                        lastCheck: result.timestamp
                    };
                }
                return s;
            }));
        }
        catch (err) {
            console.error(`[HealthChecks] Failed to run health check for ${serviceId}:`, err);
        }
    };
    const handleServiceClick = (service) => {
        setSelectedService(service);
        if (onServiceClick) {
            onServiceClick(service);
        }
    };
    const getStatusIcon = (status) => {
        switch (status) {
            case ServiceStatus.HEALTHY:
                return '✓';
            case ServiceStatus.DEGRADED:
                return '⚠';
            case ServiceStatus.DOWN:
                return '✗';
            case ServiceStatus.MAINTENANCE:
                return '🔧';
            default:
                return '?';
        }
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
    const getStatusLabel = (status) => {
        return status.charAt(0).toUpperCase() + status.slice(1);
    };
    const filteredServices = services.filter(service => {
        const matchesFilter = filter === 'all' || service.status === filter;
        const matchesSearch = service.name.toLowerCase().includes(searchQuery.toLowerCase());
        return matchesFilter && matchesSearch;
    });
    const statusCounts = {
        all: services.length,
        healthy: services.filter(s => s.status === ServiceStatus.HEALTHY).length,
        degraded: services.filter(s => s.status === ServiceStatus.DEGRADED).length,
        down: services.filter(s => s.status === ServiceStatus.DOWN).length,
        maintenance: services.filter(s => s.status === ServiceStatus.MAINTENANCE).length
    };
    if (loading && services.length === 0) {
        return (React.createElement("div", { style: styles.loading },
            React.createElement("div", { style: styles.spinner }),
            React.createElement("p", null, "Loading health checks...")));
    }
    return (React.createElement("div", { className: `health-checks ${className}`, style: styles.container },
        React.createElement("div", { style: styles.header },
            React.createElement("h2", { style: styles.title }, "Service Health Checks"),
            React.createElement("div", { style: styles.headerActions },
                React.createElement("button", { style: styles.button, onClick: () => fetchHealthData() }, "Refresh All"),
                React.createElement("button", { style: styles.button, onClick: () => setShowConfig(!showConfig) },
                    showConfig ? 'Hide' : 'Show',
                    " Config"))),
        error && (React.createElement("div", { style: styles.error },
            React.createElement("span", null,
                "\u26A0 ",
                error),
            React.createElement("button", { onClick: () => setError(null), style: styles.errorClose }, "\u00D7"))),
        React.createElement("div", { style: styles.filters },
            React.createElement("div", { style: styles.statusFilters }, ['all', ServiceStatus.HEALTHY, ServiceStatus.DEGRADED, ServiceStatus.DOWN, ServiceStatus.MAINTENANCE].map((status) => (React.createElement("button", { key: status, style: {
                    ...styles.filterButton,
                    ...(filter === status ? styles.filterButtonActive : {})
                }, onClick: () => setFilter(status) },
                status === 'all' ? 'All' : getStatusLabel(status),
                React.createElement("span", { style: styles.filterCount }, status === 'all' ? statusCounts.all : statusCounts[status]))))),
            React.createElement("input", { type: "text", placeholder: "Search services...", value: searchQuery, onChange: (e) => setSearchQuery(e.target.value), style: styles.searchInput })),
        React.createElement("div", { style: styles.grid }, filteredServices.map((service) => {
            const config = configs.find(c => c.service === service.id);
            return (React.createElement("div", { key: service.id, style: styles.card, onClick: () => handleServiceClick(service) },
                React.createElement("div", { style: {
                        ...styles.statusBar,
                        backgroundColor: getStatusColor(service.status)
                    } }),
                React.createElement("div", { style: styles.cardContent },
                    React.createElement("div", { style: styles.cardHeader },
                        React.createElement("div", { style: styles.cardTitle },
                            React.createElement("span", { style: {
                                    ...styles.statusIcon,
                                    color: getStatusColor(service.status)
                                } }, getStatusIcon(service.status)),
                            React.createElement("span", { style: styles.serviceName }, service.name)),
                        React.createElement("button", { style: styles.checkButton, onClick: (e) => {
                                e.stopPropagation();
                                runHealthCheck(service.id);
                            } }, "Check Now")),
                    service.message && (React.createElement("div", { style: styles.message }, service.message)),
                    React.createElement("div", { style: styles.metrics },
                        React.createElement("div", { style: styles.metric },
                            React.createElement("span", { style: styles.metricLabel }, "Response Time:"),
                            React.createElement("span", { style: styles.metricValue }, service.responseTime < 1000
                                ? `${service.responseTime.toFixed(0)}ms`
                                : `${(service.responseTime / 1000).toFixed(2)}s`)),
                        React.createElement("div", { style: styles.metric },
                            React.createElement("span", { style: styles.metricLabel }, "Uptime:"),
                            React.createElement("span", { style: styles.metricValue },
                                service.uptime.toFixed(3),
                                "%")),
                        React.createElement("div", { style: styles.metric },
                            React.createElement("span", { style: styles.metricLabel }, "Last Check:"),
                            React.createElement("span", { style: styles.metricValue }, new Date(service.lastCheck).toLocaleTimeString())),
                        config && (React.createElement("div", { style: styles.metric },
                            React.createElement("span", { style: styles.metricLabel }, "Interval:"),
                            React.createElement("span", { style: styles.metricValue },
                                config.interval,
                                "s")))),
                    service.dependencies.length > 0 && (React.createElement("div", { style: styles.dependencies },
                        React.createElement("span", { style: styles.dependenciesLabel }, "Dependencies:"),
                        React.createElement("div", { style: styles.dependenciesList }, service.dependencies.map((dep, idx) => (React.createElement("span", { key: idx, style: styles.dependency }, dep)))))),
                    showConfig && config && (React.createElement("div", { style: styles.config },
                        React.createElement("div", { style: styles.configTitle }, "Configuration:"),
                        React.createElement("div", { style: styles.configDetails },
                            React.createElement("div", null,
                                "Type: ",
                                config.type),
                            React.createElement("div", null,
                                "Endpoint: ",
                                config.endpoint),
                            React.createElement("div", null,
                                "Timeout: ",
                                config.timeout,
                                "s"),
                            React.createElement("div", null,
                                "Retries: ",
                                config.retries)))))));
        })),
        filteredServices.length === 0 && (React.createElement("div", { style: styles.emptyState },
            React.createElement("p", null, "No services found matching your criteria"))),
        selectedService && (React.createElement("div", { style: styles.modal, onClick: () => setSelectedService(null) },
            React.createElement("div", { style: styles.modalContent, onClick: (e) => e.stopPropagation() },
                React.createElement("div", { style: styles.modalHeader },
                    React.createElement("h3", null, selectedService.name),
                    React.createElement("button", { style: styles.modalClose, onClick: () => setSelectedService(null) }, "\u00D7")),
                React.createElement("div", { style: styles.modalBody },
                    React.createElement("div", { style: styles.detailRow },
                        React.createElement("strong", null, "Status:"),
                        " ",
                        getStatusLabel(selectedService.status)),
                    React.createElement("div", { style: styles.detailRow },
                        React.createElement("strong", null, "Response Time:"),
                        " ",
                        selectedService.responseTime,
                        "ms"),
                    React.createElement("div", { style: styles.detailRow },
                        React.createElement("strong", null, "Uptime:"),
                        " ",
                        selectedService.uptime.toFixed(3),
                        "%"),
                    React.createElement("div", { style: styles.detailRow },
                        React.createElement("strong", null, "CPU:"),
                        " ",
                        selectedService.metrics.cpu.toFixed(1),
                        "%"),
                    React.createElement("div", { style: styles.detailRow },
                        React.createElement("strong", null, "Memory:"),
                        " ",
                        selectedService.metrics.memory.toFixed(1),
                        "%"),
                    React.createElement("div", { style: styles.detailRow },
                        React.createElement("strong", null, "Disk:"),
                        " ",
                        selectedService.metrics.disk.toFixed(1),
                        "%"),
                    React.createElement("div", { style: styles.detailRow },
                        React.createElement("strong", null, "Request Rate:"),
                        " ",
                        selectedService.metrics.requestRate,
                        "/s"),
                    React.createElement("div", { style: styles.detailRow },
                        React.createElement("strong", null, "Error Rate:"),
                        " ",
                        selectedService.metrics.errorRate.toFixed(2),
                        "%")))))));
};
const styles = {
    container: {
        padding: '24px',
        fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif'
    },
    loading: {
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '48px',
        color: '#6b7280'
    },
    spinner: {
        width: '40px',
        height: '40px',
        border: '4px solid #e5e7eb',
        borderTopColor: '#3b82f6',
        borderRadius: '50%',
        animation: 'spin 1s linear infinite'
    },
    header: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '24px'
    },
    title: {
        fontSize: '24px',
        fontWeight: 700,
        color: '#111827',
        margin: 0
    },
    headerActions: {
        display: 'flex',
        gap: '12px'
    },
    button: {
        padding: '8px 16px',
        backgroundColor: '#3b82f6',
        color: '#fff',
        border: 'none',
        borderRadius: '6px',
        fontSize: '14px',
        fontWeight: 500,
        cursor: 'pointer',
        transition: 'background-color 0.2s'
    },
    error: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '12px 16px',
        backgroundColor: '#fef2f2',
        border: '1px solid #fecaca',
        borderRadius: '8px',
        color: '#991b1b',
        marginBottom: '16px'
    },
    errorClose: {
        background: 'none',
        border: 'none',
        fontSize: '24px',
        cursor: 'pointer',
        color: '#991b1b'
    },
    filters: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '24px',
        gap: '16px',
        flexWrap: 'wrap'
    },
    statusFilters: {
        display: 'flex',
        gap: '8px',
        flexWrap: 'wrap'
    },
    filterButton: {
        padding: '6px 12px',
        backgroundColor: '#fff',
        border: '1px solid #e5e7eb',
        borderRadius: '6px',
        fontSize: '13px',
        fontWeight: 500,
        color: '#6b7280',
        cursor: 'pointer',
        display: 'flex',
        alignItems: 'center',
        gap: '6px',
        transition: 'all 0.2s'
    },
    filterButtonActive: {
        backgroundColor: '#3b82f6',
        color: '#fff',
        borderColor: '#3b82f6'
    },
    filterCount: {
        fontSize: '11px',
        padding: '2px 6px',
        backgroundColor: 'rgba(0, 0, 0, 0.1)',
        borderRadius: '10px'
    },
    searchInput: {
        padding: '8px 12px',
        border: '1px solid #e5e7eb',
        borderRadius: '6px',
        fontSize: '14px',
        minWidth: '200px',
        outline: 'none'
    },
    grid: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fill, minmax(350px, 1fr))',
        gap: '16px'
    },
    card: {
        backgroundColor: '#fff',
        borderRadius: '8px',
        border: '1px solid #e5e7eb',
        overflow: 'hidden',
        cursor: 'pointer',
        transition: 'box-shadow 0.2s',
        position: 'relative'
    },
    statusBar: {
        height: '4px'
    },
    cardContent: {
        padding: '16px'
    },
    cardHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '12px'
    },
    cardTitle: {
        display: 'flex',
        alignItems: 'center',
        gap: '8px'
    },
    statusIcon: {
        fontSize: '18px',
        fontWeight: 'bold'
    },
    serviceName: {
        fontSize: '16px',
        fontWeight: 600,
        color: '#111827'
    },
    checkButton: {
        padding: '4px 12px',
        backgroundColor: '#f3f4f6',
        border: '1px solid #e5e7eb',
        borderRadius: '4px',
        fontSize: '12px',
        fontWeight: 500,
        cursor: 'pointer',
        color: '#374151'
    },
    message: {
        fontSize: '13px',
        color: '#6b7280',
        marginBottom: '12px',
        fontStyle: 'italic'
    },
    metrics: {
        display: 'grid',
        gridTemplateColumns: '1fr 1fr',
        gap: '8px',
        marginBottom: '12px'
    },
    metric: {
        fontSize: '13px',
        display: 'flex',
        justifyContent: 'space-between'
    },
    metricLabel: {
        color: '#6b7280'
    },
    metricValue: {
        color: '#111827',
        fontWeight: 500
    },
    dependencies: {
        marginTop: '12px',
        paddingTop: '12px',
        borderTop: '1px solid #e5e7eb'
    },
    dependenciesLabel: {
        fontSize: '12px',
        color: '#6b7280',
        fontWeight: 500,
        display: 'block',
        marginBottom: '6px'
    },
    dependenciesList: {
        display: 'flex',
        flexWrap: 'wrap',
        gap: '4px'
    },
    dependency: {
        fontSize: '11px',
        padding: '2px 8px',
        backgroundColor: '#f3f4f6',
        borderRadius: '4px',
        color: '#374151'
    },
    config: {
        marginTop: '12px',
        paddingTop: '12px',
        borderTop: '1px solid #e5e7eb'
    },
    configTitle: {
        fontSize: '12px',
        fontWeight: 600,
        color: '#111827',
        marginBottom: '8px'
    },
    configDetails: {
        fontSize: '12px',
        color: '#6b7280',
        display: 'grid',
        gap: '4px'
    },
    emptyState: {
        textAlign: 'center',
        padding: '48px',
        color: '#6b7280'
    },
    modal: {
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: 'rgba(0, 0, 0, 0.5)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 1000
    },
    modalContent: {
        backgroundColor: '#fff',
        borderRadius: '12px',
        maxWidth: '500px',
        width: '90%',
        maxHeight: '80vh',
        overflow: 'auto'
    },
    modalHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '20px',
        borderBottom: '1px solid #e5e7eb'
    },
    modalClose: {
        background: 'none',
        border: 'none',
        fontSize: '32px',
        cursor: 'pointer',
        color: '#6b7280'
    },
    modalBody: {
        padding: '20px'
    },
    detailRow: {
        padding: '8px 0',
        borderBottom: '1px solid #f3f4f6',
        fontSize: '14px'
    }
};
export default HealthChecks;
//# sourceMappingURL=HealthChecks.js.map