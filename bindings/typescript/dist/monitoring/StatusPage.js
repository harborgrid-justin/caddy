import React, { useEffect, useState } from 'react';
import { ServiceStatus } from './types';
export const StatusPage = ({ configId, preview = false, className = '' }) => {
    const [config, setConfig] = useState(null);
    const [services, setServices] = useState([]);
    const [incidents, setIncidents] = useState([]);
    const [loading, setLoading] = useState(true);
    const [editMode, setEditMode] = useState(false);
    useEffect(() => {
        if (configId) {
            fetchStatusPageConfig();
            fetchServiceStatuses();
            fetchActiveIncidents();
        }
    }, [configId]);
    const fetchStatusPageConfig = async () => {
        try {
            setLoading(true);
            const response = await fetch(`/api/monitoring/status-page/${configId}`);
            if (!response.ok)
                throw new Error('Failed to fetch status page config');
            const data = await response.json();
            setConfig(data);
        }
        catch (error) {
            console.error('[StatusPage] Failed to fetch config:', error);
        }
        finally {
            setLoading(false);
        }
    };
    const fetchServiceStatuses = async () => {
        try {
            const response = await fetch('/api/monitoring/services');
            if (!response.ok)
                throw new Error('Failed to fetch service statuses');
            const data = await response.json();
            if (config) {
                const servicesWithStatus = config.services.map(svc => {
                    const serviceData = data.find((d) => d.id === svc.id);
                    return {
                        ...svc,
                        currentStatus: serviceData?.status || ServiceStatus.UNKNOWN
                    };
                });
                setServices(servicesWithStatus);
            }
        }
        catch (error) {
            console.error('[StatusPage] Failed to fetch service statuses:', error);
        }
    };
    const fetchActiveIncidents = async () => {
        try {
            const response = await fetch('/api/monitoring/incidents?status=active,investigating,identified,monitoring');
            if (!response.ok)
                throw new Error('Failed to fetch incidents');
            const data = await response.json();
            setIncidents(data);
        }
        catch (error) {
            console.error('[StatusPage] Failed to fetch incidents:', error);
        }
    };
    const updateConfig = async (updates) => {
        try {
            const response = await fetch(`/api/monitoring/status-page/${configId}`, {
                method: 'PATCH',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(updates)
            });
            if (!response.ok)
                throw new Error('Failed to update config');
            const updatedConfig = await response.json();
            setConfig(updatedConfig);
        }
        catch (error) {
            console.error('[StatusPage] Failed to update config:', error);
            alert('Failed to update status page configuration');
        }
    };
    const getStatusColor = (status) => {
        switch (status) {
            case ServiceStatus.HEALTHY:
                return config?.theme.primaryColor || '#10b981';
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
        const labels = {
            [ServiceStatus.HEALTHY]: 'Operational',
            [ServiceStatus.DEGRADED]: 'Degraded Performance',
            [ServiceStatus.DOWN]: 'Major Outage',
            [ServiceStatus.MAINTENANCE]: 'Maintenance',
            [ServiceStatus.UNKNOWN]: 'Unknown'
        };
        return labels[status];
    };
    const getOverallStatus = () => {
        const statuses = services.map(s => s.currentStatus);
        if (statuses.includes(ServiceStatus.DOWN)) {
            return {
                status: ServiceStatus.DOWN,
                message: 'Some systems are experiencing major outages'
            };
        }
        if (statuses.includes(ServiceStatus.DEGRADED)) {
            return {
                status: ServiceStatus.DEGRADED,
                message: 'Some systems are experiencing degraded performance'
            };
        }
        if (statuses.includes(ServiceStatus.MAINTENANCE)) {
            return {
                status: ServiceStatus.MAINTENANCE,
                message: 'Scheduled maintenance in progress'
            };
        }
        if (statuses.every(s => s === ServiceStatus.HEALTHY)) {
            return {
                status: ServiceStatus.HEALTHY,
                message: 'All systems operational'
            };
        }
        return {
            status: ServiceStatus.UNKNOWN,
            message: 'System status unknown'
        };
    };
    const groupServicesByGroup = () => {
        const grouped = {
            '': []
        };
        services.forEach(service => {
            const group = service.group || '';
            if (!grouped[group]) {
                grouped[group] = [];
            }
            grouped[group].push(service);
        });
        Object.keys(grouped).forEach(group => {
            grouped[group].sort((a, b) => a.displayOrder - b.displayOrder);
        });
        return grouped;
    };
    if (loading || !config) {
        return (React.createElement("div", { style: styles.loading },
            React.createElement("div", { style: styles.spinner }),
            React.createElement("p", null, "Loading status page...")));
    }
    const overall = getOverallStatus();
    const groupedServices = groupServicesByGroup();
    return (React.createElement("div", { className: `status-page ${className}`, style: {
            ...styles.container,
            backgroundColor: config.theme.backgroundColor,
            color: config.theme.textColor
        } },
        React.createElement("div", { style: styles.header },
            config.logo && (React.createElement("img", { src: config.logo, alt: config.title, style: styles.logo })),
            React.createElement("h1", { style: { ...styles.title, color: config.theme.textColor } }, config.title),
            config.description && (React.createElement("p", { style: { ...styles.description, color: config.theme.textColor } }, config.description)),
            !preview && (React.createElement("div", { style: styles.headerActions },
                React.createElement("button", { style: styles.editButton, onClick: () => setEditMode(!editMode) }, editMode ? 'View Mode' : 'Edit Mode')))),
        React.createElement("div", { style: {
                ...styles.overallStatus,
                borderColor: getStatusColor(overall.status)
            } },
            React.createElement("div", { style: {
                    ...styles.statusIndicator,
                    backgroundColor: getStatusColor(overall.status)
                } }),
            React.createElement("div", { style: styles.overallStatusContent },
                React.createElement("div", { style: { ...styles.overallStatusLabel, color: config.theme.textColor } }, overall.message),
                React.createElement("div", { style: styles.lastUpdated },
                    "Last updated: ",
                    new Date().toLocaleString()))),
        config.showIncidents && incidents.length > 0 && (React.createElement("div", { style: styles.section },
            React.createElement("h2", { style: { ...styles.sectionTitle, color: config.theme.textColor } }, "Active Incidents"),
            incidents.map(incident => (React.createElement("div", { key: incident.id, style: styles.incidentCard },
                React.createElement("div", { style: styles.incidentHeader },
                    React.createElement("h3", { style: styles.incidentTitle }, incident.title),
                    React.createElement("span", { style: styles.incidentStatus }, incident.status.replace('_', ' ').toUpperCase())),
                React.createElement("p", { style: styles.incidentDescription }, incident.description),
                React.createElement("div", { style: styles.incidentMeta },
                    React.createElement("span", null,
                        "Started: ",
                        new Date(incident.startedAt).toLocaleString()),
                    incident.affectedServices.length > 0 && (React.createElement("span", null,
                        "Affected: ",
                        incident.affectedServices.join(', ')))),
                incident.timeline.length > 0 && (React.createElement("div", { style: styles.incidentTimeline },
                    React.createElement("h4", { style: styles.timelineTitle }, "Updates:"),
                    incident.timeline.slice(-3).reverse().map(entry => (React.createElement("div", { key: entry.id, style: styles.timelineEntry },
                        React.createElement("div", { style: styles.timelineTime }, new Date(entry.timestamp).toLocaleString()),
                        React.createElement("div", { style: styles.timelineMessage }, entry.message))))))))))),
        React.createElement("div", { style: styles.section },
            React.createElement("h2", { style: { ...styles.sectionTitle, color: config.theme.textColor } }, "System Status"),
            Object.entries(groupedServices).map(([group, groupServices]) => (React.createElement("div", { key: group, style: styles.serviceGroup },
                group && (React.createElement("h3", { style: { ...styles.groupTitle, color: config.theme.textColor } }, group)),
                groupServices.map(service => (React.createElement("div", { key: service.id, style: styles.serviceRow },
                    React.createElement("div", { style: styles.serviceInfo },
                        React.createElement("div", { style: styles.serviceName }, service.name),
                        service.description && (React.createElement("div", { style: styles.serviceDescription }, service.description))),
                    React.createElement("div", { style: styles.serviceStatus },
                        React.createElement("div", { style: {
                                ...styles.statusDot,
                                backgroundColor: getStatusColor(service.currentStatus)
                            } }),
                        React.createElement("span", { style: styles.statusText }, getStatusLabel(service.currentStatus)))))))))),
        config.showMetrics && (React.createElement("div", { style: styles.section },
            React.createElement("h2", { style: { ...styles.sectionTitle, color: config.theme.textColor } }, "Performance Metrics"),
            React.createElement("div", { style: styles.metricsGrid },
                React.createElement("div", { style: styles.metricCard },
                    React.createElement("div", { style: styles.metricValue }, "99.98%"),
                    React.createElement("div", { style: styles.metricLabel }, "Uptime (30d)")),
                React.createElement("div", { style: styles.metricCard },
                    React.createElement("div", { style: styles.metricValue }, "124ms"),
                    React.createElement("div", { style: styles.metricLabel }, "Avg Response")),
                React.createElement("div", { style: styles.metricCard },
                    React.createElement("div", { style: styles.metricValue }, "0.01%"),
                    React.createElement("div", { style: styles.metricLabel }, "Error Rate"))))),
        React.createElement("div", { style: styles.footer },
            React.createElement("p", { style: styles.footerText },
                "Powered by CADDY v0.4.0",
                config.customDomain && ` • ${config.customDomain}`)),
        editMode && (React.createElement("div", { style: styles.editPanel },
            React.createElement("h3", { style: styles.editPanelTitle }, "Configuration"),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.label }, "Title"),
                React.createElement("input", { type: "text", value: config.title, onChange: (e) => updateConfig({ title: e.target.value }), style: styles.input })),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.label }, "Description"),
                React.createElement("textarea", { value: config.description, onChange: (e) => updateConfig({ description: e.target.value }), style: { ...styles.input, minHeight: '60px' } })),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.label }, "Public URL"),
                React.createElement("input", { type: "text", value: config.publicUrl, onChange: (e) => updateConfig({ publicUrl: e.target.value }), style: styles.input })),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.checkbox },
                    React.createElement("input", { type: "checkbox", checked: config.showMetrics, onChange: (e) => updateConfig({ showMetrics: e.target.checked }) }),
                    "Show Performance Metrics")),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.checkbox },
                    React.createElement("input", { type: "checkbox", checked: config.showIncidents, onChange: (e) => updateConfig({ showIncidents: e.target.checked }) }),
                    "Show Active Incidents")),
            React.createElement("div", { style: styles.themeSection },
                React.createElement("h4", { style: styles.themeSectionTitle }, "Theme"),
                React.createElement("div", { style: styles.colorInputs },
                    React.createElement("div", { style: styles.colorGroup },
                        React.createElement("label", { style: styles.label }, "Primary Color"),
                        React.createElement("input", { type: "color", value: config.theme.primaryColor, onChange: (e) => updateConfig({
                                theme: { ...config.theme, primaryColor: e.target.value }
                            }), style: styles.colorInput })),
                    React.createElement("div", { style: styles.colorGroup },
                        React.createElement("label", { style: styles.label }, "Background"),
                        React.createElement("input", { type: "color", value: config.theme.backgroundColor, onChange: (e) => updateConfig({
                                theme: { ...config.theme, backgroundColor: e.target.value }
                            }), style: styles.colorInput })),
                    React.createElement("div", { style: styles.colorGroup },
                        React.createElement("label", { style: styles.label }, "Text Color"),
                        React.createElement("input", { type: "color", value: config.theme.textColor, onChange: (e) => updateConfig({
                                theme: { ...config.theme, textColor: e.target.value }
                            }), style: styles.colorInput }))))))));
};
const styles = {
    container: {
        maxWidth: '900px',
        margin: '0 auto',
        padding: '40px 20px',
        fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
        minHeight: '100vh'
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
        textAlign: 'center',
        marginBottom: '40px'
    },
    logo: {
        maxHeight: '60px',
        marginBottom: '20px'
    },
    title: {
        fontSize: '36px',
        fontWeight: 700,
        margin: 0,
        marginBottom: '12px'
    },
    description: {
        fontSize: '16px',
        margin: 0,
        opacity: 0.8
    },
    headerActions: {
        marginTop: '20px'
    },
    editButton: {
        padding: '8px 16px',
        backgroundColor: '#3b82f6',
        color: '#fff',
        border: 'none',
        borderRadius: '6px',
        fontSize: '14px',
        fontWeight: 500,
        cursor: 'pointer'
    },
    overallStatus: {
        display: 'flex',
        alignItems: 'center',
        gap: '16px',
        padding: '24px',
        backgroundColor: 'rgba(255, 255, 255, 0.5)',
        borderRadius: '12px',
        marginBottom: '32px',
        borderLeft: '4px solid'
    },
    statusIndicator: {
        width: '16px',
        height: '16px',
        borderRadius: '50%'
    },
    overallStatusContent: {
        flex: 1
    },
    overallStatusLabel: {
        fontSize: '20px',
        fontWeight: 600,
        marginBottom: '4px'
    },
    lastUpdated: {
        fontSize: '13px',
        opacity: 0.7
    },
    section: {
        marginBottom: '40px'
    },
    sectionTitle: {
        fontSize: '24px',
        fontWeight: 600,
        marginBottom: '20px'
    },
    incidentCard: {
        backgroundColor: 'rgba(239, 68, 68, 0.1)',
        border: '1px solid #fecaca',
        borderRadius: '8px',
        padding: '20px',
        marginBottom: '16px'
    },
    incidentHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'flex-start',
        marginBottom: '12px'
    },
    incidentTitle: {
        fontSize: '18px',
        fontWeight: 600,
        margin: 0,
        color: '#991b1b'
    },
    incidentStatus: {
        fontSize: '11px',
        fontWeight: 600,
        padding: '4px 8px',
        backgroundColor: '#fee2e2',
        color: '#991b1b',
        borderRadius: '10px'
    },
    incidentDescription: {
        fontSize: '14px',
        marginBottom: '12px',
        color: '#7f1d1d'
    },
    incidentMeta: {
        fontSize: '12px',
        color: '#991b1b',
        display: 'flex',
        gap: '16px',
        marginBottom: '12px'
    },
    incidentTimeline: {
        paddingTop: '12px',
        borderTop: '1px solid #fecaca'
    },
    timelineTitle: {
        fontSize: '14px',
        fontWeight: 600,
        marginBottom: '8px',
        color: '#991b1b'
    },
    timelineEntry: {
        padding: '8px 0',
        fontSize: '13px',
        color: '#7f1d1d'
    },
    timelineTime: {
        fontWeight: 600,
        marginBottom: '2px'
    },
    timelineMessage: {},
    serviceGroup: {
        marginBottom: '24px'
    },
    groupTitle: {
        fontSize: '16px',
        fontWeight: 600,
        marginBottom: '12px',
        opacity: 0.8
    },
    serviceRow: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '16px',
        backgroundColor: 'rgba(255, 255, 255, 0.3)',
        borderRadius: '8px',
        marginBottom: '8px'
    },
    serviceInfo: {
        flex: 1
    },
    serviceName: {
        fontSize: '15px',
        fontWeight: 500,
        marginBottom: '4px'
    },
    serviceDescription: {
        fontSize: '13px',
        opacity: 0.7
    },
    serviceStatus: {
        display: 'flex',
        alignItems: 'center',
        gap: '8px'
    },
    statusDot: {
        width: '10px',
        height: '10px',
        borderRadius: '50%'
    },
    statusText: {
        fontSize: '14px',
        fontWeight: 500
    },
    metricsGrid: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
        gap: '16px'
    },
    metricCard: {
        backgroundColor: 'rgba(255, 255, 255, 0.3)',
        borderRadius: '8px',
        padding: '20px',
        textAlign: 'center'
    },
    metricValue: {
        fontSize: '32px',
        fontWeight: 700,
        marginBottom: '8px'
    },
    metricLabel: {
        fontSize: '14px',
        opacity: 0.7
    },
    footer: {
        textAlign: 'center',
        padding: '20px 0',
        borderTop: '1px solid rgba(0, 0, 0, 0.1)',
        marginTop: '40px'
    },
    footerText: {
        fontSize: '13px',
        opacity: 0.6,
        margin: 0
    },
    editPanel: {
        position: 'fixed',
        right: 0,
        top: 0,
        bottom: 0,
        width: '350px',
        backgroundColor: '#fff',
        boxShadow: '-2px 0 8px rgba(0, 0, 0, 0.1)',
        padding: '24px',
        overflowY: 'auto',
        zIndex: 1000
    },
    editPanelTitle: {
        fontSize: '18px',
        fontWeight: 600,
        marginBottom: '20px',
        color: '#111827'
    },
    formGroup: {
        marginBottom: '16px'
    },
    label: {
        display: 'block',
        fontSize: '13px',
        fontWeight: 500,
        color: '#374151',
        marginBottom: '6px'
    },
    input: {
        width: '100%',
        padding: '8px 12px',
        border: '1px solid #d1d5db',
        borderRadius: '6px',
        fontSize: '14px',
        outline: 'none',
        boxSizing: 'border-box'
    },
    checkbox: {
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        fontSize: '14px',
        color: '#374151',
        cursor: 'pointer'
    },
    themeSection: {
        paddingTop: '16px',
        borderTop: '1px solid #e5e7eb',
        marginTop: '16px'
    },
    themeSectionTitle: {
        fontSize: '14px',
        fontWeight: 600,
        color: '#111827',
        marginBottom: '12px'
    },
    colorInputs: {
        display: 'flex',
        flexDirection: 'column',
        gap: '12px'
    },
    colorGroup: {},
    colorInput: {
        width: '100%',
        height: '40px',
        border: '1px solid #d1d5db',
        borderRadius: '6px',
        cursor: 'pointer'
    }
};
export default StatusPage;
//# sourceMappingURL=StatusPage.js.map