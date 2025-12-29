import React, { useEffect, useState } from 'react';
export const ResourceUsage = ({ service, className = '' }) => {
    const [resources, setResources] = useState([]);
    const [selectedService, setSelectedService] = useState(service || null);
    const [ws, setWs] = useState(null);
    const [isConnected, setIsConnected] = useState(false);
    useEffect(() => {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/api/monitoring/resources/stream`;
        const socket = new WebSocket(wsUrl);
        socket.onopen = () => {
            console.log('[ResourceUsage] WebSocket connected');
            setIsConnected(true);
            socket.send(JSON.stringify({
                type: 'subscribe',
                service: selectedService
            }));
        };
        socket.onmessage = (event) => {
            try {
                const message = JSON.parse(event.data);
                if (message.type === 'metric') {
                    updateResourceData(message.data);
                }
            }
            catch (error) {
                console.error('[ResourceUsage] Failed to parse WebSocket message:', error);
            }
        };
        socket.onerror = (error) => {
            console.error('[ResourceUsage] WebSocket error:', error);
            setIsConnected(false);
        };
        socket.onclose = () => {
            console.log('[ResourceUsage] WebSocket disconnected');
            setIsConnected(false);
        };
        setWs(socket);
        return () => {
            socket.close();
        };
    }, [selectedService]);
    useEffect(() => {
        fetchResourceData();
    }, [selectedService]);
    const fetchResourceData = async () => {
        try {
            const params = selectedService ? `?service=${selectedService}` : '';
            const response = await fetch(`/api/monitoring/resources${params}`);
            if (!response.ok)
                throw new Error('Failed to fetch resource data');
            const data = await response.json();
            setResources(data);
        }
        catch (error) {
            console.error('[ResourceUsage] Failed to fetch resources:', error);
        }
    };
    const updateResourceData = (newData) => {
        setResources(prev => {
            const updated = [...prev];
            const index = updated.findIndex(r => r.service === newData.service);
            if (index >= 0) {
                updated[index] = newData;
            }
            else {
                updated.push(newData);
            }
            return updated;
        });
    };
    const getUsageColor = (percentage) => {
        if (percentage >= 90)
            return '#ef4444';
        if (percentage >= 75)
            return '#f59e0b';
        if (percentage >= 50)
            return '#3b82f6';
        return '#10b981';
    };
    const formatBytes = (bytes) => {
        const units = ['B', 'KB', 'MB', 'GB', 'TB'];
        let size = bytes;
        let unitIndex = 0;
        while (size >= 1024 && unitIndex < units.length - 1) {
            size /= 1024;
            unitIndex++;
        }
        return `${size.toFixed(2)} ${units[unitIndex]}`;
    };
    const formatBytesPerSec = (bytes) => {
        return `${formatBytes(bytes)}/s`;
    };
    const uniqueServices = Array.from(new Set(resources.map(r => r.service)));
    return (React.createElement("div", { className: `resource-usage ${className}`, style: styles.container },
        React.createElement("div", { style: styles.header },
            React.createElement("h2", { style: styles.title }, "Resource Usage"),
            React.createElement("div", { style: styles.connectionStatus },
                React.createElement("div", { style: {
                        ...styles.statusDot,
                        backgroundColor: isConnected ? '#10b981' : '#ef4444'
                    } }),
                React.createElement("span", { style: styles.statusText }, isConnected ? 'Live' : 'Disconnected'))),
        uniqueServices.length > 1 && (React.createElement("div", { style: styles.serviceFilter },
            React.createElement("button", { style: {
                    ...styles.filterButton,
                    ...(selectedService === null ? styles.filterButtonActive : {})
                }, onClick: () => setSelectedService(null) }, "All Services"),
            uniqueServices.map(svc => (React.createElement("button", { key: svc, style: {
                    ...styles.filterButton,
                    ...(selectedService === svc ? styles.filterButtonActive : {})
                }, onClick: () => setSelectedService(svc) }, svc))))),
        React.createElement("div", { style: styles.resourceGrid }, resources
            .filter(r => !selectedService || r.service === selectedService)
            .map((resource, index) => (React.createElement("div", { key: index, style: styles.resourceCard },
            React.createElement("div", { style: styles.resourceHeader },
                React.createElement("h3", { style: styles.resourceService }, resource.service),
                React.createElement("span", { style: styles.resourceTimestamp }, new Date(resource.timestamp).toLocaleTimeString())),
            React.createElement("div", { style: styles.metricSection },
                React.createElement("div", { style: styles.metricHeader },
                    React.createElement("span", { style: styles.metricLabel }, "CPU"),
                    React.createElement("span", { style: {
                            ...styles.metricValue,
                            color: getUsageColor(resource.cpu.percentage)
                        } },
                        resource.cpu.percentage.toFixed(1),
                        "%")),
                React.createElement("div", { style: styles.progressBar },
                    React.createElement("div", { style: {
                            ...styles.progressFill,
                            width: `${Math.min(resource.cpu.percentage, 100)}%`,
                            backgroundColor: getUsageColor(resource.cpu.percentage)
                        } })),
                React.createElement("div", { style: styles.metricDetails },
                    resource.cpu.used.toFixed(2),
                    " / ",
                    resource.cpu.total.toFixed(2),
                    " cores")),
            React.createElement("div", { style: styles.metricSection },
                React.createElement("div", { style: styles.metricHeader },
                    React.createElement("span", { style: styles.metricLabel }, "Memory"),
                    React.createElement("span", { style: {
                            ...styles.metricValue,
                            color: getUsageColor(resource.memory.percentage)
                        } },
                        resource.memory.percentage.toFixed(1),
                        "%")),
                React.createElement("div", { style: styles.progressBar },
                    React.createElement("div", { style: {
                            ...styles.progressFill,
                            width: `${Math.min(resource.memory.percentage, 100)}%`,
                            backgroundColor: getUsageColor(resource.memory.percentage)
                        } })),
                React.createElement("div", { style: styles.metricDetails },
                    formatBytes(resource.memory.used),
                    " / ",
                    formatBytes(resource.memory.total))),
            React.createElement("div", { style: styles.metricSection },
                React.createElement("div", { style: styles.metricHeader },
                    React.createElement("span", { style: styles.metricLabel }, "Disk"),
                    React.createElement("span", { style: {
                            ...styles.metricValue,
                            color: getUsageColor(resource.disk.percentage)
                        } },
                        resource.disk.percentage.toFixed(1),
                        "%")),
                React.createElement("div", { style: styles.progressBar },
                    React.createElement("div", { style: {
                            ...styles.progressFill,
                            width: `${Math.min(resource.disk.percentage, 100)}%`,
                            backgroundColor: getUsageColor(resource.disk.percentage)
                        } })),
                React.createElement("div", { style: styles.metricDetails },
                    formatBytes(resource.disk.used),
                    " / ",
                    formatBytes(resource.disk.total))),
            React.createElement("div", { style: styles.metricSection },
                React.createElement("div", { style: styles.metricHeader },
                    React.createElement("span", { style: styles.metricLabel }, "Network")),
                React.createElement("div", { style: styles.networkStats },
                    React.createElement("div", { style: styles.networkStat },
                        React.createElement("span", { style: styles.networkLabel }, "\u2193 In:"),
                        React.createElement("span", { style: styles.networkValue }, formatBytesPerSec(resource.network.bytesIn))),
                    React.createElement("div", { style: styles.networkStat },
                        React.createElement("span", { style: styles.networkLabel }, "\u2191 Out:"),
                        React.createElement("span", { style: styles.networkValue }, formatBytesPerSec(resource.network.bytesOut))),
                    React.createElement("div", { style: styles.networkStat },
                        React.createElement("span", { style: styles.networkLabel }, "Packets In:"),
                        React.createElement("span", { style: styles.networkValue },
                            resource.network.packetsIn.toLocaleString(),
                            "/s")),
                    React.createElement("div", { style: styles.networkStat },
                        React.createElement("span", { style: styles.networkLabel }, "Packets Out:"),
                        React.createElement("span", { style: styles.networkValue },
                            resource.network.packetsOut.toLocaleString(),
                            "/s")),
                    (resource.network.errorsIn > 0 || resource.network.errorsOut > 0) && (React.createElement("div", { style: styles.networkErrors },
                        React.createElement("span", { style: { color: '#ef4444' } },
                            "\u26A0 Errors: ",
                            resource.network.errorsIn + resource.network.errorsOut))))))))),
        resources.length > 1 && (React.createElement("div", { style: styles.summary },
            React.createElement("h3", { style: styles.summaryTitle }, "Overall Summary"),
            React.createElement("div", { style: styles.summaryGrid },
                React.createElement("div", { style: styles.summaryCard },
                    React.createElement("div", { style: styles.summaryLabel }, "Avg CPU"),
                    React.createElement("div", { style: {
                            ...styles.summaryValue,
                            color: getUsageColor(resources.reduce((sum, r) => sum + r.cpu.percentage, 0) / resources.length)
                        } },
                        (resources.reduce((sum, r) => sum + r.cpu.percentage, 0) / resources.length).toFixed(1),
                        "%")),
                React.createElement("div", { style: styles.summaryCard },
                    React.createElement("div", { style: styles.summaryLabel }, "Avg Memory"),
                    React.createElement("div", { style: {
                            ...styles.summaryValue,
                            color: getUsageColor(resources.reduce((sum, r) => sum + r.memory.percentage, 0) / resources.length)
                        } },
                        (resources.reduce((sum, r) => sum + r.memory.percentage, 0) / resources.length).toFixed(1),
                        "%")),
                React.createElement("div", { style: styles.summaryCard },
                    React.createElement("div", { style: styles.summaryLabel }, "Avg Disk"),
                    React.createElement("div", { style: {
                            ...styles.summaryValue,
                            color: getUsageColor(resources.reduce((sum, r) => sum + r.disk.percentage, 0) / resources.length)
                        } },
                        (resources.reduce((sum, r) => sum + r.disk.percentage, 0) / resources.length).toFixed(1),
                        "%")),
                React.createElement("div", { style: styles.summaryCard },
                    React.createElement("div", { style: styles.summaryLabel }, "Total Network"),
                    React.createElement("div", { style: styles.summaryValue },
                        "\u2193 ",
                        formatBytesPerSec(resources.reduce((sum, r) => sum + r.network.bytesIn, 0)),
                        React.createElement("br", null),
                        "\u2191 ",
                        formatBytesPerSec(resources.reduce((sum, r) => sum + r.network.bytesOut, 0)))))))));
};
const styles = {
    container: {
        padding: '24px',
        fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif'
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
    connectionStatus: {
        display: 'flex',
        alignItems: 'center',
        gap: '8px'
    },
    statusDot: {
        width: '8px',
        height: '8px',
        borderRadius: '50%'
    },
    statusText: {
        fontSize: '13px',
        color: '#6b7280',
        fontWeight: 500
    },
    serviceFilter: {
        display: 'flex',
        gap: '8px',
        marginBottom: '24px',
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
        transition: 'all 0.2s'
    },
    filterButtonActive: {
        backgroundColor: '#3b82f6',
        color: '#fff',
        borderColor: '#3b82f6'
    },
    resourceGrid: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fill, minmax(400px, 1fr))',
        gap: '20px',
        marginBottom: '32px'
    },
    resourceCard: {
        backgroundColor: '#fff',
        border: '1px solid #e5e7eb',
        borderRadius: '12px',
        padding: '24px'
    },
    resourceHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '20px',
        paddingBottom: '12px',
        borderBottom: '2px solid #e5e7eb'
    },
    resourceService: {
        fontSize: '18px',
        fontWeight: 600,
        color: '#111827',
        margin: 0
    },
    resourceTimestamp: {
        fontSize: '12px',
        color: '#9ca3af'
    },
    metricSection: {
        marginBottom: '20px'
    },
    metricHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '8px'
    },
    metricLabel: {
        fontSize: '14px',
        fontWeight: 600,
        color: '#374151'
    },
    metricValue: {
        fontSize: '20px',
        fontWeight: 700
    },
    progressBar: {
        height: '10px',
        backgroundColor: '#f3f4f6',
        borderRadius: '5px',
        overflow: 'hidden',
        marginBottom: '6px'
    },
    progressFill: {
        height: '100%',
        borderRadius: '5px',
        transition: 'width 0.3s ease, background-color 0.3s ease'
    },
    metricDetails: {
        fontSize: '12px',
        color: '#6b7280'
    },
    networkStats: {
        display: 'grid',
        gridTemplateColumns: '1fr 1fr',
        gap: '8px',
        marginTop: '8px'
    },
    networkStat: {
        display: 'flex',
        justifyContent: 'space-between',
        fontSize: '13px',
        padding: '6px 0'
    },
    networkLabel: {
        color: '#6b7280'
    },
    networkValue: {
        color: '#111827',
        fontWeight: 500
    },
    networkErrors: {
        gridColumn: '1 / -1',
        fontSize: '13px',
        fontWeight: 600,
        padding: '6px 12px',
        backgroundColor: '#fef2f2',
        borderRadius: '6px',
        marginTop: '4px'
    },
    summary: {
        backgroundColor: '#fff',
        border: '1px solid #e5e7eb',
        borderRadius: '12px',
        padding: '24px'
    },
    summaryTitle: {
        fontSize: '18px',
        fontWeight: 600,
        color: '#111827',
        marginBottom: '16px'
    },
    summaryGrid: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
        gap: '16px'
    },
    summaryCard: {
        padding: '16px',
        backgroundColor: '#f9fafb',
        borderRadius: '8px',
        textAlign: 'center'
    },
    summaryLabel: {
        fontSize: '13px',
        color: '#6b7280',
        fontWeight: 500,
        marginBottom: '8px'
    },
    summaryValue: {
        fontSize: '24px',
        fontWeight: 700,
        color: '#111827'
    }
};
export default ResourceUsage;
//# sourceMappingURL=ResourceUsage.js.map