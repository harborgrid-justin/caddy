import React, { useEffect, useState } from 'react';
import { AlertSeverity, AlertState } from './types';
export const AlertHistory = ({ service, timeRange = { from: new Date(Date.now() - 86400000), to: new Date(), quick: '24h' }, severities, className = '' }) => {
    const [alerts, setAlerts] = useState([]);
    const [stats, setStats] = useState(null);
    const [loading, setLoading] = useState(true);
    const [filterState, setFilterState] = useState('all');
    const [filterSeverity, setFilterSeverity] = useState('all');
    const [searchQuery, setSearchQuery] = useState('');
    const [selectedAlert, setSelectedAlert] = useState(null);
    const [viewMode, setViewMode] = useState('timeline');
    useEffect(() => {
        fetchAlertHistory();
    }, [service, timeRange, severities]);
    useEffect(() => {
        if (alerts.length > 0) {
            calculateStats();
        }
    }, [alerts]);
    const fetchAlertHistory = async () => {
        try {
            setLoading(true);
            const params = new URLSearchParams({
                from: timeRange.from.toISOString(),
                to: timeRange.to.toISOString(),
                ...(service && { service }),
                ...(severities && { severities: severities.join(',') })
            });
            const response = await fetch(`/api/monitoring/alerts/history?${params}`);
            if (!response.ok)
                throw new Error('Failed to fetch alert history');
            const data = await response.json();
            setAlerts(data);
        }
        catch (error) {
            console.error('[AlertHistory] Failed to fetch alerts:', error);
        }
        finally {
            setLoading(false);
        }
    };
    const calculateStats = () => {
        const bySeverity = alerts.reduce((acc, alert) => {
            acc[alert.severity] = (acc[alert.severity] || 0) + 1;
            return acc;
        }, {});
        const byState = alerts.reduce((acc, alert) => {
            acc[alert.state] = (acc[alert.state] || 0) + 1;
            return acc;
        }, {});
        const resolvedAlerts = alerts.filter(a => a.resolvedAt);
        const totalResolutionTime = resolvedAlerts.reduce((sum, alert) => {
            if (alert.resolvedAt) {
                return sum + (new Date(alert.resolvedAt).getTime() - new Date(alert.triggeredAt).getTime());
            }
            return sum;
        }, 0);
        const averageResolutionTime = resolvedAlerts.length > 0
            ? totalResolutionTime / resolvedAlerts.length
            : 0;
        const alertCounts = alerts.reduce((acc, alert) => {
            acc[alert.name] = (acc[alert.name] || 0) + 1;
            return acc;
        }, {});
        const mostCommonAlert = Object.entries(alertCounts).sort((a, b) => b[1] - a[1])[0]?.[0] || '';
        setStats({
            total: alerts.length,
            bySeverity,
            byState,
            averageResolutionTime,
            mostCommonAlert
        });
    };
    const acknowledgeAlert = async (alertId) => {
        try {
            const response = await fetch(`/api/monitoring/alerts/${alertId}/acknowledge`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ acknowledgedBy: 'current-user' })
            });
            if (!response.ok)
                throw new Error('Failed to acknowledge alert');
            const updatedAlert = await response.json();
            setAlerts(prev => prev.map(a => a.id === alertId ? updatedAlert : a));
        }
        catch (error) {
            console.error('[AlertHistory] Failed to acknowledge alert:', error);
        }
    };
    const resolveAlert = async (alertId) => {
        try {
            const response = await fetch(`/api/monitoring/alerts/${alertId}/resolve`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ resolvedBy: 'current-user' })
            });
            if (!response.ok)
                throw new Error('Failed to resolve alert');
            const updatedAlert = await response.json();
            setAlerts(prev => prev.map(a => a.id === alertId ? updatedAlert : a));
        }
        catch (error) {
            console.error('[AlertHistory] Failed to resolve alert:', error);
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
                return '#9ca3af';
        }
    };
    const getStateIcon = (state) => {
        switch (state) {
            case AlertState.ACTIVE:
                return '🔴';
            case AlertState.ACKNOWLEDGED:
                return '🟡';
            case AlertState.RESOLVED:
                return '🟢';
            case AlertState.SILENCED:
                return '🔇';
            default:
                return '⚪';
        }
    };
    const formatDuration = (ms) => {
        const seconds = Math.floor(ms / 1000);
        const minutes = Math.floor(seconds / 60);
        const hours = Math.floor(minutes / 60);
        const days = Math.floor(hours / 24);
        if (days > 0)
            return `${days}d ${hours % 24}h`;
        if (hours > 0)
            return `${hours}h ${minutes % 60}m`;
        if (minutes > 0)
            return `${minutes}m ${seconds % 60}s`;
        return `${seconds}s`;
    };
    const filteredAlerts = alerts.filter(alert => {
        const matchesState = filterState === 'all' || alert.state === filterState;
        const matchesSeverity = filterSeverity === 'all' || alert.severity === filterSeverity;
        const matchesSearch = alert.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
            alert.message.toLowerCase().includes(searchQuery.toLowerCase()) ||
            alert.service.toLowerCase().includes(searchQuery.toLowerCase());
        return matchesState && matchesSeverity && matchesSearch;
    });
    const groupAlertsByDate = (alerts) => {
        return alerts.reduce((acc, alert) => {
            const date = new Date(alert.triggeredAt).toLocaleDateString();
            if (!acc[date]) {
                acc[date] = [];
            }
            acc[date].push(alert);
            return acc;
        }, {});
    };
    const groupedAlerts = groupAlertsByDate(filteredAlerts);
    if (loading) {
        return (React.createElement("div", { style: styles.loading },
            React.createElement("div", { style: styles.spinner }),
            React.createElement("p", null, "Loading alert history...")));
    }
    return (React.createElement("div", { className: `alert-history ${className}`, style: styles.container },
        React.createElement("div", { style: styles.header },
            React.createElement("h2", { style: styles.title }, "Alert History"),
            React.createElement("div", { style: styles.viewToggle },
                React.createElement("button", { style: {
                        ...styles.viewButton,
                        ...(viewMode === 'timeline' ? styles.viewButtonActive : {})
                    }, onClick: () => setViewMode('timeline') }, "Timeline"),
                React.createElement("button", { style: {
                        ...styles.viewButton,
                        ...(viewMode === 'list' ? styles.viewButtonActive : {})
                    }, onClick: () => setViewMode('list') }, "List"))),
        stats && (React.createElement("div", { style: styles.stats },
            React.createElement("div", { style: styles.statCard },
                React.createElement("div", { style: styles.statValue }, stats.total),
                React.createElement("div", { style: styles.statLabel }, "Total Alerts")),
            React.createElement("div", { style: styles.statCard },
                React.createElement("div", { style: { ...styles.statValue, color: '#dc2626' } }, stats.bySeverity[AlertSeverity.CRITICAL] || 0),
                React.createElement("div", { style: styles.statLabel }, "Critical")),
            React.createElement("div", { style: styles.statCard },
                React.createElement("div", { style: { ...styles.statValue, color: '#10b981' } }, stats.byState[AlertState.RESOLVED] || 0),
                React.createElement("div", { style: styles.statLabel }, "Resolved")),
            React.createElement("div", { style: styles.statCard },
                React.createElement("div", { style: styles.statValue }, formatDuration(stats.averageResolutionTime)),
                React.createElement("div", { style: styles.statLabel }, "Avg Resolution")))),
        React.createElement("div", { style: styles.filters },
            React.createElement("input", { type: "text", placeholder: "Search alerts...", value: searchQuery, onChange: (e) => setSearchQuery(e.target.value), style: styles.searchInput }),
            React.createElement("select", { value: filterSeverity, onChange: (e) => setFilterSeverity(e.target.value), style: styles.select },
                React.createElement("option", { value: "all" }, "All Severities"),
                Object.values(AlertSeverity).map(sev => (React.createElement("option", { key: sev, value: sev }, sev.toUpperCase())))),
            React.createElement("select", { value: filterState, onChange: (e) => setFilterState(e.target.value), style: styles.select },
                React.createElement("option", { value: "all" }, "All States"),
                Object.values(AlertState).map(state => (React.createElement("option", { key: state, value: state }, state.toUpperCase())))),
            React.createElement("button", { style: styles.refreshButton, onClick: fetchAlertHistory }, "Refresh")),
        filteredAlerts.length === 0 ? (React.createElement("div", { style: styles.emptyState },
            React.createElement("p", null, "No alerts found matching your criteria"))) : viewMode === 'timeline' ? (React.createElement("div", { style: styles.timeline }, Object.entries(groupedAlerts).map(([date, dateAlerts]) => (React.createElement("div", { key: date, style: styles.timelineGroup },
            React.createElement("div", { style: styles.timelineDate }, date),
            dateAlerts.map(alert => (React.createElement("div", { key: alert.id, style: styles.timelineItem, onClick: () => setSelectedAlert(alert) },
                React.createElement("div", { style: {
                        ...styles.timelineDot,
                        backgroundColor: getSeverityColor(alert.severity)
                    } }),
                React.createElement("div", { style: styles.timelineContent },
                    React.createElement("div", { style: styles.timelineHeader },
                        React.createElement("span", { style: styles.timelineTime }, new Date(alert.triggeredAt).toLocaleTimeString()),
                        React.createElement("span", { style: styles.timelineState },
                            getStateIcon(alert.state),
                            " ",
                            alert.state)),
                    React.createElement("div", { style: styles.timelineTitle }, alert.name),
                    React.createElement("div", { style: styles.timelineMessage }, alert.message),
                    React.createElement("div", { style: styles.timelineFooter },
                        React.createElement("span", { style: styles.timelineService }, alert.service),
                        alert.resolvedAt && (React.createElement("span", { style: styles.timelineDuration },
                            "Resolved in ",
                            formatDuration(new Date(alert.resolvedAt).getTime() - new Date(alert.triggeredAt).getTime()))))))))))))) : (React.createElement("div", { style: styles.list }, filteredAlerts.map(alert => (React.createElement("div", { key: alert.id, style: styles.listItem, onClick: () => setSelectedAlert(alert) },
            React.createElement("div", { style: {
                    ...styles.listIndicator,
                    backgroundColor: getSeverityColor(alert.severity)
                } }),
            React.createElement("div", { style: styles.listContent },
                React.createElement("div", { style: styles.listHeader },
                    React.createElement("div", null,
                        React.createElement("span", { style: styles.listTitle }, alert.name),
                        React.createElement("span", { style: styles.listState },
                            getStateIcon(alert.state),
                            " ",
                            alert.state)),
                    React.createElement("div", { style: styles.listActions }, alert.state === AlertState.ACTIVE && (React.createElement(React.Fragment, null,
                        React.createElement("button", { style: styles.actionButton, onClick: (e) => {
                                e.stopPropagation();
                                acknowledgeAlert(alert.id);
                            } }, "Acknowledge"),
                        React.createElement("button", { style: styles.actionButton, onClick: (e) => {
                                e.stopPropagation();
                                resolveAlert(alert.id);
                            } }, "Resolve"))))),
                React.createElement("div", { style: styles.listMessage }, alert.message),
                React.createElement("div", { style: styles.listFooter },
                    React.createElement("span", null, alert.service),
                    React.createElement("span", null, new Date(alert.triggeredAt).toLocaleString()),
                    alert.resolvedAt && (React.createElement("span", { style: styles.resolved },
                        "Resolved: ",
                        new Date(alert.resolvedAt).toLocaleString()))))))))),
        selectedAlert && (React.createElement("div", { style: styles.modal, onClick: () => setSelectedAlert(null) },
            React.createElement("div", { style: styles.modalContent, onClick: (e) => e.stopPropagation() },
                React.createElement("div", { style: styles.modalHeader },
                    React.createElement("h3", null, selectedAlert.name),
                    React.createElement("button", { style: styles.modalClose, onClick: () => setSelectedAlert(null) }, "\u00D7")),
                React.createElement("div", { style: styles.modalBody },
                    React.createElement("div", { style: styles.detailRow },
                        React.createElement("strong", null, "Severity:"),
                        React.createElement("span", { style: {
                                color: getSeverityColor(selectedAlert.severity),
                                fontWeight: 600
                            } }, selectedAlert.severity.toUpperCase())),
                    React.createElement("div", { style: styles.detailRow },
                        React.createElement("strong", null, "State:"),
                        " ",
                        getStateIcon(selectedAlert.state),
                        " ",
                        selectedAlert.state),
                    React.createElement("div", { style: styles.detailRow },
                        React.createElement("strong", null, "Service:"),
                        " ",
                        selectedAlert.service),
                    React.createElement("div", { style: styles.detailRow },
                        React.createElement("strong", null, "Message:"),
                        " ",
                        selectedAlert.message),
                    React.createElement("div", { style: styles.detailRow },
                        React.createElement("strong", null, "Triggered:"),
                        " ",
                        new Date(selectedAlert.triggeredAt).toLocaleString()),
                    selectedAlert.acknowledgedAt && (React.createElement("div", { style: styles.detailRow },
                        React.createElement("strong", null, "Acknowledged:"),
                        " ",
                        new Date(selectedAlert.acknowledgedAt).toLocaleString(),
                        selectedAlert.acknowledgedBy && ` by ${selectedAlert.acknowledgedBy}`)),
                    selectedAlert.resolvedAt && (React.createElement("div", { style: styles.detailRow },
                        React.createElement("strong", null, "Resolved:"),
                        " ",
                        new Date(selectedAlert.resolvedAt).toLocaleString(),
                        selectedAlert.resolvedBy && ` by ${selectedAlert.resolvedBy}`)),
                    selectedAlert.threshold && (React.createElement("div", { style: styles.detailRow },
                        React.createElement("strong", null, "Threshold:"),
                        " ",
                        selectedAlert.threshold.metric,
                        " ",
                        selectedAlert.threshold.operator,
                        " ",
                        selectedAlert.threshold.value)),
                    Object.keys(selectedAlert.metadata).length > 0 && (React.createElement("div", { style: styles.detailRow },
                        React.createElement("strong", null, "Metadata:"),
                        React.createElement("pre", { style: styles.metadata }, JSON.stringify(selectedAlert.metadata, null, 2))))))))));
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
    viewToggle: {
        display: 'flex',
        gap: '4px',
        backgroundColor: '#f3f4f6',
        borderRadius: '8px',
        padding: '4px'
    },
    viewButton: {
        padding: '6px 16px',
        border: 'none',
        background: 'transparent',
        borderRadius: '6px',
        fontSize: '14px',
        fontWeight: 500,
        color: '#6b7280',
        cursor: 'pointer',
        transition: 'all 0.2s'
    },
    viewButtonActive: {
        backgroundColor: '#fff',
        color: '#111827'
    },
    stats: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))',
        gap: '16px',
        marginBottom: '24px'
    },
    statCard: {
        backgroundColor: '#fff',
        border: '1px solid #e5e7eb',
        borderRadius: '8px',
        padding: '20px',
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
    filters: {
        display: 'flex',
        gap: '12px',
        marginBottom: '24px',
        flexWrap: 'wrap'
    },
    searchInput: {
        flex: 1,
        minWidth: '200px',
        padding: '8px 12px',
        border: '1px solid #e5e7eb',
        borderRadius: '6px',
        fontSize: '14px',
        outline: 'none'
    },
    select: {
        padding: '8px 12px',
        border: '1px solid #e5e7eb',
        borderRadius: '6px',
        fontSize: '14px',
        outline: 'none',
        backgroundColor: '#fff'
    },
    refreshButton: {
        padding: '8px 16px',
        backgroundColor: '#3b82f6',
        color: '#fff',
        border: 'none',
        borderRadius: '6px',
        fontSize: '14px',
        fontWeight: 500,
        cursor: 'pointer'
    },
    emptyState: {
        textAlign: 'center',
        padding: '48px',
        backgroundColor: '#fff',
        border: '1px solid #e5e7eb',
        borderRadius: '8px',
        color: '#6b7280'
    },
    timeline: {
        display: 'flex',
        flexDirection: 'column',
        gap: '24px'
    },
    timelineGroup: {},
    timelineDate: {
        fontSize: '14px',
        fontWeight: 600,
        color: '#111827',
        marginBottom: '12px',
        padding: '8px 12px',
        backgroundColor: '#f3f4f6',
        borderRadius: '6px',
        display: 'inline-block'
    },
    timelineItem: {
        display: 'flex',
        gap: '16px',
        marginBottom: '12px',
        marginLeft: '20px',
        cursor: 'pointer',
        position: 'relative'
    },
    timelineDot: {
        width: '12px',
        height: '12px',
        borderRadius: '50%',
        marginTop: '6px',
        flexShrink: 0,
        position: 'relative',
        zIndex: 1
    },
    timelineContent: {
        flex: 1,
        backgroundColor: '#fff',
        border: '1px solid #e5e7eb',
        borderRadius: '8px',
        padding: '16px'
    },
    timelineHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '8px'
    },
    timelineTime: {
        fontSize: '12px',
        color: '#6b7280',
        fontWeight: 500
    },
    timelineState: {
        fontSize: '12px',
        color: '#6b7280',
        textTransform: 'uppercase'
    },
    timelineTitle: {
        fontSize: '16px',
        fontWeight: 600,
        color: '#111827',
        marginBottom: '4px'
    },
    timelineMessage: {
        fontSize: '14px',
        color: '#4b5563',
        marginBottom: '8px'
    },
    timelineFooter: {
        display: 'flex',
        justifyContent: 'space-between',
        fontSize: '12px',
        color: '#6b7280'
    },
    timelineService: {
        backgroundColor: '#f3f4f6',
        padding: '2px 8px',
        borderRadius: '4px'
    },
    timelineDuration: {},
    list: {
        display: 'flex',
        flexDirection: 'column',
        gap: '12px'
    },
    listItem: {
        display: 'flex',
        backgroundColor: '#fff',
        border: '1px solid #e5e7eb',
        borderRadius: '8px',
        overflow: 'hidden',
        cursor: 'pointer',
        transition: 'box-shadow 0.2s'
    },
    listIndicator: {
        width: '4px',
        flexShrink: 0
    },
    listContent: {
        flex: 1,
        padding: '16px'
    },
    listHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '8px'
    },
    listTitle: {
        fontSize: '16px',
        fontWeight: 600,
        color: '#111827',
        marginRight: '12px'
    },
    listState: {
        fontSize: '12px',
        color: '#6b7280',
        textTransform: 'uppercase'
    },
    listActions: {
        display: 'flex',
        gap: '8px'
    },
    actionButton: {
        padding: '4px 12px',
        backgroundColor: '#f3f4f6',
        border: '1px solid #e5e7eb',
        borderRadius: '4px',
        fontSize: '12px',
        fontWeight: 500,
        cursor: 'pointer',
        color: '#374151'
    },
    listMessage: {
        fontSize: '14px',
        color: '#4b5563',
        marginBottom: '8px'
    },
    listFooter: {
        display: 'flex',
        gap: '16px',
        fontSize: '12px',
        color: '#6b7280'
    },
    resolved: {
        color: '#10b981'
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
        maxWidth: '600px',
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
        padding: '12px 0',
        borderBottom: '1px solid #f3f4f6',
        fontSize: '14px',
        display: 'flex',
        flexDirection: 'column',
        gap: '4px'
    },
    metadata: {
        backgroundColor: '#f3f4f6',
        padding: '12px',
        borderRadius: '6px',
        fontSize: '12px',
        overflow: 'auto',
        marginTop: '8px'
    }
};
export default AlertHistory;
//# sourceMappingURL=AlertHistory.js.map